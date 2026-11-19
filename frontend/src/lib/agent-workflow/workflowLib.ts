/*
@cypher schema=1 source_sha256=da18510bb6ae002282f4ac7c9f0aa1b5cfecc4f25fd755889f26b8b60aee8c18
MERGE (self:File {path:"frontend/src/lib/agent-workflow/workflowLib.ts"})
MERGE (makeIpaSkillNode:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::makeIpaSkillNode",kind:"function"})
MERGE (makeControlNode:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::makeControlNode",kind:"function"})
MERGE (wireExists:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::wireExists",kind:"function"})
MERGE (canConnect:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::canConnect",kind:"function"})
MERGE (deriveNodePorts:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::deriveNodePorts",kind:"function"})
MERGE (reconcileNodePorts:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::reconcileNodePorts",kind:"function"})
MERGE (evaluateRule:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::evaluateRule",kind:"function"})
MERGE (validateWorkflow:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::validateWorkflow",kind:"function"})
MERGE (simulateWorkflow:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::simulateWorkflow",kind:"function"})
MERGE (switchRulesOf:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::switchRulesOf",kind:"function"})
MERGE (mergeInputCountOf:Symbol {id:"frontend/src/lib/agent-workflow/workflowLib.ts::mergeInputCountOf",kind:"function"})
MERGE (self)-[:DEFINES]->(makeIpaSkillNode)
MERGE (self)-[:DEFINES]->(makeControlNode)
MERGE (self)-[:DEFINES]->(wireExists)
MERGE (self)-[:DEFINES]->(canConnect)
MERGE (self)-[:DEFINES]->(deriveNodePorts)
MERGE (self)-[:DEFINES]->(reconcileNodePorts)
MERGE (self)-[:DEFINES]->(evaluateRule)
MERGE (self)-[:DEFINES]->(validateWorkflow)
MERGE (self)-[:DEFINES]->(simulateWorkflow)
MERGE (self)-[:DEFINES]->(switchRulesOf)
MERGE (self)-[:DEFINES]->(mergeInputCountOf)
MERGE (makeControlNode)-[:CALLS]->(deriveNodePorts)
MERGE (deriveNodePorts)-[:CALLS]->(switchRulesOf)
MERGE (deriveNodePorts)-[:CALLS]->(mergeInputCountOf)
MERGE (reconcileNodePorts)-[:CALLS]->(deriveNodePorts)
MERGE (validateWorkflow)-[:CALLS]->(switchRulesOf)
MERGE (simulateWorkflow)-[:CALLS]->(evaluateRule)
MERGE (simulateWorkflow)-[:CALLS]->(switchRulesOf)
@endcypher
*/
import type {
  WorkflowNode,
  WorkflowNodeKind,
  WorkflowPort,
  WorkflowPortDataType,
  WorkflowWire,
  WorkflowConditionRule,
} from "@/lib/agent-workflow/types";
import type { IpaSkillEntry } from "@/lib/ipa-skills/catalog";

let nodeSeq = 0;
let wireSeq = 0;

export function nextNodeId(): string {
  nodeSeq += 1;
  return `node-${Date.now().toString(36)}-${nodeSeq}`;
}

export function nextWireId(): string {
  wireSeq += 1;
  return `wire-${Date.now().toString(36)}-${wireSeq}`;
}

interface ControlNodeDef {
  kind: WorkflowNodeKind;
  title: string;
  subtitle: string;
  inputs: WorkflowPort[];
  outputs: WorkflowPort[];
  defaultConfig: Record<string, unknown>;
}

const port = (
  id: string,
  label: string,
  direction: "in" | "out",
  dataType: WorkflowPortDataType = "any",
): WorkflowPort => ({ id, label, direction, dataType });

