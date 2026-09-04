# tests/integration/test_task_metadata_ddl.py
# IT-14 task_metadata SQLite DDL 集成测试 (per G-TMO-04, 守门 #13 c Master RLS + 守门 #13 d SCD Type 2)
#
# 覆盖:
#   - 4 表 W/T/M 分类 (per 守门 #DB-13)
#   - 7 索引 + tenant_id RLS 必携 + workspace_id RLS 必携
#   - 守门 #13 c: Master 表 (task_metadata + task_metadata_scd) 物理删除禁止 (schema 验证)
#   - 守门 #13 d: SCD Type 2 (version + is_current 字段) + audit 5 类事件约束
#   - 守门 #19: Python 化, 标准库 sqlite3

from __future__ import annotations

import os
import sqlite3
import sys
import tempfile
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.task_metadata_ddl import (  # noqa: E402
    DDL_INDEXES,
    DDL_TASK_METADATA,
    DDL_TASK_METADATA_AUDIT,
    DDL_TASK_METADATA_SCD,
    DDL_TASK_METADATA_SESSION,
    init_schema,
    validate_schema,
)


# ===== Fixtures =====

@pytest.fixture
def temp_db_path(tmp_path):
    """每次测试独立临时 SQLite 文件"""
    return str(tmp_path / "task_metadata_test.sqlite")


@pytest.fixture
def initialized_db(temp_db_path):
    """已初始化 schema 的临时 SQLite"""
    init_schema(temp_db_path)
    return temp_db_path


# ===== IT-14-A: W/T/M 4 表分类 =====

class TestTaskMetadataSchema:
    """IT-14-A: task_metadata 4 表 W/T/M 分类 (per 守门 #DB-13)"""

    def test_init_schema_creates_four_tables(self, temp_db_path):
        """IT-14-A-1: init_schema 落档 4 表"""
        result = init_schema(temp_db_path)
        assert result["schema_status"] == "initialized"
        assert set(result["tables"]) == {
            "task_metadata", "task_metadata_scd",
            "task_metadata_audit", "task_metadata_session",
        }

    def test_init_schema_creates_seven_indexes(self, temp_db_path):
        """IT-14-A-2: init_schema 落档 7 索引"""
        result = init_schema(temp_db_path)
        assert len(result["indexes"]) == 7
        # 验证 RLS 索引存在
        assert "idx_task_metadata_tenant" in result["indexes"]
        assert "idx_task_metadata_scd_tenant" in result["indexes"]
        assert "idx_task_metadata_audit_tenant" in result["indexes"]

    def test_validate_schema_passes_after_init(self, initialized_db):
        """IT-14-A-3: validate_schema 通过 (per 4 表 + 7 索引 + RLS 字段)"""
        result = validate_schema(initialized_db)
        assert result["schema_status"] == "valid"
        assert result["tenant_id_check"] == "pass"
        assert result["workspace_id_check"] == "pass"

    def test_validate_schema_raises_on_empty_db(self, temp_db_path):
        """IT-14-A-4: 空 db validate 失败 (per 4 表不全)"""
        conn = sqlite3.connect(temp_db_path)
        conn.close()
        with pytest.raises(RuntimeError, match="schema 不完整"):
            validate_schema(temp_db_path)


# ===== IT-14-B: 守门 #13 c Master RLS 必携 =====

