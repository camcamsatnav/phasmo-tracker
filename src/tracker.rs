use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use serde::Serialize;

use crate::config;
use crate::evidence::{self, EvidenceState};
use crate::ghosts::{self, GhostKnowledge};
use crate::page;
use crate::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

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

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TrackerEvent {
    ConfigCreated {
        path: String,
    },
    GhostDataCreated {
        path: String,
    },
    TrackerStarted {
        config_path: String,
        ghosts_path: String,
        app_name_contains: String,
        window_title_contains: String,
        poll_ms: u64,
        stable_frames: usize,
        evidence: Vec<String>,
        ghosts: Vec<String>,
    },
    WindowSearchError {
        message: String,
    },
    PageVisibility {
        elapsed_secs: f32,
        visible: bool,
    },
    EvidenceChange {
        elapsed_secs: f32,
        name: String,
        old_state: EvidenceState,
        new_state: EvidenceState,
    },
    Snapshot {
        elapsed_secs: f32,
        reason: SnapshotReason,
        image_width: u32,
        image_height: u32,
        evidence: Vec<EvidenceSnapshot>,
        selected_evidence: Vec<String>,
        rejected_evidence: Vec<String>,
        possible_ghosts: Vec<String>,
        changes: Vec<EvidenceChangeSnapshot>,
    },
    GameOver {
        elapsed_secs: f32,
        signal: String,
    },
    Stopped,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum SnapshotReason {
    Initial,
    Change,
    GameOverReset,
}

#[derive(Debug, Serialize)]
struct EvidenceSnapshot {
    name: String,
    state: EvidenceState,
}

#[derive(Debug, Serialize)]
struct EvidenceChangeSnapshot {
    name: String,
    old_state: EvidenceState,
    new_state: EvidenceState,
}

#[derive(Debug, Clone, Copy)]
struct FrameSize {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy)]
struct SnapshotContext<'a> {
    elapsed_secs: f32,
    frame_size: FrameSize,
    ghost_knowledge: &'a GhostKnowledge,
    states: &'a BTreeMap<String, EvidenceState>,
    evidence_order: &'a [config::EvidenceConfig],
}

#[derive(Debug, Clone, Copy)]
struct EndOfGameContext<'a> {
    elapsed_secs: f32,
    output: &'a TrackerOutput,
    ghost_knowledge: &'a GhostKnowledge,
    frame_size: FrameSize,
    evidence_order: &'a [config::EvidenceConfig],
}

#[derive(Debug, Clone, Copy)]
struct TrackerOutput {
    mode: OutputMode,
}

pub fn run(config_path: &Path, ghosts_path: &Path) -> Result<()> {
    run_with_output_mode(config_path, ghosts_path, OutputMode::Human)
}

