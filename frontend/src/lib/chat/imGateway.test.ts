/*
@cypher schema=1 source_sha256=e6d7bfdd0cfc816e94f25cf3895224503e1106ce70b98a62b6396d637d14e75f
MERGE (self:File {path:"frontend/src/lib/chat/imGateway.test.ts"})
@endcypher
*/
// =====================================================================
// imGateway.test.ts — IM1.0 实时通信网关客户端测试 (per 2026-11-16)
//
// 覆盖: 帧形状 (IM1.0 snake_case 对齐) / 状态机 + 重连退避 /
//       心跳 Ping / 断线队列 (幂等键不变) / onServerFrame 统一分发 /
//       工厂 + 环境变量
//
// 守门: tsc --noEmit 必 exit 0; vitest 全绿。
// 环境: jsdom 无原生 WebSocket → FakeWebSocket 工厂注入。
// =====================================================================

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  DEFAULT_WS_URL,
  ImGateway,
  STORAGE_TOKEN_KEY,
  connectImGateway,
  newUuid,
} from "@/lib/chat/imGateway";
import type {
  ClientFrame,
  SendMessageFrame,
  ServerFrame,
  WebSocketLike,
} from "@/lib/chat/imGateway";

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

// ---------------------------------------------------------------------
// FakeWebSocket — 测试用 WS 桩
// ---------------------------------------------------------------------

class FakeWebSocket implements WebSocketLike {
  static instances: FakeWebSocket[] = [];
  readonly url: string;
  readyState = 0; // CONNECTING
  sent: string[] = [];
  onopen: (() => void) | null = null;
  onmessage: ((ev: { data: unknown }) => void) | null = null;
  onclose: ((ev: { code: number; reason: string }) => void) | null = null;
  onerror: (() => void) | null = null;

  constructor(url: string) {
    this.url = url;
    FakeWebSocket.instances.push(this);
  }

  send(data: string) {
    this.sent.push(data);
  }

  close(code = 1000, reason = "") {
    this.readyState = 3;
    if (this.onclose) this.onclose({ code, reason });
  }

  // ---- 测试驱动 ----

  serverOpen() {
    this.readyState = 1;
    this.onopen?.();
  }

  serverFrame(frame: unknown) {
    this.onmessage?.({ data: JSON.stringify(frame) });
  }

  serverRaw(data: string) {
    this.onmessage?.({ data });
  }

  serverClose(code = 1006, reason = "gone") {
    this.readyState = 3;
    this.onclose?.({ code, reason });
  }

  frames(): ClientFrame[] {
    return this.sent.map((s) => JSON.parse(s) as ClientFrame);
  }

  lastFrame(): ClientFrame | null {
    const last = this.sent[this.sent.length - 1];
    return last ? (JSON.parse(last) as ClientFrame) : null;
  }
}

/** 每次 send 抛异常的 WS (模拟连接已坏的 send) */
class FlakyWebSocket extends FakeWebSocket {
  sendFail = false;
  send(data: string) {
    if (this.sendFail) throw new Error("socket already closed");
    super.send(data);
  }
}

/** 测试用 WS 工厂 (直接构造, 供 opts.socketFactory 注入) */
const socketFactory = (url: string): FakeWebSocket => new FakeWebSocket(url);

function connectedGateway(
  opts: Partial<ConstructorParameters<typeof ImGateway>[0]> = {},
): { gw: ImGateway; ws: FakeWebSocket } {
  const gw = new ImGateway({ socketFactory, ...opts });
  gw.connect();
  const ws = FakeWebSocket.instances[FakeWebSocket.instances.length - 1];
  ws.serverOpen();
  return { gw, ws };
}

function sendMessageFrames(ws: FakeWebSocket): SendMessageFrame[] {
  return ws.frames().filter((f): f is SendMessageFrame => f.type === "send_message");
}

beforeEach(() => {
  vi.useFakeTimers();
  FakeWebSocket.instances = [];
  window.localStorage.clear();
  delete process.env.NEXT_PUBLIC_IM_WS_URL;
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  FakeWebSocket.instances = [];
});

// =====================================================================
// 帧形状 — IM1.0 对齐 (serde tag='type' + snake_case)
// =====================================================================

