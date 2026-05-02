import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { AppHeader } from "./components/AppHeader";
import { EvidencePanel } from "./components/EvidencePanel";
import { GhostsPanel } from "./components/GhostsPanel";
import { ActivityPanel, SummaryPanel, TelemetryPanel } from "./components/TrackerDetails";
import {
  addActivity,
  applyTrackerEvent,
  applyTrackerLogEntry,
  applyTrackerProcessStatus,
  createInitialTrackerViewState,
  groupEvidenceByState,
} from "./trackerModel";

export function App() {
  const hasBridge = Boolean(window.tracker);
  const activitySequence = useRef(0);
  const [state, setState] = useState(() => createInitialTrackerViewState(hasBridge));

  const nextActivityId = useCallback(() => {
    activitySequence.current += 1;
    return Date.now() + activitySequence.current;
  }, []);

  const evidenceByState = useMemo(
    () => groupEvidenceByState(state.evidence),
    [state.evidence],
  );

  useEffect(() => {
    const tracker = window.tracker;
    if (!tracker) {
      setState((current) =>
        addActivity(current, "warn", "Electron bridge unavailable", nextActivityId()),
      );
      return;
    }

    const offEvent = tracker.onEvent((event) => {
      setState((current) => applyTrackerEvent(current, event, nextActivityId()));
    });
    const offLog = tracker.onLog((entry) => {
      setState((current) => applyTrackerLogEntry(current, entry, nextActivityId()));
    });
    const offProcess = tracker.onProcess((processStatus) => {
      setState((current) =>
        applyTrackerProcessStatus(current, processStatus, nextActivityId()),
      );
    });

    tracker.start().catch((error: Error) => {
      setState((current) =>
        addActivity(
          { ...current, status: "Tracker failed" },
          "warn",
          error.message,
          nextActivityId(),
        ),
      );
    });

    return () => {
      offEvent();
      offLog();
      offProcess();
    };
  }, [nextActivityId]);

  const startTracker = useCallback(() => {
    window.tracker?.start().catch((error: Error) => {
      setState((current) =>
        addActivity(
          { ...current, status: "Tracker failed" },
          "warn",
          error.message,
          nextActivityId(),
        ),
      );
    });
  }, [nextActivityId]);

  const stopTracker = useCallback(() => {
    window.tracker?.stop();
  }, []);

  return (
    <div className="app-shell">
      <AppHeader
        hasBridge={hasBridge}
        running={state.running}
        status={state.status}
        onStart={startTracker}
        onStop={stopTracker}
      />

      <main className="dashboard-grid">
        <EvidencePanel evidence={state.evidence} />
        <GhostsPanel possibleGhosts={state.possibleGhosts} />

        <aside className="side-stack" aria-label="Tracker details">
          <SummaryPanel
            rejectedEvidence={state.rejectedEvidence}
            selectedEvidence={state.selectedEvidence}
          />
          <TelemetryPanel
            clearCount={evidenceByState.clear.length}
            elapsedSecs={state.elapsedSecs}
            imageSize={state.imageSize}
            lastChange={state.lastChange}
          />
          <ActivityPanel activity={state.activity} />
        </aside>
      </main>
    </div>
  );
}
