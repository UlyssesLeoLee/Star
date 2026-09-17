// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ZoomL5.tsx
// L5 — Symbol 级 (per SRS §6 最近距离, AST-level).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function ZoomL5Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  return (
    <div data-zoom="L5" data-testid="zoom-l5" className="p-4 text-xs">
      <p className="text-slate-500">
        {selectedId
          ? `Symbol-level view for ${selectedId.slice(0, 8)} (placeholder, 阶段 2 接 treesitter / AST overlap)`
          : "请选择 Worktree 查看 Symbol 级"}
      </p>
      <ul className="mt-2 space-y-1 font-mono text-slate-600 dark:text-slate-400">
        <li>— (占位, 0 业务调用)</li>
      </ul>
    </div>
  );
}

export const ZoomL5 = memo(ZoomL5Inner);
export default ZoomL5;
