# Vibe Coding Work Management SaaS 要件定義書（統合拡張版 v2.0）

## 0. 文档说明与前提

本文档基于《Vibe Coding Work Management SaaS 要件定义提示词 — 整合增强版》生成，定位为：

```text
Kubernetes-native 工作管理 SaaS 要件定义（原文档）
        ↓ 扩展
Vibe Coding Work Management SaaS 要件定义书（本文档）
```

**重要前提说明**：本仓库（D:\Star）中未检索到被引用的原始《Kubernetes-native 工作管理 SaaS 要件定义》文档。因此本文档不是对原文档的"增量 diff"，而是一份**自包含（self-contained）的完整要件定义书**，按以下方式处理原文档依赖：

- 提示词中明确列出的架构原则（K8s-native / K3s、Rust Modular Monolith、PostgreSQL System of Record、Transactional Outbox、NATS JetStream、Worker、Service Promotion Model、低 K8s Tax）**全部照原样继承**，不做任何推翻或重新论证，具体见第 13、44 章。
- 提示词中未展开、但原文档理应存在的基础章节（Tenant/Workspace/Project、Workflow、Board/Backlog/Sprint、Permission、Data Model 等，对应 §26、§54、§55、§58-61、§90-94），本文档在第 1-17 章中**重新完整编写**，以保证 §101 要求的"从 Business Goal 到 Traceability 全面审查"可以成立。
- 若用户持有原文档，后续应以原文档为准对第 1-17 章做一致性校对；本文档在这些章节中不臆造与提示词矛盾的内容。

文档结构遵循提示词 §95 的章节升级方案：第 18-28 章为 Vibe Coding 核心扩展章节，第 29 章起为顺延章节（Observability、MVP、PoC、ADR、Risk、Traceability 等）。

**度量类数值**：全文遵守 §36、§80 的规定，凡缺乏真实测量数据的目标值，一律标注 `TBD-MEASURE`，不臆造具体百分比或数字。

---

## 1. 概要与目的

本文档定义 **Star** 平台的产品需求与架构义务（Architecture Obligation）。Star 的最终产品定义（详见第 100 章原则，§100）：

> 一个能够与 GitHub / GitLab 互通、拥有 Jira 核心工作管理能力，并以 Worktree 为 AI Coding 执行单元，将 Requirement、Agent Session、Context、Code Change、Structured Feedback、Validation 与 PR/MR 串联起来的 **Vibe Coding Development Control Plane**。

核心问题：

> 让一个人能够准确、高效、低认知负担地监督多个 Coding Agent，在多个独立 Worktree 中并行开发软件。

本文档的下游产出为《基本设计书》，因此本文档只定义 **要件（Requirement）与架构义务（Architecture Obligation）**，不输出数据库 DDL、API 具体实现或生产代码（§105）。

---

## 2. 产品重新定位

系统同时承担三种职责，且三者不得混为一个 Domain（§0）：

```text
1. Jira-class Work Management
2. GitHub / GitLab Development Integration
3. AI Coding Worktree Control Plane
```

AI-native 的含义边界（§0）：

```text
Software Development Intent
        ↓
Requirement → WorkItem → Worktree → Agent Session → Code Change
        ↓
Feedback → Validation → Commit → PR / MR → Delivery
```

必须形成完整、可观察、可追踪、可审计的开发闭环。系统不是 "AI Jira"，也不是 "AI GitHub Clone"，而是 **Vibe Coding Worktree Orchestration Platform / AI Coding Development Control Plane**（§1）。

传统系统各自回答的问题（§1）：

| 系统 | 核心问题 |
|---|---|
| Jira | 谁应该完成什么工作？ |
| GitHub / GitLab | 代码发生了什么变化？ |
| Coding Agent | AI 当前准备怎样修改代码？ |
| **Star（本系统）** | 见下方追踪链 |

Star 必须能回答的追踪链（§1）：

```text
WorkItem → Worktree → Agent → Agent 为什么这样修改 → 修改了哪些 Symbol → 产生了什么 Diff
   → 哪些 Requirement 已满足 → 哪些 Acceptance Criteria 未满足 → 哪些 Test 失败
   → 人类给出了什么 Feedback → AI 是否正确理解 Feedback → 下一次修改针对什么
   → 最终进入哪个 Commit / PR / MR
```

---

## 3. Persona

| Persona | 角色 | 核心诉求 |
|---|---|---|
| Product Owner / PM | 需求方 | 将 Business Goal 拆解为 WorkItem，跟踪交付进度 |
| Tech Lead / Architect | 架构守门人 | 定义 Architecture Constraint / Decision，审查 AI 修改是否越界 |
| Developer（人类） | 直接监督者 | 同时监督多个 Worktree/Agent，处理 Feedback Inbox 与 Intervention Queue |
| Coding Agent（Codex / Claude Code / Gemini CLI 等） | 执行者 | 在授权 Worktree 内，依据 Context Packet 完成代码修改 |
| Reviewer | 质量守门人 | 审查 PR/MR、Validation Evidence、Acceptance Coverage |
| Security / Compliance Officer | 安全合规 | 审计 AI Audit、Tenant Isolation、Local Runtime 安全边界 |
| Tenant Admin | 租户管理者 | 配置 Permission、Notification、Agent Policy、Provider Data Boundary |

非开发场景 Persona（保证 §85 的通用性）：

| Persona | 场景 |
|---|---|
| 设计 / 运营 / 文档协作者 | 使用 Board / Backlog / Sprint 管理非代码类 WorkItem，0 Repository / 0 Worktree |

---

## 4. 核心问题与产品目标

产品必须优先回答的问题清单（信息架构优先级，对应 §48、§82）：

```text
今天有哪些 WorkItem 正在开发？
哪些 Agent 正在运行？
哪些 Worktree 正在等待我的反馈？
哪些 Worktree Blocked？
哪些测试失败？
哪些 Worktree 互相冲突？
哪些反馈还没解决？
哪些代码已经 Ready for Review？
哪些 PR/MR 已经准备好？
哪个 Agent 最近偏离了需求？
```

UI 信息架构优先级（§82）：

```text
What needs my attention?
    > What is running?
    > What changed?
    > Why did it change?
    > What failed?
    > What should happen next?
    > Chat with AI
```

**AI Chat 不是系统架构中心，Worktree Control Center 才是**（§99）。

---

## 5. 用语定义（Glossary）

| 术语 | 定义 |
|---|---|
| WorkItem | Jira-class 工作单元，包括 Epic/Story/Task/Bug/Subtask/AI Task |
| Requirement | 业务需求，可关联多个 WorkItem 与 AcceptanceCriteria |
| Worktree | Vibe Coding 并行执行的隔离边界，一级领域对象（第 22 章） |
| AgentSession | 一次 Coding Agent 在某 Worktree 上的执行会话（第 24 章） |
| ChangeSet | 一次 Agent 修改产生的文件/符号级变更集合（第 21 章） |
| Feedback | 结构化的人类修正指令，非普通 Comment（第 25 章） |
| ContextPacket | Context Compiler 为 Agent 生成的最小必要上下文（第 26 章） |
| ValidationResult / Evidence | 证明 AI 修改是否真正满足 Acceptance Criteria 的证据（第 27 章） |
| DevelopmentExecution | WorkItem 在真实代码环境中一次或多次执行过程的聚合（第 20-21 章） |
| Local Runtime | 运行于开发者机器/企业 Runner 上的安全代理进程（第 23 章） |
| SoR | System of Record，业务事实的唯一来源（PostgreSQL，第 14 章） |
| Observed State | 高频、非业务事实性质的本地运行时状态（第 14、22 章） |

---

## 6. Domain Boundary 总览

系统逻辑领域（Logical Domain / Module，非 Deployment，§54）：

```text
Identity
Tenant
Workspace
Project
Work Management
Workflow
Planning
Collaboration
Permission
Automation
Integration
SCM
Development Context
Development Execution
Worktree
Agent
Feedback
Context
Validation
Audit
Search
Notification
```

Domain 边界约束：

- Domain 层不得出现厂商特有对象（`GitHubPullRequestObject` 等，§24）。
- WorkItem ≠ Git Branch ≠ Worktree ≠ AgentSession（§85）。
- 一个 WorkItem 可以关联 0/1/N 个 Repository，可以关联 0/1/N 个 Worktree（§85）。这保证系统仍可服务非开发类项目管理场景。

---

## 7. Tenant / Workspace / Project 要求

沿用 Jira-class 基础模型（§26）：

```text
Tenant
  ↓
Workspace
  ↓
Project
```

要求：

- REQ-TWP-001：系统必须支持多租户隔离，Tenant 为最高安全边界（详见第 16 章）。
- REQ-TWP-002：Workspace 用于组织多个 Project，Project 支持多种模板（软件开发、看板、Scrum 等）。
- REQ-TWP-003：Project 必须可独立配置 Workflow、Permission Scheme、Notification Scheme、Agent Policy（第 24 章）。

---

## 8. WorkItem / Workflow 要求

### 8.1 WorkItem 类型

