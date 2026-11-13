# @cypher schema=1 source_sha256=2dc5415f6f9e23be85f81debd06c09cec8fb665097c39ae929e7f354eae14dba
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/tests/test_ut_29_quota_registry.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/tests/test_ut_29_quota_registry.py
# UT-29: SubTaskQuotaRegistry.query / increment / decrement (per 03-detailed §4.4)
# + SCD Type 2 留痕 (对齐 003_task_quota_config.sql 语义)

from __future__ import annotations

import unittest

from scripts.automation.task_ops.compat import QuotaConfigStore
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry


class TestSubTaskQuotaRegistry(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        QuotaConfigStore.reset()

    async def test_query_default_when_no_config(self) -> None:
        # 无配置 → 默认 max=2 (per F-31)
        self.assertEqual(2, await SubTaskQuotaRegistry.query("task-12345678"))

    async def test_query_skipped_returns_sentinel(self) -> None:
        # admin override (per F-26 / DD §3.3.4): quota_check=False → 999
        self.assertEqual(999, await SubTaskQuotaRegistry.query("task-12345678", quota_check=False))

    async def test_increment_then_query(self) -> None:
        await SubTaskQuotaRegistry.increment("task-12345678")
        self.assertEqual(1, await SubTaskQuotaRegistry.query("task-12345678"))
        await SubTaskQuotaRegistry.increment("task-12345678")
        self.assertEqual(0, await SubTaskQuotaRegistry.query("task-12345678"))

    async def test_decrement_restores_quota(self) -> None:
        await SubTaskQuotaRegistry.increment("task-12345678")
        await SubTaskQuotaRegistry.increment("task-12345678")
        await SubTaskQuotaRegistry.decrement("task-12345678")
        self.assertEqual(1, await SubTaskQuotaRegistry.query("task-12345678"))

    async def test_decrement_never_negative(self) -> None:
        await SubTaskQuotaRegistry.decrement("task-12345678")
        row = QuotaConfigStore.current("task-12345678")
        self.assertIsNone(row)  # 无行 → 不创建 (query 走默认值)

    async def test_scd_type2_history_preserved(self) -> None:
        # SCD Type 2: 每次 increment = close 旧行 + insert 新行 (per 003 触发器语义);
        # 无配置起点 → 第 1 次 increment 直接建新行, 第 2 次起才 close 旧行.
        await SubTaskQuotaRegistry.increment("task-12345678")
        await SubTaskQuotaRegistry.increment("task-12345678")
        history = QuotaConfigStore.history("task-12345678")
        self.assertEqual(1, len(history))
        for closed in history:
            self.assertFalse(closed["is_current"])
            self.assertIsNotNone(closed["effective_to"])
            self.assertEqual(1, closed["current_subtasks"])
        current = QuotaConfigStore.current("task-12345678")
        assert current is not None
        self.assertTrue(current["is_current"])
        self.assertIsNone(current["effective_to"])
        self.assertEqual(2, current["current_subtasks"])
        self.assertEqual(2, current["max_subtasks"])

    async def test_custom_config_respected(self) -> None:
        QuotaConfigStore.upsert_scd2("task-12345678", max_subtasks=5, current_subtasks=3)
        self.assertEqual(2, await SubTaskQuotaRegistry.query("task-12345678"))

    async def test_get_token_budget_default(self) -> None:
        budget = await SubTaskQuotaRegistry.get_token_budget("st-child-0000000001")
        self.assertEqual({"per_subtask_token_budget": 100_000}, budget)


if __name__ == "__main__":
    unittest.main()
