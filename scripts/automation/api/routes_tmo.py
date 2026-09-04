# scripts/automation/api/routes_tmo.py
# TMO FastAPI routes (per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.5)
#
# 本子项 TMO-01 实装:
#   - POST /api/tmo/merge         合并任务卡 (M-N1 + SA-10)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一入口, 跨 L1 task 操作只经 L0
#   - 守门 #19: Python 化
#   - 守门 #22: routes mount 在 console_server.py (port 8080), 不污染 main 编译链
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 → subprocess
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API

from __future__ import annotations

import logging
from typing import Optional

try:
    from fastapi import APIRouter, HTTPException
    from pydantic import BaseModel, Field
except ImportError:
    raise ImportError("fastapi + pydantic not installed. pip install fastapi pydantic")

logger = logging.getLogger("api.routes_tmo")

router = APIRouter(prefix="/api/tmo", tags=["tmo"])


# ===== Request / Response models =====

class MergeRequestBody(BaseModel):
    """POST /api/tmo/merge 请求体 (per task_ops/protocols.py MergeRequest)"""
    target_task_ids: list[str] = Field(..., min_length=2, description="≥ 2 task_ids to merge")
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
    superseded_task_ids: list[str]
    merge_strategy: str
    stash_checkpoint_ids: list[str]


class MergeResponse(BaseModel):
    ok: bool
    node: str  # "M-N1"
    merged_task_id: Optional[str] = None
    result: Optional[MergeResult] = None
    ui_events: Optional[list[UIEvent]] = None
    error: Optional[str] = None
    duration_ms: float


# ===== 端点 1: POST /api/tmo/merge =====

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

    # mock 模式: 在 sub_pool 里加 2 个 L1 task (a, b)
    for tid in req.target_task_ids:
        try:
            manager.sub_pool.get(tid)
        except KeyError:
            manager.sub_pool.add(task_type="SA-09", task_id=tid, initial_state={
                "status": "running",
                "context": {"description": f"task {tid} mock"},
            })

    # 调用 TaskOperationsManager.dispatch
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


# ===== 端点 2: GET /api/tmo/operations (TMO 状态查询, TMO-08 planned) =====

@router.get("/operations")
async def tmo_operations() -> dict:
    """TMO 状态查询 (per 02 §2.6 状态查询 端点, TMO-08 planned)

    本子项 TMO-01 简单 stub: 返回 TaskOperationsManager 状态
    """
    from automation.task_ops.manager import TaskOperationsManager
    # 真实接入时, 这里会从全局 registry 拿 manager (本子项 mock 模式直接 new)
    manager = TaskOperationsManager()
    return {
        "ok": True,
        "snapshot": manager.get_state_snapshot(),
        "implemented_nodes": ["M-N1"],
        "planned_nodes": ["M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7"],
    }
