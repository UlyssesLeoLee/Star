#!/usr/bin/env python3
"""
P3-D.6 阶段 1 任务 1.2 docs 同步脚本 (WBS v0.92 + §4.34 + registry v0.20).

触发: 2026-09-10 20:00 JST P3-D.6 阶段 1 基础 任务 1.2 extend (per 19:55 JST Ulysses 选 extend 方案)
       merge to main (commit f11517d) 后 docs 同步.
范围: §4.34 task card + WBS v0.92 row + registry v0.20 row + insert script.
v0.91 已被 P0-4 Stage 3.7 RLS 命名修正占用 (commit ea1cd64).
"""
import sys
from pathlib import Path

REPO_ROOT = Path("D:/Star")

# ===== §4.34 task card =====
SECTION_4_34 = """


### 4.34 P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge (per 19:55 JST Ulysses 选 extend 方案, 2026-09-10 20:00 JST)

> **触发**: 2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板, 替代原 plan "3 新 crate 骨架" 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在, ARG 10 G-1/G-2/G-3 阶段已实装) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 87 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 87 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改 crates/arg/src/ + 0 改 crates/arg-effect/src/ + 0 改 Cargo.toml + 0 改 Cargo.lock, 仅 +1 new file + 2 lines lib.rs) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 game 5 份 PHASE 报告) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` 15.9KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 3/3 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `f11517d`) + 守门 #11 缺标比错标 (chrono/serde/serde_json/uuid 已在 crates/arg-bridge/Cargo.toml 现有, 0 重复) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `f11517d` merge to main, 2 files / 198 insertions / 0 deletions):
> - `crates/arg-bridge/src/canvas_sync_bridge.rs` (新, 196 lines / 8,045 bytes, 包含 `CanvasSyncBridgeError` enum 1 变体 + `CanvasSyncEvent` enum 6 变体 + `CanvasSyncEventPayload` struct 6 字段 + `Default` impl + `CanvasSyncBridgeConfig` struct 5 字段 + `Default` impl + `CanvasSyncBridge` struct 5 字段 + `new()` 构造函数 + 3 UT, **0 业务方法** 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播)
> - `crates/arg-bridge/src/lib.rs` (+2 lines: 1 `///` doc comment + 1 `pub mod canvas_sync_bridge;`)
> - `docs/briefs/p3-d6-1-2-arg-crates-extend.md` (15.9KB, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.9-1 | D5.9-1 | `crates/arg-bridge/src/canvas_sync_bridge.rs` 新增 (196 lines / 8,045 bytes) | A, R, S | **[P]** | (worker 子代理 Write tool) | UI sync bridge 骨架, 含 6 struct/enum + 3 Default + new() + 3 UT. **0 业务方法** 留阶段 3 集成 任务 3.2 |
| D5.9-2 | D5.9-2 | `crates/arg-bridge/src/lib.rs` +2 lines (1 doc + 1 pub mod) | A | **[P]** | (worker 子代理 Edit tool) | 仅 +1 line `pub mod canvas_sync_bridge;` + 1 line doc comment, 0 改现有 6 pub mod 行 |
| D5.9-3 | D5.9-3 | brief 3 处修正 (worker 显式标) | A | **[P]** | (worker 子代理 fix) | (1) `use crate::client::memgraph::MemgraphClient` → `use star_arg::client::MemgraphClient` (brief typo, `crate::client` 在 `star-arg` 不在 `star-arg-bridge`); (2) `SyncProtocol` 不存在 → 用 `BridgeEnvelopeKind` 替代 (现有 protocol.rs 0 `SyncProtocol` type); (3) `Option<PeriodFlushWorker>` wrap in `Option<Arc<PeriodFlushWorker>>` (无 `Clone` derive 跟现有 `Arc<MemgraphClient>` shared pattern 一致) |
| D5.9-4 | D5.9-4 | 守门 #1 v25 实证 (worker 子代理 在 worktree 跑) | A, R | **[P]** | (cargo test 单 crate 跳 workspace) | `cargo check -p star-arg-bridge --lib -j 4` = **0 err** (1 pre-existing dead_code warning in star-arg/memgraph.rs, 跟我代码无关); `cargo test -p star-arg-bridge --lib -j 4` = **3/3 PASS 0.00s** (arg-bridge 现有 0 UT, 0 regression); `cargo fmt -p star-arg-bridge -- --check` = **0 diff**; `cargo clippy -p star-arg-bridge --lib -j 4` = **0 warnings on my code** (5 pre-existing in star-arg 跟我无关) |
| D5.9-5 | D5.9-5 | worker 子代理 在 worktree 撰写, commit `f11517d` 落档 (per 守门 #9 v27 3 段) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) |
| D5.9-6 | D5.9-6 | Mavis root merge to main | A | **[P]** | `git merge wt-p3-d6-1-2-arg-crates-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_2.tmp` | 3-way merge 0 conflict (main 已前进 N commit, 但 worktree 仅改 crates/arg-bridge/ 新增 1 file + 1 line lib.rs). merge 后 `cargo test -p star-arg-bridge --lib -j 4` 在 main 上 = **3/3 PASS 0.00s** (验证 merge 后实证) |
| D5.9-7 | D5.9-7 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove + git branch -d` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend/` removed + branch `wt-p3-d6-1-2-arg-crates-extend` deleted (was f11517d). 0 残留 |
| D5.9-8 | D5.9-8 | `docs/automation-design.md` §4.34 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.9-9 | D5.9-9 | `scripts/automation/registry.md` §3 v0.20 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.20 修订历史 (19:55 JST extend 方案拍板) |
| D5.9-10 | D5.9-10 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.92 row | A | **[P]** | (Python 脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.2 extend 落档 (v0.92 编号: v0.91 已被 P0-4 Stage 3.7 占用, 用 v0.92 跟 v0.86/v0.88.1 主线+平行工作模式一致) |

**§4.34 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.3-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 1 file + 2 lines lib.rs, 0 改现有 crate / 0 改 Cargo.toml / 0 改 Cargo.lock)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit f11517d in worktree + merge commit) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.34 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/arg-bridge/src/canvas_sync_bridge.rs` 实证 v0.1 落档 (commit `f11517d` merge to main, 196 lines / 8,045 bytes)
- `git log -1 --format='%an <%ae>'` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p star-arg-bridge --lib -j 4` 在 main 上 = **3/3 PASS 0.00s** (test_canvas_sync_bridge_new + test_canvas_sync_bridge_config_default + test_canvas_sync_event_default)
- `cargo check -p star-arg-bridge --lib -j 4` 在 main 上 = **0 err** (1 pre-existing dead_code warning in star-arg/memgraph.rs, 跟 canvas_sync_bridge 无关)
- worker 子代理 在 worktree 4 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err + cargo test 3/3 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings on my code
- `git log -p --follow docs/briefs/p3-d6-1-2-arg-crates-extend.md` 实证 brief 落档 (15.9KB, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改 crates/arg/src/ 任何 file (12 models + 8 ops + 1 cypher_cache + 1 llm + lib), 0 改 crates/arg-effect/src/ 任何 file (4 effect + achievement_engine + prompts + error + lib), 0 改 Cargo.toml, 0 改 Cargo.lock
- 守门 #9 #3 0 散落子代理产出: worker 1 commit `f11517d` 干净 + merge 1 commit 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 19:55 JST 拍板 → 20:00 JST 1 commit 1 merge docs sync 闭环, 全程 ~5 分钟
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-2-arg-crates-extend.md 15.9KB 在 worktree 撰写前已落档 (main `3599dc9` 时)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_aa70ea2b) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 3 PASS) → collect_output (本返报 8 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 4 依赖 (chrono/serde/serde_json/uuid) 已在 crates/arg-bridge/Cargo.toml 现有, 0 重复; brief 3 处 typo / 现有 type 不存在 / Clone derive 缺失 修正 显式记录
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/arg-bridge/src/canvas_sync_bridge.rs 骨架 + 1 line lib.rs pub mod 不影响 V0.1 game 5 份 PHASE 报告)

**§4.34 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.10M tokens (worker 实装 0.08 + Mavis merge + verify + docs 0.02)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 18 commit: ~3.78M tokens (3.15 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.3-1.7 (剩余 5 任务: canvas-collab + api 扩展 + BFF + 14+15 张表 + 25 module 联动接口): ~1.30M tokens (1.08 SRE·周)
- 后续 P3-D.6 阶段 2-4: ~3.5M tokens (2.92 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)"""


