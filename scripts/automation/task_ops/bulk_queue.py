"""scripts/automation/task_ops/bulk_queue.py — TMO BulkOperationQueue (C-18, M-21)

TMO v0.2 批量操作队列 (per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.6.4 M-N4 +
docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.2.1.1 M-N4 bulk_node 実装):

  - 队列管理: enqueue / enqueue_many / clear
  - 协调执行: flush() 走 asyncio.gather 跑 N 个 card_action (不串行)
  - 部分失败回滚: per NFR-TMO-03 ≥80% success 视为 partial success, 失败 >20% 全部 rollback
  - reverse_action 映射: pause<->resume, cancel/set_priority 不可回滚 (None)

约束 (per 守门 #13 a/d + 守门 #19 + AGENTS.md §4 派生规):
  - 纯 Python (asyncio + dataclasses + typing, 标准库 + 第三方无依赖)
  - TMO 全部 L0 协调 (守门 #13 a L1↔L1 禁止)
  - 不开 OpenAI/Anthropic (守门 #23)
  - audit log 落 docs/reports/tmo-bulk.log (per §3.2 跨子代理 audit 实证)

用法:
    from automation.task_ops.bulk_queue import (
        BulkOperationQueue, BulkAction, BulkActionResult, BulkBatchResult,
    )

    q = BulkOperationQueue(
        card_action_fn=mock_card_action,  # injected, async (task_id, action, params) -> bool
        audit_log=Path("docs/reports/tmo-bulk.log"),
        partial_success_threshold=0.80,  # NFR-TMO-03
    )
    q.enqueue(BulkAction(target_task_ids=["a", "b", "c"], action="pause"))
    q.enqueue_many([BulkAction(...), BulkAction(...)])
    result = await q.flush()
    # result.success / result.partial / result.rolled_back / result.success_count / ...
"""

from __future__ import annotations

import asyncio
import json
import logging
import time
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Awaitable, Callable, Optional, Sequence

# ---------------------------------------------------------------------------
# 常量 (per 守门 #13 d: task card = Work 短 TTL, 4 类 action 必携)
# ---------------------------------------------------------------------------

# 4 类 action (per 02 §2.6.4 M-N4 + L0→L1 protocol `bulk_action`)
VALID_ACTIONS = frozenset({"pause", "resume", "cancel", "set_priority"})

# reverse_action 映射 (per 03 §3.2.1.1 bulk_node 実装):
#   - pause  -> resume (可逆)
#   - resume -> pause (可逆)
#   - cancel -> None (不可逆, 终态, 守门 #13 d superseded 终态)
#   - set_priority -> None (priority 不能 rollback, 改了就改了)
REVERSE_ACTION_MAP: dict[str, Optional[str]] = {
    "pause": "resume",
    "resume": "pause",
    "cancel": None,
    "set_priority": None,
}

# NFR-TMO-03: 失败率超过此阈值则视为整体失败, 全部 rollback
# 默认 0.20 (= 失败 > 20% rollback all, ≥ 80% success 视为 partial success)
DEFAULT_PARTIAL_SUCCESS_THRESHOLD = 0.80

LOG_DIR_DEFAULT = Path(__file__).resolve().parent.parent.parent.parent / "docs" / "reports"
LOG_FILE_DEFAULT = "tmo-bulk.log"


# ---------------------------------------------------------------------------
# Action / Result 数据类
# ---------------------------------------------------------------------------


class BulkOutcome(str, Enum):
    """单次 batch flush 整体结果 (per NFR-TMO-03)"""

    SUCCESS = "success"            # 0 失败 (>= 100% success)
    PARTIAL = "partial"            # 失败率 <= 1 - threshold (>= 80% success per NFR-TMO-03)
    ROLLED_BACK = "rolled_back"    # 失败率 > 1 - threshold, 全部 rollback
    EMPTY = "empty"                # 队列空, 跳过


@dataclass
class BulkAction:
    """单条 bulk action (per L0→L1 `bulk_action` 协议)

    Fields (per 02 §2.6 协议 bulk_action):
        - target_task_ids: list[str] 必填
        - action: "pause" | "resume" | "cancel" | "set_priority"
        - action_params: dict  (set_priority 需 priority 字段, 其余可选)
    """

    target_task_ids: list
    action: str
    action_params: dict = field(default_factory=dict)
    batch_id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])

    def __post_init__(self) -> None:
        if not self.target_task_ids:
            raise ValueError("BulkAction.target_task_ids must be non-empty list")
        if self.action not in VALID_ACTIONS:
            raise ValueError(
                f"BulkAction.action must be one of {sorted(VALID_ACTIONS)}, got {self.action!r}"
            )
        if self.action == "set_priority" and "priority" not in self.action_params:
            raise ValueError(
                "BulkAction(action='set_priority') requires 'priority' in action_params"
            )


