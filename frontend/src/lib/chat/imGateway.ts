/*
@cypher schema=1 source_sha256=15dc255400e822a20d24dd6965152d39e09c521d575551ae77432d592149fb7e
MERGE (self:File {path:"frontend/src/lib/chat/imGateway.ts"})
MERGE (gw:Symbol {id:"frontend/src/lib/chat/imGateway.ts::ImGateway",kind:"class"})
MERGE (factory:Symbol {id:"frontend/src/lib/chat/imGateway.ts::connectImGateway",kind:"function"})
MERGE (self)-[:DEFINES]->(gw)
MERGE (self)-[:DEFINES]->(factory)
@endcypher
*/

// =====================================================================
// imGateway — IM1.0 实时通信网关客户端 (per 2026-11-16 用户发令: 独立文件, parent 接线)
//
//   对齐 D:\IM1.0 crates/im-protocol/src/ws_frames.rs:
//   - 客户端 8 类: auth / send_message / edit_message / recall_message /
//     react / mark_read / typing / ping
//   - 服务端 11 变体: connected / auth_ok / ack / message_new /
//     message_edited / message_recalled / reaction_added /
//     presence_update / typing / pong / force_disconnect
//   - serde tag='type' + rename_all='snake_case' → JSON 键一律 snake_case
//   - content 按 kind 判别 (content.rs: text/image/file/sticker/system/custom)
//
//   诚实原则: 网关不可达时本地模式继续可用并明确标注 "IM 网关未接入"
//   (状态机停在 reconnecting, 消息进入 pendingMessages, 不抛异常不阻塞 UI)
//
//   本文件不 import chatStore —— 由 parent 接线。
// =====================================================================

/** IM1.0 消息 kind (content.rs MessageContent 的 6 种) */
export type MessageKind = "text" | "image" | "file" | "sticker" | "system" | "custom";

/** IM1.0 MessageContent —— serde tag='kind' + snake_case 判别联合 */
export type MessageContent =
  | { kind: "text"; text: string }
  | {
      kind: "image";
      media_id: string;
      width?: number;
      height?: number;
      thumbnail_media_id?: string;
    }
  | {
      kind: "file";
      media_id: string;
      file_name: string;
      size_bytes: number;
      mime_type?: string;
    }
  | { kind: "sticker"; sticker_id: string }
  | {
      kind: "system";
      event: string;
      actor_user_id?: string;
      target_user_id?: string;
    }
  | { kind: "custom"; schema: string; data: unknown };

// ---------------------------------------------------------------------
// 客户端 → 服务端 (ws_frames.rs ClientFrame, 8 类)
// ---------------------------------------------------------------------

export type SendMessageFrame = {
  type: "send_message";
  req_id: string;
  conversation_id: string;
  idempotency_key: string;
  kind: MessageKind;
  content: MessageContent;
  reply_to?: string;
};

export type ClientFrame =
  | { type: "auth"; req_id: string; access_token: string }
  | SendMessageFrame
  | { type: "edit_message"; req_id: string; message_id: string; content: MessageContent }
  | { type: "recall_message"; req_id: string; message_id: string }
  | { type: "react"; req_id: string; message_id: string; emoji: string }
  | { type: "mark_read"; req_id: string; conversation_id: string; sequence: number }
  | { type: "typing"; req_id: string; conversation_id: string }
  | { type: "ping"; ts?: number };

// ---------------------------------------------------------------------
// 服务端 → 客户端 (ws_frames.rs ServerFrame, 11 变体)
// ---------------------------------------------------------------------

/** error_body.rs ErrorBody —— ack.ok=false 的错误载荷 */
export interface ErrorBody {
  code: string;
  message: string;
  trace_id: string;
  ts: number;
  details?: Array<{ field: string; reason: string }>;
}

/** ws_frames.rs AckData */
export interface AckData {
  message_id?: string;
  sequence?: number;
  idempotent_replay?: boolean;
}

/** ws_frames.rs WireMessage (aux-13 §4 下行消息 schema) */
export interface WireMessage {
  id: string;
  conversation_id: string;
  sequence: number;
  sender_id?: string;
  kind: string;
  content: MessageContent;
  reply_to?: string;
  state: string;
  created_at: string;
  edited_at?: string;
  reactions: Array<{ emoji: string; user_ids: string[] }>;
}

