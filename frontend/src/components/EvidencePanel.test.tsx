import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { EvidencePanel } from "./EvidencePanel";

describe("EvidencePanel", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders each evidence item with its state label and class", () => {
    render(
      <EvidencePanel
        evidence={[
          { name: "EMF Level 5", state: "selected" },
          { name: "Ghost Orb", state: "rejected" },
          { name: "Spirit Box", state: "clear" },
          { name: "Ultraviolet", state: "unknown" },
        ]}
      />,
    );

    expect(screen.getByRole("heading", { name: "Evidence" })).toBeTruthy();
    expect(screen.getByTitle("EMF Level 5: Selected")).toHaveClass("selected");
    expect(screen.getByTitle("Ghost Orb: Rejected")).toHaveClass("rejected");
    expect(screen.getByTitle("Spirit Box: Clear")).toHaveClass("clear");
    expect(screen.getByTitle("Ultraviolet: Unknown")).toHaveClass("unknown");
  });
});
