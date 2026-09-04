# tests/integration/test_tmo_05_06_07.py
# IT-13 TMO M-N5/M-N6/M-N7 跨任务管理操作 end-to-end
# (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §8.2)
#
# 覆盖:
#   - M-N5 summarize_node  (TMO-05, mock LLM 备选 per 守门 #5+#23, 跨 N task 汇总)
#   - M-N6 reassign_node   (TMO-06, 跨 SA 类型切换)
#   - M-N7 metadata_node   (TMO-07, Master RLS + SCD Type 2 per 守门 #13 c)
#   - 守门 #13 a: TMO 全部 L0 协调 (TaskOperationsManager C-16 唯一 cross-task actor)
#   - 守门 #19: Python 化, pytest 9.x
#   - 守门 #20: 子代理 dispatch 必先 brief (TMO-05/06/07 父会话 Mavis 委托)
#
# 7 个 e2e case:
#   1. M-N5 跨多 task 汇总
#   2. M-N5 拒绝空 target_task_ids
#   3. M-N6 跨 SA 类型切换
#   4. M-N6 拒绝非法 new_task_type
#   5. M-N7 metadata 更新 + SCD snapshot
#   6. M-N7 RLS violation 拒绝
#   7. TMO dispatch 守门 #13 a 实证 (5/5 通过 manager, 不直接 sub-agent)

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

import pytest

# 在 tests/ 父级 import scripts/automation
REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.manager import (  # noqa: E402
    OPERATION_TO_NODE,
    SubAgentHandle,
    SubAgentPool,
    TaskOperationsManager,
)
from automation.task_ops.nodes.summarize_node import summarize_node  # noqa: E402
from automation.task_ops.nodes.reassign_node import reassign_node  # noqa: E402
from automation.task_ops.nodes.metadata_node import metadata_node  # noqa: E402


# ===== 辅助 fixture =====

def _make_manager_with_tasks(n: int = 3) -> TaskOperationsManager:
    """构造含 n 个 L1 task 的 manager (mock 模式).

    每个 task 都有 tenant_id=tenant-A, workspace_ids=[ws-1], 跑在不同 SA-XX type.
    """
    mgr = TaskOperationsManager()
    sa_types = ["SA-01", "SA-02", "SA-03", "SA-04", "SA-05"]
    for i in range(n):
        mgr.sub_pool.add(
            task_type=sa_types[i % len(sa_types)],
            task_id=f"t{i+1}",
            initial_state={
                "status": "running" if i % 2 == 0 else "done",
                "context": {"index": i, "description": f"task {i+1}"},
                "tenant_id": "tenant-A",
            },
        )
    return mgr


# ===== M-N5 summarize_node =====

class TestSummarizeNode:
    """IT-13-A: M-N5 summarize_node 跨任务汇总 (TMO-05)"""

    def test_op_to_node_mapping(self):
        """summarize 路由到 M-N5 (per OPERATION_TO_NODE 路由表)"""
        assert OPERATION_TO_NODE["summarize"] == "M-N5"

    @pytest.mark.asyncio
    async def test_summarize_multiple_tasks(self):
        """IT-13-A-1: M-N5 跨 N=3 task 汇总, summary + token_usage 正确返回"""
        mgr = _make_manager_with_tasks(n=3)
        result = await summarize_node(
            state={
                "operation": "summarize",
                "target_task_ids": ["t1", "t2", "t3"],
                "actor_session_id": "sess-test-1",
            },
            manager=mgr,
        )
        assert result["operation"] == "summarize"
        assert result["active_tmo_operation"] is None
        assert len(result["task_summaries"]) == 3
        for ts in result["task_summaries"]:
            assert "summary" in ts
            assert "token_usage" in ts
            assert ts["token_usage"]["total"] > 0
        # 聚合 token_usage
        assert result["total_token_usage"]["total"] >= 3  # 至少每 task 1 token
        # UI events
        assert len(result["ui_events"]) == 3
        for ev in result["ui_events"]:
            assert ev["type"] == "TaskCardUpdate"
            assert "summary" in ev["patch"]

    @pytest.mark.asyncio
    async def test_summarize_rejects_empty_targets(self):
        """IT-13-A-2: M-N5 拒绝空 target_task_ids"""
        mgr = _make_manager_with_tasks(n=1)
        with pytest.raises(ValueError, match="target_task_ids is required"):
            await summarize_node(
                state={"operation": "summarize", "target_task_ids": []},
                manager=mgr,
            )

    @pytest.mark.asyncio
    async def test_summarize_rejects_too_many_targets(self):
        """IT-13-A-3: M-N5 拒绝超过 MAX_SUMMARIZE_TARGETS=50"""
        from automation.task_ops.nodes.summarize_node import MAX_SUMMARIZE_TARGETS
        mgr = _make_manager_with_tasks(n=1)
        too_many = [f"t{i}" for i in range(MAX_SUMMARIZE_TARGETS + 1)]
        with pytest.raises(ValueError, match=f"count .* > MAX {MAX_SUMMARIZE_TARGETS}"):
            await summarize_node(
                state={"operation": "summarize", "target_task_ids": too_many},
                manager=mgr,
            )


