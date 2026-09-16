// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/stores/worktreeCanvasStore.ts
//
// Zustand store for AI Worktree Graph Canvas (per ULYS-57.4 T14.6 +
// WORKTREE-CANVAS-IMPL-PLAN-001 §4.14).
//
// 4 sub-store 合一 (per IMPL-PLAN §4.14.6):
// - worktrees: 全部 Worktree Node 列表
// - graph: 邻接关系 + focus N-hop subgraph
// - events: 15 SSE event buffer (recent N)
// - viewport: 画布 pan / zoom 状态
//
// 守门合规 (per 守门 #19 v19): 独立 store 文件, 0 改 V0.1 useStore.

"use client";

import { create } from "zustand";
import type {
  WorktreeNodeData,
  EdgeData,
  ViewMode,
  ZoomLevel,
  InspectorTab,
  SseEventType,
  Viewport,
} from "@/components/worktree-canvas/types";

const MAX_EVENT_BUFFER = 100;

export interface WorktreeCanvasState {
  // ── worktrees ──
  worktrees: WorktreeNodeData[];
  setWorktrees: (wts: WorktreeNodeData[]) => void;
  upsertWorktree: (wt: WorktreeNodeData) => void;
  removeWorktree: (id: string) => void;

  // ── graph (edges + focus) ──
  edges: EdgeData[];
  setEdges: (edges: EdgeData[]) => void;
  focusNodeId: string | null;
  focusHop: number;
  setFocus: (nodeId: string | null, hop?: number) => void;

  // ── events (15 SSE buffer) ──
  recentEvents: { type: SseEventType; payload: unknown; at: number }[];
  pushEvent: (type: SseEventType, payload: unknown) => void;
  clearEvents: () => void;

  // ── viewport ──
  viewport: Viewport;
  setViewport: (v: Viewport) => void;
  resetViewport: () => void;

  // ── UI preferences ──
  viewMode: ViewMode;
  setViewMode: (m: ViewMode) => void;
  zoomLevel: ZoomLevel;
  setZoomLevel: (z: ZoomLevel) => void;
  inspectorTab: InspectorTab;
  setInspectorTab: (t: InspectorTab) => void;
  selectedWorktreeId: string | null;
  selectWorktree: (id: string | null) => void;
  searchQuery: string;
  setSearchQuery: (q: string) => void;

  // ── focus mode ──
  focusMode: boolean;
  setFocusMode: (on: boolean) => void;
}

const DEFAULT_VIEWPORT: Viewport = {
  x: 0,
  y: 0,
  zoom: 1,
  width: 0,
  height: 0,
};

export const useWorktreeCanvasStore = create<WorktreeCanvasState>((set) => ({
  // worktrees
  worktrees: [],
  setWorktrees: (wts) => set({ worktrees: wts }),
  upsertWorktree: (wt) =>
    set((s) => {
      const idx = s.worktrees.findIndex((w) => w.id === wt.id);
      if (idx === -1) return { worktrees: [...s.worktrees, wt] };
      const next = s.worktrees.slice();
      next[idx] = wt;
      return { worktrees: next };
    }),
  removeWorktree: (id) =>
    set((s) => ({ worktrees: s.worktrees.filter((w) => w.id !== id) })),

  // graph
  edges: [],
  setEdges: (edges) => set({ edges }),
  focusNodeId: null,
  focusHop: 1,
  setFocus: (nodeId, hop = 1) => set({ focusNodeId: nodeId, focusHop: hop }),

  // events
  recentEvents: [],
  pushEvent: (type, payload) =>
    set((s) => {
      const next = [
        { type, payload, at: Date.now() },
        ...s.recentEvents,
      ].slice(0, MAX_EVENT_BUFFER);
      return { recentEvents: next };
    }),
  clearEvents: () => set({ recentEvents: [] }),

  // viewport
  viewport: DEFAULT_VIEWPORT,
  setViewport: (v) => set({ viewport: v }),
  resetViewport: () => set({ viewport: DEFAULT_VIEWPORT }),

  // UI
  viewMode: "TREE",
  setViewMode: (m) => set({ viewMode: m }),
  zoomLevel: "L1",
  setZoomLevel: (z) => set({ zoomLevel: z }),
  inspectorTab: "Overview",
  setInspectorTab: (t) => set({ inspectorTab: t }),
  selectedWorktreeId: null,
  selectWorktree: (id) => set({ selectedWorktreeId: id }),
  searchQuery: "",
  setSearchQuery: (q) => set({ searchQuery: q }),

  focusMode: false,
  setFocusMode: (on) => set({ focusMode: on }),
}));

export const MAX_RECENT_EVENTS = MAX_EVENT_BUFFER;
