# scripts/automation/task_ops/nodes/split_node.py
# TMO M-N2 split_node (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §3.2.1.1)
#
# 职责: 拆分 a → a1 + a2 + ... (默认 2 份, 上限 8 份)
# 流程:
#   1. validate: target_id 存在 + 状态 ≠ superseded + split_count ∈ [2, 8]
#   2. snapshot a 当前 checkpoint (Transaction append-only per 守门 #13 d)
#   3. dispatch a1..aN (相同 task_type as a, forked context, _split_from / _split_index / _split_snapshot 注入)
#   4. mark a 状态 = "superseded", a.split_into = [a1, a2, ...]
#   5. ui_streamer.push × (1 + N) (1 × TaskCardUpdate a + N × TaskCardCreate a1..aN)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口 (L1↔L1 禁止通信, 全部经 L0 TaskOperationsManager C-16)
#   - 守门 #13 d: snapshot 永存, supersede 终态不删除原 task 记录
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #22: 调试控制台走 port 8080 console_server.py, 不污染 main 编译链
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#
# 派发 (per 守门 #20): 实装前必先 brief 落档 (TMO-02 父会话 Mavis 委托)

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from typing import Any, Optional, Sequence

logger = logging.getLogger("task_ops.nodes.split_node")


# ===== 常量 (per 03 §3.2.1.1 + 02 §2.6.4) =====

VALID_SPLIT_STRATEGIES: tuple[str, ...] = ("context_fork", "checkpoint_fork")
"""M-N2 split_strategy 合法值 (per 03 §3.2.1.1):
  - context_fork: 拷贝父 context, 注入 _split_from / _split_index 等元数据
  - checkpoint_fork: 拷贝父 checkpoint history 到所有 fork
"""

DEFAULT_SPLIT_COUNT: int = 2
MIN_SPLIT_COUNT: int = 2
MAX_SPLIT_COUNT: int = 8
"""守门: 防止爆量. 默认 2 份 (per 03 §3.2.1.1 split_node 伪代码 for i in range(2))"""


# ===== 辅助函数 =====

def _validate_split_request(
    target_id: str,
    split_strategy: str,
    split_count: int,
    sub_pool,
) -> tuple[str, dict]:
    """M-N2 步骤 1: validate

    守门:
      - target_id 非空
      - split_strategy ∈ {context_fork, checkpoint_fork}
      - split_count ∈ [2, 8] (守门爆量)
      - target_id 存在
      - target 状态 ≠ superseded

    Returns:
        (task_type, base_context) — 用于后续 dispatch
    """
    if not target_id:
        raise ValueError("split_node: target_task_id is required")

    if split_strategy not in VALID_SPLIT_STRATEGIES:
        raise ValueError(
            f"split_node: split_strategy must be one of {VALID_SPLIT_STRATEGIES}, got {split_strategy!r}"
        )

    if not isinstance(split_count, int) or split_count < MIN_SPLIT_COUNT:
        raise ValueError(
            f"split_node: split_count must be int >= {MIN_SPLIT_COUNT}, got {split_count!r}"
        )
    if split_count > MAX_SPLIT_COUNT:
        raise ValueError(
            f"split_node: split_count must be <= {MAX_SPLIT_COUNT}, got {split_count} "
            f"(防止爆量, 守门 #13 a L0 协调限流)"
        )

    try:
        handle = sub_pool.get(target_id)
    except KeyError as e:
        raise ValueError(f"split_node validate failed: {e}") from e

    if handle.state.get("status") == "superseded":
        raise ValueError(
            f"split_node validate failed: task {target_id} is already superseded, cannot split"
        )

    # 返回 task_type + base context (用于后续 dispatch fork)
    return handle.task_type, dict(handle.state.get("context", {}))


