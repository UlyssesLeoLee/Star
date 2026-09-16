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

- REQ-WI-001：WorkItem 支持自由文本分类属性 `Labels: Vec<String>` 与 `Components: Vec<String>`，两者均为可选、长度不限、按字符串精确匹配（不做标签层级、不做组件依赖推导）。Labels 用于跨 WorkItem 的横切分类（如 `bug`、`perf`、`regression`），Components 可选地对应 Repository / 模块 / 子系统（由 Project 决定语义约定）。Components 在 AI Task 场景下可作为 §8.1 AI Task "Repository Scope / Allowed Files" 的粗粒度前置（Repository Scope 的初步范围划定），但是否实际生效仍以 AgentPolicy / Worktree 授权边界为准（§24.3、§28）。**已实现**：`crates/domain-work-item/src/entity.rs:100, 103` 与 `src/lib.rs:116-117, 329` 定义 `labels: Vec<String>` 与 `components: Vec<String>` 字段；`src/service.rs:146-147` 在创建 WorkItem 时初始化为空 Vec。**已实现追溯，2026-08-26 补登记。**

### 8.2 Workflow

- REQ-WF-001：默认最简三态工作流（待办 / 进行中 / 完成），支持自定义状态扩展（前次对话结论：默认给出简化方案，不强制可视化工作流配置器，属于 MVP 精简范围，见第 30 章）。
- REQ-WF-002：Worktree Status 不等于 WorkItem Status（§4）。同一 WorkItem 下，Worktree A 可为 Agent Running，Worktree B 可为 Blocked，Worktree C 可为 Reviewing，系统必须允许该并存状态。
- REQ-WF-003：WorkItem 状态转换（transition）可配置 Guard，转换只有在 Guard 满足时才允许执行。Guard 类型至少包括：角色要求（RequireRole）、人工批准（RequireApproval）、Validation 通过（RequireValidation）。典型场景包括但不限于：（a）"Agent 未通过 Validation 不能自动流转到 Done"，对应 §27 AI Task 的 Validation Policy；（b）"需要人工 Approval 才能合并到 Done / Merged"，对应 §28 Agent Policy 的 Require Approval 授权级别。Guard 校验由 Application/Authorization 层强制执行（不得仅通过 Prompt 约束，§28，与 REQ-PERM-002 一致）。Guard 失败时返回可定位错误（哪个 Guard 不满足），便于 UI/CLI 给出可执行的下一步建议。**已实现**：`crates/domain-workflow/src/lib.rs:134-148` 定义 `enum Guard { RequireRole(String), RequireValidation(String), RequireApproval }`，第 618/626/634 行在状态转换执行时实际做校验；第 1384 行有使用示例。**已实现追溯，2026-08-26 补登记。**

### 8.3 Design Artifact（无对应原提示词章节编号 — 本节为线程 C 新增设计, P0：DSG-001/002 — brainstorming 线程 C，覆盖瀑布式 SIer 项目中"设计先行"的诉求）

前两个线程（A：核心开发闭环，B：Review）都假设代码已经在写。瀑布式项目在写代码之前有一个独立的、需要正式批准才能往下走的阶段——设计书。系统不得强迫所有 Project 都走瀑布流程，但必须支持"设计书是先于 ChangeSet 存在、且需要独立 Approval Gate 才能放行"的 Project。不新建平行的"设计管理系统"，而是把设计书表达为一种可挂接到既有 WorkItem 状态机（§8.2 REQ-WF-003）与既有 ReviewRecord（§27.4）机制上的工作产出物：

```text
DesignArtifact
├── ArtifactId / ProjectId / WorkItemId（关联 Epic/Story，非强制关联单个 Task）
├── Kind: BasicDesign | ExternalDesign | InternalDesign | APIDesign | DataDesign
       | SecurityDesign | RuntimeDesign | IntegrationDesign | AIAgentDesign
       | TestDesign | OperationDesign
      （枚举值取自本文档末尾"下一阶段清单"已列出的瀑布阶段名称，不新造分类体系）
├── Version（设计书可迭代，历史版本须可追溯，不得覆盖式修改已批准版本）
├── Status: DRAFT → IN_REVIEW → APPROVED / REJECTED / SUPERSEDED
├── Content: 不规定具体格式/模板（文档或结构化字段留给《基本设计书》阶段决定，本层只定义生命周期与关联关系）
└── ApprovalReviewId: ReviewRecord（§27.4，Target 从"仅 ChangeSet"泛化为"ChangeSet | DesignArtifact"，Kind 通常为 CrossReview）
```

**与既有对象的关系（不新增平行体系）**：
- DesignArtifact 的批准流程复用 §27.4 ReviewRecord，不新建"设计评审"专属状态机；ReviewRecord 的 `ChangeSetId` 字段泛化为可选，改为对 DesignArtifact 或 ChangeSet 二选一关联（同一时刻只挂一种 Target，Review 的 Kind/Decision/Findings 语义不变）。
- WorkItem 状态转换（§8.2 REQ-WF-003）可增加 Guard 前置条件"关联的 DesignArtifact 必须为 APPROVED"，用既有 `RequireApproval` Guard 类型表达，不新增 Guard 类型。典型场景："Epic 下的 Story 不得进入 In Progress，除非其 Basic/External/Internal Design 均已 APPROVED"——由 Project 自行配置是否启用该 Guard（非强制瀑布，敏捷 Project 可完全不用）。
- DesignArtifact 不属于 DevelopmentExecution（§20），它先于 ChangeSet/Worktree 存在；DesignArtifact APPROVED 之后才允许对应 Worktree 创建（如 Project 选择启用该约束），二者关系属于 Guard 前置，不是新的执行层对象。
- REQ-DSG-001：系统必须支持为 WorkItem（通常为 Epic/Story 级）关联 0..N 个 DesignArtifact，并跟踪每个 DesignArtifact 的独立 Status 与 Version 历史。
- REQ-DSG-002：系统必须支持将"关联 DesignArtifact 全部 APPROVED"设置为既有 WorkItem 状态转换 Guard（§8.2 REQ-WF-003）的前置条件，Guard 失败时明确指出哪些 DesignArtifact 未批准。

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
- REQ-PLAN-007：Milestone（里程碑）用于对一组 WorkItem 打分组标签并设定共同 `due_date`，Roadmap 是基于 Milestone 的只读 Projection 视图（按时间线聚合 Milestone 及其下属 WorkItem 的进度）。Milestone 字段至少包括：`id / tenant_id / project_id / name / description / due_date / status / work_item_ids / created_at`，不携带发布/上线语义。**与 Jira Fix Version 的差异点**：当前 Milestone 不含 `release_date` / `released` 标记，不追踪"哪个 PR / Worktree 落地到了哪次发布"；如后续需做"agent 产出的 PR 属于哪次发布"这类追溯，需另开需求（如 REQ-PLAN-008），不在本条登记范围内。**已实现**：`crates/domain-planning/src/lib.rs:335-346` 定义 `struct Milestone`；`docs/api-design.md:429` 与 §3 端点暴露 `GET /v1/projects/{id}/roadmap`（R 投影），`/v1/projects/{id}/milestones` 系列端点由 `domain-planning` 提供 CRUD。**已实现追溯，2026-08-26 补登记。**

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
- REQ-AUTO-002（V1 候选，参考竞品 Multica「Autopilot」分析，2026-08-26 补充）：Trigger 除事件订阅（`event_type + filter`）外，须支持 Schedule/Cron 类型，用于定时 Standup / Audit / Report 等主动巡检场景，不得与事件触发混用同一执行路径（避免循环触发歧义，沿用 REQ-AUTO-001 的 Rule 聚合根，仅扩展 `Trigger` 枚举）。
- REQ-AUTO-003（V1 候选）：系统需支持对多个 WorkItem 的批量操作，至少包括**批量状态转换**（Bulk Transition）、**批量分配**（Bulk Assign）、**批量取消**（Bulk Cancel）。批量操作的输入为 WorkItem ID 列表（或 Filter 表达式结果集）+ 目标动作；输出为逐条结果（成功/失败/原因），整体操作是**部分成功**语义（不要求全成功才返回），调用方可基于结果列表做重试或回滚。**关键约束**：批量操作中的**每一条**仍须独立经过 REQ-WF-003 定义的 Guard 校验（角色 / Validation / Approval），**不得绕过**单条转换的授权检查（即使操作由 Automation Rule 触发）；同样，单条权限不足时该条失败但不影响其他条。典型 AI 开发管理场景：（a）一次性为某 Epic 拆出的 N 个 AI Task 批量分配 Agent Policy；（b）当某上游 Decision（§26.5）被否决时，批量取消该 Decision 下所有还在 Queued / In-Progress 状态的 Agent Task；（c）批量将一组已解决 WorkItem 标记为 Archived。**未实现**：当前 `crates/` 全代码库无 `bulk` / `batch` 关键字命中（`\bbulk\b|\bBulk\b|\bbatch\b|\bBatch\b` 零命中），属真实功能缺口，V1 候选。

