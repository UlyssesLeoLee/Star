# @cypher schema=1 source_sha256=6141ba064994e636efe0a723705f8b6b75062c7e5078a4c942ade0815dc9c233
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/spawn_subtask_node.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/spawn_subtask_node.py
# M-26 M-N8 spawn_subtask_node (C-26 + C-27 协作)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.1.1
# + 02-basic-design.md §2.7.1 (7 步原子化) + 01-requirements.md §2 F-26
#
# 守门:
#   - 守门 #13 a: L1↔L1 禁止通信 → 全部 L0 协调, 唯一 cross-task actor
#   - 守门 #7:   0 unsafe → type-safe (stdlib dataclass + dict)
#   - 守门 #19:  走 Python 化 (subprocess 入口见 cli.py)
#   - 守门 #12:  缺标比错标 → 失败必填 failure_reason
#
# 7 步原子化 (per 02-basic §2.7.1):
#   step 1: M-30 check_parent_state (parent.state ∈ {running, paused})
#   step 2: M-30 check_sa_type (SA-10 自嵌套已在 request 构造期拒绝)
#   step 3: M-29 SubTaskQuotaRegistry.query (≤ max_subtasks=2)
#   step 4: generate child_task_id (UUID v7 简化, prefix=`st-`) + check_self_spawn
#   step 4b: check_parent_not_exclusive (F-30 防止 exclusive 链式污染)
#   step 5: M-31 ChildSubAgentFactory.create (写 4 binding 字段)
#   step 6: M-07 SubAgentRegistry.spawn (child status = running)
#   step 7: M-20 TaskRelationshipGraph 挂边 (metadata.edge_type="spawn_exclusive")
#           + M-29 increment (UC-18 step 4) + AuditLog append
#
# 适配说明 (vs DD §3.3.1.1 文本):
#   - DD 的 TaskRelationshipGraph.add_edge(parent→child, edge_type, metadata)
#     适配为本仓 M-20 实装 4 字段约定: add_task(child, parent_task_id=parent,
#     metadata={edge_type: "spawn_exclusive", ...}), 边方向 child→parent
#     (child 依赖 parent), dependents(parent) 可反向查到全部 children.
#   - DD node 文本未显式调用 check_self_spawn / check_parent_not_exclusive;
#     本实装按 01 §2 F-30 派生约束 + UC-17 Alt D1/D2 装配 (M-30 提供该函数).

from __future__ import annotations

import uuid
from typing import Any, Dict

from scripts.automation.task_ops.compat import AuditLog, SubAgentPool, SubAgentRegistry
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.spawn_subtask.child_factory import ChildSubAgentFactory
from scripts.automation.task_ops.spawn_subtask.exceptions import (
    ExclusiveBindingViolation,
    InvalidParentState,
    InvalidSAType,
    SelfSpawnProhibited,
    SubTaskQuotaExhausted,
)
from scripts.automation.task_ops.spawn_subtask.models import (
    SpawnSubtaskRequest,
    SpawnSubtaskResult,
    TopAgentState,
)
from scripts.automation.task_ops.spawn_subtask.quota_registry import SubTaskQuotaRegistry
from scripts.automation.task_ops.spawn_subtask.spawn_validator import SpawnSubtaskValidator

# 模块级共享 relationship graph (per DD classmethod 形态适配; manager 集成时可
# set_relationship_graph 注入, 测试注入隔离实例).
_graph: TaskRelationshipGraph = TaskRelationshipGraph()


def get_relationship_graph() -> TaskRelationshipGraph:
    """读取当前共享 relationship graph (测试/审计用)."""
    return _graph


def set_relationship_graph(graph: TaskRelationshipGraph) -> None:
    """注入 relationship graph (测试隔离 / manager 集成用)."""
    global _graph
    _graph = graph


def _request_from_state(state: TopAgentState) -> SpawnSubtaskRequest:
    """从 TopAgentState.active_tmo_operation 提取请求 (per DD §3.3.1.1)."""
    op = state.get("active_tmo_operation") or {}
    req = op.get("request")
    if not isinstance(req, SpawnSubtaskRequest):
        raise KeyError("active_tmo_operation.request 缺失或非 SpawnSubtaskRequest")
    return req


def _failure_result(
    req: SpawnSubtaskRequest, exc: Exception
) -> Dict[str, Any]:
    """组装失败结果 (per DD §3.3.1.1 + 守门 #12 缺标比错标)."""
    failure_reason = f"{type(exc).__name__}: {exc}"
    return {
        "last_spawn_subtask_result": SpawnSubtaskResult(
            child_task_id="",
            parent_task_id=req.parent_task_id,
            sa_type=req.child_sa_type,
            status="failed",
            failure_reason=failure_reason,
            exclusive_owner_task_id=req.parent_task_id,
            quota_remaining=-1,
        ),
        "active_tmo_operation": None,
    }


