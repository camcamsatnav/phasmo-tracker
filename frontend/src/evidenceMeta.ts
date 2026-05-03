import type { LucideIcon } from "lucide-react";
import {
  BookOpenText,
  CircleDot,
  Fingerprint,
  Radio,
  CircleDotDashed,
  Snowflake,
  Zap,
} from "lucide-react";

interface EvidenceMeta {
  Icon: LucideIcon;
  shortLabel: string;
  className: string;
}

const fallbackMeta: EvidenceMeta = {
  Icon: CircleDot,
  shortLabel: "?",
  className: "unknown",
};

export const evidenceMetaByName: Record<string, EvidenceMeta> = {
  "EMF Level 5": {
    Icon: Zap,
    shortLabel: "EMF",
    className: "emf",
  },
  "D.O.T.S Projector": {
    Icon: CircleDotDashed,
    shortLabel: "DOTS",
    className: "dots",
  },
  Ultraviolet: {
    Icon: Fingerprint,
    shortLabel: "UV",
    className: "ultraviolet",
  },
  "Freezing Temperatures": {
    Icon: Snowflake,
    shortLabel: "FREEZE",
    className: "freezing",
  },
  "Ghost Orb": {
    Icon: CircleDot,
    shortLabel: "ORB",
    className: "orb",
  },
  "Ghost Writing": {
    Icon: BookOpenText,
    shortLabel: "WRITE",
    className: "writing",
  },
  "Spirit Box": {
    Icon: Radio,
    shortLabel: "BOX",
    className: "spirit-box",
  },
};

export function evidenceMeta(name: string): EvidenceMeta {
  return evidenceMetaByName[name] ?? fallbackMeta;
}
