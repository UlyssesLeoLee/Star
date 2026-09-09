// =====================================================================
// frontend/src/lib/arg/types.ts — TypeScript types for Agent Relationship Graph
// =====================================================================
// Per docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md v0.50 §2
// + docs/design/DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.2
// + docs/design/BD-AGENT-RELATIONSHIP-001.md v0.1 §4.3.1
//
// 1:1 镜像 crates/arg Rust 类型, 字段命名跟 serde JSON 输出对齐
// (snake_case 来自 serde, 前端读时直接用 snake_case 字段名).
//
// 6 类型族:
//   - Agent / AgentArchetype / Domain / AgentStatus (per DD §3.2.1)
//   - Edge / RelationshipType / EdgeDirection (per DD §3.2.2)
//   - TeamTemplate / TemplateId / TemplateCategory / TemplateEdge
//     + TemplateInstance (per DD §3.2.3 + §3.2.5)
//   - Achievement / AchievementCategory / Rarity / UnlockCondition
//     + AchievementUnlock (per DD §3.2.4 + §3.2.5)
//   - ARGEvent (per DD §3.2.5, 9 variants)
//   - GraphResponse (per API dto.rs §1.2)
// =====================================================================

// ----- shared primitives -----

/** UUID string (前端以 string 形式持有). */
export type Uuid = string;

/** ISO-8601 timestamp. */
export type Iso8601 = string;

// =====================================================================
// Agent (per DD §3.2.1, 16 字段)
// =====================================================================

/** 9 SA + 5 Lead + Custom = 15 variants (per DD §3.2.1). */
export type AgentArchetype =
  | "SA_01" | "SA_02" | "SA_03" | "SA_04" | "SA_05"
  | "SA_06" | "SA_07" | "SA_08" | "SA_09"
  | "LEAD_PLAYER" | "LEAD_ECONOMY" | "LEAD_MATCH" | "LEAD_SOCIAL" | "LEAD_ADMIN"
  | "CUSTOM";

/** 5 业务 sub-domains (per DD §3.2.1, 跟 AGENTS.md §5 仓库拓扑一致). */
export type Domain = "player" | "economy" | "match" | "social" | "admin";

/** 3-state machine: Active ↔ Standby → Archived. */
export type AgentStatus = "active" | "standby" | "archived";

export interface Agent {
  id: Uuid;
  name: string;
  archetype: AgentArchetype;
  /** Derived for `Lead*` variants, `null` otherwise. */
  domain: Domain | null;
  status: AgentStatus;
  /** [0.0, 1.0]. */
  trust_score: number;
  metadata: Record<string, unknown> | null;
  tenant_id: Uuid;
  created_at: Iso8601;
  updated_at: Iso8601;
  /** SCD Type 2 (per 守门 #13 c Master). */
  version: number;
  /** Author (per 守门 #10, 代签 Ulysses). */
  created_by: Uuid;
}

// =====================================================================
// Edge (per DD §3.2.2, 14 字段) + RelationshipType (10 variants)
// =====================================================================

/** 10 relationship types (4 核心 + 6 扩展, per DD §3.2.2). */
export type RelationshipType =
  // 4 核心
  | "DELEGATES_TO"
  | "CONSULTS"
  | "COLLABORATES_WITH"
  | "REPORTS_TO"
  // 6 扩展
  | "MENTORS"
  | "PEER_REVIEWS"
  | "STAND_IN_FOR"
  | "SHADOWS"
  | "CHALLENGES"
  | "TRUSTS";

/** 列出全部 10 类关系 (跟 Rust `RelationshipType::all()` 对齐). */
export const ALL_RELATIONSHIP_TYPES: readonly RelationshipType[] = [
  "DELEGATES_TO",
  "CONSULTS",
  "COLLABORATES_WITH",
  "REPORTS_TO",
  "MENTORS",
  "PEER_REVIEWS",
  "STAND_IN_FOR",
  "SHADOWS",
  "CHALLENGES",
  "TRUSTS",
] as const;

