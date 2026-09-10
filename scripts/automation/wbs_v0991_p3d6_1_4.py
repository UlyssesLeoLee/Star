#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ docs 同步脚本
(per 守门 #1 禁回溯叙事 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v21 [P] docs 同步)
- WBS v0.99.1 row 追加 (v0.99 已被 P0-4 Stage 4.3 占用)
- registry v0.23 row 追加
- automation-design.md §4.34.3 追加
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")
REGISTRY_PATH = Path("scripts/automation/registry.md")
AUTOMATION_DESIGN_PATH = Path("docs/automation-design.md")

# ============================
# WBS v0.99.1 row
# ============================
WBS_V0991_ROW = """| **v0.99.1** | **2026-09-10 21:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 90 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ 2 新 module 骨架 落档 (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.4 + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱)** — worker 子代理在 worktree `D:/Star/.worktrees/wt-p3-d6-1-4-api-extend/` (branch `wt-p3-d6-1-4-api-extend` 基于 main `0165133` brief 落档 `docs/briefs/p3-d6-1-4-api-extend.md` 21.6KB 之后) 撰写, Mavis merge to main `2733c4d` — ⚠️ 跟 v0.92-v0.99 编号冲突 (v0.92 = P0-4 Stage 3.8 7 类 RLS policy 模板生成器, v0.93 = P0-4 Stage 3.9 #5 schema_isolation 修, v0.94 = P0-4 Stage 4.0 P3-D.6 14+15 张表 RLS 模板生成, v0.95 = P3-D.6 阶段 2 业务 任务 2.6 agent-domain 5 enum 强类型, v0.96 = P3-D.6 阶段 1 基础 任务 1.2 canvas UI sync bridge, v0.97 = P0-4 Stage 4.1 26 张表 DDL DROP IF EXISTS, v0.98 = P0-4 Stage 4.2 kubeseal CLI 集成, v0.99 = P0-4 Stage 4.3 P3-D.6 14+15 张表真实 schema 模板 per 1f977f1), per 守门 #1 禁回溯叙事 显式标 v0.99.1 区分**: (1) **commit `2733c4d` merge to main 落地** (worktree commit `948fcd6`): 13 files changed, +2521 insertions, **0 deletions** — (a) `crates/api/src/agent/{mod,controller,permission,audit,dto}.rs` 5 新增 110+474+222+117+401 = 1324 lines (AgentState 4 字段 + 13 REST endpoint 占位 handler + AgentPermission 5 守门 helpers + AgentAuditEvent 5 字段 + 13 DTO + 17 UT); (b) `crates/api/src/canvas_collab/{mod,controller,permission,audit,dto}.rs` 5 新增 123+264+237+137+418 = 1179 lines (CanvasCollabState 5 字段 + 5 REST endpoint 占位 + CanvasCollabPermission + PermissionLevel enum 3 变体 View/Comment/Edit + CanvasCollabAuditEvent 6 字段 + 5 DTO + 15 UT); (c) `crates/api/src/lib.rs` +10 lines (6 头注释 + 2 doc comment + 2 new `pub mod agent;` / `pub mod canvas_collab;`, 0 改现有 511 lines + 0 改 `pub mod arg;` 行); (d) `crates/api/Cargo.toml` +6 lines (4 注释 + 2 新 path 依赖 `star-agent-domain` + `canvas-collab`, 0 改现有 39 lines + 0 重复加 axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context 全部已在现有); (e) `Cargo.lock` +2 lines (cargo 自动, 0 手动编辑); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cargo check -p api --lib -j 4` = **0 err 0.23s**; `cargo test -p api --lib -j 4` = **67/67 PASS 0.00s** (V0.1 arg/ 33 + V0.2 34 new = 67, 0 regression, V0.1 全部不动); `cargo fmt -p api -- --check` = **0 diff**; `cargo clippy -p api --lib -j 4` = **0 warnings on my code** (api crate 1 warning 全部 pre-existing V0.1 lib.rs:110); `cargo check --workspace --lib -j 4` = 0 err (跟 V0.1 + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容); **(3) 关键设计 — 跟现有 arg/ V0.1 pattern 一致 0 改动** (per 守门 #19 v19 累积规不破坏 V0.1): 2 新 module 5 file 0 file 跟 arg/ 现有 5 file 1863 lines 同形 (mod + controller + permission + audit + dto, agent 无 sse_hub 留 WebSocket 走 BFF 任务 1.5); **(4) 实施计划"3 新" 实际"2 新" 缺口显式标** (per 守门 #11 缺标比错标): 实施计划 §3 任务 1.4 说 `crates/api/src/{agent,arg,canvas_collab}/` 3 新 module, 但 `arg/` V0.1 已存在 per 4813a53, 本任务实际 2 新 module (`agent/` + `canvas_collab/`), 0 plan 冲突; **(5) 守门合规** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 crates/api/src/arg/ 5 file 任何行, 0 改 crates/api/src/lib.rs 现有 511 lines 任何行, 0 改 crates/api/Cargo.toml 现有 39 lines 任何行, 0 改 crates/agent-domain/ V0.1+V0.2+V0.3 任何代码, 0 改 crates/canvas-collab/ V0.1 任何代码, 0 改 crates/arg-bridge/ V0.1 任何代码, 0 改根 Cargo.toml [workspace] members 任何行, 0 改 Cargo.lock 任何手编行; **(6) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-4-api-extend --force + git branch -D wt-p3-d6-1-4-api-extend` 0 残留; **(7) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + 任务 1.3 ✅ 收官 (canvas-collab/) + **任务 1.4 ✅ 收官 (api 2 新 module 2521 lines)** + 任务 1.5-1.7 跨 session 续做估 ~1.00M tokens / 0.83 SRE·周 (per 实施计划 §3 任务 1.5 BFF 0.2 + 任务 1.6 14+15 张表 0.3 + 任务 1.7 25 module 联动 0.2 + 留 0.3 buffer); **(8) 守门 #1 禁回溯叙事**: v0.1-v0.99 修订历史不动, v0.99.1 row 显式标 P3-D.6 阶段 1 基础 任务 1.4 收官; **(9) 触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化); P3-D.6 阶段 1 基础 任务 1.4 收官; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 21:00 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 90 次新事件触发 仍允许 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |
"""

# ============================
# Registry v0.23 row
# ============================
REGISTRY_V023_ROW = """| **v0.23** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (10 新 file + 2 改 file 落档, 索引同步) + §3 v0.23**: P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ 2 新 module 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化), 关联 commit `2733c4d` (merge to main, 13 files / 2521 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 90 次新事件触发 + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-4-api-extend.md` 21.6KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 实施计划"3 新" 实际"2 新" 显式标缺口 + 守门 #1 v25 cargo test 改单 crate 跳 workspace 67/67 PASS): worker 子代理 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-4-api-extend/` (branch `wt-p3-d6-1-4-api-extend` 基于 main `0165133` brief 落档 之后) 撰写, commit `948fcd6` 落档 (10 新 file: 5 agent/ + 5 canvas_collab/ + 2 改 file: lib.rs +10 lines + Cargo.toml +6 lines + 1 auto Cargo.lock +2 lines), 0 业务方法实装 0 WebSocket 0 OpenAPI 0 BFF 留 P3-D.6 阶段 2/3/4 续做, 4 守门实证 cargo check 0 err 0.23s + cargo test 67/67 PASS 0.00s (V0.1 arg/ 33 + V0.2 34 new = 67, 0 regression) + cargo fmt 0 diff + cargo clippy 0 warnings on my code (api crate 1 warning 全部 pre-existing V0.1 lib.rs:110); brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正 + v0.22 任务 1.3 brief 有 0 bug 相比, 任务 1.4 0 bug 是 v0.18/v0.21/v0.22 经验累积); Mavis merge to main `--no-ff` 0 conflict (main 已前进 1 commit [brief 0165133], 但 worktree 仅改 crates/api/src/ 新增 10 file + 2 改 现有 file); worktree cleanup `git worktree remove --force + git branch -D` 0 残留; **v0.23 编号** (v0.22 已被任务 1.3 占用, v0.18/v0.19/v0.20/v0.21/v0.22 都被阶段 1 基础 任务 1.1/1.2/1.3/阶段 2 业务 任务 2.6/任务 2.1 batch 1 平行工作占用, 用 v0.23 跳 v0.18-v0.22 平行工作模式); **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + 任务 1.3 ✅ 收官 (canvas-collab/ 5 域 Rust struct 骨架) + **任务 1.4 ✅ 收官 (api 2 新 module 2521 lines)** + 任务 1.5-1.7 跨 session 续做估 ~1.00M tokens / 0.83 SRE·周 | **2026-09-10 20:08 JST Ulysses 拍板"推进" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |
"""

# ============================
# automation-design.md §4.34.3
# ============================
AUTOMATION_4343 = """

### 4.34.3 P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ 2 新 module 骨架 (per 20:08 JST Ulysses 拍板"推进", 2026-09-10 21:00 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 per commit 64b96be 平行工作, §4.34.1 = P3-D.6 阶段 1 基础 任务 1.2 canvas UI sync bridge per commit f11517d 平行工作, §4.34.2 = P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ 5 域 Rust struct 骨架 per commit 2fd60b1 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.3 区分

> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 90 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 90 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改 crates/api/src/arg/ 5 file 任何行, 0 改 crates/api/src/lib.rs 现有 511 lines 任何行, 0 改 crates/api/Cargo.toml 现有 39 lines 任何行) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 arg/ 5 file + 0 改 V0.1 任何 crate) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-4-api-extend.md` 21.6KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 67/67 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `948fcd6` worktree + `2733c4d` merge) + 守门 #11 缺标比错标 (axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context 全部已在 crates/api/Cargo.toml 现有, 仅加 2 path 依赖 `star-agent-domain` + `canvas-collab`, 0 重复; 实施计划"3 新" 实际"2 新" 显式标缺口) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `2733c4d` merge to main, 13 files / 2521 insertions / 0 deletions):
> - `crates/api/src/agent/{mod,controller,permission,audit,dto}.rs` (5 新, 110+474+222+117+401 = 1324 lines, 17 UT)
>   - `mod.rs` (110 lines, AgentState 4 字段 audit/permission/tenant_id/actor_id + 5 ops placeholder 留阶段 2 业务实装 + new() 返回 Arc<Self> + build_router() + 2 UT)
>   - `controller.rs` (474 lines, 13 REST endpoint 占位 handler A1.1 4 + A2.1-A2.4 4 + A3.2 1 + A4.1-A4.2 2 + A6.1 2, handler body = 4 守门 check_tenant + require_role + write_event + Ok(Json(T::default())) + 1 UT)
>   - `permission.rs` (222 lines, AgentPermission 8 字段 + 5 守门 helpers check_tenant/require_role/require_any_role/has_role/is_platform_admin + 7 UT)
>   - `audit.rs` (117 lines, AgentAuditEvent 5 字段 id/agent_id/actor_id/action/at + write_event 占位 + 3 UT)
>   - `dto.rs` (401 lines, 13 Request/Response DTO + ApiError 5 变体 BadRequest/Forbidden/NotFound/Internal/Unimplemented + 4 UT)
> - `crates/api/src/canvas_collab/{mod,controller,permission,audit,dto}.rs` (5 新, 123+264+237+137+418 = 1179 lines, 15 UT)
>   - `mod.rs` (123 lines, CanvasCollabState 5 字段 audit/permission/tenant_id/actor_id/canvas_id + 2 UT)
>   - `controller.rs` (264 lines, 5 REST endpoint 占位 A12.1-A12.4 + A12.7, 0 WebSocket 留任务 1.5 BFF + 1 UT)
>   - `permission.rs` (237 lines, CanvasCollabPermission + PermissionLevel enum 3 变体 View/Comment/Edit + require(level) helper + rank/satisfies 派生方法 + 6 UT)
>   - `audit.rs` (137 lines, CanvasCollabAuditEvent 6 字段 多 1 metadata per A12.5 + write_event 占位 + 3 UT)
>   - `dto.rs` (418 lines, 5 Request/Response DTO + ApiError 5 变体 + 3 UT)
> - `crates/api/src/lib.rs` (+10 lines: 6 头注释 + 2 doc comment + 2 new `pub mod agent;` / `pub mod canvas_collab;`, 0 改现有 511 lines + 0 改 `pub mod arg;` 行)
> - `crates/api/Cargo.toml` (+6 lines: 4 注释 + 2 新 path 依赖 `star-agent-domain` + `canvas-collab`, 0 改现有 39 lines)
> - `Cargo.lock` (+2 lines, cargo 自动, 0 手动编辑)
> - `docs/briefs/p3-d6-1-4-api-extend.md` (21.6KB / 382 lines, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.11-1 | D5.11-1 | `crates/api/src/agent/{mod,controller,permission,audit,dto}.rs` 5 新增 (1324 lines, 17 UT) | A, R, S | **[P]** | (worker 子代理 Write tool) | API tier 13 REST endpoint 占位 + 4 守门 helpers, 0 业务方法 0 DB query 0 Memgraph adapter 调用 |
| D5.11-2 | D5.11-2 | `crates/api/src/canvas_collab/{mod,controller,permission,audit,dto}.rs` 5 新增 (1179 lines, 15 UT) | A, R, S | **[P]** | (worker 子代理 Write tool) | API tier 5 REST endpoint 占位 + PermissionLevel 3 变体 + 4 守门 helpers, 0 WebSocket 留任务 1.5 |
| D5.11-3 | D5.11-3 | `crates/api/src/lib.rs` +10 lines (6 头注释 + 2 doc + 2 new pub mod) | A | **[P]** | (worker 子代理 Edit tool) | 0 改现有 511 lines + 0 改 `pub mod arg;` 行 |
| D5.11-4 | D5.11-4 | `crates/api/Cargo.toml` +6 lines (4 注释 + 2 新 path deps) | A | **[P]** | (worker 子代理 Edit tool) | 0 改现有 39 lines + 0 重复加 axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context |
| D5.11-5 | D5.11-5 | `Cargo.lock` +2 lines (cargo 自动) | A | **[P]** | (cargo 自动) | 0 手动编辑, 0 改任何手编行 |
| D5.11-6 | D5.11-6 | 守门 #1 v25 实证 (worker 子代理 在 worktree 跑) | A, R | **[P]** | (cargo test 单 crate 跳 workspace) | `cargo check -p api --lib -j 4` = **0 err 0.23s**; `cargo test -p api --lib -j 4` = **67/67 PASS 0.00s** (V0.1 arg/ 33 + V0.2 34 new = 67, 0 regression); `cargo fmt -p api -- --check` = **0 diff**; `cargo clippy -p api --lib -j 4` = **0 warnings on my code** (api crate 1 warning 全部 pre-existing V0.1 lib.rs:110) |
| D5.11-7 | D5.11-7 | 跨 crate 守门 #1 v25 实证 | A, R | **[P]** | (cargo test 跨 crate) | `cargo check --workspace --lib -j 4` = 0 err (跟 V0.1 + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容) |
| D5.11-8 | D5.11-8 | worker 子代理 在 worktree 撰写, commit `948fcd6` 落档 (per 守门 #9 v27 3 段) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) |
| D5.11-9 | D5.11-9 | Mavis root merge to main | A | **[P]** | `git merge wt-p3-d6-1-4-api-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_4.tmp` | 3-way merge 0 conflict (main 已前进 1 commit [brief 0165133], 但 worktree 仅改 crates/api/src/ 新增 10 file + 2 改 现有 file). merge 后 `cargo test -p api --lib -j 4` 在 main 上 = **67/67 PASS 0.00s** (验证 merge 后实证) |
| D5.11-10 | D5.11-10 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove --force + git branch -D` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-4-api-extend/` removed + branch `wt-p3-d6-1-4-api-extend` deleted (was 948fcd6). 0 残留 |
| D5.11-11 | D5.11-11 | 实施计划"3 新" 实际"2 新" 缺口显式标 (per 守门 #11 缺标比错标) | A | **[P]** | (brief + commit message 显式标) | 实施计划 §3 任务 1.4 说 `crates/api/src/{agent,arg,canvas_collab}/` 3 新 module, 但 `arg/` V0.1 已存在 per 4813a53, 本任务实际 2 新 module (`agent/` + `canvas_collab/`), 0 plan 冲突 |
| D5.11-12 | D5.11-12 | 0 业务方法实装 (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标) | A | **[P]** | (handler body = 占位) | 13 + 5 = 18 REST endpoint handler body 全部占位 `Ok(Json(T::default()))`, 留 P3-D.6 阶段 2 任务 2.1-2.5 业务实装 |
| D5.11-13 | D5.11-13 | 0 WebSocket / 0 OpenAPI / 0 BFF 集成 (per 守门 #11 缺标比错标) | A | **[P]** | (0 加 utopa / 0 加 wss_hub) | 留 P3-D.6 阶段 3 集成 任务 3.2 (A12 WebSocket 4 端点 走 bff/src/collaboration/wss_hub.rs 任务 1.5) |
| D5.11-14 | D5.11-14 | `docs/automation-design.md` §4.34.3 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.11-15 | D5.11-15 | `scripts/automation/registry.md` §3 v0.23 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.23 修订历史 (21:00 JST task 1.4 收官) |
| D5.11-16 | D5.11-16 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.99.1 row | A | **[P]** | (本脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.4 api 2 新 module 落档 (v0.99.1 编号: v0.99 已被 P0-4 Stage 4.3 占用 per 1f977f1, 用 v0.99.1 跳 v0.99 平行工作模式) |

**§4.34.3 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.5-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 10 file + 10 lines lib.rs + 6 lines Cargo.toml + 2 lines Cargo.lock, 0 改现有 crate / 0 改现有 arg/)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 948fcd6 in worktree + merge commit 2733c4d) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.34.3 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/api/src/agent/mod.rs` 实证 v0.1 落档 (commit `2733c4d` merge to main, 110 lines)
- `git log -1 --format='%an <%ae>'` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p api --lib -j 4` 在 main 上 = **67/67 PASS 0.00s** (V0.1 arg/ 33 + V0.2 34 new = 67)
- `cargo check -p api --lib -j 4` 在 main 上 = **0 err 0.23s**
- `cargo check --workspace --lib -j 4` 在 main 上 = 0 err (跟 V0.1 + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容)
- worker 子代理 在 worktree 5 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err + cargo test 67/67 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings on my code + cargo check --workspace --lib -j 4 = 0 err
- `git log -p --follow docs/briefs/p3-d6-1-4-api-extend.md` 实证 brief 落档 (21.6KB / 382 lines, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告, 0 改 crates/api/src/arg/ 5 file 任何行
- 守门 #1 禁回溯叙事: 0 改 crates/api/src/arg/ 任何 file 任何行 (5 file 1863 lines 100% 保留), 0 改 crates/api/src/lib.rs 现有 511 lines 任何行, 0 改 crates/api/Cargo.toml 现有 39 lines 任何行, 0 改 crates/agent-domain/ V0.1+V0.2+V0.3 任何代码, 0 改 crates/canvas-collab/ V0.1 任何代码, 0 改 crates/arg-bridge/ V0.1 任何代码, 0 改根 Cargo.toml [workspace] members 任何行, 0 改 Cargo.lock 任何手编行
- 守门 #9 #3 0 散落子代理产出: worker 1 commit `948fcd6` 干净 + merge 1 commit `2733c4d` 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 20:08 JST 拍板"推进" → 21:00 JST brief commit 0165133 + worker 1 commit 948fcd6 + Mavis merge 2733c4d + docs sync 闭环, 全程 ~52 分钟
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-4-api-extend.md 21.6KB 在 worktree 撰写前已落档 (main `0165133`)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_1e3ea85a) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 67 PASS) → collect_output (本返报 8 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 8 依赖 (axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context) 已在 crates/api/Cargo.toml 现有, 仅加 2 path 依赖 `star-agent-domain` + `canvas-collab`, 0 重复; brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正 + v0.22 任务 1.3 brief 有 0 bug 相比, 任务 1.4 0 bug 是 v0.18/v0.21/v0.22 经验累积); 实施计划"3 新" 实际"2 新" 显式标缺口
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/api/src/agent/ 5 file + crates/api/src/canvas_collab/ 5 file + lib.rs +10 lines + Cargo.toml +6 lines + Cargo.lock +2 lines, 不影响 V0.1 game 5 份 PHASE 报告 + 不影响 V0.1 crates/api/src/arg/ 5 file 1863 lines + 不影响 V0.1 crates/agent-domain/ + V0.1 crates/canvas-collab/ + V0.1 crates/arg-bridge/)

**§4.34.3 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.18M tokens (worker 实装 0.13 + Mavis merge + verify + docs 0.05)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 1.4 + 阶段 2 业务 任务 2.6 + 任务 2.1 batch 1 21 commit: ~4.06M tokens (3.38 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.5-1.7 (剩余 3 任务: BFF + 14+15 张表 + 25 module 联动接口): ~1.00M tokens (0.83 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2-5 跨 session 续 (10 module 业务实装 + 12 项剩余业务方法) + 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.6M tokens (1.33 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)
"""


def main() -> int:
    # 1) WBS append
    wbs_text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.99.1**" in wbs_text:
        print("[skip] WBS v0.99.1 already present")
    else:
        if not wbs_text.endswith("\n"):
            wbs_text += "\n"
        wbs_text += WBS_V0991_ROW
        WBS_PATH.write_text(wbs_text, encoding="utf-8")
        print(f"[ok] WBS v0.99.1 appended ({len(WBS_V0991_ROW)} chars)")

    # 2) Registry append
    reg_text = REGISTRY_PATH.read_text(encoding="utf-8")
    if "**v0.23**" in reg_text:
        print("[skip] registry v0.23 already present")
    else:
        if not reg_text.endswith("\n"):
            reg_text += "\n"
        reg_text += REGISTRY_V023_ROW
        REGISTRY_PATH.write_text(reg_text, encoding="utf-8")
        print(f"[ok] registry v0.23 appended ({len(REGISTRY_V023_ROW)} chars)")

    # 3) automation-design.md §4.34.3 append
    auto_text = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    if "### 4.34.3" in auto_text:
        print("[skip] automation-design §4.34.3 already present")
    else:
        if not auto_text.endswith("\n"):
            auto_text += "\n"
        auto_text += AUTOMATION_4343
        AUTOMATION_DESIGN_PATH.write_text(auto_text, encoding="utf-8")
        print(f"[ok] automation-design §4.34.3 appended ({len(AUTOMATION_4343)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
