#!/usr/bin/env python3
"""
nav_completion_i18n.py — 5 module categoryLabel 同步 + 1 new remote entry (per 2026-09-02 18:31 JST brief)

目的: commit 826bc37 改了 6 module 的 category 字段 (core → work/agent/system),
但 i18n 字典 (zh-CN.ts / en.ts / ja.ts) 仍是旧 categoryLabel "核心工作区" / "Core Workspace"
/ "コアワークスペース". AppMatrix 抽屉里显示误导.

修复: 改 3 个 i18n 文件 (UTF-8 + CRLF, 不是 GBK — brief 错, 实际是 UTF-8):
  5 module 改 categoryLabel (inbox 保持) + 1 new remote entry (system 域)

使用:
  python scripts/automation/nav_completion_i18n.py [--dry-run] [--root <path>]

输出: stdout 报告 + 写回 3 个 i18n 文件 (UTF-8 + CRLF 保真)
"""
import argparse
import os
import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict


# ── 修改映射 (per 2026-09-02 18:31 JST brief) ──────────────────────
# 注意: 实际 i18n 文件是 UTF-8 (不是 GBK), 有 CRLF 行尾 (300+ CRLF in zh-CN.ts)
# 完整行模式 (含 description + categoryLabel) 用于精确匹配替换

REPLACEMENTS_ZH: List[Tuple[str, str]] = [
    # issues: 核心工作区 → 工作管理
    ('issues: { label: "Issues", description: "工作项与任务看板 / 树形全景视图", categoryLabel: "核心工作区" }',
     'issues: { label: "Issues", description: "工作项与任务看板 / 树形全景视图", categoryLabel: "工作管理" }'),
    # projects: 核心工作区 → 工作管理
    ('projects: { label: "Projects", description: "多面板项目工作区 (Kanban / Timeline / Backlog / Agents / Worktrees)", categoryLabel: "核心工作区" }',
     'projects: { label: "Projects", description: "多面板项目工作区 (Kanban / Timeline / Backlog / Agents / Worktrees)", categoryLabel: "工作管理" }'),
    # agents: 核心工作区 → Agent 编排
    ('agents: { label: "Agents", description: "智能 Agent 运行状态、编排、会话与执行日志", categoryLabel: "核心工作区" }',
     'agents: { label: "Agents", description: "智能 Agent 运行状态、编排、会话与执行日志", categoryLabel: "Agent 编排" }'),
    # analytics: 核心工作区 → 工作管理
    ('analytics: { label: "Analytics", description: "工程效能大盘、燃尽图与遥测指标统计", categoryLabel: "核心工作区" }',
     'analytics: { label: "Analytics", description: "工程效能大盘、燃尽图与遥测指标统计", categoryLabel: "工作管理" }'),
    # settings: 核心工作区 → 系统管理
    ('settings: { label: "Settings", description: "租户全局配置、团队成员、安全与权限管理", categoryLabel: "核心工作区" }',
     'settings: { label: "Settings", description: "租户全局配置、团队成员、安全与权限管理", categoryLabel: "系统管理" }'),
]

REPLACEMENTS_EN: List[Tuple[str, str]] = [
    ('issues: { label: "Issues", description: "Work items and task board / tree overview", categoryLabel: "Core Workspace" }',
     'issues: { label: "Issues", description: "Work items and task board / tree overview", categoryLabel: "Work Management" }'),
    ('projects: { label: "Projects", description: "Multi-panel project workspace (Kanban / Timeline / Backlog / Agents / Worktrees)", categoryLabel: "Core Workspace" }',
     'projects: { label: "Projects", description: "Multi-panel project workspace (Kanban / Timeline / Backlog / Agents / Worktrees)", categoryLabel: "Work Management" }'),
    ('agents: { label: "Agents", description: "Agent runtime status, orchestration, sessions and execution logs", categoryLabel: "Core Workspace" }',
     'agents: { label: "Agents", description: "Agent runtime status, orchestration, sessions and execution logs", categoryLabel: "Agent Orchestration" }'),
    ('analytics: { label: "Analytics", description: "Engineering effectiveness dashboard, burndown and telemetry metrics", categoryLabel: "Core Workspace" }',
     'analytics: { label: "Analytics", description: "Engineering effectiveness dashboard, burndown and telemetry metrics", categoryLabel: "Work Management" }'),
    ('settings: { label: "Settings", description: "Tenant global config, team members, security and permissions", categoryLabel: "Core Workspace" }',
     'settings: { label: "Settings", description: "Tenant global config, team members, security and permissions", categoryLabel: "System Admin" }'),
]