export type ServerFrame =
  | { type: "connected"; session_id: string }
  | { type: "auth_ok"; req_id?: string | null }
  | { type: "ack"; req_id: string; ok: boolean; data?: AckData; error?: ErrorBody }
  | { type: "message_new"; message: WireMessage }
  | {
      type: "message_edited";
      message_id: string;
      conversation_id: string;
      content: MessageContent;
      edited_at: string;
    }
  | { type: "message_recalled"; message_id: string; conversation_id: string }
  | {
      type: "reaction_added";
      message_id: string;
      conversation_id: string;
      user_id: string;
      emoji: string;
    }
  | { type: "presence_update"; user_id: string; status: string }
  | { type: "typing"; conversation_id: string; user_id: string }
  | { type: "pong"; ts: number }
  | { type: "force_disconnect"; reason: string };

// ---------------------------------------------------------------------
// 连接状态机
// ---------------------------------------------------------------------

export type GatewayState = "connecting" | "connected" | "reconnecting" | "offline";

/** WebSocket 最小接口 —— 供测试注入 FakeWebSocket (jsdom 无原生 WebSocket) */
export interface WebSocketLike {
  readonly url: string;
  readyState: number;
  send(data: string): void;
  close(code?: number, reason?: string): void;
  onopen: (() => void) | null;
  onmessage: ((ev: { data: unknown }) => void) | null;
  onclose: ((ev: { code: number; reason: string }) => void) | null;
  onerror: (() => void) | null;
}

export interface ImGatewayOptions {
  /** 网关地址; 缺省 env NEXT_PUBLIC_IM_WS_URL → ws://localhost:9001 */
  url?: string;
  /** 优先于 localStorage['star-im-token'] */
  accessToken?: string;
  /** 断线自动重连 (默认 true) */
  autoReconnect?: boolean;
  /** 心跳间隔 ms (默认 25_000) */
  heartbeatMs?: number;
  /** 重连退避上限 ms (默认 30_000) */
  maxBackoffMs?: number;
  /** 重连退避基数 ms (默认 1_000 → 1s/2s/4s/8s/16s/30s…) */
  baseBackoffMs?: number;
  /** 测试注入: WebSocket 工厂 */
  socketFactory?: (url: string) => WebSocketLike;
}

interface ResolvedOptions {
  url: string;
  accessToken: string | undefined;
  autoReconnect: boolean;
  heartbeatMs: number;
  maxBackoffMs: number;
  baseBackoffMs: number;
  socketFactory: (url: string) => WebSocketLike;
}

export const DEFAULT_WS_URL = "ws://localhost:9001";
export const STORAGE_TOKEN_KEY = "star-im-token";

// ---------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------

/** RFC 4122 v4 UUID (浏览器/Node 用 crypto.randomUUID; jsdom 回退手搓) */
export function newUuid(): string {
  const c = globalThis.crypto;
  if (c && typeof c.randomUUID === "function") return c.randomUUID();
  const b = new Uint8Array(16);
  if (c && typeof c.getRandomValues === "function") c.getRandomValues(b);
  else for (let i = 0; i < 16; i++) b[i] = Math.floor(Math.random() * 256);
  b[6] = (b[6] & 0x0f) | 0x40; // version 4
  b[8] = (b[8] & 0x3f) | 0x80; // variant 10xx
  const h = Array.from(b, (x) => x.toString(16).padStart(2, "0"));
  return `${h.slice(0, 4).join("")}-${h.slice(4, 6).join("")}-${h.slice(6, 8).join("")}-${h.slice(8, 10).join("")}-${h.slice(10).join("")}`;
}

function readStoredToken(): string {
  try {
    if (typeof window !== "undefined" && window.localStorage) {
      return window.localStorage.getItem(STORAGE_TOKEN_KEY) ?? "";
    }
  } catch {
    /* 隐私模式 / 存储不可用时忽略 */
  }
  return "";
}

function resolveUrl(url: string | undefined): string {
  if (url) return url;
  if (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_IM_WS_URL) {
    return process.env.NEXT_PUBLIC_IM_WS_URL;
  }
  return DEFAULT_WS_URL;
}

function defaultSocketFactory(url: string): WebSocketLike {
  if (typeof WebSocket === "undefined") {
    throw new Error("IM 网关未接入: 运行时无 WebSocket");
  }
  // 原生 WebSocket 的 on* 签名带 (ev: Event) 参数, 结构上不满足 WebSocketLike
  // 的最小接口; 运行时完全兼容 (send/close/on* 均存在), 此处一次性收窄。
  return new WebSocket(url) as unknown as WebSocketLike;
}

