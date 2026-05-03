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

const maxActivityEntries = 8;

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
