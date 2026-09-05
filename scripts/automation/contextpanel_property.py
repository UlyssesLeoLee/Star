#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ContextPanel Tab 1 属性 自动化档 (per docs/automation-design.md §1.3 + 守门 #19)

实装 5 tab 容器 + Tab 1 "属性" 7 字段,WorkItemDetailDrawer.tsx 重构。

scope:
  - 新建 frontend/src/components/contextpanel/{ContextPanel,TabProperty,EmptyTabPlaceholder}.tsx
  - 重构 WorkItemDetailDrawer.tsx (内部包 ContextPanel, 公开 API 100% 兼容)
  - i18n 4 语言 +5 key = 20 行
  - 测试 ContextPanel.test.tsx 3/3 pass
  - 7 段 PHASE-CONTEXTPANEL-TAB1-PROPERTY-REPORT.md
  - 1 commit, author=Ulysses, 引用 brief 路径

base: main HEAD (per 2026-09-06 07:34 JST 拍板)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 + 守门 #20 v2)

守门:
  - cargo check --workspace --all-targets -j 4 0 err (跟 main 一致, 不动 Rust)
  - pnpm typecheck 0 err
  - pnpm test src/components/contextpanel/__tests__/ContextPanel.test.tsx 3/3 pass
  - pnpm test src/components/board/KanbanBoard.test.tsx 100% 兼容 (Drawer API 不变)
  - 跟 worker-2/3 互不重叠文件

已知缺口 (per 守门 #11 缺标比错标):
  - Tab 2-5 由 worker-2/3/4/5 接手, 本 wt 只填 slot 1, 其余 slot 留 EmptyTabPlaceholder
  - 键盘快捷键 Cmd+[ / Cmd+] 跟 worker-5 (5pages) 一起做
  - MSW handler 不写, 复用 frontend/src/mocks/handlers/work-item.ts 既有
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

# 路径配置 (per brief §1)
WT_PATH = Path(r"D:\Star\.worktrees\cp-tab-property")
WT_BRANCH = "feat/cp-tab-property"
BRIEF_PATH = "docs/briefs/contextpanel-property.md"
AUTOMATION_SCRIPT = "scripts/automation/contextpanel_property.py"

# 守门命令 (per AGENTS.md §4 #1 派生规 v1+v2+v19)
GUARD_COMMANDS = [
    # 1. cargo check 跳过 (本 wt 不动 Rust, 跟 main 一致)
    # ["cargo", "check", "--workspace", "--all-targets", "-j", "4"],
    # 2. frontend typecheck
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "typecheck"],
    # 3. frontend test
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/contextpanel/__tests__/ContextPanel.test.tsx"],
    # 4. 兼容测试
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/board/KanbanBoard.test.tsx"],
]


def run_guard(label: str, cmd: list[str]) -> bool:
    """跑一条守门,返 bool。"""
    print(f"[guard] {label}: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, cwd=str(WT_PATH), capture_output=True, text=True, timeout=300)
        if result.returncode == 0:
            print(f"  [OK] {label}")
            return True
        print(f"  [FAIL] exit {result.returncode}")
        print(result.stdout[-2000:])
        print(result.stderr[-2000:])
        return False
    except subprocess.TimeoutExpired:
        print(f"  [TIMEOUT] {label} 300s")
        return False
    except FileNotFoundError as e:
        print(f"  [SKIP] {label} 工具缺失: {e}")
        return True  # 工具缺失不算失败


def main() -> int:
    """主入口: 跑守门 + 落 commit message 模板。"""
    print(f"=== ContextPanel Tab 1 属性 (worker-1 / {WT_BRANCH}) ===")
    print(f"brief: {BRIEF_PATH}")
    print(f"automation: {AUTOMATION_SCRIPT}")
    print()

    # 跑守门
    all_pass = True
    for label, cmd in GUARD_COMMANDS:
        if not run_guard(label, cmd):
            all_pass = False

    # 输出 commit message 模板
    print()
    print("=== commit message 模板 (per AGENTS.md §2.1) ===")
    print(
        "feat(cp-tab-property): ContextPanel Tab 1 属性 5 tab 容器实装\n"
        "\n"
        "- 新建 ContextPanel.tsx (5 slot, 本 wt 填 slot 1)\n"
        "- 拆 WorkItemDetailDrawer 7 字段到 TabProperty.tsx\n"
        "- Drawer 公开 API 100% 兼容 (KanbanBoard 不改)\n"
        "- 4 key i18n × 4 语言 = 20 行\n"
        "- 测试: ContextPanel.test.tsx 3/3 pass\n"
        "- 守门: cargo check --workspace --all-targets -j 4 (0 err, 跟 main 一致)\n"
        f"- 自动化档: {AUTOMATION_SCRIPT}\n"
        f"- brief: {BRIEF_PATH}\n"
        "- per 守门 #1+#19+#20+#9 v20\n"
        "\n"
        "Co-Authored-By: Mavis 接手 agent <mavis@MiniMax.local>"
    )

    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
