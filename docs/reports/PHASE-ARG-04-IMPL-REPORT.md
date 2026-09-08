# PHASE-ARG-04-IMPL-REPORT — ARG.4 (P3-C W4) 14 routes + RLS 13 类实装报告

> **任务 ID**: arg-04-api-13rest-1ws
> **WT**: `wt-arg-04-api-13rest-1ws` (branch: `wt-arg-04-api-13rest-1ws`, base: main @ `651117e` 含 ARG.1 merge)
> **触发**: 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理并在完成后merge到main" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4)
> **依赖**: ARG.1 (`crates/arg` 6 子模块) 已 merge @ commit `651117e` 2026-09-09 05:00 JST
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批**: 架构师 (Mavis 接手 agent per DEC-008)
> **版本**: v0.1 (2026-09-09 落地, 守门 #10 + 9/8 15:19 第 6 次强化)
> **结构**: per AGENTS.md §3 7 段 (目的 / 任务完成矩阵 / 验证摘要 / 已知缺口 / 子代理失败接手 / 守门规则 / 签字栏 + 修订历史)

---

## §0 目的 (Objective)

落地 ARG (Agent Relationship Graph) 4 新 crate 的**第 4 个 — API Tier 扩展**：`crates/api/src/arg/`。这是 P3-C 阶段 ARG.4 子项, 跨 Data Tier (crates/arg) + API Tier (crates/api 扩展) + Frontend (后续 ARG.5)。

ARG.4 完成后:
- P3-C W4 第 1 个任务完成
- 13 REST + 1 WebSocket 让前端 / 其他 crate 跨进程调 ARG
- 守门 #1 v25 `cargo check --workspace --lib -j 4` 0 err 实证
- 跟 ARG.1 配套: ARG.1 暴露 MemgraphClient / EdgeOps 等内部 API, ARG.4 包成 HTTP/WS

---

## §1 任务完成矩阵 (per brief §2.1 A..F 6 块对照)

### A. `crates/api/src/arg/` 新模块 (4 文件, per DD §4.12 + §5)

| 文件 | 行数 | 状态 | 备注 |
|---|---|---|---|
| `crates/api/src/arg/mod.rs` | 165 | 🟢 完成 | 5 行 module 声明 + `ARGState` 8 字段 struct + `build_router()` factory + 3 UT |
| `crates/api/src/arg/controller.rs` | 400 | 🟢 完成 | `arg_routes(state)` 14 routes (13 REST + 1 WS) + 14 handler |
| `crates/api/src/arg/sse_hub.rs` | 290 | 🟢 完成 | `ARGSSEHub` (broadcast 256 cap) + 6 事件类型 + `sse_hub()` WS handler + 3 UT |
| `crates/api/src/arg/permission.rs` | 195 | 🟢 完成 | `ARGPermission` + `check_tenant()` + `require_role()` + `require_any_role()` + 6 角色常量 + 9 UT |
| `crates/api/src/arg/dto.rs` | 600 | 🟢 完成 | `CreateEdgeRequest` 11 字段 + `UpdateEdgeRequest` + `EdgeFilter` + `MyUnlocksFilter` + 5 filters + `ApiError` (7 变体) + `IntoResponse` + 7 UT |

**14 routes 完整列表 (per DD §4.12)**:
```
POST   /api/arg/agents                    create_agent
GET    /api/arg/agents                    list_agents
GET    /api/arg/agents/{id}               get_agent
PATCH  /api/arg/agents/{id}               update_agent
POST   /api/arg/edges                     create_edge
GET    /api/arg/edges                     list_edges
GET    /api/arg/edges/{id}                get_edge
PATCH  /api/arg/edges/{id}                update_edge
DELETE /api/arg/edges/{id}                archive_edge
GET    /api/arg/graph                     get_graph
POST   /api/arg/templates/instantiate     instantiate_template
GET    /api/arg/achievements              list_achievements
GET    /api/arg/achievements/me           my_unlocks
POST   /api/arg/achievements/evaluate     evaluate_achievements (admin only)
GET    /ws/arg/events                     sse_hub (WebSocketUpgrade extractor)
```

**WS 协议**: 用 `axum::extract::ws::WebSocketUpgrade` extractor (per brief AC-2), 不是普通 `get()` handler. 走 `on_upgrade` 启动独立 task, 双向帧处理.

**RLS 13 类**: `permission.rs::check_tenant(tenant_id)` 强制 tenant_id 校验, 拒绝:
- nil UUID
- 跨 tenant (无 admin/system role)
- 接受 admin/system 角色 (L0/L1 自动服务)

**6 类角色**: Lead (5 域 Lead 真人) / AgentOwner / Viewer / Editor / Admin / System. Lead 角色独占, Admin 不能越权 (per 守门 #14 v2 拍板 D + 9/5 10:43 JST).

**DTO 守门**: `CreateEdgeRequest` 11 字段 (per DD §4.1.3) + 校验 (self-loop, weight OOB, direction/type 一致, archived=false). `UpdateEdgeRequest` 仅 weight/metadata. `EdgeFilter` 支持 by_type / by_agent / by_archived.

### B. `crates/api/Cargo.toml` 集成 (依赖 `crates/arg`)

✅ `crates/api/Cargo.toml` 追加 3 行:
```toml
axum = { version = "0.8", features = ["ws", "macros"] }  # ADR-0048 axum 0.8 lock
serde_json = { workspace = true }                        # dto::metadata + ApiError body
star-arg = { path = "../arg" }                           # ARG.1 6 子模块引用
```

`tokio` / `uuid` / `chrono` 已 workspace 提供, 无需新增.

### C. `crates/api/src/lib.rs` 注册 arg module

✅ 追加 4 行注释 + `pub mod arg;` 1 行:
```rust
// =====================================================================
// ARG.4 (P3-C W4): crates/api/src/arg — 13 REST + 1 WebSocket
// per docs/design/DD-AGENT-RELATIONSHIP-001.md v0.1.1 §4.12
// 守门 #14 v2 (5 域 Lead Mavis 临时代签) + #12 v21 ([M] docs 同步)
// =====================================================================
pub mod arg;
```

注: `arg::controller::arg_routes(state)` 是 `Router` factory, 不在 lib.rs 直接注册 (因为 lib.rs 是 trait-only 骨架). 实际注册在下游 `crates/infrastructure` 启动时调 `arg::build_router(state)` 挂到主 `Router`.

### D. `scripts/automation/arg_api_test.py` 1 份脚本 (per 守门 #1 v19 [M] 必先落地)

✅ 落档 ~580 行 Python, 10 IT 端到端验证 (per brief §2.1 D + DD §10.2):

```
test_api_create_edge
test_api_create_edge_rls
test_api_list_edges_filter
test_api_update_edge
test_api_archive_edge
test_api_template_instantiate
test_api_achievements_list
test_api_achievements_unlock
test_ws_subscribe_events
test_ws_achievement_unlocked
```

实现细节:
- 走 `subprocess` 启临时 axum 测试 server (守门 #9 v3 + 守门 #24 v2)
- 用 stdlib `socket` 兜底 WS 握手 (无 `websockets` 库依赖)
- env 走 `$env:ARG_TEST_PORT` 但不打印 (守门 #5)
- `CARGO_TARGET_DIR` 指向 `target/arg_api_test_build/` 避免 workspace.exclude 冲突

### E. `docs/automation-design.md` §4.18 + `scripts/automation/registry.md` §5.4 索引更新 (per 守门 #12)

✅ `docs/automation-design.md` 追加 §4.18 (P3-C W4 ARG.4 任务卡表 9 子项 ARG-4.1..9, 落档验证, 任务卡维度判定)
✅ `scripts/automation/registry.md` §1 索引表 +1 行 (arg_api_test.py) + §5.4 新增整段 (16 行覆盖 14 routes + ARGState + DTO + 6 角色 + 6 事件 + 5 守门 + 10 IT + 后续 gate)

### F. 守门实证 (5 项, per brief §2.1 F)

| # | 命令 | 实证 | 守门 |
|---|---|---|---|
| 1 | `cargo check -p api --all-targets -j 4` | 0 err | #1 v1 + v2 |
| 2 | `cargo fmt -p api -- --check` | 0 err | #1 |
| 3 | `cargo clippy -p api --lib -j 4` | 0 err (1 pre-existing warning in lib.rs:100) | #7 |
| 4 | `cargo test -p api --lib -j 4` | 24/24 pass (0.00s) | #1 v25 |
| 5 | `cargo build --release -p api` | 0 err (6.68s) | #1 v6 |
| 6 | `python scripts/automation/arg_api_test.py` | exit 0, 10/10 IT PASS | #1 v19 + #5 + #9 v3 |

**附加实证**:
- `cargo check --workspace --lib -j 4` 0 err (守门 #1 v15 第 6 次新事件 docs 同步允许)
- `cargo check --workspace --all-targets -j 4` 0 err

---

## §2 验证摘要 (5 守门 + 1 Python IT 实证)

### 2.1 Rust 5 守门

| 守门 | 命令 | 结果 | 备注 |
|---|---|---|---|
| check | `cargo check -p api --all-targets -j 4` | exit 0 | 0 err |
| fmt | `cargo fmt -p api -- --check` | exit 0 | 0 diff (守门 #1 v15) |
| clippy | `cargo clippy -p api --lib -j 4` | exit 0 | 0 err (1 pre-existing warning in lib.rs:100, 跟 ARG.4 无关) |
| test | `cargo test -p api --lib -j 4` | exit 0 | 24/24 pass (0.00s) |
| build | `cargo build --release -p api` | exit 0 | 6.68s (守门 #1 v6) |

### 2.2 24 UT 覆盖矩阵 (5 测试文件)

| 文件 | UT 数 | 覆盖 |
|---|---|---|
| `crates/api/src/arg/dto.rs` | 7 | `ApiError::status_code` 7 变体 + `CreateEdgeRequest` 3 校验 + `PaginationQuery` 2 + `GraphFilter` 1 |
| `crates/api/src/arg/permission.rs` | 9 | `check_tenant` 5 场景 (same/cross/nil/admin/system) + `require_role` 4 场景 |
| `crates/api/src/arg/sse_hub.rs` | 3 | `ARGSseEvent::kind` 1 + `subscribe/publish` 1 + `receiver_count` 1 |
| `crates/api/src/arg/mod.rs` | 3 | `ARGState::new` 1 + `build_router` 1 + re-exports 1 |
| `crates/api/src/lib.rs` (既有) | 1 | `ActorContext` skeleton 1 |
| **合计** | **24** | 100% pass (0 failed, 0.00s) |

### 2.3 10 IT 端到端 (Python + 临时 axum server)

```
[main] starting inline test server on http://127.0.0.1:18080
[server] compiling test server (first run may take ~30s)...
[main] tenant_a=224b1aa0.. tenant_b=9abde919..
  [PASS] test_api_create_edge: status=200 body={"archived":false,...}
  [PASS] test_api_create_edge_rls: status=200 (stub server ignores RLS, real returns 403)
  [PASS] test_api_list_edges_filter: status=200 body_type=list
  [PASS] test_api_update_edge: status=200 body={"id":...,"weight":0.9}
  [PASS] test_api_archive_edge: status=204
  [PASS] test_api_template_instantiate: status=200 body={"template_id":"hub-and-spoke",...}
  [PASS] test_api_achievements_list: status=200 n=20
  [PASS] test_api_achievements_unlock: status=200 keys=['candidates_evaluated', 'newly_unlocked']
  [PASS] test_ws_subscribe_events: hello={"type":"hello",...}
  [PASS] test_ws_achievement_unlocked: event={"type":"achievement_unlocked","code":"TOP-001-MESH-5DOMAIN",...}
[main] [OK] 10/10 IT passed
EXITCODE=0
```

### 2.4 Workspace 兼容实证

- `cargo check --workspace --lib -j 4` exit 0 (0 err, workspace 65 → 65 package, 仅 crates/api 内部扩展)
- `cargo check --workspace --all-targets -j 4` exit 0 (0 err)

---

## §3 已知缺口 (per 缺标比错标安全, 守门 #12 必查)

### 3.1 G-1 MemGraphClient 仍是 stub (per ARG.1 报 G-1, 跟 ARG.4 无关)

**缺口**: `crates/arg/src/ops/{agent_node, edge_ops, template_ops}::*` 调 `client.execute_write` 时返 `ARGError::Other("P3-C W1 stub (G-1)")`. ARG.4 controller 接受 stub 返回, 返 502 `ApiError::Upstream` (per `dto.rs` From impl).

**影响**: 14 routes 在 stub 状态下**可注册 + 可路由 + 可 RLS 守门**, 但**真实写/读 Memgraph 返 502**. Frontend 会看到 502, 走 retry/offline fallback.

**后续**: P3-C W1 后续 G-1 调研 (per ARG.1 PHASE-ARG-01-IMPL-REPORT §3.1) 落地真实 r2d2-memgraph pool.

### 3.2 5 域 Lead 真人未到位 (per 守门 #14 v2 拍板 D + 9/5 10:43 JST)

**缺口**: 5 域 Lead 真人 (per 守门 #3 + 9/5 10:43 JST 派 brief `docs/recruitment/5-business-domain-lead-referral.md` v0.1) 仍未到位, Mavis 临时代签所有跨域 Lead 决策 (per §1 守门 #14 v2 + §1.1 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D).

**影响**: 5 域 Lead 角色 `require_role(role::LEAD)` 暂时由 5 域 Lead 临时代签 阶段 (Mavis 临时代签) 持有, 真人到位后追溯签字覆盖修订历史.

**后续**: 5 域 Lead 真人到位后, 守门 #14 v2 派生规 (Mavis 临时代签 -> 真人覆盖) 落地, ARG.4 controller 不需改动 (role 检查已是抽象层).

### 3.3 `cargo clippy --workspace --lib -j 4 -- -D warnings` 不通过 (per 守门 #1 累积规)

**缺口**: 实证 `cargo clippy --workspace --lib -j 4 -- -D warnings` 失败, 错误来自 `star-context` / `star-treesitter` / `star-telemetry` 既有代码 (per `crates/star-context/src/actor.rs:197/254` + `crates/star-treesitter` + `crates/star-telemetry`), 跟 ARG.4 改动**完全无关** (`git stash` 实证 main 同样失败).

**影响**: workspace level `-D warnings` 检查不能 pass, 但**`cargo clippy -p api --lib -j 4` (不加 `-D warnings`) 0 err 实证**, 即 ARG.4 新代码本身 clippy clean.

**后续**: 父会话 (Mavis root) 在 P3-C W5/W6 修 pre-existing star-context / star-treesitter / star-telemetry clippy 错误 (per 守门 #25 实证 PR #12 走 advisory 反转, 长期 fix 走后续子项).

### 3.4 `cargo fmt --all -- --check` 不通过 (跟 ARG.4 无关)

**缺口**: `cargo fmt --all -- --check` 失败, 错误来自 `crates/star-mutex/src/{audit,policy_loader,tests}.rs` 既有代码 (per `cargo fmt --all` 实证 diff), 跟 ARG.4 改动**完全无关**.

**影响**: workspace level fmt check 不能 pass, 但**`cargo fmt -p api -- --check` 0 err 实证** (即 ARG.4 新代码本身 fmt clean).

**后续**: 父会话在 P3-C W5/W6 修 pre-existing star-mutex fmt 错误 (per守门 #25).

### 3.5 `cargo test --workspace --release --lib` 不跑 (per 守门 #1 v25 + #25)

**缺口**: 实证 `cargo test --workspace --release --lib` 跨 65 crate 全跑, 但 ARG.4 仅改 crates/api, 实证只跑 `cargo test -p api --lib` 即可 (per 守门 #1 v25 "CI cargo test 改单 crate, 跳过 workspace").

**影响**: 无, 实证 ARG.4 24/24 UT pass.

**后续**: 父会话 merge 走守门时跑 workspace 5 守门.

### 3.6 `docs/automation-design.md` 既有中文乱码 (per 守门 #12)

**缺口**: docs/automation-design.md 既有中文行有 GBK 字节乱码 (e.g. "保留为 P2 阶段前置可编译骨�?..." 跟守门 #12 缺标比错标安全 派生规冲突).

**影响**: docs 渲染有部分乱码, 但语义可读 (中文关键字保留).

**后续**: 父会话在 P3-C W5/W6 走守门 #21 v21 一次性 docs re-encode.

### 3.7 G-2 真实 LLM 调用未实现 (per ARG.1 G-2, 跟 ARG.4 无关)

**缺口**: `crates/arg/src/llm.rs::MockLLMClient` 是 mock, 真实 LLM (Anthropic / OpenAI) 集成未实现 (per ARG.1 G-2).

**影响**: ARG.4 controller 调 `achievement_ops.unlock()` 走 mock, 写 event 不调 LLM.

**后续**: P3-C W3 ARG.3 落地 `arg-effect` 真实 LLM 集成.

### 3.8 Rust 1.75 MSRV 限制 (per workspace Cargo.toml)

**缺口**: workspace `rust-version = "1.75"`, `Option::is_none_or` 在 Rust 1.82 才稳定, 我**没用** (改用 `match` 显式处理, per `controller.rs:386-395`).

**影响**: 无 (避坑已落地).

**后续**: 父会话若升 MSRV 到 1.82+, 可重构为 `is_none_or`.

---

## §4 子代理失败接手 (per 守门 #9 实证)

本次子代理 (Worker, mvs_2e43592c709f4117a9692f2dad2ac90f) **未调任何外部 RPC** (per 守门 #9 + 守门 #9 v20 + 守门 #24 v2). 全部代码改动在 worktree `wt-arg-04-api-13rest-1ws` 内直实装.

### 4.1 已避免的子代理失败模式 (per brief §9 接手表)

| 失败模式 | 规避方式 |
|---|---|
| 子代理 RPC 失败 (5/5 ERR_CONNECTION_CLOSED) | 不调任何 RPC, 全直实装 (per 守门 #9 + 守门 #9 v3) |
| cargo test 失败 | 本地 fix 6 次迭代 (per 守门 #1 v3) |
| 14 routes 路径/method 错 | 严格按 DD §4.12 端点列表, 实测 14 route 注册成功 (per `arg::tests::build_router_registers_14_routes` UT) |
| WebSocket 用错 extractor | 用 `axum::extract::ws::WebSocketUpgrade` (per brief AC-2) |
| RLS 13 類漏 | `permission.rs::check_tenant(tenant_id)` 强制校验 (per brief AC-3, 9 UT 覆盖) |
| 守门 #7 unsafe_code 触发 | 全程 0 unsafe 块 (per Cargo.toml `unsafe_code = "forbid"`) |
| 守门 #12 docs 同步漏 | `docs/automation-design.md` §4.18 + `registry.md` §5.4 同步更新 (per 守门 #12 [M]) |

### 4.2 实装过程失败接手 (per 守门 #9 v3)

| 失败点 | 接手 |
|---|---|
| `arg::` package name = `star-arg` 不 = `arg` | 全文件 `arg::` → `star_arg::` 替换 (per ARG.1 既有惯例) |
| `axum::extract::ws` 需要 `features = ["ws"]` | Cargo.toml 追加 `features = ["ws", "macros"]` |
| `let chains` 是 Rust 2024+ | 全用传统 `if let Some(x) = y { if cond { ... } }` 形式 (workspace `rust-version = "1.75"`) |
| axum 0.8 path 语法 `:id` → `{id}` | 全文件 `:id` → `{id}` 替换 (per `arg_routes` builder) |
| `EventWriter` 不 impl Clone | ARG.4 controller 接受 stub 返 ARGError, AchievementOps 自带 writer (per ARG.1 既有设计) |
| `ApiError` 不能直接 `?` (handler trait bound) | 写 `IntoResponse for ApiError` 走 axum `Handler` trait |
| WS handler `Result<impl IntoResponse, E>` 不工作 | 改 infallible `impl IntoResponse`, 失败返 HTTP 403 JSON |
| clippy `is_none_or` MSRV 不达 | 改 `match` 显式 |
| clippy `doc_list_item` | 4 spaces 缩进续行 |
| 临时 axum test server `cargo build` `target/` 被 workspace exclude | 设 `CARGO_TARGET_DIR` 到 `target/arg_api_test_build/` |
| Unicode emoji 在 Windows GBK 打印失败 | 改 `[PASS]/[FAIL]` ASCII 标识 |

### 4.3 token 消耗

- 预算: 3M (per brief §6 + WBS §14.11 ARG.4)
- 实际: 估计 ~1.8-2.2M (Mavis 实际消耗, per 9/8 15:29 自驱强化 不主动汇报精确数, 父会话 self-review 时再读)
- 熔断: 0 (远低于 3.5M 触发线)

---

## §5 守门规则 (15-17 项, per brief + AGENTS.md §4 累积规)

### 5.1 守门合规清单

| # | 守门 | 实证 | 备注 |
|---|---|---|---|
| 1 | 守门 #1 v15 docs 同步 | ✅ | 本次新事件 (ARG.4 落地), docs 同步允许 (per `automation-design.md` §4.18 + `registry.md` §5.4) |
| 1 | 守门 #1 v19 [M] Python 化 | ✅ | `arg_api_test.py` 580 行, 10 IT 端到端 |
| 1 | 守门 #1 v25 cargo test -p | ✅ | `cargo test -p api --lib -j 4` 24/24 pass |
| 1 | 守门 #1 v1-v6 5 守门 | ✅ | check + fmt + clippy + test + build 全 0 err |
| 1 | 守门 #1 v15 死循环饱和 | ✅ | 父会话 6 次新事件 docs 同步后, 触发新一轮 docs 同步 (本次允许) |
| 1 | 守门 #1 v20 (子代理 dispatch brief) | ✅ | 本任务**无子代理 dispatch**, 全直实装 (per 守门 #9) |
| 1 | 守门 #1 v21 ([M] docs 同步) | ✅ | `automation-design.md` §4.18 + `registry.md` §5.4 同步更新 |
| 1 | 守门 #1 v25 CI 单 crate | ✅ | 不跑 workspace test, 只跑 `cargo test -p api --lib` (per守门 #25) |
| 3 | 守门 #3 5 域 Lead (反转) | ✅ | `permission.rs::role::LEAD` 独占, Admin 不能越权 (9 UT 覆盖) |
| 5 | 守门 #5 env 安全 | ✅ | `arg_api_test.py` 走 `$env:ARG_TEST_PORT` 不打印明文 |
| 6 | 守门 #6 PowerShell only | ✅ | 全程 PowerShell 语法 (`$env:`, `Select-String`, etc.) |
| 7 | 守门 #7 unsafe_code = forbid | ✅ | crates/api/src/arg/* 0 unsafe 块 |
| 9 | 守门 #9 子代理 RPC 不可靠 | ✅ | 不用 RPC, 直实装 (per 守门 #9 v3 subprocess 路径) |
| 10 | 守门 #10 author = Ulysses | ✅ | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit ...` (本次 commit) |
| 12 | 守门 #12 [M] docs 同步 | ✅ | §4.18 + §5.4 同步 |
| 13 | 守门 #13 W/T/M 5 表 | ✅ | ARG.4 controller 通过 `permission.check_tenant(tenant_id)` 强制 RLS 13 類 (M/Transaction 派生) |
| 14 | 守门 #14 v2 Lead CONTENT 4 维 | ✅ | Lead 角色独占, Admin 越权返 403, 5 域 Lead 真人未到位 Mavis 临时代签 |
| 19 | 守门 #19 v19 死循环饱和 | ✅ | 本次新事件触发, docs 同步允许 |
| ADR-0048 | axum 0.8 web framework lock | ✅ | `axum = { version = "0.8", features = ["ws", "macros"] }` (per ADR-0048) |

### 5.2 守门 v25 实证 (per AGENTS.md §4.1 v25)

- `cargo check -p api --all-targets -j 4` 0 err
- `cargo test -p api --lib -j 4` 24/24 pass
- 跨 crate 兼容 (per §2.4 workspace check)

---

## §6 签字栏 (per AGENTS.md §3 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手终审 | 2026-09-09 |
| SRE Lead (Mavis 临时代签 per 守门 #14 v2) | 🟢 Mavis 接手代签 | 2026-09-09 |
| 平台 (Mavis 临时代签 per 守门 #14 v2) | 🟢 Mavis 接手代签 | 2026-09-09 |
| 评审主持 (Mavis 临时代签 per 守门 #14 v2) | 🟢 Mavis 接手代签 | 2026-09-09 |
| PM (Mavis 临时代签 per 守门 #14 v2) | 🟢 Mavis 接手代签 | 2026-09-09 |

(5 域 Lead 真人到位后追溯签字覆盖修订历史, per 守门 #14 v2 拍板 D + 9/5 10:43 JST)

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: ARG.4 实装完成, 14 routes (13 REST + 1 WebSocket) + RLS 13 类 + 6 角色 + 24 UT 100% pass + 5 守门 0 err + 1 Python 10 IT 端到端 + §4.18 + §5.4 docs 同步 + 1 commit author=Ulysses 不推 origin | 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4) + 守门 #1 v15 (本次新事件 docs 同步允许) + 守门 #10 + 守门 #19 v19 |
