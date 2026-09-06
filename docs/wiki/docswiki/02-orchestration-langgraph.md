---
title: '02 — LangGraph Orchestration 拓扑'
date: 2026-09-06
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ 2026-09-06 Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ 2026-09-06 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: ["S2", "S3", "View-LangGraph", "C-01", "C-02", "C-03", "C-04", "C-05", "C-06", "C-07", "C-08", "C-09", "C-10", "C-11", "C-12", "C-13", "C-14", "C-15", "C-16", "C-17", "C-18", "C-19", "C-20", "C-21", "C-22", "T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7", "M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7", "SA-01", "SA-02", "SA-03", "SA-04", "SA-05", "SA-06", "SA-07", "SA-08", "SA-09", "SA-10", "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11", "M-12", "M-13", "M-14", "M-15", "M-16", "M-17", "M-18", "M-19", "M-20", "M-21", "M-22", "M-23", "M-24", "M-25"]
related: ["00-design-topology", "03-runtime-ecs", "05-persistence-checkpoint", "06-data-flow"]
see-also: ["S2", "S3", "View-LangGraph"]
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
  - orchestration
  - langgraph
  - tmo
  - sub-agent
  - obsidian-wiki
  - design-topology
  - obsidian-wiki
  - design-topology
---





# 02 — LangGraph Orchestration 拓扑 (L0 + TMO + L1 SA + Cross-Cutting)

> **数据源**: [[S2]] [`docs/architecture/2026-09-03-langgraph/02-basic-design.md`](../../architecture/2026-09-03-langgraph/02-basic-design.md) + [[S3]] [`docs/architecture/2026-09-03-langgraph/03-detailed-design.md`](../../architecture/2026-09-03-langgraph/03-detailed-design.md)
> **范围**: star-lg Python crate, 2-level hierarchical LangGraph (L0 全体代理 + L1 任务卡子代理), 22 组件 ([[C-01]]..[[C-22]]), 25 模块 ([[M-01]]..[[M-25]]), 9+1 SA 类型, TMO 7 节点 ([[M-N1]]..[[M-N7]])

---

## 1. L0 全体代理 (Top Agent, per [[S2]] §2.1)

```mermaid
flowchart TB
    subgraph L0["L0 全体代理 StateGraph (singleton per session)"]
        direction LR
        T1["T-N1 parse_intent<br/>LLM + intent classifier<br/>per S2 §2.1.2 L200"]
        T2["T-N2 dispatch<br/>SubAgentPool.spawn<br/>per S2 §2.1.2 L201"]
        T3["T-N3 tool_node<br/>direct MCP<br/>per S2 §2.1.2 L202"]
        T4["T-N4 collect<br/>asyncio.gather<br/>per S2 §2.1.2 L203"]
        T5["T-N5 respond<br/>LLM + aggregate<br/>per S2 §2.1.2 L204"]
        T6["T-N6 interrupt<br/>top-level<br/>per S2 §2.1.2 L205"]
        T7["T-N7 guard_check<br/>守门 #4/#9/#12/#13<br/>per S2 §2.1.2 L206"]
    end

    State["TopAgentState TypedDict<br/>per S2 §2.1.1 L172-194<br/>v0.2 TMO 扩展: per S2 §2.6.4 L552-563"]

    Start([User input]) --> T1
    T1 -->|"intent=tool_call"| T3
    T1 -->|"intent=dispatch"| T2
    T1 -->|"intent=clarify"| T6
    T1 -->|"else"| T5
    T2 -->|"spawn N sub-agents"| T4
    T3 --> T5
    T4 -->|"violations? critical"| T6
    T4 -->|"ok"| T5
    T5 --> End([last_response])
    T6 -->|"interrupt_response"| T1
    T1 -.->|"writes"| State
    T2 -.->|"add active_subagents"| State
    T4 -.->|"add completed_subagents"| State
    T7 -.->|"violations list"| State
```

---

## 2. TMO 7 节点 (Task Management Operations, v0.2, per [[S2]] §2.6.1)

