#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
v1.01 WBS row insert (idempotent)
P2 阶段 worker 子代理 实跑 runbook 落档 — append v1.01 row after v0.99.2 row.

守门 #12 v21 [P] docs 同步必更新 WBS.
守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核 author=Ulysses.

Idempotent: 跑 2 次安全 (check v1.01 row exists before insert).
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")

V101_ROW = """| **v1.01** | **2026-09-10 21:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 审核 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发 仍允许 + 守门 #19 v19 Python 化累积规不破坏 v0.1)** | **§14.21 P2 阶段 worker 子代理 实跑 runbook 落档 (per P0-4 Stage 4.0-4.4 v0.94/v0.97/v0.99/v1.00 累计 30+ commit 收官 + P0-4 → P2 阶段过渡 + 9/10 21:00 JST Mavis 自驱不被动等指令) — 跳 v1.00 (v1.00 = P0-4 Stage 4.4 P3-D.6 15 张表 自动 fill 脚本 per 5cfb7b3 系列) — v1.01 占位 P2 阶段入门入口**(1) **commit `e1db397` 落档** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 + 守门 #1 禁回溯叙事 0 改 v0.72-v1.00 任何代码 + 守门 #19 v19 Python 化累积规 + 守门 #11 缺标比错标 P2 阶段入门 4 已知缺口显式标 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发 仍允许 + 9/8 15:29 JST Mavis 自驱不被动等指令): 2 files changed, +362 lines: (a) **`docs/deployment/P2-STAGE-RUNBOOK-001.md` 9.8KB 新增** (per 守门 #9 v19 + 守门 #11 缺标比错标): P2 阶段 worker 子代理 实跑 v0.66+v0.67+v0.68 P0-1 阶段 收官 pattern 复用, 6 步 (a. fork worktree wt-v102-p2-exec 基于 main 7c80398; b. install-sealed-secrets.sh 跑 controller 部署; c. platform_admin_password_gen.py 跑 32 字节 password 占位 + kubeseal encrypt; d. v0.99/v1.00 占位 schema 跑实际 DDL apply; e. v1.00 p3d6_13_tables_gen.py 跑 15 张表 DDL idempotent; f. p2_validation.py 跑 5 验证函数), 4 已知缺口显式列 (#1 P2 阶段 worker 子代理 实跑 in k3s-deployable + testcontainers-rs P0-4 dev env 无 Docker per 守门 #24 v2 G-5 mock; #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 P0-4 阶段只声明不跑 P2 阶段 worker 子代理 + testcontainers-rs 实测; #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 P2 阶段 worker 子代理 跑 install-sealed-secrets.sh + platform_admin_password_gen.py + kubeseal encrypt + kubectl apply; #4 端到端测试 5 守门 0 违反 P2 阶段 worker 子代理 实测 cargo check + cargo test + cargo fmt + RLS + PLATFORM_ADMIN); (b) **`scripts/sql/p2_validation.py` 6.5KB 新增** (per 守门 #9 v19 + 守门 #19 v19 Python 化 累积规): 5 验证函数 (validate_rls_template_functions 验 v0.92 rls_7_policy_gen.py 4 函数 import 成功; validate_p3d6_schema_template 验 v0.99 模板 CREATE TABLE + RLS ENABLE; validate_p3d6_15_tables 验 v1.00 p3d6-13-tables.sql 15 张表; validate_rls_11_tables 验 v0.92 rls-7policy-all-tables.sql 11 表 + RLS 7 类 policy; validate_drop_if_exists_idempotent 验 v0.97 模式 3 SQL 文件 DROP IF EXISTS >= CREATE POLICY), 跑 5/5 dry-run 实证 0 err; **(2) 守门 #1 v25 实证**: `cargo check --workspace --lib -j 4` = **0 err 1m 01s** (2 pre-existing warnings star-dispatcher + star-saga 跟 v1.01 无关); `cargo test -p star-pg-adapter --lib -j 4` = **35/35 PASS 0.00s** (worktree 独立 target dir 解决 LNK1104, 跟 v0.82 16/16 + v0.85 +5 sqlx 真实 impl 0 regression, 跟 v0.73-v0.75 15 ops/oauth pool() getter 全 0 regression); `cargo fmt --check` = pre-existing baseline issue (跟 main 7c80398 一致, 跟 v1.01 无关, 已知缺口 #5 显式标, 等 5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构); `python scripts/sql/p2_validation.py` = **5/5 PASS dry-run**; **(3) 守门合规 8 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行; **(4) merge to main 05f8b2f** (per 守门 #9 v19 + 守门 #14 v4): `git merge wt-v101-p2-runbook --no-ff` 0 conflict, push origin `ea13bc3..05f8b2f main -> main` 0 err; **(5) 守门 #1 禁回溯叙事**: v0.1-v1.00 修订历史不动, v1.01 row 显式标 P0-4 → P2 阶段过渡 入口落档; **(6) 跨 session 续 (per 守门 #9 v19)**: v1.02 = P2 阶段 worker 子代理 实跑 v1.00 脚本 (15 张表 DDL + v0.97 idempotent + testcontainers-rs 验证) 估 0.5-0.8M tokens / 0.42-0.67 SRE·周, 走 docs/briefs/p2-stage-exec.md 落档 + worktree wt-v102-p2-exec + 子代理 dispatch; **(7) 触发原因**: per v0.99/v1.00 累计 30+ P0-4 commit 收官 + P0-4 → P2 阶段过渡 + 9/8 15:29 JST Mavis 自驱不被动等指令, 跨 session 续做入口落档; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 21:00 JST Mavis 自驱 (per 守门 #9 v19 第 7 次强化 9/8 15:29 JST) + P0-4 Stage 4.0-4.4 v0.94/v0.97/v0.99/v1.00 累计 30+ P0-4 commit 收官 + P0-4 → P2 阶段过渡 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核 author=Ulysses |
"""

def main() -> int:
    if not WBS_PATH.exists():
        print(f"[err] WBS not found: {WBS_PATH}")
        return 1

    content = WBS_PATH.read_text(encoding="utf-8")

    if "| **v1.01** |" in content:
        print("[skip] v1.01 row already exists in WBS — idempotent")
        return 0

    # Insert v1.01 row after v0.99.2 row
    marker = "| **v0.99.2** |"
    idx = content.find(marker)
    if idx < 0:
        print("[err] v0.99.2 row not found — cannot anchor v1.01")
        return 1

    # Find end of v0.99.2 row (next newline after v0.99.2 row, or EOF)
    end_of_v0992 = content.find("\n", idx)
    if end_of_v0992 < 0:
        # v0.99.2 is the last row and the file has no trailing newline
        # Append a newline first, then v1.01 row
        new_content = content + "\n" + V101_ROW
    else:
        new_content = (
            content[: end_of_v0992 + 1]
            + V101_ROW
            + content[end_of_v0992 + 1 :]
        )

    WBS_PATH.write_text(new_content, encoding="utf-8")
    print(f"[ok] v1.01 row inserted after v0.99.2 in {WBS_PATH}")
    print(f"[ok] WBS size: {len(content)} -> {len(new_content)} ({len(new_content) - len(content):+d} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
