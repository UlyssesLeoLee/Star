# 02. Tree-sitter Worktree Graph - 基本設計書 (Basic Design)

> **状態**：🟡 Draft v0.1
> **日期**：2026-09-03
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权）
> **依赖**：[01-requirements.md](01-requirements.md)（要件定義書）· [ADR-0026 STAR AI 兼容](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md) · [ADR-0032 MCP Transport stdio](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [docs/architecture/2026-09-03-langgraph/02-basic-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-09-03-langgraph/02-basic-design.md)
> **关联文档**：[01-requirements.md](01-requirements.md)（要件定義書）

---

## 0. 目的 (Purpose)

本文档基于 [01-requirements.md](01-requirements.md) §1-§3 的要件，定义 **Tree-sitter Worktree Graph** 的基本設計：

- 系统架构 (3-tier: parser service / graph builder / frontend view)
- コンポーネント一覧 + 责任划分
- 数据模型 (Node/Edge schema, Graph JSON, Diff overlay, Cache key)
- 内部/外部 API (HTTP REST + WebSocket)
- データフロー (worktree → parse → graph → diff overlay → render)
- UI/UX 設計 (react-flow 布局 + 节点样式 + 交互)
- セキュリティ/性能/運用/移行

> **重要范围 (per 2026-09-03 用户决策)**:
> - "所处 worktree" = 任务卡关联的 git worktree literal
> - "任务卡修改的内容呈现在那个 worktree 里面的图论构造" = task diff (add/modify/delete) 作为 graph node/edge 上的 visual overlay
> - 视图形态 = 独立新 view, 任务卡里只放跳转入口

## 1. システムアーキテクチャ (System Architecture)

### 1.1 全体構成図 (Overall Architecture)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       UI Tier (gm-console frontend, Next.js)                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Kanban Board (现有, per LangGraph 02 §1.1)                          │   │
│  │    └──> Task Card Modal (现有)                                       │   │
│  │          ├──> Tab 1 Overview (现有)                                   │   │
│  │          ├──> Tab 2 Discussion (现有)                                 │   │
│  │          ├──> Tab N ★ Graph (新入口, F-09)                            │   │
│  │          │     ┌──────────────────────────────────────────────────┐  │   │
│  │          │     │  [Open Graph View → /graph/<task-id>]            │  │   │
│  │          │     │  Summary: 42 files, 8 modified, 2 added, 1 deleted │  │   │
│  │          │     └──────────────────────────────────────────────────┘  │   │
│  │          └──> ...                                                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  ★ NEW ★ Graph View (独立 route, per F-09)                            │   │
│  │    /graph/<task-id>                                                  │   │
│  │  ┌────────────────────────────────────────────────────────────────┐  │   │
│  │  │  Toolbar (h-12, top)                                            │  │   │
│  │  │    [worktree: wt-sub-session-001] [commit: a1b2c3]              │  │   │
│  │  │    [Refresh] [Fit] [Filter ▼] [Search...] [Export]               │  │   │
│  │  ├────────────────────────────────────────────────────────────────┤  │   │
│  │  │  Graph Canvas (react-flow, flex-1)                               │  │   │
│  │  │    • 节点: file (square) / function (circle) /                   │  │   │
│  │  │            class (hexagon) / struct (hexagon-stripe) /            │  │   │
│  │  │            const (diamond)                                       │  │   │
│  │  │    • 边:   import (gray solid) / call (blue solid) /             │  │   │
│  │  │            contain (black solid) / reference (purple dashed)      │  │   │
│  │  │    • Overlay color:                                              │  │   │
│  │  │        - added = green border + green tint                        │  │   │
│  │  │        - modified = orange border + orange tint                  │  │   │
│  │  │        - deleted = red strike-through (灰色虚化)                  │  │   │
│  │  │    • Layout: dagre (top-to-bottom) by default, d3-force optional  │  │   │
│  │  ├────────────────────────────────────────────────────────────────┤  │   │
│  │  │  Side Panel (w-80, right, collapsible)                           │  │   │
│  │  │    • Node details (type, path, line range, signature)            │  │   │
│  │  │    • Code preview (前 50 lines, syntax highlight via Shiki)       │  │   │
│  │  │    • Actions: [Open in IDE] [Copy path] [Follow refs]            │  │   │
│  │  └────────────────────────────────────────────────────────────────┘  │   │
│  │  Status Bar (h-8, bottom)                                              │   │
│  │    [nodes: 1247] [edges: 3891] [parse: 28.4s] [cache: HIT/MISS]      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│   │ HTTP REST (control)              │ WebSocket (optional, progress)        │
└───┼──────────────────────────────────┼──────────────────────────────────────┘
    │                                  │
    ▼                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                  Backend Tier (graph-service, 独立 Rust binary)              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  graph-service (Rust binary, per C-21 独立进程不进 main 编译链)        │   │
│  │  ┌──────────────────────────────────────────────────────────────┐    │   │
│  │  │  HTTP Server (axum 0.7+, port 8090)                            │    │   │
│  │  │    • GET  /api/graph/<task-id>      → graph JSON               │    │   │
│  │  │    • POST /api/graph/<task-id>/refresh → 强制 invalidate         │    │   │
│  │  │    • GET  /api/graph/<task-id>/status  → parse status (json)   │    │   │
│  │  │    • GET  /api/worktrees             → worktree 列表            │    │   │
│  │  │    • GET  /api/health                → health check            │    │   │
│  │  └──────────────────────────────────────────────────────────────┘    │   │
│  │                                                                       │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌──────────────────────┐    │   │
│  │  │ Worktree       │  │ TreeSitter     │  │ Symbol               │    │   │
│  │  │ Resolver       │  │ Parser Service │  │ Resolver             │    │   │
│  │  │                │  │                │  │                      │    │   │
│  │  │ - git worktree │  │ - tree-sitter  │  │ - cross-file ref     │    │   │
│  │  │   list parse   │  │   0.25+        │  │   tracker            │    │   │
│  │  │ - path resolve │  │ - grammar reg  │  │ - scope chain        │    │   │
│  │  │ - HEAD detect  │  │   (Rust + TS)  │  │ - import graph       │    │   │
│  │  │ - path safety  │  │ - file walk    │  │ - call graph (intra) │    │   │
│  │  │   (NFR-S-01)   │  │ - secret scan  │  │                      │    │   │
│  │  └────────────────┘  │ - size limit   │  └──────────────────────┘    │   │
│  │                       │   (NFR-S-02)  │                                │   │
│  │                       └────────────────┘                                │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌──────────────────────┐    │   │
│  │  │ Graph Builder  │  │ Diff Overlay   │  │ Cache Layer          │    │   │
│  │  │                │  │                │  │                      │    │   │
│  │  │ - AST → nodes  │  │ - git diff     │  │ - LRU per (wt, sha)  │    │   │
│  │  │ - scope →      │  │   parse        │  │ - in-memory hashmap  │    │   │
│  │  │   edges        │  │ - file → node  │  │ - TTL 1h             │    │   │
│  │  │ - dedup        │  │   mapping      │  │ - size cap 500MB     │    │   │
│  │  │ - serialize    │  │ - node _diff   │  │ - metrics: hit/miss  │    │   │
│  │  │   (JSON)       │  │   field inject │  │                      │    │   │
│  │  └────────────────┘  └────────────────┘  └──────────────────────┘    │   │
│  │  ┌──────────────────────────────────────────────────────────────┐    │   │
│  │  │  AuditLogger (per 守门 #13 W/T/M, 全量 query 记录)               │    │   │
│  │  │    写到 star-audit 共享库 (跟 16 tools 一致)                      │    │   │
│  │  └──────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
    │ tree-sitter grammar deps        │ git CLI (subprocess)            │ file IO
    ▼                                  ▼                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│              Worktree Tier (filesystem + git, 现有资源)                      │
│  D:\Star\.worktrees\wt-sub-session-001\                                    │
│  D:\Star\.worktrees\wt-nav-i18n-a\                                          │
│  D:\Star\.worktrees\wt-nav-shots-b\                                         │
│  D:\Star\ (main worktree)                                                   │
│  Cargo deps: tree-sitter 0.25, tree-sitter-rust 0.23, tree-sitter-typescript 0.23│
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 进程拓扑 (Process Topology)

per AGENTS.md §4.1 v22 派生规 (守门 #22 调试控制台不污染 main 编译) + 本设计 C-21 约束:

| 进程 | 类型 | 启动 | 端口 | 依赖 |
|---|---|---|---|---|
| `graph-service` | 独立 Rust binary | 手动启动 / systemd | 8090 | tree-sitter, git CLI |
| `gm-console frontend` | Next.js dev / build | `pnpm dev` | 3000 | graph-service HTTP |
| `star-mcp` | Rust binary (现有) | Mavis 启动 | stdio | 16 tools (+ F-19 17) |

**关键**: `graph-service` 是独立二进制, **不进 main 编译链**, **不进 star-mcp 编译链**。`cargo check --workspace --all-targets` 不会触发 graph-service 重编, 守门 #1 v1-v12 不被污染。

### 1.3 Cargo workspace 影响

per 守门 #1 v1 + C-21:

- graph-service 单独 crate: `crates/graph-service/Cargo.toml`
- tree-sitter + grammar deps **仅在 graph-service Cargo.toml**
- 主 workspace `Cargo.toml` 不加 tree-sitter (避免污染 22 domain-* crate 编译链)
- 守门 #1 v1-v12 守门检查: `cargo check --workspace --all-targets` 应仍 0 err, 41+ crate 守门覆盖
- 守门 #1 v6 (release mode test 100% pass) 保持

## 2. コンポーネント (Components)

### 2.1 graph-service (Rust binary)

**责任**: 接收 HTTP 请求, 解析 worktree 代码, 返回 graph JSON。

| 模块 | 责任 | 关键 API |
|---|---|---|
| `server` | axum HTTP server, route 注册 | `Router::new().route("/api/graph/:task_id", get(get_graph))` |
| `worktree_resolver` | task_id → worktree path, path safety 校验 | `WorktreeResolver::resolve(task_id) -> Result<PathBuf>` |
| `parser` | tree-sitter 多 grammar 注册 + 文件遍历 | `ParserService::parse_file(path, grammar) -> Result<Tree>` |
| `symbol_resolver` | 跨文件 call/import/reference 追踪 | `SymbolResolver::resolve(ast, scope) -> Vec<Edge>` |
| `graph_builder` | AST + symbols → nodes/edges JSON | `GraphBuilder::build(parsed) -> Graph` |
| `diff_overlay` | git diff → node _diff 字段 | `DiffOverlay::overlay(graph, base_commit, head_commit)` |
| `cache` | LRU per (worktree, commit), TTL 1h | `GraphCache::get_or_compute(key, closure)` |
| `audit` | per 守门 #13 W/T/M, 全量 query 记录 | `AuditLogger::log(event)` |
| `secret_scanner` | parse 输出前扫 secret (per NFR-S-06) | `SecretScanner::scan(content) -> Result<()>` |

### 2.2 frontend graph view (Next.js + react-flow)

**责任**: 渲染 graph canvas, 节点交互, side panel, search/filter。

| 模块 | 责任 | 路径 |
|---|---|---|
| `app/graph/[taskId]/page.tsx` | Next.js route, 加载 graph | `frontend/src/app/graph/[taskId]/page.tsx` |
| `components/graph/Canvas.tsx` | react-flow 容器, 节点/边渲染 | `frontend/src/components/graph/Canvas.tsx` |
| `components/graph/Toolbar.tsx` | 顶部工具栏 (refresh/fit/filter/search) | `frontend/src/components/graph/Toolbar.tsx` |
| `components/graph/SidePanel.tsx` | 节点详情 side panel | `frontend/src/components/graph/SidePanel.tsx` |
| `components/graph/StatusBar.tsx` | 底部状态栏 (节点/边/parse 耗时) | `frontend/src/components/graph/StatusBar.tsx` |
| `lib/graph/loader.ts` | graph-service HTTP client | `frontend/src/lib/graph/loader.ts` |
| `lib/graph/layout.ts` | dagre / d3-force layout 算法 | `frontend/src/lib/graph/layout.ts` |
| `lib/graph/diff-overlay.ts` | 节点 _diff → 颜色/边框映射 | `frontend/src/lib/graph/diff-overlay.ts` |
| `lib/graph/search.ts` | 客户端搜索/过滤 | `frontend/src/lib/graph/search.ts` |
| `hooks/useGraphData.ts` | React hook, 数据加载 + 缓存 | `frontend/src/hooks/useGraphData.ts` |

### 2.3 Kanban 任务卡扩展 (现有 + 新增)

**责任**: 任务卡 schema 加 worktree_id 字段 + Graph tab 入口。

| 改动 | 责任模块 |
|---|---|
| `task.schema.json` 加 `worktree_id: String` 必填字段 | `frontend/src/types/task.ts` |
| Task Card Modal 加 Graph tab | `frontend/src/components/kanban/TaskCardModal.tsx` |
| Graph tab 内容: 跳转入口 + diff summary | `frontend/src/components/kanban/GraphTabEntry.tsx` |

### 2.4 依赖关系图 (Component Dependency)

```
graph-service:
  worktree_resolver ──> git CLI subprocess
  parser ──> tree-sitter + tree-sitter-rust/-typescript
  symbol_resolver ──> parser (AST 输入)
  graph_builder ──> parser + symbol_resolver
  diff_overlay ──> git CLI subprocess + graph_builder
  cache ──> graph_builder + diff_overlay
  server ──> cache + audit
  audit ──> star-audit 共享库 (跨 crate)

frontend graph view:
  page ──> loader (HTTP client)
  Canvas ──> loader + layout + diff-overlay
  Toolbar ──> loader (refresh action)
  SidePanel ──> loader (node details lazy load)
  search ──> Canvas (filter state)
  hooks/useGraphData ──> loader

Kanban:
  TaskCardModal ──> GraphTabEntry (跳转 /graph/<task-id>)
```

## 3. データモデル (Data Model)

### 3.1 Node Schema (Graph Node)

```rust
// crates/graph-service/src/schema/node.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// 文件节点 (root level)
    File {
        path: String,              // relative to worktree root, e.g. "crates/domain-worktree/src/lib.rs"
        language: String,          // "rust" | "typescript" | "tsx" | ...
        size_bytes: u64,
        line_count: u32,
    },
    /// 函数节点 (Rust fn, TS function, etc.)
    Function {
        name: String,
        signature: String,         // "fn foo(x: i32) -> Result<String, Error>"
        visibility: Visibility,    // Public | Private | Crate
        is_async: bool,
        is_unsafe: bool,
    },
    /// 类 / Struct / Interface
    Class {
        name: String,
        kind: ClassKind,           // Struct | Class | Interface | Trait | Enum
        visibility: Visibility,
    },
    /// 常量 / 静态变量
    Const {
        name: String,
        const_kind: ConstKind,     // Const | Static | Let
        value_preview: String,     // 前 30 字符
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,                // 唯一 ID, e.g. "file:crates/domain-worktree/src/lib.rs" or "fn:domain-worktree::Worktree::create"
    pub kind: NodeKind,
    pub file_path: String,         // 父文件路径
    pub line_start: u32,           // 起始行 (1-indexed)
    pub line_end: u32,             // 结束行
    pub col_start: u32,
    pub col_end: u32,
    /// Diff overlay (per F-07 + UC-04)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<DiffMark>,
    /// Symbol 上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,    // 父节点 ID (e.g. method 的 class)
    pub children: Vec<String>,     // 子节点 ID 列表
    /// 解析元数据
    pub parse_status: ParseStatus, // Ok | Partial | Failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffMark {
    Added,                        // git status: A
    Modified,                     // git status: M
    Deleted,                      // git status: D
    Renamed { from: String },     // git status: R
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    Ok,
    Partial,                      // 部分解析 (per UC-09 降级)
    Failed,
}
```

### 3.2 Edge Schema (Graph Edge)

```rust
// crates/graph-service/src/schema/edge.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EdgeKind {
    /// 父子包含 (function 在 file 内, method 在 class 内)
    Contain,
    /// 函数调用 / 方法调用
    Call {
        call_site_line: u32,
        is_async: bool,
    },
    /// 模块导入 (use / import / require)
    Import {
        import_path: String,       // 原始 import 路径
        alias: Option<String>,     // 别名
        is_reexport: bool,
    },
    /// 符号引用 (类型引用, 字段访问)
    Reference {
        reference_kind: ReferenceKind, // TypeRef | FieldAccess | TraitBound
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,                // 唯一 ID, e.g. "edge:src_id->dst_id:call"
    pub kind: EdgeKind,
    pub source: String,            // source node ID
    pub target: String,            // target node ID
    /// Diff overlay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<DiffMark>,
    /// 解析元数据
    pub confidence: f32,           // 0.0 - 1.0, symbol resolver 置信度
    pub parse_status: ParseStatus,
}
```

### 3.3 Graph JSON (Top-level Schema)

```rust
// crates/graph-service/src/schema/graph.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub version: String,           // "1.0.0", schema 版本
    pub metadata: GraphMetadata,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// 解析错误汇总 (per UC-09 降级可见)
    pub parse_errors: Vec<ParseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub worktree_path: String,     // 绝对路径
    pub worktree_id: String,       // git worktree 名称 (basename)
    pub task_id: String,           // 任务卡 ID
    pub base_commit: String,       // 任务卡基准 commit
    pub head_commit: String,       // worktree HEAD commit
    pub generated_at: String,      // ISO 8601
    pub parse_duration_ms: u64,    // parse 总耗时
    pub grammar_used: Vec<String>, // ["rust", "typescript"]
    pub cache_status: CacheStatus, // Hit | Miss | Refresh
    pub total_files: u32,          // worktree 内源码文件总数
    pub parsed_files: u32,         // 成功 parse 的文件数
    pub failed_files: u32,         // parse 失败的文件数
    pub oversized_files: u32,      // > 5MB 跳过的文件数
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheStatus {
    Hit,                          // cache 命中
    Miss,                         // cache 未命中, 重新 parse
    Refresh,                      // 强制 refresh
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub file_path: String,
    pub error: String,
    pub line: Option<u32>,
    pub parse_status: ParseStatus,
}
```

### 3.4 Cache Key Schema

```rust
// crates/graph-service/src/cache/mod.rs

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub worktree_path: PathBuf,   // 绝对路径
    pub head_commit: String,      // git rev-parse HEAD
    pub grammar_set: String,      // 排序拼接, e.g. "rust+typescript"
    pub schema_version: String,   // 当前 schema 版本, schema 变更自动 invalidate
}
```

**Cache 策略**:
- LRU, max 500MB
- TTL 1h
- Key: `(worktree_path, head_commit, grammar_set, schema_version)`
- 失效触发:
  - TTL 到期
  - worktree HEAD commit 变化
  - grammar 集变化
  - schema_version bump (e.g. 1.0.0 → 1.1.0)
  - 用户主动 refresh (POST /api/graph/.../refresh)

### 3.5 Task Card Schema Extension (per F-01)

```typescript
// frontend/src/types/task.ts

export interface TaskCard {
  id: string;
  title: string;
  description: string;
  status: 'backlog' | 'in_progress' | 'review' | 'done';
  assignee: string;
  // ★ NEW ★ 必填字段, per 01-requirements §F-01
  worktree_id: string;           // git worktree basename, e.g. "wt-sub-session-001"
  base_commit?: string;          // 任务卡创建时 worktree HEAD, 后续 diff 基准
  created_at: string;
  updated_at: string;
  // ... 其他现有字段
}
```

**DB 分类 (per 守门 #13 W/T/M)**:
- `worktree_id` 属 **Work** 类 (session-bound, 任务卡生命周期内有效, 任务卡完结后可清空)
- `base_commit` 属 **Work** 类 (短期 reference, 任务卡完结后可归档)
- 两者均不强制永久保留, **缺标比错标安全** (per 守门 #11)

## 4. API 設計 (API Design)

### 4.1 外部 API (HTTP REST, graph-service :8090)

| Method | Path | 描述 | 请求体 | 响应 |
|---|---|---|---|---|
| `GET` | `/api/health` | health check | — | `{"status": "ok", "version": "0.1.0"}` |
| `GET` | `/api/worktrees` | 列出所有 git worktree | — | `Worktree[]` (per §4.3) |
| `GET` | `/api/worktrees/:wt_id` | 单个 worktree 详情 | — | `Worktree` |
| `GET` | `/api/graph/:task_id` | 任务卡 graph (per F-06) | query: `?base_commit=<sha>` (可选, 默认 task.base_commit) | `Graph` |
| `GET` | `/api/graph/:task_id/status` | graph parse status (for long-running) | — | `{"status": "pending\|running\|done\|failed", "progress": 0.42, "nodes_so_far": 523}` |
| `POST` | `/api/graph/:task_id/refresh` | 强制 refresh (per UC-08) | — | `{"cache_invalidated": true, "parse_started_at": "..."}` |
| `GET` | `/api/graph/:task_id/node/:node_id` | 节点代码预览 (per UC-03) | — | `{"node": Node, "code": "..."}` (前 50 行, syntax highlighted) |

### 4.2 内部 API (graph-service 内部模块)

| Function | 模块 | 签名 | 说明 |
|---|---|---|---|
| `parse_file` | `parser` | `fn(path: &Path, grammar: Grammar) -> Result<Tree>` | tree-sitter 解析单文件 |
| `extract_nodes` | `parser` | `fn(tree: &Tree, file_path: &Path) -> Vec<Node>` | AST → 顶层节点抽取 |
| `resolve_symbols` | `symbol_resolver` | `fn(nodes: &[Node], worktree_path: &Path) -> Vec<Edge>` | 跨文件引用追踪 |
| `build_graph` | `graph_builder` | `fn(nodes: Vec<Node>, edges: Vec<Edge>) -> Graph` | 组装 Graph JSON |
| `apply_diff_overlay` | `diff_overlay` | `fn(graph: &mut Graph, base: &str, head: &str) -> Result<()>` | git diff → node._diff |
| `get_or_compute` | `cache` | `fn(key: &CacheKey, closure: F) -> Result<Arc<Graph>>` where F: FnOnce() -> Result<Graph> | 缓存读取或计算 |

### 4.3 Worktree Schema (返回 /api/worktrees)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    pub id: String,                // basename, e.g. "wt-sub-session-001"
    pub path: String,              // 绝对路径
    pub head_commit: String,       // git rev-parse HEAD
    pub head_short: String,        // 前 7 字符
    pub branch: Option<String>,    // 当前分支 (detached HEAD 时 None)
    pub is_main: bool,             // 是否主 worktree
    pub is_locked: bool,           // git worktree list --locked
    pub file_count: u32,           // 源码文件数 (cached)
    pub size_bytes: u64,           // worktree 大小 (cached)
}
```

### 4.4 MCP Tool 扩展 (per F-19, 16 → 17 tools)

```rust
// crates/star-mcp/src/tools/get_task_graph.rs (新)

pub struct GetTaskGraphTool;

impl McpTool for GetTaskGraphTool {
    fn name(&self) -> &'static str { "get_task_graph" }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "get_task_graph".to_string(),
            description: "Get code graph (AST + symbols + diff overlay) for a task card's worktree".to_string(),
            input: json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string", "description": "Task card ID" },
                    "refresh": { "type": "boolean", "default": false, "description": "Force refresh cache" }
                },
                "required": ["task_id"]
            }),
            output: json!({
                "type": "object",
                "properties": {
                    "graph": { "type": "object", "description": "Graph JSON" },
                    "cache_status": { "type": "string", "enum": ["hit", "miss", "refresh"] }
                }
            }),
        }
    }
    
    async fn execute(&self, args: Value) -> Result<Value> {
        let task_id = args["task_id"].as_str().unwrap();
        let refresh = args["refresh"].as_bool().unwrap_or(false);
        // HTTP call to graph-service
        let url = format!("http://localhost:8090/api/graph/{}{}", 
            task_id,
            if refresh { "?refresh=true" } else { "" }
        );
        let graph: Graph = reqwest::get(&url).await?.json().await?;
        Ok(json!({ "graph": graph, "cache_status": graph.metadata.cache_status }))
    }
}
```

**注册**: 在 `crates/star-mcp/src/main.rs` 加 `GetTaskGraphTool` 到 16 tools 列表末尾 → 17 tools。

## 5. データフロー (Data Flow)

### 5.1 主フロー: Graph 加载 (per UC-02)

```
[Frontend: /graph/<task-id>]
   │
   │ 1. GET /api/graph/<task-id>
   ▼
