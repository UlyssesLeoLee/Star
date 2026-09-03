# 01. Tree-sitter Worktree Graph - 要件定義書 (Requirements Definition)

> **状態**：🟡 Draft v0.1
> **日期**：2026-09-03
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户授权"允许你代签" + 21:59 JST 第三次强化"继续, 你可以代签"）
> **依赖**：[ADR-0033 代签规则反转](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) · [ADR-0026 STAR AI 兼容](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md) · [AGENTS.md §4 守门硬约束](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [STAR-OLU-001.md token 基线](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md)
> **关联文档**：[02-basic-design.md](02-basic-design.md)（基本設計書）
> **適用範囲**：STAR 主仓 (`D:\Star`) 全体，gm-console frontend / 22 domain-* crates / scripts/automation/ 全栈
> **触发用户原话** (2026-09-03 19:5X JST): "我想把 Tree-sitter 集成进 kanban 任务卡的一个标签页，让所处 worktree 的图论构造显示在里面，设计需求文档和基本设计"

---

## 0. 目的 (Purpose)

本文档定义 **Tree-sitter Worktree Graph** 视图 — 一个嵌入 Kanban 任务卡的 code intelligence 视图：

- **核心绑定**：每张 Kanban 任务卡 ↔ 1 个 git worktree（1:1 绑定，通过 `git worktree list` 解析）
- **视图内容**：该 worktree 当前 commit 的代码图论构造（AST + 符号引用图）
- **Diff 叠加**：任务卡产生的代码修改（add/modify/delete）作为视觉 overlay 出现在图节点上
- **入口形态**：任务卡详情里嵌入跳转入口 → 独立新 view（per 2026-09-03 用户决策"独立新 tab"），路径如 `/graph/<task-id>` 或 `/graph/<worktree-id>`

通过该视图实现：
1. **代码结构可视化**：一眼看清 worktree 内的 module/class/function 拓扑关系
2. **变更影响范围**：任务卡修改的代码在图中的位置 + 影响传播
3. **代码评审辅助**：跨文件引用追踪，变更点 vs 调用点对照
4. **与 Kanban 工作流融合**：图论视图作为任务卡 context 的一部分
5. **守门可观测**：图节点变更差异可作为 AGENTS.md §4 守门 violation 检测输入

> **重要区分 (per 2026-09-03 用户决策原文)**:
> - "所处 worktree" = 任务卡关联的 git worktree literal (per `git worktree list` 输出, e.g. `D:\Star\.worktrees\wt-sub-session-001`)
> - "任务卡修改的内容呈现在那个 worktree 里面的图论构造" = 任务卡 diff (add/modify/delete) 作为 graph node/edge 上的 visual overlay (color/border/icon)
> - 视图形态 = 独立新 view (非嵌入 Kanban 主 tab), 任务卡里只放跳转入口

## 1. 業務要件 (Business Requirements)

### 1.1 背景 (Background)

Star 项目现状（per 2026-09-03 main HEAD `05ce670`）：

- **22 domain-* crate** (DDD bounded context): identity / permission / work-item / workspace / **worktree** / feedback / validation / integration / comment / project / tenant / ...
- **`domain-worktree` crate 已存在** (per `D:\Star\crates\domain-worktree\Cargo.toml`): 提供 worktree domain entity + port + service
- **16 MCP tools** (star-mcp stdio + Streamable HTTP transport, per ADR-0032): 含 worktree 相关 tool
- **gm-console frontend** (Next.js + AppShell 5-tab: Kanban / Timeline / Backlog / Agents / Worktrees, per `docs/architecture/2026-09-03-langgraph/02-basic-design.md` §1.1)
- **Git worktree literal 现有 3 个** (per `D:\Star\.worktrees\`): `wt-sub-session-001` / `wt-nav-i18n-a` / `wt-nav-shots-b`
- **scripts/automation/** (4 基类 + 4 utility, per `scripts/automation/registry.md` v0.1)
- **AGENTS.md §4 守门 13 main (#1-#13) + 24 派生规 v1-v24 = 37 全部继承**

**关键 gap (无现有 tree-sitter 依赖)**:
- `Cargo.toml` + `Cargo.lock` 全仓 0 `tree-sitter` 引用 (per 2026-09-03 grep 实证)
- 无现有 AST 解析 / 符号图 / 代码图论构造服务
- 无独立 graph view UI component
- Kanban 任务卡无 worktree 绑定字段（当前 task card schema 待 review）

### 1.2 業務課題 (Business Challenges)

| # | 課題 | 影響範囲 |
|---|---|---|
| **B-01** | 任务卡代码修改无可视化反馈 — 改了什么文件/函数/类对调用方影响未知 | code review 需多 IDE 跳转，跨文件影响靠人脑维护 |
| **B-02** | worktree 内代码结构无整体视图 — module/class/function 拓扑不可见 | 新人 onboarding 慢，跨域改造影响分析困难 |
| **B-03** | Kanban 任务卡 ↔ git worktree 关系松散 — 哪张卡对应哪个 worktree 不明确 | 多 worktree 并行开发时，任务归属混乱 |
| **B-04** | 任务卡 diff 与代码图无关联 — commit/diff 信息不进图论视图 | "改了哪些节点" vs "影响哪些节点" 不可见 |
| **B-05** | 现有 16 MCP tools 缺 code intelligence 维度 — 都是任务管理/版本控制底层操作 | 高层"看图"需求缺统一入口 |
| **B-06** | 跨语言项目 AST 解析碎片化 — 22 domain-* (Rust) + frontend (TypeScript) + scripts (Python) | 单一 tree-sitter grammar 不够，需多 grammar 联合 |

### 1.3 期待効果 (Expected Effects)

| # | 効果 | 計測指標 |
|---|---|---|
| **E-01** | 任务卡修改在图上一目了然 | diff 节点高亮覆盖率 100% (modified/added/deleted) |
| **E-02** | worktree 内代码结构可视化 | 节点/边渲染 P95 ≤ 2s (per 1000 file worktree) |
| **E-03** | 任务卡 ↔ worktree 绑定关系明确 | 任务卡 schema 强制 worktree_id 字段 |
| **E-04** | 跨文件引用追踪 | call/import/reference 边解析准确率 ≥ 90% (MVP 阶段) |
| **E-05** | 与 Kanban 工作流融合 | 任务卡详情 → graph view 跳转 ≤ 1 click |
| **E-06** | 多语言 AST 联合 | MVP 阶段支持 Rust + TypeScript 2 种 grammar |

### 1.4 業務フロー (Business Flows)

#### 1.4.1 主シナリオ: 任务卡 → graph view → 评审

```
[User] ──click task card──> [Kanban: card detail modal]
                                  │
                                  ├──> Tab 1: Overview (现有)
                                  ├──> Tab 2: Discussion (现有)
                                  ├──> Tab N: ★ Graph (新入口)
                                  │         │
                                  │         │ click "Open Graph View"
                                  │         ▼
                                  │  [独立 view: /graph/<task-id>]
                                  │         │
                                  │         ├──> load worktree path
                                  │         │   (task.worktree_id → git worktree list resolve)
                                  │         │
                                  │         ├──> parse AST + symbols
                                  │         │   (tree-sitter + symbol resolver)
                                  │         │
                                  │         ├──> overlay task card diff
                                  │         │   (git diff HEAD → node highlight)
                                  │         │
                                  │         └──> render graph
                                  │              (react-flow + nodes + edges + diff overlay)
                                  │
                                  │  [User: 评审 / 点击节点 / 跳转 file:line]
```

#### 1.4.2 副シナリオ: 多 worktree 并行

```
[User: 5 域 Lead 配置 review]
   │
   ├──> Task Card 1 (worktree: wt-sub-session-001, 5 域 lead 决策)
   │       │
   │       └──> Graph view: 显示 wt-sub-session-001 当前结构 + 此 card diff
   │
   ├──> Task Card 2 (worktree: wt-nav-i18n-a, frontend i18n)
   │       │
   │       └──> Graph view: 显示 wt-nav-i18n-a frontend 结构 + 此 card diff
   │
   └──> Task Card N ...
```

每个 task card 独立 worktree, 独立 graph view, 互不干扰。

#### 1.4.3 異常シナリオ: parse 失败 / worktree 缺失

```
[Graph View 加载] ──> (worktree 路径不存在) ──> [error state: "worktree not found, please check git worktree list"]
                                                  │
[Graph View 加载] ──> (tree-sitter parse 失败) ──> [error state: "parse failed on file X, partial graph shown"]
                                                  │
[Graph View 加载] ──> (symbol resolver 超时) ──> [degraded: 显示 AST, 标"符号图生成超时"]
```

## 2. 機能要件 (Functional Requirements)

### 2.1 システム全体像 (System Overview)

3-tier 架构 + 任务卡入口 + 独立 view 渲染：

```
┌──────────────────────────────────────────────────────────────────┐
│                       UI Tier (gm-console frontend)               │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Kanban Board ──> Task Card Modal ──> [Graph Tab (入口)]   │  │
│  │                                              │             │  │
│  │                                              ▼             │  │
│  │  ★ NEW ★ Graph View (独立 route /graph/<task-id>)           │  │
│  │    ┌────────────────────────────────────────────────────┐  │  │
│  │    │  [Toolbar: worktree path / commit / refresh / fit]  │  │  │
│  │    ├────────────────────────────────────────────────────┤  │  │
│  │    │  [Graph Canvas: react-flow + nodes + edges]         │  │  │
│  │    │    • 节点: file (square) / function (circle) /      │  │  │
│  │    │            class (hexagon) / const (diamond)        │  │  │
│  │    │    • 边: import (gray) / call (blue) / contain      │  │  │
│  │    │            (black) / reference (dashed)             │  │  │
│  │    │    • Overlay: modified (orange) / added (green) /   │  │  │
│  │    │               deleted (red strike)                  │  │  │
│  │    └────────────────────────────────────────────────────┘  │  │
│  │    ┌────────────────────────────────────────────────────┐  │  │
│  │    │  [Side Panel: node details / file preview / breadcrumb] │  │
│  │    └────────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
                                  │ REST + WebSocket
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                    Backend Tier (graph-service)                    │
│  ┌────────────────┐ ┌────────────────┐ ┌──────────────────────┐ │
│  │ TreeSitter     │ │ Symbol         │ │ Worktree             │ │
│  │ Parser Service │ │ Resolver       │ │ Resolver             │ │
│  │ (Rust:         │ │ (Rust:         │ │ (Rust:               │ │
│  │  tree-sitter   │ │  cross-file    │ │  domain-worktree     │ │
│  │  + grammars)   │ │  ref tracker)  │ │  + git CLI)          │ │
│  └────────────────┘ └────────────────┘ └──────────────────────┘ │
│  ┌────────────────┐ ┌────────────────┐ ┌──────────────────────┐ │
│  │ Graph Builder  │ │ Diff Overlay   │ │ Cache Layer          │ │
│  │ (AST → nodes/  │ │ (git diff →    │ │ (per-worktree LRU,   │ │
│  │  edges JSON)   │ │  node marks)   │ │  content-hash key)   │ │
│  └────────────────┘ └────────────────┘ └──────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
                                  │ tree-sitter grammar + git CLI
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                  Worktree Tier (filesystem + git)                 │
│  D:\Star\.worktrees\wt-sub-session-001\                          │
│  D:\Star\.worktrees\wt-nav-i18n-a\                                │
│  D:\Star\.worktrees\wt-nav-shots-b\                               │
│  D:\Star\ (main worktree)                                         │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 アクター (Actors)

| アクター | 種別 | 説明 |
|---|---|---|
| **Primary User (Ulysses)** | 人間 | 12 角色一人公司，Star 项目唯一决策者，触发本 view 设计 |
| **Developer** | 人間 | 5 域 Lead 真人 (per 守门 #3) + 协作者，使用 graph view 评审 |
| **TreeSitter Parser** | システム | Rust 进程内 tree-sitter crate，AST 生成 |
| **Symbol Resolver** | システム | 跨文件符号引用追踪器 (简易版) |
| **Graph Builder** | システム | AST + symbols → nodes/edges JSON |
| **Diff Overlay** | システム | git diff → node 颜色/边框映射 |
| **Worktree Resolver** | システム | task.worktree_id → git worktree 路径 |
| **Graph View UI** | システム | frontend react-flow 渲染 |
| **Task Card Manager** | システム | 现有 Kanban 后端，含 worktree_id 字段扩展 |

### 2.3 ユースケース (Use Cases)

#### UC-01: 任务卡 → Graph view 入口跳转

- **Actor**: Primary User
- **Trigger**: 用户在 Kanban 任务卡详情页点击 "Graph" tab
- **Flow**:
  1. UI: 任务卡 modal 显示当前 tab 列表（含 Graph tab）
  2. User: 点击 Graph tab
  3. UI: 渲染入口卡片（"Open Graph View" 按钮 + 摘要：worktree path + file count + diff summary）
  4. User: 点击 "Open Graph View"
  5. UI: 新窗口/新路由打开 `/graph/<task-id>`
- **Postcondition**: Graph view 加载中（spinner）

#### UC-02: Graph view 加载 (worktree → graph)

- **Actor**: Graph View UI
- **Trigger**: 用户进入 `/graph/<task-id>`
- **Flow**:
  1. UI: 调用 `GET /api/graph/<task-id>`
  2. Backend (graph-service):
     - 查 task.worktree_id
     - `git worktree list --porcelain` 解析 worktree 路径
     - 检查 path 存在 + 当前 commit (HEAD)
     - 检查 cache (worktree + commit hash 命中?)
     - 缓存未命中: tree-sitter 遍历源文件 → AST → 符号解析 → graph JSON
     - 缓存命中: 直接返回
  3. Backend: 同时计算 `git diff` (worktree HEAD vs 任务卡基准 commit) → diff overlay marks
  4. Backend: 合并 graph + diff → 返回 JSON
  5. UI: 渲染 graph canvas
- **Postcondition**: graph 显示 + diff overlay 可见

#### UC-03: 节点点击 → 文件预览

- **Actor**: Developer
- **Trigger**: 用户点击 graph 中某个节点
- **Flow**:
  1. UI: 节点点击事件
  2. UI: 右侧 side panel 显示节点详情
     - 节点类型 (file / function / class / const)
     - 完整路径
     - 起始/结束行号
     - 代码预览 (前 50 行)
     - 跳转 IDE 按钮 (vscode://file/<path>:<line>)
  3. UI: 高亮该节点所有出/入边
- **Postcondition**: 用户获得节点上下文

#### UC-04: 任务卡 diff overlay 展示

- **Actor**: Diff Overlay
- **Trigger**: graph 加载时
- **Flow**:
  1. Backend: 读任务卡关联 worktree HEAD commit
  2. Backend: `git diff <task-base-commit>..HEAD --name-status` 列出 modified/added/deleted 文件
  3. Backend: 每个 changed file → 找到 graph 中对应 file 节点 + 子节点
  4. Backend: 在 graph JSON 中添加 `_diff` 字段: `added` / `modified` / `deleted`
  5. UI: 节点渲染按 `_diff` 字段上色 (added=绿, modified=橙, deleted=红删线)
- **Postcondition**: 用户直观看到任务卡修改的代码位置

#### UC-05: Worktree 切换 (跨 worktree 任务卡)

- **Actor**: Primary User
- **Trigger**: 用户在 task card A 看 graph 后，切到 task card B
- **Flow**:
  1. UI: 关闭当前 graph view
  2. UI: 打开 task card B 的 graph view (`/graph/<task-id-B>`)
  3. Backend: 同样 UC-02 流程，但 worktree 路径不同
  4. UI: 重新渲染（不同 worktree 不同 graph）
- **Postcondition**: graph 切换完成

#### UC-06: 节点搜索 / 过滤

- **Actor**: Developer
- **Trigger**: graph 节点过多 (e.g. 1000+ nodes)
- **Flow**:
  1. UI: 顶部 toolbar 提供搜索框
  2. User: 输入 "domain-worktree"
  3. UI: 客户端过滤匹配节点 (name 包含 + path 包含)
  4. UI: 高亮匹配节点，dim 其他节点
  5. User: 可选按 diff status 过滤 (只看 modified)
- **Postcondition**: 视图聚焦

#### UC-07: 跨文件引用追踪 (call/import edge 跳转)

- **Actor**: Developer
- **Trigger**: 用户点击 call edge
- **Flow**:
  1. UI: edge hover 显示 "from: fn_a@file1.rs:10 → to: fn_b@file2.rs:42"
  2. UI: edge click → 双节点高亮 + 跳转到 source node
  3. UI: 可选 "follow references" 模式，逐跳追踪
- **Postcondition**: 用户理解调用链

#### UC-08: Graph view 刷新 / 增量更新

- **Actor**: Primary User
- **Trigger**: 任务卡有新 commit
- **Flow**:
  1. UI: 提供 "Refresh" 按钮
  2. User: 点击
  3. UI: 调用 `POST /api/graph/<task-id>/refresh`
  4. Backend: 强制 invalidate cache → 重新 parse
  5. UI: 重新加载 graph
- **Postcondition**: graph 与最新 commit 同步

#### UC-09: 错误降级 (parse 失败 / worktree 缺失)

- **Actor**: Graph View UI
- **Trigger**: backend 返回 partial / error
- **Flow**:
  1. Backend: 解析失败 → 返回部分 graph + 错误列表
  2. UI: 显示 toast "X 个文件解析失败, 详见 error panel"
  3. UI: 提供 "Retry" / "View Errors" 操作
- **Postcondition**: 用户知悉降级状态

### 2.4 機能一覧 (Function List)

| # | 機能 | 説明 | 優先度 |
|---|---|---|---|
| **F-01** | 任务卡 ↔ worktree 绑定 | task schema 加 worktree_id 字段 (per 守门 #13 W/T/M, 必填) | P0 |
| **F-02** | Worktree 路径解析 | `git worktree list --porcelain` → 路径 map | P0 |
| **F-03** | Tree-sitter 多 grammar | MVP: Rust + TypeScript; extensible: Python/Go/Java | P0 |
| **F-04** | AST 解析 | 遍历 worktree 源文件 → tree-sitter parse → node 抽取 | P0 |
| **F-05** | Symbol Resolver | 跨文件 call/import/reference 边生成 (简易版) | P0 |
| **F-06** | Graph JSON 输出 | nodes + edges + metadata, JSON over HTTP | P0 |
| **F-07** | Diff overlay | git diff → node `_diff` 字段 (added/modified/deleted) | P0 |
| **F-08** | Cache layer | per (worktree_path, commit_hash) LRU cache, TTL 1h | P0 |
| **F-09** | Graph view UI | 独立 route + react-flow 渲染 + node/edge styles | P0 |
| **F-10** | 节点交互 | click → side panel; edge hover → info | P0 |
| **F-11** | 搜索 / 过滤 | client-side 节点过滤 + diff status 过滤 | P1 |
| **F-12** | 节点 → IDE 跳转 | vscode://file/<path>:<line> deep link | P1 |
| **F-13** | 增量 re-parse | file watcher (notify crate) → invalidate + reparse | P2 |
| **F-14** | Type inference | 类型图 (扩展节点) | P3 |
| **F-15** | Control flow graph | 函数内控制流 (扩展) | P3 |
| **F-16** | 多 grammar 扩展 | Python/Go/Java/Kotlin/Swift grammar 接入 | P2 |
| **F-17** | Graph export | PNG/SVG/JSON 导出 | P2 |
| **F-18** | Graph diff | 跨 commit 比较 graph 结构变化 | P3 |
| **F-19** | MCP tool 集成 | star-mcp 加 `get_task_graph` tool, 16 → 17 tools | P1 |
| **F-20** | 守门 violation 检测 | graph diff 对比 AGENTS.md §4 守门规则 (e.g. 跨域 import 警告) | P3 |

## 3. 非機能要件 (Non-Functional Requirements)

### 3.1 性能 (Performance)

| ID | 項目 | 目標値 |
|---|---|---|
| **NFR-P-01** | 1000 file worktree 首次 parse | ≤ 30s p95 |
| **NFR-P-02** | 1000 file worktree 缓存命中返回 | ≤ 500ms p95 |
| **NFR-P-03** | 1000 file worktree 节点渲染 | ≤ 2s p95 (前端) |
| **NFR-P-04** | 1000 file worktree 边渲染 | ≤ 3s p95 (前端) |
| **NFR-P-05** | 增量 re-parse 单文件 | ≤ 200ms p95 |
| **NFR-P-06** | search 过滤响应 | ≤ 100ms p95 (前端 client-side) |

### 3.2 可用性 (Availability)

| ID | 項目 | 目標値 |
|---|---|---|
| **NFR-A-01** | graph-service uptime | ≥ 99% (本地进程) |
| **NFR-A-02** | parse 失败降级 | 部分 graph + error panel, 不全失败 |
| **NFR-A-03** | worktree 缺失处理 | 友好错误提示 + 跳转 git worktree list |
| **NFR-A-04** | cache 自动失效 | worktree HEAD commit 变化 → 自动 invalidate |

### 3.3 セキュリティ (Security)

| ID | 項目 | 適用 |
|---|---|---|
| **NFR-S-01** | worktree 路径限制 | 仅解析 `.worktrees/*` + 主仓, 禁 follow 符号链接逃逸 | 
| **NFR-S-02** | 文件大小限制 | 单文件 > 5MB 跳过 parse, 标 oversized |
| **NFR-S-03** | path traversal 防护 | 文件路径校验, 禁 `..` 越界 |
| **NFR-S-04** | 环境变量 hard ban | 禁 env value 打印 (per 守门 #5) |
| **NFR-S-05** | audit log | graph 查询全量 → audit log (per 守门 #13) |
| **NFR-S-06** | secret 扫描 | parse 输出前扫 secret pattern (GHPAT/AWS key) |

### 3.4 保守性 (Maintainability)

| ID | 項目 | 適用 |
|---|---|---|
| **NFR-M-01** | grammar 模块化 | 每语言 grammar 独立 module, 插件式注册 |
| **NFR-M-02** | graph schema 文档化 | JSON schema + 自动生成 docs |
| **NFR-M-03** | 测试覆盖 | parser 单测 ≥ 80%, resolver 单测 ≥ 70% |

### 3.5 拡張性 (Scalability)

| ID | 項目 | 適用 |
|---|---|---|
| **NFR-E-01** | 新 grammar 接入 | 加 dep + 注册, 无需改核心 |
| **NFR-E-02** | 新 node type | schema 扩展, UI 渲染 map 加项 |
| **NFR-E-03** | 多 worktree 并行查询 | graph-service 多线程 + cache 隔离 |
| **NFR-E-04** | 跨 worktree 视图 | 多 wt diff 对比 (P3) |

## 4. 制約事項 (Constraints)

per AGENTS.md §4 守门硬约束 (13 main + 24 派生规 = 37 项) 全部继承：

| # | 约束 | 出处 |
|---|---|---|
| **C-01** | Star 仓独立 (per 守门 #1, R-05 反転済 push 許可) | AGENTS.md §4 #1 |
| **C-02** | PowerShell only (per 守门 #6) | AGENTS.md §4 #6 |
| **C-03** | 環境変数安全 (per 守门 #5) | AGENTS.md §4 #5 |
| **C-04** | 0 unsafe (per 守门 #7) | AGENTS.md §4 #7 |
| **C-05** | 5 域独立 Lead, Mavis 临时代签 (per 守门 #3 反転) | AGENTS.md §4 #3 |
| **C-06** | AI 協作文档治理 (per 守门 #12) | AGENTS.md §4 #12 |
| **C-07** | token OLU 計算 (per 守门 #4) | AGENTS.md §4 #4 |
| **C-08** | agent 交互 Python 化 (per 守门 #19) | AGENTS.md §4.1 v19 |
| **C-09** | 子代理 dispatch 必先 brief (per 守门 #20) | AGENTS.md §4.1 v20 |
| **C-10** | DB 三類 W/T/M 横展開 (per 守门 #13) | AGENTS.md §4 #13 |
| **C-11** | 调试控制台不污染 main (per 守门 #22) | AGENTS.md §4.1 v22 |
| **C-12** | AI 修改 mock 不开外部 API (per 守门 #23) | AGENTS.md §4.1 v23 |
| **C-13** | 调试控制台走 subprocess (per 守门 #24) | AGENTS.md §4.1 v24 |
| **C-14** | 守门 #9 子代理 status ≠ 实际成功 | AGENTS.md §4 #9 |
| **C-15** | 回溯叙事禁止 (per 守门 #12 派生) | AGENTS.md §4 #12 |
| **C-16** | 缺标比错标安全 (per 守门 #11) | AGENTS.md §4 #11 |
| **C-17** | 代签规则 (per 守门 #10) | AGENTS.md §4 #10 |
| **C-18** | BC: 22 domain-* crate 边界不可破 (per 守门 #3 disclaimer, 不建立业务子域↔DDD 映射) | AGENTS.md §5 拓扑 |

**项目级约束**:

| # | 约束 | 出处 |
|---|---|---|
| **C-19** | worktree 数据 W/T/M 严格分类 (per 守门 #13): 任务卡 worktree_id 字段属 Work 类 (session-bound, 短 TTL), 不可跨 session 持久化错位 | AGENTS.md §4 #13 + 守门 #13 派生规 (a) |
| **C-20** | tree-sitter grammar 选择: MVP 阶段仅 Rust + TypeScript (per 守门 #11 缺标比错标) | AGENTS.md §4 #11 |
| **C-21** | graph-service 进程模型: 独立 Rust 二进制 (per 守门 #22 派生), 不进 main 编译链 | AGENTS.md §4.1 v22 |

## 5. 用語集 (Glossary)

| 用語 | 説明 |
|---|---|
| **Worktree (git worktree literal)** | `git worktree list --porcelain` 输出的工作副本, e.g. `D:\Star\.worktrees\wt-sub-session-001` |
| **任务卡 (Task Card)** | Kanban 上的 issue 卡片, 绑定 1 个 worktree |
| **Graph View** | 本文档定义的新视图, 渲染 worktree 的代码图论构造 |
| **Tree-sitter** | 增量式代码解析器, 输出 concrete syntax tree (CST) |
| **AST (Abstract Syntax Tree)** | 抽象语法树, tree-sitter 输出为 CST 但常用 AST 概念 |
| **Symbol Resolver** | 跨文件符号引用追踪器, 输出 call/import/reference 边 |
| **Node** | 图节点: file / function / class / struct / const |
| **Edge** | 图边: contain (parent-child) / call / import / reference |
| **Diff Overlay** | 任务卡修改的代码在图节点上的视觉标记 (added/modified/deleted) |
| **Worktree Resolver** | task.worktree_id → git worktree 路径的解析器 |
| **Cache Key** | `(worktree_path, commit_hash)` 二元组, 命中即返回 |
| **MCP Tool** | Model Context Protocol 工具, 16 → 17 (per F-19 扩展) |
| **MVP** | Minimum Viable Product, MVP 阶段: Rust + TypeScript 2 grammar, 1000 file worktree benchmark |
| **DDD Bounded Context** | 22 domain-* crate 各自 DDD bounded context (per 守门 #3 disclaimer) |
| **守门 (Guard)** | AGENTS.md §4 硬约束 13 main + 24 派生规 = 37 项 |
| **DEC-008** | 一人公司 12 角色 治理模型 |
| **Mavis 接手** | Ulysses 授权的 root agent 代签身份 (per 19:39 JST) |
| **Token OLU** | AI 协作 token 预算单位 (1 SRE·周 = 1.2M, per STAR-OLU-001) |
| **5 域 (5 Domains)** | player / economy / match / social / admin, per 守门 #3 历史治理命名, 不映射 DDD |
| **22 Domain Crates** | DDD bounded context 22 個 (identity/permission/work-item/worktree/...) |
| **react-flow** | frontend 现有图渲染库 (per LangGraph view 02 §1.1 frontend 栈) |

## 6. 想定シナリオ (Scenarios)

### S-01: Ulysses 评审跨 worktree 任务卡

```
Ulysses: 在 Kanban 看 3 张 task card, 分别绑定 3 个 worktree
   │
   ├──> Task "B.5 OpenClaw 真实接入" (worktree: wt-sub-session-001)
   │       │
   │       └──> 点 Graph tab → 看 wt-sub-session-001 整体结构
   │              │
   │              ├──> domain-billing 节点 modified (orange) — B.5 改的
   │              ├──> domain-ai 节点 added (green) — 新建
   │              ├──> 看 call edge: domain-billing::charge → domain-ai::predict
   │              └──> 评审: 影响范围 OK, approve
   │
   ├──> Task "frontend i18n" (worktree: wt-nav-i18n-a)
   │       │
   │       └──> Graph view: 看 frontend structure
   │              │
   │              ├──> AppHeader.tsx modified
   │              ├──> i18n/ 新增 8 个文件 (added green)
   │              └──> call edge: AppHeader → useI18n() (call blue)
   │
   └──> Task "screenshot bug" (worktree: wt-nav-shots-b)
           │
           └──> Graph view: 看 e2e/ + scripts/shot/ 结构
```

### S-02: 5 域 Lead 配置跨 worktree 评审

```
Ulysses: "5 域 Lead 真人到位, 跨 3 个 worktree 评审"
   │
   ├──> graph-service: 并行拉 3 个 worktree graph (cache miss)
   ├──> cross-worktree-diff: 跨 wt 节点对比 (per F-18 P3)
   └──> 评审: 5 域 Lead 配置一致性
```

### S-03: Parse 失败降级

```
Graph view 加载 wt-nav-shots-b (含 e2e/test-results/ 二进制文件)
   │
   ├──> tree-sitter 遇到 .png/.json (非文本) → skip + log
   ├──> 某个 .ts 文件含 JSX + 复杂泛型 → 部分 node 解析失败
   │
   └──> UI: 显示 1200 nodes (85% of full) + error panel "5 files partial"
```

## 7. 既知缺口 (Known Gaps) - per 守门 #11 缺标比错标

| # | 缺口 | 影響 | 阶段 |
|---|---|---|---|
| **G-01** | 任务卡 worktree_id 字段当前 schema 未落地 (per F-01 P0) | task card → graph view binding 待 task schema review | MVP 启动前 |
| **G-02** | symbol resolver 准确率目标 ≥ 90% 未验证 (简易实现 vs IDE-grade LSP) | call edge 完整性可能不达预期 | MVP 验证 |
| **G-03** | tree-sitter 大文件性能 (> 5MB 单文件) 未知 | 5MB 文件 skip 阈值需实测校准 | MVP 验证 |
| **G-04** | 跨 worktree 任务卡 diff 基准 commit 定义 (per F-07) | "任务卡 diff" 起点是 task.created_at HEAD 还是 base commit? | 设计待 DDD Review 拍板 |
| **G-05** | graph-service 进程模型 vs 22 domain-* crate 嵌入选择 (per C-21) | 独立 binary 还是 domain crate? 决定 0 unsafe + 编译链影响 | 设计阶段决策 |
| **G-06** | 5 域 Lead 真人到位后 graph view 评审流程 | 当前 Mavis 临时代签, 真人到位流程需 DDD Review Lead 拍板 | DDD Review 阶段 |
| **G-07** | frontend react-flow 选型确认 (per LangGraph view 02 §1.1 引用) | react-flow vs cytoscape.js vs d3 vs 自研, 性能/包大小/许可 | MVP 启动前 |

## 8. 関連ドキュメント (Related Documents)

- **[02-basic-design.md](02-basic-design.md)**: 本文档的基本設計書, 包含架构图/组件/数据模型/API
- **AGENTS.md §4 守门硬约束**: 13 main + 24 派生规 = 37 项全部继承
- **AGENTS.md §5 仓库拓扑**: Star 仓 22 domain-* crate 边界
- **AGENTS.md §6 ADR 索引**: ADR-0026 / 0027 / 0028 / 0029 / 0030 / 0031 / 0032 / 0033 / 0034 / 0035-0042 / 0043-0045
- **ADR-0026 STAR AI 兼容**: 5 通道 + Fallback Ladder 4 级 (graph view 走 LLM 通道可参考)
- **ADR-0029 Universal Submit**: 12 步 + 6 字段错误模型
- **ADR-0030 Agent Lease/Heartbeat/Resume**: 跨 Agent Handoff 11 字段
- **ADR-0031 Context Graph**: MVP 4 节点 + 5 关系 (本 graph view 可参考, 但范围不同)
- **ADR-0032 MCP Transport stdio**: 16 tools 扩展到 17 tools (F-19)
- **ADR-0033 代签规则反转**: 本文档 Mavis 接手代签依据
- **ADR-0034 Jira 化**: Kanban 任务卡 schema 扩展依据
- **docs/architecture/2026-09-03-langgraph/**: LangGraph 統合架构 (平行 view, 任务卡 sub-agent 镜像)
- **docs/architecture/2026-09-03-agent-runtime/**: Agent Runtime SRS/Basic/Detail
- **docs/ol/STAR-OLU-001.md**: token OLU 基线 (1 SRE·周 = 1.2M)
- **docs/reports/STAR-P3-WBS-001.md**: WBS 双轴排期
- **crates/domain-worktree/**: 现有 worktree domain, 复用 port/service
- **crates/star-mcp/**: 现有 MCP server, F-19 扩展 17 tools

## 9. 签字栏 (Signature)

| 角色 | 姓名 | 签批 | 日期 |
|---|---|---|---|
| **架构 (代签)** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 🟢 Mavis 接手终审 | 2026-09-03 |
| **SRE Lead (代签)** | — | 🟢 Mavis 接手代签 (per 守门 #3 v2 + 守门 #14) | 2026-09-03 |
| **平台 (代签)** | — | 🟢 Mavis 接手代签 | 2026-09-03 |
| **评审主持 (代签)** | — | 🟢 Mavis 接手代签 | 2026-09-03 |
| **PM (代签)** | — | 🟢 Mavis 接手代签 | 2026-09-03 |
| **5 域 Lead (5 域真人, 待 DDD Review 阶段补)** | 真人到位后追溯签字 | ⏳ 待签 (per 守门 #3 拒绝兼任) | DDD Review 阶段 |

> **代签依据 (per AGENTS.md §1)**: 2026-08-27 19:39 JST 用户明确发令"允许你代签" + 21:59 JST 第三次强化"继续, 你可以代签"。Mavis 接手默认代签 Ulysses 无需再问。**保留派生约束**: 禁回溯叙事 / BAS git log --follow 实证 / 缺标比错标 / 子代理授权"无证据叙事=禁止"。

## 10. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 要件定義書 (Purpose / Business Req / Functional Req / NFR / Constraints / Glossary / Scenarios / Known Gaps / Related Docs / Sign / Revision) | 2026-09-03 19:5X JST 用户发令"Tree-sitter 集成进 kanban 任务卡 tab, 设计需求文档和基本设计" + 4 项决策 (STAR 仓 / git worktree literal w/ task card diff overlay / AST+符号图 / 独立新 view) |
