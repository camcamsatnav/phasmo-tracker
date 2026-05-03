import { stateMeta } from "./stateMeta";

export type ActivityTone = "info" | "good" | "warn";

export interface ActivityEntry {
  id: number;
  tone: ActivityTone;
  text: string;
}

export interface ImageSize {
  width: number;
  height: number;
}

export interface EvidenceGroups {
  unknown: string[];
  clear: string[];
  selected: string[];
  rejected: string[];
}

export interface TrackerViewState {
  running: boolean;
  status: string;
  evidence: EvidenceItemSnapshot[];
  selectedEvidence: string[];
  rejectedEvidence: string[];
  unofficialTraits: GhostTraitSnapshot[];
  selectedTraitIds: string[];
  possibleGhosts: string[];
  ghostRequirements: Record<string, GhostSnapshot>;
  activity: ActivityEntry[];
  elapsedSecs: number | null;
  imageSize: ImageSize | null;
  lastChange: string;
}

export const defaultEvidenceNames = [
  "EMF Level 5",
  "D.O.T.S Projector",
  "Ultraviolet",
  "Freezing Temperatures",
  "Ghost Orb",
  "Ghost Writing",
  "Spirit Box",
];

export const defaultGhosts: GhostSnapshot[] = [
  ghost("Banshee", ["D.O.T.S Projector", "Ghost Orb", "Ultraviolet"]),
  ghost("Dayan", ["EMF Level 5", "Ghost Orb", "Spirit Box"]),
  ghost("Demon", ["Freezing Temperatures", "Ghost Writing", "Ultraviolet"]),
  ghost("Deogen", ["D.O.T.S Projector", "Ghost Writing", "Spirit Box"]),
  ghost("Gallu", ["EMF Level 5", "Spirit Box", "Ultraviolet"]),
  ghost("Goryo", ["D.O.T.S Projector", "EMF Level 5", "Ultraviolet"]),
  ghost("Hantu", ["Freezing Temperatures", "Ghost Orb", "Ultraviolet"]),
  ghost("Jinn", ["EMF Level 5", "Freezing Temperatures", "Ultraviolet"]),
  ghost("Mare", ["Ghost Orb", "Ghost Writing", "Spirit Box"]),
  ghost("Moroi", ["Freezing Temperatures", "Ghost Writing", "Spirit Box"]),
  ghost("Myling", ["EMF Level 5", "Ghost Writing", "Ultraviolet"]),
  ghost("Obake", ["EMF Level 5", "Ghost Orb", "Ultraviolet"]),
  ghost("Obambo", ["D.O.T.S Projector", "Ghost Writing", "Ultraviolet"]),
  ghost("Oni", ["D.O.T.S Projector", "EMF Level 5", "Freezing Temperatures"]),
  ghost("Onryo", ["Freezing Temperatures", "Ghost Orb", "Spirit Box"]),
  ghost("Phantom", ["D.O.T.S Projector", "Spirit Box", "Ultraviolet"]),
  ghost("Poltergeist", ["Ghost Writing", "Spirit Box", "Ultraviolet"]),
  ghost("Raiju", ["D.O.T.S Projector", "EMF Level 5", "Ghost Orb"]),
  ghost("Revenant", ["Freezing Temperatures", "Ghost Orb", "Ghost Writing"]),
  ghost("Shade", ["D.O.T.S Projector", "Freezing Temperatures", "Ghost Writing"]),
  ghost("Spirit", ["EMF Level 5", "Ghost Writing", "Spirit Box"]),
  ghost("Thaye", ["D.O.T.S Projector", "Ghost Orb", "Ghost Writing"]),
  ghost("The Mimic", ["Freezing Temperatures", "Spirit Box", "Ultraviolet"], [
    "Ghost Orb",
  ]),
  ghost("The Twins", ["EMF Level 5", "Freezing Temperatures", "Spirit Box"]),
  ghost("Wraith", ["D.O.T.S Projector", "EMF Level 5", "Spirit Box"]),
  ghost("Yokai", ["D.O.T.S Projector", "Ghost Orb", "Spirit Box"]),
  ghost("Yurei", ["D.O.T.S Projector", "Freezing Temperatures", "Ghost Orb"]),
];

