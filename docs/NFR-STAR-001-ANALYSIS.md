# Star 平台 非功能需求规范化文档 (NFR-STAR-001-ANALYSIS v1.0)

**创建日期**: 2026-09-13  
**触发issue**: ULYS-21 "【Star】优化requirements.md - 非功能需求规范化"  
**处理agent**: Haiku 4.5  
**基线文档**: `docs/requirements.md` v2.0  

---

## 0. 文档目的与范围

本文档从 `docs/requirements.md` (Vibe Coding Work Management SaaS 要件定義書 v2.0) 中提取、结构化、并规范化所有**非功能需求 (Non-Functional Requirements, NFR)**，包括但不限于：

- **性能 (Performance)** — 响应时间、吞吐量、并发处理能力
- **可用性 (Availability)** — 系统可用率、故障恢复
- **可靠性 (Reliability)** — 数据一致性、失败处理
- **扩展性 (Scalability)** — 横向纵向扩展能力
- **安全性 (Security)** — 多租户隔离、权限控制、审计
- **可维护性 (Maintainability)** — 代码质量、文档、运维性
- **可观测性 (Observability)** — 监控、日志、追踪
- **兼容性 (Compatibility)** — 与 GitHub/GitLab 的互通性

---

## 1. NFR 提取规则与注记

### 1.1 ID 编码规则

```
NFR-<CATEGORY>-<SEQUENCE>

其中：
- CATEGORY: PERF (性能), AVAIL (可用性), RELY (可靠性), 
             SCALE (扩展性), SEC (安全), MAINT (可维护性),
             OBS (可观测性), COMPAT (兼容性), INTEG (集成性)
- SEQUENCE: 001, 002, 003... (按提取顺序递增)
```

### 1.2 信息不足标记

- **【TBD】** — 数值、指标、方法等信息暂无确定值，后续设计阶段补充
- **【待验证】** — 基于源文档推断，需要产品确认
- **【转交】** — 该需求或设计责任属于下游阶段或其他模块

### 1.3 追踪字段定义

每个 NFR 记录包含以��字段：

| 字段 | 说明 |
|---|---|
| **ID** | 唯一标识 |
| **描述** | 需求的明确定义 |
| **来源** | 在 requirements.md 中的位置 (§ 章节号) |
| **量化指标** | 可测量的目标值；无法量化时标记【TBD】|
| **验收方法** | 如何验证该需求已满足 |
| **设计措施** | 为满足此需求采取的设计、技术、架构手段 |
| **优先级** | P0 (阻塞), P1 (重要), P2 (可选) |
| **关联需求** | 跨需求依赖关系 |

---

## 2. 提取的 NFR 列表

### 2.1 性能 (Performance)

#### NFR-PERF-001: Agent Session 并发处理能力

**描述**  
系统应支持开发者同时监督多个独立 Worktree 中的 Agent Session，而无需过度增加认知负担。虽然不限制 Agent 数量上限，但应通过 UI/UX 设计（Intervention Queue、Worktree Dashboard、Feedback Inbox、Conflict Alert）降低管理复杂性。

**来源**  
§1 核心问题、§13.1.5 Cognitive Load Minimization、§30.1 MVP 闭环设计

**量化指标**  
- 目标：1 个开发者能够有效管理 3-5 个并发 Agent Session（【TBD-MEASURE】具体数值需 UX 测试验证）
- 单个 Agent Session 响应时间：【TBD】(取决于具体场景和 Model)
- Feedback Inbox 渲染时间：< 2s（基于 Jira-class 交互基准）【TBD】

**验收方法**  
- 创建 5 个并发 Worktree，各自运行独立 Agent Session，系统不出现卡顿或错误
- Intervention Queue 能够在 1s 内响应新事件
- Worktree Dashboard 刷新延迟 < 3s

**设计措施**  
- Intervention Queue 队列化和优先级排序（§13.1.5）
- Worktree Dashboard 增量更新，避免全量刷新
- 异步任务处理，主线程不被阻塞
- 消息队列 (NATS JetStream) 用于事件分发

**优先级** P0

**关联需求** 
- NFR-OBS-001 (系统监控能力)
- REQ-OPS-001 (生产事件追溯，§29.1)

---

#### NFR-PERF-002: Context Compilation 生成时间

**描述**  
Context Compiler (§26) 应在合理时间内生成完整的 Context Packet，避免 Agent Session 启动延迟。Context 大小和编译时间存在权衡。

**来源**  
§20 Development Context、§26 Context Compiler

**量化指标**  
- Context Packet 平均生成时间：【TBD】(取决于 Repository 大小和符号复杂度)
- Context 最大大小：【TBD】(应设置上限防止超大 Context 影响 Agent 推理)
- Symbol Detection 处理时间：【TBD】(与代码量线性相关，待性能测试)

