# F47 代码结构图谱

> 状态摘要: 代码结构图谱（Tree-sitter Worktree Graph）目前只有两个库级构件：`star-treesitter`（5 种语言的语法解析与符号提取，另含跨文件引用解析的内存索引）和 `star-taskgraph`（Task 与 Worktree 的 1:1 内存绑定，并输出 react-flow 形状的 JSON）。图服务进程、`/api/graph/*` 接口、`/graph/<task-id>` 页面、任务卡 Graph 标签页、diff 叠加与文件遍历缓存均未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

在 Kanban 任务卡与其绑定的 git worktree 之间建立代码结构视图：每张任务卡对应一个 worktree，视图展示该 worktree 当前 commit 的模块、类型、函数拓扑（AST 与符号引用图），并把任务卡的 add/modify/delete 改动叠加到图节点上，用于变更影响分析与评审辅助。

来源: `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` §0、§1.1、§1.3 (E-01..E-06)；`02-basic-design.md` §0。

### 1.2 范围内 / 范围外

| 范围 | 内容 |
|---|---|
| 范围内 | Rust 与 TypeScript 语法解析（MVP）；符号节点与 contain/call/import/reference 边；task→worktree 绑定；diff 叠加；图 JSON 输出；独立图视图页面；图缓存；审计与密钥扫描约束 |
| 范围外（设计已列出，按阶段） | 增量 re-parse（P2）；类型推断与控制流图（P3）；图导出 PNG/SVG（P2）；跨 commit 图比较（P3）；守门违规检测（P3）；多 worktree 并行视图（P3）；Python/Go/Java 等扩展语法（P2） |
| 不在本文档 | `/canvas/worktree-graph`（worktree 关系图，见 §5 相邻实现）；`/api/graph/ensure-fresh` 与 `/api/graph/cypher`（ArchGraphModal 的架构图，见 §5 相邻实现） |

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 主用户 / 决策者 | 在任务卡中看到其 worktree 的代码结构，检查本任务的改动是否在预期范围内 |
| 开发者 / 评审人 | 点击节点查看签名与代码预览，沿调用或引用边追踪跨文件关系，按改动状态过滤 |
| 任务卡操作者 | 将任务卡绑定到某个 worktree，并从任务卡进入图视图 |
| 系统（MCP 客户端） | 通过 `get_task_graph` 工具获取图 JSON（设计项 F-19，代码未实现） |

主要用户故事（来自 01 §2.3）: UC-01 任务卡进入图视图；UC-02 按 worktree 加载图；UC-03 节点详情；UC-04 diff 叠加；UC-06 搜索与过滤；UC-07 引用边追踪；UC-08 手动刷新；UC-09 解析失败降级。

