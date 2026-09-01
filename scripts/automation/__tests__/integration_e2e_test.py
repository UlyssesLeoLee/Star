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
    ENDPOINTS_BY_PROVIDER,
)


def _endpoint(provider: str, name: str):
    return [e for e in ENDPOINTS_BY_PROVIDER[provider] if e.name == name][0]


class OpenClawE2ETest(unittest.TestCase):
    """B.5 OpenClaw 5 endpoint 测试"""

    def setUp(self):
        self.config = EndpointConfig.from_provider("openclaw")
        self.e2e = IntegrationE2E(self.config, dry_run=True)

    def test_01_openclaw_agents(self):
        """endpoint 1: /v1/agents (GET + POST)"""
        self.e2e.results = []
        agents = _endpoint("openclaw", "agents")
        for method in agents.methods:
            result = self.e2e.run_case(agents, method)
            self.assertTrue(result.success, f"agents {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("cost", result.response_preview)
            self.assertIn("token_usage", result.response_preview)
            self.assertIn("/v1/agents", str(agents.path))

    def test_02_openclaw_sessions(self):
        """endpoint 2: /v1/sessions (GET + POST + PUT + DELETE 全 4 method)"""
        self.e2e.results = []
        sessions = _endpoint("openclaw", "sessions")
        for method in sessions.methods:
            result = self.e2e.run_case(sessions, method)
            self.assertTrue(result.success, f"sessions {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("/v1/sessions", str(sessions.path))

    def test_03_openclaw_messages(self):
        """endpoint 3: /v1/messages (GET + POST)"""
        self.e2e.results = []
        messages = _endpoint("openclaw", "messages")
        for method in messages.methods:
            result = self.e2e.run_case(messages, method)
            self.assertTrue(result.success, f"messages {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_04_openclaw_tools_invoke(self):
        """endpoint 4: /v1/tools/invoke (POST)"""
        self.e2e.results = []
        tools_invoke = _endpoint("openclaw", "tools_invoke")
        for method in tools_invoke.methods:
            result = self.e2e.run_case(tools_invoke, method)
            self.assertTrue(result.success, f"tools_invoke {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_05_openclaw_cost(self):
        """endpoint 5: /v1/cost (GET)"""
        self.e2e.results = []
        cost = _endpoint("openclaw", "cost")
        for method in cost.methods:
            result = self.e2e.run_case(cost, method)
            self.assertTrue(result.success, f"cost {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("cost", result.response_preview)

    def test_summary_runs_all_10_cases(self):
        """summary 跑全部 5 endpoint × 各自 methods = 10 case"""
        results = self.e2e.run_all()
        self.assertEqual(len(results), 10)
        for r in results:
            self.assertTrue(r.success, f"{r.endpoint} {r.method} failed")
            self.assertEqual(r.status_code, 200)


class HermesE2ETest(unittest.TestCase):
    """B.6 Hermes 5 endpoint 测试 (per docs/automation-design.md §4.1)"""

    def setUp(self):
        self.config = EndpointConfig.from_provider("hermes")
        self.e2e = IntegrationE2E(self.config, dry_run=True)

    def test_01_hermes_agents(self):
        """endpoint 1: /v2/hermes/agents (GET + POST)"""
        self.e2e.results = []
        agents = _endpoint("hermes", "agents")
        for method in agents.methods:
            result = self.e2e.run_case(agents, method)
            self.assertTrue(result.success, f"hermes agents {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("/v2/hermes/agents", str(agents.path))

    def test_02_hermes_sessions(self):
        """endpoint 2: /v2/hermes/sessions (GET + POST + PUT + DELETE 全 4 method)"""
        self.e2e.results = []
        sessions = _endpoint("hermes", "sessions")
        for method in sessions.methods:
            result = self.e2e.run_case(sessions, method)
            self.assertTrue(result.success, f"hermes sessions {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("/v2/hermes/sessions", str(sessions.path))

    def test_03_hermes_messages(self):
        """endpoint 3: /v2/hermes/messages (GET + POST)"""
        self.e2e.results = []
        messages = _endpoint("hermes", "messages")
        for method in messages.methods:
            result = self.e2e.run_case(messages, method)
            self.assertTrue(result.success, f"hermes messages {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_04_hermes_tools_invoke(self):
        """endpoint 4: /v2/hermes/tools/invoke (POST)"""
        self.e2e.results = []
        tools_invoke = _endpoint("hermes", "tools_invoke")
        for method in tools_invoke.methods:
            result = self.e2e.run_case(tools_invoke, method)
            self.assertTrue(result.success, f"hermes tools_invoke {method} failed")
            self.assertEqual(result.status_code, 200)

    def test_05_hermes_cost(self):
        """endpoint 5: /v2/hermes/cost (GET)"""
        self.e2e.results = []
        cost = _endpoint("hermes", "cost")
        for method in cost.methods:
            result = self.e2e.run_case(cost, method)
            self.assertTrue(result.success, f"hermes cost {method} failed")
            self.assertEqual(result.status_code, 200)
            self.assertIn("cost", result.response_preview)

    def test_summary_runs_all_10_cases(self):
        """summary 跑全部 5 endpoint × 各自 methods = 10 case"""
        results = self.e2e.run_all()
        self.assertEqual(len(results), 10)
        for r in results:
            self.assertTrue(r.success, f"{r.endpoint} {r.method} failed")
            self.assertEqual(r.status_code, 200)
            self.assertEqual(r.provider, "hermes")


if __name__ == "__main__":
    unittest.main(verbosity=2)