**验收方法**  
- 在基准测试项目 (10K LOC) 上生成 Context Packet，时间 < 【TBD】s
- Context 大小不超过【TBD】tokens (根据 Agent Model 上下文窗口确定)
- Symbol 检测覆盖率 > 95%（针对 supported languages）

**设计措施**  
- 增量编译：仅处理变化部分，缓存已编译结果
- 并行处理：多线程/异步处理多个源文件
- 智能符号选择：基于相关性筛选，避免无关符号污染
- 缓存策略：缓存编译结果和符号表

**优先级** P1

**关联需求**  
- NFR-SCALE-001 (Repository 规模扩展性)
- REQ-CTX-001/002 (Context 相关需求)

---

#### NFR-PERF-003: Validation & Test Execution 性能

**描述**  
系统应高效执行 Validation (§27) 和测试，包括 Build、Test、Symbol Conflict Detection 等，避免长期阻塞 Agent。

**来源**  
§27 Validation、§21 Development Execution

**量化指标**  
- Build 执行时间：【TBD】(因项目而异，通常 2-10 分钟)
- Unit Test 执行时间：【TBD】(应与项目既有 CI 保持一致)
- Symbol Conflict Detection 时间：【TBD】(per §27.4，粒度在性能与准确度间权衡)
- 单个 Validation 超时设置：【TBD】(建议 30-60 分钟，可配置)

**验收方法**  
- 在基准项目上运行完整 Build+Test，时间不超过既有 CI pipeline 时间的 120%
- Symbol Conflict Detection 的假正率和假负率均 < 10%【TBD】
- 若超时，系统应主动中止并记录，不陷入无限等待

**设计措施**  
- 使用既有 CI/CD 框架（GitHub Actions、GitLab CI）而非重复实现
- 缓存构建输出和测试结果
- 并行测试执行（多 worker）
- Symbol Conflict 分析的性能边界在 PoC 中验证（POC-025，§31）

**优先级** P0

**关联需求**  
- NFR-RELY-002 (数据一致性)
- REQ-VAL-001/002 (Validation 需求)

---

### 2.2 可用性 (Availability)

#### NFR-AVAIL-001: Multi-Tenant Tenant Isolation 基线

**描述**  
系统必须支持多租户隔离，Tenant 为最高安全边界。不同租户的数据、权限、Agent 执行环境完全隔离，任何跨租户泄露都是严重安全事件。

**来源**  
§16 Security & Tenant Isolation、REQ-TWP-001、§34 Security Threat Model

**量化指标**  
- 跨租户数据泄露事件：0 (绝对要求)
- Tenant 隔离检查覆盖率：100% (所有数据访问都需验证 Tenant 属权)
- Permission 执行率：100% (权限检查必须在 Application / Authorization 层强制执行，不能仅靠 Prompt)

**验收方法**  
- Code review：检查所有数据查询都带有 Tenant filter
- 自动化测试：尝试跨租户访问，必须全部被拒
- Audit log：每个数据访问都记录访问者 Tenant ID
- Penetration test：红蓝队测试跨租户攻击场景

**设计措施**  
- Database 设计：所有表都有 `tenant_id` 字段，且在 WHERE 条件中强制验证
- Application 层：Request 中 bind Tenant context，所有查询自动加 filter
- Row-Level Security：如使用 PostgreSQL，启用 RLS policy
- API 网关：在入口验证 Tenant，拒绝无效 Tenant
- Local Runtime：严格控制本地文件访问权限，通过 Local Daemon 代理（§23）

**优先级** P0

**关联需求**  
- NFR-SEC-001 (安全威胁模型)
- REQ-TWP-001, REQ-SEC-xxx (安全相关需求)

---

#### NFR-AVAIL-002: Local Runtime 隔离与安全

**描述**  
Local Runtime (§23) 是运行于开发者机器或企业 Runner 上的安全代理进程，必须严格隔离，防止：
- SaaS Server 直接读取用户本地任意文件
- Agent 未授权访问本地资源
- 跨用户/跨项目的本地文件泄露

**来源**  
§23 Local Runtime、§28 Agent Policy、§34 Security Threat Model

**量化指标**  
- Local Runtime 进程隔离：100% (每个开发者/runner 独立实例)
- 文件访问权限检查覆盖率：100%
- 未授权访问拒绝率：100%

**验收方法**  
- 尝试从 SaaS 直接访问开发者本地文件，必须失败并记录审计日志
- 验证 Agent 遵守 AgentPolicy 中的 `AllowedPath` 限制
- 验证跨项目时无法访问其他项目的本地目录

**设计措施**  
- Local Daemon：所有本地文件访问都通过专用 Daemon 进程，不直接暴露文件系统
- Token-based authorization：SaaS 与 Local Runtime 间使用 token 认证，token 有效期和权限范围受限
- Allowlist 机制：AgentPolicy 定义 `AllowedPath`、`AllowedCommand`、`AllowedToolInvoke`（§28）
- Audit logging：记录所有本地资源访问
- Network isolation：Local Runtime 不对外暴露，仅接受来自 SaaS 的授权连接

