# tests/integration/test_tmo_split.py
# IT-11 TMO split end-to-end (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §8.2)
#
# 集成测试范围: split_node + TaskOperationsManager + /api/tmo/split FastAPI 端点整合 (per UC-10)
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调
#   - 守门 #13 d: snapshot 永存 (Transaction append-only), supersede 终态不删除
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #20: 子代理 dispatch 必先 brief (TMO-02 父会话 Mavis 委托)
#   - 守门 #22: 调试控制台不污染 main (走 port 8080 console_server.py)
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 → subprocess

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))


# ===== IT-11-A: split_node + sub_pool 整合 =====

@pytest.mark.asyncio
async def test_split_full_flow():
    """IT-11-A: 完整 split_node + sub_pool 整合 (per UC-10)"""
    from automation.task_ops.manager import TaskOperationsManager
    from automation.task_ops.nodes.split_node import split_node

    manager = TaskOperationsManager()
    # 准备 1 个 L1 task (a) — 拆分源
    manager.sub_pool.add(
        task_type="SA-09",
        task_id="task-alpha",
        initial_state={
            "status": "running",
            "context": {"description": "task alpha description", "priority": 5},
        },
    )

    # 模拟用户 chat bar: "拆分任务 alpha"
    state = {
        "operation": "split",
        "target_task_id": "task-alpha",
        "split_strategy": "context_fork",
        "split_count": 2,
        "actor_session_id": "session-uc10-001",
    }
    result = await split_node(state=state, manager=manager)

    # 整合断言:
    # 1. snapshot_checkpoint_id 已生成 (per 守门 #13 d Transaction append-only)
    assert result["snapshot_checkpoint_id"].startswith("cp-")
    assert len(manager.sub_pool.get("task-alpha").checkpoints) == 1

    # 2. dispatch a1 + a2 (2 个 fork, 相同 task_type as a, forked context)
    assert len(result["new_task_ids"]) == 2
    assert result["new_task_ids"] == ["task-alpha-a1", "task-alpha-a2"]
    a1 = manager.sub_pool.get("task-alpha-a1")
    a2 = manager.sub_pool.get("task-alpha-a2")
    assert a1.task_type == "SA-09"  # 跟父 task_type 一致 (per 03 §3.2.1.1)
    assert a2.task_type == "SA-09"
    # base_context 继承
    assert a1.state["context"]["description"] == "task alpha description"
    assert a1.state["context"]["priority"] == 5
    # 守门 #13 d: _split_from / _split_index / _split_snapshot 注入
    assert a1.state["context"]["_split_from"] == "task-alpha"
    assert a1.state["context"]["_split_index"] == 0
    assert a2.state["context"]["_split_index"] == 1
    assert a1.state["context"]["_split_snapshot"] == result["snapshot_checkpoint_id"]

    # 3. 源 task a 标 superseded + split_into (per 守门 #13 d 终态不删除)
    assert manager.sub_pool.get("task-alpha").state["status"] == "superseded"
    assert manager.sub_pool.get("task-alpha").state["split_into"] == ["task-alpha-a1", "task-alpha-a2"]
    assert manager.sub_pool.get("task-alpha").state["superseded_by"] is None  # split 没取代指向

    # 4. UI events 3 个 (1 update a + 2 create a1/a2)
    assert len(result["ui_events"]) == 3
    update_events = [e for e in result["ui_events"] if e["type"] == "TaskCardUpdate"]
    create_events = [e for e in result["ui_events"] if e["type"] == "TaskCardCreate"]
    assert len(update_events) == 1
    assert len(create_events) == 2

    # 5. TMO operation done
    assert result["active_tmo_operation"] is None

    # 6. global_context.last_tmo_result 完整
    last = result["global_context"]["last_tmo_result"]
    assert last["operation"] == "split"
    assert last["target_task_id"] == "task-alpha"
    assert last["split_strategy"] == "context_fork"
    assert last["split_count"] == 2
    assert last["snapshot_checkpoint_id"] == result["snapshot_checkpoint_id"]
    assert last["new_task_ids"] == ["task-alpha-a1", "task-alpha-a2"]


