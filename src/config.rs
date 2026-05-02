use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_PATH: &str = "phasmo_tracker.toml";

#[derive(Debug)]
pub struct LoadedConfig {
    pub config: Config,
    pub created: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub tracker: TrackerConfig,
    pub evidence: Vec<EvidenceConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrackerConfig {
    pub window_title_contains: String,
    #[serde(default = "default_app_name_contains")]
    pub app_name_contains: String,
    pub poll_ms: u64,
    pub stable_frames: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvidenceConfig {
    pub name: String,
    pub selected: RegionMatcher,
    pub rejected: RegionMatcher,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionMatcher {
    pub x_pct: f64,
    pub y_pct: f64,
    pub w_pct: f64,
    pub h_pct: f64,
    pub color: ColorMatcher,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorMatcher {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub tolerance: u8,
    pub min_ratio: f64,
}

pub fn load_or_create(path: &Path) -> Result<LoadedConfig> {
    if path.exists() {
        let config = load(path)?;
        validate(&config)?;
        return Ok(LoadedConfig {
            config,
            created: false,
        });
    }

    let config = default_config();
    write(path, &config)?;
    validate(&config)?;
    Ok(LoadedConfig {
        config,
        created: true,
    })
}

fn load(path: &Path) -> Result<Config> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn write(path: &Path, config: &Config) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let encoded = toml::to_string_pretty(config).context("failed to encode starter config")?;
    fs::write(path, encoded).with_context(|| format!("failed to write {}", path.display()))
}

fn validate(config: &Config) -> Result<()> {
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

    if !(0.0..=1.0).contains(&region.color.min_ratio) {
        bail!("{evidence_name}.{label}.color.min_ratio must be between 0 and 1");
    }

    Ok(())
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
            poll_ms: 10,
            stable_frames: 1,
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
    use std::env;
    use std::fs;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn creates_config_once_and_reuses_it() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("phasmo_tracker_test_{}_{}.toml", process::id(), id));

        let first = load_or_create(&path).unwrap();
        assert!(first.created);
        assert!(path.exists());

        let second = load_or_create(&path).unwrap();
        assert!(!second.created);

        fs::remove_file(path).unwrap();
    }
}
