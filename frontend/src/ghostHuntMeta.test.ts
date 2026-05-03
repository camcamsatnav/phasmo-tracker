import { describe, expect, it } from "vitest";

import { defaultGhosts } from "./trackerModel";
import {
  fallbackGhostHuntMeta,
  ghostHuntMeta,
  ghostHuntMetaByName,
} from "./ghostHuntMeta";

const expectedHuntLabels = [
  ["Banshee", "50%/90s"],
  ["Dayan", "65/45%/90s"],
  ["Demon", "70%/60s"],
  ["Deogen", "40%/90s"],
  ["Gallu", "60/40%/90s"],
  ["Goryo", "50%/90s"],
  ["Hantu", "50%/90s"],
  ["Jinn", "50%/90s"],
  ["Mare", "60/40%/90s"],
  ["Moroi", "50%/90s"],
  ["Myling", "50%/90s"],
  ["Obake", "50%/90s"],
  ["Obambo", "65/10%/90s"],
  ["Oni", "50%/90s"],
  ["Onryo", "60/40%/90s"],
  ["Phantom", "50%/90s"],
  ["Poltergeist", "50%/90s"],
  ["Raiju", "65%/90s"],
  ["Revenant", "50%/90s"],
  ["Shade", "35%/90s"],
  ["Spirit", "50%/180s"],
  ["Thaye", "75-15%/90s"],
  ["The Mimic", "Varies/90s"],
  ["The Twins", "50%/90s"],
  ["Wraith", "50%/90s"],
  ["Yokai", "80%/90s"],
  ["Yurei", "50%/90s"],
] as const;

describe("ghostHuntMeta", () => {
  it("defines hunt metadata for every default ghost", () => {
    expect(Object.keys(ghostHuntMetaByName).sort()).toEqual(
      defaultGhosts.map((ghost) => ghost.name).sort(),
    );

    for (const [name, label] of expectedHuntLabels) {
      const meta = ghostHuntMeta(name);

      expect(meta.label).toBe(label);
      expect(meta.title).toContain("Hunt threshold");
      expect(meta.title).toContain("Incense");
      expect(meta).not.toBe(fallbackGhostHuntMeta);
    }
  });

  it("falls back for unknown ghost names", () => {
    expect(ghostHuntMeta("Custom Ghost")).toBe(fallbackGhostHuntMeta);
  });
});
