#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/task_ops/task_metadata_ddl.py
task_metadata SQLite DDL 落地 (per G-TMO-04, 守门 #13 c Master RLS + 守门 #13 d SCD Type 2)

W/T/M 分类 (per 守门 #DB-13 + docs/data-design/p3-d-classification-w-t-m.md):
  - task_metadata (Master): task 的 metadata 当前值 (SCD Type 2, 物理删除禁止)
  - task_metadata_scd (Master): task 的 metadata 变更历史 (SCD Type 2 history 永存)
  - task_metadata_audit (Transaction): task_metadata 变更事件 (append-only audit, 物理删除禁止)
  - task_metadata_session (Work): 短 TTL 作业中临时状态, 完成后清理

设计原则 (per 守门):
  - 守门 #13 c: Master 表 100% RLS 必携 tenant_id + workspace_ids
  - 守门 #13 d: SCD Type 2 关系变更留痕 (Transaction append-only)
  - 守门 #19: Python 化, 标准库 sqlite3, 不依赖 SQLAlchemy
  - 守门 #22: 不进 main 编译链 (Python 进程单独跑)

用法:
    python -m scripts.automation.task_ops.task_metadata_ddl init /path/to/db.sqlite
    python -m scripts.automation.task_ops.task_metadata_ddl validate-schema /path/to/db.sqlite
"""
from __future__ import annotations

import argparse
import sqlite3
import sys
from pathlib import Path
from typing import List, Tuple

# ===== DDL Statements (per 守门 #DB-13 W/T/M 分类) =====

DDL_TASK_METADATA: str = """
CREATE TABLE IF NOT EXISTS task_metadata (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    name TEXT,
    labels_json TEXT NOT NULL DEFAULT '[]',
    notes TEXT,
    priority INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 10),
    version INTEGER NOT NULL DEFAULT 1,
    is_current INTEGER NOT NULL DEFAULT 1 CHECK (is_current IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    UNIQUE (task_id, tenant_id, workspace_id, version)
)
""".strip()
"""task_metadata Master 表 (SCD Type 2 current row, 物理删除禁止 per 守门 #13 c).

字段:
  - id: task_metadata 主键 (uuid)
  - task_id: 关联 task_card.task_id (per F.4 task_card W/T/M 分类)
  - tenant_id: RLS 必携 (守门 #13 c)
  - workspace_id: RLS 必携 (守门 #13 c)
  - name: 任务名 (≤ 256 chars, per metadata_node.py MAX_NAME_LENGTH)
  - labels_json: 标签数组 JSON 序列化 (≤ 20 个, ≤ 64 chars each)
  - notes: 备注 (≤ 4096 chars, per MAX_NOTES_LENGTH)
  - priority: 1-10 (per MAX_PRIORITY/MIN_PRIORITY)
  - version: SCD Type 2 版本号 (从 1 起递增, per 守门 #13 d)
  - is_current: 1 = 当前版本, 0 = 历史版本 (守门 #13 d SCD 关系变更留痕)
  - created_at_ms: 首次创建 ms (epoch)
  - updated_at_ms: 最近更新 ms (epoch)

唯一约束: (task_id, tenant_id, workspace_id, version), 同一 (task + tenant + workspace) 内 version 唯一.
"""


DDL_TASK_METADATA_SCD: str = """
CREATE TABLE IF NOT EXISTS task_metadata_scd (
    snapshot_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    previous_metadata_json TEXT NOT NULL,
    snapshot_at_ms INTEGER NOT NULL,
    snapshot_reason TEXT NOT NULL DEFAULT 'metadata_update'
)
""".strip()
"""task_metadata_scd Master SCD Type 2 历史表 (永存, 物理删除禁止 per 守门 #13 d).

每次 metadata 更新时, 旧 metadata 完整 snapshot 落到此表, 用于关系变更留痕.
"""


DDL_TASK_METADATA_AUDIT: str = """
CREATE TABLE IF NOT EXISTS task_metadata_audit (
    audit_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    actor_session_id TEXT,
    event_type TEXT NOT NULL CHECK (event_type IN ('created', 'updated', 'scd_snapshot', 'rls_violation', 'validation_failed')),
    event_at_ms INTEGER NOT NULL,
    metadata_diff_json TEXT,
    snapshot_id TEXT
)
""".strip()
"""task_metadata_audit Transaction 表 (append-only, 物理删除禁止 per 守门 #13 d).

5 类事件: created / updated / scd_snapshot / rls_violation / validation_failed.
"""


DDL_TASK_METADATA_SESSION: str = """
CREATE TABLE IF NOT EXISTS task_metadata_session (
    session_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    session_started_ms INTEGER NOT NULL,
    session_expires_ms INTEGER NOT NULL,
    session_state_json TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
)
""".strip()
"""task_metadata_session Work 表 (短 TTL 作业中, 完成后清理 per 守门 #DB-13 a).

metadata_node 操作期间用, 完成后 set is_active=0 + cleanup.
"""


# ===== 索引 DDL (per 守门 #13 c RLS 必携) =====

DDL_INDEXES: List[str] = [
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_tenant ON task_metadata(tenant_id, workspace_id)",
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_task_current ON task_metadata(task_id, is_current)",
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_scd_task ON task_metadata_scd(task_id, version DESC)",
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_scd_tenant ON task_metadata_scd(tenant_id, workspace_id)",
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_audit_task ON task_metadata_audit(task_id, event_at_ms DESC)",
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_audit_tenant ON task_metadata_audit(tenant_id, workspace_id, event_at_ms DESC)",
    "CREATE INDEX IF NOT EXISTS idx_task_metadata_session_active ON task_metadata_session(is_active, session_expires_ms)",
]


# ===== API =====

def init_schema(db_path: str) -> dict:
    """初始化 task_metadata schema (4 表 + 7 索引).

    Returns:
        dict {table_name: row_count, indexes: count}
    """
    conn = sqlite3.connect(db_path)
    try:
        # Master 表
        conn.execute(DDL_TASK_METADATA)
        conn.execute(DDL_TASK_METADATA_SCD)
        # Transaction 表
        conn.execute(DDL_TASK_METADATA_AUDIT)
        # Work 表
        conn.execute(DDL_TASK_METADATA_SESSION)
        # 索引
        for ddl in DDL_INDEXES:
            conn.execute(ddl)
        conn.commit()

        # 验证
        cur = conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'task_metadata%' ORDER BY name"
        )
        tables = [row[0] for row in cur.fetchall()]

        cur = conn.execute(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_task_metadata%' ORDER BY name"
        )
        indexes = [row[0] for row in cur.fetchall()]

        return {
            "tables": tables,
            "indexes": indexes,
            "schema_status": "initialized",
        }
    finally:
        conn.close()


def validate_schema(db_path: str) -> dict:
    """验证 task_metadata schema 完整 (4 表 + 7 索引 + 字段约束).

    Raises:
        RuntimeError: schema 不完整
    """
    conn = sqlite3.connect(db_path)
    try:
        expected_tables = {
            "task_metadata", "task_metadata_scd",
            "task_metadata_audit", "task_metadata_session",
        }
        cur = conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'task_metadata%'"
        )
        actual_tables = {row[0] for row in cur.fetchall()}
        if actual_tables != expected_tables:
            missing = expected_tables - actual_tables
            extra = actual_tables - expected_tables
            raise RuntimeError(
                f"task_metadata schema 不完整: missing={missing} extra={extra}"
            )

        cur = conn.execute(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_task_metadata%'"
        )
        actual_indexes = {row[0] for row in cur.fetchall()}
        if len(actual_indexes) < len(DDL_INDEXES):
            raise RuntimeError(
                f"task_metadata 索引缺失: expected {len(DDL_INDEXES)}, got {len(actual_indexes)}"
            )

        # 验证关键字段 (tenant_id RLS 必携 per 守门 #13 c)
        for table in expected_tables:
            cur = conn.execute(f"PRAGMA table_info({table})")
            columns = {row[1] for row in cur.fetchall()}
            if "tenant_id" not in columns:
                raise RuntimeError(
                    f"{table} 缺 tenant_id 字段 (per 守门 #13 c Master RLS 必携)"
                )
            if table in {"task_metadata", "task_metadata_scd", "task_metadata_audit", "task_metadata_session"}:
                if "workspace_id" not in columns:
                    raise RuntimeError(
                        f"{table} 缺 workspace_id 字段 (per 守门 #13 c Master RLS 必携)"
                    )

        return {
            "schema_status": "valid",
            "tables": sorted(actual_tables),
            "indexes": sorted(actual_indexes),
            "tenant_id_check": "pass",
            "workspace_id_check": "pass",
        }
    finally:
        conn.close()


# ===== CLI =====

def main() -> int:
    parser = argparse.ArgumentParser(
        description="task_metadata SQLite DDL (G-TMO-04, per 守门 #13 c/d + 守门 #DB-13 W/T/M)"
    )
    parser.add_argument("command", choices=["init", "validate-schema"], help="操作")
    parser.add_argument("db_path", help="SQLite 数据库路径")
    args = parser.parse_args()

    db_path = Path(args.db_path).resolve()
    db_path.parent.mkdir(parents=True, exist_ok=True)

    if args.command == "init":
        result = init_schema(str(db_path))
        print(f"✅ task_metadata schema 初始化完成: {db_path}")
        print(f"   tables ({len(result['tables'])}): {result['tables']}")
        print(f"   indexes ({len(result['indexes'])}): {result['indexes']}")
    elif args.command == "validate-schema":
        result = validate_schema(str(db_path))
        print(f"✅ task_metadata schema 验证通过: {db_path}")
        print(f"   tables ({len(result['tables'])}): {result['tables']}")
        print(f"   indexes ({len(result['indexes'])}): {result['indexes']}")
        print(f"   tenant_id RLS check: {result['tenant_id_check']}")
        print(f"   workspace_id RLS check: {result['workspace_id_check']}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
