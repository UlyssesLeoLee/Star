"use client";

// =====================================================================
// TerminalSplitToolbar.tsx — Split 操作 UI (per ULYS-223 P1-D §3.6)
// =====================================================================
// "Split horizontally" / "Split vertically" 按钮 (per active pane)
// =====================================================================

import { useTerminalStackStore } from "./terminalStackStore";

export function TerminalSplitToolbar() {
  const activePaneId = useTerminalStackStore((s) => s.activePaneId);
  const split = useTerminalStackStore((s) => s.splitCurrentPane);
  const close = useTerminalStackStore((s) => s.closePaneById);
  const paneCount = useTerminalStackStore((s) => s.tree.paneCount);

  return (
    <div className="terminal-split-toolbar" data-testid="terminal-split-toolbar">
      <button
        type="button"
        onClick={() => split("vertical")}
        disabled={!activePaneId}
        data-testid="split-vertical-btn"
        aria-label="Split vertically"
      >
        ⬌ Split ↔
      </button>
      <button
        type="button"
        onClick={() => split("horizontal")}
        disabled={!activePaneId}
        data-testid="split-horizontal-btn"
        aria-label="Split horizontally"
      >
        ⬍ Split ⬍
      </button>
      <button
        type="button"
        onClick={() => activePaneId && close(activePaneId)}
        disabled={!activePaneId || paneCount <= 1}
        data-testid="close-pane-btn"
        aria-label="Close pane"
      >
        ✕ Close
      </button>
      <span className="pane-count" data-testid="pane-count">
        {paneCount} pane{paneCount !== 1 ? "s" : ""}
      </span>
    </div>
  );
}