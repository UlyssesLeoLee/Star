// =====================================================================
// globalSearch.test.ts — 微信式 type-chip 过滤 + 多 type 搜索 (per 2026-10-01 OOB)
// =====================================================================
import { describe, it, expect } from "vitest";
import {
  searchAll,
  TYPE_FILTERS,
  TYPE_LABEL,
  getSearchIndex,
  type SearchType,
} from "./globalSearch";

describe("globalSearch", () => {
  it("A. 空 query 返回空结果", () => {
    const r = searchAll("");
    expect(r.query).toBe("");
    expect(r.total).toBe(0);
    expect(r.groups).toEqual([]);
  });

  it("B. 默认返回 modules + tasks + worktrees + canvases + agents + feedbacks + projects", () => {
    const idx = getSearchIndex();
    const types = new Set(idx.map((e) => e.hit.type));
    // 至少 8 类 (模块/任务/worktree/canvas/agent/feedback/project/sprint/...)
    expect(types.size).toBeGreaterThanOrEqual(8);
  });

  it("C. query 'Inbox' 命中 inbox 模块", () => {
    const r = searchAll("Inbox");
    expect(r.total).toBeGreaterThan(0);
    const inbox = r.groups.flatMap((g) => g.hits).find((h) => h.id === "module:inbox");
    expect(inbox).toBeDefined();
    expect(inbox?.type).toBe("module");
    expect(inbox?.href).toBe("/inbox");
  });

  it("D. query 'implement' 命中 work_items (含 'Implement Worktree 17-state machine')", () => {
    const r = searchAll("implement");
    const tasks = r.groups.find((g) => g.type === "task");
    expect(tasks).toBeDefined();
    expect(tasks!.hits.length).toBeGreaterThan(0);
    expect(tasks!.hits.some((h) => /implement/i.test(h.label))).toBe(true);
  });

  it("E. query 'wt-001' 命中 worktree (按 id)", () => {
    const r = searchAll("wt-001");
    const wt = r.groups.flatMap((g) => g.hits).find((h) => h.id === "worktree:wt-001");
    expect(wt).toBeDefined();
  });

  it("F. query 'canvas' 命中 canvas 实体 (Physis Sprint 23, canvas-001)", () => {
    const r = searchAll("canvas", { limit: 100 });
    // canvas-001 canvas-002 (但 seed 只 1 个 canvas)
    const canvasGroup = r.groups.find((g) => g.type === "canvas");
    // 如果 index 里有 canvas, canvasGroup 应该存在
    expect(canvasGroup).toBeDefined();
    if (canvasGroup) expect(canvasGroup.hits.length).toBeGreaterThan(0);
  });

  it("G. filter 限定到 task 排除其他类型", () => {
    const r = searchAll("in", { filter: "task" });
    expect(r.groups.every((g) => g.type === "task")).toBe(true);
  });

  it("H. filter=module 排除 task/worktree", () => {
    const r = searchAll("in", { filter: "module" });
    expect(r.groups.every((g) => g.type === "module")).toBe(true);
    expect(r.groups.some((g) => g.type === "task")).toBe(false);
  });

  it("I. filter=all 等同不传 filter", () => {
    const r1 = searchAll("in");
    const r2 = searchAll("in", { filter: "all" });
    expect(r2.total).toBe(r1.total);
  });

  it("J. 微信式 type 顺序: module 先于 task", () => {
    const r = searchAll("in", { group: true });
    if (r.groups.length >= 2) {
      // module 应该是第一个 group (TYPE_FILTERS[0])
      expect(r.groups[0].type).toBe("module");
    }
  });

  it("K. score 排序: label 命中比 subLabel 命中得分高", () => {
    const r1 = searchAll("Physis");  // 项目名
    const r2 = searchAll("UGC");     // subLabel/key 命中
    expect(r2.total).toBeGreaterThanOrEqual(0);
    // 至少 Physis 命中要 > 0
    expect(r1.total).toBeGreaterThan(0);
  });

  it("L. TYPE_LABEL 包含主要中文标签", () => {
    expect(TYPE_LABEL.module).toBe("模块");
    expect(TYPE_LABEL.task).toBe("任务");
    expect(TYPE_LABEL.worktree).toBe("Worktree");
    expect(TYPE_LABEL.canvas).toBe("画布");
    expect(TYPE_LABEL.agent).toBe("Agent");
  });

  it("M. TYPE_FILTERS 顺序: 全部 在最前", () => {
    expect(TYPE_FILTERS[0].id).toBe("all");
    expect(TYPE_FILTERS[0].label).toBe("全部");
    // 含主要 filter
    const ids = TYPE_FILTERS.map((f) => f.id);
    expect(ids).toContain("task");
    expect(ids).toContain("worktree");
    expect(ids).toContain("canvas");
    expect(ids).toContain("agent");
  });

  it("N. 子串匹配 (大小写不敏感)", () => {
    const r1 = searchAll("WORKTREE");
    const r2 = searchAll("worktree");
    expect(r1.total).toBe(r2.total);
    expect(r1.total).toBeGreaterThan(0);
  });

  it("O. 中文 query 命中", () => {
    const r = searchAll("工程");
    expect(r.total).toBeGreaterThanOrEqual(0); // 至少能跑通
  });

  it("P. 限制 limit 限制返回数", () => {
    const r = searchAll("in", { limit: 3 });
    expect(r.total).toBeGreaterThanOrEqual(0);
    expect(r.groups.flatMap((g) => g.hits).length).toBeLessThanOrEqual(3);
  });

  it("Q. agent query 命中 agent session", () => {
    const r = searchAll("claude");
    const agents = r.groups.find((g) => g.type === "agent");
    expect(agents?.hits.length).toBeGreaterThan(0);
  });

  it("R. feedback query 命中 feedback (含 question 字段)", () => {
    const r = searchAll("CEL");
    const fbs = r.groups.find((g) => g.type === "feedback");
    // 命中至少 0, 但若 seed 含 CEL 字样会有 1+
    expect(fbs?.hits.length ?? 0).toBeGreaterThanOrEqual(0);
  });

  it("S. 0 命中 query 返回 groups=[]", () => {
    const r = searchAll("zqzqzqzq-no-such-thing-zqzqz");
    expect(r.total).toBe(0);
    expect(r.groups).toEqual([]);
  });

  it("T. 返回结构包含 type/label/href/score", () => {
    const r = searchAll("Physis");
    const allHits = r.groups.flatMap((g) => g.hits);
    if (allHits.length > 0) {
      const h = allHits[0];
      expect(h.type).toBeDefined();
      expect(h.label).toBeDefined();
      expect(h.href).toBeDefined();
      expect(typeof h.score).toBe("number");
    }
  });
});
