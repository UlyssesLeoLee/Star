# tests/integration/test_routes_tmo_metadata.py
# IT-16 routes_tmo /api/tmo/metadata 5 端点集成测试 (per G-TMO-04c, 守门 #13 c/d + 守门 #DB-13)
#
# 覆盖:
#   - POST /api/tmo/metadata (M-N7 metadata_node 持久化)
#   - GET  /api/tmo/metadata/{task_id} (读 current, RLS 隔离)
#   - GET  /api/tmo/metadata/{task_id}/history (SCD Type 2 历史)
#   - GET  /api/tmo/metadata/{task_id}/audit (audit log)
#   - GET  /api/tmo/metadata/_health (repo 状态)
#   - 守门 #13 c: Master RLS 必携 (Pydantic min_length=1 校验 + Query(...) 必填)
#   - 守门 #19: Python 化, FastAPI + Pydantic
#   - 守门 #22: routes 不进 main 编译链 (本测试用 TestClient)

from __future__ import annotations

import os
import sys
import tempfile
from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))


# ===== Fixtures =====

@pytest.fixture
def temp_db_path(tmp_path, monkeypatch):
    """每次测试独立临时 SQLite + 设 STAR_TASK_METADATA_DB_PATH env"""
    db_path = str(tmp_path / "task_metadata_routes_test.sqlite")
    monkeypatch.setenv("STAR_TASK_METADATA_DB_PATH", db_path)
    return db_path


@pytest.fixture
def client(temp_db_path):
    """FastAPI TestClient 加载 routes_tmo (会读 env 自动 init schema)"""
    # 重新 import 以走新 env
    import importlib
    if "automation.api.routes_tmo" in sys.modules:
        importlib.reload(sys.modules["automation.api.routes_tmo"])
    from automation.api import routes_tmo
    app = FastAPI(title="TMO metadata routes test")
    app.include_router(routes_tmo.router)
    return TestClient(app)


# ===== IT-16-A: POST /api/tmo/metadata =====

class TestPostMetadata:
    """IT-16-A: POST /api/tmo/metadata (M-N7 metadata_node 持久化)"""

    def test_upsert_creates_v1(self, client):
        """IT-16-A-1: POST 创建 v1 (audit created)"""
        r = client.post("/api/tmo/metadata", json={
            "task_id": "t-1", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "First", "priority": 5, "labels": ["urgent"]},
            "actor_session_id": "sess-1",
        })
        assert r.status_code == 200
        body = r.json()
        assert body["ok"] is True
        assert body["operation"] == "upsert"
        assert body["task_id"] == "t-1"
        assert body["version"] == 1
        assert body["is_current"] is True
        assert body["name"] == "First"
        assert body["priority"] == 5
        assert body["labels"] == ["urgent"]

    def test_upsert_creates_v2_scd(self, client):
        """IT-16-A-2: POST 二次派生 v2 + scd snapshot"""
        client.post("/api/tmo/metadata", json={
            "task_id": "t-2", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "Original", "priority": 5, "labels": ["a"]},
        })
        r2 = client.post("/api/tmo/metadata", json={
            "task_id": "t-2", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "Renamed", "priority": 8, "labels": ["b"], "notes": "second"},
        })
        assert r2.status_code == 200
        assert r2.json()["version"] == 2

    def test_upsert_rejects_empty_tenant_id(self, client):
        """IT-16-A-3: POST tenant_id 空字符串 → 422 (per Pydantic min_length=1)"""
        r = client.post("/api/tmo/metadata", json={
            "task_id": "t-3", "tenant_id": "", "workspace_id": "ws-1",
            "metadata": {"name": "x"},
        })
        assert r.status_code == 422

    def test_upsert_rejects_missing_tenant_id(self, client):
        """IT-16-A-4: POST 缺 tenant_id → 422"""
        r = client.post("/api/tmo/metadata", json={
            "task_id": "t-4", "workspace_id": "ws-1",
            "metadata": {"name": "x"},
        })
        assert r.status_code == 422


# ===== IT-16-B: GET /api/tmo/metadata/{task_id} =====

