#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/__tests__/integration_e2e_test.py — B.5/B.6 共享 e2e 测试
(per docs/automation-design.md v0.1 §4.1 B.5/B.6 5 测试)

5 测试 (每个 endpoint 1 测试, 跑 GET list/retrieve + POST create):
1. test_openclaw_agents
2. test_openclaw_sessions
3. test_openclaw_messages
4. test_openclaw_tools_invoke
5. test_openclaw_cost

约束 (per 守门 #1 v1):
- 标准库 only (unittest 模拟)
- sys.path 加 scripts/ 让 import automation 找到
"""

import sys
import unittest
from pathlib import Path

# sys.path 加 scripts/ (per scripts/automation/smoke_test.py 同款)
# __tests__/integration_e2e_test.py -> scripts/automation/__tests__/ -> scripts/automation/ -> scripts/
SCRIPTS_DIR = Path(__file__).resolve().parent.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.integration_e2e import (
    IntegrationE2E,
    EndpointConfig,
    ENDPOINTS,
)


class OpenClawE2ETest(unittest.TestCase):
    """B.5 OpenClaw 5 endpoint 测试"""

    def setUp(self):
        self.config = EndpointConfig.from_provider("openclaw")
        self.e2e = IntegrationE2E(self.config, dry_run=True)

    def test_01_openclaw_agents(self):
        """endpoint 1: /agents (GET + POST)"""
        self.e2e.results = []
        agents = [e for e in ENDPOINTS if e.name == "agents"][0]
        for method in agents.methods:
            result = self.e2e.run_case(agents, method)
            self.assertTrue(result.success, f"agents {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("cost", result.response_preview)
            self.assertIn("token_usage", result.response_preview)

    def test_02_openclaw_sessions(self):
        """endpoint 2: /sessions (GET + POST + PUT + DELETE 全 4 method)"""
        self.e2e.results = []
        sessions = [e for e in ENDPOINTS if e.name == "sessions"][0]
        for method in sessions.methods:
            result = self.e2e.run_case(sessions, method)
            self.assertTrue(result.success, f"sessions {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_03_openclaw_messages(self):
        """endpoint 3: /messages (GET + POST)"""
        self.e2e.results = []
        messages = [e for e in ENDPOINTS if e.name == "messages"][0]
        for method in messages.methods:
            result = self.e2e.run_case(messages, method)
            self.assertTrue(result.success, f"messages {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_04_openclaw_tools_invoke(self):
        """endpoint 4: /tools/invoke (POST)"""
        self.e2e.results = []
        tools_invoke = [e for e in ENDPOINTS if e.name == "tools_invoke"][0]
        for method in tools_invoke.methods:
            result = self.e2e.run_case(tools_invoke, method)
            self.assertTrue(result.success, f"tools_invoke {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_05_openclaw_cost(self):
        """endpoint 5: /cost (GET)"""
        self.e2e.results = []
        cost = [e for e in ENDPOINTS if e.name == "cost"][0]
        for method in cost.methods:
            result = self.e2e.run_case(cost, method)
            self.assertTrue(result.success, f"cost {method} failed")
            self.assertEqual(result.status_code, 200)
            # cost 响应必含 cost 字段
            self.assertIn("cost", result.response_preview)

    def test_summary_runs_all_20_cases(self):
        """summary 跑全部 5 × 4 = 20 case"""
        results = self.e2e.run_all()
        self.assertEqual(len(results), 10)  # 5 endpoint, 各自 method 数
        for r in results:
            self.assertTrue(r.success, f"{r.endpoint} {r.method} failed")
            self.assertEqual(r.status_code, 200)


if __name__ == "__main__":
    unittest.main(verbosity=2)
