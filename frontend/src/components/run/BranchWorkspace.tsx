/*
@cypher schema=1 source_sha256=a3b21d8a9c3bacb33f4cd753210ea63088561680cff6c85f29f268fa5e7281c6
MERGE (self:File {path:"frontend/src/components/run/BranchWorkspace.tsx"})
MERGE (shell:Symbol {id:"frontend/src/components/run/BranchWorkspace.tsx::BranchWorkspace",kind:"component"})
MERGE (self)-[:DEFINES]->(shell)
@endcypher
*/
// =====================================================================
// BranchWorkspace — Lane 级中央应用面板 (per 2026-11-15 用户发令: 布局整合)
// 中央区域 = Lane 级应用集: Backlog/Sprint / Canvas / Workflow / BI·Benchmark / Plugins
// (Agent 中心功能并入 Canvas 界面, 不再独立 tab)
// 任务卡 = Branch 下子项, 打开同一个可关闭的任务详情子窗口 (WorkItemDetailDrawer)
// =====================================================================
"use client";

import { useMemo, useState } from "react";
import {
  ListTodo,
  Frame,
  Workflow,
  BarChart3,
  Puzzle,
  ArrowRight,
  GitBranch,
} from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useTranslation } from "@/lib/i18n";
import { SprintBoardView } from "@/components/sprint/SprintBoardView";
import { WorkItemDetailDrawer, type WorkItemDrawerMode } from "@/components/board/WorkItemDetailDrawer";

export type LaneAppTab = "backlog-sprint" | "canvas" | "workflow" | "bi" | "plugins";

const LANE_APP_TABS: Array<{ id: LaneAppTab; icon: typeof ListTodo }> = [
  { id: "backlog-sprint", icon: ListTodo },
  { id: "canvas", icon: Frame },
  { id: "workflow", icon: Workflow },
  { id: "bi", icon: BarChart3 },
  { id: "plugins", icon: Puzzle },
];

