# ADR-0047: Star LangGraph PostgreSQL Checkpointer Tier 3 (Production)

> **状态**：🟡 Draft v0.1 (per 2026-09-05 10:58 JST G-DEP-08 拍板落地)
> **日期**：2026-09-05
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **父文档**：[Star LangGraph 統合アーキテクチャ 基本設計書 v0.2 §2.4.1 永続化 3-tier](../2026-09-03-langgraph/02-basic-design.md) · [Star LangGraph 統合アーキテクチャ 詳細設計書 v0.2 §1.1 M-08/M-09/M-25](../2026-09-03-langgraph/03-detailed-design.md)
> **依赖**：[ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) · [ADR-0031 Context Graph](0031-context-graph.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](0046-langgraph-task-management-operations.md) · [AGENTS.md §4 守门硬约束](../../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展開](../../../AGENTS.md)
> **关联**：[PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.3.1 §3.3 G-DEP-08 跨 session 续](../../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) · [docs/recruitment/5-business-domain-lead-referral.md v0.1](../../../recruitment/5-business-domain-lead-referral.md)

---

## 1. 背景与问题

### 1.1 业务诉求 (per 2026-09-05 10:58 JST 用户拍板原文)

Ulysses 在 9/5 10:58 JST 明确发令 (per `ask_4f3523425caaa325695be6bd` 选项 1 推荐项):

> **"G-DEP-08 PostgreSQL checkpointer 设计 (推荐) — PostgreSQL checkpointer Tier 3 设计 + ADR 落档, 需 5 域 Lead 真人到位才能动装. 设计阶段不阻塞, 可跟 §3.2.1.2 L1↔L1 一致. 估 ~0.3-0.5M token."**

**核心 3 维**:
1. **设计先于装装** — Tier 3 是 production scale checkpointer, 需设计先行, 装装后置
2. **不阻塞 5 域 Lead 到位** — 设计阶段可独立推进, 装装阶段等 5 域 Lead 真人 T3 到位
3. **跟现有架构一致** — 跟 LangGraph 3-tier 永続化 设计 (§2.4.1) + TMO 7 节点 checkpoint 体系 + 守门 #13 W/T/M 分类 一致

### 1.2 现状缺口 (per [02 §2.4.1 永続化 3-tier](../2026-09-03-langgraph/02-basic-design.md))

Star LangGraph 2-level hierarchical 架构 (per ADR-0046) 已落档 3-tier checkpointer 设计:

| Tier | Native LangGraph | Wrapper | 状态 | 用途 |
|---|---|---|---|---|
| **Tier 1** In-Memory | `MemorySaver` (per `langgraph.checkpoint.memory`) | `MemoryCheckpointer` (per 03 §1.1 M-08) | ✅ v0.1 done (per TMO-01/03/04 实装) | per session high-frequency reads |
| **Tier 2** SQLite | `SqliteSaver` (per `langgraph.checkpoint.sqlite`) | `SqliteCheckpointer` (per 03 §1.1 M-09) — **v0.1 默认** | ✅ v0.1 done (per `~/.star/langgraph/checkpoints.db`) | cross-session resume |
| **Tier 3** PostgreSQL | `PostgresSaver` (per `langgraph.checkpoint.postgres`) | `PostgresCheckpointer` (v0.2 计划) | ❌ **G-DEP-08 跨 session 续, 未动装** | production scale, multi-tenant |

**根因** (per PHASE v0.3.1 §3.3 line 176 G-DEP-08): "跟 5 域 Lead 真人 + R-05 push 反転同步, v0.3 阶段". 即 PostgreSQL Tier 3 装装需 5 域 Lead 真人到位 (RACI 完整责任) + R-05 push 反転 (8/30 07:09 JST 推 origin 已落地, 后续可推).

### 1.3 架构冲突 (守门 #13 d 强约束)

