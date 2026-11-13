# scripts/automation/task_ops/spawn_subtask/tests/test_ut_31_lifecycle_linkage.py
# UT-31: LifecycleLinkageManager.on_parent_state_change 5 联动 (per 03-detailed §4.4)
# 联动表 (per 01 §2 F-29): cancel/pause/resume/fail + child 独立 cancel 不向上传播

from __future__ import annotations

import unittest

from scripts.automation.task_ops.compat import (
    AuditLog,
    SubAgentState,
    clear_sub_agent_states,
    get_sub_agent_state,
    put_sub_agent_state,
)
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.spawn_subtask.event_bus import ParentChildEventBus
from scripts.automation.task_ops.spawn_subtask.lifecycle_linkage import (
    LifecycleLinkageManager,
)
from scripts.automation.task_ops.spawn_subtask.models import SpawnSubtaskRequest
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry
from scripts.automation.task_ops.spawn_subtask.spawn_subtask_node import (
    get_relationship_graph,
    set_relationship_graph,
    spawn_subtask_node,
)
from scripts.automation.task_ops.compat import QuotaConfigStore

PARENT_ID = "task-12345678"


async def _spawn_child(parent_id: str = PARENT_ID, name: str = "git-init-st") -> str:
    req = SpawnSubtaskRequest(
        parent_task_id=parent_id,
        child_sa_type="SA-04",
        child_name=name,
        spawned_via="api",
    )
    result = await spawn_subtask_node({"active_tmo_operation": {"request": req}})
    payload = result["last_spawn_subtask_result"]
    assert payload.status == "spawned", payload.failure_reason
    return payload.child_task_id


class TestLifecycleLinkage(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        clear_sub_agent_states()
        AuditLog.clear()
        QuotaConfigStore.reset()
        set_relationship_graph(TaskRelationshipGraph())
        put_sub_agent_state(
            SubAgentState({"task_id": PARENT_ID, "status": "running", "task_type": "SA-01"})
        )
        self._bus = ParentChildEventBus()
        self._manager = LifecycleLinkageManager(
            event_bus=self._bus, graph=get_relationship_graph()
        )

    async def test_parent_cancel_chains_child_cancel(self) -> None:
        child_id = await _spawn_child()
        await self._manager.on_parent_state_change(PARENT_ID, "cancelling")

        child = get_sub_agent_state(child_id)
        assert child is not None
        self.assertEqual("cancelled", child.get("status"))
        self.assertEqual("parent_cancelled", child.get("cancel_reason"))
        # audit: 链式 lifecycle 事件 (per UC-16 step 6)
        events = [e for e in AuditLog.get_events() if e["event_type"] == "lifecycle_linkage_applied"]
        self.assertEqual(1, len(events))
        self.assertEqual("cancel", events[0]["action"])
        # quota 批量释放 (per UC-16 Alt E1)
        self.assertEqual(2, await SubTaskQuotaRegistry.query(PARENT_ID))

    async def test_parent_pause_chains_child_pause(self) -> None:
        child_id = await _spawn_child()
        await self._manager.on_parent_state_change(PARENT_ID, "paused")

        child = get_sub_agent_state(child_id)
        assert child is not None
        self.assertEqual("paused", child.get("status"))
        self.assertEqual("parent_paused", child.get("pause_reason"))

    async def test_parent_resume_resumes_paused_child(self) -> None:
        child_id = await _spawn_child()
        await self._manager.on_parent_state_change(PARENT_ID, "paused")
        await self._manager.on_parent_state_change(PARENT_ID, "running")

        child = get_sub_agent_state(child_id)
        assert child is not None
        self.assertEqual("running", child.get("status"))
        self.assertEqual("parent_resumed", child.get("resume_reason"))

    async def test_parent_resume_skips_running_child(self) -> None:
        child_id = await _spawn_child()
        # child 当前 running → resume 不动 (per DD §3.3.2 仅 paused 时 resume)
        await self._manager.on_parent_state_change(PARENT_ID, "running")

        child = get_sub_agent_state(child_id)
        assert child is not None
        self.assertEqual("running", child.get("status"))
        self.assertNotIn("resume_reason", child)

    async def test_parent_fail_chains_child_cancel(self) -> None:
        child_id = await _spawn_child()
        await self._manager.on_parent_state_change(PARENT_ID, "failed")

        child = get_sub_agent_state(child_id)
        assert child is not None
        self.assertEqual("cancelled", child.get("status"))
        self.assertEqual("parent_failed", child.get("cancel_reason"))

    async def test_unknown_state_no_linkage(self) -> None:
        child_id = await _spawn_child()
        await self._manager.on_parent_state_change(PARENT_ID, "summarizing")

        child = get_sub_agent_state(child_id)
        assert child is not None
        self.assertEqual("running", child.get("status"))
        linkage_events = [
            e for e in AuditLog.get_events() if e["event_type"] == "lifecycle_linkage_applied"
        ]
        self.assertEqual(0, len(linkage_events))

    async def test_child_independent_cancel_does_not_touch_parent(self) -> None:
        child_id = await _spawn_child()
        # child 独立 cancel (UC-16 Alt C2): 直接走 pool, 不经过 linkage
        from scripts.automation.task_ops.compat import SubAgentPool

        await SubAgentPool.cancel(child_id, reason="child_self_cancel")
        parent = get_sub_agent_state(PARENT_ID)
        assert parent is not None
        self.assertEqual("running", parent.get("status"))

    async def test_on_child_fail_degrades_parent_via_event(self) -> None:
        child_id = await _spawn_child()
        received: list[dict] = []

        async def _capture(**kwargs) -> None:
            received.append(kwargs)

        self._bus.subscribe("parent_degraded", _capture)

        await self._manager.on_child_fail(child_id, PARENT_ID)

        self.assertEqual(1, len(received))
        self.assertEqual(PARENT_ID, received[0]["parent_id"])
        self.assertEqual(child_id, received[0]["child_id"])
        self.assertEqual("child_failed", received[0]["reason"])
        # parent 本身不 cancel (per UC-16 Alt C3)
        parent = get_sub_agent_state(PARENT_ID)
        assert parent is not None
        self.assertEqual("running", parent.get("status"))
        events = [e for e in AuditLog.get_events() if e["event_type"] == "parent_degraded"]
        self.assertEqual(1, len(events))


if __name__ == "__main__":
    unittest.main()
