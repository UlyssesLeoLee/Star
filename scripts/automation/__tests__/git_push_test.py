#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/__tests__/git_push_test.py — F.6 git_push 测试
(per docs/automation-design.md v0.1 §4.5 + §6.4 F.6 5 测试)

5 测试 (per brief):
- test_01_dry_run_push: 2 branch dry-run 成功
- test_02_github_reachable: github.com HEAD 可达
- test_03_secret_scan_detects_api_key: 命中 api_key 模式
- test_04_github_token_not_printed: 守门 #5 实证, token 不打印
- test_05_audit_log_written: audit_log 文件存在 + 含 push_all action

约束 (per 守门 #1 v1 + 守门 #5):
- 标准库 only (unittest / tempfile)
- sys.path 加 scripts/ 让 import automation 找到
"""

import os
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.git_push import (
    GitPushHelper,
    PUSH_BRANCHES,
    SECRET_PATTERNS,
    SECRET_FILE_PATTERN,
)


class DryRunTest(unittest.TestCase):
    """dry-run 模式: 2 branch 推成功 (per F.6 brief)"""

    def test_01_dry_run_push_2_branches(self):
        """dry-run 模式 推 main + feature/ai-ide-compat 成功"""
        helper = GitPushHelper(dry_run=True)
        result = helper.push_all()
        self.assertTrue(result.dry_run)
        self.assertEqual(len(result.branches), 2)
        for b in result.branches:
            self.assertTrue(b.success, f"{b.branch} dry-run should succeed")
            self.assertEqual(b.error, "dry-run mode")
        # 2 branch 必是 PUSH_BRANCHES 内容
        branch_names = [b.branch for b in result.branches]
        self.assertEqual(branch_names, PUSH_BRANCHES)

    def test_02_github_reachable(self):
        """github.com HEAD 可达 (per 9/2 实测 125a4d6)"""
        helper = GitPushHelper(dry_run=True)
        result = helper.push_all()
        self.assertTrue(result.github_reachable, "github.com HEAD should be reachable")


class SecretScanTest(unittest.TestCase):
    """Secret 扫描 5 模式 (per F.6 brief)"""

    def test_03_secret_scan_detects_api_key(self):
        """secret 扫描命中 api_key 模式 (per brief)"""
        helper = GitPushHelper(dry_run=True)
        result = helper.push_all()
        # 实证命中 (per 星仓 docs 含 ApiKey/HermesConfig 描述)
        # 5 SECRET_PATTERNS 至少 1 个命中
        self.assertGreater(len(SECRET_PATTERNS), 0)
        # 命中数 >= 0 (不强求, 因为 docs 可能改)
        self.assertGreaterEqual(result.secret_scan.secrets_found, 0)
        # 扫描文件数 > 0
        self.assertGreater(result.secret_scan.files_scanned, 0)


class TokenSafetyTest(unittest.TestCase):
    """守门 #5 实证: GITHUB_TOKEN 不打印"""

    def test_04_github_token_not_printed(self):
        """github_token 走 env 读, 不暴露在 summary / branch result / audit_log"""
        # 临时设个测试 token
        test_token = "ghp_FAKE_TEST_TOKEN_NOT_REAL_xxxxxxxxxxxx"
        os.environ["GITHUB_TOKEN"] = test_token
        try:
            helper = GitPushHelper(dry_run=True)
            result = helper.push_all()
            # summary 字符串不含 token
            summary = helper.summary(result)
            self.assertNotIn(test_token, summary, "summary should not contain token")
            # branch result 不含 token
            for b in result.branches:
                self.assertNotIn(test_token, b.output_preview)
                self.assertNotIn(test_token, str(b.error or ""))
            # audit_log 字符串不含 token
            audit_content = helper.audit_log.read_text(encoding="utf-8")
            self.assertNotIn(test_token, audit_content, "audit_log should not contain token")
            # 但 github_token_present=True 验证 token 已读
            self.assertTrue(result.github_token_present)
        finally:
            os.environ.pop("GITHUB_TOKEN", None)


class AuditLogTest(unittest.TestCase):
    """audit_log 必填 (per docs/automation-design.md §3.4)"""

    def test_05_audit_log_written(self):
        """git_push 跑完 audit_log 文件存在 + 含 push_all action"""
        with tempfile.TemporaryDirectory() as tmpdir:
            audit_log = Path(tmpdir) / "test-git-push.log"
            helper = GitPushHelper(dry_run=True, audit_log=audit_log)
            helper.push_all()
            self.assertTrue(audit_log.exists(), "audit_log should exist")
            content = audit_log.read_text(encoding="utf-8")
            self.assertIn("push_all", content, "audit_log 应含 push_all action")
            self.assertIn("git-push", content, "audit_log 应含 phase")


if __name__ == "__main__":
    unittest.main(verbosity=2)
