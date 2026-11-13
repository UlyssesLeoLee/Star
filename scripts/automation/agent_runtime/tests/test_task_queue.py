#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_task_queue — SQLite WAL 任务队列 (per 03-DD §5.1/§5.2 + §2.2)。"""

import unittest

from automation.agent_runtime.models import Priority, Task
from automation.agent_runtime.task_queue import TaskQueue, TaskQueueError


def make_task(queue, brief="b", agent_type="SA-01", tenant_id="t1", priority=Priority.Normal,
              created_at=None):
    task = Task.create(brief=brief, agent_type=agent_type, tenant_id=tenant_id, priority=priority)
    if created_at is not None:
        task = Task(
            task_id=task.task_id, brief=task.brief, agent_type=task.agent_type,
            tenant_id=task.tenant_id, priority=task.priority, payload=task.payload,
            trace_id=task.trace_id, correlation_id=task.correlation_id,
            created_at=created_at, operation_id=task.operation_id,
        )
    queue.enqueue(task)
    return task


class TestTaskQueue(unittest.TestCase):
    def setUp(self):
        self.queue = TaskQueue(db_path=None)  # :memory:

    def tearDown(self):
        self.queue.close()

    def test_enqueue_dequeue(self):
        t = make_task(self.queue)
        out = self.queue.dequeue(limit=1)
        self.assertEqual(len(out), 1)
        self.assertEqual(out[0].task_id, t.task_id)
        self.assertEqual(self.queue.status(t.task_id), "running")

    def test_dequeue_priority_then_created(self):
        """per 03-DD §2.2: ORDER BY priority ASC, created_at ASC。"""
        low = make_task(self.queue, brief="low", priority=Priority.Low, created_at=1.0)
        crit = make_task(self.queue, brief="crit", priority=Priority.Critical, created_at=5.0)
        norm_a = make_task(self.queue, brief="a", priority=Priority.Normal, created_at=2.0)
        norm_b = make_task(self.queue, brief="b", priority=Priority.Normal, created_at=3.0)
        out = self.queue.dequeue(limit=4)
        self.assertEqual([t.task_id for t in out], [crit.task_id, norm_a.task_id,
                                                    norm_b.task_id, low.task_id])

    def test_mark_done(self):
        t = make_task(self.queue)
        self.queue.dequeue(limit=1)
        self.queue.mark_done(t.task_id)
        self.assertEqual(self.queue.status(t.task_id), "done")

    def test_mark_done_unknown_raises(self):
        with self.assertRaises(TaskQueueError):
            self.queue.mark_done("nope")

    def test_mark_failed_retry_requeues(self):
        t = make_task(self.queue)
        self.queue.dequeue(limit=1)
        status = self.queue.mark_failed(t.task_id, error="boom", retry=True)
        self.assertEqual(status, "pending")
        self.assertEqual(self.queue.status(t.task_id), "pending")

    def test_mark_failed_exhausts_to_dead(self):
        t = make_task(self.queue)
        status = "unknown"
        for _ in range(3):  # max_retry=3: retry_count 0→1→2 (<3 retry), 第 3 次 retry_count+1=3 → dead
            self.queue.dequeue(limit=1)
            status = self.queue.mark_failed(t.task_id, error="boom", retry=True)
        self.assertEqual(status, "dead")
        self.assertEqual(self.queue.status(t.task_id), "dead")

    def test_requeue(self):
        t = make_task(self.queue)
        self.queue.dequeue(limit=1)
        self.queue.requeue(t.task_id)
        self.assertEqual(self.queue.status(t.task_id), "pending")

    def test_cancel(self):
        t = make_task(self.queue)
        self.assertTrue(self.queue.cancel(t.task_id))
        self.assertEqual(self.queue.status(t.task_id), "dead")
        # 已终态 → 不再取消
        self.assertFalse(self.queue.cancel(t.task_id))

    def test_size_and_pending(self):
        make_task(self.queue)
        make_task(self.queue)
        self.assertEqual(self.queue.size(), 2)
        self.queue.dequeue(limit=1)
        self.assertEqual(self.queue.size(), 2)  # pending + running
        self.assertEqual(self.queue.pending_count(), 1)

    def test_purge_expired(self):
        t = make_task(self.queue)
        self.queue.dequeue(limit=1)
        self.queue.mark_done(t.task_id)
        self.assertEqual(self.queue.purge_expired(), 0)  # retention 7 天未到
        got = self.queue.get(t.task_id)
        assert got is not None  # 刚 mark_done, 必存在
        self.assertEqual(got.task_id, t.task_id)

    def test_dlq_row_written(self):
        t = make_task(self.queue)
        for _ in range(3):
            self.queue.dequeue(limit=1)
            self.queue.mark_failed(t.task_id, error="boom", retry=True)
        row = self.queue._conn.execute(
            "SELECT task_id, failure_reason, retry_count FROM dead_letter_queue WHERE task_id=?",
            (t.task_id,),
        ).fetchone()
        self.assertIsNotNone(row)
        self.assertEqual(row[1], "boom")
        self.assertGreaterEqual(row[2], 3)


if __name__ == "__main__":
    unittest.main()
