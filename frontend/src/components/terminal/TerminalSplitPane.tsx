"use client";

// =====================================================================
// TerminalSplitPane.tsx — SplitTree React 渲染 (per ULYS-223 P1-D §3.1-3.2)
// =====================================================================
// 渲染 PaneNodeView 嵌套树:
//   - kind: "split" → 递归渲染 children + SplitDivider
//   - kind: "pane"  → 渲染 leaf pane (含 xterm container 占位, 由 caller 接 xterm.js)
// =====================================================================

import { type ReactNode } from "react";
import { type PaneNodeView, type SplitTreeView, type SplitDirection } from "./types";
import { SplitDivider } from "./SplitDivider";
import { useTerminalStackStore } from "./terminalStackStore";

export interface TerminalSplitPaneProps {
  tree: SplitTreeView;
  /** Per-leaf pane 渲染 callback (caller 注入 xterm.js 终端) */
  renderPane: (paneId: string, isActive: boolean) => ReactNode;
}

export function TerminalSplitPane({ tree, renderPane }: TerminalSplitPaneProps) {
  return (
    <div className="terminal-split-pane" data-testid="terminal-split-pane">
      <PaneNodeRenderer node={tree.root} renderPane={renderPane} />
    </div>
  );
}

interface PaneNodeRendererProps {
  node: PaneNodeView;
  renderPane: TerminalSplitPaneProps["renderPane"];
}

function PaneNodeRenderer({ node, renderPane }: PaneNodeRendererProps) {
  if (node.kind === "pane") {
    return (
      <PaneContainer
        paneId={node.pane.id}
        title={node.pane.title ?? undefined}
        renderPane={renderPane}
      />
    );
  }
  // split node → 递归 children + divider
  return (
    <SplitContainer direction={node.direction} splitId={node.id}>
      {node.children.map((child, idx) => (
        <SplitChildWrapper key={getChildKey(node, idx)} direction={node.direction}>
          <PaneNodeRenderer node={child} renderPane={renderPane} />
        </SplitChildWrapper>
      ))}
    </SplitContainer>
  );
}

function getChildKey(node: PaneNodeView & { kind: "split" }, idx: number): string {
  // idx 0 = before divider, idx 1 = after
  const child = node.children[idx];
  return child.kind === "pane" ? `pane-${child.pane.id}` : `split-${child.id}`;
}

interface SplitContainerProps {
  splitId: string;
  direction: SplitDirection;
  children: ReactNode;
}

function SplitContainer({ splitId, direction, children }: SplitContainerProps) {
  return (
    <div
      className={`split-container split-${direction}`}
      data-testid={`split-container-${splitId}`}
    >
      {children}
    </div>
  );
}

interface SplitChildWrapperProps {
  direction: SplitDirection;
  children: ReactNode;
}

/** Wraps a child + the divider that comes after it */
function SplitChildWrapper({ direction, children }: SplitChildWrapperProps) {
  return (
    <>
      <div className={`split-child split-child-${direction}`}>{children}</div>
      {/* Divider 紧跟 child 之后; 通过 :last-child CSS 处理末尾不渲染 */}
      <SplitDivider
        dividerId={`divider-after`}
        direction={direction}
        onResize={() => {
          // TODO: 实际触发 zustand store update (per backend ratio adjust)
          // MVP v0: 占位
        }}
        onDragStart={() => useTerminalStackStore.getState().setDraggingDividerId("")}
        onDragEnd={() => useTerminalStackStore.getState().setDraggingDividerId(null)}
      />
    </>
  );
}

interface PaneContainerProps {
  paneId: string;
  title?: string;
  renderPane: (paneId: string, isActive: boolean) => ReactNode;
}

function PaneContainer({ paneId, title, renderPane }: PaneContainerProps) {
  const isActive = useTerminalStackStore((s) => s.activePaneId === paneId);
  const setActive = useTerminalStackStore((s) => s.setActivePane);

  return (
    <div
      className={`terminal-pane ${isActive ? "active" : ""}`}
      data-testid={`terminal-pane-${paneId}`}
      data-active={isActive}
      onClick={() => setActive(paneId)}
      role="region"
      aria-label={title ? `Terminal ${title}` : "Terminal"}
    >
      {title && (
        <div className="terminal-pane-header" data-testid={`pane-header-${paneId}`}>
          {title}
        </div>
      )}
      <div className="terminal-pane-body" data-testid={`pane-body-${paneId}`}>
        {renderPane(paneId, isActive)}
      </div>
    </div>
  );
}