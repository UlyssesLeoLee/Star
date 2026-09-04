# scripts/automation/api/routes_tmo.py
# TMO FastAPI 路由 — 合并版 (TMO-01 + TMO-03 + TMO-04 三个 worktree 实装的端点合并)
# per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §5.2 (8 外部 API 端点)
#
# 8 端点 (合并 wt-tmo-01 + wt-tmo-03 + wt-tmo-04 实装; split/summarize/reassign/metadata planned):
#   - POST /api/tmo/merge         (TMO-01 ✅ wt-tmo-01-merge)        合并任务卡 (M-N1 + SA-10)
#   - POST /api/tmo/split         (TMO-02 planned)                   拆分任务卡 (M-N2)
#   - POST /api/tmo/dependencies  (TMO-03 ✅ wt-tmo-03-dag)           依赖边管理 (M-N3 + DAGValidator)
#   - POST /api/tmo/bulk          (TMO-04 ✅ wt-tmo-04-bulk)           批量操作 (M-N4 + BulkOperationQueue)
#   - POST /api/tmo/summarize     (TMO-05 planned)                   跨任务汇总 (M-N5)
#   - POST /api/tmo/reassign      (TMO-06 planned)                   重新分配 (M-N6)
#   - POST /api/tmo/metadata      (TMO-07 planned)                   元数据编辑 (M-N7, Master RLS)
#   - GET  /api/tmo/operations    (TMO-08 stub ✅ wt-tmo-01-merge)    状态查询
#   - GET  /api/tmo/relationships (TMO-09 stub ✅ wt-tmo-03-dag)      DAG 关系查询
#
# 挂载 (per 守门 #24 v3):
#     from automation.api.routes_tmo import router
#     app.include_router(router)
#
# 约束 (per AGENTS.md §4):
#   - 守门 #1: 0 .rs 改动, Python 化 (per 守门 #19)
#   - 守门 #13 a: L0 协调, L1↔L1 禁止通信
#   - 守门 #13 d: task card 状态 = Work (短 TTL), checkpoint history = Transaction (append-only)
#   - 守门 #22: 调试控制台不污染 main (本 routes 走 port 8080 console_server.py)
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 → subprocess

from __future__ import annotations

import json
import logging
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from fastapi import APIRouter, HTTPException, Query
    from pydantic import BaseModel, Field, field_validator
except ImportError:
    print("ERROR: fastapi + pydantic not installed. pip install fastapi pydantic", file=sys.stderr)
    raise

# 跨 worktree 兼容: 4 子代理实装在独立 worktree, merge 时 _PROJECT_ROOT 兜底
_PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent.parent
if str(_PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(_PROJECT_ROOT))

logger = logging.getLogger("api.routes_tmo")


# ===========================================================================
# Router
# ===========================================================================

router = APIRouter(prefix="/api/tmo", tags=["tmo"])


# ===========================================================================
# TMO-01: /api/tmo/merge + /api/tmo/operations (wt-tmo-01-merge 实装)
# ===========================================================================

class MergeRequestBody(BaseModel):
    """POST /api/tmo/merge 请求体 (per task_ops/protocols.py MergeRequest)"""
    target_task_ids: list = Field(..., min_length=2, description="≥ 2 task_ids to merge")
    merge_strategy: Optional[str] = Field("context_union", description="context_union | checkpoint_union | label_priority")
    original_user_input: Optional[str] = Field(None, description="用户 chat bar 原始输入")
    actor_session_id: Optional[str] = Field(None, description="L0 session id")


class UIEvent(BaseModel):
    type: str
    task_id: Optional[str] = None
    patch: Optional[dict] = None
    card: Optional[dict] = None


class MergeResult(BaseModel):
    operation: str
    merged_task_id: str
    superseded_task_ids: list
    merge_strategy: str
    stash_checkpoint_ids: list


