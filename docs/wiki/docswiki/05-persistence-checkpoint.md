# 05 — 3-Tier Checkpoint + 5 张表 W/T/M 分类拓扑

> **数据源**: S2 §2.4 [LangGraph 02 §2.4](../../architecture/2026-09-03-langgraph/02-basic-design.md) + S7 [ADR-0047 PostgreSQL Checkpointer Tier 3](../../architecture/2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md)
> **核心约束**: per 守门 #13 DB 三類横展開 (W/T/M) 強制分類, 100% 表覆盖, 禁混合分類

---

## 1. 3-Tier Checkpoint 拓扑 (per S2 §2.4 + S7 §1.2)

```mermaid
flowchart TB
    subgraph T1["Tier 1: In-Memory (per session)"]
        T1N["MemorySaver<br/>langgraph.checkpoint.memory<br/>per S2 §2.4.1 L392"]
        T1W["Wrapper: MemoryCheckpointer<br/>M-08 checkpoints.store ABC<br/>per S3 §1.2 L161"]
        T1C["特性: 最快, 无持久化<br/>高頻読み, session 内"]
    end

    subgraph T2["Tier 2: SQLite (default v0.1)"]
        T2N["SqliteSaver<br/>langgraph.checkpoint.sqlite<br/>per S2 §2.4.1 L401"]
        T2W["Wrapper: SqliteCheckpointer<br/>M-09 checkpoints.sqlite<br/>per S3 §1.2 L162"]
        T2C["特性: 持久化, 跨进程<br/>默认 v0.1, production-ready<br/>per S2 §2.4.1 L399-401"]
    end

    subgraph T3["Tier 3: PostgreSQL (v0.2 production)"]
        T3N["PostgresSaver<br/>langgraph.checkpoint.postgres<br/>per S7 §3.1 L66"]
        T3W["★ Wrapper: PostgresCheckpointer<br/>M-25 跨 session 续<br/>per S7 §3.1 L64"]
        T3C["特性: production<br/>5 域 Lead T3 至少 1 人到位触发<br/>per S7 §4 启动条件"]
    end

    T1 -->|"async flush<br/>per S2 §2.4.1 L398"| T2
    T2 -->|"async flush<br/>per S7 §4 拍板"| T3
```

---

## 2. PostgreSQL Tier 3: 5 张表 W/T/M 严格分类 (per S7 §3.2 + 守门 #13 d)

```mermaid
erDiagram
    CHECKPOINTS ||--o{ CHECKPOINT_WRITES : "1:N (channel writes)"
    CHECKPOINTS ||--o{ CHECKPOINT_SUMMARIES : "1:N (summaries)"
    CHECKPOINTS ||--o{ CHECKPOINT_METADATA : "1:1 (SCD Type 2)"
    CHECKPOINTS ||--|| AUDIT_AUDIT_EVENT : "1:1 (WORM, per ADR-0043)"

    CHECKPOINTS {
        UUID id PK
        TEXT thread_id
        TEXT checkpoint_ns
        JSONB state
        UUID parent_id FK
        TEXT schema_version
        UUID tenant_id
        UUID workspace_id
        UUID actor_id
        TIMESTAMPTZ created_at
        UUID superseded_by NULL FK
        JSONB metadata
    }

    CHECKPOINT_WRITES {
        UUID id PK
        UUID checkpoint_id FK
        TEXT channel
        JSONB value
        UUID tenant_id
        UUID actor_id
        TIMESTAMPTZ created_at
    }

    CHECKPOINT_SUMMARIES {
        UUID id PK
        TEXT thread_id
        TEXT summary
        UUID checkpoint_id FK
        UUID tenant_id
        UUID actor_id
        TIMESTAMPTZ created_at
    }

    CHECKPOINT_METADATA {
        UUID id PK
        TEXT key UK
        JSONB value
        TIMESTAMPTZ valid_from
        TIMESTAMPTZ valid_to NULL
        UUID tenant_id
        INT version
        TIMESTAMPTZ created_at
    }

    AUDIT_AUDIT_EVENT {
        UUID id PK
        TEXT event_type
        UUID actor_id
        UUID tenant_id
        JSONB payload
        TIMESTAMPTZ created_at
    }
```

---

## 3. 5 张表 W/T/M 分类 (per S7 §3.2 + 守门 #13 d 派生约束)

