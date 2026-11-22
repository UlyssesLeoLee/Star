/*
@cypher schema=1 source_sha256=2f976cf1cd29e16d4b72bd839b09afe5c7ceb80e2a053d117b3000b4f07ac14e
MERGE (self:File {path:"frontend/src/lib/nav/navStore.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (defaults:Symbol {id:"frontend/src/lib/nav/navStore.ts::DEFAULT_SIDEBAR_ITEMS",kind:"constant"})
MERGE (defaultPinned:Symbol {id:"frontend/src/lib/nav/navStore.ts::DEFAULT_PINNED_VIEWS",kind:"constant"})
MERGE (defaultTabs:Symbol {id:"frontend/src/lib/nav/navStore.ts::DEFAULT_HEADER_TABS",kind:"constant"})
MERGE (indexOwned:Symbol {id:"frontend/src/lib/nav/navStore.ts::WORKTREE_INDEX_OWNED_MODULES",kind:"constant"})
MERGE (registry:File {path:"frontend/src/lib/nav/registry.ts"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(defaults)
MERGE (self)-[:DEFINES]->(defaultPinned)
MERGE (self)-[:DEFINES]->(defaultTabs)
MERGE (self)-[:DEFINES]->(indexOwned)
MERGE (self)-[:IMPORTS]->(registry)
@endcypher
*/
"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { ALL_MODULES, MODULE_MAP, NAV_GROUPS, type ModuleDefinition } from "./registry";

/**
 * Sidebar 折叠状态 (per 2026-09-03 12:36 JST 拍板)
 * - 'expanded' (256px)  默认
 * - 'collapsed' (64px)  仅 icon + code
 */
export type SidebarFoldState = "expanded" | "collapsed";

/**
 * Sidebar 下方 toggle 的两个 scope (per 2026-09-03 12:36 JST 拍板)
 * - 'main'    全局核心模块 (Inbox / Issues / Projects / Agents ...)
 * - 'project' 当前选中项目专属 (Kanban / Timeline / Backlog / Agents / Worktrees)
 *            仅在 /projects 路径下生效, 其他路径自动 fallback 到 main
 */
export type SidebarScope = "main" | "project";

/** 左栏固定顶部页签 (per 2026-11-20 导航验收: 目录 / AI 更新, 持久化) */
export type SidebarTabId = "directory" | "aiUpdates";

/** 底部任务状态筛选 (取值对齐 WorkItemStatus + all, per 2026-11-20 导航验收) */
export const TASK_STATUS_FILTER_VALUES = ["all", "todo", "in_progress", "blocked", "review", "done"] as const;
export type TaskStatusFilter = (typeof TASK_STATUS_FILTER_VALUES)[number];

/** 快捷按钮定义 (per 2026-11-22: 自定义快捷入口，支持 tab 感知跳转) */
export interface ShortcutButton {
  id: string;                    // 唯一标识
  label: string;                 // 显示名
  icon: string;                  // lucide icon name (如 "KanbanSquare", "MessageSquare")
  href: string;                  // 目标路由 (如 "/kanban", "/projects?tab=timeline")
  targetTab?: string;            // 目标 tab (用于同页面不同 tab，如 "kanban" | "canvas" | "nema")
  moduleGroupId?: string;        // 所属模块组 (如 "tasks" | "canvas" | "execution")
  category: "custom" | "pinned" | "global" | "utility";  // 来源分类
  order?: number;                // 排序权重
  createdAt: string;             // ISO 时间
}

export interface NavState {
  // 纵向侧栏定制 (默认仅核心功能)
  sidebarItemIds: string[];
  // 纵向侧栏钉选视图
  pinnedViewIds: string[];
  // 横向顶栏标签定制 (默认核心 5 tab)
  headerTabIds: string[];

  // 弹窗状态 (App Matrix 模块抽屉)
  isMatrixOpen: boolean;

  // Sidebar 折叠状态 (per 2026-09-03 12:36 JST 拍板, 持久化)
  sidebarFold: SidebarFoldState;
  // Sidebar 当前 scope (per 2026-09-03 12:36 JST 拍板, 持久化)
  sidebarScope: SidebarScope;
  // 当前选中项目 ID (从 ProjectsClient 提升, 跨 page 共享, 持久化)
  selectedProjectId: string;
  /** Lane 选择 (per 2026-11-15: Project → Lane → Worktree → Branch 导航上下文) */
  selectedLaneId: string;
  selectedWorktreeId: string;
  selectedBranch: string;
  /** 左栏顶部页签 + 底部任务状态筛选 (per 2026-11-20 导航验收, 持久化) */
    sidebarTab: SidebarTabId;
    taskStatusFilter: TaskStatusFilter;

