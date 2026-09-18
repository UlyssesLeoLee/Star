# worktree-canvas 实施 spec (Implementation Spec)

> **AI Worktree Graph Canvas — 无限画布模块 实施 spec v0.1**
>
> - 状态: 🟡 Draft v0.1 (2026-09-15, per 日本 IPA SEC 標準 / domain-spec テンプレート + STAR 仓 spec 派生模板)
> - 触发: 2026-09-15 21:21 JST Multica ULYS-57 评论者发令 "制作 spec 和实施计划"
> - 受众: 実装エンジニア / テストエンジニア / アーキテクト / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 守门 #14 v3 + v0.62 反转)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 上游依赖:
>   - [`docs/requirements/SRS-WORKTREE-CANVAS-001.md`](../requirements/SRS-WORKTREE-CANVAS-001.md) v1.1 (126 唯一 ID: FR-WT 38 + FR-UI 14 + FR-GRAPH 12 + FR-RISK 11 + FR-AGENT 8 + FR-EXPLAIN 4 + FR-SEARCH 6 + FR-ACTION 10 + NFR 21 子段去重 ID)
>   - [`docs/design/BD-WORKTREE-CANVAS-001.md`](../design/BD-WORKTREE-CANVAS-001.md) v1.0 (14 模块 + 4 大 Trait + 15 决策点全部已拍板)
>   - [`docs/design/DD-WORKTREE-CANVAS-001.md`](../design/DD-WORKTREE-CANVAS-001.md) v1.0 (52 段 + 100+ 代码示例 + 14 crate + 30+ TS 文件)
>   - [`docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md`](../design/TRACEABILITY-WORKTREE-CANVAS-001.md) v1.0 (103 FR + 23 NFR 子段 = 126 唯一 ID, 15 决策点, 43 AC)
> - 下游交付:
>   - 实施计划 [`docs/implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md`](../implementation-plans/WORKTREE-CANVAS-IMPL-PLAN-001.md) v0.1 (本 spec 同期落档, per 评论者发令)
>   - 14 新 Rust crate 实装代码 (`crates/worktree-canvas/`, `crates/graph-core/`, `crates/git-observer/`, `crates/git-adapter/`, `crates/worktree-service/`, `crates/relationship-engine/`, `crates/risk-engine/`, `crates/health-engine/`, `crates/agent-bridge/`, `crates/canvas-renderer/`, `crates/layout-engine/`, `crates/query-engine/`, `crates/action-engine/`, `crates/event-bus/`)
>   - 1 新 BFF module (`bff/src/worktree_canvas/`, 共 11 REST + 15 SSE + 1 WS)
>   - 30+ 新前端 TS/TSX 文件 (`frontend/src/app/(worktree-canvas)/` + `frontend/src/components/worktree-canvas/`, 实际 45 文件 = 5 pages + 40 components, 超实 50%)
> - 平行参考: [`docs/specs/domain-worktree-spec.md`](domain-worktree-spec.md) v0.1 (17 状态机 + 9 项隔离检查 + 7 状态投影) + `docs/frontend-canvas-design.md` v0.1 (V0.1 MVP 实装基线) + `SRS-AGENT-VIEW-001.md` v1.0 + `SRS-CANVAS-AGENT-001.md` v1.2 (A12 多人编辑消费源)

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | worktree-canvas-spec |
| 文书名 | AI Worktree Graph Canvas — 无限画布 实施 spec |
| 版本 | v0.1 (初版) |
| 作成日 | 2026-09-15 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 (per 守门 #14 v4 v0.62 反转) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 commit | 待生成 (root 统一 commit, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件) |
| 关联文档 | SRS-WORKTREE-CANVAS-001.md v1.1 (126 唯一 ID) + BD-WORKTREE-CANVAS-001.md v1.1 (14 模块 + 15 决策点) + DD-WORKTREE-CANVAS-001.md v1.1 (52 段) + TRACEABILITY-WORKTREE-CANVAS-001.md v1.1 (126 ID 追踪) + WORKTREE-CANVAS-IMPL-PLAN-001.md v0.1 (本 spec 同期落档) |
| 范围 | worktree-canvas 模块实装的所有接口签名 + 关键算法 + 不变量 + 错误码 + 测试入口; 不重写 SRS/BD/DD, 仅继承并落地为可实施 spec |
| 守门基线 | 守门 #1+#1 v25+#3+#5+#6+#9+#10+#11+#13+#14 v3+#14 v4+#28+#29 共 13 项必过 |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-15 21:21 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v4 v0.62 反转)** | **初版落档, 12 段 IPA SEC spec 模板, 9 职责边界 + 11 实体 + 12 不变量 + 11 接口签名 (Rust) + 11 接口签名 (TS) + 18 Action 签名 + 15 Event 签名 + 8 错误码 + 8 算法入口 + 10 测试入口 + 14 crate 落地清单 + 30+ TS 文件落地清单 + 13 已知缺口** | **2026-09-15 21:21 JST 评论者发令 "制作 spec 和实施计划" (per Multica ULYS-57 thread 01a0a504-3517-7299-b3bb-30c4209911e8)** |
| **v0.2** | **2026-09-19 JST** | **Ulysses — Mavis 接手审核 (per 守门 #14 v3 + self-review 整体审查 m-2 派生)** | **修正: SRS 版本 v1.0 → v1.1 + 126 唯一 ID 口径同步 (§0.1 §0.2 关联文档); BD/DD/Trace 版本 v1.0 → v1.1 (口径同步); 前端 30 TS → 30+ TS 文件 (实际 45 = 5 pages + 40 components, per m-3)** | **2026-09-19 04:55 JST 自审整体审查 + 9/18 23:14 JST 评论者发令 "没动的也都处理到位"** |

### 0.3 撤回记录 (per 守门 #1 禁回溯叙事)

本 spec 为初版落档, 暂无撤回记录。未来如有重要撤回需在此显式记录。

---

## §1 职责与边界

`worktree-canvas` 是 STAR / Multica 平台的 **AI Worktree 无限画布管理模块**, 把 AI Agent 并行产生的 10-1000 个 Worktree 以"无限画布 + 树形拓扑 + 图结构 + 风险分析 + AI 解释"的方式呈献给人类, 帮助人类以低认知负荷监督 AI Worktree 世界。

### 1.1 属于本 spec 的

- 11 类 Node 的 **Domain Model 落地字段** (Repository / Mainline / Worktree / Task / AgentSession / Branch / Commit / File / Symbol / TestRun / PullRequest / Issue, per DD §7 + BD §17)
- 13 类 Edge 的 **详细字段 + 关系约束** (BASED_ON / USES_BRANCH / DERIVED_FROM / WORKS_ON / IMPLEMENTED_IN / MODIFIES / MODIFIES_SYMBOL / CONFLICTS_WITH / OVERLAPS_WITH / DEPENDS_ON / BLOCKS / SUPERSEDES / MERGED_INTO, per DD §8)
- 7 类 Worktree 人类态 (RUNNING / WAITING / READY / DIVERGED / CONFLICT / MERGED / STALE) + 11 Risk 类型 + 13 Health Score 因素
- 4 大抽象 Trait 接口签名 (GraphRepository / CanvasRenderer / GitProvider / AgentRuntime, per BD §0.2 D-GRAPH-001 / D-CANVAS-001 / D-GIT-001 / D-AGENT-001)
- 14 模块的 Rust crate 接口签名 (per DD §10-§18)
- 11 REST + 15 SSE + 1 WS API 的 Request/Response schema 关键字段
- 18 Action 的 trait 签名 + 危险级别 + 二次确认机制
- 15 Event 的 Payload schema
- 8 关键算法的入口签名 (Conflict Prediction / Health Score / Risk Score / Ahead-Behind / Focus N-hop / visualDistance / Layout / Search DSL Parser)
- 43 AC 的 UT/IT/E2E/PT 入口 (per TRACEABILITY §11)
- 5 View Mode + 6 Semantic Zoom + 7 Human State 状态机 + Inspector 11 tab 的 TS 接口

### 1.2 不属于本 spec 的 (per SRS §1.4 派生)

- **Git 操作底层执行** — 依赖 libgit2 / git CLI / Gitoxide (复用现有 crate, 本 spec 仅消费 `GitProvider` Trait)
- **AI Agent 执行引擎** — 复用 `SRS-STAR-AGENT-RUNTIME-001.md` 的 `domain-agent` + `domain-agent-runtime`, 本 spec 仅消费 `AgentRuntime` Trait (per BD §0.2 D-AGENT-001)
- **图数据库引擎内部** — 抽象为 `GraphRepository` Trait, Neo4j 5.x 是 BD 拍板的推荐方案, 但 trait 不绑定具体实现 (per BD §0.2 D-GRAPH-001)
- **Canvas 渲染引擎内部** — 抽象为 `CanvasRenderer` Trait, MVP 用 React-Flow 11.x, 1000+ WT 切换 PixiJS (per BD §0.2 D-CANVAS-001)
- **CRDT 协同算法内部** — 选型留 P2 (per BD §0.2 D-CRDT-001, Yjs 拍板但实装 P2)
- **L5 Symbol 级 AST 分析** — V3 Risk Engine 备选, MVP 只到 V2 (Diff Overlap, per SRS §3 Risk V1-V3)
- **多用户实时协同细节** — 消费 `SRS-CANVAS-AGENT-001.md` v1.2 A12, 不重写多人编辑 (per SRS §1.4)
- **LLM 调用实现** — 仅消费 `LLMClient` Trait (per DD §16), Claude Sonnet 是 BD 拍板 (per BD §0.2 D-LLM-001)

### 1.3 跨模块边界 (per BD §2)

**上游模块 (本 spec 消费)**:
- `domain-scm` — Repository / Branch / Commit / Ref (per SRS §23 Git Source of Truth)
- `domain-worktree` — 17 状态 Worktree 实体 + WorktreeStatusObserved Projection (per `domain-worktree-spec.md` v0.1)
- `star-context` — ActorContext (租户隔离 + 5 角色 RBAC + DeviceId 强类型, per AGENTS.md §4 v18 H2-EXT)
- `domain-agent` / `domain-agent-runtime` — AgentSession 14 状态 + Tool Calls + Token Usage
- `domain-task` — Task 6 状态 + IMPLEMENTED_IN Worktree 引用
- `domain-audit` — Audit Log 持久化 (PG 分区表 + S3 冷归档, per BD §14)
- `domain-notification` — Risk / Health 变化通知
- `star-cache` — L1 moka + L2 Redis (per BD §15)

**下游模块 (本 spec 被消费)**:
- `domain-collaboration` — Realtime 推送 (SSE / WebSocket)
- `star-canvas` (V0.1 已实装) — `/worktree-canvas` 页面挂载点
- `frontend` (Next.js 14.2.5) — React-Flow 11.x 渲染 + Zustand 状态
- `bff` — 11 REST + 15 SSE + 1 WS 端点代理

---

## §2 关键实体 (per DD §2 Domain Model + §7 Node 详细字段)

引用 DD §7 Node 详细字段 (3705 行, 12 类 Node 字段定义)。本 spec 提炼 11 类 Node 的 **必含字段** (Rust struct 形态)。

### 2.1 核心聚合根: Worktree Node

```rust
// crates/worktree-canvas/src/node/worktree.rs

/// 11 类 Node 中第 1 类: Worktree (核心聚合根)
/// 字段数: 18 (含 3 引用 + 7 状态 + 8 metric), per DD §7.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeNode {
    // === 标识 (3) ===
    pub id: WorktreeId,                     // ULID
    pub repository_id: RepositoryId,
    pub branch: BranchName,                 // 引用 domain-scm Branch

    // === 人类态 (1, 7 状态机) ===
    pub human_state: WorktreeHumanState,    // RUNNING/WAITING/READY/DIVERGED/CONFLICT/MERGED/STALE

    // === Metric (8, per Health Score 13 因素聚合) ===
    pub ahead: u32,                         // 与 main 的 ahead commit 数
    pub behind: u32,                        // 与 main 的 behind commit 数
    pub dirty_state: DirtyState,            // Clean / Modified / Staged / Mixed
    pub test_state: TestState,              // NotRun / Running / Pass / Fail / Error
    pub build_state: BuildState,            // NotBuilt / Building / Pass / Fail
    pub review_state: ReviewState,          // None / Pending / Approved / ChangesRequested
    pub agent_session_id: Option<AgentSessionId>,
    pub task_id: Option<TaskId>,
    pub health_score: HealthScore,          // 0-100, per DD §34
    pub last_activity_at: DateTime<Utc>,

    // === Provenance (3) ===
    pub derived_from: Vec<WorktreeId>,      // DERIVED_FROM Edge 起点
    pub depends_on: Vec<WorktreeId>,        // DEPENDS_ON Edge 起点
    pub superseded_by: Option<WorktreeId>,  // SUPERSEDES Edge 终点
}
```

**8 个次要 Node (Repository / Mainline / Task / AgentSession / Branch / Commit / File / Symbol)** 的字段定义引用 DD §7.2-§7.11, 本 spec 不重复 (per 守门 #1 禁回溯叙事)。

### 2.2 13 类 Edge 字段 (per DD §8)

```rust
// crates/graph-core/src/edge.rs

/// 13 类 Edge 统一基类 (per DD §8.1, 字段 6 + EdgeKind 13 enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub kind: EdgeKind,                     // 13 种 (BASED_ON / USES_BRANCH / ...)
    pub source: NodeId,                     // ULID
    pub target: NodeId,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: DateTime<Utc>,
    pub metadata: EdgeMetadata,             // JSON value, 不同 Edge 不同字段
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EdgeKind {
    BasedOn,        // (:Worktree)-[:BASED_ON]->(:Commit)
    UsesBranch,     // (:Worktree)-[:USES_BRANCH]->(:Branch)
    DerivedFrom,    // (:Worktree)-[:DERIVED_FROM]->(:Worktree)
    WorksOn,        // (:AgentSession)-[:WORKS_ON]->(:Worktree)
    ImplementedIn,  // (:Task)-[:IMPLEMENTED_IN]->(:Worktree)
    Modifies,       // (:Worktree)-[:MODIFIES]->(:File)
    ModifiesSymbol, // (:Worktree)-[:MODIFIES_SYMBOL]->(:Symbol)
    ConflictsWith,  // (:Worktree)-[:CONFLICTS_WITH]->(:Worktree)
    OverlapsWith,   // (:Worktree)-[:OVERLAPS_WITH]->(:Worktree)
    DependsOn,      // (:Worktree)-[:DEPENDS_ON]->(:Worktree)
    Blocks,         // (:Worktree)-[:BLOCKS]->(:Worktree)
    Supersedes,     // (:Worktree)-[:SUPERSEDES]->(:Worktree)
    MergedInto,     // (:Worktree)-[:MERGED_INTO]->(:Branch)
}

// 4 个高风险 Edge 的 metadata 必含字段 (per SRS §14)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEdgeMetadata {
    pub risk_score: f64,                    // 0.0-1.0
    pub shared_files: Vec<String>,
    pub shared_symbols: Vec<SymbolRef>,
    pub detected_at: DateTime<Utc>,
    pub reason: String,
    pub confidence: f64,                    // 0.0-1.0
    pub risk_version: RiskVersion,          // V1 / V2 / V3 (per SRS §13)
}
```

### 2.3 关键 Value Object (per DD §5)

```rust
// crates/common/src/value.rs

/// Worktree 7 状态机 (per SRS §9 + BD §20)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorktreeHumanState {
    Running,    // AgentSession 活跃
    Waiting,    // 等待外部反馈 (CI / Review / Input)
    Ready,      // Health >= 80 + Test Pass + 无 Conflict, 可 Merge
    Diverged,   // 落后 main > 20 commits (阈值可调, per Health 因素)
    Conflict,   // CONFLICTS_WITH 边存在 + risk_score >= 0.7
    Merged,     // 已 MERGED_INTO main (但保留历史 Provenance, per SRS §19)
    Stale,      // last_activity_at > 24h, 无 AgentSession 活跃
}

/// Health Score (per DD §34, 13 因素加权扣分, 范围 0-100)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HealthScore(pub u8);  // 0-100, 下限 0

/// Risk Score (per DD §35, 范围 0.0-1.0, 阈值 0.7 = 高 / 0.4-0.7 = 中 / <0.4 = 低)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskScore(pub f64);

/// Visual Distance (per SRS §11 + DD §36, log 非线性公式)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VisualDistance(pub f32);
// 计算: visualDistance = log(commitDistance + 1.0) * 60.0 (px), per DD §36
```

---

## §3 关键不变量 (per SRS §9 + BD §20 + DD §23)

| ID | 不变量 | 上游依据 | 验证入口 |
|---|---|---|---|
| **INV-WC-01** | **7 Human State 严格状态机**: 7 状态迁移必须有 from→to 表规定, 非法迁移返回 `WC-STT-001` | SRS §9 + BD §20 + DD §23 | `crates/worktree-service/src/lifecycle.rs::transition()` |
| **INV-WC-02** | **Health Score ∈ [0, 100]**: 加权扣分制, 13 因素 (Merge Conflict / Behind Main / Ahead / Dirty / Test / Build / Review / Agent 状态 / 活跃时间 / 文件重叠 / Symbol 重叠 / 依赖未完成 / 阻塞关系), 下限 0 | SRS §12 + DD §34 | `crates/health-engine/src/score.rs::compute()` |
| **INV-WC-03** | **Risk Score ∈ [0.0, 1.0]**: V1 Git Status / V2 Diff Overlap / V3 Symbol Overlap 三层累加 (per BD §6) | SRS §13 + DD §35 | `crates/risk-engine/src/score.rs::compute()` |
| **INV-WC-04** | **13 类 Edge 严格分类**: Graph Schema 必须用 enum 限定 13 种 kind, 非法 kind 返回 `WC-EDGE-001` | SRS §八 + DD §8 | `crates/graph-core/src/edge.rs::validate_kind()` |
| **INV-WC-05** | **Worktree-first 视觉**: 默认 UI 仅显示 5 类 Node (Repository / Mainline / Worktree / Task / AgentSession), 其他 6 类 (Branch / Commit / File / Symbol / TestRun / Issue) 默认折叠 | SRS §7 + R2 原则 | `frontend/src/components/worktree-canvas/NodeRenderer.tsx::shouldRender()` |
| **INV-WC-06** | **Semantic Zoom L0-L5**: zoom 阈值决定显示粒度, 不可越级显示 (e.g. 远距离不可显示 File 节点) | SRS §六 + DD §29 | `frontend/src/components/worktree-canvas/ZoomManager.ts::resolveZoomLevel()` |
| **INV-WC-07** | **Exception-first 视觉**: Conflict / Stale / Diverged / Health < 50 节点高亮, 正常节点透明度 60% | SRS R7 原则 | `frontend/src/components/worktree-canvas/NodeStyle.ts::applyExceptionHighlight()` |
| **INV-WC-08** | **Provenance 永久保留**: Merged Worktree 不允许物理删除 (per SRS §19), 必须折叠到 Recently Merged + 保留 DERIVED_FROM / SUPERSEDES 历史 | SRS §19 + R9 原则 | `crates/worktree-service/src/cleanup.rs::archive()` |
| **INV-WC-09** | **5 角色 RBAC**: Worktree 操作必走 `AuthorizationChecker`, 5 角色 (tenant_admin / project_admin / developer / viewer / agent), agent 角色仅允许 Safe Action | NFR-PERM-001 + DD §44 | `crates/action-engine/src/validator.rs::check_permission()` |
| **INV-WC-10** | **LLM 不修改 Source of Truth**: AI Explanation Layer 仅读 Graph + Git + Risk 数据, 不允许写任何 Node / Edge, 违规返回 `WC-AI-001` | SRS §十六 + BR-11 + R10 原则 | `crates/agent-bridge/src/explainer.rs::explain()` |
| **INV-WC-11** | **Idempotency Key TTL = 24h**: 同一 Action 在 24h 内同事务/跨事务重放返回缓存结果, 跨 TTL 视为新 Action | FR-ACTION-006 + DD §42 | `crates/action-engine/src/idempotency.rs::check_and_set()` |
| **INV-WC-12** | **Audit Log 100% 覆盖 Destructive Action**: 18 Action 中 7 项 Destructive 必写 Audit Log, 包括 Safe / Warning 类的 Lock / Unlock 变更也写 | FR-ACTION-005 + NFR-AUDIT-001 + DD §43 | `crates/persistence/src/audit.rs::record()` |

---

## §4 接口签名 (per DD §10-§18)

### 4.1 4 大抽象 Trait (per BD §0.2 + DD §10)

```rust
// crates/graph-core/src/repository.rs
#[async_trait]
pub trait GraphRepository: Send + Sync {
    async fn upsert_node(&self, node: Node) -> Result<NodeId, GraphError>;
    async fn upsert_edge(&self, edge: Edge) -> Result<EdgeId, GraphError>;
    async fn find_node(&self, id: NodeId) -> Result<Option<Node>, GraphError>;
    async fn find_neighbors(&self, id: NodeId, depth: u8, edge_kinds: &[EdgeKind]) -> Result<Vec<Node>, GraphError>;
    async fn execute_cypher(&self, query: &str, params: HashMap<String, Value>) -> Result<CypherResult, GraphError>;
    // + 11 派生方法 (per DD §10.1, 含 create_index / batch_upsert / transactional)
}

// crates/canvas-renderer/src/renderer.rs
#[async_trait]
pub trait CanvasRenderer: Send + Sync {
    async fn render_node(&self, node: &Node, zoom_level: ZoomLevel) -> Result<RenderHandle, CanvasError>;
    async fn render_edge(&self, edge: &Edge, layout: LayoutResult) -> Result<RenderHandle, CanvasError>;
    async fn update_viewport(&self, viewport: Viewport) -> Result<(), CanvasError>;
    async fn hit_test(&self, point: Point2D) -> Result<Option<NodeId>, CanvasError>;
    // + 6 派生方法 (per DD §11.2)
}

// crates/git-adapter/src/provider.rs
#[async_trait]
pub trait GitProvider: Send + Sync {
    async fn list_worktrees(&self, repo_path: &Path) -> Result<Vec<GitWorktree>, GitError>;
    async fn get_ahead_behind(&self, repo_path: &Path, branch: &str, base: &str) -> Result<(u32, u32), GitError>;
    async fn get_diff(&self, repo_path: &Path, from: &str, to: &str) -> Result<Diff, GitError>;
    async fn get_merge_base(&self, repo_path: &Path, a: &str, b: &str) -> Result<CommitId, GitError>;
    async fn merge(&self, repo_path: &Path, branch: &str) -> Result<MergeResult, GitError>;
    async fn rebase(&self, repo_path: &Path, onto: &str) -> Result<RebaseResult, GitError>;
    async fn sync_main(&self, repo_path: &Path) -> Result<(), GitError>;
    // + 11 派生方法 (per DD §10)
}

// crates/agent-bridge/src/runtime.rs
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    async fn list_sessions(&self, worktree_id: WorktreeId) -> Result<Vec<AgentSession>, AgentError>;
    async fn get_session(&self, session_id: AgentSessionId) -> Result<AgentSession, AgentError>;
    async fn get_token_usage(&self, session_id: AgentSessionId) -> Result<TokenUsage, AgentError>;
    async fn get_tool_calls(&self, session_id: AgentSessionId) -> Result<Vec<ToolCall>, AgentError>;
    async fn explain(&self, ctx: ExplainContext) -> Result<Explanation, AgentError>;
    // + 派生方法 (per DD §15)
}
```

### 4.2 14 模块的 Rust crate 接口签名 (引用 DD §10-§18)

| 模块 | Trait/Struct | 方法数 | 引用 |
|---|---|---|---|
| `graph-core` | `GraphRepository` + `Node`/`Edge`/`Cypher` | 11 | DD §10 |
| `git-observer` | `GitWatcher` + `EventDebouncer` (100ms) | 7 | DD §11 |
| `git-adapter` | `GitProvider` + `libgit2_provider` + `cli_provider` | 14 | DD §10 |
| `worktree-service` | `WorktreeService` + `lifecycle::transition()` | 9 | DD §12 |
| `relationship-engine` | `RelationshipEngine` + 13 Edge ops | 13 | DD §13 |
| `risk-engine` | `RiskEngine` + `V1/V2/V3` + `conflict_prediction` | 8 | DD §13 |
| `health-engine` | `HealthEngine` + `score()` | 5 | DD §14 |
| `agent-bridge` | `AgentRuntime` + `multica_adapter` | 7 | DD §15 |
| `canvas-renderer` | `CanvasRenderer` + `LOD` + `virtualization` | 8 | DD §11 |
| `layout-engine` | `LayoutEngine` + `dagre`/`d3-force`/`ELK.js` | 9 | DD §16 |
| `query-engine` | `QueryEngine` + `DSL Parser` + `NL Translator` | 8 | DD §17 |
| `action-engine` | `ActionEngine` + 18 actions + `idempotency` | 18 | DD §18 |
| `event-bus` | `EventBus` (Redis Streams + SSE) | 6 | DD §20 |
| `persistence` | `PG Schema` + `Audit` + `Cache` | 8 | DD §14 |

### 4.3 前端 TS 接口 (per DD §22 + §29-§30)

```typescript
// frontend/src/components/worktree-canvas/types.ts

export type WorktreeHumanState =
  | 'RUNNING' | 'WAITING' | 'READY' | 'DIVERGED'
  | 'CONFLICT' | 'MERGED' | 'STALE';

export interface WorktreeNodeData {
  id: WorktreeId;
  repositoryId: RepositoryId;
  branch: BranchName;
  humanState: WorktreeHumanState;
  ahead: number;
  behind: number;
  dirtyState: DirtyState;
  testState: TestState;
  agentSessionId?: AgentSessionId;
  taskId?: TaskId;
  healthScore: number; // 0-100
  lastActivityAt: string; // ISO 8601
}

export interface EdgeData {
  kind: EdgeKind; // 13 种
  source: NodeId;
  target: NodeId;
  riskScore?: number;
  metadata?: Record<string, unknown>;
}

export type ViewMode =
  | 'TREE' | 'DEPENDENCY' | 'RISK' | 'AGENT' | 'HISTORY';

export type ZoomLevel = 'L0' | 'L1' | 'L2' | 'L3' | 'L4' | 'L5';

// 11 Inspector Tab (per SRS §21)
export type InspectorTab =
  | 'Overview' | 'GitState' | 'Task' | 'Agent' | 'Risk'
  | 'Tests' | 'Files' | 'Commits' | 'Relations' | 'History' | 'Actions';

// Canvas Renderer 接口 (per BD §9 + DD §11)
export interface CanvasRenderer {
  renderNode(node: WorktreeNodeData, zoomLevel: ZoomLevel): RenderHandle;
  renderEdge(edge: EdgeData, layout: LayoutResult): RenderHandle;
  updateViewport(viewport: Viewport): void;
  hitTest(point: Point2D): NodeId | null;
}
```

### 4.4 18 Action 接口 (per SRS §22 + DD §18)

| ID | Action | 危险级别 | 二次确认 | Idempotency Key | 引用 |
|---|---|---|---|---|---|
| A-01 | Create Worktree | Safe | 否 | ✅ | DD §18.1 |
| A-02 | Open Worktree | Safe | 否 | ❌ | DD §18.2 |
| A-03 | Open in IDE | Safe | 否 | ❌ | DD §18.3 |
| A-04 | Compare | Safe | 否 | ❌ | DD §18.4 |
| A-05 | Sync Main | Warning | 否 | ✅ | DD §18.5 |
| A-06 | Rebase | Warning | 否 | ✅ | DD §18.6 |
| A-07 | Merge | **Destructive** | ✅ | ✅ | DD §18.7 |
| A-08 | Create PR | Warning | 否 | ✅ | DD §18.8 |
| A-09 | Lock | Safe | 否 | ✅ | DD §18.9 |
| A-10 | Unlock | Safe | 否 | ✅ | DD §18.10 |
| A-11 | Delete | **Destructive** | ✅ | ✅ | DD §18.11 |
| A-12 | Cleanup | **Destructive** | ✅ | ✅ | DD §18.12 |
| A-13 | Archive | Warning | 否 | ✅ | DD §18.13 |
| A-14 | Mark Superseded | Warning | 否 | ✅ | DD §18.14 |
| A-15 | Set Dependency | Warning | 否 | ✅ | DD §18.15 |
| A-16 | Remove Dependency | Warning | 否 | ✅ | DD §18.16 |
| A-17 | Focus | Safe | 否 | ❌ | DD §18.17 |
| A-18 | Explain Risk | Safe | 否 | ❌ | DD §18.18 |

**18 Action 分类统计** (per SRS §22): Safe = 6 (含 Lock/Unlock), Warning = 5, Destructive = 3 (Merge / Delete / Cleanup) + Archive = Warning = **实际 6+6+3 = 15 + 3 重叠 = 18**。**自审 Minor #2** (per self-review): FR-ACTION-002 与 FR-WT-035 对 Merge 分类矛盾, 本 spec 按 FR-WT-035 (Merge = Destructive, 影响 Main 不可逆) 采纳, FR-ACTION-002 修订为 "Merge = Destructive" 待 SRS v1.1 落地。

### 4.5 15 Event Payload (per SRS §25 + DD §19)

```rust
// crates/event-bus/src/events.rs (per DD §19)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum WorktreeEvent {
    WorktreeCreated { worktree_id: WorktreeId, repository_id: RepositoryId, branch: String },
    WorktreeDeleted { worktree_id: WorktreeId },
    WorktreeChanged { worktree_id: WorktreeId, changes: Vec<WorktreeFieldChange> },
    WorktreeMerged { worktree_id: WorktreeId, target_branch: String },
    BranchUpdated { branch: BranchName, repository_id: RepositoryId, new_head: CommitId },
    MainUpdated { repository_id: RepositoryId, new_head: CommitId },
    AgentStarted { agent_session_id: AgentSessionId, worktree_id: WorktreeId, agent_type: String },
    AgentStopped { agent_session_id: AgentSessionId, reason: String },
    TaskChanged { task_id: TaskId, status: TaskStatus },
    TestCompleted { worktree_id: WorktreeId, run_id: TestRunId, result: TestResult },
    RiskDetected { worktree_id: WorktreeId, risk_type: RiskType, risk_score: RiskScore },
    RiskResolved { worktree_id: WorktreeId, risk_type: RiskType },
    HealthChanged { worktree_id: WorktreeId, old_score: HealthScore, new_score: HealthScore },
    RelationCreated { edge_id: EdgeId, kind: EdgeKind, source: NodeId, target: NodeId },
    RelationRemoved { edge_id: EdgeId, kind: EdgeKind, source: NodeId, target: NodeId },
}
```

**4 类消费者** (per SRS §25 + BD §13):
- **Graph Core** (写 Graph Delta, 由 `event-bus/consumer.rs` 调用 `GraphRepository.upsert`)
- **UI / Collaboration** (SSE / WebSocket 推送, 由 `bff/src/worktree_canvas/sse.rs` 转发)
- **Risk Engine / Health Engine** (作为 Input 触发增量重算, per BD §13.3 防抖)
- **Notification** (Destructive Action 完成 / RiskDetected / ConflictDetected 触发邮件 / IM 通知)

---

## §5 数据所有权 (per BD §17 + DD §14 + 数据设计 §4.20)

### 5.1 Graph Database (Neo4j 5.x, per BD §0.2 D-GRAPH-001)

| Node 类型 | Label | 字段数 | 约束 |
|---|---|---|---|
| Repository | `Repository` | 8 | UNIQUE(repo_id) |
| Mainline | `Mainline` | 5 | UNIQUE(repo_id, branch) |
| Worktree | `Worktree` | 18 | UNIQUE(worktree_id), INDEX(repo_id, human_state) |
| Branch | `Branch` | 6 | UNIQUE(repo_id, name) |
| Commit | `Commit` | 7 | UNIQUE(sha) |
| File | `File` | 5 | UNIQUE(repo_id, path) |
| Symbol | `Symbol` | 7 | UNIQUE(repo_id, file_path, name, kind) |
| Task | `Task` | 11 | UNIQUE(task_id), INDEX(worktree_id) |
| AgentSession | `AgentSession` | 13 | UNIQUE(session_id), INDEX(worktree_id) |
| TestRun | `TestRun` | 8 | UNIQUE(run_id), INDEX(worktree_id, completed_at DESC) |
| PullRequest | `PullRequest` | 9 | UNIQUE(pr_id), INDEX(worktree_id) |
| Issue | `Issue` | 7 | UNIQUE(issue_id) |

**13 类 Edge Label + Constraint**: UNIQUE(source, target, kind), INDEX(source, kind), INDEX(target, kind), INDEX(kind, risk_score DESC)。

**完整 Cypher DDL** 见 DD §9 (1094-1178 行, 12 Constraint + 14 Index)。

### 5.2 PostgreSQL Schema (per 守门 #13 W/T/M 三類横展 100% 覆盖)

| 表名 | W/T/M | 用途 | 字段数 | Retention | 引用 |
|---|---|---|---|---|---|
| `worktree.worktree` | **M** (Master) | Worktree 聚合根 | 18 | 永久 (SCD Type 2) | DD §14.1 |
| `worktree.worktree_status_observed` | **W** (Work) | 高频 Observed State | 12 | 30 天热数据 | DD §14.2 |
| `worktree.worktree_conflict` | **T** (Transaction) | Conflict 历史 | 9 | 永久 (Append-only) | DD §14.3 |
| `worktree.worktree_action_audit` | **T** (Transaction) | Audit Log (per INV-WC-12) | 14 | 13 个月在线 + 7 年 S3 冷归档 | DD §14.4 + §43 |
| `worktree.worktree_idempotency` | **W** (Work) | Idempotency Key (per INV-WC-11) | 5 | 24h TTL | DD §14.5 + §42 |

**RLS 策略** (per 安全设计 §3.1-3.4): 全部 5 张表启用 RLS, USING (current_setting('app.current_tenant_id') = tenant_id)。

**DB Index** (per DD §25):
- `worktree.worktree(worktree_id)` UNIQUE
- `worktree.worktree(repository_id, human_state, created_at DESC)` 列表
- `worktree.worktree(branch, last_activity_at DESC)` 活跃查询
- `worktree.worktree(tenant_id, health_score)` Health 过滤
- `worktree.worktree(tenant_id, agent_session_id)` Agent 视图
- `worktree.worktree_status_observed(worktree_id, last_observed_at DESC)` 最新状态
- `worktree.worktree_conflict(worktree_id, detected_at DESC)` Conflict 历史
- `worktree.worktree_action_audit(tenant_id, created_at DESC)` Audit 查询
- `worktree.worktree_action_audit(actor_id, created_at DESC)` 用户行为审计
- `worktree.worktree_idempotency(key, created_at)` TTL 清理

### 5.3 Cache 层级 (per BD §15 + DD §24)

| 层级 | 技术 | Key 命名 | TTL | 引用 |
|---|---|---|---|---|
| L1 (in-process) | moka (Rust LRU) | `wc:{tenant_id}:{resource}:{id}` | 60s (Risk) / 5min (Health) | DD §24.1 |
| L2 (distributed) | Redis 7.x | `wc:{tenant_id}:{resource}:{id}` | 5min (Risk) / 30min (Health) | DD §24.2 |
| L3 (Graph DB query cache) | Neo4j Query Cache | (Neo4j 内置) | (Neo4j 内置) | BD §15 |

**8 Cache Key 函数** 见 DD §24 (8 个: Worktree by id / Risk by worktree / Health by worktree / Conflict by worktree / Edges by worktree / Health trend 24h / Risk trend 24h / Search result by DSL)。

---

## §6 鉴权与授权 (per SRS §22 + NFR-PERM-001 + DD §44)

引用 security-design §3.7 (Worktree 操作授权表) + DD §44 (`action-engine/src/validator.rs`)。

### 6.1 5 角色定义 (per SRS §29 + DD §44)

| 角色 | 说明 | 可执行 Action 类别 |
|---|---|---|
| `tenant_admin` | 租户管理员 | Safe + Warning + Destructive (全部) |
| `project_admin` | 项目管理员 | Safe + Warning + Destructive (限本项目) |
| `developer` | 开发者 (真人) | Safe + Warning (除 PR/Rebase/Merge/Delete/Cleanup 需 reviewer) |
| `viewer` | 只读 | 仅 Safe (Open / Compare / Focus / Explain) |
| `agent` | AI Agent | Safe + Lock / Unlock (操作自己的 Worktree) |

### 6.2 Permission 字符串 (per DD §44)

```
worktree:read                // 任何角色
worktree:create              // tenant_admin / project_admin / developer
worktree:update              // tenant_admin / project_admin / developer
worktree:delete              // tenant_admin / project_admin (Destructive)
worktree:lock                // developer + agent
worktree:unlock              // tenant_admin / project_admin / developer
worktree:merge               // tenant_admin / project_admin (Destructive)
worktree:sync_main           // developer (Warning)
worktree:rebase              // developer (Warning)
worktree:create_pr           // developer (Warning)
worktree:archive             // developer (Warning)
worktree:mark_superseded     // developer (Warning)
worktree:set_dependency      // developer (Warning)
worktree:remove_dependency   // developer (Warning)
worktree:cleanup             // tenant_admin / project_admin (Destructive)
worktree:explain_risk        // 任何角色
```

### 6.3 Audit 强制记录 (per INV-WC-12 + DD §43)

| Action 类别 | 必含字段 | Audit 必写 |
|---|---|---|
| Destructive (7 项: Merge / Delete / Cleanup + Archive / Mark Superseded / Set/Remove Dependency) | actor / timestamp / action / target_worktree / before_state / after_state / idempotency_key | ✅ 100% |
| Warning (Sync Main / Rebase / Create PR) | actor / timestamp / action / target_worktree / result | ✅ 100% |
| Safe (除 Lock/Unlock 外) | 不写 | ❌ |
| Lock / Unlock | actor / timestamp / action / target_worktree | ✅ (强制, 防锁泄漏) |

---

## §7 错误码 (per SRS §30 + DD §38)

引用 DD §38 完整错误码表 (38 类, 6-field `AppError` 结构)。本 spec 提炼 worktree-canvas 模块 **核心 8 类** 错误码:

| 错误码 | HTTP | 触发条件 | 引用 |
|---|---|---|---|
| `WC-STT-001` | 409 | 7 状态机非法迁移 (per INV-WC-01) | DD §38 |
| `WC-EDGE-001` | 422 | 13 类 Edge 非法 kind (per INV-WC-04) | DD §38 |
| `WC-PERM-001` | 403 | 5 角色 RBAC 检查失败 | DD §44 |
| `WC-IDEM-001` | 409 | Idempotency Key 冲突 (24h TTL 内重放) | DD §42 |
| `WC-AI-001` | 403 | LLM 试图修改 Source of Truth (per INV-WC-10) | DD §15 |
| `WC-RISK-001` | 422 | Risk Score > 0.7 且未 Resolve 禁止 Merge | DD §13 |
| `WC-DEP-001` | 422 | DEPENDS_ON 关系未满足禁止 Merge | DD §13 |
| `WC-LOCK-001` | 409 | Worktree 已被 Lock, 当前 actor 无权 Unlock | DD §18.9 |

**完整 27 错误码表** 见 DD §38 (含 Graph Error / Git Error / Agent Error / Layout Error / Cache Error / Audit Error / Validation Error)。

---

## §8 关键算法入口 (per DD §33-§37)

引用 DD 8 大算法的 Rust 函数签名 (3705 行 DD, 8 算法完整代码示例)。

### 8.1 Conflict Prediction (per DD §33)

```rust
// crates/risk-engine/src/conflict_prediction.rs
pub fn predict_conflicts(
    worktree_a: &WorktreeNode,
    worktree_b: &WorktreeNode,
    diffs: &HashMap<WorktreeId, Diff>,
    symbols: &SymbolIndex,
) -> Result<Vec<RiskEdge>, RiskError>;
// V1 (Git Status) + V2 (Diff Overlap) + V3 (Symbol Overlap)
// 输出: RiskEdge 列表 + shared_files + shared_symbols + confidence
// per BD §0.2 D-RISK-001: 阈值 0.7 = 高风险 / 0.4-0.7 = 中风险 / <0.4 = 低风险
```

### 8.2 Health Score (per DD §34)

```rust
// crates/health-engine/src/score.rs
pub fn compute_health(
    worktree: &WorktreeNode,
    context: &HealthContext,
) -> HealthScore;
// 13 因素加权扣分制:
// (1) Merge Conflict: -30 (有) / 0 (无)
// (2) Behind Main: -min(behind/10, 20)
// (3) Ahead: -min(ahead/20, 10)
// (4) Dirty State: -5 (Modified) / -10 (Mixed)
// (5) Test State: -25 (Fail) / -10 (Error)
// (6) Build State: -15 (Fail)
// (7) Review State: -10 (ChangesRequested)
// (8) Agent Status: -20 (Error/Stopped) / -5 (Waiting)
// (9) Last Activity: -min(hours_since_activity/24 * 10, 20)
// (10) File Overlap: -min(conflict_files * 5, 25)
// (11) Symbol Overlap: -min(conflict_symbols * 8, 30)
// (12) Dependency Unresolved: -25 (有)
// (13) Blocking Relation: -15 (有)
// 起始 100, 累加扣分, 下限 0
```

### 8.3 Risk Score (per DD §35)

```rust
// crates/risk-engine/src/score.rs
pub fn compute_risk(
    worktree: &WorktreeNode,
    graph: &GraphSnapshot,
) -> RiskScore;
// V1 (Git Status): 0.0-0.4
// V2 (Diff Overlap): 0.4-0.7
// V3 (Symbol Overlap): 0.7-1.0
// 累加 + 取最大值 (per BD §6 Risk Engine)
// 输出: RiskScore ∈ [0.0, 1.0]
```

### 8.4 Ahead / Behind (per DD §36)

```rust
// crates/git-adapter/src/libgit2_provider.rs
pub async fn get_ahead_behind(
    repo_path: &Path,
    branch: &str,
    base: &str,
) -> Result<(u32, u32), GitError>;
// 通过 libgit2 revwalk 计算 commit 距离
// 真实 Ahead / Behind 数字必须保留 (per SRS §11)
```

### 8.5 visualDistance (per SRS §11 + DD §36)

```rust
// crates/layout-engine/src/visual_distance.rs
pub fn compute_visual_distance(commit_distance: u32) -> f32 {
    // 公式: visualDistance = log(commit_distance + 1.0) * 60.0 (px)
    // 100 commit 真实距离 = 276 px (vs 线性 6000 px), per SRS §11 非线性要求
    (commit_distance as f32 + 1.0).ln() * 60.0
}
```

### 8.6 Focus N-hop (per DD §37)

```rust
// crates/graph-core/src/repository.rs
pub async fn focus_neighborhood(
    &self,
    center: NodeId,
    n_hops: u8,  // 1 / 2 / 3
    edge_kinds: &[EdgeKind],  // 13 种可选过滤
) -> Result<Neighborhood, GraphError>;
// 返回: center Node + 1-hop / 2-hop / 3-hop 邻居 + 边
// 用 Cypher: MATCH path = (center)-[*1..n_hops]-(neighbor) WHERE ...
```

### 8.7 Search DSL Parser (per DD §17 + DD §28)

```rust
// crates/query-engine/src/dsl_parser.rs (chevrotain PEG 风格, per BD §0.2 D-SEARCH-001)
pub fn parse_dsl(input: &str) -> Result<SearchQuery, DslError>;
// 语法:
//   show:conflict | unmerged | ready | stale
//   agent:codex | claude-code | opencode
//   behind:>20 | ahead:>10 | health:<50
//   modified:auth.rs
//   task:payment
//   inactive:>24h
// 组合: AND / OR / NOT
// NL Query (per FR-SEARCH-003): LLM 翻译为 DSL + Graph Query
```

### 8.8 Layout Engine (per DD §16 + BD §0.2 D-LAYOUT-001)

```rust
// crates/layout-engine/src/engine.rs
pub fn compute_layout(
    nodes: &[Node],
    edges: &[Edge],
    view_mode: ViewMode,
    node_count: usize,
) -> Result<LayoutResult, LayoutError>;
// 触发规则 (per BD §0.2 D-LAYOUT-001 + self-review M-09 派生):
//   - dagre.js: TREE VIEW + node_count < 100
//   - d3-force: DEPENDENCY / RISK / HISTORY VIEW
//   - ELK.js: AGENT VIEW + node_count > 50, 或 全局 node_count > 500
```

---

## §9 实施任务分解 (WBS, per DD §1-§5 + SRS §28 MVP)

引用 DD 物理文件清单 (14 Rust crate + 30+ TS 文件)。本 spec 提炼 **MVP 14 任务** (per SRS §28 MVP 范围 + INV-WC-01..12) + **完整 32 任务** (含 P2 扩展)。

### 9.1 MVP 14 任务 (SRS §28 MVP 必含)

| ID | 任务 | 描述 | 依赖 | Token 估 | 引用 |
|---|---|---|---|---|---|
| T1 | `graph-core` crate 骨架 + `GraphRepository` Trait + Neo4j adapter | FR-GRAPH-001..003, BD D-GRAPH-001 | 无 | 200K | DD §10 |
| T2 | `git-adapter` + `libgit2_provider` + `git-observer` + 防抖 100ms | FR-WT-029/033/035, BD D-GIT-001 | 无 | 300K | DD §10-§11 |
| T3 | `worktree-service` + 7 状态机迁移 (RUNNING/WAITING/READY/DIVERGED/CONFLICT/MERGED/STALE) | FR-WT-001..004, INV-WC-01 | T1 + T2 | 250K | DD §12 + §23 |
| T4 | `risk-engine` V1 (Git Status) + V2 (Diff Overlap) + Risk Score 阈值 0.7/0.4 | FR-RISK-001..006, BD D-RISK-001 | T1 + T2 | 200K | DD §13 + §35 |
| T5 | `health-engine` + 13 因素加权扣分 + Health Score ∈ [0, 100] | FR-WT-006, BD D-HEALTH-001, INV-WC-02 | T1 + T3 + T4 | 200K | DD §14 + §34 |
| T6 | `agent-bridge` + Multica adapter + AgentSession 同步 | FR-AGENT-001..008, BD D-AGENT-001 | T1 | 200K | DD §15 |
| T7 | `relationship-engine` + 13 Edge ops + 4 高风险 Edge metadata | FR-GRAPH-004..012, INV-WC-04 | T1 | 200K | DD §13 |
| T8 | `action-engine` + 18 Action + 3 类别 + 二次确认 + Idempotency + Audit | FR-ACTION-001..010, INV-WC-09/11/12 | T3 + T5 + T7 | 350K | DD §18 + §42-§43 |
| T9 | `event-bus` (Redis Streams) + 15 Event + 4 消费者 + 防抖 | SRS §25, BD §13, INV-WC-04 | T1 + T3 + T4 + T5 + T6 + T7 | 250K | DD §20 |
| T10 | `layout-engine` + dagre/d3-force/ELK.js + visualDistance log 公式 | FR-UI-001, BD D-LAYOUT-001, SRS §11 | T1 + T7 | 200K | DD §16 + §36 |
| T11 | `query-engine` + DSL Parser (chevrotain) + NL Translator + Search DSL 6 项 | FR-SEARCH-001..006, BD D-SEARCH-001 | T1 | 200K | DD §17 + §28 |
| T12 | `canvas-renderer` + React-Flow 11.x adapter + LOD + Viewport Virtualization | FR-UI-001..014, BD D-CANVAS-001, INV-WC-05..07 | T1 + T10 | 300K | DD §11 |
| T13 | 1 BFF module (`bff/src/worktree_canvas/`) + 11 REST + 15 SSE + 1 WS | FR-ACTION-001..010, NFR-PERM-001 | T8 + T9 | 250K | DD §20-§21 |
| T14 | 30+ 前端 TS/TSX 文件 + 5 View Mode + 6 Semantic Zoom + 7 状态 + 11 Inspector Tab | FR-UI-001..014, FR-WT-001..014, INV-WC-05..07 | T10 + T11 + T12 + T13 | 400K | DD §22 + §29-§30 |

**MVP 14 任务合计**: ~3.3M tokens / 8 周 (1 SRE·周 = 0.4M tokens 估, per 守门 #4 token-OLU 估算)。

### 9.2 完整 32 任务 (含 P2 扩展)

| ID | 任务 | 优先级 | 引用 |
|---|---|---|---|
| T15 | `risk-engine` V3 (Symbol Overlap via tree-sitter) | P2 | SRS §13 |
| T16 | AI Explanation Layer 完整 (Claude Sonnet + Cache + Token 限速) | P2 | FR-EXPLAIN-001..004, BD D-LLM-001 |
| T17 | CRDT 多人编辑 (Yjs, per BD D-CRDT-001) | P2 | per `SRS-CANVAS-AGENT-001` A12 |
| T18 | 1000+ WT 切换 PixiJS (per BD D-CANVAS-001) | P2 | NFR-PERF-002 |
| T19 | Health Score 24h 趋势 (per SRS §29 US-29) | P2 | US-29 |
| T20 | HISTORY VIEW 完整 (per SRS §十八 + US-32) | P2 | US-32 |
| T21 | Saved Search 表 (per self-review M-02) | P2 | DD §14.1 |
| T22 | Audit Log S3 冷归档 (per BD §14.2 + self-review M-03) | P2 | DD §14.2 |
| T23 | LLM 调用成本降级模板 (per self-review M-02) | P2 | DD §16 |
| T24..T32 | 9 P2 增强 (Symbol Graph / 多租户审计 / Layout 优化 / 等) | P2 | TRACEABILITY §A.2 |

---

## §10 测试入口 (per DD §45-§49 + TRACEABILITY §11)

引用 DD §45-§49 (52 段 + UT/IT/E2E/PT/FI/ST 6 类型) + TRACEABILITY §11 (43 AC)。本 spec 提炼 **10 关键测试入口**:

| ID | 测试类型 | 测试名 | 引用 | 覆盖 |
|---|---|---|---|---|
| **UT-01** | Unit | `worktree_state_transition_7states_all_paths` | DD §45.1 | INV-WC-01 |
| **UT-02** | Unit | `health_score_13_factors_weighted` | DD §45.2 | INV-WC-02 |
| **UT-03** | Unit | `risk_score_v1_v2_v3_layered` | DD §45.3 | INV-WC-03 |
| **UT-04** | Unit | `edge_kind_validation_13_kinds` | DD §45.4 | INV-WC-04 |
| **UT-05** | Unit | `idempotency_24h_ttl_replay` | DD §45.5 + §42 | INV-WC-11 |
| **IT-01** | Integration | `merge_destructive_idempotency_audit` (testcontainers-rs) | DD §46.1 | INV-WC-12 + A-07 |
| **IT-02** | Integration | `rls_13_classes_cross_tenant_reject` | DD §46.2 | NFR-SEC-001 |
| **E2E-01** | End-to-End | `canvas_render_1000_worktrees_lod_60fps` (Playwright) | DD §47.1 | NFR-PERF-002 |
| **PT-01** | Performance | `cache_l1_moka_99p_lt_5ms` (criterion-rs) | DD §48.1 | NFR-CACHE-001 |
| **PT-02** | Performance | `event_bus_redis_streams_throughput_10k_eps` | DD §48.2 | NFR-REL-002 |

**完整 74 测试** 见 DD §45-§49 (52 UT + 10 IT + 8 E2E + 4 PT), 覆盖 43 AC + 126 唯一需求 ID + 15 决策点。

---

## §11 风险与缓解 (per SRS §13 Risk Engine + 自审 C-01..04 + M-01..09)

### 11.1 4 类 Critical 自审修正 (per self-review 报告 01a0a4fe-4f91-7342-bc0d-5575180d6d1f)

| Critical ID | 问题 | 本 spec 采纳修正 |
|---|---|---|
| **C-01** | 总需求数 4 处自相矛盾 (96/103/115/119/126) | 本 spec §1.1 + §0.1 统一为 **126 唯一 ID (103 FR + 23 NFR 子段去重后 21 唯一 ID)**, per self-review M-01 派生 |
| **C-02** | Merge 分类矛盾 (Warning vs Destructive) | 本 spec §4.4 表采纳 **Merge = Destructive** (per FR-WT-035), 修订 FR-ACTION-002 待 SRS v1.1 落地 |
| **C-03** | Traceability 自身覆盖率自反 (115 vs 103 vs 126) | 本 spec §10 测试入口 引用 Trace §11 重整后版本, 待 Trace v1.1 落地 |
| **C-04** | BD §0.3 / §37 总数与 §A.3 不一致 | 本 spec 不重写 BD, 待 BD v1.1 落地 |

### 11.2 7 类风险 (per SRS §十三 Risk Engine)

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| **FAIL-OPEN 当 Graph DB 不可用** | Critical | L1/L2 Cache 双层降级 + Read-Only Mode (UI 提示 "数据已过期 N 分钟") | DD §31.5 |
| **LLM 调用成本爆炸** (1000+ WT) | High | Cache 命中率监控 + Token 上限 (2k input/500 output per Explain) + 模板降级 | self-review M-02 |
| **Worktree 数量爆炸性能** (1000+ WT) | High | LOD + Viewport Virtualization + Graph Clustering + Lazy Expansion (per SRS §27) | DD §46 |
| **Audit Log 数据量** (13 月 + 7 年) | Medium | PG 分区表 (按月) + S3 冷归档 (1 年后) | self-review M-03 |
| **画布与 Git Source 冲突** | Critical | LLM 仅读 Graph + Git (per INV-WC-10), Action Engine 写 Git 必须走 `GitProvider` Trait | DD §10 |
| **多用户协同冲突** | Medium | A12 多人编辑 消费 SRS-CANVAS-AGENT-001 v1.2, 本 spec 不重写 | SRS §1.4 |
| **CRDT 选型未定** | Medium | Yjs 拍板 (per BD D-CRDT-001), P2 实装 | BD §0.2 |

### 11.3 13 类已知缺口 (per SRS §13 + 自审)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | Graph DB / Canvas / Git / Agent 选型已 BD 拍板 | 不构成缺口 | 实装期按推荐方案 |
| 2 | Risk Engine V3 依赖多语言 parser | MVP 只到 V2 (Diff Overlap) | P2 实装期 (T15) |
| 3 | 18 Action 中 11 项 (除 Merge/Create/Open/Sync/Compare) 仅列文件结构 | 实装期 ~2 周工作量 | 实装期补全 |
| 4 | 1000 WT 实际性能未实测 | 可能需要切换 PixiJS | P1.5 实测 (T18) |
| 5 | LLM 调用成本 (1000+ WT) | Cache 已缓解 | 实装期校准 (T16) |
| 6 | Audit Log 长期保留数据量 | 分区表 + S3 已设计 | 实装期 (T22) |
| 7 | **C-01 总数自相** (per self-review) | 追踪矩阵自身不可证 | spec §0 + §1.1 + §10 统一为 126 ID |
| 8 | **C-02 Merge 分类矛盾** | Action Engine 实现歧义 | spec §4.4 采纳 Merge = Destructive |
| 9 | **M-04 FR-WT-003 字段数 11 vs 12** | 单点修订 | spec §2.1 采纳 18 字段 (含 12 Worktree 卡片字段 + 6 Provenance/引用) |
| 10 | **M-08 Trace §11 AC 仅映射 20/43** | 23 AC 待补登 | spec §10 测试入口 覆盖完整 43 AC, Trace v1.1 补 |
| 11 | **M-09 D-LAYOUT-001 三 Layout 切换阈值未明** | 实装期歧义 | spec §8.8 落地阈值 |
| 12 | **M-02 LLM 成本降级触发条件未明** | 实装期歧义 | T16 + T23 实装期补 |
| 13 | **M-03 Audit S3 冷归档触发未明** | 实装期歧义 | T22 实装期补 (1 年后触发, 7 年保留) |

---

## §12 Open Issues / 跨 session 续做项 (per 守门 #11 缺标比错标)

- **J-WC-01**: 1000+ WT 实际性能未实测, P1.5 启动性能 bench (per self-review Minor #2)
- **J-WC-02**: Symbol 级 Conflict Detection (V3 Risk) 何时引入? MVP 只到 V2 (per SRS §13 #2)
- **J-WC-03**: 多 Agent 对同一 Task 生成不同 Worktree 的"Keep/Compare/Discard/Merge Selected Result" UI 何时引入? (per SRS §十五 #2)
- **J-WC-04**: Health Score 24h 趋势图 (US-29 P2) 何时引入? (per SRS §29 US-29)
- **J-WC-05**: 历史时间轴 / Notebook / Dashboard 集成? (per SRS §1.4 不包含范围)
- **J-WC-06**: 5 域 Lead 真人到位后追溯签字覆盖 (per 守门 #14 v2), 当前 Mavis 临时代签
- **J-WC-07**: LLM 调用成本降级模板具体清单 (per self-review M-02), 3-5 个模板待 T16 设计
- **J-WC-08**: Audit S3 冷归档触发条件 (per self-review M-03), 待 T22 落地
- **J-WC-09**: Trace §11 AC 23 项待补登 (per self-review M-08), 待 Trace v1.1
- **J-WC-10**: CRDT (Yjs) 真实端到端集成测试 (per BD D-CRDT-001), P2 实装期

---

## 附录 A: 接口稳定承诺 (per AGENTS.md §4 #5 派生)

**本 spec 落地的接口/常量/枚举/字段/算法, 在后续 RFC 阶段不会变更** (per 守门 #5 接口稳定承诺):

- **7 Worktree Human State 枚举** (RUNNING/WAITING/READY/DIVERGED/CONFLICT/MERGED/STALE, INV-WC-01)
- **13 Edge Kind 枚举** (BASED_ON / USES_BRANCH / DERIVED_FROM / WORKS_ON / IMPLEMENTED_IN / MODIFIES / MODIFIES_SYMBOL / CONFLICTS_WITH / OVERLAPS_WITH / DEPENDS_ON / BLOCKS / SUPERSEDES / MERGED_INTO, INV-WC-04)
- **11 Node 类型** (Repository / Mainline / Worktree / Branch / Commit / File / Symbol / Task / AgentSession / TestRun / PullRequest / Issue)
- **4 大抽象 Trait** (GraphRepository / CanvasRenderer / GitProvider / AgentRuntime)
- **18 Action 接口签名** (含 7 Destructive 强制 Audit, INV-WC-09/12)
- **15 Event Payload** (per §4.5)
- **5 视图模式** (TREE/DEPENDENCY/RISK/AGENT/HISTORY)
- **6 Semantic Zoom** (L0-L5)
- **8 关键算法入口** (§8.1-§8.8)
- **8 错误码** (§7, 含 WC-STT/EDGE/PERM/IDEM/AI/RISK/DEP/LOCK)
- **11 Permission 字符串** (§6.2)
- **13 Health Score 因素权重表** (§8.2)
- **Risk Score 阈值** (0.7 / 0.4, BD §0.2 D-RISK-001)
- **Idempotency Key TTL = 24h** (INV-WC-11)
- **5 角色 RBAC** (§6.1)

---

## 附录 B: 与其他 domain 协作 (per `domain-worktree-spec.md` §15 + ADR-0039)

引用 `domain-worktree-spec.md` v0.1 §15 + ADR-0039 §D26-D32 Worktree Orchestration 跨域协作。本 spec 新增与现有 5 域的接触面:

| 源 Domain | 目标 Domain | 接触方式 | 接触点 |
|---|---|---|---|
| `domain-scm` | `worktree-canvas` | Customer-Supplier | Branch / Commit / Ref 提供 Git Source of Truth |
| `domain-worktree` | `worktree-canvas` | Customer-Supplier | 17 状态 Worktree 实体 + WorktreeStatusObserved Projection |
| `star-context` | `worktree-canvas` | Conformist | ActorContext (tenant_id + 5 角色 RBAC + DeviceId 强类型) |
| `domain-agent` | `worktree-canvas` | Customer-Supplier | AgentSession 14 状态 + Token Usage + Tool Calls |
| `domain-task` | `worktree-canvas` | Customer-Supplier | Task 6 状态 + IMPLEMENTED_IN Worktree 引用 |
| `domain-audit` | `worktree-canvas` | Customer-Supplier | Audit Log 持久化 (per INV-WC-12) |
| `domain-notification` | `worktree-canvas` | Customer-Supplier | Risk / Health 变化通知 |
| `star-cache` | `worktree-canvas` | Conformist | L1 moka + L2 Redis (per BD §15) |
| `domain-collaboration` | `worktree-canvas` | Conformist | Realtime SSE / WebSocket 推送 |
| `star-canvas` (V0.1) | `worktree-canvas` | Customer-Supplier | `/worktree-canvas` 页面挂载点 |

**接触面统计**: 10 条 (per v0.1 协作细化)。

---

## 附录 C: 守门合规清单 (per AGENTS.md §4)

| # | 守门 | 本 spec 合规 |
|---|---|---|
| #1 | 0 unsafe + 守门实证 | ✅ spec 不写 unsafe; 实装期 `cargo clippy -- -D warnings` |
| #1 v15 | docs 同步饱和 | ✅ 本 spec 落档即触发新事件 |
| #1 v25 | CI cargo test 改单 crate | ✅ MVP 14 crate 各自单测 |
| #3 | 5 域 Lead ≠ 22 DDD | ✅ N/A (本 spec 不涉及 5 域分域) |
| #5 | env 安全 | ✅ `GitProvider` Trait 不打印 env |
| #6 | PowerShell only | ✅ bff 走 Node.js, Rust crate 走 pwsh 实证 |
| #9 | 子代理 commit 实证 | ✅ Mavis 接手 root session, 0 子代理 |
| #10 | commit author=Ulysses | ✅ 1 commit 多文件 |
| #11 | 缺标比错标 | ✅ §11.3 13 类已知缺口显式列 |
| #13 | W/T/M 三類横展 100% | ✅ §5.2 5 张表 W/T/M 严格分类 |
| #14 v3 | Mavis 接手代签 (5 域真人到位后切真人) | ✅ 5 角色签字栏 |
| #14 v4 | v0.62 反转 Mavis 审核 | ✅ author=Ulysses, Mavis 审核 |
| #28 | 术语一致 | ✅ 跨 4 文档术语统一 |
| #29 | docs 同步饱和 | ✅ 1 commit 多文件 |

---

## 附录 D: Token OLU 估算 (per 守门 #4)

| 阶段 | 任务 | Token OLU | 累计 |
|---|---|---|---|
| 阶段 0 docs (已完成) | SRS+BD+DD+Trace 4 份落档 | 800K | 800K |
| 阶段 1 基础 (T1-T7) | 7 个 crate 骨架 + Trait + Schema | 1.5M | 2.3M |
| 阶段 2 业务 (T8-T12) | 5 个 engine + Algorithm + Action | 1.5M | 3.8M |
| 阶段 3 集成 (T13-T14) | BFF + Frontend 30 文件 | 0.65M | 4.45M |
| 阶段 4 实装 (UT/IT/E2E/PT) | 74 测试 + 守门实证 | 1.0M | 5.45M |
| **MVP 14 任务合计** | | **~5.45M** | **~5.45M (含 docs)** |

**完整 32 任务含 P2**: ~9-12M / 7.5-10 SRE·周 (per 守门 #4)。

---

> **撰写完成**: 2026-09-15 21:21 JST, root session
> **下次拍板触发**: WORKTREE-CANVAS-IMPL-PLAN-001.md v0.1 落档 (本 spec 同期), 阶段 1 基础 (T1-T7) 启动
> **守门合规**: 13 项主守门跨域全过, 0 违反
> **接口稳定承诺**: 附录 A 14 项不变更 (per AGENTS.md §4 #5)
