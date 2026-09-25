// =====================================================================
// wsProtocol.test.ts — protocol round-trip tests (per PR #98.5 修复 schema 对齐)
// =====================================================================
// 守门: 6+ 单测验证 wsProtocol.ts 与 PR #106 server 1:1 对应
//       (per ULYS-222 §6 AC-6 "JSON schema round-trip 100%")
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  type ClientMessage,
  type ServerMessage,
  decodeClientMessage,
  decodeServerMessage,
  encodeClientMessage,
  encodeServerMessage,
  buildTerminalWsUrl,
  WS_ROUTE_TEMPLATE,
  PROTOCOL_VERSION,
  EXAMPLE_CLIENT_STDIN,
  EXAMPLE_CLIENT_RESIZE,
  EXAMPLE_SERVER_HELLO,
  EXAMPLE_SERVER_OUTPUT,
} from "./wsProtocol";

describe("wsProtocol (PR #98.5 修复 schema 对齐)", () => {
  it("A. ClientMessage::stdin round-trip", () => {
    const msg: ClientMessage = { type: "stdin", data: "ls\n" };
    const json = encodeClientMessage(msg);
    expect(json).toContain('"type":"stdin"');
    expect(json).toContain('"data":"ls\\n"');
    const parsed = decodeClientMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("B. ClientMessage::resize round-trip", () => {
    const msg: ClientMessage = { type: "resize", cols: 80, rows: 24 };
    const json = encodeClientMessage(msg);
    expect(json).toContain('"type":"resize"');
    expect(json).toContain('"cols":80');
    expect(json).toContain('"rows":24');
    const parsed = decodeClientMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("C. ClientMessage::ping round-trip", () => {
    const msg: ClientMessage = { type: "ping", seq: 42 };
    const json = encodeClientMessage(msg);
    const parsed = decodeClientMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("D. ServerMessage::hello round-trip", () => {
    const msg: ServerMessage = {
      type: "hello",
      session_id: "550e8400-e29b-41d4-a716-446655440000",
      panes: ["550e8400-e29b-41d4-a716-446655440001"],
      total_bytes: 1234,
      server_time: "2026-09-24T00:00:00Z",
    };
    const json = encodeServerMessage(msg);
    expect(json).toContain('"type":"hello"');
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("E. ServerMessage::snapshot round-trip", () => {
    const msg: ServerMessage = {
      type: "snapshot",
      pane_id: "550e8400-e29b-41d4-a716-446655440001",
      from_seq: null,
      lines: [
        {
          id: "550e8400-e29b-41d4-a716-446655440002",
          timestamp: "2026-09-24T00:00:00Z",
          text: "hello",
          source: "stdout",
          byte_len: 5,
        },
      ],
    };
    const json = encodeServerMessage(msg);
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("F. ServerMessage::output round-trip (PR #106 字节流)", () => {
    const msg: ServerMessage = {
      type: "output",
      pane_id: "550e8400-e29b-41d4-a716-446655440001",
      data: "[1;36m$ ls[0m\r\n",
      seq: 1,
    };
    const json = encodeServerMessage(msg);
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("G. ServerMessage::split_update round-trip (PR #98 PaneNodeView)", () => {
    const msg: ServerMessage = {
      type: "split_update",
      root_id: "550e8400-e29b-41d4-a716-446655440010",
      tree: {
        kind: "split",
        id: "550e8400-e29b-41d4-a716-446655440011",
        direction: "horizontal",
        children: [
          {
            kind: "pane",
            pane: {
              id: "550e8400-e29b-41d4-a716-446655440012",
              ratio: 0.5,
            },
          },
          {
            kind: "pane",
            pane: {
              id: "550e8400-e29b-41d4-a716-446655440013",
              ratio: 0.5,
            },
          },
        ],
      },
      reason: "user_split",
    };
    const json = encodeServerMessage(msg);
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("H. ServerMessage::pty_exit round-trip", () => {
    const msg: ServerMessage = {
      type: "pty_exit",
      pane_id: "550e8400-e29b-41d4-a716-446655440001",
      exit_code: 0,
      reason: "normal",
    };
    const json = encodeServerMessage(msg);
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("I. ServerMessage::error round-trip", () => {
    const msg: ServerMessage = {
      type: "error",
      code: "snapshot_load_failed",
      message: "SQLite DB not found",
    };
    const json = encodeServerMessage(msg);
    expect(json).toContain('"code":"snapshot_load_failed"');
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("J. ServerMessage::pong round-trip", () => {
    const msg: ServerMessage = {
      type: "pong",
      seq: 7,
      server_time: "2026-09-24T00:00:00Z",
    };
    const json = encodeServerMessage(msg);
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("K. ServerMessage::resize_ack round-trip", () => {
    const msg: ServerMessage = {
      type: "resize_ack",
      pane_id: "550e8400-e29b-41d4-a716-446655440001",
      cols: 100,
      rows: 30,
    };
    const json = encodeServerMessage(msg);
    const parsed = decodeServerMessage(json);
    expect(parsed).toEqual(msg);
  });

  it("L. malformed JSON returns null for decoding", () => {
    expect(() => decodeClientMessage("{not valid")).toThrow();
    expect(() => decodeServerMessage("{not valid")).toThrow();
  });

  it("M. unknown message type returns error", () => {
    const bad = '{"type":"unknown_msg","data":"x"}';
    expect(() => decodeClientMessage(bad)).toThrow();
  });

  it("N. EXAMPLE_SERVER_HELLO decodes correctly (PR #106 contract smoke)", () => {
    const msg = decodeServerMessage(EXAMPLE_SERVER_HELLO);
    expect(msg.type).toBe("hello");
    if (msg.type === "hello") {
      expect(msg.session_id).toBe("550e8400-e29b-41d4-a716-446655440000");
      expect(msg.total_bytes).toBe(1234);
    }
  });

  it("O. EXAMPLE_SERVER_OUTPUT decodes correctly (PR #106 ANSI bytes)", () => {
    const msg = decodeServerMessage(EXAMPLE_SERVER_OUTPUT);
    expect(msg.type).toBe("output");
    if (msg.type === "output") {
      expect(msg.pane_id).toBe("550e8400-e29b-41d4-a716-446655440001");
      expect(msg.seq).toBe(1);
    }
  });

  it("P. EXAMPLE_CLIENT_STDIN + RESIZE decode correctly", () => {
    expect(decodeClientMessage(EXAMPLE_CLIENT_STDIN)).toEqual({
      type: "stdin",
      data: "ls\n",
    });
    expect(decodeClientMessage(EXAMPLE_CLIENT_RESIZE)).toEqual({
      type: "resize",
      cols: 80,
      rows: 24,
    });
  });

  it("Q. buildTerminalWsUrl substitutes session_id correctly", () => {
    const url = buildTerminalWsUrl("abc-123");
    expect(url).toContain("/v1/terminal/abc-123/connect");
    // ws:// for http, wss:// for https
    expect(url).toMatch(/^ws(s)?:\/\//);
  });

  it("R. WS_ROUTE_TEMPLATE matches server spec", () => {
    expect(WS_ROUTE_TEMPLATE).toBe("/v1/terminal/{session_id}/connect");
  });

  it("S. PROTOCOL_VERSION is v1.0", () => {
    expect(PROTOCOL_VERSION).toBe("v1.0");
  });
});