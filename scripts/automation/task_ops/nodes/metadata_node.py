# scripts/automation/task_ops/nodes/metadata_node.py
# TMO M-N7 metadata_node (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §3.2.1.1)
#
# 职责: 更新 task 的 metadata (name / labels / notes / priority)
# 方向: L0 → L0 (per 02-basic-design.md §2.6.2) 内部状态更新
# 协议: MetadataUpdate TypedDict (per protocols.py)
#
# 流程:
#   1. validate: target_task_id 存在 + tenant_id / workspace_ids 必携 (守门 #13 c)
#   2. validate: metadata ∈ {name?, labels?, notes?, priority?}
#   3. SCD Type 2: 旧 metadata snapshot 落档 (per 守门 #13 c Master SCD Type 2)
#   4. 应用新 metadata 到 state
#   5. RLS check: tenant_id 跟当前 task 一致 (守门 #13 c)
#   6. ui_streamer.push × 1 (1 × TaskCardUpdate metadata 字段)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口 (跨任务 metadata 只经 L0)
#   - 守门 #13 c: Master RLS 必携 tenant_id / workspace_ids
#   - 守门 #13 d: SCD Type 2 旧 metadata 永存 (Transaction append-only)
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #22: 调试控制台走 port 8080 console_server.py, 不污染 main 编译链
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#
# 派发 (per 守门 #20): 实装前必先 brief 落档 (TMO-07 父会话 Mavis 委托)

from __future__ import annotations

import logging
import os
import time
import uuid
from typing import Any, Dict, List, Optional, Sequence

logger = logging.getLogger("task_ops.nodes.metadata_node")

# 可选持久化开关 (per G-TMO-04d, 默认关闭 — 保持 in-memory 兼容)
# 设置 STAR_TASK_METADATA_PERSIST=1 + STAR_TASK_METADATA_DB_PATH=... 启用 SQLite 持久化
_TASK_METADATA_PERSIST: bool = os.environ.get("STAR_TASK_METADATA_PERSIST", "0") == "1"


# ===== 常量 (per 03 §3.2.1.1 + 02 §2.6.4) =====

VALID_METADATA_FIELDS: frozenset[str] = frozenset({"name", "labels", "notes", "priority"})
"""M-N7 metadata 合法字段 (per 03 §3.2.1.1).

守门: 防止误写 status / task_type / 任何关键字段, 那些只能通过专属 node (split / reassign / ...) 改.
"""

MIN_PRIORITY: int = 1
MAX_PRIORITY: int = 10
"""M-N7 priority 合法范围 (per 03 §3.2.1.1): 1 (low) - 10 (urgent)."""

MAX_LABELS_PER_TASK: int = 20
MAX_LABEL_LENGTH: int = 64
"""M-N7 labels 守门 (per 03 §3.2.1.1 注释): 防止 label 爆量.

守门: 防止 tags 列表无限增长 / 标签过长难以 UI 展示.
"""

MAX_NOTES_LENGTH: int = 4096
MAX_NAME_LENGTH: int = 256
"""M-N7 name / notes 长度上限 (per 03 §3.2.1.1 注释)."""


# ===== 辅助函数 =====

def _validate_metadata_request(
    target_id: str,
    metadata: Dict,
    tenant_id: Optional[str],
    workspace_ids: Optional[List[str]],
    sub_pool,
) -> Dict:
    """M-N7 步骤 1+2+5: validate (含 RLS check)

    守门:
      - target_id 非空 + 存在
      - tenant_id 必携 (守门 #13 c Master RLS)
      - workspace_ids 必携非空 (守门 #13 c)
      - metadata keys ⊆ VALID_METADATA_FIELDS
      - 守门 #13 c RLS check: tenant_id 跟当前 task.tenant_id 一致
      - priority ∈ [MIN_PRIORITY, MAX_PRIORITY]
      - labels 数量 ≤ MAX_LABELS_PER_TASK
      - name / notes / labels 单值长度 ≤ 上限

    Returns:
        handle (sub_pool.get(target_id))
    """
    if not target_id:
        raise ValueError("metadata_node: target_task_id is required")

    if not tenant_id:
        raise ValueError(
            "metadata_node: tenant_id is required (per 守门 #13 c Master RLS 必携)"
        )
    if not workspace_ids:
        raise ValueError(
            "metadata_node: workspace_ids is required (per 守门 #13 c Master RLS 必携)"
        )

    if not isinstance(metadata, dict) or not metadata:
        raise ValueError("metadata_node: metadata is required (non-empty dict)")

    unknown_fields = set(metadata.keys()) - VALID_METADATA_FIELDS
    if unknown_fields:
        raise ValueError(
            f"metadata_node: unknown metadata fields {unknown_fields!r}, "
            f"expected subset of {set(VALID_METADATA_FIELDS)}"
        )

    # 字段级守门
    if "priority" in metadata:
        p = metadata["priority"]
        if not isinstance(p, int) or p < MIN_PRIORITY or p > MAX_PRIORITY:
            raise ValueError(
                f"metadata_node: priority {p!r} must be int in [{MIN_PRIORITY}, {MAX_PRIORITY}]"
            )

    if "labels" in metadata:
        labels = metadata["labels"]
        if not isinstance(labels, list):
            raise ValueError("metadata_node: labels must be list[str]")
        if len(labels) > MAX_LABELS_PER_TASK:
            raise ValueError(
                f"metadata_node: labels count {len(labels)} > MAX {MAX_LABELS_PER_TASK}"
            )
        for i, label in enumerate(labels):
            if not isinstance(label, str):
                raise ValueError(f"metadata_node: labels[{i}] must be str, got {type(label).__name__}")
            if len(label) > MAX_LABEL_LENGTH:
                raise ValueError(
                    f"metadata_node: labels[{i}] length {len(label)} > MAX_LABEL_LENGTH {MAX_LABEL_LENGTH}"
                )

    if "name" in metadata:
        name = metadata["name"]
        if not isinstance(name, str):
            raise ValueError("metadata_node: name must be str")
        if len(name) > MAX_NAME_LENGTH:
            raise ValueError(
                f"metadata_node: name length {len(name)} > MAX_NAME_LENGTH {MAX_NAME_LENGTH}"
            )

    if "notes" in metadata:
        notes = metadata["notes"]
        if not isinstance(notes, str):
            raise ValueError("metadata_node: notes must be str")
        if len(notes) > MAX_NOTES_LENGTH:
            raise ValueError(
                f"metadata_node: notes length {len(notes)} > MAX_NOTES_LENGTH {MAX_NOTES_LENGTH}"
            )

    # RLS check (守门 #13 c)
    handle = sub_pool.get(target_id)
    task_tenant_id = handle.state.get("tenant_id")
    if task_tenant_id and task_tenant_id != tenant_id:
        raise PermissionError(
            f"metadata_node: RLS check failed, task.tenant_id={task_tenant_id!r} != request.tenant_id={tenant_id!r} (per 守门 #13 c)"
        )

    return handle


