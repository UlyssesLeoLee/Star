"""sandbox.py — 子代理 subprocess 沙箱 (per plan-031 Phase C sandbox P0)

Per docs/architecture/SANDBOX-001.md §3 (设计) + AGENTS.md §4 守门 #9 派生:
  - Windows Job Objects 提供: CPU 速率限制 / 内存工作集限制 / 进程数限制 / 父进程退出级联 kill
  - 软依赖: pywin32 + win32job; 不可用 → fail-open (per 守门 #6 跨平台 + #1 fail-open 安全)
  - 集成 dispatcher.py.invoke() 的 subprocess.run → run_with_sandbox 替代 (per 守门 #1 v15 docs 同步饱和 1 commit 多文件)

跟现有守门关系:
  - #1 fail-open: 沙箱自身错误不阻断 dispatch (per FR-6.1 + §3.4)
  - #6 跨平台: Windows Job Objects + POSIX rlimit 兜底 (v0.2 暂只 Windows, 跨平台 v0.3)
  - #9 + #9 v3: 子代理 dispatch 必先 brief, sandbox 是 dispatch 的运行时隔离层
  - #14 v3: Mavis 临时代签 5 域 Lead 决策; sandbox 给代签决策加运行时安全护栏

已知缺口 (per plan-031 Phase C + 守门 #11 缺标比错标):
  - #1 Network isolation: Windows Filtering Platform 复杂, v0.2 不实装 (0 跨网访问)
  - #2 File system isolation: AppContainer / restricted token 复杂, v0.2 不实装 (走 chmod + W 工作目录)
  - #3 POSIX rlimit 兜底: v0.2 不实装, 跨平台 v0.3 (per 守门 #6 跨平台)
  - #4 mavis CLI 尚未落地: dispatcher.invoke() 当前走 status="deferred" fallback (per 守门 #9 #2), sandbox 是 forward-looking 基础设施
  - #5 沙箱自身的 metric/observability: v0.2 不实装, v0.3 加 psutil + audit log 联动
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import logging
import os
import subprocess
import sys
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterator, List, Optional

logger = logging.getLogger(__name__)


@dataclass
class SandboxLimits:
    """沙箱资源限制 (per 守门 v0.2 sandbox).

    Attributes:
        memory_mb: 进程工作集上限 (MB), 0 = 不限
        cpu_percent: CPU 速率限制 (1-100, 100 = 1 core), 0 = 不限
        max_processes: 进程数上限 (含子进程), 0 = 不限
        kill_on_parent_exit: 父进程退出时级联 kill 子进程树 (默认 True)
    """
    memory_mb: int = 0
    cpu_percent: int = 0
    max_processes: int = 0
    kill_on_parent_exit: bool = True


class _SandboxResult:
    """subprocess.run-compatible result (per 守门 v0.2 sandbox 兼容 dispatcher)."""

    def __init__(self, returncode: Optional[int], stdout: str, stderr: str):
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


def is_available() -> bool:
    """沙箱是否可用 (Windows + pywin32 + win32job)."""
    if sys.platform != "win32":
        return False
    try:
        import win32job  # noqa: F401
        return True
    except ImportError:
        return False


def _create_windows_job(limits: SandboxLimits):
    """建 1 个 Windows Job Object 并配置 limits (per 守门 v0.2 §3.2)."""
    import win32job

    job = win32job.CreateJobObject(None, "")
    info = win32job.QueryInformationJobObject(
        job, win32job.JobObjectExtendedLimitInformation
    )
    basic = info["BasicLimitInformation"]
    # 内存工作集限制 (per 守门 v0.2 §3.2 a)
    if limits.memory_mb > 0:
        info["ProcessMemoryLimit"] = limits.memory_mb * 1024 * 1024
        basic["LimitFlags"] |= win32job.JOB_OBJECT_LIMIT_PROCESS_MEMORY
    # CPU 速率限制 (per 守门 v0.2 §3.2 b)
    if limits.cpu_percent > 0:
        # CPU rate 通过 JobObjectCpuRateControlInformation (Win 8+) 设置
        # CpuRate 是百分比的 100 倍 (e.g. 50% = 5000)
        try:
            cpu_rate_info = win32job.QueryInformationJobObject(
                job, win32job.JobObjectCpuRateControlInformation
            )
            cpu_rate_info["CpuRate"] = limits.cpu_percent * 100
            win32job.SetInformationJobObject(
                job, win32job.JobObjectCpuRateControlInformation, cpu_rate_info
            )
        except Exception as e:  # noqa: BLE001
            # 旧 Windows (Win 7) 不支持, 走 fail-open warn log
            logger.warning("[sandbox] cpu rate control unavailable: %s", e)
    # 进程数限制 (per 守门 v0.2 §3.2 c)
    if limits.max_processes > 0:
        if hasattr(win32job, "JOB_OBJECT_LIMIT_ACTIVE_PROCESS"):
            basic["LimitFlags"] |= win32job.JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            basic["ActiveProcessLimit"] = limits.max_processes
    # 级联 kill (per 守门 v0.2 §3.2 d)
    if limits.kill_on_parent_exit:
        basic["LimitFlags"] |= win32job.JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
    win32job.SetInformationJobObject(
        job, win32job.JobObjectExtendedLimitInformation, info
    )
    return job


def _assign_process_to_job(job, pid: int) -> bool:
    """把 pid 加入 job (per 守门 v0.2 §3.3). 返 False 表 assign 失败 (罕见, 走 fail-open)."""
    try:
        import win32api
        import win32job
        # PROCESS_SET_QUOTA 在 pywin32 是 JOB_OBJECT_ASSIGN_PROCESS
        h_process = win32api.OpenProcess(
            win32job.JOB_OBJECT_ASSIGN_PROCESS | win32job.JOB_OBJECT_TERMINATE,
            False,
            pid,
        )
        win32job.AssignProcessToJobObject(job, h_process)
        return True
    except Exception as e:  # noqa: BLE001
        logger.warning("[sandbox] assign pid=%d to job failed: %s", pid, e)
        return False


@contextmanager
def run_with_sandbox(
    cmd: List[str],
    *,
    timeout: int = 600,
    cwd: Optional[Path] = None,
    limits: Optional[SandboxLimits] = None,
    capture_output: bool = True,
) -> Iterator[_SandboxResult]:
    """subprocess.run 的沙箱替代 (per 守门 v0.2 sandbox 集成 dispatcher).

    Yields:
        _SandboxResult (跟 subprocess.CompletedProcess 兼容: returncode/stdout/stderr)
    """
    if limits is None or not is_available():
        # fail-open (per 守门 #1 fail-open + #6 跨平台降级)
        result = subprocess.run(
            cmd,
            capture_output=capture_output,
            text=True,
            timeout=timeout,
            check=False,
            cwd=str(cwd) if cwd else None,
        )
        yield _SandboxResult(result.returncode, result.stdout, result.stderr)
        return

    # Windows Job Object path
    import win32job

    job = _create_windows_job(limits)
    try:
        # 用 Popen 而不是 run, 以便 assign pid to job
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE if capture_output else None,
            stderr=subprocess.PIPE if capture_output else None,
            text=True,
            cwd=str(cwd) if cwd else None,
        )
        # 把子进程 assign 到 job (per 守门 v0.2 §3.3)
        _assign_process_to_job(job, proc.pid)
        try:
            stdout, stderr = proc.communicate(timeout=timeout)
            yield _SandboxResult(proc.returncode, stdout or "", stderr or "")
        except subprocess.TimeoutExpired:
            # Job close 会级联 kill proc
            try:
                proc.kill()
                proc.wait(timeout=5)
            except Exception:  # noqa: BLE001
                pass
            raise
    finally:
        # Close job (per 守门 v0.2 §3.2 d kill_on_job_close)
        # CloseHandle 在 win32api 而不是 win32job
        try:
            import win32api
            win32api.CloseHandle(job)
        except Exception as e:  # noqa: BLE001
            logger.warning("[sandbox] close job failed: %s", e)


def make_default_sandbox_limits() -> SandboxLimits:
    """默认 sandbox 限制 (per 守门 v0.2 sandbox 默认值).

    适合 mavis task dispatch 子代理 30 min timeout 场景:
      - 1 GB 内存 (子代理可能 cargo build, 留 buffer)
      - 50% CPU (1 核的 1 半, 跟 dispatcher 共享)
      - 100 进程 (子代理可能派生 cargo + rustc + test runner)
      - kill_on_parent_exit = True (默认)
    """
    return SandboxLimits(
        memory_mb=1024,
        cpu_percent=50,
        max_processes=100,
        kill_on_parent_exit=True,
    )
