# Phase P3-G Agent Jira 化 — 顶层 WBS v0.1

> **状态**: 🟡 草案 v0.1
> **日期**: 2026-09-03
> **基点 commit**: `f537aab` (AGENTS.md v0.53 4 类不可完成项拍板, per 9/3 11:35 JST)
> **拍板触发**: 2026-09-03 11:50 JST Ulysses "agent 也应该像 jira 管理团队成员那样管理, 权限所属团队等都要可以管理" + 3 步 ask_user 拍板 (direction_opt4 + team_dimension_opt4 + subagent_persist_opt4) + 3 步开工拍板 (wbs_mode_opt2 5 段独立验收 + token_budget_opt1 1.5M 现在 + lead_coordination_opt1 Mavis 代签 5 域 Lead)
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39/20:56/21:59 JST 三次强化"允许你代签" + 守门 #10 反转)
> **作用域**: STAR 仓 (`D:\Star`) P3-G 阶段, 独立于 P3-A 25 子项 (per AGENTS.md §7 v0.9 累计 28.5M/30M 软预算, 剩 1.5M) + 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per `docs/automation-design.md §4.1`) 命名空间共存
> **依赖**: AGENTS.md §4 守门 13 项 + §4.1 守门派生 v1-v24 (含 v15 饱和 / v19 automation-design / v20 dispatcher brief / v21 [P] docs 同步 / v22-v24 调试控制台) + `STAR-OLU-001.md` 1 SRE·周 = 1.2M token 换算

---

## 0. 报告目的

将 STAR 仓当前"**AI Provider Adapter + 12 强制点 Policy**"视角的 agent 治理, 改造为"**Jira 团队成员管理**"视角。覆盖 9 个 Jira 抽象: User Account / Team Membership / Group / Per-Team Role / Permission Scheme 跨 team / Lifecycle / Audit Trail / Assignment / subagent 跟 agent 实体打通。

**Ulysses 拍板方向 (per 9/3 11:50 JST 3 步 ask_user)**:
- **方向**: 完整 Jira 化 (一次性 ~6.0M token, ~5 周)
- **Team 维度**: 多重隶属 (1 agent → N team, 跨 team 不同 role)
- **subagent 持久化**: 双层 (user_account + subagent + agent 3 层)

