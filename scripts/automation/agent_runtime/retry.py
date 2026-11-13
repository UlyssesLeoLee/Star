#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=1c983b6125f101699fe97cdc1f22a10ee04f162929c0983cdefc87140e03d134
# MERGE (self:File {path:"scripts/automation/agent_runtime/retry.py"})
# @endcypher
"""agent_runtime.retry — 有限重试 + 指数退避 (per SRS §51 REQ-RETRY-001 + 03-DD §7)。

错误分类 (per 03-DD §7.1):
    Retryable     → max_retry + backoff (1s, 2s, 4s)
    Non-Retryable → 直接失败
    Fatal         → 立即失败 (OOM / 不变式违反)
    禁止无限 retry (per SRS §51)。

幂等性 (per SRS §52 REQ-IDEMPOTENT-001): Task 带 operation_id / task_id /
correlation_id, 重试时 LLM/Tool 走 idempotency key 路径。

约束: 标准库 only; sleep 注入保证测试确定性。
"""

from __future__ import annotations

import time
from typing import Callable, Optional, TypeVar

T = TypeVar("T")
E = TypeVar("E")


class RetryExhausted(Exception):
    """重试耗尽 (per 03-DD §7.2 RetryError::Exhausted)。"""

    def __init__(self, attempts: int, last_error):
        self.attempts = attempts
        self.last_error = last_error
        super().__init__(f"retry exhausted after {attempts} attempts: {last_error!r}")


def is_retryable_default(error) -> bool:
    """默认分类 (per 03-DD §7.1): Retryable 异常类型白名单。"""
    return isinstance(
        error,
        (TimeoutError, ConnectionError, OSError),
    ) and not isinstance(error, (MemoryError,))


class RetryPolicy:
    """RetryPolicy (per 03-DD §7.2): max_retry + ExponentialBackoff。"""

    def __init__(
        self,
        max_retry: int = 3,
        base_delay: float = 1.0,
        backoff_factor: float = 2.0,
        retryable: Optional[Callable[[object], bool]] = None,
    ):
        if max_retry < 0:
            raise ValueError("max_retry 必须 >= 0 (禁止无限 retry, per SRS §51)")
        self.max_retry = max_retry
        self.base_delay = base_delay
        self.backoff_factor = backoff_factor
        self.retryable = retryable or is_retryable_default

    def delay(self, attempt: int) -> float:
        """指数退避: base * factor^(attempt-1) → 1s, 2s, 4s (per 03-DD §7.1)。"""
        return self.base_delay * (self.backoff_factor ** (attempt - 1))

    def run(self, op: Callable[[], T], sleep: Callable[[float], None] = time.sleep) -> T:
        """执行 op, 失败按分类重试 (per 03-DD §7.2 retry 伪码)。

        Raises:
            RetryExhausted: max_retry 用尽。
        """
        attempt = 0
        while True:
            try:
                return op()
            except Exception as error:  # noqa: BLE001 — 重试框架必须捕获业务异常
                if attempt < self.max_retry and self.retryable(error):
                    attempt += 1
                    sleep(self.delay(attempt))
                    continue
                raise RetryExhausted(attempts=attempt, last_error=error) from error
