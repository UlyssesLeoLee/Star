# @cypher schema=1 source_sha256=9f7f0ebc214c6933a8f6f50409b9c66e3619c3a87b6e78c573e81a22800241a8
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/tests/test_ut_27_spawn_validator.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/tests/test_ut_27_spawn_validator.py
# UT-27: SpawnSubtaskValidator.check_parent_state (5 state × 2 expected, per 03-detailed §4.4)
# + check_sa_type / check_self_spawn / check_parent_not_exclusive / check_quota

from __future__ import annotations

import unittest

from scripts.automation.task_ops.spawn_subtask.exceptions import (
    ExclusiveBindingViolation,
    InvalidParentState,
    InvalidSAType,
    SelfSpawnProhibited,
    SubTaskQuotaExhausted,
)
from scripts.automation.task_ops.spawn_subtask.spawn_validator import SpawnSubtaskValidator
from scripts.automation.task_ops.compat import QuotaConfigStore


class TestCheckParentState(unittest.TestCase):
    """UT-27 核心: 5 状态 × 2 expected (per 03-detailed §4.4)."""

    def test_running_allowed(self) -> None:
        SpawnSubtaskValidator.check_parent_state({"status": "running"})

    def test_paused_allowed(self) -> None:
        SpawnSubtaskValidator.check_parent_state({"status": "paused"})

    def test_superseded_rejected(self) -> None:
        with self.assertRaises(InvalidParentState):
            SpawnSubtaskValidator.check_parent_state({"status": "superseded"})

    def test_completed_rejected(self) -> None:
        with self.assertRaises(InvalidParentState):
            SpawnSubtaskValidator.check_parent_state({"status": "completed"})

    def test_failed_rejected(self) -> None:
        with self.assertRaises(InvalidParentState):
            SpawnSubtaskValidator.check_parent_state({"status": "failed"})

    def test_cancelling_rejected(self) -> None:
        with self.assertRaises(InvalidParentState):
            SpawnSubtaskValidator.check_parent_state({"status": "cancelling"})

    def test_missing_state_rejected(self) -> None:
        with self.assertRaises(InvalidParentState):
            SpawnSubtaskValidator.check_parent_state({})

    def test_error_message_contains_state(self) -> None:
        with self.assertRaises(InvalidParentState) as ctx:
            SpawnSubtaskValidator.check_parent_state({"status": "completed"})
        self.assertIn("completed", str(ctx.exception))


class TestCheckSaType(unittest.TestCase):
    def test_valid_sa_types_pass(self) -> None:
        for sa in ("SA-01", "SA-05", "SA-09", "SA-10"):
            SpawnSubtaskValidator.check_sa_type(sa)

    def test_invalid_sa_type_rejected(self) -> None:
        with self.assertRaises(InvalidSAType):
            SpawnSubtaskValidator.check_sa_type("SA-99")


class TestCheckSelfSpawn(unittest.TestCase):
    def test_same_id_rejected(self) -> None:
        with self.assertRaises(SelfSpawnProhibited):
            SpawnSubtaskValidator.check_self_spawn("task-12345678", "task-12345678")

    def test_different_id_ok(self) -> None:
        SpawnSubtaskValidator.check_self_spawn("task-12345678", "st-abcdef0123456789")


class TestCheckParentNotExclusive(unittest.TestCase):
    def test_plain_parent_ok(self) -> None:
        SpawnSubtaskValidator.check_parent_not_exclusive({"status": "running"})

    def test_exclusive_parent_rejected(self) -> None:
        with self.assertRaises(ExclusiveBindingViolation):
            SpawnSubtaskValidator.check_parent_not_exclusive(
                {"status": "running", "exclusive_owner_task_id": "task-12345678"}
            )


class TestCheckQuota(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        QuotaConfigStore.reset()

    async def test_quota_available(self) -> None:
        # 无配置 → 默认 max=2 → remaining 2
        remaining = await SpawnSubtaskValidator.check_quota("task-12345678")
        self.assertEqual(2, remaining)

    async def test_quota_exhausted(self) -> None:
        QuotaConfigStore.reset()
        QuotaConfigStore.upsert_scd2(
            "task-12345678", max_subtasks=2, current_subtasks=2
        )
        with self.assertRaises(SubTaskQuotaExhausted):
            await SpawnSubtaskValidator.check_quota("task-12345678")
        QuotaConfigStore.reset()


if __name__ == "__main__":
    unittest.main()
