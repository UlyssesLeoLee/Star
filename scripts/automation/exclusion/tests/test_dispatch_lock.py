"""UT-13 + UT-14: DispatchLockManager.acquire / release 60s TTL 实证.

per docs/architecture/2026-09-07-exclusion-idempotency/02-basic-design.md §7.1
per ask_2e8740e6779ac6a8d854c590 拍板 (阶段 2 = EX-02/03/04 3 wt 并行)

守门合规: 守门 #19 v19 Python 化 + 守门 #9 v3 Mavis 直接落地 + 守门 #11 缺标比错标

本测试用 MockPool (per H2 v18 实证 5/5 subagent RPC 不可靠, mock 是稳的选择)
"""

import asyncio
import unittest
import uuid

from scripts.automation.exclusion.dispatch_lock import (
    DispatchLockHeld,
    DispatchLockManager,
    LockGuard,
)


class MockPool:
    """Mock asyncpg.Pool - 内存表模拟 5 张新表, per H2 v18 实证."""

    def __init__(self):
        self.idempotency_keys = {}  # key -> row dict
        self.lease_log = {}  # lock_token -> row dict
        self.advisory_locks_held = set()  # lock_class_id 集合
        self.fail_next_acquire = False  # 测试用 flag

    async def fetchval(self, query, *args):
        if "pg_try_advisory_xact_lock" in query or "pg_try_advisory_lock" in query:
            if self.fail_next_acquire:
                self.fail_next_acquire = False
                return False
            lock_class_id = args[0]
            if lock_class_id in self.advisory_locks_held:
                return False
            self.advisory_locks_held.add(lock_class_id)
            return True
        return None

    async def execute(self, query, *args):
        if "INSERT INTO idempotency_keys" in query:
            key = args[0]
            tenant_id = args[3]  # $4 = tenant_id (per SQL: $1=key $2=key_type $3=fingerprint $4=tenant_id)
            self.idempotency_keys[(key, tenant_id)] = {
                "key": key,
                "key_type": args[1],
                "request_fingerprint": args[2],
                "tenant_id": tenant_id,
            }
        elif "INSERT INTO lease_log" in query:
            lock_token = args[2]
            # SQL: $1=lease_id $2=task_id $3=lock_token $4=expires_interval $5=tenant_id $6=workspace $7=actor $8=trace
            self.lease_log[lock_token] = {
                "lease_id": args[0],
                "task_id": args[1],
                "lock_token": lock_token,
                "expires_at": args[3],
                "tenant_id": args[4],
            }
        elif "UPDATE lease_log" in query and "released_at" in query:
            lock_token, reason = args[0], args[1]
            if lock_token in self.lease_log:
                self.lease_log[lock_token]["released_at"] = "now"
                self.lease_log[lock_token]["release_reason"] = reason
                # 释放 advisory lock (per xact_lock 事务结束自动释放)
                lock_class_id = DispatchLockManager._compute_lock_class_id(
                    DispatchLockManager.LOCK_NAMESPACE, str(self.lease_log[lock_token]["task_id"])
                )
                self.advisory_locks_held.discard(lock_class_id)
        elif "pg_advisory_unlock" in query:
            lock_class_id = args[0]
            self.advisory_locks_held.discard(lock_class_id)
        return None


class TestDispatchLockManager(unittest.IsolatedAsyncioTestCase):
    """UT-13 + UT-14."""

    async def asyncSetUp(self):
        self.pool = MockPool()
        self.mgr = DispatchLockManager(self.pool)

    async def test_acquire_returns_lockguard(self):
        """UT-13: acquire 返回 LockGuard, 60s TTL."""
        task_id = uuid.uuid4()
        dispatch_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        tenant_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lock = await self.mgr.acquire(
            task_id, dispatch_id, actor_id, tenant_id, trace_id
        )

        self.assertIsInstance(lock, LockGuard)
        self.assertEqual(lock.task_id, task_id)
        self.assertEqual(lock.dispatch_id, dispatch_id)
        self.assertEqual(lock.lock_token, str(dispatch_id))
        self.assertEqual(lock.actor_id, actor_id)
        # 60s TTL 验证 (per F-02)
        self.assertAlmostEqual(lock.expires_at - lock.acquired_at, 60.0, delta=1.0)
        # 写 idempotency_keys + lease_log 验证
        self.assertIn((str(dispatch_id), tenant_id), self.pool.idempotency_keys)
        self.assertIn(str(dispatch_id), self.pool.lease_log)

    async def test_acquire_locked_raises(self):
        """UT-13: 同 task 第二次 acquire 抛 DispatchLockHeld (409 Conflict)."""
        task_id = uuid.uuid4()
        dispatch_id_1 = uuid.uuid4()
        dispatch_id_2 = uuid.uuid4()
        actor_id = uuid.uuid4()
        tenant_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        # 第一次获取成功
        await self.mgr.acquire(
            task_id, dispatch_id_1, actor_id, tenant_id, trace_id
        )

        # 第二次同 task 抛 DispatchLockHeld
        with self.assertRaises(DispatchLockHeld) as ctx:
            await self.mgr.acquire(
                task_id, dispatch_id_2, actor_id, tenant_id, trace_id
            )
        self.assertEqual(ctx.exception.task_id, task_id)

    async def test_release_writes_lease_log(self):
        """UT-14: release 写 lease_log.released_at + reason=normal."""
        task_id = uuid.uuid4()
        dispatch_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        tenant_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lock = await self.mgr.acquire(
            task_id, dispatch_id, actor_id, tenant_id, trace_id
        )
        await self.mgr.release(lock, reason="normal")

        # 验证 lease_log.released_at + reason
        row = self.pool.lease_log[str(dispatch_id)]
        self.assertEqual(row["released_at"], "now")
        self.assertEqual(row["release_reason"], "normal")

    async def test_release_advices_lock(self):
        """UT-14: release 释放 advisory lock, 允许同 task 重新 acquire."""
        task_id = uuid.uuid4()
        dispatch_id_1 = uuid.uuid4()
        dispatch_id_2 = uuid.uuid4()
        actor_id = uuid.uuid4()
        tenant_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lock1 = await self.mgr.acquire(
            task_id, dispatch_id_1, actor_id, tenant_id, trace_id
        )
        await self.mgr.release(lock1)

        # 释放后可重新 acquire 同 task
        lock2 = await self.mgr.acquire(
            task_id, dispatch_id_2, actor_id, tenant_id, trace_id
        )
        self.assertNotEqual(lock1.dispatch_id, lock2.dispatch_id)

    async def test_compute_lock_class_id_deterministic(self):
        """UT-13: _compute_lock_class_id 对相同 namespace+key 返回相同 ID."""
        id1 = DispatchLockManager._compute_lock_class_id("task", "task-uuid-1")
        id2 = DispatchLockManager._compute_lock_class_id("task", "task-uuid-1")
        id3 = DispatchLockManager._compute_lock_class_id("task", "task-uuid-2")

        self.assertEqual(id1, id2)
        self.assertNotEqual(id1, id3)
        # BIGINT 范围
        self.assertGreater(id1, 0)
        self.assertLess(id1, 2**60)


if __name__ == "__main__":
    unittest.main()
