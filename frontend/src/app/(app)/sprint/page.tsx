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

import { useState, useMemo, useCallback, Suspense } from "react";
import Link from "next/link";
import { useRouter, useSearchParams, usePathname } from "next/navigation";
import { useStore } from "@/lib/store";
import { StatusPill } from "@/components/StatusPill";
import { PageHeader } from "@/components/PageHeader";
import { SprintBoardView } from "@/components/sprint/SprintBoardView";
import { Plus, Search, X, Flag, User, Hash, Tag, FileText, GitBranch, ListTree, LayoutGrid, List, Calendar, ChevronRight } from "lucide-react";
import { clsx } from "clsx";
import type { WorkItem, WorkItemStatus, Identity } from "@/types/ids";

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
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // ---- store 读 ----
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const board = useStore((s) => s.board);
  const sprints = useStore((s) => s.sprints);
  const transitionWorkItem = useStore((s) => s.transitionWorkItem);

  // ---- view (URL ?view=) ----
  // Per 2026-09-05 19:13 JST 拍板: 默认打开 Sprint
  const rawView = searchParams.get("view");
  const view: View = isView(rawView) ? rawView : "sprint";

  // ---- new issue 模式 (?new=true) ----
  const newMode = searchParams.get("new") === "true";

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

  // ---- new issue 模式切回 ----
  const dismissNewMode = useCallback(() => {
    const params = new URLSearchParams(searchParams.toString());
    params.delete("new");
    router.push(`${pathname}?${params.toString()}`);
  }, [router, pathname, searchParams]);

  // ---- drag transition 处理 (与 board/page.tsx 一致, 同步 workItem.status + board.columns) ----
  const handleTransition = useCallback(
    (workItemId: string, toStatus: WorkItemStatus) => {
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
    [transitionWorkItem],
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
            title="Sprint"
            subtitle={`${workItems.length} work-items — Sprint 视图为主, 配合 List 表格 + Tree 层级. 默认打开 Sprint.`}
            icon={<FileText className="text-accent" size={20} />}
            count={workItems.length}
          />

          {/* 顶部 1 行: view tabs (per §5.1) + 右侧 New + Search — 神作级 (per 2026-09-05 19:13 JST) */}
          <div className="flex items-center justify-between mb-5">
            <div
              role="tablist"
              aria-label="Sprint view tabs"
              className="anime-panel anime-chamfer inline-flex items-center gap-1 p-1.5"
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
                aria-label="Toggle search"
              >
                <Search size={12} /> Search
              </button>
              <button
                type="button"
                data-testid="issues-new-button"
                onClick={() => handleSwitchView(view) /* keep view, toggle new via URL */}
                className="btn-primary text-xs"
                aria-label="Create new issue"
              >
                <Plus size={12} /> New issue
              </button>
            </div>
          </div>

          {/* new issue 表单 (per ?new=true) */}
          {newMode && (
            <div
              data-testid="issues-new-banner"
              className="card mb-4 border-accent/40 bg-accent/5 flex items-start justify-between gap-2"
            >
              <div className="text-xs">
                <div className="text-accent font-medium mb-1">+ New issue</div>
                <div className="text-ink-dim">
                  创建表单 (Phase 2+ 接后端 API). 临时表单:
                </div>
                <form
                  className="mt-2 flex gap-2"
                  onSubmit={(e) => {
                    e.preventDefault();
                    dismissNewMode();
                  }}
                >
                  <input
                    data-testid="issues-new-title-input"
                    type="text"
                    placeholder="Issue title (stub)"
                    className="bg-bg-soft border border-line rounded px-2 py-1 text-xs flex-1"
                  />
                  <button type="submit" className="btn-primary text-xs">Create</button>
                </form>
              </div>
              <button
                onClick={dismissNewMode}
                className="text-ink-mute hover:text-ink"
                aria-label="Dismiss new issue"
              >
                <X size={14} />
              </button>
            </div>
          )}

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
              // Per 2026-09-05 19:32 JST 拍板: Sprint 对标 Jira, 全套 7 项 (Backlog + 拖动 + 创建/启动/完成/删除)
              <SprintBoardView onSelect={setSelectedId} />
            )}
          </div>

          {/* 视图小贴士 (状态机 + 视图映射) */}
          <div className="mt-4 text-[10px] text-ink-mute font-mono">
            {/* Per 2026-09-05 19:13 JST 拍板: 删 Kanban tip */}
            {/* Per 2026-09-05 19:32 JST 拍板: Sprint 对标 Jira, 7 项 (Backlog + 拖动 + 创建/启动/完成/删除) */}
            {view === "list"   && <>列表视图 — key / title / kind / status / priority / assignee, 点行选中</>}
            {view === "tree"   && <>树形视图 — epic → story → task → spike 层级 (用 kind + key 推断, Phase 2+ 接 relations)</>}
            {view === "sprint" && <>Sprint 视图 — 左 Backlog 拖入右 Sprint, 启动/完成/删除/改名 + 4 列 kanban, 状态机 planned → active → completed</>}
          </div>
        </div>

        {/* 详情侧栏 (320px) */}
        {selectedWi && (
          <aside
            data-testid="issues-detail-sidebar"
            className="w-80 shrink-0 border-l-2 border-black bg-[var(--cel-surface-sub,#151c2c)] overflow-y-auto cel-shadow"
            aria-label="Issue detail"
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
// =====================================================================
type TreeNode = WorkItem & { children: TreeNode[]; depth: number };

function buildTree(items: WorkItem[]): TreeNode[] {
  // 简化: 用 kind 分层 (epic → story → task → bug/spike)
  // 真实环境应该用 parent_id / relations 表
  // 排序: kind priority (epic > story > task > bug > spike) + key 升序
  const KIND_ORDER: Record<string, number> = { epic: 0, story: 1, task: 2, bug: 3, spike: 4 };

  const byId = new Map<string, TreeNode>();
  items.forEach((w) => byId.set(w.id, { ...w, children: [], depth: 0 }));

  // 按 kind 排序的 root 节点 (epic 当 root)
  const sorted = [...items].sort((a, b) => {
    const ka = KIND_ORDER[a.kind] ?? 99;
    const kb = KIND_ORDER[b.kind] ?? 99;
    if (ka !== kb) return ka - kb;
    return a.key.localeCompare(b.key);
  });

  const roots: TreeNode[] = [];
  sorted.forEach((w) => {
    const node = byId.get(w.id)!;
    if (w.kind === "epic") {
      // epic 是 root
      node.depth = 0;
      roots.push(node);
    } else {
      // 找一个 epic (key prefix 匹配) 或者 上一个更高优先级
      const epicRoot = roots.find((r) => r.kind === "epic");
      if (epicRoot) {
        node.depth = 1;
        epicRoot.children.push(node);
      } else {
        // 没有 epic — 自当 root
        node.depth = 0;
        roots.push(node);
      }
    }
  });

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
// Sprint view (per §5.1 按 sprint 分组的 Kanban)
// =====================================================================
function IssuesSprintView({
  items,
  sprints,
  identities,
  onSelect,
}: {
  items: WorkItem[];
  sprints: ReturnType<typeof useStore.getState>["sprints"];
  identities: Identity[];
  onSelect: (id: string) => void;
}) {
  // 按 sprint 分组 (有 sprint_id 优先, 没 sprint_id 的进 "Backlog")
  const grouped = useMemo(() => {
    const map = new Map<string, WorkItem[]>();
    sprints.forEach((s) => map.set(s.id, []));
    map.set("__backlog__", []);

    items.forEach((w) => {
      const key = w.sprint_id ?? "__backlog__";
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(w);
    });

    return sprints
      .map((s) => ({ sprint: s, items: map.get(s.id) ?? [] }))
      .filter((g) => g.items.length > 0)
      .concat(
        map.get("__backlog__")!.length > 0
          ? [{ sprint: { id: "__backlog__", name: "Backlog", status: "planned" } as any, items: map.get("__backlog__")! }]
          : [],
      );
  }, [items, sprints]);

  if (grouped.length === 0) {
    return <div className="card text-center text-ink-mute italic py-6">无 sprint</div>;
  }

  return (
    <div className="space-y-4" data-testid="issues-sprint-list">
      {grouped.map(({ sprint, items: sprintItems }) => (
        <div key={sprint.id} className="card" data-testid={`issues-sprint-${sprint.id}`}>
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <GitBranch size={12} className="text-accent" />
              <span className="text-sm font-semibold">{sprint.name}</span>
              {sprint.status && <StatusPill value={sprint.status} size="xs" />}
            </div>
            <span className="text-[10px] text-ink-mute font-mono">
              {sprintItems.length} items
            </span>
          </div>
          <div className="grid grid-cols-4 gap-2">
            {(["todo", "in_progress", "review", "done"] as const).map((st) => {
              const colItems = sprintItems.filter((w) => w.status === st);
              return (
                <div
                  key={st}
                  data-testid={`issues-sprint-col-${sprint.id}-${st}`}
                  className="border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] p-2 min-h-[80px] cel-shadow"
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <StatusPill value={st} size="xs" />
                    <span className="text-[10px] text-ink-mute font-mono">{colItems.length}</span>
                  </div>
                  <div className="space-y-1">
                    {colItems.slice(0, 5).map((w) => (
                      <div
                        key={w.id}
                        onClick={() => onSelect(w.id)}
                        className="text-[11px] px-1.5 py-1 rounded bg-bg-soft/60 hover:bg-bg-soft cursor-pointer border-l-2"
                        data-testid={`issues-sprint-card-${w.id}`}
                      >
                        <div className="flex items-center justify-between">
                          <span className="font-mono text-[9px] text-info">{w.key}</span>
                          <span className={clsx("font-mono text-[9px]", PRIORITY_COLOR[w.priority])}>
                            {w.priority.toUpperCase()}
                          </span>
                        </div>
                        <div className="line-clamp-1">{w.title}</div>
                      </div>
                    ))}
                    {colItems.length > 5 && (
                      <div className="text-[10px] text-ink-mute italic text-center">+{colItems.length - 5} more</div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

// =====================================================================
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
  const ALLOWED: WorkItemStatus[] = ["todo", "in_progress", "review", "blocked", "done", "wontfix"];

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-3">
        <span className="text-[10px] uppercase tracking-wider text-ink-mute">Issue detail</span>
        <button
          onClick={onClose}
          className="text-ink-mute hover:text-ink"
          data-testid="issues-detail-close"
          aria-label="Close detail"
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
