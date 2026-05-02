use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};

use crate::config;
use crate::evidence::{self, EvidenceState};
use crate::page;
use crate::window;

pub fn run(config_path: &Path) -> Result<()> {
    let loaded = config::load_or_create(config_path)?;
    let config = loaded.config;

    if loaded.created {
        println!("created default config at {}", config_path.display());
    }

    println!(
        "looking for a visible app/window matching app={:?}, title={:?}",
        config.tracker.app_name_contains, config.tracker.window_title_contains
    );

    let running = install_ctrlc_handler()?;
    let started = Instant::now();
    let mut committed = BTreeMap::new();
    let mut pending = BTreeMap::new();
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
            pending.clear();
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

        if committed.is_empty() {
            println!(
                "[{:>6.2}s] captured {}x{}; initial state: {}",
                started.elapsed().as_secs_f32(),
                image.width(),
                image.height(),
                summarize_states(&states)
            );
            committed = states;
        } else {
            emit_stable_changes(
                started.elapsed().as_secs_f32(),
                &mut committed,
                &mut pending,
                states,
                config.tracker.stable_frames.max(1),
            );
        }

        thread::sleep(poll_interval(config.tracker.poll_ms));
    }

    println!("stopped");
    Ok(())
}

fn poll_interval(poll_ms: u64) -> Duration {
    Duration::from_millis(poll_ms.max(1))
}

fn emit_stable_changes(
    elapsed_secs: f32,
    committed: &mut BTreeMap<String, EvidenceState>,
    pending: &mut BTreeMap<String, (EvidenceState, usize)>,
    current: BTreeMap<String, EvidenceState>,
    stable_frames: usize,
) {
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
        }
    }
}

fn summarize_states(states: &BTreeMap<String, EvidenceState>) -> String {
    states
        .iter()
        .map(|(name, state)| format!("{name}={state}"))
        .collect::<Vec<_>>()
        .join(", ")
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
