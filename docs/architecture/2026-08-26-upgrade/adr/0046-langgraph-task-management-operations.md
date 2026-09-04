# ADR-0046: Star LangGraph TMO 任务卡管理操作 (Task Management Operations)

> **状态**：🟢 Accepted v1.0
> **日期**：2026-09-04
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../2026-08-26-upgrade-plan.md) (待归档) · [Star LangGraph 統合アーキテクチャ 要件定義書 v0.2](../2026-09-03-langgraph/01-requirements.md)
> **依赖**：[ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) · [ADR-0033 代签规则反转](0033-agent-co-signing-policy.md) · [AGENTS.md §4 守门硬约束](../../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展開](../../../AGENTS.md)
> **关联**：[01-requirements.md v0.2](../2026-09-03-langgraph/01-requirements.md) · [02-basic-design.md v0.2](../2026-09-03-langgraph/02-basic-design.md) · [03-detailed-design.md v0.2](../2026-09-03-langgraph/03-detailed-design.md) · [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md](../../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) · [docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md](../../../kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) · [docs/automation-design.md](../../../automation-design.md)

---

## 1. 背景与问题

### 1.1 业务诉求 (per 2026-09-04 19:15 JST 用户发令原文)

Ulysses 在 9/4 19:15 JST 明确发令:

> **"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b这种全局管理的ai功能是要能实现的"**

**核心 3 维**:
1. **L0 顶层代理 (Top-Level Agent)** 在 UI **底端聊天栏** (chat bar) 背后, 整体控制 Star 全局各个细节 (per [01 §0](../2026-09-03-langgraph/01-requirements.md))
2. **整体统筹规划** — 跨任务卡管理能力, 而非单卡操作
3. **发号施令** — 全局管理类 AI 功能必须能实现, 用户原话例子 = "合并任务 a 和任务 b"

### 1.2 现状缺口 (per [01 §7 既知の制約](../2026-09-03-langgraph/01-requirements.md) v0.1 实证)

[docs/architecture/2026-09-03-langgraph/ 3 份 IPA 文档 v0.1](../2026-09-03-langgraph/) 已落档 LangGraph 2-level hierarchical 架构 (L0 全体代理 + L1 任务卡子代理), 但只覆盖:

| 已覆盖 (v0.1) | 缺失 (用户诉求) |
|---|---|
| 创建任务卡 (UC-02 dispatch) | **合并** 任务卡 (用户原话例子) |
| 单卡生命周期 (pause / resume / cancel / interrupt) | **拆分** 任务卡 |
| 多任务并行 (UC-03 N 並行) | **依赖编排** (DAG) |
| 跨 session 状态 (UC-06 checkpoint) | **批量操作** (多选暂停/取消) |
| 16 MCP tools 子代理内调用 (UC-07) | **重新分配** (sub-agent 类型 SA-XX 切换) |
| 5 域 Lead 决策辅助 (UC-08) | **跨任务汇总** (查 N 张卡进度) |
| 通信协议: dispatch / cancel / interrupt_response / progress / result | **元数据编辑** (重命名/标签/备注) |

**根因**: L0 StateGraph v0.1 只有 `parse_intent → dispatch / tool / respond` 主路径, **缺一整组 Task Management Operations (TMO) 节点**; 通信协议 (§2.3.1) 也只定义了 `dispatch / cancel / interrupt_response`, 没有 merge/split/reorder/bulk/reassign/metadata.

### 1.3 架构冲突 (守门 #13 a 强约束)

