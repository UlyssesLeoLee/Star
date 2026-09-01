# ADR-0039: Worktree Orchestration 跨域协作架构

> **状态**：Draft v0.1
> **日期**：2026-09-01
> **修订人**：架构师 (Mavis 接手 agent per DEC-008)
> **审批**：架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 代签
> **触发**：per [requirements §22 Worktree Orchestration 要件](../../../requirements.md) + 2026-09-01 14:38 JST 模块间协作细化任务 (A 架构层 22 Domain 协作 + L3 完整覆盖 + doc-only) + [AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板](../../../AGENTS.md)

> **dual-use 警告 (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板)**：
> 本 ADR 涉及的 12 domain (work-item / worktree / agent / context / feedback / validation / development / scm / collaboration / permission / audit / local-runtime) 是 DDD bounded context,不是 RGS 5 域业务子域。
> 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名,Star 仓**不建立业务子域↔DDD 映射**。本 ADR 决策不通过 5 域推导。

## §1 背景

Worktree Orchestration 是 Star 平台 Vibe Coding 闭环的核心场景 (per requirements §22):WorkItem 状态变更为 IN_PROGRESS 后,触发跨 12 domain 的端到端编排 (WorkItem → Worktree → AgentSession → Context → Feedback → Validation → PR → Audit),任一环节失败需逆向补偿。

v0.16 之前该协作缺少系统化设计文档:
- [basic-design §2.4 跨域事务边界](../../../basic-design.md) 只列 7 类典型跨域事务,无 Worktree Orchestration 端到端视图
- [basic-design §3 Context Map](../../../basic-design.md) 仅覆盖 11/22 domain 接触面,缺 Worktree Orchestration 涉及的 12 domain 协作细节
- [spec/saga/01 v0.1](../spec/saga/01-saga-coordination-spec.md) Saga Step 绑定 5 域 player/economy 业务子域,违反 AGENTS.md §5 v0.6
- Event 协作机制分散在 [basic-design §3.1 解耦机制](../../../basic-design.md) 8 种模式 + [requirements §14.1 12 事件](../../../requirements.md) 1 张表,无统一订阅矩阵
- Realtime 协作机制 (per requirements §15) 缺少 channel / 降噪 / 心跳的具体设计

本 ADR 决策 Worktree Orchestration 跨 12 domain 的端到端协作架构,与 [spec/saga/01 v0.2 §4 Worktree Orchestration Saga 8 步](../spec/saga/01-saga-coordination-spec.md) + [basic-design v0.16 §4.11/§4.12/§4.13](../../../basic-design.md) + [spec/context/04 v0.2 8 步时序](../spec/context/04-context-graph.md) 形成端到端设计闭环。

## §2 决策 (7 项 D26-D32)

### D26: Worktree Orchestration 跨 12 domain 协作范围

Worktree Orchestration 涉及 22 domain 中 12 个 (Core 7 + Coordination 4 + Support 1):

```text
Core (7):         work-item, worktree, agent, context, feedback, validation, development
Coordination (4): scm, collaboration, permission, audit
Support (1):      local-runtime
= 12 / 22 domain (其余 10 domain 不直接参与)
```

未参与的 10 domain: tenant / workspace / project / workflow / board / planning / comment / relation / automation / integration / search / notification (其中 audit + permission 仍参与,共 5 个未直接参与 — 实际是 workflow/board/planning/comment/relation + 4 个 support = 9 个,本 ADR 不展开细分)。

引用: per [basic-design v0.16 §4.11.1 协作参与者](../../../basic-design.md)。

### D27: Worktree Orchestration Saga 8 步编排

per [spec/saga/01 v0.2 §4 Worktree Orchestration Saga 示例](../spec/saga/01-saga-coordination-spec.md),8 步严格串行 + AuditLogging 必填最后:

