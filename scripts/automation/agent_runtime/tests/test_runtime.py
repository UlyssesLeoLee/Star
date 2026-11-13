#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_runtime — AgentRuntime facade (per SRS §85 + 03-DD §4.1 UC-01)。"""

import unittest

from automation.agent_runtime.backpressure import QueueFullError, ThrottleError
from automation.agent_runtime.config import RuntimeConfig
from automation.agent_runtime.models import (
    AgentState,
    EventType,
    LifecycleState,
    Task,
)
from automation.agent_runtime.runtime import AgentRuntime
from automation.agent_runtime.runtime_mode import RuntimeMode


class TestAgentRuntimeApi(unittest.TestCase):
    def setUp(self):
        self.rt = AgentRuntime(db_path=None)

    def tearDown(self):
        self.rt.close()

    def test_create_delete_agent(self):
        agent = self.rt.create_agent(tenant_id="t1", agent_type="SA-01")
        got = self.rt.get_agent(agent.identity.agent_id)
        assert got is not None
        self.assertEqual(got.identity.agent_id, agent.identity.agent_id)
        self.assertTrue(self.rt.delete_agent(agent.identity.agent_id))
        self.assertIsNone(self.rt.get_agent(agent.identity.agent_id))
        self.assertFalse(self.rt.delete_agent(agent.identity.agent_id))

    def test_create_agent_starts_warm(self):
        agent = self.rt.create_agent(tenant_id="t1", agent_type="SA-01")
        self.assertEqual(agent.lifecycle.current, LifecycleState.Warm)

    def test_duplicate_agent_rejected(self):
        """per SRS §95 Single Active Ownership 雏形: 同 id 重复注册拒绝。"""
        self.rt.create_agent(tenant_id="t1", agent_type="SA-01", agent_id="dup-1")
        with self.assertRaises(Exception):
            self.rt.create_agent(tenant_id="t1", agent_type="SA-02", agent_id="dup-1")
        self.rt.delete_agent("dup-1")

    def test_suspend_resume_cancel(self):
        agent = self.rt.create_agent(tenant_id="t1", agent_type="SA-01")
        aid = agent.identity.agent_id
        self.assertTrue(self.rt.suspend_agent(aid))
        state = self.rt.get_agent_state(aid)
        assert state is not None
        self.assertEqual(state.current, AgentState.Suspended)
        self.assertTrue(self.rt.resume_agent(aid))
        state = self.rt.get_agent_state(aid)
        assert state is not None
        self.assertEqual(state.current, AgentState.Scheduled)
        self.assertTrue(self.rt.cancel_agent(aid))
        state = self.rt.get_agent_state(aid)
        assert state is not None
        self.assertEqual(state.current, AgentState.Cancelled)
        self.assertFalse(self.rt.cancel_agent("ghost"))

    def test_send_event_restores_cold_agent(self):
        agent = self.rt.create_agent(tenant_id="t1", agent_type="SA-01")
        agent.lifecycle.current = LifecycleState.Cold
        from automation.agent_runtime.models import AgentEvent
        event = AgentEvent.create(source_agent_id=None, target_agent_id=agent.identity.agent_id,
                                  tenant_id="t1", event_type=EventType.SystemEvent)
        self.rt.send_event(event)
        self.assertEqual(agent.lifecycle.current, LifecycleState.Warm)
        self.assertEqual(self.rt.event_bus.published_count(agent.identity.agent_id), 1)


