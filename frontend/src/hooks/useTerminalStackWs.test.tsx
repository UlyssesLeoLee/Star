/*
@cypher schema=1 source_sha256=34a7aac5bc1afe9cee93eaae50d0d9f8a0605e427a5b366eeaf5fffc3e26b865
MERGE (self:File {path:"frontend/src/hooks/useTerminalStackWs.test.tsx"})
MERGE (hook:Function {name:"useTerminalStackWs", language:"typescript", visibility:"public"})
MERGE (suite:Symbol {id:"frontend/src/hooks/useTerminalStackWs.test.tsx::useTerminalStackWs suite",kind:"test_suite"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(hook)
@endcypher
*/
// =====================================================================
// useTerminalStackWs.test.tsx — star-api-rest REST 接线 + 优雅降级测试
// =====================================================================
// 覆盖:
//   1. sessionId=null 初始 mode='preview' 且不发任何请求
//   2. createPane 成功 → mode='real' + paneId 入 state + POST 默认 base
//   3. createPane 404 / 网络抛错 → mode='error' 降级, 返回 null
//   4. writeLine 自动建 pane + POST /line, 成功后本地滚动缓冲追加
//   5. writeLine 后端不可达 → false + mode='error' + 不追加
//   6. fetchScrollback 全量快照替换 / since_seq 增量追平
//   7. NEXT_PUBLIC_API_BASE 覆盖默认 base
//   8. error 后可重试恢复 'real'
//   9. sessionId!=null 回归: 仍走真实 WS 接线
// 环境: vitest jsdom, vi.stubGlobal("fetch", mockFetch) (repo 惯例)。
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, cleanup } from "@testing-library/react";
import { useTerminalStackWs } from "./useTerminalStackWs";
import { useTerminalStackStore } from "@/components/terminal/terminalStackStore";

const CRLF = String.fromCharCode(13, 10); // 避免测试内嵌控制字符转义被改写

// mock wsClient: sessionId!=null 的 WS 分支不建真实 WebSocket
vi.mock("@/lib/terminal/wsClient", () => {
  const wsConnectMock = vi.fn();
  const wsCloseMock = vi.fn();
  const MockWsClient = vi.fn(function (this: unknown) {
    (this as { connect: () => void }).connect = wsConnectMock;
    (this as { close: () => void }).close = wsCloseMock;
    (this as { isConnected: () => boolean }).isConnected = () => false;
    (this as { sendStdin: () => void }).sendStdin = () => {};
    (this as { sendResize: () => void }).sendResize = () => {};
  });
  return { TerminalWsClient: MockWsClient, wsConnectMock, wsCloseMock };
});

const mockFetch = vi.fn();

function jsonResponse(body: unknown, status = 200, statusText = "OK") {
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText,
    json: async () => body,
  } as unknown as Response;
}

const DEFAULT_BASE = "http://localhost:8081";
const PANE_PATH = "/api/v1/terminal/pane";

async function flush() {
  await act(async () => {
    await Promise.resolve();
  });
}

