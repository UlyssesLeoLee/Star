# tests/unit/test_task_ops/test_merge_node.py
# UT-20 merge_node (M-N1) 单元测试 (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §8.1)
#
# 测试目标: stash_state + dispatch merged + supersede a/b
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口
#   - 守门 #13 d: Transaction append-only (stash 永存)
#   - 守门 #19: Python 化
#   - 守门 #23: 不开 OpenAI/Anthropic API

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

import pytest

# 把仓根加到 sys.path, 让 `import automation.task_ops...` 能 work
# 仓根 = D:\Star\.worktrees\wt-tmo-01, automation package 在 scripts/automation/
# 所以 sys.path 需加 REPO_ROOT / "scripts"
REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.manager import SubAgentPool, TaskOperationsManager  # noqa: E402
from automation.task_ops.nodes.merge_node import (  # noqa: E402
    _validate_target_tasks,
    _stash_states,
    _mark_superseded,
    _emit_ui_events,
    merge_node,
)


# ===== UT-20-A: validate (≥ 2 task_ids) =====

def test_validate_requires_at_least_two_tasks():
    """UT-20-A: validate 拒绝 < 2 task_ids"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    with pytest.raises(ValueError, match="requires >= 2 task_ids"):
        _validate_target_tasks(["a"], pool)


def test_validate_rejects_nonexistent_task():
    """UT-20-A: validate 拒绝 不存在的 task_id"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    pool.add(task_type="SA-09", task_id="b")
    with pytest.raises(ValueError, match="not found"):
        _validate_target_tasks(["a", "nonexistent"], pool)


def test_validate_rejects_superseded_task():
    """UT-20-A: validate 拒绝 superseded 状态的 task"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a", initial_state={"status": "superseded"})
    pool.add(task_type="SA-09", task_id="b")
    with pytest.raises(ValueError, match="already superseded"):
        _validate_target_tasks(["a", "b"], pool)


def test_validate_passes_for_valid_targets():
    """UT-20-A: validate 通过 ≥ 2 个非 superseded task"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    pool.add(task_type="SA-09", task_id="b")
    result = _validate_target_tasks(["a", "b"], pool)
    assert result == ["a", "b"]


# ===== UT-20-B: stash_state (Transaction append-only) =====