# ===== IT-11-B: FastAPI /api/tmo/split 端点 e2e =====

def _build_test_app():
    """构建含 /api/tmo/split 端点的 FastAPI app (mock, 不依赖 console_server)"""
    from fastapi import FastAPI
    from fastapi.testclient import TestClient

    from automation.api.routes_tmo import router as tmo_router

    app = FastAPI(title="IT-11 TMO split e2e")
    app.include_router(tmo_router)
    return app, TestClient(app)


class TestTmoSplitEndpoint:
    """IT-11-B: /api/tmo/split 端点 e2e (FastAPI TestClient)"""

    def test_split_endpoint_default_count_2(self):
        """IT-11-B-1: POST /api/tmo/split 默认 split_count=2 跑通"""
        app, client = _build_test_app()

        # mock 模式: target 不存在自动 add (per routes_tmo.py tmo_split)
        resp = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-x"},
        )
        assert resp.status_code == 200, resp.text
        body = resp.json()
        assert body["ok"] is True
        assert body["node"] == "M-N2"
        assert body["target_task_id"] == "task-x"
        assert body["result"]["operation"] == "split"
        assert body["result"]["split_count"] == 2
        assert body["result"]["split_strategy"] == "context_fork"
        # 默认 2 个 new_task_ids
        assert len(body["result"]["new_task_ids"]) == 2
        assert body["result"]["new_task_ids"] == ["task-x-a1", "task-x-a2"]
        # snapshot_checkpoint_id 已生成
        assert body["result"]["snapshot_checkpoint_id"].startswith("cp-")
        # 3 个 UI 事件 (1 update + 2 create)
        assert len(body["ui_events"]) == 3
        # duration_ms > 0
        assert body["duration_ms"] > 0

    def test_split_endpoint_count_3(self):
        """IT-11-B-2: POST /api/tmo/split split_count=3 跑通"""
        app, client = _build_test_app()

        resp = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-y", "split_count": 3, "split_strategy": "checkpoint_fork"},
        )
        assert resp.status_code == 200, resp.text
        body = resp.json()
        assert body["ok"] is True
        assert body["result"]["split_count"] == 3
        assert body["result"]["split_strategy"] == "checkpoint_fork"
        assert len(body["result"]["new_task_ids"]) == 3
        assert body["result"]["new_task_ids"] == ["task-y-a1", "task-y-a2", "task-y-a3"]
        # 4 个 UI 事件 (1 update + 3 create)
        assert len(body["ui_events"]) == 4

    def test_split_endpoint_rejects_superseded_target(self):
        """IT-11-B-3: POST /api/tmo/split 拒绝 superseded 目标 (守门)"""
        app, client = _build_test_app()

        # 先 split 一次
        resp1 = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-z"},
        )
        assert resp1.status_code == 200

        # 再 split 同一个 target (已经是 superseded)
        resp2 = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-z"},
        )
        assert resp2.status_code == 400
        assert "already superseded" in resp2.text

    def test_split_endpoint_rejects_split_count_1(self):
        """IT-11-B-4: POST /api/tmo/split 拒绝 split_count=1 (Pydantic ge=2 守门)"""
        app, client = _build_test_app()

        resp = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-x", "split_count": 1},
        )
        assert resp.status_code == 422  # Pydantic validation error

    def test_split_endpoint_rejects_split_count_16(self):
        """IT-11-B-5: POST /api/tmo/split 拒绝 split_count=16 (Pydantic le=8 守门)"""
        app, client = _build_test_app()

        resp = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-x", "split_count": 16},
        )
        assert resp.status_code == 422  # Pydantic validation error

    def test_split_endpoint_rejects_invalid_strategy(self):
        """IT-11-B-6: POST /api/tmo/split 拒绝 非法 split_strategy"""
        app, client = _build_test_app()

        resp = client.post(
            "/api/tmo/split",
            json={"target_task_id": "task-x", "split_strategy": "evil_strategy"},
        )
        assert resp.status_code == 422  # Pydantic validator


# ===== IT-11-C: 守门 #13 a L0 协调实证 =====