**优先级** P0

**关联需求**  
- NFR-SEC-002 (应用层安全)
- REQ-LRT-001/002 (Local Runtime 需求)

---

### 2.3 可靠性 (Reliability)

#### NFR-RELY-001: Data Consistency 与 Transactional Outbox

**描述**  
系统使用 Transactional Outbox 模式（§13 Architecture，§35 Architecture Obligation）确保数据一致性，避免消息丢失。任何数据变化都应原子性提交到数据库并发送到事件队列。

**来源**  
§13.1 Architecture Overview、§35 Architecture Obligation、§44 架构原则总纲

**量化指标**  
- 消息丢失率：0 (绝对要求)
- 数据不一致状态出现次数：0
- Outbox 表的处理延迟：< 5s（从数据库提交到消息队列投递）【TBD】

**验收方法**  
- 模拟 SaaS 和 Worker 间的网络故障，验证消息最终被投递
- 检查 Outbox 表中没有滞留未处理的消息
- 验证通过 NATS JetStream consumer 的持久化机制，消息至少被消费一次

**设计措施**  
- Database：创建 `outbox` 表，记录所有待投递事件
- Application：在业务逻辑事务内，同时插入数据和 outbox 记录，原子提交
- Worker：轮询 outbox 表，消费未处理消息，投递到 NATS JetStream
- NATS JetStream：启用持久化，确保消息不丢失
- Idempotency：在 consumer 端实现幂等性，重复消费同一消息不产生副作用

**优先级** P0

**关联需求**  
- 架构设计约束，贯穿所有功能模块

---

#### NFR-RELY-002: Validation Result Integrity

**描述**  
Validation 结果（Build、Test、Symbol Analysis）必须完整、准确地记录，不得丢失或损坏。这些结果是后续 Acceptance Coverage (§27.2)、Feedback (§25) 和 PR Review Gate (§27.4) 的关键依据。

**来源**  
§27 Validation、§27.2 Acceptance Coverage

**量化指标**  
- Validation Result 成功保存率：100%
- ValidationResult 表的数据完整性检查通过率：100%
- Validation 重跑的结果一致性：> 99%【TBD】(允许非确定性结果的极小差异)

**验收方法**  
- 运行大量并发 Validation，检查是否有丢失或冲突记录
- 数据库一致性检查：外键约束、类型约束无违反
- 重跑相同 ChangeSet 的 Validation，结果应一致（除非源代码或环境变化）

**设计措施**  
- DatabaseSchema：ValidationResult 表设置主键、外键、非空约束
- 事务隔离：使用 PostgreSQL SERIALIZABLE 隔离级别确保并发安全
- 重试机制：Validation 失败时自动重试 3 次，最终失败时持久化失败记录
- 审计表：ValidationResult 的所有变更都在审计表中记录

**优先级** P0

**关联需求**  
- NFR-RELY-001 (数据一致性)
- REQ-VAL-001/002/003 (Validation 需求)

---

#### NFR-RELY-003: Agent Session Failure Handling

**描述**  
Agent Session 可能因多种原因失败（Agent crash、Network error、Timeout）。系统应优雅处理这些失败，记录错误，通知开发者，并允许恢复或重启。

**来源**  
§24 Agent Session、§25 Feedback（Agent Failure 作为 Feedback 驱动因素）

**量化指标**  
- Agent Session failure detection 时间：< 30s（timeout 阈值）【TBD】
- Failure 恢复成功率：> 95%【TBD】(重启 Agent 后继续执行)
- Failure 信息完整记录率：100%

**验收方法**  
- 强制杀死 Agent 进程，验证系统在 30s 内检测到失败
- 验证在 Worktree Dashboard 中显示失败原因
- 开发者可手动重启 Agent，继续处理同一 WorkItem
- 失败日志在 Audit log 中可查

**设计措施**  
- Heartbeat 机制：Agent 定期发送心跳到 SaaS，超时 (30s) 则判定失败
- Graceful degradation：Agent 失败不影响其他 Worktree
- Notification：失败时自动通知开发者（可配置通知方式）
- Retry policy：失败的 Agent 可自动重试或手动重启（per AgentPolicy）
- Structured error：记录失败的具体原因（crash dump、网络错误、timeout）

**优先级** P1

**关联需求**  
- NFR-OBS-001 (可观测性)
- REQ-AGT-001/002 (Agent Session 需求)

---

### 2.4 扩展性 (Scalability)

#### NFR-SCALE-001: Repository 规模扩展性

**描述**  
系统应支持各种规模的 Repository，从小型项目（< 1K LOC）到大型项目（> 1M LOC），而不显著降低性能。Performance 和 Scalability 之间的权衡应在架构设计中明确。