class MergeResponse(BaseModel):
    ok: bool
    node: str  # "M-N1"
    merged_task_id: Optional[str] = None
    result: Optional[MergeResult] = None
    ui_events: Optional[list] = None
    error: Optional[str] = None
    duration_ms: float


@router.post("/merge", response_model=MergeResponse)
async def tmo_merge(req: MergeRequestBody) -> MergeResponse:
    """合并任务卡 (M-N1 + SA-10)

    流程 (per 03 §3.2.1.1):
      1. validate (target_ids ≥ 2 + 都存在 + 非 superseded)
      2. stash_state (Transaction append-only per 守门 #13 d)
      3. dispatch SA-10 (L0 协调 per 守门 #13 a)
      4. mark superseded
      5. emit UI events
    """
    from automation.task_ops.manager import TaskOperationsManager
    manager = TaskOperationsManager()

    # mock 模式: 在 sub_pool 里加 N 个 L1 task
    for tid in req.target_task_ids:
        try:
            manager.sub_pool.get(tid)
        except KeyError:
            manager.sub_pool.add(task_type="SA-09", task_id=tid, initial_state={
                "status": "running",
                "context": {"description": f"task {tid} mock"},
            })

    message = {
        "operation": "merge",
        "target_task_ids": req.target_task_ids,
        "merge_strategy": req.merge_strategy,
        "original_user_input": req.original_user_input,
        "actor_session_id": req.actor_session_id,
    }
    dispatch_result = await manager.dispatch(message)

    if not dispatch_result["ok"]:
        raise HTTPException(status_code=400, detail=dispatch_result.get("error", "merge failed"))

    result = dispatch_result["result"]
    return MergeResponse(
        ok=True,
        node=dispatch_result["node"],
        merged_task_id=result.get("merged_task_id"),
        result=MergeResult(
            operation="merge",
            merged_task_id=result.get("merged_task_id", ""),
            superseded_task_ids=result.get("superseded_tasks", []),
            merge_strategy=req.merge_strategy or "context_union",
            stash_checkpoint_ids=result.get("stash_checkpoint_ids", []),
        ),
        ui_events=[UIEvent(**e) for e in result.get("ui_events", [])],
        duration_ms=dispatch_result.get("duration_ms", 0.0),
    )


@router.get("/operations")
async def tmo_operations() -> dict:
    """TMO 状态查询 (per 02 §2.6 状态查询端点, TMO-08 stub)"""
    from automation.task_ops.manager import TaskOperationsManager
    manager = TaskOperationsManager()
    return {
        "ok": True,
        "snapshot": manager.get_state_snapshot(),
        "implemented_nodes": ["M-N1", "M-N2", "M-N3", "M-N4"],
        "planned_nodes": ["M-N5", "M-N6", "M-N7"],
    }


# ===========================================================================
# TMO-02: /api/tmo/split (wt-tmo-02-split 实装, M-N2)
# ===========================================================================

# 局部 import (避免循环依赖 + 跨 worktree namespace 隔离)
# 注意: 用 scripts.automation 前缀, 跟 routes_tmo.py 其他 import 方式保持一致
# (test_tmo_bulk_dag.py 用 PROJECT_ROOT 注入 sys.path, 'scripts.automation' 形式 import 才能 work)
from scripts.automation.task_ops.nodes.split_node import (
    DEFAULT_SPLIT_COUNT,
    MAX_SPLIT_COUNT,
    MIN_SPLIT_COUNT,
    VALID_SPLIT_STRATEGIES,
    split_node,
)


# Module-level singleton manager (per 02 §2.6.5 调试控制台设计)
# 跨请求保留 sub_pool 状态, 让 e2e 测试可验证"第二次 split 同一 task 失败 (superseded 守门)"
# 跟 _GRAPH (TMO-03) / _bulk_queue (TMO-04) 同样模式
_SPLIT_MANAGER: Optional["TaskOperationsManager"] = None


