"""guardian.paths: 集中 guardian 模块路径常量 (per DD §1.1 + T3.2 接入点).

Per SRS-PRE-TOOL-USE-GUARD-001.md §6.1 (约束: 必须在 guardian/ 目录下).
Per DD-PRE-TOOL-USE-GUARD-001.md §1.1 (物理文件 6 个: __init__ + pre_tool_use_guard + rule_database + audit_logger + user_rule_api + rules/pre_tool_use_rules.json + logs/pre_tool_use_audit.log).
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class GuardianPaths:
    """guardian 模块路径常量, 跨进程/跨平台稳定."""
    guardian_dir: Path
    rules: Path
    audit_log: Path
    session_state: Path
    user_rules: Path


def _make_paths(guardian_dir: Path) -> GuardianPaths:
    return GuardianPaths(
        guardian_dir=guardian_dir,
        rules=guardian_dir / "rules" / "pre_tool_use_rules.json",
        audit_log=guardian_dir / "logs" / "pre_tool_use_audit.log",
        session_state=guardian_dir / "state" / "pre_tool_use_session_state.json",
        user_rules=guardian_dir / "rules" / "user_rules",
    )


# 默认: scripts/automation/guardian/
_DEFAULT_GUARDIAN_DIR = Path(__file__).resolve().parent
GUARDIAN_PATHS = _make_paths(_DEFAULT_GUARDIAN_DIR)
