# Implementation Plan: PLAN-016 — Worktree as First-class Domain Entity

> **RFC**: RFC-016
> **Domain Lead**: domain-worktree Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-016, RFC-017, RFC-020, RFC-029
> **相关 Module Spec**: domain-worktree-spec.md
> **相关 PoC**: POC-017, POC-018, POC-019, POC-024

---

## 目标(Goals)

1. Worktree 作为独立聚合根实现,17 个状态机完整(《Basic Design》§4.1.3)
2. 1 WorkItem → N Worktree 关系建模(REQ-DEV-001)
3. Worktree Status 与 WorkItem Status 完全解耦(REQ-WF-002)
4. Observed State 与 Business State 分离存储(REQ-DATA-003)
5. 跨 Worktree 聚合查询(Conflict / Heatmap)以 Repository 为入口

## 非目标(Non-Goals)

1. ❌ Symbol-level Worktree(推迟到 V1,RFC-028)
2. ❌ Cross-Worktree Dependency Graph(§30.4 V2)
3. ❌ Autonomous Worktree 创建(AI 自行决定 Worktree 数量)
4. ❌ Worktree 跨 Project 共享(MVP 仅单 Project 范围)

---

## Owner 矩阵(每域独立 Lead,符合用户 5 域架构原则)

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-worktree Lead** | Worktree 聚合根 / 状态机 / 索引 | ❌ 不兼任其他域 |
| **domain-development Lead** | DevelopmentExecution 聚合层(配套) | ❌ 不兼任 |
| **domain-local-runtime Lead** | Local Runtime 端 Observed State 上报 | ❌ 不兼任 |
| **domain-collaboration Lead** | Heatmap 投影 / UI 实时性 | ❌ 不兼任 |
| **SRE Lead** | 数据库索引 / 性能监控 / 归档策略 | ❌ 不兼任 |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **WT-001** | `worktrees` 表 Schema 落地(22 字段 + 三级 tenant 隔离) | domain-worktree | RFC-016 | 200K | Migration 通过;Schema 与 data-design §6 一致 |
| **WT-002** | `development_executions.worktree_id` 外键 + 中间表设计 | domain-worktree | WT-001 | 150K | Worktree 通过 DevelopmentExecution 关联 WorkItem(无直接外键) |
| **WT-003** | Worktree 17 状态机实现 + 转换规则校验 | domain-worktree | WT-001 | 400K | 单元测试覆盖所有合法/非法转换;状态机覆盖率 100% |
| **WT-004** | `WorktreeCommandPort` trait + 6 个方法(create / assign / record_observed / transition / abandon / archive) | domain-worktree | WT-003 | 350K | 6 个方法签名 + 错误类型 + 单元测试 |
| **WT-005** | `WorktreeQueryPort` trait + 5 个查询(get / list_by_work_item / list_by_agent / detect_conflicts / heatmap) | domain-worktree | WT-003 | 300K | 5 个查询方法 + 索引优化 |
| **WT-006** | `worktree_status_observed` Projection 表 + 1s 批量上报接口 | domain-worktree | RFC-020 | 250K | Projection 表与主表分离;1s 批量 Throttle |
| **WT-007** | Heatmap 投影 Phase 1(File-level,100 Worktree / 10k File < 500ms) | domain-worktree | WT-005 | 400K | POC-024 验证通过;性能基准达标 |
| **WT-008** | Isolation 9 项强制(Filesystem / Env / Process / Port / Secret / Build Artifact / Dependency Cache / Agent Memory / Temp File) | domain-local-runtime | RFC-018,019 | 500K | POC-030 验证 9 项隔离全部生效 |
| **WT-009** | Reconciliation 协议(Local Runtime Reconnect 后偏差检测) | domain-worktree + domain-local-runtime | WT-006 | 300K | 偏差 = 不可恢复事件,不静默合并 |
| **WT-010** | Completion 7 项检查(Feedback / Test / Build / Conflict / AC / Review / Git State) | domain-worktree | RFC-023,027 | 350K | Project Policy 提供策略,默认全部必须 |
| **WT-011** | Status Independence API 校验(Worktree 状态迁移不操作 WorkItem 字段) | domain-worktree | WT-003 | 150K | API 层强制;Code Review 检查 |
| **WT-012** | Lifecycle Policy(>90d 归档 + 冷热分层) | SRE | WT-001 | 200K | 归档脚本 + 监控告警 |

