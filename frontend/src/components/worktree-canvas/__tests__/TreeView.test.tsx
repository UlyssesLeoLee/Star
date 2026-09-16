import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { TreeView } from "../TreeView";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";
import type { WorktreeNodeData } from "../types";

const wt: WorktreeNodeData = {
  id: "wt-tree-1",
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
  name: "tree-test-wt",
  riskCount: 0,
};

describe("TreeView", () => {
  beforeEach(() => {
    useWorktreeCanvasStore.setState({ worktrees: [wt], selectedWorktreeId: null });
  });

  it("renders empty state when no worktrees", () => {
    useWorktreeCanvasStore.setState({ worktrees: [] });
    render(<TreeView />);
    expect(screen.getByText(/暂无 Worktree 数据/)).toBeInTheDocument();
  });

  it("groups by repo and renders tree structure", () => {
    render(<TreeView />);
    expect(screen.getByTestId("tree-view-wt-wt-tree-1")).toBeInTheDocument();
    expect(screen.getByText(/RUNNING/)).toBeInTheDocument();
  });

  it("marks selected worktree", () => {
    useWorktreeCanvasStore.setState({ selectedWorktreeId: "wt-tree-1" });
    render(<TreeView />);
    const el = screen.getByTestId("tree-view-wt-wt-tree-1");
    expect(el.getAttribute("data-selected")).toBe("true");
  });
});
