# @cypher schema=1 source_sha256=95ad2efac8e44bae49264552ffdfd228d70c20cddb86e5cd7e2e9d90fdb33d57
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/tests/test_ut_32_token_budget.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/tests/test_ut_32_token_budget.py
# UT-32: SubTaskTokenBudget.record 100K 预算触发 SubTaskTokenBudgetExhausted
# (per 03-detailed §4.4 + NFR-SB-04: 累计 = 100K 时抛异常)

from __future__ import annotations

import unittest

from scripts.automation.task_ops.compat import QuotaConfigStore, TokenTelemetry
from scripts.automation.task_ops.spawn_subtask.event_bus import ParentChildEventBus
from scripts.automation.task_ops.spawn_subtask.exceptions import SubTaskTokenBudgetExhausted
from scripts.automation.task_ops.spawn_subtask.token_budget import (
    SubTaskTokenBudget,
    set_event_bus,
)


class TestSubTaskTokenBudget(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        TokenTelemetry.reset()
        QuotaConfigStore.reset()
        self._bus = ParentChildEventBus()
        set_event_bus(self._bus)
        self._received: list[dict] = []

        async def _capture(**kwargs) -> None:
            self._received.append(kwargs)

        self._bus.subscribe("subtask_token_budget_exhausted", _capture)

    async def test_record_below_budget_ok(self) -> None:
        await SubTaskTokenBudget.record("st-child-0000000001", 50_000, "llm_call")
        await SubTaskTokenBudget.record("st-child-0000000001", 49_999, "mcp_tool")
        self.assertEqual(99_999, await TokenTelemetry.get_total("st-child-0000000001"))
        self.assertEqual(0, len(self._received))

    async def test_record_hits_budget_raises(self) -> None:
        await SubTaskTokenBudget.record("st-child-0000000001", 50_000, "llm_call")
        with self.assertRaises(SubTaskTokenBudgetExhausted) as ctx:
            await SubTaskTokenBudget.record("st-child-0000000001", 50_000, "llm_call")
        message = str(ctx.exception)
        self.assertIn("token budget exhausted", message)
        self.assertIn("100000", message)
        # event published (per DD §3.3.8)
        self.assertEqual(1, len(self._received))
        self.assertEqual("st-child-0000000001", self._received[0]["task_id"])
        self.assertEqual(100_000, self._received[0]["used_total"])
        self.assertEqual(100_000, self._received[0]["budget"])

    async def test_per_task_budget_isolated(self) -> None:
        # 守门 #4 token-OLU: sub-task token 预算单独计量, 不跟其他 task 合并
        await SubTaskTokenBudget.record("st-child-0000000001", 99_999, "llm_call")
        await SubTaskTokenBudget.record("st-child-0000000002", 99_999, "llm_call")
        self.assertEqual(99_999, await TokenTelemetry.get_total("st-child-0000000001"))
        self.assertEqual(99_999, await TokenTelemetry.get_total("st-child-0000000002"))

    async def test_custom_budget_respected(self) -> None:
        # Master 配置 per_subtask_token_budget 覆盖默认 100K (per F-31);
        # budget 解析走 child 的 exclusive_owner_task_id → 父配置
        from scripts.automation.task_ops.compat import SubAgentState, put_sub_agent_state

        put_sub_agent_state(
            SubAgentState({
                "task_id": "st-child-0000000001",
                "exclusive_owner_task_id": "task-12345678",
            })
        )
        QuotaConfigStore.upsert_scd2(
            "task-12345678", per_subtask_token_budget=500
        )
        with self.assertRaises(SubTaskTokenBudgetExhausted):
            await SubTaskTokenBudget.record("st-child-0000000001", 500, "llm_call")


if __name__ == "__main__":
    unittest.main()