async def spawn_subtask_node(state: TopAgentState) -> Dict[str, Any]:
    """TMO M-N8: 1 派生 1 创建绑定子任务, 父继续 running, child 1:1 独占 SA-XX.

    7 步原子化 (per 02-basic §2.7.1); 5 类失败模式全部审计 (per 守门 #12).
    """
    req: SpawnSubtaskRequest = _request_from_state(state)

    try:
        # ===== step 1: 校验父 task 状态 (per F-26 互斥表) =====
        parent_state = await SubAgentPool.get_state(req.parent_task_id)
        SpawnSubtaskValidator.check_parent_state(
            parent_state if parent_state is not None else {}
        )

        # ===== step 2: 校验 SA type (SA-10 已由 request 构造期拒绝, 此处冗余) =====
        SpawnSubtaskValidator.check_sa_type(req.child_sa_type)

        # ===== step 3: 校验 quota (≤ max_subtasks=2, per F-31 默认值) =====
        quota_remaining = await SubTaskQuotaRegistry.query(
            parent_task_id=req.parent_task_id,
            quota_check=req.quota_check,
        )
        if quota_remaining <= 0:
            raise SubTaskQuotaExhausted(
                f"Parent {req.parent_task_id} sub-task quota exhausted (max=2)"
            )

        # ===== step 4: 生成 child_task_id (UUID v7 简化, prefix=`st-`) =====
        # per 01 §2 F-27: child_task_id 必须 = st- prefix
        child_task_id = f"st-{uuid.uuid4().hex[:16]}"
        SpawnSubtaskValidator.check_self_spawn(req.parent_task_id, child_task_id)

        # ===== step 4b: 父 task exclusive 链式污染防护 (per F-30 派生约束) =====
        SpawnSubtaskValidator.check_parent_not_exclusive(
            parent_state if parent_state is not None else {}
        )

        # ===== step 5: M-31 ChildSubAgentFactory.create (写 4 binding 字段) =====
        child_state = await ChildSubAgentFactory.create(
            child_task_id=child_task_id,
            parent_task_id=req.parent_task_id,
            sa_type=req.child_sa_type,
            child_name=req.child_name,
            initial_context=req.initial_context,
            bound_by=req.bound_by,
            bound_via=req.spawned_via,
        )

        # ===== step 6: M-07 SubAgentRegistry.spawn (child status = running) =====
        await SubAgentRegistry.spawn(
            task_id=child_task_id,
            sa_type=req.child_sa_type,
            state=child_state,
        )

        # ===== step 7: M-20 挂边 + quota 计量 + audit append =====
        graph = get_relationship_graph()
        graph.add_task(
            task_id=child_task_id,
            parent_task_id=req.parent_task_id,
            status="running",
            metadata={
                "edge_type": "spawn_exclusive",  # per UC-14 step 7.7
                "sa_type": req.child_sa_type,
                "bound_at": child_state["bound_at"].isoformat(),
                "bound_by": req.bound_by,
            },
        )
        # UC-18 step 4: 每次 sub-task lifecycle event → quota update
        await SubTaskQuotaRegistry.increment(req.parent_task_id)

        # audit log append (Transaction append-only, per 守门 #13 b)
        await AuditLog.append(
            event_type="subtask_spawned",
            parent_task_id=req.parent_task_id,
            child_task_id=child_task_id,
            sa_type=req.child_sa_type,
            spawned_via=req.spawned_via,
        )

        # 父 task 状态不变 (继续 running / paused, 不 supersede, per UC-14)
        return {
            "last_spawn_subtask_result": SpawnSubtaskResult(
                child_task_id=child_task_id,
                parent_task_id=req.parent_task_id,
                sa_type=req.child_sa_type,
                status="spawned",
                spawned_at=child_state["bound_at"],
                exclusive_owner_task_id=req.parent_task_id,
                quota_remaining=quota_remaining - 1,
            ),
            "active_tmo_operation": None,
        }

    except (InvalidParentState, InvalidSAType, SubTaskQuotaExhausted,
            SelfSpawnProhibited, ExclusiveBindingViolation) as e:
        # 5 类失败模式 (per 02-basic §2.7.1), 失败也审计 (per 守门 #12)
        await AuditLog.append(
            event_type="subtask_spawn_failed",
            parent_task_id=req.parent_task_id,
            failure_reason=f"{type(e).__name__}: {e}",
            sa_type=req.child_sa_type,
        )
        return _failure_result(req, e)


# DD §3.3.1.1 将 5 异常类定义于本模块; 本仓集中至 exceptions.py, 此处重导出
# 保持 import 面兼容 (from ...spawn_subtask_node import InvalidParentState 可用).
__all__ = [
    "spawn_subtask_node",
    "get_relationship_graph",
    "set_relationship_graph",
    "InvalidParentState",
    "InvalidSAType",
    "SubTaskQuotaExhausted",
    "SelfSpawnProhibited",
    "ExclusiveBindingViolation",
]