// ---------------------------------------------------------------------
// ImGateway
// ---------------------------------------------------------------------

export class ImGateway {
  private readonly opts: ResolvedOptions;
  private ws: WebSocketLike | null = null;
  private state: GatewayState = "offline";
  /** 连续连接失败次数 → 退避 1s/2s/4s/8s/16s, 上限 30s; 成功连接后复位 */
  private attempt = 0;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private manuallyClosed = false;
  /** 断线暂存: send_message 帧 (幂等键保持在帧内, 重发不变) */
  private readonly _pendingMessages: SendMessageFrame[] = [];
  private readonly frameListeners = new Set<(frame: ServerFrame) => void>();

  constructor(options: ImGatewayOptions = {}) {
    this.opts = {
      url: resolveUrl(options.url),
      accessToken: options.accessToken,
      autoReconnect: options.autoReconnect ?? true,
      heartbeatMs: options.heartbeatMs ?? 25_000,
      maxBackoffMs: options.maxBackoffMs ?? 30_000,
      baseBackoffMs: options.baseBackoffMs ?? 1_000,
      socketFactory: options.socketFactory ?? defaultSocketFactory,
    };
  }

  // ------------------------------------------------------------------
  // 观测
  // ------------------------------------------------------------------

  getState(): GatewayState {
    return this.state;
  }

  /** 断线暂存消息 (只读视图; parent 可展示 "待重发 N 条") */
  get pendingMessages(): readonly SendMessageFrame[] {
    return this._pendingMessages;
  }

  // ------------------------------------------------------------------
  // 生命周期
  // ------------------------------------------------------------------

  connect(): void {
    if (this.ws) return; // 已在连接中/已连接
    this.manuallyClosed = false;
    this.openSocket("connecting");
  }

  disconnect(): void {
    this.manuallyClosed = true;
    this.clearTimers();
    const ws = this.ws;
    this.ws = null;
    this.setState("offline");
    try {
      ws?.close(1000, "client");
    } catch {
      /* close 本身不应抛 */
    }
  }

  private openSocket(phase: "connecting" | "reconnecting"): void {
    this.clearTimers();
    this.setState(phase);
    let ws: WebSocketLike;
    try {
      ws = this.opts.socketFactory(this.opts.url);
    } catch {
      // 网关不可达 (如运行时无 WebSocket): 保持本地模式, 按退避重试
      this.scheduleReconnect();
      return;
    }
    this.ws = ws;
    ws.onopen = () => {
      if (this.ws !== ws) return;
      this.sendAuth(ws);
      this.attempt = 0; // 连接成功 → 退避复位
      this.setState("connected");
      this.startHeartbeat(ws);
      this.flushPending(ws); // 重发暂存消息 (幂等键不变)
    };
    ws.onmessage = (ev) => this.handleIncoming(ev);
    ws.onclose = (ev) => {
      if (this.ws !== ws) return;
      this.ws = null;
      this.handleClose(ev);
    };
    ws.onerror = () => {
      /* 浏览器 onerror 后必跟 onclose; 统一在 onclose 处理 */
    };
  }

  private handleClose(ev: { code: number; reason: string }): void {
    void ev;
    if (this.manuallyClosed || !this.opts.autoReconnect) {
      this.setState("offline");
      return;
    }
    this.scheduleReconnect();
  }

  private scheduleReconnect(): void {
    this.clearTimers();
    this.setState("reconnecting");
    const delay = Math.min(this.opts.baseBackoffMs * 2 ** this.attempt, this.opts.maxBackoffMs);
    this.attempt += 1;
    this.reconnectTimer = setTimeout(() => this.openSocket("reconnecting"), delay);
  }

  private setState(next: GatewayState): void {
    this.state = next;
  }

