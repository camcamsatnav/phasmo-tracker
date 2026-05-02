import { describe, expect, it } from "vitest";

import {
  addActivity,
  applyTrackerEvent,
  applyTrackerLogEntry,
  applyTrackerProcessStatus,
  createInitialTrackerViewState,
  groupEvidenceByState,
} from "./trackerModel";

describe("trackerModel", () => {
  it("groups evidence by state", () => {
    const groups = groupEvidenceByState([
      { name: "EMF Level 5", state: "selected" },
      { name: "Ghost Orb", state: "rejected" },
      { name: "Spirit Box", state: "clear" },
    ]);

    expect(groups.selected).toEqual(["EMF Level 5"]);
    expect(groups.rejected).toEqual(["Ghost Orb"]);
    expect(groups.clear).toEqual(["Spirit Box"]);
    expect(groups.unknown).toEqual([]);
  });

  it("applies snapshots from the tracker backend", () => {
    const state = applyTrackerEvent(
      createInitialTrackerViewState(true),
      {
        type: "snapshot",
        elapsed_secs: 12.5,
        reason: "change",
        image_width: 1920,
        image_height: 1080,
        evidence: [
          { name: "EMF Level 5", state: "selected" },
          { name: "Ghost Orb", state: "rejected" },
        ],
        selected_evidence: ["EMF Level 5"],
        rejected_evidence: ["Ghost Orb"],
        possible_ghosts: ["Spirit"],
        changes: [],
      },
      1,
    );

    expect(state.elapsedSecs).toBe(12.5);
    expect(state.imageSize).toEqual({ width: 1920, height: 1080 });
    expect(state.selectedEvidence).toEqual(["EMF Level 5"]);
    expect(state.rejectedEvidence).toEqual(["Ghost Orb"]);
    expect(state.possibleGhosts).toEqual(["Spirit"]);
    expect(state.status).toBe("Tracking evidence page");
  });

  it("records evidence changes and game-over resets", () => {
    const changed = applyTrackerEvent(
      createInitialTrackerViewState(true),
      {
        type: "evidence_change",
        elapsed_secs: 4,
        name: "Spirit Box",
        old_state: "clear",
        new_state: "selected",
      },
      7,
    );

    expect(changed.lastChange).toBe("Spirit Box: Clear to Selected");
    expect(changed.activity[0]).toMatchObject({
      id: 7,
      tone: "good",
      text: "Spirit Box Selected",
    });

    const reset = applyTrackerEvent(
      changed,
      { type: "game_over", elapsed_secs: 9, signal: "journal reset" },
      8,
    );

    expect(reset.status).toBe("Round reset detected");
    expect(reset.lastChange).toBe("Evidence reset");
    expect(reset.activity[0]).toMatchObject({
      id: 8,
      tone: "info",
      text: "Game over: journal reset",
    });
  });

  it("labels hidden page visibility as evidence page hidden", () => {
    const state = applyTrackerEvent(
      createInitialTrackerViewState(true),
      { type: "page_visibility", elapsed_secs: 3, visible: false },
      5,
    );

    expect(state.status).toBe("Evidence page hidden");
  });

  it("caps activity at the latest eight entries", () => {
    const state = Array.from({ length: 10 }, (_, index) => index).reduce(
      (current, index) => addActivity(current, "info", `Entry ${index}`, index),
      createInitialTrackerViewState(true),
    );

    expect(state.activity).toHaveLength(8);
    expect(state.activity.map((entry) => entry.text)).toEqual([
      "Entry 9",
      "Entry 8",
      "Entry 7",
      "Entry 6",
      "Entry 5",
      "Entry 4",
      "Entry 3",
      "Entry 2",
    ]);
  });

  it("applies log and process status events", () => {
    const logged = applyTrackerLogEntry(
      createInitialTrackerViewState(true),
      { stream: "stderr", line: "window not found" },
      2,
    );

    expect(logged.activity[0]).toMatchObject({
      id: 2,
      tone: "warn",
      text: "window not found",
    });

    const running = applyTrackerProcessStatus(
      logged,
      { running: true, command: "cargo run", pid: 123 },
      3,
    );
    expect(running.running).toBe(true);
    expect(running.status).toBe("Tracker running");

    const failed = applyTrackerProcessStatus(
      running,
      { running: false, error: "boom" },
      4,
    );
    expect(failed.running).toBe(false);
    expect(failed.status).toBe("Tracker failed");
    expect(failed.activity[0]).toMatchObject({ id: 4, tone: "warn", text: "boom" });
  });
});
