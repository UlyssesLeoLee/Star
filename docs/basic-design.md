# Star 平台《基本设计書》

> **文档版本**: v0.1 (2026-08-25)
> **上游要件定义书**: `D:\Star\docs\requirements.md` v2.0(下文以 §N 引用)
> **文档定位**: 基本设计書(架构视图 / Module 划分 / 数据所有权 / 状态机 / 接口契约 / 安全边界 / 部署拓扑 / ADR 草案)

---

## 0. 文档说明

### 0.1 文档目的与定位

本文档为 Star 平台(AI Coding Worktree Control Plane + Jira-class Work Management + SCM Integration)《基本设计書》阶段的产出。其上游是《要件定義書 v2.0》(§0-§47),下游将依次进入《外部設計》《内部設計》《API Design》《Data Design》《Security Design》《Runtime Design》《Integration Design》《AI/Agent Design》《Test Design》《Operation Design》等详细设计阶段。

**本文档不输出生产代码**(重申 §47):

- ❌ 不写 SQL DDL
- ❌ 不写 SQLx / Diesel 完整 Repository 实现
- ❌ 不写完整 Rust handler / use 语句块 / 业务函数体
- ❌ 不写前端组件代码
- ❌ 不画物理网络拓扑(用 mermaid block 即可)
- ❌ 不重新评估 §13 列出的既有架构原则

**本文档可输出**:

- ✅ 架构视图(mermaid)
- ✅ Module 划分 / 职责 / 不变量 / 依赖方向
- ✅ 数据所有权矩阵 / SoR 划分 / Event Subject 草案
- ✅ 状态机迁移表
- ✅ 接口契约签名(method 名 + 入参类型 + 返回类型)
- ✅ 事件 Schema(Subject + 字段 + 类型)
- ✅ ADR 草案(Proposed 状态)
- ✅ Risk / PoC / 决策继承表

### 0.2 与第 47 章《下一阶段输入清单》的对应关系

本文档严格继承 §47 列出的全部输入项,具体落位:

| §47 输入项 | 本文落位 |
|---|---|
| Requirement ID(§41) | §14 决策继承表、§4 各 Module 的 Requirement 索引段 |
| Architecture Obligation(§35) | §4 各 Module 的 ARCH-OBL-DEV-xxx 引用、§6 安全边界 |
| ADR Candidate(§32) | §10 ADR-016~030 |
| PoC(§31) | §11 POC-016~030 |
| Risk(§33) | §12 RISK-016~030 |
| Open Issue(§46 决策表 J) | §15 |
| Security Boundary(§16, §23.2, §34) | §6 安全边界、§4.6 Local Runtime |
| Domain Boundary(§6) | §2 Domain 划分、§3 Context Map |
| Worktree Lifecycle(§22.2) | §4.1、§7、附录 A |
| Agent Policy(§24.3) | §4.2、§6.4 |
| Feedback Model(§25) | §4.3 |
| Context Model(§26) | §4.4 |
| Validation Model(§27) | §4.5 |
| SCM Integration Contract(§18-19) | §4.7、§4.8 |
| Persona & Use Case(§3, §36) | §2 职责说明、§4 关键流程 |
| Acceptance Criteria 示例(§37) | §4 各 Module 的 AC 引用段 |
| Traceability Model(§39) | §9 |
| 决策表 A-O(§46) | §14 |
| 与原文档待核对项(§0) | §15 Open Issue |

### 0.3 命名约定

- **Module / Domain**: 同义,代表 crate 级别的逻辑划分(非 deployment)
- **Aggregate**: 聚合根,Transaction 边界
- **Projection**: 派生视图,不可作为业务事实源
- **Observed State**: 高频、非业务事实的运行时状态(§14.1)
- **SoR**: System of Record,本设计中默认为 PostgreSQL
- **ACL**: Anti-Corruption Layer
- **P0/P1/P2**: 优先级(继承 §41.2)

### 0.4 受众

- 详细设计阶段工程师(API / Data / Security / Runtime / Integration / AI / Test / Operation)
- 架构审查者(§35 ARCH-OBL 履行情况)
- SRE / Platform 团队(K3s 部署、Service Promotion、Worker 拆分)
- 安全 / 合规(§6 §16 §23.2 §28 §34 履行情况)

---

## 1. 架构总览

### 1.1 物理架构图(SaaS Control Plane + Local Runtime + External SCM)

```mermaid
flowchart TB
    subgraph Internet[外部网络]
        GH[GitHub]
        GL[GitLab]
        FutureSCM[Gitea / Bitbucket / Future SCM]
        DevMachine[Developer Machine / Self-hosted Runner]
    end

    subgraph K3sCluster[K3s Cluster]
        GW[Gateway / Ingress]
        ID[Identity Service]
        WC[work-core / Rust Modular Monolith]
        W[Worker --role all]
        NATS[(NATS JetStream)]
        PG[(PostgreSQL SoR)]
        VALK[(Valkey Cache)]
        RT{Realtime (Optional)}
    end

    subgraph LocalRuntime[Local Runtime / Daemon]
        LR[Local Daemon - Rust]
        WTA[Worktree A]
        WTB[Worktree B]
        WTC[Worktree C]
        AGTA[Agent A]
        AGTB[Agent B]
        AGTC[Agent C]
    end

    DevMachine -->|Secure Channel / mTLS| GW
    GH -->|Webhook / API| GW
    GL -->|Webhook / API| GW
    FutureSCM -.->|Future| GW
    GW --> ID
    GW --> WC
    GW -.-> RT
    WC <--> PG
    WC <--> VALK
    WC --> NATS
    W --> NATS
    W --> PG
    LR -->|HTTPS / WSS| GW
    LR --> WTA
    LR --> WTB
    LR --> WTC
    WTA --> AGTA
    WTB --> AGTB
    WTC --> AGTC
    GH <-->|Repository Sync| LR
    GL <-->|Repository Sync| LR
```

**继承自 §13.1、§13.2、§23.1**。关键设计要点:

1. **服务器端最小闭环保持不变**:`gateway / identity / work-core / worker` 四个角色,加上 PostgreSQL / NATS / Valkey 三个数据面。`realtime` 角色仅在出现真实 Long Connection Scaling Boundary 时才拆出(§13.1,§15)。
2. **Local Runtime 不计入 K8s Workload**:Developer Machine 与 K3s Cluster 是平级关系,通过 Secure Channel 对接,而非 In-Cluster Pod(§23.1)。
3. **External SCM 是事实源,不是镜像**:GitHub / GitLab 通过 Adapter 接入,平台不重新制造 Git(§19.2,§30.6)。

### 1.2 逻辑架构图(Rust Modular Monolith crates 布局)

```mermaid
flowchart LR
    subgraph api[crates/api]
        APIGW[HTTP Gateway]
        WS[WebSocket Gateway]
    end

    subgraph application[crates/application]
        APP[Application Services]
        APPPORT[Ports / Inbound]
    end

    subgraph domain[crates/domain-*]
        D_T[domain-tenant]
        D_WS[domain-workspace]
        D_PJ[domain-project]
        D_WI[domain-work-item]
        D_WF[domain-workflow]
        D_BO[domain-board]
        D_PL[domain-planning]
        D_PE[domain-permission]
        D_CO[domain-comment]
        D_RL[domain-relation]
        D_DX[domain-development]
        D_WT[domain-worktree]
        D_AG[domain-agent]
        D_FB[domain-feedback]
        D_CT[domain-context]
        D_VL[domain-validation]
        D_SC[domain-scm]
        D_ID[domain-identity]
        D_AT[domain-audit]
        D_SR[domain-search]
        D_NT[domain-notification]
        D_IN[domain-integration]
        D_AU[domain-automation]
        D_LR[domain-local-runtime]
    end

    subgraph infra[crates/infrastructure]
        INFRA_PG[PostgreSQL Adapter]
        INFRA_NATS[NATS Adapter]
        INFRA_VALK[Valkey Adapter]
        INFRA_OBJ[Object Storage Adapter]
        INFRA_SCM[SCM Adapter]
        INFRA_AGT[Agent Adapter]
    end

    api --> application
    application --> domain
    domain --> infra
    domain -.->|Domain Events| INFRA_NATS
    D_WI --> D_WF
    D_WI --> D_BO
    D_WI --> D_PL
    D_WI --> D_RL
    D_WI --> D_CO
    D_WI --> D_DX
    D_DX --> D_WT
    D_DX --> D_AG
    D_DX --> D_FB
    D_DX --> D_CT
    D_DX --> D_VL
    D_DX --> D_SC
```

**继承自 §13.3**。关键约束(§44.2):

- 19 个 `domain-*` crate ≠ 19 个 service ≠ 19 个 deployment
- Domain 之间只允许 **由内向外** 的依赖(D_WI → D_WF, D_DX → D_WT, 不允许反向)
- `application` crate 负责编排多个 Domain,所有跨域事务落在此处
- `infrastructure` crate 不允许反向依赖 `domain`,只实现 Domain 定义的 Port(§3 ACL)

### 1.3 Worker 拓扑(§13.4)

第一阶段:`worker --role all`,九种角色在同一二进制内通过 tokio::select 多路复用:

| 角色 | 职责 | 第一阶段合并 |
|---|---|---|
| notification | 邮件 / 站内通知发送 | ✅ |
| webhook | GitHub / GitLab Webhook 接收 | ✅ |
| automation | 自动化规则触发器执行 | ✅ |
| projection | Search / 报表 / Heatmap 投影 | ✅ |
| integration | 第三方平台双向同步 | ✅ |
| maintenance | 过期会话清理 / 归档 | ✅ |
| scm-sync | Repository / Branch / Commit 增量同步 | ✅ |
| context-build | Context Packet 构建(可拆分至 V1) | ✅ |
| repository-analysis | Symbol / Dependency / Risk Signal(可拆分至 V1) | ✅ |

**拆分触发条件**(§44.2):

- 真实 CPU 压力 > 70% 持续 5 分钟
- 任一角色出现独立 Scaling 需求(如 scm-sync 受 GitHub Rate Limit 制约)
- 任一角色出现独立 Failure Boundary(如 repository-analysis OOM)
- Security Boundary(如 Local Runtime 相关)

### 1.4 KEDA / Serverless 候选评估(§13.5)

| 候选任务 | Scale-to-Zero 价值 | 引入时机 |
|---|---|---|
| Repository Analysis | 高(分析 10k+ Stars Repo) | V1 评估(§30.3) |
| Large Context Build | 中(>200K Token 罕见) | V1 评估 |
| PR Analysis | 中(批量 PR 不可预测) | V1 评估 |
| Static Analysis | 高(批量触发) | V2(§30.4) |
| Agent Session Post-processing | 中 | V2 |
| Diff Summarization | 中 | V2 |
| Dependency Scan | 高(夜间) | V2 |

**判定原则**:不因 Vibe Coding 提前引入,必须先有 Resource Saving vs Operational Complexity 的明确对比(§13.5,§89)。

### 1.5 关键不变量:K8s Tax 纪律(§44.2,§86-90)

> 严禁因增加 Development Domain 就拆出 `worktree-service / agent-service / feedback-service / context-service / validation-service / github-service / gitlab-service` 等七八个独立 Deployment。

**遵守方式**:

1. 第一阶段所有 Development Domain 作为 crate 内聚于 `work-core`
2. Worker 第一阶段合并为 `worker --role all`
3. Realtime 仅在出现 Long Connection Scaling Boundary 后才拆
4. 数据库保持单一 PostgreSQL(非 Database per Domain,§30.6)
5. Event Bus 不拆解核心业务事务(§14.1,§58)

**违反的早期信号**:

- 任一 Module 出现独立 Pod > 3 个
- 跨 Module 通信 80% 走 HTTP 而非 in-process call
- 任一 Module 出现独立 Database

---

## 2. Domain / Module 划分

### 2.1 完整 Domain 列表(继承 §6 共 22 个 + 3 个拆分/合并 = 25 个逻辑 Module)

> §6 列出 22 个 Domain(Identity, Tenant, Workspace, Project, Work Management, Workflow, Planning, Collaboration, Permission, Automation, Integration, SCM, Development Context, Development Execution, Worktree, Agent, Feedback, Context, Validation, Audit, Search, Notification)。本设计书对其中 3 个作拆分/合并,并新增 1 个服务器侧 Runtime 管理面,共得到 25 个 crate 级 Module:1) `Collaboration` 拆为 `domain-comment` + `domain-collaboration`;2) `Development Context` 合并入 `domain-development`(主要实体补 `SymbolIndex`, `RepositoryContext`, `DevelopmentContext`);3) 新增 `domain-local-runtime`,对应 §23 Local Runtime 的服务器侧 Runtime Registry / Port(注意:Local Daemon 二进制进程本身**不**属此 crate,见 §4.6.1 区分)。所有 Module 均为 `crates/domain-*` 或内嵌于 `crates/application` 的 Submodule。

#### 2.1.1 核心域(Core Domain)

| # | Module | 一句话职责 | 主要实体 | 关键不变量 | 关键依赖 |
|---|---|---|---|---|---|
| 1 | domain-work-item | WorkItem 的创建 / 状态流转 / 关系 | WorkItem, Requirement, AcceptanceCriterion | WorkItem ≠ Git Branch(§44.3);1 WorkItem → 0/1/N Repository | domain-workflow, domain-project, domain-permission |
| 2 | domain-worktree | Worktree 一级领域对象,生命周期管理 | Worktree, ConflictState, HealthState | Worktree Status 独立于 WorkItem Status(§22.2,REQ-WF-002) | domain-work-item, domain-scm, domain-development |
| 3 | domain-agent | Agent Adapter 与 AgentSession 生命周期 | Agent, AgentSession, AgentPolicy | 1 AgentSession → 1 Active Worktree(§21,REQ-DEV-003) | domain-worktree, domain-feedback, domain-validation |
| 4 | domain-feedback | 结构化 Feedback 一级领域对象 | Feedback, FeedbackResolution | Feedback Target 覆盖 WorkItem→Diff Hunk 全粒度(§25.1) | domain-work-item, domain-worktree, domain-agent |
| 5 | domain-context | Context Packet 生成与 Decision Memory | ContextPacket, Decision | Context Provenance 强制可追溯(§26.3) | domain-work-item, domain-worktree, domain-feedback, domain-validation |
| 6 | domain-validation | Validation Evidence 与 Acceptance Coverage | ValidationResult, AcceptanceCoverage | AI 自我报告不构成完成(§27.3,VAL-001) | domain-work-item, domain-worktree, domain-agent |

#### 2.1.2 支撑域(Supporting Domain)

| # | Module | 一句话职责 | 主要实体 | 关键不变量 | 关键依赖 |
|---|---|---|---|---|---|
| 7 | domain-scm | SCM Adapter 抽象与 Repository 同步 | Repository, Branch, Commit, PullRequest, Review, Pipeline | Domain 层无厂商对象(§19.1,REQ-SCM-002) | domain-work-item, domain-worktree |
| 8 | domain-development | Development Execution 聚合层 + Repository Indexing(§20 合并入) | DevelopmentExecution, ChangeSet, Link, SymbolIndex, RepositoryContext, DevelopmentContext | ChangeSet ≠ Git Diff(§21.1);Symbol-aware Context 逐步演进(§21.2) | domain-work-item, domain-worktree, domain-agent, domain-scm |
| 9 | domain-workflow | Workflow 定义与状态机 | WorkflowDefinition, State, Transition | Worktree Status 与 WorkItem Status 独立(REQ-WF-002) | domain-work-item |
| 10 | domain-board | Kanban / Scrum 板视图 | Board, Column, Swimlane | 与 Sprint / Gantt 共享数据模型(§9,REQ-PLAN-003) | domain-work-item, domain-planning |
| 11 | domain-planning | Sprint / Backlog / Roadmap | Sprint, Backlog, Roadmap | Burndown 最小必需,Velocity/CFD 控制图 V1(§9) | domain-work-item, domain-board |
| 12 | domain-relation | WorkItem 关系(阻塞/关联) | Relation, Dependency | 是甘特图依赖与冲突分析基础(REQ-COLLAB-002) | domain-work-item |
| 13 | domain-comment | 评论 / @ 提及 / 附件 | Comment, Mention, Attachment | 不替代 Feedback(§25.1) | domain-work-item |
| 14 | domain-search | 全文 / 符号检索 Projection | SearchIndex, SearchQuery | 不得成为业务事实源(§12,REQ-SEARCH-001) | 所有 domain-*(只读) |
| 15 | domain-audit | 审计日志 / AI Audit Metadata | AuditEvent, AIAuditMetadata | 敏感 Prompt/Code 不默认进入普通日志(§17,§28.2) | 所有 domain-*(Append-only) |
| 16 | domain-integration | 第三方平台双向同步抽象 | Integration, SyncState | 区分 Link/Mirror/Bidirectional/Platform-owned(§18.1) | domain-scm, domain-work-item |
| 17 | domain-automation | 触发器-条件-动作规则 | Rule, Trigger, Action | MVP 不强制可视化配置器(§11,REQ-AUTO-001);Trigger 支持 Event 与 Schedule/Cron 两类,互不共用执行路径(REQ-AUTO-002,V1 候选) | domain-work-item, domain-notification |

#### 2.1.3 通用域(Generic Domain)

| # | Module | 一句话职责 | 主要实体 | 关键不变量 | 关键依赖 |
|---|---|---|---|---|---|
| 18 | domain-tenant | Tenant 最高安全边界 | Tenant, TenantPolicy | 任何聚合根必带 tenant_id(§16,REQ-SEC-001) | 无 |
| 19 | domain-workspace | Workspace 协作单位 | Workspace | Workspace → 多个 Project(§7) | domain-tenant |
| 20 | domain-project | Project 模板与配置 | Project, ProjectTemplate, ProjectPolicy | 可独立配置 Workflow/Permission/Notification/Agent Policy(REQ-TWP-003) | domain-tenant, domain-workspace |
| 21 | domain-permission | Permission Scheme 与 RBAC | Role, Permission, PermissionScheme | Agent 操作必须 Application/Authorization 强制(§11,REQ-PERM-002) | domain-tenant |
| 22 | domain-identity | 用户 / 设备身份 | User, Device, Credential, DeviceBinding | Device 需 Tenant+User+Project 三重绑定(§23.2) | domain-tenant |
| 23 | domain-notification | 通知渠道与模板 | NotificationChannel, NotificationTemplate | MVP 邮件 + 站内(REQ-NOTIF-001);默认仅在需要人类决策的节点触达,不对 Agent 中间步骤逐条通知(REQ-NOTIF-002) | domain-tenant |
| 24 | domain-collaboration | 协作(实时状态、Presence) | Presence, RealtimeSubscription | 高频 Token Stream 可不入 SaaS(§15,REQ-RT-003) | domain-work-item, domain-worktree |
| 25 | domain-local-runtime | 集群外 Local Runtime 的服务器侧 Registry / Port | Runtime, RuntimeCommand, RuntimeObservation | Local Daemon 二进制不属此 crate(§4.6.1,§23.1) | domain-worktree, domain-identity |

### 2.2 Domain 分层结论

- **Core Domain**(高业务复杂度 + 高差异化):work-item, worktree, agent, feedback, context, validation
- **Supporting Domain**(必要支撑):scm, development, workflow, board, planning, relation, comment, search, audit, integration, automation
- **Generic Domain**(通用基础):tenant, workspace, project, permission, identity, notification, collaboration

> 注:§6 的 22 个 Domain 在本设计中的拆分/合并如下(详见 §2.1 标题段):
> 
> 1. `Collaboration` 拆为 `domain-comment` + `domain-collaboration`(Realtime Presence),因为前者是 WorkItem 内嵌聚合,后者是横切能力。
> 2. `Development Context`(§20)合并入 `domain-development`,因为 Development Context 的核心实体(`SymbolIndex` / `RepositoryContext` / `DevelopmentContext`)与 Development Execution 在同一聚合内,拆分会导致跨聚合的 Symbol-level Feedback 路由复杂化(`domain-context` 仅承担 §26 Context Compiler,职责严格区分)。
> 3. 新增 `domain-local-runtime`,对应 §23 Local Runtime 的服务器侧 Runtime Registry / Port(注意:Local Daemon 二进制进程本身不属此 crate,见 §4.6.1)。

> **2026-08-26 Requirement 同步**(参考竞品 Multica 分析,详见《requirements.md》第 11/12/19/24 章):本设计书已同步以下变更,均为 V1/V2/Future 候选,不改变 MVP 边界与既有 Domain 划分:
>
> - REQ-AUTO-002:`domain-automation` 的 `Trigger` 增加 Schedule/Cron 变体(未进入本章 10 个深度设计 Module,先在本表与 §5.6 事件清单中登记)。
> - REQ-NOTIF-002:`domain-notification` 默认仅在人类决策节点触达,详见上表。
> - REQ-SCM-003:`domain-scm` 的 Adapter 扩展优先级调整,自建 Git(Gitea/Forgejo)排在 Bitbucket/Azure DevOps 之前,见 §4.7.1。
> - AgentSession 新增 `token_usage` / `cost_summary` 字段,见 §4.2.2。
> - `domain-agent` 新增 Skill/Playbook 与 Squad 分组视图(§4.2.8)两个未来扩展方向,均不改变 §24.5/INV-AGT-10 的 Multi-Agent Control 边界。

### 2.3 Domain 间调用方向(硬约束)

**绝对禁止反向依赖**。允许的调用方向:

```text
domain-tenant ← domain-workspace ← domain-project ← domain-work-item ← domain-workflow
                                                                    ↘ domain-board
                                                                     ↘ domain-planning
                                                                     ↘ domain-relation
                                                                     ↘ domain-comment
                                                                     ↘ domain-development ← domain-scm
                                                                                        ↘ domain-worktree
                                                                                        ↘ domain-agent
                                                                                        ↘ domain-feedback
                                                                                        ↘ domain-context
                                                                                        ↘ domain-validation
domain-permission(被所有 domain 依赖)
domain-audit(被所有 domain 依赖,只追加,不可读)
domain-search(被所有 domain 写,读侧仅 api 可见)
domain-identity ← domain-permission
domain-automation ← domain-work-item
domain-notification ← 任意 domain(发布事件)
domain-integration ← domain-scm
domain-collaboration ← domain-work-item, domain-worktree
domain-local-runtime ← domain-worktree(接收 Runtime Observation,§23.3)
domain-local-runtime ← domain-identity(device_identity,§23.2)
```

**禁线**:

- ❌ domain-worktree → domain-work-item(状态独立,不允许反向写)
- ❌ domain-scm → domain-worktree(SCM 是支撑,不依赖 Worktree 状态)
- ❌ domain-context → domain-agent(Context 是 Agent 输入,不依赖 Agent 内部)
- ❌ domain-feedback → domain-context(Feedback 是 Context 的输入源之一,不是反过来)
- ❌ domain-audit 读其他 domain(只追加,不可读)

### 2.4 跨域事务(Transaction Boundary)

跨域事务由 `crates/application` 中的 Application Service 编排,**不通过 Event Chain 拆分**(§14.1,§58)。

**典型跨域事务示例**:

| 事务 | 涉及 Domain | 事务边界 |
|---|---|---|
| 创建 WorkItem | work-item, workflow, project, permission, audit | 单 PG 事务 |
| 注册 Worktree | worktree, work-item, scm, development, audit | 单 PG 事务 |
| 启动 AgentSession | agent, worktree, context, audit | 单 PG 事务 + Outbox |
| 提交 Feedback | feedback, work-item, audit | 单 PG 事务 |
| 创建 Commit Link | development, scm, worktree, validation, audit | 单 PG 事务 |
| 完成 WorkItem | work-item, validation, feedback, workflow, audit | 单 PG 事务 |
| 注册 Runtime | local-runtime, identity, worktree, audit | 单 PG 事务 + Outbox(发 Runtime Registered 给 worker) |

**Outbox 触发的事件**(非事务组成,异步):

- AgentSessionCreated → 通知 worker 启动 context-build
- WorktreeStatusObserved → 通知 worker 更新 projection / heatmap
- ValidationFailed → 通知 notification / 触发 Intervention Queue

---

## 3. Context Map(Domain 间解耦)

### 3.1 解耦机制总览(继承 §14.1,§18.1,§22.4,§24.2)

