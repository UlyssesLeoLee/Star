#!/usr/bin/env python3
"""
post_merge_meta_update.py — 元 commit 任务卡 + 脚本索引 (per 守门 #21)

Per docs/automation-design.md v0.1 §4 + §6 任务卡表 + 守门 #21:
任何 [P] 子项落档后必更新 docs/automation-design.md §4 任务卡 + 
scripts/automation/registry.md 索引.

本任务: star-nav-completion-001 合并 2 worker commit (bd918e4 + 8c893a9) 后,
更新 2 文档追加 1 节 + 1 索引行.

文件编码: docs/automation-design.md / scripts/automation/registry.md 都是 GBK,
不能用 Read/Edit tool 改 (UTF-8 only), 走 Python 字节级 append.
"""
import os
import sys
from pathlib import Path

REPO = Path("D:/Star")

# === 1. docs/automation-design.md §4 追加 4.12 节 ===
design_path = REPO / "docs/automation-design.md"
design_addon = '''
\n\n### 4.12 散落 WBS 补缺口 (per 2026-09-02 18:30 JST, Ulysses 拍板开子代理和 worktree 完成它们)
\n
| 任务 | 范围 | token 预算 | 实施 | commit | 备注 |
|---|---|---|---|---|---|
| star-nav-completion-001 子任务 A (i18n categoryLabel 同步) | 7 module × 3 语言 (zh-CN/en/ja) = 21 处替换 + remote entry 新加 | 0.15M | worker 子代理 wt/star-nav-i18n-a (UTF-8 字节级 + CRLF 保真) | `bd918e4` (per git log -p --follow 实证) | brief 标 GBK 误判, 实际 UTF-8 + CRLF, worker 自识别走 Python bytes-level |
| star-nav-completion-001 子任务 B (HeaderTab 8 张视觉对比图) | light/dark × 4 active 状态 (inbox/issues/agents/settings) | 0.20M | worker 子代理 wt/star-nav-shots-b (HEADER_STATES 配置化 + dev 200s 后台) | `8c893a9` (per git log -p --follow 实证) | 8 张图全 > 16KB, dev 90s timeout 没触发 |
| star-nav-completion-001 子任务 C (其他 page SubNav 染色) | skip | 0 | 全仓 <SubNav 实测只 issues/page.tsx 1 处, 已在 f65744a 配 4 view 染色 | — | per 守门 #11 缺标比错标, mark skipped |
\n
**已知缺口 + 失败模式**: vitest pass 是必要非充分条件 (2 worktree 各跑 41 files / 345 tests pass, 但没跑 e2e); 8 张截图视觉走查是手工 byte 检查, 没真用图像 diff; main worktree 有 12 个 untracked/modified 跟别人 WIP 冲突, 合并用 stash + Move-Item 路径避开. 
'''.encode("gbk")

# === 2. scripts/automation/registry.md §1 脚本清单追加 nav_completion_i18n.py ===
registry_path = REPO / "scripts/automation/registry.md"
registry_addon = '''
| `scripts/automation/nav_completion_i18n.py` | i18n 字典 21 处 categoryLabel 字节级替换 (per star-nav-completion-001 子任务 A) | star-nav-completion-001 子任务 A | `bd918e4` | [落地] UTF-8 字节级, 7 module × 3 lang, GBK 陷阱已避 |
| `scripts/automation/post_merge_meta_update.py` | 元 commit 任务卡 + 脚本索引更新 (per 守门 #21) | star-nav-completion-001 元 commit | TBD | [落地] GBK 字节级 append |
'''.encode("gbk")

def append_gbk(path: Path, addon: bytes):
    if not path.exists():
        print(f"FAIL: {path} not found", file=sys.stderr)
        return False
    with open(path, "rb") as f:
        data = f.read()
    # 幂等检查: 已追加过则跳过 (per "守门 #11 缺标比错标")
    if b"star-nav-completion-001" in data and b"4.12" in data:
        print(f"SKIP: {path.name} already has star-nav-completion-001 marker")
        return True
    with open(path, "ab") as f:
        f.write(addon)
    print(f"OK: {path.name} appended {len(addon)} bytes (GBK)")
    return True

ok1 = append_gbk(design_path, design_addon)
ok2 = append_gbk(registry_path, registry_addon)
sys.exit(0 if (ok1 and ok2) else 1)