class TestGetMetadataCurrent:
    """IT-16-B: GET /api/tmo/metadata/{task_id} (读 current + RLS 隔离)"""

    def test_get_returns_current_metadata(self, client):
        """IT-16-B-1: GET 返回当前 metadata"""
        client.post("/api/tmo/metadata", json={
            "task_id": "t-5", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "My task", "priority": 7, "labels": ["x"]},
        })
        r = client.get("/api/tmo/metadata/t-5?tenant_id=tenant-A&workspace_id=ws-1")
        assert r.status_code == 200
        body = r.json()
        assert body["task_id"] == "t-5"
        assert body["name"] == "My task"
        assert body["version"] == 1

    def test_get_rls_isolation_returns_404_for_other_tenant(self, client):
        """IT-16-B-2: GET 跨 tenant → 404 (RLS 隔离)"""
        client.post("/api/tmo/metadata", json={
            "task_id": "t-6", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "A's secret"},
        })
        r = client.get("/api/tmo/metadata/t-6?tenant_id=tenant-B&workspace_id=ws-1")
        assert r.status_code == 404

    def test_get_rejects_missing_query_params(self, client):
        """IT-16-B-3: GET 缺 tenant_id query → 422 (per 守门 #13 c RLS 必携)"""
        r = client.get("/api/tmo/metadata/t-7")
        assert r.status_code == 422


# ===== IT-16-C: GET /api/tmo/metadata/{task_id}/history =====

class TestGetMetadataHistory:
    """IT-16-C: GET /api/tmo/metadata/{task_id}/history (SCD Type 2)"""

    def test_history_empty_for_first_upsert(self, client):
        """IT-16-C-1: 首次 upsert 后 history 应为空 (无 SCD snapshot)"""
        client.post("/api/tmo/metadata", json={
            "task_id": "t-8", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "v1"},
        })
        r = client.get("/api/tmo/metadata/t-8/history?tenant_id=tenant-A&workspace_id=ws-1")
        assert r.status_code == 200
        assert r.json()["history"] == []

    def test_history_has_one_snapshot_after_second_upsert(self, client):
        """IT-16-C-2: 二次 upsert 后 history 派生 1 snapshot"""
        client.post("/api/tmo/metadata", json={
            "task_id": "t-9", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "v1", "priority": 5, "labels": []},
        })
        client.post("/api/tmo/metadata", json={
            "task_id": "t-9", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "v2", "priority": 7, "labels": []},
        })
        r = client.get("/api/tmo/metadata/t-9/history?tenant_id=tenant-A&workspace_id=ws-1")
        assert r.status_code == 200
        history = r.json()["history"]
        assert len(history) == 1
        assert history[0]["version"] == 1
        assert history[0]["previous_metadata"]["name"] == "v1"


# ===== IT-16-D: GET /api/tmo/metadata/{task_id}/audit =====

class TestGetMetadataAudit:
    """IT-16-D: GET /api/tmo/metadata/{task_id}/audit (audit log)"""

    def test_audit_has_created_updated_scd_snapshot(self, client):
        """IT-16-D-1: audit log 包含 created + scd_snapshot + updated 3 类事件"""
        client.post("/api/tmo/metadata", json={
            "task_id": "t-10", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "v1", "priority": 5, "labels": []},
            "actor_session_id": "sess-1",
        })
        client.post("/api/tmo/metadata", json={
            "task_id": "t-10", "tenant_id": "tenant-A", "workspace_id": "ws-1",
            "metadata": {"name": "v2", "priority": 7, "labels": []},
            "actor_session_id": "sess-2",
        })
        r = client.get("/api/tmo/metadata/t-10/audit?tenant_id=tenant-A&workspace_id=ws-1")
        assert r.status_code == 200
        events = r.json()["audit_events"]
        event_types = {e["event_type"] for e in events}
        assert "created" in event_types
        assert "updated" in event_types
        assert "scd_snapshot" in event_types


# ===== IT-16-E: GET /api/tmo/metadata/_health =====

class TestGetMetadataHealth:
    """IT-16-E: GET /api/tmo/metadata/_health (repo 状态)"""

    def test_health_returns_ok(self, client):
        """IT-16-E-1: health 端点返回 ok=True + db_path"""
        r = client.get("/api/tmo/metadata/_health")
        assert r.status_code == 200
        body = r.json()
        assert body["ok"] is True
        assert "db_path" in body
        assert "ts" in body
