#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/h2_refactor.py — H2-1 star_context 扩展 范式化封装
(per docs/automation-design.md v0.1 §3.3 + §4.6 + HANDOFF-ST-001 H2 stage 1 commit 68ae5ff)

继承 RefactorTemplate, 重写 parse_report 解析 HANDOFF-ST-001 H2 stage 1 report
-> Action 列表 (is_agent_session 字段 + roles 模块 + 4 helper method + 2 builder)

per WBS §14.2 H2-1 (阶段 1 完成 commit 68ae5ff, 净修 950 -> 432 err)
+ 守门 #1+#9+#12+#19+#20+#21 实证

用法:
    # Dry-run 解析 H2 stage 1 报告 -> Action 列表
    python scripts/automation/h2_refactor.py --dry-run

    # 真跑 (需 commit 68ae5ff report 文件)
    python scripts/automation/h2_refactor.py --report docs/reports/H2-stage1.md --phase P3-H2

约束 (per 守门 #1 v1 + 守门 #9):
    - 标准库 only (re / pathlib / json / dataclasses)
    - 不修改 star-context crate 源码 (H2-1 范式化封装)
    - parse_report 重写: 解析 H2 stage 1 report 第一段 markdown 表格行 -> Action
    - apply 复用 RefactorTemplate 默认实现 (regex remove/add/replace)
    - audit_log 必填, 落 docs/reports/h2-refactor.log

已知缺口 (per docs/automation-design.md §7):
    1. docs/reports/H2-stage1.md 占位文件未创建 (parse_report 用占位 fallback)
    2. 真实 AST 操作未实装 (per §6 已知缺口, 当前用 regex)
    3. 跟 19 P0-1 fix 脚本对比: h2_refactor.py 是模板化封装, 19 脚本是 ad-hoc 实现
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional

# sys.path 加 scripts/ 让 import automation 找到
# h2_refactor.py -> scripts/automation/ -> scripts/
SCRIPTS_DIR = Path(__file__).resolve().parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from automation.refactor_template import (
    RefactorTemplate, Action, ApplyResult, VerifyResult, FinalReport,
)

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


# H2-1 stage 1 commit 68ae5ff 实证 (per HANDOFF-ST-001 v0.2 §1 v17)
# parse_report 用占位 fallback: 当 H2-stage1.md 不存在时返这 5 个 Action
H2_STAGE1_PLACEHOLDER_ACTIONS = [
    Action(
        action_id="star_context_add_is_agent_session_field",
        file_pattern="crates/star-context/src/actor.rs",
        operation="add",
        pattern=r"(pub struct ActorContext \{)",
        replacement=r"\1\n    #[serde(default)]\n    pub is_agent_session: bool,\n",
        metadata={"phase": "P3-H2", "commit": "68ae5ff", "purpose": "INV-FB-07"},
    ),
    Action(
        action_id="star_context_add_roles_module",
        file_pattern="crates/star-context/src/actor.rs",
        operation="add",
        pattern=r"(use serde::Deserialize;\nuse serde::Serialize;\n)",
        replacement=r"\1\npub mod roles {\n    pub const TENANT_ADMIN: &str = \"tenant_admin\";\n    pub const PROJECT_ADMIN: &str = \"project_admin\";\n    pub const DEVELOPER: &str = \"developer\";\n    pub const VIEWER: &str = \"viewer\";\n    pub const AGENT: &str = \"agent\";\n    pub const SERVICE_INTERNAL: &str = \"service_internal\";\n}\n",
        metadata={"phase": "P3-H2", "commit": "68ae5ff", "purpose": "roles module 6 常量"},
    ),
    Action(
        action_id="star_context_add_is_tenant_admin_helper",
        file_pattern="crates/star-context/src/actor.rs",
        operation="add",
        pattern=r"(impl ActorContext \{)",
        replacement=r"\1\n    pub fn is_tenant_admin(&self) -> bool {\n        self.roles.contains(&roles::TENANT_ADMIN.to_string())\n    }\n",
        metadata={"phase": "P3-H2", "commit": "68ae5ff", "purpose": "4 helper method #1"},
    ),
    Action(
        action_id="star_context_add_with_project_builder",
        file_pattern="crates/star-context/src/actor.rs",
        operation="add",
        pattern=r"(impl ActorContext \{)",
        replacement=r"\1\n    pub fn with_project(mut self, project_id: Uuid) -> Self {\n        self.project_ids.push(project_id);\n        self\n    }\n",
        metadata={"phase": "P3-H2", "commit": "68ae5ff", "purpose": "2 builder #1"},
    ),
    Action(
        action_id="star_context_add_with_agent_session_builder",
        file_pattern="crates/star-context/src/actor.rs",
        operation="add",
        pattern=r"(impl ActorContext \{)",
        replacement=r"\1\n    pub fn with_agent_session(mut self, is_agent: bool) -> Self {\n        self.is_agent_session = is_agent;\n        self\n    }\n",
        metadata={"phase": "P3-H2", "commit": "68ae5ff", "purpose": "2 builder #2"},
    ),
]


class H2Stage1Refactor(RefactorTemplate):
    """H2-1 stage 1 refactor 子类 (per HANDOFF-ST-001 commit 68ae5ff)"""

    def parse_report(self) -> list[Action]:
        """解析 H2 stage 1 report -> Action 列表

        优先读 self.report_path (per brief), 不存在返占位 Action 列表
        """
        if not self.report_path.exists():
            # 占位 fallback (per §7 已知缺口 #1: docs/reports/H2-stage1.md 未创建)
            return H2_STAGE1_PLACEHOLDER_ACTIONS

        content = self.report_path.read_text(encoding="utf-8")
        actions = []
        # 解析 markdown 表格行: | action_id | file_pattern | operation | pattern | replacement |
        pattern = re.compile(
            r"\|\s*([\w-]+)\s*\|\s*(`?[\w.*?/]+`?)\s*\|\s*(\w+)\s*\|\s*(`[^`]*`|[^|]*?)\s*\|\s*(`[^`]*`|[^|]*?)\s*\|",
            re.MULTILINE,
        )
        for m in pattern.finditer(content):
            action_id, file_pattern, operation, patt, repl = m.groups()
            actions.append(Action(
                action_id=action_id,
                file_pattern=file_pattern.strip("`"),
                operation=operation,
                pattern=patt.strip("`"),
                replacement=repl.strip("`"),
                metadata={"phase": self.phase, "source": "H2-stage1.md"},
            ))
        return actions or H2_STAGE1_PLACEHOLDER_ACTIONS


def main():
    parser = argparse.ArgumentParser(description="H2-1 stage 1 refactor 子类化 (per docs/automation-design.md §4.6)")
    parser.add_argument("--report", type=Path, default=Path("docs/reports/H2-stage1.md"),
                        help="H2 stage 1 report 路径 (不存在返占位 Action)")
    parser.add_argument("--phase", default="P3-H2", help="阶段 (default: P3-H2)")
    parser.add_argument("--dry-run", action="store_true", default=True, help="dry run (默认)")
    parser.add_argument("--no-dry-run", dest="dry_run", action="store_false", help="真跑 (需 commit 68ae5ff 实证)")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    r = H2Stage1Refactor(
        report_path=args.report,
        phase=args.phase,
        dry_run=args.dry_run,
        audit_log=args.audit_log,
    )
    final = r.run_full()
    print(final.summary())
    print(f"actions: {len(final.total_actions and r.actions or [])}")
    for a in r.actions:
        print(f"  [{a.operation:7}] {a.action_id:50} ({a.file_pattern})")

    sys.exit(0 if final.success else 1)


if __name__ == "__main__":
    main()
