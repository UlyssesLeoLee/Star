// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorTask.tsx
// Inspector Tab 3/11: Task.

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function InspectorTaskInner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-task" className="p-4 text-sm">
      {!wt ? (
        <p className="text-slate-500">请选择 Worktree</p>
      ) : (
        <dl className="grid grid-cols-2 gap-2">
          <dt className="text-slate-500">Task ID</dt>
          <dd className="font-mono text-xs">{wt.taskId ?? "—"}</dd>
          <dt className="text-slate-500">Repository</dt>
          <dd className="font-mono text-xs">{wt.repositoryId}</dd>
        </dl>
      )}
    </div>
  );
}

export const InspectorTask = memo(InspectorTaskInner);
export default InspectorTask;
