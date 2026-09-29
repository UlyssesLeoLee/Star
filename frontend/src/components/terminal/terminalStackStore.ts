/*
CYPHER STRUCTURE MANIFEST
CREATE
  (f:File {name:"frontend/src/components/terminal/terminalStackStore.ts",type:"file",language:"typescript"}),
  (state:Class {name:"TerminalStackState",type:"class",language:"typescript"}),
  (actions:Class {name:"TerminalStackActions",type:"class",language:"typescript"}),
  (root:Variable {name:"INITIAL_ROOT_ID",type:"variable",language:"typescript"}),
  (initial:Variable {name:"INITIAL_STATE",type:"variable",language:"typescript"}),
  (store:Variable {name:"useTerminalStackStore",type:"variable",language:"typescript"}),
  (create:Function {name:"zustand.create",type:"function",language:"typescript"}),
  (new_tree:Function {name:"newSingleTree",type:"function",language:"typescript"}),
  (split:Function {name:"splitPane",type:"function",language:"typescript"}),
  (find:Function {name:"findPane",type:"function",language:"typescript"}),
  (set_tree:Function {name:"setTree",type:"function",language:"typescript"}),
  (close:Function {name:"closeLeafPane",type:"function",language:"typescript"}),
  (remove:Function {name:"removeLeaf",type:"function",language:"typescript"}),
  (remove_recursive:Function {name:"removeLeafRecursive",type:"function",language:"typescript"}),
  (split_current:Function {name:"splitCurrentPane",type:"function",language:"typescript"}),
  (split_by_id:Function {name:"splitPaneById",type:"function",language:"typescript"}),
  (close_by_id:Function {name:"closePaneById",type:"function",language:"typescript"}),
  (set_active:Function {name:"setActivePane",type:"function",language:"typescript"}),
  (set_connected:Function {name:"setWsConnected",type:"function",language:"typescript"}),
  (set_dragging:Function {name:"setDraggingDividerId",type:"function",language:"typescript"}),
  (reset:Function {name:"reset",type:"function",language:"typescript"}),
  (f)-[:CONTAINS]->(state),(f)-[:CONTAINS]->(actions),(f)-[:CONTAINS]->(root),(f)-[:CONTAINS]->(initial),(f)-[:CONTAINS]->(store),(f)-[:CONTAINS]->(set_tree),(f)-[:CONTAINS]->(close),(f)-[:CONTAINS]->(remove),(f)-[:CONTAINS]->(remove_recursive),
  (actions)-[:HAS_METHOD]->(set_tree),(actions)-[:HAS_METHOD]->(split_current),(actions)-[:HAS_METHOD]->(split_by_id),(actions)-[:HAS_METHOD]->(close_by_id),(actions)-[:HAS_METHOD]->(set_active),(actions)-[:HAS_METHOD]->(set_connected),(actions)-[:HAS_METHOD]->(set_dragging),(actions)-[:HAS_METHOD]->(reset),
  (store)-[:CALLS]->(create),(store)-[:USES]->(initial),(initial)-[:CALLS]->(new_tree),(initial)-[:USES]->(root),
  (split_current)-[:CALLS]->(split),(split_by_id)-[:CALLS]->(split),(close_by_id)-[:CALLS]->(close),(close)-[:CALLS]->(remove),(close)-[:CALLS]->(find),(remove)-[:CALLS]->(remove_recursive),(remove_recursive)-[:CALLS]->(remove_recursive);
*/

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
  type PaneNodeView,
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
  /** Number of leaves in `tree` (computed: mirrors tree.paneCount for direct store access) */
  paneCount: number;
  /** Tree depth (computed: mirrors tree.depth for direct store access) */
  depth: number;
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
  paneCount: 1,
  depth: 0,
};

export const useTerminalStackStore = create<TerminalStackState & TerminalStackActions>(
  (set, get) => ({
    ...INITIAL_STATE,

    setTree: (tree) => set({ tree, paneCount: tree.paneCount, depth: tree.depth }),

    splitCurrentPane: (direction) => {
      const { activePaneId, tree } = get();
      if (!activePaneId) return;
      const next = splitPane(tree, activePaneId, direction);
      set({ tree: next, paneCount: next.paneCount, depth: next.depth });
    },

    splitPaneById: (paneId, direction) => {
      const { tree } = get();
      const next = splitPane(tree, paneId, direction);
      set({ tree: next, paneCount: next.paneCount, depth: next.depth });
    },

    closePaneById: (paneId) => {
      // MVP v0: 只支持 close leaf pane (留 P1 处理 split 节点的 collapse)
      const { tree } = get();
      if (tree.paneCount <= 1) return;
      const next = closeLeafPane(tree, paneId);
      if (next) set({ tree: next, paneCount: next.paneCount, depth: next.depth });
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
