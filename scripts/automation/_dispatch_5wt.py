#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""5 wt brief 落档 (per docs/automation-design.md §3.1 dispatcher 范式 + 守门 #20 v2)"""

import subprocess
import sys
from pathlib import Path

WT_BASE = Path(r"D:\Star\.worktrees\feat-auto-20260902-c8cfc4ff")
DISPATCHER = WT_BASE / "scripts" / "automation" / "dispatcher.py"

# 5 wt 任务卡 (per docs/automation-design.md §4 任务卡初判 [P] 任务卡 20 张挑 5 张最有价值)
WT_PLAN = [
    {
        "task_id": "P3-H2-1",
        "phase": "P3-H2",
        "agent": "worker",
        "wt_branch": "wt-20260902-p-h2-1",
        "wt_path": r"D:\Star\.worktrees\wt-p-h2-1",
        "content": (
            "H2-1 star_context 共享 ActorContext 字段扩展 真实实装 (per docs/automation-design.md v0.1 §4.6 + HANDOFF-ST-001 H2 stage 1 commit 68ae5ff)\n\n"
            "scope: scripts/automation/refactor_template.py 派 H2Stage1Refactor 子类化 (继承 parse_report + apply + verify + rollback + run_full 6 方法)\n"
            "base: 094284b (per automation v0.1)\n"
            "mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)\n"
            "交付:\n"
            "  1. scripts/automation/refactor_template.py 已有 6 方法 (parse_report / apply / verify / rollback / run_full / export_actions_json) + ExampleRemoveActorCtx 范例\n"
            "  2. scripts/automation/h2_refactor.py 新建, 继承 RefactorTemplate, 重写 parse_report 解析 HANDOFF-ST-001 H2 stage 1 report → Action 列表\n"
            "  3. 实证 dry_run=True 跑 run_full 0 错 + git log --follow <wt-branch> 实证 68ae5ff commit\n"
            "守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt\n"
            "docs: commit message 含 scripts/automation/h2_refactor.py 路径 + 引用 HANDOFF-ST-001 v0.2 §1 v17\n"
            "已知: H2 stage 1 已落地 commit 68ae5ff, wt 内新增 h2_refactor.py 是 stage 1 范式化封装, 不修改 star-context crate 源码"
        ),
    },
    {
        "task_id": "P3-B.5",
        "phase": "P3-B",
        "agent": "worker",
        "wt_branch": "wt-20260902-p-b5",
        "wt_path": r"D:\Star\.worktrees\wt-p-b5",
        "content": (
            "B.5 OpenClaw 真实集成 e2e (per docs/automation-design.md v0.1 §4.1 + WBS §1 B.5)\n\n"
            "scope: scripts/automation/integration_e2e.py 落 5 endpoint × 4 method OpenClaw wiremock stub\n"
            "base: 094284b (per automation v0.1)\n"
            "mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)\n"
            "交付:\n"
            "  1. scripts/automation/integration_e2e.py 新建, 5 endpoint stub: /v1/agents, /v1/sessions, /v1/messages, /v1/tools/invoke, /v1/cost\n"
            "  2. 4 method 覆盖: GET (list/retrieve), POST (create/start), PUT (update), DELETE (close)\n"
            "  3. 5 endpoint × 4 method = 20 case, 每个 case 返 wiremock 格式 response\n"
            "  4. scripts/automation/__tests__/integration_e2e_test.py 5 测试 (每个 endpoint 1 测试)\n"
            "守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt\n"
            "docs: commit message 含 scripts/automation/integration_e2e.py 路径 + 引用 WBS §1 B.5\n"
            "已知: 5 endpoint 待 Ulysses 拍板 (per WBS §1 9/2 23:59 JST 选 1 拍板 + 共享脚本优先), 真实凭证 (B.5 mock 备选 per 29692a7)"
        ),
    },
    {
        "task_id": "P3-B.6",
        "phase": "P3-B",
        "agent": "worker",
        "wt_branch": "wt-20260902-p-b6",
        "wt_path": r"D:\Star\.worktrees\wt-p-b6",
        "content": (
            "B.6 Hermes 真实集成 e2e (per docs/automation-design.md v0.1 §4.1 + WBS §1 B.6)\n\n"
            "scope: scripts/automation/integration_e2e.py 落 5 endpoint × 4 method Hermes wiremock stub (跟 B.5 共享脚本, 改 base_url + auth header)\n"
            "base: 094284b (per automation v0.1)\n"
            "mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)\n"
            "交付:\n"
            "  1. scripts/automation/integration_e2e.py 同 B.5, 加 HermesConfig dataclass (base_url / api_key / timeout)\n"
            "  2. 5 endpoint stub: /v2/hermes/agents, /v2/hermes/sessions, /v2/hermes/messages, /v2/hermes/tools/invoke, /v2/hermes/cost\n"
            "  3. 4 method 覆盖同 B.5, 5 × 4 = 20 case\n"
            "  4. scripts/automation/__tests__/integration_e2e_test.py 加 5 测试 (跟 B.5 共享, 5/10 endpoint)\n"
            "守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt\n"
            "docs: commit message 含 scripts/automation/integration_e2e.py 路径 + 引用 WBS §1 B.6\n"
            "已知: 共享 B.5 脚本, B.5 收官后 B.6 直接 import 复用; Hermes 真实凭证 (B.6 mock 备选 per 29692a7)"
        ),
    },
    {
        "task_id": "P3-C.6",
        "phase": "P3-C",
        "agent": "worker",
        "wt_branch": "wt-20260902-p-c6",
        "wt_path": r"D:\Star\.worktrees\wt-p-c6",
        "content": (
            "C.6 Saga 跨 5 域补偿 + 失败回滚 (per docs/automation-design.md v0.1 §4.2 + WBS §2 C.6 commit 25d086e)\n\n"
            "scope: scripts/automation/saga_e2e.py 跨 5 域 (player/economy/match/social/admin) 补偿 + 回滚 e2e 实证\n"
            "base: 094284b (per automation v0.1)\n"
            "mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)\n"
            "交付:\n"
            "  1. scripts/automation/saga_e2e.py 新建, 5 域补偿链: player (创建角色) → economy (扣费) → match (匹配对手) → social (发通知) → admin (审计), 任何 1 步失败回滚前 4 步\n"
            "  2. SagaStep dataclass (id / domain / action / compensation / idempotency_key per INV-SG-05)\n"
            "  3. 5 域 × 2 case (成功 + 失败回滚) = 10 case 实证\n"
            "  4. scripts/automation/__tests__/saga_e2e_test.py 10 测试\n"
            "守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt\n"
            "docs: commit message 含 scripts/automation/saga_e2e.py 路径 + 引用 WBS §2 C.6\n"
            "已知: C.6 已收官 commit 25d086e (per WBS §2), star-saga crate 增强; 5 域 Lead 真人到位前 e2e 用 mock 域"
        ),
    },
    {
        "task_id": "P3-F.6",
        "phase": "P3-F",
        "agent": "worker",
        "wt_branch": "wt-20260902-p-f6",
        "wt_path": r"D:\Star\.worktrees\wt-p-f6",
        "content": (
            "F.6 推 origin (R-05 反转) (per docs/automation-design.md v0.1 §4.5 + WBS §5 F.6 + §14.4 B-8)\n\n"
            "scope: scripts/automation/git_push.py 落真实 git push 3 branch (main + feature/ai-ide-compat + wt branch) 到 https://github.com/UlyssesLeoLee/Star.git\n"
            "base: 094284b (per automation v0.1)\n"
            "mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)\n"
            "交付:\n"
            "  1. scripts/automation/git_push.py 新建, GitPushHelper 类 (push / validate / scan_secret 3 方法)\n"
            "  2. 推 3 branch + secret 扫描 (.env / API key / PAT)\n"
            "  3. 守门 #1+#6+#9+#12 实证: 推 0 失败 + author Ulysses 唯一 + secret 0 命中 + docs commit\n"
            "  4. scripts/automation/__tests__/git_push_test.py 3 测试 (validate + scan_secret + dry_run push)\n"
            "守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt\n"
            "docs: commit message 含 scripts/automation/git_push.py 路径 + 引用 WBS §5 F.6\n"
            "已知: 推 origin 9/1 23:59 JST 失败 (github.com 443 不可达 + 无 PAT/GITHUB_TOKEN), wt 内 dry_run=True 默认, 真推需 Ulysses 提供 PAT 跨 session 续"
        ),
    },
]