class TestAgentRuntimeDispatch(unittest.TestCase):
    def _make_runtime(self, **cfg_kwargs):
        return AgentRuntime(config=RuntimeConfig(**cfg_kwargs).validate(), db_path=None)

    def test_dispatch_happy_path(self):
        rt = self._make_runtime(rate_limit_capacity=100)
        try:
            task = Task.create(brief="b1", agent_type="SA-01", tenant_id="t1")
            rt.dispatch(task)
            self.assertEqual(rt.queue.status(task.task_id), "running")  # dequeue 后 HOT
            self.assertEqual(rt.scheduler.hot_count(), 1)
            rt.queue.mark_done(task.task_id)
            self.assertEqual(rt.observer.dispatched, 1)
        finally:
            rt.close()

    def test_dispatch_rate_limited(self):
        rt = self._make_runtime(rate_limit_capacity=1, rate_limit_refill_per_sec=0.0)
        try:
            rt.dispatch(Task.create(brief="b", agent_type="SA-01", tenant_id="t1"))
            with self.assertRaises(RuntimeError):
                rt.dispatch(Task.create(brief="b2", agent_type="SA-01", tenant_id="t1"))
        finally:
            rt.close()

    def test_dispatch_queue_full_reject(self):
        rt = self._make_runtime(max_queue_size=1, overflow_policy="Reject",
                                rate_limit_capacity=100)
        try:
            rt.dispatch(Task.create(brief="b1", agent_type="SA-01", tenant_id="t1"))
            with self.assertRaises(QueueFullError):
                rt.dispatch(Task.create(brief="b2", agent_type="SA-01", tenant_id="t1"))
        finally:
            rt.close()

    def test_dispatch_throttle_default(self):
        rt = self._make_runtime(max_queue_size=1, rate_limit_capacity=100)
        try:
            rt.dispatch(Task.create(brief="b1", agent_type="SA-01", tenant_id="t1"))
            with self.assertRaises(ThrottleError) as ctx:
                rt.dispatch(Task.create(brief="b2", agent_type="SA-01", tenant_id="t1"))
            self.assertEqual(ctx.exception.retry_after, 60.0)
        finally:
            rt.close()


class TestAgentRuntimeModeAndMetrics(unittest.TestCase):
    def test_mode_switch_to_ecs(self):
        """per SRS §7.3: 12 resident 持续 30s → ECS。"""
        rt = AgentRuntime(db_path=None)
        try:
            for i in range(12):
                rt.create_agent(tenant_id="t1", agent_type="SA-01")
            self.assertEqual(rt.tick(now=0.0), RuntimeMode.Lightweight)
            self.assertEqual(rt.tick(now=29.0), RuntimeMode.Lightweight)
            self.assertEqual(rt.tick(now=30.0), RuntimeMode.Ecs)
            self.assertTrue(rt.ecs_mode_active())
        finally:
            rt.close()

    def test_mode_exit_ecs_300s(self):
        rt = AgentRuntime(db_path=None)
        try:
            agents = [rt.create_agent(tenant_id="t1", agent_type="SA-01") for _ in range(12)]
            self.assertEqual(rt.tick(now=0.0), RuntimeMode.Lightweight)  # 计时起点
            self.assertEqual(rt.tick(now=30.0), RuntimeMode.Ecs)
            for a in agents[:4]:  # 掉到 8
                rt.delete_agent(a.identity.agent_id)
            self.assertEqual(rt.tick(now=31.0), RuntimeMode.Ecs)
            self.assertEqual(rt.tick(now=330.0), RuntimeMode.Ecs)
            self.assertEqual(rt.tick(now=331.0), RuntimeMode.Lightweight)
        finally:
            rt.close()

    def test_metrics_snapshot(self):
        rt = AgentRuntime(db_path=None)
        try:
            for i in range(5):
                rt.create_agent(tenant_id="t1", agent_type="SA-01")
            m = rt.get_runtime_metrics()
            self.assertEqual(m.resident_agent_count, 5)
            self.assertEqual(m.warm_agent_count, 5)
            self.assertEqual(m.runtime_mode, "Lightweight")
            self.assertEqual(m.errors, 0)
            self.assertIsNotNone(m.to_dict())
        finally:
            rt.close()

    def test_lifecycle_sweep_via_tick(self):
        rt = AgentRuntime(db_path=None)
        try:
            agent = rt.create_agent(tenant_id="t1", agent_type="SA-01")
            rt.lifecycle.activate(agent, now=0.0)
            rt.tick(now=61.0)  # HOT → WARM
            self.assertEqual(agent.lifecycle.current, LifecycleState.Warm)
            rt.tick(now=62.0 + 601.0)  # WARM → COLD
            self.assertEqual(agent.lifecycle.current, LifecycleState.Cold)
        finally:
            rt.close()


if __name__ == "__main__":
    unittest.main()
