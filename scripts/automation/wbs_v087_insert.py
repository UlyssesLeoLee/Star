#!/usr/bin/env python3
"""
WBS v0.87 + §4.33 + registry v0.18 追加脚本 (per 守门 #12 v21 [P] docs 同步).

触发: 2026-09-10 19:50 JST P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架
       merge to main (commit eb73e0d) 后 docs 同步.
范围: §4.33 task card + WBS v0.87 row + registry v0.18 row + brief catch-up.
"""
import sys
from pathlib import Path

REPO_ROOT = Path("D:/Star")

# ===== §4.33 task card =====
SECTION_4_33 = """


### 4.33 P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main", 2026-09-10 19:50 JST)

> **触发**: 2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 84 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 84 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (不重写 V0.1 任何代码, 不动现有 crates/ 任何子目录, 0 业务逻辑 0 API 0 endpoint 0 schema 留阶段 2/3 实装) + 守门 #1 v19 累积规 (不破坏 V0.1, 守门 #19 v19 V0.1 game 5 份 PHASE 报告 0 重写) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-1-agent-domain.md` 12.9KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 2/2 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `87f8109d5056735e7f8ba996b5cea1ca8b10b529`) + 守门 #11 缺标比错标 (chrono/serde/serde_json/uuid 已在根 Cargo.toml 现有, 0 重复) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `eb73e0d` merge to main, 6 files / 141 insertions / 0 deletions):
> - `crates/agent-domain/Cargo.toml` (230 bytes, 新增, [package] + [dependencies] serde + uuid workspace 引用)
> - `crates/agent-domain/src/lib.rs` (696 bytes, 新增, module-level doc + `pub mod models;` + `pub use models::agent::AgentNode;` + `#![warn(missing_docs)]`)
> - `crates/agent-domain/src/models/mod.rs` (60 bytes, 新增, `pub mod agent;`)
> - `crates/agent-domain/src/models/agent.rs` (3,146 bytes, 新增, `AgentNode` struct 14 字段 + `Default` impl + 2 UT, 0 业务方法)
> - 根 `Cargo.toml` (+2 lines: 1 注释 + 1 member `crates/agent-domain` 加入 [workspace] members)
> - `Cargo.lock` (+10 lines, cargo 自动, 0 手动编辑)
> - `docs/briefs/p3-d6-1-1-agent-domain.md` (12.9KB, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.8-1 | D5.8-1 | `crates/agent-domain/Cargo.toml` v0.1 (230 bytes, 新增) | A, R | **[P]** | (worker 子代理 Write tool) | [package] name=agent-domain version=0.1.0 edition=2021 + [dependencies] serde+uuid workspace 引用, 0 业务依赖 (tokio/async-trait/sqlx 留阶段 2) |
| D5.8-2 | D5.8-2 | `crates/agent-domain/src/lib.rs` v0.1 (696 bytes, 新增) | A, R | **[P]** | (worker 子代理 Write tool) | module-level doc + `pub mod models;` + `pub use models::agent::AgentNode;` + `#![warn(missing_docs)]`; 0 业务函数, 留阶段 2 |
| D5.8-3 | D5.8-3 | `crates/agent-domain/src/models/mod.rs` v0.1 (60 bytes, 新增) | A | **[P]** | (worker 子代理 Write tool) | `pub mod agent;` |
| D5.8-4 | D5.8-4 | `crates/agent-domain/src/models/agent.rs` v0.1 (3,146 bytes, 新增) | A, R, S | **[P]** | (worker 子代理 Write tool) | `AgentNode` struct 14 字段 (id/name/archetype/domain/status/trust_score/metadata/created_at/updated_at/version/x/y/width/height/rotation) + `Default` impl + 2 UT (test_agent_node_default + test_agent_node_clone). **brief 模板 1 微 bug 修正**: `AgentNode` derive 去掉 `Eq` (因 `f32` 字段不实现 `Eq`, 守门 #1 禁回溯叙事不受影响, 0 行为变化). 0 业务方法 (move/resize/rotate/handoff 等 A1-A10 28 项 留阶段 2 任务 2.1) |
| D5.8-5 | D5.8-5 | 根 `Cargo.toml` +2 lines (members + 1 注释 + 1 member) | A | **[P]** | (worker 子代理 Read + Write, 避免 Edit tool exact match 难题) | `[workspace] members` 加 1 成员 `crates/agent-domain`; `[workspace.dependencies]` **0 重复加** (chrono/serde/serde_json/uuid 全部已在 line 129-143 现有, per 守门 #11 缺标比错标) |
| D5.8-6 | D5.8-6 | `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑) | A | **[P]** | (cargo 自动) | 新增 `agent-domain` 包 + 4 依赖 (chrono/serde/serde_json/uuid) |
| D5.8-7 | D5.8-7 | `docs/briefs/p3-d6-1-1-agent-domain.md` (12.9KB, brief 落档) | A | **[P]** | (Write tool) | per 守门 #9 v20 子代理 dispatch 必先 brief 落档, brief 含 6 段 (§1 范围 + §2 落地清单 + §3 守门合规 + §4 返报要求 + §5 拍板来源 + §6 后续步骤) + 9 段返报要求 (per 守门 #9 v27 3 段 fallback) |
| D5.8-8 | D5.8-8 | worker 子代理 在 worktree `wt-p3-d6-1-1-agent-domain` (基于 main `bd1e8fe`) 撰写, commit `87f8109` 落档 (6 files / +141 / 0 deletions) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) + 守门 #9 v27 3 段 fallback (invoke → verify → collect_output) |
| D5.8-9 | D5.8-9 | Mavis root merge to main (commit `eb73e0d`) | A | **[P]** | `git merge wt-p3-d6-1-1-agent-domain --no-ff` | 3-way merge 0 conflict (main 已前进 5 commit, 但 worktree 仅改 crates/agent-domain/ 新增 + Cargo.toml members +1, 无冲突). 0 改 V0.1 现有代码 |
| D5.8-10 | D5.8-10 | cargo check + cargo test 实证 在 main 上 | A, R | **[P]** | (cargo test -p agent-domain --lib -j 4) | ✅ cargo check 0 err 0.41s + cargo test 2/2 PASS 0.10s (test_agent_node_default + test_agent_node_clone) |
| D5.8-11 | D5.8-11 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove + git branch -d` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain` removed + branch `wt-p3-d6-1-1-agent-domain` deleted (was 87f8109). 0 残留. |
| D5.8-12 | D5.8-12 | `docs/automation-design.md` §4.33 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.8-13 | D5.8-13 | `scripts/automation/registry.md` §3 v0.18 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.18 修订历史 (19:40 JST worker 子代理 + 19:50 JST merge to main) |
| D5.8-14 | D5.8-14 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.87 row | A | **[P]** | (Python 脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 worker 子代理 撰写 + merge to main 落档 |

**§4.33 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.2-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 `crates/agent-domain/` 目录 + 4 新文件 + 根 Cargo.toml +2 行 + Cargo.lock +10 行, 0 改现有 crate)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 87f8109 in worktree + commit eb73e0d merge to main) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.33 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/agent-domain/Cargo.toml` 实证 v0.1 落档 (commit `eb73e0d` merge to main, 230 bytes)
- `git log -p --follow crates/agent-domain/src/lib.rs` 实证 v0.1 落档 (696 bytes, module-level doc + pub mod models + pub use AgentNode)
- `git log -p --follow crates/agent-domain/src/models/agent.rs` 实证 v0.1 落档 (3,146 bytes, AgentNode 14 字段 + Default + 2 UT)
- `git log -p --follow crates/agent-domain/src/models/mod.rs` 实证 v0.1 落档 (60 bytes, pub mod agent)
- `git log -1 --format='%an <%ae>' eb73e0d` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p agent-domain --lib -j 4` 在 main 上 = **2/2 PASS 0.10s** (test_agent_node_default + test_agent_node_clone)
- `cargo check -p agent-domain --lib -j 4` 在 main 上 = **0 err 0.41s**
- worker 子代理 在 worktree 4 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err 0.36s + cargo test 2/2 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings
- `git log -p --follow docs/briefs/p3-d6-1-1-agent-domain.md` 实证 brief 落档 (12.9KB, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改现有 crates/ 任何子目录, 0 改 Cargo.lock 手动, 0 改 docs 现有 (除 docs/briefs/ 新增 + 现有 docs 0 改)
- 守门 #9 #3 0 散落子代理产出: worker 1 commit 87f8109 干净 + merge 1 commit eb73e0d 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 19:40 JST 拍板 → 19:42 JST brief 落档 → 19:44 JST worker commit 87f8109 → 19:46 JST merge to main eb73e0d → 19:50 JST docs 同步, 全程 ~10 分钟
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-1-agent-domain.md 12.9KB 在 worktree 撰写前已落档 (main `bd1e8fe` 时)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_e6569dbb) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 2 PASS) → collect_output (本返报 9 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 4 依赖 (chrono/serde/serde_json/uuid) 已在根 Cargo.toml 现有, 0 重复; brief 模板 1 微 bug (AgentNode derive Eq) 显式记录 + 修正
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/agent-domain/ 骨架不影响 V0.1 game 5 份 PHASE 报告)

**§4.33 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.10M tokens (worker 实装 0.08 + Mavis merge + verify + docs 0.02)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 15 commit: ~3.58M tokens (2.98 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.2-1.7 (剩余 6 任务): ~1.40M tokens (1.17 SRE·周)
- 后续 P3-D.6 阶段 2 业务 (A1-A10 + A11 + A12 + G1-G12): ~2.0M tokens (1.67 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段 (阶段 1 基础 7 任务 + 阶段 2 业务 8 任务 + 阶段 3 集成 4 任务 + 阶段 4 实装 6 任务): ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)"""


