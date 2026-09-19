# TRACEABILITY-WORKTREE-CANVAS-001

> **AI Worktree Graph Canvas — 需求追踪矩阵 v1.0** (per 日本 IPA SEC 標準 / 追踪マトリックス + Multica ULYS-57 issue 要求)
>
> - 上游文档:
>   - [`docs/requirements/SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.1 (**126 唯一 ID**: 103 FR + 23 NFR 子段)
>   - [`docs/design/BD-WORKTREE-CANVAS-001.md`](BD-WORKTREE-CANVAS-001.md) v1.1 (14 模块 + 15 决策点)
>   - [`docs/design/DD-WORKTREE-CANVAS-001.md`](DD-WORKTREE-CANVAS-001.md) v1.1 (52 段 + 100+ 代码示例)
> - 目的: 形成 Requirement ↔ BD ↔ DD ↔ Test 完整闭环, 验证需求无遗漏 / 设计无偏离 / 测试覆盖完整
> - 状态: 🟢 Draft v1.1 (2026-09-17, self-review C-01/C-03/C-04 修正)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v3)
> - 日期: 2026-09-15 JST

---

## §0 文档目的

本追踪矩阵按 Multica ULYS-57 issue 创建者发令 + §三十七 输出要求, 将 SRS-WORKTREE-CANVAS-001 v1.1 (**126 唯一 ID**: 103 FR + 23 NFR 子段, 去重后 21 唯一 NFR ID) 全部追踪到:

1. **Basic Design (BD)** 章节 — 验证 SRS 需求被 BD 设计覆盖, 不存在 "孤儿需求"
2. **Detailed Design (DD)** 章节 — 验证 BD 设计被 DD 实现覆盖, 不存在 "幽灵设计"
3. **Test Target** — 验证每项需求都有可测试的验收目标, 不存在 "黑盒需求"

**矩阵覆盖率目标**: 100% (**126/126 唯一 ID**, self-review C-01 v1.1)

**测试类型分类**:
- UT (Unit Test): Rust `cargo test` / TS `vitest`
- IT (Integration Test): Rust + DB 集成
- E2E (End-to-End Test): Playwright
- PT (Performance Test): vitest bench / cargo bench
- FI (Failure Injection Test): 故障注入
- ST (Security Test): RBAC + Audit

---

## §1 FR-WT 追踪矩阵 (38 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-WT-001 | Worktree 节点展示 | §1.1 + §3 + §9 | §1.1.2 + §29 | UT: WorktreeService.list / E2E: canvas renders | P0 |
| FR-WT-002 | Worktree 状态压缩 (7 态) | §20 + §22 | §23 + §2.1 (HumanState enum) | UT: state machine transition | P0 |
| FR-WT-003 | Worktree 卡片 12 字段 | §17 + §26 | §7.1 + §3.1 (WorktreeDto) | E2E: card displays 12 fields | P0 |
| FR-WT-004 | Worktree 状态机 | §20 + §22 | §23.1 (transition 函数) | UT: lifecycle_test.rs (BD §0.2 D-HEALTH-001) | P0 |
| FR-WT-005 | Ahead/Behind 3 维可视化 | §21 + §23.3 | §36 + §36.1 (visualDistance 公式) | UT: visualDistance formula / E2E: badge display | P0 |
| FR-WT-006 | Health Score 13 因素 | §7 + §21 | §34 (完整 13 因素实现) | UT: health_score_test.rs (3 cases) | P0 |
| FR-WT-007 | Worktree 颜色编码 | §24 + §25 | §29 + §30 | E2E: 7 state colors | P0 |
| FR-WT-008 | Worktree 图标 + 文本 | §24 + §26 | §29 + §1.1.2 (StatusBadge) | E2E: icon + text dual encoding | P0 |
| FR-WT-009 | Dirty 状态指示 | §17 (Worktree.dirty) | §7.1 + §10.1 (libgit2 status) | UT: status detection | P0 |
| FR-WT-010 | Last Activity | §17 (last_activity) | §7.1 + §3.1 (last_activity_relative) | UT: format_relative_time | P0 |
| FR-WT-011 | Test State | §17 (test_state) | §7.1 + §7.6 (TestRunNode) | E2E: test badge | P0 |
| FR-WT-012 | Risk Count | §17 (risk_count) | §7.1 + §13 (RiskEngine) | UT: risk count aggregation | P0 |
| FR-WT-013 | Lock/Unlock | §12.1.2 (Warning) | §18.1 (Lock/Unlock Action) + §1.1.1 | UT: lock/unlock + RBAC | P0 |
| FR-WT-014 | Archive | §12.1.2 (Warning) | §18.1 (Archive Action) + §1.1.1 | UT: archive + state update | P1 |
| FR-WT-015 | Mark Superseded | §12.1.2 (Warning) | §18.1 (MarkSuperseded) + §1.1.1 | UT: SUPERSEDES edge created | P2 |
| FR-WT-016 | Cleanup 批量 | §12.1.3 (Destructive) | §18.1 (Cleanup Action) + §1.1.1 | UT: batch delete + confirm | P1 |
| FR-WT-017 | Provenance 派生链 | §21 + §26 (History tab) | §1.1.1 (Inspector HistoryTab) + §8.1.3 (DerivedFromEdge) | E2E: Inspector History tab | P2 |
| FR-WT-018 | Merge Readiness | §21 + §22 | §18.2 (Merge validate) + §30.2 (时序图) | UT: validate blocks when not ready | P0 |
| FR-WT-019 | 视图模式切换 5 模式 | §25 + §26 | §1.1.2 (ViewModeSelector) + §30 (5 View 算法) | E2E: mode switch (per E2E §47) | P0 |
| FR-WT-020 | TREE VIEW | §25.1 | §1.1.2 (ViewModeSelector) + §30 | E2E: TREE view default | P0 |
| FR-WT-021 | DEPENDENCY VIEW | §25.1 | §1.1.2 + §30 | E2E: DEPENDS edges bold | P1 |
| FR-WT-022 | RISK VIEW | §25.1 | §1.1.2 + §30 | E2E: high-risk glowing | P1 |
| FR-WT-023 | AGENT VIEW | §25.1 + §8 | §1.1.2 + §30 + §15 (AgentBridge) | E2E: agent clustering | P1 |
| FR-WT-024 | HISTORY VIEW | §25.1 + §21 | §1.1.2 + §30 | E2E: merged Worktrees shown | P2 |
| FR-WT-025 | Merged 处理 | §25.1 + §26 | §1.1.2 (Recently Merged UI) + §30 | E2E: filter 24h/7d/All | P1 |
| FR-WT-026 | 双击跳详情 | §26 + §3 | §1.1.2 (Inspector) | E2E: double-click opens Inspector | P0 |
| FR-WT-027 | Health 趋势 | §21 + §26 | §14 (Health history API) + §1.1.2 (recharts) | E2E: trend line displayed | P2 |
| FR-WT-028 | Health 扣分明细 | §21 + §26 | §34 (deductions) + §1.1.2 (Inspector OverviewTab) | E2E: deductions expandable | P0 |
| FR-WT-029 | Compare 2 Worktree | §12.1.1 (Safe) + §22 | §18.1 (Compare Action) + §10 (libgit2 diff) | E2E: diff view shown | P0 |
| FR-WT-030 | Open in IDE | §12.1.1 (Safe) | §18.1 (OpenInIDE Action) + §1.1.2 | UT: vscode:// protocol | P1 |
| FR-WT-031 | Set Dependency | §12.1.2 (Warning) | §18.1 (SetDependency) + §8.1.10 (DependsOnEdge) | UT: Merge blocked by dep | P1 |
| FR-WT-032 | Remove Dependency | §12.1.2 (Warning) | §18.1 (RemoveDependency) + §1.1.1 | UT: edge deleted | P1 |
| FR-WT-033 | Sync Main | §12.1.2 (Warning) | §18.1 (SyncMain Action) + §10.1 (libgit2 rebase) | UT: SyncMain success / IT: real git | P0 |
| FR-WT-034 | Rebase | §12.1.2 (Warning) | §18.1 (Rebase) + §10.1 | UT: rebase + conflict handling | P1 |
| FR-WT-035 | Merge | §12.1.2 (Warning) + §30.2 (时序图) | §18.2 (Merge 完整实现) + §10.1 (libgit2 merge) | IT: E2E Create-Merge (DD §46.1) | P0 |
| FR-WT-036 | Create PR | §12.1.2 (Warning) | §18.1 (CreatePR Action) + §1.1.1 | IT: GitHub API mock | P0 |
| FR-WT-037 | Delete | §12.1.3 (Destructive) | §18.1 (Delete Action) + §10 (libgit2 worktree remove) | E2E: confirm dialog (per §47) | P0 |
| FR-WT-038 | Focus N-hop | §24 + §26 | §37 (Focus N-hop Algorithm) | E2E: focus highlights (per §47) | P0 |

**FR-WT 覆盖率**: 38/38 = 100%

---

## §2 FR-UI 追踪矩阵 (14 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-UI-001 | 无限画布 | §9 + §24 | §1.1.2 (CanvasView.tsx) + §29 + §30 | E2E: pan/zoom/select | P0 |
| FR-UI-002 | 搜索定位 | §26 | §1.1.2 (SearchBox) + §17 + §28 | E2E: search filters | P0 |
| FR-UI-003 | Minimap | §9 + §24 | §1.1.2 (CanvasMinimap) | E2E: minimap shown | P1 |
| FR-UI-004 | 自动居中 + Fit | §9 + §24 | §1.1.2 (Toolbar) + §30 (viewport) | E2E: Fit button works | P0 |
| FR-UI-005 | Focus UI | §24 | §37 + §1.1.2 (Toolbar Focus button) | E2E: opacity tiers | P0 |
| FR-UI-006 | 图层控制 | §24 | §1.1.2 (LayerPanel) + §29 (LOD) | E2E: toggle Node type | P1 |
| FR-UI-007 | 视图模式切换 UI | §25 | §1.1.2 (ViewModeSelector) + §30 | E2E: 5 modes switch | P0 |
| FR-UI-008 | 状态过滤 | §26 | §1.1.2 (Toolbar State Filter) + §28 (DSL parser) | E2E: state filter | P0 |
| FR-UI-009 | Edge 类型过滤 | §26 | §1.1.2 + §28 | E2E: edge filter | P1 |
| FR-UI-010 | 节点拖动 | §24 + §10.2 | §1.1.2 (CanvasView drag handler) + §31 (Incremental Layout) | E2E: drag updates position | P0 |
| FR-UI-011 | 节点折叠/展开 | §24 | §1.1.2 (ClusterNode) + §30 | E2E: cluster expand | P0 |
| FR-UI-012 | 多选 + 框选 | §24 | §1.1.2 (CanvasView selection) | E2E: multi-select + batch action | P1 |
| FR-UI-013 | 键盘快捷键 | §26 | §1.1.2 (KeyboardShortcuts) | E2E: V/H/+/-/1/2/3/F/Esc | P1 |
| FR-UI-014 | Inspector 11 tab | §26 + §3 | §1.1.2 (Inspector + InspectorTabs/* 11 文件) | E2E: 11 tabs accessible | P0 |

**FR-UI 覆盖率**: 14/14 = 100%

---

## §3 FR-GRAPH 追踪矩阵 (12 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-GRAPH-001 | Graph Repository Trait | §5.1 + §2.2 | §5 (完整 Trait + Neo4j 实现) | UT: repository_test.rs | P0 |
| FR-GRAPH-002 | 11 Node 类型 | §17-§18 + §7 | §2 + §7 (完整 11 struct) | UT: node enum coverage | P0 |
| FR-GRAPH-003 | 13 Edge 类型 | §18 + §8 | §8 (完整 13 struct) | UT: edge enum coverage | P0 |
| FR-GRAPH-004 | Worktree Node 字段 | §17 | §2.1 + §7.1 (完整字段) | UT: WorktreeNode struct | P0 |
| FR-GRAPH-005 | AgentSession Node 字段 | §8 | §2.3 + §7 (AgentSession 字段) | UT: AgentSessionNode struct | P0 |
| FR-GRAPH-006 | CONFLICTS_WITH Edge 字段 | §6.1 + §18 | §8.1.8 (ConflictsWithEdge 完整字段) | UT: ConflictsWithEdge fields | P0 |
| FR-GRAPH-007 | Cypher 子集 | §11 + §19 | §9.1 (Cypher DDL) + §9.2 (6 query examples) | UT: cypher builder | P0 |
| FR-GRAPH-008 | Graph Index | §5.4 + §19 | §9.1 (14 Index 创建) | UT: index creation | P0 |
| FR-GRAPH-009 | Graph Clustering | §10 | §30 (ClusterNode) + §1.1.2 | E2E: cluster merge | P1 |
| FR-GRAPH-010 | Lazy Expansion | §10 | §30 (isExpanded filter) | E2E: cluster expand | P1 |
| FR-GRAPH-011 | Graph Delta Update | §13 + §32 | §32 (Event handler 5 步骤) | UT: delta handler | P0 |
| FR-GRAPH-012 | N-hop Query | §24 | §37 (完整 Cypher + opacity) | UT: traverse_test.rs | P0 |

**FR-GRAPH 覆盖率**: 12/12 = 100%

---

## §4 FR-RISK 追踪矩阵 (11 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-RISK-001 | 11 Risk 类型 | §6.1 | §13 (RiskType 11 enum) | UT: risk_type enum | P0 |
| FR-RISK-002 | V1 Git Status | §6.2 + §13 | §1.1.1 (v1.rs) + §33 (使用 libgit2 status) | UT: v1_test.rs | P0 |
| FR-RISK-003 | V2 Diff Overlap | §6.2 + §13 | §1.1.1 (v2.rs) + §33 (line overlap) | UT: diff_overlap | P1 |
| FR-RISK-004 | V3 Symbol/AST | §6.2 + §13 | §1.1.1 (v3.rs tree-sitter) | UT: ast_parse | P2 |
| FR-RISK-005 | Conflict Prediction | §6.3 + §30.2 | §33 (完整 Rust 实现) | UT: predict_conflict (2 cases) / IT: E2E conflict (DD §46.2) | P0 |
| FR-RISK-006 | Risk Score 0-1 | §6.1 + §22 | §5 (RiskScore value object) + §33 (公式) | UT: score_test.rs | P0 |
| FR-RISK-007 | Risk 解释 | §6 + §11 | §13 (Risk.reason 字段) + §18 (LLM prompt) | UT: reason format | P0 |
| FR-RISK-008 | Risk 实时检测 | §13 + §26 | §32 (delta handler 触发) | UT: event-driven detection | P0 |
| FR-RISK-009 | Risk 解决跟踪 | §13 | §13 (resolve_risk 方法) | UT: resolve + audit | P1 |
| FR-RISK-010 | Risk 推荐 | §6 + §16 | §14 (Recommendation struct) + §16 (LLM) | UT: recommendation logic | P1 |
| FR-RISK-011 | Risk Audit Log | §6 + §35.4 | §43 (AuditWriter) + §14.1 (canvas_risk_event 表) | UT: audit_log_test.rs | P1 |

**FR-RISK 覆盖率**: 11/11 = 100%

---

## §5 FR-AGENT 追踪矩阵 (8 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-AGENT-001 | AgentRuntime Trait | §8.1 | §15 (完整 Trait + 4 适配器) | UT: runtime_test.rs | P0 |
| FR-AGENT-002 | Agent Node 显示 | §25.1 | §1.1.2 (AGENT VIEW) + §30 (View Mode) | E2E: AGENT mode | P1 |
| FR-AGENT-003 | Agent 状态实时同步 | §13 | §15 (subscribe_events) + §32 (SSE push) | UT: SSE event handler | P0 |
| FR-AGENT-004 | 多 Agent 同 Task | §8 + §25 | §15 (1:N list) + §1.1.2 | E2E: multi-agent UI | P2 |
| FR-AGENT-005 | Token Usage | §8 + §25 | §2.3 (TokenUsage struct) + §15 | E2E: token display | P2 |
| FR-AGENT-006 | Tool Calls 计数 | §8 + §25 | §2.3 + §15 | E2E: tool count | P2 |
| FR-AGENT-007 | Agent 历史轨迹 | §26 | §1.1.2 (AgentTab Timeline) + §15 (events) | E2E: timeline UI | P2 |
| FR-AGENT-008 | Result State | §8 + §25 | §2.3 (AgentResultState) + §15 | E2E: result badge | P1 |

**FR-AGENT 覆盖率**: 8/8 = 100%

---

## §6 FR-EXPLAIN 追踪矩阵 (4 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-EXPLAIN-001 | Explanation API | §16.1 + §30.3 (时序图) | §20.4 (Endpoint schema) + §17.2 (LLM client) | IT: explanation API | P0 |
| FR-EXPLAIN-002 | 来源限制 | §16.2 | §17.2 (NL Translator prompt) + §16 (constraint) | UT: prompt template | P0 |
| FR-EXPLAIN-003 | Tooltip | §16 + §24 | §1.1.2 (ExplanationTooltip) + §20.4 | E2E: hover tooltip | P0 |
| FR-EXPLAIN-004 | Cache | §15 + §16 | §24 (explanation_key) + §15 (L2 Redis) | PT: cache hit < 50ms (AC-P-5) | P1 |

**FR-EXPLAIN 覆盖率**: 4/4 = 100%

---

## §7 FR-SEARCH 追踪矩阵 (6 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-SEARCH-001 | Search DSL Parser | §11 + §26 | §17.1 (chevrotain parser) + §28 | UT: dsl_parser_test.rs (3 cases) | P0 |
| FR-SEARCH-002 | Graph Query 翻译 | §11 | §17 (cypher_builder) + §9.2 (Cypher examples) | UT: cypher_builder_test.rs | P0 |
| FR-SEARCH-003 | NL Query | §11 + §26 | §17.2 (LLM nl_to_dsl) + §28 | UT: NL prompt / E2E: NL search | P1 |
| FR-SEARCH-004 | 实时搜索高亮 | §24 | §1.1.2 (SearchBox handler) + §29 | E2E: highlight matches | P0 |
| FR-SEARCH-005 | 搜索历史 | §26 | §14.1 (canvas_search_history 表) + §1.1.2 | UT: history localStorage | P2 |
| FR-SEARCH-006 | 保存搜索 | §26 | §14.1 (canvas_saved_search 表) + §1.1.2 | E2E: save + reuse | P2 |

**FR-SEARCH 覆盖率**: 6/6 = 100%

---

## §8 FR-ACTION 追踪矩阵 (10 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| FR-ACTION-001 | ActionEngine Trait | §12.2 | §18.1 (register_all) + §1.1.1 | UT: engine_test.rs | P0 |
| FR-ACTION-002 | Action 3 分类 | §12.1 | §18.1 (register 18 Action by class) + §1.1.1 | UT: classification | P0 |
| FR-ACTION-003 | 二次确认 | §12.3 + §22 | §18.3 (ConfirmDialog.tsx) + §18.2 (Merge validate) | E2E: confirm dialog (per §47) | P0 |
| FR-ACTION-004 | RBAC | §34.1 | §44 (check_permission) + §18 (validator.rs) | UT: RBAC matrix / ST: 5 roles | P0 |
| FR-ACTION-005 | Audit Log | §35.4 + §43 | §43 (AuditWriter) + §14.1 (canvas_audit 表) | UT: audit_test.rs | P0 |
| FR-ACTION-006 | Idempotency Key | §12 + §42 | §42 (IdempotencyStore + PG UNIQUE) | UT: idempotency | P0 |
| FR-ACTION-007 | 撤销 | §22 + §12.1.2 | §18.1 (Undone flag on Warning Actions) + §1.1.1 | UT: undo | P1 |
| FR-ACTION-008 | 批量操作 | §22 | §18.1 (Cleanup Action) + §1.1.2 (multi-select) | E2E: batch | P1 |
| FR-ACTION-009 | 进度反馈 | §22 | §1.1.2 (ProgressBar) + §18 (long action) | E2E: progress shown | P0 |
| FR-ACTION-010 | 错误处理 + Retry | §31 | §38 (Error Code 27) + §39 (Retry Strategy 8 类型) | FI: retry on failure | P0 |

**FR-ACTION 覆盖率**: 10/10 = 100%

---

## §9 NFR 追踪矩阵 (23 项)

| Requirement | Requirement Summary | BD 章节 | DD 章节 | Test Target | Priority |
|---|---|---|---|---|---|
| NFR-PERF-001 | 100 Worktree 流畅 | §33.1 | §48 (vitest bench) | PT: bench/canvas-100.ts (AC-P-1) | P0 |
| NFR-PERF-002 | 1000 Worktree 可交互 | §33.1 | §48 + §30 (LOD + Cluster) | PT: bench/canvas-1000.ts (AC-P-2) | P1 |
| NFR-PERF-003 | Graph Query 响应 | §33 | §9.2 (Cypher with index) + §48 | PT: bench/graph-query.ts | P0 |
| NFR-PERF-004 | Risk Engine 增量 | §33 | §32 (delta handler) + §48 | PT: bench/risk-delta.ts | P0 |
| NFR-PERF-005 | AI Explanation 缓存 | §33 | §15 (L2 Cache) + §24 (explanation_key) | PT: bench/explain-cache.ts (AC-P-5) | P1 |
| NFR-SCALE-001 | 水平扩展 | §1 + §2 | §5 (Neo4j Causal Cluster) + §1.1.1 | UT: cluster config | P1 |
| NFR-SCALE-002 | Canvas 渲染引擎可替换 | §9 + §33 | §9 (CanvasRenderer Trait) + §30 | UT: trait abstraction | P1 |
| NFR-MAINT-001 | 模块解耦 | §2.2 + §1.3 | §1 (14 模块 + Trait 边界) + §2 (解耦规则) | UT: import graph | P0 |
| NFR-MAINT-002 | API 优先 | §1 + §27 | §20 (4 endpoint OpenAPI) + §1.1.1 | UT: OpenAPI validation | P0 |
| NFR-REL-001 | 故障恢复 | §31 + §4 | §49 (Neo4j Down 详) + §11 (GitObserver 重连) | FI: Neo4j / PG / Redis down | P0 |
| NFR-REL-002 | 事件合并 + 防抖 | §13.3 + §4 | §11.1 (EventDebouncer) + §13 | FI: 100 events/s | P0 |
| NFR-CON-001 | 事务边界 | §32 | §40 (7 事务边界) + §42 (Idempotency) | UT: transaction boundary | P0 |
| NFR-CACHE-001 | 多级缓存 | §15 | §15 (L1 moka + L2 Redis + L3 Neo4j) + §24 (Key 设计) | PT: cache hit > 80% | P0 |
| NFR-TEST-001 | 测试覆盖 | §36 | §45-§48 (Unit + IT + E2E + PT + FI) | UT: cargo tarpaulin + vitest coverage | P0 |
| NFR-OBS-001 | 可观测性 | §35 | §35 (10 Prometheus Metrics + OTel + tracing) | UT: metrics exported | P0 |
| NFR-A11Y-001 | 可访问性 | §34.3 | §1.1.2 (ARIA + Keyboard) | ST: axe-core + NVDA | P1 |
| NFR-PERM-001 | 权限模型 | §34.1 | §44 (RBAC 5 角色) + §18 (validator.rs) | ST: permission matrix | P0 |
| NFR-SEC-001 | 数据加密 | §34.2 | (部署期, AES-256 + TLS 1.3) | ST: encryption test | P0 |
| NFR-SEC-002 | Git 隔离 | §34.2 | §10 (security.rs path validation) | UT: security_test.rs (fuzzing) | P0 |
| NFR-AUDIT-001 | 审计日志 | §14.1 + §35.4 | §43 (AuditWriter + SCD Type 2 + S3 归档) | ST: audit integrity | P0 |
| NFR-I18N-001 | 国际化 | §34.4 | §1.1.2 (i18n.ts) + §26 (Inspector 字典) | UT: i18next test | P0 |
| NFR-THEME-001 | Dark/Light Theme | §34.5 | §1.1.2 (CSS variable) + §29 | E2E: theme switch | P0 |
| NFR-KEYB-001 | 键盘可操作 | §34 + §26 | §1.1.2 (KeyboardShortcuts) + §1.1.2 | E2E: keyboard-only | P0 |

**NFR 覆盖率**: 23/23 = 100%

---

## §10 决策点追踪矩阵 (15 项, BD §0.2)

| Decision | Recommended | Fallback | DD 实现 | Status |
|---|---|---|---|---|
| D-GRAPH-001 | Neo4j 5.x | Memgraph / Embedded / Custom | §5 (Neo4jRepository) | BD 拍板 ✅ |
| D-CANVAS-001 | React-Flow 11.x → PixiJS | 自研 WebGL | §9.2 (React-Flow) | BD 拍板 ✅ |
| D-GIT-001 | libgit2 | git CLI / Gitoxide | §10.1 (libgit2_provider) | BD 拍板 ✅ |
| D-AGENT-001 | Multica 内置 + 4 适配器 | 仅 Multica | §15 (4 适配器) | BD 拍板 ✅ |
| D-LLM-001 | Claude Sonnet | GPT-4o / Llama-3 | §17.2 (LlmClient) | BD 拍板 ✅ |
| D-CRDT-001 | Yjs | Automerge / LWW | (引用 SRS-CANVAS-AGENT-001 A12.6) | BD 拍板 ✅ |
| D-INDEX-001 | Composite + Range Index | 单字段 | §9.1 (14 Index) | BD 拍板 ✅ |
| D-CACHE-001 | L1 moka + L2 Redis + L3 Neo4j | 仅 L1/L2 | §15 | BD 拍板 ✅ |
| D-AUDIT-001 | PG 分区表 + S3 冷归档 | 仅 PG / ClickHouse | §14.1 + §43 | BD 拍板 ✅ |
| D-EVENT-001 | Redis Streams + SSE | Kafka / NATS / LISTEN | §13 + §19 | BD 拍板 ✅ |
| D-LAYOUT-001 | dagre + d3-force + ELK.js | 自研 Sugiyama | §16 + §31 | BD 拍板 ✅ |
| D-SEARCH-001 | 自研 + chevrotain | PEG.js | §17.1 (chevrotain) | BD 拍板 ✅ |
| D-RISK-001 | 0.7 / 0.4 阈值 | 0.5/0.8 | §5 (RiskScore) | BD 拍板 ✅ |
| D-HEALTH-001 | 加权扣分 13 因素 | NN / Decision Tree | §34 | BD 拍板 ✅ |
| D-COLOR-001 | D3 SchemeCategory10 + 色盲友好 | 自定义 | (前端, §1.1.2 color-encoding.ts) | BD 拍板 ✅ |

**决策点覆盖率**: 15/15 = 100%

---

## §11 验收标准 (AC) 追踪

| AC | 描述 | 测量方法 | DD 引用 | 状态 |
|---|---|---|---|---|
| AC-F-1 | Worktree Node 渲染正确 | 手动 / vitest | §1.1.2 + §29 | 设计 ✅ |
| AC-F-2 | 12 项卡片字段 | 手动 | §7.1 + §3.1 | 设计 ✅ |
| AC-F-3 | visualDistance = log(n+1) | 单元测试 | §36.1 | 设计 ✅ |
| AC-F-4 | Health Score 0-100 + 扣分明细 | 手动 + UT | §34 | 设计 ✅ |
| AC-F-5 | Risk Engine V1 11 Risk | 手动 + UT | §13 | 设计 ✅ |
| AC-F-6 | CONFLICTS_WITH Hover | 手动 | §33 | 设计 ✅ |
| AC-F-7 | 5 View Mode 切换 | 手动 + E2E | §30 | 设计 ✅ |
| AC-F-8 | 6 级 Semantic Zoom | 手动 | §29 | 设计 ✅ |
| AC-F-9 | Search DSL 组合 | 手动 + UT | §17.1 | 设计 ✅ |
| AC-F-10 | NL Query 翻译 | 手动 + E2E | §17.2 | 设计 ✅ |
| AC-F-11 | Focus Mode 1/2/3-hop | 手动 + E2E | §37 | 设计 ✅ |
| AC-F-12 | 18 Action | 手动 + E2E | §18.1 | 设计 ✅ |
| AC-F-13 | Destructive 二次确认 | 手动 + UT | §18.3 + §18.2 | 设计 ✅ |
| AC-F-14 | AI Explanation 不凭空生成 | 手动 + UT (LLM 约束) | §17.2 | 设计 ✅ |
| AC-F-15 | Inspector 11 tab | 手动 | §1.1.2 (11 文件) | 设计 ✅ |
| AC-F-16 | Merged 折叠 | 手动 | §30 + §1.1.2 | 设计 ✅ |
| AC-F-17 | Audit Log SCD Type 2 | UT | §43 + §14.1 | 设计 ✅ |
| AC-F-18 | Risk Resolved 跟踪 | UT | §13 | 设计 ✅ |
| AC-F-19 | 5 角色 RBAC | UT | §44 | 设计 ✅ |
| AC-F-20 | 多用户协同 | E2E | (引用 SRS-CANVAS-AGENT-001 A12) | 设计 ✅ |
| AC-P-1 | 100 Worktree 流畅 (FPS ≥ 30) | PT | §48 (vitest bench) | 设计 ✅ |
| AC-P-2 | 1000 Worktree 可交互 (FPS ≥ 15) | PT | §48 + §30 (LOD) | 设计 ✅ |
| AC-P-3 | Graph N-hop query P95 < 200ms | PT + k6 | §48 + §17 | 设计 ✅ |
| AC-P-4 | Risk Engine delta update P95 < 500ms | PT | §48 | 设计 ✅ |
| AC-P-5 | Explanation cache hit P95 < 50ms | PT | §15 (L2 Cache) | 设计 ✅ |
| AC-P-6 | Event Bus throughput ≥ 10k eps | PT + criterion-rs | §48 | 设计 ✅ |
| AC-P-7 | Health Compute P95 < 50ms / WT | PT + criterion-rs | §48 | 设计 ✅ |
| AC-Q-1 | W/T/M 100% 覆盖 (5 表) | 落档检查 + 评审 | §1.1 + §36 | 设计 ✅ |
| AC-Q-2 | 错误码 6-field | UT | §6 + §38 | 设计 ✅ |
| AC-Q-3 | 守门 #14 v3 Mavis 接手代签 (5 域真人到位后切真人) | 落档检查 | AGENTS.md | 设计 ✅ |
| AC-Q-4 | 守门 #6 v2 6-field 错误码 | UT | §38 | 设计 ✅ |
| AC-Q-5 | 守门 #11 缺标比错标 | 落档检查 | §36 | 设计 ✅ |
| AC-Q-6 | 守门 #28 术语一致 | 落档检查 | §1 | 设计 ✅ |
| AC-Q-7 | 守门 #DB-13 + I18N | 落档检查 | §44 | 设计 ✅ |
| AC-Q-8 | 守门 #14 + A11Y + 键盘 | 评审 + axe-core | §34 + §30 | 设计 ✅ |
| AC-Q-9 | 守门 #1 + OTel + Grafana | 实证 | §35 | 设计 ✅ |
| AC-Q-10 | Audit WORM 物理删除禁止 | UT | §44 + §14 | 设计 ✅ |
| AC-D-1 | SRS 落档 (15 段 + 2 附录) | 落档检查 | (本追踪矩阵 §1-§8) | 落档 ✅ |
| AC-D-2 | BD 落档 (39 段 + 1 附录) | 落档检查 | (本追踪矩阵 §3) | 落档 ✅ |
| AC-D-3 | DD 落档 (52 段 + 1 附录) | 落档检查 | (本追踪矩阵 §4) | 落档 ✅ |
| AC-D-4 | Traceability 落档 (15 段 + 1 附录) | 落档检查 | (本追踪矩阵 §A) | 落档 ✅ |
| AC-D-5 | Spec + IMPL-PLAN 落档 | 落档检查 | IMPL-PLAN §1-§6 | 落档 ✅ |
| AC-D-6 | TEST-DESIGN 落档 (122 实证锚点) | 落档检查 | TEST-DESIGN §2-§6 | 落档 ✅ |

**AC 覆盖率**: **43/43 = 100%** (AC-F 20 + AC-P 7 + AC-Q 10 + AC-D 6, per self-review C-03 v1.1 2026-09-17 JST 全表化展开)

---

## §12 闭环验证

### 12.1 需求 → 设计 → 实现 → 测试 闭环

```
需求 (SRS FR-* / NFR-*)
  ↓ BD 章节 (设计)
  ↓ DD 章节 (实现)
  ↓ Test Target (UT/IT/E2E/PT/FI/ST)
  ↓ 实装代码 (crates/* + frontend/src/*)
  ↓ 测试通过 (cargo test + vitest + Playwright)
  ↓ 验收 (AC-F/P/Q/D 43 项)
```

### 12.2 闭环验证矩阵

| 维度 | SRS | BD | DD | Test | 闭环状态 |
|---|---|---|---|---|---|
| 概念定义 | **126 唯一 ID** | 14 模块 | 100+ 代码示例 | 43 AC | ✅ 闭环 |
| Node 11 类型 | §七 | §17 | §7 (11 struct) | UT | ✅ |
| Edge 13 类型 | §八 | §18 | §8 (13 struct) | UT | ✅ |
| Human State 7 | §九 | §20 | §23 (transition) | UT | ✅ |
| Risk 11 | §十三 | §6 | §13 + §33 | UT | ✅ |
| Health 13 因素 | §十二 | §7 | §34 | UT | ✅ |
| Action 18 | §二十二 | §12 | §18.1 (18 文件) | E2E | ✅ |
| Event 15 | §二十五 | §13 | §19 | UT | ✅ |
| View 5 | §十八 | §25 | §30 | E2E | ✅ |
| Zoom 6 | §六 | §23 | §29 | E2E | ✅ |

### 12.3 关键算法闭环验证

| 算法 | SRS 来源 | BD 设计 | DD 实现 | 测试 |
|---|---|---|---|---|
| Conflict Prediction | §十四 | §6.3 | §33 (完整 Rust) | UT + IT §46.2 |
| Health Score | §十二 | §7.2 | §34 (13 因素) | UT §45.1 |
| Risk Score | §十四 | §6.1 | §5 (RiskScore VO) + §33 | UT §45.2 |
| Ahead/Behind 可视化 | §十一 | §23.3 | §36.1 (visualDistance) | UT |
| Focus N-hop | §十七 | §24 | §37 | E2E §47 |
| Search DSL Parse | §二十 | §11 | §17.1 (chevrotain) | UT §45.3 |
| Layout | §五 | §10 | §16 + §31 | PT |

---

## §13 缺口与未覆盖项 (per SRS §十三 + DD §A.6)

### 13.1 设计未完成但 MVP 不阻塞

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | 17 个 Action (除 Merge) 仅列出文件结构 + register, 无完整代码 | 实装期补全 | 实装期 ~2 周工作量 |
| 2 | CLI Git Provider 仅列出 trait 框架, 无完整实现 | libgit2 已 100% 覆盖, CLI 仅 fallback | 触发 fallback 时补全 |
| 3 | Risk Engine V3 (tree-sitter) 仅注册 | MVP 只到 V2 (Diff Overlap) | V1.2 P2 实装 |
| 4 | LLM prompt 模板 (cache 模板降级) 未详写 | Claude Sonnet 在线 | 实装期补全模板 |
| 5 | Saved Search UI 仅列出 | BD Minor #2 | MVP P2, 实装期补 |

### 13.2 性能未实测项

| # | 项 | 影响 | 后续 |
|---|---|---|---|
| 1 | 1000 Worktree 实际性能 | 可能需要切换 PixiJS | P1.5 实测, 不达预期切换 |
| 2 | LLM 调用成本 (1000+ Worktree) | Cache 已缓解 | 实装期按真实数据校准 |

### 13.3 已完整闭环

- ✅ 11 Node + 13 Edge + 18 Action + 15 Event + 7 State + 5 View + 6 Zoom + 11 Risk + 13 Health 因素 100% 设计
- ✅ 14 模块 + 4 大 Trait + 15 决策点 100% 设计
- ✅ 43 AC 100% 映射到 DD 测试目标
- ✅ 完整需求追踪矩阵 **126/126 唯一 ID** = 100% (v1.1 self-review C-01 修正)

---

## §14 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-15 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-15 | 5 域真人 Lead 到位前 Mavis 临时代签 |

---

## §15 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v1.0 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 15 段 (目的/FR-WT/FR-UI/FR-GRAPH/FR-RISK/FR-AGENT/FR-EXPLAIN/FR-SEARCH/FR-ACTION/NFR/决策点/AC/闭环/缺口/签字/修订), 追踪矩阵 115/115 = 100% 覆盖, 43 AC 映射到 DD 测试目标, 14 模块 + 15 决策点 + 7 闭环验证 | 2026-09-15 Multica ULYS-57 issue 创建者发令 |
| v1.1 | Ulysses — Mavis 接手 (per 守门 #14 v3, self-review C-01/C-03 修正) | 修正: 总数 115 → 126 唯一 ID (103 FR + 23 NFR 子段); §11 AC 43 项全表化展开 (AC-F 20 + AC-P 7 + AC-Q 10 + AC-D 6); 覆盖率声明同步; 上游文档版本号 v1.0 → v1.1 | 2026-09-17 ULYS-62 self-review 修正落地 |
| v1.2 | Ulysses — Mavis 接手 (per 守门 #14 v3 + self-review 整体审查 m-7 派生) | 修正: "Mavis 永久代签" → "Mavis 接手代签 (5 域真人到位后切真人)" (AC-Q-3 / 守门 #10 / 守门 #14 v3 行, per self-review m-7) | 2026-09-19 04:55 JST 自审整体审查 + 9/18 23:14 JST 评论者发令 "没动的也都处理到位" |

---

## 附录 A: 总结

### A.1 文档集完整性

| 文档 | 版本 | 大小 | 章节数 | 状态 |
|---|---|---|---|---|
| `SRS-WORKTREE-CANVAS-001.md` | v1.0 | ~105KB | 15 段 + 2 附录 | 🟢 Baseline |
| `BD-WORKTREE-CANVAS-001.md` | v1.0 | ~83KB | 39 段 + 1 附录 | 🟢 Draft |
| `DD-WORKTREE-CANVAS-001.md` | v1.0 | ~120KB | 52 段 + 1 附录 | 🟢 Draft |
| `TRACEABILITY-WORKTREE-CANVAS-001.md` | v1.1 | ~28KB | 15 段 + 1 附录 | 🟢 Draft |

**总文档大小**: ~336KB
**总章节数**: 121 段 + 5 附录
**总追踪项**: **126 唯一 ID** 需求 (103 FR + 23 NFR 子段) + 15 决策点 + 43 AC = **184 项追踪** (v1.1 self-review C-01 修正)

### A.2 Multica ULYS-57 任务完成度

| STEP | 任务 | 输出 | 状态 |
|---|---|---|---|
| 1 | 《AI Worktree Graph Canvas 需求文档》 | SRS-WORKTREE-CANVAS-001.md v1.0 | ✅ |
| 1.5 | SELF REVIEW | 附录 B (0 Critical / 0 Major / 3 Minor) | ✅ |
| 2 | 《AI Worktree Graph Canvas 基本设计》 | BD-WORKTREE-CANVAS-001.md v1.0 | ✅ |
| 2.5 | SELF REVIEW | 附录 A (0 Critical / 0 Major / 2 Minor) | ✅ |
| 3 | 《AI Worktree Graph Canvas 详细设计》 | DD-WORKTREE-CANVAS-001.md v1.0 | ✅ |
| 3.5 | SELF REVIEW | 附录 A (0 Critical / 0 Major / 2 Minor) | ✅ |
| 4 | 《需求—基本设计—详细设计追踪矩阵》 | TRACEABILITY-WORKTREE-CANVAS-001.md v1.0 | ✅ |

**最终闭环验证**: 需求 → 架构 → 模块 → API → 数据结构 → UI → 算法 → 测试 完整闭环 ✅

### A.3 守门检查清单

| 守门 | 描述 | 验证位置 |
|---|---|---|
| 守门 #1 v15 | docs 同步饱和 + 1 commit 多文件 | 4 份文档同期落档 ✅ |
| 守门 #1 v25 | cargo test --workspace -j 4 | (实施期) |
| 守门 #3 | 7 段结构 | SRS/BD/DD 都含 §0 目的 + §1 改动矩阵 + §3 已知缺口 ✅ |
| 守门 #5 | 5 角色签字栏 | 每份文档含 5 角色签字 ✅ |
| 守门 #6 | 决策点明确 | BD §0.2 列出 15 决策点 + 推荐方案 ✅ |
| 守门 #9 | 守门规则 (15-17 项) | SRS/BD/DD 守门基线明确 ✅ |
| 守门 #10 | Mavis 接手代签 (5 域真人到位后切真人) | 修订人 + 审批者按 DEC-008 + 守门 #14 v3 ✅ |
| 守门 #11 | 报告 7 段结构 | (实施期, Phase Report) |
| 守门 #13 | 100% RLS + W/T/M | (实施期) |
| 守门 #14 v3 | Mavis 接手代签 (5 域真人到位后切真人) | (审批者行) ✅ |
| 守门 #14 v4 | 反转 v0.62 | 2026-09-10 12:45 JST ✅ |
| 守门 #22 | 守门基线 | (实装期) |
| 守门 #28 | 术语一致 | SRS/BD/DD 三阶段术语 100% 一致 ✅ |
| 守门 #29 | 守门规则 | SRS/BD/DD 守门 13-15 项明确 ✅ |

**守门检查**: 11/14 文档期可验证 ✅, 3 项 (实施期 + 报告期) 后续验证