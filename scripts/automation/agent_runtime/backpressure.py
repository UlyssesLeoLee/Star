#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.backpressure — 反压 + 队列溢出策略 (per SRS §46-§47 + 03-DD §6.3)。

反压 (per SRS §46 REQ-SCH-004):
    Incoming → Queue → HOT Slot? → Yes: Execute / No: Wait。
    队列必须 bounded (per SRS §47)。

溢出策略 (per 03-DD §6.3 Backpressure.await_slot, 必须可配置 per SRS §47):
    Reject  → 拒绝入队 (QueueFull)
    Delay   → 延迟重试 (delay_ms)
    DropLow → 丢最低优先级
    Persist → 移入 DLQ (persist_to_dlq)
    Throttle→ 429 retry_after (per 03-DD §6.3 Throttle { retry_after: 60 })
    禁止 OOM (per SRS §47)。

约束: 标准库 only。
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Optional


class QueueFullError(RuntimeError):
    """Reject 策略 (per 03-DD §6.3 OverflowPolicy::Reject → Error::QueueFull)。"""


@dataclass(frozen=True)
class ThrottleError(RuntimeError):
    """Throttle 策略 (per 03-DD §6.3 Throttle { retry_after: 60 })。"""

    retry_after: float = 60.0

    def __init__(self, retry_after: float = 60.0):
        super().__init__(f"Throttle: retry_after={retry_after}s")
        object.__setattr__(self, "retry_after", retry_after)


@dataclass(frozen=True)
class BackpressureDecision:
    """await_slot 判定结果 (per 03-DD §6.3)。"""

    action: str  # Proceed / Reject / Delay / DropLow / Persist / Throttle
    dropped_low: tuple = ()
    delay_ms: float = 0.0
    retry_after: float = 0.0


class Backpressure:
    """Bounded Queue + Overflow Policy (per 03-DD §6.3)。"""

    def __init__(
        self,
        max_size: int,
        overflow_policy: str = "Throttle",
        throttle_retry_after: float = 60.0,
        delay_ms: float = 200.0,
        size_provider: Optional[Callable[[], int]] = None,
        drop_low: Optional[Callable[[int], list]] = None,
        persist_to_dlq: Optional[Callable[[], int]] = None,
    ):
        if max_size < 1:
            raise ValueError("max_size 必须 >= 1 (禁止无界队列, per SRS §47)")
        if overflow_policy not in ("Reject", "Delay", "DropLow", "Persist", "Throttle"):
            raise ValueError(f"非法 overflow_policy: {overflow_policy!r}")
        self.max_size = max_size
        self.overflow_policy = overflow_policy
        self.throttle_retry_after = throttle_retry_after
        self.delay_ms = delay_ms
        # 注入回调 (测试 / 跟 TaskQueue + Scheduler 解耦)
        self._size_provider = size_provider or (lambda: 0)
        self._drop_low = drop_low or (lambda count: [])
        self._persist_to_dlq = persist_to_dlq or (lambda: 0)

    def current_size(self) -> int:
        return self._size_provider()

    def has_slot(self) -> bool:
        """队列未满 → True (per 03-DD §6.3 await_slot 快路径)。"""
        return self.current_size() < self.max_size

    def await_slot(self) -> BackpressureDecision:
        """判定 (per 03-DD §6.3)。未满 Proceed; 满 → 按策略。"""
        if self.has_slot():
            return BackpressureDecision(action="Proceed")
        if self.overflow_policy == "Reject":
            raise QueueFullError(f"queue full ({self.current_size()}/{self.max_size})")
        if self.overflow_policy == "Delay":
            return BackpressureDecision(action="Delay", delay_ms=self.delay_ms)
        if self.overflow_policy == "DropLow":
            dropped = tuple(self._drop_low(1))
            return BackpressureDecision(action="DropLow", dropped_low=dropped)
        if self.overflow_policy == "Persist":
            self._persist_to_dlq()
            return BackpressureDecision(action="Persist")
        # Throttle (Fail Safe 默认, per SRS §101)
        raise ThrottleError(retry_after=self.throttle_retry_after)
