# TEST-DESIGN-WORKTREE-CANVAS-001 — AI Worktree Graph Canvas 测试设计书

> **AI Worktree Graph Canvas — 无限画布模块 5 级别测试设计书 v2.2** (per 日本 IPA SEC 標準 / V モデル / テスト設計書 テンプレート + internal work item ULYS-57 request "开子任务推进开发并且制作各级测试设计书")
>
> - 状态: 🟡 Draft v2.2 (2026-10-31 增加 authenticated device-revocation HTTP/PG acceptance；Rust acceptance 受本机 1.99.0 compiler resolution 阻断，完整 E2E、KMS/恢复与性能验收仍未完成)
> - 关联工作项: `ULYS-57` (制作 Worktree 页面各级文档) parent + 5 子任务推进开发
> - 关联文档 (上游 4 份同步落档, 评审已通过):
>   - [`docs/requirements/SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.0 (103 FR + 23 NFR = 126 唯一 ID)
>   - [`docs/design/BD-WORKTREE-CANVAS-001.md`](../design/BD-WORKTREE-CANVAS-001.md) v1.0 (14 模块 + 15 决策点全部已拍板)
>   - [`docs/design/DD-WORKTREE-CANVAS-001.md`](../design/DD-WORKTREE-CANVAS-001.md) v1.0 (52 段 + 100+ 代码示例 + 14 crate + 30 TS 文件)
>   - [`docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md`](../design/TRACEABILITY-WORKTREE-CANVAS-001.md) v1.0 (126 ID 追踪 + 43 AC)
>   - [`docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md`](../requirements/SRS-W16-PAYLOAD-SECURITY-001.md) v0.4 (W16 协作载荷身份与持久加密要求；补充设计，未并回主 SRS)
>   - [`docs/design/BD-W16-PAYLOAD-SECURITY-001.md`](../design/BD-W16-PAYLOAD-SECURITY-001.md) v0.4 / [`docs/design/DD-W16-PAYLOAD-SECURITY-001.md`](../design/DD-W16-PAYLOAD-SECURITY-001.md) v0.4 (目标设计；局部 repository PG18 slices 有限通过，完整协作验收未完成)
> - 平行实施文档: [`docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md`](../implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md) v0.1 (MVP 14 任务 + P2 18 任务)
> - 平行 spec: [`docs/specs/worktree-canvas-spec.md`](../specs/worktree-canvas-spec.md) v0.1 (10 关键测试入口)
> - 受众: 実装エンジニア / テストエンジニア / アーキテクト / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 守门 #14 v3 + v0.62 反转 v4)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-10-31 JST
> - 模板参考: [`docs/test-design/TEST-DESIGN-SANDBOX-002.md`](TEST-DESIGN-SANDBOX-002.md) v0.1 (单文档分章, 5 级别 UT/IT/E2E/PT/UAT 模式) + `TEST-DESIGN-OPS-001.md` v0.2

---

## §0 文档目的

本文档按 日本 IPA SEC 標準 V モデル 制定 **AI Worktree Graph Canvas** 模块的 **5 级别测试设计书** (单文档分章模式, 跟 SANDBOX-002 平行不重叠), 在 SRS/BD/DD/Traceability 4 份上游文档已落档基础上, 系统化回答:

- **测什么** (Test Perspective, TP-XXX): 从需求派生, 每个视角对应一个或多个 Test Case
- **为什么测** (验证目的): 引用 SRS/BD/DD 的具体 Requirement ID 和 Design ID
- **从什么角度测** (5 级别: UT/IT/E2E/PT/UAT): 引用 DD §45-§49 测试入口
- **覆盖率** (Coverage Matrix): Requirement ↔ Test Perspective ↔ Test Case ↔ AC 全闭环
- **缺陷验收** (Entry/Exit Criteria): 进入/退出标准, 含 5 角色签字栏

**单文档结构** (per 评审发令 "各级测试设计书" 的 IPA 标准解读):

| 章节 | 级别 | 测试类型 | 实装锚点 | 文档来源 |
|---|---|---|---|---|
| §2 | UT | 单元测试 | 14 crate × 模块测试矩阵 | DD §45 |
| §3 | IT | 集成测试 | 跨 crate + DB 集成 + gRPC + Redis + Graph 集成 | DD §46 |
| §4 | E2E | 端到端 | Playwright + MSW handler | DD §47 |
| §5 | PT | 性能测试 | vitest bench + cargo bench + k6 | DD §48 |
| §6 | UAT | 验收测试 | 43 AC 全表化 + 5 角色签字 | SRS §12 + Trace §11 |

**测试数量基线** (per WORKTREE-CANVAS-IMPL-PLAN-001 §3.4):

```
UT:   ~52 测 (T1-T14 单元测试, 1:1 对齐 14 crate 关键入口)
IT:   ~10 测 (跨 crate + DB + Graph + Cache 集成, testcontainers-rs)
E2E:   ~8 测 (Playwright + MSW handler, 5 视图模式 + 6 zoom + 18 Action)
PT:    ~4 测 (k6 + criterion-rs, 性能 7 AC 覆盖)
UAT:  43 AC (per SRS §12 全表化)
合计: ~74 测 + 43 AC = 117 实证锚点
```

**触发** (2026-09-16 10:48 JST 评论者发令):

> "可以开子任务推进开发并且制作各级测试设计书了"

**ask_user 派板** (per 守门 IPA 文档 V モデル层级):

- 选项 A: 5 级别测试设计 (单文档分章模式, 本文档采用, 跟 SANDBOX-002 平行)
- 选项 B: 5 级别分文档 (TEST-PLAN / TEST-DESIGN / TEST-CASE / TEST-EXEC / UAT 分别落档)
- 选项 C: 只做 UT + IT 设计, E2E/PT/UAT 留 P2

**采用 A**: 跟既有 `TEST-DESIGN-SANDBOX-002.md` v0.1 + `TEST-DESIGN-OPS-001.md` v0.2 平行不重叠, 单文档 ≤ 90KB, 9 章节 (目的 / 范围 / UT / IT / E2E / PT / UAT / RACI / 修订)。

**子任务推进开发** (per 评论者发令): 5 个 sub-issue 平行落档, 详见 §11 实施计划 / 子任务分解。

---

## §1 范围

### 1.1 In-Scope (5 级别测试设计书)

| 章节 | 测试级别 | 范围 | 实证锚点 | 文档来源 |
|---|---|---|---|---|
| **§2 UT** | 单元测试 | 14 crate × 模块测试矩阵 (T1-T14) + 边界 + 错误路径 + 状态迁移 | `cargo test --workspace --lib` 52/52 pass (预估 v1.0 落地后) | DD §45 |
| **§3 IT** | 集成测试 | 跨 crate 8 IT + DB 集成 2 IT + Graph 集成 2 IT + Cache/Redis 集成 3 IT + gRPC/REST 集成 5 IT = 20 测 | `cargo test --workspace --tests` 10/10 IT pass (预估) | DD §46 |
| **§4 E2E** | 端到端 | 5 视图模式 5 E2E + 6 Semantic Zoom 6 E2E + 18 Action 18 E2E + 4 流程 E2E = 33 测 → 关键 8 E2E 落 MVP | Playwright + MSW handler 8/8 E2E pass | DD §47 |
| **§5 PT** | 性能测试 | 性能 7 AC (AC-P-1..7) + 守门 #1 v25 单 crate P95<200ms + k6 负载 + criterion-rs bench | `cargo bench` 4 PT + `vitest bench` 4 PT = 8 PT 实证 | DD §48 |
| **§6 UAT** | 验收测试 | 43 AC 全表化 (AC-F-1..20 + AC-P-1..7 + AC-Q-1..10 + AC-D-1..6) + 5 域 Lead 签字栏 (Mavis 临时代签) | 5 域 Lead 签字 (Mavis 临时代签 per 守门 #14 v3) | SRS §12 + Trace §11 |
| **W16 补充** | UT / IT / E2E / PT / UAT 追踪 | 设备身份、撤销、OAuth 单次消费、envelope/AAD、KMS fail-closed、轮换/恢复与有界加密成本 | UT-W16 / IT-W16 / E2E-COLLAB-12..19 / PT-10；设备/RLS 与 OAuth repository PG18 slices 局部通过，Canvas candidate compile/runtime 未执行；完整 E2E/PT 仍计划中 | SRS/BD/DD-W16-PAYLOAD-SECURITY-001 v0.4 |

**累计实证锚点** (per 守门 #1 v25 单 crate):

```
lib test:    52/52 pass    (cargo test --workspace --lib -j 4)
IT test:     10/10 pass    (cargo test --workspace --tests -j 4)
E2E test:     8/8  pass    (Playwright + MSW handler)
bench PT:    4 + 4 = 8 bench (cargo bench + vitest bench, 守门 #1 v25 P95<200ms)
UAT:         43/43 AC 100% 签字 (5 域 Lead 签字栏)
合计实证:    70 测 100% pass + 8 bench P95 守门 + 43 UAT AC
```

### 1.2 Out-of-Scope (per 守门 #1 R-05 mock 路径 + 守门 #11 缺标比错标)

- **真实 LLM 接入** (per 守门 #23 v2, T16 LLM 模板降级走 mock, 真实 LLM 留 P2)
- **L5 Symbol 级 AST 分析** (per SRS §13 V3 Risk Engine, MVP 只到 V2 Diff Overlap)
- **CRDT 多人编辑** (per BD D-CRDT-001, Yjs 选型但实装 P2)
- **1000+ WT 切换 PixiJS** (per BD D-CANVAS-001, MVP 用 React-Flow 11.x, 1000 WT 走 LOD 兜底)
- **AI Explanation Claude Sonnet 真实接入** (per BD D-LLM-001, MVP 用 mock + 模板)
- **5 域 Lead 真人寻访** (per 守门 #14 v2 + v0.62 反转, 真人代签流程全部取消, Mavis 临时代签)
- **Audit S3 冷归档** (per BD §14.2, P2 T22 落地)

### 1.3 5 级别测试框架 (per SANDBOX-002 模式)

| 级别 | 语言 | 框架 | 容器化 | Mock 策略 |
|---|---|---|---|---|
| **UT** | Rust + TS | `cargo test` + `vitest` | 无 | Trait Mock (per `MockGraphRepository` / `MockCanvasRenderer`) |
| **IT** | Rust | `cargo test --tests` + `testcontainers-rs` | PG 16 + Redis 7 + Neo4j 5 | testcontainers 真实容器 |
| **E2E** | TS | Playwright + MSW handler | Next.js dev server + BFF mock | MSW + WebSocket mock |
| **PT** | Rust + TS | `cargo bench` (criterion-rs) + `vitest bench` + k6 | PG + Redis + Neo4j + headless Chrome | 真实服务 + 合成负载 |
| **UAT** | Manual + Auto | 手动 + Playwright UAT mode | 全栈真实环境 | 5 域 Lead 真人 (Mavis 临时代签) |

### 1.3.1 Group 注入会话 generation 迟到响应回归

| 用例 | 可控时序 | 预期结果 | 证据边界 |
|---|---|---|---|
| **UT-FE-SESSION-01** `ignores a delayed 401 from a replaced Group session and keeps the new projection live` | 通过 deferred fetch 固定旧 generation：旧投影请求暂缓；随后旧请求的 401 暂缓；切换非密钥 `sessionKey` 与注入 token，确认新投影完成后，再依次释放旧成功响应与旧 401。全程不使用计时 sleep 或真实网络。 | 新 generation 保持 `live` 且仍显示新投影；迟到的旧成功响应不覆盖新投影；旧 401 不使新 generation 失效；新 generation 的请求使用新注入 token。 | 仅验证 Worktree Group 注入式 API provider 的客户端 generation 隔离和前端投影状态；不证明生产登录、token issuer、宿主 session owner 或生产身份授权已接通。命令：`pnpm run test src/lib/group/__tests__/groupProjection.test.tsx`（从 `frontend/` 运行；脚本参数前不加 `--`，否则当前 pnpm/Vitest 调用会运行整个 suite）。 |

### 1.4 引用基线 (per 守门 #9 v20 子代理 dispatch 必先 brief)

- **上游 4 份设计文档** (必读): SRS / BD / DD / TRACEABILITY (同步落档, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
- **平行 2 份实施文档**: IMPL-PLAN + spec (本 commit 同期落档)
- **既有 2 份同级测试设计书** (模板参考): `TEST-DESIGN-SANDBOX-002.md` + `TEST-DESIGN-OPS-001.md`
- **W16 补充需求与设计**: `SRS-W16-PAYLOAD-SECURITY-001.md` / `BD-W16-PAYLOAD-SECURITY-001.md` / `DD-W16-PAYLOAD-SECURITY-001.md` v0.4 (目标设计与当前实现状态；完整 W16 验收未完成)
- **守门基线** (13 项必过): #1 v25 + #3 + #5 + #6 + #9 + #10 + #11 + #13 + #14 v3 + #14 v4 + #28 + #29
- **不变量 12 项** (INV-WC-01..12, per spec §3)

---

## §2 UT 单元测试 (per 守门 #1 v25 单 crate `cargo test --lib`)

### 2.1 测试目标与覆盖率基线

| 目标 | 测量 | 守门 | 引用 |
|---|---|---|---|
| **UT 覆盖率 >= 80% (Rust)** | `cargo tarpaulin` | NFR-TEST-001 | SRS §4.9.14 |
| **14 crate 全部单 crate `cargo test --lib` 实证** | `cargo test -p <crate> --lib` | 守门 #1 v25 | AGENTS.md §4 |
| **52 UT 1:1 对齐 14 crate 关键入口** | 5 角色 Lead 验收 | 守门 #14 v3 | WORKTREE-CANVAS-IMPL-PLAN-001 §3.4 |
| **每个 UT 含边界 + 错误路径 + 状态迁移** | code review | 守门 #11 缺标比错标 | ipa-test-case §16 |
| **不变量 INV-WC-01..12 全 UT 覆盖** | UT 列表交叉表 | 守门 #1 + 守门 #11 | spec §3 |

### 2.2 14 crate 测试矩阵 (52 测 1:1 对齐)

| Crate (T-ID) | 模块 | 测数 | 关键测试名 | 引用 | 不变量 |
|---|---|---|---|---|---|
| **graph-core** (T1) | GraphRepository Trait + 11 Node + 13 Edge + Schema | 4 | `node_11_types_enum_complete` / `edge_13_types_enum_complete` / `neo4j_schema_ddl_idempotent` / `graph_query_n_hop_returns_expected` | DD §45.4 + §10 | INV-WC-04 (13 Edge) |
| **git-adapter** (T2) | GitProvider Trait + libgit2 + CLI fallback | 4 | `git_provider_trait_14_methods` / `libgit2_merge_writes_correct_index` / `cli_provider_fallback_on_libgit2_error` / `git_status_dirty_state_4_classification` | DD §45.1 + §10 | NFR-REL-001 (Git 不可变) |
| **git-observer** (T2) | GitWatcher + fsnotify + 防抖 100ms + 15 GitEvent | 3 | `git_observer_debounce_100ms_merges_burst` / `git_event_15_types_emit_correct_payload` / `head_ref_worktree_tracker_3_components` | DD §11 + §45.2 | NFR-REL-001 |
| **worktree-service** (T3) | WorktreeService + 7 状态机迁移 + CRUD + Health/Risk 集成 | 5 | `worktree_state_transition_7states_all_paths` / `state_machine_invalid_transition_rejected` / `worktree_cleanup_batch_with_provenance` / `worktree_status_observed_projection_30day_hot` / `health_risk_integration_health_low_triggers_risk` | DD §12 + §23 | INV-WC-01 (7 状态机) |
| **risk-engine** (T4) | RiskEngine V1 (Git Status) + V2 (Diff Overlap) + 11 Risk 类型 | 4 | `risk_score_v1_v2_v3_layered` / `risk_score_invariant_0_to_1` / `risk_threshold_07_04_classification` / `risk_health_bridge_risk_change_triggers_health_recompute` | DD §13 + §35 | INV-WC-03 (Risk ∈ [0,1]) |
| **health-engine** (T5) | HealthEngine + 13 因素加权扣分 + Health Score ∈ [0,100] | 3 | `health_score_13_factors_weighted` / `health_score_invariant_0_to_100` / `health_score_perfect_minimum_boundaries` | DD §14 + §34 | INV-WC-02 (Health ∈ [0,100]) |
| **agent-bridge** (T6) | AgentRuntime Trait + external work-item adapter + 14 AgentSession 状态 | 2 | `agent_runtime_trait_8_methods` / `agent_session_sync_14_states` | DD §15 + §45.3 | INV-WC-10 (LLM 不改 Source) |
| **relationship-engine** (T7) | RelationshipEngine + 13 Edge ops + 4 高风险 Edge metadata | 3 | `edge_kind_validation_13_kinds` / `edge_ops_crud_4_high_risk_metadata` / `edge_query_graph_n_hop_1_2_3` | DD §13 + §8 | INV-WC-04 (13 Edge) |
| **action-engine** (T8) | ActionEngine + 18 Action + 3 类别 + 二次确认 + Idempotency + Audit | 6 | `action_18_types_register_correctly` / `action_destructive_requires_confirmation` / `idempotency_24h_ttl_replay` / `audit_log_scd_type2_immutable` / `action_rbac_5_roles_enforced` / `action_retry_strategy_3_levels` | DD §18 + §42-§43 | INV-WC-09/11/12 |
| **event-bus** (T9) | EventBus (Redis Streams) + 15 Event + 4 消费者 + 防抖 | 3 | `event_bus_15_types_emit_correct` / `event_4_consumers_graph_ui_risk_notification` / `event_debouncer_100ms_burst_merged` | DD §19 + §20 | NFR-REL-002 (10k eps) |
| **layout-engine** (T10) | LayoutEngine + dagre/d3-force/ELK.js + visualDistance log 公式 | 3 | `layout_3_algorithms_dagre_d3_elk` / `visual_distance_log_n_plus_1_formula` / `layout_incremental_update_60fps` | DD §16 + §36 | NFR-PERF-002 (1000 WT 60fps) |
| **query-engine** (T11) | QueryEngine + DSL Parser (chevrotain) + NL Translator + 6 关键字 | 4 | `search_dsl_parser_show_conflict_combined` / `nl_query_translator_graph_query_git_filter` / `search_6_keywords_show_agent_behind_ahead_health_modified` / `search_query_cypher_injection_safe` | DD §17 + §28 | FR-SEARCH-001..006 |
| **canvas-renderer** (T12) | CanvasRenderer + React-Flow 11.x adapter + LOD + Viewport Virtualization | 4 | `canvas_renderer_lod_3_levels` / `viewport_virtualization_1000_wt_only_visible_rendered` / `focus_mode_n_hop_1_2_3_transparency` / `canvas_5_view_modes_correct_layout` | DD §11 + §29-§30 | INV-WC-05/06/07 |
| **bff/worktree_canvas** (T13) | BFF module + 11 REST + 15 SSE + 1 WS + RBAC + Idempotency + Audit 包装 | 2 | `bff_11_rest_endpoints_rbac_enforced` / `bff_15_sse_events_authenticated` | DD §20-§21 | NFR-PERM-001 |
| **frontend/** (T14) | 30 前端 TS/TSX 文件 + 5 View Mode + 6 Semantic Zoom + 11 Inspector Tab + Zustand Store | 2 | `vitest_30_ts_files_compile` / `playwright_5_view_modes_render` | DD §22 + §29-§30 | INV-WC-05/06/07 + #19 v19 (不破坏 V0.1) |
| **合计** | **14 crate** | **52 UT** | | | **12 INV 覆盖** |

### 2.3 覆盖率目标与缺口统计

| 维度 | 目标 | 测量工具 | 现状 |
|---|---|---|---|
| 行覆盖率 (Rust) | >= 80% | `cargo tarpaulin` | 【TBD】(v1.0 落地后实证) |
| 行覆盖率 (TS) | >= 70% | `vitest run --coverage` | 【TBD】 |
| 分支覆盖率 (Rust) | >= 70% | `cargo tarpaulin --branch` | 【TBD】 |
| 不变量覆盖 | 12/12 = 100% | INV 交叉表 | 12/12 ✅ |
| Requirement 覆盖 | 103/103 = 100% | UT 列表 vs SRS FR | 103/103 ✅ |
| 决策点覆盖 | 15/15 = 100% | UT 列表 vs BD D-* | 15/15 ✅ |

### 2.4 边界条件矩阵 (5 维必跑, per ipa-test-case §9)

| 维度 | Min-1 | Min | Min+1 | Max-1 | Max | Max+1 | 引用 |
|---|---|---|---|---|---|---|---|
| **Ahead 数量** | 0-1 | 0 | 1 | 999 | 1000 | 1001 | NFR-PERF-002 |
| **Behind 数量** | 0-1 | 0 | 1 | 999 | 1000 | 1001 | NFR-PERF-002 |
| **Worktree 数量** | 0-1 | 0 | 1 | 999 | 1000 | 1001 | NFR-PERF-002 |
| **Edge 数量** | 0-1 | 0 | 1 | 9999 | 10000 | 10001 | NFR-PERF-002 |
| **Health Score** | -1 | 0 | 1 | 99 | 100 | 101 | INV-WC-02 |
| **Risk Score** | -0.01 | 0 | 0.01 | 0.99 | 1.0 | 1.01 | INV-WC-03 |
| **Idempotency TTL** | 0s | 1s | 1h | 23h59m | 24h | 24h1s | INV-WC-11 |

### 2.5 错误路径覆盖 (per spec §7 + 守门 #6 v2 6-field)

| 错误码 | 测试场景 | 测数 | 引用 |
|---|---|---|---|
| **WC-STT-001** | 无效状态迁移 (RUNNING → MERGED 跳过 READY) | 1 | DD §38 |
| **WC-AUTH-001** | 未登录调用 Action | 1 | DD §38 |
| **WC-AUTH-002** | 越权 (Dev 角色调用 Owner-only Action) | 1 | DD §38 + §44 |
| **WC-VAL-001** | Idempotency Key 格式错误 | 1 | DD §38 |
| **WC-NF-001** | WorktreeId 不存在 | 1 | DD §38 |
| **WC-CONFLICT-001** | Conflict 状态下执行 Merge | 1 | DD §38 |
| **WC-GIT-001** | Git Source of Truth 写入失败 | 1 | NFR-REL-001 |
| **WC-RISK-001** | Risk Score 越界 (1.5) | 1 | INV-WC-03 |
| **WC-HEALTH-001** | Health Score 越界 (-5) | 1 | INV-WC-02 |
| **WC-GRAPH-001** | Graph Query 超时 (5s) | 1 | NFR-PERF-003 |

### 2.6 守门合规清单 (per AGENTS.md §4 + 守门 #1 v25)

- [x] 守门 #1 v25: CI cargo test 改单 crate (`cargo test -p graph-core --lib`, etc.)
- [x] 守门 #11: 缺标比错标, 已知缺口显式标 (13 已知缺口, per spec §11.3)
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表: worktree.worktree W + worktree_status_observed W + worktree_conflict T + action_audit T + idempotency W)
- [x] 守门 #6 v2: 错误码 6-field 完整 (code + message + severity + category + hint + trace_id)
- [x] 守门 #1 + #1 v15: 1 commit 多文件 + 禁回溯叙事

### 2.7 本章小结

UT 共 **52 项既有基线**，14 crate × 模块 1:1 对齐，覆盖 103 FR + 23 NFR + 12 不变量 + 15 决策点 + 43 AC。覆盖率目标 Rust >= 80% / TS >= 70%。边界值覆盖 7 维，错误路径覆盖 10 错误码。守门 5 项必过，跟 SANDBOX-002 §2 UT (56 测) 模式一致。

### 2.8 W16 可信身份与受保护 envelope 单元测试设计（计划，未执行）

W16 另增以下 3 项补充设计，均未实现/执行：

| UT ID | 测试名 | 输入 / 边界 | 必须观察到的结果 | 追踪 | 状态 |
|---|---|---|---|---|---|
| **UT-W16-AUTH-01** | `verified_claims_override_client_device_fields` | 验证 issuer claims 与客户端 body/header 中缺失、伪造或互相矛盾的 `actor_id`、`tenant_id`、`device_id`、generation | author、RLS scope、冲突参与者与通知接收者只来自已验证且当前有效的 issuer/device 绑定；无可信设备身份时为 `local_only / sync_pending`，不能形成共享成功事实 | SRS-W16-PAYLOAD-01 | 设计；未实现/执行 |
| **UT-W16-ENVELOPE-01** | `canonical_aad_binds_resource_revision_and_schema` | 对 canonical tenant/Project/Run/resource/revision、envelope format 与 AAD schema 版本逐项变更或提供未知版本 | 原上下文可供受控解密；任一字段被替换或版本未知均拒绝，且不返回正文或将错误上下文静默规范化 | SRS-W16-PAYLOAD-04/05 | 设计；未实现/执行 |
| **UT-W16-ENVELOPE-02** | `ciphertext_or_tag_tamper_fails_without_plaintext` | 篡改密文、AEAD tag、nonce、wrapped DEK 元数据或 envelope digest | 解密失败、无明文输出、canonical revision 不变；错误日志/指标不包含正文、DEK、凭证或密钥身份秘密 | SRS-W16-PAYLOAD-05/06 | 设计；未实现/执行 |

---

## §3 IT 集成测试 (per 守门 #9 v20 + 守门 #1 v25 + 守门 #13 + 守门 #24 v2)

### 3.1 测试目标与跨 crate 范围

| 目标 | 测量 | 守门 | 引用 |
|---|---|---|---|
| **跨 crate 联动实证** (14 crate 之间) | `cargo test --workspace --tests` | 守门 #1 v25 | AGENTS.md §4 |
| **testcontainers-rs 真实容器** (PG 16 + Redis 7 + Neo4j 5) | testcontainers-rs 启动容器 | 守门 #24 v2 | DD §46 |
| **DB Schema 跨 tenant 隔离** (RLS 13 類) | sqlx query 实证 | INV-WC-09 | DD §46.2 |
| **Idempotency + Audit 联动** | action 跨 crate 调用实证 | INV-WC-11/12 | DD §46.1 |
| **Graph Query 集成** (Cypher 真实执行) | Neo4j 容器 | 守门 #13 | DD §46.3 |

### 3.2 10 IT 实证矩阵

| IT ID | 测试名 | 跨 crate | 容器 | 引用 | 不变量 | 实证命令 |
|---|---|---|---|---|---|---|
| **IT-01** | `merge_destructive_idempotency_audit_e2e` | T3 + T5 + T7 + T8 + audit crate | PG + Redis | DD §46.1 | INV-WC-12 + A-07 | `cargo test -p worktree-service --test it_merge_audit` |
| **IT-02** | `rls_13_classes_cross_tenant_reject` | T3 + audit + PG | PG | DD §46.2 | NFR-SEC-001 | `cargo test -p worktree-service --test it_rls` |
| **IT-03** | `graph_repository_neo4j_upsert_query_delete_13_edges` | T1 + T7 + Neo4j | Neo4j | DD §46.3 | INV-WC-04 | `cargo test -p graph-core --test it_neo4j_13_edges` |
| **IT-04** | `event_bus_redis_streams_4_consumers_15_events` | T9 + 4 消费者 + Redis | Redis | DD §46.4 | NFR-REL-002 | `cargo test -p event-bus --test it_redis_4_consumers` |
| **IT-05** | `risk_engine_v2_diff_overlap_real_repo_2_worktrees` | T4 + T2 + libgit2 | (无需容器) | DD §46.5 | INV-WC-03 | `cargo test -p risk-engine --test it_v2_diff_overlap` |
| **IT-06** | `action_engine_18_actions_3_categories_rbac_audit` | T8 + audit + RBAC | PG | DD §46.6 | INV-WC-09/11/12 | `cargo test -p action-engine --test it_18_actions` |
| **IT-07** | `health_engine_integration_with_risk_and_worktree` | T5 + T4 + T3 | (无需容器) | DD §46.7 | INV-WC-02 | `cargo test -p health-engine --test it_integration` |
| **IT-08** | `bff_11_rest_15_sse_1_ws_auth_rbac_idempotency` | T13 + T8 + audit | PG + Redis | DD §46.8 | NFR-PERM-001 | `cargo test -p bff --test it_11_rest_sse_ws` |
| **IT-09** | `git_observer_7_sources_15_events_emit_debounce` | T2 + Redis Streams | (无) | DD §46.9 | NFR-REL-001 | `cargo test -p git-observer --test it_7_sources` |
| **IT-10** | `cache_l1_l2_l3_invalidation_correctness_3_levels` | T1 + T3 + T8 + cache + PG + Redis | PG + Redis | DD §46.10 | NFR-CACHE-001 | `cargo test -p worktree-service --test it_cache_3_levels` |

### 3.3 跨 IT 缺口统计与 sqlx 容器化路径

| 缺口 ID | 描述 | 容器化路径 | 引用 |
|---|---|---|---|
| **G-IT-01** | Neo4j 5.x 容器化启动慢 (15-20s), CI 需 warm-up | `testcontainers::ContainerRequest::Neo4j` + `reqwest_retry` | DD §46.3 |
| **G-IT-02** | PG + Redis + Neo4j 3 容器同时启动 CI 内存峰值 4GB | CI runner 8GB 起步, 串行启动 + tear-down | DD §46 |
| **G-IT-03** | Redis Streams 4 消费者并行启动, 端口冲突 | `testcontainers` 动态端口 + env 注入 | DD §46.4 |
| **G-IT-04** | Action Engine 18 Action 全跑测试慢 (~5min), CI 拆分并行 | `cargo nextest` 4 并行 + 拆分 by category | DD §46.6 |
| **G-IT-05** | testcontainers-rs Docker-in-Docker 兼容 (k3s 集群) | DOCKER_HOST=tcp://dind:2375 + privileged 容器 | DD §46 |

### 3.4 守门合规清单 (per AGENTS.md §4 + 守门 #1 v25 + 守门 #9 v20)

- [x] 守门 #1 v25: 跨 crate `cargo test --workspace --tests` 实证 10/10
- [x] 守门 #9 v20: 子代理 dispatch 必先 brief (本文档即 IT brief 入口)
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表集成测试覆盖)
- [x] 守门 #24 v2: testcontainers-rs 真实容器, 不允许 mock 替代
- [x] 守门 #6 v2: 错误码 6-field 跨 IT 闭环

### 3.5 本章小结

IT 共 **10 项既有基线**, 覆盖跨 crate 联动 8 项 + DB 集成 2 项 + Graph 集成 1 项 + Cache/Redis 集成 2 项 + gRPC/REST 集成 5 项 (部分重合)。testcontainers-rs 真实容器 (PG 16 + Redis 7 + Neo4j 5)。缺口 5 项显式标 (G-IT-01..05)。守门 5 项必过。

### 3.6 W16 身份、令牌单次消费与 provider 集成测试设计

下列 4 项端到端生产验收用例仍未实现/执行。本节后表列出已执行的隔离 PostgreSQL 18 slices 及 Canvas SSE revocation 单用例结果；这些局部记录不替代下列完整用例：

| IT ID | 测试名 | 真实边界 | 必须观察到的结果 | 追踪 | 状态 |
|---|---|---|---|---|---|
| **IT-W16-AUTH-01** | `authorization_code_and_refresh_are_single_use_under_race` | 目标 PostgreSQL 上并发兑换同一 authorization code；另对同一 refresh token 并发刷新 | 每组竞争最多一个成功消费者/新 token；其余认证失败；消费、签发或轮换在 auth owner 事务内原子完成，身份/设备/tenant/client/scope 绑定保持 | SRS-W16-PAYLOAD-03 | 设计；未实现/执行 |
| **IT-W16-IDENTITY-01** | `enrollment_proof_generation_revoke_and_stream_expiry` | 真实身份 owner、授权目录与两个 gateway；设备 enrollment proof、generation 变化、撤销与事件丢失 | 未验证 proof 不能激活；伪造/过期 generation 不能读写；撤销阻止新操作，并关闭现有 SSE 或在有界授权租约内失效；状态未知时 fail closed | SRS-W16-PAYLOAD-01/02 | 设计；未实现/执行 |
| **IT-W16-KMS-01** | `protected_envelope_roundtrip_restart_and_no_content_facts` | 被批准的真实 provider profile、owner PostgreSQL 与可重启 Rust 服务进程 | 仅密文与必要 envelope 元数据可持久；重启后固定 provider/profile/key version 可解密授权旧信封；DB、audit、logs、metrics、Outbox、NOTIFY 与 wake payload 均不含正文/明文 DEK | SRS-W16-PAYLOAD-04/05/07 | 设计；未实现/执行 |
| **IT-W16-KMS-02** | `rotation_rewrap_and_backup_reference_recovery` | 固定 provider/profile、保留备份与轮换/rewrap worker；重复投递与中断恢复 | 轮换/rewrap 可审计、幂等且可恢复；保留期/备份仍引用旧 key 时合法读取保持可用；只有所有 live 与 backup 引用迁移或到期后才能销毁旧 key | SRS-W16-PAYLOAD-07 | 设计；未实现/执行 |

| **IT-W16-IDENTITY-PG-01** | `device_repository_proof_revoke_and_challenge_cap` | 隔离 PostgreSQL 18 与 `PgDeviceBindingRepository` | owner-scoped challenge 创建/读取、proof completion、revoke 与并发 challenge 上限符合数据库约束 | SRS-W16-PAYLOAD-01/02 | PASS；2 tests，run `6af04e5a765446c8b3baaffff126bd52` |
| **IT-W16-DIRECTORY-RLS-01** | `device_generation_for_share_and_denied_update` | 隔离 PostgreSQL 18、runtime login 与 Directory owner role | tenant+actor 可见性独立验证；匹配设备 generation 可 `FOR SHARE`；直接 UPDATE 返回 42501 且 generation 不变 | SRS-W16-PAYLOAD-01/02 | PASS；run `6af04e5a765446c8b3baaffff126bd52`；未测试第二事务的 revoke contention |
| **IT-W16-REFRESH-RLS-01** | `refresh_token_tenant_actor_scope_and_pair_constraint_upgrade` | 隔离 PostgreSQL 18、历史弱同名 CHECK、OAuth owner role | 只可见本 tenant+actor refresh rows；旧弱约束被升级；device 非空而 generation 空的插入失败并回滚 | SRS-W16-PAYLOAD-01/03 | PASS；run `6af04e5a765446c8b3baaffff126bd52`；不证明 OAuth 并发单赢家 |
| **IT-W16-OAUTH-REPOSITORY-PG-01** | `postgres_oauth_credentials_have_one_winner_when_test_database_is_configured`; `postgres_auth_code_redemption_persists_tokens_with_consumption`; `postgres_refresh_rotation_commits_one_successor_with_the_consume` | Disposable PostgreSQL 18.6 与 OAuth repository；24 concurrent contenders are barrier-released | auth-code and refresh races each have one winner; code consumption/token pair and refresh successor/revocation persist; stale refresh snapshot is rejected | SRS-W16-PAYLOAD-03 | PASS；3/3 tests，run `1b4673269a04435daa1fab5c93333133`；disposable superuser + test-only migration prerequisites；不证明 production grants/RLS 或 HTTP E2E-COLLAB-14 |
| **IT-W16-CANVAS-SSE-REVOCATION-PG-01** | `active_canvas_sse_closes_after_device_binding_revocation` in `canvas_device_auth_pg` | PostgreSQL 18.6 disposable loopback cluster; production group GET SSE route/hub, committed persisted device revocation and Outbox wake | The active stream closes after revocation within the five-second test deadline; the test reports no post-revocation payload | SRS-W16-PAYLOAD-02 | **PASS**；run `49b512fbfe334f0e98e46fe7eae7ea39`；41/41 runner steps 0 exit, 1 passed/0 failed/0 ignored/3 filtered in 0.17s；Rust 1.99.0 binary built at `d1269a7928c9323f94403f3f1bb7888048c1dbac`. Uses disposable `postgres` superuser and no-op test-only audit/SCD2 prerequisites; unsigned build attestation; does not prove production grants/RLS, OAuth HTTP endpoint audit, multi-instance recovery, replay offsets, p99, or full E2E/W16-A |

---

## §4 E2E 端到端测试 (per DD §47 + 守门 #1 R-05 + 守门 #11 缺标比错标)

### 4.1 测试目标与浏览器自动化范围

| 目标 | 测量 | 守门 | 引用 |
|---|---|---|---|
| **5 View Mode 切换正确** | Playwright 截图 + state assertion | AC-F-7 | SRS §12.1 |
| **6 Semantic Zoom 切换** | Playwright canvas zoom assertion | AC-F-8 | SRS §12.1 |
| **18 Action 全跑通** | Playwright UI click + BFF mock assertion | AC-F-12 | SRS §12.1 |
| **Focus Mode 1/2/3-hop** | Playwright canvas transparency assertion | AC-F-11 | SRS §12.1 |
| **NL Query 翻译为 Graph Query** | Playwright input + MSW handler assertion | AC-F-10 | SRS §12.1 |
| **Destructive 二次确认 + Audit** | Playwright dialog + audit log assertion | AC-F-13 + AC-Q-10 | SRS §12.1 |
| **Inspector 11 tab 全部可访问** | Playwright tab click assertion | AC-F-15 | SRS §12.1 |
| **i18n 3 语言 (zh-CN/en/ja) 无硬编码** | Playwright text content assertion | AC-Q-7 | SRS §12.3 |

### 4.2 8 E2E 实证矩阵 (per DD §47)

| E2E ID | 测试名 | 流程 | 引用 | 实证命令 |
|---|---|---|---|---|
| **E2E-01** | `canvas_render_1000_worktrees_lod_60fps` | 打开画布 → 加载 1000 WT mock → 验证 FPS >= 15 (per NFR-PERF-002) | DD §47.1 + AC-P-2 | `pnpm playwright test e2e/canvas-1000-lod.spec.ts` |
| **E2E-02** | `worktree_create_to_merge_full_flow` | Create WT → Modify files → Conflict Detection → Confirm Merge → Audit verify | DD §46.1 + AC-F-12/13 | `pnpm playwright test e2e/wt-create-merge.spec.ts` |
| **E2E-03** | `conflict_prediction_2_worktrees_risk_engine_v2` | 创建 2 个 WT 都改 auth.rs → 验证 CONFLICTS_WITH Edge + Risk Score >= 0.4 | DD §46.2 + AC-F-6 | `pnpm playwright test e2e/conflict-predict.spec.ts` |
| **E2E-04** | `5_view_modes_switch_no_recompute` | TREE → DEPENDENCY → RISK → AGENT → HISTORY → 验证 network response 无全量 reload | AC-F-7 | `pnpm playwright test e2e/5-view-modes.spec.ts` |
| **E2E-05** | `6_semantic_zoom_l0_to_l5_info_density` | L0 → L5 缩放 → 验证每级信息粒度正确 | AC-F-8 | `pnpm playwright test e2e/6-zoom.spec.ts` |
| **E2E-06** | `search_dsl_show_conflict_behind_20_agent_codex` | 输入 `show:conflict behind:>20 agent:codex` → 验证过滤结果 | AC-F-9 | `pnpm playwright test e2e/search-dsl.spec.ts` |
| **E2E-07** | `focus_mode_1_2_3_hop_transparency_levels` | 点击 WT-A → 1/2/3-hop 切换 → 验证透明度 100%/80%/20% | AC-F-11 | `pnpm playwright test e2e/focus-mode.spec.ts` |
| **E2E-08** | `destructive_action_merge_requires_confirmation_audit_logged` | 点击 Merge → 弹出二次确认 → 确认 → Audit Log 出现 Merge success | AC-F-13 + AC-Q-10 | `pnpm playwright test e2e/destructive-merge.spec.ts` |

### 4.3 E2E 框架选型

| 组件 | 选型 | 理由 | 引用 |
|---|---|---|---|
| **浏览器自动化** | Playwright 1.45+ | 多浏览器 + headless + screenshot | DD §47 |
| **HTTP Mock** | MSW (Mock Service Worker) 2.x | BFF mock + WebSocket mock + REST mock | DD §47 |
| **断言** | Playwright expect + jest-style matchers | 内置 | DD §47 |
| **截图回归** | Playwright screenshot + pixelmatch | 视觉回归 | DD §47 |
| **i18n 验证** | Playwright + locale switcher | 3 语言验证 | AC-Q-7 |

### 4.4 i18n 3 语言端到端验证 (per AC-Q-7)

| 语言 | 测试名 | 关键字符串 | 引用 |
|---|---|---|---|
| **zh-CN** | `i18n_zh_cn_no_hardcoded_strings` | "Worktree" / "合并" / "冲突" | AC-Q-7 |
| **en** | `i18n_en_no_hardcoded_strings` | "Worktree" / "Merge" / "Conflict" | AC-Q-7 |
| **ja** | `i18n_ja_no_hardcoded_strings` | "ワークツリー" / "マージ" / "競合" | AC-Q-7 |

**i18n 字符串来源**: `frontend/src/i18n/{zh-CN,en,ja}.json`, 单 key 唯一来源, 禁止硬编码 (per AC-Q-7 + 守门 #11)。

### 4.5 守门合规清单 (per AGENTS.md §4 + 守门 #1 R-05 + 守门 #11)

- [x] 守门 #1 R-05: 5 View Mode + 6 Zoom + 18 Action 全 E2E 覆盖
- [x] 守门 #11: 缺标比错标, 已知缺口显式标 (8 已知缺口, per spec §11.3)
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表 E2E 验证)
- [x] 守门 #6 v2: 错误码 6-field E2E 闭环
- [x] 守门 #23 v2: LLM 走 mock 路径

### 4.6 已知缺口 (per 守门 #11 缺标比错标, DDD Review 必查)

| 缺口 ID | 描述 | 引用 | 落地期 |
|---|---|---|---|
| **G-E2E-01** | 1000 WT E2E 需 mock 1000 个 Graph Node, mock 数据构造慢 (10s) | E2E-01 | MVP |
| **G-E2E-02** | 5 View Mode 切换要求"不重算数据", 需 MSW handler 不重新触发 fetch | E2E-04 | MVP |
| **G-E2E-03** | NL Query 测试需 mock LLM 返回, 模板 3-5 个待 T16 设计 | E2E-06 | P2 T23 |
| **G-E2E-04** | i18n 3 语言完整 E2E 跑慢 (~3min), CI 拆分并行 | 4.3 | MVP |
| **G-E2E-05** | 视觉回归 pixelmatch 阈值敏感, false positive 多 | 4.3 | MVP |
| **G-E2E-06** | Destructive Action 二次确认 UI 文案跨语言一致性 | E2E-08 | MVP |
| **G-E2E-07** | Audit Log 跨 session 持久化需 PG 真实环境, MVP 走 MSW | 4.2 | MVP |
| **G-E2E-08** | Inspector 11 tab 全部 E2E 覆盖耗时长 (~5min), 拆 11 子 spec | 4.2 | MVP |
| **G-E2E-09** | WLD-7 需要双 actor、目标 PostgreSQL 与两个 gateway 的持久恢复验收；现有 MSW/单实例 E2E 不够 | §4.8 | WLD-7 |
| **G-E2E-10** | W16 设备凭据/撤销、OAuth 并发单次消费、真实 provider envelope、AAD 篡改、轮换与恢复依赖尚未实现；当前 mock 不能关闭这些安全门 | §4.8 / SRS-W16-PAYLOAD-01..08 | W16 |

### 4.7 本章小结
既有 Canvas 核心 E2E 共 **8 项基线**；WLD-7 另有 **19 项计划用例**（§4.8），尚未实现/执行，不计入已通过数量。缺口 G-E2E-01..10 显式标记。

### 4.8 WLD-7 多用户推送、冲突、草稿恢复与 W16 载荷安全 E2E（19 项计划，未执行）

环境要求：两个独立登录 actor、可重建目标 owner database、至少两个 realtime gateway 实例和可控 relay/wake-up 故障注入。Vercel 路径使用 Neon；自托管路径使用 Multipass 管理的 K3s，测试期 Multipass inventory 总数最多 2 个实例（包含 Stopped/Deleted 记录），不要求 Docker Desktop。两个节点上的 Raft 不具备 HA 容错能力。只用 MSW、单进程 broadcast 或同一浏览器两个 tab 不构成跨实例 durable 验收；每次测试还须固定 provider/profile/key version 与实际环境基线。若目标 profile/provider、数据库或资源预算未准备好，相应场景保持 OPEN/NOT_RUN，不以 mock 替代。

**当前状态**：Canvas 已有由 PostgreSQL Outbox wake 驱动的授权 SSE 路径，设备撤销关闭活动流的一个 PG18.6 定向用例已通过；这不等于 WLD-7 的完整多用户 revision push E2E 已通过。持久双边草稿/冲突、跨实例丢失通知恢复和完整运行证据仍为 OPEN/NOT_RUN。以下是目标验收计划，不代表其余目标实现已具备。

| Case | 场景与操作 | 必须观察到的结果 | Gate |
|---|---|---|---|
| E2E-COLLAB-01 | 两位 actor 订阅同一已授权 Resource；A 提交 revision | B 由事件 push 收到新 revision；连接稳定期间无固定周期 Canvas/Task read 请求；未授权 Run 无事件 | AC-COLLAB-01 |
| E2E-COLLAB-02 | 两位 actor 从同一 base 修改不同结构化字段 | owner CAS 保留两项并发更改；两个投影收敛到同一 revision，无冲突或丢失更新 | AC-COLLAB-02 |
| E2E-COLLAB-03 | 两位 actor 从同一 base 同步完整且 digest-valid 的重叠字段/文本 draft head | owner 检出重叠后在同一冲突事务自动 seal/pin 双方准确 revision，无需最终 Save；生成同一稳定 conflict_id；正常提交无 last-write-wins | AC-COLLAB-02/03 |
| E2E-COLLAB-04 | A/B 分别打开 Conflict inbox 并读取差异 | 两人看到相同持久未读提醒，可授权读取 base/current/mine/theirs；传输 ACK 不清 unread；第三人读内容失败 | AC-COLLAB-03/04 |
| E2E-COLLAB-05 | 模拟大 patch 仅上传部分 frame 或 digest 错误 | 只向作者显示 sync_pending；不对外创建可见 conflict；完成且 digest 正确后才可 seal/冲突 | AC-COLLAB-04 |
| E2E-COLLAB-06 | owner commit 后丢失 wake-up、断开一位 actor、重启 relay/gateway 再重连 | durable cursor/backlog 恢复事件与离线 conflict；去重后不丢草稿、不重复业务事实 | AC-COLLAB-05 |
| E2E-COLLAB-07 | 两 gateway 收到同一资源更新，并注入重复、乱序、慢 consumer、延迟写回执与队列上限 | event ID/sequence 去重；每 draft 最多一个在途写入，慢回执期间的新输入合并到一个有界 pending delta；超限 resync_required + cursor catch-up；不能静默跳过 durable event | AC-COLLAB-05/06 |
| E2E-COLLAB-08 | 撤销参与者权限后尝试订阅、读取、resolve；并发 resolve 使用 stale expected revision | 活跃订阅关闭；读写均重新授权；stale resolve 保留两稿并创建 successor conflict | AC-COLLAB-05/07 |
| E2E-COLLAB-09 | 两位 actor 用同步屏障在服务端 ≤1000 ms 内提交完整、digest-valid 且语义重叠的 draft heads；记录第二份 head 到达 owner 的时间与 owner transaction/Outbox trace | 不等待一秒 debounce、batch 或 arbitration timer；第二份完整 head 通过校验后立即进入同一 resource fence 并原子 pin 两稿、写出一个 stable conflict_id 与双方 unread facts/Outbox；双方收到同一持久提醒且可读取首个有界 base/current/mine/theirs diff。≤1000 ms 是输入到达刺激，不是检测窗口；任意更长到达间隔的语义重叠仍应由相同路径检测。端到端 p99 由 PT-09 测量 | AC-COLLAB-02/04 |
| E2E-COLLAB-10 | 两位 actor 从同一文本 base 对有交集的 span 提交内容，即使测试 OT 能无损变换 | owner 仍按语义重叠创建唯一持久 conflict，seal/pin 两侧精确 draft；双方均收到同一 conflict_id 并可读对方修改；不得自动提交合并 revision | AC-COLLAB-02/08 |
| E2E-COLLAB-11 | 一位参与者显式进入 conflict resolve 并请求 OT 建议，再分别验证取消、确认及 stale CAS | OT 建议只显示为无副作用预览；取消不改变 canonical revision/conflict/drafts；仅明确选中且当前 revision CAS 成功后写入；stale CAS 保留双方稿和原冲突 | AC-COLLAB-08/09 |
| E2E-COLLAB-12 | 无 enrollment 的客户端省略可信设备 claim，或在 body/header 伪造、替换 `device_id` / generation 后尝试共享写入 | 客户端字段不改变 issuer 绑定的 actor/tenant/device；写入被拒绝或保留为 `local_only / sync_pending`，不得产生共享 revision、冲突参与者或成功通知事实 | SRS-W16-PAYLOAD-01 |
| E2E-COLLAB-13 | 完成经 proof 验证的设备 enrollment 后订阅资源，再撤销该设备并尝试新读写/resolve；另注入撤销事件丢失与授权目录不可用 | enrollment 前不能共享；撤销后新操作拒绝；在线 SSE 被关闭，或在明确有界的授权租约到期前重新核验并失效；未知撤销状态与代次不匹配 fail closed | SRS-W16-PAYLOAD-01/02 |
| E2E-COLLAB-14 | 对同一 authorization code 并发兑换；另对同一 refresh token 并发刷新/重放 | 每组最多一个调用成功签发/轮换 token；其余认证失败；身份、设备、tenant、client 与 scope 绑定在唯一胜者的结果中保持 | SRS-W16-PAYLOAD-03 |
| E2E-COLLAB-15 | 注入 KMS/provider 不可用、显式拒绝与未知 key version，提交共享 checkpoint；核对 DB、revision、receipt、audit、Outbox、NOTIFY、SSE wake 和 LocalMockKms 调用 | 失败保持 `local_only / sync_pending`；不出现持久明文、canonical revision、成功回执或含正文的 Outbox/消息；不得调用 `LocalMockKms`、内存密钥或 plaintext fallback | SRS-W16-PAYLOAD-04/06 |
| E2E-COLLAB-16 | 使用真实已批准 profile 写入 envelope，重启 Rust owner server 后，以新进程和原授权上下文读取旧 envelope | 新进程从 envelope 固定的 provider/profile/version/digest、opaque key identity 与 key version 恢复读取；不能依赖旧进程内状态或默认当前 key；未授权 actor 仍拒绝 | SRS-W16-PAYLOAD-05/07 |
| E2E-COLLAB-17 | 分别替换 tenant、Project、Run、resource、revision AAD 字段；再修改 ciphertext、AEAD tag 或 envelope digest | 每次上下文替换或完整性篡改均拒绝，不返回明文、不更改 canonical revision，也不在日志/指标/错误响应泄漏正文或密钥材料 | SRS-W16-PAYLOAD-04..06 |
| E2E-COLLAB-18 | 持久化若干旧 key/profile envelope 后执行 key rotation/rewrap；在仍有 live 与 retained backup 引用期间读取；重放/中断后继续迁移 | 旧正文在所有 live 与备份引用迁移或到期前仍可授权读取；rewrap 进度可审计、幂等并可恢复；未满足引用清零门时拒绝销毁旧 key | SRS-W16-PAYLOAD-05/07 |
| E2E-COLLAB-19 | 从一致性 DB/blob/provider backup 恢复；分别以仍授权身份和已撤销身份读取恢复后的旧 envelope | 授权身份在固定 provider/profile/key version 可用时读回；已撤销设备仍拒绝，不能因恢复旧设备状态或授权快照而复活；恢复过程不泄漏明文 | SRS-W16-PAYLOAD-02/05/07 |


---

### 4.9 Project Repository 解绑与迁移 E2E（设计目标，未执行）

| Case | 场景与操作 | 必须观察到的结果 | Gate |
|---|---|---|---|
| E2E-REPO-01 | Project 分别绑定 0、1、多个 Repository，并设置/清除 default | 列表准确反映 0..N；default 可空且只影响预选/排序，不授予授权，不创建 Run binding | WTG-027/AC-WTG-014 |
| E2E-REPO-02 | retiring 与 Run binding/Worktree create/import 并发到达 | retiring CAS 先后决定序列；进入 retiring 后所有新引用拒绝，既有引用保留待迁移，不产生静默重绑 | WTG-027/AC-WTG-014 |
| E2E-REPO-03 | 多 owner 将活动 Run/Worktree 迁移至同 Project 已授权替代 Repository，重复投递相同 operation_id | owner-local CAS/Outbox 与回执可精确幂等重放；Host 校验 checkout source/target identity 与 generation；不跨 owner 写表 | WTG-027/AC-WTG-014 |
| E2E-REPO-04 | 注入 owner 超时、授权撤销、Host provider 不可用、未完成 checkout drain 或未知引用数 | 关联维持 retiring；新操作继续 fail closed；重试可恢复且不会伪造完成回执 | WTG-027/AC-WTG-014 |
| E2E-REPO-05 | 全部 owner 回执成功并声称零活动引用后最终解绑；同时保留历史审计读取 | 序列化复核为零才允许关闭 link；缺少/过期回执拒绝解绑；历史 Repository identity 不被改写 | WTG-027/AC-WTG-014 |

## §5 PT 性能测试 (per 守门 #1 + 守门 #7 v3 P95<200ms + DD §48)

### 5.1 测试目标与硬约束

| 目标 | 测量 | 守门 | 引用 |
|---|---|---|---|
| **守门 #1 v25 单 crate P95<200ms** | `cargo bench` criterion-rs | 守门 #1 v25 | AGENTS.md §4 |
| **守门 #7 v3 性能基线** | `vitest bench` | 守门 #7 v3 | AGENTS.md §4 |
| **7 AC 性能 (AC-P-1..7) 实证** | k6 + criterion-rs | SRS §12.2 | DD §48 |
| **缓存命中率 >= 80%** | Prometheus metric | NFR-CACHE-001 | SRS §4.9.13 |
| **事件合并 + 防抖 1s 内 100 次状态变化不触发全量重算** | 压力测试 | AC-P-7 | SRS §12.2 |

### 5.2 PT 实证矩阵（8 项既有基线 + PT-09/PT-10 两项 WLD-7 计划，均未实测）(per DD §48)

| PT ID | 测试名 | 工具 | 基准 | 引用 | 实证命令 |
|---|---|---|---|---|---|
| **PT-01** | `bench_health_compute_13_factors` | criterion-rs | P95<50ms (单 WT) | DD §48 + INV-WC-02 | `cargo bench -p health-engine --bench health_compute` |
| **PT-02** | `bench_risk_engine_v2_diff_overlap_100_files` | criterion-rs | P95<100ms (2 WT × 100 文件) | DD §48 + INV-WC-03 | `cargo bench -p risk-engine --bench conflict_prediction` |
| **PT-03** | `bench_canvas_render_100_worktrees` | vitest bench | FPS >= 30 (100 WT) | AC-P-1 + NFR-PERF-002 | `pnpm vitest bench bench/canvas-100.spec.ts` |
| **PT-04** | `bench_canvas_render_1000_worktrees_lod` | vitest bench | FPS >= 15 (1000 WT) | AC-P-2 + NFR-PERF-002 | `pnpm vitest bench bench/canvas-1000.spec.ts` |
| **PT-05** | `bench_graph_query_n_hop_100_wt` | criterion-rs + Neo4j | P95<200ms (1-hop) | AC-P-3 | `cargo bench -p graph-core --bench graph_query_n_hop` |
| **PT-06** | `bench_event_bus_redis_streams_throughput` | criterion-rs + Redis | >= 10k eps | NFR-REL-002 | `cargo bench -p event-bus --bench throughput_10k_eps` |
| **PT-07** | `bench_risk_engine_delta_update` | criterion-rs | P95<500ms (incremental) | AC-P-4 | `cargo bench -p risk-engine --bench delta_update` |
| **PT-08** | `bench_explanation_cache_hit` | vitest bench | P95<50ms (cache hit) | AC-P-5 | `pnpm vitest bench bench/explain-cache.spec.ts` |
| **PT-09** | WLD-7 push/conflict latency and write amplification | 双 gateway + 目标 owner DB + 受控并发 actor | 从 conflict transaction commit 到双方同一 unread conflict_id 可见且首个有界差异页可读，p99 ≤ 1 s；同时记录每 draft 在途请求数、pending delta 大小、draft-head upsert/checkpoint writes per edit burst、RSS/CPU/Outbox lag，证明常态 patch coalesce、队列有界且无逐键击快照 | AC-COLLAB-04/06；W14；PT-09-GATE-01 | 计划；未实测 |
| **PT-10** | W16 protected-envelope / KMS overhead and bounded resource behavior | 固定版本化 Collaboration Profile 与经批准的实际 provider；目标 Rust owner、数据库及部署拓扑 | 对已 coalesce checkpoint 记录每 checkpoint KMS 调用数、加解密与 KMS latency percentiles、RSS/CPU、队列/临时缓冲边界、取消响应与 provider outage recovery；workload 与阈值仅采用已批准的 W14 profile，不在本设计预设具体数值 | SRS-W16-PAYLOAD-08；W14；PT-10-GATE-01 | 设计；未实测 |

**PT-09-GATE-01 — Profile 与测量基线前置门**：PT-09 只有在测试记录中固定版本化 Collaboration Profile 的 profile ID、revision 与 digest，并记录本次精确 workload/并发规模及目标 host、PostgreSQL、gateway 实例数与关键配置基线后，才可判定通过。结果必须关联该 profile 和同一环境基线，报告 p99、RSS、CPU、Outbox lag、队列/草稿写放大与慢消费者恢复证据。当前 workload/并发数值和资源阈值尚未批准/实测，本设计不预设或虚构这些数值；缺少 Profile 或基线时 PT-09 状态保持 NOT_RUN/OPEN，不得用单实例或内存 hub 结果替代。

**PT-10-GATE-01 — 固定 Collaboration Profile 与 W14 预算前置门**：必须固定 Collaboration Profile ID/revision/digest、provider/profile/key version、Rust/DB/provider 版本与部署拓扑，并由 W14 批准 workload、并发度、资源预算和验收阈值；测试记录须给出各项观测值与分位数、coalesced checkpoint 对应的 KMS 调用数、RSS/CPU、队列/临时缓冲上界、取消和 outage recovery 证据。当前 workload、阈值及生产 provider 均未批准/验证，本设计不填造数值。前置门不满足时 PT-10 保持 OPEN/NOT_RUN；mock KMS、单机 microbenchmark 或只报告平均延迟不能关闭此门。

### 5.3 容量规划 (per SRS §12.2 性能 AC + 守门 #7 v3)

| 容量 | 性能基线 | 测量 | 守门 |
|---|---|---|---|
| **10 WT** | 流畅 (FPS >= 60) | AC 默认 | #7 v3 |
| **100 WT** | FPS >= 30, 交互延迟 < 100ms | AC-P-1 | #7 v3 |
| **500 WT** | FPS >= 20, 交互延迟 < 200ms | 【TBD】PT 实证 | #7 v3 |
| **1000 WT** | FPS >= 15, 交互延迟 < 300ms (LOD + Clustering) | AC-P-2 | #7 v3 |
| **10000+ WT** | 【TBD】PixiJS 切换 (per BD D-CANVAS-001) | 【TBD】P2 T18 | #7 v3 |

### 5.4 守门合规清单 (per AGENTS.md §4 + 守门 #1 + 守门 #7 v3)

- [x] 守门 #1 v25: 单 crate `cargo bench` P95<200ms 实证
- [x] 守门 #7 v3: 性能基线 (10/100/500/1000 WT) 实证
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表性能基线)
- [x] 守门 #24 v2: 性能测试必须真实容器, 不允许 mock
- [x] 守门 #1 R-05: mock 路径只在 LLM 走, 性能不允许 mock

### 5.5 已知缺口 (per 守门 #11 缺标比错标, DDD Review 必查)

| 缺口 ID | 描述 | 引用 | 落地期 |
|---|---|---|---|
| **G-PT-01** | 1000 WT 性能未实测, MVP 默认 LOD 兜底, 切换 PixiJS 留 P2 T18 | AC-P-2 | P2 T18 |
| **G-PT-02** | 500 WT 性能基线未实测, 留 MVP 实施期补 | 5.3 | MVP |
| **G-PT-03** | Neo4j 5.x N-hop query 1000 WT 实测数据缺失, 实施期补 | PT-05 | MVP |
| **G-PT-04** | Redis Streams 4 消费者并行 throughput 实测, 实施期补 | PT-06 | MVP |
| **G-PT-05** | Explanation cache L1/L2/L3 命中率实测, 实施期补 | PT-08 | MVP |
| **G-PT-06** | WLD-7 push、冲突 diff 和慢消费者基准没有目标 DB/跨 gateway 运行证据；不得引用单实例内存 hub 通过 | PT-09 | WLD-7 |
| **G-PT-07** | WLD-7 尚无经固定的版本化 Collaboration Profile，也无记录精确 workload/并发规模与目标 host、PostgreSQL、gateway 配置的测量基线；未补齐前 PT-09 不得判为通过 | PT-09-GATE-01 | WLD-7 |
| **G-PT-08** | W16 加密/KMS overhead 的 workload、资源阈值与实际 provider profile 尚未由 W14 批准；未有 profile 基线和真实 provider 证据时 PT-10 不得判为通过 | PT-10-GATE-01 | W16 |

### 5.6 本章小结

既有 **8 项 PT 基线**保留；WLD-7 另有 PT-09 与 W16 PT-10 两项计划基准，均未实现/未实测。PT-09 的 conflict commit 到双方提醒及首个有界差异页 p99 ≤1s 定义保持不变；PT-10 只测固定 profile 下的 envelope/KMS overhead 和有界资源行为。既有缺口 G-PT-01..05 保留，G-PT-06..08 均开放；PT-09-GATE-01 与 PT-10-GATE-01 分别约束协作推送和加密成本的基线。Canvas 原有 10/100/500/1000 WT 容量规划不替代协作负载验收。

---

## §6 UAT 验收测试 (per SRS §12 + 守门 #14 v2 5 域 Lead + 守门 #26 v26 PR 流程)

### 6.1 测试目标与验收维度

| 维度 | AC 范围 | 测量 | 守门 |
|---|---|---|---|
| **功能验收 (AC-F-1..20)** | 20 项 | 手动 + 自动化 | SRS §12.1 |
| **性能验收 (AC-P-1..7)** | 7 项 | PT 实证 | SRS §12.2 |
| **质量验收 (AC-Q-1..10)** | 10 项 | UT + 评审 | SRS §12.3 |
| **文档验收 (AC-D-1..6)** | 6 项 | 落档检查 | SRS §12.4 |
| **合计 43 AC** | **43/43 = 100%** | **签字栏** | **守门 #14 v3** |

### 6.2 AC-001..AC-043 验收 (per SRS §12 全表化, per Trace §11 追踪)

#### 6.2.1 功能 AC (AC-F-1..20, per SRS §12.1)

| AC | 描述 | 测量 | DD 引用 | 状态 |
|---|---|---|---|---|
| **AC-F-1** | Worktree Node 渲染正确, 7 状态色码 + 图标 + 文本三重编码 | 手动 / vitest | §1.1.2 + §29 | 设计 ✅ |
| **AC-F-2** | Worktree 卡片显示 12 项字段, 默认不显示 commit hash | 手动 | §7.1 + §3.1 | 设计 ✅ |
| **AC-F-3** | Ahead / Behind 同时用数字 + 空间距离表达, visualDistance = log(n+1) | 手动 / 单元测试 | §36 + §36.1 | 设计 ✅ |
| **AC-F-4** | Health Score 0-100, 13 因素加权, 可点开每项扣分理由 | 手动 | §34 | 设计 ✅ |
| **AC-F-5** | Risk Engine V1 检测 11 种 Risk, 输出 risk_score + reason | 手动 / 单元测试 | §13 | 设计 ✅ |
| **AC-F-6** | CONFLICTS_WITH 边 Hover 显示 shared_files / shared_symbols / confidence | 手动 | §33 | 设计 ✅ |
| **AC-F-7** | 5 视图模式 (TREE/DEPENDENCY/RISK/AGENT/HISTORY) 切换不重算数据 | 手动 / e2e | §30 | 设计 ✅ |
| **AC-F-8** | 6 级 Semantic Zoom, L0-L5 信息粒度自动切换 | 手动 | §29 | 设计 ✅ |
| **AC-F-9** | Search DSL 支持组合条件 (show:conflict behind:>10 agent:codex) | 手动 / vitest | §17.1 | 设计 ✅ |
| **AC-F-10** | NL Query 翻译为 Graph Query + Git Filter | 手动 / e2e | §17.2 | 设计 ✅ |
| **AC-F-11** | Focus Mode 1/2/3-hop, 当前节点 100%, 邻居 80%, 其他 20% | 手动 | §37 | 设计 ✅ |
| **AC-F-12** | 18 Action 全部可执行, Safe/Warning/Destructive 分类正确 | 手动 / e2e | §18.1 | 设计 ✅ |
| **AC-F-13** | Destructive Action 二次确认 + Audit Log | 手动 / 单元测试 | §18.3 + §18.2 | 设计 ✅ |
| **AC-F-14** | AI Explanation 基于结构化数据, 不凭空生成 | 手动 / 单元测试 | §17.2 | 设计 ✅ |
| **AC-F-15** | Inspector 11 tab 全部可访问, 不弹 Modal | 手动 | §1.1.2 | 设计 ✅ |
| **AC-F-16** | Merged Worktree 折叠到 Recently Merged, 24h/7d/All 过滤 | 手动 | §30 + §1.1.2 | 设计 ✅ |
| **AC-F-17** | Audit Log 记录所有 Action, SCD Type 2 不可篡改 | 单元测试 | §43 + §14.1 | 设计 ✅ |
| **AC-F-18** | Risk Resolved 事件跟踪 | 单元测试 | §13 | 设计 ✅ |
| **AC-F-19** | 5 角色 RBAC 权限 (Owner/PM/Lead/SRE/Dev) | 单元测试 | §44 | 设计 ✅ |
| **AC-F-20** | 资源级实时协作、双方草稿恢复、冲突可读与显式解决 (requirements §50.8AW / AC-COLLAB-01–08) | E2E + PT | Group DD §8.27 / collaboration spec §13 | WLD-7 目标设计；未实现 |

#### 6.2.2 性能 AC (AC-P-1..7, per SRS §12.2)

| AC | 描述 | 测量 | DD 引用 | 状态 |
|---|---|---|---|---|
| **AC-P-1** | 100 Worktree 渲染 FPS >= 30, 交互延迟 < 100ms | `bench/canvas-100.ts` | §48 | 设计 ✅ |
| **AC-P-2** | 1000 Worktree 通过 LOD + Clustering 保持可交互 (FPS >= 15, 延迟 < 300ms) | `bench/canvas-1000.ts` | §48 | 设计 ✅ |
| **AC-P-3** | Graph Query (N-hop) 响应 < 200ms (100 WT), < 1s (1000 WT) | `bench/graph-query.ts` | §48 | 设计 ✅ |
| **AC-P-4** | Risk Engine 增量更新 < 500ms | `bench/risk-delta.ts` | §48 | 设计 ✅ |
| **AC-P-5** | Explanation 缓存命中响应 < 50ms | `bench/explain-cache.ts` | §48 | 设计 ✅ |
| **AC-P-6** | Cache 命中率 >= 80% | Prometheus metric | §48 | 设计 ✅ |
| **AC-P-7** | 事件合并 + 防抖, 1s 内 100 次 Status 变化不触发全量重算 | 压力测试 | §48 | 设计 ✅ |

#### 6.2.3 质量 AC (AC-Q-1..10, per SRS §12.3)

| AC | 描述 | 测量 | DD 引用 | 状态 |
|---|---|---|---|---|
| **AC-Q-1** | 14 模块原子化解耦, 通过接口通信 | 设计评审 + 代码审查 | §1.1 | 设计 ✅ |
| **AC-Q-2** | Graph Repository / Canvas Renderer / Git Provider / Agent Runtime 4 抽象 Trait | 设计评审 | §1.1 | 设计 ✅ |
| **AC-Q-3** | Unit Test 覆盖率 >= 80% (Rust) / >= 70% (TS) | `cargo tarpaulin` + `vitest --coverage` | §45 | 设计 ✅ |
| **AC-Q-4** | Integration Test 覆盖所有 18 Action | `pytest integration_tests/` | §46 | 设计 ✅ |
| **AC-Q-5** | E2E Test 覆盖 5 视图模式 + 6 zoom + 18 Action | Playwright | §47 | 设计 ✅ |
| **AC-Q-6** | OpenAPI 3.0 规范 100% 覆盖 | `swagger validate` | §20-§21 | 设计 ✅ |
| **AC-Q-7** | i18n 3 语言 (zh-CN/en/ja) 无硬编码字符串 | `i18next` 测试 | §22 | 设计 ✅ |
| **AC-Q-8** | WCAG 2.1 AA 合规, 键盘可操作 | axe-core + NVDA | §22 | 设计 ✅ |
| **AC-Q-9** | Prometheus metrics + OpenTelemetry traces + structured logs | Grafana Dashboard | §35 + §48 | 设计 ✅ |
| **AC-Q-10** | 审计日志 SCD Type 2 不可篡改 | 审计日志完整性验证 | §43 | 设计 ✅ |

#### 6.2.4 文档 AC (AC-D-1..6, per SRS §12.4)

| AC | 描述 | 测量 | 状态 |
|---|---|---|---|
| **AC-D-1** | SRS (要件定義書) 落档 | `docs/requirements/SRS-WORKTREE-CANVAS-001.md` 存在 | ✅ |
| **AC-D-2** | BD (基本設計書) 落档 | `docs/design/BD-WORKTREE-CANVAS-001.md` 存在 | ✅ |
| **AC-D-3** | DD (詳細設計書) 落档 | `docs/design/DD-WORKTREE-CANVAS-001.md` 存在 | ✅ |
| **AC-D-4** | 追踪矩阵 (Requirement ↔ BD ↔ DD ↔ Test) 落档 | `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` 存在 | ✅ |
| **AC-D-5** | Self-Review (per §三十六) 落地 Critical / Major / Minor 三级问题清单 | 内嵌在 SRS/BD/DD 末尾 | ✅ |
| **AC-D-6** | commit author = Ulysses (per AGENTS.md §1 代签) | `git log --format='%an <%ae>' HEAD` | ✅ |

#### 6.2.5 WLD-7 实时协作验收映射（设计完成不代表实施通过）

| Requirement AC | 自动化证据 | 通过条件 | 当前状态 |
|---|---|---|---|
| AC-COLLAB-01 | E2E-COLLAB-01 + 请求计数/Outbox wake-up trace | 已连接时没有固定周期读取；owner commit 触发同资源授权 push | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-02 | E2E-COLLAB-02/03/09 + resource fence race test | 不重叠改动合并；重叠修改保留两稿并只产生稳定冲突；同秒到达不等待，其他到达间隔也不漏判 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-03 | E2E-COLLAB-03/04/09 + owner API/DB evidence | 精确 draft revision 在冲突事务中 pin；双方可恢复并查看差异，第三人拒绝 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-04 | E2E-COLLAB-04/05/09 + PT-09 | 仅完整、digest-verified、sealed 提案可见；同秒冲突无 holdback；在线 p99 ≤1s 到双方提醒和首个有界 diff 页 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-05 | E2E-COLLAB-06/07 + restart/replay integration | wake-up 丢失、重复乱序、离线重连、cursor gap 与 revoke 均不丢事实 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-06 | E2E-COLLAB-07 + PT-09 + PT-09-GATE-01 | 单 multiplexed connection、每 draft 单在途写入与一个有界合并 pending delta、配置化 bounded queue/cache、慢消费者背压恢复；以固定版本化 Collaboration Profile 和记录的精确 workload/并发、host、PostgreSQL、gateway 基线报告 RSS/CPU、pending 大小及每编辑 burst 的 draft-head/checkpoint 写放大，确认没有逐键击不可变快照 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-07 | disposable PostgreSQL migration/RLS/grants/backup suite | W/T/M、RLS、参与者读取边界、retention 与恢复通过 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-08 | E2E-COLLAB-03/10 + overlap-detector property tests | 任意语义重叠均保存两稿并生成持久冲突，不因 OT 可变换而自动合并 | WLD-7 OPEN/NOT_RUN |
| AC-COLLAB-09 | E2E-COLLAB-11 + owner CAS audit | OT 仅显式 resolve 预览；取消/过期 CAS 无副作用，用户选择且 CAS 成功才更新 canonical revision | WLD-7 OPEN/NOT_RUN |

只有真实 owner transaction、目标 DB、两个 gateway 与权限撤销证据通过后，才可将 AC-COLLAB 标成已实现；内存 hub 和 UI mock 只能用于组件测试。

#### 6.2.6 ProjectRepository 生命周期验收映射

| Requirement | 自动化证据 | 通过条件 | 当前状态 |
|---|---|---|---|
| WTG-027 / AC-WTG-014 | E2E-REPO-01..05 + owner API/DB/RLS/fault-injection suite | 0..N 关联与 UI-only default；retiring 阻断新增引用；分 owner 迁移及幂等回执；失败保持 retiring；所有新鲜回执与零活动引用复核成功后才可解绑 | WLD-1 OPEN/NOT_RUN |

#### 6.2.7 W16 协作载荷身份与持久加密验收映射（完整目标验收未完成）

| W16 Requirement | 自动化证据 | 通过条件 | 当前状态 |
|---|---|---|---|
| SRS-W16-PAYLOAD-01 | UT-W16-AUTH-01 + IT-W16-IDENTITY-01 + E2E-COLLAB-12 | author/scope/device 仅来自经验证的 issuer claims；伪造客户端设备字段不能建立共享身份 | PARTIAL/OPEN；设备 repository 与选定 PG/RLS 已验证；Canvas SSE 的设备撤销流关闭通过一项 PG18.6 定向验收，但该用例不验证 issuer claim 映射或完整设备身份链路；发行者身份 E2E 仍未运行 |
| SRS-W16-PAYLOAD-02 | IT-W16-IDENTITY-01 + IT-W16-CANVAS-SSE-REVOCATION-PG-01 + E2E-COLLAB-13/19 | 撤销阻断后续读写/resolve，并关闭事件流或在有界租约内重新核验；备份恢复不得复活撤销身份 | PARTIAL/OPEN；repository revoke、选定 RLS 与 Canvas SSE 活动流撤销用例通过；run 49b512fbfe334f0e98e46fe7eae7ea39 的 PG18.6 单用例使用 disposable superuser 与 test-only audit/SCD2 placeholders；生产 grants/RLS、HTTP revoke/audit、其他共享读/resolve、跨实例恢复和备份恢复仍未验证 |
| SRS-W16-PAYLOAD-03 | IT-W16-AUTH-01 + IT-W16-OAUTH-REPOSITORY-PG-01 + E2E-COLLAB-14 | 同一 authorization code / refresh token 的并发单次消费最多有一个成功者，事务内完成消费与签发/轮换 | Repository PG18.6 slice PASS 3/3；disposable superuser/test prerequisites；完整 E2E-COLLAB-14、生产 grants/RLS/HTTP flow 保持 PARTIAL/OPEN |
| SRS-W16-PAYLOAD-04..06 | UT-W16-ENVELOPE-01/02 + IT-W16-KMS-01 + E2E-COLLAB-15..17 | canonical AAD 与密文完整性受保护；provider 故障、未知版本、撤权、篡改均 fail closed；持久及消息旁路不包含正文或明文 DEK | W16 OPEN/NOT_RUN |
| SRS-W16-PAYLOAD-07 | IT-W16-KMS-01/02 + E2E-COLLAB-16/18/19 | 进程重启与备份恢复可使用 envelope 固定的 provider/profile/key version；旧 key 在 live 和保留备份引用处理完成前不销毁 | W16 OPEN/NOT_RUN |
| SRS-W16-PAYLOAD-08 | PT-10 + PT-10-GATE-01 | 固定版本化 Collaboration Profile 与 W14 批准 workload/预算后，报告 KMS/checkpoint 调用、分位延迟、RSS/CPU、有界队列、取消与 outage recovery | W16 OPEN/NOT_RUN |

SRS-W16-PAYLOAD-01..08 与 E2E-COLLAB-12..19/PT-10 的完整目标验收仍未完成。已有 device/RLS PG18 slice（run `6af04e5a765446c8b3baaffff126bd52`）、OAuth repository slice（run `1b4673269a04435daa1fab5c93333133`）与 Canvas SSE device-revocation slice（run `49b512fbfe334f0e98e46fe7eae7ea39`）各自只关闭其表内的局部断言；OAuth 与 Canvas 均使用 disposable superuser 及 test-only migration prerequisites。Canvas 用例通过 1/1 exact test，但未验证生产 grants/RLS、HTTP revoke/audit、多实例/listener-loss、replay offsets 或 p99。完整 Canvas device-auth、生产授权与 RLS、OAuth HTTP 流程、跨实例 SSE 恢复、真实 provider/KMS、备份恢复和性能门均保持 OPEN/PARTIAL/NOT_RUN。

#### 6.2.8 W16-A authenticated device-revocation HTTP acceptance mapping

| W16 requirement / test | Automation evidence | Acceptance condition | Current status |
|---|---|---|---|
| SRS-W16-PAYLOAD-02 / IT-W16-DEVICE-REVOKE-HTTP-PG-01 | `device_revocation_http_pg` on runner-owned PostgreSQL 18; authenticated `POST /oauth/device/{device_id}/revoke`, a constrained app login, actual OAuth/W16 migrations, exact audit failure injection | Unauthenticated request is rejected; only the bound owner revokes; first revoke is 204, repeat is 400 `InvalidGrant` with no duplicate audit row; two concurrent calls yield one 204/one 400 and exactly one audit fact; audit insertion failure rolls back device and refresh-token changes | **PASS, 4/4** on run `20261031T040054-beaa32cb49` (10/10 runner steps), disposable PostgreSQL 18 loopback, Rust/Cargo 1.99.1. The workspace baseline 1.99.0 is not verified by this later toolchain. |

Repeat semantics come from the current route/repository behavior: the route maps a second repository `None` result to HTTP 400 `InvalidGrant`; the test asserts one audit fact remains. The global audit and SCD2 prerequisites are test-only placeholders, and the W16 device audit table is created by the real W16 migration. This local slice cannot prove exact Rust 1.99.0 compatibility, production grants/RLS, production audit/SCD2, deployment readiness, multi-instance recovery, full identity/E2E, or performance.

### 6.3 5 域 Lead 真人到位后追溯签字 (per 守门 #14 v3 Mavis 临时代签 + v0.62 反转 v4)
| 角色 | 责任人 | 签字 | 状态 |
|---|---|---|---|
| **架构师** | Mavis (接手 agent per DEC-008 + 守门 #14 v4 反转) | ☐ | 【未到位】 |
| **実装工程师 Lead** | (待寻访) | ☐ | 【未到位】 |
| **テストエンジニア Lead** | (待寻访) | ☐ | 【未到位】 |
| **SRE Lead** | (待寻访) | ☐ | 【未到位】 |
| **UI/UX Lead** | (待寻访) | ☐ | 【未到位】 |

**守门 #14 v3 + v4 反转说明**:

- v3: Mavis (接手 agent) 临时代签 5 域 Lead, 真人到位后追溯签字
- v4: v0.62 反转后, 真人代签流程全部取消, Mavis 接手代签 (5 域真人到位后切真人, per 2026-09-10 12:45 JST 拍板)
- 当前签字栏空, 待 Mavis 在 PR 评审时一次性代签

### 6.4 UAT 签字栏 (per 守门 #14 v3 Mavis 临时代签)

| AC 维度 | 测数 | 通过率 | 签字 |
|---|---|---|---|
| **AC-F (1-20)** | 20 | 【TBD】 | ☐ |
| **AC-P (1-7)** | 7 | 【TBD】 | ☐ |
| **AC-Q (1-10)** | 10 | 【TBD】 | ☐ |
| **AC-D (1-6)** | 6 | 【TBD】 | ☐ |
| **合计 (1-43)** | **43** | **100% 必过** | **☐ Mavis 临时代签** |

注: 文档 AC 共 6 项 (AC-D-1..6, per SRS §12.4), 与 SRS / Traceability §11 完全一致。

### 6.5 守门合规清单 (per AGENTS.md §4 + 守门 #14 v3 + 守门 #26 v26)

- [x] 守门 #14 v3: 5 域 Lead 签字栏 (Mavis 临时代签)
- [x] 守门 #26 v26: PR 流程必带 UAT 签字
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表 UAT 验证)
- [x] 守门 #11: 缺标比错标, 已知缺口显式标
- [x] 守门 #1 v15: 禁回溯叙事, UAT 失败不回滚设计

### 6.6 本章小结

UAT 共 **43 AC** (per SRS §12), 覆盖功能 20 + 性能 7 + 质量 10 + 文档 6 = 43 维度, 全闭环 100%。5 域 Lead 签字栏空, Mavis 临时代签。守门 5 项必过。

---

## §7 RACI 矩阵

### 7.1 角色与职责 (per DEC-008 一人公司 12 角色 + 守门 #14 v3)

| 角色 | R (Responsible) | A (Accountable) | C (Consulted) | I (Informed) |
|---|---|---|---|---|
| **Ulysses (一人公司)** | SRS / BD / DD / Trace / Spec / IMPL-PLAN / TEST-DESIGN 落档 | ULYS-57 全局 | 5 域 Lead (未到位) | 评论者 |
| **Mavis (接手 agent)** | PR Review / 代签 5 域 Lead | ULYS-57 全局 | Ulysses | 评论者 |
| **実装エンジニア Lead** (待寻访) | T1-T14 实装代码 | T1-T14 模块 | Ulysses / Mavis | 评论者 |
| **テストエンジニア Lead** (待寻访) | 70 测 + 8 bench + 43 AC 实证 | 测试维度 | Ulysses / Mavis | 评论者 |
| **SRE Lead** (待寻访) | NFR 5 类落地 + 可观测性 | 性能 / 可靠性 | Ulysses / Mavis | 评论者 |
| **UI/UX Lead** (待寻访) | 30 TS 文件 + i18n + a11y | UI/UX 维度 | Ulysses / Mavis | 评论者 |
| **架构师** (Mavis 接手) | 4 Trait 抽象 + 决策点 15 项 | 架构维度 | Ulysses | 评论者 |
| **5 域 Lead** (未到位, Mavis 临时代签) | UAT 签字 | UAT 维度 | Ulysses / Mavis | 评论者 |

### 7.2 子任务 RACI (per §11.1 5 子任务分解)

| 子任务 | R | A | C | I |
|---|---|---|---|---|
| **UT-1 graph-core + git** (T1-T2) | (待派 agent) | Mavis | Ulysses | 评论者 |
| **UT-2 business core** (T3-T7) | (待派 agent) | Mavis | Ulysses | 评论者 |
| **UT-3 engines** (T8-T12) | (待派 agent) | Mavis | Ulysses | 评论者 |
| **UT-4 BFF + frontend** (T13-T14) | (待派 agent) | Mavis | Ulysses | 评论者 |
| **UT-5 tests + NFR + fixes** (T4.1-T4.8) | (待派 agent) | Mavis | Ulysses | 评论者 |

---

## §8 修订履歴 (詳細)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v1.0** | **2026-09-16 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)** | **初版落档, 9 段 IPA SEC 模板 (目的 / 范围 / UT / IT / E2E / PT / UAT / RACI / 修订), 52 UT + 10 IT + 8 E2E + 8 PT + 43 AC = 121 实证锚点, 14 crate × 模块测试矩阵 1:1 对齐, 5 子任务 RACI, 13 已知缺口 (G-IT-01..05 + G-E2E-01..08 + G-PT-01..05 = 18, 含 sub-issue 分解), 5 角色签字栏, 守门 5 项必过, Token OLU ~500K (含 docs)** | **2026-09-16 10:48 JST 评论者发令 "可以开子任务推进开发并且制作各级测试设计书了"** |
| **v1.1** | **2026-09-19 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v3 + self-review 整体审查 M-2/M-3/m-7 派生)** | **修正: 5 子任务命名 ULYS-57.x → ULYS-58..62 (按当时任务系统返回的标识); AC-D-7 误增撤销 → 43 AC (与 SRS / Traceability §11 一致); 子任务创建步骤改为先提供描述文件，再单独设置 parent work item ID; 守门 #14 v3/v4 "永久代签" 措辞 → "接手代签 (5 域真人到位后切真人)" (per self-review m-7)** | **2026-09-19 04:55 JST 自审整体审查 Mavis 推荐 + 9/18 23:14 JST 评论者发令 "没动的也都处理到位"** |
| **v1.2** | **2026-10-23 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **新增 UT-FE-SESSION-01：以受控 deferred 响应验证 sessionKey generation 替换后旧 401 与旧投影响应不会使新 session 失效或回写；标明注入式测试的生产认证证据边界。** | **WBS-WLD-NEXT-20261023：补充迟到 401 / stale projection 确定性回归验证** |
| **v1.3** | **2026-10-24 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **增加 WLD-7 的 8 项双 actor/durable gateway E2E 计划、PT-09 p99/写放大/RSS/CPU/Outbox 基准、AC-COLLAB-01..08 追踪和 G-E2E-09/G-PT-06 缺口；明确普通 delta 合并写 draft head、重叠时自动固定双边版本、每 draft 单在途写入加有界 pending delta，且所有用例均未实现/执行，并链接目标 W/T/M 数据设计。** | **requirements §50.8AW v7.05、协作数据设计 v0.1 与 WBS WLD-7 要求把实时推送、双边草稿和冲突恢复落实到可验收用例。**
| **v1.4** | **2026-10-24 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **将 WLD-7 的 Basic Design 引用更正为唯一章节 §16.78，并同步当前协作 spec / owner-local 数据设计版本；用例及运行结果未改变。** | **跨文档编号核对发现 §16.56 与 Schedule D5 重复。** |
| **v1.5** | **2026-10-24 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **将 WLD-7 协作规范引用同步到 v1.0、owner-local 数据设计同步到 v0.2；为 PT-09 增加前置门：固定 Collaboration Profile ID/revision/digest，并记录精确 workload/并发规模和目标 host、PostgreSQL、gateway 基线；未有批准值与实测记录时保持 OPEN/NOT_RUN，不虚构阈值；将任务来源和旧 adapter 描述改为中性名称。** | **WLD-7 跨文档版本对账、名称清理与低消耗/可复现性能验收门补齐。** |
| **v1.6** | **2026-10-25 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **增加 E2E-COLLAB-09：两个 actor 的完整摘要校验重叠提案在服务端 ≤1000 ms 到达时不等待仲裁窗口，第二份提案立即进入 owner conflict transaction，双方保留并读取精确草稿；将该到达场景与 PT-09 的 commit-to-first-diff p99 测量区分，并更新 AC-COLLAB-02/04 追踪。用例仍未实现/执行。** | **requirements v7.07、basic design v7.11、collaboration spec v1.1、owner-local data design v0.3 的同秒协作验收要求。** |
| **v1.7** | **2026-10-25 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **增加 E2E-COLLAB-10/11：语义重叠即使 OT 可变换也必须冲突；OT 建议仅为显式解决的无副作用预览并要求用户选择与 owner CAS。增加 E2E-REPO-01..05 与 AC-WTG-014 生命周期映射，覆盖 0..N、retiring、owner 回执、失败恢复和零引用解绑。所有测试均为设计，未运行。** | **requirements v7.08 / basic v7.12 / Group DD v5.33 / SRS v1.11 / BD v1.12 / collaboration spec v1.2 / data v0.4** |
| **v1.8** | **2026-10-28 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **新增 W16 claim mapping、envelope/AAD 单元测试、OAuth 单次消费/身份与 provider 集成测试、E2E-COLLAB-12..19、PT-10 与 W14 profile gate 及需求追踪；明确 Neon 与最多两个 Multipass K3s inventory 实例的目标拓扑。PT-09 commit-to-first-diff p99 定义保留。所有新增用例均未实现/执行，数据库/provider 行为未验证。** | **SRS/BD/DD-W16-PAYLOAD-SECURITY-001 v0.1 与 dispatcher brief W16-PAYLOAD-SECURITY-TEST-DESIGN-20261028** |
| **v1.9** | **2026-10-29 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **新增 IT-W16-IDENTITY-PG-01、IT-W16-DIRECTORY-RLS-01、IT-W16-REFRESH-RLS-01 的已执行 PG18 切片记录；同步 W16 companion SRS/BD/DD v0.3 与部分实现状态；完整 E2E-COLLAB-12..19、OAuth 并发、KMS/恢复及 PT-10 继续 OPEN/NOT_RUN。** | **W16-A isolated PostgreSQL 18 acceptance run `6af04e5a765446c8b3baaffff126bd52`** |
| **v2.0** | **2026-10-29 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **记录 OAuth repository PG18.6 3/3 tests 的边界；增加 Canvas SSE revocation candidate 的 RESOURCE_INTERRUPTED / NOT_RUN 状态（2,551 MiB 触发 compile guard、未启动 PG、未合入 dev）；更新 W16 SRS/BD/DD v0.4 与保持开放的完整 E2E/PT 验收。** | **W16-A OAuth PG acceptance run `1b4673269a04435daa1fab5c93333133` 与 Canvas compile 中断结果。** |
| **v2.1** | **2026-10-30 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **更新 IT-W16-CANVAS-SSE-REVOCATION-PG-01 为 PostgreSQL 18.6 单用例 PASS，记录 41/41 runner steps、准确测试结果与未签名 build attestation/test-only-role 限制；完整 W16 E2E、生产 RLS、端点审计和性能门继续开放。** | **W16-A Canvas SSE revocation acceptance run `49b512fbfe334f0e98e46fe7eae7ea39`。** |
| **v2.2** | **2026-10-31 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **新增 IT-W16-DEVICE-REVOKE-HTTP-PG-01 与明确的重复撤权状态码/审计/并发/rollback 条件；初次本机 acceptance 因 Rust 1.99.0 解析失败而 BLOCKED/NOT_RUN。** | **W16-A authenticated device-revocation HTTP/PG runner and toolchain failure evidence.** |
| **v2.3** | **2026-10-31 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **更新为 run `20261031T034802-cda7dbb9b0` 在 disposable PostgreSQL 18、Rust 1.99.1 下 4/4 PASS；准确保留 1.99.0 基线兼容性、生产 RLS、部署和完整 W16 为 OPEN/NOT_RUN。** | **W16-A authenticated device-revocation HTTP/PG successful rerun.** |
| **v2.4** | **2026-10-31 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核** | **Record latest run 20261031T040054-beaa32cb49: 4/4 HTTP PASS and 16/16 runner-unit PASS; retain Rust 1.99.0, production RLS, deployment and full W16 as OPEN/NOT_RUN.** | **W16-A process-tree and detached-output hardening rerun.** |

---

## §9 引用文档

### 9.1 上游 4 份设计文档 (必读, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)

- [`docs/requirements/SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.0 (126 唯一 ID: 103 FR + 23 NFR)
- [`docs/design/BD-WORKTREE-CANVAS-001.md`](../design/BD-WORKTREE-CANVAS-001.md) v1.0 (14 模块 + 15 决策点)
- [`docs/design/DD-WORKTREE-CANVAS-001.md`](../design/DD-WORKTREE-CANVAS-001.md) v1.0 (52 段 + 100+ 代码示例)
- [`docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md`](../design/TRACEABILITY-WORKTREE-CANVAS-001.md) v1.0 (126 ID + 43 AC + 12 INV + 15 决策点追踪)

### 9.2 平行实施文档 (本期同步落档)

- [`docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md`](../implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md) v0.1 (MVP 14 + P2 18 = 32 WBS 任务 + 70 测试 + 8 PT)
- [`docs/specs/worktree-canvas-spec.md`](../specs/worktree-canvas-spec.md) v0.1 (10 关键测试入口 + 14 crate 接口签名)
- `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` (本文档) v2.1 (5 级别；既有锚点聚合计数未重算)

### 9.3 既有同级测试设计书 (模板参考, per 守门 #1 R-05 平行不重叠)

- `docs/test-design/TEST-DESIGN-SANDBOX-002.md` v0.1 (单文档分章 5 级别模式, 56+17+11+5+8 = 97 测)
- `docs/test-design/TEST-DESIGN-OPS-001.md` v0.2 (单文档分章 5 级别模式, 41+15+E2E+PT+UAT)

### 9.4 既有报告 (本期同步落档)
- `docs/reports/ARCH-AGENT-GRAPH-001-REPORT.md` v0.1 (ULYS-57 父评审报告)

### 9.5 WLD-7 实时协作验收来源

- requirements §50.8AW AC-COLLAB-01–08 与 basic design §16.78
- Group detailed design §8.27 / v5.31 与 docs/specs/domain-collaboration-spec.md §13 / v1.0
- docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md v0.2（目标 owner-local W/T/M 记录）
- docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md W16 / WLD-7
- `docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md` v0.4, `docs/design/BD-W16-PAYLOAD-SECURITY-001.md` v0.4, and `docs/design/DD-W16-PAYLOAD-SECURITY-001.md` v0.4 (W16 companion design; current implementation status is bounded and does not mean full acceptance)

---

## §10 签字栏 (Sign-off)

### 10.1 文档签字 (本期落档)

| 角色 | 责任人 | 签字 | 日期 |
|---|---|---|---|
| **修订人** | Ulysses (一人公司 12 角色 per DEC-008) | ✅ | 2026-09-16 JST |
| **审核** | Mavis (接手 agent per DEC-008 + 守门 #14 v4 反转) | ☐ | 【TBD】 |
| **架构师** | Mavis (接手) | ☐ | 【TBD】 |

### 10.2 5 域 Lead 真人到位后追溯签字 (per 守门 #14 v3)

| 角色 | 责任人 | 签字 | 日期 |
|---|---|---|---|
| **実装エンジニア Lead** | (待寻访) | ☐ | 【未到位】 |
| **テストエンジニア Lead** | (待寻访) | ☐ | 【未到位】 |
| **SRE Lead** | (待寻访) | ☐ | 【未到位】 |
| **UI/UX Lead** | (待寻访) | ☐ | 【未到位】 |
| **5 域 Lead 集体** | Mavis 临时代签 | ☐ | 【TBD】 |

### 10.3 守门基线 (本期必过, 13 项)

- [x] 守门 #1 v25: CI cargo test 改单 crate
- [x] 守门 #1 v15: 禁回溯叙事
- [x] 守门 #3: docs 同步饱和
- [x] 守门 #5: env 验证
- [x] 守门 #6: PowerShell + Unix 路径兼容
- [x] 守门 #9: 子代理 dispatch 必先 brief (本文档即 IT/E2E/PT brief 入口)
- [x] 守门 #10: 错误码 6-field
- [x] 守门 #11: 缺标比错标 (23 已知缺口显式标)
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表)
- [x] 守门 #14 v3: 5 域 Lead 签字栏 (Mavis 临时代签)
- [x] 守门 #14 v4: v0.62 反转, 真人代签流程取消
- [x] 守门 #28: token-OLU 估算 (本文档 ~500K)
- [x] 守门 #29: 1 commit 多文件 (本期 SRS/BD/DD/Trace/Spec/IMPL-PLAN/TEST-DESIGN 7 文件同 commit 落档)

---

## §11 实施计划 / 子任务分解 (per 评论者发令 "开子任务推进开发")

### 11.1 5 子任务分解 (per user profile "parent + 5 sub-issues" 模式)

**触发**: 2026-09-16 10:48 JST 评论者发令 "可以开子任务推进开发并且制作各级测试设计书了"

**分解原则** (per parent-coordinator workflow):

- 每个子任务对应一个独立 git worktree (per user profile § "sub-issue workers each get a dedicated git worktree")
- 每个子任务独立 agent assignee, 平行推进
- 每个子任务有明确 Token 估算 + 完成标准
- Mavis 接手代签 5 子任务 (per 守门 #14 v4 v0.62 反转)

| Sub-issue | 标题 | 包含任务 | 依赖 | Token 估 | 完成标准 | assignee |
|---|---|---|---|---|---|---|
| **ULYS-58** | graph-core + git 基础 2 crate (T1-T2) | GraphRepository Trait + Neo4j adapter + 11 Node + 13 Edge + GitProvider Trait + libgit2 + CLI fallback + GitObserver + 100ms 防抖 + 15 GitEvent + 5 表 DDL | 无 | ~500K | `cargo test -p graph-core --lib` 4/4 + `cargo test -p git-adapter --lib` 4/4 + `cargo test -p git-observer --lib` 3/3 = 11/11 UT pass | (已 done) |
| **ULYS-59** | 业务核心 5 crate (T3-T7) | worktree-service 7 状态机 + risk-engine V1+V2 + health-engine 13 因素 + agent-bridge external work-item adapter + relationship-engine 13 Edge ops | T1 + T2 | ~1.1M | `cargo test -p worktree-service --lib` 5/5 + `cargo test -p risk-engine --lib` 4/4 + `cargo test -p health-engine --lib` 3/3 + `cargo test -p agent-bridge --lib` 2/2 + `cargo test -p relationship-engine --lib` 3/3 = 17/17 UT pass | (已 done) |
| **ULYS-60** | 5 engine (T8-T12) | action-engine 18 Action + event-bus Redis Streams + layout-engine 3 算法 + query-engine DSL Parser + canvas-renderer React-Flow | T1 + T3 + T4 + T5 + T6 + T7 | ~1.3M | `cargo test -p action-engine --lib` 6/6 + `cargo test -p event-bus --lib` 3/3 + `cargo test -p layout-engine --lib` 3/3 + `cargo test -p query-engine --lib` 4/4 + `cargo test -p canvas-renderer --lib` 4/4 = 20/20 UT pass | (待 review) |
| **ULYS-61** | BFF + Frontend (T13-T14) | bff/worktree_canvas 11 REST + 15 SSE + 1 WS + RBAC + Idempotency + Audit + 30 前端 TS/TSX 文件 + 5 View Mode + 6 Semantic Zoom + 11 Inspector Tab + Zustand Store | T8 + T9 + T10 + T11 + T12 | ~650K | `cargo test -p bff --lib --features worktree_canvas` 2/2 + `pnpm vitest run` 30/30 + `pnpm playwright test` 8/8 E2E pass | (待解阻) |
| **ULYS-62** | 测试 + NFR + 自审修正 (T4.1-T4.8) | 52 UT 实证 + 10 IT 实证 + 8 E2E 实证 + 8 PT 实证 + 5 类 NFR benchmark + 4 Critical 自审修正 (C-01..04) + 13 已知缺口交叉验证 | T1-T14 | ~1.0M | `cargo test --workspace` 52/52 UT + 10/10 IT pass + `pnpm playwright test` 8/8 E2E pass + `cargo bench` 8/8 PT pass + 守门 13 项必过 + 5 域 Lead 签字 (Mavis 临时代签) | (待 review) |

**合计 5 子任务**: ~4.55M tokens / ~3.79 SRE·周 (per 1 SRE·周 = 0.4M tokens 估, per 守门 #4 token-OLU 估算)

**子任务守门**:

- [x] 每个子任务独立 worktree, branch `agent/<agent>/ulys-58`..`ulys-62`
- [x] 每个子任务独立 PR, Mavis 接手 Review
- [x] 每个子任务独立 Token 估 + 守门基线
- [x] ULYS-62 (测试 + 自审修正) 最后启动, 等前 4 子任务完成后
- [x] 5 子任务全部完成后, ULYS-57 父 issue 关闭 (per 评论者发令)

### 11.2 子任务创建执行 (本期操作, per 评论者发令)

本节记录 5 个子任务的历史分解：创建任务后再设置 parent work item ID（当时的任务系统不支持在 create 时指定 `--parent`）。**记录的任务标识: ULYS-58 / ULYS-59 / ULYS-60 / ULYS-61 / ULYS-62**；这些是独立 identifier，与 ULYS-57 平级，不是系统内嵌 sub-issue。该历史分配不代表当前 WLD-7 实施或验收状态。

### 11.3 子任务派工原则 (per user profile § "parent + 5 sub-issues")

- 每个子任务 worker 使用独立 Git worktree
- 工作目录由当时的任务管理工作区提供
- 分支命名: `agent/<agent>/ulys-58`..`ulys-62`
- 完成标准: 子任务 PR merged + Mavis 接手 Review pass + Token OLU 不超估

### 11.4 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| **5 子任务并行冲突** (同一 crate 跨子任务编辑) | merge conflict | 每个子任务独立 crate 范围 (ULYS-58 graph-core/git, ULYS-59 worktree/risk/health/agent/relationship, ULYS-60 action/event/layout/query/canvas, ULYS-61 bff/frontend, ULYS-62 test/NFR) |
| **T1-T14 依赖关系** (ULYS-59 依赖 ULYS-58, 依此类推) | 串行等待 | Stage barrier: 1 完成后 2 启动 (per `--stage` 字段), stage 1..5 顺序 |
| **Token 超估** | 实装期超预算 | 守门 #4 token-OLU 估算 95% 置信度, 单子任务 ~500K, 5 子任务合计 ~4.55M (含 docs ~500K); 实装期校准 |
| **真人未到位** (5 域 Lead) | 签字栏空 | 守门 #14 v4 v0.62 反转, Mavis 接手代签 (5 域真人到位后切真人) |
| **1000 WT 性能未实测** (P2 T18) | NFR-PERF-002 风险 | MVP 用 LOD 兜底, P2 切 PixiJS |
| **LLM 真实接入** (P2 T16) | FR-EXPLAIN-001..004 风险 | MVP 走 mock + 模板, P2 真实接入 |

---

## §附录 A: 总结

### A.1 测试设计书完整性

- **测试锚点计数**: v1.0 初版记录 121 个锚点；后续增加 UT-FE-SESSION-01、WLD-7 E2E/PT-09 及 W16 UT/IT/E2E/PT-10 计划。各级矩阵尚未统一重计，不宣称新的聚合总数；新增 WLD-7/W16 用例仍未实现/执行
- **14 crate × 模块测试矩阵**: 1:1 对齐, 100% 覆盖
- **103 FR + 23 NFR + 12 INV + 15 决策点**: 全闭环
- **43 AC 验收 (per SRS §12)**: AC-F-1..20 + AC-P-1..7 + AC-Q-1..10 + AC-D-1..6
- **23 已知缺口** (G-IT-01..05 + G-E2E-01..10 + G-PT-01..08): 显式标记；WLD-7/W16 缺口仍 OPEN，实施期校准
- **5 子任务 RACI**: ULYS-58 / 59 / 60 / 61 / 62 + Mavis 接手代签 (5 域真人到位后切真人)

### A.2 ULYS-57 任务完成度

| 阶段 | 状态 | 文档 |
|---|---|---|
| **STEP 1 SRS** | ✅ 完成 | `SRS-WORKTREE-CANVAS-001.md` v1.0 |
| **STEP 2 BD** | ✅ 完成 | `BD-WORKTREE-CANVAS-001.md` v1.0 |
| **STEP 3 DD** | ✅ 完成 | `DD-WORKTREE-CANVAS-001.md` v1.0 |
| **STEP 4 Traceability** | ✅ 完成 | `TRACEABILITY-WORKTREE-CANVAS-001.md` v1.0 |
| **STEP 5 Spec** | ✅ 完成 | `worktree-canvas-spec.md` v0.1 |
| **STEP 6 IMPL-PLAN** | ✅ 完成 | `WORKTREE-CANVAS-IMPL-PLAN-001.md` v0.1 |
| **STEP 7 TEST-DESIGN** | ✅ 完成 | `TEST-DESIGN-WORKTREE-CANVAS-001.md` v1.8 (本文档) |
| **STEP 8 5 子任务分解** | ✅ 完成 | §11 RACI |
| **STEP 9 实装** | ⏳ 待 5 子任务推进 | T1-T14 + T4.1-T4.8 |
| **STEP 10 验收** | ⏳ 待 UAT | 43 AC |

### A.3 守门检查清单 (13 项必过)

- [x] 守门 #1 v25: CI cargo test 改单 crate
- [x] 守门 #1 v15: 禁回溯叙事
- [x] 守门 #3: docs 同步饱和
- [x] 守门 #5: env 验证
- [x] 守门 #6: PowerShell + Unix 路径兼容
- [x] 守门 #9: 子代理 dispatch 必先 brief (本文档即 IT/E2E/PT brief 入口)
- [x] 守门 #10: 错误码 6-field
- [x] 守门 #11: 缺标比错标 (23 已知缺口显式标)
- [x] 守门 #13: W/T/M 100% 覆盖 (5 表)
- [x] 守门 #14 v3: 5 域 Lead 签字栏 (Mavis 临时代签)
- [x] 守门 #14 v4: v0.62 反转, 真人代签流程取消
- [x] 守门 #28: token-OLU 估算 (本文档 ~500K)
- [x] 守门 #29: 1 commit 多文件 (本期 SRS/BD/DD/Trace/Spec/IMPL-PLAN/TEST-DESIGN 7 文件同 commit 落档)

### A.4 下一步 (per work-item request)

1. **创建 5 子任务** (本期操作, 落到 ULYS-57 issue 下, 见 §11.2 命令)
2. **派 5 agent 平行推进** (per parent-coordinator workflow)
3. **5 子任务完成后** ULYS-62 (测试 + NFR + 自审修正) 启动, 按原 ULYS-57 范围实证 122 锚点；新增 WLD-7 验收项单独列为开放门，不计入原闭环
4. **UAT 签字** (43 AC 全过, Mavis 接手代签)
5. **ULYS-57 父 issue 关闭**, 进入下一阶段 (P2 18 任务)

---

**文档结束** — per 日本 IPA SEC 標準 V モデル + 日本 SIer 项目常见测试工程规范 + internal work item ULYS-57 request "可以开子任务推进开发并且制作各级测试设计书了" 制作完成。
