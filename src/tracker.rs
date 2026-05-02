use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};

use crate::config;
use crate::evidence::{self, EvidenceState};
use crate::ghosts::{self, GhostKnowledge};
use crate::page;
use crate::window;

#[derive(Debug, Default)]
struct TrackerState {
    committed: BTreeMap<String, EvidenceState>,
    pending: BTreeMap<String, (EvidenceState, usize)>,
    evidence_page_hidden_after_activity: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameOverSignal {
    JournalReset,
}

pub fn run(config_path: &Path, ghosts_path: &Path) -> Result<()> {
    let loaded = config::load_or_create(config_path)?;
    let config = loaded.config;
    let loaded_ghosts = ghosts::load_or_create(ghosts_path, &config.evidence)?;
    let ghost_knowledge = loaded_ghosts.knowledge;

    if loaded.created {
        println!("created default config at {}", config_path.display());
    }

    if loaded_ghosts.created {
        println!("created default ghost data at {}", ghosts_path.display());
    }

    println!(
        "looking for a visible app/window matching app={:?}, title={:?}",
        config.tracker.app_name_contains, config.tracker.window_title_contains
    );

    let running = install_ctrlc_handler()?;
    let started = Instant::now();
    let mut state = TrackerState::default();
    let mut page_was_visible = None;

    while running.load(Ordering::SeqCst) {
        let target = match window::find_target_window(&config.tracker) {
            Ok(target) => target,
            Err(err) => {
                eprintln!("{err}");
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        let image = target
            .capture_image()
            .map_err(|err| anyhow!("failed to capture target window: {err}"))?;

        let page_visible = page::evidence_page_visible(&image, &config.evidence);
        if !page_visible {
            state.note_evidence_page_not_visible();
            if page_was_visible != Some(false) {
                println!(
                    "[{:>6.2}s] evidence page not visible; waiting",
                    started.elapsed().as_secs_f32()
                );
            }
            page_was_visible = Some(false);
            thread::sleep(poll_interval(config.tracker.poll_ms));
            continue;
        }

        if page_was_visible == Some(false) {
            println!(
                "[{:>6.2}s] evidence page visible",
                started.elapsed().as_secs_f32()
            );
        }
        page_was_visible = Some(true);

        let states = evidence::evaluate(&image, &config.evidence);

        if let Some(signal) = detect_game_over(&state, &states) {
            handle_end_of_game_actions(started.elapsed().as_secs_f32(), &mut state, states, signal);
            thread::sleep(poll_interval(config.tracker.poll_ms));
            continue;
        }

        if state.committed.is_empty() {
            println!(
                "[{:>6.2}s] captured {}x{}; initial state: {}",
                started.elapsed().as_secs_f32(),
                image.width(),
                image.height(),
                summarize_states(&states)
            );
            state.committed = states;
            emit_possible_ghosts(
                started.elapsed().as_secs_f32(),
                &ghost_knowledge,
                &state.committed,
            );
        } else {
            emit_stable_changes(
                started.elapsed().as_secs_f32(),
                &mut state.committed,
                &mut state.pending,
                states,
                config.tracker.stable_frames.max(1),
                &ghost_knowledge,
            );
        }

        thread::sleep(poll_interval(config.tracker.poll_ms));
    }

    println!("stopped");
    Ok(())
}

impl TrackerState {
    fn has_round_activity(&self) -> bool {
        self.committed
            .values()
            .any(|state| matches!(state, EvidenceState::Selected | EvidenceState::Rejected))
    }

    fn note_evidence_page_not_visible(&mut self) {
        self.pending.clear();
        if self.has_round_activity() {
            self.evidence_page_hidden_after_activity = true;
        }
    }

    fn reset_for_next_round(&mut self, current: BTreeMap<String, EvidenceState>) {
        self.committed = current;
        self.pending.clear();
        self.evidence_page_hidden_after_activity = false;
    }
}

fn poll_interval(poll_ms: u64) -> Duration {
    Duration::from_millis(poll_ms.max(1))
}

fn detect_game_over(
    state: &TrackerState,
    current: &BTreeMap<String, EvidenceState>,
) -> Option<GameOverSignal> {
    if state.has_round_activity()
        && state.evidence_page_hidden_after_activity
        && all_evidence_clear(current)
    {
        Some(GameOverSignal::JournalReset)
    } else {
        None
    }
}

fn handle_end_of_game_actions(
    elapsed_secs: f32,
    state: &mut TrackerState,
    current: BTreeMap<String, EvidenceState>,
    signal: GameOverSignal,
) {
    state.reset_for_next_round(current);
    println!("[{elapsed_secs:>6.2}s] game over detected ({signal}); reset evidence selection");
}

fn all_evidence_clear(states: &BTreeMap<String, EvidenceState>) -> bool {
    !states.is_empty() && states.values().all(|state| *state == EvidenceState::Clear)
}

impl std::fmt::Display for GameOverSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameOverSignal::JournalReset => write!(f, "journal reset"),
        }
    }
}

