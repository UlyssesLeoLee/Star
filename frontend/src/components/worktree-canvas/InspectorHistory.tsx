// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorHistory.tsx
// Inspector Tab 10/11: History.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-history" className="p-4 text-sm">
      {!wt ? <p className="text-slate-500">请选择 Worktree</p> : (
        <dl className="grid grid-cols-2 gap-2">
          <dt className="text-slate-500">Created</dt>
          <dd className="text-xs">{wt.lastActivityAt}</dd>
          <dt className="text-slate-500">Merged At</dt>
          <dd className="text-xs">{wt.mergedAt ?? "—"}</dd>
          <dt className="text-slate-500">Archived</dt>
          <dd>{wt.archived ? "是" : "否"}</dd>
        </dl>
      )}
    </div>
  );
}
export const InspectorHistory = memo(Inner);
export default InspectorHistory;
