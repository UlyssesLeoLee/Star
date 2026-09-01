#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/__tests__/saga_e2e_test.py — C.6 Saga 跨 5 域测试
(per docs/automation-design.md v0.1 §4.2 + §6.4 C.6 10 测试)

10 测试:
- 5 域 × 2 case (成功 + 失败回滚) = 10 case 实证
- 每个域都触发一次失败, 验证回滚前 N 步
- 成功 case 验证全部 step success + no compensated
- 失败 case 验证 failed_at + compensated_steps 数量

约束 (per 守门 #1 v1):
- 标准库 only (unittest)
- sys.path 加 scripts/ 让 import automation 找到
"""

import sys
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.saga_e2e import SagaE2E, SagaStep, Domain, SAGA_CHAIN


class SagaSuccessTest(unittest.TestCase):
    """成功 case: 5 域全 success"""

    def test_01_saga_success_all_5_domains(self):
        """saga 跑通 5 域全部 success"""
        saga = SagaE2E(fail_domain=None)
        result = saga.run()
        self.assertTrue(result.success, "saga should succeed")
        self.assertIsNone(result.failed_at)
        self.assertEqual(len(result.compensated_steps), 0)
        for step in result.steps:
            self.assertEqual(step.status, "success", f"{step.id} should be success")
        # 5 域全有 idempotency_key (INV-SG-05)
        for step in result.steps:
            self.assertIsNotNone(step.idempotency_key, f"{step.id} missing idempotency_key")
            self.assertIn("saga-", step.idempotency_key)

    def test_02_saga_chain_5_domains(self):
        """SAGA_CHAIN 含 5 域 (player/economy/match/social/admin)"""
        expected_domains = [Domain.PLAYER, Domain.ECONOMY, Domain.MATCH, Domain.SOCIAL, Domain.ADMIN]
        actual_domains = [s.domain for s in SAGA_CHAIN]
        self.assertEqual(actual_domains, expected_domains)


class SagaFailureTest(unittest.TestCase):
    """失败 case: 5 域各触发 1 次, 验证回滚"""

    def _assert_rollback(self, fail_domain: Domain, expected_compensated_count: int, expected_failed_at: str):
        saga = SagaE2E(fail_domain=fail_domain)
        result = saga.run()
        self.assertFalse(result.success, f"saga should fail at {fail_domain}")
        self.assertEqual(result.failed_at, expected_failed_at)
        self.assertEqual(len(result.compensated_steps), expected_compensated_count)
        for step in result.steps[:expected_compensated_count]:
            self.assertEqual(step.status, "compensated", f"{step.id} should be compensated")

    def test_03_player_failure_rollback_0(self):
        """player 域失败: 0 步需回滚 (它是第 1 步)"""
        self._assert_rollback(Domain.PLAYER, 0, "step-1-player")

    def test_04_economy_failure_rollback_1(self):
        """economy 域失败: 1 步需回滚 (player)"""
        self._assert_rollback(Domain.ECONOMY, 1, "step-2-economy")

    def test_05_match_failure_rollback_2(self):
        """match 域失败: 2 步需回滚 (player + economy)"""
        self._assert_rollback(Domain.MATCH, 2, "step-3-match")

    def test_06_social_failure_rollback_3(self):
        """social 域失败: 3 步需回滚 (player + economy + match)"""
        self._assert_rollback(Domain.SOCIAL, 3, "step-4-social")

    def test_07_admin_failure_rollback_4(self):
        """admin 域失败: 4 步需回滚 (前 4 域)"""
        self._assert_rollback(Domain.ADMIN, 4, "step-5-admin")


class SagaIdempotencyTest(unittest.TestCase):
    """INV-SG-05 SagaStep.idempotency_key 必填"""

    def test_08_all_steps_have_idempotency_key(self):
        """SAGA_CHAIN 全部 5 step 必含 idempotency_key"""
        for step in SAGA_CHAIN:
            self.assertIsNotNone(step.idempotency_key)
            self.assertIn("saga-", step.idempotency_key)
            self.assertIn(step.domain.value, step.idempotency_key + step.id, "idempotency_key 应含 domain")

    def test_09_idempotency_keys_unique(self):
        """SAGA_CHAIN 5 step idempotency_key 互不重复"""
        keys = [s.idempotency_key for s in SAGA_CHAIN]
        self.assertEqual(len(keys), len(set(keys)), f"duplicate keys: {keys}")


class SagaAuditTest(unittest.TestCase):
    """audit_log 必填 (per docs/automation-design.md §3.4)"""

    def test_10_audit_log_written(self):
        """saga 跑完 audit_log 文件存在 + 含 1 行 JSON"""
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            audit_log = Path(tmpdir) / "test-saga.log"
            saga = SagaE2E(fail_domain=None, audit_log=audit_log)
            saga.run()
            self.assertTrue(audit_log.exists(), "audit_log should exist")
            content = audit_log.read_text(encoding="utf-8")
            self.assertIn("run_saga", content, "audit_log 应含 run_saga action")
            self.assertIn("saga-e2e", content, "audit_log 应含 phase")


if __name__ == "__main__":
    unittest.main(verbosity=2)