describe("帧形状 (IM1.0 ws_frames.rs 对齐)", () => {
  it("[1] open 即发 Auth 帧 {type:'auth', req_id, access_token}, url 缺省 ws://localhost:9001", () => {
    const gw = new ImGateway({ socketFactory, accessToken: "tok-1" });
    gw.connect();
    expect(gw.getState()).toBe("connecting");
    const ws = FakeWebSocket.instances[0];
    expect(ws.url).toBe(DEFAULT_WS_URL);
    ws.serverOpen();
    expect(gw.getState()).toBe("connected");
    expect(ws.lastFrame()).toMatchObject({
      type: "auth",
      access_token: "tok-1",
      req_id: expect.stringMatching(UUID_RE),
    });
  });

  it("[2] token 回退: opts 缺省 → localStorage['star-im-token']", () => {
    window.localStorage.setItem(STORAGE_TOKEN_KEY, "ls-tok");
    const { ws } = connectedGateway();
    expect(ws.lastFrame()).toMatchObject({ type: "auth", access_token: "ls-tok" });
  });

  it("[3] opts.accessToken 优先于 localStorage", () => {
    window.localStorage.setItem(STORAGE_TOKEN_KEY, "ls-tok");
    const { ws } = connectedGateway({ accessToken: "opts-tok" });
    expect(ws.lastFrame()).toMatchObject({ type: "auth", access_token: "opts-tok" });
  });

  it("[4] sendMessage → SendMessage 帧 (snake_case + content 按 kind 判别 + reply_to)", () => {
    const { gw, ws } = connectedGateway();
    const r = gw.sendMessage({
      conversationId: "conv-1",
      kind: "text",
      content: { kind: "text", text: "你好" },
      replyTo: "msg-9",
    });
    expect(r.queued).toBe(false);
    expect(r.idempotencyKey).toMatch(UUID_RE);
    expect(ws.lastFrame()).toEqual({
      type: "send_message",
      req_id: expect.stringMatching(UUID_RE),
      conversation_id: "conv-1",
      idempotency_key: r.idempotencyKey,
      kind: "text",
      content: { kind: "text", text: "你好" },
      reply_to: "msg-9",
    });
  });

  it("[5] replyTo 缺省 → 帧不含 reply_to 键", () => {
    const { gw, ws } = connectedGateway();
    gw.sendMessage({ conversationId: "conv-1", kind: "text", content: { kind: "text", text: "hi" } });
    const raw = ws.sent[ws.sent.length - 1];
    expect(raw).not.toContain("reply_to");
  });

  it("[6] markRead → {type:'mark_read', conversation_id, sequence}", () => {
    const { gw, ws } = connectedGateway();
    expect(gw.markRead("conv-1", 7)).toBe(true);
    expect(ws.lastFrame()).toMatchObject({
      type: "mark_read",
      conversation_id: "conv-1",
      sequence: 7,
    });
  });

  it("[7] setTyping → {type:'typing', conversation_id}", () => {
    const { gw, ws } = connectedGateway();
    expect(gw.setTyping("conv-1")).toBe(true);
    expect(ws.lastFrame()).toMatchObject({ type: "typing", conversation_id: "conv-1" });
  });

  it("[8] editMessage → {type:'edit_message', message_id, content}", () => {
    const { gw, ws } = connectedGateway();
    expect(gw.editMessage("msg-1", { kind: "text", text: "v2" })).toBe(true);
    expect(ws.lastFrame()).toMatchObject({
      type: "edit_message",
      message_id: "msg-1",
      content: { kind: "text", text: "v2" },
    });
  });

  it("[9] recallMessage → {type:'recall_message', message_id}", () => {
    const { gw, ws } = connectedGateway();
    expect(gw.recallMessage("msg-1")).toBe(true);
    expect(ws.lastFrame()).toMatchObject({ type: "recall_message", message_id: "msg-1" });
  });

  it("[10] react → {type:'react', message_id, emoji}", () => {
    const { gw, ws } = connectedGateway();
    expect(gw.react("msg-1", "👍")).toBe(true);
    expect(ws.lastFrame()).toMatchObject({ type: "react", message_id: "msg-1", emoji: "👍" });
  });
});

