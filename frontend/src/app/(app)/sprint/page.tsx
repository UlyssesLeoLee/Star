/*
@cypher schema=1 source_sha256=9f75e27a92c76acbf2ec632b4ebec2ff6dcef9738b0a786cf10c7bac01cb2622
MERGE (self:File {path:"frontend/src/app/(app)/sprint/page.tsx"})
MERGE (unified:File {path:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts"})
MERGE (self)-[:IMPORTS]->(unified)
MERGE (serverTransition:Symbol {id:"frontend/src/app/(app)/sprint/page.tsx::handleServerTransition",kind:"function"})
MERGE (serverTransition)-[:CALLS]->(:Symbol {id:"frontend/src/app/(app)/kanban/unifiedWorkItems.ts::moveUnifiedWorkItem",kind:"function"})
@endcypher
*/
"use client";

// =====================================================================
// Sprint Main Panel — /sprint (per 2026-09-05 19:13 JST 拍板: 重命名自 /issues)
// =====================================================================
// 职责 (per U2 任务 + 2026-09-05 19:13 JST 拍板):
//   1. 顶部 1 行: 3 tab 切换 (Sprint | List | Tree) — 不再用左侧 180px SubNav
//   2. 右侧 1 个 "+ New issue" button + 1 个 "🔍" 搜索 button
//   3. 中部按 view 渲染 (sprint / list / tree) — Kanban view 已删除
//   4. 右侧 (可隐藏) 320px 详情侧栏
//   5. 接 useStore (W5 zustand)
//   6. 接 ?new=true 触发 new issue 表单 (per U1 CommandBar)
//
// 设计原则 (per 守门):
//   - 不重写 store.ts (W5 维护)
//   - 不引新依赖 (复用现有 IssuesSprintView / IssuesListView / IssuesTreeView)
//   - 不动 Sidebar.tsx (向后兼容 W1-W4)
//   - 不重写 22 路由 → 6 路由 redirect (U5 负责)
//
// 已知缺口 (per 缺标比错标):
//   - 创建表单是 stub (per Phase 2+ 真正接入后端 issue create API)
//   - Tree 视图用 parent_id 推断层级, 无 relation 表 cross-link (Phase 2+ 接 relations)
//   - Sprint 视图分组用 sprint_id, 缺 sprint 嵌套 cross-link (Phase 2+)
//   - 详情侧栏 transition 按钮直接调 transitionWorkItem, 不走状态机校验 (Phase 2+ 接 store validator)
//   - 搜索功能 stub (Phase 2+ 接 U1 CommandBar)
//   - 多 column 同 sprint 视图 a11y keyboard (G3) — Phase Mobile (per §10.3 #5)
// =====================================================================

import { useState, useMemo, useCallback, Suspense, useEffect } from "react";
import Link from "next/link";
import { useRouter, useSearchParams, usePathname } from "next/navigation";
import { toast } from "react-hot-toast";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { StatusPill } from "@/components/StatusPill";
import { PageHeader } from "@/components/PageHeader";
import { SprintBoardView } from "@/components/sprint/SprintBoardView";
import { Plus, Search, X, Flag, User, Hash, Tag, FileText, ListTree, List, Calendar, ChevronRight } from "lucide-react";
import { clsx } from "clsx";
import type { WorkItem, WorkItemStatus, Identity } from "@/types/ids";
import { useTranslation } from "@/lib/i18n";
import { useWorktreeGroupApi, useWorktreeGroupSessionState } from "@/lib/group/groupProjection";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import {
  listUnifiedWorkItems,
  moveUnifiedWorkItem,
  toStorePriority,
  toUnifiedError,
  UNIFIED_COLUMN_STATUS,
  UNIFIED_KANBAN_COLUMNS,
  type UnifiedKanbanColumn,
  type UnifiedWorkItem,
} from "../kanban/unifiedWorkItems";

// ---- view types ----
// Per 2026-09-05 19:13 JST 拍板: 删 Kanban view (用户明确不需要看板), 默认打开 Sprint
type View = "sprint" | "list" | "tree";

