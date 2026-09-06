---
title: '06 — 跨 View 关键数据流 (Data Flow)'
date: 2026-09-06
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ 2026-09-06 Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ 2026-09-06 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: ["S2", "S4", "C-01", "C-02", "C-03", "C-05", "C-06", "C-07", "C-12", "C-16", "C-17", "C-20", "M-N1", "T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7", "SA-08"]
related: ["00-design-topology", "02-orchestration-langgraph", "03-runtime-ecs", "05-persistence-checkpoint"]
see-also: ["S2", "S4"]
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
  - data-flow
  - sequence
  - websocket
  - mcp
  - obsidian-wiki
  - design-topology
  - obsidian-wiki
  - design-topology
---








# 06 — 跨 View 关键数据流 (Data Flow Topology)

> **数据源**: [[S2]] §4.2 交互フロー [LangGraph 02 §4.2](../../architecture/2026-09-03-langgraph/02-basic-design.md) + [[S3]] §4 时序图 [LangGraph 03 §4](../../architecture/2026-09-03-langgraph/03-detailed-design.md) + [[S4]] [Agent Runtime 02](../../architecture/2026-09-03-agent-runtime/02-basic-design.md)
> **目的**: 跨 view 关键数据流: User input → L0 → L1 → L2 → Tool → DB → Response, 含 TMO 跨任务操作流

---

## 1. 主流程: User Input → L0 → L1 → UI Stream (per [[S2]] §4.2.1 L839-884)

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant ChatBar as Chat Bar (UI)
    participant API as Backend API<br/>(FastAPI)
    participant Top as L0 Top Agent<br/>(T-N1..T-N7)
    participant Pool as L1 SubAgentPool
    participant SA as L1 Sub-Agent<br/>(SA-01..SA-09)
    participant MCP as star-mcp<br/>(16 tools)
    participant DB as DB<br/>(SQLite/PG)
    participant WS as UI WebSocket
    participant Card as Task Card Modal

    User->>ChatBar: 输入 "H2 8 domain 改造並列で"
    ChatBar->>API: POST /api/top-agent/dispatch<br/>{user_input}
    API->>Top: invoke(user_input)
    activate Top
    Top->>Top: T-N1 parse_intent_node<br/>LLM 意图分类
    Top->>Top: intent = "dispatch"<br/>subagent_plan = [SA-08 × 8]
    Top->>Pool: T-N2 dispatch_node<br/>spawn 8 sub-agents
    activate Pool
    Pool->>SA: asyncio.Queue.put<br/>(8 × SubAgentHandle)
    Pool->>WS: ui_streamer.push<br/>(TaskCardCreate × 8)
    WS-->>Card: append card to grid × 8<br/>animated_fade_in
    deactivate Pool
    loop 8 sub-agents 並行
        SA->>SA: init → plan → execute
        SA->>MCP: AuditedMcpToolNode<br/>(audit + guard)
        MCP-->>SA: tool_result
        SA->>DB: CheckpointStore.save<br/>(SQLite/PG)
        SA->>Pool: emit_progress<br/>(task_id, status, partial)
        Pool->>WS: push(TaskCardProgress × N)
        WS-->>Card: card.append_streaming_output
    end
    Top->>Pool: T-N4 collect_node<br/>asyncio.gather
    Pool-->>Top: completed_subagents × 8
    Top->>Top: T-N5 respond_node<br/>LLM 聚合
    Top->>DB: CheckpointStore.save<br/>(final state)
    Top->>WS: push(TopResponse)
    WS-->>ChatBar: chat_bar.append_assistant_message<br/>scroll to bottom
    deactivate Top
    User->>Card: click card → Modal<br/>全 state dump
    Card->>API: GET /api/sub-agent/{id}/state
    API-->>Card: SubAgentState
```

---

## 2. Human-in-the-Loop Interrupt 流 (per [[S2]] §4.2.2 L886-917)

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant SA as L1 Sub-Agent
    participant Top as L0 Top Agent
    participant WS as UI WebSocket
    participant Card as Task Card
    participant ChatBar as Chat Bar

    SA->>SA: execute 阶段遇到 critical decision<br/>(e.g., 守门 violation)
    SA->>SA: interrupt_node<br/>state.interrupt_id = uuid<br/>state.status = "waiting_input"
    SA->>Top: emit interrupt_request<br/>{task_id, decision_needed, options, default}
    Top->>WS: push(InterruptPrompt)
    WS-->>Card: overlay_decision_prompt<br/>card.status = "waiting_input"<br/>(黄色高亮)
    WS-->>ChatBar: chat_bar.badge += 1
    User->>Card: click "Approve" / "Modify" / "Cancel"
    Card->>API: POST /api/sub-agent/{id}/interact<br/>{decision: "approve"}
    API->>Top: send_interrupt_response<br/>(task_id, decision)
    Top->>SA: interrupt_response<br/>state.interrupt_response = {decision}
    SA->>SA: 恢复 execution
    SA->>WS: push(TaskCardProgress)
    WS-->>Card: card.status = "running"<br/>overlay_decision_prompt.remove
    SA->>SA: 继续执行
```

---

