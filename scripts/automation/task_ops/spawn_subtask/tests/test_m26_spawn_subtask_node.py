# @cypher schema=1 source_sha256=0cf3461ac6e1714bc3b8d9b40358eb63698ecfde617958cd2ac143db351f1b55
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/tests/test_m26_spawn_subtask_node.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/tests/test_m26_spawn_subtask_node.py
# M-26 M-N8 spawn_subtask_node 7 步原子化测试 (happy path + 5 类失败模式 + 守门集成)
# 覆盖 IT-13 语义的进程内版本 (PostgreSQL/LangGraph/console_server 集成待 Tier 3)

from __future__ import annotations

import unittest

from scripts.automation.task_ops.compat import (
    AuditLog,
    QuotaConfigStore,
    SubAgentState,
    clear_sub_agent_states,
    get_sub_agent_state,
    put_sub_agent_state,
)
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.spawn_subtask.exclusive_guard import ExclusiveBindingGuard
from scripts.automation.task_ops.spawn_subtask.exceptions import ExclusiveBindingViolation
from scripts.automation.task_ops.spawn_subtask.models import SpawnSubtaskRequest
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry
from scripts.automation.task_ops.spawn_subtask.spawn_subtask_node import (
    get_relationship_graph,
    set_relationship_graph,
    spawn_subtask_node,
)

PARENT_ID = "task-12345678"


def _register_parent(status: str = "running", **extra: object) -> SubAgentState:
    state = SubAgentState({
        "task_id": PARENT_ID,
        "task_name": "parent-task",
        "task_type": "SA-01",
        "status": status,
        **extra,
    })
    put_sub_agent_state(state)
    return state


def _request(name: str = "git-init-st", sa_type: str = "SA-04") -> SpawnSubtaskRequest:
    return SpawnSubtaskRequest(
        parent_task_id=PARENT_ID,
        child_sa_type=sa_type,
        child_name=name,
        spawned_via="api",
    )


