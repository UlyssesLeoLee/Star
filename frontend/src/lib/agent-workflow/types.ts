/*
@cypher schema=1 source_sha256=2026-11-11-agent-workflow-v1
MERGE (self:File {path:"frontend/src/lib/agent-workflow/types.ts"})
MERGE (node:Type {id:"agent_workflow::WorkflowNode",kind:"type"})
MERGE (port:Type {id:"agent_workflow::WorkflowPort",kind:"type"})
MERGE (wire:Type {id:"agent_workflow::WorkflowWire",kind:"type"})
MERGE (kind:Type {id:"agent_workflow::WorkflowNodeKind",kind:"type"})
MERGE (self)-[:DEFINES]->(node)
MERGE (self)-[:DEFINES]->(port)
MERGE (self)-[:DEFINES]->(wire)
MERGE (self)-[:DEFINES]->(kind)
@endcypher
*/
// /agent-view 工作流构建器 (n8n 风格, per 2026-11-11 用户发令)
// 每个节点卡片必带输入/输出端口; 控制流组件 (if-else / for / switch / merge) 完备。

export type WorkflowNodeKind =
  | "trigger"
  | "ipa-skill"
  | "if-else"
  | "for-loop"
  | "switch"
  | "merge"
  | "code"
  | "http"
  | "wait";

export type WorkflowPortDataType =
  | "trigger"
  | "document"
  | "evidence"
  | "review"
  | "approval"
  | "data"
  | "any";

export interface WorkflowPort {
  id: string;
  label: string;
  direction: "in" | "out";
  dataType: WorkflowPortDataType;
}

export interface WorkflowNode {
  id: string;
  kind: WorkflowNodeKind;
  title: string;
  subtitle?: string;
  /** ipa-skill 节点绑定的技能 id (对应 IPA-Atomic-Skills 目录)。 */
  skillId?: string;
  position: { x: number; y: number };
  inputs: WorkflowPort[];
  outputs: WorkflowPort[];
  config: Record<string, unknown>;
}

export interface WorkflowWire {
  id: string;
  fromNode: string;
  fromPort: string;
  toNode: string;
  toPort: string;
}

export interface WorkflowDocument {
  id: string;
  name: string;
  nodes: WorkflowNode[];
  wires: WorkflowWire[];
  updatedAt: string;
}

/** if-else / switch 条件规则 */
export interface WorkflowConditionRule {
  field: string;
  operator: "equals" | "not_equals" | "contains" | "gt" | "lt" | "exists";
  value: string;
}

export const CONDITION_OPERATORS: { id: WorkflowConditionRule["operator"]; label: string }[] = [
  { id: "equals", label: "= 等于" },
  { id: "not_equals", label: "≠ 不等于" },
  { id: "contains", label: "⊃ 包含" },
  { id: "gt", label: "> 大于" },
  { id: "lt", label: "< 小于" },
  { id: "exists", label: "∃ 存在" },
];
