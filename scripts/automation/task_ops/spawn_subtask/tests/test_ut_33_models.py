# @cypher schema=1 source_sha256=8e8fe01f59a8603a2de09b707a29f6e550bf8aa6171e733b8320d034c4145361
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/tests/test_ut_33_models.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/tests/test_ut_33_models.py
# UT-33: SpawnSubtaskRequest 校验拒绝 SA-10 自嵌套 (per 03-detailed §4.4)
# + 其余字段约束 (per 01 §2 F-27)

from __future__ import annotations

import unittest

from scripts.automation.task_ops.spawn_subtask.models import (
    SA10_SELF_SPAWN_MESSAGE,
    SpawnSubtaskRequest,
)


class TestSpawnSubtaskRequestValidation(unittest.TestCase):
    def test_sa10_self_spawn_rejected(self) -> None:
        # UT-33 核心: SA-10 task-orchestrator 自嵌套拒绝 (per UC-14 Alt A2)
        with self.assertRaises(ValueError) as ctx:
            SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type="SA-10",
                child_name="orchestrator-st",
            )
        self.assertIn("SA-10", str(ctx.exception))
        self.assertIn("self-spawn prohibited", str(ctx.exception))
        self.assertEqual(SA10_SELF_SPAWN_MESSAGE, str(ctx.exception))

    def test_valid_request_constructed(self) -> None:
        req = SpawnSubtaskRequest(
            parent_task_id="task-12345678",
            child_sa_type="SA-04",
            child_name="git-init-st",
        )
        self.assertEqual("SA-04", req.child_sa_type)
        self.assertEqual("Ulysses", req.bound_by)   # 默认 (per 守门 #10 代签)
        self.assertEqual("api", req.spawned_via)    # 默认入口标注
        self.assertTrue(req.quota_check)
        self.assertEqual({}, req.initial_context)

    def test_invalid_sa_type_rejected(self) -> None:
        with self.assertRaises(ValueError):
            SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type="SA-99",
                child_name="bad-st",
            )

    def test_child_name_too_long_rejected(self) -> None:
        with self.assertRaises(ValueError):
            SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type="SA-04",
                child_name="x" * 201,
            )

    def test_empty_child_name_rejected(self) -> None:
        with self.assertRaises(ValueError):
            SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type="SA-04",
                child_name="",
            )

    def test_parent_task_id_format_rejected(self) -> None:
        with self.assertRaises(ValueError):
            SpawnSubtaskRequest(
                parent_task_id="bad id!",  # 空格 + 短于 8
                child_sa_type="SA-04",
                child_name="git-init-st",
            )

    def test_bound_by_format_rejected(self) -> None:
        with self.assertRaises(ValueError):
            SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type="SA-04",
                child_name="git-init-st",
                bound_by="random-stranger",
            )

    def test_spawned_via_rejected(self) -> None:
        with self.assertRaises(ValueError):
            SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type="SA-04",
                child_name="git-init-st",
                spawned_via="email",
            )

    def test_all_sa_types_except_sa10_accepted(self) -> None:
        for sa in ("SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
                   "SA-06", "SA-07", "SA-08", "SA-09"):
            req = SpawnSubtaskRequest(
                parent_task_id="task-12345678",
                child_sa_type=sa,
                child_name=f"child-{sa}",
            )
            self.assertEqual(sa, req.child_sa_type)


class TestSpawnSubtaskResultToDict(unittest.TestCase):
    def test_to_dict_serializable(self) -> None:
        from scripts.automation.task_ops.spawn_subtask.models import SpawnSubtaskResult

        result = SpawnSubtaskResult(
            child_task_id="st-abc1234567890def",
            parent_task_id="task-12345678",
            sa_type="SA-04",
            status="spawned",
            exclusive_owner_task_id="task-12345678",
            quota_remaining=1,
        )
        data = result.to_dict()
        self.assertEqual("spawned", data["status"])
        self.assertIsNone(data["failure_reason"])
        self.assertEqual("st-abc1234567890def", data["child_task_id"])


if __name__ == "__main__":
    unittest.main()
