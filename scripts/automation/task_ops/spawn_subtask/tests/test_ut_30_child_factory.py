# scripts/automation/task_ops/spawn_subtask/tests/test_ut_30_child_factory.py
# UT-30: ChildSubAgentFactory.create 4 binding 字段写入 (per 03-detailed §4.4)

from __future__ import annotations

import unittest
from datetime import datetime

from scripts.automation.task_ops.spawn_subtask.child_factory import ChildSubAgentFactory


class TestChildSubAgentFactory(unittest.IsolatedAsyncioTestCase):
    async def test_create_writes_4_binding_fields(self) -> None:
        state = await ChildSubAgentFactory.create(
            child_task_id="st-abc1234567890def",
            parent_task_id="task-12345678",
            sa_type="SA-04",
            child_name="git-init-st",
            initial_context={"git_root": "/repo"},
            bound_by="Ulysses",
            bound_via="ui_rightclick",
        )

        # 4 binding 字段 (per 02-basic §3.2.1.2):
        self.assertEqual("task-12345678", state["exclusive_owner_task_id"])
        self.assertIsInstance(state["bound_at"], datetime)
        self.assertEqual("Ulysses", state["bound_by"])
        self.assertEqual("ui_rightclick", state["bound_via"])

    async def test_create_writes_base_fields(self) -> None:
        state = await ChildSubAgentFactory.create(
            child_task_id="st-abc1234567890def",
            parent_task_id="task-12345678",
            sa_type="SA-04",
            child_name="git-init-st",
            initial_context={},
            bound_by="Mavis",
            bound_via="l0_chat",
        )
        self.assertEqual("st-abc1234567890def", state["task_id"])
        self.assertEqual("git-init-st", state["task_name"])
        self.assertEqual("SA-04", state["task_type"])
        self.assertEqual("initializing", state["status"])
        self.assertEqual("task-12345678", state["parent_task_id"])

    async def test_initial_context_shallow_copy_with_exclusive_mark(self) -> None:
        context = {"git_root": "/repo"}
        state = await ChildSubAgentFactory.create(
            child_task_id="st-abc1234567890def",
            parent_task_id="task-12345678",
            sa_type="SA-01",
            child_name="code-review-st",
            initial_context=context,
            bound_by="Ulysses",
            bound_via="api",
        )
        # 浅拷贝 + exclusive 标注 (per DD §3.3.6)
        self.assertEqual("/repo", state["initial_context"]["git_root"])
        self.assertIs(True, state["initial_context"]["_exclusive_bound"])
        # 浅拷贝: 原 dict 不被污染
        self.assertNotIn("_exclusive_bound", context)

    async def test_bound_at_is_utc_aware(self) -> None:
        state = await ChildSubAgentFactory.create(
            child_task_id="st-abc1234567890def",
            parent_task_id="task-12345678",
            sa_type="SA-02",
            child_name="wbs-st",
            initial_context={},
            bound_by="Ulysses",
            bound_via="api",
        )
        bound_at = state["bound_at"]
        self.assertIsNotNone(bound_at.tzinfo)
        self.assertIsNotNone(bound_at.utcoffset())


if __name__ == "__main__":
    unittest.main()