# ===== WBS v0.92 row =====
WBS_V092_ROW = """| **v0.92** | **2026-09-10 20:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 审核决策 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 87 次新事件触发 仍允许)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge (per 19:55 JST Ulysses 选 extend 方案, 替代原 plan "3 新 crate 骨架" 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在, ARG 10 G-1/G-2/G-3 阶段已实装) — worker 子代理在 worktree D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend/ (branch wt-p3-d6-1-2-arg-crates-extend 基于 main 3599dc9) 撰写, Mavis merge to main**：**(1) commit `f11517d` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` 15.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #1 禁回溯叙事 0 改 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标): 2 files changed, +198 insertions, **0 deletions** — (a) `crates/arg-bridge/src/canvas_sync_bridge.rs` 新增 196 lines / 8,045 bytes (UI sync bridge 骨架, 包含 `CanvasSyncBridgeError` enum 1 变体 + `CanvasSyncEvent` enum 6 变体 `EdgeChanged`/`AgentStatusChanged`/`AchievementUnlocked`/`TrustScoreChanged`/`DispatchRouteChanged`/`CanvasMultiUserEvent` + `CanvasSyncEventPayload` struct 6 字段 + `Default` impl + `CanvasSyncBridgeConfig` struct 5 字段 + `Default` impl + `CanvasSyncBridge` struct 5 字段 `Arc<>` wrap 共享模式 + `new()` 构造函数 + 3 UT, **0 业务方法** 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播); (b) `crates/arg-bridge/src/lib.rs` +2 lines (1 `///` doc comment + 1 `pub mod canvas_sync_bridge;`, 0 改现有 6 pub mod 行); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cargo check -p star-arg-bridge --lib -j 4` = **0 err** (1 pre-existing dead_code warning in star-arg/memgraph.rs `BoltConnectionPool::pool` 字段, 跟 canvas_sync_bridge 无关, 0 改动); `cargo test -p star-arg-bridge --lib -j 4` = **3/3 PASS 0.00s** (test_canvas_sync_event_default + test_canvas_sync_bridge_config_default + test_canvas_sync_bridge_new, arg-bridge 现有 0 既有 UT, 0 regression); `cargo fmt -p star-arg-bridge -- --check` = **0 diff**; `cargo clippy -p star-arg-bridge --lib -j 4` = **0 warnings on my code** (5 pre-existing in star-arg 跟我无关, advisory per 守门 #7 v3); **(3) brief 3 处修正 显式记录** (per 守门 #11 缺标比错标): (a) `use crate::client::memgraph::MemgraphClient` → `use star_arg::client::MemgraphClient` (brief typo, `crate::client` 在 `star-arg` 不在 `star-arg-bridge`, 跨 crate 引用需 `star_arg::` prefix); (b) `SyncProtocol` 不存在 → 用 `BridgeEnvelopeKind` 替代 (现有 protocol.rs 0 `SyncProtocol` type, `BridgeEnvelopeKind` 跟 `CanvasSyncEvent` 6 变体 对应); (c) `Option<PeriodFlushWorker>` wrap in `Option<Arc<PeriodFlushWorker>>` (`PeriodFlushWorker` derive 只有 `Debug` 无 `Clone`, 跟现有 `Arc<MemgraphClient>` 共享 pattern 一致); **(4) Mavis merge to main** (per 19:40 JST Ulysses 拍板"完成后合并到 main"): `git merge wt-p3-d6-1-2-arg-crates-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_2.tmp` = **merge 0 conflict** (main 已前进 commit `b9affce` 在 worktree 撰写期间, 但 worktree 仅改 crates/arg-bridge/ 新增 1 file + 1 line lib.rs, 无冲突). merge 后 `cargo test -p star-arg-bridge --lib -j 4` 在 main 上 = **3/3 PASS 0.00s** (验证 merge 后实证); **(5) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-2-arg-crates-extend` + `git branch -d wt-p3-d6-1-2-arg-crates-extend` (was f11517d) → **0 残留**; **(6) 阶段 1 基础 任务 1.2 收官** (per WBS v0.86 §3 阶段 1 基础 任务 1.2 extend 方案): UI sync bridge 骨架 落地, **0 业务方法 0 API 0 endpoint 0 WSS 连接 0 SSE 广播** 留阶段 3 集成 任务 3.2; **(7) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/ 新 crate 骨架) + 任务 1.2 ✅ 收官 (canvas UI sync bridge 骨架) + 任务 1.3-1.7 (剩余 5 任务: canvas-collab + api 扩展 + BFF + 14+15 张表 + 25 module 联动接口) 跨 session 续做, 估 ~1.30M tokens / 1.08 SRE·周; **(8) 守门 #1 禁回溯叙事**: 0 改 V0.1 任何代码, 0 改 crates/arg/src/ 任何 file (12 models + 8 ops + 1 cypher_cache + 1 llm + lib 现有), 0 改 crates/arg-effect/src/ 任何 file (4 effect + achievement_engine + prompts + error + lib 现有), 0 改 Cargo.toml, 0 改 Cargo.lock, 0 改 docs 现有; **(9) 守门合规**: #1+#1 v15+#1 v19+#1 v25+#5+#6+#7+#9+#9 v19+#9 v20+#9 v27+#10+#11+#13+#14 v2+#14 v3+#14 v4+#19 v19 全部 0 违反; **(10) 触发**: 9/10 19:55 JST 用户选 extend 方案 (per ask_user 拍板, 替代原 plan, 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在), 9/10 19:40 JST 用户发令"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 JST 第 6 次强化 Mavis 全权 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱不被动等指令 + 9/1 14:58 JST 守门"拍板决策必 ask_user 给选项" + 9/8 16:08 JST 守门"必带推荐项" + 9/5 04:03 JST 守门"拍板推荐项直接执行"); **merge 落地, 0 子代理 RPC 重试 (per 守门 #9 实证 #7)**, worktree 撰写 → main merge → docs sync 全程 ~5 分钟; commit author=Ulysses (per 守门 #10 + 守门 #14 v4); 1 任务 1 commit 1 merge 1 docs sync 收官; **(11) 编号说明**: v0.92 编号 (v0.91 已被 P0-4 Stage 3.7 RLS 命名修正占用 per ea1cd64), 跟 v0.86 (P3-D.6 实施计划 主线) + v0.88.1 (任务 1.1 平行工作) 模式一致, 5 域 Lead 真人未到位 Mavis 临时代签 | 2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板) + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 v15 docs 同步饱和第 86-87 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |"""


