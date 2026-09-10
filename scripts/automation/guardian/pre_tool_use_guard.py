"""PreToolUseGuard PM-1 + PM-3: Pattern Matcher + Decision Router + 主入口.

Per DD-PRE-TOOL-USE-GUARD-001.md §2.1 + SRS-PRE-TOOL-USE-GUARD-001.md §4 FR-1~FR-3.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import hashlib
import json
import logging
import os
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional

from .rule_database import Rule, RuleDatabase
from .audit_logger import AuditEvent, AuditLogError, AuditLogger

logger = logging.getLogger(__name__)


class Decision(str, Enum):
    """三级决策 + PASS (per FR-2.1/2.2/2.3 + SRS 已知缺口 #4 priority)."""
    BLOCK = "BLOCK"
    ASK = "ASK"
    WARN = "WARN"
    PASS = "PASS"

    @property
    def priority(self) -> int:
        """BLOCK > ASK > WARN > PASS (per DD §2.1 DecisionRouter)."""
        return {"BLOCK": 4, "ASK": 3, "WARN": 2, "PASS": 1}[self.value]


@dataclass(frozen=True)
class ToolCall:
    """PreToolUse hook 入参 (per DD §2.1)."""
    tool_name: str          # "bash" / "write" / "edit" / "read" / "mcp_*"
    tool_args: Dict[str, Any] = field(default_factory=dict)
    session_id: str = ""
    agent_role: str = "orchestrator"


@dataclass(frozen=True)
class Match:
    """单条规则匹配结果."""
    rule_id: str
    level: Decision
    reason: str
    latency_us: int         # microseconds


class PatternMatcher:
    """PM-1: pure function, 给定 tool_call 返回 Match[].

    Per DD §2.1 + §4.1:
    - 提取 scan_target (跨平台)
    - 跑所有规则, 命中返回 Match
    - 延迟 p50 < 5ms, p99 < 10ms (per NFR-P-1/NFR-P-2)
    """

    @staticmethod
    def extract_scan_target(tool_call: ToolCall) -> str:
        """从 tool_args 提取可扫描字符串 (per DD §2.1)."""
        name = tool_call.tool_name
        args = tool_call.tool_args or {}
        if name == "bash":
            return str(args.get("command", ""))
        if name in ("write", "edit"):
            path = str(args.get("path", ""))
            content = str(args.get("content", "") or args.get("new_string", ""))
            # 跨平台 path normalize (per §4.1)
            path_n = path.replace("\\", "/")
            content_n = content[:500]
            return f"{path_n}\n{content_n}"
        if name == "read":
            return ""  # per FR-1.3, read 不掃
        # MCP / 未知 tool
        try:
            return json.dumps(args, ensure_ascii=False)
        except (TypeError, ValueError):
            return str(args)

    @classmethod
    def match_all(
        cls,
        tool_call: ToolCall,
        rules: List[Rule],
    ) -> List[Match]:
        """跑所有规则, 返回 Match[]."""
        target = cls.extract_scan_target(tool_call)
        if not target:
            return []
        matches: List[Match] = []
        for rule in rules:
            t0 = time.perf_counter_ns()
            hit = rule.match(target)
            latency_us = (time.perf_counter_ns() - t0) // 1000
            if hit:
                matches.append(Match(
                    rule_id=rule.id,
                    level=Decision(rule.level),
                    reason=rule.reason,
                    latency_us=latency_us,
                ))
        return matches


class DecisionRouter:
    """PM-3: 多 Match 冲突时, 按优先级选 1 个 (per DD §2.1)."""

    @staticmethod
    def route(matches: List[Match]) -> Decision:
        if not matches:
            return Decision.PASS
        return max(matches, key=lambda m: m.level.priority).level


def _hash_tool_args(tool_args: Dict[str, Any]) -> str:
    """tool_args 完整 sha256 (脱敏, per FR-4.1 + 守门 #5)."""
    try:
        s = json.dumps(tool_args, sort_keys=True, ensure_ascii=False, default=str)
    except (TypeError, ValueError):
        s = str(tool_args)
    return "sha256:" + hashlib.sha256(s.encode("utf-8", errors="replace")).hexdigest()


def _hash_env_keys() -> str:
    """env 键的 sha256 (不写实际值, per 守门 #5)."""
    keys = sorted(os.environ.keys())
    return "sha256:" + hashlib.sha256(",".join(keys).encode("utf-8")).hexdigest()


class PreToolUseGuard:
    """主入口, 整合 PM-1 + PM-3 + audit (per DD §2.1)."""

    def __init__(self, rule_db: RuleDatabase, audit: AuditLogger):
        self._rules = rule_db
        self._audit = audit

    def evaluate(self, tool_call: ToolCall) -> Decision:
        """PreToolUse hook 入口 (per FR-1.1 + NFR-P-1/NFR-P-2).

        延迟 p50 < 5ms, p99 < 10ms.
        規則加載失敗 → fail-open (PASS + WARN log).
        audit 寫失敗 → fail-closed (BLOCK, per FR-6.2).
        """
        t0 = time.perf_counter()
        decision: Decision = Decision.PASS
        rule_id: Optional[str] = None
        reason: Optional[str] = None

        try:
            rules = self._rules.get_rules()
            matches = PatternMatcher.match_all(tool_call, rules)
            decision = DecisionRouter.route(matches)
            if matches:
                rule_id = matches[0].rule_id
                reason = matches[0].reason
        except Exception as e:
            # per FR-6.1: fail-open
            logger.warning("Rule evaluation failed, fail-open: %s", e)
            decision = Decision.PASS
            reason = f"fail-open: {type(e).__name__}: {e}"

        latency_ms = (time.perf_counter() - t0) * 1000.0

        # audit log 必寫 (per FR-4.1)
        scan_target = PatternMatcher.extract_scan_target(tool_call)
        event = AuditEvent(
            ts=time.strftime("%Y-%m-%dT%H:%M:%S") + f".{int((time.time()%1)*1000):03d}+09:00",
            session_id=tool_call.session_id or "unknown",
            agent_role=tool_call.agent_role,
            tool=tool_call.tool_name,
            tool_args_hash=_hash_tool_args(tool_call.tool_args or {}),
            args_excerpt=scan_target[:200],
            decision=decision.value,
            rule_id=rule_id,
            reason=reason,
            latency_ms=round(latency_ms, 3),
            env_hash=_hash_env_keys(),
        )
        try:
            self._audit.log(event)
        except AuditLogError as e:
            # per FR-6.2: fail-closed
            logger.error("Audit log write failed, fail-closed: %s", e)
            return Decision.BLOCK

        return decision

    def evaluate_for_dispatch(
        self,
        task_id: str,
        script_path: str,
        args: Dict[str, Any],
        parent_session_id: str,
    ) -> Decision:
        """子代理 dispatch 前置入口 (per FR-1.2 + NFR-S-4)."""
        tool_call = ToolCall(
            tool_name="mcp_dispatch",
            tool_args={
                "task_id": task_id,
                "script_path": script_path,
                "args": args,
            },
            session_id=parent_session_id,
            agent_role="orchestrator",
        )
        return self.evaluate(tool_call)
