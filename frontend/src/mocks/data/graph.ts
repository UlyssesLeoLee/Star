// =====================================================================
// Mock Graph Fixtures (per ADR-0041-arch-agent-graph-viewer §2.3.5)
// =====================================================================
// 用途: 1 个 work item 的 1-hop 投影 + 2-hop 代码侧节点, 给 MSW handler 用
//
// 数据形状 (per types/graph.ts):
//   - 1 个 work_item (current)
//   - 1-hop 11 节点: worktree / agent_session / change_set / scm_repository
//                    / pull_request / identity x2 (assignee + reporter)
//                    / validation_case x2 / comment / feedback / project
//   - 1-hop 13 边 (typed edge label)
//   - 2-hop 代码侧 4 节点: cratemodule x2 + symbol x2
//   - 2-hop 3 边: LIVES_IN x2 + DEPENDS_ON x1
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 全 1 fixture 写死, 不用 mock-msw-handlers.md §2.6 模板 (避免引入 ctx context)
//   - work_item id 跟 mocks/data/kanban.ts 真实 work item 错开 (用 wi-arch-001)
//     避免污染现有 Kanban 数据 + 便于单测 pinpoint
//   - 节点 id 格式 "KIND_PREFIX:uuid" 跟 types/graph.ts 一致
// =====================================================================

import type { GraphPayload, GraphNode, GraphEdge } from "@/types/graph";

const WI_ID = "wi-arch-001";
const TENANT_ID = "tenant-physis-corp";
const PROJECT_ID = "proj-physis";
const NOW = "2026-09-02T01:00:00Z";

/** 1 个 work_item 的 1-hop 投影 + 2-hop 代码侧 */
export const MOCK_GRAPH_PHYSIS_123: GraphPayload = {
  work_item_id: WI_ID,
  tenant_id: TENANT_ID,
  fingerprint: "fp-physis-123-deadbeef-2026-09-02",
  nodes: [
    // ==================== current work_item (主节点) ====================
    {
      id: "WI:wi-arch-001",
      kind: "work_item",
      label: "PHYSIS-123: Add radial impulse to engine",
      is_current: true,
      hop_level: 1,
      properties: {
        key: "PHYSIS-123",
        status: "in_progress",
        priority: "p1",
        story_points: 5,
        assignee_id: "usr-002",
        labels: ["physics", "rigid-body"],
      },
    },

    // ==================== 1-hop: project / identity / worktree / agent ====================
    {
      id: "P:proj-physis",
      kind: "project",
      label: "PHYSIS (Rust Physics Engine)",
      is_current: false,
      hop_level: 1,
      properties: { key: "PHYSIS", visibility: "internal" },
    },
    {
      id: "ID:usr-002",
      kind: "identity",
      label: "Ulysses (架构师)",
      is_current: false,
      hop_level: 1,
      properties: { email: "u@physis.local", provider: "github" },
    },
    {
      id: "ID:usr-007",
      kind: "identity",
      label: "Mavis (agent)",
      is_current: false,
      hop_level: 1,
      properties: { email: "mavis@star.local", provider: "local-runtime-device" },
    },
    {
      id: "WT:wt-physics-radial-impulse",
      kind: "worktree",
      label: "wt-physics-radial-impulse (main)",
      is_current: false,
      hop_level: 1,
      properties: { branch: "feat/physics-radial-impulse", status: "active" },
    },
    {
      id: "AGT:claude-sonnet-42",
      kind: "agent_session",
      label: "claude-sonnet-42 (running)",
      is_current: false,
      hop_level: 1,
      properties: { agent_kind: "claude-sonnet", status: "executing", token_usage: 82000 },
    },

    // ==================== 1-hop: code change artifacts ====================
    {
      id: "CS:cs-physics-radial-001",
      kind: "change_set",
      label: "cs-physics-radial-001 (+142 -23 files:4)",
      is_current: false,
      hop_level: 1,
      properties: { status: "applied", symbol_index: { added: 12, modified: 5, removed: 0 } },
    },
    {
      id: "REPO:physis-mono",
      kind: "scm_repository",
      label: "physis-mono (github)",
      is_current: false,
      hop_level: 1,
      properties: { provider: "github", default_branch: "main" },
    },
    {
      id: "PR:physis-456",
      kind: "pull_request",
      label: "PR #456: feat/radial-impulse",
      is_current: false,
      hop_level: 1,
      properties: { status: "review_required", source_branch: "feat/physics-radial-impulse" },
    },

    // ==================== 1-hop: validation / feedback / comment ====================
    {
      id: "VC:vc-rigid-body-rotation",
      kind: "validation_case",
      label: "VC: rigid-body rotation impulse (unit)",
      is_current: false,
      hop_level: 1,
      properties: { kind: "unit", result: "passed", coverage: 0.94 },
    },
    {
      id: "VC:vc-3body-stability",
      kind: "validation_case",
      label: "VC: 3-body stability (integration)",
      is_current: false,
      hop_level: 1,
      properties: { kind: "integration", result: "running", coverage: 0.71 },
    },
    {
      id: "FB:fb-12",
      kind: "feedback",
      label: "FB: Spec clarification needed (open)",
      is_current: false,
      hop_level: 1,
      properties: { status: "open", severity: "minor", category: "spec_clarification" },
    },
    {
      id: "CM:cm-3",
      kind: "comment",
      label: "CM: 'should it support continuous impulses?'",
      is_current: false,
      hop_level: 1,
      properties: { author_id: "usr-002", target_kind: "work_item" },
    },
  ],

  edges: [
    // ==================== 1-hop edges (current) ====================
    { id: "e1",  kind: "IN_PROJECT",   source: "WI:wi-arch-001", target: "P:proj-physis", hop_level: 1 },
    { id: "e2",  kind: "ASSIGNED_TO",  source: "WI:wi-arch-001", target: "ID:usr-002",     hop_level: 1 },
    { id: "e3",  kind: "REPORTED_BY",  source: "WI:wi-arch-001", target: "ID:usr-007",     hop_level: 1 },
    { id: "e4",  kind: "ON_WORKTREE",  source: "WI:wi-arch-001", target: "WT:wt-physics-radial-impulse", hop_level: 1 },
    { id: "e5",  kind: "PRODUCED",     source: "WI:wi-arch-001", target: "CS:cs-physics-radial-001", hop_level: 1 },
    { id: "e6",  kind: "HAS_PR",       source: "WI:wi-arch-001", target: "PR:physis-456",  hop_level: 1 },
    { id: "e7",  kind: "VALIDATED_BY", source: "WI:wi-arch-001", target: "VC:vc-rigid-body-rotation", hop_level: 1 },
    { id: "e8",  kind: "VALIDATED_BY", source: "WI:wi-arch-001", target: "VC:vc-3body-stability", hop_level: 1 },
    { id: "e9",  kind: "HAS_FEEDBACK", source: "WI:wi-arch-001", target: "FB:fb-12",       hop_level: 1 },
    { id: "e10", kind: "COMMENTED_ON", source: "CM:cm-3",        target: "WI:wi-arch-001", hop_level: 1 },
    // ==================== 1-hop edges (transitive) ====================
    { id: "e11", kind: "POWERS",       source: "WT:wt-physics-radial-impulse", target: "AGT:claude-sonnet-42", hop_level: 1 },
    { id: "e12", kind: "TARGETS_BRANCH", source: "PR:physis-456", target: "WT:wt-physics-radial-impulse", hop_level: 1 },
    { id: "e13", kind: "WEBHOOK_FOR",  source: "REPO:physis-mono", target: "REPO:physis-mono", hop_level: 1, properties: { dummy: true } },
  ],

  stats: {
    node_count: 13,
    edge_count: 13,
    kind_breakdown: {
      work_item: 1, project: 1, identity: 2, worktree: 1, agent_session: 1,
      change_set: 1, scm_repository: 1, pull_request: 1,
      validation_case: 2, feedback: 1, comment: 1,
    },
  },
  generated_at: NOW,
};

