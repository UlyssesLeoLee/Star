#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=6ab237e8a1690199dc2658b4fa4beb642e47c5ac6b646a9c1614415f31e0f4f4
# MERGE (self:File {path:"scripts/automation/agent_runtime/tests/test_backpressure.py"})
# @endcypher
"""test_backpressure — 反压 + 5 溢出策略 (per SRS §46-§47 + 03-DD §6.3)。"""

import unittest

from automation.agent_runtime.backpressure import (
    Backpressure,
    BackpressureDecision,
    QueueFullError,
    ThrottleError,
)


class TestBackpressure(unittest.TestCase):
    def test_proceed_when_not_full(self):
        bp = Backpressure(max_size=10, size_provider=lambda: 5)
        self.assertTrue(bp.has_slot())
        self.assertEqual(bp.await_slot().action, "Proceed")

    def test_reject_raises(self):
        bp = Backpressure(max_size=1, overflow_policy="Reject", size_provider=lambda: 1)
        with self.assertRaises(QueueFullError):
            bp.await_slot()

    def test_delay_decision(self):
        bp = Backpressure(max_size=1, overflow_policy="Delay",
                          delay_ms=250.0, size_provider=lambda: 1)
        decision = bp.await_slot()
        self.assertEqual(decision.action, "Delay")
        self.assertEqual(decision.delay_ms, 250.0)

    def test_drop_low(self):
        dropped = []
        bp = Backpressure(max_size=1, overflow_policy="DropLow",
                          size_provider=lambda: 1,
                          drop_low=lambda count: dropped.append(count) or ["bg-task"])
        decision = bp.await_slot()
        self.assertEqual(decision.action, "DropLow")
        self.assertEqual(decision.dropped_low, ("bg-task",))
        self.assertEqual(dropped, [1])

    def test_persist(self):
        persisted = []
        bp = Backpressure(max_size=1, overflow_policy="Persist",
                          size_provider=lambda: 1,
                          persist_to_dlq=lambda: persisted.append("dlq") or 1)
        decision = bp.await_slot()
        self.assertEqual(decision.action, "Persist")
        self.assertEqual(persisted, ["dlq"])

    def test_throttle_default(self):
        """per SRS §101: Throttle 是 Fail Safe 默认 (429 retry_after=60)。"""
        bp = Backpressure(max_size=1, size_provider=lambda: 1)
        with self.assertRaises(ThrottleError) as ctx:
            bp.await_slot()
        self.assertEqual(ctx.exception.retry_after, 60.0)

    def test_throttle_custom_retry_after(self):
        bp = Backpressure(max_size=1, overflow_policy="Throttle",
                          throttle_retry_after=120.0, size_provider=lambda: 1)
        with self.assertRaises(ThrottleError) as ctx:
            bp.await_slot()
        self.assertEqual(ctx.exception.retry_after, 120.0)

    def test_invalid_policy_rejected(self):
        with self.assertRaises(ValueError):
            Backpressure(max_size=10, overflow_policy="Unlimited")

    def test_zero_max_size_rejected(self):
        with self.assertRaises(ValueError):
            Backpressure(max_size=0)

    def test_decision_shape(self):
        d = BackpressureDecision(action="Proceed")
        self.assertEqual(d.dropped_low, ())
        self.assertEqual(d.delay_ms, 0.0)


if __name__ == "__main__":
    unittest.main()