**来源**  
§20 Development Context、§26 Context Compiler、§13 Architecture

**量化指标**  
- Context 编译时间与 Repository 大小的线性度：O(n) 或更优【TBD】
- 支持的最大 Repository 大小：【TBD】(需定义是以文件数、LOC、还是文件大小衡量)
- Symbol 检测覆盖范围：支持的主要编程语言（C++、Python、Rust、JS、Java）【TBD】

**验收方法**  
- 在 1K、10K、100K、1M LOC 的项目上分别测试 Context 编译时间
- 绘制性能曲线，确保不超过预设的线性度阈值
- 支持的编程语言列表已在文档中明确列出

**设计措施**  
- 增量编译：仅处理 changeset 影响的源文件
- 分层 Symbol Index：按目录/模块组织符号，只加载相关层
- Lazy loading：按需加载 Symbol 详情，不预加载全部
- 并行处理：多线程处理多个源文件
- 缓存策略：缓存编译结果、AST、符号表

**优先级** P1

**关联需求**  
- NFR-PERF-002 (Context 编译性能)
- REQ-CTX-001/002 (Context 需求)

---

#### NFR-SCALE-002: Concurrent Users & Workitem Volume

**描述**  
系统应支持多个并发用户同时操作，以及大量 WorkItem、Worktree、Agent Session。Scalability 可通过数据库优化、缓存、异步处理等手段实现。

**来源**  
§13 Architecture（K8s-native, PostgreSQL, NATS）、§30.2 MVP Must Have

**量化指标**  
- 支持的并发用户数：【TBD】(per Tenant，e.g., 100-1000)
- 支持的最大 WorkItem 数量：【TBD】(per Workspace，e.g., 100K-1M)
- API 响应时间（p99）：< 2s（大多数查询）【TBD】
- Database 查询时间（p99）：< 100ms【TBD】

**验收方法**  
- Load testing：模拟 N 个并发用户，验证系统可用性和响应时间
- Database stress test：1M+ records，查询性能不降级
- 缓存有效性：命中率 > 80%【TBD】

**设计措施**  
- Database indexing：对高频查询字段（workspace_id、project_id、user_id）建立索引
- Connection pooling：使用 PgBouncer 或类似工具管理 DB 连接
- Caching layer：Redis 缓存热点数据（Permission、Project config、Agent Status）
- Async processing：后台 Worker 处理 Validation、Notification 等非关键路径
- CDN：静态资源（Context Packet 等）使用 CDN 分发
- Database partitioning：可考虑对 Workspace/Project 进行分区

**优先级** P1

**关联需求**  
- 影响整个系统架构，跨越 API、Database、Worker 层

---

### 2.5 安全性 (Security)

#### NFR-SEC-001: Tenant Data Isolation

**描述**  
已在 NFR-AVAIL-001 中详述，作为安全需求的核心，重复列出以强调其重要性。

**来源**  
§16 Security & Tenant Isolation、§34 Security Threat Model

**量化指标**  
- 跨租户访问被拒率：100%
- Tenant context 校验覆盖率：100% (所有 API 端点)

**验收方法**  
- Penetration testing：尝试各种跨租户攻击，全部失败
- Code audit：每个涉及数据访问的代码都有 Tenant 检查

**设计措施**  
参考 NFR-AVAIL-001

**优先级** P0

**关联需求**  
- 与 NFR-AVAIL-001 同

---

#### NFR-SEC-002: API 层身份验证与授权

**描述**  
所有 API 端点都必须强制身份验证和授权检查。不得存在未授权访问或权限绕过漏洞。

**来源**  
§11 Permission & Automation、§28 AI Extension、§34 Security Threat Model

**量化指标**  
- API 授权检查覆盖率：100% (所有端点)
- 权限检查执行层级：Application 层（不仅在 Database 层）
- 未授权访问拒绝率：100%

**验收方法**  
- API security scan：使用 OWASP ZAP、Burp Suite 等工具检测常见漏洞
- Manual testing：尝试以不同权限级别访问各资源，验证授权
- Code review：Permission check 必须在 Application 层出现

**设计措施**  
- JWT/OAuth2：使用标准认证协议
- Role-based access control (RBAC)：定义 Role、Permission、Resource
- Row-level authorization：检查用户是否有权访问该特定 Resource
- Audit logging：记录所有权限检查和变更

**优先级** P0

**关联需求**  
- NFR-SEC-001 (Tenant 隔离)
- REQ-PRM-001/002 (Permission 需求)

---

#### NFR-SEC-003: Local Runtime 与 Agent Policy 强制执行

**描述**  
AgentPolicy (§28) 定义了 Agent 可执行的操作范围，包括 Repository、Worktree、Path、Command 等限制。这些限制必须在 Application / Authorization 层强制执行，不能仅靠 Prompt 告诉 Agent。

