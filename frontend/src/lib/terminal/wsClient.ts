"use client";

// =====================================================================
// wsClient.ts — Terminal Stack WebSocket Client (per ULYS-222 P1-C PR #106)
// (per ULYS-200 P1-D + P1-C 集成, PR #98.5)
// =====================================================================
// 模式: 跟 frontend/src/lib/remote/wsClient.ts 同款 (per remote WS client 复用).
//
// 流程:
//  1. connect(sessionId) → new WebSocket(buildTerminalWsUrl)
//  2. ws.onopen → setWsConnected(true)
//  3. ws.onmessage → decode ServerMessage → dispatch to handlers
//  4. ws.onclose / onerror → setWsConnected(false) + retry (per NFR-AC)
//  5. send stdin / resize → encode ClientMessage → ws.send
// =====================================================================

import {
  type ClientMessage,
  type ServerMessage,
  buildTerminalWsUrl,
  decodeServerMessage,
  encodeClientMessage,
} from "./wsProtocol";

export type TerminalWsHandlers = {
  /** 收到 Hello (服务端 Connect ACK) */
  onHello?: (msg: Extract<ServerMessage, { type: "hello" }>) => void;
  /** 收到 Snapshot (全量或增量 scrollback) */
  onSnapshot?: (msg: Extract<ServerMessage, { type: "snapshot" }>) => void;
  /** 收到 Output (单条 scrollback 新行) */
  onOutput?: (msg: Extract<ServerMessage, { type: "output" }>) => void;
  /** 收到 ResizeAck (服务端 PTY resize 确认) */
  onResizeAck?: (msg: Extract<ServerMessage, { type: "resize_ack" }>) => void;
  /** 收到 SplitUpdate (SplitTree 变化) */
  onSplitUpdate?: (msg: Extract<ServerMessage, { type: "split_update" }>) => void;
  /** 收到 PtyExit (PTY 进程退出) */
  onPtyExit?: (msg: Extract<ServerMessage, { type: "pty_exit" }>) => void;
  /** 收到 Pong (keepalive) */
  onPong?: (msg: Extract<ServerMessage, { type: "pong" }>) => void;
  /** 收到 Error (服务端错误推送) */
  onError?: (msg: Extract<ServerMessage, { type: "error" }>) => void;
  /** WebSocket 连接状态变化 */
  onConnectionChange?: (connected: boolean) => void;
};

export interface TerminalWsClientOptions {
  sessionId: string;
  handlers: TerminalWsHandlers;
  /** Reconnect after disconnect (per NFR-AC keepalive) */
  autoReconnect?: boolean;
  /**
   * Inject a custom WebSocket constructor (for testing).
   * Defaults to global `WebSocket`.
   * Production code should leave this undefined.
   */
  WebSocketCtor?: typeof WebSocket;
  /** Initial reconnect delay in ms */
  reconnectDelayMs?: number;
}

/**
 * Terminal Stack WebSocket Client (per PR #106 server).
 *
 * MVP v0: 单 sessionId + 单 WebSocket. Reconnect 简化 (exponential backoff 留 P1).
 */
export class TerminalWsClient {
  private ws: WebSocket | null = null;
  private closed = false;
  private reconnectDelayMs: number;
  private readonly opts: TerminalWsClientOptions;

  constructor(opts: TerminalWsClientOptions) {
    this.opts = opts;
    this.reconnectDelayMs = opts.reconnectDelayMs ?? 1000;
  }

