// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ZoomL1.tsx
// L1 — Repository / Main / Worktree (per SRS §6 中距离).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function ZoomL1Inner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  return (
    <div
      data-zoom="L1"
      data-testid="zoom-l1"
      className="grid grid-cols-1 gap-2 p-4 md:grid-cols-2 lg:grid-cols-3"
    >
      {worktrees.map((wt) => (
        <div
          key={wt.id}
          data-testid={`zoom-l1-wt-${wt.id}`}
          className="rounded border border-slate-200 bg-white p-3 text-sm shadow-sm dark:border-slate-700 dark:bg-slate-900"
        >
          <div className="font-medium">{wt.name ?? wt.branch}</div>
          <div className="mt-1 flex justify-between text-xs text-slate-500">
            <span>{wt.humanState}</span>
            <span>
              +{wt.ahead} / -{wt.behind}
            </span>
          </div>
          <div className="mt-1 text-xs text-slate-400">
            health: {wt.healthScore}
          </div>
        </div>
      ))}
      {worktrees.length === 0 && (
        <p className="col-span-full text-sm text-slate-500">暂无 Worktree</p>
      )}
    </div>
  );
}

export const ZoomL1 = memo(ZoomL1Inner);
export default ZoomL1;