@dataclass
class CardActionResult:
    """单张卡的 action 执行结果"""

    task_id: str
    action: str
    success: bool
    error: Optional[str] = None
    duration_ms: float = 0.0
    rolled_back: bool = False  # True if this success was rolled back due to partial failure


@dataclass
class BulkBatchResult:
    """单次 flush 的整体结果 (per 02 §2.6 M-N4 batch_summary)"""

    batch_id: str
    action: str
    target_task_ids: list
    success_count: int
    failed_count: int
    failed_ids: list
    rolled_back_ids: list  # 卡了反向 action 试图回滚的 ids
    rollback_failed_ids: list  # rollback 也失败的 ids (per _safe_card_action 降级)
    outcome: str  # BulkOutcome.value
    started_at: str
    duration_ms: float

    @property
    def total(self) -> int:
        return self.success_count + self.failed_count

    @property
    def failure_rate(self) -> float:
        if self.total == 0:
            return 0.0
        return self.failed_count / self.total

    def to_dict(self) -> dict:
        return {
            "batch_id": self.batch_id,
            "action": self.action,
            "target_task_ids": list(self.target_task_ids),
            "success_count": self.success_count,
            "failed_count": self.failed_count,
            "failed_ids": list(self.failed_ids),
            "rolled_back_ids": list(self.rolled_back_ids),
            "rollback_failed_ids": list(self.rollback_failed_ids),
            "outcome": self.outcome,
            "started_at": self.started_at,
            "duration_ms": self.duration_ms,
            "total": self.total,
            "failure_rate": self.failure_rate,
        }


# ---------------------------------------------------------------------------
# 默认 card_action (per mock 模式, 真接入由 console_server / sub_pool 注入)
# ---------------------------------------------------------------------------


async def mock_card_action(
    task_id: str, action: str, action_params: Optional[dict] = None
) -> bool:
    """Mock card_action 用于 unit test 跟 standalone demo (per 守门 #23 mock 模式)

    行为:
        - "fail-*" 任务 ID 触发失败 (测试用)
        - 其他都成功
    """
    await asyncio.sleep(0)  # 0 ms yield, 模拟 async
    if task_id.startswith("fail-"):
        raise RuntimeError(f"mock card_action: task {task_id!r} simulated failure")
    return True


# ---------------------------------------------------------------------------
# BulkOperationQueue 主体
# ---------------------------------------------------------------------------


# 注入类型: card_action_fn(task_id, action, action_params) -> Awaitable[bool]
CardActionFn = Callable[[str, str, Optional[dict]], Awaitable[bool]]


