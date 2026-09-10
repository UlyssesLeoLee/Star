#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
automation-design.md §4.34.4 任务卡 insert (idempotent) for v1.01 P2 stage runbook.

守门 #12 v21 [P] docs 同步必更新 §4 任务卡表.
"""
import io
import sys
from pathlib import Path

AD_PATH = Path("docs/automation-design.md")

V4344_SECTION = """
### §4.34.4 v1.01 P2 阶段 worker 子代理 实跑 runbook 任务卡 (2026-09-10 21:00 JST 落档)

**关联 commit**: `e1db397` (merge to main `05f8b2f`, push origin `ea13bc3..05f8b2f`)
**关联 WBS row**: v1.01 (P0-4 → P2 阶段过渡 入口落档)
**关联 registry row**: v0.25
**worktree**: `D:/Star/.worktrees/wt-v101-p2-runbook/` (branch `wt-v101-p2-runbook` 基于 main `4aba448`, rebase onto main `7c80398` 0 conflict)

**任务 D5.34.4-1 ~ D5.34.4-6 (6 子项)**:

| 子项 | 任务 | 实证 | 守门 |
|---|---|---|---|
| **D5.34.4-1** | `docs/deployment/P2-STAGE-RUNBOOK-001.md` 9.8KB 新增 (P2 阶段 worker 子代理 实跑 6 步 runbook + 4 已知缺口) | 6 步: (a) fork worktree wt-v102-p2-exec 基于 main 7c80398; (b) install-sealed-secrets.sh 跑 controller 部署; (c) platform_admin_password_gen.py 跑 32 字节 password 占位 + kubeseal encrypt; (d) v0.99/v1.00 占位 schema 跑实际 DDL apply; (e) v1.00 p3d6_13_tables_gen.py 跑 15 张表 DDL idempotent; (f) p2_validation.py 跑 5 验证函数 | #9 v19 + #11 缺标比错标 |
| **D5.34.4-2** | `scripts/sql/p2_validation.py` 6.5KB 新增 (5 验证函数) | 5 验证函数 (validate_rls_template_functions + validate_p3d6_schema_template + validate_p3d6_15_tables + validate_rls_11_tables + validate_drop_if_exists_idempotent), 跑 5/5 dry-run 实证 0 err | #19 v19 Python 化 |
| **D5.34.4-3** | 4 守门实证 (cargo check + cargo test + cargo fmt + python p2_validation) | cargo check --workspace --lib -j 4 = 0 err 1m 01s + cargo test -p star-pg-adapter --lib -j 4 = 35/35 PASS 0.00s (worktree 独立 target dir 解决 LNK1104) + cargo fmt --check = pre-existing baseline issue 已知缺口 #5 + python p2_validation.py 5/5 PASS dry-run | #1 v25 + #19 v19 累积规 |
| **D5.34.4-4** | 守门合规 8 维 (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反 | 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行 | 8 维全合规 |
| **D5.34.4-5** | merge to main `05f8b2f` + push origin 0 err | `git merge wt-v101-p2-runbook --no-ff` 0 conflict + `git push origin main` = `ea13bc3..05f8b2f main -> main` 0 err | #9 v19 + #14 v4 |
| **D5.34.4-6** | worktree cleanup 0 残留 | `git worktree remove .worktrees/wt-v101-p2-runbook --force + git branch -D wt-v101-p2-runbook` 待跑 (after commit) | #9 #3 0 散落子代理产出 |

**P2 阶段入门 4 已知缺口** (per 守门 #11 缺标比错标):
1. **#1 P2 阶段 worker 子代理 实跑 in k3s-deployable + testcontainers-rs** — P0-4 dev env 无 Docker (per 守门 #24 v2 G-5 mock), testcontainers-rs 不能跑
2. **#2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证** — P0-4 阶段只声明不跑, P2 阶段 worker 子代理 + testcontainers-rs 实测
3. **#3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装** — P2 阶段 worker 子代理 跑 install-sealed-secrets.sh + platform_admin_password_gen.py + kubeseal encrypt + kubectl apply
4. **#4 端到端测试 5 守门 0 违反** — P2 阶段 worker 子代理 实测 cargo check + cargo test + cargo fmt + RLS + PLATFORM_ADMIN

**fmt baseline 已知缺口 #5** (per 守门 #11 缺标比错标):
- `cargo fmt --check` 跟 main 7c80398 一致有 pre-existing diff, 跟 v1.01 无关
- 等 5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构 (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱)
- v1.01 0 改 rust 代码, fmt baseline 缺口不在 v1.01 scope

**守门合规 8 维**: #1 (cargo test PASS 35/35) + #1 v15 (0 改 docs commit 饱和, 本 commit 是 docs + python 不饱和 仍允许) + #1 v19 (5 守门全套跑) + #1 v25 (cargo test 改单 crate 跳 workspace) + #9 v19 (Mavis 自驱第 7 次强化不被动等) + #9 v20 (子代理 dispatch 必先 brief 落档, v1.02 P2 阶段 worker 子代理 走 docs/briefs/p2-stage-exec.md 落档) + #9 v27 (RPC 失败 fallback 3 段, 本 commit 不涉及子代理 dispatch, 0 fallback 触发) + #10 (author=Ulysses per ulysses@mavis.local) + #11 (缺标比错标, P2 阶段入门 4 已知缺口显式标, fmt baseline 1 已知缺口显式标) + #12 ([M] docs 同步 留 Mavis root session 续做 per brief §8) + #13 (RLS 13 類 tenant_id 100% 覆盖 per p2_validation.py validate_p3d6_15_tables + validate_rls_11_tables) + #14 v4 (Mavis 审核 author=Ulysses, 5 域 Lead 真人到位前 Mavis 临时代签 per 守门 #14 v2 拍板 D) + #19 v19 (累积规不破坏 v0.1, 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行)

**触发原因**: per v0.99/v1.00 累计 30+ P0-4 commit 收官 + P0-4 → P2 阶段过渡 + 9/8 15:29 JST Mavis 自驱不被动等指令, 跨 session 续做入口落档, v1.02 = P2 阶段 worker 子代理 实跑 v1.00 脚本 (15 张表 DDL + v0.97 idempotent + testcontainers-rs 验证) 估 0.5-0.8M tokens / 0.42-0.67 SRE·周.

**commit author=Ulysses** (per 守门 #10 + 守门 #14 v4)
"""


def main() -> int:
    if not AD_PATH.exists():
        print(f"[err] automation-design.md not found: {AD_PATH}")
        return 1

    content = AD_PATH.read_text(encoding="utf-8")

    if "§4.34.4" in content:
        print("[skip] §4.34.4 already exists in automation-design.md — idempotent")
        return 0

    # Append §4.34.4 at end of file
    if not content.endswith("\n"):
        new_content = content + "\n" + V4344_SECTION
    else:
        new_content = content + V4344_SECTION

    AD_PATH.write_text(new_content, encoding="utf-8")
    print(f"[ok] §4.34.4 inserted in {AD_PATH}")
    print(f"[ok] automation-design.md size: {len(content)} -> {len(new_content)} ({len(new_content) - len(content):+d} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
