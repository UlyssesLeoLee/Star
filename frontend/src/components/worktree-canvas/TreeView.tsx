// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/TreeView.tsx
//
// TREE View — 以 Main → Worktree 派生为核心 (per SRS §18 + DD §12 Tree Backbone).
//
// 守门合规: 'use client' 边界 (per 守门 #19 v19 SSR + 客户端组件边界).

"use client";

import { memo, useMemo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";
import type { WorktreeNodeData } from "./types";

/**
 * TreeView — 默认视图 (FR-UI-007).
 * 渲染 main → worktrees 树状结构, 7 状态用色码.
 */
function TreeViewInner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const select = useWorktreeCanvasStore((s) => s.selectWorktree);

  // 按 repository + branch parent 分组 (per DD §12 Tree Backbone).
  const grouped = useMemo(() => {
    const byRepo = new Map<string, WorktreeNodeData[]>();
    for (const wt of worktrees) {
      const arr = byRepo.get(wt.repositoryId) ?? [];
      arr.push(wt);
      byRepo.set(wt.repositoryId, arr);
    }
    return Array.from(byRepo.entries());
  }, [worktrees]);

  if (grouped.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-slate-500">
        暂无 Worktree 数据
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      <div className="mb-3 text-xs uppercase tracking-wide text-slate-500">
        TREE VIEW (FR-UI-007)
      </div>
      {grouped.map(([repoId, wts]) => (
        <div key={repoId} className="mb-6">
          <div className="mb-2 flex items-center gap-2 font-mono text-sm font-semibold">
            <span className="text-slate-400">●</span>
            <span>main</span>
            <span className="text-xs text-slate-500">({repoId.slice(0, 8)})</span>
          </div>
          <ul className="ml-6 border-l border-slate-200 pl-4 dark:border-slate-700">
            {wts.map((wt) => (
              <li
                key={wt.id}
                data-selected={selectedId === wt.id}
                data-testid={`tree-view-wt-${wt.id}`}
                onClick={() => select(wt.id)}
                className={`group my-1 cursor-pointer rounded px-2 py-1 text-sm transition ${
                  selectedId === wt.id
                    ? "bg-blue-50 dark:bg-blue-900/30"
                    : "hover:bg-slate-50 dark:hover:bg-slate-800/50"
                }`}
              >
                <span className="mr-2 text-slate-400">└─</span>
                <span className="font-medium">{wt.name ?? wt.branch}</span>
                <span className="ml-2 text-xs text-slate-500">
                  {wt.humanState} · ahead {wt.ahead} / behind {wt.behind}
                </span>
                <span className="ml-2 text-xs text-slate-400">
                  health {wt.healthScore}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}

export const TreeView = memo(TreeViewInner);
export default TreeView;