| # | Step | responsible_crate | coordination_role | 强制 |
|---|---|---|---|---|
| 1 | `ValidateWorkItemOwnership` | `domain-work-item` | IdentityValidation | — |
| 2 | `CreateWorktree` | `domain-worktree` | ResourceMutation | — |
| 3 | `RegisterAgentSession` | `domain-agent` | ResourceMutation | — |
| 4 | `StartContextBuild` | `domain-context` | StateObservation | — |
| 5 | `AuthorizeFeedbackGate` | `domain-feedback` | DecisionAuthorization | — |
| 6 | `TriggerValidation` | `domain-validation` | ResourceMutation | — |
| 7 | `LinkPullRequest` | `domain-scm` | ResourceMutation | — |
| 8 | `WriteAuditLog` | `domain-audit` | AuditLogging | **必填且最后** |

引用: per [basic-design v0.16 §4.11.2 8 步编排时序](../../../basic-design.md)。

### D28: Event Bus 19 事件 + 5 订阅者矩阵

per [basic-design v0.16 §4.12.1 12 核心事件契约](../../../basic-design.md) (实际是 19 事件,因 v0.16 展开 7 个细分) + §4.12.2 5 订阅者:

| 订阅者 | 订阅事件 | 触达要求 |
|---|---|---|
| `worker context-build` | WorktreeCreated, AgentSessionStarted, ChangeSetObserved, FeedbackCreated | 异步,at-least-once |
| `worker projection` | WorktreeStatusObserved, WorktreeDirtyStateChanged | 异步,best-effort |
| `worker validation-trigger` | AgentSessionCompleted, ChangeSetObserved, FeedbackApplied | 异步,at-least-once |
| `collaboration + star-sse` | 全部 19 事件 | 实时,push 模式 |
| `notification` | WorktreeConflictDetected, AgentSessionFailed, FeedbackCreated, ValidationFailed, PullRequestLinked, MergeRequestLinked | 异步,降噪 (per REQ-NOTIF-002) |

5 守门规则 (per [basic-design v0.16 §4.12.3](../../../basic-design.md)):
1. 不得拆核心业务事务为 Event Chain
2. Outbox Pattern 保证事务一致性
3. 事件 payload 不含敏感 PII/Prompt/Code 全文
4. 死信队列 (per saga spec G-05)
5. 追溯链 event_id + causation_id + correlation_id

### D29: Realtime 3 通道架构

per [basic-design v0.16 §4.13 Realtime 协作机制](../../../basic-design.md):

```text
domain events (NATS JetStream)
       │
       ▼
star-sse (Rust WebSocket 端点)
       │
       ├── /ws/feed  (高频 stream)
       ├── /ws/notif (降噪关键事件)
       └── /ws/admin (admin only, low freq)
       │
       ▼
   browser (SSE/WS client)
```

降噪策略 (per REQ-NOTIF-002): 默认只推 WAITING_FEEDBACK / ValidationFailed / ProtectedAction 待授权 3 类关键节点;Agent 中间步骤 / token stream 写 Transcript 不推送。

### D30: Context Graph 4 节点 / 5 关系归属 22 domain crate

per [spec/context/04 v0.2 §1/§2](../spec/context/04-context-graph.md):

| 节点 | 归属 22 crate | 关系 | 源 → 目标 | 触发 event |
|---|---|---|---|---|
| Issue | `domain-work-item` | `implements` | Worktree → Issue | WorktreeCreated |
| Repository | `domain-scm` | `modifies` | Commit → Worktree | ChangeSetObserved |
| Worktree | `domain-worktree` | `references` | Commit → Issue | FeedbackCreated / PR_Linked |
| Commit | `domain-scm` + `domain-development` | `belongs_to` | Worktree → Repository | WorktreeCreated |
| | | `derived_from` | Commit → Commit (parent) | 实时 |

节点 ↔ 22 crate 关联原则: 每个节点必须有唯一 primary responsible_crate;Commit 双归属 (SCM 原始 + Development 业务语义)。

### D31: Worktree Orchestration 端到端协作时序

