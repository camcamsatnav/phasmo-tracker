use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use xcap::Window;

const DEFAULT_CONFIG_PATH: &str = "phasmo_tracker.toml";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Track evidence-state changes and print them to the console.
    Run {
        /// Capture one frame, print states, and exit.
        #[arg(long)]
        once: bool,
    },
    /// Capture the Phasmophobia window to help with calibration.
    Snapshot {
        #[arg(short, long, default_value = "phasmo-window.png")]
        output: PathBuf,
    },
    /// Sample a pixel/area from the Phasmophobia window and print a config hint.
    Probe {
        /// X coordinate in the saved window screenshot.
        #[arg(long)]
        x: u32,
        /// Y coordinate in the saved window screenshot.
        #[arg(long)]
        y: u32,
        /// Radius around the coordinate to average.
        #[arg(long, default_value_t = 3)]
        radius: u32,
    },
    /// Print raw matcher scores for a screenshot or the live game window.
    Inspect {
        /// Inspect an existing PNG snapshot instead of capturing the live window.
        #[arg(short, long)]
        image: Option<PathBuf>,
    },
    /// List capturable desktop windows.
    ListWindows,
    /// Write a starter config file.
    WriteConfig {
        /// Overwrite the existing config file.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    tracker: TrackerConfig,
    evidence: Vec<EvidenceConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TrackerConfig {
    window_title_contains: String,
    #[serde(default = "default_app_name_contains")]
    app_name_contains: String,
    poll_ms: u64,
    stable_frames: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EvidenceConfig {
    name: String,
    selected: RegionMatcher,
    rejected: RegionMatcher,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RegionMatcher {
    x_pct: f64,
    y_pct: f64,
    w_pct: f64,
    h_pct: f64,
    color: ColorMatcher,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ColorMatcher {
    r: u8,
    g: u8,
    b: u8,
    tolerance: u8,
    min_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EvidenceState {
    Unknown,
    Clear,
    Selected,
    Rejected,
    Conflict,
}

impl fmt::Display for EvidenceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvidenceState::Unknown => write!(f, "unknown"),
            EvidenceState::Clear => write!(f, "clear"),
            EvidenceState::Selected => write!(f, "selected"),
            EvidenceState::Rejected => write!(f, "rejected"),
            EvidenceState::Conflict => write!(f, "conflict"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct WindowSummary {
    title: String,
    app_name: String,
    pid: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    minimized: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Run { once: false }) {
        Command::Run { once } => run_tracker(&cli.config, once),
        Command::Snapshot { output } => write_snapshot(&cli.config, &output),
        Command::Probe { x, y, radius } => probe_pixel(&cli.config, x, y, radius),
        Command::Inspect { image } => inspect_scores(&cli.config, image.as_deref()),
        Command::ListWindows => list_windows(),
        Command::WriteConfig { force } => write_default_config(&cli.config, force),
    }
}

fn run_tracker(config_path: &Path, once: bool) -> Result<()> {
    ensure_config_exists(config_path)?;
    let config = load_config(config_path)?;
    validate_config(&config)?;

    println!(
        "looking for a visible app/window matching app={:?}, title={:?}",
        config.tracker.app_name_contains, config.tracker.window_title_contains
    );
    let mut last_committed: BTreeMap<String, EvidenceState> = BTreeMap::new();
    let mut pending: BTreeMap<String, (EvidenceState, usize)> = BTreeMap::new();
    let running = install_ctrlc_handler()?;
    let started = Instant::now();

    while running.load(Ordering::SeqCst) {
        let window = match find_target_window(&config.tracker) {
            Ok(window) => window,
            Err(err) => {
                eprintln!("{err}");
                if once {
                    return Err(err);
                }
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };

        let image = window
            .capture_image()
            .map_err(|err| anyhow!("failed to capture target window: {err}"))?;
        let states = evaluate_evidence(&image, &config.evidence);

        if last_committed.is_empty() {
            println!(
                "[{:>6.2}s] captured {}x{}; initial state: {}",
                started.elapsed().as_secs_f32(),
                image.width(),
                image.height(),
                summarize_states(&states)
            );
            last_committed = states;
            if once {
                return Ok(());
            }
        } else {
            emit_stable_changes(
                started.elapsed().as_secs_f32(),
                &mut last_committed,
                &mut pending,
                states,
                config.tracker.stable_frames.max(1),
            );
        }

        if once {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(config.tracker.poll_ms.max(50)));
    }

    println!("stopped");
    Ok(())
}

fn write_snapshot(config_path: &Path, output: &Path) -> Result<()> {
    ensure_config_exists(config_path)?;
    let config = load_config(config_path)?;
    let window = find_target_window(&config.tracker)?;
    let image = window
        .capture_image()
        .map_err(|err| anyhow!("failed to capture target window: {err}"))?;

    image
        .save(output)
        .with_context(|| format!("failed to save snapshot to {}", output.display()))?;
    println!(
        "saved {} ({}x{})",
        output.display(),
        image.width(),
        image.height()
    );
    Ok(())
}

fn probe_pixel(config_path: &Path, x: u32, y: u32, radius: u32) -> Result<()> {
    ensure_config_exists(config_path)?;
    let config = load_config(config_path)?;
    let window = find_target_window(&config.tracker)?;
    let image = window
        .capture_image()
        .map_err(|err| anyhow!("failed to capture target window: {err}"))?;

    if x >= image.width() || y >= image.height() {
        bail!(
            "coordinate ({x}, {y}) is outside the captured window size {}x{}",
            image.width(),
            image.height()
        );
    }

    let sample = average_rgb(&image, x, y, radius);
    let w_pct = ((radius * 2 + 1) as f64 / image.width() as f64).max(0.002);
    let h_pct = ((radius * 2 + 1) as f64 / image.height() as f64).max(0.002);
    let x_pct = x as f64 / image.width() as f64;
    let y_pct = y as f64 / image.height() as f64;

    println!("window: {}x{}", image.width(), image.height());
    println!(
        "average rgb around ({x}, {y}) radius {radius}: {:?}",
        sample
    );
    println!("config hint:");
    println!("x_pct = {:.6}", x_pct);
    println!("y_pct = {:.6}", y_pct);
    println!("w_pct = {:.6}", w_pct);
    println!("h_pct = {:.6}", h_pct);
    println!("[color]");
    println!("r = {}", sample.0);
    println!("g = {}", sample.1);
    println!("b = {}", sample.2);
    println!("tolerance = 35");
    println!("min_ratio = 0.18");
    Ok(())
}

fn inspect_scores(config_path: &Path, image_path: Option<&Path>) -> Result<()> {
    ensure_config_exists(config_path)?;
    let config = load_config(config_path)?;
    validate_config(&config)?;
    let image = match image_path {
        Some(path) => image::open(path)
            .with_context(|| format!("failed to open {}", path.display()))?
            .to_rgba8(),
        None => {
            let window = find_target_window(&config.tracker)?;
            window
                .capture_image()
                .map_err(|err| anyhow!("failed to capture target window: {err}"))?
        }
    };

    println!("image: {}x{}", image.width(), image.height());
    println!(
        "{:<24} {:<10} {:>10} {:>10} {:>18} {:>18}",
        "evidence", "state", "selected", "rejected", "selected region", "rejected region"
    );

    for item in &config.evidence {
        let selected = color_match_report(&image, &item.selected);
        let rejected = color_match_report(&image, &item.rejected);
        let state = classify_from_reports(&selected, &rejected);
        println!(
            "{:<24} {:<10} {:>10} {:>10} {:>18} {:>18}",
            item.name,
            state,
            selected.summary(),
            rejected.summary(),
            selected.region_summary(),
            rejected.region_summary()
        );
    }

    Ok(())
}

fn list_windows() -> Result<()> {
    for summary in visible_window_summaries()? {
        println!(
            "pid={:<7} app={:<28} {:>5} {:>5} {:>5}x{:<5} minimized={:<5} {}",
            summary.pid,
            truncate(&summary.app_name, 28),
            summary.x,
            summary.y,
            summary.width,
            summary.height,
            summary.minimized,
            summary.title
        );
    }
    Ok(())
}

fn ensure_config_exists(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    write_default_config(path, false)?;
    println!(
        "created starter config at {}; calibrate it with `snapshot` and `probe` before expecting accurate state changes",
        path.display()
    );
    Ok(())
}

fn write_default_config(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "{} already exists; pass --force to overwrite it",
            path.display()
        );
    }

    let config = default_config();
    let encoded = toml::to_string_pretty(&config).context("failed to encode starter config")?;
    fs::write(path, encoded).with_context(|| format!("failed to write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn load_config(path: &Path) -> Result<Config> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn validate_config(config: &Config) -> Result<()> {
    if config.evidence.is_empty() {
        bail!("config has no evidence entries");
    }

    for evidence in &config.evidence {
        validate_region(&evidence.name, "selected", &evidence.selected)?;
        validate_region(&evidence.name, "rejected", &evidence.rejected)?;
    }

    Ok(())
}

fn validate_region(evidence_name: &str, label: &str, region: &RegionMatcher) -> Result<()> {
    let fields = [
        ("x_pct", region.x_pct),
        ("y_pct", region.y_pct),
        ("w_pct", region.w_pct),
        ("h_pct", region.h_pct),
    ];

    for (name, value) in fields {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            bail!("{evidence_name}.{label}.{name} must be between 0 and 1");
        }
    }

    if region.color.min_ratio < 0.0 || region.color.min_ratio > 1.0 {
        bail!("{evidence_name}.{label}.color.min_ratio must be between 0 and 1");
    }

    Ok(())
}

fn find_target_window(tracker: &TrackerConfig) -> Result<Window> {
    let title_needle = tracker.window_title_contains.to_lowercase();
    let app_needle = tracker.app_name_contains.to_lowercase();
    let windows = Window::all().map_err(|err| anyhow!("failed to enumerate windows: {err}"))?;

    let mut fallback = None;
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

        let minimized = window.is_minimized().unwrap_or(false);
        if minimized {
            fallback = Some(format!("{app_name} / {title}"));
            continue;
        }

        return Ok(window);
    }

    if let Some(title) = fallback {
        bail!("found {title:?}, but it is minimized; restore it before tracking");
    }

    bail!(
        "could not find a visible app/window matching app={:?}, title={:?}",
        tracker.app_name_contains,
        tracker.window_title_contains
    )
}

fn visible_window_summaries() -> Result<Vec<WindowSummary>> {
    let windows = Window::all().map_err(|err| anyhow!("failed to enumerate windows: {err}"))?;
    let mut summaries = Vec::new();

    for window in windows {
        let title = window.title().unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }

        summaries.push(WindowSummary {
            app_name: window.app_name().unwrap_or_default(),
            pid: window.pid().unwrap_or_default(),
            title,
            x: window.x().unwrap_or_default(),
            y: window.y().unwrap_or_default(),
            width: window.width().unwrap_or_default(),
            height: window.height().unwrap_or_default(),
            minimized: window.is_minimized().unwrap_or(false),
        });
    }

    summaries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(summaries)
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{}...", truncated.trim_end())
    } else {
        truncated
    }
}

fn evaluate_evidence(
    image: &RgbaImage,
    evidence: &[EvidenceConfig],
) -> BTreeMap<String, EvidenceState> {
    let mut states = BTreeMap::new();

    for item in evidence {
        let selected = color_match_report(image, &item.selected);
        let rejected = color_match_report(image, &item.rejected);
        let state = classify_from_reports(&selected, &rejected);
        states.insert(item.name.clone(), state);
    }

    states
}

fn classify_from_reports(selected: &MatchReport, rejected: &MatchReport) -> EvidenceState {
    match (selected.region.is_some(), rejected.region.is_some()) {
        (false, _) | (_, false) => EvidenceState::Unknown,
        (true, true) if selected.active && rejected.active => EvidenceState::Conflict,
        (true, true) if selected.active => EvidenceState::Selected,
        (true, true) if rejected.active => EvidenceState::Rejected,
        (true, true) => EvidenceState::Clear,
    }
}

#[derive(Debug, Clone)]
struct MatchReport {
    region: Option<PixelRegion>,
    ratio: f64,
    threshold: f64,
    active: bool,
}

impl MatchReport {
    fn summary(&self) -> String {
        format!("{:.3}/{:.3}", self.ratio, self.threshold)
    }

    fn region_summary(&self) -> String {
        match self.region {
            Some(region) => format!(
                "{},{},{}x{}",
                region.x, region.y, region.width, region.height
            ),
            None => "out-of-bounds".to_string(),
        }
    }
}

fn color_match_report(image: &RgbaImage, matcher: &RegionMatcher) -> MatchReport {
    let Some(region) = resolve_region(image.width(), image.height(), matcher) else {
        return MatchReport {
            region: None,
            ratio: 0.0,
            threshold: matcher.color.min_ratio,
            active: false,
        };
    };

    let mut total = 0u32;
    let mut matched = 0u32;
    let tolerance = matcher.color.tolerance as i16;

    for y in region.y..region.y.saturating_add(region.height) {
        for x in region.x..region.x.saturating_add(region.width) {
            let pixel = image.get_pixel(x, y);
            total += 1;

            let r_ok = (pixel[0] as i16 - matcher.color.r as i16).abs() <= tolerance;
            let g_ok = (pixel[1] as i16 - matcher.color.g as i16).abs() <= tolerance;
            let b_ok = (pixel[2] as i16 - matcher.color.b as i16).abs() <= tolerance;
            if r_ok && g_ok && b_ok {
                matched += 1;
            }
        }
    }

    let ratio = if total == 0 {
        0.0
    } else {
        matched as f64 / total as f64
    };

    MatchReport {
        region: Some(region),
        ratio,
        threshold: matcher.color.min_ratio,
        active: ratio >= matcher.color.min_ratio,
    }
}

fn resolve_region(width: u32, height: u32, matcher: &RegionMatcher) -> Option<PixelRegion> {
    if width == 0 || height == 0 || matcher.w_pct <= 0.0 || matcher.h_pct <= 0.0 {
        return None;
    }

    let x = (matcher.x_pct * width as f64).round() as u32;
    let y = (matcher.y_pct * height as f64).round() as u32;
    if x >= width || y >= height {
        return None;
    }

    let region_width = ((matcher.w_pct * width as f64).round() as u32).max(1);
    let region_height = ((matcher.h_pct * height as f64).round() as u32).max(1);
    let clamped_width = region_width.min(width - x);
    let clamped_height = region_height.min(height - y);

    Some(PixelRegion {
        x,
        y,
        width: clamped_width,
        height: clamped_height,
    })
}

fn average_rgb(image: &RgbaImage, x: u32, y: u32, radius: u32) -> (u8, u8, u8) {
    let min_x = x.saturating_sub(radius);
    let min_y = y.saturating_sub(radius);
    let max_x = x.saturating_add(radius).min(image.width() - 1);
    let max_y = y.saturating_add(radius).min(image.height() - 1);

    let mut count = 0u32;
    let mut r = 0u32;
    let mut g = 0u32;
    let mut b = 0u32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let pixel = image.get_pixel(px, py);
            count += 1;
            r += pixel[0] as u32;
            g += pixel[1] as u32;
            b += pixel[2] as u32;
        }
    }

    ((r / count) as u8, (g / count) as u8, (b / count) as u8)
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

fn default_config() -> Config {
    let names = [
        "EMF Level 5",
        "D.O.T.S Projector",
        "Ultraviolet",
        "Freezing Temperatures",
        "Ghost Orb",
        "Ghost Writing",
        "Spirit Box",
    ];

    Config {
        tracker: TrackerConfig {
            window_title_contains: "Phasmophobia".to_string(),
            app_name_contains: "Phasmophobia".to_string(),
            poll_ms: 350,
            stable_frames: 2,
        },
        evidence: names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let selected_y = 0.235 + index as f64 * 0.098;
                let rejected_y = 0.240 + index as f64 * 0.098;
                EvidenceConfig {
                    name: (*name).to_string(),
                    selected: RegionMatcher {
                        x_pct: 0.231,
                        y_pct: selected_y,
                        w_pct: 0.008,
                        h_pct: 0.017,
                        color: ColorMatcher {
                            r: 10,
                            g: 10,
                            b: 10,
                            tolerance: 55,
                            min_ratio: 0.08,
                        },
                    },
                    rejected: RegionMatcher {
                        x_pct: 0.244,
                        y_pct: rejected_y,
                        w_pct: 0.006,
                        h_pct: 0.008,
                        color: ColorMatcher {
                            r: 10,
                            g: 10,
                            b: 10,
                            tolerance: 55,
                            min_ratio: 0.10,
                        },
                    },
                }
            })
            .collect(),
    }
}