async def _snapshot_target(target_id: str, sub_pool) -> str:
    """M-N2 步骤 2: snapshot 当前 checkpoint (Transaction append-only per 守门 #13 d)

    Returns:
        snapshot_checkpoint_id (append-only 永存)
    """
    handle = sub_pool.get(target_id)
    # 先把状态标 snapshot_pending, 让 UI 显示"拆分中"
    await sub_pool.update(target_id, {
        "status": "snapshot_pending",
        "snapshot_initiated_at": time.time(),
    })
    snapshot_id = await sub_pool.checkpoint(target_id, label=f"split_snapshot_{target_id}")
    logger.info("split_node snapshot: task=%s checkpoint=%s", target_id, snapshot_id)
    return snapshot_id


async def _dispatch_fork_tasks(
    target_id: str,
    snapshot_id: str,
    task_type: str,
    base_context: dict,
    split_strategy: str,
    split_count: int,
    sub_pool,
) -> list[str]:
    """M-N2 步骤 3: dispatch a1..aN (相同 task_type as a, forked context)

    守门 #13 a: L0 唯一入口, fork 通过 L0 sub_pool.spawn
    """
    new_task_ids: list[str] = []
    for i in range(split_count):
        # 生成新 task_id: 优先用 "<target>-aN", 冲突时回退 uuid
        candidate = f"{target_id}-a{i + 1}"
        try:
            sub_pool.get(candidate)
            candidate = f"{target_id}-a{i + 1}-{uuid.uuid4().hex[:4]}"
        except KeyError:
            pass

        forked_context = {
            **base_context,
            "_split_from": target_id,
            "_split_strategy": split_strategy,
            "_split_index": i,
            "_split_snapshot": snapshot_id,
            # 守门 #13 d: 父 snapshot ID 注入 fork, 方便后续回溯
        }

        await sub_pool.spawn(
            task_type=task_type,
            context=forked_context,
            task_id=candidate,
        )
        new_task_ids.append(candidate)
        logger.info(
            "split_node dispatch: %s (type=%s idx=%d strategy=%s)",
            candidate, task_type, i, split_strategy,
        )
    return new_task_ids


async def _mark_target_superseded_with_split_into(
    target_id: str,
    new_task_ids: Sequence[str],
    sub_pool,
) -> None:
    """M-N2 步骤 4: mark a superseded + a.split_into = [a1..aN] (守门 #13 d)

    守门:
      - 守门 #13 d: supersede 终态, 永存, 不删除
      - split_into 跟 superseded_by 互斥 (split 不指向"取代", 而是指向"分叉")
    """
    await sub_pool.update(target_id, {
        "status": "superseded",
        "split_into": list(new_task_ids),
        "superseded_by": None,  # split 没"取代"指向, 是 split_into (per 03 §3.2.1.1 注释)
        "superseded_at": time.time(),
    })
    logger.info(
        "split_node mark_superseded: task=%s split_into=%s",
        target_id, new_task_ids,
    )


def _emit_ui_events(
    target_id: str,
    task_type: str,
    new_task_ids: Sequence[str],
    snapshot_id: str,
    split_strategy: str,
) -> list[dict]:
    """M-N2 步骤 5: emit UI events (1 × TaskCardUpdate a + N × TaskCardCreate a1..aN)

    守门 #24: 调试控制台走 subprocess, 不直接 RPC
    本步骤 emit 事件到 mock UI stream (Next.js 通过 /api/tmo/split 拉)
    """
    events: list[dict] = []
    # 1 × TaskCardUpdate (a 状态变 superseded + split_into)
    events.append({
        "type": "TaskCardUpdate",
        "task_id": target_id,
        "patch": {
            "status": "superseded",
            "split_into": list(new_task_ids),
            "superseded_by": None,
        },
    })
    # N × TaskCardCreate (a1..aN)
    for i, tid in enumerate(new_task_ids):
        events.append({
            "type": "TaskCardCreate",
            "task_id": tid,
            "card": {
                "task_type": task_type,  # 跟父 task_type 保持一致
                "status": "running",
                "split_from": target_id,
                "split_index": i,
                "split_strategy": split_strategy,
                "split_snapshot_id": snapshot_id,
            },
        })
    logger.info("split_node emit_ui_events: %d events (1 update + %d create)", len(events), len(new_task_ids))
    return events