---

## 12. Notification & Search 要求

- REQ-NOTIF-001：事件触发的邮件/站内通知，覆盖 WorkItem 状态变更、Feedback 请求、Validation 失败等（详见第 25、27 章事件源）。
- REQ-NOTIF-002（参考竞品 Multica「Inbox 降噪」分析，2026-08-26 补充）：Notification/Inbox 默认策略是"仅在需要人类决策的节点触达"（如 WAITING_FEEDBACK、Validation 失败、Protected Action 待授权），而非 Agent 每一次工具调用/中间步骤都产生通知；中间过程仍需 100% 写入 AgentSession Transcript（INV-AGT-09 对应，见第 24 章）供按需查阅，二者不冲突。
- REQ-NOTIF-003：WorkItem 支持 Watcher（关注者）列表，**用户可自行加入/退出**对特定 WorkItem 的关注；Watcher 收到的通知**不受** REQ-NOTIF-002 全局降噪策略限制（即即使该 WorkItem 不满足"需要人类决策"触发条件，Watcher 仍会收到关键事件通知，如状态转换、Comment、Feedback 产生、Validation 失败、Merge / Close 等）。这是 REQ-NOTIF-002 默认行为之外的可选补充机制，**不改变** REQ-NOTIF-002 的默认行为：非 Watcher 用户仍只收到降噪后的关键通知。典型场景：人类想专门盯某个 Agent 正在处理的高风险 AI Task（即使其状态不满足降噪触发条件），或想关注某个 Project 关键路径上所有 WorkItem 的进度。实现位置在 `domain-notification`，与现有 Notification 通道（inbox / email / IM）共用投递通道；Watcher 列表变更本身应写 audit（谁在何时关注/取消关注了哪个 WorkItem）。
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
- REQ-SCM-003（V2 候选，解决 J-SCM-01 未决问题，参考竞品 Multica「Any Git host / Self-hosted included」定位，2026-08-26 补充）：自建 Git（Gitea / Forgejo）企业场景优先于 Bitbucket / Azure DevOps 排期，理由是已有 ACL 层完成厂商对象隔离（REQ-SCM-002），新增 Adapter 边际成本低于新建领域模型；仍不改变 §47 "系统不承担完整 Git Server 职能"的边界。

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

允许进入 `READY_FOR_REVIEW` 前至少考虑：No Critical Feedback、Required Tests Pass、Required Build Pass、No Blocking Conflict、Acceptance Criteria Covered、Required Review Complete、Git State Known。具体策略由 Project Policy 定义。"Required Review Complete" 的实证来源见 §27.4 ReviewRecord（`Status=APPROVED` 且关联 `ValidationResult(Type=Review)` 存在）。

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
├── TokenUsage / CostSummary（V1 候选，参考竞品 Multica「per-run token 成本可见性」分析，2026-08-26 补充；对应第 30.3 章 Context Cost Analysis 扩展，非新增独立能力）
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

AgentPolicy 至少研究：Allowed Repository、Allowed Worktree、Allowed Path、Allowed Tool、Allowed Command Category、Network Access、Secret Access、Max Runtime、Max Context、Max Change Scope、Require Review、Require Test、Require Approval。**Policy 必须由 Application / Authorization 层执行**，重要安全规则不能只靠 Prompt 告诉 Agent"不要修改 xxx"（§28）。`Require Review` 展开为 `ReviewerKind: SelfOnly | CrossHumanRequired | AgentAssistedAllowed` 与 `MinReviewers`，落地对象见 §27.4 ReviewRecord。

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

**Agent-Assisted Review**（§27.5 补充，与 Agent Comparison 明确区分）：一个 Agent 对另一 Agent 的 ChangeSet 执行只读审查、产出 Feedback/ValidationResult，属于既有 Auto 授权层级（§24.4），**不算** Agent Swarm / Agent Negotiation，因为 Reviewer 与 Author 之间零直接通信，且 ReviewRecord 的触发权始终在人类/Policy 手中（§24.7 同一边界）。

### 24.6 Skill / Playbook 复用（V2 候选，参考竞品 Multica 分析，2026-08-26 补充）

Multica 将"解决过一次的问题"沉淀为可复用 Playbook，供其他 Agent 复用。本系统目前仅有 `AgentPolicyTemplate`（权限模板），缺少"任务经验模板"维度。V2 候选方向：

- Skill/Playbook 是**只读**的 Context 素材（Instruction + 参考 Diff/Decision），挂载到 Context Compiler（第 26 章 Context Packet 生成流程），不是可执行代码，不获得独立权限。
- Skill/Playbook 与 Agent Policy 是正交概念：前者影响 Prompt/Context 内容，后者由 Application 层强制执行边界（§28），二者不得混淆，禁止通过 Playbook 绕过 REQ-PERM-002。
- 安全上须视为 Untrusted Content 同一优先级处理（§41，第 28.3 章 Prompt Injection 威胁），来源于 Repository 或第三方共享的 Playbook 不得高于 Trusted Human Policy 的 Instruction Priority。

### 24.7 Squad / 团队分组视图（Future 候选，参考竞品 Multica 分析，2026-08-26 补充）

Multica 提出"Squad"（Agent + 人类混编小队，Leader 路由任务）。本系统采纳其中**分组可见性**价值，明确排除其"自治协商"含义：

- Squad 仅作为 WorkItem/Worktree 维度的 Assignee 分组展示（谁负责、谁在跑哪个 Worktree），不引入 Agent 间自主任务分派或协商机制。
- 必须与 §51、INV-AGT-10 的既有边界一致：**禁止** Agent Swarm / Agent Negotiation / Autonomous Planning Society（第 30.6 章 Explicit Non-Goals 不变）。
- 若要落地，归入 Future（第 30.5 章），且实现方式是"人类或规则引擎指定 Assignee"，而非"Agent 自己决定谁来做"。

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

### 27.4 Review Record 领域对象（无对应原提示词章节编号 — 本节为线程 B 新增设计, P0：RVW-001/002 — brainstorming 线程 B，per Ulysses "自审交叉审核" 拍板）

`Review`（§27.1 既有 ValidationResult Type 之一）目前只是一个"通过/不通过"的校验类型值，缺少审核人身份、自审/交叉区分、结论追溯的第一级对象。禁止继续只用一个枚举值代表审核。字段至少包括：

