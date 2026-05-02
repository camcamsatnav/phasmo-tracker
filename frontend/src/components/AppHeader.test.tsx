import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AppHeader } from "./AppHeader";

describe("AppHeader", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders status and starts from both start controls", () => {
    const onStart = vi.fn();

    render(
      <AppHeader
        hasBridge
        running={false}
        status="Ready"
        onStart={onStart}
        onStop={vi.fn()}
      />,
    );

    expect(screen.getByRole("heading", { name: "Phasmo Evidence Tracker" })).toBeTruthy();
    expect(screen.getByText("Ready")).toBeTruthy();

    fireEvent.click(screen.getByTitle("Restart tracker"));
    fireEvent.click(screen.getByRole("button", { name: "Start" }));

    expect(onStart).toHaveBeenCalledTimes(2);
  });

  it("switches the primary action to stop while running", () => {
    const onStop = vi.fn();

    render(
      <AppHeader
        hasBridge
        running
        status="Tracker running"
        onStart={vi.fn()}
        onStop={onStop}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Stop" }));

    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it("disables controls when the Electron bridge is unavailable", () => {
    render(
      <AppHeader
        hasBridge={false}
        running={false}
        status="Open in Electron"
        onStart={vi.fn()}
        onStop={vi.fn()}
      />,
    );

    expect(screen.getByTitle("Restart tracker")).toBeDisabled();
    expect(screen.getByRole("button", { name: "Start" })).toBeDisabled();
  });
});