| # | 表名 | 分类 | 物理删除 | 审计 | RLS | 触发 |
|---|---|---|---|---|---|---|
| **1** | `checkpoints` | **T** Transaction (append-only) | ❌ 禁止 | ✅ 必携 | 13 類必携 | per S7 §3.2 L78 |
| **2** | `checkpoint_writes` | **T** Transaction (append-only) | ❌ 禁止 | ✅ 必携 | 13 類必携 | per S7 §3.2 L79 |
| **3** | `checkpoint_summaries` | **T** Transaction (append-only) | ❌ 禁止 | ✅ 必携 | 13 類必携 | per S7 §3.2 L80 |
| **4** | `checkpoint_metadata` | **M** Master (SCD Type 2) | ❌ 禁止 | ✅ 必携 | 13 類必携 | per S7 §3.2 L81 |
| **5** | `audit_audit_event` | **T** Transaction (WORM) | ❌ 禁止 | ✅ 必携 (自身) | 13 類必携 | per S7 §3.2 L82 + ADR-0043 |

**派生守门 (per 守门 #13 d + S7 §3.2 L85-88)**:
- (a) T = 物理删除禁止 + 監査必須 + RLS 13 類必携 ✅
- (b) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 ✅ (`checkpoint_metadata` 用 `valid_from`/`valid_to`)
- (c) 100% 表覆盖, 0 表遗漏 ✅ (5/5 严格分类)
- (d) Master 100% RLS / Transaction 100% audit / (T 主键) 100% retention_period ✅

---

## 4. 12 Reducer 跨 Tier 整合 (per S7 §3.3 L92-107)

| Channel | Reducer | PostgreSQL 存储 | 备注 |
|---|---|---|---|
| `active_subagents` | `operator.add` (append) | `checkpoint_writes.channel='active_subagents'` 顺序 append | per S7 §3.3 L96 |
| `completed_subagents` | `operator.add` (append) | 同上 | per S7 §3.3 L97 |
| `conversation_history` | `operator.add` (append) | 同上 | per S7 §3.3 L98 |
| `intermediate_steps` | `operator.add` (append) | 同上 | per S7 §3.3 L99 |
| `global_context` | custom merge (LWW per key) | `checkpoint_metadata` SCD Type 2 (`key=global_context.{namespace}`) | per S7 §3.3 L100 |
| `last_response` | replace (last-write-wins) | `checkpoints.state.last_response` 字段 | per S7 §3.3 L101 |
| `interrupt_id` | replace | `checkpoints.state.interrupt_id` 字段 | per S7 §3.3 L102 |
| `task_relationships` | custom merge (DAG 边 union) | `checkpoint_metadata.key='task_relationships'` SCD Type 2 | per S7 §3.3 L103 |
| `superseded_tasks` | `operator.add` (append) | `checkpoint_writes.channel='superseded_tasks'` 顺序 append | per S7 §3.3 L104 |
| `bulk_operations` | queue (FIFO) | `checkpoint_writes.channel='bulk_operations'` 顺序 append | per S7 §3.3 L105 |
| `last_summarize_result` | replace | `checkpoint_summaries` 最新行 (per thread_id) | per S7 §3.3 L106 |
| `active_tmo_operation` | replace | `checkpoint_metadata.key='active_tmo_operation'` SCD Type 2 | per S7 §3.3 L107 |

---

## 5. TMO 7 节点跟 PostgreSQL 交互 (per S7 §3.4 L109-119)

```mermaid
flowchart LR
    subgraph TMO["TMO 7 节点 (per S2 §2.6)"]
        MN1["M-N1 merge"]
        MN2["M-N2 split"]
        MN3["M-N3 reorder"]
        MN4["M-N4 bulk"]
        MN5["M-N5 summarize"]
        MN6["M-N6 reassign"]
        MN7["M-N7 metadata"]
    end

    subgraph PGTables["PostgreSQL 5 张表 (per S7 §3.2)"]
        CKP["checkpoints<br/>state.last_response"]
        CKW["checkpoint_writes<br/>channel"]
        CKS["checkpoint_summaries<br/>新行"]
        CKM["checkpoint_metadata<br/>SCD Type 2 新版本"]
    end

    MN1 -->|"写新 + superseded_tasks append 旧"| CKP
    MN2 -->|"active_subagents append 新 sub-IDs"| CKW
    MN3 -->|"task_relationships SCD Type 2 新版"| CKM
    MN4 -->|"bulk_operations FIFO"| CKW
    MN5 -->|"summaries 新行"| CKS
    MN6 -->|"last_response 写新 + active_tmo_operation SCD Type 2"| CKP
    MN7 -->|"metadata 任意 key SCD Type 2 新版"| CKM
```

**整合原则 (per 守门 #13 a 强约束)**: TMO 7 节点全部 L0 协调, 跨 sub-agent 写共享 `checkpoints` 表需 RLS 校验, 防止 L1↔L1 直接写 (per S7 §3.4 L121).

---

## 6. Tier 3 启动条件 (per S7 §4 L152-164)

**PostgreSQL Tier 3 装装阶段启动 = 3 条件全满足**:

1. **5 域 Lead 真人至少 1 人到位** (T3 触发, per 5-business-domain-lead-referral.md §1.2 T3 = 2026-09-19 ~ 2026-09-26)
2. **R-05 push 反転确认** (8/30 07:09 JST 已落地, 不阻塞)
3. **设计阶段落地** (本 ADR + schema + migration + Tier 切换策略 + 5 域 Lead RACI 确认) → **当前状态 ✅**

**当前状态 (2026-09-05 10:58 JST)**:
- 设计阶段 = ✅ (本 ADR 落档)
- 5 域 Lead 真人 = ⏳ (T0 启动, T1 联系 1 周内, T3 至少 1 人到位 3 周内)
- R-05 push 反転 = ✅ (8/30 07:09 JST 已落地)

**预期启动时间**: 2026-09-26 JST (T3 最早) ~ 2026-10-17 JST (T4 最晚) 之间

---

## 7. 配置 & 部署 (per S7 §3.5 L123-132)

| 维度 | 内容 |
|---|---|
| **DB URL** | `$env:DATABASE_URL` 引用 (per 守门 #5 不打印), 格式 `postgresql://user:pass@host:5432/star_checkpoints` |
| **Schema migration** | `pg-migrate` 工具走 `./scripts/migrate-checkpoints.sh`, migration 文件 `migrations/checkpoints/V001__initial.sql` |
| **Connection pool** | `PgPool` (sqlx) + max 20 connections (per LangGraph PostgresSaver 推荐) |
| **★ Envoy fronting** | PostgreSQL 用 envoy 独立 deployment 模式 (per 9/1 13:05 JST 偏好), 不走 istio sidecar, 不走 nginx |
| **k3s 部署** | `tools/k3s/checkpointer-postgres.yaml` (类似 gm-console envoy deployment) |
| **TLS** | PostgreSQL mTLS 走 envoy termination, cert per cert-manager |

---

## 8. 备选方案拒绝理由 (per S7 §5 L168-194)

| 备选 | 优势 | 劣势 | 决策 |
|---|---|---|---|
| **A: CockroachDB (分布式 NewSQL)** | 全球分布式, 强一致, multi-region | 跟 LangGraph PostgresSaver 不直接兼容; 运维成本高 3x; Star 仓 single-region k3s 部署不需要 | ❌ 拒绝 (per S7 §5.1 L176) |
| **B: TiDB (分布式 NewSQL)** | MySQL 兼容, HTAP | LangGraph PostgresSaver 走 MySQL 协议需 adapter; TiDB schema 迁移复杂; 跟 9/1 13:03 JST envoy 偏好整合度低 | ❌ 拒绝 (per S7 §5.2 L184) |
| **C: 仅 SQLite Tier 2 升级 (不加 PostgreSQL)** | 简单, 不需 PG 运维 | 不能跨 session 续, 不能 5 域 Lead RACI 协调 | ❌ 拒绝 (per S7 §5.3 L192) |

---

## 已知缺口 (per 守门 #11)

- **G-1**: `migrations/checkpoints/V001__initial.sql` 具体 DDL 未落地 (per S7 §3.5 L128 引用, P3-D 实装)
- **G-2**: 13 類 RLS policy 详细 SQL 未列出 (per S7 §3.2 L78-82 引用, 待 5 域 Lead admin 域拍板)
- **G-3**: 5 域 Lead 真人未到位, RACI 暂以 Mavis 临时代签 (per S7 §3.6 L136-147)
- **G-4**: PostgreSQL Tier 3 实装未启动 (per S7 §4 启动条件 缺 1)
- **G-5**: Tier 切换策略 (Tier 1 → Tier 2 → Tier 3) 落地代码未拍板 (per S7 §3.1 L67 提及)
- **G-6**: Backup / Restore 详细方案未展开 (per S2 §2.4.3 L399 提及)
