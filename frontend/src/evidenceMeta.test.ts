import { describe, expect, it } from "vitest";

import { evidenceMeta, evidenceMetaByName } from "./evidenceMeta";

const expectedEvidenceMeta = [
  ["EMF Level 5", "EMF", "emf"],
  ["D.O.T.S Projector", "DOTS", "dots"],
  ["Ultraviolet", "UV", "ultraviolet"],
  ["Freezing Temperatures", "FREEZE", "freezing"],
  ["Ghost Orb", "ORB", "orb"],
  ["Ghost Writing", "WRITE", "writing"],
  ["Spirit Box", "BOX", "spirit-box"],
] as const;

describe("evidenceMeta", () => {
  it("defines metadata for every supported evidence type", () => {
    expect(Object.keys(evidenceMetaByName).sort()).toEqual(
      expectedEvidenceMeta.map(([name]) => name).sort(),
    );

    for (const [name, shortLabel, className] of expectedEvidenceMeta) {
      const meta = evidenceMeta(name);

      expect(meta.shortLabel).toBe(shortLabel);
      expect(meta.className).toBe(className);
      expect(meta.Icon).toBeTruthy();
    }
  });

  it("falls back for unknown evidence names", () => {
    const meta = evidenceMeta("Custom Evidence");

    expect(meta.shortLabel).toBe("?");
    expect(meta.className).toBe("unknown");
    expect(meta.Icon).toBeTruthy();
  });
});
