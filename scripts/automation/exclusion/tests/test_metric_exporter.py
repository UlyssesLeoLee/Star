"""UT: LockMetricExporter 5 metric 暴露.

per docs/architecture/2026-09-07-exclusion-idempotency/02-basic-design.md §7.4
per ask_2e8740e6779ac6a8d854c590 拍板
"""

import unittest

from scripts.automation.exclusion.lock_metric_exporter import (
    AcquireResult,
    LockMetricExporter,
    LockTier,
)


class TestLockMetricExporter(unittest.TestCase):
    """5 metric exporter 实证."""

    def setUp(self):
        self.exporter = LockMetricExporter()

    def test_record_acquired(self):
        self.exporter.record_acquired(LockTier.L1_TopAgent, AcquireResult.SUCCESS, wait_seconds=0.01)
        self.exporter.record_acquired(LockTier.L1_TopAgent, AcquireResult.SUCCESS, wait_seconds=0.02)
        text = self.exporter.export()
        self.assertIn('star_lock_acquired_total{tier="L1_TopAgent",result="success"} 2', text)

    def test_record_hold(self):
        self.exporter.record_hold(LockTier.L2_SubAgent, hold_seconds=10.5)
        self.exporter.record_hold(LockTier.L2_SubAgent, hold_seconds=20.0)
        text = self.exporter.export()
        self.assertIn('star_lock_hold_seconds', text)
        self.assertIn('quantile="0.5"', text)

    def test_record_timeout(self):
        self.exporter.record_timeout(LockTier.L3_Domain)
        self.exporter.record_timeout(LockTier.L3_Domain)
        self.exporter.record_timeout(LockTier.L3_Domain)
        text = self.exporter.export()
        self.assertIn('star_lock_timeout_total{tier="L3_Domain"} 3', text)

    def test_record_dedup(self):
        self.exporter.record_dedup("client")
        self.exporter.record_dedup("client")
        self.exporter.record_dedup("business")
        text = self.exporter.export()
        self.assertIn('star_idempotency_dedup_total{key_type="client"} 2', text)
        self.assertIn('star_idempotency_dedup_total{key_type="business"} 1', text)

    def test_export_prometheus_text_format(self):
        """5 metric 全部暴露 + HELP + TYPE 元数据."""
        self.exporter.record_acquired(LockTier.L1_TopAgent, AcquireResult.SUCCESS)
        text = self.exporter.export()
        # 5 metric 全部出现
        self.assertIn("star_lock_acquired_total", text)
        self.assertIn("star_lock_wait_seconds", text)
        self.assertIn("star_lock_hold_seconds", text)
        self.assertIn("star_lock_timeout_total", text)
        self.assertIn("star_idempotency_dedup_total", text)
        # HELP + TYPE 元数据
        self.assertIn("# HELP", text)
        self.assertIn("# TYPE", text)

    def test_reset(self):
        self.exporter.record_acquired(LockTier.L1_TopAgent, AcquireResult.SUCCESS)
        self.exporter.reset()
        text = self.exporter.export()
        # reset 后 acquired_total 不应有数据
        self.assertNotIn("success", text.split("# HELP star_lock_wait_seconds")[0])


if __name__ == "__main__":
    unittest.main()