// =====================================================================
// 状态机 + 重连退避
// =====================================================================

describe("状态机 + 重连退避", () => {
  it("[11] offline → connecting → connected → (断线) reconnecting", () => {
    const gw = new ImGateway({ socketFactory });
    expect(gw.getState()).toBe("offline");
    gw.connect();
    expect(gw.getState()).toBe("connecting");
    const ws = FakeWebSocket.instances[0];
    ws.serverOpen();
    expect(gw.getState()).toBe("connected");
    ws.serverClose(1006);
    expect(gw.getState()).toBe("reconnecting");
  });

  it("[12] 退避序列 1s/2s/4s/8s/16s → 上限 30s", () => {
    const gw = new ImGateway({ socketFactory });
    gw.connect();
    const i0 = FakeWebSocket.instances[0];
    i0.serverOpen();
    i0.serverClose(); // → 1s

    vi.advanceTimersByTime(999);
    expect(FakeWebSocket.instances).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(FakeWebSocket.instances).toHaveLength(2);
    FakeWebSocket.instances[1].serverClose(); // → 2s

    vi.advanceTimersByTime(1999);
    expect(FakeWebSocket.instances).toHaveLength(2);
    vi.advanceTimersByTime(1);
    expect(FakeWebSocket.instances).toHaveLength(3);
    FakeWebSocket.instances[2].serverClose(); // → 4s

    vi.advanceTimersByTime(3999);
    expect(FakeWebSocket.instances).toHaveLength(3);
    vi.advanceTimersByTime(1);
    expect(FakeWebSocket.instances).toHaveLength(4);
    FakeWebSocket.instances[3].serverClose(); // → 8s

    vi.advanceTimersByTime(7999);
    expect(FakeWebSocket.instances).toHaveLength(4);
    vi.advanceTimersByTime(1);
    expect(FakeWebSocket.instances).toHaveLength(5);
    FakeWebSocket.instances[4].serverClose(); // → 16s

    vi.advanceTimersByTime(15999);
    expect(FakeWebSocket.instances).toHaveLength(5);
    vi.advanceTimersByTime(1);
    expect(FakeWebSocket.instances).toHaveLength(6);
    FakeWebSocket.instances[5].serverClose(); // → 上限 30s

    vi.advanceTimersByTime(29999);
    expect(FakeWebSocket.instances).toHaveLength(6);
    vi.advanceTimersByTime(1);
    expect(FakeWebSocket.instances).toHaveLength(7);
  });

  it("[13] 连接成功后退避复位: 重连 open 再断 → 回到 1s", () => {
    const gw = new ImGateway({ socketFactory });
    gw.connect();
    const i0 = FakeWebSocket.instances[0];
    i0.serverOpen();
    i0.serverClose(); // → 1s
    vi.advanceTimersByTime(1000);
    const i1 = FakeWebSocket.instances[1];
    i1.serverOpen(); // 成功 → attempt 复位
    i1.serverClose();
    vi.advanceTimersByTime(1000);
    expect(FakeWebSocket.instances).toHaveLength(3); // 未复位则需等 2s
  });

  it("[14] disconnect() → offline, 不再重连", () => {
    const { gw, ws } = connectedGateway();
    gw.disconnect();
    expect(gw.getState()).toBe("offline");
    ws.serverClose(); // close 已同步触发过, 此处再触发也不该重连
    vi.advanceTimersByTime(120_000);
    expect(FakeWebSocket.instances).toHaveLength(1);
  });

  it("[15] autoReconnect=false → 断线直接 offline", () => {
    const { gw, ws } = connectedGateway({ autoReconnect: false });
    ws.serverClose(1006);
    expect(gw.getState()).toBe("offline");
    vi.advanceTimersByTime(120_000);
    expect(FakeWebSocket.instances).toHaveLength(1);
  });

  it("[16] 网关不可达 (工厂抛异常) → reconnecting, 不抛异常", () => {
    const gw = new ImGateway({
      socketFactory: () => {
        throw new Error("IM 网关未接入");
      },
    });
    expect(() => gw.connect()).not.toThrow();
    expect(gw.getState()).toBe("reconnecting");
    vi.advanceTimersByTime(1000);
    expect(gw.getState()).toBe("reconnecting"); // 重试失败仍保持
  });
});