const VIEWS: { id: View; label: string; icon: React.ReactNode }[] = [
  { id: "sprint", label: "Sprint", icon: <Calendar size={12} /> },
  { id: "list",   label: "List",   icon: <List size={12} /> },
  { id: "tree",   label: "Tree",   icon: <ListTree size={12} /> },
];

const isView = (v: string | null): v is View =>
  v === "sprint" || v === "list" || v === "tree";

// ---- helper: priority color ----
const PRIORITY_COLOR: Record<WorkItem["priority"], string> = {
  p0: "text-err",
  p1: "text-warn",
  p2: "text-info",
  p3: "text-ink-dim",
};

// =====================================================================
// Inner component — 实际 page 内容 (useSearchParams 需要 Suspense boundary)
// =====================================================================
function IssuesPageInner() {
  const { t } = useTranslation();
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // ---- store 读 ----
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const board = useStore((s) => s.board);
  const sprints = useStore((s) => s.sprints);
  const transitionWorkItem = useStore((s) => s.transitionWorkItem);
  const addWorkItem = useStore((s) => s.addWorkItem);
  // per 2026-11-17 功能补齐: Sprint 视图项目范围 (null = 全局旧行为)
  const selectedProjectId = useNavStore((s) => s.selectedProjectId);
  const selectedWorktreeId = useNavStore((s) => s.selectedWorktreeId);
  // per 2026-11-21: branch 级隔离随四级树选择联动
  const selectedBranch = useNavStore((s) => s.selectedBranch);

  // ---- 服务端真源通道 (per P1: Sprint 与 Kanban/Backlog 共享同一服务端任务真源) ----
  const groupApi = useWorktreeGroupApi();
  const groupSession = useWorktreeGroupSessionState();
  const runDirectory = useRunDirectory();
  const [serverItems, setServerItems] = useState<UnifiedWorkItem[] | null>(null);
  const [serverError, setServerError] = useState<string | null>(null);
  const [serverRefreshKey, setServerRefreshKey] = useState(0);
  const serverMode =
    groupApi !== null && runDirectory.client !== null && selectedWorktreeId !== "";
  const unauthorized = groupSession.status === "invalidated";

  // 每次进入页面重新 fetch 服务端真源, 并幂等水合 store 缓存 (共享视图组件读 store 渲染;
  // 水合只做读缓存 — 状态/版本以服务端为准, 写操作一律走统一通道)
  useEffect(() => {
    if (!serverMode || !groupApi) return;
    const controller = new AbortController();
    setServerError(null);
    listUnifiedWorkItems(groupApi, selectedWorktreeId, controller.signal)
      .then((items) => {
        setServerItems(items);
        for (const item of items) {
          const current = useStore.getState().workItems.find((w) => w.id === item.id);
          if (!current) {
            addWorkItem({
              id: item.id,
              key: item.id,
              tenant_id: "00000000-0000-0000-0000-000000000001",
              project_id: selectedProjectId || "",
              title: item.title,
              description: item.description,
              kind: item.kind as WorkItem["kind"],
              status: item.column,
              priority: toStorePriority(item.priority),
              reporter_id: item.reporterId || "usr-002",
              labels: item.labels,
              worktree_id: item.worktreeId,
            });
          } else if (current.status !== item.column) {
            transitionWorkItem(item.id, item.column);
          }
        }
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) return;
        setServerError(toUnifiedError(error).message);
        setServerItems(null);
      });
    return () => controller.abort();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [serverMode, groupApi, selectedWorktreeId, serverRefreshKey, selectedProjectId]);

  // 服务端状态移动 (detail 侧栏 / 列表入口共用; 乐观更新 + 失败回滚 + 冲突反馈)
  const handleServerTransition = useCallback(
    async (workItemId: string, toStatus: WorkItemStatus) => {
      if (!groupApi || !serverMode || !serverItems) return;
      if (!(UNIFIED_KANBAN_COLUMNS as readonly string[]).includes(toStatus)) {
        toast.error("该列没有服务端状态语义 — 评审请走评审流程 (review 状态由服务端 review 流程驱动)");
        return;
      }
      const target = toStatus as UnifiedKanbanColumn;
      const item = serverItems.find((w) => w.id === workItemId);
      if (!item) return;
      // 乐观更新 store 缓存 (视图即时反馈)
      transitionWorkItem(workItemId, toStatus);
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
          `sprint-move-${workItemId}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
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
        setServerRefreshKey((k) => k + 1);
      }
    },
    [groupApi, serverMode, serverItems, selectedWorktreeId, transitionWorkItem],
  );

  // ---- view (URL ?view=) ----
  // Per 2026-09-05 19:13 JST 拍板: 默认打开 Sprint
  const rawView = searchParams.get("view");
  const view: View = isView(rawView) ? rawView : "sprint";

  // per 2026-11-21: ?new=true 深链不再渲染独立表单 (Backlog 内联新增替代),
  // newMode/dismissNewMode 已删除。

  // ---- 详情侧栏 ----
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const selectedWi = selectedId ? workItems.find((w) => w.id === selectedId) : null;

  // ---- search 模式 (mock) ----
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  // ---- 过滤 ----
  const filtered = useMemo(() => {
    if (!searchQuery.trim()) return workItems;
    const q = searchQuery.toLowerCase();
    return workItems.filter(
      (w) =>
        w.title.toLowerCase().includes(q) ||
        w.key.toLowerCase().includes(q) ||
        w.labels.some((l) => l.toLowerCase().includes(q)),
    );
  }, [workItems, searchQuery]);

  // ---- 状态计数 (Sprint view badge, 替代 SubNav) ----
  const statusCounts = useMemo(() => {
    const counts: Record<string, number> = { todo: 0, in_progress: 0, review: 0, done: 0, blocked: 0, wontfix: 0 };
    workItems.forEach((w) => {
      if (w.status in counts) counts[w.status]++;
    });
    return counts;
  }, [workItems]);

  // ---- 切 view (改 URL) ----
  const handleSwitchView = useCallback(
    (v: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("view", v);
      router.push(`${pathname}?${params.toString()}`);
    },
    [router, pathname, searchParams],
  );

  // per 2026-11-21: dismissNewMode 已删除 (New issue 表单移除, Backlog 内联新增替代)

  // ---- drag transition 处理 (与 board/page.tsx 一致, 同步 workItem.status + board.columns) ----
  // per P1: 服务端模式走统一写入路径 (乐观更新 + 冲突反馈); 本地模式保留 store 行为
  const handleTransition = useCallback(
    (workItemId: string, toStatus: WorkItemStatus) => {
      if (serverMode) {
        void handleServerTransition(workItemId, toStatus);
        return;
      }
      transitionWorkItem(workItemId, toStatus);
      // 同步 board.columns
      useStore.setState((s) => {
        const fromCol = s.board.columns.find((c) => c.work_item_ids.includes(workItemId));
        const toCol = s.board.columns.find((c) => c.status === toStatus);
        if (!fromCol || !toCol) return s;
        if (fromCol.status === toCol.status) return s;
        return {
          board: {
            ...s.board,
            columns: s.board.columns.map((c) => {
              if (c.status === fromCol.status) {
                return { ...c, work_item_ids: c.work_item_ids.filter((id) => id !== workItemId) };
              }
              if (c.status === toCol.status) {
                return { ...c, work_item_ids: [...c.work_item_ids, workItemId] };
              }
              return c;
            }),
          },
        };
      });
    },
    [transitionWorkItem, serverMode, handleServerTransition],
  );

  // ---- identity map ----
  const identityMap = useMemo(
    () => Object.fromEntries(identities.map((i) => [i.id, i])),
    [identities],
  );

  return (
    <div className="flex h-full min-h-[calc(100vh-64px)]" data-testid="issues-page">
      {/* Per 2026-09-05 19:13 JST 拍板: 干掉 SubNav, 只留全局 Sidebar; view tabs 改顶部 */}
      <div className="flex-1 min-w-0 flex">
        <div className="flex-1 min-w-0 overflow-x-auto">
          <PageHeader
            title={t.pageTitles['/sprint']?.title ?? t.pageTitles['/issues']?.title ?? 'Sprint'}
            subtitle={`${workItems.length} work-items — Sprint 视图为主, 配合 List 表格 + Tree 层级. 默认打开 Sprint.`}
            icon={<FileText className="text-accent" size={20} />}
            count={workItems.length}
          />

          {/* 数据来源标识 (per P1: 服务端真源 / 本地模式 明确区分, 不冒充) */}
          <div
            data-testid="sprint-data-source"
            className={clsx(
              "mb-3 flex items-center gap-2 rounded-lg border px-3 py-2 text-[11px]",
              serverMode ? "border-ok/40 bg-ok/10 text-ok" : "border-warn/40 bg-warn/10 text-warn",
            )}
          >
            <span className={clsx("size-1.5 rounded-full", serverMode ? "bg-ok" : "bg-warn")} />
            {serverMode ? (
              <span>
                服务端真源模式 — 任务卡来自授权服务端投影 (worktree {selectedWorktreeId}), 状态转换与 Kanban/Backlog 共享同一写入路径;
                Sprint 归属仍为本地标注 (服务端尚无 sprint 字段, 属已知缺口)。
              </span>
            ) : (
              <span>
                本地模式 — 当前 Sprint/List/Tree 展示本地状态, 非授权服务端任务事实; 宿主会话与 Worktree 上下文接入后自动切换为服务端真源。
              </span>
            )}
          </div>
          {serverMode && serverError && (
            <div
              data-testid="sprint-server-error"
              className="mb-3 flex items-center gap-2 rounded-lg border border-err/40 bg-err/10 px-3 py-2 text-[11px] text-err"
            >
              <span className="size-1.5 rounded-full bg-err" />
              服务端数据加载失败: {serverError}
            </div>
          )}

          {/* 顶部 1 行: view tabs (per §5.1) + 右侧 New + Search — 神作级 (per 2026-09-05 19:13 JST) */}
          <div className="flex items-center justify-between mb-5">
            <div
              role="tablist"
              aria-label={t.ariaLabels.issueViewTabs ?? 'Sprint view tabs'}
              className="anime-panel anime-chamfer inline-flex items-center gap-1.5 p-2"
              data-testid="issues-view-tabs"
            >
              {VIEWS.map((v) => {
                const active = view === v.id;
                // Per view count badge
                const viewCount =
                  v.id === "sprint" ? sprints.filter((s) => s.status === "active").length :
                  v.id === "list"   ? workItems.length :
                  v.id === "tree"   ? workItems.filter((w) => w.kind === "epic").length : 0;
                return (
                  <button
                    key={v.id}
                    type="button"
                    role="tab"
                    aria-selected={active}
                    data-testid={`issues-view-tab-${v.id}`}
                    onClick={() => handleSwitchView(v.id)}
                    className={clsx(
                      "relative px-4 py-2 text-sm font-semibold flex items-center gap-2 transition-all rounded-[var(--radius-md)]",
                      active
                        ? "text-ink-DEFAULT tab-glow"
                        : "text-ink-dim hover:text-ink hover:bg-bg-soft/40",
                    )}
                    style={active ? {
                      background: "linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 14%, transparent), color-mix(in srgb, var(--color-accent-violet) 10%, transparent))",
                      borderBottom: "2px solid var(--color-primary)",
                    } : undefined}
                  >
                    {v.icon}
                    <span>{v.label}</span>
                    <span
                      className={clsx(
                        "anime-hud-tag text-[9px]",
                        active && "pulse-ok"
                      )}
                    >
                      {viewCount}
                    </span>
                  </button>
                );
              })}
            </div>
            <div className="flex items-center gap-1.5 pb-1.5">
              <button
                type="button"
                data-testid="issues-search-button"
                onClick={() => setSearchOpen((cur) => !cur)}
                className="btn text-xs"
                aria-label={t.ariaLabels.toggleSearch}
              >
                <Search size={12} /> Search
              </button>
              {/* per 2026-11-21 用户发令: 去掉独立 New issue 按钮 —
                  issue 直接在 Backlog 栏内联新增 (SprintBoardView quick-add) */}
            </div>
          </div>

          {/* per 2026-11-21 用户发令: ?new=true stub 表单移除 — issue 直接在
              Backlog 栏内联新增 (SprintBoardView quick-add), 不再需要独立表单 */}

          {/* search 输入 (per 搜索 button 切换) */}
          {searchOpen && (
            <div data-testid="issues-search-bar" className="card mb-4 flex items-center gap-2">
              <Search size={14} className="text-ink-mute" />
              <input
                data-testid="issues-search-input"
                autoFocus
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="按 title / key / labels 过滤..."
                className="bg-transparent flex-1 text-xs focus:outline-none placeholder:text-ink-mute"
              />
              {searchQuery && (
                <span className="text-[10px] text-ink-mute font-mono">
                  {filtered.length} / {workItems.length}
                </span>
              )}
            </div>
          )}

          {/* main view content */}
          <div data-testid={`issues-view-${view}`} className="min-h-[400px]">
            {/* Per 2026-09-05 19:13 JST 拍板: 删 Kanban view 渲染分支 */}
            {view === "list" && (
              <IssuesListView
                items={filtered}
                identities={identities}
                selectedId={selectedId}
                onSelect={setSelectedId}
              />
            )}
            {view === "tree" && (
              <IssuesTreeView items={filtered} identities={identities} onSelect={setSelectedId} />
            )}
            {view === "sprint" && (
                          // Per 2026-09-05 19:32 JST 拍板: Sprint 全套 7 项 (Backlog + 拖动 + 创建/启动/完成/删除)
                          // per 2026-11-17 功能补齐: 传入 navStore 选中项目, SprintBoardView 按
                          //   project_id 过滤 (未选项目时保持全局旧行为)。
                          <SprintBoardView
                            onSelect={setSelectedId}
                            projectId={selectedProjectId ?? undefined}
                            worktreeId={selectedWorktreeId ?? undefined}
                            branch={selectedBranch || undefined}
                          />
                        )}
          </div>

          {/* 视图小贴士 (状态机 + 视图映射) */}
          <div className="mt-4 text-[10px] text-ink-mute font-mono">
            {/* Per 2026-09-05 19:13 JST 拍板: 删 Kanban tip */}
            {/* Per 2026-09-05 19:32 JST 拍板: Sprint 全套 7 项 (Backlog + 拖动 + 创建/启动/完成/删除) */}
            {view === "list"   && <>列表视图 — key / title / kind / status / priority / assignee, 点行选中</>}
            {view === "tree"   && <>树形视图 — epic → story → task → spike 层级 (用 kind + key 推断, Phase 2+ 接 relations)</>}
            {view === "sprint" && <>Sprint 视图 — 左 Backlog 拖入右 Sprint, 启动/完成/删除/改名 + 4 列 kanban, 状态机 planned → active → completed</>}
          </div>
        </div>

        {/* 详情侧栏 (320px) */}
        {selectedWi && (
          <aside
            data-testid="issues-detail-sidebar"
            className="w-80 shrink-0 border-l-2 border-black bg-[var(--cel-surface-sub, var(--color-surface))] overflow-y-auto shadow-soft"
            aria-label={t.ariaLabels.issueDetail}
          >
            <IssuesDetailSidebar
              workItem={selectedWi}
              assignee={selectedWi.assignee_id ? identityMap[selectedWi.assignee_id] : undefined}
              onClose={() => setSelectedId(null)}
              onTransition={(to) => handleTransition(selectedWi.id, to)}
            />
          </aside>
        )}
      </div>
    </div>
  );
}

// =====================================================================
// List view (per §5.1 简单表格)
// =====================================================================
function IssuesListView({
  items,
  identities,
  selectedId,
  onSelect,
}: {
  items: WorkItem[];
  identities: Identity[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  const identityMap = useMemo(
    () => Object.fromEntries(identities.map((i) => [i.id, i])),
    [identities],
  );

  return (
    <div className="card">
      <table className="table" data-testid="issues-list-table">
        <thead>
          <tr>
            <th>Key</th>
            <th>Title</th>
            <th>Kind</th>
            <th>Status</th>
            <th>Priority</th>
            <th>Assignee</th>
            <th>SP</th>
          </tr>
        </thead>
        <tbody>
          {items.length === 0 && (
            <tr>
              <td colSpan={7} className="text-center text-ink-mute italic py-6">无 work-item</td>
            </tr>
          )}
          {items.map((w) => (
            <tr
              key={w.id}
              onClick={() => onSelect(w.id)}
              className={clsx(
                "cursor-pointer transition-colors",
                selectedId === w.id ? "bg-accent/10" : "hover:bg-bg-soft/60",
              )}
              data-testid={`issues-list-row-${w.id}`}
            >
              <td className="font-mono text-xs text-info">{w.key}</td>
              <td className="font-medium">{w.title}</td>
              <td><StatusPill value={w.kind} size="xs" /></td>
              <td><StatusPill value={w.status} size="xs" /></td>
              <td>
                <span className={clsx("font-mono text-xs", PRIORITY_COLOR[w.priority])}>
                  <Flag size={9} className="inline mr-0.5" />
                  {w.priority.toUpperCase()}
                </span>
              </td>
              <td className="text-ink-dim text-xs">
                {w.assignee_id ? identityMap[w.assignee_id]?.display_name ?? w.assignee_id : <span className="text-ink-mute">unassigned</span>}
              </td>
              <td className="font-mono text-xs">{w.story_points ?? "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// =====================================================================
// Tree view (per §5.1 树形 epic → story → task → subtask 缩进)
// per 2026-11-17 功能补齐: 层级改用 WorkItem.parent_id (2026-11-11 addSubTask
// 写入的真实父子关系, 复用既有子任务模型) 而非 kind 推断; 无 parent_id 的旧
// 数据按 kind 归组兜底 (非 epic 归第一个 epic 下, 沿用旧行为)。
// =====================================================================
export type TreeNode = WorkItem & { children: TreeNode[]; depth: number };

const KIND_ORDER: Record<string, number> = { epic: 0, story: 1, task: 2, bug: 3, spike: 4 };

function byKindThenKey(a: WorkItem, b: WorkItem): number {
  const ka = KIND_ORDER[a.kind] ?? 99;
  const kb = KIND_ORDER[b.kind] ?? 99;
  if (ka !== kb) return ka - kb;
  return a.key.localeCompare(b.key);
}

export function buildTree(items: WorkItem[]): TreeNode[] {
  const byId = new Map<string, TreeNode>();
  items.forEach((w) => byId.set(w.id, { ...w, children: [], depth: 0 }));

  // parent_id → children 映射 (真实父子关系); parent 缺失/自环 → 孤儿
  const childrenOf = new Map<string, TreeNode[]>();
  const orphans: TreeNode[] = [];
  for (const w of items) {
    const node = byId.get(w.id)!;
    if (w.parent_id && w.parent_id !== w.id && byId.has(w.parent_id)) {
      const list = childrenOf.get(w.parent_id) ?? [];
      list.push(node);
      childrenOf.set(w.parent_id, list);
    } else {
      orphans.push(node);
    }
  }

  const visited = new Set<string>();
  const attach = (node: TreeNode, depth: number) => {
    if (visited.has(node.id)) return;
    visited.add(node.id);
    node.depth = depth;
    const kids = (childrenOf.get(node.id) ?? []).sort(byKindThenKey);
    for (const c of kids) {
      // 环边切断: 已访问节点不再挂载 (防 parent 链成环时无限递归/渲染栈溢出)
      if (visited.has(c.id)) continue;
      node.children.push(c);
      attach(c, depth + 1);
    }
  };

  const roots: TreeNode[] = [];
  const sortedOrphans = [...orphans].sort(byKindThenKey);
  let firstEpic: TreeNode | null = null;
  for (const node of sortedOrphans) {
    if (node.kind === "epic") {
      roots.push(node);
      attach(node, 0);
      if (!firstEpic) firstEpic = node;
    }
  }
  // 旧数据兜底: 无 parent_id 的非 epic 归到第一个 epic 下 (per 旧 kind 推断行为)
  for (const node of sortedOrphans) {
    if (node.kind === "epic") continue;
    if (firstEpic) {
      attach(node, 1);
      firstEpic.children.push(node);
    } else {
      roots.push(node);
      attach(node, 0);
    }
  }
  // 环兜底: parent 链成环的节点自当 root (防静默丢节点)
  for (const node of byId.values()) {
    if (!visited.has(node.id)) {
      roots.push(node);
      attach(node, 0);
    }
  }
  return roots;
}

function TreeNodeRow({
  node,
  onSelect,
  expanded,
  toggle,
}: {
  node: TreeNode;
  onSelect: (id: string) => void;
  expanded: Set<string>;
  toggle: (id: string) => void;
}) {
  const hasChildren = node.children.length > 0;
  const isOpen = expanded.has(node.id) || !hasChildren;
  return (
    <>
      <tr
        onClick={() => onSelect(node.id)}
        data-testid={`issues-tree-row-${node.id}`}
        className="cursor-pointer hover:bg-bg-soft/60"
      >
        <td colSpan={5} style={{ paddingLeft: 8 + node.depth * 20 }}>
          <div className="flex items-center gap-1.5">
            {hasChildren && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  toggle(node.id);
                }}
                className="text-ink-mute hover:text-ink"
                aria-label={isOpen ? "Collapse" : "Expand"}
              >
                <ChevronRight
                  size={10}
                  className={clsx("transition-transform", isOpen && "rotate-90")}
                />
              </button>
            )}
            <span className="font-mono text-xs text-info">{node.key}</span>
            <span className="text-xs">{node.title}</span>
          </div>
        </td>
        <td><StatusPill value={node.status} size="xs" /></td>
        <td>
          <span className={clsx("font-mono text-xs", PRIORITY_COLOR[node.priority])}>
            {node.priority.toUpperCase()}
          </span>
        </td>
      </tr>
      {isOpen && node.children.map((c) => (
        <TreeNodeRow
          key={c.id}
          node={c}
          onSelect={onSelect}
          expanded={expanded}
          toggle={toggle}
        />
      ))}
    </>
  );
}

function IssuesTreeView({
  items,
  identities,
  onSelect,
}: {
  items: WorkItem[];
  identities: Identity[];
  onSelect: (id: string) => void;
}) {
  const tree = useMemo(() => buildTree(items), [items]);
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(tree.filter((n) => n.kind === "epic").map((n) => n.id)));

  const toggle = useCallback((id: string) => {
    setExpanded((cur) => {
      const next = new Set(cur);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  if (tree.length === 0) {
    return <div className="card text-center text-ink-mute italic py-6">无 work-item</div>;
  }

  return (
    <div className="card" data-testid="issues-tree">
      <table className="table">
        <thead>
          <tr>
            <th className="w-2/3">Key / Title</th>
            <th className="w-24">Status</th>
            <th className="w-16">Priority</th>
          </tr>
        </thead>
        <tbody>
          {tree.map((n) => (
            <TreeNodeRow key={n.id} node={n} onSelect={onSelect} expanded={expanded} toggle={toggle} />
          ))}
        </tbody>
      </table>
    </div>
  );
}

// =====================================================================
// per 2026-11-17 去重: 页内旧 IssuesSprintView (按 sprint 分组 kanban) 删除 —
//   与 SprintBoardView 同数据同操作, 渲染分支早已切换, 属死代码。
// =====================================================================
// Detail sidebar (320px — per §5.1 列下拉 详情侧栏)
// =====================================================================// =====================================================================
// Detail sidebar (320px — per §5.1 列下拉 详情侧栏)
// =====================================================================
function IssuesDetailSidebar({
  workItem,
  assignee,
  onClose,
  onTransition,
}: {
  workItem: WorkItem;
  assignee?: Identity;
  onClose: () => void;
  onTransition: (to: WorkItemStatus) => void;
}) {
  const { t } = useTranslation();
  const ALLOWED: WorkItemStatus[] = ["todo", "in_progress", "review", "blocked", "done", "wontfix"];

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-3">
        <span className="text-[10px] uppercase tracking-wider text-ink-mute">Issue detail</span>
        <button
          onClick={onClose}
          className="text-ink-mute hover:text-ink"
          data-testid="issues-detail-close"
          aria-label={t.ariaLabels.closeDetail}
        >
          <X size={14} />
        </button>
      </div>
      <div className="mb-2">
        <div className="text-[10px] font-mono text-info">{workItem.key}</div>
        <div className="text-sm font-semibold">{workItem.title}</div>
      </div>
      <div className="mb-3">
        <StatusPill value={workItem.status} />
      </div>
      <dl className="text-xs space-y-1.5 mb-4">
        <Row label={<><Tag size={10} className="inline mr-1" />Kind</>} value={workItem.kind} />
        <Row label={<><Flag size={10} className="inline mr-1" />Priority</>} value={workItem.priority.toUpperCase()} />
        <Row label={<><User size={10} className="inline mr-1" />Assignee</>} value={assignee?.display_name ?? workItem.assignee_id ?? "unassigned"} />
        <Row label={<><Hash size={10} className="inline mr-1" />SP</>} value={workItem.story_points ?? "—"} />
        <Row label="Sprint" value={workItem.sprint_id ?? "—"} />
        <Row label="Created" value={new Date(workItem.created_at).toLocaleDateString()} />
      </dl>
      {workItem.labels.length > 0 && (
        <div className="mb-3">
          <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Labels</div>
          <div className="flex flex-wrap gap-1">
            {workItem.labels.map((l) => (
              <span key={l} className="pill border-line text-ink-dim text-[10px]">{l}</span>
            ))}
          </div>
        </div>
      )}
      <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1.5">Transition</div>
      <div className="flex flex-wrap gap-1.5" data-testid="issues-detail-transitions">
        {ALLOWED.filter((s) => s !== workItem.status).map((to) => (
          <button
            key={to}
            onClick={() => onTransition(to)}
            className="btn-primary text-[10px]"
            data-testid={`issues-detail-transition-${to}`}
          >
            → {to}
          </button>
        ))}
      </div>
      <div className="mt-4 pt-3 border-t border-line">
        <Link
          href={`/work-item/${workItem.id}`}
          className="btn text-[10px] w-full justify-center"
          data-testid="issues-detail-legacy-link"
        >
          打开在旧 work-item 详情页 (向后兼容)
        </Link>
      </div>
    </div>
  );
}

function Row({ label, value }: { label: React.ReactNode; value: React.ReactNode }) {
  return (
    <div className="flex justify-between">
      <dt className="text-ink-mute">{label}</dt>
      <dd className="text-ink font-mono text-[11px]">{value}</dd>
    </div>
  );
}

// =====================================================================
// Default export — wrapper with Suspense (per Next.js 14 useSearchParams 要求)
// =====================================================================
export default function IssuesPage() {
  const { t } = useTranslation();
  return (
    <Suspense
      fallback={
        <div className="p-6 text-ink-mute text-sm" data-testid="issues-page-loading">
          加载 Issues 主面板...
        </div>
      }
    >
      <IssuesPageInner />
    </Suspense>
  );
}
