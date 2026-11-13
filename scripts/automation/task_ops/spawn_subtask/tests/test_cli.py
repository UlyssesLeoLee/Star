# scripts/automation/task_ops/spawn_subtask/tests/test_cli.py
# cli.py subprocess 入口测试 (per 守门 #19 v19 Python 化 + 守门 #24 v2 subprocess 范式)
#
# 注: cli.run 内部 asyncio.run — 必须跑在无运行中 loop 的线程, 故用普通
# unittest.TestCase (非 IsolatedAsyncioTestCase).

from __future__ import annotations

import json
import unittest

from scripts.automation.task_ops.compat import (
    AuditLog,
    QuotaConfigStore,
    SubAgentState,
    clear_sub_agent_states,
    put_sub_agent_state,
)
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.spawn_subtask.cli import main
from scripts.automation.task_ops.spawn_subtask.spawn_subtask_node import (
    set_relationship_graph,
)


class TestSpawnSubtaskCli(unittest.TestCase):
    def setUp(self) -> None:
        clear_sub_agent_states()
        AuditLog.clear()
        QuotaConfigStore.reset()
        set_relationship_graph(TaskRelationshipGraph())
        put_sub_agent_state(
            SubAgentState({"task_id": "task-12345678", "status": "running"})
        )

    def test_cli_success(self) -> None:
        import contextlib
        import io

        with contextlib.redirect_stdout(io.StringIO()):
            exit_code = main([
                "spawn_subtask",
                "--parent-task-id", "task-12345678",
                "--child-sa-type", "SA-04",
                "--child-name", "git-init-st",
                "--spawned-via", "api",
            ])
        self.assertEqual(0, exit_code)

    def test_cli_success_prints_json_to_stdout(self) -> None:
        import contextlib
        import io

        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            main([
                "spawn_subtask",
                "--parent-task-id", "task-12345678",
                "--child-sa-type", "SA-04",
                "--child-name", "git-init-st",
            ])
        payload = json.loads(buf.getvalue())
        self.assertEqual("spawned", payload["status"])
        self.assertTrue(payload["child_task_id"].startswith("st-"))
        self.assertEqual("task-12345678", payload["parent_task_id"])
        self.assertEqual("task-12345678", payload["exclusive_owner_task_id"])

    def test_cli_sa10_fails_with_exit_2(self) -> None:
        import contextlib
        import io

        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = main([
                "spawn_subtask",
                "--parent-task-id", "task-12345678",
                "--child-sa-type", "SA-10",
                "--child-name", "orchestrator-st",
            ])
        self.assertEqual(2, exit_code)
        payload = json.loads(buf.getvalue())
        self.assertEqual("failed", payload["status"])
        self.assertIn("SA-10", payload["failure_reason"])

    def test_cli_invalid_parent_fails_with_exit_2(self) -> None:
        import contextlib
        import io

        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = main([
                "spawn_subtask",
                "--parent-task-id", "task-not-exist-0001",
                "--child-sa-type", "SA-04",
                "--child-name", "git-init-st",
            ])
        self.assertEqual(2, exit_code)
        payload = json.loads(buf.getvalue())
        self.assertEqual("failed", payload["status"])
        self.assertIn("InvalidParentState", payload["failure_reason"])

    def test_cli_no_quota_check_override(self) -> None:
        # admin override: --no-quota-check → query 走哨兵 999 (per F-26)
        import contextlib
        import io

        # 先耗尽 quota
        QuotaConfigStore.upsert_scd2(
            "task-12345678", max_subtasks=2, current_subtasks=2
        )
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = main([
                "spawn_subtask",
                "--parent-task-id", "task-12345678",
                "--child-sa-type", "SA-04",
                "--child-name", "override-st",
                "--no-quota-check",
            ])
        self.assertEqual(0, exit_code)
        self.assertEqual("spawned", json.loads(buf.getvalue())["status"])


if __name__ == "__main__":
    unittest.main()
