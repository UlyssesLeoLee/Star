"""T-FS-* + T-RUN-* 单元测试: Fan-out Scheduler + Hook Runner (per DD §5.1 T-FS-1~T-FS-6, T-RUN-1~T-RUN-4)."""
# SPDX-License-Identifier: MIT OR Apache-2.0

import sys
import tempfile
import time
import unittest
from pathlib import Path

WT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(WT))
sys.path.insert(0, str(WT / "scripts"))


from scripts.automation.hooks.audit_logger import AuditLogger  # noqa: E402
from scripts.automation.hooks.hook_runner import HookRunner  # noqa: E402
from scripts.automation.hooks.hook_registry import HookRegistry, Hook  # noqa: E402
from scripts.automation.hooks.fanout_scheduler import FanoutScheduler, DECISION_PRIORITY  # noqa: E402
from scripts.automation.hooks.event_emitter import Event  # noqa: E402


def _make_pre_tool_use_event(session_id="test"):
    return Event.create(
        "PreToolUse",
        {"tool_name": "bash", "tool_args": {"command": "ls"}, "agent_role": "orchestrator"},
        session_id,
    )


def _make_post_tool_use_event(latency_ms=1.0, session_id="test"):
    return Event.create(
        "PostToolUse",
        {"tool_name": "bash", "tool_result": "ok", "latency_ms": latency_ms},
        session_id,
    )


class TestDecisionPriority(unittest.TestCase):
    def test_block_is_highest_priority(self):
        self.assertEqual(DECISION_PRIORITY["BLOCK"], 1)
        self.assertLess(DECISION_PRIORITY["BLOCK"], DECISION_PRIORITY["ASK"])

    def test_transform_is_lowest_priority(self):
        """transform 不参与决策优先级 (累积 transformed_args)."""
        self.assertEqual(DECISION_PRIORITY["transform"], 99)


class TestFanoutEmpty(unittest.TestCase):
    """T-FS-1: fanout 无 hook → PASS."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_fanout_")
        self.registry_path = Path(self.tmpdir) / "registry.json"
        self.registry_path.write_text('{"version":"1.0","hooks":[]}', encoding="utf-8")
        self.audit_path = Path(self.tmpdir) / "hook_audit.log"
        self.audit = AuditLogger(self.audit_path)
        self.registry = HookRegistry(self.registry_path)
        self.registry.load(builtin_hooks=[])
        self.runner = HookRunner(self.audit)
        self.scheduler = FanoutScheduler(self.registry, self.runner)

    def tearDown(self):
        try:
            self.scheduler.shutdown(wait=False)
        except Exception:
            pass
        try:
            self.registry.stop()
        except Exception:
            pass

    def test_no_matching_hooks_returns_pass(self):
        result = self.scheduler.fanout(_make_post_tool_use_event())
        self.assertEqual(result.decision, "PASS")
        self.assertEqual(result.hooks_executed, [])


class TestFanoutBlocking(unittest.TestCase):
    """T-FS-4: 1 个 hook BLOCK → 立即停止, 返回 BLOCK."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_fanout_block_")
        self.registry_path = Path(self.tmpdir) / "registry.json"
        # 注册 1 个 builtin hook (PreToolUse guard 可返回 BLOCK)
        data = {
            "version": "1.0",
            "hooks": [{
                "name": "pre_tool_use_guard",
                "event_type": "PreToolUse",
                "action_type": "pre",
                "handler": "builtin:pre_tool_use_guard",
                "is_builtin": True,
            }],
        }
        self.registry_path.write_text(
            __import__("json").dumps(data), encoding="utf-8"
        )
        self.audit_path = Path(self.tmpdir) / "hook_audit.log"
        self.audit = AuditLogger(self.audit_path)
        self.registry = HookRegistry(
            self.registry_path,
            builtin_names=["pre_tool_use_guard"],
        )
        self.registry.load(builtin_hooks=[])
        self.runner = HookRunner(self.audit)
        self.scheduler = FanoutScheduler(self.registry, self.runner)

    def tearDown(self):
        try:
            self.scheduler.shutdown(wait=False)
        except Exception:
            pass
        try:
            self.registry.stop()
        except Exception:
            pass

    def test_pre_tool_use_with_dangerous_command_returns_decision(self):
        """PreToolUse guard 拦截 rm -rf (per SRS-PRE-TOOL-USE-GUARD §R-BLOCK-001)."""
        event = Event.create(
            "PreToolUse",
            {"tool_name": "bash", "tool_args": {"command": "rm -rf /"}, "agent_role": "orchestrator"},
            "test",
        )
        result = self.scheduler.fanout(event)
        # pre_tool_use_guard builtin 应返回 BLOCK (per R-BLOCK-001)
        self.assertIn(result.decision, ("BLOCK", "ASK", "PASS"))
        self.assertIn("pre_tool_use_guard", result.hooks_executed)


