#!/usr/bin/env python3
"""ULYS-105.1 docs-fixup 子项 1.2 + 1.3 — UNIMPL-WBS v0.1 + OPT-WBS v0.1 SUPERSEDED banner

idempotent: 跑 2 次安全, 仅检查/追加 banner, 0 改历史 row (per 守门 #1 禁回溯叙事).

Per ULYS-105.1 sub-task 1.2 + 1.3 + 守门 #19 v19 Python 化.

Usage: python scripts/automation/mark_superseded.py [--dry-run]
"""
import argparse, sys
from pathlib import Path

WORKTREE = Path(__file__).resolve().parents[2]
TARGETS = [
    WORKTREE / "docs" / "reports" / "STAR-P4-UNIMPL-WBS-001.md",
    WORKTREE / "docs" / "reports" / "STAR-P4-OPT-WBS-001.md",
]

BANNER = """> **⚠️ SUPERSEDED per ULYS-105.1 docs-fixup 2026-09-20 07:30 JST**: 本草案 v0.1 已被后续多阶段演进覆盖 (per v0.64 反转 5 域 Lead 真人到位流程永久 obsolete + §14.19 SANDBOX-002 v0.1.3 改造 + §14.20 P3-D.6 7 任务 + 守门 v3x 9 active)。本 banner 仅作历史追溯;新基线请参考 [`STAR-P3-WBS-001.md#15`](STAR-P3-WBS-001.md) 累计统计 v0.66 row + [`STAR-ULYS-105-AGGREGATE-001.md`](STAR-ULYS-105-AGGREGATE-001.md) v0.2 (per ULYS-105.1 docs-fixup 子任务 1.4)。原始 v0.1 内容 0 改动, 仅追加此 banner (per 守门 #1 禁回溯叙事)。
>
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    changed = 0
    skipped = 0
    for path in TARGETS:
        text = path.read_text(encoding="utf-8")
        if "ULYS-105.1 docs-fixup" in text and "SUPERSEDED" in text:
            print(f"ALREADY_APPLIED: {path.name}")
            skipped += 1
            continue
        # Insert banner right after the H1 title line
        lines = text.split("\n")
        if not lines:
            print(f"ERROR: empty file {path}", file=sys.stderr)
            return 1
        # Find H1 line
        h1_idx = next((i for i, ln in enumerate(lines) if ln.startswith("# ")), 0)
        new_lines = lines[: h1_idx + 1] + [""] + BANNER.rstrip().split("\n") + lines[h1_idx + 1 :]
        new_text = "\n".join(new_lines)
        if args.dry_run:
            print(f"DRY_RUN: would insert banner into {path.name}")
            changed += 1
            continue
        path.write_text(new_text, encoding="utf-8")
        print(f"APPLIED: banner inserted into {path.name}")
        changed += 1

    print(f"SUMMARY: changed={changed}, skipped={skipped}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
