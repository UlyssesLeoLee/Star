#!/usr/bin/env python3
# _lib_validate.py — Star Mock Project 通用 fixture 验证 helper (per ULYS-140 + 守门 #19)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-140 (回归测试) 2026-09-20 JST — 替代慢 per-file grep loop (per 守门 #19 Python 化)
# 守门: #5 (no secret leak) + #11 (缺标比错标) + #19 (Python 化累积规)
#
# 用途: 给 regression-test-*.sh 调, 跑批量验证. 比 bash for-loop + grep 快 5x+ on Windows.
#
# 子命令:
#   validate-json  <dir>           — 所有 .json 解析 + 报 invalid
#   scan-secret    <dir>           — forbidden pattern 检测 (守门 #5)
#   require-field  <dir> <field>   — 每 .json 必须含 field
#   count-classes  <dir>           — 统计 W/T/M 分布 (per 守门 #13)
#   count-by-pattern <dir> <regex> — 按文件名 pattern 计数
#
# 输出: 每子命令 0 exit = OK, 1+ exit = FAIL; 行格式 `LABEL=value` 给 bash 解析.

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Iterable

FORBIDDEN_PATTERNS = (
    "password=",
    "api_key=",
    "secret=",
    "token=",
    "BEGIN PRIVATE KEY",
    "GHCR_PAT",
)


def iter_json_files(root: Path) -> Iterable[Path]:
    if not root.exists():
        return
    yield from root.rglob("*.json")


def cmd_validate_json(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    invalid: list[str] = []
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            json.loads(fp.read_text(encoding="utf-8"))
        except Exception as e:
            invalid.append(f"{fp}: {e}")
    print(f"TOTAL={total}")
    print(f"INVALID={len(invalid)}")
    for line in invalid[:20]:  # cap to 20 to avoid flood
        print(f"INVALID_FILE={line}", file=sys.stderr)
    return 0 if not invalid else 1


def cmd_scan_secret(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    leaks: list[tuple[Path, str]] = []
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            content = fp.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        lower = content.lower()
        for pat in FORBIDDEN_PATTERNS:
            if pat.lower() in lower:
                leaks.append((fp, pat))
                break
    print(f"TOTAL={total}")
    print(f"LEAKS={len(leaks)}")
    for fp, pat in leaks[:20]:
        print(f"LEAK={fp.name}: {pat}", file=sys.stderr)
    return 0 if not leaks else 1


def cmd_require_field(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    field = args.field
    missing: list[Path] = []
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            content = fp.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            missing.append(fp)
            continue
        if field not in content:
            missing.append(fp)
    print(f"TOTAL={total}")
    print(f"MISSING={len(missing)}")
    for fp in missing[:20]:
        print(f"MISSING_FIELD={fp}", file=sys.stderr)
    return 0 if not missing else 1


def cmd_count_classes(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    counts: dict[str, int] = {"Work": 0, "Transaction": 0, "Master": 0, "?": 0}
    total = 0
    for fp in iter_json_files(root):
        total += 1
        try:
            data = json.loads(fp.read_text(encoding="utf-8"))
        except Exception:
            counts["?"] += 1
            continue
        cls = data.get("class") or data.get("wtm_class") or "?"
        if cls in counts:
            counts[cls] += 1
        else:
            counts["?"] += 1
    print(f"TOTAL={total}")
    for k, v in counts.items():
        print(f"CLASS_{k.upper()}={v}")
    return 0


def cmd_count_by_pattern(args: argparse.Namespace) -> int:
    root = Path(args.dir)
    pattern = re.compile(args.regex)
    matched = 0
    total = 0
    for fp in iter_json_files(root):
        total += 1
        if pattern.search(fp.name):
            matched += 1
    print(f"TOTAL={total}")
    print(f"MATCHED={matched}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Star Mock Project fixture validator")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p1 = sub.add_parser("validate-json", help="validate JSON syntax of all fixtures")
    p1.add_argument("dir")
    p1.set_defaults(func=cmd_validate_json)

    p2 = sub.add_parser("scan-secret", help="scan forbidden secret patterns (守门 #5)")
    p2.add_argument("dir")
    p2.set_defaults(func=cmd_scan_secret)

    p3 = sub.add_parser("require-field", help="each fixture must contain a literal field")
    p3.add_argument("dir")
    p3.add_argument("field")
    p3.set_defaults(func=cmd_require_field)

    p4 = sub.add_parser("count-classes", help="count W/T/M distribution (守门 #13)")
    p4.add_argument("dir")
    p4.set_defaults(func=cmd_count_classes)

    p5 = sub.add_parser("count-by-pattern", help="count files matching regex")
    p5.add_argument("dir")
    p5.add_argument("regex")
    p5.set_defaults(func=cmd_count_by_pattern)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
