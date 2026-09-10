#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ docs 同步脚本
(per 守门 #1 禁回溯叙事 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v21 [P] docs 同步)
- WBS v0.99 row 追加
- registry v0.22 row 追加
- automation-design.md §4.34.2 追加
"""
import io
import sys
from pathlib import Path

WBS_PATH = Path("docs/reports/STAR-P3-WBS-001.md")
REGISTRY_PATH = Path("scripts/automation/registry.md")
AUTOMATION_DESIGN_PATH = Path("docs/automation-design.md")

# ============================
# WBS v0.99 row
# ============================
WBS_V099_ROW = """| **v0.99** | **2026-09-10 20:55 JST** | **架构师 (Mavis 接手 agent per DEC-008) ⇆ Mavis 审核决定 author=Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 89 次新事件触发 仍允许 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规不破坏 V0.1)** | **§14.20 P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ 新 crate 骨架 落档 (per P3-D.6 实施计划 §3 阶段 1 基础 任务 1.3 + 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 守门 #9 v19 Mavis 自驱)** — worker 子代理在 worktree `D:/Star/.worktrees/wt-p3-d6-1-3-canvas-collab/` (branch `wt-p3-d6-1-3-canvas-collab` 基于 main `1a88e658` 任务 1.2 brief catch-up `8272931` 之后) 撰写, Mavis merge to main — ⚠️ 跟 v0.92-v0.98 编号冲突 (v0.92 = P0-4 Stage 3.8 7 类 RLS policy 模板生成器, v0.93 = P0-4 Stage 3.9 #5 schema_isolation 修, v0.94 = P0-4 Stage 4.0 P3-D.6 14+15 张表 RLS 模板生成, v0.95 = P3-D.6 阶段 2 业务 任务 2.6 agent-domain 5 enum 强类型 + Agent struct, v0.96 = P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge, v0.97 = P0-4 Stage 4.1 26 张表 DDL DROP IF EXISTS idempotent 加, v0.98 = P0-4 Stage 4.2 kubeseal CLI 集成), per 守门 #1 禁回溯叙事 显式标 v0.99 区分**: (1) **commit `2fd60b1` merge to main 落地** (worktree commit `5815b8c`): 8 files changed, +397 insertions, **0 deletions** — (a) `crates/canvas-collab/Cargo.toml` 230 bytes 新增 ([package] name=canvas-collab version=0.1.0 + [dependencies] serde+uuid+chrono+serde_json workspace 引用); (b) `crates/canvas-collab/src/lib.rs` 新增 (module-level doc + `pub mod models;` + 5 struct 全部 pub use); (c) `crates/canvas-collab/src/models/mod.rs` 新增 (5 mod pub); (d-f) `crates/canvas-collab/src/models/{element,presence,comment,permission,audit}.rs` 5 新增 5 域 Rust struct: 1) `CanvasElementBackend` struct 12 字段 (id/canvas_id/element_type/x/y/width/height/z_index/created_by/created_at/updated_at/rotation) + `Default` impl + 2 UT (test_canvas_element_backend_default + test_canvas_element_backend_clone); 2) `PresenceCursor` struct 8 字段 (id/user_id/canvas_id/x/y/color/last_active/created_at) + `Default` impl + 1 UT; 3) `CanvasComment` struct 9 字段 (id/canvas_id/element_id/parent_comment_id/content/created_by/created_at/updated_at/resolved) + `Default` impl + 1 UT; 4) `CanvasPermission` struct 7 字段 (id/canvas_id/user_id/permission_level/granted_by/granted_at/expires_at) + `Default` impl + 1 UT + `PermissionLevel` enum 3 variants (View / Comment / Edit); 5) `CanvasMultiUserAudit` struct 9 字段 (id/canvas_id/actor_id/action/target_id/before_state/after_state/created_at/metadata) + `Default` impl + 1 UT; (g) 根 `Cargo.toml` +2 lines (1 注释 + 1 member `crates/canvas-collab`); (h) `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo test 改单 crate, 跳 workspace): `cargo check -p canvas-collab --lib -j 4` = **0 err 0.62s**; `cargo test -p canvas-collab --lib -j 4` = **6/6 PASS 0.00s** (test_canvas_element_backend_default + test_canvas_element_backend_clone + test_presence_cursor_default + test_canvas_comment_default + test_canvas_permission_default + test_canvas_multi_user_audit_default, 0 regression, V0.1 全部不动); `cargo fmt -p canvas-collab -- --check` = **0 diff**; `cargo clippy -p canvas-collab --lib -j 4` = **0 warnings**; `cargo check --workspace --lib -j 4` = 0 err (跟 1.1 + 1.2 merge 后兼容); **(3) 关键设计 — 5 域 Rust struct 命名跟 v0.63 协调性检查报告一致** (per 协调性检查 v0.63 反转 C-25 CanvasElementsBackend → 接受跟 arg/arg-bridge 现有 pattern CanvasXxxBackend 一致, **0 改 V0.1 任何代码**, V0.1 5 域 Rust struct 命名 100% 保留, 阶段 2 业务实装时 fill 业务方法 + 阶段 3 集成 实装时 fill Memgraph/CAS sink): 1) `CanvasElementBackend` (C-25, V0.63 反转 跟 `CanvasSyncBridge` 现有 pattern 一致); 2) `CanvasMultiUserAudit` (C-26, V0.63 反转 跟 `ARGController` 现有 pattern 一致); 3) `PermissionLevel` enum 3 变体 View/Comment/Edit (跟 trust_score_tier 5 档 pattern 不同, 仅权限 3 档); **(4) 守门合规** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 V0.1 agent-domain/src/ 任何代码, 0 改 arg-bridge/src/ 任何代码, 0 改 Cargo.toml [workspace] members 任何行 (仅 +1 member), 0 改 Cargo.lock 任何手编行; **(5) worktree cleanup** (per 守门 #9 #3 0 散落子代理产出): `git worktree remove .worktrees/wt-p3-d6-1-3-canvas-collab --force + git branch -D wt-p3-d6-1-3-canvas-collab` 0 残留; **(6) 累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + **任务 1.3 ✅ 收官 (canvas-collab/ 5 域 Rust struct 骨架)** + 任务 1.4-1.7 跨 session 续做估 ~1.20M tokens / 1.00 SRE·周 (per 实施计划 §3 任务 1.4 api 扩展 0.25 + 任务 1.5 BFF 0.30 + 任务 1.6 14+15 张表 0.25 + 任务 1.7 25 module 联动接口 0.20); **(7) 守门 #1 禁回溯叙事**: v0.1-v0.98 修订历史不动, v0.99 row 显式标 P3-D.6 阶段 1 基础 任务 1.3 收官; **(8) 触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化); P3-D.6 阶段 1 基础 任务 1.3 收官; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) | 2026-09-10 20:55 JST Mavis 自驱 (per 守门 #9 v19) + 守门 #1 v15 docs 同步饱和第 89 次新事件触发 仍允许 + 守门 #19 v19 累积规不破坏 V0.1 + 守门 #1 禁回溯叙事 + 守门 #14 v4 Mavis 审核决定 author=Ulysses |
"""

# ============================
# Registry v0.22 row
# ============================
REGISTRY_V022_ROW = """| **v0.22** | **2026-09-10** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **§2 索引 0 行 (1 新 crate 落档, 索引同步) + §3 v0.22**: P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化), 关联 commit `2fd60b1` (merge to main, 8 files / 397 insertions / 0 deletions, per 守门 #1 v15 docs 同步饱和第 89 次新事件触发 + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-3-canvas-collab.md` 19.7KB + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v19 累积规不破坏 V0.1 + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 跳 workspace 6/6 PASS): worker 子代理 bg_f851b8d4 在 worktree `D:/Star/.worktrees/wt-p3-d6-1-3-canvas-collab/` (branch `wt-p3-d6-1-3-canvas-collab` 基于 main `1a88e658`) 撰写, commit `5815b8c` 落档 (5 域 Rust struct CanvasElementBackend 12 字段 + PresenceCursor 8 字段 + CanvasComment 9 字段 + CanvasPermission 7 字段 + CanvasMultiUserAudit 9 字段 + 1 enum PermissionLevel View/Comment/Edit + 6 UT, 0 业务方法 0 API 0 endpoint 0 schema 留阶段 2 业务 + 阶段 3 集成 实装), 4 守门实证 cargo check 0 err 0.62s + cargo test 6/6 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings, brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正相比, 任务 1.3 0 bug 是 v0.18/v0.21 经验累积); Mavis merge to main `--no-ff` 0 conflict (main 已前进 1+ commit [registry v0.21 task 1.2], 但 worktree 仅改 crates/canvas-collab/ 新增 + Cargo.toml members +1, 0 冲突); worktree cleanup `git worktree remove --force + git branch -D` 0 残留; **v0.22 编号** (v0.21 已被任务 1.2 占用 per 4813a53, v0.18/v0.19/v0.20/v0.21 都被阶段 1 基础 任务 1.1/1.2/阶段 2 业务 任务 2.6/任务 2.1 batch 1 平行工作占用, 用 v0.22 跳 v0.18-v0.21 平行工作模式); **5 域 Rust struct 命名跟 v0.63 协调性检查报告一致** (per 协调性检查 v0.63 反转 C-25 + C-26): CanvasElementBackend (C-25, 跟 CanvasSyncBridge 现有 pattern 一致) + CanvasMultiUserAudit (C-26, 跟 ARGController 现有 pattern 一致); **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ 收官 (agent-domain/) + 任务 1.2 ✅ 收官 (canvas UI sync bridge) + 任务 1.3 ✅ 收官 (canvas-collab/ 5 域 Rust struct 骨架) + 任务 1.4-1.7 跨 session 续做估 ~1.20M tokens / 1.00 SRE·周 | **2026-09-10 20:08 JST Ulysses 拍板"推进" (per 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和)** |
"""

# ============================
# automation-design.md §4.34.2
# ============================
AUTOMATION_4342 = """

### 4.34.2 P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进", 2026-09-10 20:55 JST) — ⚠️ 跟 §4.34 / §4.34.1 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 + Agent struct 14 字段 per commit 64b96be 平行工作, §4.34.1 = P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge per commit f11517d 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.2 区分

> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 89 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 89 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改 crates/agent-domain/ 任何 file, 0 改 crates/arg-bridge/ 任何 file, 0 改 Cargo.toml [workspace] members 任何行 (仅 +1 member), 0 改 Cargo.lock 任何手编行) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 game 5 份 PHASE 报告) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-3-canvas-collab.md` 19.7KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 6/6 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `5815b8c` worktree + `2fd60b1` merge) + 守门 #11 缺标比错标 (serde/uuid/chrono/serde_json 全部已在根 Cargo.toml [workspace.dependencies] 现有, 0 重复) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `2fd60b1` merge to main, 8 files / 397 insertions / 0 deletions):
> - `crates/canvas-collab/Cargo.toml` (新, 230 bytes, [package] name=canvas-collab version=0.1.0 + [dependencies] serde/uuid/chrono/serde_json workspace 引用)
> - `crates/canvas-collab/src/lib.rs` (新, module-level doc + `pub mod models;` + 5 struct 全部 pub use)
> - `crates/canvas-collab/src/models/mod.rs` (新, `pub mod element; pub mod presence; pub mod comment; pub mod permission; pub mod audit;`)
> - `crates/canvas-collab/src/models/element.rs` (新, `CanvasElementBackend` struct 12 字段: id/canvas_id/element_type/x/y/width/height/z_index/created_by/created_at/updated_at/rotation + `Default` impl + 2 UT, 0 业务方法)
> - `crates/canvas-collab/src/models/presence.rs` (新, `PresenceCursor` struct 8 字段: id/user_id/canvas_id/x/y/color/last_active/created_at + `Default` impl + 1 UT, 0 业务方法)
> - `crates/canvas-collab/src/models/comment.rs` (新, `CanvasComment` struct 9 字段: id/canvas_id/element_id/parent_comment_id/content/created_by/created_at/updated_at/resolved + `Default` impl + 1 UT, 0 业务方法)
> - `crates/canvas-collab/src/models/permission.rs` (新, `CanvasPermission` struct 7 字段: id/canvas_id/user_id/permission_level/granted_by/granted_at/expires_at + `Default` impl + 1 UT + `PermissionLevel` enum 3 variants View/Comment/Edit, 0 业务方法)
> - `crates/canvas-collab/src/models/audit.rs` (新, `CanvasMultiUserAudit` struct 9 字段: id/canvas_id/actor_id/action/target_id/before_state/after_state/created_at/metadata + `Default` impl + 1 UT, 0 业务方法)
> - 根 `Cargo.toml` +2 lines (1 注释 + 1 member `crates/canvas-collab`)
> - `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑)
> - `docs/briefs/p3-d6-1-3-canvas-collab.md` (19.7KB, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.10-1 | D5.10-1 | `crates/canvas-collab/Cargo.toml` 新增 (230 bytes) | A | **[P]** | (worker 子代理 Write tool) | name=canvas-collab version=0.1.0 + 4 依赖 (serde/uuid/chrono/serde_json) 全部 workspace 引用, 0 重复 |
| D5.10-2 | D5.10-2 | `crates/canvas-collab/src/lib.rs` 新增 | A | **[P]** | (worker 子代理 Write tool) | module-level doc + `pub mod models;` + 5 struct 全部 pub use |
| D5.10-3 | D5.10-3 | `crates/canvas-collab/src/models/mod.rs` 新增 (5 mod pub) | A | **[P]** | (worker 子代理 Write tool) | `pub mod element/presence/comment/permission/audit;` |
| D5.10-4 | D5.10-4 | `crates/canvas-collab/src/models/element.rs` 新增 (`CanvasElementBackend` 12 字段 + 2 UT) | A, R, S | **[P]** | (worker 子代理 Write tool) | C-25 跟 v0.63 协调性检查报告一致 (跟 arg-bridge CanvasSyncBridge 现有 pattern), 0 业务方法 |
| D5.10-5 | D5.10-5 | `crates/canvas-collab/src/models/presence.rs` 新增 (`PresenceCursor` 8 字段 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | 0 业务方法, 阶段 2 任务 2.3 A12 多人编辑 fill |
| D5.10-6 | D5.10-6 | `crates/canvas-collab/src/models/comment.rs` 新增 (`CanvasComment` 9 字段 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | 0 业务方法, 阶段 2 任务 2.3 A12 多人编辑 fill |
| D5.10-7 | D5.10-7 | `crates/canvas-collab/src/models/permission.rs` 新增 (`CanvasPermission` 7 字段 + `PermissionLevel` 3 变体 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | 0 业务方法, 阶段 2 任务 2.3 A12 多人编辑 fill |
| D5.10-8 | D5.10-8 | `crates/canvas-collab/src/models/audit.rs` 新增 (`CanvasMultiUserAudit` 9 字段 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | C-26 跟 v0.63 协调性检查报告一致, 0 业务方法, 阶段 2 任务 2.1 A3.2 audit fill |
| D5.10-9 | D5.10-9 | 根 `Cargo.toml` +2 lines (1 注释 + 1 member) | A | **[P]** | (worker 子代理 Edit tool) | 0 改 [workspace] members 任何行 (仅 +1 member `crates/canvas-collab`) |
| D5.10-10 | D5.10-10 | `Cargo.lock` +10 lines (cargo 自动) | A | **[P]** | (cargo 自动) | 0 手动编辑, 0 改任何手编行 |
| D5.10-11 | D5.10-11 | 守门 #1 v25 实证 (worker 子代理 在 worktree 跑) | A, R | **[P]** | (cargo test 单 crate 跳 workspace) | `cargo check -p canvas-collab --lib -j 4` = **0 err 0.62s**; `cargo test -p canvas-collab --lib -j 4` = **6/6 PASS 0.00s** (test_canvas_element_backend_default + test_canvas_element_backend_clone + test_presence_cursor_default + test_canvas_comment_default + test_canvas_permission_default + test_canvas_multi_user_audit_default, 0 regression); `cargo fmt -p canvas-collab -- --check` = **0 diff**; `cargo clippy -p canvas-collab --lib -j 4` = **0 warnings** |
| D5.10-12 | D5.10-12 | worker 子代理 在 worktree 撰写, commit `5815b8c` 落档 (per 守门 #9 v27 3 段) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) |
| D5.10-13 | D5.10-13 | Mavis root merge to main | A | **[P]** | `git merge wt-p3-d6-1-3-canvas-collab --no-ff -F .git/MERGE_MSG_P3_D6_1_3.tmp` | 3-way merge 0 conflict (main 已前进 1+ commit [registry v0.21 task 1.2], 但 worktree 仅改 crates/canvas-collab/ 新增 8 files + Cargo.toml members +1 + Cargo.lock +10, 0 冲突). merge 后 `cargo test -p canvas-collab --lib -j 4` 在 main 上 = **6/6 PASS 0.00s** (验证 merge 后实证) |
| D5.10-14 | D5.10-14 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove --force + git branch -D` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-3-canvas-collab/` removed + branch `wt-p3-d6-1-3-canvas-collab` deleted (was 5815b8c). 0 残留 |
| D5.10-15 | D5.10-15 | `docs/automation-design.md` §4.34.2 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.10-16 | D5.10-16 | `scripts/automation/registry.md` §3 v0.22 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.22 修订历史 (20:55 JST task 1.3 收官) |
| D5.10-17 | D5.10-17 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.99 row | A | **[P]** | (本脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.3 canvas-collab 落档 (v0.99 编号: v0.92-v0.98 都被 P0-4 Stage 3.8-4.2 + 任务 2.6/2.1 batch 1/任务 1.2 平行工作占用, 用 v0.99 跳 v0.92-v0.98 平行工作模式) |

**§4.34.2 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.4-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 8 files + 2 lines 根 Cargo.toml + 10 lines Cargo.lock, 0 改现有 crate / 0 改现有 member)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 5815b8c in worktree + merge commit 2fd60b1) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.34.2 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/canvas-collab/Cargo.toml` 实证 v0.1 落档 (commit `2fd60b1` merge to main)
- `git log -1 --format='%an <%ae>'` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p canvas-collab --lib -j 4` 在 main 上 = **6/6 PASS 0.00s** (test_canvas_element_backend_default + test_canvas_element_backend_clone + test_presence_cursor_default + test_canvas_comment_default + test_canvas_permission_default + test_canvas_multi_user_audit_default)
- `cargo check -p canvas-collab --lib -j 4` 在 main 上 = **0 err 0.62s**
- worker 子代理 在 worktree 4 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err + cargo test 6/6 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings
- `git log -p --follow docs/briefs/p3-d6-1-3-canvas-collab.md` 实证 brief 落档 (19.7KB, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改 crates/agent-domain/src/ 任何 file (5 models + lib), 0 改 crates/arg-bridge/src/ 任何 file (8 file + lib), 0 改根 Cargo.toml [workspace] members 任何行 (仅 +1 member), 0 改 Cargo.lock 任何手编行
- 守门 #9 #3 0 散落子代理产出: worker 1 commit `5815b8c` 干净 + merge 1 commit `2fd60b1` 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 20:08 JST 拍板"推进" → 20:55 JST 1 commit 1 merge docs sync 闭环, 全程 ~47 分钟 (含 worker 子代理 RPC + 5 守门实证 + brief catch-up)
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-3-canvas-collab.md 19.7KB 在 worktree 撰写前已落档 (main `1a88e658` 时)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_f851b8d4) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 6 PASS) → collect_output (本返报 8 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 4 依赖 (serde/uuid/chrono/serde_json) 已在根 Cargo.toml [workspace.dependencies] 现有, 0 重复; brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正相比, 任务 1.3 0 bug 是 v0.18/v0.21 经验累积)
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/canvas-collab/ 8 file + 根 Cargo.toml +1 member + Cargo.lock +10, 不影响 V0.1 game 5 份 PHASE 报告)

**§4.34.2 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.10M tokens (worker 实装 0.07 + Mavis merge + verify + docs 0.03)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 阶段 2 业务 任务 2.6 + 任务 2.1 batch 1 19 commit: ~3.88M tokens (3.23 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.4-1.7 (剩余 4 任务: api 扩展 + BFF + 14+15 张表 + 25 module 联动接口): ~1.20M tokens (1.00 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2-5 跨 session 续 (10 module 业务实装 + 12 项剩余业务方法) + 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.6M tokens (1.33 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)
"""


def main() -> int:
    # 1) WBS append
    wbs_text = WBS_PATH.read_text(encoding="utf-8")
    if "**v0.99**" in wbs_text:
        print("[skip] WBS v0.99 already present")
    else:
        # 末尾追加 (line 1308 是 v0.98 row 最后一行, 后面无内容)
        if not wbs_text.endswith("\n"):
            wbs_text += "\n"
        wbs_text += WBS_V099_ROW
        WBS_PATH.write_text(wbs_text, encoding="utf-8")
        print(f"[ok] WBS v0.99 appended ({len(WBS_V099_ROW)} chars)")

    # 2) Registry append
    reg_text = REGISTRY_PATH.read_text(encoding="utf-8")
    if "**v0.22**" in reg_text:
        print("[skip] registry v0.22 already present")
    else:
        if not reg_text.endswith("\n"):
            reg_text += "\n"
        reg_text += REGISTRY_V022_ROW
        REGISTRY_PATH.write_text(reg_text, encoding="utf-8")
        print(f"[ok] registry v0.22 appended ({len(REGISTRY_V022_ROW)} chars)")

    # 3) automation-design.md §4.34.2 append
    auto_text = AUTOMATION_DESIGN_PATH.read_text(encoding="utf-8")
    if "### 4.34.2" in auto_text:
        print("[skip] automation-design §4.34.2 already present")
    else:
        # 末尾追加 (line 1971 是 §4.34.1 token OLU 估算 最后一行, 后面无内容)
        if not auto_text.endswith("\n"):
            auto_text += "\n"
        auto_text += AUTOMATION_4342
        AUTOMATION_DESIGN_PATH.write_text(auto_text, encoding="utf-8")
        print(f"[ok] automation-design §4.34.2 appended ({len(AUTOMATION_4342)} chars)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