pub fn run_with_output_mode(
    config_path: &Path,
    ghosts_path: &Path,
    output_mode: OutputMode,
) -> Result<()> {
    let output = TrackerOutput::new(output_mode);
    let loaded = config::load_or_create(config_path)?;
    let config = loaded.config;
    let loaded_ghosts = ghosts::load_or_create(ghosts_path, &config.evidence)?;
    let ghost_knowledge = loaded_ghosts.knowledge;
    let evidence_order = config.evidence.clone();

    if loaded.created {
        output.config_created(config_path);
    }

    if loaded_ghosts.created {
        output.ghost_data_created(ghosts_path);
    }

    output.tracker_started(config_path, ghosts_path, &config, &ghost_knowledge);

    let running = install_ctrlc_handler()?;
    let started = Instant::now();
    let mut state = TrackerState::default();
    let mut page_was_visible = None;

    while running.load(Ordering::SeqCst) {
        let target = match window::find_target_window(&config.tracker) {
            Ok(target) => target,
            Err(err) => {
                output.window_search_error(&err.to_string());
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        let image = target
            .capture_image()
            .map_err(|err| anyhow!("failed to capture target window: {err}"))?;

        let elapsed_secs = started.elapsed().as_secs_f32();
        let page_visible = page::evidence_page_visible(&image, &config.evidence);
        if !page_visible {
            state.note_evidence_page_not_visible();
            if page_was_visible != Some(false) {
                output.page_visibility(elapsed_secs, false);
            }
            page_was_visible = Some(false);
            thread::sleep(poll_interval(config.tracker.poll_ms));
            continue;
        }

        if page_was_visible == Some(false) {
            output.page_visibility(elapsed_secs, true);
        }
        page_was_visible = Some(true);

        let states = evidence::evaluate(&image, &config.evidence);

        if let Some(signal) = detect_game_over(&state, &states) {
            handle_end_of_game_actions(
                &mut state,
                states,
                signal,
                EndOfGameContext {
                    elapsed_secs,
                    output: &output,
                    ghost_knowledge: &ghost_knowledge,
                    frame_size: FrameSize {
                        width: image.width(),
                        height: image.height(),
                    },
                    evidence_order: &evidence_order,
                },
            );
            thread::sleep(poll_interval(config.tracker.poll_ms));
            continue;
        }

        if state.committed.is_empty() {
            state.committed = states;
            output.initial_snapshot(SnapshotContext {
                elapsed_secs,
                frame_size: FrameSize {
                    width: image.width(),
                    height: image.height(),
                },
                ghost_knowledge: &ghost_knowledge,
                states: &state.committed,
                evidence_order: &evidence_order,
            });
        } else {
            emit_stable_changes(
                elapsed_secs,
                &mut state.committed,
                &mut state.pending,
                states,
                config.tracker.stable_frames.max(1),
                &ghost_knowledge,
                image.width(),
                image.height(),
                &output,
                &evidence_order,
            );
        }

        thread::sleep(poll_interval(config.tracker.poll_ms));
    }

    output.stopped();
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

impl TrackerOutput {
    fn new(mode: OutputMode) -> Self {
        Self { mode }
    }

    fn config_created(&self, path: &Path) {
        match self.mode {
            OutputMode::Human => {
                self.print_stdout(format!("created default config at {}", path.display()))
            }
            OutputMode::Json => self.emit_json(TrackerEvent::ConfigCreated {
                path: path.display().to_string(),
            }),
        }
    }

    fn ghost_data_created(&self, path: &Path) {
        match self.mode {
            OutputMode::Human => {
                self.print_stdout(format!("created default ghost data at {}", path.display()))
            }
            OutputMode::Json => self.emit_json(TrackerEvent::GhostDataCreated {
                path: path.display().to_string(),
            }),
        }
    }

    fn tracker_started(
        &self,
        config_path: &Path,
        ghosts_path: &Path,
        config: &config::Config,
        ghost_knowledge: &GhostKnowledge,
    ) {
        match self.mode {
            OutputMode::Human => self.print_stdout(format!(
                "looking for a visible app/window matching app={:?}, title={:?}",
                config.tracker.app_name_contains, config.tracker.window_title_contains
            )),
            OutputMode::Json => self.emit_json(TrackerEvent::TrackerStarted {
                config_path: config_path.display().to_string(),
                ghosts_path: ghosts_path.display().to_string(),
                app_name_contains: config.tracker.app_name_contains.clone(),
                window_title_contains: config.tracker.window_title_contains.clone(),
                poll_ms: config.tracker.poll_ms,
                stable_frames: config.tracker.stable_frames,
                evidence: config
                    .evidence
                    .iter()
                    .map(|evidence| evidence.name.clone())
                    .collect(),
                ghosts: ghost_knowledge
                    .ghosts
                    .iter()
                    .map(|ghost| ghost.name.clone())
                    .collect(),
            }),
        }
    }

    fn window_search_error(&self, message: &str) {
        match self.mode {
            OutputMode::Human => {
                eprintln!("{message}");
                let _ = io::stderr().flush();
            }
            OutputMode::Json => self.emit_json(TrackerEvent::WindowSearchError {
                message: message.to_string(),
            }),
        }
    }

    fn page_visibility(&self, elapsed_secs: f32, visible: bool) {
        match self.mode {
            OutputMode::Human => {
                let status = if visible {
                    "evidence page visible"
                } else {
                    "evidence page not visible; waiting"
                };
                self.print_stdout(format!("[{elapsed_secs:>6.2}s] {status}"));
            }
            OutputMode::Json => {
                self.emit_json(TrackerEvent::PageVisibility {
                    elapsed_secs,
                    visible,
                });
            }
        }
    }

    fn initial_snapshot(&self, context: SnapshotContext<'_>) {
        match self.mode {
            OutputMode::Human => {
                self.print_stdout(format!(
                    "[{:>6.2}s] captured {}x{}; initial state: {}",
                    context.elapsed_secs,
                    context.frame_size.width,
                    context.frame_size.height,
                    summarize_states(context.states, context.evidence_order)
                ));
                self.possible_ghosts_summary(context);
            }
            OutputMode::Json => self.snapshot(context, SnapshotReason::Initial, Vec::new()),
        }
    }

    fn evidence_change(
        &self,
        elapsed_secs: f32,
        name: &str,
        old_state: EvidenceState,
        new_state: EvidenceState,
    ) {
        match self.mode {
            OutputMode::Human => self.print_stdout(format!(
                "[{elapsed_secs:>6.2}s] {name}: {old_state} -> {new_state}"
            )),
            OutputMode::Json => self.emit_json(TrackerEvent::EvidenceChange {
                elapsed_secs,
                name: name.to_string(),
                old_state,
                new_state,
            }),
        }
    }

    fn changed_snapshot(&self, context: SnapshotContext<'_>, changes: Vec<EvidenceChangeSnapshot>) {
        match self.mode {
            OutputMode::Human => self.possible_ghosts_summary(context),
            OutputMode::Json => self.snapshot(context, SnapshotReason::Change, changes),
        }
    }

    fn game_over(&self, context: SnapshotContext<'_>, signal: GameOverSignal) {
        match self.mode {
            OutputMode::Human => self.print_stdout(format!(
                "[{:>6.2}s] game over detected ({signal}); reset evidence selection",
                context.elapsed_secs
            )),
            OutputMode::Json => {
                self.emit_json(TrackerEvent::GameOver {
                    elapsed_secs: context.elapsed_secs,
                    signal: signal.to_string(),
                });
                self.snapshot(context, SnapshotReason::GameOverReset, Vec::new());
            }
        }
    }

    fn stopped(&self) {
        match self.mode {
            OutputMode::Human => self.print_stdout("stopped".to_string()),
            OutputMode::Json => self.emit_json(TrackerEvent::Stopped),
        }
    }

    fn possible_ghosts_summary(&self, context: SnapshotContext<'_>) {
        let selected = evidence_names_with_state(
            context.states,
            EvidenceState::Selected,
            context.evidence_order,
        );
        if selected.is_empty() {
            return;
        }

        let rejected = evidence_names_with_state(
            context.states,
            EvidenceState::Rejected,
            context.evidence_order,
        );
        let candidates = possible_ghost_names(context.ghost_knowledge, context.states);
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

        self.print_stdout(format!(
            "[{:>6.2}s] selected evidence: {selected}; rejected evidence: {rejected}; possible ghosts: {candidates}",
            context.elapsed_secs
        ));
    }

    fn snapshot(
        &self,
        context: SnapshotContext<'_>,
        reason: SnapshotReason,
        changes: Vec<EvidenceChangeSnapshot>,
    ) {
        self.emit_json(TrackerEvent::Snapshot {
            elapsed_secs: context.elapsed_secs,
            reason,
            image_width: context.frame_size.width,
            image_height: context.frame_size.height,
            evidence: evidence_snapshot(context.states, context.evidence_order),
            selected_evidence: evidence_names_with_state(
                context.states,
                EvidenceState::Selected,
                context.evidence_order,
            ),
            rejected_evidence: evidence_names_with_state(
                context.states,
                EvidenceState::Rejected,
                context.evidence_order,
            ),
            possible_ghosts: possible_ghost_names(context.ghost_knowledge, context.states),
            changes,
        });
    }

    fn emit_json(&self, event: TrackerEvent) {
        let Ok(line) = serde_json::to_string(&event) else {
            return;
        };
        self.print_stdout(line);
    }

    fn print_stdout(&self, line: String) {
        println!("{line}");
        let _ = io::stdout().flush();
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
    state: &mut TrackerState,
    current: BTreeMap<String, EvidenceState>,
    signal: GameOverSignal,
    context: EndOfGameContext<'_>,
) {
    state.reset_for_next_round(current);
    context.output.game_over(
        SnapshotContext {
            elapsed_secs: context.elapsed_secs,
            frame_size: context.frame_size,
            ghost_knowledge: context.ghost_knowledge,
            states: &state.committed,
            evidence_order: context.evidence_order,
        },
        signal,
    );
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

#[allow(clippy::too_many_arguments)]
fn emit_stable_changes(
    elapsed_secs: f32,
    committed: &mut BTreeMap<String, EvidenceState>,
    pending: &mut BTreeMap<String, (EvidenceState, usize)>,
    current: BTreeMap<String, EvidenceState>,
    stable_frames: usize,
    ghost_knowledge: &GhostKnowledge,
    image_width: u32,
    image_height: u32,
    output: &TrackerOutput,
    evidence_order: &[config::EvidenceConfig],
) {
    let mut changes = Vec::new();

    for item in evidence_order {
        let name = item.name.clone();
        let Some(new_state) = current.get(&name).copied() else {
            continue;
        };
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
            output.evidence_change(elapsed_secs, &name, old_state, new_state);
            committed.insert(name.clone(), new_state);
            pending.remove(&name);
            changes.push(EvidenceChangeSnapshot {
                name,
                old_state,
                new_state,
            });
        }
    }

    if !changes.is_empty() {
        output.changed_snapshot(
            SnapshotContext {
                elapsed_secs,
                frame_size: FrameSize {
                    width: image_width,
                    height: image_height,
                },
                ghost_knowledge,
                states: committed,
                evidence_order,
            },
            changes,
        );
    }
}

fn summarize_states(
    states: &BTreeMap<String, EvidenceState>,
    evidence_order: &[config::EvidenceConfig],
) -> String {
    evidence_order
        .iter()
        .filter_map(|item| {
            states
                .get(&item.name)
                .map(|state| (item.name.as_str(), state))
        })
        .map(|(name, state)| format!("{name}={state}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn evidence_names_with_state(
    states: &BTreeMap<String, EvidenceState>,
    target: EvidenceState,
    evidence_order: &[config::EvidenceConfig],
) -> Vec<String> {
    evidence_order
        .iter()
        .filter(|item| states.get(&item.name).copied() == Some(target))
        .map(|item| item.name.clone())
        .collect()
}

fn evidence_snapshot(
    states: &BTreeMap<String, EvidenceState>,
    evidence_order: &[config::EvidenceConfig],
) -> Vec<EvidenceSnapshot> {
    evidence_order
        .iter()
        .filter_map(|item| {
            states.get(&item.name).map(|state| EvidenceSnapshot {
                name: item.name.clone(),
                state: *state,
            })
        })
        .collect()
}

fn possible_ghost_names(
    ghost_knowledge: &GhostKnowledge,
    states: &BTreeMap<String, EvidenceState>,
) -> Vec<String> {
    ghost_knowledge
        .possible_ghosts(states)
        .into_iter()
        .map(str::to_string)
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
        let output = TrackerOutput::new(OutputMode::Human);
        let ghost_knowledge = GhostKnowledge { ghosts: Vec::new() };
        let evidence_order = evidence_order(&["EMF Level 5", "Ghost Orb"]);

        handle_end_of_game_actions(
            &mut state,
            states(&[
                ("EMF Level 5", EvidenceState::Clear),
                ("Ghost Orb", EvidenceState::Clear),
            ]),
            GameOverSignal::JournalReset,
            EndOfGameContext {
                elapsed_secs: 1.0,
                output: &output,
                ghost_knowledge: &ghost_knowledge,
                frame_size: FrameSize {
                    width: 1000,
                    height: 1000,
                },
                evidence_order: &evidence_order,
            },
        );

        assert!(state.pending.is_empty());
        assert!(!state.evidence_page_hidden_after_activity);
        assert!(all_evidence_clear(&state.committed));
    }

    #[test]
    fn evidence_lists_follow_config_order_not_map_order() {
        let states = states(&[
            ("Spirit Box", EvidenceState::Selected),
            ("EMF Level 5", EvidenceState::Selected),
            ("Ghost Orb", EvidenceState::Rejected),
        ]);
        let evidence_order = evidence_order(&["EMF Level 5", "Ghost Orb", "Spirit Box"]);

        assert_eq!(
            evidence_names_with_state(&states, EvidenceState::Selected, &evidence_order),
            vec!["EMF Level 5", "Spirit Box"]
        );
        assert_eq!(
            summarize_states(&states, &evidence_order),
            "EMF Level 5=selected, Ghost Orb=rejected, Spirit Box=selected"
        );
        assert_eq!(
            evidence_snapshot(&states, &evidence_order)
                .into_iter()
                .map(|item| item.name)
                .collect::<Vec<_>>(),
            vec!["EMF Level 5", "Ghost Orb", "Spirit Box"]
        );
    }

    fn states(entries: &[(&str, EvidenceState)]) -> BTreeMap<String, EvidenceState> {
        entries
            .iter()
            .map(|(name, state)| ((*name).to_string(), *state))
            .collect()
    }

    fn evidence_order(names: &[&str]) -> Vec<config::EvidenceConfig> {
        names
            .iter()
            .map(|name| config::EvidenceConfig {
                name: (*name).to_string(),
                selected: region_matcher(),
                rejected: region_matcher(),
            })
            .collect()
    }

    fn region_matcher() -> config::RegionMatcher {
        config::RegionMatcher {
            x_pct: 0.0,
            y_pct: 0.0,
            w_pct: 0.1,
            h_pct: 0.1,
            color: config::ColorMatcher {
                r: 0,
                g: 0,
                b: 0,
                tolerance: 0,
                min_ratio: 0.5,
            },
        }
    }
}
