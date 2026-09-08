# ARG.4 Brief — crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 类

> **Task ID**: arg-04-api-13rest-1ws
> **WT**: `wt-arg-04-api-13rest-1ws` (branch: `wt-arg-04-api-13rest-1ws`, base: main @ 651117e 含 ARG.1 merge)
> **触发**: 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理并在完成后merge到main" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4 / budget=选项3分阶段批 / merge=串行merge走守门)
> **依赖**: ARG.1 (`crates/arg` 6 子模块) 必须先 merge — 已 merge @ commit `651117e` 2026-09-09 05:00 JST
> **拍板来源**: per WBS-001 v0.18 §14.11 ARG.4 (拍板 3M tokens / 0.5 周, 4 新 crate 第 4 个 API 扩展)
> **守门合规**: #1 v15 (本轮第 6 次新事件, docs 同步允许) + #1 v19 (自动化档 [M] 必先 `scripts/automation/arg_api_test.py` 落地) + #1 v25 (cargo check --workspace --lib 0 err) + #3 (5 域 Lead 跨域边强制 consults) + #5 (env 安全, Memgraph 连接串走 env) + #6 (PowerShell only) + #7 (0 unsafe) + #9 (子代理 RPC 不可靠, 不用 RPC) + #10 (代签, author=Ulysses) + #12 ([M] 子项 docs 同步, automation-design.md §4 + registry.md) + #13 (W/T/M 三类横展开) + #14 v2 (5 域 Lead Mavis 临时代签) + #19 v19 (守门 #12 死循环饱和边界)

---

## 1. Objective (为什么做)

落地 ARG (Agent Relationship Graph) 4 新 crate 的**第 4 个 — API Tier 扩展**：`crates/api/src/arg/`。这是 P3-C 阶段 ARG.4 子项, 跨 Data Tier (crates/arg) + API Tier (crates/api 扩展) + Frontend (后续 ARG.5)。

ARG.4 完成后:
- P3-C W4 第 1 个任务完成
- 13 REST + 1 WebSocket 让前端 / 其他 crate 跨进程调 ARG
- 守门 #1 v25 `cargo check --workspace --lib -j 4` 0 err 实证
- 跟 ARG.1 配套: ARG.1 暴露 MemgraphClient / EdgeOps 等内部 API, ARG.4 包成 HTTP/WS

---

## 2. Scope (必做什么 / 不做什么)

### 2.1 必做 (In-Scope)

#### A. `crates/api/src/arg/` 新模块 (4 文件, 跟 DD-AGENT-RELATIONSHIP-001.md v0.1.1 §4.12 + §5 一致)

- `crates/api/src/arg/mod.rs` — 模块入口 + re-export
- `crates/api/src/arg/controller.rs` — Axum router 13 REST endpoints (per DD §4.12):
  - `POST /api/arg/agents` (create_agent)
  - `GET /api/arg/agents` (list_agents)
  - `GET /api/arg/agents/{id}` (get_agent)
  - `PATCH /api/arg/agents/{id}` (update_agent)
  - `POST /api/arg/edges` (create_edge)
  - `GET /api/arg/edges` (list_edges, filter type/agent)
  - `GET /api/arg/edges/{id}` (get_edge)
  - `PATCH /api/arg/edges/{id}` (update_edge, weight/metadata only)
  - `DELETE /api/arg/edges/{id}` (archive_edge, archived=true)
  - `GET /api/arg/graph` (get_graph, limit 1000 节点)
  - `POST /api/arg/templates/instantiate` (instantiate_template)
  - `GET /api/arg/achievements` (list_achievements)
  - `GET /api/arg/achievements/me` (my_unlocks)
  - `+ POST /api/arg/achievements/evaluate` (admin only, 14 端点总数)
