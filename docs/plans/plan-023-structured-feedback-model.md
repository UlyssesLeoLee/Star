# Implementation Plan: PLAN-023 — Structured Feedback Model

> **RFC**: RFC-023
> **Domain Lead**: domain-feedback Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-023, RFC-024, RFC-025
> **相关 Module Spec**: domain-feedback-spec.md
> **相关 PoC**: POC-021

---

## 目标(Goals)

1. Feedback 作为独立聚合根(非 Comment 扩展)
2. `FeedbackTarget` 枚举 14 种目标粒度
3. `FeedbackType` 枚举 11 种类型
4. 结构化字段:`ExpectedBehavior / Preserve / Prohibit`
5. 6 状态机(OPEN / ACKNOWLEDGED / APPLIED / VERIFIED / REJECTED / SUPERSEDED)
6. Feedback Inbox / Intervention Queue
7. 缓解 RISK-026 Feedback Misinterpretation

## 非目标(Non-Goals)

1. ❌ Symbol-level Feedback 完整版(MVP 仅 File-level,V1 渐进)
2. ❌ AI 自生成 Feedback(V1 评估)
3. ❌ Feedback 全文搜索(Projection,V2 候选)
4. ❌ 跨 Workspace 共享 Feedback(MVP 限于 Project 内)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-feedback Lead** | Feedback 聚合根 / 状态机 / Inbox | ❌ |
| **domain-context Lead** | Feedback 编译为 Agent Instruction | ❌ |
| **domain-collaboration Lead** | UI 渐进式披露 / Inbox UI | ❌ |
| **domain-scm Lead** | PR Review Comment → Feedback 解析 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **FBK-001** | `feedbacks` 表 Schema(20 字段 + 三级 tenant 隔离) | domain-feedback | RFC-023 | 300K | Migration 通过 |
| **FBK-002** | `FeedbackTarget` 枚举 14 种(WorkItem / Requirement / AC / Worktree / AgentSession / File / Symbol(V1) / DiffHunk / Test / Build / RuntimeLog / ArchitectureDecision / PullRequest / ReviewFinding) | domain-feedback | FBK-001 | 400K | 14 种类型完整 |
| **FBK-003** | `FeedbackType` 枚举 11 种(Fix / Preserve / Refactor / Reject / Question / Constraint / Architecture / Security / Performance / Testing / Scope) | domain-feedback | FBK-001 | 250K | 11 种类型完整 |
| **FBK-004** | 结构化字段:`ExpectedBehavior / Preserve / Prohibit` | domain-feedback | FBK-001 | 250K | 3 个核心字段验证 |
| **FBK-005** | 6 状态机(OPEN / ACKNOWLEDGED / APPLIED / VERIFIED / REJECTED / SUPERSEDED) | domain-feedback | FBK-001 | 400K | 单元测试覆盖率 100% |
| **FBK-006** | `FeedbackConsumedEvent` Projection(消费追踪) | domain-feedback | FBK-001 | 250K | 记录 Agent 消费 |
| **FBK-007** | `FeedbackCommandPort` trait(create / acknowledge / mark_applied / verify / reject / supersede) | domain-feedback | FBK-005 | 350K | 6 个方法 + 错误类型 |
| **FBK-008** | `FeedbackQueryPort` trait(get / list_by_target / inbox) | domain-feedback | FBK-005 | 300K | 3 个查询 + 索引 |
| **FBK-009** | Feedback Inbox 聚合查询 | domain-feedback | FBK-008 | 350K | UI Inbox P95 < 500ms |
| **FBK-010** | Intervention Queue 聚合查询 | domain-feedback | FBK-008 | 300K | UI Queue P95 < 500ms |
| **FBK-011** | UI 渐进式披露(基础字段 → 高级字段) | domain-collaboration | FBK-004 | 400K | Feedback 模板 + 渐进披露 |
| **FBK-012** | Context Compiler 集成(Feedback 编译为 Agent Instruction) | domain-context | FBK-002 | 450K | POC-021 验证 Token 下降 50% |
| **FBK-013** | HandoffContextPacket 包含 Open Feedback 列表(§24.5) | domain-context + domain-agent | FBK-008 | 300K | Handoff 时 Open Feedback 完整传递 |

**Phase 1 合计**:约 **4.3M tokens**

### Phase 2 (V1,Week 5-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **FBK-101** | Symbol-level Feedback 完整版(依赖 POC-025) | domain-feedback + domain-context | FBK-002 | 500K | Symbol 识别准确率 > 95% |
| **FBK-102** | PR Review Feedback Import 完整版 | domain-feedback + domain-scm | FBK-012 | 500K | 解析率 > 80% |
| **FBK-103** | AI 自生成 Feedback(AI 自己的修正建议) | domain-context | FBK-001 | 600K | 走相同 Feedback 聚合根 |
| **FBK-104** | Consumed Event 投影 Lifecycle(>30d 归档) | SRE | FBK-006 | 250K | 归档脚本 + 监控 |
| **FBK-105** | Feedback 性能分析仪表板(Reopen Rate / Repetition) | domain-feedback + domain-audit | FBK-009 | 350K | RISK-026 监控指标 |
| **FBK-106** | Feedback 模板库(常见 Feedback Pattern) | domain-feedback | FBK-011 | 300K | 模板可复用 |

**Phase 2 合计**:约 **2.5M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **FBK-201** | Feedback 全文搜索(Projection) | 600K |
| **FBK-202** | Multi-Agent Feedback 协调 | 800K |
| **FBK-203** | Feedback 性能分析 ML 化 | 600K |

**Phase 3 合计**:约 **2.0M tokens**

---

## 依赖矩阵

```
RFC-023 依赖:
  - 无(独立聚合根)

RFC-023 被依赖:
  - RFC-024 (Context Compiler 编译 Feedback)
  - RFC-025 (Feedback 在 Context Packet 中)
  - RFC-021 (Agent 消费 Feedback)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Feedback Misinterpretation | Medium | 结构化字段(Expected/Preserve/Prohibit);状态机;Reopen Rate 监控 |
| UI 复杂度上升 | Medium | 渐进式披露;Feedback 模板 |
| Symbol-level 反馈推迟 | Low | MVP 仅 File-level,V1 渐进(§30.3) |
| Consumed Event 膨胀 | Low | Lifecycle Policy >30d 归档;聚合压缩 |
| Inbox 性能 | Low | 索引优化;分页 |

## 验收标准(MVP)

1. ✅ `feedbacks` 表独立(非 Comment 扩展)
2. ✅ 14 种 `FeedbackTarget` 枚举
3. ✅ 11 种 `FeedbackType` 枚举
4. ✅ `Severity` P0-P3 四级
5. ✅ 结构化字段完整(Expected/Preserve/Prohibit)
6. ✅ 6 状态机完整 + 单元测试 100%
7. ✅ `FeedbackConsumedEvent` Projection
8. ✅ Feedback Inbox / Intervention Queue
9. ✅ POC-021 验证 Token 下降 50%
10. ✅ HandoffContextPacket 包含 Open Feedback 列表

## Token-OLU 总览

- **Phase 1(MVP)**:4.3M tokens ≈ 14-43 人·天
- **Phase 2(V1)**:2.5M tokens
- **Phase 3(V2)**:2.0M tokens
- **MVP + V1**:6.8M tokens(可由 domain-feedback Lead 1 人 14-18 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
