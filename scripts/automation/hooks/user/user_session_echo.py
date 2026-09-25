"""user_session_echo: sample user-defined SessionEnd hook (per registry.json).

演示: SessionEnd 时把 session_id + duration 写到 ~/.multica/session-history.log (append-only).
非破坏性: 仅追加一行, 用于演示用户用 hook 拓展 session lifecycle.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import os
import time
from pathlib import Path


def handler(event, context):
    """SessionEnd sample hook 入口.

    Args:
        event: Event 物件 (event_type='SessionEnd', payload 含 session_id/duration_ms/exit_reason)
        context: dict

    Returns:
        HookResult: decision=PASS + 写 1 行到 ~/.multica/session-history.log
    """
    from scripts.automation.hooks.event_emitter import HookResult

    payload = event.payload or {}
    session_id = payload.get("session_id", event.session_id or "unknown")
    duration_ms = float(payload.get("duration_ms", 0))
    exit_reason = payload.get("exit_reason", "unknown")

    try:
        multica_root = Path(os.environ.get("MULTICA_HOME", Path.home() / ".multica"))
        multica_root.mkdir(parents=True, exist_ok=True)
        history = multica_root / "session-history.log"
        line = (
            f"{time.strftime('%Y-%m-%dT%H:%M:%S+09:00')} "
            f"session_id={session_id} duration_ms={duration_ms:.0f} "
            f"exit_reason={exit_reason}\n"
        )
        with history.open("a", encoding="utf-8") as f:
            f.write(line)
        return HookResult(
            decision="PASS",
            reason=f"user_session_echo: wrote {history}",
            latency_ms=0.0,
        )
    except Exception as e:
        return HookResult(
            decision="WARN",
            reason=f"user_session_echo fail-open: {type(e).__name__}: {e}",
            latency_ms=0.0,
        )