REPLACEMENTS_JA: List[Tuple[str, str]] = [
    ('issues: { label: "課題", description: "作業項目とタスクボード / ツリー概要", categoryLabel: "コアワークスペース" }',
     'issues: { label: "課題", description: "作業項目とタスクボード / ツリー概要", categoryLabel: "作業管理" }'),
    ('projects: { label: "プロジェクト", description: "マルチパネルプロジェクトワークスペース (Kanban / Timeline / Backlog / Agents / Worktrees)", categoryLabel: "コアワークスペース" }',
     'projects: { label: "プロジェクト", description: "マルチパネルプロジェクトワークスペース (Kanban / Timeline / Backlog / Agents / Worktrees)", categoryLabel: "作業管理" }'),
    ('agents: { label: "エージェント", description: "エージェント実行状態、編成、セッション、実行ログ", categoryLabel: "コアワークスペース" }',
     'agents: { label: "エージェント", description: "エージェント実行状態、編成、セッション、実行ログ", categoryLabel: "Agent 编排" }'),
    ('analytics: { label: "分析", description: "エンジニアリング効果ダッシュボード、バーンダウン、テレメトリ指標", categoryLabel: "コアワークスペース" }',
     'analytics: { label: "分析", description: "エンジニアリング効果ダッシュボード、バーンダウン、テレメトリ指標", categoryLabel: "作業管理" }'),
    ('settings: { label: "設定", description: "テナントグローバル設定、メンバー、セキュリティ、権限", categoryLabel: "コアワークスペース" }',
     'settings: { label: "設定", description: "テナントグローバル設定、メンバー、セキュリティ、権限", categoryLabel: "システム管理" }'),
]


# ── remote 新加 entry (per 2026-09-02 18:31 JST brief + commit 826bc37) ──
# registry.ts 有 `remote: { id: "remote", label: "Remote Control", category: "system", ... }`
# 但 i18n 没加, 走 kanban 之前插入新 entry

REMOTE_ENTRY_ZH = '    remote: { label: "Remote Control", description: "手机端远程连接 desktop / terminal / files (per 2026-09-01 PHASE-MOBILE-PWA v0.2)", categoryLabel: "系统管理" },'
REMOTE_ENTRY_EN = '    remote: { label: "Remote Control", description: "Mobile remote connection to desktop / terminal / files (per 2026-09-01 PHASE-MOBILE-PWA v0.2)", categoryLabel: "System Admin" },'
REMOTE_ENTRY_JA = '    remote: { label: "リモート操作", description: "モバイルから desktop / terminal / files にリモート接続 (per 2026-09-01 PHASE-MOBILE-PWA v0.2)", categoryLabel: "システム管理" },'

# 锚点: kanban 行 (插入 remote 在 kanban 之前, 保持 "core 组" 末尾, "work 组" 开头的视觉次序)
REMOTE_ANCHOR_ZH = '    kanban: { label: "Kanban Board", description: "4 态泳道即时拖拽任务看板"'
REMOTE_ANCHOR_EN = '    kanban: { label: "Kanban Board", description: "4-state swimlane with real-time drag and drop"'
REMOTE_ANCHOR_JA = '    kanban: { label: "かんばんボード", description: "4 状態スイムレーン、リアルタイムドラッグ&ドロップ"'