## 3. 需求

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-47-01 | 任务卡 ↔ worktree 绑定（设计要求 `worktree_id` 必填，并带 `base_commit`） | P0 | 🟡 | 两套模型并存：`crates/star-taskgraph/src/lib.rs:54`（`TaskCard.worktree_id: Option<String>`，无 `base_commit`）；`frontend/src/types/ids.ts:264`（`WorkItem.worktree_id?: Uuid`，可选，N:1）；`frontend/src/components/board/WorkItemDetailDrawer.tsx:498-508`（"Worktree 关联"下拉，可选择"未关联"） |
| FR-47-02 | 解析 `git worktree list --porcelain` 得到 worktree 路径映射 | P0 | 🟡 | `crates/git-adapter/src/cli_provider.rs:113-150`（`list_worktrees` 解析 `worktree`/`HEAD`/`branch` 行）；生产调用者 `crates/worktree-shared-dir/src/external_worktree_import.rs:718-722`（`scan`）。缺少向图服务暴露 task_id→路径的解析器 |
| FR-47-03 | 多语言 tree-sitter 解析（MVP：Rust + TypeScript） | P0 | 🟡 | `crates/star-treesitter/src/lib.rs:38-51`（`Language` 含 Rust/TypeScript/Python/Go/Json）；`lib.rs:171-179`（语法映射）。TypeScript 仅用 `LANGUAGE_TYPESCRIPT`，未覆盖 TSX（`lib.rs:174`）；Python/Go 已接入但未在图中使用 |
| FR-47-04 | 遍历 worktree 源文件并抽取 File/Function/Class/Const 节点 | P0 | ⬜ | 无文件遍历代码：`crates/star-treesitter/src/lib.rs` 仅解析调用方传入的源字符串（`lib.rs:149`）；无 `read_dir` 或工作区遍历模块（全仓 grep 无对应实现） |
| FR-47-05 | 符号解析：跨文件 call/import/reference 边 | P0 | 🟡 | `crates/star-treesitter/src/symbol_resolver.rs:178-205`（`resolve_references` 接收调用方提供的 `SymbolReference` 列表）；`symbol_resolver.rs:43-51`（引用由 `::` 拆分得到）。没有从源码中提取引用的逻辑；`target_file` 为启发式 `"{parts[-2]}.rs"`（`symbol_resolver.rs:187-189`） |
| FR-47-06 | Graph JSON 输出（nodes + edges + metadata） | P0 | 🟡 | `crates/star-taskgraph/src/lib.rs:214-271`（`to_react_flow` 只输出 task 与 worktree 节点及 binding 边，无符号节点）；`lib.rs:280-287`（`ReactFlowGraph`）。无 `metadata`、`parse_errors` 字段 |
| FR-47-07 | diff 叠加（added/modified/deleted）到图节点 | P0 | ⬜ | `crates/git-adapter/src/cli_provider.rs:206-236`（`diff` 使用 `--numstat`，不含 A/M/D 状态；`crates/git-adapter/src/provider.rs:65-86` 的 `Diff`/`DiffFile` 无状态字段）；该方法在仓内无调用者（全仓 `.diff(` 仅出现在 git-adapter 内部）。无 `DiffMark` 类型 |
| FR-47-08 | 缓存（按 worktree 路径与 HEAD commit 键控，LRU，TTL 1h） | P0 | ⬜ | 全仓 `crates/star-taskgraph` 与 `crates/star-treesitter` 无缓存代码；清单中无 `lru` 依赖（`Cargo.toml` 仅 `notify` 一项，见 §5） |
| FR-47-09 | 图视图独立路由 `/graph/<task-id>` 与 react-flow 渲染 | P0 | ⬜ | `frontend/src/app/` 下无 `graph` 目录（`git ls-tree -r --name-only 637ed19d frontend/src/app` 无 `graph/` 条目）；`frontend/package.json` 无 react-flow / @xyflow / dagre 依赖 |
| FR-47-10 | 节点交互：点击显示详情与代码预览，边悬停显示信息 | P0 | ⬜ | 无对应组件；`frontend/src/components/graph/` 目录不存在 |
| FR-47-11 | 搜索与过滤（节点名、路径、类型、diff 状态） | P1 | ⬜ | 无对应代码 |
| FR-47-12 | 节点跳转 IDE（`vscode://file/...`） | P1 | ⬜ | 全仓无 `vscode://` 生成代码（未检索到 `vscode://` 字符串） |
| FR-47-13 | 增量 re-parse（file watcher） | P2 | ⬜ | `Cargo.toml:291` 在工作区声明 `notify = "8"`，但只被 `crates/git-observer/Cargo.toml:30` 引用，图路径未使用 |
| FR-47-14 | MCP 工具 `get_task_graph`（16 → 17 tools） | P1 | ⬜ | `crates/star-mcp/src/tools/` 内无 `get_task_graph.rs`；`git grep get_task_graph` 在 `.rs` 与 `.toml` 中无命中 |
| FR-47-15 | 任务卡 Graph 标签页与跳转入口 | P0 | ⬜ | `frontend/src/components/board/WorkItemDetailDrawer.tsx` 中无 graph 标签或 "Open Graph View" 入口（全文件 grep "graph" 无命中） |
| FR-47-16 | graph-service 独立二进制（端口 8090） | P0 | ⬜ | 无 `graph-service` crate（`git grep graph-service` 在 `.rs`/`.toml` 中无命中）。端口 8090 被 `crates/star-ops/src/main.rs:18` 占用（默认 `unwrap_or(8090)`） |
| FR-47-17 | 审计日志（graph 查询写入审计） | P0 | ⬜ | 无图查询路径，无对应写入点 |
| FR-47-18 | 路径安全（仅允许 `.worktrees/*` 与主仓，禁止符号链接逃逸与 `..` 越界） | P0 | ⬜ | 无文件路径解析模块 |
| FR-47-19 | 文件大小限制（单文件 > 5MB 跳过并标记 oversized） | P0 | ⬜ | `git grep -i oversized` 在 `crates/star-treesitter`、`crates/star-taskgraph`、`crates/graph-core` 中无命中 |
| FR-47-20 | 解析失败降级（部分图 + 错误列表） | P0 | 🟡 | `crates/star-treesitter/src/lib.rs:156-167`（`has_errors` 布尔值，`error_count` 仅为 0 或 1，并非真实计数）；`lib.rs:19-29`（`ParseError` 变体在仓内未被构造，`git grep "ParseError {"` 仅命中定义处）。无 `parse_errors` 列表 |
| FR-47-21 | secret 扫描（输出前脱敏 `ghp_`/`AKIA` 等） | P0 | ⬜ | 图路径无扫描器。相邻可复用实现：`crates/nema-codec/src/lib.rs:2186-2195`（`looks_like_secret_v12`）位于另一 crate，未接入图 |
| FR-47-22 | 缓存与 graph 结构 schema 版本（schema_version 变更失效） | P1 | ⬜ | 无 `schema_version` 字段 |
| FR-47-23 | 节点与边 schema 文档化（JSON schema） | P1 | ⬜ | 无对应文档产物 |
| FR-47-24 | 符号节点与 worktree 的 MODIFIES_SYMBOL 关系 | P2 | 🟡 | `crates/graph-core/src/edge.rs:77-78`（`ModifiesSymbol` 变体，标注 P2）；`crates/graph-core/src/node.rs:345-365`（`SymbolNode`）。无生产者：`git grep "SymbolNode\|ModifiesSymbol"` 在 graph-core 之外无命中 |
| FR-47-25 | 节点统计（节点数、边数、parse 耗时、cache 状态）状态栏 | P1 | ⬜ | 无对应代码 |

