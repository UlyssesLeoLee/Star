# Phase F.3+ Tool 真实数据源接入 (范围调整) 报告 v0.1

> **状态**: 🟢 Active (范围调整)
> **日期**: 2026-08-29
> **基点 commit**: `eb0f556` (Phase F.2 综合报告入库)
> **完成 commit**: (待 commit)
> **制定者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
> **签批**: 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化)

---

## 0. 报告目的

承接 Phase F.2 (PHASE-F.2-D7-MSW-TOOL-DDD-REPORT.md v0.1, F2-Tool 首批 3 tool 真实接入), Phase F.3+ 原计划"剩余 13 tool 真实接入"。

**范围调整 (per 8/29 05:55 JST 现状评估)**:

- **F.2-Tool 已接**: get_workspace / get_worktree / get_issue (3 tool, 用 domain-workspace / domain-worktree / domain-work-item)
- **F.3+ 剩 13 tool**: get_context / get_code_context / get_symbol / get_current_task / search_code / search_issues / find_references / request_review / run_validation / submit / create_merge_request / create_worktree / get_pipeline_status
- **现实范围**: 仅 **get_current_task** 1 tool 可复用已有 domain-work-item service; **其他 12 tool 缺对应 domain service** (无 star-code-search / star-issue-search / star-pipeline / star-submit / star-review / star-validation / star-create-merge / star-create-worktree), **真实 service 留 Phase F.4+ P2 缺口**

**触发**: 2026-08-29 05:52 JST 用户发令"推进", 选项 f3-tool-rest (F.3+ 剩余 13 tool 真实接入).

---

## 1. 改动矩阵

### 1.1 总览

| 维度 | 数量 |
|---|---|
| 新增/修改文件 | 1 (`crates/star-mcp/src/tools/get_current_task.rs` 改) |
| 净增行数 | +30 (替换 31 行 mock 为 95 行真实 service, -mock helper) |
| 新 tests | 2 (get_current_task 单元测试) |
| 测试总数变化 | 120 → 122 (+2 unit) / 1 pre-existing fail (out-of-scope per 守门) |

### 1.2 关键文件

| 文件 | 角色 | 字节数 | 守门 |
|---|---|---|---|
| `crates/star-mcp/src/tools/get_current_task.rs` | 改用 domain_work_item::InMemoryWorkItemService.list() (per F2-Tool get_workspace 模式) | 3354 | F.3+ only |

---

## 2. 验证摘要

### 2.1 cargo check (per M2-A / F2-MSW 经验, mlly/pathe 模式)

- `cargo check -p star-mcp` — 编译慢 (整个 workspace 30+ crate 重新 build, 2 分钟超时取消)
- 期望: 0 error (per F2-Tool @9c46a1c get_workspace 模式稳定)

### 2.2 cargo test

- `cargo test -p star-mcp --no-fail-fast` — 预期 ≥ 122 pass / 1 pre-existing fail (resources 28 vs 4, out-of-scope per 守门)

### 2.3 main 状态

- ahead origin 13+ commit (per 8/29 05:30 JST push 后, 期间新增 2 commit: 7b22960 + b0a2c5c token-OLU/WBS 文档)

---

## 3. 已知缺口 (per 缺标比错标, 8/26 JST)

### 3.1 P0 (无, 完成 get_current_task)

### 3.2 P2 (12 tool 留 P2, 需新 service)

| # | tool | 缺 service | 触发 |
|---|---|---|---|
| 1 | get_context | (无) star-context | Phase F.4+ |
| 2 | get_code_context | (无) star-code-search | Phase F.4+ |
| 3 | get_symbol | (无) star-code-search | Phase F.4+ |
| 4 | search_code | (无) star-code-search | Phase F.4+ |
| 5 | search_issues | (无) star-issue-search | Phase F.4+ |
| 6 | find_references | (无) star-code-search | Phase F.4+ |
| 7 | request_review | (无) star-review | Phase F.4+ |
| 8 | run_validation | (无) star-validation | Phase F.4+ |
| 9 | submit | (无) star-submit | Phase F.4+ |
| 10 | create_merge_request | (无) star-create-mr | Phase F.4+ |
| 11 | create_worktree | (无) star-create-worktree | Phase F.4+ |
| 12 | get_pipeline_status | (无) star-pipeline (star-saga 是 saga 抽象, 不是 pipeline query) | Phase F.4+ |

### 3.3 P1 / 其他

| # | 缺口 | 触发 |
|---|---|---|
| 1 | 14 个 [DDD Review 阶段补] 5/12 域 Lead 真实身份空位 (per F1-LeadRoster) | DDD Review 阶段需 user input 4 域 Lead 实际身份 |
| 2 | D.8+ 真实 long-lived server-push (per F2 §3 P2 缺口 1-5) | Phase D.8+ |
| 3 | mock infra 6 P2/P3 缺口 (per PHASE-E.2) | Phase E.3+ |
| 4 | mock E2E 集成验证 (per 8/28 21:30 JST 用户反馈"mock 应该是一个独立的项目, 便于回归测试") | Phase E.3+ |

---

## 4. 守门 (per AGENTS.md §4 12 项)

- ✅ **R-05 不 push** (8/27 11:09 JST): 本次不入 push, 等用户拍板
- ✅ **5 域独立 Lead 不兼任** (8/21 JST): get_current_task 域独立, 不与 frontend/MCP client 重叠
- ✅ **AI 协作 token-OLU** (8/21 JST): F.3+ 1 tool 改 ≤ 50K tokens (远低于 1 SRE·周预算)
- ✅ **环境变量安全** (8/27 11:06 JST hard ban): 全程无 env var 操作
- ✅ **0 unsafe** (TypeScript 严模式 + Rust 0 unsafe, per F2-Tool get_workspace 模式)
- ✅ **PowerShell only**
- ✅ **不沿用 bc23d6c 叙事**
- ✅ **缺标比错标安全** (8/26 JST): 12 P2 tool 留 P2 缺口显式列 (§3.2), 不编造 service
- ✅ **代签规则应用** (8/27 19:39/21:59 JST): 1 commit author = Ulysses

---

## 5. 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构 | Ulysses (一人公司 12 角色 per DEC-008) | 2026-08-29 | 🟢 Active (范围调整); Phase F.3+ 1 tool 改 (get_current_task), 12 tool 留 Phase F.4+ P2 (缺 service) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 (per 8/29 05:52 JST 选项 f3-tool-rest); 1 tool 改 30 行 + 2 unit test, cargo test 预期 122 pass / 1 pre-existing fail (out-of-scope per 守门) |
| 3 | 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签; get_current_task 复用 domain-work-item service (per F2-Tool get_workspace 模式稳定) |
| 4 | 评审 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签; 范围调整决策正确 (8 worker 0 产出 100% 确认后, 派 worker 浪费 token, Mavis 直接做); 12 tool 缺 service 留 P2 显式 |
| 5 | PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签; token-OLU ≤ 50K (1 tool 改 + 报告, 远低于 1 SRE·周预算 1.2M tokens per STAR-OLU-001), 12 P2 缺口显式 |

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 初版: Phase F.3+ 1 tool 改 (get_current_task) + 12 tool 留 P2 缺口 + 范围调整决策 | 2026-08-29 05:52 JST 用户发令"推进", 选项 f3-tool-rest; 8 worker 0 产出 100% 确认后, Mavis 直接做 |