```text
ReviewRecord
├── ReviewId / WorktreeId / WorkItemId
├── Target: ChangeSet | DesignArtifact（二选一关联，同一时刻只挂一种；DesignArtifact 分支为 §8.3，线程 C 泛化，ChangeSetId 不再是唯一挂接字段）
├── Kind: SelfReview | CrossReview | AgentAssistedReview
├── Author: HumanIdentity | AgentSession（被审对象的归属者：Target=ChangeSet 时为提交者，Target=DesignArtifact 时为起草者，§8.3）
├── Reviewer: HumanIdentity | AgentSession（审核执行者）
├── Checklist: ReviewChecklistItem[]（来自 Project 级 ReviewPolicy 模板，复用 §24.6 AgentPolicyTemplate 同类机制，不新发明模板概念）
├── Findings: ReviewFinding[]（每条可转化为 Feedback，Target=Review Finding，§25.1 既有类型，无需扩展）
├── Status: DRAFT → IN_PROGRESS → APPROVED / CHANGES_REQUESTED / REJECTED / SUPERSEDED
├── Decision: Approve | RequestChanges | Reject
├── Evidence: ValidationResult[]（Type=Review，关联本 ReviewRecord，§27.1）
├── StartedAt / CompletedAt
└── TriggeredBy: AgentPolicy.RequireReview | Project ReviewPolicy | Human Manual
```

**Kind 判定规则**：`Reviewer == Author` → 必须标记 `SelfReview`；`Reviewer != Author` 且 Reviewer 为人类 → `CrossReview`；`Reviewer != Author` 且 Reviewer 为 AgentSession → `AgentAssistedReview`（见 §27.5 边界约束，禁止与 §24.5/§24.7 既有边界冲突）。

**与既有对象的关系**（不新增平行体系，全部挂接既有闭环）：
- ReviewRecord 的每条 Finding → 走既有 Feedback 状态机（`OPEN → ACKNOWLEDGED → APPLIED → VERIFIED/REJECTED/SUPERSEDED`，§25.3），不新建 Finding 专属状态机
- ReviewRecord 完成后必须产生至少一条 `ValidationResult(Type=Review)`（§27.1），作为 §22.7 Worktree Completion 判定"Required Review Complete"条件的实证来源（§22.7 原文仅提及条件名，未定义来源对象，本节补齐）
- ReviewRecord 挂接 Acceptance Coverage（§27.2）：`AC-xxx → ValidationEvidence` 映射中，`Human Review RV-12` 类证据即为 ReviewRecord 实例，非独立编号体系

### 27.5 Self-Review / Cross-Review / Agent-Assisted Review 的边界（无对应原提示词章节编号 — 本节为线程 B 新增设计）

- **Self-Review**（自审）：Author 自己在提交前走一遍 Checklist，不引入第二身份，不受 §24.4/§24.5 授权约束影响，属于最轻量 Gate。
- **Cross-Review**（交叉审核）：Reviewer 必须是与 Author 不同的人类身份（Segregation of Duties），Reviewer 的 `Reject` 决策等价于 §24.4 表中的 "Require Approval" 级别，必须经 Human/Policy Gate 才能放行到 `READY_FOR_COMMIT`（呼应 §27.3 流程）。
- **Agent-Assisted Review**（原始诉求"Agent 之间 QA"的落地形态）：Reviewer 是一个独立 AgentSession，对另一 Worktree/ChangeSet 执行只读分析并产出 Findings/ValidationResult。**这不是新的授权层级**——Review 输出即 Feedback（§25.1）与 ValidationResult（§27.1），二者均已属于 §24.4 表中 "AI Analyze / AI Suggest = Auto" 层级，Agent-as-Reviewer 不需要修改任何 Worktree、不触碰 Commit/Push/Merge，因此不产生新的授权空缺。
- **禁止事项**（与 §24.5/§24.7/§30.6 既有边界保持一致，不得放宽）：
  - Reviewer AgentSession 不得与 Author AgentSession 直接通信协商结论；所有交互必须经过 Feedback 状态机，不构成 Agent Negotiation（§24.5 Non-Goal）
  - ReviewRecord 的创建时机与 Reviewer 指派，必须来自 AgentPolicy.RequireReview 或 Project ReviewPolicy 或人类手动触发，**不得由 Agent 自主发起对其他 Agent 的审查**（呼应 §24.7 "人类或规则引擎指定，而非 Agent 自己决定"）
  - Agent-Assisted Review 的 `Reject` 决策不得自动阻断 Worktree 生命周期；必须仍经过 §24.4 Human/Policy Gate 才能生效，避免"Agent 审核 Agent"形成无人类介入的自治闭环

### 27.6 Test Level（工程别テスト，无对应原提示词章节编号 — 本节为线程 C 新增设计, P0：TST-001 — brainstorming 线程 C）

§27.1 的 ValidationResult Type 列表（Unit Test / Integration Test / Acceptance Check）回答的是"验证了什么种类的东西"，瀑布式 SIer 项目还需要回答一个正交问题——"这次验证处于哪个测试工程"（単体/結合/総合/受入）。这是粒度不同的两个维度，不是要新建一套 TestPlan/TestCase 平行对象体系：

```text
ValidationResult（§27.1 既有对象，本节仅新增一个字段维度）
├── Type: Build | Unit Test | Integration Test | Lint | Format | Static Analysis
       | Security Check | Acceptance Check | Review | Custom Validation（既有，不变）
└── Level: UnitTestLevel | IntegrationTestLevel | SystemTestLevel | AcceptanceTestLevel
      （新增字段，对应単体テスト/結合テスト/総合テスト/受入テスト；
       与 Type 正交——例如 Type=Integration Test 的一次验证既可能属于
       IntegrationTestLevel，也可能是更大范围 SystemTestLevel 演练的一部分）
```

**与既有对象的关系（不新增平行体系）**：
- 不引入独立的 TestPlan/TestCase 对象；`Level` 是 ValidationResult 的字段，不是新实体。理由：ValidationResult 已经关联 WorkItem/AcceptanceCriterion/Worktree/AgentSession/ChangeSet/Commit（§27.1），新建 TestCase 会制造第二条平行的证据链，与 §27.4 line 924 "不新增平行体系，全部挂接既有闭环"的既定原则冲突。
- §27.2 Acceptance Coverage 的 `AcceptanceCriteria → ValidationEvidence` 映射须能按 Level 筛选（例：`AC-001` 要求必须同时存在 IntegrationTestLevel 与 AcceptanceTestLevel 两条证据，而不是任意一条 Validation Passed 即视为满足）——这是对既有映射表达能力的扩展，不是新增映射体系。
- SystemTestLevel（総合テスト）通常跨多个 WorkItem，其 ValidationResult 允许关联多个 WorkItem/ChangeSet（既有对象的多对多关联能力，非新语义）。
- REQ-TST-001：系统必须支持 ValidationResult 携带 Level 字段（単体/結合/総合/受入四档），并支持按 Level 聚合查看某 WorkItem/Project 的测试覆盖状态。
- REQ-TST-002：Acceptance Coverage 映射（§27.2）必须支持声明"某 AcceptanceCriteria 需要哪些 Level 的证据才算覆盖"，缺失特定 Level 时须在 UI/CLI 明确指出缺口，而非笼统显示"未覆盖"。

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

### 29.1 Incident Record（生产事件追溯，无对应原提示词章节编号 — 本节为线程 C 新增设计, P0：OPS-001 — brainstorming 线程 C）

**边界声明（先于对象定义，避免与 §30.6 冲突）**：本节只解决"生产事件如何被记录、追溯回是哪个 WorkItem/ChangeSet 造成、修复后如何验证"，这是 Jira-class 闭环（§30.1）在时间轴上的延伸，不是新增能力。系统**不**监控生产环境、**不**接收/处理告警信号、**不**执行自动回滚或自动修复、**不**获得生产系统的运行时访问权限——这些如果做了就是在做 §30.6 明确排除的 `Autonomous Production Deployment` 类能力的邻接功能，必须避免。IncidentRecord 是人工登记的追溯对象，事件本身的探测/告警交给外部 Monitoring/Alerting 系统（不在本产品范围内，只接受人工或既有 Webhook 转发登记的既成事实）。

