# @cypher schema=1 source_sha256=d7ae7ce60abf4e91ce5e99999e8734a5adf7cd543352cb52bf09de1854e1d233
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/lifecycle_linkage.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/lifecycle_linkage.py
# M-27 LifecycleLinkageManager (C-23)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.2
# + 01-requirements.md §1 UC-16 + §2 F-29 + §3 NFR-SB-03 (联动延迟 ≤ 1s)
#
# 联动表 (per 01 §2 F-29):
#   parent cancelling → child cancel    | parent paused → child pause
#   parent running    → child resume (仅 child 在 paused 时)
#   parent failed     → child cancel (链式, 不可逆)
#   child 独立 cancel → parent 不变 (不向上传播)
#   child fail        → parent degraded (on_child_fail, 警告不强制 cancel)
#
# 适配说明: M-20 TaskRelationshipGraph 实装走本仓 4 字段约定
# (child.parent_task_id = parent, 边 child→parent), children 查找 = dependents
# 过滤 metadata.edge_type == "spawn_exclusive" + exclusive_owner_task_id == parent.
# 父 cancel/fail 时按 UC-16 Alt E1 批量释放 quota (SubTaskQuotaRegistry.decrement).

from __future__ import annotations

from enum import Enum
from typing import List

from scripts.automation.task_ops.compat import AuditLog, SubAgentPool, get_sub_agent_state
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.spawn_subtask.event_bus import ParentChildEventBus
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry


class LifecycleAction(str, Enum):
    CANCEL = "cancel"
    PAUSE = "pause"
    RESUME = "resume"
    FAIL = "fail"


class LifecycleLinkageManager:
    """C-23 LifecycleLinkageManager: parent state change → child action 链式触发."""

    def __init__(self, event_bus: ParentChildEventBus, graph: TaskRelationshipGraph):
        self._event_bus = event_bus
        self._graph = graph

    async def on_parent_state_change(self, parent_id: str, new_state: str) -> None:
        """父 task state change → 链式触发 child action (per DD §3.3.2)."""
        # 1. 查找所有 exclusive children (edge_type=spawn_exclusive)
        children_ids = self._exclusive_children(parent_id)

        # 2. 根据 parent state 决定 child action
        action_map = {
            "cancelling": LifecycleAction.CANCEL,
            "paused": LifecycleAction.PAUSE,
            "running": LifecycleAction.RESUME,  # 仅 child 在 paused 时
            "failed": LifecycleAction.FAIL,     # child cancel (不可逆)
        }
        action = action_map.get(new_state)
        if action is None:
            return  # 其他 state 不联动

        # 3. 链式触发 child action (默认串行, ≤ 1s 延迟 per NFR-SB-03)
        for child_id in children_ids:
            await self._apply_child_action(child_id, action, parent_id)

        # 4. audit log append (Transaction append-only, per 守门 #13 b)
        await AuditLog.append(
            event_type="lifecycle_linkage_applied",
            parent_task_id=parent_id,
            children_count=len(children_ids),
            action=action.value,
        )

    def _exclusive_children(self, parent_id: str) -> List[str]:
        """查找 parent 的全部 spawn_exclusive children (per DD get_children 适配)."""
        candidates = sorted(self._graph.dependents(parent_id))
        children: List[str] = []
        for child_id in candidates:
            state = get_sub_agent_state(child_id)
            if state is None:
                continue
            if state.get("exclusive_owner_task_id") != parent_id:
                continue
            children.append(child_id)
        return children

    async def _apply_child_action(
        self, child_id: str, action: LifecycleAction, parent_id: str
    ) -> None:
        """应用 child action (单 child, per DD §3.3.2)."""
        if action == LifecycleAction.CANCEL:
            await SubAgentPool.cancel(task_id=child_id, reason="parent_cancelled")
            # UC-16 Alt E1: 父 cancel → 批量释放 quota
            await SubTaskQuotaRegistry.decrement(parent_id)
        elif action == LifecycleAction.PAUSE:
            await SubAgentPool.pause(task_id=child_id, reason="parent_paused")
        elif action == LifecycleAction.RESUME:
            child_state = await SubAgentPool.get_state(child_id)
            if child_state is not None and child_state.get("status") == "paused":
                await SubAgentPool.resume(task_id=child_id, reason="parent_resumed")
        elif action == LifecycleAction.FAIL:
            # parent failed → child cancel (链式, 不可逆)
            await SubAgentPool.cancel(task_id=child_id, reason="parent_failed")
            await SubTaskQuotaRegistry.decrement(parent_id)

    async def on_child_fail(self, child_id: str, parent_id: str) -> None:
        """child 失败 → parent 状态 = degraded (警告, 不强制 cancel, per UC-16 Alt C3)."""
        await self._event_bus.publish(
            event_type="parent_degraded",
            parent_id=parent_id,
            child_id=child_id,
            reason="child_failed",
        )
        await AuditLog.append(
            event_type="parent_degraded",
            parent_task_id=parent_id,
            child_task_id=child_id,
            reason="child_failed",
        )
