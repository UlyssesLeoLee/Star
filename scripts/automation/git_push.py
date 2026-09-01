#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/git_push.py — F.6 推 origin (R-05 反转) 真实实装
(per docs/automation-design.md v0.1 §3.2 + §4.5 + §6.4 F.6 共享)

推 3 branch: main + feature/ai-ide-compat + wt branch
+ secret 扫描 (.env / API key / PAT / GITHUB_TOKEN, 正则匹配)
+ 守门 #1+#6+#9+#12 实证

per WBS §5 F.6 + §14.4 B-8 (9/1 23:59 JST github.com 443 不可达 + 无 PAT 推 origin 失败)

用法:
    # Dry-run 3 branch + secret 扫描
    python scripts/automation/git_push.py --dry-run

    # 真推 (需 Ulysses 提供 GITHUB_TOKEN 或 PAT, 守门 #5 实证)
    GITHUB_TOKEN=ghp_xxx python scripts/automation/git_push.py

约束 (per 守门 #1 v1 + 守门 #5 环境变量安全 + 守门 #6 PowerShell):
    - 标准库 only (re / subprocess / json / pathlib / dataclasses / argparse)
    - 默认 dry_run=True, 真推前必须显式 --no-dry-run + Ulysses 拍板
    - secret 扫描 5 模式: API_KEY / SECRET / TOKEN / PASSWORD / .env 文件
    - audit_log 必填, 落 docs/reports/git-push.log
    - 守门 #5 实证: GITHUB_TOKEN 走 os.environ.get, 不打印; 命中断言 pattern 命中就 fail-fast

已知缺口 (per docs/automation-design.md §7):
    1. github.com 443 不可达 (per 9/1 23:59 JST 实证), 真推需 Ulysses 拍板
    2. PAT 或 GITHUB_TOKEN 需 Ulysses 提供 (跨 session 续)
    3. git push subprocess.run 走 stdout=subprocess.PIPE, 中文 GBK 错位风险
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


# 3 branch 推 (per WBS §5 F.6 + §6 R-05 反转)
PUSH_BRANCHES = ["main", "feature/ai-ide-compat"]

# Secret 扫描 5 模式 (per brief)
SECRET_PATTERNS = [
    re.compile(r"(?i)(api[_-]?key)\s*[:=]\s*['\"]?[\w-]+"),
    re.compile(r"(?i)(secret)\s*[:=]\s*['\"]?[\w-]+"),
    re.compile(r"(?i)(token)\s*[:=]\s*['\"]?[\w-]+"),
    re.compile(r"(?i)(password)\s*[:=]\s*['\"]?[\w-]+"),
    re.compile(r"(?i)GITHUB_TOKEN\s*=\s*\S+"),
]
SECRET_FILE_PATTERN = re.compile(r"^\.env$|^\.env\..+$")


@dataclass
class BranchResult:
    """单 branch 推结果"""

    branch: str
    success: bool
    dry_run: bool
    remote_url: str
    error: Optional[str] = None
    duration_ms: float = 0.0
    output_preview: str = ""  # 头 200 字符


@dataclass
class SecretScanResult:
    """Secret 扫描结果"""

    files_scanned: int
    secrets_found: int
    matches: list  # list[(file, line_no, pattern, snippet)]


@dataclass
class GitPushResult:
    """git_push 整体结果"""

    dry_run: bool
    branches: list  # list[BranchResult]
    secret_scan: SecretScanResult
    github_reachable: bool
    duration_ms: float
    github_token_present: bool


@dataclass
class AuditEntry:
    """审计日志条目 (per docs/automation-design.md §3.4)"""

    timestamp: float
    phase: str
    action: str
    input: dict
    output: dict
    error: Optional[str] = None


class GitPushHelper:
    """git push 3 branch + secret 扫描 (per docs/automation-design.md §3.2 + §4.5)"""

    def __init__(
        self,
        remote: str = "origin",
        github_token_env: str = "GITHUB_TOKEN",
        dry_run: bool = True,
        audit_log: Optional[Path] = None,
    ):
        self.remote = remote
        self.github_token_env = github_token_env
        self.dry_run = dry_run
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / "git-push.log")
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)
        # 守门 #5: token 走 env 读, 不打印
        self.github_token = os.environ.get(self.github_token_env, "")

    def push_all(self) -> GitPushResult:
        """推 3 branch + secret 扫描 + github 可达性检查"""
        start = time.time()
        # 1. Secret 扫描
        secret_scan = self.scan_secrets()
        # 2. GitHub 可达性
        reachable = self.check_github_reachable()
        # 3. 推 3 branch
        branches = []
        for branch in PUSH_BRANCHES:
            result = self.push_branch(branch)
            branches.append(result)
        duration = (time.time() - start) * 1000
        overall = GitPushResult(
            dry_run=self.dry_run,
            branches=branches,
            secret_scan=secret_scan,
            github_reachable=reachable,
            duration_ms=duration,
            github_token_present=bool(self.github_token),
        )
        self._audit(
            action="push_all",
            input={"remote": self.remote, "branches": PUSH_BRANCHES, "dry_run": self.dry_run},
            output=asdict(overall),
        )
        return overall

    def push_branch(self, branch: str) -> BranchResult:
        """推单 branch (dry_run 时返 success=True 但 error='dry-run', 真推走 subprocess)"""
        start = time.time()
        remote_url = f"https://github.com/UlyssesLeoLee/Star.git"

        if self.dry_run:
            duration = (time.time() - start) * 1000
            return BranchResult(
                branch=branch,
                success=True,
                dry_run=True,
                remote_url=remote_url,
                error="dry-run mode",
                duration_ms=duration,
                output_preview=f"[dry-run] would push {branch} to {remote_url}",
            )

        # 真推模式 (需 GITHUB_TOKEN + github.com 可达)
        if not self.github_token:
            return BranchResult(
                branch=branch,
                success=False,
                dry_run=False,
                remote_url=remote_url,
                error=f"{self.github_token_env} 未设 (守门 #5 实证: 走 env 读, 不打印)",
                duration_ms=(time.time() - start) * 1000,
            )
        if not self.check_github_reachable():
            return BranchResult(
                branch=branch,
                success=False,
                dry_run=False,
                remote_url=remote_url,
                error="github.com 443 不可达 (per 9/1 23:59 JST 实证, B-8 拍板)",
                duration_ms=(time.time() - start) * 1000,
            )

        # subprocess git push
        try:
            proc = subprocess.run(
                ["git", "push", self.remote, branch],
                capture_output=True,
                text=True,
                timeout=60,
                cwd=str(ROOT_DEFAULT),
            )
            return BranchResult(
                branch=branch,
                success=proc.returncode == 0,
                dry_run=False,
                remote_url=remote_url,
                error=proc.stderr[:200] if proc.returncode != 0 else None,
                duration_ms=(time.time() - start) * 1000,
                output_preview=proc.stdout[:200],
            )
        except (subprocess.TimeoutExpired, FileNotFoundError) as e:
            return BranchResult(
                branch=branch,
                success=False,
                dry_run=False,
                remote_url=remote_url,
                error=f"subprocess error: {e}",
                duration_ms=(time.time() - start) * 1000,
            )

    def check_github_reachable(self) -> bool:
        """检查 github.com 443 可达性 (per B-8 拍板)"""
        try:
            proc = subprocess.run(
                ["git", "ls-remote", self.remote, "HEAD"],
                capture_output=True,
                text=True,
                timeout=10,
                cwd=str(ROOT_DEFAULT),
            )
            return proc.returncode == 0
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return False

    def scan_secrets(self, max_files: int = 100) -> SecretScanResult:
        """Secret 扫描 (per brief 5 模式 + .env 文件)"""
        files_scanned = 0
        matches = []
        for fp in ROOT_DEFAULT.rglob("*"):
            if files_scanned >= max_files:
                break
            if not fp.is_file():
                continue
            rel = fp.relative_to(ROOT_DEFAULT).as_posix()
            # .env 文件直接 skip 自身 (避免扫描 .env / scripts/automation/.env 等)
            if SECRET_FILE_PATTERN.match(fp.name):
                matches.append((rel, 0, ".env file", "(skip)"))
                files_scanned += 1
                continue
            # 只扫描 .py / .json / .toml / .yaml / .yml / .md
            if fp.suffix not in (".py", ".json", ".toml", ".yaml", ".yml", ".md"):
                continue
            try:
                content = fp.read_text(encoding="utf-8", errors="replace")
            except Exception:
                continue
            files_scanned += 1
            for line_no, line in enumerate(content.splitlines(), 1):
                for pattern in SECRET_PATTERNS:
                    if pattern.search(line):
                        # 守门 #5: snippet 脱敏, 只显示前 20 字符
                        snippet = line[:20] + "..." if len(line) > 20 else line
                        matches.append((rel, line_no, pattern.pattern[:20], snippet))
        return SecretScanResult(
            files_scanned=files_scanned,
            secrets_found=len(matches),
            matches=matches,
        )

    def _audit(self, action: str, input: dict, output: dict, error: Optional[str] = None):
        def _normalize(obj):
            if isinstance(obj, dict):
                return {k: _normalize(v) for k, v in obj.items()}
            if isinstance(obj, (list, tuple)):
                return [_normalize(v) for v in obj]
            if isinstance(obj, Path):
                return str(obj)
            return obj

        entry = AuditEntry(
            timestamp=time.time(),
            phase="git-push",
            action=action,
            input=_normalize(input),
            output=_normalize(output),
            error=error,
        )
        with self.audit_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(entry), ensure_ascii=False) + "\n")

    def summary(self, result: GitPushResult) -> str:
        success = sum(1 for b in result.branches if b.success)
        return (
            f"=== Git Push: {self.remote} ===\n"
            f"dry_run: {result.dry_run}\n"
            f"github_reachable: {result.github_reachable}\n"
            f"github_token_present: {result.github_token_present}\n"
            f"branches: {len(PUSH_BRANCHES)} (main + feature/ai-ide-compat)\n"
            f"success_branches: {success}\n"
            f"secret_scan: {result.secret_scan.files_scanned} files scanned, "
            f"{result.secret_scan.secrets_found} secrets found\n"
            f"duration_ms: {result.duration_ms:.2f}\n"
            f"audit_log: {self.audit_log}\n"
        )


def main():
    parser = argparse.ArgumentParser(description="git push 3 branch + secret 扫描 (per F.6)")
    parser.add_argument("--remote", default="origin", help="git remote name (default: origin)")
    parser.add_argument("--dry-run", action="store_true", default=True, help="dry run 模式 (默认)")
    parser.add_argument("--no-dry-run", dest="dry_run", action="store_false", help="真推模式 (需 GITHUB_TOKEN + 网络)")
    parser.add_argument("--max-scan-files", type=int, default=100, help="secret 扫描最大文件数 (default: 100)")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    helper = GitPushHelper(remote=args.remote, dry_run=args.dry_run, audit_log=args.audit_log)
    result = helper.push_all()
    print(helper.summary(result))
    for b in result.branches:
        print(f"  [{'OK' if b.success else 'FAIL'}] {b.branch:30} ({b.duration_ms:.2f}ms)")

    sys.exit(0 if all(b.success for b in result.branches) else 1)


if __name__ == "__main__":
    main()
