// frontend/src/mocks/__tests__/uat/uat-test-data.ts
// UAT (User Acceptance Testing) 跨域场景测试数据
//
// 跨域场景 fixture (per docs/test-design.md §9 测试数据管理 + §16.4 5 域业务 mock):
//   - UAT_BUSINESS_FLOW: Scenario 1 业务流配置 (Tenant → Workspace)
//   - UAT_FB_AGENT_SESSION: Scenario 2 AI agent session (per INV-FB-07)
//   - UAT_WT_PROVISION: Scenario 3 Worktree provision
//   - UAT_BILLING_CYCLE: Scenario 4 billing cycle
//
// 数据生成: 复用现有 mocks/data/five-domain.ts + mocks/data/agents.ts, 不新建
// mock data, 避免维护成本. UAT fixture 只引用现有 mock data + 跨域关联 ID.

/** Scenario 1: Tenant → Workspace → Project 业务流配置 */
export const UAT_BUSINESS_FLOW = {
  tenant_id: "t-acme", // 对应 MOCK_TENANTS[0] (plan=enterprise)
  workspace_id: "ws-physis", // 对应 MOCK_WORKSPACES[0]
  project_id: "pr-uat-cross", // 跨域项目 ID
  // 跨域场景: workspace + project + worktree 同 tenant
  cross_domain_check: {
    // RBAC 检查
    expected_rbac_roles: ["admin", "member", "viewer"],
  },
} as const;

/** Scenario 2: AI agent session (per domain-feedback INV-FB-07) */
export const UAT_FB_AGENT_SESSION = {
  tenant_id: "t-acme",
  project_id: "pr-uat-cross",
  work_item_id: "wi-uat-fb-001",
  agent_id: "ag-001", // 对应 MOCK_AGENTS[0]
  // INV-FB-07 验证: AI 提 Feedback 必带 author_agent_id, 不带 author_user_id
  feedback_payload: {
    severity: "P2",
    intent: "UAT: AI agent 跨域 Feedback 提交通道",
  },
  // 配套: human user 测试 (INV-FB-07 互补: human 必带 user, 不带 agent)
  human_user_id: "usr-uat-human-001",
} as const;

/** Scenario 3: Worktree provision (5 域 worktree 域) */
export const UAT_WT_PROVISION = {
  tenant_id: "t-acme",
  project_id: "pr-uat-cross",
  worktree_id: "wt-uat-provision-001",
  storage_required_mb: 512,
  // 状态流转 (per SUPPORTED_TRANSITION_STATES)
  initial_state: "CREATING",
  final_state: "ACTIVE",
} as const;

/** Scenario 4: Billing cycle (5 域 economy 域) */
export const UAT_BILLING_CYCLE = {
  tenant_id: "t-acme",
  // 跨月 billing (per MOCK_BILLING 4 rows 跨 3 月)
  period: "2026-08",
  // plan=enterprise 配额 (cross-domain check)
  quota_mb: 10240,
  used_mb: 4096,
} as const;
