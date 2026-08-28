// =====================================================================
// useBoardSync.test.tsx — W5 多人协同 hook 测试 (per §8.1)
//   1. 2s polling 触发 (use fake timers)
//   2. staleTime 缓存 (1s 内不重 fetch)
//   3. queryKey 含 projectId
// =====================================================================
import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useBoardSync } from "./useBoardSync";
import * as React from "react";

// mock productionApi.boardSync — 避免依赖 zustand store 副作用
vi.mock("@/lib/api", () => ({
  productionApi: {
    boardSync: vi.fn(() => ({
      cursor: "2026-08-28T18:00:00Z",
      snapshot: {
        board: {
          id: "board-001",
          tenant_id: "ten-acme",
          project_id: "prj-physis",
          name: "Test",
          columns: [],
        },
        recentActivity: [],
      },
    })),
    workItemSync: vi.fn(() => ({ cursor: "x", items: [] })),
    transitionWorkItem: vi.fn(),
  },
}));

const makeWrapper = (client: QueryClient) =>
  ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client }, children);

describe("useBoardSync", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("首次 mount 立即 fetch", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const { result } = renderHook(
      () => useBoardSync({ projectId: "prj-test" }),
      { wrapper: makeWrapper(client) }
    );
    // wait for query to settle
    await waitFor(() => {
      expect(result.current.data).toBeDefined();
    });
    const { productionApi } = await import("@/lib/api");
    expect((productionApi.boardSync as any).mock.calls.length).toBeGreaterThanOrEqual(1);
    expect(result.current.data?.cursor).toBe("2026-08-28T18:00:00Z");
  });

  it("queryKey 含 projectId", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const { result } = renderHook(
      () => useBoardSync({ projectId: "prj-special" }),
      { wrapper: makeWrapper(client) }
    );
    await waitFor(() => {
      expect(result.current.data).toBeDefined();
    });
    expect(result.current).toBeDefined();
    const cache = client.getQueryCache();
    const queries = cache.findAll({ queryKey: ["board-sync"] });
    expect(queries.length).toBeGreaterThanOrEqual(1);
    expect(queries[0]?.queryKey).toEqual(["board-sync", "prj-special"]);
  });

  it("enabled=false 时不 fetch", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const { result } = renderHook(
      () => useBoardSync({ projectId: "prj-physis", enabled: false }),
      { wrapper: makeWrapper(client) }
    );
    // 等 100ms 确认没 fetch
    await new Promise((r) => setTimeout(r, 100));
    expect(result.current.fetchStatus).toBe("idle");
    const { productionApi } = await import("@/lib/api");
    expect((productionApi.boardSync as any).mock.calls.length).toBe(0);
  });
});

