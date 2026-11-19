/*
@cypher schema=1 source_sha256=8cfe61e09e7e13303180d2b16e01b9b9d0ad0324f1e47449848e31769ce2af8e
MERGE (self:File {path:"frontend/src/lib/agent-workflow/workflowLib.test.ts"})
MERGE (lib:File {path:"frontend/src/lib/agent-workflow/workflowLib.ts"})
MERGE (fnPorts:Function {name:"deriveNodePorts", language:"typescript"})
MERGE (fnReconcile:Function {name:"reconcileNodePorts", language:"typescript"})
MERGE (fnEval:Function {name:"evaluateRule", language:"typescript"})
MERGE (fnValidate:Function {name:"validateWorkflow", language:"typescript"})
MERGE (fnSim:Function {name:"simulateWorkflow", language:"typescript"})
MERGE (self)-[:TESTS]->(fnPorts)
MERGE (self)-[:TESTS]->(fnReconcile)
MERGE (self)-[:TESTS]->(fnEval)
MERGE (self)-[:TESTS]->(fnValidate)
MERGE (self)-[:TESTS]->(fnSim)
MERGE (self)-[:IMPORTS]->(lib)
@endcypher
*/
// workflowLib 单测 — 重点回归 2026-11-19 用户反馈:
// switch 添加分支后输出端口数量必须同步变化; 各控制流组件语义正确。
import { describe, expect, it } from "vitest";
import {
  CONTROL_NODE_LIBRARY,
  makeControlNode,
  deriveNodePorts,
  reconcileNodePorts,
  evaluateRule,
  validateWorkflow,
  simulateWorkflow,
  mergeInputCountOf,
} from "./workflowLib";
import type { WorkflowNode, WorkflowWire, WorkflowConditionRule } from "./types";

const defOf = (kind: string) => CONTROL_NODE_LIBRARY.find((d) => d.kind === kind)!;
const rule = (field: string, value: string, operator: WorkflowConditionRule["operator"] = "equals"): WorkflowConditionRule => ({
  field,
  operator,
  value,
});

function makeSwitch(rules: WorkflowConditionRule[]): WorkflowNode {
  const node = makeControlNode(defOf("switch"), { x: 0, y: 0 });
  node.config.rules = rules;
  return reconcileNodePorts([node], []).nodes[0];
}

describe("switch 端口随分支数变化 (用户反馈回归)", () => {
  it("新建 switch: 1 条默认规则 → Case 1 + Default 共 2 个输出", () => {
    const node = makeControlNode(defOf("switch"), { x: 0, y: 0 });
    expect(node.outputs.map((p) => p.id)).toEqual(["case-1", "default"]);
  });

  it("添加分支后输出端口数量同步增加", () => {
    const node = makeSwitch([rule("kind", "doc")]);
    expect(node.outputs).toHaveLength(2);
    const rules = [...(node.config.rules as WorkflowConditionRule[]), rule("kind", "img"), rule("kind", "video")];
    const next = { ...node, config: { ...node.config, rules } };
    const { nodes } = reconcileNodePorts([next], []);
    expect(nodes[0].outputs.map((p) => p.id)).toEqual(["case-1", "case-2", "case-3", "default"]);
    expect(nodes[0].outputs).toHaveLength(4);
  });

  it("删除分支后输出端口数量同步减少, 引用被删端口的连线被清理", () => {
    const node = makeSwitch([rule("kind", "doc"), rule("kind", "img"), rule("kind", "video")]);
    const target = makeControlNode(defOf("wait"), { x: 300, y: 0 });
    const wires: WorkflowWire[] = [
      { id: "w1", fromNode: node.id, fromPort: "case-2", toNode: target.id, toPort: "in" },
      { id: "w2", fromNode: node.id, fromPort: "default", toNode: target.id, toPort: "in" },
    ];
    // 删掉第 3 条分支不影响 case-2; 再删第 2 条 → case-2 消失 → w1 清理
    const rules = (node.config.rules as WorkflowConditionRule[]).slice(0, 1);
    const next = { ...node, config: { ...node.config, rules } };
    const { nodes, wires: kept } = reconcileNodePorts([next, target], wires);
    expect(nodes[0].outputs.map((p) => p.id)).toEqual(["case-1", "default"]);
    expect(kept.map((w) => w.id)).toEqual(["w2"]);
  });

  it("分支端口 label 带字段名, 便于画布识别", () => {
    const node = makeSwitch([rule("status", "ok")]);
    expect(node.outputs[0].label).toBe("Case 1: status");
  });

  it("deriveNodePorts 对非 switch/merge 节点保持原端口", () => {
    const node = makeControlNode(defOf("if-else"), { x: 0, y: 0 });
    const derived = deriveNodePorts(node);
    expect(derived.inputs).toBe(node.inputs);
    expect(derived.outputs).toBe(node.outputs);
  });
});

