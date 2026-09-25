"""AL-5: Audit Logger — JSON Lines 寫 hook_audit.log + Transaction append-only (per BD §4.6 + DD §2.5).

T 表存储: scripts/automation/hooks/logs/hook_audit.log (append-only, per 守门 #13 + §NFR-S-2).
12 字段 (per BD §4.6 + DD §2.5 log_hook_run): run_id / timestamp / session_id / event_type / hook_name /
    action_type / decision / rule_id? / transformed_args? / before_args / after_args / env_hash / latency_ms
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import hashlib
import json
import logging
import os
import stat
import uuid
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


logger = logging.getLogger(__name__)


class AuditLogWriteError(Exception):
    """audit log 寫失敗 → fail-closed (per SRS §FR-5.2)."""
    pass


@dataclass
class AuditLogEntry:
    """audit log 单条记录 (per BD §4.6 12 字段)."""

    run_id: str
    timestamp: str
    session_id: str
    event_type: str
    hook_name: str
    action_type: str
    decision: str  # BLOCK|ASK|WARN|PASS|transform
    rule_id: Optional[str] = None
    transformed_args: Optional[Dict[str, Any]] = None
    before_args: Dict[str, Any] = field(default_factory=dict)
    after_args: Optional[Dict[str, Any]] = None
    env_hash: str = ""
    latency_ms: float = 0.0


class AuditLogger:
    """Audit Logger 主類 (per BD §4.6 + DD §2.5)."""

    def __init__(self, log_path: Path):
        self._path = Path(log_path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        if not self._path.exists():
            self._path.touch()
            try:
                # 0600 = mavis 进程可读写, 其他用户无权限 (per §NFR-S-2)
                os.chmod(self._path, stat.S_IRUSR | stat.S_IWUSR)
            except OSError:
                pass

    def log(
        self,
        hook_name: str,
        event_type: str,
        action_type: str,
        decision: str,
        session_id: str,
        before_args: Dict[str, Any],
        after_args: Optional[Dict[str, Any]] = None,
        rule_id: Optional[str] = None,
        transformed_args: Optional[Dict[str, Any]] = None,
        latency_ms: float = 0.0,
    ) -> AuditLogEntry:
        """写 1 条 audit log 记录, 返回 entry.

        Append-only (per 守门 #13 T 表 + §NFR-S-2). 失败 → AuditLogWriteError → fail-closed.
        """
        entry = AuditLogEntry(
            run_id=str(uuid.uuid4()),
            timestamp=datetime.now(timezone.utc).isoformat(),
            session_id=session_id,
            event_type=event_type,
            hook_name=hook_name,
            action_type=action_type,
            decision=decision,
            rule_id=rule_id,
            transformed_args=transformed_args,
            before_args=dict(before_args),
            after_args=after_args,
            env_hash=self._compute_env_hash(),
            latency_ms=round(float(latency_ms), 3),
        )
        self._write(entry)
        return entry

    def log_hook_run(
        self,
        hook: "Hook",
        event: "Event",
        result: "HookResult",
    ) -> AuditLogEntry:
        """便捷方法: 接受 Hook + Event + HookResult (per DD §2.5 原始签名).

        委托给 log(), 保持向后兼容.
        """
        return self.log(
            hook_name=hook.name,
            event_type=event.event_type,
            action_type=hook.action_type,
            decision=result.decision,
            session_id=event.session_id,
            before_args=dict(event.payload),
            after_args=result.transformed_args or dict(event.payload),
            rule_id=result.rule_id,
            transformed_args=result.transformed_args,
            latency_ms=result.latency_ms,
        )

    def query(
        self,
        hook_name: Optional[str] = None,
        event_type: Optional[str] = None,
        decision: Optional[str] = None,
        session_id: Optional[str] = None,
        start_time: Optional[str] = None,
        end_time: Optional[str] = None,
        limit: int = 50,
    ) -> List[AuditLogEntry]:
        """查询 audit log (per BD §4.6 query + DD §2.5 query).

        过滤: hook_name / event_type / decision / session_id / 时间范围
        分页: limit (默认 50, 最大 1000 per §FR-6.3)
        """
        if limit > 1000:
            limit = 1000
        if not self._path.exists():
            return []
        results: List[AuditLogEntry] = []
        try:
            with self._path.open("r", encoding="utf-8", errors="replace") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        data = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if hook_name and data.get("hook_name") != hook_name:
                        continue
                    if event_type and data.get("event_type") != event_type:
                        continue
                    if decision and data.get("decision") != decision:
                        continue
                    if session_id and data.get("session_id") != session_id:
                        continue
                    if start_time and data.get("timestamp", "") < start_time:
                        continue
                    if end_time and data.get("timestamp", "") > end_time:
                        continue
                    try:
                        results.append(AuditLogEntry(**{
                            k: data.get(k) for k in (
                                "run_id", "timestamp", "session_id",
                                "event_type", "hook_name", "action_type",
                                "decision", "rule_id", "transformed_args",
                                "before_args", "after_args", "env_hash",
                                "latency_ms",
                            )
                        }))
                    except TypeError:
                        continue
                    if len(results) >= limit:
                        break
        except (OSError, IOError) as e:
            logger.warning("audit log read failed: %s", e)
        return results

    def count_by_decision(self) -> Dict[str, int]:
        """按 decision 计数 (per NFR-O-1 命中统计)."""
        counts = {"BLOCK": 0, "ASK": 0, "WARN": 0, "PASS": 0, "transform": 0}
        for entry in self.query(limit=10_000_000):
            d = entry.decision
            if d in counts:
                counts[d] += 1
        return counts

    def rotate(self, keep_lines: int = 100_000) -> int:
        """手动 log rotation (per §10.2 BD).

        Args:
            keep_lines: 保留最后 N 行, 超出部分归档到 <log_path>.<ts>.gz
        Returns:
            rotated_lines: 归档的行数
        """
        if not self._path.exists():
            return 0
        lines = self._path.read_text(encoding="utf-8", errors="replace").splitlines()
        if len(lines) <= keep_lines:
            return 0
        # 保留最新 keep_lines 行 (tail), 旧行 archive
        archive = lines[:-keep_lines]   # 旧行 (将被 archive)
        keep = lines[-keep_lines:]       # 最新 keep_lines 行 (主 log 保留)
        # 写 archive (gzip if available)
        ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        archive_path = self._path.with_suffix(f".{ts}.log")
        try:
            import gzip
            with gzip.open(archive_path, "wt", encoding="utf-8") as f:
                f.write("\n".join(archive) + "\n")
        except ImportError:
            archive_path.write_text("\n".join(archive) + "\n", encoding="utf-8")
        # 重写主 log 为 keep (per §NFR-S-2 append-only: 仅写一次, 不物理删除)
        # 注: 严格 append-only 不允许重写, 但 DD §10.2 / §FR-5.1 明确要求 90 天 rotation.
        # 折中: rotation 只移动旧行到 archive, 主 log 保留最新 keep_lines.
        tmp = self._path.with_suffix(".rotate.tmp")
        tmp.write_text("\n".join(keep) + "\n", encoding="utf-8")
        os.replace(tmp, self._path)
        return len(archive)

    def _write(self, entry: AuditLogEntry) -> None:
        """追加写一行 JSON Lines (per 守门 #13 T append-only)."""
        try:
            d = asdict(entry)
            line = json.dumps(d, ensure_ascii=False, separators=(",", ":")) + "\n"
        except (TypeError, ValueError) as e:
            raise AuditLogWriteError(f"event serialize failed: {e}") from e
        try:
            with self._path.open("a", encoding="utf-8", errors="replace") as f:
                f.write(line)
                f.flush()
                # 不强制 fsync (per §NFR-P-3 < 1ms), OS lazy fsync
        except (OSError, IOError) as e:
            raise AuditLogWriteError(f"audit log write failed: {e}") from e

    def _compute_env_hash(self) -> str:
        """env var 哈希 (凭据 0 外泄 per §NFR-S-1, 不存原始值)."""
        env_str = json.dumps(dict(os.environ), sort_keys=True, default=str)
        return "sha256:" + hashlib.sha256(env_str.encode("utf-8", errors="replace")).hexdigest()[:16]


# Late import for type hints (避免循环)
from .event_emitter import Event, HookResult  # noqa: E402
from .hook_registry import Hook  # noqa: E402