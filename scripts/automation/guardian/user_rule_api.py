"""User-defined Rule API PM-5: v0.1 stub, v0.3 实装.

Per DD-PRE-TOOL-USE-GUARD-001.md §2.4 + guard-pre-tool-use-spec.md §2.4.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import logging
from pathlib import Path
from typing import List

logger = logging.getLogger(__name__)


def register_user_rule(rule_yaml: str, user_rules_dir: Path) -> bool:
    """v0.1 stub: 拋 NotImplementedError.

    v0.3 實裝 (per Claude hookify plugin 對照):
    - 接受 YAML / JSON
    - 寫入 user_rules_dir/{rule_id}.json
    - RuleDatabase 監聽並加載
    """
    raise NotImplementedError(
        "User-defined rule API 是 v0.3 計劃項, v0.1 僅提供 stub. "
        "見 SRS-PRE-TOOL-USE-GUARD-001.md §1.4 + 已知缺口 #4."
    )


def list_user_rules(user_rules_dir: Path) -> List[str]:
    """列出 user rules IDs (per DD §2.4 stub 行為)."""
    if not user_rules_dir.exists():
        return []
    return [p.stem for p in user_rules_dir.glob("R-*.json")]
