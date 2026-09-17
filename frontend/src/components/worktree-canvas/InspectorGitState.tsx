// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorGitState.tsx
// Inspector Tab 2/11: GitState (per spec §4.3).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function InspectorGitStateInner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-gitstate" className="p-4 text-sm">
      {!wt ? (
        <p className="text-slate-500">请选择 Worktree</p>
      ) : (
        <dl className="grid grid-cols-2 gap-2">
          <dt className="text-slate-500">Branch</dt>
          <dd className="font-mono text-xs">{wt.branch}</dd>
          <dt className="text-slate-500">Ahead</dt>
          <dd>{wt.ahead}</dd>
          <dt className="text-slate-500">Behind</dt>
          <dd>{wt.behind}</dd>
          <dt className="text-slate-500">Dirty</dt>
          <dd>{wt.dirtyState}</dd>
          <dt className="text-slate-500">Last Activity</dt>
          <dd className="text-xs">{wt.lastActivityAt}</dd>
          <dt className="text-slate-500">Locked</dt>
          <dd>{wt.locked ? "🔒 是" : "否"}</dd>
        </dl>
      )}
    </div>
  );
}

export const InspectorGitState = memo(InspectorGitStateInner);
export default InspectorGitState;