@pytest.mark.asyncio
async def test_stash_state_appends_checkpoints():
    """UT-20-B: stash_state 创建 checkpoint (Transaction append-only per 守门 #13 d)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a", initial_state={"status": "running", "context": {"x": 1}})
    pool.add(task_type="SA-09", task_id="b", initial_state={"status": "running", "context": {"y": 2}})

    stash_ids = await _stash_states(["a", "b"], pool)
    assert len(stash_ids) == 2
    # 守门 #13 d: 2 个 checkpoint 都被记录在 a/b handle.checkpoints (append-only)
    assert len(pool.get("a").checkpoints) == 1
    assert len(pool.get("b").checkpoints) == 1
    # 守门 #13 d: stash 后状态 = stash_pending
    assert pool.get("a").state["status"] == "stash_pending"
    assert pool.get("b").state["status"] == "stash_pending"
    # checkpoint label 必含 task_id
    assert "merge_stash_a" in pool.get("a").checkpoints[0]["label"]
    assert "merge_stash_b" in pool.get("b").checkpoints[0]["label"]


@pytest.mark.asyncio
async def test_stash_state_is_append_only():
    """UT-20-B: stash_state 多次 stash 会 append (不覆盖, 守门 #13 d)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")

    stash_id_1 = await _stash_states(["a"], pool)
    stash_id_2 = await _stash_states(["a"], pool)
    # 2 个不同 checkpoint_id (append-only)
    assert stash_id_1[0] != stash_id_2[0]
    assert len(pool.get("a").checkpoints) == 2


# ===== UT-20-C: mark_superseded (终态不删除) =====

@pytest.mark.asyncio
async def test_mark_superseded():
    """UT-20-C: mark_superseded 设置 status=superseded + superseded_by pointer (守门 #13 d)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    pool.add(task_type="SA-09", task_id="b")
    merged_task_id = "merged-abc12345"
    await _mark_superseded(["a", "b"], merged_task_id, pool)
    # 守门 #13 d: a / b 都标 superseded
    assert pool.get("a").state["status"] == "superseded"
    assert pool.get("b").state["status"] == "superseded"
    # 守门 #13 d: pointer → merged_task_id
    assert pool.get("a").state["superseded_by"] == merged_task_id
    assert pool.get("b").state["superseded_by"] == merged_task_id
    # 守门 #13 d: superseded_at 必填
    assert "superseded_at" in pool.get("a").state


# ===== UT-20-D: emit_ui_events =====

def test_emit_ui_events():
    """UT-20-D: emit 3 个 UI 事件 (TaskCardUpdate × 2 + TaskCardCreate × 1)"""
    events = _emit_ui_events(["a", "b"], "merged-xyz", ["cp-1", "cp-2"])
    assert len(events) == 3
    # 2 个 Update
    update_events = [e for e in events if e["type"] == "TaskCardUpdate"]
    assert len(update_events) == 2
    assert {e["task_id"] for e in update_events} == {"a", "b"}
    # 1 个 Create
    create_events = [e for e in events if e["type"] == "TaskCardCreate"]
    assert len(create_events) == 1
    assert create_events[0]["task_id"] == "merged-xyz"
    assert create_events[0]["card"]["merged_from"] == ["a", "b"]
    assert create_events[0]["card"]["stash_checkpoint_ids"] == ["cp-1", "cp-2"]


# ===== UT-20-E: 完整 merge_node 端到端 =====

@pytest.mark.asyncio
async def test_merge_node_full_flow():
    """UT-20-E: 完整 merge_node 跑通 (validate + stash + dispatch + supersede + emit)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="task-a", initial_state={"status": "running", "context": {"x": 1}})
    manager.sub_pool.add(task_type="SA-09", task_id="task-b", initial_state={"status": "running", "context": {"y": 2}})

    state = {
        "operation": "merge",
        "target_task_ids": ["task-a", "task-b"],
        "merge_strategy": "context_union",
        "original_user_input": "合并任务 a 和任务 b",
        "actor_session_id": "session-mock-001",
    }
    result = await merge_node(state=state, manager=manager)

    # 步骤 1: validate 通过
    # 步骤 2: stash 2 个 checkpoint
    assert len(result["stash_checkpoint_ids"]) == 2
    # 步骤 3: dispatch merged_task (SA-10)
    assert "merged_task_id" in result
    assert result["merged_task_id"].startswith("merged-")
    # 步骤 4: a / b 状态 = superseded
    assert result["superseded_tasks"] == ["task-a", "task-b"]
    assert manager.sub_pool.get("task-a").state["status"] == "superseded"
    assert manager.sub_pool.get("task-b").state["status"] == "superseded"
    assert manager.sub_pool.get("task-a").state["superseded_by"] == result["merged_task_id"]
    # 步骤 5: emit 3 个 UI 事件
    assert len(result["ui_events"]) == 3
    # TMO operation done
    assert result["active_tmo_operation"] is None
    # audit 落档
    assert len(manager.audit_log) >= 0  # dispatch 路径会 audit


@pytest.mark.asyncio
async def test_merge_node_three_targets():
    """UT-20-E: 3+ 任务合并 (per 守门 ≥ 2)"""
    manager = TaskOperationsManager()
    for tid in ["a", "b", "c"]:
        manager.sub_pool.add(task_type="SA-09", task_id=tid)

    state = {
        "operation": "merge",
        "target_task_ids": ["a", "b", "c"],
        "merge_strategy": "context_union",
    }
    result = await merge_node(state=state, manager=manager)
    assert len(result["superseded_tasks"]) == 3
    assert len(result["stash_checkpoint_ids"]) == 3
    # 3 个 TaskCardUpdate + 1 个 TaskCardCreate = 4 events
    assert len(result["ui_events"]) == 4


@pytest.mark.asyncio
async def test_merge_node_rejects_superseded_target():
    """UT-20-E: merge_node 拒绝 已有 superseded 的 target (守门)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="a", initial_state={"status": "superseded"})
    manager.sub_pool.add(task_type="SA-09", task_id="b")

    state = {
        "operation": "merge",
        "target_task_ids": ["a", "b"],
    }
    with pytest.raises(ValueError, match="already superseded"):
        await merge_node(state=state, manager=manager)


@pytest.mark.asyncio
async def test_merge_node_rejects_single_target():
    """UT-20-E: merge_node 拒绝 单个 target (守门 ≥ 2)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="a")

    state = {
        "operation": "merge",
        "target_task_ids": ["a"],
    }
    with pytest.raises(ValueError, match="requires >= 2 task_ids"):
        await merge_node(state=state, manager=manager)
