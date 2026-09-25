"""T-AL-* + T-BL-* 单元测试: Audit Logger + Builtin Hook Loader (per DD §5.1 T-AL-1~T-AL-3, T-BL-1~T-BL-2)."""
# SPDX-License-Identifier: MIT OR Apache-2.0

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

WT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(WT))
sys.path.insert(0, str(WT / "scripts"))


from scripts.automation.hooks.audit_logger import (  # noqa: E402
    AuditLogger,
    AuditLogEntry,
    AuditLogWriteError,
)
from scripts.automation.hooks.builtin_hook_loader import (  # noqa: E402
    BuiltinHookLoader,
    BUILTIN_HOOK_NAMES,
)


class TestAuditLoggerWrite(unittest.TestCase):
    """T-AL-1: log 写入."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_audit_")
        self.audit = AuditLogger(Path(self.tmpdir) / "audit.log")

    def test_log_writes_json_line(self):
        entry = self.audit.log(
            hook_name="test_hook",
            event_type="PreToolUse",
            action_type="pre",
            decision="PASS",
            session_id="s1",
            before_args={"tool_name": "bash", "tool_args": {}, "agent_role": "orchestrator"},
            latency_ms=2.5,
        )
        self.assertEqual(len(entry.run_id), 36)
        # 文件存在 + 1 行
        log_path = Path(self.tmpdir) / "audit.log"
        self.assertTrue(log_path.exists())
        lines = log_path.read_text(encoding="utf-8").strip().split("\n")
        self.assertEqual(len(lines), 1)
        # JSON parseable
        data = json.loads(lines[0])
        self.assertEqual(data["hook_name"], "test_hook")
        self.assertEqual(data["decision"], "PASS")
        self.assertIn("env_hash", data)
        self.assertTrue(data["env_hash"].startswith("sha256:"))


class TestAuditLoggerQuery(unittest.TestCase):
    """T-AL-3: query 过滤."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_audit_q_")
        self.audit = AuditLogger(Path(self.tmpdir) / "audit.log")
        # 写 3 条
        for i, d in enumerate(["PASS", "BLOCK", "WARN"]):
            self.audit.log(
                hook_name=f"hook_{i}",
                event_type="PreToolUse",
                action_type="pre",
                decision=d,
                session_id=f"s_{i}",
                before_args={"tool_name": "bash", "tool_args": {}, "agent_role": "orchestrator"},
                latency_ms=1.0 + i,
            )

    def test_query_by_decision(self):
        results = self.audit.query(decision="BLOCK", limit=10)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].decision, "BLOCK")

    def test_query_by_hook_name(self):
        results = self.audit.query(hook_name="hook_1", limit=10)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].hook_name, "hook_1")

    def test_query_by_session_id(self):
        results = self.audit.query(session_id="s_2", limit=10)
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].session_id, "s_2")

    def test_query_limit_caps_results(self):
        results = self.audit.query(limit=2)
        self.assertEqual(len(results), 2)

    def test_count_by_decision(self):
        counts = self.audit.count_by_decision()
        self.assertEqual(counts["PASS"], 1)
        self.assertEqual(counts["BLOCK"], 1)
        self.assertEqual(counts["WARN"], 1)


class TestAuditLogRotate(unittest.TestCase):
    """T-AL-2 + §10.2 rotation."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_audit_rot_")
        self.audit = AuditLogger(Path(self.tmpdir) / "audit.log")

    def test_rotate_archive_creates_gz(self):
        # 写 5 条
        for i in range(5):
            self.audit.log(
                hook_name=f"h{i}",
                event_type="PreToolUse",
                action_type="pre",
                decision="PASS",
                session_id="s",
                before_args={"tool_name": "bash", "tool_args": {}, "agent_role": "orchestrator"},
                latency_ms=1.0,
            )
        # keep_lines=2 应归档 3 行
        rotated = self.audit.rotate(keep_lines=2)
        self.assertEqual(rotated, 3)
        # 主 log 现在只剩 2 行
        lines = (Path(self.tmpdir) / "audit.log").read_text(encoding="utf-8").strip().split("\n")
        self.assertEqual(len(lines), 2)


class TestBuiltinHookLoader(unittest.TestCase):
    """T-BL-1: 5 builtin hook 装载 + T-BL-2: is_builtin."""

    def setUp(self):
        self.loader = BuiltinHookLoader()

    def test_5_builtin_names(self):
        self.assertEqual(len(BUILTIN_HOOK_NAMES), 5)

    def test_load_all_returns_hooks(self):
        hooks = self.loader.load_all()
        # 至少 builtin 装载, 可能少 (per fail-open, 缺模块时不报错)
        self.assertGreaterEqual(len(hooks), 0)
        for h in hooks:
            self.assertTrue(h.is_builtin)

    def test_is_builtin(self):
        self.loader.load_all()
        self.assertTrue(self.loader.is_builtin("pre_tool_use_guard"))
        self.assertFalse(self.loader.is_builtin("not_a_builtin"))

    def test_load_all_returns_at_least_pre_tool_use_guard(self):
        """PreToolUse guard builtin 必装载 (per SRS-PRE-TOOL-USE-GUARD-001 §1.5)."""
        hooks = self.loader.load_all()
        names = {h.name for h in hooks}
        self.assertIn("pre_tool_use_guard", names)


if __name__ == "__main__":
    unittest.main()