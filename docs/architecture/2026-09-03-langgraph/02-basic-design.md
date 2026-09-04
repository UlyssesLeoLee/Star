# 02. Star LangGraph 統合アーキテクチャ - 基本設計書 (Basic Design)

> **状態**：🟢 Draft v0.2
> **日期**：2026-09-04 (升版自 v0.1)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权）
> **依赖**：[01-requirements.md](01-requirements.md)（要件定義書 v0.2）· [ADR-0032 MCP Transport stdio](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0030 Agent Lease/Heartbeat/Resume](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md)
> **关联文档**：[01-requirements.md](01-requirements.md)（要件定義書 v0.2）· [03-detailed-design.md](03-detailed-design.md)（詳細設計書 v0.2）· [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md)（7 子项实装计划）

---

## 0. 目的 (Purpose)

本文档基于 [01-requirements.md](01-requirements.md) §1-§3 的要件，定义 Star LangGraph 統合アーキテクチャ (Star-LG) 的基本設計：

- 系统架构（2-level hierarchical + 3 层）
- コンポーネント一覧 + 责任划分
- 数据模型 + LangGraph state schema
- 内部/外部 API 接口设计
- UI/UX 框架
- 安全/性能/运用/移行設計

> **重要区别 (per [01 §1.0](01-requirements.md))**: 本 view 设计的 "Sub-Agent" = 任务卡子代理 (UI 驱动, LangGraph in-process), **不是** Mavis worker subagent (worker/explorer/verifier, subprocess + brief)。两套系统并存, 任务卡子代理可在 plan/execute 阶段调用 worker subagent 走 subprocess 路径。详见 [01 §1.0](01-requirements.md) 区别表 + §9.1 移行設計。

## 1. システムアーキテクチャ (System Architecture)

### 1.1 全体構成図 (Overall Architecture)

```
┌────────────────────────────────────────────────────────────────────────────┐
│                       UI Tier (gm-console frontend)                         │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  AppShell (per AGENTS.md §7 #15 v0.15 5-tab: Kanban/Timeline/Backlog) │  │
│  │  ┌────────────────────────────────────────────────────────────────┐  │  │
│  │  │  AppHeader (Top bar: Star logo + Tab nav + Theme + User)        │  │  │
│  │  ├────────────────────────────────────────────────────────────────┤  │  │
│  │  │  Sidebar (w-56, 224px: 5-tab nav + Pinned Board)                │  │  │
│  │  ├────────────────────────────────────────────────────────────────┤  │  │
│  │  │  Main Content (Tab content area)                                │  │  │
│  │  │    •  Tab 1 Kanban: Board 列 + 卡片 (跨 sub-agent 状态 mirror)  │  │  │
│  │  │    •  Tab 2 Timeline: GanttChart (sub-agent timeline)           │  │  │
│  │  │    •  Tab 3 Backlog: workItems 列表                             │  │  │
│  │  │    •  Tab 4 Agents: ★ NEW ★ 所有 sub-agent 状态一览              │  │  │
│  │  │    •  Tab 5 Worktrees: worktrees 列表                           │  │  │
│  │  ├────────────────────────────────────────────────────────────────┤  │  │
│  │  │  ★ NEW ★ Chat Bar (固定底行, 全幅)                              │  │  │
│  │  │    ┌──────────────────────────────────────────────────────┐    │  │  │
│  │  │    │  [input: "H2 8 domain 改造並列で"  ]  [Send  ↑]      │    │  │  │
│  │  │    └──────────────────────────────────────────────────────┘    │  │  │
│  │  └────────────────────────────────────────────────────────────────┘  │  │
│  │  ★ NEW ★ Task Card Modal (点击 task card 详情)                        │  │
│  │    •  Sub-agent 名称 + 类型 + status                                  │  │
│  │    •  Latest streaming output                                         │  │
│  │    •  Controls: Pause / Resume / Cancel                               │  │
│  │    •  History tab: 节点执行履歴 + checkpoint info                      │  │
│  │    •  Decision tab: human-in-the-loop prompts                          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│   │ WebSocket (SSE-fallback)         │ HTTP REST (control)                  │
└───┼──────────────────────────────────┼──────────────────────────────────────┘
    │                                  │
    ▼                                  ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                Backend Tier (Star-LG Orchestrator)                          │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  L0 全体代理 (Top-Level Agent)                                       │    │
│  │  ┌───────────────────────────────────────────────────────────────┐  │    │
│  │  │  StateGraph (singleton per session)                            │  │    │
│  │  │  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐  │  │    │
│  │  │  │ parse_intent    │  │ dispatch        │  │ collect      │  │  │    │
│  │  │  │ (LLM + intent   │→│ (sub-agent pool │→│ (asyncio     │  │  │    │
│  │  │  │  classifier)    │  │  spawn)         │  │  .gather)    │  │  │    │
│  │  │  └─────────────────┘  └─────────────────┘  └──────────────┘  │  │    │
│  │  │  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐  │  │    │
│  │  │  │ tool_node       │  │ respond         │  │ interrupt    │  │  │    │
│  │  │  │ (direct MCP)    │  │ (LLM + agg.)    │  │ (top-level)  │  │  │    │
│  │  │  └─────────────────┘  └─────────────────┘  └──────────────┘  │  │    │
│  │  │  ★ State Schema: TopAgentState (TypedDict)                     │  │    │
│  │  └───────────────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  L1 Sub-Agent Pool (per task card, N 並行)                          │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │    │
│  │  │ SA-01       │ │ SA-02       │ │ SA-03       │ │ SA-04       │  │    │
│  │  │ code-review │ │ test-gen    │ │ 5-域-lead   │ │ git-ops     │  │    │
│  │  │ (subgraph)  │ │ (subgraph)  │ │ -audit      │ │ (subgraph)  │  │    │
│  │  │             │ │             │ │ (subgraph)  │ │             │  │    │
│  │  ├─────────────┤ ├─────────────┤ ├─────────────┤ ├─────────────┤  │    │
│  │  │ SA-05       │ │ SA-06       │ │ SA-07       │ │ SA-08       │  │    │
│  │  │ doc-sync    │ │ refactor    │ │ db-migration│ │ domain-dev  │  │    │
│  │  ├─────────────┤ ├─────────────┤ ├─────────────┤ ├─────────────┤  │    │
│  │  │ SA-09       │ │ SA-N        │ │             │ │             │  │    │
│  │  │ free-form   │ │ (新)        │ │             │ │             │  │    │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │    │
│  │                                                                     │    │
│  │  ★ 共通: init → plan → execute → verify → report 节点模板           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Cross-Cutting Components                                           │    │
│  │  • CheckpointStore (3-tier: Memory → SQLite → PostgreSQL)            │    │
│  │  • McpClient (star-mcp proxy, 16 tools)                              │    │
│  │  • TaskCardManager (UI state ↔ Sub-agent state mirror)               │    │
│  │  • AuditLogger (全 tool call / dispatch / interrupt)                 │    │
│  │  • TokenTelemetry (per-task / per-session token 计量)                │    │
│  │  • GuardEnforcer (AGENTS.md §4 守门 13 main + 24 派生规 = 37 项 自动检查)                 │    │
│  │  • UIStreamer (WebSocket / SSE / REST 3 通道)                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────────────────────────┘
    │ MCP stdio / Streamable HTTP
    ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  L2 Tool Tier                                                              │
│  • star-mcp 16 tools (per ADR-0032)                                         │
│  • 22 domain-* crates (DDD bounded context)                                │
│  • scripts/automation/ (8 基类 + dispatcher / console_server / ai_edit_mock)│
│  • 5 域 Lead 配置 (Mavis 临时代签 per 守门 #3)                              │
└────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 階層構造 (Hierarchy)

| 階層 | 名称 | 数量 | 生命周期 | 永続化 |
|---|---|---|---|---|
| **L0** | 全体代理 (Top-Level Agent) | 1 / session | session 期間 | checkpoint (Memory + SQLite) |
| **L1** | 子代理 (Sub-Agent) | N 並行 (≤ 50) | task card 期間 | checkpoint (per-task, 3-tier) |
| **L2** | ツール (Tool) | 16 MCP + N domain | process 期間 | N/A (stateless facade) |

**通信方向**：
- L0 → L1: dispatch (downstream)
- L1 → L0: progress / result / interrupt (upstream)
- L1 ↔ L1: **原則禁止** (per 守门 #13 横展開, 防止状态污染)
- L0/L1 → L2: tool call (downstream)
- L2 → L0/L1: tool result (upstream)
- L0/L1 → UI: stream (bidirectional WebSocket)

### 1.3 コンポーネント一覧 (Component List)

| ID | 名称 | 責務 | 階層 | 重要度 |
|---|---|---|---|---|
| **C-01** | TopAgent | 全体代理 LangGraph instance, 意图解析/dispatch/collect/respond | L0 | P0 |
| **C-02** | SubAgentPool | sub-agent spawn / pool / lifecycle 管理 | L0 | P0 |
| **C-03** | SubAgent (9 types) | SA-01..SA-09 各 subgraph instance | L1 | P0 |
| **C-04** | CheckpointStore | 3-tier 永続化 (Memory / SQLite / PostgreSQL) | Cross | P0 |
| **C-05** | McpClient | star-mcp 16 tools proxy | L2 | P0 |
| **C-06** | UIStreamer | WebSocket / SSE / REST 3 通道推送 | Cross | P0 |
| **C-07** | TaskCardManager | UI 状态 ↔ Sub-agent state mirror | L0/L1 | P0 |
| **C-08** | AuditLogger | 全 tool call / dispatch / interrupt 記録 | Cross | P0 |
| **C-09** | TokenTelemetry | token 計量 + OLU 集計 (per 守门 #4) | Cross | P1 |
| **C-10** | GuardEnforcer | AGENTS.md §4 守门 13 main + 24 派生规 = 37 项 自动检查 | Cross | P1 |
| **C-11** | StateSchemaRegistry | LangGraph state schema 中央管理 | Cross | P1 |
| **C-12** | InterruptManager | human-in-the-loop interrupt / resume | L0/L1 | P0 |
| **C-13** | SubAgentRegistry | sub-agent 类型注册表 (SA-01..SA-09 + new) | L0 | P1 |
| **C-14** | CrossDomainDispatcher | 跨 domain crate 调用协调 (per 守门 #3) | L2 | P2 |
| **C-15** | HealthCheck | /api/health endpoint, 状態監視 | Cross | P1 |
| **C-16** | TaskOperationsManager | TMO 集中管理: 7 节点 (M-N1..M-N7) + 7 协议 + DAG 校验; 唯一 cross-task actor | L0 | P0 (v0.2) |
| **C-17** | TaskRelationshipGraph | 任务卡 DAG (parent_task_id / merged_from / split_into / superseded_by 4 字段), cycle prevention | L0/L1 | P0 (v0.2) |
| **C-18** | BulkOperationQueue | bulk_action 队列 + asyncio.gather 协调, 部分失败回滚 | L0 | P0 (v0.2) |
| **C-19** | MetadataRegistry | task_metadata 表中央管理 (Master RLS 必携 per 守门 #13 c) | L0 | P1 (v0.2) |
| **C-20** | DAGValidator | cycle detection O(V+E) 校验, 检测到环 → reject + interrupt | L0 | P0 (v0.2) |
| **C-21** | ReassignManager | SA-XX 类型切换, checkpoint preserved (per §2.6 M-N6) | L0 | P1 (v0.2) |
| **C-22** | SummarizeCollector | 跨 N SubAgentState 状态聚合, LLM 表格化 | L0 | P1 (v0.2) |

## 2. 機能設計 (Function Design)

### 2.1 全体代理 (Top-Level Agent)

#### 2.1.1 状态

```python
class TopAgentState(TypedDict, total=False):
    """全体代理 state schema (LangGraph TypedDict)"""
    # user input
    user_input: str                                # 用户输入 (chat bar)
    intent: Optional[str]                          # LLM 解析后的意图 (tool_call | dispatch | clarify)
    
    # active / completed sub-agents (reducer add)
    active_subagents: Annotated[list[SubAgentRef], operator.add]
    completed_subagents: Annotated[list[SubAgentResult], operator.add]
    
    # conversation history
    conversation_history: Annotated[list[Message], operator.add]
    
    # global context (5 域 Lead 状态, token 集計, etc.)
    global_context: dict
    
    # last response
    last_response: Optional[str]
    
    # interrupt 状态
    interrupt_id: Optional[str]
    interrupt_response: Optional[dict]
