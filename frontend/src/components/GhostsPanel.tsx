import { Ghost } from "lucide-react";

interface GhostsPanelProps {
  possibleGhosts: string[];
}

export function GhostsPanel({ possibleGhosts }: GhostsPanelProps) {
  return (
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
  );
}
