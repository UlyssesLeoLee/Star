# Implementation Plan: PLAN-024 — Context Compiler

> **RFC**: RFC-024
> **Domain Lead**: domain-context Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-024, RFC-023, RFC-025, RFC-027
> **相关 Module Spec**: domain-context-spec.md
> **相关 PoC**: POC-022, POC-023

---

## 目标(Goals)

1. Context Compiler 是确定性 / 半确定性系统(非 LLM)
2. 14 类源数据结构化输入(WorkItem / Acceptance / Worktree / Repository / Relevant Files / Symbols / ADR / Previous Decisions / Open Feedback / Failed Tests / Build Failure / Git Diff / PR Review / Agent Rules)
3. P0-P4 Priority Layer + Token Budget 约束
4. Provenance 追踪(每个 Context 片段可追溯)
5. Decision 独立管理
6. P5 Untrusted Repo Content 单独分类(§4.10.7 Prompt Injection 防护)

## 非目标(Non-Goals)

1. ❌ Semantic Context Selection(基于 Embedding,V2)
2. ❌ Predictive Context Preloading(V2)
3. ❌ Multi-Agent Context Sharing(V2)
4. ❌ LLM 自行选择 Context(违反 §26.1 硬约束)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-context Lead** | Context Compiler 核心算法 / Priority Layer | ❌ |
| **domain-agent Lead** | Context Packet 喂给 Agent | ❌ |
| **domain-feedback Lead** | Feedback 编译为 Agent Instruction | ❌ |
| **SRE Lead** | Token Budget 监控 / Relevant Context Ratio 监控 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-5)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **CTX-001** | Context Compiler 核心算法设计 | domain-context | RFC-024 | 500K | 确定性 / 半确定性,可单元测试 |
| **CTX-002** | 14 类源数据采集器(WorkItem / Acceptance / Worktree / Repository / Relevant Files / Symbols / ADR / Previous Decisions / Open Feedback / Failed Tests / Build Failure / Git Diff / PR Review / Agent Rules) | domain-context | CTX-001 | 1.0M | 14 类源数据接口完整 |
| **CTX-003** | P0-P4 Priority Layer 严格分桶 | domain-context | CTX-001 | 350K | 5 层结构 + 单元测试 |
| **CTX-004** | P0-P4 Token Budget 分配(P0:30% / P1:30% / P2:25% / P3:10% / P4:5%) | domain-context | CTX-003 | 300K | POC-023 校准 |
| **CTX-005** | P5 Untrusted 单独分类(绝不与 P0-P4 混合,§4.10.7) | domain-context | CTX-003 | 300K | Untrusted 隔离 100% |
| **CTX-006** | Token Budget 硬约束(超限时按优先级截断) | domain-context | CTX-004 | 250K | 超限处理逻辑 |
| **CTX-007** | Provenance 追踪(每个 Context 片段可追溯) | domain-context | CTX-002 | 350K | Provenance 完整 |
| **CTX-008** | Decision 独立管理(Active Decision 单独 Context) | domain-context | CTX-002 | 300K | Decision 提取准确 |
| **CTX-009** | HandoffContextPacket 生成 | domain-context + domain-agent | CTX-002 | 400K | §24.5 完整字段 |
| **CTX-010** | Decision 提取(从历史中 Active Decision) | domain-context | CTX-008 | 400K | MVP 人工标记 + 工具 |
| **CTX-011** | Unit Test Suite(确定性算法,Mock 数据集) | domain-context + SRE | CTX-001 | 500K | 单元测试覆盖率 > 90% |
| **CTX-012** | POC-022 验证(Given 1 WorkItem + 1 Worktree + 3 Feedback,生成 ContextPacket) | domain-context | CTX-001~009 | 300K | POC-022 通过 |
| **CTX-013** | POC-023 Token Budget 校准(30 WorkItem,Token 分布 P50/P95) | domain-context + SRE | CTX-004 | 350K | POC-023 校准表完成 |

**Phase 1 合计**:约 **5.3M tokens**

### Phase 2 (V1,Week 6-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **CTX-101** | Context Cost Analysis(§9 Token 分布监控) | domain-context + SRE | CTX-013 | 400K | 仪表板 + 告警 |
| **CTX-102** | Relevant Context Ratio 监控(RISK-025) | domain-context + SRE | CTX-007 | 350K | First-pass Acceptance Rate 监控 |
| **CTX-103** | Advanced Context Selection(ML 辅助,§30.3) | domain-context | CTX-001 | 600K | V1 中期评估 |
| **CTX-104** | Token Budget 持续校准(基于真实数据) | domain-context | CTX-013 | 300K | 季度校准 |
| **CTX-105** | P0 不可裁剪约束(§6.8 AI Content Retention) | domain-context + Compliance | CTX-005 | 250K | P0 永久保留 |

**Phase 2 合计**:约 **1.9M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **CTX-201** | Semantic Context Selection(基于 Embedding) | 1.2M |
| **CTX-202** | Predictive Context Preloading | 800K |
| **CTX-203** | Multi-Agent Context Sharing | 1.0M |

**Phase 3 合计**:约 **3.0M tokens**

---

## 依赖矩阵

```
RFC-024 依赖:
  - RFC-023 (Structured Feedback 输入)
  - RFC-025 (Context Packet 持久化)
  - RFC-027 (ChangeSet 引用 Context Packet)

RFC-024 被依赖:
  - RFC-021 (Agent 接收 Context)
  - RFC-026 (AgentSession 关联 ContextPacket)
  - RFC-017 (Execution 持有 ContextPacket)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Context Explosion | Medium | Token Budget + Priority Layer + Decision 优先;P95 监控 |
| Low-quality Context Selection | Medium | Relevant Context Ratio 监控;Provenance 强制;First-pass Acceptance Rate 监控 |
| Token Budget 校准偏差 | Medium | POC-023 真实数据校准;V1 持续监控;可调百分比 |
| Priority Layer 冲突 | Low | 明确规则(同片段跨优先级取最高);单元测试 |
| Decision 提取不准确 | Low | MVP 人工标记;V1 NLP 渐进;Fallback 人工标注 |

## 验收标准(MVP)

1. ✅ Context Compiler 确定性 / 半确定性,单元测试覆盖率 > 90%
2. ✅ 14 类源数据采集器
3. ✅ P0-P4 Priority Layer 严格分桶(P5 单独分类)
4. ✅ Token Budget 硬约束(超限时按优先级截断)
5. ✅ Provenance 追踪完整
6. ✅ Decision 独立管理
7. ✅ P5 Untrusted 100% 隔离
8. ✅ HandoffContextPacket 完整字段(§24.5)
9. ✅ POC-022 验证通过
10. ✅ POC-023 Token Budget 校准表完成

## Token-OLU 总览

- **Phase 1(MVP)**:5.3M tokens ≈ 18-53 人·天(Context Engineering 复杂)
- **Phase 2(V1)**:1.9M tokens
- **Phase 3(V2)**:3.0M tokens
- **MVP + V1**:7.2M tokens(可由 domain-context Lead 1 人 16-20 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
