import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { ActivityPanel, SummaryPanel, TelemetryPanel } from "./TrackerDetails";

describe("SummaryPanel", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders selected and rejected evidence counts and names", () => {
    render(
      <SummaryPanel
        selectedEvidence={["Spirit Box", "Ghost Writing"]}
        rejectedEvidence={["Ghost Orb"]}
      />,
    );

    expect(screen.getByText("Selected")).toBeTruthy();
    expect(screen.getByText("Spirit Box, Ghost Writing")).toHaveClass("selected");
    expect(screen.getByText("Rejected")).toBeTruthy();
    expect(screen.getByText("Ghost Orb")).toHaveClass("rejected");
  });

  it("renders none when evidence lists are empty", () => {
    render(<SummaryPanel selectedEvidence={[]} rejectedEvidence={[]} />);

    expect(screen.getAllByText("None")).toHaveLength(2);
  });
});

describe("TelemetryPanel", () => {
  afterEach(() => {
    cleanup();
  });

  it("formats elapsed time, capture size, last change, and clear count", () => {
    render(
      <TelemetryPanel
        clearCount={5}
        elapsedSecs={12.345}
        imageSize={{ width: 1920, height: 1080 }}
        lastChange="Spirit Box: Clear to Selected"
      />,
    );

    expect(screen.getByText("12.35s")).toBeTruthy();
    expect(screen.getByText("1920x1080")).toBeTruthy();
    expect(screen.getByText("Spirit Box: Clear to Selected")).toBeTruthy();
    expect(screen.getByText("5")).toBeTruthy();
  });

  it("uses placeholders before telemetry exists", () => {
    render(
      <TelemetryPanel
        clearCount={0}
        elapsedSecs={null}
        imageSize={null}
        lastChange="None yet"
      />,
    );

    expect(screen.getAllByText("--")).toHaveLength(2);
    expect(screen.getByText("None yet")).toBeTruthy();
  });
});

describe("ActivityPanel", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders activity entries with tone classes", () => {
    render(
      <ActivityPanel
        activity={[
          { id: 1, tone: "good", text: "Spirit Box Selected" },
          { id: 2, tone: "warn", text: "Window missing" },
        ]}
      />,
    );

    expect(screen.getByRole("heading", { name: "Activity" })).toBeTruthy();
    expect(screen.getByText("Spirit Box Selected")).toHaveClass("good");
    expect(screen.getByText("Window missing")).toHaveClass("warn");
  });

  it("renders a waiting row when activity is empty", () => {
    render(<ActivityPanel activity={[]} />);

    expect(screen.getByText("Waiting for events")).toHaveClass("info");
  });
});
