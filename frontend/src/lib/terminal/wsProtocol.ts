// =====================================================================
// wsProtocol.ts — WebSocket Protocol Schema (per ULYS-222 P1-C PR #106)
// (per ULYS-200 P1-D + P1-C 集成, PR #98.5)
// =====================================================================
// 1:1 mirror of crates/terminal-stack/src/ws/protocol.rs:
//   - ClientMessage (stdin / resize / ping) → frontend → server
//   - ServerMessage (hello / snapshot / output / resize_ack / split_update
//                     / pty_exit / pong / error) → server → frontend
// =====================================================================
// 守门:
// - JSON schema 100% round-trip (per ULYS-222 §6 AC-6)
// - tag = "type" / rename_all = "snake_case" (per server contract)
// =====================================================================

// =====================================================================
// 1. Mirror types (per crates/terminal-stack/src/scrollback_buffer.rs + split_pane.rs)
// =====================================================================

export type ScrollbackSource = "stdout" | "stderr" | "agent_injected";

export interface ScrollbackLine {
  id: string;
  timestamp: string; // ISO 8601
  text: string;
  source: ScrollbackSource;
  byte_len: number;
}

export type SplitDirection = "horizontal" | "vertical";

export interface SplitPaneView {
  id: string;
  ratio: number;
  title?: string | null;
}

export type PaneNodeView =
  | {
      kind: "split";
      id: string;
      direction: SplitDirection;
      children: PaneNodeView[];
    }
  | { kind: "pane"; pane: SplitPaneView };

// =====================================================================
// 2. Client → server messages (per crates/terminal-stack/ws/protocol.rs)
// =====================================================================

/** Client → server tagged union (tag = "type", snake_case) */
export type ClientMessage =
  | { type: "stdin"; data: string }
  | { type: "resize"; cols: number; rows: number }
  | { type: "ping"; seq: number };

// =====================================================================
// 3. Server → client messages (per crates/terminal-stack/ws/protocol.rs)
// =====================================================================

/** SplitTree 变化原因 (per 可观测性) */
export type SplitUpdateReason =
  | "user_split"
  | "user_remove"
  | "pane_added"
  | "pane_removed"
  | "resize";

/** Pty 退出原因分类 */
export type PtyExitReason =
  | "normal"
  | "signal"
  | "error"
  | "timeout"
  | "disconnected";

/** 错误码 (stable lowercase snake_case, e.g. "invalid_message") */
export type WsErrorCode =
  | "invalid_message"
  | "session_not_found"
  | "snapshot_load_failed"
  | "output_lagged"
  | "split_lagged"
  | "pty_error"
  | "internal";

/** Server → client tagged union (tag = "type", snake_case) */
export type ServerMessage =
  | {
      type: "hello";
      session_id: string;
      /** 当前 pane UUID 列表 (从 SplitTree 派生) */
      panes: string[];
      /** 滚动缓冲总字节数 (per cli_session.scrollback_bytes 兼容) */
      total_bytes: number;
      /** Server 时间戳 */
      server_time: string;
    }
  | {
      type: "snapshot";
      pane_id: string;
      /** 起始 seq_no (incremental 续传; null = 全量) */
      from_seq: number | null;
      /** 行集合 (按 created_at ASC) */
      lines: ScrollbackLine[];
    }
  | {
      type: "output";
      pane_id: string;
      /** Raw bytes (ANSI escape + text, UTF-8) */
      data: string;
      /** Sequence number (per P1-B 增量 recovery) */
      seq: number;
    }
  | {
      type: "resize_ack";
      pane_id: string;
      cols: number;
      rows: number;
    }
  | {
      type: "split_update";
      root_id: string;
      tree: PaneNodeView;
      reason: SplitUpdateReason;
    }
  | {
      type: "pty_exit";
      pane_id: string;
      /** 退出码 (per POSIX wait 语义, None = signal-killed) */
      exit_code: number | null;
      reason: PtyExitReason;
    }
  | {
      type: "pong";
      seq: number;
      server_time: string;
    }
  | {
      type: "error";
      code: WsErrorCode;
      message: string;
    };

// =====================================================================
// 4. Round-trip helpers (per AC-6)
// =====================================================================

export function decodeClientMessage(s: string): ClientMessage {
  return JSON.parse(s) as ClientMessage;
}

export function encodeClientMessage(msg: ClientMessage): string {
  return JSON.stringify(msg);
}

export function decodeServerMessage(s: string): ServerMessage {
  return JSON.parse(s) as ServerMessage;
}

export function encodeServerMessage(msg: ServerMessage): string {
  return JSON.stringify(msg);
}

// =====================================================================
// 5. WS route template + protocol constants (per server crate)
// =====================================================================

export const WS_ROUTE_TEMPLATE = "/v1/terminal/{session_id}/connect";
export const PROTOCOL_VERSION = "v1.0";

/**
 * 构造 WebSocket URL (per buildRemoteUrl 同款)。
 *
 * @param sessionId - 客户端 session id (UUID)
 * @param basePath - 默认 "/v1/terminal/{session_id}/connect"
 * @returns ws://host/v1/terminal/{session_id}/connect
 */
export function buildTerminalWsUrl(
  sessionId: string,
  basePath = WS_ROUTE_TEMPLATE,
): string {
  const path = basePath.replace("{session_id}", encodeURIComponent(sessionId));
  if (typeof window === "undefined") return path;
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${window.location.host}${path}`;
}

// =====================================================================
// 6. Smoke test (Node-only, in-frontend 跑)
// =====================================================================

// 测过的 raw JSON examples (per PR #106 server serde 格式)
export const EXAMPLE_CLIENT_STDIN = `{"type":"stdin","data":"ls\\n"}`;
export const EXAMPLE_CLIENT_RESIZE = `{"type":"resize","cols":80,"rows":24}`;
export const EXAMPLE_SERVER_HELLO = `{
  "type": "hello",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "panes": ["550e8400-e29b-41d4-a716-446655440001"],
  "total_bytes": 1234,
  "server_time": "2026-09-24T00:00:00Z"
}`;
export const EXAMPLE_SERVER_OUTPUT = `{
  "type": "output",
  "pane_id": "550e8400-e29b-41d4-a716-446655440001",
  "data": "\\u001b[1;36m$ ls\\u001b[0m\\r\\n",
  "seq": 1
}`;

// Note: `EXAMPLE_SERVER_OUTPUT` uses literal escape sequences. In real JSON from server,
// they'll appear as actual control chars; for testing we use raw string with escapes.