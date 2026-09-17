// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/RiskView.tsx
//
// RISK VIEW — 正常节点弱化, 高风险节点强调 (per SRS §18 R7 Exception-first).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function RiskViewInner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const edges = useWorktreeCanvasStore((s) => s.edges);

  const ranked = [...worktrees].sort(
    (a, b) => (b.riskCount ?? 0) - (a.riskCount ?? 0)
  );
  const conflicts = edges.filter((e) => e.kind === "CONFLICTS_WITH");

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
        RISK VIEW (FR-UI-009)
      </div>
      <ul className="space-y-2">
        {ranked.map((wt) => {
          const risk = wt.riskCount ?? 0;
          const opacity = risk === 0 ? 0.4 : 1;
          return (
            <li
              key={wt.id}
              data-testid={`risk-view-wt-${wt.id}`}
              data-risk={risk}
              style={{ opacity }}
              className={`rounded border px-3 py-2 text-sm ${
                risk >= 3
                  ? "border-red-300 bg-red-50 dark:border-red-800 dark:bg-red-900/20"
                  : risk > 0
                  ? "border-amber-300 bg-amber-50 dark:border-amber-800 dark:bg-amber-900/20"
                  : "border-slate-200 dark:border-slate-700"
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="font-medium">{wt.name ?? wt.branch}</span>
                <span
                  className={`text-xs font-bold ${
                    risk >= 3
                      ? "text-red-600"
                      : risk > 0
                      ? "text-amber-600"
                      : "text-slate-400"
                  }`}
                >
                  risk: {risk}
                </span>
              </div>
              <div className="mt-1 text-xs text-slate-500">
                health: {wt.healthScore} · {wt.humanState}
              </div>
            </li>
          );
        })}
      </ul>
      <div className="mt-6">
        <h3 className="mb-2 text-sm font-semibold">
          Conflicts ({conflicts.length})
        </h3>
        {conflicts.length === 0 ? (
          <p className="text-xs text-slate-500">无 CONFLICTS_WITH</p>
        ) : (
          <ul className="space-y-1 text-sm">
            {conflicts.map((e, i) => (
              <li
                key={i}
                data-testid={`risk-conflict-${i}`}
                className="rounded bg-red-50 px-2 py-1 font-mono text-xs dark:bg-red-900/20"
              >
                {e.source.slice(0, 8)} ⊗ {e.target.slice(0, 8)}
                {e.riskScore !== undefined && (
                  <span className="ml-2 text-red-600">
                    score: {e.riskScore.toFixed(2)}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

export const RiskView = memo(RiskViewInner);
export default RiskView;