export const defaultUnofficialTraits: GhostTraitSnapshot[] = [
  ghostTrait("banshee_scream", "Banshee scream recorded", "Unique scream heard on Parabolic Microphone or Sound Recorder.", ["Banshee"]),
  ghostTrait("banshee_ignores_non_target", "Ignores non-target in hunt", "During a hunt, the ghost passes through or ignores one player while its target is inside.", ["Banshee"]),
  ghostTrait("female_only_ghost", "Female-only model/name", "The ghost has a female model/name; this keeps female-only ghosts in consideration.", ["Banshee", "Dayan"]),
  ghostTrait("dayan_movement_speed_change", "Walking speed: moving/still player changes speed", "Within 10m, a moving player makes it fast and a still player makes it slow.", ["Dayan"]),
  ghostTrait("demon_hunts_before_90s_incense", "Hunts before 90s after incense", "The ghost starts a normal hunt less than 90 seconds after incense.", ["Demon"]),
  ghostTrait("demon_crucifix_extra_range", "Crucifix burned from extra range", "A crucifix prevents a hunt from farther away than normal.", ["Demon"]),
  ghostTrait("deogen_finds_and_slows_near_player", "Walking speed: fast far, crawl slow close", "The ghost always finds players, is fast at distance, then becomes extremely slow close up.", ["Deogen"]),
  ghostTrait("gallu_salt_enrage", "Walking speed: salt/incense enrage", "Salt, incense, or crucifix behavior appears to push the ghost into a faster enraged state.", ["Gallu"]),
  ghostTrait("gallu_refuses_salt_while_enraged", "Salt not disturbed while enraged", "The ghost can cross salt without disturbing it after entering an enraged state.", ["Gallu", "Wraith"]),
  ghostTrait("goryo_camera_only_dots", "D.O.T.S only visible on camera", "D.O.T.S silhouette is visible through a video camera but not with the naked eye.", ["Goryo"]),
  ghostTrait("dots_seen_without_camera", "D.O.T.S seen without camera", "D.O.T.S silhouette was visible directly without a video camera.", [], ["Goryo"]),
  ghostTrait("hantu_temperature_speed", "Walking speed: cold fast, warm slow", "The ghost is faster in cold rooms, slower in warm rooms, and has no line-of-sight acceleration.", ["Hantu"]),
  ghostTrait("hantu_freezing_breath_hunt", "Freezing breath during hunt", "The ghost shows freezing breath during a hunt when the breaker is off.", ["Hantu"]),
  ghostTrait("jinn_breaker_off", "Breaker manually turned off", "The ghost directly turned the fuse box off rather than overloading it.", [], ["Jinn"]),
  ghostTrait("jinn_breaker_emf_sanity_zap", "Breaker EMF after sanity zap", "A sudden sanity drain produces EMF at the breaker while the fuse box is on.", ["Jinn"]),
  ghostTrait("jinn_los_breaker_speed", "Walking speed: breaker-on LOS burst", "With the fuse box on, the ghost speeds to about 2.5m/s with line of sight from distance.", ["Jinn"]),
  ghostTrait("mare_instant_light_off", "Light instantly turned back off", "A light switch is turned off within about one second after a player turns it on.", ["Mare"]),
  ghostTrait("ghost_turned_light_on", "Ghost turned light on", "The ghost directly turned on a light switch.", [], ["Mare"]),
  ghostTrait("moroi_speed_changes_with_sanity", "Walking speed: changes with sanity", "Roaming speed changes as average sanity changes, including after sanity medication.", ["Moroi"]),
  ghostTrait("moroi_longer_incense_blind", "Incense blind lasted longer", "Incense blinds the ghost for longer than normal during a hunt.", ["Moroi"]),
  ghostTrait("myling_quiet_footsteps", "Quiet hunt footsteps", "Footsteps and vocalization are only audible close to the ghost, near electronics interference range.", ["Myling"]),
  ghostTrait("obake_unique_uv_print", "Unique UV print", "Six-finger handprint, two light-switch fingerprints, or other unique Obake UV pattern.", ["Obake"]),
  ghostTrait("obake_shapeshift_hunt", "Shapeshift during hunt", "The ghost briefly changes model while blinking during a hunt.", ["Obake"]),
  ghostTrait("obambo_phase_speed_change", "Walking speed: calm/aggressive phases", "The ghost snaps between slower calm and faster aggressive movement, sometimes mid-hunt.", ["Obambo"]),
  ghostTrait("obambo_fast_short_hunt", "Fast hunt ended 20% early", "A hunt that began fast lasted about 20% less than expected.", ["Obambo"]),
  ghostTrait("oni_visible_longer_hunt", "More visible during hunt", "The ghost flickers less invisibly and remains visible longer during hunts.", ["Oni"]),
  ghostTrait("mist_airball_event", "Mist airball event happened", "A mist-form ghost event happened, which rules out Oni.", [], ["Oni"]),
  ghostTrait("onryo_flame_prevented_hunt", "Flame prevented hunt", "A nearby lit flame is blown out instead of the ghost hunting or burning a crucifix.", ["Onryo"]),
  ghostTrait("onryo_hunt_after_flame_blowout", "Hunt after flame blowout", "The ghost hunts soon after extinguishing a flame near it.", ["Onryo"]),
  ghostTrait("phantom_photo_disappears", "Ghost vanished from photo", "A ghost photo makes the ghost disappear or the photo lacks the visible ghost/interference.", ["Phantom"]),
  ghostTrait("phantom_long_invisible_blinks", "Long invisible hunt blinks", "The ghost is invisible for longer than normal between hunt blinks.", ["Phantom"]),
  ghostTrait("poltergeist_multi_throw", "Multiple objects thrown", "Several objects are thrown at once, or an object is thrown with unusual force.", ["Poltergeist"]),
  ghostTrait("raiju_electronics_speed", "Walking speed: fast near electronics", "The ghost becomes about 2.5m/s near active electronic equipment.", ["Raiju"]),
  ghostTrait("raiju_long_electronic_disruption", "Electronics disrupted at long range", "Electronics disrupt from farther away than normal during a hunt.", ["Raiju"]),
  ghostTrait("revenant_slow_hidden_fast_detected", "Walking speed: slow hidden, fast detected", "The ghost is very slow when nobody is detected, then rushes quickly after detecting a player.", ["Revenant"]),
  ghostTrait("shade_no_same_room_activity", "No activity with player in room", "The ghost refuses interactions, events, and hunts while a player is in its room.", ["Shade"]),
  ghostTrait("same_room_activity_seen", "Activity with player in room", "The ghost interacted, evented, or hunted while a player was in its current room.", [], ["Shade"]),
  ghostTrait("spirit_no_hunt_180s_incense", "No hunt for 180s after incense", "Incense prevents normal hunts for about 180 seconds.", ["Spirit"]),
  ghostTrait("hunted_before_180s_after_incense", "Hunted before 180s after incense", "The ghost started a normal hunt before a Spirit-length incense timer expired.", [], ["Spirit"]),
  ghostTrait("thaye_fast_then_ages_slow", "Walking speed: starts fast, ages slow", "The ghost starts very fast and becomes slower/less active as time is spent near it.", ["Thaye"]),
  ghostTrait("twins_back_to_back_interactions", "Back-to-back interactions under 2s", "Two separate interactions happen with less than two seconds between them.", ["The Twins"]),
  ghostTrait("twins_two_base_speeds", "Walking speed: two base speeds", "Hunts alternate between a slightly slow and slightly fast base speed.", ["The Twins"]),
  ghostTrait("mimic_ghost_orbs", "Ghost Orbs as false evidence", "Ghost Orbs appear in addition to the ghost's real evidence set.", ["The Mimic"]),
  ghostTrait("mimic_behavior_changes", "Trait changes between hunts", "The ghost shows different ghost-specific behaviors across hunts or over time.", ["The Mimic"]),
  ghostTrait("wraith_does_not_disturb_salt", "Salt not disturbed when crossed", "The ghost crosses a salt line or pile without disturbing it.", ["Wraith", "Gallu"]),
  ghostTrait("yokai_short_hearing_range", "Short voice/electronics detection", "During a hunt, voice or held electronics only attract the ghost at very close range.", ["Yokai"]),
  ghostTrait("yokai_talking_early_hunt", "Talking caused early hunt", "The ghost hunts around high sanity while players are talking near it.", ["Yokai"]),
  ghostTrait("yurei_full_door_close", "Full smooth door close", "A door fully closes smoothly without creaking outside a hunt or ghost event.", ["Yurei"]),
  ghostTrait("yurei_left_room_after_incense", "Left room after incense", "The ghost leaves its room within 90 seconds after being incensed there.", [], ["Yurei"]),
];

