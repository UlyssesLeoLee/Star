// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorTests.tsx
// Inspector Tab 6/11: Tests.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-tests" className="p-4 text-sm">
      {!wt ? <p className="text-slate-500">请选择 Worktree</p> : (
        <dl className="grid grid-cols-2 gap-2">
          <dt className="text-slate-500">Test State</dt>
          <dd className="font-mono">{wt.testState}</dd>
        </dl>
      )}
    </div>
  );
}
export const InspectorTests = memo(Inner);
export default InspectorTests;