    // 快捷按钮 (per 2026-11-22: 自定义快捷入口，支持 tab 感知跳转，持久化到 localStorage)
    shortcutButtons: ShortcutButton[];

    // Actions
    openMatrix: () => void;
    closeMatrix: () => void;
    toggleMatrix: () => void;

    addSidebarItem: (id: string) => void;
    removeSidebarItem: (id: string) => void;
    toggleSidebarItem: (id: string) => void;

    addPinnedView: (id: string) => void;
    removePinnedView: (id: string) => void;

    addHeaderTab: (id: string) => void;
    removeHeaderTab: (id: string) => void;
    toggleHeaderTab: (id: string) => void;

    // 快捷按钮管理 (per 2026-11-22: 自定义快捷入口，支持 tab 感知跳转)
    addShortcutButton: (button: ShortcutButton) => void;
    removeShortcutButton: (id: string) => void;
    updateShortcutButton: (id: string, patch: Partial<ShortcutButton>) => void;
    reorderShortcutButtons: (ids: string[]) => void;

    // Sidebar 折叠 / scope 切换 (per 2026-09-03 12:36 JST 拍板)
  toggleSidebarFold: () => void;
  setSidebarFold: (state: SidebarFoldState) => void;
  setSidebarScope: (scope: SidebarScope) => void;
  setSelectedProjectId: (id: string) => void;
  /** Lane/Worktree/Branch 上下文一键设置 (per 2026-11-15) */
  setLaneContext: (laneId: string, worktreeId: string, branch: string) => void;
  setSidebarTab: (tab: SidebarTabId) => void;
  setTaskStatusFilter: (filter: TaskStatusFilter) => void;

  resetToDefault: () => void;
}

// per 2026-11-14 实测对照设计书: 5 组 (NAV_GROUPS) 已全量承载模块,
// 默认不再预填 sidebar/pinned (设计书 §1.2 无"其他入口"预填区; 仅用户主动钉选才显示)
export const DEFAULT_SIDEBAR_ITEMS: string[] = [];
// per 2026-11-14: 视图组已承载 kanban/timeline/backlog/canvas/refactor/search, 默认不预填
export const DEFAULT_PINNED_VIEWS: string[] = [];
// per 2026-11-14 artifact 设计: 顶栏 = Branch 功能 tabs (任务/画布/协同/沙盒/原型)
export const DEFAULT_HEADER_TABS = ["kanban", "canvas", "nema", "sandbox", "prototype"];

/** 由 Project Worktree Index 承载的模块 id — 不允许出现在 sidebar/pinned/其他入口 */
export const WORKTREE_INDEX_OWNED_MODULES = ["worktree"] as const;

type PersistedNavState = Pick<
  NavState,
  | "sidebarItemIds"
  | "pinnedViewIds"
  | "headerTabIds"
  | "sidebarFold"
  | "sidebarScope"
  | "selectedProjectId"
  | "selectedLaneId"
  | "selectedWorktreeId"
  | "selectedBranch"
  | "sidebarTab"
  | "taskStatusFilter"
  | "shortcutButtons"
>;