```

#### 2.1.2 ノード

| Node ID | 名称 | 責務 | 入力 | 出力 |
|---|---|---|---|---|
| **T-N1** | parse_intent | LLM 意图分类 + 必要 sub-agent 抽出 | user_input | intent, subagent_plan |
| **T-N2** | dispatch | SubAgentPool.spawn, 任务卡生成 | subagent_plan | active_subagents (add) |
| **T-N3** | tool_node | MCP tool 直接呼出 (sub-agent 不要時) | tool_name, params | tool_result |
| **T-N4** | collect | sub-agent 結果待合 (asyncio.gather) | active_subagents | completed_subagents (add) |
| **T-N5** | respond | LLM 生成 user-facing 回答 | completed_subagents + global_context | last_response |
| **T-N6** | interrupt | 暂停等待 user 决策 | critical_decision_needed | interrupt_id |
| **T-N7** | guard_check | 守门 #4 / #9 / #12 / #13 检查 | current_state | violations (list) |

#### 2.1.3 エッジ (条件分岐)

```python
# Top-level graph edges
def route_after_parse_intent(state):
    if state["intent"] == "tool_call":
        return "tool_node"
    elif state["intent"] == "dispatch":
        return "dispatch"
    elif state["intent"] == "clarify":
        return "interrupt"  # ask user for clarification
    else:
        return "respond"

def route_after_dispatch(state):
    if all(s["status"] == "done" for s in state["active_subagents"]):
        return "collect"
    else:
        return "__end__"  # wait for sub-agents to complete via streaming

def route_after_collect(state):
    if state.get("interrupt_id"):
        return "interrupt"
    elif any(v["severity"] == "critical" for v in state.get("violations", [])):
        return "interrupt"
    else:
        return "respond"
```

### 2.2 子代理 (Sub-Agent)

#### 2.2.1 共通 state schema

```python
class SubAgentState(TypedDict, total=False):
    """子代理 state schema (LangGraph TypedDict)"""
    task_id: str                                    # 唯一 ID (UUID v7)
    task_type: str                                  # SA-01..SA-09
    context: dict                                   # task 別 context (input, params)
    intermediate_steps: Annotated[list[Step], operator.add]
    final_result: Optional[Any]
    status: str                                     # pending | running | waiting_input | done | failed
    checkpoint_id: Optional[str]
    error: Optional[str]
    started_at: datetime
    completed_at: Optional[datetime]
    token_usage: dict                               # input/output/total
