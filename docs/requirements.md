# Vibe Coding Work Management SaaS 要件定义书（统合扩展版 v5.66）

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

> **v2.2 补充（2026-09-28）**：导航明确采用“先选 Project → 查看/管理该项目 Worktree → 展开 Worktree → 进入同级 Group Apps”。Worktree 管理是项目开发的核心工作面；展开的 Worktree 下包含 Multica、Jira 类工作管理、Task Card、Infinite Canvas、Workflow/LangGraph 和插件入口。完整要件见第 50 章。
>
> **v2.3 补充（2026-09-29）**：Project Worktree Index 是可直接访问的产品入口；Project 页的 Worktrees 视图须能进入该管理面，Worktree 导航不能被通用任务列表路由替代。展开后的 Worktree Group 路由须保留 `worktree_id` 上下文。完整要件见第 50 章。
>
> **v2.4 补充（2026-09-29）**：Index 深链以 `/worktree?project_id={project_id}` 保留所选项目；项目 Worktrees 入口必须传递 `project_id`，无项目参数时要求先选 Project。Group 深链继续以 `worktree_id` 解析并校验 Project 归属。完整要件见第 50 章。
>
> **v3.6 补充（2026-09-29）**：Multica 与 Jira / Task Card 在线视图可通过同一版本化生命周期命令更新 canonical WorkItem；界面按六态迁移和 review gate 提供合法操作，冲突后刷新授权投影。完整要件见第 50 章。
>
> **v4.1 补充（2026-09-29）**：Worktree Index 明确作为多 Agent Worktree 的核心管理面：授权服务端投影支持稳定游标读取；归档/恢复必须经过短时 plan 与用户二次确认；provider 或 API 不可用时不得用 seed 冒充生产结果。完整要件见第 50 章。
>
> **v4.2 补充（2026-09-29）**：Worktree Index 必须提供基于当前 Project membership 的受权成员目录，并支持通过版本化 plan-confirm 转派负责人；界面须明确新旧 owner、计划有效期和确认结果，成员目录失败时关闭转派入口。完整要件见第 50 章。
>
> **v4.3 补充（2026-09-29）**：Canvas Outbox 浏览器投影消费者须在授权投影刷新成功后持久化 Worktree/Canvas 复合游标，刷新失败时重放当前页；持久游标仅是可丢弃的刷新提示，不是授权凭据或 NATS consumer offset。完整要件见第 50 章。
>
> **v4.4 补充（2026-09-29）**：Task Card lifecycle review gate 提供版本化请求评审、通过与驳回命令；提交者必须是当前执行 claimant，评审者不得是该 claimant，拒绝须记录理由；只有评审通过才可由 `in_progress` 进入 `completed`。完整要件见第 50 章。
> **v4.5 补充（2026-09-29）**：Task CLI grant 使用 Ed25519 issuer 私钥签名和带 key id 的公钥轮换验证（旧公钥须至少保留至该 key 签发的 grant 全部过期，grant 最长五分钟）；Local Runtime 验签后仍须重验当前 ACL 与 Runtime health，再校验执行上下文并原子消费 nonce。此契约已实现独立签名/验证 helper，尚未接入 Session API 或启动链。完整要件见第 50.4 章。
>
> **v4.6 补充（2026-09-29）**：增加认证的 Task CLI Session start API 契约，要求在当前 Project membership、Worktree/Task Card 关联、claimant、lifecycle version、Runtime binding 与 scope 校验后才委托受信任 provisioner；请求须幂等且关联 correlation ID。Session 未实际启动或 provisioner 未装配时不得返回成功，attachment ticket 仅通过 no-store 响应交付且最长 60 秒。完整要件见第 50.4 章。
>
> **v4.7 补充（2026-09-29）**：Local Runtime 的 Task CLI 必须采用交互式 PTY 承载 stdin/stdout、resize 与退出状态，并从清空的进程环境中只注入已批准配置；PTY 只提供终端能力，不构成 OS sandbox。当前已增加底层 PTY adapter，但它尚未接入真实 Session provisioner、terminal-stack 或 OS 隔离执行器。完整要件见第 50.4 章。
>
> **v4.8 补充（2026-09-29）**：Task CLI 的 PTY 输出在进入终端 sink 前须由有界 FIFO 队列承载并施加背压，避免 attachment 尚未建立时静默丢弃输出；PTY manager 关闭时须终止其拥有的子进程。当前 adapter 仍为 Local Runtime 内部切片，未接真实 provisioner、sink、审计和 OS sandbox。完整要件见第 50.4 章。
>
> **v4.9 补充（2026-09-29）**：Task Card UI 增加认证 Session start、一次性 ticket 的页面内存持有与 ticket-first WebSocket attachment；xterm 只在 Hello 授权后接收输入并显示按 pane 输出。浏览器不复用已消费 ticket，也不自动重连，直到服务端提供签发新 ticket 的 reattach API。当前这只是可调用 UI/API 与传输显示切片，真实 Local Runtime provisioner、PTY sink、sandbox、审计和宿主 provider 尚未装配。完整要件见第 50.4 章。
>
> **v5.0 补充（2026-09-29）**：固定底栏 Chat 的提交请求携带显式 WORKTREE/GLOBAL 范围、目标 Worktree、实体引用、幂等键与 correlation；服务端先逐目标解析当前 GroupContext，再委托 L0 workflow。GLOBAL 必须显式选择 1–20 个目标；任一目标无权访问时整个请求不派发。当前只有 fail-closed workflow injection seam；没有持久 Transcript、LangGraph runtime/checkpoint、resume 再授权或流式 UI，未配置 workflow 时返回 503。完整要件见第 50.6 章。
>
> **v5.1 补充（2026-09-29）**：GLOBAL scope 提供 Bearer 授权的 Worktree 目标目录与底栏多选；目录只返回当前 actor 仍是有效 Project 成员的非归档 Worktree 最小信息，并分页，不包含 checkout 路径。目录结果不构成授权凭证，消息提交时仍逐目标重新校验；省略 correlation 时使用 Idempotency-Key 作为稳定默认值；宿主 provider 或 workflow 未接入时不得用 seed 目标或模拟发送。完整要件见第 50.6 章。
>
> **v5.2 补充（2026-09-29）**：Task CLI 增加受授权的 session status、cancel 与 reattach API 契约。各操作须重验当前 Project writer membership 和 canonical Task/Worktree 关联；Runtime provisioner 再核验完整 session binding 与当前 ACL/Runtime health。status 不返回终端数据；cancel 必须幂等；reattach 只能返回新签发的 ≤60 秒单次 ticket。UI 尚未接入恢复流程，缺少 provider 时 API 必须 fail closed。完整要件见第 50.4 章。
> **v5.3 补充（2026-09-29）**：Task Card 在页面仍持有 Session 关联时可刷新状态、显式取消 Session，并在连接失败/断开后先查询状态再请求新的 attachment ticket；只允许 `running` / `disconnected` 状态重新连接。Ticket 只保存在页面内存且每次连接单次使用；页面刷新后的 Session 恢复仍需要受授权的 Session 列表/恢复 API。完整要件见第 50.4 章。
> **v5.4 补充（2026-09-29）**：Task Card 增加 Worktree/Task-scoped Session 列表，默认返回最近 20 条、服务端上限 50 条的脱敏状态；列表不得返回 attachment ticket 或终端输出。页面刷新后可从列表重新发现会话，恢复时仍须重新查询状态并请求新 ticket。API 逐请求重验身份、Project membership 与 canonical Task/Worktree 绑定；真实 provisioner 未装配时 fail closed。完整要件见第 50.4 章。
> **v5.5 补充（2026-09-29）**：明确 LangGraph checkpoint/thread 只保存非敏感执行状态，不保存 JWT、可复用权限快照或 capability grant；thread ID 由服务端生成并与 Chat Session、Task Card、WorkItem、Agent Session 分离。恢复、interrupt 批准、节点 replay 与每次工具调用都重新解析 GroupContext/Plugin grant；可能产生副作用的节点必须通过带稳定幂等身份的领域命令/outbox，防止从 checkpoint 重放造成重复写入。完整要件见第 50.6 章。
>
> **v5.6 补充（2026-09-29）**：Group Chat PostgreSQL adapter 在单一事务内写入服务端 thread/session、user Transcript、Run intent、actor-scoped 幂等回执、dispatch outbox 与审计事件；同一有效幂等键重放返回原 receipt，不同 fingerprint 返回冲突。新增表逐表标注 W/T 分类并强制 RLS；当前尚无 outbox consumer/L0 worker、LangGraph/checkpointer、retention cleanup job、目标数据库部署或运行时 ACL/RLS 验收，因此发送仍保持禁用。完整要件见第 50.6 章。
> **v5.13 补充（2026-09-29）**：明确 PostgreSQL schema/table SQL 权限与 RLS 是独立门禁；部署必须显式 provision 最小权限应用角色，并以实际运行登录角色验证授权与 RLS，禁止 superuser/BYPASSRLS 运行 Group API。Phase 5/6 迁移和策略已在隔离 PostgreSQL 验证库实测、可重复执行；目标库和生产应用角色授权仍未配置。完整要件见第 50.8 章。

---

## 2. 产品重新定位

系统同时承担三种职责，且三者不得混为一个 Domain（§0）：

```text
1. Jira-class Work Management
2. GitHub / GitLab Development Integration
3. AI Coding Worktree Control Plane
4. Worktree-first Group Experience（以 Worktree 为顶层索引，承载任务、画布、CLI、聊天与可插拔应用）
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

ARCH-OBL-GRP-001  Worktree Group Coherence（第 50 章新增）
  → 任务、Canvas、CLI、聊天、LangGraph 与插件必须在明确的 Worktree Group Context 中协作；任何跨 Worktree 操作必须显式列出目标 Worktree 并分别经过授权、审计与状态校验。
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
| `WTG-xxx` | 渡口 Worktree 群组 Requirement（Worktree 顶层索引、同级应用与群组上下文，第 50 章） |
| `TCI-xxx` | Task Card / CLI Requirement（任务卡、受控 CLI 会话与运行时绑定，第 50 章） |
| `CAN-xxx` | Worktree 群组 Canvas Requirement（Canvas 与任务、Agent、Jira 类对象互操作，第 50 章） |
| `CHAT-xxx` | 范围化底栏聊天 Requirement（`GLOBAL` / `WORKTREE` 选择与审计，第 50 章） |
| `PLG-xxx` | Group App Plugin Requirement（插件声明、群组启用与热插拔，第 50 章） |
| `LGS-xxx` | LangGraph Scope Requirement（L0/L1 在群组范围内的编排边界，第 50 章） |

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
| WTG-001 | 用户选择 Project 后，左侧依次展现该项目的 Cloud Branch、Engineering Run 与其 Worktree；见第 50.1-50.2 章 | ARCH-OBL-GRP-001 |
| WTG-002 | 点击 Run Worktree 后显示共享 Run Context 的 Inbox、Task Card、Canvas、Workflow、BI/Benchmark 与 Plugin Apps；见第 50.2-50.3 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-001 |
| WTG-006 | Project Worktree Index 与 Engineering Run Shell 可分别访问；Index 深链以 `project_id` 保留 Project，Run deep link 保留 `project_id/branch_id/engineering_run_id` 并可附 `worktree_id` focus；旧 Worktree Group route 经兼容 adapter 解析 Run；Index 不得被通用任务列表替代 | 第 50.1-50.2 章 | ARCH-OBL-GRP-001 |
| WTG-010 | Worktree Index 分开呈现当前 Git lock observation 与持久化 lock 标记；unknown 不得通过 cleanup guard | 第 50.1、50.3、50.8 章 | ARCH-OBL-GRP-001 |
| WTG-011 | Project Worktree Index 必须以 `project:read` + 当前 membership 查询脱敏的已绑定 Repository 清单（仅 ID/name/default branch，不含 URL/路径），并提供授权候选发现、创建与导入；仅接受 Project 已绑定 repository_id、经过 Git ref 校验的 branch/base_ref 或可信 Host Runtime opaque candidate_id；拒绝客户端路径、仓库 URL 和 Git 命令；副作用前重验 membership 与 Project-Repository binding，以 Idempotency-Key/correlation_id 幂等持久化 operation、Worktree projection、Audit/Outbox 后返回 202；缺少 provider 返回 503；Index UI 不得回退本地 seed，须验证 Project/Repository/candidate/receipt 关联并在受理后刷新 Index | 第 50.3.1 章 | ARCH-OBL-GRP-001 |
| TCI-001 | Multica、Jira 类视图与 Engineering Run 下直接访问的 Task Card 必须共用 Run-scoped WorkItem 事实源；见第 50.3 章 | ARCH-OBL-GRP-001 |
| TCI-002 | 任务卡必须能够在已授权的关联 Worktree 中打开受控交互式 PTY CLI 会话，并将会话、命令结果与审计关联回任务卡 | 第 50.4 章 | ARCH-OBL-DEV-004/ARCH-OBL-GRP-001 |
| TCI-006 | Task Card review gate 必须使用版本化、幂等且可审计的请求/通过/驳回命令，只有非提交者评审通过后才可完成 | 第 50.3 章 | ARCH-OBL-GRP-001 |
| TCI-007 | Task CLI 执行 grant 必须由受信任 API 私钥签发并带 key id；Local Runtime 只配置 issuer 公钥、验签后仍需重验当前 ACL 与 Runtime 状态，再消费单次 nonce | 第 50.4 章 | ARCH-OBL-DEV-004/ARCH-OBL-GRP-001 |
| TCI-008 | Task CLI Session start 必须由 Bearer-authenticated Group API 验证当前 Project/Worktree/Task、claimant、lifecycle version、Runtime binding 与 `agent_session:start` scope，再委托具备实时授权复验、grant 与 sandbox spawn 能力的 Local Runtime provisioner；缺少 provisioner 时 fail closed | 第 50.4 章 | ARCH-OBL-DEV-004/ARCH-OBL-GRP-001 |
| TCI-009 | Task CLI 必须经交互 PTY 提供受限 stdin、resize、原始输出字节与退出状态；attachment 建立前输出由有界 FIFO 缓冲并以背压防止静默丢失；manager 关闭时终止所拥有的子进程；从空环境启动并仅注入获批配置；PTY 不能代替 OS sandbox | 第 50.4 章 | ARCH-OBL-DEV-004/ARCH-OBL-GRP-001 |
| TCI-010 | Task Card UI 必须通过当前 Bearer 会话发起版本化、幂等 Session start；ticket 仅保存在页面内存并单次用于首帧授权；Hello 前禁用终端输入；输出绑定 session pane；断线不得复用已消费 ticket，重连只能使用服务端新签发的 ticket | 第 50.4 章 | ARCH-OBL-DEV-004/ARCH-OBL-GRP-001 |
| TCI-012 | Task Card 必须能够在页面刷新后通过受授权的 Worktree/Task-scoped Session 列表重新发现最近会话；列表返回有界脱敏状态且不含旧 ticket/终端输出，每次列表与恢复操作都重验当前 membership、canonical Task/Worktree 和 Runtime 绑定 | 第 50.4 章 | ARCH-OBL-DEV-004/ARCH-OBL-GRP-001 |
| TCI-013 | Task Contract 必须按版本保存 goal、scope、dependencies 与 acceptance criteria；每次 TaskExecutionRun 固定引用该版本并保存创建时的 input/acceptance snapshot | 第 50.8 章 | ARCH-OBL-GRP-001 |
| TCI-014 | 每次真实 Task 执行尝试必须有独立 TaskExecutionRun；Task 可有多个 Run，Worktree 是可选历史执行上下文且不作为 Run 的父级或生命周期外键；不得与 `group_chat_run` 混用 | 第 50.8 章 | ARCH-OBL-GRP-001 |
| TCI-015 | TaskExecutionRun 必须分别记录执行器状态、Agent 声明、自动验证、人工接受/返工与集成状态；缺失数据为 unknown，actual/estimated cost 分开记录 | 第 50.8 章 | ARCH-OBL-GRP-001 |
| TCI-016 | Task Run Evidence 仅保存经过脱敏的 metadata/digest/受控 locator；禁止复制大日志、Secret、完整 transcript 或模型隐式推理，且读取须逐次经过 Project/Task ACL | 第 50.8 章 | ARCH-OBL-GRP-001 |
| CAN-001 | Canvas 必须作为 Worktree Group 内与 Task Management 同级的应用，并以实体链接关联 WorkItem、TaskCard、AgentSession 与 Worktree | 第 50.5 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-002 |
| CAN-002 | Canvas 对任务状态、关系和自动化的写操作必须走领域命令及既有 Guard；Canvas 仅消费已提交结果 | 第 50.5 章 | ARCH-OBL-GRP-001 |
| CHAT-001 | 固定底栏聊天必须支持 `GLOBAL` 与单一 `WORKTREE` 两种范围，并将范围、目标、会话和命令写入审计与 checkpoint | 第 50.6 章 | ARCH-OBL-GRP-001 |
| CHAT-002 | `WORKTREE` 范围只能装载该群组上下文和授权工具；`GLOBAL` 写操作必须显式列出目标 Worktree | 第 50.6 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-001 |
| CHAT-003 | 每次聊天提交必须经 Bearer actor 和 scope 授权，逐目标解析 GroupContext 与 EntityRef 范围；任何未授权目标整批拒绝，且不得向 L0 派发 | 第 50.6 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-001 |
| CHAT-004 | GLOBAL scope 必须从当前 actor 的有效 Project membership 中发现非归档 Worktree，并以有界游标分页返回最小目标信息；发现结果不能替代提交时的实时授权 | 第 50.6 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-001 |
| CHAT-005 | 接受的 Group Chat 请求必须原子持久化 Transcript metadata 与加密 payload、Run intent、稳定幂等 receipt、dispatch outbox 与 Audit；有效期内同 key 同 fingerprint 重放返回相同 receipt，不同 fingerprint 冲突 | 第 50.6 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-001 |
| CHAT-006 | Chat Transcript 正文需加密、有限保留、可清理并具授权导出/删除路径，append-only metadata 不含正文 | 第 50.6 章 | REQ-SEC-001/Agent Policy |
| PLG-001 | 插件必须声明 UI surface、能力、事件订阅、权限、版本兼容性与支持范围，且只能通过稳定应用接口接入 | 第 50.7 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-004 |
| PLG-002 | 插件必须支持可审计的注册、配置、启用、降级、排空和禁用生命周期；禁用后立即撤销能力而不删除既有业务事实 | 第 50.7 章 | ARCH-OBL-GRP-001 |
| PLG-003 | 插件产生的命令、事件和资源引用必须携带 Group Context、`correlation_id` 与 `schema_version` | 第 50.7 章 | ARCH-OBL-GRP-001 |
| PLG-004 | 插件导航只能使用当前 Worktree 的服务端授权 Registry projection | 第 50.7 章 | ARCH-OBL-GRP-001 |
| PLG-005 | Plugin Registry Master 记录必须按 tenant/worktree 隔离、保留 SCD Type 2 历史且禁止物理删除；Audit 作为 append-only Transaction | 第 50.7 章 | REQ-DATA-001/REQ-AUDIT-001 |
| LGS-001 | LangGraph L0 必须接收 `GLOBAL` / `WORKTREE` 范围；L1 TaskCard 的 checkpoint、工具调用与 Canvas/插件动作必须带 Worktree ID | 第 50.6、50.8 章 | ARCH-OBL-GRP-001 |
| LGS-002 | 跨任务、Canvas 或插件发起的编排必须经 L0、领域命令与事件执行；L1 之间不得直接通信 | 第 50.8 章 | ARCH-OBL-GRP-001 |
| LGS-003 | LangGraph checkpoint 不得成为授权凭证；服务端生成并映射 thread ID；resume、interrupt、replay 与每次 capability/tool call 重新授权，副作用命令须幂等且可审计 | 第 50.6 章 | ARCH-OBL-GRP-001/ARCH-OBL-DEV-001 |

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

---

## 50. 渡口 Project / Branch / Run / Worktree 管理与群组体验要求（v4.1）

### 50.1 目的与产品树

本节把 Star 的独特工作方式定义为 **Project → 云端 Branch → Engineering Run → 本地 Worktree**：用户先选择 Project，在左侧展开属于项目的远端 Branch，再展开该 Branch 下的 Engineering Run；Run 管理绑定该 Branch 的一个或多个本地 Worktree。点击 Run 内某个 Worktree 后，右侧才呈现该 Run 的 Inbox、工作项、Task Card、Canvas、Workflow、BI/Benchmark 和 Plugin tabs。它要解决多 Agent 并行开发时 worktree 分散、归属不清、Agent 分配/状态不可见、合并冲突和生命周期清理不可控的问题。CLI 和 Agent Session 从任务卡内打开。

Tenant / Workspace / Project 继续承担归属、权限和安全边界。Branch 表示云端分支目标，Engineering Run 是一次可协作、可观测并可做 Run 级分析的工作区；本地 Worktree 是绑定 Branch 的 checkout 及可选焦点/执行位置，不是 WorkItem 或 Run 的替代身份。Project Worktree Index 保留为跨 Branch/Run 的 Worktree 汇总和管理视图，不作为主导航的父层。该设计不新增 Git Worktree 或 WorkItem 的替代事实源；每个 Engineering Run 可管理多个 Worktree，但一个活跃 checkout 同时只能属于一个 active Run，历史 Run 仅保存可空 ID/ref snapshot。必须区分 `EngineeringRun` 与每次任务尝试的 `TaskExecutionRun`：前者承载 Run 工作区及跨 App 视图，后者是 Task Card 下的一次执行事实。

```text
Tenant → Workspace → Project（左侧第一级）
                                  └─ Cloud Branch（云端合并目标 / branch_id）
                                     └─ Engineering Run（Run 工作区 / engineering_run_id）
                                        ├─ Worktree Set（Run 所管理的本地 checkout）
                                        │  └─ Worktree（点击后成为 focus_worktree_id）
                                        └─ Run App tabs（由 engineering_run_id 标识；选中 Worktree 后展示）
                                           ├─ Inbox / Work Items（Multica + Jira 等价视图）
                                           ├─ Task Cards（与 Canvas 同级；卡内打开 CLI/Agent）
                                           ├─ Infinite Canvas（与 Task Cards 同级，可引用多个 Run Worktree）
                                           ├─ Workflow / LangGraph
                                           ├─ Run BI / Benchmark
                                           └─ 已启用的 Run Plugin Apps

