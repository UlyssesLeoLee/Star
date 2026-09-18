// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/WorktreeCard.tsx
// Default Worktree Node card (per DD §10 12 字段 + INV-WC-05/07).
"use client";
import { memo } from "react";
import type { WorktreeNodeData } from "./types";
import { StatusBadge } from "./StatusBadge";
import { HealthBadge } from "./HealthBadge";
import { RiskBadge } from "./RiskBadge";

interface Props {
  wt: WorktreeNodeData;
  selected?: boolean;
  onClick?: () => void;
}

function Inner({ wt, selected, onClick }: Props) {
  return (
    <div
      data-testid={`worktree-card-${wt.id}`}
      data-selected={selected ? "true" : "false"}
      onClick={onClick}
      className={`rounded border bg-white p-2 text-xs shadow-sm transition dark:bg-slate-900 ${
        selected
          ? "border-blue-500 ring-2 ring-blue-200"
          : "border-slate-200 hover:border-slate-300 dark:border-slate-700"
      }`}
    >
      <div className="mb-1 flex items-center justify-between">
        <span className="truncate font-medium" title={wt.name}>
          {wt.name ?? wt.branch}
        </span>
        <StatusBadge state={wt.humanState} />
      </div>
      <div className="mb-1 truncate font-mono text-slate-500">{wt.branch}</div>
      <div className="flex items-center justify-between text-slate-400">
        <span>+{wt.ahead} / -{wt.behind}</span>
        <HealthBadge score={wt.healthScore} />
        <RiskBadge count={wt.riskCount ?? 0} />
      </div>
    </div>
  );
}
export const WorktreeCard = memo(Inner);
export default WorktreeCard;
