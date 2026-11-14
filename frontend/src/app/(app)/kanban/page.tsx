/*
@cypher schema=1 source_sha256=b5f2dfafa636a1a6b0d1188de8e51dae72491e6d07f06fe4780b37f34a9b4d9d
MERGE (self:File {path:"frontend/src/app/(app)/kanban/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/kanban/page.tsx::KanbanPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /kanban — 1b · 任务 · IPA 看板
// 布局 per 2026-11-14 参考 mockup (TasksIpa.dc.html):
//   面包屑 + 6 视图 tabs + 生命周期条 + 右侧 rail
// 内容 = 原有真实功能 (per 用户发令: 功能其实都是原来那些, 不摆静态 mock):
//   Sprint → SprintBoardView (真实 sprints/transition)
//   任务看板 → KanbanBoard (真实拖拽 + transitionWorkItem)
//   IPA 流程图 → IpaSkillWorkflowBoard (真实 IPA 技能工作流)
//   按文件开卡 → nema 文件即任务 (真实 nemaStore + openNemaAsStarTask)
//   时间线/日历 → 真实实现在 /projects?tab=timeline, 提供直达入口
// =====================================================================
"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import {
  ChevronDown,
  GitBranch,
  FileText,
  MessageSquare,
  Users,
  GitCommitHorizontal,
  CalendarDays,
  ArrowRight,
  FileCode2,
} from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { SprintBoardView } from "@/components/sprint/SprintBoardView";
import { KanbanBoard } from "@/components/board/KanbanBoard";
import { IpaSkillWorkflowBoard } from "@/components/ipa-skills/IpaSkillWorkflowBoard";
import { useNemaStore } from "@/lib/nema/nemaStore";
import { KANBAN_COLUMNS } from "@/mocks/data/kanban";
import { GuardedLink as Link } from "@/components/navigation/GuardedLink";

const VIEW_TABS = ["Sprint", "任务看板", "IPA 流程图", "按文件开卡", "时间线", "日历"] as const;
type ViewTab = (typeof VIEW_TABS)[number];

// mockup 生命周期条 6 阶段 ← 真实 KANBAN_COLUMNS (todo/in_progress/review/blocked/done/wontfix)
const LIFECYCLE_LABEL: Record<string, string> = {
  todo: "Backlog",
  in_progress: "进行中",
  review: "评审",
  blocked: "阻塞",
  done: "完成",
  wontfix: "归档",
};

const isViewTab = (v: string | null): v is ViewTab =>
  VIEW_TABS.includes(v as ViewTab);

export default function KanbanPage() {
  const router = useRouter();
  const [activeView, setActiveView] = useState<ViewTab>("Sprint");

  // 深链 ?view= (client-only 解析; 不用 useSearchParams — dev 下与页面级
  // Suspense 组合会产生双份 DOM 渲染)
  useEffect(() => {
    if (typeof window === "undefined") return;
    const v = new URLSearchParams(window.location.search).get("view");
    if (isViewTab(v)) setActiveView(v as ViewTab);
  }, []);

  // ---- 真实数据源 (store) ----
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const board = useStore((s) => s.board);
  const transitionWorkItem = useStore((s) => s.transitionWorkItem);
  const projects = useStore((s) => s.projects);
  const selectedProjectId = useNavStore((s) => s.selectedProjectId);
  const projectId = selectedProjectId || projects[0]?.id || "";
  const project = projects.find((p) => p.id === projectId) ?? projects[0];

  // ---- nema 文件即任务 (真实 store) ----
  const nemaTasks = useNemaStore((s) => s.nemaTasks);
  const openNemaAsStarTask = useNemaStore((s) => s.openNemaAsStarTask);

  const lifecycleCols = board?.columns && board.columns.length > 0
    ? board.columns.map((c) => ({ id: c.status, label: LIFECYCLE_LABEL[c.status] ?? c.name ?? c.status }))
    : KANBAN_COLUMNS.map((id) => ({ id, label: LIFECYCLE_LABEL[id] ?? id }));

  return (
    <div className="flex h-full bg-bg">
      {/* ============ 主区 ============ */}
      <main className="flex-1 min-w-0 overflow-y-auto p-4 space-y-4">
        {/* 面包屑: Project → LANE → Worktree → Branch */}
        <nav data-testid="kanban-breadcrumb" className="flex items-center gap-1.5 text-xs text-ink-dim flex-wrap">
          <button type="button" className="flex items-center gap-1 hover:text-ink">
            <span className="font-medium">{project?.name ?? "Physis / GVPE"}</span> <ChevronDown size={12} />
          </button>
          <span className="text-ink-mute">▸</span>
          <button type="button" className="flex items-center gap-1 hover:text-ink">
            LANE · auth-refactor <ChevronDown size={12} />
          </button>
          <span className="text-ink-mute">▸</span>
          <button type="button" className="flex items-center gap-1 hover:text-ink">
            wt-relation-bfs <ChevronDown size={12} />
          </button>
          <button type="button" className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-line bg-bg-card font-mono hover:border-line">
            <GitBranch size={11} /> feat/relation-bfs
          </button>
          <button type="button" className="flex items-center gap-1 px-1.5 py-0.5 rounded border border-line bg-bg-card font-mono hover:border-line">
            <GitBranch size={11} /> fix/bfs-cycle
          </button>
        </nav>

        {/* 视图 tabs (?view= 深链) */}
        <div data-testid="kanban-view-tabs" className="flex items-center gap-1 border-b border-line">
          {VIEW_TABS.map((tab) => (
            <button
              key={tab}
              type="button"
              onClick={() => setActiveView(tab)}
              data-testid={`kanban-view-tab-${tab}`}
              className={clsx(
                "px-3 py-2 text-sm border-b-2 -mb-px transition-colors",
                activeView === tab
                  ? "border-accent text-accent font-semibold"
                  : "border-transparent text-ink-dim hover:text-ink",
              )}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* 生命周期条 (真实 board.columns) */}
        <div data-testid="kanban-lifecycle" className="flex items-center gap-2 text-[11px] text-ink-dim flex-wrap">
          {lifecycleCols.map((stage, i) => (
            <span key={stage.id} className="flex items-center gap-2">
              <span className={clsx("px-2 py-0.5 rounded-full border", i === 1 ? "bg-accent/10 border-accent/50 text-accent font-semibold" : "border-line bg-bg-card")}>
                {stage.label}
              </span>
              {i < lifecycleCols.length - 1 && <span className="text-ink-mute">→</span>}
            </span>
          ))}
        </div>

        {/* ============ 视图内容 = 原有真实功能 ============ */}
        {activeView === "Sprint" && (
          <section data-testid="kanban-sprint-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            <SprintBoardView />
          </section>
        )}

        {activeView === "任务看板" && (
          <section data-testid="kanban-board-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            <KanbanBoard
              board={board}
              workItems={workItems}
              identities={identities}
              onTransition={transitionWorkItem}
              onWorkItemClick={(id) => router.push(`/work-item/${id}`)}
            />
          </section>
        )}

        {activeView === "IPA 流程图" && (
          <section data-testid="kanban-ipa-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            <IpaSkillWorkflowBoard
              projectId={projectId}
              tenantId={project?.tenant_id ?? "00000000-0000-0000-0000-000000000001"}
              reporterId={project?.owner_id}
              onOpenTaskCard={(workItemId) => router.push(`/work-item/${workItemId}`)}
            />
          </section>
        )}

        {activeView === "按文件开卡" && (
          <section data-testid="kanban-file-cards" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-sm font-bold text-ink">文件即任务 · .nema 侧车</h2>
              <Link href="/nema" className="text-xs text-accent hover:underline flex items-center gap-1">
                协同页管理 <ArrowRight size={11} />
              </Link>
            </div>
            {nemaTasks.length === 0 ? (
              <p className="text-xs text-ink-dim">暂无 .nema 任务 — 在 /nema 关联共享文件后会自动生成侧车文件，可在此直接开卡。</p>
            ) : (
              <ul className="space-y-1.5">
                {nemaTasks.map((task) => (
                  <li key={task.id} data-testid={`kanban-nema-${task.id}`} className="flex items-center gap-3 px-3 py-2.5 rounded-lg border border-line text-xs">
                    <FileCode2 size={13} className="text-accent shrink-0" />
                    <span className="font-mono text-ink-dim flex-1 truncate">{task.filePath}</span>
                    <span className="text-[10px] text-ink-mute">{task.editors.length} 人</span>
                    {task.workItemId ? (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-ok/10 text-ok">已开卡 {task.workItemId}</span>
                    ) : (
                      <button
                        type="button"
                        onClick={() => {
                          const wid = openNemaAsStarTask(task.id);
                          if (wid) router.push(`/work-item/${wid}`);
                        }}
                        className="text-[11px] px-2 py-1 rounded-lg bg-accent text-white hover:bg-accent-soft flex items-center gap-1"
                      >
                        <FileText size={11} /> 打开为任务
                      </button>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>
        )}

        {(activeView === "时间线" || activeView === "日历") && (
          <section data-testid="kanban-timeline-link" className="bg-bg-card rounded-xl border border-line shadow-soft p-6 text-center">
            <CalendarDays size={20} className="mx-auto text-accent mb-2" />
            <h2 className="text-sm font-bold text-ink">
              {activeView === "时间线" ? "甘特图 / 时间线" : "日历（月 / 周视图）"}
            </h2>
            <p className="text-xs text-ink-dim mt-1">
              完整实现（里程碑拖拽、WorkItem 移动、事件拖动）在项目页时间线 tab。
            </p>
            <Link
              href="/projects?tab=timeline"
              data-testid="kanban-go-timeline"
              className="inline-flex items-center gap-1.5 mt-3 text-xs px-3 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft"
            >
              打开项目时间线 <ArrowRight size={12} />
            </Link>
          </section>
        )}
      </main>

      {/* ============ 右侧 rail ============ */}
      <aside data-testid="kanban-right-rail" className="w-72 shrink-0 border-l border-line bg-bg-card flex flex-col">
        <div className="px-3 py-2.5 border-b border-line flex items-center gap-2 text-xs font-semibold text-ink-dim">
          <GitCommitHorizontal size={13} className="text-ok" />
          Worktree 状态 · <span className="text-ok font-mono">干净</span>
        </div>

        <div className="flex-1 overflow-y-auto p-3 space-y-3">
          <div className="flex items-center gap-2 text-xs font-semibold text-ink-dim">
            <MessageSquare size={13} className="text-accent" /> 聊天
          </div>
          <div data-testid="kanban-chat-panel" className="rounded-lg border border-line bg-bg p-3 space-y-2">
            <p className="text-xs text-ink leading-relaxed">详细设计还有哪些流程没建卡？</p>
            <p className="text-xs text-ink-dim leading-relaxed">
              P4 共 20 个流程，已建卡 3 个。建议先建 <span className="font-mono">ロジック設計</span> 和{" "}
              <span className="font-mono">DB詳細設計</span>，要我放进 Backlog 吗？
            </p>
            <button type="button" data-testid="kanban-chat-ask" className="text-[11px] px-2 py-1 rounded border border-line bg-bg-card hover:border-accent/60 text-ink-dim">
              问问这个 Branch 的进展
            </button>
          </div>

          <div className="flex items-center gap-2 text-xs font-semibold text-ink-dim">
            <Users size={13} className="text-ink-dim" /> Team
          </div>
          <div className="flex -space-x-1.5">
            {identities.slice(0, 4).map((who, i) => (
              <span
                key={who.id ?? i}
                title={who.display_name}
                className="size-7 rounded-full border-2 border-white bg-accent/15 text-accent text-[10px] font-bold grid place-items-center"
              >
                {who.display_name?.slice(0, 1) ?? "?"}
              </span>
            ))}
            <span className="size-7 rounded-full border-2 border-white bg-bg-soft text-ink-dim text-[9px] font-bold grid place-items-center">
              {identities.length} 在线
            </span>
          </div>
        </div>
      </aside>
    </div>
  );
}