/** 控制流组件库 — n8n 同构: trigger / if-else / for / switch / merge / code / http / wait */
export const CONTROL_NODE_LIBRARY: ControlNodeDef[] = [
  {
    kind: "trigger",
    title: "手動トリガー",
    subtitle: "Manual Trigger",
    inputs: [],
    outputs: [port("start", "Start", "out", "trigger")],
    defaultConfig: { mode: "manual" },
  },
  {
    kind: "if-else",
    title: "条件分岐 (If-Else)",
    subtitle: "If / Else",
    inputs: [port("in", "Input", "in", "any")],
    outputs: [
      port("true", "True 成立", "out", "any"),
      port("false", "False 不成立", "out", "any"),
    ],
    defaultConfig: {
      rules: [{ field: "status", operator: "equals", value: "ok" }],
    },
  },
  {
    kind: "for-loop",
    title: "繰り返し (For Loop)",
    subtitle: "For Loop",
    inputs: [port("in", "Input", "in", "data")],
    outputs: [
      port("loop", "Loop 循环体", "out", "data"),
      port("items", "Items 结果集", "out", "data"),
      port("done", "Done 结束", "out", "trigger"),
    ],
    defaultConfig: { iterateMode: "count", count: 3, expression: "$items" },
  },
  {
    kind: "switch",
    title: "スイッチ (Switch)",
    subtitle: "Switch / Router",
    inputs: [port("in", "Input", "in", "any")],
    outputs: [
      port("case-1", "Case 1: kind", "out", "any"),
      port("default", "Default 兜底", "out", "any"),
    ],
    defaultConfig: {
      rules: [{ field: "kind", operator: "equals", value: "doc" }],
    },
  },
  {
    kind: "merge",
    title: "マージ (Merge)",
    subtitle: "Merge N inputs",
    inputs: [
      port("in-1", "输入 1", "in", "any"),
      port("in-2", "输入 2", "in", "any"),
    ],
    outputs: [port("out", "Output", "out", "any")],
    defaultConfig: { mode: "append", inputCount: 2 },
  },
  {
    kind: "code",
    title: "コード (Code)",
    subtitle: "JavaScript / Python",
    inputs: [port("in", "Input", "in", "data")],
    outputs: [port("out", "Output", "out", "data")],
    defaultConfig: { language: "javascript", code: "// transform $input -> $output\nreturn $input;" },
  },
  {
    kind: "http",
    title: "HTTP リクエスト",
    subtitle: "HTTP Request",
    inputs: [port("in", "Input", "in", "trigger")],
    outputs: [
      port("out", "Response", "out", "data"),
      port("error", "Error", "out", "data"),
    ],
    defaultConfig: { method: "GET", url: "https://api.example.com/" },
  },
  {
    kind: "wait",
    title: "待機 (Wait)",
    subtitle: "Wait / Delay",
    inputs: [port("in", "Input", "in", "trigger")],
    outputs: [port("out", "After wait", "out", "trigger")],
    defaultConfig: { seconds: 5 },
  },
];

/** IPA 技能节点: 输入端口取自 workflow-contract required_inputs, 输出端口取自 required_outputs。 */
export function makeIpaSkillNode(skill: IpaSkillEntry, position: { x: number; y: number }): WorkflowNode {
  const inputs: WorkflowPort[] = skill.inputs
    .slice(0, 6)
    .map((label, index) => port(`in-${index + 1}`, label.slice(0, 18), "in", "evidence"));
  const outputs: WorkflowPort[] = (skill.outputs.length > 0 ? skill.outputs : ["document"])
    .slice(0, 4)
    .map((kind, index) => port(`out-${index + 1}`, kind, "out", kind as WorkflowPortDataType));
  return {
    id: nextNodeId(),
    kind: "ipa-skill",
    title: skill.titleJa,
    subtitle: `${skill.id} · ${skill.phaseJa}`,
    skillId: skill.id,
    position,
    inputs,
    outputs,
    config: {
      skillId: skill.id,
      mode: skill.mode,
      hasHumanGate: skill.hasHumanGate,
      projectUid: "",
      taskUid: "",
    },
  };
}

export function makeControlNode(
  def: ControlNodeDef,
  position: { x: number; y: number },
): WorkflowNode {
  const node: WorkflowNode = {
    id: nextNodeId(),
    kind: def.kind,
    title: def.title,
    subtitle: def.subtitle,
    position,
    inputs: def.inputs.map((p) => ({ ...p })),
    outputs: def.outputs.map((p) => ({ ...p })),
    config: JSON.parse(JSON.stringify(def.defaultConfig)),
  };
  // 配置驱动端口: switch/merge 的端口数由 defaultConfig 派生, 与后续编辑一致
  const derived = deriveNodePorts(node);
  return { ...node, inputs: derived.inputs, outputs: derived.outputs };
}

export function wireExists(wires: WorkflowWire[], fromNode: string, fromPort: string, toNode: string, toPort: string): boolean {
  return wires.some(
    (w) => w.fromNode === fromNode && w.fromPort === fromPort && w.toNode === toNode && w.toPort === toPort,
  );
}

