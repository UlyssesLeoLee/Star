"""BL-5: tool_error_fallback — ToolError builtin hook (per BD §4.7).

ToolError 时自动 fallback: 检查 error 是否为 transient (timeout / connection refused / 5xx), 是则注入 system_reminder
让 LLM 决定是否 retry. 不可重试错误 (PermissionError / ValueError) → PASS + WARN reason.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import re
from typing import Any, Dict, List


# Transient 错误 pattern (可 retry)
_TRANSIENT_PATTERNS: List[re.Pattern] = [
    re.compile(r"timeout", re.IGNORECASE),
    re.compile(r"timed out", re.IGNORECASE),
    re.compile(r"connection (refused|reset)", re.IGNORECASE),
    re.compile(r"5\d\d (server error|internal)", re.IGNORECASE),
    re.compile(r"temporarily unavailable", re.IGNORECASE),
    re.compile(r"try again", re.IGNORECASE),
]


# 不可 retry 错误 pattern (永久失败)
_FATAL_PATTERNS: List[re.Pattern] = [
    re.compile(r"permission denied", re.IGNORECASE),
    re.compile(r"unauthorized", re.IGNORECASE),
    re.compile(r"forbidden", re.IGNORECASE),
    re.compile(r"not found", re.IGNORECASE),
    re.compile(r"invalid argument", re.IGNORECASE),
    re.compile(r"valueerror", re.IGNORECASE),
    re.compile(r"typeerror", re.IGNORECASE),
]


HOOK_DEF: Dict[str, Any] = {
    "name": "tool_error_fallback",
    "event_type": "ToolError",
    "action_type": "post",
    "handler": "builtin:tool_error_fallback",
    "priority": 100,
    "parallel": False,
    "timeout_ms": 200,
    "retry": 0,
    "description": "ToolError 时分类: transient (timeout/5xx) → WARN 注入 retry reminder, fatal (PermissionError/ValueError) → PASS + 提示人工介入.",
}


def handler(event, context):
    """ToolError builtin fallback hook 入口.

    Returns:
        HookResult: decision=WARN (transient) 或 PASS (fatal)
        transformed_args: 含 retry_recommended 标志 (供 LLM 决策)
    """
    from scripts.automation.hooks.event_emitter import HookResult

    payload = event.payload or {}
    error = str(payload.get("error", ""))
    stack_trace = str(payload.get("stack_trace", ""))

    text = f"{error}\n{stack_trace}"

    # 1. fatal → PASS + 人工介入
    for pat in _FATAL_PATTERNS:
        if pat.search(text):
            return HookResult(
                decision="PASS",
                reason=f"tool_error_fallback: fatal error ({pat.pattern!r}), 需人工介入",
                latency_ms=0.0,
            )

    # 2. transient → WARN + retry reminder (transform 注入)
    for pat in _TRANSIENT_PATTERNS:
        if pat.search(text):
            return HookResult(
                decision="WARN",
                reason=f"tool_error_fallback: transient error ({pat.pattern!r}), 建议 retry",
                rule_id="tool_error_fallback.transient",
                transformed_args={
                    **payload,
                    "retry_recommended": True,
                    "retry_reason": f"transient: {pat.pattern!r}",
                },
                latency_ms=0.0,
            )

    # 3. 未知错误 → PASS + 保守
    return HookResult(
        decision="PASS",
        reason="tool_error_fallback: 未知错误类型, 保守 PASS",
        latency_ms=0.0,
    )