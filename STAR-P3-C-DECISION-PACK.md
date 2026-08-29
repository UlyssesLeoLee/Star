# STAR-P3-C-DECISION-PACK P3-C 阶段 9 子项拍板包 (per 2026-08-30 07:44 JST)

> **Status**: 🟡 Draft (P3-A 收官 + P3-B 7/9 子项收官落地后, P3-C 启动门槛准备)
> **Created**: 2026-08-30
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008)
> **承接**: STAR-P3-WBS-001 §2 P3-C 占位表 (9 子项 / 40M / 6.7 周) + AGENTS.md §4 守门 #12 v15 派生饱和
> **For**: Ulysses 拍板 P3-C 9 子项真实标题 + token 软预算 + 依赖, 触发 P3-C 启动 (新 INC-SESSION-003 / 后续)

本文件是 P3-C 阶段 9 子项的拍板包. P3-A + P3-B 实证收官 (per `PHASE-P3-A-INC-SESSION-002.md` v0.5 + 7 P3-B PHASE 报告), P3-C 当前 9 子项全 "待拍" 阻塞 P3 阶段推进.

---

## §0 背景

P3-C 阶段软预算 40M tokens / 9 子项 / 6.7 周 (per STAR-OLU-001 §1 1 SRE·周 = 1.2M).

9 子项当前占位 = 9 个 "TBD", 缺:
- 真实子项标题
- 软预算分配 (40M 平摊 ≈ 4.4M/子项)
- 依赖 (无 / 串行 / 并行)
- 状态 (拍板后从 🟡 占位 → 🟢 进行中)

---

## §1 P3-C 拍板包 (9 子项推荐标题)

> **推荐策略**: 5 域 DDD 边界 + 跨域编排 + 持久层 (per 8/21 JST Ulysses 5 域独立 Lead 拍板基础). 9 子项覆盖:
> - 5 域业务 (C.1-C.5, 跟 RGS 5 域 player/economy/match/social/admin 镜像, 但给 Star 用)
> - 1 跨域编排 (C.6 Saga)
> - 1 持久层 (C.7 Postgres 接入)
> - 1 多租户 (C.8 Tenant 边界)
> - 1 性能 + DDD Review (C.9 5 域 Lead 真人到位)

| # | 子项 | 标题(推荐) | 软预算 | 软参考周 | 依赖 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|
| C.1 | C.1 | **Workspace 域** (per-tenant workspace CRUD) | 4.4M | 0.7 周 | 无 | 🟡 占位 | 推荐: Workspace 是 Star 顶层租户边界, RGS 5 域 (player/economy/match/social/admin) 之上 |
| C.2 | C.2 | **Project 域** (per-workspace project CRUD + per_project_role RBAC) | 4.4M | 0.7 周 | C.1 | 🟡 占位 | 推荐: Project 是 workspace 子域, 角色矩阵 5 域 Lead 拒绝兼任硬约束 |
| C.3 | C.3 | **Identity 域** (Identity + Permission + WorkspaceMember 三实体) | 4.4M | 0.7 周 | C.1 | 🟡 占位 | 推荐: 跟 RGS identity 域镜像 |
| C.4 | C.4 | **WorkItem 域** (work_item + status 状态机 + per_project 过滤) | 4.4M | 0.7 周 | C.2 | 🟡 占位 | 推荐: WorkItem 是 Project 子域, 跟 frontend 5 tab 命名 (Kanban / Timeline / Backlog) 配合 |
| C.5 | C.5 | **Workflow 域** (workflow + workflow_state + per_project 自动化) | 4.4M | 0.7 周 | C.4 | 🟡 占位 | 推荐: Workflow 是 WorkItem 子域, 跟 P3-A.6 / P3-A.7 接入 |
| C.6 | C.6 | **Saga 跨域编排** (Q-003 / Per-domain saga + 跨域 compensation) | 4.4M | 0.7 周 | C.1-C.5 | 🟡 占位 | 推荐: 5 域业务子域都齐后, 跨域 Saga 编排 |
| C.7 | C.7 | **Postgres 持久层** (sqlx + per-tenant connection pool + migration) | 4.4M | 0.7 周 | C.1 | 🟡 占位 | 推荐: C.7 持久层独立子项, sqlx 已 workspace dep |
| C.8 | C.8 | **Tenant 边界** (per-tenant row-level security + tenant context 注入) | 4.4M | 0.7 周 | C.7 | 🟡 占位 | 推荐: 多租户安全硬约束 (per Star 顶层) |
| C.9 | C.9 | **5 域 Lead 真人到位** (per 8/21 JST 拒绝兼任硬约束, DDD Review 签字) | 4.4M | 0.7 周 | C.1-C.5 | 🟡 占位 | 推荐: DDD Review 阶段, 5 域 Lead 真人签字 |
| **小计** | | | **40M** | **6.7 周** | | | |

