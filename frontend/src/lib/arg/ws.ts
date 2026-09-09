// =====================================================================
// frontend/src/lib/arg/ws.ts — WebSocket 客户端 (per ARG.4 §4.12 + doc 14 §2)
// =====================================================================
// Per docs/architecture/2026-09-03-arg/14-arg-10-arg-frontend.md §2.2
// + docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §2.1
//
// 协议 fanout (per doc 14 §2.1):
//   - arg_edge_changed          → RelationshipView + RelationshipEditor
//   - arg_dispatch_route        → RelationshipView (高亮 dispatch path)
//   - arg_context_inject        → RelationshipView (event log 追加)
//   - arg_trust_score_update    → RelationshipView + AchievementWall
//   - arg_achievement_unlocked  → AchievementWall (解锁动画 + 推送)
//
// 守门合规:
//   - 守门 #5 env 安全: ws URL 走 process.env.NEXT_PUBLIC_ARG_WS_BASE, 不读 secret
//   - 守门 #9 RPC 不可靠: 自动重连 (exponential backoff), 上限 30s, 永远不抛
//   - 守门 #12 v22 mock fallback: 后端不可达时返回 mock event loop, 保持 dev
//     开发体验 (UI 仍能展示数据)
// =====================================================================

import type { ARGEvent, WSBridgeEvent, WSBridgeEventKind } from "./types";

// ----- config -----

