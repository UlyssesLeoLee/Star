/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/components/terminal/TerminalStackContainer.tsx",type:"file",language:"typescript"}),
  (props:Class {name:"TerminalStackContainerProps",type:"class",language:"typescript"}),
  (container:Function {name:"TerminalStackContainer",type:"function",language:"typescript",signature:"TerminalStackContainer(props): JSX.Element"}),(terminalPane:Function {name:"InteractiveTaskTerminalPane",type:"function",language:"typescript"}),
  (render:Function {name:"defaultRenderPane",type:"function",language:"typescript"}),
  (file)-[:CONTAINS]->(props),(file)-[:CONTAINS]->(container),(file)-[:CONTAINS]->(render),(file)-[:CONTAINS]->(terminalPane),(container)-[:CONTAINS]->(render),(container)-[:CALLS]->(render),(render)-[:CALLS]->(terminalPane);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (props:Class {name:"TerminalStackContainerProps"}),
      (container:Function {name:"TerminalStackContainer"}),
      (hook:Function {name:"useTerminalStackWs"}),
      (connectionChange:Function {name:"handleTaskCliConnectionChange"});
CREATE (onConnectionChange:Variable {name:"TerminalStackContainerProps.onConnectionChange",type:"variable",language:"typescript"});
CREATE (props)-[:HAS_FIELD]->(onConnectionChange),
       (container)-[:CALLS]->(hook),
       (container)-[:USES]->(onConnectionChange),
       (connectionChange)-[:PASSES_TO]->(onConnectionChange);
*/

"use client";

// =====================================================================
// TerminalStackContainer.tsx — 主入口 (per ULYS-223 P1-D + ULYS-222 P1-C 集成)
// =====================================================================
// 集成:
//   - TerminalSplitToolbar (操作 UI)
//   - TerminalSplitPane (SplitTree 渲染)
//   - useTerminalStackWs hook (PR #98.5: 真实 WS 接入 PR #106 协议)
//   - authorized session uses xterm panes; preview mode keeps a disconnected placeholder
// =====================================================================
// P1-C 后端 WS push SplitUpdate 时, wsClient dispatch → setTree 自动重渲
// =====================================================================

import { TerminalSplitToolbar } from "./TerminalSplitToolbar";
import { TerminalSplitPane } from "./TerminalSplitPane";
import { useTerminalStackStore } from "./terminalStackStore";
import { useTerminalStackWs } from "@/hooks/useTerminalStackWs";
import { useEffect, useRef, type ReactNode } from "react";
import type { Terminal as XtermTerminal } from "@xterm/xterm";

export interface TerminalStackContainerProps {
  /** Optional pane renderer (default: 占位 div + pane id) */
  renderPane?: (paneId: string, isActive: boolean) => ReactNode;
  /**
   * WebSocket session id (per PR #106 server pane id).
   * - null / undefined → preview placeholder, no active connection
   * - string → 真实 WS 接入 (per PR #98.5 wsClient)
   */
  sessionId?: string | null;
  /** Fresh single-use ticket provider; required when attaching through the protected Group route. */
  getAttachmentTicket?: () => Promise<string>;
  /** Reports authorized attachment success and terminal connection loss to the owner. */
  onConnectionChange?: (connected: boolean) => void;
  /** Task CLI uses explicit REST reattachment; automatic reconnect remains disabled. */
  autoReconnect?: boolean;
  /** Task Card CLI has one server-owned PTY pane until each split is backed by its own session. */
  allowSplit?: boolean;
}

