# scripts/automation/task_ops/nodes/reassign_node.py
# M-N6 reassign_node (TMO-06, per 02-basic-design.md v0.2 §2.6)
#
# 职责:
#   - 重新分配 task 给不同 SA-XX 类型 (L0 唯一入口)
#   - 守门 #13 a: 跨 SA 类型切换需 L0 协调
#   - 守门 #13 d: checkpoint preserved (L0 切换 SA 不丢状态)
#   - 守门 #19: Python 化
#
# 输入: ReassignRequest (per protocols.py)
# 输出: ReassignResponse (old_sa + new_sa + checkpoint_id + worktree 迁移)
#
# 阻塞 (per G-DEP-01 P0 工具): create_worktree (新 worktree 创建/迁移)
# PoC stub: 不真迁移 worktree, 仅改 task_type + checkpoint stash

from __future__ import annotations

import logging
import time
import uuid
from typing import Optional

logger = logging.getLogger("task_ops.reassign")

# 合法 SA 类型 (per 02 §2.1.3, 9 + SA-10)
VALID_SA_TYPES = (
    "SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
    "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
)


async def reassign_node(
    sub_agent_pool,  # SubAgentPool 实例 (L0 唯一入口 per 守门 #13 a)
    request: dict,
) -> dict:
    """M-N6 reassign_node stub (per TMO-06 7 子项实装 phase)

    最小可行骨架 (per V2-6 5 子代理 + Mavis 跨域协调模式):
      1. 验证 task_id 存在
      2. 验证 new_sa_type 在 VALID_SA_TYPES 内
      3. checkpoint stash (preserved per 守门 #13 d)
      4. 改 task_type (L0 唯一入口)
      5. worktree 迁移 (stub: 仅记录, 真实接入 create_worktree P0 工具)
    """
    task_id = request.get("task_id")
    new_sa_type = request.get("new_sa_type")
    actor_session_id = request.get("actor_session_id")

    if not task_id:
        raise ValueError("reassign_request: task_id required")
    if new_sa_type not in VALID_SA_TYPES:
        raise ValueError(f"reassign_request: new_sa_type must be one of {VALID_SA_TYPES}, got {new_sa_type}")

    # 1. 取 task + checkpoint stash (preserved)
    task = sub_agent_pool.get(task_id)
    old_sa_type = task.task_type
    preserved_checkpoint = await sub_agent_pool.checkpoint(task_id, f"reassign:{old_sa_type}->{new_sa_type}")

    # 2. 改 task_type (L0 唯一入口)
    await sub_agent_pool.update(task_id, {
        "task_type": new_sa_type,
        "reassigned_at": time.time(),
        "reassigned_from": old_sa_type,
    })

    # 3. worktree 迁移 (stub: 仅记录, 真实接入 create_worktree P0 工具 per G-DEP-01)
    worktree_migration = {
        "old_worktree": task.state.get("worktree_id"),
        "new_worktree": None,  # stub: 真实由 create_worktree tool 落地
        "migration_status": "pending",  # 真实工具接入后改为 "completed"
    }

    logger.info(f"M-N6 reassign_node: {task_id} {old_sa_type} -> {new_sa_type}")

    return {
        "operation": "reassign",
        "task_id": task_id,
        "old_sa_type": old_sa_type,
        "new_sa_type": new_sa_type,
        "checkpoint_id": preserved_checkpoint,
        "worktree_migration": worktree_migration,
        "actor_session_id": actor_session_id,
        "completed_at_ms": int(time.time() * 1000),
    }
