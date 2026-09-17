// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ZoomL2.tsx
// L2 — Task / Agent (per SRS §6 中距离更深一级).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function ZoomL2Inner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  return (
    <div
      data-zoom="L2"
      data-testid="zoom-l2"
      className="grid grid-cols-1 gap-2 p-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
    >
      {worktrees.map((wt) => (
        <div
          key={wt.id}
          data-testid={`zoom-l2-wt-${wt.id}`}
          className="rounded border border-slate-200 bg-white p-3 text-xs shadow-sm dark:border-slate-700 dark:bg-slate-900"
        >
          <div className="truncate font-medium" title={wt.name}>
            {wt.name ?? wt.branch}
          </div>
          <dl className="mt-2 grid grid-cols-2 gap-1">
            <dt className="text-slate-500">Task</dt>
            <dd className="truncate font-mono">
              {wt.taskId ? wt.taskId.slice(0, 8) : "—"}
            </dd>
            <dt className="text-slate-500">Agent</dt>
            <dd className="truncate font-mono">
              {wt.agentSessionId ? wt.agentSessionId.slice(0, 8) : "—"}
            </dd>
            <dt className="text-slate-500">State</dt>
            <dd>{wt.humanState}</dd>
            <dt className="text-slate-500">Test</dt>
            <dd>{wt.testState}</dd>
          </dl>
        </div>
      ))}
    </div>
  );
}

export const ZoomL2 = memo(ZoomL2Inner);
export default ZoomL2;