function migratePersistedNavState(persistedState: unknown): PersistedNavState {
  const persisted = persistedState as Partial<PersistedNavState> | null;
  const storedSidebarItems = Array.isArray(persisted?.sidebarItemIds)
    ? persisted.sidebarItemIds
    : DEFAULT_SIDEBAR_ITEMS;
  const storedPinnedViews = Array.isArray(persisted?.pinnedViewIds)
    ? persisted.pinnedViewIds
    : DEFAULT_PINNED_VIEWS;
  const storedShortcutButtons = Array.isArray(persisted?.shortcutButtons)
    ? persisted.shortcutButtons
    : [];

  // per 2026-11-14 实测对照设计书: NAV_GROUPS 5 组已全量承载模块,
  // 旧版预填的 sidebar/pinned 一次性清理 (组内已显示, 不再重复渲染)
  const mappedIds = new Set<string>(Object.values(NAV_GROUPS).flat());
  const cleanedSidebar = storedSidebarItems.filter((id) => !mappedIds.has(id));
  const cleanedPinned = storedPinnedViews.filter((id) => !mappedIds.has(id));

  // per 2026-11-14 artifact 重排 (v8): 顶栏从 inbox/issues/projects/agents/analytics
  // 改为 Branch 功能 5 tabs — 旧持久化的 headerTabIds 一次性替换, 不复用
  // (否则浏览器 localStorage 残留旧 tabs, UI 看起来"没变化")
  const headerTabIds = Array.isArray(persisted?.headerTabIds)
    ? persisted.headerTabIds
    : DEFAULT_HEADER_TABS;
  const hasLegacyHeaderTabs = headerTabIds.some((id) =>
    ["inbox", "issues", "projects", "agents", "analytics"].includes(id),
  );
  const cleanedHeaderTabs = hasLegacyHeaderTabs ? DEFAULT_HEADER_TABS : headerTabIds;

  return {
    sidebarItemIds: cleanedSidebar,
    pinnedViewIds: cleanedPinned,
    headerTabIds: cleanedHeaderTabs,
    sidebarFold: persisted?.sidebarFold === "collapsed" ? "collapsed" : "expanded",
    sidebarScope: persisted?.sidebarScope === "project" ? "project" : "main",
    selectedProjectId:
      typeof persisted?.selectedProjectId === "string" ? persisted.selectedProjectId : "",
    selectedLaneId:
      typeof persisted?.selectedLaneId === "string" ? persisted.selectedLaneId : "",
    selectedWorktreeId:
      typeof persisted?.selectedWorktreeId === "string" ? persisted.selectedWorktreeId : "",
    selectedBranch:
      typeof persisted?.selectedBranch === "string" ? persisted.selectedBranch : "",
    sidebarTab: persisted?.sidebarTab === "aiUpdates" ? "aiUpdates" : "directory",
    taskStatusFilter: TASK_STATUS_FILTER_VALUES.includes(persisted?.taskStatusFilter as TaskStatusFilter)
      ? (persisted!.taskStatusFilter as TaskStatusFilter)
      : "all",
    shortcutButtons: storedShortcutButtons,
  };
}

