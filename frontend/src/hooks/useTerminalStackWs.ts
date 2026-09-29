/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/hooks/useTerminalStackWs.ts",type:"file",language:"typescript"}),
  (options:Class {name:"UseTerminalStackWsOptions",type:"class",language:"typescript"}),
  (hook:Function {name:"useTerminalStackWs",type:"function",language:"typescript",signature:"useTerminalStackWs(opts): TerminalWsControls"}),
  (count:Function {name:"countPanes",type:"function",language:"typescript"}),
  (depth:Function {name:"computeDepth",type:"function",language:"typescript"}),
  (file)-[:CONTAINS]->(options),(file)-[:CONTAINS]->(hook),(file)-[:CONTAINS]->(count),(file)-[:CONTAINS]->(depth),
  (hook)-[:CALLS]->(count),(hook)-[:CALLS]->(depth);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/hooks/useTerminalStackWs.ts"}),
      (hook:Function {name:"useTerminalStackWs"});
CREATE (output:Variable {name:"outputByPane",type:"variable",language:"typescript"}),
       (appendOutput:Function {name:"appendTerminalOutput",type:"function",language:"typescript"}),
       (maxOutput:Variable {name:"MAX_RENDERED_OUTPUT_CHARS",type:"variable",language:"typescript"});
CREATE (file)-[:CONTAINS]->(output),
       (file)-[:CONTAINS]->(appendOutput),
       (file)-[:CONTAINS]->(maxOutput),
       (hook)-[:USES]->(output),
       (hook)-[:CALLS]->(appendOutput),
       (appendOutput)-[:USES]->(maxOutput);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (hook:Function {name:"useTerminalStackWs"}),
      (options:Class {name:"UseTerminalStackWsOptions"});
CREATE (onConnectionChange:Variable {name:"UseTerminalStackWsOptions.onConnectionChange",type:"variable",language:"typescript"});
CREATE (options)-[:HAS_FIELD]->(onConnectionChange),
       (hook)-[:CALLS]->(onConnectionChange);
*/

"use client";

// =====================================================================
// useTerminalStackWs.ts — React hook (per ULYS-222 P1-C + ULYS-223 P1-D 集成)
// (per PR #98.5 修复 PR #98 ↔ PR #106 schema 不对齐)
// =====================================================================
// 流程:
//  1. useEffect: 创建 TerminalWsClient + bind handlers
//  2. onHello → setWsConnected(true)
//  3. onOutput → 写入有界 per-pane output projection，由授权 xterm pane 渲染
//  4. onSplitUpdate → setTree (per PR #98 store.setTree)
//  5. onSnapshot → 替换该 pane 的恢复输出，再应用后续增量
//  6. cleanup: close WS + reset store
// =====================================================================

import { useCallback, useEffect, useRef, useState } from "react";
import { TerminalWsClient } from "@/lib/terminal/wsClient";
import type {
  ScrollbackLine,
  ServerMessage,
} from "@/lib/terminal/wsProtocol";
import {
  newSingleTree,
  type SplitTreeView,
} from "@/components/terminal/types";
import { useTerminalStackStore } from "@/components/terminal/terminalStackStore";

export interface UseTerminalStackWsOptions {
  /** Session id; null means preview placeholder mode without a WebSocket. */
  sessionId: string | null;
  /** Obtain a fresh, one-use attachment ticket before every socket connection. */
  getAttachmentTicket?: () => Promise<string>;
  /** Auto-reconnect after disconnect (per NFR-AC keepalive) */
  autoReconnect?: boolean;
  /** Reports authorized attachment success and terminal connection loss to the owner. */
  onConnectionChange?: (connected: boolean) => void;
}

const MAX_RENDERED_OUTPUT_CHARS = 1_000_000;

function appendTerminalOutput(current: string, next: string): string {
  const combined = current + next;
  return combined.length > MAX_RENDERED_OUTPUT_CHARS
    ? combined.slice(-MAX_RENDERED_OUTPUT_CHARS)
    : combined;
}