# ===== M-N6 reassign_node =====

class TestReassignNode:
    """IT-13-B: M-N6 reassign_node 跨 SA 类型切换 (TMO-06)"""

    def test_op_to_node_mapping(self):
        assert OPERATION_TO_NODE["reassign"] == "M-N6"

    @pytest.mark.asyncio
    async def test_reassign_change_sa_type(self):
        """IT-13-B-1: M-N6 跨 SA 类型切换 (SA-01 → SA-04)"""
        mgr = _make_manager_with_tasks(n=1)
        result = await reassign_node(
            state={
                "operation": "reassign",
                "target_task_id": "t1",
                "new_task_type": "SA-04",
                "actor_session_id": "sess-test-1",
            },
            manager=mgr,
        )
        assert result["operation"] == "reassign"
        assert result["old_task_type"] == "SA-01"
        assert result["new_task_type"] == "SA-04"
        assert result["preserved_checkpoint_id"].startswith("cp-")
        # 副作用: handle.task_type 已更新
        handle = mgr.sub_pool.get("t1")
        assert handle.task_type == "SA-04"
        # 副作用: state 状态 = reassigned
        assert handle.state["status"] == "reassigned"
        assert handle.state["reassigned_from"] == "SA-01"
        assert handle.state["reassigned_to"] == "SA-04"
        # UI events
        assert len(result["ui_events"]) == 1
        assert result["ui_events"][0]["type"] == "TaskCardUpdate"

    @pytest.mark.asyncio
    async def test_reassign_rejects_invalid_sa_type(self):
        """IT-13-B-2: M-N6 拒绝非法 new_task_type"""
        mgr = _make_manager_with_tasks(n=1)
        with pytest.raises(ValueError, match="not in VALID_SA_TYPES"):
            await reassign_node(
                state={
                    "operation": "reassign",
                    "target_task_id": "t1",
                    "new_task_type": "SA-99",  # 非法
                },
                manager=mgr,
            )

    @pytest.mark.asyncio
    async def test_reassign_noop_when_same_type(self):
        """IT-13-B-3: M-N6 同类型 reassign 是 no-op (per 设计)"""
        mgr = _make_manager_with_tasks(n=1)
        result = await reassign_node(
            state={
                "operation": "reassign",
                "target_task_id": "t1",
                "new_task_type": "SA-01",  # 跟当前一致
            },
            manager=mgr,
        )
        assert result["noop"] is True
        assert result["old_task_type"] == "SA-01"
        assert result["new_task_type"] == "SA-01"


# ===== M-N7 metadata_node =====