## 3. Agent View 派生流 (per [[S1]] §2.2 L117-142)

```mermaid
flowchart LR
    subgraph Inputs["输入"]
        AS["store.agentSessions"]
        WS2["store.worktrees"]
        WI["store.workItems"]
        URL["URL ?agent=ag-XXX"]
    end

    subgraph PureFns["7 公开纯函数 (per S1 §2.3)"]
        RCA["resolveCurrentAgent<br/>(F-3)"]
        PAW["pickAgentWorktree<br/>(F-4)"]
        PAWI["pickAgentWorkItems<br/>(F-5)"]
        LAC["layoutAgentCanvas<br/>(F-6)"]
        FTC["fitToContentViewport<br/>(F-7)"]
    end

    subgraph Output["输出"]
        AC["AgentCanvas<br/>nodes + connectors + viewport"]
        SVG["AgentCanvasView<br/>SVG 渲染"]
    end

    AS --> RCA
    URL --> RCA
    RCA -->|resolved agent| PAW
    WS2 --> PAW
    PAW --> PAWI
    AS --> PAWI
    WI --> PAWI
    PAWI --> LAC
    RCA --> LAC
    PAW --> LAC
    LAC --> FTC
    FTC --> AC
    AC --> SVG
    SVG -->|"节点 hover/select/dblclick"| User[("用户")]
```

---

## 4. Runtime Mode 切换流 (per [[S5]] §4.2 UC-02)

```mermaid
sequenceDiagram
    autonumber
    participant Monitor as Resident Monitor
    participant Mgr as RuntimeModeManager
    participant Sched as SchedulerSystem
    participant ECS as ECS World
    participant Tokio as Tokio Dispatcher

    Monitor->>Mgr: poll resident_agent_count
    alt Resident < 10
        Mgr-->>Tokio: mode = Lightweight
        Tokio->>Tokio: 1 Agent 1 Tokio task<br/>(struct 实例)
    else Resident ≥ 12 持续 30s
        Mgr-->>ECS: mode = ECS
        ECS->>ECS: 1 Agent 1 Entity<br/>(9 Archetype)
        ECS->>Sched: Systems frame loop
    else Resident 10-11 (Hysteresis)
        Mgr-->>Tokio: 保持当前模式
    end
    Note over Mgr,Tokio: 切换不丢 Event<br/>不重复 Tool<br/>不丢 Agent State<br/>不丢 ContextRef<br/>不重复 LLM 请求<br/>(per S4 §2.2 L155)
```

---

## 5. HOT → WARM → COLD Lifecycle 流 (per [[S5]] §3.2)

```mermaid
sequenceDiagram
    autonumber
    participant Sched as SchedulerSystem
    participant Agent as Agent Entity
    participant Life as LifecycleSystem
    participant Pers as PersistenceSystem
    participant Store as CheckpointStore
    participant Mem as Memory

    Sched->>Agent: 调度完成, last_active = now
    Agent->>Life: 闲置超时检测
    alt HOT → WARM
        Life->>Agent: current = Warm<br/>精简 Component
    else WARM → COLD
        Life->>Pers: persist(agent_id)
        Pers->>Store: CheckpointStore.save<br/>(SQLite/PG)
        Life->>Agent: current = Cold<br/>RAM 释放, 仅 Persist
    end
    alt Cold → Hot (restore)
        Sched->>Life: 重新调度
        Life->>Pers: restore(agent_id)
        Pers->>Store: CheckpointStore.load
        Store-->>Pers: state snapshot
        Pers-->>Life: Agent 重建
        Life->>Agent: current = Hot
    end
```

---

## 6. TMO 合并流 ([[M-N1]] merge, per [[S2]] §2.6.1 + [[S7]] §3.4)

```mermaid
sequenceDiagram
    autonumber
    actor User
    participant ChatBar as Chat Bar
    participant API as Backend API
    participant Top as L0 Top Agent
    participant MN1 as M-N1 merge_node
    participant Mgr as TaskOperationsManager (C-16)
    participant Pool as L1 SubAgentPool
    participant A as Sub-Agent a
    participant B as Sub-Agent b
    participant PG as PostgreSQL<br/>checkpoints

    User->>ChatBar: "合并任务 a 和任务 b"
    ChatBar->>API: POST /api/tmo/merge<br/>{target_task_ids: [a, b], merge_strategy}
    API->>Top: invoke tmo_action
    Top->>Top: parse_intent<br/>intent = "task_merge"
    Top->>MN1: route_after_parse_intent_tmo
    activate MN1
    MN1->>Mgr: TaskOperationsManager.merge(a, b)
    Mgr->>A: merge_request (L0→L1)<br/>stash_state (Transaction append-only)
    A->>PG: INSERT checkpoint_writes<br/>(superseded_tasks append)
    Mgr->>B: merge_request
    B->>PG: INSERT checkpoint_writes<br/>(superseded_tasks append)
    Mgr->>Pool: spawn merged_task (新 SA instance)
    Pool->>A: state.superseded_by = merged_task_id
    Pool->>B: state.superseded_by = merged_task_id
    Mgr->>PG: UPDATE checkpoints.state.last_response
    PG-->>Mgr: merged_task_id, stash_checkpoint_ids
    Mgr-->>MN1: 合并完成
    MN1-->>Top: task_relationships 更新
    Top->>WS: push(summarize_result)
    WS-->>ChatBar: 显示合并结果
    deactivate MN1
```

