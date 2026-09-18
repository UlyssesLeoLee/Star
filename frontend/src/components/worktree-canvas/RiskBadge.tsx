// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/RiskBadge.tsx
// Risk count badge (per DD §13).
"use client";
import { memo } from "react";

interface Props {
  count: number;
}

function Inner({ count }: Props) {
  if (count === 0) {
    return <span data-testid="risk-badge" className="text-[10px] text-slate-400">R 0</span>;
  }
  const cls = count >= 3 ? "text-red-600" : "text-amber-600";
  return (
    <span data-testid="risk-badge" className={`text-[10px] font-bold ${cls}`}>
      R {count}
    </span>
  );
}
export const RiskBadge = memo(Inner);
export default RiskBadge;