优先级沿用 01 §2.4（F-01..F-20）与 02 §8.1 里程碑，FR 编号为本文档归并编号。

## 4. 设计

### 4.1 概念模型

| 概念 | 设计（来源 01 §5、02 §3） | 当前代码 |
|---|---|---|
| Worktree | `git worktree list` 输出的工作副本，id 为 basename | `git-adapter::WorktreeInfo`（`provider.rs:44-61`）；`star-taskgraph::Worktree`（`lib.rs:112-121`，内存） |
| TaskCard | 绑定 1 个 worktree 的看板卡 | `star-taskgraph::TaskCard`（`lib.rs:44-61`）；`WorkItem`（`frontend/src/types/ids.ts:264`） |
| Node | file / function / class / struct / const，带 `diff` 标记 | 符号级 `Symbol`（`star-treesitter/src/lib.rs:80-89`，仅 kind/name/line/column）；`graph-core::SymbolNode`（`node.rs:345-365`）与 `FileNode`（`node.rs:328-341`） |
| Edge | contain / call / import / reference | `symbol_resolver::ReferenceEdge`（`symbol_resolver.rs:63-75`）；`graph-core::EdgeKind`（`edge.rs:64-91`，worktree 关系类，不含 call/import） |
| DiffMark | added / modified / deleted / renamed | 不存在 |
| Graph | 含 metadata 与 parse_errors 的 JSON | 不存在；`ReactFlowGraph`（`star-taskgraph/src/lib.rs:280-287`）仅 task/worktree 节点 |

### 4.2 数据模型

设计定义的 Rust 结构（02 §3.1–§3.4）：`NodeKind`（File/Function/Class/Const）、`Node`、`DiffMark`、`ParseStatus`、`EdgeKind`（Contain/Call/Import/Reference）、`Edge`、`Graph`、`GraphMetadata`、`CacheKey`。

代码现状：

| 设计结构 | 代码 | 状态 |
|---|---|---|
| `Symbol { kind, name, line, column }` | `crates/star-treesitter/src/lib.rs:79-89` | 🟡 缺 `end_line`、`signature`、`visibility`、`file_path`、`parent` |
| `SymbolKind`（函数/结构体/枚举/trait/impl/const/static/module/type alias/class/interface/method/variable/other） | `lib.rs:92-122` | 🟡 与设计的 Function/Class/Const 不完全对应；无 `Partial` 或 `Failed` 状态 |
| `ParseResult { language, symbols, has_errors, error_count }` | `lib.rs:125-135` | 🟡 `error_count` 语义为布尔近似 |
| `SymbolReference { raw, parts, line, column }` | `symbol_resolver.rs:29-39` | 🟡 结构存在，无生产者 |
| `SymbolIndex`（按文件与全局名称索引） | `symbol_resolver.rs:77-83` | 🟡 内存结构；`add_file` 同名符号以后插入者覆盖（`symbol_resolver.rs:100-106`） |
| `TaskCard`、`Worktree`、`TaskGraph` | `star-taskgraph/src/lib.rs:44-61, 112-121, 148-151` | 🟡 全部为内存结构，无持久化 |
| `Graph` / `GraphMetadata` / `CacheKey` / `NodeKind` / `DiffMark` / `Edge` | 无 | ⬜ |
| `graph-core::FileNode`（id, repo_id, path, size_bytes, lines, language） | `crates/graph-core/src/node.rs:328-341` | 🟡 与设计的 File 节点字段接近，无生产者 |
| `graph-core::SymbolNode`（id, file_id, qualified_name, kind, line_start/end, signature） | `node.rs:345-365` | 🟡 字段比 `star-treesitter::Symbol` 丰富，二者未互通 |

持久化分类（守门 #13，依据 01 §4 C-19 与 02 §3.5）：`worktree_id`、`base_commit` 属 Work 类（短 TTL）。`graph-core` 模块的 5 表 DDL 位于 `db/migrations/`（`graph-core/src/lib.rs` 头注释），本文档未核对该 DDL 的具体内容。

### 4.3 接口（routes, APIs, ports）

设计接口（02 §4）与代码对照：

