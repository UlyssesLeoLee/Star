# ADR-0048: Star Multi-User & Multi-Agent 排他与幂等架构 view (Exclusion & Idempotency Architecture)

> **状态**: 🟢 Accepted v1.0 (per 2026-09-07 20:55 JST `ask_37d138ffb93a12279b35a46e` 4 推荐项拍板)
> **生效**: 2026-09-07
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 架构师 (Mavis 接手 agent per DEC-008) (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签" + 守门 #10 author=Ulysses)
> **母文档**: [Star LangGraph 多代理架构系统 详细设计 v0.2](../2026-09-03-langgraph/03-detailed-design.md) §1.1 M-08/M-09 + [Star Agent Runtime 详细设计 v0.1](../2026-09-03-agent-runtime/03-detailed-design.md) §3 Systems
> **关联**: [ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) · [ADR-0043 audit_audit_event WORM](0043-audit-onboarding-failed.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](0046-langgraph-task-management-operations.md) · [ADR-0047 PostgreSQL Checkpointer Tier 3](0047-postgresql-checkpointer-tier3.md) · [AGENTS.md §4 守门硬约束](../../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展开强约束](../../../AGENTS.md)
> **下游**: [01-requirements.md v1.0](../2026-09-07-exclusion-idempotency/01-requirements.md) (要件定義書) · [02-basic-design.md v1.0](../2026-09-07-exclusion-idempotency/02-basic-design.md) (基本設計書) · [03-detailed-design.md v1.0](../2026-09-07-exclusion-idempotency/03-detailed-design.md) (詳細設計書) · [PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1](../../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md) (8 子项实施计划)

---

## 1. 背景与问题

### 1.1 业务背景 (per 2026-09-07 20:45 JST 用户发令原话)

Ulysses 在 2026-09-07 20:45 JST 明确发令:

> **"多用户、多agent的排他和幂等设计要做到位, 专门制作一套架构view, 用于排他设计, 需求文档和基本设计以及详细设计按部就班制作出来"**

**关键 3 维**:
1. **多用户并发场景** — 同一 Star 租户内多用户同时操作 Kanban / worktree / agent (e.g. User A "暂停 task X" + User B "执行 task X" 同时发起)
2. **多 agent 并发场景** — L0 TopAgent 派发 N 个 L1 SubAgent 并行执行同一份数据, e.g. SA-01 + SA-04 同时改 `work_item` 同一行
3. **跨场景混合** — UI 客户端去重 + 服务端 lock + Domain 行级 fence 三层共存, 任何一层失守都导致重复执行 / 锁泄漏 / 数据错乱

### 1.2 现状缺口 (per 02-basic-design.md §7 已知缺口 + AGENTS.md §4 #13)

Star 现有架构 view (`2026-09-03-langgraph` + `2026-09-03-agent-runtime` + `2026-08-26-upgrade-plan`) 已落地:

| 已落地 (v0.x) | 缺口 (本 view 补) |
|---|---|
| ADR-0030 Agent Lease/Heartbeat/Resume (11 字段) | **多用户并发排他** — Lease 解决 agent 单实例, 未解决多用户竞争同一资源 |
| ADR-0032 MCP Transport stdio (16 tools + 6 字段错误模型) | **跨 agent 幂等** — 16 tool 各自重试, 未统一去重 |
| ADR-0046 LangGraph TMO 7 节点 (merge/split/reorder/bulk/summarize/reassign/metadata) | **TMO 操作原子性** — bulk_node 失败部分回滚 OK, 跨用户并发 TMO 操作无锁 |
| ADR-0047 PostgreSQL Checkpointer Tier 3 (5 表 schema) | **checkpoint vs lock 边界** — Tier 3 共享 PG, advisory lock 与 checkpoint 写入事务隔离 |
| AGENTS.md §4 #13 W/T/M 严格分类 | **idem/lease/lock 表归类** — 3 类新表 (idempotency_keys / lease_log / advisory_lock_audit) W/T/M 严格归类缺 |

**关键缺口**:
- **多用户并发**: 同一 tenant 内 User A + User B 同时点"暂停 task X", 当前无任何锁机制, race condition
- **多 agent 并发**: SA-01 写入 + SA-04 读取同一 `work_item` 行, 缺 PG row-level fence + version 校验
- **TMO 跨用户**: bulk_node 暂停 N 任务, 中途 User C 修改其中一个, 缺乐观锁
- **16 tool 幂等**: star-mcp 16 tool 重试无去重, 客户端 / 网络 / 服务端三处重试叠加可能产生副作用 (e.g. `create_worktree` 重复创建)
- **审计追溯**: 多用户多 agent 操作错乱后, 缺统一 `audit_audit_event` 记录谁 / 何时 / 持锁多久 / 是否正常释放

### 1.3 架构冲突 (守门 #13 a + #13 c 强约束)

per [AGENTS.md §4 #13 a](../../../AGENTS.md): **L1↔L1 禁止通信** (防止状态污染). 排他/幂等操作 (e.g. 多 SubAgent 抢同一资源) **必须经 L0 协调**, 不能 L1 直接互锁.

per [AGENTS.md §4 #13 c](../../../AGENTS.md): **Master 100% RLS 必携**. 新表 `idempotency_keys` / `lease_log` / `advisory_lock_audit` 必须带 RLS 13 类 + `tenant_id` / `workspace_id` 字段.

per [AGENTS.md §4 #13 d](../../../AGENTS.md): **Transaction 100% audit 必携**. 排他/幂等关键表 (idempotency_keys / lease_log) = Transaction (append-only), 锁获取/释放事件进 `audit_audit_event` (per ADR-0043 WORM).

---

## 2. 决策

**新建独立架构 view `2026-09-07-exclusion-idempotency/`, 跟 LangGraph / Agent Runtime view 平行, 专门处理"多用户 + 多 agent"竞争场景下的排他 (exclusion) 与幂等 (idempotency) 设计. 锁服务 = PostgreSQL advisory lock (跟 ADR-0047 PG checkpointer Tier 3 共享, 0 额外组件), 幂等键 = 双键 (client_uuid + business_hash) 兜底, 跨层架构 = 4 层 (UI / L0 / L1 / Domain 22 crate) 全栈覆盖.**

### 2.1 4 推荐项拍板 (per 2026-09-07 20:55 JST `ask_37d138ffb93a12279b35a46e`)

| # | 决策轴 | 拍板 | 理由 |
|---|---|---|---|
| **D-01** | 锁服务 | **PostgreSQL advisory lock** | 跟 ADR-0047 PG checkpointer Tier 3 共享, 0 额外组件, pg_try_advisory_lock + pg_advisory_xact_lock 适合事务内排他; 跨 cluster 限制可接受 (Star 部署在单 cluster) |
| **D-02** | 幂等键 | **双键 (client_uuid + business_hash)** | 优先用 client UUID, 缺失则 fallback business hash; 业务方零负担 (Stripe 模式) + 兜底强 (无 UUID 也能 dedup) |
| **D-03** | 跨层架构 | **4 层 (UI / L0 / L1 / Domain)** | 每层都有自己的排他/幂等职责: UI = 客户端 dedup / L0 = dispatch 去重 / L1 = task card 锁 / Domain = 业务行锁; 责任清晰, 跟守门 #13 a L0 协调一致 |
| **D-04** | view 命名 | **exclusion-idempotency** | 直接对应用户原话, 跟前两个 view 风格一致 (日期 + 主题) |

### 2.2 4 层责任矩阵 (per 02 §2.5)

| 层 | 排他职责 | 幂等职责 | 关键组件 | 协议 |
|---|---|---|---|---|
| **L0 UI** (gm-console) | 客户端 dedup 锁 (双键 + localStorage) | 客户端 in-flight 请求去重 (AbortController + key) | `IdempotencyManager` (browser) | `Idempotency-Key` header |
| **L1 TopAgent** (L0 派发层) | 派发级排他 (同一 task 拒绝并发 dispatch) | dispatch_id 幂等 (24h dedup 表) | `DispatchLockManager` + `idempotency_keys` 表 (T) | `X-Dispatch-Id` header |
| **L2 SubAgent** (L1 执行层) | task card 持锁 (lock_token + heartbeat) | sub-agent 操作幂等 (tool 调用 dedup) | `SubAgentLock` + `lease_log` 表 (T) | `X-Task-Lock-Token` |
| **L3 Domain** (22 crate 业务层) | 业务行锁 (PG advisory lock + version CAS) | 业务主键 hash 幂等 (写入校验) | `DomainMutex` + `pg_advisory_xact_lock` | `pg_try_advisory_lock(key)` |

### 2.3 5 张新表 (per 守门 #13 W/T/M 严格分类)

| # | 表名 | 分类 (per 守门 #13) | 物理删除 | 审计 | RLS 13 类 | 关键字段 |
|---|---|---|---|---|---|---|
| 1 | `idempotency_keys` | **T** Transaction (append-only) | 禁止 | 必携 | 13 类必携 | `id UUID PK` + `key TEXT UNIQUE` + `key_type TEXT` (client/business/dual) + `request_fingerprint TEXT` + `response_status INT` + `response_body JSONB` + `tenant_id UUID` + `workspace_id UUID` + `actor_id UUID` + `created_at TIMESTAMPTZ` + `expires_at TIMESTAMPTZ` |
| 2 | `lease_log` | **T** Transaction (append-only) | 禁止 | 必携 | 13 类必携 | `id UUID PK` + `lease_id UUID` + `task_id UUID` + `lock_token TEXT` + `heartbeat_at TIMESTAMPTZ` + `expires_at TIMESTAMPTZ` + `released_at TIMESTAMPTZ NULL` + `release_reason TEXT NULL` + `tenant_id UUID` + `actor_id UUID` + `created_at TIMESTAMPTZ` |
| 3 | `advisory_lock_audit` | **T** Transaction (append-only) | 禁止 | 必携 | 13 类必携 | `id UUID PK` + `lock_class_id BIGINT` + `lock_target TEXT` (e.g. `work_item:UUID`) + `acquired_at TIMESTAMPTZ` + `released_at TIMESTAMPTZ NULL` + `wait_ms INT` + `tenant_id UUID` + `actor_id UUID` + `created_at TIMESTAMPTZ` |
| 4 | `idempotency_keys_archive` | **T** Transaction (archive) | 禁止 | 必携 | 13 类必携 | 同 #1 + `archived_at TIMESTAMPTZ` (per 守门 #13 d 24h 后归档) |
| 5 | `exclusion_policy_master` | **M** Master (RLS 必携) | 禁止 | 必携 | 13 类必携 | `id UUID PK` + `policy_name TEXT UNIQUE` + `lock_strategy TEXT` (advisory/lease/optimistic) + `timeout_ms INT` + `retry_max INT` + `tenant_id UUID` + `is_active BOOLEAN` + `created_at TIMESTAMPTZ` + `updated_at TIMESTAMPTZ` |

**派生约束** (per 守门 #13 a + c + d):
- (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention
- (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携
- (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携
- (d) Master 100% RLS / Transaction 100% audit / Work 100% retention_period

---

## 3. 拒绝方案 (3 备选 + 理由)

### 3.1 锁服务备选 (D-01)

| 备选 | 拒绝理由 |
|---|---|
| Redis 分布式锁 (SETNX + Redlock) | 引入 Redis 单点 + 主从切换锁丢失风险; 跟 ADR-0047 PG checkpointer 不一致, 部署多 1 组件 |
| Etcd lease + lock + Watch | 强一致但运维负担重, 引入新组件; Star 部署在单 cluster, 不需要 Raft 共识 |
| 应用层 in-process 锁 | 单实例够用, 但 5 域 Lead RACI 多人协作 / 多 backend 实例时立即失效 |

### 3.2 幂等键备选 (D-02)

| 备选 | 拒绝理由 |
|---|---|
| 客户端 UUID 单键 (Stripe 纯模式) | 客户端漏带 UUID 立即失去幂等保护 (e.g. SDK bug / 重试 code 没传 key); 不够稳健 |
| 业务主键 hash 单键 | 业务侧 schema 紧耦合, 改 schema 风险大; e.g. work_item 主键由 UUID 改 (tenant, work_id) 复合, 所有 dedup 失效 |

### 3.3 跨层备选 (D-03)

| 备选 | 拒绝理由 |
|---|---|
| 3 层 (UI / L0 / L1+Domain 合并) | L1 SubAgent 跟 Domain 22 crate 业务合并, 改 Domain 范围广 (134+ 现有 file); 责任不清晰 |
| 2 层 (UI / Backend 全栈) | 跟守门 #13 a "L0 协调 L1↔L1" 冲突; 缺中间派发层导致多 agent 直接互锁, 违反架构原则 |

---

## 4. 后果

### 4.1 正面

1. **多用户并发安全**: 同一 tenant 内 User A + User B 同时操作, 通过 4 层锁链 (UI dedup → L0 dispatch lock → L1 task card lease → Domain row lock) 防止 race condition
2. **多 agent 协调**: L0 TopAgent 作为唯一协调者, 所有 L1↔L1 排他经 L0 (per 守门 #13 a 派生规)
3. **TMO 跨用户原子性**: bulk_node / merge_node / split_node 等 7 节点操作加 L0 dispatch lock + Domain advisory lock 双层保护
4. **16 tool 幂等**: star-mcp 16 tool 全部走 `idempotency_keys` 表 (T, 24h dedup), 客户端/网络/服务端三重试安全
5. **审计完整**: 所有锁获取/释放事件进 `audit_audit_event` (per ADR-0043 WORM), 多用户多 agent 错乱后可追溯

### 4.2 负面 / 风险

1. **PG 写入压力**: idempotency_keys / lease_log / advisory_lock_audit 三表高频写入, 跟 ADR-0047 PG checkpointer 共用同一 PG, 需容量规划
2. **跨 cluster 不支持**: 锁服务绑单 PG, 跨 cluster 部署需引入外部锁 (Etcd), 暂不考虑
3. **5 张新表运维**: 5 表 + 索引 + RLS 13 类 policy, schema 演进需 DDD Review 拍板
4. **3 层协议栈**: UI/L0/L1/Domain 4 层各持一种锁, 故障排查需跨层 trace, 需 unified observability

### 4.3 5 域 Lead RACI (per 守门 #14 v2)

| 域 | RACI | 拍板权 |
|---|---|---|
| player 域 | R+A (Mavis 临时代签) | Mavis 临时代签 (per 守门 #14 拍板 D 维持) |
| economy 域 | R+A (Mavis 临时代签) | 同上 |
| match 域 | R+A (Mavis 临时代签) | 同上 |
| social 域 | R+A (Mavis 临时代签) | 同上 |
| admin 域 | R+A (Mavis 临时代签) | 同上 (排他/幂等主要落在 admin 域) |

**说明**: 5 域 Lead 真人到位前 Mavis 临时代签维持 (per 守门 #14 v2 拍板 D), 真人到位后追溯签字覆盖.

---

## 5. 实施计划 (per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1)

8 子项 ~2.0-2.5M token 估, 走守门 #19 Python 化 + #9 v3 subprocess + #22 控制台不污染 main + #23 AI mock.

| # | 子项 | 内容 | token 估 | 状态 |
|---|---|---|---|---|
| **EX-01** | 5 张新表 DDL | idempotency_keys / lease_log / advisory_lock_audit / idempotency_keys_archive / exclusion_policy_master + RLS 13 类 | ~0.2M | 🟡 v0.1 plan |
| **EX-02** | PG advisory lock wrapper (Rust) | `crates/star-mutex/src/lib.rs` + `pg_try_advisory_lock` + `pg_advisory_xact_lock` 包装 | ~0.3M | 🟡 v0.1 plan |
| **EX-03** | L0 DispatchLockManager (Python) | `scripts/automation/exclusion/dispatch_lock.py` + dispatch_id dedup | ~0.3M | 🟡 v0.1 plan |
| **EX-04** | L1 SubAgentLock (Python) | `scripts/automation/exclusion/subagent_lock.py` + lease heartbeat | ~0.3M | 🟡 v0.1 plan |
| **EX-05** | UI IdempotencyManager (TypeScript) | `frontend/src/lib/exclusion/idempotency.ts` + localStorage + AbortController | ~0.2M | 🟡 v0.1 plan |
| **EX-06** | star-mcp 16 tool 幂等改造 | 16 tool 加 `Idempotency-Key` header 解析 + 写入 idempotency_keys 表 | ~0.4M | 🟡 v0.1 plan |
| **EX-07** | 4 层统一可观测性 | 锁 trace_id 贯穿 4 层 + 锁获取/释放 metric + 锁泄漏 alert | ~0.3M | 🟡 v0.1 plan |
| **EX-08** | 集成测试 + 性能压测 | 8 想定シナリオ E2E + 1000 并发用户压测 (PG advisory lock 极限 QPS) | ~0.3M | 🟡 v0.1 plan |
| **总** | 8 子项 |  | **~2.3M** | 🟡 v0.1 plan |

**触发条件**: 5 域 Lead 真人到位后 (per 守门 #14 v2 拍板 D), 排他/幂等 view 实施进入"装装"阶段.

---

## 6. 验证

### 6.1 守门 #1 验证 (4 项全 0 错)

- (1) `cargo check --workspace --all-targets` 0 err
- (2) `cargo fmt + clippy` 0 警告
- (3) `cargo test --workspace --release --lib` 100% pass
- (4) `cargo build --release + doc + bench --no-run` 全 0 错

### 6.2 守门 #13 验证 (W/T/M 严格 + RLS 13 类 + 审计必携)

5 张新表 + 100% RLS policy + 100% audit trigger + 0 缺标.

### 6.3 8 想定シナリオ E2E (per 03 §5 集成测试)

| # | シナリオ | 验证目标 | 优先级 |
|---|---|---|---|
| **S-01** | 同一用户重复点 Send 按钮 | 客户端 dedup, 1 次服务端写入 | P0 |
| **S-02** | 网络抖动服务端重试 | 双键 dedup, 1 次副作用 | P0 |
| **S-03** | User A + User B 同时"暂停 task X" | L0 dispatch lock + Domain advisory lock, 后到者收到 409 | P0 |
| **S-04** | SA-01 + SA-04 同时改 work_item | Domain advisory lock + version CAS, 后到者重试 | P0 |
| **S-05** | bulk_node 暂停 5 任务中途 User C 修改 1 个 | 乐观锁 + 部分回滚 + 通知 | P1 |
| **S-06** | L1 SubAgent 崩溃, lease 过期 | 锁自动释放 + 重试 1 次 | P0 |
| **S-07** | 客户端漏带 Idempotency-Key | 双键 fallback business_hash 兜底 | P1 |
| **S-08** | 1000 并发用户压测 advisory lock QPS | 验证 PG 极限 (估 5K-10K QPS) | P1 |

---

## 7. 签字

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿, 4 拍板项 + 4 层责任矩阵 + 5 张新表 schema + 8 子项实施计划 | per `ask_37d138ffb93a12279b35a46e` 4 推荐项拍板 |
| v1.0 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Accepted v1.0 拍板落地, 同步 01/02/03 IPA 3 文档 + PHASE report 落档 | 拍板 + IPA 三文档齐备 |