# ===== WBS v0.87 row =====
WBS_V087_ROW = """| **v0.87** | **2026-09-10 19:50 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 审核决策 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 84 次新事件触发 仍允许)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + WBS v0.86 §3 阶段 1 基础 任务 1.1 实施计划) — worker 子代理在 worktree D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain/ (branch wt-p3-d6-1-1-agent-domain 基于 main bd1e8fe) 撰写, Mavis merge to main**：(1) commit `87f8109` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-1-agent-domain.md` 12.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标): 6 files changed, +141 insertions, **0 deletions** — (a) `crates/agent-domain/Cargo.toml` 230 bytes 新增 ([package] + [dependencies] serde+uuid workspace 引用); (b) `crates/agent-domain/src/lib.rs` 696 bytes 新增 (module-level doc + `pub mod models;` + `pub use models::agent::AgentNode;` + `#![warn(missing_docs)]`); (c) `crates/agent-domain/src/models/mod.rs` 60 bytes 新增 (`pub mod agent;`); (d) `crates/agent-domain/src/models/agent.rs` 3,146 bytes 新增 (`AgentNode` struct 14 字段: id/name/archetype/domain/status/trust_score/metadata/created_at/updated_at/version/x/y/width/height/rotation + `Default` impl + 2 UT `test_agent_node_default` + `test_agent_node_clone`); (e) 根 `Cargo.toml` +2 lines (1 注释 + 1 member `crates/agent-domain`); (f) `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cargo check -p agent-domain --lib -j 4` = **0 err 0.36s**; `cargo test -p agent-domain --lib -j 4` = **2/2 PASS 0.00s**; `cargo fmt -p agent-domain -- --check` = **0 diff** (cargo fmt 自动修 1 处空格后); `cargo clippy -p agent-domain --lib -j 4` = **0 warnings** (advisory per 守门 #7 v3); **(3) brief 模板 1 微 bug 修正** (per 守门 #1 禁回溯叙事, 显式记录): `AgentNode` derive 去掉 `Eq` (因 `f32` 字段不实现 `Eq` per `rustc --explain E0277`), 保留 `PartialEq`. 0 业务方法, 0 行为变化, 2/2 UT 仍 PASS. **(4) Mavis merge to main** (per 19:40 JST Ulysses 拍板"完成后合并到 main"): `git merge wt-p3-d6-1-1-agent-domain --no-ff -F .git/MERGE_MSG_P3_D6_1_1.tmp` = **merge commit `eb73e0d` 0 conflict** (main 已前进 5 commit `6212ab1` `7f2dd64` `1d06f70` `ffdda56` `132d946`, 但 worktree 仅改 crates/agent-domain/ 新增 + Cargo.toml members +1, 无冲突). merge 后 `cargo test -p agent-domain --lib -j 4` 在 main 上 = **2/2 PASS 0.10s** (验证 merge 后实证); **(5) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-1-agent-domain` + `git branch -d wt-p3-d6-1-1-agent-domain` (was 87f8109) → **0 残留**; **(6) 阶段 1 基础 任务 1.1 收官** (per WBS v0.86 §3 阶段 1 基础 任务 1.1): 6 新 crate 中第 1 个 `agent-domain` 落地, 0 业务方法 0 API 0 endpoint 0 schema 留阶段 2 业务 + 阶段 3 集成 + 阶段 4 实装; **(7) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 + 任务 1.2-1.7 (剩余 6 任务: arg + arg-bridge + arg-effect + canvas-collab + api 扩展 + 1 BFF + 14+15 张表 + 25 module 联动接口) 跨 session 续做, 估 ~1.40M tokens / 1.17 SRE·周; **(8) 守门 #1 禁回溯叙事**: 0 改 V0.1 任何代码, 0 改现有 crates/ 任何子目录, 0 改 docs 现有 (除 docs/briefs/ 新增 + 现有 docs 0 改), 0 改 Cargo.lock 手动; **(9) 守门合规**: #1+#1 v15+#1 v19+#1 v25+#5+#6+#7+#9+#9 v19+#9 v20+#9 v27+#10+#11+#13+#14 v2+#14 v3+#14 v4+#19 v19 全部 0 违反; **(10) 触发**: 9/10 19:40 JST 用户发令"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令 + 9/1 14:58 JST 守门"拍板决策必 ask_user 给选项" + 9/8 16:08 JST 守门"必带推荐项" + 9/5 04:03 JST 守门"拍板推荐项直接执行"); **merge commit `eb73e0d` 落地, 0 子代理 RPC 重试 (per 守门 #9 实证 #7)**, worktree 撰写 → main merge 全程 ~10 分钟; commit author=Ulysses (per 守门 #10 + 守门 #14 v4); 1 任务 1 commit 1 merge 1 docs sync 收官 | 2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 84 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |"""


