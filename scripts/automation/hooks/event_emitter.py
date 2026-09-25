"""EM-1: Event Emitter — 14 类事件定義 + payload 構造 + 事件分發入口 (per BD §4.2 + DD §2.1).

跟 FS-3 + AL-5 解耦: EM-1 只負責構造事件 + payload 校验, fan-out + audit 委托给 scheduler/runner.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Any, Callable, Dict, List, Optional


logger = logging.getLogger(__name__)


class EventType(str, Enum):
    """14 类事件枚举 (per SRS §FR-2.1)."""

    PRE_TOOL_USE = "PreToolUse"
    POST_TOOL_USE = "PostToolUse"
    USER_PROMPT_SUBMIT = "UserPromptSubmit"
    SESSION_START = "SessionStart"
    SESSION_END = "SessionEnd"
    SUBAGENT_DISPATCH = "SubagentDispatch"
    SUBAGENT_RETURN = "SubagentReturn"
    TOOL_ERROR = "ToolError"
    FILE_WATCH = "FileWatch"
    CRON_TICK = "CronTick"
    RUNTIME_SCAN = "RuntimeScan"
    WORKSPACE_SWITCH = "WorkspaceSwitch"
    NETWORK_EGRESS = "NetworkEgress"
    CUSTOM_EVENT = "CustomEvent"


# per-event payload 必填字段 (per BD §4.2 表)
PAYLOAD_SCHEMA: Dict[EventType, List[str]] = {
    EventType.PRE_TOOL_USE: ["tool_name", "tool_args", "agent_role"],
    EventType.POST_TOOL_USE: ["tool_name", "tool_result", "latency_ms"],
    EventType.USER_PROMPT_SUBMIT: ["prompt", "user_id"],
    EventType.SESSION_START: ["session_id", "workspace", "runtime"],
    EventType.SESSION_END: ["session_id", "duration_ms", "exit_reason"],
    EventType.SUBAGENT_DISPATCH: ["task_id", "script_path", "args", "parent_session_id"],
    EventType.SUBAGENT_RETURN: ["task_id", "result", "success"],
    EventType.TOOL_ERROR: ["tool_name", "error", "stack_trace"],
    EventType.FILE_WATCH: ["file_path", "event_type", "mtime"],
    EventType.CRON_TICK: ["cron_id", "scheduled_at"],
    EventType.RUNTIME_SCAN: ["runtimes", "poisoned"],
    EventType.WORKSPACE_SWITCH: ["from_workspace", "to_workspace"],
    EventType.NETWORK_EGRESS: ["url", "method", "headers"],
    EventType.CUSTOM_EVENT: ["name", "payload"],
}


# payload validator (per DD §2.1 _validate_payload)
def _validate_pre_tool_use(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("tool_name"), str)
        and isinstance(p.get("tool_args"), dict)
        and isinstance(p.get("agent_role"), str)
    )


def _validate_post_tool_use(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("tool_name"), str)
        and "tool_result" in p
        and isinstance(p.get("latency_ms"), (int, float))
    )


def _validate_session_start(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("session_id"), str)
        and isinstance(p.get("workspace"), str)
        and isinstance(p.get("runtime"), str)
    )


def _validate_session_end(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("session_id"), str)
        and isinstance(p.get("duration_ms"), (int, float))
        and isinstance(p.get("exit_reason"), str)
    )


def _validate_subagent_dispatch(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("task_id"), str)
        and isinstance(p.get("script_path"), str)
        and isinstance(p.get("args"), dict)
        and isinstance(p.get("parent_session_id"), str)
    )


def _validate_subagent_return(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("task_id"), str)
        and "result" in p
        and isinstance(p.get("success"), bool)
    )


def _validate_tool_error(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("tool_name"), str)
        and isinstance(p.get("error"), str)
        and isinstance(p.get("stack_trace"), str)
    )


def _validate_file_watch(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("file_path"), str)
        and isinstance(p.get("event_type"), str)
        and isinstance(p.get("mtime"), (int, float))
    )


def _validate_user_prompt_submit(p: Dict[str, Any]) -> bool:
    return isinstance(p.get("prompt"), str) and isinstance(p.get("user_id"), str)


def _validate_cron_tick(p: Dict[str, Any]) -> bool:
    return isinstance(p.get("cron_id"), str) and "scheduled_at" in p


def _validate_runtime_scan(p: Dict[str, Any]) -> bool:
    return isinstance(p.get("runtimes"), list) and isinstance(p.get("poisoned"), list)


def _validate_workspace_switch(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("from_workspace"), str)
        and isinstance(p.get("to_workspace"), str)
    )


def _validate_network_egress(p: Dict[str, Any]) -> bool:
    return (
        isinstance(p.get("url"), str)
        and isinstance(p.get("method"), str)
        and isinstance(p.get("headers"), dict)
    )


def _validate_custom_event(p: Dict[str, Any]) -> bool:
    return isinstance(p.get("name"), str) and isinstance(p.get("payload"), dict)


VALIDATORS: Dict[EventType, Callable[[Dict[str, Any]], bool]] = {
    EventType.PRE_TOOL_USE: _validate_pre_tool_use,
    EventType.POST_TOOL_USE: _validate_post_tool_use,
    EventType.USER_PROMPT_SUBMIT: _validate_user_prompt_submit,
    EventType.SESSION_START: _validate_session_start,
    EventType.SESSION_END: _validate_session_end,
    EventType.SUBAGENT_DISPATCH: _validate_subagent_dispatch,
    EventType.SUBAGENT_RETURN: _validate_subagent_return,
    EventType.TOOL_ERROR: _validate_tool_error,
    EventType.FILE_WATCH: _validate_file_watch,
    EventType.CRON_TICK: _validate_cron_tick,
    EventType.RUNTIME_SCAN: _validate_runtime_scan,
    EventType.WORKSPACE_SWITCH: _validate_workspace_switch,
    EventType.NETWORK_EGRESS: _validate_network_egress,
    EventType.CUSTOM_EVENT: _validate_custom_event,
}


@dataclass(frozen=True)
class Event:
    """事件物件 (per BD §6.1)."""

    event_id: str
    event_type: str
    timestamp: str
    session_id: str
    payload: Dict[str, Any]

    @classmethod
    def create(
        cls,
        event_type: str,
        payload: Dict[str, Any],
        session_id: str,
    ) -> "Event":
        """工廠方法: 生成 UUID + ISO 8601 timestamp."""
        return cls(
            event_id=str(uuid.uuid4()),
            event_type=event_type,
            timestamp=datetime.now(timezone.utc).isoformat(),
            session_id=session_id,
            payload=dict(payload),
        )


@dataclass
class HookResult:
    """単 hook 執行結果 (per BD §6.1)."""

    decision: str  # BLOCK|ASK|WARN|PASS|transform
    reason: Optional[str] = None
    rule_id: Optional[str] = None
    transformed_args: Optional[Dict[str, Any]] = None
    latency_ms: float = 0.0


@dataclass
class EventResult:
    """事件聚合結果 (per BD §6.1)."""

    decision: str = "PASS"
    transformed_args: Optional[Dict[str, Any]] = None
    reason: str = ""
    latency_ms: float = 0.0
    hooks_executed: List[str] = field(default_factory=list)


class EventEmitter:
    """Event Emitter 主類 (per BD §4.2 + DD §2.1)."""

    def __init__(self, fanout_scheduler: "FanoutScheduler"):
        # Forward reference: FS-3 在 sibling module, 避免循环 import
        self._fanout_scheduler = fanout_scheduler

    def emit(
        self,
        event_type: str,
        payload: Dict[str, Any],
        session_id: str,
    ) -> EventResult:
        """触发 1 个事件, fan-out 到所有匹配的 hook.

        流程 (per BD §4.4 FS-3 调度算法):
        1. 構造 Event 物件 (含 UUID + timestamp)
        2. 校驗 payload schema (per event_type 不同, e.g. PreToolUse 必有 tool_name)
        3. 調用 FS-3.fanout(event) 取得聚合 EventResult
        4. 返回 EventResult (audit log 由 FS-3 / HR-4 内部寫)

        Args:
            event_type: 14 类事件之一 (e.g. "PreToolUse")
            payload: per-event payload (e.g. {"tool_name": "bash", "tool_args": {...}})
            session_id: 當前 session UUID

        Returns:
            EventResult: 聚合決策 + transformed_args + 執行 hook 列表

        Raises:
            ValueError: payload schema 校驗失敗 (上層處理)
        """
        # Step 1: 構造 Event 物件
        event = Event.create(event_type, payload, session_id)

        # Step 2: 校驗 payload schema
        self._validate_payload(event_type, payload)

        # Step 3: fan-out (audit log 寫入委托给 FS-3 / HR-4)
        result = self._fanout_scheduler.fanout(event)

        return result

    def _validate_payload(self, event_type: str, payload: Dict[str, Any]) -> None:
        """校驗 payload schema (per event_type 不同).

        PreToolUse: 必有 tool_name (str) + tool_args (dict) + agent_role (str)
        PostToolUse: 必有 tool_name (str) + tool_result (any) + latency_ms (float)
        ... (per SRS §4.2.1 14 类事件 payload, DD §2.1 VALIDATORS)

        校驗失敗 → ValueError (上層處理)
        """
        try:
            et = EventType(event_type)
        except ValueError:
            raise ValueError(f"Unknown event_type: {event_type!r}")
        validator = VALIDATORS.get(et)
        if validator and not validator(payload):
            required = PAYLOAD_SCHEMA.get(et, [])
            raise ValueError(
                f"Invalid payload for {event_type}: missing/invalid fields, "
                f"required={required}, got={list(payload.keys())}"
            )


# Late import for FanoutScheduler (避免循环)
from .fanout_scheduler import FanoutScheduler  # noqa: E402