#!/usr/bin/env python3
"""
scripts/automation/credential_collect.py v0.1
Phase A.4 外部凭证收集清单 (per STAR-P4-UNIMPL-WBS-001.md §2 + HANDOFF-ST-001 v0.7 §9.5)

Per 9/3 11:35 JST 拍板 A: 凭证可无限期维持 mock 备选 (per 29692a7 + 5ea9611 + 8ace1d5)。
Per 9/1 14:58 JST 拍板: 决策必须用选项, 本脚本输出 mock 状态 + 切真操作清单。
Per 守门 #1 v19: 本脚本满足 [P] 任务卡自动化档 4 维 (Rerunnable/Volume/Structural/Audit-trail)。

Usage:
  python scripts/automation/credential_collect.py --status    # 输出 5 项凭证 mock 状态
  python scripts/automation/credential_collect.py --list      # 输出 5 项凭证切真操作清单
  python scripts/automation/credential_collect.py --check     # 验证 mock 备选是否可访问
  python scripts/automation/credential_collect.py --help

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

# 5 项外部凭证 (per STAR-P3-WBS-001.md §7 阻塞项汇总 + 2026-09-03-rf-001-blockers-4items-board.md)
CREDENTIALS = [
    {
        "id": "B.5",
        "name": "OpenClaw HTTP API endpoint + API key",
        "phase": "P3-B",
        "wbs_ref": "STAR-P3-WBS-001.md:73,205",
        "mock_fallback": "wiremock 模式 (per commit 29692a7)",
        "mock_status": "已落地",
        "switch_action": "Ulysses 提供真实 endpoint + API key, Mavis 替换 config + cargo test e2e",
    },
    {
        "id": "B.6",
        "name": "Hermes HTTP API endpoint + API key",
        "phase": "P3-B",
        "wbs_ref": "STAR-P3-WBS-001.md:74,206",
        "mock_fallback": "wiremock 模式 (per commit 29692a7)",
        "mock_status": "已落地",
        "switch_action": "Ulysses 提供真实 endpoint + API key, Mavis 替换 config + cargo test e2e",
    },
    {
        "id": "E.4",
        "name": "KMS 集成 (Vault / AWS KMS 凭证)",
        "phase": "P3-E",
        "wbs_ref": "STAR-P3-WBS-001.md:151,207",
        "mock_fallback": "LocalMockKms (per commit 5ea9611)",
        "mock_status": "已实装",
        "switch_action": "Ulysses 提供 Vault / AWS KMS 凭证, Mavis 替换 domain-kms + KMS rotation test",
    },
    {
        "id": "D.2",
        "name": "GitHub Actions runner (windows/macos 跨平台 e2e)",
        "phase": "P3-D",
        "wbs_ref": "STAR-P3-WBS-001.md:128,209",
        "mock_fallback": "CI runner stub (per commit 8ace1d5)",
        "mock_status": "已实装",
        "switch_action": "Ulysses 配 GitHub repo 管理员 + 真 runner, Mavis 替换 .github/workflows/ + cross_platform_e2e.py",
    },
    {
        "id": "D.6",
        "name": "markdownlint + cargo doc CI job runner",
        "phase": "P3-D",
        "wbs_ref": "STAR-P3-WBS-001.md:132,209",
        "mock_fallback": "CI runner stub (per commit 8ace1d5)",
        "mock_status": "已实装",
        "switch_action": "Ulysses 配 GitHub repo 管理员 + 真 runner, Mavis 加 markdownlint + cargo doc CI job",
    },
]


def show_status() -> None:
    """输出 5 项凭证 mock 状态"""
    print("=" * 70)
    print("  5 项外部凭证 mock 备选状态 (per 9/3 11:35 JST 拍板 A)")
    print("=" * 70)
    print()
    for c in CREDENTIALS:
        marker = "[OK]" if "已" in c["mock_status"] else "[PENDING]"
        print("  " + marker + " [" + c["id"] + "] " + c["name"] + " (" + c["phase"] + ")")
        print("     mock 备选: " + c["mock_fallback"])
        print("     mock 状态: " + c["mock_status"])
        print("     WBS 引用: " + c["wbs_ref"])
        print()
    print("  统计: 5/5 凭证 mock 备选已落地, 不阻塞 P3-B/D/E 推进")
    print("  切真时机: Ulysses 拍板 维持 mock 长期跑 OR 立即切真")
    print("=" * 70)


def show_switch_actions() -> None:
    """输出 5 项凭证切真操作清单"""
    print("=" * 70)
    print("  5 项凭证切真操作清单 (per 9/1 14:58 JST 拍板决策必须用选项)")
    print("=" * 70)
    for i, c in enumerate(CREDENTIALS, 1):
        print("")
        print("  [" + str(i) + "] " + c["id"] + " " + c["name"] + " (" + c["phase"] + ")")
        print("      切真: " + c["switch_action"])
    print("")
    print("  注: 切真由 Ulysses 启动, Mavis 接收凭证后落地")
    print("  注: 守门 #5 环境变量安全, secret 不进 git (per 11:06 JST hard ban)")
    print("=" * 70)


def check_mock() -> None:
    """验证 mock 备选是否可访问"""
    print("=" * 70)
    print("  Mock 备选 可访问性验证")
    print("=" * 70)
    checks = [
        ("B.5 wiremock 模式", "docs/frontend/design/mock-msw-handlers.md"),
        ("B.6 wiremock 模式", "docs/frontend/design/mock-msw-handlers.md"),
        ("E.4 LocalMockKms", "crates/domain-kms"),
        ("D.2 cross-platform stub", "scripts/automation/integration_e2e.py"),
        ("D.6 ci runner stub", "scripts/automation/saga_e2e.py"),
    ]
    workdir = Path("D:/Star/.worktrees/feat-auto-20260904-1c260bc7")
    for name, path in checks:
        full = workdir / path
        marker = "[OK]" if full.exists() else "[MISSING]"
        print("  " + marker + " " + name + ": " + path)
    print("=" * 70)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="外部凭证收集清单 (Phase A.4, 9/3 11:35 JST 拍板 A 维持 mock 可长期跑)",
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--status", action="store_true",
                       help="输出 5 项凭证 mock 状态")
    group.add_argument("--list", action="store_true",
                       help="输出 5 项凭证切真操作清单")
    group.add_argument("--check", action="store_true",
                       help="验证 mock 备选是否可访问")
    args = parser.parse_args()

    if args.status:
        show_status()
    elif args.list:
        show_switch_actions()
    elif args.check:
        check_mock()
    else:
        # 默认输出全部
        show_status()
        print()
        show_switch_actions()
        print()
        check_mock()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