/** 方向. `Undirected` only for `COLLABORATES_WITH` / `PEER_REVIEWS`. */
export type EdgeDirection = "directed" | "undirected";

export interface Edge {
  id: Uuid;
  from_agent: Uuid;
  to_agent: Uuid;
  edge_type: RelationshipType;
  /** [0.0, 1.0]. */
  weight: number;
  direction: EdgeDirection;
  /** SCD Type 2 (per 守门 #13 c). */
  archived: boolean;
  metadata: Record<string, unknown> | null;
  tenant_id: Uuid;
  created_at: Iso8601;
  updated_at: Iso8601;
  version: number;
  created_by: Uuid;
}

// =====================================================================
// TeamTemplate (per DD §3.2.3, 5 variants) + TemplateInstance
// =====================================================================

/** 5 canonical templates (kebab-case slug for URLs). */
export type TemplateId =
  | "hub-and-spoke"
  | "mesh"
  | "chain"
  | "hierarchical"
  | "review-council";

/** 5 categories (per DD §3.2.3). */
export type TemplateCategory =
  | "standard"
  | "high_density"
  | "pipeline"
  | "management"
  | "decision";

export interface TemplateEdge {
  /** Index of source agent in `agent_ids` of the instantiate call. */
  from_index: number;
  /** Index of target agent in the same list. */
  to_index: number;
  edge_type: RelationshipType;
  default_weight: number;
}

export interface TeamTemplate {
  id: TemplateId;
  name: string;
  description: string;
  min_agents: number;
  max_agents: number;
  edges: TemplateEdge[];
  category: TemplateCategory;
}

/** TTL 30 d Work class row (per 守门 #13 a). */
export interface TemplateInstance {
  id: Uuid;
  template_id: TemplateId;
  instance_name: string;
  agent_ids: Uuid[];
  edges_json: unknown;
  tenant_id: Uuid;
  created_at: Iso8601;
  /** Soft TTL = created_at + 30 days. */
  expires_at: Iso8601;
  created_by: Uuid;
}

// =====================================================================
// Achievement (per DD §3.2.4, 20 variants) + AchievementUnlock
// =====================================================================

/** 3 评估维度 (per DD §3.2.4). */
export type AchievementCategory = "topology" | "behavior" | "output";

/** 4 稀有度 (per DD §3.2.4). */
export type Rarity = "common" | "rare" | "epic" | "legendary";

/** Unlock condition (3 变体 per DD §3.2.4). */
export type UnlockCondition =
  | { type: "cypher_query"; query: string; params: Record<string, unknown> }
  | { type: "event_pattern"; pattern: string; count: number }
  | { type: "aggregate_metric"; metric: string; threshold: number; time_window_seconds: number };

export interface Achievement {
  code: string;
  name_zh: string;
  name_en: string;
  description: string;
  category: AchievementCategory;
  rarity: Rarity;
  icon_url: string;
  unlock_condition: UnlockCondition;
}

/** Transaction class (per 守门 #13 b). */
export interface AchievementUnlock {
  id: Uuid;
  achievement_code: string;
  user_id: Uuid;
  agent_ids: Uuid[];
  trigger_metadata: Record<string, unknown> | null;
  tenant_id: Uuid;
  unlocked_at: Iso8601;
}

// =====================================================================
// ARGEvent (per DD §3.2.5, 9 variants)
// =====================================================================

/** 9-variant event enum (per DD §3.2.5). */
export type ARGEvent =
  | { type: "agent_created"; payload: Agent }
  | { type: "agent_updated"; id: Uuid; before: Agent; after: Agent }
  | { type: "agent_archived"; id: Uuid }
  | { type: "edge_created"; payload: Edge }
  | { type: "edge_updated"; id: Uuid; before: Edge; after: Edge }
  | { type: "edge_archived"; id: Uuid }
  | { type: "template_instantiated"; payload: TemplateInstance }
  | { type: "trust_score_changed"; agent_id: Uuid; before: number; after: number; delta: number }
  | { type: "achievement_unlocked"; payload: AchievementUnlock };

