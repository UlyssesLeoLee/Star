// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ZoomL3.tsx
// L3 — Commit 级 (per SRS §6 近距离).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function ZoomL3Inner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const wt = worktrees.find((w) => w.id === selectedId);

  if (!wt) {
    return (
      <div
        data-zoom="L3"
        data-testid="zoom-l3"
        className="p-4 text-sm text-slate-500"
      >
        请选择一个 Worktree 查看 Commit 级细节
      </div>
    );
  }
  return (
    <div
      data-zoom="L3"
      data-testid="zoom-l3"
      className="p-4 text-xs"
    >
      <div className="mb-2 font-medium">{wt.name ?? wt.branch}</div>
      <ul className="space-y-1 font-mono text-slate-600 dark:text-slate-400">
        <li>commit list (placeholder, 阶段 2 接 git-adapter)</li>
        <li className="text-slate-400">ahead {wt.ahead} commits</li>
        <li className="text-slate-400">behind {wt.behind} commits</li>
      </ul>
    </div>
  );
}

export const ZoomL3 = memo(ZoomL3Inner);
export default ZoomL3;
