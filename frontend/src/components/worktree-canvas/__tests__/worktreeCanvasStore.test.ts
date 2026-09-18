// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/__tests__/worktreeCanvasStore.test.ts
//
// Zustand store 守门测试 (per IMPL-PLAN §4.14.6).

import { describe, it, expect, beforeEach } from "vitest";
import { useWorktreeCanvasStore, MAX_RECENT_EVENTS } from "@/stores/worktreeCanvasStore";
import type { WorktreeNodeData } from "../types";

const mockWt: WorktreeNodeData = {
  id: "wt-1",
  repositoryId: "repo-1",
  branch: "feature/test",
  humanState: "RUNNING",
  machineState: "git_clean",
  ahead: 0,
  behind: 0,
  dirtyState: "CLEAN",
  testState: "PASSED",
  healthScore: 100,
  lastActivityAt: "2026-09-17T00:00:00Z",
  name: "test-wt",
  riskCount: 0,
};

describe("worktreeCanvasStore", () => {
  beforeEach(() => {
    useWorktreeCanvasStore.getState().clearEvents();
    useWorktreeCanvasStore.setState({ worktrees: [], edges: [] });
  });

  it("initial state has empty worktrees/edges and default UI", () => {
    const s = useWorktreeCanvasStore.getState();
    expect(s.worktrees).toEqual([]);
    expect(s.edges).toEqual([]);
    expect(s.viewMode).toBe("TREE");
    expect(s.zoomLevel).toBe("L1");
    expect(s.inspectorTab).toBe("Overview");
    expect(s.focusNodeId).toBeNull();
  });

  it("upsertWorktree adds new and updates existing", () => {
    const { upsertWorktree, worktrees } = useWorktreeCanvasStore.getState();
    upsertWorktree(mockWt);
    expect(useWorktreeCanvasStore.getState().worktrees).toHaveLength(1);

    const updated = { ...mockWt, ahead: 5 };
    upsertWorktree(updated);
    const list = useWorktreeCanvasStore.getState().worktrees;
    expect(list).toHaveLength(1);
    expect(list[0].ahead).toBe(5);
    // Reference cleanup
    void worktrees;
  });

  it("removeWorktree filters by id", () => {
    const { upsertWorktree, removeWorktree } = useWorktreeCanvasStore.getState();
    upsertWorktree(mockWt);
    upsertWorktree({ ...mockWt, id: "wt-2" });
    expect(useWorktreeCanvasStore.getState().worktrees).toHaveLength(2);
    removeWorktree("wt-1");
    expect(useWorktreeCanvasStore.getState().worktrees).toHaveLength(1);
    expect(useWorktreeCanvasStore.getState().worktrees[0].id).toBe("wt-2");
  });

  it("pushEvent stores with FIFO order and cap", () => {
    const { pushEvent } = useWorktreeCanvasStore.getState();
    for (let i = 0; i < MAX_RECENT_EVENTS + 5; i++) {
      pushEvent("WorktreeCreated", { i });
    }
    const evs = useWorktreeCanvasStore.getState().recentEvents;
    expect(evs.length).toBe(MAX_RECENT_EVENTS);
    // Most recent (i = MAX+4) should be first.
    expect(evs[0].payload).toEqual({ i: MAX_RECENT_EVENTS + 4 });
  });

  it("setFocus records nodeId + hop", () => {
    const { setFocus } = useWorktreeCanvasStore.getState();
    setFocus("wt-1", 2);
    expect(useWorktreeCanvasStore.getState().focusNodeId).toBe("wt-1");
    expect(useWorktreeCanvasStore.getState().focusHop).toBe(2);
  });

  it("UI preferences setters work independently", () => {
    const { setViewMode, setZoomLevel, setInspectorTab, setSearchQuery, setFocusMode } =
      useWorktreeCanvasStore.getState();
    setViewMode("RISK");
    setZoomLevel("L3");
    setInspectorTab("Tests");
    setSearchQuery("show:conflict");
    setFocusMode(true);
    const s = useWorktreeCanvasStore.getState();
    expect(s.viewMode).toBe("RISK");
    expect(s.zoomLevel).toBe("L3");
    expect(s.inspectorTab).toBe("Tests");
    expect(s.searchQuery).toBe("show:conflict");
    expect(s.focusMode).toBe(true);
  });
});
