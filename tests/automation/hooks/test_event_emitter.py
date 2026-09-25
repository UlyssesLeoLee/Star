"""T-EM-* 单元测试: Event Emitter 14 类事件 + payload 校验 (per DD §5.1 T-EM-1/T-EM-2/T-EM-3).

使用 stdlib unittest (per scripts/automation/__init__.py "标准库 only").
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import os
import sys
import tempfile
import unittest
from pathlib import Path

# 把 scripts 加到 sys.path, 允许 import scripts.automation.hooks
WT = Path(__file__).resolve().parents[3]  # worktree root
sys.path.insert(0, str(WT))
sys.path.insert(0, str(WT / "scripts"))


from scripts.automation.hooks.event_emitter import (  # noqa: E402
    EventEmitter,
    Event,
    EventType,
    VALIDATORS,
)
from scripts.automation.hooks.audit_logger import AuditLogger  # noqa: E402
from scripts.automation.hooks.hook_runner import HookRunner  # noqa: E402
from scripts.automation.hooks.hook_registry import HookRegistry  # noqa: E402
from scripts.automation.hooks.fanout_scheduler import FanoutScheduler  # noqa: E402
from scripts.automation.hooks.builtin_hook_loader import BuiltinHookLoader  # noqa: E402


class TestEventType(unittest.TestCase):
    """T-EM-2: 14 类事件枚举存在性."""

    def test_14_event_types_present(self):
        expected = {
            "PreToolUse", "PostToolUse", "UserPromptSubmit",
            "SessionStart", "SessionEnd", "SubagentDispatch",
            "SubagentReturn", "ToolError", "FileWatch",
            "CronTick", "RuntimeScan", "WorkspaceSwitch",
            "NetworkEgress", "CustomEvent",
        }
        actual = {et.value for et in EventType}
        self.assertEqual(actual, expected, f"missing: {expected - actual}")

    def test_event_type_count(self):
        self.assertEqual(len(list(EventType)), 14)


class TestPayloadValidators(unittest.TestCase):
    """T-EM-3: 14 类事件 payload 校验 (per DD §2.1 VALIDATORS)."""

    def test_pre_tool_use_valid(self):
        v = VALIDATORS[EventType.PRE_TOOL_USE]
        self.assertTrue(v({
            "tool_name": "bash",
            "tool_args": {"command": "ls"},
            "agent_role": "orchestrator",
        }))

    def test_pre_tool_use_missing_agent_role(self):
        v = VALIDATORS[EventType.PRE_TOOL_USE]
        self.assertFalse(v({
            "tool_name": "bash",
            "tool_args": {"command": "ls"},
        }))

    def test_post_tool_use_valid(self):
        v = VALIDATORS[EventType.POST_TOOL_USE]
        self.assertTrue(v({
            "tool_name": "bash",
            "tool_result": "ok",
            "latency_ms": 12.3,
        }))

    def test_session_start_valid(self):
        v = VALIDATORS[EventType.SESSION_START]
        self.assertTrue(v({
            "session_id": "uuid",
            "workspace": "/home/u/proj",
            "runtime": "mavis-0.4",
        }))

    def test_subagent_dispatch_valid(self):
        v = VALIDATORS[EventType.SUBAGENT_DISPATCH]
        self.assertTrue(v({
            "task_id": "t1",
            "script_path": "scripts/foo.py",
            "args": {},
            "parent_session_id": "uuid",
        }))

    def test_tool_error_valid(self):
        v = VALIDATORS[EventType.TOOL_ERROR]
        self.assertTrue(v({
            "tool_name": "bash",
            "error": "timeout",
            "stack_trace": "Traceback...",
        }))

    def test_file_watch_valid(self):
        v = VALIDATORS[EventType.FILE_WATCH]
        self.assertTrue(v({
            "file_path": "/tmp/foo",
            "event_type": "modified",
            "mtime": 1234567890.0,
        }))


class TestEventCreation(unittest.TestCase):
    """T-EM-1: Event.create 工廠方法."""

    def test_create_event_has_uuid_and_iso_timestamp(self):
        e = Event.create(
            "PreToolUse",
            {"tool_name": "bash", "tool_args": {}, "agent_role": "orchestrator"},
            "session-uuid",
        )
        self.assertEqual(len(e.event_id), 36)  # UUID v4
        self.assertTrue(e.timestamp.endswith("+00:00"))
        self.assertEqual(e.event_type, "PreToolUse")
        self.assertEqual(e.session_id, "session-uuid")
        self.assertEqual(e.payload["tool_name"], "bash")


class TestEventEmitterIntegration(unittest.TestCase):
    """T-EM-1 集成: EventEmitter.emit 走通 fan-out (无 hook 匹配时 PASS)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_test_")
        # registry.json 临时空文件 (含 1 个 user-defined hook)
        self.registry_path = Path(self.tmpdir) / "registry.json"
        self.registry_path.write_text('{"version":"1.0","hooks":[]}', encoding="utf-8")
        self.audit_path = Path(self.tmpdir) / "hook_audit.log"
        self.audit = AuditLogger(self.audit_path)
        self.builtin_loader = BuiltinHookLoader()
        self.builtin_hooks = self.builtin_loader.load_all()
        self.registry = HookRegistry(self.registry_path, [h.name for h in self.builtin_hooks])
        self.registry.load(self.builtin_hooks)
        self.runner = HookRunner(self.audit)
        self.scheduler = FanoutScheduler(self.registry, self.runner, max_workers=2)
        self.emitter = EventEmitter(self.scheduler)

    def tearDown(self):
        try:
            self.scheduler.shutdown(wait=False)
        except Exception:
            pass
        try:
            self.registry.stop()
        except Exception:
            pass

    def test_emit_no_matching_hook_returns_pass(self):
        """T-EM-1: 无 hook 匹配 → PASS."""
        result = self.emitter.emit(
            "PostToolUse",
            {"tool_name": "bash", "tool_result": "ok", "latency_ms": 1.0},
            "session-1",
        )
        self.assertEqual(result.decision, "PASS")
        self.assertEqual(result.hooks_executed, [])

    def test_emit_invalid_payload_raises_value_error(self):
        """T-EM-3: payload schema 校验失败 → ValueError."""
        with self.assertRaises(ValueError) as cm:
            self.emitter.emit(
                "PreToolUse",
                {"tool_name": "bash"},  # missing tool_args + agent_role
                "session-1",
            )
        self.assertIn("Invalid payload", str(cm.exception))

    def test_emit_unknown_event_type_raises_value_error(self):
        with self.assertRaises(ValueError):
            self.emitter.emit("NotARealEvent", {}, "session-1")


if __name__ == "__main__":
    unittest.main()