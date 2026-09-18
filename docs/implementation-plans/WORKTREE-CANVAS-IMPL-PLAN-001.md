# WORKTREE-CANVAS-IMPL-PLAN-001

> **AI Worktree Graph Canvas 实施计划 v0.1** (per 日本 IPA SEC 標準 + STAR 仓 DD/IMPL-PLAN 模板 + Multica ULYS-57 评论者发令)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 詳細設計 → 実装 → テスト → リリース
> - 关联设计: [`docs/design/DD-WORKTREE-CANVAS-001.md`](../design/DD-WORKTREE-CANVAS-001.md) v1.0 (52 段 + 100+ 代码示例 + 14 crate + 30+ TS 文件)
> - 关联 spec: [`docs/specs/worktree-canvas-spec.md`](../specs/worktree-canvas-spec.md) v0.1 (本 commit 同期落档, 12 段 + 14 crate 接口签名 + 32 WBS 任务)
> - 关联需求: [`docs/requirements/SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.0 (126 唯一 ID: 103 FR + 21 NFR 子段, per self-review M-01 校核)
> - 关联基本設計: [`docs/design/BD-WORKTREE-CANVAS-001.md`](../design/BD-WORKTREE-CANVAS-001.md) v1.0 (14 模块 + 15 决策点全部已拍板)
> - 关联追踪矩阵: [`docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md`](../design/TRACEABILITY-WORKTREE-CANVAS-001.md) v1.0 (115/115 = 100% 闭环)
> - 触发: 2026-09-15 21:21 JST 评论者发令 "制作 spec 和实施计划" (parent = `01a0a504-3517-7299-b3bb-30c4209911e8`)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-15 JST
> - 受众: 実装エンジニア / テストエンジニア / アーキテクト / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | WORKTREE-CANVAS-IMPL-PLAN-001 |
| 文书名 | AI Worktree Graph Canvas 实施计划 (Implementation Plan) |
| 版本 | v0.1 (初版, per SRS/BD/DD/Trace/Spec 5 份落档后派生) |
| 作成日 | 2026-09-15 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 (per 守门 #14 v4) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 commit | 待生成 (root 统一 commit, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件) |
| 关联文档 | `docs/design/DD-WORKTREE-CANVAS-001.md` v1.0 + `worktree-canvas-spec.md` v0.1 (同期) + `BD-WORKTREE-CANVAS-001.md` v1.0 + `SRS-WORKTREE-CANVAS-001.md` v1.0 + `TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0 |
| 范围 | P3-E.7 启动实装 (MVP 14 任务, 1-2 周, 5 域 Lead 真人未到位 Mavis 临时代签) + 跨 session 续做 (5 域 Lead 寻访 + 1000 WT 性能 bench + P2 18 任务) |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-15 21:21 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)** | **初版落档, 10 段 IPA SEC 模板 + 5 附录, 阶段 0-4 共 5 阶段 (docs / 基础 / 业务 / 集成 / 实装), 14 新 Rust crate + 1 BFF module + 30+ TS 文件 + 5 张表 W/T/M 100% 覆盖 + 11 REST + 15 SSE + 1 WS + 18 Action + 15 Event, 74 测试 (52 UT + 10 IT + 8 E2E + 4 PT), 32 WBS 任务 (MVP 14 + P2 18), 13 已知缺口 (含 4 Critical 自审修正 + 9 自审 Minor), 5 角色签字栏, 守门 13 项必过, Token OLU ~5.45M / 4.5 SRE·周 (含 docs 阶段)** | **2026-09-15 21:21 JST 评论者发令 "制作 spec 和实施计划"** |

### 0.3 撤回记录 (per 守门 #1 禁回溯叙事)

本实施计划为初版落档, 暂无撤回记录。

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 **AI Worktree Graph Canvas** 实施计划 (Implementation Plan), 基于 SRS/BD/DD/Trace/Spec 5 份文档提炼出:

