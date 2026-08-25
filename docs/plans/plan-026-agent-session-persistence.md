# Implementation Plan: PLAN-026 — Agent Session Persistence

> **RFC**: RFC-026
> **Domain Lead**: domain-agent Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-026, RFC-021, RFC-017, RFC-025
> **相关 Module Spec**: domain-agent-spec.md
> **相关 PoC**: POC-020

---

## 目标(Goals)

1. AgentSession 持久化(元数据 + 关键字段入 PostgreSQL,全文 Transcript 走 Object Storage)
2. 14 状态机完整(《Basic Design》§4.2.3,锁定状态数)
3. Plan / Decisions / ChangeSet / ValidationResult / FeedbackConsumed / TraceReference 关联
4. AI Audit 通过 AgentSessionId + ExecutionId + TraceId 联合索引(REQ-AUDIT-002)
5. 符合 §6.8 AI Content Retention Policy
6. 缓解 RISK-023 Agent Session State Divergence

## 非目标(Non-Goals)

1. ❌ Transcript 全文搜索(Projection,V2 候选)
2. ❌ AI 异常行为检测(V2)
3. ❌ 跨 Tenant AgentSession Federation(V2)
4. ❌ Agent Session Template(V2 候选)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-agent Lead** | AgentSession 聚合根 / 14 状态机 / 关联 | ❌ |
| **SRE Lead** | Object Storage 部署 / 备份 / Lifecycle Policy | ❌ |
| **domain-audit Lead** | AI Audit 索引 / TraceId 关联 | ❌ |
| **Compliance Lead** | §6.8 AI Content Retention 合规审查 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **AGS-001** | `agent_sessions` 表 Schema(20 字段 + 三级 tenant 隔离) | domain-agent | RFC-026 | 300K | Migration 通过 |
| **AGS-002** | 14 状态机实现(CREATED / STARTING / RUNNING / WAITING_TOOL / TOOL_RUNNING / TOOL_COMPLETED / WAITING_FEEDBACK / FEEDBACK_RECEIVED / VALIDATING / COMPLETED / FAILED / ABORTED / CRASHED / TIMEOUT) | domain-agent | AGS-001 | 500K | 单元测试覆盖率 100% |
| **AGS-003** | 状态变更触发者(代码层独立,非 Agent 自报) | domain-agent | AGS-002 | 350K | CRASHED 由 Local Runtime 上报 |
| **AGS-004** | Plan / Decisions / ChangeSet / ValidationResult / FeedbackConsumed / TraceReference 关联 | domain-agent | AGS-001 | 450K | 6 个关联完整 |
| **AGS-005** | 全文 Transcript 走 Object Storage(类似 RFC-025) | domain-agent + SRE | AGS-001 | 400K | Object Storage 多副本 |
| **AGS-006** | 状态变更走 Outbox 模式(保证事件最终一致) | domain-agent | AGS-002 | 350K | Outbox 实施 |
| **AGS-007** | `AgentSessionCommandPort`(create / start / transition / complete / abort / crash) | domain-agent | AGS-002 | 400K | 6 个方法 + 错误类型 |
| **AGS-008** | `AgentSessionQueryPort`(get / list_by_worktree / list_by_execution) | domain-agent | AGS-002 | 300K | 3 个查询 + 索引 |
| **AGS-009** | AI Audit 索引(AgentSessionId + ExecutionId + TraceId 联合索引) | domain-audit | AGS-001 | 350K | REQ-AUDIT-002 完全覆盖 |
| **AGS-010** | §6.8 AI Content Retention 实施(P0 不可裁剪) | domain-agent + Compliance | AGS-005 | 350K | 合规审查通过 |
| **AGS-011** | Lifecycle Policy(>90d 归档,§5.8) | SRE | AGS-001 | 300K | 归档脚本 + 监控 |
| **AGS-012** | POC-020 验证(状态机完整迁移 + 事件全部触发) | domain-agent | AGS-001~008 | 300K | POC-020 通过 |

**Phase 1 合计**:约 **4.35M tokens**

### Phase 2 (V1,Week 5-8)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **AGS-101** | Transcript 压缩 + 分片存储 | SRE | AGS-005 | 400K | 存储成本下降 30% |
| **AGS-102** | HandoffContextPacket 跨 Session 传递(§24.5) | domain-agent | AGS-007 | 500K | Handoff 时 Session 状态正确迁移 |
| **AGS-103** | AgentSession 性能分析仪表板 | domain-audit | AGS-009 | 350K | 平均时长 / 重启率 / Handoff 频率 |
| **AGS-104** | Transcript 类型化拆分(P0 不可裁剪 vs 普通) | Compliance | AGS-010 | 250K | 分级存储 |
| **AGS-105** | 状态变更持久化 Throttle(每 1s 批量) | domain-agent | AGS-006 | 300K | 写放大控制 |

**Phase 2 合计**:约 **1.8M tokens**

### Phase 3 (V2,Week 9+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **AGS-201** | Transcript 全文搜索(Projection) | 800K |
| **AGS-202** | AI 异常行为检测 | 1.0M |
| **AGS-203** | AgentSession 性能分析 ML 化 | 600K |

**Phase 3 合计**:约 **2.4M tokens**

---

## 依赖矩阵

```
RFC-026 依赖:
  - RFC-021 (Agent Adapter)
  - RFC-017 (Development Execution)
  - RFC-025 (类似持久化模式)

RFC-026 被依赖:
  - RFC-024 (Context Compiler 关联 AgentSession)
  - RFC-030 (Agent Policy 在 AgentSession 实施)
  - RFC-016 (Worktree 持有 AgentSession)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Agent Session State Divergence | Medium | 持久化 + Reconciliation(§4.2);Local Runtime 上报机制(§4.6.5) |
| Transcript 存储增长 | Medium | Lifecycle Policy >90d 归档;Transcript 压缩;分片存储 |
| AI Content Retention 违规 | High | §6.8 Retention Policy;P0 不可裁剪;Compliance 审查 |
| Object Storage 故障 | High | 多副本;定期备份;故障转移 |
| 写入开销影响性能 | Low | 异步批量持久化;Throttle(每 1s 批量) |

## 验收标准(MVP)

1. ✅ `agent_sessions` 表 20 字段 + 三级 tenant 隔离
2. ✅ 14 状态机完整(锁定状态数,单元测试 100%)
3. ✅ 6 个关联完整(Plan / Decisions / ChangeSet / ValidationResult / FeedbackConsumed / TraceReference)
4. ✅ 全文 Transcript 走 Object Storage
5. ✅ 状态变更走 Outbox 模式
6. ✅ AI Audit 索引(AgentSessionId + ExecutionId + TraceId 联合)
7. ✅ §6.8 AI Content Retention 实施,P0 不可裁剪
8. ✅ Lifecycle Policy >90d 归档
9. ✅ POC-020 验证状态机完整迁移 + 事件全部触发
10. ✅ Agent Handoff 时 Session 状态正确迁移

## Token-OLU 总览

- **Phase 1(MVP)**:4.35M tokens ≈ 14-44 人·天
- **Phase 2(V1)**:1.8M tokens
- **Phase 3(V2)**:2.4M tokens
- **MVP + V1**:6.15M tokens(可由 domain-agent Lead 1 人 14-18 周完成,Object Storage 部署需 SRE 配合)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
