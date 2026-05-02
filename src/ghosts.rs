use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::EvidenceConfig;
use crate::evidence::EvidenceState;

pub const DEFAULT_GHOSTS_PATH: &str = "phasmo_ghosts.toml";

#[derive(Debug)]
pub struct LoadedGhostKnowledge {
    pub knowledge: GhostKnowledge,
    pub created: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GhostKnowledge {
    pub ghosts: Vec<GhostConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GhostConfig {
    pub name: String,
    pub evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub false_evidence: Vec<String>,
}

impl GhostKnowledge {
    pub fn possible_ghosts<'a>(
        &'a self,
        states: &std::collections::BTreeMap<String, EvidenceState>,
    ) -> Vec<&'a str> {
        let selected = normalized_states(states, EvidenceState::Selected);
        let rejected = normalized_states(states, EvidenceState::Rejected);

        self.ghosts
            .iter()
            .filter(|ghost| {
                let possible = ghost.possible_evidence();
                selected.is_subset(&possible) && rejected.is_disjoint(&possible)
            })
            .map(|ghost| ghost.name.as_str())
            .collect()
    }
}

impl GhostConfig {
    fn possible_evidence(&self) -> BTreeSet<String> {
        self.evidence
            .iter()
            .chain(self.false_evidence.iter())
            .map(|evidence| normalize_name(evidence))
            .collect()
    }
}

pub fn load_or_create(
    path: &Path,
    evidence_config: &[EvidenceConfig],
) -> Result<LoadedGhostKnowledge> {
    if path.exists() {
        let knowledge = load(path)?;
        validate(&knowledge, evidence_config)?;
        return Ok(LoadedGhostKnowledge {
            knowledge,
            created: false,
        });
    }

    let knowledge = default_ghost_knowledge();
    write(path, &knowledge)?;
    validate(&knowledge, evidence_config)?;
    Ok(LoadedGhostKnowledge {
        knowledge,
        created: true,
    })
}

fn load(path: &Path) -> Result<GhostKnowledge> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn write(path: &Path, knowledge: &GhostKnowledge) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let encoded =
        toml::to_string_pretty(knowledge).context("failed to encode starter ghost data")?;
    fs::write(path, encoded).with_context(|| format!("failed to write {}", path.display()))
}

fn validate(knowledge: &GhostKnowledge, evidence_config: &[EvidenceConfig]) -> Result<()> {
    if knowledge.ghosts.is_empty() {
        bail!("ghost data has no ghosts");
    }

    let known_evidence = evidence_config
        .iter()
        .map(|evidence| normalize_name(&evidence.name))
        .collect::<BTreeSet<_>>();

    let mut ghost_names = BTreeSet::new();
    for ghost in &knowledge.ghosts {
        let ghost_name = ghost.name.trim();
        if ghost_name.is_empty() {
            bail!("ghost data contains a ghost with an empty name");
        }

        let normalized_ghost_name = normalize_name(ghost_name);
        if !ghost_names.insert(normalized_ghost_name) {
            bail!("ghost data contains duplicate ghost name {ghost_name:?}");
        }

        if ghost.evidence.is_empty() {
            bail!("{ghost_name} has no evidence entries");
        }

        let mut seen_evidence = BTreeSet::new();
        for evidence in ghost.evidence.iter().chain(ghost.false_evidence.iter()) {
            let normalized = normalize_name(evidence);
            if normalized.is_empty() {
                bail!("{ghost_name} has an empty evidence entry");
            }

            if !known_evidence.contains(&normalized) {
                bail!("{ghost_name} references unknown evidence {evidence:?}");
            }

            if !seen_evidence.insert(normalized) {
                bail!("{ghost_name} repeats evidence {evidence:?}");
            }
        }
    }

    Ok(())
}