[graph-service: server]
   │
   │ 2. WorktreeResolver::resolve(task_id)
   │    - 查 task.worktree_id
   │    - git worktree list --porcelain 解析
   │    - path safety 校验 (NFR-S-01)
   │    - git rev-parse HEAD
   │    → (worktree_path, head_commit, base_commit)
   │
   │ 3. Cache::get(CacheKey { path, head, grammar, schema_version })
   │    │
   │    ├─ HIT → 直接返回 Graph JSON
   │    │
   │    └─ MISS → 进入 4
   │
   │ 4. ParserService::parse_worktree(worktree_path)
   │    - 遍历 src 目录 (跳过 .git/, target/, node_modules/, .next/, dist/)
   │    - 按扩展名选 grammar (.rs → Rust, .ts/.tsx → TypeScript)
   │    - SecretScanner::scan(file_content)  (NFR-S-06)
   │    - file size 检查 (NFR-S-02, > 5MB skip)
   │    - tree-sitter parse → Tree
   │    - Extract nodes (File, Function, Class, Const)
   │    → Vec<Node> (raw)
   │
   │ 5. SymbolResolver::resolve(nodes, worktree_path)
   │    - 跨文件 call/import/reference 追踪
   │    - scope chain 解析
   │    - import path → module path 解析
   │    → Vec<Edge> (raw)
   │
   │ 6. GraphBuilder::build(nodes, edges)
   │    - dedup (按 id)
   │    - parent/children 关联
   │    - metadata 注入
   │    → Graph (no diff yet)
   │
   │ 7. DiffOverlay::apply(graph, base_commit, head_commit)
   │    - git diff --name-status base..head
   │    - file → node id mapping
   │    - inject node._diff = Some(Added|Modified|Deleted|Renamed)
   │    - inject edge._diff (从 source/target node 继承)
   │    → Graph (with diff)
   │
   │ 8. Cache::put(key, graph)
   │
   │ 9. AuditLogger::log(event) (per 守门 #13)
   │
   │ 10. HTTP 200 + Graph JSON
   ▼
[Frontend]
   │
   │ 11. React: setState graph
   │ 12. Canvas: render nodes + edges via react-flow
   │ 13. layout.ts: dagre top-to-bottom
   │ 14. diff-overlay.ts: apply color/border
   ▼
[用户看到 graph]
```

### 5.2 Diff Overlay 详细流程 (per UC-04)

```
[GraphBuilder 完成后]
   │
   │ DiffOverlay::apply(graph, base, head)
   │
   │ 1. git diff --name-status <base>..<head>
   │    → Vec<{path, status, old_path?}>
   │    e.g. [
   │      { path: "crates/domain-worktree/src/lib.rs", status: 'M' },
   │      { path: "crates/domain-ai/src/predict.rs", status: 'A' },
   │      { path: "crates/old_module.rs", status: 'D' },
   │    ]
   │
   │ 2. File path → Node ID mapping
   │    - For each changed file:
   │      - find graph.nodes where kind=File and path matches
   │      - mark parent File node._diff
   │      - also mark all child nodes (function/class/const) of that file
   │      - for Deleted: mark File + children as deleted (visible但 strike-through)
   │      - for Renamed: map old_path → old node, mark renamed
   │
   │ 3. Edge._diff propagation
   │    - if source OR target node has _diff, mark edge._diff
   │
   │ 4. Return updated graph
   │
[Frontend rendering]
   │
   │ diff-overlay.ts: 
   │   - if node._diff = Added → border: 2px solid #22c55e, fill: #dcfce7
   │   - if node._diff = Modified → border: 2px solid #f59e0b, fill: #fef3c7
   │   - if node._diff = Deleted → border: 1px dashed #ef4444, opacity: 0.5, strike text
   │   - if node._diff = None → default (gray border, white fill)
```

### 5.3 Cache 失效流程 (per F-08 + NFR-A-04)

```
[Cache Layer 初始化/查询]
   │
   │ Cache::get(key)
   │
   │ 1. HashMap lookup
   │    │
   │    ├─ Hit → check TTL
   │    │     │
   │    │     ├─ TTL not expired → return Arc<Graph>
   │    │     │
   │    │     └─ TTL expired → remove, return None
   │    │
   │    └─ Miss → return None
   │
   │ 2. (Caller handles Miss: 重新 parse, 完成后 Cache::put)
   │
   │ Cache::put(key, graph)
   │
   │ 1. Size check: 当前 total_size + graph.size > 500MB?
   │    │
   │    ├─ Yes → LRU eviction until under cap
   │    │
   │    └─ No → insert
   │
   │ 2. Update LRU order
   │
   │ Cache::invalidate(pattern)
   │
   │ 1. By worktree_path → remove all entries with matching path
   │ 2. By commit → remove all entries with matching head_commit
   │ 3. By schema_version → remove all entries with old version
```

### 5.4 错误降级流程 (per UC-09)

```
[ParserService::parse_file]
   │
   │ tree-sitter parse(path)
   │
   ├─ Ok → Extract nodes → return Vec<Node>
   │
   ├─ Err (parse error) → return Err, log
   │
   └─ File size > 5MB → skip, mark oversized
       │
       └─ continue to next file
       │
       └─ add to parse_errors
       │
       └─ add to metadata.oversized_files

[GraphBuilder::build (collecting errors)]
   │
   │ 1. parse_files loop, collect all errors
   │ 2. continue with parsed files only
   │ 3. inject errors into graph.parse_errors
   │ 4. metadata.parsed_files + metadata.failed_files
   │
   ▼
[Frontend]
   │
   │ if graph.parse_errors.length > 0:
   │   - 显示 toast: "X 个文件解析失败"
   │   - 按钮 "View Errors" 打开 modal 列出
   │   - graph 仍正常渲染 (使用 parsed 部分)
```

## 6. UI/UX 設計 (UI/UX Design)

### 6.1 Graph View 页面布局 (per §1.1)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Toolbar (h-12, bg-slate-50, border-b)                              │
│ ┌──────────┐ ┌──────────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌────────┐    │
│ │ wt-name  │ │ a1b2c3   │ │ ↻    │ │ ⊕⊖   │ │ ⌕...  │ │ Export │    │
│ └──────────┘ └──────────┘ └──────┘ └──────┘ └──────┘ └────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                                                                     │
│                                                                     │
│           Graph Canvas (flex-1, react-flow)                         │
│                                                                     │
│           ┌──────┐  call  ┌──────┐                                   │
│           │ File │ ────> │ Func │                                   │
│           │  M   │       │  M   │                                   │
│           └──────┘       └──────┘                                   │
│              │              │                                       │
│           contain       contain                                    │
│              ▼              ▼                                       │
│           ┌──────┐       ┌──────┐                                   │
│           │ Func │       │Class │                                   │
│           │  M   │       │  +   │  (green = added)                  │
│           └──────┘       └──────┘                                   │
│                                                                     │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ Side Panel (w-80, border-l, collapsible)                            │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Node: fn domain-worktree::create(...)                          │ │
│ │ Type: Function | Visibility: Public | Async: Yes               │ │
│ │ File: crates/domain-worktree/src/service.rs                    │ │
│ │ Line: 42 - 78                                                   │ │
│ ├─────────────────────────────────────────────────────────────────┤ │
│ │ ```rust                                                         │ │
│ │ pub async fn create(&self, ...) -> Result<...> {              │ │
│ │     ...                                                         │ │
│ │ }                                                               │ │
│ │ ```                                                             │ │
│ ├─────────────────────────────────────────────────────────────────┤ │
│ │ [Open in IDE] [Copy path] [Follow refs]                        │ │
│ └─────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│ Status Bar (h-8, bg-slate-50, border-t)                             │
│ nodes: 1247 | edges: 3891 | parse: 28.4s | cache: HIT | grammar: rust,ts │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 节点样式 (Node Styling)

per 01-requirements §2.1:

| 节点类型 | 形状 | 默认颜色 | Diff 颜色 (added/modified/deleted) |
|---|---|---|---|
| File | square (rounded 4px) | bg: #f8fafc, border: 1px #cbd5e1 | green #22c55e / orange #f59e0b / red #ef4444 |
| Function | circle | bg: #eff6ff, border: 1px #3b82f6 | 同上 |
| Class | hexagon | bg: #fef3c7, border: 1px #f59e0b | 同上 |
| Struct | hexagon (stripe pattern) | bg: #fef3c7, border: 1px #d97706 | 同上 |
| Const | diamond | bg: #f0fdf4, border: 1px #16a34a | 同上 |

### 6.3 边样式 (Edge Styling)

| 边类型 | 样式 | 颜色 | 宽度 |
|---|---|---|---|
| Contain | solid | #475569 (slate-600) | 2px |
| Call | solid + arrow | #2563eb (blue-600) | 1.5px |
| Import | solid + arrow | #6b7280 (gray-500) | 1px |
| Reference | dashed + arrow | #9333ea (purple-600) | 1px (dashed) |
| Diff (any of above) | + colored border | inherit from node._diff | + 0.5px |

### 6.4 Layout 算法 (per §1.1)

**默认**: dagre (top-to-bottom by file hierarchy)
- file nodes 顶层 → function/class/const 子节点
- call/import/reference 边跨 file → 上 → 下

**可选**: d3-force (物理模拟, 用户切换)
- 适用大型图 (> 5000 nodes)
- 边权 = 1 / edge.confidence

### 6.5 交互 (Interactions)

| 操作 | 触发 | 行为 |
|---|---|---|
| Click 节点 | 鼠标左键 | Side panel 显示节点详情 + 代码预览 |
| Hover 节点 | 鼠标悬停 | 高亮所有出/入边, dim 其他 |
| Hover 边 | 鼠标悬停 | tooltip: "from: src → to: dst" |
| Click 边 | 鼠标左键 | 双节点高亮 + 跳 source node |
| Drag 节点 | 鼠标拖拽 | 移动节点位置 (dagre 模式下锁定) |
| Scroll | 滚轮 | 缩放 |
| Right-click | 鼠标右键 | 上下文菜单: Open in IDE / Copy path / Follow refs |
| Ctrl+F | 键盘 | 聚焦搜索框 |
| Esc | 键盘 | 取消选中 / 关闭 side panel |
| 1-9 | 键盘 | 切换 layout (1=dagre, 2=d3-force, ...) |

### 6.6 Search / Filter (per F-11)

**搜索框位置**: Toolbar 右侧
**搜索范围**: node.name + node.file_path (client-side filter)
**匹配模式**:
- 精确: "domain-worktree::create"
- 子串: "domain-worktree"
- 正则: "/domain-.*::.*/"

**过滤维度** (multi-select):
- Node type: file / function / class / const
- Diff status: added / modified / deleted / unchanged
- Language: rust / typescript
- Visibility: public / private

**操作**:
- "只看 diff 节点" (一键聚焦任务卡修改)
- "Hide unchanged" (隐藏未修改节点)
- "Reset filter" (清空)

## 7. セキュリティ/性能/運用 (Security / Performance / Operations)

### 7.1 セキュリティ (per 01-requirements §3.3)

| ID | 実装箇所 | 詳細 |
|---|---|---|
| NFR-S-01 | `worktree_resolver` | path 必须以 `.worktrees/<name>/` 或 `<star_root>/` 开头, 禁 follow 符号链接 |
| NFR-S-02 | `parser` | 单文件 > 5MB 跳过 + metadata.oversized_files++ |
| NFR-S-03 | `parser` | path 校验, 禁 `..` 越界 |
| NFR-S-04 | 全層 | 禁 env value 打印 (per 守门 #5) |
| NFR-S-05 | `audit` | 全 graph query 写 audit log (per 守门 #13) |
| NFR-S-06 | `secret_scanner` | parse 输出前扫 `ghp_*` / `AKIA*` / `xox[abpr]-*` 等 pattern, 命中则 redact |

### 7.2 性能 (per 01-requirements §3.1)

| ID | 目標 | 実装戦略 |
|---|---|---|
| NFR-P-01 (1000 file parse) | ≤ 30s p95 | tree-sitter 多线程 (rayon), 8 workers 并行 parse |
| NFR-P-02 (cache hit) | ≤ 500ms p95 | HashMap O(1) lookup + 内存 graph 直接返回 |
| NFR-P-03 (节点渲染) | ≤ 2s p95 | react-flow 虚拟化 (只渲染 viewport 内节点), 5000 节点测试 |
| NFR-P-04 (边渲染) | ≤ 3s p95 | dagre layout 缓存, 第二次进入同 worktree 直接用 |
| NFR-P-05 (增量 re-parse) | ≤ 200ms p95 | notify crate file watcher → invalidate 单文件 + 局部重 parse |
| NFR-P-06 (search) | ≤ 100ms p95 | client-side JS filter, debounce 100ms |

### 7.3 運用 (Operations)

**部署**:
- graph-service: 独立 binary, 跟现有 star-cli/star-mcp/star-api-rest 同构建
- 启动: `cargo run --bin graph-service` 或 systemd unit
- 端口: 8090
- 日志: stdout JSON, 跟其他 star-* 进程一致

**监控**:
- `/api/health` 端点 (K8s liveness probe)
- metrics: parse_duration_ms, cache_hit_ratio, nodes_count_avg
- audit log → star-audit (跨 17 tools 共享)

**升级**:
- graph JSON schema version 字段, 升级时 bump → 旧 cache 自动失效
- grammar 升级: 重新 parse, 旧 cache 失效

**回滚**:
- graph-service 独立 binary, 旧版本 binary 保留
- 旧 cache 自动过期 (TTL 1h)

## 8. 移行/未解決 (Migration / Open Issues)

### 8.1 移行計画 (Migration Plan)

| Phase | 内容 | 期間 (軟参考) |
|---|---|---|
| **M1 (MVP)** | graph-service 骨架 (1 binary, 1 endpoint `/api/health`) + task schema 加 worktree_id 字段 + 1 grammar (Rust) | 1 周 |
| **M2 (MCP 集成)** | 17th tool `get_task_graph` + frontend `/graph/<task-id>` 入口 | 1 周 |
| **M3 (Diff overlay)** | git diff 集成 + node._diff 渲染 + status bar | 0.5 周 |
| **M4 (TypeScript grammar)** | tree-sitter-typescript 接入 + frontend graph 支持 | 0.5 周 |
| **M5 (Search/Filter)** | client-side search + filter + 节点交互 | 0.5 周 |
| **M6 (Cache 完善)** | LRU + TTL + metrics | 0.5 周 |
| **M7 (优化)** | 增量 re-parse (file watcher) + 性能调优 | 1 周 (per NFR-P-05) |
| **M8 (P3 features)** | 增量 re-parse / Type inference / Control flow / 多 grammar | 2+ 周 (per F-13..F-18 P2-P3) |

**Token 預算 (per STAR-OLU-001, 1 SRE·周 = 1.2M tokens)**:
- MVP (M1-M6): 4 周 = 4.8M tokens
- P3 features (M7-M8): 3+ 周 = 3.6M+ tokens
- 合計: 8.4M+ tokens (per 守门 #4 token-OLU 框架)

### 8.2 未解決 (per 01-requirements §7 已知缺口衔接)

| # | 缺口 (G-01~G-07) | M 阶段 | 决定 |
|---|---|---|---|
| G-01 (task schema worktree_id 字段) | M1 | task schema review (DDD Lead 拍板) |
| G-02 (symbol resolver 准确率) | M1-M2 | MVP 验证, 准确率 < 90% 则标 degraded |
| G-03 (5MB threshold) | M1 | 实测校准, < 5MB 全量, ≥ 5MB skip |
| G-04 (diff base commit 定义) | M1 | 暂用 task.base_commit, 无则 HEAD~1, DDD Review 拍板 |
| G-05 (graph-service 进程模型) | M1 | 独立 binary (per C-21), 不嵌入 domain crate |
| G-06 (5 域 Lead 真人到位流程) | post-MVP | DDD Review 阶段 |
| G-07 (react-flow vs cytoscape) | M1 | 暂用 react-flow (跟 LangGraph view 02 §1.1 一致), 性能不达则换 cytoscape |

### 8.3 风险 (Risks)

| # | 风险 | 影響 | 缓解 |
|---|---|---|---|
| **R-01** | tree-sitter Rust grammar 升级导致 AST schema 变化 | 节点抽取错位 | 锁定 grammar 版本 (Cargo.toml), 升级单独 phase |
| **R-02** | symbol resolver 准确率 < 90% | call/import 边不完整, 影响 E-04 | MVP 阶段标 degraded, P3 阶段考虑 LSP-grade resolver |
| **R-03** | 1000 file worktree parse 超 30s | NFR-P-01 不达标 | 增量 parse + cache 预热, 实际 95% 请求走 cache hit |
| **R-04** | react-flow 5000+ node 性能 | NFR-P-03 不达标 | 节点聚合 (folder 视图), 切换到 cytoscape.js |
| **R-05** | worktree HEAD 频繁变化 (active dev) | cache miss 高 | 短 TTL (10min), 强制 refresh 显式触发 |
| **R-06** | 5 域 Lead 真人到位时, graph view 评审流程未明 | DDD Review 阻塞 | M8 阶段补流程文档, 当前 Mavis 临时代签 (per 守门 #3) |

## 9. 制約/既知缺口 (Constraints / Known Gaps)

per 01-requirements §4 制約事項 (C-01~C-21) 全部继承, 另加本设计阶段新增:

| # | 派生约束 | 出处 |
|---|---|---|
| **C-22** | graph-service 独立 binary, 不进 22 domain-* crate 编译链 | 本设计 §1.2 + C-21 |
| **C-23** | tree-sitter grammar 锁版本, 升级走单独 phase | 本设计 §8.3 R-01 |
| **C-24** | graph JSON schema 带 version 字段, 旧 cache 自动失效 | 本设计 §3.4 + §7.3 |
| **C-25** | 任务卡 worktree_id / base_commit 属 Work 类 (per 守门 #13 W/T/M) | 本设计 §3.5 + 守门 #13 (a) |
| **C-26** | frontend 选 react-flow, 跟 LangGraph view 02 §1.1 一致, 性能不达则 P3 换 cytoscape | 本设计 §6.4 + G-07 |
| **C-27** | secret 扫描在前端 fetch 之前, redact 在 graph JSON 中 | 本设计 §7.1 NFR-S-06 |

## 10. 签字栏 (Signature)

| 角色 | 姓名 | 签批 | 日期 |
|---|---|---|---|
| **架构 (代签)** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 🟢 Mavis 接手终审 | 2026-09-03 |
| **SRE Lead (代签)** | — | 🟢 Mavis 接手代签 (per 守门 #3 v2 + 守门 #14) | 2026-09-03 |
| **平台 (代签)** | — | 🟢 Mavis 接手代签 | 2026-09-03 |
| **评审主持 (代签)** | — | 🟢 Mavis 接手代签 | 2026-09-03 |
| **PM (代签)** | — | 🟢 Mavis 接手代签 | 2026-09-03 |
| **5 域 Lead (5 域真人, 待 DDD Review 阶段补)** | 真人到位后追溯签字 | ⏳ 待签 (per 守门 #3 拒绝兼任) | DDD Review 阶段 |

> **代签依据 (per AGENTS.md §1)**: 2026-08-27 19:39 JST 用户明确发令"允许你代签" + 21:59 JST 第三次强化"继续, 你可以代签"。Mavis 接手默认代签 Ulysses 无需再问。**保留派生约束**: 禁回溯叙事 / BAS git log --follow 实证 / 缺标比错标 / 子代理授权"无证据叙事=禁止"。

## 11. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 基本設計書 (架构 / 组件 / 数据模型 / API / 数据流 / UI / 安全性能 / 移行 / 约束 / 签字 / 修订) | 2026-09-03 19:5X JST 用户发令"设计需求文档和基本设计" + 01-requirements.md v0.1 落档 (per 守门 #1 + #10 + #12 派生, Mavis 接手代签 author=Ulysses) |