class BulkOperationQueue:
    """TMO BulkOperationQueue (C-18, M-21) per 02 §2.6.4

    队列:
        - enqueue / enqueue_many / clear
    执行:
        - flush() 走 asyncio.gather 协调 N 个 card_action
        - 部分失败回滚 (per NFR-TMO-03)
    Audit:
        - 每次 flush 落 audit_log (JSONL, 可选)
    """

    def __init__(
        self,
        card_action_fn: Optional[CardActionFn] = None,
        audit_log: Optional[Path] = None,
        partial_success_threshold: float = DEFAULT_PARTIAL_SUCCESS_THRESHOLD,
    ) -> None:
        if not 0.0 <= partial_success_threshold <= 1.0:
            raise ValueError(
                f"partial_success_threshold must be in [0, 1], got {partial_success_threshold}"
            )
        self._card_action_fn: CardActionFn = card_action_fn or mock_card_action
        self._partial_success_threshold = partial_success_threshold
        self._queue: list = []
        self._audit_log: Optional[Path] = None
        if audit_log is not None:
            self._audit_log = Path(audit_log)
            self._audit_log.parent.mkdir(parents=True, exist_ok=True)
        # 累计统计 (跨 flush)
        self._total_flushes: int = 0
        self._total_success_batches: int = 0
        self._total_partial_batches: int = 0
        self._total_rolled_back_batches: int = 0
        self._total_empty_batches: int = 0
        self._total_card_actions: int = 0

    # ------------------------------------------------------------------
    # 队列管理
    # ------------------------------------------------------------------

    def enqueue(self, action: BulkAction) -> None:
        self._queue.append(action)

    def enqueue_many(self, actions: Sequence[BulkAction]) -> None:
        for a in actions:
            self.enqueue(a)

    def clear(self) -> None:
        self._queue.clear()

    @property
    def size(self) -> int:
        return len(self._queue)

    @property
    def partial_success_threshold(self) -> float:
        return self._partial_success_threshold

    @property
    def stats(self) -> dict:
        return {
            "total_flushes": self._total_flushes,
            "success_batches": self._total_success_batches,
            "partial_batches": self._total_partial_batches,
            "rolled_back_batches": self._total_rolled_back_batches,
            "empty_batches": self._total_empty_batches,
            "total_card_actions": self._total_card_actions,
            "queue_size": self._queue_size,
        }

    @property
    def _queue_size(self) -> int:
        return len(self._queue)

    # ------------------------------------------------------------------
    # Flush
    # ------------------------------------------------------------------

    async def flush(self) -> list:
        """Flush 队列中所有 BulkAction, 返回每条 action 的 BulkBatchResult 列表

        流程 per 03 §3.2.1.1:
          1. 取出队列中所有 BulkAction (snapshot, 不在循环中修改)
          2. 对每条 action, 走 asyncio.gather(N 个 card_action)
          3. 收集 success / failed
          4. 失败率 > 1 - partial_success_threshold 时 rollback (per NFR-TMO-03)
          5. 落 audit log
        """
        if not self._queue:
            self._total_empty_batches += 1
            return []
        # 快照并清空队列 (避免 flush 期间 enqueue 干扰)
        actions = list(self._queue)
        self.clear()
        results: list = []
        for action in actions:
            r = await self._flush_one(action)
            results.append(r)
            self._total_flushes += 1
            self._total_card_actions += r.total
            if r.outcome == BulkOutcome.SUCCESS.value:
                self._total_success_batches += 1
            elif r.outcome == BulkOutcome.PARTIAL.value:
                self._total_partial_batches += 1
            elif r.outcome == BulkOutcome.ROLLED_BACK.value:
                self._total_rolled_back_batches += 1
            self._audit(r)
        return results

    async def _flush_one(self, action: BulkAction) -> BulkBatchResult:
        started_at_dt = datetime.now(timezone.utc)
        started_at = started_at_dt.isoformat()
        t0 = time.monotonic()

        # 1. asyncio.gather N 个 card_action
        coros = [
            self._safe_card_action(tid, action.action, action.action_params)
            for tid in action.target_task_ids
        ]
        raw = await asyncio.gather(*coros, return_exceptions=False)
        # raw is list[(task_id, success_or_exc, duration_ms, error_str_or_None)]
        # 但 _safe_card_action 返 (success: bool, error: Optional[str], duration_ms)
        success_count = sum(1 for _, ok, _, _ in raw if ok)
        failed_count = len(raw) - success_count
        failed_ids = [tid for tid, ok, _, _ in raw if not ok]

        # 2. 部分失败回滚判定 (per NFR-TMO-03)
        # 失败率 = failed_count / total
        # 阈值: 失败率 > (1 - partial_success_threshold) 时全部 rollback
        rolled_back_ids: list = []
        rollback_failed_ids: list = []
        outcome: str
        if failed_count == 0:
            outcome = BulkOutcome.SUCCESS.value
        else:
            failure_rate = failed_count / len(raw) if raw else 0.0
            # success rate < partial_success_threshold → rollback all
            success_rate = 1.0 - failure_rate
            if success_rate < self._partial_success_threshold:
                # 失败 > 20% (i.e. success < 80%) → rollback
                reverse = REVERSE_ACTION_MAP.get(action.action)
                if reverse is None:
                    # 不可逆 (cancel / set_priority)
                    outcome = BulkOutcome.ROLLED_BACK.value
                    # 但 _no_ actual rollback, just mark as such
                    # failed_ids 已经在 failed_ids 里, success ones 无法回滚
                    rolled_back_ids = []  # actually we tried but action is non-reversible
                    rollback_failed_ids = []
                else:
                    # 可逆: 走 reverse action (只回滚成功的卡)
                    success_ids = [tid for tid, ok, _, _ in raw if ok]
                    rev_coros = [
                        self._safe_card_action(tid, reverse, action.action_params)
                        for tid in success_ids
                    ]
                    rev_raw = await asyncio.gather(*rev_coros, return_exceptions=False)
                    rolled_back_ids = [tid for tid, ok, _, _ in rev_raw if ok]
                    rollback_failed_ids = [
                        tid for tid, ok, _, _ in rev_raw if not ok
                    ]
                    outcome = BulkOutcome.ROLLED_BACK.value
            else:
                # 失败率 <= 1 - partial_success_threshold → partial success, 保留结果
                outcome = BulkOutcome.PARTIAL.value

        duration_ms = (time.monotonic() - t0) * 1000.0
        return BulkBatchResult(
            batch_id=action.batch_id,
            action=action.action,
            target_task_ids=list(action.target_task_ids),
            success_count=success_count,
            failed_count=failed_count,
            failed_ids=failed_ids,
            rolled_back_ids=rolled_back_ids,
            rollback_failed_ids=rollback_failed_ids,
            outcome=outcome,
            started_at=started_at,
            duration_ms=duration_ms,
        )

    async def _safe_card_action(
        self, task_id: str, action: str, action_params: Optional[dict]
    ) -> tuple:
        """调 card_action_fn, 捕获所有异常, 返 (success, error_str_or_None, duration_ms)"""
        t0 = time.monotonic()
        try:
            ok = await self._card_action_fn(task_id, action, action_params)
            duration = (time.monotonic() - t0) * 1000.0
            if not ok:
                return (task_id, False, duration, "card_action_fn returned False")
            return (task_id, True, duration, None)
        except Exception as exc:  # noqa: BLE001  # card_action_fn 异常都吞
            duration = (time.monotonic() - t0) * 1000.0
            return (task_id, False, duration, f"{type(exc).__name__}: {exc}")

    # ------------------------------------------------------------------
    # Audit
    # ------------------------------------------------------------------

    def _audit(self, result: BulkBatchResult) -> None:
        if self._audit_log is None:
            return
        line = json.dumps(
            {
                "ts": datetime.now(timezone.utc).isoformat(),
                "event": "bulk.flush",
                **result.to_dict(),
            },
            ensure_ascii=False,
        )
        try:
            with self._audit_log.open("a", encoding="utf-8") as f:
                f.write(line + "\n")
        except OSError as exc:
            # audit log 写失败不阻塞主流程 (per 守门 #9 不因 audit 失败回滚)
            logging.getLogger(__name__).warning(
                "audit log write failed: %s (path=%s)", exc, self._audit_log
            )


