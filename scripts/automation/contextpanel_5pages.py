#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ContextPanel 5 页面 2-Pane 改造 自动化档 (per docs/automation-design.md + 守门 #19) — 批 2

把 spec §1.1 拍板的 2-Pane 模式从 Kanban 卡片扩展到全 5 页面:
  - /agent (/projects?tab=agents)
  - /notification (/projects?tab=inbox)
  - /analytics
  - /planning
  - /projects Kanban (已有, 验证兼容)

scope:
  - 4 新组件: AgentDetailPanel/NotificationDetailPanel/AnalyticsDetailPanel/PlanningDetailPanel
  - 1 hook: useDetailDrawer (统一开关 + active tab 状态)
  - 集成测试: DetailDrawer.integration.test.tsx (4 页面 × 3 操作 = 12 测试)
  - 4 页面文件各 +30 行 (改)
  - 键盘快捷键: Cmd+. 全折叠 / Cmd+[ / Cmd+] 切 tab
  - i18n 4 语言 × 4 key × 4 页面 = 64 行
  - 7 段 PHASE-CONTEXTPANEL-5PAGES-REPORT.md
  - 5 commit: 1 hook + 4 页面各 1 commit

base: main HEAD (per 2026-09-06 07:34 JST 拍板, 批 2 启动, 等批 1 三 wt 合并)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 + 守门 #20 v2)

启动条件 (批 2):
  - 批 1 三 wt (property / comment / relation) 全部 merge 到 main 链
  - ContextPanel.tsx 5 slot API 稳定
  - TabProperty / TabComment / TabRelation / TabAi 4 组件可复用

守门:
  - cargo check --workspace --all-targets -j 4 0 err (跟 main 一致)
  - pnpm typecheck 0 err
  - pnpm test src/components/contextpanel/__tests__/DetailDrawer.integration.test.tsx 12/12 pass
  - 4 页面现有测试 100% 兼容

已知缺口 (per 守门 #11):
  - Phase 2+ 留: Export (PDF/CSV) / 离线缓存 / 移动端
  - /projects Kanban 验证兼容, 不重做
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

WT_PATH = Path(r"D:\Star\.worktrees\cp-orchestrator")  # 主 wt, Mavis 接手
WT_BRANCH = "feat/cp-orchestrator"
BRIEF_PATH = "docs/briefs/contextpanel-5pages.md"
AUTOMATION_SCRIPT = "scripts/automation/contextpanel_5pages.py"

GUARD_COMMANDS = [
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "typecheck"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/contextpanel/__tests__/DetailDrawer.integration.test.tsx"],
    # 4 页面现有测试
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/app/(app)/projects/"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/app/(app)/agents/"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/app/(app)/analytics/"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/app/(app)/planning/"],
]


def run_guard(label: str, cmd: list[str]) -> bool:
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
        return True


def main() -> int:
    print(f"=== ContextPanel 5 页面 2-Pane 改造 (worker-5 / {WT_BRANCH}, 批 2) ===")
    print(f"brief: {BRIEF_PATH}")
    print(f"automation: {AUTOMATION_SCRIPT}")
    print()

    all_pass = True
    for label, cmd in GUARD_COMMANDS:
        if not run_guard(label, cmd):
            all_pass = False

    print()
    print("=== commit message 模板 (5 commit) ===")
    print(
        "feat(cp-5pages-hook): useDetailDrawer hook + 键盘快捷键\n"
        "\n"
        "- useDetailDrawer.ts: 统一 4 页面开关 + active tab 状态\n"
        "- 键盘: Cmd+. 全折叠 / Cmd+[ / Cmd+] 切 tab\n"
        f"- 自动化档: {AUTOMATION_SCRIPT}\n"
        f"- brief: {BRIEF_PATH}\n"
        "\n"
        "---\n"
        "\n"
        "feat(cp-5pages-agent): /agent 页面 2-Pane 改造\n"
        "\n"
        "- AgentDetailPanel.tsx: 接 ContextPanel 5 tab 容器, 默认 active 属性 tab\n"
        "- /projects?tab=agents 页面 改用 2-Pane 模式\n"
        "- i18n 4 语言 +4 key = 16 行\n"
        "\n"
        "---\n"
        "\n"
        "feat(cp-5pages-notification): /notification 页面 2-Pane 改造\n"
        "\n"
        "- NotificationDetailPanel.tsx: 默认 active 活动 tab (通知已读记录)\n"
        "- i18n 4 语言 +4 key = 16 行\n"
        "\n"
        "---\n"
        "\n"
        "feat(cp-5pages-analytics): /analytics 页面 2-Pane 改造\n"
        "\n"
        "- AnalyticsDetailPanel.tsx: 默认 active 活动 tab (数据刷新历史)\n"
        "- i18n 4 语言 +4 key = 16 行\n"
        "\n"
        "---\n"
        "\n"
        "feat(cp-5pages-planning): /planning 页面 2-Pane 改造\n"
        "\n"
        "- PlanningDetailPanel.tsx: 默认 active 关联 tab (milestone → work-item 依赖图)\n"
        "- i18n 4 语言 +4 key = 16 行\n"
        "\n"
        "Co-Authored-By: Mavis 接手 agent <mavis@MiniMax.local>"
    )

    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