# ===== registry v0.18 row =====
V018_ROW = """| **v0.18** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (1 新 crate 落档, 索引同步) + §3 v0.18**: P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main"), 关联 commit `eb73e0d` (merge to main, 6 files / 141 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 84 次新事件触发 + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-1-agent-domain.md` 12.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 跳 workspace 2/2 PASS): worker 子代理 bg_e6569dbb 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain/` (branch `wt-p3-d6-1-1-agent-domain` 基于 main `bd1e8fe`) 撰写, commit `87f8109` 落档 (AgentNode struct 14 字段 + Default + 2 UT), 0 业务方法 0 API 0 endpoint 0 schema 留阶段 2/3/4 实装, 4 守门实证 cargo check 0 err + cargo test 2/2 PASS + cargo fmt 0 diff + cargo clippy 0 warnings, brief 模板 1 微 bug 修正 AgentNode derive 去掉 Eq (f32 不实现 Eq) 显式记录; Mavis merge to main `--no-ff` 0 conflict (main 已前进 5 commit, 但 worktree 仅改 crates/agent-domain/ 新增 + Cargo.toml members +1); worktree cleanup `git worktree remove + git branch -d` 0 残留; **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 + 任务 1.2-1.7 跨 session 续做估 ~1.40M tokens / 1.17 SRE·周 | **2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和) |"""