class TestMetadataNode:
    """IT-13-C: M-N7 metadata_node Master RLS + SCD Type 2 (TMO-07)"""

    def test_op_to_node_mapping(self):
        assert OPERATION_TO_NODE["metadata"] == "M-N7"

    @pytest.mark.asyncio
    async def test_metadata_update_with_scd_snapshot(self):
        """IT-13-C-1: M-N7 metadata 更新 + SCD snapshot 永存"""
        mgr = _make_manager_with_tasks(n=1)
        # 第一次 metadata 更新 (无 prev snapshot)
        r1 = await metadata_node(
            state={
                "operation": "metadata",
                "target_task_id": "t1",
                "metadata": {"name": "Task 1", "priority": 5, "labels": ["urgent"]},
                "tenant_id": "tenant-A",
                "workspace_ids": ["ws-1"],
            },
            manager=mgr,
        )
        assert r1["updated_fields"] == ["labels", "name", "priority"]
        assert r1["scd_snapshot_id"] is None  # 第一次没 prev metadata
        # 第二次 metadata 更新 (有 prev snapshot)
        r2 = await metadata_node(
            state={
                "operation": "metadata",
                "target_task_id": "t1",
                "metadata": {"priority": 8, "notes": "second update"},
                "tenant_id": "tenant-A",
                "workspace_ids": ["ws-1"],
            },
            manager=mgr,
        )
        assert r2["updated_fields"] == ["notes", "priority"]
        assert r2["scd_snapshot_id"] is not None
        assert r2["scd_snapshot_id"].startswith("metadata-scd-")
        # 副作用: state.metadata 已合并
        handle = mgr.sub_pool.get("t1")
        assert handle.state["metadata"]["name"] == "Task 1"
        assert handle.state["metadata"]["priority"] == 8
        assert handle.state["metadata"]["notes"] == "second update"
        assert handle.state["metadata"]["labels"] == ["urgent"]
        # 副作用: scd_history 永存 (Transaction append-only)
        assert len(handle.state["metadata_scd_history"]) == 1
        assert handle.state["metadata_scd_history"][0]["snapshot_id"] == r2["scd_snapshot_id"]

    @pytest.mark.asyncio
    async def test_metadata_rls_violation_rejected(self):
        """IT-13-C-2: M-N7 RLS violation 拒绝 (per 守门 #13 c Master RLS)"""
        mgr = _make_manager_with_tasks(n=1)
        with pytest.raises(PermissionError, match="RLS check failed"):
            await metadata_node(
                state={
                    "operation": "metadata",
                    "target_task_id": "t1",
                    "metadata": {"priority": 5},
                    "tenant_id": "tenant-B",  # 跟 t1.tenant_id=tenant-A 不一致
                    "workspace_ids": ["ws-1"],
                },
                manager=mgr,
            )

    @pytest.mark.asyncio
    async def test_metadata_rejects_missing_tenant_id(self):
        """IT-13-C-3: M-N7 拒绝缺 tenant_id (per 守门 #13 c 必携)"""
        mgr = _make_manager_with_tasks(n=1)
        with pytest.raises(ValueError, match="tenant_id is required"):
            await metadata_node(
                state={
                    "operation": "metadata",
                    "target_task_id": "t1",
                    "metadata": {"priority": 5},
                    "tenant_id": None,
                    "workspace_ids": ["ws-1"],
                },
                manager=mgr,
            )

    @pytest.mark.asyncio
    async def test_metadata_rejects_unknown_field(self):
        """IT-13-C-4: M-N7 拒绝未知字段 (status / task_type 必须通过专属 node 改)"""
        mgr = _make_manager_with_tasks(n=1)
        with pytest.raises(ValueError, match="unknown metadata fields"):
            await metadata_node(
                state={
                    "operation": "metadata",
                    "target_task_id": "t1",
                    "metadata": {"status": "done", "name": "x"},  # status 非法
                    "tenant_id": "tenant-A",
                    "workspace_ids": ["ws-1"],
                },
                manager=mgr,
            )


# ===== 守门 #13 a 实证 =====

class TestTmoL0Coordination:
    """IT-13-D: 守门 #13 a 实证 — TMO 5/5 节点经 manager.dispatch 走 L0 协调

    per 02-basic-design.md §2.6: TaskOperationsManager C-16 唯一 cross-task actor.
    """

    @pytest.mark.asyncio
    async def test_all_seven_ops_route_to_l0_manager(self):
        """IT-13-D-1: 7 节点路由表完整 (M-N1..M-N7)"""
        expected = {
            "merge": "M-N1",
            "split": "M-N2",
            "dep_set": "M-N3",
            "bulk_action": "M-N4",
            "summarize": "M-N5",
            "reassign": "M-N6",
            "metadata": "M-N7",
        }
        for op, node in expected.items():
            assert OPERATION_TO_NODE[op] == node, f"op={op} -> node={node} mismatch"

    @pytest.mark.asyncio
    async def test_dispatch_summarize_reassign_metadata_via_manager(self):
        """IT-13-D-2: M-N5/M-N6/M-N7 经 manager.dispatch 走 L0 协调 (守门 #13 a)"""
        mgr = _make_manager_with_tasks(n=3)
        # M-N5
        r1 = await mgr.dispatch({
            "operation": "summarize",
            "target_task_ids": ["t1", "t2"],
            "actor_session_id": "sess-test-1",
        })
        assert r1["ok"] is True
        assert r1["node"] == "M-N5"
        # M-N6
        r2 = await mgr.dispatch({
            "operation": "reassign",
            "target_task_id": "t1",
            "new_task_type": "SA-05",
        })
        assert r2["ok"] is True
        assert r2["node"] == "M-N6"
        # M-N7
        r3 = await mgr.dispatch({
            "operation": "metadata",
            "target_task_id": "t1",
            "metadata": {"priority": 9},
            "tenant_id": "tenant-A",
            "workspace_ids": ["ws-1"],
        })
        assert r3["ok"] is True
        assert r3["node"] == "M-N7"
        # audit log 应有 3 条
        assert len(mgr.audit_log) >= 3
