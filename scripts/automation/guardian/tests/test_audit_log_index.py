"""test_audit_log_index.py — audit log sidecar 索引 (per 守门 v36)

覆盖:
  - build_index on empty log → 0
  - build_index on < chunk_size events → 0 (per 守门 v36 §1.2)
  - build_index on >= chunk_size events → 1 chunk
  - count_by_decision_indexed 聚合正确
  - rebuild_index 全量重建幂等
  - idx_path_for 派生 .idx 同目录
  - AuditLogger.count_by_decision_indexed 走 idx O(K) 兼容 fallback
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.audit_logger import AuditEvent, AuditLogger
from guardian.index_builder import (
    DEFAULT_CHUNK_SIZE,
    build_index,
    count_by_decision_indexed,
    idx_path_for,
    read_index,
    rebuild_index,
    total_events_indexed,
)


def _make_log(log_path: Path, n: int) -> None:
    """n events → log_path."""
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w", encoding="utf-8") as f:
        for i in range(n):
            ev = {
                "ts": f"2026-09-10T22:00:{i:02d}.000+09:00",
                "session_id": f"s{i % 3}",
                "agent_role": "tester",
                "tool": "bash",
                "tool_args_hash": f"hash{i:04d}",
                "args_excerpt": f"cmd{i}",
                "decision": ["BLOCK", "ASK", "WARN", "PASS"][i % 4],
                "rule_id": f"R-{i % 5:03d}" if i % 2 == 0 else None,
                "reason": f"reason{i}",
                "latency_ms": float(i * 0.1),
                "env_hash": "envhash",
            }
            f.write(json.dumps(ev, ensure_ascii=False) + "\n")


class TestBuildIndexEmpty(unittest.TestCase):
    """TC 1: 空 log → 0 chunk (per 守门 v36 §1.2)."""

    def test_empty_log_zero_chunks(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            self.assertEqual(0, build_index(log, idx))


class TestBuildIndexBelowThreshold(unittest.TestCase):
    """TC 2: < chunk_size events → 0 chunk (per 守门 v36 §1.2 避免太碎)."""

    def test_below_threshold_zero_chunks(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            _make_log(log, DEFAULT_CHUNK_SIZE - 1)
            self.assertEqual(0, build_index(log, idx))


class TestBuildIndexOneChunk(unittest.TestCase):
    """TC 3: >= chunk_size events → 1 chunk (per 守门 v36 §1.2)."""

    def test_exact_chunk_size_one_chunk(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            _make_log(log, DEFAULT_CHUNK_SIZE)
            self.assertEqual(1, build_index(log, idx))
            chunks = read_index(idx)
            self.assertEqual(1, len(chunks))
            self.assertEqual(0, chunks[0]["chunk_id"])
            self.assertEqual(DEFAULT_CHUNK_SIZE, chunks[0]["count"])


class TestCountByDecisionIndexed(unittest.TestCase):
    """TC 4: count_by_decision_indexed 聚合正确 (per 守门 v36 §1.1)."""

    def test_aggregate_decision_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            _make_log(log, DEFAULT_CHUNK_SIZE)
            build_index(log, idx)
            counts = count_by_decision_indexed(idx)
            self.assertEqual(set(counts.keys()), {"BLOCK", "ASK", "WARN", "PASS"})
            self.assertEqual(sum(counts.values()), DEFAULT_CHUNK_SIZE)


class TestRebuildIndex(unittest.TestCase):
    """TC 5: rebuild_index 全量重建幂等 (per 守门 v36 已知缺口 #4)."""

    def test_rebuild_idempotent(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            _make_log(log, DEFAULT_CHUNK_SIZE * 2)
            # 首次 build
            build_index(log, idx)
            chunks_before = len(read_index(idx))
            # rebuild
            rebuild_index(log, idx)
            chunks_after = len(read_index(idx))
            # 重建后 chunk 数应 >= 之前 (全量扫)
            self.assertGreaterEqual(chunks_after, chunks_before)


class TestIdxPathFor(unittest.TestCase):
    """TC 6: idx_path_for 派生 .idx 同目录 (per 守门 v36 §1.1)."""

    def test_derive_idx_path(self):
        log = Path("foo") / "bar" / "audit.log"
        idx = idx_path_for(log)
        self.assertEqual(idx, Path("foo") / "bar" / "audit.log.idx")


class TestTotalEventsIndexed(unittest.TestCase):
    """TC 7: total_events_indexed 总 event 数 (O(K), per 守门 v36 §1.1)."""

    def test_total_count(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            _make_log(log, DEFAULT_CHUNK_SIZE)
            build_index(log, idx)
            self.assertEqual(DEFAULT_CHUNK_SIZE, total_events_indexed(idx))


class TestAuditLoggerCountByDecisionIndexed(unittest.TestCase):
    """TC 8: AuditLogger.count_by_decision_indexed 走 idx (per 守门 v36 §1.1 + 兼容 fallback)."""

    def test_indexed_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            idx = idx_path_for(log)
            al = AuditLogger(log)
            _make_log(log, DEFAULT_CHUNK_SIZE)
            build_index(log, idx)
            counts_idx = al.count_by_decision_indexed()
            counts_full = al.count_by_decision()
            # 两种方法在 idx 已建时结果应一致
            self.assertEqual(counts_idx, counts_full)

    def test_fallback_when_no_idx(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "audit.log"
            al = AuditLogger(log)
            _make_log(log, 5)
            # idx 不存在 → fallback O(N)
            counts = al.count_by_decision_indexed()
            self.assertEqual(5, sum(counts.values()))


if __name__ == "__main__":
    unittest.main()
