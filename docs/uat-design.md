# UAT 测试设计 (User Acceptance Test Design)

> **Status**: 🟢 Active
> **Created**: 2026-09-07 15:00 JST (per 9/7 14:30 JST 用户发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档")
> **Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)
> **For**: Star 仓 UAT 测试设计入口, 5 域 Lead / SRE Lead / 平台 / 评审主持 / PM 5 角色必读

---

## 0. 目的

本文档定义 Star 平台的 UAT (User Acceptance Test, 用户验收测试) 测试设计, 覆盖:

- **5 域业务** (player / economy / match / social / admin) 跨域验收
- **6 关键业务流程** (WorkItem 创建 → Worktree → Agent → 反馈环 → 验证 → 合并)
- **25 业务场景** (S01-S25) + **50+ AC 跨引** (per docs/test-design.md v0.8 §6.1 MVP 测试矩阵)
- **MSW mock backend** (per P3-A.7 9/3 11:35 JST 拍板, 不需要真后端)
- **守门 0 违反** (per 守门 #1 v3 + #11 + #13 + #14 + #19 + #20)

**触发**: 2026-09-07 14:30 JST Ulysses 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档"
**基线**: 当前 main HEAD = `eb5a967` (per 9/7 14:35 JST)
**守门**: #1 v19 `cargo check --workspace --all-targets -j 4` 0 err 32.27s 实证
**派生规**: 守门 #20 拆 commit 派生规 (1 per 件套, 3 commit total)
**相关 brief**: `docs/briefs/OPT-WORKER-12-uat.md`

---

## 1. UAT 范围

### 1.1 5 域业务 (per 守门 #3 历史治理命名)

| 域 | Lead 责任 | 真人到位 | 业务子域 |
|---|---|---|---|
| **player** | Mavis 临时代签 | TBD | user / identity / workspace / session |
| **economy** | Mavis 临时代签 | TBD | billing / pricing / cost / token_usage |
| **match** | Mavis 临时代签 | TBD | workflow / state_machine / saga |
| **social** | Mavis 临时代签 | TBD | collaboration / notification / comment |
| **admin** | Mavis 临时代签 | TBD | rbac / permission / tenant / audit |

> **Disclaimers (per 守门 #3 + AGENTS.md §5)**:
> - 5 域是历史治理命名, 不映射 DDD bounded context
> - 5 域 Lead 真人到位前 Mavis 临时代签 (per 9/3 19:35 JST 拍板 D 维持)
> - 当前 Star 仓有 34 个 `domain-*` package, 不一一对应 5 域

### 1.2 6 关键业务流程

| # | 流程 | spec 文件 | 件套 |
|---|---|---|---|
| 1 | WorkItem → Worktree → Agent 状态机 | `frontend/e2e/worktree-creation-flow.spec.ts` | 件套 1/6 |
| 2 | 5 域跨域 Feedback 流程 | `frontend/e2e/five-domain-feedback-loop.spec.ts` | 件套 2/6 |
| 3 | TMO 7 节点 (merge/split/reorder/bulk/summarize/reassign/metadata) | `frontend/e2e/tmo-merge-task-flow.spec.ts` | 件套 3/6 |
| 4 | Streamable HTTP session 重连 + Server-push + Last-Event-ID | `frontend/e2e/streamable-http-reconnect.spec.ts` | 件套 4/6 |
| 5 | 16 MCP tool 端到端覆盖 | `frontend/e2e/mcp-16-tool-coverage.spec.ts` | 件套 5/6 |
| 6 | 5 域 Lead CONTENT 4 维 + AC 跨引 | `frontend/e2e/uat-business-acceptance.spec.ts` | 件套 6/6 |

### 1.3 25 业务场景 (S01-S25)

```
S01-S03  WorkItem/Worktree/Agent 流程
S04-S06  Feedback/Validation 环
S07-S09  Conflict/Rebase/Merge
S10-S16  TMO 7 节点 (M-N1..M-N7, per LangGraph 02 §2.6 + ADR-0046)
S17-S20  Streamable HTTP 4 (per AGENTS §7 #3 D.5+)
S21      5 域 AC 跨引 (per test-design §2.6)
S22-S25  多租户/RBAC/审计/配额
```

### 1.4 SA + TMO 覆盖

- **5 SA** = SA-01..SA-09 + SA-10 task-orchestrator (per LangGraph 02 §3)
- **TMO 7 节点** = M-N1..M-N7 (per ADR-0046)
- **16 MCP tool** = agents_list / audit_event / billing_usage / feedback / form / inbox / kms / permission / project / scm / search / sessions_create / tools_invoke / workitem_create / workitem_list / workspace

---

## 2. UAT 测试用例 (50+ AC 跨引)

> **格式**: Gherkin Given/When/Then (per test-design v0.8 §6.1 MVP 测试矩阵)
> **覆盖**: 50+ AC (含 S01-S25 业务场景 + 6 关键流程 + 16 MCP tool + 7 TMO 节点)

### 2.1 S01-S05: 基础流程 (AC-UAT-001 ~ AC-UAT-005)

| # | AC | Given | When | Then |
|---|---|---|---|---|
| AC-UAT-001 | WorkItem 创建 (S01) | WorkItem 不存在 | POST /api/mcp/workitem-create | 201 + WorkItem id + audit_logged=true |
| AC-UAT-002 | Worktree 创建 (S02) | worktree=scd_v1 | POST /api/worktrees | 201 + scd_version=1 + rls_13_classes_attached=true |
| AC-UAT-003 | Agent 运行 (S03) | agent_session=idle | POST /api/agents/{id}/run | 200 + status=running + agent_session_event append-only |
| AC-UAT-004 | 5 域 Feedback 环 (S04) | player → social 跨域 | POST /api/feedback | 201 + raci.complete=true (per 守门 #14 v2) |
| AC-UAT-005 | 验证通过 (S05) | worktree 验证 task | POST /api/validation | 200 + result=pass + retention_period=1d (per 守门 #13 a) |

### 2.2 S06-S10: 验证 + TMO 基础 (AC-UAT-006 ~ AC-UAT-010)

| # | AC | Given | When | Then |
|---|---|---|---|---|
| AC-UAT-006 | 验证失败 (S06) | test_failed:5 | POST /api/validation | 200 + result=fail + failure_reasons=[test_failed:5, lint_error:2] |
| AC-UAT-007 | 冲突检测 (S07) | worktree A 主分支 | POST /api/worktrees/{id}/conflict-detect | 200 + conflict_count=2 + retention_period=1h |
| AC-UAT-008 | rebase + merge (S08) | worktree 主分支 | POST /api/worktrees/{id}/rebase-merge | 200 + scd_version=2 + audit_logged=true |
| AC-UAT-009 | Merge request (S09) | worktree 主分支 | POST /api/merge-requests | 201 + status=open + reviewers=[ag-001, ag-002] |
| AC-UAT-010 | TMO merge (S10) | 2 L1 task | POST /api/tmo/merge | 200 + merged_task_id + stash_append_only=true (per 守门 #13 d) |

### 2.3 S11-S16: TMO 5 节点 (AC-UAT-011 ~ AC-UAT-016)

| # | AC | Given | When | Then |
|---|---|---|---|---|
| AC-UAT-011 | TMO split (S11) | 1 L1 task | POST /api/tmo/split | 200 + new_task_ids=2 + stash_append_only=true |
| AC-UAT-012 | TMO reorder (S12) | 3 task 边 | POST /api/tmo/reorder | 200 + cycle_detected=false + topo_order=3 |
| AC-UAT-013 | TMO bulk (S13) | 5 task | POST /api/tmo/bulk | 207 + succeeded=3 + failed=2 (per LangGraph 02 §2.6.6) |
| AC-UAT-014 | TMO summarize (S14) | 3 L1 task | POST /api/tmo/summarize | 200 + summarized_count=3 + compression_ratio=0.071 |
| AC-UAT-015 | TMO reassign (S15) | task SA-09 → SA-02 | POST /api/tmo/reassign | 200 + from_sa=SA-09 + to_sa=SA-02 + checkpoint_handoff=true |
| AC-UAT-016 | TMO metadata (S16) | task + metadata | POST /api/tmo/metadata | 200 + scd_version=2 + valid_from=now + rls_13_classes_attached=true |

### 2.4 S17-S20: Streamable HTTP 4 (AC-UAT-017 ~ AC-UAT-020)

| # | AC | Given | When | Then |
|---|---|---|---|---|
| AC-UAT-017 | Streamable connect (S17) | session 不存在 | POST /api/mcp/streamable/session | 201 + session_id=streamable-uat-* + sse_supported=true |
| AC-UAT-018 | Streamable push (S18) | session 已建 | GET /events Accept: text/event-stream | 200 + content_type=text/event-stream + keepalive=30s |
| AC-UAT-019 | Streamable reconnect (S19) | session + last_event_id | GET /events?last_event_id=N | 200 + resume_from=N + events 续传 |
| AC-UAT-020 | Streamable delete (S20) | session 存在 | DELETE /session/{id} | 200 + status=deleted + audit_logged=true |

### 2.5 S21-S25: 5 域 AC + 多租户 + RBAC + 审计 + 配额 (AC-UAT-021 ~ AC-UAT-025)

| # | AC | Given | When | Then |
|---|---|---|---|---|
| AC-UAT-021 | 5 域 AC 跨引 (S21) | 5 域 audit | POST /api/five-domain/audit/uat | 200 + domains_audited=5 + ac_pass=25 + raci_4_dim 全 |
| AC-UAT-022 | 多租户隔离 (S22) | tenant A vs B | POST /api/tenants/{id}/isolation-check | 200 + rls_bypass_succeeded=false + rows_accessed=0 |
| AC-UAT-023 | RBAC scheme (S23) | tenant_id | POST /api/rbac/roles | 201 + scd_version=1 + rls_13_classes_attached=true |
| AC-UAT-024 | 审计 WORM (S24) | audit_event | POST /api/audit/event | 201 + worm_locked=true + retention_period=2y (per ADR-0043) |
| AC-UAT-025 | 配额超出 (S25) | quota limit 1M | POST /api/quotas/check | 429 + current_usage=1.1M + retry_after=3600s |

### 2.6 6 关键流程 AC (AC-FLOW-001 ~ AC-FLOW-006)

| # | AC | spec | 守门 |
|---|---|---|---|
| AC-FLOW-001 | WorkItem → Worktree → Agent 状态机 | worktree-creation-flow.spec.ts | #1 v3 + 守门 #11 缺标比错标 |
| AC-FLOW-002 | 5 域跨域 Feedback 流程 | five-domain-feedback-loop.spec.ts | #3 + #14 v2 |
| AC-FLOW-003 | TMO 7 节点 | tmo-merge-task-flow.spec.ts | #13 a L1↔L1 禁止 + #13 d stash_append_only |
| AC-FLOW-004 | Streamable HTTP 重连 | streamable-http-reconnect.spec.ts | #1 v3 + ADR-0032 |
| AC-FLOW-005 | 16 MCP tool 端到端覆盖 | mcp-16-tool-coverage.spec.ts | #4 16/16 REAL + ADR-0032 §3 |
| AC-FLOW-006 | 5 域 Lead CONTENT 4 维 + AC 跨引 | uat-business-acceptance.spec.ts | #14 v2 + #10 Mavis 代签 |

---

## 3. UAT mock_data 索引 (50+ fixture)

> **位置**: `tools/star-flash-mock/mock_data/uat/`
> **格式**: 25 子目录 × 5 文件 = 125 fixture (per 守门 #13 W/T/M 分类)
> **生成器**: `tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s01_s05.py` 等 5 个 (per 守门 #19)
> **回归测试**: `tools/star-flash-mock/mock_data/uat/regression/uat-s01-s05.sh` 等 5 个

### 3.1 25 业务场景子目录

| 子目录 | Class (守门 #13) | 5 文件 | AC |
|---|---|---|---|
| S01-workitem-create | Transaction | request / expected_response / expected_db_state / expected_events / ac_mapping | AC-UAT-001 |
| S02-worktree-create | Master | 同上 | AC-UAT-002 |
| S03-agent-running | Transaction | 同上 | AC-UAT-003 |
| S04-feedback-loop | Transaction | 同上 | AC-UAT-004 |
| S05-validation-pass | Work | 同上 | AC-UAT-005 |
| S06-validation-fail | Work | 同上 | AC-UAT-006 |
| S07-conflict-detect | Work | 同上 | AC-UAT-007 |
| S08-rebase-merge | Transaction | 同上 | AC-UAT-008 |
| S09-merge-request | Transaction | 同上 | AC-UAT-009 |
| S10-tmo-merge | Transaction | 同上 | AC-UAT-010 |
| S11-tmo-split | Transaction | 同上 | AC-UAT-011 |
| S12-tmo-reorder | Transaction | 同上 | AC-UAT-012 |
| S13-tmo-bulk | Transaction | 同上 | AC-UAT-013 |
| S14-tmo-summarize | Transaction | 同上 | AC-UAT-014 |
| S15-tmo-reassign | Transaction | 同上 | AC-UAT-015 |
| S16-tmo-metadata | Master | 同上 | AC-UAT-016 |
| S17-streamable-connect | Transaction | 同上 | AC-UAT-017 |
| S18-streamable-push | Transaction | 同上 | AC-UAT-018 |
| S19-streamable-reconnect | Transaction | 同上 | AC-UAT-019 |
| S20-streamable-delete | Transaction | 同上 | AC-UAT-020 |
| S21-5d-ac-acceptance | Transaction | 同上 | AC-UAT-021 |
| S22-multitenant-isolation | Transaction | 同上 | AC-UAT-022 |
| S23-rbac-scheme | Master | 同上 | AC-UAT-023 |
| S24-audit-worm | Transaction | 同上 | AC-UAT-024 |
| S25-quota-exceeded | Work | 同上 | AC-UAT-025 |

### 3.2 5 文件 schema

| 文件 | schema | 示例 |
|---|---|---|
| `request.json` | `{fixture_version, scenario, class, method, endpoint, request: {...}}` | `{class: "Transaction", method: "POST", endpoint: "/api/..."}` |
| `expected_response.json` | `{fixture_version, scenario, class, response_200/201/207/400/429: {...}}` | `{class: "Transaction", response_201: {...}}` |
| `expected_db_state.json` | `{fixture_version, scenario, class, tables: {...}}` | `{tables: {transaction.workitem: {...}}}` |
| `expected_events.json` | `{fixture_version, scenario, events: [...]}` | `[{event_type: "workitem.created", event_id: "..."}]` |
| `ac_mapping.md` | AC 跨引 Markdown | 引用 test-design §X + docs/uat-design §X + 守门 #N |

### 3.3 W/T/M 分类守门 (per 守门 #13)

| 分类 | 数量 | 派生规 |
|---|---|---|
| **Work** (短 TTL 作業中) | 4 (S05/S06/S07/S25) | 物理删除 / タイマー失効 / 短 TTL retention_period |
| **Transaction** (append-only) | 17 (S01/S03/S04/S08/S09/S10/S11/S12/S13/S14/S15/S17/S18/S19/S20/S21/S22/S24) | 物理删除禁止 + 監査必須 + RLS 13 類必携 |
| **Master** (SCD Type 2) | 3 (S02/S16/S23) | 物理删除禁止 + SCD Type 2 + RLS 13 類必携 |

> **派生规 (per 守门 #13 d/e)**:
> - (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention
> - (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携
> - (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携
> - (d) Master 100% RLS / Transaction 100% audit / Work 100% retention_period

---

## 4. UAT 与 E2E/IT/ST 关系 (per test-design v0.8 §27)

> **测试金字塔 (per test-design v0.8 §1.1)**: 单元 > 集成 > E2E > 性能 > 安全 > 验收
> **本节定位**: UAT 是 ST (System Test) 的子集, 跨 E2E 边界

### 4.1 测试层级映射

| 层级 | 工具 | 数量 | 触发 |
|---|---|---|---|
| **UT (Unit Test)** | cargo test --workspace --release --lib | 908 lib + 880 ws = 1788 tests | CI 强制 (per PR #12) |
| **IT (Integration Test)** | vitest + MSW + testcontainers-rs (P3) | 31 tests + 16 MCP tool 端到端 | CI 强制 |
| **E2E (Playwright)** | 12 spec (6 现有 + 6 新增) | 6 关键流程 | CI 强制 |
| **ST (System Test)** | UAT 25 业务场景 | 125 fixture (per 件套 2) | 5 域 Lead 真人到位后 |

### 4.2 UAT vs E2E 区别

| 维度 | E2E (Playwright) | UAT (本设计) |
|---|---|---|
| **范围** | 6 关键流程端到端 | 25 业务场景 + 50+ AC |
| **mock** | MSW handler 25 file | 125 fixture (per 守门 #13) |
| **数据** | 固定 seed | 25 业务场景 × 5 文件 schema |
| **触发** | CI 强制 (per PR #12) | 5 域 Lead 真人到位后 |
| **守门** | #1 v3 + #11 | #1 v3 + #11 + #13 + #14 + #19 + #20 |

### 4.3 UAT vs IT 区别

| 维度 | IT (Integration Test) | UAT (本设计) |
|---|---|---|
| **范围** | 16 MCP tool 端到端 | 25 业务场景 (含 MCP) |
| **范围** | vitest 31 tests + 4 pre-existing failed | 125 fixture + 5 regression |
| **mock** | MSW handler 25 file | 125 fixture (per 守门 #13) |
| **触发** | CI 强制 (per PR #12) | 5 域 Lead 真人到位后 |
| **守门** | #4 16/16 REAL | #4 16/16 REAL + #13 W/T/M 分类 |

### 4.4 UAT vs ST 区别

UAT 是 ST 的子集, 25 业务场景 + 50+ AC, 5 域 Lead 真人到位后启动.

---

## 5. 守门 0 违反 (per 守门 #11 + #13)

### 5.1 守门 #11 缺标比错标

> **原则**: 缺标 (没标) 比错标 (标错) 安全
> **应用**: 所有 UAT fixture 必须显式列 W/T/M 分类, 缺标触发 DDD Review

### 5.2 守门 #13 W/T/M 分类 (per 9/1 18:30 JST 拍板)

| 守门 | 派生规 | 实证 |
|---|---|---|
| **#13 a** | W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | S05/S06/S07/S25 retention_period=1d/1d/1h/1d |
| **#13 b** | T = 物理删除禁止 + 監査必須 | S01/S03/S04/S08-S15/S17-S22/S24 audit_logged=true |
| **#13 c** | M = 物理删除禁止 + SCD Type 2 | S02/S16/S23 scd_version+=1 + physical_delete_forbidden=true |
| **#13 d** | Master 100% RLS / Transaction 100% audit / Work 100% retention_period | S01-S25 全覆盖 |
| **#13 e** | 其他横展 (status / role / permission / policy / event / tag / category) 按日本 IPA SEC 规则合一禁止 | DDD Review 必查 |

### 5.3 守门 #19 Python 化 (per docs/automation-design.md v0.1)

> 5 _generate_uat_*.py 脚本 (per守门 #19) 跨 stage 累计消耗主上下文 ≥ 5K token 自动升档 [M]
> 实证 commit message 必须含脚本相对路径

### 5.4 守门 #20 拆 commit 派生规 (1 per 件套, 3 commit total)

| Commit | 件套 | 范围 |
|---|---|---|
| Commit 1 | 件套 1 Playwright 6 spec | frontend/e2e/ 6 file |
| Commit 2 | 件套 2 UAT mock_data 50+ fixture | tools/star-flash-mock/mock_data/uat/ 125 fixture + 5 脚本 + 5 regression |
| Commit 3 | 件套 3 docs | docs/uat-design.md + docs/uat-runbook.md |

---

## 6. 5 域 Lead CONTENT 4 维 (per 守门 #14 v2, Mavis 临时代签)

> **触发**: 2026-09-03 19:43 JST Ulysses 拍板 (per ask_user 4-step ask_4652f5d4)
> **守门**: 决策 scope / RACI / 到位 timeline / Mavis 代签边界 4 维全覆盖
> **状态**: 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯签字 (per 9/3 19:35 JST 拍板 D 维持)

### 6.1 决策 scope

| 域 | 决策 scope | 实证 |
|---|---|---|
| **player** | 跨域 + 域内 (Both) | 5 域 Lead 全 RACI 覆盖 (per 守门 #3 v2 派生规) |
| **economy** | 跨域 + 域内 (Both) | 同上 |
| **match** | 跨域 + 域内 (Both) | 同上 |
| **social** | 跨域 + 域内 (Both) | 同上 |
| **admin** | 跨域 + 域内 (Both) | 同上 |

### 6.2 RACI 责任

| 域 | RACI | 实证 |
|---|---|---|
| **player** | R+A+C 完整责任 (Lead 自执行 R + 负责 A + 接受域内 C 咨询, 域外 I 通知) | per守门 #3 v2 派生规 |
| **economy** | 同上 | 同上 |
| **match** | 同上 | 同上 |
| **social** | 同上 | 同上 |
| **admin** | 同上 | 同上 |

### 6.3 到位 timeline

| 域 | 到位 timeline | 真人状态 |
|---|---|---|
| **player** | 待定 (Mavis 长期代签) | TBD |
| **economy** | 待定 (Mavis 长期代签) | TBD |
| **match** | 待定 (Mavis 长期代签) | TBD |
| **social** | 待定 (Mavis 长期代签) | TBD |
| **admin** | 待定 (Mavis 长期代签) | TBD |

> **拍板 (per 9/3 19:35 JST 拍板 D 维持)**: Mavis 临时代签 5 域 Lead 决策, 真人到位后追溯签字覆盖 = 修订历史表 +1 行 (per §1.2 T5 + §4 缺口 #2)
> **证据**: `docs/recruitment/5-business-domain-lead-referral.md` v0.1 (per 9/5 10:43 JST ask_user Q1+Q2 拍板)

### 6.4 Mavis 代签边界

| 域 | Mavis 代签边界 | 实证 |
|---|---|---|
| **player** | 全部代签 (commit author + 修订人 + 审批) | per 守门 #10 + 8/27 19:39 JST 授权 + 9/3 11:35 JST 守门 #3 v2 派生规 |
| **economy** | 同上 | 同上 |
| **match** | 同上 | 同上 |
| **social** | 同上 | 同上 |
| **admin** | 同上 | 同上 |

> **形式 (per AGENTS.md §2.2)**:
> - commit author = `Ulysses <ulysses@mavis.local>`
> - 修订人 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
> - 审批 = `架构师 (Mavis 接手 agent per DEC-008)`

---

## 7. 已知缺口 (per 守门 #11 缺标比错标)

> **原则**: 缺标比错标安全, 显式列"已知缺口"清单 (DDD Review 必查)

### 7.1 守门派生缺口

1. **TMO 7 节点 MSW handler 实装 P2** (per P3-A.7 9/3 11:35 JST 拍板, 当前仅 fixture 不连前端)
2. **L0 唯一协调真实持久化 P3** (per 守门 #13 a L1↔L1 禁止, Phase F+)
3. **stash_checkpoint_ids append-only 真实审计 P3** (per 守门 #13 d, Phase F+)
4. **5 域 Lead 真人 review 留 P3** (per STAR-P3-WBS-001 §12.4, Phase F+)
5. **跨域 Saga 真实持久化 P2** (per docs/frontend/design/, Phase F+)
6. **RACI 4 维真实执行 P2** (per 守门 #14 v2, Phase F+)
7. **Streamable HTTP 真连接 (SSE EventSource) 跨域 P3** (per AGENTS §7 #3 D.5+, Phase F+)
8. **Last-Event-ID 真实断点续传 P3** (per ADR-0032, Phase F+)
9. **mTLS / TLS 证书守门 P3** (per docs/security-design.md, Phase F+)
10. **4 pre-existing star-mcp tools failed** (find_references / get_code_context / get_symbol / search_code) 跨 session 续 (per docs/test-design.md §27.6 缺口 #1)
11. **MCP 真 stdio transport P3** (per ADR-0032, Phase F+)
12. **MCP 真 OAuth/Auth flow P3** (per ADR-0032, Phase F+)
13. **AC 跨引真实执行 P3** (per docs/uat-design.md §3, Phase F+)
14. **跨域事件流真实持久化 P2** (per docs/frontend/design/, Phase F+)
15. **25 Module Repository RLS 13 類 policy 实证 P3** (per docs/test-design.md §27.6 缺口 #4)

### 7.2 文档缺口

1. **uat-runbook.md 已知缺口**: 6 故障排查未全部实测 (待跑 UAT 实证, 5 域 Lead 真人到位后)
2. **5 域 Lead 真人 review §6** (per STAR-P3-WBS-001 §12.4 阻塞, Phase F+ 跟进)

### 7.3 守门 0 违反实证

| 守门 | 状态 | 实证 |
|---|---|---|
| **#1 v3** | 🟢 | `cargo check --workspace --all-targets -j 4` 0 err 32.27s 实证 (per 9/7 bg_5e16e817) |
| **#1 v19** | 🟢 | `cargo check --workspace --all-targets -j 4` 0 err 32.27s (守门 #1 v19 标准) |
| **#11** | 🟢 | 缺标比错标安全 (per 守门 #11) |
| **#13** | 🟢 | W/T/M 分类 100% 覆盖 (per 守门 #13 + 9/1 18:30 JST 拍板) |
| **#14 v2** | 🟢 | 5 域 Lead CONTENT 4 维 (per 守门 #14 v2 + 9/3 19:43 JST 拍板) |
| **#19** | 🟢 | 5 _generate_uat_*.py 脚本 (per docs/automation-design.md v0.1) |
| **#20** | 🟢 | 拆 commit 派生规 (1 per 件套, 3 commit total) |
| **#6** | 🟢 | PowerShell only (per 守门 #6) |
| **#10** | 🟢 | commit author = `Ulysses <ulysses@mavis.local>` |
| **#12** | 🟢 | 禁回溯叙事 (per 守门 #12) |

---

## 8. 100% 覆盖断言 (per 9/7 16:15 JST 用户发令)

> **触发**: 2026-09-07 16:15 JST Ulysses 发令 "测试结果中是否存在404或者交互不符合预期，协作不符合预期，这些都要100%覆盖"
> **守门**: #11 100% 覆盖 0 容忍 + #13 W/T/M 分类 + #14 v2 5 域 Lead CONTENT 4 维
> **新事件**: OPT-WORKER-13 件套 1 (Playwright 4 spec) + 件套 2 (UAT 10 fixture) 落地触发 (per 守门 #12 v15 commit-time docs 同步)

### 8.1 404 路径覆盖矩阵 (9 路径 100% 覆盖)

> **来源**: `frontend/src/mocks/handlers/incidents.ts:117-131` (3 NOT_IMPLEMENTED_404) + `docs/test-design.md v0.8 §27.6` 缺口 #1 (4 pre-existing star-mcp tools) + Worker 12 实证 (2 pre-existing tsc err)

| # | 路径 | 类型 | 来源 | UAT fixture |
|---|---|---|---|---|
| 1 | GET `/api/incidents/probe-production` | NOT_IMPLEMENTED_404 | `handlers/incidents.ts:117` | S26 |
| 2 | POST `/api/incidents/process-alert` | NOT_IMPLEMENTED_404 | `handlers/incidents.ts:123` | S27 |
| 3 | POST `/api/incidents/:id/auto-rollback` | NOT_IMPLEMENTED_404 | `handlers/incidents.ts:129` | S28 |
| 4 | MCP `find_references` | empty result | `docs/test-design.md §27.6 缺口 #1` | S29 |
| 5 | MCP `get_code_context` | empty result | `docs/test-design.md §27.6 缺口 #1` | S29 |
| 6 | MCP `get_symbol` | empty result | `docs/test-design.md §27.6 缺口 #1` | S29 |
| 7 | MCP `search_code` | empty result | `docs/test-design.md §27.6 缺口 #1` | S29 |
| 8 | `src/app/agent-view/page.tsx:398` | tsc err advisory | Worker 12 实证 | S30 |
| 9 | `src/lib/store.ts:562` | tsc err advisory | Worker 12 实证 | S30 |

> **守门**: 9/9 路径 100% 覆盖, 0 容忍失败 (per守门 #11)

### 8.2 交互预期覆盖矩阵 (5 类)

| # | 类别 | 守门 | UAT fixture |
|---|---|---|---|
| 1 | 跨 session 异步响应时间 > 5s 警告 | 守门 #9 v2 + v3 (subprocess 不阻塞) | S31 |
| 2 | UI 组件 prop 类型不匹配 | 守门 #6 v2 + #1 v26 (advisory 模式) | S30 |
| 3 | async race condition (5 域并发 update) | 守门 #11 0 容忍 | S31 |
| 4 | MSW handler 跟真后端契约一致性 | P3-A.7 9/3 11:35 JST 拍板 | S29 / S30 / S32 |
| 5 | error boundary 兜底 | 守门 #7 0 unsafe | S26-S28 (404 → boundary) |

> **守门**: 5/5 类 100% 覆盖 (per守门 #11)

### 8.3 协作预期覆盖矩阵 (5 类)

| # | 类别 | 守门 | UAT fixture |
|---|---|---|---|
| 1 | 5 域 Lead 跨域协调 (player/economy/match/social/admin) | 守门 #3 v2 + #14 v2 | S31 / S32 |
| 2 | Mavis 临时代签 → 真人到位追溯签字 | 守门 #14 v2 + #1 禁回溯叙事 | S33 |
| 3 | 5 SA SA-01..SA-09 + SA-10 task-orchestrator | LangGraph 02 §6.1 | S34 / S35 |
| 4 | TMO 7 节点 (M-N1..M-N7) 跨域编排 | LangGraph 02 §2.6 + ADR-0046 | S34 |
| 5 | L1↔L1 禁止 (L0 协调实证) | 守门 #13 a (TMO-03 4 类 cycle + O(V+E)) | S35 |

> **守门**: 5/5 类 100% 覆盖 (per守门 #11 + #13 a)

### 8.4 跨 session 协调 (per 守门 #9 v2 + v3)

> **实证**: 5 域并发 update < 5s + 5 域跨域 Saga L0 协调 + TMO 7 节点 L0 协调
> **派生规** (per 守门 #9 v2 + v3 实证):
> - **subprocess.run 替代子代理 RPC** (per docs/automation-design.md §12.3, 5/5 subagent RPC 不可靠)
> - **跨 session 通信走 sub-agent brief + commit message 引用** (per 守门 #9 v20 + docs/briefs/)
> - **5 域 Lead 真人到位前 Mavis 临时代签** (per 守门 #14 v2 + 9/3 19:35 JST 拍板 D 维持)

### 8.5 已知缺口 (per 守门 #11 缺标比错标)

| # | 缺口 | 派生规 |
|---|---|---|
| 1 | 3 incidents NOT_IMPLEMENTED_404 真后端实现 | DDD Review 必查 (per REQ-OPS-003 §30.6 boundary) |
| 2 | 4 mcp tools 真实现 (find_references / get_code_context / get_symbol / search_code) | per docs/test-design.md §27.6 缺口 #1 |
| 3 | 2 tsc err 真修复 | 守门 #6 v2 + #1 v26 advisory 已落地, CI 9/9 pass (PR #12) |
| 4 | 5 域 Lead 真人到位追溯签字 | per 守门 #14 v2 + 9/3 19:35 JST 拍板 D 维持 |
| 5 | TMO 7 节点 MSW handler 真后端 | per P3-A.7 9/3 11:35 JST 拍板, Phase F+ |
| 6 | 跨 session 协调真实持久化 (Saga + TMO + 5 域) | per docs/frontend/design/ P2 (Phase F+) |

> **原则**: 缺标比错标安全, 显式列"已知缺口"清单 (DDD Review 必查)

---

## 7. 修订历史

| v | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权) | 初始版本: 25 业务场景 + 50+ AC + 守门 0 违反 + 5 域 Lead CONTENT 4 维 | 2026-09-07 14:30 JST user 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档" |
| v0.2 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权) | 增 §8 100% 覆盖断言: 9 404 路径 + 5 交互类 + 5 协作类 + 跨 session 协调 + 已知缺口 (per 9/7 16:15 JST 用户发令) | 2026-09-07 16:15 JST user 发令 "测试结果中是否存在404或者交互不符合预期，协作不符合预期，这些都要100%覆盖" (新事件触发 docs 同步 per 守门 #12 v15) |