const maxActivityEntries = 8;
const deprecatedStarterTraitIds = new Set([
  "two_salts_within_two_seconds",
  "breaker_turned_off",
  "no_salt_footsteps",
]);

function ghost(
  name: string,
  evidence: string[],
  false_evidence: string[] = [],
): GhostSnapshot {
  return { name, evidence, false_evidence };
}

function ghostRequirementsByName(ghosts: GhostSnapshot[]): Record<string, GhostSnapshot> {
  return Object.fromEntries(ghosts.map((ghost) => [ghost.name, ghost]));
}

function ghostTrait(
  id: string,
  label: string,
  description: string,
  possible_ghosts: string[],
  excluded_ghosts: string[] = [],
): GhostTraitSnapshot {
  return { id, label, description, possible_ghosts, excluded_ghosts };
}

export function initialEvidence(): EvidenceItemSnapshot[] {
  return defaultEvidenceNames.map((name) => ({ name, state: "clear" }));
}

export function defaultGhostNames(): string[] {
  return defaultGhosts.map((ghost) => ghost.name);
}

export function createInitialTrackerViewState(hasBridge: boolean): TrackerViewState {
  return {
    running: false,
    status: hasBridge ? "Starting tracker" : "Open in Electron",
    evidence: initialEvidence(),
    selectedEvidence: [],
    rejectedEvidence: [],
    unofficialTraits: mergeDefaultUnofficialTraits(defaultUnofficialTraits),
    selectedTraitIds: [],
    possibleGhosts: defaultGhostNames(),
    ghostRequirements: ghostRequirementsByName(defaultGhosts),
    activity: [],
    elapsedSecs: null,
    imageSize: null,
    lastChange: "None yet",
  };
}

