use anyhow::{Result, anyhow, bail};
use xcap::Window;

use crate::config::TrackerConfig;

pub fn find_target_window(tracker: &TrackerConfig) -> Result<Window> {
    let title_needle = tracker.window_title_contains.to_lowercase();
    let app_needle = tracker.app_name_contains.to_lowercase();
    let windows = Window::all().map_err(|err| anyhow!("failed to enumerate windows: {err}"))?;
    let mut minimized_match = None;

    for window in windows {
        let title = window.title().unwrap_or_default();
        let app_name = window.app_name().unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }

        if !app_needle.is_empty() && !app_name.to_lowercase().contains(&app_needle) {
            continue;
        }

        if !title_needle.is_empty() && !title.to_lowercase().contains(&title_needle) {
            continue;
        }

        if window.is_minimized().unwrap_or(false) {
            minimized_match = Some(format!("{app_name} / {title}"));
            continue;
        }

        return Ok(window);
    }

    if let Some(title) = minimized_match {
        bail!("found {title:?}, but it is minimized; restore it before tracking");
    }

    bail!(
        "could not find a visible app/window matching app={:?}, title={:?}",
        tracker.app_name_contains,
        tracker.window_title_contains
    )
}
