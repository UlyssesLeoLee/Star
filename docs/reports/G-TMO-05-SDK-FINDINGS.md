# G-TMO-05 LangGraph SDK 0.2.x interrupt_response API 实证 — 关闭

> **报告主题**: G-TMO-05 (LangGraph SDK 0.2.x interrupt_response API alpha 确认) 实证
> **报告时间**: 2026-09-05 02:25 JST
> **报告人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **结论**: G-TMO-05 **不适用**, Star 仓用纯 Python 概念 interrupt, 不依赖 LangGraph SDK
> **触发**: 2026-09-04 17:19 JST 用户发令"完成后续全部任务" + 9/4 18:30 JST 守门 #3 反转 5 子代理兼任 + 9/5 02:25 JST 自主推进

---

## §0 目的

G-TMO-05 (per HANDOFF-ST-001 v1.4 §18.6 6 待续做项 + PHASE-P4-V2-TMO-CI-IMPL-REPORT v0.2 §3) 要求 "LangGraph SDK 0.2.x interrupt_response API alpha 确认 (实装前先 pip show langgraph)".

实证 G-TMO-05 是否需落地.

## §1 实证流程

### 1.1 pip show langgraph

```
$ pip show langgraph
WARNING: Package(s) not found: langgraph
```

**结果**: langgraph **未安装**.

### 1.2 Star 仓 requirements 检索

```
$ glob **/requirements*.txt
No files matched
$ glob **/pyproject.toml
No files matched
```

**结果**: Star 仓**无 Python requirements.txt / pyproject.toml** (workspace 是 Rust + Cargo, Python 仅限 scripts/automation/ 工具脚本).

### 1.3 02-basic-design.md v0.2 引用分析

per `docs/architecture/2026-09-03-langgraph/02-basic-design.md` v0.2:

| # | 引用 | 性质 |
|---|---|---|
| §2.1 | "StateSchema (per LangGraph state schema 概念)" | 概念参考 |
| §2.6.5 C-12 | "InterruptManager (human-in-the-loop interrupt / resume, L0/L1, P0)" | 概念组件, 实际实装 = Python signal |
| §3.4 | "interrupt (upstream)" | 概念, 实际实装 = TopAgentState.active_tmo_operation = None |
| §4.2.3 | "interrupt_response API" | **概念参考, 不是强制依赖** |

**结果**: "LangGraph" 在 Star 仓**仅是概念参考** (state schema / interrupt / checkpoint), 实际实装用**纯 Python async fn + FastAPI + SQLite WAL** (per 守门 #19 + 守门 #22 + 守门 #23 mock 备选).

### 1.4 现有 8 节点实装与 LangGraph SDK 关系

| 节点 | 实装方式 | LangGraph SDK 依赖 |
|---|---|---|
| TMO-01 merge_node | Python async fn (wt-tmo-01-merge) | 无 |
| TMO-02 split_node | Python async fn (wt-tmo-02-split, 132/132 tests) | 无 |
| TMO-03 reorder_node | Python class ReorderNode (wt-tmo-03-dag) | 无 |
| TMO-04 bulk_node | Python factory make_bulk_node (wt-tmo-04-bulk) | 无 |
| TMO-05 summarize_node | Python async fn (本 session feat/tmo-05-06-07) | 无 |
| TMO-06 reassign_node | Python async fn (本 session) | 无 |
| TMO-07 metadata_node | Python async fn (本 session) | 无 |
| TMO-08 deps-survey | Python doc-only (origin/main) | 无 |

**结果**: 8 节点全部纯 Python 实装, **0 节点依赖 LangGraph SDK**.

## §2 决策

G-TMO-05 **不适用**, 关闭:

- **理由 1**: Star 仓不依赖 LangGraph SDK, pip show langgraph not found, 无 requirements 引用
- **理由 2**: "interrupt" 在 Star 仓是**纯 Python 概念** (C-12 InterruptManager, TopAgentState.active_tmo_operation reducer), 实际实装 = Python 内部 signal / state machine, **不调** langgraph.interrupt() API
- **理由 3**: 8 节点全部实装 100% pass (per 守门 #1 v19 + pytest 37/37 + 守门 #13 a L0 协调), **不需要** LangGraph SDK 也能跑

**影响**:
- HANDOFF-ST-001 v1.4 §18.6 6 待续做项 → **G-TMO-05 关闭** → 5 待续做项
- PHASE-P4-V2-TMO-CI-IMPL-REPORT v0.2 §3 已知缺口 → 12 项 → **11 项**
- 不需要预留 token (~0.05M 原 G-TMO-05 估)

## §3 后续

| 候选 | 消耗 | 阻塞 | 备注 |
|---|---|---|---|
| G-TMO-04 task_metadata DDL | ~0.1M | 无 | SQL schema + Python adapter + 1 集成 test |
| G-DEP-01 P0 工具实装 | ~0.4-0.6M | 无 | 3 tool: create_merge_request / create_worktree / search_issues |
| G-DEP-02 P1 工具实装 | ~0.3-0.5M | 无 | 4 tool: search_code / get_symbol / find_references / get_code_context |
| _ARCHIVED_*.md 收编 | ~0.1M | 无 | git mv + WBS 同步 + HANDOFF v1.5 |
| Frontend 4 err 修根因 | ~0.3-0.5M | 无 | FeatureToggles + refactor-state-machine + tailwind-merge |
| HANDOFF v1.5 综合升版 | ~0.05M | 无 | 聚合 v0.2 + 本报告 + 5 域 Lead 真人到位流程 |

---

## §4 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 2026-09-05 02:25 JST | per 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:25 JST | per 8/27 20:56 JST 强化 + 真人到位后追溯签字 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:25 JST | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:25 JST | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手临时代签 | 2026-09-05 02:25 JST | 同上 |

## §5 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-05 02:25 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses** | **G-TMO-05 关闭: Star 仓不依赖 LangGraph SDK, interrupt 走纯 Python 概念 (per pip show langgraph not found + 02-basic-design.md v0.2 §2.6.5 C-12 + 8 节点全纯 Python 实证); 6 待续做项 → 5 待续做项** | **9/5 02:25 JST 自主推进 (per 9/4 17:36 JST "允许按照你推荐推进" + no-progress guard 触发) → 守门 #12 commit-time docs 同步触发 v0.1** |
