// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/HistoryView.tsx
//
// HISTORY VIEW — 显示已 merged Worktree 和历史关系 (per SRS §19 Merged Worktree 处理).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function HistoryViewInner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const merged = worktrees
    .filter((wt) => wt.humanState === "MERGED")
    .sort(
      (a, b) =>
        new Date(b.mergedAt ?? b.lastActivityAt).getTime() -
        new Date(a.mergedAt ?? a.lastActivityAt).getTime()
    );

  return (
    <div className="h-full overflow-auto p-4">
      <div className="mb-3 text-xs uppercase tracking-wide text-slate-500">
        HISTORY VIEW (FR-UI-011)
      </div>
      {merged.length === 0 ? (
        <p className="text-sm text-slate-500">暂无已合并的 Worktree</p>
      ) : (
        <ul className="space-y-2">
          {merged.map((wt) => (
            <li
              key={wt.id}
              data-testid={`history-wt-${wt.id}`}
              className="rounded border border-slate-200 bg-slate-50 px-3 py-2 text-sm opacity-70 dark:border-slate-700 dark:bg-slate-800/30"
            >
              <div className="flex items-center justify-between">
                <span className="font-medium">{wt.name ?? wt.branch}</span>
                <span className="text-xs text-slate-500">
                  merged
                  {wt.mergedAt &&
                    ` · ${new Date(wt.mergedAt).toLocaleDateString()}`}
                </span>
              </div>
              <div className="mt-1 text-xs text-slate-500">
                {wt.branch} · last seen {wt.lastActivityAt.slice(0, 10)}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export const HistoryView = memo(HistoryViewInner);
export default HistoryView;