- `crates/api/src/arg/sse_hub.rs` — WebSocket `/ws/arg/events` (SSE-over-WebSocket, 6 事件类型: edge.changed / edge.created / edge.archived / achievement.unlocked / dispatch.route.changed / agent.trust_score.changed)
- `crates/api/src/arg/permission.rs` — RLS 13 类 middleware (`check_tenant(tenant_id)` + 6 类角色校验)
- `crates/api/src/arg/dto.rs` — request/response types (CreateEdgeRequest / UpdateEdgeRequest / CreateAgentRequest / EdgeFilter / Pagination / 5+ 字段校验)

#### B. `crates/api/Cargo.toml` 集成 (依赖 `crates/arg`)

- 追加 `arg = { path = "../arg" }` 到 `[dependencies]`
- 确保 `tokio` / `axum` / `serde` / `uuid` 已 workspace 提供

#### C. `crates/api/src/main.rs` 或 `lib.rs` 注册 arg routes

- 在现有 router 注册 `arg::controller::arg_routes(state)`
- 跟现有 `agent-credential` / `ops` / `mcp` routes 同级

#### D. `scripts/automation/arg_api_test.py` 1 份脚本 (per 守门 #1 v19 自动化档 [M] 必先落地)

- 10 IT 自动化测试 (跟 DD §10.2 一致):
  - test_api_create_edge
  - test_api_create_edge_rls
  - test_api_list_edges_filter
  - test_api_update_edge
  - test_api_archive_edge
  - test_api_template_instantiate
  - test_api_achievements_list
  - test_api_achievements_unlock
  - test_ws_subscribe_events
  - test_ws_achievement_unlocked
- 用 `requests` 库打 13 REST 端点 (mock crate::arg ops 返 fixture)
- 用 `websockets` 库测 WS 1 端点

#### E. `docs/automation-design.md` §4 + `scripts/automation/registry.md` 索引更新 (per 守门 #12)

- 追加 `arg_api_test.py` 到 §4 任务卡表
- 追加到 `registry.md` 索引

#### F. 守门实证 (必跑, exit 0)