// =====================================================================
// 心跳
// =====================================================================

describe("心跳 Ping (25s)", () => {
  it("[17] 连接后每 25s 发 Ping 帧", () => {
    const { ws } = connectedGateway();
    vi.advanceTimersByTime(25_000);
    expect(ws.frames().filter((f) => f.type === "ping")).toHaveLength(1);
    vi.advanceTimersByTime(25_000);
    expect(ws.frames().filter((f) => f.type === "ping")).toHaveLength(2);
  });

  it("[18] 断线后心跳停止", () => {
    const { ws } = connectedGateway();
    vi.advanceTimersByTime(25_000);
    expect(ws.frames().filter((f) => f.type === "ping")).toHaveLength(1);
    ws.serverClose(1006);
    vi.advanceTimersByTime(100_000);
    expect(ws.frames().filter((f) => f.type === "ping")).toHaveLength(1);
  });
});

// =====================================================================
// 断线队列 pendingMessages (幂等键不变)
// =====================================================================

describe("断线队列 pendingMessages", () => {
  it("[19] 未连接时 sendMessage 入队 (queued=true), 不发帧", () => {
    const gw = new ImGateway({ socketFactory });
    const r = gw.sendMessage({ conversationId: "c1", kind: "text", content: { kind: "text", text: "hi" } });
    expect(r.queued).toBe(true);
    expect(r.idempotencyKey).toMatch(UUID_RE);
    expect(FakeWebSocket.instances).toHaveLength(0);
    expect(gw.pendingMessages).toHaveLength(1);
    expect(gw.pendingMessages[0].idempotency_key).toBe(r.idempotencyKey);
  });

  it("[20] 重连后按序重发, 幂等键保持不变, 队列清空", () => {
    const gw = new ImGateway({ socketFactory });
    const r1 = gw.sendMessage({ conversationId: "c1", kind: "text", content: { kind: "text", text: "first" } });
    const r2 = gw.sendMessage({ conversationId: "c1", kind: "text", content: { kind: "text", text: "second" } });
    gw.connect();
    const ws = FakeWebSocket.instances[0];
    ws.serverOpen();
    expect(gw.pendingMessages).toHaveLength(0);
    const sends = sendMessageFrames(ws);
    expect(sends).toHaveLength(2);
    expect(sends[0].idempotency_key).toBe(r1.idempotencyKey);
    expect(sends[1].idempotency_key).toBe(r2.idempotencyKey);
    expect(sends[0].content).toEqual({ kind: "text", text: "first" });
    expect(sends[1].content).toEqual({ kind: "text", text: "second" });
  });

  it("[21] markRead / setTyping 未连接 → false, 不入队", () => {
    const gw = new ImGateway({ socketFactory });
    expect(gw.markRead("c1", 5)).toBe(false);
    expect(gw.setTyping("c1")).toBe(false);
    expect(gw.editMessage("m1", { kind: "text", text: "x" })).toBe(false);
    expect(gw.recallMessage("m1")).toBe(false);
    expect(gw.react("m1", "👍")).toBe(false);
    expect(gw.pendingMessages).toHaveLength(0);
  });

  it("[22] send 抛异常 (连接已坏) → send_message 帧回到暂存队列", () => {
    const flaky = new FlakyWebSocket("ws://x");
    const gw = new ImGateway({ socketFactory: () => flaky });
    gw.connect();
    flaky.serverOpen();
    flaky.sendFail = true;
    const r = gw.sendMessage({ conversationId: "c1", kind: "text", content: { kind: "text", text: "hi" } });
    expect(r.queued).toBe(false); // 状态机当时认为已连接
    expect(gw.pendingMessages).toHaveLength(1);
    expect(gw.pendingMessages[0].idempotency_key).toBe(r.idempotencyKey);
  });
});

// =====================================================================
// onServerFrame 统一分发
// =====================================================================

