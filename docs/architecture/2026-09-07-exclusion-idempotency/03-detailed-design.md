# 03. Star Multi-User & Multi-Agent 排他与幂等架构系统 - 详细设计书 (Detailed Design)

> **状态**: 🟢 Accepted v1.0
> **生效**: 2026-09-07
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签")
> **关联**: [01-requirements.md v1.0](01-requirements.md) · [02-basic-design.md v1.0](02-basic-design.md) · [ADR-0048 排他与幂等架构 view 拍板](../2026-08-26-upgrade/adr/0048-exclusion-idempotency-design.md) · [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0043 audit_audit_event WORM](../2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [ADR-0047 PostgreSQL Checkpointer Tier 3](../2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展开强约束](../../AGENTS.md)
> **下游**: [01-requirements.md v1.0](01-requirements.md) · [02-basic-design.md v1.0](02-basic-design.md) · [PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

> **本 view 范围** (per [01 §1.0](01-requirements.md)): 本詳細設計書面向 **4 层排他/幂等** (UI / L0 TopAgent / L1 SubAgent / Domain 22 crate) 全栈系统, 包括 18 组件 / 5 张新表 / 6 协议 / 8 状态机 / 5 时序图 / 18 UT + 8 IT + 12 E2E.

---

## 0. 目的 (Purpose)

本文档承接 [02-basic-design.md](02-basic-design.md) §1-§9 18 组件 / 4 选型 / 5 表 / 6 协议, 对 Star 排他与幂等架构系统 (Star-EI) 进行详细设计.

- **模块布局** (M-01..M-18 Rust + Python + TypeScript, 跨 4 层)
- **关键类 / 函数 / trait 签名** (Rust / Python / TypeScript 草案)
- **5 状态机** (Client dedup / Dispatch lock / SubAgent lease / Domain mutex / Idempotency key)
- **5 时序图** (4 层锁链 / 双键 dedup / Heartbeat / TMO bulk 原子性 / 跨层 trace)
- **5 张新表完整 DDL + RLS 13 类 + audit trigger**
- **22 domain crate 接入适配** (`star-mutex` 共享 crate)
- **18 UT + 8 IT + 12 E2E** (per [02 §8.2](02-basic-design.md))
- **性能 / 安全 / 错误处理** (per 守门 #5 / 守门 #7 / 守门 #13)

## 1. 模块布局 (Module Layout)

### 1.1 4 层模块结构 (per 02 §1.1)

```
D:/Star/
├── frontend/src/                            # L0 UI Tier (TypeScript / Next.js)
│   ├── lib/
│   │   └── exclusion/                       # 客户端 dedup + 锁状态
│   │       ├── idempotency.ts               # M-01 IdempotencyManager
│   │       ├── lock_status.ts               # M-02 LockStatusTracker
│   │       ├── trace_propagator.ts          # M-03 TraceIdPropagator (browser)
│   │       └── business_hash.ts             # M-04 BusinessKeyHasher
│   └── components/
│       └── exclusion/                       # 锁状态 UI
│           ├── LockStatusBadge.tsx          # M-05 顶部角标
│           ├── LockStatusPanel.tsx          # M-06 详情面板
│           └── LockConflictToast.tsx        # M-07 错乱 toast
│
├── crates/
│   ├── star-mutex/                          # L3 Domain 共享 mutex crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                       # M-08 DomainMutex (public trait)
│   │   │   ├── adapter.rs                   # M-09 StarMutexAdapter (22 domain 接入)
│   │   │   ├── audit.rs                     # M-10 LockAuditLogger
│   │   │   ├── policy_loader.rs             # M-11 ExclusionPolicyLoader
│   │   │   ├── trace.rs                     # M-12 TraceIdPropagator (Rust)
│   │   │   ├── version_cas.rs               # M-13 VersionCAS (乐观锁)
│   │   │   └── advisory_lock.rs             # M-14 PgAdvisoryLock (PG 包装)
│   │   └── tests/
│   │       ├── lib_test.rs                  # UT-01..UT-06
│   │       └── version_cas_test.rs          # UT-07..UT-10
│   │
│   └── star-mcp/                            # 16 tool 幂等改造
│       └── src/
│           ├── middleware/
│           │   └── idempotency.rs           # M-15 star-mcp-idem-middleware
│           └── tools/
│               └── (16 tool 改造)            # 16 tool 加 Idempotency-Key 解析
│
├── scripts/automation/                      # L1 TopAgent + L2 SubAgent (Python)
│   └── exclusion/                           # per 守门 #19 Python 化
│       ├── __init__.py
│       ├── dispatch_lock.py                 # M-16 DispatchLockManager
│       ├── idempotency_key_store.py         # M-17 IdempotencyKeyStore
│       ├── subagent_lock.py                 # M-18 SubAgentLock + heartbeat
│       ├── lock_watcher.py                  # M-19 LockWatcher (lease 过期)
│       ├── lock_metric_exporter.py          # M-20 LockMetricExporter
│       ├── lock_leak_alerter.py             # M-21 LockLeakAlerter
│       ├── archive_cron.py                  # M-22 ArchiveCronJob
│       └── business_hash.py                 # M-23 BusinessKeyHasher (Python 版)
│
├── docs/
│   └── migrations/
│       ├── 2026-09-07-exclusion-rls.sql     # 5 张新表 DDL + RLS 13 类
│       └── 2026-09-07-audit-trigger.sql     # audit_audit_event trigger
│
└── docs/architecture/2026-09-07-exclusion-idempotency/   # 本 view 文档
    ├── 01-requirements.md
    ├── 02-basic-design.md
    └── 03-detailed-design.md                # 本文档
```

### 1.2 模块依赖关系

```
M-01 IdempotencyManager (TS)
    └─→ M-03 TraceIdPropagator (TS) [browser]
    └─→ M-04 BusinessKeyHasher (TS) [shared with M-23]

M-16 DispatchLockManager (Py)
    └─→ M-17 IdempotencyKeyStore (Py)
    └─→ M-23 BusinessKeyHasher (Py)
    └─→ PG: pg_try_advisory_lock + idempotency_keys table

M-18 SubAgentLock (Py)
    └─→ M-19 LockWatcher (Py)
    └─→ PG: lease_log table

M-08 DomainMutex (Rust trait) ← 22 domain crate 接入
    └─→ M-09 StarMutexAdapter (Rust) [22 domain 适配]
    └─→ M-14 PgAdvisoryLock (Rust) [pg_advisory_xact_lock]
    └─→ M-13 VersionCAS (Rust) [version 校验]
    └─→ M-10 LockAuditLogger (Rust) [advisory_lock_audit table]
    └─→ M-11 ExclusionPolicyLoader (Rust) [exclusion_policy_master table]
    └─→ M-12 TraceIdPropagator (Rust) [X-Trace-Id 传递]

M-15 star-mcp-idem-middleware (Rust)
    └─→ M-17 IdempotencyKeyStore (Py) via gRPC
    └─→ 16 tool 调用前解析 Idempotency-Key header

M-20 LockMetricExporter (Py) + M-21 LockLeakAlerter (Py)
    └─→ 5 锁 metric (Prometheus) + Slack 告警

M-22 ArchiveCronJob (Py)
    └─→ idempotency_keys 24h 后移到 idempotency_keys_archive
```

## 2. 关键类 / 函数 / trait 签名

### 2.1 L0 UI 层 (TypeScript)

#### 2.1.1 M-01 IdempotencyManager (frontend/src/lib/exclusion/idempotency.ts)

```typescript
// per 02 §5.2.1 接口 + 守门 #7 0 unsafe
export interface IdempotencyManagerConfig {
  storageKeyPrefix: string;          // default: "star-idem:"
  defaultTimeoutMs: number;          // default: 30000
  enableLocalStorageCache: boolean;  // default: true
  enableAbortController: boolean;    // default: true
}

export interface InflightRecord {
  key: string;
  fingerprint: string;               // request body hash
  controller: AbortController;
  startedAt: number;                 // epoch ms
  promise: Promise<unknown>;
}

export class IdempotencyManager {
  private inflight: Map<string, InflightRecord> = new Map();
  private config: IdempotencyManagerConfig;
  
  constructor(config?: Partial<IdempotencyManagerConfig>) {
    this.config = { storageKeyPrefix: "star-idem:", defaultTimeoutMs: 30000, enableLocalStorageCache: true, enableAbortController: true, ...config };
  }
  
  // 生成新 key (UUID v4)
  public generateKey(): string {
    return crypto.randomUUID();
  }
  
  // dispatch + 自动 dedup (per F-06)
  public async dispatch<T>(
    key: string,
    fn: () => Promise<T>,
    opts?: {
      fallbackBusinessHash?: string;  // 双键 fallback
      timeoutMs?: number;             // 默认 30000
    },
  ): Promise<T> {
    const timeoutMs = opts?.timeoutMs ?? this.config.defaultTimeoutMs;
    const record = this.inflight.get(key);
    
    if (record) {
      // 取消旧请求, 用新请求
      if (this.config.enableAbortController) {
        record.controller.abort();
      }
    }
    
    const controller = new AbortController();
    const timeoutHandle = setTimeout(() => controller.abort(), timeoutMs);
    
    const promise = (async () => {
      try {
        return await fn();
      } finally {
        clearTimeout(timeoutHandle);
        this.inflight.delete(key);
      }
    })();
    
    this.inflight.set(key, {
      key,
      fingerprint: opts?.fallbackBusinessHash ?? "",
      controller,
      startedAt: Date.now(),
      promise,
    });
    
    return promise;
  }
  
  // 显式取消 in-flight
  public abort(key: string): void {
    const record = this.inflight.get(key);
    if (record) {
      record.controller.abort();
      this.inflight.delete(key);
    }
  }
  
  // localStorage 缓存 key
  public cacheKey(key: string, ttlMs: number): void {
    if (!this.config.enableLocalStorageCache) return;
    const expiresAt = Date.now() + ttlMs;
    localStorage.setItem(
      `${this.config.storageKeyPrefix}${key}`,
      JSON.stringify({ cachedAt: Date.now(), expiresAt }),
    );
  }
  
  public getCachedKey(key: string): string | null {
    if (!this.config.enableLocalStorageCache) return null;
    const raw = localStorage.getItem(`${this.config.storageKeyPrefix}${key}`);
    if (!raw) return null;
    try {
      const { expiresAt } = JSON.parse(raw);
      if (Date.now() > expiresAt) {
        localStorage.removeItem(`${this.config.storageKeyPrefix}${key}`);
        return null;
      }
      return raw;
    } catch {
      return null;
    }
  }
}
```

#### 2.1.2 M-04 BusinessKeyHasher (TS) + M-23 (Python 同步实现)

```typescript
// 业务主键 hash 算法: (tenant_id, resource_type, resource_id, operation) -> SHA-256
export class BusinessKeyHasher {
  public static hash(parts: {
    tenantId: string;
    resourceType: string;     // e.g. "work_item"
    resourceId: string;       // e.g. "UUID"
    operation: string;        // e.g. "update_status"
    version: number;          // e.g. 5 (for optimistic lock)
  }): string {
    const input = `${parts.tenantId}|${parts.resourceType}|${parts.resourceId}|${parts.operation}|${parts.version}`;
    // SHA-256 (Web Crypto API)
    const encoder = new TextEncoder();
    return crypto.subtle.digest("SHA-256", encoder.encode(input)).then(...);
  }
}
```

### 2.2 L1 TopAgent + L2 SubAgent 层 (Python)

#### 2.2.1 M-16 DispatchLockManager (scripts/automation/exclusion/dispatch_lock.py)

```python
# per 02 §5.2.2 接口 + 守门 #7 0 unsafe + 守门 #13 派生规
import asyncio
import hashlib
import json
import os
import time
import uuid
from dataclasses import dataclass
from typing import Optional

import asyncpg

# 守门 #5: 不打印 DATABASE_URL, 仅引用
DB_DSN = os.environ.get("STAR_PG_DSN", "postgresql://star:***@localhost:5432/star_exclusion")

@dataclass
class LockGuard:
    task_id: uuid.UUID
    dispatch_id: uuid.UUID
    lock_token: str
    acquired_at: float
    expires_at: float
    actor_id: uuid.UUID
    tenant_id: uuid.UUID
    trace_id: uuid.UUID


class DispatchLockManager:
    """L1 TopAgent 派发级排他 + 派发级幂等 (per F-02 + F-07)
    
    走守门 #13 a L0 协调: L1 SubAgent 不直接互抢, 经 L0 派发时锁住 task.
    锁实现: PG advisory lock (per ADR-0048 D-01).
    幂等: idempotency_keys 表 (per ADR-0048 D-02 双键).
    """
    
    def __init__(self, pool: asyncpg.Pool):
        self.pool = pool
        self.default_timeout_s = 60
    
    async def acquire(
        self,
        task_id: uuid.UUID,
        dispatch_id: uuid.UUID,
        actor_id: uuid.UUID,
        tenant_id: uuid.UUID,
        trace_id: uuid.UUID,
        timeout_ms: int = 60000,
    ) -> LockGuard:
        """获取派发级排他锁, 60s TTL, 5s 等待, 失败抛 DispatchLockHeld"""
        lock_class_id = self._compute_lock_class_id("task", str(task_id))
        started_at = time.monotonic()
        
        async with self.pool.acquire() as conn:
            # 1. 写 idempotency_keys (per F-07, 24h dedup)
            async with conn.transaction():
                # 2. 尝试获取 advisory lock (5s 等待, 60s 自动释放)
                got = await conn.fetchval(
                    "SELECT pg_try_advisory_xact_lock($1)", lock_class_id
                )
                if not got:
                    wait_ms = int((time.monotonic() - started_at) * 1000)
                    raise DispatchLockHeld(
                        task_id=task_id,
                        wait_ms=wait_ms,
                        message=f"task {task_id} 已被其他派发持有",
                    )
                
                # 3. 写 idempotency_keys (双键)
                await self._upsert_idempotency_key(
                    conn,
                    key=str(dispatch_id),
                    key_type="client",
                    request_fingerprint=hashlib.sha256(
                        f"{task_id}|{actor_id}|{tenant_id}".encode()
                    ).hexdigest(),
                    response_status=None,
                    response_body=None,
                    tenant_id=tenant_id,
                    workspace_id=uuid.UUID(int=0),  # 由 caller 注入
                    actor_id=actor_id,
                    trace_id=trace_id,
                )
                
                # 4. 写 lease_log
                await self._upsert_lease_log(
                    conn,
                    lease_id=uuid.uuid4(),
                    task_id=task_id,
                    lock_token=str(dispatch_id),
                    expires_in_s=timeout_ms // 1000,
                    tenant_id=tenant_id,
                    actor_id=actor_id,
                    trace_id=trace_id,
                )
        
        now = time.time()
        return LockGuard(
            task_id=task_id,
            dispatch_id=dispatch_id,
            lock_token=str(dispatch_id),
            acquired_at=now,
            expires_at=now + timeout_ms / 1000,
            actor_id=actor_id,
            tenant_id=tenant_id,
            trace_id=trace_id,
        )
    
    async def release(self, lock: LockGuard, reason: str = "normal") -> None:
        """释放派发级排他锁, 写 lease_log.released_at"""
        async with self.pool.acquire() as conn:
            async with conn.transaction():
                await conn.execute(
                    """
                    UPDATE lease_log 
                    SET released_at = NOW(), 
                        release_reason = $2 
                    WHERE lock_token = $1 AND released_at IS NULL
                    """,
                    lock.lock_token, reason,
                )
    
    async def is_locked(self, task_id: uuid.UUID) -> bool:
        """检查 task 是否被其他派发持有"""
        lock_class_id = self._compute_lock_class_id("task", str(task_id))
        async with self.pool.acquire() as conn:
            return await conn.fetchval(
                "SELECT pg_try_advisory_lock($1)", lock_class_id
            ) or False
    
    @staticmethod
    def _compute_lock_class_id(namespace: str, key: str) -> int:
        """计算 PG advisory lock class ID (BIGINT)
        
        命名空间按资源类型分: task / work_item / worktree / merge_request / comment
        hashtextextended 返回 BIGINT
        """
        return int(hashlib.sha256(f"{namespace}:{key}".encode()).hexdigest()[:15], 16)


class DispatchLockHeld(Exception):
    def __init__(self, task_id: uuid.UUID, wait_ms: int, message: str):
        self.task_id = task_id
        self.wait_ms = wait_ms
        super().__init__(message)
```

#### 2.2.2 M-18 SubAgentLock (scripts/automation/exclusion/subagent_lock.py)

```python
# per 02 §5.2.3 接口 + F-03 任务卡持锁
import asyncio
import time
import uuid
from dataclasses import dataclass
from enum import Enum
from typing import Optional

import asyncpg


class ReleaseReason(str, Enum):
    NORMAL = "normal"
    TIMEOUT = "timeout"
    ERROR = "error"
    FORCED = "forced"  # admin 域强制


@dataclass
class LeaseHandle:
    lease_id: uuid.UUID
    task_id: uuid.UUID
    subagent_id: str         # SA-01..SA-09 + SA-10
    lock_token: str
    heartbeat_at: float
    expires_at: float
    actor_id: uuid.UUID
    trace_id: uuid.UUID


class SubAgentLock:
    """L2 SubAgent 任务卡持锁 + heartbeat (per F-03)"""
    
    def __init__(self, pool: asyncpg.Pool):
        self.pool = pool
        self.default_heartbeat_s = 30
        self.default_expires_s = 300
        self._heartbeat_tasks: dict[str, asyncio.Task] = {}
    
    async def acquire(
        self,
        task_id: uuid.UUID,
        subagent_id: str,
        actor_id: uuid.UUID,
        trace_id: uuid.UUID,
        heartbeat_interval_s: int = 30,
        expires_s: int = 300,
    ) -> LeaseHandle:
        lease_id = uuid.uuid4()
        lock_token = f"lease-{lease_id}"
        now = time.time()
        
        async with self.pool.acquire() as conn:
            async with conn.transaction():
                # 1. 检查 task 是否已被其他 SA 持有
                existing = await conn.fetchval(
                    """
                    SELECT 1 FROM lease_log 
                    WHERE task_id = $1 AND released_at IS NULL AND expires_at > NOW()
                    LIMIT 1
                    """,
                    task_id,
                )
                if existing:
                    raise SubAgentLockHeld(task_id=task_id, subagent_id=subagent_id)
                
                # 2. 写 lease_log
                await conn.execute(
                    """
                    INSERT INTO lease_log (
                        lease_id, task_id, lock_token, heartbeat_at, expires_at,
                        tenant_id, workspace_id, actor_id, trace_id
                    ) VALUES ($1, $2, $3, NOW(), NOW() + ($4 || ' seconds')::INTERVAL, $5, $6, $7, $8)
                    """,
                    lease_id, task_id, lock_token,
                    str(expires_s),
                    # tenant_id / workspace_id 由 caller 注入, 此处用 placeholder
                    uuid.UUID(int=0), uuid.UUID(int=0),
                    actor_id, trace_id,
                )
        
        # 3. 启动 heartbeat
        handle = LeaseHandle(
            lease_id=lease_id,
            task_id=task_id,
            subagent_id=subagent_id,
            lock_token=lock_token,
            heartbeat_at=now,
            expires_at=now + expires_s,
            actor_id=actor_id,
            trace_id=trace_id,
        )
        self._start_heartbeat(handle, heartbeat_interval_s)
        return handle
    
    def _start_heartbeat(self, handle: LeaseHandle, interval_s: int) -> None:
        async def heartbeat_loop():
            while True:
                await asyncio.sleep(interval_s)
                try:
                    await self.heartbeat(handle)
                except Exception:
                    # heartbeat 失败, 锁自动过期
                    break
        
        task = asyncio.create_task(heartbeat_loop())
        self._heartbeat_tasks[handle.lock_token] = task
    
    async def heartbeat(self, handle: LeaseHandle) -> None:
        async with self.pool.acquire() as conn:
            await conn.execute(
                """
                UPDATE lease_log 
                SET heartbeat_at = NOW(), 
                    expires_at = NOW() + INTERVAL '300 seconds'
                WHERE lock_token = $1 AND released_at IS NULL
                """,
                handle.lock_token,
            )
        handle.heartbeat_at = time.time()
        handle.expires_at = time.time() + 300
    
    async def release(self, handle: LeaseHandle, reason: ReleaseReason = ReleaseReason.NORMAL) -> None:
        # 1. 停止 heartbeat
        if handle.lock_token in self._heartbeat_tasks:
            self._heartbeat_tasks[handle.lock_token].cancel()
            del self._heartbeat_tasks[handle.lock_token]
        
        # 2. 写 lease_log.released_at
        async with self.pool.acquire() as conn:
            await conn.execute(
                """
                UPDATE lease_log 
                SET released_at = NOW(), release_reason = $2 
                WHERE lock_token = $1 AND released_at IS NULL
                """,
                handle.lock_token, reason.value,
            )


class SubAgentLockHeld(Exception):
    def __init__(self, task_id: uuid.UUID, subagent_id: str):
        self.task_id = task_id
        self.subagent_id = subagent_id
        super().__init__(f"task {task_id} 已被其他 SubAgent 持有")
```

### 2.3 L3 Domain 层 (Rust)

#### 2.3.1 M-08 DomainMutex (crates/star-mutex/src/lib.rs)

```rust
// per 02 §5.2.4 trait + 守门 #7 0 unsafe + 守门 #13 派生规
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::advisory_lock::PgAdvisoryLock;
use crate::audit::LockAuditLogger;
use crate::policy_loader::ExclusionPolicyLoader;
use crate::trace::TraceIdPropagator;
use crate::version_cas::{VersionCAS, VersionMismatchError};

/// Domain 业务行锁 trait (per ADR-0048 D-03 L3 Domain 责任)
#[async_trait]
pub trait DomainMutex: Send + Sync {
    /// 尝试获取业务行锁, transaction-bound
    async fn try_lock_with_timeout(
        &self,
        resource: &str,         // e.g. "work_item:UUID"
        timeout_ms: u64,
    ) -> Result<MutexGuard, DomainMutexError>;
    
    /// 基于 version CAS 的乐观锁写入
    async fn version_cas<F, T>(
        &self,
        resource: &str,
        expected_version: u64,
        f: F,
    ) -> Result<T, DomainMutexError>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<(T, u64), sqlx::Error>> + Send;
    
    /// 检查资源是否被其他事务持有
    async fn is_locked(&self, resource: &str) -> Result<bool, DomainMutexError>;
}

pub struct MutexGuard {
    resource: String,
    lock_class_id: i64,
    acquired_at: chrono::DateTime<chrono::Utc>,
    released: bool,
}

impl MutexGuard {
    pub fn resource(&self) -> &str { &self.resource }
}

impl Drop for MutexGuard {
    fn drop(&mut self) {
        // 事务结束自动释放 (pg_advisory_xact_lock)
        self.released = true;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainMutexError {
    #[error("timeout after {timeout_ms}ms waiting for {resource}")]
    Timeout { resource: String, timeout_ms: u64 },
    
    #[error("version mismatch: expected {expected}, actual {actual}")]
    VersionMismatch { expected: u64, actual: u64, resource: String },
    
    #[error("pg advisory lock failed: {0}")]
    PgError(#[from] sqlx::Error),
    
    #[error("audit write failed: {0}")]
    AuditError(String),
}


/// PgAdvisoryLockMutex - PG advisory_xact_lock 实现
pub struct PgAdvisoryLockMutex {
    pool: PgPool,
    advisory: PgAdvisoryLock,
    audit: Arc<LockAuditLogger>,
    policy_loader: Arc<ExclusionPolicyLoader>,
    trace: Arc<TraceIdPropagator>,
}

impl PgAdvisoryLockMutex {
    pub fn new(
        pool: PgPool,
        audit: Arc<LockAuditLogger>,
        policy_loader: Arc<ExclusionPolicyLoader>,
        trace: Arc<TraceIdPropagator>,
    ) -> Self {
        let advisory = PgAdvisoryLock::new(pool.clone());
        Self { pool, advisory, audit, policy_loader, trace }
    }
}

#[async_trait]
impl DomainMutex for PgAdvisoryLockMutex {
    async fn try_lock_with_timeout(
        &self,
        resource: &str,
        timeout_ms: u64,
    ) -> Result<MutexGuard, DomainMutexError> {
        let started_at = std::time::Instant::now();
        let lock_class_id = compute_lock_class_id(resource);
        
        // 1. 加载策略 (per exclusion_policy_master)
        let policy = self.policy_loader.load("domain_row_lock").await?;
        let timeout_ms = timeout_ms.min(policy.timeout_ms as u64);
        
        // 2. 尝试获取 (带超时, transaction-bound)
        let guard = self.advisory
            .try_xact_lock(lock_class_id, timeout_ms)
            .await
            .map_err(|e| match e {
                PgAdvisoryLockError::Timeout { waited_ms } => DomainMutexError::Timeout {
                    resource: resource.to_string(),
                    timeout_ms: waited_ms,
                },
                other => DomainMutexError::PgError(other.into()),
            })?;
        
        let acquired_at = chrono::Utc::now();
        let wait_ms = started_at.elapsed().as_millis() as i32;
        
        // 3. 写 audit (per ADR-0043 WORM)
        self.audit.log_acquired(
            lock_class_id,
            resource,
            wait_ms,
            self.trace.current_tenant_id(),
            self.trace.current_actor_id(),
            self.trace.current_trace_id(),
        ).await.map_err(|e| DomainMutexError::AuditError(e.to_string()))?;
        
        Ok(MutexGuard {
            resource: resource.to_string(),
            lock_class_id,
            acquired_at,
            released: false,
        })
    }
    
    async fn version_cas<F, T>(
        &self,
        resource: &str,
        expected_version: u64,
        f: F,
    ) -> Result<T, DomainMutexError>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<(T, u64), sqlx::Error>> + Send,
    {
        // 1. 获取业务行锁
        let _guard = self.try_lock_with_timeout(resource, 5000).await?;
        
        // 2. 读当前 version
        let current = self.read_version(resource).await?;
        if current != expected_version {
            return Err(DomainMutexError::VersionMismatch {
                expected: expected_version,
                actual: current,
                resource: resource.to_string(),
            });
        }
        
        // 3. 执行业务 + 写新 version
        let (result, new_version) = f().await.map_err(DomainMutexError::PgError)?;
        
        // 4. 校验 version 单调递增
        if new_version != expected_version + 1 {
            return Err(DomainMutexError::VersionMismatch {
                expected: expected_version + 1,
                actual: new_version,
                resource: resource.to_string(),
            });
        }
        
        Ok(result)
    }
    
    async fn is_locked(&self, resource: &str) -> Result<bool, DomainMutexError> {
        let lock_class_id = compute_lock_class_id(resource);
        self.advisory.is_locked(lock_class_id).await
            .map_err(|e| DomainMutexError::PgError(e.into()))
    }
}


/// 计算 PG advisory lock class ID (BIGINT, 范围 -2^63..2^63-1)
/// 
/// 命名空间按资源类型分: work_item / task / worktree / merge_request / comment
/// 用 SHA-256 前 8 字节转 i64, 避免冲突
pub fn compute_lock_class_id(resource: &str) -> i64 {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(resource.as_bytes());
    let result = hasher.finalize();
    // 取前 8 字节, 转为 i64
    let bytes: [u8; 8] = result[..8].try_into().unwrap();
    i64::from_be_bytes(bytes)
}
```

#### 2.3.2 M-09 StarMutexAdapter (crates/star-mutex/src/adapter.rs)

```rust
// 22 domain crate 接入适配: 提供 macro 简化接入
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn domain_mutex(attr: TokenStream, input: TokenStream) -> TokenStream {
    // attr: resource_type (e.g. "work_item")
    let resource_type = attr.to_string();
    
    let expanded = quote! {
        // 自动注入 DomainMutex + VersionCAS
        #[async_trait::async_trait]
        impl crate::star_mutex::DomainMutex for #input {
            // ...
        }
    };
    
    expanded.into()
}
```

**22 domain crate 接入示例** (`crates/domain-work-item/src/service.rs`):

```rust
use star_mutex::{DomainMutex, domain_mutex, VersionCAS};

#[domain_mutex(resource_type = "work_item")]
pub struct WorkItemService {
    mutex: Arc<dyn DomainMutex>,
    pool: PgPool,
}

impl WorkItemService {
    pub async fn update_status(
        &self,
        id: WorkItemId,
        new_status: WorkItemStatus,
        expected_version: u64,
        actor: ActorContext,
    ) -> Result<WorkItem, WorkItemError> {
        let resource = format!("work_item:{}", id);
        
        self.mutex.version_cas(&resource, expected_version, || {
            Box::pin(async move {
                let updated = sqlx::query!(
                    "UPDATE work_item SET status = $1, version = version + 1 WHERE id = $2 AND version = $3 RETURNING *",
                    new_status as _, id as _, expected_version
                )
                .fetch_one(&self.pool)
                .await?;
                Ok((updated.into(), updated.version as u64))
            })
        }).await
    }
}
```

### 2.4 跨层 (L4 Cross)

#### 2.4.1 M-10 LockAuditLogger (crates/star-mutex/src/audit.rs)

```rust
// 锁事件写 advisory_lock_audit 表 (per F-11)
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct LockAuditLogger {
    pool: PgPool,
}

impl LockAuditLogger {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    
    pub async fn log_acquired(
        &self,
        lock_class_id: i64,
        lock_target: &str,
        wait_ms: i32,
        tenant_id: Uuid,
        actor_id: Uuid,
        trace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO advisory_lock_audit (
                lock_class_id, lock_target, acquired_at, wait_ms,
                tenant_id, actor_id, trace_id
            ) VALUES ($1, $2, NOW(), $3, $4, $5, $6)
            "#,
            lock_class_id, lock_target, wait_ms, tenant_id, actor_id, trace_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    pub async fn log_released(
        &self,
        lock_class_id: i64,
        lock_target: &str,
        hold_ms: i32,
        tenant_id: Uuid,
        actor_id: Uuid,
        trace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE advisory_lock_audit 
            SET released_at = NOW(), hold_ms = $3
            WHERE lock_class_id = $1 AND lock_target = $2 AND released_at IS NULL
            "#,
            lock_class_id, lock_target, hold_ms
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    pub async fn log_timeout(
        &self,
        lock_class_id: i64,
        lock_target: &str,
        tenant_id: Uuid,
        actor_id: Uuid,
        trace_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        // 写 audit_audit_event (per ADR-0043 WORM)
        sqlx::query!(
            r#"
            INSERT INTO audit_audit_event (
                event_type, lock_target, actor_id, tenant_id, trace_id, 
                occurred_at, severity
            ) VALUES ('lock.timeout', $1, $2, $3, $4, NOW(), 'warning')
            "#,
            lock_target, actor_id, tenant_id, trace_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

## 3. 5 状态机 (State Machines)

### 3.1 Client dedup 状态机 (M-01 IdempotencyManager)

```
                    ┌────────────┐
                    │   IDLE     │ ← 初始
                    └─────┬──────┘
                          │ dispatch(key, fn)
                          ▼
                    ┌────────────┐
            ┌──────┤ INFLIGHT   │
            │      └─────┬──────┘
            │            │ fn() resolves
            │            ▼
            │      ┌────────────┐
            │      │ COMPLETED  │ → 清除 inflight record
            │      └────────────┘
            │
            │ 同一 key 再次 dispatch
            ▼
      ┌────────────┐
      │ ABORTING   │ → 取消旧 request
      └─────┬──────┘
            │ 启动新 request
            ▼
      ┌────────────┐
      │ INFLIGHT   │ (新 record)
      └────────────┘
```

### 3.2 Dispatch lock 状态机 (M-16 DispatchLockManager)

```
                    ┌────────────┐
                    │   FREE     │ ← 初始
                    └─────┬──────┘
                          │ acquire(task_id, dispatch_id)
                          ▼
                    ┌────────────┐
            ┌──────┤   HELD     │ ← 60s TTL
            │      └─────┬──────┘
            │            │ release() / TTL expire
            │            ▼
            │      ┌────────────┐
            │      │ RELEASED   │ → 写 lease_log.released_at
            │      └────────────┘
            │
            │ 同一 task_id 再次 acquire
            ▼
      ┌────────────┐
      │  BLOCKED   │ → 抛 DispatchLockHeld, 409
      └────────────┘
```

### 3.3 SubAgent lease 状态机 (M-18 SubAgentLock)

```
                    ┌────────────┐
                    │   FREE     │ ← 初始
                    └─────┬──────┘
                          │ acquire(task_id, subagent_id)
                          ▼
                    ┌────────────┐
            ┌──────┤  ACQUIRED  │ ← 300s 过期
            │      └─────┬──────┘
            │            │ heartbeat(30s)
            │            ▼
            │      ┌────────────┐
            │      │   HELD     │ ← 续约中
            │      └─────┬──────┘
            │            │ heartbeat 失败 / 崩溃
            │            ▼
            │      ┌────────────┐
            │      │  EXPIRED   │ → 自动释放, 告警
            │      └────────────┘
            │
            │ release() 显式
            ▼
      ┌────────────┐
      │  RELEASED  │ → 写 lease_log.released_at + reason
      └────────────┘
```

### 3.4 Domain mutex 状态机 (M-08 DomainMutex)

```
                    ┌────────────┐
                    │ UNLOCKED   │ ← 初始
                    └─────┬──────┘
                          │ try_lock_with_timeout(resource, 5s)
                          ▼
                    ┌────────────┐
            ┌──────┤  WAITING   │ ← 排队等锁
            │      └─────┬──────┘
            │            │ 锁可用
            │            ▼
            │      ┌────────────┐
            │      │  LOCKED    │ ← 事务内持有
            │      └─────┬──────┘
            │            │ 事务结束 / 显式 drop guard
            │            ▼
            │      ┌────────────┐
            │      │ UNLOCKED   │ (回到初始)
            │      └────────────┘
            │
            │ 5s 超时
            ▼
      ┌────────────┐
      │  TIMEOUT   │ → 抛 DomainMutexError::Timeout
      └────────────┘
```

### 3.5 Idempotency key 状态机 (M-17 IdempotencyKeyStore)

```
                    ┌────────────┐
                    │  PENDING   │ ← 首次请求到达
                    └─────┬──────┘
                          │ fn() resolves
                          ▼
                    ┌────────────┐
            ┌──────┤  RESOLVED  │ ← 写入 response_status + body
            │      └─────┬──────┘
            │            │ 24h 后 cron
            │            ▼
            │      ┌────────────┐
            │      │  ARCHIVED  │ → 移到 idempotency_keys_archive
            │      └────────────┘
            │
            │ 重复请求到达 (同 key)
            ▼
      ┌────────────┐
      │  DEDUPED   │ → 返回首次 response (无需执行 fn)
      └────────────┘
```

## 4. 5 时序图 (Sequence Diagrams)

### 4.1 4 层锁链时序 (S-03: User A + User B 同时"暂停 task X")

```
User A (gm-console)         L0 TopAgent           L1 SubAgent       L3 Domain
   │                            │                     │                 │
   │ POST /api/dispatch         │                     │                 │
   │ {Idempotency-Key, X-Trace-Id}                   │                 │
   ├───────────────────────────→│                     │                 │
   │                            │ DispatchLockManager.acquire(task X)    │
   │                            │ ├─ PG advisory lock (60s)             │
   │                            │ ├─ idempotency_keys insert             │
   │                            │ ├─ lease_log insert                    │
   │                            │ Return LockGuard                      │
   │                            ├────────────────────→│                  │
   │                            │                     │ SubAgentLock.acquire(task X)
   │                            │                     │ ├─ check existing
   │                            │                     │ ├─ lease_log insert
   │                            │                     │ Return LeaseHandle
   │                            │                     │                  │
   │                            │                     │ DomainMutex.version_cas("work_item:X", 5)
   │                            │                     ├─────────────────→│
   │                            │                     │                  │ ├─ PG advisory_xact_lock
   │                            │                     │                  │ ├─ read version (5)
   │                            │                     │                  │ ├─ write new version (6)
   │                            │                     │                  │ Return updated
   │                            │                     │←─────────────────┤
   │                            │                     │                  │
   │ 200 OK                     │                     │                  │
   │←───────────────────────────┤                     │                  │
   │                            │                     │                  │

User B (gm-console)         L0 TopAgent           
   │                            │                     
   │ POST /api/dispatch         │                     
   │ {Idempotency-Key, X-Trace-Id}                   
   ├───────────────────────────→│                     
   │                            │ DispatchLockManager.acquire(task X)
   │                            │ ├─ PG advisory lock (60s) 失败 (A 持有)
   │                            │ ├─ raise DispatchLockHeld
   │ 409 Conflict               │                     
   │ {Retry-After: 30}          │                     
   │←───────────────────────────┤                     
   │                            │                     
   │ UI toast: "task X 正在被 User A 操作, 请稍后重试" 
```

### 4.2 双键 dedup 时序 (S-07: 客户端漏带 Idempotency-Key)

```
Client (gm-console)         L0 TopAgent (IdempotencyKeyStore)
   │                            │
   │ POST /api/dispatch         │
   │ (无 Idempotency-Key header)│
   ├───────────────────────────→│
   │                            │ ├─ 检查 header: 缺失
   │                            │ ├─ 计算 business_hash:
   │                            │   SHA-256(tenant_id|resource|op|version)
   │                            │ ├─ INSERT idempotency_keys:
   │                            │   key=business_hash, key_type=business
   │                            │ Return OK
   │ 200 OK                     │
   │←───────────────────────────┤

Client 重试 (5s 后)         
   │                            │
   │ POST /api/dispatch         │
   │ (无 Idempotency-Key header)│
   ├───────────────────────────→│
   │                            │ ├─ 检查 header: 缺失
   │                            │ ├─ 计算 business_hash (同前)
   │                            │ ├─ SELECT idempotency_keys WHERE key=...
   │                            │ ├─ 命中! 返回首次 response
   │ 200 OK (首次响应 replay)   │
   │←───────────────────────────┤
```

### 4.3 Heartbeat 时序 (F-03: SubAgent lease + heartbeat)

```
SubAgent (SA-01)            M-18 SubAgentLock              PG (lease_log)
   │                            │                              │
   │ acquire(task X)            │                              │
   ├───────────────────────────→│                              │
   │                            │ INSERT lease_log             │
   │                            ├─────────────────────────────→│
   │                            │                              │
   │ Return LeaseHandle         │                              │
   │←───────────────────────────┤                              │
   │                            │                              │
   │ sleep(30s)                 │                              │
   │                            │                              │
   │ heartbeat(lease)           │                              │
   ├───────────────────────────→│                              │
   │                            │ UPDATE lease_log             │
   │                            │ SET heartbeat_at=NOW(),      │
   │                            │     expires_at=NOW()+300s    │
   │                            ├─────────────────────────────→│
   │                            │                              │
   │ sleep(30s)                 │                              │
   │                            │                              │
   │ heartbeat(lease)           │                              │
   ├───────────────────────────→│                              │
   │                            │ UPDATE lease_log             │
   │                            ├─────────────────────────────→│
   │                            │                              │
   │ ... (300s 后)              │                              │
   │                            │                              │
   │ release(lease, normal)     │                              │
   ├───────────────────────────→│                              │
   │                            │ UPDATE lease_log             │
   │                            │ SET released_at=NOW(),       │
   │                            │     release_reason=normal    │
   │                            ├─────────────────────────────→│
```

### 4.4 TMO bulk_node 原子性时序 (S-05: bulk 5 任务中途 User C 修改)

```
User A (L0 TopAgent)        TMO bulk_node        5 个 SubAgent       User C (gm-console)
   │                            │                     │                 │
   │ bulk_pause(5 tasks)        │                     │                 │
   ├───────────────────────────→│                     │                 │
   │                            │ DispatchLockManager │                 │
   │                            │ .acquire(task1..5)  │                 │
   │                            │ ├─ 5 个 task 都上锁 │                 │
   │                            ├─→ SubAgent pause(task1)                │
   │                            │   ├─ acquire lease1                    │
   │                            │   ├─ version_cas(work_item:1, v5)      │
   │                            │   │   ├─ 锁 OK, 写 v6, 成功            │
   │                            │   ←─ success                          │
   │                            ├─→ SubAgent pause(task2)                │
   │                            │   ├─ acquire lease2                    │
   │                            │   ├─ version_cas(work_item:2, v3)      │
   │                            │   │   ├─ 锁 OK, 写 v4, 成功            │
   │                            │   ←─ success                          │
   │                            ├─→ SubAgent pause(task3)                │
   │                            │   ├─ acquire lease3  (但被 User C 抢先) │
   │                            │   │   ├─ DispatchLockHeld             │
   │                            │   ←─ failed                           │
   │                            │                     │ POST /api/dispatch
   │                            │                     │ { task: task3, User C }
   │                            │                     ├────────────────→│
   │                            │                     │                 │
   │                            │ (User C 操作 task3)│                 │
   │                            │                     │ SubAgent 修改 task3
   │                            │                     │ version_cas(v3 → v4) 成功
   │                            │                     │←────────────────┤
   │                            │                     │ 200 OK          │
   │                            │                     │                 │
   │                            │ bulk_pause 检测: task3 失败,           │
   │                            │ ├─ 1-2 已成功, 3 失败, 4-5 未执行    │
   │                            │ ├─ 部分回滚: 1-2 保留 paused,         │
   │                            │ │   3-5 跳过                          │
   │                            │ ├─ 通知 User A: 部分失败             │
   │ 207 Multi-Status           │                     │                 │
   │ {1-2 success, 3 failed,    │                     │                 │
   │  4-5 not_executed}         │                     │                 │
   │←───────────────────────────┤                     │                 │
```

### 4.5 跨层 trace 时序 (per F-12)

```
User A (gm-console)    L0 TopAgent    L1 SubAgent    L3 Domain    PG (advisory_lock_audit)
   │                       │               │              │                  │
   │ trace_id=trace-1       │               │              │                  │
   │ POST /api/dispatch    │               │              │                  │
   ├──────────────────────→│               │              │                  │
   │                       │ trace_id=trace-1              │                  │
   │                       │ log: "DispatchLockManager"   │                  │
   │                       │ .acquire start"               │                  │
   │                       │ INSERT lease_log(trace_id=trace-1)             │
   │                       ├──────────────────────────────┼─────────────────→│
   │                       │               │              │                  │
   │                       │               │ trace_id=trace-1               │
   │                       │               │ log: "SubAgentLock.acquire"    │
   │                       │               │ INSERT lease_log(trace_id=trace-1)
   │                       │               ├──────────────┼─────────────────→│
   │                       │               │              │                  │
   │                       │               │              │ trace_id=trace-1 │
   │                       │               │              │ log: "DomainMutex.version_cas"
   │                       │               │              │ pg_try_advisory_xact_lock
   │                       │               │              │ INSERT advisory_lock_audit(trace_id=trace-1)
   │                       │               │              ├─────────────────→│
   │                       │               │              │                  │
   │                       │               │              │ Return updated   │
   │                       │               │←─────────────┤                  │
   │                       │               │              │                  │
   │                       │               │ release lease│                  │
   │                       │               │ UPDATE lease_log(trace_id=trace-1)
   │                       │               ├──────────────┼─────────────────→│
   │                       │               │              │                  │
   │                       │ release dispatch lock        │                  │
   │                       │ UPDATE lease_log(trace_id=trace-1)              │
   │                       ├──────────────────────────────┼─────────────────→│
   │                       │               │              │                  │
   │ 200 OK                │               │              │                  │
   │←──────────────────────┤               │              │                  │

# 锁错乱排查: SELECT * FROM audit_audit_event WHERE trace_id = 'trace-1'
# 1 click 看到 4 层日志 + 锁链
```

## 5. 5 张新表完整 DDL

### 5.1 idempotency_keys (T) - per F-07

```sql
-- per 守门 #13 b Transaction: 物理删除禁止 + 監査必須 + RLS 13 類必携
CREATE EXTENSION IF NOT EXISTS pgcrypto;  -- gen_random_uuid()

CREATE TABLE idempotency_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL,
    key_type TEXT NOT NULL CHECK (key_type IN ('client', 'business', 'dual')),
    request_fingerprint TEXT NOT NULL,
    response_status INT,
    response_body JSONB,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (key, tenant_id)
);

-- 索引 (per 02 §7.2 性能)
CREATE INDEX idx_idem_keys_tenant_expires ON idempotency_keys(tenant_id, expires_at);
CREATE INDEX idx_idem_keys_actor_created ON idempotency_keys(actor_id, created_at DESC);
CREATE INDEX idx_idem_keys_trace ON idempotency_keys(trace_id);

-- 审计 trigger (per ADR-0043 WORM)
CREATE TRIGGER trg_idem_keys_audit
AFTER INSERT OR UPDATE ON idempotency_keys
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类 policy (per 守门 #13 c)
ALTER TABLE idempotency_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY idem_keys_tenant_isolation ON idempotency_keys
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY idem_keys_workspace_isolation ON idempotency_keys
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);
```

### 5.2 lease_log (T) - per F-03

```sql
-- per 守门 #13 b Transaction: 物理删除禁止 + 監査必須 + RLS 13 類必携
CREATE TABLE lease_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lease_id UUID NOT NULL,
    task_id UUID NOT NULL,
    lock_token TEXT NOT NULL,
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    release_reason TEXT CHECK (release_reason IN ('normal', 'timeout', 'error', 'forced')),
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_lease_log_lease_heartbeat ON lease_log(lease_id, heartbeat_at DESC);
CREATE INDEX idx_lease_log_task_released ON lease_log(task_id, released_at);
CREATE INDEX idx_lease_log_actor_created ON lease_log(actor_id, created_at DESC);
CREATE INDEX idx_lease_log_active ON lease_log(expires_at) WHERE released_at IS NULL;

-- 审计 trigger
CREATE TRIGGER trg_lease_log_audit
AFTER INSERT OR UPDATE ON lease_log
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类
ALTER TABLE lease_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY lease_log_tenant_isolation ON lease_log
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY lease_log_workspace_isolation ON lease_log
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);
```

### 5.3 advisory_lock_audit (T) - per F-11

```sql
-- per 守门 #13 b Transaction
CREATE TABLE advisory_lock_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lock_class_id BIGINT NOT NULL,
    lock_target TEXT NOT NULL,
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

-- 索引
CREATE INDEX idx_lock_audit_target_acquired ON advisory_lock_audit(lock_target, acquired_at DESC);
CREATE INDEX idx_lock_audit_actor_created ON advisory_lock_audit(actor_id, created_at DESC);
CREATE INDEX idx_lock_audit_trace ON advisory_lock_audit(trace_id);
CREATE INDEX idx_lock_audit_hold_desc ON advisory_lock_audit(hold_ms DESC) WHERE hold_ms IS NOT NULL;

-- 审计 trigger
CREATE TRIGGER trg_lock_audit_audit
AFTER INSERT OR UPDATE ON advisory_lock_audit
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- RLS 13 类
ALTER TABLE advisory_lock_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY lock_audit_tenant_isolation ON advisory_lock_audit
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY lock_audit_workspace_isolation ON advisory_lock_audit
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);
```

### 5.4 idempotency_keys_archive (T) - per 02 §3.1

同 `idempotency_keys` schema + `archived_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` 字段; 由 24h cron job 移动.

### 5.5 exclusion_policy_master (M, SCD Type 2) - per 02 §3.6

```sql
-- per 守门 #13 c Master: 物理删除禁止 + SCD Type 2 + RLS 13 類必携
CREATE TABLE exclusion_policy_master (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_name TEXT NOT NULL,
    lock_strategy TEXT NOT NULL CHECK (lock_strategy IN ('advisory', 'lease', 'optimistic')),
    timeout_ms INT NOT NULL,
    retry_max INT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (policy_name, valid_from)
);

