"""scripts/automation/task_ops/nodes/bulk_node.py — TMO M-N4 bulk_node (per 03 §3.2.1.1)

TMO v0.2 节点: N 张卡批量 action (pause/resume/cancel/set_priority)

流程 (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.2.1.1):
  1. 读 state["active_tmo_operation"] -> target_task_ids, action
  2. asyncio.gather(N 个 card_action), 不串行
  3. 收集 success / failed
  4. 部分失败回滚 (per NFR-TMO-03 partial success ≥ 80%)
  5. 返 state diff (active_tmo_operation=None, last_tmo_result, bulk_operations=[])

约束 (per 守门 #13 a + 守门 #19 + AGENTS.md §4):
  - TMO 全部 L0 协调 (守门 #13 a L1↔L1 禁止通信)
  - 不开 OpenAI/Anthropic (守门 #23)
  - 纯 Python (asyncio + typing, 标准库 + 第三方无依赖)

用法 (LangGraph 集成):
    from automation.task_ops.nodes.bulk_node import make_bulk_node

    # Option A: 注入 BulkOperationQueue (推荐, e2e 测 / 真实接入)
    def make_bulk_node_with_queue(queue: BulkOperationQueue):
        return make_bulk_node(queue=queue)

    # Option B: standalone mode (mock card_action, unit test 友好)
    bulk_node_fn = make_bulk_node()  # 用 mock_card_action

    # StateGraph 集成
    builder.add_node("bulk", bulk_node_fn)
    builder.add_conditional_edges("parse_intent", ..., {"bulk_action": "bulk"})
"""

from __future__ import annotations

import asyncio
import logging
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Optional

from ..bulk_queue import (
    BulkAction,
    BulkBatchResult,
    BulkOperationQueue,
    REVERSE_ACTION_MAP,
    VALID_ACTIONS,
    mock_card_action,
)

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# State diff 类型 (per 02 §2.5 BulkOperationQueue state 字段)
# ---------------------------------------------------------------------------


@dataclass
class BulkNodeResult:
    """bulk_node 返的 state diff (兼容 TopAgentState schema)"""

    bulk_operations: list            # queue drained
    active_tmo_operation: Optional[dict]
    global_context: dict              # last_tmo_result

    def to_state_diff(self) -> dict:
        """转 LangGraph state diff (per 02 §2.5)"""
        return {
            "bulk_operations": self.bulk_operations,
            "active_tmo_operation": self.active_tmo_operation,
            "global_context": self.global_context,
        }


# ---------------------------------------------------------------------------
# bulk_node factory
# ---------------------------------------------------------------------------


def make_bulk_node(
    queue: Optional[BulkOperationQueue] = None,
):
    """工厂: 返 async function(state) -> state_diff

    Args:
        queue: BulkOperationQueue 实例, None 时用 mock_card_action 走 standalone mode

    Returns:
        bulk_node(state: dict) -> dict
    """

    _queue = queue or BulkOperationQueue(card_action_fn=mock_card_action)

    async def bulk_node(state: dict) -> dict:
        """TMO M-N4 bulk_node — N 张卡批量 action (pause/resume/cancel/set_priority)"""
        op = state.get("active_tmo_operation") or {}
        target_ids = op.get("target_task_ids", [])
        action = op.get("action", "pause")
        action_params = op.get("action_params", {}) or {}

        if not target_ids:
            # empty target_task_ids = 0 卡 = 0 失败 = success (per NFR-TMO-03 0 fail = 100% success)
            logger.warning("bulk_node called with empty target_task_ids, no-op treated as success")
            return BulkNodeResult(
                bulk_operations=[],
                active_tmo_operation=None,
                global_context={
                    "last_tmo_result": {
                        "operation": "bulk",
                        "action": action,
                        "success_count": 0,
                        "failed_count": 0,
                        "failed_ids": [],
                        "rolled_back_ids": [],
                        "rollback_failed_ids": [],
                        "outcome": "success",
                        "failure_rate": 0.0,
                        "reverse_action": REVERSE_ACTION_MAP.get(action),
                        "note": "empty target_task_ids",
                    }
                },
            ).to_state_diff()

        if action not in VALID_ACTIONS:
            logger.error(
                "bulk_node invalid action %r, must be one of %s", action, sorted(VALID_ACTIONS)
            )
            return BulkNodeResult(
                bulk_operations=[],
                active_tmo_operation=None,
                global_context={
                    "last_tmo_result": {
                        "operation": "bulk",
                        "action": action,
                        "error": f"invalid action, must be one of {sorted(VALID_ACTIONS)}",
                        "success_count": 0,
                        "failed_count": len(target_ids),
                    }
                },
            ).to_state_diff()

        # 构造 BulkAction 并 enqueue
        bulk_action = BulkAction(
            target_task_ids=list(target_ids),
            action=action,
            action_params=dict(action_params),
        )
        _queue.enqueue(bulk_action)

        # flush
        batch_results = await _queue.flush()
        # batch_results 是 list[BulkBatchResult], 我们只 enqueue 了 1 个, 取 [0]
        if not batch_results:
            # 异常, _queue.flush 走完
            br_dict: dict = {}
        else:
            br = batch_results[0]
            br_dict = br.to_dict()

        # 构造 state diff (per 02 §2.5)
        result = BulkNodeResult(
            bulk_operations=[],
            active_tmo_operation=None,
            global_context={
                "last_tmo_result": {
                    "operation": "bulk",
                    "action": action,
                    "success_count": br_dict.get("success_count", 0),
                    "failed_count": br_dict.get("failed_count", 0),
                    "failed_ids": br_dict.get("failed_ids", []),
                    "rolled_back_ids": br_dict.get("rolled_back_ids", []),
                    "rollback_failed_ids": br_dict.get("rollback_failed_ids", []),
                    "outcome": br_dict.get("outcome", "unknown"),
                    "batch_id": br_dict.get("batch_id", ""),
                    "started_at": br_dict.get("started_at", ""),
                    "duration_ms": br_dict.get("duration_ms", 0.0),
                    "failure_rate": br_dict.get("failure_rate", 0.0),
                    "reverse_action": REVERSE_ACTION_MAP.get(action),
                }
            },
        )
        return result.to_state_diff()

    return bulk_node


