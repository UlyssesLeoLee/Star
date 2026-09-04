#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/api/routes_tmo.py — TMO FastAPI Router (M-N3 reorder 端点)
(per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.6.4 8 端点 / 03-detailed-design.md §3.2.1.1)

端点 (本子代理 wt-tmo-03 落档):
  - POST /api/tmo/dependencies         — 添加依赖边 (a 依赖 b) 入图
  - GET  /api/tmo/dependencies         — 列出全部依赖边
  - GET  /api/tmo/dependencies/{task_id} — 查 task 的依赖 + dependents
  - DELETE /api/tmo/dependencies       — 清空整图 (reset)
  - POST /api/tmo/dependencies/validate — 仅校验 cycle, 不修改图
  - POST /api/tmo/reorder              — 触发 M-N3 reorder_node (返回 topological order)
  - GET  /api/tmo/graph                — 序列化整图 (调试 + 持久化)

挂载 (per 守门 #24 v3):
    from scripts.automation.api.routes_tmo import router as tmo_router
    app.include_router(tmo_router)

约束 (per 守门 #1 v1 + 守门 #13 a + 守门 #19 + 守门 #22 + 守门 #23):
  - L0 协调, in-memory graph (重启 reset, 跨 session 续可加持久化)
  - cycle detection O(V+E) 走 DAGValidator
  - interrupt 协议 (ReorderInterrupted) → HTTP 409 Conflict
  - 不开 OpenAI / Anthropic API
  - audit log 落 docs/reports/tmo.log
"""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Dict, List, Optional

try:
    from fastapi import APIRouter, HTTPException
    from pydantic import BaseModel, Field
except ImportError:
    raise ImportError(
        "FastAPI + pydantic not installed. pip install fastapi pydantic"
    )

# 局部 import (跨 worktree 兼容: reorder_node.py 是本子代理实装, relationship_graph / dag_validator 同理)
from scripts.automation.task_ops.relationship_graph import TaskRelationshipGraph
from scripts.automation.task_ops.dag_validator import DAGValidator
from scripts.automation.task_ops.nodes.reorder_node import (
    ReorderNode,
    ReorderState,
    ReorderResult,
    ReorderInterrupted,
)


# === Router ===

router = APIRouter(prefix="/api/tmo", tags=["tmo"])

# 全局 in-memory graph (per session, 重启 reset, 跨 session 续可加持久化)
_GRAPH: TaskRelationshipGraph = TaskRelationshipGraph()
_REORDER_NODE: ReorderNode = ReorderNode()


# === Pydantic Schemas (请求 / 响应) ===


class DependencyEdge(BaseModel):
    """单条依赖边: a 依赖 b (a → b)."""

    task_id: str = Field(..., description="依赖方 (a)", min_length=1)
    depends_on: str = Field(..., description="被依赖方 (b, a 依赖 b)", min_length=1)


class DependencyAddRequest(BaseModel):
    """POST /api/tmo/dependencies 批量添加依赖边."""

    edges: List[DependencyEdge] = Field(..., description="依赖边列表")
    # 也支持一次只声明 task + 4 字段 (per TaskRelationshipGraph 语义)
    task_id: Optional[str] = Field(None, description="单节点 task_id")
    parent_task_id: Optional[str] = Field(None)
    merged_from: Optional[List[str]] = Field(None)
    split_into: Optional[List[str]] = Field(None)
    superseded_by: Optional[List[str]] = Field(None)


class DependencyAddResponse(BaseModel):
    ok: bool
    added_edges: List[List[str]]  # [[a, b], ...]
    total_edges: int
    total_nodes: int
    cycle_detected: bool
    cycle_path: Optional[List[str]] = None


class DependencyListResponse(BaseModel):
    edges: List[List[str]]
    nodes: List[str]
    total_edges: int
    total_nodes: int
    is_dag: bool


class DependencyQueryResponse(BaseModel):
    task_id: str
    dependencies: List[str]
    dependents: List[str]
    is_dag: bool
    cycle_path: Optional[List[str]] = None


class ValidateRequest(BaseModel):
    """POST /api/tmo/dependencies/validate — 仅校验, 不修改图."""

    edges: List[DependencyEdge]


class ValidateResponse(BaseModel):
    is_dag: bool
    cycle_path: Optional[List[str]] = None
    proposed_edges: List[List[str]]


class ReorderRequest(BaseModel):
    """POST /api/tmo/reorder — 触发 M-N3 reorder_node."""

    task_ids: List[str] = Field(..., description="参与重排的任务 ID 列表")
    dep_set: Dict[str, List[str]] = Field(
        default_factory=dict,
        description="task_id → [depends_on_task_ids]",
    )
    session_id: Optional[str] = None


class ReorderResponse(BaseModel):
    ok: bool
    order: List[str] = []
    cycle_path: Optional[List[str]] = None
    reason: Optional[str] = None
    duration_ms: float = 0.0


class GraphResponse(BaseModel):
    graph: Dict
    is_dag: bool
    cycle_path: Optional[List[str]] = None
    total_nodes: int
    total_edges: int


class SimpleOk(BaseModel):
    ok: bool
    message: str = ""


# === Helper: audit log (per §3.4) ===

def _audit(action: str, payload: Dict, error: Optional[str] = None) -> None:
    try:
        log_path = Path("docs/reports/tmo.log")
        log_path.parent.mkdir(parents=True, exist_ok=True)
        entry = {
            "timestamp": time.time(),
            "phase": "task_ops.api.routes_tmo",
            "action": action,
            "payload": payload,
            "error": error,
        }
        with log_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except Exception:
        import sys
        print(f"[WARN] tmo.log write failed for action={action}", file=sys.stderr)


# === 端点 ===

@router.post("/dependencies", response_model=DependencyAddResponse)
def add_dependencies(req: DependencyAddRequest) -> DependencyAddResponse:
    """批量添加依赖边.

    流程:
      1. 先在临时图上 apply 全部 edges + 单节点 4 字段
      2. 跑 DAGValidator 检测 cycle
      3. 有环 → 拒绝 (HTTP 200 + ok=False + cycle_path), 不写入主图
      4. 无环 → 写入主图 + 返 ok=True + 总览
    """
    global _GRAPH

    # 1. 临时图
    tmp = TaskRelationshipGraph()
    for node_id in _GRAPH.nodes():
        existing = _GRAPH.get_node(node_id)
        if existing:
            tmp.add_task(
                task_id=existing.task_id,
                parent_task_id=existing.parent_task_id,
                merged_from=list(existing.merged_from),
                split_into=list(existing.split_into),
                superseded_by=list(existing.superseded_by),
                status=existing.status,
                metadata=dict(existing.metadata),
            )
    for u, v in _GRAPH.edges():
        tmp.add_edge(u, v)

    added: List[List[str]] = []

    # 2. apply 显式边
    for edge in req.edges:
        tmp.add_edge(edge.task_id, edge.depends_on)
        added.append([edge.task_id, edge.depends_on])

    # 3. apply 单节点 4 字段
    if req.task_id is not None:
        tmp.add_task(
            task_id=req.task_id,
            parent_task_id=req.parent_task_id,
            merged_from=req.merged_from,
            split_into=req.split_into,
            superseded_by=req.superseded_by,
        )

    # 4. cycle check
    cycle = DAGValidator.find_cycle(tmp)
    if cycle is not None:
        _audit(
            "add_dependencies.cycle_rejected",
            {
                "added_attempted": added,
                "cycle_path": cycle,
            },
        )
        raise HTTPException(
            status_code=409,
            detail={
                "error": "cycle_detected",
                "cycle_path": cycle,
                "message": f"adding these edges creates a cycle: {' -> '.join(cycle)}",
            },
        )

    # 5. 写入主图
    _GRAPH = tmp

    _audit(
        "add_dependencies.success",
        {
            "added": added,
            "total_edges": _GRAPH.edge_count(),
            "total_nodes": len(_GRAPH.nodes()),
        },
    )

    return DependencyAddResponse(
        ok=True,
        added_edges=added,
        total_edges=_GRAPH.edge_count(),
        total_nodes=len(_GRAPH.nodes()),
        cycle_detected=False,
        cycle_path=None,
    )


@router.get("/dependencies", response_model=DependencyListResponse)
def list_dependencies() -> DependencyListResponse:
    """列出全部依赖边 + 节点."""
    edges = [list(e) for e in _GRAPH.edges()]
    is_dag, cycle = DAGValidator.validate(_GRAPH)
    return DependencyListResponse(
        edges=edges,
        nodes=_GRAPH.nodes(),
        total_edges=len(edges),
        total_nodes=len(_GRAPH.nodes()),
        is_dag=is_dag,
    )


@router.get("/dependencies/{task_id}", response_model=DependencyQueryResponse)
def get_task_dependencies(task_id: str) -> DependencyQueryResponse:
    """查 task 的依赖 (dependencies) + 反向 (dependents)."""
    if not _GRAPH.has_node(task_id):
        raise HTTPException(status_code=404, detail=f"task_id not found: {task_id}")
    is_dag, cycle = DAGValidator.validate(_GRAPH)
    return DependencyQueryResponse(
        task_id=task_id,
        dependencies=_GRAPH.dependencies(task_id),
        dependents=_GRAPH.dependents(task_id),
        is_dag=is_dag,
        cycle_path=cycle,
    )


@router.delete("/dependencies", response_model=SimpleOk)
def clear_dependencies() -> SimpleOk:
    """清空整图 (reset)."""
    global _GRAPH
    _GRAPH = TaskRelationshipGraph()
    _audit("clear_dependencies.success", {})
    return SimpleOk(ok=True, message="graph reset")


@router.post("/dependencies/validate", response_model=ValidateResponse)
def validate_proposed(req: ValidateRequest) -> ValidateResponse:
    """仅校验 cycle, 不写入主图."""
    tmp = TaskRelationshipGraph()
    for node_id in _GRAPH.nodes():
        existing = _GRAPH.get_node(node_id)
        if existing:
            tmp.add_task(task_id=existing.task_id)
    for u, v in _GRAPH.edges():
        tmp.add_edge(u, v)
    proposed: List[List[str]] = []
    for edge in req.edges:
        tmp.add_edge(edge.task_id, edge.depends_on)
        proposed.append([edge.task_id, edge.depends_on])
    is_dag, cycle = DAGValidator.validate(tmp)
    return ValidateResponse(
        is_dag=is_dag,
        cycle_path=cycle,
        proposed_edges=proposed,
    )


@router.post("/reorder", response_model=ReorderResponse)
def reorder(req: ReorderRequest) -> ReorderResponse:
    """触发 M-N3 reorder_node.

    成功: 返 200 + ReorderResponse (ok=True, order=...)
    cycle: 返 409 + detail 含 cycle_path (ReorderInterrupted interrupt 协议落地)
    """
    state = ReorderState(
        task_ids=req.task_ids,
        dep_set=req.dep_set,
        existing_graph=None,  # 用独立 graph (不污染主图)
        session_id=req.session_id,
    )
    try:
        result = _REORDER_NODE.execute(state)
        return ReorderResponse(
            ok=result.ok,
            order=result.order,
            cycle_path=None,
            reason=None,
            duration_ms=result.duration_ms,
        )
    except ReorderInterrupted as intr:
        _audit(
            "reorder.cycle_rejected",
            {
                "task_ids": req.task_ids,
                "dep_set": req.dep_set,
                "cycle_path": intr.cycle_path,
            },
        )
        raise HTTPException(
            status_code=409,
            detail=intr.to_dict(),
        )


@router.get("/graph", response_model=GraphResponse)
def get_graph() -> GraphResponse:
    """序列化整图 (调试 + 持久化)."""
    is_dag, cycle = DAGValidator.validate(_GRAPH)
    return GraphResponse(
        graph=_GRAPH.to_dict(),
        is_dag=is_dag,
        cycle_path=cycle,
        total_nodes=len(_GRAPH.nodes()),
        total_edges=_GRAPH.edge_count(),
    )


# === CLI 入口 (调试用, 不被 console_server 调) ===

if __name__ == "__main__":
    import sys

    print("scripts/automation/api/routes_tmo.py — FastAPI router")
    print("挂载到 console_server.py:")
    print("  from scripts.automation.api.routes_tmo import router as tmo_router")
    print("  app.include_router(tmo_router)")
    print("端点: POST/GET/DELETE /api/tmo/dependencies, POST /api/tmo/reorder, ...")
    sys.exit(0)


__all__ = ["router"]
