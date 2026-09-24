// =====================================================================
// wsClient.test.ts — WebSocket Client integration test (per PR #98.5)
// =====================================================================
// 守门: 6+ 单测验证 wsClient.ts 行为 (mock WebSocket)
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { TerminalWsClient } from "./wsClient";

// Mock WebSocket class
class MockWebSocket {
  readyState = 0; // CONNECTING
  binaryType = "";
  onopen: ((e: unknown) => void) | null = null;
  onclose: ((e: unknown) => void) | null = null;
  onerror: ((e: unknown) => void) | null = null;
  onmessage: ((e: unknown) => void) | null = null;
  sent: string[] = [];

  constructor(public url: string) {}

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = 3; // CLOSED
    if (this.onclose) this.onclose({});
  }

  // Test helpers
  triggerOpen() {
    this.readyState = 1; // OPEN
    if (this.onopen) this.onopen({});
  }

  triggerMessage(data: string) {
    if (this.onmessage) this.onmessage({ data });
  }

  triggerError() {
    if (this.onerror) this.onerror({});
  }

  triggerClose() {
    this.readyState = 3;
    if (this.onclose) this.onclose({});
  }
}

let mockWs: MockWebSocket | null = null;
let mockWsUrl = "";

// @ts-expect-error mock global
globalThis.WebSocket = vi.fn().mockImplementation((url: string) => {
  mockWs = new MockWebSocket(url);
  mockWsUrl = url;
  return mockWs;
});

describe("TerminalWsClient (PR #98.5)", () => {
  beforeEach(() => {
    mockWs = null;
    mockWsUrl = "";
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("A. connect() opens WebSocket with correct URL", () => {
    const client = new TerminalWsClient({
      sessionId: "test-session",
      handlers: {},
    });
    client.connect();
    expect(mockWsUrl).toContain("/v1/terminal/test-session/connect");
  });

  it("B. onopen fires onConnectionChange(true)", () => {
    let connected: boolean | null = null;
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {
        onConnectionChange: (c) => {
          connected = c;
        },
      },
    });
    client.connect();
    mockWs!.triggerOpen();
    expect(connected).toBe(true);
  });

  it("C. sendStdin sends JSON-encoded message", () => {
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {},
    });
    client.connect();
    mockWs!.triggerOpen();
    client.sendStdin("ls\n");
    expect(mockWs!.sent).toHaveLength(1);
    const msg = JSON.parse(mockWs!.sent[0]);
    expect(msg).toEqual({ type: "stdin", data: "ls\n" });
  });

  it("D. sendResize sends cols/rows JSON", () => {
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {},
    });
    client.connect();
    mockWs!.triggerOpen();
    client.sendResize(100, 30);
    expect(mockWs!.sent).toHaveLength(1);
    expect(JSON.parse(mockWs!.sent[0])).toEqual({
      type: "resize",
      cols: 100,
      rows: 30,
    });
  });

  it("E. sendPing sends seq number", () => {
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {},
    });
    client.connect();
    mockWs!.triggerOpen();
    client.sendPing(42);
    expect(JSON.parse(mockWs!.sent[0])).toEqual({ type: "ping", seq: 42 });
  });

  it("F. incoming hello dispatches to onHello handler", () => {
    let received: { session_id: string } | null = null;
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {
        onHello: (msg) => {
          received = { session_id: msg.session_id };
        },
      },
    });
    client.connect();
    mockWs!.triggerOpen();
    mockWs!.triggerMessage(
      JSON.stringify({
        type: "hello",
        session_id: "550e8400-e29b-41d4-a716-446655440000",
        panes: [],
        total_bytes: 0,
        server_time: "2026-09-24T00:00:00Z",
      }),
    );
    expect(received).toEqual({
      session_id: "550e8400-e29b-41d4-a716-446655440000",
    });
  });

  it("G. incoming output dispatches to onOutput handler", () => {
    let receivedSeq: number | null = null;
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {
        onOutput: (msg) => {
          receivedSeq = msg.seq;
        },
      },
    });
    client.connect();
    mockWs!.triggerOpen();
    mockWs!.triggerMessage(
      JSON.stringify({
        type: "output",
        pane_id: "pane-1",
        data: "hello\n",
        seq: 5,
      }),
    );
    expect(receivedSeq).toBe(5);
  });

  it("H. incoming split_update dispatches to onSplitUpdate handler", () => {
    let received: unknown = null;
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {
        onSplitUpdate: (msg) => {
          received = msg;
        },
      },
    });
    client.connect();
    mockWs!.triggerOpen();
    mockWs!.triggerMessage(
      JSON.stringify({
        type: "split_update",
        root_id: "root-1",
        tree: {
          kind: "split",
          id: "split-1",
          direction: "vertical",
          children: [
            { kind: "pane", pane: { id: "p1", ratio: 0.5 } },
            { kind: "pane", pane: { id: "p2", ratio: 0.5 } },
          ],
        },
        reason: "user_split",
      }),
    );
    expect(received).toBeTruthy();
    expect((received as { reason: string }).reason).toBe("user_split");
  });

  it("I. malformed JSON triggers onError with invalid_message code", () => {
    let received: { code: string; message: string } | null = null;
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {
        onError: (msg) => {
          received = msg;
        },
      },
    });
    client.connect();
    mockWs!.triggerOpen();
    mockWs!.triggerMessage("{not valid json");
    expect(received).toEqual({
      type: "error",
      code: "invalid_message",
      message: "failed to parse server message",
    });
  });

  it("J. close() marks as closed (subsequent connect is no-op)", () => {
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {},
    });
    client.connect();
    expect(mockWs).not.toBeNull();
    const firstWs = mockWs;
    client.close();
    client.connect();
    // Should not create new WS
    expect(mockWs).toBe(firstWs);
  });

  it("K. isConnected() reflects WebSocket readyState", () => {
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {},
    });
    client.connect();
    expect(client.isConnected()).toBe(false); // CONNECTING
    mockWs!.triggerOpen();
    expect(client.isConnected()).toBe(true); // OPEN
    mockWs!.triggerClose();
    expect(client.isConnected()).toBe(false); // CLOSED
  });

  it("L. sendStdin silently drops if not connected (per MVP v0)", () => {
    const client = new TerminalWsClient({
      sessionId: "test",
      handlers: {},
    });
    client.connect();
    // Don't triggerOpen → readyState is 0 (CONNECTING)
    client.sendStdin("test");
    expect(mockWs!.sent).toHaveLength(0);
  });
});