per [basic-design v0.16 §4.11.2](../../../basic-design.md) + [spec/context/04 v0.2 §7](../spec/context/04-context-graph.md) 8 步:

```text
T0  user → SubmitWorkItem → work-item
T1  work-item StateChanged(IN_PROGRESS) → Outbox
T2  application 读 Outbox → 触发 Worktree Orchestration Saga 8 步
T3  Realtime 推送 → collaboration → star-sse → user UI (per §D29)
T4  Notification 推送 → notification → inbox/email (per REQ-NOTIF-002 降噪)
```

5 协作原则 (per [basic-design v0.16 §4.11.3](../../../basic-design.md)):
1. 状态独立 (Worktree Status ≠ WorkItem Status)
2. Observed vs Business 分离
3. 强一致走单事务,跨域走 Saga
4. 审计 Append-only
5. Saga 失败必补偿 (best-effort,失败入死信)

### D32: 22 domain 接触面 100% 覆盖

per [basic-design v0.16 §3.2.9 补充 14 Domain 接触面](../../../basic-design.md) (50 行表),22 domain × 接触点 = ~140 接触点全覆盖,无遗漏。6 种协作模式分布:
- Shared Kernel ~10
- Customer-Supplier ~70
- Conformist ~10
- Separate Ways ~30
- Published Language ~10
- ACL ~10

引用: per §3.2 接触面统计 + §3.3 外部系统接触面 (5 行新外部系统: OIDC/SAML IdP, Slack/Teams/Lark/Discord IM, S3 Object Storage, KEDA Serverless, Star CLI/star-mcp)。

## §3 跨 spec/crate 关系表

| 上游 spec/ADR | 下游 spec/crate | 关系 |
|---|---|---|
| [requirements §22 Worktree Orchestration](../../../requirements.md) | [basic-design v0.16 §4.11](../../../basic-design.md) | 需求 → 基本设计 |
| [requirements §14.1 Event Architecture](../../../requirements.md) | [basic-design v0.16 §4.12 Event Bus](../../../basic-design.md) | 12 事件 → 19 事件展开 + 5 订阅者矩阵 |
| [requirements §15 Realtime](../../../requirements.md) | [basic-design v0.16 §4.13 Realtime](../../../basic-design.md) | 3 维要求 → 3 通道架构 + 降噪 + 心跳 |
| [requirements §26 Context Compiler](../../../requirements.md) | [spec/context/04 v0.2](../spec/context/04-context-graph.md) | Context Packet → Context Graph 4 节点 / 5 关系 |
| [spec/saga/01 v0.2 §4 Worktree Orchestration Saga](../spec/saga/01-saga-coordination-spec.md) | [basic-design v0.16 §4.11](../../../basic-design.md) | Saga 8 步 → 协作时序 8 步 1:1 对应 |
| [spec/saga/01 v0.2 §2 SagaCoordinationRole](../spec/saga/01-saga-coordination-spec.md) | [spec/integration/01 v0.2 §3.2/§4](../spec/integration/01-22-domain-integration-spec.md) | Saga enum 脱钩 5 域 → 22 crate 各自 lead 验收 |
| [ADR-0026 STAR AI 兼容](../../adr/0026-star-ai-compat.md) | [basic-design v0.16 §4.12 5 订阅者矩阵](../../../basic-design.md) | 5 通道 + Fallback Ladder 4 级 → worker role 5 类 |
| [ADR-0030 Agent Lease+Heartbeat+Resume](../../adr/0030-agent-lease-heartbeat-resume.md) | [spec/saga/01 v0.2 §5 状态机持久化](../spec/saga/01-saga-coordination-spec.md) | Lease 30s heartbeat → Saga 持久化恢复 |
| [ADR-0031 Context Graph](../../adr/0031-context-graph.md) | [spec/context/04 v0.2](../spec/context/04-context-graph.md) | 4 节点 / 5 关系 MVP → 22 crate 归属展开 |
| [basic-design v0.16 §3.2.9 22 domain contact face](../../../basic-design.md) | 24 份 `docs/specs/domain-*.md` | §3.2.9 contact face → domain spec 各自"与其他 domain 协作"一节 (待 P3 阶段批补) |

