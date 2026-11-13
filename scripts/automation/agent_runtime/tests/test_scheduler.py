#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_scheduler — 调度器 (per SRS §42-§44 + 03-DD §6.1/§6.4)。"""

import unittest

from automation.agent_runtime.models import Priority, Task
from automation.agent_runtime.scheduler import Scheduler, ScheduleResult


def make_task(task_id, tenant="t1", priority=Priority.Normal):
    return Task(
        task_id=task_id,
        brief="b",
        agent_type="SA-01",
        tenant_id=tenant,
        priority=priority,
    )


class TestScheduler(unittest.TestCase):
    def test_max_hot_agents_must_be_positive(self):
        with self.assertRaises(ValueError):
            Scheduler(max_hot_agents=0)

    def test_priority_order(self):
        s = Scheduler(max_hot_agents=10)
        tasks = [
            make_task("low", priority=Priority.Low),
            make_task("crit", priority=Priority.Critical),
            make_task("norm", priority=Priority.Normal),
            make_task("high", priority=Priority.High),
        ]
        for t in tasks:
            s.register_task(t)
        result = s.schedule(ready_tasks=tasks)
        # Critical → High → Normal → Low (per 03-DD §6.1)
        self.assertEqual(result.scheduled,
                         [t.task_id for t in sorted(tasks, key=lambda t: t.priority)])

    def test_hot_slot_limit(self):
        """per 03-DD §6.4: hot_count < max_hot_agents, 否则排队。"""
        s = Scheduler(max_hot_agents=2)
        tasks = [make_task(f"t{i}") for i in range(5)]
        for t in tasks:
            s.register_task(t)
        result = s.schedule(ready_tasks=tasks)
        self.assertEqual(len(result.scheduled), 2)
        self.assertEqual(s.hot_count(), 2)
        self.assertEqual(s.ready_queue_depth(), 3)  # 未调度保留在 Ready Queue
        # 释放一个 → 下一轮可续
        s.release_hot(result.scheduled[0])
        result2 = s.schedule()
        self.assertEqual(len(result2.scheduled), 1)
        reason2 = result2.reason
        assert reason2 is not None  # HOT 满触发
        self.assertIn("HotSlotFull", reason2)

    def test_hot_slot_usage(self):
        s = Scheduler(max_hot_agents=4)
        t = make_task("t1")
        s.register_task(t)
        s.schedule(ready_tasks=[t])
        self.assertAlmostEqual(s.hot_slot_usage(), 0.25)

    def test_fair_share_round_robin(self):
        """per SRS §57: Normal/Low per-tenant round-robin, 防单 tenant 占满。"""
        s = Scheduler(max_hot_agents=10, fair_share=2)
        tasks = [make_task(f"a{i}", tenant="A") for i in range(5)] + \
                [make_task(f"b{i}", tenant="B") for i in range(5)]
        for t in tasks:
            s.register_task(t)
        result = s.schedule(ready_tasks=tasks)
        # A 取 2 → B 取 2 → (deferred 重排, 单轮内即结束)
        self.assertEqual(len(result.scheduled), 4)
        scheduled_tenants = [tid[0] for tid in result.scheduled]
        self.assertEqual(scheduled_tenants, ["a", "a", "b", "b"])

    def test_critical_bypasses_fair_share(self):
        s = Scheduler(max_hot_agents=10, fair_share=1)
        tasks = [
            make_task("a1", tenant="A"),
            make_task("a2", tenant="A"),
            make_task("c1", tenant="A", priority=Priority.Critical),
        ]
        for t in tasks:
            s.register_task(t)
        result = s.schedule(ready_tasks=tasks)
        # Critical 优先 (priority ASC), 且不受 fair share 限制;
        # Normal 超 fair_share(1) → a2 重排, 一轮内 a1 之后即停
        self.assertEqual(result.scheduled, ["c1", "a1"])
        self.assertEqual(s.ready_queue_depth(), 1)  # a2 留在 Ready Queue

    def test_drop_low_priority(self):
        s = Scheduler(max_hot_agents=10)
        tasks = [make_task("t1", priority=Priority.Background),
                 make_task("t2", priority=Priority.Normal)]
        for t in tasks:
            s.register_task(t)
            s.enqueue_ready(t)
        dropped = s.drop_low_priority(1)
        self.assertEqual(dropped, ["t1"])  # Background 最低

    def test_drain_deferred(self):
        s = Scheduler(max_hot_agents=1)
        tasks = [make_task(f"t{i}") for i in range(3)]
        for t in tasks:
            s.register_task(t)
        s.schedule(ready_tasks=tasks)
        deferred = s.drain_deferred()
        self.assertEqual(len(deferred), 2)
        self.assertEqual(s.ready_queue_depth(), 0)

    def test_schedule_result_shape(self):
        r = ScheduleResult()
        self.assertEqual(r.scheduled, [])
        self.assertEqual(r.deferred, [])
        self.assertIsNone(r.reason)


if __name__ == "__main__":
    unittest.main()
