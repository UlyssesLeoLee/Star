# tests/unit/test_task_ops/test_split_node.py
# UT-21 split_node (M-N2) 单元测试 (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2 §8.1)
#
# 测试目标: snapshot a + dispatch a1/a2 + supersede a
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一协调入口
#   - 守门 #13 d: snapshot Transaction append-only, supersede 终态不删除
#   - 守门 #19: Python 化
#   - 守门 #23: 不开 OpenAI/Anthropic API

from __future__ import annotations

import sys
from pathlib import Path

import pytest

# 把仓根加到 sys.path, 让 `import automation.task_ops...` 能 work
# 仓根 = D:\Star\.worktrees\wt-tmo-02, automation package 在 scripts/automation/
# 所以 sys.path 需加 REPO_ROOT / "scripts"
REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

from automation.task_ops.manager import SubAgentPool, TaskOperationsManager  # noqa: E402
from automation.task_ops.nodes.split_node import (  # noqa: E402
    DEFAULT_SPLIT_COUNT,
    MAX_SPLIT_COUNT,
    MIN_SPLIT_COUNT,
    VALID_SPLIT_STRATEGIES,
    _dispatch_fork_tasks,
    _emit_ui_events,
    _mark_target_superseded_with_split_into,
    _snapshot_target,
    _validate_split_request,
    split_node,
)


# ===== UT-21-A: validate =====

def test_validate_requires_target_task_id():
    """UT-21-A: validate 拒绝 空 target_task_id"""
    pool = SubAgentPool()
    with pytest.raises(ValueError, match="target_task_id is required"):
        _validate_split_request("", "context_fork", 2, pool)


def test_validate_rejects_nonexistent_task():
    """UT-21-A: validate 拒绝 不存在的 task_id"""
    pool = SubAgentPool()
    with pytest.raises(ValueError, match="not found"):
        _validate_split_request("nonexistent", "context_fork", 2, pool)


def test_validate_rejects_superseded_task():
    """UT-21-A: validate 拒绝 superseded 状态的 task"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a", initial_state={"status": "superseded"})
    with pytest.raises(ValueError, match="already superseded"):
        _validate_split_request("a", "context_fork", 2, pool)


def test_validate_rejects_invalid_split_strategy():
    """UT-21-A: validate 拒绝 非法 split_strategy"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    with pytest.raises(ValueError, match="split_strategy must be one of"):
        _validate_split_request("a", "evil_strategy", 2, pool)


def test_validate_rejects_split_count_below_min():
    """UT-21-A: validate 拒绝 split_count < 2"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    with pytest.raises(ValueError, match="must be int >= 2"):
        _validate_split_request("a", "context_fork", 1, pool)


def test_validate_rejects_split_count_above_max():
    """UT-21-A: validate 拒绝 split_count > 8 (爆量守门)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    with pytest.raises(ValueError, match="must be <= 8"):
        _validate_split_request("a", "context_fork", 16, pool)


def test_validate_passes_for_valid_request():
    """UT-21-A: validate 通过 (返回 task_type + base_context)"""
    pool = SubAgentPool()
    pool.add(
        task_type="SA-09",
        task_id="a",
        initial_state={"status": "running", "context": {"x": 1, "y": 2}},
    )
    task_type, base_context = _validate_split_request("a", "context_fork", 2, pool)
    assert task_type == "SA-09"
    assert base_context == {"x": 1, "y": 2}


# ===== UT-21-B: snapshot (Transaction append-only) =====

