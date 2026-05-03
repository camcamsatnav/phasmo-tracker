import { Ghost } from "lucide-react";

import { EvidenceChip } from "./EvidenceChip";

interface GhostsPanelProps {
  ghostRequirements: Record<string, GhostSnapshot>;
  possibleGhosts: string[];
  selectedEvidence: string[];
}

export function GhostsPanel({
  ghostRequirements,
  possibleGhosts,
  selectedEvidence,
}: GhostsPanelProps) {
  const selectedEvidenceNames = new Set(selectedEvidence);

  return (
    <section className="panel ghosts-panel" aria-labelledby="ghosts-heading">
      <div className="panel-heading">
        <Ghost size={20} aria-hidden="true" />
        <h2 id="ghosts-heading">Possible Ghosts</h2>
        <span className="count-badge">{possibleGhosts.length}</span>
      </div>

      {possibleGhosts.length > 0 ? (
        <div className="ghost-grid">
          {possibleGhosts.map((ghost) => {
            const requirements = ghostRequirements[ghost];
            return (
              <div className="ghost-card" key={ghost}>
                <div className="ghost-name">{ghost}</div>
                {requirements ? (
                  <div className="ghost-evidence-list" aria-label={`${ghost} evidence`}>
                    {sortRequiredEvidence(
                      requirements.evidence,
                      selectedEvidenceNames,
                    ).map((evidence) => (
                      <EvidenceChip
                        evidence={evidence}
                        isSelected={selectedEvidenceNames.has(evidence)}
                        key={evidence}
                      />
                    ))}
                    {requirements.false_evidence.map((evidence) => (
                      <EvidenceChip
                        evidence={evidence}
                        isFalse
                        isSelected={selectedEvidenceNames.has(evidence)}
                        key={`false-${evidence}`}
                      />
                    ))}
                  </div>
                ) : (
                  <div className="ghost-evidence-missing">Evidence unknown</div>
                )}
              </div>
            );
          })}
        </div>
      ) : (
        <div className="empty-state">No matching ghosts</div>
      )}
    </section>
  );
}

function sortRequiredEvidence(
  evidence: string[],
  selectedEvidenceNames: Set<string>,
): string[] {
  return [...evidence].sort((left, right) => {
    const leftSelected = selectedEvidenceNames.has(left);
    const rightSelected = selectedEvidenceNames.has(right);

    if (leftSelected === rightSelected) {
      return 0;
    }

    return leftSelected ? -1 : 1;
  });
}