fn normalized_states(
    states: &std::collections::BTreeMap<String, EvidenceState>,
    target: EvidenceState,
) -> BTreeSet<String> {
    states
        .iter()
        .filter(|(_, state)| **state == target)
        .map(|(name, _)| normalize_name(name))
        .collect()
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn default_ghost_knowledge() -> GhostKnowledge {
    GhostKnowledge {
        ghosts: vec![
            ghost(
                "Banshee",
                &["D.O.T.S Projector", "Ghost Orb", "Ultraviolet"],
            ),
            ghost("Dayan", &["EMF Level 5", "Ghost Orb", "Spirit Box"]),
            ghost(
                "Demon",
                &["Freezing Temperatures", "Ghost Writing", "Ultraviolet"],
            ),
            ghost(
                "Deogen",
                &["D.O.T.S Projector", "Ghost Writing", "Spirit Box"],
            ),
            ghost("Gallu", &["EMF Level 5", "Spirit Box", "Ultraviolet"]),
            ghost(
                "Goryo",
                &["D.O.T.S Projector", "EMF Level 5", "Ultraviolet"],
            ),
            ghost(
                "Hantu",
                &["Freezing Temperatures", "Ghost Orb", "Ultraviolet"],
            ),
            ghost(
                "Jinn",
                &["EMF Level 5", "Freezing Temperatures", "Ultraviolet"],
            ),
            ghost("Mare", &["Ghost Orb", "Ghost Writing", "Spirit Box"]),
            ghost(
                "Moroi",
                &["Freezing Temperatures", "Ghost Writing", "Spirit Box"],
            ),
            ghost("Myling", &["EMF Level 5", "Ghost Writing", "Ultraviolet"]),
            ghost("Obake", &["EMF Level 5", "Ghost Orb", "Ultraviolet"]),
            ghost(
                "Obambo",
                &["D.O.T.S Projector", "Ghost Writing", "Ultraviolet"],
            ),
            ghost(
                "Oni",
                &["D.O.T.S Projector", "EMF Level 5", "Freezing Temperatures"],
            ),
            ghost(
                "Onryo",
                &["Freezing Temperatures", "Ghost Orb", "Spirit Box"],
            ),
            ghost(
                "Phantom",
                &["D.O.T.S Projector", "Spirit Box", "Ultraviolet"],
            ),
            ghost(
                "Poltergeist",
                &["Ghost Writing", "Spirit Box", "Ultraviolet"],
            ),
            ghost("Raiju", &["D.O.T.S Projector", "EMF Level 5", "Ghost Orb"]),
            ghost(
                "Revenant",
                &["Freezing Temperatures", "Ghost Orb", "Ghost Writing"],
            ),
            ghost(
                "Shade",
                &[
                    "D.O.T.S Projector",
                    "Freezing Temperatures",
                    "Ghost Writing",
                ],
            ),
            ghost("Spirit", &["EMF Level 5", "Ghost Writing", "Spirit Box"]),
            ghost(
                "Thaye",
                &["D.O.T.S Projector", "Ghost Orb", "Ghost Writing"],
            ),
            ghost_with_false(
                "The Mimic",
                &["Freezing Temperatures", "Spirit Box", "Ultraviolet"],
                &["Ghost Orb"],
            ),
            ghost(
                "The Twins",
                &["EMF Level 5", "Freezing Temperatures", "Spirit Box"],
            ),
            ghost(
                "Wraith",
                &["D.O.T.S Projector", "EMF Level 5", "Spirit Box"],
            ),
            ghost("Yokai", &["D.O.T.S Projector", "Ghost Orb", "Spirit Box"]),
            ghost(
                "Yurei",
                &["D.O.T.S Projector", "Freezing Temperatures", "Ghost Orb"],
            ),
        ],
    }
}

fn ghost(name: &str, evidence: &[&str]) -> GhostConfig {
    ghost_with_false(name, evidence, &[])
}

fn ghost_with_false(name: &str, evidence: &[&str], false_evidence: &[&str]) -> GhostConfig {
    GhostConfig {
        name: name.to_string(),
        evidence: evidence.iter().map(|entry| (*entry).to_string()).collect(),
        false_evidence: false_evidence
            .iter()
            .map(|entry| (*entry).to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::{ColorMatcher, RegionMatcher};

    use super::*;

    #[test]
    fn filters_ghosts_from_selected_and_rejected_evidence() {
        let knowledge = default_ghost_knowledge();
        let mut states = BTreeMap::new();
        states.insert("Spirit Box".to_string(), EvidenceState::Selected);
        states.insert("Ghost Writing".to_string(), EvidenceState::Selected);
        states.insert("EMF Level 5".to_string(), EvidenceState::Rejected);
        states.insert("Ultraviolet".to_string(), EvidenceState::Rejected);

        assert_eq!(
            knowledge.possible_ghosts(&states),
            vec!["Deogen", "Mare", "Moroi"]
        );
    }

    #[test]
    fn treats_mimic_ghost_orbs_as_possible_false_evidence() {
        let knowledge = default_ghost_knowledge();
        let mut states = BTreeMap::new();
        states.insert("Freezing Temperatures".to_string(), EvidenceState::Selected);
        states.insert("Spirit Box".to_string(), EvidenceState::Selected);
        states.insert("Ultraviolet".to_string(), EvidenceState::Selected);
        states.insert("Ghost Orb".to_string(), EvidenceState::Selected);

        assert_eq!(knowledge.possible_ghosts(&states), vec!["The Mimic"]);

        states.insert("Ghost Orb".to_string(), EvidenceState::Rejected);
        assert!(knowledge.possible_ghosts(&states).is_empty());
    }

    #[test]
    fn accepts_punctuation_variants_in_evidence_names() {
        let knowledge = GhostKnowledge {
            ghosts: vec![ghost("Dots Ghost", &["D.O.T.S. Projector"])],
        };
        let mut states = BTreeMap::new();
        states.insert("D.O.T.S Projector".to_string(), EvidenceState::Selected);

        assert_eq!(knowledge.possible_ghosts(&states), vec!["Dots Ghost"]);
    }

    #[test]
    fn creates_ghost_data_once_and_reuses_it() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("phasmo_ghosts_test_{}_{}.toml", process::id(), id));

        let first = load_or_create(&path, &evidence_items()).unwrap();
        assert!(first.created);
        assert_eq!(first.knowledge.ghosts.len(), 27);

        let second = load_or_create(&path, &evidence_items()).unwrap();
        assert!(!second.created);

        fs::remove_file(path).unwrap();
    }

    fn evidence_items() -> Vec<EvidenceConfig> {
        [
            "EMF Level 5",
            "D.O.T.S Projector",
            "Ultraviolet",
            "Freezing Temperatures",
            "Ghost Orb",
            "Ghost Writing",
            "Spirit Box",
        ]
        .iter()
        .map(|name| EvidenceConfig {
            name: (*name).to_string(),
            selected: region_matcher(),
            rejected: region_matcher(),
        })
        .collect()
    }

    fn region_matcher() -> RegionMatcher {
        RegionMatcher {
            x_pct: 0.0,
            y_pct: 0.0,
            w_pct: 0.1,
            h_pct: 0.1,
            color: ColorMatcher {
                r: 0,
                g: 0,
                b: 0,
                tolerance: 0,
                min_ratio: 0.5,
            },
        }
    }
}
