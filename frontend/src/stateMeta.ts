import type { LucideIcon } from "lucide-react";
import { Check, Circle, TriangleAlert, X } from "lucide-react";

export const stateMeta: Record<
  EvidenceState,
  { label: string; Icon: LucideIcon; className: string }
> = {
  unknown: { label: "Unknown", Icon: TriangleAlert, className: "unknown" },
  clear: { label: "Clear", Icon: Circle, className: "clear" },
  selected: { label: "Selected", Icon: Check, className: "selected" },
  rejected: { label: "Rejected", Icon: X, className: "rejected" },
};