describe("merge 输入路数可配置", () => {
  it("inputCount=4 → 4 个输入端口", () => {
    const node = makeControlNode(defOf("merge"), { x: 0, y: 0 });
    const next = { ...node, config: { ...node.config, inputCount: 4 } };
    const { nodes } = reconcileNodePorts([next], []);
    expect(nodes[0].inputs.map((p) => p.id)).toEqual(["in-1", "in-2", "in-3", "in-4"]);
  });

  it("inputCount 钳制在 2–8", () => {
    const node = makeControlNode(defOf("merge"), { x: 0, y: 0 });
    expect(mergeInputCountOf({ ...node, config: { inputCount: 1 } })).toBe(2);
    expect(mergeInputCountOf({ ...node, config: { inputCount: 99 } })).toBe(8);
    expect(mergeInputCountOf({ ...node, config: {} })).toBe(2);
  });

  it("减少输入路数时清理悬空输入连线", () => {
    const node = makeControlNode(defOf("merge"), { x: 0, y: 0 });
    const grown = { ...node, config: { ...node.config, inputCount: 3 } };
    const { nodes: grownNodes } = reconcileNodePorts([grown], []);
    const src = makeControlNode(defOf("wait"), { x: -300, y: 0 });
    const wires: WorkflowWire[] = [
      { id: "w3", fromNode: src.id, fromPort: "out", toNode: grownNodes[0].id, toPort: "in-3" },
    ];
    const shrunk = { ...grownNodes[0], config: { ...grownNodes[0].config, inputCount: 2 } };
    const { nodes, wires: kept } = reconcileNodePorts([shrunk, src], wires);
    expect(nodes[0].inputs).toHaveLength(2);
    expect(kept).toHaveLength(0);
  });
});

describe("evaluateRule 六种运算符", () => {
  const data = { status: "ok", count: 5, name: "star-canvas", empty: "" };
  it("equals / not_equals", () => {
    expect(evaluateRule(rule("status", "ok"), data)).toBe(true);
    expect(evaluateRule(rule("status", "no"), data)).toBe(false);
    expect(evaluateRule(rule("status", "no", "not_equals"), data)).toBe(true);
  });
  it("contains", () => {
    expect(evaluateRule(rule("name", "canvas", "contains"), data)).toBe(true);
    expect(evaluateRule(rule("name", "moon", "contains"), data)).toBe(false);
  });
  it("gt / lt 数值比较", () => {
    expect(evaluateRule(rule("count", "3", "gt"), data)).toBe(true);
    expect(evaluateRule(rule("count", "9", "lt"), data)).toBe(true);
    expect(evaluateRule(rule("count", "5", "gt"), data)).toBe(false);
  });
  it("exists: 空串/缺失/null 均视为不存在", () => {
    expect(evaluateRule(rule("status", "", "exists"), data)).toBe(true);
    expect(evaluateRule(rule("empty", "", "exists"), data)).toBe(false);
    expect(evaluateRule(rule("missing", "", "exists"), data)).toBe(false);
    expect(evaluateRule(rule("nulled", "", "exists"), { nulled: null })).toBe(false);
  });
});

describe("validateWorkflow 结构校验", () => {
  it("无触发器 → error", () => {
    const issues = validateWorkflow([], []);
    expect(issues.some((i) => i.level === "error" && i.message.includes("触发器"))).toBe(true);
  });
  it("悬空连线 / 占用输入端口 → error", () => {
    const trigger = makeControlNode(defOf("trigger"), { x: 0, y: 0 });
    const wait = makeControlNode(defOf("wait"), { x: 200, y: 0 });
    const wires: WorkflowWire[] = [
      { id: "w1", fromNode: trigger.id, fromPort: "start", toNode: wait.id, toPort: "in" },
      { id: "w2", fromNode: trigger.id, fromPort: "start", toNode: wait.id, toPort: "in" },
      { id: "w3", fromNode: trigger.id, fromPort: "start", toNode: "ghost", toPort: "in" },
    ];
    const issues = validateWorkflow([trigger, wait], wires);
    expect(issues.filter((i) => i.level === "error")).toHaveLength(2);
  });
  it("switch 空规则 / 未填字段 → warning", () => {
    const trigger = makeControlNode(defOf("trigger"), { x: 0, y: 0 });
    const sw = makeSwitch([]);
    const issues = validateWorkflow([trigger, sw], []);
    expect(issues.some((i) => i.level === "warning" && i.message.includes("Default"))).toBe(true);
  });
  it("合法示例工作流无 error", () => {
    const trigger = makeControlNode(defOf("trigger"), { x: 0, y: 0 });
    const cond = makeControlNode(defOf("if-else"), { x: 200, y: 0 });
    const wires: WorkflowWire[] = [
      { id: "w1", fromNode: trigger.id, fromPort: "start", toNode: cond.id, toPort: "in" },
    ];
    expect(validateWorkflow([trigger, cond], wires).filter((i) => i.level === "error")).toHaveLength(0);
  });
});

