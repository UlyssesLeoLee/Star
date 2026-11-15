/*
@cypher schema=1 source_sha256=c3947a273daf44697aa4f45867a92481deb064be16c9d87942c02ff8dc02d1df
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

import { Suspense, useEffect, useMemo, useState } from "react";
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
  PenTool,
  Plus,
} from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { SprintBoardView } from "@/components/sprint/SprintBoardView";
import { KanbanBoard } from "@/components/board/KanbanBoard";
import { WorkItemDetailDrawer, type WorkItemDrawerMode } from "@/components/board/WorkItemDetailDrawer";
import { GanttView } from "@/components/gantt";
import { CalendarView } from "@/components/calendar/CalendarView";
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
  // 新任务卡创建栏 (per 1b · 任务 · IPA 看板 mockup)
  const [draftTitle, setDraftTitle] = useState("");
  // per 1b mockup 默认值: IPA 默认第一个流程, 工作流默认标准研发, 估点默认 3 点
  const [draftIpaFlow, setDraftIpaFlow] = useState("ipa-s05-002 ソースコード作成");
  const [draftWorkflow, setDraftWorkflow] = useState("");
  const [draftFile, setDraftFile] = useState("");
  const [draftPoints, setDraftPoints] = useState("3");
  const ipaFlows = useMemo(
    () => ["ipa-s05-002 ソースコード作成", "ipa-s06-ut-005 単体試験実施"],
    [],
  );
  const workflows = useStore((s) => s.workflows);
  const addWorkItem = useStore((s) => s.addWorkItem);

  const handleCreateTaskCard = async () => {
    const title = draftTitle.trim();
    if (!title) return;
    try {
      addWorkItem({
        tenant_id: "tn-001",
        project_id: "prj-physis",
        title,
        kind: "task",
        status: "todo",
        priority: "p2",
        reporter_id: "usr-002",
        story_points: Number(draftPoints) || 3,
        sprint_id: "sp-12",
        labels: [
          ...(draftIpaFlow ? [draftIpaFlow.split(" ")[0]] : []),
          ...(draftFile && draftFile !== "无" ? [`file:${draftFile}`] : []),
        ],
        workflow_id: draftWorkflow || undefined,
        description: draftFile && draftFile !== "无" ? `关联文件: ${draftFile}` : undefined,
      });
      setDraftTitle("");
      setDraftIpaFlow("ipa-s05-002 ソースコード作成");
      setDraftWorkflow("");
      setDraftFile("");
      setDraftPoints("3");
      setActiveView("IPA 流程图");
    } catch {
      /* 创建失败静默 (store 本地) */
    }
  };

  // 列内创建 Drawer + 列操作 (per 2026-11-15 用户发令: 每列创建按钮/移动列/增减列 — 沿用原有 kanban 功能)
  const [drawerMode, setDrawerMode] = useState<WorkItemDrawerMode | null>(null);
  const addBoardColumn = useStore((s) => s.addBoardColumn);
  const removeBoardColumn = useStore((s) => s.removeBoardColumn);
  const renameBoardColumn = useStore((s) => s.renameBoardColumn);
  const reorderBoardColumns = useStore((s) => s.reorderBoardColumns);

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
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
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

        {/* Branch 上下文条 (per 2026-11-15 用户发令: 不放右侧栏位, 内容进中心区域) */}
        <div data-testid="kanban-branch-context" className="flex items-center gap-2 flex-wrap text-[11px]">
          <a href="/worktree" className="flex items-center gap-1.5 rounded-full border border-ok/40 bg-ok/10 px-2.5 py-1 font-semibold text-ok hover:bg-ok/20 transition-colors">
            <GitBranch size={11} /> feat/relation-bfs · 干净
          </a>
          <a href="/canvas/canvas-001" className="flex items-center gap-1.5 rounded-full border border-accent/40 bg-accent/10 px-2.5 py-1 font-semibold text-accent hover:bg-accent/20 transition-colors">
            <PenTool size={11} /> 画布
          </a>
          <span className="flex items-center gap-1 text-ink-mute">
            <Users size={11} />
            <span className="flex -space-x-1">
              {identities.slice(0, 4).map((who, i) => (
                <span
                  key={who.id ?? i}
                  title={who.display_name}
                  className="size-5 rounded-full border border-line bg-accent/15 text-accent text-[9px] font-bold grid place-items-center"
                >
                  {who.display_name?.slice(0, 1) ?? "?"}
                </span>
              ))}
            </span>
            {identities.length} 在线
          </span>
        </div>

        {/* 数据来源标识 (per 2026-11-15 Phase 3.2: 预览/seed 数据不得冒充正式管理视图) */}
        <div data-testid="kanban-data-source" className="flex items-center gap-2 rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[11px] text-warn">
          <span className="size-1.5 rounded-full bg-warn" />
          预览数据模式 — 当前看板展示本地 seed 数据, 非授权服务端任务事实。宿主会话接入后自动切换为授权投影。
        </div>

        {/* 新任务卡创建栏 (per 1b · 任务 · IPA 看板 mockup: 标题 + IPA 流程 + 工作流) */}
        <div data-testid="kanban-create-bar" className="flex items-center gap-2 flex-wrap rounded-xl border border-line bg-bg-card p-2.5">
          <input
            value={draftTitle}
            onChange={(e) => setDraftTitle(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void handleCreateTaskCard();
            }}
            placeholder="新任务卡标题"
            aria-label="新任务卡标题"
            data-testid="kanban-create-title"
            className="h-8 flex-1 min-w-[180px] rounded-lg border border-line bg-bg-soft px-2.5 text-xs focus:border-accent focus:outline-none"
          />
          <select
            value={draftIpaFlow}
            onChange={(e) => setDraftIpaFlow(e.target.value)}
            aria-label="IPA 流程"
            data-testid="kanban-create-ipa"
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs"
          >
            <option value="">IPA：不关联 IPA</option>
            {ipaFlows.map((f) => (
              <option key={f} value={f}>IPA：{f}</option>
            ))}
          </select>
          <select
            value={draftWorkflow}
            onChange={(e) => setDraftWorkflow(e.target.value)}
            aria-label="工作流"
            data-testid="kanban-create-workflow"
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs"
          >
            <option value="">不关联工作流</option>
            {workflows.map((wf) => (
              <option key={wf.id} value={wf.id}>工作流：{wf.name}</option>
            ))}
          </select>
          <select
            value={draftFile}
            onChange={(e) => setDraftFile(e.target.value)}
            aria-label="关联文件"
            data-testid="kanban-create-file"
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs"
          >
            <option value="">文件：无</option>
            {nemaTasks.slice(0, 6).map((nt) => (
              <option key={nt.id} value={nt.filePath.split("/").pop() ?? nt.filePath}>文件：{nt.filePath.split("/").pop() ?? nt.filePath}</option>
            ))}
            <option value="branch">从 Branch 选择…</option>
          </select>
          <select
            value={draftPoints}
            onChange={(e) => setDraftPoints(e.target.value)}
            aria-label="估点"
            data-testid="kanban-create-points"
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs"
          >
            {["1", "3", "5", "8"].map((p) => (
              <option key={p} value={p}>{p} 点</option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => void handleCreateTaskCard()}
            disabled={!draftTitle.trim()}
            data-testid="kanban-create-submit"
            className="h-8 px-4 rounded-lg text-white text-xs font-semibold hover:bg-accent-soft disabled:opacity-50 flex items-center gap-1"
            style={{ backgroundColor: "var(--accent-DEFAULT, #1D4ED8)" }}
          >
            开卡
          </button>
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
            {/* Backlog 快捷建卡 (per 1b mockup: 从 IPA 流程建卡 / 按文件开卡) */}
            <div className="mb-3 flex items-center gap-2">
              <span className="text-[11px] font-semibold text-ink-dim">从下方 Backlog 把卡移进来</span>
              <span className="flex-1" />
              <button
                type="button"
                onClick={() => setActiveView("IPA 流程图")}
                data-testid="kanban-backlog-create-ipa"
                className="rounded-lg border border-accent/40 bg-accent/10 px-2.5 py-1.5 text-[11px] font-semibold text-accent hover:bg-accent/20 transition-colors"
              >
                从 IPA 流程建卡
              </button>
              <button
                type="button"
                onClick={() => setActiveView("按文件开卡")}
                data-testid="kanban-backlog-create-file"
                className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1.5 text-[11px] font-semibold text-ink hover:border-accent/50 transition-colors"
              >
                按文件开卡
              </button>
            </div>
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
              onRequestNewWorkItem={(status) => setDrawerMode({ kind: "new", defaultStatus: status })}
              onAddColumn={addBoardColumn}
              onRemoveColumn={removeBoardColumn}
              onRenameColumn={renameBoardColumn}
              onReorderColumns={reorderBoardColumns}
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

        {activeView === "时间线" && (
          <section data-testid="kanban-gantt-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4 h-full">
            <Suspense fallback={<div className="flex h-full items-center justify-center text-ink-dim">Loading Gantt...</div>}>
              <GanttView
                board={board}
                workItems={workItems}
                identities={identities}
              />
            </Suspense>
          </section>
        )}
        {activeView === "日历" && (
          <section data-testid="kanban-calendar-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4 h-full overflow-auto">
            {/* 老版本日历移植 (per 2026-11-14: 从 ProjectsClient 提取 CalendarView 合并复用) */}
            <CalendarView
              sprints={sprints}
              milestones={milestones}
              workItems={workItems}
            />
          </section>
        )}
      </main>

      {/* 列内创建 Drawer (per 2026-11-15: 每列 +Add task → Drawer 新建) */}
      <WorkItemDetailDrawer
        open={drawerMode}
        onClose={() => setDrawerMode(null)}
        projectId={projectId}
        tenantId={project?.tenant_id ?? "00000000-0000-0000-0000-000000000001"}
        reporterId={project?.owner_id ?? "usr-002"}
      />
    </div>
  );
}