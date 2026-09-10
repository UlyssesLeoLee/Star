"""AuditLogger PM-4: JSON Lines 寫入, fail-closed.

Per DD-PRE-TOOL-USE-GUARD-001.md §2.3 + SRS-PRE-TOOL-USE-GUARD-001.md §4.4 + 守门 #13 Transaction.

v36 增强 (per docs/guardian/v36_audit_log_index.md §1.1):
  - `count_by_decision_indexed()` 走 sidecar idx O(K) 聚合
  - 主 log 仍 append-only (per 守门 #13 T), idx 是 W idempotent rebuild
  - 旧 `count_by_decision()` 保持 O(N) 扫描兼容 (per 缺标比错标)
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import logging
import os
import stat
from dataclasses import dataclass, asdict, field
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from guardian.index_builder import (
        count_by_decision_indexed as _count_by_decision_indexed,
        idx_path_for,
        total_events_indexed as _total_events_indexed,
    )
    _V36_INDEX_AVAILABLE = True
except ImportError:
    _V36_INDEX_AVAILABLE = False

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
        """統計 decision 計數, 供 session state 更新 (per NFR-O-1).

        旧 O(N) 扫描版本 (保留兼容, per 缺标比错标); 新版 v36 走索引 O(K) 见 count_by_decision_indexed.
        """
        counts = {"BLOCK": 0, "ASK": 0, "WARN": 0, "PASS": 0}
        for ev in self.query(limit=10_000_000):
            d = ev.decision
            if d in counts:
                counts[d] += 1
        return counts

    def count_by_decision_indexed(self) -> Dict[str, int]:
        """v36: 走 sidecar idx O(K) 聚合 (per 守门 v36 §1.1 + 性能 5000x+).

        idx 不存在 → 返旧 O(N) 扫描结果 (per 缺标比错标: 兼容未 build 状态).
        """
        if not _V36_INDEX_AVAILABLE:
            return self.count_by_decision()
        idx_p = idx_path_for(self._path)
        if not idx_p.exists():
            return self.count_by_decision()
        return _count_by_decision_indexed(idx_p)

    def total_events_indexed(self) -> int:
        """v36: 走 idx 总 event 数 (O(K), per 守门 v36 §1.1).

        idx 不存在 → fallback count_by_decision 总和.
        """
        if not _V36_INDEX_AVAILABLE:
            return sum(self.count_by_decision().values())
        idx_p = idx_path_for(self._path)
        if not idx_p.exists():
            return sum(self.count_by_decision().values())
        return _total_events_indexed(idx_p)
