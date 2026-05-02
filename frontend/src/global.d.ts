export {};

declare global {
  interface Window {
    tracker?: TrackerBridge;
  }

  interface TrackerBridge {
    start(options?: TrackerStartOptions): Promise<TrackerProcessResult>;
    stop(): Promise<TrackerProcessResult>;
    getDefaultPaths(): Promise<TrackerPaths>;
    onEvent(callback: (event: TrackerEvent) => void): () => void;
    onLog(callback: (entry: TrackerLogEntry) => void): () => void;
    onProcess(callback: (status: TrackerProcessStatus) => void): () => void;
  }

  interface TrackerStartOptions {
    configPath?: string;
    ghostsPath?: string;
  }

  interface TrackerPaths {
    configPath: string;
    ghostsPath: string;
  }

  interface TrackerProcessResult {
    running: boolean;
    reused?: boolean;
  }

  interface TrackerProcessStatus {
    running: boolean;
    command?: string;
    pid?: number;
    code?: number | null;
    signal?: string | null;
    error?: string;
  }

  interface TrackerLogEntry {
    stream: "stdout" | "stderr";
    line: string;
  }

  type EvidenceState = "unknown" | "clear" | "selected" | "rejected";

  interface EvidenceItemSnapshot {
    name: string;
    state: EvidenceState;
  }

  interface EvidenceChangeSnapshot {
    name: string;
    old_state: EvidenceState;
    new_state: EvidenceState;
  }

  type TrackerEvent =
    | { type: "config_created"; path: string }
    | { type: "ghost_data_created"; path: string }
    | {
        type: "tracker_started";
        config_path: string;
        ghosts_path: string;
        app_name_contains: string;
        window_title_contains: string;
        poll_ms: number;
        stable_frames: number;
        evidence: string[];
        ghosts: string[];
      }
    | { type: "window_search_error"; message: string }
    | { type: "page_visibility"; elapsed_secs: number; visible: boolean }
    | {
        type: "evidence_change";
        elapsed_secs: number;
        name: string;
        old_state: EvidenceState;
        new_state: EvidenceState;
      }
    | {
        type: "snapshot";
        elapsed_secs: number;
        reason: "initial" | "change" | "game_over_reset";
        image_width: number;
        image_height: number;
        evidence: EvidenceItemSnapshot[];
        selected_evidence: string[];
        rejected_evidence: string[];
        possible_ghosts: string[];
        changes: EvidenceChangeSnapshot[];
      }
    | { type: "game_over"; elapsed_secs: number; signal: string }
    | { type: "stopped" };
}
