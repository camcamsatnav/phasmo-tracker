import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { EvidenceChip } from "./EvidenceChip";

describe("EvidenceChip", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders missing required evidence", () => {
    render(<EvidenceChip evidence="Spirit Box" />);

    const chip = screen.getByTitle("Required evidence: Spirit Box");
    expect(chip).toHaveClass("ghost-evidence-chip", "spirit-box", "missing-evidence");
    expect(chip).not.toHaveClass("selected-evidence", "false-evidence");
    expect(chip).toHaveTextContent("BOX");
  });

  it("renders selected required evidence", () => {
    render(<EvidenceChip evidence="Ghost Writing" isSelected />);

    const chip = screen.getByTitle("Required evidence: Ghost Writing");
    expect(chip).toHaveClass("ghost-evidence-chip", "writing", "selected-evidence");
    expect(chip).not.toHaveClass("missing-evidence", "false-evidence");
    expect(chip).toHaveTextContent("WRITE");
  });

  it("renders missing false evidence", () => {
    render(<EvidenceChip evidence="Ghost Orb" isFalse />);

    const chip = screen.getByTitle("False evidence: Ghost Orb");
    expect(chip).toHaveClass("ghost-evidence-chip", "orb", "missing-evidence");
    expect(chip).toHaveClass("false-evidence");
    expect(chip).not.toHaveClass("selected-evidence");
    expect(chip).toHaveTextContent("ORB");
  });

  it("renders selected false evidence", () => {
    render(<EvidenceChip evidence="Ghost Orb" isFalse isSelected />);

    const chip = screen.getByTitle("False evidence: Ghost Orb");
    expect(chip).toHaveClass(
      "ghost-evidence-chip",
      "orb",
      "selected-evidence",
      "false-evidence",
    );
    expect(chip).not.toHaveClass("missing-evidence");
    expect(chip).toHaveTextContent("ORB");
  });

  it("renders unknown evidence with fallback metadata", () => {
    render(<EvidenceChip evidence="Custom Evidence" />);

    const chip = screen.getByTitle("Required evidence: Custom Evidence");
    expect(chip).toHaveClass("ghost-evidence-chip", "unknown", "missing-evidence");
    expect(chip).toHaveTextContent("?");
  });
});
