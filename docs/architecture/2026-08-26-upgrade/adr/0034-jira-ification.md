# ADR-0034: Agent Jira 化 (P3-G 阶段)

> **状态**: 🟢 Accepted v0.1
> **日期**: 2026-09-03
> **作者**: Mavis (Star 架构师)
> **拍板**: 2026-09-03 11:50 JST Ulysses "agent 也应该像 jira 管理团队成员那样管理, 权限所属团队等都要可以管理" + 3 步 ask_user 拍板 (direction_opt4 完整 Jira 化 + team_dimension_opt4 多重隶属 + subagent_persist_opt4 双层)
> **相关 RFC**: 后续 RFC-031 (per W4 G.15) — 本 ADR 是 RFC 化前的初版
> **相关 ADR**: 0021 (Zero Vendor Cooperation) / 0026 (STAR AI 兼容 5 通道) / 0027 (STAR IDE 网关) / 0030 (Lease + Heartbeat + Resume) / 0032 (MCP Transport stdio) / 0033 (Agent Co-signing Policy)
> **依赖**: `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` v0.1 + `docs/briefs/p3-g-w1.md` + `docs/reports/PHASE-P3-G-W1-REPORT.md` v0.1

---

## 摘要

本 ADR 提议将 STAR 仓当前"**AI Provider Adapter + 12 强制点 Policy**"视角的 agent 治理, 改造为"**Jira 团队成员管理**"视角, 覆盖 9 个 Jira 抽象: User Account / Team Membership / Group / Per-Team Role / Permission Scheme 跨 team / Lifecycle / Audit Trail / Assignment / subagent 跟 agent 实体打通。

