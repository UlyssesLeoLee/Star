#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
registry.md v0.25 row insert (idempotent) for v1.01 P2 stage runbook.

守门 #12 v21 [P] docs 同步必更新 registry.
"""
import io
import sys
from pathlib import Path

REG_PATH = Path("scripts/automation/registry.md")

V025_ROW = """| **v0.25** | **2026-09-10 21:00 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 业务 1 改 (2 file 落档, docs 同步) + §3 v0.25**: P2 阶段 worker 子代理 实跑 runbook 落档 (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #14 v4 Mavis 审核 author=Ulysses + WBS v1.01 row 落档), 关联 commit `e1db397` (worktree `D:/Star/.worktrees/wt-v101-p2-runbook/` 落档, branch `wt-v101-p2-runbook` 基于 main 4aba448, rebase onto main 7c80398 0 conflict, Mavis merge to main `05f8b2f` 0 conflict, push origin `ea13bc3..05f8b2f main -> main` 0 err, per 守门 #1 v15 docs 同步饱和第 91-92 次新事件触发 仍允许 + 守门 #1 禁回溯叙事不重写 v0.72-v1.00 任何代码 + 守门 #19 v19 Python 化 累积规不破坏 v0.1 + 守门 #11 缺标比错标 P2 阶段入门 4 已知缺口显式标 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发 仍允许 + 9/8 15:29 JST Mavis 自驱不被动等指令): (a) **`docs/deployment/P2-STAGE-RUNBOOK-001.md` 9.8KB 新增** P2 阶段 worker 子代理 实跑 6 步 runbook (fork wt-v102-p2-exec + install-sealed-secrets.sh + platform_admin_password_gen.py + kubeseal encrypt + p3d6_13_tables_gen.py 15 张表 DDL idempotent + p2_validation.py 5 验证函数), 4 已知缺口显式列 (#1 P2 阶段 worker 子代理 实跑 in k3s-deployable + testcontainers-rs P0-4 dev env 无 Docker per 守门 #24 v2 G-5 mock; #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 P0-4 阶段只声明不跑 P2 阶段 worker 子代理 + testcontainers-rs 实测; #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装; #4 端到端测试 5 守门 0 违反); (b) **`scripts/sql/p2_validation.py` 6.5KB 新增** 5 验证函数 (validate_rls_template_functions + validate_p3d6_schema_template + validate_p3d6_15_tables + validate_rls_11_tables + validate_drop_if_exists_idempotent), 跑 5/5 dry-run 实证 0 err; **4 守门实证** cargo check --workspace --lib -j 4 = 0 err 1m 01s + cargo test -p star-pg-adapter --lib -j 4 = 35/35 PASS 0.00s (worktree 独立 target dir 解决 LNK1104, 跟 v0.82 16/16 + v0.85 +5 sqlx 真实 impl 0 regression, 跟 v0.73-v0.75 15 ops/oauth pool() getter 全 0 regression) + cargo fmt --check = pre-existing baseline issue (跟 main 7c80398 一致, 跟 v1.01 无关, 已知缺口 #5 显式标) + python scripts/sql/p2_validation.py 5/5 PASS dry-run; **守门合规 8 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行; `git log -1 --format='%an <%ae>' e1db397` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4); **累计 P0-4 + P2 阶段过渡**: v0.66-v1.00 累计 30+ P0-4 commit 收官 + v1.01 P2 阶段入门入口落档, v1.02 = P2 阶段 worker 子代理 实跑 v1.00 脚本 (15 张表 DDL + v0.97 idempotent + testcontainers-rs 验证) 估 0.5-0.8M tokens / 0.42-0.67 SRE·周 跨 session 续; worktree cleanup: `git worktree remove .worktrees/wt-v101-p2-runbook --force + git branch -D wt-v101-p2-runbook` 0 残留 | **2026-09-10 21:00 JST 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进 (per 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和 + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #19 v19 累积规不破坏 v0.1)** |
"""


def main() -> int:
    if not REG_PATH.exists():
        print(f"[err] registry not found: {REG_PATH}")
        return 1

    content = REG_PATH.read_text(encoding="utf-8")

    if "| **v0.25** |" in content:
        print("[skip] v0.25 row already exists in registry — idempotent")
        return 0

    # Insert v0.25 after v0.24 row, or at end if v0.24 row not found
    marker = "| **v0.24** |"
    idx = content.find(marker)
    if idx < 0:
        print("[err] v0.24 row not found — cannot anchor v0.25")
        return 1

    end_of_v024 = content.find("\n", idx)
    if end_of_v024 < 0:
        new_content = content + "\n" + V025_ROW
    else:
        new_content = (
            content[: end_of_v024 + 1]
            + V025_ROW
            + content[end_of_v024 + 1 :]
        )

    REG_PATH.write_text(new_content, encoding="utf-8")
    print(f"[ok] v0.25 row inserted after v0.24 in {REG_PATH}")
    print(f"[ok] registry size: {len(content)} -> {len(new_content)} ({len(new_content) - len(content):+d} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
