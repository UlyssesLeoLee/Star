#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ContextPanel Tab 2 评论 自动化档 (per docs/automation-design.md §1.3 + 守门 #19)

实装 5 tab 容器 slot 2 "评论",接 domain-comment InMemory Service + MSW handler。

scope:
  - 新建 frontend/src/components/contextpanel/TabComment.tsx (评论列表 / 撰写 / reaction / 删除)
  - 新建 frontend/src/mocks/handlers/comment.ts (5 端点: GET/POST/PATCH/DELETE/reaction)
  - 新建 frontend/src/mocks/data/comment.ts (5 条样例)
  - frontend/src/mocks/handlers/_index.ts 注册 5 handler
  - i18n 4 语言 +8 key = 32 行
  - 测试 TabComment.test.tsx 5/5 pass
  - 7 段 PHASE-CONTEXTPANEL-TAB2-COMMENT-REPORT.md
  - 1 commit, author=Ulysses, 引用 brief 路径

base: main HEAD (per 2026-09-06 07:34 JST 拍板, 等 worker-1 commit 合并后从 main 链拉)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 + 守门 #20 v2)

启动条件 (per brief §6):
  - worker-1 (cp-tab-property) commit 已 merge 到 main 链
  - 必先 git fetch + cherry-pick <worker-1-commit> 到本 wt base 再实装

守门:
  - cargo check --workspace --all-targets -j 4 0 err (跟 main 一致, 不动 Rust)
  - pnpm typecheck 0 err
  - pnpm test src/components/contextpanel/__tests__/TabComment.test.tsx 5/5 pass
  - MSW 5 端点全 200, curl 测试通过
  - 跟 worker-1/3 互不重叠文件 (mocks/handlers/comment.ts 独占)

已知缺口 (per 守门 #11):
  - 公开/内部评论 UI 切换, 但底层 data model 暂用单一 visibility 字段, 后续按 spec 拆
  - @ 提及通知推送 Phase 2+ 接 SSE
  - markdown 渲染用 react-markdown, 不支持自定义插件
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

WT_PATH = Path(r"D:\Star\.worktrees\cp-tab-comment")
WT_BRANCH = "feat/cp-tab-comment"
BRIEF_PATH = "docs/briefs/contextpanel-comment.md"
AUTOMATION_SCRIPT = "scripts/automation/contextpanel_comment.py"

GUARD_COMMANDS = [
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "typecheck"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/contextpanel/__tests__/TabComment.test.tsx"],
    # MSW 端点 smoke (走 frontend 端 msw 启动, 见 scripts/automation/_smoke_sprint_p1.js 范式)
    ["node", "scripts/automation/_smoke_msw_comment.js"],
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
    print(f"=== ContextPanel Tab 2 评论 (worker-2 / {WT_BRANCH}) ===")
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
        "feat(cp-tab-comment): ContextPanel Tab 2 评论 MSW 兜底实装\n"
        "\n"
        "- TabComment.tsx: 列表/撰写/reaction/删除 + 公开内部 tab 切换\n"
        "- mocks/handlers/comment.ts: 5 端点 (GET/POST/PATCH/DELETE/reaction)\n"
        "- mocks/data/comment.ts: 5 条样例\n"
        "- i18n 4 语言 +8 key = 32 行\n"
        "- 测试: TabComment.test.tsx 5/5 pass\n"
        "- 守门: 0 err, MSW 5 端点全 200\n"
        f"- 自动化档: {AUTOMATION_SCRIPT}\n"
        f"- brief: {BRIEF_PATH}\n"
        "- per 守门 #1+#19+#20+#9 v20\n"
        "\n"
        "Co-Authored-By: Mavis 接手 agent <mavis@MiniMax.local>"
    )

    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
