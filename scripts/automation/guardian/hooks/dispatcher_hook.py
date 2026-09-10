"""dispatcher.py invoke() 前置 hook 接入点 (per DD §1.3 + FR-1.2).

Use this snippet to integrate PreToolUseGuard into scripts/automation/dispatcher.py v0.1+ invoke() function.

Example integration (in dispatcher.py, at start of invoke()):

    from scripts.automation.guardian.pre_tool_use_guard import (
        PreToolUseGuard, Decision
    )
    from scripts.automation.guardian.rule_database import RuleDatabase
    from scripts.automation.guardian.audit_logger import AuditLogger
    from pathlib import Path

    RULES_PATH = Path(__file__).parent / "rules" / "pre_tool_use_rules.json"
    AUDIT_PATH = Path(__file__).parent / "logs" / "pre_tool_use_audit.log"
    rule_db = RuleDatabase(RULES_PATH)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(rule_db, audit)

    def invoke(task_id, script_path, args, parent_session_id):
        # ★ PreToolUse dispatch 前置 (per FR-1.2 + NFR-S-4)
        decision = guard.evaluate_for_dispatch(
            task_id=task_id,
            script_path=script_path,
            args=args,
            parent_session_id=parent_session_id,
        )
        if decision == Decision.BLOCK:
            raise DispatchBlocked(
                reason="PreToolUse guard BLOCK",
                rule_id=guard.last_rule_id(),  # 需要扩展 guard 暴露
            )
        if decision == Decision.ASK:
            # 走 ask_user (per 守门 v28)
            user_choice = ask_user(
                title="Sub-agent dispatch 需确认",
                options=[
                    {"label": "取消 (推荐)", "value": "cancel"},
                    {"label": "确认执行", "value": "confirm"},
                ],
            )
            if user_choice != "confirm":
                raise DispatchBlocked(reason="User rejected")
        # PASS → 继续 invoke
        ...
"""
# SPDX-License-Identifier: MIT OR Apache-2.0