```text
IncidentRecord
├── IncidentId / ProjectId / Severity（Project 自定义分级，不规定具体档位）
├── DetectedAt / ReportedBy（人工登记，或外部系统通过受限 Webhook 转发的既成事实，非本产品主动探测）
├── Status: OPEN → INVESTIGATING → ROOT_CAUSE_IDENTIFIED → FIX_IN_PROGRESS
       → RESOLVED → POSTMORTEM_DONE / WONT_FIX
├── LinkedWorkItem: WorkItem（修复工作在既有 WorkItem/Worktree/ChangeSet 闭环内完成，不新建修复流程）
├── RootCauseChangeSet: ChangeSet[]（可选，指向被认为引入问题的历史 ChangeSet，§21.1）
├── ViolatedAcceptanceCriteria: AcceptanceCriteria[]（可选，指出事件暴露了哪条 AC 实际未被覆盖，§27.2）
├── ResolutionEvidence: ValidationResult[]（修复后的验证证据，复用 §27.1，不新建证据体系）
└── PostmortemNote：自由文本，不规定模板（模板留给后续团队按 SIer 惯例定义）
```

**与既有对象的关系（不新增平行体系）**：
- 事件的修复不走独立流程：一旦 IncidentRecord 关联了 LinkedWorkItem，后续修复完全走既有 WorkItem → Worktree → AgentSession → ChangeSet → ValidationResult → ReviewRecord 闭环（§20-27），IncidentRecord 只是这条闭环之前的"为什么要开这个 WorkItem"的追溯挂钩，类似 Feedback（§25.1）挂接到 WorkItem 的方式。
- ViolatedAcceptanceCriteria 字段回填 §27.2 Acceptance Coverage：如果事件证明某条 AC 的既有 ValidationEvidence 不足以真正保证质量，必须能在 Coverage 映射上看到"这条 AC 曾经被事件击穿过"，为后续补充 Level（§27.6）或 Review（§27.4）要求提供依据。
- REQ-OPS-001：系统必须支持登记 IncidentRecord 并关联到 0..N 个 WorkItem，用于追溯"生产问题 → 根因 ChangeSet → 修复 WorkItem → 验证证据"的完整链条。
- REQ-OPS-002：系统必须允许 IncidentRecord 反向标注哪些 AcceptanceCriteria 被证明覆盖不足，但不得自动修改历史 ValidationResult 或 Acceptance Coverage 的既有判定（保留历史事实，新增标注而非覆写）。
- REQ-OPS-003（边界，与 §30.6 对齐）：系统不得实现生产环境探测、告警接收处理、自动回滚、自动修复能力；IncidentRecord 的创建只能来自人工输入，或经既有 §18 Integration Webhook 机制转发的、明确声明来源的外部登记，不新增独立的入站接口。

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
Self-Review Gate / Cross-Review Assignment（REQ §27.4-27.5，per brainstorming 线程 B 拍板"自审交叉审核核心化"）
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
Context Cost Analysis（含 Agent Session Token/Cost 明细，§24.1 补充）
Scheduled Automation Trigger（Autopilot 型 Cron 触发，REQ-AUTO-002）
Agent-Assisted Review（Policy-Enforced Review Pass，REQ §27.4-27.5，仅只读分析 + Feedback/ValidationResult 输出，不引入新授权层级）
Design Artifact + Approval Guard（REQ-DSG-001/002，§8.3，非强制瀑布——由 Project 自行启用，敏捷 Project 可完全不用，故不列入 §30.2 Must Have）
Test Level 维度（単体/結合/総合/受入，REQ-TST-001/002，§27.6，ValidationResult 既有字段扩展，非新对象）
Incident Record 追溯（REQ-OPS-001/002，§29.1，仅追溯既有 WorkItem→Worktree→ChangeSet→ValidationResult 链，不含监控/告警/自动回滚，见 §30.6）
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
Skill / Playbook Library（REQ §24.6）
Self-hosted SCM：Gitea / Forgejo（REQ-SCM-003）
```

### 30.5 Future（§68）

```text
Agent Swarm / Autonomous Task Decomposition / Autonomous Multi-Agent Scheduling
Graph Database / Vector Database / Semantic Repository Memory
Cloud IDE / Managed Git Hosting
Autonomous Merge / Autonomous Deployment
Squad / 团队分组视图（人类指定 Assignee，非自治协商，REQ §24.7）
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
| RISK-031 | Skill/Playbook Content Injection（第 24.6 章，参考竞品 Multica 分析，2026-08-26 补充） |

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

