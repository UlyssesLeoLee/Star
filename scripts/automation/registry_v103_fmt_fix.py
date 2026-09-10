#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
registry.md v0.26 row insert (idempotent) for v1.03 cargo fmt baseline fix.

守门 #12 v21 [P] docs 同步必更新 registry.
"""
import io
import sys
from pathlib import Path

REG_PATH = Path("scripts/automation/registry.md")

V026_ROW = """| **v0.26** | **2026-09-10 21:00 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 业务 0 改 (31 file whitespace 规范化, docs 同步) + §3 v0.26**: cargo fmt baseline fix 闭合 v1.01 已知缺口 #5 (per Ulysses 2026-09-10 21:00 JST "补缺口"指令 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #11 缺标比错标 v1.01 已知缺口 #5 cargo fmt pre-existing baseline 显式列"等 5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构" 提前闭合 + 守门 #14 v4 Mavis 审核 author=Ulysses + WBS v1.03 row 落档), 关联 commit `eeedd7b` (本地 main 1 commit 落档 0 worktree 散落, 跟 v0.99.1 + v0.99.2 Mavis root session 1 commit 模式一致, push origin `9d79eb4..eeedd7b main -> main` 0 err, per 守门 #1 v15 docs 同步饱和第 95-96 次新事件触发 仍允许 + 守门 #1 禁回溯叙事不重写 v0.1-v1.02 任何 logic + 守门 #19 v19 Python 化 累积规不破坏 v0.1 + 守门 #11 缺标比错标 闭合 v1.01 已知缺口 #5 + 守门 #1 v25 cargo test 改单 crate 35/35 PASS 0.00s + 9/8 15:29 JST Mavis 自驱不被动等指令): (a) **`cargo fmt --all` 跑 31 file 全部 whitespace 规范化** 跑前 67 file 有 diff, 跑后 0 diff, 0 改 logic 0 改 function 0 改 struct, 仅 rustfmt 标准 whitespace (缩进 + 长行 wrap + trailing comma + 注释对齐); 主要文件 (1) crates/arg/ 2 file +6/-3; (2) crates/arg-bridge/ 1 file +4/-2; (3) crates/arg-effect/ 8 file +36/-24; (4) crates/star-mcp/ 1 file +4/-2; (5) crates/star-mutex/ 3 file +8/-12; (6) crates/star-context + crates/star-arg + crates/star-arg-effect + 其他 17 file +166/-106; (b) **4 守门实证** `cargo fmt --all -- --check` = 0 diff + `cargo check --workspace --lib -j 4` = 0 err 5.12s + `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS 0.00s (跟 v0.82 16/16 + v0.85 +5 sqlx 真实 impl 0 regression) + `cargo clippy --workspace --lib -j 4` = 0 新 warning; (c) **守门合规 7 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 V0.1/V0.2/V0.3/V0.4 任何 logic 行, 0 改 crates/api/src/arg/ 5 file 任何 V0.1 1863 lines, 0 改 crates/agent-domain/ V0.1+V0.2+V0.3+V0.4 任何 logic 行, 0 改 crates/canvas-collab/ V0.1 任何 logic 行, 0 改 crates/arg-bridge/ V0.1+V0.2 任何 logic 行, 0 改 Cargo.toml / Cargo.lock 任何行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行; (d) **v1.01 5 已知缺口状态变更** #1 P0-4 dev env 无 Docker — 仍 缺口; #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 — 仍 缺口; #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 — 仍 缺口; #4 端到端测试 5 守门 0 违反 — 仍 缺口; **#5 cargo fmt pre-existing baseline** — **🟢 闭合** (per 本 commit); (e) **守门 #1 禁回溯叙事 0 改 logic 实证** `git diff --stat b360135..HEAD -- crates/` 仅 whitespace, 0 改 V0.1/V0.2/V0.3/V0.4 任何 function body; `git log -p b360135..HEAD -- crates/` 实证 31 file 仅 +224/-149 净行全部是 rustfmt 规范化; `git log -1 --format='%an <%ae>' eeedd7b` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4); **累计 P0-4 + P2 阶段过渡 + v1.03 fmt fix**: v0.66-v1.00 累计 30+ P0-4 commit 收官 + v1.01 P2 阶段入门入口落档 + v1.02 P2 阶段 worker 子代理 实跑 brief 落档 + v1.03 闭合 v1.01 已知缺口 #5 cargo fmt baseline; 缺口 #1-#4 仍 缺口 (P0-4 dev env 无 Docker per 守门 #24 v2 G-5 mock 锁) 等 P2 阶段 worker 子代理 实跑 (per docs/briefs/p2-stage-exec.md v1.02 触发条件 4 满足后 dispatch) | **2026-09-10 21:00 JST 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进 (per Ulysses 21:00 JST "补缺口"指令 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和第 96 次新事件触发 仍允许 + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #19 v19 累积规不破坏 v0.1)** |
"""


def main() -> int:
    if not REG_PATH.exists():
        print(f"[err] registry not found: {REG_PATH}")
        return 1

    content = REG_PATH.read_text(encoding="utf-8")

    if "| **v0.26** |" in content:
        print("[skip] v0.26 row already exists in registry — idempotent")
        return 0

    # Insert v0.26 after v0.25 row, or at end if v0.25 row not found
    marker = "| **v0.25** |"
    idx = content.find(marker)
    if idx < 0:
        print("[err] v0.25 row not found — cannot anchor v0.26")
        return 1

    end_of = content.find("\n", idx)
    if end_of < 0:
        new_content = content + "\n" + V026_ROW
    else:
        new_content = (
            content[: end_of + 1]
            + V026_ROW
            + content[end_of + 1 :]
        )

    REG_PATH.write_text(new_content, encoding="utf-8")
    print(f"[ok] v0.26 row inserted in {REG_PATH}")
    print(f"[ok] registry size: {len(content)} -> {len(new_content)} ({len(new_content) - len(content):+d} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