# ===== 主函数: split_node =====

async def split_node(state: dict, manager) -> dict:
    """TMO M-N2: 拆分 a → a1 + a2 + ... (per 03 §3.2.1.1)

    输入 (state = SplitRequest dict, per protocols.py):
      operation: "split"
      target_task_id: a (拆分的源)
      split_strategy: "context_fork" (default) | "checkpoint_fork"
      split_count: 2 (default) — 守门 [2, 8]
      actor_session_id: L0 session id

    输出 (TopAgentState 增量更新, per 03 §3.2.1.1):
      superseded_tasks: [target_id] (reducer operator.add append-only)
      active_tmo_operation: None
      global_context: {last_tmo_result: {operation: split, target_task_id, snapshot_checkpoint_id, new_task_ids, ...}}
      ui_events: 1 + N 个 UI 事件 (1 TaskCardUpdate a + N TaskCardCreate a1..aN)
      new_task_ids: [a1, a2, ...]
      snapshot_checkpoint_id: snapshot id

    守门:
      - 守门 #13 a: L0 唯一入口, 跨 L1 task 操作只经 L0
      - 守门 #13 d: snapshot 永存 (Transaction append-only), supersede 终态不删除
      - 守门 #19: Python 化, 不写 .rs
      - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
    """
    target_id = state.get("target_task_id")
    split_strategy = state.get("split_strategy", "context_fork")
    split_count = state.get("split_count", DEFAULT_SPLIT_COUNT)
    actor_session_id = state.get("actor_session_id")

    logger.info(
        "split_node start: target=%s strategy=%s count=%d actor=%s",
        target_id, split_strategy, split_count, actor_session_id,
    )

    sub_pool = manager.sub_pool

    # 步骤 1: validate (含 target_id 非空校验)
    task_type, base_context = _validate_split_request(
        target_id=target_id,
        split_strategy=split_strategy,
        split_count=split_count,
        sub_pool=sub_pool,
    )

    # 步骤 2: snapshot a (Transaction append-only per 守门 #13 d)
    snapshot_id = await _snapshot_target(target_id, sub_pool)

    # 步骤 3: dispatch a1..aN (相同 task_type as a, forked context)
    new_task_ids = await _dispatch_fork_tasks(
        target_id=target_id,
        snapshot_id=snapshot_id,
        task_type=task_type,
        base_context=base_context,
        split_strategy=split_strategy,
        split_count=split_count,
        sub_pool=sub_pool,
    )

    # 步骤 4: mark a superseded + split_into
    await _mark_target_superseded_with_split_into(target_id, new_task_ids, sub_pool)

    # 步骤 5: emit UI events
    ui_events = _emit_ui_events(
        target_id=target_id,
        task_type=task_type,
        new_task_ids=new_task_ids,
        snapshot_id=snapshot_id,
        split_strategy=split_strategy,
    )

    # 返回 TopAgentState 增量 (per 03 §3.2.1.1)
    return {
        "superseded_tasks": [target_id],  # reducer operator.add (append-only)
        "active_tmo_operation": None,  # TMO operation done
        "global_context": {
            "last_tmo_result": {
                "operation": "split",
                "target_task_id": target_id,
                "snapshot_checkpoint_id": snapshot_id,
                "new_task_ids": list(new_task_ids),
                "split_strategy": split_strategy,
                "split_count": split_count,
            }
        },
        "ui_events": ui_events,
        "new_task_ids": list(new_task_ids),
        "snapshot_checkpoint_id": snapshot_id,
    }


# ===== 同步 wrapper (用于非 async context) =====

def split_node_sync(state: dict, manager) -> dict:
    """split_node 同步 wrapper (用于 FastAPI handler 等非 async 场景)"""
    return asyncio.run(split_node(state, manager))
