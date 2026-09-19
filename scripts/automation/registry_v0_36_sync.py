#!/usr/bin/env python3
"""ULYS-105.1 docs-fixup 子项 1.5 — registry v0.36 row + automation-design §4.34.6 任务卡 三方一致

idempotent: 跑 2 次安全, 仅追加 missing row/§4 task card (per 守门 #1 禁回溯叙事).

Per ULYS-105.1 sub-task 1.5 + 守门 #12 v21 [P] docs 同步必更新 WBS+registry+§4 + 守门 #19 v19 Python 化.

Usage: python scripts/automation/registry_v0_36_sync.py [--dry-run]
"""
import argparse, sys
from pathlib import Path

WORKTREE = Path(__file__).resolve().parents[2]
REGISTRY = WORKTREE / "scripts" / "automation" / "registry.md"
AUTOMATION_DESIGN = WORKTREE / "docs" / "automation-design.md"

REGISTRY_ROW_CANARY = "**v0.36** | **2026-09-20 07:30 JST** | **Ulysses"
AUTOMATION_DESIGN_CANARY = "### §4.34.6 P3-D.6 阶段 1 收官 + 阶段 2 启动重新评估 (v0.99.4 row, per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡)"


def registry_row() -> str:
    return """| **v0.36** | **2026-09-20 07:30 JST** | **Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手** | **ULYS-105.1 docs-fixup 5 子项收官索引 (per 守门 #12 v21 [P] docs 同步必更新 WBS+registry+§4)**: (1) `docs/reports/STAR-P3-WBS-001.md` §15 累计统计升版 (per 子项 1.1 + `scripts/automation/wbs_section15_sync.py`) — 合计 row 125 → 144 子项, 17 → 36 阻塞, 4 row append (P3-D.6 阶段 1 + 阶段 2 batch 2.1 + SANDBOX-002 v0.1.3 + 守门 v3x 9 active + ULYS-105.1 docs-fixup 5 子项); (2) `docs/reports/STAR-P4-UNIMPL-WBS-001.md` + `STAR-P4-OPT-WBS-001.md` 顶部 ⚠️ SUPERSEDED 标 (per 子项 1.2 + 1.3 + `scripts/automation/mark_superseded.py`); (3) `docs/reports/STAR-ULYS-105-AGGREGATE-001.md` v0.2 升版 (per 子项 1.4 + `scripts/automation/aggregate_v02.py`) — 新文件, 含 5 子项实施结果 + 9 项已知缺口; (4) `docs/automation-design.md` §4.34.6 任务卡 (per 子项 1.5) — P3-D.6 阶段 1 收官 + 阶段 2 启动重新评估, 关联 WBS v0.99.4 row; (5) **4 个 [P] 脚本 idempotent 跑 2 次安全** (per 守门 #19 v19 Python 化): `wbs_section15_sync.py` (1.1) + `mark_superseded.py` (1.2+1.3 共用) + `aggregate_v02.py` (1.4) + `registry_v0_36_sync.py` (1.5); **守门合规 11 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#10+#11+#12 v21+#14 v4+#19 v19+#28) 全部 0 违反: (a) **守门 #1 禁回溯叙事**: 0 改 WBS v0.1-v0.65 修订历史任何 row + 0 改 UNIMPL/OPT-WBS 历史内容 (仅加 banner) + 0 改 registry.md §3 任何 v0.1-v0.35 历史 row (仅 append v0.36); (b) **守门 #9 v19 Mavis 自驱**: docs-only 改动, 0 子代理 RPC 派, 0 cargo 触发; (c) **守门 #10 author=Ulysses**: commit author `Ulysses <ulysses@mavis.local>`; (d) **守门 #11 缺标比错标**: 9 项已知缺口显式列 (3 docs 同步饱和 + aggregate §3.4.1 ARG.11 obsolete + 守门 v3x 6 项状态 + §15 v0.99.7 baseline); (e) **守门 #12 v21 [P] docs 同步**: WBS+registry+§4 三方一致 (1 commit 多文件); (f) **守门 #19 v19 Python 化**: 4 个 [P] 脚本 idempotent 跑 2 次安全; (g) **守门 #28 拍板必带推荐项**: 拍板时附 1 项推荐 (per issue brief §6 拍板 #1 推荐 A); (h) **守门 #14 v4 Mavis 审核决定 author=Ulysses**: 5 角色签字栏全部 Mavis 接手**审核**; (i) **守门 #9 v20 子代理 dispatch 必先 brief 落档**: docs-only 改动 0 子代理 dispatch; (j) **守门 #1 v19 [P] docs 同步**: docs-only 改动 0 cargo 改动; (k) **守门 #1 v25 cargo test 改单 crate**: 0 cargo test 改动; **累计**: registry v0.36 = 第 36 个 [P] 自动化脚本索引, 5 子项全部 done; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | **2026-09-20 07:30 JST ULYS-105.1 docs-fixup 子任务拍板 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v21 [P] docs 同步 + 守门 #1 v15 docs 同步饱和第 102 次新事件触发仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 Python 化** |
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    reg_text = REGISTRY.read_text(encoding="utf-8")
    if REGISTRY_ROW_CANARY not in reg_text:
        # Find last |v0.XX| row in registry §3 and append after it
        marker = "\n| **v0.35** |"
        idx = reg_text.rfind(marker)
        if idx < 0:
            print(f"ERROR: could not find v0.35 anchor in registry", file=sys.stderr)
            return 1
        # Find end of that row (next \n after the row's closing |)
        end = reg_text.find("\n", idx + 1)
        # Append v0.36 row after v0.35 row
        new_reg = reg_text[: end + 1] + registry_row() + reg_text[end + 1 :]
        if args.dry_run:
            print(f"DRY_RUN: would append v0.36 row to registry.md")
        else:
            REGISTRY.write_text(new_reg, encoding="utf-8")
            print(f"APPLIED: registry.md v0.36 row appended")
    else:
        print(f"ALREADY_APPLIED: registry.md v0.36 row present")

    # automation-design.md §4.34.6 already exists (per line 2303), verify
    ad_text = AUTOMATION_DESIGN.read_text(encoding="utf-8")
    if AUTOMATION_DESIGN_CANARY in ad_text:
        print(f"VERIFIED: automation-design.md §4.34.6 task card present (per v0.99.4 row)")
    else:
        print(f"ERROR: automation-design.md §4.34.6 task card missing", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
