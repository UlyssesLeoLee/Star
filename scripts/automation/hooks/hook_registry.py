"""HR-2: Hook Registry — registry.json 加載/熱更新/schema 校驗/CRUD (per BD §4.3 + DD §2.2).

依赖:
    - jsonschema (可选, 用于严格 schema 校验, 缺则降级到最小自检 per §3.1 fail-open)
    - watchdog (可选, 用于 mtime 监听, 缺则手动 reload per §10.1)

W/M 表存储: scripts/automation/hooks/registry.json (per BD §5.2).
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import json
import logging
import os
import re
import shutil
import tempfile
import threading
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


logger = logging.getLogger(__name__)


# 14 类事件 (跟 EventType 枚举保持一致)
_EVENT_TYPES = frozenset({
    "PreToolUse", "PostToolUse", "UserPromptSubmit",
    "SessionStart", "SessionEnd", "SubagentDispatch",
    "SubagentReturn", "ToolError", "FileWatch",
    "CronTick", "RuntimeScan", "WorkspaceSwitch",
    "NetworkEgress", "CustomEvent",
})
_ACTION_TYPES = frozenset({"pre", "post", "block", "transform"})

# 最小自检 schema (per BD §4.3, 跟 jsonschema 草案对齐)
HOOK_SCHEMA: Dict[str, Any] = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
        "name": {"type": "string", "pattern": r"^[a-z0-9_-]+$"},
        "event_type": {"enum": sorted(_EVENT_TYPES)},
        "action_type": {"enum": sorted(_ACTION_TYPES)},
        "handler": {"type": "string"},
        "enabled": {"type": "boolean", "default": True},
        "archived": {"type": "boolean", "default": False},
        "priority": {"type": "integer", "minimum": 0, "maximum": 1000, "default": 100},
        "parallel": {"type": "boolean", "default": False},
        "timeout_ms": {"type": "integer", "minimum": 100, "maximum": 30000, "default": 1000},
        "retry": {"type": "integer", "minimum": 0, "maximum": 3, "default": 0},
        "description": {"type": "string"},
        "is_builtin": {"type": "boolean", "default": False},
        "created_at": {"type": "string", "format": "date-time"},
        "updated_at": {"type": "string", "format": "date-time"},
        "disabled_at": {"type": "string", "format": "date-time"},
        "archived_at": {"type": "string", "format": "date-time"},
    },
    "required": ["name", "event_type", "action_type", "handler"],
}


def _minimal_validate(hook_def: Dict[str, Any]) -> Optional[str]:
    """最小自检: name / event_type / action_type / handler + 必填字段.

    Returns:
        None: 校验通过
        str: 校验失败原因
    """
    name = hook_def.get("name")
    if not isinstance(name, str) or not re.match(r"^[a-z0-9_-]+$", name):
        return f"invalid name: {name!r}"
    et = hook_def.get("event_type")
    if et not in _EVENT_TYPES:
        return f"invalid event_type: {et!r}"
    at = hook_def.get("action_type")
    if at not in _ACTION_TYPES:
        return f"invalid action_type: {at!r}"
    h = hook_def.get("handler")
    if not isinstance(h, str) or not h:
        return f"invalid handler: {h!r}"
    p = hook_def.get("priority")
    if p is not None and (not isinstance(p, int) or p < 0 or p > 1000):
        return f"invalid priority: {p!r}"
    t = hook_def.get("timeout_ms")
    if t is not None and (not isinstance(t, int) or t < 100 or t > 30000):
        return f"invalid timeout_ms: {t!r}"
    r = hook_def.get("retry")
    if r is not None and (not isinstance(r, int) or r < 0 or r > 3):
        return f"invalid retry: {r!r}"
    return None


def _jsonschema_validate(hook_def: Dict[str, Any]) -> Optional[str]:
    """严格 schema 校验 (jsonschema 可选, 缺则返回 None 退到最小自检)."""
    try:
        import jsonschema  # type: ignore
    except ImportError:
        return None
    try:
        jsonschema.validate(hook_def, HOOK_SCHEMA)
        return None
    except Exception as e:  # ValidationError + 其他
        return f"jsonschema: {e}"


def _full_validate(hook_def: Dict[str, Any]) -> Optional[str]:
    """组合校验: jsonschema 优先, 缺则退到最小自检."""
    js_err = _jsonschema_validate(hook_def)
    if js_err is None:
        return _minimal_validate(hook_def)
    # jsonschema 报错, 直接返回
    return js_err


@dataclass
class Hook:
    """Hook 物件 (per BD §4.3)."""

    name: str
    event_type: str
    action_type: str
    handler: str
    enabled: bool = True
    archived: bool = False
    priority: int = 100
    parallel: bool = False
    timeout_ms: int = 1000
    retry: int = 0
    description: str = ""
    is_builtin: bool = False
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    disabled_at: Optional[str] = None
    archived_at: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Hook":
        return cls(
            name=data["name"],
            event_type=data["event_type"],
            action_type=data["action_type"],
            handler=data["handler"],
            enabled=bool(data.get("enabled", True)),
            archived=bool(data.get("archived", False)),
            priority=int(data.get("priority", 100)),
            parallel=bool(data.get("parallel", False)),
            timeout_ms=int(data.get("timeout_ms", 1000)),
            retry=int(data.get("retry", 0)),
            description=data.get("description", "") or "",
            is_builtin=bool(data.get("is_builtin", False)),
            created_at=data.get("created_at"),
            updated_at=data.get("updated_at"),
            disabled_at=data.get("disabled_at"),
            archived_at=data.get("archived_at"),
        )

    def to_dict(self) -> Dict[str, Any]:
        """转 dict, 过滤 None 字段."""
        out: Dict[str, Any] = {
            "name": self.name,
            "event_type": self.event_type,
            "action_type": self.action_type,
            "handler": self.handler,
            "enabled": self.enabled,
            "archived": self.archived,
            "priority": self.priority,
            "parallel": self.parallel,
            "timeout_ms": self.timeout_ms,
            "retry": self.retry,
            "description": self.description,
            "is_builtin": self.is_builtin,
        }
        for k in ("created_at", "updated_at", "disabled_at", "archived_at"):
            v = getattr(self, k)
            if v is not None:
                out[k] = v
        return out


class HookRegistry:
    """Hook Registry 主類 (per BD §4.3 + DD §2.2)."""

    def __init__(self, registry_path: Path, builtin_names: Optional[List[str]] = None):
        self._registry_path = Path(registry_path)
        self._builtin_names = set(builtin_names or [])
        self._hooks: Dict[str, Hook] = {}
        self._lock = threading.RLock()
        self._observer = None  # watchdog Observer 可选
        self._mtime: Optional[float] = None

    # ------------------- load / reload -------------------

    def load(self, builtin_hooks: Optional[List[Hook]] = None) -> List[Hook]:
        """从 registry.json 加載所有 hook, 含 builtin hook 合并.

        流程 (per DD §2.2):
        1. 讀 registry.json (per fail-open)
        2. 校驗 schema (per fail-open)
        3. 合并 builtin hook (per BL-6 load_all 返回)
        4. 啟動 watchdog 监听 (可选)

        Returns:
            List[Hook]: 所有 hook (含 builtin)
        """
        with self._lock:
            self._hooks.clear()
            try:
                if self._registry_path.exists():
                    raw = self._registry_path.read_text(encoding="utf-8")
                    data = json.loads(raw)
                    self._mtime = self._registry_path.stat().st_mtime
                    for hook_data in data.get("hooks", []):
                        err = _full_validate(hook_data)
                        if err:
                            logger.warning(
                                "registry.json schema 校驗失敗, 跳过 hook %s: %s",
                                hook_data.get("name"), err,
                            )
                            continue
                        try:
                            hook = Hook.from_dict(hook_data)
                            self._hooks[hook.name] = hook
                        except (KeyError, TypeError) as e:
                            logger.warning(
                                "registry.json hook 解析失敗: %s",
                                e,
                            )
                else:
                    # 首次启动, registry.json 不存在, fail-open
                    self._mtime = None
                    logger.info(
                        "registry.json 不存在 (%s), fail-open: 仅 builtin hook 可用",
                        self._registry_path,
                    )
            except (json.JSONDecodeError, OSError) as e:
                # per SRS §NFR-A-1: fail-open
                logger.warning("registry.json 加載失敗, fail-open: %s", e)
                self._hooks.clear()

            # 合并 builtin hook
            if builtin_hooks:
                for bh in builtin_hooks:
                    self._hooks[bh.name] = bh

            # 启动 watchdog (可选, 缺则静默跳过)
            self._start_watchdog()

            return list(self._hooks.values())

    def reload(self, builtin_hooks: Optional[List[Hook]] = None) -> None:
        """热更新: 從 registry.json 重新加載."""
        self.load(builtin_hooks=builtin_hooks)

    def check_reload(self) -> bool:
        """手动检查 mtime, 若变化则 reload. (watchdog 缺时的回退路径 per §10.1)."""
        if not self._registry_path.exists():
            return False
        try:
            mtime = self._registry_path.stat().st_mtime
        except OSError:
            return False
        if self._mtime is None or mtime > self._mtime:
            self.reload()
            return True
        return False

    # ------------------- CRUD -------------------

    def register(self, hook_def: Dict[str, Any]) -> Hook:
        """創建 1 个 hook, 寫入 registry.json.

        Raises:
            ValueError: schema 校驗失敗
            FileExistsError: name 已存在 (且未 archived)
            PermissionError: builtin hook 重名
        """
        err = _full_validate(hook_def)
        if err:
            raise ValueError(f"schema 校驗失敗: {err}")
        name = hook_def["name"]
        with self._lock:
            if name in self._builtin_names:
                raise PermissionError(f"builtin hook {name} 不可注册 (由 BL-6 管理)")
            if name in self._hooks and not self._hooks[name].archived:
                raise FileExistsError(f"Hook {name} 已存在")
            now = datetime.now(timezone.utc).isoformat()
            hook_def["created_at"] = now
            hook_def["updated_at"] = now
            hook_def.setdefault("enabled", True)
            hook_def.setdefault("archived", False)
            hook_def.setdefault("priority", 100)
            hook_def.setdefault("parallel", False)
            hook_def.setdefault("timeout_ms", 1000)
            hook_def.setdefault("retry", 0)
            hook = Hook.from_dict(hook_def)
            self._hooks[name] = hook
            self._persist()
            return hook

    def update(self, name: str, hook_def: Dict[str, Any]) -> Hook:
        """更新 1 个 hook."""
        err = _full_validate(hook_def)
        if err:
            raise ValueError(f"schema 校驗失敗: {err}")
        with self._lock:
            if name not in self._hooks:
                raise KeyError(f"Hook {name} 不存在")
            if self._hooks[name].is_builtin:
                raise PermissionError("builtin hook 不可修改 (僅可 enable/disable)")
            hook_def["updated_at"] = datetime.now(timezone.utc).isoformat()
            hook_def.setdefault("created_at", self._hooks[name].created_at)
            hook = Hook.from_dict(hook_def)
            self._hooks[name] = hook
            self._persist()
            return hook

    def enable(self, name: str) -> Hook:
        """啟用 hook."""
        return self._set_flag(name, "enabled", True)

    def disable(self, name: str) -> Hook:
        """禁用 hook, 記錄 disabled_at."""
        with self._lock:
            if name not in self._hooks:
                raise KeyError(f"Hook {name} 不存在")
            self._hooks[name].disabled_at = datetime.now(timezone.utc).isoformat()
            return self._set_flag(name, "enabled", False)

    def archive(self, name: str) -> Hook:
        """歸檔 hook (不物理刪除, 30 天内可复活 per SRS §FR-1.4)."""
        with self._lock:
            if name not in self._hooks:
                raise KeyError(f"Hook {name} 不存在")
            if self._hooks[name].is_builtin:
                raise PermissionError("builtin hook 不可歸檔")
            self._hooks[name].archived = True
            self._hooks[name].archived_at = datetime.now(timezone.utc).isoformat()
            self._hooks[name].enabled = False
            self._persist()
            return self._hooks[name]

    def get(self, name: str) -> Optional[Hook]:
        """按 name 查詢 1 个 hook."""
        with self._lock:
            return self._hooks.get(name)

    def list_by_event(self, event_type: str) -> List[Hook]:
        """按 event_type 查詢所有启用的 hook (含 builtin).

        過濾條件: event_type 匹配 + enabled=true + archived=false
        按 priority 升序排序 (per DD §2.2 list_by_event)
        """
        with self._lock:
            hooks = [
                h for h in self._hooks.values()
                if h.event_type == event_type and h.enabled and not h.archived
            ]
            return sorted(hooks, key=lambda h: h.priority)

    def list_all(self) -> List[Hook]:
        """查询所有 hook (含 archived)."""
        with self._lock:
            return list(self._hooks.values())

    def list_builtin(self) -> List[Hook]:
        """查询所有 builtin hook."""
        with self._lock:
            return [h for h in self._hooks.values() if h.is_builtin]

    # ------------------- internal -------------------

    def _set_flag(self, name: str, flag: str, value: bool) -> Hook:
        with self._lock:
            if name not in self._hooks:
                raise KeyError(f"Hook {name} 不存在")
            setattr(self._hooks[name], flag, value)
            self._hooks[name].updated_at = datetime.now(timezone.utc).isoformat()
            self._persist()
            return self._hooks[name]

    def _persist(self) -> None:
        """原子写: 临时文件 → rename (per BD §4.3 _persist)."""
        data = {
            "version": "1.0",
            "hooks": [
                h.to_dict() for h in self._hooks.values() if not h.is_builtin
            ],
        }
        # 原子写: tmp dir + replace
        tmp_dir = self._registry_path.parent
        tmp_dir.mkdir(parents=True, exist_ok=True)
        fd, tmp_path = tempfile.mkstemp(
            dir=tmp_dir,
            prefix=".registry.",
            suffix=".tmp",
        )
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
                f.flush()
            shutil.move(tmp_path, self._registry_path)
            self._mtime = self._registry_path.stat().st_mtime
        except Exception:
            # 清理 tmp, 避免污染
            try:
                os.unlink(tmp_path)
            except OSError:
                pass
            raise

    def _start_watchdog(self) -> None:
        """启动 watchdog 监听 registry.json mtime (per FR-3.2). 缺则静默跳过."""
        if self._observer is not None:
            return
        try:
            from watchdog.observers import Observer  # type: ignore
            from watchdog.events import FileSystemEventHandler  # type: ignore
        except ImportError:
            logger.debug("watchdog 不可用, 跳过 mtime 监听 (回退到手动 check_reload)")
            return
        registry_ref = self

        class _Handler(FileSystemEventHandler):
            def on_modified(self, event):
                if not event.is_directory and Path(event.src_path).name == registry_ref._registry_path.name:
                    logger.info("registry.json mtime 变化, 触发 reload")
                    registry_ref.reload()

        try:
            obs = Observer()
            obs.schedule(_Handler(), str(self._registry_path.parent), recursive=False)
            obs.daemon = True
            obs.start()
            self._observer = obs
        except Exception as e:
            logger.warning("watchdog 启动失敗: %s", e)
            self._observer = None

    def stop(self) -> None:
        """停止 watchdog."""
        if self._observer is not None:
            try:
                self._observer.stop()
                self._observer.join(timeout=2.0)
            except Exception:
                pass
            self._observer = None