def _snapshot_previous_metadata(
    target_id: str,
    handle,
) -> Optional[Dict]:
    """M-N7 步骤 3: SCD Type 2 旧 metadata 快照 (per 守门 #13 c Master SCD Type 2)

    Returns:
        previous_metadata dict (or None if not exist)
    """
    prev = handle.state.get("metadata", {})
    if not prev:
        return None
    snapshot = {
        "snapshot_id": f"metadata-scd-{uuid.uuid4().hex[:8]}",
        "task_id": target_id,
        "previous_metadata": dict(prev),
        "snapshot_at": time.time(),
    }
    logger.info(
        "metadata_node scd_snapshot: task=%s snapshot_id=%s",
        target_id, snapshot["snapshot_id"],
    )
    return snapshot


def _merge_metadata(
    target_id: str,
    new_metadata: Dict,
    prev_snapshot: Optional[Dict],
    handle,
) -> Dict:
    """M-N7 步骤 4: 合并新 metadata 到 state.metadata (保留 SCD 历史链)

    守门:
      - 守门 #13 c: SCD Type 2 关系变更留痕
      - 守门 #13 d: scd_history 永存 (Transaction append-only)
    """
    merged = dict(handle.state.get("metadata") or {})
    merged.update(new_metadata)

    # 维护 scd_history (Transaction append-only)
    scd_history = list(handle.state.get("metadata_scd_history") or [])
    if prev_snapshot is not None:
        scd_history.append(prev_snapshot)

    return {
        "metadata": merged,
        "metadata_scd_history": scd_history,
    }


def _emit_ui_events(
    target_id: str,
    new_metadata: Dict,
) -> list[dict]:
    """M-N7 步骤 6: emit UI events (1 × TaskCardUpdate metadata 字段)

    守门 #24: 调试控制台走 subprocess, 不直接 RPC.
    """
    events: list[dict] = [{
        "type": "TaskCardUpdate",
        "task_id": target_id,
        "patch": {
            "metadata": new_metadata,
        },
    }]
    logger.info("metadata_node emit_ui_events: 1 event (TaskCardUpdate metadata)")
    return events


# ===== 主函数: metadata_node =====


def _persist_to_sqlite(
    target_id: str,
    tenant_id: str,
    workspace_id: str,
    metadata: Dict[str, Any],
    actor_session_id: Optional[str],
) -> Dict[str, Any]:
    """委托 TaskMetadataRepository.upsert_metadata (per G-TMO-04d + G-TMO-04b).

    守门:
      - 守门 #13 c: Master RLS 必携 tenant_id + workspace_id (Pydantic 已校验)
      - 守门 #13 d: SCD Type 2 (旧 version is_current=0 + 新 version is_current=1) 走 repo
      - 守门 #19: Python 化, 标准库 sqlite3
      - 守门 #22: 调试控制台不污染 main 编译链
    """
    # 延迟 import (避免在 in-memory 模式加载 repo 模块, 守门 #22 不进 main 编译链)
    from automation.task_ops.task_metadata_ddl import init_schema
    from automation.task_ops.task_metadata_repo import TaskMetadataRepository

    db_path = os.environ.get(
        "STAR_TASK_METADATA_DB_PATH",
        str(os.path.join(os.getcwd(), "data", "task_metadata.sqlite")),
    )
    init_schema(db_path)  # idempotent
    repo = TaskMetadataRepository(db_path)
    record = repo.upsert_metadata(
        task_id=target_id,
        tenant_id=tenant_id,
        workspace_id=workspace_id,
        metadata=metadata,
        actor_session_id=actor_session_id,
    )
    audit = repo.get_audit_log(
        task_id=target_id, tenant_id=tenant_id, workspace_id=workspace_id, limit=100,
    )
    return {
        "version": record.version,
        "scd_snapshot_id": None,  # repo 内部派生, 详细 ID 不暴露
        "audit_count": len(audit),
    }


