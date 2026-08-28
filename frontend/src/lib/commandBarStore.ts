// =====================================================================
// commandBarStore — CommandBar 状态 (per §6 + 任务要求)
// =====================================================================
// - 与 W5 store 同样的 zustand+persist 模式
// - 单独文件避免与 W5 大 store 产生 merge 冲突 (per worker 并行实装规则)
// - 部分持久化: 只持久化 recent (open/close/query 每次会话无关)
// - localStorage key: "star-commandbar:v1"
// - SSR-safe storage (per W5 store.ts 同款)
// =====================================================================
"use client";

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

export type CommandBarItemType = "page" | "command";

export interface RecentItem {
  id: string;
  label: string;
  href?: string;
  type: CommandBarItemType;
  at: number;
}

export interface CommandBarState {
  isOpen: boolean;
  query: string;
  recent: RecentItem[];
  open: () => void;
  close: () => void;
  toggle: () => void;
  setQuery: (q: string) => void;
  pushRecent: (item: RecentItem) => void;
  clearRecent: () => void;
}

// ---- SSR-safe localStorage 包装 (与 lib/store.ts 同款) ----
const ssrSafeStorage = createJSONStorage(() => ({
  getItem: (name: string): string | null => {
    if (typeof window === "undefined") return null;
    try {
      return window.localStorage.getItem(name);
    } catch {
      return null;
    }
  },
  setItem: (name: string, value: string): void => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem(name, value);
    } catch {
      // silent
    }
  },
  removeItem: (name: string): void => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.removeItem(name);
    } catch {
      // silent
    }
  },
}));

export const useCommandBarStore = create<CommandBarState>()(
  persist(
    (set) => ({
      isOpen: false,
      query: "",
      recent: [],
      open: () => set({ isOpen: true, query: "" }),
      close: () => set({ isOpen: false, query: "" }),
      toggle: () => set((s) => ({ isOpen: !s.isOpen, query: !s.isOpen ? "" : s.query })),
      setQuery: (q) => set({ query: q }),
      pushRecent: (item) =>
        set((s) => {
          // 去重 + 最多 5 条 + 最新在前
          const filtered = s.recent.filter((r) => r.id !== item.id);
          return { recent: [item, ...filtered].slice(0, 5) };
        }),
      clearRecent: () => set({ recent: [] }),
    }),
    {
      name: "star-commandbar:v1",
      storage: ssrSafeStorage,
      // 只持久化 recent, isOpen/query 每次会话无关
      partialize: (state) => ({ recent: state.recent }),
      version: 1,
    }
  )
);
