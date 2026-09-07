"""UT-15 + UT-16 + UT-17: SubAgentLock.acquire / heartbeat / release 实证.

per docs/architecture/2026-09-07-exclusion-idempotency/02-basic-design.md §7.1
per ask_2e8740e6779ac6a8d854c590 拍板

守门合规: 守门 #19 v19 Python 化 + 守门 #9 v3 Mavis 直接落地
"""

import asyncio
import unittest
import uuid

from scripts.automation.exclusion.subagent_lock import (
    LeaseHandle,
    ReleaseReason,
    SubAgentLock,
    SubAgentLockHeld,
)


class MockPool:
    """Mock asyncpg.Pool - 内存表模拟 lease_log + heartbeat 续约."""

    def __init__(self):
        self.lease_log = {}  # lock_token -> row dict
        self.return_zero_on_update = False  # 测 "已释放过" 场景

    async def fetchval(self, query, *args):
        if "SELECT 1 FROM lease_log" in query:
            task_id = args[0]
            for row in self.lease_log.values():
                if (row["task_id"] == task_id
                        and row.get("released_at") is None
                        and row.get("expires_at_seconds", 0) > 0):
                    return 1
            return None
        return None

    async def execute(self, query, *args):
        if "INSERT INTO lease_log" in query:
            # SQL: $1=lease_id $2=task_id $3=lock_token $4=expires_interval $5=tenant $6=workspace $7=actor $8=trace
            lock_token = args[2]
            self.lease_log[lock_token] = {
                "lease_id": args[0],
                "task_id": args[1],
                "lock_token": lock_token,
                "expires_at_seconds": int(args[3]) if args[3].isdigit() else 300,
                "released_at": None,
                "release_reason": None,
            }
        elif "UPDATE lease_log" in query and "heartbeat_at" in query:
            lock_token = args[0]
            if lock_token in self.lease_log:
                self.lease_log[lock_token]["heartbeat_at"] = "now"
                self.lease_log[lock_token]["expires_at_seconds"] = 300
        elif "UPDATE lease_log" in query and "released_at" in query:
            lock_token, reason = args[0], args[1]
            if self.return_zero_on_update:
                return "UPDATE 0"
            if lock_token in self.lease_log and self.lease_log[lock_token]["released_at"] is None:
                self.lease_log[lock_token]["released_at"] = "now"
                self.lease_log[lock_token]["release_reason"] = reason
                return "UPDATE 1"
            return "UPDATE 0"
        return None


class TestSubAgentLock(unittest.IsolatedAsyncioTestCase):
    """UT-15 + UT-16 + UT-17."""

    async def asyncSetUp(self):
        self.pool = MockPool()
        self.lock = SubAgentLock(self.pool)

    async def test_acquire_returns_lease(self):
        """UT-15: acquire 返回 LeaseHandle, 300s 过期."""
        task_id = uuid.uuid4()
        subagent_id = "SA-01"
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease = await self.lock.acquire(task_id, subagent_id, actor_id, trace_id)

        self.assertIsInstance(lease, LeaseHandle)
        self.assertEqual(lease.task_id, task_id)
        self.assertEqual(lease.subagent_id, subagent_id)
        # 300s 过期 (per F-03)
        self.assertAlmostEqual(lease.expires_at - lease.heartbeat_at, 300.0, delta=1.0)
        # 写 lease_log 验证
        self.assertIn(lease.lock_token, self.pool.lease_log)
        # 默认参数
        self.assertEqual(self.lock.DEFAULT_HEARTBEAT_S, 30)
        self.assertEqual(self.lock.DEFAULT_EXPIRES_S, 300)

    async def test_acquire_locked_raises(self):
        """UT-15: 同 task 第二次 acquire 抛 SubAgentLockHeld (per F-03 + S-06)."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease1 = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        self.assertIsNotNone(lease1)

        # 第二次同 task 抛 SubAgentLockHeld
        with self.assertRaises(SubAgentLockHeld) as ctx:
            await self.lock.acquire(task_id, "SA-04", actor_id, trace_id)
        self.assertEqual(ctx.exception.task_id, task_id)
        self.assertEqual(ctx.exception.subagent_id, "SA-04")

    async def test_heartbeat_renews_lease(self):
        """UT-16: heartbeat 续约 lease, 延长 expires_at 300s."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        old_expires = lease.expires_at

        # 等 0.05s 后 heartbeat
        await asyncio.sleep(0.05)
        await self.lock.heartbeat(lease)

        # expires_at 应延长
        self.assertGreater(lease.expires_at, old_expires)
        # lease_log.heartbeat_at 已更新
        self.assertEqual(self.pool.lease_log[lease.lock_token]["heartbeat_at"], "now")

    async def test_release_normal(self):
        """UT-17: release 写 lease_log.released_at + reason=normal."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        await self.lock.release(lease, reason=ReleaseReason.NORMAL)

        row = self.pool.lease_log[lease.lock_token]
        self.assertEqual(row["released_at"], "now")
        self.assertEqual(row["release_reason"], "normal")

    async def test_release_forced(self):
        """UT-17: admin 域强制释放, reason=forced."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        await self.lock.release(lease, reason=ReleaseReason.FORCED)

        row = self.pool.lease_log[lease.lock_token]
        self.assertEqual(row["release_reason"], "forced")

    async def test_release_twice_raises(self):
        """UT-17: 重复 release 抛 ValueError (防双释放)."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        await self.lock.release(lease)

        # 第二次 release 应抛 ValueError
        with self.assertRaises(ValueError):
            await self.lock.release(lease)

    async def test_acquire_after_release(self):
        """UT-15: release 后允许同 task 重新 acquire (per 守门 #13 a L0 协调闭环)."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease1 = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        await self.lock.release(lease1)

        # 释放后可重新 acquire 同 task
        lease2 = await self.lock.acquire(task_id, "SA-04", actor_id, trace_id)
        self.assertNotEqual(lease1.lease_id, lease2.lease_id)
        self.assertEqual(lease2.subagent_id, "SA-04")

    async def tearDown(self):
        """清理 heartbeat 后台任务."""
        await self.lock.stop()


if __name__ == "__main__":
    unittest.main()
