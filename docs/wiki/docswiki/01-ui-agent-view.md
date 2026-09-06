---
title: '01 — Agent View 拓扑 (派生视图)'
date: 2026-09-06
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ 2026-09-06 Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ 2026-09-06 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: ["S1", "View-AgentView", "M-AGV-1", "M-AGV-2", "M-AGV-3", "M-AGV-4", "M-AGV-5", "M-AGV-6", "M-AGV-7", "M-AGV-8", "M-AGV-9", "M-AGV-10", "domain-worktree", "domain-work-item"]
related: ["00-design-topology", "02-orchestration-langgraph"]
see-also: ["S1", "View-AgentView"]
guards:
  - id: '#1'
    name: 0 unsafe + 0 err
    evidence: mermaid 语法自检, git 提交
  - id: '#3'
    name: 5 域独立 Lead / 3 view 平行
    evidence: View-AgentView / View-LangGraph / View-AgentRuntime [[wikilink]]
  - id: '#7'
    name: 0 unsafe
    evidence: 纯 markdown, 无代码
  - id: '#11'
    name: 缺标比错标
    evidence: 每图末「已知缺口」显式列
  - id: '#12'
    name: AI 協作文档治理
    evidence: 0 回溯叙事, 395+ 处 file:line 引用
  - id: '#19'
    name: agent 交互 Python 化
    evidence: scripts/automation/obsidian_topology_linkify.py
  - id: '#10'
    name: 代签规则应用
    evidence: author=Ulysses (per 19:39 JST 授权)
tags:
  - ui
  - agent-view
  - obsidian-wiki
  - design-topology
  - obsidian-wiki
  - design-topology
---





# 01 — Agent View 拓扑 (派生视图)

> **数据源**: [[S1]] [`docs/design/BD-AGENT-VIEW-001.md`](../../design/BD-AGENT-VIEW-001.md)
> **范围**: frontend/src/app/agent-view + frontend/src/lib/agent-view + frontend/src/components/agent-view
> **核心**: 3-tier UI / 10 模块 / 7 派生纯函数 / 3 类节点 + connector

---

## 1. 3-Tier 架构 (per [[S1]] §2.1)

```mermaid
flowchart TB
    subgraph UITier["UI Tier (per S1 §2.1 L75-87)"]
        direction TB
        Page["page.tsx<br/>M-AGV-9 root<br/>per S1 §6.1 L511"]
        PageHeader["PageHeader<br/>title=Agent"]
        AF["AgentFilter<br/>M-AGV-8 顶部 dropdown<br/>per S1 §6.1 L510"]
        ACV["AgentCanvasView<br/>M-AGV-6 SVG 无限画布<br/>per S1 §6.1 L508"]
    end

    subgraph DerivationTier["Derivation Tier (per S1 §2.1 L91-100)"]
        direction TB
        Sel["selectors.ts<br/>M-AGV-2 7 公开纯函数<br/>per S1 §6.1 L504"]
        Lay["layout.ts<br/>M-AGV-3 自由散开布局<br/>per S1 §6.1 L505"]
        Types["types.ts<br/>M-AGV-1 派生类型<br/>per S1 §6.1 L503"]
        SelTest["selectors.test.ts<br/>M-AGV-5 14 tests"]
        LayTest["layout.test.ts<br/>M-AGV-4 11 tests"]
    end

    subgraph DataTier["Data Tier (per S1 §2.1 L105-112)"]
        direction TB
        Store["zustand store<br/>localStorage 'star-store:v1'<br/>per S1 §7.4.3 L668"]
        AgentS["agentSessions: AgentSession[]<br/>per S1 §4.2 L329"]
        WorktreeS["worktrees: Worktree[]<br/>per S1 §4.2 L330"]
        WorkItemS["workItems: WorkItem[]<br/>per S1 §4.2 L331"]
    end

    Page -->|"render"| PageHeader
    Page -->|"render"| AF
    Page -->|"render"| ACV
    Page -->|"useStore 订阅 (顶层)<br/>per S1 §9.2 L740-742"| Store

    AF -->|"selectedId + auto + onChange"| Page
    ACV -->|"AgentCanvas + AgentSession + Worktree"| Page

    Page -->|"useMemo 调用<br/>per S1 §2.2 L122-128"| Sel
    Page -->|"useMemo 调用"| Lay

    Sel -->|"isActiveAgent<br/>pickDefaultAgent<br/>resolveCurrentAgent<br/>pickAgentWorktree<br/>pickAgentWorkItems"| Types
    Lay -->|"layoutAgentCanvas<br/>fitToContentViewport"| Types

    SelTest -.->|test| Sel
    LayTest -.->|test| Lay

    Sel -->|"读 agentSessions/worktrees/workItems"| Store
    Lay -->|"读 + 处理 派生节点"| Store

    Store --> AgentS
    Store --> WorktreeS
    Store --> WorkItemS
```

---

## 2. 派生链 (Derivation Chain, per [[S1]] §2.3 L146-174)

