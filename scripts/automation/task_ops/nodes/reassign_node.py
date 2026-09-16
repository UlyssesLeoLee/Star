# scripts/automation/task_ops/nodes/reassign_node.py
# TMO M-N6 reassign_node (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §3.2.1.1)
#
# 职责: 跨 SA 类型切换 (e.g. SA-01 code-gen → SA-04 test-gen)
# 方向: L0 → L1 (per 02-basic-design.md §2.6.2)
# 协议: ReassignRequest TypedDict (per protocols.py)
#
# 流程:
#   1. validate: target_task_id 存在 + 状态 ∈ {pending, in_progress, blocked, done} (守门: 终态不可重分配)
#   2. validate: new_task_type ∈ {SA-01..SA-10}
#   3. 保留 checkpoint (preserved_checkpoint_id 由 ReassignManager C-21 派生, 真实接入时)
#   4. sub_pool.update 同步 task_type 字段 (dataclass handle 字段, 不只 state mirror)
#   5. mark status = "reassigned" + 记录 reassigned_at + reassigned_to 字段
#   6. ui_streamer.push × 1 (1 × TaskCardUpdate: task_type 改变)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口 (跨 SA 类型切换只经 L0)
#   - 守门 #13 d: 保留 checkpoint (Transaction append-only)
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #22: 调试控制台走 port 8080 console_server.py, 不污染 main 编译链
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#
# 派发 (per 守门 #20): 实装前必先 brief 落档 (TMO-06 父会话 Mavis 委托)

from __future__ import annotations

import logging
import time
from typing import Any, Optional

logger = logging.getLogger("task_ops.nodes.reassign_node")


# ===== 常量 (per 03 §3.2.1.1 + 02 §2.6.4) =====

VALID_SA_TYPES: tuple[str, ...] = (
    "SA-01",  # code-gen
    "SA-02",  # doc-gen
    "SA-03",  # refactor
    "SA-04",  # test-gen
    "SA-05",  # review
    "SA-06",  # deploy
    "SA-07",  # monitor
    "SA-08",  # search
    "SA-09",  # translate
    "SA-10",  # task-orchestrator (TMO 自己)
)
"""M-N6 reassign 合法目标 SA 类型 (per 03 §3.2.1.1 + 02 §2.6.1).

守门: 防止误填非 SA 类型, 跟 star_context.sub_agent.types 注册一致.
"""

REASSIGN_FORBIDDEN_STATUSES: frozenset[str] = frozenset({"superseded", "cancelled"})
"""M-N6 reassign 禁止源状态 (per 03 §3.2.1.1 注释): 终态不可重分配.

supersede 已经在 split_node / merge_node 终态, 不应再 reassign.
"""


# ===== 辅助函数 =====

def _validate_reassign_request(
    target_id: str,
    new_task_type: str,
    sub_pool,
) -> str:
    """M-N6 步骤 1+2: validate

    守门:
      - target_id 非空 + 存在
      - new_task_type ∈ VALID_SA_TYPES
      - target 状态 ∉ REASSIGN_FORBIDDEN_STATUSES

    Returns:
        old_task_type (用于返回结果)
    """
    if not target_id:
        raise ValueError("reassign_node: target_task_id is required")

    if new_task_type not in VALID_SA_TYPES:
        raise ValueError(
            f"reassign_node: new_task_type {new_task_type!r} not in VALID_SA_TYPES {VALID_SA_TYPES}"
        )

    handle = sub_pool.get(target_id)
    current_status = handle.state.get("status", "pending")
    if current_status in REASSIGN_FORBIDDEN_STATUSES:
        raise ValueError(
            f"reassign_node: target {target_id} has terminal status {current_status!r}, "
            f"cannot reassign (per REASSIGN_FORBIDDEN_STATUSES)"
        )

    return handle.task_type


async def _preserve_checkpoint(
    target_id: str,
    sub_pool,
    new_task_type: str,
) -> str:
    """M-N6 步骤 3: 保留 checkpoint (Transaction append-only per 守门 #13 d).

    Returns:
        preserved_checkpoint_id (append-only 永存)
    """
    checkpoint_id = await sub_pool.checkpoint(
        target_id, label=f"reassign_preserve_{target_id}_to_{new_task_type}",
    )
    logger.info(
        "reassign_node preserve_checkpoint: task=%s checkpoint=%s",
        target_id, checkpoint_id,
    )
    return checkpoint_id