/** 包含 2-hop 代码侧节点的扩展版 (per ADR §2.1, 用于 max_hop=2) */
export const MOCK_GRAPH_PHYSIS_123_2HOP: GraphPayload = {
  ...MOCK_GRAPH_PHYSIS_123,
  fingerprint: "fp-physis-123-extended-2026-09-02",
  nodes: [
    ...MOCK_GRAPH_PHYSIS_123.nodes,
    // 2-hop: agent_session 的 REFERENCES + cratemodule
    {
      id: "MOD:domain-physics-core",
      kind: "cratemodule",
      label: "crates/domain-physics-core",
      is_current: false,
      hop_level: 2,
      properties: { crate: "domain-physics-core", kind: "lib" },
    },
    {
      id: "MOD:domain-physics-rigid",
      kind: "cratemodule",
      label: "crates/domain-physics-rigid-body",
      is_current: false,
      hop_level: 2,
      properties: { crate: "domain-physics-rigid-body", kind: "lib" },
    },
    {
      id: "SYM:RigidBody::apply_radial_impulse",
      kind: "symbol",
      label: "RigidBody::apply_radial_impulse()",
      is_current: false,
      hop_level: 2,
      properties: { file: "src/rigid_body.rs", line: 142, kind: "fn" },
    },
    {
      id: "SYM:PhysicsCore::step",
      kind: "symbol",
      label: "PhysicsCore::step()",
      is_current: false,
      hop_level: 2,
      properties: { file: "src/core.rs", line: 89, kind: "fn" },
    },
  ],
  edges: [
    ...MOCK_GRAPH_PHYSIS_123.edges,
    // 2-hop edges (code-side)
    { id: "e14", kind: "REFERENCES", source: "CS:cs-physics-radial-001", target: "SYM:RigidBody::apply_radial_impulse", hop_level: 2 },
    { id: "e15", kind: "LIVES_IN",    source: "SYM:RigidBody::apply_radial_impulse", target: "MOD:domain-physics-rigid", hop_level: 2 },
    { id: "e16", kind: "LIVES_IN",    source: "SYM:PhysicsCore::step", target: "MOD:domain-physics-core", hop_level: 2 },
    { id: "e17", kind: "DEPENDS_ON",  source: "MOD:domain-physics-rigid", target: "MOD:domain-physics-core", hop_level: 2 },
  ],
  stats: {
    node_count: 17,
    edge_count: 17,
    kind_breakdown: {
      ...MOCK_GRAPH_PHYSIS_123.stats.kind_breakdown,
      cratemodule: 2, symbol: 2,
    },
  },
  generated_at: NOW,
};

/** 简化版 fallback (work_item 不存在时, 返空图) */
export const MOCK_GRAPH_EMPTY: GraphPayload = {
  work_item_id: "wi-empty",
  tenant_id: TENANT_ID,
  fingerprint: "fp-empty",
  nodes: [],
  edges: [],
  stats: { node_count: 0, edge_count: 0, kind_breakdown: {} },
  generated_at: NOW,
};