```mermaid
flowchart LR
    subgraph Input["输入 (per S1 §2.2 L120-130)"]
        A1["store.agentSessions"]
        A2["store.worktrees"]
        A3["store.workItems"]
        A4["URL ?agent=ag-XXX<br/>(per S1 §3.1 FR-AGV-011)"]
    end

    subgraph PureFns["7 公开纯函数 (per S1 §2.3 L149-168)"]
        F1["1. isActiveAgent(a) → boolean<br/>FR-AGV-001"]
        F2["2. pickDefaultAgent(agents) → Agent | null<br/>FR-AGV-002"]
        F3["3. resolveCurrentAgent(agents, urlAgentId) → CurrentAgentResolution | null<br/>FR-AGV-011"]
        F4["4. pickAgentWorktree(worktrees, agent) → Worktree | null<br/>BR-2 1:1 关联"]
        F5["5. pickAgentWorkItems(workItems, agent, wt) → WorkItem[]<br/>BR-3 1:N via worktree_id"]
        F6["6. layoutAgentCanvas(input) → LayoutOutput<br/>FR-AGV-003 自由散开"]
        F7["7. fitToContentViewport(bbox, w, h, pad) → { x, y, zoom }<br/>FR-AGV-004"]
    end

    subgraph Output["输出 (per S1 §4.3.1-4.3.4)"]
        O1["AgentCanvas<br/>nodes + connectors + viewport + derivedAt"]
        O2["AgentCanvasNode[]<br/>3 kinds: agent/worktree/work_item"]
        O3["AgentCanvasConnector[]<br/>id/from/to/color/label"]
    end

    A1 --> F1
    A1 --> F2
    A1 --> F3
    A4 --> F3
    F2 --> F3
    F3 --> F4
    F3 --> F5
    F3 --> F6
    A2 --> F4
    A2 --> F6
    A3 --> F5
    A3 --> F6
    F4 --> F5
    F4 --> F6
    F5 --> F6
    F6 --> F7
    F6 --> O1
    F6 --> O2
    F6 --> O3
```

---

## 3. 模块依赖图 (per [[S1]] §6.2 L516-540)

```mermaid
flowchart TD
    M9["M-AGV-9 page.tsx<br/>root"]
    M6["M-AGV-6 AgentCanvasView<br/>SVG canvas"]
    M8["M-AGV-8 AgentFilter<br/>dropdown"]
    M2["M-AGV-2 selectors.ts<br/>7 pure fns"]
    M3["M-AGV-3 layout.ts<br/>2 pure fns + 1 helper"]
    M1["M-AGV-1 types.ts<br/>leaf"]
    Store["zustand store<br/>M-AGV-9 dep"]
    M4["M-AGV-4 layout.test"]
    M5["M-AGV-5 selectors.test"]
    M7["M-AGV-7 AgentCanvasView.test"]
    M10["M-AGV-10 nav/registry.ts<br/>+ Bot icon"]
    StatusPill["StatusPill<br/>共享组件"]

    M9 --> M6
    M9 --> M8
    M9 --> M2
    M9 --> M3
    M9 --> M1
    M9 --> Store
    M6 --> M1
    M6 --> Store
    M6 --> StatusPill
    M8 --> M2
    M8 --> Store
    M2 --> M1
    M3 --> M1
    M4 --> M3
    M5 --> M2
    M7 --> M6
    M10 -.->|"Bot icon"| M9
```

**依赖方向**: 单向, 无循环 (per [[S1]] §6.2 L542-546).

---

## 4. 布局算法 (per [[S1]] §3.2 L201-275)

```mermaid
flowchart TD
    Input["LayoutInput<br/>{ agent, worktree, workItems }"]
    Init["(1) agent 节点<br/>中心 (0,0) 220×110<br/>per S1 §3.2 L211-218"]
    WTNode["(2) worktree 节点<br/>右侧 80px gap, 240×80<br/>per S1 §3.2 L222-232"]
    Connector1["(3) agent → wt connector<br/>color: #2f81f7 'executes on'<br/>per S1 §3.2 L235-241"]
    Sort["(4) sort workItems<br/>[status_order ASC, due_date ASC, id ASC]<br/>per S1 §3.2 L246-247"]
    Ring1["内圈 Ring1<br/>8 槽, R=280px<br/>per S1 §3.2 L252-262"]
    Ring2["外圈 Ring2<br/>12 槽, R=460px<br/>per S1 §3.2 L254-268"]
    Node["(5) wi 节点<br/>180×64"]
    Conn2["(6) wt → wi connector<br/>color per status<br/>per S1 §4.3.3 L378-386"]

    Input --> Init
    Init -->|"wt?"| WTNode
    WTNode --> Connector1
    WTNode --> Sort
    Sort --> Ring1
    Sort -->|"idx ≥ 8"| Ring2
    Ring1 --> Node
    Ring2 --> Node
    Node --> Conn2
```

---

## 5. 画布状态机 (per [[S1]] §5.1 L417-437)

```mermaid
stateDiagram-v2
    [*] --> IDLE
    IDLE --> PANNING: 中键 / pan tool / shift + mousedown
    PANNING --> IDLE: mouseup / mouseleave
    IDLE --> ZOOMING: wheel
    ZOOMING --> IDLE: wheel 结束
    IDLE --> SELECTING: select tool + hover 节点
    SELECTING --> IDLE: click 空白 / tool 切换
```

