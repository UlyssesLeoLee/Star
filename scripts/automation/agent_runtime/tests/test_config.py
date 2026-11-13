#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_config — RuntimeConfig Fail Safe 校验 (per SRS §100-§101)。"""

import unittest

from automation.agent_runtime.config import ConfigError, OVERFLOW_POLICIES, RuntimeConfig


class TestRuntimeConfig(unittest.TestCase):
    def test_defaults_valid(self):
        cfg = RuntimeConfig().validate()
        self.assertEqual(cfg.ecs_enable_threshold, 12)
        self.assertEqual(cfg.ecs_disable_threshold, 8)
        self.assertEqual(cfg.max_hot_agents, 100)
        self.assertEqual(cfg.overflow_policy, "Throttle")

    def test_default_hysteresis_nonempty(self):
        cfg = RuntimeConfig().validate()
        self.assertGreater(cfg.ecs_enable_threshold - 3, cfg.ecs_disable_threshold)

    def test_zero_hot_agents_rejected(self):
        with self.assertRaises(ConfigError):
            RuntimeConfig(max_hot_agents=0).validate()

    def test_unbounded_queue_rejected(self):
        with self.assertRaises(ConfigError):
            RuntimeConfig(max_queue_size=0).validate()

    def test_bad_overflow_policy_rejected(self):
        with self.assertRaises(ConfigError):
            RuntimeConfig(overflow_policy="Unlimited").validate()

    def test_empty_hysteresis_rejected(self):
        with self.assertRaises(ConfigError):
            RuntimeConfig(ecs_enable_threshold=10, ecs_disable_threshold=9).validate()

    def test_unlimited_concurrency_rejected(self):
        from dataclasses import replace

        for name in ("max_llm_requests", "max_tool_requests", "max_mcp_requests",
                     "max_rag_requests", "max_context_loads"):
            with self.assertRaises(ConfigError):
                replace(RuntimeConfig(), **{name: 0}).validate()

    def test_with_override(self):
        cfg = RuntimeConfig().validate().with_override(max_hot_agents=200)
        self.assertEqual(cfg.max_hot_agents, 200)
        self.assertEqual(cfg.ecs_enable_threshold, 12)

    def test_overflow_policies_set(self):
        self.assertEqual(
            OVERFLOW_POLICIES,
            ("Reject", "Delay", "DropLow", "Persist", "Throttle"),
        )


if __name__ == "__main__":
    unittest.main()
