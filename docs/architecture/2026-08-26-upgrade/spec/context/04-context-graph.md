# 19. Context Graph

> **状态**：🟡 草案 v0.2
> **日期**：2026-09-01
> **修订人**：架构师 (Mavis 接手 agent per DEC-008)
> **依赖**：[spec/context/01-context-api.md](01-context-api.md) + [basic-design §4.4 domain-context + §4.12 Event Bus 协作机制](../../../../basic-design.md) + [spec/saga/01 v0.2 SagaCoordinationRole](../saga/01-saga-coordination-spec.md)

> **dual-use 警告 (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板)**：
> Context Graph 节点 / 关系类型不绑定 RGS 5 域（player/economy/match/social/admin）业务子域命名。
> - Issue / Repository / Worktree / Commit 4 节点归属到 22 domain crate 之一，通过 [basic-design §3.2.9](../saga/01-saga-coordination-spec.md) contact face 表显式声明
> - 不通过本 spec 反推 5 域业务子域映射

## 1. MVP 节点类型（4 类）

| 节点 | 字段 | 归属 22 domain crate | 接触方式 |
|---|---|---|---|
| Issue | id / title / status / labels | `domain-work-item` (Issue ≡ WorkItem) | Customer-Supplier (Context Graph 读 Issue 元数据) |
| Repository | id / provider / url / name | `domain-scm` | ACL(下游) (SCM Adapter 隔离) |
| Worktree | id / path / branch / head_commit | `domain-worktree` | Customer-Supplier (Context Graph 读 worktree 状态) |
| Commit | sha / author / message / files_changed | `domain-scm` + `domain-development` (ChangeSet → Commit 桥接) | ACL(下游) |

**节点 ↔ 22 domain crate 关联原则** (v0.2 新增, per modules 协作细化):
- 每个节点必须有 **唯一 primary responsible_crate** (per [spec/saga/01 v0.2 §2](../saga/01-saga-coordination-spec.md) SagaStep.responsible_crate 模式)
- Issue ↔ WorkItem 一致性: Issue 是 WorkItem 在 Context Graph 视图的别名(per [basic-design §6 22 bounded context](../saga/01-saga-coordination-spec.md))
- Commit 双归属: domain-scm 持有原始 Git commit sha,domain-development 持有 ChangeSet 业务语义,Context Graph 通过 `derived_from` 关系桥接

## 2. MVP 关系类型（5 类）

| 关系 | 含义 | 源节点 | 目标节点 | 触发 domain event |
|---|---|---|---|---|
| `implements` | Worktree → Issue | Worktree (domain-worktree) | Issue (domain-work-item) | WorktreeCreated (per [basic-design §4.12.1](../../../../basic-design.md)) |
| `modifies` | Commit → Worktree | Commit (domain-scm) | Worktree (domain-worktree) | ChangeSetObserved |
| `references` | Commit → Issue | Commit (domain-scm) | Issue (domain-work-item) | FeedbackCreated / PR_Linked |
| `belongs_to` | Worktree → Repository | Worktree (domain-worktree) | Repository (domain-scm) | WorktreeCreated |
| `derived_from` | Commit → Commit (parent) | Commit (domain-scm) | Commit (domain-scm) | 实时,不触发 event |

**关系 ↔ 22 domain 协作** (v0.2 新增):
- 所有 5 关系由 Context Compiler 维护 (per `domain-context` crate)
- 关系创建/更新走 `domain-context` 的 projection role (per [basic-design §4.12.2 5 订阅者矩阵](../../../../basic-design.md))
- 关系查询走 Context Graph API (per [spec/context/01-context-api.md](01-context-api.md))

## 3. Phase 2+ 节点类型（10+ 类）

Symbol / File / MR / Test / Pipeline / Deployment / Incident / Agent / User / Document / Package / Vulnerability

**Phase 2+ 节点归属 (v0.2 计划)**:
- Symbol / File: `domain-scm` (符号索引 per [spec/services/02-symbol-analysis](../services/02-symbol-analysis.md))
- MR / Test / Pipeline: `domain-scm` + `domain-validation` (per [basic-design §4.5](../../../../basic-design.md))
- Deployment / Incident: `domain-local-runtime` (per [basic-design §4.6](../../../../basic-design.md))
- Agent: `domain-agent` (per [basic-design §4.2](../../../../basic-design.md))
- User: `domain-identity`
- Document / Package: `domain-scm` + `domain-work-item`
- Vulnerability: `domain-validation` (Security Check 派生)

## 4. Phase 2+ 关系类型（12+ 类）

depends_on / generated_by / reviewed_by / tested_by / deployed_by / caused_by / fixed_by / related_to / located_in / opened_in

## 5. 存储

- MVP: SQLite + 简单外键
- Phase 2: 考虑图数据库（per [Compatibility Matrix §6 已知缺口](../../../../ecosystem-survey/compatibility-matrix.md) — **不**自建图数据库）

**存储跨域协作 (v0.2 新增)**:
- Context Graph 存储在 `star-context` crate 自有 SQLite (per `crates/star-context/migrations/`)
- 不复用 PostgreSQL SoR (per requirements §14 REQ-DATA-001) — Context Graph 是 Projection 而非 SoR
- 22 domain 通过 `domain-context` Port 访问 (per [basic-design §3.2.4 context → 多个](../../../../basic-design.md))

## 6. 实施位置

