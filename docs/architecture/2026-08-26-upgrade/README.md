# STAR × GitGit AI/IDE 零厂商适配架构升级 — 索引

> **状态**：🟡 草案集 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **分支**：`feature/ai-ide-compat`（STAR）· `feature/ide-boundary`（GitGit）

---

## 0. 任务概览

> **核心约束**：绝不假设任何 AI 厂商主动适配 STAR。
> **核心结论**：IDE 归 STAR，GitGit 只做 VCS Core。
> **验收问**（per §50）：全新 Coding Agent / IDE 不懂 STAR/GitGit，能接入吗？答案必须 = YES。

## 1. 文档地图

### 1.1 生态事实基线（Phase A）

- [ecosystem-survey/ai-compatibility-matrix.md](../../ecosystem-survey/ai-compatibility-matrix.md) — 7 款 Coding Agent
- [ecosystem-survey/ide-compatibility-matrix.md](../../ecosystem-survey/ide-compatibility-matrix.md) — 6 款 IDE / CDE
- [ecosystem-survey/protocol-survey.md](../../ecosystem-survey/protocol-survey.md) — 4 套协议 + 2 套工具
- [ecosystem-survey/compatibility-matrix.md](../../ecosystem-survey/compatibility-matrix.md) — 综合矩阵 + 接入路径

### 1.2 边界 ADR（Phase B）

- [adr/0021-zero-vendor-cooperation.md](adr/0021-zero-vendor-cooperation.md) — 最高原则
- [adr/0022-ide-placement.md](adr/0022-ide-placement.md) — IDE 归 STAR
- [adr/0023-version-control-provider.md](adr/0023-version-control-provider.md) — VCS Provider 抽象
- [adr/0024-ide-session-identity.md](adr/0024-ide-session-identity.md) — IDE Session 独立
- [adr/0025-vendor-adapter-anti-contamination.md](adr/0025-vendor-adapter-anti-contamination.md) — Vendor Adapter 隔离

### 1.3 责任矩阵（Phase B）

- [responsibility-matrix/star-vs-gitgit.md](../../responsibility-matrix/star-vs-gitgit.md) — 60 项能力正交表
- [responsibility-matrix/gitgit-ide-boundary.md](../../responsibility-matrix/gitgit-ide-boundary.md) — GitGit IDE 接口边界

### 1.4 架构分析（Phase C 第 1 组）

- [arch/01-current-architecture-analysis.md](arch/01-current-architecture-analysis.md) — 现状
- [arch/02-ide-capability-boundary.md](arch/02-ide-capability-boundary.md) — 4 层分解
- [arch/03-star-ai-compat-arch.md](arch/03-star-ai-compat-arch.md) — STAR AI 兼容
- [arch/04-star-ide-gateway-arch.md](arch/04-star-ide-gateway-arch.md) — IDE Gateway
- [arch/05-gitgit-compat-arch.md](arch/05-gitgit-compat-arch.md) — GitGit 兼容
- [arch/06-threat-model-nfr.md](arch/06-threat-model-nfr.md) — Threat + NFR

### 1.5 接口规范（Phase C 第 2 组）

- [spec/cli/01-cli-spec.md](spec/cli/01-cli-spec.md) — `star` CLI 17 命令
- [spec/agent-api/01-schema.md](spec/agent-api/01-schema.md) — Agent JSON Schema
- [spec/ide-api/01-schema.md](spec/ide-api/01-schema.md) — IDE JSON Schema
- [spec/mcp/01-mcp-spec.md](spec/mcp/01-mcp-spec.md) — MCP server 13 tools
- [spec/rest/01-rest-strategy.md](spec/rest/01-rest-strategy.md) — OpenAPI 3.1
- [spec/context/01-context-api.md](spec/context/01-context-api.md) — Context API
- [spec/context/02-code-intelligence-arch.md](spec/context/02-code-intelligence-arch.md) — Code Intelligence
- [spec/context/03-code-navigation-arch.md](spec/context/03-code-navigation-arch.md) — Code Navigation
- [spec/context/04-context-graph.md](spec/context/04-context-graph.md) — Context Graph

### 1.6 资源模型（Phase C 第 3 组）

- [spec/resources/01-workspace-protocol.md](spec/resources/01-workspace-protocol.md)
- [spec/resources/02-worktree-protocol.md](spec/resources/02-worktree-protocol.md)
- [spec/resources/03-agent-identity.md](spec/resources/03-agent-identity.md)
- [spec/resources/04-ide-session-identity.md](spec/resources/04-ide-session-identity.md)
- [spec/resources/05-agent-permission-model.md](spec/resources/05-agent-permission-model.md)
- [spec/resources/06-ide-permission-model.md](spec/resources/06-ide-permission-model.md)

### 1.7 流程（Phase C 第 4 组）

