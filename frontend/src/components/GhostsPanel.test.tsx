import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { GhostsPanel } from "./GhostsPanel";

describe("GhostsPanel", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders possible ghost chips and a count", () => {
    render(<GhostsPanel possibleGhosts={["Spirit", "Wraith", "The Mimic"]} />);

    expect(screen.getByRole("heading", { name: "Possible Ghosts" })).toBeTruthy();
    expect(screen.getByText("3")).toHaveClass("count-badge");
    expect(screen.getByText("Spirit")).toHaveClass("ghost-chip");
    expect(screen.getByText("Wraith")).toHaveClass("ghost-chip");
    expect(screen.getByText("The Mimic")).toHaveClass("ghost-chip");
  });

  it("renders an empty state when no ghosts match", () => {
    render(<GhostsPanel possibleGhosts={[]} />);

    expect(screen.getByText("0")).toHaveClass("count-badge");
    expect(screen.getByText("No matching ghosts")).toHaveClass("empty-state");
  });
});
