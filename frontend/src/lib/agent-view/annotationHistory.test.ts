import { describe, expect, it } from "vitest";
import {
  initHistory, commit, redo as redoStep, undo as undoStep, canUndo, canRedo, emptyHistory,
} from "./annotationHistory";
import type { AgentCanvasAnnotation, AgentCanvasFreeConnector } from "./types";

const sampleAnn: AgentCanvasAnnotation = {
  id: "ann-001", kind: "sticky_note", x: 0, y: 0, width: 100, height: 100,
  created_at: "2026-10-01T00:00:00Z", created_by: "usr-001",
  content: { color: "#f9d77e", text: "" },
};

describe("annotationHistory undo/redo (per 任务 #5)", () => {
  it("A. initHistory 创建空 past/current/future", () => {
    const h = initHistory([sampleAnn], []);
    expect(h.past).toEqual([]);
    expect(h.current.annotations).toHaveLength(1);
    expect(h.future).toEqual([]);
  });

  it("B. commit 推 current 到 past + 新状态", () => {
    const h0 = initHistory([], []);
    const h1 = commit(h0, { annotations: [sampleAnn], freeConnectors: [] });
    expect(h1.past).toHaveLength(1);
    expect(h1.current.annotations).toHaveLength(1);
    expect(h1.future).toEqual([]);
  });

  it("C. undo 取 past 顶, 推进 future", () => {
    const h0 = initHistory([], []);
    const h1 = commit(h0, { annotations: [sampleAnn], freeConnectors: [] });
    const h2 = undoStep(h1);
    expect(h2.current.annotations).toEqual([]);
    expect(h2.past).toEqual([]);
    expect(h2.future).toHaveLength(1);
  });

  it("D. redo 取 future 顶, 推回 past", () => {
    const h0 = initHistory([], []);
    const h1 = commit(h0, { annotations: [sampleAnn], freeConnectors: [] });
    const h2 = undoStep(h1);
    const h3 = redoStep(h2);
    expect(h3.current.annotations).toHaveLength(1);
    expect(h3.future).toEqual([]);
    expect(h3.past).toHaveLength(1);
  });

  it("E. 新 commit 后清空 redo stack", () => {
    const h0 = initHistory([], []);
    const h1 = commit(h0, { annotations: [sampleAnn], freeConnectors: [] });
    const h2 = undoStep(h1);
    expect(h2.future).toHaveLength(1);
    const h3 = commit(h2, { annotations: [{ ...sampleAnn, x: 100 }], freeConnectors: [] });
    expect(h3.future).toEqual([]);
  });

  it("F. 重复 commit 限制在 50 步", () => {
    let h = emptyHistory();
    for (let i = 0; i < 60; i += 1) {
      h = commit(h, { annotations: [{ ...sampleAnn, x: i }], freeConnectors: [] });
    }
    expect(h.past).toHaveLength(50);
  });

  it("G. canUndo / canRedo helper 反映 past/future 长度", () => {
    const h0 = initHistory([], []);
    expect(canUndo(h0)).toBe(false);
    expect(canRedo(h0)).toBe(false);
    const h1 = commit(h0, { annotations: [sampleAnn], freeConnectors: [] });
    expect(canUndo(h1)).toBe(true);
    expect(canRedo(h1)).toBe(false);
    const h2 = undoStep(h1);
    expect(canRedo(h2)).toBe(true);
  });

  it("H. 相同 snapshot commit 是 no-op (避免冗余 history)", () => {
    const h0 = initHistory([sampleAnn], []);
    const h1 = commit(h0, { annotations: [sampleAnn], freeConnectors: [] });
    expect(h1.past).toEqual(h0.past);
    expect(h1).toBe(h0);
  });

  it("I. freeConnectors 也参与 history", () => {
    const conn: AgentCanvasFreeConnector = { id: "fc-001", fromAnnotationId: "a", toAnnotationId: "b", color: "#fff" };
    const h0 = initHistory([], []);
    const h1 = commit(h0, { annotations: [], freeConnectors: [conn] });
    expect(h1.current.freeConnectors).toHaveLength(1);
    const h2 = undoStep(h1);
    expect(h2.current.freeConnectors).toEqual([]);
  });
});
