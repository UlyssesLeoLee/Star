# Implementation Plan: PLAN-030 — Agent Policy Enforcement

> **RFC**: RFC-030
> **Domain Lead**: domain-permission Lead(主) + domain-agent Lead(协同)
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-030, RFC-018, RFC-019, RFC-021
> **相关 Module Spec**: domain-agent-spec.md
> **相关 PoC**: POC-029

---

## 目标(Goals)

1. Policy 由 Application / Authorization 层强制(非仅 Prompt)
2. 12 个强制点全覆盖:Repository / Worktree / Path / Tool / Network / Secret / Runtime / Context / Change Scope / Review / Test / Approval
3. AgentPolicy 值对象完整(15+ 字段)
4. Project Policy 模板库(V1)
5. Policy Violation 事件记录到 Audit
6. 缓解 RISK-017 Agent Escapes Worktree Scope
7. 符合 REQ-PERM-002 硬约束

## 非目标(Non-Goals)

1. ❌ 仅靠 Prompt 约束(违反 REQ-PERM-002)
2. ❌ Policy 误配置容忍(MVP 严格模式)
3. ❌ AI 辅助 Policy 推荐(V2 候选)
4. ❌ 跨 Tenant Policy Federation(V2)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-permission Lead** | AgentPolicy 值对象 / 12 强制点 / Project Policy 模板 | ❌(主) |
| **domain-agent Lead** | Agent Adapter 集成 Policy 强制 | ❌(协同) |
| **SRE Lead** | Policy 检查性能监控 | ❌ |
| **domain-audit Lead** | Policy Violation 事件记录 | ❌ |

> **关键约束**:domain-permission Lead 不兼任 domain-agent Lead,确保 Policy 与 Agent 实现解耦,避免 Agent 实现者"自己约束自己"

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **POL-001** | `AgentPolicy` 值对象(15+ 字段:allowed_repositories / allowed_worktrees / allowed_paths / forbidden_paths / allowed_tools / allowed_command_categories / network_access / secret_access / max_runtime_seconds / max_context_tokens / max_change_files / max_change_lines / require_review / require_test / require_approval) | domain-permission | RFC-030 | 500K | 15 字段完整 |
| **POL-002** | Policy 配置 Schema(Tenant / Project / User 三级) | domain-permission | POL-001 | 350K | 层级覆盖 |
| **POL-003** | Policy 检查引擎(Application / Authorization 层) | domain-permission | POL-001 | 600K | 12 强制点全部生效 |
| **POL-004** | Repository 范围检查(Policy.allowed_repositories) | domain-permission | POL-003 | 200K | 越权 Repository 拒绝 |
| **POL-005** | Worktree 范围检查(Local Runtime Command Scope) | domain-permission + domain-local-runtime | POL-003 | 250K | 越权 Worktree 拒绝 |
| **POL-006** | Path 范围检查(Local Runtime Filesystem Scope) | domain-permission + domain-local-runtime | POL-003 | 250K | 越权 Path 拒绝 |
| **POL-007** | Tool 范围检查(Agent Adapter 解析 Tool Call) | domain-agent + domain-permission | POL-003 | 300K | 越权 Tool 拒绝 |
| **POL-008** | Network 检查(Local Runtime Egress Proxy) | domain-permission + domain-local-runtime | POL-003 | 300K | 越权 Network 拒绝 |
| **POL-009** | Secret 检查(Credential Broker) | domain-permission + domain-local-runtime | POL-003 | 300K | Agent 不直接持有 Secret |
| **POL-010** | Runtime Limit(Application 启动 + Worker 监控) | domain-permission | POL-003 | 250K | 超时强制停止 |
| **POL-011** | Context Limit(Context Compiler 强制) | domain-permission + domain-context | POL-003 | 250K | Token 超限截断 |
| **POL-012** | Change Scope(Local Runtime fs watcher + commit gate) | domain-permission + domain-local-runtime | POL-003 | 300K | 超 max_change_files 拒绝 |
| **POL-013** | Review / Test / Approval Gate(application 提交前) | domain-permission | POL-003 | 350K | require_* 强制 |
| **POL-014** | Policy Violation 事件记录到 Audit | domain-audit + domain-permission | POL-003 | 300K | 完整可追溯 |
| **POL-015** | Policy Dry-run 模式(配置验证不立即生效) | domain-permission | POL-001 | 250K | 配置变更可验证 |
| **POL-016** | POC-029 验证(12 强制点全部生效) | domain-permission + domain-agent | POL-001~015 | 400K | 越权 Path / Tool / Network / Secret 全部拦截 |