```text
Epic
Story
Task
Bug
Subtask
AI Task（新增，第 27 节，§27）
```

**AI Task** 不是"由 AI 创建的 Task"，而是"预计主要由 Coding Agent 执行、但受人类需求和 Acceptance Criteria 控制的开发工作单元"（§27），可包含：

```text
Objective
Repository Scope
Allowed Files
Forbidden Files
Acceptance Criteria
Agent Policy
Validation Policy
Context Policy
```

### 8.2 Workflow

- REQ-WF-001：默认最简三态工作流（待办 / 进行中 / 完成），支持自定义状态扩展（前次对话结论：默认给出简化方案，不强制可视化工作流配置器，属于 MVP 精简范围，见第 30 章）。
- REQ-WF-002：Worktree Status 不等于 WorkItem Status（§4）。同一 WorkItem 下，Worktree A 可为 Agent Running，Worktree B 可为 Blocked，Worktree C 可为 Reviewing，系统必须允许该并存状态。

---

## 9. Planning 要求（敏捷规划）

沿用最小敏捷闭环（前次对话结论 + §26）：

```text
Backlog → Sprint 规划 → 看板执行（含甘特图排期视图）→ 燃尽图反馈 → 下一轮 Sprint
```

- REQ-PLAN-001：Backlog 统一待办池，支持排序与故事点估算。
- REQ-PLAN-002：Sprint 支持计划/开始/结束时间盒管理。
- REQ-PLAN-003：Board 同时支持 Kanban（持续流）与 Scrum 板视图，二者共享同一份 WorkItem 数据模型，不做成两套系统。
- REQ-PLAN-004：甘特图（Gantt）基于 WorkItem 的开始/截止日期与依赖关系生成，与 Sprint/看板共享同一份问题数据，是"看板"的排期视图变体，不是独立子系统。
- REQ-PLAN-005：燃尽图（Burndown）为 Sprint 内剩余工作量趋势展示，是敏捷闭环反馈的最小必需图表；速度图 / 累积流图 / 控制图为进阶分析，列入 V1（第 30 章）。
- REQ-PLAN-006（Agent-aware Planning，§35）：Backlog/Sprint 应研究提供 Agent Suitability、Parallelizable、Context Cost、Conflict Risk、Dependency Risk、Human Review Cost、Validation Cost 等规划辅助信息，属于 Planning Assistance，不构成 AI 强制调度。

---

## 10. Collaboration 要求

- REQ-COLLAB-001：评论 + @提及 + 附件为问题详情页标配。
- REQ-COLLAB-002：问题关联（Relation）至少支持阻塞 / 被阻塞 / 关联，是甘特图依赖排期与 Worktree 冲突分析的数据基础。
- REQ-COLLAB-003：实时状态同步为多人协作基础体验（第 15 章 Realtime）。
- REQ-COLLAB-004：Agent Chat 必须关联 WorkItem / Worktree / AgentSession / Feedback / Context，不得形成孤立 Chat Thread（§83）。用户在 Chat 中表达的重要规则，系统应支持将其提升为 Structured Feedback 甚至 Decision / Constraint（§83、第 25-26 章）。

---

## 11. Permission & Automation 要求

- REQ-PERM-001：项目级、角色级细粒度权限控制（Permission Scheme）。
- REQ-PERM-002：Agent 相关操作（第 24、28 章 Agent Policy）必须由 Application / Authorization 层强制执行，不得仅通过 Prompt 约束（§28）。
- REQ-AUTO-001：自动化规则采用触发器-条件-动作模式，MVP 提供默认方案，不强制可视化配置器（第 30 章范围裁剪）。

---

## 12. Notification & Search 要求

- REQ-NOTIF-001：事件触发的邮件/站内通知，覆盖 WorkItem 状态变更、Feedback 请求、Validation 失败等（详见第 25、27 章事件源）。
- REQ-SEARCH-001：Search 为 Projection，不得成为业务事实源（§90）。初期覆盖 WorkItem / Comment / Project，未来扩展 Repository / Worktree / AgentSession / Feedback / Decision / Symbol（§90）。
- REQ-SEARCH-002（精简范围）：MVP 不做 JQL 高级查询语言，以 Filter（状态/负责人/标签/Sprint）替代（前次对话结论），降低学习成本。

---

## 13. Architecture 总览

**不得推翻的既有架构原则**（提示词导言 + §56、§86）：

```text
Kubernetes-native / K3s
Rust Modular Monolith
PostgreSQL System of Record
Transactional Outbox
NATS JetStream
Worker
Service Promotion Model
低 K8s Tax
```

### 13.1 服务器端物理架构（§56，保持不变）

```text
                       Internet
                           │
                  Gateway API / Ingress
                           │
                        Gateway
                           │
              ┌────────────┴────────────┐
              │                         │
          Identity                  Work Core
                                        │
                             ┌──────────┴──────────┐
                             │                     │
                         PostgreSQL              Valkey
                             │
                    Transactional Outbox
                             │
                             ▼
                       NATS JetStream
                             │
                             ▼
                           Worker
```

可选 `realtime` 服务，仅在出现真实 Long Connection Scaling Boundary 时才拆出（§56）。

### 13.2 Development Runtime 不打破服务器架构（§57）

```text
GitHub / GitLab
       │ Integration
       ▼
┌─────────────────────────────┐
│        SaaS Control Plane   │
│ Work / Workflow / Feedback  │
│ Context / Agent Metadata    │
│ Worktree Observed State     │
└──────────────┬──────────────┘
               │ Secure Runtime Channel
┌──────────────▼──────────────┐
│        Local Runtime        │
│ Git / Worktree / Agent Process │
│ Build / Test / Symbol Analysis │
└──────────────┬──────────────┘
        ┌──────┼──────┐
        ▼      ▼      ▼
      WT-A   WT-B   WT-C
      Agent  Agent  Agent
```

### 13.3 Rust Modular Monolith 扩展（§55）

`work-core` 内部逻辑代码结构（16 crates ≠ 16 services ≠ 16 deployments）：

```text
crates/
├── domain-tenant
├── domain-workspace
├── domain-project
├── domain-work-item
├── domain-workflow
├── domain-board
├── domain-planning
├── domain-permission
├── domain-comment
├── domain-relation
│
├── domain-development
├── domain-worktree
├── domain-agent
├── domain-feedback
├── domain-context
├── domain-validation
├── domain-scm
│
├── application
├── infrastructure
└── api
```

### 13.4 Worker 扩展（§88）

```text
worker
├── notification
├── webhook
├── automation
├── projection
├── integration
├── maintenance
├── scm-sync
├── context-build
└── repository-analysis
```

第一阶段 `worker --role all`，未来按真实负载拆分独立 Scaling（如 `worker --role repository-analysis`）。

### 13.5 Serverless / KEDA 候选（§89）

适合 Scale-to-Zero 的任务：Repository Analysis、Large Context Build、PR Analysis、Static Analysis、Agent Session Post-processing、Diff Summarization、Dependency Scan。是否引入需比较 Resource Saving vs Operational Complexity，不因 Vibe Coding 提前部署（§89）。

---

## 14. Data Model 总览

- REQ-DATA-001：PostgreSQL 是 System of Record，保存 WorkItem、Requirement、AcceptanceCriteria、Worktree Registration、DevelopmentExecution、AgentSession Metadata、Feedback、Decision、ContextPacket Metadata、ValidationResult、SCM Link、Audit 等业务事实（§59）。
- REQ-DATA-002：大型 Raw Diff / Large Log / Build Artifact / Agent Transcript / Binary 需评估 PostgreSQL vs Object Storage 的合理边界，不得把无限 Agent Transcript 塞入 PostgreSQL 热表（§59）。
- REQ-DATA-003：Worktree Observed State（`dirty=true`、`agent=running`、`tests=41/44` 等）属于 Observed State，可保存在 Projection / Snapshot 中，不要求每个 filesystem event 进入核心事务历史，必须控制 Write Amplification / Event Volume / Database Growth / Observability Cardinality（§60）。

### 14.1 Event Architecture 扩展（§58）

```text
WorktreeCreated / WorktreeAssigned / WorktreeStatusObserved
WorktreeDirtyStateChanged / WorktreeConflictDetected
AgentSessionStarted / AgentSessionCompleted / AgentSessionFailed
ChangeSetObserved
FeedbackCreated / FeedbackAcknowledged / FeedbackApplied / FeedbackVerified
ValidationStarted / ValidationPassed / ValidationFailed
ContextPacketCreated
PullRequestLinked / MergeRequestLinked
```

原则：Event Bus 用于外围解耦，不得把核心业务事务拆成 Event Chain（§58）。

---

## 15. Realtime 要求

- REQ-RT-001：实时展示 Agent Status、Worktree Status、Test Result、Build Result、Feedback Request、Conflict Warning（§61）。
- REQ-RT-002：用户应能近实时看到 `Agent Running → Validation → Waiting Feedback → Running` 的状态流转。
- REQ-RT-003：高频 Token Stream 不一定需要进入 SaaS Server，须区分 Persistent Business Event 与 Ephemeral Realtime Signal（§61）。