fn emit_stable_changes(
    elapsed_secs: f32,
    committed: &mut BTreeMap<String, EvidenceState>,
    pending: &mut BTreeMap<String, (EvidenceState, usize)>,
    current: BTreeMap<String, EvidenceState>,
    stable_frames: usize,
    ghost_knowledge: &GhostKnowledge,
) {
    let mut changed = false;

    for (name, new_state) in current {
        let old_state = committed
            .get(&name)
            .copied()
            .unwrap_or(EvidenceState::Unknown);

        if old_state == new_state {
            pending.remove(&name);
            continue;
        }

        let entry = pending.entry(name.clone()).or_insert((new_state, 0));
        if entry.0 == new_state {
            entry.1 += 1;
        } else {
            *entry = (new_state, 1);
        }

        if entry.1 >= stable_frames {
            println!("[{elapsed_secs:>6.2}s] {name}: {old_state} -> {new_state}");
            committed.insert(name.clone(), new_state);
            pending.remove(&name);
            changed = true;
        }
    }

    if changed {
        emit_possible_ghosts(elapsed_secs, ghost_knowledge, committed);
    }
}

fn summarize_states(states: &BTreeMap<String, EvidenceState>) -> String {
    states
        .iter()
        .map(|(name, state)| format!("{name}={state}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn emit_possible_ghosts(
    elapsed_secs: f32,
    ghost_knowledge: &GhostKnowledge,
    states: &BTreeMap<String, EvidenceState>,
) {
    let selected = evidence_names_with_state(states, EvidenceState::Selected);
    if selected.is_empty() {
        return;
    }

    let rejected = evidence_names_with_state(states, EvidenceState::Rejected);
    let candidates = ghost_knowledge.possible_ghosts(states);
    let selected = selected.join(", ");
    let rejected = if rejected.is_empty() {
        "none".to_string()
    } else {
        rejected.join(", ")
    };
    let candidates = if candidates.is_empty() {
        "none".to_string()
    } else {
        candidates.join(", ")
    };

    println!(
        "[{elapsed_secs:>6.2}s] selected evidence: {selected}; rejected evidence: {rejected}; possible ghosts: {candidates}"
    );
}

fn evidence_names_with_state(
    states: &BTreeMap<String, EvidenceState>,
    target: EvidenceState,
) -> Vec<String> {
    states
        .iter()
        .filter(|(_, state)| **state == target)
        .map(|(name, _)| name.clone())
        .collect()
}

fn install_ctrlc_handler() -> Result<Arc<AtomicBool>> {
    let running = Arc::new(AtomicBool::new(true));
    let signal = Arc::clone(&running);
    ctrlc::set_handler(move || {
        signal.store(false, Ordering::SeqCst);
    })
    .context("failed to install Ctrl-C handler")?;
    Ok(running)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_game_over_when_active_journal_reappears_clear() {
        let mut state = TrackerState {
            committed: states(&[
                ("EMF Level 5", EvidenceState::Selected),
                ("Ghost Orb", EvidenceState::Clear),
            ]),
            ..TrackerState::default()
        };
        state.note_evidence_page_not_visible();

        assert_eq!(
            detect_game_over(
                &state,
                &states(&[
                    ("EMF Level 5", EvidenceState::Clear),
                    ("Ghost Orb", EvidenceState::Clear),
                ])
            ),
            Some(GameOverSignal::JournalReset)
        );
    }

    #[test]
    fn does_not_detect_game_over_before_round_activity() {
        let mut state = TrackerState {
            committed: states(&[
                ("EMF Level 5", EvidenceState::Clear),
                ("Ghost Orb", EvidenceState::Clear),
            ]),
            ..TrackerState::default()
        };
        state.note_evidence_page_not_visible();

        assert_eq!(
            detect_game_over(
                &state,
                &states(&[
                    ("EMF Level 5", EvidenceState::Clear),
                    ("Ghost Orb", EvidenceState::Clear),
                ])
            ),
            None
        );
    }

    #[test]
    fn end_of_game_actions_reset_evidence_selection() {
        let mut state = TrackerState {
            committed: states(&[
                ("EMF Level 5", EvidenceState::Selected),
                ("Ghost Orb", EvidenceState::Rejected),
            ]),
            pending: BTreeMap::from([("Spirit Box".to_string(), (EvidenceState::Selected, 1))]),
            evidence_page_hidden_after_activity: true,
        };

        handle_end_of_game_actions(
            1.0,
            &mut state,
            states(&[
                ("EMF Level 5", EvidenceState::Clear),
                ("Ghost Orb", EvidenceState::Clear),
            ]),
            GameOverSignal::JournalReset,
        );

        assert!(state.pending.is_empty());
        assert!(!state.evidence_page_hidden_after_activity);
        assert!(all_evidence_clear(&state.committed));
    }

    fn states(entries: &[(&str, EvidenceState)]) -> BTreeMap<String, EvidenceState> {
        entries
            .iter()
            .map(|(name, state)| ((*name).to_string(), *state))
            .collect()
    }
}
