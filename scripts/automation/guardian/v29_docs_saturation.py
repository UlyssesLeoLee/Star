"""v29_docs_saturation: docs 同步饱和 40+ 次主动告警.

拍板激活后, 任何 docs 同步 commit 落地后, `automation/docs_sync_saturation.py` 自动跑计数:
- 累计 30 次 -> 终端 warning
- 累计 40 次 -> warning + ask_user 必问"维持当前饱和点 / 上调到 50 / 暂停 docs 同步 1 session"
- 累计 50 次 -> error 阻断 commit, 强制走新事件触发

计数器**仅记录 docs 同步 commit** (per 守门 #12 v21 [P] 子项 docs 同步), 不含代码 commit.

status: 待 Ulysses 拍板激活.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import argparse
import re
import subprocess
import sys
from pathlib import Path


WORKSPACE_ROOT = Path("D:/Star")
WARN_THRESHOLD = 30
ASK_THRESHOLD = 40
BLOCK_THRESHOLD = 50


def _count_docs_sync_commits() -> int:
    """Count commits in current HEAD whose message mentions docs sync (e.g. docs(wbs):)."""
    proc = subprocess.run(
        ["git", "log", "--oneline", "--grep=docs(wbs):", "--grep=docs(arg-arch):", "--grep=docs(adr):"],
        cwd=str(WORKSPACE_ROOT),
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return 0
    return len([line for line in proc.stdout.splitlines() if line.strip()])


def evaluate(current_count: int) -> tuple[int, str]:
    if current_count >= BLOCK_THRESHOLD:
        return 2, f"BLOCK: docs sync saturation {current_count} >= {BLOCK_THRESHOLD}; require new event trigger"
    if current_count >= ASK_THRESHOLD:
        return 1, f"ASK: docs sync saturation {current_count} >= {ASK_THRESHOLD}; ask user to raise threshold or pause"
    if current_count >= WARN_THRESHOLD:
        return 0, f"WARN: docs sync saturation {current_count} >= {WARN_THRESHOLD}; consider asking user soon"
    return 0, f"OK: docs sync saturation {current_count} < {WARN_THRESHOLD}"


def main() -> int:
    parser = argparse.ArgumentParser(description="v29 docs saturation threshold check")
    parser.add_argument("--count", type=int, default=None, help="override current count (for testing)")
    args = parser.parse_args()

    count = args.count if args.count is not None else _count_docs_sync_commits()
    code, msg = evaluate(count)
    print(msg)
    return code


if __name__ == "__main__":
    sys.exit(main())
