# P3-D.6 阶段 1 基础 任务 1.4 Brief: `crates/api/src/{agent,canvas_collab}/` 2 新 module 骨架

> **任务 ID**: p3-d6-1-4-api-extend
> **优先级**: P0 (P3-D.6 阶段 1 基础 第 4 任务)
> **估时**: ~0.20M tokens / 0.17 SRE·周 (per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.4)
> **依赖**: 任务 1.1 (`crates/agent-domain/` 新 crate, 已落档) + 任务 1.2 (`crates/arg-bridge/src/canvas_sync_bridge.rs` 已落档) + 任务 1.3 (`crates/canvas-collab/` 新 crate, 已落档)
> **作者**: Mavis (per Ulysses 19:40 JST 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进" + 守门 #9 v19 Mavis 自驱第 7 次强化)
> **worktree 分支**: `wt-p3-d6-1-4-api-extend` 基于 main `1f977f1` (任务 1.3 docs sync + 任务 1.3 merge `2fd60b1` 之后) / 当前 main `e053447` 之前最新 commit

---

## §0 目的

扩展 `crates/api/src/` 加 2 新 module (`agent/` + `canvas_collab/`), 为 P3-D.6 阶段 2 业务实装 (A1-A10 + A12 业务方法) 准备 API 骨架。

**Per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.4**:
> 1.4 `crates/api/src/{agent,arg,canvas_collab}/` 3 新 module (per DD-AGENT §3.3 跨域) | 1.1-1.3 | ~0.2M | #1 v25 + #14 v4

**⚠️ 已知缺口 (per 守门 #11 缺标比错标)**: 实施计划说"3 新 module"但 `arg/` 已存在 (V0.1 落档 per `4813a53`, 5 file 1863 lines), 所以本任务实际是 **2 新 module** (`agent/` + `canvas_collab/`), 跟 §0 目的 一致, 0 plan 冲突。

---

## §1 范围 / Out-of-scope

### 1.1 范围内 (要做的)

1. **`crates/api/src/agent/`** (5 文件, per `docs/design/DD-CANVAS-AGENT-001.md` §3.1 line 287-292):
   - `mod.rs` (~50 lines: module-level doc + 4 sub-mod pub + AgentState struct ~10 字段 + new() + build_router() + 1-2 UT)
   - `controller.rs` (~600 lines: 13 REST endpoints for A1-A10 + 路由 function + 占位 handler stub)
   - `permission.rs` (~150 lines: AgentPermission struct + 5 守门 helpers + check_tenant + require_role)
   - `audit.rs` (~80 lines: AuditEvent 5 字段 + audit_write helper)
   - `dto.rs` (~400 lines: 13 Request/Response DTO per A1-A10)

2. **`crates/api/src/canvas_collab/`** (5 文件, per `docs/design/DD-CANVAS-AGENT-001.md` §3.1 line 301-307 API 衍生):
   - `mod.rs` (~50 lines: module-level doc + 4 sub-mod pub + CanvasCollabState struct ~8 字段 + new() + build_router() + 1-2 UT)
   - `controller.rs` (~300 lines: 5 REST endpoints for canvas_collab + 路由 function + 占位 handler stub)
   - `permission.rs` (~150 lines: CanvasCollabPermission struct + 3 档权限 (View/Comment/Edit) + check_tenant + require_role)
   - `audit.rs` (~80 lines: CanvasCollabAuditEvent 5 字段 + audit_write helper per 守门 #13 d)
   - `dto.rs` (~250 lines: 5 Request/Response DTO)

3. **`crates/api/src/lib.rs`** (+2 lines, 0 改现有 `pub mod arg;`):
   - 1 line `/// canvas-collab API module (per P3-D.6 任务 1.4)`
   - 1 line `pub mod canvas_collab;`
   - 1 line `/// agent API module (per P3-D.6 任务 1.4)`
   - 1 line `pub mod agent;`
   - **总共 +4 lines** (0 改现有任何行)

4. **`crates/api/Cargo.toml`** (per 守门 #11 缺标比错标, 检查现有依赖, **0 重复加**):
   - 检查 `agent-domain` (path 引用) / `canvas-collab` (path 引用) / `star-context` (path 引用) 3 新依赖需加
   - axum / tokio / serde / serde_json / uuid / chrono / async-trait / thiserror 全部已在现有 `crates/api/Cargo.toml` 现有, 0 重复

### 1.2 Out-of-scope (不做的)

1. **0 业务方法实装** (留 P3-D.6 阶段 2 任务 2.1-2.5):
   - 13 REST endpoint handler = `async fn handler() -> Result<Json<T>, ApiError> { Ok(Json(T::default())) }` 占位
   - 0 真实 DB query / 0 真实 Memgraph / 0 真实 PostgreSQL adapter 调用

2. **0 WebSocket / SSE 端点** (留 P3-D.6 阶段 3 集成 任务 3.2):
   - 0 `wss_hub.rs` / 0 SSE 推送 (跟 `crates/api/src/arg/sse_hub.rs` 不同, 本任务仅 REST 骨架)
   - A12 WebSocket 4 端点 走 `bff/src/collaboration/wss_hub.rs` (per 实施计划 任务 1.5)

3. **0 OpenAPI spec 生成** (per 守门 #14 v4 + 守门 #11 缺标比错标):
   - 0 `utoipa` macro / 0 `#[derive(ToSchema)]` / 0 swagger.json
   - 0 端点文档注释 (`/// `) 可选, 但 0 完整 OpenAPI 3.1 spec

4. **0 前端集成 / 0 BFF** (留 P3-D.6 阶段 3 集成 + 阶段 4 实装):
   - 0 改 `bff/` 任何代码 (BFF 是任务 1.5)
   - 0 改 `frontend/` 任何代码

5. **0 改 `crates/api/src/arg/` 任何代码** (per 守门 #1 禁回溯叙事):
   - 0 改 `crates/api/src/arg/{mod,controller,dto,permission,sse_hub}.rs` 任何行
   - 0 改 `crates/api/src/lib.rs` 现有 `pub mod arg;` 行

---

## §2 详细设计

### 2.1 `crates/api/src/agent/mod.rs` (~50 lines)

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! `crates/api/src/agent` — STAR Agent API tier (A1-A10).
//!
//! Reference: [`docs/design/DD-CANVAS-AGENT-001.md`](../../../../docs/design/DD-CANVAS-AGENT-001.md)
//! v0.1.1 §3.1 line 287-292. This module is the **API tier** for A1-A10
//! agent operations; it sits on top of `crates/agent-domain` (data tier) and
//! exposes 13 REST endpoints.
//!
//! 4 sub-modules:
//! - [`controller`]  — 13 REST route handlers (A1.1 + A2.1 + A2.2 + A2.3 + A2.4 + A3.2 + A4.1 + A4.2 + A5.1 + A5.2 + A6.1 + A6.2 + A7.2)
//! - [`permission`]  — RLS 13 類 + 6 角色 守门
//! - [`audit`]       — 100% audit middleware (per 守门 #13 d)
//! - [`dto`]         — request / response types + [`ApiError`]
//!
//! 守门合规 (per 守门 #1 v25 + 守门 #7 + 守门 #12 v21 + 守门 #14 v2):
//! - 0 `unsafe` blocks (守门 #7 `unsafe_code = "forbid"`).
//! - RLS tenant 校验每个写路径都过 [`permission::AgentPermission::check_tenant`].
//! - 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 拍板 D, 2026-09-05 10:43 JST).
//! - docs 同步走 `docs/automation-design.md` §4.34.3 + `scripts/automation/registry.md` §5.5.

use std::sync::Arc;
use axum::Router;
use uuid::Uuid;

use star_agent_domain::models::agent::{Agent, AgentDomainError};

pub mod audit;
pub mod controller;
pub mod dto;
pub mod permission;

// ... AgentState struct 8-10 字段 + new() + build_router() + re-exports + 1-2 UT
```

**字段 (AgentState, 8 字段)**:
- `agent_ops: Arc<star_agent_domain::AgentOps>` (per V0.2 强类型域)
- `audit: Arc<audit::AgentAudit>`
- `permission: Arc<permission::AgentPermission>`
- `tenant_id: Uuid` (per 守门 #13 b RLS 13 類)
- + 5 ops placeholder (V0.2 强类型后续 fill)

### 2.2 `crates/api/src/agent/controller.rs` (~600 lines)

13 REST endpoints (Axum router, 占位 handler):

| Endpoint | Method | Path | Handler | A# | 业务方法 |
|---|---|---|---|---|---|
| 1 | POST | `/api/v1/agents` | `create_agent` | A1.1 | `Agent::create` |
| 2 | GET | `/api/v1/agents/:id` | `get_agent` | A1.1 | `Agent::read` |
| 3 | PATCH | `/api/v1/agents/:id` | `update_agent` | A1.1 | `Agent::update` |
| 4 | DELETE | `/api/v1/agents/:id` | `delete_agent` | A1.1 | `Agent::delete` |
| 5 | POST | `/api/v1/agents/:id/handoff` | `handoff_agent` | A2.1 | `HandoffConnector::handoff` |
| 6 | GET | `/api/v1/agents/:id/topology` | `get_topology` | A2.2 | `DomainFrame::topology` |
| 7 | POST | `/api/v1/agents/:id/parent` | `set_parent` | A2.3 | `ParentChildConnector::set_parent` |
| 8 | POST | `/api/v1/agents/:id/pipeline` | `add_to_pipeline` | A2.4 | `PipelineConnector::add` |
| 9 | POST | `/api/v1/agents/:id/state` | `change_state` | A3.2 | `Agent::state_change_audit` (V0.3 已实装) |
| 10 | GET | `/api/v1/agents/:id/worktree` | `get_worktree` | A4.1 | `WorktreeAssoc::get` |
| 11 | GET | `/api/v1/agents/:id/work-items` | `get_work_items` | A4.2 | `WorkItemAssoc::list` |
| 12 | POST | `/api/v1/agents/:id/start` | `start_agent` | A6.1 | `Agent::start` (V0.3 已实装) |
| 13 | POST | `/api/v1/agents/:id/stop` | `stop_agent` | A6.1 | `Agent::stop` (V0.3 已实装) |

**Handler 占位 pattern** (13 个统一用):
```rust
async fn create_agent(
    State(state): State<Arc<AgentState>>,
    Json(req): Json<dto::CreateAgentRequest>,
) -> Result<Json<dto::AgentResponse>, dto::ApiError> {
    state.permission.check_tenant(state.tenant_id)?;
    state.audit.write_event(/* ... */).await?;
    // V0.4 阶段 2 任务 2.1 实装:
    // let agent = state.agent_ops.create(req.into(), state.tenant_id).await?;
    Ok(Json(dto::AgentResponse::default()))
}
```

**0 业务方法实装** (handler body = 守门 + 占位 `Ok(Json(T::default()))`)

### 2.3 `crates/api/src/agent/permission.rs` (~150 lines)

跟 `crates/api/src/arg/permission.rs` (251 lines) 同形:
- `pub mod role { pub type RoleSet = HashSet<String>; }`
- `pub struct AgentPermission { current_tenant: Uuid, roles: RoleSet }`
- `impl AgentPermission { pub fn new() / pub fn check_tenant() / pub fn require_role() }`
- 1-2 UT

### 2.4 `crates/api/src/agent/audit.rs` (~80 lines)

- `pub struct AgentAuditEvent { id: Uuid, agent_id: Uuid, actor_id: Uuid, action: String, at: DateTime<Utc> }`
- `pub struct AgentAudit { /* sink placeholder */ }`
- `impl AgentAudit { pub async fn write_event(&self, ...) -> Result<(), ApiError> }`
- 1-2 UT

### 2.5 `crates/api/src/agent/dto.rs` (~400 lines)

13 Request/Response DTO:
- `CreateAgentRequest` (10 字段 per Agent struct V0.2)
- `UpdateAgentRequest` (8 字段, partial)
- `AgentResponse` (14 字段, 跟 Agent struct 1:1)
- `HandoffRequest` / `HandoffResponse`
- `TopologyResponse` (5 域 Frame info)
- `ParentRequest` / `ParentResponse`
- `PipelineRequest` / `PipelineResponse`
- `StateChangeRequest` (new_state: AgentState)
- `WorktreeResponse` / `WorkItemListResponse`
- `AgentActionResponse` (start/stop/restart 共用)
- + 1 `ApiError` enum 5 变体 (NotFound / Forbidden / BadRequest / Internal / Unimplemented)

### 2.6 `crates/api/src/canvas_collab/mod.rs` (~50 lines)

跟 agent/ 同形, 字段:
- `canvas_ops: Arc<canvas_collab::CanvasOps>` (per 任务 1.3 强类型域)
- `audit: Arc<audit::CanvasCollabAudit>`
- `permission: Arc<permission::CanvasCollabPermission>`
- `tenant_id: Uuid`

### 2.7 `crates/api/src/canvas_collab/controller.rs` (~300 lines)

5 REST endpoints (per A12.7 + A12.8):

| Endpoint | Method | Path | Handler | A# | 业务方法 |
|---|---|---|---|---|---|
| 1 | POST | `/api/v1/canvas/elements` | `create_element` | A12.1 | `CanvasElementBackend::create` |
| 2 | PATCH | `/api/v1/canvas/elements/:id` | `update_element` | A12.2 | `CanvasElementBackend::update` |
| 3 | POST | `/api/v1/canvas/elements/:id/comments` | `add_comment` | A12.3 | `CanvasComment::add` |
| 4 | GET | `/api/v1/canvas/:id/presence` | `get_presence` | A12.4 | `PresenceCursor::list` |
| 5 | POST | `/api/v1/canvas/:id/permission` | `grant_permission` | A12.7 | `CanvasPermission::grant` |

### 2.8 `crates/api/src/canvas_collab/permission.rs` (~150 lines)

跟 agent/permission.rs 同形 + 3 档权限 enum:
```rust
pub enum PermissionLevel { View, Comment, Edit }
impl CanvasCollabPermission {
    pub fn require(&self, level: PermissionLevel) -> Result<(), ApiError> { ... }
}
```

### 2.9 `crates/api/src/canvas_collab/audit.rs` (~80 lines)

跟 agent/audit.rs 同形, 字段:
- `id: Uuid, canvas_id: Uuid, actor_id: Uuid, action: String, at: DateTime<Utc>, metadata: serde_json::Value`

### 2.10 `crates/api/src/canvas_collab/dto.rs` (~250 lines)

5 Request/Response DTO + ApiError:
- `CreateElementRequest` (12 字段 per CanvasElementBackend)
- `UpdateElementRequest` (8 字段, partial)
- `ElementResponse` (12 字段)
- `AddCommentRequest` / `CommentResponse`
- `PresenceListResponse`
- `GrantPermissionRequest` (user_id, level)
- `PermissionResponse`
- + ApiError 同 agent/

### 2.11 `crates/api/src/lib.rs` 改动

```diff
 pub mod arg;
+/// canvas-collab API module (per P3-D.6 任务 1.4)
+pub mod canvas_collab;
+/// agent API module (per P3-D.6 任务 1.4)
+pub mod agent;
```

### 2.12 `crates/api/Cargo.toml` 改动

检查现有依赖, 加 2 新 path 引用 (0 重复):
```toml
star-agent-domain = { path = "../agent-domain" }
canvas-collab = { path = "../canvas-collab" }
```

(serde / serde_json / async-trait / thiserror / uuid / chrono / tokio / star-context / axum 全部已在现有, 0 重复加)

---

## §3 守门实证要求

### 3.1 守门 #1 v25 (cargo test 改单 crate 跳 workspace)

worker 子代理 在 worktree 跑 (4 守门实证):
- `cargo check -p api --lib -j 4` = **0 err** (V0.1 arg/ + 0 改)
- `cargo test -p api --lib -j 4` = **N/N PASS 0.00s** (V0.1 现有 0 UT + V0.2 新加 ~10-20 UT)
- `cargo fmt -p api -- --check` = **0 diff** (V0.1 现有 + V0.2 新加)
- `cargo clippy -p api --lib -j 4` = **0 warnings on my code** (advisory per 守门 #7 v3)

### 3.2 守门 #11 缺标比错标 (4 已知缺口显式列)

worker 子代理 在 brief catch-up commit 写明:
- (a) `agent-domain` + `canvas-collab` 2 新 path 依赖加进 `crates/api/Cargo.toml` (检查现有 0 重复)
- (b) `crates/api/src/lib.rs` +4 lines (0 改现有 `pub mod arg;` 行)
- (c) `arg/` 已存在 0 改, 实施计划说"3 新"实际"2 新" (per守门 #1 禁回溯叙事 不重写 plan, 显式标缺口)
- (d) 0 WebSocket / 0 OpenAPI / 0 BFF 集成 (留阶段 2/3/4)

### 3.3 守门 #14 v4 (Mavis 审核 author=Ulysses)

commit author=`Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m '...'
```

### 3.4 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码)

worker 子代理 在 commit 显式标 0 改:
- `crates/api/src/arg/{mod,controller,dto,permission,sse_hub}.rs` 5 file 任何行
- `crates/api/src/lib.rs` 现有 `pub mod arg;` 行
- 根 `Cargo.toml` [workspace] members 行
- `Cargo.lock` 任何手编行 (cargo 自动)

### 3.5 守门 #19 v19 累积规 (0 破坏 V0.1)

worker 子代理 在 commit 显式标:
- V0.1 `arg/` 5 file 100% 保留 (per 4813a53 任务 1.2 merge 后)
- V0.1 `crates/api/src/lib.rs` 511 lines 100% 保留
- V0.1 `crates/api/Cargo.toml` 39 lines 100% 保留
- V0.2 仅追加 10 file (2 module x 5 file) + 4 lines lib.rs + 2 lines Cargo.toml

---

## §4 交付物 (Deliverable)

worker 子代理 在 worktree 撰写 + commit + push branch:

1. **代码 10 file** (新增, 0 改 V0.1 任何行):
   - `crates/api/src/agent/{mod,controller,permission,audit,dto}.rs` 5 file
   - `crates/api/src/canvas_collab/{mod,controller,permission,audit,dto}.rs` 5 file
   - 总估 ~1900 lines (agent ~1280 + canvas_collab ~830 - 共享 pattern)

2. **Cargo.toml +4 lines** (0 改现有 39 lines):
   - `crates/api/Cargo.toml` +2 lines (2 新 path 依赖)

3. **lib.rs +4 lines** (0 改现有 511 lines):
   - `crates/api/src/lib.rs` +4 lines (2 new `pub mod` + 2 doc comments)

4. **6-12 新 UT** (per 守门 #1 v25 实证 0 regression):
   - `crates/api/src/agent/mod.rs` 1-2 UT (AgentState::new + build_router)
   - `crates/api/src/agent/permission.rs` 1-2 UT
   - `crates/api/src/agent/audit.rs` 1 UT
   - `crates/api/src/canvas_collab/mod.rs` 1-2 UT
   - `crates/api/src/canvas_collab/permission.rs` 1 UT
   - `crates/api/src/canvas_collab/audit.rs` 1 UT

5. **commit message** 包含:
   - author=Ulysses
   - 4 守门实证 (cargo check 0 err + cargo test N/N PASS + cargo fmt 0 diff + cargo clippy 0 warnings)
   - 0 改 V0.1 任何代码
   - 5 域 Lead 真人未到位 Mavis 临时代签

---

## §5 Acceptance Criteria

worker 子代理 提交前必跑 + 在 commit message 显式标:
- [x] `cargo check -p api --lib -j 4` = 0 err
- [x] `cargo test -p api --lib -j 4` = (V0.1 N + V0.2 M) / (V0.1 N + V0.2 M) PASS 0.00s, 0 regression
- [x] `cargo fmt -p api -- --check` = 0 diff
- [x] `cargo clippy -p api --lib -j 4` = 0 warnings on my code
- [x] `cargo check --workspace --lib -j 4` = 0 err (跟 V0.1 + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容)

---

## §6 风险 / 已知缺口

1. **`crates/api/Cargo.toml` 缺 `axum`** (per 守门 #11 缺标比错标 必查): V0.1 用 `axum::Router` per `crates/api/src/arg/mod.rs` line 23, V0.2 0 改 axum, 但 worker 必先 `cat crates/api/Cargo.toml` 检查现有依赖, 0 重复加
2. **`crates/api/Cargo.toml` 缺 `tokio`** (per 守门 #11 缺标比错标 必查): V0.1 `arg/sse_hub.rs` 用 `tokio::sync::broadcast` per line 6, V0.2 0 改 tokio
3. **0 WebSocket / 0 SSE** (per §1.2 out-of-scope): 实施计划 A12 WebSocket 4 端点 走 `bff/src/collaboration/wss_hub.rs` (per 任务 1.5)
4. **0 OpenAPI spec**: 0 `utoipa` 依赖, 0 `#[derive(ToSchema)]`, 留 P3-D.6 阶段 3 集成 任务 3.2 实装
5. **5 域 Lead 真人未到位**: Mavis 临时代签 (per 守门 #14 v2 拍板 D), 真人到位后追溯签字覆盖修订历史

---

## §7 commit message 模板 (worker 子代理 用)

```
feat(api): P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ 2 新 module 骨架 落档 (per 实施计划 §3 任务 1.4 + DD-AGENT §3.1 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #1 禁回溯叙事不重写 V0.1 任何代码 + 守门 #19 v19 累积规不破坏 V0.1): 12 files changed, +~1900/-0 lines (0 deletions), 0 子代理调用除自身外 (跟 V0.1 任务 1.1 模式一致), V0.1 100% compat (crates/api/src/arg/ 5 file 1863 lines + crates/api/src/lib.rs 511 lines + crates/api/Cargo.toml 39 lines 全部不动, V0.2 仅追加 10 file + 4 lines lib.rs + 2 lines Cargo.toml): (1) **`crates/api/src/agent/` 5 新增** (per DD-AGENT §3.1 line 287-292): (a) `mod.rs` AgentState 8 字段 + new() + build_router() + 2 UT; (b) `controller.rs` 13 REST endpoint 占位 handler (A1.1 4 + A2.1-A2.4 4 + A3.2 1 + A4.1-A4.2 2 + A6.1 2, 跟 V0.3 agent-domain 5 业务方法一致, 0 业务方法 0 DB query 0 Memgraph adapter 调用); (c) `permission.rs` AgentPermission struct + check_tenant + require_role + 2 UT; (d) `audit.rs` AgentAuditEvent 5 字段 + write_event 占位 + 1 UT; (e) `dto.rs` 13 Request/Response DTO + ApiError 5 变体; (2) **`crates/api/src/canvas_collab/` 5 新增** (per DD-AGENT §3.1 line 301-307 API 衍生): (a) `mod.rs` CanvasCollabState 8 字段 + 1-2 UT; (b) `controller.rs` 5 REST endpoint (A12.1-A12.4 + A12.7, 0 WebSocket 留任务 1.5 BFF); (c) `permission.rs` CanvasCollabPermission + PermissionLevel enum 3 变体 (View/Comment/Edit) + require(level) helper + 1 UT; (d) `audit.rs` CanvasCollabAuditEvent 6 字段 + write_event 占位 + 1 UT; (e) `dto.rs` 5 Request/Response DTO + ApiError 5 变体; (3) **`crates/api/src/lib.rs` +4 lines** (0 改现有 511 lines + 0 改 `pub mod arg;` 行): 2 new `pub mod agent;` / `pub mod canvas_collab;` + 2 doc comment; (4) **`crates/api/Cargo.toml` +2 lines** (0 改现有 39 lines + 0 重复加 axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context): 2 新 path 依赖 `star-agent-domain` + `canvas-collab`; (5) **守门 #1 v25 实证**: `cargo check -p api --lib -j 4` = 0 err; `cargo test -p api --lib -j 4` = (V0.1 0 + V0.2 6-12) / (V0.1 0 + V0.2 6-12) PASS 0.00s 0 regression; `cargo fmt -p api -- --check` = 0 diff; `cargo clippy -p api --lib -j 4` = 0 warnings on my code; (6) **守门合规** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反: 0 改 crates/api/src/arg/ 任何 file 任何行, 0 改 crates/api/src/lib.rs 511 lines 任何行, 0 改 crates/api/Cargo.toml 39 lines 任何行, 0 改根 Cargo.toml [workspace] members 任何行, 0 改 Cargo.lock 任何手编行; (7) **跨 crate 实证**: `cargo check --workspace --lib -j 4` = 0 err (跟 V0.1 arg/ + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容); (8) **累计 P3-D.6 阶段 1 基础**: 任务 1.1 ✅ (agent-domain) + 任务 1.2 ✅ (canvas UI sync bridge) + 任务 1.3 ✅ (canvas-collab/) + **任务 1.4 ✅ (api 2 新 module)** + 任务 1.5-1.7 跨 session 续做估 ~1.00M tokens / 0.83 SRE·周; (9) **守门 #1 禁回溯叙事**: V0.1-V0.99 修订历史不动, 本 commit 显式标 P3-D.6 阶段 1 基础 任务 1.4 收官; (10) **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化; commit author=Ulysses (per 守门 #10 + 守门 #14 v4)
```

---

## §8 docs 同步 (per 守门 #12 v21)

worker 子代理 commit 后, Mavis root session 必跑:
1. `docs/reports/STAR-P3-WBS-001.md` v0.99.1 row (per 守门 #11 缺标比错标 跟 P0-4 Stage 4.3 v0.99 平行编号) — 任务 1.4 收官
2. `scripts/automation/registry.md` v0.23 row — 任务 1.4 索引同步
3. `docs/automation-design.md` §4.34.3 段 (跟 §4.34 + §4.34.1 + §4.34.2 平行编号) — 任务 1.4 17 子项任务卡 (D5.11-1 ~ D5.11-17)
4. `docs/briefs/p3-d6-1-4-api-extend.md` (本 brief) catch-up commit

---

## §9 跨 session 续做 (per 守门 #9 v19 Mavis 自驱)

任务 1.4 收官后跨 session 续做:
- **任务 1.5** `bff/src/collaboration/` BFF 骨架 (per 任务 1.4 依赖), 5 REST + 4 WebSocket, envoy 独立 deployment per 9/1 13:05 JST
- **任务 1.6** 14+15 张表 SQL DDL 落档 (per 任务 1.4 依赖), 守门 #13 W/T/M 100% 覆盖 + 0 混在
- **任务 1.7** 25 module 联动接口定义 (per 任务 1.4-1.6 依赖)
- **任务 2.1-2.5** 阶段 2 业务实装 A1-A12 46 项 + G1-G12 32 项 + 13 关键 class, 估 ~1.6M tokens / 1.33 SRE·周
- **任务 3.1-3.4** 阶段 3 集成 23 REST + 5 WSS + 25 module 联动
- **任务 4.1-4.6** 阶段 4 实装 e2e test + k3s deploy

---

**Brief end** (per 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #12 v21 [P] docs 同步)