```

#### 2.2.2 9 种 sub-agent 类型 (初版 v0.1)

| ID | 类型名 | 用途 | 主要工具 | 节点模板 |
|---|---|---|---|---|
| **SA-01** | code-review | 代码审查 (per PR/MR) | git diff, code search, comment | review-plan → review-execute → review-report |
| **SA-02** | test-gen | 测试生成 | code search, test run | test-plan → test-execute → test-verify |
| **SA-03** | 5-域-lead-audit | 5 域 Lead 配置审计 (per UC-08, 跨 22 domain crates + 5 域治理矩阵) | 22 domain crates read | audit-plan → audit-execute (跨域) → audit-report |
> ⚠️ **SA-03 特殊**: 唯一一个**跨域 + 治理矩阵**型 sub-agent。依赖 5 域 Lead 真人到位 (per 守门 #3 反転 Mavis 临时代签, 见 19:39 JST 授权); 真人到位后追溯签字。其余 8 个 SA 都是 task-bound 单域/单工具型。 |
| **SA-04** | git-ops | git 操作 (worktree/commit/push) | git worktree, star CLI | ops-plan → ops-execute → ops-verify |
| **SA-05** | doc-sync | 文档同步 (AGENTS.md / WBS / ADR) | file write, git add+commit | doc-plan → doc-execute → doc-verify |
| **SA-06** | refactor | 代码重构 | code search, edit, test | refactor-plan → refactor-execute → refactor-verify (cargo test) |
| **SA-07** | db-migration | DB migration (per 守门 #13 W/T/M) | 22 domain DB schema | db-plan → db-migrate → db-verify |
| **SA-08** | domain-dev | DDD bounded context 開発 (per 22 domain crates) | 22 domain crate APIs | dev-plan → dev-execute → dev-verify (cargo test) |
| **SA-09** | free-form | 默认 fallback, 自由形式 | 全部 16 tools | generic plan-execute-verify-report |

#### 2.2.3 共通 节点模板

```python
# Generic sub-agent node template
def make_subagent_graph(task_type: str) -> StateGraph:
    """各 sub-agent 实例, 共通 5 节点 + task_type 特定节点"""
    graph = StateGraph(SubAgentState)
    
    # 共通 5 节点
    graph.add_node("init", init_node)             # 状态初始化, parent context 注入
    graph.add_node("plan", plan_node)             # LLM 计划生成
    graph.add_node("execute", execute_node)       # task_type 特定 execute subgraph
    graph.add_node("verify", verify_node)         # 守门 #1 / #12 / cargo test 等
    graph.add_node("report", report_node)         # 最终结果生成 + 通知 Top
    
    # 边
    graph.add_edge(START, "init")
    graph.add_edge("init", "plan")
    graph.add_conditional_edges(
        "plan",
        route_after_plan,  # proceed | need_user_input | abort
        {"proceed": "execute", "need_user_input": "interrupt", "abort": "report"}
    )
    graph.add_edge("execute", "verify")
    graph.add_conditional_edges(
        "verify",
        route_after_verify,  # ok | retry | abort
        {"ok": "report", "retry": "execute", "abort": "report"}
    )
    graph.add_edge("report", END)
    
    # 条件: 守门 violation 拦截
    graph.add_node("guard_check", guard_check_node)
    graph.add_edge("execute", "guard_check")
    graph.add_conditional_edges(
        "guard_check",
        route_after_guard,  # ok | critical_violation
        {"ok": "verify", "critical_violation": "interrupt"}
    )
    
    return graph.compile(checkpointer=...)
```

### 2.3 通信プロトコル (Communication Protocol)

#### 2.3.1 メッセージ类型

| 方向 | 类型 | 説明 | フィールド |
|---|---|---|---|
| **L0 → L1** | dispatch | task 割当 | task_id, task_type, context, parent_task_id |
| **L0 → L1** | cancel | task 取消 | task_id, reason |
| **L0 → L1** | interrupt_response | human 决策 | task_id, decision (approve/modify/cancel), payload |
| **L1 → L0** | progress | 状态更新 | task_id, status, partial_output, node_id |
| **L1 → L0** | result | 完成 | task_id, status=done, final_result, token_usage |
| **L1 → L0** | interrupt_request | 决策请求 | task_id, decision_needed, options, default |
| **L1 → L0** | error | 失败 | task_id, status=failed, error_msg, stack_trace |
| **L1 ↔ L1** | (N/A) | 禁止 | — |
| **L0/L1 → UI** | stream | SSE 推送 | type (token/state/event), payload |
| **UI → L0** | user_input | chat bar | text, attachments |
| **UI → L0** | card_action | task card 操作 | task_id, action (pause/resume/cancel) |
| **UI → L0** | tmo_action | TMO 入口 (chat bar / 卡片菜单 / 多选工具栏) | operation (merge/split/reorder/bulk/summarize/reassign/metadata), target_task_ids, payload |
| **UI → L1** | (proxy 経由) | task card 详情操作 | task_id, action |
| **L0 → L1** | merge_request | TMO 合并通知 (per §2.6.2 M-N1) | target_task_ids, merge_strategy |
| **L0 → L1** | split_request | TMO 拆分通知 (per §2.6.2 M-N2) | target_task_id, split_strategy |
| **L0 → L1** | dep_set | TMO 依赖 DAG 边更新 (per §2.6.2 M-N3) | dep_set (DAG 边集合) |
| **L0 → L1** | bulk_action | TMO 批量操作 (per §2.6.2 M-N4) | target_task_ids, action |
| **L0 → L1** | reassign_request | TMO 类型 SA-XX 切换 (per §2.6.2 M-N6) | target_task_id, new_task_type, preserved_checkpoint_id |
| **L0 → L0** | metadata_update | TMO task_metadata 表更新 (per §2.6.2 M-N7, Master RLS 必携) | target_task_id, metadata |
| **L0 → UI** | summarize_result | TMO 跨任务汇总结果 (per §2.6.2 M-N5) | task_summaries (Table) |

#### 2.3.2 通信 実装

```python
# L0 → L1 dispatch (in-process, asyncio.Queue)
class SubAgentPool:
    def __init__(self):
        self._pools: dict[str, SubAgentHandle] = {}  # task_id -> handle
        self._dispatch_queue: asyncio.Queue = asyncio.Queue()
    
    async def spawn(self, task_type: str, context: dict) -> SubAgentHandle:
        task_id = uuid7()
        handle = SubAgentHandle(task_id=task_id, type=task_type, state=SubAgentState(...))
        self._pools[task_id] = handle
        await self._dispatch_queue.put(DispatchMessage(task_id=task_id, type=task_type, context=context))
        # notify UI
        await ui_streamer.push(TaskCardCreateMessage(task_id=task_id, type=task_type, status="pending"))
        return handle
    
    async def get_result(self, task_id: str) -> SubAgentResult:
        # wait for completion
        handle = self._pools[task_id]
        await handle.completion_event.wait()
        return handle.state["final_result"]

# L1 → L0 progress (in-process, callback)
class SubAgentHandle:
    def __init__(self, task_id: str, type: str, state: SubAgentState):
        self.task_id = task_id
        self.type = type
        self.state = state
        self.completion_event = asyncio.Event()
        self.on_progress: Optional[Callable] = None  # injected by Top
    
    async def emit_progress(self, partial_output: str, node_id: str):
        self.state["intermediate_steps"].append(Step(node_id=node_id, output=partial_output))
        if self.on_progress:
            await self.on_progress(task_id=self.task_id, status="running", partial=partial_output)
        # also push to UI
        await ui_streamer.push(TaskCardProgressMessage(task_id=self.task_id, node_id=node_id, output=partial_output))