| 设计接口 | 代码 | 状态 |
|---|---|---|
| `GET /api/health`（端口 8090） | 无 | ⬜ |
| `GET /api/worktrees` 与 `GET /api/worktrees/:wt_id` | 无 graph 服务；仅有 star-api-rest 的 `worktree_external` 路由（`crates/star-api-rest/src/routes/worktree_external.rs`），与本功能无关 | ⬜ |
| `GET /api/graph/:task_id`（可带 `base_commit`） | 无 | ⬜ |
| `GET /api/graph/:task_id/status` | 无 | ⬜ |
| `POST /api/graph/:task_id/refresh` | 无 | ⬜ |
| `GET /api/graph/:task_id/node/:node_id` | 无 | ⬜ |
| 内部：`parse_file`、`extract_nodes`、`resolve_symbols`、`build_graph`、`apply_diff_overlay`、`get_or_compute` | 对应代码：`parse`（`lib.rs:149`）、`extract_symbols`（`lib.rs:182`）、`resolve_references`（`symbol_resolver.rs:178`）；`build_graph`、`apply_diff_overlay`、`get_or_compute` 无 | 🟡 / ⬜ |
| MCP `get_task_graph`（F-19） | 无 | ⬜ |
| `ReactFlowGraph` 输出（INV-TG-03） | `crates/star-taskgraph/src/lib.rs:214-271` | 🟡 库函数，无 HTTP 或 MCP 暴露 |

已存在、与图相邻的接口（不属本功能）：`crates/star-mcp/src/tools/search_code.rs`、`find_references.rs`、`get_symbol.rs`、`get_code_context.rs` 4 个 MCP 工具（tools 目录现有实现，见 §5）。

### 4.4 状态机与关键流程

设计流程（02 §5.1）：WorktreeResolver → Cache 查询 → parse 所有源文件 → symbol 解析 → graph 组装 → diff 叠加 → 缓存写入 → 审计 → 返回 JSON。

代码现状：仅 `star-treesitter` 的 "解析单个字符串 → 提取符号" 这一步有实现（`lib.rs:149-169`）。`star-taskgraph` 的绑定有双向校验：`TaskGraph::bind`（`star-taskgraph/src/lib.rs:173-202`）要求任务与 worktree 均未绑定，否则返回 `BindingConflict`。绑定关系为 1:1 内存映射，无状态迁移。

TaskCard 状态枚举（`lib.rs:64-76`）：`Pending` / `Active` / `Completed` / `Failed` / `Cancelled`，与设计中任务卡状态无关联（设计未定义图视图的状态机）。

### 4.5 权限与安全

| 设计要求（01 §3.3） | 当前代码 | 状态 |
|---|---|---|
| NFR-S-01 路径限制、禁止符号链接逃逸 | 无路径解析 | ⬜ |
| NFR-S-02 单文件 > 5MB 跳过 | 无 | ⬜ |
| NFR-S-03 `..` 越界防护 | 无 | ⬜ |
| NFR-S-04 禁止打印环境变量值（守门 #5） | `star-treesitter`/`star-taskgraph` 不读取环境变量 | ✅（不涉及，仅确认未读取） |
| NFR-S-05 图查询全量审计 | 无 | ⬜ |
| NFR-S-06 输出前 secret 扫描 | 无；相邻实现 `nema-codec::looks_like_secret_v12`（`crates/nema-codec/src/lib.rs:2186-2195`）未接入 | ⬜ |
| 租户隔离 | `TaskCard.tenant_id: String`（`star-taskgraph/src/lib.rs:51-52`）存在；无 ActorContext 校验、无跨租户拒绝逻辑 | 🟡 |
| `worktree_id` 不跨 session 持久化（C-19） | 内存结构，进程退出即丢失 | 🟡（满足"不持久"，但也不满足"持久化业务事实"之外的任何要求） |

### 4.6 依赖关系

| 依赖 | 设计用途 | 代码状态 | 证据 |
|---|---|---|---|
| tree-sitter 与语法包 | 解析源码 | ✅ 已接入 | `crates/star-treesitter/Cargo.toml:15-22`（`tree-sitter = "0.27"`、`tree-sitter-rust = "0.24"`、`tree-sitter-typescript = "0.23"`、`tree-sitter-python = "0.25"`、`tree-sitter-go = "0.25"`、`tree-sitter-json = "0.24"`）；`Cargo.lock` 中 tree-sitter 0.27.2 |
| git CLI（worktree 解析、diff） | F-02、F-07 | 🟡 worktree 解析已用于生产扫描；diff 无调用者 | `crates/git-adapter/src/cli_provider.rs:113-150, 206-236` |
| star-treesitter → star-taskgraph | 符号类型复用 | ✅ | `crates/star-taskgraph/Cargo.toml:17`；`crates/star-taskgraph/src/lib.rs:18`（`use star_treesitter::Symbol`） |
| domain-worktree | 复用 port/service | ❓ 未核实 | 01 §8 与 02 §2 指向 `crates/domain-worktree`；本次核对未深入其 port/service，未确认与本功能的接口 |
| star-mcp（17th tool） | F-19 | ⬜ | 无 `get_task_graph` |
| star-audit 共享库 | NFR-S-05 | ❓ 未核实 | 02 §2.1 指向 star-audit；本次未检索到该 crate 名称（未核实其存在性） |
| react-flow（前端） | 图渲染（G-07） | ⬜ | `frontend/package.json` 无 react-flow / dagre 依赖 |
| cytoscape（前端，已有） | 架构图渲染（非本功能） | ✅（用于 ArchGraphModal） | `frontend/package.json:30-31`；`frontend/src/components/board/ArchGraphModal.tsx:82-91` |
| `layout-engine`（Rust，dagre/d3-force/elk 选择器） | 设计 §6.4 布局 | 🟡 Rust 库存在，未接入图视图 | `crates/layout-engine/src/lib.rs:1-25` |
| `graph-core`（worktree-canvas 图模型） | 设计未提及 | 🟡 存在，与本功能目标相邻 | `crates/graph-core/src/lib.rs:1-30`（Neo4j 适配器仅生成 Cypher 字符串，无 bolt 客户端依赖） |
| `file-session-management` 对本功能的消费 | FR-CONTEXT-01 需要 `GraphService` | ⬜ | `docs/architecture/file-session-management/02-implementation-plan.md:180` 引用 `treesitter_worktree_graph::GraphService`；`docs/features/F12-file-session.md:197` 记录该 crate 不存在 |

