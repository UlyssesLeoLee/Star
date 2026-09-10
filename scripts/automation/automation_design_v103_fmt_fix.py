#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
automation-design.md §4.34.5 任务卡 insert (idempotent) for v1.03 cargo fmt baseline fix.

守门 #12 v21 [P] docs 同步必更新 §4 任务卡表.
"""
import io
import sys
from pathlib import Path

AD_PATH = Path("docs/automation-design.md")

V4345_SECTION = """
### §4.34.5 v1.03 cargo fmt baseline fix 任务卡 (2026-09-10 21:00 JST 落档)

**关联 commit**: `eeedd7b` (push origin `9d79eb4..eeedd7b main -> main`)
**关联 WBS row**: v1.03 (闭合 v1.01 已知缺口 #5 cargo fmt pre-existing baseline)
**关联 registry row**: v0.26
**本地 main 1 commit 落档** (0 worktree 散落, 0 误 add untracked file, 跟 v0.99.1 + v0.99.2 Mavis root session 1 commit 模式一致)

**任务 D5.34.5-1 ~ D5.34.5-5 (5 子项)**:

| 子项 | 任务 | 实证 | 守门 |
|---|---|---|---|
| **D5.34.5-1** | `cargo fmt --all` 跑 31 file 全部 whitespace 规范化 | 跑前 67 file 有 diff, 跑后 0 diff; 0 改 logic 0 改 function 0 改 struct, 仅 rustfmt 标准 whitespace (缩进 + 长行 wrap + trailing comma + 注释对齐) | #11 缺标比错标 闭合 v1.01 已知缺口 #5 |
| **D5.34.5-2** | 4 守门实证 (cargo fmt --check + cargo check + cargo test + cargo clippy) | `cargo fmt --all -- --check` = 0 diff + `cargo check --workspace --lib -j 4` = 0 err 5.12s + `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS 0.00s (跟 v0.82 16/16 + v0.85 +5 sqlx 真实 impl 0 regression) + `cargo clippy --workspace --lib -j 4` = 0 新 warning | #1 v19 + #1 v25 实证 |
| **D5.34.5-3** | 守门合规 7 维 (#1+#1 v15+#1 v19+#1 v25+#9 v19+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反 | 0 改 V0.1/V0.2/V0.3/V0.4 任何 logic 行, 0 改 crates/api/src/arg/ 5 file 任何 V0.1 1863 lines, 0 改 crates/agent-domain/ V0.1+V0.2+V0.3+V0.4 任何 logic 行, 0 改 crates/canvas-collab/ V0.1 任何 logic 行, 0 改 crates/arg-bridge/ V0.1+V0.2 任何 logic 行, 0 改 Cargo.toml / Cargo.lock 任何行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行 | 7 维全合规 |
| **D5.34.5-4** | v1.01 5 已知缺口状态变更 | #1 P0-4 dev env 无 Docker — 仍 缺口 (P0-4 dev env 限制, per 守门 #24 v2 G-5 mock 锁); #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 — 仍 缺口 (P2 阶段 worker 实跑补); #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 — 仍 缺口 (P2 阶段 worker 实跑补); #4 端到端测试 5 守门 0 违反 — 仍 缺口 (P2 阶段 worker 实跑补); **#5 cargo fmt pre-existing baseline — 🟢 闭合** (per 本 commit) | #11 缺标比错标 |
| **D5.34.5-5** | push origin 0 err | `git push origin main` = `9d79eb4..eeedd7b main -> main` 0 err | #9 v19 + #14 v4 |

**v1.01 5 已知缺口当前状态** (per 守门 #11 缺标比错标):
- #1 P0-4 dev env 无 Docker — 🔴 仍 缺口 (P2 阶段 worker 实跑触发, per 守门 #24 v2 G-5 mock 锁)
- #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 — 🔴 仍 缺口 (P2 阶段 worker 实跑补)
- #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 — 🔴 仍 缺口 (P2 阶段 worker 实跑补)
- #4 端到端测试 5 守门 0 违反 — 🔴 仍 缺口 (P2 阶段 worker 实跑补)
- #5 cargo fmt pre-existing baseline — **🟢 闭合** (per v1.03 commit `eeedd7b`)

**累计 P0-4 + P2 阶段过渡 + v1.03 fmt fix 收官** (per 守门 #9 v19 Mavis 自驱):
- v0.66-v1.00 累计 30+ P0-4 commit 收官
- v1.01 P2 阶段入门入口落档 (runbook + 验证脚本)
- v1.02 P2 阶段 worker 子代理 实跑 brief 落档
- v1.03 闭合 v1.01 已知缺口 #5 cargo fmt baseline

**守门合规 7 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反

**触发原因**: per Ulysses 2026-09-10 21:00 JST "补缺口"指令 + 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进 (per 守门 #11 缺标比错标 闭合已知缺口 #5)

**commit author=Ulysses** (per 守门 #10 + 守门 #14 v4)
"""


def main() -> int:
    if not AD_PATH.exists():
        print(f"[err] automation-design.md not found: {AD_PATH}")
        return 1

    content = AD_PATH.read_text(encoding="utf-8")

    if "§4.34.5" in content:
        print("[skip] §4.34.5 already exists in automation-design.md — idempotent")
        return 0

    if not content.endswith("\n"):
        new_content = content + "\n" + V4345_SECTION
    else:
        new_content = content + V4345_SECTION

    AD_PATH.write_text(new_content, encoding="utf-8")
    print(f"[ok] §4.34.5 inserted in {AD_PATH}")
    print(f"[ok] automation-design.md size: {len(content)} -> {len(new_content)} ({len(new_content) - len(content):+d} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
