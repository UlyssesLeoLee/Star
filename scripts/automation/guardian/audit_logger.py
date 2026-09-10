"""AuditLogger PM-4: JSON Lines 寫入, fail-closed.

Per DD-PRE-TOOL-USE-GUARD-001.md §2.3 + SRS-PRE-TOOL-USE-GUARD-001.md §4.4 + 守门 #13 Transaction.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import logging
import os
import stat
from dataclasses import dataclass, asdict, field
from pathlib import Path
from typing import Any, Dict, List, Optional

logger = logging.getLogger(__name__)


@dataclass
class AuditEvent:
    """audit log 单条事件 (per BD §4.3.2 schema)."""
    ts: str
    session_id: str
    agent_role: str
    tool: str
    tool_args_hash: str      # sha256:hex prefix, 必填 (脱敏)
    args_excerpt: str        # <= 200 chars, 必填 (脱敏)
    decision: str            # BLOCK|ASK|WARN|PASS
    rule_id: Optional[str] = None
    reason: Optional[str] = None
    latency_ms: float = 0.0
    env_hash: str = ""       # sha256 of all env keys


class AuditLogError(Exception):
    """Audit log 寫入失敗, 用於 fail-closed 判斷 (per FR-6.2)."""
    pass


class AuditLogger:
    """PM-4: 同步寫 audit log, append-only, fail-closed.

    Per SRS FR-4.1 + FR-6.2:
    - 必寫 (無論 decision)
    - 寫失敗 → 拋 AuditLogError → caller 判 BLOCK
    - JSON Lines format, append-only
    """

    def __init__(self, log_path: Path):
        self._path = Path(log_path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        # 確保文件存在 + 權限 (per NFR-S-2 audit log 完整性)
        if not self._path.exists():
            self._path.touch()
            try:
                os.chmod(self._path, stat.S_IRUSR | stat.S_IWUSR)  # 0644
            except OSError:
                pass

    def log(self, event: AuditEvent) -> None:
        """同步寫入 (per NFR-P-3 < 1ms), 失敗 raise AuditLogError."""
        try:
            d = asdict(event)
            line = json.dumps(d, ensure_ascii=False, separators=(",", ":")) + "\n"
        except (TypeError, ValueError) as e:
            # 序列化失敗, 必報 (per NFR-S-2)
            raise AuditLogError(f"event serialize failed: {e}") from e
        try:
            with self._path.open("a", encoding="utf-8", errors="replace") as f:
                f.write(line)
                f.flush()
                # 不 fsync (per NFR-P-3 < 1ms), OS lazy fsync
        except (OSError, IOError) as e:
            # per FR-6.2 + NFR-A-2: 寫失敗 → fail-closed
            raise AuditLogError(f"audit log write failed: {e}") from e

    def query(
        self,
        session_id: Optional[str] = None,
        decision: Optional[str] = None,
        rule_id: Optional[str] = None,
        limit: int = 100,
    ) -> List[AuditEvent]:
        """查詢 audit log, 供調試/報告 (per BD §5.1.3)."""
        results: List[AuditEvent] = []
        if not self._path.exists():
            return results
        try:
            with self._path.open("r", encoding="utf-8", errors="replace") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        d = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if session_id is not None and d.get("session_id") != session_id:
                        continue
                    if decision is not None and d.get("decision") != decision:
                        continue
                    if rule_id is not None and d.get("rule_id") != rule_id:
                        continue
                    try:
                        results.append(AuditEvent(**{
                            k: d.get(k) for k in (
                                "ts", "session_id", "agent_role", "tool",
                                "tool_args_hash", "args_excerpt", "decision",
                                "rule_id", "reason", "latency_ms", "env_hash"
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
        """統計 decision 計數, 供 session state 更新 (per NFR-O-1)."""
        counts = {"BLOCK": 0, "ASK": 0, "WARN": 0, "PASS": 0}
        for ev in self.query(limit=10_000_000):
            d = ev.decision
            if d in counts:
                counts[d] += 1
        return counts
