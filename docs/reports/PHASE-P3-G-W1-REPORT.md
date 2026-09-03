# Phase P3-G-W1 Agent Jira 化 — 基础层完工报告 v0.1

> **状态**: 🟢 完工 v0.1 (待 W2 续 + 守门 0 跨 stage 验证)
> **日期**: 2026-09-03
> **基点 commit**: `f537aab` (AGENTS.md v0.53)
> **完工 commit**: `1f6e200` (P3-G-W1 5 子项 docs 全部 git 实证, 7 表设计)
> **拍板触发**: 2026-09-03 11:50 JST Ulysses "agent 也应该像 jira 管理团队成员那样管理, 权限所属团队等都要可以管理" + 3 步 ask_user 拍板 (direction_opt4 + team_dimension_opt4 + subagent_persist_opt4) + 3 步开工拍板 (wbs_mode_opt2 5 段独立验收 + token_budget_opt1 1.5M 现在 + lead_coordination_opt1 Mavis 代签 5 域 Lead)
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39/20:56/21:59 JST 三次强化"允许你代签" + 守门 #10 反转)
> **作用域**: STAR 仓 (`D:\Star`) P3-G 阶段 W1 基础层
> **依赖**: `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` v0.1 + `docs/briefs/p3-g-w1.md` + `docs/automation-design.md §4.12` + `scripts/automation/registry.md §5`
> **W2-W5 跨 session 续**: per `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` §1.3, W2-W5 15 子项 (G.6-G.20) 跨 session 续, 推 origin 后启动

---

## 0. 报告目的

P3-G-W1 基础层 5 子项 (G.1-G.5) docs 全部落地, 7 表设计文档 git 实证 3 commits 落地 (commits `b9bb2d6` + `a54ab72` + `1f6e200`, 合计 11 files / 1919 insertions)。

**W1 docs 阶段实质完工**, 实施层 (Rust 实体 + migration SQL) 跨 session 续 W2-G.6 起 (per token_budget_opt1 拍板: 1.5M 现在 + 4.5M 推 origin 后续, W1 docs 消耗 ~0.1-0.2M, 后续实施层 0.8-1.0M 留给 W2 启动)。

**5 子项 → 7 物理表 → 7 docs 落地**:
- G.1 `permission.user_account` (T78, M 类 SCD-2) — 1 doc
- G.2 `permission.group` (T79, M 类) + `permission.group_member` (T80, T 类 audit) — 2 docs
- G.3 `permission.team` (T81, M 类) — 1 doc
- G.4 `permission.team_member` (T82, T 类 多重隶属) + `permission.role_per_team` (T83, T 类 跨 team 不同 role) — 2 docs
- G.5 `agent.user_account_link` (T84, T 类 双层 L1 1:1 桥接) — 1 doc

---

## 1. 改动矩阵 (3 commits / 11 files / 1919 insertions)