### 4.7 页面设计

本功能计划中的独立页面为 `/graph/<task-id>`（01 §1.4.1、02 §6.1），代码中不存在对应路由（见 FR-47-09）。

任务卡入口（设计中的 Graph 标签页，FR-47-15）同样未实现。`WorkItemDetailDrawer.tsx` 仅提供 worktree 选择下拉，不含图入口。

相邻页面（不属本功能，列出以免混淆）：

- `frontend/src/app/canvas/worktree-graph/page.tsx`：worktree 关系图（树 / 依赖 / 风险 / Agent / 历史五视图），使用 `useStore` 的 `worktrees`、`workItems`、`agentSessions`、`projects`、`lanes`，无符号或代码级节点。
- `frontend/src/components/board/ArchGraphModal.tsx`：卡级架构查看器（ADR-0041），使用 cytoscape，调用 `/api/graph/ensure-fresh` 与 `/api/graph/cypher`（仅 MSW mock，见 `frontend/src/mocks/handlers/graph.ts:54-90`）。这些端点不是本设计的 `/api/graph/:task_id`。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 语法解析与符号提取 | `crates/star-treesitter/src/lib.rs` | 🟡 | `lib.rs:149-169`（parse），`lib.rs:182-244`（walk 与 extract_name）；仅 5 种语言，无文件遍历 |
| 跨文件引用解析（内存） | `crates/star-treesitter/src/symbol_resolver.rs` | 🟡 | `symbol_resolver.rs:77-221`；引用由调用方提供，无源码提取 |
| 解析测试 | `crates/star-treesitter/src/tests.rs` | ✅（存在） | 3 个 `h5_*` 测试（`tests.rs:8-108`） |
| 符号解析测试 | `crates/star-treesitter/src/symbol_resolver_tests.rs` | ✅（存在） | 4 个 `h7_*` 测试（`symbol_resolver_tests.rs:8-98`）；文件头注释写为"3 e2e tests"，实际为 4 个 |
| Task↔Worktree 绑定与 react-flow JSON | `crates/star-taskgraph/src/lib.rs` | 🟡 | `lib.rs:173-202`（bind），`lib.rs:214-271`（to_react_flow）；无符号节点，无持久化，无 HTTP 暴露 |
| 任务绑定测试 | `crates/star-taskgraph/src/tests.rs` | ✅（存在） | 4 个 `h6_*` 测试（`tests.rs:9-81`） |
| worktree 列表解析 | `crates/git-adapter/src/cli_provider.rs` | ✅ | `cli_provider.rs:113-150`；生产调用者 `crates/worktree-shared-dir/src/external_worktree_import.rs:718-722` |
| diff 数值统计 | `crates/git-adapter/src/cli_provider.rs` | 🟡 | `cli_provider.rs:206-236`（`--numstat`）；无 A/M/D；无调用者 |
| worktree 关系图模型 | `crates/graph-core/src/` | 🟡 | `node.rs:328-365`（FileNode、SymbolNode）；`edge.rs:64-91`（EdgeKind，含 ModifiesSymbol 但标 P2）；Neo4j 适配器不连接真实库 |
| 布局算法 | `crates/layout-engine/src/lib.rs` | 🟡 | 3 种算法与阈值选择（`lib.rs:1-25`）；未接入图视图 |
| 相邻 MCP 代码搜索工具 | `crates/star-mcp/src/tools/search_code.rs`、`find_references.rs`、`get_symbol.rs`、`get_code_context.rs` | 🟡 | `search_code.rs:12`（"paths 忽略... 真实路径过滤 P2 接入 tree-sitter"）；底层为 `domain-search::InMemorySearchService`，`crates/domain-search/src/lib.rs:845-848`（注释："Tree-sitter / ripgrep 接入在 P2"）；行号为启发式估算（`lib.rs:856-862`）；无生产索引写入者 |
| 相邻 Cargo 工作区注册 | `Cargo.toml:99`（star-treesitter）、`Cargo.toml:101`（star-taskgraph） | ✅ | 已注册为 workspace 成员 |
| 图服务进程 | 无 `graph-service` crate | ⬜ | 全仓无匹配 |
| 图 HTTP 路由 | 无 `/api/graph/:task_id` | ⬜ | `crates/star-api-rest/src/routes/` 无 graph 路由 |
| 任务卡 Graph 标签 | 无 | ⬜ | `frontend/src/components/board/WorkItemDetailDrawer.tsx` 无 graph 入口 |
| 图页面 `/graph/<task-id>` | 无 | ⬜ | `frontend/src/app/` 无 `graph/` 目录 |
| 图前端组件 `components/graph/*`、`lib/graph/*`、`hooks/useGraphData.ts` | 无 | ⬜ | `git ls-tree -r --name-only 637ed19d frontend/src` 无此类路径（`frontend/src/lib/graph` 不存在） |
| 源码 `@cypher` 结构头注释 | 541 个 `frontend/src` 与 `crates` 文件，以及 `scripts/automation/agent_runtime/*.py` 等 | 🟡 | 例：`scripts/automation/agent_runtime/backpressure.py` 第 3–4 行。文件内容为 `MERGE (self:File ...)` 形式的结构描述，仓内未检索到消费该头注释的解析器或生成器（`git grep -l source_sha256` 仅命中 `.github/workflows` 三个工作流与 `scripts/automation/` 中的注释头）。`.codegraph/.gitignore`（仓内唯一的 CodeGraph 痕迹）表明 CodeGraph 本地索引不入库 |