/**
 * React hook: wire TerminalWsClient to zustand store.
 *
 * Returns a stable callback for sending client messages (stdin/resize/ping).
 */
export function useTerminalStackWs(opts: UseTerminalStackWsOptions) {
  const clientRef = useRef<TerminalWsClient | null>(null);
  const setWsConnected = useTerminalStackStore((s) => s.setWsConnected);
  const setTree = useTerminalStackStore((s) => s.setTree);
  const reset = useTerminalStackStore((s) => s.reset);
  const [outputByPane, setOutputByPane] = useState<Record<string, string>>({});
  const sendStdin = useCallback((data: string) => clientRef.current?.sendStdin(data), []);
  const sendResize = useCallback(
    (cols: number, rows: number) => clientRef.current?.sendResize(cols, rows),
    [],
  );
  const isConnected = useCallback(() => clientRef.current?.isConnected() ?? false, []);

  useEffect(() => {
    if (!opts.sessionId) {
      // A missing session is a preview/placeholder, never a live connection.
      setWsConnected(false);
      setOutputByPane({});
      return;
    }

    const client = new TerminalWsClient({
      sessionId: opts.sessionId,
      getAttachmentTicket: opts.getAttachmentTicket,
      autoReconnect: opts.autoReconnect ?? true,
      handlers: {
        onHello: (msg) => {
          setWsConnected(true);
          if (msg.panes.length === 1) {
            setTree(newSingleTree(msg.panes[0], "Task CLI"));
          }
        },
        onSnapshot: (msg) => {
          const snapshot = msg.lines.map((line: ScrollbackLine) => line.text).join("\r\n");
          setOutputByPane((current) => ({ ...current, [msg.pane_id]: snapshot }));
        },
        onOutput: (msg) => {
          setOutputByPane((current) => ({
            ...current,
            [msg.pane_id]: appendTerminalOutput(current[msg.pane_id] ?? "", msg.data),
          }));
        },
        onSplitUpdate: (msg) => {
          // msg.tree is PaneNodeView (1:1 mirror of SplitTree).
          // Per wsProtocol.ts: split_update has `tree:` (not `root:`) field.
          const treeNode = msg.tree;
          const wrapped: SplitTreeView = {
            root: treeNode,
            paneCount: countPanes(treeNode),
            depth: computeDepth(treeNode),
            kind: treeNode.kind === "split" ? "split" : "single",
          };
          setTree(wrapped);
        },
        onConnectionChange: (connected) => {
          setWsConnected(connected);
          opts.onConnectionChange?.(connected);
        },
        onError: (msg: Extract<ServerMessage, { type: "error" }>) => {
          // eslint-disable-next-line no-console
          console.warn(
            `[terminal-ws] server error ${msg.code}: ${msg.message}`,
          );
        },
      },
      });

    clientRef.current = client;
    client.connect();

    return () => {
      client.close();
      clientRef.current = null;
      reset();
      setWsConnected(false);
      setOutputByPane({});
    };
  }, [opts.sessionId, opts.getAttachmentTicket, opts.autoReconnect, opts.onConnectionChange, setWsConnected, setTree, reset]);

  // Return send helpers as a stable API
  return {
    sendStdin,
    sendResize,
    isConnected,
    outputByPane,
  };
}

// =====================================================================
// helpers
// =====================================================================

import type { PaneNodeView } from "@/components/terminal/types";

function countPanes(node: PaneNodeView): number {
  if (node.kind === "pane") return 1;
  return node.children.reduce((acc, c) => acc + countPanes(c), 0);
}

function computeDepth(node: PaneNodeView, depth = 0): number {
  if (node.kind === "pane") return depth;
  return Math.max(...node.children.map((c) => computeDepth(c, depth + 1)));
}

// Suppress unused warnings on initialSingleTree re-export
export { newSingleTree };