# ---------------------------------------------------------------------------
# Default bulk_node 实例 (per standalone mode, mock_card_action)
# ---------------------------------------------------------------------------


bulk_node = make_bulk_node()


# ---------------------------------------------------------------------------
# Self-test
# ---------------------------------------------------------------------------


async def _self_test() -> None:
    """Self-test: 跑 4 个 case 验证 bulk_node + BulkOperationQueue 整合"""
    from ..bulk_queue import LOG_DIR_DEFAULT, LOG_FILE_DEFAULT  # local import
    log_path = LOG_DIR_DEFAULT / LOG_FILE_DEFAULT
    print(f"audit log: {log_path}")
    q = BulkOperationQueue(audit_log=log_path)
    node = make_bulk_node(queue=q)

    # Case 1: 0 失败 (5 张卡全部成功)
    s1 = {
        "active_tmo_operation": {
            "target_task_ids": ["t1", "t2", "t3", "t4", "t5"],
            "action": "pause",
            "action_params": {},
        }
    }
    r1 = await node(s1)
    print(f"[1] 0 fail         : outcome={r1['global_context']['last_tmo_result']['outcome']}")
    print(
        f"    success={r1['global_context']['last_tmo_result']['success_count']}"
        f" failed={r1['global_context']['last_tmo_result']['failed_count']}"
    )

    # Case 2: 1/5 失败 (20%, success_rate=80% = 阈值, partial success)
    s2 = {
        "active_tmo_operation": {
            "target_task_ids": ["ok1", "ok2", "ok3", "ok4", "fail-1"],
            "action": "pause",
            "action_params": {},
        }
    }
    r2 = await node(s2)
    print(f"[2] 1/5 fail=20%   : outcome={r2['global_context']['last_tmo_result']['outcome']}")
    print(
        f"    success={r2['global_context']['last_tmo_result']['success_count']}"
        f" failed={r2['global_context']['last_tmo_result']['failed_count']}"
        f" failure_rate={r2['global_context']['last_tmo_result']['failure_rate']:.0%}"
    )

    # Case 3: 3/5 失败 (60%, success_rate=40% < 80%, 全部 rollback)
    s3 = {
        "active_tmo_operation": {
            "target_task_ids": ["ok1", "ok2", "fail-1", "fail-2", "fail-3"],
            "action": "pause",
            "action_params": {},
        }
    }
    r3 = await node(s3)
    print(f"[3] 3/5 fail=60%   : outcome={r3['global_context']['last_tmo_result']['outcome']}")
    print(
        f"    success={r3['global_context']['last_tmo_result']['success_count']}"
        f" failed={r3['global_context']['last_tmo_result']['failed_count']}"
        f" rolled_back={r3['global_context']['last_tmo_result']['rolled_back_ids']}"
        f" failure_rate={r3['global_context']['last_tmo_result']['failure_rate']:.0%}"
    )

    # Case 4: 4/4 失败 (100%, cancel 不可逆)
    s4 = {
        "active_tmo_operation": {
            "target_task_ids": ["fail-1", "fail-2", "fail-3", "fail-4"],
            "action": "cancel",
            "action_params": {},
        }
    }
    r4 = await node(s4)
    print(f"[4] 4/4 fail cancel: outcome={r4['global_context']['last_tmo_result']['outcome']}")
    print(
        f"    success={r4['global_context']['last_tmo_result']['success_count']}"
        f" failed={r4['global_context']['last_tmo_result']['failed_count']}"
        f" rolled_back={r4['global_context']['last_tmo_result']['rolled_back_ids']}"
        f" failure_rate={r4['global_context']['last_tmo_result']['failure_rate']:.0%}"
    )

    print("\nstats:", q.stats)


if __name__ == "__main__":  # pragma: no cover
    asyncio.run(_self_test())
