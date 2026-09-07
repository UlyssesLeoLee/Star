"""UT-18: IdempotencyKeyStore 双键 dedup.

per docs/architecture/2026-09-07-exclusion-idempotency/02-basic-design.md §7.1
per ADR-0048 D-02 双键 (client_uuid + business_hash)
per ask_2e8740e6779ac6a8d854c590 拍板

守门合规: 守门 #19 v19 Python 化 + 守门 #9 v3 Mavis 直接落地
"""

import unittest
import uuid
import json

from scripts.automation.exclusion.idempotency_key_store import (
    IdempotencyKeyMissing,
    IdempotencyKeyReuse,
    IdempotencyKeyStore,
)


class MockPool:
    """Mock asyncpg.Pool - 内存表模拟 idempotency_keys 表."""

    def __init__(self):
        self.idempotency_keys = {}  # (key, tenant_id) -> row dict

    async def fetchrow(self, query, *args):
        if "SELECT" in query and "idempotency_keys" in query:
            key, tenant_id = args[0], args[1]
            row = self.idempotency_keys.get((key, tenant_id))
            if row is None:
                return None
            # 模拟 expires_at > NOW() 检查
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
            self.idempotency_keys[(key, tenant_id)] = {
                "key": key,
                "key_type": args[1],
                "request_fingerprint": args[2],
                "response_status": args[3],
                "response_body": args[4],
                "tenant_id": tenant_id,
            }
        return None


class TestIdempotencyKeyStore(unittest.IsolatedAsyncioTestCase):
    """UT-18 双键 dedup."""

    async def asyncSetUp(self):
        self.pool = MockPool()
        self.store = IdempotencyKeyStore(self.pool)
        self.tenant_id = uuid.uuid4()
        self.workspace_id = uuid.uuid4()
        self.actor_id = uuid.uuid4()
        self.trace_id = uuid.uuid4()
        self.fingerprint = "test-fingerprint-v1"

    async def test_compute_business_hash_deterministic(self):
        """业务主键 hash 跟 (tenant, resource, op, version) 一一对应."""
        h1 = IdempotencyKeyStore.compute_business_hash(
            self.tenant_id, "work_item", "uuid-1", "update_status", 5
        )
        h2 = IdempotencyKeyStore.compute_business_hash(
            self.tenant_id, "work_item", "uuid-1", "update_status", 5
        )
        h3 = IdempotencyKeyStore.compute_business_hash(
            self.tenant_id, "work_item", "uuid-1", "update_status", 6  # version 不同
        )
        self.assertEqual(h1, h2)
        self.assertNotEqual(h1, h3)

    async def test_get_missing_key_raises(self):
        """双键都缺失抛 IdempotencyKeyMissing (per F-10)."""
        with self.assertRaises(IdempotencyKeyMissing):
            await self.store.get(
                client_key=None,
                business_key=None,
                request_fingerprint=self.fingerprint,
                tenant_id=self.tenant_id,
            )

    async def test_put_then_get_dedup(self):
        """put 后 get 命中, is_dedup=True, 返回首次响应."""
        client_key = "client-uuid-1"
        response_body = {"result": "ok", "id": 42}

        # 1. 首次 put
        await self.store.put(
            client_key=client_key,
            business_key=None,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_id,
            workspace_id=self.workspace_id,
            actor_id=self.actor_id,
            trace_id=self.trace_id,
            response_status=200,
            response_body=response_body,
        )

        # 2. 第二次 get 应命中
        result = await self.store.get(
            client_key=client_key,
            business_key=None,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_id,
        )
        self.assertTrue(result.found)
        self.assertTrue(result.is_dedup)
        self.assertEqual(result.response_status, 200)
        self.assertEqual(result.response_body, response_body)

    async def test_reuse_different_fingerprint_raises(self):
        """key 复用但 fingerprint 不同抛 IdempotencyKeyReuse (per F-07)."""
        client_key = "client-uuid-2"

        # 首次 put 用 fingerprint-v1
        await self.store.put(
            client_key=client_key,
            business_key=None,
            request_fingerprint="fingerprint-v1",
            tenant_id=self.tenant_id,
            workspace_id=self.workspace_id,
            actor_id=self.actor_id,
            trace_id=self.trace_id,
            response_status=200,
            response_body={"v": 1},
        )

        # 第二次 get 用 fingerprint-v2 应抛 IdempotencyKeyReuse
        with self.assertRaises(IdempotencyKeyReuse):
            await self.store.get(
                client_key=client_key,
                business_key=None,
                request_fingerprint="fingerprint-v2",
                tenant_id=self.tenant_id,
            )

    async def test_business_key_fallback(self):
        """client_key 缺失 fallback business_key (per F-09)."""
        business_hash = IdempotencyKeyStore.compute_business_hash(
            self.tenant_id, "work_item", "uuid-x", "update_status", 1
        )

        # 首次 put 用 business_key
        await self.store.put(
            client_key=None,
            business_key=business_hash,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_id,
            workspace_id=self.workspace_id,
            actor_id=self.actor_id,
            trace_id=self.trace_id,
            response_status=200,
            response_body={"v": "biz"},
        )

        # 第二次 get 用 business_key (无 client_key) 应命中
        result = await self.store.get(
            client_key=None,
            business_key=business_hash,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_id,
        )
        self.assertTrue(result.found)
        self.assertEqual(result.key_type, "business")
        self.assertEqual(result.response_body, {"v": "biz"})

    async def test_dual_key_type(self):
        """双键都提供时, key_type=dual."""
        client_key = "client-uuid-3"
        business_hash = IdempotencyKeyStore.compute_business_hash(
            self.tenant_id, "work_item", "uuid-y", "create", 1
        )

        await self.store.put(
            client_key=client_key,
            business_key=business_hash,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_id,
            workspace_id=self.workspace_id,
            actor_id=self.actor_id,
            trace_id=self.trace_id,
            response_status=201,
            response_body={"created": True},
        )

        # client_key 查询应命中, key_type=dual
        result = await self.store.get(
            client_key=client_key,
            business_key=business_hash,
            request_fingerprint=self.fingerprint,
            tenant_id=self.tenant_id,
        )
        self.assertTrue(result.found)
        self.assertEqual(result.key_type, "dual")


if __name__ == "__main__":
    unittest.main()
