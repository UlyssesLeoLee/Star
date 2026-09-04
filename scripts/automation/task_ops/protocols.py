# scripts/automation/task_ops/protocols.py
# TMO 7 协议 TypedDict (per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.2)
#
# 方向: L0→L1 (5 类) + L0→L0 (1 类) + L0→UI (1 类)
#   - merge_request (M-N1)        L0 → L1
#   - split_request (M-N2)        L0 → L1
#   - dep_set (M-N3)              L0 → L1
#   - bulk_action (M-N4)          L0 → L1
#   - reassign_request (M-N6)     L0 → L1
#   - metadata_update (M-N7)      L0 → L0
#   - summarize_result (M-N5)     L0 → UI
#
# 约束 (per 守门 #13 a L1↔L1 禁止): 跨任务操作只经 L0 协调 (TaskOperationsManager C-16)
# 约束 (per 守门 #13 d Transaction): checkpoint stash_id 全部 append-only
# 约束 (per 守门 #13 c Master RLS): metadata_update 必携 tenant_id / workspace_ids

from __future__ import annotations

from typing import Literal, Optional, TypedDict


# ===== M-N1 merge_request =====

class MergeRequest(TypedDict, total=False):
    """L0 → L1 合并请求 (M-N1 merge_node)

    字段:
      operation: 固定 "merge"
      target_task_ids: 至少 2 个 task_id (守门)
      merge_strategy: 合并策略, 默认 "context_union"
        - "context_union": 拼接 context (默认)
        - "checkpoint_union": 拼接 checkpoint history
        - "label_priority": 按 label 优先级合并
      original_user_input: 用户原始 chat bar 输入 (e.g. "合并任务 a 和任务 b")
      actor_session_id: 发起者 session_id (L0 chat bar session)
    """
    operation: Literal["merge"]
    target_task_ids: list[str]  # ≥ 2
    merge_strategy: str  # default "context_union"
    original_user_input: Optional[str]
    actor_session_id: Optional[str]


class MergeResponse(TypedDict, total=False):
    """merge_node 内部返回 (TopAgentState 增量更新, per 03 §3.2.1.1)"""
    superseded_tasks: list[str]  # reducer operator.add (append-only)
    active_tmo_operation: None  # TMO operation done
    global_context: dict  # {last_tmo_result: {...}}
    merged_task_id: str
    stash_checkpoint_ids: list[str]


# ===== M-N2 split_request =====

class SplitRequest(TypedDict, total=False):
    """L0 → L1 拆分请求 (M-N2 split_node)

    字段:
      operation: 固定 "split"
      target_task_id: 单一 task_id (拆分的源)
      split_strategy: "context_fork" | "checkpoint_fork"
      split_count: 拆分份数 (默认 2, 守门 ≥ 2)
      actor_session_id: 发起者 session_id
    """
    operation: Literal["split"]
    target_task_id: str
    split_strategy: str  # default "context_fork"
    split_count: int  # default 2
    actor_session_id: Optional[str]


# ===== M-N3 dep_set =====

class DepSet(TypedDict, total=False):
    """L0 → L1 依赖 DAG 边更新 (M-N3 reorder_node)

    字段:
      operation: 固定 "dep_set"
      dep_set: DAG 边集合, [(from_task_id, to_task_id), ...]
      actor_session_id: 发起者 session_id

    守门 #13 a 强约束: dep_set 必通过 DAGValidator C-20 cycle detection
    """
    operation: Literal["dep_set"]
    dep_set: list[tuple[str, str]]  # [(from, to), ...]
    actor_session_id: Optional[str]


# ===== M-N4 bulk_action =====

class BulkAction(TypedDict, total=False):
    """L0 → L1 批量操作 (M-N4 bulk_node)

    字段:
      operation: 固定 "bulk_action"
      target_task_ids: N 张卡 (N ≥ 1)
      action: "pause" | "resume" | "cancel" | "set_priority"
      payload: action 特定参数 (e.g. set_priority → {priority: 5})
      actor_session_id: 发起者 session_id
    """
    operation: Literal["bulk_action"]
    target_task_ids: list[str]  # ≥ 1
    action: str  # pause / resume / cancel / set_priority
    payload: dict  # action-specific
    actor_session_id: Optional[str]


# ===== M-N6 reassign_request =====

class ReassignRequest(TypedDict, total=False):
    """L0 → L1 重分配请求 (M-N6 reassign_node)

    字段:
      operation: 固定 "reassign"
      target_task_id: 单一 task_id
      new_task_type: 新 SA 类型 (SA-01..SA-10)
      preserved_checkpoint_id: 保留的 checkpoint (ReassignManager C-21)
    """
    operation: Literal["reassign"]
    target_task_id: str
    new_task_type: str  # SA-01..SA-10
    preserved_checkpoint_id: Optional[str]
    actor_session_id: Optional[str]


# ===== M-N7 metadata_update =====

class MetadataUpdate(TypedDict, total=False):
    """L0 → L0 元数据更新 (M-N7 metadata_node)

    字段:
      operation: 固定 "metadata"
      target_task_id: 单一 task_id
      metadata: name / labels / notes / priority 字典
      tenant_id: 必携 (守门 #13 c Master RLS)
      workspace_ids: 必携 (守门 #13 c)
    """
    operation: Literal["metadata"]
    target_task_id: str
    metadata: dict  # {name?, labels?, notes?, priority?}
    tenant_id: str  # Master RLS 必携
    workspace_ids: list[str]  # Master RLS 必携


# ===== M-N5 summarize_result =====

class TaskSummary(TypedDict, total=False):
    """单任务汇总 (summarize_result 数组元素)"""
    task_id: str
    task_type: str  # SA-XX
    status: str  # running / completed / failed / superseded
    summary: str  # LLM 生成的 summary
    token_usage: dict  # {input, output, total}


class SummarizeResult(TypedDict, total=False):
    """L0 → UI 跨任务汇总 (M-N5 summarize_node)

    字段:
      operation: 固定 "summarize"
      target_task_ids: 被汇总的 task_id 列表
      task_summaries: TaskSummary 数组
    """
    operation: Literal["summarize"]
    target_task_ids: list[str]
    task_summaries: list[TaskSummary]


# ===== 协议联合类型 (用于 type hints) =====

TMOMessage = MergeRequest | SplitRequest | DepSet | BulkAction | ReassignRequest | MetadataUpdate | SummarizeResult
"""所有 TMO 7 协议的联合类型, 用于 TMO 路由判定"""

TMO_OPERATION_TYPES = ("merge", "split", "dep_set", "bulk_action", "reassign", "metadata", "summarize")
"""TMO 7 操作类型字面量 (per 02 §2.6.3 路由表)"""
