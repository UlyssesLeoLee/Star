#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_rate_limiter — Token Bucket (per 02-BD §3.1 RateLimiter)。"""

import unittest

from automation.agent_runtime.rate_limiter import TenantRateLimiter, TokenBucket


class TestTokenBucket(unittest.TestCase):
    def test_acquire_until_empty(self):
        bucket = TokenBucket(capacity=3, refill_per_sec=1.0, now=0.0)
        self.assertTrue(bucket.try_acquire(now=0.0))
        self.assertTrue(bucket.try_acquire(now=0.0))
        self.assertTrue(bucket.try_acquire(now=0.0))
        self.assertFalse(bucket.try_acquire(now=0.0))

    def test_refill_over_time(self):
        bucket = TokenBucket(capacity=2, refill_per_sec=1.0, now=0.0)
        bucket.try_acquire(2, now=0.0)
        self.assertFalse(bucket.try_acquire(now=0.5))
        self.assertTrue(bucket.try_acquire(now=1.0))

    def test_refill_caps_at_capacity(self):
        bucket = TokenBucket(capacity=2, refill_per_sec=1.0, now=0.0)
        bucket.try_acquire(2, now=0.0)
        self.assertEqual(bucket.available(now=100.0), 2.0)

    def test_invalid_params(self):
        with self.assertRaises(ValueError):
            TokenBucket(capacity=0, refill_per_sec=1.0)
        with self.assertRaises(ValueError):
            TokenBucket(capacity=1, refill_per_sec=-1.0)

    def test_zero_refill_allowed(self):
        """refill=0: 不自动补充 (配额/纯限流场景)。"""
        bucket = TokenBucket(capacity=1, refill_per_sec=0.0, now=0.0)
        self.assertTrue(bucket.try_acquire(now=0.0))
        self.assertFalse(bucket.try_acquire(now=999.0))  # 永不补充


class TestTenantRateLimiter(unittest.TestCase):
    def test_per_tenant_isolation(self):
        """per SRS §56: 单 tenant 不得独占 (各租户独立桶)。"""
        rl = TenantRateLimiter(capacity=2, refill_per_sec=0.0)
        self.assertTrue(rl.acquire("A", now=0.0))
        self.assertTrue(rl.acquire("A", now=0.0))
        self.assertFalse(rl.acquire("A", now=0.0))
        # B 不受 A 耗尽影响
        self.assertTrue(rl.acquire("B", now=0.0))

    def test_release_returns_token(self):
        rl = TenantRateLimiter(capacity=1, refill_per_sec=0.0)
        self.assertTrue(rl.acquire("A", now=0.0))
        self.assertFalse(rl.acquire("A", now=0.0))
        rl.release("A", now=0.0)
        self.assertTrue(rl.acquire("A", now=0.0))

    def test_bucket_count(self):
        rl = TenantRateLimiter(capacity=5, refill_per_sec=0.0)
        rl.acquire("A", now=0.0)
        rl.acquire("B", now=0.0)
        self.assertEqual(rl.bucket_count(), 2)


if __name__ == "__main__":
    unittest.main()
