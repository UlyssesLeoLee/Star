#!/usr/bin/env python3
"""
scripts/automation/cleanup_worktrees.py v0.1
Phase A.2 .worktrees 残留 3 项清理 (per STAR-P4-UNIMPL-WBS-001.md §2 + 2026-09-03-rf-001-blockers-4items-board.md §2.2)

Per 守门 #5 v2: Mavis 不越权 PowerShell 永久删,仅输出命令清单供 Ulysses 手动操作。
Per 守门 #1 v19: 本脚本满足 [P] 任务卡自动化档 4 维 (Rerunnable/Volume/Structural/Audit-trail)。

Usage:
  python scripts/automation/cleanup_worktrees.py --dry-run   # 输出 PowerShell 命令 (默认)
  python scripts/automation/cleanup_worktrees.py --list     # 输出残留项清单
  python scripts/automation/cleanup_worktrees.py --help     # 帮助

Author: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
Created: 2026-09-04 09:00 JST
"""
from __future__ import annotations
import argparse
import os
import sys
from pathlib import Path

# 守门 #5 v2: 强制 UTF-8 (per Windows PowerShell console codepage GBK)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

WORKTREE_ROOT = Path("D:/Star/.worktrees")

# 9/3 12:30 JST rf-001 4 阻塞项拍板 A: 永久删 (Ulysses 手动)
# per 2026-09-03-rf-001-blockers-4items-board.md §2.2 阻塞项 2
RESIDUAL_ITEMS = [
    {
        "path": WORKTREE_ROOT / "integration-e2e-openclaw.log",
        "type": "file",
        "description": "9/2 8:22 wt 调试 log, 9/2 后无引用",
        "created": "2026-09-02 08:22",
    },
    {
        "path": WORKTREE_ROOT / "wt-nav-i18n-a",
        "type": "dir",
        "description": "残留 dir, worktree 索引已清",
        "created": "deprecated worktree",
    },
    {
        "path": WORKTREE_ROOT / "wt-nav-shots-b",
        "type": "dir",
        "description": "残留 dir, worktree 索引已清",
        "created": "deprecated worktree",
    },
]

# 9/1 _archive_id_rs_bak 保留 (per 9/3 拍板, 9/1 备份, Mavis 不擅自删)
PRESERVE_ITEMS = [
    {
        "path": WORKTREE_ROOT / "_archive_id_rs_bak_20260901",
        "type": "dir",
        "description": "9/1 备份, 保留",
        "created": "2026-09-01",
    },
]


def list_items() -> None:
    """输出残留项 + 保留项清单"""
    print("=" * 70)
    print("  .worktrees 残留项清单 (per 9/3 12:30 JST 拍板 A)")
    print("=" * 70)
    print("\n[待删] 3 项 (Ulysses 手动 PowerShell):")
    for i, item in enumerate(RESIDUAL_ITEMS, 1):
        exists = "✓" if item["path"].exists() else "✗"
        print(f"  {i}. {exists} {item['path']} ({item['type']})")
        print(f"     描述: {item['description']}")
        print(f"     创建: {item['created']}")
    print("\n[保留] 1 项 (per 9/3 拍板):")
    for i, item in enumerate(PRESERVE_ITEMS, 1):
        exists = "✓" if item["path"].exists() else "✗"
        print(f"  {i}. {exists} {item['path']} ({item['type']})")
        print(f"     描述: {item['description']}")
    print("=" * 70)


def dry_run() -> None:
    """输出 PowerShell 删除命令清单 (per 守门 #5 v2, Mavis 不越权)"""
    print("# " + "=" * 65)
    print("# Ulysses 手动 PowerShell 删除命令 (per 守门 #5 v2)")
    print("# Mavis 不越权, 仅输出命令清单, 等 Ulysses 验证后手动执行")
    print("# " + "=" * 65)
    print()
    for item in RESIDUAL_ITEMS:
        item_path = str(item["path"])
        if item["type"] == "file":
            print("# " + item["description"])
            print("Remove-Item -Path '" + item_path + "' -Force")
        else:  # dir
            print("# " + item["description"])
            print("Remove-Item -Path '" + item_path + "' -Recurse -Force")
        print()
    print("# 保留 (per 9/3 拍板 A):")
    for item in PRESERVE_ITEMS:
        item_path = str(item["path"])
        print("# 保留: " + item_path + " (" + item["description"] + ")")
    print()
    print("# 验证清理结果:")
    print("git worktree list")
    root_str = str(WORKTREE_ROOT)
    print("Get-ChildItem '" + root_str + "' -Force | Select-Object Name")


def main() -> int:
    parser = argparse.ArgumentParser(
        description=".worktrees 残留项清理脚本 (Phase A.2, 守门 #5 v2 Mavis 不越权)",
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--dry-run", action="store_true", default=True,
                       help="输出 PowerShell 删除命令 (默认)")
    group.add_argument("--list", action="store_true",
                       help="输出残留项 + 保留项清单")
    args = parser.parse_args()

    if args.list:
        list_items()
    else:
        dry_run()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
