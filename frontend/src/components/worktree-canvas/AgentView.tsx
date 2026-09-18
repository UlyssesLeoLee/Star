// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/AgentView.tsx
//
// AGENT VIEW — 以 Agent → Task → Worktree 为核心 (per SRS §18).

"use client";

import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function AgentViewInner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);

  // 按 agentSessionId 分组 (per DD §15 Agent Session 一等实体).
  const byAgent = new Map<string, typeof worktrees>();
  for (const wt of worktrees) {
    const aid = wt.agentSessionId ?? "_no_agent";
    const arr = byAgent.get(aid) ?? [];
    arr.push(wt);
    byAgent.set(aid, arr);
  }

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
        AGENT VIEW (FR-UI-010)
      </div>
      <ul className="space-y-4">
        {Array.from(byAgent.entries()).map(([agentId, wts]) => (
          <li
            key={agentId}
            data-testid={`agent-group-${agentId}`}
            className="rounded border border-slate-200 p-3 dark:border-slate-700"
          >
            <div className="mb-2 flex items-center gap-2">
              <span className="font-mono text-xs text-slate-500">
                {agentId === "_no_agent" ? "no agent" : `agent: ${agentId.slice(0, 8)}`}
              </span>
              <span className="ml-auto text-xs text-slate-400">
                {wts.length} worktree{wts.length === 1 ? "" : "s"}
              </span>
            </div>
            <ul className="ml-4 space-y-1 text-sm">
              {wts.map((wt) => (
                <li
                  key={wt.id}
                  data-testid={`agent-view-wt-${wt.id}`}
                  className="rounded px-2 py-1 hover:bg-slate-50 dark:hover:bg-slate-800/50"
                >
                  <span className="font-medium">{wt.name ?? wt.branch}</span>
                  <span className="ml-2 text-xs text-slate-500">
                    {wt.taskId ? `task: ${wt.taskId.slice(0, 8)}` : "no task"} ·{" "}
                    {wt.humanState}
                  </span>
                </li>
              ))}
            </ul>
          </li>
        ))}
      </ul>
    </div>
  );
}

export const AgentView = memo(AgentViewInner);
export default AgentView;
