# tests/integration/test_metadata_node_persist.py
# IT-17 metadata_node 可选 SQLite 持久化集成测试 (per G-TMO-04d, 守门 #13 c/d + 守门 #19 + 守门 #22)
#
# 覆盖:
#   - 默认 in-memory 模式 (STAR_TASK_METADATA_PERSIST=0) → 不写 SQLite
#   - 启用 persist 模式 (STAR_TASK_METADATA_PERSIST=1) → 写 SQLite
#   - 守门 #13 c: 启用 persist 时 tenant_id + workspace_ids 必携
#   - 守门 #13 d: 启用 persist 时 SCD Type 2 (v1 → v2) + audit 3 事件落档
#   - 守门 #19: Python 化, 标准库 sqlite3
#   - 守门 #22: 持久化失败不应破坏 in-memory 状态 (优雅降级)

from __future__ import annotations

import asyncio
import importlib
import os
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.manager import TaskOperationsManager  # noqa: E402


# ===== Fixtures =====

@pytest.fixture
def temp_db_path(tmp_path, monkeypatch):
    return str(tmp_path / "task_metadata_persist_test.sqlite")


def _reload_metadata_node():
    """重新 import metadata_node 以读取最新 env (per 守门 #22 优雅降级)."""
    if "automation.task_ops.nodes.metadata_node" in sys.modules:
        importlib.reload(sys.modules["automation.task_ops.nodes.metadata_node"])


def _make_manager(task_id: str = "t-persist", tenant_id: str = "tenant-A") -> TaskOperationsManager:
    m = TaskOperationsManager()
    m.sub_pool.add(
        "SA-01",
        task_id=task_id,
        initial_state={"status": "running", "context": {}, "tenant_id": tenant_id},
    )
    return m


# ===== IT-17-A: 默认 in-memory 模式不写 SQLite =====

class TestInMemoryModeDefault:
    """IT-17-A: 默认 in-memory 模式 (STAR_TASK_METADATA_PERSIST=0)"""

    def test_default_mode_does_not_persist(
        self, temp_db_path, monkeypatch,
    ):
        """IT-17-A-1: 默认模式不调用 repo (向后兼容 per 守门 #22)"""
        monkeypatch.setenv("STAR_TASK_METADATA_PERSIST", "0")
        monkeypatch.setenv("STAR_TASK_METADATA_DB_PATH", temp_db_path)
        _reload_metadata_node()
        from automation.task_ops.nodes.metadata_node import metadata_node

        m = _make_manager()
        r = asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "in-memory", "priority": 5, "labels": []},
            "tenant_id": "tenant-A",
            "workspace_ids": ["ws-1"],
            "actor_session_id": "sess-1",
        }, manager=m))
        assert "persisted" not in r
        assert r["updated_fields"] == ["labels", "name", "priority"]
        # 验证 SQLite 文件未被创建 (per 守门 #22 不污染)
        assert not Path(temp_db_path).exists()

    def test_default_mode_update_in_memory(self, monkeypatch):
        """IT-17-A-2: 默认模式仍正常更新 in-memory state (无 regression)"""
        monkeypatch.setenv("STAR_TASK_METADATA_PERSIST", "0")
        _reload_metadata_node()
        from automation.task_ops.nodes.metadata_node import metadata_node

        m = _make_manager()
        asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "v1", "priority": 5, "labels": []},
            "tenant_id": "tenant-A",
            "workspace_ids": ["ws-1"],
        }, manager=m))
        # in-memory handle.state 应更新
        handle = m.sub_pool.get("t-persist")
        assert handle.state["metadata"]["name"] == "v1"


# ===== IT-17-B: 启用 persist 模式 =====