---

## 7. 跨 View 数据共享流 (per 守门 #3 反转 + [[S1]] §4.2)

```mermaid
flowchart LR
    subgraph SharedStores["★ Shared Frontend Stores (per S1 §4.2)"]
        direction TB
        AS2["agentSessions"]
        WT2["worktrees"]
        WI2["workItems"]
    end

    subgraph AgentView["Agent View (S1)"]
        AV1["AgentCanvasView<br/>M-AGV-6 读"]
        AV2["AgentFilter<br/>M-AGV-8 读"]
        AV3["page.tsx<br/>M-AGV-9 读"]
    end

    subgraph LGFrontend["LangGraph Frontend (S2 §4)"]
        LGF1["AppShell"]
        LGF2["TaskCardModal<br/>镜像 Sub-agent"]
        LGF3["Tab 4 Agents"]
    end

    subgraph LGBackend["LangGraph Backend (S2)"]
        LGB1["UIStreamer (C-06)"]
        LGB2["TaskCardManager (C-07)"]
    end

    subgraph RuntimeBackend["Agent Runtime Backend (S4)"]
        RTB1["Observer (metrics)"]
        RTB2["L0 Dispatcher"]
    end

    AS2 -.->|"useStore 读<br/>(per S1 §9.2)"| AgentView
    WT2 -.->|"useStore 读"| AgentView
    WI2 -.->|"useStore 读"| AgentView
    AS2 -.->|"mirror via TaskCardManager"| LGFrontend
    WT2 -.->|"mirror via TaskCardManager"| LGFrontend
    WI2 -.->|"mirror via TaskCardManager"| LGFrontend

    LGBackend -->|"WS /api/top-agent/stream"| LGFrontend
    RuntimeBackend -->|"Prometheus /api/metrics"| LGFrontend

    AgentView -.->|"不调 store action<br/>(per S1 NFR-7 只读)"| SharedStores
    LGFrontend -.->|"只读 mirror"| SharedStores
```

---

## 8. 通信通道总览 (per [[S2]] §1.2 L130-136)

```mermaid
flowchart TB
    subgraph Channels["★ 通信方向 (per S2 §1.2 L130-136)"]
        direction LR
        C1["L0 → L1: dispatch (downstream)"]
        C2["L1 → L0: progress / result / interrupt (upstream)"]
        C3["❌ L1 ↔ L1: 原則禁止 (per 守门 #13 a)"]
        C4["L0/L1 → L2: tool call (downstream)"]
        C5["L2 → L0/L1: tool result (upstream)"]
        C6["L0/L1 ↔ UI: stream (bidirectional WebSocket)"]
    end

    L0["L0 Top Agent"] -->|dispatch| L1["L1 Sub-Agent"]
    L1 -->|progress/result| L0
    L0 -->|tool call| L2["L2 Tool"]
    L2 -->|tool result| L0
    L1 -->|tool call| L2
    L2 -->|tool result| L1
    L0 <-->|stream WS/SSE| UI["UI Tier"]
    L1 <-->|stream WS/SSE| UI
```

---

## 已知缺口 (per 守门 #11)

- **G-1**: TMO split / reorder / bulk / summarize / reassign / metadata 6 子项详细时序不画 (per [[S2]] §2.6, 等 PHASE-LANGGRAPH-TMO-IMPL-REPORT 实装)
- **G-2**: 跨 session resume 协议 (per ADR-0030 Agent Lease/Heartbeat/Resume) 不画, 详见 `docs/architecture/2026-08-26-upgrade/spec/flows/03-agent-resume.md`
- **G-3**: Saga 5 步流程 (AuditLog/Notification/CacheInvalidate 等) 不画, 详见 `docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md`
- **G-4**: Backpressure Overflow Policy (per [[S5]] §6.3) 详细算法不画
- **G-5**: Multi-tenant isolation 13 類 RLS (per 守门 #13 d) 实施细节不画
- **G-6**: 5 域 Lead 真人未到位, RACI 跨域流暂以 Mavis 临时代签 (per 守门 #14 v2)









## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

- [[S2]]
- [[S4]]
- [[C-01]]
- [[C-02]]
- [[C-03]]
- [[C-05]]
- [[C-06]]
- [[C-07]]
- [[C-12]]
- [[C-16]]
- [[C-17]]
- [[C-20]]
- [[M-N1]]
- [[T-N1]]
- [[T-N2]]
- [[T-N3]]
- [[T-N4]]
- [[T-N5]]
- [[T-N6]]
- [[T-N7]]
- [[SA-08]]

### 2. 横向相关 (related)

- [[00-design-topology]]
- [[02-orchestration-langgraph]]
- [[03-runtime-ecs]]
- [[05-persistence-checkpoint]]

### 3. 参见 (see-also)

- [[S2]]
- [[S4]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/06-data-flow.canvas`
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