class TestMasterRlsFields:
    """IT-14-B: 守门 #13 c Master RLS 必携 tenant_id / workspace_id"""

    def test_task_metadata_has_tenant_id_workspace_id(self, initialized_db):
        """IT-14-B-1: task_metadata 必含 tenant_id + workspace_id (守门 #13 c)"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "tenant_id" in columns
        assert "workspace_id" in columns

    def test_task_metadata_scd_has_tenant_id_workspace_id(self, initialized_db):
        """IT-14-B-2: task_metadata_scd 必含 tenant_id + workspace_id (守门 #13 c)"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata_scd)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "tenant_id" in columns
        assert "workspace_id" in columns

    def test_task_metadata_audit_has_tenant_id_workspace_id(self, initialized_db):
        """IT-14-B-3: task_metadata_audit 必含 tenant_id + workspace_id (Transaction 但 RLS 仍必携)"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata_audit)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "tenant_id" in columns
        assert "workspace_id" in columns

    def test_task_metadata_session_has_tenant_id_workspace_id(self, initialized_db):
        """IT-14-B-4: task_metadata_session 必含 tenant_id + workspace_id (Work 也 RLS)"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata_session)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "tenant_id" in columns
        assert "workspace_id" in columns


# ===== IT-14-C: 守门 #13 d SCD Type 2 =====

class TestScdType2:
    """IT-14-C: 守门 #13 d SCD Type 2 关系变更留痕"""

    def test_task_metadata_has_version_is_current(self, initialized_db):
        """IT-14-C-1: task_metadata 必含 version + is_current 字段 (SCD Type 2)"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "version" in columns
        assert "is_current" in columns

    def test_task_metadata_scd_snapshot_fields(self, initialized_db):
        """IT-14-C-2: task_metadata_scd 必含 snapshot_id + previous_metadata_json + version"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata_scd)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "snapshot_id" in columns
        assert "previous_metadata_json" in columns
        assert "version" in columns
        assert "snapshot_at_ms" in columns

    def test_insert_metadata_then_snapshot_then_new_version(self, initialized_db):
        """IT-14-C-3: 完整 SCD 流程 — 插入 v1 + 更新派生 v2 + 旧 metadata snapshot 落档"""
        conn = sqlite3.connect(initialized_db)
        try:
            # v1
            conn.execute(
                """INSERT INTO task_metadata
                   (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                ("tm-001", "task-001", "tenant-A", "ws-1", "Original", "[]", "v1 notes", 5, 1, 1, 1000, 1000),
            )
            # snapshot v1 (per 守门 #13 d)
            conn.execute(
                """INSERT INTO task_metadata_scd
                   (snapshot_id, task_id, tenant_id, workspace_id, version, previous_metadata_json, snapshot_at_ms, snapshot_reason)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                ("scd-001", "task-001", "tenant-A", "ws-1", 1,
                 '{"name":"Original","priority":5}', 2000, "metadata_update"),
            )
            # v1 is_current = 0
            conn.execute("UPDATE task_metadata SET is_current = 0 WHERE task_id = ? AND version = 1", ("task-001",))
            # v2 (new current)
            conn.execute(
                """INSERT INTO task_metadata
                   (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                ("tm-002", "task-001", "tenant-A", "ws-1", "Renamed", "[]", "v2 notes", 8, 2, 1, 3000, 3000),
            )
            conn.commit()

            # 验证: 1 row is_current=1 (v2), 1 row is_current=0 (v1)
            cur = conn.execute(
                "SELECT version, is_current, name FROM task_metadata WHERE task_id = ? ORDER BY version", ("task-001",)
            )
            rows = cur.fetchall()
            assert len(rows) == 2
            assert rows[0] == (1, 0, "Original")
            assert rows[1] == (2, 1, "Renamed")

            # 验证: scd history 永存 1 snapshot
            cur = conn.execute(
                "SELECT snapshot_id, version, previous_metadata_json FROM task_metadata_scd WHERE task_id = ?", ("task-001",)
            )
            scd_rows = cur.fetchall()
            assert len(scd_rows) == 1
            assert scd_rows[0][0] == "scd-001"
            assert scd_rows[0][1] == 1
        finally:
            conn.close()


# ===== IT-14-D: 守门 #13 d Transaction audit =====

class TestAuditEventConstraint:
    """IT-14-D: 守门 #13 d Transaction audit 5 类事件约束"""

    def test_audit_event_type_constraint_accepts_5_types(self, initialized_db):
        """IT-14-D-1: task_metadata_audit event_type 接受 5 类"""
        conn = sqlite3.connect(initialized_db)
        try:
            for event_type in ("created", "updated", "scd_snapshot", "rls_violation", "validation_failed"):
                conn.execute(
                    """INSERT INTO task_metadata_audit
                       (audit_id, task_id, tenant_id, workspace_id, actor_session_id, event_type, event_at_ms, metadata_diff_json, snapshot_id)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    (f"audit-{event_type}", "task-001", "tenant-A", "ws-1", "sess-1", event_type, 1000, "{}", None),
                )
            conn.commit()
            cur = conn.execute("SELECT COUNT(*) FROM task_metadata_audit WHERE task_id = 'task-001'")
            count = cur.fetchone()[0]
            assert count == 5
        finally:
            conn.close()

    def test_audit_event_type_constraint_rejects_unknown(self, initialized_db):
        """IT-14-D-2: task_metadata_audit event_type 拒绝未知值 (per CHECK 约束)"""
        conn = sqlite3.connect(initialized_db)
        try:
            with pytest.raises(sqlite3.IntegrityError, match="CHECK constraint"):
                conn.execute(
                    """INSERT INTO task_metadata_audit
                       (audit_id, task_id, tenant_id, workspace_id, actor_session_id, event_type, event_at_ms, metadata_diff_json, snapshot_id)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    ("audit-bad", "task-001", "tenant-A", "ws-1", "sess-1", "unknown_event", 1000, "{}", None),
                )
        finally:
            conn.close()


# ===== IT-14-E: 守门 #DB-13 a Work 表 retention =====

class TestWorkRetention:
    """IT-14-E: 守门 #DB-13 a Work 表 (task_metadata_session) 短 TTL 作业中"""

    def test_session_has_ttl_and_active_fields(self, initialized_db):
        """IT-14-E-1: task_metadata_session 必含 session_expires_ms + is_active (Work TTL 守门)"""
        conn = sqlite3.connect(initialized_db)
        cur = conn.execute("PRAGMA table_info(task_metadata_session)")
        columns = {row[1] for row in cur.fetchall()}
        conn.close()
        assert "session_expires_ms" in columns
        assert "is_active" in columns

    def test_session_insert_and_expire(self, initialized_db):
        """IT-14-E-2: Work session 可插入 + 标记 is_active=0 (per 作业完成清理)"""
        conn = sqlite3.connect(initialized_db)
        try:
            conn.execute(
                """INSERT INTO task_metadata_session
                   (session_id, task_id, tenant_id, workspace_id, session_started_ms, session_expires_ms, session_state_json, is_active)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                ("sess-001", "task-001", "tenant-A", "ws-1", 1000, 60000, '{"phase":"validating"}', 1),
            )
            conn.commit()
            # 标记过期 (per 守门 #DB-13 a 完成后清理)
            conn.execute(
                "UPDATE task_metadata_session SET is_active = 0 WHERE session_id = ?", ("sess-001",)
            )
            conn.commit()
            cur = conn.execute(
                "SELECT is_active FROM task_metadata_session WHERE session_id = ?", ("sess-001",)
            )
            assert cur.fetchone()[0] == 0
        finally:
            conn.close()


# ===== IT-14-F: 守门 #13 c priority CHECK 约束 =====

class TestPriorityCheckConstraint:
    """IT-14-F: task_metadata priority 1-10 CHECK 约束"""

    def test_priority_rejects_zero(self, initialized_db):
        """IT-14-F-1: priority=0 拒绝 (per CHECK BETWEEN 1 AND 10)"""
        conn = sqlite3.connect(initialized_db)
        try:
            with pytest.raises(sqlite3.IntegrityError, match="CHECK constraint"):
                conn.execute(
                    """INSERT INTO task_metadata
                       (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    ("tm-bad", "task-001", "tenant-A", "ws-1", "x", "[]", None, 0, 1, 1, 1000, 1000),
                )
        finally:
            conn.close()

    def test_priority_rejects_eleven(self, initialized_db):
        """IT-14-F-2: priority=11 拒绝 (per CHECK BETWEEN 1 AND 10)"""
        conn = sqlite3.connect(initialized_db)
        try:
            with pytest.raises(sqlite3.IntegrityError, match="CHECK constraint"):
                conn.execute(
                    """INSERT INTO task_metadata
                       (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    ("tm-bad", "task-001", "tenant-A", "ws-1", "x", "[]", None, 11, 1, 1, 1000, 1000),
                )
        finally:
            conn.close()

    def test_priority_accepts_boundaries(self, initialized_db):
        """IT-14-F-3: priority=1 + priority=10 接受 (per CHECK 边界值)"""
        conn = sqlite3.connect(initialized_db)
        try:
            for p in (1, 10):
                conn.execute(
                    """INSERT INTO task_metadata
                       (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    (f"tm-{p}", f"task-{p}", "tenant-A", "ws-1", "x", "[]", None, p, 1, 1, 1000, 1000),
                )
            conn.commit()
            cur = conn.execute("SELECT COUNT(*) FROM task_metadata")
            assert cur.fetchone()[0] == 2
        finally:
            conn.close()


# ===== IT-14-G: 守门 #13 c RLS 多租户隔离 =====

class TestRlsMultiTenantIsolation:
    """IT-14-G: 守门 #13 c RLS 多租户隔离 (索引 + 查询路径)"""

    def test_rls_index_supports_tenant_workspace_query(self, initialized_db):
        """IT-14-G-1: idx_task_metadata_tenant 支持 (tenant_id, workspace_id) 查询路径"""
        conn = sqlite3.connect(initialized_db)
        try:
            # 插入 2 个 tenant 的 task_metadata
            for tid, ws in (("tenant-A", "ws-1"), ("tenant-A", "ws-2"), ("tenant-B", "ws-1")):
                conn.execute(
                    """INSERT INTO task_metadata
                       (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    (f"tm-{tid}-{ws}", f"task-{tid}-{ws}", tid, ws, "x", "[]", None, 5, 1, 1, 1000, 1000),
                )
            conn.commit()
            # tenant-A 隔离查询
            cur = conn.execute(
                "SELECT COUNT(*) FROM task_metadata WHERE tenant_id = ? AND workspace_id = ?", ("tenant-A", "ws-1")
            )
            assert cur.fetchone()[0] == 1
            # tenant-B 隔离查询
            cur = conn.execute(
                "SELECT COUNT(*) FROM task_metadata WHERE tenant_id = ?", ("tenant-B",)
            )
            assert cur.fetchone()[0] == 1
        finally:
            conn.close()

    def test_unique_constraint_on_task_id_version(self, initialized_db):
        """IT-14-G-2: UNIQUE (task_id, version) 约束 (per SCD Type 2)"""
        conn = sqlite3.connect(initialized_db)
        try:
            conn.execute(
                """INSERT INTO task_metadata
                   (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                ("tm-dup", "task-dup", "tenant-A", "ws-1", "x", "[]", None, 5, 1, 1, 1000, 1000),
            )
            conn.commit()
            # 重复 (task_id, version=1) 应被 UNIQUE 约束拒绝
            with pytest.raises(sqlite3.IntegrityError, match="UNIQUE constraint"):
                conn.execute(
                    """INSERT INTO task_metadata
                       (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                    ("tm-dup-2", "task-dup", "tenant-A", "ws-1", "y", "[]", None, 5, 1, 1, 1000, 1000),
                )
        finally:
            conn.close()
