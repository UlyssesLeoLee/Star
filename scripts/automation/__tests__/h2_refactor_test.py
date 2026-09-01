#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/__tests__/h2_refactor_test.py — H2-1 refactor_template 子类化测试
(per docs/automation-design.md v0.1 §4.6 + §6.4 H2-1 5 测试)

5 测试:
- test_01_parse_5_actions: parse_report 返 5 个 Action (per HANDOFF-ST-001 commit 68ae5ff)
- test_02_action_1_is_agent_session: action 1 含 is_agent_session 字段
- test_03_action_2_roles_module: action 2 含 roles 模块 6 常量
- test_04_apply_dry_run: dry_run=True 不修改文件
- test_05_inherits_from_template: H2Stage1Refactor 继承 RefactorTemplate

约束 (per 守门 #1 v1):
- 标准库 only (unittest / tempfile / pathlib)
- sys.path 加 scripts/ 让 import automation 找到
"""

import sys
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.h2_refactor import H2Stage1Refactor, H2_STAGE1_PLACEHOLDER_ACTIONS
from automation.refactor_template import RefactorTemplate, Action


class H2RefactorTest(unittest.TestCase):
    """H2-1 stage 1 refactor 子类化 (per HANDOFF-ST-001 commit 68ae5ff)"""

    def test_01_parse_5_actions(self):
        """parse_report 返 5 个 Action (per HANDOFF-ST-001 stage 1)"""
        r = H2Stage1Refactor(
            report_path=Path("docs/reports/H2-stage1.md"),  # 不存在, 走占位
            phase="P3-H2",
            dry_run=True,
        )
        actions = r.parse_report()
        self.assertEqual(len(actions), 5, f"应该 5 个 action, 实际 {len(actions)}")

    def test_02_action_1_is_agent_session(self):
        """action 1: star_context_add_is_agent_session_field"""
        actions = H2_STAGE1_PLACEHOLDER_ACTIONS
        self.assertEqual(actions[0].action_id, "star_context_add_is_agent_session_field")
        self.assertEqual(actions[0].file_pattern, "crates/star-context/src/actor.rs")
        self.assertEqual(actions[0].operation, "add")
        self.assertIn("is_agent_session", actions[0].replacement)
        self.assertEqual(actions[0].metadata.get("commit"), "68ae5ff")

    def test_03_action_2_roles_module(self):
        """action 2: star_context_add_roles_module 含 6 常量"""
        actions = H2_STAGE1_PLACEHOLDER_ACTIONS
        self.assertEqual(actions[1].action_id, "star_context_add_roles_module")
        replacement = actions[1].replacement
        for constant in ["TENANT_ADMIN", "PROJECT_ADMIN", "DEVELOPER", "VIEWER", "AGENT", "SERVICE_INTERNAL"]:
            self.assertIn(constant, replacement, f"roles module 应含 {constant}")

    def test_04_apply_dry_run(self):
        """apply dry_run=True 不修改文件"""
        r = H2Stage1Refactor(
            report_path=Path("docs/reports/H2-stage1.md"),
            phase="P3-H2",
            dry_run=True,
        )
        actions = r.parse_report()
        result = r.apply(actions[0])
        # dry_run 模式下 action 0 找不到文件 (wt base 094284b 没 actor.rs)
        # 但 result.success=True 因为 apply 本身没 error
        self.assertTrue(result.success, f"apply should succeed: {result.error}")
        # files_matched = 0 (actor.rs 不在 wt base)
        self.assertEqual(result.files_matched, 0)

    def test_05_inherits_from_template(self):
        """H2Stage1Refactor 继承 RefactorTemplate"""
        r = H2Stage1Refactor(
            report_path=Path("docs/reports/H2-stage1.md"),
            phase="P3-H2",
            dry_run=True,
        )
        self.assertIsInstance(r, RefactorTemplate)
        # 5 Action 必是 Action 实例
        for action in r.parse_report():
            self.assertIsInstance(action, Action)


if __name__ == "__main__":
    unittest.main(verbosity=2)
