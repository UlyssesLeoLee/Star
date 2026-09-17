// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorRisk.tsx
// Inspector Tab 5/11: Risk.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const edges = useWorktreeCanvasStore((s) => s.edges);
  const wt = worktrees.find((w) => w.id === selectedId);
  const riskEdges = edges.filter(
    (e) => (e.source === selectedId || e.target === selectedId) && (e.kind === "CONFLICTS_WITH" || e.kind === "OVERLAPS_WITH")
  );
  return (
    <div data-testid="inspector-risk" className="p-4 text-sm">
      {!wt ? <p className="text-slate-500">请选择 Worktree</p> : (
        <>
          <div className="mb-2 text-slate-500">Risk Count: {wt.riskCount ?? 0}</div>
          <ul className="space-y-1 text-xs">
            {riskEdges.map((e, i) => (
              <li key={i} className="rounded bg-amber-50 px-2 py-1 font-mono dark:bg-amber-900/20">
                {e.kind}: {e.source.slice(0, 8)} → {e.target.slice(0, 8)}
                {e.riskScore !== undefined && <span className="ml-2">score {e.riskScore.toFixed(2)}</span>}
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}
export const InspectorRisk = memo(Inner);
export default InspectorRisk;