**说明**：`@cypher` 注释与 `.codegraph/` 都不是本功能的实现，但与"代码结构图谱"关键词相关，故列入。仓内既有 briefs（如 `docs/briefs/D19-current-profile-endpoint.md:14`）引用了外部 CodeGraph MCP 工具作为查询来源，该工具不属于本仓代码。

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 无文件遍历、无源码引用抽取 | 图只能对调用方给出的字符串做符号提取，无法覆盖 worktree 全量 | 在 `star-treesitter` 增加 `walk_worktree(path)`（遵守 NFR-S-01/02/03）；为 call/import 增加 tree-sitter query 抽取，输出 `SymbolReference` |
| 无图服务进程与 HTTP 接口 | 前端与 MCP 无法获取图 JSON | 新建 `crates/graph-service`（独立二进制，不进 22 个 domain crate 编译链，见 02 C-21/C-22）；端口冲突需先决定：8090 已被 `star-ops` 使用（`crates/star-ops/src/main.rs:18`） |
| 无 diff 叠加；git-adapter 的 diff 不含状态 | FR-47-07 与 UC-04 不可用 | 将 `git diff --numstat` 改为 `--name-status` 或新增方法返回 A/M/D；在 graph 组装后注入 `_diff` |
| 无缓存 | 1000 文件 worktree 每次全量 parse，NFR-P-02 不可达 | 实现 `(worktree_path, head_commit, grammar_set, schema_version)` 键的内存缓存，TTL 与 LRU 按 02 §3.4 |
| 任务卡模型分歧：`Option<String>`（1:1，`star-taskgraph`）与 `Option<Uuid>`（N:1，`WorkItem`） | F-01 "必填 + base_commit" 无法落地；两套绑定可能不一致 | 与 DDD Review 拍板 1:1 或 N:1，统一到一个模型；补充 `base_commit`（G-04） |
| 前端无 react-flow，且已有 cytoscape | G-07 未决；设计假设与代码不一致 | 决策 react-flow 或 cytoscape；若选 cytoscape，与 `ArchGraphModal` 共用依赖 |
| 无 `/graph/<task-id>` 页面与任务卡标签 | 用户无入口 | 在 `frontend/src/app/` 增加路由；在 `WorkItemDetailDrawer.tsx` 增加标签与跳转 |
| 无 secret 扫描、无大小限制、无审计 | 违反 NFR-S-02/S-05/S-06 与守门 #5/#13 | 接入 secret 扫描（可复用 `looks_like_secret_v12` 的规则，需迁移到共享位置）；在 graph 查询路径写审计 |
| `ParseError` 变体未使用，`error_count` 为布尔近似 | 无法向前端展示部分失败（NFR-A-02、UC-09） | 解析循环中收集每个文件的错误并输出 `parse_errors` 列表 |
| TypeScript 仅支持非 TSX；Python/Go 未在图中使用 | MVP 语言范围与前端 TSX 文件不匹配 | 增加 `LANGUAGE_TSX` 映射；按 F-16 扩展 |
| `TaskCard.symbols` 字段存在但从未填充 | 结构误导，表示"已有符号数据"但实际为空 | 在解析后填充或删除该字段 |
| 设计版本不一致：`tree-sitter 0.25` / `tree-sitter-rust 0.23`（02 §1.1）vs 代码 `0.27` / `0.24`（`star-treesitter/Cargo.toml:17-18`） | R-01 锁版本策略需以代码为准 | 更新 02 §1.1 与 C-23 记录的版本，或将代码降级（代码为准） |
| 文件头注释与测试数量不符（"3 e2e tests" 实为 4 个） | 文档偏差 | 修正注释，计数以代码为准 |
| `star-taskgraph` 的 `to_react_flow` 布局为固定网格（`lib.rs:225, 242`），设计为 dagre | 布局与设计不符 | 接入 `layout-engine` 的 dagre 算法 |
| 图视图依赖的文件会话与上下文组装（FR-CONTEXT-01）假设 `treesitter_worktree_graph::GraphService` | F12 FR-12-18 阻塞 | 按 `star-treesitter` + `graph-service` 的实际接口更新 `02-implementation-plan.md:180` 的引用 |

