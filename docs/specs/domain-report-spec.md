# domain-report 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-report 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-report` 负责 **Star Report Engine (per crates/domain-report/src/lib.rs v0.1 实施实装)**。

**属于本 crate 的**:
- 10 种报表类型:
-   1. Burndown (Sprint 燃尽图, per REQ-PLAN-005)
-   2. Burnup (Sprint 燃起图)
-   3. Velocity (跨 Sprint 速度)
-   4. CFD (Cumulative Flow Diagram)
-   5. Control Chart (周期时间 + 异常检测)
-   + 5 种 V1 报表 (Cycle Time / Throughput / Workload / SLA / Forecast)

**不属于本 crate 的**:
- WorkItem 数据源 (从 `domain-work-item` Projection 读取, 不持有事实)
- Sprint 数据源 (从 `domain-planning` 读取)

## 2. 关键实体

- `ReportDefinition` (聚合根): report_id / tenant_id / project_id / type (10 种) / config{} / schedule? (cron) / recipient_ids[]
- `ReportSnapshot` (Projection): report_id / generated_at / data{} / data_source_refs[]
- `ReportSchedule`: cron_expression / next_run_at / last_run_at? / enabled

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-REPORT-01 | Report 是 Projection, 不得持有 SoR 业务事实 (per requirements §12 REQ-SEARCH-001) |
| INV-REPORT-02 | Report 数据走 cache 5min TTL, 不实时拉源 |
| INV-REPORT-03 | Report 订阅触发走 worker projection role (per basic-design v0.16 §4.12.2) |

## 4. 接口契约

- `ReportDefinitionCommandPort`: create / update / enable / disable / delete
- `ReportQueryPort`: get / list-by-project / generate (即时生成, 不缓存) / latest-snapshot
- `ReportSchedulePort`: schedule / unschedule / list-pending-runs

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-report` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `report` 触发工单创建 | work-item | Customer-Supplier (Port) | per `report` 提交触发 |
| `report` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `report` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `report` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-REPORT-01: 大报表性能 | 异步生成 + 缓存 + 分页 (per ADR-0026 §3 Fallback Ladder) | — | domain-report §6 |
| RISK-REPORT-02: 数据不一致 (源数据更新中) | snapshot 时间戳 + 增量更新 | — | domain-report §6 |
| RISK-REPORT-03: 订阅触发噪音 | REQ-NOTIF-002 降噪策略 | — | domain-report §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