/** 端口可连: 同 dataType 或任一端 any; 输入端口只接一条线。 */
export function canConnect(
  wires: WorkflowWire[],
  from: WorkflowPort,
  to: WorkflowPort,
): boolean {
  if (from.direction !== "out" || to.direction !== "in") return false;
  if (from.dataType !== "any" && to.dataType !== "any" && from.dataType !== to.dataType) {
    return false;
  }
  const occupied = wires.some((w) => w.toPort === to.id && w.toNode === to.id);
  return !occupied;
}

/** 默认示例工作流: trigger → if-else → (分支) ipa skill。 */
export function createExampleWorkflow(): { nodes: WorkflowNode[]; wires: WorkflowWire[] } {
  const trigger = makeControlNode(CONTROL_NODE_LIBRARY[0], { x: 80, y: 220 });
  const condition = makeControlNode(CONTROL_NODE_LIBRARY[1], { x: 360, y: 220 });
  const wires: WorkflowWire[] = [
    { id: nextWireId(), fromNode: trigger.id, fromPort: "start", toNode: condition.id, toPort: "in" },
  ];
  return { nodes: [trigger, condition], wires };
}

// =====================================================================
// 配置驱动端口派生 — switch 分支数 / merge 输入数随配置实时变化
// (per 2026-11-19 用户反馈: switch 加分支后输出端口数量必须跟着变)
// =====================================================================

export function switchRulesOf(node: WorkflowNode): WorkflowConditionRule[] {
  const rules = node.config.rules;
  return Array.isArray(rules) ? (rules as WorkflowConditionRule[]) : [];
}

export function mergeInputCountOf(node: WorkflowNode): number {
  const raw = Number(node.config.inputCount);
  return Number.isFinite(raw) ? Math.min(8, Math.max(2, Math.round(raw))) : 2;
}

/**
 * 按节点 kind + config 派生端口。switch: 每条规则一个 Case 输出 + Default 兜底;
 * merge: inputCount 个输入。其余 kind 保持静态端口。端口 id 稳定 (case-N / in-N),
 * 增删分支/输入时已有连线尽量保留。
 */
export function deriveNodePorts(node: WorkflowNode): { inputs: WorkflowPort[]; outputs: WorkflowPort[] } {
  if (node.kind === "switch") {
    const rules = switchRulesOf(node);
    return {
      inputs: node.inputs,
      outputs: [
        ...rules.map((rule, i) =>
          port(
            `case-${i + 1}`,
            rule.field ? `Case ${i + 1}: ${rule.field}` : `Case ${i + 1}`,
            "out",
          ),
        ),
        port("default", "Default 兜底", "out"),
      ],
    };
  }
  if (node.kind === "merge") {
    const count = mergeInputCountOf(node);
    return {
      inputs: Array.from({ length: count }, (_, i) => port(`in-${i + 1}`, `输入 ${i + 1}`, "in")),
      outputs: node.outputs,
    };
  }
  return { inputs: node.inputs, outputs: node.outputs };
}

/**
 * 配置变更后重算端口, 并清理引用已消失端口的连线 (删分支/减输入时)。
 * 返回新的 nodes/wires; 无变化时返回原引用。
 */
export function reconcileNodePorts(
  nodes: WorkflowNode[],
  wires: WorkflowWire[],
): { nodes: WorkflowNode[]; wires: WorkflowWire[] } {
  let nodesChanged = false;
  const nextNodes = nodes.map((n) => {
    const derived = deriveNodePorts(n);
    const same =
      derived.inputs.length === n.inputs.length &&
      derived.outputs.length === n.outputs.length &&
      derived.inputs.every((p, i) => p.id === n.inputs[i]?.id && p.label === n.inputs[i]?.label) &&
      derived.outputs.every((p, i) => p.id === n.outputs[i]?.id && p.label === n.outputs[i]?.label);
    if (same) return n;
    nodesChanged = true;
    return { ...n, inputs: derived.inputs, outputs: derived.outputs };
  });
  if (!nodesChanged) return { nodes, wires };
  const validWires = wires.filter((w) => {
    const from = nextNodes.find((n) => n.id === w.fromNode);
    const to = nextNodes.find((n) => n.id === w.toNode);
    return (
      !!from?.outputs.some((p) => p.id === w.fromPort) &&
      !!to?.inputs.some((p) => p.id === w.toPort)
    );
  });
  return { nodes: nextNodes, wires: validWires };
}

