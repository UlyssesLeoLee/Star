#!/usr/bin/env python3
# aci_summary.py - Star mock assertions 聚合 CLI 雏形 (per ULYS-191 §4.4 brief v0.1)
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: ULYS-191 v0.2 approved + brief v0.1 §4.1.2 (Stage 1)
# 守门: #5 (no secret leak) + #7 (0 unsafe) + #10 (author=Ulysses) +
#       #11 (缺标比错标) + #14v4 (Mavis 审核) + #19v19 (Python 化) +
#       #24 (vendor 中立 = stdlib only)
#
# 用途: 对 1 个目录 (含 *.aci.json 文件) 聚合输出 critical/high/medium 统计
#       + Top N issues (按 severity 排序) 给 LLM agent 一行指令读懂 mock 结果.
#
# 雏形 MVP (per brief §1.2): 只出基础聚合, 完整版 (含 LLM prompt 适配) 留 P1 followup
#
# 用法:
#   python3 aci_summary.py <dir> [--top N] [--json]

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any, Iterable

SEVERITY_ORDER = ("critical", "high", "medium", "low", "info")
STATUS_ORDER = ("FAIL", "WARN", "PASS", "SKIP")


def iter_assertion_files(root: Path) -> Iterable[Path]:
    """递归收集 *.json / *.aci.json 文件."""
    if not root.exists():
        return
    for p in root.rglob("*.json"):
        if p.is_file():
            yield p
    for p in root.rglob("*.aci.json"):
        if p.is_file():
            yield p


def extract_aci_assertion(path: Path) -> list[dict[str, Any]]:
    """从 JSON 文件提取所有 aci_assertion 字段. 支持 3 种 layout:
    - {aci_assertion: {...}} (per brief 单文件)
    - {assertions: [{aci_assertion: ...}]} (未来扩展)
    - 直接 {...} (顶层即 assertion)
    """
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return []

    results: list[dict[str, Any]] = []

    def _flatten(obj: Any) -> None:
        if isinstance(obj, dict):
            if "aci_assertion" in obj and isinstance(obj["aci_assertion"], dict):
                a = dict(obj["aci_assertion"])
                a.setdefault("_source_file", str(path))
                results.append(a)
                return
            for v in obj.values():
                _flatten(v)
        elif isinstance(obj, list):
            for item in obj:
                _flatten(item)

    _flatten(data)
    return results


def aggregate(assertions: list[dict[str, Any]]) -> dict[str, Any]:
    """聚合统计: 按 severity/status 计数, 收集 critical/high/medium 的 list."""
    sev_counter: Counter[str] = Counter()
    status_counter: Counter[str] = Counter()
    layer_counter: Counter[str] = Counter()
    fail_with_high_sev: list[dict[str, Any]] = []

    for a in assertions:
        sev = a.get("severity", "unknown")
        status = a.get("status", "unknown")
        layer = a.get("layer", "unknown")
        sev_counter[sev] += 1
        status_counter[status] += 1
        layer_counter[layer] += 1
        if status in ("FAIL", "WARN") and sev in ("critical", "high", "medium"):
            fail_with_high_sev.append(a)

    # 按 severity 排序 (critical > high > medium)
    sev_rank = {s: i for i, s in enumerate(SEVERITY_ORDER)}
    status_rank = {s: i for i, s in enumerate(STATUS_ORDER)}
    fail_with_high_sev.sort(
        key=lambda a: (
            sev_rank.get(a.get("severity", ""), 99),
            status_rank.get(a.get("status", ""), 99),
            a.get("assertion_id", ""),
        )
    )

    return {
        "total": len(assertions),
        "by_severity": {s: sev_counter.get(s, 0) for s in SEVERITY_ORDER},
        "by_status": {s: status_counter.get(s, 0) for s in STATUS_ORDER},
        "by_layer": dict(layer_counter),
        "issues": fail_with_high_sev,
    }


def format_text(summary: dict[str, Any], top_n: int) -> str:
    """人类可读输出 (LLM agent 主用例)."""
    lines: list[str] = []
    lines.append("=" * 70)
    lines.append(f"ACI Summary — total: {summary['total']}")
    lines.append("=" * 70)
    lines.append("")
    lines.append("By severity:")
    for s in SEVERITY_ORDER:
        c = summary["by_severity"].get(s, 0)
        if c > 0:
            marker = "⚠" if s in ("critical", "high") else " "
            lines.append(f"  {marker} {s:10} {c}")
    lines.append("")
    lines.append("By status:")
    for s in STATUS_ORDER:
        c = summary["by_status"].get(s, 0)
        if c > 0:
            lines.append(f"  {s:10} {c}")
    if summary["by_layer"]:
        lines.append("")
        lines.append("By layer:")
        for layer, c in sorted(summary["by_layer"].items()):
            lines.append(f"  {layer:10} {c}")

    issues = summary["issues"][:top_n]
    if issues:
        lines.append("")
        lines.append(f"Top {len(issues)} issues (severity-sorted):")
        lines.append("-" * 70)
        for i, iss in enumerate(issues, 1):
            sev = iss.get("severity", "?")
            status = iss.get("status", "?")
            aid = iss.get("assertion_id", "?")
            layer = iss.get("layer", "?")
            reasoning = iss.get("reasoning", "")[:120]
            lines.append(f"  [{i}] {sev}/{status} {aid} (layer={layer})")
            lines.append(f"      {reasoning}{'...' if len(iss.get('reasoning', '')) > 120 else ''}")
            fix = iss.get("suggested_fix")
            if fix:
                lines.append(f"      fix: {fix[:100]}{'...' if len(fix) > 100 else ''}")
    else:
        lines.append("")
        lines.append("✓ No critical/high/medium FAIL/WARN issues found.")
    lines.append("=" * 70)
    return "\n".join(lines) + "\n"


def format_json(summary: dict[str, Any]) -> str:
    return json.dumps(summary, indent=2, ensure_ascii=False, sort_keys=True)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(
        prog="aci_summary.py",
        description=(
            "Aggregate ACI assertion JSON files in a directory tree "
            "(per ULYS-191 §4.4 brief v0.1). 输出 critical/high 统计 + Top issues."
        ),
    )
    p.add_argument("dir", type=Path, help="包含 .aci.json / .json 文件的根目录")
    p.add_argument("--top", type=int, default=3, help="Top N issues (默认 3)")
    p.add_argument("--json", action="store_true", help="输出 JSON (机器可读)")
    args = p.parse_args(argv)

    if not args.dir.exists():
        print(f"FAIL: 目录不存在: {args.dir}", file=sys.stderr)
        return 1
    if not args.dir.is_dir():
        print(f"FAIL: 不是目录: {args.dir}", file=sys.stderr)
        return 1

    assertions: list[dict[str, Any]] = []
    for path in iter_assertion_files(args.dir):
        assertions.extend(extract_aci_assertion(path))

    if not assertions:
        print(f"WARN: 在 {args.dir} 下未找到 aci_assertion 字段 (0 个)", file=sys.stderr)
        # 仍输出空统计
    summary = aggregate(assertions)

    if args.json:
        print(format_json(summary))
    else:
        print(format_text(summary, args.top), end="")
    return 0


if __name__ == "__main__":
    sys.exit(main())
