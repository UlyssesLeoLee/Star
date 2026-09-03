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

export const DEFAULT_SIDEBAR_ITEMS = ["inbox", "issues", "projects", "agents"];
export const DEFAULT_PINNED_VIEWS = ["kanban", "timeline"];
export const DEFAULT_HEADER_TABS = ["inbox", "issues", "projects", "agents", "analytics"];

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
      // bump v2: 加 sidebarFold + sidebarScope + selectedProjectId 持久化 (per 2026-09-03)
      name: "star-nav-store:v2",
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