---

## 16. Security & Tenant Isolation 要求

- REQ-SEC-001（Tenant Isolation 扩展，§91）：除原有 Tenant Isolation P0 外，必须额外覆盖 Repository Credential、Local Runtime、Worktree、AgentSession、ContextPacket、Feedback、AI Prompt、AI Response、Diff、Build Log、Test Log、PR Content、Symbol Index 的隔离边界，任何遗漏 `tenant_id` 或等效隔离边界都可能造成严重数据泄漏。
- REQ-SEC-002（企业私有代码要求，§92）：支持 Tenant / Project 级 Policy：`Cloud AI Allowed` / `Cloud AI Restricted` / `Local AI Only` / `Specific Provider Allowed` / `No Code Upload` / `Metadata Only`。Context Compiler 和 Agent Adapter 必须遵守这些 Policy。
- REQ-SEC-003（Provider Data Boundary，§93）：对每个 AI Provider 必须能表达 Provider / Model / Region / Data Sent / Retention Policy / Credential / Tenant Policy / Project Policy，AI 不得为了方便绕过企业数据边界。

详细威胁模型见第 34 章，详细 Local Runtime 安全边界见第 23 章。

---

## 17. Audit 要求（基线）

- REQ-AUDIT-001：Audit 记录覆盖创建 / 修改 / 权限变更 / 删除等基础操作。
- REQ-AUDIT-002（AI Audit 扩展，第 28 章详述）：Audit 必须能够回答"谁要求 AI 做什么、AI 使用了什么 Context、AI 修改了什么、哪个 Agent 执行、在哪个 Worktree、什么时间、哪些验证通过、哪些 Feedback 被消费、谁批准 Commit/PR/Merge"。敏感 Prompt/Code 不默认进入普通日志，需单独定义 AI Audit Metadata 与 AI Content Retention Policy（§40）。

---

## 18. Integration 要求

平台与外部系统的职责划分原则（§23）：

```text
Platform                          GitHub / GitLab
├── Development Intent            ├── Repository
├── WorkItem                      ├── Branch
├── Worktree                      ├── Commit
├── Agent Session                 ├── Pull Request / Merge Request
├── Feedback                      ├── CI
├── Context                       └── Review
└── Execution State
```

不得重新制造 GitHub / GitLab。平台必须能够：Repository Sync、Branch Sync、Commit Link、Issue Link、PR/MR Link、Review Sync、Build Status、Pipeline Status、Webhook、Merge Status（§23）。

### 18.1 Bidirectional Link 原则（§25）

必须明确区分四类关系，不得盲目双向同步：

| 关系类型 | 说明 |
|---|---|
| Link | 仅建立引用关系 |
| Mirror | 单向镜像 |
| Bidirectional Sync | 双向同步（需谨慎评估，防止 Infinite Sync Loop） |
| Platform-owned | 数据所有权归平台 |

必须定义：Source System、Ownership、Version、External ID、Sync Token、Last Synced、Conflict Strategy（§25）。例如 Platform WorkItem ↔ GitHub Issue 是否真的需要完全双向，必须逐一分析而非默认全部双向同步。

---

## 19. SCM / GitHub / GitLab 要求

### 19.1 SCM Adapter 模型（§24）

```text
SCM Port
      │
 ┌────┴────┐
GitHub   GitLab
```

未来扩展候选：Gitea、Forgejo、Bitbucket、Azure DevOps、Self-hosted Git（§24）。

- REQ-SCM-001（P0，§63）：GitHub / GitLab 必须通过统一 SCM Adapter 接入。
- REQ-SCM-002：Domain 层只允许出现 `Repository / Branch / Commit / PullRequest / Review / Pipeline`，不得出现 `GitHubPullRequestObject` / `GitLabMergeRequestEntity` 等厂商污染对象（§24）。

### 19.2 Repository Ownership（§47）

必须区分：Connected Repository / Mirrored Repository / Managed Repository / Local-only Repository。初期系统不承担完整 Git Server 职能，GitHub/GitLab 继续作为远端 SCM 事实来源，避免项目范围膨胀为"自建 GitHub + GitLab + Jira + IDE"（§47）。

---

## 20. Development Context 要求

Development Context 是 WorkItem 与真实代码环境之间的抽象层，进一步细化为 Development Execution（第 21 章）。

核心关系模型（§2）：

```text
WorkItem
   ├── Requirement
   ├── AcceptanceCriteria
   └── DevelopmentExecution
            ├── Repository
            ├── Branch
            ├── Worktree
            ├── AgentSession
            ├── ChangeSet
            ├── ValidationResult
            ├── Feedback
            ├── Commit
            └── PullRequest / MergeRequest
```

系统必须将以下对象提升为一级领域概念（§2）：`WorkItem, Worktree, AgentSession, ChangeSet, Feedback, ContextPacket, ValidationResult, PullRequest/MergeRequest`。

---

## 21. Development Execution 要求

Development Execution 表示"一个 WorkItem 在真实代码环境中的一次或多次执行过程"（§6）。

```text
DevelopmentExecution
├── WorkItem
├── Repository
├── Worktree
├── AgentSession[]
├── ChangeSet[]
├── Feedback[]
├── Validation[]
├── Commit[]
└── PullRequest / MergeRequest[]
```

- REQ-DEV-001：必须支持 `1 WorkItem → N Worktrees`。
- REQ-DEV-002：必须支持 `1 Worktree → N Agent Sessions`。
- REQ-DEV-003（默认约束）：`1 AgentSession → 1 Active Worktree`，除非未来出现明确的 Multi-Worktree Agent Use Case（§6）。

### 21.1 ChangeSet（§9）

不得只保存 Git Diff，必须建立 ChangeSet 概念：

```text
ChangeSet
├── Files
├── Symbols
├── DiffReference
├── AddedLines / DeletedLines / RenamedFiles / GeneratedFiles
├── DependencyChanges
├── SchemaChanges
├── ConfigChanges
├── TestChanges
└── RiskSignals
```

必要时可通过本地分析器获得 AST / Symbol / Call Graph / Dependency 信息，但 MVP 不因此强制引入 Graph Database（§9）。

### 21.2 Symbol-aware Development Context（§10）

逐步从 File-level Context 提升到 Symbol-level Context：

```text
Repository → Module → File → Symbol → Reference → Dependency
```

目标不是建立完整 IDE Compiler Database，而是支持 Feedback Targeting、Context Selection、Change Impact、Conflict Detection、Agent Guidance（§10）。

---

## 22. Worktree Orchestration 要求

### 22.1 Worktree 作为一级领域对象（§3，P0：WT-001~003）

不得仅设计为 Repository Metadata 或 Branch 的附属字段。字段至少包括：

```text
Worktree
├── WorktreeId / TenantId / WorkspaceId / ProjectId
├── RepositoryId / WorkItemId
├── Branch / BaseBranch
├── LocalPathReference
├── Machine / Runner
├── Owner / Agent / AgentSession
├── Status / Health / DirtyState
├── Ahead / Behind / ConflictState
├── ChangedFiles / ChangedSymbols
├── TestState / BuildState
├── ContextState / FeedbackState
├── LastActivity
└── SynchronizationState
```

`LocalPathReference` 不得意味着 SaaS Server 可直接读取用户任意本地文件，必须通过 Local Runtime / Local Daemon 安全代理（第 23 章）。

### 22.2 Worktree 生命周期（§4）

候选状态：

```text
CREATED → READY → ASSIGNED → AGENT_RUNNING → WAITING_FEEDBACK
→ FEEDBACK_RECEIVED → VALIDATING → BLOCKED / CONFLICTED
→ READY_FOR_REVIEW → REVIEWING → READY_FOR_COMMIT → COMMITTED
→ PR_OPEN → MERGED → ABANDONED → ARCHIVED
```

原则：**Worktree Status 不等于 WorkItem Status**。必须研究 Worktree Lifecycle 与 WorkItem Workflow 之间的映射关系，不得硬编码状态耦合（§4）。

### 22.3 Worktree Control Center（§5）

系统主页除 Board / Backlog / Sprint / Roadmap 外，必须新增 **Worktree Control Center**，至少可查看：Worktree、Task、Agent、Agent Session、Status、Branch、Changed Files、Changed Symbols、Diff Size、Tests、Build、Conflict、Context Usage、Feedback、PR/MR、Last Activity；必须支持 Filter / Sort / Group / Search / Saved View（如 Group by Agent / Project / WorkItem / Status / Repository）。

### 22.4 Worktree Conflict Intelligence（§32-34）

第一阶段 File-level Conflict（如 WT-A 与 WT-B 同时修改 `auth.rs` → Risk = High），逐步发展到 Symbol-level Conflict。Development Dependency Graph 第一阶段由 PostgreSQL Relation + Projection 实现，只有真实数据规模证明需要，才评估引入 Graph Database（§33）。**Worktree Heatmap** 展示 Repository/Module/File/Symbol 正被哪些 Worktree 修改，用于 Conflict Awareness、Parallel Planning、Worktree Scheduling、Human Oversight（§34）。