describe("onServerFrame 统一分发", () => {
  const MESSAGE_NEW = {
    type: "message_new",
    message: {
      id: "m-1",
      conversation_id: "c1",
      sequence: 1,
      kind: "text",
      content: { kind: "text", text: "hi" },
      state: "delivered",
      created_at: "2026-11-16T00:00:00Z",
      reactions: [],
    },
  };

  it("[23] 服务端帧 → 回调收到已解析 ServerFrame", () => {
    const { gw, ws } = connectedGateway();
    const frames: ServerFrame[] = [];
    gw.onServerFrame((f) => frames.push(f));
    ws.serverFrame({ type: "pong", ts: 123 });
    expect(frames).toHaveLength(1);
    expect(frames[0]).toEqual({ type: "pong", ts: 123 });
  });

  it("[24] message_new 帧分发给回调", () => {
    const { gw, ws } = connectedGateway();
    const frames: ServerFrame[] = [];
    gw.onServerFrame((f) => frames.push(f));
    ws.serverFrame(MESSAGE_NEW);
    expect(frames).toHaveLength(1);
    expect(frames[0]).toMatchObject({ type: "message_new" });
    expect((frames[0] as { message: { sequence: number } }).message.sequence).toBe(1);
  });

  it("[25] ack 帧携带 data/error 形状", () => {
    const { gw, ws } = connectedGateway();
    const frames: ServerFrame[] = [];
    gw.onServerFrame((f) => frames.push(f));
    ws.serverFrame({
      type: "ack",
      req_id: "r-1",
      ok: false,
      error: { code: "RATE_LIMITED", message: "auth.rate_limit.send_message", trace_id: "tr-1", ts: 1 },
    });
    expect(frames).toHaveLength(1);
    expect(frames[0]).toMatchObject({
      type: "ack",
      ok: false,
      error: { code: "RATE_LIMITED" },
    });
  });

  it("[26] 退订后不再收到帧", () => {
    const { gw, ws } = connectedGateway();
    const frames: ServerFrame[] = [];
    const off = gw.onServerFrame((f) => frames.push(f));
    off();
    ws.serverFrame(MESSAGE_NEW);
    expect(frames).toHaveLength(0);
  });

  it("[27] 非法 JSON / 缺 type → 静默丢弃, 不崩, 状态不变", () => {
    const { gw, ws } = connectedGateway();
    const frames: ServerFrame[] = [];
    gw.onServerFrame((f) => frames.push(f));
    expect(() => ws.serverRaw("not-json")).not.toThrow();
    expect(() => ws.serverRaw('{"foo":1}')).not.toThrow();
    expect(frames).toHaveLength(0);
    expect(gw.getState()).toBe("connected");
  });
});

// =====================================================================
// 工厂 + 环境变量 + UUID
// =====================================================================

describe("工厂 + 环境变量", () => {
  it("[28] connectImGateway 创建后立即连接 (connecting)", () => {
    const gw = connectImGateway({ socketFactory });
    expect(gw.getState()).toBe("connecting");
    FakeWebSocket.instances[0].serverOpen();
    expect(gw.getState()).toBe("connected");
  });

  it("[29] 网关地址: opts.url > NEXT_PUBLIC_IM_WS_URL > 缺省 ws://localhost:9001", () => {
    new ImGateway({ socketFactory }).connect();
    expect(FakeWebSocket.instances[0].url).toBe(DEFAULT_WS_URL);

    FakeWebSocket.instances = [];
    process.env.NEXT_PUBLIC_IM_WS_URL = "ws://env-gw:9";
    new ImGateway({ socketFactory }).connect();
    expect(FakeWebSocket.instances[0].url).toBe("ws://env-gw:9");

    FakeWebSocket.instances = [];
    new ImGateway({ socketFactory, url: "wss://prod.example.com" }).connect();
    expect(FakeWebSocket.instances[0].url).toBe("wss://prod.example.com");
  });

  it("[30] newUuid 返回 RFC4122 v4 形状且唯一", () => {
    const a = newUuid();
    const b = newUuid();
    expect(a).toMatch(UUID_RE);
    expect(b).toMatch(UUID_RE);
    expect(a).not.toBe(b);
  });
});
