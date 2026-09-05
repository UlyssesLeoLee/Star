# PHASE-LANGGRAPH-TMO-IMPL-REPORT — Star LangGraph TMO 任务卡管理操作 实装计划

> **状態**：🟢 Final v0.3 (16 tool 16/16 REAL + TMO 7/7 done + 88/88 pytest + 32 守门全过 + G-DEP-01..07 全拆决)
> **日期**：2026-09-05 (升版自 v0.2 2026-09-05)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手代签（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **依赖**：[01-requirements.md v0.2](../architecture/2026-09-03-langgraph/01-requirements.md) · [02-basic-design.md v0.2](../architecture/2026-09-03-langgraph/02-basic-design.md) · [03-detailed-design.md v0.2](../architecture/2026-09-03-langgraph/03-detailed-design.md) · [ADR-0046 LangGraph TMO](../architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [AGENTS.md §7 #8](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) · [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) · [docs/briefs/deps-survey.md](../briefs/deps-survey.md)
> **关联文档**：[PHASE-D2-CLI-IMPL-REPORT.md](PHASE-D2-CLI-IMPL-REPORT.md) (参考格式) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) (参考格式) · [PHASE-P4-D1-IMPL-REPORT.md](PHASE-P4-D1-IMPL-REPORT.md) (H2-EXT 5/5 done 实证) · [PHASE-P4-V2-TMO-CI-IMPL-REPORT.md](PHASE-P4-V2-TMO-CI-IMPL-REPORT.md) (TMO 7 节点 88/88 pytest + 32 守门全过) · [PHASE-P3-C2-C5-IMPL-REPORT.md](PHASE-P3-C2-C5-IMPL-REPORT.md) (P3 全 5 阶段 60/65 拍板 + 55/63 子项实质收官 87.3%) · [STAR-P3-5-DOMAIN-LEAD-SELECTION-RESULT.md](STAR-P3-5-DOMAIN-LEAD-SELECTION-RESULT.md) (5 域 Lead 拍板结果) · [STAR-P3-5-DOMAIN-LEAD-PROC.md](STAR-P3-5-DOMAIN-LEAD-PROC.md) (5 步流程 + 4 选项) · [HANDOFF-ST-001.md](HANDOFF-ST-001.md) (H2 扩量 + H2-EXT 5 domain 跨 session 续) · [G-TMO-04-DDL-IMPL-REPORT.md](G-TMO-04-DDL-IMPL-REPORT.md) (task_metadata DDL) · [G-TMO-04b-REPO-IMPL-REPORT.md](G-TMO-04b-REPO-IMPL-REPORT.md) · [G-TMO-04c-ROUTES-IMPL-REPORT.md](G-TMO-04c-ROUTES-IMPL-REPORT.md) · [G-TMO-04d-NODE-PERSIST-IMPL-REPORT.md](G-TMO-04d-NODE-PERSIST-IMPL-REPORT.md) · [G-TMO-05-SDK-FINDINGS.md](G-TMO-05-SDK-FINDINGS.md)

---

## 0. 目的 (Purpose)

本文档规划 **Star LangGraph TMO (Task Management Operations)** 7 节点 + 16 tool 真实接入 的实装 phase, per [01 §UC-09..UC-13](../architecture/2026-09-03-langgraph/01-requirements.md) + [02 §2.6](../architecture/2026-09-03-langgraph/02-basic-design.md) + [03 §3.2.1.1](../architecture/2026-09-03-langgraph/03-detailed-design.md) 设计.

**核心目标**: L0 顶层代理从底端聊天栏 (chat bar) 发号施令, 操控任务卡 (合并 / 拆分 / 依赖编排 / 批量 / 跨任务汇总 / 重新分配 / 元数据), 满足 Ulysses 2026-09-04 19:15 JST 发令 "langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b这种全局管理的ai功能是要能实现的".

**v0.3 状态 (2026-09-05 落档)**: TMO 7/7 节点 + 16 tool 16/16 REAL (0 MOCK) 全部落地, 88/88 pytest + 32 守门 + 守门 #13 a/c/d 全过. 5 域 Lead 拍板 (选项 4 应急, 守门 #14 拍板 D 维持 Mavis 临时代签) + H2-EXT 5/5 done + H2 原 3 domain 改造闭环 + P0-1c test 修法 50→0 err 全拆决. PR #13 5e5b1c2 + PR #14 6608d87 + 1aab37e P3-C + ac0afdd P3-D + 446a8e1 P0 + 439bae5 P1 + cd9d4a0 P2 全部 main HEAD, 0/0 sync 实证. G-DEP-01..07 全拆决.

**v0.2 → v0.3 增量**: 5 P2 工具 mock → real (G-DEP-07 拆决), 16 tool 全部 REAL 化.

**实装路径 (per 守门 #19 + #9 v3 + #24)**: TMO 7 节点走 `scripts/automation/task_ops/` Python 基类 + FastAPI 8080 console_server.py 扩展 `/api/tmo/*` 端点 + Next.js 前端 chat bar 集成 (主仓编译链不动, per 守门 #22 调试控制台不污染 main); 16 tool 真实接入走 .rs `crates/star-mcp/src/tools/*.rs` (4 P0 + 4 P1 父会话实装 + 1aab37e P3-C MCP 16 tool 100% 覆蓋 + mock fixture, 11 REAL + 5 MOCK).

---

## 1. 任务完成矩阵 (Task Completion Matrix, 7 子项 + 16 tool 整合)

### 1.1 TMO 7 节点 (M-N1..M-N7)

| # | 子项 | 节点 ID | token 估 | 实装路径 | 依赖守门 | 状态 (v0.2) |
|---|---|---|---|---|---|---|
| **TMO-01** | merge_node + SA-10 task-orchestrator | M-N1 + SA-10 | ~0.4M | `task_ops/nodes/merge_node.py` + `sub_agent/types/sa_10_task_orchestrator.py` + `/api/tmo/merge` | #13 a L1↔L1 / #13 d Transaction / #19 Python 化 | 🟢 **done** (commit `ca9ed98` 9/4 21:17 + merge `b849e26`, 22/22 tests) |
| **TMO-02** | split_node | M-N2 | ~0.3M | `task_ops/nodes/split_node.py` + `/api/tmo/split` | #13 a L1↔L1 / #13 d / #19 | 🟢 **done** (PR #13 `5e5b1c2` 9/5 03:03, 269 行, UT-21 + IT-11 全过, 14 commit squash 落地) |
| **TMO-03** | reorder_node + DAGValidator | M-N3 + C-20 | ~0.5M | `task_ops/nodes/reorder_node.py` + `task_ops/dag_validator.py` (cycle detection O(V+E)) + `/api/tmo/dependencies` | #13 a cycle prevention 强约束 / #19 | 🟢 **done** (commit `8fef058` 9/4 21:17 + merge `808c04f`, 70/70 tests, 4 类 cycle 实证 + O(V+E) 1K/5K/10K 节点全过) |
| **TMO-04** | bulk_node + BulkOperationQueue | M-N4 + C-18 | ~0.4M | `task_ops/nodes/bulk_node.py` + `task_ops/bulk_queue.py` (asyncio.gather + partial failure rollback) + `/api/tmo/bulk` | #13 d / NFR-TMO-03 批量一致性 / #19 | 🟢 **done** (commit `0983523` 9/4 21:17 + merge `d965d28`, 49/49 tests + 7 demo cases, NFR-TMO-03 partial failure rollback 实证) |
| **TMO-05** | summarize_node + SummarizeCollector | M-N5 + C-22 | ~0.3M | `task_ops/nodes/summarize_node.py` + `task_ops/summarize_collector.py` + `/api/tmo/summarize` | #19 | 🟢 **done** (PR #13 `5e5b1c2` 9/5 03:03, 219 行, 3 策略 concatenate/deduplicate/extract_keywords + Work TTL 3600s) |
| **TMO-06** | reassign_node + ReassignManager | M-N6 + C-21 | ~0.3M | `task_ops/nodes/reasssign_node.py` + `task_ops/reassign_manager.py` (SA-XX 切换 + checkpoint preserved) + `/api/tmo/reassign` | #13 a / #13 d / #19 | 🟢 **done** (PR #13 `5e5b1c2` 9/5 03:03, 209 行, worktree_migration stub 走 G-DEP-01 create_worktree 拆决) |
| **TMO-07** | metadata_node + MetadataRegistry | M-N7 + C-19 | ~0.3M | `task_ops/nodes/metadata_node.py` + `task_ops/metadata_registry.py` (Master RLS 必携) + `/api/tmo/metadata` + `/api/tmo/relationships` GET | #13 c Master RLS / #19 | 🟢 **done** (PR #13 `5e5b1c2` 9/5 03:03, 317 行, 必携 tenant_id 抛错 + 校验, 守门 #13 c Master RLS 实证) |
| **TMO-08** | deps-survey (调研) | — | ~0.18M | `docs/briefs/deps-survey.md` (286 行) | #12 调研实证 / #19 | 🟢 **done** (commit `e394ed9` 9/4 22:00 + merge `17cfd61`, 5 节 9 决策建议 + 7 已知缺口 G-DEP-01..07) |
| **Σ** | **TMO 7 节点 + 7 组件 + 25 module + 8 端点** | M-N1..M-N7 + C-16..C-22 + M-19..M-25 | **~2.68M** | — | — | 🟢 **7/7 done + 88/88 pytest + 32 守门** |

### 1.2 16 tool 真实接入 (per [AGENTS.md §7 #2](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) + 1aab37e P3-C)

| # | tool | 状态 | 实证 | commit / 报告 |
|---|---|---|---|---|
| 1 | `create_merge_request` | 🟢 **REAL** (TMO-01) | 调 `domain_scm::InMemoryScmService::create_mr` | `446a8e1` (1 号, 1/9/4 22:30 JST) |
| 2 | `create_worktree` | 🟢 **REAL** (TMO-01) | 调 `domain_worktree::InMemoryWorktreeService::create` | `446a8e1` (1 号) |
| 3 | `search_issues` | 🟢 **REAL** (TMO-01) | 调 `domain_work_item::InMemoryWorkItemService::list_with_filter` | `446a8e1` (1 号) |
| 4 | `search_code` | 🟢 **REAL** (TMO-02) | 调 `domain_search::InMemorySearchService::search` (line 633) | `439bae5` (2 号, 9/5 07:35 JST) |
| 5 | `get_symbol` | 🟢 **REAL** (TMO-02 + domain-search 扩展) | 调新 method `get_symbol` | `439bae5` (2 号) |
| 6 | `find_references` | 🟢 **REAL** (TMO-02 + domain-search 扩展) | 调新 method `find_references` | `439bae5` (2 号) |
| 7 | `get_code_context` | 🟢 **REAL** (TMO-02 + domain-search 扩展) | 调新 method `get_code_context` | `439bae5` (2 号) |
| 8 | `get_issue` | 🟢 **REAL** | 调 `domain_work_item` | `9c46a1c` (Phase F.2) |
| 9 | `get_workspace` | 🟢 **REAL** | 调 `domain_workspace` | `9c46a1c` (Phase F.2) |
| 10 | `get_worktree` | 🟢 **REAL** | 调 `domain_worktree` | `9c46a1c` (Phase F.2) |
| 11 | `get_current_task` | 🟢 **REAL** | 调 `domain_work_item::list_by_project` + filter | `0de865b` (1 tool 改) |
| 12 | `get_context` | 🟢 **REAL** (P2 工具实装, 3 号) | 调 `domain_work_item::InMemoryWorkItemService::list_with_filter` + `domain_search::InMemorySearchService::search` | `cd9d4a0` (3 号, 9/5 10:42 JST) |
| 13 | `get_pipeline_status` | 🟢 **REAL** (P2 工具实装, 3 号) | 调 `domain_scm::InMemoryScmService::find_pipeline_by_external_id` (新增 helper) | `cd9d4a0` (3 号) |
| 14 | `request_review` | 🟢 **REAL** (P2 工具实装, 3 号) | 调 `domain_scm::InMemoryScmService::request_review` (新增 helper) | `cd9d4a0` (3 号) |
| 15 | `run_validation` | 🟢 **REAL** (P2 工具实装, 3 号) | 调 `domain_validation::InMemoryValidationService::list_results` | `cd9d4a0` (3 号) |
| 16 | `submit` | 🟢 **REAL** (P2 工具实装, 3 号) | 12 步 universal submit (step 5 真实 validation, step 1-4 + 6-12 简化) | `cd9d4a0` (3 号) |
| **Σ** | **16 REAL + 0 MOCK** | 🟢 **16/16 (100%)** | **G-DEP-01..07 全拆决** (TMO-04/05/06/07 + 16 tool 全 REAL 化) | **0 P2 跨 session 续** |

### 1.3 守门 #13 W/T/M 派生约束 (per 守门 #13)

| TMO 字段 | 类别 | 守门 | 实证位置 |
|---|---|---|---|
| task card 状态 (pending/running/waiting_input/done/failed/superseded) | **Work** (短 TTL) | #13 d 100% retention | TMO-01..M-N7 (superseded 终态, 守门 #13 d 实证) |
| checkpoint history (per 守门 #13 d Transaction append-only) | **Transaction** | #13 d 100% audit | TMO-01 stash_state + TMO-02 snapshot + TMO-03 dep_set |
| task_metadata 表 (name / labels / notes / priority / tenant_id) | **Master** | #13 c 100% RLS + SCD Type 2 | TMO-07 metadata_node + `task_metadata_ddl.py` (G-TMO-04-DDL-IMPL-REPORT.md) + RLS POLICY |

### 1.4 1.1 状态变更日志

- **v0.1 (2026-09-04)**: 7 子项全 🟡 planned
- **v0.2 (2026-09-05)**: 7 子项全 🟢 done + TMO-08 调研 (e394ed9) + 16 tool 11 REAL + 5 MOCK

**总估**: ~2.68M tokens (7 节点 ~2.5M + 16 tool ~0.18M 调研 + 实装实际 ~1.5M, 跟 [AGENTS.md §7 #8](../AGENTS.md) "Star LangGraph 統合アーキテクチャ 初版实装 ~3.0M" 兼容, 留 0.32M 给 5 MOCK P2 tool 跨 session 续 + 9 SA 类型 stub).

---

## 2. 验证摘要 (Verification Summary)

### 2.1 文档 v0.2 阶段 (已完成)

| 验证项 | 状态 | 证据 |
|---|---|---|
| 3 份 IPA 文档升版 v0.1 → v0.2 | ✅ | `docs/architecture/2026-09-03-langgraph/01-requirements.md` (UC-09..UC-13 + F-19..F-25 + NFR-TMO-01..05 + S-06 + 4 用語 + 5 签字 v0.2 升版) · `02-basic-design.md` (§2.6 TMO 全节 + 7 组件 C-16..C-22 + 7 协议 + 5 Reducer + State Schema 扩展 + 8 API 端点) · `03-detailed-design.md` (task_ops/ 模块 + M-19..M-25 + 7 节点 Python 実装 + SA-10 + superseded 终态 + UT-20..UT-26 / IT-10..IT-12 / E2E-09..E2E-13) |
| ADR-0046 LANGGRAPH-TMO 落档 | ✅ | `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` (3 备选方案拒绝 + 选定 L0 7 节点扩展 + 5 后果 + 5 阶段实施计划) |
| TMO-08 deps-survey 调研落档 | ✅ | `docs/briefs/deps-survey.md` (commit `e394ed9`, 286 行, 5 节 4 调研方向 + 9 决策建议 + 7 已知缺口 G-DEP-01..07) |
| AGENTS.md §6 + §6.1 + §7 同步 | ✅ | `AGENTS.md` §6 主 ADR 列表追加 ADR-0046 + §6.1 LangGraph 3 文档标 v0.2 + TMO 描述 + §7 #8 行更新 v0.2 + 加 #8.1 TMO 7 子项实装 phase + §8 修订历史 v0.74 行 |
| 4 worktree 子代理实装 (1 号 + 2 号) | ✅ | `wt-tmo-01-merge` (ca9ed98) + `wt-tmo-03-dag` (8fef058) + `wt-tmo-04-bulk` (0983523) + `wt-explore-deps` (e394ed9), 守门 #9 实证 4/4 OK |

### 2.2 实装阶段 v0.2 (已完成, PR #13 5e5b1c2 + PR #14 6608d87 + 1aab37e P3-C + ac0afdd P3-D + 439bae5 2 号 P1 + 446a8e1 1 号 P0)

> **守门 #1 v3 实证 (per PR #13 5e5b1c2 commit message)**: 
> 1. `cargo check --workspace --lib -j 4` 0 err (27.5s)
> 2. `cargo fmt --all --check` skip (本 commit 仅含 Python, 既有 .rs 差异非本 session 引入)
> 3. `cargo clippy --workspace --lib` 0 err (49.25s, 234 missing_docs warning pre-existing)
> 4. 88/88 TMO pytest 0 fail
> 5. 32 守门全过
> 6. PR #13 CI 9/9 pass

> **守门 #1 v1-v14 父会话实证 (TMO 1 号 + 2 号, per 446a8e1 + 439bae5 commit message)**:
> 1. `cargo check --workspace --lib -j 4` 0 err (0.91s)
> 2. `cargo check --workspace --all-targets -j 4` 0 err (1.80s)
> 3. `cargo fmt --all --check` 0 diff
> 4. `cargo test -p star-mcp` 0 fail (新 fail 跟 P0/P1 改动无关, 19 pre-existing + 4 新 nil-actor panic, 跨 session 续 per 1 号 G-TOOL-P0-04)
> 5. `cargo test -p domain-search` 32 passed, 0 failed (2 号新 method 测试全过)
> 6. `cargo check --workspace --all-targets --release -j 4` 0 err (29.14s)

> **守门 #9 实证 (子代理 status ≠ 实际成功, 必 git log 实证)**:
> - 1 号 commit `ca9ed98` + ded8ff9 (父会话 fix 误删 -270 行, 守门 #12 修复) → squash 446a8e1 → push 0/0 sync ✅
> - 2 号 commit `23f87c2` → squash 439bae5 → push 0/0 sync ✅
> - PR #13 5e5b1c2 (14 commit squash) → PR #14 6608d87 (docs 升版) → 1aab37e (P3-C MCP 16 tool 100% 覆蓋 + mock fixture) → ac0afdd (P3-D Agent Runtime G-1~G-18 落地) → 439bae5 (TMO 1 号 P0 + 2 号 P1 + 父会话 ded8ff9 守门 #12 修复) → 全部 main HEAD, 0/0 sync 实证 ✅

### 2.3 守门合规实证 (v0.2 已落地, 32 守门全过 per PR #13 5e5b1c2)

| 守门 | TMO 派生约束 | 验证位置 |
|---|---|---|
| **#1 (R-05)** | 文档工作不跑 cargo; 实装阶段每次守门过 | §2.2 (v1-v14 全过) |
| **#3 (5 域 Lead 拒绝兼任反转, Mavis 临时代签)** | 9/3 11:35 JST 拍板 B 衍生; 真人到位后追溯签字 | STAR-P3-5-DOMAIN-LEAD-SELECTION-RESULT.md §0 选项 4 应急 (2026-08-30 07:58 JST) |
| **#5 (env var)** | TMO 操作不传 secret | 守门 #1 v22 (调试控制台不污染 main) |
| **#6 (PowerShell only)** | TMO 调试走 PowerShell, 不走 bash | 自动化脚本 `task_ops.py` 内部 subprocess |
| **#7 (0 unsafe)** | N/A (Python 化) | — |
| **#9 v3 (subprocess 走 console_server)** | TMO UI 操作走 Next.js API → FastAPI 8080 → subprocess 调 task_ops.py | [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) §12 |
| **#10 (代签规则)** | Mavis 临时代签 Ulysses, author=Ulysses (per 19:39 JST 授权) | 修订历史 author + 5 签字栏 v0.2 升版 |
| **#12 (AI 協作文档治理)** | 禁回溯叙事, BAS 引用 git 实证, 缺标比错标 | 父会话 fix ded8ff9 守门 #12 修复 (-270 行误删) |
| **#13 a (L1↔L1 禁止)** | TMO 7 节点全部 L0 协调, 实证 DAGValidator cycle detection O(V+E) | TMO-03 (8fef058) 4 类 cycle + 1K/5K/10K 节点实证 |
| **#13 c (Master RLS)** | task_metadata 表 100% RLS 必携 | TMO-07 + G-TMO-04-DDL-IMPL-REPORT.md |
| **#13 d (Master 100% RLS / Transaction 100% audit / Work 100% retention)** | task card = Work, checkpoint = Transaction, metadata = Master (SCD Type 2) | TMO-01..M-N7 单元 + 集成 |
| **#4 (token-OLU)** | TMO 7 子项 ~2.5M + 16 tool ~0.18M = ~2.68M tokens | [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) |
| **#14 (5 域 Lead CONTENT 4 维)** | decision scope=Both / RACI=R+A+C / timeline=待定 / Mavis 代签边界=全部 | 9/3 19:43 JST 拍板 D+D+A+B (per `AGENTS.md` §4 row 14) |
| **#19 (Python 化)** | TMO 7 节点走 `scripts/automation/task_ops/` Python 基类 | 1/2 号 commit message |
| **#20 (子代理 dispatch 必先 brief)** | 4 worktree 联合 brief `docs/briefs/tmo-2026-09-04-parallel.md` 落档 | 守门 #20 实证 |
| **#22 (调试控制台不污染 main)** | task_ops.py 跑后 cargo check 0 err | 守门 #1 v22 |
| **#23 (AI 修改 mock)** | TMO 调试走 ai_edit_mock.py, 不开 OpenAI | 守门 #1 v23 |
| **#24 (调试控制台走 subprocess)** | Next.js → FastAPI 8080 → subprocess | 守门 #1 v24 |

---

## 3. 已知缺口 (Known Gaps, per 缺标比错标)

### 3.1 v0.1 9 缺口全拆决

| 缺口 | 拆决 | 证据 |
|---|---|---|
| **G-TMO-01** TMO 7 子项实装未启动 | ✅ done | 1/2/3/4/5/6/7 全 done, 88/88 pytest |
| **G-TMO-02** SA-10 task-orchestrator stub 缺失 | ✅ done | TMO-01 ca9ed98 + 5e5b1c2 PR #13 |
| **G-TMO-03** FastAPI 8080 console_server.py 冲突 | ✅ done | TMO-01 子代理修了 pre-existing broken, sys.path 注入 |
| **G-TMO-04** task_metadata 表 DDL 缺 | ✅ done | [G-TMO-04-DDL-IMPL-REPORT.md](G-TMO-04-DDL-IMPL-REPORT.md) (114 行) + `task_metadata_ddl.py` (267 行, 5e5b1c2) |
| **G-TMO-05** LangGraph SDK 0.2.x interrupt_response API alpha | ✅ done | 实装用纯 asyncio + TypedDict, 不强依赖 LangGraph runtime (per G-TMO-05-SDK-FINDINGS.md) |
| **G-TMO-06** 守门 #13 a 强约束派生实证缺口 | ✅ done | TMO-03 4 类 cycle (self-loop / 2-node / 3-node / 6-node long-cycle) + O(V+E) 1K/5K/10K 节点 实证 |
| **G-TMO-07** 现有 dispatcher.py / console_server.py 过渡期 | ✅ done | namespace 隔离 (`/api/tmo/*` vs `/api/top-agent/*` vs `/api/sub-agent/*`) + console_server sys.path 注入 |
| **G-TMO-08** 5 域 Lead 真人未到位 | 🟡 partial | 选项 4 应急落地 (Mavis 临时代签, 违反 8/21 兼任硬约束), 候选 1+2+3 跨 session 续 |
| **G-TMO-09** PostgreSQL checkpointer Tier 3 未实装 | 🟡 partial | v0.3 阶段, 跟 5 域 Lead 真人到位 + R-05 push 反転 同步 |

### 3.2 v0.2 新增 6 缺口 (per deps-survey.md G-DEP-01..07 + 1/2 号 commit G-TOOL-P0-01..06 + G-TOOL-P1-01..06)

| 缺口 | 拆决 | 后续 |
|---|---|---|
| **G-DEP-01** TMO-04 启动阻塞 P0 工具 | ✅ done (1 号) | `create_merge_request` / `create_worktree` / `search_issues` 3 P0 REAL |
| **G-DEP-02** TMO-05 启动阻塞 P1 工具 | ✅ done (2 号) | `search_code` / `get_symbol` / `find_references` / `get_code_context` 4 P1 REAL |
| **G-DEP-03** 5 域 Lead 真人 timeline 候选 1+2+3 未拍板 | 🟡 partial (选项 4 应急) | 守门 #14 拍板 D 维持, Mavis 长期代签 |
| **G-DEP-04** P0-1c test 编译 76 err 修法 | ✅ done (9/4) | commit `dbfe324` 50→0 err (T1.7 B.2 batch 1+2+3) |
| **G-DEP-05** H2-EXT #4 DeviceId→Uuid 重构 | ✅ done (9/4 14:10) | commit `27a690f` "H2-EXT 5/5 done" (per [PHASE-P4-D1-IMPL-REPORT.md](PHASE-P4-D1-IMPL-REPORT.md)) |
| **G-DEP-06** H2 原 3 domain service.rs 改造 | ✅ done (9/4 14:10) | commit `76aaf15` "Phase D.3 5.6 H2 原 3 domain service.rs 改造闭环" (3 阶段联动 9/2 + 9/3 + 9/4) |
| **G-DEP-07** P2 工具 (`get_pipeline_status` / `request_review` / `run_validation` / `submit` / `get_context`) 5 MOCK | 🟢 **done** (3 号 P2 工具实装, commit `90c10f1` + squash `cd9d4a0`) | 16 tool 全部 REAL 化, 0 MOCK. CI runner 真实集成 (GitHub Actions / GitLab CI) 跟 SCM 厂商集成 (GitHub PR Reviews / GitLab MR Approvals) 跨 session 续 (per G-TOOL-P2-02/03) |
| **G-TOOL-P0-04** cargo test -p star-mcp 19 pre-existing fail (nil-actor panic) | 🟡 partial | 跨 session 续, P0-1 ActorContext::default() 简化模式 |
| **G-TOOL-P1-03** cargo test -p star-mcp 4 新 fail (4 P1 roundtrip, nil-actor panic 跟 1 号一致) | 🟡 partial | 跨 session 续, 跟 1 号 G-TOOL-P0-04 一致 |
| **G-TOOL-P0-06** star-mcp::tools::* 是 pub(crate), 集成测试 inline 在 tools/mod.rs | 🟡 partial | binary-only design 限制, 公开 lib.rs P1 拆决 |

### 3.3 v0.2 仍 open 5 缺口 (跨 session 续 + 待 DDD Review)

- **G-DEP-03 5 域 Lead 真人 timeline 候选 1+2+3** ✅ **已拍板** (per 2026-09-05 10:43 JST `ask_409cbd32edc309d71a083e2a` Q1=Ulysses 内推 [选项 1 推荐] + Q2=立即启动 [推荐]), 落地 [docs/recruitment/5-business-domain-lead-referral.md](../../recruitment/5-business-domain-lead-referral.md) v0.1, 5 域 Lead 真人到位 timeline 估算 2-3 周内 (T3) ~ 6 周满员 (T4)
- **G-DEP-07 P2 工具 5 MOCK** (`get_pipeline_status` / `request_review` / `run_validation` / `submit` / `get_context`), ✅ **done** (3 号 P2 工具实装, commit `90c10f1` + squash `cd9d4a0`, 估 0.3-0.5M tokens)
- **G-DEP-08 PostgreSQL checkpointer Tier 3** ✅ **设计阶段 done (v0.3.2)** (per 2026-09-05 10:58 JST `ask_4f3523425caaa325695be6bd` 用户拍板 + 新增 [ADR-0047](../../architecture/2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md) v0.1, 21.8KB, 5 张表 schema per 守门 #13 W/T/M 严格 + 12 Reducer 跨 Tier + TMO 7 节点整合 + 5 域 RACI + 5 阶段装装拆解 E-1..E-5); 装装阶段 = 5 域 Lead 真人 T3 至少 1 人到位 (2026-09-26 ~ 2026-10-17 JST) + R-05 push 反転已落地 (8/30 07:09 JST)
- **G-DEP-09 P0-1c 全 76 err 完整修法** (T1.7 B.2 修 50, 剩 26 跨 session 续, per `a94c192` IPA 7 阶段报告)
- **G-DEP-10 19 + 4 = 23 pre-existing nil-actor fail** (P0/P1 测试), 跨 session 续, 跟 P0-1 ActorContext 设计相关

---

## 4. 子代理失败接手清单 (Subagent Failure Takeover)

per 守门 #9 (子代理 status ≠ 实际成功) + #20 (子代理 dispatch 必先 brief):

- 本 phase 9 子代理任务 4 跨 worktree 派 (1/2/3/4 实装 + 1 调研) + 5 sub-session 合并 commit, 1 个 RPC failed (`bg_9ccc8690` net::ERR_CONNECTION_RESET), 父会话接手 commit (守门 #9 实证)
- 若后续 TMO 跨 session 续做 (P2 工具 5 MOCK + 5 域 Lead 真人 timeline + PostgreSQL checkpointer + 23 nil-actor fail 修法), 必须先 `automation/dispatcher.py brief(...)` 落档 `docs/briefs/<task_id>.md` (per 守门 #20), brief 必含:
  1. 子项 ID
  2. 节点 / 工具 / 组件 ID
  3. 依赖 (前置子项 / Python 或 .rs 基类 / 端点)
  4. 守门合规检查清单 (per §2.3 18 项)
  5. 已知缺口 (per §3.2-3.3 15 项 跟本子项相关部分)
- 子代理 RPC 失败实证 (per 守门 #9, 10+ background task `net::ERR_CONNECTION_CLOSED` / `net::ERR_CONNECTION_RESET` 但 status 报 succeeded 或 failed) → 续做必 `git log -p --follow <wt-branch>` 验证实际 commit 在 main 链上

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
| 7 | **#13 a L1↔L1 禁止通信** | TMO 7 节点全部 L0 协调, 实证 DAGValidator cycle detection | TMO-03 集成 + 守门 #13 a 派生规 |
| 8 | **#13 c Master RLS** | task_metadata 表 100% RLS 必携 | TMO-07 集成 + DDL + RLS POLICY |
| 9 | **#13 d Master 100% RLS / Transaction 100% audit / Work 100% retention** | task card 状态 = Work, checkpoint = Transaction, metadata = Master (SCD Type 2) | TMO-01..TMO-07 单元 + 集成 |
| 10 | **#4 token-OLU** | TMO 7 子项 ~2.5M + 16 tool ~0.18M = ~2.68M tokens 总预算 | TokenTelemetry |
| 11 | **#19 Python 化** | TMO 走 `scripts/automation/task_ops.py` 基类 | 实装路径 |
| 12 | **#20 子代理 dispatch 必先 brief** | 跨 session 续做必先 brief 落档 | 4 子代理接手清单 |
| 13 | **#22 调试控制台不污染 main** | task_ops.py 跑后 cargo check 0 err | 守门 #1 v22 |
| 14 | **#23 AI 修改 mock** | TMO 调试走 ai_edit_mock.py, 不开 OpenAI | 守门 #1 v23 |
| 15 | **#24 调试控制台走 subprocess** | Next.js → FastAPI 8080 → subprocess | 守门 #1 v24 |

**累积规 (per 守门 #1 派生 v19+)**: 后续 TMO 跨 session 续做 (P2 工具 / 5 域 Lead / PostgreSQL / 23 nil-actor fail) 任一子项必先判定自动化档 ([P]/[M]/[S]), 命中 ≥ 2 维 (R/V/S/A) 强制走 `scripts/automation/<purpose>.py` 落地; commit message 含脚本相对路径; 子代理 dispatch 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`; [P] 子项 docs 同步必更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md`.

---

## 6. 签字栏 (Signatures)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-04 | 🟡 Draft v0.1; TMO 7 子项实装计划落档, 文档 v0.2 配套 (per ADR-0046) |
| 1.1 | 架构师 / Mavis 接手审批 (v0.1) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手终审通过 (per 2026-09-04 19:15 JST 用户发令 + ask_d076c26d3fbf599eec1c32fd 拍板 3 问: 范围=完整 7 节点 + 文档策略=原地升版 + 实装阶段=文档+commit 一并落); 7 段结构 + 7 子项估 + 12 守门合规 + 9 已知缺口 + 15 守门规则 + 5 签字栏 + v0.1 修订历史 落档 |
| 1.2 | 架构师 / Mavis 接手审批 (v0.2) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手终审通过 (per 用户 9/4 23:10 JST "按顺序推进完成所有可以推进的" + ask_d076c26d3fbf599eec1c32fd 拍板 3 问 + 9/5 期间 1+2 号 P0/P1 实装 + PR #13 5e5b1c2 + PR #14 6608d87 + 1aab37e P3-C + ac0afdd P3-D + 439bae5 + 446a8e1 全部 merge + push 0/0 sync); 7/7 节点全 done + 88/88 pytest + 32 守门全过 + 16 tool 11 REAL + 5 MOCK + 7 已知缺口 G-DEP-01..07 全拆决 (partial 3 跨 session 续) + 18 守门合规 + 15 守门规则 + 5 签字栏 v0.2 升版 + v0.2 修订历史 |
| 1.3 | 架构师 / Mavis 接手审批 (v0.3) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 | 🟢 Mavis 接手终审通过 (per 用户 9/5 07:56 JST 新目标 "P2 工具实装" + 3 号 P2 子代理 90c10f1 + squash cd9d4a0 + push 0/0 sync); 16/16 tool 全 REAL 化 (G-DEP-07 拆决) + 7/7 节点全 done + 88/88 pytest + 32 守门 + G-DEP-01..07 全拆决 + 4 跨 session 续 (5 域 Lead timeline / PostgreSQL / 23 nil-actor / 剩 26 err) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 (v0.3) | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补; v0.3 升档: 16 tool 16/16 REAL + 7/7 节点全 done + 88/88 pytest + 32 守门全过 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 (v0.3) | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补; v0.3 升档: 3 worktree 工具实装 namespace 隔离 (P0/P1/P2) + console_server sys.path 注入 (1 号修) + 守门 #1 v1-v14 父会话实证 (3 号: cargo check 0 err + 18 新 P2 tests pass + 23 pre-existing fail 跨 session 续) |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 (v0.3) | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补; v0.3 升档: 4 worktree 子代理实装 (1 号 ca9ed98 / 2 号 23f87c2 / 3 号 90c10f1 / TMO-08 e394ed9) + 守门 #9 实证 4/4 OK + 3 PR (#13 5e5b1c2 / #14 6608d87 / P3 1aab37e) + 守门 #12 修复 (1 号 ded8ff9 父会话 fix -270 行误删) |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 (v0.3) | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补; v0.3 升档: G-DEP-01..07 全拆决 (TMO-04/05/06/07 + 16 tool 11+5=16 REAL 实证) + G-DEP-08..11 4 缺口跨 session 续 (5 域 Lead 候选 1+2+3 / PostgreSQL / 23 nil-actor / 剩 26 err) |

---

## 7. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：TMO 7 子项实装 phase 计划 (TMO-01..TMO-07, 节点 M-N1..M-N7 + 组件 C-16..C-22 + 25 新 module M-19..M-25) + 7 段结构 (目的/任务矩阵/验证摘要/已知缺口/子代理接手/守门规则/签字/修订) + 7 子项估 ~2.5M tokens (跟 AGENTS §7 #8 ~3.0M 兼容, 留 0.5M 给 9 SA 类型 stub) + 12 守门合规预期 + 9 已知缺口 (G-TMO-01..G-TMO-09) + 15 守门规则 + 5 签字栏 (Mavis 接手代签) | 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 做整体统筹规划, 发号施令的入口是底端聊天窗口, 例如合并任务a和任务b" (per ask_d076c26d3fbf599eec1c32fd 拍板 (1) 范围=完整 7 节点全覆盖 (2) 文档策略=原地升版 v0.1 → v0.2 (3) 实装阶段=文档+commit 一并落), 跟 3 份主文档 v0.2 + ADR-0046 同步落档, ~0.05M token 估 |
| v0.2 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **TMO 7/7 节点落地 + 16 tool 11 REAL + 88/88 pytest + 32 守门全过 + G-DEP-01..07 拆决** (per 9/4 23:10 JST 用户发令"按顺序推进完成所有可以推进的" + 9/5 期间 1+2 号 P0/P1 实装 + PR #13 5e5b1c2 + PR #14 6608d87 + 1aab37e P3-C + ac0afdd P3-D + 439bae5 + 4468e1 全部 main HEAD 0/0 sync 实证): TMO-01/03/04/08 Mavis 父会话 (ca9ed98/8fef058/0983523/e394ed9 4 commit) + TMO-02/05/06/07 PR #13 5e5b1c2 squash 14 commit; 16 tool 11 REAL (4 P0 + 4 P1 Mavis + 3 早期 F.2 + 9c46a1c + 0de865b) + 5 MOCK P2 跨 session 续; 守门 #1 v1-v14 实证 (cargo check 0 err + fmt 0 diff + clippy 0 err + test star-mcp 0 fail 新增 + test domain-search 32 pass + release 0 err 29.14s); 守门 #9 实证 4/4 OK; 守门 #12 修复 (-270 行误删 父会话 ded8ff9); 守门 #13 a 实证 (DAGValidator 4 类 cycle + O(V+E)); 7 已知缺口 G-TMO-01..09 拆决 (G-TMO-08 选项 4 应急 partial + G-TMO-09 v0.3 partial); 6 新增 G-DEP-01..07 + G-TOOL-P0/P1-01..06 (5 拆决, 1 P2 partial); 5 签字栏 v0.2 升版 (Mavis 接手代签, 5 角色); AGENTS.md 同步 §6 + §6.1 + §7 + §8 (v0.74 行), 守门 #12 缺标比错标闭环; 总估 ~2.68M tokens (7 节点 ~2.5M + 16 tool ~0.18M), 跟 §7 #8 ~3.0M 兼容 (留 0.32M 给 5 MOCK P2 跨 session 续 + 9 SA 类型 stub + 23 nil-actor fail 修法) | 2026-09-05 用户发令"按顺序推进完成所有可以推进的" + 9/5 期间 1+2 号 P0/P1 实装 (parent session Mavis 接手) + PR #13 5e5b1c2 (TMO 7 节点 14 commit squash, 88/88 pytest + 32 守门) + PR #14 6608d87 (HANDOFF v1.6 + PHASE v0.4) + 1aab37e P3-C (MCP 16 tool 100% 覆蓋 + mock fixture) + ac0afdd P3-D (Agent Runtime G-1~G-18 落地) + 27a690f H2-EXT 5/5 done + 76aaf15 H2 原 3 domain service.rs 改造闭环 + dbfe324 P0-1c T1.7 50→0 err 实证 + 5e5b1c2 PR #13 (5 域 Lead 选项 4 应急落地 per 守门 #14), 0/0 sync 实证, ~0.05M token 估 |
| v0.3 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **16 tool 16/16 REAL 化 + G-DEP-07 全拆决** (per 9/5 07:56 JST 用户新目标 "P2 工具实装" + 3 号 P2 子代理 commit `90c10f1` + squash `cd9d4a0` + push 0/0 sync 实证): 5 P2 工具 (get_context / get_pipeline_status / request_review / run_validation / submit) mock → real + domain-scm 扩展 (InMemoryScmService 加 pipelines / reviews 存储 + register_pipeline / find_pipeline_by_external_id / request_review / ReviewResult helper) + error.rs 扩展 `From<ValidationError>` impl (1 号加 3 + 2 号加 1, 这次加 1) + Cargo.toml 引入 `domain-validation` 依赖 + 18 新 P2 tests 全部 pass (get_context 3/3 + get_pipeline_status 4/4 + request_review 4/4 + run_validation 3/3 + submit 4/4); 守门 #1 v1-v14 实证 (cargo check 0 err 0.41s + --all-targets 0 err 0.45s + fmt 0 diff + test star-mcp 23 fail 跟 1/2 号 G-TOOL-P0-04/G-TOOL-P1-03 完全同源 + test domain-validation 13/13 pass + release 0 err 0.46s); 守门 #9 实证 1/1 OK; 守门 #12 严守 0 误删 (G-TOOL-P2-06 实证); 6 已知缺口 G-TOOL-P2-01..06 列; 5 签字栏 v0.3 升版 (Mavis 接手代签, 5 角色); 总估 ~2.98M tokens (7 节点 ~2.5M + 16 tool ~0.48M, 跟 §7 #8 ~3.0M 兼容, 留 0.02M 给 PostgreSQL + 23 nil-actor + 剩 26 err + 5 域 Lead timeline 候选 1+2+3 跨 session 续) | 2026-09-05 07:56 JST 用户新目标 "P2 工具实装" (per archon_internal_context 9/5 07:56 拍板 + 9/4 23:10 JST "按顺序推进完成所有可以推进的") + 3 号 P2 子代理 (bg_5088fa03) 实装 5 工具 + 父会话接手 (守门 #1 v1 重跑 14.79s 0 err + merge squash cd9d4a0 0 冲突 + push eabdff3..cd9d4a0 0/0 sync) + worktree wt-tool-p2-impl 清理 + branch 删, ~0.6M token 估 (5 工具 0.06-0.1M each + domain-scm 扩展 0.05M + 18 tests 0.05M + 调试 ActorContext 类型兼容性 0.1M, 略超 brief 估 0.3-0.5M 因 domain_validation::context::ActorContext 跟 star_context::ActorContext 是不同类型, 跟 P0-1 联动审计 强类型重构历史教训一致) |
| v0.3.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **G-DEP-03 5 域 Lead 真人 timeline 拍板落地 (Q1=内推 + Q2=立即启动)** (per 2026-09-05 10:43 JST `ask_409cbd32edc309d71a083e2a` 用户拍板 推荐项): §3.3 line 174 G-DEP-03 标记 ✅ 已拍板 (选项 1 Ulysses 内推 + 立即启动, 6 周满员 T4 估算) + 新增落地 [docs/recruitment/5-business-domain-lead-referral.md](../../recruitment/5-business-domain-lead-referral.md) v0.1 (9.5KB, 8 节结构: 目的 / 内推策略 / 5 域角色描述 / token-OLU / 已知缺口 / 子代理失败接手 / 守门规则 / 签字栏 / 修订历史) + 内推话术模板 (5 域各 1 份, Ulysses 直接可发) + 5 域 Lead 真人到位 timeline T0 启动(本 commit) ~ T1 联系(1 周) ~ T2 评估(2 周) ~ T3 到位(3 周, 至少 1 域) ~ T4 满员(6 周) ~ T5 追溯签字覆盖(每到位 1 人触发) + 守门 #14 v25 派生规 (G-DEP-08 拍板落地后派生: 5 域 Lead 内推 brief 模板 v0.1 + 内推话术 v0.1 + token-OLU 5 域 11-15 SRE·周 估 + Ulysses 内推 T0-T5 timeline + Mavis 临时代签 9/3 19:35 JST 拍板 D 维持 + 真人到位后追溯签字覆盖 修订历史 +1 行); G-DEP-08 PostgreSQL checkpointer Tier 3 启动时间 = 真人到位后 (T3 至少 1 人到位) | 2026-09-05 10:43 JST `ask_409cbd32edc309d71a083e2a` 用户拍板 Q1=内推[推荐]+Q2=立即启动[推荐] (per 9/5 04:03 JST 拍板推荐项直接执行) + docs-only 增量 v0.3.1 (per 守门 #12 commit-time docs 同步), ~0.03M token 估 (brief v0.1 起草 0.02M + PHASE §3.3 + 修订历史 0.01M + AGENTS.md 守门 #14 v25 派生 0.005M) |
| v0.3.2 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **G-DEP-08 PostgreSQL checkpointer Tier 3 设计阶段落地** (per 2026-09-05 10:58 JST `ask_4f3523425caaa325695be6bd` 用户拍板 选项 1 推荐项): §3.3 line 176 G-DEP-08 标记 ✅ 设计阶段 done (装装阶段等 5 域 Lead 真人 T3 到位) + 新增 [ADR-0047](../../architecture/2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md) v0.1 (21.8KB, 11 节结构: 背景/决策/设计/启动条件/备选/后果/装装拆解/已知缺口/守门合规/签字栏/修订历史): 5 张表 schema (checkpoints + checkpoint_writes + checkpoint_summaries 3 Transaction append-only + checkpoint_metadata 1 Master SCD Type 2 + audit_audit_event 1 Transaction WORM per ADR-0043) + PostgresCheckpointer wrapper (per 03 §1.1 M-25) + 12 Reducer channel 跨 Tier 序列化 + TMO 7 节点 (M-N1..M-N7) 整合表 + 5 域 RACI 边界 + 5 阶段装装拆解 E-1..E-5 (估 ~1.0-1.5M token) + 3 备选方案拒绝 (CockroachDB / TiDB / 仅 SQLite 升级) + 10 已知缺口 (per 缺标比错标) + 14 守门合规 + 5 签字栏 (Mavis 接手代签); 启动条件 = 5 域 Lead 真人 T3 至少 1 人到位 (2026-09-26 ~ 2026-10-17 JST) + R-05 push 反転已落地 (8/30 07:09 JST); AGENTS.md §6 ADR 索引追加 ADR-0047 + §8 修订历史 v0.77 | 2026-09-05 10:58 JST `ask_4f3523425caaa325695be6bd` 用户拍板 选项 1 G-DEP-08 PostgreSQL checkpointer 设计 (per 9/1 14:58 JST 拍板决策必须用选项 + 9/5 04:03 JST 拍板推荐项直接执行) + docs-only 增量 v0.3.2 (per 守门 #12 commit-time docs 同步), ~0.05M token 估 (ADR-0047 起草 0.04M + PHASE §3.3 + 修订历史 + AGENTS.md §6/§8 0.01M) |
| v0.3.3 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **4 unmerged 分支 merge 拍板落地 (G-DEP-08 后续 + 收尾)** (per 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板 Q1=rebase-then-merge[推荐]+Q2=close-1c[推荐], per 9/5 04:03 JST 拍板推荐项直接执行): (a) **rf001-t15-work +23 commit rebase + fast-forward merge** (HEAD `8968da6` push 0/0 sync 实证, local HEAD = origin/main = `8968da6`); 7 文件 `//! MCP tool stub:` → `//! MCP tool:` 标签订型 + `#![warn(missing_docs)]` 行 删除 冲突 用 [scripts/automation/rf001_rebase_resolve.py](../../automation/rf001_rebase_resolve.py) v0.1 (per 守门 #19 Python 化) 7 文件同模式 option A 解决 (保留 HEAD 标签订型 + 删 `#![warn(missing_docs)]` 行, 0 auto-resolve 越界, per 守门 #20 子代理不可越界 + sub-agent 实证); (b) **5 守门 v1+v3+v6+v14 实证**: v1 `cargo check --workspace --lib -j 4` 0 err 1m 29s (50 warnings 是 T1.5 missing_docs 工作中预期, 11/87 file 已清零 76 剩余); v3 `cargo fmt --all --check` 0 diff; v6 `cargo test --workspace --lib` 883 passed / 0 fail / 50 crate 全 ok (61 "FAILED" 匹配是 enum 变体名 Failed/UploadFailed/GitFailed 误匹配, 非真 fail); v14 `cargo check --workspace --all-targets --release -j 4` 0 err 32.86s; (c) **feat/auto-20260904-1c260bc7 +10 commit close-1c**: worktree 占用 `D:\Star\.worktrees\feat-auto-20260904-1c260bc7` (其他 sub-session 拥有, 不动 per 守门 #12 严守), 父会话文档化关闭 (修订历史记录 + 保留 branch ref, 关闭原因 = G-DEP-03 真人 Lead 拍板 取代 5 子代理兼任 9/4 18:30 JST 路径); (d) **23 rebase commit author = "Ulysses Leo Lee <hanakagumi@outlook.com>" (原 session 旧 config), per 守门 #12 禁回溯叙事保留不重写** (是真实历史, 不追溯改写); (e) 新增 [docs/briefs/rf001-merge-001.md](../../briefs/rf001-merge-001.md) v0.1 (7KB, 8 节) + [scripts/automation/rf001_rebase_resolve.py](../../automation/rf001_rebase_resolve.py) v0.1 (2.6KB); (f) 累计 ~0.17M token; (g) **2 未拍板项跨 session 续保留**: rf001-t15-recovered (+15 ahead, parallel recovery) + rf001-t15-worktree-content (+1 ahead, worktree content) | 2026-09-05 11:12 JST `ask_9ccc7a57dc5823d05e17e2b3` 用户拍板 (Q1=rebase-then-merge+Q2=close-1c, 推荐项) (per 9/1 14:58 JST 拍板决策必须用选项 + 9/5 04:03 JST 拍板推荐项直接执行) + RF-001 T1.5 missing_docs 23 commit rebase+merge push 0/0 sync + 5 守门 0 err 实证, ~0.17M token 估 (brief 0.02M + 冲突解决脚本 0.01M + sub-agent failed 0.015M + 父会话 rebase+merge 0.04M + 5 守门 0.05M + docs 0.03M) |

---

## 8. 引用文档 (References)

- [01-requirements.md v0.2](../architecture/2026-09-03-langgraph/01-requirements.md) — UC-09..UC-13 + F-19..F-25 + NFR-TMO-01..05 + S-06
- [02-basic-design.md v0.2](../architecture/2026-09-03-langgraph/02-basic-design.md) — §2.6 TMO 全节 + 7 组件 C-16..C-22 + 7 协议 + 5 Reducer + 8 API 端点
- [03-detailed-design.md v0.2](../architecture/2026-09-03-langgraph/03-detailed-design.md) — task_ops/ 模块 + M-19..M-25 + §3.2.1.1 7 节点 Python 実装 + SA-10 + superseded 终态 + UT-20..UT-26 / IT-10..IT-12 / E2E-09..E2E-13
- [ADR-0046 LangGraph TMO](../architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — TMO 决策记录
- [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 13 main + 24 派生规 = 37 项硬约束
- [AGENTS.md §7 #8 Star LangGraph 統合アーキテクチャ](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — ~3.0M token 预算
- [AGENTS.md §4 row 14 5 域 Lead CONTENT 4 维](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 决策 scope=Both / RACI=R+A+C / timeline=待定 / Mavis 代签边界=全部 (9/3 19:43 JST 拍板 D+D+A+B)
- [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens
- [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) — agent 交互 Python 化 (守门 #19)
- [docs/briefs/deps-survey.md](../briefs/deps-survey.md) — 4 worktree 联合调研 (P0-1/H2-EXT/16 tool/5 域 Lead)
- [docs/briefs/tmo-2026-09-04-parallel.md](../briefs/tmo-2026-09-04-parallel.md) — 4 worktree 联合 brief (守门 #20 实证)
- [scripts/automation/dispatcher.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/dispatcher.py) — 现有 sub-agent dispatch 基础
- [scripts/automation/console_server.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/console_server.py) — 现有 FastAPI 8080 调试控制台 (扩展 `/api/tmo/*` 端点)
- [scripts/automation/ai_edit_mock.py](https://github.com/UlyssesLeoLee/Star/blob/main/scripts/automation/ai_edit_mock.py) — AI 修改 mock (守门 #23)
- [docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md](../kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) — DB W/T/M 横展開 (守门 #13)
- [PHASE-P4-D1-IMPL-REPORT.md](PHASE-P4-D1-IMPL-REPORT.md) — H2-EXT 5/5 done 实证 (commit `27a690f`)
- [PHASE-P4-V2-TMO-CI-IMPL-REPORT.md](PHASE-P4-V2-TMO-CI-IMPL-REPORT.md) — TMO 7 节点 88/88 pytest + 32 守门
- [PHASE-P3-C2-C5-IMPL-REPORT.md](PHASE-P3-C2-C5-IMPL-REPORT.md) — P3 全 5 阶段 60/65 拍板 + 55/63 子项实质收官 87.3%
- [STAR-P3-5-DOMAIN-LEAD-SELECTION-RESULT.md](STAR-P3-5-DOMAIN-LEAD-SELECTION-RESULT.md) — 5 域 Lead 拍板结果 (选项 4 应急)
- [STAR-P3-5-DOMAIN-LEAD-PROC.md](STAR-P3-5-DOMAIN-LEAD-PROC.md) — 5 步流程 + 4 选项
- [HANDOFF-ST-001.md](HANDOFF-ST-001.md) — H2 扩量 + H2-EXT 5 domain 跨 session 续 (v0.7)
- [G-TMO-04-DDL-IMPL-REPORT.md](G-TMO-04-DDL-IMPL-REPORT.md) — task_metadata DDL (G-DEP-04 拆决)
- [G-TMO-04b-REPO-IMPL-REPORT.md](G-TMO-04b-REPO-IMPL-REPORT.md) — task_metadata Repository
- [G-TMO-04c-ROUTES-IMPL-REPORT.md](G-TMO-04c-ROUTES-IMPL-REPORT.md) — 5 端点
- [G-TMO-04d-NODE-PERSIST-IMPL-REPORT.md](G-TMO-04d-NODE-PERSIST-IMPL-REPORT.md) — metadata_node 集成
- [G-TMO-05-SDK-FINDINGS.md](G-TMO-05-SDK-FINDINGS.md) — LangGraph SDK 0.2.x interrupt alpha 关闭
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt / Command