/** Stable string code (mirror `ARGEvent::kind()`). */
export function argEventKind(ev: ARGEvent): string {
  return ev.type;
}

// =====================================================================
// API DTOs (per crates/api/src/arg/dto.rs)
// =====================================================================

/** 跟 backend PaginationQuery (limit <= 200, defaults 50). */
export interface PaginationQuery {
  limit?: number;
  offset?: number;
}

/** 跟 backend AgentFilter. */
export interface AgentFilter {
  archetype?: AgentArchetype;
  domain?: Domain;
  status?: AgentStatus;
}

/** 跟 backend EdgeFilter. */
export interface EdgeFilter {
  edge_type?: RelationshipType;
  from_agent?: Uuid;
  to_agent?: Uuid;
  archived?: boolean;
}

/** 跟 backend CreateEdgeRequest (per DTO §4.1.3 11 fields). */
export interface CreateEdgeInput {
  from_agent: Uuid;
  to_agent: Uuid;
  edge_type: RelationshipType;
  weight: number;
  direction: EdgeDirection;
  tenant_id: Uuid;
  metadata?: Record<string, unknown>;
  created_by: Uuid;
  id?: Uuid;
  version?: number;
}

/** 跟 backend UpdateEdgeRequest. */
export interface UpdateEdgePatch {
  weight?: number;
  metadata?: Record<string, unknown>;
}

/** 跟 backend InstantiateTemplateRequest. */
export interface InstantiateTemplateInput {
  template_id: TemplateId;
  agent_ids: Uuid[];
  instance_name: string;
  tenant_id: Uuid;
  created_by: Uuid;
}

/** 跟 backend CreateAgentRequest. */
export interface CreateAgentInput {
  name: string;
  archetype: AgentArchetype;
  tenant_id: Uuid;
  trust_score?: number;
  metadata?: Record<string, unknown>;
}

/** 跟 backend UpdateAgentRequest. */
export interface UpdateAgentPatch {
  name?: string;
  trust_score?: number;
  metadata?: Record<string, unknown>;
}

/** 跟 backend GraphResponse. */
export interface GraphResponse {
  nodes: Agent[];
  edges: Edge[];
  node_count: number;
  edge_count: number;
}

/** 跟 backend GraphFilter. */
export interface GraphFilter {
  max_nodes?: number;
  tenant_id?: Uuid;
}

/** 跟 backend EvaluateAchievementsResponse. */
export interface EvaluateAchievementsResponse {
  newly_unlocked: string[];
  candidates_evaluated: number;
}

/** 跟 backend ApiError JSON envelope. */
export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}

// =====================================================================
// WebSocket 协议 (per docs/architecture/2026-09-03-arg/14-arg-10-arg-frontend.md §2)
// =====================================================================

/** WebSocket envelope (跟 backend sse_hub 协议对齐). */
export type ARGSseEvent =
  | { type: "edge_created"; edge: Edge }
  | { type: "edge_changed"; edge: Edge }
  | { type: "edge_archived"; id: Uuid; tenant_id: Uuid };

/** 6 事件类型 (per doc 14 §2.1). */
export const WS_EVENT_KINDS = [
  "arg_edge_changed",
  "arg_dispatch_route",
  "arg_context_inject",
  "arg_trust_score_update",
  "arg_achievement_unlocked",
] as const;
export type WSBridgeEventKind = (typeof WS_EVENT_KINDS)[number];

/** WS bridge event (前端订阅的 6 事件). */
export interface WSBridgeEvent {
  kind: WSBridgeEventKind;
  received_at: Iso8601;
  payload: unknown;
}