```

### 2.4 状態管理 (State Management)

#### 2.4.1 永続化 3-tier

> **设计选择**: 本 view 包装 LangGraph native savers (`MemorySaver` / `SqliteSaver` / `PostgresSaver`) 通过自定义 `CheckpointStore` ABC (per [03 §1.1 M-08 / M-09](03-detailed-design.md)), 加 audit + 业务级 metadata + 守门 hook。下表 native 名称 + 实际包装 クラス名 并列。

```
┌─────────────────────────────────────────────────────────────┐
│  Tier 1: In-Memory (per session)                              │
│  • Native: LangGraph MemorySaver (from langgraph.checkpoint.memory) │
│  • Wrapper: MemoryCheckpointer (per 03 §1.1 M-08)             │
│  • Fastest, no persistence                                     │
│  • Used for: high-frequency reads within session              │
└────────────────────────┬────────────────────────────────────────┘
                         │ async flush
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Tier 2: SQLite (cross-session)                               │
│  • Native: LangGraph SqliteSaver (from langgraph.checkpoint.sqlite) │
│  • Wrapper: SqliteCheckpointer (per 03 §1.1 M-09) — v0.1 默认 │
│  • Single file: ~/.star/langgraph/checkpoints.db              │
│  • Used for: cross-session resume                             │
└────────────────────────┬────────────────────────────────────────┘
                         │ async batch
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Tier 3: PostgreSQL (production)                              │
│  • Native: LangGraph PostgresSaver (from langgraph.checkpoint.postgres) │
│  • Wrapper: PostgresCheckpointer (v0.2 计划)                  │
│  • Multi-session, multi-tenant                                │
│  • Used for: production scale (v0.2 计划)                    │
│  • Per 守门 #13 Master data RLS                               │
└─────────────────────────────────────────────────────────────┘
```

#### 2.4.2 Reducer 設計

| Channel | Reducer | 理由 |
|---|---|---|
| `active_subagents` | `operator.add` (append) | 多 sub-agent 並行, append-only |
| `completed_subagents` | `operator.add` (append) | 履歴保持, append-only |
| `conversation_history` | `operator.add` (append) | 跨 turn 累積 |
| `intermediate_steps` | `operator.add` (append) | 节点実行履歴 |
| `global_context` | custom merge (LWW per key) | 5 域 Lead 状态 / token 集計等, 最終書込優先 |
| `last_response` | replace (last-write-wins) | 最新响应覆盖 |
| `interrupt_id` | replace | 单一时点 1 个 interrupt |
| `task_relationships` | custom merge (DAG 边 union) | TMO M-N3 dep_set, cycle prevention 必携 (per 守门 #13 a) |
| `superseded_tasks` | `operator.add` (append) | TMO M-N1/M-N2/M-N6 supersede 記録, 血缘可追溯 (per NFR-TMO-04) |
| `bulk_operations` | queue (FIFO) | TMO M-N4 批量操作队列, 部分失败回滚 |
| `last_summarize_result` | replace (last-write-wins) | TMO M-N5 上次汇总结果 (UI 显示) |
| `active_tmo_operation` | replace | 单一时点 1 个 TMO 操作 ID (merge/split/reorder/bulk/summarize/reassign/metadata) |

#### 2.4.3 State versioning

- 每个 checkpoint 附带 schema version (e.g., `state_schema_v1`)
- Schema migration: load → upgrade → save
- Git tag: `state-schema-v1.0.0`, `state-schema-v1.1.0` 等

### 2.5 ツール統合 (Tool Integration)

#### 2.5.1 全体代理 direct tool call

```python
# Top Agent tool_node, direct MCP 16 tools
from langgraph.prebuilt import ToolNode
from langchain_mcp_adapters.client import MultiServerMCPClient

mcp_client = MultiServerMCPClient({
    "star-mcp": {
        "command": "star-mcp",
        "args": ["--transport", "stdio"],
        "transport": "stdio",
    }
})
top_tools = await mcp_client.get_tools()  # 16 tools

top_tool_node = ToolNode(top_tools, handle_tool_errors=True)
```

#### 2.5.2 子代理 proxy tool call

```python
# Sub-agent 内部 ToolNode, proxy through McpClient (audit + guard)
class AuditedMcpToolNode(ToolNode):
    """MCP tool call with audit + guard enforcement"""
    async def _arun_tool(self, tool_call: ToolCall) -> ToolMessage:
        # 1. guard check (per AGENTS.md §4)
        guard_result = await guard_enforcer.check_tool_call(tool_call)
        if guard_result.violation_severity == "critical":
            raise GuardViolation(guard_result)
        
        # 2. execute tool
        result = await super()._arun_tool(tool_call)
        
        # 3. audit log
        await audit_logger.log(AuditEntry(
            actor=self.task_id,
            action="tool_call",
            params=tool_call,
            result=result,
            token_usage=...
        ))
        
        # 4. telemetry
        await token_telemetry.record(tool_call, result)
        
        return result
```

#### 2.5.3 Tool call 監査 + 守门 統合

| Layer | 检查 | 守门引用 |
|---|---|---|
| L1 入口 | 守门 #5 (env var 安全) | 禁 env value 打印 |
| L1 中间 | 守门 #9 (子代理 status ≠ 实际成功) | audit 实证 |
| L1 中间 | 守门 #12 (AI 協作文档治理) | doc 操作 git 实证 |
| L1 中间 | 守门 #13 (DB 三類 W/T/M) | DB 操作分類检查 |
| L2 出口 | 守门 #6 (PowerShell only) | 禁 bash 命令 |
| L2 出口 | 守门 #7 (0 unsafe) | 禁 unsafe_code |
| L2 出口 | 守门 #1 (R-05 push 反転済) | push 需 final-action 确认 |

### 2.6 Task Management Operations (TMO, v0.2 新增) — L0 跨任务卡管理操作

> **核心约束 (per 守门 #13 a)**: L1 ↔ L1 禁止通信, 防止状态污染. 因此所有"跨任务卡"操作 (合并/拆分/依赖/批量/重分配/汇总/元数据) **必须走 L0 协调** (per 01 §UC-09..UC-13). TMO 是 L0 StateGraph 的 7 节点扩展, 7 协议扩展, state schema 扩展.

#### 2.6.1 TMO 7 节点 (M-N1..M-N7)

| Node ID | 名称 | 責務 | 入力 | 出力 |
|---|---|---|---|---|
| **M-N1** | `merge_node` | 合并 a+b → merged_task, stash_state + supersede + dispatch | target_task_ids=[a,b], merge_strategy | merged_task_id, stash_checkpoint_ids |
| **M-N2** | `split_node` | 拆分 a → a1 + a2, checkpoint snapshot + forked context dispatch | target_task_id=a, split_strategy | new_task_ids=[a1,a2], snapshot_checkpoint_id |
| **M-N3** | `reorder_node` | 依赖 DAG 调整, cycle detection (per DAGValidator C-20) | dep_set={a→b} | updated_task_relationships |
| **M-N4** | `bulk_node` | N 张卡批量 action (pause/resume/cancel/set_priority), asyncio.gather + 部分失败回滚 | target_task_ids=[a..n], action | batch_summary (success_count, failed_count) |
| **M-N5** | `summarize_node` | 跨 N SubAgentState 聚合 + LLM 表格化 | target_task_ids=[a..n] | last_summarize_result (TaskSummary[]) |
| **M-N6** | `reassign_node` | sub-agent 类型 SA-XX 切换, checkpoint preserved (ReassignManager C-21) | target_task_id, new_task_type | new_task_id, preserved_checkpoint_id |
| **M-N7** | `metadata_node` | task_metadata 表更新 (Master RLS 必携 per 守门 #13 c) | target_task_id, metadata_update | updated_metadata |

#### 2.6.2 TMO 7 协议 (扩展 02 §2.3.1 通信 メッセージ类型)

| 方向 | 类型 | 説明 | 触发节点 | 字段 |
|---|---|---|---|---|
| **L0 → L1** | `merge_request` | 通知 a/b 进入 stash_state (Transaction append-only) | M-N1 | target_task_ids, merge_strategy |
| **L0 → L1** | `split_request` | snapshot a 当前 checkpoint | M-N2 | target_task_id, split_strategy |
| **L0 → L1** | `dep_set` | DAG 边更新, C-20 校验 | M-N3 | dep_set (DAG 边集合) |
| **L0 → L1** | `bulk_action` | N 张卡并行 action | M-N4 | target_task_ids, action |
| **L0 → L1** | `reassign_request` | 类型 SA-XX 切换 | M-N6 | target_task_id, new_task_type, preserved_checkpoint_id |
| **L0 → L0** | `metadata_update` | TaskCardManager (C-07) 元数据更新 | M-N7 | target_task_id, metadata (name/labels/notes/priority) |
| **L0 → UI** | `summarize_result` | 跨任务汇总结果 | M-N5 | task_summaries (Table) |

#### 2.6.3 TMO 路由 (扩展 §2.1.3 エッジ)

```python
# TMO 入口: parse_intent_node 输出 intent 走 TMO
def route_after_parse_intent_tmo(state):
    intent = state.get("intent")
    if intent == "task_merge":         return "merge_node"        # M-N1
    elif intent == "task_split":       return "split_node"        # M-N2
    elif intent == "set_dependencies": return "reorder_node"      # M-N3
    elif intent == "bulk_action":      return "bulk_node"         # M-N4
    elif intent == "summarize":        return "summarize_node"    # M-N5
    elif intent == "reassign":         return "reassign_node"     # M-N6
    elif intent == "metadata":         return "metadata_node"     # M-N7
    else:                              return "respond"           # default