Run Shell 底栏 Chat Bar：GLOBAL 范围 ｜ 当前 WORKTREE 范围（后者绑定焦点 checkout）
```

左侧主导航是 Project → Branch → Engineering Run → Worktree；Project Worktree Index 是可选的跨层汇总管理视图。Branch 下的 Run 管理与该远端 Branch 绑定的本地 Worktree，选中 Worktree 后打开其所属 Run 的右侧 tabs；除 CLI/Git 操作和显式焦点筛选外，Inbox、WorkItem、Task Card、Canvas、Workflow、BI/Benchmark 与 Plugin 的 canonical scope 是 `engineering_run_id`。`EngineeringRun` 不等于 `TaskExecutionRun`，后者仅表示某个 WorkItem 的单次执行。当前 `/worktree?.../group` 及 Worktree-scoped Group API 是迁移兼容路径，不改变 Run-owned 数据事实；Project 范围的 Worktree Graph Overview 继续用于跨 Branch/Run 冲突、依赖与全景查看，Run Infinite Canvas 用于跨该 Run 的协作，两类画布必须区分路由和数据范围。

Index 是管理面而非仅供跳转的清单：必须汇总 owner、Agent Session、Runtime、分支、生命周期、PR、dirty/ahead/behind、健康/风险和锁状态，帮助用户发现多 Agent Worktree 的归属漂移与执行冲突。`git_lock { state, source, observed_at }` 专指 Git Worktree retention lock（保护 Git 管理记录免遭 prune，并影响 Git 对该 Worktree 的移动/删除）；它不是 Agent 活跃状态、文件编辑互斥锁或独占租约，`unlocked` 不表示没有 Agent/Session 正在工作。缺少 Runtime/observer、读取失败、时间戳缺失或观测超过 30 秒时为 `unknown`，不得从持久化 `locked` 字段推断当前 Git 保留锁状态。Index 对每页观测最多并发 8 个请求、单个 provider 最长等待 2 秒、总等待最多 3 秒；未按时返回的项目显示 `unknown`。`unknown` 与 `unlocked` 均不能证明 Agent 已停止；物理清理还须读取独立的 Agent/Session/Runtime 活跃状态，在 drain 后重新观测 Git 保留锁，并由 Repository/Runtime 执行器确认。生产视图必须以当前用户授权的服务端投影为准；请求失败时显示错误且不回退到本地 seed。归档/恢复必须先生成有版本条件的短时计划，再由用户二次确认并写入审计；归档不得被解释为删除本地 Git checkout。创建/导入、停止执行和物理清理继续走各自的授权 Runtime / Repository 生命周期接口。

### 50.2 术语与共同上下文

| 术语 | 定义 | 事实来源 |
|---|---|---|
| Cloud Branch | Project 中的远端合并目标/分支身份，绑定仓库与 Git ref | Project / Repository / Branch registry |
| Engineering Run | Branch 下的跨 App 工作区，归属 Run Apps、Work Item 视图、Canvas 与 Run BI；可管理多个绑定该 Branch 的本地 Worktree | Run Domain / `engineering_run_id` |
| Worktree Group | 选中某个 Run Worktree 后显示的 Run Shell；Run 是授权及 App 数据所有者，Worktree 是焦点与本地 checkout | EngineeringRun + 可选 `focus_worktree_id` |
| WorkItem | Jira 类业务任务，承载状态、负责人、Backlog、Sprint、关系和 Guard | `domain-work-item` |
| Task Card | Engineering Run 下的任务入口；由 Run Task Management 视图访问，并在卡内呈现 CLI 与 Agent/LangGraph 状态 | Run-scoped WorkItem / TaskExecutionRun / AgentSession 投影 |
| Canvas Element | Canvas 中的节点、连线、Frame 或绑定；可引用任务卡，但不成为第二套任务事实 | Canvas 聚合与实体链接 |
| Task CLI Session | 从任务卡发起、绑定具体 Worktree 的受控本地命令会话 | Local Runtime / Agent Policy / Audit |
| Run Context | `tenant_id / project_id / repository_id / branch_id / engineering_run_id / actor_id` 与独立 WorktreeFocus 的服务端授权上下文；checkout workspace 只属于 focus | Application Authorization Layer |

每个 Run App 必须先取得服务端解析的 Run Context，再读取 Run projection 或调用其事实所有者 API；Worktree/Git/CLI 命令还必须按选中 Worktree 单独授权。任一实体链接使用带类型的 `EntityRef`，至少支持 `branch`、`engineering_run`、`work_item`、`task_card`、`task_execution_run`、`agent_session`、`worktree`、`canvas_element`、`automation_flow`、`comment`、`relation` 和 `plugin_resource`。

### 50.3 同级应用、任务卡与单一任务事实源

每张 Task Card 均可打开关联的受控 CLI / Agent Session 面板；会话归属该 Task Card 和明确选定的 Worktree，不作为 Worktree 导航树中的独立同级入口。

Inbox、Multica Task Lifecycle、Jira-class Work Management、Task Cards、Infinite Canvas、Workflow、Run BI/Benchmark 和 Run Plugin Apps 都是 Engineering Run 的同级 App 能力入口。**Task Cards 与 Infinite Canvas 必须在 Run tabs 同级显示**；Task Card 可从 Inbox、Multica 生命周期或 Jira 类 Board / Backlog / Sprint 打开，但不是其中任何 App 的私有子对象。点击 Worktree 只打开其所属 Run 并设置焦点；一个 Run Canvas 可引用该 Run 的多个 Worktree。Multica 提供 claim / execution / review gate / failed / poisoned-session 等执行生命周期能力；Jira 类提供 Board / Backlog / Sprint / Relation 管理能力；二者共用 Run-scoped WorkItem、Workflow、Planning、Relation 和 Audit 的事实来源。

在线 Group UI 必须从 canonical lifecycle 状态、版本和 review state 计算可用动作；Multica、Jira 和 Task Card 操作同一 WorkItem。命令必须携带当前 `expected_version`、`Idempotency-Key` 与 `correlation_id`，经服务端 lifecycle guard、writer ACL 与事务审计。客户端成功或发生冲突后刷新授权投影；`pending_review` 不得被客户端绕过而标记为 `completed`。

Canvas 可以创建任务链接、定位任务、展示状态、发起受权的任务命令和展示 Agent/CLI 进度。Canvas 不持久化 WorkItem 状态副本，也不直接修改其他 App 的前端状态。状态与关系变更必须经过 Application Command、既有 Workflow Guard、领域事务与 Outbox 事件，再投影回 Task Management、Canvas、Chat 和插件。

| ID | 要件 | 优先级 |
|---|---|---|
| WTG-001 | 用户选择 Project 后，左侧第一级列出该 Project 当前 actor 可访问的 Cloud Branch；展开 Branch 显示该 Branch 下的 Engineering Run，再展开 Run 显示其绑定 Worktree | P0 |
| WTG-002 | 点击 Run 内 Worktree 后才显示所属 Run 的 Inbox、Multica、Jira 类管理、Task Card、Infinite Canvas、Workflow/LangGraph、Run BI/Benchmark 与插件同级 tabs，并共享 Run Context | P0 |
| WTG-003 | Worktree 归档、删除观察或失联后，群组资源必须保留可追溯关系并按 Project Policy 转为只读、恢复或归档状态 | P1 |
| WTG-004 | Project Worktree Index 必须可跨 Branch/Run 比较 Worktree owner/Agent、绑定 branch、状态、Runtime、PR、冲突/锁和最近活动，并从同一处进入受权管理动作 | P0 |
| WTG-005 | Worktree 切换只更新当前 Engineering Run 的 focus/CLI checkout target；Run-owned App 数据仍按 `engineering_run_id` 查询；切换 Project/Branch/Run 时必须清除旧授权投影 | P0 |
| WTG-006 | 主导航深链必须保留 Project/Branch/EngineeringRun/Worktree 身份；旧 `/worktree?project_id=...` 与 `/worktree/{id}/group` 在迁移期间仅作为可验证兼容路由，Index 不得被通用任务列表替代 | P0 |
| WTG-007 | 已装配宿主认证时，Project Worktree Index 必须读取授权 API 投影并支持 owner/state/archive 筛选与稳定游标续页；401/403/网络错误不得混入本地 seed 作为生产结果 | P0 |
| WTG-008 | Index 的归档/恢复操作必须使用当前 Worktree version 创建短时 plan 并二次确认；执行绑定未解除时归档必须失败；归档只改变平台可见状态，不删除 Git checkout | P0 |
| WTG-009 | Worktree owner 必须可由有权管理员转派给当前 Project 的有效成员；候选人只能来自当前认证成员目录，转派使用 Worktree version、短时 plan、二次确认、幂等与审计；成员目录不可用时不得开放自由文本或 seed 转派 | P0 |
| WTG-010 | Project Worktree Index 必须分开呈现持久化 `locked` 标记与 Host Runtime 实时 Git Worktree retention lock observation；该 Git 锁只保护 Git 管理记录，不能表示 Agent 活跃或文件编辑互斥；缺少 provider/runtime、失败、超时、无效或超过 30 秒的观测为 `unknown`；unknown/unlocked 均不能证明 Agent 已停止，物理清理须检查独立活跃状态并在 drain 后重新观测 | P0 |
| WTG-011 | Project Worktree Index 必须以 `project:read` + 当前 membership 查询脱敏的已绑定 Repository 清单（仅 ID/name/default branch，不含 URL/路径），并提供授权候选发现、创建与导入；仅接受 Project 已绑定 repository_id、经过 Git ref 校验的 branch/base_ref 或可信 Host Runtime opaque candidate_id；拒绝客户端路径、仓库 URL 和 Git 命令；副作用前重验 membership 与 Project-Repository binding，以 Idempotency-Key/correlation_id 幂等持久化 operation、Worktree projection、Audit/Outbox 后返回 202；缺少 provider 返回 503；Index UI 不得回退本地 seed，须验证 Project/Repository/candidate/receipt 关联并在受理后刷新 Index | P0 |
| WTG-012 | 生产 Project Selector 必须从当前 Bearer actor 在当前 tenant 下的有效 Project membership 目录加载 Project；返回最小 `project_id`/role 投影并稳定分页，禁止用本地 seed 冒充授权范围；当 Project 主数据源未接入时，标签仅能显示 ID，不得从非权威本地数据补名 | P0 |
| WTG-013 | Task Contract 必须版本化保存 goal、scope、dependencies 与 acceptance criteria；每次 Run 固定引用 contract version 与输入/验收快照，变更不改写已有 Run | P0 |
| WTG-014 | 每个真实执行尝试拥有独立 TaskExecutionRun；同一 Task 可有多个 Run；Worktree 为可选执行上下文快照，清理 Worktree 不得删除或级联删除 Task、Run、事件和证据；TaskExecutionRun 与 `group_chat_run` 身份及生命周期分离 | P0 |
| WTG-015 | Run 事件必须分别记录执行器状态、Agent 声明、验证、人工接受/返工和集成状态；Evidence 只存脱敏 metadata/digest/受控 locator，不复制大日志、Secret、原始 transcript 或模型隐式推理；缺失值保持 unknown，实际成本与估算成本分开 | P0 |
| WTG-016 | Project 级 Quality & Improvement 从 Run facts 派生 BI；每个指标记录公式、分子/分母、单位、时间窗、覆盖率和版本，并可下钻 Task/Run/Evidence；按类型/复杂度分层，禁止 LOC/commit/Agent 数作为生产力指标 | P1 |
| WTG-017 | Benchmark 使用固定且版本化的 Task、repo、环境、验收标准和评分口径，隔离 tuning/holdout 并记录不可复现条件；改进 proposal 必须可追溯、可回滚，候选策略不得降低验收标准 | P2 |
| WTG-018 | `EngineeringRun` 必须与 Task 的 `TaskExecutionRun` 区分；EngineeringRun 汇总协作范围/Worktree set/App tabs/Run BI，TaskExecutionRun 记录一次执行尝试并保留可空 Worktree snapshot | P0 |
| WTG-019 | 每个 Run App 必须声明 canonical owner scope、读写 API 和 capability；前端同级 App 不等于独立微服务；跨域写通过 owner API/领域事务/Outbox + 幂等 Inbox 投影，不共享表写入 | P0 |
| WTG-020 | Project 导航必须提供 Project Worktree Index 入口；客户端已选 `project_id` 仅是深链提示，目标页必须重新读取当前 actor 的 membership 并授权 Index；无有效项目时要求从服务端目录选择，禁止用固定 repository/worktree ID 或本地 seed 填充导航 | P0 |
| WTG-021 | Project → Cloud Branch → Engineering Run → Worktree 目录按层懒加载并有界；Branch 来自可信 SCM 身份，不从 checkout 分支字符串归类生成；三层 current grant 每次重验；只有叶节点解析完整 Run/focus 后才建立执行焦点；缺会话或绑定时明确阻断 | P0 |
| TCI-001 | Multica 生命周期和 Jira 类计划视图必须投影同一 Run-scoped WorkItem；Task Card 索引作为 Engineering Run 下的平级 App 访问该任务；不得产生并行任务状态机或第二个任务事实源 | P0 |
| TCI-005 | Multica、Jira 与 Task Card 的生命周期动作必须调用同一个 WorkItem lifecycle command，并遵循合法状态迁移、review gate、writer ACL、版本冲突、幂等、correlation 与审计规则 | P0 |
| TCI-006 | `in_progress` claimant 可提交当前 WorkItem 进入 `pending_review`；仅当前 Worktree 的非 claimant `tenant_admin` / `project_admin` / `developer` 可通过或驳回；驳回必须有理由；三类命令均校验 `expected_version`、幂等键、scope 与审计；通过转为 `completed`，驳回转为 `failed` | P0 |
| CAN-001 | Canvas 必须作为 Engineering Run 内与 Task Card 同级 App，并能互动 Run 内 WorkItem、Task Card、AgentSession、多个 Worktree、Relation 与自动化流程 | P0 |
| CAN-002 | Canvas 发起的任务写操作必须经过 Task Domain Command 与 Guard；结果由事件回写所有订阅 App | P0 |

### 50.3.1 Project scoped Worktree 创建与导入

创建/导入是 Worktree Index 的受授权生命周期命令。浏览器仅选择当前 Project 已绑定的 repository，并在导入时引用 Host Runtime 返回的不透明候选 ID；路径和远端 URL 由服务端配置解析，不能由客户端指定。Host Runtime provider 必须在 Git 副作用前重新验证当前 actor membership、Project-Repository binding 与 Worktree 冲突，并幂等持久化 operation、归属投影、Audit 和 Outbox。provider 或受信任 repository binding 未配置时，请求失败关闭且不得生成“成功”Worktree。

### 50.4 任务卡内 CLI

任务卡允许打开 CLI，以便人或 Agent 在卡内完成实际工作。CLI 会话必须绑定 `work_item_id`、`worktree_id`、Runtime、Agent Policy 和已批准的启动配置；界面在任务卡中展示会话状态、输出流、取消入口与返回的验证结果。

CLI 由 Local Runtime 在目标 Worktree 中通过交互式 PTY 启动，并持续校验 Repository、Worktree、允许路径、工具类别、命令类别、Secret Scope 与运行时间限制。进程环境必须从空环境开始，只注入批准的静态变量和独立授予的 secret capability；终端输入、尺寸、输出字节流与退出状态由 session 明确关联。PTY 不是 sandbox，OS-enforced sandbox/path jail 未就绪时必须 fail closed。每次启动、输入、输出摘要、拒绝、取消和结果均关联 `task_card_id`、`work_item_id`、`worktree_id`、`agent_session_id`（如有）和 `correlation_id` 写入审计。卡片在多个 Worktree 中出现时，用户必须选择当前执行 Worktree，系统不得隐式跨 Worktree 启动会话。

| ID | 要件 | 优先级 |
|---|---|---|
| TCI-002 | 任务卡可在经授权的关联 Worktree 中打开受控交互式 PTY CLI Session，并将上下文、命令结果和审计关联回任务卡 | P0 |
| TCI-003 | CLI Session 被拒绝、断开、超时或 Runtime 失联时，任务卡必须显示可定位状态，且不得误写 WorkItem 完成状态 | P0 |
| TCI-004 | CLI 输出进入 Agent Context 前必须按 Untrusted Content 处理，并沿用 Prompt Injection、Secret Redaction 与 Content Retention 规则 | P0 |
| TCI-007 | 执行 grant 必须使用 Ed25519 私钥签名并包含 key id；Local Runtime 仅持有可轮换的受信公钥，并在轮换期间保留旧 key 至已签 grant 过期（最长五分钟）；必须先验签，再校验 scope/profile/checkout 并原子消费 nonce；签名不得替代实时 ACL 或 Runtime health 检查 | P0 |
| TCI-008 | Task Session start 必须验证 Bearer actor、`agent_session:start`、当前 Project writer membership、canonical WorkItem-Worktree association、`in_progress`/active claimant、expected lifecycle version、已分配 Runtime 与 server-approved profile；通过 Idempotency-Key 和 correlation ID 防止重复启动，request fingerprint 必须绑定 tenant/actor/project/repository/worktree/task/runtime/profile/lifecycle version；只有实际启动成功且签发 ≤60 秒 single-use attachment ticket 后才返回成功，未装配 provisioner / 不健康 Runtime / 不可用 sandbox 时 fail closed | P0 |
| TCI-009 | Task CLI 必须通过交互式 PTY 转发受限 stdin、输出字节、resize 与退出状态；attachment 前输出由有界 FIFO 以背压保留；manager 关闭时终止子进程；子进程从空环境启动并仅接收获批配置；PTY 不构成安全隔离，OS sandbox/path jail 未就绪时不得 spawn | P0 |
| TCI-010 | Task Card UI 通过当前 Bearer 用户会话发起 Session start，发送 lifecycle version、管理员批准的 profile UUID、correlation ID 和 Idempotency-Key；attachment ticket 仅保存在页面内存并作为 WebSocket 首帧单次提交；服务端 Hello 前不开放 stdin；断线不复用 ticket，只有获取新 ticket 后才可重新 attachment | P0 |
| TCI-011 | Task Card UI 必须支持状态刷新、显式取消和人工重连；重连前先查询当前 session，仅 `running` / `disconnected` 可申请新票据；每个操作使用新 correlation ID；页面刷新后不可凭旧 ticket 恢复，须由独立授权的 session listing/recovery 契约恢复 session 关联 | P0 |
| TCI-013 | Task Contract 必须以 Master/SCD2 保存 goal、scope、dependencies 和 acceptance criteria；每次 Run 固定版本与 input/acceptance snapshot，历史 Run 不随合同更新改变 | P0 |
| TCI-014 | 每个真实执行尝试拥有独立 TaskExecutionRun；Task 可有多次 Run；Run 对 Worktree/repository/runtime 只保存可空历史 ref，Worktree 清理不级联删除 Run/Event/Evidence；与 `group_chat_run` 分离 | P0 |
| TCI-015 | Run timeline 必须分别记录 execution、Agent declaration、verification、human acceptance/rework、integration、intervention、failure 和 cost；未知值保持 null，实际成本与估算成本及单位分列 | P0 |
| TCI-016 | Run Evidence 仅保存经过脱敏的类型/摘要/digest/受控 locator metadata；不复制大日志、Secret、完整 transcript 或模型隐式推理；Task/Run/Evidence 查询逐次执行当前 Project/Task ACL | P0 |

### 50.5 Canvas 与任务、Jira 类能力的互动

Run Canvas 是 Engineering Run 的任务编排和协作表面。任务卡可拖入 Canvas、从 Canvas 跳回 Inbox/Board/Backlog/Sprint、在元素上显示任务状态和 Agent/CLI 摘要；Canvas 可通过实体链接显示该 Run 下 Worktree、Blocked、Dependency、Feedback、Review、Validation 与自动化 Flow。Project Worktree Overview Graph 是另一项 Project 级聚合视图，不能与 Run Canvas 共用 owner 或 layout。

Run Canvas 在 Engineering Run 范围内创建或关联 WorkItem，再建立 Canvas Element Link。用户可以把该 Run 中尚未关联的 Task Card、AgentSession 和多个 Worktree 放到 Canvas；所有实体只以带类型的引用呈现。Canvas 的 canonical owner/API scope 是 `engineering_run_id`，Element 可选关联 `worktree_id`；具体 CLI/Git 操作必须显式解析当前 Worktree。当前 `/api/v1/worktrees/{worktree_id}/canvases...`、Canvas-Worktree 外键和单 Worktree Element relation 是兼容实现切片，迁移到 Run-owned Canvas Master、Run membership ACL/API 前不得宣称目标模型已实现。迁移期间，当前已实现的 CAS、幂等、审计、Outbox 与 fail-closed 要件继续有效；新 Run API 必须保留版本控制、审计、授权投影刷新与不回退 seed 的同等守门。

| ID | 要件 | 优先级 |
|---|---|---|
| CAN-003 | 从 Run Canvas 创建或关联任务时，必须保留 `engineering_run_id`、`work_item_id`、`canvas_element_id` 与 `correlation_id`；可选 Worktree 引用不改变 Run owner | P0 |
| CAN-004 | Task Management 状态、Agent 进度、CLI 结果与 Validation 变化必须以实时投影更新 Canvas；Canvas 订阅失败后必须可重放 | P1 |
| CAN-005 | Canvas、Board、Backlog 和 Sprint 对同一 WorkItem 的跳转必须保持同一实体引用与同一权限判定 | P0 |
| CAN-006 | Canvas viewport、frames 与 visual connectors 必须作为 EngineeringRun-scoped 版本化文档持久化；更新携带 expected version 与 idempotency key，冲突不得覆盖新版本；Frame / connector 只能引用当前 Canvas 的元素 | P0 |
| CAN-007 | 多 Canvas 必须可在当前 Engineering Run 下列出、选择、新建和通过 `canvas_id` 分享深链；每次读取与写入仍须由服务端校验 actor、Run membership 与 Canvas 归属 | P0 |
| CAN-008 | 用户必须能把当前 Engineering Run 未关联的 canonical Task Card 放到选定 Canvas；Element 与 typed EntityRef 在同一认证幂等事务内写入，并过滤已关联任务 | P0 |
| CAN-009 | live Run Canvas 的元素位置可由用户拖动并通过 Run-scoped Element CAS 保存；请求使用 `update_mode=position` 且只允许提交坐标字段，服务端保留当前其它字段；携带 `expected_version`、`Idempotency-Key` 和 `correlation_id`；冲突不得覆盖新版本，且必须刷新授权投影 | P0 |
| CAN-010 | live Canvas 可编辑 Frame 标题/几何/演示标记、连接线展示样式及无实体绑定的文本/便笺内容；删除 Element 必须用 Worktree scope、版本和幂等键，且在同一事务移除 Frame/connector 悬空引用，不删除 canonical WorkItem；locked Element 不允许修改或删除 | P0 |
| CAN-011 | live Canvas 中未锁定 Element 的尺寸与旋转角度可单独编辑；请求使用 `update_mode=geometry` 且只允许提交宽、高、旋转字段，服务端保留位置、内容、实体引用及其它字段；携带 Element `expected_version`、幂等键和 `correlation_id`；冲突不得覆盖新版本，并须刷新授权投影 | P0 |
| CAN-012 | 点击某个 Run Worktree 后显示所属 Engineering Run 的 Canvas App；Canvas 数据/API 由 `engineering_run_id` 授权，可引用 Run 中多个 Worktree；旧 Worktree-scoped Canvas 路由仅作兼容且不得作为新的事实所有者 | P0 |

### 50.6 固定底栏聊天与 LangGraph 范围

App Shell 只提供一个固定底栏 Chat Bar。用户在发送前选择 `WORKTREE` 或 `GLOBAL` 范围，并可附加当前任务卡、Canvas Element 或其他 EntityRef。选择器必须始终可见，消息历史、checkpoint、工具调用和审计均记录范围。

- `WORKTREE`：L0 只加载当前 Run Context、当前 focus Worktree 的任务/画布/运行时摘要和该范围内已授权工具；由此创建的 L1 Task Card 归属当前 `engineering_run_id`，执行时显式绑定焦点 `worktree_id`。
- `GLOBAL`：L0 用于跨 Project/Branch/Run/Worktree 统筹、规划、合并、拆分、依赖、批量操作和汇总。任何会写入数据或启动执行的指令必须显式列出目标 Project/Run/Worktree，系统对每个目标独立解析授权、状态和审计校验。
- L1 Task Card Agent 保持任务卡级隔离。跨卡操作、Canvas 发起的编排和插件发起的任务操作由 L0 的 Task Operations Manager 协调，L1 之间不得直接通信。

LangGraph checkpointer 只保存非敏感 workflow state 和服务端 thread/run 关联；不保存可复用的 `GroupContext`、JWT、permission snapshot 或 Plugin capability grant。thread ID 由服务端生成，并映射到 Chat Session；不得复用 WorkItem、Task Card 或 Agent Session ID。每次 start、resume、interrupt approval、从 checkpoint replay 与 capability/tool call 都必须重新解析当前 GroupContext、目标集合及 Plugin grant。LangGraph 节点可从 checkpoint 边界重新执行，所有外部副作用必须调用带稳定幂等身份的领域命令/outbox；不能把 checkpoint replay 当成“只执行尚未发生的代码”。

Group Chat 的 PostgreSQL adapter 在同一个 tenant/actor-scoped transaction 中写入 session 与 server-generated thread ID、user Transcript metadata、加密 payload、queued Run intent、30 天幂等 receipt、append-only dispatch event 和 append-only Audit facts；提交成功后才返回 queued receipt。Session、Transcript metadata、dispatch 与 Audit 为 T（不可改写并保留审计事实）；正文 payload、Run 与幂等回执为 W。正文必须通过显式注入的 `TranscriptBodyProtector` 使用认证加密封套保护，密文与 wrapped data key 分开存入有期限 payload 表，默认最多 90 天并遵循适用 Project Policy；到期须物理清除密文或销毁专属数据密钥。Transcript 的 append-only 审计事实不授权永久保留正文。当前 adapter 已拒绝直接写明文并提供 protector seam，但尚无真实密钥服务、密钥轮换/销毁、正文清理 worker 或授权导出/删除流程，仍禁止部署到生产，须先完成实现与验收。Outbox 只包含 run 引用，不复制正文、JWT、授权快照或能力 grant。API 当前没有安装该 adapter；provider 缺失或数据库 schema 不可用时必须 fail closed，不能返回 202。

| ID | 要件 | 优先级 |
|---|---|---|
| CHAT-001 | 固定底栏提供 `GLOBAL` / `WORKTREE` 范围选择，并将范围和实体引用写入聊天、checkpoint、工具调用与 Audit | P0 |
| CHAT-002 | `WORKTREE` 范围只能读取并执行该群组已授权的上下文和工具；`GLOBAL` 写操作必须提供明确目标 Worktree | P0 |
| CHAT-003 | 每次聊天提交必须由 Bearer actor 提供显式范围和幂等身份；服务端逐目标解析当前授权 GroupContext，任何目标或实体引用越权时整批拒绝，且不得向 L0 派发；缺少 workflow provider 时 fail-closed 返回服务不可用；省略 correlation 时以幂等键作为稳定默认值 | P0 |
| CHAT-004 | GLOBAL scope 必须从当前 actor 的有效 Project membership 中发现非归档 Worktree，并以有界游标分页返回最小目标信息；发现结果不能替代提交时的实时授权 | P0 |
| CHAT-005 | workflow persistence adapter 必须在一个事务中写入 user Transcript metadata 与受保护 payload、queued Run、幂等 receipt、dispatch outbox 与 Audit；只在 commit 后返回 202；同 key 同 fingerprint 重放相同 receipt，不同 fingerprint 冲突 | P0 |
| CHAT-006 | Transcript 正文必须使用批准的 at-rest 加密与密钥轮换方案，并遵循 Agent Policy/Project Policy 的保留期限；到期必须物理删除密文或完成 crypto-erasure，append-only 审计只保留不含正文的事实；提供授权导出/删除流程；明文正文不得写入日志、outbox 或 checkpoint | P0 |
| LGS-001 | L0 与所有 L1 的状态、checkpoint、工具调用和流式事件必须携带适用的 Worktree Scope | P0 |
| LGS-002 | 所有跨任务、Canvas 或插件编排必须由 L0、领域命令和事件实现，保持 Task Card 的隔离边界 | P0 |
| LGS-003 | checkpoint 不保存授权凭据或可复用 permission snapshot；thread ID 由服务端生成并与业务 Session/Task ID 分离；resume、interrupt、replay、工具调用均实时重授权，副作用经幂等、可审计的领域命令/outbox 执行 | P0 |

### 50.7 Group App Plugin 热插拔

插件作为 Worktree Group 内的可选 App 或能力提供者接入。插件 Manifest 必须声明标识与版本、兼容范围、UI surface、路由、命令与工具能力、事件订阅、EntityRef 类型、所需权限、数据模式版本、迁移策略和资源限制。插件通过稳定 Application API / ACL 调用领域能力；不得直接连接数据库、读取其他 App 的前端状态或绕过 Group Context。

插件生命周期为：`registered → configuring → active → degraded → draining → disabled`。启用前校验来源、签名或可信发布策略、版本兼容性、权限授予和数据迁移；禁用时先停止新调用并排空活动会话，再撤销 UI 入口、命令和工具能力。业务事实、Canvas 链接、CLI 审计与历史事件在禁用后保持可读可追溯。

生产导航只能由服务端返回的当前 Worktree Group App Registry 投影生成：只有该 Worktree 已启用且授权有效的插件才作为同级入口出现；停用或撤权后入口必须随授权投影移除。存在 API provider 时，Group UI 必须消费该 Worktree 的授权投影，加载、错误或 provider/session 切换期间不得显示旧投影或回落本地预览；只有未安装 API provider 的原型环境才可显示本地预览开关。客户端本地开关不构成注册、启用、capability grant 或运行时热插拔。

`GET /api/v1/worktrees/{worktree_id}/group-apps` 是 Group Shell 的生产导航投影读取入口，要求当前 Bearer actor 有 `worktree:read` 且仍属于该 Project。Registry provider 必须在读取期间重新验证当前 Project membership、Worktree binding、插件兼容状态和 actor/Worktree capability grant；只返回当前 active 的最小导航信息（plugin ID、manifest version、label、sort order），不得返回 capability secrets、外部 URL 或原始授权快照。缺少 provider 时返回 503；UI 预览开关不得回退/合并进该生产投影。

Plugin Registry 持久化拆分为 manifest/version、Worktree Registry revision、Worktree binding、actor `group_app:open` grant 和 append-only Audit。Manifest 的 `verification_status=verified` 只能由受信任的签名/来源验证与注册流程写入；该状态值本身不是信任根。Registry read provider 只读，当前主程序使用 Host API version 1；生命周期命令与 manifest ingest 未实现前，不能注册、启用或授予新插件。迁移文件需先依序完成 Worktree/Project 主表及关联 migration，再由受控发布作业评审和部署。

| ID | 要件 | 优先级 |
|---|---|---|
| PLG-001 | Plugin Registry 必须声明 UI、能力、订阅、权限、版本兼容性与支持范围，并以群组级别启用 | P0 |
| PLG-002 | 插件必须支持注册、配置、启用、降级、排空和禁用的可审计热插拔生命周期 | P0 |
| PLG-003 | 插件产生的命令、事件和资源引用必须携带 Group Context、`correlation_id` 与 `schema_version` | P0 |
| PLG-004 | Group Shell 只能从服务端当前授权 Registry projection 渲染插件同级入口；projection 查询须重验 Project membership、Worktree binding、兼容状态与 capability grant；provider 已安装但读取中或失败时不得显示旧投影或预览兜底；provider 缺失时仅允许明确标记的本地原型预览 | P0 |
| PLG-005 | Plugin Manifest、Registry revision、Worktree binding 与 actor `group_app:open` grant 必须按 Master/SCD2 保存；Registry Audit 必须为 Transaction/append-only；五张表均须 tenant/worktree RLS 且禁止 Master 物理删除 | P0 |

### 50.8 跨 App 事件、审计与验收

跨 App 写操作统一遵循下列路径：

```text
Task Management / Canvas / Chat / Plugin
  → Group Context Resolver
  → Authorization + Policy Guard
  → Application Command
  → Domain Transaction + Transactional Outbox
  → Versioned Domain Event
  → Task / Canvas / Chat / Plugin / Search / Audit Projections
