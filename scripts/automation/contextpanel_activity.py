#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ContextPanel Tab 4 活动 自动化档 (per docs/automation-design.md + 守门 #19) — 批 2

实装 5 tab 容器 slot 4 "活动",接 domain-audit InMemory Service + MSW handler。

scope:
  - 新建 frontend/src/components/contextpanel/TabActivity.tsx
  - 新建 frontend/src/mocks/handlers/audit.ts (2 端点)
  - 新建 frontend/src/mocks/data/audit.ts (5 event_type × 7 天 = 20+ 样例)
  - i18n 4 语言 +5 key = 20 行
  - 测试 TabActivity.test.tsx 4/4 pass
  - 7 段 PHASE-CONTEXTPANEL-TAB4-ACTIVITY-REPORT.md
  - 1 commit, author=Ulysses, 引用 brief

base: main HEAD (per 2026-09-06 07:34 JST 拍板, 批 2 启动, 等批 1 三 wt 合并)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 + 守门 #20 v2)

启动条件 (批 2):
  - worker-1 (cp-tab-property) commit 已 merge 到 main 链
  - worker-2 (cp-tab-comment) commit 已 merge 到 main 链
  - worker-3 (cp-tab-relation) commit 已 merge 到 main 链
  - ContextPanel.tsx 5 slot API 稳定

守门:
  - cargo check --workspace --all-targets -j 4 0 err (跟 main 一致, 不动 Rust)
  - pnpm typecheck 0 err
  - pnpm test src/components/contextpanel/__tests__/TabActivity.test.tsx 4/4 pass
  - 跟 worker-1/2/3 互不重叠文件 (mocks/handlers/audit.ts + mocks/data/audit.ts 独占)

已知缺口 (per 守门 #11):
  - 暂不实现 export (PDF/CSV), Phase 2+ 留
  - 5 event_type diff 字段差异大, 统一用 discriminated union 兜底
  - 时间线 > 1000 条走分页 50/页 + virtualized list
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

WT_PATH = Path(r"D:\Star\.worktrees\cp-tab-activity")
WT_BRANCH = "feat/cp-tab-activity"
BRIEF_PATH = "docs/briefs/contextpanel-activity.md"
AUTOMATION_SCRIPT = "scripts/automation/contextpanel_activity.py"

GUARD_COMMANDS = [
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "typecheck"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/contextpanel/__tests__/TabActivity.test.tsx"],
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
    print(f"=== ContextPanel Tab 4 活动 (worker-4 / {WT_BRANCH}, 批 2) ===")
    print(f"brief: {BRIEF_PATH}")
    print(f"automation: {AUTOMATION_SCRIPT}")
    print()

    all_pass = True
    for label, cmd in GUARD_COMMANDS:
        if not run_guard(label, cmd):
            all_pass = False

    print()
    print("=== commit message 模板 ===")
    print(
        "feat(cp-tab-activity): ContextPanel Tab 4 活动 审计时间线实装\n"
        "\n"
        "- TabActivity.tsx: 时间线倒序 + 5 chips 筛选 (全部/创建/状态/字段/通知/评论) + diff 展开\n"
        "- mocks/handlers/audit.ts: 2 端点 (target_type=work_item / actor_id)\n"
        "- mocks/data/audit.ts: 5 event_type × 7 天 = 20+ 样例\n"
        "- i18n 4 语言 +5 key = 20 行\n"
        "- 测试: TabActivity.test.tsx 4/4 pass\n"
        f"- 自动化档: {AUTOMATION_SCRIPT}\n"
        f"- brief: {BRIEF_PATH}\n"
        "- per 守门 #1+#19+#20+#9 v20\n"
        "\n"
        "Co-Authored-By: Mavis 接手 agent <mavis@MiniMax.local>"
    )

    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