```

#### 2.6.4 TMO State Schema 扩展 (TopAgentState + SubAgentState)

**TopAgentState 扩展**:

```python
class TopAgentState(TypedDict, total=False):
    # ... 既有 v0.1 字段 (per §2.1.1) ...
    
    # TMO 新增 (v0.2)
    intent: Optional[str]  # 扩展: 含 task_merge / task_split / set_dependencies / bulk_action / summarize / reassign / metadata
    task_relationships: dict[str, list[str]]     # DAG, key=task_id, value=successors
    superseded_tasks: Annotated[list[str], operator.add]  # 被取代的 task_id 列表, append-only
    bulk_operations: queue[BulkAction]            # 批量操作队列
    last_summarize_result: Optional[list[TaskSummary]]  # 上次汇总结果
    active_tmo_operation: Optional[str]           # 当前 TMO 操作 ID (merge/split/reorder/bulk/reassign/metadata)
```

**SubAgentState 扩展 (per §2.2.1 既有 schema)**:

```python
class SubAgentState(TypedDict, total=False):
    # ... 既有 v0.1 字段 ...
    
    # TMO 血缘字段 (v0.2 新增, 100% 填 per NFR-TMO-04)
    parent_task_id: Optional[str]                 # 拆分/合并的来源
    merged_from: Annotated[list[str], operator.add]  # 合并来源 (append)
    split_into: Annotated[list[str], operator.add]   # 拆分结果 (append)
    superseded_by: Optional[str]                  # 被取代的目标 task_id
    checkpoint_snapshot: Optional[bytes]          # 拆分/重分配前的快照
```

#### 2.6.5 TMO 守门合规 (per AGENTS.md §4)

| 守门 | TMO 派生约束 | 实证位置 |
|---|---|---|
| **#13 a (L1↔L1 禁止)** | 7 节点全部 L0 协调, 跨任务操作只经 L0 (TaskOperationsManager C-16) | §2.6.1 节点设计 |
| **#13 c (Master RLS)** | task_metadata 表 100% RLS 必携 (per 守门 d) | §2.6.4 M-N7 + 03 §7 schema |
| **#13 d (Master 100% RLS / Transaction 100% audit / Work 100% retention)** | task card 状态 = Work (短 TTL, supersede 后 retention), checkpoint history = Transaction (append-only, audit 必携), metadata = Master (SCD Type 2) | §2.6.4 SubAgentState.blood 字段全部 append-only |
| **#4 (token-OLU)** | TMO 是 L0 决策, 不重 L1 token; TokenTelemetry (C-09) 计量每个 TMO 操作 token | §2.6.6 telemetry |
| **#19 (Python 化)** | TMO 实装走 `scripts/automation/task_ops.py` (per 守门 #19 派生, 后续 phase 起), 不写 .rs | [PHASE-LANGGRAPH-TMO-IMPL-REPORT](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) |
| **#9 v3 (subprocess 走 console_server)** | TMO UI 操作走 Next.js API route → FastAPI 8080 console_server.py → subprocess 调 task_ops.py | [PHASE-LANGGRAPH-TMO-IMPL-REPORT](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) §5 守门 |
| **#12 (AI 協作文档治理)** | TMO 文档修订历史 + ADR-0046 落档, 禁回溯叙事, BAS 引用 git 实证 | 本节 + ADR-0046 |

#### 2.6.6 TMO Telemetry + Metrics

TMO 操作产生以下 metrics (per 02 §8.1 Prometheus):

- `tmo_operation_total{operation, status}` — M-N1..M-N7 操作总数 (operation=merge/split/reorder/bulk/summarize/reassign/metadata)
- `tmo_operation_duration_seconds{operation}` — 操作延迟
- `tmo_task_relationship_edges` — DAG 边数
- `tmo_superseded_task_count` — 被取代 task 数
- `tmo_bulk_action_partial_failure_total` — 批量部分失败数 (per NFR-TMO-03)

#### 2.6.7 TMO 状态机扩展 (per 03 §5.1)

Task Card 状态机扩展:
- 新增状态: `superseded` (被取代, 历史保留, 不执行)
- 新增转换:
  - `running` → `superseded` (M-N1 merge 完成 / M-N2 split 完成 / M-N6 reassign 完成)
  - `pending` → `superseded` (M-N1 merge 在 a/b 还没启动时)
  - `waiting_input` → `superseded` (M-N1 merge 时 a/b 在等待用户决策)
  - `superseded` 终态, 不再转换 (per 守门 #13 d Transaction append-only)

详细状态图见 [03-detailed-design.md §5.1](03-detailed-design.md).

## 3. データモデル (Data Model)

### 3.1 概念データモデル

```
┌─────────────────┐ 1     N ┌─────────────────┐
│  TopAgent       │────────│  SubAgent       │
│  (1 / session)  │        │  (per task)     │
└─────────────────┘        └─────────────────┘
                                  │ 1
                                  │
                                  ▼ N
                            ┌─────────────────┐
                            │  Checkpoint     │
                            │  (per node)     │
                            └─────────────────┘
                                  
┌─────────────────┐ 1     1 ┌─────────────────┐
│  TaskCard       │────────│  SubAgent       │ (1:1 mirror)
│  (UI)           │        │                 │
└─────────────────┘        └─────────────────┘
                                  
┌─────────────────┐ 1     N ┌─────────────────┐
│  TopAgent       │────────│  AuditEntry     │
│                 │        │  (per action)   │
└─────────────────┘        └─────────────────┘
                                  
┌─────────────────┐ N     N ┌─────────────────┐
│  SubAgent       │────────│  ToolCall       │
│                 │        │  (16 MCP tools) │
└─────────────────┘        └─────────────────┘
```

### 3.2 LangGraph State Schema (TypeScript 拟似)

```typescript
// top_agent/state.ts
export interface TopAgentState {
  user_input: string;
  intent?: 'tool_call' | 'dispatch' | 'clarify'
        | 'task_merge' | 'task_split' | 'set_dependencies'  // v0.2 TMO
        | 'bulk_action' | 'summarize' | 'reassign' | 'metadata';
  subagent_plan?: SubAgentPlan[];
  active_subagents: SubAgentRef[];        // reducer add
  completed_subagents: SubAgentResult[];  // reducer add
  conversation_history: Message[];        // reducer add
  global_context: Record<string, any>;    // custom LWW
  last_response?: string;
  interrupt_id?: string;
  interrupt_response?: Record<string, any>;

  // v0.2 TMO 新增 (per 02 §2.6.4)
  task_relationships: Record<string, string[]>;  // DAG, key=task_id, value=successors
  superseded_tasks: string[];                    // reducer add (append-only)
  bulk_operations: BulkAction[];                 // FIFO queue
  last_summarize_result?: TaskSummary[];
  active_tmo_operation?: TMOOperation;           // 当前 TMO 操作
}

// sub_agent/state.ts
export interface SubAgentState {
  task_id: string;
  task_type: SA_01 | SA_02 | ... | SA_09;
  context: Record<string, any>;
  intermediate_steps: Step[];              // reducer add
  final_result?: any;
  status: 'pending' | 'running' | 'waiting_input' | 'done' | 'failed';
  checkpoint_id?: string;
  error?: string;
  started_at: string;                      // ISO 8601
  completed_at?: string;
  token_usage: {
    input: number;
    output: number;
    total: number;
  };
  guard_violations: GuardViolation[];      // reducer add

  // v0.2 TMO 血缘字段 (per 02 §2.6.4, 100% 填 per NFR-TMO-04)
  parent_task_id?: string;                 // 拆分/合并的来源
  merged_from: string[];                   // reducer add (append)
  split_into: string[];                    // reducer add (append)
  superseded_by?: string;                  // 被取代的目标 task_id
  checkpoint_snapshot?: string;            // 拆分/重分配前的快照 ID (per 守门 #13 d Transaction append-only)
}

export interface SubAgentRef {
  task_id: string;
  task_type: string;
  started_at: string;
  status: string;
}