def main():
    for wt in WT_PLAN:
        print(f"=== {wt['task_id']} → {wt['wt_branch']} ===")
        result = subprocess.run(
            [
                sys.executable,
                str(DISPATCHER),
                "--task-id", wt["task_id"],
                "--phase", wt["phase"],
                "--agent", wt["agent"],
                "--content", wt["content"],
                "--timeout", "1800",
            ],
            capture_output=True,
            text=True,
            cwd=str(WT_BASE),
        )
        print(f"  exit_code: {result.returncode}")
        print(f"  stdout: {result.stdout.strip()}")
        if result.stderr.strip():
            print(f"  stderr: {result.stderr.strip()[:200]}")
        # 验证 brief 落档
        brief_path = WT_BASE / "docs" / "briefs" / f"{wt['task_id']}.md"
        status_path = WT_BASE / "docs" / "briefs" / f"{wt['task_id']}.status.json"
        output_path = WT_BASE / "docs" / "briefs" / f"{wt['task_id']}.output.md"
        print(f"  brief: exists={brief_path.exists()} size={brief_path.stat().st_size if brief_path.exists() else 0}")
        print(f"  status: exists={status_path.exists()}")
        print(f"  output: exists={output_path.exists()}")
        print()


if __name__ == "__main__":
    main()
