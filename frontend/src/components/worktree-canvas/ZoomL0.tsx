// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ZoomL0.tsx
//
// L0 — Project 级 (per SRS §6, 远距离只显示 Repo + 计数).
// 守门 INV-WC-06 (Semantic Zoom 6 级).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function ZoomL0Inner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const byRepo = new Map<string, number>();
  for (const wt of worktrees) {
    byRepo.set(wt.repositoryId, (byRepo.get(wt.repositoryId) ?? 0) + 1);
  }
  return (
    <div
      data-zoom="L0"
      data-testid="zoom-l0"
      className="grid grid-cols-1 gap-3 p-4 md:grid-cols-2 lg:grid-cols-3"
    >
      {Array.from(byRepo.entries()).map(([repoId, count]) => (
        <div
          key={repoId}
          className="rounded border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-900"
        >
          <div className="font-mono text-xs text-slate-500">
            {repoId.slice(0, 8)}
          </div>
          <div className="mt-1 text-2xl font-bold">{count}</div>
          <div className="text-xs text-slate-500">active worktrees</div>
        </div>
      ))}
      {byRepo.size === 0 && (
        <p className="col-span-full text-sm text-slate-500">暂无 Repository</p>
      )}
    </div>
  );
}

export const ZoomL0 = memo(ZoomL0Inner);
export default ZoomL0;
