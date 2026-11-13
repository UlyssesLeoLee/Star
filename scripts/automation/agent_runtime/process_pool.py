#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""agent_runtime.process_pool — 常驻 subprocess worker 池 (per 02-BD §3.1 ProcessPool)。

职责: 8-16 常驻 worker (per 02-BD §3.1), 替代 RPC 走 subprocess (per 守门 #24)。
预热 (warm_up, per 守门 #24 v24) 与 idle 回收是池级概念; 本骨架实现 bounded
concurrent submit (Semaphore, per SRS §44 禁止 unlimited spawn)。

复用现役模块 (AGENTS.md §0 能 import 的绝不重写):
    automation.dispatcher.SubagentDispatcher (守门 #24 subprocess 路径,
    brief/invoke/verify/collect_output 已实装) 作为软依赖: 可 import 时
    dispatch_subagent 走它, 否则走裸 subprocess.run list-mode。

约束 (per 守门 #6 + #9 v3): subprocess list-mode, shell=False; env 值不打印
(per 守门 #5)。
"""

from __future__ import annotations

import subprocess
import threading
import time
from concurrent.futures import ThreadPoolExecutor, TimeoutError as FutureTimeout
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional

#: 软依赖: automation.dispatcher (per AGENTS.md §0 复用现役模块)
try:
    from automation.dispatcher import SubagentDispatcher as _SubagentDispatcher
    _DISPATCHER_AVAILABLE = True
except ImportError:  # pragma: no cover — 独立运行 (不在 scripts/ 包上下文)
    _SubagentDispatcher = None
    _DISPATCHER_AVAILABLE = False


@dataclass
class PoolResult:
    """一次 submit 结果 (per 02-BD §3.1 WorkerHandle 简化)。"""

    cmd: List[str]
    exit_code: Optional[int] = None
    stdout_tail: str = ""
    stderr_tail: str = ""
    duration_sec: float = 0.0
    status: str = field(default="pending")  # pending/succeeded/failed/timeout


class ProcessPool:
    """Bounded subprocess worker 池 (per 02-BD §3.1 + 守门 #24)。"""

    def __init__(self, size: int = 16, timeout: float = 600.0, cwd: Optional[Path] = None):
        if size < 1:
            raise ValueError("size 必须 >= 1 (禁止无界池, per SRS §44)")
        self.size = size
        self.timeout = timeout
        self.cwd = cwd
        #: Semaphore 限流 (per 02-BD §3.1 semaphore)
        self._sem = threading.BoundedSemaphore(size)
        self._executor = ThreadPoolExecutor(max_workers=size, thread_name_prefix="proc-pool")

    def pool_size(self) -> int:
        return self.size

    def submit(self, cmd: List[str]) -> PoolResult:
        """submit(cmd) → 同步执行 (池内并发上限 size; 调用方决定并发)。

        shell=False list-mode (per 守门 #6); 超时 → status=timeout。
        """
        if not cmd:
            raise ValueError("cmd 不能为空")
        result = PoolResult(cmd=list(cmd))
        with self._sem:
            result.status = "running"
            start = time.monotonic()
            try:
                proc = subprocess.run(
                    cmd,
                    capture_output=True,
                    text=True,
                    timeout=self.timeout,
                    check=False,
                    cwd=self.cwd,
                )
                result.exit_code = proc.returncode
                result.stdout_tail = (proc.stdout or "")[-500:]
                result.stderr_tail = (proc.stderr or "")[-500:]
                result.status = "succeeded" if proc.returncode == 0 else "failed"
            except subprocess.TimeoutExpired:
                result.status = "timeout"
            finally:
                result.duration_sec = time.monotonic() - start
        return result

    def submit_many(self, cmds: List[List[str]]) -> List[PoolResult]:
        """并发提交多个命令 (bounded: 池大小即并发上限)。"""
        futures = [self._executor.submit(self.submit, cmd) for cmd in cmds]
        out = []
        for fut in futures:
            try:
                out.append(fut.result(timeout=self.timeout + 10))
            except FutureTimeout:  # pragma: no cover — 双保险
                out.append(PoolResult(cmd=[], status="timeout"))
        return out

    def dispatch_subagent(
        self,
        task_id: str,
        content: str,
        agent: str = "worker",
        phase: str = "agent-runtime",
        briefs_dir: Optional[Path] = None,
        audit_log: Optional[Path] = None,
        timeout: float = 600.0,
    ) -> dict:
        """守门 #24 subprocess 路径派发 (复用 automation.dispatcher 现役模块)。

        返回 {task_id, status, exit_code, brief_path}。dispatcher 不可用时
        状态 deferred (per dispatcher.py 已知缺口 #2 语义)。
        briefs_dir/audit_log 建议显式传临时目录, 避免污染 repo。
        """
        if _DISPATCHER_AVAILABLE:
            dispatcher_cls = _SubagentDispatcher
            assert dispatcher_cls is not None  # guarded by _DISPATCHER_AVAILABLE
            dispatcher = dispatcher_cls(phase=phase, briefs_dir=briefs_dir, audit_log=audit_log)
            brief_path = dispatcher.brief(task_id=task_id, content=content, agent=agent)
            handle = dispatcher.invoke(brief_path, timeout=int(timeout))
            return {
                "task_id": task_id,
                "status": handle.status,
                "exit_code": handle.exit_code,
                "brief_path": str(brief_path),
            }
        # 软依赖缺失 → deferred (per dispatcher.py 已知缺口 #2 同语义)
        return {
            "task_id": task_id,
            "status": "deferred",
            "exit_code": None,
            "brief_path": "",
        }

    def shutdown(self) -> None:
        self._executor.shutdown(wait=True)