```mermaid
flowchart LR
    ParseIntent["parse_intent_node<br/>(T-N1)"]
    RouteTMO["route_after_parse_intent_tmo<br/>per S2 §2.6.3 L535-546"]

    subgraph TMO["★ TMO 7 节点 (TaskOperationsManager C-16 / M-19 唯一 cross-task actor)"]
        direction TB
        MN1["M-N1 merge_node<br/>a+b → merged_task<br/>stash + supersede<br/>per S2 §2.6.1 L512"]
        MN2["M-N2 split_node<br/>a → a1 + a2<br/>snapshot + fork<br/>per S2 §2.6.1 L513"]
        MN3["M-N3 reorder_node<br/>DAG 边调整<br/>per S2 §2.6.1 L514"]
        MN4["M-N4 bulk_node<br/>N 张卡批量 action<br/>per S2 §2.6.1 L515"]
        MN5["M-N5 summarize_node<br/>跨 N SubAgentState<br/>per S2 §2.6.1 L516"]
        MN6["M-N6 reassign_node<br/>SA-XX 类型切换<br/>per S2 §2.6.1 L517"]
        MN7["M-N7 metadata_node<br/>task_metadata 表<br/>per S2 §2.6.1 L518"]
    end

    ParseIntent -->|"intent=task_merge"| RouteTMO
    RouteTMO --> MN1
    RouteTMO -->|"task_split"| MN2
    RouteTMO -->|"set_dependencies"| MN3
    RouteTMO -->|"bulk_action"| MN4
    RouteTMO -->|"summarize"| MN5
    RouteTMO -->|"reassign"| MN6
    RouteTMO -->|"metadata"| MN7
    RouteTMO -->|"else"| Respond["respond_node (T-N5)"]
```