### 22.5 Worktree Isolation（§43）

多 Agent 同机运行时，必须隔离：Filesystem、Environment Variable、Build Artifact、Dependency Cache、Agent Memory、Context、Secret、Port、Process、Temporary File。

### 22.6 Worktree Reconciliation（§45）

Local Runtime reconnect 后必须支持 Desired State ↔ Observed State 的 Reconciliation，但第一阶段保持应用层状态同步即可，不建立 Kubernetes-style CRD/Controller 系统。

### 22.7 Worktree Completion 判定（§78）

允许进入 `READY_FOR_REVIEW` 前至少考虑：No Critical Feedback、Required Tests Pass、Required Build Pass、No Blocking Conflict、Acceptance Criteria Covered、Required Review Complete、Git State Known。具体策略由 Project Policy 定义。

---

## 23. Local Runtime 要求

### 23.1 架构定位（§19）

```text
SaaS Control Plane
      │ HTTPS / WebSocket
   Gateway → Development Domain
      │ Secure Channel
Local Runtime / Daemon（候选实现：Rust Local Daemon）
      │
  ┌───┼───┐
 Git Worktree Agent
```

Local Runtime 不属于 Kubernetes Application Workload 数量，服务器端最小闭环（`gateway / identity / work-core / worker`）保持不变。

### 23.2 Local Runtime Security Boundary（§20，P0：LRT-001/002）

必须研究：Device Identity、Device Registration、User Binding、Tenant Binding、Project Binding、Repository Authorization、Short-lived Credential、Mutual Authentication、Command Authorization、Command Scope、Filesystem Scope、Process Scope、Secret Isolation、Agent Credential Isolation、Audit、Revocation、Remote Disable。

**默认禁止 `SaaS Server → Arbitrary Shell`**。必须建立有限能力接口，例如：

```text
GitStatus / CreateWorktree / ReadDiff / RunApprovedTest
QueryAgentStatus / SubmitFeedback / StartAuthorizedAgentSession
```

而不是 `execute(any_command)`（§20）。

### 23.3 Local-first State（§21）

区分 Server Truth（WorkItem/Feedback/Requirement/Permission）与 Local Observation（Dirty Files/Local Git Status/Running Agent PID/Current Worktree Path/Local Test Process），同步后形成 Observed Development State，不得将瞬时 Local State 当成永久业务事实。

### 23.4 State Synchronization（§22）

研究 Snapshot、Incremental Event、Heartbeat、Sequence、Version、Offline、Reconnect、Replay、Conflict、Idempotency、Stale State。UI 必须区分 Current / Possibly Stale / Offline / Unknown，不得显示虚假的实时状态。

### 23.5 Local Runtime Fault Model（§44）

必须考虑：Developer Machine Offline、Daemon Crash、Agent Crash、Git Lock、Worktree Deleted、Repository Moved、Branch Rebased、Force Push、Disk Full、Build Process Hung、Credential Expired、Network Interrupted、Version Mismatch。SaaS UI 禁止把最后一次状态永久显示成 "Running"。

### 23.6 Runtime 抽象扩展（§46）

未来允许 Developer Laptop / Self-hosted Runner / Enterprise Build Machine / Cloud Workspace / Ephemeral Coding Environment 作为 Development Runtime，Domain 层使用 `Runtime` 抽象（`LocalMachine / SelfHostedRunner / CloudWorkspace / FutureRuntime`），Worktree 运行于 Runtime 之上。

---

## 24. Agent Session 要求

### 24.1 AgentSession 字段（§7）

```text
AgentSession
├── SessionId / AgentType / AgentProvider / AgentVersion
├── WorktreeId / WorkItemId
├── StartedAt / EndedAt / Status
├── Intent / ContextPacket / Plan / Decisions
├── ToolActivitySummary
├── ChangeSet / ValidationResult / FeedbackConsumed
├── ResultSummary
└── TraceReference
```

### 24.2 Agent Adapter 模型（§7）

不得绑定单一厂商，通过 Agent Port 接入：

```text
Agent Port
    ├── Codex Adapter
    ├── Claude Code Adapter
    ├── Gemini CLI Adapter
    ├── OpenAI Compatible Adapter
    ├── Local Agent Adapter
    └── Future Agent Adapter
```

Domain 层不得出现厂商特有对象。

### 24.3 AI Task 与 Agent Policy（§27-28）

AgentPolicy 至少研究：Allowed Repository、Allowed Worktree、Allowed Path、Allowed Tool、Allowed Command Category、Network Access、Secret Access、Max Runtime、Max Context、Max Change Scope、Require Review、Require Test、Require Approval。**Policy 必须由 Application / Authorization 层执行**，重要安全规则不能只靠 Prompt 告诉 Agent"不要修改 xxx"（§28）。

### 24.4 Human-in-the-loop 授权等级（§29）

| 动作 | 授权级别 |
|---|---|
| AI Analyze | Auto |
| AI Suggest | Auto |
| AI Modify Authorized Worktree | Policy Controlled |
| Commit | Policy Controlled |
| Push | User/Tenant Policy |
| PR Creation | User/Tenant Policy |
| Merge | Protected Action |
| Production Deployment | 单独授权 |

真正禁止的是：Unbounded Autonomous Modification、Cross-Worktree Modification、Unauthorized Repository Modification、Direct Database Modification、Uncontrolled Merge、Uncontrolled Production Deployment（§29）。系统核心场景就是"AI 在授权 Worktree 中修改代码"，不是"AI 完全不能写代码"。

### 24.5 Multi-Agent Control（§51-53）

允许 `Worktree A→Agent A / Worktree B→Agent B / Worktree C→Agent C` 并行，但 MVP 重点是 Visibility / Isolation / Feedback / Context / Validation / Conflict Awareness，不做 Agent Swarm / Agent Negotiation / Autonomous Planning Society（§51）。

**Agent Handoff**（§52）：接管同一 Worktree 时不得依赖发送全部聊天记录，应生成 Handoff Context Packet：`Objective / Current State / Completed Work / Open Work / Decisions / Open Feedback / Changed Symbols / Failed Tests / Constraints`。

**Agent Comparison**（§53）：同一 Task 由多个 Agent 并行产生 Worktree 对比 Diff/Tests/Complexity/Review Finding/Context Cost/Feedback Count，列为 V2 候选（第 32 章），不进入初始 MVP。

---

## 25. Feedback 要求

### 25.1 Feedback 作为一级领域对象（§11，P0：FBK-001/002）

禁止只设计成普通 Comment。字段：

```text
Feedback
├── FeedbackId / Target / Type / Severity / Intent / Reason
├── ExpectedBehavior / Preserve / Prohibit
├── AcceptanceCriteria
├── Author / Agent / Status
├── CreatedAt / ResolvedAt
```

Feedback Target 至少支持：WorkItem、Requirement、AcceptanceCriterion、Worktree、AgentSession、File、Symbol、Diff Hunk、Test、Build、Runtime Log、Architecture Decision、PullRequest、Review Finding。

Feedback Type 至少研究：Fix、Preserve、Refactor、Reject、Question、Constraint、Architecture、Security、Performance、Testing、Scope。

### 25.2 Precise Feedback（§12）

系统必须解决传统 Coding Agent Feedback"这里不对，重新做"信息密度不足的问题。示例：用户选中 `src/auth/service.rs::authenticate_user` 并提交 Type=Architecture Constraint / Expected=使用 AuthProvider abstraction / Preserve=Public API, Existing Error Model / Prohibit=Database Schema Change，系统生成结构化 Agent Instruction（Target/Required/Preserve/Do not/Acceptance）。需要研究"如何从结构化 Feedback 生成高密度、低歧义、低 Token 的 Agent Instruction"。

### 25.3 Feedback Loop 与状态机（§13）

```text
Agent Output → Human Review → Structured Feedback
→ Context Compiler → Agent Instruction → Agent Revision → Validation
```

Feedback 状态：`OPEN → ACKNOWLEDGED → APPLIED → VERIFIED / REJECTED / SUPERSEDED`。系统必须能判断哪些 Feedback 已被 AI 消费、哪些已修改、哪些通过验证、哪些仍未解决。

### 25.4 Feedback Inbox 与 Intervention Queue（§49-50）

**Feedback Inbox** 聚合：Agent Waiting Feedback、Failed Acceptance、Review Finding、Test Failure、Architecture Question、Conflict、Agent Clarification，用户不必进入每个 Agent Chat 才知道哪里需要介入。

**Intervention Queue（Needs Human 视图）** 按优先级展示，例如：

```text
P0  Security Decision
P1  Architecture Feedback
P1  Merge Conflict
P2  Test Failure
P2  Agent Question
P3  Optional Refactor
```

是人类同时管理多个 Coding Agent 的核心工作台。

---

## 26. Context Compiler 要求

### 26.1 定位（§14，P0：CTX-001/002）

Context Compiler 不是 LLM，而是"根据当前任务、代码状态、历史决策和反馈，为 Coding Agent 生成最小必要 Context Packet 的确定性/半确定性系统能力"。

