"use client";

// =====================================================================
// useTerminalStackWs.ts — React hook (per ULYS-222 P1-C + ULYS-223 P1-D 集成)
// (per PR #98.5 修复 PR #98 ↔ PR #106 schema 不对齐)
// =====================================================================
// 流程:
//  1. useEffect: 创建 TerminalWsClient + bind handlers
//  2. onHello → setWsConnected(true)
//  3. onOutput → 暂存 (per pane_id) — T23 followup 接到 xterm container
//  4. onSplitUpdate → setTree (per PR #98 store.setTree)
//  5. onSnapshot → 暂存 — T23 followup 触发 initial mount
//  6. cleanup: close WS + reset store
// =====================================================================

import { useEffect, useRef } from "react";
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
  /** session id (per server pane id) — null = mock mode (no WS) */
  sessionId: string | null;
  /** Auto-reconnect after disconnect (per NFR-AC keepalive) */
  autoReconnect?: boolean;
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

  useEffect(() => {
    if (!opts.sessionId) {
      // Mock mode (per TerminalStackContainer.tsx PR #98 默认行为)
      setWsConnected(true);
      return;
    }

    // T23.6: 测试 can inject a custom WebSocket constructor via window.__mockWsCtor
    // (per Playwright addInitScript before page.goto). This avoids needing to
    // override window.WebSocket which Chromium does not allow.
    const WebSocketCtor =
      typeof window !== "undefined"
        ? ((window as unknown as { __mockWsCtor?: typeof WebSocket })
            .__mockWsCtor as typeof WebSocket | undefined)
        : undefined;

    const client = new TerminalWsClient({
      sessionId: opts.sessionId,
      autoReconnect: opts.autoReconnect ?? true,
      WebSocketCtor,
      handlers: {
        onHello: () => {
          setWsConnected(true);
        },
        onSnapshot: (msg) => {
          // Snapshot 内的 pane + lines 由 T23 followup 接到 xterm container
          // MVP v0 仅记录, 暂不写 store (per pane_id → scrollback buffer 留 P1)
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const _lines: ScrollbackLine[] = msg.lines;
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const _paneId: string = msg.pane_id;
          // T23 followup will hook this up to a per-pane scrollback store
        },
        onSplitUpdate: (msg) => {
          // msg.tree is PaneNodeView (1:1 mirror of SplitTree)
          // We need to wrap in a SplitTreeView
          const wrapped: SplitTreeView = {
            root: msg.root.kind === "split" ? msg.root : msg.root,
            paneCount: countPanes(msg.root),
            depth: computeDepth(msg.root),
            kind: msg.root.kind === "split" ? "split" : "single",
          };
          setTree(wrapped);
        },
        onConnectionChange: (connected) => {
          setWsConnected(connected);
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
    };
  }, [opts.sessionId, opts.autoReconnect, setWsConnected, setTree, reset]);

  // Return send helpers as a stable API
  return {
    sendStdin: (data: string) => clientRef.current?.sendStdin(data),
    sendResize: (cols: number, rows: number) =>
      clientRef.current?.sendResize(cols, rows),
    isConnected: () => clientRef.current?.isConnected() ?? false,
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