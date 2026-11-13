/*
@cypher schema=1 source_sha256=8a949a4904895fd01d62b6b7ae20c2699c959ef40deac0a25115e238e989fafa
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
import { ALL_MODULES, MODULE_MAP, type ModuleDefinition } from "./registry";

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

  // Sidebar 折叠 / scope 切换 (per 2026-09-03 12:36 JST 拍板)
  toggleSidebarFold: () => void;
  setSidebarFold: (state: SidebarFoldState) => void;
  setSidebarScope: (scope: SidebarScope) => void;
  setSelectedProjectId: (id: string) => void;

  resetToDefault: () => void;
}

export const DEFAULT_SIDEBAR_ITEMS = ["inbox", "issues", "projects", "agents", "advanced-settings"];
// per WBS §23.32-A (2026-11-13 用户发令): kanban/timeline/backlog 归入"其他入口"折叠区;
// worktree 模块集中在 Project Worktree Index (/worktree), 不允许进入 pinned/sidebar
export const DEFAULT_PINNED_VIEWS = ["kanban", "timeline", "backlog"];
export const DEFAULT_HEADER_TABS = ["inbox", "issues", "projects", "agents", "analytics"];

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
>;

function migratePersistedNavState(persistedState: unknown): PersistedNavState {
  const persisted = persistedState as Partial<PersistedNavState> | null;
  const storedSidebarItems = Array.isArray(persisted?.sidebarItemIds)
    ? persisted.sidebarItemIds
    : DEFAULT_SIDEBAR_ITEMS;
  const storedPinnedViews = Array.isArray(persisted?.pinnedViewIds)
    ? persisted.pinnedViewIds
    : DEFAULT_PINNED_VIEWS;

  // per WBS §23.32-A: worktree 模块由 Project Worktree Index 承载,
  // 从 sidebar/pinned 双区清理; kanban/timeline/backlog 补齐到 pinned (去重)
  const cleanedSidebar = storedSidebarItems.filter(
    (id) => !(WORKTREE_INDEX_OWNED_MODULES as readonly string[]).includes(id)
  );
  const cleanedPinned = storedPinnedViews.filter(
    (id) => !(WORKTREE_INDEX_OWNED_MODULES as readonly string[]).includes(id)
  );
  const mergedPinned = [...cleanedPinned];
  for (const id of DEFAULT_PINNED_VIEWS) {
    if (!mergedPinned.includes(id)) mergedPinned.push(id);
  }

  return {
    sidebarItemIds: cleanedSidebar.includes("advanced-settings")
      ? cleanedSidebar
      : [...cleanedSidebar, "advanced-settings"],
    pinnedViewIds: mergedPinned,
    headerTabIds: Array.isArray(persisted?.headerTabIds)
      ? persisted.headerTabIds
      : DEFAULT_HEADER_TABS,
    sidebarFold: persisted?.sidebarFold === "collapsed" ? "collapsed" : "expanded",
    sidebarScope: persisted?.sidebarScope === "project" ? "project" : "main",
    selectedProjectId:
      typeof persisted?.selectedProjectId === "string" ? persisted.selectedProjectId : "",
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

      openMatrix: () => set({ isMatrixOpen: true }),
      closeMatrix: () => set({ isMatrixOpen: false }),
      toggleMatrix: () => set((s) => ({ isMatrixOpen: !s.isMatrixOpen })),

      addSidebarItem: (id: string) => {
        // per WBS §23.32-A: worktree 集中在 Project Worktree Index, 禁止进 sidebar
        if ((WORKTREE_INDEX_OWNED_MODULES as readonly string[]).includes(id)) return;
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
        // per WBS §23.32-A: worktree 集中在 Project Worktree Index, 禁止进 pinned
        if ((WORKTREE_INDEX_OWNED_MODULES as readonly string[]).includes(id)) return;
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
      setSelectedProjectId: (id) => set({ selectedProjectId: id }),

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
      version: 4,
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
      }),
    }
  )
);
