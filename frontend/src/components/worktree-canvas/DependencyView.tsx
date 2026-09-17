// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/DependencyView.tsx
//
// DEPENDENCY VIEW — 突出 DEPENDS_ON / BLOCKS Edge (per SRS §18 + DD §8).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function DependencyViewInner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const edges = useWorktreeCanvasStore((s) => s.edges);

  const dependsEdges = edges.filter(
    (e) => e.kind === "DEPENDS_ON" || e.kind === "BLOCKS"
  );
  const supersedesEdges = edges.filter((e) => e.kind === "SUPERSEDES");

  const wtName = (id: string) =>
    worktrees.find((w) => w.id === id)?.name ?? id.slice(0, 8);

  if (worktrees.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-slate-500">
        暂无 Worktree 数据
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      <div className="mb-3 text-xs uppercase tracking-wide text-slate-500">
        DEPENDENCY VIEW (FR-UI-008)
      </div>
      <section className="mb-6">
        <h3 className="mb-2 text-sm font-semibold">
          DEPENDS_ON / BLOCKS ({dependsEdges.length})
        </h3>
        {dependsEdges.length === 0 ? (
          <p className="text-xs text-slate-500">无依赖关系</p>
        ) : (
          <ul className="space-y-1 text-sm">
            {dependsEdges.map((e, i) => (
              <li
                key={i}
                data-testid={`dep-edge-${i}`}
                className="rounded bg-slate-50 px-2 py-1 font-mono dark:bg-slate-800/50"
              >
                <span>{wtName(e.source)}</span>
                <span className="mx-2 text-slate-400">
                  {e.kind === "DEPENDS_ON" ? "→" : "⊘"}
                </span>
                <span>{wtName(e.target)}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
      <section>
        <h3 className="mb-2 text-sm font-semibold">
          SUPERSEDES ({supersedesEdges.length})
        </h3>
        {supersedesEdges.length === 0 ? (
          <p className="text-xs text-slate-500">无取代关系</p>
        ) : (
          <ul className="space-y-1 text-sm">
            {supersedesEdges.map((e, i) => (
              <li
                key={i}
                className="rounded bg-amber-50 px-2 py-1 font-mono dark:bg-amber-900/20"
              >
                <span>{wtName(e.source)}</span>
                <span className="mx-2 text-amber-500">⊃</span>
                <span>{wtName(e.target)}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

export const DependencyView = memo(DependencyViewInner);
export default DependencyView;