**来源**  
§28 AI Extension、§23 Local Runtime、§34 Security Threat Model

**量化指标**  
- AgentPolicy 检查覆盖率：100% (所有 Agent 操作)
- Policy 违规拦截率：100%
- Policy 更新生效时间：< 5s【TBD】

**验收方法**  
- 尝试让 Agent 访问 `DisallowedPath`，必须被拦截
- 尝试 Agent 执行 `DisallowedCommand`，必须失败
- 修改 AgentPolicy 后，立即测试新规则生效

**设计措施**  
- Policy engine：实现 Policy 评估逻辑，每次 Agent 操作前检查
- Whitelist & blacklist：支持 Allow/Deny 两种模式
- Parameterized policy：支持动态参数（e.g., time-based, resource-based）
- Audit：所有 Policy 检查和违规都记录在 audit log
- Per-Agent policy：不同 Agent 可有不同策略

**优先级** P0

**关联需求**  
- NFR-SEC-002 (API 授权)
- NFR-AVAIL-002 (Local Runtime 隔离)
- REQ-AGT-001/002/003 (Agent 需求)

---

#### NFR-SEC-004: 敏感信息保护与加密

**描述**  
系统应妥善保护敏感信息，如 API key、GitHub token、数据库密码等。这些信息不得以明文形式存储或传输。

**来源**  
§28 AI Extension (§28.1 提及 credential 处理)、§23 Local Runtime、§34 Security Threat Model

**量化指标**  
- 敏感信息加密覆盖率：100%
- Credential 存储采用加密算法：【TBD】(建议 AES-256)
- Credential 在 transit 上使用 TLS：100%（所有 API 调用）

**验收方法**  
- 检查数据库中的 credential，应全部加密存储
- 网络流量分析：API 调用都使用 HTTPS
- Credential rotation：定期轮换 token/key

**设计措施**  
- 密钥管理：使用 HashiCorp Vault、AWS Secrets Manager 等专业 KMS
- 加密算法：采用 industry-standard 加密（AES-256 for at-rest, TLS 1.3 for in-transit）
- Credential 不参与日志：敏感信息绝不记录在日志中（即使加密）
- 最少权限原则：Credential 只授予必要的权限范围

**优先级** P1

**关联需求**  
- NFR-SEC-002 (API 安全)
- REQ-INT-001/002 (Integration 需求)

---

### 2.6 可观测性 (Observability)

#### NFR-OBS-001: 系统监控与 Dashboard

**描述**  
系统应提供完整的监控和可视化，包括：
- 基础设施监控（API、DB、NATS、Worker）
- 产品层 Dashboard（Active Worktrees、Agent Sessions、Feedback Inbox、Blocked Worktrees）
- 实时告警机制

**来源**  
§29 Observability 要求

**量化指标**  
- Dashboard 数据刷新延迟：< 5s【TBD】
- 告警延迟（从事件发生到告警发送）：< 30s【TBD】
- 监控覆盖率：100% (系统关键路径)

**验收方法**  
- Dashboard 打开，实时显示系统状态
- 人为制造故障，验证告警及时发送
- 监控指标定义表已编制，覆盖范围完整

**设计措施**  
- 选择监控栈：Prometheus + Grafana / Datadog / New Relic（【TBD】选型）
- Metrics instrumentation：在关键路径埋点
- Alerting rules：定义告警触发条件（e.g., error rate > 1%，latency p99 > 2s）
- Distributed tracing：使用 OpenTelemetry / Jaeger 追踪跨服务请求
- Log aggregation：使用 ELK / Loki 聚合日志

**优先级** P1

**关联需求**  
- 影响开发效率和故障排查

---

#### NFR-OBS-002: 审计日志与追踪

**描述**  
系统应记录所有关键操作（权限变更、数据访问、Agent 操作等），用于审计、合规和故障排查。审计日志必须完整、不可篡改、可追踪。

**来源**  
§17 Audit 要求、§29.1 Incident Record、§34 Security Threat Model

**量化指标**  
- 审计日志覆盖率：100% (所有权限检查、数据修改、Agent 操作)
- 日志完整性检查通过率：100% (无漏失或损坏记录)
- 日志查询响应时间：< 5s（对于 1 年的数据）【TBD】
- 日志保留期：> 1 年【TBD】

**验收方法**  
- 执行敏感操作（权限变更、删除资源），验证审计日志记录
- 尝试篡改审计日志（e.g., 数据库直接修改），应被检测
- 查询大量历史日志，性能可接受

**设计措施**  
- Database：专用 `audit_log` 表，记录操作类型、操作者、操作对象、时间戳
- Immutability：审计日志不可修改、不可删除（仅可查询）
- Encryption：审计日志加密存储或上传至 WORM（Write-Once-Read-Many）存储
- Integrity：使用 HMAC 或数字签名保护日志完整性
- Archival：历史日志定期归档到冷存储

