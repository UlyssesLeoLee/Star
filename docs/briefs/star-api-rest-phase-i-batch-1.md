# Brief: star-api-rest Phase I Batch 1 — 8 路由接线（work-items/workspaces/worktrees）

> **Status**: 🟡 计划（per 2026-09-09 12:07 JST 用户拍板 Phase I 推进 + 守门 #9 v19 Mavis 自驱 + 守门 #14 v3 升级 + 守门 #19 agent 交互 Python 化）
> **For**: Mavis (root) / 子代理 worker（如派）
> **Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses
> **关联**: `docs/reports/STAR-P3-WBS-001.md` §14.12 / `docs/reports/STAR-API-REST-BACKEND-TAKEOVER-WBS-001.md` §1.2 / `docs/reports/HANDOFF-ST-001.md` v1.7 §20

---

## 0. 目的

把 `crates/star-api-rest/src/routes/` 下 8 个 501 stub handler 改成真实调用 `domain-work-item` / `domain-workspace` / `domain-worktree` 已有的 `InMemory*Service`（per Cargo.toml 已声明 3 个 path-dep），逐条对照 `star-mcp/src/tools/*.rs` 16 个已验证范式抄写（不是凭空重写），单测断言从 NOT_IMPLEMENTED 改为真实数据校验。

## 1. 8 路由清单（per 独立 WBS §1.1 表 #1-8）

| # | Method + Path | 当前文件 | 目标真实业务（domain crate） | MCP 范式来源（per 独立 WBS §1.1） |
|---|---|---|---|---|
| 1 | `GET /work-items` | `work_items.rs` | `domain_work_item::InMemoryWorkItemService::list` (或 search) | `crates/star-mcp/src/tools/search_issues.rs` |
| 2 | `GET /work-items/current` | `work_items.rs` | `domain_work_item::InMemoryWorkItemService::get_current_for_actor` | `crates/star-mcp/src/tools/get_current_task.rs` |
| 3 | `GET /work-items/{id}` | `work_items.rs` | `domain_work_item::InMemoryWorkItemService::get` | `crates/star-mcp/src/tools/get_issue.rs` |
| 4 | `POST /work-items` | `work_items.rs` | `domain_work_item::InMemoryWorkItemService::create` | (per `search_issues.rs` 反向) |
| 5 | `PATCH /work-items/{id}` | `work_items.rs` | `domain_work_item::InMemoryWorkItemService::update` | (per `get_issue.rs` 反向) |
| 6 | `GET /workspaces/{id}` | `workspaces.rs` | `domain_workspace::InMemoryWorkspaceService::get` | `crates/star-mcp/src/tools/get_workspace.rs` |
| 7 | `POST /worktrees` | `worktrees.rs` | `domain_worktree::InMemoryWorktreeService::create` | `crates/star-mcp/src/tools/create_worktree.rs` |
| 8 | `GET /worktrees/{id}` | `worktrees.rs` | `domain_worktree::InMemoryWorktreeService::get` | `crates/star-mcp/src/tools/get_worktree.rs` |

**依赖状态** (per `crates/star-api-rest/Cargo.toml` 实际状态): 3 个 path-dep (`domain-work-item` / `domain-workspace` / `domain-worktree`) 已声明, **0 改动需要**。

## 2. 改动矩阵

| 文件 | 当前状态 | 目标状态 | 行数估计 |
|---|---|---|---|
| `crates/star-api-rest/src/routes/work_items.rs` | 5 个 handler 全部 `RestError::not_implemented()` 一行 stub | 5 个 handler 真实调用 `InMemoryWorkItemService` 5 个方法 | +30-50 行 |
| `crates/star-api-rest/src/routes/workspaces.rs` | 1 个 handler stub | 1 个 handler 真实调用 `InMemoryWorkspaceService::get` | +10-20 行 |
| `crates/star-api-rest/src/routes/worktrees.rs` | 2 个 handler stub | 2 个 handler 真实调用 `InMemoryWorktreeService::create` + `::get` | +15-30 行 |
| `crates/star-api-rest/src/routes/mod.rs` | (无变化) | (无变化) | 0 |
| `crates/star-api-rest/Cargo.toml` | (无变化) | (无变化) | 0 |
| 单元测试 `crates/star-api-rest/src/routes/*.rs` 或 `tests/*.rs` | 1 个 negative `business_endpoint_returns_501_not_implemented` | 新增 8 个 positive 测试 (per 路由) + 1 个 negative 改为"已实现 200 OK"测试 | +40-80 行 |

**总数**: +95-180 行, 估 0.5-0.8M token（含读 + 写 + 验证）

## 3. 守门（per 独立 WBS §5 + AGENTS.md §4）

