#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.rate_limiter — Token Bucket 速率控制 (per 02-BD §3.1 RateLimiter)。

职责: per-tenant / per-agent 速率控制 (per SRS §42 调度器职责 + §57 Fair)。
禁止: 单 tenant 独占 (per SRS §56 REQ-TENANT-001)。

约束: 标准库 only; 时间注入保证测试确定性。
"""

from __future__ import annotations

import threading
import time
from typing import Dict, Optional


class TokenBucket:
    """Token Bucket (per 02-BD §3.3 Provider Rate Limit 同构)。"""

    def __init__(self, capacity: int, refill_per_sec: float, now: Optional[float] = None):
        if capacity < 1:
            raise ValueError("capacity 必须 >= 1")
        if refill_per_sec < 0:
            raise ValueError("refill_per_sec 必须 >= 0 (0 = 不自动补充, 配额/纯限流场景)")
        self.capacity = capacity
        self.refill_per_sec = refill_per_sec
        self._tokens = float(capacity)
        self._last = time.monotonic() if now is None else now

    def _refill(self, now: float) -> None:
        elapsed = now - self._last
        if elapsed > 0:
            self._tokens = min(float(self.capacity), self._tokens + elapsed * self.refill_per_sec)
            self._last = now

    def try_acquire(self, n: int = 1, now: Optional[float] = None) -> bool:
        """获取 n 个 token; 成功 True (per 02-BD §3.1 acquire)。"""
        t = time.monotonic() if now is None else now
        self._refill(t)
        if self._tokens >= n:
            self._tokens -= n
            return True
        return False

    def available(self, now: Optional[float] = None) -> float:
        t = time.monotonic() if now is None else now
        self._refill(t)
        return self._tokens


class TenantRateLimiter:
    """per-tenant Token Bucket 表 (per SRS §42 Tenant Quota 雏形)。

    thread-safe; 缺省桶配置共享 (per-tenant 独立计数)。
    """

    def __init__(self, capacity: int = 10, refill_per_sec: float = 2.0):
        self.capacity = capacity
        self.refill_per_sec = refill_per_sec
        self._buckets: Dict[str, TokenBucket] = {}
        self._lock = threading.Lock()

    def acquire(self, tenant_id: str, n: int = 1, now: Optional[float] = None) -> bool:
        """acquire(tenant) (per 02-BD §3.1)。"""
        t = time.monotonic() if now is None else now
        with self._lock:
            bucket = self._buckets.get(tenant_id)
            if bucket is None:
                bucket = TokenBucket(self.capacity, self.refill_per_sec, now=t)
                self._buckets[tenant_id] = bucket
        return bucket.try_acquire(n, now=t)

    def release(self, tenant_id: str, n: int = 1, now: Optional[float] = None) -> None:
        """release(tenant) (per 02-BD §3.1): 归还 token (不超 capacity)。"""
        t = time.monotonic() if now is None else now
        with self._lock:
            bucket = self._buckets.get(tenant_id)
            if bucket is None:
                return
        bucket._tokens = min(float(bucket.capacity), bucket._tokens + n)  # noqa: SLF001
        bucket._last = t

    def bucket_count(self) -> int:
        with self._lock:
            return len(self._buckets)