async def metadata_node(state: dict, manager) -> dict:
    """TMO M-N7: 更新 task metadata (per 03 §3.2.1.1)

    输入 (state = MetadataUpdate TypedDict, per protocols.py):
      operation: "metadata"
      target_task_id: 单一 task_id
      metadata: {name?, labels?, notes?, priority?}
      tenant_id: 必携 (守门 #13 c Master RLS)
      workspace_ids: 必携非空 (守门 #13 c)
      actor_session_id: 发起者 session_id

    输出 (TopAgentState 增量更新, per 03 §3.2.1.1):
      superseded_tasks: [] (无副作用)
      active_tmo_operation: None
      global_context: {last_tmo_result: {operation, target_task_id, updated_fields, scd_snapshot_id, ...}}
      ui_events: 1 个 TaskCardUpdate (metadata 字段)
      updated_fields: list[str] 实际更新的字段
      scd_snapshot_id: 旧 metadata snapshot id (per 守门 #13 c SCD Type 2)

    守门:
      - 守门 #13 a: L0 唯一入口
      - 守门 #13 c: Master RLS 必携 tenant_id / workspace_ids + SCD Type 2
      - 守门 #13 d: SCD history 永存
      - 守门 #19: Python 化
    """
    target_id: str = state.get("target_task_id") or ""
    metadata: Dict = state.get("metadata") or {}
    tenant_id: Optional[str] = state.get("tenant_id")
    workspace_ids: Optional[List[str]] = state.get("workspace_ids")
    actor_session_id: Optional[str] = state.get("actor_session_id")

    logger.info(
        "metadata_node start: target=%s fields=%s tenant=%s actor=%s",
        target_id, sorted(metadata.keys()) if metadata else [], tenant_id, actor_session_id,
    )

    sub_pool = manager.sub_pool

    # 步骤 1+2+5: validate (含 RLS check)
    handle = _validate_metadata_request(
        target_id=target_id,
        metadata=metadata,
        tenant_id=tenant_id,
        workspace_ids=workspace_ids,
        sub_pool=sub_pool,
    )

    # 步骤 3: SCD Type 2 旧 metadata 快照
    prev_snapshot = _snapshot_previous_metadata(target_id, handle)
    scd_snapshot_id = prev_snapshot["snapshot_id"] if prev_snapshot else None

    # 步骤 4: 合并新 metadata
    merged_state = _merge_metadata(target_id, metadata, prev_snapshot, handle)
    await sub_pool.update(target_id, merged_state)

    # 步骤 4.5: 可选 SQLite 持久化 (per G-TMO-04d, STAR_TASK_METADATA_PERSIST=1 启用)
    # 默认关闭, 保持 in-memory 兼容. 开启后委托 TaskMetadataRepository.upsert_metadata,
    # 走 SCD Type 2 + audit 5 类事件 (per 守门 #13 d + 守门 #DB-13).
    persist_result: Optional[Dict[str, Any]] = None
    if _TASK_METADATA_PERSIST:
        try:
            persist_result = _persist_to_sqlite(
                target_id=target_id,
                tenant_id=tenant_id,
                workspace_id=workspace_ids[0] if workspace_ids else "default",
                metadata=metadata,
                actor_session_id=actor_session_id,
            )
            logger.info(
                "metadata_node persist: task=%s version=%s scd=%s",
                target_id,
                persist_result.get("version"),
                persist_result.get("scd_snapshot_id"),
            )
        except Exception as exc:
            # 持久化失败不应破坏 in-memory 状态 (per 守门 #22 + 守门 #19 优雅降级)
            logger.warning(
                "metadata_node persist failed (fallback to in-memory only): task=%s err=%s",
                target_id, exc,
            )

    # 步骤 6: emit UI events
    ui_events = _emit_ui_events(target_id, metadata)

    result: dict = {
        "operation": "metadata",
        "target_task_id": target_id,
        "updated_fields": sorted(metadata.keys()),
        "scd_snapshot_id": scd_snapshot_id,
        "ui_events": ui_events,
        "active_tmo_operation": None,
    }
    if persist_result is not None:
        result["persisted"] = {
            "backend": "sqlite_task_metadata",
            "version": persist_result.get("version"),
            "scd_snapshot_id": persist_result.get("scd_snapshot_id"),
            "audit_count": persist_result.get("audit_count"),
        }
    logger.info(
        "metadata_node done: task=%s fields=%s scd_snapshot=%s",
        target_id, result["updated_fields"], scd_snapshot_id,
    )
    return result
