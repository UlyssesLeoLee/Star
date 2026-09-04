# tests/integration/test_tmo_merge.py
# IT-10 TMO merge end-to-end (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §8.2)
#
# 集成测试范围: merge_node + SA-10 + TaskOperationsManager + FastAPI 端点整合 (per UC-09)
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调
#   - 守门 #19: Python 化
#   - 守门 #20: 子代理 dispatch 必先 brief (TMO-01 brief: docs/briefs/tmo-2026-09-04-parallel.md)
#   - 守门 #22: 调试控制台不污染 main
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 → subprocess

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.manager import TaskOperationsManager  # noqa: E402
from automation.task_ops.nodes.merge_node import merge_node  # noqa: E402
from automation.sub_agent.types.sa_10_task_orchestrator import SA10TaskOrchestrator  # noqa: E402


# ===== IT-10-A: merge_node + SA-10 整合 =====

@pytest.mark.asyncio
async def test_merge_with_sa10_orchestrator_full_flow():
    """IT-10-A: 完整 merge_node + SA-10 task-orchestrator 整合 (per UC-09)"""
    manager = TaskOperationsManager()
    # 准备 2 个 L1 task (a / b)
    manager.sub_pool.add(
        task_type="SA-09",
        task_id="task-alpha",
        initial_state={"status": "running", "context": {"description": "task alpha description"}},
    )
    manager.sub_pool.add(
        task_type="SA-09",
        task_id="task-beta",
        initial_state={"status": "running", "context": {"description": "task beta description"}},
    )

    # 模拟用户 chat bar: "合并任务 alpha 和任务 beta"
    state = {
        "operation": "merge",
        "target_task_ids": ["task-alpha", "task-beta"],
        "merge_strategy": "context_union",
        "original_user_input": "合并任务 alpha 和任务 beta",
        "actor_session_id": "session-uc09-001",
    }
    result = await merge_node(state=state, manager=manager)

    # 整合断言:
    # 1. SA-10 5 节点 subgraph 都跑通 (per 02 §2.2.3)
    #    通过 merged_task_id 找到 SA-10 实例 (mock 模式: merge_node 返回的 merged_task_id = SA10TaskOrchestrator.merged_task_id)
    assert result["merged_task_id"].startswith("merged-") or result["merged_task_id"].startswith("sa10-")
    # 2. 源 task a / b 都标 superseded (per 守门 #13 d)
    assert manager.sub_pool.get("task-alpha").state["status"] == "superseded"
    assert manager.sub_pool.get("task-beta").state["status"] == "superseded"
    # 3. stash 2 个 checkpoint 都被记录 (Transaction append-only)
    assert len(manager.sub_pool.get("task-alpha").checkpoints) == 1
    assert len(manager.sub_pool.get("task-beta").checkpoints) == 1
    # 4. UI events 3 个 (2 update + 1 create)
    assert len(result["ui_events"]) == 3
    # 5. TMO operation done
    assert result["active_tmo_operation"] is None
    # 6. global_context.last_tmo_result 完整
    last = result["global_context"]["last_tmo_result"]
    assert last["operation"] == "merge"
    assert last["merged_task_id"] == result["merged_task_id"]
    assert set(last["superseded_task_ids"]) == {"task-alpha", "task-beta"}
    assert last["merge_strategy"] == "context_union"


# ===== IT-10-B: SA-10 5 节点 subgraph 独立跑通 =====

