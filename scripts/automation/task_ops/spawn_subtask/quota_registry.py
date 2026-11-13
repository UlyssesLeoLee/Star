# @cypher schema=1 source_sha256=c14df035d14fe36aa3d9b9bb5cfb9284f3151ab8ffc6fa702c943c3ce5b0ce01
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/quota_registry.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/quota_registry.py
# M-29 SubTaskQuotaRegistry (C-25)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.4
# + 01-requirements.md §1 UC-18 + §2 F-31 + 守门 #13 c/d (Master SCD Type 2 + RLS 13 类)
#
# 生产实现走 PostgreSQL Tier 3 (task_quota_config 表, per 003_task_quota_config.sql,
# close_old_quota_config 触发器); 本仓 Tier 3 待真人到位 (per 03-detailed §7 新增风险),
# 此处由 compat.QuotaConfigStore (内存版, 同 SCD Type 2 语义) 承载.
#
# 默认 quota (per F-31 + NFR-SB-04): max_subtasks=2, per_subtask_token_budget=100K.

from __future__ import annotations

from typing import Dict, Optional

from scripts.automation.task_ops.compat import QuotaConfigStore, get_sub_agent_state


class SubTaskQuotaRegistry:
    """C-25 SubTaskQuotaRegistry: per-parent sub-task quota 计量 + 配置."""

    DEFAULT_MAX_SUBTASKS = 2
    DEFAULT_PER_SUBTASK_TOKEN_BUDGET = 100_000  # 100K tokens (per F-31)

    @staticmethod
    async def query(parent_task_id: str, quota_check: bool = True) -> int:
        """查询父 task 剩余 quota (active sub-task count).

        per 01 §1 UC-18 Main flow step 2; quota_check=False 为 admin override
        (per F-26 admin override), 返回哨兵值 999 (per DD §3.3.4).
        """
        if not quota_check:
            return 999

        row = QuotaConfigStore.current(parent_task_id)
        if row is None:
            # 父 task 没有 quota config → 用默认值 (per DD §3.3.4)
            return SubTaskQuotaRegistry.DEFAULT_MAX_SUBTASKS

        return int(row["max_subtasks"]) - int(row["current_subtasks"])

    @staticmethod
    async def increment(parent_task_id: str) -> int:
        """active sub-task count +1 (per UC-18 step 4, SCD Type 2 close+insert).

        DD §3.3.4 注明"实际需读旧 max_subtasks + current_subtasks, 然后 +1" —
        此处按该语义完整实现: 继承旧行配置, current_subtasks+1.
        """
        row = QuotaConfigStore.current(parent_task_id)
        new_current = (int(row["current_subtasks"]) if row else 0) + 1
        new_row = QuotaConfigStore.upsert_scd2(
            parent_task_id,
            current_subtasks=new_current,
            created_by="Mavis",
        )
        return int(new_row["current_subtasks"])

    @staticmethod
    async def decrement(parent_task_id: str) -> int:
        """active sub-task count -1 (per UC-16 Alt flow E1 父 cancel 批量释放)."""
        row = QuotaConfigStore.current(parent_task_id)
        if row is None:
            return 0
        new_current = max(0, int(row["current_subtasks"]) - 1)
        new_row = QuotaConfigStore.upsert_scd2(
            parent_task_id,
            current_subtasks=new_current,
            created_by="Mavis",
        )
        return int(new_row["current_subtasks"])

    @staticmethod
    async def get_token_budget(task_id: str) -> Dict[str, int]:
        """查询 task 的 per-sub-task token 预算 (per F-31 + M-33 依赖).

        task 是 sub-task 时解析其父任务 (exclusive_owner_task_id 优先, 其次
        parent_task_id) 的 quota config; 无配置时返回默认 100K (per F-31).
        """
        owner_task_id: Optional[str] = None
        state = get_sub_agent_state(task_id)
        if state is not None:
            owner_task_id = state.get("exclusive_owner_task_id") or state.get("parent_task_id")

        budget = SubTaskQuotaRegistry.DEFAULT_PER_SUBTASK_TOKEN_BUDGET
        if owner_task_id:
            row = QuotaConfigStore.current(owner_task_id)
            if row is not None:
                budget = int(row["per_subtask_token_budget"])
        return {"per_subtask_token_budget": budget}