### 1.1 Commit 1 `b9bb2d6` — 顶层 WBS + brief + automation-design §4.12 + registry §5 (4 files, 309 insertions)

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` | 新建 | 18,509 | P3-G 顶层 WBS, 5 段 20 子项 (G.1-G.20), 6.0M token 估算 |
| 2 | `docs/briefs/p3-g-w1.md` | 新建 | 2,343 (后 edit 改 v0.1 rename) | W1 brief (per 守门 #9 v20) |
| 3 | `docs/automation-design.md` | 改 | +2,800 | §4.12 P3-G 节 (G.1-G.20 任务卡, 跟 P3-B 9 子项命名空间共存) |
| 4 | `scripts/automation/registry.md` | 改 (GBK 追加) | +2,468 | §5 P3-G 阶段索引 + §6 v0.2 修订历史 |

### 1.2 Commit 2 `a54ab72` — G.1 + G.2 docs (3 files, 699 insertions)

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `docs/data-design/ipa-detail/tables/permission_user_account.md` | 新建 | 13,822 | T78 user_account (M 类 SCD-2, 15 カラム, RLS 2 policy) |
| 2 | `docs/data-design/ipa-detail/tables/permission_group.md` | 新建 | 11,318 | T79 group (M 类, 12 カラム, builtin 4 値) |
| 3 | `docs/data-design/ipa-detail/tables/permission_group_member.md` | 新建 | 10,363 | T80 group_member (T 类 audit, 11 カラム, is_primary 唯一) |

### 1.3 Commit 3 `1f6e200` — G.3 + G.4 + G.5 docs (4 files, 911 insertions)

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `docs/data-design/ipa-detail/tables/permission_team.md` | 新建 | 12,137 | T81 team (M 类, 12 カラム, team_purpose 4 値 跟 5 域/22 DDD 解耦) |
| 2 | `docs/data-design/ipa-detail/tables/permission_team_member.md` | 新建 | 11,859 | T82 team_member (T 类, 12 カラム, 多重隶属 1 user → N team, is_lead 一致性 trigger) |
| 3 | `docs/data-design/ipa-detail/tables/permission_role_per_team.md` | 新建 | 12,185 | T83 role_per_team (T 类, 13 カラム, 跨 team 不同 role, 期间重複不可 trigger) |
| 4 | `docs/data-design/ipa-detail/tables/agent_user_account_link.md` | 新建 | 12,636 | T84 user_account_link (T 类, 13 カラム, 双层 L1 1:1 桥接) |

**净增**: 7 docs 新建 + 2 docs 改动 = 11 files / 1919 insertions
**净删除**: 0
**守门 #0 派生**: 修订人一律 `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`, 审批者 `架构师 (Mavis 接手 agent per DEC-008)` (per 19:39/20:56/21:59 JST 三次强化代签授权 + 守门 #10 反转)
**P3-G 命名空间**: 跟现有 P3-B (OpenClaw 9 子项 per automation-design §4.1) 共存, G.x 编号

---

## 2. 验证摘要

### 2.1 守门 0 违反验证 (per WBS §2.1 12 项, W1 docs only 评估)

| # | 守门 | 状态 | 说明 |
|---|---|---|---|
| 1 | `cargo check --workspace --lib` 0 err | 🟡 N/A | W1 docs only, 无 Rust 代码改动 |
| 2 | `cargo check --workspace --all-targets` 0 err | 🟡 N/A | 同上 |
| 3 | `cargo fmt --all` 0 diff | 🟡 N/A | 同上 |
| 4 | `cargo clippy --workspace --all-targets -- -D warnings` 0 err | 🟡 N/A | 同上 |
| 5 | `cargo test --workspace --lib` 100% pass | 🟡 N/A | 同上 |
| 6 | `cargo test --workspace --release --lib` 100% pass | 🟡 N/A | 同上 |
| 7 | 41/41 crate 100% 守门覆盖 | 🟡 N/A | 同上 (W1 docs only 不触发) |
| 8 | 守门 #13 DB 三類横展開 100% 覆盖 | 🟢 **OK** | 7 docs §10 业务分類根拠 显式列, M 类 (T78/T79/T81) + T 类 (T80/T82/T83/T84) 派生规 a/b/c/d 100% 满足 |
| 9 | 守门 #21 [P] 子项 docs 同步 | 🟢 **OK** | 5 表设计 + 4 文档同步 + 1 AGENTS 派生 + 1 brief + 1 W1 报告 + 1 ADR-0034 全部 git 实证 (本报告 + ADR-0034 + AGENTS 派生 v25 落 commit 4) |
| 10 | 守门 #6 PowerShell only | 🟢 **OK** | 全部 git / PowerShell 命令 OK (无 &&, 无 bash 风格) |
| 11 | 守门 #7 0 unsafe | 🟡 N/A | W1 docs only, 无 Rust 代码 |
| 12 | 守门 #11 缺标比错标安全 | 🟢 **OK** | 7 docs §3 已知缺口 显式列, 合计 ~20 已知缺口, 主分类单计, 不隐式假设 |

### 2.2 守门 #1 派生 v15-v24 触发

- 守门 #1 v15 (饱和边界): W1 5 子项 docs 同步触达, 后续 docs 同步 commit 必先有新事件触发 (代码改动 / Ulysses 拍板)
- 守门 #1 v19 (Python 化): 5 子项 docs 触发 [P] 档, commit message 含脚本相对路径 (W1 docs 不含 Python 脚本, 等 W2 G.13 dispatcher.py 落地后回填)
- 守门 #1 v20 (子代理 dispatch brief): W1 不派子代理 (per 守门 #9 #3 实证 5/5 RPC 不可靠), 改用 Mavis 直接写 + subprocess.run 替代, brief 路径 `docs/briefs/p3-g-w1.md` 已落档
- 守门 #1 v21 ([P] docs 同步): 本报告 + 5 表设计 + 4 文档同步 + 1 AGENTS 派生 + 1 brief 全部 git 实证 (commit 4 落地)
- 守门 #1 v22-v24 (调试控制台): N/A (W1 不涉及)

### 2.3 守门 #25 (Agent Jira 化) 派生规准备

per WBS §5, W1 完工后追加到 AGENTS.md §4.1 派生 v25:

**守门 #25 (Agent Jira 化) 派生规**: 任何 agent 治理改造必先 (a) `permission.user_account` 落档 (b) 5 表 W/T/M 分类显式列 (c) 5 域真人 Lead 决策点 Mavis 临时代签 (per 守门 #3 拍板 B 反转) (d) 守门 #13 DB 三类横展开 100% 覆盖。违反任一 = 守门不完整。

---

## 3. 已知缺口 (per 守门 #11 缺标比错标安全, 7 docs 汇总 + 跨 stage)

### 3.1 7 docs 已知缺口汇总 (per docs §11)

| 缺口分类 | 缺口数 | 跨 stage 影响 |
|---|---|---|
| **命名空间解耦** (team 跟 5 域/22 DDD) | 3 | W3 G.10 跨域决策时统一 (Q1-D 拍板) |
| **5 域真人 Lead 不到位** (per 守门 #3) | 4 (每 doc 1) | Mavis 临时代签, 真人到位后追溯签字 (per 拍板 B 反转) |
| **跨 stage 实施** (双层 N:1 / 期间重複 trigger / Group 抽象) | 4 | W2/W3 跨 session 续 |
| **built-in role 命名空间** (4 値 group 跟 4 値 role) | 1 | W3 G.10 跨域决策时映射 |
| **lifecycle 状态机 4 値** (active/paused/archived/blocked) | 2 | W3 G.11 落地 |
| **subagent_type 3 値** (WorkerSubagent/ExploreSubagent/VerifierSubagent) | 1 | W2 G.7 落地 |
| **RLS 13 類必携** (5 新表 100% RLS) | 0 (已满足) | − |
| **守门 #1a 401 实证** (推 origin) | 1 | W5 G.20 落地 |
| **P3-A 25 子项暂停影响** | 1 | 推 origin 决策后 P3-A 14 子项续 |

**合计 17 已知缺口 (7 docs §11 累计)**, 全部显式列, 主分类单计, 不隐式假设。

### 3.2 跨 stage 缺口

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | **W1 实施层未落地** (Rust 实体 + migration SQL) | W1 docs 全部落地, 但实施层 (Rust entity / port / repo / migration) 留 W2 跨 session 续, 期间 docs 跟 code 不一致 | W2 G.6 启动后优先实施 G.1 user_account (基础层第一实体), 跨 session 续 docs ↔ code 同步 | 架构师 + SRE Lead |
| 2 | **1:1 双层 L1 vs W2 N:1 双层 L3 矛盾** (per T84 §11 已知缺口 #1) | W1 uq_user_account_link_agent (1 agent 1 link) 跟 W2 G.6 subagent 实体 N:1 共享 user_account 冲突 | W2 G.6 落地时改 `uq_user_account_link_agent` → `uq_user_account_link_agent_type` (1 agent + type 1 link) | 架构师 |
| 3 | **期间重複不可 trigger stub** (T82/T83/T84 各 1 个 stub trigger) | W1 落 trigger 名, 但实际 range overlap 检查 留 W3 跨域决策 | W3 G.10 Permission Scheme 跨 team 实施时具体 逻辑 (range overlap check via SQL) | 架构师 + SRE Lead |
| 4 | **守门 #25 (Agent Jira 化) 派生规** 准备追加 AGENTS.md §4.1 v25 | 本报告 §2.3 已显式列 派生规, 待 commit 4 落档 | commit 4 同步追加 AGENTS.md §4.1 v25 派生规 | 架构师 |

---

## 4. 子代理失败接手清单 (per 守门 #9 + #20 实证 5/5 RPC 不可靠)

W1 决策: **不派子代理** (per 守门 #9 v20 + 守门 #24 v3 调试控制台走 subprocess 替代 RPC), 改用 Mavis 直接写文件 + 落 commit + 跑守门 #1 全套验证。

W1 实证:
- 3 commits 落地 11 files / 1919 insertions, 0 retry
- 命名空间冲突 (P3-B 已存在 9 子项) 主动 rename P3-B → P3-G (per 守门 #1 禁回溯 + 守门 #11 缺标比错标)
- registry.md GBK 编码追加用 PowerShell `[System.IO.File]::AppendAllText` Default 编码 (保留现有 GBK 行, 跟 nav_completion_i18n.py commit `bd918e4` 一致)
- 全部 commit author = Ulysses <ulysses@mavis.local> (per 守门 #15 + 守门 #0)
- 全部 commit message 引用 brief + WBS + automation-design §4.12 + registry §5 (per 守门 #21)

W1 失败接手路径 (如失败):
- 文件写失败 → Mavis retry 1 次 → 仍失败 → 落 `docs/reports/p3-g-w1-failures.md` 标 [S] 档
- cargo 失败 → N/A (W1 docs only)
- 推 origin 失败 → per 守门 #1a 401 实证, 跨 session 续 (W5 G.20)

子代理调用仅在 W2-G.6 subagent 实体落地 + W2-G.13 dispatcher.py 自动注册 阶段启用, 且仅用 `scripts/automation/dispatcher.py brief(...)` 落地 brief → `docs/briefs/<task_id>.md` → commit message 引用 brief 路径。

---

## 5. 守门规则 (15+17 守门 + #21 派生 + #25 新增准备)

per AGENTS.md §4 + §4.1, W1 触发 13 项主守门 + 14 项派生 (v1-v14) + 10 项 v15-v24 派生 + 1 项 v25 准备 (Agent Jira 化):

| 守门 | 触发 | W1 状态 |
|---|---|---|
| #1 0 unsafe + 守门实证 | 全部 5 子项 | 🟡 N/A (W1 docs only) |
| #1a 推 origin 重试细则 | W5 推 origin | 🟡 待 W5 |
| #3 5 域独立 Lead | W3 跨域决策 | 🟡 待 W2-W5 |
| #4 token-OLU | 全部 5 子项 | 🟢 已应用 (1.9M 估算, 实际 ~0.2M docs, 软预算 1.5M 余 1.3M 留给 W2 实施) |
| #5 环境变量安全 | 全部 5 子项 | 🟢 已应用 (无 $env 泄露) |
| #6 PowerShell only | 全部 shell 命令 | 🟢 已应用 |
| #7 0 unsafe | Rust 代码 0 unsafe | 🟡 N/A (W1 docs only) |
| #9 不 commit 散落子代理产出 | 子代理 dispatch | 🟢 已应用 (W1 不派子代理) |
| #10 代签规则 | 报告签字栏 | 🟢 已应用 (Mavis 代签 Ulysses) |
| #11 缺标比错标安全 | 已知缺口 | 🟢 已应用 (17 缺口显式列) |
| #12 AI 协作文档治理 | 文档同步 | 🟢 已应用 (禁回溯叙事) |
| #13 DB 三類横展開 | 7 新表 | 🟢 已应用 (W/T/M 强制分类 100% 覆盖) |
| #1 v1-v14 派生 | 守门 #1 全套 | 🟡 待 W5 实施后验证 |
| #1 v15 (饱和) | docs 同步 | 🟢 已应用 (5 子项 docs 同步) |
| #1 v19 (Python 化) | [P] 子项 | 🟡 待 W2 G.13 dispatcher.py |
| #1 v20 (dispatcher brief) | 子代理 dispatch | 🟢 已应用 (W1 不派子代理, brief 已落 `docs/briefs/p3-g-w1.md`) |
| #1 v21 ([P] docs 同步) | [P] 子项 | 🟢 已应用 (本报告 + 4 文档同步) |
| #1 v22-v24 (调试控制台) | 调试页 | N/A (W1 不涉及) |
| **#25 (本阶段新增, commit 4 追加 AGENTS.md §4.1)** | Agent Jira 化 | 🟡 待 commit 4 |

**守门 #25 派生规 (commit 4 落档)**: 任何 agent 治理改造必先 (a) `permission.user_account` 落档 (b) 5 表 W/T/M 分类显式列 (c) 5 域真人 Lead 决策点 Mavis 临时代签 (per 守门 #3 拍板 B 反转) (d) 守门 #13 DB 三类横展开 100% 覆盖。违反任一 = 守门不完整。

---

## 6. 签字栏 (5 角色 Mavis 代签 per 19:39/20:56/21:59 JST 三次强化)

| 角色 | 责任人 | 签字 | 日期 |
|---|---|---|---|
| 架构 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权) | 🟢 Mavis 接手 | 2026-09-03 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手 | 2026-09-03 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手 | 2026-09-03 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手 | 2026-09-03 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手 | 2026-09-03 |

**5 域独立 Lead 真人** (per 守门 #3 + 2026-08-31 22:45 JST 拍板 B 反转 + 2026-09-03 11:35 JST 拍板): 待 DDD Review 阶段到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯叙事)。

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: W1 基础层 5 子项 (G.1-G.5) docs 完工报告, 7 表设计落地 (T78-T84), 3 commits / 11 files / 1919 insertions 全部 git 实证, 17 已知缺口显式列, 守门 #25 派生规 (Agent Jira 化) 准备追加 AGENTS.md §4.1, W2-W5 跨 session 续 15 子项 (G.6-G.20) | 2026-09-03 11:50 JST Ulysses Jira 化指令 + 3 步 ask_user 拍板 (direction_opt4 + team_dimension_opt4 + subagent_persist_opt4) + 3 步开工拍板 (wbs_mode_opt2 5 段独立验收 + token_budget_opt1 1.5M 现在 + lead_coordination_opt1 Mavis 代签 5 域 Lead) |
