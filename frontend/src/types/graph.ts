// =====================================================================
// Star Platform — Graph (Cypher) Type Definitions
// =====================================================================
// 来源: docs/architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md
//       §2.1 节点/边数据模型 (25 domain 投影, 1-hop 查询)
//       §2.3.5 API 契约 3 endpoint
//
// 用途: ArchGraphModal 渲染层 (cytoscape) 的 TypeScript 投影层;
//       不复制 backend Rust 类型, 而是按 UI 渲染需要做的"投影层"。
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 字段对应后端契约, 必加 1 字段 = Uuid 字符串 (不强类型, 留 25 MRU 兼容)
//   - 节点/边类型 union 全列, 显式缺标 (per 守门 #11 缺标比错标)
//   - 不引 vendor SDK (Bolt/HTTP client 留 Phase 3)
// =====================================================================

import type { Uuid, Iso8601 } from "./ids";

// =====================================================================
// 1. 节点类型 (25 domain kind)
// =====================================================================
export type GraphNodeKind =
  // MRU 1-13 (13 类)
  | "tenant" | "project" | "identity" | "workspace"
  | "work_item" | "comment" | "permission_scheme" | "workflow"
  | "change_set" | "worktree" | "agent_session" | "feedback"
  | "validation_case" | "local_runtime" | "scm_repository" | "pull_request"
  // 扩展 (per 25 MRU + 22 domain 投影)
  | "design_artifact" | "context_packet"
  | "audit_event" | "automation_rule" | "notification"
  | "incident_record" | "integration_webhook"
  // 代码侧 (cypher 投影, AST 提取, 仅 LLM agent 写)
  | "cratemodule" | "symbol";

// 节点视觉编码 (per ADR-0041 §2.1)
export interface GraphNodeStyle {
  color: string;
  shape: "round-rectangle" | "rectangle" | "circle" | "ellipse"
       | "hexagon" | "octagon" | "round-octagon" | "round-triangle"
       | "round-pentagon" | "round-diamond" | "diamond" | "tag" | "barrel";
  size: number;
}

export interface GraphNode {
  /** 内部 id, 格式 "{KIND_PREFIX}:{uuid_or_path}", e.g. "WI:wi-001" */
  id: string;
  kind: GraphNodeKind;
  /** 节点显示文本 (key/title/name) */
  label: string;
  /** 是否是当前 work_item 节点 (用于高亮主色) */
  is_current: boolean;
  /** 1 = 1-hop 邻居, 2 = 2-hop (仅 code-side cratemodule/symbol) */
  hop_level: 1 | 2;
  /** 透传原始属性 (k8s metadata / git sha / file path 等) */
  properties: Record<string, unknown>;
  /** 前端 hint: 视觉编码 (与 kind 一一对应, 但允许覆盖) */
  style?: Partial<GraphNodeStyle>;
}

// =====================================================================
// 2. 边类型 (24 typed edge label)
// =====================================================================
export type GraphEdgeKind =
  | "ASSIGNED_TO" | "REPORTED_BY" | "IN_PROJECT" | "IN_WORKSPACE"
  | "ON_WORKTREE" | "PRODUCED" | "HAS_FEEDBACK" | "VALIDATED_BY"
  | "COMMENTED_ON" | "DESIGNED_BY" | "RUNS_ON" | "POWERS"
  | "INTEGRATES" | "REFERENCES" | "LIVES_IN" | "DEPENDS_ON"
  | "INHERITS_FROM" | "TRIGGERS" | "RAISED_INCIDENT" | "WEBHOOK_FOR"
  | "HAS_PR" | "TARGETS_BRANCH" | "WITH_PERMISSION" | "FOLLOWING_WORKFLOW";

export interface GraphEdge {
  id: string;
  kind: GraphEdgeKind;
  source: string;  // GraphNode.id
  target: string;  // GraphNode.id
  /** 1 = 1-hop, 2 = 2-hop (代码侧) */
  hop_level: 1 | 2;
  /** 透传原始属性 (e.g. created_at, lock_version) */
  properties?: Record<string, unknown>;
}

// =====================================================================
// 3. 完整 Graph payload (1-hop 查询结果)
// =====================================================================
export interface GraphPayload {
  work_item_id: Uuid;
  tenant_id: Uuid;
  /** sha256(work_item_id + worktree_branch + worktree_sha + source_kind + project_id) */
  fingerprint: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  stats: {
    node_count: number;
    edge_count: number;
    /** 按 kind 拆解节点数, 便于 modal footer 统计 */
    kind_breakdown: Partial<Record<GraphNodeKind, number>>;
  };
  generated_at: Iso8601;
}

