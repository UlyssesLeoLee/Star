/*
@cypher schema=1 source_sha256=fb3e47bbcd858fc8d23cd4bab1393c44a5d30ea1e1249858b509b0df4de16f00
MERGE (self:File {path:"frontend/src/app/(app)/kanban/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/kanban/page.tsx::KanbanPage",kind:"component"})
MERGE (ideFileWorkspace:File {path:"frontend/src/components/ide/IdeFileWorkspace.tsx"})
MERGE (ideFileWorkspaceComponent:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::IdeFileWorkspace",kind:"component"})
MERGE (nema_store:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (open_nema:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::openNemaAsStarTask",kind:"function"})
MERGE (groupProjection:File {path:"frontend/src/lib/group/groupProjection.tsx"})
MERGE (groupApiHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupApi",kind:"function"})
MERGE (sessionStateHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupSessionState",kind:"function"})
MERGE (kanbanFilters:File {path:"frontend/src/components/board/KanbanFilters.tsx"})
MERGE (applyBoardFilters:Symbol {id:"frontend/src/components/board/KanbanFilters.tsx::applyBoardFilters",kind:"function"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(nema_store)
MERGE (self)-[:IMPORTS]->(groupProjection)
MERGE (self)-[:IMPORTS]->(kanbanFilters)
MERGE (self)-[:IMPORTS]->(ideFileWorkspace)
MERGE (page)-[:CALLS]->(open_nema)
MERGE (page)-[:CALLS]->(groupApiHook)
MERGE (page)-[:CALLS]->(sessionStateHook)
MERGE (page)-[:CALLS]->(applyBoardFilters)
MERGE (page)-[:CALLS]->(ideFileWorkspaceComponent)
MERGE (unified:File {path:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts"})
MERGE (self)-[:IMPORTS]->(unified)
MERGE (listUnified:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::listUnifiedWorkItems",kind:"function"})
MERGE (createUnified:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::createUnifiedWorkItem",kind:"function"})
MERGE (moveUnified:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::moveUnifiedWorkItem",kind:"function"})
MERGE (page)-[:CALLS]->(listUnified)
MERGE (page)-[:CALLS]->(createUnified)
MERGE (page)-[:CALLS]->(moveUnified)
MERGE (serverTransition:Symbol {id:"frontend/src/app/(app)/kanban/page.tsx::handleServerTransition",kind:"function"})
MERGE (serverTransition)-[:CALLS]->(moveUnified)
MERGE (page)-[:CALLS]->(serverTransition)
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
import { toast } from "react-hot-toast";
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
import { KanbanBoard } from "@/components/board/KanbanBoard";
import { KanbanFilters, applyBoardFilters, type KanbanFiltersValue } from "@/components/board/KanbanFilters";
import { WorkItemDetailDrawer, type WorkItemDrawerMode } from "@/components/board/WorkItemDetailDrawer";
import { IdeFileWorkspace } from "@/components/ide/IdeFileWorkspace";
import { CalendarView } from "@/components/calendar/CalendarView";
import { IpaSkillWorkflowBoard } from "@/components/ipa-skills/IpaSkillWorkflowBoard";
import { useNemaStore } from "@/lib/nema/nemaStore";
import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { useWorktreeGroupApi, useWorktreeGroupSessionState } from "@/lib/group/groupProjection";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import type { Board, WorkItem } from "@/types/ids";
import {
  createUnifiedWorkItem,
  listUnifiedWorkItems,
  moveUnifiedWorkItem,
  toStorePriority,
  toUnifiedError,
  UNIFIED_COLUMN_LABELS,
  UNIFIED_COLUMN_STATUS,
  UNIFIED_KANBAN_COLUMNS,
  type UnifiedKanbanColumn,
  type UnifiedWorkItem,
} from "./unifiedWorkItems";

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

  // ---- 宿主数据通道信号 (per 2026-11-16 清 seed/mock 显示入口) ----
  // Group API 会话存在 = 宿主已接入; 网页预览模式为 null → 任务看板不渲染
  // 本地 seed 任务卡, 创建/移动等写操作禁用; 401 信号来自
  // useWorktreeGroupSessionState() (status === "invalidated")。
  const groupApi = useWorktreeGroupApi();
  const groupSession = useWorktreeGroupSessionState();
  const unauthorized = groupSession.status === "invalidated";
  const hostConnected = groupApi !== null;

  // ---- 服务端真源通道 (per P1: Backlog/Sprint/Kanban 共享同一任务 ID/状态规则/写入路径) ----
  // serverMode = 宿主会话 + 当前 Worktree 上下文 (navStore.selectedWorktreeId) 齐备;
  // 缺其一即退化为「本地模式」并明确标注, 不冒充服务端数据。
  const runDirectory = useRunDirectory();
  const selectedWorktreeId = useNavStore((s) => s.selectedWorktreeId);
  const [serverItems, setServerItems] = useState<UnifiedWorkItem[] | null>(null);
  const [serverError, setServerError] = useState<string | null>(null);
  const [serverRefreshKey, setServerRefreshKey] = useState(0);
  const serverMode = hostConnected && runDirectory.client !== null && selectedWorktreeId !== "";
  const createDisabled = unauthorized || !hostConnected;

  // 每次进入页面/上下文切换都重新 fetch 服务端真源 (不依赖 persist 旧数据, per P1 刷新一致)
  useEffect(() => {
    if (!serverMode || !groupApi) return;
    const controller = new AbortController();
    setServerError(null);
    listUnifiedWorkItems(groupApi, selectedWorktreeId, controller.signal)
      .then((items) => {
        setServerItems(items);
        syncStoreCache(items);
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) return;
        const unified = toUnifiedError(error);
        setServerError(unified.message);
        setServerItems(null);
      });
    return () => controller.abort();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [serverMode, groupApi, selectedWorktreeId, serverRefreshKey]);

  // store 只读缓存水合 (per P1 复用清单: WorkItemDetailDrawer 读 store.workItems, 不可改共享文件;
  // 此处用既有 addWorkItem/transitionWorkItem 幂等水合, 供详情抽屉展示服务端真源; 写操作一律不走 store)
  const syncStoreCache = (items: UnifiedWorkItem[]) => {
    for (const item of items) {
      const current = useStore.getState().workItems.find((w) => w.id === item.id);
      if (!current) {
        addWorkItem(toStoreWorkItem(item));
      } else if (current.status !== item.column) {
        transitionWorkItem(item.id, item.column);
      }
    }
  };

  // 服务端真源 → store 形状 (仅缓存水合/看板渲染; 状态/版本仍以服务端为准)
  const toStoreWorkItem = (item: UnifiedWorkItem): WorkItem => ({
    id: item.id,
    tenant_id: project?.tenant_id ?? "00000000-0000-0000-0000-000000000001",
    project_id: projectId || "00000000-0000-0000-0000-000000000000",
    key: item.id,
    title: item.title,
    description: item.description,
    kind: item.kind as WorkItem["kind"],
    status: item.column,
    priority: toStorePriority(item.priority),
    reporter_id: item.reporterId || "usr-002",
    labels: item.labels,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    worktree_id: item.worktreeId,
  });

  const handleCreateTaskCard = async () => {
    const title = draftTitle.trim();
    if (!title) return;
    // 未接入/未授权: 禁止写本地 seed (per 2026-11-16, 本地 seed 不冒充正式任务)
    if (createDisabled) return;
    if (serverMode && runDirectory.client) {
      // 服务端真源写入: 创建挂当前 worktree (branch) — 与 Backlog/Sprint 入口同一写入路径
      try {
        const runContext =
          runDirectory.focus ?? (await runDirectory.client.getWorktreeRunContext(selectedWorktreeId));
        await createUnifiedWorkItem(runDirectory.client, runContext, selectedWorktreeId, {
          title,
          description: draftFile && draftFile !== "无" ? `关联文件: ${draftFile}` : "",
          priority: "medium",
          idempotencyKey: `kanban-create-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        });
        setDraftTitle("");
        setDraftIpaFlow("ipa-s05-002 ソースコード作成");
        setDraftWorkflow("");
        setDraftFile("");
        setDraftPoints("3");
        setServerRefreshKey((k) => k + 1);
        toast.success("任务卡已创建 (服务端真源)");
      } catch (error) {
        const unified = toUnifiedError(error);
        if (unified.isForbidden()) {
          toast.error("越权写入被拒绝 — 当前会话没有创建任务卡的授权");
        } else {
          toast.error(`创建失败: ${unified.message}`);
        }
      }
      return;
    }
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

  // 服务端移动 (kanban/sprint 同一 moveUnifiedWorkItem 写入路径; 乐观更新 + 失败回滚 + 冲突反馈)
  const handleServerTransition = async (workItemId: string, toStatus: WorkItem["status"]) => {
    if (!groupApi || !serverMode || !serverItems) return;
    if (!(UNIFIED_KANBAN_COLUMNS as readonly string[]).includes(toStatus)) {
      toast.error("该列没有服务端状态语义 — 评审请走评审流程 (review 状态由服务端 review 流程驱动)");
      return;
    }
    const target = toStatus as UnifiedKanbanColumn;
    const item = serverItems.find((w) => w.id === workItemId);
    if (!item) return;
    // 乐观更新
    setServerItems((cur) =>
      cur
        ? cur.map((w) =>
            w.id === workItemId
              ? { ...w, column: target, lifecycleStatus: UNIFIED_COLUMN_STATUS[target] }
              : w,
          )
        : null,
    );
    try {
      const updated = await moveUnifiedWorkItem(
        groupApi,
        selectedWorktreeId,
        item,
        target,
        `kanban-move-${workItemId}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      );
      setServerItems((cur) => (cur ? cur.map((w) => (w.id === updated.id ? updated : w)) : null));
      transitionWorkItem(updated.id, updated.column);
    } catch (error) {
      const unified = toUnifiedError(error);
      if (unified.isConflict()) {
        toast.error("版本冲突, 已刷新最新状态");
      } else if (unified.isForbidden()) {
        toast.error("越权写入被拒绝 — 当前会话没有移动任务卡的授权");
      } else {
        toast.error(`移动失败: ${unified.message}`);
      }
      // 失败回滚: 重新拉取服务端真源 (乐观更新作废)
      setServerRefreshKey((k) => k + 1);
    }
  };

  // 列内创建 Drawer + 列操作 (per 2026-11-15 用户发令: 每列创建按钮/移动列/增减列 — 沿用原有 kanban 功能)
  const [drawerMode, setDrawerMode] = useState<WorkItemDrawerMode | null>(null);
  const addBoardColumn = useStore((s) => s.addBoardColumn);
  const removeBoardColumn = useStore((s) => s.removeBoardColumn);
  const renameBoardColumn = useStore((s) => s.renameBoardColumn);
  const reorderBoardColumns = useStore((s) => s.reorderBoardColumns);

  // per 2026-11-17 去重: Sprint 视图收敛到 /sprint (同一 SprintBoardView 不渲染两处),
  // 本页默认视图改为"任务看板"; Sprint tab 变为直达入口卡 (沿用"时间线→/analytics"入口卡模式)。
  const [activeView, setActiveView] = useState<ViewTab>("任务看板");

  // 任务看板筛选 (per 2026-11-17 功能补齐: 复用 KanbanFilters 孤儿组件, kind/assignee/priority)
  const [boardFilter, setBoardFilter] = useState<KanbanFiltersValue>({
    kind: "all",
    assignee_id: "all",
    priority: "all",
  });

  // 深链 ?view= + ?work_item= (client-only 解析; 不用 useSearchParams — dev 下与页面级
  // Suspense 组合会产生双份 DOM 渲染)
  // ?work_item=<id> = Lane 任务卡「打开详情」深链: 服务端真源水合 store 后开 Drawer;
  // 未水合 (本地模式/拉取失败/无此卡) 时不开空 Drawer, 诚实留痕。
  useEffect(() => {
    if (typeof window === "undefined") return;
    const params = new URLSearchParams(window.location.search);
    const v = params.get("view");
    if (isViewTab(v)) setActiveView(v as ViewTab);
    const wi = params.get("work_item");
    if (wi && useStore.getState().workItems.some((w) => w.id === wi)) {
      setDrawerMode({ kind: "view", workItemId: wi });
    }
  }, [serverItems]);

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

  // ---- 任务看板数据 (per 2026-11-17 去重合并): 从 /projects?tab=kanban 合并进来的
  //      project 范围语义 — 选中项目时按 project_id 过滤, 未选中时保持全局旧行为。
  const boardWorkItems = useMemo(
    () => (selectedProjectId ? workItems.filter((w) => w.project_id === selectedProjectId) : workItems),
    [workItems, selectedProjectId],
  );
  const filteredBoardWorkItems = useMemo(
    () => applyBoardFilters(boardWorkItems, boardFilter),
    [boardWorkItems, boardFilter],
  );

  // ---- nema 文件即任务 (真实 store) ----
  const nemaTasks = useNemaStore((s) => s.nemaTasks);
  const openNemaAsStarTask = useNemaStore((s) => s.openNemaAsStarTask);

  // ---- 列定义 (per P1): 服务端无 board 列配置, 列闭集来自统一域常量
  //      UNIFIED_KANBAN_COLUMNS (镜像 group_api `unified_column_status`, 与
  //      groupProjection 读侧映射互逆); 不再依赖 mocks/data/kanban。 ----
  const lifecycleCols = serverMode
    ? UNIFIED_KANBAN_COLUMNS.map((id) => ({ id, label: UNIFIED_COLUMN_LABELS[id]["zh-CN"] }))
    : board?.columns && board.columns.length > 0
      ? board.columns.map((c) => ({ id: c.status, label: LIFECYCLE_LABEL[c.status] ?? c.name ?? c.status }))
      : UNIFIED_KANBAN_COLUMNS.map((id) => ({ id, label: LIFECYCLE_LABEL[id] ?? id }));

  // 服务端模式的看板投影 (列 = 统一闭集; work_item_ids 从服务端真源派生, 非本地事实)
  const unifiedBoard = useMemo<Board>(() => ({
    id: "00000000-0000-0000-0000-000000000000",
    tenant_id: project?.tenant_id ?? "00000000-0000-0000-0000-000000000001",
    project_id: projectId || "00000000-0000-0000-0000-000000000000",
    name: "统一服务端看板 (worktree 真源)",
    columns: UNIFIED_KANBAN_COLUMNS.map((status) => ({
      status,
      name: UNIFIED_COLUMN_LABELS[status]["zh-CN"],
      work_item_ids: (serverItems ?? []).filter((w) => w.column === status).map((w) => w.id),
    })),
  }), [serverItems, projectId, project?.tenant_id]);

  // 服务端模式看板卡片 (WorkItem 形状仅用于 KanbanBoard 渲染/筛选; 状态以服务端为准)
  const serverBoardItems = useMemo(
    () => (serverItems ?? []).map((item) => toStoreWorkItem(item)),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [serverItems, projectId, project?.tenant_id],
  );
  const filteredServerBoardItems = useMemo(
    () => applyBoardFilters(serverBoardItems, boardFilter),
    [serverBoardItems, boardFilter],
  );

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

        {/* 数据来源标识 (per P1: 服务端真源 / 本地模式 明确区分, 不冒充) */}
        <div
          data-testid="kanban-data-source"
          className={clsx(
            "flex items-center gap-2 rounded-lg border px-3 py-2 text-[11px]",
            serverMode
              ? "border-ok/40 bg-ok/10 text-ok"
              : "border-warn/40 bg-warn/10 text-warn",
          )}
        >
          <span className={clsx("size-1.5 rounded-full", serverMode ? "bg-ok" : "bg-warn")} />
          {serverMode ? (
            <span>
              服务端真源模式 — 任务卡来自授权服务端投影 (worktree {selectedWorktreeId}), 每次进入页面重新拉取; 创建/移动/状态转换与 Backlog·Sprint 共享同一任务 ID 与写入路径。
            </span>
          ) : (
            <span>
              本地模式 — 当前看板展示本地状态, 非授权服务端任务事实; 宿主会话与 Worktree 上下文接入后自动切换为服务端真源。
            </span>
          )}
        </div>
        {serverMode && serverError && (
          <div
            data-testid="kanban-server-error"
            className="flex items-center gap-2 rounded-lg border border-err/40 bg-err/10 px-3 py-2 text-[11px] text-err"
          >
            <span className="size-1.5 rounded-full bg-err" />
            服务端数据加载失败: {serverError}
          </div>
        )}

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
            disabled={createDisabled}
            className="h-8 flex-1 min-w-[180px] rounded-lg border border-line bg-bg-soft px-2.5 text-xs focus:border-accent focus:outline-none disabled:opacity-50"
          />
          <select
            value={draftIpaFlow}
            onChange={(e) => setDraftIpaFlow(e.target.value)}
            aria-label="IPA 流程"
            data-testid="kanban-create-ipa"
            disabled={createDisabled}
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs disabled:opacity-50"
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
            disabled={createDisabled}
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs disabled:opacity-50"
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
            disabled={createDisabled}
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs disabled:opacity-50"
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
            disabled={createDisabled}
            className="h-8 rounded-lg border border-line bg-bg-soft px-2 text-xs disabled:opacity-50"
          >
            {["1", "3", "5", "8"].map((p) => (
              <option key={p} value={p}>{p} 点</option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => void handleCreateTaskCard()}
            disabled={createDisabled || !draftTitle.trim()}
            data-testid="kanban-create-submit"
            className="h-8 px-4 rounded-lg text-white text-xs font-semibold hover:bg-accent-soft disabled:opacity-50 flex items-center gap-1"
            style={{ backgroundColor: "var(--accent-DEFAULT, #1D4ED8)" }}
          >
            开卡
          </button>
        </div>

        {/* 创建/移动禁用诚实说明 (per 2026-11-16 清 seed/mock: 无授权数据不写入本地 seed) */}
        {createDisabled && (
          <div
            data-testid="kanban-create-disabled-hint"
            className="flex items-center gap-1.5 text-[11px] text-warn"
          >
            <span className="size-1.5 rounded-full bg-warn" />
            未接入授权会话 — 创建与移动操作已禁用 (本地 seed 数据不作为任务事实写入)。宿主会话接入后自动恢复。
          </div>
        )}

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
            {/* per 2026-11-17 去重: SprintBoardView 收敛到 /sprint 独立模块
                (同一组件 + 同一数据 + 同一组操作不在两处渲染), 本 tab 变为直达入口卡。
                沿用"时间线→/analytics"入口卡模式, 不另起炉灶。 */}
            <div className="flex h-full min-h-[220px] flex-col items-center justify-center gap-2 text-center">
              <p className="text-xs font-semibold text-ink">Sprint 规划已收敛到 Sprint 主面板</p>
              <p className="text-[11px] text-ink-dim">
                Backlog 拖入 / 创建 / 启动 / 完成 / 删除 Sprint 与
                List · Tree 视图现在统一在
                <a
                  href="/sprint?view=sprint"
                  className="mx-1 font-semibold text-accent underline underline-offset-2"
                  data-testid="kanban-sprint-open-issues"
                >
                  Sprint 主面板
                </a>
                打开 (per 2026-11-17 去重)
              </p>
            </div>
          </section>
        )}

        {activeView === "任务看板" && (
          <section data-testid="kanban-board-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            {/* 筛选 (per 2026-11-17 功能补齐: KanbanFilters 原为无接线孤儿组件, 此处接入) */}
            <KanbanFilters
              value={boardFilter}
              onChange={setBoardFilter}
              assignees={identities}
              total={serverMode ? serverBoardItems.length : boardWorkItems.length}
              shown={serverMode ? filteredServerBoardItems.length : filteredBoardWorkItems.length}
            />
            <KanbanBoard
              board={serverMode ? unifiedBoard : board}
              workItems={serverMode ? filteredServerBoardItems : filteredBoardWorkItems}
              identities={identities}
              onTransition={serverMode ? (id, to) => void handleServerTransition(id, to) : transitionWorkItem}
              onWorkItemClick={(wi) => setDrawerMode({ kind: "view", workItemId: wi.id })}
              // 服务端模式: 列编辑与列内新建是本地 board 语义 (服务端无列配置),
              // 不渲染以免产生与真源分裂的本地写 (诚实隐藏); 创建走顶部统一创建栏
              onRequestNewWorkItem={
                serverMode ? undefined : (status) => setDrawerMode({ kind: "new", defaultStatus: status })
              }
              onAddColumn={serverMode ? undefined : addBoardColumn}
              onRemoveColumn={serverMode ? undefined : removeBoardColumn}
              onRenameColumn={serverMode ? undefined : renameBoardColumn}
              onReorderColumns={serverMode ? undefined : reorderBoardColumns}
              // 诚实态 (per 2026-11-16 清 seed/mock): 未授权/未接入/空数据不渲染本地 seed 任务卡
              unavailableState={
                unauthorized
                  ? "unauthorized"
                  : !hostConnected
                    ? "disconnected"
                    : serverMode && serverError
                      ? "empty"
                      : serverMode && (serverItems ?? []).length === 0
                        ? "empty"
                        : null
              }
            />
          </section>
        )}

        {activeView === "IPA 流程图" && (
          <section data-testid="kanban-ipa-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            <IpaSkillWorkflowBoard
              projectId={projectId}
              tenantId={project?.tenant_id ?? "00000000-0000-0000-0000-000000000001"}
              reporterId={project?.owner_id}
              onOpenTaskCard={(workItemId) => setDrawerMode({ kind: "view", workItemId })}
            />
          </section>
        )}

        {activeView === "按文件开卡" && (
          <section data-testid="kanban-file-cards" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            {/* IDE1.0 编辑内核工作区 (per 2026-11-15: 文件编辑相关页面体现 IDE1.0 功能) */}
            <div className="mb-4" data-testid="kanban-ide-workspace">
              <IdeFileWorkspace
                probeKernel
                files={nemaTasks.map((task) => ({
                id: task.id, filePath: task.filePath, editors: task.editors, workItemId: task.workItemId,
              }))} />
            </div>
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
                        onClick={async () => {
                          const wid = await openNemaAsStarTask(task.id);
                          if (wid) setDrawerMode({ kind: "view", workItemId: wid });
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
          <section data-testid="kanban-gantt-view" className="bg-bg-card rounded-xl border border-line shadow-soft p-4">
            <div className="flex h-full flex-col items-center justify-center gap-2 text-center">
              <p className="text-xs font-semibold text-ink">甘特图已合并进分析界面</p>
              <p className="text-[11px] text-ink-dim">
                排期甘特图现在
                <a href="/analytics" className="mx-1 font-semibold text-accent underline underline-offset-2" data-testid="kanban-gantt-open-analytics">
                  分析界面「甘特图」标签
                </a>
                打开 (per 2026-11-15)
              </p>
            </div>
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
        onOpenChildTask={(childId) => setDrawerMode({ kind: "view", workItemId: childId })}
      />
    </div>
  );
}