# ---------------------------------------------------------------------------
# Self-test (per 守门 #19 Python 化, 入口可独立 `python -m ...` 跑)
# ---------------------------------------------------------------------------


async def _self_test() -> None:
    """3 demo case: 0 失败 / 部分失败 (1/5=20%) / 大量失败 (5/5=100%) / cancel 不可逆"""
    log_path = LOG_DIR_DEFAULT / LOG_FILE_DEFAULT
    print(f"audit log: {log_path}")
    q = BulkOperationQueue(audit_log=log_path)

    # 1. 0 失败 (5 张卡全部成功)
    q.enqueue(BulkAction(target_task_ids=["t1", "t2", "t3", "t4", "t5"], action="pause"))
    r1 = (await q.flush())[0]
    print(f"[1] 0 fail       : {r1.outcome}  success={r1.success_count} failed={r1.failed_count}")

    # 2. 部分失败 20% (5 张卡 1 张失败) → success_rate=80% = 阈值, 视为 partial success
    q.enqueue(BulkAction(target_task_ids=["ok1", "ok2", "ok3", "ok4", "fail-1"], action="pause"))
    r2 = (await q.flush())[0]
    print(
        f"[2] 1/5 fail=20% : {r2.outcome}  success={r2.success_count} failed={r2.failed_count}"
        f"  (failure_rate={r2.failure_rate:.0%})"
    )

    # 3. 大量失败 40% (5 张卡 2 张失败) → success_rate=60% < 80% threshold, 全部 rollback
    q.enqueue(
        BulkAction(
            target_task_ids=["ok1", "ok2", "fail-1", "fail-2", "fail-3"], action="pause"
        )
    )
    r3 = (await q.flush())[0]
    print(
        f"[3] 3/5 fail=60% : {r3.outcome}  success={r3.success_count} failed={r3.failed_count}"
        f"  rolled_back={r3.rolled_back_ids}  (failure_rate={r3.failure_rate:.0%})"
    )

    # 4. 全部失败 100% → rolled_back (但 pause 可逆, 实际无 success_ids 可 rollback)
    q.enqueue(BulkAction(target_task_ids=["fail-1", "fail-2"], action="pause"))
    r4 = (await q.flush())[0]
    print(
        f"[4] 2/2 fail=100%: {r4.outcome}  success={r4.success_count} failed={r4.failed_count}"
        f"  rolled_back={r4.rolled_back_ids}  (failure_rate={r4.failure_rate:.0%})"
    )

    # 5. cancel 不可逆 + 大量失败 → outcome=rolled_back 但 rolled_back_ids=[]
    q.enqueue(BulkAction(target_task_ids=["ok1", "ok2", "fail-1", "fail-2"], action="cancel"))
    r5 = (await q.flush())[0]
    print(
        f"[5] cancel 2/4   : {r5.outcome}  success={r5.success_count} failed={r5.failed_count}"
        f"  rolled_back={r5.rolled_back_ids}  (failure_rate={r5.failure_rate:.0%})"
    )

    print("\nstats:", json.dumps(q.stats, ensure_ascii=False, indent=2))


if __name__ == "__main__":  # pragma: no cover
    asyncio.run(_self_test())