| 机制 | 适用场景 | 示例 |
|---|---|---|
| **Domain Event**(NATS JetStream) | 异步通知,无强一致需求 | AgentSessionStarted, WorktreeDirtyStateChanged |
| **ACL(Anti-Corruption Layer)** | 外部系统适配,防止厂商对象污染 | SCM Adapter(GitHub↔Domain), Agent Adapter(Codex↔Domain) |
| **Shared Kernel** | 跨域通用概念,放在最低层 | TenantId, UserId, TimeRange(Currency-like Value Object) |
| **Customer-Supplier** | 上游定义契约,下游实现 | SCM(S) → Development(C);Agent(S) → Worktree(C) |
| **Conformist** | 下游完全接受上游模型,无翻译 | Local Runtime 上报 Observed State,Control Plane 直接接受 |
| **Open Host Service(OHS)** | 平台对外提供稳定 HTTP/WS API | /api/v1/* Gateway |
| **Published Language** | 跨域事件 / API 的标准化格式 | CloudEvents 1.0, JSON Schema for Domain Events |
| **Separate Ways** | 完全独立,可独立演进 | Notification 与 Audit 互不依赖 |

### 3.2 Domain 对之间的接触点(Context Map 详表)

> "接触点" = 这两个 Domain 之间具体通过什么交互。不列出所有 24×24 对,只列真实存在的接触。

#### 3.2.1 work-item → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| workflow | Customer-Supplier | WorkItem.workflow_id → WorkflowDefinition(由 workflow 提供) |
| board | Customer-Supplier | BoardConfiguration.project_id → WorkItem.project_id |
| planning | Customer-Supplier | Sprint.contains_work_item_ids(只读) |
| relation | Conformist | WorkItem 接受 relation 写入 |
| comment | Customer-Supplier | Comment.parent = WorkItem |
| development | Customer-Supplier | WorkItem 1 → N DevelopmentExecution(由 development 创建) |
| audit | Separate Ways(Append-only) | domain-audit 订阅 WorkItem Domain Event |
| permission | Shared Kernel | WorkItem.project_id 受 PermissionScheme 约束 |
| search | Published Language | WorkItem 投影到 Search Index(由 worker projection role) |
| collaboration | Customer-Supplier | WorkItem 状态变化触发 Realtime 推送 |

#### 3.2.2 worktree → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| work-item | Customer-Supplier | Worktree.work_item_id 引用(只读 FK) |
| scm | Customer-Supplier | Worktree 通过 SCM Adapter 创建(由 scm 提供 Port) |
| agent | Conformist | Worktree 接受 AgentSession 分配(由 agent 创建) |
| development | Customer-Supplier | Worktree.development_execution_id(由 development 提供) |
| context | Separate Ways(读取) | Context Compiler 读取 Worktree.current_change_set_id |
| validation | Separate Ways(读取) | Validation 读取 Worktree.test_state |
| audit | Separate Ways(Append-only) | 订阅 Worktree Domain Event |
| collaboration | Customer-Supplier | Worktree Status 触发 Realtime 推送 |

#### 3.2.3 agent → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| worktree | Conformist | AgentSession.worktree_id 引用 |
| feedback | Customer-Supplier | AgentSession.feedback_consumed[] 由 feedback 提供 |
| context | Customer-Supplier | AgentSession.context_packet_id 由 context 提供 |
| validation | Customer-Supplier | AgentSession.validation_result_ids[] 由 validation 提供 |
| development | Customer-Supplier | AgentSession.development_execution_id |
| audit | Separate Ways(Append-only) | 订阅 AgentSession Domain Event |

#### 3.2.4 context → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| work-item | Customer-Supplier | 读取 Requirement/AcceptanceCriterion |
| worktree | Customer-Supplier | 读取 Worktree.current_change_set, test_state |
| feedback | Customer-Supplier | 读取 Open Feedback |
| validation | Customer-Supplier | 读取 Failed Validation |
| scm | Conformist | 通过 SCM Adapter 读取 Repository 元数据(只读) |
| identity | Customer-Supplier | 读取 AgentPolicy 决策 |

#### 3.2.5 feedback → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| work-item | Customer-Supplier | Feedback.target = WorkItem |
| worktree | Customer-Supplier | Feedback.target = Worktree |
| agent | Customer-Supplier | Feedback.target = AgentSession |
| context | Separate Ways(发布) | 发布 FeedbackCreated Domain Event(由 context 订阅) |
| validation | Separate Ways(发布) | 发布 FeedbackVerified Domain Event |
| audit | Separate Ways(Append-only) | 订阅 Feedback Domain Event |

#### 3.2.6 validation → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| work-item | Customer-Supplier | 写 AcceptanceCoverage |
| worktree | Customer-Supplier | 写 Worktree.test_state |
| agent | Customer-Supplier | 写 AgentSession.validation_result_ids |
| audit | Separate Ways(Append-only) | 订阅 Validation Domain Event |

#### 3.2.7 scm → 多个

| 目标 Domain | 接触方式 | 接触点 |
|---|---|---|
| work-item | ACL(下游) | WorkItem 通过 scm 提供的 Link 关联 Commit/PR |
| worktree | ACL(下游) | Worktree 通过 scm 创建 Git Worktree |
| development | ACL(下游) | DevelopmentExecution 引用 scm 提供的 Repository/Branch |
| integration | Separate Ways | integration 是 scm 的子域,共 Port |

#### 3.2.8 identity / permission / audit / search / notification 横切

| 来源 Domain | 去向 Domain | 接触方式 | 接触点 |
|---|---|---|---|
| identity | 所有 | Shared Kernel | UserId, DeviceId 作为 Value Object |
| permission | 所有 | Customer-Supplier | PermissionChecker Port(由 application 调用) |
| audit | 所有 | Separate Ways(Append-only) | AuditRecorder Port(由 application 调用) |
| search | 所有 | Conformist(读) | SearchQuery Port(只读) |
| notification | 所有 | Separate Ways(发布) | NotificationDispatcher Port(由 application 调用) |

#### 3.2.9 补充 14 Domain 接触面 (v0.16 模块间协作细化新增)

per requirements §6 Domain Boundary 22 logical domain 列表,§3.2.1-§3.2.8 仅覆盖 11 domain,本节补 14 domain 核心接触面 (tenant/workspace/project/workflow/board/planning/comment/relation/collaboration/automation/integration/development/search(单独)/notification(单独)/local-runtime,扣除 §3.2.8 综述的 5 个 = 14)。完整 22 domain × N target 表如下,核心 1-3 接触面为主,非穷举。

| 源 Domain | 目标 Domain | 接触方式 | 接触点 |
|---|---|---|---|
| **tenant** | identity | Customer-Supplier | TenantMembership / TenantPolicy 校验 (per requirements §16) |
| **tenant** | workspace | Customer-Supplier | Workspace.tenant_id 引用 (FK) |
| **tenant** | project | Customer-Supplier | Project.tenant_id 引用 (FK) |
| **tenant** | audit | Separate Ways | Tenant 创建 / SecurityPolicy 替换事件全量审计 (LRT-001) |
| **workspace** | project | Customer-Supplier | Project.workspace_id + WorkspacePermissionScheme 派生 |
| **workspace** | permission | Customer-Supplier | Workspace 级 Permission Scheme (per requirements §11) |
| **project** | work-item | Customer-Supplier | WorkItem.project_id + ProjectPolicy (Workflow 扩展状态机源) |
| **project** | workflow | Customer-Supplier | Project.workflow_definition_id 引用 |
| **project** | board | Customer-Supplier | Project.board_configuration_id 引用 |
| **project** | planning | Customer-Supplier | Project.sprint_scheme_id 引用 |
| **project** | automation | Customer-Supplier | Project.automation_rules[] 派生 |
| **project** | notification | Customer-Supplier | Project.notification_scheme_id 引用 |
| **workflow** | work-item | Customer-Supplier | WorkflowDefinition → state machine (per REQ-WF-001) |
| **workflow** | permission | Customer-Supplier | Transition Guard (RequireRole/RequireValidation/RequireApproval, per REQ-WF-003) |
| **board** | work-item | Customer-Supplier | BoardConfiguration.project_id 投影 WorkItem 列表 |
| **board** | planning | Shared Kernel | Board 列定义与 Sprint 状态映射 (Kanban/Scrum 共享) |
| **planning** | work-item | Customer-Supplier | Sprint.contains_work_item_ids[] (只读 FK) |
| **planning** | board | Customer-Supplier | Board 视图从 Planning.Sprint 投影 (per REQ-PLAN-003) |
| **planning** | relation | Customer-Supplier | Gantt 依赖基于 Relation (per REQ-PLAN-004) |
| **comment** | work-item | Customer-Supplier | Comment.parent = WorkItem (per REQ-COLLAB-001) |
| **comment** | identity | Shared Kernel | @UserId 引用 |
| **comment** | attachment | ACL | Attachment.StorageKey (S3 兼容 Object Storage) |
| **comment** | audit | Separate Ways | Comment Created/Updated/Deleted 全量审计 |
| **relation** | work-item | Customer-Supplier | Relation.source/target = WorkItem (blocks/relates/duplicates, per REQ-COLLAB-002) |
| **relation** | worktree | Customer-Supplier | Relation 含 Worktree 冲突分析源 (per RFC-029) |
| **collaboration** | work-item | Customer-Supplier | Realtime 状态推送 (per requirements §15) |
| **collaboration** | comment | Customer-Supplier | Realtime 推送 Comment / @mention |
| **collaboration** | star-sse | Shared Kernel | 通过 star-sse crate WebSocket 通道 (per star-sse/src/lib.rs) |
| **automation** | work-item | Customer-Supplier | AutomationRule.action = WorkItem transition (per REQ-AUTO-001) |
| **automation** | notification | Customer-Supplier | AutomationRule.action = Notification 触发 (per REQ-NOTIF-001) |
| **automation** | worktree | Customer-Supplier | AutomationRule.action = Worktree reconcile |
| **automation** | workflow | Customer-Supplier | AutomationRule 走 Workflow Guard 校验,不可绕过 (per REQ-AUTO-003 批量操作派生) |
| **integration** | scm | ACL(隔离) | integration 消费 scm Port,提供 SCM Sync / Webhook Receiver |
| **integration** | notification | Customer-Supplier | integration 通过 notification 分发 GitHub/GitLab 事件 |
| **integration** | identity | Customer-Supplier | OIDC/SAML 通过 identity 完成 IdP 联邦 |
| **development** | work-item | Customer-Supplier | DevelopmentExecution.work_item_id 引用 |
| **development** | worktree | Customer-Supplier | Worktree.development_execution_id 引用 |
| **development** | agent | Customer-Supplier | DevelopmentExecution.assignee_agent_id 引用 |
| **development** | change-set | Customer-Supplier | DevelopmentExecution 聚合 ChangeSet[] (per requirements §21) |
| **development** | audit | Separate Ways | Development 状态机事件全量审计 |
| **search**(单独) | work-item | Published Language | 投影 WorkItem → Search Index (worker projection role) |
| **search**(单独) | comment | Published Language | 投影 Comment → Search Index |
| **search**(单独) | agent | Published Language | 投影 AgentSession → Search Index (per requirements §12) |
| **notification**(单独) | work-item | Separate Ways(异步) | 监听 WorkItem StateChanged 触发 |
| **notification**(单独) | feedback | Separate Ways(异步) | 监听 FeedbackCreated 触发 Inbox/Email (per REQ-NOTIF-002 降噪) |
| **notification**(单独) | validation | Separate Ways(异步) | 监听 ValidationFailed 触发 (per REQ-NOTIF-001) |
| **local-runtime** | worktree | Conformist | Local Runtime 上报 Worktree.observed_state (per requirements §23) |
| **local-runtime** | agent | Customer-Supplier | Local Runtime 调 Agent Process (spawn/kill/lease, per ADR-0030) |
| **local-runtime** | audit | Separate Ways | Local Runtime 所有 Command/Observation 全量审计 (per LRT-002) |
| **local-runtime** | identity | Shared Kernel | DeviceId 三重绑定 (tenant+user+project) |

**§3.2 接触面统计 (v0.16)**:
- 22 domain 共 ~140+ 接触点 (原 8 节 ~60 + 本节新增 80+)
- 接触方式分布: Shared Kernel ~10 / Customer-Supplier ~70 / Conformist ~10 / Separate Ways ~30 / Published Language ~10 / ACL ~10
- 全部 22 domain 至少有一条接触面被显式定义,无遗漏

### 3.3 与外部系统的接触

| 外部系统 | 接触方式 | 接触点 |
|---|---|---|
| GitHub | ACL + OHS | SCM Adapter 实现 SCM Port(由 GitHub Adapter) |
| GitLab | ACL + OHS | SCM Adapter 实现 SCM Port(由 GitLab Adapter) |
| Local Runtime | Conformist | Local Runtime 上报 Observed State(直接接受) |
| AI Provider(Codex 等) | ACL + OHS | Agent Adapter 实现 Agent Port |
| SMTP / Email | OHS | Notification Provider 适配器 |
| 浏览器 / WebSocket | OHS | API Gateway 暴露的公开 API |
| **OIDC / SAML IdP** (v0.16 新增) | ACL + OHS | Identity Provider Adapter (per integration-design §5) |
| **Slack / Teams / Lark / Discord IM** (v0.16 新增) | OHS | Notification IM Provider (per integration-design §4) |
| **S3 兼容 Object Storage** (v0.16 新增) | ACL | Attachment / ContextPacket / AgentTranscript 二级存储 (per requirements §14 REQ-DATA-002) |
| **KEDA / Serverless Worker** (v0.16 新增) | Separate Ways | Scale-to-Zero 任务触发 (Repository Analysis / Large Context Build, per requirements §13.5) |
| **Star CLI / star-mcp** (v0.16 新增) | OHS | 对外 CLI + MCP 16 tools 接入点 (per ADR-0026 + ADR-0032) |

---

## 4. 关键 Module 详细设计

> 本章挑选 10 个核心 Module 进行 200-500 行的深度设计。每个 Module 包含:**职责 / 关键实体 / 接口契约 / 关键不变量 / 状态机(如有)/ Requirement 索引 / 跨域交互 / 安全要点**。

### 4.1 domain-worktree(Worktree 一级领域对象)

#### 4.1.1 职责与定位

Worktree 是 Vibe Coding 并行执行的隔离边界,**一级领域对象**(§22.1,REQ-WT-001~003)。不得降级为 Repository Metadata 或 Branch 的附属字段。其设计需支持:

- 多 Agent 同 Repository 并行
- 跨 Worktree Conflict Awareness
- 与 WorkItem Status 独立的状态机
- Observed State 与 Business State 分离(§23.3)

#### 4.1.2 关键实体(字段不写类型,仅列语义)

**Worktree**(聚合根):

- 标识:`worktree_id`, `tenant_id`, `workspace_id`, `project_id`, `work_item_id`
- 关联:`repository_id`, `branch`, `base_branch`, `development_execution_id`
- 物理引用:`runtime_id`(LocalRuntime / SelfHostedRunner / CloudWorkspace),`local_path_reference`(由 Local Runtime 解释,平台不可信)
- 角色:`owner_user_id`, `assigned_agent_id`(可选), `current_agent_session_id`(可选)
- 状态:`status`, `health`, `dirty_state`, `conflict_state`, `ahead`, `behind`
- 内容:`changed_files[]`, `changed_symbols[]`, `test_state`, `build_state`
- 协调:`context_state`, `feedback_state`, `synchronization_state`, `last_activity_at`

**WorktreeStatusObserved**(Projection):高频本地状态,不入核心事务(§14.1,REQ-DATA-003)。

**WorktreeConflict**(实体):File-level / Symbol-level Conflict 记录,关联两个 Worktree。

**WorktreeReconciliationState**(值对象):Desired vs Observed 比对结果。

#### 4.1.3 状态机(§22.2)

```text
CREATED → READY → ASSIGNED → AGENT_RUNNING
       → WAITING_FEEDBACK → FEEDBACK_RECEIVED
       → VALIDATING
       → BLOCKED / CONFLICTED
       → READY_FOR_REVIEW → REVIEWING
       → READY_FOR_COMMIT → COMMITTED
       → PR_OPEN → MERGED
       → ABANDONED → ARCHIVED
```

完整状态机见附录 A.1。

#### 4.1.4 接口契约(方法签名级)

```rust
// crates/domain-worktree/src/port.rs

/// 跨域入口:由 application 编排
pub trait WorktreeCommandPort {
    async fn create_worktree(
        &self,
        cmd: CreateWorktreeCommand,  // 含 work_item_id, repository_id, branch, runtime_id
        actor: ActorContext,           // user_id, device_id, project_id
    ) -> Result<WorktreeId, WorktreeError>;

    async fn assign_to_agent(
        &self,
        cmd: AssignWorktreeCommand,   // 含 agent_id, agent_session_id
        actor: ActorContext,
    ) -> Result<(), WorktreeError>;

    async fn record_observed_state(
        &self,
        cmd: RecordObservedStateCommand, // 含 dirty_state, ahead, behind, current_agent_session_id
        actor: ActorContext,             // 必须是 Local Runtime
    ) -> Result<(), WorktreeError>;

    async fn transition_status(
        &self,
        cmd: TransitionStatusCommand,    // 含 from, to, reason
        actor: ActorContext,
    ) -> Result<WorktreeStatus, WorktreeError>;

    async fn abandon(
        &self,
        cmd: AbandonCommand,             // 含 reason
        actor: ActorContext,
    ) -> Result<(), WorktreeError>;
}

pub trait WorktreeQueryPort {
    async fn get_by_id(&self, id: WorktreeId, viewer: ActorContext) -> Result<Worktree, WorktreeError>;
    async fn list_by_work_item(&self, work_item_id: WorkItemId, viewer: ActorContext) -> Result<Vec<WorktreeSummary>, WorktreeError>;
    async fn list_by_agent(&self, agent_id: AgentId, viewer: ActorContext) -> Result<Vec<WorktreeSummary>, WorktreeError>;
    async fn detect_conflicts(&self, worktree_id: WorktreeId, viewer: ActorContext) -> Result<Vec<WorktreeConflict>, WorktreeError>;
    async fn heatmap(&self, repository_id: RepositoryId, viewer: ActorContext) -> Result<WorktreeHeatmap, WorktreeError>;
}
```

#### 4.1.5 关键不变量(§22,§23.3,§85)

1. **Status Independence**:`Worktree.status` 与 `WorkItem.status` 独立,可同时存在任意组合(REQ-WF-002)
2. **Runtime Anchor**:每个 Worktree 必绑一个 Runtime(Local / Self-hosted / Cloud)
3. **Local Path Opacity**:平台不直接读 `local_path_reference`,仅 Local Runtime 可信
4. **Reconciliation Required**:Local Runtime 重连后必须 Reconcile Desired ↔ Observed(§22.6)
5. **Observed vs Business**:高频本地状态(§22.1 dirty_state, test_state)走 Projection,不入核心事务(REQ-DATA-003)
6. **Stale Display**:UI 必须区分 Current / Possibly Stale / Offline / Unknown(§23.4)
7. **Completion Gate**:进入 `READY_FOR_REVIEW` 需通过 §22.7 列出的 7 项检查

#### 4.1.6 Conflict Intelligence(§22.4)

**第一阶段 File-level**:

```rust
pub struct FileConflictDetector {
    repo: Arc<dyn RepositoryViewPort>,
    heatmap: Arc<dyn WorktreeHeatmapPort>,
}

impl FileConflictDetector {
    /// 同 Repository 下,其他 Worktree 已修改 file_paths 集合
    pub async fn detect(&self, worktree_id: WorktreeId) -> Result<Vec<FileConflict>, WorktreeError>;
    /// Risk Level: None / Low(1-2 file)/ Medium(3-5)/ High(>5 或核心文件)
}
```

**演进到 Symbol-level**(V1,§30.3,REQ-AUT 后续):通过 `repository-analysis` worker 提供的 Symbol 索引实现。

#### 4.1.7 Isolation(§22.5)

Worktree 必须实现以下隔离(由 Local Runtime 强制):

- Filesystem:每个 Worktree 独占目录,通过 Git Worktree 原生机制
- Environment Variable:Agent Process 只读 Project Policy 注入的 Env
- Build Artifact:`target/` 隔离(per-worktree)
- Dependency Cache:可共享,但 cache key 含 worktree_id
- Agent Memory / Context:严格 per-worktree,禁止跨 Worktree 读取
- Secret:仅 Credential Broker 注入,Agent 不可直接读文件系统 Secret
- Port:Local Runtime 分配临时端口池
- Process:每个 Worktree 的 Agent 进程由 Local Runtime 监控
- Temporary File:`/tmp/star-worktree-{worktree_id}/`

#### 4.1.8 Reconciliation(§22.6,§45)

```rust
pub trait WorktreeReconciler {
    /// Local Runtime reconnect 后,Desired State(由 Control Plane 持有)
    /// ↔ Observed State(由 Local Runtime 上报)
    async fn reconcile(&self, runtime_id: RuntimeId) -> ReconciliationReport;
}
```

**Reconciliation 原则**(§45):

- 应用层同步,不引入 K8s-style CRD/Controller
- 偏差 = 不可恢复事件(强制 re-sync 或人工介入),不静默合并
- Reconciliation 本身是 Domain Event,不直接写业务聚合(仅触发 Outbox)

#### 4.1.9 Completion 判定(§22.7,§78)

```text
READY_FOR_REVIEW 前必须全部通过:
1. No Critical Feedback(FEEDBACK_SEVERITY >= HIGH 全部 VERIFIED/REJECTED/SUPERSEDED)
2. Required Tests Pass(Project Policy.required_test_passes)
3. Required Build Pass
4. No Blocking Conflict(本 Worktree 不在对方 ahead set 中)
5. Acceptance Criteria Covered(Validation → AcceptanceCoverage 100%)
6. Required Review Complete(若 Policy.require_review)
7. Git State Known(SCM Sync 状态 = IN_SYNC,无 force-push 未同步)
```

由 Project Policy 提供具体策略;默认策略 = 全部必须。

#### 4.1.10 Requirement 索引

- REQ-WF-002(Status Independence)
- REQ-DEV-001(1 WorkItem → N Worktree)
- REQ-DEV-002(1 Worktree → N AgentSession)
- REQ-DATA-003(Observed State 分离)
- ARCH-OBL-DEV-001(Worktree Isolation)
- ARCH-OBL-DEV-006(Observed State 分离存储与治理)
- WT-001~003(§41 P0 Requirement)
- §22 全章

---

### 4.2 domain-agent(AgentSession + Agent Adapter)

#### 4.2.1 职责与定位

Agent Domain 承担双重职责:

1. **Agent Adapter 抽象**(§24.2):统一 Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local / Future Agent
2. **AgentSession 生命周期**(§24.1):一次 Agent 在某 Worktree 上的执行会话

#### 4.2.2 关键实体

**Agent**(注册表):

- `agent_id`, `agent_type`(Codex / ClaudeCode / GeminiCLI / OpenAICompatible / Local / Future)
- `agent_provider`(厂商标识)
- `agent_version`
- `capabilities[]`(允许的工具 / 命令类别)
- `policy_template_id`(可选)

**AgentSession**(聚合根):

- `session_id`, `agent_id`, `agent_type`, `agent_provider`, `agent_version`
- `worktree_id`, `work_item_id`
- `started_at`, `ended_at`, `status`
- `intent`, `context_packet_id`
- `plan`(执行计划,可选)
- `decisions[]`(Decision Memory 引用)
- `tool_activity_summary`(摘要,非全文)
- `change_set_ids[]`, `validation_result_ids[]`, `feedback_consumed_ids[]`
- `result_summary`
- `token_usage` / `cost_summary`(V1 候选,§24.1 补充,参考竞品 Multica「per-run token 成本可见性」;与 Context Cost Analysis 共用统计口径,不新增独立采集链路)
- `trace_reference`(OpenTelemetry TraceId)

**AgentPolicy**(值对象 + 策略对象):

- `allowed_repositories[]`, `allowed_worktrees[]`, `allowed_paths[]`, `forbidden_paths[]`
- `allowed_tools[]`, `allowed_command_categories[]`
- `network_access`(Allow / Deny / Scoped)
- `secret_access`(BrokerOnly / Scoped / None)
- `max_runtime_seconds`, `max_context_tokens`, `max_change_files`, `max_change_lines`
- `require_review`, `require_test`, `require_approval`

#### 4.2.3 状态机(AgentSession)

```text
CREATED → STARTING → RUNNING
       → WAITING_TOOL → TOOL_RUNNING → TOOL_COMPLETED
       → WAITING_FEEDBACK → FEEDBACK_RECEIVED
       → RUNNING(loop)
       → VALIDATING
       → COMPLETED / FAILED / ABORTED / TIMEOUT
       → CRASHED(由 Local Runtime 上报)
```

**触发者**:

- `CREATED`:`agent --type XX` API 或 AgentSession 自动启动
- `RUNNING → WAITING_FEEDBACK`:Context Compiler 检测到 OpenFeedback
- `WAITING_FEEDBACK → RUNNING`:Feedback 提交
- `VALIDATING → COMPLETED`:ValidationResult.all_passed = true
- `VALIDATING → FAILED`:ValidationResult.critical_failure
- `ABORTED`:用户主动 / Policy 拒绝
- `CRASHED`:Local Runtime 上报(不依赖 Agent 自报)

#### 4.2.4 Agent Adapter 模型(§24.2)

```rust
// crates/domain-agent/src/port.rs

/// 统一 Agent Port(由 infrastructure 层的 Adapter 实现)
#[async_trait]
pub trait AgentPort {
    /// 由 application 调用,在 Local Runtime 中启动 Agent Process
    async fn start(
        &self,
        cmd: StartAgentCommand, // 含 agent_id, worktree_id, context_packet_id, policy
    ) -> Result<AgentHandle, AgentError>;

    /// 发送 Feedback(在 WAITING_FEEDBACK → RUNNING 时)
    async fn submit_feedback(
        &self,
        session_id: AgentSessionId,
        feedback: AgentInstruction, // 由 Context Compiler 编译
    ) -> Result<(), AgentError>;

    /// 停止 Agent(用户 / Policy / Abort)
    async fn stop(&self, session_id: AgentSessionId, reason: StopReason) -> Result<(), AgentError>;

    /// 查询 Agent Process 状态(由 Local Runtime 主动上报为主,此接口为 polling 兜底)
    async fn query_status(&self, session_id: AgentSessionId) -> Result<AgentProcessStatus, AgentError>;
}
```

**禁止**:

- ❌ Domain 层出现 `CodexTool`, `ClaudeCodeEvent` 等厂商类型
- ❌ Domain 层依赖具体 AI Provider SDK

#### 4.2.5 Agent Policy 强制点(§24.3,REQ-PERM-002)

> **关键原则**:Policy 必须由 Application / Authorization 层强制执行,不能只靠 Prompt 告诉 Agent "不要修改 xxx"。

**强制点清单**:

| 强制点 | 在哪一层 | 检查什么 |
|---|---|---|
| Repository 范围 | application 启动 Agent 时 | policy.allowed_repositories |
| Worktree 范围 | Local Runtime Command Scope | policy.allowed_worktrees |
| Path 范围 | Local Runtime Filesystem Scope | policy.allowed_paths / forbidden_paths |
| Tool 范围 | Agent Adapter 解析 Tool Call | policy.allowed_tools |
| Network | Local Runtime Egress Proxy | policy.network_access |
| Secret | Credential Broker | policy.secret_access |
| Runtime Limit | Application 启动时 + Worker 监控 | policy.max_runtime_seconds |
| Context Limit | Context Compiler | policy.max_context_tokens |
| Change Scope | Local Runtime fs watcher + commit gate | policy.max_change_files / max_change_lines |
| Review Gate | application 提交前 | policy.require_review |
| Test Gate | application 提交前 | policy.require_test |
| Approval Gate | application 提交前 | policy.require_approval |

#### 4.2.6 Human-in-the-loop 授权等级(§24.4)

| 动作 | 授权级别 | 实现位置 |
|---|---|---|
| AI Analyze | Auto | 无需审批 |
| AI Suggest | Auto | 无需审批 |
| AI Modify Authorized Worktree | Policy Controlled | AgentPolicy.require_* |
| Commit | Policy Controlled | Worktree → READY_FOR_COMMIT 触发 ProjectPolicy.commit_gate |
| Push | User/Tenant Policy | ProjectPolicy.push_requires_user |
| PR Creation | User/Tenant Policy | ProjectPolicy.pr_creation_requires_user |
| Merge | Protected Action | ProjectPolicy.merge_gate = 必须人类 |
| Production Deployment | 单独授权 | 不在本文档范围(V2) |

#### 4.2.7 Multi-Agent Control(§24.5,§51-53)

**MVP 边界**:允许 `1 Worktree → 1 Agent` 并行,Visibility / Isolation / Feedback / Context / Validation / Conflict Awareness 完整支持。

**禁止 MVP**:

- ❌ Agent Swarm
- ❌ Agent Negotiation
- ❌ Autonomous Planning Society

**Agent Handoff**(§24.5):接管同一 Worktree 时,**不**依赖发送全量聊天记录,生成 Handoff Context Packet:

```rust
pub struct HandoffContextPacket {
    pub objective: String,
    pub current_state: WorktreeSnapshot,
    pub completed_work: Vec<ChangeSetSummary>,
    pub open_work: Vec<OpenTask>,
    pub decisions: Vec<DecisionId>,         // 引用 Active Decision
    pub open_feedback: Vec<FeedbackId>,
    pub changed_symbols: Vec<SymbolRef>,
    pub failed_tests: Vec<TestFailure>,
    pub constraints: Vec<PolicyRef>,
}
```

**Agent Comparison**(§53,V2 候选,§30.4):同 Task 多个 Agent 并行 → Worktree 对比 Diff/Tests/Complexity/Review/Context Cost/Feedback Count。

#### 4.2.8 未来扩展方向:Skill/Playbook 与 Squad(§24.6-24.7,V2/Future 候选,参考竞品 Multica 分析,2026-08-26 补充)

> 以下两项均为方向性登记,不在当前 MVP/V1 范围内实现,仅约束未来设计不得违反已有不变量。

**Skill/Playbook 复用**(§24.6,V2 候选):

- 定位为**只读 Context 素材**,不是可执行代码,不获得独立权限;挂载方式是作为 `domain-context`(§4.4)Context Packet 的一个新增 `SourceType::Skill` Provenance 来源,而不是 `domain-agent` 内部新聚合根。
- 与 `AgentPolicy` 正交:Skill/Playbook 只影响 Prompt/Context 内容,不得绕过 §4.2.5 的 12 个强制点。
- 安全等级视为 Untrusted Content(§28.3),Instruction Priority 不得高于 Trusted Human Policy,对应 RISK-031(Skill/Playbook Content Injection)。

**Squad 分组视图**(§24.7,Future 候选):

- 仅是 WorkItem/Worktree 维度的 Assignee 分组展示(Query 侧),不新增 Command 语义,不引入 Agent 间自主任务分派。
- 必须与 §24.7、INV-AGT-10 一致:**禁止** Agent Swarm / Agent Negotiation / Autonomous Planning Society,分组只能由人类或规则引擎(`domain-automation`)指定 Assignee。

#### 4.2.9 Requirement 索引

- REQ-PERM-002(Policy 由 Application 强制)
- REQ-DEV-002(1 Worktree → N AgentSession)
- REQ-DEV-003(1 AgentSession → 1 Active Worktree)
- ARCH-OBL-DEV-001(Worktree Isolation → Agent 限制在授权 Runtime/Repository/Worktree)
- AGT-001/002(§41 P0)
- §24 全章(含 §24.6/24.7 未来扩展)
- §28.4(Agent Secret Boundary)

---

### 4.3 domain-feedback(结构化 Feedback)

#### 4.3.1 职责与定位

Feedback 是**一级领域对象**(§25.1,REQ-FBK-001/002),**禁止**降级为 Comment。需支持精准目标绑定(WorkItem→Diff Hunk)、结构化字段(Expected/Preserve/Prohibit)、消费追踪(VERIFIED/REJECTED/SUPERSEDED)。

#### 4.3.2 关键实体

**Feedback**(聚合根):

- `feedback_id`, `tenant_id`, `project_id`
- `target`:FeedbackTarget 枚举(WorkItem / Requirement / AcceptanceCriterion / Worktree / AgentSession / File / Symbol / DiffHunk / Test / Build / RuntimeLog / ArchitectureDecision / PullRequest / ReviewFinding)
- `type`:FeedbackType 枚举(Fix / Preserve / Refactor / Reject / Question / Constraint / Architecture / Security / Performance / Testing / Scope)
- `severity`(P0-P3)
- `intent`(短句,如"将 auth 抽象为 AuthProvider")
- `expected_behavior`(预期行为)
- `preserve`(必须保留的语义/接口)
- `prohibit`(禁止的修改)
- `acceptance_criteria_id`(可选,关联到具体 AC)
- `author_user_id`, `author_agent_id`(AI 自己提的 Feedback 也要记录)
- `status`(OPEN/ACKNOWLEDGED/APPLIED/VERIFIED/REJECTED/SUPERSEDED)
- `created_at`, `resolved_at`, `resolution_evidence[]`

**FeedbackConsumedEvent**(Projection):记录哪条 Feedback 被哪个 AgentSession / ContextPacket / ChangeSet 消费。

#### 4.3.3 Feedback Target 全粒度(§25.1)

```rust
pub enum FeedbackTarget {
    WorkItem(WorkItemId),
    Requirement(RequirementId),
    AcceptanceCriterion(AcceptanceCriterionId),
    Worktree(WorktreeId),
    AgentSession(AgentSessionId),
    File { repository_id: RepositoryId, path: String, line_range: Option<Range<u32>> },
    Symbol { repository_id: RepositoryId, symbol_ref: SymbolRef },
    DiffHunk { commit_id: CommitId, hunk_index: u32 },
    Test { test_id: TestId },
    Build { build_id: BuildId },
    RuntimeLog { agent_session_id: AgentSessionId, log_offset: Range<u64> },
    ArchitectureDecision(DecisionId),
    PullRequest(PullRequestRef),
    ReviewFinding(ReviewFindingRef),
}
```

#### 4.3.4 Precise Feedback(§25.2)

> 解决传统 Coding Agent Feedback "这里不对,重新做" 信息密度不足。

**示例(§25.2 原文)**:

用户选中 `src/auth/service.rs::authenticate_user` 提交:

```text
Target: Symbol(auth_service::authenticate_user)
Type: Architecture
Severity: P1
ExpectedBehavior: 使用 AuthProvider abstraction
Preserve: Public API, Existing Error Model
Prohibit: Database Schema Change
```

→ Context Compiler 生成结构化 AgentInstruction(见 §4.4)。

#### 4.3.5 状态机(§25.3)

```text
OPEN
  ↓ (Agent 下一次 Session 启动时拉取)
ACKNOWLEDGED
  ↓ (Agent 提交 ChangeSet 包含对应 Target)
APPLIED
  ↓ (Validation 跑过)
VERIFIED
  ↓ (用户标记 / 被新 Feedback Supersede)
SUPERSEDED
  ↓ (用户标记无效)
REJECTED
```

合法迁移:
- `OPEN → ACKNOWLEDGED`:Agent 拉取并加入 Context Packet
- `OPEN → REJECTED`:用户在 OPEN 状态下直接关闭
- `ACKNOWLEDGED → APPLIED`:Agent 提交含该 Target 的 ChangeSet
- `APPLIED → VERIFIED`:Validation 跑过对应 AC
- `APPLIED → REJECTED`:用户明确拒绝
- `OPEN/ACKNOWLEDGED → SUPERSEDED`:被新 Feedback 取代

#### 4.3.6 Feedback Inbox 与 Intervention Queue(§25.4,§49-50)

**Feedback Inbox**(聚合查询):

```rust
pub trait FeedbackInboxQueryPort {
    async fn list_for_user(&self, user_id: UserId, project_ids: Vec<ProjectId>, filter: FeedbackInboxFilter) -> Vec<FeedbackInboxItem>;
}

pub struct FeedbackInboxItem {
    pub feedback: Feedback,
    pub worktree: Option<WorktreeSummary>,
    pub agent_session: Option<AgentSessionSummary>,
    pub priority: Priority,        // P0/P1/P2/P3
    pub source: FeedbackSource,    // AgentWaitingFeedback, FailedAcceptance, ReviewFinding, TestFailure, ArchitectureQuestion, Conflict, AgentClarification
    pub sla_due_at: Option<DateTime>,  // 根据 ProjectPolicy 计算
}
```

**Intervention Queue 优先级**(§25.4 原文):

```text
P0  Security Decision
P1  Architecture Feedback
P1  Merge Conflict
P2  Test Failure
P2  Agent Question
P3  Optional Refactor
```

#### 4.3.7 关键不变量

1. **Target 必须可解析**:Feedback 创建时必须能解析 target_ref 到当前存在的对象
2. **Status 转换必须可审计**:每次状态迁移写 AuditEvent
3. **Supersede 必须有 successor**:新 Feedback 必须显式引用被取代的 Feedback
4. **Cross-Worktree 禁止**:Feedback 不得自动修改未经授权的 Worktree(§37 AC 示例 2)

#### 4.3.8 Requirement 索引

- REQ-FBK-001(全粒度 Target 反馈)
- REQ-FBK-002(Feedback 消费追踪)
- ARCH-OBL-DEV-002(Context Traceability → Feedback 来源可追溯)
- §25 全章
- §37 AC 示例 2

---

### 4.4 domain-context(Context Compiler + Decision Memory)

#### 4.4.1 职责与定位

Context Compiler **不是 LLM**,而是"根据当前任务、代码状态、历史决策和反馈,为 Coding Agent 生成最小必要 Context Packet 的确定性/半确定性系统能力"(§26.1)。

#### 4.4.2 关键实体

**ContextPacket**(聚合根,§26.2):

- `packet_id`, `tenant_id`, `project_id`
- `work_item_id`, `worktree_id`, `agent_session_id`(消费方)
- `intent`, `objective`, `scope`
- `relevant_requirements[]`, `acceptance_criteria[]`
- `relevant_files[]`, `relevant_symbols[]`
- `architecture_constraints[]`, `existing_decisions[]`
- `current_change_set_id`, `open_feedback[]`, `failed_validation[]`
- `preserve_rules[]`, `prohibited_changes[]`
- `expected_output`, `verification_instructions`
- `token_budget`, `actual_tokens`
- `priority_layers`(P0/P1/P2/P3/P4)
- `provenance`:Vec<ProvenanceEntry>(每条引用源的标识)
- `created_at`, `created_by`(user_id 或 system:context-compiler)

**ProvenanceEntry**(值对象,§26.3):

```rust
pub struct ProvenanceEntry {
    pub source_type: SourceType, // Requirement / AcceptanceCriterion / Decision / Feedback / File / Symbol / Test / ADR / FailedValidation / OpenFeedback / Skill(V2 候选,§24.6)
    pub source_id: SourceId,
    pub version: u64,            // 用于追踪被取代的版本
    pub included_at_layer: Priority,
}
```

> **Skill/Playbook 来源**(`SourceType::Skill`,V2 候选,§4.2.8,参考竞品 Multica 分析,2026-08-26 补充):挂载方式与 File/Symbol 等其他来源一致,必须携带 Provenance;安全等级视为 Untrusted Content(§28.3),Instruction Priority 不得高于 P0 Explicit Human Constraint。

**Decision**(聚合根,§26.5):

- `decision_id`, `tenant_id`, `project_id`
- `statement`, `reason`, `scope`
- `source`(ConversationId / RequirementId / ArchitectureReviewId)
- `status`(Active / Superseded / Invalidated)
- `superseded_by`, `invalidated_by`
- `created_at`, `created_by`

#### 4.4.3 Context Packet 字段(§26.2)

```rust
pub struct ContextPacket {
    pub packet_id: ContextPacketId,
    pub intent: String,
    pub objective: String,
    pub scope: WorktreeScope,                 // 含 allowed_paths / forbidden_paths
    pub relevant_requirements: Vec<RequirementId>,
    pub acceptance_criteria: Vec<AcceptanceCriterionId>,
    pub relevant_files: Vec<FileRef>,
    pub relevant_symbols: Vec<SymbolRef>,
    pub architecture_constraints: Vec<DecisionId>,
    pub existing_decisions: Vec<DecisionId>,
    pub current_change_set: Option<ChangeSetId>,
    pub open_feedback: Vec<FeedbackId>,
    pub failed_validation: Vec<ValidationResultId>,
    pub preserve_rules: Vec<PreserveRule>,
    pub prohibited_changes: Vec<ProhibitedChange>,
    pub expected_output: String,
    pub verification_instructions: Vec<VerificationStep>,
    pub token_budget: TokenBudget,
    pub actual_tokens: u32,
    pub priority_layers: PriorityLayers,
    pub provenance: Vec<ProvenanceEntry>,
    pub created_at: DateTime<Utc>,
    pub created_by: CreatedBy,
}
```

#### 4.4.4 Token Budget 与优先级(§26.4)

```text
P0  Explicit Human Constraint
P1  Acceptance Criteria / Security Requirement / Open Feedback
P2  Relevant Current Code / Failed Test
P3  Historical Discussion
P4  Low-confidence AI Summary
```

**Token Budget 分级草案**(需 TBD-MEASURE 校准,§46 决策表 J.3):

| Model Tier | Total Budget | P0 | P1 | P2 | P3 | P4 |
|---|---|---|---|---|---|---|
| Mini (Codex Haiku 等) | 32K | 2K | 4K | 12K | 8K | 6K |
| Standard (Codex Sonnet 等) | 128K | 4K | 12K | 60K | 32K | 20K |
| Pro (Codex Opus 等) | 200K | 8K | 24K | 100K | 48K | 20K |

> 草案值,需 PoC 校准(§11 POC-023,§15 Open Issue J.3)。

**强制规则**:

- 不得让历史 Agent 对话无限增长(§26.4)
- P0 不可被裁剪,只可被新的 P0 取代
- Decision 优先于聊天历史(§26.5)

#### 4.4.5 Context Provenance(§26.3)

> 所有进入 AI 的重要 Context 必须可追溯来源(例:`Requirement REQ-102 / ADR-004 / Feedback FBK-221 / Test TEST-932 / File auth.rs / Symbol AuthService::login`)。AI 生成的重要 Decision 必须关联 Source Context、AgentSession、Timestamp、Worktree。**不得形成无法解释来源的 "AI Memory Blob"**(§26.3)。

**Provenance 强制规则**:

- 每个 `relevant_*` 字段必须带 `ProvenanceEntry`
- Decision 必须带 `source` 引用
- Context Packet 必须可重放(给定 Provenance 可重新生成)

#### 4.4.6 Decision Memory(§26.5)

```rust
pub trait DecisionMemoryPort {
    async fn create(&self, cmd: CreateDecisionCommand) -> Result<DecisionId, DecisionError>;
    async fn supersede(&self, cmd: SupersedeDecisionCommand) -> Result<DecisionId, DecisionError>;
    /// 使某个 Decision 失效(不取代,只是标记无效)
    async fn invalidate(&self, cmd: InvalidateDecisionCommand) -> Result<(), DecisionError>;
    async fn list_active(&self, project_id: ProjectId) -> Result<Vec<Decision>, DecisionError>;
    async fn trace(&self, decision_id: DecisionId) -> Result<DecisionTrace, DecisionError>;
}
```

**操作**:

- Create / Supersede / Invalidate / Trace(§26.5)
- Active Decision = Context Compiler 优先来源(§26.5)

#### 4.4.7 从结构化 Feedback 编译 Agent Instruction(§25.2,§26 派生)

```rust
/// 由 Context Compiler 在 Feedback 被消费时调用
pub trait FeedbackToInstructionCompiler {
    fn compile(
        &self,
        feedback: &Feedback,
        target: &ResolvedTarget,
        project_policy: &ProjectPolicy,
    ) -> Result<AgentInstruction, CompilerError>;
}

pub struct AgentInstruction {
    pub header: String,                 // "针对 auth_service::authenticate_user 的修改要求"
    pub required: Vec<String>,          // 必须做
    pub preserve: Vec<String>,          // 必须保留
    pub prohibit: Vec<String>,          // 禁止
    pub acceptance: Vec<String>,        // 验收标准
    pub token_estimate: u32,
    pub source_feedback_ids: Vec<FeedbackId>,
}
```

#### 4.4.8 Handoff Context Packet(§24.5,§52)

见 §4.2.7 中的 `HandoffContextPacket`。

#### 4.4.9 Requirement 索引

- REQ-CTX-001(Context Packet 自动生成)
- REQ-CTX-002(Context Provenance 保留)
- ARCH-OBL-DEV-002(Context Traceability)
- §26 全章
- 决策表 N(Top 10 Context Engineering Decisions)

---

### 4.5 domain-validation(Validation Domain + Acceptance Coverage)

#### 4.5.1 职责与定位

> AI 修改不能以"Agent says done"作为完成条件(§27.3,VAL-001)。ValidationResult 须覆盖 Build、Unit Test、Integration Test、Lint、Format、Static Analysis、Security Check、Acceptance Check、Review、Custom Validation。

#### 4.5.2 关键实体

**ValidationResult**(聚合根,§27.1):

- `validation_id`, `tenant_id`, `project_id`
- `work_item_id`, `worktree_id`, `agent_session_id`, `change_set_id`, `commit_id`(可选)
- `triggered_by`(User / Agent / Webhook / Schedule)
- `kind`:ValidationKind 枚举(Build / UnitTest / IntegrationTest / Lint / Format / StaticAnalysis / SecurityCheck / AcceptanceCheck / Review / CustomValidation)
- `status`(Pending / Running / Passed / Failed / Errored / Skipped)
- `started_at`, `completed_at`
- `evidence_refs[]`:EvidenceReference(指向 TestReport, BuildArtifact 等,可存储在 Object Storage)
- `failure_summary`, `log_excerpt_ref`
- `policy_required`(是否 ProjectPolicy 必需)
- `is_ai_complete_claim`(bool,标识是否 Agent 自我声明完成)

**AcceptanceCoverage**(§27.2):

- `coverage_id`, `acceptance_criterion_id`
- `validation_result_ids[]`, `review_finding_ids[]`, `human_acknowledged_by`
- `coverage_status`(Covered / Partial / Uncovered / Disputed)

**ValidationPolicy**(值对象):

- 哪些 kind 是 Required / Optional
- Pass 阈值(如 Unit Test Coverage >= 80%)
- 是否允许 Agent 自报

#### 4.5.3 ValidationKind 清单(§27.1)

| Kind | 来源 | 必需性默认 |
|---|---|---|
| Build | CI / Local Runtime | Required |
| UnitTest | CI / Local Runtime | Required |
| IntegrationTest | CI | ProjectPolicy |
| Lint | CI / Local Runtime | Required |
| Format | CI / Local Runtime | Required |
| StaticAnalysis | CI | Optional(V1 Required) |
| SecurityCheck | CI | Required(P0/P1 Project) |
| AcceptanceCheck | AI / Human | Required |
| Review | Human | ProjectPolicy.require_review |
| CustomValidation | User-Defined | Optional |

#### 4.5.4 Acceptance Coverage 映射(§27.2)

```rust
pub trait AcceptanceCoveragePort {
    /// 建立 AC → ValidationEvidence 映射
    async fn link(&self, cmd: LinkAcceptanceEvidenceCommand) -> Result<(), ValidationError>;
    /// 计算某 WorkItem 的 AC 覆盖率
    async fn coverage(&self, work_item_id: WorkItemId) -> Result<AcceptanceCoverageReport, ValidationError>;
}

pub struct AcceptanceCoverageReport {
    pub work_item_id: WorkItemId,
    pub total_criteria: u32,
    pub covered: u32,
    pub partial: u32,
    pub uncovered: u32,
    pub disputed: u32,
    pub per_criterion: Vec<AcceptanceCriterionCoverage>,
}
```

#### 4.5.5 AI Completion 判定链(§27.3,§77)

```text
AgentSession.ended_at 触发
    ↓
ValidationStarted (自动)
    ↓
ValidationPassed (全部 ProjectPolicy.required validation 跑过)
    ↓
AcceptanceCoverage = 100%
    ↓
FeedbackResolution = (No Open Critical Feedback)
    ↓
Human/Policy Gate (ProjectPolicy.merge_gate)
    ↓
READY_FOR_REVIEW
```

**禁止**:`Agent: Done → WorkItem Done` 的简单映射(§27.3)。

**关键不变量**:`is_ai_complete_claim` 字段为 true 时,必须经过 `ValidationPassed && AcceptanceCoverage==100 && FeedbackResolved && GateApproved` 四重门,缺一不可。

#### 4.5.6 Requirement 索引

- VAL-001(§41 P0:AI 完成不依赖自我报告)
- ARCH-OBL-DEV-005(Validation Evidence)
- §27 全章
- 决策表 K.6

---

### 4.6 domain-local-runtime(集群外 Runtime 的服务器侧 Registry / Port)

#### 4.6.1 职责与定位

> **重要区分**:本节描述的是**服务器侧**的 Runtime Registry / Port(`domain-local-runtime` crate,跑在 work-core 进程内,部署于 K3s Cluster 内),不是 Local Daemon 二进制进程本身。Local Daemon 是独立 Rust 二进制,运行在 Developer Machine / Self-hosted Runner / Cloud Workspace 上,通过 Secure Channel 与本 crate 对接,部署拓扑见 §1.1 LocalRuntime 子图。两个制品命名易混,本节描述的是前者。

`domain-local-runtime` 的职责是管理集群外 Local Daemon 的注册、命令下发、Observation 接收。它**不**实现 Local Daemon 进程本身,Local Daemon 进程属于另一个独立制品(Local Daemon Binary),不在 `crates/domain-*` 任何 crate 内。

Local Runtime **不**属于 Kubernetes Application Workload(§23.1),服务器端最小闭环(`gateway / identity / work-core / worker`)保持不变(§13.1,§23.1)。

#### 4.6.2 关键实体

**Runtime**(注册表,§23.6):

- `runtime_id`, `tenant_id`, `project_id`
- `kind`:RuntimeKind(LocalMachine / SelfHostedRunner / CloudWorkspace / FutureRuntime)
- `device_identity`(由 domain-identity 提供)
- `capabilities[]`(Git / Build / Test / StaticAnalysis / Symbol)
- `status`(Online / Offline / Stale)
- `last_heartbeat_at`, `version`

**RuntimeCommand**(白名单命令,§23.2):

```rust
pub enum RuntimeCommand {
    GitStatus(GitStatusQuery),
    CreateWorktree(CreateWorktreeArgs),
    ReadDiff(ReadDiffArgs),
    RunApprovedTest(RunApprovedTestArgs),
    QueryAgentStatus(AgentSessionId),
    SubmitFeedback(SubmitFeedbackArgs),
    StartAuthorizedAgentSession(StartAgentSessionArgs),
    StopAgentSession(StopAgentSessionArgs),
    /// 严禁出现 ExecuteArbitraryShell(§23.2)
}
```

**RuntimeObservation**(上报事件):

```rust
pub enum RuntimeObservation {
    WorktreeStatusObserved(WorktreeObservedState),
    AgentSessionStateObserved(AgentObservedState),
    BuildCompleted(BuildObservation),
    TestCompleted(TestObservation),
    DiffAvailable(DiffRef),
    Heartbeat(Heartbeat),
    Disconnected(DisconnectReason),
}
```

#### 4.6.3 Security Boundary(§23.2,LRT-001/002)

**强制项**:

| 项 | 实现位置 | 备注 |
|---|---|---|
| Device Identity | Local Runtime 启动时由 Control Plane 颁发 | 设备证书 |
| Device Registration | Tenant Admin 审批 | 设备注册表 |
| User Binding | Control Plane 校验 device ↔ user | 设备 ↔ 用户 |
| Tenant Binding | 设备仅可见绑定 Tenant 的 Project | 多租户隔离 |
| Project Binding | 设备仅可见绑定 Project 的 Repository | 项目级隔离 |
| Repository Authorization | 每条命令带 Repository 范围 | SCM Adapter 二次校验 |
| Short-lived Credential | mTLS 证书 1h,Command Token 5min | §28.4 |
| Mutual Authentication | mTLS 双向认证 | TLS 1.3 |
| Command Authorization | 每次 Command 由 Control Plane 验证 | 白名单 |
| Command Scope | 命令带 Repository/Worktree/Path 范围 | 不可越界 |
| Filesystem Scope | Local Runtime 强制 Path Jail | syscall 拦截 |
| Process Scope | Local Runtime 监控所有子进程 | 禁止 fork outside scope |
| Secret Isolation | Credential Broker | 进程 Env 隔离 |
| Agent Credential Isolation | 仅 Agent 进程可读 | OS-level 隔离 |
| Audit | 所有命令/上报写 Audit | 不脱敏但加密 |
| Revocation | Control Plane 主动撤销 | 设备黑名单 |
| Remote Disable | Control Plane 强制停机命令 | §34 Runtime Impersonation 防护 |

**严禁出现的能力**:

- ❌ `ExecuteArbitraryShell(cmd: String)`
- ❌ `ReadArbitraryFile(path: String)`
- ❌ `WriteArbitraryFile(path: String, content: String)`
- ❌ 任何 `*` 范围的命令

#### 4.6.4 Local-first State(§23.3,REQ-DATA-003)

**Server Truth**(写入 PostgreSQL):WorkItem, Feedback, Requirement, Permission, Decision, Audit。

**Local Observation**(Local Runtime → Projection):Dirty Files, Local Git Status, Running Agent PID, Current Worktree Path, Local Test Process。

**同步后形成 Observed Development State**(Projection),**不得**将瞬时 Local State 当成永久业务事实。

#### 4.6.5 State Synchronization(§23.4)

**协议**:Snapshot(启动时全量)+ Incremental Event(运行中)+ Heartbeat(30s)+ Sequence(Numbered)+ Version(Vector Clock per Project)+ Offline(本地缓存)+ Reconnect(全量 + 增量)+ Replay(idempotency_key 去重)+ Conflict(显式 report)+ Idempotency(由 Server 端去重)+ Stale State(标记 server_time - last_heartbeat > threshold)。

**UI 区分**:

- `Current`(last_heartbeat < 60s)
- `Possibly Stale`(60s ≤ last_heartbeat < 300s)
- `Offline`(last_heartbeat ≥ 300s 或无记录)
- `Unknown`(启动 < 60s)

**严禁**显示虚假的实时状态(§23.4)。

#### 4.6.6 Runtime 抽象(§23.6)

```rust
pub trait RuntimePort {
    async fn execute_command(&self, cmd: RuntimeCommand) -> Result<RuntimeCommandResult, RuntimeError>;
    /// 由 Local Runtime 主动调用,上报 Observed State
    async fn report_observation(&self, obs: RuntimeObservation) -> Result<(), RuntimeError>;
    /// 由 Local Runtime 主动调用,拉取 Desired State(可选双向)
    async fn fetch_desired_state(&self) -> Result<DesiredStateSnapshot, RuntimeError>;
}
```

**未来 Runtime 候选**(§23.6):

- Developer Laptop(默认)
- Self-hosted Runner(企业)
- Cloud Workspace(GitHub Codespaces 等)
- Ephemeral Coding Environment(K8s 上的临时 Pod)

Domain 层不区分具体 Runtime 类型,通过 `RuntimeKind` 枚举实现多态。

#### 4.6.7 Fault Model(§23.5,§44)

**必须处理的故障**:

- Developer Machine Offline
- Daemon Crash
- Agent Crash
- Git Lock
- Worktree Deleted
- Repository Moved
- Branch Rebased
- Force Push
- Disk Full
- Build Process Hung
- Credential Expired
- Network Interrupted
- Version Mismatch(§29)

**UI 行为**:UI 禁止把最后一次状态永久显示成 "Running"(§23.5)。所有 Stale 状态必须可见。

#### 4.6.8 Reconciliation(§22.6,§45)

见 §4.1.8。Local Runtime reconnect 后触发 Desired ↔ Observed 比对。

#### 4.6.9 Requirement 索引

- LRT-001(Local Runtime 身份认证)
- LRT-002(无任意 Shell)
- ARCH-OBL-DEV-004(Local Runtime Security)
- §23 全章
- §28.3(Prompt Injection 防护中,Local Runtime 是第一道防线)
- §34 Threat Model

---

### 4.7 domain-scm(SCM Adapter 抽象 + Repository Ownership)

#### 4.7.1 职责与定位

SCM Domain 通过统一 Port 接入 GitHub / GitLab / 未来 SCM(§19.1,REQ-SCM-001/002)。**Domain 层不得出现厂商特有对象**(`GitHubPullRequestObject` / `GitLabMergeRequestEntity` 等)。

> **扩展优先级**(REQ-SCM-003,V2 候选,解决 J-SCM-01,参考竞品 Multica「Any Git host / Self-hosted included」定位,2026-08-26 补充):自建 Git(Gitea / Forgejo)排在 Bitbucket / Azure DevOps 之前,理由是本节 ACL 已完成厂商对象隔离,新增 Adapter 边际成本低于新建领域模型;不改变 §19.2 "系统不承担完整 Git Server 职能"的边界。

#### 4.7.2 关键实体

**Repository**(聚合根,§19.2):

- `repository_id`, `tenant_id`, `project_id`
- `external_id`(在 GitHub/GitLab 中的 ID)
- `provider`(GitHub / GitLab / Gitea / Forgejo / Bitbucket / Future)
- `url`, `default_branch`
- `ownership`:RepositoryOwnership(Connected / Mirrored / Managed / LocalOnly)
- `last_sync_token`, `last_synced_at`
- `sync_status`(InSync / Behind / Ahead / Conflict / Disabled)

**Branch**(实体):

- `branch_id`, `repository_id`, `name`
- `head_commit_id`, `base_commit_id`(可选)
- `protected`(bool)

**Commit**(实体):

- `commit_id`, `repository_id`, `sha`
- `author`, `committer`, `message`
- `parent_shas[]`, `tree_sha`
- `linked_work_item_id`(可选,通过 Commit Link 关联)

**PullRequest**(实体,统一抽象 GitHub PR 与 GitLab MR):

- `pull_request_id`, `repository_id`, `external_id`
- `source_branch`, `target_branch`
- `title`, `description`
- `author`, `state`(Open / Merged / Closed / Draft)
- `review_ids[]`, `pipeline_ids[]`
- `linked_work_item_id`(可选)

**Review / Pipeline / Webhook Event**(实体):统一抽象,具体厂商细节由 ACL 翻译。

#### 4.7.3 SCM Port 抽象

```rust
// crates/domain-scm/src/port.rs

#[async_trait]
pub trait ScmPort {
    /// 仓库元数据
    async fn get_repository(&self, external_id: ExternalRepositoryId) -> Result<Repository, ScmError>;
    async fn list_branches(&self, repository_id: ExternalRepositoryId) -> Result<Vec<Branch>, ScmError>;
    async fn get_commit(&self, repository_id: ExternalRepositoryId, sha: &str) -> Result<Commit, ScmError>;
    async fn get_pull_request(&self, repository_id: ExternalRepositoryId, external_pr_id: &str) -> Result<PullRequest, ScmError>;
    async fn list_pull_requests(&self, repository_id: ExternalRepositoryId, filter: PullRequestFilter) -> Result<Vec<PullRequest>, ScmError>;

    /// 写入操作(慎用,需 Permission 校验)
    async fn create_pull_request(&self, cmd: CreatePullRequestCommand) -> Result<PullRequest, ScmError>;
    async fn add_comment(&self, cmd: AddCommentCommand) -> Result<(), ScmError>;
    async fn request_review(&self, cmd: RequestReviewCommand) -> Result<(), ScmError>;

    /// Webhook 注册
    async fn register_webhook(&self, cmd: RegisterWebhookCommand) -> Result<WebhookHandle, ScmError>;
}
```

**ACL 位置**:`crates/infrastructure/src/scm/github.rs`, `crates/infrastructure/src/scm/gitlab.rs`,未来 `gitea.rs` 等。

#### 4.7.4 Repository Ownership 分类(§19.2)

| Ownership | 定义 | 平台角色 | 数据真相 |
|---|---|---|---|
| **Connected** | 外部 GitHub/GitLab 是 SoR,平台只读镜像 | Link / Pull | 外部 SCM |
| **Mirrored** | 平台单向镜像到内部 Mirror | 读优化 | 外部 SCM(可降级) |
| **Managed** | 平台创建并管理,外部只读 | Push 限制 | 平台 = 临时 SoR,但仍受外部保护分支约束 |
| **LocalOnly** | 仅 Local Runtime 可见 | 实验 | Local Runtime |

**MVP 范围**:仅 Connected(§30.6 强化:不自建 Git Server)。

#### 4.7.5 Bidirectional Link 原则(§18.1,§25)

> 禁止盲目双向同步。必须明确区分 4 类关系:

| 关系类型 | 说明 | 典型用例 |
|---|---|---|
| **Link** | 仅建立引用关系,无数据移动 | WorkItem ↔ GitHub Issue(默认) |
| **Mirror** | 单向镜像 | Worktree Status → External Status Check |
| **Bidirectional Sync** | 双向同步(需评估 Loop) | PR Comment ↔ WorkItem Comment(慎) |
| **Platform-owned** | 数据所有权归平台,外部仅引用 | WorkItem, Worktree, Feedback |

**强制要求**(§18.1):

- 每条关系定义 `Source System`, `Ownership`, `Version`, `External ID`, `Sync Token`, `Last Synced`, `Conflict Strategy`

**示例**:WorkItem ↔ GitHub Issue 默认 Link(非 Bidirectional Sync),仅建立 Webhook 让平台知道 Issue 状态变化;不反向把 WorkItem 状态写入 Issue。

#### 4.7.6 Sync Token & Conflict Strategy(§18.1)

```rust
pub struct SyncState {
    pub sync_token: String,        // ETag / X-Next-Sync-Token / cursor
    pub last_synced_at: DateTime<Utc>,
    pub conflict_strategy: ConflictStrategy, // LatestWins / FirstWins / ManualReview / Bidirectional
}

pub enum ConflictStrategy {
    LatestWins,                   // 外部 SoR,平台服从
    FirstWins,                    // 平台 First
    ManualReview,                 // 创建人工 Conflict 任务
    Bidirectional {               // 慎用,需 Loop 防护
        platform_field: String,
        external_field: String,
    },
}
```

#### 4.7.7 Requirement 索引

- REQ-SCM-001/002
- ARCH-OBL-DEV-003(SCM Independence)
- SCM-001(§41 P0)
- §18,§19 全章

---

### 4.8 domain-development(ChangeSet + DevelopmentExecution 聚合)

#### 4.8.1 职责与定位

DevelopmentExecution 聚合 WorkItem 在真实代码环境中的一次或多次执行(§21)。ChangeSet **不只存 Git Diff**,需承载 Files / Symbols / Diff / Risk Signals 等结构化信息(§21.1)。

#### 4.8.2 关键实体

**DevelopmentExecution**(聚合根,§21):

- `execution_id`, `tenant_id`, `project_id`
- `work_item_id`, `repository_id`
- `worktree_ids[]`(1..N)
- `agent_session_ids[]`
- `change_set_ids[]`
- `validation_result_ids[]`
- `feedback_ids[]`
- `commit_ids[]`
- `pull_request_ids[]`
- `started_at`, `ended_at`
- `execution_state`

**ChangeSet**(聚合根,§21.1):

- `change_set_id`, `tenant_id`, `project_id`
- `worktree_id`, `agent_session_id`, `commit_id`
- `files[]`:Vec<FileChange>(path, status[Added/Modified/Deleted/Renamed/Generated], old_path, lines_added, lines_deleted)
- `symbols[]`:Vec<SymbolChange>(symbol_ref, status, old_signature)
- `diff_reference`:DiffReference(指向 Object Storage 中的 diff artifact)
- `added_lines`, `deleted_lines`, `renamed_files`, `generated_files`
- `dependency_changes[]`:Vec<DependencyChange>(package, from, to)
- `schema_changes[]`:Vec<SchemaChange>(file, ddl_summary)
- `config_changes[]`:Vec<ConfigChange>(file, key_path, old_value, new_value)
- `test_changes[]`:Vec<TestChange>(test_id, status, coverage_delta)
- `risk_signals[]`:Vec<RiskSignal>(type, severity, source, evidence)
- `created_at`

**RiskSignal**(值对象):

```rust
pub struct RiskSignal {
    pub kind: RiskKind,            // LargeChange / GeneratedFile / SchemaChange / DependencyUpgrade / SecurityHint / TestCoverageDrop / ConflictRisk / AISelfClaim
    pub severity: Severity,        // Info / Low / Medium / High / Critical
    pub source: RiskSource,        // StaticAnalysis / Lint / AIClassifier / Human / Heuristic
    pub evidence: String,          // 简短描述,不存全文
    pub suggested_action: Option<String>,
}
```

#### 4.8.3 ChangeSet 数据所有权

| 数据 | 所有权 | 存储 |
|---|---|---|
| files / symbols 摘要 | ChangeSet | PostgreSQL |
| diff_reference | ChangeSet → Object Storage | Object Storage(如 S3 兼容) |
| added_lines / deleted_lines | ChangeSet(可由 diff 派生) | PostgreSQL |
| dependency_changes | ChangeSet(由 dependency parser 提取) | PostgreSQL |
| schema_changes | ChangeSet(由 schema diff 工具提取) | PostgreSQL |
| config_changes | ChangeSet(由 config diff 工具提取) | PostgreSQL |
| test_changes | ChangeSet + ValidationResult 联合 | PostgreSQL |
| risk_signals | ChangeSet(由多种分析器 + AI 评估) | PostgreSQL |

> **严禁**:把整个 diff 全文塞入 PostgreSQL 热表(REQ-DATA-002)。

#### 4.8.4 ChangeSet 与 Worktree 的关系

```text
Worktree
  ↓ (1..N)
ChangeSet (每个 AgentSession 提交一次)
  ↓ (1..1)
Commit
  ↓ (0..1)
PullRequest
```

**强制**:1 ChangeSet 关联 1 Commit,1 Commit 可被 0..1 PullRequest 引用。

#### 4.8.5 Risk Signal 触发与门控

| Risk Signal | 来源 | 默认门控 |
|---|---|---|
| LargeChange(>500 lines) | Heuristic | 触发 ProjectPolicy.require_review |
| GeneratedFile(`*.pb.go`, `migrations/*.sql` 等) | FileNamePattern | 触发 ProjectPolicy.require_review |
| SchemaChange | SchemaDiff Tool | 强制 Reviewer |
| DependencyUpgrade | Cargo.lock / package.json diff | ProjectPolicy |
| SecurityHint | StaticAnalysis | P0/P1 Project 强制 Review |
| TestCoverageDrop | Coverage Tool | ProjectPolicy |
| ConflictRisk | Worktree Conflict Detector | 不阻止,但 Notification |
| AISelfClaim | Agent.report_done | 必须走 Validation Chain(§4.5) |

#### 4.8.6 Requirement 索引

- REQ-DEV-001(1 WorkItem → N Worktree)
- §21 全章
- §21.1(ChangeSet ≠ Git Diff)
- §21.2(Symbol-aware Context)

---

### 4.9 domain-work-item + workflow + board + planning(Work Management Core)

#### 4.9.1 职责与定位

Work Management Core 承担 Jira-class 闭环(§30.1)。本节合并描述,因为四者共享数据模型。

#### 4.9.2 关键实体

**WorkItem**(聚合根,§8.1):

- `work_item_id`, `tenant_id`, `workspace_id`, `project_id`
- `type`:WorkItemType(Epic / Story / Task / Bug / Subtask / AITask)
- `title`, `description`
- `status`(由 Workflow 决定,默认三态)
- `assignee_user_id`, `assignee_agent_id`
- `reporter_user_id`
- `priority`, `severity`
- `story_points`(可选)
- `sprint_id`(可选)
- `parent_work_item_id`(Epic / Story / Subtask 关系)
- `requirement_ids[]`, `acceptance_criteria_ids[]`
- `repository_ids[]`(0..N)
- `worktree_ids[]`(0..N)
- `labels[]`, `components[]`
- `created_at`, `updated_at`, `due_date`

**AITask 子类型字段**(§8.1,§27):

- `objective`
- `repository_scope`, `allowed_files[]`, `forbidden_files[]`
- `agent_policy_id`
- `validation_policy_id`
- `context_policy_id`
- `acceptance_criteria_ids[]`

**Requirement**(§39 Traceability):

- `requirement_id`, `tenant_id`, `business_goal_id`
- `statement`, `rationale`
- `linked_work_item_ids[]`

**AcceptanceCriterion**:

- `acceptance_criterion_id`, `requirement_id`, `work_item_id`
- `statement`
- `coverage_status`(由 Validation 写入)
- `covered_by_validation_ids[]`

**Workflow / State / Transition**(§8.2):

- `workflow_id`, `project_id`
- `states[]`, `transitions[]`(from, to, required_permission)
- 默认最简三态:TODO → IN_PROGRESS → DONE(REQ-WF-001)

**Board / Column / Swimlane**(§9,REQ-PLAN-003):

- `board_id`, `project_id`, `board_type`(Kanban / Scrum)
- `columns[]`(state_id → order)
- `swimlanes[]`(group_by 字段)

**Sprint / Backlog / Roadmap**(§9):

- `sprint_id`, `project_id`, `name`, `goal`
- `start_at`, `end_at`
- `work_item_ids[]`
- `state`(Planning / Active / Closed)

#### 4.9.3 状态机(WorkItem 默认,§8.2 REQ-WF-001)

> **默认最简三态**(REQ-WF-001 强约束,不属于 MVP 范围裁剪):

```text
TODO → IN_PROGRESS → DONE
```

**Project Policy 自定义扩展示例**(非默认,以下为常见项目可选项):

- `IN_REVIEW`:在 IN_PROGRESS 与 DONE 之间的显式审查阶段
- `BLOCKED`:WorkItem 因依赖/外部因素被阻塞,可由 IN_PROGRESS 转入,解除后回 IN_PROGRESS
- `CANCELLED`:任意状态均可转入(终态)
- `IN_TESTING`, `READY_FOR_DEPLOY`, `NEEDS_INFO` 等

**与 Worktree 状态的独立性**(REQ-WF-002):WorkItem.status = IN_PROGRESS 时,其下 Worktree A 可为 AGENT_RUNNING,Worktree B 可为 BLOCKED,Worktree C 可为 REVIEWING。

#### 4.9.4 Sprint / Backlog / Gantt / Burndown 关系(§9,REQ-PLAN-001~005)

```text
Project
  ├── Backlog (排序池,无时间盒)
  ├── Sprint 1 (时间盒: 2026-08-25 → 2026-09-08)
  │     ├── WorkItem A
  │     ├── WorkItem B
  │     └── WorkItem C
  └── Gantt (Project 全局排期视图,跨 Sprint)
        ├── WorkItem A: 2026-08-25 → 2026-08-30
        └── WorkItem B: 2026-08-28 → 2026-09-05
```

**Burndown**(§9,REQ-PLAN-005):最小必需图表。Sprint 内剩余 Story Points / WorkItem 数随时间变化。
**Velocity / CFD / Control Chart**:V1(§30.3)。

**Gantt 与 Board 共享数据**:Gantt 是 Board 的排期视图变体,**不**做独立子系统(§9,REQ-PLAN-004,决策表 F.10)。

#### 4.9.5 AI Task(§8.1,§27)

AI Task 是"预计主要由 Coding Agent 执行、但受人类需求和 Acceptance Criteria 控制的开发工作单元"。

**特有字段**(在 WorkItem 基础上):

- `objective`
- `repository_scope`(必须)
- `allowed_files[]`, `forbidden_files[]`
- `agent_policy_id`(关联 AgentPolicy 模板)
- `validation_policy_id`
- `context_policy_id`

**创建流程**:必须先有 Repository Link + Agent Policy 模板 + Validation Policy,否则拒绝创建。

#### 4.9.6 Requirement 索引

- REQ-TWP-001~003
- REQ-WF-001/002
- REQ-PLAN-001~006
- §7,§8,§9 全章
- 决策表 F.1~10(其中 F.9, F.10 属本 Domain)

---

### 4.10 Permission & Security(横切能力 + 安全威胁模型)

#### 4.10.1 职责与定位

Permission 是横切 Domain,所有其他 Domain 都受其约束。Security 边界覆盖 §16、§23.2、§28.3、§34 全章。

#### 4.10.2 关键实体

**Role**:

- `role_id`, `tenant_id`, `name`, `permissions[]`

**Permission**:

- 形如 `work_item:read`, `worktree:create`, `agent_session:start`, `feedback:create`, `scm:push`, `validation:override`, `local_runtime:register` 等

**PermissionScheme**:

- `permission_scheme_id`, `project_id`
- `role_assignments[]`(user_id / group_id / device_id → role_id)
- `agent_role_assignments[]`(agent_id → role_id,**强制**)

**UserBinding / DeviceBinding / ProjectBinding**:

- §23.2 要求的三重绑定

**SecurityPolicy**(值对象,§16 REQ-SEC-002):

- `cloud_ai_allowed`(bool)
- `cloud_ai_restricted`(bool)
- `local_ai_only`(bool)
- `specific_provider_allowed[]`
- `no_code_upload`(bool)
- `metadata_only`(bool)

**ProviderDataBoundary**(§16 REQ-SEC-003):

- `provider_id`, `model_id`, `region`
- `data_sent`(List[DataCategory]: Prompt / Code / Diff / Symbol / Test / BuildLog)
- `retention_policy`(RetentionPolicy: Zero / N_Days / UntilTaskEnd)
- `credential_ref`(引用 Credential Broker,不存明文)
- `tenant_policy_id`, `project_policy_id`

#### 4.10.3 Permission 强制点(§11,REQ-PERM-002)

**所有 Permission 检查在 Application 层强制**(不是 Domain,不是 UI):

```rust
// crates/application/src/authz.rs (示意,非完整)
pub trait AuthorizationChecker {
    fn check(&self, actor: &ActorContext, action: &Action, resource: &Resource) -> Result<(), AuthzError>;
}

pub struct ActorContext {
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub roles: Vec<RoleId>,
}
```

**强制覆盖**:

- WorkItem CRUD
- Worktree 创建 / 分配 / 状态变更
- AgentSession 启动 / 停止
- Feedback 创建 / 解决
- Context Packet 触发
- Validation Override
- SCM Push / Merge
- PermissionScheme 修改
- Local Runtime 注册

**严禁**:仅通过 Prompt 告诉 Agent "不要修改 xxx"(§11)。

#### 4.10.4 Tenant Isolation 扩展边界(§16,REQ-SEC-001)

> 任何遗漏 `tenant_id` 或等效隔离边界都可能造成严重数据泄漏(§16,§91)。

**强制 tenant_id 携带的对象**(13 项):

| # | 对象 | 强制位置 |
|---|---|---|
| 1 | Repository Credential | domain-scm + application 鉴权 |
| 2 | Local Runtime | domain-local-runtime + domain-identity |
| 3 | Worktree | domain-worktree |
| 4 | AgentSession | domain-agent |
| 5 | ContextPacket | domain-context |
| 6 | Feedback | domain-feedback |
| 7 | AI Prompt | Agent Adapter 入参 + 审计 |
| 8 | AI Response | Agent Adapter 出参 + 审计 |
| 9 | Diff | domain-development(Object Storage Key 含 tenant_id) |
| 10 | Build Log | domain-validation(Object Storage Key 含 tenant_id) |
| 11 | Test Log | domain-validation(Object Storage Key 含 tenant_id) |
| 12 | PR Content | domain-scm |
| 13 | Symbol Index | domain-context 的 Symbol 投影 |

**每条对象**都必须在 Application 层调用 `AuthorizationChecker` 验证 `actor.tenant_id == resource.tenant_id`。

#### 4.10.5 企业私有代码 Policy(§16 REQ-SEC-002,§92)

支持的 Policy 级别:

```text
- Cloud AI Allowed
- Cloud AI Restricted
- Local AI Only
- Specific Provider Allowed
- No Code Upload
- Metadata Only
```

**强制点**:

- Context Compiler:根据 Policy 决定是否上传 Code/Diff 到 AI Provider
- Agent Adapter:发送请求前检查 Provider 是否在 Allowed 列表
- ProviderDataBoundary:每个 Provider 独立配置(§16 REQ-SEC-003,§93)

#### 4.10.6 Threat Model 威胁列表(§34,§73)

| # | 威胁 | 缓解 |
|---|---|---|
| 1 | Malicious Repository Prompt Injection | Untrusted Content 与 Trusted Human Policy 优先级分离(§28.3) |
| 2 | Agent Unauthorized File Access | AgentPolicy.allowed_paths + Local Runtime Filesystem Scope(§23.2) |
| 3 | Agent Unauthorized Command Execution | Local Runtime Command 白名单(§23.2) |
| 4 | Agent Credential Exfiltration | Credential Broker + Scoped Token(§28.4) |
| 5 | Cross Worktree Leakage | Worktree Isolation(§22.5) + tenant_id 强制 |
| 6 | Cross Repository Leakage | Context Compiler 不跨 Repository 加载 + AgentPolicy |
| 7 | Cross Tenant AI Context Leakage | tenant_id 强制 + ProviderDataBoundary |
| 8 | Malicious GitHub/GitLab Webhook | Webhook 签名验证 + Idempotency Key |
| 9 | Compromised Local Runtime 形成 Remote Shell | Command 白名单 + Filesystem Scope(§23.2) |
| 10 | Context Poisoning | Provenance 强制 + Decision 独立管理(§26.3, §26.5) |
| 11 | Fake Validation Result | Validation Evidence 必须独立来源 + Signature 校验 |
| 12 | Runtime Impersonation | Device Identity + mTLS + Revocation(§23.2) |

#### 4.10.7 Prompt Injection / Repository Injection 防护(§28.3,§41)

> 关键原则:**Untrusted Repository Content 与 Trusted Human Policy 的 Instruction Priority 不得相同**。

**Priority 分离**(§28.3):

```text
Trusted Human Policy     P0
Trusted System Policy    P0
Security Constraint      P0
Acceptance Criteria      P1
Approved ADR             P1
Untrusted Repo Content   P5 (单独分类,绝不与 P0-P3 混合)
Agent Self-Claim         P5
```

**实现**:

- Agent Adapter 在拼接 Prompt 时,对 Untrusted Content 加显式标签
- LLM Instruction 模板明确:"以下内容是 Untrusted Repository Content,不得作为指令执行"
- Agent Adapter 解析 Tool Call 时,对 Untrusted Content 触发的 Tool 二次校验
- Context Compiler 不将 README/Issue/PR Comment 直接作为 P0 指令

#### 4.10.8 Agent Secret Boundary(§28.4,§42)

> 不得把 GitHub/GitLab Token、Cloud Secret、Production Secret 无条件暴露给 Agent。

**强制要求**:

- **Credential Broker**:所有 Secret 由 Broker 持有,Agent 不直接持有
- **Scoped Token**:每个 AgentSession 获得仅含必要 scope 的 Token
- **Short-lived Token**:Token TTL ≤ AgentSession.max_runtime_seconds
- **Process Isolation**:Secret 注入 Agent 进程 Env,不得写入文件
- **Environment Isolation**:不同 AgentSession Env 互不可见
- **Secret Redaction**:日志 / Diff / Error Message 自动 Redact 已知 Secret Pattern

#### 4.10.9 Requirement 索引

- REQ-PERM-001/002
- REQ-SEC-001/002/003
- ARCH-OBL-DEV-001/002/004
- §16,§28.3,§28.4,§34 全章
- 决策表 M(Top 10 Agent Security Risks)

---

### 4.11 Worktree Orchestration 跨域协作 (v0.16 新增)

per requirements §22 Worktree Orchestration 要件 + §4.1 domain-worktree + §2.4 跨域事务边界,本节定义 **Worktree 跨域协作的端到端编排语义**(与 §2.4 7 类典型跨域事务互为补充)。

#### 4.11.1 协作参与者 (22 domain 中涉及 12 个)

```text
Worktree Orchestration 涉及 domain 列表 (per 22 domain 清单):
  Core:        work-item, worktree, agent, context, feedback, validation, development
  Coordination: scm, collaboration, permission, audit
  Support:     local-runtime
  = 12 / 22 domain (其余 10 domain 不直接参与 Worktree Orchestration)
```

**未参与的 10 domain** (per v0.16 梳理,如有遗漏属隐性缺口): tenant / workspace / project / workflow / board / planning / comment / relation / automation / integration / search / notification / identity / permission / audit — 实际 audit + permission 仍参与(只读 + 强制),共 5 个未直接参与(workflow/board/planning/comment/relation + 4 个 support = 9 个)。

#### 4.11.2 协作时序 (8 步编排,per saga spec v0.2 §4)

```text
T0  user   ──── SubmitWorkItem ────▶ work-item
T1  work-item ── StateChanged(IN_PROGRESS) ──▶ Outbox
T2  application 读 Outbox ──▶ 触发 Worktree Orchestration Saga:
    1. ValidateWorkItemOwnership  (IdentityValidation, domain-work-item)
    2. CreateWorktree             (ResourceMutation,    domain-worktree)
    3. RegisterAgentSession       (ResourceMutation,    domain-agent)
    4. StartContextBuild          (StateObservation,    domain-context)
    5. AuthorizeFeedbackGate      (DecisionAuthorization, domain-feedback)
    6. TriggerValidation          (ResourceMutation,    domain-validation)
    7. LinkPullRequest            (ResourceMutation,    domain-scm)
    8. WriteAuditLog              (AuditLogging,        domain-audit)  -- 必填且最后
T3  Realtime 推送 (per §4.13) ──▶ collaboration ──▶ star-sse ──▶ user UI
T4  Notification 推送 (per REQ-NOTIF-002 降噪) ──▶ notification ──▶ inbox/email
```

#### 4.11.3 协作原则 (5 条)

1. **状态独立**: Worktree Status 与 WorkItem Status 解耦 (per REQ-WF-002, §4.1.3 状态机)
2. **Observed vs Business 分离**: Worktree.observed_state 不入核心事务 (per REQ-DATA-003, §14.1)
3. **强一致走单事务,跨域走 Saga**: 涉及多 domain 写用 Saga 编排,单 domain 写用 PG 事务 (per §2.4)
4. **审计 Append-only**: 任何 Worktree 状态变化全量入 domain-audit,不可删改 (per REQ-AUDIT-001)
5. **Saga 失败必补偿**: 8 步任一失败触发逆向补偿,best-effort,失败入死信 (per saga spec §5 Compensating 状态)

#### 4.11.4 与 Saga spec v0.2 对应

per spec/saga/01-saga-coordination-spec.md v0.2 §4 Worktree Orchestration Saga 示例 (8 步 + 逆向补偿表),本节 §4.11 是 Saga 8 步流程在 Worktree Orchestration 场景的协作视角展开,二者 1:1 对应。

---

### 4.12 Event Bus 协作机制 (v0.16 新增)

per requirements §14.1 Event Architecture 12 核心事件 + §3.1 Domain Event (NATS JetStream),本节定义 22 domain 间 Event 协作的 **事件契约 + 订阅矩阵**。

#### 4.12.1 12 核心事件契约 (per requirements §14.1)

| 事件 | 源 domain | 投递目标 | 触发条件 | payload 必填 |
|---|---|---|---|---|
| `WorktreeCreated` | worktree | application + worker context-build + sse push | worktree 首次创建成功 | worktree_id, tenant_id, work_item_id |
| `WorktreeAssigned` | worktree | application + worker projection + notification | worktree 分配给 user/agent | worktree_id, assignee_id |
| `WorktreeStatusObserved` | worktree | worker projection + sse push (高频) | Local Runtime 上报 observed_state | worktree_id, observed_state(快照) |
| `WorktreeDirtyStateChanged` | worktree | worker projection + sse push | dirty=true/false 切换 | worktree_id, dirty, changed_files_count |
| `WorktreeConflictDetected` | worktree + relation | notification + sse push + audit | 跨 worktree 文件冲突 | worktree_id, conflict_worktree_ids[] |
| `AgentSessionStarted` | agent | application + worker context-build + sse push | agent 进程 spawn 成功 | agent_session_id, worktree_id, agent_id |
| `AgentSessionCompleted` | agent | application + worker validation-trigger + audit | agent 退出(成功) | agent_session_id, worktree_id, completion_status |
| `AgentSessionFailed` | agent | notification + audit | agent 退出(失败) | agent_session_id, worktree_id, error |
| `ChangeSetObserved` | development | worker context-build + validation-trigger | 新的 ChangeSet 落盘 | change_set_id, worktree_id, files[] |
| `FeedbackCreated` | feedback | context (re-compile trigger) + notification + sse push | user 提交 Feedback | feedback_id, target_type, target_id |
| `FeedbackAcknowledged` | feedback | context (state refresh) + sse push | agent consume Feedback | feedback_id, agent_session_id |
| `FeedbackApplied` | feedback | worktree (re-validate trigger) + audit | user 验证 Feedback 已应用 | feedback_id, change_set_id |
| `FeedbackVerified` | feedback | work-item (state gate) + audit | user 验证 Feedback 完成 | feedback_id, verified_by |
| `ValidationStarted` | validation | sse push + audit | 触发 validation 流程 | validation_id, work_item_id |
| `ValidationPassed` | validation | work-item (state gate) + sse push + notification | validation 全部通过 | validation_id, work_item_id |
| `ValidationFailed` | validation | feedback (auto-generate) + sse push + notification | validation 失败 | validation_id, failure_summary |
| `ContextPacketCreated` | context | agent (load) + audit | Context Compiler 产出新 packet | context_packet_id, worktree_id, token_budget |
| `PullRequestLinked` | scm | work-item (state gate) + sse push | PR/MR 创建成功 | pull_request_id, worktree_id, scm_url |
| `MergeRequestLinked` | scm | work-item (state gate) + sse push | MR 创建成功 | merge_request_id, worktree_id, scm_url |

**事件命名规范** (per §3.1 Published Language):
- 格式: `<Entity><PastTenseAction>` (如 `WorktreeCreated`)
- 来源: 必须含 `tenant_id` (per REQ-SEC-001)
- 不可变: payload schema 演进走 CloudEvents 1.0 backward-compatible 规则

#### 4.12.2 事件订阅矩阵 (5 类订阅者)

| 订阅者 | 订阅事件 | 用途 | 触达要求 |
|---|---|---|---|
| `worker context-build` | WorktreeCreated, AgentSessionStarted, ChangeSetObserved, FeedbackCreated | 异步构建 Context Packet | 异步,at-least-once |
| `worker projection` | WorktreeStatusObserved, WorktreeDirtyStateChanged | 写 Search Index / Projection | 异步,best-effort |
| `worker validation-trigger` | AgentSessionCompleted, ChangeSetObserved, FeedbackApplied | 触发 Validation 流程 | 异步,at-least-once |
| `collaboration + star-sse` | 全部 19 事件 | Realtime 推送 UI | 实时,push 模式 |
| `notification` | WorktreeConflictDetected, AgentSessionFailed, FeedbackCreated, ValidationFailed, PullRequestLinked, MergeRequestLinked | 触发 Inbox/Email/IM | 异步,降噪 (per REQ-NOTIF-002) |

#### 4.12.3 事件总线守门 (5 条)

1. **不得拆核心业务事务为 Event Chain** (per requirements §14.1, §2.4 7 类跨域事务) — 跨域写走 Saga,Event 只做异步解耦
2. **Outbox Pattern** 保证事务一致性 (per requirements §13.1, Transactional Outbox): domain 写 PG 后立即写 outbox 表,worker 异步投递 NATS
3. **事件 payload 不含敏感 PII/Prompt/Code 全文** (per REQ-SEC-002, §17 AI Audit) — 大块内容用 object_storage_ref 引用
4. **死信队列** (per saga spec G-05): 3 次重试失败入 DLQ,需 ops 介入
5. **追溯链**: 每个事件必含 `event_id` (UUID) + `causation_id` (父事件) + `correlation_id` (per requirements §39 Traceability)

---

### 4.13 Realtime 协作机制 (v0.16 新增)

per requirements §15 Realtime 要求 + §4.12 Event Bus + star-sse crate,本节定义 22 domain 间 Realtime 协作的 **通道 + 降噪 + 心跳**。

#### 4.13.1 Realtime 通道架构

```text
domain events (NATS JetStream)
       │
       ▼
star-sse (Rust WebSocket 端点)            per star-sse/src/lib.rs
       │
       ├── /ws/feed  (高频 stream, agent token stream, raw diff)
       ├── /ws/notif (降噪, REQ-NOTIF-002 关键事件)
       └── /ws/admin (admin only, low freq)
       │
       ▼
   browser (SSE/WS client)
```

**3 通道分工** (per ADR-0027 §2 STAR IDE Gateway 3 通道衍生):
- `/ws/feed`: 高频 feed,只走当前选中 Worktree / WorktreeGroup,不全局广播
- `/ws/notif`: 降噪后关键事件,默认全部订阅,可基于 Watcher 列表扩展
- `/ws/admin`: 管理面,只给 Platform Admin / Tenant Admin 开放

#### 4.13.2 降噪策略 (per REQ-NOTIF-002)

默认**只推送需要人类决策的节点**:
- `WAITING_FEEDBACK` (per §4.12.1 FeedbackCreated)
- `ValidationFailed` (per §4.12.1 ValidationFailed)
- `ProtectedAction 待授权` (per ADR-0025 vendor adapter anti-contamination)

**不推送** (但仍 100% 写 AgentSession Transcript 供按需查阅,per INV-AGT-09):
- Agent 每一次工具调用
- 中间步骤 (LLM token stream)
- 临时 observed state (per §4.12.1 WorktreeStatusObserved 走 /ws/feed,不进 /ws/notif)

**Watcher 覆盖** (per REQ-NOTIF-003): 用户加 Watcher 后即使不满足降噪触发条件也收关键事件。

#### 4.13.3 心跳与重连

- 客户端 30s 发 heartbeat (per ADR-0030 §3 11 字段对齐)
- 服务端 60s 无消息推 keep-alive frame
- 重连策略: exponential backoff (1s, 2s, 4s, 8s, max 30s) + Last-Event-ID 续传 (per MCP Streamable HTTP D.6+)
- 断线期间事件: 不重放 (客户端需通过 REST 拉取最新 snapshot),只续传 Last-Event-ID 之后的事件

#### 4.13.4 与 §4.12 Event Bus 的边界

- Event Bus 是 domain 间异步通信 (NATS, 多订阅者)
- Realtime 是 user 端 push 通道 (WebSocket, 1:1 session)
- **不允许 Realtime 反推 domain 状态变更** (单向),Realtime 只读 Outbox / NATS,不改业务事实

---

## 5. 数据架构

### 5.1 System of Record 划分(§14,§58-60)

| 存储 | 用途 | 强制数据 |
|---|---|---|
| **PostgreSQL(SoR)** | 业务事实 | WorkItem, Requirement, AcceptanceCriterion, Worktree(注册), DevelopmentExecution, AgentSession(注册), Feedback, Decision, ContextPacket(元数据), ValidationResult(摘要), SCM Link, Permission, Notification, Audit, Comment, Relation, Sprint, Board |
| **Object Storage(S3 兼容)** | 大型 Raw / 二进制 / Transcript | Diff Artifact(>1MB), Build Log, Test Log, Agent Transcript(完整对话,需 AI Content Retention Policy 决定), Symbol Index Snapshot(>10MB), Agent Attachments |
| **Valkey(缓存)** | 临时缓存 | Session Token, Rate Limit, Realtime Subscription, Heatmap Snapshot, Search Query Cache |
| **NATS JetStream** | 异步事件流 | Domain Event(短生命周期), Webhook 缓冲(去重) |
| **Search Projection(独立索引,初版基于 PostgreSQL FTS)** | 全文检索 | WorkItem 全文, Comment 全文, Symbol 全文(V1 扩展) |

**REQ-DATA-002 边界**:Large Diff / Large Log / Build Artifact / Agent Transcript / Binary 评估 PostgreSQL vs Object Storage,不得把无限 Agent Transcript 塞入 PostgreSQL 热表(§14,§59)。

**判断标准(草案)**:

```text
> 1MB 或 > 10K 行 → 必走 Object Storage
含 Binary / Base64 → 必走 Object Storage
PostgreSQL 存储的对象 = 元数据 + 摘要 + 引用(ref)
```

### 5.2 Business Truth vs Observed State(§43.1,§97)

> 不得混为一个 "giant status JSON"(§43.1,§97)。

| 事实类型 | 定义 | 存储位置 | 写入频率 | 例子 |
|---|---|---|---|---|
| **Business Truth** | 业务事实,影响决策 | PostgreSQL | 低频,受事务约束 | WorkItem.status, Worktree.status(Business), Feedback.status, ValidationResult.status |
| **Observed Runtime State** | 高频本地状态,非业务事实 | Projection(独立表) | 高频,异步 | Worktree.dirty, Agent.process_pid, Test.progress(45/50) |
| **SCM Truth** | Git 远端事实 | SCM Adapter 镜像 + 引用 | 中频 | Commit/PR 最新状态 |
| **AI Suggestion** | AI 输出的中间建议 | AgentSession.context(不写业务) | 高频 | AgentPlan, ToolCall |
| **Human Feedback** | 人类修正指令 | PostgreSQL(Feedback) | 低频 | Feedback |
| **Validation Evidence** | 证明 AC 满足的证据 | PostgreSQL + Object Storage | 中频 | ValidationResult + Evidence |

**架构含义**:

- Observed State 走独立 Projection 表,**不**进入核心事务(§14.1)
- UI 读 Observed State 必须带 `last_observed_at`,显示 "Possibly Stale"(§23.4)
- Business Truth 与 Observed State 冲突时,以 Business Truth 为准(§43.2)

### 5.3 Event Bus 边界(§14.1,§58,§97)

> Event Bus 用于外围解耦,不得把核心业务事务拆成 Event Chain(§14.1,§58)。

**核心事务不拆 Event Chain**:

```text
错误:  WorkItemCreated Event → WorktreeCreated Event → AgentSessionCreated Event
正确:  Application Service 单事务创建 WorkItem + Worktree + AgentSession,Outbox 触发 3 个 Event
```

**Event Bus 用途**:

1. 跨进程解耦(如 worker projection role 订阅)
2. Webhook 缓冲(去重 + 重试)
3. 通知触发
4. Search Projection 更新

**Event Bus 不用途**:

- 核心业务事务编排
- 一致性补偿(用 Application 事务,不用 Eventual Consistency 兜底)
- 跨域数据传递(直接调用 Port)

### 5.4 Transactional Outbox(§13.1,§58)

```text
Application Service 事务
    ├── 写业务聚合
    ├── 写 outbox 表(同事务)
PG Transactional Outbox
    ├── Worker Polling(每 1s)
    └── 推送至 NATS JetStream
NATS JetStream
    ├── 持久化
    └── 订阅者异步消费
```

**Outbox 表字段**(不写 DDL,只列语义):

- `outbox_id`, `aggregate_type`, `aggregate_id`, `event_type`, `payload_json`, `created_at`, `published_at`, `retry_count`

**Outbox 强制规则**:

- 与业务聚合同事务写入(原子性)
- Worker Polling 推送至 NATS
- 推送成功后标记 `published_at`
- 失败重试(指数退避,最多 5 次)
- 超过重试次数进入 DLQ

### 5.5 NATS JetStream Subject 命名空间草案

```text
star.events.{domain}.{aggregate}.{action}
star.webhook.{provider}.{event_type}
star.worker.{role}.{command}
star.realtime.{tenant_id}.{project_id}.{entity}
star.dlq.{original_subject}
```

**示例**:

- `star.events.work_item.work_item.created`
- `star.events.worktree.worktree.status_observed`
- `star.events.agent.agent_session.started`
- `star.events.feedback.feedback.verified`
- `star.events.validation.validation_result.passed`
- `star.webhook.github.push`
- `star.webhook.gitlab.merge_request`
- `star.worker.projection.refresh_search_index`
- `star.realtime.{tenant_id}.{project_id}.worktree.status`

### 5.6 核心事件清单(§14.1)

```text
WorktreeCreated / WorktreeAssigned / WorktreeStatusObserved
WorktreeDirtyStateChanged / WorktreeConflictDetected
AgentSessionStarted / AgentSessionCompleted / AgentSessionFailed
ChangeSetObserved
FeedbackCreated / FeedbackAcknowledged / FeedbackApplied / FeedbackVerified
ValidationStarted / ValidationPassed / ValidationFailed
ContextPacketCreated
PullRequestLinked / MergeRequestLinked
AutomationRuleScheduleTriggered(V1 候选,REQ-AUTO-002,2026-08-26 补充)
```

每个事件都包含 `tenant_id`, `aggregate_id`, `version`, `occurred_at`, `actor`(用户/Agent/系统), `payload`(JSON Schema 描述)。

### 5.7 主要聚合根与不变量(表格)

| 聚合根 | 必带 tenant_id | 跨域事务 | 核心不变量 |
|---|---|---|---|
| WorkItem | 是 | work-item + workflow + project + permission + audit(单事务) | type ∈ {Epic, Story, Task, Bug, Subtask, AITask}; 0/1/N Repository; 0/1/N Worktree |
| Worktree | 是 | worktree + work-item(读) + scm + development + audit | status 独立于 WorkItem.status; 必绑 Runtime |
| AgentSession | 是 | agent + worktree + context + audit | 1 Active Worktree; 必带 policy 校验 |
| ChangeSet | 是 | development + worktree + validation | 1 Commit; 必带 risk_signals 摘要 |
| ContextPacket | 是 | context + work-item + worktree + feedback + validation | 必带 provenance; 不可生成无 provenance 的 packet |
| Feedback | 是 | feedback + work-item(读) + audit | target 必可解析; status 转换必审计 |
| ValidationResult | 是 | validation + worktree + change-set | kind ∈ 已知集合; 不可缺失 evidence_ref |
| DevelopmentExecution | 是 | development + work-item + 多个子聚合 | 1 WorkItem; 0..N Worktree; 0..N AgentSession |
| Decision | 是 | context + audit | status ∈ {Active, Superseded, Invalidated}; superseded 必带 successor |
| PullRequest | 是 | scm + worktree(可选) | 1 Repository; 0..1 WorkItem Link |

**强制规则**:

- 每个聚合根的 `INSERT` / `UPDATE` / `DELETE` 必须带 `tenant_id`
- `tenant_id` 由 Application 层从 `ActorContext` 注入,Domain 层不信任调用方传入(由 Port 实现校验)
- 跨域事务在 `application` crate 中,不在 domain 层

### 5.8 数据生命周期与归档

| 数据 | 保留期 | 归档策略 |
|---|---|---|
| WorkItem / Comment / Feedback | 永久(直到 Tenant 显式删除) | 不归档 |
| AgentSession Transcript | AI Content Retention Policy 决定(§40,§28.2) | 默认 90 天,Project Policy 可调整 |
| Observed State | 30 天热数据,冷数据归档 | 周级别 Partition |
| Audit Log | 7 年(企业级) | 月级别 Partition |
| Object Storage 大文件 | 1 年 | Lifecycle Policy |
| Search Projection | 7 天滞后 SoR | 增量重建 |
| Webhook 事件 | 30 天 | 失败重试后丢弃 |

---

## 6. 安全边界

### 6.1 §16 Tenant Isolation 扩展边界(REQ-SEC-001,§91)

> 任何遗漏 `tenant_id` 或等效隔离边界都可能造成严重数据泄漏(§16,§91)。

**13 类对象必带 tenant_id 隔离**(继承 §16):

| # | 对象 | 隔离方式 | 强制检查点 |
|---|---|---|---|
| 1 | Repository Credential | tenant_id 索引 + Encryption at Rest | domain-scm / application |
| 2 | Local Runtime | tenant_id + user_id + project_id 三重绑定 | domain-identity |
| 3 | Worktree | tenant_id | domain-worktree |
| 4 | AgentSession | tenant_id | domain-agent |
| 5 | ContextPacket | tenant_id + provenance 强制 | domain-context |
| 6 | Feedback | tenant_id | domain-feedback |
| 7 | AI Prompt | tenant_id + 加密落盘 | Agent Adapter + Audit |
| 8 | AI Response | tenant_id + 加密落盘 | Agent Adapter + Audit |
| 9 | Diff | Object Storage Key 含 tenant_id | domain-development |
| 10 | Build Log | Object Storage Key 含 tenant_id | domain-validation |
| 11 | Test Log | Object Storage Key 含 tenant_id | domain-validation |
| 12 | PR Content | tenant_id | domain-scm |
| 13 | Symbol Index | tenant_id | domain-context |

**实现机制**:

```text
1. PostgreSQL:  每张表必有 tenant_id 列 + 复合索引
2. Row Level Security(RLS): PostgreSQL RLS 强制 tenant_id 匹配 session 变量
3. Application:  AuthorizationChecker 在每个 Query 之前检查
4. Object Storage: Bucket/Key 前缀含 tenant_id,Policy 限制跨租户访问
5. NATS Subject: star.events.{tenant_id}.{...} 命名空间隔离
6. Audit:      每个跨租户访问尝试都记录
```

### 6.2 Local Runtime Security Boundary(§23.2,LRT-001/002)

详见 §4.6.3。核心要素:

- Device Identity + Registration
- User / Tenant / Project Binding
- Short-lived Credential(1h mTLS, 5min Command Token)
- Mutual Authentication(mTLS)
- Command Authorization(白名单)
- Command Scope(Repository/Worktree/Path 范围)
- Filesystem Scope(syscall 拦截)
- Process Scope(子进程监控)
- Secret Isolation(Credential Broker)
- Agent Credential Isolation(OS-level 隔离)
- Audit(所有命令/上报)
- Revocation(黑名单)
- Remote Disable(强制停机)

**严禁出现的能力**:

- `ExecuteArbitraryShell(cmd: String)`
- `ReadArbitraryFile(path: String)`
- `WriteArbitraryFile(path: String, content: String)`
- 任何 `*` 范围的命令

### 6.3 默认禁止 SaaS Server → Arbitrary Shell(§20,§23.2)

**允许的有限能力接口**(白名单):

```text
GitStatus / CreateWorktree / ReadDiff / RunApprovedTest
QueryAgentStatus / SubmitFeedback / StartAuthorizedAgentSession
StopAgentSession / RegisterLocalRuntime / Heartbeat
ReportObservation(WorktreeStatus / AgentSessionStatus / Build / Test / DiffAvailable)
```

**每个接口都必带**:

- 必带 `worktree_id` / `agent_session_id` / `repository_id` 范围
- 必带 `command_token`(短时,5min TTL)
- 必带 mTLS 设备身份
- 必带 `actor` 标识(用户/Agent/系统)
- 必写 Audit Log

### 6.4 Agent Secret Boundary(§28.4,§42)

详见 §4.10.8。要点:

- Credential Broker 持有所有 Secret
- Scoped Token(每个 AgentSession 独立 scope)
- Short-lived Token(TTL ≤ max_runtime_seconds)
- Process Isolation(Env 注入,不写文件)
- Environment Isolation(进程间 Env 互不可见)
- Secret Redaction(日志/Diff 自动 Redact)

### 6.5 Prompt Injection / Repository Injection 防护(§28.3,§41)

详见 §4.10.7。要点:

- Untrusted Content(P5)与 Trusted Human Policy(P0)优先级分离
- Agent Adapter 拼接 Prompt 时加显式标签
- LLM Instruction 模板明确"Untrusted 不得作为指令"
- Tool Call 二次校验
- Context Compiler 不把 README/Issue/PR Comment 作为 P0

### 6.6 Cross-Tenant / Cross-Repository / Cross-Worktree 防护(§34,§91)

**Cross-Tenant**:PostgreSQL RLS + AuthorizationChecker + Object Storage Policy

**Cross-Repository**:

- Context Compiler 不跨 Repository 加载(同 Repo 内可跨 Module)
- AgentPolicy 必带 `allowed_repositories[]`
- Agent 改文件前 Local Runtime 校验 Repository ID

**Cross-Worktree**:

- Worktree Isolation(§22.5):Filesystem / Env / Process / Port 隔离
- Agent 进程不读其他 Worktree 的 `local_path_reference`
- Context Compiler 不跨 Worktree 加载(除非显式 Aggregate)

### 6.7 AI Audit Metadata(§17,§28.2,§40)

**Audit 必须能回答的问题**(§17 REQ-AUDIT-002):

```text
谁要求 AI 做什么?
AI 使用了什么 Context?
AI 修改了什么?
哪个 Agent 执行?
在哪个 Worktree?
什么时间?
哪些验证通过?
哪些 Feedback 被消费?
谁批准 Commit / PR / Merge?
```

**AuditEvent 字段**(语义):

- `audit_id`, `tenant_id`, `actor`(user/agent/system), `action`
- `resource_type`, `resource_id`
- `before_state`(可选), `after_state`(可选)
- `context_refs[]`(Provenance 引用)
- `ai_metadata`:AIAuditMetadata
  - `agent_session_id`
  - `context_packet_id`
  - `change_set_id`
  - `validation_result_ids[]`
  - `feedback_consumed_ids[]`
  - `approver_user_id`(Commit/PR/Merge 时)

**敏感 Prompt/Code** 不默认进入普通 Audit Log;走独立 `AIAuditMetadata` 表 + AI Content Retention Policy(§40)。

### 6.8 AI Content Retention Policy 草案(§28.2,§40)

> 敏感 Prompt/Code 需单独的 AI Audit Metadata 与 AI Content Retention Policy(§40)。

**分级草案**:

| 级别 | 包含 | 默认保留期 | Project 可配置 |
|---|---|---|---|
| **Metadata** | agent_session_id, context_packet_id, change_set_id, decision_id | 永久 | 否 |
| **Summary** | intent, result_summary, decision 摘要 | 1 年 | 是 |
| **Full Prompt** | 完整 Prompt 输入 | 90 天 | 是(0~365 天) |
| **Full Response** | 完整 Response 输出 | 90 天 | 是(0~365 天) |
| **Tool Call Trace** | Tool 名称,参数摘要 | 1 年 | 是 |
| **Code Diff** | 完整 Diff | 1 年(与 ChangeSet 同) | 否 |
| **Sensitive Code** | 包含 Secret / PII 的代码片段 | 0 天(不存) | 否 |

**强制**:

- Full Prompt/Response 默认 90 天
- Sensitive Code(经 Secret Scanner 检测)立即 Redact
- Project Admin 可调整 Summary / Prompt / Response 保留期
- 超过保留期物理删除(非软删除)

---

## 7. 关键状态机

### 7.1 Worktree 生命周期(§22.2)

```text
CREATED → READY → ASSIGNED → AGENT_RUNNING
       → WAITING_FEEDBACK → FEEDBACK_RECEIVED
       → VALIDATING
       → BLOCKED / CONFLICTED
       → READY_FOR_REVIEW → REVIEWING
       → READY_FOR_COMMIT → COMMITTED
       → PR_OPEN → MERGED
       → ABANDONED → ARCHIVED
```

**完整状态机见附录 A.1**。

| 状态迁移 | 触发者 | 迁移条件 |
|---|---|---|
| CREATED → READY | Local Runtime | Worktree 路径创建成功,Git 初始化完成 |
| READY → ASSIGNED | User / Application | 分配 AgentSession |
| ASSIGNED → AGENT_RUNNING | Local Runtime | Agent Process 启动成功 |
| AGENT_RUNNING → WAITING_FEEDBACK | Application | OpenFeedback 创建且与本 Worktree 关联 |
| WAITING_FEEDBACK → FEEDBACK_RECEIVED | Application | Feedback 状态 = APPLIED |
| AGENT_RUNNING → VALIDATING | Application | AgentSession.ended_at + is_ai_complete_claim |
| VALIDATING → READY_FOR_REVIEW | Application | §4.1.9 七项检查全通过 |
| VALIDATING → BLOCKED | Application | 关键 Validation Failed |
| * → CONFLICTED | Worktree Conflict Detector | 检测到 File-level Conflict |
| CONFLICTED → ASSIGNED | User | 冲突已解决 |
| * → ABANDONED | User | 显式放弃 |
| ABANDONED → ARCHIVED | Worker maintenance | 90 天后自动归档 |
| READY_FOR_COMMIT → COMMITTED | Application | Commit 成功 |
| COMMITTED → PR_OPEN | SCM Adapter | PR 创建成功 |
| PR_OPEN → MERGED | SCM Webhook | PR Merged 事件 |
| MERGED → ARCHIVED | Worker maintenance | 30 天后自动归档 |

### 7.2 WorkItem Workflow(§8.2,REQ-WF-001/002)

**默认最简三态**(REQ-WF-001 强约束,不属于 MVP 范围裁剪):

```text
TODO → IN_PROGRESS → DONE
```

**Project Policy 自定义扩展示例**(非默认):

- `IN_REVIEW`(在 IN_PROGRESS 与 DONE 之间)
- `BLOCKED`(可由 IN_PROGRESS 转入,解除后回 IN_PROGRESS)
- `CANCELLED`(任意状态均可转入,终态)
- `IN_TESTING`, `READY_FOR_DEPLOY`, `NEEDS_INFO` 等

**与 Worktree Status 独立性**(REQ-WF-002,§4):

```text
WorkItem = IN_PROGRESS
├── Worktree A: AGENT_RUNNING
├── Worktree B: BLOCKED
└── Worktree C: REVIEWING
```

合法:**所有组合**。

### 7.3 Feedback 状态机(§25.3)

```text
OPEN
  ↓
ACKNOWLEDGED  ← (Agent 拉取并加入 Context Packet)
  ↓
APPLIED       ← (Agent 提交含该 Target 的 ChangeSet)
  ↓
VERIFIED      ← (Validation 跑过对应 AC)
  
任意状态 → REJECTED    (用户明确拒绝)
任意状态 → SUPERSEDED  (被新 Feedback 取代,新 Feedback 必带 predecessor_id)
```

**触发者**:

- `OPEN → ACKNOWLEDGED`:Application(AgentSession 启动时拉取 Feedback)
- `ACKNOWLEDGED → APPLIED`:Application(ChangeSet 提交,自动匹配 Target)
- `APPLIED → VERIFIED`:Application(ValidationResult 通过对应 AC)
- `OPEN/ACKNOWLEDGED → REJECTED`:User
- `任意 → SUPERSEDED`:User / Application(创建新 Feedback 显式 Supersede)

### 7.4 AgentSession 状态机(§24.1)

```text
CREATED → STARTING → RUNNING
       → WAITING_TOOL → TOOL_RUNNING → TOOL_COMPLETED → RUNNING
       → WAITING_FEEDBACK → FEEDBACK_RECEIVED → RUNNING
       → VALIDATING
       → COMPLETED
       → FAILED
       → ABORTED
       → CRASHED
       → TIMEOUT
```

**触发者**:

- `CREATED → STARTING`:Application
- `STARTING → RUNNING`:Local Runtime(Agent Process 启动成功)
- `RUNNING → WAITING_TOOL`:Agent Adapter 检测到 Tool Call
- `WAITING_TOOL → TOOL_RUNNING`:Local Runtime 启动 Tool
- `TOOL_RUNNING → TOOL_COMPLETED`:Local Runtime Tool 完成
- `TOOL_COMPLETED → RUNNING`:Agent Adapter 继续
- `RUNNING → WAITING_FEEDBACK`:Application(OpenFeedback 触发)
- `WAITING_FEEDBACK → FEEDBACK_RECEIVED`:Application(Feedback 提交)
- `RUNNING → VALIDATING`:Application(AgentSession.ended_at + is_ai_complete_claim)
- `VALIDATING → COMPLETED`:Application(§4.5.5 链全通过)
- `VALIDATING → FAILED`:Application(关键 Validation 失败)
- `* → ABORTED`:User / Application(Policy 拒绝)
- `* → CRASHED`:Local Runtime(进程异常退出)
- `* → TIMEOUT`:Worker(超过 max_runtime_seconds)

### 7.5 PR / MR 链接与合并状态(§18,§19)

```text
DRAFT → OPEN → REVIEWING
                ↓
            CHANGES_REQUESTED → OPEN(循环)
                ↓
            APPROVED
                ↓
            MERGEABLE → MERGED
                ↓
            CLOSED
```

**触发者**:

- `DRAFT → OPEN`:User / Application
- `OPEN → REVIEWING`:SCM Webhook(review requested)
- `REVIEWING → CHANGES_REQUESTED`:SCM Webhook(review submitted with changes_requested)
- `CHANGES_REQUESTED → OPEN`:User
- `REVIEWING → APPROVED`:SCM Webhook(review approved)
- `APPROVED → MERGEABLE`:SCM(CI Pass + Branch 同步)
- `MERGEABLE → MERGED`:User / Application(ProjectPolicy.merge_gate)
- `* → CLOSED`:User / SCM

### 7.6 状态机总览表

| 实体 | 状态数 | 触发者种类 | 见附录 |
|---|---|---|---|
| Worktree | 17 | 4 (SaaS / Local / Webhook / Human) | A.1 |
| WorkItem(默认 + 扩展) | 3 + 扩展 | 3 (User / System / Workflow) | A.2 |
| Feedback | 6 | 3 (User / Agent / Application) | A.3 |
| AgentSession | 14 | 4 (SaaS / Local / Agent / Timeout) | A.4 |
| ValidationResult | 6 | 2 (CI / Local) | A.5 |
| PullRequest | 7 | 2 (User / Webhook) | A.6 |
| Decision | 3 | 2 (User / System) | A.7 |

---

## 8. 部署与运行时拓扑

### 8.1 K3s 集群布局

```mermaid
flowchart TB
    subgraph Edge[Edge Layer]
        LB[Cloud LB / MetalLB]
    end

    subgraph GatewayNode[Gateway Node Pool]
        GW1[gateway-1]
        GW2[gateway-2]
    end

    subgraph IdentityNode[Identity Node Pool]
        ID1[identity-1]
        ID2[identity-2]
    end

    subgraph WorkCoreNode[Work Core Node Pool]
        WC1[work-core-1]
        WC2[work-core-2]
        WC3[work-core-3]
    end

    subgraph WorkerNode[Worker Node Pool]
        WK1[worker-1 --role all]
        WK2[worker-2 --role all]
        WK3[worker-3 --role all]
    end

    subgraph DataNode[Data Node Pool]
        PG[(PostgreSQL HA - Patroni)]
        NATS[(NATS JetStream Cluster)]
        VALK[(Valkey Sentinel)]
        OBJ[(Object Storage - S3 兼容)]
    end

    subgraph RealtimeNode[Realtime Node Pool - Optional V1]
        RT1[realtime-1]
        RT2[realtime-2]
    end

    LB --> GW1
    LB --> GW2
    GW1 --> ID1
    GW2 --> ID2
    GW1 --> WC1
    GW2 --> WC2
    GW1 --> WC3
    GW1 --> WK1
    GW2 --> WK2
    WC1 --> PG
    WC2 --> NATS
    WC1 --> VALK
    WC1 --> OBJ
    WK1 --> NATS
    WK2 --> PG
    WK3 --> NATS
    GW1 -.-> RT1
    GW2 -.-> RT2
```

**继承 §13.1**。关键约束:

- **第一阶段不部署 realtime**(§13.1,§15):仅在出现真实 Long Connection Scaling Boundary 时才拆
- **worker --role all**:九种角色合并(§13.4)
- **Local Runtime 不计入 Workload**:Developer Machine 与 K3s Cluster 平级

### 8.2 Service Promotion Model 在 K3s 下的具体含义

> Service Promotion Model(§13)指按真实负载证明拆分,而不是按名称拆分(§44.2,§86)。

**判定拆分条件**(§44.2):

1. 真实 CPU 压力 > 70% 持续 5 分钟
2. 独立 Scaling 需求(如 scm-sync 受 GitHub Rate Limit 制约)
3. 独立 Failure Boundary(如 repository-analysis OOM)
4. 独立 Security Boundary(如 Local Runtime 相关)
5. 独立 Runtime Boundary(如 Realtime WebSocket)
6. 独立 Ownership Boundary(如独立团队维护)

**当前第一阶段不拆**:

- `worktree-service`(domain-worktree 内聚于 work-core)
- `agent-service`(domain-agent 内聚于 work-core)
- `feedback-service`(domain-feedback 内聚于 work-core)
- `context-service`(domain-context 内聚于 work-core)
- `validation-service`(domain-validation 内聚于 work-core)
- `github-service`(SCM Adapter 内聚于 worker)
- `gitlab-service`(同上)

**未来可能拆**(需真实负载证明):

- `realtime-service`(V1,§30.3)
- `repository-analysis-service`(V1,§30.3)
- `context-build-service`(V1,§30.3)
- `local-runtime-gateway-service`(V1,§30.3)

### 8.3 Worker 角色分配(§13.4)

第一阶段:`worker --role all`,所有九种角色在同一进程内:

```rust
crates/worker/src/main.rs(示意)
fn main() {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let roles: Vec<Box<dyn WorkerRole>> = vec![
            Box::new(NotificationRole::new()),
            Box::new(WebhookRole::new()),
            Box::new(AutomationRole::new()),
            Box::new(ProjectionRole::new()),
            Box::new(IntegrationRole::new()),
            Box::new(MaintenanceRole::new()),
            Box::new(ScmSyncRole::new()),
            Box::new(ContextBuildRole::new()),
            Box::new(RepositoryAnalysisRole::new()),
        ];
        tokio::select! {
            _ = run_all(roles) => {},
            _ = signal::ctrl_c() => {},
        }
    });
}
```

**未来拆分**(§13.4):`worker --role repository-analysis`, `worker --role scm-sync` 等独立进程。

### 8.4 Serverless / KEDA 候选(§13.5,§89)

详见 §1.4。本节仅列引入策略:

| 任务 | KEDA 触发器 | Scale-to-Zero | 引入阶段 |
|---|---|---|---|
| Repository Analysis | NATS Queue Length | 是 | V1 评估 |
| Large Context Build | NATS Queue Length | 是 | V1 评估 |
| PR Analysis | NATS Queue Length | 是 | V1 评估 |
| Static Analysis | NATS Queue Length | 是 | V2(§30.4) |
| Agent Session Post-processing | NATS Queue Length | 是 | V2 |
| Diff Summarization | NATS Queue Length | 是 | V2 |
| Dependency Scan | Cron | 是 | V2 |

**判定原则**:不因 Vibe Coding 提前引入,Resource Saving vs Operational Complexity 明确对比后才引入(§13.5,§89)。

### 8.5 Local Runtime 不计入 K8s Workload(§23.1)

Local Runtime 是**外部进程**,运行于 Developer Machine / Self-hosted Runner / Cloud Workspace。其与 K3s Cluster 的关系是:

```text
Local Runtime  ──Secure Channel──> Gateway
        │                              │
        └→ 不在 K8s 内,不受 K8s GC    └→ 验证 + 路由
```

**部署位置**:

- Developer Laptop:用户自主安装
- Self-hosted Runner:企业内部 K8s(独立 Cluster)或裸机
- Cloud Workspace(V2,§30.4):临时 Pod,不与 Control Plane 共集群

### 8.6 低 K8s Tax 纪律(§44.2,§86)

**体现**:

1. 第一阶段总 Service 数 ≤ 6(gateway / identity / work-core / worker / postgres / nats)
2. 第一阶段 Deployment 数 ≤ 8(每个 Service 1-2 个)
3. 第一阶段不引入 Service Mesh(§30.6)
4. 第一阶段不引入 Database per Domain(§30.6,单 PostgreSQL)
5. 第一阶段不引入 Vector/Graph/OpenSearch(§30.6)
6. 第一阶段不引入 Full Event Sourcing / Complex CQRS(§30.6)

**监控指标**:

- Service 数 ≤ 10(MVP 阶段)
- Deployment 数 ≤ 15(MVP 阶段)
- Cluster 内 Pod 数 ≤ 100(MVP 阶段)
- 每 Service 平均 CPU < 70%

---

## 9. Traceability & AI Audit

### 9.1 完整追踪链(§39,§42 E 图)

```text
Business Goal
  ↓
Business Requirement
  ↓
WorkItem
  ↓
Acceptance Criteria
  ↓
Worktree
  ↓
Agent Session
  ↓
Context Packet
  ↓
ChangeSet
  ↓
Feedback
  ↓
Validation Evidence
  ↓
Commit
  ↓
PR / MR
  ↓
Acceptance (业务验收)
```

**所有节点必须可双向追溯**:给定任意节点,可上溯到 Business Goal,可下溯到最终 Commit。

### 9.2 节点存储位置

| 节点 | PostgreSQL(SoR) | Search Projection | Object Storage | Audit |
|---|---|---|---|---|
| Business Goal | 是(business_goal 表) | 是 | - | 是 |
| Business Requirement | 是(requirement 表) | 是 | - | 是 |
| WorkItem | 是(work_item 表) | 是 | - | 是 |
| Acceptance Criteria | 是(acceptance_criterion 表) | 是 | - | 是 |
| Worktree | 是(worktree 表) | 是 | - | 是 |
| Agent Session | 是(agent_session 表) | 是 | - | 是 |
| Context Packet | 是(元数据 + Provenance) | 是 | 大型 Symbol Index | 是 |
| ChangeSet | 是(元数据 + Risk Signals) | - | Diff Artifact | 是 |
| Feedback | 是(feedback 表) | 是 | - | 是 |
| Validation Evidence | 是(validation_result 表) | - | Build/Test Log | 是 |
| Commit | 是(commit_link 表) | - | - | 是 |
| PR / MR | 是(pull_request 表) | - | - | 是 |
| Acceptance | 是(acceptance_event 表) | - | - | 是 |

**强制规则**(§17 REQ-AUDIT-002):

- AI Audit Metadata 必须包含全部 9 项问题(见 §6.7)
- 每个节点必带 `tenant_id`
- 跨节点查询由 `domain-trace` 子模块(嵌于 application)提供

### 9.3 AI Audit 字段(§17 REQ-AUDIT-002)

针对 Agent 行为,Audit 必须能回答:

```text
Q1. 谁要求 AI 做什么?
    → AuditEvent.actor (user_id) + AuditEvent.context_refs (WorkItem/AcceptanceCriterion/ADR)

Q2. AI 使用了什么 Context?
    → AIAuditMetadata.context_packet_id → ContextPacket.provenance[]

Q3. AI 修改了什么?
    → AIAuditMetadata.change_set_id → ChangeSet.files / symbols / diff_reference

Q4. 哪个 Agent 执行?
    → AIAuditMetadata.agent_session_id → AgentSession.agent_type / agent_provider / agent_version

Q5. 在哪个 Worktree?
    → AgentSession.worktree_id → Worktree.local_path_reference

Q6. 什么时间?
    → AuditEvent.created_at + AgentSession.started_at / ended_at

Q7. 哪些验证通过?
    → AIAuditMetadata.validation_result_ids[] → ValidationResult.status

Q8. 哪些 Feedback 被消费?
    → AIAuditMetadata.feedback_consumed_ids[] → Feedback.status (VERIFIED)

Q9. 谁批准 Commit/PR/Merge?
    → AIAuditMetadata.approver_user_id
```

### 9.4 追踪链查询 API(接口契约)

```rust
pub trait TraceabilityQueryPort {
    async fn trace_forward(&self, node: TraceNode) -> Result<TraceChain, TraceError>;
    async fn trace_backward(&self, node: TraceNode) -> Result<TraceChain, TraceError>;
    async fn acceptance_coverage(&self, work_item_id: WorkItemId) -> Result<AcceptanceCoverageReport, TraceError>;
    async fn ai_audit(&self, agent_session_id: AgentSessionId) -> Result<AIAuditReport, TraceError>;
}
```

### 9.5 决策表 N 体现(§46 N.1~10,Context Engineering)

| # | 决策 | 体现位置 |
|---|---|---|
| N.1 | Minimum Sufficient Context | §4.4.4 Token Budget, §5.5 Subject 命名 |
| N.2 | Provenance 强制 | §4.4.5, §9.2, §5.7 |
| N.3 | Decision Memory 独立 | §4.4.6, §A.7 状态机 |
| N.4 | Context Priority 分级 | §4.4.4, §4.10.7 P0-P5 |
| N.5 | Active Decision 优先 | §4.4.6 DecisionMemoryPort.list_active |
| N.6 | Handoff Context Packet | §4.2.7 HandoffContextPacket |
| N.7 | Symbol-level 渐进 | §21.2, §30.2 MVP 范围 |
| N.8 | Context Cost 纳入 Planning | §9 REQ-PLAN-006 |
| N.9 | Context Efficiency 观测 | §28.1 AI Observability 指标 |
| N.10 | 敏感 Context AI Content Retention | §6.8 草案 |

### 9.6 AI Content Retention Policy 落地(§28.2,§40)

详见 §6.8。落地机制:

```text
Agent Adapter
    ↓ 写入
AIAuditMetadata (PostgreSQL, 永久)
    ↓ 引用
Sensitive Content Storage (Object Storage, TTL 由 Policy 决定)
    ↓ Lifecycle Policy
物理删除
```

**关键约束**:

- Full Prompt/Response 默认 90 天
- Sensitive Code 立即 Redact
- Project Policy 可调 Summary/Prompt/Response 保留期
- 全程加密(AES-256 at rest)

---

## 10. ADR 草案(对应 §32 ADR-016~030)

> 状态:全部 **Proposed**(等待 RFC + Architect 评审)。本节为基本设计阶段的草案,详细 ADR 在 RFC 阶段补充。

### ADR-016: Worktree as First-class Domain Entity

- **状态**: Proposed
- **背景**: Worktree 不得仅作为 Repository Metadata 或 Branch 附属字段(§22.1,REQ-WT-001~003)。它需承载 Status / Health / ConflictState / Ahead / Behind / ChangedFiles / TestState 等独立状态。
- **选项**:
  - A. Worktree 作为 Repository Metadata 字段
  - B. Worktree 作为独立表但与 WorkItem 直接关联
  - C. **Worktree 作为独立聚合根,通过 development_execution 间接关联 WorkItem**(本设计选定)
- **决策**: 选 C
- **后果**:
  - 支持 1 WorkItem → N Worktree(REQ-DEV-001)
  - Worktree Status 独立于 WorkItem Status(REQ-WF-002)
  - 隔离边界清晰(RISK-019 缓解)
  - 跨 Worktree 聚合查询需经过 development_execution
- **风险**: Worktree 数量可能爆炸,需 Heatmap 优化

### ADR-017: Development Execution Domain

- **状态**: Proposed
- **背景**: WorkItem 与真实代码环境之间需要抽象层 DevelopmentExecution(§21),聚合 Worktree / AgentSession / ChangeSet / Validation / Feedback / Commit / PR。
- **选项**:
  - A. WorkItem 直接关联所有子对象
  - B. **DevelopmentExecution 作为聚合根,WorkItem 1 → N**(本设计选定)
  - C. 使用 Graph Database 表达复杂关系(§30.6 排除)
- **决策**: 选 B
- **后果**:
  - 事务边界清晰
  - 与 Worktree 关系灵活(1 Execution → N Worktree)
  - 多 Execution 间追溯需要额外查询
- **风险**: Execution 数量过多时性能下降(需 V2 优化)

### ADR-018: Local Runtime Architecture

- **状态**: Proposed
- **背景**: Local Runtime 是开发环境与 Control Plane 的桥梁(§23)。它必须独立于 K8s Application Workload 计数(§23.1),且不得形成 Remote Shell(§23.2)。
- **选项**:
  - A. SSH 远程执行(形成 Remote Shell)
  - B. Agent Container 内嵌(破坏 K8s Tax 纪律)
  - C. **独立 Local Daemon,通过白名单 Command 与 Control Plane 通信**(本设计选定)
- **决策**: 选 C
- **后果**:
  - 严格的安全边界
  - 不增加 K8s Workload
  - 可支持 Self-hosted / Cloud Workspace
  - 需要 Device Identity + mTLS 实施成本
- **风险**: Local Runtime Compromise(RISK-016)

### ADR-019: Local Runtime Security Model

- **状态**: Proposed
- **背景**: Local Runtime 是系统最易受攻击的边界(§34)。必须研究 16 项强制项(§23.2)。
- **选项**:
  - A. 仅 mTLS 双向认证
  - B. **mTLS + Device Identity + Command 白名单 + Filesystem Scope + Process Scope + Credential Broker**(本设计选定)
  - C. 不实施 Filesystem Scope(降级)
- **决策**: 选 B
- **后果**:
  - 多层防御
  - Revocation / Remote Disable 可行
  - 实施复杂度高,需 POC 验证
- **风险**: Filesystem Scope 在 Linux/macOS/Windows 行为不一致(§29)

### ADR-020: Observed State vs Business State

- **状态**: Proposed
- **背景**: Worktree 高频本地状态(§22.1 dirty_state, test_state)与业务状态(WorkItem.status)不能混存(§23.3,REQ-DATA-003)。
- **选项**:
  - A. 单一 Status JSON 字段
  - B. **Business Truth 入核心事务,Observed State 入独立 Projection 表**(本设计选定)
  - C. 全部走 Event Sourcing(§30.6 排除)
- **决策**: 选 B
- **后果**:
  - 控制 Write Amplification / Event Volume
  - 区分 UI 显示 Current / Stale / Offline
  - 需要 Reconciliation 协议(§4.1.8)
- **风险**: 长期 Observed State 数据治理(§5.8)

### ADR-021: Agent Adapter Model

- **状态**: Proposed
- **背景**: Agent 厂商多样(Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local / Future)(§24.2)。Domain 层不得绑定单一厂商。
- **选项**:
  - A. 直接调用厂商 SDK
  - B. **Agent Port 抽象 + Adapter 实现**(本设计选定)
  - C. 等待行业标准(被动)
- **决策**: 选 B
- **后果**:
  - 厂商可插拔
  - AgentPolicy 跨厂商统一
  - 抽象成本(§46 决策表 J.5 提示 V1 复审)
- **风险**: Agent Vendor Lock-in(RISK-030)

### ADR-022: SCM Adapter Model

- **状态**: Proposed
- **背景**: GitHub / GitLab 主导,未来可能 Gitea / Bitbucket / Azure DevOps / Self-hosted(§19.1,REQ-SCM-001/002)。Domain 层不得出现厂商对象。
- **选项**:
  - A. 各自独立集成
  - B. **SCM Port 抽象 + 多 Adapter 实现**(本设计选定)
- **决策**: 选 B
- **后果**:
  - 多 SCM 厂商可插拔
  - 业务逻辑统一
  - 不同 SCM 能力差异需在 ACL 中补偿
- **风险**: SCM Sync Loop(RISK-027)

### ADR-023: Structured Feedback Model

- **状态**: Proposed
- **背景**: Feedback 是结构化人类修正指令(§25),而非普通 Comment。必须含 Expected / Preserve / Prohibit 字段。
- **选项**:
  - A. Comment 字段扩展
  - B. **独立 Feedback 聚合根,Target/Type/Expected/Preserve/Prohibit**(本设计选定)
- **决策**: 选 B
- **后果**:
  - 高密度、低歧义 Agent Instruction
  - 全粒度 Target 绑定(WorkItem → Diff Hunk)
  - Feedback Inbox / Intervention Queue 可行
  - UI 复杂度上升
- **风险**: Feedback Misinterpretation(RISK-026)

### ADR-024: Context Compiler

- **状态**: Proposed
- **背景**: Context Compiler 是确定性/半确定性系统能力,非 LLM(§26.1)。输入 WorkItem / Acceptance / Worktree / Repository / Relevant Files / Symbols / ADR / Previous Decisions / Open Feedback / Failed Tests / Build Failure / Git Diff / PR Review / Agent Rules。输出 ContextPacket。
- **选项**:
  - A. 简单 Prompt Template
  - B. **Context Compiler 子系统(含 Token Budget, Provenance, Priority Layer)**(本设计选定)
  - C. 借助 LLM 自行选择(不可控)
- **决策**: 选 B
- **后果**:
  - Minimum Sufficient Context
  - 避免 Context Pollution / Repeated Prompt
  - Decision 独立管理
  - 实现复杂度高,需 PoC 验证(§11 POC-022)
- **风险**: Context Explosion(RISK-024), Low-quality Context Selection(RISK-025)

### ADR-025: Context Packet Persistence

- **状态**: Proposed
- **背景**: Context Packet 是否需要持久化?需要支持 Provenance 反查、Handoff、可重放(§26.3)。
- **选项**:
  - A. 不持久化,每次重算
  - B. **持久化(元数据 + Provenance,大文件走 Object Storage)**(本设计选定)
- **决策**: 选 B
- **后果**:
  - Trace 反查可行
  - Handoff Context Packet 可生成
  - 存储成本上升
- **风险**: Storage 增长需 Lifecycle Policy(§5.8)

### ADR-026: Agent Session Persistence

- **状态**: Proposed
- **背景**: AgentSession 是否持久化?需要支持 §24.1 字段(Plan / Decisions / ChangeSet / ValidationResult / FeedbackConsumed / TraceReference)。
- **选项**:
  - A. 仅内存
  - B. **持久化(元数据,大文件走 Object Storage)**(本设计选定)
- **决策**: 选 B
- **后果**:
  - AI Audit 可查
  - 跨 Session 状态可追踪
  - 全文 Transcript 需 AI Content Retention Policy(§6.8)
- **风险**: Agent Session State Divergence(RISK-023)

### ADR-027: ChangeSet Storage

- **状态**: Proposed
- **背景**: ChangeSet 不只存 Git Diff(§21.1),需承载 Files / Symbols / Risk Signals / Dependency / Schema / Config / Test Changes。
- **选项**:
  - A. 仅 Git Diff
  - B. **结构化 ChangeSet 聚合根 + Diff Reference 走 Object Storage**(本设计选定)
- **决策**: 选 B
- **后果**:
  - 风险门控可行
  - Symbol-level Feedback 可关联
  - 实现成本(§11 POC-021)
- **风险**: Storage 增长(§5.1)

### ADR-028: Symbol Analysis Strategy

- **状态**: Proposed
- **背景**: Symbol-level Context 需 Symbol 索引(§21.2)。MVP 不强制完整 IDE Compiler Database(§30.6)。
- **选项**:
  - A. 完整 IDE Compiler Database(成本爆炸)
  - B. **第一阶段 File-level + Basic Symbol Detection,V1 渐进到 Symbol-level**(本设计选定)
  - C. 引入 Graph Database(§30.6 排除)
- **决策**: 选 B
- **后果**:
  - MVP 可行
  - 避免 Graph DB 早期投资
  - Symbol-level Conflict Detection 推迟到 V1
- **风险**: Symbol-level Conflict Detection 推迟(§15 Open Issue J.2)

### ADR-029: Worktree Conflict Detection

- **状态**: Proposed
- **背景**: Worktree Conflict Intelligence 第一阶段 File-level(§22.4),第二阶段 Symbol-level。
- **选项**:
  - A. 全文 AI 分析(成本高)
  - B. **File-level 通过 Git diff metadata,Symbol-level 通过本地解析器 + AI 辅助**(本设计选定)
  - C. 推迟到 V2
- **决策**: 选 B
- **后果**:
  - 第一阶段可行
  - V1 渐进
- **风险**: Worktree Conflict Explosion(RISK-028)

### ADR-030: Agent Policy Enforcement

- **状态**: Proposed
- **背景**: Agent Policy 必须由 Application/Authorization 层强制(§24.3,REQ-PERM-002),不能仅靠 Prompt。
- **选项**:
  - A. Prompt 约束
  - B. **Application 层 Policy Enforcement(Repository / Worktree / Path / Tool / Network / Secret / Runtime / Context / Change Scope / Review / Test / Approval)**(本设计选定)
- **决策**: 选 B
- **后果**:
  - 多层防御
  - Agent Escapes Worktree Scope 风险下降(RISK-017)
  - 实施成本(§11 POC-029)
- **风险**: Policy 误配置可能影响合法 Agent 行为

---

## 11. PoC 实施计划(对应 §31 POC-016~030)

> 优先级:MVP 必做 / V1 候选 / V2

| ID | 目标 | 范围 | 成功标准 | 依赖 | 优先级 |
|---|---|---|---|---|---|
| **POC-016** | Local Runtime Secure Connection | mTLS + Device Identity + Command 白名单 | 模拟 Local Daemon 与 Control Plane 双向认证通过;Command Token 5min TTL 验证;Revocation 测试通过 | ADR-019 | **MVP 必做** |
| **POC-017** | Worktree State Synchronization | Snapshot + Incremental + Heartbeat | 1k Worktree 状态 1s 内同步;UI 区分 Current/Stale/Offline | ADR-020 | **MVP 必做** |
| **POC-018** | Worktree Offline / Reconnect | 离线缓存 + Reconnect 后 Reconciliation | 离线 1h 后重连,Reconciliation 报告偏差正确;不静默合并 | POC-017 | **MVP 必做** |
| **POC-019** | Multiple Worktree Observation | 1 Project 下 100 Worktree 同屏观察 | UI 渲染 100 Worktree < 500ms;Filter/Sort/Group 流畅 | POC-017 | **MVP 必做** |
| **POC-020** | Agent Session Tracking | AgentSession 状态机 + Domain Event | AgentSession 状态机完整迁移;事件全部触发 | ADR-026 | **MVP 必做** |
| **POC-021** | Structured Feedback → Agent Instruction | 编译 Feedback 为 AgentInstruction | 10 个典型 Feedback 编译后,Token 下降 50%(对比完整聊天);Provenance 完整 | ADR-023, ADR-024 | **MVP 必做** |
| **POC-022** | Context Compiler | 最小 ContextPacket 生成 | Given 1 WorkItem + 1 Worktree + 3 Feedback,生成 ContextPacket;Token Budget 符合 §4.4.4 | ADR-024 | **MVP 必做** |
| **POC-023** | Context Packet Size / Relevance | Token Budget 校准 | 真实 WorkItem 30 个,Token 分布 P50/P95 测量;校准 §4.4.4 表 | POC-022 | **V1 候选** |
| **POC-024** | File-level Conflict Detection | Worktree Heatmap + File-level Conflict | 100 Worktree / 10k File 下,Conflict 检测 < 1s;Heatmap 正确 | ADR-029 | **MVP 必做** |
| **POC-025** | Symbol-level Feedback | Symbol 解析 + Feedback Target = Symbol | 给定 1 Rust / 1 TypeScript / 1 Python 文件,Symbol 识别准确率 > 95% | ADR-028 | **V1 候选** |
| **POC-026** | GitHub Adapter | SCM Port GitHub 实现 | Repository / Branch / Commit / PR / Review / Webhook 全功能;Rate Limit 兜底 | ADR-022 | **MVP 必做** |
| **POC-027** | GitLab Adapter | SCM Port GitLab 实现 | 同上,含 MR / Pipeline | ADR-022 | **MVP 必做** |
| **POC-028** | Agent Adapter | Agent Port 至少 1 厂商实现(Codex 或 Claude Code) | AgentSession 完整生命周期;Policy 强制点全部生效 | ADR-021 | **MVP 必做** |
| **POC-029** | Agent Policy Enforcement | 12 个强制点全部验证 | 越权 Path / Tool / Network / Secret 全部被拦截;Audit 完整 | POC-028 | **MVP 必做** |
| **POC-030** | Cross-Worktree Isolation | Filesystem / Env / Process / Port 隔离 | 同机 5 Worktree 并行,互不可见 Env / Process / Port | ADR-019 | **MVP 必做** |

**MVP 必做 13 个**:POC-016/017/018/019/020/021/022/024/026/027/028/029/030
**V1 候选 2 个**:POC-023, POC-025
**V2 0 个**(待 V1 完成后新增)

---

## 12. 风险登记与缓解(对应 §33 RISK-016~030)

| ID | 风险 | 影响等级 | 缓解措施 | 监控指标 |
|---|---|---|---|---|
| **RISK-016** | Local Runtime Compromise | Critical | mTLS + Device Identity + Command 白名单 + Filesystem Scope + Revocation + Remote Disable(§4.6.3, ADR-019) | remote_disable 触发次数;异常 Command 占比 |
| **RISK-017** | Agent Escapes Worktree Scope | High | AgentPolicy.allowed_* 强制;Local Runtime Filesystem Scope;Application 层 Authorization(§4.2.5, ADR-030) | Agent Policy Violation 次数 |
| **RISK-018** | Agent Secret Leakage | High | Credential Broker + Scoped Token + Short-lived + Process Isolation + Secret Redaction(§6.4) | Secret 命中 Redaction 规则次数 |
| **RISK-019** | Cross-Worktree Context Leakage | High | Worktree Isolation(§22.5);tenant_id 强制;Context Compiler 不跨 Worktree 加载(§4.4) | Cross-Worktree Access 拦截次数 |
| **RISK-020** | Cross-Repository Context Leakage | High | Context Compiler 不跨 Repository 加载;AgentPolicy.allowed_repositories(§4.2.5) | Cross-Repository Access 拦截次数 |
| **RISK-021** | Prompt Injection from Repository | Critical | Untrusted Content(P5)与 Trusted Human Policy(P0)优先级分离(§4.10.7) | Untrusted-as-Instruct 检测次数 |
| **RISK-022** | Stale Worktree State | Medium | UI 区分 Current/Stale/Offline/Unknown(§23.4);Observed State 走 Projection(§4.1.5) | Stale Worktree 占比 |
| **RISK-023** | Agent Session State Divergence | Medium | AgentSession 持久化 + Reconciliation(§4.2);Local Runtime 上报机制(§4.6.5) | AgentSession 状态偏差次数 |
| **RISK-024** | Context Explosion | Medium | Token Budget + Priority Layer + Decision 优先于历史(§4.4.4) | Context Packet Token 分布 P95 |
| **RISK-025** | Low-quality Context Selection | Medium | Relevant Context Ratio 监控;Provenance 强制(§4.4.5) | Relevant Context Ratio;First-pass Acceptance Rate |
| **RISK-026** | Feedback Misinterpretation | Medium | Precise Feedback(Expected/Preserve/Prohibit 强制);Feedback 状态机(§4.3.5);Rejection 监控(§28.1) | Feedback Reopen Rate;Feedback Repetition |
| **RISK-027** | SCM Sync Loop | High | Bidirectional Sync 需评估 Loop 防护(§18.1);Idempotency Key;Sync Token 校验 | Sync Loop 检测次数 |
| **RISK-028** | Worktree Conflict Explosion | Medium | File-level 第一阶段(§4.1.6);Heatmap 投影;Symbol-level 推迟 V1 | Conflict Rate;Heatmap Lag |
| **RISK-029** | Local Runtime Version Fragmentation | Medium | Runtime 升级策略 + 强制最低版本(§23.5);向后兼容 API | Runtime Version 分布 |
| **RISK-030** | Agent Vendor Lock-in | Medium | Agent Port 抽象(§4.2.4, ADR-021);AgentPolicy 跨厂商统一(§4.2.5) | Agent Vendor 数量;Adapter 复用率 |

**监控指标分类**:

- 业务安全:RISK-016, 017, 018, 019, 020, 021, 027
- 状态一致性:RISK-022, 023, 029
- AI 质量:RISK-024, 025, 026
- 性能:RISK-028

---

## 13. MVP 范围裁剪(对应 §30.2~6)

### 13.1 MVP Must Have 逐项映射(§30.2,§65)

| §30.2 项 | 对应 Module / Worktree | 状态 |
|---|---|---|
| GitHub Integration | domain-scm(SCM Adapter GitHub 实现) | MVP 必做 |
| GitLab Integration | domain-scm(SCM Adapter GitLab 实现) | MVP 必做 |
| Repository Link | domain-scm + domain-work-item | MVP 必做 |
| Worktree Registration | domain-worktree | MVP 必做 |
| Worktree Status | domain-worktree + Observed State | MVP 必做 |
| Worktree Dashboard | domain-collaboration(Realtime) + UI Projection | MVP 必做 |
| Agent Session Registration | domain-agent | MVP 必做 |
| Agent Status | domain-agent + Observed State | MVP 必做 |
| File-level ChangeSet | domain-development | MVP 必做 |
| Basic Symbol Detection | domain-context(Symbol Index 最小集) | MVP 必做 |
| Structured Feedback | domain-feedback | MVP 必做 |
| Feedback Inbox | domain-feedback(Projection) + UI | MVP 必做 |
| Context Packet Generation | domain-context | MVP 必做 |
| Build/Test Result | domain-validation(基础 Build/Test) | MVP 必做 |
| Basic Conflict Detection | domain-worktree(§4.1.6) | MVP 必做 |
| Commit Link | domain-scm + domain-development | MVP 必做 |
| PR/MR Link | domain-scm + domain-development | MVP 必做 |
| Development Timeline | domain-audit + UI | MVP 必做 |
| Local Runtime | domain-local-runtime + Local Daemon | MVP 必做 |
| Tenant-aware Security | domain-tenant + domain-permission | MVP 必做 |
| Audit | domain-audit | MVP 必做 |

### 13.2 V1 Should Have 对应 Module(§30.3,§66)

| §30.3 项 | 对应 Module | 说明 |
|---|---|---|
| Symbol-level Feedback | domain-feedback(扩展 Target = Symbol) | 需 Symbol 索引支撑(POC-025) |
| Symbol-level Conflict | domain-worktree(扩展) | 依赖 Symbol 索引 |
| Decision Memory | domain-context(Decision) | 已有,需 UI 暴露 |
| Agent Handoff | domain-agent(HandoffContextPacket) | 已有,需 UI 流程 |
| Acceptance Coverage | domain-validation(AcceptanceCoverage) | 已有,需 UI |
| Advanced Context Selection | domain-context(ML 辅助) | V1 中期评估 |
| PR Review Feedback Import | domain-scm + domain-feedback | 解析 Review Comment |
| Saved Worktree Views | domain-collaboration | UI 个性化 |
| Development Heatmap | domain-worktree(§4.1.6) | 第一阶段简化版 |
| Agent Policy Templates | domain-agent + domain-permission | Policy 模板库 |
| Remote Runner | domain-local-runtime(Self-hosted) | 第二种 Runtime 类型 |
| Context Cost Analysis | domain-context(§9) | UI 报表 |

### 13.3 V2 Candidates 对应 Module(§30.4,§67)

| §30.4 项 | 对应 Module | 说明 |
|---|---|---|
| Semantic Conflict Detection | domain-worktree(AI 辅助) | 需 AI 分类器 |
| Cross-Worktree Dependency Graph | domain-development + Projection | 需 Graph 或关系 DB(§30.6 限制) |
| AI Planning Assistance | domain-planning(扩展) | §9 REQ-PLAN-006 |
| Multi-Agent Comparison | domain-agent + domain-worktree | 需多 Agent 并行基础设施 |
| Task Parallelization Recommendation | domain-planning + AI | 同上 |
| Agent Performance Analytics | domain-audit(分析) | BI 报表 |
| Advanced Runtime Isolation | domain-local-runtime(Kata 等) | 重型方案 |
| Cloud Development Runtime | domain-local-runtime(Cloud Workspace) | 第四种 Runtime |

### 13.4 Future(§30.5,§68)

仅在验证价值后研究:

- Agent Swarm / Autonomous Task Decomposition / Autonomous Multi-Agent Scheduling
- Graph Database / Vector Database / Semantic Repository Memory
- Cloud IDE / Managed Git Hosting
- Autonomous Merge / Autonomous Deployment

### 13.5 Explicit Non-Goals 强化(§30.6,§69)

**MVP / V1 / V2 / Future 任何阶段均不实现**:

- GitHub Clone / GitLab Clone / Full Jira Enterprise Clone
- Full IDE / Cloud IDE / Git Hosting Platform
- Agent Swarm / Autonomous Company / Autonomous Production Deployment
- Service Mesh / 几十个微服务 / Database per Domain
- Graph Database / Vector Database / OpenSearch Cluster
- Full Event Sourcing / Complex CQRS

**设计纪律**:

- 单 PostgreSQL(非 Database per Domain)
- 单 Modular Monolith(非 Microservices)
- Projection-based Search(非 OpenSearch Cluster)
- Application Service 编排(非 Event Sourcing)

---

## 14. 决策继承表(§46 决策表 A-O)

> 每条决策在本设计书中的落实位置。

| 决策表 | 关键决策 | 本文落位 | 备注 |
|---|---|---|---|
| **A. MVP Must Have** | 见 §30.2 / §13.1 | §13.1 表 | |
| **B. V1 Should Have** | 见 §30.3 / §13.2 | §13.2 表 | |
| **C. V2 Candidates** | 见 §30.4 / §13.3 | §13.3 表 | |
| **D. Future Architecture** | 见 §30.5 / §13.4 | §13.4 表 | |
| **E. Explicit Non-Goals** | 见 §30.6 / §13.5 | §13.5 | |
| **F. Top 10 Product Decisions** | 1-10 见 §46 | §4.1.5(F.1/2), §4.3(F.3), §4.4.1(F.4), §4.9.5(F.5), §4.1.3(F.6), §4.3.6(F.7), §4.5.5(F.8), §4.9.4(F.9), §4.9.4(F.10) | |
| **G. Top 10 Architecture Decisions** | 1-10 见 §46 | §1.2(G.1), §1.1(G.2), §5.1(G.3), §5.2(G.4), §4.7.3(G.5), §4.2.4(G.6), §5.3(G.7), §5.1(G.8), §4.1.8(G.9), §1.4(G.10) | |
| **H. Top 10 SaaS Risks** | RISK-016~030 | §12 | |
| **I. Top 10 K8s Risks** | 沿用原文档 | §15 Open Issue J.1 | 待原文档核对 |
| **J. Top 10 Open Issues** | 1-5 见 §46 | §15 | 新增 5 条 |
| **K. Top 10 Vibe Coding Decisions** | 1-10 见 §46 | §2.1.1(K.1), §4.8.1(K.2), §4.1.1(K.3), §4.8.2(K.4), §21.2(K.5), §4.5.5(K.6), §4.2.6(K.7), §4.2.7(K.8), §4.2.7(K.9), §40.1(K.10) | |
| **L. Top 10 Worktree Risks** | 1-10 见 §46 | §12 + §4.1 全章 | |
| **M. Top 10 Agent Security Risks** | 1-10 见 §46 | §4.10.6, §4.10.7, §4.10.8, §6 | |
| **N. Top 10 Context Engineering Decisions** | 1-10 见 §46 | §9.5 | |
| **O. Top 10 Human Feedback Design Decisions** | 1-10 见 §46 | §4.3(全章), §4.3.6(O.6), §4.3.6(O.7), §4.3.5(O.4), §28.1(O.9) | |

### 14.1 冲突优先级在设计中的体现(§43.2,§43.3,§97-98,§104)

**事实冲突优先级(§43.2)**:

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

**实现位置**:

- §4.4.4 Context Priority(P0-P4):与本优先级对齐
- §4.10.7 Prompt Injection 防护(Trusted P0 vs Untrusted P5)
- §4.5.5 AI Completion 判定链(Validation > Feedback Resolution > Gate)
- §4.1.9 Worktree Completion 判定(7 项检查)
- §6.8 AI Content Retention(全 P0 不可裁剪)

**最终冲突决策优先级(§43.3,§104)**:

```text
Business Correctness > Tenant Isolation > Data Integrity > Security
> Explicit Human Intent > Acceptance Correctness > Traceability
> Availability > Maintainability > Developer Experience
> AI Interaction Quality > Performance > Scalability
> K8s Extensibility > Resource Efficiency > Microservices > Serverless
> AI Autonomy > Technology Novelty
```

**关键设计含义**:

- AI Autonomy 永远不得凌驾于 Human Intent / Security / Data Integrity / AC(§43.3)
- Microservices 优先级 < Resource Efficiency(支持 K8s Tax 纪律)
- Serverless 优先级 < Microservices(支持 Scale-to-Zero 评估原则)

---

## 15. Open Issues(继承 §46 决策表 J + 新增)

### 15.1 继承 §46 决策表 J

| # | Open Issue | 状态 |
|---|---|---|
| **J.1** | 原《Kubernetes-native 工作管理 SaaS 要件定义》文档未能在本仓库定位,§0-§17、§31-§33、§44.2 部分内容为重新编写,需与原文档核对一致性 | **继承待解决** |
| **J.2** | Symbol-level Conflict Detection 的具体分析粒度与性能边界待 PoC 验证(POC-025) | V1 验证 |
| **J.3** | Context Compiler 的 Token Budget 具体阈值待真实数据校准(TBD-MEASURE) | V1 校准 |
| **J.4** | Local Runtime 与 SaaS Control Plane 之间的 Reconciliation 协议细节待 ADR-020 确定 | RFC 阶段 |
| **J.5** | Agent Vendor 数量增长后 Agent Port 抽象是否足够,需在 V1 阶段复审 | V1 复审 |

### 15.2 基本设计阶段新发现的 Open Issue

| # | Open Issue | 建议解决阶段 |
|---|---|---|
| **J.6** | 现有 §4.4.4 Token Budget 分级表(§4.4.4 草案)需 PoC 校准,POC-023 应给出 P50/P95 实测值 | V1 |
| **J.7** | §4.10.8 Secret Redaction 规则的覆盖范围(PEM / JWT / API Key / Database URL 等)需在详细设计阶段明确 | 详细设计 |
| **J.8** | §5.1 Object Storage 与 PostgreSQL 的边界判断(>1MB 或 >10K 行)需考虑 Code Diff 压缩后的实际大小,可能在详细设计阶段调整 | 详细设计 |
| **J.9** | §4.1.9 Worktree Completion 判定的 7 项检查在不同 Project 的可配置粒度,需要在 Project Policy Schema 详细设计时明确 | 详细设计 |
| **J.10** | §4.7 SCM Adapter 是否需要支持 Self-hosted Git(非 GitHub/GitLab 公有云),V1 评估 | V1 评估 |
| **J.11** | §4.9.4 Traceability Query Port 的反向追溯在跨 Project / 跨 Tenant 时的权限边界需明确(默认禁止跨 Tenant 追溯) | 详细设计 |
| **J.12** | §6.8 AI Content Retention Policy 的 Project 可配置范围(Summary / Prompt / Response)需 Product/Compliance 共同决定 | 详细设计 |
| **J.13** | §4.6.6 Future Runtime(Cloud Workspace / Ephemeral Coding Environment)的 Domain 抽象是否需要新增 RuntimeKind 枚举,或在 V2 评估 | V1 评估 |
| **J.14** | §7 状态机中"任意状态 → ABANDONED"是否需要保留所有路径,还是限定为特定状态,需 UX 验证 | V1 |
| **J.15** | §4.10.7 Prompt Injection 防护中"Untrusted-as-Instruct"的检测是依赖 LLM 自身判断还是平台侧分类器,需要 RFC 评估准确率与成本 | RFC |

---

## 附录 A:关键状态机图

### A.1 Worktree 生命周期(§22.2)

```mermaid
stateDiagram-v2
    [*] --> CREATED
    CREATED --> READY: Local Runtime 路径创建成功
    READY --> ASSIGNED: 分配 AgentSession
    ASSIGNED --> AGENT_RUNNING: Agent Process 启动成功
    AGENT_RUNNING --> WAITING_FEEDBACK: OpenFeedback 触发
    WAITING_FEEDBACK --> FEEDBACK_RECEIVED: Feedback APPLIED
    FEEDBACK_RECEIVED --> AGENT_RUNNING: Agent 继续
    AGENT_RUNNING --> VALIDATING: AgentSession.ended_at
    VALIDATING --> READY_FOR_REVIEW: §4.1.9 七项检查全通过
    VALIDATING --> BLOCKED: 关键 Validation Failed
    AGENT_RUNNING --> CONFLICTED: Worktree Conflict Detector 触发
    CONFLICTED --> ASSIGNED: 冲突已解决
    READY_FOR_REVIEW --> REVIEWING: Reviewer 开始
    REVIEWING --> READY_FOR_COMMIT: 审查通过
    READY_FOR_COMMIT --> COMMITTED: Commit 成功
    COMMITTED --> PR_OPEN: PR 创建成功
    PR_OPEN --> MERGED: SCM Webhook
    MERGED --> ARCHIVED: 30 天后自动归档
    BLOCKED --> ASSIGNED: Block 解除
    AGENT_RUNNING --> ABANDONED: 用户显式放弃
    VALIDATING --> ABANDONED: 用户显式放弃
    REVIEWING --> ABANDONED: 用户显式放弃
    ABANDONED --> ARCHIVED: 90 天后自动归档
    ARCHIVED --> [*]
```

### A.2 WorkItem Workflow(默认三态 + 扩展)

> 默认最简三态路径:`TODO → IN_PROGRESS → DONE`(basic-design §4.9.3 / §7.2,F-05 修复后口径)。扩展状态 `IN_REVIEW / BLOCKED / CANCELLED` 属于 Project Policy 自定义扩展,非默认。

```mermaid
stateDiagram-v2
    [*] --> TODO
    TODO --> IN_PROGRESS: User / System
    IN_PROGRESS --> DONE: 直接完成(默认三态)
    IN_PROGRESS --> IN_REVIEW: User 提交审查
    IN_REVIEW --> DONE: User 审查通过
    IN_PROGRESS --> BLOCKED: 阻塞
    BLOCKED --> IN_PROGRESS: 解除
    TODO --> CANCELLED: User
    IN_PROGRESS --> CANCELLED: User
    IN_REVIEW --> CANCELLED: User
    DONE --> [*]
    CANCELLED --> [*]
    note right of IN_PROGRESS
        Worktree Status 独立
        可同时存在多种状态
    end note
```

### A.3 Feedback 状态机(§25.3)

```mermaid
stateDiagram-v2
    [*] --> OPEN
    OPEN --> ACKNOWLEDGED: Agent 拉取并加入 Context Packet
    ACKNOWLEDGED --> APPLIED: Agent 提交含 Target 的 ChangeSet
    APPLIED --> VERIFIED: Validation 跑过对应 AC
    OPEN --> REJECTED: User 拒绝
    ACKNOWLEDGED --> REJECTED: User 拒绝
    OPEN --> SUPERSEDED: 新 Feedback 显式取代
    ACKNOWLEDGED --> SUPERSEDED
    APPLIED --> SUPERSEDED
    VERIFIED --> [*]
    REJECTED --> [*]
    SUPERSEDED --> [*]
```

### A.4 AgentSession 状态机(§24.1)

```mermaid
stateDiagram-v2
    [*] --> CREATED
    CREATED --> STARTING: Application
    STARTING --> RUNNING: Local Runtime Agent Process 启动成功
    RUNNING --> WAITING_TOOL: Agent Adapter 检测到 Tool Call
    WAITING_TOOL --> TOOL_RUNNING: Local Runtime 启动 Tool
    TOOL_RUNNING --> TOOL_COMPLETED: Local Runtime Tool 完成
    TOOL_COMPLETED --> RUNNING: Agent Adapter 继续
    RUNNING --> WAITING_FEEDBACK: OpenFeedback 触发
    WAITING_FEEDBACK --> FEEDBACK_RECEIVED: Feedback 提交
    FEEDBACK_RECEIVED --> RUNNING
    RUNNING --> VALIDATING: AgentSession.ended_at + is_ai_complete_claim
    VALIDATING --> COMPLETED: §4.5.5 链全通过
    VALIDATING --> FAILED: 关键 Validation 失败
    RUNNING --> ABORTED: User / Policy 拒绝
    STARTING --> ABORTED
    WAITING_TOOL --> ABORTED
    WAITING_FEEDBACK --> ABORTED
    RUNNING --> CRASHED: Local Runtime 进程异常
    WAITING_TOOL --> CRASHED
    WAITING_FEEDBACK --> CRASHED
    RUNNING --> TIMEOUT: 超过 max_runtime_seconds
    COMPLETED --> [*]
    FAILED --> [*]
    ABORTED --> [*]
    CRASHED --> [*]
    TIMEOUT --> [*]
```

### A.5 ValidationResult 状态机

```mermaid
stateDiagram-v2
    [*] --> PENDING
    PENDING --> RUNNING: CI / Local Runtime 启动
    RUNNING --> PASSED: 全部 assertion 通过
    RUNNING --> FAILED: 任意 assertion 失败
    RUNNING --> ERRORED: 异常(如编译失败 / 网络中断)
    PENDING --> SKIPPED: Policy 跳过
    PASSED --> [*]
    FAILED --> [*]
    ERRORED --> [*]
    SKIPPED --> [*]
```

### A.6 PullRequest 状态机(§18,§19)

```mermaid
stateDiagram-v2
    [*] --> DRAFT
    DRAFT --> OPEN: User / Application
    OPEN --> REVIEWING: SCM Webhook review requested
    REVIEWING --> CHANGES_REQUESTED: SCM Webhook review with changes_requested
    CHANGES_REQUESTED --> OPEN: User
    REVIEWING --> APPROVED: SCM Webhook review approved
    APPROVED --> MERGEABLE: SCM CI Pass + Branch 同步
    MERGEABLE --> MERGED: User / Application (ProjectPolicy.merge_gate)
    OPEN --> CLOSED: User / SCM
    REVIEWING --> CLOSED
    CHANGES_REQUESTED --> CLOSED
    MERGED --> [*]
    CLOSED --> [*]
```

### A.7 Decision 状态机(§26.5)

```mermaid
stateDiagram-v2
    [*] --> ACTIVE
    ACTIVE --> SUPERSEDED: 新 Decision 取代
    ACTIVE --> INVALIDATED: 显式标记无效
    SUPERSEDED --> [*]
    INVALIDATED --> [*]
```

---

## 附录 B:模块依赖图

```mermaid
flowchart LR
    classDef coreDomain fill:#ffd54f,stroke:#333,stroke-width:2px
    classDef supportingDomain fill:#81d4fa,stroke:#333,stroke-width:1px
    classDef genericDomain fill:#c5e1a5,stroke:#333,stroke-width:1px
    classDef crosscut fill:#f8bbd0,stroke:#333,stroke-width:2px

    DT[domain-tenant]:::genericDomain
    DWS[domain-workspace]:::genericDomain
    DPJ[domain-project]:::genericDomain
    DID[domain-identity]:::genericDomain
    DPE[domain-permission]:::genericDomain
    DNT[domain-notification]:::genericDomain
    DCB[domain-collaboration]:::genericDomain
    DLR[domain-local-runtime]:::genericDomain

    DWI[domain-work-item]:::coreDomain
    DWF[domain-workflow]:::supportingDomain
    DBO[domain-board]:::supportingDomain
    DPL[domain-planning]:::supportingDomain
    DCO[domain-comment]:::supportingDomain
    DRL[domain-relation]:::supportingDomain
    DAU[domain-automation]:::supportingDomain

    DDX[domain-development]:::coreDomain
    DSC[domain-scm]:::supportingDomain
    DWT[domain-worktree]:::coreDomain
    DAG[domain-agent]:::coreDomain
    DFB[domain-feedback]:::coreDomain
    DCT[domain-context]:::coreDomain
    DVL[domain-validation]:::coreDomain
    DIN[domain-integration]:::supportingDomain

    DAT[domain-audit]:::crosscut
    DSR[domain-search]:::crosscut

    DT --> DWS
    DWS --> DPJ
    DT --> DID
    DID --> DPE
    DPJ --> DPE
    DPJ --> DWI
    DPJ --> DWF
    DWI --> DWF
    DWI --> DBO
    DWI --> DPL
    DWI --> DRL
    DWI --> DCO
    DWI --> DDX
    DDX --> DWT
    DDX --> DAG
    DDX --> DFB
    DDX --> DCT
    DDX --> DVL
    DDX --> DSC
    DWT --> DSC
    DWT --> DAG
    DWT --> DLR
    DFB --> DCT
    DFB --> DVL
    DCT --> DVL
    DSC --> DIN

    DAT -.Append-only.-> ALL[All Domains]
    DPE -.Check.-> ALL
    DNT -.Publish.-> ALL
    DSR -.Read.-> ALL
    DCB -.Realtime.-> DWI
    DCB -.Realtime.-> DWT
    DAU -.Trigger.-> DNT
    DID --> DLR
```

**依赖方向规则**:

- Generic → Work Management → Core Domain(由内向外)
- Development Domain 共享 Work Management 的 Project/WorkItem
- 不允许反向(Worktree → Work-item,SCM → Worktree 等)
- Audit / Search / Notification / Permission 不被任何 domain 反向依赖
- Audit 不可读(只 Append)

---

## 附录 C:数据所有权矩阵

> Module × 数据类型 = 所有权/存储

| Module / 数据类型 | WorkItem | Worktree | AgentSession | ChangeSet | ContextPacket | Feedback | Validation | Decision | Commit | PR | Audit | Symbol | Diff | BuildLog | TestLog | AgentTranscript |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **domain-tenant** | R(tenant_id) | R | R | R | R | R | R | R | R | R | R | R | R | R | R | R |
| **domain-workspace** | R/W(workspace_id) | R | R | R | R | R | R | R | R | R | R | R | R | R | R | R |
| **domain-project** | R/W(project_id) | R | R | R | R | R | R | R | R | R | R | R | R | R | R | R |
| **domain-work-item** | **R/W**(SoR) | R | R | R | R | R | R | R | R | R | R | R | R | - | - | - |
| **domain-workflow** | R/W(state) | - | - | - | - | - | - | - | - | - | R | - | - | - | - | - |
| **domain-board** | R/W(column) | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
| **domain-planning** | R/W(sprint_id) | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
| **domain-relation** | R/W(relation) | - | - | - | - | - | - | - | - | - | R | - | - | - | - | - |
| **domain-comment** | R/W(comment) | - | - | - | - | - | - | - | - | - | R | - | - | - | - | - |
| **domain-development** | R | R | R | **R/W**(SoR) | R | R | R | R | R | R | R | R | **W**(Object Storage) | - | - | - |
| **domain-scm** | R | R | - | R | - | - | - | - | **R/W**(link) | **R/W**(SoR) | R | - | R | - | - | - |
| **domain-worktree** | R | **R/W**(SoR) | R | R | R | R | R | - | R | R | R | R | R | - | - | - |
| **domain-agent** | R | R | **R/W**(SoR) | R | R | R | R | - | - | - | R | - | - | - | - | **R/W**(Object Storage, Retention) |
| **domain-feedback** | R | R | R | R | R | **R/W**(SoR) | R | - | - | - | R | R | R | - | - | - |
| **domain-context** | R | R | R | R | **R/W**(SoR) | R | R | **R/W**(SoR) | - | - | R | R | R | - | - | - |
| **domain-validation** | R | R | R | R | R | R | **R/W**(SoR) | - | R | R | R | - | R | **R/W**(Object Storage) | **R/W**(Object Storage) | - |
| **domain-audit** | Append(R) | Append | Append | Append | Append | Append | Append | Append | Append | Append | **R/W**(SoR) | Append | Append | Append | Append | Append |
| **domain-search** | R(Projection) | R | R | R | R | R | R | R | R | R | - | R | R | R | R | R |
| **domain-notification** | R | R | R | R | R | R | R | R | R | R | - | - | - | - | - | - |
| **domain-permission** | R(actor check) | R | R | R | R | R | R | R | R | R | R | R | R | R | R | R |
| **domain-identity** | R(actor) | R | R | - | R | R | R | R | R | R | R | - | - | - | - | - |
| **domain-collaboration** | R(Realtime) | R | R | - | - | - | - | - | - | - | - | - | - | - | - | - |
| **domain-automation** | R(Trigger) | R | R | - | R | R | R | - | - | - | - | - | - | - | - | - |
| **domain-integration** | R | - | - | - | - | - | - | - | R | R | - | - | - | - | - | - |
| **domain-local-runtime** | R | R(查询) | R(查询) | - | - | - | - | - | - | - | Append | - | - | - | - | - |

**图例**:

- **R/W(SoR)**:该 Module 是该数据的 System of Record,可读写
- **R/W(Object Storage)**:该 Module 负责大文件,存 Object Storage
- **R**:只读引用
- **Append**:只追加,不可读(由审计接口访问)
- **R(Projection)**:派生视图,不可作为业务事实源(§12,REQ-SEARCH-001)
- **-**:无关

**关键观察**:

1. PostgreSQL SoR 集中在 work-item, worktree, agent, feedback, context, validation, decision, scm, audit
2. Object Storage 主要承担 diff, build log, test log, agent transcript(大文件)
3. Audit 是唯一 Append-only Module
4. Search 是唯一 Projection 写入者
5. Permission / Identity 是横切,只读引用

---

## 接口稳定承诺(给后续阶段)

本文档作为基本设计書输出,以下接口将在后续阶段保持稳定(不会因详细设计而变更契约,除非 §15 Open Issue 解决):

1. **Domain 列表与依赖方向**(§2):保持 25 个 Module 划分(含 `domain-local-runtime` 集群外 Runtime 服务器侧 Registry / Port,见 §4.6.1 与 Local Daemon 二进制区分)
2. **聚合根与不变量**(§5.7):保持 10 个核心聚合根
3. **Context Priority 分级**(§4.4.4):P0-P4 五层
4. **Risk Signal 类型**(§4.8.5):8 种类型
5. **Worktree 状态机**(§7.1):17 个状态
6. **WorkItem 状态机**(§7.2):3 个默认(TODO/IN_PROGRESS/DONE)+ 扩展(§4.9.3 列出常见扩展示例,实际由 Project Policy 定义)
7. **Feedback 状态机**(§7.3):6 个状态
8. **AgentSession 状态机**(§7.4):14 个状态
9. **Decision 状态机**(§A.7):3 个状态
10. **NATS Subject 命名空间**(§5.5):`star.*` 前缀
11. **13 类 tenant_id 必带对象**(§6.1)
12. **Object Storage vs PostgreSQL 边界草案**(§5.1)
13. **AI Content Retention Policy 分级**(§6.8)
14. **ADR-016~030 决策**(§10):如变更需走 RFC
15. **MVP / V1 / V2 范围**(§13)

**可能因 PoC 校准的项**(§15):

- §4.4.4 Token Budget 具体值(J.3, J.6)
- §5.1 Object Storage 边界阈值(J.8)
- §4.7 Self-hosted Git 支持范围(J.10)
- §4.10.7 Prompt Injection 检测方式(J.15)

**给后续阶段的关键提示**:

- **API Design**:基于 §4 各 Module 接口签名(Port),设计 REST/WS/GRPC
- **Data Design**:基于 §5 + §附录 C,设计 DDL(本设计不输出 DDL)
- **Security Design**:基于 §6 + §4.10,设计 Security Implementation Plan
- **Runtime Design**:基于 §4.6,设计 Local Daemon 内部架构
- **Integration Design**:基于 §4.7,设计 SCM / Agent / Notification 具体 Adapter
- **AI/Agent Design**:基于 §4.2 + §4.4,设计 Agent Port 实现细节
- **Test Design**:基于 §11 PoC + §37 AC 示例 + §4.1.9 / §4.5.5 判定链,设计 E2E
- **Operation Design**:基于 §8,设计 K3s 部署 Manifest / Helm / Kustomize / GitOps

---

*文档结束。本文档为基本设计書阶段产出,后续团队据此继续制作外部設計 / 内部設計 / API Design / Data Design / Security Design / Runtime Design / Integration Design / AI・Agent Design / Test Design / Operation Design。*


## 11. arch-agent-graph-viewer 基本設計 (per ADR-0041 v0.1)

> **追加日**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **依据**: [ADR-0041-arch-agent-graph-viewer v0.1](../architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md) + [ARCH-AGENT-GRAPH-001-REPORT v0.1](../reports/ARCH-AGENT-GRAPH-001-REPORT.md) + 詳細設計 [spec §1-§10](../architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md)
> **位置付け**: 業務要件 §48 を受けた基本設計 (Phase 1 完了, Phase 2/3 待ち)

> **dual-use 提醒 (per AGENTS.md §5 + 2026-08-31 22:45 JST Q1-D 拍板)**: 本節で扱う "25 domain ノード" は Star 倉 22 `domain-*` crate DDD bounded context の投影, **RGS 5 域 (player/economy/match/social/admin) とは非対応**。5 域は RGS 倉歴史治理命名, 業務子域↔DDD マッピングは構築しない。

### 11.1 アーキテクチャ概要

3 層構造 (per 詳細設計 §1.1):

| 層 | コンポーネント | 状態 |
|---|---|---|
| **Layer 1** Frontend | KanbanCard (🕸 Arch) + ArchGraphModal (cytoscape) + types/graph.ts + mocks/handlers/graph.ts | 🟢 Phase 1 完了 |
| **Layer 2** API Gateway | `POST /api/graph/ensure-fresh` + `POST /api/graph/cypher` + `GET /api/graph/health` | 🟢 MSW 完了 / ⏳ Phase 2 実 backend |
| **Layer 3** Backend | `crates/star-graph-agent/` (GraphService + LlmAgentWorker + AdvisoryLock + FingerprintCalculator) | ⏳ Phase 2 |
| **Storage** | Memgraph (graph DB) + PostgreSQL (audit + RLS) | ⏳ Phase 3 |

### 11.2 ノードモデル (per ADR-0041 §2.1)

25 kind union, 1-hop 表示 = 11 kind, 2-hop code-side 限定表示 = 2 kind (cratemodule / symbol):

| 主要 kind | 表示色 | 形状 | サイズ (px) | hop_level |
|---|---|---|---|---|
| `work_item` (現) | cyan #00f0ff | round-rectangle | 64 | 1 |
| `work_item` (他) | #7c8499 | round-rectangle | 48 | 1 |
| `worktree` | purple #a78bfa | hexagon | 48 | 1 |
| `agent_session` | warn #f59e0b | diamond | 44 | 1 |
| `change_set` | info #10b981 | ellipse | 44 | 1 |
| `scm_repository` | ok #22c55e | round-triangle | 48 | 1 |
| `pull_request` | magenta #ec4899 | round-pentagon | 44 | 1 |
| `feedback` | err #f43f5e | octagon | 40 | 1 |
| `validation_case` | blue #3b82f6 | round-diamond | 40 | 1 |
| `comment` | slate #94a3b8 | tag | 36 | 1 |
| `identity` | sky #0ea5e9 | circle | 40 | 1 |
| `cratemodule` | ink #475569 | round-rectangle | 44 | 2 (code-side) |
| `symbol` | ink-dim #64748b | ellipse | 28 | 2 (code-side) |

### 11.3 エッジモデル (per ADR-0041 §2.1)

24 typed edge label, hop_level 1/2 区分:

| 区分 | 主要エッジ | 線色 | 幅 (px) | dash |
|---|---|---|---|---|
| 1-hop 業務 | ASSIGNED_TO / REPORTED_BY / IN_PROJECT / IN_WORKSPACE / ON_WORKTREE / PRODUCED / HAS_FEEDBACK / VALIDATED_BY / COMMENTED_ON / DESIGNED_BY / HAS_PR / WITH_PERMISSION / FOLLOWING_WORKFLOW | cyan #00f0ff | 2 | solid |
| 1-hop transitive | RUNS_ON / POWERS / TARGETS_BRANCH / WEBHOOK_FOR | cyan #00f0ff | 2 | solid |
| 2-hop code-side | REFERENCES / LIVES_IN / DEPENDS_ON / INHERITS_FROM | ink-mute #475569 | 1 | dotted, 30% opacity |

### 11.4 データモデル (DB 三類横展開, per 2026-09-01 18:30 JST 拍板)

| 物理名 | 種別 | 主キー | RLS | 役割 |
|---|---|---|---|---|
| `graph.graph_node` | **Master (M)** | `id UUID` | 13 類 | 25 kind 投影, SCD Type 2 |
| `graph.graph_edge` | **Master (M)** | `id UUID` | 13 類 | 24 kind 投影, SCD Type 2 |
| `graph.graph_fingerprint` | **Transaction (T)** | `id UUID` | 13 類 | append-only 監査ログ, 90 日 TTL |

> **Work (W) 類なし**: 短 TTL データは `agent.agent_session` で扱う, 本モジュールは長期保存/監査/参照専用

詳細: [data-design/ipa-detail/tables/graph_*.md](../data-design/ipa-detail/tables/) (3 表 T-NEW-001/002/003)

### 11.5 冪等・排他設計 (per ADR-0041 §2.2)

#### 11.5.1 冪等性 (5 層)

| 層 | 仕組み | 効果 |
|---|---|---|
| L1 クライアント | React Query `staleTime: 30_000` | 30s 以内重複 fetch skip |
| L2 バックエンド | `fingerprint = sha256(work_item_id + worktree_branch + worktree_sha + source + project_id)` | コード未変 = skip agent |
| L3 DB | `MERGE ... ON MATCH SET ... ON CREATE SET ...` (Cypher) | 既存ノード上書き, 新規作成 |
| L4 監査 | `graph_fingerprint` 履歴 append-only | 同 fingerprint でも実行時刻別行 |
| L5 LLM | `temperature=0`, `top_p=0.1`, `seed=work_item_id.hash()` | LLM 出力 deterministic |

#### 11.5.2 排他性 (5 層)

| 層 | 仕組み | TTL |
|---|---|---|
| L1 advisory lock | `pg_try_advisory_xact_lock(work_item_id_hash)` | 5 分 |
| L2 Redis | `SETNX graph:lock:{work_item_id} 1 EX 300` (任意) | 5 分 |
| L3 in-process coalesce | `pending[work_item_id] = oneshot::Receiver` | - |
| L4 失敗時 | lock 自動解放 (advisory_xact / SETNX) | - |
| L5 agent 状態 | `agent_session` 14 状態機で `failed/cancelled` 時即解放 | - |

### 11.6 フロントエンド実装 (Phase 1 完了)

| ファイル | 行数 | 役割 |
|---|---|---|
| `frontend/src/types/graph.ts` | 8.5KB | 25 ノード kind + 24 エッジ kind + 3 endpoint 契約 |
| `frontend/src/components/board/ArchGraphModal.tsx` | 21.1KB | modal + cytoscape 描画 + 1-hop 高亮 |
| `frontend/src/components/board/KanbanCard.tsx` | (+~20 行) | 🕸 Arch ボタン + e.stopPropagation |
| `frontend/src/components/board/KanbanBoard.tsx` | (+~10 行) | onArchClick prop 透伝 |
| `frontend/src/app/projects/ProjectsClient.tsx` | (+~5 行) | useArchGraphTrigger + ArchGraphModal 挂载 |
| `frontend/src/mocks/handlers/graph.ts` | 4.1KB | 3 endpoint MSW mock |
| `frontend/src/mocks/data/graph.ts` | 9.5KB | 1-hop 13 ノード + 2-hop 4 ノード fixture |
| `frontend/src/components/board/KanbanCard.test.tsx` | 4 tests | arch ボタン表示 + click stopPropagation |
| `frontend/src/mocks/__tests__/graph.test.ts` | 6 tests | handler 登録 + fixture 完全性 + orphan edge 検出 |

**守門 #1 実証**: tsc --noEmit 0 错, vitest 320/320 pass (per ARCH-AGENT-GRAPH-001-REPORT §2)

### 11.7 API 設計 (3 endpoint, per 詳細設計 §2)

| Method | Path | 用途 | 200 / 202 | 認証 |
|---|---|---|---|---|
| POST | `/api/graph/ensure-fresh` | 冪等+排他 trigger | 200 fresh / 202 running | Bearer JWT |
| POST | `/api/graph/cypher` | 1-hop 問合せ | 200 GraphPayload | Bearer JWT |
| GET | `/api/graph/health` | 健全性 | 200 / 503 | Bearer JWT |

### 11.8 モジュール配置 (Phase 2 計画)

```
crates/
└── domain-graph-agent/        # 第 23 個 domain crate (per ADR-0040 22 crate 平行)
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── port/              # GraphServicePort, LlmAgentWorkerPort, AdvisoryLockPort
    │   ├── domain/            # GraphPayload, GraphNode, GraphEdge, Fingerprint
    │   ├── service/           # GraphService (ensure_fresh + cypher_query + health)
    │   ├── infrastructure/    # MemgraphClient (Phase 3) + Postgres RLS adapter
    │   └── api/               # REST + MCP tool 露出
    └── docs/
```

### 11.9 段階計画 (per ADR-0041 §3)

| Phase | 内容 | token 予算 | 状態 |
|---|---|---|---|
| 1 | フロント契約 + MSW mock | 1.0M | **🟢 完了** (commit 4dd0df1 時点) |
| 2 | backend LLM worker + 冪等 + 排他 + agent-runtime 14 状態機 | 4.8M | ⏳ P3-B 拍板待ち |
| 3 | 実 memgraph + 25 schema + バックアップ | 2.0M | ⏳ Phase 2 完了後 |
| **計** | | **7.8M** | (per STAR-OLU-001 v0.1 1 SRE·週 = 1.2M, 約 6.5 週) |

### 11.10 既知の缺口 (per 缺标比错标, 守門 #11, 10 項)

| # | 缺口 | Phase 計画 |
|---|---|---|
| 1 | 実 memgraph 例未配備 | Phase 3 |
| 2 | LLM worker 未実装 | Phase 2 |
| 3 | 冪等 advisory lock 未実装 | Phase 2 |
| 4 | ノード click 遷移先未実装 | Phase 2+ |
| 5 | export PNG / SVG / JSON なし | Phase 2+ |
| 6 | cytoscape-cose-bilkent 公式 d.ts なし | 自作 `cytoscape-ext.d.ts` 兜底 |
| 7 | Symbol 詳細未表示 | Phase 2+ |
| 8 | Playwright 冒煙未実行 | Phase 2 |
| 9 | Agent 14 状態機との正式統合未実装 | Phase 2 |
| 10 | Worktree 状態変化 webhook 自動再生成未実装 | Phase 3+ |

### 11.11 トレーサビリティ

- 一次出典: ADR-0041 v0.1
- 業務要件: requirements.md §48 (REQ-ARCH-001~005)
- 詳細設計: spec/agent-api/arch-agent-graph-viewer.md v0.1 (11 段)
- データ設計: data-design/ipa-detail/tables/graph_*.md (3 表 T-NEW-001/002/003)
- Phase 1 報告: docs/reports/ARCH-AGENT-GRAPH-001-REPORT.md v0.1 (7 段)
- 関連 ADR: ADR-0027 (STAR IDE Gateway), ADR-0030 (Lease+Heartbeat+Resume), ADR-0040 (domain-batch 22 → 23 crate 拡張)

---

*本節 §11 は arch-agent-graph-viewer 機能追加 (2026-09-02 02:10 JST Ulysses "需求和基本设计, 詳細设计 補完" 発令) による。*
