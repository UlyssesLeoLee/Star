# 02. Star Multi-User & Multi-Agent 排他与幂等架构系统 - 基本设计书 (Basic Design)

> **状态**: 🟢 Accepted v1.0
> **生效**: 2026-09-07
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签")
> **关联**: [01-requirements.md v1.0](01-requirements.md) · [ADR-0048 排他与幂等架构 view 拍板](../2026-08-26-upgrade/adr/0048-exclusion-idempotency-design.md) · [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0043 audit_audit_event WORM](../2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [ADR-0047 PostgreSQL Checkpointer Tier 3](../2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展开强约束](../../AGENTS.md)
> **下游**: [01-requirements.md v1.0](01-requirements.md) · [03-detailed-design.md v1.0](03-detailed-design.md) · [PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

---

## 0. 目的 (Purpose)

本文档承接 [01-requirements.md](01-requirements.md) §2-§5 业务/功能/非功能需求 + 8 想定シナリオ, 对 Star 排他与幂等架构系统 (Star-EI) 进行基本设计.

- **系统架构** (4 层 + 5 张新表 + 4 选型 + 6 协议)
- **4 层责任矩阵** + 关键组件 (UI / L0 / L1 / Domain)
- **5 张新表 schema** (per 守门 #13 W/T/M 严格分类)
- **6 协议** (Idempotency-Key / X-Dispatch-Id / X-Task-Lock-Token / X-Trace-Id / PG advisory lock / lock 释放)
- **UI/UX 设计** (gm-console 客户端 dedup + 锁状态可视化)
- **安全/性能/可靠性/可观测性设计**

> **重要澄清 (per [01 §1.0](01-requirements.md))**: 本 view 设计的"排他/幂等"是 **跨 L0 派发层 / L1 SubAgent / Domain 22 crate / UI 4 层** 全栈方案, **不** 跟 LangGraph view 重复 (LangGraph 关注 L0 拓扑 + SubAgent 类型), **不** 跟 Agent Runtime view 重复 (Agent Runtime 关注 ECS Runtime + LLM/HTTP/MCP 池). 三个 view 是 **平行 + 互补** 关系.

## 1. 系统架构 (System Architecture)

### 1.1 整体架构图 (Overall Architecture)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ L0 UI Tier (gm-console frontend / Next.js)                              │
│ ┌─────────────────────────────────────────────────────────────────────┐ │
│ │ IdempotencyManager (browser)  ←→  localStorage (key cache)         │ │
│ │  ├─ 客户端 dedup 锁 (双键 + AbortController)                        │ │
│ │  ├─ 锁状态可视化 (4 层状态: 持有/等待/超时/释放)                     │ │
│ │  └─ 锁错乱 toast 提示                                              │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
│ Chat Bar → L0 TopAgent 派发                                             │
└─────────────────────────────────────────────────────────────────────────┘
    │ HTTP/WS (Idempotency-Key + X-Trace-Id header)
    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ L1 TopAgent Tier (派发层 / Python + LangGraph)                          │
│ ┌─────────────────────────────────────────────────────────────────────┐ │
│ │ DispatchLockManager (Python)                                        │ │
│ │  ├─ 派发级排他 (PG advisory lock, BIGINT, 60s TTL)                  │ │
│ │  ├─ 派发级幂等 (idempotency_keys 表, 24h dedup, 双键)               │ │
│ │  ├─ TMO 7 节点 (per ADR-0046) 串行化                                 │ │
│ │  └─ 锁链 trace_id 贯穿 (4 层共享)                                   │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
│ TMO 7 节点 → L1 SubAgent 派发                                           │
└─────────────────────────────────────────────────────────────────────────┘
    │ in-process asyncio (X-Task-Lock-Token + X-Trace-Id)
    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ L2 SubAgent Tier (执行层 / Python + 9 SA + SA-10)                       │
│ ┌─────────────────────────────────────────────────────────────────────┐ │
│ │ SubAgentLock (Python)                                               │ │
│ │  ├─ 任务卡持锁 (lease + heartbeat 30s, 过期 300s)                    │ │
│ │  ├─ SubAgent 操作幂等 (tool 调用 dedup)                              │ │
│ │  ├─ 锁自动恢复 (崩溃 30s 内自动释放)                                 │ │
│ │  └─ 锁泄漏告警 (24h 1h 阈值)                                        │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
│ 9 SA + SA-10 → Domain 22 crate 业务调用                                 │
└─────────────────────────────────────────────────────────────────────────┘
    │ gRPC/RPC (X-Trace-Id)
    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│ L3 Domain Tier (业务层 / Rust 22 domain-* crate + PG advisory)          │
│ ┌─────────────────────────────────────────────────────────────────────┐ │
│ │ DomainMutex (Rust)                                                  │ │
│ │  ├─ 业务行锁 (PG advisory_xact_lock, transaction-bound)              │ │
│ │  ├─ 业务主键 hash 幂等 (写入校验, business_hash 兜底)                │ │
│ │  ├─ 22 domain crate 接入 (统一 DomainMutex trait)                   │ │
│ │  └─ version CAS 乐观锁 (work_item 写入必带 version)                 │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
│ 22 domain crate → PostgreSQL (跟 ADR-0047 checkpointer Tier 3 共享)    │
└─────────────────────────────────────────────────────────────────────────┘
                                              │
                                              ▼
            ┌─────────────────────────────────────────────────┐
            │  PostgreSQL (跟 ADR-0047 checkpointer Tier 3 共享) │
            │  ┌─────────────────────────────────────────────┐ │
            │  │ 5 张新表:                                    │ │
            │  │  ├─ idempotency_keys (T, 24h TTL)            │ │
            │  │  ├─ lease_log (T, append-only)               │ │
            │  │  ├─ advisory_lock_audit (T, append-only)     │ │
            │  │  ├─ idempotency_keys_archive (T, 24h 后归档) │ │
            │  │  └─ exclusion_policy_master (M, SCD Type 2)  │ │
            │  │ RLS 13 类 + audit trigger 100% 必携         │ │
            │  └─────────────────────────────────────────────┘ │
            └─────────────────────────────────────────────────┘
```

### 1.2 4 层责任矩阵 (per 守门 #13 a + c + d 派生规)

| 层 | 排他职责 | 幂等职责 | 关键组件 | 协议 | W/T/M |
|---|---|---|---|---|---|
| **L0 UI** (gm-console) | 客户端 dedup 锁 (双键 + localStorage) | 客户端 in-flight 请求去重 (AbortController + key) | `IdempotencyManager` (browser) | `Idempotency-Key` header | (无 DB) |
| **L1 TopAgent** (派发层) | 派发级排他 (同一 task 拒绝并发 dispatch) | dispatch_id 幂等 (24h dedup 表) | `DispatchLockManager` + `idempotency_keys` (T) | `X-Dispatch-Id` header | T |
| **L2 SubAgent** (执行层) | task card 持锁 (lock_token + heartbeat) | sub-agent 操作幂等 (tool 调用 dedup) | `SubAgentLock` + `lease_log` (T) | `X-Task-Lock-Token` | T |
| **L3 Domain** (22 crate) | 业务行锁 (PG advisory lock + version CAS) | 业务主键 hash 幂等 (写入校验) | `DomainMutex` + `pg_advisory_xact_lock` | `pg_try_advisory_lock(key)` | T (audit) |

### 1.3 跨层调用关系

```
UI (IdempotencyManager) 
    → HTTP POST /api/dispatch  { Idempotency-Key, X-Trace-Id }
    → L0 TopAgent (DispatchLockManager) 
        → PG advisory lock (60s TTL) + idempotency_keys dedup
    → L1 SubAgent (SubAgentLock) 
        → lease acquire (lock_token) + heartbeat start
        → 调用 L3 Domain (DomainMutex) 
            → PG advisory_xact_lock (txn-bound) + version CAS
        → 释放 lease
    → 返回 UI
```

**关键约束** (per 守门 #13 a):
- L2 SubAgent **不能**直接调 L3 Domain 互抢锁, 必须经 L1 TopAgent 协调
- L0 UI **不能**绕过 L1 直接调 L2, 必须走 L0 派发链
- L3 Domain 锁是 transaction-bound, 事务结束自动释放

## 2. 选型表 (Selection Tables)

### 2.1 锁服务选型 (per ADR-0048 D-01)

| 备选 | 评估维度 | 选/不选 | 理由 |
|---|---|---|---|
| **PostgreSQL advisory lock** | 性能 / 一致性 / 部署 / 跟现有 | ✅ **选** | 跟 ADR-0047 checkpointer Tier 3 共享 PG, 0 额外组件; `pg_try_advisory_xact_lock` 适合事务内排他; 跨 cluster 不支持可接受 |
| Redis 分布式锁 (SETNX + Redlock) | 性能 / 一致性 / 部署 / 跟现有 | ❌ 不选 | 引入 Redis 单点 + 主从切换锁丢失风险; 跟 ADR-0047 不一致, 部署多 1 组件 |
| Etcd lease + lock + Watch | 性能 / 一致性 / 部署 / 跟现有 | ❌ 不选 | 强一致但运维负担重, 引入新组件; Star 部署在单 cluster, 不需要 Raft 共识 |
| 应用层 in-process 锁 | 性能 / 一致性 / 部署 / 跟现有 | ❌ 不选 | 单实例够用, 但 5 域 Lead RACI 多人协作 / 多 backend 实例时立即失效 |

### 2.2 幂等键选型 (per ADR-0048 D-02)

| 备选 | 评估维度 | 选/不选 | 理由 |
|---|---|---|---|
| **双键 (client_uuid + business_hash)** | 兜底 / 业务方负担 / schema 耦合 | ✅ **选** | 优先 client UUID, 缺失 fallback business hash; 业务方零负担 (Stripe 模式) + 兜底强 |
| 客户端 UUID 单键 (Stripe 纯模式) | 兜底 / 业务方负担 / schema 耦合 | ❌ 不选 | 客户端漏带 UUID 立即失去幂等保护; 不够稳健 |
| 业务主键 hash 单键 | 兜底 / 业务方负担 / schema 耦合 | ❌ 不选 | 业务侧 schema 紧耦合, 改 schema 风险大 |

### 2.3 跨层架构选型 (per ADR-0048 D-03)

| 备选 | 评估维度 | 选/不选 | 理由 |
|---|---|---|---|
| **4 层 (UI / L0 / L1 / Domain)** | 责任清晰 / 改造范围 / 守门一致 | ✅ **选** | 每层都有自己的排他/幂等职责, 跟守门 #13 a L0 协调一致 |
| 3 层 (UI / L0 / L1+Domain 合并) | 责任清晰 / 改造范围 / 守门一致 | ❌ 不选 | L1 跟 Domain 22 crate 业务合并, 改 Domain 范围广 (134+ 现有 file); 责任不清晰 |
| 2 层 (UI / Backend 全栈) | 责任清晰 / 改造范围 / 守门一致 | ❌ 不选 | 跟守门 #13 a "L0 协调 L1↔L1" 冲突 |

### 2.4 锁类型选型 (per 4 层)

| 层 | 锁类型 | 选型 | 锁粒度 | 锁超时 | 释放方式 |
|---|---|---|---|---|---|
| L0 UI | 客户端 dedup 锁 | Map + AbortController | request-level | 30s (页面内) | 自动 |
| L1 TopAgent | 派发级排他 | PG advisory lock (60s) | task-level | 60s | 自动 (TTL) |
| L2 SubAgent | 任务卡持锁 | lease + heartbeat | task-card | 300s | 显式 + 自动 (heartbeat 超时) |
| L3 Domain | 业务行锁 | PG advisory_xact_lock + version CAS | row-level | transaction-bound | 事务结束自动释放 |

## 3. 5 张新表 schema (per 守门 #13 W/T/M 严格分类)

### 3.1 表分类总览

| # | 表名 | 分类 | 物理删除 | 审计 | RLS 13 类 | retention |
|---|---|---|---|---|---|---|
| 1 | `idempotency_keys` | **T** Transaction (append-only) | 禁止 | 必携 | 13 类必携 | 24h 后归档 |
| 2 | `lease_log` | **T** Transaction (append-only) | 禁止 | 必携 | 13 类必携 | append-only 永久 |
| 3 | `advisory_lock_audit` | **T** Transaction (append-only) | 禁止 | 必携 | 13 类必携 | append-only 永久 |
| 4 | `idempotency_keys_archive` | **T** Transaction (archive) | 禁止 | 必携 | 13 类必携 | append-only 永久 |
| 5 | `exclusion_policy_master` | **M** Master (RLS 必携) | 禁止 | 必携 | 13 类必携 | SCD Type 2 永久 |

### 3.2 idempotency_keys (T)

```sql
CREATE TABLE idempotency_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL,                          -- 客户端 UUID 或 business_hash
    key_type TEXT NOT NULL,                     -- client / business / dual
    request_fingerprint TEXT NOT NULL,          -- request body hash, 防 key 复用但内容不同
    response_status INT,                        -- 首次响应状态码
    response_body JSONB,                        -- 首次响应体 (用于重放)
    tenant_id UUID NOT NULL,                    -- 租户 (RLS 必携)
    workspace_id UUID NOT NULL,                 -- workspace (RLS 必携)
    actor_id UUID NOT NULL,                    -- user/agent
    trace_id UUID NOT NULL,                    -- 跨层 trace
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,           -- 24h 后过期
    UNIQUE (key, tenant_id)                     -- 同一租户内 key 唯一
);

CREATE INDEX idx_idem_keys_tenant ON idempotency_keys(tenant_id, expires_at);
CREATE INDEX idx_idem_keys_actor ON idempotency_keys(actor_id, created_at DESC);
CREATE INDEX idx_idem_keys_trace ON idempotency_keys(trace_id);

-- 审计 trigger (per ADR-0043 WORM)
CREATE TRIGGER trg_idem_keys_audit
AFTER INSERT OR UPDATE ON idempotency_keys
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类 policy (per 守门 #13 c)
ALTER TABLE idempotency_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY idem_keys_tenant_isolation ON idempotency_keys
    USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

### 3.3 lease_log (T)

```sql
CREATE TABLE lease_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lease_id UUID NOT NULL,
    task_id UUID NOT NULL,
    lock_token TEXT NOT NULL,
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,            -- 300s
    released_at TIMESTAMPTZ,
    release_reason TEXT,                        -- normal / timeout / error / forced
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lease_log_lease ON lease_log(lease_id, heartbeat_at DESC);
CREATE INDEX idx_lease_log_task ON lease_log(task_id, released_at);
CREATE INDEX idx_lease_log_actor ON lease_log(actor_id, created_at DESC);
CREATE INDEX idx_lease_log_expires ON lease_log(expires_at) WHERE released_at IS NULL;

-- 审计 trigger
CREATE TRIGGER trg_lease_log_audit
AFTER INSERT OR UPDATE ON lease_log
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类
ALTER TABLE lease_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY lease_log_tenant_isolation ON lease_log
    USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

### 3.4 advisory_lock_audit (T)

```sql
CREATE TABLE advisory_lock_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lock_class_id BIGINT NOT NULL,              -- PG advisory lock class
    lock_target TEXT NOT NULL,                  -- e.g. "work_item:UUID"
    acquired_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    wait_ms INT NOT NULL,
    hold_ms INT,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lock_audit_target ON advisory_lock_audit(lock_target, acquired_at DESC);
CREATE INDEX idx_lock_audit_actor ON advisory_lock_audit(actor_id, created_at DESC);
CREATE INDEX idx_lock_audit_trace ON advisory_lock_audit(trace_id);
CREATE INDEX idx_lock_audit_hold ON advisory_lock_audit(hold_ms DESC) WHERE hold_ms IS NOT NULL;

-- 审计 trigger
CREATE TRIGGER trg_lock_audit_audit
AFTER INSERT OR UPDATE ON advisory_lock_audit
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类
ALTER TABLE advisory_lock_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY lock_audit_tenant_isolation ON advisory_lock_audit
    USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

### 3.5 idempotency_keys_archive (T)

同 `idempotency_keys` schema + `archived_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` 字段; 由 24h cron job 从 `idempotency_keys` 移到此处.

### 3.6 exclusion_policy_master (M, SCD Type 2)

```sql
CREATE TABLE exclusion_policy_master (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_name TEXT NOT NULL,
    lock_strategy TEXT NOT NULL,                -- advisory / lease / optimistic
    timeout_ms INT NOT NULL,
    retry_max INT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- SCD Type 2 起点
    valid_to TIMESTAMPTZ,                            -- SCD Type 2 终点 (NULL = 当前)
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (policy_name, valid_from)
);

CREATE INDEX idx_excl_policy_active ON exclusion_policy_master(policy_name) WHERE valid_to IS NULL;
CREATE INDEX idx_excl_policy_tenant ON exclusion_policy_master(tenant_id);

-- SCD Type 2 trigger: 更新时关闭旧记录
CREATE TRIGGER trg_excl_policy_scd2
BEFORE UPDATE ON exclusion_policy_master
FOR EACH ROW EXECUTE FUNCTION scd_type2_close();

-- 审计 trigger
CREATE TRIGGER trg_excl_policy_audit
AFTER INSERT OR UPDATE ON exclusion_policy_master
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类
ALTER TABLE exclusion_policy_master ENABLE ROW LEVEL SECURITY;
CREATE POLICY excl_policy_tenant_isolation ON exclusion_policy_master
    USING (tenant_id = current_setting('app.tenant_id')::UUID);
```

## 4. 6 协议 (Protocols)

### 4.1 Idempotency-Key header (UI → L0 → L1)

```
HTTP/1.1 POST /api/dispatch
Idempotency-Key: 7f3e4b2a-1234-5678-9abc-def012345678
X-Trace-Id: trace-uuid
X-Tenant-Id: tenant-uuid
Content-Type: application/json
```

**处理规则**:
- UI 必带, 缺失则 400 拒绝
- 服务端写入 `idempotency_keys` 表, 24h 内同 key 命中返回首次响应
- 请求 fingerprint 不匹配则 422 拒绝 (防 key 复用但内容不同)

### 4.2 X-Dispatch-Id header (L0 → L1)

```
HTTP/1.1 POST /api/l1/dispatch
X-Dispatch-Id: dispatch-uuid
X-Trace-Id: trace-uuid
X-Task-Lock-Token: lock-token-text
```

**处理规则**:
- L0 派发时生成 UUID, 24h dedup
- 写入 `idempotency_keys` 表 (key_type=client)
- 派发级排他通过 `pg_try_advisory_lock(lock_class_id)` 实现, 60s TTL

### 4.3 X-Task-Lock-Token (L1 SubAgent 持锁)

```
HTTP/1.1 POST /api/l2/task/{task_id}/acquire
X-Trace-Id: trace-uuid
```

**响应**:

```json
{
  "lock_token": "lease-uuid",
  "expires_at": "2026-09-07T21:00:00Z",
  "heartbeat_interval_ms": 30000
}
```

**处理规则**:
- 持锁者每 30s 发心跳续约
- 超过 300s 无心跳自动释放 + 告警
- 显式释放时校验 lock_token, 不匹配拒绝

### 4.4 X-Trace-Id (4 层共享)

```
HTTP/1.1 GET /api/...
X-Trace-Id: trace-uuid
```

**处理规则**:
- UI 生成 UUID, 4 层传递
- 所有日志 / metric / audit 必带 trace_id
- 锁错乱可端到端追踪 (UI 端 → L0 → L1 → Domain → PG)

### 4.5 PG advisory lock 调用约定 (L0 / L1 / L3)

```sql
-- 获取 (60s TTL, 5s 等待)
SELECT pg_try_advisory_xact_lock(lock_class_id);  -- transaction-bound, 自动释放

-- 锁类 ID 计算
lock_class_id = hashtextextended('work_item:' || task_id::TEXT, 0);
-- hashtextextended 返回 BIGINT, 范围 -2^63..2^63-1
```

**约束**:
- L3 业务行锁必须用 `pg_advisory_xact_lock` (transaction-bound), 不用 `pg_advisory_lock` (session-bound, 需手动释放)
- L0 / L1 用 `pg_try_advisory_lock` (带超时), 不用 `pg_advisory_lock` (阻塞)
- 锁类 ID 命名空间按资源类型分: `work_item:` / `task:` / `worktree:` / `merge_request:` / `comment:`

### 4.6 lock 释放协议 (L1 / L3)

**L1 SubAgent 显式释放**:

```
HTTP/1.1 DELETE /api/l2/task/{task_id}/release
X-Task-Lock-Token: lock-token-text
```

**L3 Domain 隐式释放**: 事务结束自动释放 (`pg_advisory_xact_lock`)

**释放规则**:
- 显式释放校验 lock_token, 写入 `lease_log.released_at` + `release_reason`
- 自动过期 (heartbeat 超时) 写 `release_reason=timeout`
- 强制释放 (admin 域) 写 `release_reason=forced`

## 5. 组件设计 (Component Design)

### 5.1 组件清单 (per 4 层 + 跨层 + 5 张新表)

| # | 组件 | 层 | 语言 | 路径 | 职责 |
|---|---|---|---|---|---|
| **C-01** | `IdempotencyManager` | L0 UI | TypeScript | `frontend/src/lib/exclusion/idempotency.ts` | 客户端 dedup + localStorage + AbortController |
| **C-02** | `LockStatusToast` | L0 UI | TypeScript | `frontend/src/components/exclusion/LockStatusToast.tsx` | 锁状态可视化 toast |
| **C-03** | `DispatchLockManager` | L1 TopAgent | Python | `scripts/automation/exclusion/dispatch_lock.py` | 派发级排他 + 派发级幂等 |
| **C-04** | `IdempotencyKeyStore` | L1 TopAgent | Python | `scripts/automation/exclusion/idempotency_key_store.py` | idempotency_keys 表读写 |
| **C-05** | `SubAgentLock` | L2 SubAgent | Python | `scripts/automation/exclusion/subagent_lock.py` | 任务卡持锁 + heartbeat |
| **C-06** | `LockWatcher` | L2 SubAgent | Python | `scripts/automation/exclusion/lock_watcher.py` | lease 过期检测 + 告警 |
| **C-07** | `DomainMutex` | L3 Domain | Rust | `crates/star-mutex/src/lib.rs` | 业务行锁 + version CAS |
| **C-08** | `StarMutexAdapter` | L3 Domain | Rust | `crates/star-mutex/src/adapter.rs` | 22 domain crate 接入适配 |
| **C-09** | `LockAuditLogger` | L4 Cross | Rust | `crates/star-mutex/src/audit.rs` | 锁事件写 advisory_lock_audit |
| **C-10** | `ExclusionPolicyLoader` | L4 Cross | Rust | `crates/star-mutex/src/policy_loader.rs` | exclusion_policy_master 加载 |
| **C-11** | `TraceIdPropagator` | L4 Cross | Rust | `crates/star-mutex/src/trace.rs` | 跨层 trace_id 传递 |
| **C-12** | `star-mcp-idem-middleware` | L1 MCP | Rust | `crates/star-mcp/src/middleware/idempotency.rs` | 16 tool 幂等中间件 |
| **C-13** | `LockMetricExporter` | L4 Obs | Python | `scripts/automation/exclusion/lock_metric_exporter.py` | Prometheus 5 指标 |
| **C-14** | `LockLeakAlerter` | L4 Obs | Python | `scripts/automation/exclusion/lock_leak_alerter.py` | 锁泄漏告警 (24h 1h 阈值) |
| **C-15** | `ArchiveCronJob` | L4 Ops | Python | `scripts/automation/exclusion/archive_cron.py` | idempotency_keys 24h 归档 |
| **C-16** | `exclusion_rls_migration` | L4 DB | SQL | `docs/migrations/2026-09-07-exclusion-rls.sql` | 5 张新表 DDL + RLS 13 类 |
| **C-17** | `audit_trigger_setup` | L4 DB | SQL | `docs/migrations/2026-09-07-audit-trigger.sql` | audit_audit_event trigger 挂 5 表 |
| **C-18** | `exclusion_debug_console` | L4 Debug | Python | `scripts/automation/console_server.py` 扩展 | 调试控制台 (per 守门 #22) |

**18 组件**, 跟 02 排他/幂等 view 范围匹配.

### 5.2 关键组件接口 (per 03 §1 module 详化)

#### 5.2.1 IdempotencyManager (C-01)

```typescript
interface IdempotencyManager {
  // 生成新 key (UUID v4)
  generateKey(): string;
  
  // dispatch + 自动 dedup
  dispatch<T>(key: string, fn: () => Promise<T>, opts?: { 
    fallbackBusinessHash?: string,  // 双键 fallback
    timeoutMs?: number,             // 默认 30000
  }): Promise<T>;
  
  // 显式取消 in-flight
  abort(key: string): void;
  
  // localStorage 缓存 key
  cacheKey(key: string, ttlMs: number): void;
  getCachedKey(key: string): string | null;
}
```

#### 5.2.2 DispatchLockManager (C-03)

```python
class DispatchLockManager:
    async def acquire(
        self,
        task_id: UUID,
        dispatch_id: UUID,
        actor_id: UUID,
        tenant_id: UUID,
        trace_id: UUID,
        timeout_ms: int = 60000,
    ) -> LockGuard: ...
    
    async def release(self, lock: LockGuard) -> None: ...
    
    async def is_locked(self, task_id: UUID) -> bool: ...
```

#### 5.2.3 SubAgentLock (C-05)

```python
class SubAgentLock:
    async def acquire(
        self,
        task_id: UUID,
        subagent_id: str,  # SA-01..SA-09 + SA-10
        actor_id: UUID,
        trace_id: UUID,
        heartbeat_interval_s: int = 30,
        expires_s: int = 300,
    ) -> LeaseHandle: ...
    
    async def heartbeat(self, lease: LeaseHandle) -> None: ...
    
    async def release(self, lease: LeaseHandle, reason: ReleaseReason) -> None: ...
```

#### 5.2.4 DomainMutex (C-07)

```rust
#[async_trait]
pub trait DomainMutex: Send + Sync {
    async fn try_lock_with_timeout(
        &self,
        resource: &str,         // e.g. "work_item:UUID"
        timeout_ms: u64,
    ) -> Result<MutexGuard>;
    
    async fn version_cas<F, T>(
        &self,
        resource: &str,
        expected_version: u64,
        f: F,
    ) -> Result<T>
    where F: FnOnce() -> BoxFuture<T>;
}
```

## 6. UI/UX 设计 (UI/UX Design)

### 6.1 客户端 dedup 状态 (per F-06)

gm-console frontend 在以下场景启用 `IdempotencyManager`:

| 场景 | dedup 行为 | UI 反馈 |
|---|---|---|
| 用户点 Send 按钮 | AbortController 取消 in-flight | 按钮 disabled + spinner |
| 用户快速切换 tab | 取消旧 fetch | 旧 tab 取消 loading |
| 网络抖动重试 | 同一 key 重试, 命中 dedup | toast: "已重试, 结果一致" |
| 漏带 key (老 SDK) | 400 拒绝 + 提示 | error toast: "客户端需升级" |

### 6.2 锁状态可视化 (per F-12 + F-13)

AppShell 顶部新增 "锁状态" 角标:

```
[🔒 3 持锁 | 1 等待 | 0 超时]   ← 4 层锁链聚合
```

| 状态 | 颜色 | 含义 |
|---|---|---|
| 持锁 | 🟢 绿 | 锁正常持有 |
| 等待 | 🟡 黄 | 排队等待获取 |
| 超时 | 🔴 红 | 锁超时, 触发升级 |
| 释放 | ⚪ 灰 | 锁已释放 |

点击角标展开详情:

```
L0 UI   持锁 0  等待 0  超时 0
L1 TopAgent  持锁 1 等待 0  超时 0
L2 SubAgent  持锁 2 等待 1  超时 0
L3 Domain    持锁 0 等待 0  超时 0
```

每行可点击 → 跳转到 `trace_id` 日志.

### 6.3 锁错乱 toast (per F-14)

```typescript
// 用户收到 409 时的 UI 提示
{
  type: "warning",
  title: "task X 正在被 User A 操作",
  body: "请稍后重试, 或联系 User A 协调",
  actions: [
    { label: "重试", action: "retry" },
    { label: "取消", action: "cancel" },
  ],
}
```

## 7. 安全/性能/可靠性/可观测性设计

### 7.1 安全设计 (per 守门 #13 c + NFR-SEC)

| 维度 | 设计 |
|---|---|
| **跨 tenant 隔离** | 5 张新表 100% RLS 13 类 policy 必携, `tenant_id = current_setting('app.tenant_id')::UUID` |
| **lock_token 不可猜** | 256-bit random UUID v4, 不暴露 |
| **审计 WORM** | 5 张新表全部挂 `audit_audit_event` trigger (per ADR-0043) |
| **key 复用防护** | request_fingerprint 校验, key 复用但内容不同 → 422 拒绝 |
| **强制释放审计** | admin 域强制释放写入 `release_reason=forced` + audit |

### 7.2 性能设计 (per NFR-PERF)

| 维度 | 设计 |
|---|---|
| **PG advisory lock 性能** | `pg_try_advisory_xact_lock` (5s timeout), 不用阻塞版; 锁类 ID 用 `hashtextextended` 计算 |
| **idempotency_keys 索引** | `(tenant_id, expires_at)` / `(actor_id, created_at DESC)` / `(trace_id)` 3 索引 |
| **lease_log 索引** | `(lease_id, heartbeat_at DESC)` / `(task_id, released_at)` / `(expires_at) WHERE released_at IS NULL` 部分索引 |
| **advisory_lock_audit 索引** | `(lock_target, acquired_at DESC)` / `(hold_ms DESC) WHERE hold_ms IS NOT NULL` 部分索引 |
| **24h 归档** | cron job 把 `idempotency_keys` 中 `expires_at < NOW()` 移到 `idempotency_keys_archive`, 减轻主表压力 |

### 7.3 可靠性设计 (per NFR-REL)

| 维度 | 设计 |
|---|---|
| **锁不泄漏** | lease heartbeat 30s + 过期 300s, 自动释放 |
| **崩溃恢复** | L1 SubAgent 崩溃 → 30s 内 L0 检测 + 重派 |
| **PG 故障** | 锁获取失败重试 3 次 (1s/2s/4s 退避), 仍失败 → 5xx + 告警 |
| **幂等命中率** | 客户端 SDK 集成 + 业务方培训, 目标 > 99% |
| **审计完整性** | chaos test 验证 100% 锁事件落 audit |

### 7.4 可观测性设计 (per NFR-OBS)

| 维度 | 设计 |
|---|---|
| **trace_id 4 层贯穿** | UI 生成 UUID, HTTP header 传递, 4 层日志/metric/audit 必带 |
| **5 锁 metric** | `star_lock_acquired_total` (counter, 按 tier/result) / `star_lock_wait_seconds` (histogram) / `star_lock_hold_seconds` (histogram) / `star_lock_timeout_total` (counter, 按 tier) / `star_idempotency_dedup_total` (counter, 按 key_type) |
| **锁错乱定位** | 1 click 看到 4 层日志, 通过 `trace_id` 聚合 |
| **Grafana dashboard** | 5 锁 metric + 4 层持锁数 + 锁泄漏告警 |
| **Slack 告警** | 锁泄漏 24h 1h 阈值触发 + 强制释放审计 |

## 8. 部署 / 运维设计 (Deployment / Operations)

### 8.1 部署架构

- **PG 单 cluster** — 5 张新表 + ADR-0047 checkpointer Tier 3 共享同一 PG
- **3 进程** — L0 TopAgent (Python) + L1 SubAgent (Python) + L3 Domain (Rust, 22 crate)
- **1 进程** — gm-console frontend (Next.js)
- **1 cron** — ArchiveCronJob 24h 跑一次

### 8.2 容量规划

| 资源 | 基线 (1000 并发用户 30 天) | 备注 |
|---|---|---|
| `idempotency_keys` | 1000 user × 100 req/day × 30 day = 3M 行 | 24h 归档后主表 100K 行 |
| `lease_log` | 1000 user × 50 task/day × 30 day = 1.5M 行 | append-only, 季度归档 |
| `advisory_lock_audit` | 5000 锁/s × 86400 × 30 day = 13B 行 | append-only, 季度归档 |
| `exclusion_policy_master` | 100 行 (SCD Type 2 后 ~500) | 永久 |
| `idempotency_keys_archive` | 3M 行 / 30 day | 季度归档 |

### 8.3 监控 + 告警 + 故障恢复

- **监控**: 5 锁 metric + 4 层持锁数 + 锁泄漏告警
- **告警**: 锁泄漏 24h 1h 阈值 / 锁 QPS < 1K / 锁获取失败 > 5% / audit 写入失败
- **故障恢复**: PG 故障 → 重试 3 次 → 5xx + 告警 + SRE 介入; lease 崩溃 → 30s 内 L0 重派; admin 强制释放 → 审计 + 通知 owner

## 9. 跟现有架构 view 的关系

### 9.1 跟 LangGraph view (per ADR-0046) 关系

- **LangGraph view** = L0 拓扑 + L1 9 SA + TMO 7 节点
- **本 view (Star-EI)** = 4 层排他/幂等 (UI / L0 / L1 / Domain) + 5 张新表 + 6 协议
- **关系**: Star-EI 复用 LangGraph 的 L0/L1 分层, 但 Star-EI 多了 UI 客户端层 + Domain 业务行锁层

### 9.2 跟 Agent Runtime view (per ADR-0045) 关系

- **Agent Runtime view** = Hybrid Runtime (Lightweight + ECS + Event Driven) + LLM/HTTP/MCP 池
- **本 view (Star-EI)** = 4 层排他/幂等
- **关系**: Star-EI 不依赖 Agent Runtime, 但 16 tool 幂等改造 (`star-mcp-idem-middleware` C-12) 跟 Agent Runtime 共享 LLM/HTTP/MCP 池调用

### 9.3 跟 ADR-0047 PG Checkpointer Tier 3 关系

- **ADR-0047** = 5 表 schema (checkpoints / checkpoint_writes / ...) + PostgresCheckpointer wrapper
- **本 view (Star-EI)** = 5 张新表 + 4 层锁链
- **关系**: 共享同一 PG, 锁服务 `pg_try_advisory_xact_lock` 跟 checkpointer 事务隔离 (锁在事务内, checkpointer 跨事务)

### 9.4 跟 ADR-0030 Lease/Heartbeat/Resume 关系

- **ADR-0030** = 11 字段 Agent Lease (per-session lease + heartbeat)
- **本 view (Star-EI)** = 任务卡持锁 (per-task lease + heartbeat)
- **关系**: Star-EI 的 `lease_log` 表 + `SubAgentLock` 跟 ADR-0030 共享 lease 语义, 但 Star-EI 多了 4 层责任矩阵 + 5 张新表

### 9.5 跟守门 #13 W/T/M 关系

- **守门 #13** = DB 表必须横展 W/T/M 严格分类
- **本 view (Star-EI)** = 5 张新表 100% 覆盖 (3 Transaction + 1 Master + 1 Transaction archive)
- **派生规**: 5 表全部 RLS 13 类 + audit trigger 必携

## 10. 已知缺口 (Known Gaps, per 守门 #11 缺标比错标安全)

| # | 缺口 | 影响 | 拍板需求 |
|---|---|---|---|
| **G-02-EI** | 5 域 Lead 真人未到位, 排他/幂等实施进入"装装"阶段触发条件需 DDD Review Lead 拍板 | 中 | DDD Review Lead 真人到位后 |
| **G-03-EI** | PG 跨 cluster 锁不支持, 后续如需扩展需引入 Etcd 备选 | 中 | 跨 cluster 部署需求出现时 |
| **G-04-EI** | 锁 schema 演进走 DDD Review 拍板, 5 张新表 DDL 需 PG 容量规划 | 中 | 实施阶段 |
| **G-05-EI** | 双键 dedup 业务主键 hash 算法的版本兼容性 (e.g. work_item 主键从 UUID 改 (tenant, work_id) 复合) 需 DDD Review 拍板 | 中 | 业务 schema 演进时 |
| **G-06-EI** | 锁泄漏告警阈值 (24h 1h) 需 SRE Lead 拍板 | 低 | 部署阶段 |
| **G-07-EI** | 锁 metric 5 项 + Grafana dashboard 需 SRE Lead 评审 | 低 | 部署阶段 |
| **G-08-EI** | 16 tool 幂等改造涉及 5 域 Lead RACI 责任分配 (哪个域负责哪个 tool) | 中 | 5 域 Lead 到位后 |

## 11. 签字

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿, 18 组件 + 4 选型 + 5 表 schema + 6 协议 | per `ask_37d138ffb93a12279b35a46e` 4 推荐项拍板 |
| v1.0 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Accepted v1.0 拍板落地, 同步 01/03 IPA 文档 + PHASE report 落档 | 拍板 + 01/03 IPA 文档齐备 |
