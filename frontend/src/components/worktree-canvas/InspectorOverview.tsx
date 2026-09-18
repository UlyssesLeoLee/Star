// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorOverview.tsx
// Inspector Tab 1/11: Overview (per spec §4.3 + DD §21).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function InspectorOverviewInner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-overview" className="p-4 text-sm">
      {!wt ? (
        <p className="text-slate-500">请选择 Worktree</p>
      ) : (
        <dl className="grid grid-cols-2 gap-2">
          <dt className="text-slate-500">Name</dt>
          <dd className="font-medium">{wt.name ?? wt.branch}</dd>
          <dt className="text-slate-500">Branch</dt>
          <dd className="font-mono text-xs">{wt.branch}</dd>
          <dt className="text-slate-500">State</dt>
          <dd>{wt.humanState}</dd>
          <dt className="text-slate-500">Health</dt>
          <dd>{wt.healthScore}</dd>
          <dt className="text-slate-500">Ahead / Behind</dt>
          <dd>
            +{wt.ahead} / -{wt.behind}
          </dd>
        </dl>
      )}
    </div>
  );
}

export const InspectorOverview = memo(InspectorOverviewInner);
export default InspectorOverview;
