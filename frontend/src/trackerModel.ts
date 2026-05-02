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

export const defaultGhosts = [
  "Banshee",
  "Dayan",
  "Demon",
  "Deogen",
  "Gallu",
  "Goryo",
  "Hantu",
  "Jinn",
  "Mare",
  "Moroi",
  "Myling",
  "Obake",
  "Obambo",
  "Oni",
  "Onryo",
  "Phantom",
  "Poltergeist",
  "Raiju",
  "Revenant",
  "Shade",
  "Spirit",
  "Thaye",
  "The Mimic",
  "The Twins",
  "Wraith",
  "Yokai",
  "Yurei",
];

const maxActivityEntries = 8;

export function initialEvidence(): EvidenceItemSnapshot[] {
  return defaultEvidenceNames.map((name) => ({ name, state: "clear" }));
}

export function createInitialTrackerViewState(hasBridge: boolean): TrackerViewState {
  return {
    running: false,
    status: hasBridge ? "Starting tracker" : "Open in Electron",
    evidence: initialEvidence(),
    selectedEvidence: [],
    rejectedEvidence: [],
    possibleGhosts: defaultGhosts,
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
          possibleGhosts: event.ghosts,
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
