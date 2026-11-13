# @cypher schema=1 source_sha256=5ff33f36275dff83ef288290a721f3ef4666328dd9f5e8fd5ab2356f7edbde70
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/spawn_validator.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/spawn_validator.py
# M-30 SpawnSubtaskValidator (C-26)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.5
# + 02-basic-design.md §2.7.1 step 1-3 校验
#
# spawn 7 步原子化前 5 步校验 (per 02-basic §1.1 C-26 責務):
#   check_parent_state / check_sa_type / check_quota / check_self_spawn /
#   check_parent_not_exclusive

from __future__ import annotations

from typing import Any, Dict

from scripts.automation.task_ops.spawn_subtask.exceptions import (
    ExclusiveBindingViolation,
    InvalidParentState,
    InvalidSAType,
    SelfSpawnProhibited,
    SubTaskQuotaExhausted,
)
from scripts.automation.task_ops.spawn_subtask.models import SA_TYPE_VALUES
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry


class SpawnSubtaskValidator:
    """C-26 SpawnSubtaskValidator: spawn 7 步原子化前 5 步校验."""

    ALLOWED_PARENT_STATES = frozenset(("running", "paused"))
    VALID_SA_TYPES = SA_TYPE_VALUES

    @staticmethod
    def check_parent_state(parent_state: Dict[str, Any]) -> None:
        """step 1: 父 task 状态校验 (per 02-basic §2.7.1 step 1 + F-26 互斥表)."""
        if parent_state.get("status") not in SpawnSubtaskValidator.ALLOWED_PARENT_STATES:
            raise InvalidParentState(
                f"Parent task state must be in {sorted(SpawnSubtaskValidator.ALLOWED_PARENT_STATES)}, "
                f"got '{parent_state.get('status')}'"
            )

    @staticmethod
    def check_sa_type(sa_type: str) -> None:
        """step 2: SA type 校验 (SA-10 自嵌套由 request 构造期拒绝, 此处冗余)."""
        if sa_type not in SpawnSubtaskValidator.VALID_SA_TYPES:
            raise InvalidSAType(f"Invalid SA type: {sa_type}")

    @staticmethod
    async def check_quota(parent_task_id: str) -> int:
        """step 3: quota 校验 (per 02-basic §2.7.1 step 3 + F-31 默认值 max=2)."""
        quota_remaining = await SubTaskQuotaRegistry.query(parent_task_id)
        if quota_remaining <= 0:
            raise SubTaskQuotaExhausted(
                f"Parent {parent_task_id} sub-task quota exhausted (max=2)"
            )
        return quota_remaining

    @staticmethod
    def check_self_spawn(parent_task_id: str, child_task_id: str) -> None:
        """校验 parent != child (per 01 §2 F-30 派生约束 + UC-17 Alt flow D2)."""
        if parent_task_id == child_task_id:
            raise SelfSpawnProhibited(
                f"parent_task_id ({parent_task_id}) == child_task_id ({child_task_id}), "
                "self-spawn prohibited"
            )

    @staticmethod
    def check_parent_not_exclusive(parent_state: Dict[str, Any]) -> None:
        """校验父 task 不是 exclusive sub-task (per 01 §2 F-30 派生约束,
        防止 exclusive 链式污染)."""
        if parent_state.get("exclusive_owner_task_id") is not None:
            raise ExclusiveBindingViolation(
                "Parent task is itself exclusive sub-task, "
                "cannot spawn further sub-task (防止 exclusive 链式污染)"
            )