export interface SubAgentResult {
  task_id: string;
  task_type: string;
  status: 'done' | 'failed';
  final_result: any;
  token_usage: { input: number; output: number; total: number };
  duration_ms: number;
}

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  task_id?: string;  // 关联 sub-agent (assistant 消息)
}

export interface Step {
  node_id: string;
  started_at: string;
  completed_at?: string;
  output: any;
  token_usage?: { input: number; output: number; total: number };
}

export interface GuardViolation {
  guard_id: string;          // e.g., "守门#5"
  severity: 'info' | 'warn' | 'critical';
  message: string;
  context: Record<string, any>;
  detected_at: string;
}
```

### 3.3 永続化 形式

#### 3.3.1 Checkpoint format (LangGraph native)

LangGraph 标准 pickle + json hybrid:
- thread_id: 唯一 session ID
- checkpoint_ns: namespace (top / sub-{task_id})
- checkpoint: state snapshot
- metadata: { task_type, session_id, user_id, schema_version }

#### 3.3.2 Storage schema (SQLite / PostgreSQL)

```sql
-- per LangGraph SqliteSaver / PostgresSaver native schema
CREATE TABLE checkpoints (
    thread_id TEXT,
    checkpoint_ns TEXT,
    checkpoint_id TEXT,
    parent_checkpoint_id TEXT,
    type TEXT,
    checkpoint JSONB,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (thread_id, checkpoint_ns, checkpoint_id)
);

CREATE TABLE checkpoint_blobs (
    thread_id TEXT,
    checkpoint_ns TEXT,
    channel TEXT,
    version TEXT,
    type TEXT,
    blob BYTEA,
    PRIMARY KEY (thread_id, checkpoint_ns, channel, version)
);

-- 拡張: 我们的 metadata
CREATE TABLE task_metadata (
    task_id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    user_id TEXT,
    session_id TEXT,
    status TEXT,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    token_usage JSONB,
    guard_violations JSONB
);