  /** Open WebSocket connection. Idempotent: closes existing then reopens. */
  connect(): void {
    if (this.closed) return;
    if (this.ws && this.ws.readyState <= WebSocket.OPEN) return;

    const url = buildTerminalWsUrl(this.opts.sessionId);
    const Ctor = this.opts.WebSocketCtor ?? WebSocket;
    const ws = new Ctor(url);
    ws.binaryType = "arraybuffer";
    this.ws = ws;

    // Cross-env compat: use addEventListener if available (real WS has it),
    // fallback to onopen/onmessage/onerror/onclose property assignment (mock WS).
    if (typeof ws.addEventListener === "function") {
      ws.addEventListener("open", () => {
        this.opts.handlers.onConnectionChange?.(true);
      });
      ws.addEventListener("message", (e: MessageEvent) => {
        const text = typeof e.data === "string" ? e.data : "";
        if (!text) return;
        try {
          const msg = decodeServerMessage(text);
          this.dispatch(msg);
        } catch {
          this.opts.handlers.onError?.({
            type: "error",
            code: "invalid_message",
            message: `failed to parse server message`,
          });
        }
      });
      ws.addEventListener("error", () => {
        // Browsers fire error + close together; defer state-change to onclose
      });
      ws.addEventListener("close", () => {
        this.opts.handlers.onConnectionChange?.(false);
        if (!this.closed && this.opts.autoReconnect) {
          this.scheduleReconnect();
        }
      });
    } else {
      // Fallback: direct property assignment (used by vitest MockWebSocketCtor)
      (ws as unknown as { onopen: (() => void) | null }).onopen = () => {
        this.opts.handlers.onConnectionChange?.(true);
      };
      (ws as unknown as { onmessage: ((e: MessageEvent) => void) | null }).onmessage = (e: MessageEvent) => {
        const text = typeof e.data === "string" ? e.data : "";
        if (!text) return;
        try {
          const msg = decodeServerMessage(text);
          this.dispatch(msg);
        } catch {
          this.opts.handlers.onError?.({
            type: "error",
            code: "invalid_message",
            message: `failed to parse server message`,
          });
        }
      };
      (ws as unknown as { onclose: (() => void) | null }).onclose = () => {
        this.opts.handlers.onConnectionChange?.(false);
        if (!this.closed && this.opts.autoReconnect) {
          this.scheduleReconnect();
        }
      };
    }
  }

  private scheduleReconnect(): void {
    setTimeout(() => {
      if (this.closed) return;
      this.connect();
    }, this.reconnectDelayMs);
  }

  /** Dispatch server message to the right handler. */
  private dispatch(msg: ServerMessage): void {
    const h = this.opts.handlers;
    switch (msg.type) {
      case "hello":
        h.onHello?.(msg);
        break;
      case "snapshot":
        h.onSnapshot?.(msg);
        break;
      case "output":
        h.onOutput?.(msg);
        break;
      case "resize_ack":
        h.onResizeAck?.(msg);
        break;
      case "split_update":
        h.onSplitUpdate?.(msg);
        break;
      case "pty_exit":
        h.onPtyExit?.(msg);
        break;
      case "pong":
        h.onPong?.(msg);
        break;
      case "error":
        h.onError?.(msg);
        break;
    }
  }

  /** Send stdin data to PTY (per AC-2). */
  sendStdin(data: string): void {
    this.send({ type: "stdin", data });
  }

  /** Send PTY resize to server (per AC-3). */
  sendResize(cols: number, rows: number): void {
    this.send({ type: "resize", cols, rows });
  }

  /** Send keepalive ping (server → Pong { seq }). */
  sendPing(seq: number): void {
    this.send({ type: "ping", seq });
  }

  private send(msg: ClientMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      // MVP v0: silently drop if not connected
      return;
    }
    const json = encodeClientMessage(msg);
    this.ws.send(json);
  }

  /** Close connection. Idempotent. */
  close(): void {
    this.closed = true;
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        // ignore
      }
      this.ws = null;
    }
  }

  /** Whether currently connected (per WebSocket readyState). */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

// =====================================================================
// 6. Smoke test (manual, see wsProtocol.ts §6)
// =====================================================================

/**
 * SSR-safe helper: returns true if running in browser.
 * Used to skip WebSocket init during Next.js SSR.
 */
export function isBrowser(): boolean {
  return typeof window !== "undefined" && typeof WebSocket !== "undefined";
}