// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/__tests__/types.test.ts
//
// 类型契约守门测试 — 验证 types.ts 导出的常量和类型守门值符合 spec.

import { describe, it, expect } from "vitest";
import {
  ALL_VIEW_MODES,
  ALL_ZOOM_LEVELS,
  ALL_INSPECTOR_TABS,
  ALL_SSE_EVENTS,
  ACTION_DANGER,
} from "../types";

describe("worktree-canvas types", () => {
  it("5 View Modes per spec §4.3", () => {
    expect(ALL_VIEW_MODES).toHaveLength(5);
    expect(ALL_VIEW_MODES).toEqual(
      expect.arrayContaining(["TREE", "DEPENDENCY", "RISK", "AGENT", "HISTORY"])
    );
  });

  it("6 Semantic Zoom Levels per spec §4.3", () => {
    expect(ALL_ZOOM_LEVELS).toHaveLength(6);
    expect(ALL_ZOOM_LEVELS).toEqual(
      expect.arrayContaining(["L0", "L1", "L2", "L3", "L4", "L5"])
    );
  });

  it("11 Inspector Tabs per spec §4.3", () => {
    expect(ALL_INSPECTOR_TABS).toHaveLength(11);
    expect(ALL_INSPECTOR_TABS).toEqual(
      expect.arrayContaining([
        "Overview",
        "GitState",
        "Task",
        "Agent",
        "Risk",
        "Tests",
        "Files",
        "Commits",
        "Relations",
        "History",
        "Actions",
      ])
    );
  });

  it("15 SSE Event Types per spec §4.5", () => {
    expect(ALL_SSE_EVENTS).toHaveLength(15);
    expect(ALL_SSE_EVENTS).toEqual(
      expect.arrayContaining([
        "WorktreeCreated",
        "WorktreeDeleted",
        "WorktreeChanged",
        "WorktreeMerged",
        "BranchUpdated",
        "MainUpdated",
        "AgentStarted",
        "AgentStopped",
        "TaskChanged",
        "TestCompleted",
        "RiskDetected",
        "RiskResolved",
        "HealthChanged",
        "RelationCreated",
        "RelationRemoved",
      ])
    );
  });

  it("18 Actions danger classification per spec §4.4", () => {
    expect(Object.keys(ACTION_DANGER)).toHaveLength(18);
    // Destructive: Merge / Delete / Cleanup (3)
    expect(ACTION_DANGER.Merge).toBe("Destructive");
    expect(ACTION_DANGER.Delete).toBe("Destructive");
    expect(ACTION_DANGER.Cleanup).toBe("Destructive");
    // Safe: Focus / ExplainRisk / Lock / Unlock etc.
    expect(ACTION_DANGER.Focus).toBe("Safe");
    expect(ACTION_DANGER.ExplainRisk).toBe("Safe");
    expect(ACTION_DANGER.Lock).toBe("Safe");
    expect(ACTION_DANGER.Unlock).toBe("Safe");
    // Warning: SyncMain / Rebase / CreatePR / Archive
    expect(ACTION_DANGER.SyncMain).toBe("Warning");
    expect(ACTION_DANGER.Rebase).toBe("Warning");
    expect(ACTION_DANGER.CreatePR).toBe("Warning");
    expect(ACTION_DANGER.Archive).toBe("Warning");
  });
});