export function BranchWorkspace({ worktreeId, branch, projectId, laneId }: {
  worktreeId: string;
  branch: string;
  projectId: string;
  laneId?: string;
}) {
  const { t } = useTranslation();
  const worktrees = useStore((s) => s.worktrees);
  const workItems = useStore((s) => s.workItems);
  const sprints = useStore((s) => s.sprints);
  const workflows = useStore((s) => s.workflows);
  const integrations = useStore((s) => s.integrations);
  const lanes = useStore((s) => s.lanes);
  const [active, setActive] = useState<LaneAppTab>("backlog-sprint");
  // 可关闭任务详情子窗口 (per 需求二.4: 同一子窗口, 导航树/看板共用)
  const [drawer, setDrawer] = useState<WorkItemDrawerMode | null>(null);

  const worktree = worktrees.find((w) => w.id === worktreeId) ?? null;
  const lane = laneId ? lanes.find((l) => l.id === laneId) : undefined;

  const scopedItems = useMemo(
    () => workItems.filter((w) => w.worktree_id === worktreeId || (!w.worktree_id && w.project_id === projectId)),
    [workItems, worktreeId, projectId],
  );

  // BI: 该 worktree 真实统计 (状态分布 + sprint 进度 + 任务吞吐)
  const stats = useMemo(() => {
    const byStatus = new Map<string, number>();
    for (const w of scopedItems) byStatus.set(w.status, (byStatus.get(w.status) ?? 0) + 1);
    const done = byStatus.get("done") ?? 0;
    const throughput = scopedItems.filter((w) => w.updated_at && w.updated_at > new Date(Date.now() - 7 * 864e5).toISOString()).length;
    return { total: scopedItems.length, byStatus, done, throughput };
  }, [scopedItems]);

  const labels = t.branchWorkspace.tabs as unknown as Record<string, string>;

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
          {lane && (
            <span className="flex items-center gap-1 text-[10px] font-mono px-1.5 py-0.5 rounded border border-line text-ink-mute">
              {lane.name}
              {!lane.git_remote_url && <span className="text-warn"> · 未绑定远端</span>}
            </span>
          )}
        </div>
        <p className="mt-1 text-xs text-ink-dim">
          {t.branchWorkspace.subtitle} · {worktree?.name ?? worktreeId} · {worktree?.status ?? ""}
        </p>
      </header>

      <div role="tablist" aria-label="Lane Apps" className="flex flex-wrap gap-1.5">
        {LANE_APP_TABS.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={active === tab.id}
              data-testid={`lane-app-tab-${tab.id}`}
              onClick={() => setActive(tab.id)}
              className={clsx(
                "flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors",
                active === tab.id
                  ? "border-accent bg-accent/15 text-accent font-semibold"
                  : "border-line text-ink-mute hover:bg-bg-soft/60 hover:text-ink",
              )}
            >
              <Icon size={13} /> {labels[tab.id] ?? tab.id}
            </button>
          );
        })}
      </div>

      <div role="tabpanel" aria-label={labels[active] ?? active} className="mt-4">
        {active === "backlog-sprint" && (
          <div data-testid="lane-app-backlog-sprint" className="max-h-[480px] overflow-y-auto">
            <SprintBoardView onSelect={(id) => setDrawer({ kind: "view", workItemId: id })} />
          </div>
        )}

        {active === "canvas" && (
          <div data-testid="lane-app-canvas" className="space-y-2">
            <a
              href={`/canvas/canvas-001?worktree=${encodeURIComponent(worktreeId)}`}
              className="flex items-center gap-2 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2.5 text-xs font-semibold text-accent hover:bg-accent/20 transition-colors"
            >
              <Frame size={14} /> 打开 Lane 画布 (白板 / Agent 画布 / 关系图) <ArrowRight size={12} className="ml-auto" />
            </a>
            <a
              href="/canvas/worktree-graph"
              className="flex items-center gap-2 rounded-lg border border-line bg-bg-soft/60 px-3 py-2.5 text-xs text-ink hover:border-accent/50 transition-colors"
            >
              <GitBranch size={14} className="text-ok" /> Worktree 关系图 · 5 视图 <ArrowRight size={12} className="ml-auto" />
            </a>
            <p className="text-[11px] text-ink-mute">
              Agent 中心功能已并入画布界面 — Agent 节点/拓扑在「Agent 画布」标签内统一管理。
            </p>
          </div>
        )}

        {active === "workflow" && (
          <div data-testid="lane-app-workflow" className="space-y-1.5">
            <a
              href="/workflow"
              className="flex items-center gap-2 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2.5 text-xs font-semibold text-accent hover:bg-accent/20 transition-colors"
            >
              <Workflow size={14} /> 低代码流程编排 (LangGraph 状态机) <ArrowRight size={12} className="ml-auto" />
            </a>
            {workflows.length === 0 ? (
              <p className="text-xs text-ink-mute">{t.branchWorkspace.empty.workflow}</p>
            ) : (
              <ul className="space-y-1">
                {workflows.slice(0, 6).map((wf) => (
                  <li key={wf.id} className="flex items-center gap-2 rounded-lg border border-line bg-bg-soft/50 px-3 py-2 text-xs">
                    <Workflow size={12} className="text-ink-mute shrink-0" />
                    <span className="truncate flex-1 text-ink">{wf.name}</span>
                    <span className="font-mono text-[10px] text-ink-mute">{(wf as { states?: unknown[] }).states ? "有状态流" : "定义"}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {active === "bi" && (
          <div data-testid="lane-app-bi" className="space-y-2">
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
              <div className="rounded-lg border border-line bg-bg-soft/50 p-2.5 text-center">
                <div className="text-lg font-bold text-ink">{stats.total}</div>
                <div className="text-[10px] text-ink-mute">任务总数</div>
              </div>
              <div className="rounded-lg border border-line bg-bg-soft/50 p-2.5 text-center">
                <div className="text-lg font-bold text-ok">{stats.done}</div>
                <div className="text-[10px] text-ink-mute">已完成</div>
              </div>
              <div className="rounded-lg border border-line bg-bg-soft/50 p-2.5 text-center">
                <div className="text-lg font-bold text-accent">{stats.throughput}</div>
                <div className="text-[10px] text-ink-mute">7 日吞吐</div>
              </div>
              <div className="rounded-lg border border-line bg-bg-soft/50 p-2.5 text-center">
                <div className="text-lg font-bold text-warn">{sprints.filter((s) => (s.status ?? "planning") === "active").length}</div>
                <div className="text-[10px] text-ink-mute">进行中 Sprint</div>
              </div>
            </div>
            {stats.total > 0 && (
              <div className="rounded-lg border border-line bg-bg-soft/50 p-2.5">
                <div className="mb-1.5 text-[10px] text-ink-mute">状态分布</div>
                <div className="flex gap-1.5 flex-wrap">
                  {[...stats.byStatus.entries()].map(([status, count]) => (
                    <span key={status} className="text-[10px] font-mono px-1.5 py-0.5 rounded border border-line bg-bg text-ink-dim">
                      {status} ×{count}
                    </span>
                  ))}
                </div>
              </div>
            )}
            <p className="text-[10px] text-ink-mute">
              数据源: {worktreeId ? `worktree ${worktreeId}` : `project ${projectId}`} · 宿主 BI 面板就绪后切换服务端指标
            </p>
          </div>
        )}

        {active === "plugins" && (
          <div data-testid="lane-app-plugins" className="space-y-1.5">
            {integrations.length === 0 ? (
              <p className="text-xs text-ink-mute">{t.branchWorkspace.empty.plugins}</p>
            ) : (
              <ul className="space-y-1">
                {integrations.slice(0, 8).map((it) => (
                  <li key={it.id} className="flex items-center gap-2 rounded-lg border border-line bg-bg-soft/50 px-3 py-2 text-xs">
                    <Puzzle size={12} className="text-ink-mute shrink-0" />
                    <span className="truncate flex-1 text-ink">{it.display_name}</span>
                    <span className="font-mono text-[10px] text-ink-mute">{it.status ?? ""}</span>
                  </li>
                ))}
              </ul>
            )}
            <a href="/settings?tab=integrations" className="flex items-center gap-2 rounded-lg border border-line bg-bg-soft/60 px-3 py-2.5 text-xs text-ink hover:border-accent/50 transition-colors">
              <Puzzle size={14} /> 集成管理 / Webhooks <ArrowRight size={12} className="ml-auto" />
            </a>
          </div>
        )}
      </div>

      {/* 可关闭任务详情子窗口 (per 需求二.4) */}
      <WorkItemDetailDrawer
        open={drawer}
        onClose={() => setDrawer(null)}
        projectId={projectId}
        tenantId={worktree?.tenant_id}
        reporterId="usr-002"
      />
    </section>
  );
}