describe("useTerminalStackWs (star-api-rest REST mode)", () => {
  beforeEach(() => {
    mockFetch.mockReset();
    vi.stubGlobal("fetch", mockFetch);
    useTerminalStackStore.getState().reset();
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });

  it("sessionId=null 初始 mode='preview' 且不发任何请求", async () => {
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));
    await flush();
    expect(result.current.mode).toBe("preview");
    expect(result.current.paneId).toBeNull();
    expect(result.current.outputByPane).toEqual({});
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it("createPane 成功 → mode='real' + paneId 入 state + POST 默认 base", async () => {
    mockFetch.mockResolvedValueOnce(jsonResponse({ id: "pane-1" }, 201, "Created"));
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let returned: string | null = null;
    await act(async () => {
      returned = await result.current.createPane();
    });

    expect(returned).toBe("pane-1");
    expect(result.current.mode).toBe("real");
    expect(result.current.paneId).toBe("pane-1");
    expect(result.current.outputByPane).toEqual({ "pane-1": "" });
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(mockFetch).toHaveBeenCalledWith(
      `${DEFAULT_BASE}${PANE_PATH}`,
      { method: "POST" },
    );
  });

  it("createPane 404 → mode='error' 降级 + 返回 null", async () => {
    mockFetch.mockResolvedValueOnce(
      jsonResponse({ message: "not found" }, 404, "Not Found"),
    );
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let returned: string | null = "unset";
    await act(async () => {
      returned = await result.current.createPane();
    });

    expect(returned).toBeNull();
    expect(result.current.mode).toBe("error");
    expect(result.current.paneId).toBeNull();
  });

  it("createPane 网络抛错 → mode='error' 降级", async () => {
    mockFetch.mockRejectedValue(new TypeError("network down"));
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let returned: string | null = "unset";
    await act(async () => {
      returned = await result.current.createPane();
    });

    expect(returned).toBeNull();
    expect(result.current.mode).toBe("error");
  });

  it("createPane 幂等: 已建 pane 直接复用, 不再发请求", async () => {
    mockFetch.mockResolvedValueOnce(jsonResponse({ pane_id: "pane-9" }, 201, "Created"));
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let first: string | null = null;
    let second: string | null = null;
    await act(async () => {
      first = await result.current.createPane();
      second = await result.current.createPane();
    });

    expect(first).toBe("pane-9");
    expect(second).toBe("pane-9");
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it("writeLine 自动建 pane + POST /line, 成功后本地滚动缓冲追加", async () => {
    mockFetch
      .mockResolvedValueOnce(jsonResponse({ id: "pane-2" }, 201, "Created"))
      .mockResolvedValueOnce(jsonResponse({ ok: true }));
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let ok = false;
    await act(async () => {
      ok = await result.current.writeLine("ls");
    });

    expect(ok).toBe(true);
    expect(result.current.mode).toBe("real");
    expect(result.current.outputByPane["pane-2"]).toBe("ls" + CRLF);
    expect(mockFetch).toHaveBeenCalledTimes(2);
    expect(mockFetch).toHaveBeenNthCalledWith(
      2,
      `${DEFAULT_BASE}${PANE_PATH}/pane-2/line`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text: "ls" }),
      },
    );
  });

  it("writeLine 后端不可达 → false + mode='error' + 不追加", async () => {
    mockFetch.mockRejectedValue(new TypeError("ECONNREFUSED"));
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let ok = true;
    await act(async () => {
      ok = await result.current.writeLine("star run");
    });

    expect(ok).toBe(false);
    expect(result.current.mode).toBe("error");
    expect(result.current.outputByPane).toEqual({});
  });

  it("fetchScrollback 全量快照 (无 since_seq) 替换本地缓冲并返回 next seq", async () => {
    mockFetch
      .mockResolvedValueOnce(jsonResponse({ id: "pane-3" }, 201, "Created"))
      .mockResolvedValueOnce(
        jsonResponse({
          lines: [{ text: "hello" }, "world"],
          next_since_seq: 2,
        }),
      );
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let next: number | null = null;
    await act(async () => {
      await result.current.createPane();
      next = await result.current.fetchScrollback();
    });

    expect(next).toBe(2);
    expect(result.current.outputByPane["pane-3"]).toBe("hello" + CRLF + "world");
    expect(mockFetch).toHaveBeenLastCalledWith(
      `${DEFAULT_BASE}${PANE_PATH}/pane-3/scrollback`,
      undefined,
    );
  });

  it("fetchScrollback since_seq 增量追平本地缓冲", async () => {
    mockFetch
      .mockResolvedValueOnce(jsonResponse({ id: "pane-4" }, 201, "Created"))
      .mockResolvedValueOnce(jsonResponse({ ok: true }))
      .mockResolvedValueOnce(
        jsonResponse({ lines: ["out-1", { line: "out-2" }], last_seq: 5 }),
      );
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let next: number | null = null;
    await act(async () => {
      await result.current.writeLine("ls");
      next = await result.current.fetchScrollback(2);
    });

    expect(next).toBe(5);
    expect(result.current.outputByPane["pane-4"]).toBe(
      "ls" + CRLF + "out-1" + CRLF + "out-2",
    );
    expect(mockFetch).toHaveBeenLastCalledWith(
      `${DEFAULT_BASE}${PANE_PATH}/pane-4/scrollback?since_seq=2`,
      undefined,
    );
  });

  it("fetchScrollback 失败 → mode='error' + 返回 null", async () => {
    mockFetch.mockResolvedValueOnce(
      jsonResponse({ message: "gone" }, 500, "Internal Server Error"),
    );
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    let next: number | null = 123;
    await act(async () => {
      next = await result.current.fetchScrollback();
    });

    expect(next).toBeNull();
    expect(result.current.mode).toBe("error");
  });

  it("NEXT_PUBLIC_API_BASE 覆盖默认 base", async () => {
    const prev = process.env.NEXT_PUBLIC_API_BASE;
    process.env.NEXT_PUBLIC_API_BASE = "http://localhost:9999";
    try {
      mockFetch.mockResolvedValueOnce(jsonResponse({ id: "pane-env" }, 201, "Created"));
      const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));
      await act(async () => {
        await result.current.createPane();
      });
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:9999" + PANE_PATH,
        { method: "POST" },
      );
    } finally {
      if (prev === undefined) delete process.env.NEXT_PUBLIC_API_BASE;
      else process.env.NEXT_PUBLIC_API_BASE = prev;
    }
  });

  it("error 后可重试恢复 'real' (后端恢复后再次写行)", async () => {
    mockFetch.mockRejectedValueOnce(new TypeError("network down"));
    const { result } = renderHook(() => useTerminalStackWs({ sessionId: null }));

    await act(async () => {
      await result.current.writeLine("ls");
    });
    expect(result.current.mode).toBe("error");

    mockFetch
      .mockResolvedValueOnce(jsonResponse({ id: "pane-5" }, 201, "Created"))
      .mockResolvedValueOnce(jsonResponse({ ok: true }));
    let ok = false;
    await act(async () => {
      ok = await result.current.writeLine("ls");
    });

    expect(ok).toBe(true);
    expect(result.current.mode).toBe("real");
    expect(result.current.paneId).toBe("pane-5");
    expect(result.current.outputByPane["pane-5"]).toBe("ls" + CRLF);
  });

  it("回归: sessionId!=null 仍走真实 WS 接线 (connect + unmount close)", async () => {
    const { wsConnectMock, wsCloseMock } = await import("@/lib/terminal/wsClient");
    const { unmount } = renderHook(() =>
      useTerminalStackWs({ sessionId: "sess-1" }),
    );
    await flush();
    expect(wsConnectMock).toHaveBeenCalledTimes(1);
    unmount();
    expect(wsCloseMock).toHaveBeenCalledTimes(1);
  });
});
