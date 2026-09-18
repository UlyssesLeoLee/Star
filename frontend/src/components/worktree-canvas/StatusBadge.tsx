// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/StatusBadge.tsx
// 7 Human State 状态 badge (per SRS §九).
"use client";
import { memo } from "react";
import type { WorktreeHumanState } from "./types";

const COLORS: Record<WorktreeHumanState, string> = {
  RUNNING: "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300",
  WAITING: "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300",
  READY: "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300",
  DIVERGED: "bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300",
  CONFLICT: "bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300",
  MERGED: "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
  STALE: "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400",
};

interface Props {
  state: WorktreeHumanState;
}
function Inner({ state }: Props) {
  return (
    <span
      data-testid={`status-badge-${state}`}
      className={`rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase ${COLORS[state]}`}
    >
      {state}
    </span>
  );
}
export const StatusBadge = memo(Inner);
export default StatusBadge;
