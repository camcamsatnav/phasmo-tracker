# AGENTS.md

## Project Overview

**Phasmophobia Evidence Tracker** is a full-stack Rust + TypeScript + Electron application that watches the Phasmophobia game window from outside the game process and provides real-time evidence tracking with a modern desktop UI.

The core architecture consists of:
- **Rust backend**: Standalone CLI that observes the game window, samples evidence UI regions, and logs state changes
- **Electron frontend**: Desktop app that displays live evidence state, selected/rejected evidence, possible ghost candidates, and unofficial traits
- **Configuration system**: Persistent TOML files for user-calibrated UI regions and ghost identification data

The backend must remain external and low-touch:
- Does not inject into the game, read game memory, intercept network traffic, modify game files, or decompile code
- Uses visual observation only: window discovery → image capture → Evidence page detection → UI region sampling
- Logs state transitions between `clear`, `selected`, and `rejected` evidence

## Quick Start

### CLI Mode (Rust backend only)

Run from the repo root:

```powershell
cargo run
```

The default config path is `phasmo_tracker.toml`. If it does not exist, the app creates it on first run and reuses it later.

Custom config path:

```powershell
cargo run -- --config my-tracker.toml
```

Custom ghost evidence data file:

```powershell
cargo run -- --ghosts my-ghosts.toml
```

JSON event stream (for integration):

```powershell
cargo run -- --json
```

### Desktop App Mode

Install dependencies once:

```powershell
pnpm install
```

Run dev mode with Vite + Electron:

```powershell
pnpm run dev
```

Build production bundle:

```powershell
pnpm run desktop
```

### Portable Single EXE

Build a self-contained Windows executable:

```powershell
pnpm run bundle
```

Output: `release\Phasmo Evidence Tracker-0.1.0-portable.exe`

For fastest startup, run: `release\win-unpacked\Phasmo Evidence Tracker.exe`

## Architecture & Technical Details

### Configuration

**phasmo_tracker.toml** (user-calibrated UI sampling):
- Evidence regions are stored as normalized percentages of the captured window, not fixed pixels
- This allows the tracker to work across different resolutions
- User-calibrated on first run and persists between launches
- Can be manually edited if game UI changes or language affects label positions
- Stores polling settings: `poll_ms = 10` (aggressive for quick interactions), `stable_frames = 1`

**phasmo_ghosts.toml** (ghost identification data):
- `[[ghosts]]` entries list evidence that can identify each ghost
- `false_evidence` is supported for special cases (e.g., The Mimic's ghost orbs)
- `[[traits]]` entries define unofficial ghost traits with `id`, `label`, optional `description`, and `possible_ghosts` or `excluded_ghosts`
- Traits are streamed by Rust backend and stored locally in frontend; frontend filters ghosts in `filterGhostsByTraits`
- Older ghost files without traits are auto-migrated with bundled defaults

### Evidence State Machine

- Evidence can be in three states: `clear`, `selected`, or `rejected`
- Window discovery uses `xcap` to find the visible Phasmophobia window
- Page gating in `src/page.rs` requires: journal paper UI, Evidence page title/prompt markers, evidence checkbox column, ghost grid
- Frames are ignored unless Evidence page is visible (prevents false positives during gameplay)
- `rejected` wins over `selected` if both matchers fire (strikethrough crosses the checkbox)
- End-of-game reset: if Evidence page disappears then reappears fully clear after activity, the tracker treats it as round over and clears selections

### State Management

- Rust backend runs the main event loop, capturing windows, detecting state changes
- JSON mode (`--json`) streams events to frontend for real-time UI updates
- Frontend stores checked trait IDs locally and derives possible ghost list
- No complex state synchronization needed; events are authoritative

## Module Map

- `src/main.rs`: Thin CLI entrypoint. Parses `--config`, `--ghosts`, `--json` flags and calls `tracker::run`
- `src/lib.rs`: Crate module root
- `src/config.rs`: Config schema, defaults, load-or-create behavior, validation
- `src/window.rs`: Phasmophobia window discovery using `xcap`
- `src/page.rs`: Evidence-page visibility gate. Requires specific UI markers to prevent false positives
- `src/evidence.rs`: Evidence-state classification from configured selected/rejected region samples
- `src/ghosts.rs`: Ghost evidence and trait knowledge loading, candidate filtering
- `src/tracker.rs`: Main loop, window capture, page gating, state transitions, end-of-game reset, Ctrl-C handling

Frontend (TypeScript + React):
- `frontend/src/main.tsx`: Electron entry point
- `frontend/src/App.tsx`: Main app component, Evidence panel, Ghosts panel, Traits
- `frontend/src/components/`: Reusable UI components (EvidenceChip, TrackerDetails, etc.)
- `frontend/src/trackerModel.ts`: Data structures and filtering logic
- `frontend/src/evidenceMeta.ts`, `ghostHuntMeta.ts`: Evidence/ghost type definitions
- `electron/main.cjs`: Electron main process, spawns Rust backend
- `vite.config.ts`: Build configuration for frontend bundle

## Development Workflow

### Testing & Quality

Before committing changes, run:

```powershell
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

Frontend tests:

```powershell
pnpm test
```

On Windows, `cargo build` may fail with access denied if `target\debug\phasmo_evidence_tracker.exe` is still running. Stop the running tracker process and retry.

### Git Practices

Use semantic commits:

```text
fix: gate evidence tracking to journal page
refactor: separate logic into different files
feat: detect changes in evidence
test: add unit tests for evidence state
docs: update AGENTS.md with new architecture
```

## Troubleshooting

### Common Issues

- **False positives during gameplay**: Tighten `src/page.rs` page gating; do not weaken without adding tests
- **False positives on other journal pages**: Require Evidence-page-specific markers, especially the Evidence headings and ghost grid
- **Missed quick selections**: Lower `poll_ms`, reduce `stable_frames`, or improve capture strategy
- **All evidence appears selected**: Selected sample regions may be sampling paper/text instead of checkbox marks
- **Rejected evidence appears as conflict**: Remember the strikethrough crosses the checkbox; `rejected` should take priority (verify in `src/evidence.rs`)
- **Config drift with new Phasmophobia versions**: If UI changes significantly, delete `phasmo_tracker.toml` to recalibrate

### Debug Mode

Add temporary debug logging in `src/tracker.rs` to capture window images and trace state transitions:

```rust
image::save_buffer_with_format(
    "debug_frame.png",
    &frame_bytes,
    window_rect.width() as u32,
    window_rect.height() as u32,
    image::ColorType::Rgb8,
    image::ImageFormat::Png,
)?;
```