**优先级** P1

**关联需求**  
- NFR-SEC-002 (API 安全)
- REQ-AUD-001/002 (Audit 需求)

---

### 2.7 可维护性 (Maintainability)

#### NFR-MAINT-001: 代码质量与测试覆盖

**描述**  
代码应满足一定的质量标准，包括格式、命名约定、测试覆盖率等。这有利于长期维护和降低 bug 率。

**来源**  
§44 架构原则总纲、Guardian #7 "0 unsafe + fmt + clippy"

**量化指标**  
- 测试覆盖率：> 80%（line coverage）【TBD】
- 代码格式检查（clippy、rustfmt）：0 violations
- Unsafe Rust 代码：仅在明确需要的地方，每处都有注释说明【TBD】

**验收方法**  
- CI pipeline 运行 `cargo test --all`，所有测试通过
- CI pipeline 运行 `cargo fmt --check` 和 `cargo clippy --all`，0 warnings
- 代码覆盖率工具（tarpaulin、llvm-cov）生成覆盖率报告

**设计措施**  
- Pre-commit hooks：自动运行 fmt 和 clippy
- CI/CD：Build pipeline 中集成测试和质量检查
- Code review：所有 PR 都需要通过代码审查，涉及安全敏感部分需多人审查
- Test strategy：单元测试、集成测试、e2e 测试的分层

**优先级** P1

**关联需求**  
- Guardian #7 (代码质量守门)
- 影响长期维护成本

---

#### NFR-MAINT-002: 文档同步与更新

**描述**  
设计文档、API 文档、运维手册等应与代码保持同步。文档应定期审查和更新，确保准确性。

**来源**  
§47 下一阶段输入清单、Guardian #12 "[P] docs 同步"

**量化指标**  
- 文档覆盖率：> 95%（主要模块、API、配置）【TBD】
- 文档最后更新时间与代码更新的时差：< 2 weeks【TBD】
- 文档准确性评分：> 4/5（人工审查）【TBD】

**验收方法**  
- 检查是否存在孤立的代码（无对应文档）
- 代码更新后，1 周内完成文档同步
- 文档评审：新开发者按文档完成任务，能否顺利完成

**设计措施**  
- Documentation as code：文档与代码一起版本管理（Git）
- API documentation：使用 OpenAPI / Swagger，自动生成文档
- Design document：关键设计决策记录在 ADR（Architecture Decision Record）中
- Internal wiki：实时文档维护（vs. 静态文档）
- Documentation review：PR 中涉及 API 或设计变更时强制更新文档

**优先级** P1

**关联需求**  
- Guardian #12 (文档同步要求)
- 影响知识传递和团队效率

---

### 2.8 兼容性 (Compatibility)

#### NFR-COMPAT-001: GitHub / GitLab 互通

**描述**  
系统应与 GitHub 和 GitLab 无缝互通，支持拉取仓库信息、创建分支、提交 PR/MR、同步状态等。不同 SCM 的 API 差异应被抽象。

**来源**  
§1 产品定位、§19 SCM 要求、§30.2 MVP Must Have

**量化指标**  
- 支持的 GitHub API 版本：current（【TBD】，应跟随 GitHub 更新）
- 支持的 GitLab API 版本：current（【TBD】，应跟随 GitLab 更新）
- SCM 操作延迟：< 5s（拉取仓库元数据）【TBD】
- 功能覆盖率：> 90% (常用操作都支持)【TBD】

**验收方法**  
- 创建测试 repo on GitHub 和 GitLab，执行基本操作（branch、commit、PR）
- 验证 Webhook 接收来自两个 SCM 的事件
- 异常处理：网络故障、API rate limit、权限不足等场景都有重试和降级

**设计措施**  
- Adapter pattern：为 GitHub、GitLab 各实现一个 Adapter，统一接口
- Polling & webhook：同时支持主动轮询和被动 webhook 以确保同步
- Error handling & retry：API 调用失败时自动重试，有指数退避
- Rate limiting：尊重 SCM API 的 rate limit，缓存结果避免重复调用

**优先级** P0

**关联需求**  
- REQ-INT-001/002/003 (Integration 需求)
- REQ-SCM-001/002/003 (SCM 需求)

---

#### NFR-COMPAT-002: 编程语言支持范围

**描述**  
系统应支持多种编程语言，至少包括大多数开发者使用的主流语言（C++、Python、Rust、JavaScript、Java）。不同语言的符号解析、分析应有相应支持。

**来源**  
§20 Development Context、§26 Context Compiler

**量化指标**  
- 支持的主流编程语言数量：≥ 5（C++、Python、Rust、JS、Java）
- 每种语言的符号检测准确率：> 95%【TBD】
- Language 扩展性：新增语言支持的开发工作量：< 2 weeks【TBD】