export function groupEvidenceByState(evidence: EvidenceItemSnapshot[]): EvidenceGroups {
  return evidence.reduce<EvidenceGroups>(
    (groups, item) => {
      groups[item.state].push(item.name);
      return groups;
    },
    { unknown: [], clear: [], selected: [], rejected: [] },
  );
}

export function addActivity(
  state: TrackerViewState,
  tone: ActivityTone,
  text: string,
  id: number,
): TrackerViewState {
  return {
    ...state,
    activity: [{ id, tone, text }, ...state.activity].slice(0, maxActivityEntries),
  };
}

export function applyTrackerEvent(
  state: TrackerViewState,
  event: TrackerEvent,
  activityId: number,
): TrackerViewState {
  switch (event.type) {
    case "tracker_started":
      return addActivity(
        {
          ...state,
          evidence: event.evidence.map((name) => ({ name, state: "clear" })),
          selectedEvidence: [],
          rejectedEvidence: [],
          unofficialTraits: mergeDefaultUnofficialTraits(event.traits ?? []),
          selectedTraitIds: [],
          possibleGhosts: event.ghosts.map((ghost) => ghost.name),
          ghostRequirements: ghostRequirementsByName(event.ghosts),
          status: "Looking for Phasmophobia",
          imageSize: null,
          lastChange: "None yet",
        },
        "info",
        "Tracker started",
        activityId,
      );
    case "config_created":
      return addActivity(state, "info", `Created ${event.path}`, activityId);
    case "ghost_data_created":
      return addActivity(state, "info", `Created ${event.path}`, activityId);
    case "window_search_error":
      return addActivity(
        { ...state, status: "Waiting for Phasmophobia window" },
        "warn",
        event.message,
        activityId,
      );
    case "page_visibility":
      return {
        ...state,
        elapsedSecs: event.elapsed_secs,
        status: event.visible ? "Evidence page visible" : "Evidence page hidden",
      };
    case "evidence_change":
      return addActivity(
        {
          ...state,
          elapsedSecs: event.elapsed_secs,
          lastChange: `${event.name}: ${stateMeta[event.old_state].label} to ${
            stateMeta[event.new_state].label
          }`,
        },
        "good",
        `${event.name} ${stateMeta[event.new_state].label}`,
        activityId,
      );
    case "snapshot":
      return {
        ...state,
        elapsedSecs: event.elapsed_secs,
        evidence: event.evidence,
        selectedEvidence: event.selected_evidence,
        rejectedEvidence: event.rejected_evidence,
        selectedTraitIds:
          event.reason === "game_over_reset" ? [] : state.selectedTraitIds,
        possibleGhosts: event.possible_ghosts,
        imageSize: { width: event.image_width, height: event.image_height },
        status:
          event.reason === "game_over_reset"
            ? "Ready for next round"
            : "Tracking evidence page",
      };
    case "game_over":
      return addActivity(
        {
          ...state,
          elapsedSecs: event.elapsed_secs,
          status: "Round reset detected",
          lastChange: "Evidence reset",
        },
        "info",
        `Game over: ${event.signal}`,
        activityId,
      );
    case "stopped":
      return addActivity(
        { ...state, running: false, status: "Stopped" },
        "info",
        "Tracker stopped",
        activityId,
      );
  }
}

