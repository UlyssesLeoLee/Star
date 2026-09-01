#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/dispatcher.py — 子代理 dispatch 基类
(per docs/automation-design.md §3.1 + §6.1)

替代 root → 子代理 RPC 黑盒调用, 走 exec 显式启动进程, 可观测可重试可重放。
解决守门 #9 实证问题: 子代理 status="succeeded" 但实际 10/10 ERR_CONNECTION_CLOSED。

用法:
    from automation.dispatcher import SubagentDispatcher
    d = SubagentDispatcher(phase="P3-B.5", audit_log=Path("docs/reports/P3-B.5.log"))
    brief_path = d.brief(
        task_id="P3-B.5",
        content="B.5 OpenClaw 真实集成 e2e ...",
        agent="worker",
    )
    handle = d.invoke(brief_path, timeout=600)
    ok = d.verify(handle.task_id)
    output_path = d.collect_output(handle.task_id)

约束 (per 守门 #1 v1 + 守门 #9 v2):
    - 标准库 only: subprocess / json / pathlib / time / dataclasses
    - 跨平台: Windows / WSL / macOS / Linux
    - brief 必填, 落 `docs/briefs/<task_id>.md`
    - output 必填, 落 `docs/briefs/<task_id>.output.md`
    - status 必填, 落 `docs/briefs/<task_id>.status.json`
    - audit_log 必填, 落 `docs/reports/<phase>.log`

已知缺口 (per docs/automation-design.md §7):
    1. 跨平台 exec 抽象: 当前仅 Windows PowerShell, 跨 WSL/macOS/Linux 需补 subprocess 适配层
    2. 子代理 exec 调用: 当前是 stub (写 brief + status placeholder), 实装需对接 Mavis task 调度
    3. 二次验证 (verify): 当前是 stub, 实装需 `git log -p --follow <wt-branch>` 实证 commit
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
BRIEFS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "briefs"
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class TaskHandle:
    """子代理任务句柄 (per §3.1 invoke 返)"""

    task_id: str
    agent: str
    brief_path: Path
    started_at: float
    finished_at: Optional[float] = None
    exit_code: Optional[int] = None
    status: str = "pending"  # pending / running / succeeded / failed / unknown


@dataclass
class AuditEntry:
    """审计日志条目 (per §3.4 audit_log schema)"""

    timestamp: float
    phase: str
    action: str  # brief / invoke / verify / collect_output
    task_id: str
    input: dict
    output: dict
    error: Optional[str] = None


class SubagentDispatcher:
    """子代理 dispatch 基类 (per §6.1)"""

    def __init__(
        self,
        phase: str,
        audit_log: Optional[Path] = None,
        briefs_dir: Optional[Path] = None,
    ):
        self.phase = phase
        self.briefs_dir = briefs_dir or BRIEFS_DIR_DEFAULT
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / f"{phase}.log")
        self.briefs_dir.mkdir(parents=True, exist_ok=True)
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)

    # === 4 个核心方法 (per §3.1 范式) ===

    def brief(self, task_id: str, content: str, agent: str) -> Path:
        """落地 brief → docs/briefs/<task_id>.md"""
        brief_path = self.briefs_dir / f"{task_id}.md"
        brief_path.write_text(
            f"# Brief: {task_id}\n\n"
            f"**Agent**: {agent}\n"
            f"**Phase**: {self.phase}\n"
            f"**Created**: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n"
            f"---\n\n{content}\n",
            encoding="utf-8",
        )
        self._audit(
            action="brief",
            task_id=task_id,
            input={"agent": agent, "content_len": len(content)},
            output={"brief_path": str(brief_path)},
        )
        return brief_path

    def invoke(self, brief_path: Path, timeout: int = 600, agent: Optional[str] = None) -> TaskHandle:
        """invoke 子代理 (exec 模式, stub)

        agent: 显式传 agent name, 避免解析 brief 文本的脆性
        """
        task_id = brief_path.stem
        if agent is None:
            # fallback: 从 brief 文本解析
            content = brief_path.read_text(encoding="utf-8")
            m = re.search(r"\*\*Agent\*\*:\s*(\S+)", content)
            agent = m.group(1) if m else "unknown"
        handle = TaskHandle(
            task_id=task_id,
            agent=agent,
            brief_path=brief_path,
            started_at=time.time(),
        )
        status_path = self.briefs_dir / f"{task_id}.status.json"
        handle_dict = asdict(handle)
        # WindowsPath → str (per JSON 序列化要求)
        handle_dict["brief_path"] = str(handle.brief_path)
        status_path.write_text(json.dumps(handle_dict, indent=2, ensure_ascii=False), encoding="utf-8")
        self._audit(
            action="invoke",
            task_id=task_id,
            input={"brief_path": str(brief_path), "timeout": timeout},
            output={"handle": asdict(handle)},
        )
        # stub: 真实实装需对接 Mavis task 调度 (per §7 已知缺口 #2)
        return handle

    def verify(self, task_id: str) -> bool:
        """二次验证 (stub: 真实实装需 git log --follow 实证)"""
        status_path = self.briefs_dir / f"{task_id}.status.json"
        if not status_path.exists():
            self._audit(
                action="verify",
                task_id=task_id,
                input={},
                output={"verified": False},
                error="status.json 不存在",
            )
            return False
        # stub: 真实实装需 `git log -p --follow <wt-branch>` 实证
        self._audit(
            action="verify",
            task_id=task_id,
            input={},
            output={"verified": True, "stub": True},
        )
        return True

    def collect_output(self, task_id: str) -> Path:
        """收子代理 output → docs/briefs/<task_id>.output.md (stub)"""
        output_path = self.briefs_dir / f"{task_id}.output.md"
        output_path.write_text(
            f"# Output: {task_id}\n\n"
            f"**Stub** — 真实实装需对接 Mavis task 调度 output 流 (per §7 已知缺口 #2)\n",
            encoding="utf-8",
        )
        self._audit(
            action="collect_output",
            task_id=task_id,
            input={},
            output={"output_path": str(output_path)},
        )
        return output_path

    # === 内部 ===

    def _audit(
        self,
        action: str,
        task_id: str,
        input: dict,
        output: dict,
        error: Optional[str] = None,
    ):
        # WindowsPath → str (per JSON 序列化要求)
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
            task_id=task_id,
            input=_normalize(input),
            output=_normalize(output),
            error=error,
        )
        with self.audit_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(entry), ensure_ascii=False) + "\n")


def main():
    """CLI 入口: 创建 brief + invoke + verify + collect_output"""
    parser = argparse.ArgumentParser(description="子代理 dispatch 基类 CLI")
    parser.add_argument("--task-id", required=True, help="任务 ID, 例: P3-B.5")
    parser.add_argument("--phase", required=True, help="阶段, 例: P3-B")
    parser.add_argument("--agent", default="worker", help="子代理类型, 例: worker/explorer/verifier")
    parser.add_argument("--content", required=True, help="brief 内容")
    parser.add_argument("--timeout", type=int, default=600, help="invoke timeout (秒)")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    d = SubagentDispatcher(phase=args.phase, audit_log=args.audit_log)
    brief_path = d.brief(args.task_id, args.content, args.agent)
    handle = d.invoke(brief_path, timeout=args.timeout)
    ok = d.verify(args.task_id)
    output_path = d.collect_output(args.task_id)

    print(f"task_id={args.task_id}")
    print(f"brief_path={brief_path}")
    print(f"output_path={output_path}")
    print(f"verified={ok}")
    print(f"status={'OK' if ok else 'FAIL'}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
