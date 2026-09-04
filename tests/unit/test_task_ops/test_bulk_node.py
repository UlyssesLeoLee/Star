"""tests/unit/test_task_ops/test_bulk_node.py — UT-23 bulk_node (M-N4)

Per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §8.2:
    UT-23 | bulk_node (M-N4) | asyncio.gather N + partial failure rollback
    (per NFR-TMO-03) | tests/unit/test_task_ops_nodes.py

覆盖 4 类 partial failure test case (per brief §3 + 守门 #13 a):
    1. 0 失败 → outcome=success
    2. 部分失败 < 20% (1/10=10%) → outcome=partial (success_rate=90% >= 80%)
    3. 部分失败 > 20% (3/5=60%, success=40%) → outcome=rolled_back
    4. 全部失败 → outcome=rolled_back

扩展 case (NFR-TMO-03 边界 + 不可逆 action):
    5. 边界值 1/5 失败 (20%, success=80%) → outcome=partial (= 阈值)
    6. cancel 不可逆 + 部分失败 → outcome=rolled_back 但 rolled_back_ids=[]

约束:
    - pytest_asyncio (mode=auto)
    - 标准库 only (无外部卡 action 实现依赖)
    - mock card_action_fn (per 守门 #23)
"""

from __future__ import annotations

import asyncio
import sys
from pathlib import Path

import pytest

# 让 tests/ 能 import scripts/automation
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.automation.task_ops.bulk_queue import (  # noqa: E402
    BulkAction,
    BulkOperationQueue,
    BulkOutcome,
    REVERSE_ACTION_MAP,
    VALID_ACTIONS,
    mock_card_action,
)
from scripts.automation.task_ops.nodes.bulk_node import make_bulk_node  # noqa: E402


# ---------------------------------------------------------------------------
# mock card_action (per 守门 #23 mock 模式, 不开外部 API)
# ---------------------------------------------------------------------------


def make_selective_card_action(fail_ids: set):
    """返 async card_action_fn: 失败 fail_ids 中 task_id, 成功其余"""

    async def selective_card_action(task_id: str, action: str, action_params=None) -> bool:
        await asyncio.sleep(0)
        if task_id in fail_ids:
            raise RuntimeError(f"simulated failure for {task_id!r}")
        return True

    return selective_card_action


# ---------------------------------------------------------------------------
# Test: BulkAction 验证
# ---------------------------------------------------------------------------


def test_bulk_action_validates_target_ids():
    """空 target_task_ids 必 raise"""
    with pytest.raises(ValueError, match="must be non-empty"):
        BulkAction(target_task_ids=[], action="pause")


def test_bulk_action_validates_action_type():
    """action 必须在 VALID_ACTIONS 4 类内"""
    with pytest.raises(ValueError, match="must be one of"):
        BulkAction(target_task_ids=["t1"], action="invalid_action")


def test_bulk_action_set_priority_requires_priority():
    """set_priority action 必须带 priority 字段"""
    with pytest.raises(ValueError, match="requires 'priority'"):
        BulkAction(target_task_ids=["t1"], action="set_priority", action_params={})


def test_bulk_action_set_priority_with_priority_ok():
    """set_priority + priority 字段 OK"""
    a = BulkAction(
        target_task_ids=["t1"], action="set_priority", action_params={"priority": 5}
    )
    assert a.action_params["priority"] == 5


def test_reverse_action_map_pause_resume():
    """pause <-> resume 互逆 (per 03 §3.2.1.1 reverse_action 映射)"""
    assert REVERSE_ACTION_MAP["pause"] == "resume"
    assert REVERSE_ACTION_MAP["resume"] == "pause"


def test_reverse_action_map_cancel_set_priority_irreversible():
    """cancel + set_priority 不可逆 (None)"""
    assert REVERSE_ACTION_MAP["cancel"] is None
    assert REVERSE_ACTION_MAP["set_priority"] is None


# ---------------------------------------------------------------------------
# Test: BulkOperationQueue standalone (4 类 partial failure)
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_bulk_queue_no_failure_outcome_success():
    """Case 1: 0 失败 (5 张卡全部成功) → outcome=success"""
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(set()))
    q.enqueue(BulkAction(target_task_ids=["t1", "t2", "t3", "t4", "t5"], action="pause"))
    results = await q.flush()
    assert len(results) == 1
    r = results[0]
    assert r.outcome == BulkOutcome.SUCCESS.value
    assert r.success_count == 5
    assert r.failed_count == 0
    assert r.failed_ids == []
    assert r.rolled_back_ids == []