输入：`WorkItem / Requirement / Acceptance Criteria / Worktree / Repository / Relevant Files / Relevant Symbols / Architecture Constraints / Previous Decisions / Previous Agent Sessions / Open Feedback / Failed Tests / Build Failure / Git Diff / PR Review / Agent Rules`。输出：`ContextPacket`。

### 26.2 Context Packet 字段（§15）

```text
ContextPacket
├── Intent / Objective / Scope
├── RelevantRequirements / AcceptanceCriteria
├── RelevantFiles / RelevantSymbols
├── ArchitectureConstraints / ExistingDecisions
├── CurrentChangeSet / OpenFeedback / FailedValidation
├── PreserveRules / ProhibitedChanges
├── ExpectedOutput / VerificationInstructions
```

目标是 Minimum Sufficient Context，而非 Maximum Context，须减少 Context Pollution、Repeated Prompt、Token Waste、Instruction Drift、Forgotten Constraint、Unrelated Modification。

### 26.3 Context Provenance（§16）

所有进入 AI 的重要 Context 必须可追溯来源（如 `Requirement REQ-102 / ADR-004 / Feedback FBK-221 / Test TEST-932 / File auth.rs / Symbol AuthService::login`）。AI 生成的重要 Decision 必须关联 Source Context、AgentSession、Timestamp、Worktree。不得形成无法解释来源的 "AI Memory Blob"。

### 26.4 Context Budget 与优先级（§17）

Context Compiler 须考虑 Token Budget、Priority、Freshness、Relevance、Authority、Duplication：

```text
P0  Explicit Human Constraint
P1  Acceptance Criteria / Security Requirement / Open Feedback
P2  Relevant Current Code / Failed Test
P3  Historical Discussion
P4  Low-confidence AI Summary
```

不得让历史 Agent 对话无限增长。

### 26.5 Decision Memory（§18）

与普通 Chat History 分开建立 Decision 对象（`Decision / Reason / Scope / Source / Status`），必须能够 Create / Supersede / Invalidate / Trace。Context Compiler 应优先使用 Active Decision，而不是重新发送完整聊天历史。

### 26.6 AI Memory 原则（§84）

禁止建立 Unlimited Chat Memory。推荐逻辑：`Conversation → Extract → Decision / Feedback / Constraint / Summary → Context Compiler`。原始聊天可保留作历史，但不默认重复发送。

---

## 27. Validation 要求

### 27.1 Validation Domain（§30，P0：VAL-001）

AI 修改不能以"Agent says done"作为完成条件。ValidationResult 须覆盖：Build、Unit Test、Integration Test、Lint、Format、Static Analysis、Security Check、Acceptance Check、Review、Custom Validation，并关联 WorkItem、Acceptance Criterion、Worktree、AgentSession、ChangeSet、Commit。

### 27.2 Acceptance Coverage（§31）

建立 `AcceptanceCriteria → ValidationEvidence` 映射（例：`AC-001` 关联 `TEST-201`、`Symbol Analysis SA-92`、`Human Review RV-12`）。目标是知道需求为什么可以判定为满足，而不仅是知道 Tests Passed。

### 27.3 AI Completion 判定（§77）

禁止 `Agent: Done → WorkItem Done` 的简单映射，必须经过：

```text
Agent Result → Validation → Acceptance Coverage → Feedback Resolution
→ Human / Policy Gate → Ready for Review
```

---

## 28. AI Extension 要求

### 28.1 AI Interaction Quality 指标（§36-39）

不能只监控 Token / Latency / Cost，还应研究（数值目标一律 `TBD-MEASURE`）：

```text
Feedback Iteration Count / First-pass Acceptance Rate / Rework Rate
Context Reuse Rate / Unrelated Change Rate / Constraint Violation Rate
Test Failure After Agent Change / Human Correction Count
Feedback Resolution Rate / Agent Session Success Rate / PR Review Finding Rate
Feedback Precision / Feedback-to-Fix Ratio / Feedback Repetition
Feedback Rejection / Feedback Reopen Rate
Input Context Tokens / Relevant Context Ratio / Repeated Context Ratio
Context Cache Hit / Session Count / Successful Completion
```

Agent Observability 指标（须谨慎处理高 Cardinality 标签，禁止把 Repository/Worktree/AgentSession/File/Symbol/Tenant 等 ID 直接作为 Prometheus Label，§39）：

```text
agent_session_duration / agent_session_success_rate / agent_feedback_count
agent_rework_count / agent_context_size / agent_change_files / agent_change_symbols
agent_validation_failure / worktree_active_count / worktree_conflict_count
worktree_stale_count / local_runtime_online / local_runtime_sync_lag
```

### 28.2 AI Audit（§40）

Audit 须回答第 17 章 REQ-AUDIT-002 列出的全部问题；敏感 Prompt/Code 需单独的 AI Audit Metadata 与 AI Content Retention Policy。

### 28.3 Prompt Injection / Repository Injection 威胁（§41）

新增威胁面：Malicious Repository/README/Issue/PR Comment/Test Output/Tool Output Instruction、Prompt Injection、Indirect Prompt Injection、Context Poisoning、Agent Tool Abuse、Secret Exfiltration、Cross-Worktree/Cross-Repository Data Leakage。**必须区分 Untrusted Repository Content 与 Trusted Human Policy，二者 Instruction Priority 不得相同**。

### 28.4 Agent Secret Boundary（§42）

不得把 GitHub/GitLab Token、Cloud Secret、Production Secret 无条件暴露给 Agent，须研究 Credential Broker、Scoped Token、Short-lived Token、Process Isolation、Environment Isolation、Secret Redaction。

### 28.5 Agent Chat 定位与 UX 原则（§82-83）

见第 4 章、第 10 章 REQ-COLLAB-004。AI Chat 是交互方式，不是系统架构中心（§99）。

---

## 29. Observability 要求

除基础设施可观测性（API / DB / NATS / Worker）外，产品层 Dashboard 须独立展示（§94，与基础设施监控区分）：

```text
Active Worktrees / Agent Sessions / Waiting Feedback / Blocked Worktrees
Conflict Risk / Validation Failure / Context Size / Agent Failure
Runtime Offline / SCM Sync Error
```

Agent Observability 具体指标见第 28.1 节，须遵守高 Cardinality 标签处理原则。

---

## 30. MVP / Roadmap 要求

### 30.1 MVP 两个必须共存的闭环（§64）

**Jira-class 闭环**：

```text
Tenant → Workspace → Project → WorkItem → Workflow → Board → Comment → Permission → History → Notification
```

**Vibe Coding 最小闭环**：

```text
WorkItem → Repository → Worktree → AgentSession → ChangeSet → Validation
→ Feedback → Agent Revision → Commit → PR/MR Link
```

两个闭环必须共同构成 MVP，缺一不可。

### 30.2 MVP Must Have（§65）

```text
GitHub Integration / GitLab Integration
Repository Link
Worktree Registration / Worktree Status / Worktree Dashboard
Agent Session Registration / Agent Status
File-level ChangeSet / Basic Symbol Detection
Structured Feedback / Feedback Inbox
Context Packet Generation
Build/Test Result
Basic Conflict Detection
Commit Link / PR/MR Link
Development Timeline
Local Runtime
Tenant-aware Security / Audit
```

### 30.3 V1 Should Have（§66）

```text
Symbol-level Feedback / Symbol-level Conflict
Decision Memory
Agent Handoff
Acceptance Coverage
Advanced Context Selection
PR Review Feedback Import
Saved Worktree Views
Development Heatmap
Agent Policy Templates
Remote Runner
Context Cost Analysis
```

### 30.4 V2 Candidates（§67）

```text
Semantic Conflict Detection / Impact Analysis
Cross-Worktree Dependency Graph
AI Planning Assistance
Multi-Agent Comparison
Task Parallelization Recommendation
Agent Performance Analytics
Advanced Runtime Isolation
Cloud Development Runtime
```

### 30.5 Future（§68）

```text
Agent Swarm / Autonomous Task Decomposition / Autonomous Multi-Agent Scheduling
Graph Database / Vector Database / Semantic Repository Memory
Cloud IDE / Managed Git Hosting
Autonomous Merge / Autonomous Deployment
```

只有验证价值后才研究。

### 30.6 Explicit Non-Goals（§69）

```text
GitHub Clone / GitLab Clone / Full Jira Enterprise Clone
Full IDE / Cloud IDE / Git Hosting Platform
Agent Swarm / Autonomous Company / Autonomous Production Deployment
Service Mesh / 几十个微服务 / Database per Domain
Graph Database / Vector Database / OpenSearch Cluster
Full Event Sourcing / Complex CQRS
```

---

## 31. PoC 一览

原有 PoC-001~015（沿用原文档，本文档不重新枚举，因原文档未在本仓库中提供，留待与既有 PoC 清单核对）。新增 PoC（§70）：

