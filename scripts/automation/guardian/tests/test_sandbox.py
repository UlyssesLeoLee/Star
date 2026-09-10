"""test_sandbox.py — 子代理 sandbox 沙箱 (per plan-031 Phase C sandbox P0)

覆盖:
  - is_available() 跨平台检测
  - SandboxLimits dataclass 默认值
  - make_default_sandbox_limits 默认值符合预期
  - run_with_sandbox limits=None → fail-open (subprocess.run 路径)
  - run_with_sandbox 正常进程 exit 0
  - run_with_sandbox 异常进程 exit 非 0
  - run_with_sandbox timeout 触发
  - run_with_sandbox 在 Windows + 不可用时 fail-open
  - 进程数限制下 父进程退出级联 kill (cascading kill) - 跨平台降级
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import os
import subprocess
import sys
import time
import unittest
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from guardian.sandbox import (  # noqa: E402
    SandboxLimits,
    _SandboxResult,
    is_available,
    make_default_sandbox_limits,
    run_with_sandbox,
)


class TestIsAvailable(unittest.TestCase):
    """TC 1: 跨平台 is_available 检测 (per 守门 v0.2 sandbox + 守门 #6 跨平台)."""

    def test_returns_bool(self):
        result = is_available()
        self.assertIsInstance(result, bool)
        if sys.platform == "win32":
            try:
                import win32job  # noqa: F401
                self.assertTrue(result, "Windows + win32job should be available")
            except ImportError:
                self.assertFalse(result, "win32job missing, should be unavailable")
        else:
            self.assertFalse(result, f"{sys.platform} not supported in v0.2")


class TestSandboxLimits(unittest.TestCase):
    """TC 2: SandboxLimits dataclass 默认值 (per 守门 v0.2 sandbox)."""

    def test_default_values(self):
        limits = SandboxLimits()
        self.assertEqual(0, limits.memory_mb)
        self.assertEqual(0, limits.cpu_percent)
        self.assertEqual(0, limits.max_processes)
        self.assertTrue(limits.kill_on_parent_exit)

    def test_custom_values(self):
        limits = SandboxLimits(memory_mb=512, cpu_percent=75, max_processes=50, kill_on_parent_exit=False)
        self.assertEqual(512, limits.memory_mb)
        self.assertEqual(75, limits.cpu_percent)
        self.assertEqual(50, limits.max_processes)
        self.assertFalse(limits.kill_on_parent_exit)


class TestDefaultLimits(unittest.TestCase):
    """TC 3: make_default_sandbox_limits 默认值 (per 守门 v0.2 sandbox §3.5)."""

    def test_default_limits(self):
        limits = make_default_sandbox_limits()
        self.assertEqual(1024, limits.memory_mb)
        self.assertEqual(50, limits.cpu_percent)
        self.assertEqual(100, limits.max_processes)
        self.assertTrue(limits.kill_on_parent_exit)


class TestRunWithSandboxFailOpen(unittest.TestCase):
    """TC 4: run_with_sandbox limits=None → fail-open (per 守门 #1 + 守门 #6)."""

    def test_no_limits_fallback_to_subprocess_run(self):
        # 跨平台: 不带 limits 走 subprocess.run
        with run_with_sandbox([sys.executable, "-c", "print('hello')"], limits=None) as result:
            self.assertEqual(0, result.returncode)
            self.assertIn("hello", result.stdout)


class TestRunWithSandboxAvailable(unittest.TestCase):
    """TC 5: run_with_sandbox 实际可用 (per 守门 v0.2 sandbox §3.2)."""

    def test_basic_success(self):
        limits = make_default_sandbox_limits() if is_available() else None
        with run_with_sandbox([sys.executable, "-c", "print('sandboxed')"], limits=limits, timeout=10) as result:
            self.assertEqual(0, result.returncode)
            self.assertIn("sandboxed", result.stdout)

    def test_basic_failure(self):
        limits = make_default_sandbox_limits() if is_available() else None
        with run_with_sandbox([sys.executable, "-c", "import sys; sys.exit(7)"], limits=limits, timeout=10) as result:
            self.assertEqual(7, result.returncode)

    def test_timeout(self):
        limits = make_default_sandbox_limits() if is_available() else None
        with self.assertRaises(subprocess.TimeoutExpired):
            with run_with_sandbox(
                [sys.executable, "-c", "import time; time.sleep(5)"],
                limits=limits,
                timeout=1,
            ):
                pass


class TestRunWithSandboxUnavailable(unittest.TestCase):
    """TC 6: Windows + win32job 不可用时 fail-open (per 守门 #1 fail-open)."""

    def test_sandbox_unavailable_fallback(self):
        # 模拟不可用: 传 limits 但 is_available() False 时仍应走 subprocess.run 路径
        # Windows + pywin32 不可用 → limits 被忽略
        limits = SandboxLimits(memory_mb=1, cpu_percent=1)
        if not is_available():
            with run_with_sandbox([sys.executable, "-c", "print('no_sandbox')"], limits=limits) as result:
                self.assertEqual(0, result.returncode)
                self.assertIn("no_sandbox", result.stdout)
        else:
            self.skipTest("sandbox available, skip unavailable test")


class TestSandboxCascadingKill(unittest.TestCase):
    """TC 7: cascading kill 验证 (per 守门 v0.2 §3.2 d kill_on_parent_exit).

    Windows: 用 win32job JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE 验证 job close 触发子进程 kill.
    非 Windows: 跳过 (v0.2 不实装).

    注: 在非 elevated 进程, AssignProcessToJobObject 可能因 Access Denied (err 5) 失败,
    走 fail-open (per 守门 #1 + 守门 #6 跨平台降级). 此测试验证 fail-open 行为
    (TimeoutExpired 仍 raise 出 with 块) 而非 sandbox 强制执行.
    """

    @unittest.skipUnless(is_available(), "sandbox not available")
    def test_cascading_kill_fail_open_on_access_denied(self):
        # 起一个长跑子进程, 用 sandbox 包; 在非 elevated 进程, sandbox 可能
        # 因 Access Denied (err 5) 走 fail-open 路径. 验证 TimeoutExpired 仍 raise.
        proc_script = (
            "import time, sys; "
            "sys.stdout.write('child_started\\n'); sys.stdout.flush(); "
            "time.sleep(60)"
        )
        limits = SandboxLimits(memory_mb=0, cpu_percent=0, max_processes=0, kill_on_parent_exit=True)
        with self.assertRaises(subprocess.TimeoutExpired):
            with run_with_sandbox(
                [sys.executable, "-c", proc_script],
                limits=limits,
                timeout=2,
            ):
                pass


if __name__ == "__main__":
    unittest.main()