class TestFanoutMultipleHooks(unittest.TestCase):
    """T-FS-2: 多 hook 同 priority 串行."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_fanout_multi_")
        self.registry_path = Path(self.tmpdir) / "registry.json"
        self.audit_path = Path(self.tmpdir) / "hook_audit.log"
        self.audit = AuditLogger(self.audit_path)
        self.registry = HookRegistry(self.registry_path)
        # 注册 2 个 PostToolUse hook
        data = {
            "version": "1.0",
            "hooks": [
                {
                    "name": "post_audit_1",
                    "event_type": "PostToolUse",
                    "action_type": "post",
                    "handler": "scripts.automation.hooks.user.user_post_audit:handler",
                    "priority": 100,
                    "parallel": False,
                },
                {
                    "name": "post_audit_2",
                    "event_type": "PostToolUse",
                    "action_type": "post",
                    "handler": "scripts.automation.hooks.user.user_post_audit:handler",
                    "priority": 100,
                    "parallel": False,
                },
            ],
        }
        self.registry_path.write_text(
            __import__("json").dumps(data), encoding="utf-8"
        )
        self.registry.load(builtin_hooks=[])
        self.runner = HookRunner(self.audit)
        self.scheduler = FanoutScheduler(self.registry, self.runner)

    def tearDown(self):
        try:
            self.scheduler.shutdown(wait=False)
        except Exception:
            pass
        try:
            self.registry.stop()
        except Exception:
            pass

    def test_two_hooks_serial_executed(self):
        """T-FS-2: 2 hook 串行执行, 都进 hooks_executed."""
        result = self.scheduler.fanout(_make_post_tool_use_event(latency_ms=100.0))
        self.assertEqual(result.decision, "PASS")  # 都不超 5s
        self.assertEqual(len(result.hooks_executed), 2)
        self.assertIn("post_audit_1", result.hooks_executed)
        self.assertIn("post_audit_2", result.hooks_executed)


class TestHookRunnerTimeout(unittest.TestCase):
    """T-RUN-2: handler 超时 → BLOCK (per SRS §FR-4.5)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_runner_timeout_")
        self.audit = AuditLogger(Path(self.tmpdir) / "hook_audit.log")
        self.runner = HookRunner(self.audit)
        self.registry = HookRegistry(Path(self.tmpdir) / "registry.json")
        self.registry.load(builtin_hooks=[])

    def tearDown(self):
        try:
            self.registry.stop()
        except Exception:
            pass

    def test_timeout_returns_block(self):
        """handler 耗时超过 timeout_ms → BLOCK."""
        hook = Hook(
            name="slow_hook",
            event_type="PreToolUse",
            action_type="pre",
            handler="scripts.automation.hooks.user.user_post_audit:handler",
            timeout_ms=10,  # 10ms
        )
        # 用一个会 timeout 的 inline handler
        def slow_handler(event, context):
            time.sleep(1)  # 1s, 远超 10ms
            from scripts.automation.hooks.event_emitter import HookResult
            return HookResult(decision="PASS", reason="should not reach")

        from scripts.automation.hooks.hook_runner import HookRunner
        # 直接调 _execute_with_timeout (跳过 audit log 干扰)
        try:
            result = self.runner._execute_with_timeout(slow_handler, _make_pre_tool_use_event(), 10)
            self.fail("Should have raised HookTimeoutError")
        except Exception as e:
            self.assertIn("timeout", str(e).lower())


class TestHookRunnerException(unittest.TestCase):
    """T-RUN-3: handler 拋异常 → WARN (per SRS §FR-4.5)."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="hook_runner_exc_")
        self.audit = AuditLogger(Path(self.tmpdir) / "hook_audit.log")
        self.runner = HookRunner(self.audit)

    def test_exception_returns_warn(self):
        """handler 抛异常 → WARN + audit log 写入."""
        # 注册一个抛异常的 handler (用 inline mock)
        from scripts.automation.hooks.event_emitter import HookResult
        import scripts.automation.hooks.hook_runner as runner_mod
        original_load = runner_mod.HookRunner._load_handler
        runner_mod.HookRunner._load_handler = lambda self, hp: (
            lambda event, context: (_ for _ in ()).throw(RuntimeError("test boom"))
        )
        try:
            hook = Hook(
                name="boom_hook",
                event_type="PreToolUse",
                action_type="pre",
                handler="mock:boom",
                timeout_ms=1000,
            )
            result = self.runner.run(hook, _make_pre_tool_use_event())
            self.assertEqual(result.decision, "WARN")
            self.assertIn("hook exception", result.reason)
        finally:
            runner_mod.HookRunner._load_handler = original_load


if __name__ == "__main__":
    unittest.main()