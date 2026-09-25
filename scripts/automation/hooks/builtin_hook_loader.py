"""BL-6: Builtin Hook Loader — builtin hook (PreToolUse guard 等) 預設裝載 + 不可刪除 (per BD §4.7 + DD §2.6).

5 builtin hooks (per BD §4.7 v0.1):
    1. pre_tool_use_guard: PreToolUse guard (per DD-PRE-TOOL-USE-GUARD-001) — 复用 guardian/pre_tool_use_guard.py
    2. session_start_cleanup: SessionStart 清理临时文件 (e.g. .multica/tmp/*.lock > 7 天)
    3. session_end_summary: SessionEnd 生成 session 总结 (count audit log entries, 写 reports/)
    4. subagent_dispatch_audit: SubagentDispatch 前置审计 (per §NFR-S-6: 100% 覆盖)
    5. tool_error_fallback: ToolError 自动 fallback (e.g. retry 1 次 + 记录到 audit log)

所有 builtin hook:
    - 启动时自动装载 (per FR-3.3)
    - 用户可在 UI 禁用 (enabled=false, per UI §FR-6.2)
    - 不可删除 / 不可修改 (per UI / registry PermissionError, per §NFR-S-4)
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import importlib
import logging
import os
from pathlib import Path
from typing import Any, Dict, List, Optional


logger = logging.getLogger(__name__)


# 5 builtin hook 名称 (per BD §4.7)
BUILTIN_HOOK_NAMES: List[str] = [
    "pre_tool_use_guard",
    "session_start_cleanup",
    "session_end_summary",
    "subagent_dispatch_audit",
    "tool_error_fallback",
]


class BuiltinHookLoader:
    """Builtin Hook Loader 主類 (per BD §4.7 + DD §2.6)."""

    def __init__(self) -> None:
        self._loaded: List["Hook"] = []

    def load_all(self) -> List["Hook"]:
        """从 scripts/automation/hooks/builtin/ 加載所有 builtin hook.

        Returns:
            List[Hook]: builtin hook 列表 (含 HOOK_DEF + is_builtin=True)
        """
        if self._loaded:
            return list(self._loaded)
        from datetime import datetime, timezone
        from .hook_registry import Hook

        now = datetime.now(timezone.utc).isoformat()
        loaded: List[Hook] = []
        for hook_name in BUILTIN_HOOK_NAMES:
            try:
                module = importlib.import_module(
                    f"scripts.automation.hooks.builtin.{hook_name}"
                )
                hook_def: Dict[str, Any] = dict(getattr(module, "HOOK_DEF"))
            except (ImportError, AttributeError) as e:
                # builtin hook 缺装, 记 warn 并跳过 (per §NFR-A-1 fail-open)
                logger.warning("builtin hook %s 加载失败, 跳过: %s", hook_name, e)
                continue
            hook_def["is_builtin"] = True
            hook_def.setdefault("enabled", True)
            hook_def.setdefault("archived", False)
            hook_def["created_at"] = now
            hook_def["updated_at"] = now
            try:
                hook = Hook.from_dict(hook_def)
                loaded.append(hook)
            except (KeyError, TypeError) as e:
                logger.warning("builtin hook %s schema 错: %s", hook_name, e)
                continue
        self._loaded = loaded
        logger.info("builtin hook 装载完成: %d/%d", len(loaded), len(BUILTIN_HOOK_NAMES))
        return list(self._loaded)

    def is_builtin(self, name: str) -> bool:
        """检查 hook 名是否是 builtin."""
        return name in BUILTIN_HOOK_NAMES


# Late imports
from .hook_registry import Hook  # noqa: E402