**Phase 1 合计**:约 **5.15M tokens**

### Phase 2 (V1,Week 5-8)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **POL-101** | Agent Policy Templates 库(Project / Tenant) | domain-permission | POL-002 | 500K | 模板可复用 |
| **POL-102** | Tenant Policy 继承(子 Project 继承 + override) | domain-permission | POL-002 | 400K | 继承规则 |
| **POL-103** | Policy 性能优化(静态规则预编译,检查 P95 < 50ms) | domain-permission + SRE | POL-003 | 400K | 性能达标 |
| **POL-104** | Policy 性能分析仪表板(检查 QPS / 拦截率) | domain-permission + domain-audit | POL-014 | 300K | 仪表板 |
| **POL-105** | RISK-017 监控指标(Agent Policy Violation 次数) | SRE + domain-audit | POL-014 | 250K | 监控告警 |

**Phase 2 合计**:约 **1.85M tokens**

### Phase 3 (V2,Week 9+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **POL-201** | AI 辅助 Policy 推荐 | 1.0M |
| **POL-202** | Policy 异常行为检测 | 800K |
| **POL-203** | Cross-Agent Policy Sharing | 600K |

**Phase 3 合计**:约 **2.4M tokens**

---

## 依赖矩阵

```
RFC-030 依赖:
  - RFC-018 (Local Runtime 架构)
  - RFC-019 (Security Model)
  - RFC-021 (Agent Adapter Model)
  - RFC-016 (Worktree First-class)

RFC-030 被依赖:
  - RFC-026 (AgentSession 应用 Policy)
  - RFC-027 (ChangeScope 在 ChangeSet 实施)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Policy 误配置 | High | Policy 模板;Dry-run 模式;变更审计;POC-029 验证 |
| Policy 检查性能开销 | Low | 静态规则预编译;批量检查;按需启用 |
| Policy 模板不灵活 | Medium | Project Policy 自定义;Tenant Policy 继承;UI 模板编辑器 |
| Agent 绕过 Application 层 | Low | Application 层是必经路径;Audit 强制 |
| Policy Violation 处理复杂 | Medium | 明确分类(Warning / Block / Audit);Agent 反馈机制 |

## 验收标准(MVP)

1. ✅ 12 强制点全部实现
2. ✅ AgentPolicy 15 字段完整
3. ✅ Policy 由 Application / Authorization 层强制
4. ✅ Policy 误配置 Dry-run 模式
5. ✅ Policy Violation 事件记录到 Audit
6. ✅ POC-029 验证 12 强制点全部生效
7. ✅ Agent Policy Templates 库(V1)
8. ✅ Tenant Policy 继承(V1)
9. ✅ Policy 检查 P95 < 50ms
10. ✅ RISK-017 监控指标

## Token-OLU 总览

- **Phase 1(MVP)**:5.15M tokens ≈ 17-52 人·天(12 强制点全覆盖)
- **Phase 2(V1)**:1.85M tokens
- **Phase 3(V2)**:2.4M tokens
- **MVP + V1**:7.0M tokens(由 domain-permission Lead + domain-agent Lead 2 人 16-20 周完成,**不兼任**符合用户偏好)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
