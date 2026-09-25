"""HR-4: Hook Runner — 単 hook 執行 + timeout + retry + transform 累積 (per BD §4.5 + DD §2.4).

跨平台 timeout (per SRS §NFR-T-1 + §NFR-T-2):
    - POSIX: signal.SIGALRM
    - Windows: threading.Timer

异常处理 (per SRS §FR-4.5):
    - handler 拋异常 → WARN (继续 fan-out, log level = ERROR)
    - handler 超时 → BLOCK (安全优先)
    - handler 返回非法 decision → WARN
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import importlib
import logging
import platform
import signal
import threading
import time
from contextlib import contextmanager
from typing import Any, Callable, Dict, Optional


logger = logging.getLogger(__name__)


class HookTimeoutError(Exception):
    """Hook 執行超時異常 (per SRS §FR-4.5)."""
    pass


class HookRunner:
    """Hook Runner 主類 (per BD §4.5 + DD §2.4)."""

    def __init__(self, audit_logger: "AuditLogger"):
        self._audit_logger = audit_logger

    def run(self, hook: "Hook", event: "Event") -> "HookResult":
        """執行 1 个 hook, 返回 HookResult.

        異常處理 (per SRS §FR-4.5):
        - handler 拋异常: 視為 WARN, 繼續下一個 hook, 寫 audit log
        - handler 超时: 視為 BLOCK, 寫 audit log
        - handler 返回非法 decision: 視為 WARN, 寫 audit log
        """
        start_time = time.monotonic()
        result: HookResult
        try:
            handler_fn = self._load_handler(hook.handler)

            # 帶 timeout 執行 (per SRS §FR-4.5)
            result = self._execute_with_timeout(handler_fn, event, hook.timeout_ms)

            # 校驗 result.decision
            if result.decision not in ("BLOCK", "ASK", "WARN", "PASS", "transform"):
                logger.warning(
                    "Hook %s 返回非法 decision %r, 视为 WARN",
                    hook.name, result.decision,
                )
                result = HookResult(
                    decision="WARN",
                    reason=f"invalid decision: {result.decision}",
                    rule_id=hook.name,
                    latency_ms=(time.monotonic() - start_time) * 1000,
                )
        except HookTimeoutError:
            logger.warning(
                "Hook %s 超時 (%dms), 视为 BLOCK",
                hook.name, hook.timeout_ms,
            )
            result = HookResult(
                decision="BLOCK",  # 安全优先
                reason=f"hook timeout after {hook.timeout_ms}ms",
                rule_id=hook.name,
                latency_ms=(time.monotonic() - start_time) * 1000,
            )
        except Exception as e:
            # handler 拋异常 → WARN (per SRS §FR-4.5)
            logger.error(
                "Hook %s 拋异常, 视为 WARN: %s: %s",
                hook.name, type(e).__name__, e,
            )
            result = HookResult(
                decision="WARN",
                reason=f"hook exception: {type(e).__name__}: {e}",
                rule_id=hook.name,
                latency_ms=(time.monotonic() - start_time) * 1000,
            )

        # 寫 audit log (per SRS §FR-5)
        try:
            self._audit_logger.log_hook_run(hook, event, result)
        except Exception as e:
            # audit 写失败 → fail-closed (per SRS §NFR-A-2 + §FR-5.2)
            logger.error("audit log 寫失败 (fail-closed): %s", e)
            result = HookResult(
                decision="BLOCK",
                reason=f"audit log write failed: {e}",
                rule_id=hook.name,
                latency_ms=(time.monotonic() - start_time) * 1000,
            )

        return result

    def _load_handler(self, handler_path: str) -> Callable:
        """加载 handler (builtin 或 user-defined Python 模組).

        builtin: "builtin:pre_tool_use_guard" → 从 scripts.automation.hooks.builtin 加載
        user-defined: "my_module:my_handler" → 从 sys.path 加載

        Returns:
            callable (event, context) -> HookResult

        Raises:
            ImportError / AttributeError: handler 加載失败 (上层捕获后视為 WARN)
        """
        if not handler_path:
            raise ImportError("handler 路径为空")
        if handler_path.startswith("builtin:"):
            module_name = handler_path[len("builtin:"):]
            module = importlib.import_module(
                f"scripts.automation.hooks.builtin.{module_name}"
            )
            fn = getattr(module, "handler", None)
            if fn is None:
                raise AttributeError(
                    f"builtin.{module_name} 必须导出 handler(callable)"
                )
            return fn
        # user-defined: "module_path:func_name"
        if ":" not in handler_path:
            raise ValueError(
                f"user-defined handler 格式 'module:func', got: {handler_path!r}"
            )
        module_name, func_name = handler_path.split(":", 1)
        module = importlib.import_module(module_name)
        fn = getattr(module, func_name, None)
        if fn is None:
            raise AttributeError(
                f"{module_name}.{func_name} 不存在"
            )
        return fn

    @contextmanager
    def _timeout(self, seconds: float):
        """跨平台 timeout (per SRS §NFR-T-1 + §NFR-T-2).

        POSIX: signal.SIGALRM
        Windows: threading.Timer (信号不支持)
        """
        if platform.system() == "Windows":
            # 用 threading.Timer
            timer_state = {"fired": False, "error": None}

            def _fire():
                timer_state["fired"] = True
                timer_state["error"] = HookTimeoutError(
                    f"hook timeout after {seconds}s"
                )

            timer = threading.Timer(seconds, _fire)
            timer.start()
            try:
                yield
                if timer_state["fired"]:
                    raise timer_state["error"]  # type: ignore[misc]
            finally:
                timer.cancel()
        else:
            # POSIX: signal.SIGALRM
            def _handler(signum, frame):
                raise HookTimeoutError(f"hook timeout after {seconds}s")

            old_handler = signal.signal(signal.SIGALRM, _handler)
            signal.setitimer(signal.ITIMER_REAL, seconds)
            try:
                yield
            finally:
                signal.setitimer(signal.ITIMER_REAL, 0)
                signal.signal(signal.SIGALRM, old_handler)

    def _execute_with_timeout(
        self,
        handler_fn: Callable,
        event: "Event",
        timeout_ms: int,
    ) -> "HookResult":
        """帶 timeout 執行 handler.

        Args:
            handler_fn: callable (event, context) -> HookResult
            event: 触发的事件
            timeout_ms: 超時阈值 (per hook.timeout_ms)

        Returns:
            HookResult
        """
        context = {
            "session_id": event.session_id,
            "event_id": event.event_id,
            "timestamp": event.timestamp,
        }
        with self._timeout(timeout_ms / 1000.0):
            return handler_fn(event, context)


# Late imports for type hints
from .event_emitter import Event, HookResult  # noqa: E402
from .hook_registry import Hook  # noqa: E402
from .audit_logger import AuditLogger  # noqa: E402