per [AGENTS.md §4 #13 a](../../../AGENTS.md): **L1 ↔ L1 禁止通信** (防止状态污染). 因此所有"跨任务卡"操作 (合并/拆分/依赖/批量/重分配/汇总/元数据) **必须走 L0 协调** — TMO 是 L0 StateGraph 的扩展, 不是新 sub-agent type.

---

## 2. 决策

**新增 L0 顶层代理 TMO (Task Management Operations) 能力集: 7 节点 + 7 协议 + 7 组件 + 25 新 module, 满足 Ulysses 2026-09-04 19:15 JST 发令"langgraph功能需要可以操控任务卡"诉求.**

### 2.1 TMO 7 节点 (per [02 §2.6.1](../2026-09-03-langgraph/02-basic-design.md))

| Node ID | 名称 | 責務 | 触发 chat bar 例子 |
|---|---|---|---|
| **M-N1** | `merge_node` | 合并 a + b → merged_task | "合并任务 a 和任务 b" (用户原话) |
| **M-N2** | `split_node` | 拆分 a → a1 + a2 | "把任务 a 拆成 a1 和 a2" |
| **M-N3** | `reorder_node` | 依赖 DAG 边更新 + cycle detection | "任务 b 完成后才能启动 c" |
| **M-N4** | `bulk_node` | N 张卡批量 action | "暂停 a b c 三张卡" |
| **M-N5** | `summarize_node` | 跨任务汇总 | "任务 a b c 进度" |
| **M-N6** | `reassign_node` | sub-agent 类型 SA-XX 切换 | "把 a 改用 SA-04 重跑" |
| **M-N7** | `metadata_node` | task_metadata 表更新 (Master RLS 必携) | "把 a 改名为 xxx" |

### 2.2 TMO 7 协议 (per [02 §2.6.2](../2026-09-03-langgraph/02-basic-design.md))

`merge_request` / `split_request` / `dep_set` / `bulk_action` / `reassign_request` / `metadata_update` / `summarize_result` — 7 类新通信协议, 全部 L0 → L1 (除 metadata_update 是 L0 → L0).

### 2.3 TMO 7 组件 (per [02 §1.3 C-16..C-22](../2026-09-03-langgraph/02-basic-design.md))

- **C-16** TaskOperationsManager (集中调度 7 节点)
- **C-17** TaskRelationshipGraph (DAG, 4 字段 parent/merged_from/split_into/superseded_by)
- **C-18** BulkOperationQueue (asyncio.gather + 部分失败回滚)
- **C-19** MetadataRegistry (task_metadata 表 Master RLS 必携)
- **C-20** DAGValidator (cycle detection O(V+E))
- **C-21** ReassignManager (SA-XX 切换 + checkpoint preserved)
- **C-22** SummarizeCollector (跨 N SubAgentState 聚合)

### 2.4 State Schema 扩展 (per [02 §2.6.4 + §3.2](../2026-09-03-langgraph/02-basic-design.md))

- **TopAgentState** 加 5 字段: `task_relationships` (DAG) / `superseded_tasks` (append-only) / `bulk_operations` (FIFO queue) / `last_summarize_result` / `active_tmo_operation`
- **SubAgentState** 加 5 血缘字段: `parent_task_id` / `merged_from` (append) / `split_into` (append) / `superseded_by` / `checkpoint_snapshot`

### 2.5 8 外部 API 端点 (per [02 §5.2](../2026-09-03-langgraph/02-basic-design.md))

`/api/tmo/merge` / `/split` / `/dependencies` / `/bulk` / `/summarize` / `/reassign` / `/metadata` (POST) + `/relationships` (GET) — 走 FastAPI 8080 console_server.py 扩展 (per 守门 #9 v3 / #24 subprocess 走 console_server).

---

## 3. 备选方案 (Alternatives Considered)

### 3.1 备选 A: L1 ↔ L1 直连 (拒绝)

**思路**: 让 sub-agent a 直接调 sub-agent b 完成"合并", 不经 L0.

**否决理由**:
- 违反守门 #13 a 强约束 (L1 ↔ L1 禁止通信, 防止状态污染)
- 状态污染实证风险高 (a/b 各自 state merge 时 race condition)
- 守门 #9 (子代理 status ≠ 实际成功) 加剧, 跨 sub-agent 协调不可观测

### 3.2 备选 B: 新建 worker subagent type (拒绝)

**思路**: 派 worker subagent (走 `dispatcher.py`) 协调 a/b 合并, 不经 L0 LangGraph.

**否决理由**:
- 跟现有 Mavis worker subagent (per [01 §1.0](../2026-09-03-langgraph/01-requirements.md) Sub-Agent vs Worker subagent 区别) 混用, 破坏两套系统并存架构
- worker subagent 走 subprocess + brief, 没法 in-process asyncio 协调 (latency 高)
- 守门 #19 Python 化派生规: TMO 应该是 L0 LangGraph 状态化能力, 不是单次 task

### 3.3 备选 C: 仅 UI 层 (chat bar → 调后端单次操作) (拒绝)

**思路**: 只在 UI 层加按钮, 不扩展 L0 LangGraph 节点.

**否决理由**:
- 守门 #11 缺标比错标: 现状 L0 缺一整组 TMO 节点, 仅 UI 按钮是 UI 表面能力, L0 不能跨任务协调
- 守门 #4 token-OLU: UI 直调后端没法走 LLM 解析 (per NFR-P-01 first token ≤ 200ms)
- 跟"整体统筹规划"诉求不匹配: 用户要的是 L0 AI 能力, 不是手动 UI 按钮

### 3.4 备选 D (选定): TMO 7 节点 + 7 协议 + 7 组件 — L0 StateGraph 扩展

**思路**: 扩展 L0 StateGraph, 加 7 节点 + 7 协议 + 7 组件 + 25 新 module + 8 外部 API 端点; 走守门 #19 Python 化 (`scripts/automation/task_ops.py`) + 守门 #9 v3 (subprocess 走 console_server) + 守门 #22 (调试控制台不污染 main).

**选定理由**:
- 守门 #13 a 强约束: 全部 L0 协调, 唯一 cross-task actor
- 守门 #13 c/d W/T/M: task card 状态 = Work (短 TTL), checkpoint history = Transaction (append-only), metadata = Master (SCD Type 2)
- 守门 #4 token-OLU: TMO 是 L0 决策, 不重 L1 token; TokenTelemetry 计量每个 TMO 操作
- 守门 #19 Python 化: 实装走 `scripts/automation/task_ops.py` (跟 console_server.py 复用)
- 守门 #23 (AI 修改 mock): TMO 调试走 ai_edit_mock.py, 不开 OpenAI
- 守门 #12 AI 协作文档治理: 禁回溯叙事, BAS 引用 git 实证, 缺标比错标 (per 本 ADR 3 段结构 + 7 段报告 + 9 已知缺口)

---

## 4. 后果 (Consequences)

### 4.1 正面后果 (Positive)

1. **满足用户诉求**: 底端聊天栏 → L0 chat input → 意图解析 → 7 节点分发 → 任务卡操作, 全链路 AI 驱动
2. **守门合规**: 守门 #13 a/d 强约束派生规 (L1↔L1 禁止 → 全部 L0 协调, W/T/M 分类清晰)
3. **架构对称**: TMO 是 L0 StateGraph 7 节点扩展, 跟既有 T-N1..T-N7 (parse_intent / dispatch / tool / collect / respond / interrupt / guard_check) 对称, 不破坏既有架构
4. **可观测**: 7 Prometheus metrics (per [02 §2.6.6](../2026-09-03-langgraph/02-basic-design.md)) + 审计 log append-only + GuardEnforcer 7 项守门合规
5. **可重放**: 任务卡血缘 100% 追溯 (parent_task_id / merged_from / split_into / superseded_by 4 字段, per NFR-TMO-04)
6. **可扩展**: 后续 TMO 节点 (e.g., M-N8 模板化 / M-N9 调度优化) 走 SubAgentRegistry 注册即可, 无需改 L0 StateGraph

### 4.2 负面后果 / 风险 (Negative / Risks)

1. **实装工作量**: 7 子项估 ~2.5M tokens (per [PHASE-LANGGRAPH-TMO-IMPL-REPORT §1](../2026-09-03-langgraph/01-requirements.md)), 跟 AGENTS §7 #8 ~3.0M 兼容 (留 0.5M 给 9 SA 类型 stub)
2. **P0-1 / H2 阻塞依赖**: 实装待 P0-1 联动审计 + H2-EXT 5 domain 跨域字段扩展 阻塞解除, 不能立刻起
3. **LangGraph SDK alpha 风险**: 守门 #9 实证 (per [03 §9](../2026-09-03-langgraph/03-detailed-design.md)) LangGraph 0.2.x interrupt_response API alpha, 实装前必先 `uv add langgraph@latest` + `pip show langgraph` 确认
4. **守门 #13 a 实证缺口**: L1↔L1 禁止通信 → TMO 全部 L0 协调; DAGValidator cycle detection O(V+E) 实证待实装
5. **5 域 Lead 真人未到位**: TMO 跨域操作代签权归 Mavis 临时代签 (per 守门 #3), 真人到位后追溯签字, 不沿用代签决策

### 4.3 中和措施 (Mitigations)

| 风险 | 中和措施 | 触发 |
|---|---|---|
| 实装工作量 | 7 子项按 token 预算排序推进, 优先 TMO-01 (merge) + TMO-03 (DAG validator, 守门 #13 a 实证) | PHASE-LANGGRAPH-TMO-IMPL-REPORT §1 |
| P0-1 / H2 阻塞 | 实装 phase 跨 session 续做, brief 必先落档 (per 守门 #20) | PHASE-LANGGRAPH-TMO-IMPL-REPORT §4 |
| LangGraph SDK alpha | 实装前 `uv add langgraph@latest` + `pip show langgraph` 确认 | TMO-01..TMO-07 启动前 |
| 守门 #13 a 实证 | TMO-03 集成测试 跑通 cycle detection O(V+E) | TMO-03 子项交付 |
| 5 域 Lead 真人 | 守门 #3 v2 派生规: 真人到位后追溯签字, 不沿用代签决策 | DDD Review 阶段 |

---

## 5. 实施计划 (Implementation Plan)

per [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md](../../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md):

| 阶段 | 内容 | 状态 |
|---|---|---|
| 文档 v0.2 (本 ADR 落档配套) | 3 份 IPA 文档升版 v0.1 → v0.2 + ADR-0046 + PHASE 报告 v0.1 + AGENTS.md §6.1 / §7 / §8 同步 | ✅ v0.2 落档 (本 commit 一起) |
| 实装 v0.3 (跨 session 续) | TMO-01..TMO-07 7 子项实装, 走守门 #19 Python 化 + #9 v3 subprocess + #22 控制台不污染 main + #23 AI mock | 🟡 planned (per PHASE-LANGGRAPH-TMO-IMPL-REPORT) |
| E2E v0.4 (TMO 全部实装后) | E2E-09..E2E-13 跑通, chat bar → 7 节点 → UI 卡片灰显 + 新卡 | 🟡 pending (实装完成后) |
| 5 域 Lead 真人到位 v0.5 | 5 域 Lead 真人追溯签字 (per 守门 #3 v2), 不沿用代签决策 | 🟡 pending (5 域 Lead 真人到位) |

---

## 6. 决策日志 (Decision Log)

| 日期 | 决策 | 触发 | 来源 |
|---|---|---|---|
| 2026-09-04 17:51 JST | 拍板方向: TMO 完整 7 节点 + 原地升版 v0.2 + 文档+commit 一并落 | ask_d076c26d3fbf599eec1c32fd 3 问拍板 | Ulysses 9/4 17:51 JST |
| 2026-09-04 19:15 JST | 用户发令"langgraph功能需要可以操控任务卡, 合并任务a和任务b" | TMO 业务诉求 | Ulysses 9/4 19:15 JST |
| 2026-09-04 19:35 JST | Mavis 接手代签, 落地文档 v0.2 + ADR-0046 + PHASE 报告 + AGENTS.md 同步 | 守门 #10 + 19:39 JST 授权 | Mavis 9/4 19:35 JST |

---

## 7. 签字栏 (Signatures)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-04 | 🟢 Accepted v1.0; TMO 7 节点决策落档, 跟 [01 §UC-09..UC-13](../2026-09-03-langgraph/01-requirements.md) + [02 §2.6](../2026-09-03-langgraph/02-basic-design.md) + [03 §3.2.1.1](../2026-09-03-langgraph/03-detailed-design.md) 同步 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手终审通过 (per 2026-09-04 19:15 JST 用户发令 + ask_d076c26d3fbf599eec1c32fd 拍板 3 问: 范围=完整 7 节点 + 文档策略=原地升版 + 实装阶段=文档+commit 一并落); 3 备选方案拒绝理由 + 5 后果 (3 正 / 5 风险 / 5 中和) + 5 阶段实施计划 + 3 决策日志 + 5 签字栏落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 8. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：TMO 7 节点决策落档 (per Ulysses 9/4 19:15 JST 发令"langgraph功能需要可以操控任务卡, 合并任务a和任务b") + 3 备选方案 (L1↔L1 直连 / 新建 worker subagent / 仅 UI 层) 全部拒绝 + 选定 D 方案 (L0 StateGraph 7 节点扩展) + 5 后果 (3 正 / 5 风险 / 5 中和) + 5 阶段实施计划 + 3 决策日志 + 5 签字栏 (Mavis 接手代签) | 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b这种全局管理的ai功能是要能实现的" (per ask_d076c26d3fbf599eec1c32fd 拍板 (1) 范围=完整 7 节点全覆盖 (2) 文档策略=原地升版 v0.1 → v0.2 (3) 实装阶段=文档+commit 一并落), 跟 3 份主文档 v0.2 + PHASE-LANGGRAPH-TMO-IMPL-REPORT v0.1 同步落档, ~0.04M token 估 |

---

## 9. 引用文档 (References)

- [01-requirements.md v0.2](../2026-09-03-langgraph/01-requirements.md) — UC-09..UC-13 + F-19..F-25 + NFR-TMO-01..05 + S-06
- [02-basic-design.md v0.2](../2026-09-03-langgraph/02-basic-design.md) — §2.6 TMO 全节 + 7 组件 C-16..C-22 + 7 协议 + 5 Reducer + 8 API 端点
- [03-detailed-design.md v0.2](../2026-09-03-langgraph/03-detailed-design.md) — task_ops/ 模块 + M-19..M-25 + §3.2.1.1 7 节点 Python 実装 + SA-10 + superseded 终态 + UT-20..UT-26 / IT-10..IT-12 / E2E-09..E2E-13
- [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.1](../../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) — 7 子项实装 phase 计划
- [AGENTS.md §4 #13 W/T/M 横展開](../../../AGENTS.md) — DB 三類横展開硬约束
- [AGENTS.md §7 #8 Star LangGraph 統合アーキテクチャ 初版实装](../../../AGENTS.md) — ~3.0M token 预算
- [ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) — 11 字段 + 跨 Agent Handoff
- [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) — 16 tools
- [ADR-0033 代签规则反转](0033-agent-co-signing-policy.md) — Mavis 接手代签授权
- [docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md](../../../kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) — DB W/T/M 横展開验证
- [docs/automation-design.md](../../../automation-design.md) — agent 交互 Python 化 (守门 #19)
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt / Command
