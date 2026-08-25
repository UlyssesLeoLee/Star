# Implementation Plan: PLAN-017 — Development Execution Domain

> **RFC**: RFC-017
> **Domain Lead**: domain-development Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-017, RFC-016, RFC-021, RFC-022, RFC-023, RFC-027
> **相关 Module Spec**: domain-development-spec.md
> **相关 PoC**: POC-020, POC-021, POC-022, POC-029

---

## 目标(Goals)

1. DevelopmentExecution 作为独立聚合根(5 状态机:MVP)
2. 1 WorkItem → N DevelopmentExecution 关系(REQ-DEV-001)
3. Execution 聚合 Worktree / AgentSession / ChangeSet / Validation / Feedback / Commit / PR
4. AI Audit 通过 ExecutionId 统一索引(REQ-AUDIT-002)
5. 状态机分层:WorkItem 3 态 / Execution 5 态 / AgentSession 14 态

## 非目标(Non-Goals)

1. ❌ Graph Database 表达 Execution 关系(§30.6 Non-Goals)
2. ❌ Full Event Sourcing 重建 Execution 状态(§30.6)
3. ❌ Execution Template(V2 候选,§30.4)
4. ❌ 跨 Execution 自动 Commit 合并(MVP 人工触发)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-development Lead** | Execution 聚合根 / 状态机 / 事务边界 | ❌ |
| **domain-audit Lead** | AI Audit 索引 / TraceId 关联 | ❌ |
| **domain-worktree Lead** | Worktree 与 Execution 关联 | ❌ |
| **domain-agent Lead** | AgentSession 关联 Execution | ❌ |
| **SRE Lead** | Lifecycle Policy / 归档策略 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **DEV-001** | `development_executions` 表 Schema(15 字段 + 三级 tenant 隔离) | domain-development | RFC-017 | 250K | Migration 通过 |
| **DEV-002** | `work_item_id` 外键 + `WorkItem 1 → N Execution` 关系建模 | domain-development | DEV-001 | 200K | 无 N+1 查询 |
| **DEV-003** | Execution 5 状态机(CREATED / RUNNING / VALIDATING / COMPLETED / ABANDONED) | domain-development | DEV-001 | 350K | 单元测试覆盖率 100% |
| **DEV-004** | Execution 内部子对象外键统一(Worktree / AgentSession / ChangeSet / Validation / Feedback / Commit / PR) | domain-development | DEV-001 | 400K | 7 个子对象全部 `execution_id` 外键 |
| **DEV-005** | `ExecutionCommandPort` trait(create / start / transition / complete / abandon) | domain-development | DEV-003 | 350K | 5 个方法 + 错误类型 |
| **DEV-006** | `ExecutionQueryPort` trait(get / list_by_work_item / list_by_agent_session) | domain-development | DEV-003 | 300K | 3 个查询 + 索引 |
| **DEV-007** | Saga 拆分(Worktree 创建 / ChangeSet 提交 / Validation 触发分阶段) | domain-development | DEV-005 | 500K | Outbox 模式 + 最终一致 |
| **DEV-008** | HandoffContextPacket 跨 Execution 传递(§24.5) | domain-agent | DEV-007 | 400K | 复用已持久化 Context Packet |
| **DEV-009** | AI Audit ExecutionId 索引(REQ-AUDIT-002 关键问题) | domain-audit | DEV-001 | 350K | 5 问全可答:谁要 AI / Context / 修改 / Agent / Worktree / 时间 / 验证 / Feedback / 批准 |
| **DEV-010** | 跨 Execution 状态查询 API | domain-development | DEV-006 | 200K | P95 < 500ms(1000 Execution) |

**Phase 1 合计**:约 **3.3M tokens**

### Phase 2 (V1,Week 5-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **DEV-101** | Execution Lifecycle Policy(>90d 归档 + 冷热分层) | domain-development + SRE | DEV-001 | 250K | 归档脚本 + 监控 |
| **DEV-102** | 跨 Execution 状态对比 UI | domain-collaboration | DEV-006 | 300K | UI 支持 Execution 列表 + 对比 |
| **DEV-103** | Execution 性能分析(平均时长 / 重启率 / Handoff 频率) | domain-audit | DEV-009 | 350K | 性能仪表板 |
| **DEV-104** | Handoff 决策树(同 Execution vs 新 Execution,Open Question #2) | domain-agent + domain-development | DEV-008 | 250K | 决策规则落地 |
| **DEV-105** | 跨 Execution Commit 合并策略(Open Question #4) | domain-development + domain-scm | DEV-004 | 300K | 人工 / 自动合并规则 |

**Phase 2 合计**:约 **1.45M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **DEV-201** | Execution 性能分析 ML 化 | 600K |
| **DEV-202** | Execution Template 库 | 500K |
| **DEV-203** | 跨 Tenant Execution Federation(企业场景) | 800K |

**Phase 3 合计**:约 **1.9M tokens**

---

## 依赖矩阵

```
RFC-017 依赖:
  - RFC-016 (Worktree 聚合根)
  - RFC-021 (Agent Adapter)
  - RFC-022 (SCM Adapter)
  - RFC-027 (ChangeSet)

RFC-017 被依赖:
  - RFC-024 (Context Compiler 加载 Execution)
  - RFC-026 (AgentSession 关联 Execution)
  - RFC-029 (Conflict Detection 基于 Execution)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Execution 数量爆炸 | Medium | Lifecycle Policy + 状态机压缩(连续 FAILED 合并) |
| 跨 Execution 状态不一致 | High | Reconciliation 协议;Outbox 模式 |
| Execution 事务过大 | Medium | Saga 拆分 + Outbox 事件驱动 |
| 状态机分层复杂 | Low | 代码层独立状态机;API 严格校验 |

## 验收标准(MVP)

1. ✅ `development_executions` 表 15 字段 + 三级 tenant 隔离
2. ✅ 1 WorkItem → N Execution 关系(无 N+1)
3. ✅ 5 状态机完整 + 单元测试 100% 覆盖
4. ✅ 7 个子对象全部 `execution_id` 外键
5. ✅ AI Audit 5 问可答(REQ-AUDIT-002)
6. ✅ HandoffContextPacket 跨 Execution 复用 Context Packet
7. ✅ P95:WorkItem 反查 Execution < 500ms

## Token-OLU 总览

- **Phase 1(MVP)**:3.3M tokens ≈ 11-33 人·天
- **Phase 2(V1)**:1.45M tokens ≈ 5-15 人·天
- **Phase 3(V2)**:1.9M tokens
- **MVP + V1**:4.75M tokens(可由 domain-development Lead 1 人 12-16 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