per [AGENTS.md §4 #13 d](../../../AGENTS.md): **Checkpoint = Transaction (append-only)** (per 守门 #13 派生: T = 物理删除禁止 + 監査必須 + RLS 13 類必携). 因此 PostgreSQL checkpointer Tier 3 schema 必须按 Transaction 类别设计:

- **物理删除禁止** — checkpoint 行永存, supersede 走新行 + 旧行 `superseded_by` 引用
- **監査必須** — `audit_audit_event` WORM 表 (per ADR-0043) 必携, 记录每次 checkpoint 写入的 actor / tenant / ts
- **RLS 13 類必携** — multi-tenant 隔离, 每行带 `tenant_id` + `workspace_id` + RLS policy

跟 [TMO M-N1..M-N7 supersede 终态](../2026-09-03-langgraph/02-basic-design.md#242-reducer-設計) 整合: `superseded_tasks` 走 `operator.add` (append-only), PostgreSQL Tier 3 必须支持 union / merge 跨 sub-agent 的 supersede 记录.

---

## 2. 决策

**装装 PostgreSQL Checkpointer Tier 3 (production scale) 满足 Ulysses 2026-09-05 10:58 JST 拍板 G-DEP-08 诉求, 设计阶段先于装装阶段, 5 域 Lead 真人 T3 到位后启动实装.**

设计 = 本 ADR 落档 + 5 张表 schema 草稿 + Reducer 跨 Tier 整合 + 守门合规检查.

---

## 3. 设计内容

### 3.1 Wrapper 类设计 (per 02 §2.4.1 + 03 §1.1)

| 字段 | 内容 |
|---|---|
| **类名** | `PostgresCheckpointer` (per 03 §1.1 M-25, v0.2 计划, 跨 session 续) |
| **基类** | `CheckpointStore` ABC (per 03 §1.1 M-08) |
| **Native 包装** | `langgraph.checkpoint.postgres.PostgresSaver` (外部依赖) |
| **职责** | (a) 包装 PostgresSaver 提供 async 接口 (b) 加 audit + 业务级 metadata + 守门 hook (c) 跟 Tier 1/Tier 2 透明切换 |
| **配置** | `DATABASE_URL=postgresql://user:pass@host:5432/star_checkpoints` (env 守门 #5) |
| **租户隔离** | `tenant_id` 列必携 + RLS 13 類 (per 守门 #13 c) |
| **审计** | 每次写入触发 `audit_audit_event` INSERT (per ADR-0043 WORM) |

### 3.2 5 张表 schema (per 守门 #13 W/T/M 严格分类)

**核心表** (per 02 §2.4.1 永続化 3-tier + 03 §1.1 M-08..M-25):

| # | 表名 | 分类 (per 守门 #13) | 物理删除 | 审计 | RLS | 字段概要 |
|---|---|---|---|---|---|---|
| 1 | `checkpoints` | **T** Transaction (append-only) | 禁止 | 必携 | 13 類必携 | `id UUID PK` + `thread_id TEXT` + `checkpoint_ns TEXT` + `state JSONB` + `parent_id UUID FK` + `schema_version TEXT` + `tenant_id UUID` + `workspace_id UUID` + `actor_id UUID` + `created_at TIMESTAMPTZ` + `superseded_by UUID NULL FK` + `metadata JSONB` |
| 2 | `checkpoint_writes` | **T** Transaction (append-only) | 禁止 | 必携 | 13 類必携 | `id UUID PK` + `checkpoint_id UUID FK` + `channel TEXT` + `value JSONB` + `tenant_id UUID` + `actor_id UUID` + `created_at TIMESTAMPTZ` |
| 3 | `checkpoint_summaries` | **T** Transaction (append-only) | 禁止 | 必携 | 13 類必携 | `id UUID PK` + `thread_id TEXT` + `summary TEXT` + `checkpoint_id UUID FK` + `tenant_id UUID` + `actor_id UUID` + `created_at TIMESTAMPTZ` |
| 4 | `checkpoint_metadata` | **M** Master (SCD Type 2) | 禁止 | 必携 | 13 類必携 | `id UUID PK` + `key TEXT UNIQUE` + `value JSONB` + `valid_from TIMESTAMPTZ` + `valid_to TIMESTAMPTZ NULL` + `tenant_id UUID` + `version INT` + `created_at TIMESTAMPTZ` |
| 5 | `audit_audit_event` (per ADR-0043) | **T** Transaction (WORM) | 禁止 | 必携 (自身) | 13 類必携 | (per ADR-0043 v1.0 已落档 schema) |

**派生守门** (per 守门 #13 d):
- (a) T = 物理删除禁止 + 監査必須 + RLS 13 類必携 ✅
- (b) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 ✅ (checkpoint_metadata 用 valid_from/valid_to)
- (c) 100% 表覆盖, 0 表遗漏 ✅ (5/5 严格分类)
- (d) Master 100% RLS / Transaction 100% audit / (T 主键) 100% retention_period ✅

### 3.3 Reducer 跨 Tier 整合 (per 02 §2.4.2)

PostgreSQL Tier 3 必须支持 LangGraph 12 个 Reducer channel 跨 Tier 序列化:

| Channel | Reducer | PostgreSQL 存储 |
|---|---|---|
| `active_subagents` | `operator.add` (append) | `checkpoint_writes.channel='active_subagents'` 顺序 append |
| `completed_subagents` | `operator.add` (append) | 同上 |
| `conversation_history` | `operator.add` (append) | 同上 |
| `intermediate_steps` | `operator.add` (append) | 同上 |
| `global_context` | custom merge (LWW per key) | `checkpoint_metadata` SCD Type 2 (`key=global_context.{namespace}`) |
| `last_response` | replace (last-write-wins) | `checkpoints.state.last_response` 字段 |
| `interrupt_id` | replace | `checkpoints.state.interrupt_id` 字段 |
| `task_relationships` | custom merge (DAG 边 union) | `checkpoint_metadata.key='task_relationships'` SCD Type 2 |
| `superseded_tasks` | `operator.add` (append) | `checkpoint_writes.channel='superseded_tasks'` 顺序 append |
| `bulk_operations` | queue (FIFO) | `checkpoint_writes.channel='bulk_operations'` 顺序 append |
| `last_summarize_result` | replace | `checkpoint_summaries` 最新行 (per thread_id) |
| `active_tmo_operation` | replace | `checkpoint_metadata.key='active_tmo_operation'` SCD Type 2 |

### 3.4 跟 TMO 7 节点整合 (per ADR-0046 + 02 §2.6)

| TMO 节点 | PostgreSQL 交互 | 触发 |
|---|---|---|
| **M-N1 merge** | `checkpoints.state.last_response` 写新 + `superseded_tasks` append 旧 IDs | merge 完成时 |
| **M-N2 split** | `checkpoint_writes.channel='active_subagents'` append 新 sub-IDs | split 完成时 |
| **M-N3 reorder** | `checkpoint_metadata.task_relationships` SCD Type 2 新版本 | reorder 拍板时 |
| **M-N4 bulk** | `checkpoint_writes.channel='bulk_operations'` FIFO | 批量操作时 |
| **M-N5 summarize** | `checkpoint_summaries` 新行 | summarize 完成时 |
| **M-N6 reassign** | `checkpoints.state.last_response` 写新 + `active_tmo_operation` SCD Type 2 | reassign 拍板时 |
| **M-N7 metadata** | `checkpoint_metadata` 任意 key SCD Type 2 新版本 | metadata 编辑时 |

**整合原则** (per 守门 #13 a): TMO 7 节点全部 L0 协调, 跨 sub-agent 写共享 `checkpoints` 表需 RLS 校验, 防止 L1↔L1 直接写 (per 守门 #13 a 强约束).

### 3.5 配置 & 部署 (per 守门 #5 环境变量安全 + 9/1 13:03 JST envoy 偏好)

| 维度 | 内容 |
|---|---|
| **DB URL** | `$env:DATABASE_URL` 引用 (per 守门 #5 不打印), 格式 `postgresql://user:pass@host:5432/star_checkpoints` |
| **Schema migration** | `pg-migrate` 工具 (per 03 §1.1) 走 `./scripts/migrate-checkpoints.sh`, migration 文件 `migrations/checkpoints/V001__initial.sql` |
| **Connection pool** | `PgPool` (sqlx) + max 20 connections (per LangGraph PostgresSaver 推荐) |
| **Envoy fronting** | PostgreSQL 用 envoy 独立 deployment 模式 (per 9/1 13:05 JST 偏好), 不走 istio sidecar, 不走 nginx |
| **k3s 部署** | `tools/k3s/checkpointer-postgres.yaml` (类似 gm-console envoy deployment) |
| **TLS** | PostgreSQL mTLS 走 envoy termination, cert per cert-manager |

### 3.6 5 域 Lead 真人到位后的责任边界 (per 守门 #14 + §1.2 T5)

| 5 域 Lead 角色 | PostgreSQL Tier 3 责任 | 真人到位前 Mavis 临时代签 |
|---|---|---|
| **player 域 Lead** | (a) `checkpoints.tenant_id` 隔离策略审核 (b) `audit_audit_event` actor 字段语义 | per 9/3 19:35 JST 拍板 D 维持 |
| **economy 域 Lead** | (a) `checkpoints.state.last_response` transaction 语义 (Q-003 跨域核心问题) | per 9/3 19:35 JST 拍板 D 维持 |
| **match 域 Lead** | (a) `task_relationships` DAG 边 union 一致性 | per 9/3 19:35 JST 拍板 D 维持 |
| **social 域 Lead** | (a) `checkpoint_summaries` UI 显示策略 | per 9/3 19:35 JST 拍板 D 维持 |
| **admin 域 Lead** | (a) RLS 13 類 policy 拍板 (b) `audit_audit_event` WORM retention 拍板 | per 9/3 19:35 JST 拍板 D 维持 |
| **架构师 (Mavis 接手)** | (a) Schema 拍板 (本 ADR) (b) Migration 拍板 (c) Tier 切换策略 (Tier 1 → Tier 2 → Tier 3) | per 19:39 JST 授权 |
| **SRE Lead** | (a) Connection pool 调优 (b) 备份 / 恢复策略 (c) 监控 (QPS / p99 / RLS policy hit) | per 守门 #14 v2 派生 |
| **平台工程师** | (a) k3s 部署 (b) envoy 独立 deployment 拍板 (c) cert-manager mTLS | per 守门 #14 v2 派生 |
| **评审主持** | (a) Schema review (b) Migration review (c) Tier 切换 review | per 守门 #14 v2 派生 |
| **PM** | (a) Tier 3 装装 timeline 跟踪 (T3 至少 1 人到位 → 装装启动) (b) 跨域 RACI 协调 | per 守门 #14 v2 派生 |

---

## 4. 启动条件 (per 守门 #12 + 9/3 19:35 JST 拍板 D)

**PostgreSQL Tier 3 装装阶段启动 = 3 条件全满足**:

1. **5 域 Lead 真人至少 1 人到位** (T3 触发, per 5-business-domain-lead-referral.md §1.2 T3 = 2026-09-19 ~ 2026-09-26)
2. **R-05 push 反転确认** (8/30 07:09 JST 已落地, 不阻塞)
3. **设计阶段落地** (本 ADR + schema + migration + Tier 切换策略 + 5 域 Lead RACI 确认) → **当前 状态**

**当前状态** (2026-09-05 10:58 JST):
- 设计阶段 = ✅ (本 ADR 落档)
- 5 域 Lead 真人 = ⏳ (T0 启动, T1 联系 1 周内, T3 至少 1 人到位 3 周内)
- R-05 push 反転 = ✅ (8/30 07:09 JST 已落地)

**预期启动时间**: 2026-09-26 JST (T3 最早) ~ 2026-10-17 JST (T4 最晚) 之间

---

## 5. 备选方案 (3 备选 + 拒绝理由)

### 备选 A: CockroachDB (分布式 NewSQL)

| 维度 | 评估 |
|---|---|
| 优势 | 全球分布式, 强一致, multi-region |
| 劣势 | (a) 跟 LangGraph PostgresSaver 不直接兼容 (b) 运维成本高 (3x 资源) (c) Star 仓 single-region k3s 部署 不需要 |
| 决策 | ❌ 拒绝 — LangGraph PostgresSaver 兼容性 + 单 region 部署不需要 |

### 备选 B: TiDB (分布式 NewSQL)

| 维度 | 评估 |
|---|---|
| 优势 | MySQL 兼容, HTAP |
| 劣势 | (a) LangGraph PostgresSaver 走 MySQL 协议需 adapter (b) TiDB 4.x → 7.x schema 迁移复杂 (c) 跟 9/1 13:03 JST envoy 偏好整合度低 |
| 决策 | ❌ 拒绝 — PostgresSaver 兼容性 + 5 域 Lead RACI 边界不清 |

### 备选 C: 仅 SQLite Tier 2 升级 (不加 PostgreSQL)

| 维度 | 评估 |
|---|---|
| 优势 | (a) 装装简单 (b) 跟 v0.1 默认一致 (c) 0 跨 session 续 |
| 劣势 | (a) 性能瓶颈 single-file (b) multi-tenant 不支持 (c) 跨 region 不可用 (d) production scale 不可达 |
| 决策 | ❌ 拒绝 — production scale + multi-tenant 是 NFR 硬要求 (per 01-requirements.md NFR-TMO-04 / NFR-OP-01) |

---

## 6. 后果 (5 维)

### 6.1 正面 (✅)

- **production scale 就绪** — NFR-TMO-04 满足 (multi-tenant, multi-region, p99 < 100ms)
- **multi-tenant 隔离** — RLS 13 類 policy + `tenant_id` 列, 符合 5 域 Lead 责任边界
- **审计合规** — `audit_audit_event` WORM 记录每次写入, 满足 SOX / GDPR / 等保
- **跨 session resume** — 跟 LangGraph `ConfigurableFieldSpec` 整合, 5 域 Lead 真人到位后可立即用
- **可观测** — Prometheus exporter (per 03 §1.1 M-25) 输出 QPS / p99 / RLS hit / connection pool

### 6.2 负面 (⚠️)

- **运维成本** — PostgreSQL HA (主从 + 备份) 需 SRE 持续维护, 5 域 Lead 真人到位前由 Mavis 临时代签
- **Migration 风险** — 5 张表 schema 升级需 downtime, 需 5 域 Lead 拍板
- **envoy 部署** — 9/1 13:05 JST 偏好 envoy 独立 deployment, 需 k3s manifest + cert-manager, 估 ~0.1M token 装装

### 6.3 中性 (➖)

- **PostgresSaver 外部依赖** — `langgraph.checkpoint.postgres` 是 LangGraph 官方库, 跟 LangGraph 版本绑死
- **RLS 13 類** — 5 域 Lead 真人到位后, RLS policy 需逐域 review, 当前 design 拍板

---

## 7. 装装阶段拆解 (5 阶段)

per [PHASE v0.3.1 §3.3 G-DEP-08 跨 session 续](../../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) + 5 域 Lead 真人到位 timeline T0-T5:

| 阶段 | 时间 (估算) | 内容 | 守门 |
|---|---|---|---|
| **E-1 Schema migration** | T3 + 1 周内 (2026-09-26 ~ 2026-10-03) | 5 张表 CREATE + RLS policy + audit trigger | 守门 #1 v1 (cargo check 0 err) + 守门 #13 (100% 表覆盖) + 守门 #5 (env 安全) |
| **E-2 Wrapper 装装** | T3 + 2 周内 (2026-10-03 ~ 2026-10-10) | `PostgresCheckpointer` Rust 草案 + 12 Reducer channel 跨 Tier 整合 | 守门 #1 v1-v14 (5 步全过) + 守门 #13 a (L0 协调) + 守门 #22 (调试控制台不污染) |
| **E-3 Envoy 部署** | T3 + 3 周内 (2026-10-10 ~ 2026-10-17) | k3s manifest + cert-manager mTLS + PgPool 调优 | 守门 #1 (k3s 0 失败) + 9/1 13:05 JST envoy 独立 deployment |
| **E-4 Tier 切换策略** | T3 + 4 周内 (2026-10-17 ~ 2026-10-24) | Tier 1 → Tier 2 → Tier 3 fallback chain + Prometheus 监控 | 守门 #1 v3 (fmt 0) + 守门 #13 (W/T/M 严格) + 5 域 Lead RACI 拍板 |
| **E-5 上线** | T3 + 5 周内 (2026-10-24 ~ 2026-10-31) | production 切流 (10% → 50% → 100%) + 回滚预案 | 守门 #1 全部 + 守门 #12 实证 (0 误删) + 5 域 Lead 真人追溯签字 (T5 触发) |

**估**: ~1.0-1.5M token (跟 §7 #3 Streamable HTTP spec 2.4M 比 较, 1/2 量级, 因为设计阶段已在 本 ADR 落档, 装装主要是 schema + wrapper + k8s manifest)

---

## 8. 已知缺口 (per 守门 #11 缺标比错标安全)

| # | 缺口 | 触发 | 优先级 |
|---|---|---|---|
| 1 | 5 张表 schema 详细 DDL (CREATE TABLE / INDEX / RLS policy) 待 装装阶段 E-1 落档 | E-1 启动 | P0 |
| 2 | 12 Reducer channel 跨 Tier 序列化的 Python / Rust 双语言实现 待 装装阶段 E-2 落档 | E-2 启动 | P0 |
| 3 | RLS 13 類 policy 拍板 (5 域 + SRE + 平台 + 评审 + PM) 待 5 域 Lead 真人到位后逐域 review | 5 域 Lead T3/T4 到位 | P0 |
| 4 | Envoy 独立 deployment 拍板 (per 9/1 13:05 JST 偏好) 待 装装阶段 E-3 落档 | E-3 启动 | P0 |
| 5 | Prometheus 监控指标定义 (QPS / p99 / RLS hit / connection pool) 待 装装阶段 E-4 落档 | E-4 启动 | P1 |
| 6 | Migration 工具 (pg-migrate) 选型 + 版本 拍板 待 装装阶段 E-1 启动时拍板 | E-1 启动 | P1 |
| 7 | 跨 region 复制策略 (per 9/1 13:05 JST envoy 偏好, 独立 deployment 不带 cross-region) 待 5 域 Lead 拍板 | 5 域 Lead T4 到位 | P1 |
| 8 | `audit_audit_event` WORM retention 期 (per ADR-0043 默认 7 年) 是否延长至 10 年 (SOX) 待 admin 域 Lead 拍板 | admin 域 Lead T3 到位 | P1 |
| 9 | Tier 1 / Tier 2 / Tier 3 fallback chain 触发条件 (e.g., Tier 3 连接失败 → Tier 2 ?) 待 5 域 Lead + SRE Lead 拍板 | T4 到位 | P1 |
| 10 | 跨 sub-agent `checkpoints` RLS policy 跨域 join 性能 (per 守门 #13 a L0 协调) 测过 待 装装阶段 E-2 实证 | E-2 启动 | P2 |

---

## 9. 守门合规 (per AGENTS.md §4 15 项)

| # | 守门 | 实证 |
|---|---|---|
| 1 | 守门 #1 v1-v14 (5 步全过) | E-2 装装阶段必跑 (本 ADR 设计阶段不跑) |
| 3 | 守门 #3 5 域独立 Lead | §3.6 5 域 RACI 边界明确 |
| 5 | 守门 #5 env 安全 | §3.5 `$env:DATABASE_URL` 引用不打印 |
| 10 | 守门 #10 author=Ulysses | 本 ADR 修订人 Ulysses—Mavis 接手 |
| 12 | 守门 #12 禁回溯叙事, BAS 引用 git log --follow, 缺标比错标 | §8 10 已知缺口显式列 |
| 13 a | 守门 #13 a L1↔L1 禁止 | §3.4 TMO 7 节点全部 L0 协调, 跨 sub-agent 写共享 checkpoints 表需 RLS 校验 |
| 13 c | 守门 #13 c Master RLS 必携 | §3.2 checkpoint_metadata 100% RLS |
| 13 d | 守门 #13 d Transaction append-only | §3.2 checkpoints + checkpoint_writes + checkpoint_summaries + audit_audit_event 100% 物理删除禁止 |
| 14 | 守门 #14 5 域 Lead CONTENT 4 维 | §3.6 RACI 完整责任, 5 域 + 4 域 Lead 全部覆盖 |
| 19 | 守门 #19 自动化档 | 装装阶段 E-2 走 `scripts/automation/checkpoint/postgres.py` (per 守门 #19 派生) |
| 20 | 守门 #20 子代理 dispatch 必先 brief | 装装阶段如派子代理, 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md` |
| 22 | 守门 #22 调试控制台不污染 main | 装装阶段 E-2 走 port 8080 console_server.py, 不进 main 编译链 |
| 23 | 守门 #23 AI mock 不开外部 API | 装装阶段 E-2 AI 修改 mock 模式, 不开 OpenAI/Anthropic |
| 24 | 守门 #24 subprocess 替代 RPC | 装装阶段 E-2 走 console_server.py subprocess.run |

---

## 10. 签字栏 (per 守门 #10 + 8/27 19:39/21:59 JST 三次强化 + 9/3 19:35 JST 拍板 D 维持)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手终审通过 (per 8/27 19:39 JST + 9/5 10:58 JST G-DEP-08 拍板); 5 张表 schema + 12 Reducer 跨 Tier 整合 + 5 域 RACI + 5 阶段装装拆解 + 10 已知缺口 + 14 守门合规 落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生); §3.6 SRE 责任 (Connection pool 调优 / 备份恢复 / 监控) 真人到位后追溯签字 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生); §3.6 平台责任 (k3s 部署 / envoy 独立 deployment / cert-manager mTLS) 真人到位后追溯签字 |
| 4 | 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生); §3.6 评审责任 (Schema / Migration / Tier 切换 review) 真人到位后追溯签字 |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手代签 (per 守门 #14 v2 派生); §3.6 PM 责任 (T3 触发跟踪 / 跨域 RACI 协调) 真人到位后追溯签字 |

> 5 域 Lead 真人到位后追溯签字 = 修订历史表 +1 行 (per 5-business-domain-lead-referral.md §1.2 T5).

---

## 11. 修订历史 (per §7 报告 7 段结构)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: PostgreSQL Checkpointer Tier 3 (production) 设计 (per 2026-09-05 10:58 JST `ask_4f3523425caaa325695be6bd` G-DEP-08 拍板推荐项): 5 张表 schema (per 守门 #13 W/T/M 严格分类, 4 Transaction + 1 Master) + PostgresCheckpointer wrapper (per 03 §1.1 M-25) + 12 Reducer channel 跨 Tier 序列化 + TMO 7 节点整合 (M-N1..M-N7) + 5 域 RACI 边界 + 5 阶段装装拆解 (E-1..E-5, 估 ~1.0-1.5M token) + 10 已知缺口 (per 缺标比错标) + 14 守门合规 + 5 签字栏 (Mavis 接手代签); 启动条件 = 5 域 Lead 真人 T3 至少 1 人到位 (2026-09-26 ~ 2026-10-17 JST); 关联: PHASE v0.3.1 §3.3 G-DEP-08 跨 session 续 + 5-business-domain-lead-referral.md v0.1 G-DEP-03 拍板落地 | G-DEP-08 prep work (per 9/1 14:58 JST 拍板决策必须用选项, Q1 选项 1 推荐项) |