| ID | 内容 |
|---|---|
| POC-016 | Local Runtime Secure Connection |
| POC-017 | Worktree State Synchronization |
| POC-018 | Worktree Offline / Reconnect |
| POC-019 | Multiple Worktree Observation |
| POC-020 | Agent Session Tracking |
| POC-021 | Structured Feedback → Agent Instruction |
| POC-022 | Context Compiler |
| POC-023 | Context Packet Size / Relevance |
| POC-024 | File-level Conflict Detection |
| POC-025 | Symbol-level Feedback |
| POC-026 | GitHub Adapter |
| POC-027 | GitLab Adapter |
| POC-028 | Agent Adapter |
| POC-029 | Agent Policy Enforcement |
| POC-030 | Cross-Worktree Isolation |

---

## 32. ADR Candidates

原有 ADR-001~015（沿用原文档编号空间，待与既有 ADR 清单核对）。新增 ADR（§71）：

| ID | 标题 |
|---|---|
| ADR-016 | Worktree as First-class Domain Entity |
| ADR-017 | Development Execution Domain |
| ADR-018 | Local Runtime Architecture |
| ADR-019 | Local Runtime Security Model |
| ADR-020 | Observed State vs Business State |
| ADR-021 | Agent Adapter Model |
| ADR-022 | SCM Adapter Model |
| ADR-023 | Structured Feedback Model |
| ADR-024 | Context Compiler |
| ADR-025 | Context Packet Persistence |
| ADR-026 | Agent Session Persistence |
| ADR-027 | ChangeSet Storage |
| ADR-028 | Symbol Analysis Strategy |
| ADR-029 | Worktree Conflict Detection |
| ADR-030 | Agent Policy Enforcement |

---

## 33. Risk Register

原有 RISK-001~015（沿用原文档编号空间，待核对）。新增 Risk（§72）：

| ID | 风险 |
|---|---|
| RISK-016 | Local Runtime Compromise |
| RISK-017 | Agent Escapes Worktree Scope |
| RISK-018 | Agent Secret Leakage |
| RISK-019 | Cross-Worktree Context Leakage |
| RISK-020 | Cross-Repository Context Leakage |
| RISK-021 | Prompt Injection from Repository |
| RISK-022 | Stale Worktree State |
| RISK-023 | Agent Session State Divergence |
| RISK-024 | Context Explosion |
| RISK-025 | Low-quality Context Selection |
| RISK-026 | Feedback Misinterpretation |
| RISK-027 | SCM Sync Loop |
| RISK-028 | Worktree Conflict Explosion |
| RISK-029 | Local Runtime Version Fragmentation |
| RISK-030 | Agent Vendor Lock-in |

---

## 34. Security Threat Model 扩展

至少输出以下威胁（§73）：

```text
Malicious Repository Prompt Injection
Agent Unauthorized File Access
Agent Unauthorized Command Execution
Agent Credential Exfiltration
Cross Worktree Leakage
Cross Repository Leakage
Cross Tenant AI Context Leakage
Malicious GitHub/GitLab Webhook
Compromised Local Runtime
Context Poisoning
Fake Validation Result
Runtime Impersonation
```

---

## 35. Architecture Obligation（架构义务）

新增 Development 相关义务（§74）：

```text
ARCH-OBL-DEV-001  Worktree Isolation
  → Agent Execution 必须限制在明确授权的 Runtime / Repository / Worktree Scope。

ARCH-OBL-DEV-002  Context Traceability
  → 进入 Coding Agent 的关键 Requirement、Constraint、Feedback 必须能够追溯来源。

ARCH-OBL-DEV-003  SCM Independence
  → GitHub / GitLab 必须通过统一 Adapter 接入，不得污染 Work Management Domain。

ARCH-OBL-DEV-004  Local Runtime Security
  → Server 不得拥有对 Developer Machine 的无界远程 Shell 权限。

ARCH-OBL-DEV-005  Validation Evidence
  → AI Coding Task 完成必须存在可验证 Evidence，不得仅依赖 Agent 自我报告。

ARCH-OBL-DEV-006  Observed State
  → Worktree 高频本地状态必须与核心业务事实区分。
```

---

## 36. Use Case 一览

至少覆盖（§75）：

| ID | Use Case |
|---|---|
| UC-DEV-001 | Developer 从 WorkItem 创建 Worktree |
| UC-DEV-002 | Developer 将 Worktree 分配给 Coding Agent |
| UC-DEV-003 | Agent 在授权 Worktree 修改代码 |
| UC-DEV-004 | 系统展示 Agent / Worktree 实时状态 |
| UC-DEV-005 | Agent 修改后执行 Build / Test |
| UC-DEV-006 | Developer 对具体 Symbol 提交 Feedback |
| UC-DEV-007 | Context Compiler 生成 Feedback Revision Context |
| UC-DEV-008 | Agent 根据 Feedback 修改 |
| UC-DEV-009 | 系统验证 Feedback 是否解决 |
| UC-DEV-010 | 创建 Commit / PR / MR |
| UC-DEV-011 | 多个 Worktree 同时修改代码并产生 Conflict Warning |
| UC-DEV-012 | Agent A Handoff 给 Agent B |

---

## 37. Acceptance Criteria 示例

### AC 示例 1：Worktree 创建（§76）

```gherkin
Given  WorkItem DEV-100 已关联 Repository
And    用户拥有 Development Execute 权限
When   用户创建新的 Worktree
Then   系统生成唯一 Worktree ID
And    Worktree 与 WorkItem、Repository、Branch 建立关联
And    Local Runtime 返回真实 Worktree 状态
And    Audit 记录创建动作
And    其他 Tenant 不可访问该 Worktree
```

### AC 示例 2：结构化 Feedback（§76）

```gherkin
Given  Agent 已修改 AuthService::login
When   用户针对该 Symbol 创建 Architecture Feedback
Then   Feedback 必须关联对应 Symbol
And    Feedback 必须包含 Expected / Preserve / Prohibit
And    Context Compiler 下一次为该 Worktree 生成 ContextPacket 时包含该 Feedback
And    AgentSession 必须记录 Feedback 已被消费
And    系统不得因为 Feedback 自动修改未经授权的其他 Worktree
```

---

## 38. AI / Worktree Completion 判定基准

见第 27.3 节（AI Completion 判定）与第 22.7 节（Worktree Completion 判定）。核心原则：**AI 自我报告"完成"不构成完成的充分条件**，必须经过 Validation → Acceptance Coverage → Feedback Resolution → Human/Policy Gate。

---

## 39. Traceability Model

完整追踪链（§79，对应第 96 章 E 图）：

```text
Business Goal → Business Requirement → WorkItem → Acceptance Criteria
→ Worktree → Agent Session → Context Packet → ChangeSet → Feedback
→ Validation Evidence → Commit → PR / MR → Acceptance
```

这条 Traceability Chain 是系统差异化核心（§79），也是第 105 章要求的《基本设计书》继承基础。

---

## 40. Product Success Criteria

除传统项目管理指标外（§80，数值目标一律 `TBD-MEASURE`，未获得真实测量数据前不得臆造）：

```text
更低 Feedback Iteration / 更低 AI Rework Rate / 更低 Context Waste
更低 Constraint Violation / 更低 Worktree Conflict
更高 First-pass Acceptance / 更高 Test Pass after Revision
更高 Feedback Resolution Rate / 更高 Requirement-to-Code Traceability
更高 Human-to-Agent Parallelism
```

### 40.1 Human-to-Agent Parallelism（§81）

目标不是无限增加并发 Agent 数量 N，而是"一个开发者能够在不过度增加认知负担的情况下，同时监督多个独立 Worktree"，须通过 Intervention Queue、Worktree Dashboard、Feedback Inbox、Conflict Alert、Agent Status、Validation Status 降低 Cognitive Load。

---

## 41. Requirement ID 一览与体系

### 41.1 ID 前缀体系（§62）

| 前缀 | 含义 |
|---|---|
| （原文档既有前缀，如 REQ-xxx） | 原有 Requirement（未在本仓库中提供，待与既有文档核对） |
| `DEV-xxx` | Development Execution Requirement |
| `WT-xxx` | Worktree Requirement |
| `AGT-xxx` | Agent Requirement |
| `FBK-xxx` | Feedback Requirement |
| `CTX-xxx` | Context Requirement |
| `VAL-xxx` | Validation Requirement |
| `SCM-xxx` | Source Control Integration Requirement |
| `LRT-xxx` | Local Runtime Requirement |
| `SEC-xxx` | 安全 / 隔离边界 Requirement（跨 Tenant/Repository/Worktree Leakage 防护，见第 16 章） |

### 41.2 关键 P0 Requirement 登记表（§63）

