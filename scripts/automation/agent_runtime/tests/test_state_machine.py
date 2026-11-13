#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_state_machine — Agent 12 状态机 + Lifecycle HOT/WARM/COLD (per 03-DD §3)。"""

import unittest

from automation.agent_runtime.models import Agent, AgentState, LifecycleState
from automation.agent_runtime.state_machine import (
    AGENT_TRANSITIONS,
    AgentStateMachine,
    LifecycleMachine,
)


class TestAgentStateMachine(unittest.TestCase):
    def setUp(self):
        self.agent = Agent.create(tenant_id="t1", agent_type="SA-01")

    def test_happy_path(self):
        """per 03-DD §3.1 主链: Idle→Ready→Scheduled→Planning→Waiting→Processing→Completed→Idle。"""
        path = [
            AgentState.Ready,
            AgentState.Scheduled,
            AgentState.Planning,
            AgentState.WaitingLlm,
            AgentState.Processing,
            AgentState.Completed,
            AgentState.Idle,
        ]
        for target in path:
            self.assertTrue(
                AgentStateMachine.transition(self.agent, target),
                f"{self.agent.state.prev} -> {target} 应合法",
            )
        self.assertEqual(self.agent.state.current, AgentState.Idle)
        self.assertEqual(self.agent.state.prev, AgentState.Completed)

    def test_three_wait_branches(self):
        for wait in (AgentState.WaitingLlm, AgentState.WaitingTool, AgentState.WaitingEvent):
            agent = Agent.create(tenant_id="t1", agent_type="SA-01")
            for target in (AgentState.Ready, AgentState.Scheduled, AgentState.Planning, wait):
                self.assertTrue(AgentStateMachine.transition(agent, target))
            self.assertEqual(agent.state.current, wait)

    def test_failed_retry_returns_ready(self):
        for target in (AgentState.Ready, AgentState.Scheduled, AgentState.Planning):
            self.assertTrue(AgentStateMachine.transition(self.agent, target))
        self.assertTrue(AgentStateMachine.fail(self.agent))
        self.assertEqual(self.agent.state.current, AgentState.Failed)
        self.assertEqual(self.agent.state.retry_count, 1)
        self.assertTrue(AgentStateMachine.retry(self.agent))
        self.assertEqual(self.agent.state.current, AgentState.Ready)

    def test_suspend_resume(self):
        self.assertTrue(AgentStateMachine.transition(self.agent, AgentState.Ready))
        self.assertTrue(AgentStateMachine.suspend(self.agent))
        self.assertEqual(self.agent.state.current, AgentState.Suspended)
        self.assertTrue(AgentStateMachine.resume(self.agent))
        self.assertEqual(self.agent.state.current, AgentState.Scheduled)

    def test_cancel_terminal(self):
        self.assertTrue(AgentStateMachine.transition(self.agent, AgentState.Ready))
        self.assertTrue(AgentStateMachine.cancel(self.agent))
        self.assertEqual(self.agent.state.current, AgentState.Cancelled)
        # 终态不可再转 (per 03-DD §3.1)
        for target in AgentState:
            self.assertFalse(AgentStateMachine.transition(self.agent, target))

    def test_illegal_transitions_rejected(self):
        self.assertFalse(AgentStateMachine.transition(self.agent, AgentState.Processing))
        self.assertFalse(AgentStateMachine.transition(self.agent, AgentState.Completed))
        # Idle → WaitingLlm 非法
        self.assertFalse(AgentStateMachine.transition(self.agent, AgentState.WaitingLlm))
        # 状态不变
        self.assertEqual(self.agent.state.current, AgentState.Idle)

    def test_transition_table_covers_12_states(self):
        self.assertEqual(len(AGENT_TRANSITIONS), 12)


class TestLifecycleMachine(unittest.TestCase):
    def setUp(self):
        self.machine = LifecycleMachine(hot_timeout=60.0, cold_idle_timeout=600.0)
        self.agent = Agent.create(tenant_id="t1", agent_type="SA-01")

    def test_activate_to_hot(self):
        self.machine.activate(self.agent, now=100.0)
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Hot)

    def test_hot_to_warm_after_timeout(self):
        self.machine.activate(self.agent, now=100.0)
        self.machine.sweep(self.agent, now=100.0 + 61.0)
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Warm)

    def test_hot_stays_hot_within_timeout(self):
        self.machine.activate(self.agent, now=100.0)
        self.assertIsNone(self.machine.sweep(self.agent, now=100.0 + 59.0))
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Hot)

    def test_warm_to_cold_after_idle(self):
        self.machine.activate(self.agent, now=0.0)
        self.machine.sweep(self.agent, now=61.0)  # HOT → WARM
        self.machine.sweep(self.agent, now=61.0 + 601.0)  # WARM → COLD
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Cold)

    def test_cold_restore_to_warm(self):
        self.machine.activate(self.agent, now=0.0)
        self.machine.sweep(self.agent, now=61.0)
        self.machine.sweep(self.agent, now=662.0)
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Cold)
        self.machine.restore(self.agent, now=700.0)
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Warm)
        self.machine.activate(self.agent, now=701.0)  # Schedule → HOT
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Hot)

    def test_touch_keeps_hot(self):
        self.machine.activate(self.agent, now=0.0)
        self.machine.touch(self.agent, now=59.0)
        self.assertIsNone(self.machine.sweep(self.agent, now=118.0))
        self.assertEqual(self.agent.lifecycle.current, LifecycleState.Hot)


if __name__ == "__main__":
    unittest.main()