- [spec/flows/01-agent-task-lifecycle.md](spec/flows/01-agent-task-lifecycle.md) — 9+4 状态
- [spec/flows/02-agent-lease-heartbeat.md](spec/flows/02-agent-lease-heartbeat.md) — Lease
- [spec/flows/03-agent-resume.md](spec/flows/03-agent-resume.md) — Resume
- [spec/flows/04-multi-agent.md](spec/flows/04-multi-agent.md) — Multi-Agent
- [spec/flows/05-universal-submit.md](spec/flows/05-universal-submit.md) — Submit 11 步
- [spec/flows/06-error-recovery.md](spec/flows/06-error-recovery.md) — 错误模型
- [spec/flows/07-audit-model.md](spec/flows/07-audit-model.md) — Audit
- [spec/flows/08-event-model.md](spec/flows/08-event-model.md) — 事件

### 1.8 VCS（Phase C 第 5 组）

- [spec/vcs/01-version-control-provider.md](spec/vcs/01-version-control-provider.md)
- [spec/vcs/02-gitgit-provider.md](spec/vcs/02-gitgit-provider.md)
- [spec/vcs/03-github-gitlab-compat.md](spec/vcs/03-github-gitlab-compat.md)
- [spec/vcs/04-fallback-strategy.md](spec/vcs/04-fallback-strategy.md)

### 1.9 验收（Phase C 第 6 组）

- [spec/acceptance/01-unknown-agent-test.md](spec/acceptance/01-unknown-agent-test.md)
- [spec/acceptance/02-zero-knowledge-agent-test.md](spec/acceptance/02-zero-knowledge-agent-test.md)
- [spec/acceptance/03-unknown-ide-test.md](spec/acceptance/03-unknown-ide-test.md)
- [spec/acceptance/04-mvp.md](spec/acceptance/04-mvp.md)
- [spec/acceptance/05-phase2.md](spec/acceptance/05-phase2.md)
- [spec/acceptance/06-phase3.md](spec/acceptance/06-phase3.md)
- [spec/acceptance/07-adr-list.md](spec/acceptance/07-adr-list.md)
- [spec/acceptance/08-risk-register.md](spec/acceptance/08-risk-register.md)
- [spec/acceptance/09-agent-instructions-spec.md](spec/acceptance/09-agent-instructions-spec.md)
- [spec/acceptance/10-ide-instructions-spec.md](spec/acceptance/10-ide-instructions-spec.md)
- [spec/acceptance/11-token-efficiency.md](spec/acceptance/11-token-efficiency.md)
- [spec/acceptance/12-capability-discovery.md](spec/acceptance/12-capability-discovery.md)
- [spec/acceptance/13-schema-stability.md](spec/acceptance/13-schema-stability.md)
- [spec/acceptance/14-performance-requirements.md](spec/acceptance/14-performance-requirements.md)
- [spec/acceptance/15-final-acceptance.md](spec/acceptance/15-final-acceptance.md)
- [spec/acceptance/16-ecosystem-research-summary.md](spec/acceptance/16-ecosystem-research-summary.md)
- [spec/acceptance/17-master-plan-update.md](spec/acceptance/17-master-plan-update.md)

## 2. 关键决策（per Phase B）

| 决策 | 文档 |
|---|---|
| Zero Vendor Cooperation 最高原则 | [ADR-0021](adr/0021-zero-vendor-cooperation.md) |
| IDE 归 STAR | [ADR-0022](adr/0022-ide-placement.md) |
| VCS Provider 4 个并列 | [ADR-0023](adr/0023-version-control-provider.md) |
| IDE Session 独立 | [ADR-0024](adr/0024-ide-session-identity.md) |
| Optional Adapter 子 crate | [ADR-0025](adr/0025-vendor-adapter-anti-contamination.md) |

## 3. 关键事实（per Phase A 2026-08-26 调研）

- 7/7 主流 Coding Agent 都支持 Git + Shell + FS
- 6/7 支持 MCP（增强层）
- 5/7 支持独立 Worktree
- 20+ 工具读 AGENTS.md
- 2026-07-28 是 MCP 当前最新规范（12 个月迁移窗口）
- 2025-10 Gitpod 关停、2026-06 Gemini CLI 个人账户停服 → 强化 Zero Vendor Cooperation

## 4. 终极验收（per §50）

> **Q1**: 全新 Coding Agent 从未听说过 STAR/GitGit，但会 Git + Shell，能接入吗？
> **A: YES**
>
> **Q2**: 全新 IDE 从未听说过 STAR/GitGit，但支持 Git + Shell + FS + 终端，能接入吗？
> **A: YES**

## 5. 工作量

| Phase | Tokens | SRE·周 | 窗口 |
|---|---|---|---|
| A 调研 | 300K | 0.3 | 1 周 |
| B 边界 ADR | 400K | 0.4 | 1 周 |
| C spec (54 份) | 3-4M | 3-4 | 4-5 周 |
| D MVP 闭环 | 5-8M | 5-8 | 3-4 周 |
| **合计** | **9-13M** | **9-13** | **4-6 周（子代理并行）/ 9-13 周（Mavis 单干）** |

## 6. 实施分支

- STAR：`feature/ai-ide-compat`（基于 main@4b3b8dc）
- GitGit：`feature/ide-boundary`（基于 `feature/v0-gui-and-keychain` 起点，保留 V0 Tauri GUI 独立）

## 7. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板全套草案 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：54 份 spec 全套索引 | Phase C 第 1 轮 + 第 2 轮完成 |