def _get_split_manager():
    """获取 module-level singleton split manager (跨请求共享 sub_pool 状态)"""
    global _SPLIT_MANAGER
    if _SPLIT_MANAGER is None:
        from scripts.automation.task_ops.manager import TaskOperationsManager
        _SPLIT_MANAGER = TaskOperationsManager()
    return _SPLIT_MANAGER


class SplitRequestBody(BaseModel):
    """POST /api/tmo/split 请求体 (per task_ops/protocols.py SplitRequest)"""
    target_task_id: str = Field(..., min_length=1, description="拆分的源 task_id (a)")
    split_strategy: Optional[str] = Field(
        "context_fork", description="context_fork | checkpoint_fork"
    )
    split_count: Optional[int] = Field(
        DEFAULT_SPLIT_COUNT,
        ge=MIN_SPLIT_COUNT,
        le=MAX_SPLIT_COUNT,
        description=f"拆分份数, 守门 [{MIN_SPLIT_COUNT}, {MAX_SPLIT_COUNT}]",
    )
    actor_session_id: Optional[str] = Field(None, description="L0 session id")

    @field_validator("split_strategy")
    @classmethod
    def _validate_split_strategy(cls, v: str) -> str:
        if v not in VALID_SPLIT_STRATEGIES:
            raise ValueError(
                f"split_strategy must be one of {VALID_SPLIT_STRATEGIES}, got {v!r}"
            )
        return v


class SplitResult(BaseModel):
    """POST /api/tmo/split result (per 03 §3.2.1.1 last_tmo_result)"""
    operation: str = "split"
    target_task_id: str
    snapshot_checkpoint_id: str
    new_task_ids: list
    split_strategy: str
    split_count: int


class SplitResponse(BaseModel):
    ok: bool
    node: str  # "M-N2"
    target_task_id: Optional[str] = None
    result: Optional[SplitResult] = None
    ui_events: Optional[list] = None
    error: Optional[str] = None
    duration_ms: float


@router.post("/split", response_model=SplitResponse)
async def tmo_split(req: SplitRequestBody) -> SplitResponse:
    """拆分任务卡 (M-N2, per 03 §3.2.1.1)

    流程:
      1. validate (target_id 存在 + 非 superseded + split_count ∈ [2, 8])
      2. snapshot a 当前 checkpoint (Transaction append-only per 守门 #13 d)
      3. dispatch a1..aN (相同 task_type as a, forked context, L0 协调 per 守门 #13 a)
      4. mark a superseded + a.split_into = [a1..aN]
      5. emit UI events (1 × TaskCardUpdate + N × TaskCardCreate)
    """
    manager = _get_split_manager()  # module-level singleton, 跨请求保留 sub_pool 状态

    # mock 模式: target 不存在就 add 一个 L1 task (跟 /api/tmo/merge 行为一致)
    try:
        manager.sub_pool.get(req.target_task_id)
    except KeyError:
        manager.sub_pool.add(task_type="SA-09", task_id=req.target_task_id, initial_state={
            "status": "running",
            "context": {"description": f"task {req.target_task_id} mock"},
        })

    message = {
        "operation": "split",
        "target_task_id": req.target_task_id,
        "split_strategy": req.split_strategy or "context_fork",
        "split_count": req.split_count or DEFAULT_SPLIT_COUNT,
        "actor_session_id": req.actor_session_id,
    }
    dispatch_result = await manager.dispatch(message)

    if not dispatch_result["ok"]:
        raise HTTPException(status_code=400, detail=dispatch_result.get("error", "split failed"))

    result = dispatch_result["result"]
    return SplitResponse(
        ok=True,
        node=dispatch_result["node"],
        target_task_id=result.get("target_task_id", req.target_task_id),
        result=SplitResult(
            operation="split",
            target_task_id=result.get("target_task_id", req.target_task_id),
            snapshot_checkpoint_id=result.get("snapshot_checkpoint_id", ""),
            new_task_ids=result.get("new_task_ids", []),
            split_strategy=req.split_strategy or "context_fork",
            split_count=req.split_count or DEFAULT_SPLIT_COUNT,
        ),
        ui_events=[UIEvent(**e) for e in result.get("ui_events", [])],
        duration_ms=dispatch_result.get("duration_ms", 0.0),
    )


