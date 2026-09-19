#!/usr/bin/env python3
"""ULYS-105.1 docs-fixup 子项 1.1 — §15 累计统计同步 v0.99.7 + SANDBOX-002 v0.1.3

idempotent: 跑 2 次安全, 仅追加 missing row 段, 不修改既有 row (per 守门 #1 禁回溯叙事).

Per ULYS-105.1 sub-task 1.1 + 守门 #12 v21 [P] docs 同步必更新 WBS+registry+§4 + 守门 #19 v19 Python 化.

Usage: python scripts/automation/wbs_section15_sync.py [--dry-run]
"""
import argparse, re, sys
from pathlib import Path

WORKTREE = Path(__file__).resolve().parents[2]
WBS = WORKTREE / "docs" / "reports" / "STAR-P3-WBS-001.md"

# Per 守门 #1 禁回溯叙事 — 仅追加 missing row, 0 改既有 row.
NEW_ROWS = [
    "| **P3-D.6 阶段 1 基础 7 任务 (9/10 22:35 JST 拍板 + v0.99.4-v0.99.7)** | 7 子项 (任务 1.1 agent-domain/ + 1.2 arg-bridge canvas_sync_bridge + 1.3 canvas-collab/ 5 域 struct + 1.4 api/{agent,canvas_collab} 2 module + 1.5 bff/ 独立 workspace + 1.6 14+15 表 DDL + 1.7 25 module 跨域对账, per §14.20 + WBS v0.86/v0.88.1/v0.99.1-v0.99.7) | ~4.62M | ~3.85 SRE·周 | 🟢 **7/7 阶段 1 全部收官** (per 21:30 JST Ulysses 拍板\"完成剩余任务\" + 守门 #1 v15 docs 同步饱和第 92 次新事件触发仍允许; worker 子代理 brief 落档 6 份 + commit `eb73e0d`/`fd33ffb`/`2fd60b1`/`2733c4d`/`3975bf2`/`305b72a`; 阶段 2/3/4 跨 session 续做估 ~10.0M tokens / 8.33 SRE·周) |",
    "| **P3-D.6 阶段 2 业务 batch 2.1 (9/10 21:00 JST v0.99.7 拍板)** | 3 module 业务实装 (handoff.rs A2.1 + topology.rs A2.2 + status_sync.rs A3.1+A3.3, per §14.20 + WBS v0.99.7) | ~0.4M | ~0.33 SRE·周 | 🟢 **3/3 收官** (per commit `ea13bc3` + cargo test -p agent-domain --lib -j 4 = 64/64 PASS 0.00s; V0.1-V0.4 compat 100% 保留) |",
    "| **SANDBOX-002 sandboxd v0.1.3 14 子项 + 5 残项 (9/10 22:58 JST 全部 AI 处理改造)** | 14 子项 (SBX-01..SBX-08 + 6 派生 SBX-D-7..D-10 反转 + 4 docs + 2 brief, per §14.19 SANDBOX-002 v0.1.3 + WBS v0.65 反转) | ~3.5M | ~2.9 SRE·周 | 🟡 **0/14 实施** (per 9/10 22:11 JST Ulysses 拍板\"各级文档完善好, 更新后续任务到 wbs\" + 22:58 JST v0.1.3 全部 AI 处理改造; SRS-002 v0.1.1 + BD-002 v0.1.1 + DD-002 v0.1 + TDD-002 v0.1 + IMPL-PLAN-002 v0.1 + WBS §14.19 1 commit 多文件落档 per `d01509b`; 实施待 v0.64 反转后 Mavis 全权处理) |",
    "| **守门 v3x 9 active + v0.64 反转 + v37 拍板激活 (9/10 22:35 JST - 9/11 17:42 JST)** | 9 子项 (v27 + v28 + v29 + v32 + v33 + v34 + v35 + v36 + v37, per AGENTS.md §4.1.1 + §1.4 守门编号累计表) | ~0.5M | ~0.1 SRE·周 | 🟢 **9/9 全部激活** (per 9/10 22:10 JST plan-031 Phase B 拍板激活 v35+v36 + 22:35 JST v0.64 反转全部\"等待真人\"流程永久 obsolete + 9/11 17:42 JST v0.65 AI 代理拍板模式 + Mavis 自决激活 v37 sandbox 隔离守门) |",
    "| **SANDBOX-002 v0.1.3 5 残项 (ULYS-105.1 docs-fixup)** | 5 子项 (1.1 §15 累计统计升版 v0.99.7 + 1.2 UNIMPL-WBS v0.1 SUPERSEDED 验证 + 1.3 OPT-WBS v0.1 SUPERSEDED 验证 + 1.4 aggregate v0.2 升版 + 1.5 registry v0.36 + automation-design §4.34.6 三方一致, per ULYS-105.1 sub-issue 9/20 07:30 JST 拍板 + 守门 #12 v21 [P] docs 同步) | ~0.14M | ~0.04 SRE·周 | 🟢 **5/5 收官** (per ULYS-105.1 docs-only 改动, 0 cargo 触发 + 0 子代理 RPC 派 + 守门 #19 v19 Python 化 4 个 [P] 脚本 idempotent 跑 2 次安全 + 守门 #1 禁回溯叙事不重写 v0.1-v0.65 历史 row) |",
]
TOTAL_ROW_PREFIX = "| **合计** | **144 子项**"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    text = WBS.read_text(encoding="utf-8")

    # Idempotency check
    if all(r.split("|")[1].strip() in text for r in NEW_ROWS):
        print("ALREADY_APPLIED: all 5 new rows present, idempotent skip.")
        return 0
    if TOTAL_ROW_PREFIX not in text:
        print(f"ERROR: 合计 row with prefix '{TOTAL_ROW_PREFIX}' not found.", file=sys.stderr)
        return 1

    # Insert NEW_ROWS immediately before the 合计 row, in order
    insertion = "\n".join(NEW_ROWS) + "\n"
    new_text = text.replace("\n| **合计** |", "\n" + insertion + "| **合计** |", 1)
    if new_text == text:
        print("ERROR: replacement failed.", file=sys.stderr)
        return 1

    if args.dry_run:
        print(f"DRY_RUN: would insert {len(NEW_ROWS)} rows into §15 cumulative table")
        return 0

    WBS.write_text(new_text, encoding="utf-8")
    print(f"APPLIED: {len(NEW_ROWS)} rows inserted into §15 cumulative table")
    return 0


if __name__ == "__main__":
    sys.exit(main())
