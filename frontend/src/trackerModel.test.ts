import { describe, expect, it } from "vitest";

import {
  addActivity,
  applyTrackerEvent,
  applyTrackerLogEntry,
  applyTrackerProcessStatus,
  createInitialTrackerViewState,
  filterGhostsByTraits,
  groupEvidenceByState,
  mergeDefaultUnofficialTraits,
  relevantTraitsForGhosts,
  toggleSelectedTrait,
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

  it("stores ghost evidence from tracker startup", () => {
    const state = applyTrackerEvent(
      createInitialTrackerViewState(true),
      {
        type: "tracker_started",
        config_path: "phasmo_tracker.toml",
        ghosts_path: "phasmo_ghosts.toml",
        app_name_contains: "Phasmophobia",
        window_title_contains: "Phasmophobia",
        poll_ms: 10,
        stable_frames: 1,
        evidence: ["EMF Level 5", "Ghost Orb"],
        ghosts: [
          {
            name: "The Mimic",
            evidence: ["Freezing Temperatures", "Spirit Box", "Ultraviolet"],
            false_evidence: ["Ghost Orb"],
          },
        ],
        traits: [
          {
            id: "custom_trait",
            label: "Custom trait",
            description: "User configured trait.",
            possible_ghosts: ["The Mimic"],
            excluded_ghosts: [],
          },
        ],
      },
      6,
    );

    expect(state.possibleGhosts).toEqual(["The Mimic"]);
    expect(state.ghostRequirements["The Mimic"]).toEqual({
      name: "The Mimic",
      evidence: ["Freezing Temperatures", "Spirit Box", "Ultraviolet"],
      false_evidence: ["Ghost Orb"],
    });
    expect(state.unofficialTraits[0]).toEqual({
      id: "custom_trait",
      label: "Custom trait",
      description: "User configured trait.",
      possible_ghosts: ["The Mimic"],
      excluded_ghosts: [],
    });
    expect(state.unofficialTraits).toHaveLength(1);
  });

  it("toggles unofficial traits and filters possible ghosts", () => {
    const initial = createInitialTrackerViewState(true);
    const selected = toggleSelectedTrait(initial, "jinn_breaker_off");

    expect(selected.selectedTraitIds).toEqual(["jinn_breaker_off"]);
    expect(toggleSelectedTrait(selected, "jinn_breaker_off").selectedTraitIds).toEqual(
      [],
    );
    expect(
      filterGhostsByTraits(
        ["Jinn", "Spirit", "The Twins"],
        selected.unofficialTraits,
        selected.selectedTraitIds,
      ),
    ).toEqual(["Spirit", "The Twins"]);
    expect(
      filterGhostsByTraits(
        ["Jinn", "Spirit", "The Twins"],
        selected.unofficialTraits,
        ["twins_back_to_back_interactions"],
      ),
    ).toEqual(["The Twins"]);
  });

  it("only shows traits relevant to remaining ghosts plus selected traits", () => {
    const traits: GhostTraitSnapshot[] = [
      {
        id: "wraith_salt",
        label: "Salt not disturbed",
        description: "Salt was not disturbed.",
        possible_ghosts: ["Wraith"],
        excluded_ghosts: [],
      },
      {
        id: "jinn_breaker_off",
        label: "Breaker manually turned off",
        description: "The ghost directly turned the fuse box off.",
        possible_ghosts: [],
        excluded_ghosts: ["Jinn"],
      },
      {
        id: "banshee_scream",
        label: "Banshee scream recorded",
        description: "Unique scream heard.",
        possible_ghosts: ["Banshee"],
        excluded_ghosts: [],
      },
    ];

    expect(
      relevantTraitsForGhosts(traits, ["Wraith", "Jinn"], ["banshee_scream"]).map(
        (trait) => trait.id,
      ),
    ).toEqual(["wraith_salt", "jinn_breaker_off", "banshee_scream"]);
  });

  it("falls back to bundled traits when tracker startup has none", () => {
    const state = applyTrackerEvent(
      createInitialTrackerViewState(true),
      {
        type: "tracker_started",
        config_path: "phasmo_tracker.toml",
        ghosts_path: "phasmo_ghosts.toml",
        app_name_contains: "Phasmophobia",
        window_title_contains: "Phasmophobia",
        poll_ms: 10,
        stable_frames: 1,
        evidence: ["EMF Level 5"],
        ghosts: [],
      },
      10,
    );

    expect(state.unofficialTraits.length).toBeGreaterThan(40);
    expect(state.unofficialTraits.map((trait) => trait.id)).toContain(
      "twins_back_to_back_interactions",
    );
  });

  it("removes deprecated starter traits while merging bundled defaults", () => {
    const traits = mergeDefaultUnofficialTraits([
      {
        id: "two_salts_within_two_seconds",
        label: "2 salts within 2 seconds",
        description: "Deprecated starter trait.",
        possible_ghosts: ["The Twins"],
        excluded_ghosts: [],
      },
    ]);
    const traitIds = traits.map((trait) => trait.id);

    expect(traitIds).not.toContain("two_salts_within_two_seconds");
    expect(traitIds).toContain("twins_back_to_back_interactions");
    expect(traits.length).toBeGreaterThan(40);
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
      { ...changed, selectedTraitIds: ["jinn_breaker_off"] },
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

    const resetSnapshot = applyTrackerEvent(
      reset,
      {
        type: "snapshot",
        elapsed_secs: 10,
        reason: "game_over_reset",
        image_width: 1920,
        image_height: 1080,
        evidence: [],
        selected_evidence: [],
        rejected_evidence: [],
        possible_ghosts: ["Spirit"],
        changes: [],
      },
      9,
    );

    expect(resetSnapshot.selectedTraitIds).toEqual([]);
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