@pytest.mark.asyncio
async def test_sa10_subgraph_5_nodes():
    """IT-10-B: SA-10 task-orchestrator 5 节点 (init → plan → execute → verify → report) 独立跑通"""
    orchestrator = SA10TaskOrchestrator(merged_task_id="sa10-test-001")

    # 1. init
    init_result = await orchestrator.init_node(operation="merge", merged_from=["a", "b"])
    assert init_result["status"] == "init_done"
    assert init_result["operation"] == "merge"

    # 2. plan
    plan_result = await orchestrator.plan_node(merged_from=["a", "b"], merged_state=["cp-1"], merge_strategy="context_union")
    assert plan_result["status"] == "plan_done"
    assert plan_result["plan"]["operation"] == "merge"
    assert plan_result["plan"]["merged_from"] == ["a", "b"]
    assert len(plan_result["plan"]["steps"]) == 4

    # 3. execute
    exec_result = await orchestrator.execute_node()
    assert exec_result["status"] == "execute_done"
    assert exec_result["result"]["merged_task_id"] == "sa10-test-001"
    assert exec_result["result"]["merged_context"]["merge_strategy"] == "context_union"

    # 4. verify
    verify_result = await orchestrator.verify_node()
    assert verify_result["status"] == "verify_done"
    assert verify_result["verify"]["ok"] is True
    assert verify_result["verify"]["violations"] == []
    # 守门检查清单
    assert "守门 #13 a (L0 唯一协调)" in verify_result["verify"]["checked"]
    assert "守门 #13 d (Transaction append-only)" in verify_result["verify"]["checked"]

    # 5. report
    report_result = await orchestrator.report_node()
    assert report_result["status"] == "report_done"
    assert report_result["report"]["merged_task_id"] == "sa10-test-001"
    assert report_result["report"]["verify_ok"] is True
    # report 节点自身也 audit 一次, 所以跑完后总 audit_count = 5
    # (报告内 audit_count 字段是 report 自身 audit 之前的 count, 所以 >= 4)
    assert report_result["report"]["audit_count"] >= 4
    assert len(orchestrator.state.audit_log) >= 5  # 实际跑完后 5 个节点都 audit 了


@pytest.mark.asyncio
async def test_sa10_subgraph_run_helper():
    """IT-10-B: SA-10 完整 run() helper 跑通 (5 节点 + verify_ok 检查)"""
    orchestrator = SA10TaskOrchestrator()
    merged_id = await orchestrator.run(
        operation="merge",
        merged_from=["x", "y"],
        merged_state=["cp-x", "cp-y"],
        merge_strategy="context_union",
        original_user_input="合并任务 x 和任务 y",
    )
    assert merged_id == orchestrator.merged_task_id
    assert orchestrator.state.status == "done"
    assert orchestrator.state.verify_result["ok"] is True


@pytest.mark.asyncio
async def test_sa10_subgraph_run_raises_on_verify_fail():
    """IT-10-B: SA-10 verify 失败时 run() raise (空 merged_from)"""
    orchestrator = SA10TaskOrchestrator()
    with pytest.raises(ValueError, match="verify failed"):
        await orchestrator.run(operation="merge", merged_from=[], merged_state=[])


# ===== IT-10-C: TaskOperationsManager 路由 + dispatch =====

def test_tmo_manager_route():
    """IT-10-C: TaskOperationsManager.route 路由 (per 02 §2.6.3 路由表)"""
    manager = TaskOperationsManager()
    assert manager.route({"operation": "merge"}) == "M-N1"
    assert manager.route({"operation": "split"}) == "M-N2"
    assert manager.route({"operation": "dep_set"}) == "M-N3"
    assert manager.route({"operation": "bulk_action"}) == "M-N4"
    assert manager.route({"operation": "summarize"}) == "M-N5"
    assert manager.route({"operation": "reassign"}) == "M-N6"
    assert manager.route({"operation": "metadata"}) == "M-N7"


def test_tmo_manager_route_rejects_unknown():
    """IT-10-C: TaskOperationsManager.route 拒绝未知 operation"""
    manager = TaskOperationsManager()
    with pytest.raises(ValueError, match="unknown TMO operation"):
        manager.route({"operation": "unknown_op"})


