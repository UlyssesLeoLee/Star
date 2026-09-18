// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ZoomL4.tsx
// L4 — File 级 (per SRS §6 近距离更深一级).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function ZoomL4Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);

  return (
    <div data-zoom="L4" data-testid="zoom-l4" className="p-4 text-xs">
      {!wt ? (
        <p className="text-slate-500">请选择 Worktree 查看 File 列表</p>
      ) : (
        <>
          <div className="mb-2 font-medium">{wt.name ?? wt.branch}</div>
          <p className="text-slate-500">
            File list (placeholder, 阶段 2 接 git-adapter ls-files)
          </p>
          <ul className="mt-2 space-y-1 font-mono text-slate-600 dark:text-slate-400">
            <li>— (占位, 0 业务调用)</li>
          </ul>
        </>
      )}
    </div>
  );
}

export const ZoomL4 = memo(ZoomL4Inner);
export default ZoomL4;