// =====================================================================
// 条件求值 + 工作流校验 + 模拟执行 (纯函数, 可单测)
// =====================================================================

export function evaluateRule(rule: WorkflowConditionRule, data: Record<string, unknown>): boolean {
  const actual = data[rule.field];
  switch (rule.operator) {
    case "exists":
      return actual !== undefined && actual !== null && actual !== "";
    case "equals":
      return String(actual ?? "") === rule.value;
    case "not_equals":
      return String(actual ?? "") !== rule.value;
    case "contains":
      return String(actual ?? "").includes(rule.value);
    case "gt":
      return Number(actual) > Number(rule.value);
    case "lt":
      return Number(actual) < Number(rule.value);
    default:
      return false;
  }
}

export interface WorkflowIssue {
  nodeId?: string;
  level: "error" | "warning";
  message: string;
}

/** 结构校验: 触发器唯一、无悬空连线、输入端口占用、分支规则完整性。 */
export function validateWorkflow(nodes: WorkflowNode[], wires: WorkflowWire[]): WorkflowIssue[] {
  const issues: WorkflowIssue[] = [];
  const triggers = nodes.filter((n) => n.kind === "trigger");
  if (triggers.length === 0) issues.push({ level: "error", message: "缺少触发器节点" });
  if (triggers.length > 1) issues.push({ level: "warning", message: `存在 ${triggers.length} 个触发器, 模拟仅从第一个开始` });
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const occupied = new Set<string>();
  for (const w of wires) {
    const from = byId.get(w.fromNode);
    const to = byId.get(w.toNode);
    if (!from || !to) {
      issues.push({ level: "error", message: `连线 ${w.id} 引用了不存在的节点` });
      continue;
    }
    if (!from.outputs.some((p) => p.id === w.fromPort)) {
      issues.push({ nodeId: from.id, level: "error", message: `连线引用了 ${from.title} 上不存在的输出端口 ${w.fromPort}` });
    }
    if (!to.inputs.some((p) => p.id === w.toPort)) {
      issues.push({ nodeId: to.id, level: "error", message: `连线引用了 ${to.title} 上不存在的输入端口 ${w.toPort}` });
    }
    const key = `${w.toNode}:${w.toPort}`;
    if (occupied.has(key)) {
      issues.push({ nodeId: to.id, level: "error", message: `${to.title} 的输入端口 ${w.toPort} 被多条连线占用` });
    }
    occupied.add(key);
  }
  for (const n of nodes) {
    if (n.kind === "switch") {
      const rules = switchRulesOf(n);
      if (rules.length === 0) issues.push({ nodeId: n.id, level: "warning", message: `${n.title} 没有分支规则, 全部流量走 Default` });
      rules.forEach((r, i) => {
        if (!r.field.trim()) issues.push({ nodeId: n.id, level: "warning", message: `${n.title} Case ${i + 1} 未填写字段` });
      });
    }
    if (n.kind === "if-else") {
      const rules = Array.isArray(n.config.rules) ? (n.config.rules as WorkflowConditionRule[]) : [];
      if (rules.length === 0) issues.push({ nodeId: n.id, level: "warning", message: `${n.title} 没有条件规则, 恒走 True` });
    }
    if (n.kind === "http" && !String(n.config.url ?? "").trim()) {
      issues.push({ nodeId: n.id, level: "warning", message: `${n.title} 未配置 URL` });
    }
    if (n.kind !== "trigger" && n.inputs.length > 0) {
      const hasIncoming = wires.some((w) => w.toNode === n.id);
      if (!hasIncoming) issues.push({ nodeId: n.id, level: "warning", message: `${n.title} 没有输入连线, 不会被执行` });
    }
  }
  return issues;
}

export interface SimulationStep {
  nodeId: string;
  title: string;
  viaPort?: string;
  detail: string;
}

export interface SimulationResult {
  steps: SimulationStep[];
  /** 按节点 id 记录每个输出端口是否被走到 (用于画布高亮)。 */
  takenPorts: Record<string, string[]>;
  truncated: boolean;
}

const MAX_SIM_STEPS = 500;

/**
 * 从触发器开始沿连线做确定性模拟: if-else 按规则求值选 True/False,
 * switch 顺序匹配选 Case-N 否则 Default, for-loop 按次数展开 Loop 边。
 * 数据载荷来自 trigger.config.payload (JSON), 缺省 {}。
 */
