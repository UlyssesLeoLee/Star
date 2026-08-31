"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { ALL_MODULES, MODULE_MAP, type ModuleDefinition } from "./registry";

export interface NavState {
  // 纵向侧栏定制 (默认仅核心功能)
  sidebarItemIds: string[];
  // 纵向侧栏钉选视图
  pinnedViewIds: string[];
  // 横向顶栏标签定制 (默认核心 5 tab)
  headerTabIds: string[];

  // 弹窗状态 (App Matrix 模块抽屉)
  isMatrixOpen: boolean;

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

      resetToDefault: () => {
        set({
          sidebarItemIds: DEFAULT_SIDEBAR_ITEMS,
          pinnedViewIds: DEFAULT_PINNED_VIEWS,
          headerTabIds: DEFAULT_HEADER_TABS,
        });
      },
    }),
    {
      name: "star-nav-store:v1",
      storage: createJSONStorage(() => (typeof window !== "undefined" ? localStorage : {
        getItem: () => null,
        setItem: () => {},
        removeItem: () => {},
      })),
      partialize: (s) => ({
        sidebarItemIds: s.sidebarItemIds,
        pinnedViewIds: s.pinnedViewIds,
        headerTabIds: s.headerTabIds,
      }),
    }
  )
);