  private clearTimers(): void {
    if (this.heartbeatTimer !== null) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  // ------------------------------------------------------------------
  // 发送
  // ------------------------------------------------------------------

  private sendAuth(ws: WebSocketLike): void {
    const token = this.opts.accessToken ?? readStoredToken();
    const frame: Extract<ClientFrame, { type: "auth" }> = {
      type: "auth",
      req_id: newUuid(),
      access_token: token,
    };
    this.sendRaw(ws, frame);
  }

  private startHeartbeat(ws: WebSocketLike): void {
    this.heartbeatTimer = setInterval(() => {
      const ping: Extract<ClientFrame, { type: "ping" }> = {
        type: "ping",
        ts: Date.now(),
      };
      this.sendRaw(ws, ping);
    }, this.opts.heartbeatMs);
  }

  private flushPending(ws: WebSocketLike): void {
    const queued = this._pendingMessages.splice(0, this._pendingMessages.length);
    for (const frame of queued) this.sendRaw(ws, frame);
  }

  private sendRaw(ws: WebSocketLike, frame: ClientFrame): boolean {
    try {
      ws.send(JSON.stringify(frame));
      return true;
    } catch {
      // 连接已坏: send_message 帧回到暂存队列等下次重连
      if (frame.type === "send_message") this._pendingMessages.push(frame);
      return false;
    }
  }

  /** 已连接则直接发送, 返回 false 表示未发 */
  private sendOrDrop(frame: ClientFrame): boolean {
    if (this.state === "connected" && this.ws) return this.sendRaw(this.ws, frame);
    return false;
  }

  /**
   * 发消息 → SendMessage 帧。
   * 已连接: 直接发送 (queued=false); 未连接: 暂存 pendingMessages,
   * 重连后按入队顺序重发, idempotency_key 保持不变 (queued=true)。
   */
  sendMessage(input: {
    conversationId: string;
    kind: MessageKind;
    content: MessageContent;
    replyTo?: string;
  }): { idempotencyKey: string; queued: boolean } {
    const idempotencyKey = newUuid();
    const frame: SendMessageFrame = {
      type: "send_message",
      req_id: newUuid(),
      conversation_id: input.conversationId,
      idempotency_key: idempotencyKey,
      kind: input.kind,
      content: input.content,
      ...(input.replyTo ? { reply_to: input.replyTo } : {}),
    };
    if (this.state === "connected" && this.ws) {
      this.sendRaw(this.ws, frame);
      return { idempotencyKey, queued: false };
    }
    this._pendingMessages.push(frame);
    return { idempotencyKey, queued: true };
  }

  /** 上报已读 → MarkRead 帧; 未连接返回 false (不入队) */
  markRead(conversationId: string, sequence: number): boolean {
    return this.sendOrDrop({
      type: "mark_read",
      req_id: newUuid(),
      conversation_id: conversationId,
      sequence,
    });
  }

  /** 输入中指示 → Typing 帧; 未连接返回 false (瞬态帧不入队) */
  setTyping(conversationId: string): boolean {
    return this.sendOrDrop({ type: "typing", req_id: newUuid(), conversation_id: conversationId });
  }

  /** 编辑消息 → EditMessage 帧 */
  editMessage(messageId: string, content: MessageContent): boolean {
    return this.sendOrDrop({
      type: "edit_message",
      req_id: newUuid(),
      message_id: messageId,
      content,
    });
  }

  /** 撤回消息 → RecallMessage 帧 */
  recallMessage(messageId: string): boolean {
    return this.sendOrDrop({ type: "recall_message", req_id: newUuid(), message_id: messageId });
  }

  /** 添加 reaction → React 帧 */
  react(messageId: string, emoji: string): boolean {
    return this.sendOrDrop({ type: "react", req_id: newUuid(), message_id: messageId, emoji });
  }

  // ------------------------------------------------------------------
  // 接收: 统一分发
  // ------------------------------------------------------------------

  private handleIncoming(ev: { data: unknown }): void {
    let frame: ServerFrame;
    try {
      const parsed: unknown = JSON.parse(String(ev.data));
      if (!parsed || typeof parsed !== "object" || typeof (parsed as { type?: unknown }).type !== "string") {
        return; // 形状不符: 静默丢弃 (不崩, 不影响本地模式)
      }
      frame = parsed as ServerFrame;
    } catch {
      return; // 非法 JSON: 静默丢弃
    }
    for (const cb of this.frameListeners) {
      try {
        cb(frame);
      } catch {
        /* 单个监听器异常不阻断其它监听器 */
      }
    }
  }

  /** 注册统一帧分发回调; 返回退订函数 */
  onServerFrame(cb: (frame: ServerFrame) => void): () => void {
    this.frameListeners.add(cb);
    return () => {
      this.frameListeners.delete(cb);
    };
  }
}

/** 工厂: 创建并立即 connect() */
export function connectImGateway(options: ImGatewayOptions = {}): ImGateway {
  const gateway = new ImGateway(options);
  gateway.connect();
  return gateway;
}
