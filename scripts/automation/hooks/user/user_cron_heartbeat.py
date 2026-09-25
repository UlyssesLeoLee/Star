"""user_cron_heartbeat: sample user-defined CronTick hook (per registry.json).

演示: CronTick 时写 1 行 heartbeat timestamp 到 audit log (audit log 由 HR-4 统一处理).
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations


def handler(event, context):
    """CronTick sample hook 入口.

    Args:
        event: Event 物件 (event_type='CronTick', payload 含 cron_id/scheduled_at)
        context: dict

    Returns:
        HookResult: decision=PASS + cron_id
    """
    from scripts.automation.hooks.event_emitter import HookResult

    payload = event.payload or {}
    cron_id = payload.get("cron_id", "unknown")
    return HookResult(
        decision="PASS",
        reason=f"user_cron_heartbeat: cron_id={cron_id} tick",
        latency_ms=0.0,
    )