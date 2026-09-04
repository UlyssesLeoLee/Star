# scripts/automation/task_ops/nodes/split_node.py
# M-N2 split_node (TMO-02, per 02-basic-design.md v0.2 §2.6)
#
# 职责:
#   - 拆分 1 个 task 到 N 个子 task
#   - 守门 #13 a: L0 唯一拆分入口 (子任务 L1 sub-agent)
#   - 守门 #13 d: original task 标 superseded, 新 task 标 active
#   - 守门 #19: Python 化 (不碰 cargo 链)
#
# 输入: SplitRequest (per protocols.py)
# 输出: SplitResponse (supernumerary_task_ids + parent_superseded + checkpoint_ids)
#
# 跟 M-N1 merge 镜像设计: stub node + SubAgentPool.spawn + checkpoint stash
# 真实拆分配额 (per 03 §3.2.2) 待 L0 全局依赖跨子任务 DAG 拍板 (推下 session)

from __future__ import annotations

import logging
import time
import uuid
from typing import Optional

logger = logging.getLogger("task_ops.split")

# 拆分策略 (per 02 §2.6.4)
SPLIT_STRATEGIES = ("context_fork", "checkpoint_fork")


async def split_node(
    sub_agent_pool,  # SubAgentPool 实例 (L0 唯一入口 per 守门 #13 a)
    relationship_graph,  # TaskRelationshipGraph (DAG, 跟踪拆分后父子关系)
    request: dict,
) -> dict:
    """M-N2 split_node stub (per TMO-02 7 子项实装 phase)

    最小可行骨架 (per V2-6 5 子代理 + Mavis 跨域协调模式):
      1. 验证 target_task_id 存在
      2. 验证 split_strategy 在 SPLIT_STRATEGIES 内
      3. 标 original task 为 superseded (L0 唯一入口)
      4. spawn N 个 child sub-agent (mock 模式 L0 同步, 真实模式异步)
      5. relationship_graph 加 parent → child edge
      6. checkpoint stash (Transaction append-only per 守门 #13 d)
    """
    target_task_id = request.get("target_task_id")
    split_strategy = request.get("split_strategy", "context_fork")
    actor_session_id = request.get("actor_session_id")

    if not target_task_id:
        raise ValueError("split_request: target_task_id required")
    if split_strategy not in SPLIT_STRATEGIES:
        raise ValueError(f"split_request: split_strategy must be one of {SPLIT_STRATEGIES}, got {split_strategy}")

    # 1. 验证 target 存在
    parent = sub_agent_pool.get(target_task_id)
    parent_state = parent.state
    children_count = parent_state.get("split_children_count", 0) or 2  # 默认拆 2 个

    # 2. 标 original task superseded
    await sub_agent_pool.update(target_task_id, {"status": "superseded", "superseded_at": time.time()})
    parent_checkpoint = await sub_agent_pool.checkpoint(target_task_id, f"split:{split_strategy}")

    # 3. spawn N 个 child (mock 模式 L0 同步, per 守门 #9 v3 fallback)
    child_ids = []
    for i in range(children_count):
        child_id = f"{target_task_id}-child-{i}-{uuid.uuid4().hex[:4]}"
        # fork context (mock: 共享 parent context)
        if split_strategy == "context_fork":
            child_context = dict(parent_state.get("context", {}))
        else:  # "checkpoint_fork"
            child_context = {"parent_checkpoints": list(parent.checkpoints)}
        await sub_agent_pool.spawn(
            task_type=parent.task_type,  # 继承
            context=child_context,
            task_id=child_id,
        )
        child_ids.append(child_id)

    # 4. DAG 关系 (per 守门 #13 a, L0 唯一)
    relationship_graph.add_children(target_task_id, child_ids, split_strategy)

    logger.info(f"M-N2 split_node: {target_task_id} -> {len(child_ids)} children ({split_strategy})")

    return {
        "operation": "split",
        "parent_task_id": target_task_id,
        "child_task_ids": child_ids,
        "parent_superseded": True,
        "split_strategy": split_strategy,
        "checkpoint_id": parent_checkpoint,
        "actor_session_id": actor_session_id,
        "completed_at_ms": int(time.time() * 1000),
    }