# ===========================================================================
# TMO-03: /api/tmo/dependencies + /api/tmo/reorder + /api/tmo/graph (wt-tmo-03-dag 实装)
# ===========================================================================

# 局部 import (跨 worktree 兼容)
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.dag_validator import DAGValidator
from scripts.automation.task_ops.nodes.reorder_node import (
    ReorderNode,
    ReorderState,
    ReorderResult,
    ReorderInterrupted,
)

# 全局 in-memory graph (per session, 重启 reset, 跨 session 续可加持久化)
_GRAPH: TaskRelationshipGraph = TaskRelationshipGraph()
_REORDER_NODE: ReorderNode = ReorderNode()


class DependencyEdge(BaseModel):
    """单条依赖边: a 依赖 b (a → b)."""
    task_id: str = Field(..., description="依赖方 (a)", min_length=1)
    depends_on: str = Field(..., description="被依赖方 (b, a 依赖 b)", min_length=1)


class DependencyAddRequest(BaseModel):
    """POST /api/tmo/dependencies 批量添加依赖边."""
    edges: Optional[List[DependencyEdge]] = Field(None, description="依赖边列表")
    # 也支持一次只声明 task + 4 字段 (per TaskRelationshipGraph 语义)
    task_id: Optional[str] = Field(None, description="单节点 task_id")
    parent_task_id: Optional[str] = Field(None)
    merged_from: Optional[List[str]] = Field(None)
    split_into: Optional[List[str]] = Field(None)
    superseded_by: Optional[List[str]] = Field(None)


class DependencyAddResponse(BaseModel):
    ok: bool
    added_edges: List[List[str]]
    total_edges: int
    total_nodes: int
    cycle_detected: bool
    cycle_path: Optional[List[str]] = None


@router.post("/dependencies", response_model=DependencyAddResponse)
async def add_dependencies(req: DependencyAddRequest) -> DependencyAddResponse:
    """POST /api/tmo/dependencies — 添加依赖边 (M-N3 + DAGValidator)

    守门 #13 a 强约束: 检测到 cycle 立即 reject + interrupt (HTTP 409)
    """
    added_edges: List[List[str]] = []

    # 模式 1: 边列表
    if req.edges:
        for edge in req.edges:
            try:
                _GRAPH.add_edge(edge.task_id, edge.depends_on)
                added_edges.append([edge.task_id, edge.depends_on])
            except ValueError as exc:
                raise HTTPException(status_code=400, detail=str(exc))

    # 模式 2: 单节点 4 字段
    if req.task_id:
        if req.parent_task_id:
            _GRAPH.add_edge(req.task_id, req.parent_task_id)
            added_edges.append([req.task_id, req.parent_task_id])
        if req.merged_from:
            for dep in req.merged_from:
                _GRAPH.add_edge(req.task_id, dep)
                added_edges.append([req.task_id, dep])
        if req.split_into:
            for sub in req.split_into:
                _GRAPH.add_edge(sub, req.task_id)
                added_edges.append([sub, req.task_id])
        if req.superseded_by:
            _GRAPH.add_edge(req.superseded_by, req.task_id)
            added_edges.append([req.superseded_by, req.task_id])

    # cycle detection (守门 #13 a)
    validator = DAGValidator(_GRAPH)
    if validator.has_cycle():
        cycle_path = validator.find_cycle()
        # 回滚 (reject)
        for edge in added_edges:
            try:
                _GRAPH.remove_edge(edge[0], edge[1])
            except Exception:
                pass
        return DependencyAddResponse(
            ok=False,
            added_edges=[],
            total_edges=len(_GRAPH.edges()),
            total_nodes=len(_GRAPH.nodes()),
            cycle_detected=True,
            cycle_path=cycle_path,
        )

    return DependencyAddResponse(
        ok=True,
        added_edges=added_edges,
        total_edges=len(_GRAPH.edges()),
        total_nodes=len(_GRAPH.nodes()),
        cycle_detected=False,
    )