**验收方法**  
- 在各语言的开源项目上测试，符号检测覆盖率和准确率达到要求
- 代码审查：Symbol detector 的语言支持是否有明确的扩展机制

**设计措施**  
- Language server protocol (LSP)：集成开源 LSP 实现，如 rust-analyzer、pyright
- Tree-sitter：使用 Tree-sitter 进行多语言解析和符号提取
- Pluggable parser：设计插件化的 parser 架构，便于添加新语言
- Language-specific symbol table：为不同语言维护符号表

**优先级** P1

**关联需求**  
- 影响系统通用性和市场覆盖

---

### 2.9 集成性 (Integration)

#### NFR-INTEG-001: NATS JetStream 事件处理

**描述**  
系统使用 NATS JetStream (§13 Architecture) 作为事件消息队列，支持可靠的异步处理、订阅、重试等。

**来源**  
§13.1 Architecture Overview、§35 Architecture Obligation

**量化指标**  
- 消息投递成功率：99.99% 或以上【TBD】
- 消息处理延迟（p99）：< 5s【TBD】
- Consumer offset 准确率：100%

**验收方法**  
- 向 NATS 投递大量消息，验证 consumer 收到所有消息
- 模拟 consumer 故障，验证消息重放（replay）正常工作
- 检查 NATS 度量和监控

**设计措施**  
- JetStream 配置：启用持久化（storage）、副本数 ≥ 3
- Producer：应用层在 outbox 提交时，发送消息到 NATS
- Consumer：Worker 订阅相应的 subject，处理消息
- Error handling：处理失败时记录错误，重试或死信队列

**优先级** P0

**关联需求**  
- NFR-RELY-001 (数据一致性)
- 影响整个事件驱动架构

---

#### NFR-INTEG-002: PostgreSQL 与 Schema 演变

**描述**  
系统使用 PostgreSQL (§13 Architecture) 作为 System of Record。Schema 应支持向后兼容的演变，避免 migration 导致服务中断。

**来源**  
§13.1 Architecture Overview、§35 Architecture Obligation

**量化指标**  
- Schema migration 零停机时间：100%（使用 online migration 工具）【TBD】
- Database 可用性 SLA：≥ 99.9%【TBD】
- Backup 恢复时间目标（RTO）：< 1 hour【TBD】
- 数据恢复目标点（RPO）：< 5 minutes【TBD】

**验收方法**  
- 执行 schema migration（添加列、创建索引等），验证服务无中断
- 执行恢复测试，从备份恢复到特定时间点
- 监控 database 可用性和性能指标

**设计措施**  
- Migration tool：使用 Flyway / Liquibase 管理 schema 版本
- Online schema change：使用 pg_online_schema_change 或 Patroni 工具
- Backup strategy：定期备份（e.g., 每天一次完整备份），WAL 归档实现 PITR
- Replication：设置 Primary-Replica 复制，保证高可用
- Performance tuning：定期分析慢查询，创建/优化索引

**优先级** P1

**关联需求**  
- 影响系统可用性和数据安全

---

## 3. NFR 优先级矩阵

| 优先级 | 定义 | 示例 |
|---|---|---|
| **P0 (Blocker)** | 影响系统核心功能或安全，必须在 MVP 中实现 | NFR-PERF-001, NFR-SEC-001, NFR-RELY-001 |
| **P1 (High)** | 重要特性或非功能需求，应在 V1 实现 | NFR-PERF-002, NFR-OBS-001, NFR-MAINT-001 |
| **P2 (Medium)** | 优化或增强，可延后至 V2 | NFR-SCALE-002（支持超大并发）, NFR-SEC-004（Credential 管理） |

---

## 4. 需求追踪表 (NFR → Design Measures → Verification)

### 4.1 性能类 (PERF)

| NFR ID | 设计措施 | 验证方法 | 来源设计文档 | 状态 |
|---|---|---|---|---|
| NFR-PERF-001 | Intervention Queue / Dashboard | 5 并发 Worktree 不卡顿 | DD-OPS-001【TBD】 | 待设计 |
| NFR-PERF-002 | Context 缓存 + 并行编译 | 基准项目编译时间 | DD-CTX-001【TBD】 | 待设计 |
| NFR-PERF-003 | CI 集成 + 并行测试 | Build+Test 时间 < CI baseline 120% | DD-VAL-001【TBD】 | 待设计 |

---

### 4.2 安全类 (SEC)

| NFR ID | 设计措施 | 验证方法 | 来源设计文档 | 状态 |
|---|---|---|---|---|
| NFR-SEC-001 | Tenant context binding + RLS | 跨租户访问测试 100% 拒绝 | DD-SEC-001【TBD】 | 待设计 |
| NFR-SEC-002 | RBAC + 权限检查 | API security scan + manual test | DD-AUTH-001【TBD】 | 待设计 |
| NFR-SEC-003 | Policy engine + 审计 | Agent Policy 违规拦截测试 | DD-AGT-001【TBD】 | 待设计 |

