# tests/integration/test_task_metadata_repo.py
# IT-15 TaskMetadataRepository 集成测试 (per G-TMO-04b, 守门 #13 c/d + 守门 #DB-13)
#
# 覆盖:
#   - upsert SCD Type 2 完整流程 (v1 → v2 + scd snapshot + audit)
#   - get_current_metadata 跨 tenant RLS 隔离
#   - get_scd_history / get_audit_log 时间倒序
#   - audit 5 类事件 + invalid 拒绝
#   - Master 表物理删除禁止 (per 守门 #13 c)
#   - 守门 #19: Python 化, sqlite3 only

from __future__ import annotations

import json
import sqlite3
import sys
import tempfile
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.task_metadata_ddl import init_schema  # noqa: E402
from automation.task_ops.task_metadata_repo import (  # noqa: E402
    TaskMetadataRecord,
    TaskMetadataRepository,
)


# ===== Fixtures =====

@pytest.fixture
def temp_db_path(tmp_path):
    return str(tmp_path / "task_metadata_repo_test.sqlite")


@pytest.fixture
def repo(temp_db_path):
    init_schema(temp_db_path)
    return TaskMetadataRepository(temp_db_path)


# ===== IT-15-A: upsert SCD Type 2 =====

class TestUpsertScdType2:
    """IT-15-A: upsert_metadata SCD Type 2 完整流程"""

    def test_first_insert_creates_v1(self, repo):
        """IT-15-A-1: 首次 insert 创建 v1 (audit created)"""
        r = repo.upsert_metadata(
            "t-1", "tenant-A", "ws-1",
            {"name": "First", "priority": 5, "labels": ["urgent"]},
            actor_session_id="sess-1",
        )
        assert r.task_id == "t-1"
        assert r.version == 1
        assert r.is_current is True
        assert r.name == "First"
        assert r.priority == 5
        assert r.labels == ["urgent"]
        # audit log: 1 created event
        audit = repo.get_audit_log("t-1", "tenant-A", "ws-1")
        assert len(audit) == 1
        assert audit[0]["event_type"] == "created"
        assert audit[0]["actor_session_id"] == "sess-1"

    def test_second_upsert_creates_v2_scd_snapshot(self, repo):
        """IT-15-A-2: 二次 upsert 派生 v2 + scd_snapshot (audit updated + scd_snapshot)"""
        repo.upsert_metadata("t-2", "tenant-A", "ws-1",
                            {"name": "Original", "priority": 5, "labels": ["a"]}, actor_session_id="sess-1")
        r2 = repo.upsert_metadata("t-2", "tenant-A", "ws-1",
                                 {"name": "Renamed", "priority": 8, "labels": ["b"], "notes": "second"},
                                 actor_session_id="sess-2")
        assert r2.version == 2
        assert r2.is_current is True
        assert r2.name == "Renamed"
        assert r2.priority == 8
        # scd history: 1 row
        scd = repo.get_scd_history("t-2", "tenant-A", "ws-1")
        assert len(scd) == 1
        assert scd[0]["version"] == 1
        assert scd[0]["previous_metadata"]["name"] == "Original"
        assert scd[0]["previous_metadata"]["priority"] == 5
        assert scd[0]["previous_metadata"]["labels"] == ["a"]
        # audit log: created + scd_snapshot + updated = 3 events
        audit = repo.get_audit_log("t-2", "tenant-A", "ws-1")
        assert len(audit) == 3
        event_types = [a["event_type"] for a in audit]
        assert "created" in event_types
        assert "updated" in event_types
        assert "scd_snapshot" in event_types

    def test_third_upsert_creates_v3_with_two_scd(self, repo):
        """IT-15-A-3: 三次 upsert 派生 v3 + 2 个 scd snapshot"""
        for i, name in enumerate(["v1-name", "v2-name", "v3-name"], start=1):
            repo.upsert_metadata("t-3", "tenant-A", "ws-1",
                                {"name": name, "priority": 5, "labels": []}, actor_session_id=f"sess-{i}")
        # current = v3
        cur = repo.get_current_metadata("t-3", "tenant-A", "ws-1")
        assert cur.version == 3
        assert cur.name == "v3-name"
        # scd history = 2 rows (v1, v2 都 snapshot)
        scd = repo.get_scd_history("t-3", "tenant-A", "ws-1")
        assert len(scd) == 2
        assert scd[0]["version"] == 2  # DESC
        assert scd[0]["previous_metadata"]["name"] == "v2-name"
        assert scd[1]["version"] == 1
        assert scd[1]["previous_metadata"]["name"] == "v1-name"


# ===== IT-15-B: get_current_metadata RLS =====

class TestRlsIsolation:
    """IT-15-B: get_current_metadata 跨 tenant RLS 隔离"""

    def test_tenant_a_sees_own_metadata(self, repo):
        """IT-15-B-1: tenant-A 读自己 metadata"""
        repo.upsert_metadata("t-4", "tenant-A", "ws-1", {"name": "A's task", "priority": 5, "labels": []})
        cur = repo.get_current_metadata("t-4", "tenant-A", "ws-1")
        assert cur is not None
        assert cur.tenant_id == "tenant-A"
        assert cur.name == "A's task"

    def test_tenant_b_cannot_see_tenant_a_metadata(self, repo):
        """IT-15-B-2: tenant-B 读 tenant-A metadata 应 None (RLS 隔离)"""
        repo.upsert_metadata("t-5", "tenant-A", "ws-1", {"name": "A's secret", "priority": 5, "labels": []})
        cur = repo.get_current_metadata("t-5", "tenant-B", "ws-1")
        assert cur is None

    def test_workspace_isolation_within_tenant(self, repo):
        """IT-15-B-3: 同 tenant 不同 workspace 隔离"""
        repo.upsert_metadata("t-6", "tenant-A", "ws-1", {"name": "ws1 task", "priority": 5, "labels": []})
        repo.upsert_metadata("t-6", "tenant-A", "ws-2", {"name": "ws2 task", "priority": 8, "labels": []})
        cur_ws1 = repo.get_current_metadata("t-6", "tenant-A", "ws-1")
        cur_ws2 = repo.get_current_metadata("t-6", "tenant-A", "ws-2")
        assert cur_ws1.name == "ws1 task"
        assert cur_ws2.name == "ws2 task"