export function mergeDefaultUnofficialTraits(
  traits: GhostTraitSnapshot[],
): GhostTraitSnapshot[] {
  const hadDeprecatedStarterTraits = traits.some((trait) =>
    deprecatedStarterTraitIds.has(trait.id),
  );
  const mergedTraits = traits.filter(
    (trait) => !deprecatedStarterTraitIds.has(trait.id),
  );
  if (mergedTraits.length > 0 && !hadDeprecatedStarterTraits) {
    return mergedTraits;
  }

  const existingTraitIds = new Set(mergedTraits.map((trait) => trait.id));

  for (const defaultTrait of defaultUnofficialTraits) {
    if (!existingTraitIds.has(defaultTrait.id)) {
      mergedTraits.push(defaultTrait);
    }
  }

  return mergedTraits;
}

export function toggleSelectedTrait(
  state: TrackerViewState,
  traitId: string,
): TrackerViewState {
  const selectedTraitIds = state.selectedTraitIds.includes(traitId)
    ? state.selectedTraitIds.filter((selectedId) => selectedId !== traitId)
    : [...state.selectedTraitIds, traitId];

  return { ...state, selectedTraitIds };
}

export function filterGhostsByTraits(
  possibleGhosts: string[],
  traits: GhostTraitSnapshot[],
  selectedTraitIds: string[],
): string[] {
  if (selectedTraitIds.length === 0) {
    return possibleGhosts;
  }

  const selectedTraits = traits.filter((trait) => selectedTraitIds.includes(trait.id));

  return possibleGhosts.filter((ghost) =>
    selectedTraits.every((trait) => {
      const possibleGhostSet = normalizedNameSet(trait.possible_ghosts);
      const excludedGhosts = normalizedNameSet(trait.excluded_ghosts);
      const ghostName = normalizeName(ghost);

      return (
        !excludedGhosts.has(ghostName) &&
        (possibleGhostSet.size === 0 || possibleGhostSet.has(ghostName))
      );
    }),
  );
}

export function relevantTraitsForGhosts(
  traits: GhostTraitSnapshot[],
  remainingGhosts: string[],
  selectedTraitIds: string[],
): GhostTraitSnapshot[] {
  if (remainingGhosts.length === 0) {
    return traits.filter((trait) => selectedTraitIds.includes(trait.id));
  }

  const remainingGhostNames = normalizedNameSet(remainingGhosts);

  return traits.filter((trait) => {
    if (selectedTraitIds.includes(trait.id)) {
      return true;
    }

    const possibleGhosts = normalizedNameSet(trait.possible_ghosts);
    const excludedGhosts = normalizedNameSet(trait.excluded_ghosts);

    if (possibleGhosts.size > 0) {
      return setsIntersect(possibleGhosts, remainingGhostNames);
    }

    return setsIntersect(excludedGhosts, remainingGhostNames);
  });
}

function normalizedNameSet(names: string[]): Set<string> {
  return new Set(names.map((name) => normalizeName(name)));
}

function normalizeName(name: string): string {
  return name.replace(/[^a-z0-9]/gi, "").toLowerCase();
}

function setsIntersect(left: Set<string>, right: Set<string>): boolean {
  for (const entry of left) {
    if (right.has(entry)) {
      return true;
    }
  }

  return false;
}

export function applyTrackerLogEntry(
  state: TrackerViewState,
  entry: TrackerLogEntry,
  activityId: number,
): TrackerViewState {
  return addActivity(
    state,
    entry.stream === "stderr" ? "warn" : "info",
    entry.line,
    activityId,
  );
}

export function applyTrackerProcessStatus(
  state: TrackerViewState,
  processStatus: TrackerProcessStatus,
  activityId: number,
): TrackerViewState {
  if (processStatus.running) {
    return { ...state, running: true, status: "Tracker running" };
  }

  if (processStatus.error) {
    return addActivity(
      { ...state, running: false, status: "Tracker failed" },
      "warn",
      processStatus.error,
      activityId,
    );
  }

  if (processStatus.code !== undefined || processStatus.signal) {
    return { ...state, running: false, status: "Tracker stopped" };
  }

  return { ...state, running: false };
}