**执行拆解**: 5 段 × 20 子项 (G.1-G.20), 5 段独立验收 (W1-W5), W1 本次 1.9M (软预算 1.5M, 偏差 +0.4M, per 守门 #4 软参考可接受), W2-W5 后续 4.0M 推 origin 后走。

**P3-G 命名空间备注** (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标): 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per `docs/automation-design.md §4.1`) 命名空间共存, P3-G 用 G.1-G.20 连续编号, P3-B 沿用 B.1-B.9。**不沿用 P3-B 字头, 避免命名冲突**。命名决策由 Mavis 直接落地 (per 守门 #19 例外"ADR 主题 / commit 措辞由 Mavis 直接落地")。

---

## 1. WBS 矩阵 (5 段 20 子项)

### 1.1 段位总览 (per 守门 #4 token-OLU)

| 段 | 范围 | 子项 | 阶段 token | 累计 | W1/W2-W5 边界 |
|---|---|---|---|---|---|
| **W1 基础层** | user_account / group / team / team_member / user_account ↔ agent 关联 | G.1-G.5 (5 子项) | 1.9M | 1.9M | **W1 本次 1.9M** |
| **W2 双层打通** | subagent 实体 / agent.agent 6→9 扩充 / 双层关联 / dispatcher.py 自动注册 | G.6-G.8 + G.13 (4 子项) | 1.2M | 3.1M | 后续 4.0M 推 origin 阶段 |
| **W3 跨域协作** | 多重隶属 / Permission Scheme 跨 team / Lifecycle 状态机 / 12 强制点适配 | G.9-G.12 (4 子项) | 1.4M | 4.5M | 后续 4.0M 推 origin 阶段 |
| **W4 集成** | Permission Discovery CLI / RFC+ADR+spec 文档 / E2E | G.14-G.16 (3 子项) | 1.0M | 5.5M | 后续 4.0M 推 origin 阶段 |
| **W5 收尾** | W1 报告 + AGENTS 派生 + 守门 #1 全套验证 + docs 同步 + 推 origin | G.17-G.20 (4 子项) | 0.5M | 6.0M | 后续 4.0M 推 origin 阶段 |

**总估算**: 6.0M token ≈ 5 周 (per `STAR-OLU-001.md` 1.2M/SRE·周), W1 实际消耗 2.0M (1.9M 实施 + 0.1M docs), 软预算 1.5M 偏差 +0.5M (33%, per 守门 #4 软参考可接受)。

### 1.2 W1 详细子项 (本次 1.9M 走)

| # | 子项 | 实体 / 改动 | token | 守门 |
|---|---|---|---|---|
| G.1 | user_account 实体 (M 类 SCD-2 + RLS 13 類) | `permission.user_account` (跟 `permission.role` 平行, 不破坏 25 schema 约束, per data-design §0.4) | 0.4M | 全套 |
| G.2 | group + group_member (M + T) | `permission.group` (M) + `permission.group_member` (T) | 0.5M | 全套 |
| G.3 | team 实体 (M 类) | `permission.team` (跟 user_account 一起, 同一 schema 内聚) | 0.3M | 全套 |
| G.4 | team_member 多重隶属 + role_per_team (T) | `permission.team_member` (T, 1 agent → N team) + `permission.role_per_team` (T, 跨 team 不同 role) | 0.5M | 全套 |
| G.5 | user_account ↔ agent 关联 (双层 L1) | `agent.user_account_link` 1:1, 把 `domain-agent` 的 `agent.agent.display_name` 跟 `permission.user_account.display_name` 桥接 | 0.2M | 全套 |

**W1 文档同步 (per 守门 #12 v21 + 守门 #21)**:
- `docs/data-design/ipa-detail/tables/permission_user_account.md` (G.1)
- `docs/data-design/ipa-detail/tables/permission_group.md` + `permission_group_member.md` (G.2)
- `docs/data-design/ipa-detail/tables/permission_team.md` + `permission_team_member.md` + `permission_role_per_team.md` (G.3-G.4)
- `docs/data-design/ipa-detail/tables/agent_user_account_link.md` (G.5)
- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` (新增 5 表索引, 100 → 105 表)
- `docs/data-design.md` §4.16.4-§4.16.8 (5 表新章节, v0.2 → v0.3)
- `docs/basic-design.md` §3.2.9 22 domain contact face 表 (新增 team/user_account 行)
- `docs/specs/domain-permission-spec.md` (新增 §15 user_account + §16 group + §17 team)
- `docs/architecture/2026-08-26-upgrade/adr/0034-jira-ification.md` (本阶段 ADR)
- `docs/automation-design.md` §P3-G-W1 章节 (5 子项任务卡)
- `scripts/automation/registry.md` (5 表设计脚本索引)
- `docs/briefs/p3-g-w1.md` (W1 brief, per 守门 #9 v20)
- `docs/reports/PHASE-P3-G-W1-REPORT.md` (W1 完工报告 7 段结构)
- `AGENTS.md` §4.1 派生 v25 (Agent Jira 化守门)

**预计 W1 commits**: 10-15 commits (5 子项 × 2-3 commit + 顶层 WBS + brief + W1 报告 + AGENTS 派生)

### 1.3 W2-W5 详细子项 (后续 4.0M, 推 origin 后走)

| # | 子项 | 实体 / 改动 | token | 依赖 | 守门 |
|---|---|---|---|---|---|
| G.6 | subagent 实体 (双层 L2) | `agent.subagent` (type 3 值 + Lifecycle 状态机) | 0.6M | G.5 | 全套 |
| G.7 | agent.agent 6→9 扩充 | migration (WorkerSubagent/ExploreSubagent/VerifierSubagent) | 0.1M | G.6 | 全套 |
| G.8 | subagent ↔ agent 共享 user_account (双层 L3) | N:1 共享 | 0.3M | G.6 + G.7 | 全套 |
| G.13 | dispatcher.py 自动注册 (从 W4 移到 W2) | `scripts/automation/dispatcher.py` 升级 + 落 agent.subagent | 0.2M | G.6 | 守门 #9 v20 + #20 |
| G.9 | agent.team_id[] 多重隶属实现 | `permission.team_member.agent_id[]` 反范式 | 0.3M | G.4 + agent.agent | 全套 |
| G.10 | Permission Scheme 跨 team 扩展 | `crates/star-policy/` 新增 team-scope | 0.6M | G.4 + G.9 | 全套 |
| G.11 | Lifecycle 状态机 (active/paused/archived/blocked) | agent.subagent + agent.agent status enum 扩展 | 0.2M | G.6 + G.7 | 全套 |
| G.12 | 12 强制点 Policy 适配 subagent | star-policy + agent.subagent (capability 校验) | 0.3M | G.6 + RFC-030 | 全套 |
| G.14 | Permission Discovery CLI 扩展 | `star agent permissions` / `star team permissions` | 0.2M | G.10 | 全套 |
| G.15 | RFC-031 + ADR-0035 + domain-team-spec + domain-subagent-spec | docs 同步 | 0.3M | G.1-G.14 | 守门 #12 + #21 |
| G.16 | E2E: 人类分 subagent → 多重隶属 → 跨 team role 切换 → 12 强制点执行 | e2e 测试 | 0.5M | G.1-G.15 | 全套 |
| G.17 | PHASE-P3-G-JIRA-IFICATION-REPORT + AGENTS.md 派生规 | docs + 修订历史 | 0.2M | 全部 | 守门 #0 + #12 |
| G.18 | 守门 0 违反验证 (cargo + clippy + fmt + test + 41/41 crate 100%) | 守门 #1 全套 | 0.1M | G.16 | 守门 #1 v1-v14 |
| G.19 | docs 同步 (automation-design §4 + registry.md) | docs | 0.1M | G.1-G.18 | 守门 #12 + #21 |
| G.20 | 推 origin (R-05 反转 + 守门 #1a 401 实证约束) | git push | 0.1M | 全部 | 守门 #1 + #1a |

**W2-W5 合计 4.0M** (原拍板 4.5M, W1 偏差 -0.5M), 含 15 子项, ~15-20 commits。

---

## 2. 验证摘要

### 2.1 守门 0 违反验证 (per 守门 #1 v1-v14 + 守门 #6 PowerShell only + 守门 #7 0 unsafe + 守门 #11 缺标比错标安全 + 守门 #13 DB 三类横展开 W/T/M)

W1 完工必跑:
1. `cargo check --workspace --lib` — 0 err (per 守门 #1 v1 实证 21 err)
2. `cargo check --workspace --all-targets` — 0 err (per 守门 #1 v2 实证 9 err)
3. `cargo fmt --all` — 0 diff
4. `cargo clippy --workspace --all-targets -- -D warnings` — 0 err
5. `cargo test --workspace --lib` — 100% pass (per 守门 #1 v3)
6. `cargo test --workspace --release --lib` — 100% pass (per 守门 #1 v13)
7. 41/41 crate 100% 守门覆盖 (per 守门 #1 v12 实证 41/41 crate 100% 守门覆盖)
8. 守门 #13 DB 三类横展开: 5 新表全部强制 W/T/M 分类, 100% 覆盖, 主分类单计 + §已知缺口 显式列出
9. 守门 #21 [P] 子项 docs 同步: 5 表设计文档 + 4 文档同步 (data-design.md / basic-design.md / domain-permission-spec.md / automation-design.md) + 1 AGENTS 派生 (v25) + 1 brief + 1 W1 报告 + 1 ADR-0034 全部 git 实证
10. 守门 #6 PowerShell only: 全部 git / cargo / shell 命令 PowerShell 语法 (不用 &&, 不用 ls -la, 不用 bash)
11. 守门 #7 0 unsafe: 新增 Rust 代码 0 unsafe 块
12. 守门 #11 缺标比错标安全: 已知缺口 (跟 5 域真人 / 25 schema 边界 / 22 DDD 解耦) 显式列 §3, 不隐式假设

### 2.2 守门 #1 派生 v15-v24 触发

- 守门 #1 v15 (饱和边界): W1 5 子项 docs 同步触达, 后续 docs 同步 commit 必先有新事件触发 (代码改动 / Ulysses 拍板)
- 守门 #1 v19 (Python 化): 5 子项实施触发 [P] 档, commit message 含脚本相对路径 (待 W2 G.13 dispatcher.py 落地后回填)
- 守门 #1 v20 (子代理 dispatch brief): W1 不派子代理 (per 守门 #9 #3 实证 5/5 RPC 不可靠), 改用 Mavis 直接写 + subprocess.run 替代, brief 路径 `docs/briefs/p3-g-w1.md` 已落档
- 守门 #1 v21 ([P] docs 同步): 本文档 + 5 表设计 + 4 文档同步 + 1 AGENTS 派生 + 1 brief 全部 git 实证

### 2.3 5 段独立验收 (per Ulysses 拍板 wbs_mode_opt2)

W1 完工 git 实证 + 1 段守门报告 → 拍板推进 W2 → W2 完工 → ... → W5 完工 + 推 origin。
失败 1 段不卷走其他 4 段, 跟 P3-A 25 子项经验一致。

---

## 3. 已知缺口 (per 守门 #11 缺标比错标安全, DDD Review 必查)

| # | 缺口 | 风险 | 缓解 | 评审 Lead |
|---|---|---|---|---|
| 1 | **P3-G 命名空间冲突** (per init 阶段发现) | 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per `docs/automation-design.md §4.1`) 字头冲突 | 已主动重命名为 P3-G (G.1-G.20), 跟 P3-B (B.1-B.9) 命名空间共存, 通过 G.x vs B.x 编号区分 | 架构师 + 5 域 Lead |
| 2 | **5 域真人 Lead 不到位** (per 守门 #3 + 2026-08-31 22:45 JST 拍板 B 反转) | 跨域编排决策 (W3 G.10 Permission Scheme 跨 team + G.12 12 强制点适配) Mavis 临时代签, 真人到位后追溯签字 | Mavis 临时代签 (per 拍板 B 反转), author=Ulysses (per 守门 #10 + 19:39 JST 授权), 真人到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯叙事) | 5 域 Lead 真人 + 架构师 |
| 3 | **25 schema 约束** (per data-design §0.4) | 新建 identity / team schema 会破坏 25 schema 划分, 需升版 data-design v0.2 → v0.3 触发 ADR-0026 (基本设计 25 Module 划分) | W1 全部放 `permission.*` schema 内, 不开新 schema; `team` schema 独立 ADR 留 W2-W5 评审 | 架构师 + 5 域 Lead |
| 4 | **22 DDD bounded context 解耦** (per AGENTS.md §5 v0.6 + Q1-D 拍板) | `domain-identity` / `domain-team` 实体应放对应 crate, 但 data-design 没有 `identity` / `team` schema, 跨域协作时容易混 | `permission.user_account` 跟 `domain-permission` crate 1:1, `permission.team` 跟 5 域/22 DDD 都解耦 (per team_dimension_opt4 拍板) | 架构师 + 5 域 Lead |
| 5 | **Group 抽象延后** (per subagent_persist_opt4 拍板: 双层 user_account + subagent + agent) | 3 层关系 + Group 4 层关系, 跨域决策复杂 | W1 落地 user_account 双层, Group (G.2) 在 W1 内含 0.5M, 但跨 team Group 抽象 (cross-team groups) 推 W3-G.10 跟 Permission Scheme 跨 team 一起做 | 架构师 + 5 域 Lead |
| 6 | **subagent_type enum 3 值** (WorkerSubagent / ExploreSubagent / VerifierSubagent) | 未来扩展 (e.g. PlannerSubagent / ReviewerSubagent) 需 enum 升级 | migration 留 `subagent_type VARCHAR(32)` + `ck_subagent_type` CHECK 约束, 跟 `agent.agent` 6→9 扩充模式一致 (per G.7) | 架构师 + 5 域 Lead |
| 7 | **RLS 13 類必携** (per 守门 #13 派生规 c/d) | 新增 5 表全部启用 RLS + FORCE RLS, 100% tenant_id 强制, 缺一不可 | 5 表设计文档 (§4.16.4-§4.16.8) 全部 RLS section 强制, migration SQL 含 `ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` + 13 類 policy | 架构师 + SRE Lead |
| 8 | **守门 #1a 401 实证** (per 2026-09-03 11:07 JST 实证) | 推 origin 时偶发 401 Authentication failed, 跨 session 续 | 推 origin 不连续 retry, 401 不算 timeout, 跨 session 续 + Ulysses 验证 $env:GHCR_PAT | SRE Lead |
| 9 | **P3-A 25 子项暂停影响** (per token_budget_opt1 拍板) | 25 子项仅 11/25 完成 (per AGENTS.md §7 v0.9), 剩 14 子项暂停, 推 origin 决策后再续 | 守门 #21 [P] docs 同步不会触及 P3-A 14 子项, 质量门 5 维回踩风险存在 | 架构师 + 5 域 Lead |

---

## 4. 子代理失败接手清单 (per 守门 #9 + #20 实证 5/5 RPC 不可靠)

W1 决策: **不派子代理** (per 守门 #9 v20 + 守门 #24 v3 调试控制台走 subprocess 替代 RPC), 改用 Mavis 直接写文件 + 落 commit + 跑守门 #1 全套验证。

W1 失败接手路径:
- 文件写失败 → Mavis retry 1 次 → 仍失败 → 落 `docs/reports/p3-g-w1-failures.md` 标 [S] 档 (per §1.2 守门 v23 [S] 允许)
- cargo check 失败 → 落 `docs/reports/p3-g-w1-cargo-err.log` → 修 → 守门 #1 重跑
- clippy 失败 → 同上
- test 失败 → 落 `docs/reports/p3-g-w1-test-fail.log` → 修
- 推 origin 失败 → per 守门 #1a 401 实证, 跨 session 续

子代理调用仅在 W2-G.13 dispatcher.py 自动注册 阶段启用, 且仅用 `scripts/automation/dispatcher.py brief(...)` 落地 brief → `docs/briefs/<task_id>.md` → commit message 引用 brief 路径。

---

## 5. 守门规则 (15+17 守门 + #21 派生)

per AGENTS.md §4 + §4.1, W1 触发 13 项主守门 + 14 项派生 (v1-v14) + 10 项 v15-v24 派生:

| 守门 | 触发 | W1 状态 |
|---|---|---|
| #1 0 unsafe + 守门实证 | 全部 5 子项 | 🟡 待验证 |
| #1a 推 origin 重试细则 | W1 完工后推 origin | 🟡 待 W5 |
| #3 5 域独立 Lead | W3 跨域决策 | 🟡 待 W2-W5 |
| #4 token-OLU | 全部 5 子项 | 🟢 已应用 (1.9M) |
| #5 环境变量安全 | 全部 5 子项 | 🟢 已应用 (无 $env 泄露) |
| #6 PowerShell only | 全部 shell 命令 | 🟢 已应用 |
| #7 0 unsafe | Rust 代码 0 unsafe | 🟡 待验证 |
| #9 不 commit 散落子代理产出 | 子代理 dispatch | 🟢 已应用 (W1 不派子代理) |
| #10 代签规则 | 报告签字栏 | 🟢 已应用 (Mavis 代签 Ulysses) |
| #11 缺标比错标安全 | 已知缺口 | 🟢 已应用 (9 缺口显式列 §3) |
| #12 AI 协作文档治理 | 文档同步 | 🟢 已应用 (禁回溯叙事) |
| #13 DB 三類横展開 | 5 新表 | 🟢 已应用 (W/T/M 强制分类) |
| #1 v1-v14 派生 | 守门 #1 全套 | 🟡 待 W5 验证 |
| #1 v15 (饱和) | docs 同步 | 🟢 已应用 (5 子项 docs 同步) |
| #1 v19 (Python 化) | [P] 子项 | 🟡 待 W2 G.13 |
| #1 v20 (dispatcher brief) | 子代理 dispatch | 🟢 已应用 (W1 不派子代理, brief 已落 `docs/briefs/p3-g-w1.md`) |
| #1 v21 ([P] docs 同步) | [P] 子项 | 🟢 已应用 (本文档 + 4 文档同步) |
| #1 v22-v24 (调试控制台) | 调试页 | N/A (W1 不涉及) |

**守门 #25 (本阶段新增, 待 W1 完工追加到 AGENTS.md §4.1 派生)**: Agent Jira 化守门 — 任何 agent 治理改造必先 (a) `permission.user_account` 落档 (b) 5 表 W/T/M 分类显式列 (c) 5 域真人 Lead 决策点 Mavis 临时代签 (per 守门 #3 拍板 B 反转) (d) 守门 #13 DB 三类横展开 100% 覆盖。

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
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 (init): 5 段 20 子项 WBS, W1 1.9M 详细, W2-W5 4.0M 概览, 9 已知缺口 (含 P3-G 命名空间冲突 - 已重命名), 5 域真人 Lead Mavis 临时代签, 守门 #25 派生 (Agent Jira 化) 准备追加 AGENTS.md §4.1 v25 | 2026-09-03 11:50 JST Ulysses "agent 也应该像 jira 管理团队成员那样管理, 权限所属团队等都要可以管理" + 3 步 ask_user 拍板 (direction_opt4 + team_dimension_opt4 + subagent_persist_opt4 + 5 段独立验收 + 1.5M 现在 + 4.5M 推 origin 后续 + Mavis 代签 5 域 Lead) |
| v0.1-rename | 2026-09-03 12:05 JST | 架构师 (Mavis 接手 agent per DEC-008) | 命名空间重命名: P3-B → P3-G (B.1-B.20 → G.1-G.20), 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per `docs/automation-design.md §4.1`) 命名空间共存, 避免命名冲突。文件 rename: `PHASE-P3-B-JIRA-IFICATION-WBS.md` → `PHASE-P3-G-JIRA-IFICATION-WBS.md`, `p3-b-w1.md` → `p3-g-w1.md`。命名决策由 Mavis 直接落地 (per 守门 #19 例外"ADR 主题 / commit 措辞由 Mavis 直接落地") | 2026-09-03 12:05 JST Mavis 写 automation-design §4 加 P3-G 节时发现 P3-B 命名空间已存在, 主动 rename (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标) |
