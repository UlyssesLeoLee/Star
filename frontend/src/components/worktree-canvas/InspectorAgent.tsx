// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorAgent.tsx
// Inspector Tab 4/11: Agent.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const wt = worktrees.find((w) => w.id === selectedId);
  return (
    <div data-testid="inspector-agent" className="p-4 text-sm">
      {!wt ? <p className="text-slate-500">请选择 Worktree</p> : (
        <dl className="grid grid-cols-2 gap-2">
          <dt className="text-slate-500">Agent Session</dt>
          <dd className="font-mono text-xs">{wt.agentSessionId ?? "—"}</dd>
          <dt className="text-slate-500">Last Activity</dt>
          <dd className="text-xs">{wt.lastActivityAt}</dd>
        </dl>
      )}
    </div>
  );
}
export const InspectorAgent = memo(Inner);
export default InspectorAgent;
