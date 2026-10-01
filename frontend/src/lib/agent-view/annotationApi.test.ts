import { describe, expect, it, beforeEach } from "vitest";
import {
  loadAgentAnnotations,
  saveAgentAnnotations,
  clearAgentAnnotations,
  listPersistedAgentIds,
} from "./annotationApi";
import type { AgentCanvasAnnotation } from "./types";

beforeEach(() => {
  if (typeof window !== "undefined") {
    // 清掉所有 star-agent-view-annotations:* key
    const ids = listPersistedAgentIds();
    ids.forEach((id) => clearAgentAnnotations(id));
  }
});

const sampleAnnotation: AgentCanvasAnnotation = {
  id: "ann-001",
  kind: "sticky_note",
  x: 100,
  y: 100,
  width: 180,
  height: 100,
  created_at: "2026-10-01T00:00:00Z",
  created_by: "usr-001",
  content: { color: "#f9d77e", text: "Hello" },
};

describe("annotationApi 本地持久化 (per 2026-10-01 OOB 任务 #1)", () => {
  it("A. 空 storage 返回空 state", () => {
    const result = loadAgentAnnotations("ag-001");
    expect(result.annotations).toEqual([]);
    expect(result.freeConnectors).toEqual([]);
  });

  it("B. save 后 load 能拿回 annotations + connectors", () => {
    saveAgentAnnotations("ag-001", [sampleAnnotation], []);
    const result = loadAgentAnnotations("ag-001");
    expect(result.annotations).toHaveLength(1);
    expect(result.annotations[0]).toEqual(sampleAnnotation);
    expect(result.savedAt).toBeTruthy();
  });

  it("C. 不同 agentId 独立存储空间", () => {
    saveAgentAnnotations("ag-001", [sampleAnnotation], []);
    saveAgentAnnotations("ag-002", [], []);
    expect(loadAgentAnnotations("ag-001").annotations).toHaveLength(1);
    expect(loadAgentAnnotations("ag-002").annotations).toHaveLength(0);
  });

  it("D. clear 后 load 返回空", () => {
    saveAgentAnnotations("ag-001", [sampleAnnotation], []);
    clearAgentAnnotations("ag-001");
    expect(loadAgentAnnotations("ag-001").annotations).toEqual([]);
  });

  it("E. JSON 损坏 / 异常 shape 时 fallback 到空", () => {
    if (typeof window !== "undefined") {
      window.localStorage.setItem("star-agent-view-annotations:ag-007", "{not valid json");
    }
    const result = loadAgentAnnotations("ag-007");
    expect(result.annotations).toEqual([]);
  });

  it("F. listPersistedAgentIds 列出所有保存的 agent-id", () => {
    saveAgentAnnotations("ag-A", [], []);
    saveAgentAnnotations("ag-B", [], []);
    const ids = listPersistedAgentIds();
    expect(ids.sort()).toEqual(["ag-A", "ag-B"]);
  });
});
