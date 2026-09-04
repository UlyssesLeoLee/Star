"""tests/integration/test_tmo_bulk_dag.py — IT-12 TMO bulk + cycle prevention (partial)

Per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §8.3:
    IT-12 | TMO bulk + cycle prevention | bulk_node + DAGValidator cycle detection
    (per UC-11/UC-12) | tests/integration/test_tmo_bulk_dag.py

本 worktree (wt-tmo-04) 仅 IT-12 bulk 部分:
  - bulk HTTP 端点 end-to-end (FastAPI TestClient)
  - 4 类 partial failure 实证
  - audit log 落档
  - 跟 console_server.py 集成 (mock 模式, per 守门 #22)

cycle prevention 部分 (DAGValidator, TMO-03) 归 wt-tmo-03 owner, 本文件不重复。

约束:
  - pytest_asyncio
  - 跨 worktree 集成靠 namespace 隔离 (per G-TMO-07: /api/tmo/* vs /api/top-agent/*)
  - mock card_action (per 守门 #23)
"""

from __future__ import annotations

import asyncio
import json
import sys
from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

# 让 tests/ 能 import scripts/automation
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.automation.api.routes_tmo import create_bulk_router  # noqa: E402
from scripts.automation.task_ops.bulk_queue import (  # noqa: E402
    BulkAction,
    BulkOperationQueue,
    LOG_DIR_DEFAULT,
    REVERSE_ACTION_MAP,
    VALID_ACTIONS,
)
from scripts.automation.task_ops.nodes.bulk_node import make_bulk_node  # noqa: E402


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


def make_selective_card_action(fail_ids: set):
    async def selective_card_action(task_id: str, action: str, action_params=None) -> bool:
        if task_id in fail_ids:
            raise RuntimeError(f"simulated failure for {task_id!r}")
        return True
    return selective_card_action


@pytest.fixture
def audit_log_path(tmp_path) -> Path:
    """每个测试一个临时 audit log 路径"""
    return tmp_path / "tmo-bulk-it12.log"


@pytest.fixture
def selective_fail_ids() -> set:
    return set()


# ---------------------------------------------------------------------------
# IT-12 partial: bulk HTTP 端点 + 4 类 partial failure
# ---------------------------------------------------------------------------


class TestBulkHttpEndpoint:
    """IT-12 partial: 验证 /api/tmo/bulk HTTP 端点 4 类 case"""

    def _client(self, fail_ids: set, audit_log: Path) -> TestClient:
        app = FastAPI(title="IT-12 TMO bulk")
        app.include_router(
            create_bulk_router(
                card_action_fn=make_selective_card_action(fail_ids),
                audit_log=audit_log,
            )
        )
        return TestClient(app)

    def test_it12_case_1_no_failure_success(self, audit_log_path):
        """IT-12 case 1: 0 失败 → outcome=success"""
        client = self._client(set(), audit_log_path)
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ["t1", "t2", "t3"], "action": "pause", "action_params": {}},
        )
        assert r.status_code == 200
        body = r.json()
        assert body["operation"] == "bulk"
        assert body["action"] == "pause"
        assert body["outcome"] == "success"
        assert body["success_count"] == 3
        assert body["failed_count"] == 0
        assert body["failed_ids"] == []
        assert body["rolled_back_ids"] == []
        assert body["reverse_action"] == "resume"
        assert body["total"] == 3

    def test_it12_case_2_partial_failure_10_percent(self, audit_log_path):
        """IT-12 case 2: 1/10 失败 (10% fail, 90% success) → outcome=partial"""
        fail_ids = {"fail-1"}
        client = self._client(fail_ids, audit_log_path)
        ids = [f"ok-{i}" for i in range(9)] + ["fail-1"]
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ids, "action": "pause", "action_params": {}},
        )
        assert r.status_code == 200
        body = r.json()
        assert body["outcome"] == "partial"
        assert body["success_count"] == 9
        assert body["failed_count"] == 1
        assert body["failed_ids"] == ["fail-1"]
        assert body["rolled_back_ids"] == []  # partial, no rollback

    def test_it12_case_3_partial_failure_60_percent_rollback(self, audit_log_path):
        """IT-12 case 3: 3/5 失败 (60% fail) → outcome=rolled_back, 成功卡被回滚"""
        fail_ids = {"fail-1", "fail-2", "fail-3"}
        client = self._client(fail_ids, audit_log_path)
        r = client.post(
            "/api/tmo/bulk",
            json={
                "target_task_ids": ["ok-1", "ok-2", "fail-1", "fail-2", "fail-3"],
                "action": "pause",
                "action_params": {},
            },
        )
        assert r.status_code == 200
        body = r.json()
        assert body["outcome"] == "rolled_back"
        assert body["success_count"] == 2
        assert body["failed_count"] == 3
        assert body["failure_rate"] == pytest.approx(0.60)
        # pause 可逆, 成功卡 ok-1/ok-2 被 resume 回滚
        assert set(body["rolled_back_ids"]) == {"ok-1", "ok-2"}
        assert body["rollback_failed_ids"] == []
        assert body["reverse_action"] == "resume"

    def test_it12_case_4_all_failure_rolled_back(self, audit_log_path):
        """IT-12 case 4: 全部失败 → outcome=rolled_back, 无 success 可 rollback"""
        fail_ids = {"fail-1", "fail-2", "fail-3", "fail-4"}
        client = self._client(fail_ids, audit_log_path)
        r = client.post(
            "/api/tmo/bulk",
            json={
                "target_task_ids": ["fail-1", "fail-2", "fail-3", "fail-4"],
                "action": "pause",
                "action_params": {},
            },
        )
        assert r.status_code == 200
        body = r.json()
        assert body["outcome"] == "rolled_back"
        assert body["success_count"] == 0
        assert body["failed_count"] == 4
        assert body["rolled_back_ids"] == []


