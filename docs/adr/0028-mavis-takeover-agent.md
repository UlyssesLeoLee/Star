# ADR-0028: Mavis 接手 agent 永久代签 author=Ulysses (DEC-008)

> **status**: ✅ Accepted (per 2026-08-27 19:39 JST Ulysses 授权 + 9/10 12:45 v0.62 反转拍板)
> **date**: 2026-08-27 JST (decision); 2026-10-02 JST (ADR formalized per PR-276)
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **supersedes**: none
> **superseded by**: ADR-0032 (Mavis 接手**审核** policy v0.70, 替代本 ADR 的"永久代签"边界)

## Context

Ulysses 以"一人公司"模式运营本项目 (Star monorepo + RGS + Physis + GVPE + GVPE mock 5 跨项目)，需要 12 角色扮演 (架构师 / 开发者 / 测试 / SRE / 安全 / 产品 / 设计 / 数据 / DevOps / AI Lead / 文档 / 决策记录)，但单人无法在合理时间完成全部 12 角色 deep review。Mavis (root agent / Mavis 接手 agent) 作为 Ulysses 委托的 agent，承担跨角色审核职责。

### DEC-008 实际决策 (历史追溯)

| 时间 | 决策 | 含义 |
|---|---|---|
| **2026-08-27 19:39 JST** | Ulysses 授权 Mavis 接手 agent | 12 角色 deep review 永久由 Mavis 代签 |
| **2026-09-08 15:19 JST** | 第 6 次强化 (Mavis 全权代理) | Mavis 自驱 vs ask_user 推荐项判定 |
| **2026-09-08 15:29 JST** | 第 7 次强化 (Mavis 自驱) | 整体方向大转弯 / host 状态永久改变边界 |
| **2026-09-10 12:45 JST** | v0.62 反转拍板: Mavis **审核** (非代签) | 修订人 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**`, Mavis 承担审核责任 (非代签责任) |
| **2026-09-10 14:55 JST** | **v0.70 拍板激活** | 落地 `docs/guardian/v32_audit_boundary.md` v0.1 |

## Decision

DEC-008 决策点为 "Mavis 接手 agent 永久代签 author=Ulysses"。该决策在 2026-09-10 12:45 JST 由 v0.62 反转升级为 "Mavis 接手**审核**" (更清晰的责任划分)，并在 2026-09-10 14:55 JST 由 v0.70 拍板激活。

### AGENTS.md / docs 中 DEC-008 引用统一格式

- 修订人 (文档 / commit / 报告头部): `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**`
- 审批 (审批行 / AGENTS.md §commit author 列表): `架构师 (Mavis 接手 agent per DEC-008)`
- 引用本 ADR: `[ADR-0028](0028-mavis-takeover-agent.md)` 或 `docs/adr/0028-mavis-takeover-agent.md`

### 12 角色 per DEC-008 (per AGENTS.md §1)

| # | 角色 | 主要职责 |
|---|---|---|
| 1 | 架构师 | 整体架构 / 跨模块决策 / ADR 拍板 |
| 2 | 开发者 | 代码实现 / 测试编写 / bug 修复 |
| 3 | 测试工程师 | 测试设计 / 测试覆盖率 / E2E |
| 4 | SRE | 部署 / 监控 / 告警 / 容量 |
| 5 | 安全 | 威胁建模 / 代码审计 / 渗透测试 |
| 6 | 产品 | 需求 / 优先级 / 用户故事 |
| 7 | 设计 | UI / UX / 视觉设计 / 原型 |
| 8 | 数据 | 数据建模 / 数据库 / 备份 / GDPR |
| 9 | DevOps | CI/CD / 自动化 / IaC / 镜像 |
| 10 | AI Lead | AI/ML 决策 / LLM 选择 / prompt 工程 |
| 11 | 文档 | API 文档 / 用户手册 / ADR / 设计文档 |
| 12 | 决策记录 | commit 审计 / 决策追溯 / 责任划分 |

Mavis 接手 agent 承担 12 角色 deep review 任务 (per Ulysses 2026-08-27 19:39 JST 授权)，并按 v0.70 拍板承担审核责任。

## Consequences

### Positive

- 单人公司可以运营 5 跨项目 (Star + RGS + Physis + GVPE + GVPE mock)
- 决策速度 vs 单人 deep review trade-off 优化
- 责任清晰: 修订人 = Ulysses (实际作者), 审核 = Mavis (审核者)

### Negative

- Mavis 失误传播风险 (per 守门 #15 饱和边界 + 守门 #1 R-05)
- Ulysses 单点失效 (host-down / Mavis-down 时无备份)

### 适用边界 (per v0.62 + v0.70 拍板)

✅ **Mavis 审核适用**: commit / 报告 / WBS row / docs sync 是否合理, 决定 push / merge / 拍板
✅ **适用跨项目**: STAR / RGS / Physis / GVPE / GVPE mock
✅ **适用跨 session**: root + child subagent

❌ **不适用边界**:
- 整体方向大转弯 (e.g., 重新选型)
- host 状态永久改变 (e.g., 重大架构重构)
- 涉及法律 / 财务 / 用户隐私的决策

## Refs

- AGENTS.md §1 Authority + §commit author 列表 (5 处 DEC-008 引用)
- .multica/ULYS-57-report.md (修订人 + 审批 行)
- docs/guardian/v32_audit_boundary.md v0.1 (5 类审核决策 + 6 适用边界)
- PR-272 (README.md 8 数字 actualize, Mavis 接手审核 line) + PR-276 (本 ADR 形式化)

## Revision history

| version | date | author | change |
|---|---|---|---|
| **v0.1** | 2026-10-02 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手审核 | DEC-008 历史追溯 + ADR 形式化 (per PR-276 follow-up of PR-272 docs 乖离 audit) |
