import { useEffect, useMemo, useState } from "react";
import type { LucideIcon } from "lucide-react";
import {
  Activity,
  Check,
  Circle,
  Ghost,
  Play,
  RotateCw,
  ScanLine,
  Square,
  TriangleAlert,
  X,
} from "lucide-react";

const defaultEvidenceNames = [
  "EMF Level 5",
  "D.O.T.S Projector",
  "Ultraviolet",
  "Freezing Temperatures",
  "Ghost Orb",
  "Ghost Writing",
  "Spirit Box",
];

const defaultGhosts = [
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

const stateMeta: Record<
  EvidenceState,
  { label: string; Icon: LucideIcon; className: string }
> = {
  unknown: { label: "Unknown", Icon: TriangleAlert, className: "unknown" },
  clear: { label: "Clear", Icon: Circle, className: "clear" },
  selected: { label: "Selected", Icon: Check, className: "selected" },
  rejected: { label: "Rejected", Icon: X, className: "rejected" },
};

interface ActivityEntry {
  id: number;
  tone: "info" | "good" | "warn";
  text: string;
}

function initialEvidence(): EvidenceItemSnapshot[] {
  return defaultEvidenceNames.map((name) => ({ name, state: "clear" }));
}

export function App() {
  const hasBridge = Boolean(window.tracker);
  const [running, setRunning] = useState(false);
  const [status, setStatus] = useState(
    hasBridge ? "Starting tracker" : "Open in Electron",
  );
  const [evidence, setEvidence] = useState<EvidenceItemSnapshot[]>(initialEvidence);
  const [selectedEvidence, setSelectedEvidence] = useState<string[]>([]);
  const [rejectedEvidence, setRejectedEvidence] = useState<string[]>([]);
  const [possibleGhosts, setPossibleGhosts] = useState<string[]>(defaultGhosts);
  const [activity, setActivity] = useState<ActivityEntry[]>([]);
  const [elapsedSecs, setElapsedSecs] = useState<number | null>(null);
  const [imageSize, setImageSize] = useState<{ width: number; height: number } | null>(
    null,
  );
  const [lastChange, setLastChange] = useState<string>("None yet");

  const evidenceByState = useMemo(() => {
    return evidence.reduce(
      (groups, item) => {
        groups[item.state].push(item.name);
        return groups;
      },
      {
        unknown: [] as string[],
        clear: [] as string[],
        selected: [] as string[],
        rejected: [] as string[],
      },
    );
  }, [evidence]);

  useEffect(() => {
    if (!window.tracker) {
      setActivity([
        {
          id: Date.now(),
          tone: "warn",
          text: "Electron bridge unavailable",
        },
      ]);
      return;
    }

    const addActivity = (tone: ActivityEntry["tone"], text: string) => {
      setActivity((entries) =>
        [{ id: Date.now() + Math.random(), tone, text }, ...entries].slice(0, 8),
      );
    };

    const handleEvent = (event: TrackerEvent) => {
      switch (event.type) {
        case "tracker_started":
          setEvidence(event.evidence.map((name) => ({ name, state: "clear" })));
          setPossibleGhosts(event.ghosts);
          setStatus("Looking for Phasmophobia");
          addActivity("info", "Tracker started");
          break;
        case "config_created":
          addActivity("info", `Created ${event.path}`);
          break;
        case "ghost_data_created":
          addActivity("info", `Created ${event.path}`);
          break;
        case "window_search_error":
          setStatus("Waiting for Phasmophobia window");
          addActivity("warn", event.message);
          break;
        case "page_visibility":
          setElapsedSecs(event.elapsed_secs);
          setStatus(event.visible ? "Evidence page visible" : "Journal page hidden");
          break;
        case "evidence_change":
          setElapsedSecs(event.elapsed_secs);
          setLastChange(
            `${event.name}: ${stateMeta[event.old_state].label} to ${
              stateMeta[event.new_state].label
            }`,
          );
          addActivity("good", `${event.name} ${stateMeta[event.new_state].label}`);
          break;
        case "snapshot":
          setElapsedSecs(event.elapsed_secs);
          setEvidence(event.evidence);
          setSelectedEvidence(event.selected_evidence);
          setRejectedEvidence(event.rejected_evidence);
          setPossibleGhosts(event.possible_ghosts);
          setImageSize({ width: event.image_width, height: event.image_height });
          setStatus(
            event.reason === "game_over_reset"
              ? "Ready for next round"
              : "Tracking evidence page",
          );
          break;
        case "game_over":
          setElapsedSecs(event.elapsed_secs);
          setStatus("Round reset detected");
          setLastChange("Evidence reset");
          addActivity("info", `Game over: ${event.signal}`);
          break;
        case "stopped":
          setRunning(false);
          setStatus("Stopped");
          addActivity("info", "Tracker stopped");
          break;
      }
    };

    const offEvent = window.tracker.onEvent(handleEvent);
    const offLog = window.tracker.onLog((entry) => {
      addActivity(entry.stream === "stderr" ? "warn" : "info", entry.line);
    });
    const offProcess = window.tracker.onProcess((processStatus) => {
      setRunning(processStatus.running);
      if (processStatus.running) {
        setStatus("Tracker running");
      } else if (processStatus.error) {
        setStatus("Tracker failed");
        addActivity("warn", processStatus.error);
      } else if (processStatus.code !== undefined || processStatus.signal) {
        setStatus("Tracker stopped");
      }
    });

    window.tracker.start().catch((error: Error) => {
      setStatus("Tracker failed");
      addActivity("warn", error.message);
    });

    return () => {
      offEvent();
      offLog();
      offProcess();
    };
  }, []);

  const startTracker = () => {
    window.tracker?.start().catch((error: Error) => {
      setStatus("Tracker failed");
      setActivity((entries) =>
        [{ id: Date.now(), tone: "warn" as const, text: error.message }, ...entries].slice(
          0,
          8,
        ),
      );
    });
  };

  const stopTracker = () => {
    window.tracker?.stop();
  };

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="title-block">
          <span className="eyebrow">External visual tracker</span>
          <h1>Phasmo Evidence Tracker</h1>
        </div>

        <div className={`status-pill ${running ? "active" : "idle"}`}>
          <Activity size={18} aria-hidden="true" />
          <span>{status}</span>
        </div>

        <div className="header-actions">
          <button
            type="button"
            className="icon-button"
            title="Restart tracker"
            onClick={startTracker}
            disabled={!hasBridge}
          >
            <RotateCw size={18} aria-hidden="true" />
          </button>
          <button
            type="button"
            className="primary-button"
            title={running ? "Stop tracker" : "Start tracker"}
            onClick={running ? stopTracker : startTracker}
            disabled={!hasBridge}
          >
            {running ? (
              <Square size={18} aria-hidden="true" />
            ) : (
              <Play size={18} aria-hidden="true" />
            )}
            <span>{running ? "Stop" : "Start"}</span>
          </button>
        </div>
      </header>

      <main className="dashboard-grid">
        <section className="panel evidence-panel" aria-labelledby="evidence-heading">
          <div className="panel-heading">
            <ScanLine size={20} aria-hidden="true" />
            <h2 id="evidence-heading">Evidence</h2>
          </div>

          <div className="evidence-list">
            {evidence.map((item) => {
              const meta = stateMeta[item.state];
              const Icon = meta.Icon;
              return (
                <div
                  className={`evidence-row ${meta.className}`}
                  key={item.name}
                  title={`${item.name}: ${meta.label}`}
                >
                  <span className="evidence-icon">
                    <Icon size={18} aria-hidden="true" />
                  </span>
                  <span className="evidence-name">{item.name}</span>
                  <span className="evidence-state">{meta.label}</span>
                </div>
              );
            })}
          </div>
        </section>

        <section className="panel ghosts-panel" aria-labelledby="ghosts-heading">
          <div className="panel-heading">
            <Ghost size={20} aria-hidden="true" />
            <h2 id="ghosts-heading">Possible Ghosts</h2>
            <span className="count-badge">{possibleGhosts.length}</span>
          </div>

          {possibleGhosts.length > 0 ? (
            <div className="ghost-grid">
              {possibleGhosts.map((ghost) => (
                <div className="ghost-chip" key={ghost}>
                  {ghost}
                </div>
              ))}
            </div>
          ) : (
            <div className="empty-state">No matching ghosts</div>
          )}
        </section>

        <aside className="side-stack" aria-label="Tracker details">
          <section className="panel summary-panel">
            <div className="summary-row">
              <span>Selected</span>
              <strong>{selectedEvidence.length}</strong>
            </div>
            <div className="name-strip selected">
              {selectedEvidence.length > 0 ? selectedEvidence.join(", ") : "None"}
            </div>

            <div className="summary-row">
              <span>Rejected</span>
              <strong>{rejectedEvidence.length}</strong>
            </div>
            <div className="name-strip rejected">
              {rejectedEvidence.length > 0 ? rejectedEvidence.join(", ") : "None"}
            </div>
          </section>

          <section className="panel telemetry-panel">
            <div className="metric-row">
              <span>Elapsed</span>
              <strong>{elapsedSecs === null ? "--" : `${elapsedSecs.toFixed(2)}s`}</strong>
            </div>
            <div className="metric-row">
              <span>Capture</span>
              <strong>{imageSize ? `${imageSize.width}x${imageSize.height}` : "--"}</strong>
            </div>
            <div className="metric-row">
              <span>Last change</span>
              <strong>{lastChange}</strong>
            </div>
            <div className="metric-row">
              <span>Clear</span>
              <strong>{evidenceByState.clear.length}</strong>
            </div>
          </section>

          <section className="panel activity-panel">
            <div className="panel-heading compact">
              <Activity size={18} aria-hidden="true" />
              <h2>Activity</h2>
            </div>
            <ol className="activity-list">
              {activity.length > 0 ? (
                activity.map((entry) => (
                  <li className={entry.tone} key={entry.id}>
                    {entry.text}
                  </li>
                ))
              ) : (
                <li className="info">Waiting for events</li>
              )}
            </ol>
          </section>
        </aside>
      </main>
    </div>
  );
}