- `crates/star-context/src/graph.rs` — 节点 + 关系
- `crates/star-context/migrations/` — SQLite schema
- `crates/star-context/src/projection_worker.rs` — worker projection role (per [basic-design §4.12.2](../../../../basic-design.md))

## 7. 跨域协作时序 (v0.2 新增, per requirements §26 Context Compiler 要件)

```text
T0  WorktreeCreated (NATS event, per §2 implements 关系)
T1  context-build worker 订阅 → 写 Context Graph (Issue + Worktree + Repository 3 节点 + implements + belongs_to 2 关系)
T2  ContextPacketCreated (NATS event)
T3  Agent 进程 spawn → 拉取 Context Graph 关联节点 → 注入 Context Packet
T4  ChangeSetObserved (NATS event, per §2 modifies 关系)
T5  context-build worker 写 Context Graph (Commit 节点 + modifies + references 关系)
T6  PR_Linked (NATS event, per §2 references 关系)
T7  Context Packet recompile 触发 (per [spec/saga/01 v0.2 §4 5 步流程](../saga/01-saga-coordination-spec.md) StartContextBuild step)
```

## 8. 与 22 domain 接触面总览 (v0.2 新增)

per [basic-design §3.2.9 22 domain contact face 表](../../../../basic-design.md),`domain-context` 与 22 domain 中 11 个有显式接触面:

| 源 | 目标 | 接触方式 | 接触点 |
|---|---|---|---|
| context | work-item | Customer-Supplier | 读取 Requirement / AcceptanceCriterion |
| context | worktree | Customer-Supplier | 读取 Worktree.current_change_set, test_state |
| context | feedback | Customer-Supplier | 读取 Open Feedback (per requirements §26) |
| context | validation | Customer-Supplier | 读取 Failed Validation (per requirements §27) |
| context | scm | Conformist | 通过 SCM Adapter 读取 Repository 元数据 |
| context | identity | Customer-Supplier | 读取 AgentPolicy |
| context | agent | Customer-Supplier | 推送 Context Packet (per requirements §24) |
| context | notification | Separate Ways | 监听 ValidationFailed 触发 (per REQ-NOTIF-001) |
| context | collaboration | Customer-Supplier | Realtime 推送 Context Graph 更新 |
| context | audit | Separate Ways | ContextPacketCreated 事件全量审计 |
| context | worktree | Customer-Supplier (re-confirm) | `domain-context` 监听 Worktree 状态变化 → 重新编译 Context Packet |

## 9. 已知缺口 (v0.2 新增, per 缺标比错标安全)

| # | 缺口 | 状态 | 触发 |
|---|------|------|------|
| CG-01 | Phase 2+ 12 节点类型的字段详细定义 (Symbol/File/MR/Test 等) | 🟡 Phase 2 启动时 | v0.1 仅列名,字段待 Phase 2 补 |
| CG-02 | Context Graph 与 Saga spec v0.2 SagaContext.crate_state 集成 (key = responsible_crate 字符串) | 🟡 Phase G 评估 | SagaContext 已支持,Context Graph 集成待定 |
| CG-03 | 跨 tenant 隔离边界 (per REQ-SEC-001) | 🟡 Phase H+ 评估 | Context Graph 是 Projection,需 RLS 同步 |
| CG-04 | 性能基线 (跨 22 domain Context Packet 编译 P99) | 🟡 SRE Lead 量化 | 端到端 P99 未基线 |
| CG-05 | 与 [spec/services/02-symbol-analysis](../services/02-symbol-analysis.md) 符号分析的融合 | 🟡 Phase 2 | 符号分析是 Phase 2 节点类型 |
| CG-06 | Context Graph 与 star-sse Realtime 推送的 SLA | 🟡 SRE Lead 量化 | Realtime 降噪策略 (per [basic-design §4.13](../../../../basic-design.md)) 对 Context Graph 推送的影响未量化 |

## 10. 签字栏 / 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis 接手 (per arch/01 模板) | 初版：MVP 4 节点 + 5 关系 + Phase 2+ 12 节点 / 12 关系 + 存储 + 实施位置 | per [ADR-0031 §MVP](../../adr/0031-context-graph.md) 4 节点 / 5 关系 |
| v0.2 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | **模块间协作细化 (per AGENTS.md §5 v0.6 + Q1-D 拍板)**：① §0 文件头加 dual-use 警告 + 5 域脱钩声明;② §1 4 节点加归属 22 domain crate + 接触方式列 + 节点 ↔ 22 crate 关联原则;③ §2 5 关系加源/目标节点 22 crate 归属 + 触发 domain event + 关系 ↔ 22 domain 协作;④ §3 Phase 2+ 12 节点加 22 crate 归属计划;⑤ §5 存储加跨域协作 + Projection 性质;⑥ §6 实施位置加 projection_worker.rs;⑦ §7 新增跨域协作时序 (8 步);⑧ §8 新增 22 domain 接触面总览 (11 domain 显式);⑨ §9 新增 6 已知缺口 (CG-01~CG-06, per 缺标比错标安全) | 2026-09-01 14:38 JST 模块间协作细化任务 (A 架构层 22 Domain 协作 + L3 完整覆盖 + doc-only) |

---

> **审批者**：架构师 (Mavis 接手 agent per DEC-008) — 2026-08-26 (v0.1) / 2026-09-01 (v0.2)
> **per AGENTS.md §1 代签规则反转 + 2026-08-27 19:39 JST 代签授权升级 + 21:59 JST 第三次强化**：Mavis 接手默认代签 Ulysses 无需再问
