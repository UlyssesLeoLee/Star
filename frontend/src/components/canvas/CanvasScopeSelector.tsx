/*
@cypher schema=1 source_sha256=9d39f1b9a62975c03cb7bd05dc9a15bca1d9c0c6b8d96a5f22bc0af02a9a6dc8
MERGE (self:File {path:"frontend/src/components/canvas/CanvasScopeSelector.tsx"})
MERGE (comp:Symbol {id:"frontend/src/components/canvas/CanvasScopeSelector.tsx::CanvasScopeSelector",kind:"component"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (dataStore:Symbol {id:"frontend/src/lib/store.ts::useStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(comp)
MERGE (comp)-[:CALLS]->(navStore)
MERGE (comp)-[:READS]->(dataStore)
@endcypher
*/
"use client";

// =====================================================================
// CanvasScopeSelector — Canvas 三级范围选择器 (Project → Lane → Worktree)
//   per 2026-11-16 Canvas 范围选择器需求 (10 条第 4 条):
//   - 紧凑横向三级联动; 默认值取 navStore (selectedProjectId /
//     selectedLaneId / selectedWorktreeId)
//   - onChange 写回 navStore (切 Project 自动清空下级上下文, 防跨项目残留)
//   - 通过 onScopeChange 通知宿主 (canvas/[id] page) 触发画布重新加载
//   - 无 lane_id 的历史 worktree 归入"默认 LANE (未归类)"伪 lane
//     (同 ProjectRunTree fallback 分组, 不做 hardcode lane 回退)
// 数据: store.projects / store.lanes / store.worktrees (seed 预览;
//   宿主目录投影接入后切换为授权数据)
// =====================================================================

import { useMemo } from "react";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";

/** 未归类 (lane_id 缺失) worktree 的伪 lane id — 仅用于选择器分组展示 */
export const UNCLASSIFIED_LANE_ID = "__unclassified";

export interface CanvasScope {
  projectId: string;
  laneId: string;
  worktreeId: string;
}

export interface CanvasScopeSelectorProps {
  /** 范围变更回调 (宿主据此触发画布重新加载) */
  onScopeChange?: (scope: CanvasScope) => void;
}

export function CanvasScopeSelector({ onScopeChange }: CanvasScopeSelectorProps) {
  const projects = useStore((s) => s.projects);
  const lanes = useStore((s) => s.lanes);
  const worktrees = useStore((s) => s.worktrees);

  const selectedProjectId = useNavStore((s) => s.selectedProjectId);
  const selectedLaneId = useNavStore((s) => s.selectedLaneId);
  const selectedWorktreeId = useNavStore((s) => s.selectedWorktreeId);
  const selectedBranch = useNavStore((s) => s.selectedBranch);
  const setSelectedProjectId = useNavStore((s) => s.setSelectedProjectId);
  const setLaneContext = useNavStore((s) => s.setLaneContext);

  // 防持久化残留/跨项目脏数据: 选项列表中不存在的选中值回退为"未选择"
  const effectiveProjectId = projects.some((p) => p.id === selectedProjectId)
    ? selectedProjectId
    : "";

  const projectLanes = useMemo(
    () => lanes.filter((l) => l.project_id === effectiveProjectId),
    [lanes, effectiveProjectId],
  );
  const projectWorktrees = useMemo(
    () => worktrees.filter((w) => w.project_id === effectiveProjectId),
    [worktrees, effectiveProjectId],
  );
  const hasUnclassified = projectWorktrees.some((w) => !w.lane_id);
  const laneId =
    selectedLaneId &&
    (projectLanes.some((l) => l.id === selectedLaneId) ||
      (selectedLaneId === UNCLASSIFIED_LANE_ID && hasUnclassified))
      ? selectedLaneId
      : "";
  const laneWorktrees = useMemo(
    () => projectWorktrees.filter((w) => (w.lane_id ?? UNCLASSIFIED_LANE_ID) === laneId),
    [projectWorktrees, laneId],
  );
  const effectiveWorktreeId =
    selectedWorktreeId && laneWorktrees.some((w) => w.id === selectedWorktreeId)
      ? selectedWorktreeId
      : "";

  const handleProjectChange = (projectId: string) => {
    // setSelectedProjectId 内部同步清空 selectedLaneId/selectedWorktreeId/selectedBranch
    setSelectedProjectId(projectId);
    onScopeChange?.({ projectId, laneId: "", worktreeId: "" });
  };

  const handleLaneChange = (nextLaneId: string) => {
    setLaneContext(nextLaneId, "", "");
    onScopeChange?.({ projectId: effectiveProjectId, laneId: nextLaneId, worktreeId: "" });
  };

  const handleWorktreeChange = (worktreeId: string) => {
    const wt = worktrees.find((w) => w.id === worktreeId);
    setLaneContext(laneId, worktreeId, wt?.branch ?? selectedBranch);
    onScopeChange?.({ projectId: effectiveProjectId, laneId, worktreeId });
  };

  const project = projects.find((p) => p.id === effectiveProjectId);
  const lane = projectLanes.find((l) => l.id === laneId);
  const worktree = laneWorktrees.find((w) => w.id === effectiveWorktreeId);
  const badgeText = worktree
    ? `${project?.name ?? effectiveProjectId} · ${lane?.name ?? laneId} · ${worktree.name}`
    : lane
      ? `${project?.name ?? effectiveProjectId} · ${lane.name}`
      : project
        ? project.name
        : "未指定范围";

  const selectCls =
    "h-6 rounded border border-line bg-bg-card px-1.5 text-[11px] text-ink outline-none focus:border-accent disabled:opacity-40";

  return (
    <div className="flex flex-wrap items-center gap-1.5" data-testid="canvas-scope-selector">
      <span className="shrink-0 font-mono text-[10px] text-ink-dim">范围</span>
      <select
        data-testid="canvas-scope-project"
        aria-label="Project 范围"
        className={selectCls}
        value={effectiveProjectId}
        onChange={(e) => handleProjectChange(e.target.value)}
      >
        <option value="">项目…</option>
        {projects.map((p) => (
          <option key={p.id} value={p.id}>
            {p.name} ({p.key})
          </option>
        ))}
      </select>
      <select
        data-testid="canvas-scope-lane"
        aria-label="Lane 范围"
        className={selectCls}
        value={laneId}
        disabled={!effectiveProjectId}
        onChange={(e) => handleLaneChange(e.target.value)}
      >
        <option value="">Lane…</option>
        {projectLanes.map((l) => (
          <option key={l.id} value={l.id}>
            {l.name}
          </option>
        ))}
        {hasUnclassified && <option value={UNCLASSIFIED_LANE_ID}>默认 LANE (未归类)</option>}
      </select>
      <select
        data-testid="canvas-scope-worktree"
        aria-label="Worktree 范围"
        className={selectCls}
        value={effectiveWorktreeId}
        disabled={!laneId}
        onChange={(e) => handleWorktreeChange(e.target.value)}
      >
        <option value="">Worktree…</option>
        {laneWorktrees.map((w) => (
          <option key={w.id} value={w.id}>
            {w.name} · {w.branch}
          </option>
        ))}
      </select>
      <span
        data-testid="canvas-scope-badge"
        className="rounded border border-line bg-bg-soft px-1.5 py-0.5 font-mono text-[9px] text-ink-mute"
      >
        {badgeText}
      </span>
    </div>
  );
}
