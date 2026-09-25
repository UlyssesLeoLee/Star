"""user_post_audit: sample user-defined PostToolUse hook (per registry.json).

演示: 工具执行 > 5s 时 transform 注入 slow_tool 标志 (供后续 hook 参考).
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

from typing import Any, Dict


def handler(event, context):
    """PostToolUse sample hook 入口.

    Args:
        event: Event 物件 (event_type='PostToolUse', payload 含 tool_name/tool_result/latency_ms)
        context: dict

    Returns:
        HookResult: decision=transform (slow_tool 标志) 或 PASS
    """
    from scripts.automation.hooks.event_emitter import HookResult

    payload = event.payload or {}
    latency_ms = float(payload.get("latency_ms", 0))
    tool_name = payload.get("tool_name", "unknown")

    if latency_ms > 5000:
        return HookResult(
            decision="transform",
            reason=f"slow_tool: {tool_name} took {latency_ms:.1f}ms",
            rule_id="user_post_audit.slow_tool",
            transformed_args={
                **payload,
                "slow_tool": True,
                "slow_threshold_ms": 5000,
            },
            latency_ms=0.0,
        )
    return HookResult(
        decision="PASS",
        reason=f"user_post_audit: {tool_name} took {latency_ms:.1f}ms (within threshold)",
        latency_ms=0.0,
    )