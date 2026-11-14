/*
@cypher schema=1 source_sha256=789ced53c12fdbd9de74625b50b742d1511bd7f80f4739e7a2fb522bbb978708
MERGE (self:File {path:"frontend/src/components/run/BranchWorkspace.tsx"})
MERGE (shell:Symbol {id:"frontend/src/components/run/BranchWorkspace.tsx::BranchWorkspace",kind:"component"})
MERGE (self)-[:DEFINES]->(shell)
@endcypher
*/
// =====================================================================
// BranchWorkspace — Branch 功能集面板
// (per 2026-11-14 用户架构: Tenant → Project → LANE → Worktree → Branch,
//  每个 Branch 有自己的一整套功能; 本面板 = 选中 worktree/branch 后的功能集)
// 功能集 tabs: 任务卡(看板) / Agent 会话 / Validation / Nema / Canvas
// 数据按 worktree scope 过滤 (seed 无宿主可达降级)
// =====================================================================
"use client";

import { useMemo, useState } from "react";
import { CheckSquare, Bot, ShieldCheck, FileText, Frame } from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useTranslation } from "@/lib/i18n";

export type BranchAppTab = "tasks" | "agents" | "validation" | "nema" | "canvas";

const BRANCH_APP_TABS: Array<{ id: BranchAppTab; icon: typeof CheckSquare }> = [
  { id: "tasks", icon: CheckSquare },
  { id: "agents", icon: Bot },
  { id: "validation", icon: ShieldCheck },
  { id: "nema", icon: FileText },
  { id: "canvas", icon: Frame },
];

export function BranchWorkspace({ worktreeId, branch, projectId }: {
  worktreeId: string;
  branch: string;
  projectId: string;
}) {
  const { t } = useTranslation();
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const agentSessions = useStore((s) => s.agentSessions);
  const canvases = useStore((s) => s.canvases);
  const [active, setActive] = useState<BranchAppTab>("tasks");

  const worktree = worktrees.find((w) => w.id === worktreeId) ?? null;
  const scopedItems = useMemo(
    () => workItems.filter((w) => w.worktree_id === worktreeId || (!w.worktree_id && w.project_id === projectId)),
    [workItems, worktreeId, projectId],
  );
  const scopedAgents = useMemo(
    () => agentSessions.filter((a) => a.worktree_id === worktreeId),
    [agentSessions, worktreeId],
  );
  const scopedCanvases = useMemo(
    () => canvases.filter((c) => (c as { ref_kind?: string; ref_id?: string }).ref_id === projectId),
    [canvases, projectId],
  );

  return (
    <section
      data-testid="branch-workspace"
      data-worktree-id={worktreeId}
      data-branch={branch}
      className="rounded-xl border-2 border-accent/40 bg-bg-card p-4"
    >
      <header className="mb-3">
        <div className="flex items-center gap-2 flex-wrap">
          <h2 className="text-base font-bold tracking-tight text-ink">{t.branchWorkspace.title}</h2>
          <span className="text-[10px] font-mono px-1.5 py-0.5 rounded border border-accent/50 text-accent">{branch}</span>
        </div>
        <p className="mt-1 text-xs text-ink-dim">
          {t.branchWorkspace.subtitle} · {worktree?.name ?? worktreeId} · {worktree?.status ?? ""}
        </p>
      </header>

      <div role="tablist" aria-label="Branch Apps" className="flex flex-wrap gap-1.5">
        {BRANCH_APP_TABS.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={active === tab.id}
              data-testid={`branch-app-tab-${tab.id}`}
              onClick={() => setActive(tab.id)}
              className={clsx(
                "flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors",
                active === tab.id
                  ? "border-accent bg-accent/15 text-accent font-semibold"
                  : "border-line text-ink-mute hover:bg-bg-soft/60 hover:text-ink",
              )}
            >
              <Icon size={13} /> {t.branchWorkspace.tabs[tab.id]}
            </button>
          );
        })}
      </div>

      <div role="tabpanel" aria-label={t.branchWorkspace.tabs[active]} className="mt-4">
        {active === "tasks" && (
          <ul className="space-y-1.5" data-testid="branch-app-tasks">
            {scopedItems.length === 0 && <li className="text-xs text-ink-mute">{t.branchWorkspace.empty.tasks}</li>}
            {scopedItems.map((item) => (
              <li key={item.id} className="flex items-center gap-2 rounded-lg border border-line bg-bg-soft/50 px-3 py-2 text-xs">
                <span className={clsx("size-2 rounded-full", item.status === "done" ? "bg-ok" : item.status === "in_progress" ? "bg-warn" : "bg-ink-mute")} />
                <span className="truncate flex-1 text-ink">{item.title}</span>
                <span className="font-mono text-[10px] text-ink-mute">{item.status}</span>
              </li>
            ))}
          </ul>
        )}
        {active === "agents" && (
          <ul className="space-y-1.5" data-testid="branch-app-agents">
            {scopedAgents.length === 0 && <li className="text-xs text-ink-mute">{t.branchWorkspace.empty.agents}</li>}
            {scopedAgents.map((agent) => (
              <li key={agent.id} className="flex items-center gap-2 rounded-lg border border-line bg-bg-soft/50 px-3 py-2 text-xs">
                <Bot size={13} className="text-emerald-400 shrink-0" />
                <span className="truncate flex-1 text-ink">{agent.name}</span>
                <span className="font-mono text-[10px] text-ink-mute">{agent.status} · {agent.current_step}</span>
              </li>
            ))}
          </ul>
        )}
        {active === "validation" && (
          <div data-testid="branch-app-validation" className="text-xs text-ink-mute">
            {t.branchWorkspace.empty.validation}
          </div>
        )}
        {active === "nema" && (
          <div data-testid="branch-app-nema" className="text-xs text-ink-mute">
            {t.branchWorkspace.empty.nema}
          </div>
        )}
        {active === "canvas" && (
          <ul className="space-y-1.5" data-testid="branch-app-canvas">
            {scopedCanvases.length === 0 && <li className="text-xs text-ink-mute">{t.branchWorkspace.empty.canvas}</li>}
            {scopedCanvases.map((canvas) => (
              <li key={canvas.id} className="text-xs text-ink">
                {canvas.id} · {canvas.title}
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
