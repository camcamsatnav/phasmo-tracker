const { app, BrowserWindow, ipcMain } = require("electron");
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const devServerUrl = "http://127.0.0.1:5173";
const isWindows = process.platform === "win32";
const binaryName = isWindows
  ? "phasmo_evidence_tracker.exe"
  : "phasmo_evidence_tracker";

let mainWindow = null;
let trackerProcess = null;
let stdoutBuffer = "";
let stderrBuffer = "";

function resolveRuntimeRoot() {
  const root = app.isPackaged ? app.getPath("userData") : path.resolve(__dirname, "..");
  if (app.isPackaged) {
    fs.mkdirSync(root, { recursive: true });
  }
  return root;
}

function resolveAppRoot() {
  return app.isPackaged ? app.getAppPath() : path.resolve(__dirname, "..");
}

function shouldLoadDist() {
  return app.isPackaged || process.argv.includes("--dist");
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1180,
    height: 760,
    minWidth: 940,
    minHeight: 620,
    backgroundColor: "#151512",
    title: "Phasmo Evidence Tracker",
    autoHideMenuBar: true,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, "preload.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  mainWindow.once("ready-to-show", () => {
    mainWindow?.show();
  });

  if (shouldLoadDist()) {
    mainWindow.loadFile(path.join(resolveAppRoot(), "dist", "index.html"));
  } else {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL ?? devServerUrl);
  }
}

function send(channel, payload) {
  for (const window of BrowserWindow.getAllWindows()) {
    window.webContents.send(channel, payload);
  }
}

function defaultTrackerPaths() {
  const repoRoot = resolveRuntimeRoot();
  return {
    configPath: path.join(repoRoot, "phasmo_tracker.toml"),
    ghostsPath: path.join(repoRoot, "phasmo_ghosts.toml"),
  };
}

function resolveTrackerCommand(options = {}) {
  const repoRoot = resolveRuntimeRoot();
  const configPath = options.configPath || defaultTrackerPaths().configPath;
  const ghostsPath = options.ghostsPath || defaultTrackerPaths().ghostsPath;
  const trackerArgs = ["--json", "--config", configPath, "--ghosts", ghostsPath];
  const override = process.env.PHASMO_TRACKER_BACKEND;

  if (override) {
    return {
      command: override,
      args: trackerArgs,
      cwd: repoRoot,
      label: override,
    };
  }

  const candidates = app.isPackaged
    ? [
        path.join(process.resourcesPath, "backend", binaryName),
        path.join(path.dirname(process.execPath), "backend", binaryName),
      ]
    : [
        path.join(repoRoot, "target", "release", binaryName),
        path.join(repoRoot, "target", "debug", binaryName),
      ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return {
        command: candidate,
        args: trackerArgs,
        cwd: repoRoot,
        label: candidate,
      };
    }
  }

  if (app.isPackaged) {
    throw new Error(
      `Packaged tracker backend not found. Expected ${candidates.join(" or ")}`,
    );
  }

  return {
    command: "cargo",
    args: ["run", "--quiet", "--", ...trackerArgs],
    cwd: repoRoot,
    label: "cargo run --quiet",
  };
}

function startTracker(options = {}) {
  if (trackerProcess) {
    return { running: true, reused: true };
  }

  const commandSpec = resolveTrackerCommand(options);
  stdoutBuffer = "";
  stderrBuffer = "";

  trackerProcess = spawn(commandSpec.command, commandSpec.args, {
    cwd: commandSpec.cwd,
    windowsHide: true,
    env: {
      ...process.env,
      RUST_BACKTRACE: process.env.RUST_BACKTRACE || "1",
    },
  });

  send("tracker:process", {
    running: true,
    command: commandSpec.label,
    pid: trackerProcess.pid,
  });

  trackerProcess.stdout.setEncoding("utf8");
  trackerProcess.stdout.on("data", (chunk) => {
    stdoutBuffer = readLines(stdoutBuffer + chunk, (line) => {
      if (!line.trim()) {
        return;
      }

      try {
        send("tracker:event", JSON.parse(line));
      } catch {
        send("tracker:log", { stream: "stdout", line });
      }
    });
  });

  trackerProcess.stderr.setEncoding("utf8");
  trackerProcess.stderr.on("data", (chunk) => {
    stderrBuffer = readLines(stderrBuffer + chunk, (line) => {
      if (line.trim()) {
        send("tracker:log", { stream: "stderr", line });
      }
    });
  });

  trackerProcess.on("error", (error) => {
    send("tracker:process", {
      running: false,
      error: error.message,
    });
    trackerProcess = null;
  });

  trackerProcess.on("exit", (code, signal) => {
    send("tracker:process", {
      running: false,
      code,
      signal,
    });
    trackerProcess = null;
  });

  return { running: true, reused: false };
}

function stopTracker() {
  if (!trackerProcess) {
    return { running: false };
  }

  const child = trackerProcess;
  trackerProcess = null;
  child.kill();
  return { running: false };
}

function readLines(buffer, onLine) {
  const lines = buffer.split(/\r?\n/);
  const rest = lines.pop() ?? "";
  for (const line of lines) {
    onLine(line);
  }
  return rest;
}

ipcMain.handle("tracker:start", (_event, options) => startTracker(options));
ipcMain.handle("tracker:stop", () => stopTracker());
ipcMain.handle("tracker:get-default-paths", () => defaultTrackerPaths());

app.whenReady().then(() => {
  createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on("before-quit", () => {
  stopTracker();
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
