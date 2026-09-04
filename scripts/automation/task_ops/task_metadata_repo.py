#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/task_ops/task_metadata_repo.py
TaskMetadataRepository — task_metadata 表 CRUD 适配器 (per G-TMO-04b)

职责: 把 metadata_node 的 in-memory state 操作桥接到 SQLite 持久化 (per G-TMO-04 DDL).

守门 (per AGENTS.md §4):
  - 守门 #13 c: Master RLS 必携 tenant_id / workspace_id (call site 校验, per metadata_node 5 步 validate)
  - 守门 #13 d: SCD Type 2 关系变更留痕 (旧 metadata snapshot 落 task_metadata_scd)
  - 守门 #13 d: audit 5 类事件落 task_metadata_audit (created / updated / scd_snapshot / rls_violation / validation_failed)
  - 守门 #19: Python 化, 标准库 sqlite3, 不依赖 SQLAlchemy
  - 守门 #22: 调试控制台 (port 8080) 不进 main 编译链 (本模块独立进程)

用法:
    from automation.task_ops.task_metadata_repo import TaskMetadataRepository
    repo = TaskMetadataRepository("/path/to/db.sqlite")
    repo.upsert_metadata(task_id="t1", tenant_id="A", workspace_id="ws-1",
                         metadata={"name": "x", "priority": 5, "labels": [], "notes": None},
                         actor_session_id="sess-1")