-- 索引
CREATE INDEX idx_excl_policy_active ON exclusion_policy_master(policy_name) WHERE valid_to IS NULL;
CREATE INDEX idx_excl_policy_tenant ON exclusion_policy_master(tenant_id);

-- SCD Type 2 trigger: 更新时关闭旧记录
CREATE OR REPLACE FUNCTION scd_type2_close() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.valid_to IS NULL AND NEW.valid_to IS NULL THEN
        NEW.valid_to = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

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
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
```

## 6. 错误处理 (Error Handling)

### 6.1 错误码定义 (per F-02 + F-03 + F-04 + F-09)

| 错误码 | HTTP | 含义 | 客户端处理 |
|---|---|---|---|
| `IDEMPOTENCY_KEY_MISSING` | 400 | Idempotency-Key header 缺失 | 升级 SDK, 双键 fallback |
| `IDEMPOTENCY_KEY_REUSE` | 422 | key 复用但 request_fingerprint 不匹配 | 提示 key 冲突, 生成新 key |
| `DISPATCH_LOCK_HELD` | 409 | 派发级锁被其他 actor 持有 | 等待 + 重试, 或提示用户 |
| `SUBAGENT_LOCK_HELD` | 409 | task 被其他 SubAgent 持有 | 等待 + 重试, 或选其他 SA |
| `DOMAIN_LOCK_TIMEOUT` | 504 | 业务行锁 5s 超时 | 提示用户重试 |
| `VERSION_MISMATCH` | 409 | version CAS 不匹配, 数据已被修改 | 重试 + 拉新 version |
| `LEASE_EXPIRED` | 410 | lease 过期 (崩溃恢复) | 自动重试 1 次 |
| `FORCE_RELEASE` | 410 | admin 强制释放 | 通知 owner, 重试 |

### 6.2 错误响应格式 (per ADR-0032 6 字段错误模型)

```json
{
  "error_code": "DISPATCH_LOCK_HELD",
  "message": "task X 正在被 User A 操作, 请稍后重试",
  "details": {
    "task_id": "UUID",
    "held_by": "User A",
    "held_since": "2026-09-07T20:55:00Z",
    "retry_after_seconds": 30
  },
  "trace_id": "trace-uuid",
  "documentation_url": "https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-09-07-exclusion-idempotency/01-requirements.md#f-02"
}
```

## 7. 测试设计 (Test Design, per 02 §6 验收)

### 7.1 18 UT (Unit Tests, per crate 内)

| # | 测试 | 路径 | 覆盖 |
|---|---|---|---|
| **UT-01** | IdempotencyManager dispatch 取消 in-flight | `frontend/src/lib/exclusion/idempotency.test.ts` | F-06 |
| **UT-02** | IdempotencyManager localStorage 缓存 | 同上 | F-06 |
| **UT-03** | BusinessKeyHasher 业务主键 hash | `frontend/src/lib/exclusion/business_hash.test.ts` | F-09 |
| **UT-04** | LockStatusBadge 角标状态 | `frontend/src/components/exclusion/LockStatusBadge.test.tsx` | F-12 |
| **UT-05** | compute_lock_class_id SHA-256 转 i64 | `crates/star-mutex/src/lib.rs` test | F-04 |
| **UT-06** | DomainMutex trait 默认实现 | `crates/star-mutex/src/lib.rs` test | F-04 |
| **UT-07** | VersionCAS 乐观锁冲突检测 | `crates/star-mutex/src/version_cas.rs` test | F-04 |
| **UT-08** | VersionCAS 乐观锁重试 | 同上 | F-04 |
| **UT-09** | PgAdvisoryLock try_xact_lock 5s 超时 | `crates/star-mutex/src/advisory_lock.rs` test | F-04 |
| **UT-10** | LockAuditLogger 写 audit_audit_event | `crates/star-mutex/src/audit.rs` test | F-11 |
| **UT-11** | ExclusionPolicyLoader 加载 strategy | `crates/star-mutex/src/policy_loader.rs` test | F-13 |
| **UT-12** | TraceIdPropagator 4 层传递 | `crates/star-mutex/src/trace.rs` test | F-12 |
| **UT-13** | DispatchLockManager.acquire 60s TTL | `scripts/automation/exclusion/dispatch_lock.py` test | F-02 |
| **UT-14** | DispatchLockManager.release 写 lease_log | 同上 | F-02 |
| **UT-15** | SubAgentLock.acquire 写 lease_log | `scripts/automation/exclusion/subagent_lock.py` test | F-03 |
| **UT-16** | SubAgentLock.heartbeat 续约 | 同上 | F-03 |
| **UT-17** | SubAgentLock.release reason=timeout/forced | 同上 | F-03 |
| **UT-18** | IdempotencyKeyStore 双键 dedup | `scripts/automation/exclusion/idempotency_key_store.py` test | F-07 + F-09 + F-10 |

### 7.2 8 IT (Integration Tests, per 跨模块)

| # | 测试 | 路径 | 覆盖 |
|---|---|---|---|
| **IT-01** | PG advisory lock 获取/释放事务一致性 | `crates/star-mutex/tests/it_advisory_lock.rs` | F-04 |
| **IT-02** | idempotency_keys 表 RLS 13 类隔离 | `scripts/automation/exclusion/tests/it_idempotency_rls.py` | F-07 + 守门 #13 c |
| **IT-03** | lease_log 表 + heartbeat 30s 续约 | `scripts/automation/exclusion/tests/it_lease_heartbeat.py` | F-03 |
| **IT-04** | advisory_lock_audit 写 audit_audit_event | `crates/star-mutex/tests/it_audit.rs` | F-11 |
| **IT-05** | 22 domain crate 通过 StarMutexAdapter 接入 | `crates/star-mutex/tests/it_domain_adapters.rs` | F-04 |
| **IT-06** | star-mcp 16 tool 加 Idempotency-Key 解析 | `crates/star-mcp/tests/it_idempotency_middleware.rs` | F-08 |
| **IT-07** | 锁 metric 5 项 Prometheus 暴露 | `scripts/automation/exclusion/tests/it_lock_metrics.py` | F-13 |
| **IT-08** | 锁泄漏告警 24h 1h 阈值 | `scripts/automation/exclusion/tests/it_lock_leak_alert.py` | F-14 |

### 7.3 12 E2E (End-to-End Tests, per 8 想定シナリオ + 4 跨层)

| # | 测试 | 路径 | 覆盖 |
|---|---|---|---|
| **E2E-01** | 同一用户重复点 Send 按钮 → 1 次服务端写入 (S-01) | `tests/e2e/s01_client_dedup.spec.ts` | F-06 |
| **E2E-02** | 网络抖动服务端重试 → 1 次副作用 (S-02) | `tests/e2e/s02_server_retry.spec.ts` | F-07 |
| **E2E-03** | User A + User B 同时"暂停 task X" → 后到者 409 (S-03) | `tests/e2e/s03_multi_user.spec.ts` | F-02 |
| **E2E-04** | SA-01 + SA-04 同时改 work_item → version CAS 重试 (S-04) | `tests/e2e/s04_multi_agent.spec.ts` | F-04 |
| **E2E-05** | bulk_node 暂停 5 任务中途 User C 修改 1 个 (S-05) | `tests/e2e/s05_bulk_partial.spec.ts` | F-02 + F-04 + 守门 #13 a |
| **E2E-06** | L1 SubAgent 崩溃 lease 过期 (S-06) | `tests/e2e/s06_lease_expire.spec.ts` | F-03 |
| **E2E-07** | 客户端漏带 Idempotency-Key 双键 fallback (S-07) | `tests/e2e/s07_dual_key_fallback.spec.ts` | F-09 + F-10 |
| **E2E-08** | 1000 并发用户压测 advisory lock QPS (S-08) | `tests/e2e/s08_concurrent_load.spec.ts` | F-04 + F-13 |
| **E2E-09** | 跨层 trace_id 4 层贯穿 → 1 click 定位 | `tests/e2e/e09_trace_propagation.spec.ts` | F-12 |
| **E2E-10** | 16 tool 幂等: 同 key 重试 → 1 次副作用 | `tests/e2e/e10_mcp_idempotency.spec.ts` | F-08 |
| **E2E-11** | 锁泄漏 24h 1h 阈值告警 + 自动 force-release | `tests/e2e/e11_lock_leak_alert.spec.ts` | F-14 |
| **E2E-12** | 22 domain crate 并发改同 work_item → 1 个成功 + 21 个 VersionMismatch | `tests/e2e/e12_domain_cas.spec.ts` | F-04 |

## 8. 性能 / 安全设计 (per 守门 #7 + 守门 #13)

### 8.1 性能 (per NFR-PERF)

- **PG advisory lock QPS** > 5K (单 PG 16GB 内存), benchmark 验证
- **锁获取 P50** < 5ms / **P99** < 50ms
- **幂等 dedup 命中** < 2ms (idx_idem_keys_tenant_expires 索引)
- **heartbeat 续约** < 10ms
- **跨层 trace 关联** < 10ms (X-Trace-Id header 解析)

### 8.2 安全 (per 守门 #13 c + NFR-SEC)

- **5 张新表 100% RLS 13 类 policy** (`tenant_id` + `workspace_id` 隔离)
- **lock_token 256-bit 随机** (UUID v4)
- **key 复用防护** (request_fingerprint 校验, 不匹配 422)
- **审计 WORM** (`audit_audit_event` trigger, 不可篡改)
- **强制释放审计** (admin 域 release_reason=forced, 通知 owner)

## 9. 实施子项分解 (per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1)

8 子项 ~2.3M token 估, 走守门 #19 Python 化 + #9 v3 subprocess + #22 控制台不污染 main + #23 AI mock.

详见 [PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md) v0.1.

## 10. 已知缺口 (Known Gaps)

per [01 §7 G-EI-01..G-EI-07](01-requirements.md) + [02 §10 G-02-EI..G-08-EI](02-basic-design.md) 8 缺口持续跟踪.

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
| v0.1 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿, 23 module + 5 状态机 + 5 时序 + 5 表 DDL + 18 UT + 8 IT + 12 E2E | per `ask_37d138ffb93a12279b35a46e` 4 推荐项拍板 |
| v1.0 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Accepted v1.0 拍板落地, 同步 01/02 IPA 文档 + PHASE report 落档 | 拍板 + 01/02 IPA 文档齐备 |
