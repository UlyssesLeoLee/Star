# @cypher schema=1 source_sha256=c3d63f792f90eeafd63f49fb806995177c8d49028af6d7644a3d815077b5cb38
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/token_budget.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/token_budget.py
# M-33 SubTaskTokenBudget (C-29)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.8
# + 01-requirements.md §1 UC-18 + §2 F-31 + §3 NFR-SB-04
# + 守门 #4 token-OLU (sub-task token 预算单独计量, 不跟父任务合并)
#
# 达到预算 → publish subtask_token_budget_exhausted + 抛 SubTaskTokenBudgetExhausted
# (per NFR-SB-04: 累计 = 100K 时触发).

from __future__ import annotations

from scripts.automation.task_ops.compat import TokenTelemetry
from scripts.automation.task_ops.spawn_subtask.event_bus import ParentChildEventBus
from scripts.automation.task_ops.spawn_subtask.exceptions import SubTaskTokenBudgetExhausted
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry

# 模块级默认 event bus (per DD §3.3.8 ParentChildEventBus() 形态); 测试可注入.
_bus: ParentChildEventBus = ParentChildEventBus()


def get_event_bus() -> ParentChildEventBus:
    """读取当前 event bus (测试/审计用)."""
    return _bus


def set_event_bus(bus: ParentChildEventBus) -> None:
    """注入 event bus (测试隔离 / 集成用)."""
    global _bus
    _bus = bus


class SubTaskTokenBudget:
    """C-29 SubTaskTokenBudget: per-sub-task token 预算计量 (NFR-SB-04)."""

    @staticmethod
    async def record(task_id: str, tokens_used: int, op: str) -> None:
        """记录 sub-task token 使用, 达到预算抛 SubTaskTokenBudgetExhausted.

        per 01 §1 UC-18 Main flow step 1 (per DD §3.3.8).
        """
        # 1. 累加 TokenTelemetry (per 守门 #4 token-OLU 独立计量)
        await TokenTelemetry.record(task_id, tokens_used, op)

        # 2. 查询 budget (per F-31 per-sub-task 默认 100K)
        budget_config = await SubTaskQuotaRegistry.get_token_budget(task_id)
        used_total = await TokenTelemetry.get_total(task_id)

        # 3. 达到预算抛异常 + publish (per DD §3.3.8)
        if used_total >= budget_config["per_subtask_token_budget"]:
            await _bus.publish(
                event_type="subtask_token_budget_exhausted",
                task_id=task_id,
                used_total=used_total,
                budget=budget_config["per_subtask_token_budget"],
            )
            raise SubTaskTokenBudgetExhausted(
                f"Sub-task {task_id} token budget exhausted: "
                f"{used_total} >= {budget_config['per_subtask_token_budget']}"
            )
