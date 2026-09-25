// =====================================================================
// use-restart.test.tsx — ULYS-214 useRestart hook 测试
//
// 4 个测试:
//   1. statusFetcher 拉数据后 `restartAvailable` 反映字段
//   2. `restart()` 触发 mutationFn (POST)
//   3. mutation 成功后 `setQueryData` 立即把 chip 翻回 false
//   4. fetcher 抛错时 `restartAvailable` 仍默认 false (不假阳)
// =====================================================================
import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useRestart } from "./use-restart";
import * as React from "react";

const makeWrapper = (client: QueryClient) =>
  ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client }, children);

describe("useRestart (ULYS-214)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("statusFetcher=available 时 restartAvailable=true", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const statusFetcher = vi.fn(async () => ({
      restart_available: true,
      updated_at: "2026-09-23T10:00:00Z",
    }));

    const { result } = renderHook(
      () => useRestart("sess-1", { statusFetcher, pollIntervalMs: 0 }),
      { wrapper: makeWrapper(client) },
    );

    await waitFor(() => {
      expect(result.current.restartAvailable).toBe(true);
    });
    expect(statusFetcher).toHaveBeenCalledWith("sess-1");
    expect(result.current.isRestarting).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("restart() 调 restartFetcher (POST)", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const statusFetcher = vi.fn(async () => ({
      restart_available: true,
    }));
    const restartFetcher = vi.fn(async () => ({
      restart_available: false,
    }));

    const { result } = renderHook(
      () =>
        useRestart("sess-2", {
          statusFetcher,
          restartFetcher,
          pollIntervalMs: 0,
        }),
      { wrapper: makeWrapper(client) },
    );

    await waitFor(() => {
      expect(result.current.restartAvailable).toBe(true);
    });
    // statusFetcher 至少调 1 次 (初次 mount)
    expect(statusFetcher.mock.calls.length).toBeGreaterThanOrEqual(1);

    await act(async () => {
      result.current.restart();
    });

    expect(restartFetcher).toHaveBeenCalledWith("sess-2");
    await waitFor(() => {
      expect(result.current.isRestarting).toBe(false);
    });
  });

  it("mutation 成功后 restartAvailable 翻回 false (setQueryData)", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const statusFetcher = vi.fn(async () => ({
      restart_available: true,
    }));
    const restartFetcher = vi.fn(async () => ({
      restart_available: false,
      updated_at: "2026-09-23T10:01:00Z",
    }));

    const { result } = renderHook(
      () =>
        useRestart("sess-3", {
          statusFetcher,
          restartFetcher,
          pollIntervalMs: 0,
        }),
      { wrapper: makeWrapper(client) },
    );

    await waitFor(() => {
      expect(result.current.restartAvailable).toBe(true);
    });

    act(() => {
      result.current.restart();
    });

    // 等 mutation 真正结束 + onSuccess 同步写 cache
    await waitFor(() => {
      expect(result.current.isRestarting).toBe(false);
    });
    // 此时 cache 已被 setQueryData(restart_available:false) 覆盖
    await waitFor(() => {
      expect(result.current.restartAvailable).toBe(false);
    });
  });

  it("statusFetcher 抛错时 restartAvailable 默认 false (不假阳)", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const statusFetcher = vi.fn(async () => {
      throw new Error("404 not found");
    });

    const { result } = renderHook(
      () => useRestart("sess-4", { statusFetcher, pollIntervalMs: 0 }),
      { wrapper: makeWrapper(client) },
    );

    // 等 query 进入 error 状态 (retry 关); 即便 error, restart_available 默认 false
    await waitFor(
      () => {
        expect(result.current.error).not.toBeNull();
      },
      { timeout: 3000 },
    );
    expect(result.current.restartAvailable).toBe(false);
    expect((result.current.error as Error).message).toMatch(/404/);
  });
});