| ID | 内容 | 对应章节 | 对应 Architecture Obligation |
|---|---|---|---|
| WT-001 | 系统必须能够注册并跟踪与 WorkItem 关联的 Worktree | 第 22.1 章 | ARCH-OBL-DEV-001 |
| WT-002 | 系统必须能够区分 Worktree Server Metadata 与 Local Observed State | 第 22.1、23.3 章 | ARCH-OBL-DEV-006 |
| WT-003 | 系统必须能够查看多个 Worktree 的开发状态 | 第 22.3 章 | ARCH-OBL-DEV-001 |
| AGT-001 | 系统必须将 AgentSession 与 Worktree 关联 | 第 24.1 章 | ARCH-OBL-DEV-001 |
| AGT-002 | 系统不得允许 Agent 越过授权 Worktree 执行受保护修改 | 第 24.3-24.4 章 | ARCH-OBL-DEV-001 |
| FBK-001 | 用户必须能够向 WorkItem/File/Symbol/Diff/Test 等目标发送结构化 Feedback | 第 25.1 章 | ARCH-OBL-DEV-002 |
| FBK-002 | 系统必须能够追踪 Feedback 是否被 Agent 消费、应用和验证 | 第 25.3 章 | ARCH-OBL-DEV-002 |
| CTX-001 | 系统必须能够根据任务自动生成 Context Packet | 第 26.1 章 | ARCH-OBL-DEV-002 |
| CTX-002 | Context Packet 必须保留来源追踪信息 | 第 26.3 章 | ARCH-OBL-DEV-002 |
| VAL-001 | Agent 完成状态不能仅以 Agent 自我报告作为依据 | 第 27.3 章 | ARCH-OBL-DEV-005 |
| SCM-001 | GitHub / GitLab 必须通过统一 SCM Adapter 接入 | 第 19.1 章 | ARCH-OBL-DEV-003 |
| LRT-001 | Local Runtime 必须经过身份认证和设备授权 | 第 23.2 章 | ARCH-OBL-DEV-004 |
| LRT-002 | SaaS 不得获得任意本地 Shell 执行能力 | 第 23.2 章 | ARCH-OBL-DEV-004 |
| SEC-xxx | 必须防止 Cross-Tenant / Cross-Repository / Cross-Worktree Context Leakage | 第 16、28.3、34 章 | ARCH-OBL-DEV-001/002 |

本文档第 1-17 章新增的基础 Requirement（`REQ-TWP-xxx / REQ-WF-xxx / REQ-PLAN-xxx / REQ-COLLAB-xxx / REQ-PERM-xxx / REQ-AUTO-xxx / REQ-NOTIF-xxx / REQ-SEARCH-xxx / REQ-DATA-xxx / REQ-RT-xxx / REQ-SEC-xxx / REQ-AUDIT-xxx`）与 Vibe Coding 扩展 P0 Requirement 共同构成完整 ID 登记表，下游《基本设计书》须逐项继承。

---

## 42. 核心模型图（§96）

### A. Work Management Model

```text
Tenant
↓
Workspace
↓
Project
↓
WorkItem
```

### B. Development Execution Model

```text
WorkItem
↓
DevelopmentExecution
↓
Worktree
↓
AgentSession
↓
ChangeSet
↓
Validation
↓
Feedback
```

### C. SCM Model

```text
Repository
↓
Branch
↓
Commit
↓
PR / MR
```

### D. Local Runtime Model

```text
Control Plane
↕
Runtime
↓
Repository
↓
Worktree
↓
Agent
```

### E. Traceability Model

```text
Requirement
↓
WorkItem
↓
Worktree
↓
AgentSession
↓
Change
↓
Validation
↓
PR/MR
```

---

## 43. 系统事实优先级与冲突决策原则

### 43.1 事实来源区分（§97）

必须区分，不得混为一个 "giant status JSON"：

```text
Business Truth
Observed Runtime State
SCM Truth
AI Suggestion
Human Feedback
Validation Evidence
```

### 43.2 事实冲突优先级（§98）

发生冲突时按以下顺序处理：

```text
Business Requirement
    > Explicit Human Constraint
    > Security Policy
    > Acceptance Criteria
    > Approved Architecture Decision
    > Repository Reality
    > Validation Evidence
    > Current Worktree State
    > Agent Plan
    > Agent Suggestion
    > Historical AI Summary
```

**AI 不能因为自己的历史总结覆盖新的人工要求。**

### 43.3 最终冲突决策优先级（§104，覆盖并细化原有原则）

```text
Business Correctness
    > Tenant Isolation
    > Data Integrity
    > Security
    > Explicit Human Intent
    > Acceptance Correctness
    > Traceability
    > Availability
    > Maintainability
    > Developer Experience
    > AI Interaction Quality
    > Performance
    > Scalability
    > K8s Extensibility
    > Resource Efficiency
    > Microservices
    > Serverless
    > AI Autonomy
    > Technology Novelty
```

**AI Autonomy 永远不得凌驾于 Human Intent、Security、Data Integrity、Acceptance Criteria 之上。**

---

## 44. 架构原则总纲

### 44.1 最终架构原则（§99）

```text
WorkItem 管理 Intent。
Worktree 管理 Execution Isolation。
AgentSession 管理 AI Execution。
ChangeSet 管理代码变化。
Feedback 管理 Human Correction。
ContextPacket 管理 Agent Input。
ValidationEvidence 管理"是否真的完成"。
GitHub / GitLab 管理远端 SCM 事实。
PostgreSQL 管理平台业务事实。
Local Runtime 观察真实本地开发状态。
AI 可以修改授权 Worktree，但 AI 不得成为业务事实源。
AI Chat 是交互方式，不是系统架构中心。
Worktree Control Center 才是 Vibe Coding 的核心操作界面。
所有重要 AI Coding 行为必须能够从 Requirement 追踪到最终 Commit / PR / MR。
提高 AI Coding 品质的关键不是无限增加 Prompt，而是精准 Context + Structured Feedback + Validation。
```

### 44.2 Kubernetes Tax 纪律（§86-90，continuation of original architecture principles）

即使增加 Development / Worktree / Agent / Feedback / Context / Validation / SCM 等 Domain，它们首先仍然只是 **Domain Module**，禁止形成 `worktree-service / agent-service / feedback-service / context-service / validation-service / github-service / gitlab-service` 等七八个独立 Deployment。第一阶段继续在 `work-core` 内聚，只有出现 Scaling Boundary / Failure Boundary / Security Boundary / Runtime Boundary / Ownership Boundary 之后才拆分（§86）。

须重点观察（而非预设）以下候选是否率先形成真实拆分边界：Realtime、AI Heavy Processing、Repository Analysis、Runtime Connection、SCM Integration Worker（§87）。不得仅因名称不同就拆服务。

### 44.3 Development Context 与 Work Core 解耦原则（§85，重申）

```text
WorkItem ≠ Git Branch ≠ Worktree ≠ AgentSession
```

一个 WorkItem 可以关联 0/1/N 个 Repository 与 0/1/N 个 Worktree，保证系统仍可服务非开发项目、设计任务、运营任务、文档任务、普通项目管理。

---

## 45. 专项 Review 执行结论

按 §102 要求，Review 7-10 的发现已直接修正进入正文对应章节，不另设独立评审日志。执行摘要如下：

| Review | 检查重点 | 落实位置 |
|---|---|---|
| Review 7 — Vibe Coding Product Review | Worktree 一等公民、多 Agent 可观察性、Feedback Inbox、Feedback 精准绑定、AI Session 与 Worktree 强关联、"为什么这么改"可追溯、Intervention Queue | 第 22、24、25 章 |
| Review 8 — Context Engineering Review | 不依赖无限增长 Chat History、Context Provenance、Decision 独立管理、Feedback 进入下一次 Context、Token Budget、避免无关代码进入 Context | 第 26 章 |
| Review 9 — Agent Security Review | Agent 越权访问、跨 Worktree 修改、Secret 越权读取、Repository Prompt Injection、Local Runtime 是否形成 Remote Shell、AI Provider 数据边界 | 第 23.2、24.3-24.4、28.3-28.4、34 章 |
| Review 10 — Development Runtime Review | Runtime Offline、Worktree State Stale、Agent Crash、Git Rebase、Repository Move、Daemon Version 不一致、重新连接 Reconcile | 第 22.6、23.3-23.5 章 |

---

## 46. 决策表

### A. MVP Must Have — 见第 30.2 章
### B. V1 Should Have — 见第 30.3 章
### C. V2 Candidates — 见第 30.4 章
### D. Future Architecture — 见第 30.5 章
### E. Explicit Non-Goals — 见第 30.6 章
### F. Top 10 Product Decisions

| # | 决策 |
|---|---|
| 1 | Worktree 提升为一级领域对象，而非 Repository 附属字段 |
| 2 | Worktree Status 与 WorkItem Status 分离建模 |
| 3 | Feedback 结构化为一级领域对象，不做普通 Comment |
| 4 | Context Compiler 作为确定性/半确定性系统能力，独立于 LLM |
| 5 | AI Task 作为正式 WorkItem 类型，受 Acceptance Criteria 与 Agent Policy 约束 |
| 6 | Worktree Control Center 成为产品主入口之一，与 Board/Backlog/Sprint 并列 |
| 7 | Feedback Inbox + Intervention Queue 作为人机协作核心工作台 |
| 8 | Agent 完成判定必须经过 Validation Evidence，禁止自我报告 |
| 9 | 敏捷规划闭环精简为 Backlog → Sprint → Board(含 Gantt) → Burndown |
| 10 | 甘特图作为 Board 的排期视图变体接入，不建独立子系统 |

