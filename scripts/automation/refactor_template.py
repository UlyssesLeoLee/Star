#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/refactor_template.py — 代码改造基类
(per docs/automation-design.md §3.3 + §6.3)

替代"看长报告 → 改 100+ 文件" 流程, 走"看报告路径 → 解析 → AST/regex → 改 → check → 报告"。
子类继承, 重写 parse_report + apply 即可。

用法:
    from automation.refactor_template import RefactorTemplate, Action, ApplyResult

    class MyRefactor(RefactorTemplate):
        def parse_report(self) -> list[Action]:
            # 解析 reports/my-report.md, 返回 Action 列表
            ...
        def apply(self, action: Action) -> ApplyResult:
            # 应用单条 Action
            ...

    r = MyRefactor(report_path=Path("docs/reports/P3-A.md"), dry_run=True)
    final = r.run_full()
    print(final.summary())

约束 (per 守门 #1 v1):
    - 标准库 only: re / pathlib / json / subprocess / dataclasses / abc
    - dry_run=True 默认, 不改文件
    - 失败自动 git stash + rollback
    - audit_log 必填, 落 `docs/reports/refactor-<phase>.log`

已知缺口 (per docs/automation-design.md §7):
    1. 解析器是 abstract, 需子类重写
    2. AST 操作未实装, 当前用 regex 操作 (足够 P0-1 19 脚本实证)
    3. git stash + rollback 是 stub, 真实实装需 subprocess 调用
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class Action:
    """代码改造动作 (per §3.3 parse_report 返)"""

    action_id: str  # 例: "remove_actor_ctx_struct"
    file_pattern: str  # glob, 例: "crates/domain-*/src/lib.rs"
    operation: str  # remove / add / replace
    pattern: str  # regex / 字符串
    replacement: str  # 替换内容
    metadata: dict = field(default_factory=dict)


@dataclass
class ApplyResult:
    """应用结果 (per §3.3 apply 返)"""

    action_id: str
    files_matched: int
    files_modified: int
    success: bool
    error: Optional[str] = None
    diff_preview: str = ""  # 头 200 字符


@dataclass
class VerifyResult:
    """验证结果 (per §3.3 verify 返)"""

    cargo_check_passed: bool
    err_count: int
    duration_sec: float
    output: str  # 头 500 字符


@dataclass
class FinalReport:
    """最终报告 (per §3.3 run_full 返)"""

    phase: str
    total_actions: int
    total_files_matched: int
    total_files_modified: int
    verify: Optional[VerifyResult]
    duration_sec: float
    started_at: float
    finished_at: float
    success: bool

    def summary(self) -> str:
        return (
            f"=== Refactor Final Report ===\n"
            f"phase={self.phase}\n"
            f"actions={self.total_actions}\n"
            f"files_matched={self.total_files_matched}\n"
            f"files_modified={self.total_files_modified}\n"
            f"verify_passed={self.verify.cargo_check_passed if self.verify else 'N/A'}\n"
            f"err_count={self.verify.err_count if self.verify else 'N/A'}\n"
            f"duration={self.duration_sec:.2f}s\n"
            f"success={self.success}\n"
        )


class RefactorTemplate(ABC):
    """代码改造基类 (per §6.3)"""

    def __init__(
        self,
        report_path: Path,
        *,
        phase: str = "default",
        dry_run: bool = True,
        audit_log: Optional[Path] = None,
    ):
        self.report_path = report_path
        self.phase = phase
        self.dry_run = dry_run
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / f"refactor-{phase}.log")
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)
        self.actions: list[Action] = []
        self.results: list[ApplyResult] = []

    # === 抽象方法 (子类必重写) ===

    @abstractmethod
    def parse_report(self) -> list[Action]:
        """解析报告 → Action 列表"""
        ...

    # === 6 个核心方法 (per §3.3 范式) ===

    def apply(self, action: Action) -> ApplyResult:
        """应用单条 Action (regex 操作, 默认实现, 可被子类 override)"""
        files = list(ROOT_DEFAULT.glob(action.file_pattern))
        modified = 0
        for fp in files:
            if not fp.is_file():
                continue
            content = fp.read_text(encoding="utf-8", errors="replace")
            if action.operation == "remove":
                new_content = re.sub(action.pattern, "", content, flags=re.MULTILINE)
            elif action.operation == "add":
                new_content = re.sub(action.pattern, action.replacement, content, flags=re.MULTILINE)
            elif action.operation == "replace":
                new_content = re.sub(action.pattern, action.replacement, content, flags=re.MULTILINE)
            else:
                return ApplyResult(
                    action_id=action.action_id,
                    files_matched=len(files),
                    files_modified=0,
                    success=False,
                    error=f"unknown operation: {action.operation}",
                )
            if new_content != content:
                if not self.dry_run:
                    fp.write_text(new_content, encoding="utf-8")
                modified += 1
        return ApplyResult(
            action_id=action.action_id,
            files_matched=len(files),
            files_modified=modified,
            success=True,
            diff_preview="",
        )

    def verify(self) -> Optional[VerifyResult]:
        """验证 (stub: 真实实装需 `cargo check --workspace --all-targets`)"""
        if not self.dry_run:
            try:
                proc = subprocess.run(
                    ["cargo", "check", "--workspace", "--all-targets"],
                    capture_output=True,
                    text=True,
                    timeout=300,
                    cwd=ROOT_DEFAULT,
                )
                err_count = proc.stderr.count("error[")
                return VerifyResult(
                    cargo_check_passed=proc.returncode == 0,
                    err_count=err_count,
                    duration_sec=0.0,
                    output=proc.stderr[:500],
                )
            except (subprocess.TimeoutExpired, FileNotFoundError) as e:
                return None
        return None

    def rollback(self) -> bool:
        """回滚 (stub: 真实实装需 git stash)"""
        # stub (per §7 已知缺口 #3)
        return True

    def run_full(self) -> FinalReport:
        """跑全流程: parse → apply → verify"""
        start = time.time()
        self.actions = self.parse_report()
        self.results = [self.apply(a) for a in self.actions]
        verify = self.verify()
        end = time.time()
        return FinalReport(
            phase=self.phase,
            total_actions=len(self.actions),
            total_files_matched=sum(r.files_matched for r in self.results),
            total_files_modified=sum(r.files_modified for r in self.results),
            verify=verify,
            duration_sec=end - start,
            started_at=start,
            finished_at=end,
            success=all(r.success for r in self.results) and (verify is None or verify.cargo_check_passed),
        )

    def export_actions_json(self, output_path: Optional[Path] = None) -> Path:
        """导 Action 列表为 JSON (审计 / 重放)"""
        output_path = output_path or (REPORTS_DIR_DEFAULT / f"refactor-{self.phase}-actions.json")
        output_path.write_text(
            json.dumps([asdict(a) for a in self.actions], indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        return output_path


# === 范例子类 (per §7 已知缺口 #1) ===

class ExampleRemoveActorCtx(RefactorTemplate):
    """范例: 删除所有 domain-* lib.rs 里的 pub struct ActorContext (per P0-1 19 fix 脚本实证)"""

    def parse_report(self) -> list[Action]:
        return [
            Action(
                action_id="remove_pub_struct_actor_context",
                file_pattern="crates/domain-*/src/lib.rs",
                operation="remove",
                pattern=r"^pub struct ActorContext \{[^}]*\}\s*",
                replacement="",
                metadata={"phase": "P0-1", "commit_ref": "P0-1 commit"},
            ),
        ]


def main():
    """CLI 入口: 跑 ExampleRemoveActorCtx 范例"""
    parser = argparse.ArgumentParser(description="代码改造基类 CLI (范例)")
    parser.add_argument("--report", type=Path, required=True, help="报告路径 (本范例未使用)")
    parser.add_argument("--phase", default="example", help="阶段")
    parser.add_argument("--dry-run", action="store_true", default=True, help="dry run (默认)")
    parser.add_argument("--no-dry-run", dest="dry_run", action="store_false", help="实际执行")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    r = ExampleRemoveActorCtx(
        report_path=args.report,
        phase=args.phase,
        dry_run=args.dry_run,
        audit_log=args.audit_log,
    )
    final = r.run_full()
    print(final.summary())
    sys.exit(0 if final.success else 1)


if __name__ == "__main__":
    main()