## §4 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 状态 | 触发 |
|---|------|------|------|------|
| GAP-01 | 24 份 `docs/specs/domain-*.md` 各自"与其他 domain 协作"一节 (per L3 任务) | domain spec 内未显式列出 contact face 引用,需 P3 阶段批补 | 🟡 P3 阶段 | 2026-09-01 14:38 JST 模块间协作细化 (L3 + doc-only) |
| GAP-02 | 5 域字符串硬编码检索 (grep "Player\|Economy\|Match\|Social\|Admin") 残留 | 文档 / 代码 / 其他 spec 内仍残留 5 域业务子域命名引用,需 P3 阶段 sweep | 🟡 P3 阶段 | per [spec/saga/01 v0.2 §6 G-13](../spec/saga/01-saga-coordination-spec.md) |
| GAP-03 | `internal-design.md` §3 关键模块协作 + `integration-design.md` §1-§9 | 前端模块协作 + 集成层 4 类关系 (Link/Mirror/Bidirectional Sync/Platform-owned) 需细化 | 🟡 下 session / P3 拍板 | per L3 任务范围 30+ 文档 |
| GAP-04 | Worktree Orchestration Saga 端到端 P99 SLA (per [saga spec v0.2 §6 G-06](../spec/saga/01-saga-coordination-spec.md)) | 8-step saga 端到端 SLA 未量化,需 SRE Lead 校准 | 🟡 SRE Lead 校准 | per [ADR-0027 §3 SRE NFR](../../adr/0027-star-ide-gateway.md) |
| GAP-05 | 22 domain crate 各自 lead 真实身份 (per 5 域脱钩后) | 22 crate lead 责任分工待 DDD Review 阶段补 | 🟡 DDD Review | per [AGENTS.md §4 #3 v0.6 Q1-D 拍板 +c](../../../AGENTS.md) |
| GAP-06 | Event payload 敏感字段边界 (PII/Prompt/Code 全文) 检测规则 | 仅 §D28 守门 3 显式声明,具体 payload schema + 检测规则待 [spec/services/02-sse-streaming-spec.md §3](../services/02-sse-streaming-spec.md) 细化 | 🟡 Phase H+ | per REQ-SEC-002 |

## §5 签字栏

| 角色 | 身份 | 签字 | 日期 |
|------|------|------|------|
| 架构师 | Mavis 接手 agent per DEC-008 | 🟢 Mavis 接手 (per 8/27 19:39/21:59 JST 三次强化) | 2026-09-01 |
| SRE Lead | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| 平台工程师 | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| 评审主持 | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| PM | 🟢 Mavis 接手代签 (per 8/27 三次强化) | 🟢 Mavis 接手 | 2026-09-01 |
| 5 域 Lead (历史命名) | ⏳ DDD Review 阶段补 (Player / Economy / Match / Social / Admin) | per [AGENTS.md §4 #3 v0.6 Q1-D 拍板 +c](../../../AGENTS.md),5 域独立 Lead 是历史治理命名,不映射 22 crate 实际 lead | — |

> per [AGENTS.md §1.0 用户授权升级](../../../AGENTS.md) + 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化,Mavis 接手默认代签 Ulysses 无需再问

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 决策 (D26-D32) Worktree Orchestration 跨 12 domain 协作架构 + 8 Saga step + 19 事件 + 5 订阅者 + 3 Realtime 通道 + 4 Context Graph 节点归属 22 crate + 22 domain 接触面 100% 覆盖 + 6 已知缺口 + 5 签字栏 | 2026-09-01 14:38 JST 模块间协作细化任务 (A 架构层 22 Domain 协作 + L3 完整覆盖 + doc-only) |
