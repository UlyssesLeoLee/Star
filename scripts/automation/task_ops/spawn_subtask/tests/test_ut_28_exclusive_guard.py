# @cypher schema=1 source_sha256=f6e1bb6344d0e33cc8612919717dd2890b1b18f3b9d9f61ad61c6b4cb6e28342
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/tests/test_ut_28_exclusive_guard.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/tests/test_ut_28_exclusive_guard.py
# UT-28: ExclusiveBindingGuard.is_exclusive / check_reassign_allowed (per 03-detailed §4.4)
# + check_merge_allowed / check_split_allowed (per F-28 互斥)

from __future__ import annotations

import unittest

from scripts.automation.task_ops.compat import (
    SubAgentState,
    clear_sub_agent_states,
    put_sub_agent_state,
)
from scripts.automation.task_ops.spawn_subtask.exceptions import ExclusiveBindingViolation
from scripts.automation.task_ops.spawn_subtask.exclusive_guard import ExclusiveBindingGuard


class TestIsExclusive(unittest.TestCase):
    def setUp(self) -> None:
        clear_sub_agent_states()

    def test_unknown_task_not_exclusive(self) -> None:
        self.assertFalse(ExclusiveBindingGuard.is_exclusive("st-unknown00000000"))

    def test_bound_child_is_exclusive(self) -> None:
        put_sub_agent_state(
            SubAgentState({
                "task_id": "st-child-0000000001",
                "parent_task_id": "task-12345678",
                "exclusive_owner_task_id": "task-12345678",
                "status": "running",
            })
        )
        self.assertTrue(ExclusiveBindingGuard.is_exclusive("st-child-0000000001"))

    def test_plain_parent_not_exclusive(self) -> None:
        put_sub_agent_state(
            SubAgentState({"task_id": "task-12345678", "status": "running"})
        )
        self.assertFalse(ExclusiveBindingGuard.is_exclusive("task-12345678"))


class TestCheckReassignAllowed(unittest.TestCase):
    def setUp(self) -> None:
        clear_sub_agent_states()
        put_sub_agent_state(
            SubAgentState({
                "task_id": "st-child-0000000001",
                "parent_task_id": "task-12345678",
                "exclusive_owner_task_id": "task-12345678",
                "status": "running",
            })
        )

    def test_exclusive_reassign_rejected(self) -> None:
        with self.assertRaises(ExclusiveBindingViolation) as ctx:
            ExclusiveBindingGuard.check_reassign_allowed("st-child-0000000001")
        message = str(ctx.exception)
        self.assertIn("reassign prohibited", message)
        self.assertIn("NFR-SB-02", message)
        self.assertIn("task-12345678", message)

    def test_non_exclusive_reassign_allowed(self) -> None:
        put_sub_agent_state(
            SubAgentState({"task_id": "task-12345678", "status": "running"})
        )
        ExclusiveBindingGuard.check_reassign_allowed("task-12345678")

    def test_unknown_task_reassign_allowed(self) -> None:
        ExclusiveBindingGuard.check_reassign_allowed("task-not-exist-12345")


class TestMergeSplitGuards(unittest.TestCase):
    def setUp(self) -> None:
        clear_sub_agent_states()
        put_sub_agent_state(
            SubAgentState({
                "task_id": "st-child-0000000001",
                "exclusive_owner_task_id": "task-12345678",
                "status": "running",
            })
        )

    def test_merge_rejected(self) -> None:
        with self.assertRaises(ExclusiveBindingViolation) as ctx:
            ExclusiveBindingGuard.check_merge_allowed("st-child-0000000001")
        self.assertIn("merge prohibited", str(ctx.exception))

    def test_split_rejected(self) -> None:
        with self.assertRaises(ExclusiveBindingViolation) as ctx:
            ExclusiveBindingGuard.check_split_allowed("st-child-0000000001")
        self.assertIn("split prohibited", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