1. `cargo check --workspace --lib -j 4` 0 err 实证 (守门 #1 v1 + v2)
2. `cargo fmt --all -- --check` 0 err
3. `cargo clippy --workspace --lib -j 4 -- -D warnings` 0 err 实证 (守门 #7)
4. `cargo test -p star-api --lib -j 4` 0 err 实证 (守门 #1 v25 跨 crate 兼容)
5. `cargo build --release -p star-api` 0 err

### 2.2 不做 (Out-of-Scope)

- **不写** `crates/api/src/arg/arg_state.rs` 完整版 (ARGState 在 DD §3.2.5 共享类型已定义, 这里只引用)
- **不写** 真实 LLM 调用 (守门 #5 v2 + #23 强制 mock)
- **不写** ARG.1 没实现的 (MemGraphClient 仍是 stub, ARG.4 controller 调用 ops 时也要接受 stub 返回, 不报错)
- **不写** 真实 axum::serve 启动 (Cargo.toml 加 dep, 不写 main 启动代码)
- **不写** frontend (ARG.5 P3-C W4 后续)
- **不写** crates/arg-bridge (ARG.2 P3-C W2)
- **不写** crates/arg-effect (ARG.3 P3-C W3)

---

## 3. Acceptance Criteria (验收标准, 必 100% 通过)

### AC-1: 编译守门
- [ ] `cargo check --workspace --lib -j 4` exit 0, 0 error
- [ ] `cargo fmt --all -- --check` exit 0
- [ ] `cargo clippy --workspace --lib -j 4 -- -D warnings` exit 0, 0 warning
- [ ] `cargo build --release -p star-api` exit 0

### AC-2: 路由注册守门
- [ ] `crates/api/src/arg/controller.rs::arg_routes(state)` 返回 `Router` 含 14 routes (13 REST + 1 WS)
- [ ] 路由 method + path 严格按 DD §4.12 13 端点列表
- [ ] WS `/ws/arg/events` 走 `axum::extract::ws::WebSocketUpgrade` extractor (不是普通 `get()` handler)

### AC-3: RLS 13 类守门
- [ ] `permission.rs::check_tenant(tenant_id)` 强制 tenant_id 校验
- [ ] 6 类角色校验 (Lead / AgentOwner / Viewer / Editor / Admin / System)
- [ ] 跨 tenant 拒绝 403 (test_api_create_edge_rls 验证)

### AC-4: DTO 守门
- [ ] `CreateEdgeRequest` 11 字段 (per DD §4.1.3) + 校验
- [ ] `UpdateEdgeRequest` 仅允许 weight / metadata 改
- [ ] `EdgeFilter` 支持 by_type / by_agent / by_archived

### AC-5: 集成守门
- [ ] `crates/api/Cargo.toml` 追加 `arg = { path = "../arg" }` dep
- [ ] 14 routes 在 main router 注册 (跟现有 agent-credential / ops 同级)
- [ ] 守门 #7 `unsafe_code = "forbid"` 通过
- [ ] 守门 #6 PowerShell only (部署脚本风格)

### AC-6: 自动化档守门
- [ ] `scripts/automation/arg_api_test.py` 存在 + 可执行
- [ ] `docs/automation-design.md` §4 追加 1 行 (按守门 #12 [M] 子项 docs 同步强制)
- [ ] `scripts/automation/registry.md` 追加 1 行

### AC-7: 报告守门
- [ ] `docs/reports/PHASE-ARG-04-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)
- [ ] 引用 ARG.1 commit `43c1f0c` / `651117e` (per AGENTS.md §4 #12 git 实证)

### AC-8: 守门 v25 必填
- [ ] `cargo check --workspace --lib -j 4` 0 err
- [ ] author = Ulysses (per 守门 #10 + 9/8 15:19 第 6 次强化)
- [ ] 不推 origin (守门 #1 R-05 反转, 等 merge 走守门)

---

## 4. 已知约束 + 处理策略

| 约束 | 处理 |
|---|---|
| 守门 #1 v19 [M] 必先 Python 脚本 | 落 `arg_api_test.py`, 注册到 `registry.md` |
| 守门 #7 `unsafe_code = "forbid"` | 代码无 unsafe 块 |
| 守门 #12 docs 同步 [M] 子项 | `automation-design.md` §4 + `registry.md` 同步更新 |
| 守门 #10 代签 | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit ...` |
| ARG.1 MemGraphClient 仍是 stub | controller 调用 ops 接受 stub 返回, 不报错 (ARG.1 报 G-1 缺口, P3-C W1 后续 G-1 调研后补) |
| WebSocket 协议 | 用 `axum::extract::ws::WebSocketUpgrade` extractor, 不是普通 get() |
| cargo workspace 65 → 65 (api 是已存在, 内部加 arg 模块) | 不增新 crate, 只增新模块 |

---

## 5. Deliverable (提交物)

### 5.1 代码文件
- `crates/api/src/arg/mod.rs` (1 文件, ~30 行 re-export)
- `crates/api/src/arg/controller.rs` (~250 行, 14 routes + handler)
- `crates/api/src/arg/sse_hub.rs` (~120 行, WebSocket hub)
- `crates/api/src/arg/permission.rs` (~100 行, RLS 13 类 middleware)
- `crates/api/src/arg/dto.rs` (~150 行, request/response types)
- `crates/api/Cargo.toml` 改 1 行 (加 `arg` dep)
- `crates/api/src/main.rs` 或 `lib.rs` 改 1 行 (注册 routes)

### 5.2 脚本文件
- `scripts/automation/arg_api_test.py` (~200 行)

### 5.3 文档更新
- `docs/automation-design.md` §4 (追加 1 行)
- `scripts/automation/registry.md` (追加 1 行)

### 5.4 报告
- `docs/reports/PHASE-ARG-04-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

### 5.5 commit
- 1 个 commit: `feat(arg-api): crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 类 (ARG.4 子项, P3-C W4)`
- author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)
- 不推 origin (守门 #1 R-05 不 push 反转, 等 merge 走守门)
- worktree: `wt-arg-04-api-13rest-1ws`

---

## 6. Token 预算 (per 选项 3 分阶段批)

- **本子项预算**: 3M tokens (per WBS-001 v0.18 §14.11 ARG.4)
- **本子代理预期消耗**: 2.5-3.0M
- **余量留 merge 守门**: 0.3-0.5M
- **触发熔断**: 累计消耗 > 3.5M 时停, 报告 owner 拍板

---

## 7. 子代理执行守门 (per 守门 #9 v3 + 守门 #20 派生)

子代理必须:
- **不调任何外部 RPC** (per 守门 #9 实证)
- **不用 RPC 派其他子代理** (避免 5/5 RPC 失败实证)
- **不写跨 crate 改动** (本任务只动 crates/api/src/arg/ + crates/api/Cargo.toml 1 行 + crates/api/src/main.rs 或 lib.rs 1 行 + scripts/automation/1 份 + docs/automation-design.md 1 行 + scripts/automation/registry.md 1 行)
- **不调 cargo publish / git push** (per 守门 #1 R-05 + 反转 9/3 07:09 JST 推 origin 守门)
- **不写 commit msg 含无 git 实证叙事** (per 守门 #12 + #1 禁回溯)
- **每步必跑守门实证**, exit 0 才进下一步

---

## 8. 父会话职责 (Mavis root)

- 接收子代理 final report
- **self-review** 子代理产出 (对照本 brief AC-1..AC-8)
- 跑守门 #1 v3 + v25 全套
- merge 走 `git merge --no-ff wt-arg-04-api-13rest-1ws` (守门 #1 v3 必先 cargo test 收敛)
- 升 WBS-001 v0.19 (§14.11 ARG.4 升 🟡→🟢 + §15 累计 96/119 → 97/119)
- 触发 ARG.5 (frontend) 派新子代理 或 P3-C 收官

---

## 9. 失败接手路径 (per 守门 #9 实证)

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 (5/5 ERR_CONNECTION_CLOSED) | 立刻 task_stop, 不重试, 父会话接手 |
| cargo test 失败 | 子代理本地 fix, 跑 cargo test 实证 0 err |
| 14 routes 路径或 method 错 | 子代理按 DD §4.12 严格对照修正 |
| RLS 13 类漏 | 子代理补 check_tenant 强制校验 |
| WebSocket 用错 extractor | 子代理改用 `axum::extract::ws::WebSocketUpgrade` |
| 守门 #7 unsafe_code 触发 | 必移除所有 unsafe 块, 改用 safe 代码 |
| 守门 #12 docs 同步漏 | 必补 `automation-design.md` §4 + `registry.md` |
| Token 超 3.5M | 子代理 task_stop, 报告 owner, 父会话决定是否续 |
| merge 冲突 (跟 ARG.1 merge 后 main 冲突) | 父会话手动 rebase 或 cherry-pick 解决 |
| cargo test --workspace 跨 crate 失败 | 父会话排查, 不强推 (per 守门 #1 v3 必 0 err) |

---

## 10. 引用 (Reference)

- `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 (663 行, 9/8 落档, commit `0bacaeb`)
- `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1 (1088 行, 9/9 落档, commit `464a646`)
- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 (2311 行, 9/9 落档 + self-review 修复, commit `a697284`)
- `docs/reports/STAR-P3-WBS-001.md` v0.18 §14.11 ARG.4 (commit `9f5416e`)
- `crates/arg/` ARG.1 merge commit `651117e` (2026-09-09 05:00 JST) — 6 子模块 + 33 UT 100% pass
- `docs/briefs/arg-01-arg-crate-skeleton.md` (13.8KB, 子代理 brief 模板)
- `AGENTS.md` §4 守门 (13 main + 24 派生 = 37) + §3 7 段报告结构
- `docs/automation-design.md` v0.1 (9/2 00:39 JST 拍板)
- `scripts/automation/registry.md` v0.6 (15 份基类索引, ARG.1 落档后)