ARCH-OBL-DEV-007  Review Segregation of Duties（无对应原提示词章节编号 — per §27.4-27.5 新增）
  → Cross-Review 的 Reviewer 不得等于 Author；Agent-Assisted Review 不得因"审核"身份获得超出既有 Feedback/ValidationResult（Auto 层级）以外的额外权限，Reject 决策不得绕过 Human/Policy Gate 自动阻断 Worktree 生命周期。
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
| UC-DEV-013 | Project 配置 WorkItem 状态转换 Guard，要求关联 DesignArtifact 先 APPROVED（§8.3，线程 C） |
| UC-DEV-014 | Reviewer 对 DesignArtifact 执行 CrossReview 并 Approve/RequestChanges（§8.3、27.4，线程 C） |
| UC-DEV-015 | Developer 按 Level（単体/結合/総合/受入）查看某 WorkItem 的测试覆盖缺口（§27.6，线程 C） |
| UC-DEV-016 | 运维人员登记 IncidentRecord 并关联到修复 WorkItem，追溯根因 ChangeSet（§29.1，线程 C） |

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
→ Review Record → Validation Evidence → Commit → PR / MR → Acceptance
```

`Review Record`（§27.4，per 线程 B 拍板补入）插在 Feedback 与 Validation Evidence 之间：ReviewRecord 消费 ChangeSet + 已有 Feedback，产出新的 Finding（回流成 Feedback）与 `ValidationResult(Type=Review)`（汇入 Validation Evidence），不打断原有追踪链方向。

`Design Artifact`（§8.3，per 线程 C 拍板补入）挂在链条最前端，`Business Requirement → WorkItem` 之后、`Worktree` 之前：DesignArtifact APPROVED（经 §27.4 ReviewRecord 批准）可作为 WorkItem 状态转换 Guard（§8.2 REQ-WF-003）的前置条件，非强制串接，Project 可选择不启用。`Incident Record`（§29.1，per 线程 C 拍板补入）挂在链条末端 `Acceptance` 之后，反向指回 `WorkItem`/`ChangeSet`/`Acceptance Criteria`，形成"生产事件 → 根因 → 修复 → 再验证"的回溯支线，不改变原有正向链条方向。`Validation Evidence` 的 Level 维度（§27.6，per 线程 C 拍板补入）是对既有节点的字段扩展，不新增链条节点。

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
| `WI-xxx` | WorkItem 属性 Requirement（Labels / Components 等 WorkItem 字段语义，§8.1） |
| `RVW-xxx` | Review Requirement（自审 / 交叉审核 / Agent-Assisted Review，第 27.4-27.5 章，per brainstorming 线程 B） |
| `DSG-xxx` | Design Artifact Requirement（设计书生命周期与批准 Guard，第 8.3 章，per brainstorming 线程 C） |
| `TST-xxx` | Test Level Requirement（単体/結合/総合/受入 Level 维度，第 27.6 章，per brainstorming 线程 C） |
| `OPS-xxx` | Incident Record Requirement（生产事件追溯，第 29.1 章，per brainstorming 线程 C） |

### 41.2 关键 P0 Requirement 登记表（§63）

| ID | 内容 | 对应章节 | 对应 Architecture Obligation |
|---|---|---|---|
| WT-001 | 系统必须能够注册并跟踪与 WorkItem 关联的 Worktree | 第 22.1 章 | ARCH-OBL-DEV-001 |
| WT-002 | 系统必须能够区分 Worktree Server Metadata 与 Local Observed State | 第 22.1、23.3 章 | ARCH-OBL-DEV-006 |
| WT-003 | 系统必须能够查看多个 Worktree 的开发状态 | 第 22.3 章 | ARCH-OBL-DEV-001 |
| AGT-001 | 系统必须将 AgentSession 与 Worktree 关联 | 第 24.1 章 | ARCH-OBL-DEV-001 |
| AGT-002 | 系统不得允许 Agent 越过授权 Worktree 执行受保护修改 | 第 24.3-24.4 章 | ARCH-OBL-DEV-001 |
| FBK-001 | 用户必须能够向 WorkItem/File/Symbol/Diff/Test 等目标发送结构化 Feedback | 第 25.1 章 | ARCH-OBL-DEV-002 |
| WF-003 | WorkItem 状态转换必须可配置 Guard（角色/Validation/Approval），由 Application/Authorization 层强制执行 | 第 8.2 章 | ARCH-OBL-DEV-001/002 |
| FBK-002 | 系统必须能够追踪 Feedback 是否被 Agent 消费、应用和验证 | 第 25.3 章 | ARCH-OBL-DEV-002 |
| CTX-001 | 系统必须能够根据任务自动生成 Context Packet | 第 26.1 章 | ARCH-OBL-DEV-002 |
| CTX-002 | Context Packet 必须保留来源追踪信息 | 第 26.3 章 | ARCH-OBL-DEV-002 |
| VAL-001 | Agent 完成状态不能仅以 Agent 自我报告作为依据 | 第 27.3 章 | ARCH-OBL-DEV-005 |
| RVW-001 | 系统必须在 Worktree 进入 `READY_FOR_REVIEW` 前提供 Self-Review Checklist Gate | 第 22.7、27.4-27.5 章 | ARCH-OBL-DEV-005/007 |
| RVW-002 | 系统必须支持 Reviewer ≠ Author 的 Cross-Review 指派与 Approve/RequestChanges/Reject 决策记录 | 第 27.4-27.5 章 | ARCH-OBL-DEV-007 |
| SCM-001 | GitHub / GitLab 必须通过统一 SCM Adapter 接入 | 第 19.1 章 | ARCH-OBL-DEV-003 |
| LRT-001 | Local Runtime 必须经过身份认证和设备授权 | 第 23.2 章 | ARCH-OBL-DEV-004 |
| LRT-002 | SaaS 不得获得任意本地 Shell 执行能力 | 第 23.2 章 | ARCH-OBL-DEV-004 |
| SEC-xxx | 必须防止 Cross-Tenant / Cross-Repository / Cross-Worktree Context Leakage | 第 16、28.3、34 章 | ARCH-OBL-DEV-001/002 |
| DSG-001 | 系统必须支持为 WorkItem 关联 0..N 个 DesignArtifact，并跟踪独立 Status 与 Version 历史 | 第 8.3 章 | ARCH-OBL-DEV-001 |
| DSG-002 | 系统必须支持将"关联 DesignArtifact 全部 APPROVED"设为既有 WorkItem 状态转换 Guard 的前置条件 | 第 8.2、8.3 章 | ARCH-OBL-DEV-001 |
| TST-001 | 系统必须支持 ValidationResult 携带 Level 字段（単体/結合/総合/受入），并按 Level 聚合测试覆盖 | 第 27.6 章 | ARCH-OBL-DEV-005 |
| OPS-001 | 系统必须支持登记 IncidentRecord 并关联到修复 WorkItem，追溯"生产问题 → 根因 ChangeSet → 修复 → 验证证据" | 第 29.1 章 | ARCH-OBL-DEV-002/005 |

本文档第 1-17 章新增的基础 Requirement（`REQ-TWP-xxx / REQ-WF-xxx / REQ-PLAN-xxx / REQ-COLLAB-xxx / REQ-PERM-xxx / REQ-AUTO-xxx / REQ-NOTIF-xxx / REQ-SEARCH-xxx / REQ-DATA-xxx / REQ-RT-xxx / REQ-SEC-xxx / REQ-AUDIT-xxx / REQ-WI-xxx`）与 Vibe Coding 扩展 P0 Requirement 共同构成完整 ID 登记表，下游《基本设计书》须逐项继承。

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

（本图为主链简化版，完整链条含 Review Record / Design Artifact / Incident Record 分支，见 §39）

```text
(Design Artifact，可选前置，§8.3)
↓
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
Review Record（§27.4）
↓
Validation
↓
PR/MR
↓
(Incident Record，可选回溯分支，反向指回 WorkItem/Change/Validation，§29.1)
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
Requirement ID（第 41 章登记表，含 REQ-xxx / DEV-xxx / WT-xxx / AGT-xxx / FBK-xxx / CTX-xxx / VAL-xxx / SCM-xxx / LRT-xxx / SEC-xxx / RVW-xxx / DSG-xxx / TST-xxx / OPS-xxx）
Architecture Obligation（第 35 章 ARCH-OBL-DEV-001~007，及原有 ARCH-OBL 登记表）
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
Validation Model（第 27 章，含 Review Record 第 27.4-27.5 章、Test Level 第 27.6 章）
SCM Integration Contract（第 18-19 章）
Design Artifact Model（第 8.3 章，含批准 Guard 与 ReviewRecord 挂接关系）
Incident Record Model（第 29.1 章，须与 §30.6 Non-Goals 边界声明一并继承）
```

《基本设计书》阶段建议输入清单还应包括：Persona 与 Use Case 清单（第 3、36 章）、Acceptance Criteria 示例集（第 37 章）、Traceability Model（第 39 章）、决策表 A-O（第 46 章）、以及本文档第 0 章列出的与原文档待核对项。

---

*文档结束。本文档为要件定义阶段产出，后续团队据此继续制作基本設計 / 外部設計 / 内部設計 / API Design / Data Design / Security Design / Runtime Design / Integration Design / AI・Agent Design / Test Design / Operation Design。*


## 48. Architecture Agent Graph Viewer 要件 (per ADR-0041 v0.1, 2026-09-02 拍板)

> **追加日**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **依据**: [ADR-0041-arch-agent-graph-viewer v0.1](../architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md) + [ARCH-AGENT-GRAPH-001-REPORT v0.1](../reports/ARCH-AGENT-GRAPH-001-REPORT.md)
> **ステータス**: Phase 1 完了 (frontend 契約 + MSW mock 実裝), Phase 2/3 は token 拍板待ち

> **dual-use 提醒 (per AGENTS.md §5 + 2026-08-31 22:45 JST Q1-D 拍板)**: 本節で扱う "25 domain 節点" は Star 倉 22 `domain-*` crate DDD bounded context の投影, **RGS 5 域 (player/economy/match/social/admin) とは非対応**。5 域は RGS 倉歴史治理命名, 業務子域↔DDD マッピングは構築しない。

### 48.1 背景・動機 (per 2026-09-02 00:33 JST)

Star 倉 22 `domain-*` crate (per ADR-0040) + 25 MRU (per api-design.md §2.1) が複雑に連携し, 業務者が「ある WorkItem がシステム全体のアーキテクチャのどこに位置するか」を把握することが困難。Kanban カードから 1 クリックで cypher 図を表示し, 1-hop 隣人ノードとエッジを高亮, 2-hop code-side は 20% opacity で弱化する。

### 48.2 業務要件 (5 件)

#### REQ-ARCH-001: Kanban カードに Arch ボタン必須

- **業務価値**: 業務者がタスクから即座にシステム全体での位置関係を把握
- **要件**:
  - Kanban カードに 🕸 Arch icon ボタン (lucide Network) を第 4 行 (priority + assignee) 旁に配置
  - クリック → `e.stopPropagation()` で既存 onClick (router.push) を抑止, 父組件が ArchGraphModal を弹起
  - onArchClick prop を受け取った時のみボタン表示 (optional)
  - title="View architecture context (cypher graph)" 必須
- **AC**:
  - AC-1: アーキテクトが Kanban カードで 🕸 Arch 按钮を確認できる
  - AC-2: クリックで modal が弹起, 既存跳详情動作と干渉しない
  - AC-3: ボタン未传递 (no onArchClick) の場合, ボタン非表示
- **守門**: 守門 #1 禁回溯叙事 / 守門 #11 缺标比错标 / 守門 #12 文档治理

#### REQ-ARCH-002: ArchGraphModal 1-hop 高亮

- **業務価値**: 該当タスクがシステムのどこに位置するかを視覚的に把握
- **要件**:
  - Modal 80vw × 80vh, 中央, z-50
  - 3 endpoint 调用: `POST /api/graph/ensure-fresh` → 200/202 → `POST /api/graph/cypher` fallback
  - 描画 library: cytoscape.js 3.x + cose-bilkent 4.x レイアウト
  - **高亮规则 (per ADR-0041 §2.3.3)**:
    - 現 work_item ノード: cyan #00f0ff 64px 太枠 (主色)
    - 1-hop 隣人ノード: kind 別既定色 (11 種), 48px
    - 1-hop エッジ: cyan 2px solid
    - 2-hop code-side ノード: 20% opacity (cratemodule / symbol のみ)
    - 2-hop エッジ: gray #475569 1px dotted 30% opacity
- **AC**:
  - AC-1: Modal 表示後 1 秒以内に cytoscape 描画完了
  - AC-2: 現 work_item ノードが他ノードと視覚的に区別できる (cyan + 64px)
  - AC-3: 1-hop 隣人ノード (最大 11 種) が全て描画される
  - AC-4: 2-hop コード側 (cratemodule / symbol) は 20% opacity で弱化
- **守門**: 守門 #7 0 unsafe (TypeScript strict) / 守門 #14 tc-skip 不滥用

#### REQ-ARCH-003: 冪等 (idempotency) 必須

- **業務価値**: 同一 work_item への反復操作で DB に重複書込しない
- **要件**:
  - **fingerprint = sha256(work_item_id + worktree_branch + worktree_sha + source + project_id)** で冪等キー
  - fingerprint 命中 → agent 起動 skip, 既存 graph 返却 (200 fresh)
  - fingerprint 不一致 → agent 起動, 完了後 fingerprint 記録
  - LLM 出力 deterministic: `temperature=0`, `top_p=0.1`, `seed=work_item_id.hash()`
  - 書込は Cypher `MERGE ... ON MATCH SET ... ON CREATE SET ...` (重複書込防止)
- **AC**:
  - AC-1: 同一 fingerprint で 2 回連続 ensure-fresh → 2 回目 agent 起動 skip, < 200ms
  - AC-2: worktree_sha 変化 → fingerprint 変化 → agent 起動
  - AC-3: 同 work_item_id で 5 人同時クリック → 1 回 agent 起動, 残り 4 人は同じ結果
- **守門**: 守門 #5 環境変数安全 / 守門 #12 文档治理

#### REQ-ARCH-004: 排他 (mutex) 必須

- **業務価値**: 多人同時アクセスで memgraph の書込が衝突しない
- **要件**:
  - per-work_item_id advisory lock (Postgres `pg_try_advisory_xact_lock(work_item_id_hash)`) 5 分 TTL
  - 補完: Redis `SETNX graph:lock:{work_item_id} 1 EX 300` (任意, Phase 2+)
  - in-process coalesce: `pending[work_item_id] = oneshot::Receiver` で同期待ち
  - lock 取得失敗 → 202 Accepted + `Retry-After: 3s`, frontend 30s polling
  - agent 失敗 / cancelled → lock 即解放 (advisory_xact は transaction end)
- **AC**:
  - AC-1: 2 人が同時に同一 work_item を ensure-fresh → 1 人は 200 fresh, もう 1 人は 202 running + retry_after_ms=3000
  - AC-2: 30s 以内に 2 人目も 200 fresh 取得
  - AC-3: agent 失敗時 lock 解放確認 (advisory lock のトランザクション commit/rollback)
  - AC-4: 5 分 TTL 超過 → 自動解放, 別ユーザー取得可能
- **守門**: 守門 #9 子代理实证 / 守門 #10 代签規則

#### REQ-ARCH-005: データ源双支持 (local | git)

- **業務価値**: ローカル開発 + CI/マルチユーザー環境の両方で動作
- **要件**:
  - `source: "local"` | `"git"` 2 値
  - **local**: 当該 worktree の作業ディレクトリを直接走査 (Phase 2 で実装, Phase 1 mock のみ)
  - **git**: git remote URL + branch + commit SHA を libgit2 で clone, ephemeral directory で走査
  - フロントデフォルト: `ActorContext.local_runtime_id` 存在時 `"local"`, なければ `"git"`
- **AC**:
  - AC-1: source=local で 1 ワークツリー走査, AST 抽出, LLM 推断, memgraph 書込完了
  - AC-2: source=git で remote URL + branch + SHA 指定, clone + 走査 + 書込完了
  - AC-3: source 不正値 → 400 invalid_payload
- **守門**: 守門 #6 PowerShell only / 守門 #8 不沿用历史叙事

### 48.3 データ要件 (DB 三類横展開, per 2026-09-01 18:30 JST 拍板)

| 物理名 | 論理名 | 種別 | 概要 |
|---|---|---|---|
| `graph.graph_node` | グラフノード | **Master (M)** | SCD Type 2, 物理削除禁止, 25 kind union |
| `graph.graph_edge` | グラフエッジ | **Master (M)** | SCD Type 2, source/target 両 FK 必須, 24 kind union |
| `graph.graph_fingerprint` | 指紋監査ログ | **Transaction (T)** | append-only, 物理削除禁止, 90 日 TTL |

> Work (W) 類なし: 短 TTL データは `agent.agent_session` で扱う, 物理削除 + タイマー失効

詳細: [data-design/ipa-detail/tables/graph_graph_node.md](../data-design/ipa-detail/tables/graph_graph_node.md) (T-NEW-001) / `graph_graph_edge.md` (T-NEW-002) / `graph_graph_fingerprint.md` (T-NEW-003)

### 48.4 インターフェース要件

- `POST /api/graph/ensure-fresh`: 冪等+排他 trigger (per REQ-ARCH-003, REQ-ARCH-004)
- `POST /api/graph/cypher`: 1-hop 問合せ (max_hop=1 or 2)
- `GET /api/graph/health`: memgraph + agent_runtime 健全性

詳細: [architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md §2](../architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md) (詳細設計 11 段)

### 48.5 セキュリティ・テナント要件

- **13 類 tenant_id 必帯** (per REQ-SEC-001): 3 表全て RLS 13 類ポリシー強制
- **JWT 検証**: API Gateway (per ADR-0027 STAR IDE Gateway) で全 request 検証
- **LLM Secret**: Phase 2 で `agent.credential_broker` (per REQ-SEC-004)
- **PII 排除**: ノード properties に email 含めない, display_name のみ
- **AI Audit**: `graph_fingerprint` 記録全実行, per REQ-AUDIT-002 17 問遵守

### 48.6 非目標 (per 缺标比错标, 守門 #11)

| # | 非目標 | 理由 | 計画 |
|---|---|---|---|
| NG-001 | IDE ジャンプ (node click 遷移) | Phase 1 は in-modal 描画のみ | Phase 2+ |
| NG-002 | git push webhook 自動再生成 | webhook 統合は別途 work | Phase 3+ |
| NG-003 | マルチ monorepo 跨倉分析 | 単倉前提 | Phase 3+ |
| NG-004 | ノード/辺手動編集 (DB 書込) | Phase 1 read-only | Phase 2+ |
| NG-005 | export PNG / SVG / JSON | 単 modal 内表示のみ | Phase 2+ |
| NG-006 | 実 memgraph 接続 | Phase 1 MSW mock, Phase 2 advisory lock + fingerprint のみ, Phase 3 で Bolt/HTTP 接続 | Phase 3 |

### 48.7 既知の缺口 (per 缺标比错标, 守門 #11)

- 1% random 202 パス (mock 動作確認) — 確率低, 100 リクエスト中 1 回
- `useStore.actorContext` 不存在 → Phase 1 fallback で `workItem.tenant_id` 使用
- cytoscape-cose-bilkent 公式 d.ts なし → 自作 `cytoscape-ext.d.ts` 兜底
- Worktree 状態変化 webhook → Phase 3+ 自動再生成未実装
- Symbol 詳細 (file/line/snippet) → Phase 2+ 节点 click 遷移先未実装

### 48.8 段階計画 (per ADR-0041 §3)

| Phase | 内容 | token 予算 | 状態 |
|---|---|---|---|
| 1 | フロント契約 + MSW mock 実装 | 1.0M | **🟢 完了** (per ARCH-AGENT-GRAPH-001-REPORT v0.1) |
| 2 | backend LLM worker (`crates/star-graph-agent/`) + 冪等 advisory lock + agent-runtime 14 状態機統合 | 4.8M | ⏳ P3-B 拍板待ち |
| 3 | 実 memgraph 例 (Bolt/HTTP) + 25 domain schema + インデックス + バックアップ | 2.0M | ⏳ Phase 2 完了後 |
| **計** | | **7.8M** | (per STAR-OLU-001 v0.1 1 SRE·週 = 1.2M, 約 6.5 週) |

### 48.9 受け入れ基準 (Acceptance Criteria 集約)

- AC-ARCH-1: REQ-ARCH-001/002/003/004/005 全 5 件が unit test + integration test で pass
- AC-ARCH-2: tsc --noEmit 0 错, vitest 320+/320+ pass (per Phase 1 実續)
- AC-ARCH-3: 13 類 RLS 13 類ポリシー強制 (Phase 3 検証)
- AC-ARCH-4: 並走 100 work_item で lock 競合率 < 1% (Phase 2 k6 検証)
- AC-ARCH-5: P95 latency < 1s (fingerprint 命中), P95 < 60s (agent 起動含む)

### 48.10 トレーサビリティ

- 一次出典: ADR-0041 v0.1
- 詳細設計: spec/agent-api/arch-agent-graph-viewer.md v0.1 (11 段)
- データ設計: data-design/ipa-detail/tables/graph_*.md (3 表 T-NEW-001/002/003)
- Phase 1 報告: docs/reports/ARCH-AGENT-GRAPH-001-REPORT.md v0.1 (7 段)
- 関連要件: REQ-SEC-001 (13 類), REQ-AUDIT-002 (17 問), REQ-DATA-001/002/003
- 関連 ADR: ADR-0027 (STAR IDE Gateway), ADR-0030 (Lease+Heartbeat+Resume)

### 48.11 段階要件 (MVP / V1 / V2 / Future)

| 段階 | 含める | 除外 |
|---|---|---|
| MVP (Phase 1) | フロント契約 + MSW mock | 実 memgraph, LLM agent |
| V1 (Phase 2) | LLM worker + 冪等 + 排他 | 実 memgraph 接続, export, IDE ジャンプ |
| V2 (Phase 3) | 実 memgraph + 25 schema + バックアップ | git push webhook, 跨倉分析 |
| Future | webhook 自動再生成 + export + マルチ monorepo + 跨 tenant 共有 | (per NG-001~006 段階拡張) |

---

*本節 §48 は arch-agent-graph-viewer 機能追加 (2026-09-02 02:10 JST Ulysses "需求和基本设计, 詳細设计 補完" 発令) による。*


## 49. Onboarding (First-Run) 要件 (per ADR-0042 v0.1, 2026-09-02 08:01 JST 拍板)

> **追加日**: 2026-09-02
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **依据**: [ADR-0042-onboarding-first-run v0.1](../architecture/2026-08-26-upgrade/adr/0042-onboarding-first-run.md) + [commit `a54c79d` OnboardingGuard 实现](../.git)
> **ステータス**: Phase 1 完了 (frontend contract + 3 探测器 + 5 retry mock, per 8/1 08:14 JST 11/11 vitest pass), Phase 2 等 P3-B 拍板

### 49.1 背景・動機 (per 2026-09-02 07:58 JST)

ユーザーは初回起動時, 既に存在する LLM API key 凭证 (localStorage / env-var-hint / IDE 残留) を **自動識別** したい。**手動で 1 つ 1 つ入力** するのは摩擦が高い。識別出来后, ユーザーエージェントを選んで **関連付け**, 失敗したら **自動 5 回リトライ**, 最終的に失敗したら **解决步骤をユーザーに提示** + **audit log 記録** すべき。

既存 `AgentSettingsModal` (per commit `cb2475e`) は **能動的な齿轮手動入力** のみで, **初回起動の自動オンボーディング** には対応していない。

### 49.2 業務要件 (5 件)

#### REQ-ONB-001: 初回起動で 3 探测を並列実行

- **業務価値**: ユーザーが既存凭证を再入力する手間を排除
- **要件**:
  - アプリ起動時 (SettingsProvider init / mount) に 3 探测を並列実行
  - localStorage `star:api-keys` (既存 /settings/api-keys 保存先) をスキャン
  - env-var-hint: `process.env.NEXT_PUBLIC_*_API_KEY_HINT` の存在性のみ (値は読み取らない, 守門 #5 遵守)
  - IDE-residual: `/.vscode/settings.json` 等 5 路径を fetch (Phase 1 mock, 4xx → 空配列)
  - 検出完后, 重複排除 (provider + label 一致で最初の 1 件を残し)
- **AC**:
  - AC-1: 初回起動後 1 秒以内に 3 探测が並列完走
  - AC-2: localStorage に 3 個のキー, 検出结果は 3 件 + 重複排除正しい
  - AC-3: env-var-hint は 存在性のみで, 実値はメモリ/ログに现れない (守門 #5)
- **守門**: 守門 #1 禁回溯叙事 / 守門 #5 環境変数安全 / 守門 #11 缺标比错标 / 守門 #12 文档治理

#### REQ-ONB-002: ユーザーがエージェントを選んで関連付け

- **業務価値**: 1 つの key を複数の agent で使う or 別々に使う, ユーザー選択で柔軟
- **要件**:
  - 検出キーの一覧 (provider / label / preview / source_label) を modal に表示
  - 各 key に agent select dropdown (existing CliTab list)
  - 「暂不关联」 (skip per key) を選択可能
  - 4 必备 provider (openai / claude / gemini / minimax) を cyan chip で強調表示
  - encrypted_rust モードで保存 (per 8/1 02:49 JST 拍板 storage_opt1)
- **AC**:
  - AC-1: 3 個の key 全部に agent select が表示され, 1 件も選ばず「确认关联」できる (0 件 = ボタン disabled)
  - AC-2: 4 必备 provider は chip に "必备" マーク表示
  - AC-3: 关联选择は `cli_profile_id` + `agent_kind` + `agent_id` 3 フィールドで保存
- **守門**: 守門 #7 0 unsafe / 守門 #14 tc-skip 不滥用

#### REQ-ONB-003: 失敗時の自動 5 回リトライ (3-6-12-24-48s 指数 backoff)

- **業務価値**: 1 過性のネットワークジッタで関連付け失敗しない
- **要件**:
  - 1 過性失敗時, 指数 backoff で 5 回まで自動リトライ (3s / 6s / 12s / 24s / 48s)
  - 1 回のテストは fetch タイムアウト 10 秒
  - リトライ中, UI に attempt 数 + 次の backoff 秒数を表示
  - 5 回すべて失敗 → 自動停止, 次の REQ-ONB-004 に遷移
- **AC**:
  - AC-1: 1 過性失敗 (e.g. timeout) → 3s 後 2 回目, 6s 後 3 回目 … 48s 後 5 回目
  - AC-2: 1 回目で成功 → 1 回で停止 (リトライしない)
  - AC-3: リトライ中 UI に `attempt 2/5 · 次回リトライ 6s 後` を表示
- **守門**: 守門 #5 環境変数安全 (timeout 中も preview のみ, 明文なし)

#### REQ-ONB-004: 失敗時の解决步骤提示

- **業務価値**: 5 回リトライ後も失敗, ユーザーが自力で解决できる
- **要件**:
  - 5 回失敗後, 各失敗 key ごとに error card 表示
  - error code 6 種類 (unauthorized 401 / forbidden 403 / rate_limited 429 / model_unavailable 404|503 / network_timeout / unknown) を分類
  - 各 error code ごとに 解决步骤 (1-3 steps) + doc URL + curl test command
  - 例: 401 の場合 → "API key が有效か確認" + platform.openai.com/account/api-keys リンク + `curl -H "Authorization: Bearer $KEY" ...`
- **AC**:
  - AC-1: 5 回失敗した key ごとに error card 表示
  - AC-2: 401 / 403 / 429 / 0 / 404 / 503 / 500 が正しい code に分類
  - AC-3: error card 内に "重试" ボタン表示, クリックすると 5 回リトライ再開
- **守門**: 守門 #5 環境変数安全 (error message に明文含まない)

#### REQ-ONB-005: audit log 記録 (per 守門 #9)

- **業務価値**: どの key がどのユーザーでいつ失敗したか追跡可能
- **要件**:
  - 5 回失敗時, `star:onboarding-audit` localStorage に append (Phase 1 mock)
  - 記録内容: `audit-{timestamp}-{provider}` ID + action `onboarding.test_key.failed` + provider + label + attempts (5) + status_code + error_message + timestamp
  - Phase 2 で `audit_audit_event` テーブルに真書き (per AGENTS.md §4 #9 監査必帯)
  - 13 類 tenant_id 必帯 (per REQ-SEC-001)
- **AC**:
  - AC-1: 5 回失敗後, `localStorage.getItem("star:onboarding-audit")` に 1 件以上の entry
  - AC-2: entry 内に provider / label / status_code / timestamp 全部含む
  - AC-3: Phase 2 で backend 監査ログに同期 (per #9 17 問遵守)
- **守門**: 守門 #9 子代理実証 (audit log 必須) / 守門 #10 代签規則

### 49.3 データ要件

- **localStorage 2 key**: `star:api-keys` (既存 /settings/api-keys 保存) + `star:onboarding-completed` (boolean "true" | "skipped")
- **audit log 1 key** (Phase 1 mock): `star:onboarding-audit` JSON 配列
- **DB 三類横展開** (per 2026-09-01 18:30 JST 拍板, Phase 2 で audit_audit_event):
  - `audit_audit_event` 走 Transaction (T) append-only (per 仓内 100 表実續)
  - 物理削除禁止 + 90 日 TTL (per AI Content Retention §6.8)

### 49.4 インターフェース要件

- 3 探测エンドポイント (Phase 1 mock, Phase 2 后端):
  - `GET /api/onboarding/env-hint` → 存在性 array
  - `POST /api/onboarding/test-key` → 单 key 测试 (1 attempt)
  - `POST /api/audit/onboarding-failed` → audit log 写入
- 客户端既存 `/api/api-keys` 沿用 (encrypted_rust 存储)

### 49.5 セキュリティ・テナント要件

- **13 類 tenant_id 必帯** (per REQ-SEC-001): `tenantId` prop で OnboardingGuard に注入, Phase 1 mock = `tenant-physis-corp`
- **JWT 検証**: 既存 /settings/api-keys 沿用
- **LLM Secret**: preview のみ, 永続化しない (守門 #5)
- **PII 排除**: audit log 内に preview ではなく status_code のみ
- **AI Audit**: REQ-AUDIT-002 17 問遵守 (Phase 2 真接 audit_audit_event テーブル)

### 49.6 非目標 (per 缺标比错标, 守門 #11)

| # | 非目標 | 理由 | 計画 |
|---|---|---|---|
| NG-001 | IDE-residual Phase 1 mock 返空 | service worker / fs API ブラウザ制約 | Phase 2+ 接 service worker |
| NG-002 | env-var-hint Phase 1 mock 返空 | process.env ブラウザ端不可 | Phase 2+ 接 /api/onboarding/env-hint |
| NG-003 | 真 fetch テスト (testKeyOnce) | Phase 1 mock ランダム | Phase 2 真接 fetch + ep.build_headers |
| NG-004 | 真 audit log テーブル | Phase 1 localStorage mock | Phase 2 audit_audit_event テーブル |
| NG-005 | 関連付け時 backend 真接 | Phase 1 mock 走 /api/api-keys | Phase 2 + KMS 統合 |

### 49.7 既知の缺口 (per 缺标比错标, 守門 #11)

- test retry 真等 3-6-12-24-48s (最大 48s, テスト時 45s 経過): Phase 1 mock 化, vi.useFakeTimers で高速化可能
- audit log 容量無制限 (append-only, 90 日後手動 cleanup 必要)
- 関連付け時 key の masking (preview = `sk-***xyz` 形式, 真値取得不可 → Phase 1 mock, Phase 2 真接時 backend で真値復号化必要)

### 49.8 段階計画 (per ADR-0042 §4)

| 段階 | 内容 | token 予算 | 状態 |
|---|---|---|---|
| 1 | 4 段設計 + 11 ファイル実装 (ADR + types + scanner + retry + Guide + Guard + layout + test) | 4-5M | **🟢 完了** (per commit `a54c79d`, tsc 0 + 337/337 vitest pass) |
| 2 | backend KmsAudit 真接 (audit_audit_event テーブル + KMS) | 0.8M | ⏳ P3-B 拍板待ち |
| 3 | 真 fetch + IDE-residual + env-var-hint 后端 API | 1.5M | ⏳ Phase 2 完了後 |

### 49.9 受け入れ基準 (Acceptance Criteria 集約)

- AC-ONB-1: REQ-ONB-001~005 全 5 件が vitest 11/11 + tsc --noEmit 0 错
- AC-ONB-2: 3 探测並列完走 + 重複排除正しい
- AC-ONB-3: 5 回リトライ (3-6-12-24-48s) 動作
- AC-ONB-4: 失敗時 6 error code に分類 + 解决步骤提示
- AC-ONB-5: 5 回失敗時 audit log 記録 (Phase 1 localStorage, Phase 2 audit_audit_event)

### 49.10 トレーサビリティ

- 一次出典: ADR-0042 v0.1
- 詳細設計: spec/agent-api/onboarding.md v0.1 (10 段, 別途)
- 基本設計: basic-design.md §12 (3 段, 別途)
- Phase 1 実装: frontend/src/{types,lib,components}/onboarding + app/layout.tsx (8 ファイル)
- Phase 1 報告: docs/reports/ARCH-AGENT-GRAPH-001-REPORT.md v0.1 (同 session, onboarding も包含予定)
- 関連要件: REQ-SEC-001 (13 類), REQ-AUDIT-002 (17 問), REQ-DATA-001/002/003
- 関連 ADR: ADR-0027 (STAR IDE Gateway), ADR-0030 (Lease+Heartbeat+Resume), ADR-0041 (arch-graph)

### 49.11 段階要件 (MVP / V1 / V2 / Future)

| 段階 | 含める | 除外 |
|---|---|---|
| MVP (Phase 1) | 3 探测 mock + 5 retry + audit log localStorage + 4 必备 provider | IDE-residual / 真 fetch / audit テーブル |
| V1 (Phase 2) | 真 fetch + IDE-residual + audit_audit_event テーブル | (per NG-001~005 段階拡張) |
| V2 (Phase 3+) | 関連付け時 backend 真接 + KMS 統合 | (per NG-005 段階拡張) |

---

*本节 §49 は onboarding 機能追加 (2026-09-02 08:01 JST Ulysses 4 拍板) による。*
