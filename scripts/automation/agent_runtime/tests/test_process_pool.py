#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""test_process_pool — bounded subprocess 池 (per 02-BD §3.1 + 守门 #24)。"""

import sys
import unittest

from automation.agent_runtime.process_pool import ProcessPool, PoolResult


class TestProcessPool(unittest.TestCase):
    def test_submit_success(self):
        pool = ProcessPool(size=2, timeout=60.0)
        try:
            result = pool.submit([sys.executable, "-c", "print('hello')"])
            self.assertEqual(result.status, "succeeded")
            self.assertEqual(result.exit_code, 0)
            self.assertIn("hello", result.stdout_tail)
        finally:
            pool.shutdown()

    def test_submit_failure(self):
        pool = ProcessPool(size=2, timeout=60.0)
        try:
            result = pool.submit([sys.executable, "-c", "import sys; sys.exit(3)"])
            self.assertEqual(result.status, "failed")
            self.assertEqual(result.exit_code, 3)
        finally:
            pool.shutdown()

    def test_submit_timeout(self):
        pool = ProcessPool(size=1, timeout=1.0)
        try:
            result = pool.submit([sys.executable, "-c", "import time; time.sleep(10)"])
            self.assertEqual(result.status, "timeout")
        finally:
            pool.shutdown()

    def test_submit_many_bounded(self):
        pool = ProcessPool(size=2, timeout=60.0)
        try:
            cmds = [[sys.executable, "-c", "print('x')"] for _ in range(4)]
            results = pool.submit_many(cmds)
            self.assertEqual(len(results), 4)
            self.assertTrue(all(r.status == "succeeded" for r in results))
        finally:
            pool.shutdown()

    def test_empty_cmd_rejected(self):
        pool = ProcessPool(size=1)
        try:
            with self.assertRaises(ValueError):
                pool.submit([])
        finally:
            pool.shutdown()

    def test_zero_size_rejected(self):
        with self.assertRaises(ValueError):
            ProcessPool(size=0)

    def test_dispatch_subagent_deferred_fallback(self):
        """dispatcher 软依赖路径: 不可用 → deferred (per dispatcher.py 已知缺口 #2)。

        briefs/audit 落临时目录, 不污染 repo (per 文件域约束)。
        """
        import tempfile
        from pathlib import Path

        pool = ProcessPool(size=1)
        try:
            with tempfile.TemporaryDirectory(prefix="agent-runtime-test-") as tmp:
                tmp_path = Path(tmp)
                out = pool.dispatch_subagent(
                    task_id="t1",
                    content="brief",
                    agent="worker",
                    briefs_dir=tmp_path,   # 显式临时目录, 不写 docs/briefs/
                    audit_log=tmp_path / "audit.log",
                )
                self.assertIn(out["status"], ("deferred", "succeeded", "failed"))
                self.assertEqual(out["task_id"], "t1")
        finally:
            pool.shutdown()

    def test_pool_result_shape(self):
        r = PoolResult(cmd=["a"])
        self.assertEqual(r.status, "pending")
        self.assertIsNone(r.exit_code)


if __name__ == "__main__":
    unittest.main()