**列含义**:
- 软预算: token 预算 ÷ 1.2M SRE·周上限 → 周数
- 软参考周 **不参与 gating**, 仅供"若按人类节奏"预估 (per STAR-OLU-001 §1)
- 阻塞: 需外部凭证/拍板, 不能 root 单方推进
- 占位: 草案标题, 需 Ulysses 拍板真实范围

---

## §2 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 9 子项标题均为草案, 真实范围需 Ulysses 拍板 | 等 Ulysses 决策 |
| 2 | C.9 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任), 等 Ulysses 找 5 个真人 | 跨 session 续 |
| 3 | 软预算为占位估算, 真实 token 待 SRE Lead 接入 telemetry 后回填 | P3-C phase 2 续 |
| 4 | 跨子项依赖图未画 (C.1-C.5 串行, C.6-C.9 并行未定) | 等 Ulysses 拍板后回填 |
| 5 | 质量门 5 维未在 C.* 子项上实证 (C.* 还没启动) | C.* 阶段启动后实证 |
| 6 | C.7 Postgres 真实凭证未到位 (连接串 / KMS 加密) | P3-C 启动前需 Ulysses 凭证 |

---

## §3 拍板选项 (Ulysses 一键决定)

### 选项 1: 批准推荐草案 (9 子项全按推荐拍, 6.7 周 + 40M)

- **推荐**: 9 子项全用推荐标题, 软预算 40M 平摊 4.4M/子项
- **触发**: 7 wt 并行启动 (per 10:58 JST 每子项 1 wt 决策, D 阶段已验证)
- **前置依赖**: C.1 (Workspace) 单 wt 启动, C.2-C.5 等 C.1 merge 后开 wt, C.6-C.9 等 C.1-C.5 merge 后开 wt

### 选项 2: 推迟 P3-C 启动, 推进 P3-D

- **理由**: P3-B 收官后, 跨入 P3-D 7 子项 (D.1-D.7, per STAR-P3-WBS-001 §3) 可能更优先 (e2e 矩阵 / Playwright / markdownlint)
- **风险**: P3-C 5 域业务子域都未实装, P3-D 接 frontend e2e 等没业务底座

### 选项 3: 折中, P3-C 只拍 C.1 + C.7 (Workspace + Postgres) 2 子项, 后续增量拍

- **理由**: 5 域业务等 phase 2, 持久层 + 顶层边界先实装
- **token 估算**: 8.8M (2 子项), 1.4 周
- **风险**: C.2-C.9 依赖 C.1 / C.7, 推迟影响后续

### 选项 4: 自定义

- 你给 9 子项的真实标题 + 软预算分配, 我按你的方案实装
- 需在备注里写明 9 子项定义

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- 拍板后 7 wt 并行启动 + 子代理 brief 写明"无证据叙事 = 禁止" (per AGENTS §1.2 派生规 4)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件仅作拍板草案 + 推荐, **不实施 P3-C 任何子项**, 等 Ulysses 拍板 | 2026-08-30 07:44 JST Mavis 接手代签 |
| 2 | 每推荐行标 🟡 占位, 拍板后行标 🟢 进行中 | 本文件 §1 状态列 |
| 3 | token 软预算 ÷ 1.2M SRE·周上限 → 软参考周, **不参与 gating** | STAR-OLU-001 §1 |
| 4 | 推进门槛是质量门禁 ≥4/5, 不是截止日期 | STAR-OLU-001 §0 |
| 5 | 守门 #12 commit-time 同步 (本文件 commit 即触发, 后续 docs 同步接 v15 派生饱和) | AGENTS §4.1 v15 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟡 Draft; 9 子项推荐标题 (5 域业务 + Saga + 持久层 + 多租户 + 5 域 Lead 真人) + 4 拍板选项 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 9 子项推荐 (C.1 Workspace / C.2 Project / C.3 Identity / C.4 WorkItem / C.5 Workflow / C.6 Saga / C.7 Postgres / C.8 Tenant / C.9 5 域 Lead) + 4 拍板选项 + 已知缺口 6 项 | 2026-08-30 P3-B 7/9 子项收官 (B.1+B.3+B.4+B.6+B.7+B.8+B.9 commit 落地) 后, P3-C 启动门槛准备 |
