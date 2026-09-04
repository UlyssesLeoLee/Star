"""tests/conftest.py — pytest 路径配置

让 `python -m pytest tests/...` 能找到 scripts/automation/ 模块
"""

from __future__ import annotations

import sys
from pathlib import Path

# 把 D:\Star\.worktrees\wt-tmo-04 (项目根) 加到 sys.path
PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))
