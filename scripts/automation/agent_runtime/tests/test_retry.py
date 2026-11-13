#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_retry — 有限重试 + 指数退避 (per SRS §51 + 03-DD §7)。"""

import unittest

from automation.agent_runtime.retry import RetryExhausted, RetryPolicy, is_retryable_default


class TestRetryPolicy(unittest.TestCase):
    def test_success_first_try(self):
        policy = RetryPolicy(max_retry=3)
        self.assertEqual(policy.run(lambda: "ok", sleep=lambda s: None), "ok")

    def test_retry_then_success(self):
        calls = {"n": 0}

        def flaky():
            calls["n"] += 1
            if calls["n"] < 3:
                raise ConnectionError("boom")
            return "recovered"

        delays = []
        result = policy_run_ok(flaky, delays)
        self.assertEqual(result, "recovered")
        self.assertEqual(calls["n"], 3)
        self.assertEqual(delays, [1.0, 2.0])  # 指数退避 1s, 2s (per 03-DD §7.1)

    def test_exhausted_raises(self):
        def always_fail():
            raise TimeoutError("down")

        with self.assertRaises(RetryExhausted) as ctx:
            RetryPolicy(max_retry=2).run(always_fail, sleep=lambda s: None)
        self.assertEqual(ctx.exception.attempts, 2)
        self.assertIsInstance(ctx.exception.last_error, TimeoutError)

    def test_non_retryable_fails_immediately(self):
        """per 03-DD §7.1: Invalid input / Permission denied → 不重试。"""
        calls = {"n": 0}

        def invalid():
            calls["n"] += 1
            raise ValueError("invalid input")

        with self.assertRaises(RetryExhausted):
            RetryPolicy(max_retry=3).run(invalid, sleep=lambda s: None)
        self.assertEqual(calls["n"], 1)

    def test_fatal_never_retried(self):
        """per 03-DD §7.1: OOM 类 Fatal → 立即失败。"""
        calls = {"n": 0}

        def fatal():
            calls["n"] += 1
            raise MemoryError("oom")

        with self.assertRaises(RetryExhausted):
            RetryPolicy(max_retry=3).run(fatal, sleep=lambda s: None)
        self.assertEqual(calls["n"], 1)

    def test_delay_sequence(self):
        policy = RetryPolicy(max_retry=3, base_delay=1.0, backoff_factor=2.0)
        self.assertEqual([policy.delay(a) for a in (1, 2, 3)], [1.0, 2.0, 4.0])

    def test_max_retry_nonnegative(self):
        with self.assertRaises(ValueError):
            RetryPolicy(max_retry=-1)

    def test_default_classifier(self):
        self.assertTrue(is_retryable_default(TimeoutError()))
        self.assertTrue(is_retryable_default(ConnectionError()))
        self.assertFalse(is_retryable_default(ValueError()))
        self.assertFalse(is_retryable_default(MemoryError()))


def policy_run_ok(flaky, delays):
    """RetryPolicy.run 记录 sleep 序列。"""
    return RetryPolicy(max_retry=3).run(flaky, sleep=delays.append)


if __name__ == "__main__":
    unittest.main()