# ===== IT-15-C: get_scd_history 时间倒序 =====

class TestScdHistoryOrder:
    """IT-15-C: get_scd_history 按 version DESC 倒序"""

    def test_scd_history_descending_version(self, repo):
        """IT-15-C-1: scd history 倒序 (新 → 旧)"""
        for i, name in enumerate(["a", "b", "c", "d"], start=1):
            repo.upsert_metadata("t-7", "tenant-A", "ws-1", {"name": name, "priority": 5, "labels": []})
        scd = repo.get_scd_history("t-7", "tenant-A", "ws-1")
        assert len(scd) == 3  # 4 upsert → 3 scd (v4 当前, v1-v3 snapshot)
        versions = [s["version"] for s in scd]
        assert versions == [3, 2, 1]

    def test_scd_history_limit(self, repo):
        """IT-15-C-2: scd history limit 限制"""
        for i in range(5):
            repo.upsert_metadata("t-8", "tenant-A", "ws-1", {"name": f"v{i+1}", "priority": 5, "labels": []})
        scd = repo.get_scd_history("t-8", "tenant-A", "ws-1", limit=2)
        assert len(scd) == 2
        assert scd[0]["version"] == 4
        assert scd[1]["version"] == 3


# ===== IT-15-D: audit log 5 类事件 =====

class TestAuditLog:
    """IT-15-D: get_audit_log 5 类事件 + invalid 拒绝"""

    def test_audit_log_contains_created_updated_scd_snapshot(self, repo):
        """IT-15-D-1: audit log 包含 created / updated / scd_snapshot 3 类"""
        for i in range(3):
            repo.upsert_metadata("t-9", "tenant-A", "ws-1", {"name": f"v{i+1}", "priority": 5, "labels": []})
        audit = repo.get_audit_log("t-9", "tenant-A", "ws-1")
        event_types = {a["event_type"] for a in audit}
        assert "created" in event_types
        assert "updated" in event_types
        assert "scd_snapshot" in event_types

    def test_audit_log_includes_actor_session_id(self, repo):
        """IT-15-D-2: audit log 含 actor_session_id"""
        repo.upsert_metadata("t-10", "tenant-A", "ws-1", {"name": "x", "priority": 5, "labels": []}, actor_session_id="my-sess-42")
        audit = repo.get_audit_log("t-10", "tenant-A", "ws-1")
        assert len(audit) >= 1
        assert any(a["actor_session_id"] == "my-sess-42" for a in audit)


# ===== IT-15-E: Master 表物理删除禁止 (per 守门 #13 c) =====

class TestMasterDeleteForbidden:
    """IT-15-E: Master 表物理删除禁止 (per 守门 #13 c)"""

    def test_delete_metadata_raises_permission_error(self, repo):
        """IT-15-E-1: delete_metadata 抛 PermissionError (per 守门 #13 c 物理删除禁止)"""
        repo.upsert_metadata("t-11", "tenant-A", "ws-1", {"name": "x", "priority": 5, "labels": []})
        with pytest.raises(PermissionError, match="physical DELETE"):
            repo.delete_metadata("t-11", "tenant-A", "ws-1")

    def test_master_table_still_has_row_after_attempted_delete(self, repo):
        """IT-15-E-2: delete_metadata 失败后, current row 仍存在 (物理删除禁止)"""
        repo.upsert_metadata("t-12", "tenant-A", "ws-1", {"name": "preserve me", "priority": 5, "labels": []})
        try:
            repo.delete_metadata("t-12", "tenant-A", "ws-1")
        except PermissionError:
            pass
        # 仍能读出
        cur = repo.get_current_metadata("t-12", "tenant-A", "ws-1")
        assert cur is not None
        assert cur.name == "preserve me"


# ===== IT-15-F: 守门 #19 sqlite3 only =====

class TestStandardLibraryOnly:
    """IT-15-F: 守门 #19 Python 化 (sqlite3 only, 无 SQLAlchemy)"""

    def test_repo_conn_uses_stdlib_sqlite3(self, repo):
        """IT-15-F-1: _conn 用标准库 sqlite3 (per 守门 #19)"""
        conn = repo._conn()
        try:
            assert isinstance(conn, sqlite3.Connection)
        finally:
            conn.close()

    def test_metadata_persists_across_connections(self, repo):
        """IT-15-F-2: metadata 跨 connection 持久化 (SQLite 持久化有效)"""
        repo.upsert_metadata("t-13", "tenant-A", "ws-1", {"name": "persistent", "priority": 7, "labels": ["x"]})
        # 模拟新 connection
        new_conn = sqlite3.connect(repo.db_path)
        cur = new_conn.execute(
            "SELECT name, priority, labels_json FROM task_metadata WHERE task_id = ? AND is_current = 1",
            ("t-13",),
        )
        row = cur.fetchone()
        new_conn.close()
        assert row is not None
        assert row[0] == "persistent"
        assert row[1] == 7
        assert json.loads(row[2]) == ["x"]
