"""scripts/automation/hooks/ — Multica Hook 域 (per SRS-MULTICA-HOOK-001 + BD-MULTICA-HOOK-001 + DD-MULTICA-HOOK-001, v0.1, 2026-09-24).

ULYS-235 v0.5 落档: 6 module 物理骨架 + 5 builtin hook + registry.json 初始内容 + audit log mock.

Module 索引 (per BD §4.1):
    EM-1  event_emitter        14 类事件定義 + payload 構造 + 事件分發入口
    HR-2  hook_registry        registry.json 加載/熱更新/schema 校驗/CRUD
    FS-3  fanout_scheduler     per-event 多 hook 調度 + 串行/並行 + 優先級排序
    HR-4  hook_runner          単 hook 執行 + timeout + retry + transform 累積
    AL-5  audit_logger         JSON Lines 寫 hook_audit.log + Transaction append-only
    BL-6  builtin_hook_loader  builtin hook (PreToolUse guard 等) 預設裝載

約束 (per scripts/automation/__init__.py "标准库 only"):
    - 仅依赖 Python stdlib (json, pathlib, threading, dataclasses, enum, hashlib, time, uuid, logging, datetime)
    - jsonschema 校验回退: 仅在可选 import 成功时启用, 否则用最小自检 (per §3.1 fail-open)
    - watchdog mtime 监听回退: 仅在可选 import 成功时启用, 否则手动 reload (per §10.1)

W/T/M 三表 (per SRS §FR-5.1 + BD §5.1 + 守门 #13):
    - TBL-HOOK        hooks                      W/M  registry.json
    - TBL-HOOK-RUN    hook_runs                  T    logs/hook_audit.log (JSON Lines, append-only)
    - TBL-SESSION     hook_session_state         M    state/session_state.json

跨域契约 (per SRS §FR-7.2 + BD §6.5):
    - skill registry: docs/skills/<name>/SKILL.md (独立, 不冲突)
    - hook registry: scripts/automation/hooks/registry.json (本模块)
    - session state: scripts/automation/hooks/state/session_state.json (跟 skills 域共享, 仅读 API)
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from .event_emitter import (
    EventType,
    Event,
    HookResult,
    EventResult,
    EventEmitter,
    VALIDATORS,
)
from .hook_registry import (
    HOOK_SCHEMA,
    Hook,
    HookRegistry,
)
from .fanout_scheduler import (
    DECISION_PRIORITY,
    FanoutScheduler,
)
from .hook_runner import (
    HookTimeoutError,
    HookRunner,
)
from .audit_logger import (
    AuditLogWriteError,
    AuditLogEntry,
    AuditLogger,
)
from .builtin_hook_loader import (
    BUILTIN_HOOK_NAMES,
    BuiltinHookLoader,
)

__all__ = [
    # EM-1
    "EventType",
    "Event",
    "HookResult",
    "EventResult",
    "EventEmitter",
    "VALIDATORS",
    # HR-2
    "HOOK_SCHEMA",
    "Hook",
    "HookRegistry",
    # FS-3
    "DECISION_PRIORITY",
    "FanoutScheduler",
    # HR-4
    "HookTimeoutError",
    "HookRunner",
    # AL-5
    "AuditLogWriteError",
    "AuditLogEntry",
    "AuditLogger",
    # BL-6
    "BUILTIN_HOOK_NAMES",
    "BuiltinHookLoader",
]

__version__ = "0.1.0"