#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @cypher schema=1 source_sha256=e23b35dabe39f4ac08e42f707b3e5bf550ec97b14e526da395e403dbbbbf9c53
# MERGE (self:File {path:"scripts/automation/agent_runtime/task_queue.py"})
# @endcypher
"""agent_runtime.task_queue — L0 SQLite WAL 任务队列 (per 03-DD §5.1/§5.2)。

Schema (per 03-DD §5.1 task_queue, W 类 per 守门 #13 DB 三類横展開):
    task_id / brief / agent_type / tenant_id / priority / status /
    retry_count / max_retry / payload / created_at / started_at /
    completed_at / trace_id / correlation_id / error / retention_until

Dead Letter Queue (per 03-DD §5.2, W 类, 30 天 retention)。

PRAGMA (per 03-DD §5.1): journal_mode=WAL / synchronous=NORMAL /
wal_autocheckpoint=1000。内存库 (:memory:) 不适用 WAL, 静默跳过 (per 守门 #12 缺标比错标)。

约束: 标准库 only (sqlite3 / threading / json)。
"""

from __future__ import annotations

import json
import sqlite3
import threading
import time
from typing import List, Optional

from .models import Priority, Task

#: 队列状态 (per 03-DD §5.1 status 列)
QUEUE_STATUS = ("pending", "running", "done", "failed", "dead")

#: task_queue DDL (per 03-DD §5.1, 守门 #13 W 类: retention_until 必携)
DDL_TASK_QUEUE = """
CREATE TABLE IF NOT EXISTS task_queue (
    task_id          TEXT PRIMARY KEY,
    brief            TEXT NOT NULL,
    agent_type       TEXT NOT NULL,
    tenant_id        TEXT NOT NULL,
    priority         INTEGER NOT NULL,
    status           TEXT NOT NULL,
    retry_count      INTEGER NOT NULL DEFAULT 0,
    max_retry        INTEGER NOT NULL DEFAULT 3,
    payload          TEXT,
    created_at       REAL NOT NULL,
    started_at       REAL,
    completed_at     REAL,
    trace_id         TEXT,
    correlation_id   TEXT,
    error            TEXT,
    retention_until  REAL NOT NULL
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS idx_task_queue_status_priority
    ON task_queue (status, priority, created_at);
CREATE INDEX IF NOT EXISTS idx_task_queue_tenant ON task_queue (tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_task_queue_trace ON task_queue (trace_id);
"""

#: dead_letter_queue DDL (per 03-DD §5.2, W 类)
DDL_DEAD_LETTER = """
CREATE TABLE IF NOT EXISTS dead_letter_queue (
    task_id          TEXT PRIMARY KEY,
    original_task    TEXT NOT NULL,
    failure_reason   TEXT NOT NULL,
    retry_count      INTEGER NOT NULL,
    first_failed_at  REAL NOT NULL,
    last_failed_at   REAL NOT NULL,
    retention_until  REAL NOT NULL
) WITHOUT ROWID;
"""


class TaskQueueError(RuntimeError):
    """任务队列操作异常。"""


