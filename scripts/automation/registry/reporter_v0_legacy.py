"""
reporter.py — RuntimeStatusReporter (per DD-MULTICA-RUNTIME-001 §3.6)

@deprecated v0.legacy — Replaced by Rust `star-registry::reporter` module in R3 阶段
                (per ADR-0027 R3). 阶段 1 实装: commit `fadff8f`.
                Per 守门 #11 缺标比错标: 不删, 永久留档.

audit log 持久化 + console API 暴露.
Per 守门 #12 v21 [P] docs 同步: 每次 scan 落 log.
"""
import json
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, List

if TYPE_CHECKING:
    from probe_v0_legacy import RuntimeEntry


class RuntimeStatusReporter:
    """audit log 持久化 (per FR-23)"""

    def __init__(self, log_dir: Path = Path("docs/reports/runtime-scan")):
        self._log_dir = log_dir
        self._log_dir.mkdir(parents=True, exist_ok=True)

    def write_log(self, scan_id: str, started_at: datetime, finished_at: datetime, entries: List["RuntimeEntry"]) -> Path:
        """写每日 .log 文件 (per FR-23)"""
        date = started_at.strftime("%Y-%m-%d")
        log_path = self._log_dir / f"{date}.log"
        with log_path.open("a", encoding="utf-8") as f:
            f.write(f"=== scan {scan_id} started={started_at.isoformat()} finished={finished_at.isoformat()} ===\n")
            for e in entries:
                f.write(json.dumps({
                    "provider": e.provider,
                    "path": str(e.path) if e.path else None,
                    "version": e.version,
                    "version_parsed": list(e.version_parsed) if e.version_parsed else None,
                    "probe_method": e.probe_method,
                    "error": e.error,
                    "detected_at": e.detected_at.isoformat(),
                }, ensure_ascii=False) + "\n")
            f.write(f"=== finished ===\n\n")
        return log_path