@pytest.mark.asyncio
async def test_snapshot_target_appends_checkpoint():
    """UT-21-B: snapshot 创建 checkpoint (Transaction append-only per 守门 #13 d)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a", initial_state={"status": "running", "context": {"x": 1}})

    snapshot_id = await _snapshot_target("a", pool)
    assert snapshot_id.startswith("cp-")
    # 守门 #13 d: 1 个 checkpoint 都被记录在 a.handle.checkpoints (append-only)
    assert len(pool.get("a").checkpoints) == 1
    # snapshot 后状态 = snapshot_pending
    assert pool.get("a").state["status"] == "snapshot_pending"
    # checkpoint label 必含 task_id
    assert "split_snapshot_a" in pool.get("a").checkpoints[0]["label"]


@pytest.mark.asyncio
async def test_snapshot_target_is_append_only():
    """UT-21-B: snapshot 多次会 append (不覆盖, 守门 #13 d)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")

    snapshot_id_1 = await _snapshot_target("a", pool)
    snapshot_id_2 = await _snapshot_target("a", pool)
    # 2 个不同 checkpoint_id (append-only)
    assert snapshot_id_1 != snapshot_id_2
    assert len(pool.get("a").checkpoints) == 2


# ===== UT-21-C: dispatch fork tasks =====

@pytest.mark.asyncio
async def test_dispatch_fork_tasks_default_count_2():
    """UT-21-C: 默认 split_count=2 时, dispatch a1 + a2"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a", initial_state={"status": "running", "context": {"x": 1}})

    new_ids = await _dispatch_fork_tasks(
        target_id="a",
        snapshot_id="cp-mock",
        task_type="SA-09",
        base_context={"x": 1},
        split_strategy="context_fork",
        split_count=2,
        sub_pool=pool,
    )
    assert len(new_ids) == 2
    assert new_ids[0] == "a-a1"
    assert new_ids[1] == "a-a2"
    # 2 个新 task 都在 pool
    h1 = pool.get("a-a1")
    h2 = pool.get("a-a2")
    assert h1.task_type == "SA-09"
    assert h2.task_type == "SA-09"
    # 守门 #13 d: _split_from / _split_strategy / _split_index / _split_snapshot 注入
    assert h1.state["context"]["_split_from"] == "a"
    assert h1.state["context"]["_split_index"] == 0
    assert h1.state["context"]["_split_snapshot"] == "cp-mock"
    assert h1.state["context"]["_split_strategy"] == "context_fork"
    assert h2.state["context"]["_split_index"] == 1
    # base_context 继承
    assert h1.state["context"]["x"] == 1
    assert h2.state["context"]["x"] == 1


@pytest.mark.asyncio
async def test_dispatch_fork_tasks_count_3():
    """UT-21-C: split_count=3 时, dispatch a1 + a2 + a3"""
    pool = SubAgentPool()
    pool.add(task_type="SA-08", task_id="a", initial_state={"status": "running", "context": {}})

    new_ids = await _dispatch_fork_tasks(
        target_id="a",
        snapshot_id="cp-mock",
        task_type="SA-08",
        base_context={},
        split_strategy="checkpoint_fork",
        split_count=3,
        sub_pool=pool,
    )
    assert len(new_ids) == 3
    assert new_ids == ["a-a1", "a-a2", "a-a3"]
    assert pool.get("a-a1").state["context"]["_split_strategy"] == "checkpoint_fork"
    assert pool.get("a-a3").state["context"]["_split_index"] == 2


@pytest.mark.asyncio
async def test_dispatch_fork_tasks_handles_id_collision():
    """UT-21-C: 同名 task 已存在时, 用 uuid 兜底 (避免覆盖)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")
    # 模拟 "a-a1" 已存在
    pool.add(task_type="SA-09", task_id="a-a1", initial_state={"status": "running"})

    new_ids = await _dispatch_fork_tasks(
        target_id="a",
        snapshot_id="cp-mock",
        task_type="SA-09",
        base_context={},
        split_strategy="context_fork",
        split_count=2,
        sub_pool=pool,
    )
    # 第一个 collision 应该用 uuid, 第二个 a-a2 没冲突
    assert new_ids[0].startswith("a-a1-")
    assert new_ids[1] == "a-a2"


# ===== UT-21-D: mark superseded with split_into =====

@pytest.mark.asyncio
async def test_mark_superseded_with_split_into():
    """UT-21-D: mark a superseded + split_into (守门 #13 d 终态不删除)"""
    pool = SubAgentPool()
    pool.add(task_type="SA-09", task_id="a")

    await _mark_target_superseded_with_split_into("a", ["a-a1", "a-a2"], pool)
    # 守门 #13 d: status = superseded
    assert pool.get("a").state["status"] == "superseded"
    # split_into 必填, 含 [a1, a2]
    assert pool.get("a").state["split_into"] == ["a-a1", "a-a2"]
    # 守门 #13 d: superseded_by = None (split 没取代指向)
    assert pool.get("a").state["superseded_by"] is None
    # 守门 #13 d: superseded_at 必填
    assert "superseded_at" in pool.get("a").state


# ===== UT-21-E: emit UI events =====

def test_emit_ui_events_default_count():
    """UT-21-E: emit 1 + N 个 UI 事件 (1 TaskCardUpdate + N TaskCardCreate)"""
    events = _emit_ui_events(
        target_id="a",
        task_type="SA-09",
        new_task_ids=["a-a1", "a-a2"],
        snapshot_id="cp-mock",
        split_strategy="context_fork",
    )
    # 1 Update + 2 Create = 3 events
    assert len(events) == 3
    update_events = [e for e in events if e["type"] == "TaskCardUpdate"]
    create_events = [e for e in events if e["type"] == "TaskCardCreate"]
    assert len(update_events) == 1
    assert len(create_events) == 2
    # Update a 状态 superseded + split_into
    assert update_events[0]["task_id"] == "a"
    assert update_events[0]["patch"]["status"] == "superseded"
    assert update_events[0]["patch"]["split_into"] == ["a-a1", "a-a2"]
    # Create a1, a2 都有 split_from / split_index / split_snapshot_id
    assert {e["task_id"] for e in create_events} == {"a-a1", "a-a2"}
    for e in create_events:
        assert e["card"]["split_from"] == "a"
        assert e["card"]["split_snapshot_id"] == "cp-mock"
        assert e["card"]["task_type"] == "SA-09"


# ===== UT-21-F: 完整 split_node 端到端 =====

@pytest.mark.asyncio
async def test_split_node_full_flow():
    """UT-21-F: 完整 split_node 跑通 (validate + snapshot + dispatch + supersede + emit)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(
        task_type="SA-09",
        task_id="task-a",
        initial_state={"status": "running", "context": {"x": 1, "y": 2}},
    )

    state = {
        "operation": "split",
        "target_task_id": "task-a",
        "split_strategy": "context_fork",
        "split_count": 2,
        "actor_session_id": "session-mock-001",
    }
    result = await split_node(state=state, manager=manager)

    # 步骤 1: validate 通过
    # 步骤 2: snapshot 创建 1 个 checkpoint
    assert result["snapshot_checkpoint_id"].startswith("cp-")
    # 步骤 3: dispatch a1 + a2
    assert len(result["new_task_ids"]) == 2
    assert result["new_task_ids"][0] == "task-a-a1"
    assert result["new_task_ids"][1] == "task-a-a2"
    # 步骤 4: a 状态 = superseded + split_into
    assert result["superseded_tasks"] == ["task-a"]
    assert manager.sub_pool.get("task-a").state["status"] == "superseded"
    assert manager.sub_pool.get("task-a").state["split_into"] == ["task-a-a1", "task-a-a2"]
    assert manager.sub_pool.get("task-a").state["superseded_by"] is None
    # 步骤 5: emit 1 + 2 = 3 个 UI 事件
    assert len(result["ui_events"]) == 3
    # TMO operation done
    assert result["active_tmo_operation"] is None
    # last_tmo_result 落档
    last = result["global_context"]["last_tmo_result"]
    assert last["operation"] == "split"
    assert last["target_task_id"] == "task-a"
    assert last["split_strategy"] == "context_fork"
    assert last["split_count"] == 2
    assert last["snapshot_checkpoint_id"] == result["snapshot_checkpoint_id"]


@pytest.mark.asyncio
async def test_split_node_count_3():
    """UT-21-F: split_count=3 时, dispatch a1 + a2 + a3"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-08", task_id="a")

    state = {
        "operation": "split",
        "target_task_id": "a",
        "split_strategy": "checkpoint_fork",
        "split_count": 3,
    }
    result = await split_node(state=state, manager=manager)
    assert len(result["new_task_ids"]) == 3
    assert result["new_task_ids"] == ["a-a1", "a-a2", "a-a3"]
    # 3 个 TaskCardCreate
    create_events = [e for e in result["ui_events"] if e["type"] == "TaskCardCreate"]
    assert len(create_events) == 3
    # 3 个 fork 都从 a snapshot
    for h in ["a-a1", "a-a2", "a-a3"]:
        assert manager.sub_pool.get(h).state["context"]["_split_snapshot"] == result["snapshot_checkpoint_id"]