function getWsBase(): string {
  // 优先 NEXT_PUBLIC_ARG_WS_BASE; 缺省推同源 ws/wss 转换
  const fromEnv =
    typeof process !== "undefined" && process.env
      ? process.env.NEXT_PUBLIC_ARG_WS_BASE
      : undefined;
  if (fromEnv) return fromEnv;
  if (typeof window === "undefined") return "ws://localhost";
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${window.location.host}`;
}

/** WS 连接配置. */
export interface ArgWSOptions {
  /** Tenant id (RLS 13 類 hint, 服务端 extractor 读). */
  tenantId?: string;
  /** 最大重连间隔 ms (default 30 000). */
  maxReconnectDelay?: number;
  /** 初始重连间隔 ms (default 1 000). */
  initialReconnectDelay?: number;
  /** 收到事件 callback. */
  onEvent: (ev: WSBridgeEvent) => void;
  /** 状态变更 callback. */
  onStatusChange?: (status: WSStatus) => void;
  /** 启用 mock fallback (default true in dev, 守门 #12 v22). */
  mockFallback?: boolean;
}

export type WSStatus = "connecting" | "connected" | "disconnected" | "mock";

/** Default WS event kind list (5 协议 per doc 14 §2.1). */
const DEFAULT_KINDS: readonly WSBridgeEventKind[] = [
  "arg_edge_changed",
  "arg_dispatch_route",
  "arg_context_inject",
  "arg_trust_score_update",
  "arg_achievement_unlocked",
];

// ----- client class -----

/**
 * ArgWebSocketClient — wraps a single WS connection to `/ws/arg/events`.
 *
 * Usage:
 *   const ws = new ArgWebSocketClient({ onEvent: (ev) => store.applyWs(ev) });
 *   ws.connect();
 *   ...
 *   ws.disconnect();
 */
export class ArgWebSocketClient {
  private ws: WebSocket | null = null;
  private opts: Required<Omit<ArgWSOptions, "tenantId" | "onEvent" | "onStatusChange">> &
    Pick<ArgWSOptions, "tenantId" | "onEvent" | "onStatusChange">;
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private mockTimer: ReturnType<typeof setInterval> | null = null;
  private closedByUser = false;
  private status: WSStatus = "disconnected";

  constructor(opts: ArgWSOptions) {
    this.opts = {
      maxReconnectDelay: opts.maxReconnectDelay ?? 30_000,
      initialReconnectDelay: opts.initialReconnectDelay ?? 1_000,
      mockFallback: opts.mockFallback ?? (typeof window !== "undefined"),
      tenantId: opts.tenantId,
      onEvent: opts.onEvent,
      onStatusChange: opts.onStatusChange,
    };
  }

  /** Current connection status. */
  getStatus(): WSStatus {
    return this.status;
  }

  /** Open the connection. Idempotent: reconnect closes any prior socket. */
  connect(): void {
    this.closedByUser = false;
    this.openSocket();
  }

  /** Close + stop reconnect. */
  disconnect(): void {
    this.closedByUser = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.mockTimer) {
      clearInterval(this.mockTimer);
      this.mockTimer = null;
    }
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        /* ignore */
      }
      this.ws = null;
    }
    this.setStatus("disconnected");
  }

  /** Internal: open the WebSocket and bind handlers. */
  private openSocket(): void {
    if (typeof window === "undefined" || typeof WebSocket === "undefined") {
      // SSR / non-browser — bail to mock
      this.startMockFallback("WebSocket unavailable in non-browser env");
      return;
    }
    const base = getWsBase();
    const url = new URL("/ws/arg/events", base);
    if (this.opts.tenantId) {
      url.searchParams.set("tenant_id", this.opts.tenantId);
    }
    this.setStatus("connecting");
    let socket: WebSocket;
    try {
      socket = new WebSocket(url.toString());
    } catch (e) {
      this.scheduleReconnect();
      return;
    }
    this.ws = socket;

    socket.onopen = () => {
      this.reconnectAttempt = 0;
      this.setStatus("connected");
    };
    socket.onmessage = (ev: MessageEvent<string>) => {
      try {
        const parsed = JSON.parse(ev.data) as WSBridgeEvent;
        this.opts.onEvent(parsed);
      } catch {
        // ignore non-JSON frames
      }
    };
    socket.onerror = () => {
      // error events precede close; let onclose drive reconnect
    };
    socket.onclose = () => {
      this.ws = null;
      if (!this.closedByUser) {
        this.scheduleReconnect();
      } else {
        this.setStatus("disconnected");
      }
    };
  }

  /** Internal: schedule a reconnect with exponential backoff. */
  private scheduleReconnect(): void {
    if (this.closedByUser) return;
    const delay = Math.min(
      this.opts.maxReconnectDelay,
      this.opts.initialReconnectDelay * Math.pow(2, this.reconnectAttempt),
    );
    this.reconnectAttempt += 1;
    // If we've been retrying for a while, kick into mock fallback
    if (this.reconnectAttempt > 3 && this.opts.mockFallback) {
      this.startMockFallback(`reconnect attempts > 3, falling back to mock`);
      return;
    }
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.openSocket();
    }, delay);
  }

  /** Internal: kick a mock event stream so the UI keeps working without backend. */
  private startMockFallback(reason: string): void {
    if (this.mockTimer) return; // already running
    this.setStatus("mock");
    if (typeof console !== "undefined") {
      // mock fallback is a known dev affordance, log info-only
      console.info(`[ARG WS] mock fallback (${reason})`);
    }
    // Emit 1 mock event every 7s to drive the UI; cheap and rare
    this.mockTimer = setInterval(() => {
      const mock = makeMockWsEvent();
      if (mock) this.opts.onEvent(mock);
    }, 7_000);
  }

  /** Internal: status setter. */
  private setStatus(s: WSStatus): void {
    if (this.status === s) return;
    this.status = s;
    this.opts.onStatusChange?.(s);
  }
}

// ----- mock event factory (per 守门 #12 v22) -----

/**
 * Generate a plausible mock WS event. Rotates through the 5 protocol kinds
 * so the UI exercises every fanout path. Returns `null` ~30% of the time to
 * keep the noise level low.
 */
function makeMockWsEvent(): WSBridgeEvent | null {
  if (Math.random() < 0.3) return null;
  const kinds = DEFAULT_KINDS;
  const kind = kinds[Math.floor(Math.random() * kinds.length)];
  return {
    kind,
    received_at: new Date().toISOString(),
    payload: mockPayloadFor(kind),
  };
}

function mockPayloadFor(kind: WSBridgeEventKind): unknown {
  switch (kind) {
    case "arg_edge_changed":
      return { edge_type: "DELEGATES_TO", weight: 0.7 };
    case "arg_dispatch_route":
      return { route: "SA_01 -> SA_02" };
    case "arg_context_inject":
      return { source: "MENTORS", context_size: 256 };
    case "arg_trust_score_update":
      return { agent_id: "ag-mock-001", delta: 0.01 };
    case "arg_achievement_unlocked":
      return { code: "BEH-001-FIRST-EDGE" };
  }
}

// ----- helper: convert WS bridge event to ARGEvent for store -----

/**
 * Best-effort conversion of a WS bridge event to a typed ARGEvent.
 * Returns `null` if the kind does not map to a known ARGEvent variant.
 *
 * Per doc 14 §2.1, the 5 WS kinds fanout to 9 ARGEvent variants:
 *   - arg_edge_changed      → EdgeCreated / EdgeUpdated / EdgeArchived
 *   - arg_dispatch_route    → (no ARGEvent, derived state)
 *   - arg_context_inject    → (no ARGEvent, derived state)
 *   - arg_trust_score_update→ TrustScoreChanged
 *   - arg_achievement_unlocked → AchievementUnlocked
 */
export function wsBridgeToArgEvent(ev: WSBridgeEvent): ARGEvent | null {
  switch (ev.kind) {
    case "arg_trust_score_update": {
      const p = ev.payload as { agent_id?: string; before?: number; after?: number; delta?: number };
      if (!p.agent_id || typeof p.delta !== "number") return null;
      return {
        type: "trust_score_changed",
        agent_id: p.agent_id,
        before: typeof p.before === "number" ? p.before : 0.5,
        after: typeof p.after === "number" ? p.after : 0.5 + p.delta,
        delta: p.delta,
      };
    }
    case "arg_achievement_unlocked": {
      const p = ev.payload as { code?: string };
      if (!p.code) return null;
      return {
        type: "achievement_unlocked",
        payload: {
          id: "unlock-mock",
          achievement_code: p.code,
          user_id: "user-mock",
          agent_ids: [],
          trigger_metadata: { source: "ws-bridge", received_at: ev.received_at },
          tenant_id: "tenant-mock",
          unlocked_at: ev.received_at,
        },
      };
    }
    case "arg_edge_changed": {
      // backend sse_hub distinguishes created/updated/archived; for the bridge
      // event we forward as updated with mock payload
      const p = ev.payload as { edge_type?: string; weight?: number };
      return {
        type: "edge_updated",
        id: "edge-mock",
        before: mockEdge(p, "before"),
        after: mockEdge(p, "after"),
      };
    }
    case "arg_dispatch_route":
    case "arg_context_inject":
    default:
      // dispatch / context inject don't directly map to ARGEvent;
      // store applies as no-op fanout
      return null;
  }
}

function mockEdge(
  p: { edge_type?: string; weight?: number },
  _when: "before" | "after",
): import("./types").Edge {
  return {
    id: "edge-mock",
    from_agent: "ag-mock-from",
    to_agent: "ag-mock-to",
    edge_type: (p.edge_type as import("./types").RelationshipType) ?? "DELEGATES_TO",
    weight: p.weight ?? 0.7,
    direction: "directed",
    archived: false,
    metadata: null,
    tenant_id: "tenant-mock",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    version: 1,
    created_by: "user-mock",
  };
}
