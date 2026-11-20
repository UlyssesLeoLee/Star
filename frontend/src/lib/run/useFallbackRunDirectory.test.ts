/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/run/useFallbackRunDirectory.test.ts"})
MERGE (hook:Symbol {id:"frontend/src/lib/run/useFallbackRunDirectory.ts::useFallbackRunDirectory",kind:"function"})
MERGE (test:Symbol {id:"frontend/src/lib/run/useFallbackRunDirectory.test.ts::fallback-directory-bootstrap",kind:"test"})
MERGE (test)-[:TESTS]->(hook)
@endcypher
*/
// 降级目录引导回归 (per 2026-11-19: Project 导航应是树形结构, 不能只剩文字;
//  P0-lane 2026-11-20: 数据源改为独立演示模块 demoDirectory.ts, 不再走网络)
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useFallbackRunDirectory } from "./useFallbackRunDirectory";
import { useStore } from "@/lib/store";
import { demoRunDirectoryPayload } from "./demoDirectory";

vi.mock("@/mocks/real-mode", () => ({
  isRealMode: vi.fn(() => false),
}));
import { isRealMode } from "@/mocks/real-mode";

const emptyDirectory = { projects: [], lanes: [], worktrees: [] };
const demoPayload = demoRunDirectoryPayload();

beforeEach(() => {
  useStore.setState(emptyDirectory);
  vi.mocked(isRealMode).mockReturnValue(false);
});

afterEach(() => {
  vi.restoreAllMocks();
  useStore.setState(emptyDirectory);
});

describe("useFallbackRunDirectory", () => {
  it("store 为空时从独立演示模块注入 Project/Lane/Worktree 层级 (无网络请求)", async () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    renderHook(() => useFallbackRunDirectory());
    await waitFor(() => {
      expect(useStore.getState().projects.length).toBe(demoPayload.projects.length);
    });
    expect(useStore.getState().lanes.length).toBe(demoPayload.lanes.length);
    expect(useStore.getState().worktrees.length).toBe(demoPayload.worktrees.length);
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it("已有数据时不覆盖 (不重复注入)", async () => {
    useStore.setState({
      projects: demoPayload.projects,
      lanes: demoPayload.lanes,
      worktrees: demoPayload.worktrees,
    });
    const snapshot = demoRunDirectoryPayload().worktrees[0].name;
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(useStore.getState().projects.length).toBe(demoPayload.projects.length);
    expect(useStore.getState().worktrees[0].name).toBe(snapshot);
  });

  it("real-mode 开启时绝不注入演示数据", async () => {
    vi.mocked(isRealMode).mockReturnValue(true);
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(useStore.getState().projects.length).toBe(0);
    expect(useStore.getState().lanes.length).toBe(0);
    expect(useStore.getState().worktrees.length).toBe(0);
  });

  it("部分集合已有数据时视为已有真实数据, 不注入演示数据", async () => {
    useStore.setState({ projects: demoPayload.projects, lanes: [], worktrees: [] });
    renderHook(() => useFallbackRunDirectory());
    await new Promise((r) => setTimeout(r, 30));
    expect(useStore.getState().lanes.length).toBe(0);
    expect(useStore.getState().worktrees.length).toBe(0);
  });
});
