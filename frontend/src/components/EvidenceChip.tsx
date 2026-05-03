import { evidenceMeta } from "../evidenceMeta";

interface EvidenceChipProps {
  evidence: string;
  isFalse?: boolean;
  isSelected?: boolean;
}

export function EvidenceChip({
  evidence,
  isFalse = false,
  isSelected = false,
}: EvidenceChipProps) {
  const meta = evidenceMeta(evidence);
  const Icon = meta.Icon;

  return (
    <span
      className={[
        "ghost-evidence-chip",
        meta.className,
        isSelected ? "selected-evidence" : "missing-evidence",
        isFalse ? "false-evidence" : "",
      ]
        .filter(Boolean)
        .join(" ")}
      title={isFalse ? `False evidence: ${evidence}` : `Required evidence: ${evidence}`}
    >
      <Icon size={14} aria-hidden="true" />
      <span>{meta.shortLabel}</span>
    </span>
  );
}