CREATE INDEX idx_task_metadata_session ON task_metadata(session_id);
CREATE INDEX idx_task_metadata_status ON task_metadata(status);
```

#### 3.3.3 RLS (Row-Level Security, per 守门 #13)

```sql
-- Master data RLS (per 守门 #13 d)
ALTER TABLE task_metadata ENABLE ROW LEVEL SECURITY;
CREATE POLICY task_metadata_isolation ON task_metadata
    USING (user_id = current_setting('app.current_user_id')::TEXT);
```

## 4. UI/UX 設計 (UI/UX Design)

### 4.1 画面レイアウト (Screen Layout)

```
┌──────────────────────────────────────────────────────────────────┐
│  AppHeader (h-12, 48px)                                            │
│  [Star logo] [5 tab nav]                [Theme] [User Avatar]      │
├──────┬───────────────────────────────────────────────────────────┤
│      │                                                            │
│      │                                                            │
│  S   │                                                            │
│  i   │           Main Content Area (Tab content)                   │
│  d   │                                                            │
│  e   │  ┌─────────────────────────────────────────────┐         │
│  b   │  │  Tab 4 Agents (★ NEW, 主要 tab)              │         │
│  a   │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐       │         │
│  r   │  │  │ Card 1  │ │ Card 2  │ │ Card 3  │       │         │
│      │  │  │ SA-01   │ │ SA-03   │ │ SA-04   │       │         │
│  w-  │  │  │ running │ │ done ✓  │ │ failed  │       │         │
│  56  │  │  │ ...     │ │ ...     │ │ ...     │       │         │
│      │  │  └─────────┘ └─────────┘ └─────────┘       │         │
│      │  └─────────────────────────────────────────────┘         │
│ 224 │                                                            │
│ px  │                                                            │
│      │                                                            │
│      │                                                            │
│      │                                                            │
├──────┴───────────────────────────────────────────────────────────┤
│  ★ NEW ★ Chat Bar (固定底行, h-14, 56px)                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  [📎] [input: "H2 8 domain 改造並列で"        ]  [Send↑]│    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

### 4.2 インタラクションフロー (Interaction Flow)

#### 4.2.1 User input → Top → Sub → UI

```
[User types in chat bar]
    │ (on Enter or Send click)
    ▼
[UI: POST /api/top-agent/dispatch]
    │ { user_input: "H2 8 domain 改造並列で" }
    ▼
[Backend: Top Agent parse_intent_node]
    │ intent = "dispatch"
    │ subagent_plan = [SA-08 domain-dev x 8]
    ▼
[Backend: Top dispatch_node]
    │ spawn 8 sub-agents
    │ ui_streamer.push(TaskCardCreate x 8)
    ▼
[Frontend: WebSocket receive]
    │ for each TaskCardCreate:
    │   append card to Agents tab grid
    │   card.animated_fade_in
    ▼
[Backend: sub-agents run]
    │ emit progress events
    │ ui_streamer.push(TaskCardProgress x N)
    ▼
[Frontend: WebSocket receive]
    │ for each TaskCardProgress:
    │   card.append_streaming_output
    │   card.status = "running"
    ▼
[Backend: sub-agent done]
    │ ui_streamer.push(TaskCardDone)
    ▼
[Frontend: WebSocket receive]
    │ card.status = "done ✓"
    │ card.append_final_result
    ▼
[Backend: Top collect_node, all done]
    │ aggregate results
    │ ui_streamer.push(TopResponse)
    ▼
[Frontend: WebSocket receive]
    │ chat_bar.append_assistant_message
    │ scroll to bottom
```

#### 4.2.2 Human-in-the-loop

```
[Sub-Agent: encounter critical decision]
    │ e.g., 守门 violation detected
    ▼
[Sub-Agent: interrupt_node]
    │ state.interrupt_id = uuid
    │ state.status = "waiting_input"
    ▼
[Backend: ui_streamer.push(InterruptPrompt)]
    │ { task_id, decision_needed, options, default }
    ▼
[Frontend: WebSocket receive]
    │ card.overlay_decision_prompt
    │ card.status = "waiting_input" (黄色高亮)
    │ chat_bar.badge += 1
    ▼
[User: clicks "Approve" / "Modify" / "Cancel"]
    │ UI: POST /api/top-agent/interrupt-response
    │     { task_id, decision: "approve" }
    ▼
[Backend: Top → Sub interrupt_response]
    │ state.interrupt_response = { decision: "approve" }
    │ sub-agent resume
    ▼
[Frontend: WebSocket receive]
    │ card.status = "running"
    │ card.overlay_decision_prompt.remove
    ▼
[Sub-Agent: continue execution]
```

### 4.3 タスクカード (Task Card) 詳細

| 要素 | 説明 |
|---|---|
| **Header** | agent type (SA-01..SA-09) + task_id (短码) + status badge |
| **Body** | latest streaming output (markdown render, syntax highlight) |
| **Footer** | started_at + duration + token_usage + controls (Pause/Resume/Cancel) |
| **Click → Modal** | 全 state dump, history, checkpoint info, decision log |
| **Drag** | 5 tab 间 drag (e.g., 从 Agents → Kanban) |

## 5. インターフェース設計 (Interface Design)

### 5.1 内部 API (sub-agent ↔ top-agent, in-process)

```python
# Top → Sub dispatch
async def dispatch_subagent(task_type: str, context: dict) -> SubAgentHandle:
    """SubAgentPool.spawn()"""
    pass

# Top ← Sub progress
async def on_subagent_progress(task_id: str, status: str, partial: str, node_id: str):
    """callback injected by Top, pushes to UI"""
    pass

# Top ← Sub result
async def on_subagent_result(task_id: str, result: SubAgentResult):
    """final result, appends to completed_subagents"""
    pass

# Top → Sub interrupt_response
async def send_interrupt_response(task_id: str, decision: str, payload: dict = None):
    """resume sub-agent"""
    pass

# Top → Sub cancel
async def cancel_subagent(task_id: str, reason: str):
    """soft cancel, allow checkpoint save"""
    pass
```

### 5.2 外部 API (UI ↔ backend, HTTP/WS)

| Endpoint | Method | 用途 | 応答 |
|---|---|---|---|
| `/api/top-agent/dispatch` | POST | UI → Top, user input | { session_id, intent, initial_state } |
| `/api/top-agent/stream` | WS | Top → UI, streaming | { type, payload } events |
| `/api/top-agent/state` | GET | UI → Top, state poll | TopAgentState snapshot |
| `/api/top-agent/cancel` | POST | UI → Top, cancel all | { cancelled_count } |
| `/api/sub-agent/{task_id}/stream` | WS | Sub → UI, streaming | { type, payload } events |
| `/api/sub-agent/{task_id}/state` | GET | UI → Sub, state | SubAgentState snapshot |
| `/api/sub-agent/{task_id}/interact` | POST | UI → Sub, interrupt_response | { resumed: true/false } |
| `/api/sub-agent/{task_id}/cancel` | POST | UI → Sub, cancel | { cancelled: true/false } |
| `/api/tasks` | GET | UI → backend, list all | { tasks: SubAgentRef[] } |
| `/api/tasks/{task_id}` | GET | UI → backend, task detail | SubAgentState full |
| `/api/health` | GET | UI / monitoring → backend | { status, uptime, ... } |
| `/api/metrics` | GET | monitoring → backend | Prometheus-format metrics |
| `/api/tmo/merge` | POST | UI → Top, TMO M-N1 合并 a+b | { merged_task_id, superseded_task_ids, stash_checkpoint_ids } |
| `/api/tmo/split` | POST | UI → Top, TMO M-N2 拆分 a→a1+a2 | { new_task_ids, snapshot_checkpoint_id, superseded_task_id } |
| `/api/tmo/dependencies` | POST | UI → Top, TMO M-N3 dep_set | { dep_set, cycle_detected, updated_relationships } |
| `/api/tmo/bulk` | POST | UI → Top, TMO M-N4 批量 action | { batch_summary (success_count, failed_count) } |
| `/api/tmo/summarize` | POST | UI → Top, TMO M-N5 跨任务汇总 | { task_summaries (Table) } |
| `/api/tmo/reassign` | POST | UI → Top, TMO M-N6 类型 SA-XX 切换 | { new_task_id, preserved_checkpoint_id, superseded_task_id } |
| `/api/tmo/metadata` | POST | UI → Top, TMO M-N7 task_metadata 更新 | { updated_metadata } |
| `/api/tmo/relationships` | GET | UI → Top, 查询 DAG 边 | { relationships (DAG adjacency list) } |

### 5.3 MCP 統合 (per ADR-0032 stdio + Streamable HTTP)

```python
# 既存 star-mcp 16 tools (per AGENTS.md §7 #2 部分完成)
mcp_tools = [
    "star_task_current",        # 現状 task 取得
    "star_context_get",         # context 取得
    "star_code_search",         # code 検索
    "star_code_symbol",         # symbol 検索
    "star_workspace_list",      # workspace 一覧
    "star_worktree_create",     # worktree 作成
    "star_worktree_status",     # worktree 状態
    "star_mr_create",           # MR 作成
    "star_mr_show",             # MR 表示
    "star_test_affected",       # affected test 実行
    "star_submit",              # Universal Submit
    "star_diff",                # diff (P1-H 新增)
    "star_policy_check",        # policy 確認 (P1-H 新增)
    "star_commit",              # commit (P1-H 新增)
    "star_push",                # push (P1-H 新增)
    "star_mr_link",             # MR ↔ Issue 連動 (P1-H 新增)
]

# Top Agent: direct access via ToolNode
# Sub-Agent: proxy access via AuditedMcpToolNode (per §2.5.2)
```

## 6. セキュリティ設計 (Security Design)

per 要件 §3.3 NFR-S-01..06 全部継承 + 実装詳細：

### 6.1 認証 / 認可

- **Top Agent**: session-bound, 1 instance / session, 永続 state 包含 session_id
- **Sub Agent**: task-bound, 1 instance / task_id, 永続 state 包含 user_id + session_id
- **Tool call**: 5 域 Lead 真人到位前, 全部 Mavis 临时代签 (per 守门 #3 反転, 19:39 JST 授权)
- **跨 session resume**: user_id 認証必須 (per ADR-0030 Lease/Heartbeat/Resume 11 字段)

### 6.2 データ保護

- **In-memory**: LangGraph state, 永続化前不加密 (process 内)
- **SQLite**: 単一 file, ファイルパーミッション 600 (user only)
- **PostgreSQL**: TLS connection + RLS (per 守门 #13 d)
- **Audit log**: append-only, 日次 backup, 6 ヶ月 retention

### 6.3 守门 統合 (GuardEnforcer)

```python
class GuardEnforcer:
    """AGENTS.md §4 守门 13 main + 24 派生规 = 37 项 自动检查"""
    
    async def check_tool_call(self, tool_call: ToolCall) -> GuardResult:
        # 守门 #5: env var 安全
        if self._has_env_var_leak(tool_call):
            return GuardResult(violation_severity="critical", guard_id="守门#5")
        # 守门 #6: PowerShell only
        if self._has_bash_command(tool_call):
            return GuardResult(violation_severity="critical", guard_id="守门#6")
        # 守门 #13: DB 三類 W/T/M
        if self._is_db_op(tool_call):
            wtm = self._classify_wtm(tool_call)
            if wtm == "MIXED":
                return GuardResult(violation_severity="warn", guard_id="守门#13")
        # 守门 #1: R-05 push 反転済
        if self._is_push_command(tool_call):
            return GuardResult(violation_severity="info", guard_id="守门#1", requires_final_action=True)
        # ...
        return GuardResult(violation_severity="ok")
```

## 7. 性能設計 (Performance Design)

per 要件 §3.1 NFR-P-01..06 目標 + 実装戦略：

| NFR | 目標 | 実装戦略 |
|---|---|---|
| NFR-P-01 | first token ≤ 200ms p95 | LangGraph streaming start, 不等全 graph 完成 |
| NFR-P-02 | dispatch ≤ 500ms p95 | SubAgentPool.spawn 是 lightweight (state init + asyncio.create_task) |
| NFR-P-03 | 並行 ≥ 50 | asyncio + uvloop, single-thread event loop |
| NFR-P-04 | streaming ≤ 100ms p95 | SSE 100ms batch, WebSocket per-event |
| NFR-P-05 | checkpoint flush ≤ 1s | async flush, fsync 1s batch |
| NFR-P-06 | state query ≤ 50ms p95 | In-memory snapshot + JSON serialize |

### 7.1 キャッシュ戦略

| データ | TTL | キャッシュ層 |
|---|---|---|
| 5 域 Lead 設定 | 5 min | Redis (将来) / In-memory (v0.1) |
| MCP tool metadata | 5 min | In-memory LRU |
| Top state | session-bound | LangGraph MemorySaver |
| Sub state | task-bound | LangGraph SqliteSaver |

### 7.2 バックプレッシャー

- 並行 sub-agent ≥ 40 時, 警告 (UI 黄色提示)
- 並行 sub-agent ≥ 50 時, hard limit, dispatch queue 待機

## 8. 運用設計 (Operations Design)

### 8.1 監視

- **Health check**: `/api/health` (per NFR-A-01, 99.5% uptime)
- **Metrics**: Prometheus format `/api/metrics`
  - `top_agent_dispatch_total`
  - `sub_agent_active_count`
  - `sub_agent_duration_seconds`
  - `tool_call_total{tool_name, status}`
  - `token_usage_total{task_type, direction}`
  - `guard_violation_total{guard_id, severity}`
- **Logs**: structured JSON, 1 file/day, logrotate

### 8.2 ログ

```json
{
  "timestamp": "2026-09-03T17:00:00+09:00",
  "level": "INFO",
  "actor": "top_agent",
  "task_id": "...",
  "action": "dispatch",
  "details": {
    "task_type": "SA-03",
    "context_size": 1234,
    "user_id": "ulysses"
  },
  "token_usage": { "input": 100, "output": 50, "total": 150 }
}
```

### 8.3 バックアップ

- **SQLite**: 日次 copy → `~/.star/langgraph/backup/checkpoints-{date}.db`
- **PostgreSQL**: pg_dump 日次 (v0.2)
- **Audit log**: append-only, 6 ヶ月 retention, 過去 log 圧縮归档

## 9. 移行設計 (Migration Design)

### 9.1 既存 automation 脚本 段階的 移行

> **重要 (per [01 §1.0](01-requirements.md))**: 现有 `dispatcher.py` / `console_server.py` / `ai_edit_mock.py` 等是 **Mavis worker subagent 系统** (subprocess + brief), 跟本 view 设计的 **任务卡子代理 (Sub-Agent) 系统** (LangGraph in-process) 是两套独立系统。下表是 **共存** 計画, **不是** 取代 計画。

| 现有 (worker subagent 系统) | 目标 (任务卡子代理 系统) | 关系 |
|---|---|---|
| `scripts/automation/dispatcher.py` | `SubAgentPool.spawn()` (NEW, 任务卡子代理用) | 并存, 两套各管各的; 任务卡子代理 可在 execute 阶段 invoke dispatcher.py 派 worker subagent 干粗活 |
| `scripts/automation/console_server.py` | `UIStreamer` (NEW, FastAPI 8080 已有 + 加 /api/sub-agent/* 端点) | 共存, console_server.py 现有 FastAPI 端点保留, 加 任务卡子代理 路由 |
| `scripts/automation/ai_edit_mock.py` | SA-09 free-form 内部 tool | 関数 import (复用) |
| `scripts/automation/judge.py` | `[P]/[S]/[M]` 判定 CLI | 既存 continue, 任务卡子代理 dispatch 时机 判定 可复用 |
| `scripts/automation/refactor_template.py` | SA-06 refactor 内部分支 | 选择性 wrap |
| `scripts/automation/registry_check.py` | 脚本注册表 一致性校验 | CI gate, 既存 continue |
| 其他业务脚本 (h2_refactor / kanban_sprint_gen / ...) | 选择性 迁移为 SA-XX sub-agent 类型 | 不强制, per task 评估 |

### 9.2 既存 gm-console frontend 拡張

- 既存 5 tab 维持 (per AGENTS.md §7 #15 v0.15 拍板)
- Tab 4 "Agents" → 加入 sub-agent 状态 (per §4.1)
- Chat Bar 底行 追加 (per §4.1)
- 既存 Kanban / Timeline 等 tab, task card 状态 mirror (sub-agent ↔ card 連動)

### 9.3 既存 star-mcp 統合

- 16 tools そのまま利用, Top ToolNode + Sub AuditedMcpToolNode 経由
- 新規 tool 追加時, MCP server 拡張のみ (LangGraph 側変更不要)

## 10. 既知の制約 (Known Constraints) — 初版 v0.1

per 要件 §7 + 追加:

- 5 域 Lead 真人未到位 (per 守门 #3 反転: Mavis 临时代签)
- PostgreSQL checkpointer 未実装 (v0.2 计划)
- 跨仓 (Physis/RGS) RPC 未実装 (v0.3 计划)
- 並行 sub-agent 数上限 50 (NFR-P-03, リソース制約)
- 5 域 Lead 决策追跡 UI 未完成 (F-15 标 P2)
- token OLU telemetry 接入待 SRE Lead 真人
- Chat Bar 既存フロントエンド統合 UI 検証未実施 (デザイン段階, v0.1 は MVP 機能)
- Task Card Modal 詳細 view 未実装 (F-10 部分, v0.1 は一覧のみ)
- 既存 dispatcher.py / console_server.py との共存 過渡期 (per §9.1)
- LangGraph SDK バージョン固定 (lock to 0.2.x, 2026-09-03 時点)
- **TMO 7 节点 (M-N1..M-N7) 实装 P0**: v0.2 文档完成, 组件 C-16..C-22 schema 落档, 实装待 P0-1/H2 阻塞解除 (per [PHASE-LANGGRAPH-TMO-IMPL-REPORT](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) 7 子项 phase 计划, 走守门 #19 Python 化 + 守门 #9 v3 subprocess 路径)
- **守门 #13 a 强约束派生实证缺口**: L1↔L1 禁止通信 → TMO 全部 L0 协调; 实证待 TMO 实装阶段 补 (sub-session 续做)

## 11. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-03 | 🟡 Draft v0.1; 2-level hierarchical LangGraph 基本設計 (全体代理 + 任务卡子代理) 落档 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手终审通过 (per 2026-09-03 17:51 JST 用户发令); 15 component + 9 sub-agent 类型 + 3-tier checkpoint + 12 API endpoint + 守门 統合落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 1.2 | 架构师 / Mavis 接手审批 (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手终审通过 (per 2026-09-04 19:15 JST 用户发令); TMO 7 节点 (M-N1..M-N7) + 7 协议 + 7 组件 (C-16..C-22) + State Schema 扩展 + 8 外部 API 端点 (/api/tmo/*) + 7 Prometheus metrics 落档, 随 01-requirements.md + 03-detailed-design.md 同步升档 v0.2 + PHASE-LANGGRAPH-TMO-IMPL-REPORT 7 子项实装 phase 起 |
| 6 | SRE Lead (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 7 | 平台工程师 (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 8 | 评审主持人 (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 9 | 项目负责人 (PM, v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：15 component + 9 sub-agent 类型 + 3-tier checkpoint + 12 API endpoint + 守门 統合 + 性能/运用/移行設計 | 2026-09-03 17:51 JST 用户发令"另起一套架构view,专门设计langgraph相关的功能" (随 01-requirements.md 同步落档) |
| v0.2 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **TMO 升版**: 新增 §2.6 Task Management Operations 全节 (7 节点 M-N1..M-N7 + 7 协议 + 7 组件 C-16..C-22 + State Schema 扩展 + Reducer 5 新 channel + 5 route_after_parse_intent_tmo + 守门合规表 7 项 + Telemetry/Metrics 7 项 + 状态机扩展 superseded 终态); §1.3 组件表加 C-16..C-22; §2.3.1 通信协议加 7 类 (merge_request / split_request / dep_set / bulk_action / reassign_request / metadata_update / summarize_result) + tmo_action; §2.4.2 Reducer 加 5 新 channel (task_relationships / superseded_tasks / bulk_operations / last_summarize_result / active_tmo_operation); §3.2 State Schema (TS) 加 TopAgentState 5 新字段 + SubAgentState 5 血缘字段; §5.2 外部 API 加 8 端点 (/api/tmo/merge|split|dependencies|bulk|summarize|reassign|metadata|relationships); §10 加 2 新已知缺口 (TMO 实装 P0 / 守门 #13 a 实证); 5 签字栏 v0.2 升版; 守门 #1+#5+#6+#7+#9+#10+#12+#13+#19+#20+#22 跨 stage 全过 (文档工作无 .rs 改动, cargo check 不需要跑) | 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b" (per ask_d076c26d3fbf599eec1c32fd 拍板 (1) 范围=完整 7 节点全覆盖 (2) 文档策略=原地升版 v0.1 → v0.2 (3) 实装阶段=文档+commit 一并落), ~0.06M token 估 |

---

## 13. 引用文档

- [01-requirements.md](01-requirements.md) — 要件定義書 (本架构 view 起点)
- [03-detailed-design.md](03-detailed-design.md) — 詳細設計書
- [ADR-0030 Agent Lease/Heartbeat/Resume](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) — 11 字段 + 跨 Agent Handoff
- [ADR-0032 MCP Transport stdio](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) — 16 tools
- [ADR-0033 代签规则反转](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) — 本规则正式 ADR
- [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 13 main + 24 派生规 = 37 项硬约束
- [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) — token 基线
- [STAR-P3-WBS-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/reports/STAR-P3-WBS-001.md) — P3 阶段 WBS
- [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) — agent 交互 Python 化
- [scripts/automation/dispatcher.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/dispatcher.py)
- [scripts/automation/console_server.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/console_server.py)
- [scripts/automation/ai_edit_mock.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/ai_edit_mock.py)
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt
