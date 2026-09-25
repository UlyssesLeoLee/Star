"use client";

// =====================================================================
// terminal-stack types.ts — TypeScript mirror of crates/terminal-stack/src/{scrollback_buffer.rs,split_pane.rs}
// (per ULYS-200 数据层 + ULYS-223 P1-D frontend 镜像)
// =====================================================================
// 守门: 1:1 对应 backend DTO, PaneNode 树结构严格保持 nested 关系
// =====================================================================

export type SplitDirection = "horizontal" | "vertical";

export interface SplitPaneView {
  id: string;
  ratio: number;
  title?: string | null;
}

export type PaneNodeView =
  | { kind: "split"; id: string; direction: SplitDirection; children: PaneNodeView[] }
  | { kind: "pane"; pane: SplitPaneView };

export type SplitTreeKind = "single" | "split";

export interface SplitTreeView {
  root: PaneNodeView;
  paneCount: number;
  depth: number;
  kind: SplitTreeKind;
}

// =====================================================================
// builders (frontend 镜像 SplitTree::new_single / new_split / from_parts)
// =====================================================================

export function newSingleTree(rootPaneId: string, title?: string): SplitTreeView {
  return {
    root: {
      kind: "pane",
      pane: { id: rootPaneId, ratio: 1.0, title: title ?? null },
    },
    paneCount: 1,
    depth: 0,
    kind: "single",
  };
}

export function splitPane(
  tree: SplitTreeView,
  paneId: string,
  direction: SplitDirection,
  newPaneId: string = crypto.randomUUID(),
): SplitTreeView {
  // 找到 paneId 节点, 替换为 Split node
  const splitRoot = splitRecursive(tree.root, paneId, direction, newPaneId);
  if (splitRoot === tree.root) {
    // 未找到, 抛错或返回原 tree
    throw new Error(`pane not found: ${paneId}`);
  }
  // Recompute depth by walking the new tree (depth may increase by 0..N depending on where split happened)
  const newDepth = treeDepth({ root: splitRoot, paneCount: tree.paneCount + 1, depth: 0, kind: "split" });
  return {
    root: splitRoot,
    paneCount: tree.paneCount + 1,
    depth: newDepth,
    kind: "split",
  };
}

function splitRecursive(
  node: PaneNodeView,
  target: string,
  direction: SplitDirection,
  newPaneId: string,
): PaneNodeView {
  if (node.kind === "pane" && node.pane.id === target) {
    // 找到目标 pane, 替换为 Split node
    const a = { ...node.pane };
    const b: SplitPaneView = {
      id: newPaneId,
      ratio: 0.5,
      title: null,
    };
    return {
      kind: "split",
      id: crypto.randomUUID(),
      direction,
      children: [
        { kind: "pane", pane: a },
        { kind: "pane", pane: b },
      ],
    };
  }
  if (node.kind === "split") {
    return {
      ...node,
      children: node.children.map((c) => splitRecursive(c, target, direction, newPaneId)),
    };
  }
  return node;
}

// =====================================================================
// selector helpers
// =====================================================================

export function findPane(tree: SplitTreeView, paneId: string): SplitPaneView | null {
  const found = findPaneRecursive(tree.root, paneId);
  return found;
}

function findPaneRecursive(node: PaneNodeView, paneId: string): SplitPaneView | null {
  if (node.kind === "pane" && node.pane.id === paneId) {
    return node.pane;
  }
  if (node.kind === "split") {
    for (const child of node.children) {
      const r = findPaneRecursive(child, paneId);
      if (r) return r;
    }
  }
  return null;
}

export function countLeaves(tree: SplitTreeView): number {
  return countLeavesRecursive(tree.root);
}

function countLeavesRecursive(node: PaneNodeView): number {
  if (node.kind === "pane") return 1;
  return node.children.reduce((acc, c) => acc + countLeavesRecursive(c), 0);
}

export function treeDepth(tree: SplitTreeView): number {
  return depthRecursive(tree.root, 0);
}

function depthRecursive(node: PaneNodeView, depth: number): number {
  if (node.kind === "pane") return depth;
  return Math.max(...node.children.map((c) => depthRecursive(c, depth + 1)));
}