@router.get("/dependencies")
async def list_dependencies() -> dict:
    """GET /api/tmo/dependencies — 列出全部依赖边"""
    return {
        "ok": True,
        "edges": [[u, v] for u, v in _GRAPH.edges()],
        "total_edges": len(_GRAPH.edges()),
        "total_nodes": len(_GRAPH.nodes()),
    }


@router.get("/dependencies/{task_id}")
async def get_task_dependencies(task_id: str) -> dict:
    """GET /api/tmo/dependencies/{task_id} — 查 task 的依赖 + dependents"""
    return {
        "ok": True,
        "task_id": task_id,
        "dependencies": list(_GRAPH.dependencies(task_id)),
        "dependents": list(_GRAPH.dependents(task_id)),
    }


@router.delete("/dependencies")
async def clear_dependencies() -> dict:
    """DELETE /api/tmo/dependencies — 清空整图 (reset)"""
    global _GRAPH
    edges_before = len(_GRAPH.edges())
    _GRAPH = TaskRelationshipGraph()
    return {
        "ok": True,
        "cleared_edges": edges_before,
        "total_edges": 0,
        "total_nodes": 0,
    }


@router.post("/dependencies/validate")
async def validate_dependencies() -> dict:
    """POST /api/tmo/dependencies/validate — 仅校验 cycle, 不修改图"""
    validator = DAGValidator(_GRAPH)
    if validator.has_cycle():
        cycle_path = validator.find_cycle()
        return {
            "ok": False,
            "cycle_detected": True,
            "cycle_path": cycle_path,
        }
    return {
        "ok": True,
        "cycle_detected": False,
        "total_edges": len(_GRAPH.edges()),
        "total_nodes": len(_GRAPH.nodes()),
    }


class ReorderRequest(BaseModel):
    """POST /api/tmo/reorder 触发 M-N3 reorder_node"""
    task_ids: Optional[List[str]] = Field(None, description="task_ids (None=全图)")


@router.post("/reorder")
async def post_reorder(req: ReorderRequest) -> dict:
    """POST /api/tmo/reorder — 触发 M-N3 reorder_node, 返回 topological order"""
    try:
        result = _REORDER_NODE.execute(_GRAPH, task_ids=req.task_ids)
    except ReorderInterrupted as exc:
        raise HTTPException(
            status_code=409,
            detail={
                "error": "cycle_detected",
                "source_node": exc.source_node,
                "reason": exc.reason,
                "cycle_path": exc.cycle_path,
            },
        )
    return {
        "ok": True,
        "topological_order": result.order,
        "total_nodes": len(result.order),
    }


@router.get("/graph")
async def get_graph() -> dict:
    """GET /api/tmo/graph — 序列化整图 (调试 + 持久化)"""
    return {
        "ok": True,
        "graph": {
            "nodes": list(_GRAPH.nodes()),
            "edges": [[u, v] for u, v in _GRAPH.edges()],
        },
        "total_nodes": len(_GRAPH.nodes()),
        "total_edges": len(_GRAPH.edges()),
    }


# ===========================================================================
# TMO-04: /api/tmo/bulk + /api/tmo/bulk/health (wt-tmo-04-bulk 实装)
# ===========================================================================

from scripts.automation.task_ops.bulk_queue import (  # noqa: E402
    VALID_ACTIONS,
    BulkAction,
    BulkOperationQueue,
    mock_card_action,
    REVERSE_ACTION_MAP,
)

