/*
@cypher schema=1 source_sha256=2026-11-11-agent-workflow-lib-v1
MERGE (self:File {path:"frontend/src/lib/agent-workflow/workflowLib.ts"})
MERGE (lib:Symbol {id:"agent_workflow::workflowLib",kind:"module"})
MERGE (makeNode:Symbol {id:"agent_workflow::workflowLib.makeNode",kind:"function"})
MERGE (makeIpaSkillNode:Symbol {id:"agent_workflow::workflowLib.makeIpaSkillNode",kind:"function"})
MERGE (wireExists:Symbol {id:"agent_workflow::workflowLib.wireExists",kind:"function"})
MERGE (canConnect:Symbol {id:"agent_workflow::workflowLib.canConnect",kind:"function"})
MERGE (self)-[:DEFINES]->(makeNode)
MERGE (self)-[:DEFINES]->(makeIpaSkillNode)
MERGE (self)-[:DEFINES]->(wireExists)
MERGE (self)-[:DEFINES]->(canConnect)
@endcypher
*/
import type {
  WorkflowNode,
  WorkflowNodeKind,
  WorkflowPort,
  WorkflowPortDataType,
  WorkflowWire,
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
      port("rule-1", "分支 1", "out", "any"),
      port("fallback", "Fallback 兜底", "out", "any"),
    ],
    defaultConfig: {
      rules: [{ field: "kind", operator: "equals", value: "doc" }],
    },
  },
  {
    kind: "merge",
    title: "マージ (Merge)",
    subtitle: "Merge 2 inputs",
    inputs: [
      port("in-1", "输入 1", "in", "any"),
      port("in-2", "输入 2", "in", "any"),
    ],
    outputs: [port("out", "Output", "out", "any")],
    defaultConfig: { mode: "append" },
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
  return {
    id: nextNodeId(),
    kind: def.kind,
    title: def.title,
    subtitle: def.subtitle,
    position,
    inputs: def.inputs.map((p) => ({ ...p })),
    outputs: def.outputs.map((p) => ({ ...p })),
    config: JSON.parse(JSON.stringify(def.defaultConfig)),
  };
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