| 守门 | 内容 | 验证命令 |
|---|---|---|
| 守门 #1 v25 单 crate | `cargo test -p star-api-rest --lib -j 4` 必须 0 fail | `cargo test -p star-api-rest --lib -j 4` |
| 守门 #1 v19 自驱 | Mavis 拿到 §14.18 永久代签授权后自驱推进 | (per brief 触发) |
| 守门 #5 (per 独立 WBS §5 #3) | 新增 path-dep 后必须 `cargo check --workspace` 确认无循环依赖（本 batch 无新 dep, 仍跑兜底） | `cargo check --workspace --lib -j 4` |
| 守门 #4 (per 独立 WBS §5 #4) | 任何"看起来完成"的路由, 验收标准是真实 curl/集成测试返回非 501 + 返回体可验证 | (新单元测试 8 个 positive 实证) |
| 守门 #5 (per 独立 WBS §5 #5) | 持久化/鉴权暂不做, 必须在交付说明里显式写明 (本 batch 报告 §3 缺口部分) | (per 报告) |
| 守门 #7 (per 独立 WBS §5 #7) | Phase 完成后报告必须遵循 AGENTS.md §3 7 段结构 | (per `PHASE-STAR-API-REST-BATCH-1-REPORT.md`) |
| 守门 #10 author=Ulysses | commit author = `Ulysses <ulysses@mavis.local>` | (per `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`) |
| 守门 #12 commit-time docs 同步 | 报告 + WBS 升版 + HANDOFF 升版一并 commit | (per commit message) |
| 守门 #13 W/T/M | (本 batch 不涉及 DB schema, N/A) | — |
| 守门 #14 v3 永久代签 | Mavis 永久代签, 真人到位流程不阻塞 | (per 报告签字栏) |

## 4. 子代理边界（per AGENTS.md §4 #9 守门 + 独立 WBS §4）

| 子代理 | 范围 | 失败时如何判断 |
|---|---|---|
| A | 本 Batch 1 全部 8 路由 | `git log -p --follow` 实证改动真的落到 `routes/work_items.rs` / `workspaces.rs` / `worktrees.rs`, 不是只改了测试断言掩盖未接线; `cargo test -p star-api-rest --lib -j 4` 至少 8 个新 positive 测试 pass + 0 旧 negative 测试 fail |

**子代理 dispatch 必先 brief** (per 守门 #9 v20): 本 brief 已落档 `docs/briefs/star-api-rest-phase-i-batch-1.md`, 子代理 dispatch 时 commit message 引用本 brief 路径。

**子代理 RPC 不可靠** (per 守门 #9 v3 实证): 5 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded 历史实证, 任何子代理委派必 `git log -p --follow <wt-branch>` 实证 worktree commit 在 main 链上, 不沿用 RPC status。

**Mavis 优先** (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 JST 第 7 次强化): Mavis 拿到 §14.18 永久代签后, root 直实装本 Batch 1, 不派子代理, 避免 RPC 不可靠 + 节省 token。

## 5. 报告结构（per AGENTS.md §3 7 段结构）

1. §0 目的
2. §1 改动矩阵 / 任务完成矩阵
3. §2 验证摘要（实测 cargo test + cargo check + 单元测试 8 个 positive）
4. §3 已知缺口（per 缺标比错标: 持久化 InMemory / 鉴权 no-op / 7 路由 NOT_IMPLEMENTED 仍存 per Batch 2/3）
5. §4 子代理失败接手清单（per 守门 #9 v3）
6. §5 守门规则（per 本 brief §3 + 独立 WBS §5）
7. §6 签字栏（Mavis 永久代签 5 角色）
8. §7 修订历史（v0.X → v0.Y）

## 6. 下次 session 入口

| 任务 | 状态 | 备注 |
|---|---|---|
| Batch 2 10 路由接线 (code/context/MR/reviews/validations/submissions/pipelines) | 🟡 待启动 | 需先新增 3 个 path-dep `domain-search` / `domain-scm` / `domain-validation` |
| Batch 3 9 webhook 路由 | 🟡 待启动 | 原创接线, 无 MCP 范式 |
| Phase II 持久化决策 | 🟡 待 Ulysses 拍板 | 维持 InMemory vs 投入 SQLite/Postgres |
| Phase III 前端容器化 | 🟡 待启动 | 依赖 Phase I 至少部分完成 |
| Phase IV 鉴权真实化 | 🟡 待拍板 | 视 Phase III 后用户是否要多用户/对外暴露 |

## 7. Token 估

- 读 MCP 范式 16 文件 + 现状 stub 8 个: ~0.1M
- 写 8 个 handler 真实调用: ~0.2-0.3M
- 写 8 个新单元测试: ~0.1-0.2M
- 验证 (cargo check + test 跑 2 轮): ~0.05-0.1M
- 报告 + WBS 升版: ~0.05M
- **合计**: 估 0.5-0.8M, 1 sub-session 搞定

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-09 12:07 JST | 架构师 (Mavis 永久代签 Ulysses) | 初版: Phase I Batch 1 范围 + 8 路由清单 + 守门 + 报告结构 + Token 估 |