```

事件至少携带 `schema_version`、`tenant_id`、`project_id`、`worktree_id`（适用时）、`actor_id`、`event_id`、`causation_id`、`correlation_id` 和 `idempotency_key`。NATS JetStream 是领域事件权威；Canvas 或 App 的实时流只负责投影与分发，不建立第二份业务事件事实源。

Group UI 的 API 认证必须由宿主登录会话注入异步 access-token provider，客户端每次请求向 provider 取当前用户 JWT。API 适配层不得读取或持久化 token、使用共享 API key 伪装用户、将 token 放入 URL/WebSocket、或自行刷新 token；缺少 token 时必须在网络请求前 fail-closed。Bearer 只通过 HTTPS `Authorization` header 发送；仅允许 loopback 本地开发 API origin 使用 HTTP。API 请求禁用 cookie 凭据与缓存。401/403 返回宿主会话层处理，服务端独立解析 actor 并逐请求重验 scope、Project membership、Worktree 与实体权限。

#### Task Contract 与 Task Execution Run

`Task` 是跨尝试持续存在的目标与验收约定，保存 goal、scope、dependencies、acceptance criteria 及其版本。`TaskExecutionRun` 是一次独立执行尝试；一个 Task 可有多个 Run，重试必须生成新的 Run。Worktree 是某次 Run 的可选执行工作区绑定和历史快照，不是 Run 的父级容器。Run 保存 `worktree_id`、repository/ref 与 start/result commit 等当时可观测引用，但这些是快照标识，不建立阻止 Worktree 清理的外键；Worktree 清理后 Task、Run、事件与证据索引仍可查询。

每张当前 Task Card 必须有唯一 canonical Engineering Run owner，身份是完整 `(tenant_id, project_id, repository_id, branch_id, engineering_run_id)` tuple；Worktree 仅通过同 Run 的当前关联表示 checkout/focus，不拥有 Task Card。Run list/read、Worktree 兼容读写、Canvas 创建与 CLI/Task Run 查询都必须重新校验该 tuple 及当前 Project/Branch/Run grants。新建任务不能省略 Run owner；历史未归属数据不按所在 Worktree 猜测回填，也不参与 Run App 或 CLI 执行，直至经过显式授权的 SCD2 reconciliation。Task owner 变更必须写新 metadata 版本；跨 Run Worktree 关联和 Worktree 重绑定由数据库约束拒绝。

Task metadata 当前版本写入与 `task_run_outbox` 的 append-only Transaction event 同事务提交，事件仅含 typed owner、metadata version、actor、correlation 与 schema version 等有限字段，不携带标题、描述、凭据或执行输出。消费者使用有界页、复合游标与幂等 inbox；Outbox 尚未部署/验证时不能宣称跨 App 投影已闭环。

Run 至少记录 task/run ID、开始/结束时间、执行渠道、Agent/Model/Skill/Orchestrator 版本、repo/ref/commit、输入与 acceptance snapshot、验证事件、人工介入与返工、成本、失败类别及 acceptance 结果。状态维度必须分开记录：执行器状态、Agent 声明、验证结果、人工接受/返工、集成结果。`agent_declared_complete` 不等于验证通过、人工接受或已集成；空缺数据为 `null/unknown`，不得填零。`multica.group_chat_run` 是短期 Chat/LangGraph 工作队列，不是 TaskExecutionRun；可以通过引用关联，但不得共用身份或生命周期。

Run detail 采用简明摘要与按需展开：Task Card 的 Goal / Execution / Evidence / Feedback / Compare 页签显示目标、最近 Run、验收进度、人工介入、当前阻塞；原始 tool output 或完整 transcript 不复制进 Run/Tables，也不存模型隐式推理。Evidence 保存经过脱敏的类型、摘要、digest 与受控 artifact locator，禁止内嵌大日志、token、Secret 或原始模型思维过程。未提供的时间、token、成本或验证项保持 unknown；实际成本与估算成本分栏、分单位。

#### Project Quality & Improvement（BI / Benchmark）

BI 是由 Task/Run/Audit/Evidence 派生的 Project 级质量视图，可从 Project Worktree Index 进入；它不新增 Worktree Group 导航层级，也不在 Worktree 与同级 Apps 之间插入 Run 节点。每个指标必须提供公式、分子/分母、单位、时间窗、样本范围、数据覆盖率、metric version，并可下钻到 Task、Run 和 Evidence。Acceptance rate、first-pass acceptance、rework、human intervention、cycle time 与 accepted-task cost 必须按任务类型/复杂度分层；不得把 LOC、commit 数或 Agent 数当生产力。

Benchmark 使用固定任务集、repo commit、环境与验收/评分版本，tuning 与 holdout 样本隔离；在隔离环境重放并记录不可复现条件。候选策略不能降低 acceptance criteria 或修改评分口径。改善流程为重复失败 → 版本化 proposal → 隔离测试 → 固定 benchmark 对比 → 有授权地采纳 → BI 复查；策略变更可追溯、可回滚，不能自行改写历史 Run 的评分标准。

| 验收 ID | 受入基准 |
|---|---|
| AC-WTG-001 | Project 是左侧首层；其下按 Cloud Branch → Engineering Run → 本地 Worktree 展开。Project Worktree Index 只汇总跨 Branch/Run 状态；选中 Run 内 Worktree 后才打开该 Run 的 App tabs。Multica/Jira 类任务管理、Task Card、Infinite Canvas、Workflow/LangGraph、BI/Benchmark 和已启用 Plugin Apps 都归属同一 Run；Task Card 与 Canvas 是同级 App，CLI / Agent Session 从任务卡内打开 |
| AC-WTG-002 | 展开另一 Worktree 后，各 App、实体查询、底栏 Scope 和实时订阅同步切换；Project 切换后旧项目的 Worktree 不出现在列表和 Group Context 中 |
| AC-WTG-003 | Worktree 清单能显示 owner/Agent、branch、status、Runtime、PR、冲突/锁与最近活动；管理动作经权限、确认和幂等校验并写审计 |
| AC-WTG-004 | Project Worktrees 视图进入 `/worktree?project_id={project_id}`；直接访问、复制和刷新 Index / Group 深链后仍解析同一 Project 与 `worktree_id`；缺少或无效 Project 时不能静默切换到另一 Project；所有这些路由均不重定向到通用任务列表 |
| AC-WTG-005 | live Index 从授权 Project Worktree API 投影读取 owner/执行绑定/健康信号，按 owner/state/archive 筛选并按 cursor 续页；API 失败时显示错误且不显示本地 seed 结果；没有宿主 provider 时界面必须明确标为本地预览 |
| AC-WTG-006 | 归档/恢复必须先以当前 version 生成有期限 plan，再经用户二次确认；活动 Agent/Runtime 绑定阻止归档；确认后刷新授权 Index；此动作不得运行 `git worktree remove` 或删除 checkout |
| AC-WTG-007 | owner 候选只来自当前 Project 的认证成员目录；仅 `tenant_admin` / `project_admin` 可发起转派；用户选择候选后，系统以当前 Worktree version 创建 `assign_owner` plan，校验返回 Worktree / 操作 / 到期时间，并经二次确认；服务端在 plan / confirm 重验角色与候选成员状态；确认成功后重载授权 Index，失败、过期或成员目录读取异常时不乐观更新 owner |
| AC-WTG-008 | `GET /api/v1/projects/{project_id}/worktree-repositories` 必须使用当前 Bearer actor、`project:read` 与有效 Project membership，且只返回已绑定 Repository 的 ID/name/default branch；name 不得是 URL、主机路径或含控制字符。候选发现与 create/import 使用 `worktree:create`，仅 tenant_admin/project_admin/developer 可写。Index UI 只消费当前 Project 的服务端 Repository/候选投影，严格校验字段 allowlist、ID、分支、候选唯一性与回执的 Project/Repository/correlation 关联，不回退 seed；候选 ID 必须来自 Host Runtime。请求拒绝 path、repo_url 和命令；provider 在 Git 副作用前重验 membership/binding，并在返回 202 前幂等写入 operation、Worktree 归属、Audit/Outbox；缺少 provider 或 durable writer 时必须返回非成功响应且不产生成功投影；成功受理后刷新 Index。 |
| AC-WTG-009 | 生产 Project Selector 必须通过 `GET /api/v1/projects` 读取当前 Bearer actor 在当前 tenant 下的有效 Project membership；接口只返回 `project_id` 与允许的 role，使用稳定 UUID keyset cursor，单页不超过 200，并设置 `no-store`。客户端拒绝额外字段、无效/重复 ID、未知角色和不匹配游标；分页未完成时不得把未加载的深链判作无权，完整加载后未命中才显示不可访问；provider/请求失败时不得回退到本地 Project seed。API session/actor 切换时，Project directory、Worktree Index 与成员角色投影必须立即清除旧 session 数据，在当前 session 的授权数据重新返回前隐藏旧 Project 和管理入口。当前仓库没有持久 Project 名称 SoR，生产 selector 用明确的 `Project {UUID}` 标签，不借本地名称补齐。 |
| AC-TCI-001 | 从任务卡打开 CLI 后，工作目录、Runtime、允许路径和命令策略都与所选 Worktree 一致；越界请求被拒绝并审计 |
| AC-WTI-001 | Project Worktree Index 将 Git Worktree retention lock observation 与持久化 `locked` 字段分开；该 Git 锁只说明 Git 是否保护管理记录免遭 prune/移动/删除，不表示 Agent 活跃、编辑互斥或任务空闲；仅带可信来源且年龄不超过 30 秒的 `locked` / `unlocked` observation 可显示为确定状态；缺少 Runtime/provider、读取失败、缺少或过期时间戳、未来时间戳一律显示 `unknown`；unknown/unlocked 都不得视为 Agent 已停止，物理清理须检查独立活跃状态、完成 session/agent drain、重新观测并经受权 Repository/Runtime 执行 |
| AC-TCI-002 | 在线 Multica、Jira 与 Task Card 对同一任务显示相同 lifecycle version；只显示服务端允许的状态操作；领取/开始/完成/失败/取消/重试使用当前版本和幂等键；pending review 阻止完成；成功与冲突后刷新同一授权投影 |
| AC-TCI-003 | review submit 仅由当前 Worktree 的 active claimant 在 `in_progress` 发起；accept/reject 仅由非 claimant 的 `tenant_admin` / `project_admin` / `developer` 对 `pending_review` 决定；reject 必须给出理由；提交、通过和驳回都使用当前 lifecycle version、幂等键、correlation 与 append-only audit；accept 进入 `completed`，reject 进入 `failed`，跨 Worktree / stale version / 自我评审均拒绝 |
| AC-TCI-004 | 篡改 grant context、签名无效、issuer key id 未知或 payload 无效时，Local Runtime 必须 fail closed 且不得消费 nonce；有效 grant 必须绑定 key id、完整 `TaskExecutionContext` 与版本化签名域，验签后仍需重验当前 ACL / Runtime health；签名私钥只部署在受信任 grant issuer，Runtime 只配置公钥并至少保留旧 key 至在途 grant 过期（最长五分钟） |
| AC-TCI-005 | `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions` 必须使用真实 Bearer actor、`agent_session:start` scope、当前 Project writer membership 和 Worktree/WorkItem canonical link；仅 lifecycle version 匹配、任务为当前 Worktree 的 `in_progress` claimant、且 Worktree 有 Runtime binding 时才委托 provisioner；每个幂等请求最多启动一个 session，重试须比对覆盖 tenant/actor/project/repository/worktree/task/runtime/profile/lifecycle version 的 request fingerprint；响应必须关联 correlation ID、包含新鲜且有效期 ≤60 秒的 attachment ticket 并设 `Cache-Control: no-store`；状态过期返回 409，缺 Runtime/provisioner 返回明确非成功，未通过实时 ACL、grant 验签、nonce、Runtime health 或 OS sandbox 检查时不得 spawn |
| AC-TCI-006 | Task CLI PTY 必须只启动 server-approved executable/argv/cwd，清空宿主继承环境后注入获批变量；输入与 resize 必须有界，PTY output 保持字节流语义并带 session 序号；PTY 不能代替 OS sandbox，未接入隔离执行器、当前 ACL 与 TaskRun Audit 时，任何 REST provisioner 均不得启用该 adapter |
| AC-TCI-007 | Session status / cancel / reattach 必须使用 Bearer actor、专用 `agent_session:read` / `agent_session:cancel` / `agent_session:attach` scope、当前 Project writer membership、canonical Task/Worktree association 和非空 `X-Correlation-ID`；Runtime provisioner 必须再次核验 tenant/project/repository/worktree/task/actor/runtime/session 绑定及当前 ACL/Runtime health，并将操作关联 TaskRun Audit；status 仅返回有限 lifecycle state / exit code / updated_at，不返回终端内容或 ticket；cancel 幂等；reattach 只能为活跃 session 新签 ≤60 秒 single-use ticket 并使用 no-store 响应；归档 Worktree 禁止 reattach；缺少 provisioner 时返回非成功响应，前端不得重用旧 ticket 自动重连 |
| AC-TCI-008 | 页面内保留或由授权列表重新发现的 Task CLI Session 可调用状态刷新与幂等取消；断线或刷新恢复后的人工重连必须先读取当前状态，仅 `running` / `disconnected` 才请求新的一次性 ticket，校验 ticket/session/Worktree/Task 绑定和 ≤60 秒有效期后再重建终端；ticket 不得从浏览器缓存或 URL 还原 |
| AC-TCI-009 | `GET /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions?limit=N` 必须验证 Bearer actor、`agent_session:read`、当前 Project membership 与 canonical Task/Worktree/Runtime 绑定；默认 limit 为 20、允许 1–50，响应仅含最近状态/exit code/updated_at 并设 `Cache-Control: no-store`，不得包含 ticket 或终端输出；缺少 provisioner 时 fail closed，恢复连接另行签发新 ticket |
| AC-CAN-001 | Canvas 中选中任务卡或 WorkItem 节点可打开其详情；任务状态变更实时反映到节点；从 Canvas 变更状态仍经过既有 Guard |
| AC-CAN-002 | Canvas Document 更新仅修改 viewport / frame / visual connector；需通过 Worktree ACL、writer role、expected version 和幂等校验；Frame / connector 越界引用被拒绝，connector 不作为 canonical WorkItem relation |
| AC-CAN-003 | 从 Canvas 新建任务卡时，WorkItem metadata、Multica lifecycle、Worktree association、Canvas Element、typed EntityRef、Audit 和 Outbox 在同一事务提交；失败不得留下部分任务或孤儿画布元素；操作通过 `work-item:write`、`canvas:write` 和同一 Worktree writer ACL，并以一个 `correlation_id` 串联 |
| AC-CAN-004 | Canvas 事件读取必须逐请求重验当前 actor、Project membership、Worktree 和 Canvas scope；使用完整 `(occurred_at,event_id)` 复合游标与有界分页；响应不暴露可能失效的 EntityRef 目标或事件 payload，客户端据事件元数据刷新已授权的 Canvas projection；轮询 API 不得被视作 NATS consumer 或实时推送已完成 |
| AC-CAN-005 | live Canvas 元素移动必须通过当前 Worktree 授权的 Element update API，使用 `update_mode=position` 且只提交 x/y；服务端从当前锁定行保留其它字段，携带 Element `expected_version`、幂等键与 `correlation_id`；版本冲突后刷新投影，不得覆盖更新版本 |
| AC-CAN-006 | live Canvas 可暂存并创建/删除 Frame、将当前 Canvas 元素加入 Frame、创建/删除纯视觉连线；所有改动作为一个完整 Document draft 以固定 `expected_version` 和幂等键显式保存；引用必须属于当前 Canvas；视觉连线不得创建或暗示 canonical WorkItem/Jira relation；冲突不得自动重基覆盖新版本 |
| AC-CAN-007 | Frame 标题、坐标、尺寸和演示标记，以及 connector 颜色、线宽、路由、箭头与标签通过完整 Document CAS 保存；Text/Sticky Note 内容更新只允许无 typed/legacy EntityRef 且未锁定的当前 Canvas Element，使用 `update_mode=content` 且只提交 content，并使用 Element `expected_version`、幂等键和 correlation ID；Element 删除拒绝 locked Element，并在同一事务推进 Canvas Document version、移除 Frame membership/引用该元素的 connector、关闭 Element 与 EntityRef 版本并写 Audit/Outbox；删除仅影响 Canvas 表现，不删除 WorkItem；版本冲突不覆盖新版本 |
| AC-CAN-008 | 未锁定 Element 的宽、高、旋转值以 `update_mode=geometry` 单独更新且只接受 geometry 字段；服务端从当前锁定行保留其坐标、内容、locked/hidden、实体引用及 canonical 任务事实并校验数值范围；写入同一 Element Audit/Outbox correlation，成功或冲突后刷新授权投影；locked Element 不显示可编辑入口 |
| AC-CAN-009 | 浏览器 Canvas Outbox 投影消费者按 Worktree/Canvas scope 恢复已校验的本地持久游标；每页仍逐请求重验授权，并且只有当前授权 Canvas projection 刷新成功后才保存下一复合游标；游标损坏或本地存储不可用时安全退回 at-least-once 重放；该游标不授予访问权、不替代服务端 durable consumer offset 或 NATS JetStream |
| AC-GRP-AUTH-001 | Group API 客户端每次调用都通过宿主注入的 access-token provider 获取用户 JWT，并以 Bearer header 发请求；provider 同时带非敏感 session generation key，宿主必须在用户身份或登录会话切换时更新该 key，客户端立即清除旧身份的数据 projection 并重新加载；provider 缺失时不得调用网络，adapter 不持久化或记录 token/session key、不使用共享 API key、不把 token 放入 URL/WS，拒绝非 HTTPS 的远程 origin，401/403 不得退化为 seed 写入或模拟授权成功 |
| AC-CHAT-001 | 底栏切换 `WORKTREE` / `GLOBAL` 后，消息、checkpoint、工具调用与 Audit 均记录范围；`WORKTREE` 范围不可读取其他 Worktree 上下文 |
| AC-CHAT-002 | `GLOBAL` 发起跨 Worktree 编排时，L0 为每个目标创建明确归属的 L1 卡片，且不存在 L1 到 L1 的直接通信 |
| AC-CHAT-003 | `POST /api/v1/worktrees/{worktree_id}/chat/messages` 必须使用当前 Bearer actor、`chat:submit` scope、UUID `Idempotency-Key` 和显式 scope；WORKTREE 目标固定为路径 Worktree，GLOBAL 必须显式提供 1–20 个不重复目标；服务端先逐一解析当前 Project membership / GroupContext，再验证 EntityRef 所属目标，任何目标未授权时整个请求不得调用 L0；相同幂等键配不同 fingerprint 返回冲突；省略 `correlation_id` 时使用 `Idempotency-Key` 作为稳定 correlation；只在 workflow 接受后返回 202 run receipt，workflow 未配置时返回 503；执行器仍须在 resume 与每次 capability call 重验 ACL |
| AC-CHAT-004 | `GET /api/v1/worktrees/{worktree_id}/chat/targets` 必须要求当前 Bearer actor 的 `chat:submit` scope，并在同一 Tenant-scoped transaction 中锁定并授权路径 Worktree 与当前 Project membership；只返回当前 actor 仍有效成员所属的非归档 Worktree ID、Project ID 和名称，游标稳定且 limit 为 1–100；不得返回 checkout 路径或以目录结果授权写入；GLOBAL 提交仍须逐个重新解析 GroupContext，越权时整批不派发 |
| AC-CHAT-005 | 首次接受请求时 session/thread、user Transcript metadata 与受保护 payload、queued Run、幂等 receipt、dispatch event 与 Audit 必须同事务提交；同 actor/tenant/key 且 fingerprint 相同的 30 天内重放返回同 receipt 且不增加第二个 run/message/outbox；fingerprint 不同返回冲突且不写入；事务失败或 provider 未配置不得返回 202 |
| AC-CHAT-006 | Transcript 正文只允许经受信任 protector 加密后写入 W payload，T metadata 不含正文；Project Policy 保留期可配置且默认最多 90 天；到期清理可验证，crypto-erasure 或物理删除不能破坏不含正文的 append-only Audit；必须验证授权导出/删除、密钥轮换、RLS 与日志/outbox/checkpoint 无正文；当前 migration 虽已改为 ciphertext/wrapped-key schema，缺少真实密钥服务/清理 worker/导出删除路径时仍不得用于生产 |
| AC-PLG-001 | 当前 actor 可访问的 Engineering Run 中，服务端授权投影里的 active/compatible/authorized 插件作为 Run 同级 App 出现；撤权后移除入口并拒绝新 capability 调用，保留既有业务事实/引用；Worktree 只决定焦点和 CLI/Git 目标；本地预览开关不改变服务端状态 |
| AC-PLG-002 | 目标 `GET /api/v1/engineering-runs/{engineering_run_id}/apps?focus_worktree_id=...` 返回重新校验后的 active/compatible/authorized 最小 Run App 投影；要求 Bearer actor、当前 Project/Branch/Run membership 与必要的 Worktree grant，响应 no-store，不返回 secrets/grants/raw manifest；Registry ID 唯一稳定；UI 在加载/错误/身份变化时清除旧项且不 fallback。旧 `/api/v1/worktrees/{worktree_id}/group-apps` 仅为兼容路由，provider/schema/read 不可用时不得伪装成功 |
| AC-PLG-003 | Manifest、Registry revision、binding、grant 四类 Master 均有有效区间与 current-row 唯一约束；更新只允许关闭当前行并追加 successor，物理删除被数据库 trigger 拒绝；Registry Audit UPDATE/DELETE 被拒绝；五张表启用并 FORCE RLS。Migration 静态定义通过不代表已部署或真实 PostgreSQL RLS 已验收；manifest trust root/ingest、binding/grant lifecycle writes 与 capability gateway 仍须单独实现 |
| AC-GRP-DB-001 | Group PostgreSQL 生产部署必须分别配置 migration owner 与运行时 service role；运行时角色仅获得各 adapter 所需的 schema `USAGE` 与逐表最小 SQL 权限，不得为 superuser、BYPASSRLS 或表 owner；RLS policy 本身不授予 SQL 权限。必须使用目标环境实际登录身份验证权限、tenant/actor 隔离、拒绝越权读写及 append-only 约束。迁移不得假定未声明的全局应用角色；角色名、成员关系和 grants 由受控部署/bootstrap 清单显式提供 |
| AC-RUN-001 | 从 Task Card 发起的每次 CLI 执行都创建独立 `TaskExecutionRun`，启动幂等重放返回相同 Run；新的真实重试创建新 Run。Run 保存创建时的 Task/input/acceptance snapshot，并将 `run_id` 传给 Runtime；Worktree ID 是不阻止 Worktree 清理的历史快照 |
| AC-RUN-002 | Run 的执行器状态、Agent 声明、验证、人工接受/返工和集成结果独立呈现；只有明确的 verifier/human/integration 事件可以推进对应维度；缺失值保持 unknown，不能将 Agent 自报完成显示为验收通过 |
| AC-RUN-003 | Task Card 可按 Goal / Execution / Evidence / Feedback / Compare 浏览 Run 摘要与事件；每个 Evidence 只返回经过 ACL 校验的脱敏 metadata/digest/受控 locator，不返回 raw transcript、Secret、token 或完整 tool output |
| AC-RUN-004 | TaskExecutionRun 历史在 canonical Task Card 内读取，并与 EngineeringRun 导航/身份分离；目标 Run-scoped API 每次重验当前 actor、tenant、Project/Branch/EngineeringRun membership、WorkItem grant 和可选 Worktree 关联，使用有界稳定游标、no-store 与字段 allowlist。现有 `/api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/runs` 仅是 Worktree compatibility endpoint；详情限制 100 Event/100 Evidence，禁止返回任意 details、locator、原文或大输出；状态维度各自取最新非空事件，认证 API 不可用时 fail closed |
| AC-WTG-010 | Project Worktree Index 可进入 Project 级 Quality & Improvement 视图；所有指标显示公式、分子/分母、单位、时间窗、coverage 与版本，并能下钻至授权 Task/Run/Evidence；unknown 数据不作为零计入分母，结果按 task type/complexity 分层 |
| AC-WTG-011 | 固定 Benchmark 重放使用版本化 task/repo/environment/acceptance/scoring snapshots，并分离 tuning/holdout；候选策略不得改标准；proposal 经过隔离对比和授权采纳后可回滚，历史分数保持原口径 |
| AC-TRACE-001 | 一次从 Canvas 或底栏聊天发起的任务操作，可由同一 `correlation_id` 串起 Worktree、WorkItem、TaskCard、TaskExecutionRun、CLI Session、Canvas Element、Plugin 调用与 Audit |
| AC-ERUN-001 | Project → Branch → Engineering Run → Worktree 主导航保持身份与深链；点击 Run Worktree 后才加载所属 Run tabs，Worktree 只设 focus/CLI target；右侧 Task/Canvas/BI 数据按 Run 授权 | P0 |
| AC-ERUN-002 | Run Context/API 对每个实体域复验 membership 与 capability；跨域命令调用 owner API，同 owner DB 事务使用 Outbox，消费者经 Inbox 幂等去重；前端不能跨域表写入或直接使用 stored procedure | P0 |
| AC-ERUN-003 | 同一 Run 切换 Worktree 时 RunContext/context_version 保持 owner 与授权身份，workspace/checkout/binding version 在独立 focus/focus_version 中改变；撤销 Project、Branch 或 Run 任一 grant 后再次解析拒绝；列表不泄露路径，不自动选择首个 checkout | P0 |
| AC-INFRA-001 | 增加 Rust-native Infrastructure Manager 与 versioned backend/profile/binding 契约，支持用户自有或远端 Linux 环境，并为 K3s 提供受控 discover/provision/readiness/start/stop/drain/upgrade；默认按 Host/environment 共享按需基础设施、Project/Run 分配 namespace/权限/预算，不按每个 Worktree/Agent 复制 VM/控制面。Provider 不得以商业用途、席位或用量计划限制核心能力；版本准入按具体许可履约路径及完整 SPDX/SBOM 检查，活跃社区以近期发布/维护及公开渠道核验。Multipass、Podman machine、Incus、Lima 和 existing cluster 按 host capability 分平台验收；native Hook、租约/CAS、操作幂等和 BI 证据与现有体系一致。Namespace 不独自构成不可信 Agent sandbox，缺 execution capability 仍禁用；不得静默改变用户全局 WSL 配置或回收他人环境 | P0 |
| AC-ERUN-004 | 新 CLI TaskExecutionRun 的 EngineeringRun 身份由服务端反查当前 Worktree binding，复验 Project/Branch/Run writer grants、当前 revisions、checkout 分支及 Task 关联；在准入短事务中再次核验完整快照，持久化独立 EngineeringRun ID 与精确 tuple，并绑定 V2 Runtime fence。旧无归属记录保留只读，不能猜回填或重复启动；新 CLI NULL 身份写入、snapshot 错配/超限、跨 scope FK 与 binding/grant 变化必须拒绝。CLI attachment 签发也须复核当前三层授权与该 Session 对应的已存身份；status/cancel 保留授权后的安全清理语义。Task 本身的 Run ownership、生产 provisioner/OS sandbox 与执行结果闭环另行验收 | P0 |
| AC-ERUN-005 | 每张新 Task Card 必须写入完整 canonical Run owner tuple；Run 列表按完整 owner 身份分页，Worktree 兼容 API、Canvas Task Card 创建、CLI admission、Session attach 与 TaskExecutionRun list/detail 均校验当前 owner 和所选 Worktree 的 Run 一致；跨 Run 关联/重绑定必须拒绝。旧无 owner 记录不自动回填，须只读直到受审计的 SCD2 reconciliation；新 owner migration/outbox 在目标数据库与应用身份 RLS 验收前不算生产启用 | P0 |
| AC-ERUN-006 | Engineering Run 默认打开同级 Task Cards App；列表只消费带当前宿主 Bearer 会话的 canonical Run-scoped API，逐页最多 12 条、响应最多 2 MiB、当前仅驻留一页并限制最多浏览 100 页；服务端未授权/未开放时显示明确阻断或错误状态，不得回退 Worktree 旧列表、seed 或浏览器演示任务。卡内 CLI 在执行准入、Runtime sandbox、取消/恢复、独立验证与结果回写闭环验收前必须禁用 | P0 |
| AC-TASK-DATA-001 | 所有历史演示 Task Card (`wi-001`..`wi-030`，以及独立桌面端 `wi-001`..`wi-004`) 与其演示专属关联必须从产品运行时移除；Runtime seed/MSW task history 不提供这些任务，Tauri MockDb 不生成 WorkItem，Tauri/browser-dev 无本地 mock fallback，缺少 canonical Run provider 时 fail closed。validation/comment mock endpoint 不得为这些 ID 返回历史数据。升级仅过滤精确已知 ID 的浏览器持久化数据并保留其他用户数据。不得用前缀清理数据库记录或把旧 ID 映射到 Canvas；隔离测试 fixture 仅能使用 `test-*` ID，不得进入产品 seed、MSW 任务历史或业务投影。服务器端旧行不按 mock 假设删除，未归属行保持 unknown/不可执行，直至显式 reconciliation | P0 |
| AC-EVENT-001 | 当前领域事件基线为 PostgreSQL SoR + Transactional Outbox + NATS JetStream；Kafka/Fluvio 不在运行依赖；新增 broker 前需经 ADR 和同 workload 的保留/回放/资源/恢复基准 | P1 |

### 50.8A 多 Agent 并行、资源预算与 Rust 桌面性能

Pi Agent 作为设计参考，借鉴其小核心、可组合工具/扩展、明确生命周期事件、可分支持久历史及按需压缩上下文；渡口须自建 Rust 核心、并行调度、权限和插件运行边界，不得把 Pi/Node 运行时嵌入产品执行链。Agent 并行按 Project → Cloud Branch → Engineering Run → Worktree → Agent/Plugin 分层资源控制：内存、CPU 并发、子进程、文件描述符、磁盘/事件队列、模型请求并发和时间预算均须显式配置；未知预算不得按无限容量处理。

调度器按依赖 DAG 只派发已就绪任务，并在 Project/Worktree 间采用有界配额与公平调度；禁止无界 fan-out。队列必须有容量和背压，阻塞时延迟/拒绝新 Run 或降低并发；取消、deadline、插件撤权和 Worktree drain 必须传播到子 Agent/进程并完成回收。Agent lease、文件/资源 claim、Git retention lock 是不同信号；Git lock 不能代表 Agent 活跃、文件互斥或安全清理。Agent 间协作通过受授权、可重放的 coordinator/event 契约，不允许跨目标 L1 Agent 直接通信。事实事件须持久化且不可静默丢弃；可重建的高频进度投影可合并/节流，并携带稳定序号和 `correlation_id`。

Rust 桌面端以 Rust 为 UI 与执行控制的主要实现边界，尤其是 Worktree Index、Agent/Run 时间线、Canvas 大场景和卡内 CLI。界面只保留当前页/视口及有界缓存；Worktree/Run/Canvas 使用游标分页、虚拟列表/视口裁剪、增量投影、共享不可变快照与按需获取详情，终端输出和大证据写入有界 ring buffer 或磁盘 artifact，不复制进多个 UI store。不可见面板暂停轮询与重绘；CPU/IO 工作不在 UI 线程运行；异步队列、worker pool、Canvas 索引和资源缓存均须有容量/淘汰策略。插件采用隔离进程或受限 WASM 等运行时及版本化 capability manifest；撤权先拒绝新调用，再 drain/cancel 在途操作并回收资源；主进程不得加载任意 native 插件代码。

性能目标须以设备档位和实测 workload 建立，不臆造内存/延迟数值。基准记录设备、Worktree/Run/Canvas 数量、活跃 Agent 数、desktop 进程树 peak RSS、空闲/高峰 CPU、首屏与事件更新 p95 延迟、取消/drain 时间及测量覆盖率；预算阈值先标记 `TBD-MEASURE`，完成基线测量后才能作为 release gate。

Evaluator API v2 为每条 HookRule 提供 phase scope。旧的无 phase 规则继续只适用于 Worktree archive/cleanup；v1 policy 保持既有 canonical JSON 与 digest，并仅能在 archive/cleanup phase 使用。Run admission 规则只允许使用 actor authorization、lifecycle version 与 Runtime health typed facts；不支持的 phase/fact 组合必须在验证时拒绝并 fail closed。REST 已提供条件式 Run admission producer contract：锁外最多等待 2 秒取得 Runtime readiness/fence，等待期间不得持有 DB transaction/row lock；readiness 最多新鲜 5 秒，fence 必须至少留有 5 秒提交余量且 TTL 不超过 30 秒。随后在短事务内重授权、重读 Worktree/Task/lifecycle/有效策略并执行 Rust evaluator。Allow 事务必须原子写入不可变 HookSet snapshot、Run/start event、Hook ledger 与 Run `hook_evaluated` 镜像，两个事件共享 `tenant_id + event_id`；Deny 只写无 Task/Run FK 的 Hook ledger，不创建 Run。事务提交后，Runtime adapter 必须消费绑定完整 Task/Worktree/Runtime/profile/request fingerprint 的一次性 fence 并在 spawn 前再次校验。`TaskCliSessionProvisioner` 默认不声明 producer；当前仓库没有生产 adapter 装配，因此当前运行环境必须保持 publish/rollback 拒绝与 Builder 禁用，coverage 不得宣称 Run admission 已实际部署。无 producer 时 Project/Worktree policy publish/rollback 服务拒绝该 phase；Builder 依据服务端 capability 禁用该选项。
| 要求 ID | 要求 | 优先级 |
|---|---|---|
| PAR-001 | Run/Agent/Plugin 启动先通过分层 CPU/内存/进程/IO/时间预算 admission；实际值、估计值和未知值分开记录 | P0 |
| PAR-002 | scheduler 使用依赖就绪、Project/Worktree 配额与公平调度；有界队列支持背压、取消、deadline、进程回收与 drain；禁止无界 fan-out | P0 |
| PAR-003 | Agent 活跃 lease、文件 claim、Git retention lock 和 Git merge conflict 独立建模；冲突/观测过期显示 unknown 并阻止危险管理动作 | P0 |
| PAR-004 | 多 Agent 只能经 coordinator 交换受授权 command/event；Run 独立身份、事件序号与 `correlation_id`，GLOBAL 编排禁止 L1→L1 直接通信 | P0 |
| PERF-001 | Rust 桌面 UI 对 Worktree、Run、Canvas 和 CLI 输出使用分页/虚拟化/按需加载及有界缓存；不可见视图暂停订阅，事实事件不能因 UI 合并而删除 | P0 |
| PERF-002 | 性能基准记录设备档位、workload、desktop 进程树峰值内存、CPU、p95 呈现延迟与 coverage；目标先经测量定标，未定标值标记 `TBD-MEASURE` | P0 |
| PERF-003 | Rust UI 线程不得执行阻塞网络、磁盘或高开销图计算；后台 worker/queue、Canvas 索引与插件资源有明确上限、取消和淘汰策略 | P0 |
| PERF-004 | Plugin 按版本化 capability 与资源预算隔离；热插拔按“停止新调用 → drain/cancel → 回收 → 更新投影”执行；禁止主进程加载任意 native code | P0 |

| 验收 ID | 受入基准 |
|---|---|
| AC-PAR-001 | 所有 Run/Agent/Plugin 启动均经过分层预算 admission 与公平调度；并发上限、队列容量、拒绝/等待原因及 CPU/内存/进程实际值可观测；超限触发背压而非无界扩张 |
| AC-PAR-002 | 用户取消、deadline、撤权与 Worktree drain 可传递到所有子 Run/进程；完成或报告明确未能回收状态；持久化事实事件完整，不因进度合并丢失 |
| AC-PAR-003 | Agent lease / file claim / Git lock / merge conflict 分别显示来源与新鲜度；过期信号为 unknown；Git lock 不单独授权 archive/cleanup |
| AC-PERF-001 | 桌面基准使用版本化大列表/Canvas/Run/终端负载；只保留当前页/视口与有界缓存，确认列表虚拟化、Canvas 视口裁剪及不可见面板停更；设备内存/延迟阈值在测量后设定并记录版本 |
| AC-PERF-002 | Worktree/Run/Canvas/CLI 的网络、磁盘和图计算不阻塞 Rust UI 线程；队列与缓存达到上限时按背压/淘汰策略工作，UI 显示可解释的限流状态 |
| AC-PERF-003 | 插件更新先关闭新 capability 调用，在途调用按版本化 drain/cancel policy 收敛，资源释放后才切换 Registry projection；插件崩溃不能拖垮桌面主进程 |

### 50.8B Schedule Loop 与 Engineering Loop

渡口分别定义 **Schedule Loop**（何时/为何触发工作）和 **Engineering Loop**（一次 Run 如何逐轮推进工作）。二者共享授权、资源 admission、事件账本、cancel/deadline 和 drain 契约，但不得合并成单一“循环状态”。Schedule Loop definition 是带版本的配置事实；每次触发建立不可混淆的 `schedule_occurrence_id`，受租约、fencing token 与幂等键保护，只有 admission 成功后才创建独立 `TaskExecutionRun`。手动启动不伪装成 schedule occurrence。

Schedule Loop 必须复用现有 Automation/Workflow 架构边界：`domain-automation` 的 versioned Rule 是唯一 schedule definition/occurrence source；Phase 9F3 已有真实 recurrence parser、bounded materializer 与 PostgreSQL occurrence/lease adapter，但尚无生产 rule API/持久 worker 和 occurrence-to-Run admission，不能把本地 substrate 说成 production Schedule。Workflow/LangGraph 消费已接受的 occurrence 并编排 Run 内步骤；`star-scheduler` 只负责依赖 DAG 就绪，不负责计时；Canvas Workflow 的 Schedule Trigger 作为 Automation adapter，不另建竞争的 rule store、cron daemon 或 occurrence identity。实现和迁移状态必须区分事件自动化、源码/隔离 DB substrate 与已装配的生产 schedule worker。

Schedule Loop 必须定义时区/事件触发、目标 Task 与 scope、pause/disable、最大并发、重叠策略、misfire 策略（skip/coalesce/受限 catch-up）、重试预算、退避/jitter、deadline 和告警。重复调度以 occurrence ID 去重；worker lease 到期可被重新领取，但旧 worker 的 fencing token 失效后不能写入结果。暂停阻止新 occurrence；取消当前 schedule 可按 policy 继续或取消已接受 Run，但行为必须显式且可审计。DST、系统重启和队列过载不能导致无界补跑。

Engineering Loop 是单一 Run 内受限的 `Plan → Act → Observe → Verify/Evaluate → Continue / Request Review / Complete / Stop` 周期。每轮使用不可变 Task Contract / acceptance snapshot；每轮只追加结构化的决策、验证、证据引用与资源事件，不持久化 chain-of-thought 或大段日志。必须设 `max_iterations`、总时间/资源/模型请求预算、无进展与振荡检测、重试/backoff 和明确 stop reason。达到验收可结束；需人工评审时进入 review gate；预算耗尽、反复无进展、撤权或无法回收子进程时停止并显式标出失败/待处理，不得无限自我调用或降低验收标准。恢复 Run 前重新授权，并从最近已提交 loop boundary/checkpoint 接续。

Run detail 应能折叠查看每轮输入摘要、采取的工具/命令类别、外部可观测结果、自动验证、continue/stop 决策、迭代耗时/预算和证据链接；BI 可观察 loop completion、iteration-to-acceptance、no-progress/stall、重试与资源消耗，但不得优化“迭代数越多越好”或以模型调用数作为生产力。Workflow/LangGraph 是 Worktree 下既有同级 App，Schedule Loop 的管理入口留在 Workflow 或 Task Card，不新增 Worktree 树层级。

| 要求 ID | 要求 | 优先级 |
|---|---|---|
| LOOP-001 | Schedule Loop definition 必须版本化保存 trigger/timezone/目标 scope/并发与 overlap/misfire/retry/deadline/pause policy；修改不改写已创建的 occurrence/run | P0 |
| LOOP-002 | 每个触发创建稳定幂等 occurrence，使用带 fencing 的 worker lease 与 bounded dispatch；重启/过期 lease 可恢复，旧 worker 不得提交迟到结果 | P0 |
| LOOP-003 | Engineering Loop 仅在独立 TaskExecutionRun 内执行；每轮保留结构化 Plan/Act/Observe/Verify/Decision 边界，Task Contract/acceptance snapshot 不得在循环中被改写 | P0 |
| LOOP-004 | Engineering Loop 必须受迭代、时间、内存/CPU/process/provider 请求预算约束，并检测无进展/振荡；撤权/cancel/deadline/预算耗尽都要收敛到明确 stop reason 和 child drain 结果 | P0 |
| LOOP-005 | Loop 投影和 BI 由 occurrence/RunEvent/Evidence/Audit 派生；可重建进度可合并，durable occurrence/verification/review/stop facts 不得丢弃，loop 指标不得奖励迭代数或调用量本身 | P0 |

| 验收 ID | 受入基准 |
|---|---|
| AC-LOOP-001 | 相同 schedule occurrence 重放只创建一个 Run；schedule version、occurrence ID、target GroupContext 和 idempotency 可追溯；手动 Run 与 scheduled Run 的来源不同 |
| AC-LOOP-002 | 暂停/重启/DST/misfire/overlap/lease expiry/fencing/queue full 的场景遵循配置策略；catch-up 与并发有硬上限，取消行为可审计，过期 worker 不能写结果 |
| AC-LOOP-003 | 每次 Engineering Loop 只能在 acceptance snapshot 未变更时继续；完成须满足受信验证/人工 review；预算耗尽、stall、oscillation、撤权或取消均有明确终止原因和已回收/未回收 child 状态 |
| AC-LOOP-004 | Task Card Run Detail 可显示各 loop iteration 的结构化摘要、工具类别、可观测结果、验证、耗时/预算和 stop reason；不暴露 chain-of-thought、Secret 或未脱敏大日志 |
| AC-LOOP-005 | Resume 从 loop boundary/checkpoint 续做时重新校验当前 actor、GroupContext、Worktree、Plugin capability 和 Task Contract/version；重放副作用仍由幂等 Domain Command 收敛 |
| AC-LOOP-006 | BI 只按固定公式报告 scheduled success/misfire、loop acceptance、iteration-to-acceptance、stall/rework 与资源成本；unknown 有 coverage 标记且原始迭代数不是优化目标 |
| AC-LOOP-007 | Schedule rule Master/SCD2、append-only rule Audit/Occurrence/Event 和带 TTL 的 Work dispatch state 具备 tenant FORCE RLS；tenant/rule/version/UTC-slot 复合唯一键、target/Profile/HookSet snapshot、DST/timezone/policy version 与 monotonic lease generation 可被重复迁移及并发/replay 场景验证；只有 domain/schema/dispatch substrate 而未完成生产 rule API、Run writer/Auth/Outbox/BI 时不得声称 AC-LOOP-001/002 通过 |
| AC-LOOP-008 | 首版 Rule 的 `run_as_actor_id` 必须等于已授权创建者，所有后续版本和 occurrence snapshot 身份一致；数据库拒绝伪造初始主体、替换 successor run-as 或伪造 occurrence 身份；worker 每次触发读取实时 Project/Run 授权并记录拒绝原因，撤权、目录/权限服务不可用或身份不完整时 occurrence 不得产生 TaskExecutionRun |
| AC-LOOP-009 | Run admission 必须将通过实时 run-as 授权、target/profile/hookset/预算复验的 schedule occurrence 与唯一 TaskExecutionRun、Reservation、RunEvent 和 append-only Outbox 在一致性边界内关联；被接受的 dispatch 固定且不可替换 `admitted_run_id`，只有对应 occurrence 的 schedule-origin Run 可关联，重复 admission 幂等；撤权、授权源不可用、target 漂移、预算不足或关联不一致时不得创建 Run；仅有数据库关联约束而没有生产 admission writer/consumer 不得判为通过 |

实施对账（2026-10-02）：Phase 9F1 已在 `domain-agent::engineering_loop` 提供 Run-local bounded controller 代码切片，固定 Profile/Task Contract/acceptance/HookSet/Validation identity，限制 iteration、wall-clock、CPU/RSS、child process、provider calls、output/event buffer 与 per-Run tool concurrency，并输出 digest-only receipt。验证通过只进入 AwaitingReview，不自动修改 Task 状态。该切片尚未接 Run admission/Auth recheck、CLI/OS process、durable checkpoint/Outbox、Schedule occurrence、BI 或跨 Run 公平调度；当前 Profile v1 没有累计成本预算字段，retry/backoff 也未实现，因此 LOOP-003/004/005 与 AC-LOOP-001..006 仍未整体通过。

实施对账（2026-10-02，Phase 9F2）：domain-automation::schedule 加入版本化 Rule、target/Profile/HookSet snapshot、UTC-slot key、DST/overlap/misfire/retry/deadline policy 与 lease-fence DTO；新增 automation schema 五表 W/T/M 与 SCD2、append-only、FORCE RLS、fencing、terminal TTL guards。该阶段仅交付 domain/schema substrate。

实施对账（2026-10-03，Phase 9F3）：加入固定版本的 `cron 0.17.0` + `chrono-tz 0.10.4` recurrence parser、DST gap/fold 与 misfire/cursor bounded materializer，以及 PostgreSQL current-rule reader、occurrence idempotent persistence、`SKIP LOCKED` claim/reclaim、heartbeat、retry/exhaustion、deadline finalization、terminal transition 与 Work TTL purge。disabled Rule 在 materializer 和 adapter 两层 fail closed；lease-expired audit event 记录被回收的旧 attempt 与旧 fencing generation。候选槽扫描与 DST 转换探测各有 32,768 次硬上限；窗口过宽时调用方必须拆分后续页。PostgreSQL 18.6 disposable loopback cluster 中 migration 重复应用两次、五张表 FORCE RLS catalog check 通过；非 superuser runtime role 实跑 4 个 integration tests（RLS/idempotency、并发 claim/fencing、retry/TTL/reclaim、deadline/exhausted lease），最终 domain release tests 26/26 通过。此为本地 disposable DB 证据，不是目标库部署/grants。生产 rule API、clock/worker、occurrence→Run/reservation/RunEvent/Outbox 原子 admission、BI/Auth/target DB 仍开放；production Schedule capability 继续关闭，AC-LOOP-001/002 未整体通过。

实施对账（2026-10-04，Phase 9F4A）：新增 Run-scoped Schedule Rule list/get/create/revise API；每次请求复验 Project binding 与 Run grant，写事务重新验证当前 Run-owned Worktree、Task link、Profile、Provider/Skill/Grant catalog 和有效 HookSet，并排除 future-dated facts。CAS successor revision 使用同一 transaction timestamp 收口 SCD2 边界；过期幂等 key 先精确删除再执行至多 64 条其他 stale cleanup；Outbox 的复合 FK 固定其 Run 与 Rule revision 一致。显式启用 `jsonwebtoken` RustCrypto，修复原依赖只含 PEM 解析、没有 RS256 crypto provider 的运行时配置缺口。Focused Rust check、6 个 Schedule Rule 模块测试和 PostgreSQL 18.6 disposable 重复迁移/RLS/约束场景已通过；production `build_group_router` 的四种未认证请求均返回 401，四种签名 JWT 缺少所需 scope 时均返回 403，携有效写 scope 的超限 body 返回 413，拒绝响应含 `private, no-store` / `Vary: Authorization`。scope/body-bound 请求用 lazy pool，不进入 PostgreSQL；仍未覆盖成功授权的 role/Project/Run ACL、target binding、page bounds、并发 CAS/replay 或目标 PostgreSQL/RLS/grants。worker、occurrence producer、TaskExecutionRun/reservation/RunEvent admission 与 BI consumer 仍开放，不得声称 Schedule Loop 已生产可用。逐项合同见 [SRS](requirements/SRS-AUTOMATION-SCHEDULE-API-001.md)、[基本设计](design/BD-AUTOMATION-SCHEDULE-API-001.md)、[详细设计](design/DD-AUTOMATION-SCHEDULE-API-001.md) 和阶段报告。

实施对账（2026-10-04，Phase 9F4B）：Rule 创建者作为 run-as principal 写入每个版本化 Schedule Rule revision；API 只从已授权创建者取得该主体，successor 复用第一版本身份，DB trigger 禁止伪造初始 creator、替换后续身份，并校验 occurrence 与精确 pinned Rule revision 身份一致；domain 与 PostgreSQL occurrence snapshot 均持久携带主体。Disposable PostgreSQL 18.6 fixture 用历史两版 Rule（首版创建者与 successor 编辑者不同）和旧 occurrence 验证历史回填；迁移在一个事务持有 DDL 表锁时由 schema owner 暂时越过自身 FORCE RLS、停用仅阻断回填的旧 guard，随后恢复所有 guard/FORCE RLS。验证通过 Domain 26/26、API 身份拒绝 1/1、五个 PostgreSQL adapter scenarios；目标 DB migration principal/grants 未验收。后续 worker 每次无人值守触发必须重新读取该主体当前 Project binding 与 Engineering Run grant；撤权或无法确认 ACL 时不得创建 Run。该 phase 尚未实现 worker/Run admission，故 AC-LOOP-008 尚未整体通过；Launch Profile 权威 resolver、目标数据库/RLS/grants、最终 quota/Auth/target recheck、Reservation/TaskExecutionRun/RunEvent、Outbox consumer 与 BI 仍开放。

实施对账（2026-10-05，Phase 9F4C-A）：新增 `occurrence_dispatch.admitted_run_id` 复合 tenant FK 和数据库状态迁移守卫，只允许 dispatch 从 `leased` 进入 `admitted` 时绑定 Run，且已绑定 Run 不可替换；DB trigger 不知道 worker 提交的当前 lease owner/generation，未来 writer 必须在条件更新中校验 owner、generation、lease expiry 与 occurrence deadline。Run 必须来自同一 tenant、Project、Run scope、原 occurrence、固定 run-as 和 schedule-origin agent channel。新增 tenant FORCE RLS、append-only `automation.schedule_run_outbox`，以复合 FK 固定 occurrence、Rule revision 与 TaskExecutionRun，insert trigger 校验主体/来源。Disposable PostgreSQL 18.6 runner 双次应用完整 migration chain，验证 8 张 Schedule 表 FORCE RLS、admission 正向关联、run-as 错配、已绑定 Run 替换与 Outbox UPDATE 拒绝，以及非 superuser tenant 隔离；runner 未单独测试其它 source/channel mismatch 或 DELETE/TRUNCATE 拒绝。既有五个 adapter 场景通过。此切片只建立数据库持久化约束；当前尚无生产 admission writer、触发时 Project/Run ACL 与 target/profile/quota 最终复验、Reservation/RunEvent 同事务写入、Outbox consumer 或 BI/Benchmark 投影。因此 AC-LOOP-009 未通过，Schedule production capability 继续关闭。

实施对账（2026-10-05，Phase 9F4C-B enable-time authorization）：API 写入身份与固定 run-as 身份分离；创建启用规则时在同一写事务复验创建者的当前 Project/Run writer grant 与 Run active 状态，启用 successor 时复验锁定 Rule 的原始 `run_as_actor_id`；停用只需仍有权限的规则管理员，可在创建者撤权后停止 Rule。当前 Branch binding 必须仍有效。定向 `star-api-rest --all-targets` check、rustfmt 和纯角色策略 helper 单测 `task_execution_rules_do_not_grant_agent_role_schedule_authority`（1/1）通过；该单测不证明 SQL/路由授权；新授权 SQL 尚无包含 canonical Project/Branch/Run ACL 表的隔离 PostgreSQL fixture。该 gate 只保护 API 启用操作，不能替代 AC-LOOP-008 对每次触发、重试、恢复的复验；worker/admission writer、能力/目标/配额复验、原子 Run admission 与 BI 未完成，AC-LOOP-008/009 仍未通过。

#### Phase 9F4A Rule API acceptance gate

`SCHED-API-001..010` 的详细要求及验收证据由 `SRS-AUTOMATION-SCHEDULE-API-001` 维护。阶段关闭前须通过目标 Run/Project 授权正负例、Worktree/Task/Profile/HookSet 解绑与漂移拒绝、并发 CAS、相同/冲突幂等键、Audit/Outbox rollback、FORCE RLS 与 runtime grants 验证；代码存在或 source-text 检查不视为通过。API 子集通过也不等于 Schedule occurrence→Run 的 AC-LOOP-001/002 生产闭环通过。

#### Phase 9F4B immutable run-as identity acceptance gate

创建者由创建请求中已通过当前 Project/Run 写授权的 actor 确定；请求 body 不接受 `run_as_actor_id`。迁移按每个 Rule 最早 revision 的 `changed_by` 回填历史行，首版必须满足 run-as 等于 creator，之后数据库检查 successor 保持同一主体；每个 occurrence 将主体复制进专用快照列，并由数据库确认其与精确 pinned Rule revision 一致。occurrence worker 必须在每次新触发、重试与恢复前，以该主体复验有效 Project membership、Run grant 与当前执行 capability；撤权、身份目录不可用或 ACL 事实不能确认时，occurrence 记为拒绝且不得产生 TaskExecutionRun。当前仅实现身份存储和不变式，未启用 worker；Schedule execution 必须 fail closed。

#### Phase 9F4C-A Schedule Run admission persistence acceptance gate

数据库仅允许 `leased → admitted` 转换时绑定不可变的 `admitted_run_id`，并检查 dispatch 状态及 fencing/attempt 序列不变量；trigger 不验证 worker 提交的当前 lease owner/generation。应用层必须通过条件更新校验当前 owner、generation、lease expiry 与 occurrence deadline。Run、occurrence、Rule revision、tenant/Project/Engineering Run、schedule 来源、agent channel 与 run-as 必须一致。schedule Run Outbox 使用 append-only、tenant FORCE RLS 与复合 FK，且每个 occurrence/event type 唯一。此迁移与 disposable PostgreSQL 负例验证不等同于应用层原子 writer：AC-LOOP-009 关闭前，还需生产 worker 在同一 admission transaction 完成当前授权与 target/profile/HookSet/quota 复验、Reservation/TaskExecutionRun/RunEvent/Outbox 写入，并验收失败回滚、并发重放、目标数据库 grants、consumer 幂等与 BI coverage。

#### Phase 9F4C-B Rule enable-time run-as authorization acceptance gate

创建启用 Rule 或将 disabled Rule 重新启用时，API 必须在写事务内锁定并复验固定 run-as actor 的 current Project/Run writer grants 与 active Run；current Branch grant 必须存在。修改规则的编辑者仍单独通过当前 Project/Run 管理授权。停用 Rule 不依赖 run-as actor 仍有权限，使管理员在撤权后可以安全停止后续计划。该 API gate 不满足无人值守的每次触发要求；新触发/retry/resume 仍须由同事务 admission writer 重验权限与 execution capability。当前 source compile 已通过，但该 ACL SQL 尚未通过带 canonical directory ACL fixtures 的数据库运行验证。

### 50.8C 可扩展 Agent Execution Profile：Agent、Memory、Skill、Context、Validation

Agent 执行能力按稳定契约组合，不把某个 CLI、模型、记忆实现、Skill 格式、上下文算法或验证器写死进 Task/Worktree 身份模型。`AgentExecutionProfile` 是版本化 Master，引用具名且版本固定的 `AgentProvider`、`MemoryProvider`、`SkillRegistry`、`ContextAssembler`、`ValidationProvider`、`LoopPolicy` 与资源预算；Provider 可由内建 Rust 实现或通过隔离 Plugin capability 提供。新增实现应只注册兼容 provider/version/manifest，不改变 `work_item_id`、`run_id`、Worktree 关系或已有历史 Run 语义。未支持的 provider/capability 必须显式标为 unavailable，不得用 mock 或空成功冒充。

Phase 9E-1 Rust profile verifier 使用 ≤65,536 字节 serialized document、版本化 schema、固定字段顺序 JSON SHA-256、排序去重引用与明确上限；verified wrapper 只暴露不可变借用。Phase 9E-2 resolver 将 snapshot 与当前 bounded Provider/Skill catalog、grant、effective HookSet 和 Worktree lifecycle 逐项精确匹配，漂移 fail closed。Phase 9E-3 建立 Profile Master/SCD2 + append-only Audit，scope/schema/digest 一致且两表 FORCE RLS。Phase 9E-4A 增加 Run Profile snapshot all-or-none、Run/document tenant/project/可选 Worktree scope 与 digest CHECK，不建立 Profile 外键。Run、Profile、guard 三份 migration 在隔离 PostgreSQL 数据库执行，guard migration 重复应用通过；无 Profile 的旧 Run 与 Project/Worktree 两类完整快照插入成功，部分 tuple、tenant/project/Worktree scope 与 digest 不一致的 5 类负例均被拒绝，Profile FK 数为 0；临时数据库已清理。Phase 9E-4B2 已提供 Project/Worktree Profile 生命周期 publish/disable/reenable/rollback API 代码切片；当前 Provider/Skill/Grant catalog adapter、生产 Run writer、目标 DB 部署与资源 admission 仍开放，恢复或创建 Run 仍需外层完成 actor ACL/GroupContext 授权。

Phase 9E-4B3 将当前授权视图中的 verified effective Hook policy 映射为 Profile/Run admission 使用的 `HookSetSnapshot`：存在 Worktree restrictive overlay 时采用其 policy-set ID，否则采用 Project baseline ID；version 与 lowercase digest 取自经 Rust verify 的 effective snapshot。该切片只提供身份桥接，不解析 Provider/Skill/Grant catalogs、不创建 Run，也不预约资源；缺失或不一致的策略继续 fail closed。

Phase 9E-4B4 固定 Task Card CLI admission 的两类 profile identity：`approved_launch_profile_id` 只标识 Local Runtime 可启动的 executable/argv/environment policy，`execution_profile_id` 标识 Agent、Memory、Skill、Context、Validation、Loop、HookSet 与资源策略，不得相互代用。新 Run 必须绑定经过当前 Provider/Skill/Grant resolver 验证的 Profile ID/version/digest/document snapshot；request fingerprint、Run admission fence 和 Runtime spawn receipt 都分别绑定 Agent Execution Profile 与 Approved Launch Profile identity。Profile snapshot、Task/acceptance、effective HookSet、RunEvent/Hook ledger 与资源 reservation 必须在最终短事务按既定原子边界提交。旧 Run 的幂等重放保留旧指纹语义；新请求指纹引入显式版本，避免新增字段破坏历史 idempotency key。B4 阶段审计时 CLI API 尚无 `execution_profile_id` 输入；C1 后已补入独立选择与请求字段，C2 已实现 current catalog migration、bounded SQL reader、Arc snapshot 与最终 revision/HookSet recheck；C3 已增加条件式 Run/Profile/resource reservation writer 与跨 Worktree quota admission；C4 已交付双 Profile spawn-fence contract 与 Run identity projection；C5 已增加 shared DTO、signature v2 及 Local Runtime nonce/fence 原子 consume foundation。但 catalog publisher、生产数据装载、真实 Runtime lifecycle/production caller、目标 DB/Auth/RLS 与完整 BI 仍未完成，因此新 Run 继续 fail closed。Profile/catalog 预取不得在持有 DB row lock 时执行；锁内重新授权、锁定并重读 DB-owned Profile/HookSet 与版本事实，外部依赖必须通过短时有效且可校验的版本 fence 绑定。
Phase 9E-4C1 已增加与 Approved Launch Profile 分离的 AgentExecutionProfile 选择输入和 Task Card 选择器。选择器只读当前 Worktree 有界 Profile metadata，要求显式选择，不提供 seed/default 回退；列表超过单页上限时保持启动禁用。带 Profile ID 的新请求使用含版本标记的幂等指纹；缺少该字段的旧 Run 重放仍使用原序列化指纹。REST 新 Run 路径要求 provisioner 显式声明 Profile-bound admission 能力，该能力默认关闭；C1 只交付身份/选择 seam，不代表新的 Profile-bound Run 已可用。C2 已增加权威目录 schema、bounded read/recheck 与 Arc snapshot 代码切片，但 publisher、生产装载和生产 Run writer 尚未完成；C3 已交付条件式 Run/resource writer；C4 已增加双 Profile spawn-fence contract、Run identity columns 与 Run Detail 投影；C5 已增加本地 Runtime fence-consume foundation，但未接 production caller，因此仍不得创建或启动 Profile-bound Run。

Phase 9E-4C4 的一次性 `TaskRunSpawnFence` 绑定 tenant/actor/Project/repository/Worktree/Task/Runtime/lifecycle、原请求 fingerprint、当前 catalog revisions、effective HookSet、bounded ResourceBudget，以及分别解析得到的 Approved Launch Profile 与 AgentExecutionProfile ID/version/digest。版本化、domain-separated `spawn_fence_binding_digest` 覆盖完整绑定；Run 只保存 digest 和两个 Profile 身份，不保存可复用的 opaque fence ID。Run Detail 可追溯实际声明的两种 Profile。Runtime 必须在授予进程能力前原子消费一次 fence，重验 actor ACL、scope/lifecycle、两份 Profile 当前版本、catalog/HookSet revisions 与资源预算，并激活或拒绝 reservation；缺字段、错绑、过期、重放或消费失败均不得 spawn。原 `request_fingerprint` 继续承载客户端请求幂等身份，server-resolved versions 纳入独立 fence binding digest，避免不同 revision 导致相同请求幂等键产生多次执行。Local Runtime 现存 `domain-local-runtime::task_execution` 仅校验签名 grant、scope、Approved Launch Profile ID/字段、canonical checkout、有效期和一次性 nonce；签名 context 不含 Launch Profile version/digest、AgentExecutionProfile、catalog/HookSet/ResourceBudget 或 C4 fence，helper 也未接入生产 provisioner/OS spawn，不能视为 C4 Runtime consumer。

Phase 9E-4C2 的每次执行目录读取只包含已验证 Profile 引用的 Provider（最多 5 个）、Skill（最多 128 个）与当前 GrantSet，不复制 tenant-wide Provider/Skill catalog。Rust `CurrentExecutionCatalogSnapshot` 用不可变 Arc-backed slices 共享并行 admission 所需目录，估算快照载荷上限 1 MiB。Fence 绑定 Profile ID/version/digest、tenant/Project/Worktree scope、Provider/Skill revision、GrantSet ID/version 与观察/过期时点；TTL 最长 5 秒，最终 admission transaction 至少保留 1 秒并重读来源 revision。漂移、越界、失效或来源不可用都必须回滚且不创建 Run。读取 Skill capability labels 前先以数据库聚合预估 heap，超过 384 KiB 即拒绝，避免超限 labels 先进入 Rust heap。

Phase 9E-4C2 已增加 `db/migrations/2026-10-01-multica-agent-execution-catalog.sql`：Provider、Skill、GrantSet 是 Project-scoped Master/SCD2，能力关联规范化存储，Provider/Skill revision 为 M 投影，catalog audit 为 tenant-scoped append-only Transaction；迁移包含 FORCE RLS、parent/child same-transaction guard、deferred capability-count guard、SCD2 guard、revision bump 与脱敏审计。`star-api-rest::execution_catalogs` 只查询 Profile 引用 ID；preflight/final admission 使用短 REPEATABLE READ transaction，锁定 Profile、revision projection 和当前 GrantSet，并复核 current snapshot resolver、revision fence 与 effective HookSet。Arc snapshot 随 readiness command 传递，不逐条复制 catalog entries。

当前仍是 fail-closed 接入切片：catalog publisher/source mutation API、生产 Provider/Skill/Grant 数据装载、目标数据库/RLS grants、真实 Auth/Project ACL 与生产 Runtime reservation lifecycle 未完成；`supports_current_execution_catalogs()` 和 Profile-bound Run capability 保持默认关闭。C3 migration/Rust writer 在隔离环境增加了 Run Profile/ResourceBudget/Loop snapshot 与 Project-wide reservation 原子提交，但不代表目标环境部署或新 Run 已开放；待接 Runtime activation/release、过期 epoch/reservation maintenance、Outbox/Run outcome BI 和 C4 双身份 spawn fence 后才重新评估 capability gate。

每个 Run 创建时保存不可变的 `execution_profile_snapshot`：各 provider ID/API version/实现版本、Skill ID/version/content digest/capability grant、Memory policy 与引用摘要、Context assembler version/budget/source digest、Validation suite/version/命令标识与 toolchain digest、Engineering Loop policy、Schedule occurrence（若有）和资源预算。快照只保存复现与审计所需引用、版本、脱敏摘要和 digest，不保存 Secret、未脱敏提示正文、原始大日志或模型隐式推理。provider 更新不得回写历史快照；恢复 Run 时复核当前授权并明确记录使用原版本还是兼容的新版本。

MemoryProvider 必须实施 tenant/project/worktree/task scope、读写 ACL、来源/时间/置信度或审核状态、TTL/保留与删除策略；跨租户或未获批的 scope 不得被 ContextAssembler 读取。SkillRegistry 发布不可变 manifest，声明版本、内容 hash、输入输出契约、所需 capability、资源需求、兼容 API 与撤销状态；Run 固定本次实际采用的 Skill 集合，skill 更新/撤销不改变旧记录。ContextAssembler 按 Task Contract、当前 Worktree 授权可见的仓库资料、获批 memory 与 skill 说明构建有预算、可归因的 Context Packet；必须保留 source provenance 与 compaction 边界，超预算按策略压缩可恢复材料，不能丢掉 acceptance criteria、权限约束或作用域信息，且不能把 compaction 摘要冒充原始证据。

ValidationProvider 与 AgentProvider 解耦：验证 profile 独立定义固定命令/规则版本、环境与输入 digest、覆盖的 acceptance criterion、结果、耗时/资源和 Evidence 引用。Agent 的“完成声明”不能成为验证通过；未运行、不可复现或缺证据必须记为 unknown/blocked。Benchmark 固定 profile 与验证标准，改进提案可以比较 Agent/Model/Skill/Memory/Context/Loop/Validation provider，但不得自行修改验收或评分标准；BI 按 profile/version/任务类型/复杂度/数据覆盖率分析有效交付、人工投入、返工、质量、成本与资源，不能把 Agent 数、调用次数或迭代次数单独当产出。

第一期可通过 `AgentCliAdapter` 调用已有 CLI；适配器由 Rust 执行控制面托管，采用 typed request/response、直接 executable+argv（不拼接 shell）、允许列表环境变量、已授权的 canonical Worktree cwd、文件/网络/tool capability 显式授权、stdin/stdout/stderr 与事件队列上限、deadline/resource budget、取消 token 和子进程树回收。CLI 不能授予权限、扩大 scope、改变 Task Contract、选择未批准的 Validation 标准或直接提交“已验收”结论；adapter/provider 版本、退出状态和可审计 Evidence 必须进入 Run。该 Phase 1 接入只是一种可替换 provider，不把 Node/Pi runtime 作为产品核心，也不能妨碍未来 Rust-native Agent/Memory/Skill/Context/Validation 实现。

每个项目可选择性提供版本化 `ProjectEngineeringManifest`（脚手架/工程适配包）：任务模板、仓库内 Agent 指引、构建/测试/格式/静态检查命令 ID、验收夹具、环境要求、artifact 映射与 redaction 规则。manifest 与 repository commit/digest 绑定，命令仅引用服务端批准的 executable/profile，不执行仓库或 Agent 自带的任意脚本作为特权命令。第一期可适配已有项目 CLI、AGENTS/任务说明和验证入口；以后可生成/安装统一工程包，但不要求所有项目先重构。

| 要求 ID | 要求 | 优先级 |
|---|---|---|
| AEC-001 | Agent Execution Profile 及 Agent/Memory/Skill/ContextAssembler/Validation/LoopPolicy Provider 具有稳定、版本化 API/capability contract；每个 Run 固定实际 provider/version/hash/profile 与授权快照，升级不改历史 | P0 |
| AEC-002 | MemoryProvider 强制 scope/ACL/provenance/TTL/保留边界，ContextAssembler 不得读取越权或跨 tenant/project/worktree 的记忆 | P0 |
| AEC-003 | Skill manifest 固定 ID/version/hash/capability/resource/compatibility；授权和撤销在执行时复验，历史 Run 固定本次版本 | P0 |
| AEC-004 | ContextAssembler 提供预算、source provenance、压缩边界和可恢复引用；Task Contract、acceptance、permission 与 scope 不得被静默截断 | P0 |
| AEC-005 | ValidationProvider 独立于 Agent 完成声明，保存规则/toolchain/input digest、criterion coverage、结果和 Evidence；缺失值保留 unknown | P0 |
| AEC-006 | 第一阶段可通过 Rust-owned CLI adapter 接入现有 CLI；必须 direct argv、allowlisted env、canonical cwd、显式 capability、bounded I/O、deadline/cancel 与进程回收，不接受 CLI 自授权限或自判验收 | P0 |
| AEC-007 | ProjectEngineeringManifest 可按 repository commit/version 增加任务约定、验证入口、环境和证据映射，新增项目适配不改变 Task/Run 主身份模型 | P1 |
| AEC-008 | BI/Benchmark/Improvement 按 Execution Profile/Provider/Loop/Validation 版本切片并固定评分标准、coverage 与复现条件；改进可回滚且不得自改验收标准 | P0 |
| AEC-009 | Run admission 仅接受经 schema、scope、canonical SHA-256 与 bounded-value 校验的不可变 Profile snapshot；Agent/Memory/ContextAssembler/Validation/LoopPolicy/Skill capability 不得超出当前 grant，当前 Provider/Skill/HookSet/Worktree 状态必须与 snapshot 相符且不得静默回退 | P0 |
| AEC-010 | AgentExecutionProfile 持久化为 Project/Worktree scoped Master/SCD2；document、scope/schema/digest 一致；revision 单调，active/disabled 仅以 successor 表达，禁止历史覆写/删除；Audit append-only + FORCE RLS；Run Profile ID/version/digest/snapshot 全空或全有，snapshot scope/digest 与 Run envelope 一致，不依赖当前 Master 存活 | P0 |
| AEC-011 | Worktree current Profile 只读 API 每次验证 Bearer actor、`worktree:read`、tenant RLS 与 Project/Worktree binding；列表有界分页且只返回元数据，详情经 Rust verifier 校验 scope/schema/digest，响应 `no-store`；该读取不等价于 Run admission、Profile publish 或当前 Provider/Skill/Grant 解析 | P0 |
| AEC-012 | Profile 生命周期 API 只接受有界 typed document 与授权后的 Project/Worktree scope；expected-version CAS 下发布 successor、停用、恢复或回滚，Profile revision 与对应 append-only Audit 必须同事务提交；Rust 重验 schema/scope/digest，历史版本不可改写，响应 `no-store` 且不回传完整 document | P0 |
| AEC-013 | Task Card CLI Run 分别绑定 Approved Launch Profile 与 Agent Execution Profile；请求幂等 fingerprint 固定两个用户选择的 ID，server-resolved 的两份 ID/version/digest、scope、catalog revisions、HookSet 与 ResourceBudget 纳入版本化的一次性 Runtime fence binding digest 和 Run snapshot；缺少当前 catalogs、resolver、snapshot writer 或 fence consumer 时 fail closed，历史 Run replay 保持版本化指纹兼容 | P0 |
| AEC-014 | Task Card 只呈现当前授权范围内的 bounded Profile metadata，必须显式选择 AgentExecutionProfile；超出单页上限、列表无效或读取失败时禁用新 Run，不得回退到 seed/default；新增 Profile identity 使用版本化指纹且保留旧 Run replay 的原指纹 | P0 |
| AEC-015 | 当前 Provider/Skill/Grant 读取必须来自权威 store；只投影所选 Profile 引用的最多 5 个 Provider、128 个 Skill 与当前 GrantSet，以 ≤1 MiB 的 Arc-backed bounded snapshot 和 ≤5 秒 revision fence 绑定精确 scope/Profile；最终短事务至少保留 1 秒 fence 并重读来源 revision，漂移、过期或未装配 adapter 时 fail closed | P0 |
| AEC-018 | Runtime spawn fence 必须是有界、短 TTL、opaque 且一次消费；binding digest 覆盖 tenant/actor/Project/repository/Worktree/Task/Runtime/lifecycle、请求 fingerprint、两份 Profile 的 ID/version/digest、catalog revisions、effective HookSet 与 ResourceBudget。新 Run 保存 launch Profile identity 与 binding digest，Run Detail 返回脱敏身份；Runtime 在 process create 前复验当前授权/版本/预算并 consume once；缺失、错绑、漂移、超时、重复消费或 adapter 未安装均不建可运行 Run、不 spawn，capability 保持关闭 | P0 |

| 验收 ID | 受入基准 |
|---|---|
| AC-AEC-001 | 两种 Agent/Memory/Skill/ContextAssembler/Validation/LoopPolicy provider 能以不同版本挂入同一 Task/Worktree 契约；历史 Run 仍显示原版本与 digest，未知 capability 明确 unavailable |
| AC-AEC-002 | 尝试跨 tenant/project/worktree/task 读取 memory/skill/context 均被拒绝并审计；合法来源可追溯到 source ID/version/digest，TTL/撤销后不能新读取 |
| AC-AEC-003 | 超出 Context budget 时保留 Task Contract/验收/scope/permission，压缩来源可审计且 raw evidence 不被改写或伪造 |
| AC-AEC-004 | Agent 报告完成但 Validator 未运行/失败时 Run 仍分别呈现 declared/verified/accepted/integrated 状态；每项通过判定可下钻到 Evidence |
| AC-AEC-005 | CLI adapter 验收拒绝 shell 插值、未批准 executable/env/cwd/capability 和越界 I/O；超时/取消会结束子进程树或明确记录未回收，重放保留相同 Run/幂等语义 |
| AC-AEC-006 | ProjectEngineeringManifest 跟 repository commit/version 固定；换项目 manifest 可换验证命令与夹具而不改 WorkItem/Run 身份，未配置时按明确的项目 capability 缺口处理 |
| AC-AEC-007 | BI/Benchmark 对两个 profile 做同标准对比，能展示任务分层、人工介入、返工、验证结果、实际/估算成本和 coverage；proposal 经隔离验证、批准采纳与回滚，评分历史不变 |
| AC-AEC-008 | Profile 解码拒绝未知 schema/字段、非 canonical 列表、digest 篡改、越 scope、越 grant、Memory 缺失或超限、ContextAssembler/LoopPolicy capability 越权、Context 丢失关键约束及 Loop/资源预算越界；Project profile 只能在同 Project Worktree 使用，Worktree profile 必须精确匹配；每项负向边界拒绝，历史 digest 不因后续 profile 更新改变 |
| AC-AEC-009 | Provider/Skill 缺失、撤销、版本或 digest 不匹配、Grant 变更/过期、effective HookSet 改变、Worktree 进入 draining/archive 均拒绝 admission；目录超限、乱序、重复也拒绝；resolver 不复制 Profile snapshot，缺失版本不得选择兼容项替代 |
| AC-AEC-010 | Profile Master migration 重复应用、连续 revision/SCD2、append-only Audit 和 tenant RLS 正确；Run guard migration 可重复应用；兼容无 Profile 旧 Run，接受完整 Project/Worktree 快照，拒绝部分 tuple 与 tenant/project/Worktree scope 或 digest mismatch；Run 不设 Profile FK，历史 snapshot 在 successor 后仍可独立读取 |
| AC-AEC-011 | Profile list 默认 20、拒绝 0 或大于 50 的 limit、非法 UUID cursor 且不返回 document；detail 仅允许当前 Project profile 或当前 Worktree profile，拒绝 scope/schema/digest 不一致；两类 API 均带 `Cache-Control: no-store` |
| AC-AEC-012 | Profile publish/disable/reenable/rollback 都要求 `execution-profile:publish` 与当前 Project admin membership；并发旧版本写入冲突；创建与每次状态/内容变化都递增 revision 并在同事务写匹配 Audit；rollback 仅能引用同一 Profile 的历史版本并产生新的 active successor；越 scope、非法 typed document/digest、未知/当前 rollback target 均拒绝；请求 ≤67,584 bytes、响应无缓存且不回传全文 |
| AC-AEC-013 | Run admission 必须从同一已授权的当前策略视图冻结 effective HookSet ID/version/digest；Worktree overlay 存在时使用 overlay identity，否则使用 Project baseline identity；baseline/overlay 继承不一致、版本或 digest 校验失败时拒绝 admission | P0 |
| AC-AEC-014 | CLI 新 Run 分别提交 Approved Launch Profile 与 Agent Execution Profile identity；版本化 request fingerprint 固定两个用户选择的 ID，Run snapshot 与一次性 Runtime fence 固定服务端当前 ID/version/digest；缺少 Profile/current catalog/resolver/snapshot writer/fence consumer 时无新 Run、无 spawn；旧 Run replay 在新增指纹版本后仍返回原 Run，且不得把两个 profile ID 混用 |
| AC-AEC-015 | Task Card 的 Profile picker 只使用当前有界 metadata API，要求显式选择且不得 seed/default 回退；无效/超页响应禁用启动；新 Profile 请求指纹含版本标记与所选 ID，缺少 Profile 的旧请求保持原 fingerprint/replay；生产 provisioner 未显式启用 Profile-bound capability 时无新 Run、无 spawn |
| AC-AEC-016 | 目录快照只含所选 Profile 引用且满足 Provider ≤5、Skill ≤128、估算载荷 ≤1 MiB；Skill capability labels 载入 Rust heap 前执行 ≤384 KiB 数据库聚合预算；Fence 精确绑定 tenant/Project/Worktree、Profile ID/version/digest、Provider/Skill revision 与 GrantSet ID/version，TTL ≤5 秒；最终事务剩余 <1 秒、revision drift、Scope mismatch、失效或权威 adapter 缺失时拒绝 admission 且无 Run/spawn |
| AC-AEC-017 | C3 最终 REPEATABLE READ admission 必须先原子写入 Project allocation epoch，再读取配额和跨该 Project 全部 Worktree 的 pending/active reservation；并发写冲突必须 fail closed，不能以旧快照超额预留。每 Run 的 RSS/CPU/runtime/process/tool/provider/output/event-buffer maxima 与 Project 聚合的 active-run/RSS/process/tool/provider/output/event-buffer ceilings 均通过后，Run 的 Profile/ResourceBudget/Loop snapshot、pending reservation、reservation ledger 与共享 `event_id` 的 RunEvent 在同一事务提交；任何缺配额、过期 fence、revision/scope drift、容量不足或写入失败均不建 Run、不 spawn。相同 idempotency replay 不得重复 reservation；Runtime activate/release 与实时 BI 未接通前，producer capability 继续默认关闭 |
| AC-AEC-018 | C4 的 typed one-time Runtime fence 必须将用户请求 fingerprint 与 tenant/actor/Project/repository/Worktree/Task/Runtime/lifecycle、Approved Launch Profile 与 AgentExecutionProfile 的当前 ID/version/digest、catalog revisions、effective HookSet 和 ResourceBudget 绑定进 domain-separated digest；Run 保存两份 Profile identity 与绑定 digest，不持久化 opaque fence ID。Runtime 在 process create 前重验当前 ACL/scope/version/catalog/HookSet/budget 并原子消费一次，随后同步 reservation lifecycle；缺失/错绑/过期/重放或消费、reservation transition 失败均 fail closed。Run list/detail 投影两类 Profile identity；production consumer/部署未完成前 capability 保持关闭 |
| AC-AEC-019 | C5 Runtime consume foundation 必须在共享 `star-dto::task_run` 提供拒绝未知字段、定长 digest 校验的 bounded fence DTO；ERUN-P2 带 V2 directory fence 的 grant 使用签名 v3（C5 基线为 v2），在序列化/哈希前先校验字段与 digest 长度，legacy CLI consumer 必须拒绝携带 fence 的 grant，只有 dedicated profile-bound consumer 可继续。Runtime 在消费前将 grant/fence 与刚重读的当前完整双 Profile、scope/lifecycle、catalog、HookSet 和 ResourceBudget binding 逐字段比较并重算 digest；在同一个 durable transaction 中一次性消费 grant nonce 与 fence ID，重复、过期、错绑、容量耗尽或 store 故障均拒绝且不能 spawn。Fence receipt store 有界（最多保留 50,000 条，过期 5 分钟偏差窗口后清理），catalog entries 不复制入 fence。此基础 helper 不替代实时 ACL、reservation lifecycle、生产 Provider/OS spawn 与 BI；这些 adapter 安装并通过验收前 producer capability 保持关闭 | P0 |

Phase 9E-4B1 增加 Worktree-scoped current Profile 只读 API：列表默认 20、上限 50、使用 UUID keyset cursor 且只返回 profile metadata；详情重新运行 Rust decode/digest/schema/scope verifier；两类响应均 `no-store`。Phase 9E-4B2 增加 Project/Worktree Profile 生命周期写 API：采用有界 tagged typed request、`execution-profile:publish` 与当前 Project admin membership；expected-current-version CAS 后，以同一短事务关闭 current revision、插入新 SCD2 revision 和 append-only Audit。Publish、disable、reenable、rollback 均通过 successor 表达；rollback 重用已验证的历史 document 并创建新的 active revision。请求最多 67,584 bytes，响应仅返回小型 receipt 且禁止缓存。API 不解析当前 Provider/Skill/Grant、不创建 Run、不预约资源。目标 DB/RLS/grants、真实 Auth Provider、Run writer 和资源 admission 仍开放；恢复或创建 Run 仍需外层重新完成 actor ACL/GroupContext 授权。9E-4B4 另要求 CLI approved launch profile 与 AgentExecutionProfile 分开绑定，当前接口尚未承载 AgentExecutionProfile identity。验收以 AC-AEC-011/012/013/014/015/016 为准。ULYS-235 Hooks 仍为 Settings“高级设置”内容区与 Skills/MCP/Plugins 并列标签。

### 50.8D Rust 原生 Hook Engine 与高级设置可视化

Hook Engine 是 Agent/Run/Worktree 的原生控制点，用于在关键操作前后施加不可绕过的授权、安全和生命周期约束并留下可分析事实。它必须由渡口 Rust 核心实现，包含强制内置规则；不是可关闭的 Plugin provider，也不能被 CLI、Skill、仓库脚手架或用户 HookSet 替换。Project HookSet 是 Master/version，Worktree 继承 Project 基线并可追加更严格的规则；不可降低平台/tenant/project 安全基线。每次 Run 固定有效 HookSet、规则与 evaluator 的 version/hash；Worktree 管理命令也记录当时策略 provenance。

Hook 配置沿用既有 ULYS-235 导航：Settings 主导航中的“高级设置”是父入口，设置页路径为 `/settings/advanced`；Hooks 的规范路由为 `/settings/advanced/hooks`，它是该页面内容区的选项卡，与 Skills、MCP、Plugins 等并列，不是独立主导航项，也不得新增 Worktree 树层级。必须提供可视化规则列表、结构化编辑器、规则解释、草稿/发布版本 diff、继承与覆盖视图、冲突/不可覆盖提示、影响范围预览、dry-run/历史事件模拟、审批和 rollback。规则通过受限 typed condition/action schema 配置，不要求编写代码；拒绝任意脚本、shell、动态库和无限制表达式。高级设置负责策略定义和规则管理；Project Worktree Index 显示该 Worktree 生效的 HookSet/version/健康状态和常见阻断原因，Run detail 与 Quality & Improvement 提供可下钻执行记录和 BI 分析。

原生同步 Hook 至少覆盖 Run admission、工具调用前后、验证前后、review/complete、Worktree 创建/导入/归档/恢复/转派/binding 变更/清理。决策限定 `allow / deny / require_human / defer`；Hook 不能授予 capability、改变目标/argv、改写 Task Contract 或代替验证/人工接受。critical hook 使用受限、确定性的 Rust evaluator，无网络与任意代码装载，配置/引擎/审计不可用或超时则 fail closed。non-critical post-commit 通知/指标 enrich 通过有界 Outbox 异步执行、可重放且不回滚已提交事实。Plugin 可提供隔离的 advisory hook，但不能替代核心决策。

Worktree 的归档、解绑或物理清理前，Hook 与 Worktree domain command 必须重新校验 Project membership/role、lifecycle version、Runtime health、活跃 Run/Agent lease、file claim、PTY/process drain 和新鲜 Git retention-lock observation；任何 unknown/过期/冲突信号都阻断破坏性操作。Agent lease、file claim 和 Git lock 是独立信号。成功提交后写 append-only Audit/Outbox 并刷新 Worktree Index 授权投影。

每次 Hook 评估记录 HookSet/rule/evaluator version/hash、phase、decision/result class、duration、timeout/fail-closed/override、Project/Worktree/Run/Task/actor/correlation scope；不存 Secret、prompt、完整 stdout 或模型隐式推理。人工 override 限定可覆盖等级、角色、理由、期限并审计，核心安全规则不可 override。BI 从 Hook Event/Audit/Worktree/Run/Event 派生版本化 coverage、allow/deny/require-human、timeout/failure、override、阻断/恢复时间，并与 Worktree 冲突、lease/claim、drain/cleanup 故障、验证失败、返工与接受结果关联。unknown coverage 不等于零次触发或零风险；HookSet 改善提案经过独立审批和固定 Benchmark，不能由 Hook 自动降低自己的保护或评分标准。

Phase 9D 的有界摘要使用 metric v2 合并 Hook 执行账本与字段完整的 `hook_evaluated` RunEvent 投影，以 `(tenant_id,event_id)` 去重，并以 `(tenant_id,project_id,work_item_id,run_id)` 关联最新 Run 状态；不使用 correlation ID 作为事件身份。缺少 phase/decision/duration/timeout 的 RunEvent 计入排除数据质量计数。摘要窗口限制 1–90 天，coverage 仍按真实 producer 状态报告为 partial/unknown；状态 join 完整不等于 Hook 阶段 coverage 完整，也不等于所有 Run 已达到终态。

| 要求 ID | 要求 | 优先级 |
|---|---|---|
| HOOK-001 | Rust 核心原生提供版本化、fail-closed Hook Engine 与不可关闭的 builtin guard；HookSet/规则/evaluator version/hash 固定到 Run 和 Worktree command provenance | P0 |
| HOOK-002 | 高级设置 → Hooks 提供无代码的可视化规则管理、继承/覆盖、版本 diff、dry-run、冲突说明、审批与 rollback；拒绝 arbitrary code | P0 |
| HOOK-003 | Hook 覆盖 Run、tool、validation、review 及 Worktree lifecycle 关键点，只能 allow/deny/require_human/defer，不能授予权限或改写命令/验收事实 | P0 |
| HOOK-004 | Worktree destructive command 前复验 ACL、lifecycle version、Run/Agent/file/process drain 和 fresh Git lock；信号 unknown/过期/冲突 fail closed，并由 domain command 再原子校验 | P0 |
| HOOK-005 | Hook outcome/latency/failure/override 作为 scope/version/correlation 完整的 append-only event；敏感正文不进入 event；非关键 after-commit 采用有界可重放队列 | P0 |
| HOOK-006 | HookEvent 与 Worktree/Run/Evidence/Audit 联动进入 BI，支持规则版本/Project/Worktree/task cohort 下钻、coverage 与质量/冲突/人工介入关联，未知值保留 unknown | P0 |
| HOOK-007 | Plugin Hook 仅可在隔离运行时提供受 grant 的 advisory/post-commit capability，不得替代/关闭/减弱 Rust builtin guard | P0 |
| HOOK-008 | HookRule 按受支持的同步 phase 作用域执行；兼容旧 v1 archive policy 与 digest；Run admission 在 producer/readiness 尚未接入时不得启用或宣称覆盖 | P0 |
| HOOK-009 | 新 Run admission 在锁外完成有界 Runtime readiness/fencing，在短授权事务内重验并原子写入 HookSet snapshot、Run、Hook ledger 与共享 event_id 的 RunEvent；拒绝不创建 Run；缺少真实 Runtime adapter 时保持 fail closed | P0 |

| 验收 ID | 受入基准 |
|---|---|
| AC-HOOK-001 | Settings 主导航中的“高级设置”作为父入口，规范路由为 `/settings/advanced`；Hooks 规范路由为 `/settings/advanced/hooks`，位于该页内容区并与 Skills/MCP/Plugins 并列；可视化表单创建、比较、模拟、审批和回滚 typed rule，无须写代码。Run 深链中的 Project/Worktree 仅作 hint，必须经当前会话授权目录确认；pending/非法/越界/目录不完整/会话替换时关闭策略、事件读取和写操作，不静默切到其他项目；用户可显式清除 hint 后手动选择。深链不授予权限，不进入策略写 body |
| AC-HOOK-002 | Project 基线与 Worktree policy 合并后只能等强或更严格；Run 和 Worktree 命令可回看命中的版本、规则、decision 与理由 |
| AC-HOOK-003 | Hook 缺失、超时、版本不兼容或 audit 无法持久化时，关键 Run/tool/Worktree cleanup 命令阻断；非关键通知任务可按有界重试重放 |
| AC-HOOK-004 | 过期 lock observation、活跃 Run/Agent lease/file claim/子进程阻止 Worktree archive/cleanup；drain + 新鲜重检后才允许继续，重复请求不重复执行 |
| AC-HOOK-005 | BI 可下钻 Hook rule/version → HookRun/Event → Worktree/Run/Evidence/Audit，并报告 coverage、deny、timeout、override、运行成本、返工/接受关联；缺失数据为 unknown |
| AC-HOOK-006 | Hook Engine 队列、CPU、内存和运行时间有硬上限；配置、plugin 或 worker 故障不造成 UI 阻塞、无界缓存或绕开内置 guard |
| AC-HOOK-007 | Phase 9D summary v2 对 Hook ledger/RunEvent 镜像按 tenant+event_id 去重，Run state 按完整 tenant/project/task/run 键关联；缺字段记录显式计数，join 状态与 producer/phase coverage 分开呈现 |
| AC-HOOK-008 | Rust evaluator tests 验证 v1 policy 仅限 archive、未指定 phase 的 rule 不跨 phase 生效、Run admission 只接受允许的 typed facts；policy publish/rollback 在 producer 未就绪时拒绝该 phase，UI Builder 禁用配置 | 100% negative/compatibility cases pass；不可支持配置 fail closed |
| AC-HOOK-009 | REST/DB 验收验证 Runtime readiness 不持有数据库锁；fence scope/freshness/TTL 与 spawn 消费绑定；Allow 的 Run、Hook ledger、RunEvent 和 HookSet snapshot 原子提交且共享 event_id；Deny 只追加无 Run FK 的 ledger；事件列表/summary coverage 与已装配 producer capability 一致 | 正/负路径和并发/idempotency 场景通过；当前未装配的 production adapter 不得报告为已启用 |

### 50.8E 原子化领域边界与前后端服务拆分

每个右侧 Run App 是独立产品能力入口，不自动等同独立微服务。后端按事实所有权拆分 bounded context；前端 App 通过稳定、版本化 API 消费授权投影并提交命令。第一阶段采用 Rust modular monolith，维持同进程部署、模块级依赖和独立 schema owner；只有当吞吐、可用性、发布节奏、团队 ownership 或故障隔离数据证明必要时，才提升某个 bounded context 为独立服务，避免每个 UI tab 增加常驻进程、内存与运维面。

| 域 / 服务候选 | Canonical owner | 允许的职责 | 不允许 |
|---|---|---|---|
| Project / Branch / Worktree Control | Project、Branch binding、Worktree lifecycle、Git/Host Runtime projection | Project membership、远端 Branch 元数据、本地 checkout 绑定/锁/冲突、create/import/archive/cleanup commands | 写 WorkItem、Canvas 或 Run 业务事实 |
| Work Item / Planning | WorkItem、Task Contract、Board/Backlog/Sprint/Relation | Inbox、Multica/Jira projections、版本化任务命令与 review gate | 私有化第二任务状态机或直接 spawn 进程 |
| Engineering Run / Agent Runtime / Scheduler | EngineeringRun、TaskExecutionRun admission、resource reservation、Agent/Loop lifecycle | 版本快照、调度、公平配额、取消/drain、受控 Runtime bridge | 把 TaskExecutionRun 当 EngineeringRun，或绕过 Task/Hook/ACL owner |
| Canvas | Run-owned Canvas、Document、Element、typed EntityRef | Run 范围布局与跨 Worktree 引用 | 复制 WorkItem 状态、直接写任务库 |
| BI / Benchmark | 版本化 metric、cohort、coverage、Benchmark receipt、proposal projection | 只读消费 Run/Event/Evidence/Audit，授权下钻 | 修改源领域事实或把缺失数据补零 |
| Plugin / Hook Control | Plugin Registry/capability 与 Rust-native Hook policy/evaluator | 校验、发布、撤权、调用隔离；安全 Hook fail-closed | 让插件覆盖或削弱原生 Hook |

每个 bounded context 拥有自己的 schema/table 写权限和版本化 API；跨域读优先使用授权 projection/API，跨域写调用事实所有者命令。PostgreSQL stored procedure/function 只封装同一 owner 内需要数据库原子性、行锁、CAS 或 RLS 的规则，并由该 owner API 调用；禁止把 procedure 当跨服务 RPC、让一个域直接写另一个域的表，或把存储过程暴露给浏览器。跨域工作流由显式 Saga/Run coordinator 驱动，每一步使用稳定 command ID、幂等键和 append-only audit/outbox；可补偿，但不宣称分布式 ACID。

异步事实采用 owner transaction 同写 Transactional Outbox；消费者在本地 Inbox/processed-event key 上幂等去重，再更新可重建 projection。NATS JetStream 是当前事件传输基线，Canvas realtime 或 BI read model 不能替代它。版本化事件至少携带 `event_id/schema_version/tenant_id/project_id/branch_id/engineering_run_id/可选 worktree_id/actor_id/correlation_id/causation_id`。前端只读取当前 Actor 有权看的最小字段并使用分页、虚拟化和 bounded cache；服务间授权、RLS、trace 和资源预算继续 fail closed。

### 50.8F 事件流平台现状与 Kafka / Fluvio 评估

截至 2026-10-01 的仓库架构/依赖检索，当前设计选择 PostgreSQL System of Record + Transactional Outbox + NATS JetStream；未发现 Kafka 或 Fluvio runtime dependency/部署配置。部分旧源文件注释把 Redis/Kafka 写作未来候选，Canvas 旧设计也存在 Redis Streams 选择；这些是需逐步统一的历史设计文本，不能解释为当前在用。当前不引入第二个 broker：业务事实继续由 PostgreSQL owner transaction 持有，NATS 承担版本化异步领域事件；BI/Benchmark 从受控 Outbox/RunEvent 构建可重放投影。只有实测保留、重放吞吐、connector 或查询隔离 SLO 无法满足时，才启动独立流平台 ADR。

| 候选 | 与本项目匹配点 | 成本/风险 | 判断 |
|---|---|---|---|
| NATS JetStream（当前基线） | 仓库已有事件契约与 Outbox 设计；适合当前跨域通知与消费解耦，不增加第二套集群 | 需验证长期留存、大规模历史重放/connector 是否满足未来 BI SLO | 继续使用，当前最少运行部件 |
| Apache Kafka | 官方 4.3 API 提供 Producer/Consumer/Streams/Connect/Admin；Connect 有预构建 connector，适合未来广泛 BI/数据系统集成、长期留存和独立分析消费者；非 Java client 由社区项目维护 | 引入会增加 broker/partition/replication/schema/consumer-group 与资源运维面；Streams 是 Java/Scala 应用库，4.3.0 曾披露 RocksDB native-memory leak，需使用修复版 4.3.1 并监控 RSS；Rust 桌面前端并不要求 broker 用 Rust | 若 NATS 无法满足经过量化的 connector、保留或回放 SLO，优先做 Kafka PoC/ADR；限定事件平台与 analytics consumer，不改变 PostgreSQL SoR |
| Fluvio | Rust 实现、Kubernetes/edge 取向，并有 Rust client；官方称小内存占用，适合列入资源敏感 BI/replay PoC | 低内存优势是供应方陈述，尚无 Star workload 对比；官方 GitHub release 页面最新非预发布版本为 v0.18.1（2025-07-04），页面同时可见 2026 dev prerelease；必须核实支持周期、稳定版节奏、connector/托管及故障恢复能力 | 仅在相同负载基准证明 RSS/CPU 优势且稳定性、连接器和运维需求都达标时采用 |

**选择结论**：Star 仓库当前没有 Kafka 或 Fluvio runtime dependency/deployment；当前方案是 PostgreSQL SoR + Transactional Outbox + NATS JetStream。现阶段继续用 NATS，不叠加第二套 broker。若未来经 SLO 与同负载基准确认必须在 Kafka/Fluvio 中选一个生产平台，优先选择 Kafka，理由是 Connect/Streams 与集成生态更贴合 Run BI/Benchmark 的外部数据接入和历史分析需求；接受其额外运维/内存成本，并只采用包含 Kafka 4.3.1 修复的版本。Fluvio 保留为资源敏感、Rust/Kubernetes 形态下的受限候选；只有 RSS/CPU/恢复实测明显占优且 release/support、connector 和托管能力达标才反转选择。基准须固定事件集、保留时长、吞吐、消费组数、consumer lag、故障恢复和部署资源，测峰值 RSS/CPU/存储与恢复时间，记录 workload/coverage。迁移只发生在 Outbox 后的 analytics/projection consumer，不改变 PostgreSQL SoR、owner API、Inbox 去重或事务写入边界。

资料（官方，核对日期 2026-10-01）：[Kafka 4.3 API 与 Connect/Streams](https://kafka.apache.org/43/apis/)、[Kafka Streams 4.3 升级说明及 4.3.1 native-memory 修复](https://kafka.apache.org/43/streams/upgrade-guide/)、[Fluvio 0.17.2 架构/资源说明](https://www.fluvio.io/docs/0.17.2/fluvio/overview/)、[Fluvio 官方 release 状态](https://github.com/fluvio-community/fluvio/releases)。

### 50.8G 商业开源基础设施与许可证标准

Rust Host Infrastructure Manager 与其所支持的开源组件不得因商业用途、行业、部署规模、席位、用量或付费 tier 设产品限制。GPL/AGPL/LGPL 允许商业使用和销售，copyleft 不构成用途限制；每种修改、链接、捆绑、安装与再分发形态须按实际组合履行对应源码、许可证、NOTICE、安装信息等义务。履约方式应支持上游分发、符合要求的受管安装/捆绑和独立 provider 交付；不得仅因 copyleft 强制用户手工自装。排除非商业、field-of-use、source-available 和实际禁止商业使用的组件。每个版本检查实际构建/交付闭包的 SPDX/SBOM，不能只凭上游仓库根许可证放行；按适用许可证保留版权、NOTICE 与专利声明。

社区活跃度以评估日前 12 个月的维护提交或正式发布、公开维护/安全渠道、明确维护者与升级策略复核。建议 provider 组合为本地 VM 的 Multipass、容器/VM workflow 的 Podman machine、Linux shared VM/container 的 Incus、macOS/Linux 的 Lima，以及用户自有/远端 Linux K3s；它们是可替换 adapter 候选。产品须支持发现既有 provider、引导安装和履行许可义务后的受管安装/捆绑；不得因 copyleft 把手工安装设为唯一入口，也不按席位、用量或用途收费封锁。Multipass GPL-3.0 允许商业使用；随产品分发时履行 GPL 义务。上游将其定位于本地开发/测试环境，daemon 控制权不能单独构成不可信 Agent sandbox。K3s 运行在 Linux 节点/guest，不原生支持 Windows。Windows、macOS 与 Linux backend 以每个已验收版本的 capability probe 判定；截至本次审查 K3s stable channel 指向 v1.36.4+k3s1，1.37 系列仍为预发布。候选来源、许可义务及未验证范围见 DD-LOCAL-INFRASTRUCTURE-001 v0.4。

### 50.9 追溯与后续专题同步

本节为总要件基线。`SRS-MULTICA-TASK-001` 负责任务生命周期与 review gate，`SRS-MULTICA-HOOK-001` / `BD-MULTICA-HOOK-001` / `DD-MULTICA-HOOK-001` 负责高级设置 Hooks tab 的原生引擎、可视编辑、Worktree enforcement 与 Hook BI；`DD-MULTICA-TASK-001` §14.7-14.12 负责 Task Contract / Run / BI / Benchmark / Execution Profile / 双 Loop / Hook；`DD-WORKTREE-GROUP-001` §6.3-6.5 / §8.4-8.7 负责调度资源、扩展 Provider、Hook 与 CLI 边界；`DD-CANVAS-WORKFLOW-001` 的 Schedule Trigger 必须适配唯一 Automation occurrence source；`SRS-CANVAS-001` / `SRS-CANVAS-WORKFLOW-001` 负责画布与 Flow，`architecture/2026-09-03-langgraph/01-requirements.md` 负责 L0/L1 编排。`basic-design.md` §16.14-16.17 固定跨专题基本设计。后续专题文档必须继承 GroupContext、单一任务事实源、范围化聊天、Worktree-first 导航、可扩展 Agent Profile、原生 Hook、Loop、资源与跨 App 事件约束。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v2.1 | 2026-09-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Worktree Group、任务卡 CLI、范围化聊天、Canvas 互操作、插件热插拔和 LangGraph 约束 | 用户提出 Worktree 顶层索引与群组 App 体系 |
| v2.2 | 2026-09-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 导航修订为先选 Project，再展示项目 Worktree 清单，展开 Worktree 后显示同级 Group Apps；补充多 Agent Worktree 可见性和受控管理要求 | 用户澄清产品核心是解决多 Agent Worktree 混乱与内部管理不可控 |
| v2.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 Project Worktree Index / Group 的可访问入口与深链验收，禁止 Worktree Index 被通用任务列表路由替代 | 浏览器验收发现 `/worktree` 被重定向到 Sprint 树视图 |
| v2.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 规定 Project Index 深链必须携带 `project_id`，缺少/无效 Project 时不静默回退到另一项目，并将该路由行为纳入 AC-WTG-004 | Group 路由和实施计划复核发现项目入口需保留所选 Project，且无参数不应默认切换项目 |
| v2.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Canvas Document 版本化持久化要求：viewport、frames、visual connectors 受 Worktree scope、expected version、幂等与元素引用校验保护，并明确 Canvas connector 不代替 canonical WorkItem relation | Phase 3 Canvas 文档保存 API 切片实现后同步验收契约 |
| v2.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Canvas 新建 Task Card 原子受入标准：canonical WorkItem、Multica、Worktree association、Canvas Element / EntityRef、Audit / Outbox 必须同事务提交并共享 correlation ID | Phase 3D 实现 Canvas → WorkItem 原子创建命令后同步验收契约 |
| v2.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Canvas Outbox 安全读取受入标准：当前 Worktree 授权重验、复合游标与限页、元数据响应不得泄露事件 payload 或过期实体引用；明确轮询 API 不等同于 Outbox consumer / realtime 完成 | Phase 3E 新增受保护 Canvas Outbox 读取端点后同步验收契约 |
| v2.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 冻结 Group API 的宿主 JWT provider 契约：逐请求取 token、Bearer header、缺 token fail-closed，远程 origin 强制 HTTPS，禁止客户端持久化/shared API key/URL 或 WebSocket 携带，并明确 401/403 不得回退到 seed 写入 | Phase 3E 前端 API adapter 实现后补齐真实登录会话接线边界 |
| v2.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 session generation key 与身份切换清除规则，避免认证回调函数引用稳定时保留旧 actor 的 Group projection；要求 key 非敏感、不作为授权凭据且不持久化 | Phase 3E UI 接线自审发现需显式定义切换账号/登录会话后的 projection 生命周期 |
| v3.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确真实认证 Canvas projection 上的创建任务入口必须调用受保护、幂等的 Canvas→WorkItem 命令，成功后刷新授权投影并跳转 canonical Task Card；seed 不得伪造创建成功，Document 布局编辑仍单独验收 | Phase 3E 把既有后端原子创建命令接入 Group Canvas 页面 |
| v3.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Worktree 尚无 Canvas 时的认证幂等初始化入口；初始化成功后刷新授权投影，再开放 Canvas→Task Card 命令，失败时不得回退到 seed | Phase 3E 自审发现首次进入空 Worktree 无法启动 Canvas 与任务协作闭环 |
| v3.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确实时 Worktree Canvas 的 viewport 平移/缩放须可用 expected_version 与 idempotency key 显式保存；冲突不覆盖新版本，Frame/connector/element 编辑仍独立验收 | Phase 3E 接通现有 Canvas Document CAS API 后更新可用边界 |
| v3.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确一个 Worktree 可管理多个 Canvas，通过 `canvas_id` 选择/分享；创建后切换到新 Canvas，任何加载和变更仍由服务端校验当前 Worktree 归属 | Phase 3E 实现多 Canvas 投影选择与认证创建入口 |
| v3.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充将既有 Worktree Task Card 放入 Canvas 的 P0 验收：元素与 typed EntityRef 由一个认证幂等请求原子创建，服务端复验 Worktree 关联 | Phase 3E 使用同一 Canvas Element API 接通已有任务卡关联 |
| v3.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 live Canvas 元素拖动与 CAS 保存要求；移动只改变坐标，携带 Element 版本、幂等键和 correlation ID，冲突后刷新授权投影 | Phase 3E 接入已有 Element versioned update endpoint，开放安全的位置编辑 |
| v3.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 canonical WorkItem 生命周期命令的在线 UI 受入：Multica/Jira/Task Card 共享版本、合法状态操作、review gate、幂等、冲突刷新与审计 | Phase 3F 将现有服务端生命周期 API 接入 Worktree Group 视图 |
| v3.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确生产 Plugin App 同级入口只由当前 Worktree 的服务端 Registry/授权投影生成；客户端本地启用开关仅属原型预览，不代表注册、capability grant 或热插拔 | Phase 6 UI 切片将启用的预览插件展示为 Worktree 同级导航，并标明非生产能力 |
| v3.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 为 Task CLI 补充 Task Card 认证 Session start、一次性内存 ticket、Hello 后开放 xterm 输入与 pane output、禁止复用旧 ticket 自动重连的 UI 验收；明确宿主 provider、Runtime provisioner、PTY sink 和 sandbox 未装配时仍 fail closed | Phase 4B3 加入 Group Task Card → Session API → ticket-first terminal UI 接线切片 |
| v3.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 live Canvas Frame 创建/删除、元素归入 Frame、纯视觉连线创建/删除的验收；要求草稿以固定版本与幂等键显式 CAS 保存，并禁止连线冒充 canonical WorkItem relation | Phase 3E 复用 Canvas Document CAS 接通 Frame 与 visual connector 编辑 |
| v3.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 扩展 live Canvas 的 Frame 几何/演示属性和 connector 样式编辑；加入无实体绑定便笺内容 CAS 与 Element 原子删除契约，删除同时清理 Document 悬空引用且不删除 WorkItem | Phase 3E 补齐 Canvas 展示编辑与 Element 删除一致性边界 |
| v4.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加未锁定 Canvas Element 尺寸与旋转的独立版本化写入验收；要求保留其它字段、幂等与冲突刷新 | Phase 3E 将 Element 尺寸/旋转接入现有版本 CAS API |
| v4.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 强化 Worktree Index 的管理面责任；新增授权 API 投影、fail-closed seed 边界、游标续页和归档/恢复 plan-confirm 验收 | Phase 2D 将现有 Index 与管理计划 API 接入 Worktree 管理 UI |
| v4.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Project ACL 保护的成员目录和负责人转派验收：仅使用当前 Project 有效成员，基于当前 Worktree version 经过短时 plan-confirm、幂等审计及投影刷新 | Phase 2D 补齐 owner SCD2/管理计划 API 已有但 Index 缺失的成员选择与转派 UI |
| v4.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Canvas Outbox 浏览器投影消费者的 Worktree/Canvas 复合游标持久化与恢复要求；明确本地游标仅为可丢弃刷新提示，不能替代服务端消费 offset 或 NATS JetStream | Phase 3E 将内存游标扩为浏览器本地可恢复游标，同时保持服务端逐请求授权 |
| v4.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 细化 Task Card review gate 命令：claimant 请求评审、不同 reviewer 通过或驳回、拒绝理由、version CAS、幂等及 append-only audit；通过转 `completed`、驳回转 `failed` | Phase 2C / 3F 增加受 Worktree scope 保护的 review command API 与 Group UI |
| v4.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Task CLI Ed25519 grant 要件：issuer 私钥与 Runtime 公钥分离、带 key id 轮换、完整 context 签名、验签先于 scope/checkout 校验与单次 nonce 消费；明确签名不替代当前 ACL / Runtime health，且 API / spawn 尚未接通 | Phase 4A 增加独立 grant signer/verifier 与 fail-closed 执行准备入口 |
| v4.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 authenticated Task CLI Session start API 的授权与委托契约：Project membership、Worktree/WorkItem 关联、claimant、lifecycle version、Runtime/profile、Idempotency-Key、correlation ID、single-use attachment ticket 与 no-store；明确 REST route 仅有 injection seam，未配置真实 provisioner 时返回服务不可用 | Phase 4B1 加入 Group REST Task Session API 接入边界，保持无 Runtime 环境 fail closed |
| v4.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确任务卡 CLI 的交互式 PTY、bounded input/resize、字节输出与退出状态、清空进程继承环境和 OS sandbox 独立约束；记录 Local Runtime 有 PTY adapter 代码切片但尚未接 provisioner / terminal-stack / OS sandbox | Phase 4B2 建立 Local Runtime PTY adapter，避免将 pipe 或 PTY 误当成受控生产终端 |
| v4.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 attachment 前输出须进入有界 FIFO 背压队列、不得静默丢失，并要求 Runtime manager 退出时终止所拥有的 PTY 子进程；记录当前仍是未接线的内部 adapter | Phase 4B2 自审发现 attachment 前无 receiver 会丢 PTY 首段输出，并补充 Runtime 进程回收语义 |
| v4.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Task Card 认证 Session start、ticket 只驻留内存、首帧 WebSocket 授权后才启用 xterm stdin/输出，以及无 reattach API 时禁止重用旧 ticket；记录当前只是条件式 UI/API 切片 | Phase 4B3 将受保护 Session API 接入卡内终端面板，并显式保持重连 fail closed |
| v5.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 scope-aware Group Chat 提交要件：显式 WORKTREE/GLOBAL 与目标集合、逐目标 GroupContext 授权、EntityRef scope、幂等指纹和 fail-closed L0 workflow seam；明确 LangGraph runtime、持久 transcript、checkpoint/resume 与流式 UI 未完成 | Phase 5 开始落地群组聊天服务端授权边界，避免把 UI scope selector 当成授权 |
| v5.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增当前成员可见的非归档 Worktree 目标目录、稳定 ID 游标分页与 GLOBAL 多选 UI 要件；目录只返回最小投影、不构成 grant，提交时逐目标实时重新授权 | Phase 5 补齐 GLOBAL scope 的受权目标选择入口并保留 fail-closed 发送边界 |
| v5.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Task CLI Session status/cancel/reattach 接口验收：每次操作重验 Project membership 与 canonical Task/Worktree scope，Runtime 重验完整 session binding；状态响应脱敏、cancel 幂等、reattach 新签短时单次 ticket；明确 UI 与真实 provisioner 尚未接通 | Phase 4B 增加 Task Session 生命周期控制面 API seam，并保持生产执行 fail closed |
| v5.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Task Card Session 状态刷新、取消与断线后人工重连接入认证 API client；重连先查询状态并只接受 running/disconnected，再校验新 ticket 与 session scope；明确内存 Session 关联不支持页面刷新恢复，需后续受授权的 session listing/recovery API | Phase 4B3 完成生命周期 UI/API 调用切片并显式记录刷新恢复缺口 |
| v5.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Worktree/Task-scoped Session list REST seam 和 Task Card 刷新后 Session 发现/手动恢复 UI；列表最多 50 条、默认 20 条、仅脱敏状态且 no-store，不含旧 ticket/终端输出；恢复仍逐次查状态并签发新 ticket；真实 provisioner/provider 尚未装配时保持 503 | 继续推进全部 Phase，关闭 Task CLI 页面刷新后无法重新发现 Session 的设计与 UI 缺口 |
| v5.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于 LangGraph 当前官方 checkpoint/runtime 文档冻结 Group Chat 的安全契约：thread ID 服务端生成并与业务 ID 分离；checkpoint 不承载授权；resume、interrupt、replay、capability call 重新授权；副作用通过幂等领域命令/outbox；明确 LangGraph service、Transcript/checkpoint 存储与 stream UI 尚未接入 | 继续推进 Phase 5，先收紧 LangGraph checkpoint / replay 的授权与幂等边界 |
| v5.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 5 PostgreSQL Transcript/Run/outbox persistence 要件；冻结六张表的逐表 W/T、30 天 W TTL、T append-only/Audit、tenant+actor RLS 与原子 receipt 幂等语义；记录 adapter/schema 有代码但未接 production main、outbox consumer/L0、cleanup job、目标 DB 与 ACL/RLS 验收仍缺 | 继续推进所有 Phase，建立提交请求的原子持久化边界 |
| v5.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Group Chat 正文遵循 Agent Policy 的加密、默认 90 天可配置保留与到期清理；区分 append-only Audit 元数据和有限期正文；标记当前明文 `TEXT` migration 不可部署，并新增密钥轮换、crypto-erasure、导出/删除验收 | Phase 5 自审发现 Transcript 明文持久化与正文保留政策不兼容 |
| v5.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Phase 5 Transcript 拆成 T 元数据与 W 加密 payload；`PgScopedChatWorkflow` 强制注入 `TranscriptBodyProtector`，migration 不再包含明文 body；真实密钥服务、轮换/销毁、清理与授权导出/删除仍是生产门禁 | 按 CHAT-006 将持久化代码与 schema 收紧为 fail-closed 加密边界 |
| v5.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 fail-closed `GET /group-apps` 导航投影要求和 AC-PLG-002；Registry provider 必须复验当前 membership、binding、兼容状态与能力授权，返回最小安全投影；runtime/enable-disable/grant provider 仍未实现 | 继续 Phase 6，先建立 Worktree Group App 导航的授权 Registry API 边界 |
| v5.10 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Plugin Registry 五表 Master/SCD2 与 append-only Audit 数据要件；`PgGroupAppRegistryProvider` 由生产 API 装配并在读取事务内复验 membership version、Worktree binding/archive、manifest verification/Host API compatibility 和 actor `group_app:open` grant；增加 AC-PLG-003，明确 migration 尚未部署且 trust root/lifecycle writer/runtime/UI consumer 未完成 | 继续 Phase 6，推进从导航 API seam 到只读 PostgreSQL Registry projection，并固定逐表 W/T/M 守则 |
| v5.11 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 AC-PLG-002 扩充至 Group UI live consumer：服务端授权 projection 成为同级插件入口唯一来源，校验 Worktree/version/entries，加载与错误不回退预览，仅无 provider 的原型环境显示本地预览；刷新和 provider/session 切换边界写入需求 | 继续 Phase 6，接入 PostgreSQL 授权导航并保持错误时 fail closed |
| v5.12 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Group App 授权导航响应校验细化为 UUID correlation、非负安全整数 Registry version、最多 100 条、唯一 plugin ID、受限字段长度/控制字符与 int32 sort order；要求按 sort order 与 plugin ID 稳定排序；补充实现与聚焦测试证据 | 继续 Phase 6，验证服务端投影的客户端消费边界并扩充 WG-ACC-19 |
| v5.13 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Group PostgreSQL runtime SQL grants 与 RLS 分离验收；记录 Phase 5/6 迁移在隔离库幂等重放、12 表 FORCE RLS、租户/actor 策略和 append-only trigger 实测；目标 DB、实际应用角色与正式 grants 尚未确定 | Phase 5/6 隔离数据库验证发现 SQL role privilege 未由 RLS policy 或迁移自动提供 |
| v5.14 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Project Worktree Index 的实时 Git Worktree retention lock observation 契约；明确该信号不是 Agent 活跃或编辑互斥状态，与持久化 `locked` 字段分离；缺少/失败/过期显示 unknown，unknown/unlocked 均不能通过清理前置检查，且清理前须检查独立活跃状态并在 drain 后复验；仅记录当前 observer 接口/UI 切片，宿主 Runtime observer 未装配 | 继续推进 Phase 2D，补齐 Worktree 管理面的锁可见性并明确安全清理前置条件 |
| v5.15 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Worktree create/import 生命周期 API 要件：Project Repository binding、服务端路径解析、不透明候选 ID、严格拒绝客户端路径/URL、授权复核、幂等 operation 与 Audit/Outbox；缺少 Host Runtime provider 时 fail closed；明确本轮只增加条件式 API contract，不代表 production provider 或 UI 已接通 | 继续 Phase 2D，推进 Worktree Index 的安全创建与导入闭环 |
| v5.16 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Project Repository 脱敏发现 API 与 Index create/import UI 契约；要求服务端当前 Project projection、路径/URL 安全名称、客户端严格字段 allowlist/关联/回执复核、写后刷新和 provider 缺失 fail closed；实现仅为条件式 UI/API seam，生产 provider、binding 数据源和宿主认证仍未接通 | Phase 2D 将 Worktree 生命周期接口推进到 Project Index 可交互入口 |
| v5.17 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增基于当前租户有效 membership 的 `GET /api/v1/projects` 与生产 Project Selector；限定 ID/role 投影、稳定 UUID cursor/200 条上限/no-store，前端拒绝未识别字段并支持分页/重试；API session 更换时清除旧 Project/Index/member-role projection；生产 Project 名称 SoR、宿主 provider、目标 DB/ACL/RLS 部署仍未接通，selector 不使用本地 seed 冒充 | 继续 Phase 2D，将 Project 选择源接到当前用户授权目录并关闭跨 session 旧投影窗口 |
| v5.18 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Task Contract/Run/Event/Evidence、Agent 声明与独立验证/人工接受/集成事实分开；增加 Project Quality BI、Benchmark 与可回滚 Improvement Proposal；Phase 8A Run schema/CLI start writer 状态标为条件式、未部署 | 用户引用“AI提升方向”对话，要求将 Run 和 BI 架构融入 Worktree-first 任务 |
| v5.19 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Rust 桌面多 Agent 资源预算、Schedule/Engineering Loop、版本化 Agent/Memory/Skill/Context/Validation/Profile contract 与 Rust CLI adapter；明确唯一 Automation occurrence source；新增 Rust-native fail-closed Hook、ULYS-235 Advanced Settings Hooks tab、无代码可视化策略、Worktree lifecycle enforcement 与 Run/BI correlation；本版仅定义架构，不把未实现引擎/provider/UI 标为完成 | 用户要求高性能 Rust 多代理、Loop、可扩展 AI 能力和原生/可视 Hook 与 Worktree/BI 联动 |
| v5.20 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Worktree/Task-scoped Run list/detail 的认证查询契约、稳定游标与 20/50/100 条硬上限；明确状态维度独立投影、响应脱敏边界和 Task Card 内历史 UI。代码切片与隔离 migration 验证不等同于目标数据库部署或生产 RLS 验收 | Phase 8B 已加入 Run read API 与 Task Card 历史面板，需求需同步到可审计的实际接口 |
| v5.21 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Phase 9D summary v2 的 Hook ledger/RunEvent 双来源去重、完整 Run 状态关联键、缺失投影计数和 partial/unknown coverage 分离语义；Hooks 仍是 ULYS-235 Advanced Settings 并列 tab | 9D-4 建立双来源 Run-state summary read model，需求同步到可复算 BI contract |
| v5.22 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 phase-scoped HookRule evaluator API v2 契约、v1 archive policy digest 兼容、Run admission typed-fact allowlist 与 producer 未就绪时禁用 Builder 配置要求；保留 Hooks 在 ULYS-235 Advanced Settings 内容区并列标签中的导航位置 | Phase 9D-5a 扩展 Rust evaluator 的 Run admission phase contract，Run producer 与事务双写仍未接入 |
| v5.23 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充条件式 Run admission readiness/fence、短事务重授权/evaluator、Run/HookEvent/RunEvent 同 event_id 双写与动态 coverage contract；明确当前无生产 TaskCliSessionProvisioner adapter，UI/policy capability 因而默认关闭；Hooks 继续位于既有 Advanced Settings 并列标签 | Phase 9D-5b 接入 CLI Run admission REST/DB producer seam 并复核 BI 去重与 fence 边界 |

| v5.24 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 9E-1 Rust immutable AgentExecutionProfile snapshot verifier、scope/digest/canonical list/capability 与 Memory/Context/Loop/RSS/queue hard ceilings；明确 profile resolver、Run persistence 与 Rust CLI adapter 仍未接通；Hooks 继续位于既有 Advanced Settings 并列标签 | 开始 Phase 9E 的类型化执行 profile 核心切片 |
| v5.25 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 AgentExecutionProfile 必须固定 ContextAssembler 与 LoopPolicy provider/version/digest/grant，不能只记录 compaction digest 或 Loop 数值预算；Run occurrence 与 Task/Memory evidence 仍作为 Run admission snapshot 维度 | 9E-1 自审发现上下文与循环实现版本缺少可复现引用 |
| v5.26 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 9E-2 当前依赖 resolver 受入基准：bounded Provider/Skill catalog、grant expiry/version、HookSet 与 Worktree lifecycle 全部精确匹配且 fail closed；明确该 domain seam 不替代 ACL、DB writer 与原子资源预约 | Profile verifier 具备后推进当前注册表与 Run admission 的一致性检查 |
| v5.27 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Phase 9E-3 Profile Master/SCD2 与 append-only Audit 的数据库受入契约，要求 scope/schema/digest 一致、revision 单调、FORCE RLS；区分 additive migration 与尚未接通的 API、Run snapshot writer、目标 DB 和资源 reservation | 将 9E-2 域层 resolver 推进到 Profile 持久化基底 |
| v5.28 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 9E-4A Run Profile 快照 all-or-none 与 tenant/project/Worktree scope/digest 数据库约束；保留无 Profile FK；注明 migration 尚未隔离库执行验收 | 修复 Run nullable Profile 字段可能形成不完整或跨 scope 快照的风险 |

| v5.29 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9E-4A Run/Profile guard 三迁移隔离 PostgreSQL 验收：重复应用成功，旧 Run 与 Project/Worktree 完整快照接受，5 类不完整/错 scope/digest 负例拒绝，零 Profile 外键，临时数据库清理 | 完成 Run Profile snapshot database invariants 验收 |
| v5.30 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Phase 9E-4B1 current Profile bounded list/detail API、actor/Worktree scope、metadata-only list、Rust read-time verifier 与 no-store；明确读 API 不等同于 Profile 发布或 Run admission；Hooks 继续是 ULYS-235 高级设置内容区并列 tab | Profile read API 代码切片与单测完成 |
| v5.31 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Phase 9E-4B2 Project/Worktree Profile 生命周期写 API：typed body 上限、管理员 scope、CAS、SCD2 successor、publish/disable/reenable/rollback Audit 原子事务与 no-store 小回执；明确此 API 不接 Run writer/current catalogs/resource reservation，保留 ULYS-235 高级设置 Hooks 并列 tab | Profile 管理写路径与一致性边界进入实现 |
| v5.32 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Phase 9E-4B3 effective HookSet 身份桥接要求：Run/Profile admission 冻结当前 Project baseline 或 Worktree overlay 的 ID/version/digest，继承不一致或 verifier 失败时拒绝；Hooks 配置继续位于 Advanced Settings 内并列标签 | 将 verified Hook policy 与 AgentExecutionProfile admission snapshot 对齐 |
| v5.33 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 区分 Task Card CLI 的 Approved Launch Profile 与 AgentExecutionProfile；新增两种 identity 在 request fingerprint、Run snapshot 与 Runtime fence 中分别绑定的受入要求和缺少 Provider/Skill/Grant catalog、resolver 或 snapshot writer 时禁止创建/启动的 fail-closed 门禁；Hooks 继续是 Advanced Settings 内并列 tab | Run writer 审查发现当前 API 只有 launch profile ID，尚未选择或持久化 AgentExecutionProfile |
| v5.34 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Phase 9E-4C1 的 Profile metadata picker 与显式选择要求、50 项 bounded 上限、无 default/seed fallback、profile-bound capability 默认关闭和 request fingerprint v2/legacy replay 兼容；明确 C1 不等于新 Run 可用，C2-C4 仍需完成；保留 ULYS-235 Advanced Settings Hooks tab 位置 | Task Card Profile 选择与 REST identity/fingerprint 代码切片落地 |
| v5.35 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Phase 9E-4C2 reference-scoped current catalog snapshot（Provider ≤5、Skill ≤128、≤1 MiB）、Arc sharing 与 5 秒 revision fence/1 秒 transaction margin；明确当前缺权威 Provider/Skill/Grant persistence adapter 时 capability 必须关闭；重申 ULYS-235 规范路由 `/settings/advanced/hooks` 为高级设置内容区并列 tab | 用户要求持续推进所有 Phase，并确认 Hooks 沿用既有高级设置导航需求 |
| v5.36 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 9E-4C2 同步为实际 migration、bounded PostgreSQL reader 与 final transaction fence recheck；记录 W/T/M、FORCE RLS、SCD2/audit/deferred count guard、隔离 PG18 迁移与 mutation 约束验证、133 domain tests、118 REST tests 及 `star-api-rest` cargo check 通过；明确 publish/source 管理、目标 DB/RLS grants、Runtime adapter 与 Run writer 未关闭，能力仍默认 false；Hooks 保持既有 Advanced Settings tab 路由 | 完成 Phase 9E-4C2 SQL/read/recheck 代码切片并验证 |


| v5.37 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更正 9E-4B4/C1/C2 阶段状态：current catalog migration/read/recheck 已落地，但 catalog publisher、生产数据装载、Run writer 与 Runtime adapter 仍未完成；增加 Skill capability 载入前 ≤384 KiB 数据库聚合预算，降低并行 admission 瞬时 Rust heap 峰值；删除重复的 B4 状态段 | 自审发现 C2 完成状态描述过时且 B4 段重复 |
| v5.38 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-AEC-017：C3 将 Run/Profile/Task/Hook/Event 与 Project-wide ResourceBudget reservation 原子提交；以 Project allocation epoch 序列化 REPEATABLE READ admissions，防止跨 Worktree 并发超额；幂等 replay 不重复预留；未接通 production catalog publisher、Runtime reservation lifecycle、目标 DB/Auth/RLS 与完整 BI 前仍 fail closed；Hooks 继续位于 ULYS-235 Advanced Settings 内容区并列标签 | 完成 Phase 9E-4C3 Run 与资源预留代码切片并修正并发快照风险 |
| v5.39 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-AEC-018：一次性 Runtime fence 绑定双 Profile revision、scope、catalog revisions、HookSet、ResourceBudget 与用户请求；Run 持久化 Approved Launch Profile identity 和版本化 binding digest，Run Detail 可追溯；opaque fence ID 不进 Run snapshot；说明 adapter 未装配前继续 fail closed，并明确原请求幂等 fingerprint 与 server-resolved fence digest 的不同职责；Hooks 保持 ULYS-235 Advanced Settings 内容区并列 tab | 完成 Phase 9E-4C4 双 Profile spawn-fence 与 Run 投影代码切片 |
| v5.41 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-AEC-019：共享 typed dual-Profile fence DTO、C4 signature v2、Runtime binding recheck 与 grant nonce/fence 的原子一次性消费；规定 receipt 上限与 TTL cleanup，并明确该基础设施不等于生产 ACL/reservation/OS spawn/BI consumer，capability 继续关闭 | 推进 Phase 9E-4C5 Runtime fence consume foundation |
| v5.42 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 Project → Cloud Branch → Engineering Run → Worktree 主导航和 Run-owned tabs/BI/Benchmark；Project Worktree Index 限定为 aggregate 管理视图；补充 owner API、同域存储过程、Outbox/Inbox、Rust 桌面资源边界与 NATS/Kafka/Fluvio 选择门；新增 AC-ERUN-001/002、AC-EVENT-001；明确未迁移 Run schema/API 仍保持未完成 | 用户澄清 Branch/Run/Worktree 层级并要求服务原子解耦及 Kafka/Fluvio 评估 |
| v5.43 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 WTG-020：Project 导航提供 Worktree Index 入口；本地 selected project 只能作为深链提示，目标页必须重新读取 membership 并授权，不能使用固定 Repository/Worktree ID 或 seed 填充导航；保留 Cloud Branch/Engineering Run 权威目录未实现的状态 | 移除 Project 侧栏中指向固定 repository ID 的旧 Worktree 卡片，并提供授权 Index 链接 |
| v5.44 | 2026-10-02 | Ulysses（一人公司12角色 per DEC-008）— Mavis接手审核 | WTG-021/AC-ERUN-003：可信Branch/Run目录、三层current grant、owner/focus版本分离与有界懒树；同步方向指引与实际未完成门 | canonical目录与导航基础实施、独立源码review改进 |
| v5.46 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 §50.8G 商用宽松许可和活跃社区准入门，明确 Incus/Lima/K3s/WSL2/Multipass 边界及实际分发依赖 SBOM/NOTICE 检查 | 用户明确要求不限用途商用且社区活跃的开源方案 |
| v5.47 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按用户澄清重写基础设施商业开源准入：商业用途不按席位/用量/行业封锁，copyleft 不等同于禁止商用，加入独立 provider 履约交付路径；推荐 Multipass 本地 VM + Linux K3s，并保留 Podman machine/Incus/Lima 替换 adapter 和确切平台能力验证 | 用户明确不接受商业用途限制并要求活跃社区开源方案 |
| v5.48 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 移除“copyleft provider 只能手工自装”的隐性产品限制，要求发现、引导和合规受管安装/捆绑路径；澄清 unlimited commercial use 与 GPL 发行义务的区别，并纠正 K3s stable channel 版本；同步 basic design v5.45、Group DD v4.32、Infrastructure DD v0.4 | 用户再次明确拒绝商业用途限制，要求活跃社区开源方案 |
| v5.49 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-ERUN-005 与 AC-TASK-DATA-001：为当前 Task Card 固定完整 Run owner tuple、Outbox 与同 Run 读写/CLI/Canvas 校验；移除 30 条旧 mock seed 及其本地历史引用，并明确精确 localStorage 清理、保留测试 fixture、禁止按前缀删除真实数据库行；目标 DB/RLS 验收仍未完成 | 用户授权清理全部旧 mock 任务，并要求把 Run 层级作为真实任务归属 |
| v5.50 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-ERUN-006：Run 默认 Task Cards 投影、认证 Run-scoped bounded list、单页内存与页数上限、无旧 Worktree/mock 回退，以及 CLI 生产闭环前禁用；列清前端代码切片与当前宿主 Provider、服务端 capability、目标 DB/RLS 门禁 | 将实际 Run Task Cards UI 代码与认证/数据库未就绪事实对账 |
| v5.45 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-ERUN-004：server-resolved directory identity、两事务重验、V2 fence/signature v3、严格完整快照与新 CLI NULL guard、attachment 再授权；扩展 AC-HOOK-001 的会话授权深链和有界查找 | 独立 worktree 并行实现 CLI 身份与 Hooks 深链、源码第二意见修正 |
| v5.51 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 扩展 AC-TASK-DATA-001：清除独立 Tauri 桌面端的四条运行时演示 WorkItem，移除 MockDb 任务记录和 browser-dev fallback；缺少 canonical Run provider 时 IPC fail closed，测试 fixture 统一使用 `test-*`；不删除服务器端未知归属行 | 全仓审查发现旧 Tauri 桌面端仍暴露演示 Task Card |
| v5.52 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 为 Schedule/Engineering Loop 增加 Phase 9F1 当前实现对账：Run-local Rust controller 已有 bounded iteration、Profile/Contract/Acceptance/Hook/Validation snapshot guard、预算 stop、进度检测、独立验证 gate、tool permit backpressure 与 drain receipt；明确成本预算字段、retry/backoff、Run admission/Auth、持久化、Schedule、BI 与公平调度仍缺，未将其写成生产闭环 | 实现受限 Engineering Loop 核心并同步验收范围 |
| v5.53 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-LOOP-007 与 Phase 9F2 当前实现对账：versioned Automation rule/occurrence/lease fence domain contracts、five-table W/T/M + FORCE RLS migration substrate 和 20/20 domain tests；明确本机无 PostgreSQL/Docker daemon，SQL/RLS 未实证，parser/worker/API/Run admission/Outbox/BI/target DB 仍开放且 capability 关闭 | 推进 Schedule/Occurrence durable substrate 并对照需求和实现 |
| v5.54 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 加强 Phase 9F2 AC-LOOP-007 对账：occurrence key 明确为 tenant + rule ID + rule version + UTC slot；dispatch contract 记录 DB monotonic/contiguous fencing、active lease steal guard 与 terminal-based TTL；保持 DDL/target DB/runtime 验收未完成 | 自审发现 tenant 幂等键与 dispatch fencing/TTL 的 database invariant 需同步到需求 |
| v5.55 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 Phase 9F3 recurrence materializer、PostgreSQL occurrence/lease adapter 与 disposable PostgreSQL 18.6 的 migration/FORCE RLS/4 场景证据；说明生产 rule API、Run admission/Auth/Outbox/BI 与目标 DB 仍开放 | 实现 9F3 adapter 并完成需求/实现/数据库验证对账 |
| v5.56 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 收紧 9F3 Schedule 验收：disabled Rule 必须在物化层 fail closed；lease-expired audit event 固定记录旧 attempt 与旧 fencing generation；候选槽及 DST 探测均受 32,768 上限约束，domain release tests 26/26；保持 9F4 生产闭环门开放 | 最终自审发现停用规则、lease 审计对应关系和 DST 转换扫描预算需明确入规 |
| v5.57 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 在 §50.8B 记录 9F4A Run-scoped Rule API 的授权、目标快照、CAS/幂等、Audit/Outbox 原子性与验证边界；明确 worker/Run admission/BI 仍未实现 | 将 Rule API 切片及其开放验收门同步到正式需求 |
| v5.58 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对账 9F4A 独立审查与验证：future-dated currentness 过滤、Run-consistent Outbox FK、SCD2 transaction boundary、expired-key reuse 和私有响应缓存；记录 focused Rust/isolated PostgreSQL 证据，保持 route/target DB/worker/BI 门开放 | 修复实现后同步需求与验收证据 |
| v5.59 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录通过生产 `build_group_router` 的四方法未认证请求覆盖及私有响应头断言；明确它只验证路由接线/认证拒绝路径，不关闭有效身份、scope/role、Run ACL、target、数据库、worker、admission 或 BI 验收门 | 补充 Schedule route-level 验证后对齐需求与实现状态 |
| v5.60 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录确定性 RustCrypto RS256 后端及 production router 的签名 JWT scope 拒绝/body 上限测试；精确保留成功授权、DB ACL、分页、CAS/replay 和执行闭环为未验收 | 修复 JWT 测试发现的缺失 crypto provider 并同步实际 HTTP 证据 |
| v5.61 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 AC-LOOP-008：固定 Schedule 创建者 run-as、数据库禁止 successor 更换主体，并要求每次触发复验 Project/Run 权限，撤权即拒绝创建 Run；明确当前实现不含 worker/Run admission | 用户选定规则创建者为 run-as 且撤权后 fail closed |
| v5.62 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 加强 AC-LOOP-008：首版数据库写入必须证明 run-as 等于创建者，且 occurrence 必须匹配精确 Rule revision；登记 9F4B PostgreSQL 18.6 重复 migration、FORCE RLS 与 5 个真实 adapter 场景验证，同时保留 worker 实时授权与 Run admission 未实现门禁 | Rule/occurrence 身份 guard 与完整 Schedule phase runner 验证完成后对账 |
| v5.63 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 9F4B 历史双 revision/不同编辑者与旧 occurrence 的 backfill fixture、schema-owner migration 事务边界和验证数量；明确目标 DB grants、worker ACL recheck、Run admission 与 BI 未验收 | 自审发现空库 migration 未覆盖已有 Rule/Occurrence 身份回填，补齐夹具并修复 guard/RLS 阻断 |
| v5.64 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 AC-LOOP-009 Schedule occurrence→Run admission 不变量；记录 9F4C-A 的不可变 admitted Run 关联、schedule Run Outbox、8 表 FORCE RLS 与 PostgreSQL 负例证据；明确 admission writer、实时授权/预算复验、Reservation/RunEvent、consumer 与 BI 未验收 | 用户确定 immutable run-as 撤权 fail-closed 后继续 Schedule Run admission persistence 阶段 |
| v5.65 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 9F4C-B Rule create/enable 的 run-as 当前 Project/Run writer 与 active Run 检查、撤权后可停用契约；明确这不替代每次触发授权，也不关闭 Schedule worker/admission/BI 门 | 用户确认 Schedule creator 固定 run-as、每次触发复验且撤权 fail closed，并继续落实 API enable gate |
| v5.66 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 9F4C-B 的纯角色策略 helper 单测 1/1 通过，并把 SQL ACL fixture 缺失与 trigger/retry/resume worker 未实现作为独立缺口；保持 AC-LOOP-008/009 未通过 | 补充最终隔离 target 链接测试结果并复核文档不得将 enable-time gate 记作执行时授权 |