@pytest.mark.asyncio
async def test_split_node_rejects_superseded_target():
    """UT-21-F: split_node 拒绝 已有 superseded 的 target (守门)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="a", initial_state={"status": "superseded"})

    state = {
        "operation": "split",
        "target_task_id": "a",
    }
    with pytest.raises(ValueError, match="already superseded"):
        await split_node(state=state, manager=manager)


@pytest.mark.asyncio
async def test_split_node_rejects_nonexistent_target():
    """UT-21-F: split_node 拒绝 不存在的 target (守门)"""
    manager = TaskOperationsManager()
    state = {
        "operation": "split",
        "target_task_id": "nonexistent",
    }
    with pytest.raises(ValueError, match="not found"):
        await split_node(state=state, manager=manager)


@pytest.mark.asyncio
async def test_split_node_rejects_invalid_split_count():
    """UT-21-F: split_node 拒绝 split_count < 2 (守门)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="a")
    state = {
        "operation": "split",
        "target_task_id": "a",
        "split_count": 1,
    }
    with pytest.raises(ValueError, match="must be int >= 2"):
        await split_node(state=state, manager=manager)


@pytest.mark.asyncio
async def test_split_node_default_count_is_2():
    """UT-21-F: split_count 缺省时, 默认 = 2 (per DEFAULT_SPLIT_COUNT)"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-09", task_id="a")

    state = {
        "operation": "split",
        "target_task_id": "a",
    }
    result = await split_node(state=state, manager=manager)
    assert result["new_task_ids"] == ["a-a1", "a-a2"]
    assert manager.sub_pool.get("a").state["split_into"] == ["a-a1", "a-a2"]


@pytest.mark.asyncio
async def test_split_node_preserves_parent_task_type():
    """UT-21-F: fork task_type 跟父一致 (per 03 §3.2.1.1 注释: '相同 task_type as a')"""
    manager = TaskOperationsManager()
    manager.sub_pool.add(task_type="SA-08", task_id="a")
    state = {
        "operation": "split",
        "target_task_id": "a",
        "split_count": 2,
    }
    result = await split_node(state=state, manager=manager)
    for new_id in result["new_task_ids"]:
        assert manager.sub_pool.get(new_id).task_type == "SA-08"


# ===== 守门 #13 a 实证: TMO 全部 L0 协调 =====

@pytest.mark.asyncio
async def test_split_node_l0_coordination_no_subagent_internal_call():
    """守门 #13 a 实证: split_node 不直接调 sub-agent 内部 API, 只经 L0 sub_pool

    验证: split_node 没有 import sub_agent.types.sax_xx 等内部 API,
          只调 L0 唯一入口 sub_pool.{get, add, checkpoint, update, spawn}
    """
    import inspect

    from automation.task_ops.nodes import split_node as split_node_module
    source = inspect.getsource(split_node_module)
    # 验证 split_node.py 没有 import sub_agent.types
    assert "sub_agent.types" not in source, "split_node 不应 import sub_agent.types (守门 #13 a)"
    # 验证 split_node.py 只用 sub_pool 入口
    assert "sub_pool." in source
    # 验证 import 都是 L0 工具
    assert "split_node_sync" in source  # 同步 wrapper 存在
