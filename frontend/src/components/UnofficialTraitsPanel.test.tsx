import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { UnofficialTraitsPanel } from "./UnofficialTraitsPanel";

describe("UnofficialTraitsPanel", () => {
  afterEach(() => {
    cleanup();
  });

  const traits: GhostTraitSnapshot[] = [
    {
      id: "twins_back_to_back_interactions",
      label: "Back-to-back interactions under 2s",
      description: "Two separate interactions happen with less than two seconds between them.",
      possible_ghosts: ["The Twins"],
      excluded_ghosts: [],
    },
    {
      id: "jinn_breaker_off",
      label: "Breaker manually turned off",
      description: "The ghost directly turned the fuse box off rather than overloading it.",
      possible_ghosts: [],
      excluded_ghosts: ["Jinn"],
    },
  ];

  it("renders unofficial evidence checkboxes", () => {
    render(
      <UnofficialTraitsPanel
        onToggleTrait={vi.fn()}
        selectedTraitIds={["jinn_breaker_off"]}
        traits={traits}
      />,
    );

    expect(screen.getByRole("heading", { name: "Unofficial Evidence" })).toBeTruthy();
    expect(
      screen.getByRole("checkbox", { name: /Back-to-back interactions under 2s/ }),
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: /Breaker manually turned off/ }),
    ).toBeChecked();
    expect(
      screen.queryByText(
        "The ghost directly turned the fuse box off rather than overloading it.",
      ),
    ).toBeNull();
    expect(
      screen.getByTitle(
        "The ghost directly turned the fuse box off rather than overloading it.",
      ),
    ).toHaveClass("trait-row", "selected");
    expect(screen.getByText("Clear")).toHaveClass("trait-state");
    expect(screen.getByText("Selected")).toHaveClass("trait-state");
  });

  it("notifies when a trait is toggled", () => {
    const onToggleTrait = vi.fn();
    render(
      <UnofficialTraitsPanel
        onToggleTrait={onToggleTrait}
        selectedTraitIds={[]}
        traits={traits}
      />,
    );

    fireEvent.click(
      screen.getByRole("checkbox", { name: /Back-to-back interactions under 2s/ }),
    );

    expect(onToggleTrait).toHaveBeenCalledWith("twins_back_to_back_interactions");
  });

  it("renders an empty state when no traits are relevant", () => {
    render(
      <UnofficialTraitsPanel
        onToggleTrait={vi.fn()}
        selectedTraitIds={[]}
        traits={[]}
      />,
    );

    expect(screen.getByText("No relevant traits")).toHaveClass("trait-empty");
  });
});
