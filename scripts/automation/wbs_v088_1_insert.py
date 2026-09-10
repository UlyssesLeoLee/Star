#!/usr/bin/env python3
"""
WBS v0.88.1 + registry v0.19 追加脚本 (per 守门 #12 v21 [P] docs 同步).

触发: 2026-09-10 19:55 JST P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架
       merge to main (commit eb73e0d) 后 docs 同步.
范围: WBS v0.88.1 row 显式标 v0.88 编号冲突 + registry v0.19 row + §4.33 task card.
v0.87 已被 P0-4 Stage 3.3 RLS 占用 (commit 53364ac), 用 v0.88.1 跟 v0.80.1/v0.81.1/v0.82.1/v0.89.1 平行工作编号一致.
"""
import sys
from pathlib import Path

REPO_ROOT = Path("D:/Star")

# ===== WBS v0.88.1 row =====
WBS_V088_1_ROW = """| **v0.88.1** | **2026-09-10 19:55 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 审核决策 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 85 次新事件触发 仍允许 + 守门 #1 禁回溯叙事)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + WBS v0.86 §3 阶段 1 基础 任务 1.1 实施计划) — worker 子代理在 worktree D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain/ (branch wt-p3-d6-1-1-agent-domain 基于 main bd1e8fe) 撰写, Mavis merge to main — ⚠️ 跟 v0.87/v0.88 编号冲突 (v0.87 = P0-4 Stage 3.3 RLS per commit 53364ac, v0.88 = P0-4 Stage 3.4 PLATFORM_ADMIN per commit bd1e8fe, per 守门 #1 禁回溯叙事) 显式标 v0.88.1 区分**: (1) **commit `87f8109` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-1-agent-domain.md` 12.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标): 6 files changed, +141 insertions, **0 deletions** — (a) `crates/agent-domain/Cargo.toml` 230 bytes 新增 ([package] + [dependencies] serde+uuid workspace 引用); (b) `crates/agent-domain/src/lib.rs` 696 bytes 新增 (module-level doc + `pub mod models;` + `pub use models::agent::AgentNode;` + `#![warn(missing_docs)]`); (c) `crates/agent-domain/src/models/mod.rs` 60 bytes 新增 (`pub mod agent;`); (d) `crates/agent-domain/src/models/agent.rs` 3,146 bytes 新增 (`AgentNode` struct 14 字段: id/name/archetype/domain/status/trust_score/metadata/created_at/updated_at/version/x/y/width/height/rotation + `Default` impl + 2 UT `test_agent_node_default` + `test_agent_node_clone`); (e) 根 `Cargo.toml` +2 lines (1 注释 + 1 member `crates/agent-domain`); (f) `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cargo check -p agent-domain --lib -j 4` = **0 err 0.36s**; `cargo test -p agent-domain --lib -j 4` = **2/2 PASS 0.00s**; `cargo fmt -p agent-domain -- --check` = **0 diff** (cargo fmt 自动修 1 处空格后); `cargo clippy -p agent-domain --lib -j 4` = **0 warnings** (advisory per 守门 #7 v3); **(3) brief 模板 1 微 bug 修正** (per 守门 #1 禁回溯叙事, 显式记录): `AgentNode` derive 去掉 `Eq` (因 `f32` 字段不实现 `Eq` per `rustc --explain E0277`), 保留 `PartialEq`. 0 业务方法, 0 行为变化, 2/2 UT 仍 PASS. **(4) Mavis merge to main** (per 19:40 JST Ulysses 拍板"完成后合并到 main"): `git merge wt-p3-d6-1-1-agent-domain --no-ff -F .git/MERGE_MSG_P3_D6_1_1.tmp` = **merge commit `eb73e0d` 0 conflict** (main 已前进 5 commit `6212ab1` `7f2dd64` `1d06f70` `ffdda56` `132d946` 在 worktree 撰写期间, 但 worktree 仅改 crates/agent-domain/ 新增 + Cargo.toml members +1, 无冲突). merge 后 `cargo test -p agent-domain --lib -j 4` 在 main 上 = **2/2 PASS 0.10s** (验证 merge 后实证); **(5) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-1-agent-domain` + `git branch -d wt-p3-d6-1-1-agent-domain` (was 87f8109) → **0 残留**; **(6) 阶段 1 基础 任务 1.1 收官** (per WBS v0.86 §3 阶段 1 基础 任务 1.1): 6 新 crate 中第 1 个 `agent-domain` 落地, 0 业务方法 0 API 0 endpoint 0 schema 留阶段 2 业务 + 阶段 3 集成 + 阶段 4 实装; **(7) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 + 任务 1.2-1.7 (剩余 6 任务: arg + arg-bridge + arg-effect + canvas-collab + api 扩展 + 1 BFF + 14+15 张表 + 25 module 联动接口) 跨 session 续做, 估 ~1.40M tokens / 1.17 SRE·周; **(8) 守门 #1 禁回溯叙事**: 0 改 V0.1 任何代码, 0 改现有 crates/ 任何子目录, 0 改 docs 现有 (除 docs/briefs/ 新增 + 现有 docs 0 改), 0 改 Cargo.lock 手动; **(9) 守门合规**: #1+#1 v15+#1 v19+#1 v25+#5+#6+#7+#9+#9 v19+#9 v20+#9 v27+#10+#11+#13+#14 v2+#14 v3+#14 v4+#19 v19 全部 0 违反; **(10) 触发**: 9/10 19:40 JST 用户发令"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令 + 9/1 14:58 JST 守门"拍板决策必 ask_user 给选项" + 9/8 16:08 JST 守门"必带推荐项" + 9/5 04:03 JST 守门"拍板推荐项直接执行"); **merge commit `eb73e0d` 落地, 0 子代理 RPC 重试 (per 守门 #9 实证 #7)**, worktree 撰写 → main merge 全程 ~10 分钟; commit author=Ulysses (per 守门 #10 + 守门 #14 v4); 1 任务 1 commit 1 merge 1 docs sync 收官; **(11) 编号说明**: v0.87/v0.88/v0.89/v0.89.1/v0.90 都被 P0-4 Stage 3.3-3.6 平行工作占用 (per 53364ac/bd1e8fe/1d06f70/132d946/6212ab1), 本 P3-D.6 阶段 1 基础 任务 1.1 用 v0.88.1 平行工作编号 (跟 v0.80.1/v0.81.1/v0.82.1/v0.89.1 模式一致, per 守门 #1 禁回溯叙事 显式标第二份 v0.XX 编号冲突), 跟 v0.86 P3-D.6 实施计划 主线 (commit 5e361da, 19:18 JST 拍板) 平行, 5 域 Lead 真人未到位 Mavis 临时代签 | 2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 84-85 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |"""