class TestSpawnSubtaskNodeHappyPath(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        clear_sub_agent_states()
        AuditLog.clear()
        QuotaConfigStore.reset()
        set_relationship_graph(TaskRelationshipGraph())
        _register_parent("running")

    async def test_7_step_happy_path(self) -> None:
        result = await spawn_subtask_node(
            {"active_tmo_operation": {"request": _request()}}
        )
        payload = result["last_spawn_subtask_result"]
        self.assertEqual("spawned", payload.status)
        self.assertIsNone(payload.failure_reason)
        self.assertTrue(payload.child_task_id.startswith("st-"))
        self.assertEqual(PARENT_ID, payload.parent_task_id)
        self.assertEqual("SA-04", payload.sa_type)
        self.assertEqual(PARENT_ID, payload.exclusive_owner_task_id)  # 1:1 独占
        self.assertEqual(1, payload.quota_remaining)                 # 2-1

        # step 5+6: child 写入 4 binding 字段 + status = running (UC-14 post-condition)
        child = get_sub_agent_state(payload.child_task_id)
        assert child is not None
        self.assertEqual("running", child.get("status"))
        self.assertEqual(PARENT_ID, child.get("exclusive_owner_task_id"))
        self.assertEqual(PARENT_ID, child.get("parent_task_id"))
        self.assertIsNotNone(child.get("bound_at"))
        self.assertEqual("Ulysses", child.get("bound_by"))
        self.assertEqual("api", child.get("bound_via"))

        # step 7: M-20 血缘边 (metadata.edge_type=spawn_exclusive)
        graph = get_relationship_graph()
        self.assertIn(payload.child_task_id, graph.dependents(PARENT_ID))
        node = graph.get_node(payload.child_task_id)
        assert node is not None
        self.assertEqual(PARENT_ID, node.parent_task_id)
        self.assertEqual("spawn_exclusive", node.metadata.get("edge_type"))

        # 父 task 状态不变 (per UC-14 post-condition)
        parent = get_sub_agent_state(PARENT_ID)
        assert parent is not None
        self.assertEqual("running", parent.get("status"))

        # audit append (Transaction append-only)
        spawn_events = [
            e for e in AuditLog.get_events() if e["event_type"] == "subtask_spawned"
        ]
        self.assertEqual(1, len(spawn_events))
        self.assertEqual(payload.child_task_id, spawn_events[0]["child_task_id"])

        # quota 已计量 (UC-18 step 4)
        self.assertEqual(1, await SubTaskQuotaRegistry.query(PARENT_ID))

    async def test_parent_state_unchanged_after_spawn(self) -> None:
        result = await spawn_subtask_node(
            {"active_tmo_operation": {"request": _request()}}
        )
        self.assertEqual("spawned", result["last_spawn_subtask_result"].status)
        parent = get_sub_agent_state(PARENT_ID)
        assert parent is not None
        self.assertEqual("running", parent.get("status"))


class TestSpawnSubtaskNodeFailureModes(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        clear_sub_agent_states()
        AuditLog.clear()
        QuotaConfigStore.reset()
        set_relationship_graph(TaskRelationshipGraph())

    async def _run(self, req: SpawnSubtaskRequest) -> dict:
        return await spawn_subtask_node({"active_tmo_operation": {"request": req}})

    async def test_invalid_parent_state_fails(self) -> None:
        for status in ("superseded", "completed", "failed", "cancelling"):
            clear_sub_agent_states()
            _register_parent(status)
            result = await self._run(_request())
            payload = result["last_spawn_subtask_result"]
            self.assertEqual("failed", payload.status)
            self.assertTrue(payload.failure_reason.startswith("InvalidParentState"),
                            payload.failure_reason)
            self.assertEqual("", payload.child_task_id)
            # 失败也审计 (per 守门 #12)
            failed_events = [
                e for e in AuditLog.get_events() if e["event_type"] == "subtask_spawn_failed"
            ]
            self.assertGreaterEqual(len(failed_events), 1)

    async def test_missing_parent_fails(self) -> None:
        # 父 task 不存在 → get_state None → InvalidParentState
        result = await self._run(_request())
        payload = result["last_spawn_subtask_result"]
        self.assertEqual("failed", payload.status)
        self.assertTrue(payload.failure_reason.startswith("InvalidParentState"))

    async def test_quota_exhausted_after_2_children(self) -> None:
        _register_parent("running")
        first = await self._run(_request("child-1"))
        second = await self._run(_request("child-2"))
        self.assertEqual("spawned", first["last_spawn_subtask_result"].status)
        self.assertEqual("spawned", second["last_spawn_subtask_result"].status)

        third = await self._run(_request("child-3"))
        payload = third["last_spawn_subtask_result"]
        self.assertEqual("failed", payload.status)
        self.assertTrue(payload.failure_reason.startswith("SubTaskQuotaExhausted"),
                        payload.failure_reason)
        self.assertIn("max=2", payload.failure_reason)

    async def test_exclusive_parent_cannot_spawn_chain(self) -> None:
        # F-30 派生约束: 父任务 exclusive = true 时不能再 spawn (链式污染防护)
        _register_parent("running", exclusive_owner_task_id="task-grandparent")
        result = await self._run(_request())
        payload = result["last_spawn_subtask_result"]
        self.assertEqual("failed", payload.status)
        self.assertTrue(payload.failure_reason.startswith("ExclusiveBindingViolation"),
                        payload.failure_reason)


class TestSpawnSubtaskNodeGuardIntegration(unittest.IsolatedAsyncioTestCase):
    """E2E-15 语义的进程内版本: 1:1 binding 不可绕过 (NFR-SB-02)."""

    async def asyncSetUp(self) -> None:
        clear_sub_agent_states()
        AuditLog.clear()
        QuotaConfigStore.reset()
        set_relationship_graph(TaskRelationshipGraph())
        _register_parent("running")

    async def test_reassign_exclusive_child_blocked(self) -> None:
        result = await spawn_subtask_node(
            {"active_tmo_operation": {"request": _request()}}
        )
        child_id = result["last_spawn_subtask_result"].child_task_id

        # M-N6 reassign 入口守门 (per UC-17 Main flow)
        with self.assertRaises(ExclusiveBindingViolation) as ctx:
            ExclusiveBindingGuard.check_reassign_allowed(child_id)
        self.assertIn("reassign prohibited", str(ctx.exception))
        self.assertIn("NFR-SB-02", str(ctx.exception))

        # merge / split 互斥 (per F-28)
        with self.assertRaises(ExclusiveBindingViolation):
            ExclusiveBindingGuard.check_merge_allowed(child_id)
        with self.assertRaises(ExclusiveBindingViolation):
            ExclusiveBindingGuard.check_split_allowed(child_id)

    async def test_guard_true_after_spawn_false_before(self) -> None:
        # spawn 前不存在 child → is_exclusive False
        self.assertFalse(ExclusiveBindingGuard.is_exclusive("st-not-yet-spawned00"))
        result = await spawn_subtask_node(
            {"active_tmo_operation": {"request": _request()}}
        )
        child_id = result["last_spawn_subtask_result"].child_task_id
        self.assertTrue(ExclusiveBindingGuard.is_exclusive(child_id))
        # 父任务不是 exclusive (spawn 不污染父)
        self.assertFalse(ExclusiveBindingGuard.is_exclusive(PARENT_ID))


if __name__ == "__main__":
    unittest.main()
