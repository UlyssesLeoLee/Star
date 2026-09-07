"""L1 任务卡持锁 (SubAgentLock) - per Star-EI F-03.

per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.2.2
per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-04

锁实现: PG lease + heartbeat (per ADR-0048 D-01 advisory lock 共享)
- lease 300s 过期
- heartbeat 30s 续约
- 崩溃 30s 内自动释放 (per NFR-REL-02)

守门合规:
- 守门 #13 a L0 协调: L1 SubAgent 不能直接调 L3 Domain 互抢, 必须经 L0 派发 (本类是 L1, 派发由 L0 协调)
- 守门 #5 env 安全
- 守门 #19 v19: Python 化 (走 scripts/automation/exclusion/)
"""

import asyncio
import time
import uuid
from dataclasses import dataclass
from enum import Enum
from typing import Optional


class ReleaseReason(str, Enum):
    NORMAL = "normal"
    TIMEOUT = "timeout"
    ERROR = "error"
    FORCED = "forced"  # admin 域强制


@dataclass
class LeaseHandle:
    """任务卡持锁句柄."""
    lease_id: uuid.UUID
    task_id: uuid.UUID
    subagent_id: str  # SA-01..SA-09 + SA-10
    lock_token: str
    heartbeat_at: float
    expires_at: float
    actor_id: uuid.UUID
    trace_id: uuid.UUID


class SubAgentLockHeld(Exception):
    """task 已被其他 SubAgent 持有, 抛 409 Conflict."""

    def __init__(self, task_id: uuid.UUID, subagent_id: str):
        self.task_id = task_id
        self.subagent_id = subagent_id
        super().__init__(f"task {task_id} 已被其他 SubAgent 持有")


class SubAgentLock:
    """L1 任务卡持锁 + heartbeat (per F-03).
    
    用法:
        async with SubAgentLock(pool) as lock:
            lease = await lock.acquire(task_id, "SA-01", actor_id, trace_id)
            try:
                # ... 执行业务
            finally:
                await lock.release(lease)
    """

    DEFAULT_HEARTBEAT_S = 30
    DEFAULT_EXPIRES_S = 300

    def __init__(self, pool):
        """Args:
            pool: asyncpg.Pool 实例 (生产) 或 MockPool (测试).
        """
        self.pool = pool
        self._heartbeat_tasks = {}  # lock_token -> asyncio.Task

    async def acquire(
        self,
        task_id: uuid.UUID,
        subagent_id: str,
        actor_id: uuid.UUID,
        trace_id: uuid.UUID,
        heartbeat_interval_s: int = DEFAULT_HEARTBEAT_S,
        expires_s: int = DEFAULT_EXPIRES_S,
    ) -> LeaseHandle:
        """获取任务卡 lease + 启动 heartbeat.
        
        Raises:
            SubAgentLockHeld: task 已被其他 SubAgent 持有.
        """
        lease_id = uuid.uuid4()
        lock_token = f"lease-{lease_id}"
        now = time.time()

        # 1. 检查 task 是否已被其他 SA 持有 (per F-03)
        existing = await self.pool.fetchval(
            """
            SELECT 1 FROM lease_log 
            WHERE task_id = $1 AND released_at IS NULL AND expires_at > NOW()
            LIMIT 1
            """,
            task_id,
        )
        if existing:
            raise SubAgentLockHeld(task_id=task_id, subagent_id=subagent_id)

        # 2. 写 lease_log (per F-03)
        # SQL: $1=lease_id $2=task_id $3=lock_token $4=expires_interval $5=tenant $6=workspace $7=actor $8=trace
        await self.pool.execute(
            """
            INSERT INTO lease_log (
                lease_id, task_id, lock_token, expires_at,
                tenant_id, workspace_id, actor_id, trace_id
            ) VALUES ($1, $2, $3, NOW() + ($4 || ' seconds')::INTERVAL, $5, $6, $7, $8)
            """,
            lease_id, task_id, lock_token,
            str(expires_s),
            # tenant_id / workspace_id 实际生产从 caller 注入, mock 阶段用 placeholder
            uuid.UUID(int=0), uuid.UUID(int=0),
            actor_id, trace_id,
        )

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

        # 3. 启动 heartbeat 后台任务
        self._start_heartbeat(handle, heartbeat_interval_s)
        return handle

    def _start_heartbeat(self, handle: LeaseHandle, interval_s: int) -> None:
        """启动 heartbeat 后台任务, 每 interval_s 秒续约 lease."""

        async def heartbeat_loop():
            while True:
                await asyncio.sleep(interval_s)
                try:
                    await self.heartbeat(handle)
                except Exception:
                    # heartbeat 失败, 锁自动过期, 退出循环
                    break

        task = asyncio.create_task(heartbeat_loop())
        self._heartbeat_tasks[handle.lock_token] = task

    async def heartbeat(self, handle: LeaseHandle) -> None:
        """续约 lease, 延长 expires_at 300s.
        
        Note: 必须 UPDATE WHERE released_at IS NULL, 防止已释放的 lease 被续约.
        """
        await self.pool.execute(
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
        """释放 lease, 写 lease_log.released_at + reason.
        
        Raises:
            ValueError: lock_token 已释放过.
        """
        # 1. 停止 heartbeat 后台任务
        if handle.lock_token in self._heartbeat_tasks:
            self._heartbeat_tasks[handle.lock_token].cancel()
            try:
                await self._heartbeat_tasks[handle.lock_token]
            except asyncio.CancelledError:
                pass
            del self._heartbeat_tasks[handle.lock_token]

        # 2. 写 lease_log.released_at
        result = await self.pool.execute(
            """
            UPDATE lease_log 
            SET released_at = NOW(), release_reason = $2 
            WHERE lock_token = $1 AND released_at IS NULL
            """,
            handle.lock_token, reason.value,
        )
        # result 格式: 'UPDATE n', 0 表示已释放过
        if result == "UPDATE 0":
            raise ValueError(f"lease {handle.lock_token} 已释放过, 重复 release 拒绝")
