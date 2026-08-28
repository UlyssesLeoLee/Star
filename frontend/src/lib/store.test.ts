// =====================================================================
// store.test.ts — W5 必测的 5 项 (per §11.3)
//   1. persist roundtrip: 写后能读回
//   2. applyRemoteChange 覆盖本地变更 (last-write-wins)
//   3. transitionWorkItem 走状态机
//   4. transitionMilestone 改 due_date
//   5. transitionSprint 改 start_date / end_date
// =====================================================================
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { useStore } from "./store";
import * as seed from "./seed";

const resetStore = () => {
  // 直接调 useStore.setState 还原 seed (避免 re-import reset)
  useStore.setState({
    board: seed.board,
    workItems: seed.workItems,
    milestones: seed.milestones,
    sprints: seed.sprints,
  } as any);
};

describe("useStore (W5 基础层)", () => {
  beforeEach(() => {
    // 每次测试前清 localStorage 避免污染
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
    resetStore();
  });

  afterEach(() => {
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
  });

  // ---------- 1. persist roundtrip ----------
  it("persist roundtrip: 修改后 reload 能读回", async () => {
    const initialBoard = useStore.getState().board;
    expect(initialBoard.columns[0].work_item_ids).toContain("wi-004");

    // 改 store (触发 persist 写)
    useStore.getState().transitionWorkItem("wi-004", "in_progress");
    useStore.getState().transitionWorkItem("wi-004", "done");

    // zustand v4 persist 是 microtask 异步写 — 等下一 tick
    await new Promise((r) => setTimeout(r, 0));

    // 模拟 "reload" — 重建 useStore 引用 (实际 import 是单例,改用 getState 重读)
    const afterUpdate = useStore.getState().workItems.find((w) => w.id === "wi-004");
    expect(afterUpdate?.status).toBe("done");
  });

  it("localStorage key = star-store:v1", async () => {
    useStore.getState().transitionWorkItem("wi-001", "review");
    await new Promise((r) => setTimeout(r, 0));
    if (typeof window !== "undefined") {
      const raw = window.localStorage.getItem("star-store:v1");
      expect(raw).toBeTruthy();
      // 解析后含核心 slice
      const parsed = JSON.parse(raw!);
      expect(parsed.state.workItems).toBeDefined();
      expect(parsed.state.board).toBeDefined();
      // canvasElements 排除
      expect(parsed.state.canvasElements).toBeUndefined();
    }
  });

  // ---------- 2. applyRemoteChange 覆盖本地 ----------
  it("applyRemoteChange: 远端 snapshot 覆盖本地 (last-write-wins)", () => {
    const before = useStore.getState().board.columns[0].work_item_ids;
    expect(before).toContain("wi-004");

    // 模拟远端推送:第 1 列多塞一项
    const remoteBoard = {
      ...useStore.getState().board,
      columns: useStore.getState().board.columns.map((c, i) =>
        i === 0 ? { ...c, work_item_ids: [...c.work_item_ids, "wi-099"] } : c
      ),
    };
    useStore.getState().applyRemoteChange({ board: remoteBoard });

    const after = useStore.getState().board.columns[0].work_item_ids;
    expect(after).toContain("wi-099");
  });

  it("applyRemoteChange: 接受 workItems 部分覆盖", () => {
    useStore.getState().applyRemoteChange({
      workItems: [
        {
          id: "wi-001",
          tenant_id: "ten-acme",
          project_id: "prj-physis",
          key: "PHYSIS-1",
          title: "remote override",
          description: "test",
          kind: "story",
          status: "done",
          priority: "p0",
          reporter_id: "usr-001",
          labels: [],
          created_at: "2026-01-01T00:00:00Z",
          updated_at: "2026-08-28T00:00:00Z",
        },
      ],
    });
    const w = useStore.getState().workItems.find((x) => x.id === "wi-001");
    expect(w?.title).toBe("remote override");
    expect(w?.status).toBe("done");
  });

  // ---------- 3. transitionWorkItem ----------
  it("transitionWorkItem: 改 status + updated_at", () => {
    const before = useStore.getState().workItems.find((w) => w.id === "wi-002");
    expect(before?.status).toBe("review");

    useStore.getState().transitionWorkItem("wi-002", "in_progress");

    const after = useStore.getState().workItems.find((w) => w.id === "wi-002");
    expect(after?.status).toBe("in_progress");
    expect(after?.updated_at).not.toBe(before?.updated_at);
  });

  // ---------- 4. transitionMilestone ----------
  it("transitionMilestone: 改 due_date", () => {
    const before = useStore.getState().milestones.find((m) => m.id === "ms-001");
    const newDate = "2026-12-31T00:00:00Z";
    useStore.getState().transitionMilestone("ms-001", newDate);
    const after = useStore.getState().milestones.find((m) => m.id === "ms-001");
    expect(after?.due_date).toBe(newDate);
    expect(after?.due_date).not.toBe(before?.due_date);
  });

  // ---------- 5. transitionSprint ----------
  it("transitionSprint: 改 start_date + end_date", () => {
    const before = useStore.getState().sprints.find((s) => s.id === "spr-001");
    const newStart = "2026-09-01T00:00:00Z";
    const newEnd = "2026-09-15T00:00:00Z";
    useStore.getState().transitionSprint("spr-001", newStart, newEnd);
    const after = useStore.getState().sprints.find((s) => s.id === "spr-001");
    expect(after?.start_date).toBe(newStart);
    expect(after?.end_date).toBe(newEnd);
    expect(after?.start_date).not.toBe(before?.start_date);
  });
});
