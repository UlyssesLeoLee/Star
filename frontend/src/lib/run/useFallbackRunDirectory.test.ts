/*
@cypher schema=1 source_sha256=2d69f619c0d60f5e986151c73d16230e3b7c46bb9783fec82aefeb3dffb8f0cb
MERGE (self:File {path:"frontend/src/lib/run/useFallbackRunDirectory.test.ts"})
MERGE (hook:Symbol {id:"frontend/src/lib/run/useFallbackRunDirectory.ts::useFallbackRunDirectory",kind:"function"})
MERGE (test:Symbol {id:"frontend/src/lib/run/useFallbackRunDirectory.test.ts::fallback-directory-bootstrap",kind:"test"})
MERGE (test)-[:TESTS]->(hook)
@endcypher
*/
// 降级目录引导回归 (per 2026-11-19: Project 导航应是树形结构, 不能只剩文字)
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useFallbackRunDirectory } from "./useFallbackRunDirectory";
import { useStore } from "@/lib/store";
import { lanes, projects, worktrees } from "@/lib/seed";

vi.mock("@/mocks/real-mode", () => ({
  isRealMode: vi.fn(() => false),
}));
import { isRealMode } from "@/mocks/real-mode";

const emptyDirectory = { projects: [], lanes: [], worktrees: [] };

beforeEach(() => {
  useStore.setState(emptyDirectory);
  vi.mocked(isRealMode).mockReturnValue(false);
});

afterEach(() => {
  vi.restoreAllMocks();
  useStore.setState(emptyDirectory);
});

function mockFetch(payload: unknown, ok = true) {
  const fetchMock = vi.fn().mockResolvedValue({
    ok,
    json: async () => payload,
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

describe("useFallbackRunDirectory", () => {
  it("store 为空时从 /api/run-directory 注入 Project/Lane/Worktree 层级", async () => {
    const fetchMock = mockFetch({ projects, lanes, worktrees });
    renderHook(() => useFallbackRunDirectory());
    await waitFor(() => {
      expect(useStore.getState().projects.length).toBe(projects.length);
    });
    expect(fetchMock).toHaveBeenCalledWith("/api/run-directory");
    expect(useStore.getState().lanes.length).toBe(lanes.length);
    expect(useStore.getState().worktrees.length).toBe(worktrees.length);
  });

  it("已有数据时不覆盖 (不重复拉取)", async () => {
    useStore.setState({ projects, lanes, worktrees });
    const fetchMock = mockFetch({ projects: [], lanes: [], worktrees: [] });
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(useStore.getState().projects.length).toBe(projects.length);
  });

  it("real-mode 开启时不注入演示数据", async () => {
    vi.mocked(isRealMode).mockReturnValue(true);
    const fetchMock = mockFetch({ projects, lanes, worktrees });
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(useStore.getState().projects.length).toBe(0);
  });

  it("端点失败时保持空态 (不抛错, 由 UI 显示诚实提示)", async () => {
    mockFetch({}, false);
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(useStore.getState().projects.length).toBe(0);
  });

  it("响应形状非法时不注入", async () => {
    mockFetch({ projects: "not-an-array" });
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(useStore.getState().projects.length).toBe(0);
  });
});
