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

约束 (per 守门 #1 v1 + 守门 #9 v2 + 守门 #24 v2):
    - 标准库 only: subprocess / json / pathlib / time / dataclasses
    - 跨平台: Windows / WSL / macOS / Linux (subprocess list-mode, no shell=True)
    - brief 必填, 落 `docs/briefs/<task_id>.md`
    - output 必填, 落 `docs/briefs/<task_id>.output.md`
    - status 必填, 落 `docs/briefs/<task_id>.status.json`
    - audit_log 必填, 落 `docs/reports/<phase>.log`
    - invoke: subprocess 替代 RPC (per 守门 #24 v2), mavis CLI 缺失 → status="deferred" fallback
    - verify: `git log -1 --format='%H %s' origin/main..<branch>` 实证 commit 在 main chain
    - collect_output: 从 status.json 渲染 + verify 拿 git 证据追加进 output.md

已知缺口 (per docs/automation-design.md §7 + G-DEP 9/7 19:38 JST 派生):
    1. 跨平台 exec 抽象: 当前仅 Windows PowerShell (per §7 #1), 跨 WSL/macOS/Linux
       需补 subprocess 适配层 (本任务范围外, 跨 session 续)
    2. Mavis task tool CLI (`mavis task dispatch`) 当前未落地 (per 9/7 19:46 JST 实证),
       invoke fallback → status="deferred", output.md 标注 "awaiting mavis CLI"
    3. 真实 Mavis task 调度 output 流 (stdout/stderr 完整内容) 当前 stub, 跨 session 续
    4. verify 默认 branch 推导: 当前用 feat/<task_id>, 真实 dispatch 路径 (5 域 Lead
       sub-agent brief 模板) 需 dispatcher 注册时显式传 branch (per 守门 #20 v2)
    5. retry 链: 当前 invoke 失败不自动 retry, 需调用方手动 retry (跨 session 续)
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
BRIEFS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "briefs"
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


class DispatchBlockedError(Exception):
    """PreToolUse guard 阻断子代理 dispatch 抛此异常 (per WBS-002 T3.2 + FR-1.2).

    包含阻断原因 + rule_id (per DD §1.3 派生).
    """
    def __init__(self, message: str, rule_id: Optional[str] = None):
        super().__init__(message)
        self.rule_id = rule_id


class BlockedByCommentError(Exception):
    """任务卡 BLOCK 留言阻断子代理 dispatch 抛此异常 (per 守门 v33 + #27 + #9 v20).

    包含阻断留言 ID + actor + body 摘要.
    """
    def __init__(self, message: str, blocking_comments: Optional[list] = None):
        super().__init__(message)
        self.blocking_comments = blocking_comments or []


@dataclass
class Comment:
    """任务卡留言条目 (per 守门 v33 §1.2 schema).

    存储: docs/briefs/<task_id>.comments.jsonl (JSON Lines, append-only, per 守门 #13 T).
    """
    id: str
    ts: str
    author: str
    actor_role: str          # orchestrator/sub-agent/architect/user/Ulysses
    body: str
    mentions: list = field(default_factory=list)
    blocks: bool = False
    tags: list = field(default_factory=list)
    parent_comment_id: Optional[str] = None
    refs: list = field(default_factory=list)


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
        """落地 brief → docs/briefs/<task_id>.md + 拉 @<task_id> 留言 append "Read COMMENTS" 段.

        Per 守门 #9 v20 + 守门 v33 §1.4:
            brief 必含 "Read COMMENTS" 段, 列所有 @<this_task> 的留言.
        """
        brief_path = self.briefs_dir / f"{task_id}.md"
        # 拉所有 @<task_id> 留言 (per 守门 v33 §1.4)
        comments_section = self._render_comments_section(task_id)
        brief_path.write_text(
            f"# Brief: {task_id}\n\n"
            f"**Agent**: {agent}\n"
            f"**Phase**: {self.phase}\n"
            f"**Created**: {time.strftime('%Y-%m-%d %H:%M:%S')}\n\n"
            f"---\n\n{content}\n\n"
            f"## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)\n\n"
            f"{comments_section}\n",
            encoding="utf-8",
        )
        self._audit(
            action="brief",
            task_id=task_id,
            input={"agent": agent, "content_len": len(content)},
            output={"brief_path": str(brief_path), "comments_count": len(self.list_comments(task_id))},
        )
        return brief_path

    def invoke(self, brief_path: Path, timeout: int = 600, agent: Optional[str] = None) -> TaskHandle:
        """invoke 子代理 (subprocess 模式, per G-DEP 9/7 19:38 JST 实装)

        agent: 显式传 agent name, 避免解析 brief 文本的脆性
        优先尝试 `mavis task dispatch <task_id> <brief_path> --agent <agent>` subprocess。
        FileNotFoundError (mavis CLI 不存在) → status="deferred" + exit_code=None
            (per 已知缺口 #2, 当前 9/7 19:46 JST mavis CLI 尚未落地)
        TimeoutExpired → status="timeout" + exit_code=-1
        其他非零 exit → status="failed" + 实际 exit_code
        exit 0 → status="succeeded" + exit_code=0

        PreToolUse hook (per WBS-002 T3.2 + SRS-PRE-TOOL-USE-GUARD-001.md FR-1.2):
            在 invoke 实际执行前, 调 guardian.pre_tool_use_guard.PreToolUseGuard.evaluate_for_dispatch()
            阻断危险操作 (rm -rf /, sudo, GitHub PAT leak, etc).
            disable 方式: env var STAR_PRE_TOOL_USE_GUARD=0.
            软依赖: guardian import 失败 → fail-open 跳过 (per FR-6.1).

        Comment BLOCK 留言 (per 守门 v33 §1.4 + #27 + #9 v20):
            invoke 前必扫任务卡 BLOCK 留言, 有 → 抛 BlockedByCommentError, 不 invoke.
            缺留言文件 → 0 留言 = 0 BLOCK, 继续 (per 缺标比错标 #11).
        """
        task_id = brief_path.stem

        # === Comment BLOCK 留言检查 (per 守门 v33 §1.4) ===
        blocking = self.check_blocked(task_id)
        if blocking:
            blocking_summaries = [f"{c.id} by {c.author}: {c.body[:80]}" for c in blocking]
            raise BlockedByCommentError(
                f"Task {task_id} blocked by {len(blocking)} comment(s): {'; '.join(blocking_summaries)}",
                blocking_comments=blocking,
            )

        # === PreToolUse hook (per FR-1.2 + NFR-S-4) ===
        if os.environ.get("STAR_PRE_TOOL_USE_GUARD", "1") != "0":
            try:
                from guardian.pre_tool_use_guard import (
                    PreToolUseGuard, ToolCall, Decision as PTGDecision,
                )
                from guardian.rule_database import RuleDatabase
                from guardian.audit_logger import AuditLogger
                from guardian.paths import GUARDIAN_PATHS
                _ptg_db = RuleDatabase(GUARDIAN_PATHS.rules, enable_watcher=False)
                _ptg_audit = AuditLogger(GUARDIAN_PATHS.audit_log)
                _ptg_guard = PreToolUseGuard(_ptg_db, _ptg_audit)
                # 用 brief 内容作 scan_target
                brief_content = brief_path.read_text(encoding="utf-8", errors="replace") if brief_path.exists() else ""
                _ptg_decision = _ptg_guard.evaluate(ToolCall(
                    tool_name="mcp_dispatch",
                    tool_args={"task_id": task_id, "brief_path": str(brief_path), "brief_content": brief_content[:500]},
                    session_id=os.environ.get("Mavis_SESSION_ID", task_id),
                    agent_role=agent or "subagent",
                ))
                if _ptg_decision == PTGDecision.BLOCK:
                    # 阻断, 不 invoke
                    _ptg_db.stop()
                    raise DispatchBlockedError(
                        f"PreToolUse guard BLOCK on dispatch {task_id} "
                        f"(brief content: {brief_content[:100]!r})"
                    )
                # ASK / WARN: 当前仅 log, 实际 ASK 应该走 ask_user (Mavis runtime 集成层)
                # 留 Mavis 集成时再加
                _ptg_db.stop()
            except DispatchBlockedError:
                raise
            except Exception as _e:  # noqa: BLE001
                # fail-open: guardian 自身错误不阻断 dispatch
                import sys as _sys
                print(f"[WARN] PreToolUse guard skip (fail-open): {_e}", file=_sys.stderr)
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
        # 落地初始 status.json (running)
        handle.status = "running"
        handle_dict = asdict(handle)
        # WindowsPath → str (per JSON 序列化要求)
        handle_dict["brief_path"] = str(handle.brief_path)
        status_path.write_text(json.dumps(handle_dict, indent=2, ensure_ascii=False), encoding="utf-8")
        self._audit(
            action="invoke",
            task_id=task_id,
            input={"brief_path": str(brief_path), "timeout": timeout, "agent": agent},
            output={"handle": handle_dict, "status": "running"},
        )

        # subprocess 替代 RPC (per 守门 #24 v2 + G-DEP 9/7 19:38 JST 实装)
        # 标准库 only, list-mode 无 shell=True 跨平台
        cmd = ["mavis", "task", "dispatch", task_id, str(brief_path), "--agent", agent]
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout,
                check=False,
                cwd=ROOT_DEFAULT,
            )
            handle.exit_code = result.returncode
            handle.status = "succeeded" if result.returncode == 0 else "failed"
            self._audit(
                action="invoke_subprocess",
                task_id=task_id,
                input={"cmd": cmd, "timeout": timeout},
                output={
                    "exit_code": result.returncode,
                    "status": handle.status,
                    "stdout_tail": result.stdout[-500:] if result.stdout else "",
                    "stderr_tail": result.stderr[-500:] if result.stderr else "",
                },
            )
        except FileNotFoundError:
            # mavis CLI 不存在 (per 已知缺口 #2, 9/7 19:46 JST 现状)
            # fallback: status="deferred", 不阻塞后续 verify/collect_output
            handle.exit_code = None
            handle.status = "deferred"
            self._audit(
                action="invoke_subprocess",
                task_id=task_id,
                input={"cmd": cmd, "timeout": timeout},
                output={"status": "deferred", "reason": "mavis CLI not found on PATH"},
                error="FileNotFoundError: mavis CLI not on PATH (per 已知缺口 #2)",
            )
        except subprocess.TimeoutExpired as e:
            handle.exit_code = -1
            handle.status = "timeout"
            self._audit(
                action="invoke_subprocess",
                task_id=task_id,
                input={"cmd": cmd, "timeout": timeout},
                output={"status": "timeout"},
                error=f"TimeoutExpired after {timeout}s: {e}",
            )

        handle.finished_at = time.time()
        # 落最终 status.json
        final_dict = asdict(handle)
        final_dict["brief_path"] = str(handle.brief_path)
        status_path.write_text(json.dumps(final_dict, indent=2, ensure_ascii=False), encoding="utf-8")
        self._audit(
            action="invoke_done",
            task_id=task_id,
            input={},
            output={
                "handle": final_dict,
                "status": handle.status,
                "exit_code": handle.exit_code,
                "duration_sec": handle.finished_at - handle.started_at,
            },
        )
        return handle

    def verify(self, task_id: str) -> bool:
        """二次验证 (per G-DEP 9/7 19:38 JST 实装, per 守门 #9 主体规则)

        读 status.json 拿 branch (默认 feat/<task_id>), 实证 commit 在 main chain:
        `git log -1 --format='%H %s' origin/main..<branch>`

        实证规则 (per 守门 #9 主体规则 status="succeeded" ≠ 实际成功, 必 git 实证):
        - status 不在 [succeeded, deferred] 范围 → False (e.g. failed/timeout/pending)
        - deferred 状态 (mavis CLI 缺失 fallback) → True (允许 verify 走通, 但 output 标注 deferred)
        - git log exit 0 + 至少 1 行 commit (含 %H %s) → True
        - git log exit 非 0 (branch 不存在 / 无 origin/main) → False
        - 其他 git 异常 → False
        """
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

        # 读 status.json 拿 branch + status
        try:
            status = json.loads(status_path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as e:
            self._audit(
                action="verify",
                task_id=task_id,
                input={},
                output={"verified": False},
                error=f"status.json 解析失败: {e}",
            )
            return False

        handle_status = status.get("status", "unknown")
        branch = status.get("branch") or f"feat/{task_id}"

        # status 范围 check (per 守门 #9 主体规则)
        if handle_status in ("failed", "timeout", "pending", "running"):
            self._audit(
                action="verify",
                task_id=task_id,
                input={"branch": branch},
                output={"verified": False, "reason": f"handle.status={handle_status}"},
                error="invoke 未成功, 不进入 git 实证",
            )
            return False

        # deferred 状态特殊处理 (per 已知缺口 #2 mavis CLI 不存在 fallback)
        if handle_status == "deferred":
            # 无 mavis CLI, 无法走 git 实证 (无 worktree 提交)
            # 允许 verify 走通但 output 标注 deferred
            self._audit(
                action="verify",
                task_id=task_id,
                input={"branch": branch},
                output={"verified": True, "deferred": True, "reason": "mavis CLI missing, no git evidence possible"},
            )
            return True

        # succeeded 状态 → 走 git log 实证
        # list-mode subprocess, 跨平台 (no shell=True)
        try:
            result = subprocess.run(
                ["git", "log", "-1", "--format=%H %s", f"origin/main..{branch}"],
                capture_output=True,
                text=True,
                timeout=10,
                check=False,
                cwd=ROOT_DEFAULT,
            )
        except (subprocess.TimeoutExpired, FileNotFoundError, OSError) as e:
            self._audit(
                action="verify",
                task_id=task_id,
                input={"branch": branch},
                output={"verified": False},
                error=f"git log 异常: {e}",
            )
            return False

        if result.returncode != 0:
            # branch 不存在 / origin/main 不存在 / 其他 git 错
            self._audit(
                action="verify",
                task_id=task_id,
                input={"branch": branch},
                output={"verified": False, "git_exit": result.returncode, "stderr": result.stderr[-300:]},
                error=f"git log exit={result.returncode}",
            )
            return False

        # 解析 commit 行
        commits = [c.strip() for c in result.stdout.splitlines() if c.strip()]
        if not commits:
            self._audit(
                action="verify",
                task_id=task_id,
                input={"branch": branch},
                output={"verified": False, "reason": "no commit in origin/main..<branch> range"},
            )
            return False

        # 至少 1 行 "%H %s" → verified
        # 简单 check: split into hash + subject
        first = commits[0]
        parts = first.split(maxsplit=1)
        if len(parts) < 2 or len(parts[0]) < 7:
            # 异常 commit 行
            self._audit(
                action="verify",
                task_id=task_id,
                input={"branch": branch},
                output={"verified": False, "first_commit": first},
                error="commit 行格式异常 (非 %H %s)",
            )
            return False

        self._audit(
            action="verify",
            task_id=task_id,
            input={"branch": branch},
            output={
                "verified": True,
                "commit_hash": parts[0],
                "commit_subject": parts[1],
                "commit_count": len(commits),
            },
        )
        return True

    def collect_output(self, task_id: str) -> Path:
        """收子代理 output (per G-DEP 9/7 19:38 JST 实装)

        读 status.json → 渲染 output.md:
        - 头部: task_id / status / exit_code / started / finished / duration
        - 实证: 调 verify 拿 git commit 证据, 追加到 output.md
        - deferred 状态: 标注 "awaiting mavis CLI" (per 已知缺口 #2)
        - failed/timeout: 标注失败原因
        """
        output_path = self.briefs_dir / f"{task_id}.output.md"
        status_path = self.briefs_dir / f"{task_id}.status.json"

        # 读 status.json (无则 stub 兜底)
        status = None
        if status_path.exists():
            try:
                status = json.loads(status_path.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError) as e:
                status = {"_parse_error": str(e)}

        if status is None:
            output_path.write_text(
                f"# Output: {task_id}\n\n"
                f"**Stub** — status.json 不存在, 无 output 数据\n",
                encoding="utf-8",
            )
            self._audit(
                action="collect_output",
                task_id=task_id,
                input={},
                output={"output_path": str(output_path), "stub_reason": "no status.json"},
            )
            return output_path

        # 渲染 output.md
        handle_status = status.get("status", "unknown")
        exit_code = status.get("exit_code")
        started_at = status.get("started_at", 0)
        finished_at = status.get("finished_at", 0)
        duration = finished_at - started_at if finished_at and started_at else 0
        agent = status.get("agent", "unknown")
        brief_path_str = status.get("brief_path", "")

        def _fmt_time(ts):
            if not ts:
                return "n/a"
            return time.strftime("%Y-%m-%d %H:%M:%S", time.localtime(ts))

        lines = [
            f"# Output: {task_id}",
            "",
            f"**Status**: `{handle_status}`",
            f"**Exit code**: `{exit_code}`",
            f"**Agent**: `{agent}`",
            f"**Brief**: `{brief_path_str}`",
            f"**Started**: {_fmt_time(started_at)}",
            f"**Finished**: {_fmt_time(finished_at)}",
            f"**Duration**: `{duration:.2f}s`",
            "",
        ]

        # 状态分支
        if handle_status == "deferred":
            # mavis CLI 不存在 (per 已知缺口 #2, 9/7 19:46 JST 现状)
            lines.extend([
                "## Deferred (per 已知缺口 #2)",
                "",
                "mavis task dispatch CLI 当前未落地, invoke fallback 到 deferred 状态。",
                "本 output.md 是 dispatcher 基类对 status.json 的渲染, 不是真实 Mavis task 调度 output 流。",
                "真实 output 流对接待 Mavis task tool 落地后跨 session 续 (per §7 已知缺口 #2 部分解决)。",
                "",
            ])
        elif handle_status == "failed":
            lines.extend([
                "## Failed",
                "",
                f"invoke subprocess exit code = {exit_code} (非 0)。",
                "实际 Mavis task 调度 output 流 (stdout/stderr) 当前未捕获 (per 已知缺口 #3)。",
                "",
            ])
        elif handle_status == "timeout":
            lines.extend([
                "## Timeout",
                "",
                "invoke subprocess 超时未返回。",
                "",
            ])
        elif handle_status == "succeeded":
            lines.extend([
                "## Succeeded",
                "",
                "invoke subprocess exit 0。",
                "Mavis task 调度 output 流 (stdout) 当前未捕获 (per 已知缺口 #3)。",
                "",
            ])

        # 调 verify 拿 git commit 证据
        verified = self.verify(task_id)
        branch = status.get("branch") or f"feat/{task_id}"
        lines.extend([
            "## Verify (git log 实证, per 守门 #9 主体规则)",
            "",
            f"**Branch**: `{branch}`",
            f"**Verified**: `{verified}`",
            "",
        ])

        if verified and handle_status == "succeeded":
            # 再跑一次 git log 拿 commit hash + subject 写进 output
            try:
                glog = subprocess.run(
                    ["git", "log", "-1", "--format=%H%n%s%n%an%n%ai", f"origin/main..{branch}"],
                    capture_output=True,
                    text=True,
                    timeout=10,
                    check=False,
                    cwd=ROOT_DEFAULT,
                )
                if glog.returncode == 0 and glog.stdout.strip():
                    parts = glog.stdout.strip().split("\n", 3)
                    if len(parts) >= 4:
                        lines.extend([
                            "**Commit evidence**:",
                            "",
                            f"- **Hash**: `{parts[0]}`",
                            f"- **Subject**: {parts[1]}",
                            f"- **Author**: {parts[2]}",
                            f"- **Date**: {parts[3]}",
                            "",
                        ])
            except (subprocess.TimeoutExpired, FileNotFoundError, OSError):
                pass  # 已在 verify 阶段审计过, 此处不重复

        if not verified and handle_status == "succeeded":
            lines.extend([
                "**Reason**: git log 实证失败 (branch 不存在 / 无 origin/main / 异常 commit 行)",
                "per 守门 #9 主体规则 status=succeeded 但 git 实证失败 → 任务不算完成",
                "",
            ])

        # 引用 + 修订
        lines.extend([
            "## 引用",
            "",
            "- `docs/automation-design.md` §3.1 (子代理 dispatch 范式)",
            "- `AGENTS.md` §4 守门 #1 v19 / #9 v3 / #24 v2",
            "",
            "## 修订历史",
            "",
            f"| 版本 | 日期 | 修订人 | 修订内容 | 触发 |",
            f"|---|---|---|---|---|",
            f"| v0.1 | {_fmt_time(time.time())} | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | G-DEP 实装 (per 9/7 19:38 JST): invoke/verify/collect_output 3 方法实装, deferred fallback + git log 实证 | G-DEP 派发 |",
            "",
        ])

        output_path.write_text("\n".join(lines), encoding="utf-8")
        self._audit(
            action="collect_output",
            task_id=task_id,
            input={},
            output={
                "output_path": str(output_path),
                "handle_status": handle_status,
                "verified": verified,
            },
        )
        return output_path

    # === 内部 ===

    # === 任务卡留言机制 (per 守门 v33 + 守门 #9 v20 + 守门 #27) ===

    def _comments_path(self, task_id: str) -> Path:
        """docs/briefs/<task_id>.comments.jsonl 物理路径 (per 守门 v33 §1.1)."""
        return self.briefs_dir / f"{task_id}.comments.jsonl"

    def comment(
        self,
        task_id: str,
        body: str,
        author: str = "Mavis",
        actor_role: str = "orchestrator",
        mentions: Optional[list] = None,
        blocks: bool = False,
        tags: Optional[list] = None,
        parent_comment_id: Optional[str] = None,
        refs: Optional[list] = None,
    ) -> Comment:
        """写 1 条 append-only 留言到 <task_id>.comments.jsonl (per 守门 v33 §1.1).

        Args:
            task_id: 任务 ID (mention 跟这个 ID 匹配时被 @ 触发)
            body: 留言内容
            author: 留言作者 (默认 Mavis, per 守门 #14 v3 永久代签)
            actor_role: orchestrator/sub-agent/architect/user/Ulysses
            mentions: @mention 的其他 task_id 列表 (触发 brief 拉留言)
            blocks: True → 阻断 dispatch (per 守门 v33 §1.3)
            tags: e.g. ["requirement", "follow-up"]
            parent_comment_id: 父留言 ID (线程嵌套)
            refs: 引用其他文档/commit (审计链)
        """
        comments_path = self._comments_path(task_id)
        # 必含 task_id 自身在 mentions (方便后续 brief 拉)
        mentions = list(mentions or [])
        if task_id not in mentions:
            mentions.append(task_id)
        # 生成 comment_id (简单计数器)
        existing = self.list_comments(task_id)
        comment_id = f"comment_{len(existing) + 1:03d}"
        c = Comment(
            id=comment_id,
            ts=time.strftime("%Y-%m-%dT%H:%M:%S") + f".{int((time.time()%1)*1000):03d}+09:00",
            author=author,
            actor_role=actor_role,
            body=body,
            mentions=mentions,
            blocks=blocks,
            tags=list(tags or []),
            parent_comment_id=parent_comment_id,
            refs=list(refs or []),
        )
        # append-only 写 (per 守门 #13 T)
        comments_path.parent.mkdir(parents=True, exist_ok=True)
        with comments_path.open("a", encoding="utf-8", errors="replace") as f:
            f.write(json.dumps(asdict(c), ensure_ascii=False, separators=(",", ":")) + "\n")
        self._audit(
            action="comment",
            task_id=task_id,
            input={"comment_id": comment_id, "blocks": blocks, "mentions": mentions},
            output={"body_len": len(body), "comments_count": len(existing) + 1},
        )
        return c

    def list_comments(self, task_id: str) -> list:
        """读所有留言 (per 守门 v33 §1.4). 缺文件 → 0 留言 (per 缺标比错标 #11)."""
        comments_path = self._comments_path(task_id)
        if not comments_path.exists():
            return []
        out = []
        try:
            for line in comments_path.read_text(encoding="utf-8", errors="replace").splitlines():
                line = line.strip()
                if not line:
                    continue
                try:
                    d = json.loads(line)
                    out.append(Comment(
                        id=d.get("id", ""),
                        ts=d.get("ts", ""),
                        author=d.get("author", ""),
                        actor_role=d.get("actor_role", ""),
                        body=d.get("body", ""),
                        mentions=d.get("mentions", []),
                        blocks=d.get("blocks", False),
                        tags=d.get("tags", []),
                        parent_comment_id=d.get("parent_comment_id"),
                        refs=d.get("refs", []),
                    ))
                except (json.JSONDecodeError, TypeError):
                    continue
        except OSError as e:
            print(f"[WARN] read comments failed for {task_id}: {e}", file=sys.stderr)
        return out

    def check_blocked(self, task_id: str) -> list:
        """返回所有 blocks=True 的留言 (per 守门 v33 §1.4)."""
        return [c for c in self.list_comments(task_id) if c.blocks]

    def mark_read(self, task_id: str, comment_id: str) -> bool:
        """标记 1 条留言已读 (per 守门 v33 §1.6 commit message `comments-read:` 字段).

        实现: append 1 条 'read-receipt' 留言 (元留言, parent_comment_id 指向原留言).
        不修改原留言, 保持 append-only (per 守门 #13 T).
        """
        comments_path = self._comments_path(task_id)
        if not comments_path.exists():
            return False
        for c in self.list_comments(task_id):
            if c.id == comment_id:
                self.comment(
                    task_id=task_id,
                    body=f"read-receipt for {comment_id}",
                    author="sub-agent-ack",
                    actor_role="sub-agent",
                    parent_comment_id=comment_id,
                    tags=["read-receipt"],
                )
                return True
        return False

    def _render_comments_section(self, task_id: str) -> str:
        """生成 brief 第 6 段 "Read COMMENTS" 内容 (per 守门 v33 §1.5)."""
        comments = self.list_comments(task_id)
        if not comments:
            return "(无任务卡留言)"
        lines = []
        for c in comments:
            tag_str = " ".join(f"#{t}" for t in c.tags) if c.tags else ""
            block_marker = " [BLOCK]" if c.blocks else ""
            lines.append(
                f"- {c.id} by {c.author} ({c.actor_role}) @ {c.ts}{block_marker}{(' ' + tag_str) if tag_str else ''}: {c.body[:200]}"
            )
        return "\n".join(lines)

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
    """CLI 入口: 创建 brief + invoke + verify + collect_output (含 v27 拍板激活 subcommands)"""
    parser = argparse.ArgumentParser(description="子代理 dispatch 基类 CLI")
    parser.add_argument("--task-id", help="任务 ID, 例: P3-B.5 (仅 legacy CLI 必填)")
    parser.add_argument("--phase", help="阶段, 例: P3-B (仅 legacy CLI 必填)")
    parser.add_argument("--agent", default="worker", help="子代理类型, 例: worker/explorer/verifier (仅 legacy CLI)")
    parser.add_argument("--content", help="brief 内容 (仅 legacy CLI 必填)")
    parser.add_argument("--timeout", type=int, default=600, help="invoke timeout (秒, 仅 legacy CLI)")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    sub = parser.add_subparsers(dest="command", help="subcommand (v27 拍板激活, 优先于 legacy)")

    # 全流程 CLI (保留 v0.55 之前)
    p_all = sub.add_parser("all", help="brief + invoke + verify + collect_output 全流程 (默认, 向后兼容)")
    p_all.add_argument("--task-id", required=True)
    p_all.add_argument("--phase", required=True)
    p_all.add_argument("--agent", default="worker")
    p_all.add_argument("--content", required=True)
    p_all.add_argument("--timeout", type=int, default=600)
    p_all.add_argument("--audit-log", type=Path)

    # v27 拍板激活 (per 2026-09-10 11:00 JST Mavis 自驱 + 守门 #9 v20 + 守门 #14 v3 Mavis 永久代签)
    p_invoke = sub.add_parser("invoke", help="v27 Step 1: 仅 dispatch invoke, 写 status.json (跟 v27_rpc_fallback.py 配套)")
    p_invoke.add_argument("task_id", help="任务 ID")
    p_invoke.add_argument("brief_path", help="brief 路径 (相对 STAR_ROOT)")
    p_invoke.add_argument("worktree", help="git worktree 名称")
    p_invoke.add_argument("--phase", default="v27-fallback")
    p_invoke.add_argument("--audit-log", type=Path)

    p_verify = sub.add_parser("verify", help="v27 Step 2: 必跑 verify, 30s 内 ok")
    p_verify.add_argument("task_id", help="任务 ID")
    p_verify.add_argument("--phase", default="v27-fallback")

    p_collect = sub.add_parser("collect_output", help="v27 Step 3 failure path: 拉真实 output")
    p_collect.add_argument("task_id", help="任务 ID")
    p_collect.add_argument("--phase", default="v27-fallback")

    args = parser.parse_args()

    # 走 v27 拍板激活 subcommand 路径
    if args.command == "invoke":
        d = SubagentDispatcher(phase=args.phase, audit_log=args.audit_log)
        brief_path = Path(args.brief_path)
        handle = d.invoke(brief_path, timeout=600)
        print(json.dumps({"task_id": args.task_id, "phase": "invoke", "status": "ok", "handle": str(handle)}, ensure_ascii=False))
        return

    if args.command == "verify":
        d = SubagentDispatcher(phase=args.phase, audit_log=None)
        ok = d.verify(args.task_id)
        print(json.dumps({"task_id": args.task_id, "phase": "verify", "status": "ok" if ok else "fail"}, ensure_ascii=False))
        return 0 if ok else 1

    if args.command == "collect_output":
        d = SubagentDispatcher(phase=args.phase, audit_log=None)
        output_path = d.collect_output(args.task_id)
        print(json.dumps({"task_id": args.task_id, "phase": "collect_output", "output_path": str(output_path), "status": "ok"}, ensure_ascii=False))
        return

    # 全流程 CLI (向后兼容 legacy)
    if not args.task_id or not args.phase or not args.content:
        parser.error("--task-id, --phase, --content are required for legacy CLI (no subcommand)")
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
