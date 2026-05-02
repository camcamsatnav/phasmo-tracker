import { Activity } from "lucide-react";

import type { ActivityEntry, ImageSize } from "../trackerModel";

interface SummaryPanelProps {
  selectedEvidence: string[];
  rejectedEvidence: string[];
}

interface TelemetryPanelProps {
  clearCount: number;
  elapsedSecs: number | null;
  imageSize: ImageSize | null;
  lastChange: string;
}

interface ActivityPanelProps {
  activity: ActivityEntry[];
}

export function SummaryPanel({
  rejectedEvidence,
  selectedEvidence,
}: SummaryPanelProps) {
  return (
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
  );
}

export function TelemetryPanel({
  clearCount,
  elapsedSecs,
  imageSize,
  lastChange,
}: TelemetryPanelProps) {
  return (
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
        <strong>{clearCount}</strong>
      </div>
    </section>
  );
}

export function ActivityPanel({ activity }: ActivityPanelProps) {
  return (
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
  );
}