# ---------------------------------------------------------------------------
# IT-12 partial: 4 类 action 端点 (pause/resume/cancel/set_priority)
# ---------------------------------------------------------------------------


class TestBulkActionsAllFour:
    """IT-12 partial: 验证 4 类 action 端点全支持"""

    def test_pause_action_success(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ["t1", "t2"], "action": "pause", "action_params": {}},
        )
        assert r.status_code == 200
        assert r.json()["action"] == "pause"
        assert r.json()["outcome"] == "success"

    def test_resume_action_success(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ["t1", "t2"], "action": "resume", "action_params": {}},
        )
        assert r.status_code == 200
        assert r.json()["action"] == "resume"
        assert r.json()["outcome"] == "success"

    def test_cancel_action_success(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ["t1", "t2"], "action": "cancel", "action_params": {}},
        )
        assert r.status_code == 200
        assert r.json()["action"] == "cancel"
        assert r.json()["outcome"] == "success"
        # cancel 不可逆, reverse_action=None
        assert r.json()["reverse_action"] is None

    def test_set_priority_action_success(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={
                "target_task_ids": ["t1", "t2"],
                "action": "set_priority",
                "action_params": {"priority": 5},
            },
        )
        assert r.status_code == 200
        assert r.json()["action"] == "set_priority"
        assert r.json()["outcome"] == "success"
        # set_priority 不可逆
        assert r.json()["reverse_action"] is None

    def _make_app(self, fail_ids: set, audit_log: Path) -> FastAPI:
        app = FastAPI(title="TMO-04 bulk action test")
        app.include_router(
            create_bulk_router(
                card_action_fn=make_selective_card_action(fail_ids),
                audit_log=audit_log,
            )
        )
        return app


# ---------------------------------------------------------------------------
# IT-12 partial: validation 422 错误
# ---------------------------------------------------------------------------


class TestBulkValidation:
    """IT-12 partial: 验证 invalid request 返 422"""

    def test_invalid_action_returns_422(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ["t1"], "action": "invalid_xyz", "action_params": {}},
        )
        assert r.status_code == 422
        # pydantic validation error
        body = r.json()
        assert "detail" in body

    def test_empty_target_ids_returns_422(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": [], "action": "pause", "action_params": {}},
        )
        assert r.status_code == 422

    def test_set_priority_without_priority_returns_422(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={
                "target_task_ids": ["t1"],
                "action": "set_priority",
                "action_params": {},
            },
        )
        assert r.status_code == 422

    def _make_app(self, fail_ids: set, audit_log: Path) -> FastAPI:
        app = FastAPI(title="TMO-04 bulk validation test")
        app.include_router(
            create_bulk_router(
                card_action_fn=make_selective_card_action(fail_ids),
                audit_log=audit_log,
            )
        )
        return app


# ---------------------------------------------------------------------------
# IT-12 partial: audit log 落档实证
# ---------------------------------------------------------------------------


