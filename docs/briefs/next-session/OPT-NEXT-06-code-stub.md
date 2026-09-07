# Brief: OPT-NEXT-06 — OPT-CODE-01..47 stub 实施 (per OPT-CODE, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟡 等待 Phase D / Phase H 实施进度
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

实装 47 个 stub 函数 (per OPT-A1 §3), 跨 4 stub 集中区:

- **OPT-CODE-01..25**: `star-api-rest` 22 路由 + 3 中间件 (auth/rate_limit/audit) — per spec §2.2 + §2.3 P2 阶段
- **OPT-CODE-26..47**: `domain-batch` 22 stub (4 不变量 + 16 NoopBatchService + 2 helper) — per BATCH-REQ-001 §3.3
- **OPT-CODE-48..67**: `domain-report` 14 chart P1/P2 + 4 port stub — per report v0.1
- **OPT-CODE-68..80**: `domain-form` 2 regex + `domain-search` JQL + 6 supporting crate 占位

**总估 token**: 1.5-3.0M
**触发**: Phase D (5.6 H2 原 3 domain) + Phase H (H.3 16 tool sub-agent 経由) 推进

## 2. 依赖 (per OPT-A1 §3)

| # | 依赖 | 状态 |
|---|---|---|
| 1 | `star-api-rest` spec §2.2 + §2.3 (P2 阶段实装计划) | 🟢 已 docs 化 |
| 2 | `domain-batch` invariant 12 个 v0 phase 1 stub | 🟡 partial (4 实装 + 8 占位 per `invariant.rs:51`) |
| 3 | `domain-report` 22 chart stub | 🟡 partial (8 真实 + 14 stub per `lib.rs:5-65`) |
| 4 | `domain-form` regex 真实实现 | 🟡 stub `regex_lite` 永远 true |
| 5 | `domain-search` JQL parser AST | 🟡 stub 内存 executor |

## 3. 实施路径 (per crate 分批)

### Batch 1: `star-api-rest` 22 路由 (OPT-CODE-01..25)

- 11 routes 模块文件: context/code/submissions/work_items/reviews/worktrees/workspaces/pipelines/webhooks/validations/merge_requests
- 22 路由 + 3 中间件 (auth/rate_limit/audit)
- 实证: 0 err + integration test 覆盖

### Batch 2: `domain-batch` 22 stub (OPT-CODE-26..47)

- 4 不变量 v0 phase 1: tenant_domain / dag_acyclic / node_type_approved / scd_type2
- 2 helper: validate_dag_topology (DFS/Kahn) / validate_node_type_approved
- 16 NoopBatchService 方法: create/update/delete/enable/disable/trigger/cancel_run + 9

### Batch 3: `domain-report` 18 stub (OPT-CODE-48..65)

- 6 P1 chart: C08 Throughput / C09 Forecast / C10 TimeTracking / C11 ResolutionTime / C12 SLA / C14 IssueTypeDist
- 8 P2 chart: C15-C22
- 4 port stub: InMemoryWorkItemPort + InMemorySprintPort + 2 others (V2 接真实)

### Batch 4: 散落 (OPT-CODE-66..80)

- `domain-form` regex_lite + re_is_match
- `domain-search` JQL 内存 executor → 真实 parser
- `crates/application/api/infrastructure` 3 supporting crate 占位结构 (12 占位) — Phase 2 删除, 改 use domain_xxx

## 4. Worktree

```bash
git worktree add -b feat/opt-stub-impl D:/Star/.worktrees/wt-opt-stub-impl main
cd D:/Star/.worktrees/wt-opt-stub-impl
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err
- 守门 #4: 25 domain-* crate 真实数据接入 (per 9/4 P3-A 11/25 实证, 目标 25/25)
- 守门 #10: author=Ulysses
- 守门 #12: 禁回溯叙事
- 守门 #19: 优先 Python 化 (per `docs/automation-design.md` v0.1)

## 6. 提交

4 commit (per 4 batch) 或 1 合并

## 7. 失败处理

- 0 重试 > 2 次 (per PowerShell fail-fast)
- 仍失败: 报告具体错误, 不 commit

## 8. 状态

🟡 **推下 session** (Phase D + H 推进触发)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §3.2 #1-#4
- 基线: `docs/briefs/OPT-A1-code-todo-scan.output.md` §3
- 25 domain-* baseline: `docs/reports/STAR-P3-WBS-001.md` §0 (11/25 已落地)
- 守门 #4: `AGENTS.md` §4 #4
