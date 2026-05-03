import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { GhostsPanel } from "./GhostsPanel";

describe("GhostsPanel", () => {
  afterEach(() => {
    cleanup();
  });

  const ghostRequirements: Record<string, GhostSnapshot> = {
    Spirit: {
      name: "Spirit",
      evidence: ["EMF Level 5", "Ghost Writing", "Spirit Box"],
      false_evidence: [],
    },
    Wraith: {
      name: "Wraith",
      evidence: ["D.O.T.S Projector", "EMF Level 5", "Spirit Box"],
      false_evidence: [],
    },
    "The Mimic": {
      name: "The Mimic",
      evidence: ["Freezing Temperatures", "Spirit Box", "Ultraviolet"],
      false_evidence: ["Ghost Orb"],
    },
  };

  it("renders possible ghosts with required evidence chips and a count", () => {
    render(
      <GhostsPanel
        ghostRequirements={ghostRequirements}
        possibleGhosts={["Spirit", "Wraith", "The Mimic"]}
        selectedEvidence={["EMF Level 5", "Ghost Writing"]}
      />,
    );

    expect(screen.getByRole("heading", { name: "Possible Ghosts" })).toBeTruthy();
    expect(screen.getByText("3")).toHaveClass("count-badge");
    expect(screen.getByText("Spirit")).toHaveClass("ghost-name");
    expect(screen.getByText("Wraith")).toHaveClass("ghost-name");
    expect(screen.getByText("The Mimic")).toHaveClass("ghost-name");
    expect(screen.getAllByTitle("Required evidence: EMF Level 5")[0]).toHaveClass(
      "ghost-evidence-chip",
      "emf",
      "selected-evidence",
    );
    expect(screen.getAllByTitle("Required evidence: Spirit Box")[0]).toHaveClass(
      "spirit-box",
    );
    expect(screen.getAllByTitle("Required evidence: Spirit Box")[0]).toHaveClass(
      "missing-evidence",
    );
    expect(screen.getByTitle("False evidence: Ghost Orb")).toHaveClass("false-evidence");

    const spiritEvidence = screen.getByLabelText("Spirit evidence");
    expect(
      within(spiritEvidence)
        .getAllByTitle(/Required evidence:/)
        .map((chip) => chip.textContent),
    ).toEqual(["EMF", "WRITE", "BOX"]);
  });

  it("renders an empty state when no ghosts match", () => {
    render(
      <GhostsPanel
        ghostRequirements={ghostRequirements}
        possibleGhosts={[]}
        selectedEvidence={[]}
      />,
    );

    expect(screen.getByText("0")).toHaveClass("count-badge");
    expect(screen.getByText("No matching ghosts")).toHaveClass("empty-state");
  });

  it("handles ghosts without known evidence data", () => {
    render(
      <GhostsPanel
        ghostRequirements={{}}
        possibleGhosts={["Custom Ghost"]}
        selectedEvidence={[]}
      />,
    );

    expect(screen.getByText("Custom Ghost")).toHaveClass("ghost-name");
    expect(screen.getByText("Evidence unknown")).toHaveClass("ghost-evidence-missing");
  });
});
