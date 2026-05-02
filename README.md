# Phasmophobia Evidence Tracker

Standalone Rust experiment for observing the Phasmophobia journal evidence UI from outside the game process and logging state changes.

## Feasibility

There does not appear to be an official Phasmophobia gameplay API for reading journal evidence selections. The route implemented here is intentionally external and low-touch:

- find the visible game window by title
- require the owning app/process name to look like Phasmophobia, so unrelated browser tabs are ignored
- capture the window image
- sample calibrated UI regions for each evidence row
- print console logs when a row changes between `clear`, `selected`, and `rejected`

This project does not inject into the game, read process memory, intercept network traffic, patch files, or decompile game code. That is deliberate. Kinetic Games' published EULA and Code of Conduct restrict unauthorized game access/modification and list hacking/cheating and game-file tampering as bannable behavior.

## Current Limitations

The first version needs calibration for your resolution, display scaling, language, and current Phasmophobia UI. The included `phasmo_tracker.toml` contains starter regions only; expect to adjust it before relying on the output.

Window capture may not work with exclusive fullscreen. Use borderless windowed or windowed mode if snapshots are blank.

If `list-windows` shows the game with an unexpected app or title value, edit `window_title_contains` or `app_name_contains` in `phasmo_tracker.toml`.

Snapshots such as `phasmo-window.png` are only calibration references. Once `phasmo_tracker.toml` is correct for your setup, the tracker does not need the snapshot file.

## Commands

```powershell
cargo run -- list-windows
cargo run -- write-config --force
cargo run -- snapshot --output phasmo-window.png
cargo run -- inspect --image phasmo-window.png
cargo run -- probe --x 800 --y 420 --radius 3
cargo run -- run
cargo run -- run --once
```

## Calibration Flow

1. Start Phasmophobia in windowed or borderless windowed mode.
2. Open the in-game journal/evidence page.
3. Run `cargo run -- snapshot --output phasmo-window.png`.
4. Run `cargo run -- inspect --image phasmo-window.png` and confirm the current all-empty journal reads as `clear`.
5. Open `phasmo-window.png` and find the small visual areas that change when an evidence item is selected or rejected.
6. Use `probe` on those screenshot coordinates to get normalized coordinates and RGB hints.
7. Update the matching `selected` and `rejected` blocks in `phasmo_tracker.toml`.
8. Run `cargo run -- run` and change evidence in the game journal. The app should print transitions like:

```text
[ 12.31s] EMF Level 5: clear -> selected
[ 14.08s] Spirit Box: clear -> rejected
```

## Config Shape

Each evidence entry has two sampled regions:

- `selected`: the visual mark that appears when evidence is confirmed
- `rejected`: the visual mark that appears when evidence is ruled out

Coordinates are normalized percentages of the captured window image. For example, `x_pct = 0.5` means halfway across the captured image. Color matching is intentionally simple: each pixel in the region is counted if RGB channels are within `tolerance`; the region is considered active when the matching-pixel ratio reaches `min_ratio`.

The current starter config samples dark ink in the checkbox interiors for selected evidence, and the blank gap between the checkbox and label for crossed-out evidence. If a future UI update draws those marks somewhere else, use `inspect` to see the raw `actual/threshold` ratio and move the relevant region.
