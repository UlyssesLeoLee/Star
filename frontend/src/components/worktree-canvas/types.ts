// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/types.ts
//
// AI Worktree Graph Canvas — 共享 TS 类型 (per DD-WORKTREE-CANVAS-001 §7-§8 +
// WORKTREE-CANVAS-IMPL-PLAN-001 §4.14 + spec §4.3).
//
// 守门合规 (per 守门 #19 v19): 0 改 V0.1 types/ids.ts, 新增独立 types 文件.
// Mirror graph-core::state + git-adapter + spec §4.4 Action 矩阵.
//
// 5 类型:
// - WorktreeHumanState (7 态 per SRS §九)
// - WorktreeNodeData / EdgeData (per spec §4.3)
// - ViewMode / ZoomLevel (5+6 per spec §4.3)
// - InspectorTab (11 per spec §4.3)
// - CanvasRenderer (5 方法 interface)

export type WorktreeHumanState =
  | "RUNNING"
  | "WAITING"
  | "READY"
  | "DIVERGED"
  | "CONFLICT"
  | "MERGED"
  | "STALE";

export type WorktreeMachineState = string; // 12 态 per graph-core

export type RiskType =
  | "MERGE_CONFLICT"
  | "FILE_OVERLAP"
  | "SYMBOL_OVERLAP"
  | "STALE"
  | "DIVERGENCE"
  | "DEPENDENCY"
  | "TEST_FAILURE"
  | "BUILD_FAILURE"
  | "AGENT_INCOMPLETE"
  | "REVIEW_MISSING"
  | "SUPERSEDED";

export type DirtyState = "CLEAN" | "MODIFIED" | "STAGED" | "MIXED";

export type TestState = "PASSED" | "FAILED" | "RUNNING" | "SKIPPED" | "NOT_RUN";

export interface AgentSummary {
  id: string;
  agentType: string;
  model: string;
}

export interface TaskSummary {
  id: string;
  title: string;
}

export interface HealthDeduction {
  factor: string;
  points: number;
  reason: string;
}

export interface HealthScore {
  value: number; // 0-100
  deductions: HealthDeduction[];
  computedAt: string;
}

export interface WorktreeNodeData {
  id: string;
  repositoryId: string;
  branch: string;
  humanState: WorktreeHumanState;
  machineState: WorktreeMachineState;
  ahead: number;
  behind: number;
  dirtyState: DirtyState;
  testState: TestState;
  agentSessionId?: string;
  taskId?: string;
  healthScore: number;
  lastActivityAt: string;
  // 扩展字段 (per DD §10):
  name?: string;
  riskCount?: number;
  locked?: boolean;
  archived?: boolean;
  mergedAt?: string | null;
}

export type EdgeKind =
  | "BASED_ON"
  | "USES_BRANCH"
  | "DERIVED_FROM"
  | "WORKS_ON"
  | "IMPLEMENTED_IN"
  | "MODIFIES"
  | "MODIFIES_SYMBOL"
  | "CONFLICTS_WITH"
  | "OVERLAPS_WITH"
  | "DEPENDS_ON"
  | "BLOCKS"
  | "SUPERSEDES"
  | "MERGED_INTO"
  | "VALIDATES"
  | "REVIEWS";

export interface EdgeData {
  kind: EdgeKind;
  source: string;
  target: string;
  riskScore?: number;
  metadata?: Record<string, unknown>;
}

export type ViewMode =
  | "TREE"
  | "DEPENDENCY"
  | "RISK"
  | "AGENT"
  | "HISTORY";

export const ALL_VIEW_MODES: readonly ViewMode[] = [
  "TREE",
  "DEPENDENCY",
  "RISK",
  "AGENT",
  "HISTORY",
] as const;

export type ZoomLevel = "L0" | "L1" | "L2" | "L3" | "L4" | "L5";

export const ALL_ZOOM_LEVELS: readonly ZoomLevel[] = [
  "L0",
  "L1",
  "L2",
  "L3",
  "L4",
  "L5",
] as const;

export type InspectorTab =
  | "Overview"
  | "GitState"
  | "Task"
  | "Agent"
  | "Risk"
  | "Tests"
  | "Files"
  | "Commits"
  | "Relations"
  | "History"
  | "Actions";

export const ALL_INSPECTOR_TABS: readonly InspectorTab[] = [
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
] as const;

// 18 Action (per spec §4.4 A-01..A-18) — WorktreeCanvas Action 矩阵.
export type WorktreeAction =
  | "CreateWorktree"
  | "OpenWorktree"
  | "OpenInIde"
  | "Compare"
  | "SyncMain"
  | "Rebase"
  | "Merge"
  | "CreatePR"
  | "Lock"
  | "Unlock"
  | "Delete"
  | "Cleanup"
  | "Archive"
  | "MarkSuperseded"
  | "SetDependency"
  | "RemoveDependency"
  | "Focus"
  | "ExplainRisk";

export type ActionDanger = "Safe" | "Warning" | "Destructive";

export const ACTION_DANGER: Record<WorktreeAction, ActionDanger> = {
  CreateWorktree: "Safe",
  OpenWorktree: "Safe",
  OpenInIde: "Safe",
  Compare: "Safe",
  SyncMain: "Warning",
  Rebase: "Warning",
  Merge: "Destructive",
  CreatePR: "Warning",
  Lock: "Safe",
  Unlock: "Safe",
  Delete: "Destructive",
  Cleanup: "Destructive",
  Archive: "Warning",
  MarkSuperseded: "Warning",
  SetDependency: "Warning",
  RemoveDependency: "Warning",
  Focus: "Safe",
  ExplainRisk: "Safe",
};

// Canvas renderer interface (per BD §9 + DD §11).
export interface Point2D {
  x: number;
  y: number;
}

export interface Viewport {
  x: number;
  y: number;
  zoom: number;
  width: number;
  height: number;
}

export interface RenderHandle {
  id: string;
  destroy: () => void;
}

export interface CanvasRenderer {
  renderNode(node: WorktreeNodeData, zoomLevel: ZoomLevel): RenderHandle;
  renderEdge(edge: EdgeData, layout: LayoutResult): RenderHandle;
  updateViewport(viewport: Viewport): void;
  hitTest(point: Point2D): string | null;
}

export interface LayoutResult {
  positions: Map<string, Point2D>;
  bounds: { minX: number; minY: number; maxX: number; maxY: number };
}

// 15 SSE Event (per spec §4.5).
export type SseEventType =
  | "WorktreeCreated"
  | "WorktreeDeleted"
  | "WorktreeChanged"
  | "WorktreeMerged"
  | "BranchUpdated"
  | "MainUpdated"
  | "AgentStarted"
  | "AgentStopped"
  | "TaskChanged"
  | "TestCompleted"
  | "RiskDetected"
  | "RiskResolved"
  | "HealthChanged"
  | "RelationCreated"
  | "RelationRemoved";

export const ALL_SSE_EVENTS: readonly SseEventType[] = [
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
] as const;