export function TerminalStackContainer({
  renderPane,
  sessionId,
  getAttachmentTicket,
  onConnectionChange,
  autoReconnect = true,
  allowSplit = true,
}: TerminalStackContainerProps) {
  const tree = useTerminalStackStore((s) => s.tree);

  // PR #98.5: 真实 WS 接入 (per PR #106 协议 + wsClient.ts)
  const ws = useTerminalStackWs({
    sessionId: sessionId ?? null,
    getAttachmentTicket,
    autoReconnect,
    onConnectionChange,
  });

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

  const renderInteractivePane = (paneId: string): ReactNode => (
    <InteractiveTaskTerminalPane
      paneId={paneId}
      output={ws.outputByPane[paneId] ?? ""}
      connected={ws.isConnected()}
      onInput={ws.sendStdin}
      onResize={ws.sendResize}
    />
  );

  return (
    <div className="terminal-stack-container" data-testid="terminal-stack-container">
      {allowSplit && <TerminalSplitToolbar />}
      <TerminalSplitPane
        tree={tree}
        renderPane={sessionId ? (paneId) => renderInteractivePane(paneId) : renderPane ?? defaultRenderPane}
      />
      {/* PR #98.5: ws sendStdin / ws sendResize 暴露给 caller via ws return */}
      {/* Placeholder panes remain disconnected until an authorized session is supplied. */}
      <div data-testid="ws-debug" hidden>
        connected={String(ws.isConnected())}
      </div>
    </div>
  );
}

function InteractiveTaskTerminalPane({
  paneId,
  output,
  connected,
  onInput,
  onResize,
}: {
  paneId: string;
  output: string;
  connected: boolean;
  onInput: (data: string) => void;
  onResize: (cols: number, rows: number) => void;
}) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<XtermTerminal | null>(null);
  const previousOutputRef = useRef("");

  useEffect(() => {
    let disposed = false;
    let resizeObserver: ResizeObserver | undefined;
    let inputSubscription: { dispose: () => void } | undefined;
    let resizeSubscription: { dispose: () => void } | undefined;
    let terminal: XtermTerminal | undefined;

    void Promise.all([import("@xterm/xterm"), import("@xterm/addon-fit")]).then(
      ([{ Terminal }, { FitAddon }]) => {
        if (disposed || !containerRef.current) return;
        terminal = new Terminal({
          fontFamily: 'ui-monospace, "JetBrains Mono", Menlo, monospace',
          fontSize: 12,
          cursorBlink: true,
          convertEol: true,
          disableStdin: !connected,
          theme: { background: "#0d1117", foreground: "#d1fae5", cursor: "#34d399" },
        });
        const fit = new FitAddon();
        terminal.loadAddon(fit);
        terminal.open(containerRef.current);
        terminalRef.current = terminal;
        fit.fit();
        onResize(terminal.cols, terminal.rows);
        inputSubscription = terminal.onData(onInput);
        resizeSubscription = terminal.onResize(({ cols, rows }) => onResize(cols, rows));
        resizeObserver = new ResizeObserver(() => {
          try {
            fit.fit();
          } catch {
            // The pane may be between layout states while its parent is resizing.
          }
        });
        resizeObserver.observe(containerRef.current);
      },
    );

    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      inputSubscription?.dispose();
      resizeSubscription?.dispose();
      terminal?.dispose();
      terminalRef.current = null;
      previousOutputRef.current = "";
    };
  }, [onInput, onResize]);

  useEffect(() => {
    const terminal = terminalRef.current;
    if (!terminal) return;
    terminal.options.disableStdin = !connected;
    if (connected) onResize(terminal.cols, terminal.rows);
    const previous = previousOutputRef.current;
    if (output.startsWith(previous)) {
      terminal.write(output.slice(previous.length));
    } else {
      terminal.reset();
      terminal.write(output);
    }
    previousOutputRef.current = output;
  }, [connected, onResize, output]);

  return (
    <div className="relative flex h-full min-h-48 flex-col bg-[#0d1117]" data-testid={`task-cli-terminal-${paneId}`}>
      <div ref={containerRef} className="min-h-0 flex-1" />
      {!connected && (
        <div className="absolute inset-0 grid place-items-center bg-[#0d1117]/90 text-xs text-ink-mute">
          等待 ticket 授权与终端连接；连接前不接收输入
        </div>
      )}
    </div>
  );
}
