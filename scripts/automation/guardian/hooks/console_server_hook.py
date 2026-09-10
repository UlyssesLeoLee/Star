"""console_server.py PreToolUse hook 接入点 (per DD §1.3).

Use this snippet to integrate PreToolUseGuard into scripts/automation/console_server.py v0.1+.

Example integration (in console_server.py, before tool execution):

    from scripts.automation.guardian.pre_tool_use_guard import (
        PreToolUseGuard, ToolCall, Decision
    )
    from scripts.automation.guardian.rule_database import RuleDatabase
    from scripts.automation.guardian.audit_logger import AuditLogger
    from pathlib import Path

    RULES_PATH = Path(__file__).parent.parent / "rules" / "pre_tool_use_rules.json"
    AUDIT_PATH = Path(__file__).parent.parent / "logs" / "pre_tool_use_audit.log"
    rule_db = RuleDatabase(RULES_PATH)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(rule_db, audit)

    # 假设 LLM 决定调 bash, 插入以下拦截:
    tool_call = ToolCall(
        tool_name="bash",
        tool_args={"command": cmd_str},
        session_id=session_id,
        agent_role=agent_role,
    )
    decision = guard.evaluate(tool_call)
    if decision == Decision.BLOCK:
        return {"error": f"Blocked by PreToolUse guard", "rule_id": ...}
    if decision == Decision.ASK:
        # 走 ask_user (per 守门 v28)
        user_choice = ask_user(...)
        if user_choice != "confirm":
            return {"error": "User rejected", "decision": decision.value}
    # WARN: 继续执行, 警告已注入 LLM context
    # PASS: 直接执行
"""
# SPDX-License-Identifier: MIT OR Apache-2.0
