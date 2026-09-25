"""BL-3: session_end_summary — SessionEnd builtin hook (per BD §4.7).

SessionEnd 时生成 session 总结: count audit log entries by decision, 写到 docs/reports/<session_id>.summary.json.
非破坏性: 仅生成报告, 不影响 session 退出.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import json
import logging
import os
import time
from pathlib import Path
from typing import Any, Dict


logger = logging.getLogger(__name__)


HOOK_DEF: Dict[str, Any] = {
    "name": "session_end_summary",
    "event_type": "SessionEnd",
    "action_type": "post",
    "handler": "builtin:session_end_summary",
    "priority": 50,
    "parallel": False,
    "timeout_ms": 5000,
    "retry": 0,
    "description": "SessionEnd 时聚合 audit log 决策计数, 写到 docs/reports/<session_id>.summary.json. 用于回溯 session.",
}


def handler(event, context):
    """SessionEnd builtin summary hook 入口.

    Returns:
        HookResult: decision=PASS + reason 包含 summary path
    """
    from scripts.automation.hooks.event_emitter import HookResult

    session_id = event.session_id or context.get("session_id", "")
    payload = event.payload or {}
    duration_ms = float(payload.get("duration_ms", 0))
    exit_reason = payload.get("exit_reason", "unknown")

    if not session_id:
        return HookResult(
            decision="PASS",
            reason="session_id 缺失, 跳过 summary",
            latency_ms=0.0,
        )

    try:
        from scripts.automation.hooks.audit_logger import AuditLogger
        hooks_root = Path(__file__).resolve().parents[2]  # scripts/automation/hooks/
        audit = AuditLogger(hooks_root / "logs" / "hook_audit.log")
        counts = audit.count_by_decision()
        # 同 session_id 过滤
        same_session = audit.query(session_id=session_id, limit=10_000)
        decision_counts = {"BLOCK": 0, "ASK": 0, "WARN": 0, "PASS": 0, "transform": 0}
        for e in same_session:
            if e.decision in decision_counts:
                decision_counts[e.decision] += 1
        summary = {
            "session_id": session_id,
            "ended_at": time.strftime("%Y-%m-%dT%H:%M:%S+09:00"),
            "duration_ms": duration_ms,
            "exit_reason": exit_reason,
            "total_hook_runs": len(same_session),
            "by_decision": decision_counts,
        }
        reports_dir = hooks_root.parent.parent / "docs" / "reports"
        reports_dir.mkdir(parents=True, exist_ok=True)
        # 安全文件名 (UUID 含 -)
        safe_id = session_id.replace("/", "_")
        summary_path = reports_dir / f"session-{safe_id}.summary.json"
        summary_path.write_text(
            json.dumps(summary, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        return HookResult(
            decision="PASS",
            reason=f"session_end_summary wrote {summary_path}",
            latency_ms=0.0,
        )
    except Exception as e:
        return HookResult(
            decision="WARN",
            reason=f"session_end_summary fail-open: {type(e).__name__}: {e}",
            latency_ms=0.0,
        )