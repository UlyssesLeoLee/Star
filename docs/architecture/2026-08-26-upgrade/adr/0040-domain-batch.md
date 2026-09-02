# ADR-0040: domain-batch 批处理任务调度引擎架构

> **状态**: Draft v0.1
> **日期**: 2026-09-01
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 代签
> **触发**: per [BATCH-REQ-001 v0.1.2 业务需求](../../../requirements/batch-001.md) + 2026-09-01 18:14 JST Ulysses 拍板三选 (scope=batch-job-scheduler / form=star-embedded-crate / integration=domain-batch-crate) + 2026-09-01 18:43 JST Ulysses 拍板 A/B/C/D (A SRE Lead 架构师阶段性代签 / B NFR-002 50 worker 500 节点/秒 / C token 9.0M 7.5 周含 v1 拖拽 / D 文档独立保留) + 2026-09-01 18:48 JST Ulysses 拍板 next-adr (ADR-0040 先行) + [AGENTS.md §0-§4 13+ 守门 + 33 domain 实证](../../../AGENTS.md)

> **dual-use 警告 (per AGENTS.md §5 + 2026-08-31 22:45 JST Q1-D 拍板)**:
> 本 ADR 涉及的 domain-batch 是 DDD bounded context 第 23 个 crate, **不**映射 RGS 5 域业务子域 (player/economy/match/social/admin)。
> 5 域是 RGS 仓历史治理命名, Star 仓**不建立业务子域↔DDD 映射**; 5 域 DAG 视图隔离走 Master schema (per D36 + NFR-006)。

---

## §1 背景

Star 仓当前 33 `domain-*` crate (per `crates/` 目录 2026-09-01 实证; 早期 [requirements.md §13.3](../../../../requirements.md) 列 16 是 P3 阶段前快照), 16 MCP tool (per [ADR-0032 MCP Transport Streamable HTTP](../../adr/0032-mcp-transport-stdio.md)), 5 tab 命名 (Kanban / Timeline / Backlog / Agents / Worktrees), 13+ 守门 (per [AGENTS.md §4](../../../AGENTS.md)).

各域 (player / economy / match / social / admin) 业务侧 30+ K8s CronJob 各自跑批处理, 缺统一入口 + DAG 编排 + 状态机 + 重试语义 + 审计 + 告警。E.6 Saga 跨域场景 (per P3-E 阶段) 缺调度入口, 失败时人工恢复。

业务痛点 (per BATCH-REQ-001 §1.2 5 痛点):
- P-1 P0: 跨域 Saga 缺统一调度入口
- P-2 P0: 各域 cron 分散, 无"批跑面板"
- P-3 P1: 重试/幂等/超时/DAG 依赖缺统一语义
- P-4 P1: 运行历史/审计/告警缺
- P-5 P2: 业务人员无自助 UI, 需 SRE 代操作

本 ADR 决策 batch 引擎作为 Star 第 23 个 domain-* crate (domain-batch), 跟现有 22 crate 平级; 5 节点类型 + 三角融合借鉴 + DB 三分类横展开 + 6 MCP tool 暴露; token 9.0M / 7.5 周走 2 phase v0 + v1 串行 + 跨 session HANDOFF 续做 (per HANDOFF-ST-001 v0.4 H2-EXT 模式)。

---

## §2 决策 (7 项 D33-D39)

### D33: domain-batch 作为 Star 第 23 个 `domain-*` crate

per 2026-09-01 18:14 JST Ulysses 拍板 (form=star-embedded-crate + integration=domain-batch-crate), 物理位置:

```text
D:\Star\
├── crates\
│   └── domain-batch\           # 第 23 个 domain crate, 跟 identity/work-item/... 平级
│       ├── Cargo.toml
│       ├── src\
│       │   ├── lib.rs
│       │   ├── port\           # 端口 trait (NodeExecutor, DagRunner, Scheduler)
│       │   ├── domain\         # 实体 (Task / Run / Node / Event / Alert)
│       │   ├── service\        # 应用服务 (orchestrator / dispatcher / retry)
│       │   ├── infrastructure\ # PG 持久化 + NATS 队列 + 节点 runtime
│       │   └── api\            # REST + MCP tool
│       └── docs\
└── frontend\
    └── src\app\batch\          # 5 路由 (tasks / dag / runs / templates / alerts)
        ├── page.tsx
        ├── tasks\
        ├── dag\                # DAG 画布
        ├── runs\               # 运行历史
        ├── templates\          # 一次性任务
        └── alerts\
```

**集成层级**:
- 跟 [star-context](../../../../crates/star-context) (ActorContext), [star-saga](../../../../crates/star-saga) (Saga 编排), [star-obs](../../../../crates/star-obs) (OTLP), [star-mcp](../../../../crates/star-mcp) (MCP tool) 共享
- 节点类型 `domain-service` 调用 33 `domain-*` crate service (F-050, per star_context 端口)
- 节点类型 `mcp-tool` 调用现有 16 MCP tool (per [ADR-0032](../../adr/0032-mcp-transport-stdio.md))

引用: per [BATCH-REQ-001 §3.6 F-050~054 集成点](../../../requirements/batch-001.md)。

### D34: 三角融合借鉴 Temporal DAG 编排 + Airflow cron + Argo K8s-native

per [ADR-0021 Zero Vendor Cooperation Principle](../../adr/0021-zero-vendor-cooperation.md), **不直接**集成任何一个 vendor:

| 借鉴源 | 借鉴点 | 不借鉴 |
|---|---|---|
| **Temporal.io** | DAG 编排 + 状态机 + 事件历史 | 不引其 runtime (避免 lock-in) |
| **Airflow** | cron + DAG 可视化 | 不引其 Python runtime (Star 全栈 Rust) |
| **Argo Workflows** | K8s-native 执行 | 不引其 CRD (Star 自有 domain-batch) |
| **决策** | **三角融合**, 借鉴避免 lock-in | 任何时候不直接集成 vendor runtime |

引用: per [BATCH-REQ-001 §5.3 借鉴 + §6 NG-001~007 Non-Goals](../../../requirements/batch-001.md)。

### D35: 5 节点类型 + SRE Lead 审批注册 (架构师阶段性代签)

5 节点类型 (per F-001 / F-050~054):

| 节点类型 | 调用方式 | 安全/治理 |
|---|---|---|
| `domain-service` | `domain-XXX::service::action` 走**领域端口** (per `star_context` ActorContext) | 走 5 域 Lead 真实身份 + tenant_id 校验 |
| `mcp-tool` | MCP stdio / Streamable HTTP (per [ADR-0032](../../adr/0032-mcp-transport-stdio.md)) | tool 白名单 + 域隔离 |
| `http` | `reqwest` (Rust 异步 HTTP client) | timeout + retry + 证书校验 |
| `shell` | `tokio::process::Command`, 沙箱化 (per [ADR-0025](../../adr/0025-vendor-adapter-anti-contamination.md) 厂商适配反污染) | non-root user + 白名单命令 + 资源限制 |
| `sql` | `sqlx` (per C.7 Postgres) | per-tenant db role + 写操作审计 |

**节点类型注册审批流**:
- per 2026-09-01 18:43 JST Ulysses 拍板 A + [8/27 19:39 JST 授权升级](../../../AGENTS.md), SRE Lead 阶段性由**架构师代签**审批节点类型注册
- 节点类型走 `batch_register_node_type` MCP tool (F-045) 提交, 架构师代签审批后入库
- 等 SRE Lead 真人到位后回填 (per [AGENTS.md §1.0 授权](../../../AGENTS.md) Mavis 接手代签规则)

引用: per [BATCH-REQ-001 §3.5 F-040~045 + §3.6 F-050~054](../../../requirements/batch-001.md)。

