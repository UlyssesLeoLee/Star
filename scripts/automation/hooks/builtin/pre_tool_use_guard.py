"""BL-1: pre_tool_use_guard — PreToolUse builtin guard hook (per SRS-PRE-TOOL-USE-GUARD-001 §FR-1~FR-3).

复用 scripts.automation.guardian.pre_tool_use_guard.PreToolUseGuard 决策逻辑.
handler 返回 decision: BLOCK|ASK|WARN|PASS (transform 不适用 per PreToolUse guard 范围).

跟 SRS-PRE-TOOL-USE-GUARD-001 关系: 这是 1 个 builtin guard hook, 跟 §1.5 一致.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict


HOOK_DEF: Dict[str, Any] = {
    "name": "pre_tool_use_guard",
    "event_type": "PreToolUse",
    "action_type": "pre",
    "handler": "builtin:pre_tool_use_guard",
    "priority": 10,  # builtin 高优先级 (priority 越小越先执行 per SRS §FR-2.3)
    "parallel": False,
    "timeout_ms": 1000,
    "retry": 0,
    "description": "复用 guardian/pre_tool_use_guard.py 的 15 条危险模式规则; 覆盖 R-BLOCK-001~008 + R-ASK-001~003 + R-WARN-001~002. 决策: BLOCK|ASK|WARN|PASS (transform 不适用).",
}


def handler(event, context):
    """PreToolUse builtin guard hook 入口.

    Args:
        event: Event 物件 (event_id / event_type / timestamp / session_id / payload)
        context: dict (session_id / event_id / timestamp)

    Returns:
        HookResult: decision=BLOCK|ASK|WARN|PASS (无 transformed_args)

    Note:
        真实实现走 scripts.automation.guardian.pre_tool_use_guard.PreToolUseGuard.evaluate().
        此 handler 包一层 thin proxy, 转换 payload (event) → ToolCall (per PreToolUse guard API).
        audit log 写入由 HR-4 (HookRunner) 统一处理, 不在此处重复写.
    """
    # Late import 避免循环依赖
    from scripts.automation.hooks.event_emitter import HookResult

    payload = event.payload or {}
    tool_name = payload.get("tool_name", "unknown")
    tool_args = payload.get("tool_args", {})
    session_id = event.session_id or context.get("session_id", "")

    # 真实实现依赖 guardian.rules + audit logger (per SRS-PRE-TOOL-USE-GUARD-001 §1.5)
    # builtin/pre_tool_use_guard.py 的 parents: [builtin/, hooks/, automation/]
    # → parents[2] = scripts/automation/ → 加 guardian/ → scripts/automation/guardian/
    guardian_dir = Path(__file__).resolve().parents[2] / "guardian"
    rules_path = guardian_dir / "rules" / "pre_tool_use_rules.json"
    audit_path = guardian_dir / "logs" / "pre_tool_use_audit.log"

    try:
        if not rules_path.exists():
            return HookResult(
                decision="PASS",
                reason="PreToolUse guard rules not found (fail-open per §NFR-A-1)",
                latency_ms=0.0,
            )
        from scripts.automation.guardian.pre_tool_use_guard import (
            Decision as GDecision,
            PreToolUseGuard,
            ToolCall,
        )
        from scripts.automation.guardian.rule_database import RuleDatabase
        from scripts.automation.guardian.audit_logger import AuditLogger
        rule_db = RuleDatabase(rules_path, enable_watcher=False)
        audit = AuditLogger(audit_path)
        guard = PreToolUseGuard(rule_db, audit)
        tc = ToolCall(
            tool_name=tool_name,
            tool_args=tool_args,
            session_id=session_id,
            agent_role=payload.get("agent_role", "orchestrator"),
        )
        d = guard.evaluate(tc)
        try:
            rule_db.stop()
        except Exception:
            pass
        return HookResult(
            decision=d.value,
            reason=f"pre_tool_use_guard: {d.value}",
            rule_id=None,
            transformed_args=None,
            latency_ms=0.0,
        )
    except Exception as e:
        # fail-open: any exception → PASS + WARN reason
        return HookResult(
            decision="PASS",
            reason=f"pre_tool_use_guard fail-open: {type(e).__name__}: {e}",
            latency_ms=0.0,
        )