def process_file(path: Path, replacements: List[Tuple[str, str]], remote_entry: str, remote_anchor: str, lang: str, dry_run: bool) -> Dict:
    """处理单个 i18n 文件: 5 处 categoryLabel 替换 + 1 处 remote entry 插入"""
    # 读 UTF-8 bytes (避免解码错误)
    with open(path, "rb") as f:
        original_bytes = f.read()
    original_text = original_bytes.decode("utf-8")
    new_text = original_text
    stats = {"replaced": 0, "remote_inserted": False, "errors": []}

    # Step 1: 5 处 categoryLabel 替换
    for old, repl in replacements:
        if old in new_text:
            new_text = new_text.replace(old, repl, 1)
            stats["replaced"] += 1
        else:
            stats["errors"].append(f"未找到: {old[:80]}...")

    # Step 2: 插入 remote entry (锚点 = kanban 行)
    if remote_anchor in new_text:
        # 找到锚点行, 前面插入 remote entry (保留锚点行原缩进, 不加额外空格)
        # 原始: "    kanban: {..." (4 spaces + content)
        # 期望: "    remote: {...}\r\n    kanban: {..." (4 + content each)
        new_text = new_text.replace(
            remote_anchor,
            remote_entry + "\r\n" + remote_anchor,
            1,
        )
        stats["remote_inserted"] = True
    else:
        stats["errors"].append(f"未找到 remote 锚点: {remote_anchor}")

    if not dry_run and new_text != original_text:
        new_bytes = new_text.encode("utf-8")
        # 写回 UTF-8
        with open(path, "wb") as f:
            f.write(new_bytes)
        # 验证 UTF-8 编码保真
        with open(path, "rb") as f:
            verified = f.read()
        if verified != new_bytes:
            stats["errors"].append("UTF-8 写回验证失败: 字节不一致")
        else:
            stats["orig_bytes"] = len(original_bytes)
            stats["new_bytes"] = len(new_bytes)
            stats["delta_bytes"] = len(new_bytes) - len(original_bytes)
            stats["orig_crlf"] = original_bytes.count(b"\r\n")
            stats["new_crlf"] = new_bytes.count(b"\r\n")
            # 字节级 diff 校验: 应该只有 5 处 categoryLabel 字符串变化 + 1 处 remote entry 插入
            # 简单做: 比 orig 和 new 的非空白字节差异数
            stats["delta_string_replacements"] = sum(1 for o, n in replacements if o != n and o in original_text) + (1 if stats["remote_inserted"] else 0)

    return stats


def main():
    parser = argparse.ArgumentParser(description="i18n categoryLabel 同步 (UTF-8 + CRLF 保真)")
    parser.add_argument("--dry-run", action="store_true", help="只统计不写回")
    parser.add_argument("--root", default=r"D:\Star", help="Star 仓根目录")
    args = parser.parse_args()

    root = Path(args.root)
    i18n_dir = root / "frontend" / "src" / "lib" / "i18n"

    files = [
        (i18n_dir / "zh-CN.ts", REPLACEMENTS_ZH, REMOTE_ENTRY_ZH, REMOTE_ANCHOR_ZH, "zh-CN"),
        (i18n_dir / "en.ts", REPLACEMENTS_EN, REMOTE_ENTRY_EN, REMOTE_ANCHOR_EN, "en"),
        (i18n_dir / "ja.ts", REPLACEMENTS_JA, REMOTE_ENTRY_JA, REMOTE_ANCHOR_JA, "ja"),
    ]

    print(f"=== nav_completion_i18n.py ===")
    print(f"Root: {root}")
    print(f"Mode: {'DRY-RUN' if args.dry_run else 'WRITE'}")
    print()

    total_replaced = 0
    total_remote = 0
    total_errors = 0
    for path, repls, remote_entry, remote_anchor, lang in files:
        if not path.exists():
            print(f"[{lang}] X 文件不存在: {path}")
            total_errors += 1
            continue
        stats = process_file(path, repls, remote_entry, remote_anchor, lang, args.dry_run)
        print(f"[{lang}] {path.name}:")
        print(f"  categoryLabel 替换: {stats['replaced']}/5")
        print(f"  remote 插入: {stats['remote_inserted']}")
        if "delta_bytes" in stats:
            print(f"  字节差: {stats['delta_bytes']:+d} (orig={stats['orig_bytes']}, new={stats['new_bytes']})")
            print(f"  CRLF: orig={stats['orig_crlf']}, new={stats['new_crlf']}")
        if stats["errors"]:
            print(f"  错误 ({len(stats['errors'])}):")
            for e in stats["errors"]:
                print(f"    - {e}")
        total_replaced += stats["replaced"]
        total_remote += 1 if stats["remote_inserted"] else 0
        total_errors += len(stats["errors"])
        print()

    print(f"=== 汇总 ===")
    print(f"  categoryLabel 替换: {total_replaced}/15 (5 mod x 3 lang)")
    print(f"  remote 插入: {total_remote}/3 (1 entry x 3 lang)")
    print(f"  错误: {total_errors}")
    return 0 if total_errors == 0 and total_replaced == 15 and total_remote == 3 else 1


if __name__ == "__main__":
    sys.exit(main())
