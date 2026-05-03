export interface GhostHuntMeta {
  label: string;
  title: string;
}

const standardTitle = "Hunt threshold: 50%; Incense hunt prevention: 90s.";
const standardMeta: GhostHuntMeta = {
  label: "50%/90s",
  title: standardTitle,
};

export const fallbackGhostHuntMeta: GhostHuntMeta = {
  label: "?%/?s",
  title: "Hunt threshold and incense timing unknown.",
};

export const ghostHuntMetaByName: Record<string, GhostHuntMeta> = {
  Banshee: {
    label: "50%/90s",
    title:
      "Hunt threshold: 50% target sanity instead of average sanity; Incense hunt prevention: 90s.",
  },
  Dayan: {
    label: "65/45%/90s",
    title:
      "Hunt threshold: 65% when a nearby player is moving, 45% when still; Incense hunt prevention: 90s.",
  },
  Demon: {
    label: "70%/60s",
    title:
      "Hunt threshold: 70%, with an ability that can hunt regardless of sanity; Incense hunt prevention: 60s.",
  },
  Deogen: {
    label: "40%/90s",
    title: "Hunt threshold: 40%; Incense hunt prevention: 90s.",
  },
  Gallu: {
    label: "60/40%/90s",
    title:
      "Hunt threshold: 60% when enraged, 40% when weakened; Incense hunt prevention: 90s.",
  },
  Goryo: standardMeta,
  Hantu: standardMeta,
  Jinn: standardMeta,
  Mare: {
    label: "60/40%/90s",
    title:
      "Hunt threshold: 60% with lights off or broken in its room, 40% with lights on; Incense hunt prevention: 90s.",
  },
  Moroi: standardMeta,
  Myling: standardMeta,
  Obake: standardMeta,
  Obambo: {
    label: "65/10%/90s",
    title:
      "Hunt threshold: 65% when aggressive, 10% when calm; Incense hunt prevention: 90s.",
  },
  Oni: standardMeta,
  Onryo: {
    label: "60/40%/90s",
    title:
      "Hunt threshold: 60%, or 40% near a flame; flame ability can hunt regardless of sanity; Incense hunt prevention: 90s.",
  },
  Phantom: standardMeta,
  Poltergeist: standardMeta,
  Raiju: {
    label: "65%/90s",
    title:
      "Hunt threshold: 65% near active electronic equipment; Incense hunt prevention: 90s.",
  },
  Revenant: standardMeta,
  Shade: {
    label: "35%/90s",
    title:
      "Hunt threshold: 35%, but it cannot hunt with a player in its current room; Incense hunt prevention: 90s.",
  },
  Spirit: {
    label: "50%/180s",
    title: "Hunt threshold: 50%; Incense hunt prevention: 180s.",
  },
  Thaye: {
    label: "75-15%/90s",
    title:
      "Hunt threshold: 75% at age 0, decreasing to 15% by age 10; Incense hunt prevention: 90s.",
  },
  "The Mimic": {
    label: "Varies/90s",
    title:
      "Hunt threshold and abilities inherit from the mimicked ghost; Incense hunt prevention: 90s.",
  },
  "The Twins": standardMeta,
  Wraith: standardMeta,
  Yokai: {
    label: "80%/90s",
    title:
      "Hunt threshold: 80% when a player uses voice chat nearby; Incense hunt prevention: 90s.",
  },
  Yurei: standardMeta,
};

export function ghostHuntMeta(name: string): GhostHuntMeta {
  return ghostHuntMetaByName[name] ?? fallbackGhostHuntMeta;
}
