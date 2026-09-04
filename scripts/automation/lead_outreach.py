#!/usr/bin/env python3
"""
scripts/automation/lead_outreach.py v0.1
Phase A.3 5 域 Lead 真人寻访流程 (per STAR-P4-UNIMPL-WBS-001.md §2 + STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md §1)

Per 守门 #3: 5 域独立 Lead, 不接受兼任 (per 8/21 JST Ulysses 拍板硬约束)。
Per 守门 #3 v2: Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转), 真人到位后追溯签字。
Per 守门 #1 v19: 本脚本满足 [P] 任务卡自动化档 4 维 (Rerunnable/Volume/Structural/Audit-trail)。

Usage:
  python scripts/automation/lead_outreach.py --list     # 输出 15 候选清单 (5 域 × 3 寻访方法)
  python scripts/automation/lead_outreach.py --check    # 检查 5 域 Lead 真人到位状态
  python scripts/automation/lead_outreach.py --help

Author: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
Created: 2026-09-04 09:00 JST
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from pathlib import Path

# 守门 #5 v2: 强制 UTF-8 (per Windows PowerShell console codepage GBK)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

# 5 域 = 守门 #3 历史治理命名 (5 位真人 Lead 问责结构)
# per AGENTS.md §5 仓库拓扑 disclaimer: 5 域不引用 RGS 仓, Star 仓独立业务子域
# per STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md §1 步骤 2: REGISTRY 5 行
DOMAINS = [
    {
        "id": "player",
        "scope": "用户/identity/workspace",
        "bc_doc": "docs/ddd/01-player-bc.md",
        "lead_name": "<待填>",
        "lead_email": "<待填>",
        "role": "Player Lead",
        "onboard_date": "<YYYY-MM-DD>",
        "status": "🟡 待到岗",
    },
    {
        "id": "economy",
        "scope": "billing/pricing/cost",
        "bc_doc": "docs/ddd/02-economy-bc.md",
        "lead_name": "<待填>",
        "lead_email": "<待填>",
        "role": "Economy Lead",
        "onboard_date": "<YYYY-MM-DD>",
        "status": "🟡 待到岗",
    },
    {
        "id": "match",
        "scope": "workflow/状态机/saga",
        "bc_doc": "docs/ddd/03-match-bc.md",
        "lead_name": "<待填>",
        "lead_email": "<待填>",
        "role": "Match Lead",
        "onboard_date": "<YYYY-MM-DD>",
        "status": "🟡 待到岗",
    },
    {
        "id": "social",
        "scope": "collaboration/通知",
        "bc_doc": "docs/ddd/04-social-bc.md",
        "lead_name": "<待填>",
        "lead_email": "<待填>",
        "role": "Social Lead",
        "onboard_date": "<YYYY-MM-DD>",
        "status": "🟡 待到岗",
    },
    {
        "id": "admin",
        "scope": "RBAC/permission/tenant",
        "bc_doc": "docs/ddd/05-admin-bc.md",
        "lead_name": "<待填>",
        "lead_email": "<待填>",
        "role": "Admin Lead",
        "onboard_date": "<YYYY-MM-DD>",
        "status": "🟡 待到岗",
    },
]

# 3 寻访方法 (per CONTENT-REVIEW-PACK §1 步骤 1)
OUTREACH_METHODS = [
    {
        "id": "A",
        "name": "Ulysses 个人网络",
        "description": "5 工程师各认领 1 域, 签署 DDD Review 协议 (推荐)",
        "expected_lead_time": "1-2 周",
    },
    {
        "id": "B",
        "name": "freelance 平台",
        "description": "Toptal / Upwork 找 5 个 Rust 工程师",
        "expected_lead_time": "2-4 周",
    },
    {
        "id": "C",
        "name": "开源社区招募",
        "description": "GitHub / Rust 社区发帖招募 5 域 Lead",
        "expected_lead_time": "3-6 周",
    },
]


def list_candidates() -> None:
    """输出 5 域 × 3 寻访方法 = 15 候选清单"""
    print("=" * 70)
    print("  5 域 Lead 真人寻访 候选清单 (per 守门 #3 + 8/21 硬约束)")
    print("=" * 70)
    print("\n  寻访方法 × 3:")
    for m in OUTREACH_METHODS:
        print("    [" + m["id"] + "] " + m["name"] + ": " + m["description"] + " (预计 " + m["expected_lead_time"] + ")")
    print("\n  域 × 5 (per 守门 #3 + 5 域独立 Lead, 不接受兼任):")
    print()
    for d in DOMAINS:
        print("  * " + d["id"] + " (scope: " + d["scope"] + ")")
        print("    BC doc: " + d["bc_doc"])
        print("    角色: " + d["role"])
        print("    状态: " + d["status"] + " (lead: " + d["lead_name"] + " / " + d["lead_email"] + ")")
        print()
    print("  合计: 5 域 × 3 寻访方法 = 15 候选")
    print("  待 Ulysses 拍板寻访方法 + 启动真人到位流程")
    print("=" * 70)


def check_status() -> None:
    """检查 5 域 Lead 真人到位状态 (per REGISTRY 5 行)"""
    print("=" * 70)
    print("  5 域 Lead 真人到位状态检查 (per 守门 #3 v2 Mavis 临时代签)")
    print("=" * 70)
    total_pending = 0
    total_filled = 0
    for d in DOMAINS:
        is_filled = (d["lead_name"] != "<待填>" and d["lead_email"] != "<待填>")
        marker = "[OK]" if is_filled else "[PENDING]"
        status_text = "[已到位]" if is_filled else "[待到岗] Mavis 临时代签 per 守门 #3 v2"
        print("  " + marker + " " + d["id"] + ": " + status_text)
        if is_filled:
            total_filled += 1
        else:
            total_pending += 1
    print()
    print("  统计: " + str(total_filled) + "/5 已到位, " + str(total_pending) + "/5 待到岗")
    print()
    if total_pending > 0:
        print("  阻塞项:")
        print("    - P3-C C.9 (5 域 Lead 真人到位)")
        print("    - P3-E E.5 (5 域 Lead 真人到位 DDD Review)")
        print("    - P3-F F.1 (5 域 Lead 真人到位 DDD Review)")
        print("    - P3-E E.6 (5 域 Saga 跨域编排, 等 match 域 Lead)")
        print("    - P3-E E.7 (5 域 DDD 边界验证, 等 5 真人到位)")
    print("=" * 70)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="5 域 Lead 真人寻访流程 (Phase A.3, 守门 #3 8/21 硬约束)",
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--list", action="store_true",
                       help="输出 15 候选清单 (5 域 × 3 寻访方法)")
    group.add_argument("--check", action="store_true",
                       help="检查 5 域 Lead 真人到位状态")
    args = parser.parse_args()

    if args.check:
        check_status()
    elif args.list:
        list_candidates()
    else:
        # 默认输出两者
        list_candidates()
        print()
        check_status()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
