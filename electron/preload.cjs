const { contextBridge, ipcRenderer } = require("electron");

function subscribe(channel, callback) {
  const listener = (_event, payload) => callback(payload);
  ipcRenderer.on(channel, listener);
  return () => ipcRenderer.removeListener(channel, listener);
}

contextBridge.exposeInMainWorld("tracker", {
  start: (options) => ipcRenderer.invoke("tracker:start", options),
  stop: () => ipcRenderer.invoke("tracker:stop"),
  getDefaultPaths: () => ipcRenderer.invoke("tracker:get-default-paths"),
  onEvent: (callback) => subscribe("tracker:event", callback),
  onLog: (callback) => subscribe("tracker:log", callback),
  onProcess: (callback) => subscribe("tracker:process", callback),
});
