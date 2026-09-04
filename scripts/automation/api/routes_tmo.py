"""scripts/automation/api/routes_tmo.py — TMO /api/tmo/* 路由 (TMO-04 partial)

TMO v0.2 8 端点 (per docs/architecture/2026-09-03-langgraph/02-basic-design.md §4):
    /api/tmo/merge         POST   (TMO-01 owner)
    /api/tmo/split         POST   (TMO-02 owner)
    /api/tmo/dependencies  POST   (TMO-03 owner)
    /api/tmo/bulk          POST   (TMO-04 owner, 本 worktree)
    /api/tmo/summarize     POST   (TMO-05 owner)
    /api/tmo/reassign      POST   (TMO-06 owner)
    /api/tmo/metadata      POST   (TMO-07 owner)
    /api/tmo/relationships GET    (TMO-07 owner)

本文件仅含 /api/tmo/bulk (TMO-04 owner, wt-tmo-04); 其他端点由兄弟 worktree 各自
mount 进来 (per 守门分工 + G-TMO-07 namespace 隔离)。

约束 (per 守门 #22 + 守门 #24):
  - 子包独立 Python 进程 (port 8080), 不进主仓 cargo 编译链
  - 浏览器 → Next.js API → FastAPI → subprocess (per 守门 #9 v3 + #24)
  - card_action 走 mock_card_action (per 守门 #23 AI mock 模式)
  - audit log 落 docs/reports/tmo-api-bulk.log

用法:
    from automation.api.routes_tmo import router, create_bulk_router
    app.include_router(router)  # bulk only
    # 或
    bulk_app = create_bulk_router(card_action_fn=my_real_card_action)
"""

from __future__ import annotations

import json
import logging
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

try:
    from fastapi import APIRouter, HTTPException
    from pydantic import BaseModel, Field, field_validator
except ImportError:
    print("ERROR: fastapi + pydantic not installed. pip install fastapi pydantic", file=sys.stderr)
    raise

# 让 `python scripts/automation/api/routes_tmo.py` 跑 self-test 时能找到 bulk_queue
# (per 守门 #19 Python 化, 跨 stage entry point 走 python -m)
_PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent.parent
if str(_PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(_PROJECT_ROOT))

from scripts.automation.task_ops.bulk_queue import (  # noqa: E402
    VALID_ACTIONS,
    BulkAction,
    BulkOperationQueue,
    mock_card_action,
)

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Request / Response schemas (per 02 §4 API endpoint 表)
# ---------------------------------------------------------------------------


VALID_ACTIONS_LIST = sorted(VALID_ACTIONS)


class BulkRequest(BaseModel):
    """POST /api/tmo/bulk 请求体

    Fields (per L0→L1 `bulk_action` 协议):
        - target_task_ids: list[str] 必填, 1+ 张卡
        - action: "pause" | "resume" | "cancel" | "set_priority"
        - action_params: dict 可选 (set_priority 需 priority 字段)
    """

    target_task_ids: list = Field(min_length=1, description="1+ 张 task card id")
    action: str = Field(description="bulk action type (4 类)")
    action_params: dict = Field(default_factory=dict, description="action 额外参数")

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


# ---------------------------------------------------------------------------
# Router factory
# ---------------------------------------------------------------------------


def create_bulk_router(
    card_action_fn=None,
    audit_log: Optional[Path] = None,
) -> APIRouter:
    """工厂: 返 APIRouter (含 /api/tmo/bulk POST 端点)

    Args:
        card_action_fn: 注入的 card_action 实现, None = mock_card_action
        audit_log: audit 日志路径, None = 不写 audit
    """
    _queue = BulkOperationQueue(card_action_fn=card_action_fn, audit_log=audit_log)

    router = APIRouter(prefix="/api/tmo", tags=["tmo-bulk"])

    @router.post("/bulk", response_model=BulkResponse)
    async def post_bulk(req: BulkRequest) -> dict:
        """POST /api/tmo/bulk — TMO M-N4 批量 action (per 02 §2.6.4)"""
        try:
            bulk_action = BulkAction(
                target_task_ids=list(req.target_task_ids),
                action=req.action,
                action_params=dict(req.action_params),
            )
        except ValueError as exc:
            raise HTTPException(status_code=400, detail=str(exc))

        _queue.enqueue(bulk_action)
        results = await _queue.flush()
        if not results:
            # 队列空 (理论不会发生, 因为刚刚 enqueue)
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
            reverse_action=VALID_ACTIONS.__contains__(r.action) and _get_reverse(r.action) or None,
            total=r.total,
        )

    @router.get("/bulk/health")
    async def get_bulk_health() -> dict:
        """GET /api/tmo/bulk/health — 健康检查 + queue stats"""
        return {
            "status": "ok",
            "ts": datetime.now(timezone.utc).isoformat(),
            "queue_stats": _queue.stats,
            "valid_actions": VALID_ACTIONS_LIST,
        }

    return router


