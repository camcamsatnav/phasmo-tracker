import { ScanLine } from "lucide-react";

import { stateMeta } from "../stateMeta";

interface EvidencePanelProps {
  evidence: EvidenceItemSnapshot[];
}

export function EvidencePanel({ evidence }: EvidencePanelProps) {
  return (
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
  );
}