**Phase 1 合计**:约 **3.55M tokens**(≈ 12-35 人·天)

### Phase 2 (V1,Week 5-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **WT-101** | 跨 DevelopmentExecution Worktree 复用 + Handoff 状态迁移 | domain-worktree | RFC-017 | 300K | HandoffContextPacket 完整传递 |
| **WT-102** | Saved Worktree Views 个性化(§30.3) | domain-collaboration | WT-005 | 250K | 用户可保存 Worktree 过滤视图 |
| **WT-103** | Heatmap Phase 2(Symbol-level 集成) | domain-worktree + domain-context | RFC-028 | 400K | Symbol 维度 Heatmap |
| **WT-104** | Worktree 软删除 vs 硬删除策略实施(Open Question #1) | domain-worktree + SRE | WT-012 | 200K | SRE + DBA 决策落地 |
| **WT-105** | 跨 Project 共享 Worktree(Project 归属,Open Question #4) | domain-worktree | 决策后 | 250K | Tenant / Workspace 维度决策实施 |

**Phase 2 合计**:约 **1.4M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | 负责 Lead | Token 估算 |
|---|---|---|---:|
| **WT-201** | Semantic Conflict Detection(AI 辅助) | domain-worktree + domain-context | 600K |
| **WT-202** | Cross-Worktree Dependency Graph | domain-development | 800K |
| **WT-203** | Multi-Agent Comparison UI(同 Task 多 Worktree 对比) | domain-collaboration | 500K |

**Phase 3 合计**:约 **1.9M tokens**

---

## 依赖矩阵

```
RFC-016 依赖:
  - RFC-017 (DevelopmentExecution 聚合层)
  - RFC-020 (Observed State 分离)
  - RFC-018, RFC-019 (Local Runtime)

RFC-016 被依赖:
  - RFC-027 (ChangeSet 关联 Worktree)
  - RFC-029 (Worktree Conflict Detection)
  - RFC-021 (Agent 绑定 Worktree)
  - RFC-024 (Context Compiler 加载 Worktree)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Worktree 数量爆炸 | Medium | Heatmap 投影 + 冷热分层;UI 分页 + 虚拟滚动 |
| Status Independence 误破坏 | High | 状态机代码层独立;API 严格校验;单元测试 |
| Performance N+1 | Low | JOIN 优化;Repository 层一次性加载;N+1 检测 CI Gate |

## 验收标准(MVP)

1. ✅ `worktrees` 表包含 22 字段 + 三级 tenant 隔离
2. ✅ 17 状态机完整,转换规则单元测试 100% 覆盖
3. ✅ 1 WorkItem → N Worktree 关系成立(无直接外键)
4. ✅ Observed State 走 Projection 表,1s 批量 Throttle
5. ✅ Heatmap 100 Worktree / 10k File < 500ms
6. ✅ Completion 7 项检查默认全部必须
7. ✅ Reconciliation 偏差不静默合并
8. ✅ Code Review 通过(评审检查清单 10 项)

## Token-OLU 总览

- **Phase 1(MVP)**:3.55M tokens ≈ 12-35 人·天
- **Phase 2(V1)**:1.4M tokens ≈ 5-14 人·天
- **Phase 3(V2)**:1.9M tokens ≈ 6-19 人·天
- **MVP + V1 合计**:4.95M tokens ≈ 17-49 人·天(可由 domain-worktree Lead 1 人 14-18 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
