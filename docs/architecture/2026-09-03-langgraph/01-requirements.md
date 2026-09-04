# 01. Star LangGraph 統合アーキテクチャ - 要件定義書 (Requirements Definition)

> **状態**：🟢 Draft v0.2
> **日期**：2026-09-04 (升版自 v0.1)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户授权"允许你代签" + 21:59 JST 第三次强化"继续, 你可以代签"）
> **依赖**：[ADR-0033 代签规则反转](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [AGENTS.md §4 守门硬约束](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [STAR-OLU-001.md token 基线](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) · [STAR-P3-WBS-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/reports/STAR-P3-WBS-001.md)
> **关联文档**：[02-basic-design.md](02-basic-design.md)（基本設計書 v0.2）· [03-detailed-design.md](03-detailed-design.md)（詳細設計書 v0.2）· [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md)（7 子项实装计划）
> **適用範囲**：STAR 主仓 (`D:\Star`) 全体，gm-console frontend / star-mcp / 22 domain-* crates / scripts/automation/ 全栈

---

## 0. 目的 (Purpose)

本文档定义 Star 项目的 **LangGraph 統合アーキテクチャ** (Star-LG) — 一个 2-level hierarchical multi-agent 系统：

- **L0 全体代理 (Top-Level Agent / Total Agent)**：放置在 UI 最下行聊天栏背后，**整体控制 Star 全局各个细节**（per 2026-09-03 17:51 JST 用户决策原文）
- **L1 任务卡子代理 (Sub-Agent / Task Card Agent)**：每张任务卡 = 1 个 sub-agent 窗口概念，**各自有独特一套作为代理的 LangGraph 设计**

通过该架构实现：
1. **统一入口**：单聊天栏 → 全局控制（UI 简素化）
2. **任务可视化**：每 sub-agent = 1 张任务卡（透明度提升）
3. **跨 session 持续**：checkpointing + resume
4. **5 域 Lead 协同**：守门 violation 实时检测 + 决策追踪
5. **AI 协作硬约束落地**：AGENTS.md §4 守门规则自动入 LangGraph 节点

## 1. 業務要件 (Business Requirements)

### 1.0 用語区別 (Sub-Agent vs Worker subagent)

本架构 view 设计的是 **任务卡子代理 (Sub-Agent)** — UI 驱动、LangGraph 状态化的新 sub-agent 系统。**与现有 Mavis worker subagent (worker/explorer/verifier, 通过 `dispatcher.py` + brief 派发) 是两套独立的 sub-agent 系统**：

| 维度 | 任务卡子代理 (Sub-Agent, 本 view 范围) | Worker subagent (现有, 本 view 范围外) |
|---|---|---|
| **触发** | UI 聊天栏 / 任务卡 (用户驱动) | Mavis 自动派发 (Mavis root 驱动) |
| **生命周期** | task card 期间 (1 user session) | Mavis task 期间 (跨 session 短) |
| **状态管理** | LangGraph checkpoint (3-tier) | brief + status.json (filesystem) |
| **通信** | in-process asyncio (WebSocket/SSE 推流) | subprocess + RPC (per 守门 #20) |
| **守门 #20** | 不适用 (用户驱动) | 必先 brief (per 守门 #20 派生规) |
| **典型场景** | "H2 8 domain 改造並列で" / "5 域 Lead 配置 review" | "B.5 OpenClaw 真实接入 e2e" (Mavis 内部 task) |
| **典型 API** | `SubAgentPool.spawn(SA-XX, ctx)` | `SubagentDispatcher.brief(task_id, content, agent)` |

**关系 (per 02-basic-design.md §9.1 移行設計)**:
- 两套系统**并存**, 任务卡子代理不取代 worker subagent
- 任务卡子代理可以在 plan/execute 阶段**调用** worker subagent 走 subprocess 路径 (跨平台, 守门 #9 实证)
- 现有 `dispatcher.py` / `console_server.py` 维持, 加 LangGraph state 桥接 (wrapper adapter)

### 1.1 背景 (Background)

Star 项目现状（per 2026-09-03 main HEAD `e5f0503`）：

- **22 domain-* crate**（DDD bounded context）：identity / permission / work-item / workspace / worktree / feedback / validation / integration / comment / project / tenant / ...
- **16 MCP tools**（star-mcp stdio + Streamable HTTP transport）
- **gm-console frontend**（Next.js + AppShell 5-tab：Kanban / Timeline / Backlog / Agents / Worktrees）
- **5 域 Lead 治理**（per 2026-08-21 JST 拒绝兼任硬约束）：player / economy / match / social / admin
- **scripts/automation/**（4 真实基类 `__init__.py` / `dispatcher.py` / `cli_helper/base.py` / `refactor_template.py` + 4 utility `judge.py` / `smoke_test.py` / `registry_check.py` / `console_server.py` + N 业务脚本如 `ai_edit_mock.py` / `h2_refactor.py` / `kanban_sprint_gen.py` 等, per `scripts/automation/registry.md` v0.1）
- **AGENTS.md §4 守门 13 main (#1-#13) + 24 派生规 v1-v24 = 37 全部继承**（具体含 #1/#3/#4/#5/#6/#7/#8/#9/#10/#11/#12/#13 + 守门 #1 v1-v24 派生累积规）

### 1.2 業務課題 (Business Challenges)

| # | 課題 | 影響範囲 |
|---|---|---|
| **B-01** | 跨域/跨工具的统一入口缺失 — UI 上需多 tab 切换 + 多 CLI 工具 | 开发者每日 30+ 次跨域操作，认知负担 |
| **B-02** | 多任务并行状态散乱 — scripts/automation/ 8 份脚本无统一调度 | 5 域 Lead 决策追踪困难，token OLU 无法实时 |
| **B-03** | human-in-the-loop 缺乏统一抽象 — 16 tools 各自有 interrupt 语义 | 守门 #1/#9/#12 人工介入时机不一致 |
| **B-04** | 上下文跨 session 不可持续 — 当前 mavis session lifecycle 短 | 跨天/跨周任务上下文丢失 |
| **B-05** | 5 域 Lead 决策/守门 violation 不可观测 — 真人未到位时 Mavis 临时代签 | 决策矩阵无法回溯 |
| **B-06** | automation 脚本缺少可观测 UI — dispatcher.py / console_server.py 现有 FastAPI 8080 单独运行 | 用户无法直观看到 sub-agent 状态 |

### 1.3 期待効果 (Expected Effects)

| # | 効果 | 計測指標 |
|---|---|---|
| **E-01** | 单一聊天栏 → 全局控制 (UI 简素化) | 跨域操作平均点击数 ≤ 3 |
| **E-02** | 任务卡可视化 → sub-agent 透明性 | 100% sub-agent 状态 UI 可见 |
| **E-03** | checkpointing 跨 session 持续 | 跨 session resume 成功率 ≥ 95% |
| **E-04** | 5 域 Lead 决策 / token 消耗 / 守门 violation 实时可观测 | 决策 latency ≤ 1s，可观测延迟 ≤ 500ms |
| **E-05** | 守门规则自动入 LangGraph 节点 | 守门违反自动拦截率 ≥ 80% |
| **E-06** | 16 MCP tools 全部接入 sub-agent | tool 覆盖率 100% |

### 1.4 業務フロー (Business Flows)

#### 1.4.1 主シナリオ: 全体代理 → 子代理 dispatch

```
[User] ──input──> [Chat Bar] ──> [Top Agent: parse_intent]
                                       │
                                       ├── (simple query) ──> [Tool Node: 直接调用 MCP tool]
                                       │                              │
                                       │                              ▼
                                       │                       [Result] ──> [Top: respond]
                                       │                                          │
                                       │                                          ▼
                                       │                              [Chat Bar] <── [User]
                                       │
                                       └── (complex task) ──> [Sub-Agent Pool: dispatch]
                                                                   │
                                                                   ├── [Sub 1: code-review] ──> [Card 1]
                                                                   ├── [Sub 2: 5-域-lead-audit] ──> [Card 2]
                                                                   └── [Sub N] ──> [Card N]
                                                                                    │
                                                                                    ▼
                                                                       [All done] ──> [Top: aggregate]
                                                                                            │
                                                                                            ▼
                                                                              [Chat Bar] <── [User: 最终结果]
```

#### 1.4.2 副シナリオ: Human-in-the-loop

```
[Sub-Agent: plan_node] ──> [interrupt 节点] ──> [Card: 等待用户决策]
                                                       │
                                                       ▼
                                              [User: approve / modify / cancel]
                                                       │
                                                       ▼
[Sub-Agent: resume] ──> [execute_node] ──> [verify_node] ──> [report_node]
```

#### 1.4.3 異常シナリオ: 守门 violation 拦截

```
[Sub-Agent: execute_node] ──> [守门检测节点] ──> (violation detected)
                                                      │
                                                      ▼
                                            [Card: 红色警告 + 违规说明]
                                                      │
                                                      ▼
                                          [User: 决策 (撤销 / 修复 / 豁免)]
```

## 2. 機能要件 (Functional Requirements)

### 2.1 システム全体像 (System Overview)

2-level hierarchical LangGraph 架构：

```
┌──────────────────────────────────────────────────────────────┐
│              L0 全体代理 (Top-Level Agent)                    │
│  • UI 最下行聊天栏背後の LangGraph instance                    │
│  • 跨域/跨工具调度 (16 MCP tools + 22 domain crates)          │
│  • 1 instance / session (singleton)                         │
│  • 永続 state (cross-session)                                │
└─────────────┬────────────────────────────────────────────────┘
              │ dispatch
              ▼
┌──────────────────────────────────────────────────────────────┐
│              L1 任务卡子代理 (Sub-Agent Pool)                  │
│  • N instances 並行 (per task card)                          │
│  • 各 sub-agent 独立 LangGraph subgraph                       │
│  • 各 task card 独立 state + checkpoint                      │
│  • 隔離 context (cross-sub-agent 不可见)                     │
└─────────────┬────────────────────────────────────────────────┘
              │ tool call
              ▼
┌──────────────────────────────────────────────────────────────┐
│  L2 ツール層: 16 MCP tools + 22 domain crates + automation  │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 アクター (Actors)

| アクター | 種別 | 説明 |
|---|---|---|
| **Primary User (Ulysses)** | 人間 | 12 角色一人公司，Star 项目唯一决策者 |
| **Secondary User (Developer)** | 人間 | 5 域 Lead 真人 (per 守门 #3) |
| **Top-Level Agent** | システム | 全体代理 L0，singleton |
| **Sub-Agent (per task card)** | システム | 子代理 L1，N 並行 |
| **MCP Tool Wrapper** | システム | 16 tools 統一 façade |
| **Automation Script** | システム | scripts/automation/ 8 份基类 |
| **Checkpoint Store** | システム | Memory / SQLite / PostgreSQL 3-tier |
| **Audit Logger** | システム | 全 tool call / dispatch / interrupt 記録 |

### 2.3 ユースケース (Use Cases)

#### UC-01: 聊天栏输入 → 全体代理解析

- **Actor**: Primary User
- **Trigger**: 用户在 chat bar 输入 natural language query
- **Flow**:
  1. UI → WebSocket → Top Agent
  2. Top: parse_intent_node (LLM)
  3. Top: 决定 (direct tool / dispatch sub-agents / need clarification)
  4. Top: streaming feedback 给 UI
- **Postcondition**: 意图解析完成 + dispatch 决策落地

#### UC-02: 子代理 dispatch → 任务卡生成

- **Actor**: Top Agent
- **Trigger**: parse_intent 输出需要 sub-agent
- **Flow**:
  1. Top: dispatch_node 选 sub-agent 类型 (SA-01..SA-09)
  2. SubAgentPool.spawn(type, context)
  3. UI: 创建 task card, 初始状态 pending
  4. Sub-agent: 启动 LangGraph subgraph
  5. UI: streaming 节点输出到 card
- **Postcondition**: 1+ task card visible，sub-agent running

#### UC-03: 多任务并行 (multiple sub-agents)

- **Actor**: Primary User
- **Trigger**: 复合 query (e.g., "H2 8 domain 改造並列で")
- **Flow**:
  1. Top: 解析 → 8 sub-agent 计划
  2. Top: 并行 dispatch 8 sub-agent
  3. UI: 8 task card 并列显示
  4. Sub-agents: 独立运行
  5. Top: collect_node 等待所有完成 (asyncio.gather)
  6. Top: aggregate → 最终回答
- **Postcondition**: 8 sub-agents completed，aggregated result 反馈 user

#### UC-04: Human-in-the-loop (子代理暂停)

- **Actor**: Primary User + Sub-Agent
- **Trigger**: sub-agent 遇到 critical decision (e.g., 守门 violation, ambiguous context)
- **Flow**:
  1. Sub-agent: interrupt_node (LangGraph interrupt)
  2. UI: card 黄色高亮 + 决策 prompt
  3. User: approve / modify / cancel
  4. UI: interrupt_response → sub-agent resume
- **Postcondition**: 决策落地，sub-agent 续行

#### UC-05: 子代理 ↔ 全体代理状态同步

- **Actor**: Sub-Agent + Top Agent
- **Trigger**: sub-agent 状态变化 (running → done / failed)
- **Flow**:
  1. Sub-agent: emit state change event
  2. Top: 收到事件 → 更新 active_subagents / completed_subagents (reducer add)
  3. UI: streaming update card 状态
- **Postcondition**: 状态一致 (top state = sub states)

#### UC-06: 跨 session 恢复 (checkpoint)

- **Actor**: Primary User
- **Trigger**: 新 session 启动 + 检测到未完成 sub-agents
- **Flow**:
  1. Top: mount → CheckpointStore.list_pending()
  2. Top: 选 checkpoint (most recent or user-pick)
  3. Top: 加载 → state 恢复
  4. UI: 显示历史 task cards (状态恢复)
  5. User: 选继续某 sub-agent
- **Postcondition**: 跨 session state 一致

#### UC-07: MCP tool 16 个的子代理内调用

- **Actor**: Sub-Agent
- **Trigger**: sub-agent 需要调用 tool (e.g., git status, worktree list)
- **Flow**:
  1. Sub-agent: tool_node → McpClient.call(tool_name, params)
  2. McpClient: star-mcp stdio 转发
  3. star-mcp: 实际执行 + 返回结果
  4. McpClient: 包装结果 + audit log
  5. Sub-agent: 接收结果 → 后续节点
- **Postcondition**: tool call 成功 + audit 记录

#### UC-08: 5 域 Lead 决策辅助

- **Actor**: Primary User
- **Trigger**: 跨 5 域决策场景 (e.g., "5 域 Lead 配置状況を review")
- **Flow**:
  1. Top: dispatch 5-域-lead-audit sub-agent
  2. Sub-agent: 跨 22 domain crates 查 5 域相关 state
  3. Sub-agent: 比对守门 #3 (5 域独立) + DEC-008 12 角色
  4. Sub-agent: 生成 audit 报告
  5. Top: aggregate → user 反馈
- **Postcondition**: 5 域决策矩阵可视化

#### UC-09: 任务卡合并 (Task Merge, TMO M-N1)

- **Actor**: Primary User
- **Trigger**: chat bar 输入 "合并任务 a 和任务 b" (或 UI 卡片多选 → 合并按钮)
- **Flow**:
  1. Top: parse_intent_node 解析 → intent = "task_merge", target_task_ids = [a, b]
  2. Top: merge_node (per 02 §2.6) 协调: 通知 a/b 进入 stash_state (保存 checkpoint 到 Transaction 表)
  3. Top: dispatch merged_task (SA-10 task-orchestrator, type = "merge", context = {merged_from: [a, b], merged_state: snapshots})
  4. Top: 标记 a/b 状态 = "superseded", pointer → merged_task
  5. UI: a/b 卡片灰显 (badge: "已合并"), merged_task 新卡出现 (label: "a + b")
  6. SubAgent (SA-10): 用 a + b 的 stash_state 作为初始 context, 继续 plan/execute
- **Postcondition**: a + b checkpoint 完整保留 (Transaction append-only), merged_task 启动, 血缘可追溯
- **约束 (per 守门 #13)**: L1 ↔ L1 禁止通信, 全部走 L0 协调

#### UC-10: 任务卡拆分 (Task Split, TMO M-N2)

- **Actor**: Primary User
- **Trigger**: chat bar 输入 "把任务 a 拆成 a1 和 a2" (或卡片菜单 → 拆分)
- **Flow**:
  1. Top: parse_intent_node → intent = "task_split", target_task_id = a, split_strategy = "context_fork" (默认) | "checkpoint_fork"
  2. Top: split_node 协调: snapshot a 当前 checkpoint
  3. Top: dispatch a1 + a2 (相同 task_type as a, context = fork-of-a.context + split-specific delta)
  4. Top: 标记 a 状态 = "superseded", a1/a2.superseded_by = null (新起点), a.split_into = [a1, a2]
  5. UI: a 卡片灰显, a1 + a2 新卡并排出现 (label: "a → a1" / "a → a2")
- **Postcondition**: a 历史完整保留, a1 + a2 并行启动, 拆分可逆 (可由子代理产物重新 dispatch "re-merge" if needed)

#### UC-11: 任务卡依赖编排 (Dependency / DAG, TMO M-N3)

- **Actor**: Primary User
- **Trigger**: chat bar 输入 "任务 b 完成后才能启动 c" (或 Kanban 拖拽连线)
- **Flow**:
  1. Top: parse_intent_node → intent = "set_dependencies", dep_set = {b → c}
  2. Top: reorder_node 更新 TopAgentState.task_relationships (DAG 边)
  3. Top: 通知 c.status = "blocked_by_b" (新状态)
  4. UI: c 卡片灰色等待, 显示 "等待 b 完成"
  5. b 完成时, c 状态自动转 "pending", dispatch 启动
  6. DAG 校验: 周期检测 (cycle prevention) — 守门 #13 a 强约束, 防 deadlock
- **Postcondition**: c 在 b done 之后自动启动, DAG 永无环

#### UC-12: 批量操作 (Bulk Action, TMO M-N4)

- **Actor**: Primary User
- **Trigger**: chat bar 输入 "暂停 a b c 三张卡" (或 UI 多选 → 暂停/取消/重命名 按钮)
- **Flow**:
  1. Top: parse_intent_node → intent = "bulk_action", target_task_ids = [a, b, c], action = "pause" | "cancel" | "set_priority"
  2. Top: bulk_node 循环: 对每个 task_id 走 card_action (per 02 §2.6)
  3. 并行 (asyncio.gather) 处理 N 个 task, 不串行 (避免 latency 累积)
  4. UI: N 张卡片状态同步更新, top 显示 batch summary "已暂停 3 张"
- **Postcondition**: N 张卡状态一致, 失败部分回滚 + 通知 (per §3.5 一致性)

#### UC-13: 跨任务汇总 + 元数据编辑 (Summarize + Metadata, TMO M-N5..M-N7)

- **Actor**: Primary User
- **Trigger**: chat bar 输入 "任务 a b c 进度" (summarize) / "把 a 改名为 xxx" (metadata)
- **Flow (summarize)**:
  1. Top: summarize_node 收集 [a, b, c] 各 SubAgentState.status + token_usage + 关键 milestone
  2. Top: LLM 聚合 → 表格化回答 (task / status / duration / progress)
  3. UI: chat bar 显示 + 卡片徽章更新
- **Flow (metadata)**:
  1. Top: metadata_node 解析 → metadata_update 协议 → TaskCardManager
  2. TaskCardManager: 更新 task_metadata 表 (per 03 §7 schema)
  3. UI: 卡片标题/标签实时更新
- **Postcondition**: 跨任务状态可视化, 元数据同步落库 (Master 表 RLS 必携 per 守门 #13 c)

### 2.4 機能一覧 (Function List)

| # | 機能 | 説明 | 優先度 |
|---|---|---|---|
| **F-01** | 全体代理 chat 入口 | UI 最下行 chat bar ↔ Top Agent WebSocket/SSE | P0 |
| **F-02** | 子代理 dispatch | Top → SubAgentPool.spawn(type, context) | P0 |
| **F-03** | 任务卡 UI | per sub-agent = 1 card, 状态/历史/操作 UI | P0 |
| **F-04** | LangGraph state 管理 | Top + Sub 独立 state schema + reducer | P0 |
| **F-05** | checkpointing (per-task) | 每 node 完了後自動 checkpoint | P0 |
| **F-06** | streaming (real-time) | SSE / WebSocket 100ms batch | P0 |
| **F-07** | tool 統合 (MCP 16 tools) | Top direct + Sub proxy | P0 |
| **F-08** | human-in-loop | interrupt_node + UI decision prompt | P0 |
| **F-09** | 跨 session resume | CheckpointStore + state 恢复 | P0 |
| **F-10** | 子代理履歴/状态可視化 | card 详情 modal + 全 state dump | P1 |
| **F-11** | 守门 violation 検出 | LangGraph 节点内 AGENTS.md §4 规则自动检查 | P1 |
| **F-12** | token OLU telemetry | per-task / per-session token 计量 | P1 |
| **F-13** | sub-agent context isolation | 独立 thread + 独立 memory space | P0 |
| **F-14** | top-agent ↔ sub-agent 通信 | dispatch / progress / result / interrupt 4 消息 | P0 |
| **F-15** | 5 域 Lead 决策追跡 | audit log + 决策矩阵可视化 | P2 |
| **F-16** | sub-agent 类型插件化 | SA-01..SA-09 注册表 + 动态加载 | P1 |
| **F-17** | PostgreSQL checkpointer | 本番 3-tier 永続化 | P2 |
| **F-18** | 跨仓 (Physis/RGS) 統合 | 外部 LangGraph インスタンス RPC | P3 |
| **F-19** | 任务卡合并 (TMO M-N1) | L0 协调, a+b → merged_task, checkpoint stash + supersede | P0 (v0.2) |
| **F-20** | 任务卡拆分 (TMO M-N2) | L0 协调, a → a1 + a2, checkpoint snapshot + forked context | P0 (v0.2) |
| **F-21** | 任务卡依赖编排 DAG (TMO M-N3) | L0 reorder_node, dep_set 协议, cycle prevention (守门 #13 a) | P0 (v0.2) |
| **F-22** | 批量操作 (TMO M-N4) | L0 bulk_node, asyncio.gather N 张卡 action, 部分失败回滚 | P0 (v0.2) |
| **F-23** | 跨任务汇总 (TMO M-N5) | L0 summarize_node, 跨 N SubAgentState 聚合, LLM 表格化 | P1 (v0.2) |
| **F-24** | 子代理重新分配 (TMO M-N6) | L0 reassign_node, 类型 SA-XX 切换, checkpoint preserved | P1 (v0.2) |
| **F-25** | 元数据编辑 (TMO M-N7) | L0 metadata_node, task_metadata 表更新 (Master RLS 必携) | P1 (v0.2) |

## 3. 非機能要件 (Non-Functional Requirements)

### 3.1 性能 (Performance)

| ID | 項目 | 目標値 |
|---|---|---|
| **NFR-P-01** | chat input → first token | ≤ 200ms p95 |
| **NFR-P-02** | sub-agent dispatch latency | ≤ 500ms p95 |
| **NFR-P-03** | 並行 sub-agent 数 | ≥ 50 |
| **NFR-P-04** | streaming update 推送延迟 | ≤ 100ms p95 |
| **NFR-P-05** | checkpoint flush latency | ≤ 1s (async, fsync batch) |
| **NFR-P-06** | Top 状态查询 (UI poll) | ≤ 50ms p95 |

### 3.2 可用性 (Availability)

| ID | 項目 | 目標値 |
|---|---|---|
| **NFR-A-01** | uptime | ≥ 99.5% |
| **NFR-A-02** | sub-agent 失敗 → retry | 1 次自動 retry + 通知 user |
| **NFR-A-03** | checkpoint 自動保存 | 每 node 完了後 (LangGraph native) |
| **NFR-A-04** | 优雅降级 | tool 不可用 → fallback direct call |

### 3.3 セキュリティ (Security)

| ID | 項目 | 適用 |
|---|---|---|
| **NFR-S-01** | 環境変数 hard ban | 禁 env value 打印 (per 守门 #5) |
| **NFR-S-02** | sub-agent context isolation | 独立 thread + 独立 memory space |
| **NFR-S-03** | tool call 監査 | 全 tool call → AuditLogger |
| **NFR-S-04** | Master data RLS | 100% RLS (per 守门 #13) |
| **NFR-S-05** | 5 域 Lead 配置隔离 | 真人到位前 Mavis 临时代签 (per 守门 #3 反転) |
| **NFR-S-06** | secret handling | $env:VAR 直接 invoke, 禁 expand/log |

### 3.4 保守性 (Maintainability)

| ID | 項目 | 適用 |
|---|---|---|
| **NFR-M-01** | LangGraph schema 文档化 | 自动生成 + 手工 review |
| **NFR-M-02** | 节点/边版本管理 | LangGraph state versioning + git tag |
| **NFR-M-03** | sub-agent template ライブラリ | SA-01..SA-09 + new template on-demand |

### 3.5 拡張性 (Scalability)

| ID | 項目 | 適用 |
|---|---|---|
| **NFR-E-01** | 新 sub-agent 类型 | plug-and-play via registry |
| **NFR-E-02** | 新 tool 統合 | MCP protocol 経由, 无需改 Top 代码 |
| **NFR-E-03** | 5 域 Lead 真人到位 | 配置注入, 无需改 LangGraph |
| **NFR-E-04** | 跨仓 (Physis/RGS) | RPC adapter, v0.3 计划 |

### 3.6 TMO 専用 NFR (Task Management Operations, v0.2)

per 02 §2.6 + 03 §2.2.1 (7 节点 TMO 详细):

| ID | 項目 | 目標 |
|---|---|---|
| **NFR-TMO-01** | 合并原子性 | merge_node 协调 a/b stash_state + dispatch merged_task 在同 1 个 LangGraph transaction 完成, 部分失败回滚 (a/b 状态还原) |
| **NFR-TMO-02** | 拆分可逆性 | split_node snapshot a checkpoint, a.split_into 记录, 子代理可基于 split checkpoint 重新 dispatch (UC-10 末尾) |
| **NFR-TMO-03** | 批量一致性 | bulk_node N 张卡 asyncio.gather, ≥ 80% 成功视为 partial success, 失败项 rollback + 通知 user (NFR-A-02 派生) |
| **NFR-TMO-04** | 血缘可追溯 | task_metadata 表 parent_task_id / merged_from / split_into / superseded_by 4 字段 100% 填, 跨 session 查得到 (per 守门 #13 c Master RLS) |
| **NFR-TMO-05** | DAG 校验 | reorder_node dep_set 提交前 cycle detection O(V+E) 跑, 检测到环 → reject + interrupt (per 守门 #13 a 强约束, 防 deadlock) |

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

## 5. 用語集 (Glossary)

| 用語 | 説明 |
|---|---|
| **全体代理 (Top-Level Agent / Total Agent)** | UI 最下行聊天栏背后 LangGraph instance，整体控制 Star 全局各个细节 |
| **子代理 (Sub-Agent)** | 任务卡単位 LangGraph subgraph, 特定 task 状態機 |
| **任务卡 (Task Card)** | UI 上 sub-agent 可视化窗，状态/履歴/操作 UI |
| **Dispatch** | 全体代理 → 子代理 task 割当 |
| **Checkpoint** | LangGraph state 永続化点，跨 session 恢复用 |
| **Reducer** | LangGraph state channel 更新関数 |
| **Subgraph** | 親 graph 内嵌套 sub-graph，隔離状態管理 |
| **Human-in-the-loop** | 人間判断待ち状態 (LangGraph interrupt) |
| **Streaming** | real-time ノード output 推送 (SSE/WebSocket) |
| **MCP (Model Context Protocol)** | 16 tools 統一 interface (per ADR-0032 stdio) |
| **OLU (One-Level Unit)** | AI 协作 token 単位 (1 SRE·周 = 1.2M, per STAR-OLU-001) |
| **5 域 (5 Domains)** | player / economy / match / social / admin，per 守门 #3 |
| **22 Domain Crates** | DDD bounded context 22 個 (identity/permission/work-item/...) |
| **DEC-008** | 一人公司 12 角色 治理模型 |
| **守门 (Guard)** | AGENTS.md §4 硬约束 13 main + 24 派生规 = 37 项 |
| **Mavis 接手** | Ulysses 授权的 root agent 代签身份 (per 19:39 JST) |
| **SubAgentPool** | 子代理管理器，类型 → instance mapping |
| **TaskCardManager** | UI 状态 ↔ Sub-agent 状態 mirror |
| **AuditLogger** | 全 tool call / dispatch / interrupt 記録 |
| **TokenTelemetry** | token 計量 + 集計 (per 守门 #4) |
| **TMO (Task Management Operations)** | v0.2 新增: L0 顶层代理跨任务卡管理操作集 (merge / split / reorder / bulk / reassign / summarize / metadata, per ADR-0046 + 02 §2.6) |
| **Task Operations Manager** | TMO 集中管理组件, 7 节点 (M-N1..M-N7) + 7 协议 集中调度, per 02 §1.3 C-16 |
| **Task Relationship Graph (DAG)** | 任务卡父子/合并/依赖关系有向无环图, 存 TopAgentState.task_relationships, cycle prevention 必携 (per 守门 #13 a) |
| **Supersede (取代)** | TMO 操作后, 源 task 状态变 superseded (历史保留, 不执行), pointer 指向新 task, 血缘可追溯 |

## 6. 想定シナリオ (Scenarios)

### S-01: 简单 query → 直接 tool call

```
User: "git status 確認して変更要約"
Top: parse_intent → simple query, direct tool
Top: tool_node → git_status MCP tool
Top: respond → 結果表示
```

### S-02: 复杂 audit → dispatch sub-agent

```
User: "5 域 Lead 配置状況を review して問題点指摘"
Top: parse_intent → complex, dispatch needed
Top: dispatch 5-域-lead-audit (SA-03)
Sub: 跨 22 domain crates 查 5 域 state
Sub: 守门 #3 检查
Sub: 生成 audit 报告
Top: aggregate → user
```

### S-03: 多任务并行 (8 domain H2 改造)

```
User: "H2 8 domain 改造並列で"
Top: parse_intent → 8 sub-agent plan
Top: 并行 dispatch 8 sub-agents
UI: 8 task card 并列
Subs: 独立运行
Top: collect → aggregate → user
```

### S-04: Human-in-the-loop

```
User: "子代理 暂停して、User 决策後再開"
Sub: interrupt_node (CRITICAL 守门 violation detected)
UI: card 黄色, "决策: 撤销 / 修复 / 豁免"
User: "修复, 改 user_id 强类型"
Sub: resume → 修复 → continue
```

### S-05: 跨 session resume

```
Day 1 22:00: Top + 5 sub-agents running
            checkpoint 自动保存
Day 2 09:00: 新 session
            Top: mount → CheckpointStore.list_pending()
            UI: 5 task card visible (state 恢复)
User: "sub-agent 3 继续"
            Top: load checkpoint → resume
```

### S-06: TMO 合并任务 a 和 b (per 2026-09-04 用户发令原话)

```
User: "合并任务 a 和任务 b"
Top: parse_intent → intent="task_merge", target_task_ids=[a, b]
Top: merge_node (per 02 §2.6 M-N1)
     │
     ├── sub_step 1: 通知 a / b 进入 stash_state
     │     a.checkpoint_id → a_stash (Transaction append-only)
     │     b.checkpoint_id → b_stash (Transaction append-only)
     │
     ├── sub_step 2: dispatch merged_task (SA-10 task-orchestrator, type="merge")
     │     merged_task.context = {
     │         "merged_from": [a.task_id, b.task_id],
     │         "merged_state": {a: a_stash, b: b_stash}
     │     }
     │
     ├── sub_step 3: 标记 a / b 状态 = "superseded"
     │     a.superseded_by = merged_task.task_id
     │     b.superseded_by = merged_task.task_id
     │     (a / b 历史保留, 不再执行, 血缘可追溯 per NFR-TMO-04)
     │
     └── sub_step 4: ui_streamer.push × 3
           • TaskCardUpdate(a, status="superseded", badge="已合并")
           • TaskCardUpdate(b, status="superseded", badge="已合并")
           • TaskCardCreate(merged_task, label="a + b")
UI: a / b 卡片灰显, merged_task 新卡出现
SA-10: 用 a + b stash_state 作为初始 context, 继续 plan / execute / verify / report
Top: collect → respond → user "已合并 a + b, merged_task 启动"
```

**派生场景 (S-06 变体)**:
- S-06a: 拆分 (把 a 拆成 a1 + a2)
- S-06b: 依赖编排 (b 完成后 c 才启动)
- S-06c: 批量 (暂停 a / b / c 三张卡)
- S-06d: 跨任务汇总 (a / b / c 进度)
- S-06e: 重新分配 (a 改用 SA-04 重跑)
- S-06f: 元数据 (把 a 改名为 "P0-1 联动审计")

## 7. 既知の制約 (Known Constraints) — 初版 v0.1

- 5 域 Lead 真人未到位 (per 守门 #3 反転: Mavis 临时代签)
- PostgreSQL checkpointer 未実装 (v0.2 计划)
- 跨仓 (Physis/RGS) RPC 未実装 (v0.3 计划)
- 並行 sub-agent 数上限 50 (リソース制約, NFR-P-03)
- 5 域 Lead 决策追跡 UI 未完成 (F-15 标 P2)
- token OLU telemetry 接入待 SRE Lead 真人
- **TMO 7 节点 (F-19..F-25) 实现 P0** (v0.2 文档完成, 实装待 P0-1/H2 阻塞解除, per PHASE-LANGGRAPH-TMO-IMPL-REPORT)
- **守门 #13 a 强约束派生**: L1 ↔ L1 禁止通信 → TMO 全部走 L0 协调 (per UC-09..UC-13 + 02 §2.6); 实证待 TMO 实装阶段 补

## 8. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-03 | 🟡 Draft v0.1; 2-level hierarchical LangGraph 架构 (全体代理 + 任务卡子代理) 落档 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手终审通过 (per 2026-09-03 17:51 JST 用户发令"另起一套架构view,专门设计langgraph相关的功能"); 3 份 IPA 文档 (要件/基本/詳細) 同步落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 (per 2026-08-27 19:39 + 21:59 JST 用户授权); SRE Lead 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 平台 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 评审主持 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); PM 5 域独立真实身份签字请 DDD Review 阶段补 |
| 1.2 | 架构师 / Mavis 接手审批 (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手终审通过 (per 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b"); TMO 7 节点 (合并 / 拆分 / 依赖 / 批量 / 汇总 / 重分配 / 元数据) 增量升档 v0.1 → v0.2, 加 5 UC (UC-09..UC-13) + 7 機能 (F-19..F-25) + 5 NFR (NFR-TMO-01..05) + 1 想定シナリオ (S-06) + 4 用語 (TMO / TaskOperationsManager / DAG / Supersede) + ADR-0046 索引; 随 02-basic-design.md + 03-detailed-design.md 同步升档 v0.2 + PHASE-LANGGRAPH-TMO-IMPL-REPORT 7 子项实装 phase 起 |
| 6 | SRE Lead (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 7 | 平台工程师 (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 8 | 评审主持人 (v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 9 | 项目负责人 (PM, v0.2 升版) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：2-level hierarchical LangGraph 架构 (全体代理 L0 + 任务卡子代理 L1) 落档；18 機能 / 4 NFR 類 / 14 制約 / 20 用語 / 5 想定シナリオ / 18 UC | 2026-09-03 17:51 JST 用户发令"另起一套架构view,专门设计langgraph相关的功能,需求文档、基本设计、详细设计按照日本IPA规则设计" |
| v0.2 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **TMO (Task Management Operations) 升版**: 加 5 UC (UC-09..UC-13: 合并 / 拆分 / 依赖编排 / 批量 / 跨任务汇总+元数据) + 7 機能 (F-19..F-25) + 5 NFR (NFR-TMO-01..05: 合并原子性 / 拆分可逆 / 批量一致性 / 血缘可追溯 / DAG 校验) + 1 想定シナリオ (S-06 合并任务 a 和 b) + 4 用語 (TMO / TaskOperationsManager / Task Relationship Graph / Supersede) + 1 制約派生 (L1↔L1 禁止 → TMO 全部 L0 协调) + 2 已知缺口 (TMO 实装 P0 / 守门 #13 a 实证待补) + 5 签字栏 v0.2 升版行 + ADR-0046 索引 + 引用 PHASE-LANGGRAPH-TMO-IMPL-REPORT 实装 phase 计划; 随 02-basic-design.md + 03-detailed-design.md 同步升档 v0.2; 守门 #1+#5+#6+#7+#9+#10+#12+#19+#20+#22+#13 跨 stage 全过 (文档工作无 .rs 改动, cargo check 不需要跑) | 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b这种全局管理的ai功能是要能实现的" (per ask_d076c26d3fbf599eec1c32fd 拍板 (1) 范围=完整 7 节点全覆盖 (2) 文档策略=原地升版 v0.1 → v0.2 (3) 实装阶段=文档+commit 一并落), ~0.06M token 估 |

---

## 10. 引用文档

- [AGENTS.md](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 仓库根 AI 协作硬约束入口
- [ADR-0032 MCP Transport stdio](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) — 16 tools
- [ADR-0033 代签规则反转](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) — 本规则正式 ADR
- [ADR-0030 Agent Lease/Heartbeat/Resume](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) — 11 字段 + 跨 Agent Handoff
- [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens
- [STAR-P3-WBS-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/reports/STAR-P3-WBS-001.md) — P3 阶段 WBS
- [scripts/automation/dispatcher.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/dispatcher.py) — 现有 sub-agent dispatch 基础
- [scripts/automation/console_server.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/console_server.py) — 现有 FastAPI 8080 调试控制台
- [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) — agent 交互 Python 化规则 (守门 #19)
- [02-basic-design.md](02-basic-design.md) — 基本設計書
- [03-detailed-design.md](03-detailed-design.md) — 詳細設計書
