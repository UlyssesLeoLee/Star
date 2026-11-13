# @cypher schema=1 source_sha256=6584aa1d4e3bffd4cd9d31f604022e8861d72160e4b26fedb90464d1a3cd9b38
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/child_factory.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/child_factory.py
# M-31 ChildSubAgentFactory (C-27)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.6
# + 01-requirements.md §2 F-27 + F-28 (child SubAgentState 初始化 + 4 binding 字段)
#
# child SubAgentState 初始化 (per 02-basic §3.2.1.2 SubAgentState 4 字段扩展):
#   - exclusive_owner_task_id = parent_task_id (1:1 独占, write-once per R-12)
#   - bound_at = UTC now
#   - bound_by = 创建者 (per 守门 #10 代签)
#   - bound_via = 入口标注 (ui_rightclick / l0_chat / api)

from __future__ import annotations

from datetime import datetime, timezone
from typing import Any, Dict

from scripts.automation.task_ops.compat import SubAgentState


class ChildSubAgentFactory:
    """C-27 ChildSubAgentFactory: child SubAgentState 初始化."""

    @staticmethod
    async def create(
        child_task_id: str,
        parent_task_id: str,
        sa_type: str,
        child_name: str,
        initial_context: Dict[str, Any],
        bound_by: str,
        bound_via: str,
    ) -> SubAgentState:
        """创建 child SubAgentState + 写 4 binding 字段 (per 02-basic §3.2.1.2)."""
        now = datetime.now(timezone.utc)
        child_state: SubAgentState = SubAgentState(
            {
                "task_id": child_task_id,
                "task_name": child_name,
                "task_type": sa_type,
                "status": "initializing",
                # 已有血缘字段 (per LangGraph 03-detailed §3.2.1.1)
                "parent_task_id": parent_task_id,
                # v0.1 增量 4 binding 字段 (per F-28):
                "exclusive_owner_task_id": parent_task_id,  # 1:1 独占
                "bound_at": now,                            # 绑定创建时间
                "bound_by": bound_by,                       # 绑定创建者
                "bound_via": bound_via,                     # 入口标注
                # 父 context 浅拷贝 + exclusive 标注 (per DD §3.3.6)
                "initial_context": {
                    **initial_context,
                    "_exclusive_bound": True,
                },
            }
        )
        return child_state