---

## 6. 节点视觉规格 (per [[S1]] §7.2 L627-633)

| 节点 | 尺寸 | 背景 | 边框 (默认/hover/select) | 内容 |
|---|---|---|---|---|
| **agent** | 220×110 | `#0d2849` | `#1f6feb` / `#2f81f7` / `#79c0ff` | Bot icon + kind + id + status pill + tokens + cost |
| **worktree** | 240×80 | `#161b22` | `#30363d` / `#2f81f7` / `#79c0ff` | GitBranch icon + "worktree" + branch + status pill |
| **work_item** | 180×64 | `#161b22` | `#30363d` / `#2f81f7` / `#79c0ff` | key + title + status pill + priority |

**Connector 颜色** (per [[S1]] §4.3.3 L378-386):
- `in_progress` → `#2f81f7` (info blue)
- `review` → `#d29922` (warn amber)
- `blocked` → `#f85149` (err red)
- `todo` → `#8b949e` (ink-dim)
- `done` → `#3fb950` (ok green)
- `wontfix` → `#6e7681` (ink-mute)

---

## 7. 路由表 (per [[S1]] §9.1 L724-734)

| 路径 | Method | Handler | 用途 |
|---|---|---|---|
| `/agent-view` | GET | `app/agent-view/page.tsx` | 主入口 |
| `/agent-view?agent=ag-XXX` | GET | 同上 | URL State override (FR-AGV-011) |
| `/agent?selected=ag-XXX` | GET | `app/agent/page.tsx` | 双击 agent 跳详情 |
| `/worktree?selected=wt-XXX` | GET | `app/worktree/page.tsx` | 双击 worktree 跳详情 |
| `/work-item?selected=wi-XXX` | GET | `app/work-item/page.tsx` | 双击 wi 跳详情 |
| `/board?worktree_id=wt-XXX` | GET | `app/board/page.tsx` | header Kanban 跳 |
| `/agents` | GET | `app/(app)/agents/page.tsx` | 空状态跳 |

---

## 8. 异常处理 (per [[S1]] §5.3 L484-493)

| # | 异常 | 触发 | 处理 | 返回 |
|---|---|---|---|---|
| EX-1 | store.agentSessions.length === 0 | 渲染空状态 | 空 state + 跳 /agents | 空 |
| EX-2 | resolveCurrentAgent 返回 null | 渲染空状态 | "No resolvable agent" | 空 |
| EX-3 | worktree 为 null (agent 没 wt) | layout 跳过 wt 节点 | 只画 agent | 部分画布 |
| EX-4 | workItems 为空 | layout 跳过 wi 节点 | 画 agent + wt + 1 connector | 部分画布 |
| EX-5 | URL `?agent=ag-XXX` 找不到 | fallback 默认 | auto=true | 默认 + auto 角标 |
| EX-6 | worktree_id 引用问题 | renderNode 跳过 wi 节点 | 部分画布 | 部分 |

---

## 已知缺口 (per 守门 #11)

- **G-1**: 不画入 MCP/Star-LG 后端 (per [[S1]] §0 dual-use 提醒, Agent View 是 SPA in-memory, 0 外部服务)
- **G-2**: 不画入 zustand store 内部 action (per [[S1]] §9.2 L744 NFR-7 只读, 不调 action)
- **G-3**: 不画入 minimap 点击跳转 (per [[S1]] §1.2 缺口 #6)
- **G-4**: 不画入 i18n agent/worktree status 字典 (per [[S1]] §1.2 缺口 #5)
- **G-5**: 不画入 canvas 持久化 (per [[S1]] §1.2 缺口 #3)
- **G-6**: 不画入 minimap 节点位置 click handler (per [[S1]] §1.2 缺口 #6)






## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

- [[S1]]
- [[View-AgentView]]
- [[M-AGV-1]]
- [[M-AGV-2]]
- [[M-AGV-3]]
- [[M-AGV-4]]
- [[M-AGV-5]]
- [[M-AGV-6]]
- [[M-AGV-7]]
- [[M-AGV-8]]
- [[M-AGV-9]]
- [[M-AGV-10]]
- [[domain-worktree]]
- [[domain-work-item]]

### 2. 横向相关 (related)

- [[00-design-topology]]
- [[02-orchestration-langgraph]]

### 3. 参见 (see-also)

- [[S1]]
- [[View-AgentView]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/01-ui-agent-view.canvas`
- Obsidian Canvas 插件打开, 节点按 sub-graph 分色, 边显式标

### 5. 节点笔记索引

- 152 份节点笔记位于 `docs/wiki/docswiki/nodes/`
- 节点 ID = 文件名 (e.g. `C-01.md` / `domain-tenant.md` / `M-N1.md`)

### 6. 守门实证 (本段 v0.2 NEW)

- 0 回溯叙事 (per 守门 #12)
- 100% 文档实证 (per 守门 #12)
- 缺标比错标 (per 守门 #11)
- 3 view 平行, 不建立业务子域↔DDD 映射 (per 守门 #3)
- 修订 author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)
