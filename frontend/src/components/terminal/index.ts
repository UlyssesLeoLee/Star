"use client";

// =====================================================================
// index.ts — public exports (per ULYS-223 P1-D public API)
// =====================================================================

export { TerminalStackContainer } from "./TerminalStackContainer";
export { TerminalSplitPane } from "./TerminalSplitPane";
export { TerminalSplitToolbar } from "./TerminalSplitToolbar";
export { SplitDivider } from "./SplitDivider";
export { useTerminalStackStore } from "./terminalStackStore";

export type {
  SplitDirection,
  SplitPaneView,
  PaneNodeView,
  SplitTreeKind,
  SplitTreeView,
} from "./types";

export type { TerminalStackState, TerminalStackActions } from "./terminalStackStore";