VALID_ACTIONS_LIST = sorted(VALID_ACTIONS)
_bulk_queue: BulkOperationQueue = BulkOperationQueue(card_action_fn=mock_card_action)


class BulkRequest(BaseModel):
    """POST /api/tmo/bulk 请求体 (per L0→L1 bulk_action 协议)"""
    target_task_ids: list = Field(min_length=1, description="1+ 张 task card id")
    action: str = Field(description="bulk action type (4 类)")
    action_params: dict = Field(default_factory=dict, description="action 附加参数")

    @field_validator("action")
    @classmethod
    def _validate_action(cls, v: str) -> str:
        if v not in VALID_ACTIONS:
            raise ValueError(
                f"action must be one of {VALID_ACTIONS_LIST}, got {v!r}"
            )
        return v

    @field_validator("action_params")
    @classmethod
    def _validate_set_priority(cls, v: dict, info) -> dict:
        action = info.data.get("action")
        if action == "set_priority" and "priority" not in v:
            raise ValueError("action='set_priority' requires 'priority' in action_params")
        return v


class BulkResponse(BaseModel):
    """POST /api/tmo/bulk 响应体 (per 02 §4 M-N4 batch_summary)"""
    operation: str = "bulk"
    action: str
    batch_id: str
    success_count: int
    failed_count: int
    failed_ids: list
    rolled_back_ids: list
    rollback_failed_ids: list
    outcome: str
    failure_rate: float
    started_at: str
    duration_ms: float
    reverse_action: Optional[str]
    total: int


@router.post("/bulk", response_model=BulkResponse)
async def post_bulk(req: BulkRequest) -> dict:
    """POST /api/tmo/bulk — TMO M-N4 批量 action (per 02 §2.6.4)

    守门 NFR-TMO-03: 部分失败 rollback (失败 > 20% 时全部 rollback)
    """
    try:
        bulk_action = BulkAction(
            target_task_ids=list(req.target_task_ids),
            action=req.action,
            action_params=dict(req.action_params),
        )
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))

    _bulk_queue.enqueue(bulk_action)
    results = await _bulk_queue.flush()
    if not results:
        raise HTTPException(status_code=500, detail="flush returned no results")
    r = results[0]
    return BulkResponse(
        operation="bulk",
        action=r.action,
        batch_id=r.batch_id,
        success_count=r.success_count,
        failed_count=r.failed_count,
        failed_ids=r.failed_ids,
        rolled_back_ids=r.rolled_back_ids,
        rollback_failed_ids=r.rollback_failed_ids,
        outcome=r.outcome,
        failure_rate=r.failure_rate,
        started_at=r.started_at,
        duration_ms=r.duration_ms,
        reverse_action=REVERSE_ACTION_MAP.get(r.action),
        total=r.total,
    )


@router.get("/bulk/health")
async def get_bulk_health() -> dict:
    """GET /api/tmo/bulk/health — 路由器状态 + queue stats"""
    return {
        "status": "ok",
        "ts": datetime.now(timezone.utc).isoformat(),
        "queue_stats": _bulk_queue.stats,
        "valid_actions": VALID_ACTIONS_LIST,
    }


# ===========================================================================
# TMO-09: /api/tmo/relationships (DAG 关系查询, 跟 dependencies 配套)
# ===========================================================================

@router.get("/relationships")
async def get_relationships() -> dict:
    """GET /api/tmo/relationships — DAG 边查询 (per 02 §5.2 GET 端点)"""
    return {
        "ok": True,
        "relationships": [[u, v] for u, v in _GRAPH.edges()],
        "total": len(_GRAPH.edges()),
    }


# ===========================================================================
# TMO-07: /api/tmo/metadata (M-N7 + TaskMetadataRepository 持久化)
# per G-TMO-04c, 守门 #13 c Master RLS + 守门 #13 d SCD Type 2 + 守门 #DB-13
# ===========================================================================

