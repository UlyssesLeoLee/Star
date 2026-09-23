"use client";

// =====================================================================
// TerminalStackContainer.tsx — 主入口 (per ULYS-223 P1-D 验收)
// =====================================================================
// 集成:
//   - TerminalSplitToolbar (操作 UI)
//   - TerminalSplitPane (SplitTree 渲染)
//   - 简化版 xterm container (per pane, MVP 占位)
// =====================================================================
// P1-C 后端 WS push SplitUpdate 时, 接 server 替换 mock pane 渲染即可
// =====================================================================

import { TerminalSplitToolbar } from "./TerminalSplitToolbar";
import { TerminalSplitPane } from "./TerminalSplitPane";
import { useTerminalStackStore } from "./terminalStackStore";
import { type ReactNode, useEffect } from "react";

export interface TerminalStackContainerProps {
  /** Optional pane renderer (default: 占位 div + pane id) */
  renderPane?: (paneId: string, isActive: boolean) => ReactNode;
  /** WebSocket URL (per P1-C, MVP v0: unused, mock mode) */
  wsUrl?: string;
}

export function TerminalStackContainer({
  renderPane,
  wsUrl,
}: TerminalStackContainerProps) {
  const tree = useTerminalStackStore((s) => s.tree);
  const setWsConnected = useTerminalStackStore((s) => s.setWsConnected);

  // MVP v0: 模拟 WS 状态 (P1-C 接入后真实连接)
  useEffect(() => {
    if (wsUrl) {
      // 真实模式: 占位 (P1-C 实现)
      setWsConnected(false);
    } else {
      setWsConnected(true); // mock mode
    }
  }, [wsUrl, setWsConnected]);

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
    </div>
  );
}