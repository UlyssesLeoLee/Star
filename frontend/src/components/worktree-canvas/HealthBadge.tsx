// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/HealthBadge.tsx
// Health score badge (per DD §12 0-100).
"use client";
import { memo } from "react";

interface Props {
  score: number;
}

function color(score: number): string {
  if (score >= 80) return "text-green-600";
  if (score >= 60) return "text-amber-600";
  return "text-red-600";
}

function Inner({ score }: Props) {
  return (
    <span data-testid="health-badge" className={`text-[10px] font-bold ${color(score)}`}>
      H {score}
    </span>
  );
}
export const HealthBadge = memo(Inner);
export default HealthBadge;