class TestPersistMode:
    """IT-17-B: 启用 persist 模式 (STAR_TASK_METADATA_PERSIST=1)"""

    def test_persist_mode_writes_v1_to_sqlite(
        self, temp_db_path, monkeypatch,
    ):
        """IT-17-B-1: 启用 persist 模式写 SQLite v1"""
        monkeypatch.setenv("STAR_TASK_METADATA_PERSIST", "1")
        monkeypatch.setenv("STAR_TASK_METADATA_DB_PATH", temp_db_path)
        _reload_metadata_node()
        from automation.task_ops.nodes.metadata_node import metadata_node

        m = _make_manager()
        r = asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "persisted v1", "priority": 7, "labels": ["urgent"]},
            "tenant_id": "tenant-A",
            "workspace_ids": ["ws-1"],
            "actor_session_id": "sess-1",
        }, manager=m))
        assert "persisted" in r
        assert r["persisted"]["backend"] == "sqlite_task_metadata"
        assert r["persisted"]["version"] == 1
        assert r["persisted"]["audit_count"] == 1
        # SQLite 文件已创建
        assert Path(temp_db_path).exists()
        # 验证 SQLite 内容
        from automation.task_ops.task_metadata_repo import TaskMetadataRepository
        repo = TaskMetadataRepository(temp_db_path)
        cur = repo.get_current_metadata("t-persist", "tenant-A", "ws-1")
        assert cur.name == "persisted v1"
        assert cur.version == 1

    def test_persist_mode_writes_v2_with_scd_snapshot(
        self, temp_db_path, monkeypatch,
    ):
        """IT-17-B-2: 启用 persist 模式派生 v2 + scd snapshot (per 守门 #13 d)"""
        monkeypatch.setenv("STAR_TASK_METADATA_PERSIST", "1")
        monkeypatch.setenv("STAR_TASK_METADATA_DB_PATH", temp_db_path)
        _reload_metadata_node()
        from automation.task_ops.nodes.metadata_node import metadata_node
        from automation.task_ops.task_metadata_repo import TaskMetadataRepository

        m = _make_manager()
        # v1
        asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "v1", "priority": 5, "labels": []},
            "tenant_id": "tenant-A", "workspace_ids": ["ws-1"],
        }, manager=m))
        # v2
        r2 = asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "v2", "priority": 8, "labels": [], "notes": "second"},
            "tenant_id": "tenant-A", "workspace_ids": ["ws-1"],
        }, manager=m))
        assert r2["persisted"]["version"] == 2
        assert r2["persisted"]["audit_count"] == 3  # created + scd_snapshot + updated

        repo = TaskMetadataRepository(temp_db_path)
        cur = repo.get_current_metadata("t-persist", "tenant-A", "ws-1")
        assert cur.version == 2
        scd = repo.get_scd_history("t-persist", "tenant-A", "ws-1")
        assert len(scd) == 1
        assert scd[0]["version"] == 1
        assert scd[0]["previous_metadata"]["name"] == "v1"

    def test_persist_mode_rls_isolation(
        self, temp_db_path, monkeypatch,
    ):
        """IT-17-B-3: 启用 persist 模式 tenant_id 隔离 (per 守门 #13 c)"""
        monkeypatch.setenv("STAR_TASK_METADATA_PERSIST", "1")
        monkeypatch.setenv("STAR_TASK_METADATA_DB_PATH", temp_db_path)
        _reload_metadata_node()
        from automation.task_ops.nodes.metadata_node import metadata_node
        from automation.task_ops.task_metadata_repo import TaskMetadataRepository

        m = _make_manager()
        asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "A's data", "priority": 5, "labels": []},
            "tenant_id": "tenant-A", "workspace_ids": ["ws-1"],
        }, manager=m))
        # tenant-B 读不到
        repo = TaskMetadataRepository(temp_db_path)
        cur = repo.get_current_metadata("t-persist", "tenant-B", "ws-1")
        assert cur is None


# ===== IT-17-C: 优雅降级 (per 守门 #22) =====

class TestGracefulDegradation:
    """IT-17-C: 持久化失败不应破坏 in-memory 状态 (per 守门 #22)"""

    def test_persist_failure_keeps_in_memory_state(self, monkeypatch, tmp_path, caplog):
        """IT-17-C-1: 持久化失败 (DB 路径无效) 不破坏 in-memory 状态, 仍返 ok result"""
        # 故意给一个无效 db path (目录不存在 + 不可写)
        # 用一个会失败的 db path: Windows reserved name
        bad_path = "CON"  # Windows 设备名, 必失败
        monkeypatch.setenv("STAR_TASK_METADATA_PERSIST", "1")
        monkeypatch.setenv("STAR_TASK_METADATA_DB_PATH", bad_path)
        _reload_metadata_node()
        from automation.task_ops.nodes.metadata_node import metadata_node

        m = _make_manager()
        r = asyncio.run(metadata_node({
            "operation": "metadata",
            "target_task_id": "t-persist",
            "metadata": {"name": "fallback", "priority": 5, "labels": []},
            "tenant_id": "tenant-A", "workspace_ids": ["ws-1"],
        }, manager=m))
        # 仍返 ok + updated_fields (per 守门 #22 优雅降级)
        assert r["operation"] == "metadata"
        assert r["updated_fields"] == ["labels", "name", "priority"]
        # persisted 应为 None (持久化失败, 但 in-memory 仍 OK)
        assert r.get("persisted") is None
        # in-memory state 仍更新
        handle = m.sub_pool.get("t-persist")
        assert handle.state["metadata"]["name"] == "fallback"
