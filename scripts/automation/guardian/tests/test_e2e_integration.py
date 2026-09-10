"""T2.4 集成测试 - PreToolUseGuard 端到端 (per WBS-002 T2.4 + DD §7.3).

5+ TC, 验证 guard.evaluate_for_dispatch + 子代理 invoke 前置链路.
注: 实际接入 console_server.py / dispatcher.py 在 T3.2 (per WBS), 本 e2e
    测试在 module 层验证完整生命周期.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import subprocess
import sys
import time
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.pre_tool_use_guard import (
    PreToolUseGuard, ToolCall, Decision
)
from guardian.rule_database import RuleDatabase
from guardian.audit_logger import AuditLogger

GUARDIAN_DIR = Path(__file__).resolve().parents[1]
RULES_PATH = GUARDIAN_DIR / "rules" / "pre_tool_use_rules.json"


@pytest.fixture
def full_stack(tmp_path):
    """完整 stack: rules + audit + guard."""
    audit_path = tmp_path / "e2e_audit.log"
    rule_db = RuleDatabase(RULES_PATH, enable_watcher=False)
    audit = AuditLogger(audit_path)
    guard = PreToolUseGuard(rule_db, audit)
    yield guard, audit, rule_db
    rule_db.stop()


# ============ E2E-1: PASS case 完整生命周期 ============

def test_e2e_pass_normal_command(full_stack):
    """正常命令: evaluate → PASS → audit 写入 1 条 PASS."""
    guard, audit, _ = full_stack
    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "echo hello"},
        session_id="e2e_1",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tc)
    assert decision == Decision.PASS
    events = audit.query(session_id="e2e_1")
    assert len(events) == 1
    assert events[0].decision == "PASS"


# ============ E2E-2: BLOCK case 完整生命周期 ============

def test_e2e_block_dangerous_command(full_stack):
    """危险命令: evaluate → BLOCK → audit 写入 1 条 BLOCK + rule_id."""
    guard, audit, _ = full_stack
    tc = ToolCall(
        tool_name="bash",
        tool_args={"command": "rm -rf /"},
        session_id="e2e_2",
        agent_role="orchestrator",
    )
    decision = guard.evaluate(tc)
    assert decision == Decision.BLOCK
    events = audit.query(session_id="e2e_2", decision="BLOCK")
    assert len(events) == 1
    assert events[0].rule_id == "R-BLOCK-001"
    assert "rm -rf" in events[0].reason or "不可逆" in events[0].reason


# ============ E2E-3: 子代理 dispatch 前置 (FR-1.2) ============

def test_e2e_dispatch_safe_passes(full_stack):
    """子代理 dispatch 安全 task → PASS."""
    guard, _, _ = full_stack
    decision = guard.evaluate_for_dispatch(
        task_id="task_safe_001",
        script_path="scripts/automation/echo.py",
        args={"message": "hi"},
        parent_session_id="e2e_3",
    )
    assert decision == Decision.PASS


def test_e2e_dispatch_dangerous_blocked(full_stack):
    """子代理 dispatch 危险 task → BLOCK (per NFR-S-4 100% 覆盖)."""
    guard, _, _ = full_stack
    decision = guard.evaluate_for_dispatch(
        task_id="task_danger_001",
        script_path="scripts/automation/evil.py",
        args={"command": "rm -rf /"},
        parent_session_id="e2e_4",
    )
    assert decision == Decision.BLOCK


# ============ E2E-4: 多 tool type 混合 ============

def test_e2e_mixed_tool_types(full_stack):
    """bash + write + read 混合 tool_call, 各自正确决策."""
    guard, audit, _ = full_stack
    cases = [
        (ToolCall(tool_name="bash", tool_args={"command": "ls"}, session_id="e2e_5", agent_role="orchestrator"), Decision.PASS),
        (ToolCall(tool_name="bash", tool_args={"command": "sudo apt install"}, session_id="e2e_5", agent_role="orchestrator"), Decision.ASK),
        (ToolCall(tool_name="read", tool_args={"path": "/etc/shadow"}, session_id="e2e_5", agent_role="orchestrator"), Decision.PASS),  # read 不扫
        (ToolCall(tool_name="write", tool_args={"path": "/home/x/.ssh/x", "content": "x"}, session_id="e2e_5", agent_role="orchestrator"), Decision.ASK),
    ]
    for tc, expected in cases:
        d = guard.evaluate(tc)
        assert d == expected, f"tool={tc.tool_name} args={tc.tool_args} 期望 {expected}, 实际 {d}"
    assert len(audit.query(session_id="e2e_5")) == 4


# ============ E2E-5: 持久化 (audit 重启可查) ============

def test_e2e_audit_persists_across_restart(tmp_path):
    """audit log 跨进程重启仍可查 (per FR-4.1 持久化)."""
    audit_path = tmp_path / "persist_audit.log"
    rule_db = RuleDatabase(RULES_PATH, enable_watcher=False)
    # 第一次 evaluate
    audit1 = AuditLogger(audit_path)
    guard1 = PreToolUseGuard(rule_db, audit1)
    for i in range(5):
        guard1.evaluate(ToolCall(
            tool_name="bash",
            tool_args={"command": f"echo iter{i}"},
            session_id="persist",
            agent_role="orchestrator",
        ))
    # 模拟重启
    rule_db.stop()
    audit2 = AuditLogger(audit_path)
    events = audit2.query(session_id="persist", limit=100)
    assert len(events) == 5
    rule_db.stop()


# ============ E2E-6: 真实 subprocess 启动 self-test (验证 Python 进程可独立跑) ============

def test_e2e_subprocess_smoke():
    """用 subprocess 跑 smoke test, 验证模块可独立 import + 跑."""
    smoke = GUARDIAN_DIR / "tests" / "test_smoke_t1.py"
    result = subprocess.run(
        [sys.executable, str(smoke)],
        capture_output=True, text=True, timeout=30,
    )
    assert result.returncode == 0, f"smoke test 失败: {result.stderr[:500]}"
    assert "[OK] T1 smoke test ALL PASS" in result.stdout
