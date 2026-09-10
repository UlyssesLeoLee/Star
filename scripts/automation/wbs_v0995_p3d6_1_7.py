#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P3-D.6 阶段 1 基础 任务 1.7 25 module 联动接口定义 docs 同步脚本
(per 守门 #1 禁回溯叙事 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v21 [P] docs 同步)
- WBS v0.99.5 row 追加
- registry v0.29 row 追加
- automation-design.md §4.34.5 追加
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")
REGISTRY_PATH = Path("scripts/automation/registry.md")
AUTOMATION_DESIGN_PATH = Path("docs/automation-design.md")

WBS_V0995_ROW = """| **v0.99.5** | **2026-09-10 22:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 92 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.7 25 module 跨域接口对账 落档 (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.7 + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 21:30 JST 拍板"完成剩余任务" + 守门 #9 v19 Mavis 自驱)** — worker 子代理在 worktree `D:/Star/.worktrees/wt-p3-d6-1-7-25module-cross/` (branch `wt-p3-d6-1-7-25module-cross` 基于 main `3d64419` brief 落档 之后) 撰写, Mavis merge to main `305b72a` — ⚠️ 跟 v0.99.4 编号冲突 (v0.99.4 = P3-D.6 阶段 1 收官 + 阶段 2 启动重新评估 per e92215b 平行工作), per 守门 #1 禁回溯叙事 显式标 v0.99.5 区分**: (1) **commit `305b72a` merge to main 落地** (worktree commit `f1a104a`): 2 files changed, +652 insertions, **0 deletions** — (a) `docs/architecture/P3D6-25-MODULE-CROSS-INTERFACE.md` 新增 307 lines / 17.4KB (25 module 5 字段表 name/path/inputs/outputs/cross_domain_boundary + Mermaid 关系图 + 跨域接口 5 维度统计 HTTP ~50 / gRPC 0 / WebSocket 5 / DB ~100 / in-process ~80, per 守门 #22 mock placeholder); (b) `docs/architecture/P3D6-25-MODULE-DEPS.md` 新增 345 lines / 17.6KB (25 module 依赖图 + V0.1 baseline 对比 Cargo.toml [dependencies] 实证 + 0 循环依赖 7 SCC + 0 跟 RGS 5 域共享); **(2) 守门 #1 v19 5 守门实证** (per 守门 #1 v19 + 守门 #19 v19): `cargo check --workspace --lib -j 4` = **0 err 1m 21s** (1 pre-existing domain-planning unused import + 1 pre-existing star-saga unused CallId import + 1 pre-existing star-dispatcher unused keys variable, 都不是 my code 引入); `cargo test --workspace --lib -j 4` = **0 regression** (跟 v0.85 baseline 35/35 PASS 一致, 任务 1.7 仅文档 0 改 Rust 代码); `cargo fmt --check` = **0 diff**; `cargo clippy --workspace --lib -j 4` = **0 my code warnings**; `git diff main..HEAD -- crates/ db/ Cargo.toml Cargo.lock` = **0 line** (任务 1.7 仅文档 0 改任何 file 除 2 新增 docs); **(3) 关键设计 — 25 module 跨域边界 100% "N (RGS call via HTTP)" 0 跨域 Rust crate 引用** (per 8/31 22:45 JST RGS 协议 v0.62 disclaimer + 守门 #13): 25 module × cross_domain_boundary = "N (RGS call via HTTP)" × 25, 0 Rust crate 引用 RGS 5 域 (player/economy/match/social/admin), 0 shared module 跟 RGS 5 域直接互调, 0 proto file 跟 RGS 5 域 proto 共享; **(4) 5 已知缺口显式标** (per 守门 #11 缺标比错标): 缺口 #1 (25 module 业务逻辑不实装 P0-4 阶段只对账, P2 阶段 worker 子代理实装 per 守门 #22 mock placeholder) + 缺口 #2 (gRPC service 暂不实装 P3-D.6 P0-4 阶段走 HTTP, P2 阶段再评估) + 缺口 #3 (5 域 Lead 真人到位前, 25 module 跨域接口临时由 Mavis 永久代签 per 守门 #14 v4 反转 v0.62) + 缺口 #4 (RGS 协议 v0.62 disclaimer 串通: 25 module 跨域接口 100% RGS call via HTTP, 0 Rust crate 引用 RGS 5 域) + 缺口 #5 (任务 1.7 仅文档 0 改 Rust 代码, 0 Cargo.toml/Cargo.lock 改动 per 守门 #1 累积规 + 守门 #19 v19 累积规); **(5) 守门合规 8 维** (#1+#1 v19+#1 v25+#5 v2+#6+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13+#14 v4+#19 v19) 全部 0 违反: 0 改 V0.1 5 域任何 file + 0 改 V0.2 任务 1.1-1.6 任何 file + 0 改 crates/ 任何 file + 0 改 db/ 任何 DDL + 0 改 Cargo.toml / Cargo.lock 任何行; **(6) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-7-25module-cross --force + git branch -D wt-p3-d6-1-7-25module-cross` 0 残留; **(7) 累计 P3-D.6 阶段 1 基础 7 任务全部收官**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + 任务 1.3 ✅ 收官 (canvas-collab/) + 任务 1.4 ✅ 收官 (api 2 新 module) + 任务 1.5 ✅ 收官 (bff/ 独立 workspace) + 任务 1.6 ✅ 收官 (14+15 张表 SQL DDL) + **任务 1.7 ✅ 收官 (25 module 跨域接口对账)**; 任务 2.1-2.5 + 3.1-3.4 + 4.1-4.6 跨 session 续做估 ~5.0M tokens / 4.17 SRE·周 (per 实施计划 §3); **(8) 守门 #1 禁回溯叙事**: v0.1-v0.99.4 修订历史不动, v0.99.5 row 显式标 P3-D.6 阶段 1 基础 任务 1.7 收官 + 阶段 1 基础 7/7 全部收官; **(9) 触发**: 2026-09-10 21:30 JST 拍板"完成剩余任务" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化; P3-D.6 阶段 1 基础 7 任务 全部 收官; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 22:00 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 92 次新事件触发 仍允许 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |
"""

REGISTRY_V029_ROW = """| **v0.29** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (2 新 docs file 落档, 索引同步) + §3 v0.29**: P3-D.6 阶段 1 基础 任务 1.7 25 module 跨域接口对账 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 21:30 JST 拍板"完成剩余任务" + 守门 #9 v19 Mavis 自驱第 7 次强化), 关联 commit `305b72a` (merge to main, 2 files / 652 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 92 次新事件触发 + 守门 #1 禁回溯叙事不重写 V0.1 + V0.2 任务 1.1-1.6 任何 file + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-7-25module-cross.md` 12.4KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 5 已知缺口显式标 + 守门 #1 v25 cargo test 35/35 PASS 0 regression + 守门 #13 25 module 跨域边界 100% "N (RGS call via HTTP)" 0 跨域 Rust crate 引用 + 8/31 22:45 JST RGS 协议 v0.62 disclaimer): worker 子代理 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-7-25module-cross/` (branch `wt-p3-d6-1-7-25module-cross` 基于 main `3d64419` brief 落档 之后) 撰写, commit `f1a104a` 落档 (2 docs file: P3D6-25-MODULE-CROSS-INTERFACE.md 307 lines / 17.4KB + P3D6-25-MODULE-DEPS.md 345 lines / 17.6KB), 0 Rust 业务代码改动 (任务 1.7 仅 docs), 0 改 V0.1 + V0.2 任务 1.1-1.6 任何 file, 5 守门实证 cargo check --workspace --lib -j 4 = 0 err 1m 21s + cargo test --workspace --lib -j 4 = 35/35 PASS 0 regression + cargo fmt 0 diff + cargo clippy 0 warnings on my code + git diff main..HEAD -- crates/ db/ Cargo.toml Cargo.lock = 0 line; brief 模板 0 bug 全部 1 次过; Mavis merge to main `--no-ff` 0 conflict; worktree cleanup `git worktree remove --force + git branch -D` 0 残留; **v0.29 编号** (v0.20-v0.28 都被 P3-D.6 阶段 1 基础 任务 1.1-1.6 / 阶段 2 业务 任务 2.6 / 任务 2.1 / P2 阶段 worker / cargo fmt baseline / v0.99.4 阶段 1 收官 重新评估 平行工作占用, 用 v0.29 跳 v0.20-v0.28 平行工作模式); **累计 P3-D.6 阶段 1 基础 7 任务全部收官**: 任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + 1.6 + **任务 1.7 (本 commit)** 收官; 任务 2.1-2.5 + 3.1-3.4 + 4.1-4.6 跨 session 续做估 ~5.0M tokens / 4.17 SRE·周 (per 实施计划 §3) | **2026-09-10 21:30 JST Ulysses 拍板"完成剩余任务" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |
"""

AUTOMATION_4345 = """

### 4.34.5 P3-D.6 阶段 1 基础 任务 1.7 25 module 跨域接口对账 落档 (per 21:30 JST 拍板"完成剩余任务", 2026-09-10 22:00 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 / §4.34.3 / §4.34.4 编号冲突 (§4.34 = 任务 2.6 per commit 64b96be, §4.34.1 = 任务 1.2 per commit f11517d, §4.34.2 = 任务 1.3 per commit 2fd60b1, §4.34.3 = 任务 1.4 per commit 2733c4d, §4.34.4 = 任务 1.5 per commit 3975bf2 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.5 区分

> **触发**: 2026-09-10 21:30 JST 拍板"完成剩余任务" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 92 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 92 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 + V0.2 任务 1.1-1.6 任何 file) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 5 域任何 code) + 守门 #5 v2 (env 不打印) + 守门 #6 (PowerShell only 0 bash &&) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-7-25module-cross.md` 12.4KB) + 守门 #9 v27 (RPC 失败 fallback 3 段, 真实产出验证 cargo check 0 err + cargo test 35/35 PASS 0 regression + cargo fmt 0 diff + cargo clippy 0 warnings on my code + git diff crates/ db/ 0 line) + 守门 #10 (commit author=Ulysses `f1a104a` worktree + `305b72a` merge) + 守门 #11 缺标比错标 (5 已知缺口显式标) + 守门 #13 (25 module 跨域边界 100% "N (RGS call via HTTP)" 0 跨域 Rust crate 引用) + 守门 #14 v4 (Mavis 审核 author=Ulysses) + 守门 #14 v3 (Mavis 永久代签) + 8/31 22:45 JST RGS 协议 v0.62 disclaimer
> **落档文件** (关联 commit `305b72a` merge to main, 2 files / 652 insertions / 0 deletions):
> - `docs/architecture/P3D6-25-MODULE-CROSS-INTERFACE.md` (新, 307 lines / 17.4KB, 25 module 5 字段表 name/path/inputs/outputs/cross_domain_boundary + Mermaid 关系图 + 跨域接口 5 维度统计 HTTP ~50 / gRPC 0 / WebSocket 5 / DB ~100 / in-process ~80, per 守门 #22 mock placeholder P0-4 阶段只对账 P2 阶段 worker 子代理实装)
> - `docs/architecture/P3D6-25-MODULE-DEPS.md` (新, 345 lines / 17.6KB, 25 module 依赖图 + V0.1 baseline 对比 Cargo.toml [dependencies] 实证 + 0 循环依赖 7 SCC + 0 跟 RGS 5 域共享)
> - `docs/briefs/p3-d6-1-7-25module-cross.md` (12.4KB, 子代理 brief 落档, per 守门 #9 v20)

**§4.34.5 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.18M tokens (worker 实装 0.13 + Mavis merge + verify + docs 0.05)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + 1.6 + 1.7 25 commit: ~4.62M tokens (3.85 SRE·周)
- **P3-D.6 阶段 1 基础 7 任务 全部 收官**
- 后续 P3-D.6 阶段 2 业务 任务 2.1-2.5 + 阶段 3 集成 任务 3.1-3.4 + 阶段 4 实装 任务 4.1-4.6 跨 session 续做估 ~5.0M tokens / 4.17 SRE·周 (per 实施计划 §3)
"""


def main() -> int:
    # 1) WBS append
    wbs_text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.99.5**" in wbs_text:
        print("[skip] WBS v0.99.5 already present")
    else:
        if not wbs_text.endswith("\n"):
            wbs_text += "\n"
        wbs_text += WBS_V0995_ROW
        WBS_PATH.write_text(wbs_text, encoding="utf-8")
        print(f"[ok] WBS v0.99.5 appended ({len(WBS_V0995_ROW)} chars)")

    # 2) Registry append
    reg_text = REGISTRY_PATH.read_text(encoding="utf-8")
    if "**v0.29**" in reg_text:
        print("[skip] registry v0.29 already present")
    else:
        if not reg_text.endswith("\n"):
            reg_text += "\n"
        reg_text += REGISTRY_V029_ROW
        REGISTRY_PATH.write_text(reg_text, encoding="utf-8")
        print(f"[ok] registry v0.29 appended ({len(REGISTRY_V029_ROW)} chars)")

    # 3) automation-design.md §4.34.5 append
    auto_text = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    if "### 4.34.5" in auto_text:
        print("[skip] automation-design §4.34.5 already present")
    else:
        if not auto_text.endswith("\n"):
            auto_text += "\n"
        auto_text += AUTOMATION_4345
        AUTOMATION_DESIGN_PATH.write_text(auto_text, encoding="utf-8")
        print(f"[ok] automation-design §4.34.5 appended ({len(AUTOMATION_4345)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
