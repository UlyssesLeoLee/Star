"use client";

// =====================================================================
// TerminalStackContainer.tsx — 主入口 (per ULYS-223 P1-D + ULYS-222 P1-C 集成)
// =====================================================================
// 集成:
//   - TerminalSplitToolbar (操作 UI)
//   - TerminalSplitPane (SplitTree 渲染)
//   - useTerminalStackWs hook (PR #98.5: 真实 WS 接入 PR #106 协议)
//   - 简化版 xterm container (per pane, MVP 占位)
// =====================================================================
// P1-C 后端 WS push SplitUpdate 时, wsClient dispatch → setTree 自动重渲
// =====================================================================

import { TerminalSplitToolbar } from "./TerminalSplitToolbar";
import { TerminalSplitPane } from "./TerminalSplitPane";
import { useTerminalStackStore } from "./terminalStackStore";
import { useTerminalStackWs } from "@/hooks/useTerminalStackWs";
import { type ReactNode, useEffect } from "react";

export interface TerminalStackContainerProps {
  /** Optional pane renderer (default: 占位 div + pane id) */
  renderPane?: (paneId: string, isActive: boolean) => ReactNode;
  /**
   * WebSocket session id (per PR #106 server pane id).
   * - null / undefined → mock mode (per TerminalStackContainer.tsx PR #98 默认行为)
   * - string → 真实 WS 接入 (per PR #98.5 wsClient)
   */
  sessionId?: string | null;
}

export function TerminalStackContainer({
  renderPane,
  sessionId,
}: TerminalStackContainerProps) {
  const tree = useTerminalStackStore((s) => s.tree);
  const wsConnected = useTerminalStackStore((s) => s.wsConnected);
  const setWsConnected = useTerminalStackStore((s) => s.setWsConnected);

  // PR #98.5: 真实 WS 接入 (per PR #106 协议 + wsClient.ts)
  const ws = useTerminalStackWs({ sessionId: sessionId ?? null });
  // mock mode 行为 (per PR #98): 无 sessionId → setWsConnected(true)
  useEffect(() => {
    if (!sessionId) setWsConnected(true);
  }, [sessionId, setWsConnected]);

  const defaultRenderPane = (paneId: string, _isActive: boolean): ReactNode => (
    <div
      className="terminal-pane-placeholder"
      data-testid={`pane-placeholder-${paneId}`}
    >
      <span className="terminal-pane-id">{paneId}</span>
      <span className="terminal-pane-hint">
        xterm.js + addon-fit terminal container (MVP placeholder)
      </span>
    </div>
  );

  return (
    <div className="terminal-stack-container" data-testid="terminal-stack-container">
      <TerminalSplitToolbar />
      <TerminalSplitPane tree={tree} renderPane={renderPane ?? defaultRenderPane} />
      {/* PR #98.5: ws sendStdin / ws sendResize 暴露给 caller via ws return */}
      {/* MVP v0: 由 caller 在 P1-C 接通后通过 ws.sendStdin / ws.sendResize 接入 */}
      <div data-testid="ws-debug" hidden>
        connected={String(wsConnected)}
      </div>
    </div>
  );
}