**改造分 5 段 20 子项 (G.1-G.20), 6.0M token ≈ 5 周** (per 守门 #4 token-OLU, 1 SRE·周 = 1.2M token per `STAR-OLU-001.md`), W1 (1.9M, 本次) 基础层 + W2 (1.2M) 双层打通 + W3 (1.4M) 跨域协作 + W4 (1.0M) 集成 + W5 (0.5M) 收尾。

**W1 已实质落地** (per `docs/reports/PHASE-P3-G-W1-REPORT.md` v0.1): 7 表设计 (T78-T84) + 顶层 WBS + brief + automation-design §4.12 + registry §5 全部 git 实证, 3 commits / 11 files / 1919 insertions。

---

## 动机

### 背景

STAR 仓当前 agent 治理 (`docs/specs/domain-agent-spec.md` v0.1 + `docs/rfcs/rfc-030-agent-policy-enforcement.md` v0.1 + `docs/architecture/2026-08-26-upgrade/spec/resources/05-agent-permission-model.md` v0.1) 核心是"**AI Provider Adapter + 12 强制点 Policy**"视角:
- `agent.agent` (T77) 注册 6 値 AI Provider (Codex / ClaudeCode / GeminiCLI / OpenAICompatible / Local / Future)
- `agent.agent_session` 14 状态机 + 12 强制点 (Repository / Worktree / Path / Tool / Network / Secret / Runtime / Context / Change Scope / Review / Test / Approval)
- `agent.agent_policy` 12 字段值对象
- `agent.agent_policy_template` 模板
- 内置 4 値 Role (tenant_admin / project_admin / developer / viewer) 绑在 tenant / project 层级
- AgentPermission L0-L7 八级 (crates/star-agent/src/permission.rs)
- subagent 治理 (本地 mavis worker/explore/verifier) 走 `scripts/automation/dispatcher.py` + brief, 没跟 `agent.agent` 实体打通 (per 守门 #9 实证 RPC 不可靠)

**现状 9 个 Jira 抽象缺口** (per WBS §0):
| Jira 抽象 | STAR 现状 | 缺口 |
|---|---|---|
| User Account (login/email/avatar) | agent.agent 只有 display_name | 🔴 缺 |
| Team Membership (1 agent → N team) | 只有 tenant_id 单一归属 | 🔴 缺 |
| Group (跨 team group) | 无 | 🔴 缺 |
| Per-Team Role (每队角色独立) | 4 role 绑 tenant/project, 非 per-team | 🟡 |
| Lifecycle (active/paused/archived) | 仅 is_enabled boolean | 🟡 |
| Audit Trail | domain-audit + Policy Violation 已有 | 🟢 |
| Assignment | worktree.assigned_agent_id 已有 | 🟢 |
| Permission Scheme 跨 Project | star-policy/ 已有, 不支持 team-scope | 🟡 |
| subagent 跟 agent 实体打通 | 完全分离 | 🔴 |

### 现状问题

1. **subagent (本地 mavis worker/explore/verifier) 不可观测** (per 守门 #9 实证 10/10 ERR_CONNECTION_CLOSED but status="succeeded")
2. **agent.agent.display_name 跟 人类 user 命名空间不区分** (Human / Subagent / ServiceAccount 命名一致)
3. **跨域协作时 agent 隶属不明确** (5 域真人 Lead vs 22 DDD bounded context vs team 维度)
4. **Permission Scheme 跨 team 不可用** (现有 permission_scheme (T51) 绑 project_id, team-scope 未实现)
5. **Lifecycle 状态机不完整** (active/paused/archived/blocked 4 値未落地, 仅 is_enabled boolean)

### 解决目标

1. **User Account 抽象** (T78 `permission.user_account`): login/email/avatar 字段, 3 値 account_type (Human/Subagent/ServiceAccount)
2. **Team 抽象** (T81 `permission.team`): team_purpose 4 値 (Engineering/Operations/Review/CrossFunctional), 跟 5 域/22 DDD 解耦 (per Q1-D 拍板 + team_dimension_opt4 拍板)
3. **多重隶属** (T82 `permission.team_member`): 1 user → N team, 跨 team 不同 role (per team_dimension_opt4 拍板)
4. **Group 抽象** (T79 `permission.group` + T80 `permission.group_member`): 跨 team group, 4 値 builtin (jira-administrators/jira-users/platform-operators/all-users)
5. **双层 L1 桥接** (T84 `agent.user_account_link`): agent.agent ↔ user_account 1:1 桥接 (per subagent_persist_opt4 拍板)
6. **Lifecycle 状态机** (4 値 active/paused/archived/blocked): T78-T84 全部 lifecycle_status 字段
7. **RLS 13 類必携**: 7 新表全部启用 RLS + FORCE RLS + tenant_id 强制 + lifecycle filter (per 守门 #13 DB 三類横展開 派生规 c/d)
8. **跨 stage 实施**: W1 docs → W2 Rust 实体 + migration → W3 跨域协作 (Permission Scheme 跨 team) → W4 RFC-031 + ADR-0035 + E2E → W5 收尾 (报告 + AGENTS 派生 + 守门 #1 全套 + 推 origin)

---

## 详细设计

### 决策 (Decision)

**采用方案 A: 完整 Jira 化** (per direction_opt4 拍板)

5 段 × 4 层 × 20 子项, 总 6.0M token ≈ 5 周:

| 层 | 子项 | 范围 | token |
|---|---|---|---|
| L1 Identity (基础) | G.1-G.5 | user_account / group / team / team_member / role_per_team / user_account_link | 1.9M (W1, 已落地) |
| L2 Subagent (双层) | G.6-G.8 + G.13 | subagent 实体 / agent.agent 6→9 扩充 / 双层关联 / dispatcher.py | 1.2M (W2) |
| L3 Cross-domain (跨域) | G.9-G.12 | 多重隶属 / Permission Scheme 跨 team / Lifecycle / 12 强制点 | 1.4M (W3) |
| L4 Integration (集成) | G.14-G.16 | Permission Discovery CLI / RFC-031 + ADR-0035 + spec / E2E | 1.0M (W4) |
| L5 Wrap-up (收尾) | G.17-G.20 | 报告 + AGENTS 派生 + 守门 #1 全套 + 推 origin | 0.5M (W5) |
| **合计** | **G.1-G.20** | **完整 Jira 化** | **6.0M ≈ 5 周** |

### 替代方案 (Alternatives Considered)

#### 方案 A: 完整 Jira 化 (选定)

- **描述**: 5 段 20 子项, 6.0M token 一次性
- **优点**:
  - 9 个 Jira 抽象全覆盖, 跟 Jira 团队成员管理对齐
  - 双层 (user_account + subagent + agent) 灵活, 未来扩展 (人类 + AI 混部) 友好
  - 多重隶属 + 跨 team 不同 role 跟守门 #3 5 域独立 Lead + 跨域编排 (Saga orchestrator) 完美匹配
- **缺点**:
  - 实施成本高, 6.0M token 需 5 周
  - 跟现有 permission 域 5 角色 + 12 强制点 Policy 集成复杂
  - 5 域真人 Lead 不到位, Mavis 临时代签 (per 守门 #3 拍板 B 反转)
- **本设计选定** (per direction_opt4 拍板)

#### 方案 B: 仅 subagent 打通

- **描述**: 1.0M token, 0.8 周, 仅把本地 mavis worker/explore/verifier 持久化到 agent.agent
- **优点**: 最小落地, 缓解守门 #9 实证 RPC 不可靠
- **缺点**: Team 维度 / User Account / Group 抽象均不做, 跟 Jira 团队成员管理差距大
- **拒绝理由**: 跟 Ulysses "agent 也应该像 jira 管理团队成员那样管理" 指令不匹配, 缺口 7/9

#### 方案 C: 仅 Team 维度

- **描述**: 2.5M token, 2.1 周, 仅新增 team.team + team_member + role_per_team, subagent 延后
- **优点**: Team 维度落地, 多重隶属 + 跨 team 不同 role 满足
- **缺点**: User Account 抽象不做, subagent 仍不可观测 (守门 #9 风险)
- **拒绝理由**: User Account + subagent 打通 是 Ulysses 拍板双层 (subagent_persist_opt4) 的核心, 缺一不可

#### 方案 D: 不做, 维持现状

- **描述**: 0M token, 0 周, 等 P3-B 阶段合入
- **优点**: 0 风险
- **缺点**: 9 缺口全部保留, 守门 #9 风险持续
- **拒绝理由**: Ulysses 明确指令"agent 也应该像 jira 管理团队成员那样管理", 跟 0 改动矛盾

---

## 后果

### 正面后果 (Positive Consequences)

1. **9 个 Jira 抽象全覆盖**, 跟 Jira 团队成员管理对齐, 未来扩展 (e.g. 人类 + AI 混部) 灵活
2. **双层 (user_account + subagent + agent)** 灵活架构, 跨 stage 续 W2-G.6 subagent 实体落地后 1 user_account 共享 N subagent
3. **多重隶属 + 跨 team 不同 role** 跟守门 #3 5 域独立 Lead + 跨域编排 完美匹配
4. **Lifecycle 状态机 4 値** (active/paused/archived/blocked) 跟现有 is_enabled 兼容, 升级路径明确
5. **RLS 13 類必携 100% 覆盖**, 守门 #13 DB 三類横展開 派生规 c/d 满足
6. **Mavis 临时代签 5 域 Lead 决策** (per 守门 #3 拍板 B 反转), 真人到位后追溯签字, 责任矩阵清晰

### 负面后果 (Negative Consequences / Trade-offs)

1. **实施成本高**: 6.0M token ≈ 5 周, 跟 P3-A 25 子项累计 28.5M + P3-G 6.0M = 34.5M, 软预算 30M 超 4.5M (per token_budget_opt1 拍板: 1.5M 现在 + 4.5M 推 origin 后续)
2. **5 域真人 Lead 不到位**: Mavis 临时代签 5 域 Lead 决策, 真人到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯叙事)
3. **25 schema 约束** (per data-design §0.4): 新建 identity / team schema 会破坏 25 schema 划分, W1 全部放 `permission.*` schema 内, 不开新 schema (per W1 决策); `team` schema 独立 ADR 留 W2-W5 评审
4. **22 DDD bounded context 解耦** (per AGENTS.md §5 + Q1-D 拍板): `domain-identity` / `domain-team` 实体应放对应 crate, W1 跟 `domain-permission` crate 1:1, 跟 22 DDD 解耦 (per team_dimension_opt4 拍板)
5. **双层 L1 vs W2 N:1 矛盾** (per T84 §11 已知缺口 #1): W1 1:1 强制 (uq_user_account_link_agent) 跟 W2 G.6 subagent 实体 N:1 共享 user_account 冲突, W2 实施时改 `uq_user_account_link_agent` → `uq_user_account_link_agent_type`
6. **守门 #1a 401 实证** (per 2026-09-03 11:07 JST 实证): 推 origin 时偶发 401, 跨 session 续 + Ulysses 验证 $env:GHCR_PAT

### 风险 (Risks)

| ID | 风险 | 影响 | 缓解措施 |
|---|---|---|---|
| RISK-G-1 | P3-G 命名空间冲突 | Low | 已主动重命名 P3-B → P3-G, 跟现有 P3-B (OpenClaw 9 子项) 命名空间共存 |
| RISK-G-2 | 5 域真人 Lead 不到位 | Medium | Mavis 临时代签 (per 守门 #3 拍板 B 反转), 真人到位后追溯签字 |
| RISK-G-3 | 25 schema 约束 + 22 DDD 解耦 | Medium | W1 docs 全部放 `permission.*` schema, 跟 22 DDD 解耦, `team` schema 独立 ADR 留 W2-W5 评审 |
| RISK-G-4 | subagent RPC 不可靠 (守门 #9 实证) | High | W1 不派子代理, 改用 Mavis 直接写 + subprocess.run 替代, brief 必先落地 (per 守门 #9 v20) |
| RISK-G-5 | 双层 L1 vs W2 N:1 矛盾 | Medium | W1 落档后 T84 §11 已知缺口 #1 显式列, W2 G.6 实施时改 `uq_user_account_link_agent` → `uq_user_account_link_agent_type` |
| RISK-G-6 | 推 origin 401 Authentication failed | Medium | 守门 #1a 401 实证约束, 跨 session 续 + Ulysses 验证 $env:GHCR_PAT |
| RISK-G-7 | 期间重複不可 trigger stub | Low | T82/T83/T84 落 trigger 名, W3 G.10 实施具体 逻辑 (range overlap check via SQL) |
| RISK-G-8 | P3-A 25 子项暂停影响 | Medium | 25 子项仅 11/25 完成, 剩 14 子项暂停, 推 origin 决策后再续; 守门 #21 [P] docs 同步不会触及 P3-A 14 子项, 质量门 5 维回踩风险 |

---

## 实施计划

### 依赖

- 上游: ADR-0021 (Zero Vendor Cooperation) / ADR-0026 (STAR AI 兼容) / ADR-0030 (Lease + Heartbeat + Resume) / ADR-0032 (MCP Transport stdio) / ADR-0033 (Agent Co-signing Policy)
- 上游: 守门 #3 (5 域独立 Lead) / 守门 #9 (子代理 dispatch brief) / 守门 #13 (DB 三類横展開) / 守门 #21 ([P] docs 同步)
- 平级: ADR-0035 (P3-G 阶段 W3-G.10 Permission Scheme 跨 team 落地, 待续) / RFC-031 (P3-G 阶段 W4-G.15 落地, 待续)
- 下游: 22 domain-* crate 实施层 (W2 跨 session 续)
- PoC 验证: P3-G-W1 5 子项 docs 已落地, 实施层 Rust entity / port / repo / migration 待 W2 续

### 阶段

1. **W1 (1.9M, 本次已实质完工)**: user_account / group / team / team_member / role_per_team / user_account_link 7 表设计落地, 守门 #13 100% 覆盖, 守门 #21 [P] docs 同步 落地
2. **W2 (1.2M, 跨 session 续)**: subagent 实体 (G.6) + agent.agent 6→9 扩充 (G.7) + 双层 L3 关联 (G.8) + dispatcher.py 自动注册 (G.13)
3. **W3 (1.4M, 跨 session 续)**: 多重隶属 实施 (G.9) + Permission Scheme 跨 team (G.10, ADR-0035) + Lifecycle 状态机 (G.11) + 12 强制点 Policy 适配 subagent (G.12)
4. **W4 (1.0M, 跨 session 续)**: Permission Discovery CLI (G.14) + RFC-031 + ADR-0035 + domain-team-spec + domain-subagent-spec (G.15) + E2E (G.16)
5. **W5 (0.5M, 跨 session 续)**: PHASE-P3-G-JIRA-IFICATION-REPORT + AGENTS 派生 (G.17) + 守门 #1 全套验证 (G.18) + docs 同步 (G.19) + 推 origin (G.20)

### 回滚策略

如果 P3-G 阶段在 W2/W3 阶段遇到严重问题, 降级方案:

1. **W2 降级**: subagent 实体 暂缓, 改 W3-G.10 Permission Scheme 跨 team 先实施
2. **W3 降级**: 跨域决策 (B.10/B.12) 推迟, 仅实施 Lifecycle 状态机 (B.11) + 多重隶属 (B.9)
3. **W4 降级**: E2E 推迟, 仅 CLI + RFC/ADR/spec
4. **W5 降级**: 推 origin 推迟, 仅 docs 同步 + 守门验证

回滚触发条件: 守门 #1 全套验证 (cargo + clippy + fmt + test) 任一阶段 0 错失败, 或 5 域真人 Lead 拍板否决决策。

---

## 待决问题 (Open Questions)

1. **5 域真人 Lead 寻访**: per 守门 #3, 真人到位后追溯签字, 不沿用代签决策
2. **`team` schema 独立 ADR**: 25 schema 约束, W1 全部放 `permission.*` schema, `team` schema 独立 ADR 留 W2-W5 评审
3. **双层 L1 vs W2 N:1 矛盾**: W2 G.6 subagent 实体实施时改 `uq_user_account_link_agent` → `uq_user_account_link_agent_type`
4. **Group 跨 team 抽象** (W3-G.10): 跟 Permission Scheme 跨 team 一起实施
5. **subagent_type 3 値** (WorkerSubagent / ExploreSubagent / VerifierSubagent): W2 G.7 落地, future 扩展 (PlannerSubagent / ReviewerSubagent) 留 enum upgrade

---

## 评审检查清单 (Code Review Checklist)

1. [x] 9 个 Jira 抽象全覆盖 (per §0 现状缺口表)
2. [x] 双层 (user_account + subagent + agent) 灵活架构 (per subagent_persist_opt4 拍板)
3. [x] 多重隶属 + 跨 team 不同 role (per team_dimension_opt4 拍板)
4. [x] 5 段 20 子项 WBS (per WBS v0.1 §1)
5. [x] 守门 #13 DB 三類横展開 100% 覆盖 (per W1 报告 §2.1)
6. [x] 守门 #21 [P] docs 同步 落地 (per W1 报告 §2.1)
7. [x] 守门 #6 PowerShell only 落地 (per W1 报告 §2.1)
8. [x] 守门 #11 缺标比错标安全 17 缺口显式列 (per W1 报告 §3)
9. [x] 守门 #9 不 commit 散落子代理产出 (W1 不派子代理, per 守门 #9 v20)
10. [x] 守门 #10 代签规则 (Mavis 接手代签 Ulysses, per 19:39/20:56/21:59 JST 三次强化)
11. [x] 守门 #1 (R-05 不 push, 已反转但 W1 不推, 留 W5 G.20)
12. [x] 守门 #25 (Agent Jira 化) 派生规 准备追加 AGENTS.md §4.1 v25
13. [x] P3-G 命名空间解耦 (跟 P3-B OpenClaw 9 子项共存, per WBS §3 缺口 #1)

---

## 替代方案 ADR 引用

- ADR-0021 (Zero Vendor Cooperation) — 不引入 Vendor 合作
- ADR-0026 (STAR AI 兼容 5 通道) — 跟 subagent 跨 Provider 兼容
- ADR-0030 (Lease + Heartbeat + Resume) — 跟 AgentSession 跨 session 续
- ADR-0032 (MCP Transport stdio) — 跟 MCP 协议兼容
- ADR-0033 (Agent Co-signing Policy) — 跟 co-signing 协议兼容
- 本 ADR-0034 (Agent Jira 化) — 本阶段

---

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-09-03 | v0.1 | 初版: 完整 Jira 化 5 段 20 子项 WBS, W1 docs 已实质完工 (7 表设计落地, 3 commits / 11 files / 1919 insertions), W2-W5 跨 session 续, 守门 #25 派生规 准备追加 AGENTS.md §4.1 v25 |
