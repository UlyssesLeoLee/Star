"""IT-02: 跨模块 idempotency_keys 表 RLS 13 类隔离 (per 守门 #13 c 派生规).

per docs/architecture/2026-09-07-exclusion-idempotency/02-basic-design.md §7.2
per ask_2e8740e6779ac6a8d854c590 拍板 (4 阶段 8 wt 串行)

简化决策 (per brief §9 风险):
- Mock pool 模拟 RLS 13 类隔离行为
- 真实 PG 部署后跨 session 续实证
"""

import asyncio
import unittest
import uuid

from scripts.automation.exclusion.idempotency_key_store import IdempotencyKeyStore


class MockPoolWithRLS:
    """Mock asyncpg.Pool, 模拟 RLS 13 类 tenant 隔离."""

    def __init__(self):
        self.records = {}  # (key, tenant_id) -> row
        self.current_tenant = None  # 模拟 current_setting('app.tenant_id')

    def set_tenant(self, tenant_id: uuid.UUID) -> None:
        self.current_tenant = tenant_id

    async def fetchrow(self, query, *args):
        if "SELECT" in query and "idempotency_keys" in query:
            key, tenant_id = args[0], args[1]
            # RLS: 仅返回 current_tenant 的记录
            if self.current_tenant != tenant_id:
                return None
            row = self.records.get((key, tenant_id))
            if row is None:
                return None
            return {
                "response_status": row.get("response_status"),
                "response_body": row.get("response_body"),
                "request_fingerprint": row["request_fingerprint"],
                "key_type": row["key_type"],
            }
        return None

    async def execute(self, query, *args):
        if "INSERT INTO idempotency_keys" in query:
            key = args[0]
            tenant_id = args[5]
            # RLS: 强制 tenant_id == current_tenant
            if self.current_tenant != tenant_id:
                raise PermissionError(f"RLS: tenant_id mismatch ({self.current_tenant} != {tenant_id})")
            self.records[(key, tenant_id)] = {
                "key": key,
                "key_type": args[1],
                "request_fingerprint": args[2],
                "response_status": args[3],
                "response_body": args[4],
                "tenant_id": tenant_id,
            }
        return None


class TestIdempotencyRLS(unittest.IsolatedAsyncioTestCase):
    """IT-02: idempotency_keys RLS 13 类隔离实证."""

    async def asyncSetUp(self):
        self.pool = MockPoolWithRLS()
        self.store = IdempotencyKeyStore(self.pool)
        self.tenant_a = uuid.uuid4()
        self.tenant_b = uuid.uuid4()
        self.fingerprint = "fp-test"

    async def test_tenant_a_writes_visible_to_tenant_a(self):
        """tenant A 写入, tenant A 读取可见."""
        self.pool.set_tenant(self.tenant_a)
        await self.store.put(
            client_key="key-A",
            business_key=None,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_a,
            workspace_id=uuid.uuid4(),
            actor_id=uuid.uuid4(),
            trace_id=uuid.uuid4(),
            response_status=200,
            response_body={"v": "A"},
        )
        # tenant A 读
        result = await self.store.get("key-A", None, self.fingerprint, self.tenant_a)
        self.assertTrue(result.found)

    async def test_tenant_b_cannot_read_tenant_a(self):
        """tenant B 读 tenant A 不可见 (RLS 隔离)."""
        # tenant A 写入
        self.pool.set_tenant(self.tenant_a)
        await self.store.put(
            client_key="key-A",
            business_key=None,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_a,
            workspace_id=uuid.uuid4(),
            actor_id=uuid.uuid4(),
            trace_id=uuid.uuid4(),
            response_status=200,
            response_body={"v": "A"},
        )

        # tenant B 切换 + 读 tenant A 的 key, 应 NOT found (RLS 隔离)
        self.pool.set_tenant(self.tenant_b)
        result = await self.store.get("key-A", None, self.fingerprint, self.tenant_a)
        self.assertFalse(result.found)  # RLS 隔离, 跨 tenant 不可见

    async def test_tenant_b_cannot_write_to_tenant_a(self):
        """tenant B 写入 tenant A 应抛 PermissionError (RLS 写隔离)."""
        # tenant B 切到自己的 context, 尝试写 tenant A 的数据
        self.pool.set_tenant(self.tenant_b)
        with self.assertRaises(PermissionError):
            await self.store.put(
                client_key="key-A",
                business_key=None,
                request_fingerprint=self.fingerprint,
                tenant_id=self.tenant_a,  # 写 tenant A
                workspace_id=uuid.uuid4(),
                actor_id=uuid.uuid4(),
                trace_id=uuid.uuid4(),
                response_status=200,
                response_body={"v": "B"},
            )


if __name__ == "__main__":
    unittest.main()
