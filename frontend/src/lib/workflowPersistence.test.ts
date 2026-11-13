/*
@cypher schema=1 source_sha256=c5fcb066dad6817dd7bda51baf2355556415c28394bd328041376ef26ca43da4
MERGE (self:File {path:"frontend/src/lib/workflowPersistence.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/workflowPersistence.test.ts::workflowPersistence",kind:"suite"})
MERGE (store:File {path:"frontend/src/lib/store.ts"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(store)
MERGE (suite)-[:TESTS]->(store)
@endcypher
*/
// per WBS 8.6.2: 工作流持久化 store action 测试
import { describe, it, expect, beforeEach } from "vitest";
import { useStore } from "./store";
import type { WorkflowNode, WorkflowWire } from "./agent-workflow/types";

const sampleNodes: WorkflowNode[] = [
  { id: "n1", kind: "trigger", title: "Start", position: { x: 0, y: 0 }, ports: [] },
];
const sampleWires: WorkflowWire[] = [];

describe("workflow persistence (WBS 8.6.2)", () => {
  beforeEach(() => {
    useStore.setState({ savedWorkflows: [] });
  });

  it("saveWorkflowDraft 新增命名工作流并返回 id", () => {
    const id = useStore.getState().saveWorkflowDraft("我的工作流", sampleNodes, sampleWires);
    expect(id).toContain("wf-saved-");
    const saved = useStore.getState().savedWorkflows;
    expect(saved).toHaveLength(1);
    expect(saved[0].name).toBe("我的工作流");
    expect(saved[0].nodes).toHaveLength(1);
  });

  it("同名保存覆盖旧草稿 (不产生重复)", () => {
    useStore.getState().saveWorkflowDraft("dup", sampleNodes, sampleWires);
    useStore.getState().saveWorkflowDraft("dup", [...sampleNodes, { id: "n2", kind: "code", title: "X", position: { x: 1, y: 1 }, ports: [] }], sampleWires);
    const saved = useStore.getState().savedWorkflows;
    expect(saved).toHaveLength(1);
    expect(saved[0].nodes).toHaveLength(2);
  });

  it("空名称不保存", () => {
    const id = useStore.getState().saveWorkflowDraft("   ", sampleNodes, sampleWires);
    expect(id).toBe("");
    expect(useStore.getState().savedWorkflows).toHaveLength(0);
  });

  it("deleteSavedWorkflow 按 id 删除", () => {
    const id = useStore.getState().saveWorkflowDraft("todel", sampleNodes, sampleWires);
    useStore.getState().deleteSavedWorkflow(id);
    expect(useStore.getState().savedWorkflows).toHaveLength(0);
  });

  it("持久化到 localStorage (persist 中间件自动写入)", () => {
    useStore.getState().saveWorkflowDraft("persist-me", sampleNodes, sampleWires);
    const raw = window.localStorage.getItem("star-store:v1");
    expect(raw).toBeTruthy();
    expect(raw).toContain("persist-me");
  });
});