## 7. 验收与测试

**现有测试（静态核对，未执行）**

| 测试 | 位置 | 覆盖 |
|---|---|---|
| `h5_parse_rust_extracts_symbols` / `h5_parse_typescript_extracts_symbols` / `h5_language_from_str_validation` | `crates/star-treesitter/src/tests.rs:8-108` | Rust struct/enum/fn、TypeScript interface/function、语言字符串解析与错误分支 |
| `h7_symbol_reference_parse` / `h7_symbol_index_add_and_lookup` / `h7_resolve_references_cross_file` / `h7_cross_file_lookup` | `crates/star-treesitter/src/symbol_resolver_tests.rs:8-98` | 引用拆分、索引增删查、跨文件解析（含未解析分支）、全局查找 |
| `h6_task_card_bind_worktree` / `h6_task_card_double_bind_error` / `h6_task_graph_bidirectional_binding` / `h6_task_graph_react_flow_render` | `crates/star-taskgraph/src/tests.rs:8-81` | 任务卡绑定、重复绑定冲突、双向绑定、react-flow JSON 形状 |
| graph-core 的 `serde_and_mapping`、`repository_behavior` 测试 | `crates/graph-core/tests/` | worktree 图模型的序列化与内存仓储（与本功能相邻，不针对符号图） |

**已验证（静态）**：上述测试存在且断言的是上表所列行为；未检查其在 637ed19d 上的通过状态。

**未验证或缺失**