**核心约束 (per 守门 #13 a)**: TMO 7 节点全部 L0 协调, 跨任务操作只经 L0, **禁止 L1↔L1** (per [[S2]] §2.6 L506).

---

## 3. TMO 7 协议 (per [[S2]] §2.6.2 L521-530)

```mermaid
flowchart LR
    subgraph TMO_C["★ TMO 协议 (L0 → L1 / L0 → UI)"]
        P1["merge_request<br/>M-N1 → L1<br/>target_task_ids + merge_strategy"]
        P2["split_request<br/>M-N2 → L1<br/>target_task_id + split_strategy"]
        P3["dep_set<br/>M-N3 → L1<br/>DAG 边集合 + C-20 校验"]
        P4["bulk_action<br/>M-N4 → L1<br/>target_task_ids + action"]
        P5["reassign_request<br/>M-N6 → L1<br/>+ preserved_checkpoint_id"]
        P6["metadata_update<br/>M-N7 → L0/L0<br/>TaskCardManager (C-07)"]
        P7["summarize_result<br/>M-N5 → UI<br/>task_summaries (Table)"]
    end

    L0TMO["L0 TMO 7 节点"]
    L1Pool["L1 SubAgentPool"]
    UIC["UI 端"]
    TaskCardMgr["TaskCardManager C-07"]
    DAGVal["DAGValidator C-20"]

    L0TMO --> P1 --> L1Pool
    L0TMO --> P2 --> L1Pool
    L0TMO --> P3 --> DAGVal
    L0TMO --> P4 --> L1Pool
    L0TMO --> P5 --> L1Pool
    L0TMO --> P6 --> TaskCardMgr
    L0TMO --> P7 --> UIC
```

---

## 4. L1 Sub-Agent Pool + 9+1 SA (per [[S2]] §2.2.2 + [[S3]] §3.5)

```mermaid
flowchart TB
    subgraph Pool["SubAgentPool (C-02 + M-04, per S2 §2.2 + S3 §2.1)"]
        PoolImpl["_pools: dict[str, SubAgentHandle]<br/>_dispatch_queue: asyncio.Queue<br/>per S2 §2.3.2 L348-381"]
    end

    subgraph Common["★ 共通 5 节点 模板 (per S2 §2.2.3 L274-313)"]
        direction LR
        InitN["init"]
        PlanN["plan<br/>LLM"]
        ExecN["execute<br/>task_type 特定"]
        GuardN["guard_check<br/>守门拦截"]
        VerifyN["verify<br/>守门 #1 / #12 / cargo test"]
        ReportN["report"]
    end

    subgraph SATypes["9+1 SA 类型 (per S2 §2.2.2 + S3 §3.5 L1452+)"]
        SA1["SA-01 code-review<br/>review-plan→review-execute→review-report<br/>per S2 §2.2.2 L261"]
        SA2["SA-02 test-gen<br/>test-plan→test-execute→test-verify<br/>per S2 §2.2.2 L262"]
        SA3["★ SA-03 5-域-lead-audit<br/>跨 22 domain crates + 5 域治理矩阵<br/>per S2 §2.2.2 L263"]
        SA4["SA-04 git-ops<br/>ops-plan→ops-execute→ops-verify<br/>per S2 §2.2.2 L265"]
        SA5["SA-05 doc-sync<br/>AGENTS.md / WBS / ADR<br/>per S2 §2.2.2 L266"]
        SA6["SA-06 refactor<br/>refactor-plan→refactor-execute→refactor-verify<br/>per S2 §2.2.2 L267"]
        SA7["SA-07 db-migration<br/>per 守门 #13 W/T/M<br/>per S2 §2.2.2 L268"]
        SA8["SA-08 domain-dev<br/>DDD bounded context<br/>per S2 §2.2.2 L269"]
        SA9["SA-09 free-form<br/>默认 fallback<br/>per S2 §2.2.2 L270"]
        SA10["★ SA-10 task-orchestrator<br/>(v0.2 TMO 跨任务编排型, NEW)<br/>per S3 §1.1 L66"]
    end

    Pool --> InitN
    InitN --> PlanN
    PlanN -->|"proceed"| ExecN
    PlanN -->|"need_user_input"| InterruptN["interrupt"]
    PlanN -->|"abort"| ReportN
    ExecN --> GuardN
    GuardN -->|"ok"| VerifyN
    GuardN -->|"critical_violation"| InterruptN
    VerifyN -->|"ok"| ReportN
    VerifyN -->|"retry"| ExecN
    VerifyN -->|"abort"| ReportN
    ReportN --> EndN([END])

    Pool -->|"spawn"| SA1
    Pool -->|"spawn"| SA2
    Pool -->|"spawn"| SA3
    Pool -->|"spawn"| SA4
    Pool -->|"spawn"| SA5
    Pool -->|"spawn"| SA6
    Pool -->|"spawn"| SA7
    Pool -->|"spawn"| SA8
    Pool -->|"spawn"| SA9
    Pool -->|"spawn"| SA10

    SA1 -.->|uses| Common
    SA2 -.->|uses| Common
    SA3 -.->|uses| Common
    SA10 -.->|"TMO 跨任务"| Common
```

---

## 5. 跨切关注点 (Cross-Cutting Components, per [[S2]] §1.1 L101-109)

```mermaid
flowchart TB
    subgraph CC["Cross-Cutting (per S2 §1.3 L140-163)"]
        C04["C-04 CheckpointStore<br/>3-tier ABC<br/>M-08 checkpoints.store<br/>per S3 §1.2 L161"]
        C05["C-05 McpClient<br/>star-mcp 16 tools proxy<br/>M-10 mcp.client<br/>per S3 §1.2 L163"]
        C06["C-06 UIStreamer<br/>WebSocket / SSE / REST<br/>M-12 ui.streamer<br/>per S3 §1.2 L165"]
        C07["C-07 TaskCardManager<br/>UI ↔ Sub-agent mirror<br/>per S2 §1.3 L147"]
        C08["C-08 AuditLogger<br/>全 tool_call 記録<br/>M-13 cross_cutting.audit_logger<br/>per S3 §1.2 L166"]
        C09["C-09 TokenTelemetry<br/>token 計量 + OLU 集計<br/>M-14 cross_cutting.token_telemetry<br/>per S3 §1.2 L167"]
        C10["C-10 GuardEnforcer<br/>AGENTS.md §4 37 项<br/>M-15 cross_cutting.guard_enforcer<br/>per S3 §1.2 L168"]
        C11["C-11 StateSchemaRegistry<br/>LangGraph state schema<br/>M-18 schema.registry<br/>per S3 §1.2 L171"]
        C12["C-12 InterruptManager<br/>human-in-the-loop<br/>M-16 cross_cutting.interrupt_manager<br/>per S3 §1.2 L169"]
        C13["C-13 SubAgentRegistry<br/>SA 类型注册表<br/>per S2 §1.3 L154"]
        C15["C-15 HealthCheck<br/>/api/health<br/>per S2 §1.3 L156"]
    end

    SAPool["L1 SubAgentPool"]
    TopAgent["L0 TopAgent"]
    McpServer["star-mcp 16 tools"]
    UI["UI Tier"]
    PG["PostgreSQL Tier 3"]
    SQLite["SQLite Tier 2"]
    Memory["Memory Tier 1"]

    TopAgent --> C04
    SAPool --> C04
    SAPool --> C05
    TopAgent --> C05
    C05 -->|"stdio/Streamable HTTP"| McpServer
    C10 -->|"37 项检查"| C05
    C08 -->|"log entry"| C05
    C09 -->|"record"| C05
    C05 -.->|audit| C08
    C04 --> Memory
    C04 --> SQLite
    C04 --> PG
    TopAgent --> C12
    C12 --> SAPool
    TopAgent --> C06
    SAPool --> C06
    C06 -->|"WS/SSE/REST 3 通道"| UI
    C07 -.->|mirror state| UI
    SAPool -.->|registry| C13
    TopAgent -.->|registry| C13
    SAPool -.->|state schema| C11
```

---

## 6. 通信协议 (per [[S2]] §2.3.1 L317-340)

```mermaid
flowchart LR
    subgraph Down["L0 → L1 (downstream)"]
        D1["dispatch<br/>task_id, task_type, context, parent_task_id"]
        D2["cancel<br/>task_id, reason"]
        D3["interrupt_response<br/>task_id, decision, payload"]
        D4["merge_request (TMO M-N1)"]
        D5["split_request (TMO M-N2)"]
        D6["dep_set (TMO M-N3)"]
        D7["bulk_action (TMO M-N4)"]
        D8["reassign_request (TMO M-N6)"]
    end

    subgraph Up["L1 → L0 (upstream)"]
        U1["progress<br/>task_id, status, partial_output, node_id"]
        U2["result<br/>task_id, status=done, final_result, token_usage"]
        U3["interrupt_request<br/>task_id, decision_needed, options, default"]
        U4["error<br/>task_id, status=failed, error_msg, stack_trace"]
    end

    subgraph UI_Comm["L0/L1 ↔ UI (bidirectional)"]
        I1["stream (L0/L1 → UI)<br/>type: token/state/event"]
        I2["user_input (UI → L0)<br/>text, attachments"]
        I3["card_action (UI → L0)<br/>task_id, pause/resume/cancel"]
        I4["tmo_action (UI → L0)<br/>operation + target_task_ids + payload"]
        I5["summarize_result (L0 → UI)"]
    end

    subgraph L1L1["L1 ↔ L1"]
        BANNED["❌ 禁止 (per 守门 #13 a)"]
    end

    TopAgent["L0 TopAgent"] --> D1
    TopAgent --> D2
    TopAgent --> D3
    TopAgent --> D4
    TopAgent --> D5
    TopAgent --> D6
    TopAgent --> D7
    TopAgent --> D8

    SAPool["L1 SA"] --> U1
    SAPool --> U2
    SAPool --> U3
    SAPool --> U4

    TopAgent <-->|"stream + user_input"| UI_Comm
    SAPool <-->|"sub stream"| UI_Comm
```

**L1↔L1 禁止 (per 守门 #13 a)**: 防止状态污染, 跨任务操作全部走 L0 协调 (per [[S2]] §2.6 L506).

---

## 7. 22 组件清单 ([[C-01]]..[[C-22]], per [[S2]] §1.3 L140-163)

| ID | 名称 | 層 | 重要度 | TMO v0.2 |
|---|---|---|---|---|
| **[[C-01]]** | TopAgent | L0 | P0 | — |
| **[[C-02]]** | SubAgentPool | L0 | P0 | — |
| **[[C-03]]** | SubAgent (9 types) | L1 | P0 | — |
| **[[C-04]]** | CheckpointStore | Cross | P0 | — |
| **[[C-05]]** | McpClient | L2 | P0 | — |
| **[[C-06]]** | UIStreamer | Cross | P0 | — |
| **[[C-07]]** | TaskCardManager | L0/L1 | P0 | — |
| **[[C-08]]** | AuditLogger | Cross | P0 | — |
| **[[C-09]]** | TokenTelemetry | Cross | P1 | — |
| **[[C-10]]** | GuardEnforcer | Cross | P1 | — |
| **[[C-11]]** | StateSchemaRegistry | Cross | P1 | — |
| **[[C-12]]** | InterruptManager | L0/L1 | P0 | — |
| **[[C-13]]** | SubAgentRegistry | L0 | P1 | — |
| **[[C-14]]** | CrossDomainDispatcher | L2 | P2 | — |
| **[[C-15]]** | HealthCheck | Cross | P1 | — |
| **[[C-16]]** | TaskOperationsManager | L0 | P0 | ★ v0.2 |
| **[[C-17]]** | TaskRelationshipGraph | L0/L1 | P0 | ★ v0.2 |
| **[[C-18]]** | BulkOperationQueue | L0 | P0 | ★ v0.2 |
| **[[C-19]]** | MetadataRegistry | L0 | P1 | ★ v0.2 |
| **[[C-20]]** | DAGValidator | L0 | P0 | ★ v0.2 |
| **[[C-21]]** | ReassignManager | L0 | P1 | ★ v0.2 |
| **[[C-22]]** | SummarizeCollector | L0 | P1 | ★ v0.2 |

---

## 8. 25 模块清单 ([[M-01]]..[[M-25]], per [[S3]] §1.2 L150-178)

| ID | 名称 | 責務 | 公開 interface | 依存 |
|---|---|---|---|---|
| [[M-01]] | `top_agent.graph` | TopAgent StateGraph 定義 | `TopAgent` class | sub_agent.pool, checkpoints, mcp |
| [[M-02]] | `top_agent.nodes` | [[T-N1]]..[[T-N7]] 実装 | `parse_intent_node`, `dispatch_node` | mcp.client, sub_agent.pool, llm |
| [[M-03]] | `top_agent.state` | TopAgentState TypedDict | `TopAgentState` | — |
| [[M-04]] | `sub_agent.pool` | spawn / lifecycle | `SubAgentPool.spawn()`, `.cancel()` | sub_agent.handle, sub_agent.registry |
| [[M-05]] | `sub_agent.base` | 共通 5 节点 模板 | `make_subagent_graph(task_type)` | sub_agent.state, mcp.audited_tool_node |
| [[M-06]] | `sub_agent.types` | [[SA-01]]..[[SA-09]] 実装 | `SA_01_CODE_REVIEW`... | sub_agent.base |
| [[M-07]] | `sub_agent.registry` | 类型 → 実装 mapping | `register(type, factory)`, `get(type)` | sub_agent.types |
| [[M-08]] | `checkpoints.store` | 3-tier ABC | `CheckpointStore` (abstract) | — |
| [[M-09]] | `checkpoints.sqlite` | Tier 2 実装 (default v0.1) | `SqliteCheckpointer` | checkpoints.store |
| [[M-10]] | `mcp.client` | star-mcp 16 tools proxy | `McpClient.call(tool, params)` | mcp tool metadata |
| [[M-11]] | `mcp.audited_tool_node` | audit + guard ToolNode | `AuditedMcpToolNode` | cross_cutting.audit_logger, guard_enforcer |
| [[M-12]] | `ui.streamer` | WebSocket / SSE 推送 | `UIStreamer.push(msg)`, `.subscribe(ws)` | — |
| [[M-13]] | `cross_cutting.audit_logger` | 全 tool call 記録 | `AuditLogger.log(entry)` | db (per 守门 #13 T) |
| [[M-14]] | `cross_cutting.token_telemetry` | token 計量 | `TokenTelemetry.record(call, result)` | — |
| [[M-15]] | `cross_cutting.guard_enforcer` | AGENTS.md §4 37 项 自动检查 | `GuardEnforcer.check_tool_call(call)` | — |
| [[M-16]] | `cross_cutting.interrupt_manager` | human-in-loop interrupt / resume | `InterruptManager.interrupt/resume` | — |
| [[M-17]] | `api.app` | FastAPI app + 路由 mount | `create_app()` | api.routes_* |
| [[M-18]] | `schema.registry` | State schema 中央管理 | `StateSchemaRegistry.register/migrate` | schema.v1, schema.migration |
| [[M-19]] | `task_ops.manager` | TMO 7 节点 集中调度, 唯一 cross-task actor | `TaskOperationsManager.merge/split/.../metadata()` | sub_agent.pool, task_ops.relationship_graph, sub_agent.registry |
| [[M-20]] | `task_ops.relationship_graph` | 任务卡 DAG (4 字段), cycle prevention | `TaskRelationshipGraph.add_edge/set/get/has_cycle()` | — |
| [[M-21]] | `task_ops.bulk_queue` | 批量操作队列 + asyncio.gather, 部分失败回滚 | `BulkOperationQueue.enqueue/flush()` | sub_agent.pool, task_ops.manager |
| [[M-22]] | `task_ops.dag_validator` | cycle detection O(V+E), 检测到环 → reject + interrupt | `DAGValidator.validate(relationships)` | task_ops.relationship_graph |
| [[M-23]] | `task_ops.metadata_registry` | task_metadata 表中央管理 (Master RLS per 守门 #13 c) | `MetadataRegistry.update/get` | db (守门 #13 M 表) |
| [[M-24]] | `task_ops.reassign_manager` | SA-XX 类型切换 + checkpoint preserved | `ReassignManager.reassign(task_id, new_type)` | sub_agent.pool, sub_agent.registry, checkpoints.store |
| [[M-25]] | `task_ops.summarize_collector` | 跨 N SubAgentState 聚合, LLM 表格化 | `SummarizeCollector.collect/llm_summarize` | sub_agent.pool, llm |

---

## 9. 外部 API 端点 (per [[S2]] §5.2 L960-983)

| Endpoint | Method | 用途 | TMO |
|---|---|---|---|
| `/api/top-agent/dispatch` | POST | UI → Top, user input | — |
| `/api/top-agent/stream` | WS | Top → UI, streaming | — |
| `/api/top-agent/state` | GET | UI → Top, state poll | — |
| `/api/top-agent/cancel` | POST | UI → Top, cancel all | — |
| `/api/sub-agent/{task_id}/stream` | WS | Sub → UI, streaming | — |
| `/api/sub-agent/{task_id}/state` | GET | UI → Sub, state | — |
| `/api/sub-agent/{task_id}/interact` | POST | UI → Sub, interrupt_response | — |
| `/api/sub-agent/{task_id}/cancel` | POST | UI → Sub, cancel | — |
| `/api/tasks` | GET | UI → backend, list all | — |
| `/api/tasks/{task_id}` | GET | UI → backend, task detail | — |
| `/api/health` | GET | UI / monitoring → backend | — |
| `/api/metrics` | GET | monitoring → backend (Prometheus) | — |
| `/api/tmo/merge` | POST | UI → Top, TMO [[M-N1]] 合并 a+b | ★ |
| `/api/tmo/split` | POST | UI → Top, TMO [[M-N2]] 拆分 a→a1+a2 | ★ |
| `/api/tmo/dependencies` | POST | UI → Top, TMO [[M-N3]] dep_set | ★ |
| `/api/tmo/bulk` | POST | UI → Top, TMO [[M-N4]] 批量 action | ★ |
| `/api/tmo/summarize` | POST | UI → Top, TMO [[M-N5]] 跨任务汇总 | ★ |
| `/api/tmo/reassign` | POST | UI → Top, TMO [[M-N6]] 类型 SA-XX 切换 | ★ |
| `/api/tmo/metadata` | POST | UI → Top, TMO [[M-N7]] task_metadata 更新 | ★ |
| `/api/tmo/relationships` | GET | UI → Top, 查询 DAG 边 | ★ |

---

## 10. LangGraph State Schema (per [[S3]] §3.1)

```mermaid
classDiagram
    class TopAgentState {
        +user_input: str
        +intent: Optional[str]
        +active_subagents: list[SubAgentRef]
        +completed_subagents: list[SubAgentResult]
        +conversation_history: list[Message]
        +global_context: dict
        +last_response: Optional[str]
        +interrupt_id: Optional[str]
        +interrupt_response: Optional[dict]
        ★ +task_relationships: dict  (v0.2 TMO)
        ★ +superseded_tasks: list  (v0.2 TMO)
        ★ +bulk_operations: queue  (v0.2 TMO)
        ★ +last_summarize_result: list  (v0.2 TMO)
        ★ +active_tmo_operation: Optional  (v0.2 TMO)
    }
    class SubAgentState {
        +task_id: str
        +task_type: SA_01..SA_09
        +context: dict
        +intermediate_steps: list[Step]
        +final_result: Optional[Any]
        +status: str
        +checkpoint_id: Optional[str]
        +error: Optional[str]
        +started_at: datetime
        +completed_at: Optional[datetime]
        +token_usage: dict
        +guard_violations: list[GuardViolation]
        ★ +parent_task_id: Optional  (v0.2 TMO 血缘)
        ★ +merged_from: list  (v0.2 TMO)
        ★ +split_into: list  (v0.2 TMO)
        ★ +superseded_by: Optional  (v0.2 TMO)
        ★ +checkpoint_snapshot: Optional  (v0.2 TMO)
    }
    TopAgentState "1" -- "0..N" SubAgentState: active_subagents/completed_subagents
```

---

## 已知缺口 (per 守门 #11)

- **G-1**: `state_schema_v1` 跟 v0.2 TMO migration 路径细节未画 (per [[S3]] §3.1 + 04-state-schema-v1-migration.md)
- **G-2**: [[SA-10]] task-orchestrator subgraph 内部节点未展开 (per [[S3]] §1.1 L66 提及, 3.5 仅列出)
- **G-3**: schema/migration.py 跨版本迁移算法不画 (per [[S3]] §1.1 L128)
- **G-4**: 不画入 9 SA × ECS Archetype 业务逻辑具体实现 (per [[S4]] §3.5 G-13 已知缺口)
- **G-5**: 22 domain 真实数据接入状态 (部分接入) 不画 (per AGENTS.md §7 #1 11/25 部分)
- **G-6**: 5 域 Lead 真人未到位, [[SA-03]] audit 暂以 Mavis 临时代签 (per 守门 #14 v2)






## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

- [[S2]]
- [[S3]]
- [[View-LangGraph]]
- [[C-01]]
- [[C-02]]
- [[C-03]]
- [[C-04]]
- [[C-05]]
- [[C-06]]
- [[C-07]]
- [[C-08]]
- [[C-09]]
- [[C-10]]
- [[C-11]]
- [[C-12]]
- [[C-13]]
- [[C-14]]
- [[C-15]]
- [[C-16]]
- [[C-17]]
- [[C-18]]
- [[C-19]]
- [[C-20]]
- [[C-21]]
- [[C-22]]
- [[T-N1]]
- [[T-N2]]
- [[T-N3]]
- [[T-N4]]
- [[T-N5]]
- [[T-N6]]
- [[T-N7]]
- [[M-N1]]
- [[M-N2]]
- [[M-N3]]
- [[M-N4]]
- [[M-N5]]
- [[M-N6]]
- [[M-N7]]
- [[SA-01]]
- [[SA-02]]
- [[SA-03]]
- [[SA-04]]
- [[SA-05]]
- [[SA-06]]
- [[SA-07]]
- [[SA-08]]
- [[SA-09]]
- [[SA-10]]
- [[M-01]]
- [[M-02]]
- [[M-03]]
- [[M-04]]
- [[M-05]]
- [[M-06]]
- [[M-07]]
- [[M-08]]
- [[M-09]]
- [[M-10]]
- [[M-11]]
- [[M-12]]
- [[M-13]]
- [[M-14]]
- [[M-15]]
- [[M-16]]
- [[M-17]]
- [[M-18]]
- [[M-19]]
- [[M-20]]
- [[M-21]]
- [[M-22]]
- [[M-23]]
- [[M-24]]
- [[M-25]]

### 2. 横向相关 (related)

- [[00-design-topology]]
- [[03-runtime-ecs]]
- [[05-persistence-checkpoint]]
- [[06-data-flow]]

### 3. 参见 (see-also)

- [[S2]]
- [[S3]]
- [[View-LangGraph]]

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/02-orchestration-langgraph.canvas`
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