class TaskQueue:
    """SQLite WAL 持久化任务队列 (per 03-DD §5.1)。

    thread-safe: 单 Connection + Lock (SQLite 串行写, WAL 并发读)。
    """

    def __init__(
        self,
        db_path: Optional[str] = None,
        retention_seconds: int = 7 * 86400,
        dlq_retention_seconds: int = 30 * 86400,
    ):
        self.db_path = db_path  # None → :memory:
        self.retention_seconds = retention_seconds
        self.dlq_retention_seconds = dlq_retention_seconds
        self._lock = threading.Lock()
        self._conn = sqlite3.connect(db_path if db_path else ":memory:", check_same_thread=False)
        with self._lock:
            self._conn.execute("PRAGMA journal_mode=WAL")  # :memory: 静默忽略失败
            self._conn.execute("PRAGMA synchronous=NORMAL")
            self._conn.execute("PRAGMA wal_autocheckpoint=1000")
            self._conn.executescript(DDL_TASK_QUEUE)
            self._conn.executescript(DDL_DEAD_LETTER)
            self._conn.commit()

    # === 核心 API (per 02-BD §3.1 TaskQueue 接口) ===

    def enqueue(self, task: Task) -> str:
        """INSERT pending (per 03-DD §2.2 enqueue)。返回 task_id。"""
        if task.priority not in Priority:
            raise TaskQueueError(f"非法 priority: {task.priority!r}")
        with self._lock:
            self._conn.execute(
                """INSERT INTO task_queue
                   (task_id, brief, agent_type, tenant_id, priority, status,
                    retry_count, max_retry, payload, created_at, trace_id,
                    correlation_id, retention_until)
                   VALUES (?, ?, ?, ?, ?, 'pending', 0, 3, ?, ?, ?, ?, ?)""",
                (
                    task.task_id,
                    task.brief,
                    task.agent_type,
                    task.tenant_id,
                    int(task.priority),
                    task.payload,
                    task.created_at,
                    task.trace_id,
                    task.correlation_id,
                    time.time() + self.retention_seconds,
                ),
            )
            self._conn.commit()
        return task.task_id

    def dequeue(self, limit: int = 1) -> List[Task]:
        """取 pending → 原子置 running (per 03-DD §2.2 dequeue:
        ORDER BY priority ASC, created_at ASC LIMIT ?)。"""
        with self._lock:
            rows = self._conn.execute(
                """SELECT task_id, brief, agent_type, tenant_id, priority,
                          payload, trace_id, correlation_id, created_at
                   FROM task_queue WHERE status = 'pending'
                   ORDER BY priority ASC, created_at ASC LIMIT ?""",
                (limit,),
            ).fetchall()
            if not rows:
                return []
            ids = [r[0] for r in rows]
            now = time.time()
            self._conn.executemany(
                "UPDATE task_queue SET status='running', started_at=? WHERE task_id=?",
                [(now, i) for i in ids],
            )
            self._conn.commit()
        return [self._row_to_task(r) for r in rows]

    def mark_done(self, task_id: str) -> None:
        """UPDATE status='done' (per 03-DD §2.2 mark_done)。"""
        with self._lock:
            cur = self._conn.execute(
                "UPDATE task_queue SET status='done', completed_at=? WHERE task_id=?",
                (time.time(), task_id),
            )
            if cur.rowcount == 0:
                raise TaskQueueError(f"mark_done: task {task_id} 不存在或状态非法")
            self._conn.commit()

    def mark_failed(self, task_id: str, error: str = "", retry: bool = True) -> str:
        """失败处理 (per 03-DD §2.2 mark_failed): retry=True → requeue
        (retry_count+1, 回 pending), 超 max_retry → dead + DLQ。返回新状态。"""
        with self._lock:
            row = self._conn.execute(
                "SELECT retry_count, max_retry FROM task_queue WHERE task_id=? AND status='running'",
                (task_id,),
            ).fetchone()
            if row is None:
                raise TaskQueueError(f"mark_failed: task {task_id} 不存在或非 running")
            retry_count, max_retry = row
            if retry and retry_count + 1 < max_retry:
                self._conn.execute(
                    """UPDATE task_queue SET status='pending',
                       retry_count=retry_count+1, error=? WHERE task_id=?""",
                    (error, task_id),
                )
                new_status = "pending"
            else:
                self._conn.execute(
                    """UPDATE task_queue SET status='dead',
                       retry_count=retry_count+1, error=?,
                       completed_at=? WHERE task_id=?""",
                    (error, time.time(), task_id),
                )
                self._to_dlq(task_id, error, retry_count + 1)
                new_status = "dead"
            self._conn.commit()
        return new_status

    def requeue(self, task_id: str) -> None:
        """running → pending (公平调度 re-enqueue, per 03-DD §6.1)。"""
        with self._lock:
            cur = self._conn.execute(
                "UPDATE task_queue SET status='pending', started_at=NULL WHERE task_id=?",
                (task_id,),
            )
            if cur.rowcount == 0:
                raise TaskQueueError(f"requeue: task {task_id} 不存在")
            self._conn.commit()

    def cancel(self, task_id: str) -> bool:
        """取消 pending/running 任务 (per SRS §50)。"""
        with self._lock:
            cur = self._conn.execute(
                "UPDATE task_queue SET status='dead', completed_at=?, error='cancelled' "
                "WHERE task_id=? AND status IN ('pending', 'running')",
                (time.time(), task_id),
            )
            self._conn.commit()
            return cur.rowcount > 0

    # === 查询 ===

    def get(self, task_id: str) -> Optional[Task]:
        with self._lock:
            row = self._conn.execute(
                """SELECT task_id, brief, agent_type, tenant_id, priority,
                          payload, trace_id, correlation_id, created_at
                   FROM task_queue WHERE task_id=?""",
                (task_id,),
            ).fetchone()
        return self._row_to_task(row) if row else None

    def size(self) -> int:
        """队列当前长度 (pending + running, per Backpressure 输入)。"""
        with self._lock:
            return self._conn.execute(
                "SELECT COUNT(*) FROM task_queue WHERE status IN ('pending', 'running')"
            ).fetchone()[0]

    def pending_count(self) -> int:
        with self._lock:
            return self._conn.execute(
                "SELECT COUNT(*) FROM task_queue WHERE status='pending'"
            ).fetchone()[0]

    def status(self, task_id: str) -> Optional[str]:
        with self._lock:
            row = self._conn.execute(
                "SELECT status FROM task_queue WHERE task_id=?", (task_id,)
            ).fetchone()
        return row[0] if row else None

    def purge_expired(self) -> int:
        """清理超 retention_until 的行 (W 类 TTL, per 03-DD §5.1/§5.2)。"""
        now = time.time()
        with self._lock:
            cur = self._conn.execute(
                "DELETE FROM task_queue WHERE retention_until < ? AND status IN ('done', 'dead')",
                (now,),
            )
            n = cur.rowcount
            cur2 = self._conn.execute(
                "DELETE FROM dead_letter_queue WHERE retention_until < ?", (now,)
            )
            self._conn.commit()
        return n + cur2.rowcount

    def close(self) -> None:
        with self._lock:
            self._conn.close()

    # === 内部 ===

    def _to_dlq(self, task_id: str, reason: str, retry_count: int) -> None:
        """死信 (per 03-DD §5.2: original_task 完整 JSON)。"""
        row = self._conn.execute(
            "SELECT * FROM task_queue WHERE task_id=?", (task_id,)
        ).fetchone()
        now = time.time()
        self._conn.execute(
            """INSERT OR REPLACE INTO dead_letter_queue
               (task_id, original_task, failure_reason, retry_count,
                first_failed_at, last_failed_at, retention_until)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (
                task_id,
                json.dumps(self._row_dict(row), ensure_ascii=False),
                reason,
                retry_count,
                now,
                now,
                now + self.dlq_retention_seconds,
            ),
        )

    @staticmethod
    def _row_to_task(row) -> Task:
        return Task(
            task_id=row[0],
            brief=row[1],
            agent_type=row[2],
            tenant_id=row[3],
            priority=Priority(row[4]),
            payload=row[5],
            trace_id=row[6] or "",
            correlation_id=row[7],
            created_at=row[8],
        )

    @staticmethod
    def _row_dict(row) -> dict:
        return {
            "task_id": row[0],
            "brief": row[1],
            "agent_type": row[2],
            "tenant_id": row[3],
            "priority": row[4],
            "status": row[5],
            "retry_count": row[6],
            "max_retry": row[7],
            "payload": row[8],
            "created_at": row[9],
            "started_at": row[10],
            "completed_at": row[11],
            "trace_id": row[12],
            "correlation_id": row[13],
            "error": row[14],
            "retention_until": row[15],
        }
