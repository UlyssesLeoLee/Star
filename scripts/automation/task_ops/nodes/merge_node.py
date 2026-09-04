# scripts/automation/task_ops/nodes/merge_node.py
# TMO M-N1 merge_node (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §3.2.1.1)
#
# 职责: 合并 a + b → merged_task
# 流程:
#   1. validate: a / b 都存在, 不是 superseded 状态
#   2. 通知 a / b 进入 stash_state (Transaction append-only per 守门 #13 d)
#   3. dispatch merged_task (SA-10 task-orchestrator)
#   4. 标记 a / b 状态 = "superseded", pointer → merged_task
#   5. ui_streamer.push × 3 (TaskCardUpdate a/b, TaskCardCreate merged)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口 (L1↔L1 禁止通信)
#   - 守门 #13 d: stash_state append-only, supersede 终态不删除
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#
# 派发 (per 守门 #20): 实装前必先 brief 落档 (本子项 brief: docs/briefs/tmo-2026-09-04-parallel.md)

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from typing import Any, Optional

logger = logging.getLogger("task_ops.nodes.merge_node")


# ===== 辅助函数 (per 03 §3.2.1.1 草稿改 Python mock 模式) =====

def _validate_target_tasks(target_ids: list[str], sub_pool) -> list[str]:
    """M-N1 步骤 1: validate

    守门:
      - target_ids ≥ 2 (per 02 §2.6.1)
      - 所有 task_id 存在
      - 所有 task 状态 ≠ superseded
    """
    if len(target_ids) < 2:
        raise ValueError(f"merge_node requires >= 2 task_ids, got {target_ids}")

    for tid in target_ids:
        try:
            handle = sub_pool.get(tid)
        except KeyError as e:
            raise ValueError(f"merge_node validate failed: {e}") from e
        if handle.state.get("status") == "superseded":
            raise ValueError(
                f"merge_node validate failed: task {tid} is already superseded, cannot merge"
            )
    return target_ids


async def _stash_states(target_ids: list[str], sub_pool) -> list[str]:
    """M-N1 步骤 2: stash_state (Transaction append-only per 守门 #13 d)

    返回 stash_checkpoint_ids 列表 (append-only history)
    """
    stash_ids: list[str] = []
    for tid in target_ids:
        handle = sub_pool.get(tid)
        # 先把状态标 stash_pending, 让 UI 显示"合并中"
        await sub_pool.update(tid, {"status": "stash_pending", "stash_initiated_at": time.time()})
        stash_id = await sub_pool.checkpoint(tid, label=f"merge_stash_{tid}")
        stash_ids.append(stash_id)
        logger.info("merge_node stash_state: task=%s checkpoint=%s", tid, stash_id)
    return stash_ids


async def _dispatch_merged_task(target_ids: list[str], stash_ids: list[str], sub_pool, user_input: Optional[str] = None) -> str:
    """M-N1 步骤 3: dispatch merged_task (SA-10 task-orchestrator)

    守门 #13 a: L0 唯一入口, SA-10 由 L0 协调
    守门 #23: 不开 OpenAI/Anthropic, SA-10 走 mock
    """
    # 导入 SA-10 (避免循环依赖)
    try:
        from automation.sub_agent.types.sa_10_task_orchestrator import SA10TaskOrchestrator
    except ImportError:
        # 在 tests/ 目录下跑时 fallback
        import sys
        sys.path.insert(0, str(__file__).rsplit("/scripts/", 1)[0] if "/scripts/" in __file__ else ".")
        try:
            from scripts.automation.sub_agent.types.sa_10_task_orchestrator import SA10TaskOrchestrator  # type: ignore
        except ImportError:
            # 最后 fallback: inline minimal SA-10
            SA10TaskOrchestrator = None

    merged_task_id = f"merged-{uuid.uuid4().hex[:8]}"

    if SA10TaskOrchestrator is not None:
        orchestrator = SA10TaskOrchestrator(merged_task_id=merged_task_id)
        merged_task_id = await orchestrator.run(
            operation="merge",
            merged_from=target_ids,
            merged_state=stash_ids,
            original_user_input=user_input,
        )
    else:
        # fallback: 直接 sub_pool.spawn (mock mode)
        merged_handle = await sub_pool.spawn(
            task_type="SA-10",
            context={
                "operation": "merge",
                "merged_from": target_ids,
                "merged_state": stash_ids,
                "original_user_input": user_input,
            },
            task_id=merged_task_id,
        )
        merged_task_id = merged_handle.task_id

    logger.info("merge_node dispatch_merged_task: id=%s from=%s", merged_task_id, target_ids)
    return merged_task_id


