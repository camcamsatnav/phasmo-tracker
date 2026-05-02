# Phasmophobia Evidence Tracker

Standalone Rust app that watches the Phasmophobia journal evidence page from outside the game process and logs evidence-state changes.

## Feasibility

Phasmophobia does not appear to expose an official gameplay API for reading journal evidence selections. This app stays external: it finds the visible game window, captures the window image, samples configured UI regions, and prints changes between `clear`, `selected`, and `rejected`.

Frames are ignored unless the Evidence journal page is visible, so normal gameplay pixels are not treated as evidence selections.

It does not inject into the game, read process memory, intercept network traffic, patch files, or decompile game code.

## Run

```powershell
cargo run
```

On first run, the app creates `phasmo_tracker.toml` if it does not already exist. Later runs reuse that same file.

Use a custom config path when needed:

```powershell
cargo run -- --config my-tracker.toml
```

Use a custom ghost evidence data file when needed:

```powershell
cargo run -- --ghosts my-ghosts.toml
```

## Desktop UI

The Electron frontend starts the same Rust tracker in JSON event mode and shows the live evidence state, selected/rejected evidence, and possible ghosts.

Install the desktop dependencies once:

```powershell
npm.cmd install
```

Run the Vite dev server and Electron app together:

```powershell
npm.cmd run dev
```

Build the frontend and open the desktop shell from the production bundle:

```powershell
npm.cmd run desktop
```

The JSON event stream is also available directly:

```powershell
cargo run -- --json
```

## Config

The config is persistent app state. It should usually be created once and left alone.

It may need to change if Phasmophobia changes the journal UI, if your game language moves the evidence labels/marks, or if a very different aspect ratio changes the journal layout. Normal resolution changes should work because regions are stored as normalized percentages of the captured window.

The default tracker cadence is tuned for quick journal interactions: `poll_ms = 10` and `stable_frames = 1`.

Ghost identification data is stored separately in `phasmo_ghosts.toml`. Each `[[ghosts]]` entry lists the evidence that can identify that ghost. `false_evidence` is supported for special cases such as The Mimic's ghost orbs.

If the tracker has active round state, the Evidence page disappears, and then the Evidence page later reappears fully clear, the app treats that as a game-over journal reset. It clears the current evidence selection before tracking the next round, which also clears the derived possible-ghost output.

Snapshots such as `phasmo-window.png` are only temporary calibration references. The tracker does not need them after the config is correct.
