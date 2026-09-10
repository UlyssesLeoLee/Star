"""RuleDatabase PM-2: 規則加載 / 熱更新 / schema 校驗.

Per DD-PRE-TOOL-USE-GUARD-001.md §2.2 + SRS-PRE-TOOL-USE-GUARD-001.md §4.3.1.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import logging
import re
import threading
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional

logger = logging.getLogger(__name__)

# 規則 schema (per BD §4.3.1, simplified for stdlib-only runtime)
RULE_SCHEMA_VERSION = "1.0"
RULE_ID_RE = re.compile(r"^R-(BLOCK|ASK|WARN)-\d{3}$")
LEVELS = ("BLOCK", "ASK", "WARN")
TOOL_CLASSES = ("bash", "write", "edit", "read", "mcp")


@dataclass(frozen=True)
class Rule:
    """单条危险模式规则."""
    id: str
    name: str
    level: str          # BLOCK / ASK / WARN
    tool_class: str
    pattern: str
    reason: str
    _compiled: "Optional[re.Pattern]" = None

    def match(self, text: str) -> bool:
        """跑 regex, return True if 命中."""
        if not text:
            return False
        if self._compiled is None:
            return False
        try:
            return self._compiled.search(text) is not None
        except re.error as e:
            # per F-7: regex catastrophic backtracking -> skip + WARN
            logger.warning("Rule %s regex error: %s", self.id, e)
            return False


def _validate_schema(data: dict) -> List[str]:
    """手动 schema 校验 (avoid jsonschema 依赖, per DD §1.2 原则).

    Returns list of error messages, empty if valid.
    """
    errors: List[str] = []
    if not isinstance(data, dict):
        return ["root must be object"]
    sv = data.get("schema_version", "")
    if not re.match(r"^\d+\.\d+$", str(sv)):
        errors.append(f"schema_version invalid: {sv!r}")
    rules = data.get("rules")
    if not isinstance(rules, list) or len(rules) < 1:
        errors.append("rules must be non-empty array")
        return errors
    seen_ids = set()
    for i, r in enumerate(rules):
        if not isinstance(r, dict):
            errors.append(f"rules[{i}] must be object")
            continue
        rid = r.get("id", "")
        if not isinstance(rid, str) or not RULE_ID_RE.match(rid):
            errors.append(f"rules[{i}].id invalid: {rid!r}")
        elif rid in seen_ids:
            errors.append(f"rules[{i}].id duplicate: {rid}")
        else:
            seen_ids.add(rid)
        for fld in ("name", "level", "tool_class", "pattern", "reason"):
            if fld not in r:
                errors.append(f"rules[{i}].{fld} missing")
        if r.get("level") not in LEVELS:
            errors.append(f"rules[{i}].level invalid: {r.get('level')!r}")
        if r.get("tool_class") not in TOOL_CLASSES:
            errors.append(f"rules[{i}].tool_class invalid: {r.get('tool_class')!r}")
        pat = r.get("pattern", "")
        if not isinstance(pat, str) or not (1 <= len(pat) <= 500):
            errors.append(f"rules[{i}].pattern length invalid: {len(pat) if isinstance(pat, str) else 'not-str'}")
    return errors


class RuleLoadError(Exception):
    """規則加載失敗, 用于 fail-open 判斷."""
    pass


class RuleDatabase:
    """PM-2: 規則加載/熱更新/schema 校驗.

    線程安全 (RLock), watchdog 可选 (graceful fallback to no-watcher).
    """

    def __init__(self, rules_path: Path, enable_watcher: bool = True):
        self._path = Path(rules_path)
        self._rules: List[Rule] = []
        self._lock = threading.RLock()
        self._observer = None
        self._load_rules()
        if enable_watcher:
            self._try_start_watcher()

    def get_rules(self) -> List[Rule]:
        """線程安全讀取, 返回拷貝避免外部修改."""
        with self._lock:
            return list(self._rules)

    def _load_rules(self) -> None:
        """加載規則, 失敗 → 保留舊 rules + WARN log (per FR-6.1)."""
        try:
            with self._path.open("r", encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, json.JSONDecodeError) as e:
            logger.warning("Rule load failed (IO/JSON), keeping old rules: %s", e)
            return
        errors = _validate_schema(data)
        if errors:
            logger.warning("Rule schema validation failed, keeping old rules: %s", errors[:3])
            return
        new_rules: List[Rule] = []
        for r in data["rules"]:
            try:
                compiled = re.compile(r["pattern"], re.MULTILINE | re.DOTALL)
            except re.error as e:
                logger.warning("Rule %s pattern compile failed, skipping: %s", r["id"], e)
                continue
            new_rules.append(Rule(
                id=r["id"],
                name=r["name"],
                level=r["level"],
                tool_class=r["tool_class"],
                pattern=r["pattern"],
                reason=r["reason"],
                _compiled=compiled,
            ))
        with self._lock:
            self._rules = new_rules
        logger.info("Loaded %d rules from %s", len(new_rules), self._path)

    def _try_start_watcher(self) -> None:
        """嘗試啟動 watchdog, 失敗 graceful fallback (no-watcher, 仍可手動 reload)."""
        try:
            from watchdog.observers import Observer
            from watchdog.events import FileSystemEventHandler
        except ImportError:
            logger.info("watchdog not installed, skipping file watcher (manual reload only)")
            return
        try:
            db = self

            class _Handler(FileSystemEventHandler):
                def on_modified(self, event):
                    if not event.is_directory:
                        try:
                            if Path(event.src_path).resolve() == db._path.resolve():
                                logger.info("Rules file modified, reloading...")
                                db._load_rules()
                        except OSError:
                            pass

            self._observer = Observer()
            self._observer.schedule(_Handler(), str(self._path.parent), recursive=False)
            self._observer.daemon = True
            self._observer.start()
            logger.info("File watcher started for %s", self._path)
        except Exception as e:
            # per F-5: watchdog 啟動失敗 -> 不影響主流程
            logger.warning("File watcher failed, manual reload only: %s", e)
            self._observer = None

    def stop(self) -> None:
        if self._observer is not None:
            try:
                self._observer.stop()
                self._observer.join(timeout=2)
            except Exception:
                pass
            self._observer = None

    def reload(self) -> int:
        """手動 reload, 返回當前 rules 數量 (for testing)."""
        self._load_rules()
        return len(self.get_rules())
