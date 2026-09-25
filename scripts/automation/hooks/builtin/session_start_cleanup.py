"""BL-2: session_start_cleanup — SessionStart builtin hook (per BD §4.7).

SessionStart 时清理临时文件: .multica/tmp/*.lock > 7 天, .multica/*.tmp > 1 天.
非破坏性: 仅清理过期 lock/temp, 不影响活跃 session.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import logging
import os
import time
from pathlib import Path
from typing import Any, Dict


logger = logging.getLogger(__name__)


HOOK_DEF: Dict[str, Any] = {
    "name": "session_start_cleanup",
    "event_type": "SessionStart",
    "action_type": "post",  # SessionStart 后置清理
    "handler": "builtin:session_start_cleanup",
    "priority": 50,
    "parallel": False,
    "timeout_ms": 5000,  # 文件 IO 可能较慢, 给 5s
    "retry": 0,
    "description": "SessionStart 后清理过期 lock/tmp 文件: .multica/tmp/*.lock > 7d, .multica/*.tmp > 1d. 不影响活跃 session.",
}


def handler(event, context):
    """SessionStart builtin cleanup hook 入口.

    Returns:
        HookResult: decision=PASS (info-level) + reason 包含清理数量
    """
    from scripts.automation.hooks.event_emitter import HookResult

    cleaned = 0
    try:
        multica_root = Path(os.environ.get("MULTICA_HOME", Path.home() / ".multica"))
        if not multica_root.exists():
            return HookResult(
                decision="PASS",
                reason=f"multica root {multica_root} 不存在, 跳过",
                latency_ms=0.0,
            )
        now = time.time()
        # 清理 1: *.tmp > 1 天
        for tmp in multica_root.glob("**/*.tmp"):
            try:
                if (now - tmp.stat().st_mtime) > 86400:
                    tmp.unlink()
                    cleaned += 1
            except OSError:
                pass
        # 清理 2: tmp/*.lock > 7 天
        tmp_dir = multica_root / "tmp"
        if tmp_dir.exists():
            for lock in tmp_dir.glob("*.lock"):
                try:
                    if (now - lock.stat().st_mtime) > 7 * 86400:
                        lock.unlink()
                        cleaned += 1
                except OSError:
                    pass
    except Exception as e:
        # cleanup 失败不影响 session, 返回 WARN
        return HookResult(
            decision="WARN",
            reason=f"session_start_cleanup fail-open: {type(e).__name__}: {e}",
            latency_ms=0.0,
        )

    return HookResult(
        decision="PASS",
        reason=f"session_start_cleanup cleaned {cleaned} file(s)",
        latency_ms=0.0,
    )