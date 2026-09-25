"""BL-4: subagent_dispatch_audit — SubagentDispatch builtin hook (per BD §4.7 + §NFR-S-6).

SubagentDispatch 前置审计: 检查子代理 dispatch 参数, 拦截明显越权 (e.g. script_path 指向 ~/.ssh/, args 含凭据).
跟 SRS-PRE-TOOL-USE-GUARD §NFR-S-4 一致: 子代理 dispatch 前置 100% 覆盖.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from __future__ import annotations

import logging
import re
from typing import Any, Dict, List


logger = logging.getLogger(__name__)


HOOK_DEF: Dict[str, Any] = {
    "name": "subagent_dispatch_audit",
    "event_type": "SubagentDispatch",
    "action_type": "pre",
    "handler": "builtin:subagent_dispatch_audit",
    "priority": 10,  # 高优先级, 先执行
    "parallel": False,
    "timeout_ms": 500,
    "retry": 0,
    "description": "SubagentDispatch 前置审计: 拦截 script_path 指向敏感路径 (e.g. ~/.ssh/, /etc/, /boot/) + args 含 GitHub PAT/SSH 私钥等凭据. 跟 SRS-PRE-TOOL-USE-GUARD §NFR-S-4 一致.",
}


# 敏感路径 (per 守门 #5 + guardian/pre_tool_use_guard.py R-BLOCK-005)
_SENSITIVE_PATHS: List[re.Pattern] = [
    re.compile(r"(^|/)\.ssh/", re.IGNORECASE),
    re.compile(r"(^|/)etc/(passwd|shadow|sudoers)", re.IGNORECASE),
    re.compile(r"(^|/)boot/", re.IGNORECASE),
    re.compile(r"(^|/)\.aws/credentials", re.IGNORECASE),
    re.compile(r"(^|/)\.npmrc", re.IGNORECASE),
    re.compile(r"(^|/)\.netrc", re.IGNORECASE),
    re.compile(r"(^|/)\.git-credentials", re.IGNORECASE),
]


# 凭据 pattern (per 守门 #5 + R-BLOCK-007/008)
_CREDENTIAL_PATTERNS: List[re.Pattern] = [
    re.compile(r"ghp_[a-zA-Z0-9]{36,}"),  # GitHub PAT
    re.compile(r"github_pat_[a-zA-Z0-9_]{22,}"),  # GitHub fine-grained PAT
    re.compile(r"sk-[a-zA-Z0-9]{20,}"),  # OpenAI API key
    re.compile(r"xox[baprs]-[a-zA-Z0-9-]{10,}"),  # Slack token
    re.compile(r"AKIA[0-9A-Z]{16}"),  # AWS access key
    re.compile(r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----"),  # SSH/PEM private key
]


def handler(event, context):
    """SubagentDispatch builtin audit hook 入口.

    Returns:
        HookResult: decision=BLOCK 凭据命中, ASK 敏感路径命中, PASS 其他
    """
    from scripts.automation.hooks.event_emitter import HookResult

    payload = event.payload or {}
    script_path = str(payload.get("script_path", ""))
    args = payload.get("args", {})

    # 1. 凭据 pattern 扫描 (BLOCK)
    args_str = str(args) if args else ""
    for pat in _CREDENTIAL_PATTERNS:
        if pat.search(script_path) or pat.search(args_str):
            return HookResult(
                decision="BLOCK",
                reason=f"subagent_dispatch_audit: 凭据 pattern 命中 ({pat.pattern!r})",
                rule_id="subagent_dispatch_audit.credential",
                latency_ms=0.0,
            )

    # 2. 敏感路径扫描 (ASK — 推荐项必放"取消" per 守门 v28)
    for pat in _SENSITIVE_PATHS:
        if pat.search(script_path):
            return HookResult(
                decision="ASK",
                reason=f"subagent_dispatch_audit: 敏感路径命中 ({pat.pattern!r}), 需用户确认",
                rule_id="subagent_dispatch_audit.sensitive_path",
                latency_ms=0.0,
            )

    # 3. PASS
    return HookResult(
        decision="PASS",
        reason="subagent_dispatch_audit: 无凭据 / 敏感路径命中",
        latency_ms=0.0,
    )