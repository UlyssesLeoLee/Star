"use client";

// =====================================================================
// terminalStackStore.ts — zustand store (per ULYS-198 unified status store 同款)
// (per ULYS-223 P1-D §2 状态管理)
// =====================================================================
// 守门:
// - 单一 pane tree SoT (source of truth) - 避免 split / merge 竞态
// - WebSocket push SplitUpdate → setTree (per P1-C 协议)
// - resize delta → applyResize (debounced, 不触发 backend roundtrip)
// =====================================================================

import { create } from "zustand";
import {
  type SplitDirection,
  type SplitTreeView,
  newSingleTree,
  splitPane,
  findPane,
} from "./types";

export interface TerminalStackState {
  tree: SplitTreeView;
  /** 当前激活 pane id (focus highlight) */
  activePaneId: string | null;
  /** WebSocket 连接状态 (per P1-C WS 协议) */
  wsConnected: boolean;
  /** 是否正在 drag-resize divider */
  draggingDividerId: string | null;
}

export interface TerminalStackActions {
  setTree: (tree: SplitTreeView) => void;
  splitCurrentPane: (direction: SplitDirection) => void;
  splitPaneById: (paneId: string, direction: SplitDirection) => void;
  closePaneById: (paneId: string) => void;
  setActivePane: (paneId: string | null) => void;
  setWsConnected: (connected: boolean) => void;
  setDraggingDividerId: (id: string | null) => void;
  reset: () => void;
}

const INITIAL_ROOT_ID = "pane-root";

const INITIAL_STATE: TerminalStackState = {
  tree: newSingleTree(INITIAL_ROOT_ID, "root"),
  activePaneId: INITIAL_ROOT_ID,
  wsConnected: false,
  draggingDividerId: null,
};

export const useTerminalStackStore = create<TerminalStackState & TerminalStackActions>(
  (set, get) => ({
    ...INITIAL_STATE,

    setTree: (tree) => set({ tree }),

    splitCurrentPane: (direction) => {
      const { activePaneId, tree } = get();
      if (!activePaneId) return;
      const next = splitPane(tree, activePaneId, direction);
      set({ tree: next });
    },

    splitPaneById: (paneId, direction) => {
      const { tree } = get();
      const next = splitPane(tree, paneId, direction);
      set({ tree: next });
    },

    closePaneById: (paneId) => {
      // MVP v0: 只支持 close leaf pane (留 P1 处理 split 节点的 collapse)
      const { tree } = get();
      if (tree.paneCount <= 1) return;
      const next = closeLeafPane(tree, paneId);
      if (next) set({ tree: next });
    },

    setActivePane: (paneId) => set({ activePaneId: paneId }),
    setWsConnected: (connected) => set({ wsConnected: connected }),
    setDraggingDividerId: (id) => set({ draggingDividerId: id }),

    reset: () => set({ ...INITIAL_STATE }),
  }),
);

// =====================================================================
// 内部 helpers
// =====================================================================

/** Close a leaf pane; returns null if paneId not found or tree would be empty */
function closeLeafPane(
  tree: SplitTreeView,
  paneId: string,
): SplitTreeView | null {
  const pane = findPane(tree, paneId);
  if (!pane) return null;
  // MVP v0: 简单 remove leaf (不处理 collapse split parent)
  return removeLeaf(tree, paneId);
}

function removeLeaf(tree: SplitTreeView, paneId: string): SplitTreeView | null {
  const result = removeLeafRecursive(tree.root, paneId);
  if (!result) return null;
  return {
    root: result,
    paneCount: tree.paneCount - 1,
    depth: tree.depth, // 简化: 不重算 depth (UI 层容忍)
    kind: result.kind === "split" ? "split" : "single",
  };
}

function removeLeafRecursive(
  node: PaneNodeView,
  paneId: string,
): PaneNodeView | null {
  if (node.kind === "pane" && node.pane.id === paneId) {
    return null; // 删除该节点 (由 caller 决定如何 collapse)
  }
  if (node.kind === "split") {
    const newChildren: PaneNodeView[] = [];
    let removed = false;
    for (const c of node.children) {
      if (c.kind === "pane" && c.pane.id === paneId) {
        removed = true;
      } else if (removed === false && c.kind === "pane") {
        newChildren.push(c);
      } else if (removed) {
        newChildren.push(c);
      } else {
        newChildren.push(c);
      }
    }
    if (removed) {
      // 简化: 如果 split 只剩 1 个 child, 降级为 pane
      if (newChildren.length === 1) {
        return newChildren[0];
      }
      return { ...node, children: newChildren };
    }
    // 没找到, 递归到 children
    let foundRemoved = false;
    const recursed: PaneNodeView[] = [];
    for (const c of node.children) {
      if (c.kind === "split") {
        const inner = removeLeafRecursive(c, paneId);
        if (inner === null) {
          foundRemoved = true;
        } else {
          recursed.push(inner);
        }
      } else {
        recursed.push(c);
      }
    }
    if (foundRemoved) {
      if (recursed.length === 1) {
        return recursed[0];
      }
      return { ...node, children: recursed };
    }
    return node;
  }
  return node;
}