export const useNavStore = create<NavState>()(
  persist(
    (set, get) => ({
      sidebarItemIds: DEFAULT_SIDEBAR_ITEMS,
      pinnedViewIds: DEFAULT_PINNED_VIEWS,
      headerTabIds: DEFAULT_HEADER_TABS,
            isMatrixOpen: false,
            sidebarFold: "expanded",
            sidebarScope: "main",
            selectedProjectId: "",
            selectedLaneId: "",
            selectedWorktreeId: "",
            selectedBranch: "",
            sidebarTab: "directory",
            taskStatusFilter: "all",
            shortcutButtons: [],

            openMatrix: () => set({ isMatrixOpen: true }),
      closeMatrix: () => set({ isMatrixOpen: false }),
      toggleMatrix: () => set((s) => ({ isMatrixOpen: !s.isMatrixOpen })),

      addSidebarItem: (id: string) => {
        if (!get().sidebarItemIds.includes(id)) {
          set((s) => ({ sidebarItemIds: [...s.sidebarItemIds, id] }));
        }
      },
      removeSidebarItem: (id: string) => {
        set((s) => ({
          sidebarItemIds: s.sidebarItemIds.filter((item) => item !== id),
        }));
      },
      toggleSidebarItem: (id: string) => {
        const has = get().sidebarItemIds.includes(id);
        if (has) {
          get().removeSidebarItem(id);
        } else {
          get().addSidebarItem(id);
        }
      },

      addPinnedView: (id: string) => {
        if (!get().pinnedViewIds.includes(id)) {
          set((s) => ({ pinnedViewIds: [...s.pinnedViewIds, id] }));
        }
      },
      removePinnedView: (id: string) => {
        set((s) => ({
          pinnedViewIds: s.pinnedViewIds.filter((item) => item !== id),
        }));
      },

      addHeaderTab: (id: string) => {
        if (!get().headerTabIds.includes(id)) {
          set((s) => ({ headerTabIds: [...s.headerTabIds, id] }));
        }
      },
      removeHeaderTab: (id: string) => {
        set((s) => ({
          headerTabIds: s.headerTabIds.filter((item) => item !== id),
        }));
      },
      toggleHeaderTab: (id: string) => {
        const has = get().headerTabIds.includes(id);
        if (has) {
          get().removeHeaderTab(id);
        } else {
          get().addHeaderTab(id);
        }
      },

      // 折叠 toggle: expanded ↔ collapsed (per 2026-09-03 12:36 JST 拍板)
      toggleSidebarFold: () => {
        set((s) => ({
          sidebarFold: s.sidebarFold === "expanded" ? "collapsed" : "expanded",
        }));
      },
      // 显式设置 (e.g. 快捷键 / hover 触发)
      setSidebarFold: (state) => set({ sidebarFold: state }),
      // scope 切换: main ↔ project (per 2026-09-03 12:36 JST 拍板)
      setSidebarScope: (scope) => set({ sidebarScope: scope }),
      // selectedProjectId setter (per 2026-09-03 12:36 JST 从 ProjectsClient 提升)
      setSelectedProjectId: (id) =>
        set((s) => ({
          selectedProjectId: id,
          // per 2026-11-15: 切换 Project 清空下级上下文, 防残留 (Lane/Worktree/Branch 属于旧项目)
          ...(s.selectedProjectId !== id
            ? { selectedLaneId: "", selectedWorktreeId: "", selectedBranch: "" }
            : {}),
        })),
      setLaneContext: (laneId, worktreeId, branch) =>
              set({ selectedLaneId: laneId, selectedWorktreeId: worktreeId, selectedBranch: branch }),
            setSidebarTab: (tab) => set({ sidebarTab: tab }),
                        setTaskStatusFilter: (filter) => set({ taskStatusFilter: filter }),

                        // 快捷按钮管理 (per 2026-11-22: 自定义快捷入口，支持 tab 感知跳转)
                        addShortcutButton: (button: ShortcutButton) =>
                          set((s) => ({ shortcutButtons: [...s.shortcutButtons, button] })),
                        removeShortcutButton: (id: string) =>
                          set((s) => ({ shortcutButtons: s.shortcutButtons.filter((b) => b.id !== id) })),
                        updateShortcutButton: (id: string, patch: Partial<ShortcutButton>) =>
                          set((s) => ({
                            shortcutButtons: s.shortcutButtons.map((b) =>
                              b.id === id ? { ...b, ...patch } : b
                            ),
                          })),
                        reorderShortcutButtons: (ids: string[]) =>
                          set((s) => {
                            const map = new Map(s.shortcutButtons.map((b) => [b.id, b]));
                            const reordered = ids.map((id) => map.get(id)).filter(Boolean) as ShortcutButton[];
                            // 保留未在 ids 中的按钮 (放在最后)
                            const remaining = s.shortcutButtons.filter((b) => !ids.includes(b.id));
                            return { shortcutButtons: [...reordered, ...remaining] };
                          }),

                        resetToDefault: () => {
        set({
          sidebarItemIds: DEFAULT_SIDEBAR_ITEMS,
          pinnedViewIds: DEFAULT_PINNED_VIEWS,
          headerTabIds: DEFAULT_HEADER_TABS,
          // 折叠/scope/selectedProjectId 不在 reset 范围 — 用户偏好不属于"误改"恢复
        });
      },
    }),
    {
      // bump v4: WBS §23.32-A — worktree 模块移出 sidebar/pinned, kanban/timeline/backlog 归并 pinned
      name: "star-nav-store:v2",
      // v9 (2026-11-14): v8 的 migrate 沿用了旧持久化 headerTabIds — 再跑一次强制替换
      version: 9,
      migrate: (persistedState) => migratePersistedNavState(persistedState),
      storage: createJSONStorage(() => (typeof window !== "undefined" ? localStorage : {
        getItem: () => null,
        setItem: () => {},
        removeItem: () => {},
      })),
      partialize: (s) => ({
                    sidebarItemIds: s.sidebarItemIds,
                    pinnedViewIds: s.pinnedViewIds,
                    headerTabIds: s.headerTabIds,
                    sidebarFold: s.sidebarFold,
                    sidebarScope: s.sidebarScope,
                    selectedProjectId: s.selectedProjectId,
                    selectedLaneId: s.selectedLaneId,
                    selectedWorktreeId: s.selectedWorktreeId,
                    selectedBranch: s.selectedBranch,
                    sidebarTab: s.sidebarTab,
                    taskStatusFilter: s.taskStatusFilter,
                    shortcutButtons: s.shortcutButtons,
                  }),
    }
  )
);