def main() -> int:
    """Append §4.33 + WBS v0.87 row + registry v0.18 row."""
    # 1. Append §4.33 to automation-design.md
    auto_path = REPO_ROOT / "docs/automation-design.md"
    auto_content = auto_path.read_text(encoding="utf-8")
    print(f"Read {len(auto_content)} bytes from {auto_path.name}")

    if "### 4.33 P3-D.6 阶段 1 基础 任务 1.1" in auto_content:
        print(f"  {auto_path.name}: §4.33 already exists, skipping")
    else:
        if not auto_content.endswith("\n"):
            auto_content = auto_content + "\n"
        new_content = auto_content + SECTION_4_33 + "\n"
        auto_path.write_text(new_content, encoding="utf-8")
        print(f"  Appended §4.33 ({len(SECTION_4_33)} chars)")

    # 2. Append v0.87 row to WBS
    wbs_path = REPO_ROOT / "docs/reports/STAR-P3-WBS-001.md"
    wbs_content = wbs_path.read_text(encoding="utf-8")
    print(f"Read {len(wbs_content)} bytes from {wbs_path.name}")

    if "**v0.87**" in wbs_content:
        print(f"  WBS: v0.87 already exists, skipping")
    else:
        if not wbs_content.endswith("\n"):
            wbs_content = wbs_content + "\n"
        new_wbs = wbs_content + WBS_V087_ROW + "\n"
        wbs_path.write_text(new_wbs, encoding="utf-8")
        print(f"  Appended v0.87 row ({len(WBS_V087_ROW)} chars)")

    # 3. Append v0.18 row to registry
    reg_path = REPO_ROOT / "scripts/automation/registry.md"
    reg_content = reg_path.read_text(encoding="utf-8")
    print(f"Read {len(reg_content)} bytes from {reg_path.name}")

    if "**v0.18**" in reg_content:
        print(f"  registry: v0.18 already exists, skipping")
    else:
        if not reg_content.endswith("\n"):
            reg_content = reg_content + "\n"
        new_reg = reg_content + V018_ROW + "\n"
        reg_path.write_text(new_reg, encoding="utf-8")
        print(f"  Appended v0.18 row ({len(V018_ROW)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
