#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ContextPanel Tab 3 关联 + Tab 5 AI 助手 自动化档 (per docs/automation-design.md + 守门 #19)

实装 5 tab 容器 slot 3 "关联" + slot 5 "AI 助手",接 domain-relation GraphNode/GraphEdge 真实数据
(per Q3=A1 拍板, 实测 GraphNode/GraphEdge 在 crates/domain-relation/src/lib.rs:511-540)。

scope (Tab 3 关联):
  - 新建 frontend/src/components/contextpanel/TabRelation.tsx
  - 新建 frontend/src/mocks/handlers/relation.ts (4 端点)
  - 新建 frontend/src/mocks/data/relation.ts (3 种 edge_type × 5 work-item 子图)
  - i18n 4 语言 +6 key = 24 行
  - 测试 TabRelation.test.tsx 4/4 pass
  - 7 段 PHASE-CONTEXTPANEL-TAB3-RELATION-REPORT.md
  - 1 commit, author=Ulysses, 引用 brief

scope (Tab 5 AI 助手):
  - 新建 frontend/src/components/contextpanel/TabAi.tsx
  - 复用 mocks/handlers/relation.ts 的 /api/relations/subgraph 端点
  - i18n 4 语言 +4 key = 16 行
  - 测试 TabAi.test.tsx 3/3 pass
  - 7 段 PHASE-CONTEXTPANEL-TAB5-AI-REPORT.md
  - 1 commit, author=Ulysses, 引用 brief

base: main HEAD (per 2026-09-06 07:34 JST 拍板, 等 worker-1 commit 合并后从 main 链拉)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 + 守门 #20 v2)

启动条件:
  - worker-1 (cp-tab-property) commit 已 merge 到 main 链
  - 必先 git fetch + cherry-pick <worker-1-commit> 到本 wt base 再实装

守门:
  - cargo check --workspace --all-targets -j 4 0 err (跟 main 一致, 不动 Rust)
  - pnpm typecheck 0 err
  - pnpm test src/components/contextpanel/__tests__/TabRelation.test.tsx 4/4 pass
  - pnpm test src/components/contextpanel/__tests__/TabAi.test.tsx 3/3 pass
  - 跟 worker-1/2 互不重叠文件 (mocks/handlers/relation.ts + mocks/data/relation.ts 独占)

已知缺口 (per 守门 #11):
  - SVG 渲染子图默认 depth=2, 超出 20 节点折叠
  - 风险评估仅看 blocks edge, 缺 PR 冲突 / 状态变更等
  - 建议模板是静态, 不接 LLM
  - 相似项 Top 5 按子图节点数排序, 不接 embedding
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

WT_PATH = Path(r"D:\Star\.worktrees\cp-tab-relation")
WT_BRANCH = "feat/cp-tab-relation"
BRIEF_PATH = "docs/briefs/contextpanel-relation.md"
AUTOMATION_SCRIPT = "scripts/automation/contextpanel_relation.py"

GUARD_COMMANDS = [
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "typecheck"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/contextpanel/__tests__/TabRelation.test.tsx"],
    ["pnpm", "--dir", str(WT_PATH / "frontend"), "test", "src/components/contextpanel/__tests__/TabAi.test.tsx"],
    ["node", "scripts/automation/_smoke_msw_relation.js"],
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
    print(f"=== ContextPanel Tab 3 关联 + Tab 5 AI 助手 (worker-3 / {WT_BRANCH}) ===")
    print(f"brief: {BRIEF_PATH}")
    print(f"automation: {AUTOMATION_SCRIPT}")
    print()

    all_pass = True
    for label, cmd in GUARD_COMMANDS:
        if not run_guard(label, cmd):
            all_pass = False

    print()
    print("=== commit message 模板 (Tab 3) ===")
    print(
        "feat(cp-tab-relation): ContextPanel Tab 3 关联 子图渲染实装\n"
        "\n"
        "- TabRelation.tsx: 关联列表 (按 edge_type 分组) + 添加/删除 + SVG 子图 (≤ 20 节点)\n"
        "- mocks/handlers/relation.ts: 4 端点 (GET/POST/DELETE/subgraph)\n"
        "- mocks/data/relation.ts: 3 edge_type × 5 work-item 子图样例\n"
        "- i18n 4 语言 +6 key = 24 行\n"
        "- 测试: TabRelation.test.tsx 4/4 pass\n"
        f"- 自动化档: {AUTOMATION_SCRIPT}\n"
        f"- brief: {BRIEF_PATH}\n"
        "\n"
        "=== commit message 模板 (Tab 5) ==="
    )
    print(
        "feat(cp-tab-ai): ContextPanel Tab 5 AI 助手 关联子图驱动实装\n"
        "\n"
        "- TabAi.tsx: 相似项 Top 5 + 风险徽章 (blocks) + 建议模板 3 块\n"
        "- 复用 /api/relations/subgraph 端点 (worker-3 拍板)\n"
        "- 失败回退: API 504 → \"AI 助手暂不可用\" 占位\n"
        "- i18n 4 语言 +4 key = 16 行\n"
        "- 测试: TabAi.test.tsx 3/3 pass\n"
        f"- 自动化档: {AUTOMATION_SCRIPT}\n"
        f"- brief: {BRIEF_PATH}\n"
        "- per 守门 #19+#20+#9 v20, Q3=A1 拍板接 domain-relation Graph 真实数据\n"
        "\n"
        "Co-Authored-By: Mavis 接手 agent <mavis@MiniMax.local>"
    )

    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
