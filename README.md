# Phasmophobia Evidence Tracker

**Real-time ghost evidence tracking for Phasmophobia** – See what evidence you've collected and which ghosts you can still identify, all in a sleek desktop app.

This is a standalone Rust app that watches the Phasmophobia journal evidence page from outside the game process and logs evidence-state changes. It provides a live desktop UI with evidence tracking, ghost candidate filtering, and unofficial trait support.

## What It Does

- **Monitors your Phasmophobia game window** without injecting into the game or reading game memory
- **Tracks evidence selections** in real-time as you click checkboxes in the Evidence journal
- **Shows possible ghosts** based on your evidence selections
- **Filters by unofficial traits** (cursed, strength, weakness, etc.) to narrow down candidates
- **Syncs across game rounds** – evidence clears automatically when a new round starts

## Get Started

### Option 1: Download the Latest Release

1. Go to [GitHub Actions](https://github.com/camcamsatnav/phasmo-tracker/actions/workflows/desktop-artifact.yml) and find the latest successful build
2. Look for the **Desktop Artifact** workflow run
3. Download the `phasmo-evidence-tracker-windows-...` artifact
4. Extract the ZIP and run `Phasmo Evidence Tracker-0.1.0-portable.exe`

That's it! The app will create its config files on first run.

### Option 2: Clone and Run from Source

Install Rust and Node.js, then:

```powershell
git clone https://github.com/your-repo/phasmo_tracker
cd phasmo_tracker
pnpm install
pnpm run bundle
```

The portable EXE will be generated in `release\`.

For development with live reload:

```powershell
pnpm run dev
```

## User-Friendly Features

### Live Evidence Tracking
The app monitors your Evidence journal page in real-time. As you click evidence checkboxes in Phasmophobia, the tracker immediately shows your selections.

### Ghost Candidate Filtering
Based on your evidence, the app calculates which ghosts are still possible. The list updates every time you select or reject evidence.

### Trait Filtering
Use the unofficial traits tab (cursed, strength, weakness, camouflage, etc.) to further narrow down candidates.

### Auto-Reset on New Rounds
When you finish a hunt and start a new round, the app automatically clears the evidence tracking and resets your ghost list.