class TestBulkAuditLog:
    """IT-12 partial: 验证每次 flush 落 audit log (per 守门 #13 d Transaction append-only)"""

    def test_audit_log_written_per_flush(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        r = client.post(
            "/api/tmo/bulk",
            json={"target_task_ids": ["t1", "t2"], "action": "pause", "action_params": {}},
        )
        assert r.status_code == 200
        # audit log 应该有 1 行
        assert audit_log_path.exists()
        lines = audit_log_path.read_text(encoding="utf-8").strip().splitlines()
        assert len(lines) == 1
        entry = json.loads(lines[0])
        assert entry["event"] == "bulk.flush"
        assert entry["action"] == "pause"
        assert entry["outcome"] == "success"
        assert entry["success_count"] == 2

    def test_audit_log_multiple_flushes_appended(self, audit_log_path):
        client = TestClient(self._make_app(set(), audit_log_path))
        for i in range(3):
            r = client.post(
                "/api/tmo/bulk",
                json={"target_task_ids": [f"t-{i}"], "action": "pause", "action_params": {}},
            )
            assert r.status_code == 200
        lines = audit_log_path.read_text(encoding="utf-8").strip().splitlines()
        assert len(lines) == 3
        # 全部 success
        for line in lines:
            entry = json.loads(line)
            assert entry["outcome"] == "success"

    def _make_app(self, fail_ids: set, audit_log: Path) -> FastAPI:
        app = FastAPI(title="TMO-04 audit log test")
        app.include_router(
            create_bulk_router(
                card_action_fn=make_selective_card_action(fail_ids),
                audit_log=audit_log,
            )
        )
        return app


# ---------------------------------------------------------------------------
# IT-12 partial: bulk_node + BulkOperationQueue 整合 (不走 HTTP)
# ---------------------------------------------------------------------------


class TestBulkNodeIntegration:
    """IT-12 partial: bulk_node state graph 整合 (per 03 §3.2.1.1)"""

    @pytest.mark.asyncio
    async def test_bulk_node_5_cards_1_fails_20pct_rollback(self, audit_log_path):
        """5 张卡 1 张失败 (20%, success=80% = 阈值) → outcome=partial"""
        # 边界 = 0.20, partial_success_threshold=0.80
        # success_rate(0.80) < threshold(0.80) 严格不成立 → partial
        fail_ids = {"fail-1"}
        q = BulkOperationQueue(
            card_action_fn=make_selective_card_action(fail_ids),
            audit_log=audit_log_path,
        )
        node = make_bulk_node(queue=q)
        state = {
            "active_tmo_operation": {
                "target_task_ids": ["ok-1", "ok-2", "ok-3", "ok-4", "fail-1"],
                "action": "pause",
                "action_params": {},
            }
        }
        diff = await node(state)
        ltr = diff["global_context"]["last_tmo_result"]
        assert ltr["outcome"] == "partial"
        assert ltr["success_count"] == 4
        assert ltr["failed_count"] == 1

    @pytest.mark.asyncio
    async def test_bulk_node_5_cards_2_fail_40pct_rollback_all(self, audit_log_path):
        """5 张卡 2 张失败 (40% fail, success=60% < 80%) → outcome=rolled_back
        验证成功卡 (ok-1/ok-2/ok-3) 被 reverse_action=resume 回滚"""
        fail_ids = {"fail-1", "fail-2"}
        q = BulkOperationQueue(
            card_action_fn=make_selective_card_action(fail_ids),
            audit_log=audit_log_path,
        )
        node = make_bulk_node(queue=q)
        state = {
            "active_tmo_operation": {
                "target_task_ids": ["ok-1", "ok-2", "ok-3", "fail-1", "fail-2"],
                "action": "pause",
                "action_params": {},
            }
        }
        diff = await node(state)
        ltr = diff["global_context"]["last_tmo_result"]
        assert ltr["outcome"] == "rolled_back"
        assert ltr["success_count"] == 3
        assert ltr["failed_count"] == 2
        assert set(ltr["rolled_back_ids"]) == {"ok-1", "ok-2", "ok-3"}


# ---------------------------------------------------------------------------
# IT-12 partial: NFR-TMO-03 partial success threshold 实证
# ---------------------------------------------------------------------------


class TestNfrTmo03Threshold:
    """IT-12 partial: NFR-TMO-03 ≥80% success 实证 (per 03 §3.2.1.1)"""

    @pytest.mark.parametrize(
        "n_total,n_fail,expected_outcome,expected_threshold_breach",
        [
            # (总卡数, 失败数, 期望 outcome, 是否过阈值)
            (5, 0, "success", False),       # 0% fail, 100% success
            (10, 1, "partial", False),      # 10% fail, 90% success >= 80%
            (5, 1, "partial", False),       # 20% fail, 80% success (边界)
            (5, 2, "rolled_back", True),    # 40% fail, 60% success < 80%
            (5, 3, "rolled_back", True),    # 60% fail, 40% success < 80%
            (5, 4, "rolled_back", True),    # 80% fail, 20% success < 80%
            (5, 5, "rolled_back", True),    # 100% fail, 0% success < 80%
        ],
    )
    @pytest.mark.asyncio
    async def test_nfr_tmo_03_threshold_breach(
        self, n_total, n_fail, expected_outcome, expected_threshold_breach, tmp_path
    ):
        """NFR-TMO-03 阈值边界 parametrize 实证"""
        fail_ids = {f"fail-{i}" for i in range(n_fail)}
        ok_ids = [f"ok-{i}" for i in range(n_total - n_fail)]
        all_ids = ok_ids + list(fail_ids)

        q = BulkOperationQueue(
            card_action_fn=make_selective_card_action(fail_ids),
            audit_log=tmp_path / "audit.log",
        )
        q.enqueue(BulkAction(target_task_ids=all_ids, action="pause"))
        results = await q.flush()
        r = results[0]

        assert r.outcome == expected_outcome, (
            f"n_total={n_total} n_fail={n_fail} expected {expected_outcome} got {r.outcome}"
        )
        assert r.failure_rate == pytest.approx(n_fail / n_total)
        # 过阈值时 (rolled_back), pause 可逆 → 成功卡被 resume 回滚
        if expected_threshold_breach:
            assert r.outcome == "rolled_back"
            # pause 可逆, 成功卡都在 rolled_back_ids
            if n_fail < n_total:
                assert set(r.rolled_back_ids) == set(ok_ids)
        else:
            # 未过阈值 (success/partial) → 无 rollback
            assert r.rolled_back_ids == []


# ---------------------------------------------------------------------------
# IT-12 partial: 健康检查端点
# ---------------------------------------------------------------------------


def test_health_endpoint(audit_log_path):
    """GET /api/tmo/bulk/health 返 ok + queue stats"""
    app = FastAPI(title="TMO-04 health test")
    app.include_router(
        create_bulk_router(
            card_action_fn=make_selective_card_action(set()),
            audit_log=audit_log_path,
        )
    )
    client = TestClient(app)
    r = client.get("/api/tmo/bulk/health")
    assert r.status_code == 200
    body = r.json()
    assert body["status"] == "ok"
    assert "queue_stats" in body
    assert body["valid_actions"] == ["cancel", "pause", "resume", "set_priority"]


# ---------------------------------------------------------------------------
# IT-12 partial: 守门 #13 a L0 协调实证 (TMO 全部 L0, 无 L1↔L1)
# ---------------------------------------------------------------------------


def test_l0_coordination_no_subagent_import(audit_log_path):
    """TMO bulk 全部 L0 协调, 不直接调 sub-agent 内部 API

    实证: bulk_queue + bulk_node 只依赖注入的 card_action_fn,
    不 import sub_agent.* (per 守门 #13 a L1↔L1 禁止)
    """
    # 静态分析: 检查 bulk_queue.py 跟 bulk_node.py 不 import sub_agent.*
    bulk_queue_src = (PROJECT_ROOT / "scripts/automation/task_ops/bulk_queue.py").read_text(
        encoding="utf-8"
    )
    bulk_node_src = (PROJECT_ROOT / "scripts/automation/task_ops/nodes/bulk_node.py").read_text(
        encoding="utf-8"
    )
    routes_src = (PROJECT_ROOT / "scripts/automation/api/routes_tmo.py").read_text(
        encoding="utf-8"
    )

    # bulk_queue 不 import sub_agent.* (L0 协调)
    assert "from scripts.automation.sub_agent" not in bulk_queue_src
    assert "import sub_agent" not in bulk_queue_src
    # bulk_node 同理
    assert "from scripts.automation.sub_agent" not in bulk_node_src
    # routes 同理
    assert "from scripts.automation.sub_agent" not in routes_src

    # 但允许 mock_card_action 默认注入 (per 守门 #23 mock 模式)
    # 真接入由 console_server.py / sub_pool 通过 card_action_fn 参数注入