// =====================================================================
// 4. API 请求/响应 (per ADR-0041 §2.3.4)
// =====================================================================

/** 数据源类型 (per 2026-09-02 02:00 JST 拍板 dataorigin_opt3) */
export type GraphSourceKind = "local" | "git";

/** POST /graph/ensure-fresh request */
export interface EnsureFreshRequest {
  work_item_id: Uuid;
  tenant_id: Uuid;
  source: GraphSourceKind;
}

/** POST /graph/ensure-fresh response (200 数据已最新) */
export interface EnsureFreshResponse {
  status: "fresh";
  graph: GraphPayload;
}

/** POST /graph/ensure-fresh response (202 agent 正在跑) */
export interface EnsureFreshPendingResponse {
  status: "running";
  retry_after_ms: number;
  /** 当前 phase, e.g. "scanning" / "ast_extract" / "llm_infer" / "upsert" */
  phase?: string;
}

/** POST /graph/cypher request */
export interface GraphCypherRequest {
  work_item_id: Uuid;
  tenant_id: Uuid;
  /** 1 = 严格 1 跳, 2 = 1 跳 + 代码侧 2 跳 (per §2.1) */
  max_hop: 1 | 2;
}

/** POST /graph/cypher response */
export type GraphCypherResponse = GraphPayload;

/** GET /graph/health response */
export interface GraphHealthResponse {
  memgraph: "up" | "down";
  agent_runtime: "up" | "down";
  last_successful_run: Iso8601 | null;
  queue_depth: number;
}

// =====================================================================
// 5. Union helper (Phase 1 mock handler 用)
// =====================================================================
export type EnsureFreshResult =
  | EnsureFreshResponse
  | EnsureFreshPendingResponse;

// =====================================================================
// 6. 节点/边视觉编码常量 (per ADR-0041 §2.1 节点视觉编码表)
// =====================================================================
export const NODE_STYLE: Record<GraphNodeKind, GraphNodeStyle> = {
  // 当前 work_item 用 cyan 主色, 在渲染层覆盖
  work_item:        { color: "#7c8499", shape: "round-rectangle", size: 48 },
  worktree:         { color: "#a78bfa", shape: "hexagon",         size: 48 },
  agent_session:    { color: "#f59e0b", shape: "diamond",         size: 44 },
  change_set:       { color: "#10b981", shape: "ellipse",         size: 44 },
  scm_repository:   { color: "#22c55e", shape: "round-triangle",  size: 48 },
  pull_request:     { color: "#ec4899", shape: "round-pentagon",  size: 44 },
  feedback:         { color: "#f43f5e", shape: "octagon",         size: 40 },
  validation_case:  { color: "#3b82f6", shape: "round-diamond",   size: 40 },
  comment:          { color: "#94a3b8", shape: "tag",             size: 36 },
  design_artifact:  { color: "#fbbf24", shape: "round-octagon",   size: 44 },
  identity:         { color: "#0ea5e9", shape: "circle",          size: 40 },
  cratemodule:      { color: "#475569", shape: "round-rectangle", size: 44 },
  symbol:           { color: "#64748b", shape: "ellipse",         size: 28 },
  tenant:           { color: "#6b7280", shape: "rectangle",       size: 36 },
  project:          { color: "#6b7280", shape: "rectangle",       size: 36 },
  workspace:        { color: "#6b7280", shape: "rectangle",       size: 36 },
  permission_scheme:{ color: "#6b7280", shape: "rectangle",       size: 36 },
  workflow:         { color: "#6b7280", shape: "rectangle",       size: 36 },
  local_runtime:    { color: "#6b7280", shape: "rectangle",       size: 36 },
  context_packet:   { color: "#6b7280", shape: "rectangle",       size: 36 },
  audit_event:      { color: "#6b7280", shape: "rectangle",       size: 36 },
  automation_rule:  { color: "#6b7280", shape: "rectangle",       size: 36 },
  notification:     { color: "#6b7280", shape: "rectangle",       size: 36 },
  incident_record:  { color: "#6b7280", shape: "rectangle",       size: 36 },
  integration_webhook:{ color: "#6b7280", shape: "rectangle",     size: 36 },
};

export const CURRENT_WI_STYLE: GraphNodeStyle = {
  color: "#00f0ff", shape: "round-rectangle", size: 64,
};

export const EDGE_STYLE = {
  hop1: { color: "#00f0ff", width: 2, dash: "solid" as const },
  hop2: { color: "#475569", width: 1, dash: "dotted" as const },
  /** 当前 work_item 直接关联的边 (高亮 + 加粗) */
  current: { color: "#00f0ff", width: 3, dash: "solid" as const },
} as const;
