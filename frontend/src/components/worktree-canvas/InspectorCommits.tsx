// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorCommits.tsx
// Inspector Tab 8/11: Commits.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-commits" className="p-4 text-sm">
      {!wt ? <p className="text-slate-500">请选择 Worktree</p> : (
        <p className="text-slate-500">
          ahead {wt.ahead} / behind {wt.behind} (commit list placeholder, 阶段 2 接 git-adapter)
        </p>
      )}
    </div>
  );
}
export const InspectorCommits = memo(Inner);
export default InspectorCommits;
