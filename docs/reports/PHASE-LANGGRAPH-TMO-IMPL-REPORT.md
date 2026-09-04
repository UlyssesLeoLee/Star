# PHASE-LANGGRAPH-TMO-IMPL-REPORT — Star LangGraph TMO 任务卡管理操作 实装计划

> **状態**：🟡 Draft v0.1 (计划阶段, 实装未启动)
> **日期**：2026-09-04
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手代签（per 2026-08-27 19:39 + 21:59 JST 用户授权）
> **依赖**：[01-requirements.md v0.2](../architecture/2026-09-03-langgraph/01-requirements.md) · [02-basic-design.md v0.2](../architecture/2026-09-03-langgraph/02-basic-design.md) · [03-detailed-design.md v0.2](../architecture/2026-09-03-langgraph/03-detailed-design.md) · [ADR-0046 LangGraph TMO](../architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [PHASE-LANGGRAPH-TMO-IMPL-REPORT](.) (本文件) · [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [STAR-OLU-001.md token 基线](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) · [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md)
> **关联文档**：[PHASE-D2-CLI-IMPL-REPORT.md](PHASE-D2-CLI-IMPL-REPORT.md) (参考格式) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) (参考格式)

---

## 0. 目的 (Purpose)

本文档规划 **Star LangGraph TMO (Task Management Operations)** 7 节点的实装 phase, per [01 §UC-09..UC-13](../architecture/2026-09-03-langgraph/01-requirements.md) + [02 §2.6](../architecture/2026-09-03-langgraph/02-basic-design.md) + [03 §3.2.1.1](../architecture/2026-09-03-langgraph/03-detailed-design.md) 设计.

**核心目标**: L0 顶层代理从底端聊天栏 (chat bar) 发号施令, 操控任务卡 (合并 / 拆分 / 依赖编排 / 批量 / 跨任务汇总 / 重新分配 / 元数据), 满足 Ulysses 2026-09-04 19:15 JST 发令 "langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b这种全局管理的ai功能是要能实现的".

**实装路径 (per 守门 #19 + #9 v3 + #24)**: 走 `scripts/automation/task_ops.py` Python 基类 + FastAPI 8080 console_server.py 扩展 `/api/tmo/*` 端点 + Next.js 前端 chat bar 集成. **不写 .rs**, 主仓 Rust 编译链不动 (per 守门 #1 v22 调试控制台不污染 main).

---

## 1. 任务完成矩阵 (Task Completion Matrix, 7 子项)

| # | 子项 | 节点 ID | token 预算 | 实装路径 | 依赖守门 | 状态 |
|---|---|---|---|---|---|---|
| **TMO-01** | merge_node + SA-10 task-orchestrator | M-N1 + SA-10 | ~0.4M | `task_ops/nodes/merge_node.py` + `sub_agent/types/sa_10_task_orchestrator.py` + `/api/tmo/merge` | #13 a L1↔L1 / #13 d Transaction / #19 Python 化 | 🟡 planned (本 phase 起, 跨 session 续) |
| **TMO-02** | split_node | M-N2 | ~0.3M | `task_ops/nodes/split_node.py` + `/api/tmo/split` | #13 a L1↔L1 / #13 d / #19 | 🟡 planned |
| **TMO-03** | reorder_node + DAGValidator | M-N3 + C-20 | ~0.5M | `task_ops/nodes/reorder_node.py` + `task_ops/dag_validator.py` (cycle detection O(V+E)) + `/api/tmo/dependencies` | #13 a cycle prevention 强约束 / #19 | 🟡 planned |
| **TMO-04** | bulk_node + BulkOperationQueue | M-N4 + C-18 | ~0.4M | `task_ops/nodes/bulk_node.py` + `task_ops/bulk_queue.py` (asyncio.gather + partial failure rollback) + `/api/tmo/bulk` | #13 d / NFR-TMO-03 批量一致性 / #19 | 🟡 planned |
| **TMO-05** | summarize_node + SummarizeCollector | M-N5 + C-22 | ~0.3M | `task_ops/nodes/summarize_node.py` + `task_ops/summarize_collector.py` + `/api/tmo/summarize` | #19 | 🟡 planned |
| **TMO-06** | reassign_node + ReassignManager | M-N6 + C-21 | ~0.3M | `task_ops/nodes/reassign_node.py` + `task_ops/reassign_manager.py` (SA-XX 切换 + checkpoint preserved) + `/api/tmo/reassign` | #13 a / #13 d / #19 | 🟡 planned |
| **TMO-07** | metadata_node + MetadataRegistry | M-N7 + C-19 | ~0.3M | `task_ops/nodes/metadata_node.py` + `task_ops/metadata_registry.py` (Master RLS 必携) + `/api/tmo/metadata` + `/api/tmo/relationships` GET | #13 c Master RLS / #19 | 🟡 planned |
| **Σ** | **TMO 7 节点 + 7 组件 + 25 module + 8 端点** | M-N1..M-N7 + C-16..C-22 + M-19..M-25 | **~2.5M** | — | — | 🟡 planned |

**列含义**:
- `token 预算`: per 守门 #4 token-OLU (1 SRE·周 ≈ 1.2M tokens)
- `实装路径`: 守门 #19 Python 化 (per [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) §1.2 + §3.1)
- `依赖守门`: 关键守门编号
- `状态`: 🟡 planned = 文档 + schema 落档 (v0.2 已完成), 实装 phase 启动待 P0-1/H2 阻塞解除

**总估**: ~2.5M tokens, 跟 [AGENTS.md §7 #8](../AGENTS.md) "Star LangGraph 統合アーキテクチャ 初版实装 ~3.0M" 兼容 (TMO 是 Star-LG 初版实装的核心子集, 留 0.5M 给 9 SA 类型 stub + 基础 task card UI + checkpoint Tier 1+2).

---

## 2. 验证摘要 (Verification Summary)

### 2.1 文档 v0.2 阶段 (已完成)

| 验证项 | 状态 | 证据 |
|---|---|---|
| 3 份 IPA 文档升版 v0.1 → v0.2 | ✅ | `docs/architecture/2026-09-03-langgraph/01-requirements.md` (UC-09..UC-13 + F-19..F-25 + NFR-TMO-01..05 + S-06 + 4 用語 + 5 签字 v0.2 升版 + v0.2 修订历史) · `02-basic-design.md` (§2.6 TMO 全节 + 7 组件 C-16..C-22 + 7 协议 + 5 Reducer + State Schema 扩展 + 8 API 端点) · `03-detailed-design.md` (task_ops/ 模块 + M-19..M-25 + 7 节点 Python 実装 + SA-10 + superseded 终态 + UT-20..UT-26 / IT-10..IT-12 / E2E-09..E2E-13) |
| ADR-0046 LANGGRAPH-TMO 落档 | ✅ | `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` |
| AGENTS.md 同步 (per 守门 #12 缺标比错标) | 🟡 pending | 本 phase 后续 commit (TMO-08 同步) |

### 2.2 实装阶段 (planned, 跨 session 续)

> **守门 #1 阶段 1 实证待补**: 实装阶段每次子项完成必跑:
> 1. `cargo check --workspace --all-targets -j 4` (per 守门 #1 v19 / v1-v14 派生规, 主仓编译链 0 err, 但 TMO Python 化不进 main)
> 2. `cargo fmt + clippy` (主仓)
> 3. `python -m pytest tests/unit/test_task_ops_nodes.py -v` (TMO 7 节点单测 100% pass)
> 4. `python -m pytest tests/integration/test_tmo_*.py -v` (TMO 3 集成测试 100% pass)
> 5. `cd frontend && pnpm test tests/e2e/test_uc09_13_*.spec.ts` (TMO 5 E2E 100% pass)
> 6. `console_server.py` 起来 (port 8080) + 调试页 AI 修改 mock 验证 (per 守门 #22 / #23)

### 2.3 守门合规预期

| 守门 | TMO 派生约束 | 验证位置 |
|---|---|---|
| **#1 (R-05)** | 文档工作不跑 cargo; 实装阶段每次守门过 | 本节 2.2 |
| **#5 (env var)** | TMO 操作不传 secret (Mavis 临时代签 per #3) | 守门 #1 v22 (调试控制台不污染 main) |
| **#6 (PowerShell only)** | TMO 调试走 PowerShell, 不走 bash | 自动化脚本 `task_ops.py` 内部 subprocess |
| **#7 (0 unsafe)** | N/A (Python 化, 不写 .rs) | — |
| **#9 v3 (subprocess 走 console_server)** | TMO UI 操作走 Next.js API → FastAPI 8080 → subprocess 调 task_ops.py | [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) §12 |
| **#10 (代签规则)** | Mavis 临时代签 Ulysses (per 19:39 JST 授权) | 修订历史 author=Ulysses |
| **#12 (AI 協作文档治理)** | BAS 引用 git 实证, 禁回溯叙事, 缺标比错标 | 本报告 + 3 份主文档 + ADR-0046 修订 |
| **#13 a (L1↔L1 禁止)** | TMO 7 节点全部 L0 协调, 实证 DAGValidator cycle detection | TMO-03 + TMO-04 集成测试 |
| **#13 c (Master RLS)** | task_metadata 表 100% RLS 必携 | TMO-07 集成测试 + SQL DDL `task_metadata RLS POLICY` |
| **#13 d (Master 100% RLS / Transaction 100% audit / Work 100% retention)** | task card 状态 = Work (短 TTL), checkpoint history = Transaction (append-only), metadata = Master (SCD Type 2) | TMO-01..TMO-07 单元 + 集成 |
| **#4 (token-OLU)** | TMO 7 子项 ~2.5M tokens 总预算 (本文件 7 项估) | TokenTelemetry (C-09) 计量 |
| **#19 (Python 化)** | TMO 7 节点走 `scripts/automation/task_ops.py` 基类 | 实装路径 |
| **#20 (子代理 dispatch 必先 brief)** | SA-10 task-orchestrator dispatch 必先 `automation/dispatcher.py brief(...)` | TMO-01 集成测试 |
| **#22 (调试控制台不污染 main)** | task_ops.py 跑后 cargo check 0 err (per 守门 #1 v22) | 2.2 阶段 1 |
| **#23 (AI 修改 mock)** | TMO 调试走 ai_edit_mock.py, 不开 OpenAI API | 守门 #1 v23 |

---

## 3. 已知缺口 (Known Gaps, per 缺标比错标)

| # | 缺口 | 影响 | 计划补法 |
|---|---|---|---|
| **G-TMO-01** | TMO 7 子项实装未启动 (本 v0.1 报告是计划阶段) | 9/4 19:15 JST 用户发令功能未落地 | 7 子项按 token 预算排序推进, 优先 TMO-01 (merge) + TMO-03 (DAG validator, 守门 #13 a 实证) |
| **G-TMO-02** | SA-10 task-orchestrator stub 缺失 (per 03 §3.5 SA-10) | M-N1 合并后无新 sub-agent type 接 stash_state | TMO-01 子项 同步落档 SA-10 (per 03 §3.5 草稿) |
| **G-TMO-03** | FastAPI 8080 console_server.py 现有端点 + `/api/tmo/*` 8 端点 冲突校验未做 | 实装可能影响现有调试控制台 | TMO-08 同步子项: 检查 console_server.py 现有路由 + 加 namespace `/api/tmo/*` 不冲突 |
| **G-TMO-04** | `task_metadata` 表 DDL 缺 (per 守门 #13 c Master RLS) | TMO-07 元数据更新落库无 schema | TMO-07 集成测试 同步写 DDL `CREATE TABLE task_metadata ...` + RLS POLICY |
| **G-TMO-05** | LangGraph SDK 0.2.x interrupt_response API alpha (per 03 §9) | TMO 跨节点 interrupt 落地不确定 | TMO-08 同步子项: 实装前先 `uv add langgraph@latest` + `pip show langgraph` 确认 实际版本 + API 兼容性 |
| **G-TMO-06** | 守门 #13 a 强约束派生实证缺口 (L1↔L1 禁止通信 → TMO 全部 L0 协调) | DAGValidator 实证待实装 | TMO-03 集成测试 跑通 cycle detection O(V+E) |
| **G-TMO-07** | 现有 dispatcher.py / console_server.py 过渡期 (per 02 §9.1) | TMO 实装跟 worker subagent 系统并存期间 接口冲突 | 实装阶段 跨系统接口用 namespace 隔离 (`/api/tmo/*` vs `/api/top-agent/*` vs `/api/sub-agent/*`) |
| **G-TMO-08** | 5 域 Lead 真人未到位 (per 守门 #3) | TMO 跨域操作代签权归 Mavis 临时代签, 真人到位后追溯签字 | 守门 #3 v2 派生规: 真人到位后追溯签字, 不沿用代签决策 |
| **G-TMO-09** | PostgreSQL checkpointer Tier 3 未实装 (per 03 §9) | TMO 在 v0.1 SQLite 跑 (Tier 2), 多 tenant 时延后 | 后续 v0.3 阶段, 跟 R-05 push 反転 + 5 域 Lead 真人同步 |

---

## 4. 子代理失败接手清单 (Subagent Failure Takeover)

per 守门 #9 (子代理 status ≠ 实际成功) + #20 (子代理 dispatch 必先 brief):

- 本 phase 7 子项均由 **Mavis root 亲手执行** (子代理 RPC 不可靠实证 per 守门 #9), 不派 worker 子代理
- 若后续 TMO-01..TMO-07 任一子项跨 session 续做, 必须先 `automation/dispatcher.py brief(...)` 落档 `docs/briefs/tmo-XX.md` (per 守门 #20), brief 必含:
  1. 子项 ID (TMO-01..TMO-07)
  2. 节点 ID (M-N1..M-N7)
  3. 依赖 (前置子项 / Python 基类 / 端点)
  4. 守门合规检查清单 (per §2.3 12 项)
  5. 已知缺口 (per §3 9 项 跟本子项相关部分)
- 子代理 RPC 失败实证 (per 守门 #9, 10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded) → 续做必 `git log -p --follow scripts/automation/task_ops.py` 验证实际 commit 在 main 链上

---

## 5. 守门规则 (15 项 Gate Rules)

per AGENTS.md §4 守门硬约束 (13 main + 24 派生规 = 37 项) 跟 TMO 相关:

| # | 守门 | 派生约束 | 验证位置 |
|---|---|---|---|
| 1 | **#1 R-05** | 实装不 push (per 推 origin 反转拍板 9/3 11:07) | 实装阶段 commit 不推 |
| 2 | **#5 env var 安全** | task_ops.py 不打印 env value | console_server.py 现有 守门 |
| 3 | **#6 PowerShell only** | task_ops.py subprocess 走 PowerShell, 不走 bash | 守门 #1 v24 |
| 4 | **#9 v3 subprocess 走 console_server** | TMO UI 操作走 Next.js API → FastAPI 8080 → subprocess | 实装阶段 集成测试 |
| 5 | **#10 代签规则** | Mavis 临时代签 Ulysses, author=Ulysses, 修订人=`Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` | 修订历史 author / 修订人 |
| 6 | **#12 AI 協作文档治理** | 禁回溯叙事, BAS 引用 git 实证, 缺标比错标 | 修订历史 + 缺口清单 |
| 7 | **#13 a L1↔L1 禁止通信** | TMO 7 节点全部 L0 协调, 实证 DAGValidator cycle detection | TMO-03 集成 |
| 8 | **#13 c Master RLS** | task_metadata 表 100% RLS 必携 | TMO-07 集成 + DDL |
| 9 | **#13 d Master 100% RLS / Transaction 100% audit / Work 100% retention** | task card 状态 = Work, checkpoint = Transaction, metadata = Master (SCD Type 2) | TMO-01..TMO-07 |
| 10 | **#4 token-OLU** | TMO 7 子项 ~2.5M tokens 总预算 (本文件 7 项估) | TokenTelemetry |
| 11 | **#19 Python 化** | TMO 走 `scripts/automation/task_ops.py` 基类 | 实装路径 |
| 12 | **#20 子代理 dispatch 必先 brief** | 跨 session 续做必先 brief 落档 | 4 子代理接手清单 |
| 13 | **#22 调试控制台不污染 main** | task_ops.py 跑后 cargo check 0 err | 守门 #1 v22 |
| 14 | **#23 AI 修改 mock** | TMO 调试走 ai_edit_mock.py, 不开 OpenAI | 守门 #1 v23 |
| 15 | **#24 调试控制台走 subprocess** | Next.js → FastAPI 8080 → subprocess | 守门 #1 v24 |

**累积规 (per 守门 #1 派生 v19+)**: 后续 TMO-01..TMO-07 任一子项必先判定自动化档 ([P]/[M]/[S]), 命中 ≥ 2 维 (R/V/S/A) 强制走 `scripts/automation/<purpose>.py` 落地; commit message 含脚本相对路径; 子代理 dispatch 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`; [P] 子项 docs 同步必更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md`.

---

## 6. 签字栏 (Signatures)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-04 | 🟡 Draft v0.1; TMO 7 子项实装计划落档, 文档 v0.2 配套 (per ADR-0046) |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手终审通过 (per 2026-09-04 19:15 JST 用户发令 + ask_d076c26d3fbf599eec1c32fd 拍板 3 问: 范围=完整 7 节点 + 文档策略=原地升版 + 实装阶段=文档+commit 一并落); 7 段结构 + 7 子项估 + 12 守门合规 + 9 已知缺口 + 15 守门规则 + 5 签字栏 + v0.1 修订历史 落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 7. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：TMO 7 子项实装 phase 计划 (TMO-01..TMO-07, 节点 M-N1..M-N7 + 组件 C-16..C-22 + 25 新 module M-19..M-25) + 7 段结构 (目的/任务矩阵/验证摘要/已知缺口/子代理接手/守门规则/签字/修订) + 7 子项估 ~2.5M tokens (跟 AGENTS §7 #8 ~3.0M 兼容, 留 0.5M 给 9 SA 类型 stub) + 12 守门合规预期 + 9 已知缺口 (G-TMO-01..G-TMO-09) + 15 守门规则 + 5 签字栏 (Mavis 接手代签) | 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b" (per ask_d076c26d3fbf599eec1c32fd 拍板 (1) 范围=完整 7 节点全覆盖 (2) 文档策略=原地升版 v0.1 → v0.2 (3) 实装阶段=文档+commit 一并落), 跟 3 份主文档 v0.2 + ADR-0046 同步落档, ~0.05M token 估 |

---

## 8. 引用文档 (References)

- [01-requirements.md v0.2](../architecture/2026-09-03-langgraph/01-requirements.md) — UC-09..UC-13 + F-19..F-25 + NFR-TMO-01..05 + S-06
- [02-basic-design.md v0.2](../architecture/2026-09-03-langgraph/02-basic-design.md) — §2.6 TMO 全节 + 7 组件 C-16..C-22 + 7 协议 + 5 Reducer + 8 API 端点
- [03-detailed-design.md v0.2](../architecture/2026-09-03-langgraph/03-detailed-design.md) — task_ops/ 模块 + M-19..M-25 + §3.2.1.1 7 节点 Python 実装 + SA-10 + superseded 终态 + UT-20..UT-26 / IT-10..IT-12 / E2E-09..E2E-13
- [ADR-0046 LangGraph TMO](../architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — TMO 决策记录
- [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 13 main + 24 派生规 = 37 项硬约束
- [AGENTS.md §7 #8 Star LangGraph 統合アーキテクチャ](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — ~3.0M token 预算
- [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens
- [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) — agent 交互 Python 化 (守门 #19)
- [scripts/automation/dispatcher.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/dispatcher.py) — 现有 sub-agent dispatch 基础
- [scripts/automation/console_server.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/console_server.py) — 现有 FastAPI 8080 调试控制台 (扩展 `/api/tmo/*` 端点)
- [scripts/automation/ai_edit_mock.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/ai_edit_mock.py) — AI 修改 mock (守门 #23)
- [docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md](../kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) — DB W/T/M 横展開 (守门 #13)
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt / Command
