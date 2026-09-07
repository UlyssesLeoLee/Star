#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Update §3.1 and §3.2 counts in 00-CLASSIFICATION-W-T-M.md after 4 mixed tables resolved.

Before: 33 M + 47 T + 14 W + 2 M/T + 2 T/W = 100
After:  35 M + 49 T + 14 W + 0 M/T + 0 T/W = 98  (+2 mixed moved to M/T primary)
        But we still have 100 tables in total. The 2 M/T (now M) and 2 T/W (now T) move counts.

Schema impact:
- workspace: 0 M + 1 M/T = 1  ->  1 M + 0 T = 1
- planning:  1 M + 3 T = 4  ->  2 M + 2 T = 4  (roadmap M/T -> M)
- audit:     0 M + 3 T (含 T/W 1) = 3  ->  0 M + 3 T = 3  (outbox T/W -> T)
- local_runtime: 2 M + 3 T (含 T/W 1) = 5  ->  2 M + 3 T = 5  (observation T/W -> T)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")


def update_wtm_counts(path: Path) -> bool:
    """Update §3.1 and §3.2 counts after 4 mixed tables resolved."""
    text = path.read_text(encoding="utf-8")
    orig = text

    # 1) Update §3.1 count table
    # M 33 -> 35 (workspace.workspace + planning.roadmap 移到 M)
    # T 47 -> 49 (audit.audit_event_outbox + local_runtime.runtime_observation 移到 T)
    # W 14 -> 14 (不变)
    # M/T 混合 2 -> 0
    # T/W 混合 2 -> 0
    text = re.sub(
        r"\| \*\*Master \(M\)\*\* \| 33 \| 33\.0% \|",
        "| **Master (M)** | 35 | 35.0% |",
        text,
    )
    text = re.sub(
        r"\| \*\*Transaction \(T\)\*\* \| 47 \| 47\.0% \|",
        "| **Transaction (T)** | 49 | 49.0% |",
        text,
    )
    text = re.sub(
        r"\| \*\*Work \(W\)\*\* \| 14 \| 14\.0% \|",
        "| **Work (W)** | 14 | 14.0% |",
        text,
    )
    text = re.sub(
        r"\| \*\*M/T 混合\*\* \| 2 \| 2\.0% \| `workspace\.workspace` / `planning\.roadmap`",
        "| **M/T 混合** | 0 | 0.0% | (v0.3 升版: 4 已知混合表消解) | `n/a (workspace/roadmap 已消解 per OPT-ADR-28..29)` |",
        text,
    )
    text = re.sub(
        r"\| \*\*T/W 混合\*\* \| 2 \| 2\.0% \| `audit\.audit_event_outbox` / `local_runtime\.runtime_observation`",
        "| **T/W 混合** | 0 | 0.0% | (v0.3 升版: 4 已知混合表消解) | `n/a (outbox/observation 已消解 per OPT-ADR-30..31)` |",
        text,
    )

    # 2) Update §3.2 schema-specific counts
    # workspace: 0 M + 1 M/T -> 1 M + 0
    text = re.sub(
        r"\| 2 \| workspace \| 0 \| 1 \(M/T\) \| 0 \| 1 \| workspace は M/T \|",
        "| 2 | workspace | 1 | 0 | 0 | 1 | workspace は M (v0.3 M/T -> M, per OPT-ADR-28) |",
        text,
    )
    # planning: 1 M + 3 T -> 2 M + 2 T (roadmap M/T -> M)
    text = re.sub(
        r"\| 7 \| planning \| 1 \| 3 \| 0 \| 4 \| state = M,rest = T \|",
        "| 7 | planning | 2 | 2 | 0 | 4 | state/roadmap = M,rest = T (v0.3 roadmap M/T -> M, per OPT-ADR-29) |",
        text,
    )
    # audit: 0 M + 3 T (含 T/W 1) -> 0 M + 3 T
    text = re.sub(
        r"\| 11 \| audit \| 0 \| 3 \(含 T/W 1\) \| 0 \| 3 \| event = T,outbox = T/W \|",
        "| 11 | audit | 0 | 3 | 0 | 3 | event/outbox = T (v0.3 outbox T/W -> T, per OPT-ADR-30) |",
        text,
    )
    # local_runtime: 2 M + 3 T (含 T/W 1) -> 2 M + 3 T
    text = re.sub(
        r"\| 25 \| local_runtime \| 2 \| 3 \(含 T/W 1\) \| 0 \| 5 \| runtime/status = M,command/reconciliation = T,observation = T/W \|",
        "| 25 | local_runtime | 2 | 3 | 0 | 5 | runtime/status = M,command/reconciliation/observation = T (v0.3 observation T/W -> T, per OPT-ADR-31) |",
        text,
    )

    # 3) Update the summary line at the bottom of §3.2
    text = re.sub(
        r"\| \*\*計\*\* \| \*\*25\*\* \| \*\*33\*\* \| \*\*47\*\* \| \*\*14\*\* \| \*\*94\*\* \| 重複計上なし,混合 6 件は主分類計上 \|",
        "| **計** | **25** | **35** | **49** | **14** | **98** | 重複計上なし, v0.3 升版: 4 混合表消解 (2 M/T -> M, 2 T/W -> T), 0 混合 |",
        text,
    )

    # 4) Update 集計ルール comment
    text = re.sub(
        r"\*\*集計ルール\*\*: 混合分類（M/T / T/W）は主分類で 1 回計上。`M/T` は M 寄りだが業務事実側面も持つものを M 主分類、`T/W` は T 主分類で計上。",
        (
            "**集計ルール (v0.3 升版)**: 4 已知 混合表 (workspace.workspace / planning.roadmap / audit.audit_event_outbox / local_runtime.runtime_observation) 全部消解, 0 混合。\n"
            "v0.2 时期 4 已知 混合表 全部走主分类:\n"
            "- `M/T` (workspace.workspace, planning.roadmap) -> M 主分类 (per守门 #13 (c) Master 判定基準)\n"
            "- `T/W` (audit.audit_event_outbox, local_runtime.runtime_observation) -> T 主分类 (per守门 #13 (b) Transaction 判定基準, 短TTL 走 retention_period_days 列)\n"
            "v0.2 §9 推测 2 混合 (`ほか`) 待 P3-B SRE Lead 拍板 (per OPT-A3 §3.5 + §7 #14 P1)"
        ),
        text,
    )

    # 5) Update 整合確認 line
    text = re.sub(
        r"種別集計（§26 INVENTORY）= 100 件、業務分類集計 = 33 \+ 47 \+ 14 = 94 件 \+ 混合 6 件 = 100 件、合致。",
        "種別集計 (v0.2 §26 INVENTORY) = 100 件, 業務分類集計 (v0.3) = 35 M + 49 T + 14 W = 98 件 + 0 已知 混合 = 98 件; v0.2 §9 推测 2 混合 (`ほか`) 待 P3-B SRE Lead 拍板, 拍板后 100 件完整。",
        text,
    )

    # 6) Update 3.3 cross table M count
    text = re.sub(
        r"\| Entity \(E\) \| 22 \| 25 \| 2 \| 49 \|",
        "| Entity (E) | 24 | 25 | 2 | 51 |",
        text,
    )
    # 3.3 集計 should be 100 not 102
    text = re.sub(
        r"\| \*\*計\*\* \| \*\*38\*\* \| \*\*51\*\* \| \*\*11\*\* \| \*\*100\*\* \|",
        "| **計** | **38** | **51** | **11** | **100** |",
        text,
    )

    if text != orig:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def main() -> int:
    base = Path.cwd()
    doc_path = base / "docs" / "data-design" / "ipa-detail" / "00-CLASSIFICATION-W-T-M.md"
    if not doc_path.exists():
        print(f"ERROR: doc not found at {doc_path}", file=sys.stderr)
        return 1

    if update_wtm_counts(doc_path):
        print("OK: 00-CLASSIFICATION-W-T-M.md counts updated (v0.3)")
    else:
        print("ERROR: no changes", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
