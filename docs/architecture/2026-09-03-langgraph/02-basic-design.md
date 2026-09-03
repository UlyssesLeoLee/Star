# 02. Star LangGraph 統合アーキテクチャ - 基本設計書 (Basic Design)

> **状態**：🟡 Draft v0.1
> **日期**：2026-09-03
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权）
> **依赖**：[01-requirements.md](01-requirements.md)（要件定義書）· [ADR-0032 MCP Transport stdio](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0030 Agent Lease/Heartbeat/Resume](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md)
> **关联文档**：[01-requirements.md](01-requirements.md) · [03-detailed-design.md](03-detailed-design.md)（詳細設計書）

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
| **UI → L1** | (proxy 経由) | task card 详情操作 | task_id, action |

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
  intent?: 'tool_call' | 'dispatch' | 'clarify';
  subagent_plan?: SubAgentPlan[];
  active_subagents: SubAgentRef[];        // reducer add
  completed_subagents: SubAgentResult[];  // reducer add
  conversation_history: Message[];        // reducer add
  global_context: Record<string, any>;    // custom LWW
  last_response?: string;
  interrupt_id?: string;
  interrupt_response?: Record<string, any>;
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

## 11. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-03 | 🟡 Draft v0.1; 2-level hierarchical LangGraph 基本設計 (全体代理 + 任务卡子代理) 落档 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手终审通过 (per 2026-09-03 17:51 JST 用户发令); 15 component + 9 sub-agent 类型 + 3-tier checkpoint + 12 API endpoint + 守门 統合落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签; 5 域独立真实身份签字请 DDD Review 阶段补 |

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：15 component + 9 sub-agent 类型 + 3-tier checkpoint + 12 API endpoint + 守门 統合 + 性能/运用/移行設計 | 2026-09-03 17:51 JST 用户发令"另起一套架构view,专门设计langgraph相关的功能" (随 01-requirements.md 同步落档) |

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
