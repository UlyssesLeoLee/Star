# scripts/automation/task_ops/spawn_subtask/exclusive_guard.py
# M-28 ExclusiveBindingGuard (C-24)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.3
# + 01-requirements.md §1 UC-17 + §2 F-30 + §3 NFR-SB-02 (不可绕过)
#
# 编译期 (type system, SubAgentState.exclusive_owner_task_id: Optional[str]) +
# 运行期 (本 class) 双层守门; L1 sub_pool 5 入口 (spawn/cancel/pause/resume/reassign)
# 强制调用 (per F-30 运行期守门).

from __future__ import annotations

from typing import Optional

from scripts.automation.task_ops.compat import SubAgentState, get_sub_agent_state
from scripts.automation.task_ops.spawn_subtask.exceptions import ExclusiveBindingViolation


class ExclusiveBindingGuard:
    """C-24 ExclusiveBindingGuard: 1:1 独占守门, 不可绕过 (NFR-SB-02)."""

    @staticmethod
    def is_exclusive(task_id: str) -> bool:
        """检查 task 是否 exclusive 1:1 绑定 (per 01 §2 F-28)."""
        state: Optional[SubAgentState] = get_sub_agent_state(task_id)
        if state is None:
            return False
        return state.get("exclusive_owner_task_id") is not None

    @staticmethod
    def check_reassign_allowed(task_id: str) -> None:
        """reassign 守门 (M-N6 入口): exclusive sub-task 必拒绝."""
        if ExclusiveBindingGuard.is_exclusive(task_id):
            owner = ""
            state = get_sub_agent_state(task_id)
            if state is not None:
                owner = str(state.get("exclusive_owner_task_id"))
            raise ExclusiveBindingViolation(
                f"Sub-task `{task_id}` is exclusive-bound to parent "
                f"`{owner}`, "
                f"reassign prohibited (per NFR-SB-02)"
            )

    @staticmethod
    def check_merge_allowed(task_id: str) -> None:
        """merge 守门 (M-N1 入口): exclusive sub-task 必拒绝 (per F-28 互斥)."""
        if ExclusiveBindingGuard.is_exclusive(task_id):
            raise ExclusiveBindingViolation(
                f"Sub-task `{task_id}` is exclusive-bound, merge prohibited "
                f"(per F-28 exclusive sub-task cannot merge)"
            )

    @staticmethod
    def check_split_allowed(task_id: str) -> None:
        """split 守门 (M-N2 入口): exclusive sub-task 必拒绝 (per F-28 互斥)."""
        if ExclusiveBindingGuard.is_exclusive(task_id):
            raise ExclusiveBindingViolation(
                f"Sub-task `{task_id}` is exclusive-bound, split prohibited "
                f"(per F-28 exclusive sub-task cannot split)"
            )