async def _mark_superseded(target_ids: list[str], merged_task_id: str, sub_pool) -> None:
    """M-N1 步骤 4: mark a / b superseded, pointer → merged_task

    守门 #13 d: supersede 是终态, append-only, 不删除原 task 记录
    """
    for tid in target_ids:
        await sub_pool.update(tid, {
            "status": "superseded",
            "superseded_by": merged_task_id,
            "superseded_at": time.time(),
        })
        logger.info("merge_node mark_superseded: task=%s -> merged=%s", tid, merged_task_id)


def _emit_ui_events(target_ids: list[str], merged_task_id: str, stash_ids: list[str]) -> list[dict]:
    """M-N1 步骤 5: ui_streamer.push × 3 (TaskCardUpdate a/b, TaskCardCreate merged)

    守门 #24: 调试控制台走 subprocess, 不直接 RPC
    本步骤 emit 事件到 mock UI stream (Next.js 通过 /api/tmo/merge 拉)
    """
    events = []
    # 2 × TaskCardUpdate (a / b 状态变 superseded)
    for tid in target_ids:
        events.append({
            "type": "TaskCardUpdate",
            "task_id": tid,
            "patch": {"status": "superseded", "superseded_by": merged_task_id},
        })
    # 1 × TaskCardCreate (新 merged_task)
    events.append({
        "type": "TaskCardCreate",
        "task_id": merged_task_id,
        "card": {
            "task_type": "SA-10",
            "status": "running",
            "merged_from": target_ids,
            "stash_checkpoint_ids": stash_ids,
        },
    })
    logger.info("merge_node emit_ui_events: %d events", len(events))
    return events


# ===== 主函数: merge_node =====

async def merge_node(state: dict, manager) -> dict:
    """TMO M-N1: 合并 a + b → merged_task

    输入 (state = MergeRequest dict, per protocols.py):
      operation: "merge"
      target_task_ids: [a, b, ...]
      merge_strategy: "context_union" (default) | "checkpoint_union" | "label_priority"
      original_user_input: 用户 chat bar 输入
      actor_session_id: L0 session id

    输出 (TopAgentState 增量更新, per 03 §3.2.1.1):
      superseded_tasks: target_ids (reducer operator.add append-only)
      active_tmo_operation: None
      global_context: {last_tmo_result: {operation, merged_task_id, superseded_task_ids}}
      ui_events: 3 个 UI 事件 (TaskCardUpdate × 2 + TaskCardCreate × 1)

    守门:
      - 守门 #13 a: L0 唯一入口, 跨 L1 task 操作只经 L0
      - 守门 #13 d: stash_state append-only, supersede 终态不删除
    """
    target_ids = state.get("target_task_ids", [])
    merge_strategy = state.get("merge_strategy", "context_union")
    user_input = state.get("original_user_input")
    actor_session_id = state.get("actor_session_id")

    logger.info(
        "merge_node start: targets=%s strategy=%s actor=%s",
        target_ids, merge_strategy, actor_session_id,
    )

    sub_pool = manager.sub_pool

    # 步骤 1: validate
    _validate_target_tasks(target_ids, sub_pool)

    # 步骤 2: stash_state (Transaction append-only per 守门 #13 d)
    stash_ids = await _stash_states(target_ids, sub_pool)

    # 步骤 3: dispatch merged_task (SA-10 task-orchestrator)
    merged_task_id = await _dispatch_merged_task(target_ids, stash_ids, sub_pool, user_input)

    # 步骤 4: mark a / b superseded
    await _mark_superseded(target_ids, merged_task_id, sub_pool)

    # 步骤 5: emit UI events
    ui_events = _emit_ui_events(target_ids, merged_task_id, stash_ids)

    # 返回 TopAgentState 增量 (per 03 §3.2.1.1)
    return {
        "superseded_tasks": list(target_ids),  # reducer operator.add (append-only)
        "active_tmo_operation": None,  # TMO operation done
        "global_context": {
            "last_tmo_result": {
                "operation": "merge",
                "merged_task_id": merged_task_id,
                "superseded_task_ids": list(target_ids),
                "merge_strategy": merge_strategy,
                "stash_checkpoint_ids": stash_ids,
            }
        },
        "ui_events": ui_events,
        "merged_task_id": merged_task_id,
        "stash_checkpoint_ids": stash_ids,
    }


# ===== 同步 wrapper (用于非 async context) =====

def merge_node_sync(state: dict, manager) -> dict:
    """merge_node 同步 wrapper (用于 FastAPI handler 等非 async 场景)"""
    return asyncio.run(merge_node(state, manager))
