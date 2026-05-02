# AGENTS.md

## Project Overview

This is a standalone Rust CLI that watches the Phasmophobia game window from outside the game process and logs journal evidence changes.

The app must remain external and low-touch. It should not inject into the game, read game memory, intercept network traffic, modify game files, decompile code, or behave like a cheat/mod. The intended approach is visual observation only:

1. Find the visible Phasmophobia window.
2. Capture the window image.
3. Confirm the Evidence journal page is visible.
4. Sample configured UI regions for each evidence item.
5. Log transitions between `clear`, `selected`, and `rejected`.

## Current UX

Run from the repo root:

```powershell
cargo run
```

The default config path is `phasmo_tracker.toml`. If it does not exist, the app creates it on first run and reuses it later.

Custom config path:

```powershell
cargo run -- --config my-tracker.toml
```

## Important Behavior

- Evidence regions are stored as normalized percentages of the captured window, not fixed pixels.
- `phasmo_tracker.toml` is persistent app state and can be user-calibrated.
- `phasmo-window.png` style snapshots are temporary calibration/debug artifacts and are not required at runtime.
- The tracker ignores frames unless the Evidence journal page is detected. This prevents normal gameplay and other journal pages from producing false evidence changes.
- `rejected` wins over `selected` if both matchers fire. This is intentional because Phasmophobia draws the rejection strikethrough through the checkbox, so a rejected row can also trip the selected ink region.
- The polling cadence is intentionally aggressive for quick journal interactions: `poll_ms = 10`, `stable_frames = 1`. Effective speed is still limited by window capture time.
- End-of-game handling is centralized in `handle_end_of_game_actions` in `src/tracker.rs`. The current signal treats an active round as over when the Evidence page was hidden after activity and later reappears fully clear; reset evidence selections there so future end-of-game actions can extend the same function. Possible ghosts are derived from evidence and do not need separate stored state.

## Module Map

- `src/main.rs`: Thin CLI entrypoint. Parses `--config` and calls `tracker::run`.
- `src/lib.rs`: Crate module root.
- `src/config.rs`: Config schema, defaults, load-or-create behavior, validation, config tests.
- `src/window.rs`: Phasmophobia window discovery using `xcap`.
- `src/page.rs`: Evidence-page visibility gate. Requires journal paper, evidence checkbox column, and right-side ghost-name grid.
- `src/evidence.rs`: Evidence-state classification from configured selected/rejected regions.
- `src/ghosts.rs`: Ghost evidence knowledge loading, default ghost data, and candidate filtering.
- `src/tracker.rs`: Main loop, window capture, page gating, state transition logging, end-of-game reset handling, Ctrl-C handling.

## Verification

Before finishing changes, run:

```powershell
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

On Windows, `cargo build` may fail with access denied if `target\debug\phasmo_evidence_tracker.exe` is still running. Stop the running tracker process and retry.

## Common Failure Modes

- False positives during gameplay: tighten `src/page.rs`; do not weaken page gating without adding tests.
- False positives on other journal pages: require Evidence-page-specific markers, especially the right-side ghost grid.
- Missed quick selections: lower `poll_ms`, avoid extra stable-frame requirements, or improve capture strategy.
- All evidence appears selected: selected sample regions may be sitting on paper/text instead of the checkbox mark.
- Rejected evidence appears as conflict/selected: remember the strikethrough crosses the checkbox; rejected should take priority.

## Git Notes

The repo currently lives at:

```text
C:\Users\camde\dev\phasmo_tracker
```

Use semantic commits, for example:

```text
fix: gate evidence tracking to journal page
refactor: separate logic into different files
feat: detect changes in evidence
```