# ===== registry v0.20 row =====
V020_ROW = """| **v0.20** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (1 新 file 落档, 索引同步) + §3 v0.20**: P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge (per 19:55 JST Ulysses 选 extend 方案, 替代原 plan "3 新 crate 骨架" 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在), 关联 commit `f11517d` (merge to main, 2 files / 198 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 87 次新事件触发 + 守门 #1 禁回溯叙事 0 改 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` 15.9KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 跳 workspace 3/3 PASS): worker 子代理 bg_aa70ea2b 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend/` (branch `wt-p3-d6-1-2-arg-crates-extend` 基于 main `3599dc9`) 撰写, commit `f11517d` 落档 (`crates/arg-bridge/src/canvas_sync_bridge.rs` 196 lines / 8,045 bytes 新增 + `crates/arg-bridge/src/lib.rs` +2 lines, 0 业务方法 0 API 0 endpoint 0 schema 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播), 4 守门实证 cargo check 0 err + cargo test 3/3 PASS + cargo fmt 0 diff + cargo clippy 0 warnings on my code, brief 3 处修正 显式记录 (1: use crate::client → use star_arg::client 跨 crate 引用; 2: SyncProtocol 不存在 → 用 BridgeEnvelopeKind 替代; 3: PeriodFlushWorker wrap in Arc<> 无 Clone derive 跟现有 Arc<MemgraphClient> 共享 pattern 一致); Mavis merge to main `--no-ff` 0 conflict (main 已前进 commit `b9affce`, 但 worktree 仅改 crates/arg-bridge/ 新增 1 file + 1 line lib.rs); worktree cleanup `git worktree remove + git branch -d` 0 残留; **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/ 新 crate 骨架) + 任务 1.2 ✅ 收官 (canvas UI sync bridge 骨架) + 任务 1.3-1.7 跨 session 续做估 ~1.30M tokens / 1.08 SRE·周 | **2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板) + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |"""