class TestTmoSplitL0Coordination:
    """IT-11-C: 守门 #13 a L0 协调实证 — TMO split 全部经 L0, 跨 L1 task 操作只经 L0"""

    def test_routes_tmo_source_no_subagent_internal_import(self):
        """IT-11-C-1: routes_tmo.py 跟 TMO split 段没直接 import sub_agent.types

        守门 #13 a: 跨 L1 task 操作只经 L0 (TaskOperationsManager C-16),
        routes_tmo 不应绕过 L0 直接调 sub-agent 内部 API
        """
        from pathlib import Path
        routes_src = (REPO_ROOT / "scripts/automation/api/routes_tmo.py").read_text(encoding="utf-8")
        # 验证 split 段 (TMO-02) 没 import sub_agent.types
        # 简化: 整文件没有 from automation.sub_agent 引用
        assert "from automation.sub_agent" not in routes_src, (
            "routes_tmo.py 不应 from automation.sub_agent 直接 import (守门 #13 a L0 唯一入口)"
        )
        assert "sub_agent.types" not in routes_src, (
            "routes_tmo.py 不应 import sub_agent.types (守门 #13 a)"
        )

    def test_split_node_does_not_import_subagent_types(self):
        """IT-11-C-2: split_node.py 没 import sub_agent.types (守门 #13 a)"""
        from pathlib import Path
        split_src = (REPO_ROOT / "scripts/automation/task_ops/nodes/split_node.py").read_text(encoding="utf-8")
        assert "sub_agent.types" not in split_src, (
            "split_node.py 不应 import sub_agent.types (守门 #13 a L0 唯一入口)"
        )

    def test_manager_dispatch_routes_m_n2_to_split_node(self):
        """IT-11-C-3: TaskOperationsManager.dispatch M-N2 路由到 split_node"""
        from automation.task_ops.manager import TaskOperationsManager
        from automation.task_ops.nodes.split_node import split_node as split_node_fn

        manager = TaskOperationsManager()
        manager.sub_pool.add(
            task_type="SA-09",
            task_id="task-route",
            initial_state={"status": "running", "context": {}},
        )

        # 走 dispatch 路径, 验证 M-N2 路由到 split_node
        message = {
            "operation": "split",
            "target_task_id": "task-route",
        }
        result = asyncio.run(manager.dispatch(message))
        assert result["ok"] is True
        assert result["node"] == "M-N2"
        assert "new_task_ids" in result["result"]
        assert len(result["result"]["new_task_ids"]) == 2


# ===== IT-11-D: namespace 隔离 (TMO-01/03/04 端点不被破坏) =====

class TestTmoNamespaceIsolation:
    """IT-11-D: 验证 TMO-02 split 端点没破坏 TMO-01 merge / TMO-03 dep / TMO-04 bulk 端点

    跨 worktree 整合守门: per wt-tmo-04 bulk + wt-tmo-01 merge, TMO-02 split 应共存
    """

    def test_tmo_merge_endpoint_still_works(self):
        """IT-11-D-1: TMO-01 /api/tmo/merge 端点 仍跑通"""
        app, client = _build_test_app()
        resp = client.post(
            "/api/tmo/merge",
            json={"target_task_ids": ["a1", "a2"]},
        )
        assert resp.status_code == 200, resp.text
        body = resp.json()
        assert body["ok"] is True
        assert body["node"] == "M-N1"
        assert "merged_task_id" in body
        assert body["merged_task_id"] is not None

    def test_tmo_operations_endpoint_includes_m_n2(self):
        """IT-11-D-2: GET /api/tmo/operations implemented_nodes 包含 M-N2"""
        app, client = _build_test_app()
        resp = client.get("/api/tmo/operations")
        assert resp.status_code == 200
        body = resp.json()
        assert "M-N2" in body["implemented_nodes"], (
            f"implemented_nodes 应包含 M-N2, got {body['implemented_nodes']}"
        )
        assert "M-N1" in body["implemented_nodes"]
        assert "M-N3" in body["implemented_nodes"]
        assert "M-N4" in body["implemented_nodes"]
