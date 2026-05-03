import { ClipboardCheck } from "lucide-react";

interface UnofficialTraitsPanelProps {
  traits: GhostTraitSnapshot[];
  selectedTraitIds: string[];
  onToggleTrait: (traitId: string) => void;
}

export function UnofficialTraitsPanel({
  traits,
  selectedTraitIds,
  onToggleTrait,
}: UnofficialTraitsPanelProps) {
  const selectedTraits = new Set(selectedTraitIds);

  return (
    <section
      className="panel unofficial-traits-panel"
      aria-labelledby="unofficial-traits-heading"
    >
      <div className="panel-heading compact">
        <ClipboardCheck size={20} aria-hidden="true" />
        <h2 id="unofficial-traits-heading">Unofficial Evidence</h2>
      </div>

      <div className="trait-list">
        {traits.length > 0 ? (
          traits.map((trait) => (
            <label
              className={`trait-row ${
                selectedTraits.has(trait.id) ? "selected" : "clear"
              }`}
              key={trait.id}
              title={trait.description}
            >
              <span className="trait-icon">
                <input
                  checked={selectedTraits.has(trait.id)}
                  onChange={() => onToggleTrait(trait.id)}
                  type="checkbox"
                />
              </span>
              <span className="trait-copy">
                <span className="trait-label">{trait.label}</span>
              </span>
              <span className="trait-state">
                {selectedTraits.has(trait.id) ? "Selected" : "Clear"}
              </span>
            </label>
          ))
        ) : (
          <div className="trait-empty">No relevant traits</div>
        )}
      </div>
    </section>
  );
}
