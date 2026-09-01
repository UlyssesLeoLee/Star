#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/cli_helper/base.py — CLI 调用基类
(per docs/automation-design.md §3.2 + §6.2)

替代主上下文长 shell 反复写, 跨平台差异 (PowerShell vs WSL) 抽象, 失败可重试。

用法:
    from automation.cli_helper.base import CliHelper
    h = CliHelper(audit_log=Path("docs/reports/P3-B.5.log"))
    result = h.cargo("check", ["--workspace", "--all-targets"], retries=2)
    print(f"err_count={result.stderr.count('error[')}")

约束 (per 守门 #1 v1):
    - 标准库 only: subprocess / pathlib / json / time / dataclasses
    - 跨平台: Windows / WSL / macOS / Linux
    - 失败可重试 (默认 1 次, 指数 backoff)
    - audit_log 必填

已知缺口 (per docs/automation-design.md §7):
    1. 跨平台 exec 抽象: 当前仅 Windows PowerShell, 跨 WSL/macOS/Linux 需补 subprocess 适配层
    2. cargo / git / wt 子命令未全部实现, 当前 stub
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional, Sequence

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class CliResult:
    """CLI 调用结果 (per §3.2 run 返)"""

    cmd: list
    exit_code: int
    stdout: str
    stderr: str
    duration_sec: float
    retries: int
    success: bool


@dataclass
class WorktreeContext:
    """worktree 上下文 (per §3.2 with_worktree 返)"""

    branch: str
    path: Path
    base: str
    created_at: float


@dataclass
class AuditEntry:
    """审计日志条目 (per §3.4)"""

    timestamp: float
    phase: str
    action: str  # run / cargo / git / wt
    input: dict
    output: dict
    error: Optional[str] = None


class CliHelper:
    """CLI 调用基类 (per §6.2)"""

    def __init__(
        self,
        phase: str = "default",
        audit_log: Optional[Path] = None,
        platform: Optional[str] = None,
    ):
        self.phase = phase
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / f"{phase}.log")
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)
        # 跨平台检测 (per §7 已知缺口 #1)
        self.platform = platform or self._detect_platform()

    # === 平台检测 (per §7 已知缺口 #1 抽象) ===

    def _detect_platform(self) -> str:
        import platform as _platform
        system = _platform.system().lower()
        if system == "windows":
            return "windows"
        elif system == "darwin":
            return "macos"
        elif system == "linux":
            return "linux"
        return "unknown"

    # === 核心方法 (per §3.2 范式) ===

    def run(
        self,
        cmd: Sequence[str],
        *,
        retries: int = 1,
        timeout: int = 60,
        cwd: Optional[Path] = None,
        shell: bool = False,
    ) -> CliResult:
        """通用 run, 失败可重试 (默认 1 次)"""
        last_result = None
        for attempt in range(retries + 1):
            start = time.time()
            try:
                proc = subprocess.run(
                    list(cmd) if not shell else " ".join(cmd),
                    capture_output=True,
                    text=True,
                    timeout=timeout,
                    cwd=cwd,
                    shell=shell,
                )
                duration = time.time() - start
                result = CliResult(
                    cmd=list(cmd),
                    exit_code=proc.returncode,
                    stdout=proc.stdout,
                    stderr=proc.stderr,
                    duration_sec=duration,
                    retries=attempt,
                    success=proc.returncode == 0,
                )
                self._audit(
                    action="run",
                    input={"cmd": list(cmd), "attempt": attempt, "timeout": timeout},
                    output=asdict(result),
                )
                if result.success:
                    return result
                last_result = result
                if attempt < retries:
                    # 指数 backoff: 2^attempt 秒
                    time.sleep(2 ** attempt)
            except subprocess.TimeoutExpired as e:
                self._audit(
                    action="run",
                    input={"cmd": list(cmd), "attempt": attempt, "timeout": timeout},
                    output={"success": False},
                    error=f"timeout after {timeout}s: {e}",
                )
                if attempt >= retries:
                    return CliResult(
                        cmd=list(cmd),
                        exit_code=-1,
                        stdout="",
                        stderr=f"timeout after {timeout}s",
                        duration_sec=time.time() - start,
                        retries=attempt,
                        success=False,
                    )
                time.sleep(2 ** attempt)
        return last_result or CliResult(
            cmd=list(cmd), exit_code=-1, stdout="", stderr="", duration_sec=0, retries=retries, success=False
        )

    def cargo(self, subcmd: str, args: Sequence[str] = (), **kwargs) -> CliResult:
        """cargo 子命令 (stub: 真实实装需补全)"""
        cmd = ["cargo", subcmd, *args]
        return self.run(cmd, **kwargs)

    def git(self, subcmd: str, args: Sequence[str] = (), **kwargs) -> CliResult:
        """git 子命令 (stub: 真实实装需补全)"""
        cmd = ["git", subcmd, *args]
        return self.run(cmd, **kwargs)

    def wt(self, subcmd: str, args: Sequence[str] = (), **kwargs) -> CliResult:
        """wt (git worktree) 子命令 (stub)"""
        cmd = ["git", "worktree", subcmd, *args]
        return self.run(cmd, **kwargs)

    def with_worktree(self, branch: str, base: str = "main") -> WorktreeContext:
        """worktree 上下文 (stub: 真实实装需 git worktree add)"""
        wt_path = ROOT_DEFAULT / ".worktrees" / branch.replace("/", "-")
        return WorktreeContext(
            branch=branch,
            path=wt_path,
            base=base,
            created_at=time.time(),
        )

    # === 内部 ===

    def _audit(
        self,
        action: str,
        input: dict,
        output: dict,
        error: Optional[str] = None,
    ):
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
            phase=self.phase,
            action=action,
            input=_normalize(input),
            output=_normalize(output),
            error=error,
        )
        with self.audit_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(entry), ensure_ascii=False) + "\n")


def main():
    """CLI 入口: 通用 run"""
    parser = argparse.ArgumentParser(description="CLI 调用基类 CLI")
    parser.add_argument("--cmd", nargs="+", required=True, help="命令, 例: cargo check --workspace")
    parser.add_argument("--retries", type=int, default=1, help="失败重试次数")
    parser.add_argument("--timeout", type=int, default=60, help="timeout (秒)")
    parser.add_argument("--phase", default="default", help="阶段, 例: P3-B.5")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    h = CliHelper(phase=args.phase, audit_log=args.audit_log)
    result = h.run(args.cmd, retries=args.retries, timeout=args.timeout)

    print(f"cmd={' '.join(result.cmd)}")
    print(f"exit_code={result.exit_code}")
    print(f"success={result.success}")
    print(f"duration={result.duration_sec:.2f}s")
    print(f"retries={result.retries}")
    sys.exit(0 if result.success else 1)


if __name__ == "__main__":
    main()
