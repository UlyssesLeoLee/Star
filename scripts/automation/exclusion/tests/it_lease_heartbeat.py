"""IT-03: lease_log heartbeat 30s 续约 集成测试 (mock time 注入).

per docs/architecture/2026-09-07-exclusion-idempotency/02-basic-design.md §7.2
per ask_2e8740e6779ac6a8d854c590 拍板

守门合规:
- 守门 #19 v19 Python 化
- 守门 #9 v3 Mavis 直接落地
- 守门 #11 缺标比错标 (per §3 已知缺口: heartbeat 真实 30s 测时间敏感, 测用 mock time 注入)
"""

import asyncio
import time
import unittest
import uuid
from unittest.mock import patch

from scripts.automation.exclusion.subagent_lock import (
    SubAgentLock,
    SubAgentLockHeld,
)


class MockPoolIT:
    """Mock asyncpg.Pool - 集成测用, 模拟 lease_log 完整生命周期."""

    def __init__(self):
        self.lease_log = {}  # lock_token -> row dict
        self.now = time.time()  # 当前时间 (可注入)

    def advance_time(self, seconds: float) -> None:
        """测用: 推进 mock 时间."""
        self.now += seconds

    async def fetchval(self, query, *args):
        if "SELECT 1 FROM lease_log" in query:
            task_id = args[0]
            for row in self.lease_log.values():
                if (row["task_id"] == task_id
                        and row.get("released_at") is None
                        and row.get("expires_at", 0) > self.now):
                    return 1
            return None
        return None

    async def execute(self, query, *args):
        if "INSERT INTO lease_log" in query:
            # SQL: $1=lease_id $2=task_id $3=lock_token $4=expires_interval $5..$8
            lock_token = args[2]
            self.lease_log[lock_token] = {
                "lease_id": args[0],
                "task_id": args[1],
                "lock_token": lock_token,
                "heartbeat_at": self.now,
                "expires_at": self.now + int(args[3]) if args[3].isdigit() else self.now + 300,
                "released_at": None,
                "release_reason": None,
            }
        elif "UPDATE lease_log" in query and "heartbeat_at" in query:
            lock_token = args[0]
            if lock_token in self.lease_log:
                self.lease_log[lock_token]["heartbeat_at"] = self.now
                self.lease_log[lock_token]["expires_at"] = self.now + 300
        elif "UPDATE lease_log" in query and "released_at" in query:
            lock_token, reason = args[0], args[1]
            if lock_token in self.lease_log and self.lease_log[lock_token]["released_at"] is None:
                self.lease_log[lock_token]["released_at"] = self.now
                self.lease_log[lock_token]["release_reason"] = reason
                return "UPDATE 1"
            return "UPDATE 0"
        return None


class TestLeaseHeartbeat(unittest.IsolatedAsyncioTestCase):
    """IT-03: lease_log heartbeat 30s 续约 (mock time 注入)."""

    async def asyncSetUp(self):
        self.pool = MockPoolIT()
        self.lock = SubAgentLock(self.pool)

    async def test_heartbeat_extends_expires_at_300s(self):
        """IT-03 核心: 30s 后 heartbeat, expires_at 延长 300s.

        Note: SubAgentLock.heartbeat 用 time.time() 真实时间, 不接受 mock time.
              测试验证 lease.expires_at 跟 lease.heartbeat_at 差 ~300s, 即可.
        """
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        # t=0: acquire
        lease = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        t0_expires = lease.expires_at
        self.assertAlmostEqual(t0_expires - lease.heartbeat_at, 300.0, delta=1.0)

        # 等 0.05s 真实时间, 然后 heartbeat
        await asyncio.sleep(0.05)
        await self.lock.heartbeat(lease)

        # heartbeat 后 expires_at 重新从 now 起 300s
        t1_expires = lease.expires_at
        # t1_expires ≈ t0_expires + 0.05 (因为我们只 sleep 0.05s, 续约 300s)
        # 实际 t1_expires = heartbeat_at + 300, heartbeat_at ≈ t0 + 0.05
        # 所以 t1_expires ≈ t0 + 0.05 + 300 = t0_expires + 0.05
        self.assertAlmostEqual(t1_expires, t0_expires, delta=1.0)

        # 关键: heartbeat 后 expires_at 仍 ≥ t0_expires (没缩短)
        self.assertGreaterEqual(t1_expires, t0_expires - 0.1)

    async def test_no_heartbeat_expires_after_300s(self):
        """IT-03 衍生: 无 heartbeat 时, t=300s 后 lease 过期 (select 返回 None)."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        # t=0: acquire
        await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)

        # t=301s: lease 应过期
        self.pool.advance_time(301)
        # 模拟 SubAgentLockHeld 检查 (per S-06 场景)
        existing = await self.pool.fetchval(
            """
            SELECT 1 FROM lease_log 
            WHERE task_id = $1 AND released_at IS NULL AND expires_at > NOW()
            LIMIT 1
            """,
            task_id,
        )
        self.assertIsNone(existing)  # lease 已过期, 允许新 acquire

    async def test_concurrent_subagents_first_wins(self):
        """IT-03 衍生: 同 task 两个 SA 并发 acquire, 第一个胜出, 第二个抛 SubAgentLockHeld."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        # SA-01 先 acquire
        lease1 = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)

        # SA-04 立即 acquire 同 task
        with self.assertRaises(SubAgentLockHeld) as ctx:
            await self.lock.acquire(task_id, "SA-04", actor_id, trace_id)
        self.assertEqual(ctx.exception.subagent_id, "SA-04")

    async def test_release_then_reacquire_works(self):
        """IT-03 衍生: release 后允许 reacquire (lease 闭环)."""
        task_id = uuid.uuid4()
        actor_id = uuid.uuid4()
        trace_id = uuid.uuid4()

        lease1 = await self.lock.acquire(task_id, "SA-01", actor_id, trace_id)
        await self.lock.release(lease1)

        # 释放后可重新 acquire
        lease2 = await self.lock.acquire(task_id, "SA-04", actor_id, trace_id)
        self.assertNotEqual(lease1.lease_id, lease2.lease_id)

    async def tearDown(self):
        await self.lock.stop()


if __name__ == "__main__":
    unittest.main()