describe("simulateWorkflow 控制流语义", () => {
  function chain() {
    const trigger = makeControlNode(defOf("trigger"), { x: 0, y: 0 });
    const sw = makeSwitch([rule("kind", "doc"), rule("kind", "img")]);
    const docWait = makeControlNode(defOf("wait"), { x: 400, y: 0 });
    const imgWait = makeControlNode(defOf("wait"), { x: 400, y: 200 });
    const defWait = makeControlNode(defOf("wait"), { x: 400, y: 400 });
    const wires: WorkflowWire[] = [
      { id: "w0", fromNode: trigger.id, fromPort: "start", toNode: sw.id, toPort: "in" },
      { id: "w1", fromNode: sw.id, fromPort: "case-1", toNode: docWait.id, toPort: "in" },
      { id: "w2", fromNode: sw.id, fromPort: "case-2", toNode: imgWait.id, toPort: "in" },
      { id: "w3", fromNode: sw.id, fromPort: "default", toNode: defWait.id, toPort: "in" },
    ];
    return { trigger, sw, docWait, imgWait, defWait, wires };
  }

  it("switch 命中 Case 2 → 只走 case-2 分支", () => {
    const { trigger, sw, docWait, imgWait, defWait, wires } = chain();
    const result = simulateWorkflow([trigger, sw, docWait, imgWait, defWait], wires, { kind: "img" });
    expect(result.steps.map((s) => s.nodeId)).toEqual([trigger.id, sw.id, imgWait.id]);
    expect(result.takenPorts[sw.id]).toEqual(["case-2"]);
  });

  it("switch 无命中 → Default", () => {
    const { trigger, sw, docWait, imgWait, defWait, wires } = chain();
    const result = simulateWorkflow([trigger, sw, docWait, imgWait, defWait], wires, { kind: "other" });
    expect(result.steps[result.steps.length - 1].nodeId).toBe(defWait.id);
    expect(result.takenPorts[sw.id]).toEqual(["default"]);
  });

  it("if-else 全部规则满足走 True, 否则 False", () => {
    const trigger = makeControlNode(defOf("trigger"), { x: 0, y: 0 });
    const cond = makeControlNode(defOf("if-else"), { x: 200, y: 0 });
    const t = makeControlNode(defOf("wait"), { x: 400, y: 0 });
    const f = makeControlNode(defOf("wait"), { x: 400, y: 200 });
    const wires: WorkflowWire[] = [
      { id: "w0", fromNode: trigger.id, fromPort: "start", toNode: cond.id, toPort: "in" },
      { id: "w1", fromNode: cond.id, fromPort: "true", toNode: t.id, toPort: "in" },
      { id: "w2", fromNode: cond.id, fromPort: "false", toNode: f.id, toPort: "in" },
    ];
    const nodes = [trigger, cond, t, f];
    expect(simulateWorkflow(nodes, wires, { status: "ok" }).takenPorts[cond.id]).toEqual(["true"]);
    expect(simulateWorkflow(nodes, wires, { status: "bad" }).takenPorts[cond.id]).toEqual(["false"]);
  });

  it("for-loop count=3 → Loop 边展开 3 次 + Items + Done", () => {
    const trigger = makeControlNode(defOf("trigger"), { x: 0, y: 0 });
    const loop = makeControlNode(defOf("for-loop"), { x: 200, y: 0 });
    const body = makeControlNode(defOf("wait"), { x: 400, y: 0 });
    const wires: WorkflowWire[] = [
      { id: "w0", fromNode: trigger.id, fromPort: "start", toNode: loop.id, toPort: "in" },
      { id: "w1", fromNode: loop.id, fromPort: "loop", toNode: body.id, toPort: "in" },
    ];
    const result = simulateWorkflow([trigger, loop, body], wires, {});
    expect(result.takenPorts[loop.id]).toEqual(["loop", "loop", "loop", "items", "done"]);
    expect(result.steps.filter((s) => s.nodeId === body.id)).toHaveLength(3);
  });

  it("trigger.config.payload JSON 作为缺省载荷", () => {
    const { trigger, sw, docWait, imgWait, defWait, wires } = chain();
    trigger.config.payload = '{"kind":"doc"}';
    const result = simulateWorkflow([trigger, sw, docWait, imgWait, defWait], wires);
    expect(result.takenPorts[sw.id]).toEqual(["case-1"]);
  });

  it("非法 payload JSON 按空载荷处理, 不抛异常", () => {
    const { trigger, sw, docWait, imgWait, defWait, wires } = chain();
    trigger.config.payload = "{not json";
    const result = simulateWorkflow([trigger, sw, docWait, imgWait, defWait], wires);
    expect(result.takenPorts[sw.id]).toEqual(["default"]);
  });

  it("无触发器 → 空步骤", () => {
    const result = simulateWorkflow([], []);
    expect(result.steps).toHaveLength(0);
  });
});