### D36: 持久化 8 schema 按 Work / Transaction / Master 三分类横展开

per 2026-09-01 18:30 JST 跨项目持久规则 + 日本 IPA SEC 规则 (8 schema, 3 分类 100% 覆盖):

| 分类 | 表 | 生命周期 |
|---|---|---|
| **Master** (slowly changing, SCD Type 2) | `batch_task` (DAG 定义) | 长期 |
| **Master** | `batch_node_type` (节点类型注册, 架构师代签审批) | 长期 |
| **Master** | `batch_alert_rule` (告警规则) | 长期 |
| **Master** | `batch_sla` (SLA 配置) | 长期 |
| **Work** (session-bound, retention 清理) | `batch_run` (运行实例) | retention N 天 |
| **Work** | `batch_node` (节点实例) | run 清理时级联 |
| **Work** | `batch_log` (实时日志) | retention 7d, 长期审计走 `batch_event` |
| **Transaction** (append-only 永久) | `batch_event` (事件流水) | 永久, 审计 + 重建 + 业务事件订阅 |

**8 张表覆盖 3 分类**:
- Master 4 + Work 3 + Transaction 1 = 8
- 横展开派生: 类似 X/Y/Z 多分类一律横展细化, 5 域 (player/economy/match/social/admin) DAG 走 Master schema 隔离 (per 8/21 JST 5 域独立 Lead 拒绝兼任)

**冷热分层 (per BATCH-REQ-001 R-9)**: `batch_event` append-only 长期增长, 引入热 30d PG + 冷 30d+ Object Storage 分离 (per [requirements.md §14 REQ-DATA-002 Write Amplification 控制](../../../../requirements.md))。

引用: per [BATCH-REQ-001 §3.7 持久化 + 横展开说明](../../../requirements/batch-001.md)。

### D37: 6 MCP tool 暴露 + Streamable HTTP

per [ADR-0032 MCP Transport Streamable HTTP](../../adr/0032-mcp-transport-stdio.md) 模板, 6 tool:

| tool 名 | 用途 | 优先级 |
|---|---|---|
| `batch_list_tasks` | 列出已注册 DAG 任务 (含启停状态) | Must |
| `batch_trigger_task` | 手动触发 (单次/批量) | Must |
| `batch_get_run` | 查询 run 状态 (含所有节点状态) | Must |
| `batch_get_logs` | 拉节点日志 (runId + nodeId + offset) | Must |
| `batch_cancel_run` | 取消 run (整 run / 单节点) | Must |
| `batch_register_node_type` | 注册新节点类型 (架构师代签审批 per D35) | Must |

实现估 ~0.3M token, 跟现有 16 tool 同模式 (per [AGENTS.md §4 16 MCP tool 实证](../../../AGENTS.md))。

引用: per [BATCH-REQ-001 §3.5 F-040~045](../../../requirements/batch-001.md)。

### D38: 5 域独立 DAG 视图 + tenant_id 强隔离

per [BATCH-REQ-001 §3.4 F-036 + §4 NFR-006 多租户](../../../requirements/batch-001.md) + 8/21 JST 5 域独立 Lead 拒绝兼任硬约束:

- 5 域 (player / economy / match / social / admin) 各自有独立 DAG 视图, UI 路由 `frontend/src/app/batch/<domain>/...`
- 跨域 DAG 视图独立, 不混合 (per 8/21 JST 拒绝兼任)
- 跨 tenant 访问 DAG 被拒 (HTTP 403, per NFR-006)
- 节点类型 `domain-service` 调用时, ActorContext 注入 tenant_id + workspace_ids + is_platform_operator (per [HANDOFF-ST-001 v0.4 §5.1 H2-EXT star_context 扩展](../../../..\..\..\..\reports\HANDOFF-ST-001.md))