def _get_reverse(action: str) -> Optional[str]:
    from scripts.automation.task_ops.bulk_queue import REVERSE_ACTION_MAP
    return REVERSE_ACTION_MAP.get(action)


# ---------------------------------------------------------------------------
# Default router (mock card_action, per 守门 #23 + standalone demo)
# ---------------------------------------------------------------------------


router = create_bulk_router(card_action_fn=mock_card_action)


# ---------------------------------------------------------------------------
# Self-test (FastAPI TestClient 走端点 + 4 类 case 实证)
# ---------------------------------------------------------------------------


def _self_test() -> None:
    """Self-test: 4 类 partial failure 走 HTTP 端点"""
    try:
        from fastapi.testclient import TestClient
    except ImportError:
        print("ERROR: fastapi.testclient not available, skipping self-test", file=sys.stderr)
        return
    from fastapi import FastAPI
    from scripts.automation.task_ops.bulk_queue import LOG_DIR_DEFAULT, LOG_FILE_DEFAULT

    log_path = LOG_DIR_DEFAULT / "tmo-api-bulk.log"
    print(f"audit log: {log_path}")

    # 用一个可控制 fail 的 card_action_fn
    def selective_card_action(task_id, action, action_params=None):
        async def _inner() -> bool:
            if task_id.startswith("fail-"):
                raise RuntimeError(f"simulated failure for {task_id!r}")
            return True
        return _inner()

    app = FastAPI(title="TMO-04 bulk API self-test")
    app.include_router(create_bulk_router(card_action_fn=selective_card_action, audit_log=log_path))
    client = TestClient(app)

    # Case 1: 0 失败 → success
    r1 = client.post(
        "/api/tmo/bulk",
        json={"target_task_ids": ["t1", "t2", "t3"], "action": "pause", "action_params": {}},
    )
    print(f"[1] 0 fail       : {r1.status_code} outcome={r1.json().get('outcome')}")
    assert r1.status_code == 200
    assert r1.json()["outcome"] == "success"

    # Case 2: 1/5 失败 (20%) → partial
    r2 = client.post(
        "/api/tmo/bulk",
        json={
            "target_task_ids": ["ok-1", "ok-2", "ok-3", "ok-4", "fail-1"],
            "action": "pause",
            "action_params": {},
        },
    )
    print(f"[2] 1/5 fail=20% : {r2.status_code} outcome={r2.json().get('outcome')}")
    assert r2.status_code == 200
    assert r2.json()["outcome"] == "partial"

    # Case 3: 3/5 失败 (60%) → rolled_back
    r3 = client.post(
        "/api/tmo/bulk",
        json={
            "target_task_ids": ["ok-1", "ok-2", "fail-1", "fail-2", "fail-3"],
            "action": "pause",
            "action_params": {},
        },
    )
    print(f"[3] 3/5 fail=60% : {r3.status_code} outcome={r3.json().get('outcome')}")
    assert r3.status_code == 200
    assert r3.json()["outcome"] == "rolled_back"
    assert set(r3.json()["rolled_back_ids"]) == {"ok-1", "ok-2"}

    # Case 4: cancel + 2/4 失败 → rolled_back (cancel 不可逆)
    r4 = client.post(
        "/api/tmo/bulk",
        json={
            "target_task_ids": ["ok-1", "ok-2", "fail-1", "fail-2"],
            "action": "cancel",
            "action_params": {},
        },
    )
    print(f"[4] cancel 2/4   : {r4.status_code} outcome={r4.json().get('outcome')}")
    assert r4.status_code == 200
    assert r4.json()["outcome"] == "rolled_back"
    assert r4.json()["rolled_back_ids"] == []

    # Case 5: invalid action → 422 (pydantic validation error)
    r5 = client.post(
        "/api/tmo/bulk",
        json={"target_task_ids": ["t1"], "action": "invalid_xyz", "action_params": {}},
    )
    print(f"[5] invalid      : {r5.status_code} detail={str(r5.json().get('detail', ''))[:50]}")
    assert r5.status_code == 422  # pydantic validation error

    # Case 6: set_priority OK
    r6 = client.post(
        "/api/tmo/bulk",
        json={
            "target_task_ids": ["t1", "t2"],
            "action": "set_priority",
            "action_params": {"priority": 5},
        },
    )
    print(f"[6] set_priority : {r6.status_code} outcome={r6.json().get('outcome')}")
    assert r6.status_code == 200
    assert r6.json()["outcome"] == "success"

    # Case 7: health
    r7 = client.get("/api/tmo/bulk/health")
    print(f"[7] health       : {r7.status_code} stats={r7.json().get('queue_stats', {}).get('total_flushes')}")
    assert r7.status_code == 200
    assert r7.json()["status"] == "ok"

    print("\nALL HTTP SELF-TEST PASSED")


if __name__ == "__main__":  # pragma: no cover
    _self_test()
