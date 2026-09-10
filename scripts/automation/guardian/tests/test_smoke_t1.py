"""T1 smoke test: 验证 5 module import + 15 规则加载 + 1 BLOCK 命中 + 1 PASS.

Per WBS-002 §T1 + DD §7.2.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
import time
from pathlib import Path

# 允许 guardian 作为 package import (scripts/automation 在 sys.path)
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.pre_tool_use_guard import (
    PreToolUseGuard, ToolCall, Decision, PatternMatcher, DecisionRouter
)
from guardian.rule_database import RuleDatabase, _validate_schema
from guardian.audit_logger import AuditLogger, AuditEvent
from guardian.user_rule_api import list_user_rules

GUARDIAN_DIR = Path(__file__).resolve().parents[1]
RULES_PATH = GUARDIAN_DIR / "rules" / "pre_tool_use_rules.json"
AUDIT_PATH = GUARDIAN_DIR / "logs" / "smoke_test_audit.log"


def t1_module_imports():
    """T1.6: 5 module 全部可 import."""
    print("[T1.6] Module imports...")
    assert PreToolUseGuard is not None
    assert ToolCall is not None
    assert Decision is not None
    assert PatternMatcher is not None
    assert DecisionRouter is not None
    assert RuleDatabase is not None
    assert _validate_schema is not None
    assert AuditLogger is not None
    assert AuditEvent is not None
    assert list_user_rules is not None
    print("    OK: 5 module + entities all importable")


def t2_rule_load():
    """T1.2: 15 规则加载, schema 校验通过."""
    print("[T1.2] Rule load + schema validation...")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    rules = db.get_rules()
    assert len(rules) == 15, f"期望 15 条, 实际 {len(rules)}"
    # 验证 BLOCK/ASK/WARN 比例
    by_level = {"BLOCK": 0, "ASK": 0, "WARN": 0}
    for r in rules:
        by_level[r.level] += 1
    assert by_level == {"BLOCK": 8, "ASK": 5, "WARN": 2}, f"比例不对: {by_level}"
    print(f"    OK: 15 rules loaded, BLOCK={by_level['BLOCK']} ASK={by_level['ASK']} WARN={by_level['WARN']}")
    db.stop()
    return rules


def t3_block_命中_rm_rf():
    """T1.5: rm -rf / 必须 BLOCK (R-BLOCK-001)."""
    print("[T1.5] BLOCK case: rm -rf /")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(db, audit)

    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "rm -rf /tmp/xxx"},
        session_id="smoke_test",
        agent_role="orchestrator",
    )
    # 注意: 这条命令实际是 rm -rf /tmp/xxx, 不是 rm -rf /, 应该 PASS
    decision = guard.evaluate(tc)
    # 让我换一条更明确的
    tc2 = ToolCall(
        tool_name="bash",
        tool_args={"command": "rm -rf /"},
        session_id="smoke_test",
        agent_role="orchestrator",
    )
    decision2 = guard.evaluate(tc2)
    # rm -rf / 应该命中 R-BLOCK-001
    assert decision2 == Decision.BLOCK, f"rm -rf / 期望 BLOCK, 实际 {decision2}"
    print(f"    OK: rm -rf / -> {decision2.value}")

    # 验证 audit log 写了
    events = audit.query(decision="BLOCK", session_id="smoke_test")
    assert len(events) >= 1, "audit log 应至少有 1 条 BLOCK"
    print(f"    OK: audit log 写入 {len(events)} 条 BLOCK 事件")

    db.stop()


def t4_pass_normal_command():
    """T1.5: ls -la 应该 PASS (无规则命中)."""
    print("[T1.5] PASS case: ls -la")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(db, audit)

    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "ls -la"},
        session_id="smoke_test",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tc)
    assert decision == Decision.PASS, f"ls -la 期望 PASS, 实际 {decision}"
    print(f"    OK: ls -la -> {decision.value}")
    db.stop()


def t5_ask_命中_sudo():
    """T1.5: sudo apt install 应该 ASK (R-ASK-002)."""
    print("[T1.5] ASK case: sudo apt install")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(db, audit)

    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "sudo apt install nginx"},
        session_id="smoke_test",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tc)
    assert decision == Decision.ASK, f"sudo 期望 ASK, 实际 {decision}"
    print(f"    OK: sudo apt install -> {decision.value}")
    db.stop()


def t6_block_命中_github_pat():
    """T1.5: 包含 GitHub PAT 的参数应该 BLOCK (R-BLOCK-007, per 守门 #5)."""
    print("[T1.5] BLOCK case: GitHub PAT in args (守门 #5)")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(db, audit)

    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "curl -H 'Authorization: token ghp_abcdefghijklmnopqrstuvwxyz1234567890' https://api.github.com"},
        session_id="smoke_test",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tc)
    assert decision == Decision.BLOCK, f"含 ghp_ token 期望 BLOCK, 实际 {decision}"
    print(f"    OK: GitHub PAT in args -> {decision.value} (守门 #5 联动)")
    db.stop()


def t7_warn_命中_全局_install():
    """T1.5: npm install -g 应该 WARN (R-WARN-002)."""
    print("[T1.5] WARN case: npm install -g")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(db, audit)

    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "npm install -g some-package"},
        session_id="smoke_test",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tc)
    assert decision == Decision.WARN, f"npm install -g 期望 WARN, 实际 {decision}"
    print(f"    OK: npm install -g -> {decision.value}")
    db.stop()


def t8_latency_benchmark():
    """T1.5: p50 < 5ms."""
    print("[T1.5] Latency benchmark: 100 次 evaluate")
    db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(AUDIT_PATH)
    guard = PreToolUseGuard(db, audit)

    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "echo hello"},
        session_id="bench",
        agent_role="orchestrator",
    )
    # 预热
    for _ in range(10):
        guard.evaluate(tc)
    # 测量
    latencies = []
    for _ in range(100):
        t0 = time.perf_counter()
        guard.evaluate(tc)
        latencies.append((time.perf_counter() - t0) * 1000)
    latencies.sort()
    p50 = latencies[50]
    p99 = latencies[99]
    print(f"    p50 = {p50:.3f}ms, p99 = {p99:.3f}ms")
    assert p50 < 5, f"p50 = {p50:.3f}ms, 期望 < 5ms (NFR-P-1 违反)"
    print(f"    OK: p50 < 5ms 满足 NFR-P-1")
    db.stop()


def t9_user_rule_api_stub():
    """T1.6: user_rule_api.stub 必须抛 NotImplementedError."""
    print("[T1.6] user_rule_api.py stub")
    from guardian.user_rule_api import register_user_rule
    try:
        register_user_rule("dummy: foo", Path("/tmp/foo"))
        assert False, "stub 应抛 NotImplementedError"
    except NotImplementedError as e:
        print(f"    OK: stub 抛 NotImplementedError: {str(e)[:60]}...")


if __name__ == "__main__":
    print("=" * 60)
    print("T1 smoke test: 5 module + 15 规则 + 5 case + 1 benchmark + 1 stub")
    print("=" * 60)
    t1_module_imports()
    t2_rule_load()
    t3_block_命中_rm_rf()
    t4_pass_normal_command()
    t5_ask_命中_sudo()
    t6_block_命中_github_pat()
    t7_warn_命中_全局_install()
    t8_latency_benchmark()
    t9_user_rule_api_stub()
    print("=" * 60)
    print("[OK] T1 smoke test ALL PASS")
    print("=" * 60)
