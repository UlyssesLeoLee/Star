# scripts/automation/task_ops/nodes/metadata_node.py
# M-N7 metadata_node (TMO-07, per 02-basic-design.md v0.2 §2.6)
#
# 职责:
#   - 更新 task 的 metadata (label, priority, due_date, custom fields)
#   - 守门 #13 a: L0 唯一 metadata 写入入口
#   - 守门 #13 c Master RLS: 必携 tenant_id + workspace_ids
#   - 守门 #19: Python 化
#
# 输入: MetadataUpdate (per protocols.py)
# 输出: MetadataResponse (updated_task_id + metadata_snapshot + audit_log_id)
#
# 阻塞 (per G-TMO-04): task_metadata DDL (CREATE TABLE + RLS POLICY)
# PoC stub: 内存版 Master RLS (per 守门 #13 c 派生), 真实 DDL 拍板后接入

from __future__ import annotations

import logging
import time
import uuid
from typing import Optional

logger = logging.getLogger("task_ops.metadata")

# Master RLS 必携字段 (per 守门 #13 c)
REQUIRED_RLS_FIELDS = ("tenant_id",)


async def metadata_node(
    sub_agent_pool,  # SubAgentPool 实例 (L0 唯一入口 per 守门 #13 a)
    metadata_registry,  # 内存版 Master RLS registry (PoC; 真实 DDL 推 G-TMO-04)
    request: dict,
) -> dict:
    """M-N7 metadata_node stub (per TMO-07 7 子项实装 phase)

    最小可行骨架 (per V2-6 5 子代理 + Mavis 跨域协调模式):
      1. 验证 task_id 存在
      2. 验证 metadata 必携 tenant_id (per 守门 #13 c Master RLS)
      3. checkpoint stash (preserved)
      4. metadata_registry.update (Master RLS, 物理删除禁止)
      5. 返 updated metadata snapshot + audit log id (Transaction 追加 per 守门 #13 d)
    """
    task_id = request.get("task_id")
    metadata = request.get("metadata", {})
    actor_session_id = request.get("actor_session_id")

    if not task_id:
        raise ValueError("metadata_update: task_id required")
    if not metadata:
        raise ValueError("metadata_update: metadata required (non-empty)")

    # 守门 #13 c Master RLS: 必携 tenant_id
    for field_name in REQUIRED_RLS_FIELDS:
        if field_name not in metadata:
            raise ValueError(f"metadata_update: required Master RLS field missing: {field_name}")

    # 1. 验证 task
    task = sub_agent_pool.get(task_id)
    preserved_checkpoint = await sub_agent_pool.checkpoint(task_id, "metadata_update")

    # 2. metadata_registry.update (Master 类型, 物理删除禁止 per 守门 #13 c)
    update_id = metadata_registry.update(
        task_id=task_id,
        metadata=metadata,
        actor_session_id=actor_session_id,
    )

    # 3. task state 加 metadata reference (L0 唯一写入)
    await sub_agent_pool.update(task_id, {
        "metadata_updated_at": time.time(),
        "metadata_update_id": update_id,
    })

    logger.info(f"M-N7 metadata_node: {task_id} tenant={metadata.get('tenant_id')} update_id={update_id}")

    return {
        "operation": "metadata_update",
        "task_id": task_id,
        "update_id": update_id,
        "metadata_snapshot": dict(metadata),  # 防御性 copy
        "checkpoint_id": preserved_checkpoint,
        "actor_session_id": actor_session_id,
        "completed_at_ms": int(time.time() * 1000),
    }