import os as _os  # noqa: E402

_TASK_METADATA_DB_PATH: str = _os.environ.get(
    "STAR_TASK_METADATA_DB_PATH",
    str(_PROJECT_ROOT / "data" / "task_metadata.sqlite"),
)
_TASK_METADATA_DB_PATH = str(Path(_TASK_METADATA_DB_PATH).resolve())
Path(_TASK_METADATA_DB_PATH).parent.mkdir(parents=True, exist_ok=True)

# 初始化 schema (per G-TMO-04 DDL)
from automation.task_ops.task_metadata_ddl import init_schema as _init_task_metadata_schema  # noqa: E402
_init_task_metadata_schema(_TASK_METADATA_DB_PATH)

from automation.task_ops.task_metadata_repo import TaskMetadataRepository as _TaskMetadataRepository  # noqa: E402
_task_metadata_repo: _TaskMetadataRepository = _TaskMetadataRepository(_TASK_METADATA_DB_PATH)


class MetadataUpsertRequest(BaseModel):
    """POST /api/tmo/metadata 请求体 (per M-N7 MetadataUpdate + G-TMO-04b repo)"""
    task_id: str = Field(min_length=1, description="task card id")
    tenant_id: str = Field(min_length=1, description="Master RLS 必携 (守门 #13 c)")
    workspace_id: str = Field(min_length=1, description="Master RLS 必携 (守门 #13 c)")
    metadata: dict = Field(description="metadata 字段: {name?, labels?, notes?, priority?}")
    actor_session_id: Optional[str] = Field(default=None, description="L0 session id (audit 用)")


class MetadataUpsertResponse(BaseModel):
    """POST /api/tmo/metadata 响应"""
    ok: bool
    operation: str  # "upsert"
    task_id: str
    version: int
    is_current: bool
    name: Optional[str]
    labels: List[str]
    notes: Optional[str]
    priority: int
    updated_at_ms: int


class MetadataGetResponse(BaseModel):
    """GET /api/tmo/metadata/{task_id} 响应"""
    ok: bool
    task_id: str
    version: int
    name: Optional[str]
    labels: List[str]
    notes: Optional[str]
    priority: int
    created_at_ms: int
    updated_at_ms: int


class MetadataHistoryResponse(BaseModel):
    """GET /api/tmo/metadata/{task_id}/history 响应 (SCD Type 2 历史)"""
    ok: bool
    task_id: str
    history: List[dict]


class MetadataAuditResponse(BaseModel):
    """GET /api/tmo/metadata/{task_id}/audit 响应 (audit log)"""
    ok: bool
    task_id: str
    audit_events: List[dict]


@router.post("/metadata", response_model=MetadataUpsertResponse)
async def post_metadata_upsert(req: MetadataUpsertRequest) -> MetadataUpsertResponse:
    """POST /api/tmo/metadata — M-N7 metadata_node 持久化 (per G-TMO-04b TaskMetadataRepository)

    流程:
      1. 委托 _task_metadata_repo.upsert_metadata (走 SCD Type 2 + audit 5 类事件)
      2. 异常 → 400 (R-12 = RLS violation, 403 forbidden; 其他 = 500)

    守门:
      - 守门 #13 a: L0 唯一入口, 跨 task metadata 操作只经 L0
      - 守门 #13 c: tenant_id / workspace_id 必携 (Pydantic min_length=1 校验)
      - 守门 #13 d: SCD Type 2 (旧 version is_current=0 + 新 version is_current=1 + scd snapshot)
      - 守门 #19: Python 化, 不写 .rs
    """
    try:
        record = _task_metadata_repo.upsert_metadata(
            task_id=req.task_id,
            tenant_id=req.tenant_id,
            workspace_id=req.workspace_id,
            metadata=req.metadata,
            actor_session_id=req.actor_session_id,
        )
    except PermissionError as exc:
        # 守门 #13 c Master 物理删除禁止 (虽然 upsert 不会触发, 防御性)
        raise HTTPException(status_code=403, detail=str(exc))
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))

    return MetadataUpsertResponse(
        ok=True,
        operation="upsert",
        task_id=record.task_id,
        version=record.version,
        is_current=record.is_current,
        name=record.name,
        labels=record.labels,
        notes=record.notes,
        priority=record.priority,
        updated_at_ms=record.updated_at_ms,
    )