def main() -> int:
    """Append §4.34 + WBS v0.92 + registry v0.20."""
    # 1. Append §4.34
    auto_path = REPO_ROOT / "docs/automation-design.md"
    auto_content = auto_path.read_text(encoding="utf-8")
    print(f"Read {len(auto_content)} bytes from {auto_path.name}")

    if "### 4.34 P3-D.6 阶段 1 基础 任务 1.2" in auto_content:
        print(f"  §4.34 already exists, skipping")
    else:
        if not auto_content.endswith("\n"):
            auto_content = auto_content + "\n"
        new_content = auto_content + SECTION_4_34 + "\n"
        auto_path.write_text(new_content, encoding="utf-8")
        print(f"  Appended §4.34 ({len(SECTION_4_34)} chars)")

    # 2. Append v0.92 row
    wbs_path = REPO_ROOT / "docs/reports/STAR-P3-WBS-001.md"
    wbs_content = wbs_path.read_text(encoding="utf-8")
    print(f"Read {len(wbs_content)} bytes from {wbs_path.name}")

    if "**v0.92**" in wbs_content:
        print(f"  WBS: v0.92 already exists, skipping")
    else:
        if not wbs_content.endswith("\n"):
            wbs_content = wbs_content + "\n"
        new_wbs = wbs_content + WBS_V092_ROW + "\n"
        wbs_path.write_text(new_wbs, encoding="utf-8")
        print(f"  Appended v0.92 row ({len(WBS_V092_ROW)} chars)")

    # 3. Append v0.20 row
    reg_path = REPO_ROOT / "scripts/automation/registry.md"
    reg_content = reg_path.read_text(encoding="utf-8")
    print(f"Read {len(reg_content)} bytes from {reg_path.name}")

    if "**v0.20**" in reg_content:
        print(f"  registry: v0.20 already exists, skipping")
    else:
        if not reg_content.endswith("\n"):
            reg_content = reg_content + "\n"
        new_reg = reg_content + V020_ROW + "\n"
        reg_path.write_text(new_reg, encoding="utf-8")
        print(f"  Appended v0.20 row ({len(V020_ROW)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