export function simulateWorkflow(
  nodes: WorkflowNode[],
  wires: WorkflowWire[],
  payload?: Record<string, unknown>,
): SimulationResult {
  const steps: SimulationStep[] = [];
  const takenPorts: Record<string, string[]> = {};
  let truncated = false;
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const outWires = (nodeId: string, portId: string) =>
    wires.filter((w) => w.fromNode === nodeId && w.fromPort === portId);
  const mark = (nodeId: string, portId: string) => {
    (takenPorts[nodeId] ??= []).push(portId);
  };
  const data: Record<string, unknown> =
    payload ??
    (() => {
      const trigger = nodes.find((n) => n.kind === "trigger");
      const raw = trigger?.config.payload;
      if (typeof raw === "string" && raw.trim()) {
        try {
          const parsed = JSON.parse(raw);
          if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) return parsed as Record<string, unknown>;
        } catch {
          /* 非法 JSON 按空载荷 */
        }
      }
      return {};
    })();

  const visit = (nodeId: string, viaPort?: string): void => {
    if (steps.length >= MAX_SIM_STEPS) {
      truncated = true;
      return;
    }
    const node = byId.get(nodeId);
    if (!node) return;
    const follow = (portId: string) => {
      mark(nodeId, portId);
      for (const w of outWires(nodeId, portId)) visit(w.toNode, w.toPort);
    };
    switch (node.kind) {
      case "trigger":
        steps.push({ nodeId, title: node.title, viaPort, detail: "触发, 载荷 " + JSON.stringify(data) });
        follow("start");
        break;
      case "if-else": {
        const rules = Array.isArray(node.config.rules) ? (node.config.rules as WorkflowConditionRule[]) : [];
        const pass = rules.every((r) => evaluateRule(r, data));
        steps.push({ nodeId, title: node.title, viaPort, detail: `${rules.length} 条规则 → ${pass ? "True" : "False"}` });
        follow(pass ? "true" : "false");
        break;
      }
      case "switch": {
        const rules = switchRulesOf(node);
        const idx = rules.findIndex((r) => evaluateRule(r, data));
        const portId = idx >= 0 ? `case-${idx + 1}` : "default";
        steps.push({ nodeId, title: node.title, viaPort, detail: idx >= 0 ? `命中 Case ${idx + 1}` : "无命中 → Default" });
        follow(portId);
        break;
      }
      case "for-loop": {
        const mode = String(node.config.iterateMode ?? "count");
        const count = mode === "count" ? Math.max(1, Number(node.config.count ?? 3)) : 1;
        steps.push({ nodeId, title: node.title, viaPort, detail: mode === "count" ? `循环 ${count} 次` : "遍历表达式 (模拟按 1 次)" });
        for (let i = 0; i < count; i++) follow("loop");
        follow("items");
        follow("done");
        break;
      }
      case "merge": {
        const incoming = wires.filter((w) => w.toNode === nodeId).length;
        steps.push({ nodeId, title: node.title, viaPort, detail: `合并 ${incoming} 路输入 (append)` });
        follow("out");
        break;
      }
      case "wait":
        steps.push({ nodeId, title: node.title, viaPort, detail: `等待 ${Number(node.config.seconds ?? 5)}s (模拟跳过)` });
        follow("out");
        break;
      case "http":
        steps.push({ nodeId, title: node.title, viaPort, detail: `${String(node.config.method ?? "GET")} ${String(node.config.url ?? "(未配置)")} (模拟不发起请求)` });
        follow("out");
        break;
      case "code":
        steps.push({ nodeId, title: node.title, viaPort, detail: `代码节点 (${String(node.config.language ?? "javascript")}, 模拟不执行)` });
        follow("out");
        break;
      case "ipa-skill":
        steps.push({ nodeId, title: node.title, viaPort, detail: `IPA 技能 ${node.skillId ?? "?"} (模拟不执行)` });
        for (const p of node.outputs) follow(p.id);
        break;
      default:
        steps.push({ nodeId, title: node.title, viaPort, detail: "通过" });
        for (const p of node.outputs) follow(p.id);
    }
  };

  const trigger = nodes.find((n) => n.kind === "trigger");
  if (trigger) visit(trigger.id);
  return { steps, takenPorts, truncated };
}
