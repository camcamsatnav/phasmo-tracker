import { Activity, Play, RotateCw, Square } from "lucide-react";

interface AppHeaderProps {
  hasBridge: boolean;
  running: boolean;
  status: string;
  onStart: () => void;
  onStop: () => void;
}

export function AppHeader({
  hasBridge,
  running,
  status,
  onStart,
  onStop,
}: AppHeaderProps) {
  return (
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
          onClick={onStart}
          disabled={!hasBridge}
        >
          <RotateCw size={18} aria-hidden="true" />
        </button>
        <button
          type="button"
          className="primary-button"
          title={running ? "Stop tracker" : "Start tracker"}
          onClick={running ? onStop : onStart}
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
  );
}
