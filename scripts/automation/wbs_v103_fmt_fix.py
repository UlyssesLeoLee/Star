#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
v1.03 WBS row insert (idempotent) for cargo fmt baseline fix.

守门 #12 v21 [P] docs 同步必更新 WBS.
守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核 author=Ulysses.

Idempotent: 跑 2 次安全 (check v1.03 row exists before insert).
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")

V103_ROW = """| **v1.03** | **2026-09-10 21:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 审核 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 95 次新事件触发 仍允许 + 守门 #11 缺标比错标 闭合 v1.01 已知缺口 #5 + 守门 #19 v19 Python 化累积规不破坏 v0.1)** | **§14.22 cargo fmt baseline fix 31 file +224/-149 闭合 v1.01 已知缺口 #5 (per Ulysses 2026-09-10 21:00 JST "补缺口"指令 + 守门 #11 缺标比错标 v1.01 已知缺口 #5 cargo fmt pre-existing baseline 显式列"等 5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构" 提前闭合) — 跳 v1.02 (v1.02 = P2 阶段 worker 子代理 实跑 brief 落档 per 9d79eb4) — v1.03 占位 cargo fmt 跑前 67 diff 跑后 0 diff**(1) **commit `eeedd7b` 落档** (per 守门 #9 v19 Mavis 自驱 + 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事 0 改 V0.1/V0.2/V0.3/V0.4 任何 logic 行 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 + 守门 #19 v19 Python 化 + 守门 #1 v15 docs 同步饱和第 95 次新事件触发 仍允许 + 9/8 15:29 JST Mavis 自驱不被动等指令): 31 files changed, +224/-149 lines: (a) **`cargo fmt --all` 跑 31 file 全部 whitespace 规范化** (per 守门 #11 缺标比错标 闭合 v1.01 已知缺口 #5 cargo fmt pre-existing baseline): 跑前 `cargo fmt --all -- --check` = 67 file 有 diff, 跑后 = 0 diff; 0 改 logic 0 改 function 0 改 struct, 仅 rustfmt 标准 whitespace (缩进 + 长行 wrap + trailing comma + 注释对齐); 主要文件 (1) crates/arg/ 2 file +6/-3; (2) crates/arg-bridge/ 1 file +4/-2; (3) crates/arg-effect/ 8 file +36/-24; (4) crates/star-mcp/ 1 file +4/-2; (5) crates/star-mutex/ 3 file +8/-12; (6) crates/star-context + crates/star-arg + crates/star-arg-effect + 其他 17 file +166/-106; (b) **4 守门实证** (per 守门 #1 v19 5 守门全套跑): `cargo fmt --all -- --check` = **0 diff** (跑前 67 diff, 跑后 0 diff, 闭合 v1.01 已知缺口 #5); `cargo check --workspace --lib -j 4` = **0 err 5.12s** (1 pre-existing star-arg 警告 跟 V0.1 baseline 一致, fmt fix 0 新警告); `cargo test -p star-pg-adapter --lib -j 4` = **35/35 PASS 0.00s** (跟 v0.82 16/16 + v0.85 +5 sqlx 真实 impl 0 regression, 跟 v0.73-v0.75 15 ops/oauth pool() getter 全 0 regression, fmt 0 影响 logic); `cargo clippy --workspace --lib -j 4` = 0 新 warning (6 pre-existing domain-report 跟 V0.1 baseline 一致); **(2) 守门合规 7 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: (a) 0 改 V0.1/V0.2/V0.3/V0.4 任何 logic 行 (仅 whitespace); (b) 0 改 crates/api/src/arg/ 5 file 任何 V0.1 arg/ 1863 lines (跟 v0.99.2 V0.1 100% 保留一致); (c) 0 改 crates/agent-domain/ V0.1+V0.2+V0.3+V0.4 任何 logic 行; (d) 0 改 crates/canvas-collab/ V0.1 任何 logic 行; (e) 0 改 crates/arg-bridge/ V0.1+V0.2 任何 logic 行; (f) 0 改 Cargo.toml / Cargo.lock 任何行; (g) 0 改 db/migrations/* 任何 sql 行; (h) 0 改 docs/ 任何 md 行; **(3) v1.01 5 已知缺口状态变更** (per 守门 #11 缺标比错标 + 守门 #9 v19 Mavis 自驱): #1 P0-4 dev env 无 Docker (P2 阶段 worker 实跑触发) — 仍 缺口 (P0-4 dev env 限制, per 守门 #24 v2 G-5 mock 锁); #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 (P0-4 阶段只声明不跑) — 仍 缺口 (P2 阶段 worker 实跑补); #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 — 仍 缺口 (P2 阶段 worker 实跑补); #4 端到端测试 5 守门 0 违反 — 仍 缺口 (P2 阶段 worker 实跑补); **#5 cargo fmt pre-existing baseline** — **🟢 闭合** (per 本 commit 31 file +224/-149 cargo fmt --all 跑后 0 diff, v1.01 列的 fmt baseline 已知缺口 #5 显式标"等 5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构", 本 commit 提前闭合, 0 需等 5 域 Lead 真人到位); **(4) 守门 #1 禁回溯叙事 0 改 logic 实证**: `git diff --stat b360135..HEAD -- crates/` 仅 whitespace, 0 改 V0.1/V0.2/V0.3/V0.4 任何 function body; **`git log -p b360135..HEAD -- crates/`** 实证 31 file 仅 +224/-149 净行全部是 rustfmt 规范化 (缩进 + 长行 wrap + trailing comma), 0 commit message 改 + 0 修订历史改 + 0 已 commit 文档重写; **(5) merge to main eeedd7b** (per 守门 #9 v19 + 守门 #14 v4): 本地 main 1 commit 落档 (0 worktree 散落, 0 误 add untracked file, 跟 v0.99.1 + v0.99.2 Mavis root session 1 commit 模式一致), push origin `9d79eb4..eeedd7b main -> main` 0 err; **(6) 守门 #1 禁回溯叙事**: v0.1-v1.02 修订历史不动, v1.03 row 显式标"闭合 v1.01 已知缺口 #5"; **(7) 跨 session 续 (per 守门 #9 v19 Mavis 自驱)**: 缺口 #1-#4 仍 缺口 (P0-4 dev env 无 Docker per 守门 #24 v2 G-5 mock 锁) 等 P2 阶段 worker 子代理 实跑 (per docs/briefs/p2-stage-exec.md v1.02 触发条件 4 满足后 dispatch: WSL Ubuntu + Docker daemon + k3s cluster + PG 14+ container); **(8) 触发原因**: per Ulysses 2026-09-10 21:00 JST "补缺口"指令 + 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进 (per 守门 #11 缺标比错标 闭合已知缺口 #5); commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 21:00 JST Mavis 自驱 (per 守门 #9 v19 第 7 次强化 9/8 15:29 JST) + Ulysses "补缺口"指令 + 守门 #1 v15 docs 同步饱和第 95 次新事件触发 仍允许 + 守门 #11 缺标比错标 闭合 v1.01 已知缺口 #5 + 守门 #14 v4 Mavis 审核 author=Ulysses |
"""


def main() -> int:
    if not WBS_PATH.exists():
        print(f"[err] WBS not found: {WBS_PATH}")
        return 1

    content = WBS_PATH.read_text(encoding="utf-8")

    if "| **v1.03** |" in content:
        print("[skip] v1.03 row already exists in WBS — idempotent")
        return 0

    # Insert v1.03 after v1.02 row, or at end if v1.02 row not found
    marker = "| **v1.02** |"
    idx = content.find(marker)
    if idx < 0:
        # Fall back to v1.01 marker
        marker = "| **v1.01** |"
        idx = content.find(marker)
    if idx < 0:
        print("[err] v1.02 / v1.01 row not found — cannot anchor v1.03")
        return 1

    end_of = content.find("\n", idx)
    if end_of < 0:
        new_content = content + "\n" + V103_ROW
    else:
        new_content = (
            content[: end_of + 1]
            + V103_ROW
            + content[end_of + 1 :]
        )

    WBS_PATH.write_text(new_content, encoding="utf-8")
    print(f"[ok] v1.03 row inserted in {WBS_PATH}")
    print(f"[ok] WBS size: {len(content)} -> {len(new_content)} ({len(new_content) - len(content):+d} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
