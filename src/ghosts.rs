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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<GhostTraitConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GhostConfig {
    pub name: String,
    pub evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub false_evidence: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GhostTraitConfig {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub possible_ghosts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_ghosts: Vec<String>,
}

impl GhostKnowledge {
    pub fn possible_ghosts<'a>(
        &'a self,
        states: &std::collections::BTreeMap<String, EvidenceState>,
    ) -> Vec<&'a str> {
        self.possible_ghosts_with_traits(states, &[])
    }

    pub fn possible_ghosts_with_traits<'a>(
        &'a self,
        states: &std::collections::BTreeMap<String, EvidenceState>,
        selected_traits: &[String],
    ) -> Vec<&'a str> {
        let selected = normalized_states(states, EvidenceState::Selected);
        let rejected = normalized_states(states, EvidenceState::Rejected);
        let selected_traits = selected_traits
            .iter()
            .map(|trait_id| normalize_name(trait_id))
            .collect::<BTreeSet<_>>();

        self.ghosts
            .iter()
            .filter(|ghost| {
                let possible = ghost.possible_evidence();
                selected.is_subset(&possible)
                    && rejected.is_disjoint(&possible)
                    && self.ghost_matches_selected_traits(ghost, &selected_traits)
            })
            .map(|ghost| ghost.name.as_str())
            .collect()
    }

    fn ghost_matches_selected_traits(
        &self,
        ghost: &GhostConfig,
        selected_traits: &BTreeSet<String>,
    ) -> bool {
        if selected_traits.is_empty() {
            return true;
        }

        let ghost_name = normalize_name(&ghost.name);
        self.traits
            .iter()
            .filter(|entry| selected_traits.contains(&normalize_name(&entry.id)))
            .all(|entry| {
                let possible_ghosts = normalized_names(&entry.possible_ghosts);
                let excluded_ghosts = normalized_names(&entry.excluded_ghosts);

                !excluded_ghosts.contains(&ghost_name)
                    && (possible_ghosts.is_empty() || possible_ghosts.contains(&ghost_name))
            })
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
        let mut knowledge = load(path)?;
        if migrate_ghost_traits(&mut knowledge.traits) {
            write(path, &knowledge)?;
        }
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

    let mut trait_ids = BTreeSet::new();
    for trait_config in &knowledge.traits {
        let trait_id = trait_config.id.trim();
        if trait_id.is_empty() {
            bail!("ghost trait data contains a trait with an empty id");
        }

        let normalized_trait_id = normalize_name(trait_id);
        if !trait_ids.insert(normalized_trait_id) {
            bail!("ghost trait data contains duplicate trait id {trait_id:?}");
        }

        if trait_config.label.trim().is_empty() {
            bail!("{trait_id} has an empty label");
        }

        if trait_config.possible_ghosts.is_empty() && trait_config.excluded_ghosts.is_empty() {
            bail!("{trait_id} must list possible_ghosts or excluded_ghosts");
        }

        let possible_ghosts = normalized_names(&trait_config.possible_ghosts);
        let excluded_ghosts = normalized_names(&trait_config.excluded_ghosts);

        for ghost_name in trait_config
            .possible_ghosts
            .iter()
            .chain(trait_config.excluded_ghosts.iter())
        {
            let normalized = normalize_name(ghost_name);
            if normalized.is_empty() {
                bail!("{trait_id} references an empty ghost name");
            }

            if !ghost_names.contains(&normalized) {
                bail!("{trait_id} references unknown ghost {ghost_name:?}");
            }
        }

        if !possible_ghosts.is_disjoint(&excluded_ghosts) {
            bail!("{trait_id} lists the same ghost as possible and excluded");
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

fn normalized_names(names: &[String]) -> BTreeSet<String> {
    names.iter().map(|name| normalize_name(name)).collect()
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
        traits: default_ghost_traits(),
    }
}

fn default_ghost_traits() -> Vec<GhostTraitConfig> {
    vec![
        ghost_trait(
            "banshee_scream",
            "Banshee scream recorded",
            "Unique scream heard on Parabolic Microphone or Sound Recorder.",
            &["Banshee"],
            &[],
        ),
        ghost_trait(
            "banshee_ignores_non_target",
            "Ignores non-target in hunt",
            "During a hunt, the ghost passes through or ignores one player while its target is inside.",
            &["Banshee"],
            &[],
        ),
        ghost_trait(
            "female_only_ghost",
            "Female-only model/name",
            "The ghost has a female model/name; this keeps female-only ghosts in consideration.",
            &["Banshee", "Dayan"],
            &[],
        ),
        ghost_trait(
            "dayan_movement_speed_change",
            "Walking speed: moving/still player changes speed",
            "Within 10m, a moving player makes it fast and a still player makes it slow.",
            &["Dayan"],
            &[],
        ),
        ghost_trait(
            "demon_hunts_before_90s_incense",
            "Hunts before 90s after incense",
            "The ghost starts a normal hunt less than 90 seconds after incense.",
            &["Demon"],
            &[],
        ),
        ghost_trait(
            "demon_crucifix_extra_range",
            "Crucifix burned from extra range",
            "A crucifix prevents a hunt from farther away than normal.",
            &["Demon"],
            &[],
        ),
        ghost_trait(
            "deogen_finds_and_slows_near_player",
            "Walking speed: fast far, crawl slow close",
            "The ghost always finds players, is fast at distance, then becomes extremely slow close up.",
            &["Deogen"],
            &[],
        ),
        ghost_trait(
            "gallu_salt_enrage",
            "Walking speed: salt/incense enrage",
            "Salt, incense, or crucifix behavior appears to push the ghost into a faster enraged state.",
            &["Gallu"],
            &[],
        ),
        ghost_trait(
            "gallu_refuses_salt_while_enraged",
            "Salt not disturbed while enraged",
            "The ghost can cross salt without disturbing it after entering an enraged state.",
            &["Gallu", "Wraith"],
            &[],
        ),
        ghost_trait(
            "goryo_camera_only_dots",
            "D.O.T.S only visible on camera",
            "D.O.T.S silhouette is visible through a video camera but not with the naked eye.",
            &["Goryo"],
            &[],
        ),
        ghost_trait(
            "dots_seen_without_camera",
            "D.O.T.S seen without camera",
            "D.O.T.S silhouette was visible directly without a video camera.",
            &[],
            &["Goryo"],
        ),
        ghost_trait(
            "hantu_temperature_speed",
            "Walking speed: cold fast, warm slow",
            "The ghost is faster in cold rooms, slower in warm rooms, and has no line-of-sight acceleration.",
            &["Hantu"],
            &[],
        ),
        ghost_trait(
            "hantu_freezing_breath_hunt",
            "Freezing breath during hunt",
            "The ghost shows freezing breath during a hunt when the breaker is off.",
            &["Hantu"],
            &[],
        ),
        ghost_trait(
            "jinn_breaker_off",
            "Breaker manually turned off",
            "The ghost directly turned the fuse box off rather than overloading it.",
            &[],
            &["Jinn"],
        ),
        ghost_trait(
            "jinn_breaker_emf_sanity_zap",
            "Breaker EMF after sanity zap",
            "A sudden sanity drain produces EMF at the breaker while the fuse box is on.",
            &["Jinn"],
            &[],
        ),
        ghost_trait(
            "jinn_los_breaker_speed",
            "Walking speed: breaker-on LOS burst",
            "With the fuse box on, the ghost speeds to about 2.5m/s with line of sight from distance.",
            &["Jinn"],
            &[],
        ),
        ghost_trait(
            "mare_instant_light_off",
            "Light instantly turned back off",
            "A light switch is turned off within about one second after a player turns it on.",
            &["Mare"],
            &[],
        ),
        ghost_trait(
            "ghost_turned_light_on",
            "Ghost turned light on",
            "The ghost directly turned on a light switch.",
            &[],
            &["Mare"],
        ),
        ghost_trait(
            "moroi_speed_changes_with_sanity",
            "Walking speed: changes with sanity",
            "Roaming speed changes as average sanity changes, including after sanity medication.",
            &["Moroi"],
            &[],
        ),
        ghost_trait(
            "moroi_longer_incense_blind",
            "Incense blind lasted longer",
            "Incense blinds the ghost for longer than normal during a hunt.",
            &["Moroi"],
            &[],
        ),
        ghost_trait(
            "myling_quiet_footsteps",
            "Quiet hunt footsteps",
            "Footsteps and vocalization are only audible close to the ghost, near electronics interference range.",
            &["Myling"],
            &[],
        ),
        ghost_trait(
            "obake_unique_uv_print",
            "Unique UV print",
            "Six-finger handprint, two light-switch fingerprints, or other unique Obake UV pattern.",
            &["Obake"],
            &[],
        ),
        ghost_trait(
            "obake_shapeshift_hunt",
            "Shapeshift during hunt",
            "The ghost briefly changes model while blinking during a hunt.",
            &["Obake"],
            &[],
        ),
        ghost_trait(
            "obambo_phase_speed_change",
            "Walking speed: calm/aggressive phases",
            "The ghost snaps between slower calm and faster aggressive movement, sometimes mid-hunt.",
            &["Obambo"],
            &[],
        ),
        ghost_trait(
            "obambo_fast_short_hunt",
            "Fast hunt ended 20% early",
            "A hunt that began fast lasted about 20% less than expected.",
            &["Obambo"],
            &[],
        ),
        ghost_trait(
            "oni_visible_longer_hunt",
            "More visible during hunt",
            "The ghost flickers less invisibly and remains visible longer during hunts.",
            &["Oni"],
            &[],
        ),
        ghost_trait(
            "mist_airball_event",
            "Mist airball event happened",
            "A mist-form ghost event happened, which rules out Oni.",
            &[],
            &["Oni"],
        ),
        ghost_trait(
            "onryo_flame_prevented_hunt",
            "Flame prevented hunt",
            "A nearby lit flame is blown out instead of the ghost hunting or burning a crucifix.",
            &["Onryo"],
            &[],
        ),
        ghost_trait(
            "onryo_hunt_after_flame_blowout",
            "Hunt after flame blowout",
            "The ghost hunts soon after extinguishing a flame near it.",
            &["Onryo"],
            &[],
        ),
        ghost_trait(
            "phantom_photo_disappears",
            "Ghost vanished from photo",
            "A ghost photo makes the ghost disappear or the photo lacks the visible ghost/interference.",
            &["Phantom"],
            &[],
        ),
        ghost_trait(
            "phantom_long_invisible_blinks",
            "Long invisible hunt blinks",
            "The ghost is invisible for longer than normal between hunt blinks.",
            &["Phantom"],
            &[],
        ),
        ghost_trait(
            "poltergeist_multi_throw",
            "Multiple objects thrown",
            "Several objects are thrown at once, or an object is thrown with unusual force.",
            &["Poltergeist"],
            &[],
        ),
        ghost_trait(
            "raiju_electronics_speed",
            "Walking speed: fast near electronics",
            "The ghost becomes about 2.5m/s near active electronic equipment.",
            &["Raiju"],
            &[],
        ),
        ghost_trait(
            "raiju_long_electronic_disruption",
            "Electronics disrupted at long range",
            "Electronics disrupt from farther away than normal during a hunt.",
            &["Raiju"],
            &[],
        ),
        ghost_trait(
            "revenant_slow_hidden_fast_detected",
            "Walking speed: slow hidden, fast detected",
            "The ghost is very slow when nobody is detected, then rushes quickly after detecting a player.",
            &["Revenant"],
            &[],
        ),
        ghost_trait(
            "shade_no_same_room_activity",
            "No activity with player in room",
            "The ghost refuses interactions, events, and hunts while a player is in its room.",
            &["Shade"],
            &[],
        ),
        ghost_trait(
            "same_room_activity_seen",
            "Activity with player in room",
            "The ghost interacted, evented, or hunted while a player was in its current room.",
            &[],
            &["Shade"],
        ),
        ghost_trait(
            "spirit_no_hunt_180s_incense",
            "No hunt for 180s after incense",
            "Incense prevents normal hunts for about 180 seconds.",
            &["Spirit"],
            &[],
        ),
        ghost_trait(
            "hunted_before_180s_after_incense",
            "Hunted before 180s after incense",
            "The ghost started a normal hunt before a Spirit-length incense timer expired.",
            &[],
            &["Spirit"],
        ),
        ghost_trait(
            "thaye_fast_then_ages_slow",
            "Walking speed: starts fast, ages slow",
            "The ghost starts very fast and becomes slower/less active as time is spent near it.",
            &["Thaye"],
            &[],
        ),
        ghost_trait(
            "twins_back_to_back_interactions",
            "Back-to-back interactions under 2s",
            "Two separate interactions happen with less than two seconds between them.",
            &["The Twins"],
            &[],
        ),
        ghost_trait(
            "twins_two_base_speeds",
            "Walking speed: two base speeds",
            "Hunts alternate between a slightly slow and slightly fast base speed.",
            &["The Twins"],
            &[],
        ),
        ghost_trait(
            "mimic_ghost_orbs",
            "Ghost Orbs as false evidence",
            "Ghost Orbs appear in addition to the ghost's real evidence set.",
            &["The Mimic"],
            &[],
        ),
        ghost_trait(
            "mimic_behavior_changes",
            "Trait changes between hunts",
            "The ghost shows different ghost-specific behaviors across hunts or over time.",
            &["The Mimic"],
            &[],
        ),
        ghost_trait(
            "wraith_does_not_disturb_salt",
            "Salt not disturbed when crossed",
            "The ghost crosses a salt line or pile without disturbing it.",
            &["Wraith", "Gallu"],
            &[],
        ),
        ghost_trait(
            "yokai_short_hearing_range",
            "Short voice/electronics detection",
            "During a hunt, voice or held electronics only attract the ghost at very close range.",
            &["Yokai"],
            &[],
        ),
        ghost_trait(
            "yokai_talking_early_hunt",
            "Talking caused early hunt",
            "The ghost hunts around high sanity while players are talking near it.",
            &["Yokai"],
            &[],
        ),
        ghost_trait(
            "yurei_full_door_close",
            "Full smooth door close",
            "A door fully closes smoothly without creaking outside a hunt or ghost event.",
            &["Yurei"],
            &[],
        ),
        ghost_trait(
            "yurei_left_room_after_incense",
            "Left room after incense",
            "The ghost leaves its room within 90 seconds after being incensed there.",
            &[],
            &["Yurei"],
        ),
    ]
}

fn migrate_ghost_traits(traits: &mut Vec<GhostTraitConfig>) -> bool {
    let before = traits.len();
    traits.retain(|entry| {
        !matches!(
            entry.id.as_str(),
            "two_salts_within_two_seconds" | "breaker_turned_off" | "no_salt_footsteps"
        )
    });

    let removed_deprecated = traits.len() != before;
    let should_seed_defaults = traits.is_empty() || removed_deprecated;
    let mut changed = removed_deprecated;
    if !should_seed_defaults {
        return changed;
    }

    let mut existing_ids = traits
        .iter()
        .map(|entry| normalize_name(&entry.id))
        .collect::<BTreeSet<_>>();

    for default_trait in default_ghost_traits() {
        if existing_ids.insert(normalize_name(&default_trait.id)) {
            traits.push(default_trait);
            changed = true;
        }
    }

    changed
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

fn ghost_trait(
    id: &str,
    label: &str,
    description: &str,
    possible_ghosts: &[&str],
    excluded_ghosts: &[&str],
) -> GhostTraitConfig {
    GhostTraitConfig {
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        possible_ghosts: possible_ghosts
            .iter()
            .map(|entry| (*entry).to_string())
            .collect(),
        excluded_ghosts: excluded_ghosts
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
            traits: Vec::new(),
        };
        let mut states = BTreeMap::new();
        states.insert("D.O.T.S Projector".to_string(), EvidenceState::Selected);

        assert_eq!(knowledge.possible_ghosts(&states), vec!["Dots Ghost"]);
    }

    #[test]
    fn filters_ghosts_from_selected_traits() {
        let knowledge = default_ghost_knowledge();
        let states = BTreeMap::new();

        assert_eq!(
            knowledge.possible_ghosts_with_traits(
                &states,
                &["twins_back_to_back_interactions".to_string()]
            ),
            vec!["The Twins"]
        );
        assert!(
            !knowledge
                .possible_ghosts_with_traits(&states, &["jinn_breaker_off".to_string()])
                .contains(&"Jinn")
        );
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
        assert!(first.knowledge.traits.len() > 40);

        let second = load_or_create(&path, &evidence_items()).unwrap();
        assert!(!second.created);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn migrates_existing_ghost_data_without_traits() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "phasmo_ghosts_migration_test_{}_{}.toml",
            process::id(),
            id
        ));
        let mut legacy_knowledge = default_ghost_knowledge();
        legacy_knowledge.traits.clear();
        write(&path, &legacy_knowledge).unwrap();

        let loaded = load_or_create(&path, &evidence_items()).unwrap();

        assert!(!loaded.created);
        assert!(loaded.knowledge.traits.len() > 40);
        assert!(
            loaded
                .knowledge
                .traits
                .iter()
                .any(|entry| entry.id == "twins_back_to_back_interactions")
        );

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn replaces_deprecated_starter_traits_during_migration() {
        let mut traits = vec![ghost_trait(
            "two_salts_within_two_seconds",
            "2 salts within 2 seconds",
            "Deprecated starter trait.",
            &["The Twins"],
            &[],
        )];

        assert!(migrate_ghost_traits(&mut traits));

        let trait_ids = traits
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>();
        assert!(!trait_ids.contains(&"two_salts_within_two_seconds"));
        assert!(trait_ids.contains(&"twins_back_to_back_interactions"));
        assert!(traits.len() > 40);
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
