# Implementation Plan: PLAN-020 — Observed State vs Business State

> **RFC**: RFC-020
> **Domain Lead**: domain-worktree Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-020, RFC-016, RFC-018
> **相关 Module Spec**: domain-worktree-spec.md
> **相关 PoC**: POC-017, POC-018

---

## 目标(Goals)

1. Business State(`worktrees` 表)与 Observed State(`worktree_status_observed` 表)严格分离
2. Observed State 走独立 Projection 表(最终一致)
3. Write Amplification / Event Volume / Database Growth 全部受控(REQ-DATA-003)
4. UI 区分 Current / Possibly Stale / Offline / Unknown(§23.4)
5. Reconciliation 协议(Local Runtime Reconnect 后偏差检测,§22.6)
6. Observed State 独立 Lifecycle Policy(§5.8)

## 非目标(Non-Goals)

1. ❌ TimescaleDB Hypertables(V1 评估,MVP PostgreSQL 即可)
2. ❌ AI 异常检测(V2)
3. ❌ Event Sourcing 重建状态(§30.6 Non-Goals)
4. ❌ Observed State 跨 Project 共享(tenant 隔离)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-worktree Lead** | Observed State 表 Schema / 索引 | ❌ |
| **domain-local-runtime Lead** | Local Runtime 端上报逻辑 | ❌ |
| **domain-collaboration Lead** | UI Current / Stale / Offline 区分 | ❌ |
| **SRE Lead** | Lifecycle Policy / 数据库性能监控 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **OSS-001** | `worktree_status_observed` Projection 表 Schema(10 字段 + 三级 tenant 隔离) | domain-worktree | RFC-020 | 200K | 与 `worktrees` 表分离;无外键约束(避免影响主表) |
| **OSS-002** | `worktree_id` + `last_heartbeat` 复合索引 | domain-worktree | OSS-001 | 150K | 查询 P95 < 50ms |
| **OSS-003** | Local Runtime 1s 批量上报(Throttle) | domain-local-runtime | OSS-001 | 300K | 网络流量可控;无风暴 |
| **OSS-004** | `RecordObservedStateCommand` Port 方法 | domain-worktree | OSS-001 | 200K | 接受 4 类字段:dirty / test / build / agent_running |
| **OSS-005** | UI 状态判断逻辑(Current / Stale / Offline / Unknown) | domain-collaboration | OSS-001 | 250K | TTL 阈值:Current < 5s / Stale 5-60s / Offline > 60s |
| **OSS-006** | UI 渲染层(Worktree Dashboard 区分状态) | domain-collaboration | OSS-005 | 300K | Dashboard P95 < 500ms(100 Worktree) |
| **OSS-007** | Reconciliation 协议 §22.6(Local Runtime Reconnect 触发) | domain-worktree + domain-local-runtime | RFC-016 | 400K | 偏差 = 不可恢复事件;不静默合并 |
| **OSS-008** | Lifecycle Policy(>7d 归档,§5.8) | SRE | OSS-001 | 200K | 归档脚本 + 监控 |
| **OSS-009** | Stale Worktree UI 警告(避免基于过期数据决策) | domain-collaboration | OSS-005 | 200K | UI 明确标注"Possibly Stale" |
| **OSS-010** | Observed State 写放大监控(metric) | SRE | OSS-001 | 200K | 监控 QPS < 10/s / Worktree |
| **OSS-011** | Event Volume 监控(节流后) | SRE | OSS-003 | 200K | 监控 QPS < 10/s / Worktree |

**Phase 1 合计**:约 **2.6M tokens**

### Phase 2 (V1,Week 5-8)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **OSS-101** | TimescaleDB Hypertables 时序压缩(评估) | domain-worktree + SRE | OSS-001 | 500K | 数据压缩比 10x+;查询性能不降 |
| **OSS-102** | Reconciliation 报告增强(偏差分类 + 修复建议) | domain-worktree | OSS-007 | 300K | UI 展示偏差 + 建议 |
| **OSS-103** | Observed State 实时订阅(WebSocket) | domain-collaboration | OSS-005 | 400K | UI 实时刷新,无轮询 |
| **OSS-104** | Lifecycle Policy V1(冷热分层,30d 热 / 30-90d 温 / >90d 冷) | SRE | OSS-008 | 350K | 存储成本下降 50% |

**Phase 2 合计**:约 **1.55M tokens**

### Phase 3 (V2,Week 9+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **OSS-201** | AI 异常检测(Observed State Pattern Recognition) | 800K |
| **OSS-202** | Predictive Reconciliation | 600K |
| **OSS-203** | Multi-Region Observed State Federation | 1.0M |

**Phase 3 合计**:约 **2.4M tokens**

---

## 依赖矩阵

```
RFC-020 依赖:
  - RFC-016 (Worktree 聚合根)
  - RFC-018 (Local Runtime 上报路径)

RFC-020 被依赖:
  - RFC-021 (Agent Status Observed State)
  - RFC-024 (Context Compiler 加载 Observed State)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Observed State 上报延迟 | Medium | 1s 批量 Throttle + Reconciliation 兜底;UI 明确标注 Stale |
| Projection 表数据膨胀 | Medium | Lifecycle Policy(>7d 归档);TimescaleDB 压缩(V1) |
| Reconciliation 偏差不静默合并 | High | 偏差 = 不可恢复事件;强制 re-sync 或人工介入;Audit 记录 |
| UI 与 Business State 不一致 | Medium | UI 区分 Current / Stale / Offline / Unknown;Stale 警告 |

## 验收标准(MVP)

1. ✅ `worktree_status_observed` 表与主表分离
2. ✅ Local Runtime 1s 批量上报
3. ✅ UI 区分 Current / Stale / Offline / Unknown
4. ✅ Reconciliation 协议触发(Local Runtime Reconnect)
5. ✅ Lifecycle Policy >7d 归档
6. ✅ Write Amplification 受控(主表 UPDATE QPS < 1/s)
7. ✅ Event Volume 受控(节流后 QPS < 10/s / Worktree)
8. ✅ Stale Worktree UI 警告
9. ✅ tenant_id / workspace_id / project_id 三级隔离
10. ✅ POC-017 验证 1k Worktree 状态 1s 内同步

## Token-OLU 总览

- **Phase 1(MVP)**:2.6M tokens ≈ 9-26 人·天
- **Phase 2(V1)**:1.55M tokens
- **Phase 3(V2)**:2.4M tokens
- **MVP + V1**:4.15M tokens(可由 domain-worktree Lead 1 人 10-14 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
