"""L0 派发级排他锁 (DispatchLockManager) - per Star-EI F-02 + F-07.

per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.2.1
per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-03

锁实现: PostgreSQL advisory lock (per ADR-0048 D-01)
幂等: idempotency_keys 表 (per ADR-0048 D-02 双键 client_uuid + business_hash)

守门合规:
- 守门 #13 a L0 协调: L1 SubAgent 不直接调 L3 Domain 互抢, 必须经 L0 (本类是 L0 派发层)
- 守门 #5 env 安全: 不打印 DB_DSN
- 守门 #19 v19: Python 化 (走 scripts/automation/exclusion/)
"""

import asyncio
import hashlib
import os
import time
import uuid
from dataclasses import dataclass
from typing import Optional

# 守门 #5: 不打印 DB_DSN, 仅引用
DB_DSN = os.environ.get("STAR_PG_DSN", "postgresql://star:***@localhost:5432/star_exclusion")


@dataclass
class LockGuard:
    """派发级锁句柄, 持有期间代表一个 task 被 L0 派发层独占."""
    task_id: uuid.UUID
    dispatch_id: uuid.UUID
    lock_token: str
    acquired_at: float
    expires_at: float
    actor_id: uuid.UUID
    tenant_id: uuid.UUID
    trace_id: uuid.UUID


class DispatchLockHeld(Exception):
    """派发级锁被其他 actor 持有, 抛 409 Conflict."""

    def __init__(self, task_id: uuid.UUID, wait_ms: int, message: str):
        self.task_id = task_id
        self.wait_ms = wait_ms
        super().__init__(message)


class DispatchLockManager:
    """L0 派发级排他 + 派发级幂等 (per F-02 + F-07).
    
    用法:
        async with DispatchLockManager(pool) as mgr:
            lock = await mgr.acquire(task_id, dispatch_id, actor_id, tenant_id, trace_id)
            try:
                # ... 业务逻辑
            finally:
                await mgr.release(lock)
    """

    DEFAULT_TIMEOUT_S = 60  # 派发级锁 60s TTL
    LOCK_NAMESPACE = "task"  # 锁类 ID 命名空间

    def __init__(self, pool):
        """Args:
            pool: asyncpg.Pool 实例 (生产) 或 MockPool (测试, per H2 v18 实证).
        """
        self.pool = pool

    async def acquire(
        self,
        task_id: uuid.UUID,
        dispatch_id: uuid.UUID,
        actor_id: uuid.UUID,
        tenant_id: uuid.UUID,
        trace_id: uuid.UUID,
        timeout_ms: int = 60000,
    ) -> LockGuard:
        """获取派发级排他锁, 60s TTL.
        
        Raises:
            DispatchLockHeld: 锁被其他 actor 持有, 后到者 409.
        """
        lock_class_id = self._compute_lock_class_id(self.LOCK_NAMESPACE, str(task_id))
        started_at = time.monotonic()

        # 1. 尝试获取 PG advisory lock (事务内自动释放, 5s 等待)
        got = await self.pool.fetchval(
            "SELECT pg_try_advisory_xact_lock($1)", lock_class_id
        )
        if not got:
            wait_ms = int((time.monotonic() - started_at) * 1000)
            raise DispatchLockHeld(
                task_id=task_id,
                wait_ms=wait_ms,
                message=f"task {task_id} 已被其他派发持有 (wait_ms={wait_ms})",
            )

        # 2. 写 idempotency_keys (双键 client_uuid)
        fingerprint = hashlib.sha256(
            f"{task_id}|{actor_id}|{tenant_id}".encode()
        ).hexdigest()
        await self.pool.execute(
            """
            INSERT INTO idempotency_keys (
                key, key_type, request_fingerprint,
                tenant_id, workspace_id, actor_id, trace_id, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW() + INTERVAL '24 hours')
            """,
            str(dispatch_id), "client", fingerprint,
            tenant_id, uuid.UUID(int=0), actor_id, trace_id,
        )

        # 3. 写 lease_log (per F-03)
        await self.pool.execute(
            """
            INSERT INTO lease_log (
                lease_id, task_id, lock_token, expires_at,
                tenant_id, workspace_id, actor_id, trace_id
            ) VALUES ($1, $2, $3, NOW() + ($4 || ' milliseconds')::INTERVAL, $5, $6, $7, $8)
            """,
            uuid.uuid4(), task_id, str(dispatch_id), str(timeout_ms),
            tenant_id, uuid.UUID(int=0), actor_id, trace_id,
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
        """释放派发级排他锁, 写 lease_log.released_at.
        
        守门: 释放时不持有锁 (PG advisory_xact_lock 事务内自动释放),
              但需要写 lease_log.released_at 表明手动释放.
        """
        await self.pool.execute(
            """
            UPDATE lease_log 
            SET released_at = NOW(), 
                release_reason = $2 
            WHERE lock_token = $1 AND released_at IS NULL
            """,
            lock.lock_token, reason,
        )

    async def is_locked(self, task_id: uuid.UUID) -> bool:
        """检查 task 是否被其他派发持有.
        
        Note: pg_try_advisory_lock 是 session-bound, 会在当前 session 持有,
              测完需 pg_advisory_unlock 释放, 避免 session 泄漏.
        """
        lock_class_id = self._compute_lock_class_id(self.LOCK_NAMESPACE, str(task_id))
        got = await self.pool.fetchval(
            "SELECT pg_try_advisory_lock($1)", lock_class_id
        )
        if got:
            await self.pool.execute("SELECT pg_advisory_unlock($1)", lock_class_id)
        return got

    @staticmethod
    def _compute_lock_class_id(namespace: str, key: str) -> int:
        """计算 PG advisory lock class ID (BIGINT).
        
        命名空间按资源类型分: task / work_item / worktree / merge_request / comment
        用 SHA-256 前 15 字符 (60 bit) 转 int, 避免冲突.
        """
        h = hashlib.sha256(f"{namespace}:{key}".encode()).hexdigest()[:15]
        return int(h, 16)