@pytest.mark.asyncio
async def test_tmo_manager_dispatch_merge():
    """IT-10-C: TaskOperationsManager.dispatch 调 M-N1 merge_node"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="m-1")
    manager.sub_pool.add(task_type="SA-09", task_id="m-2")

    message = {
        "operation": "merge",
        "target_task_ids": ["m-1", "m-2"],
        "merge_strategy": "context_union",
    }
    result = await manager.dispatch(message)
    assert result["ok"] is True
    assert result["node"] == "M-N1"
    assert "merged_task_id" in result["result"]
    # audit 落档
    assert any(a["operation"] == "merge" for a in manager.audit_log)


@pytest.mark.asyncio
async def test_tmo_manager_dispatch_unknown_node_fails():
    """IT-10-C: TaskOperationsManager.dispatch 对未实装节点返 ok=False"""
    manager = TaskOperationsManager()
    message = {"operation": "split", "target_task_id": "x"}  # M-N2 planned, not implemented
    result = await manager.dispatch(message)
    assert result["ok"] is False
    assert "not yet implemented" in result["error"]


# ===== IT-10-D: 守门 #13 a 实证 (L0 唯一入口) =====

@pytest.mark.asyncio
async def test_l0_only_coordination_no_l1_to_l1():
    """IT-10-D: 实证 守门 #13 a L0 唯一协调, L1↔L1 禁止通信

    验证:
      - merge_node 只经 L0 (TaskOperationsManager)
      - L1 sub-agent handle 互不可见 (没有 L1↔L1 通信通道)
      - 所有跨 task 操作 (validate / stash / dispatch / supersede) 都经 sub_pool (L0 唯一入口)
    """
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="l1-a")
    manager.sub_pool.add(task_type="SA-09", task_id="l1-b")

    # merge_node 内部全部走 sub_pool (L0 唯一入口 per 守门 #13 a)
    # 验证: L1 handle._handles 是私有, 外部访问只通过 sub_pool.get (L0 入口)
    assert hasattr(manager.sub_pool, "_handles")
    # _handles 命名以下划线开头 (Python 私有约定) → 外部不应直接访问
    # 所有 sub-agent 操作必经过 sub_pool (L0 入口)
    state = {"operation": "merge", "target_task_ids": ["l1-a", "l1-b"]}
    result = await merge_node(state=state, manager=manager)

    # 操作结果证明 L0 协调成功
    assert result["merged_task_id"]
    assert manager.sub_pool.get("l1-a").state["status"] == "superseded"
    assert manager.sub_pool.get("l1-b").state["status"] == "superseded"
    # L1↔L1 通信未发生 (因为 merge_node 只用 sub_pool, 不直接读其他 L1 handle)
    # 间接验证: 合并后 l1-a 和 l1-b 状态都是 superseded (L0 协调结果一致)
    assert manager.sub_pool.get("l1-a").state["superseded_by"] == manager.sub_pool.get("l1-b").state["superseded_by"]


# ===== IT-10-E: 守门 #13 d Transaction append-only 实证 =====

@pytest.mark.asyncio
async def test_transaction_append_only():
    """IT-10-E: 实证 守门 #13 d Transaction append-only

    验证:
      - stash_state 创建的 checkpoint 永远不被删除
      - merge 多次跑同一对 task 不会清空历史
    """
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="t-1")
    manager.sub_pool.add(task_type="SA-09", task_id="t-2")

    # 第 1 次 merge
    await merge_node(state={"operation": "merge", "target_task_ids": ["t-1", "t-2"]}, manager=manager)
    cp_count_after_1 = len(manager.sub_pool.get("t-1").checkpoints)
    assert cp_count_after_1 == 1

    # 第 2 次 merge (会失败因为 t-1/t-2 都已 superseded)
    with pytest.raises(ValueError, match="already superseded"):
        await merge_node(state={"operation": "merge", "target_task_ids": ["t-1", "t-2"]}, manager=manager)
    # 守门 #13 d: 上次 merge 的 checkpoint 还在 (没被清空)
    assert len(manager.sub_pool.get("t-1").checkpoints) == cp_count_after_1