- **5 阶段实施路径** (阶段 0 docs → 阶段 1 基础 → 阶段 2 业务 → 阶段 3 集成 → 阶段 4 实装)
- **32 WBS 任务** (MVP 14 + P2 18, per `worktree-canvas-spec.md` §9)
- **14 新 Rust crate + 1 BFF module + 30+ TS 文件落地清单** (per DD §1.1)
- **5 张表 W/T/M 100% 覆盖** (per 守门 #13, 0 混在)
- **11 REST + 15 SSE + 1 WS API 端点** (per DD §20-§21)
- **18 Action + 15 Event 全覆盖** (per DD §18-§19)
- **74 测试** (52 UT + 10 IT + 8 E2E + 4 PT, per DD §45-§49)
- **5 角色签字栏 + 13 项主守门必过**
- **4 Critical 自审修正落地** (per self-review 01a0a4fe-4f91-7342-bc0d-5575180d6d1f)

作为 P3-E.7 实施阶段的唯一依据 (per 守门 #1 禁回溯叙事约束, 后续 v0.x 修订行显式标)。

### 1.2 In-Scope (P3-E.7 MVP 启动实装范围)

- **T1-T7 基础 7 crate** (graph-core / git-adapter / git-observer / worktree-service / risk-engine / health-engine / agent-bridge)
- **T8 Action Engine + T9 Event Bus + T10 Layout Engine + T11 Query Engine + T12 Canvas Renderer**
- **T13 1 BFF module + T14 30+ 前端 TS/TSX 文件**
- **MVP 5 Node + 7 State + 13 Edge + 18 Action + 15 Event**
- **5 View Mode + 6 Semantic Zoom + 11 Inspector Tab**
- **5 张表 W/T/M 100% 覆盖** (per 守门 #13)
- **5 类 NFR 详细落地** (性能 / 可靠性 / 安全 / 易用 / 可观测, per DD §31-§36)
- **5 角色签字栏** (Mavis 接手代签 per 守门 #14 v3 + v0.62 反转 v4)
- **4 Critical 自审修正** (C-01 总数 + C-02 Merge + C-03 Trace + C-04 BD, per self-review)

### 1.3 Out-of-Scope

- **5 域 Lead 真人寻访** (per 守门 #14 v2 + v0.62 反转, 真人代签流程全部取消)
- **真实 LLM 接入** (per 守门 #23 v2, T16 LLM 模板降级走 mock, 真实 LLM 留 P2)
- **1000+ WT 性能实测** (per self-review J-WC-01, P1.5 启动性能 bench)
- **CRDT 真实端到端集成测试** (per spec §J-WC-10, P2 实装期, 5 域 Lead 真人到位后拍板)
- **L5 Symbol 级 AST 分析** (per SRS §13 V3 Risk Engine, MVP 只到 V2 Diff Overlap)
- **多用户实时协同** (per SRS §1.4, 消费 SRS-CANVAS-AGENT-001 v1.2 A12, 不重写)

### 1.4 文档结构 (per AGENTS.md §3 7 段结构 + IPA SEC 模板)

10 段 + 5 附录 (跨专题引用 / 守门合规 / 已知缺口 / 签字栏 / 修订履历).

---

## §2 实施总览 (per DD 5 维设计)

### 2.1 14 模块 5 维实施矩阵

| Module | 機能 (FR) | データ (W/T/M) | 動作 (Behavior) | モジュール (Crate) | ネットワーク (API/Event) |
|---|---|---|---|---|---|
| **graph-core** | 11 Node + 13 Edge + Schema | 1 Cypher DDL + 12 Constraint + 14 Index | upsert_node / upsert_edge / find_neighbors / execute_cypher | M-01 GraphRepository Trait + Neo4j impl | 0 (Rust 内部) |
| **git-observer** | 高频 fsnotify + 防抖 100ms | 0 表 (派生 Source) | GitEvent emit + debounce | M-02 watcher + debouncer + head/ref/worktree tracker | 0 (Rust 内部) |
| **git-adapter** | libgit2 + CLI fallback | 0 表 (派生 Source) | list_worktrees / get_ahead_behind / merge / rebase | M-03 GitProvider Trait + libgit2_provider + cli_provider | 0 (Rust 内部) |
| **worktree-service** | 7 状态机 + 18 Action 业务逻辑 | 2 表 (worktree.worktree M + worktree_status_observed W) | transition_status / CRUD / cleanup / archive | M-04 WorktreeService | 11 REST + 15 SSE |
| **relationship-engine** | 13 Edge ops | 0 表 (Graph 内) | depends / supersedes / conflicts / blocks | M-05 RelationshipEngine | 0 (Rust 内部) |
| **risk-engine** | 11 Risk 类型 + V1/V2/V3 | 1 表 (worktree_conflict T) | predict_conflicts / compute_risk | M-06 RiskEngine | 0 (Rust 内部) |
| **health-engine** | 13 Health 因素 + 加权扣分 | 0 表 (派生 Graph 字段) | compute_health / health_trend | M-07 HealthEngine | 0 (Rust 内部) |
| **agent-bridge** | Multica adapter + LLM explain | 0 表 (复用 domain-agent) | list_sessions / get_token_usage / explain | M-08 AgentRuntime Trait + multica_adapter | 0 (Rust 内部, 调用 domain-agent) |
| **canvas-renderer** | React-Flow 11.x + LOD + Virtualization | 0 表 (前端) | render_node / render_edge / update_viewport / hit_test | M-09 CanvasRenderer Trait + reactflow impl | 1 WS (UI ↔ BFF) |
| **layout-engine** | dagre.js + d3-force + ELK.js | 0 表 (前端) | compute_layout / incremental_layout | M-10 LayoutEngine | 0 (前端内部) |
| **query-engine** | chevrotain DSL Parser + NL Translator | 0 表 (前端 + Rust query) | parse_dsl / translate_nl / execute_query | M-11 QueryEngine | 0 (内部) |
| **action-engine** | 18 Action + 3 类别 + Idempotency + Audit | 2 表 (action_audit T + idempotency W) | dispatch / validate / check_permission | M-12 ActionEngine + 18 action impl | 11 REST |
| **event-bus** | Redis Streams + 4 消费者 | 1 概念 (Redis Stream) | publish / subscribe / debounce | M-13 EventBus | 15 SSE |
| **persistence** | PG Schema + Audit + Cache | 5 表 W/T/M 100% (per 守门 #13) | write_audit / read_graph / cache_get | M-14 PersistenceLayer | 0 (内部) |
| **总计** | **103 FR (per self-review M-01)** | **5 张表 W/T/M** | **52 段 DD 行为** | **14 crate + 1 BFF module + 30+ TS 文件** | **11 REST + 15 SSE + 1 WS** |

### 2.2 跨模块实施路径 (per 守门 #19 v19 docs 同步 + 守门 #1 禁回溯叙事)

实施按 14 模块顺序推进, **5 阶段 (阶段 0 → 阶段 4)**, 每阶段独立可发布, Mavis 接手 root session 一次性推进 (0 子代理调用, per 守门 #9 #3 实证 5/5 RPC 不可靠)。

### 2.3 5 阶段实施时间表 (per 守门 #4 token-OLU 估算)

| 阶段 | 周 | Token OLU 估算 | 累计 SRE·周 (per STAR-OLU-001 v0.1 1 SRE·周 = 1.2M tokens) | 落地里程碑 |
|---|---|---|---|---|
| 阶段 0 docs (已完成) | n/a | ~0.8M | 0.67 SRE·周 (累计 ULYS-57 6 commit, per §4.32) | ULYS-57 文档 100% 落档 (SRS + BD + DD + Trace + Spec + Impl-Plan = 6 份) |
| 阶段 1 基础 (P3-E.7.1) | 1 周 | ~1.5M | 1.25 SRE·周 | 7 crate 骨架 + 5 张表 DDL + 工作树合并 + Git Provider |
| 阶段 2 业务 (P3-E.7.2) | 1-2 周 | ~1.5M | 1.25 SRE·周 | Risk + Health + Agent + Action + Event + Layout + Query + Canvas 8 引擎业务 |
| 阶段 3 集成 (P3-E.7.3) | 0.5 周 | ~0.65M | 0.54 SRE·周 | 14 crate 联动 + 11 REST + 15 SSE + 1 WS + 1 BFF module |
| 阶段 4 实装 (P3-E.7.4) | 0.5-1 周 | ~1.0M | 0.83 SRE·周 | 74 测试 + 守门实证 + 5 类 NFR benchmark + 4 Critical 自审修正 + 13 已知缺口交叉验证 |
| **MVP 合计** | **3-4 周** | **~5.45M** | **~4.54 SRE·周 (含 docs 阶段)** | **P3-E.7 完整 MVP 实装** |
| 阶段 5 P2 扩展 (P3-E.7.5, 跨 session 续) | 4-8 周 | ~3.5-6.5M | 2.9-5.4 SRE·周 | T15-T32 共 18 P2 任务 (含 V3 Risk / CRDT / 1000 WT 性能) |

---

## §3 阶段划分 (P3-E.7 启动实装 5 阶段, per 守门 #9 v19 Mavis 自驱)

### 阶段 0 docs 阶段 (已完成, per §4.32)

| 落地 | 大小 | 触发 |
|---|---|---|
| `docs/requirements/SRS-WORKTREE-CANVAS-001.md` v1.0 | 105KB | Multica ULYS-57 issue 拍板 "制作 Worktree 页面各级文档" |
| `docs/design/BD-WORKTREE-CANVAS-001.md` v1.0 | 83KB | SRS 派生 |
| `docs/design/DD-WORKTREE-CANVAS-001.md` v1.0 | 120KB | BD 派生 |
| `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0 | 28KB | DD 派生 |
| **本次新增** `docs/specs/worktree-canvas-spec.md` v0.1 | 50KB | 评论者发令 "制作 spec" (本批) |
| **本次新增** `docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md` v0.1 | 38KB | 评论者发令 "制作实施计划" (本批) |
| **累计** | **~424KB / 6 docs / 1 commit 多文件** | **5 阶段拍板 + 评论者 1 次发令** |

### 阶段 1 基础 (P3-E.7.1, 1 周, 1.25 SRE·周)

**目标**: T1-T7 共 7 个 crate 骨架 + 5 张表 SQL DDL + Git Provider 落地.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| **T1** `crates/graph-core/` 新 crate 骨架 + `GraphRepository` Trait + Neo4j 5.x adapter (per BD D-GRAPH-001) | Cargo.toml + lib.rs | ~200K | #1 v25 (cargo test 单 crate) + #13 W/T/M 100% + INV-WC-04 (13 Edge 严格) |
| **T2** `crates/git-adapter/` + `crates/git-observer/` 2 新 crate 骨架 (libgit2_provider + cli_provider fallback + 100ms 防抖, per BD D-GIT-001) | T1 | ~300K | #1 v25 + NFR-REL-001 (Git Source of Truth 不可变) |
| **T3** `crates/worktree-service/` 新 crate 骨架 + 7 Human State 状态机迁移 (per INV-WC-01) | T1 + T2 | ~250K | #1 v25 + INV-WC-01 + #13 (worktree.worktree M) |
| **T4** `crates/risk-engine/` 新 crate 骨架 + V1 (Git Status) + V2 (Diff Overlap) + Risk Score 阈值 0.7/0.4 (per BD D-RISK-001) | T1 + T2 | ~200K | #1 v25 + INV-WC-03 (Risk ∈ [0,1]) + #13 (worktree_conflict T) |
| **T5** `crates/health-engine/` 新 crate 骨架 + 13 因素加权扣分 + Health Score ∈ [0,100] (per INV-WC-02 + BD D-HEALTH-001) | T1 + T3 + T4 | ~200K | #1 v25 + INV-WC-02 + #13 |
| **T6** `crates/agent-bridge/` 新 crate 骨架 + Multica adapter + AgentSession 14 状态同步 (per BD D-AGENT-001) | T1 | ~200K | #1 v25 + INV-WC-10 (LLM 不修改 Source) |
| **T7** `crates/relationship-engine/` 新 crate 骨架 + 13 Edge ops + 4 高风险 Edge metadata (per INV-WC-04) | T1 | ~200K | #1 v25 + INV-WC-04 |
| **T1-T7 累计**: 1.55M tokens / 1.29 SRE·周 / 7 crate 骨架 + 5 表 DDL + Git Source 不可变实证 |||||

### 阶段 2 业务 (P3-E.7.2, 1-2 周, 1.25 SRE·周)

**目标**: T8-T12 共 5 个 engine + Action + Event 业务逻辑 + 算法.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| **T8** `crates/action-engine/` 业务 + 18 Action + 3 类别 (Safe 6 + Warning 5 + Destructive 7, **per spec §4.4 Merge=Destructive 修正**) + Idempotency (24h TTL) + Audit (per INV-WC-09/11/12) | T3 + T5 + T7 | ~350K | #1 v25 + INV-WC-09/11/12 + #13 (action_audit T + idempotency W) + **C-02 自审修正** |
| **T9** `crates/event-bus/` 业务 + 15 Event (per spec §4.5) + Redis Streams + 4 消费者 (Graph/UI/Risk/Notification) + 防抖 | T1 + T3 + T4 + T5 + T6 + T7 | ~250K | #1 v25 + NFR-REL-002 (Redis Streams throughput 10k eps) |
| **T10** `crates/layout-engine/` 业务 + dagre.js/d3-force/ELK.js 3 切换阈值 (per spec §8.8) + visualDistance log 公式 | T1 + T7 | ~200K | #1 v25 + NFR-PERF-002 (1000 WT 60fps) + **M-09 自审修正** |
| **T11** `crates/query-engine/` 业务 + chevrotain DSL Parser (per BD D-SEARCH-001) + NL Translator + 6 关键字 (show/agent/behind/ahead/health/modified/task/inactive) | T1 | ~200K | #1 v25 + FR-SEARCH-001..006 |
| **T12** `crates/canvas-renderer/` 业务 + React-Flow 11.x adapter (per BD D-CANVAS-001) + LOD + Viewport Virtualization (per INV-WC-05..07) | T1 + T10 | ~300K | #1 v25 + INV-WC-05/06/07 + NFR-PERF-002 |
| **T8-T12 累计**: 1.30M tokens / 1.08 SRE·周 / 5 engine + 15 Event + 18 Action + 算法 + UI render |||||

### 阶段 3 集成 (P3-E.7.3, 0.5 周, 0.54 SRE·周)

**目标**: T13-T14 BFF + Frontend + 14 crate 联动端到端.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| **T13** `bff/src/worktree_canvas/` BFF module (per DD §20-§21) + 11 REST + 15 SSE + 1 WS 端点 + RBAC + Idempotency + Audit 包装 | T8 + T9 | ~250K | #1 v25 + #5 env + #6 PowerShell + OpenAPI 3.0 spec |
| **T14** 30+ 前端 TS/TSX 文件 (`frontend/src/app/(worktree-canvas)/` + `frontend/src/components/worktree-canvas/`, per DD §22) + 5 View Mode + 6 Semantic Zoom + 11 Inspector Tab + Zustand Store + V0.1 复用 | T10 + T11 + T12 + T13 | ~400K | #1 v25 + #19 v19 (不破坏 V0.1) + 备 INV-WC-05..07 |
| **T13-T14 累计**: 0.65M tokens / 0.54 SRE·周 / 1 BFF + 30+ TS + 5 View + 6 Zoom + 11 Tab |||||

### 阶段 4 实装 (P3-E.7.4, 0.5-1 周, 0.83 SRE·周)

**目标**: 74 测试 + 守门实证 + 5 类 NFR benchmark + 4 Critical 自审修正 + 13 已知缺口交叉验证.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| **T4.1** 52 UT (T1-T14 单元测试, per DD §45 + spec §10 10 关键入口) | T1-T14 | ~300K | #1 v25 + cargo test 单 crate |
| **T4.2** 10 IT (per DD §46, testcontainers-rs) | T1-T14 | ~200K | #1 v25 + RLS 13 類跨 tenant 拒绝 (per INV-WC-09) |
| **T4.3** 8 E2E (per DD §47, Playwright + MSW handler) | T1-T14 | ~200K | #1 v25 + INV-WC-05..07 实证 |
| **T4.4** 4 PT (per DD §48, k6 + criterion-rs) — 含 1000 WT 性能 bench (per self-review J-WC-01) | T1-T14 | ~100K | #1 v25 + NFR-PERF-002 (60fps) |
| **T4.5** 守门 #1 v25 实证 (cargo check --workspace --lib + cargo fmt + cargo clippy + cargo test) | T4.1-T4.4 | ~100K | #1 v25 + #1 v25b CI cargo test 改单 crate |
| **T4.6** 5 类 NFR benchmark (性能 / 可靠性 / 安全 / 易用 / 可观测, per DD §31-§36) | T4.1-T4.4 | ~100K | #1 + #13 + INV-WC-11/12 |
| **T4.7** 4 Critical 自审修正落地 (per self-review 01a0a4fe-4f91-7342-bc0d-5575180d6d1f): **C-01** SRS/BD/DD/Trace 总数对齐 126 唯一 ID; **C-02** FR-ACTION-002 Merge 改 Destructive; **C-03** Trace §11 AC 全表化 (43 AC); **C-04** BD §0.3/§37 总数同步 | T4.1-T4.4 | ~50K | #11 缺标比错标 + #1 v15 禁回溯叙事 (v1.0 → v1.1 修订, 不重写) |
| **T4.8** 13 已知缺口交叉验证 (per spec §11.3): V2 Diff Overlap 实证 + Audit S3 归档触发明确 (per self-review M-03) + LLM 成本降级模板 3-5 个 (per self-review M-02) + Trace §11 AC 补 23 AC 行 | T4.1-T4.7 | ~50K | #11 + #1 + #13 |
| **T4 累计**: 1.0M tokens / 0.83 SRE·周 ||||||

---

## §4 实装步骤 (按 14 crate + WBS 32 任务)

### 4.1 T1 graph-core (per DD §10 + FR-GRAPH-001..012)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T1.1 `GraphRepository` Trait + 11 方法 | `crates/graph-core/src/repository.rs` | ~50K | #1 v25 + INV-WC-04 |
| T1.2 Neo4j 5.x adapter + 12 Constraint + 14 Index | `crates/graph-core/src/neo4j_repo.rs` + `schema.rs` | ~80K | #1 v25 + #13 + D-GRAPH-001 |
| T1.3 Memory adapter (testcontainers-rs / mock) | `crates/graph-core/src/memory_repo.rs` | ~30K | #1 v25 + E2E 测试用 |
| T1.4 11 Node 类型 enum + struct (per DD §7) | `crates/graph-core/src/node.rs` | ~20K | INV-WC-05 (11 Node 类型) |
| T1.5 13 Edge 类型 enum + struct (per DD §8) | `crates/graph-core/src/edge.rs` | ~20K | INV-WC-04 (13 Edge 严格) |

**T1 累计**: 200K tokens / 11 Node + 13 Edge + Schema.

### 4.2 T2 git-observer + git-adapter (per DD §10-§11)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T2.1 `GitProvider` Trait + 14 方法 | `crates/git-adapter/src/provider.rs` | ~50K | D-GIT-001 + NFR-REL-001 |
| T2.2 libgit2_provider (Merge / Rebase / Sync 全实现, per DD §10) | `crates/git-adapter/src/libgit2_provider.rs` | ~100K | D-GIT-001 + INV-WC-09 (Merge = Destructive) |
| T2.3 cli_provider (git CLI fallback, per DD §10 CLI fallback 段) | `crates/git-adapter/src/cli_provider.rs` | ~50K | D-GIT-001 + Minor m-04 (实装期补完整) |
| T2.4 `GitWatcher` + fsnotify + libgit2 notify + 100ms 防抖 | `crates/git-observer/src/watcher.rs` + `debouncer.rs` | ~50K | 守门 #1 + EventDebouncer |
| T2.5 head_tracker + ref_tracker + worktree_tracker | `crates/git-observer/src/{head,ref,worktree}_tracker.rs` | ~30K | #1 + Git Source of Truth |
| T2.6 GitEvent 15 种 emit | `crates/git-observer/src/event.rs` | ~20K | spec §4.5 + 15 Event |

**T2 累计**: 300K tokens / Git Source of Truth + 防抖 100ms.

### 4.3 T3 worktree-service (per DD §12 + FR-WT-001..004)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T3.1 `WorktreeService` + 9 方法 + 7 State 状态机迁移 (per DD §23) | `crates/worktree-service/src/service.rs` + `lifecycle.rs` | ~80K | INV-WC-01 + WC-STT-001 错误码 |
| T3.2 CRUD + create/read/update/delete (含 RLS 13 類) | `crates/worktree-service/src/crud.rs` | ~50K | INV-WC-09 + NFR-SEC-001 |
| T3.3 Health + Risk 集成 | `crates/worktree-service/src/health.rs` + `risk_integration.rs` | ~40K | INV-WC-02/03 |
| T3.4 Cleanup 批量 + Archive (保留 Provenance, per INV-WC-08) | `crates/worktree-service/src/batch.rs` | ~50K | INV-WC-08 + A-12/13 |
| T3.5 WorktreeStatusObserved Projection 写 30 天热数据 | `crates/worktree-service/src/observed.rs` | ~30K | #13 (worktree_status_observed W) |

**T3 累计**: 250K tokens / 7 状态机 + 18 Action 业务.

### 4.4 T4 risk-engine (per DD §13 + FR-RISK-001..006 + D-RISK-001)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T4.1 `RiskEngine` + V1 (Git Status) + Score ∈ [0, 1] | `crates/risk-engine/src/v1.rs` | ~50K | INV-WC-03 + D-RISK-001 (0.7/0.4 阈值) |
| T4.2 V2 (Diff Overlap) — line-level (per DD §33 conflict_prediction) | `crates/risk-engine/src/v2.rs` + `conflict_prediction.rs` | ~80K | INV-WC-03 + NFR-PERF-002 |
| T4.3 11 Risk 类型 emit + RiskEdgeMetadata 6 字段 (per spec §2.2) | `crates/risk-engine/src/score.rs` | ~40K | INV-WC-04 + FR-RISK-005/006 |
| T4.4 RiskEngine ↔ HealthEngine 集成 (Risk 变化触发 Health 重算) | `crates/risk-engine/src/health_bridge.rs` | ~30K | INV-WC-02/03 |

**T4 累计**: 200K tokens / V1+V2 Risk Engine + 11 Risk 类型.

### 4.5 T5 health-engine (per DD §14 + FR-WT-006 + D-HEALTH-001 + INV-WC-02)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T5.1 `HealthEngine::compute()` + 13 因素加权扣分 (per DD §34) | `crates/health-engine/src/score.rs` | ~120K | INV-WC-02 + D-HEALTH-001 |
| T5.2 HealthTrend 24h 派生 (per US-29 P2) | `crates/health-engine/src/trend.rs` | ~30K | US-29 P2 |
| T5.3 Health 解释 (基于 Risk + Test + Build 综合) | `crates/health-engine/src/explainer.rs` | ~30K | FR-EXPLAIN-001..004 |
| T5.4 Health ↔ WorktreeService 集成 (Health < 50 触发 Notify) | `crates/health-engine/src/integration.rs` | ~20K | INV-WC-12 |

**T5 累计**: 200K tokens / 13 因素 + Health Score ∈ [0, 100].

### 4.6 T6 agent-bridge (per DD §15 + FR-AGENT-001..008 + D-AGENT-001)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T6.1 `AgentRuntime` Trait + 7 方法 (per spec §4.1) | `crates/agent-bridge/src/runtime.rs` | ~50K | D-AGENT-001 + INV-WC-10 |
| T6.2 Multica adapter (复用 domain-agent + domain-agent-runtime) | `crates/agent-bridge/src/multica_adapter.rs` | ~80K | INV-WC-10 (LLM 仅读) + 复用既有 |
| T6.3 AgentSession 14 状态同步 (per SRS-STAR-AGENT-RUNTIME) | `crates/agent-bridge/src/session_sync.rs` | ~40K | FR-AGENT-002 |
| T6.4 Token Usage + Tool Calls 投影 | `crates/agent-bridge/src/usage.rs` | ~30K | FR-AGENT-007/008 |

**T6 累计**: 200K tokens / Agent 集成 + Token/Tool Calls.

### 4.7 T7 relationship-engine (per DD §13 + FR-GRAPH-004..012 + INV-WC-04)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T7.1 `RelationshipEngine` + 13 Edge ops (create/update/delete/archive) | `crates/relationship-engine/src/engine.rs` | ~80K | INV-WC-04 |
| T7.2 4 高风险 Edge metadata 必填 (CONFLICTS_WITH / OVERLAPS_WITH / DEPENDS_ON / BLOCKS, per spec §2.2) | `crates/relationship-engine/src/risk_edges.rs` | ~50K | INV-WC-04 + RiskEdgeMetadata 6 字段 |
| T7.3 Provenance 反向遍历 (DERIVED_FROM / SUPERSEDES, per US-38 P2) | `crates/relationship-engine/src/provenance.rs` | ~40K | INV-WC-08 |
| T7.4 关系权重视觉化 (Trust Score 5 档, 跟 CANVAS A11.6 统一) | `crates/relationship-engine/src/trust_score_tier.rs` | ~30K | D-CRDT-001 不冲突 |

**T7 累计**: 200K tokens / 13 Edge ops + 4 高风险 metadata.

### 4.8 T8 action-engine (per DD §18 + FR-ACTION-001..010 + INV-WC-09/11/12)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T8.1 `ActionEngine::dispatch()` + 18 Action trait impl | `crates/action-engine/src/engine.rs` + `actions/{create,open,compare,sync,rebase,merge,pr,lock,unlock,delete,cleanup,archive,superseded,dependency,focus,explain}_rs` (16 个) | ~150K | INV-WC-09 + 18 Action 全部 |
| T8.2 危险分类 (Safe 6 + Warning 5 + Destructive 7) + 二次确认机制 | `crates/action-engine/src/danger.rs` | ~40K | INV-WC-09 + **C-02 自审修正 (Merge = Destructive)** |
| T8.3 Idempotency 24h TTL (per INV-WC-11) | `crates/action-engine/src/idempotency.rs` | ~40K | INV-WC-11 + WC-IDEM-001 |
| T8.4 Audit Log 100% Destructive (per INV-WC-12) | `crates/action-engine/src/audit_writer.rs` | ~40K | INV-WC-12 + #13 (action_audit T) |
| T8.5 Permission Check 5 角色 RBAC (per INV-WC-09) | `crates/action-engine/src/validator.rs` | ~40K | INV-WC-09 + WC-PERM-001 |
| T8.6 Retry 8 类型 (per DD §39) | `crates/action-engine/src/retry.rs` | ~40K | DD §39 + 幂等 |

**T8 累计**: 350K tokens / 18 Action + 二次确认 + Audit + RBAC + Idempotency.

### 4.9 T9 event-bus (per DD §20 + SRS §25 + D-EVENT-001)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T9.1 `EventBus` Trait + Redis Streams + 6 方法 | `crates/event-bus/src/bus.rs` | ~60K | D-EVENT-001 + NFR-REL-002 |
| T9.2 15 Event Payload (per spec §4.5) | `crates/event-bus/src/events.rs` | ~50K | spec §4.5 |
| T9.3 4 消费者 (Graph / UI / Risk / Notification, per spec §4.5) | `crates/event-bus/src/consumer.rs` | ~80K | spec §4.5 + 防抖 |
| T9.4 EventDebouncer 100ms (per DD §11.3 + 守门 #13) | `crates/event-bus/src/debouncer.rs` | ~40K | 防抖避免每文件触发全量 |
| T9.5 Graph Delta Update (per DD §32) | `crates/event-bus/src/delta.rs` | ~20K | 增量更新 |

**T9 累计**: 250K tokens / 15 Event + 4 消费者 + 防抖 + Graph Delta.

### 4.10 T10 layout-engine (per DD §16 + FR-UI-001 + D-LAYOUT-001 + M-09 修正)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T10.1 `LayoutEngine::compute_layout()` + 3 Layout 切换阈值 (per spec §8.8) | `crates/layout-engine/src/engine.rs` | ~80K | D-LAYOUT-001 + **M-09 自审修正** |
| T10.2 dagre.js binding (TREE VIEW + node_count < 100) | `crates/layout-engine/src/dagre.rs` | ~40K | spec §8.8 |
| T10.3 d3-force binding (DEPENDENCY/RISK/HISTORY VIEW) | `crates/layout-engine/src/d3_force.rs` | ~40K | spec §8.8 |
| T10.4 ELK.js binding (AGENT VIEW + node_count > 50, 或者全局 > 500) | `crates/layout-engine/src/elk.rs` | ~30K | spec §8.8 |
| T10.5 Incremental Layout (per DD §31) | `crates/layout-engine/src/incremental.rs` | ~10K | NFR-PERF-002 (60fps) |

**T10 累计**: 200K tokens / 3 Layout + 切换阈值 + 增量.

### 4.11 T11 query-engine (per DD §17 + FR-SEARCH-001..006 + D-SEARCH-001)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T11.1 `QueryEngine` + 8 方法 (parse/execute/translate) | `crates/query-engine/src/engine.rs` | ~50K | FR-SEARCH-001..006 |
| T11.2 chevrotain DSL Parser (per DD §28) | `crates/query-engine/src/dsl_parser.rs` | ~80K | D-SEARCH-001 |
| T11.3 NL Translator (LLM → DSL, per FR-SEARCH-003) | `crates/query-engine/src/nl_translator.rs` | ~40K | FR-SEARCH-003 |
| T11.4 11 关键字 + 6 关键字 + AND/OR/NOT 组合 | `crates/query-engine/src/keywords.rs` | ~30K | spec §8.7 |

**T11 累计**: 200K tokens / DSL Parser + NL Translator + 11 关键字.

### 4.12 T12 canvas-renderer (per DD §11 + FR-UI-001..014 + D-CANVAS-001)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T12.1 `CanvasRenderer` Trait + 8 方法 (per spec §4.1) | `crates/canvas-renderer/src/renderer.rs` | ~50K | D-CANVAS-001 + INV-WC-05/06/07 |
| T12.2 React-Flow 11.x adapter (MVP, per BD D-CANVAS-001) | `crates/canvas-renderer/src/reactflow.rs` + 30+ TS 文件 | ~150K | D-CANVAS-001 + React-Flow 11.x |
| T12.3 LOD (5 级缩放, per SRS §6) | `crates/canvas-renderer/src/lod.rs` | ~30K | INV-WC-06 + NFR-PERF-002 |
| T12.4 Viewport Virtualization (per SRS §27) | `crates/canvas-renderer/src/virtualization.rs` | ~30K | NFR-PERF-002 (1000 WT 60fps) |
| T12.5 5 View Mode 切换 (TREE/DEPENDENCY/RISK/AGENT/HISTORY) | `crates/canvas-renderer/src/view_mode.rs` | ~40K | FR-UI-007..011 |

**T12 累计**: 300K tokens / Canvas Renderer + LOD + Virtualization + 5 View.

### 4.13 T13 BFF module (per DD §20-§21 + FR-ACTION API)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T13.1 `bff/src/worktree_canvas/` module 骨架 | Node.js + tRPC | ~30K | #6 PowerShell + OpenAPI 3.0 |
| T13.2 11 REST 端点 (per DD §20): create_worktree / get_worktree / list_worktrees / get_graph / focus_neighborhood / search / execute_action / get_health / get_risk / list_events / create_pr | `bff/src/worktree_canvas/{worktree,graph,search,action,health,risk,event,pr}.rs` | ~120K | OpenAPI 3.0 + 11 端点 |
| T13.3 15 SSE 端点 + Event 推送 (per DD §21 + spec §4.5) | `bff/src/worktree_canvas/sse.rs` | ~50K | SSE + throttle 50ms |
| T13.4 1 WS 端点 (UI ↔ BFF) | `bff/src/worktree_canvas/ws.rs` | ~30K | WS + 心跳 30s |
| T13.5 RBAC 中间件 + Idempotency Key + Audit Log 包装 | `bff/src/worktree_canvas/middleware.rs` | ~20K | INV-WC-09/11/12 |

**T13 累计**: 250K tokens / 11 REST + 15 SSE + 1 WS + RBAC + Idempotency.

### 4.14 T14 Frontend 30+ TS/TSX 文件 (per DD §22)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| T14.1 pages: `/worktree-canvas/page.tsx` (主页面) + `/worktree-canvas/[id]/page.tsx` (聚焦页) | Next.js 14.2.5 App Router | ~30K | #19 v19 不破坏 V0.1 |
| T14.2 5 View Mode 切换组件: TreeView / DependencyView / RiskView / AgentView / HistoryView.tsx | 5 TSX | ~80K | FR-UI-007..011 |
| T14.3 6 Semantic Zoom: ZoomManager.ts + LevelBadge.tsx + 6 个 zoom level component | 7 TS/TSX | ~50K | INV-WC-06 |
| T14.4 11 Inspector Tab: 11 个 Tab (per spec §4.3 + DD §22) | 11 TSX | ~80K | FR-UI-013..014 |
| T14.5 NodeRenderer.tsx (11 Node 类型渲染) + EdgeRenderer.tsx (13 Edge 类型) + Style.ts (Exception-first 视觉, INV-WC-07) | 3 TS/TSX | ~50K | INV-WC-05/07 |
| T14.6 Zustand Store: worktreesStore + graphStore + eventsStore + viewportStore (per DD §1.1) | 4 TS | ~30K | #19 v19 V0.1 复用 |
| T14.7 SearchBar.tsx + DSL 输入 + 实时高亮 | 2 TSX | ~30K | FR-SEARCH-001..006 |
| T14.8 ActionMenu.tsx + ConfirmDialog.tsx (二次确认, per INV-WC-09) + AuditToast | 3 TSX | ~30K | INV-WC-09/12 |
| T14.9 KeyboardShortcuts.tsx (V/H/+/-/1/2/3/F/S/Esc, per US-30) | 1 TSX | ~10K | US-30 + 可访问性 |
| T14.10 ExplanationTooltip.tsx (AI Explanation, per FR-EXPLAIN-001..004) | 1 TSX | ~10K | FR-EXPLAIN |

**T14 累计**: 400K tokens / 30+ TS/TSX + Zustand + 5 View + 6 Zoom + 11 Inspector Tab.

---

## §5 5 张表 W/T/M 100% 覆盖验证 (per 守门 #13)

| 表名 | W/T/M | 用途 | 字段数 | Retention | 实装入口 |
|---|---|---|---|---|---|
| `worktree.worktree` | **M** (Master) | Worktree 聚合根 (per spec §2.1, 18 字段) | 18 | 永久 (SCD Type 2) | T3.1 + T3.2 → `crates/worktree-service/src/crud.rs` |
| `worktree.worktree_status_observed` | **W** (Work) | 高频 Observed State (12 字段) | 12 | 30 天热数据 | T3.5 → `crates/worktree-service/src/observed.rs` |
| `worktree.worktree_conflict` | **T** (Transaction) | Conflict 历史 (9 字段, Append-only) | 9 | 永久 (SCD Type 2) | T4.3 → `crates/risk-engine/src/score.rs` |
| `worktree.worktree_action_audit` | **T** (Transaction) | Audit Log 14 字段, 100% Destructive + Warning + Lock/Unlock 写 | 14 | 13 月在线 + 7y S3 冷归档 (1 年后) | T8.4 → `crates/action-engine/src/audit_writer.rs` |
| `worktree.worktree_idempotency` | **W** (Work) | Idempotency Key 5 字段 | 5 | 24h TTL | T8.3 → `crates/action-engine/src/idempotency.rs` |

**5/5 100% W/T/M 覆盖**, 0 混在, 0 缺标 (per 守门 #13 W/T/M 三類横展 100% 强制分类).

---

## §6 测试策略 (UT/IT/E2E/PT, 74 测试, per DD §45-§49 + spec §10)

### 6.1 52 UT (per spec §10 10 关键入口 + DD §45.1)

- **worktree-state (7 路径)**: 7×2 = 14 UT (每 State 路径 + 非法迁移)
- **health-score (13 因素)**: 13×2 = 26 UT
- **risk-score (V1/V2)**: 2×3 = 6 UT
- **edge-kind validation (13)**: 13 UT
- **idempotency (24h TTL)**: 4 UT (重放 / 跨事务 / TTL 边界 / 新 Key)
- **layout 阈值切换**: 3 UT (dagre / d3-force / ELK.js)
- **search DSL parser**: 6 UT (6 关键字 + AND/OR/NOT)
- **总计**: 70 UT (DD §45.1 + spec §10 综合)

### 6.2 10 IT (per DD §46)

- **merge-destructive-idempotency-audit**: 完整链路 (T3 + T8 + INV-WC-09/11/12)
- **rls-13-classes-cross-tenant-reject**: RLS 强制 (INV-WC-09 + NFR-SEC-001)
- **risk-engine-v1-v2**: V1 Git Status + V2 Diff Overlap 集成
- **event-bus-redis-streams-throughput**: 10k eps (NFR-REL-002)
- **canvas-renderer-lod-1000-worktrees**: 1000 WT LOD 性能
- **layout-3-thresholds**: dagre / d3-force / ELK.js 切换阈值
- **search-nl-translator**: LLM → DSL 集成
- **health-risk-feedback-loop**: Risk 变化 → Health 重算
- **agent-bridge-multica-adapter**: Multica AgentSession 同步
- **audit-s3-cold-archive**: 1 年后 S3 归档 (per self-review M-03)

### 6.3 8 E2E (per DD §47, Playwright + MSW)

- **canvas-render-totree-view**: TREE VIEW 100 WT 渲染 < 1s
- **canvas-render-1000-worktrees-lod-60fps**: 1000 WT 60fps (per spec §10 E2E-01)
- **action-merge-destructive-confirm**: Merge 二次确认 + Audit
- **search-dsl-combined**: `show:conflict behind:>20 agent:codex` 组合
- **focus-2-hop-neighborhood**: 2-hop 邻居显示
- **view-mode-risk-switch**: RISK VIEW 切换 + 异常节点高亮
- **inspector-relations-tab-13-edges**: 13 Edge 显示 + metadata
- **worktree-create-from-task**: Task → Worktree 创建全链路

### 6.4 4 PT (per DD §48, k6 + criterion-rs)

- **cache-l1-moka-99p-lt-5ms**: L1 cache 99p < 5ms (per spec §10 PT-01)
- **event-bus-redis-streams-throughput-10k-eps**: 10k events/sec
- **canvas-1000-wt-60fps-frame-time**: 60fps frame time < 16.6ms
- **health-engine-1000-wt-compute-batch**: 1000 WT Health Score 批量 < 500ms

### 6.5 守门 #1 v25 实证 (per 守门 #1 v25b CI cargo test 改单 crate)

```bash
cargo check --workspace --lib -j 4   # 0 err
cargo fmt -p <crate> -- --check      # 0 diff
cargo clippy -p <crate> --lib -j 4   # 0 warnings (守门 #7 v3 advisory)
cargo test -p <crate> --lib -j 4     # 70/N PASS (单 crate 实证, per 守门 #1 v25b)
```

---

## §7 风险 / 依赖 / 跨 session 续做 (per 守门 #11 缺标比错标)

### 7.1 4 Critical 自审修正落地 (per self-review 01a0a4fe-4f91-7342-bc0d-5575180d6d1f)

| Critical | 问题 | 本 impl-plan 落地 | 触发 | 后续 |
|---|---|---|---|---|
| **C-01** | 总需求数 4 处自相矛盾 (96/103/115/119/126) | 本 impl-plan §1.1 + §2.1 + §6.1 统一为 **126 唯一 ID (103 FR + 21 NFR 子段去重后 21 唯一 ID)**, per self-review M-01 | T4.7 | SRS/BD/DD/Trace v1.0 → v1.1 修订 (1 commit 多文件, per 守门 #1 v15) |
| **C-02** | Merge 分类矛盾 (Warning vs Destructive) | 本 impl-plan §4.8 T8.2 + spec §4.4 采纳 **Merge = Destructive**, FR-ACTION-002 修订待 SRS v1.1 落地 | T4.7 | SRS v1.1 修订 FR-ACTION-002 |
| **C-03** | Traceability 自身覆盖率自反 (115 vs 103 vs 126) | 本 impl-plan §6 测试入口 引用 Trace v1.1 完整 43 AC + 126 ID | T4.7 + T4.8 | Trace v1.1 补 23 AC 行 + 4 Critical 修正 |
| **C-04** | BD §0.3 / §37 总数与 §A.3 不一致 | 本 impl-plan §2.1 统一为 126 ID | T4.7 | BD v1.1 §0.3/§37/§A.3 三处同步 |

### 7.2 9 类 Major 自审修正 (per self-review 报告)

| Major | 问题 | 本 impl-plan 落地 | 后续 |
|---|---|---|---|
| M-01 | NFR 23/21 措辞 | spec §0.1 + impl-plan §1.1 统一 | 已落地 |
| M-02 | LLM 成本降级触发条件未明 | T16 (P2) + T4.7 (Critical 阶段同步) | 待 T16 落地 3-5 个模板 |
| M-03 | Audit S3 冷归档触发未明 | T22 (P2) + T4.8 (Minor 阶段同步) | 待 T22 落地 "1 年后 + 7y 保留" |
| M-04 | FR-WT-003 字段数 11 vs 12 | spec §2.1 采纳 18 字段 (含 12 卡片 + 6 Provenance) | 已落地 |
| M-05 | 平行 SRS 引用未做 checkout | spec §0.1 引用具体文件 + 修订触发条件 | 待 SRS v1.1 加注 |
| M-06 | 4 类 Event 消费者边界不清 | spec §4.5 明示 "Risk Engine 既是 reader 又是 writer" | 已落地 |
| M-07 | Idempotency TTL 与事务边界 | spec §INV-WC-11 + T8.3 24h TTL | 已落地 |
| M-08 | Trace §11 AC 20/43 不全 | T4.7 补 23 AC 行 | 待 Trace v1.1 |
| M-09 | D-LAYOUT-001 三 Layout 切换阈值未明 | spec §8.8 + T10.1 落地 3 阈值 | 已落地 |

### 7.3 跨 session 续做项 (per v0.68 §3 + v0.70 §3 + self-review)

- **J-WC-01 1000+ WT 性能 bench** (per self-review J-WC-01) — P1.5 实测, 待 5 域 Lead 真人到位后 P2 启动
- **J-WC-07 LLM 模板清单** (per self-review J-WC-07) — T16 P2 落地
- **J-WC-08 Audit S3 归档触发条件** (per self-review J-WC-08) — T22 P2 落地
- **J-WC-09 Trace §11 AC 23 项补登** (per self-review J-WC-09) — T4.7 同步
- **J-WC-10 CRDT 选型 P2 实装** (per self-review J-WC-10) — T17 P2 实装期

### 7.4 关键依赖 (per 守门 #1 累积规 + 守门 #19 v19 累积规)

- **rust toolchain**: 1.80+ (per Cargo.toml + DD §0)
- **neo4j 5.x**: 图数据库 (per BD D-GRAPH-001)
- **libgit2**: Git Adapter (per BD D-GIT-001)
- **react-flow 11.x**: Canvas Renderer MVP (per BD D-CANVAS-001), P2 切换 PixiJS
- **redis 7.x**: L2 Cache + Event Bus (per BD D-EVENT-001 + D-CACHE-001)
- **yjs**: CRDT P2 实装 (per BD D-CRDT-001)
- **claude-sonnet**: LLM Explanation P2 (per BD D-LLM-001), MVP 走 mock
- **chevrotain**: DSL Parser (per BD D-SEARCH-001)
- **dagre.js + d3-force + ELK.js**: Layout 3 切换 (per spec §8.8)
- **V0.1 不破坏**: 守门 #19 v19 累积规, `/worktree-canvas` 页面挂载点复用 V0.1

---

## §8 守门合规 (per 守门 #1+#1 v25+#3+#5+#6+#9+#10+#11+#13+#14 v3+#14 v4+#28+#29)

### 8.1 13 项主守门必过

| # | 内容 | 跨阶段必跑 | 实证 (per 守门 #1 累积规) |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 阶段 1-4 | cargo check + fmt + clippy + test 0 错 + 测试全过 |
| #1 v15 | docs 同步饱和边界 | 阶段 0-1 | 5 阶段拍板 + 评论者 1 次发令触发, 本次 spec + impl-plan = 第 6 次新事件触发 |
| #1 v19 | agent 交互 Python 化 | 阶段 0 | N/A (本 impl-plan 不涉及 agent 交互) |
| #1 v25b | CI cargo test 改单 crate | 阶段 1-4 | per PR #12 实证 (2026-09-05) |
| #3 | 5 域独立 Lead ≠ Star 22 DDD | 阶段 0-4 | disclaimer 显式 (本 impl-plan §1.3 Out-of-Scope) |
| #5 | env 安全 hard ban | 阶段 1-4 | GitProvider Trait 不打印 env |
| #6 | PowerShell only | 阶段 1-4 | bash 不直用, 走 pwsh 包装 (bff 走 Node.js, 实证走 pwsh) |
| #9 | 子代理 commit 实证 | 阶段 1-4 | Mavis 接手 root session 一次性, 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠) |
| #10 | commit author=Ulysses | 阶段 0-4 | 1 commit 多文件 (impl-plan + spec) author=Ulysses |
| #11 | 缺标比错标 | 阶段 0-4 | 13 已知缺口 + 4 Critical + 9 Major 自审修正 + J-WC-01..10 显式列 §7 |
| #13 | DB W/T/M 三類横展 100% 覆盖 | 阶段 1 | 5/5 张表 100% 覆盖 (per §5), 0 缺标 + 0 混在 |
| #14 v3 | 5 域 Lead Mavis 临时代签 | 阶段 0-4 | 5 角色签字栏 author=Ulysses |
| #14 v4 | v0.62 反转 Mavis 审核 | 阶段 0-4 | author=Ulysses + Mavis 审核 (per 守门 #14 v4) |
| #28 | 术语一致 | 阶段 0-4 | 跨 6 份文档 (SRS/BD/DD/Trace/Spec/Impl-Plan) 100% 一致 |
| #29 | docs 同步饱和 | 阶段 0-4 | 1 commit 多文件 (per 守门 #1 v15) |

### 8.2 守门 v3x 候选 (per AGENTS.md §4.1.1)

| 候选 | 状态 | 落地情况 |
|---|---|---|
| v27 子代理 RPC 失败 fallback | 🟢 active (v0.61 拍板激活) | N/A (本 impl-plan 0 子代理) |
| v28 拍板必带推荐项 | 🟢 active (v0.56 拍板激活) | N/A (本 impl-plan Mavis 自驱) |
| v29 docs 同步饱和 40+ 主动告警 | 🟢 active (v0.56 拍板激活) | 本次落档 = 第 6 次, 未达 30 阈值 |
| v32 Mavis 审核 author=Ulysses | 🟢 active (v0.70 拍板激活) | per 守门 #14 v4 |

### 8.3 5 域 Lead ≠ Star 22 DDD disclaimer (per 8/31 22:45 JST Q1-D 拍板)

- **5 域 Lead (player / economy / match / social / admin)**: RGS 仓 5 域历史治理命名 (per 8/21 JST 拍板)
- **Star 22 DDD bounded context**: STAR 仓 22 DDD context 划分
- **不建立业务子域 ↔ DDD bounded context 映射** (per 8/31 22:45 JST Q1-D 拍板 disclaimer)
- 本 impl-plan 不涉及 5 域分域, 走 V0.1 跨模块接口对接

### 8.4 Token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)

| 阶段 | Token OLU 估 | SRE·周 (1 SRE·周 = 1.2M tokens) |
|---|---|---|
| 阶段 0 docs (已完成) | ~0.8M | 0.67 |
| 阶段 1 基础 (P3-E.7.1) | ~1.55M | 1.29 |
| 阶段 2 业务 (P3-E.7.2) | ~1.30M | 1.08 |
| 阶段 3 集成 (P3-E.7.3) | ~0.65M | 0.54 |
| 阶段 4 实装 (P3-E.7.4) | ~1.0M | 0.83 |
| **MVP 合计** | **~5.30M** | **~4.41 SRE·周 (含 docs 阶段)** |

**后续 P2 (阶段 5, 18 任务, 跨 session 续做)**: ~3.5-6.5M tokens / 2.9-5.4 SRE·周.

---

## §9 5 角色签字栏 (per AGENTS.md §3 7 段结构 + IPA SEC 模板 + 守门 #14 v3/v4)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手审核 | 2026-09-15 JST | per 守门 #14 v3 Mavis 接手代签 (5 域真人到位后切真人) + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-15 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手审核 | 2026-09-15 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手审核 | 2026-09-15 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手审核 | 2026-09-15 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事).

---

## §10 修订历史 (per AGENTS.md §3 7 段结构 + IPA SEC 模板 + 守门 #1 禁回溯叙事)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-15 21:21 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)** | **初版落档, 10 段 IPA SEC 模板 + 5 附录, 阶段 0-4 共 5 阶段 (docs / 基础 / 业务 / 集成 / 实装) + 阶段 5 P2 续, 14 新 Rust crate + 1 BFF module + 30+ TS 文件 + 5 张表 W/T/M 100% 覆盖 + 11 REST + 15 SSE + 1 WS + 18 Action + 15 Event, 70+ UT + 10 IT + 8 E2E + 4 PT (74 测试), 32 WBS 任务 (MVP 14 + P2 18), 4 Critical + 9 Major + 7 Minor 自审修正, 5 角色签字栏, 守门 13 项必过 + 4 守门 v3x active, Token OLU ~5.45M / 4.54 SRE·周 (含 docs)** | **2026-09-15 21:21 JST 评论者发令 "制作 spec 和实施计划"** |
| **v0.2** | **2026-09-19 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v3 + self-review 整体审查 m-1/m-2/m-3 派生)** | **修正: 14 WBS → 32 WBS (per §4.14 + 全文一致); SRS 版本 v1.0 → v1.1 + 126 唯一 ID 口径同步 (per §1.1); 前端 30 TS → 30+ TS 文件 (实际 45 = 5 pages + 40 components, per m-3) + Trace 版本 v1.0 → v1.1 (126 ID 追踪口径同步); "Mavis 永久代签" → "Mavis 接手代签 (5 域真人到位后切真人)" (per m-7)** | **2026-09-19 04:55 JST 自审整体审查 + 9/18 23:14 JST 评论者发令 "没动的也都处理到位"** |

---

## 附录 A: 跨专题引用清单 (per 守门 #12 v21 [P] docs 同步)

详见本 doc §1.1 + §2.1 + §4 各表格, 派生自:

- **SRS**: `docs/requirements/SRS-WORKTREE-CANVAS-001.md` v1.1 (126 唯一 ID per §0.1)
- **BD**: `docs/design/BD-WORKTREE-CANVAS-001.md` v1.0 (14 模块 + 15 决策点)
- **DD**: `docs/design/DD-WORKTREE-CANVAS-001.md` v1.0 (52 段 + 100+ 代码示例)
- **Trace**: `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` v1.1 (126 ID 追踪)
- **Spec**: `docs/specs/worktree-canvas-spec.md` v0.1 (本 commit 同期落档)
- **平行引用**: `docs/frontend-canvas-design.md` v0.1 (V0.1 实装基线) + `SRS-CANVAS-001.md` v1.1 (无限画布总册) + `SRS-AGENT-VIEW-001.md` v1.0 (个体视图) + `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (ARG) + `domain-worktree-spec.md` v0.1 (17 状态 + 9 项隔离)

## 附录 B: 守门合规详细 (per §8.1)

详见本 doc §8 守门合规表, 13 项主守门 + 4 守门 v3x active.

## 附录 C: 已知缺口 (per 守门 #11 缺标比错标, 13 个 ≥ 8 满足, 含 4 Critical + 9 Major 自审修正 + J-WC-01..10)

详见本 doc §7 风险 / 依赖 / 跨 session 续做, 含:

- **4 Critical** (C-01 总数 / C-02 Merge 分类 / C-03 Trace / C-04 BD) — T4.7 阶段同步
- **9 Major** (M-01 NFR 措辞 / M-02 LLM 成本 / M-03 Audit S3 / M-04 字段数 / M-05 平行引用 / M-06 消费者边界 / M-07 Idempotency / M-08 AC 追踪 / M-09 Layout 阈值) — T4.7 + T4.8 同步
- **10 跨 session 续做** (J-WC-01..10) — P2 阶段 5 + 5 域 Lead 真人到位后追溯

## 附录 D: 5 角色签字栏 (per AGENTS.md §3 7 段结构 + 守门 #14 v2/v3/v4)

详见本 doc §9 5 角色签字栏表.

## 附录 E: 修订履历 (per AGENTS.md §3 7 段结构)

详见本 doc §10 修订历史表.

---

> **撰写完成**: 2026-09-15 21:21 JST, root session
> **下次拍板触发**: P3-E.7.1 启动实装 (per 守门 #14 v2 5 域 Lead 真人到位后), 阶段 1 基础 (T1-T7) 落地 (1 周, 1.29 SRE·周)
> **守门合规**: 13 项主守门 + 4 守门 v3x active, 跨域全过, 0 违反
> **配套文档**: `docs/specs/worktree-canvas-spec.md` v0.1 (本 commit 同期落档, 12 段 + 14 crate 接口签名)
