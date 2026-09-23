// =====================================================================
// terminalStackStore.test.ts — zustand store + tree 操作 (per ULYS-223 P1-D §6.8)
// =====================================================================
// 守门: 6+ 组件单测 (per description §3.8)
// =====================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { useTerminalStackStore } from "./terminalStackStore";
import { newSingleTree, splitPane, countLeaves } from "./types";

describe("terminalStackStore (ULYS-223 P1-D)", () => {
  beforeEach(() => {
    useTerminalStackStore.getState().reset();
  });

  it("A. initial state is single root pane", () => {
    const { tree, activePaneId, paneCount } = useTerminalStackStore.getState();
    expect(paneCount).toBe(1);
    expect(tree.kind).toBe("single");
    expect(activePaneId).toBe("pane-root");
  });

  it("B. splitCurrentPane(vertical) creates 2 panes with split direction", () => {
    const { splitCurrentPane } = useTerminalStackStore.getState();
    splitCurrentPane("vertical");
    const { tree, paneCount } = useTerminalStackStore.getState();
    expect(paneCount).toBe(2);
    expect(tree.kind).toBe("split");
    expect(countLeaves(tree)).toBe(2);
  });

  it("C. splitCurrentPane(horizontal) creates 2 panes with split direction", () => {
    const { splitCurrentPane } = useTerminalStackStore.getState();
    splitCurrentPane("horizontal");
    const { tree } = useTerminalStackStore.getState();
    expect(tree.kind).toBe("split");
    if (tree.root.kind === "split") {
      expect(tree.root.direction).toBe("horizontal");
    } else {
      throw new Error("expected split root");
    }
  });

  it("D. split 3 times yields 4 leaves (depth 2)", () => {
    const s = useTerminalStackStore.getState();
    s.splitCurrentPane("vertical"); // 1 → 2
    s.splitCurrentPane("horizontal"); // activePaneId still pane-root → split again
    s.splitCurrentPane("horizontal"); // 2 → 4
    const { tree, depth } = useTerminalStackStore.getState();
    expect(countLeaves(tree)).toBe(4);
    expect(depth).toBe(2);
  });

  it("E. setActivePane changes active pane id", () => {
    const { splitCurrentPane, setActivePane } = useTerminalStackStore.getState();
    splitCurrentPane("vertical");
    const { tree } = useTerminalStackStore.getState();
    if (tree.root.kind !== "split") throw new Error("expected split");
    const newPaneId = tree.root.children[1].kind === "pane" ? tree.root.children[1].pane.id : "";
    setActivePane(newPaneId);
    expect(useTerminalStackStore.getState().activePaneId).toBe(newPaneId);
  });

  it("F. closePaneById removes leaf and updates paneCount", () => {
    const s = useTerminalStackStore.getState();
    s.splitCurrentPane("vertical");
    expect(useTerminalStackStore.getState().paneCount).toBe(2);
    const { tree } = useTerminalStackStore.getState();
    if (tree.root.kind !== "split") throw new Error("expected split");
    const leafId = tree.root.children[1].kind === "pane" ? tree.root.children[1].pane.id : "";
    s.closePaneById(leafId);
    expect(useTerminalStackStore.getState().paneCount).toBe(1);
  });

  it("G. closePaneById refuses to remove last pane", () => {
    const { closePaneById } = useTerminalStackStore.getState();
    closePaneById("pane-root");
    expect(useTerminalStackStore.getState().paneCount).toBe(1);
    expect(useTerminalStackStore.getState().tree.kind).toBe("single");
  });

  it("H. reset restores initial state", () => {
    const s = useTerminalStackStore.getState();
    s.splitCurrentPane("vertical");
    s.splitCurrentPane("horizontal");
    expect(useTerminalStackStore.getState().paneCount).toBe(3);
    s.reset();
    const after = useTerminalStackStore.getState();
    expect(after.paneCount).toBe(1);
    expect(after.tree.kind).toBe("single");
  });
});

describe("types.ts pure helpers (ULYS-223 P1-D)", () => {
  it("I. splitPane throws on missing pane id", () => {
    const tree = newSingleTree("a");
    expect(() => splitPane(tree, "nonexistent", "vertical")).toThrow(/pane not found/);
  });

  it("J. countLeaves equals 1 for new single tree", () => {
    const tree = newSingleTree("a");
    expect(countLeaves(tree)).toBe(1);
  });
});