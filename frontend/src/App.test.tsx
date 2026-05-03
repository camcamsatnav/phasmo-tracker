import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";

describe("App", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    delete window.tracker;
  });

  it("shows a bridge warning outside Electron", async () => {
    render(<App />);

    expect(screen.getByText("Open in Electron")).toBeTruthy();
    expect(await screen.findByText("Electron bridge unavailable")).toBeTruthy();
  });

  it("starts the tracker bridge and renders snapshot events", async () => {
    const eventHandlers: Array<(event: TrackerEvent) => void> = [];
    window.tracker = {
      start: vi.fn().mockResolvedValue({ running: true }),
      stop: vi.fn().mockResolvedValue({ running: false }),
      getDefaultPaths: vi.fn().mockResolvedValue({
        configPath: "phasmo_tracker.toml",
        ghostsPath: "phasmo_ghosts.toml",
      }),
      onEvent: (callback) => {
        eventHandlers.push(callback);
        return vi.fn();
      },
      onLog: () => vi.fn(),
      onProcess: () => vi.fn(),
    };

    render(<App />);

    await waitFor(() => expect(window.tracker?.start).toHaveBeenCalledTimes(1));

    act(() => {
      eventHandlers[0]({
        type: "snapshot",
        elapsed_secs: 2.25,
        reason: "game_over_reset",
        image_width: 1280,
        image_height: 720,
        evidence: [
          { name: "EMF Level 5", state: "clear" },
          { name: "Spirit Box", state: "selected" },
        ],
        selected_evidence: ["Spirit Box"],
        rejected_evidence: [],
        possible_ghosts: ["Spirit", "Wraith"],
        changes: [],
      });
    });

    expect(screen.getByText("Ready for next round")).toBeTruthy();
    expect(screen.getAllByText("Spirit Box").length).toBeGreaterThan(0);
    expect(screen.getByText("Spirit")).toBeTruthy();
    expect(screen.getByText("Wraith")).toBeTruthy();
    expect(screen.getByText("1280x720")).toBeTruthy();
    expect(
      screen.queryByRole("checkbox", { name: /Banshee scream recorded/ }),
    ).toBeNull();
    expect(
      screen.getByRole("checkbox", { name: /Salt not disturbed when crossed/ }),
    ).toBeTruthy();

    fireEvent.click(
      screen.getByRole("checkbox", { name: /Salt not disturbed when crossed/ }),
    );

    expect(screen.queryByText("Spirit")).toBeNull();
    expect(screen.getByText("Wraith")).toBeTruthy();
  });
});
