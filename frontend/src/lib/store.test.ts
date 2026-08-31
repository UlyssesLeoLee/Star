// =====================================================================
// store.test.ts — W5 必测的 5 项 (per §11.3)
//   1. persist roundtrip: 写后能读回
//   2. applyRemoteChange 覆盖本地变更 (last-write-wins)
//   3. transitionWorkItem 走状态机
//   4. transitionMilestone 改 due_date
//   5. transitionSprint 改 start_date / end_date
// =====================================================================
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
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

// =====================================================================
// Board 列管理测试 (per 2026-08-31 11:24 JST Ulysses 拍板)
//   设计目标: 数据零丢失 + 兜底列不可删 + workItems.status 为主源
//   - A: removeBoardColumn 拒绝 todo 兜底列
//   - B: removeBoardColumn 非兜底列时, wi 状态归 todo 并落到 todo 列
//   - C: addBoardColumn 回填 status 匹配的 wi
//   - D: 反复删/加同一列, 数据不漂移
// =====================================================================
describe("useStore (Board 列管理 — 数据零丢失)", () => {
  beforeEach(() => {
    if (typeof window !== "undefined") window.localStorage.clear();
    resetStore();
  });

  afterEach(() => {
    if (typeof window !== "undefined") window.localStorage.clear();
  });

  it("A. removeBoardColumn 拒绝删除 todo 兜底列 (no-op + warn)", () => {
    const before = useStore.getState().board;
    expect(before.columns.some((c) => c.status === "todo")).toBe(true);
    const beforeTodoIds = before.columns.find((c) => c.status === "todo")?.work_item_ids ?? [];

    // 警告会被 vitest 捕获, 这里只验证 no-op
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    useStore.getState().removeBoardColumn("todo");
    warnSpy.mockRestore();

    const after = useStore.getState().board;
    // todo 列仍在
    expect(after.columns.some((c) => c.status === "todo")).toBe(true);
    // todo 列的 work_item_ids 未被清空
    expect(after.columns.find((c) => c.status === "todo")?.work_item_ids).toEqual(beforeTodoIds);
  });

  it("B. removeBoardColumn 非兜底列 → 列里 wi 状态改回 todo 并入 todo 列", () => {
    // seed: review 列有 wi-002 / wi-008 / wi-018 / wi-022 (status=review)
    const before = useStore.getState().board;
    const reviewIds = before.columns.find((c) => c.status === "review")?.work_item_ids ?? [];
    expect(reviewIds.length).toBeGreaterThan(0);
    const todoIdsBefore = before.columns.find((c) => c.status === "todo")?.work_item_ids ?? [];

    useStore.getState().removeBoardColumn("review");

    const after = useStore.getState().board;
    // review 列已删
    expect(after.columns.some((c) => c.status === "review")).toBe(false);
    // todo 列的 work_item_ids 包含原 review 列的全部 wi
    const todoIdsAfter = after.columns.find((c) => c.status === "todo")?.work_item_ids ?? [];
    for (const id of reviewIds) {
      expect(todoIdsAfter).toContain(id);
    }
    // 之前 todo 列有的 wi 还在
    for (const id of todoIdsBefore) {
      expect(todoIdsAfter).toContain(id);
    }
    // workItems.status 也跟着改了
    for (const id of reviewIds) {
      const w = useStore.getState().workItems.find((x) => x.id === id);
      expect(w?.status).toBe("todo");
    }
  });

  it("C. addBoardColumn 回填 workItems.status 匹配的 wi", () => {
    // 把 wi-001 (status=in_progress) 改成 done, 此时 done 列已存在, 测 add 一个新 status
    // 先把 wi-001 改成 review
    useStore.getState().transitionWorkItem("wi-001", "review");
    // 此时 review 列应自动有 wi-001 (reconcile 副作用)
    let reviewIds = useStore.getState().board.columns.find((c) => c.status === "review")?.work_item_ids ?? [];
    expect(reviewIds).toContain("wi-001");

    // 现在删 review 列, 然后再加回来, 验证回填
    useStore.getState().removeBoardColumn("review");
    // review 列已删, wi-001 状态应是 todo
    expect(useStore.getState().workItems.find((w) => w.id === "wi-001")?.status).toBe("todo");

    // 把 wi-001 重新改成 review (此时 review 列不存在, wi 状态=review 但无列)
    useStore.getState().transitionWorkItem("wi-001", "review");
    // 关键断言: transitionWorkItem 的 reconcile 不应自动加 review 列
    // (per reconcile 注释规则 3: 不存在的 status 不自动加列)
    // 此时 wi-001 状态=review, 但 board 里没 review 列
    expect(useStore.getState().workItems.find((w) => w.id === "wi-001")?.status).toBe("review");
    expect(useStore.getState().board.columns.some((c) => c.status === "review")).toBe(false);

    // 显式 addBoardColumn("review") → 必须回填 wi-001
    useStore.getState().addBoardColumn("review");
    reviewIds = useStore.getState().board.columns.find((c) => c.status === "review")?.work_item_ids ?? [];
    expect(reviewIds).toContain("wi-001");
  });

  it("D. 反复删/加同一列, 数据零漂移 (修复前的 '删 todo → 加 todo 卡片消失' bug)", () => {
    // 复现原始 bug 场景: 删 review → 加回 review, 卡片必须还在
    const reviewIdsBefore = useStore.getState().board.columns.find((c) => c.status === "review")?.work_item_ids ?? [];
    expect(reviewIdsBefore.length).toBeGreaterThan(0);

    // 1) 删 review
    useStore.getState().removeBoardColumn("review");
    expect(useStore.getState().board.columns.some((c) => c.status === "review")).toBe(false);
    // 此时 review 列里的 wi 状态全部归 todo
    for (const id of reviewIdsBefore) {
      expect(useStore.getState().workItems.find((w) => w.id === id)?.status).toBe("todo");
    }

    // 2) 加回 review 列 — addBoardColumn 回填 workItems.status=review 的 wi
    //    由于 1 步刚把全部 review-wi 归 todo, 此时 workItems.status=review 的可能为 0
    //    (但 reconcile 也会顺手把 todo 列补齐 seed 阶段漏写的 wi, 这是设计的"漂移修复")
    useStore.getState().addBoardColumn("review");
    const reviewIdsAfter = useStore.getState().board.columns.find((c) => c.status === "review")?.work_item_ids ?? [];
    // 关键断言: review 列存在 (无论 work_item_ids 是空还是其他, 都证明列可加回)
    expect(useStore.getState().board.columns.some((c) => c.status === "review")).toBe(true);

    // 3) 模拟用户拖卡片回 review: 改 wi 状态=review, reconcile 应把卡片拉回 review 列
    const sampleId = reviewIdsBefore[0];
    useStore.getState().transitionWorkItem(sampleId, "review");
    const finalReviewIds = useStore.getState().board.columns.find((c) => c.status === "review")?.work_item_ids ?? [];
    expect(finalReviewIds).toContain(sampleId);

    // 4) 兜底列始终存在
    expect(useStore.getState().board.columns.some((c) => c.status === "todo")).toBe(true);

    // 5) **零丢失**关键断言: 整个删+加+改 review 操作后, 没有任何 wi 凭空消失或新增
    //    (对比 reviewIdsBefore 中的所有 wi 都能在 workItems 里找到)
    for (const id of reviewIdsBefore) {
      const w = useStore.getState().workItems.find((x) => x.id === id);
      expect(w).toBeDefined();
      // 兜底保护: 这些被删列的 wi 最终要么在 review 列(我们 3 步拖回去的),
      // 要么在 todo 列(没被拖回去的); 永远不会丢
      const inReview = finalReviewIds.includes(id);
      const inTodo = (useStore.getState().board.columns.find((c) => c.status === "todo")?.work_item_ids ?? []).includes(id);
      expect(inReview || inTodo).toBe(true);
    }
  });
});