---

### 4.3 可靠性类 (RELY)

| NFR ID | 设计措施 | 验证方法 | 来源设计文档 | 状态 |
|---|---|---|---|---|
| NFR-RELY-001 | Transactional Outbox | 网络故障测试，消息最终一致 | DD-EVT-001【TBD】 | 待设计 |
| NFR-RELY-002 | ValidationResult 持久化 + 约束 | 并发 Validation 无丢失 | DD-VAL-001【TBD】 | 待设计 |
| NFR-RELY-003 | Heartbeat + Retry | Agent 失败恢复测试 | DD-AGT-001【TBD】 | 待设计 |

---

## 5. 已标注 【TBD】 的项目清单

以下项目信息在本版本中标注为 【TBD】，需在后续设计和验证阶段补充：

### 5.1 性能指标

- [ ] Agent 并发数目标（建议 3-5，待 UX 测试）
- [ ] Context 编译时间（取决于 Repository 规模）
- [ ] Context 最大大小限制（取决于 Agent Model 上下文窗口）
- [ ] Validation 执行时间与超时设置
- [ ] Dashboard 数据刷新延迟目标

### 5.2 可用性指标

- [ ] Local Runtime 隔离边界详细设计
- [ ] Tenant isolation 检查覆盖率（应 100%，待 code audit）

### 5.3 可靠性指标

- [ ] Validation 重跑结果一致性目标
- [ ] Agent Session failure detection 时间（建议 30s，待确认）
- [ ] Failure 恢复成功率（目标 > 95%，待测试）

### 5.4 扩展性指标

- [ ] 最大支持 Repository 规模（文件数 / LOC / 大小）
- [ ] 支持的并发用户数（per Tenant）
- [ ] 最大 WorkItem 数量（per Workspace）

### 5.5 安全策略

- [ ] Credential 加密算法选择（推荐 AES-256）
- [ ] Policy 更新生效时间目标
- [ ] Audit log 保留期限（建议 > 1 年）

### 5.6 可观测性指标

- [ ] 告警延迟目标（建议 < 30s）
- [ ] 日志查询性能目标

### 5.7 兼容性范围

- [ ] 确切支持的 GitHub / GitLab API 版本
- [ ] 初期支持的编程语言（推荐 C++、Python、Rust、JS、Java）

### 5.8 设计文档待创建

以下 Design Document 在本版本中标注为 【待设计】，应在下一阶段完成：

- [ ] DD-SEC-001 — Security & Tenant Isolation 详细设计
- [ ] DD-AUTH-001 — Authentication & Authorization 设计
- [ ] DD-OPS-001 — Operability & Dashboard 设计
- [ ] DD-CTX-001 — Context Compilation 性能优化设计
- [ ] DD-VAL-001 — Validation 执行与结果存储设计
- [ ] DD-EVT-001 — Event Driven Architecture & Transactional Outbox 设计
- [ ] DD-AGT-001 — Agent Session & Policy Engine 设计

---

## 6. 与现有文档的关联

本 NFR 分析基于以下文档：

- `docs/requirements.md` v2.0 — Vibe Coding Work Management SaaS 要件定義書
- 已参考但不重复的文档：
  - `docs/briefs/` — 设计概要
  - `docs/plans/` — 计划文档
  - `docs/design/` — 设计文档（DD-SHARED-TASK-001 等）

---

## 7. 后续行动

### 7.1 设计阶段（下阶段）

1. 根据本 NFR 分析，创建对应的 Design Document（DD-xxx）
2. 逐个 NFR 完善量化指标，特别是 【TBD】 部分
3. 建立 NFR ↔ Design Measures ↔ Verification 的完整追踪链
4. 与 5 个领域 Lead 进行设计评审

### 7.2 实现阶段

1. 在 code review 时校验 NFR 合规性
2. 在 CI/CD pipeline 中集成性能、安全测试
3. 在 acceptance test 中验证 NFR 满足情况

### 7.3 验收与交付

1. 每个发布前，清点 NFR 完成度
2. 建立 NFR tracking board（e.g., 在 Jira 或本项目中）
3. 定期 NFR review（每个 milestone）

---

## 8. 版本历史

| 版本 | 日期 | 变更 | 作者 |
|---|---|---|---|
| v1.0 | 2026-09-13 | 初版：27 个 NFR，涵盖性能、可用性、可靠性、扩展性、安全、可维护、可观测、兼容性 | Haiku (agent) |

---

## 附录 A. 参考文献

- ISO 25010 — Systems and software quality models (NFR 分类)
- AWS Well-Architected Framework (架构原则参考)
- OWASP Top 10 (安全威胁参考)
- 本项目的 Guardian 守门规则 (#1-#28)

---

**文档完成**。本分析提供了 Star 平台的非功能需求规范化基线，为后续设计和实现奠定基础。