@router.get("/metadata/_health")
async def get_metadata_health() -> dict:
    """GET /api/tmo/metadata/_health — repo 状态 (per 守门 #1)

    注册顺序优先于 /metadata/{task_id}, 避免 _health 被解析成 task_id.
    """
    return {
        "ok": True,
        "db_path": _TASK_METADATA_DB_PATH,
        "ts": datetime.now(timezone.utc).isoformat(),
    }


@router.get("/metadata/{task_id}", response_model=MetadataGetResponse)
async def get_metadata_current(
    task_id: str,
    tenant_id: str = Query(..., min_length=1, description="Master RLS 必携 (守门 #13 c)"),
    workspace_id: str = Query(..., min_length=1, description="Master RLS 必携 (守门 #13 c)"),
) -> MetadataGetResponse:
    """GET /api/tmo/metadata/{task_id} — 读 task 当前 metadata (is_current=1)

    守门 #13 c: tenant_id + workspace_id 强制 query param 必携 (RLS 隔离)
    """
    record = _task_metadata_repo.get_current_metadata(
        task_id=task_id,
        tenant_id=tenant_id,
        workspace_id=workspace_id,
    )
    if record is None:
        raise HTTPException(
            status_code=404,
            detail=f"task_metadata not found: task_id={task_id} tenant_id={tenant_id} workspace_id={workspace_id}",
        )
    return MetadataGetResponse(
        ok=True,
        task_id=record.task_id,
        version=record.version,
        name=record.name,
        labels=record.labels,
        notes=record.notes,
        priority=record.priority,
        created_at_ms=record.created_at_ms,
        updated_at_ms=record.updated_at_ms,
    )


@router.get("/metadata/{task_id}/history", response_model=MetadataHistoryResponse)
async def get_metadata_history(
    task_id: str,
    tenant_id: str = Query(..., min_length=1, description="Master RLS 必携 (守门 #13 c)"),
    workspace_id: str = Query(..., min_length=1, description="Master RLS 必携 (守门 #13 c)"),
    limit: int = Query(50, ge=1, le=500, description="history limit"),
) -> MetadataHistoryResponse:
    """GET /api/tmo/metadata/{task_id}/history — SCD Type 2 历史 (version DESC)"""
    history = _task_metadata_repo.get_scd_history(
        task_id=task_id,
        tenant_id=tenant_id,
        workspace_id=workspace_id,
        limit=limit,
    )
    return MetadataHistoryResponse(
        ok=True,
        task_id=task_id,
        history=history,
    )


@router.get("/metadata/{task_id}/audit", response_model=MetadataAuditResponse)
async def get_metadata_audit(
    task_id: str,
    tenant_id: str = Query(..., min_length=1, description="Master RLS 必携 (守门 #13 c)"),
    workspace_id: str = Query(..., min_length=1, description="Master RLS 必携 (守门 #13 c)"),
    limit: int = Query(50, ge=1, le=500, description="audit limit"),
) -> MetadataAuditResponse:
    """GET /api/tmo/metadata/{task_id}/audit — audit log (event_at_ms DESC)"""
    audit_events = _task_metadata_repo.get_audit_log(
        task_id=task_id,
        tenant_id=tenant_id,
        workspace_id=workspace_id,
        limit=limit,
    )
    return MetadataAuditResponse(
        ok=True,
        task_id=task_id,
        audit_events=audit_events,
    )