"""
from __future__ import annotations

import json
import sqlite3
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional


# ===== Domain Models =====

@dataclass
class TaskMetadataRecord:
    """task_metadata 当前行 (Master, SCD Type 2 is_current=1)."""
    id: str
    task_id: str
    tenant_id: str
    workspace_id: str
    name: Optional[str]
    labels: List[str]
    notes: Optional[str]
    priority: int
    version: int
    is_current: bool
    created_at_ms: int
    updated_at_ms: int


# ===== Repository =====

class TaskMetadataRepository:
    """task_metadata 仓库 (per G-TMO-04 DDL).

    约束 (per 守门 #13 c/d + 守门 #DB-13):
      - upsert_metadata 走 SCD Type 2 (旧 version is_current=0 + 新 version is_current=1)
      - insert_audit_event 5 类 CHECK 约束
      - delete_metadata 禁止 (Master 物理删除禁止 per 守门 #13 c)
    """

    VALID_AUDIT_EVENT_TYPES: frozenset[str] = frozenset({
        "created", "updated", "scd_snapshot", "rls_violation", "validation_failed",
    })

    def __init__(self, db_path: str) -> None:
        self.db_path = str(Path(db_path).resolve())
        Path(self.db_path).parent.mkdir(parents=True, exist_ok=True)

    def _conn(self) -> sqlite3.Connection:
        conn = sqlite3.connect(self.db_path)
        conn.execute("PRAGMA foreign_keys = ON")
        return conn

    # ===== CRUD: Metadata =====

    def get_current_metadata(
        self,
        task_id: str,
        tenant_id: str,
        workspace_id: str,
    ) -> Optional[TaskMetadataRecord]:
        """读 task 当前 metadata (is_current=1). 跨 tenant RLS 必携校验."""
        conn = self._conn()
        try:
            cur = conn.execute(
                """SELECT id, task_id, tenant_id, workspace_id, name, labels_json, notes,
                          priority, version, is_current, created_at_ms, updated_at_ms
                   FROM task_metadata
                   WHERE task_id = ? AND tenant_id = ? AND workspace_id = ? AND is_current = 1""",
                (task_id, tenant_id, workspace_id),
            )
            row = cur.fetchone()
            if row is None:
                return None
            return self._row_to_record(row)
        finally:
            conn.close()

    def upsert_metadata(
        self,
        task_id: str,
        tenant_id: str,
        workspace_id: str,
        metadata: Dict[str, Any],
        actor_session_id: Optional[str] = None,
    ) -> TaskMetadataRecord:
        """插入或更新 task metadata (SCD Type 2).

        流程:
          1. 读旧 current row (per SCD 关系变更留痕)
          2. 旧 is_current=1 → 0 (旧版本 SCD)
          3. 旧 metadata 完整 snapshot → task_metadata_scd
          4. 插入新 version is_current=1
          5. audit event "created" (新) 或 "updated" (覆盖) + "scd_snapshot" (旧)

        守门:
          - 守门 #13 c: tenant_id / workspace_id 必携
          - 守门 #13 d: SCD Type 2 (旧 version 永存)
          - 守门 #13 d: audit 5 类事件
        """
        now_ms = int(time.time() * 1000)
        conn = self._conn()
        try:
            cur = conn.execute("BEGIN IMMEDIATE")

            # 1. 读旧 current row
            cur = conn.execute(
                """SELECT id, version, name, labels_json, notes, priority, created_at_ms
                   FROM task_metadata
                   WHERE task_id = ? AND tenant_id = ? AND workspace_id = ? AND is_current = 1""",
                (task_id, tenant_id, workspace_id),
            )
            old_row = cur.fetchone()

            if old_row is not None:
                # 2. 旧 is_current=1 → 0
                cur = conn.execute(
                    "UPDATE task_metadata SET is_current = 0 WHERE id = ?",
                    (old_row[0],),
                )
                # 3. 旧 metadata snapshot → scd
                old_metadata = {
                    "name": old_row[2],
                    "labels": json.loads(old_row[3]) if old_row[3] else [],
                    "notes": old_row[4],
                    "priority": old_row[5],
                }
                snapshot_id = f"scd-{uuid.uuid4().hex[:8]}"
                cur = conn.execute(
                    """INSERT INTO task_metadata_scd
                       (snapshot_id, task_id, tenant_id, workspace_id, version, previous_metadata_json, snapshot_at_ms, snapshot_reason)
                       VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                    (snapshot_id, task_id, tenant_id, workspace_id, old_row[1],
                     json.dumps(old_metadata), now_ms, "metadata_update"),
                )
                new_version = old_row[1] + 1
                # 5b. audit scd_snapshot
                self._insert_audit_event(
                    conn, task_id, tenant_id, workspace_id, actor_session_id,
                    "scd_snapshot", now_ms,
                    metadata_diff_json=json.dumps({"snapshot_id": snapshot_id, "version": old_row[1]}),
                    snapshot_id=snapshot_id,
                )
                event_type = "updated"
                created_at_ms = old_row[6]
            else:
                new_version = 1
                event_type = "created"
                created_at_ms = now_ms

            # 4. 插入新 version is_current=1
            new_id = f"tm-{uuid.uuid4().hex[:8]}"
            cur = conn.execute(
                """INSERT INTO task_metadata
                   (id, task_id, tenant_id, workspace_id, name, labels_json, notes, priority, version, is_current, created_at_ms, updated_at_ms)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                (
                    new_id, task_id, tenant_id, workspace_id,
                    metadata.get("name"),
                    json.dumps(metadata.get("labels") or []),
                    metadata.get("notes"),
                    int(metadata.get("priority") or 5),
                    new_version, 1, created_at_ms, now_ms,
                ),
            )
            # 5a. audit created/updated
            self._insert_audit_event(
                conn, task_id, tenant_id, workspace_id, actor_session_id,
                event_type, now_ms,
                metadata_diff_json=json.dumps({"version": new_version, "fields_updated": sorted(metadata.keys())}),
            )
            conn.commit()
            return TaskMetadataRecord(
                id=new_id,
                task_id=task_id,
                tenant_id=tenant_id,
                workspace_id=workspace_id,
                name=metadata.get("name"),
                labels=metadata.get("labels") or [],
                notes=metadata.get("notes"),
                priority=int(metadata.get("priority") or 5),
                version=new_version,
                is_current=True,
                created_at_ms=created_at_ms,
                updated_at_ms=now_ms,
            )
        except Exception:
            conn.rollback()
            raise
        finally:
            conn.close()

    def get_scd_history(
        self,
        task_id: str,
        tenant_id: str,
        workspace_id: str,
        limit: int = 50,
    ) -> List[Dict[str, Any]]:
        """读 task SCD 历史 (per 守门 #13 d 关系变更留痕)."""
        conn = self._conn()
        try:
            cur = conn.execute(
                """SELECT snapshot_id, version, previous_metadata_json, snapshot_at_ms, snapshot_reason
                   FROM task_metadata_scd
                   WHERE task_id = ? AND tenant_id = ? AND workspace_id = ?
                   ORDER BY version DESC
                   LIMIT ?""",
                (task_id, tenant_id, workspace_id, limit),
            )
            return [
                {
                    "snapshot_id": row[0],
                    "version": row[1],
                    "previous_metadata": json.loads(row[2]),
                    "snapshot_at_ms": row[3],
                    "snapshot_reason": row[4],
                }
                for row in cur.fetchall()
            ]
        finally:
            conn.close()

    def get_audit_log(
        self,
        task_id: str,
        tenant_id: str,
        workspace_id: str,
        limit: int = 50,
    ) -> List[Dict[str, Any]]:
        """读 task audit log (per 守门 #13 d Transaction 100% audit)."""
        conn = self._conn()
        try:
            cur = conn.execute(
                """SELECT audit_id, event_type, event_at_ms, actor_session_id, metadata_diff_json, snapshot_id
                   FROM task_metadata_audit
                   WHERE task_id = ? AND tenant_id = ? AND workspace_id = ?
                   ORDER BY event_at_ms DESC
                   LIMIT ?""",
                (task_id, tenant_id, workspace_id, limit),
            )
            return [
                {
                    "audit_id": row[0],
                    "event_type": row[1],
                    "event_at_ms": row[2],
                    "actor_session_id": row[3],
                    "metadata_diff": json.loads(row[4]) if row[4] else None,
                    "snapshot_id": row[5],
                }
                for row in cur.fetchall()
            ]
        finally:
            conn.close()

    # ===== Audit =====

    def _insert_audit_event(
        self,
        conn: sqlite3.Connection,
        task_id: str,
        tenant_id: str,
        workspace_id: str,
        actor_session_id: Optional[str],
        event_type: str,
        event_at_ms: int,
        metadata_diff_json: Optional[str] = None,
        snapshot_id: Optional[str] = None,
    ) -> None:
        """插入 audit 事件 (5 类 CHECK 约束 per 守门 #13 d)."""
        if event_type not in self.VALID_AUDIT_EVENT_TYPES:
            raise ValueError(
                f"task_metadata_repo: invalid event_type {event_type!r}, "
                f"expected one of {sorted(self.VALID_AUDIT_EVENT_TYPES)}"
            )
        audit_id = f"audit-{uuid.uuid4().hex[:8]}"
        conn.execute(
            """INSERT INTO task_metadata_audit
               (audit_id, task_id, tenant_id, workspace_id, actor_session_id, event_type, event_at_ms, metadata_diff_json, snapshot_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (audit_id, task_id, tenant_id, workspace_id, actor_session_id, event_type, event_at_ms, metadata_diff_json, snapshot_id),
        )

    # ===== Helpers =====

    def _row_to_record(self, row: tuple) -> TaskMetadataRecord:
        return TaskMetadataRecord(
            id=row[0],
            task_id=row[1],
            tenant_id=row[2],
            workspace_id=row[3],
            name=row[4],
            labels=json.loads(row[5]) if row[5] else [],
            notes=row[6],
            priority=row[7],
            version=row[8],
            is_current=bool(row[9]),
            created_at_ms=row[10],
            updated_at_ms=row[11],
        )

    # ===== Master 表物理删除禁止 (per 守门 #13 c) =====

    def delete_metadata(self, task_id: str, tenant_id: str, workspace_id: str) -> None:
        """Master 表物理删除禁止 (per 守门 #13 c).

        Raises:
            PermissionError: Master 表物理删除禁止
        """
        raise PermissionError(
            f"task_metadata_repo: physical DELETE on Master table task_metadata is forbidden "
            f"(per 守门 #13 c Master 100% RLS 必携 + 物理删除禁止)"
        )