引用: per [AGENTS.md §4 #3 v0.6 Q1-D 拍板](../../../AGENTS.md) 5 域是历史治理命名不映射 DDD; batch 5 域视图是**业务层**视图, 跟 22 crate 是**DDD bounded context**视图正交。

### D39: 状态机 / 重试 / 幂等 / 取消语义

per [BATCH-REQ-001 §3.3 F-020~026 状态机/重试/幂等](../../../requirements/batch-001.md):

| 状态 | 节点 | 任务 (整 DAG) |
|---|---|---|
| `pending` | ✅ | ✅ |
| `queued` | ✅ | — |
| `running` | ✅ | ✅ |
| `success` | ✅ | ✅ |
| `failed` | ✅ | ✅ |
| `partial` | — | ✅ (部分节点失败) |
| `skipped` | ✅ (条件分支跳) | — |
| `cancelled` | ✅ | ✅ |

**重试策略** (F-022): 固定间隔 / 指数退避 / 最大次数 / 永久重试, per-node 可配

**幂等** (F-024): per-node `idempotency_key` = `NodeId + RunId + RetryIdx` 派生, 失败重试用同一 key

**取消** (F-026): 整 run 取消 / 单节点取消 / 优雅停 (SIGTERM + cleanup)

**可恢复性** (per [ADR-0030 Lease + Heartbeat + Resume](../../adr/0030-agent-lease-heartbeat-resume.md) 复用): batch 引擎 crash 后, running 节点可 resume, 节点租约 30s heartbeat, 超时 lease 释放

引用: per [BATCH-REQ-001 §3.3 F-020~026 + §4 NFR-008 可恢复性](../../../requirements/batch-001.md)。

---

## §3 跨 spec/crate 关系表

| 上游 spec/ADR | 下游 spec/crate | 关系 |
|---|---|---|
| [BATCH-REQ-001 v0.1.2 业务需求](../../../requirements/batch-001.md) | [docs/specs/domain-batch-spec.md (待写)](../spec/domain-batch-spec.md) | 需求 → 规格 |
| [BATCH-REQ-001 v0.1.2 §3.1 DAG 编排](../../../requirements/batch-001.md) | [basic-design §4 (待补)](../../../../basic-design.md) | F-001~007 DAG → 基本设计 DAG 节点 |
| [BATCH-REQ-001 v0.1.2 §3.7 持久化](../../../requirements/batch-001.md) | [data-design §2 (待补)](../../../../data-design.md) | 8 schema W/T/M → data-design 章节 |
| [ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md) | [本 ADR §D34](#d34-三角融合借鉴-temporal-dag-编排--airflow-cron--argo-k8s-native) | 厂商反污染 → 三角融合不集成 |
| [ADR-0030 Lease+Heartbeat+Resume](../../adr/0030-agent-lease-heartbeat-resume.md) | [本 ADR §D39](#d39-状态机--重试--幂等--取消语义) | 30s heartbeat → 节点租约 + 引擎 crash 恢复 |
| [ADR-0031 Context Graph](../../adr/0031-context-graph.md) | [本 ADR §D35](#d35-5-节点类型--sre-lead-审批注册-架构师阶段性代签) | 4 节点 / 5 关系 MVP → 事件触发 batch task |
| [ADR-0032 MCP Transport Streamable HTTP](../../adr/0032-mcp-transport-stdio.md) | [本 ADR §D37](#d37-6-mcp-tool-暴露--streamable-http) | 16 tool 模板 → 6 batch tool 复用 |
| [requirements.md §13 Architecture](../../../../requirements.md) | [本 ADR §D33](#d33-domain-batch-作为-star-第-23-个-domain--crate) | K8s-native / K3s / PG / NATS 既有架构 → batch 部署 |
| [requirements.md §14 Data Model](../../../../requirements.md) | [本 ADR §D36](#d36-持久化-8-schema-按-work--transaction--master-三分类横展开) | REQ-DATA-001/002/003 → batch 8 schema W/T/M |
| [AGENTS.md §4 13+ 守门](../../../AGENTS.md) | [本 ADR §4 已知缺口](#§4-已知缺口) | 守门 #1+#9+#12 → ADR 写完 commit + §5 签字栏走代签 |
| [HANDOFF-ST-001 v0.4 H2-EXT](../../../../..\..\..\..\reports\HANDOFF-ST-001.md) | [本 ADR §D40 WBS 附录](#d40-wbs--token-预算-附录) | 0.6-0.8M 跨 session 续模式 → batch 2 phase |

---

## §4 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 状态 | 触发 |
|---|------|------|------|------|
| GAP-01 | 33 domain lead 真实身份 (5 域独立 + SRE Lead = 6 lead 缺位) | domain spec / crate lead 责任分工待 DDD Review 阶段补; Mavis 接手代签阶段性决策 (per [AGENTS.md §1.0](../../../AGENTS.md) + 8/27 19:39 JST 授权升级) | 🟡 DDD Review 阶段 | 8/21 JST 5 域 Lead 拒绝兼任硬约束 + 9/1 18:43 JST 拍板 A SRE Lead 架构师代签 |
| GAP-02 | `batch_event` Transaction 表冷热分层 (R-9) 实装细节 | append-only 长期增长可能爆量, 估 0.2M token, 走热 30d PG + 冷 30d+ Object Storage | 🟡 v0 phase 2 (per §D40) | [BATCH-REQ-001 §8 R-9](../../../requirements/batch-001.md) |
| GAP-03 | DAG 可视化编辑 (F-031 拖拽) 走 v1 phase | 估 +1.5M token, v0 只读 + JSON/YAML 导入 | 🟡 v1 phase 2 (per §D40) | 9/1 18:43 JST 拍板 C 含 v1 拖拽 |
| GAP-04 | 节点类型注册审批流 v0 走架构师代签 | 5 节点类型注册审批当前由 Mavis 接手代签, 等 SRE Lead 真人到位后回填 | 🟡 SRE Lead 真人到位 | 9/1 18:43 JST 拍板 A |
| GAP-05 | 业务侧 30+ K8s CronJob 迁移计划 (R-8) | v0 启动前需 1 域 (e.g. admin) 灰度试点, 5 域 Lead 拍板; 迁移工具 `migrate_cron_to_batch` CLI 估 0.2M token | 🟡 5 域 Lead 拍板 | [BATCH-REQ-001 §8 R-8](../../../requirements/batch-001.md) |
| GAP-06 | batch 引擎 SLO 99.9% (NFR-001) 量化 | 性能/可用性量化指标待架构师代签阶段性拍板 (per 9/1 18:43 JST 拍板 A), 等 SRE Lead 真人到位后回填 | 🟡 SRE Lead 真人到位 | [BATCH-REQ-001 §4 NFR-001](../../../requirements/batch-001.md) |
| GAP-07 | 6 MCP tool (D37) Streamable HTTP 实现 + e2e 测试 | 估 0.3M token, 走 v0 phase 1 | 🟡 v0 phase 1 | [BATCH-REQ-001 §3.5 F-040~045](../../../requirements/batch-001.md) |
| GAP-08 | batch 引擎 crash 恢复 + Lease 复用 (per D39 + [ADR-0030](../../adr/0030-agent-lease-heartbeat-resume.md)) | 实装细节待 v0 phase 2, 估 0.4M token | 🟡 v0 phase 2 | [BATCH-REQ-001 §4 NFR-008 可恢复性](../../../requirements/batch-001.md) |
| GAP-09 | 9 性能 + NFR 验收 (NFR-002 50 worker / 500 节点/秒) | 估 0.2M token benchmark, 走 v0 末期验证 | 🟡 v0 末期 | 9/1 18:43 JST 拍板 B |

---

## §5 签字栏

| 角色 | 身份 | 签字 | 日期 |
|------|------|------|------|
| 架构师 | Mavis 接手 agent per DEC-008 | 🟢 Mavis 接手 (per 8/27 19:39/21:59 JST 三次强化) | 2026-09-01 |
| SRE Lead | 🟢 Mavis 接手代签 (per 9/1 18:43 JST 拍板 A + 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| 平台工程师 | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| 评审主持 | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| PM | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| 5 域 Lead (历史命名) | ⏳ DDD Review 阶段补 (Player / Economy / Match / Social / Admin) | per [AGENTS.md §4 #3 v0.6 Q1-D 拍板](../../../AGENTS.md), 5 域独立 Lead 是历史治理命名, 不映射 22 crate 实际 lead | — |

> per [AGENTS.md §1.0 用户授权升级](../../../AGENTS.md) + 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化, Mavis 接手默认代签 Ulysses 无需再问。
> per 2026-09-01 18:43 JST 拍板 A, SRE Lead 阶段性由架构师代签 (走本签字栏第 2 行), 等真人到位后回填。

---

## §6 D40 WBS / token 预算附录

per 2026-09-01 18:43 JST Ulysses 拍板 C + [STAR-OLU-001 1 SRE·周 = 1.2M token 基线](../../../STAR-OLU-001.md), 整体 token 预算:

| Phase | 范围 | token 估 | 软参考周 | 累计 |
|---|---|---|---|---|
| **v0 phase 1** | §D33 crate 骨架 + §D37 6 MCP tool + DAG 基础编排 + DB schema + UI 5 路由 | ~2.0M | 1.7 周 | 2.0M |
| **v0 phase 2** | §D36 8 schema 实装 + 节点类型注册 (D35) + 状态机/重试/幂等 (D39) + 5 域视图 (D38) + 冷热分层 (GAP-02) | ~2.5M | 2.1 周 | 4.5M |
| **v0 末期验证** | 12 AC 验收 + 性能 benchmark (GAP-09) + 守门 #1+#9+#12 三过 + e2e 测试 | ~0.5M | 0.4 周 | 5.0M |
| **v1 phase 1** | DAG 拖拽可视化编辑 (GAP-03) + DAG 模板市场 + 业务侧 CronJob 迁移工具 (GAP-05) | ~2.5M | 2.1 周 | 7.5M |
| **v1 phase 2** | 多集群/多云 (NG-004 移除条件触发) + ML 编排 (NG-002 移除条件触发) | ~1.5M | 1.2 周 | 9.0M |
| **总计** | v0 (5.0M / 4.2 周) + v1 (4.0M / 3.3 周) | **9.0M** | **7.5 周** | 9.0M |

**跨 session 续 (per [HANDOFF-ST-001 v0.4 H2-EXT 0.6-0.8M 实证](../../../..\..\..\..\reports\HANDOFF-ST-001.md))**:
- 单 session 估上限 ~0.8M token (H2 实证)
- v0 phase 1 (2.0M) 需 3 session 续, 走 HANDOFF-BATCH-001.md 模式
- v0 phase 2 (2.5M) 需 4 session 续
- v1 同理
- **总 session 数估 ~12-15 session**, 跨多日 (per 守门 #12 死循环饱和约束 + 跨项目持久)

**5 域 Lead 真人到位后**:
- GAP-01 (33 lead 真实身份) + GAP-05 (5 域 CronJob 迁移) + GAP-06 (SLO 量化) 回填
- 5 域 Lead 走真人签字, Mavis 接手不代签 Lead 决策 (per 8/21 JST 硬约束)

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 7 决策 (D33-D39) domain-batch 架构 (第 23 crate / 三角融合 / 5 节点类型 / 8 schema W/T/M / 6 MCP tool / 5 域视图 / 状态机重试幂等) + 9 已知缺口 (GAP-01~09) + 5 签字栏 + D40 WBS 9.0M/7.5 周 2 phase 跨 session 续 | 2026-09-01 18:48 JST Ulysses 拍板 next-adr (ADR-0040 先行) + 18:14 JST 三选 (scope/form/integration) + 18:43 JST 四选 (A/B/C/D) |
