"""T2.3 单元测试 - AuditLogger (per WBS-002 T2.3 + DD §7.2).

5+ TC, 覆盖寫入/查詢/fail-closed.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import os
import stat
import sys
import time
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.audit_logger import AuditLogger, AuditEvent, AuditLogError


@pytest.fixture
def audit(tmp_path):
    return AuditLogger(tmp_path / "test_audit.log")


def _make_event(session_id="s1", decision="PASS", rule_id=None, reason=None, tool="bash"):
    return AuditEvent(
        ts="2026-09-10T19:00:00.000+09:00",
        session_id=session_id,
        agent_role="orchestrator",
        tool=tool,
        tool_args_hash="sha256:abc",
        args_excerpt="ls -la",
        decision=decision,
        rule_id=rule_id,
        reason=reason,
        latency_ms=1.0,
        env_hash="sha256:env",
    )


# ============ AC-4: 写入 + 读 ============

def test_log_single_event(audit):
    audit.log(_make_event())
    events = audit.query(limit=10)
    assert len(events) == 1
    assert events[0].session_id == "s1"


def test_log_100_events(audit):
    for i in range(100):
        audit.log(_make_event(session_id=f"s{i}"))
    assert len(audit.query(limit=200)) == 100


def test_query_by_session_id(audit):
    audit.log(_make_event(session_id="a"))
    audit.log(_make_event(session_id="b"))
    audit.log(_make_event(session_id="a"))
    assert len(audit.query(session_id="a")) == 2
    assert len(audit.query(session_id="b")) == 1


def test_query_by_decision(audit):
    audit.log(_make_event(decision="PASS"))
    audit.log(_make_event(decision="BLOCK", rule_id="R-BLOCK-001", reason="x"))
    audit.log(_make_event(decision="BLOCK", rule_id="R-BLOCK-002", reason="y"))
    assert len(audit.query(decision="BLOCK")) == 2
    assert len(audit.query(decision="PASS")) == 1


def test_query_by_rule_id(audit):
    audit.log(_make_event(decision="BLOCK", rule_id="R-BLOCK-001", reason="x"))
    audit.log(_make_event(decision="BLOCK", rule_id="R-BLOCK-002", reason="y"))
    assert len(audit.query(rule_id="R-BLOCK-001")) == 1


def test_query_limit(audit):
    for i in range(20):
        audit.log(_make_event(session_id=f"s{i}"))
    assert len(audit.query(limit=5)) == 5


# ============ AC-3: 写失败 → fail-closed ============

def test_log_readonly_raises(tmp_path):
    """AC-3: 写 read-only log 必 raise (per FR-6.2)."""
    log = tmp_path / "ro.log"
    log.write_text("")
    os.chmod(log, stat.S_IRUSR | stat.S_IRGRP | stat.S_IROTH)  # 0444
    audit = AuditLogger(log)
    with pytest.raises(AuditLogError):
        audit.log(_make_event())
    # 还原
    try:
        os.chmod(log, stat.S_IRUSR | stat.S_IWUSR)
    except OSError:
        pass


# ============ FR-4.2: append-only ============

def test_log_appends_not_overwrites(audit):
    """AC-4: append-only, 多次 log 累加不覆盖."""
    audit.log(_make_event(session_id="first"))
    audit.log(_make_event(session_id="second"))
    audit.log(_make_event(session_id="third"))
    events = audit.query(limit=10)
    assert [e.session_id for e in events] == ["first", "second", "third"]


# ============ 字段完整性 ============

def test_event_contains_required_fields(audit):
    audit.log(_make_event(decision="BLOCK", rule_id="R-BLOCK-001", reason="dangerous"))
    events = audit.query(decision="BLOCK")
    e = events[0]
    assert e.ts
    assert e.session_id == "s1"
    assert e.agent_role == "orchestrator"
    assert e.tool == "bash"
    assert e.tool_args_hash.startswith("sha256:")
    assert e.args_excerpt
    assert e.decision == "BLOCK"
    assert e.rule_id == "R-BLOCK-001"
    assert e.reason == "dangerous"


# ============ 性能 ============

def test_log_latency_under_1ms(audit):
    """NFR-P-3: audit 写 < 1ms (avg of 100)."""
    times = []
    for _ in range(100):
        t0 = time.perf_counter()
        audit.log(_make_event(session_id="perf"))
        times.append((time.perf_counter() - t0) * 1000)
    avg = sum(times) / len(times)
    assert avg < 1, f"avg 写耗时 {avg:.3f}ms 违反 NFR-P-3 (< 1ms)"


# ============ count_by_decision ============

def test_count_by_decision(audit):
    audit.log(_make_event(decision="PASS"))
    audit.log(_make_event(decision="PASS"))
    audit.log(_make_event(decision="BLOCK", rule_id="R-BLOCK-001", reason="x"))
    audit.log(_make_event(decision="WARN", rule_id="R-WARN-001", reason="y"))
    counts = audit.count_by_decision()
    assert counts == {"BLOCK": 1, "ASK": 0, "WARN": 1, "PASS": 2}