fn default_app_name_contains() -> String {
    "Phasmophobia".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher(x_pct: f64, y_pct: f64, w_pct: f64, h_pct: f64) -> RegionMatcher {
        RegionMatcher {
            x_pct,
            y_pct,
            w_pct,
            h_pct,
            color: ColorMatcher {
                r: 0,
                g: 0,
                b: 0,
                tolerance: 0,
                min_ratio: 0.5,
            },
        }
    }

    #[test]
    fn classifies_scores() {
        let inactive = MatchReport {
            region: Some(PixelRegion {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            }),
            ratio: 0.0,
            threshold: 0.5,
            active: false,
        };
        let active = MatchReport {
            active: true,
            ratio: 0.9,
            ..inactive.clone()
        };

        assert_eq!(
            classify_from_reports(&active, &inactive),
            EvidenceState::Selected
        );
        assert_eq!(
            classify_from_reports(&inactive, &active),
            EvidenceState::Rejected
        );
        assert_eq!(
            classify_from_reports(&active, &active),
            EvidenceState::Conflict
        );
        assert_eq!(
            classify_from_reports(&inactive, &inactive),
            EvidenceState::Clear
        );
    }

    #[test]
    fn resolves_normalized_region() {
        assert_eq!(
            resolve_region(1000, 500, &matcher(0.1, 0.2, 0.05, 0.1)),
            Some(PixelRegion {
                x: 100,
                y: 100,
                width: 50,
                height: 50,
            })
        );
    }

    #[test]
    fn rejects_empty_region() {
        assert_eq!(resolve_region(100, 100, &matcher(0.0, 0.0, 0.0, 0.1)), None);
    }
}