@pytest.mark.asyncio
async def test_bulk_queue_partial_failure_below_threshold_outcome_partial():
    """Case 2: 1/10 失败 (10% fail, 90% success >= 80%) → outcome=partial"""
    fail_ids = {"fail-1"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    ids = [f"ok-{i}" for i in range(9)] + ["fail-1"]
    q.enqueue(BulkAction(target_task_ids=ids, action="pause"))
    results = await q.flush()
    r = results[0]
    assert r.outcome == BulkOutcome.PARTIAL.value
    assert r.success_count == 9
    assert r.failed_count == 1
    assert r.failure_rate == pytest.approx(0.10)
    assert r.rolled_back_ids == []  # partial success, no rollback


@pytest.mark.asyncio
async def test_bulk_queue_partial_failure_above_threshold_rollback_all():
    """Case 3: 3/5 失败 (60% fail, 40% success < 80%) → outcome=rolled_back
    验证成功卡 (ok-1, ok-2) 被 reverse_action=resume 回滚"""
    fail_ids = {"fail-1", "fail-2", "fail-3"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    q.enqueue(
        BulkAction(target_task_ids=["ok-1", "ok-2", "fail-1", "fail-2", "fail-3"], action="pause")
    )
    results = await q.flush()
    r = results[0]
    assert r.outcome == BulkOutcome.ROLLED_BACK.value
    assert r.success_count == 2
    assert r.failed_count == 3
    assert r.failure_rate == pytest.approx(0.60)
    # pause 可逆, 成功卡 ok-1/ok-2 走 reverse_action=resume 回滚
    assert set(r.rolled_back_ids) == {"ok-1", "ok-2"}
    assert r.rollback_failed_ids == []


@pytest.mark.asyncio
async def test_bulk_queue_all_failure_outcome_rolled_back():
    """Case 4: 5/5 失败 (100% fail) → outcome=rolled_back
    无 success_ids 可 rollback, rolled_back_ids=[]"""
    fail_ids = {f"fail-{i}" for i in range(5)}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    q.enqueue(BulkAction(target_task_ids=list(fail_ids), action="pause"))
    results = await q.flush()
    r = results[0]
    assert r.outcome == BulkOutcome.ROLLED_BACK.value
    assert r.success_count == 0
    assert r.failed_count == 5
    assert r.failure_rate == 1.0
    assert r.rolled_back_ids == []  # 无 success 可 rollback


@pytest.mark.asyncio
async def test_bulk_queue_boundary_1_of_5_failure_partial():
    """Case 5: 边界值 1/5 失败 (20% fail, 80% success = 阈值) → outcome=partial

    阈值: failure_rate <= 1 - partial_success_threshold (0.20) 时算 partial
    1/5 = 0.20 == 0.20 → 边界 = partial"""
    fail_ids = {"fail-1"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    q.enqueue(
        BulkAction(
            target_task_ids=["ok-1", "ok-2", "ok-3", "ok-4", "fail-1"], action="pause"
        )
    )
    results = await q.flush()
    r = results[0]
    assert r.failure_rate == pytest.approx(0.20)
    # 边界 = 0.20, partial_success_threshold=0.80
    # success_rate (0.80) < partial_success_threshold (0.80) 严格不成立
    # → 走 partial 路径
    assert r.outcome == BulkOutcome.PARTIAL.value


@pytest.mark.asyncio
async def test_bulk_queue_cancel_irreversible_with_failure():
    """Case 6: cancel 不可逆 + 部分失败 → outcome=rolled_back 但 rolled_back_ids=[]

    REVERSE_ACTION_MAP['cancel'] = None, 无 reverse_action 可走"""
    fail_ids = {"fail-1", "fail-2"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    q.enqueue(
        BulkAction(
            target_task_ids=["ok-1", "ok-2", "fail-1", "fail-2"], action="cancel"
        )
    )
    results = await q.flush()
    r = results[0]
    assert r.outcome == BulkOutcome.ROLLED_BACK.value
    assert r.success_count == 2
    assert r.failed_count == 2
    assert r.rolled_back_ids == []  # cancel 不可逆


@pytest.mark.asyncio
async def test_bulk_queue_set_priority_irreversible_with_failure():
    """Case 7: set_priority 不可逆 + 部分失败 → outcome=rolled_back"""
    fail_ids = {"fail-1"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    q.enqueue(
        BulkAction(
            target_task_ids=["ok-1", "ok-2", "fail-1"],
            action="set_priority",
            action_params={"priority": 5},
        )
    )
    results = await q.flush()
    r = results[0]
    assert r.outcome == BulkOutcome.ROLLED_BACK.value
    assert r.rolled_back_ids == []


# ---------------------------------------------------------------------------
# Test: empty queue
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_bulk_queue_empty_flush_returns_empty_list():
    """空队列 flush 返 []"""
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(set()))
    results = await q.flush()
    assert results == []
    assert q.stats["empty_batches"] == 1


# ---------------------------------------------------------------------------
# Test: partial_success_threshold 越界
# ---------------------------------------------------------------------------


def test_bulk_queue_invalid_threshold_raises():
    """partial_success_threshold 越界 (非 [0, 1]) 必 raise"""
    with pytest.raises(ValueError, match="must be in"):
        BulkOperationQueue(
            card_action_fn=make_selective_card_action(set()),
            partial_success_threshold=1.5,
        )
    with pytest.raises(ValueError, match="must be in"):
        BulkOperationQueue(
            card_action_fn=make_selective_card_action(set()),
            partial_success_threshold=-0.1,
        )


# ---------------------------------------------------------------------------
# Test: custom partial_success_threshold (custom 60%)
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_bulk_queue_custom_threshold_60_percent():
    """自定义 partial_success_threshold=0.60, 1/5 失败 (20% < 40%) → partial"""
    fail_ids = {"fail-1"}
    q = BulkOperationQueue(
        card_action_fn=make_selective_card_action(fail_ids),
        partial_success_threshold=0.60,
    )
    # 5 张卡 1 失败, failure_rate=0.20, 阈值 0.40
    # success_rate=0.80 >= 0.60 → partial
    q.enqueue(
        BulkAction(
            target_task_ids=["ok-1", "ok-2", "ok-3", "ok-4", "fail-1"], action="pause"
        )
    )
    results = await q.flush()
    r = results[0]
    assert r.outcome == BulkOutcome.PARTIAL.value
    assert r.failure_rate == pytest.approx(0.20)


# ---------------------------------------------------------------------------
# Test: bulk_node (M-N4) 端到端 (state -> state diff)
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_bulk_node_no_failure_returns_success_state():
    """bulk_node 0 失败 → state diff 包含 last_tmo_result.outcome=success"""
    fail_ids: set = set()
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    node = make_bulk_node(queue=q)

    state = {
        "active_tmo_operation": {
            "target_task_ids": ["t1", "t2", "t3"],
            "action": "pause",
            "action_params": {},
        }
    }
    diff = await node(state)
    assert diff["active_tmo_operation"] is None
    assert diff["bulk_operations"] == []
    ltr = diff["global_context"]["last_tmo_result"]
    assert ltr["operation"] == "bulk"
    assert ltr["action"] == "pause"
    assert ltr["outcome"] == "success"
    assert ltr["success_count"] == 3
    assert ltr["failed_count"] == 0
    assert ltr["reverse_action"] == "resume"


@pytest.mark.asyncio
async def test_bulk_node_partial_failure_below_threshold():
    """bulk_node 1/10 失败 (10%) → outcome=partial"""
    fail_ids = {"fail-1"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    node = make_bulk_node(queue=q)

    ids = [f"ok-{i}" for i in range(9)] + ["fail-1"]
    state = {
        "active_tmo_operation": {
            "target_task_ids": ids,
            "action": "pause",
            "action_params": {},
        }
    }
    diff = await node(state)
    ltr = diff["global_context"]["last_tmo_result"]
    assert ltr["outcome"] == "partial"
    assert ltr["success_count"] == 9
    assert ltr["failed_count"] == 1
    assert ltr["failed_ids"] == ["fail-1"]
    assert ltr["rolled_back_ids"] == []


@pytest.mark.asyncio
async def test_bulk_node_partial_failure_above_threshold_rollback():
    """bulk_node 3/5 失败 (60%) → outcome=rolled_back, 成功卡被 resume 回滚"""
    fail_ids = {"fail-1", "fail-2", "fail-3"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    node = make_bulk_node(queue=q)

    state = {
        "active_tmo_operation": {
            "target_task_ids": ["ok-1", "ok-2", "fail-1", "fail-2", "fail-3"],
            "action": "pause",
            "action_params": {},
        }
    }
    diff = await node(state)
    ltr = diff["global_context"]["last_tmo_result"]
    assert ltr["outcome"] == "rolled_back"
    assert ltr["success_count"] == 2
    assert ltr["failed_count"] == 3
    assert set(ltr["rolled_back_ids"]) == {"ok-1", "ok-2"}
    assert ltr["reverse_action"] == "resume"


@pytest.mark.asyncio
async def test_bulk_node_all_failure_rolled_back():
    """bulk_node 全部失败 → outcome=rolled_back, rolled_back_ids=[]"""
    fail_ids = {"fail-1", "fail-2", "fail-3"}
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(fail_ids))
    node = make_bulk_node(queue=q)

    state = {
        "active_tmo_operation": {
            "target_task_ids": ["fail-1", "fail-2", "fail-3"],
            "action": "pause",
            "action_params": {},
        }
    }
    diff = await node(state)
    ltr = diff["global_context"]["last_tmo_result"]
    assert ltr["outcome"] == "rolled_back"
    assert ltr["success_count"] == 0
    assert ltr["failed_count"] == 3
    assert ltr["rolled_back_ids"] == []


@pytest.mark.asyncio
async def test_bulk_node_empty_target_ids_no_op():
    """bulk_node 空 target_task_ids → no-op, 返回 error state"""
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(set()))
    node = make_bulk_node(queue=q)

    state = {
        "active_tmo_operation": {
            "target_task_ids": [],
            "action": "pause",
            "action_params": {},
        }
    }
    diff = await node(state)
    ltr = diff["global_context"]["last_tmo_result"]
    assert ltr["outcome"] == "success"  # 0 fail = success
    assert ltr["success_count"] == 0
    assert ltr["failed_count"] == 0


@pytest.mark.asyncio
async def test_bulk_node_invalid_action_returns_error_state():
    """bulk_node invalid action → 返回 error state, 不 raise"""
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(set()))
    node = make_bulk_node(queue=q)

    state = {
        "active_tmo_operation": {
            "target_task_ids": ["t1"],
            "action": "invalid_xyz",
            "action_params": {},
        }
    }
    diff = await node(state)
    ltr = diff["global_context"]["last_tmo_result"]
    assert "error" in ltr
    assert ltr["failed_count"] == 1  # 标记 1 张卡 fail


@pytest.mark.asyncio
async def test_bulk_node_uses_mock_when_no_queue_injected():
    """bulk_node 工厂 default queue 用 mock_card_action (不 raise)"""
    node = make_bulk_node()  # 无 queue -> mock

    state = {
        "active_tmo_operation": {
            "target_task_ids": ["ok-1", "ok-2"],
            "action": "pause",
            "action_params": {},
        }
    }
    diff = await node(state)
    ltr = diff["global_context"]["last_tmo_result"]
    assert ltr["outcome"] == "success"
    assert ltr["success_count"] == 2


# ---------------------------------------------------------------------------
# Test: VALID_ACTIONS 完整性
# ---------------------------------------------------------------------------


def test_valid_actions_complete_set():
    """VALID_ACTIONS 4 类: pause/resume/cancel/set_priority"""
    assert VALID_ACTIONS == frozenset({"pause", "resume", "cancel", "set_priority"})


# ---------------------------------------------------------------------------
# Test: BulkOperationQueue 累计统计
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_bulk_queue_stats_accumulate_across_flushes():
    """多次 flush 后 stats 累计"""
    q = BulkOperationQueue(card_action_fn=make_selective_card_action(set()))
    # 3 次 success flush
    for i in range(3):
        q.enqueue(BulkAction(target_task_ids=[f"ok-{i}"], action="pause"))
        await q.flush()
    assert q.stats["total_flushes"] == 3
    assert q.stats["success_batches"] == 3
    assert q.stats["total_card_actions"] == 3
    assert q.stats["queue_size"] == 0