# ===== registry v0.19 row =====
V019_ROW = """| **v0.19** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (1 新 crate 落档, 索引同步) + §3 v0.19**: P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + WBS v0.88.1 row 编号), 关联 commit `eb73e0d` (merge to main, 6 files / 141 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 85 次新事件触发 + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-1-agent-domain.md` 12.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 跳 workspace 2/2 PASS): worker 子代理 bg_e6569dbb 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain/` (branch `wt-p3-d6-1-1-agent-domain` 基于 main `bd1e8fe`) 撰写, commit `87f8109` 落档 (AgentNode struct 14 字段 + Default + 2 UT), 0 业务方法 0 API 0 endpoint 0 schema 留阶段 2/3/4 实装, 4 守门实证 cargo check 0 err + cargo test 2/2 PASS + cargo fmt 0 diff + cargo clippy 0 warnings, brief 模板 1 微 bug 修正 AgentNode derive 去掉 Eq (f32 不实现 Eq) 显式记录; Mavis merge to main `--no-ff` 0 conflict (main 已前进 5 commit, 但 worktree 仅改 crates/agent-domain/ 新增 + Cargo.toml members +1); worktree cleanup `git worktree remove + git branch -d` 0 残留; **v0.88.1 编号** (跟 v0.80.1/v0.81.1/v0.82.1/v0.89.1 平行工作模式一致, v0.87/v0.88/v0.89/v0.89.1/v0.90 都被 P0-4 Stage 3.3-3.6 平行工作占用, per 守门 #1 禁回溯叙事); **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 + 任务 1.2-1.7 跨 session 续做估 ~1.40M tokens / 1.17 SRE·周 | **2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |"""


def main() -> int:
    """Append v0.88.1 to WBS + v0.19 to registry."""
    # 1. Append v0.88.1 row to WBS
    wbs_path = REPO_ROOT / "docs/reports/STAR-P3-WBS-001.md"
    wbs_content = wbs_path.read_text(encoding="utf-8")
    print(f"Read {len(wbs_content)} bytes from {wbs_path.name}")

    if "**v0.88.1**" in wbs_content:
        print(f"  WBS: v0.88.1 already exists, skipping")
    else:
        if not wbs_content.endswith("\n"):
            wbs_content = wbs_content + "\n"
        new_wbs = wbs_content + WBS_V088_1_ROW + "\n"
        wbs_path.write_text(new_wbs, encoding="utf-8")
        print(f"  Appended v0.88.1 row ({len(WBS_V088_1_ROW)} chars)")

    # 2. Append v0.19 row to registry
    reg_path = REPO_ROOT / "scripts/automation/registry.md"
    reg_content = reg_path.read_text(encoding="utf-8")
    print(f"Read {len(reg_content)} bytes from {reg_path.name}")

    if "**v0.19**" in reg_content:
        print(f"  registry: v0.19 already exists, skipping")
    else:
        if not reg_content.endswith("\n"):
            reg_content = reg_content + "\n"
        new_reg = reg_content + V019_ROW + "\n"
        reg_path.write_text(new_reg, encoding="utf-8")
        print(f"  Appended v0.19 row ({len(V019_ROW)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