- 测试通过情况：❓ 未核实。需在 637ed19d 上执行 `cargo test -p star-treesitter -p star-taskgraph`（以确认 H.5/H.6/H.7 的通过记录）。
- 无 Python、Go、JSON 解析的断言（`tests.rs` 仅覆盖 Rust 与 TypeScript）。
- 无 worktree 遍历、缓存、diff 叠加、HTTP、MCP、前端的任何测试（对应 FR-47-04、07、08、09、14、15 等为 ⬜，无测试）。
- 无解析性能基准（NFR-P-01 ≤ 30s、NFR-P-03 ≤ 2s 等）。
- 无 `oversized` 与 secret 扫描的测试。
- 设计的验收条件（E-01 diff 覆盖率 100%、E-04 调用解析准确率 ≥ 90%）均无度量。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/architecture/2026-09-03-treesitter-worktree-graph/README.md`（seed） | 无 | 文件在 637ed19d 不存在（`git show` 返回 `does not exist`） | 不存在，已记入 open_items；未用其他文件替代为"README" |
| `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` | 目标、业务课题、F-01..F-20、NFR、约束 C-01..C-21、G-01..G-07、UC-01..UC-09 | §1.1 称"Cargo.toml 全仓 0 tree-sitter 引用"，代码已引入（`Cargo.toml:99`、`star-treesitter/Cargo.toml`），此陈述已过时；§1.1 称 domain-worktree 为 Cargo 依赖，未在本文档核实 | 以代码为准：tree-sitter 已引入，worktree 图未实现 |
| `docs/architecture/2026-09-03-treesitter-worktree-graph/02-basic-design.md` | 组件、接口、数据模型、流程、缓存、安全、UI 布局、M1–M8 里程碑 | §1.1 与 §3 的 tree-sitter 版本与代码不符（0.25/0.23 vs 0.27/0.24）；§2.1 将图服务定为独立二进制（端口 8090）而代码中 8090 被 `star-ops` 占用；§6.4 布局为 dagre，代码为固定网格；§2.2 与 §6.5 的 react-flow 无依赖 | 设计意图保留（独立进程、缓存、安全约束）；版本与端口冲突以代码/现状为准并列入 §6 |
| `crates/star-treesitter/**`（代码） | 解析与符号提取、符号解析内存索引 | 与 01 §1.4 INV-TS-01..04 对照：INV-TS-02 通过返回 Err 满足（`lib.rs:156-158`），但 `ParseError` 变体未被构造 | 作为 🟡 现状依据 |
| `crates/star-taskgraph/**`（代码） | 任务绑定、react-flow JSON | INV-TG-02（节点含 symbol）未满足：`to_react_flow` 不输出符号节点（`lib.rs:214-271`） | 作为 🟡 现状依据 |
| `docs/features/F12-file-session.md` | FR-12-18 与依赖表（确认 treesitter-worktree-graph crate 不存在） | 与本次核对一致（`F12-file-session.md:60, 197`） | 采纳其事实陈述作为相邻依据 |
| `docs/features/F07-worktree-canvas-relations.md` | 指出 V3 不做 AST 解析（`v3_symbol_overlap.rs:25`） | 与代码一致（risk-engine 只消费调用方提供的 `shared_symbols`） | 作为相邻依据 |
| `docs/architecture/file-session-management/02-implementation-plan.md` | ContextBuilder 依赖 `GraphService` | 该类型不存在 | 列为下游依赖缺口（§4.6、§6） |
| `docs/architecture/file-session-management/01-requirements.md` | FR-CONTEXT-01 复用 treesitter-worktree-graph | 与 F12 一致，依赖未满足 | 作为下游依赖（§4.6） |
| `docs/architecture/2026-08-26-upgrade/adr/0052-file-session-management.md` | 符号索引依赖 treesitter-worktree-graph（:199, :600） | 同上 | 仅作为引用来源，未深入 |
| `docs/design/BD-WORKTREE-CANVAS-001.md`、`docs/design/DD-WORKTREE-CANVAS-001.md`、`docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` | V3 Symbol Overlap 需 tree-sitter（P2） | 与代码一致（V3 未接入 tree-sitter） | 作为相邻设计依据，未深入 |
| `docs/ecosystem-survey/protocol-survey.md` 第 5 节 | tree-sitter 作为 Code Intelligence 可选底层，MVP 用 ripgrep + 简单索引 | 与 `domain-search` 注释（P2 接入）一致 | 仅作为背景 |
| `docs/briefs/OPT-NEXT-05-phase-h.md`、`docs/briefs/OPT-A2-wbs-pending-scan.output.md`、`docs/briefs/OPT-A4-quality-debt-scan.output.md` | H.5–H.7 状态与"7/7 test pass"记录，以及 `treesitter_init.py`、`task_graph_view.py`、`symbol_resolver.py` 脚本名 | 三个脚本在 637ed19d 的 `scripts/` 下不存在（`git ls-tree` 无匹配）；"7/7 pass"为历史记录，未在本次执行 | 脚本引用不采纳（不存在）；测试通过记为 ❓ |
| `Cargo.toml:98-101`（注释与成员） | 注册说明（H.5 为 5 语言；H.6 为 react-flow） | 注释称 H.5 "5 语言"，`star-treesitter/Cargo.toml:10` 称 "Rust + 4 语言" | 以 `lib.rs:38-51` 的枚举为准（5 种） |
| `crates/star-mcp/src/tools/search_code.rs` 等 4 个工具 | 相邻代码搜索 MCP 工具 | 依赖 in-memory 搜索，行号为启发式 | 列为相邻实现，不属本功能 |
| `crates/domain-search/src/lib.rs` | 注释声明 tree-sitter 在 P2 接入 | 与代码一致 | 作为相邻依据 |
| `crates/graph-core/**` | worktree 图模型（含 SymbolNode、MODIFIES_SYMBOL） | 设计文档未引用 graph-core；无生产者 | 作为相邻实现，记为 🟡 |
| `crates/layout-engine/**` | dagre/d3-force/elk 选择器 | 图视图未接入 | 作为相邻实现，记为 🟡 |
| `frontend/src/app/canvas/worktree-graph/page.tsx` | worktree 关系图页面 | 与本功能的图路由不同，无符号节点 | 不属本功能，作为相邻页面列出 |
| `frontend/src/components/board/ArchGraphModal.tsx`、`frontend/src/mocks/handlers/graph.ts`、`frontend/src/types/graph.ts` | 架构图 cytoscape 查看器（ADR-0041）与 `/api/graph/*` mock | 端点与本设计 `/api/graph/:task_id` 不同，仅 mock，无后端 | 不属本功能，作为相邻实现列出；不采纳其端点 |
| `frontend/package.json` | 依赖清单 | 无 react-flow、dagre | 证实 FR-47-09 ⬜ |
| `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` 之外的 `AGENTS.md` 守门引用（#5、#11、#13、#19、#22、#23、#24） | 守门约束 | 仅作为约束来源，本次未逐条核对守门是否满足 | 作为约束来源引用，未验证 |

## Open items

- seed 文件 `docs/architecture/2026-09-03-treesitter-worktree-graph/README.md` 在 637ed19d 不存在。本文档以 `01-requirements.md` 与 `02-basic-design.md` 为设计来源。
- 测试通过情况未核实（需在 637ed19d 执行 `cargo test -p star-treesitter -p star-taskgraph`）。
- `domain-worktree` 与 `star-audit` 的接口关系未核实（设计文档引用，本次未在 637ed19d 检索到 star-audit 名称）。
- `@cypher` 头注释的生成方式未确定：仓内未找到生成器或解析器，`.codegraph/` 仅有 `.gitignore`。
- 设计中 `graph-service` 的端口 8090 与 `crates/star-ops` 冲突，需要拍板。