async def _apply_reassignment(
    target_id: str,
    new_task_type: str,
    old_task_type: str,
    sub_pool,
) -> None:
    """M-N6 步骤 4+5: sub_pool 同步 task_type + status.

    守门:
      - 守门 #13 a: L0 唯一入口, 跨 SA 切换只经 L0 sub_pool
      - 守门 #13 d: reassigned_at / reassigned_to 字段记账 (Transaction append-only)
    """
    # 关键: 同时同步 dataclass 字段 (handle.task_type) 和 state 镜像
    # sub_pool.update 默认只更新 state 字典, 需要在 manager 派生
    # 这里直接通过 setattr 改 handle.task_type (因 sub_pool.get() 返回的是引用)
    handle = sub_pool.get(target_id)
    handle.task_type = new_task_type
    await sub_pool.update(target_id, {
        "status": "reassigned",
        "reassigned_from": old_task_type,
        "reassigned_to": new_task_type,
        "reassigned_at": time.time(),
    })
    logger.info(
        "reassign_node apply: task=%s %s -> %s",
        target_id, old_task_type, new_task_type,
    )


def _emit_ui_events(
    target_id: str,
    old_task_type: str,
    new_task_type: str,
    preserved_checkpoint_id: str,
) -> list[dict]:
    """M-N6 步骤 6: emit UI events (1 × TaskCardUpdate task_type 改变)

    守门 #24: 调试控制台走 subprocess, 不直接 RPC.
    """
    events: list[dict] = [{
        "type": "TaskCardUpdate",
        "task_id": target_id,
        "patch": {
            "task_type": new_task_type,
            "reassigned_from": old_task_type,
            "reassigned_to": new_task_type,
            "preserved_checkpoint_id": preserved_checkpoint_id,
        },
    }]
    logger.info("reassign_node emit_ui_events: 1 event (TaskCardUpdate task_type change)")
    return events


# ===== 主函数: reassign_node =====

async def reassign_node(state: dict, manager) -> dict:
    """TMO M-N6: 跨 SA 类型切换 (per 03 §3.2.1.1)

    输入 (state = ReassignRequest TypedDict, per protocols.py):
      operation: "reassign"
      target_task_id: 单一 task_id
      new_task_type: SA-XX (守门 ∈ VALID_SA_TYPES)
      preserved_checkpoint_id: 可选, ReassignManager C-21 派生
      actor_session_id: 发起者 session_id

    输出 (TopAgentState 增量更新, per 03 §3.2.1.1):
      superseded_tasks: [] (无副作用, 不取代原 task)
      active_tmo_operation: None
      global_context: {last_tmo_result: {operation, target_task_id, old_task_type, new_task_type, ...}}
      ui_events: 1 个 TaskCardUpdate (task_type 改变)
      old_task_type: 切换前类型
      new_task_type: 切换后类型
      preserved_checkpoint_id: 保留的 checkpoint id

    守门:
      - 守门 #13 a: L0 唯一入口, 跨 SA 切换只经 L0
      - 守门 #13 d: checkpoint 永存, reassigned 状态不删除原 task
      - 守门 #19: Python 化, 不写 .rs
    """
    target_id: str = state.get("target_task_id") or ""
    new_task_type: str = state.get("new_task_type") or ""
    actor_session_id: Optional[str] = state.get("actor_session_id")
    preserved_checkpoint_id_provided: Optional[str] = state.get("preserved_checkpoint_id")

    logger.info(
        "reassign_node start: target=%s new_type=%s actor=%s",
        target_id, new_task_type, actor_session_id,
    )

    sub_pool = manager.sub_pool

    # 步骤 1+2: validate
    old_task_type = _validate_reassign_request(target_id, new_task_type, sub_pool)

    # 同类型 reassign 是 no-op, 直接返回 (但仍记录)
    if old_task_type == new_task_type:
        logger.info(
            "reassign_node noop: target=%s already type=%s", target_id, new_task_type,
        )
        return {
            "operation": "reassign",
            "target_task_id": target_id,
            "old_task_type": old_task_type,
            "new_task_type": new_task_type,
            "noop": True,
            "active_tmo_operation": None,
        }

    # 步骤 3: 保留 checkpoint (除非 caller 已经提供)
    if preserved_checkpoint_id_provided:
        preserved_checkpoint_id = preserved_checkpoint_id_provided
        logger.info(
            "reassign_node use provided checkpoint: %s", preserved_checkpoint_id,
        )
    else:
        preserved_checkpoint_id = await _preserve_checkpoint(target_id, sub_pool, new_task_type)

    # 步骤 4+5: 应用 reassignment
    await _apply_reassignment(target_id, new_task_type, old_task_type, sub_pool)

    # 步骤 6: emit UI events
    ui_events = _emit_ui_events(target_id, old_task_type, new_task_type, preserved_checkpoint_id)

    result: dict = {
        "operation": "reassign",
        "target_task_id": target_id,
        "old_task_type": old_task_type,
        "new_task_type": new_task_type,
        "preserved_checkpoint_id": preserved_checkpoint_id,
        "ui_events": ui_events,
        "active_tmo_operation": None,
    }
    logger.info(
        "reassign_node done: %s -> %s checkpoint=%s",
        old_task_type, new_task_type, preserved_checkpoint_id,
    )
    return result