### G. Top 10 Architecture Decisions

| # | 决策 |
|---|---|
| 1 | 保持 Rust Modular Monolith，新增 Domain 以 crate 形式内聚于 work-core |
| 2 | Local Runtime 独立于 Kubernetes Application Workload 计数之外 |
| 3 | PostgreSQL 继续作为唯一 System of Record |
| 4 | Observed State 与 Business State 分离存储与治理 |
| 5 | SCM 通过统一 Adapter 接入，Domain 层禁止厂商对象 |
| 6 | Agent 通过统一 Agent Port/Adapter 接入，不绑定单一厂商 |
| 7 | Event Bus 仅用于外围解耦，不拆解核心业务事务 |
| 8 | 大型二进制/Transcript 数据评估 Object Storage 而非 PostgreSQL 热表 |
| 9 | Worktree Reconciliation 采用应用层同步，不引入 K8s-style Controller |
| 10 | Serverless/KEDA 仅在真实负载证明后引入 |

### H. Top 10 SaaS Risks — 见第 33 章 RISK-016~030 及原有 RISK-001~015（待核对）
### I. Top 10 Kubernetes Risks — 沿用原文档 Kubernetes Risk 登记表（未在本仓库中提供，待核对）
### J. Top 10 Open Issues

| # | Open Issue |
|---|---|
| 1 | 原《Kubernetes-native 工作管理 SaaS 要件定义》文档未能在本仓库定位，第 1-17、31-33、44.2 章部分内容为重新编写，需与原文档核对一致性 |
| 2 | Symbol-level Conflict Detection 的具体分析粒度与性能边界待 PoC 验证（POC-025） |
| 3 | Context Compiler 的 Token Budget 具体阈值待真实数据校准（TBD-MEASURE） |
| 4 | Local Runtime 与 SaaS Control Plane 之间的 Reconciliation 协议细节待 ADR-020 确定 |
| 5 | Agent Vendor 数量增长后 Agent Port 抽象是否足够，需在 V1 阶段复审 |

### K. Top 10 Vibe Coding Decisions

| # | 决策 |
|---|---|
| 1 | 系统同时承担 Jira-class / SCM Integration / AI Worktree Control Plane 三种职责，且不合并 Domain |
| 2 | Development Execution 作为 WorkItem 与代码环境之间的聚合层 |
| 3 | 1 WorkItem → N Worktree，1 Worktree → N AgentSession，1 AgentSession → 1 Active Worktree（默认） |
| 4 | ChangeSet 不等于 Git Diff，需承载 Files/Symbols/Risk Signals 等结构化信息 |
| 5 | Symbol-level Context 逐步演进，MVP 不强制完整 IDE Compiler Database |
| 6 | AI Completion 判定必须经过 Validation → Acceptance Coverage → Feedback Resolution → Human/Policy Gate |
| 7 | Human-in-the-loop 按动作分级授权（Analyze/Suggest 自动，Commit/Push/Merge 受策略或保护控制） |
| 8 | Multi-Agent 并行以 Visibility/Isolation/Feedback/Context/Validation 为 MVP 边界，不做 Agent Swarm |
| 9 | Agent Handoff 依赖结构化 Handoff Context Packet，而非全量聊天记录 |
| 10 | Human-to-Agent Parallelism 以降低认知负担为目标，而非最大化并发 Agent 数量 |

### L. Top 10 Worktree Risks

| # | 风险 |
|---|---|
| 1 | Worktree Conflict Explosion（RISK-028） |
| 2 | Stale Worktree State（RISK-022） |
| 3 | Agent Escapes Worktree Scope（RISK-017） |
| 4 | Worktree Isolation 失效导致跨 Worktree 数据污染（第 22.5 章） |
| 5 | Local Runtime Compromise 导致 Worktree 被篡改（RISK-016） |
| 6 | Git Rebase / Force Push 导致 Worktree 与远端分叉（第 23.5 章） |
| 7 | Agent Session State Divergence（RISK-023） |
| 8 | Worktree Reconciliation 缺失导致 Desired/Observed State 永久不一致 |
| 9 | Worktree Heatmap 数据滞后导致冲突预警失效 |
| 10 | Local Runtime Version Fragmentation 导致 Worktree 行为不一致（RISK-029） |

### M. Top 10 Agent Security Risks

| # | 风险 |
|---|---|
| 1 | Malicious Repository Prompt Injection（第 34 章） |
| 2 | Agent Credential Exfiltration（RISK-018） |
| 3 | Cross-Worktree Context Leakage（RISK-019） |
| 4 | Cross-Repository Context Leakage（RISK-020） |
| 5 | Cross-Tenant AI Context Leakage（第 34 章） |
| 6 | Agent Unauthorized Command Execution（第 34 章） |
| 7 | Compromised Local Runtime 形成事实上的 Remote Shell（第 34 章） |
| 8 | Fake Validation Result 被 Agent 伪造或误报（第 34 章） |
| 9 | Malicious GitHub/GitLab Webhook 触发未授权操作（第 34 章） |
| 10 | Agent Vendor Lock-in 导致安全策略无法统一执行（RISK-030） |

### N. Top 10 Context Engineering Decisions

| # | 决策 |
|---|---|
| 1 | Context Packet 目标为 Minimum Sufficient Context，而非 Maximum Context |
| 2 | Context Provenance 强制要求，禁止 "AI Memory Blob" |
| 3 | Decision Memory 独立于 Chat History 管理，支持 Supersede/Invalidate |
| 4 | Context Priority 分级（P0 Explicit Human Constraint 最高） |
| 5 | Context Compiler 优先使用 Active Decision 而非完整聊天历史 |
| 6 | Handoff Context Packet 替代全量聊天记录传递给下一个 Agent |
| 7 | Symbol-level Context 逐步演进，不一次性引入 Graph Database |
| 8 | Context Cost 纳入 Planning Assistance 指标 |
| 9 | Context Efficiency 观测 Relevant Context Ratio / Repeated Context Ratio 而非单纯扩大 Context Window |
| 10 | 敏感 Context（Prompt/Code）遵循 AI Content Retention Policy，不默认进入普通日志 |

### O. Top 10 Human Feedback Design Decisions

| # | 决策 |
|---|---|
| 1 | Feedback Target 覆盖 WorkItem 到 Diff Hunk 的全粒度对象 |
| 2 | Feedback 必须包含 Expected/Preserve/Prohibit 结构化字段 |
| 3 | Feedback Type 覆盖 Fix/Preserve/Refactor/Reject/Question/Constraint 等语义 |
| 4 | Feedback 状态机（OPEN→ACKNOWLEDGED→APPLIED→VERIFIED/REJECTED/SUPERSEDED）强制追踪消费情况 |
| 5 | 从结构化 Feedback 生成高密度、低歧义、低 Token 的 Agent Instruction |
| 6 | Feedback Inbox 聚合多来源待处理项，避免用户逐个进入 Agent Chat |
| 7 | Intervention Queue 按 P0-P3 优先级呈现需要人工介入的事项 |
| 8 | Chat 中的重要规则可提升为 Structured Feedback / Decision，防止淹没在聊天历史中 |
| 9 | Feedback Efficiency Metric（Precision/Fix Ratio/Repetition/Rejection/Reopen）纳入产品度量 |
| 10 | Feedback Resolution Rate 作为 Product Success Criteria 之一 |

---

## 47. 下一阶段输入清单（《基本设计书》阶段建议输入）

本要件定义书完成后停止，**不进入生产代码编写，不将要求偷换为技术实现**（§105）。下一阶段《基本设计书（基本設計書）》必须继承以下产出：

```text
Requirement ID（第 41 章登记表，含 REQ-xxx / DEV-xxx / WT-xxx / AGT-xxx / FBK-xxx / CTX-xxx / VAL-xxx / SCM-xxx / LRT-xxx / SEC-xxx）
Architecture Obligation（第 35 章 ARCH-OBL-DEV-001~006，及原有 ARCH-OBL 登记表）
ADR Candidate（第 32 章）
PoC Result（第 31 章，需在基本设计前实际执行并记录结果）
Risk（第 33 章）
Open Issue（第 46 章 决策表 J）
Security Boundary（第 16、23.2、34 章）
Domain Boundary（第 6 章）
Worktree Lifecycle（第 22.2 章）
Agent Policy（第 24.3 章）
Feedback Model（第 25 章）
Context Model（第 26 章）
Validation Model（第 27 章）
SCM Integration Contract（第 18-19 章）
```

《基本设计书》阶段建议输入清单还应包括：Persona 与 Use Case 清单（第 3、36 章）、Acceptance Criteria 示例集（第 37 章）、Traceability Model（第 39 章）、决策表 A-O（第 46 章）、以及本文档第 0 章列出的与原文档待核对项。

---

*文档结束。本文档为要件定义阶段产出，后续团队据此继续制作基本設計 / 外部設計 / 内部設計 / API Design / Data Design / Security Design / Runtime Design / Integration Design / AI・Agent Design / Test Design / Operation Design。*
