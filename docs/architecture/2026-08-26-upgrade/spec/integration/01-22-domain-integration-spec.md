# Spec-01: 22 domain crate 真实数据接入规范

> **状态**：Draft v0.2
> **日期**：2026-09-01
> **修订人**：架构师 (Mavis 接手 agent per DEC-008)
> **触发**：per [ADR-0036 §8.2 Phase H 方向](../../adr/0036-phase-g-architecture.md) / 2026-08-27 21:59 JST 用户授权第三次强化 / 2026-09-01 14:38 JST 模块间协作细化任务

> **dual-use 警告 (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板)**：
> 本 spec 沿用 5 域（Permission/Worktree/Flow/Integration/Agent/Admin）作为"5 位真人 Lead 问责结构"的历史治理命名，但**不**作为 22 domain crate 的业务子域映射。
> - 5 域是 5 位真人 Lead 责任分工（per 8/21 JST 用户偏好"不接受兼任"）
> - 22 domain crate 是 DDD bounded context（per [spec/agents/02 §2](../agents/02-data-sources-spec.md) 22 domain 数据源清单）
> - 二者**非同一分类**，本 spec §2 Tier 表"5 域映射"列**仅作历史命名兼容性 footnote**，实际代码层归属通过 [spec/saga/01 v0.2 §2](../saga/01-saga-coordination-spec.md) `SagaStep.responsible_crate: &str` 字段显式声明
> - Lead 兼任约束（架构师不得兼任意 5 域 Lead / SRE 不得兼 Admin Lead / 同一 Lead 不得签 2 域）**仅适用于 5 域历史治理命名**，22 domain crate 各自 lead 待 DDD Review 阶段补

## §1 目的

定义 Star 22 domain crate（per [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md)）真实数据接入 Phase H 完整规范：接入顺序、验收标准、测试要求、风险控制、Saga 触发点。Phase H 是 Phase F mock → 真实的最后一公里（per [ADR-0035 §7 #10 L247](../../adr/0035-phase-f-architecture.md) "22 domain 接入优先级排期" 已知缺口 + [ADR-0036 §7 #5 L272](../../adr/0036-phase-g-architecture.md) PM 拍板事项）。

**Phase H 范围对齐**（per [ADR-0036 §8.2 L293-298](../../adr/0036-phase-g-architecture.md)）：
- 缓存命中率优化（InMemory 调优 + 预热 + 22 domain 真实数据接入完整化）
- Saga 测试框架（time-travel debug + 嵌套 sub-saga）
- **22 domain 真实数据接入完整化**：3 非核心 domain crate（collaboration / comment / board）接入 + **5 业务域 Step trait 全部实装** + 19 核心 domain 真实数据替换 mock
- Redis Cluster 模式（star-cache Redis stub 实装 + star-sse 多 node + star-webhook DB 持久化）
- Phase G+ 性能预算收敛（D15 收敛 SLO 实装）

本 spec 解决四个核心问题：
1. **接入顺序**：6 Tier 分层（per 依赖深度）确保每 Tier 验收后推下一 Tier，避免下游 crate 阻塞上游
2. **验收标准**：5 项硬约束（Resources handler + URI 命名 + Read 矩阵 + Write 矩阵 + cache 策略）+ 3 测试基线（read 1 + write 1 + permission 1）
3. **Saga 触发**：每 Tier 触发跨域 Saga（5 业务域独立 Lead per 8/21 JST）的具体步骤
4. **风险控制**：顺序不可逆 + 回滚 commit + InMemory 同步 + 性能基线 + DDD Review

本 spec 适用范围：Phase H 全周期 19 核心 + 3 非核心 = 22 domain crate 接入；不覆盖 Phase F 已有 mock stub 维护（per [ADR-0035 §2 D7 L66-82](../../adr/0035-phase-f-architecture.md) 当前 22 crate 真实接入进度 = 0%）。

## §2 22 domain 接入顺序

按依赖深度分层（低依赖先接入），per [spec/agents/02 §2 22 domain crate 数据源清单](../agents/02-data-sources-spec.md) + [spec/agents/02 §2.1 5 域映射](../agents/02-data-sources-spec.md)：

### Tier 1: 基础数据（无依赖，3 crate）

| # | Crate | 主键 | Resource URI | 5 域映射 | 接入工作量估算 |
|---|-------|------|--------------|----------|---------------|
| 1 | domain-tenant | tenant_id | `tenant://{tenant_id}` | Permission 域 | 0.8-1.2M tokens |
| 2 | domain-identity | user_id | `identity://{user_id}` | Permission 域 | 0.8-1.2M tokens |
| 3 | domain-permission | rule_id | `permission://{rule_id}` | Permission 域 | 1.0-1.5M tokens |

**Tier 1 rationale**：3 个 crate 互相无依赖，是所有其他 Tier 的前置（tenant → workspace, identity → agent/feedback, permission → policy）。Permission 域 Lead（per 8/21 JST 5 域独立）独立实施。

**Tier 1 验收门**：3 crate 全部 100% 接入 + 3×3=9 测试 + DDD Review + Permission Lead 签字。

### Tier 2: 业务原子（依赖 Tier 1，3 crate）

| # | Crate | 主键 | 依赖 | 5 域映射 | 接入工作量估算 |
|---|-------|------|------|----------|---------------|
| 4 | domain-workspace | ws_id | tenant | Worktree 域 | 1.2-1.6M tokens |
| 5 | domain-project | (待 spec/agents/02 v0.2 补，per [spec/agents/02 §6 #1](../agents/02-data-sources-spec.md) 已知缺口) | tenant | Flow 域 | 1.0-1.5M tokens |
| 6 | domain-work-item | wi_id | project | Flow 域 | 1.2-1.6M tokens |

**Tier 2 rationale**：workspace 是 5 域 Lead 协作的工作空间（per [ADR-0035 §4 5 域责任矩阵](../../adr/0035-phase-f-architecture.md)），project/work-item 是 Flow 域 Lead 核心。**触发 Saga**：workspace creation → Tenant scope check（per [spec/saga/01 §4 Q-003 交易 Saga 流程](../saga/01-saga-coordination-spec.md) 简化版 + tenant_id 校验 step）。

**Tier 2 验收门**：3 crate 全部 + 9 测试 + Worktree Lead + Flow Lead 双签字 + Saga 触发跑通。

### Tier 3: 业务实体（依赖 Tier 2，4 crate）

| # | Crate | 主键 | 依赖 | 5 域映射 | 接入工作量估算 |
|---|-------|------|------|----------|---------------|
| 7 | domain-worktree | wt_id | project + work-item | Worktree 域 | 1.5-2.0M tokens |
| 8 | domain-agent | agent_id | identity + workspace | Agent 域 | 1.5-2.0M tokens |
| 9 | domain-feedback | fb_id | work-item + identity | Integration 域 | 1.0-1.5M tokens |
| 10 | domain-decision | dec_id | work-item | Flow 域 | 1.2-1.6M tokens |

**Tier 3 rationale**：worktree/agent 是 Worktree/Agent 两域核心实体（per [ADR-0035 §4](../../adr/0035-phase-f-architecture.md)），feedback 是 Integration 域 Lead 接入，decision 是 Flow 域决策核心（Q-003 决策点 per [spec/saga/01 §4 G-06](../saga/01-saga-coordination-spec.md) Economy Lead 决策 SLA 待量化）。**触发 Saga**：worktree create → decision log（per [spec/agents/01 §2 Lease 协议](../agents/01-agent-runtime-spec.md) 30s heartbeat 复用 + decision 写 audit log）。

**Tier 3 验收门**：4 crate + 12 测试 + 3 域 Lead（Worktree/Agent/Integration）签字 + Flow Lead decision 域决策签字。

### Tier 4: 业务复合（依赖 Tier 3，4 crate）

| # | Crate | 主键 | 依赖 | 5 域映射 | 接入工作量估算 |
|---|-------|------|------|----------|---------------|
| 11 | domain-scm | scm_id | worktree + agent | Worktree 域 | 1.5-2.0M tokens |
| 12 | domain-validation | val_id | work-item + decision | Agent 域 | 1.0-1.5M tokens |
| 13 | domain-automation | rule_id | agent + decision | Flow 域 | 1.2-1.6M tokens |
| 14 | domain-search | query_id | work-item + agent | Integration 域 | 1.0-1.5M tokens |

**Tier 4 rationale**：scm 是 Worktree 域与 VCS 真实数据桥接（per [spec/vcs/05 §2 4 Git Provider 接入规范](../vcs/05-real-providers-spec.md)），validation/automation 是 Agent/Flow 域协作关键，search 是 Integration 域查询聚合。**触发 Saga**：pr open → audit log + notification（per [spec/saga/01 §4 5 步流程](../saga/01-saga-coordination-spec.md) AuditLog step + NotificationStep）。

**Tier 4 验收门**：4 crate + 12 测试 + 4 域 Lead 签字 + Saga 跑通 + scm 真实 Git provider 联通（per [ADR-0035 §2 D8 L84-103](../../adr/0035-phase-f-architecture.md) star-sa 4 provider trait）。

### Tier 5: 业务扩展（依赖 Tier 4，4 crate）

| # | Crate | 主键 | 依赖 | 5 域映射 | 接入工作量估算 |
|---|-------|------|------|----------|---------------|
| 15 | domain-policy | policy_id | validation + automation | Permission 域 | 1.0-1.5M tokens |
| 16 | domain-notification | nt_id | agent + decision | Integration 域 | 1.0-1.5M tokens |
| 17 | domain-context | ctx_id | worktree + decision + context | Integration 域 | 1.5-2.0M tokens |
| 18 | domain-resume | resume_id | agent + lease | Agent 域 | 1.0-1.5M tokens |

**Tier 5 rationale**：policy 是 Permission 域策略核心（per [ADR-0030 Agent Lease/Heartbeat/Resume](../../adr/0030-agent-lease-heartbeat-resume.md) Resume 协议），notification 是 Integration 域推送，context 是 Context Graph 4 节点（per [ADR-0031 §MVP](../../adr/0031-context-graph.md)），resume 是 Agent 域 lease 恢复。**触发 Saga**：policy update → audit + notification（per [spec/saga/01 §4 Q-003 流程](../saga/01-saga-coordination-spec.md) AuditLog step + NotificationStep + per [spec/cache/01 §5 失效策略](../cache/01-cache-contract-spec.md) cache 写穿透）。

**Tier 5 验收门**：4 crate + 12 测试 + 3 域 Lead 签字 + Saga 跑通 + cache 策略按 [spec/cache/01 §4 TTL 表](../cache/01-cache-contract-spec.md) 实装。

### Tier 6: 业务高级（依赖 Tier 5，3 crate + 3 非核心）

| # | Crate | 主键 | 依赖 | 5 域映射 | 接入工作量估算 |
|---|-------|------|------|----------|---------------|
| 19 | domain-audit | audit_id | 所有 Tier 1-5 | Admin 域 | 1.5-2.0M tokens |
| 20 | domain-integration | int_id | worktree + scm + agent | Integration 域 | 1.5-2.0M tokens |
| 21 | domain-event | event_id | agent + audit + notification | Integration 域 | 1.0-1.5M tokens |
| 22 | domain-flow | flow_id | (聚合) | Flow 域 | 1.0-1.5M tokens |
| 23 | domain-lease | lease_id | agent + resume | Agent 域 | 1.0-1.5M tokens |

**Tier 6 rationale**：audit 是 Admin 域核心（per [ADR-0027 §3 STAR IDE Gateway 5 域责任矩阵](../../adr/0027-star-ide-gateway.md) + COC 独立控制面 per 8/21 JST），integration 是 3 非核心（collaboration / comment / board per [ADR-0036 §8.2 L296](../../adr/0036-phase-g-architecture.md) "3 非核心 domain crate"）的接入，event/flow/lease 是跨域聚合实体。**触发 Saga**：integration event → audit + notification（per [spec/services/02 §3 SSE event schema](../services/02-sse-streaming-spec.md) CacheInvalidate 广播 + [spec/saga/01 §5 状态机持久化](../saga/01-saga-coordination-spec.md)）。

**Tier 6 验收门**：6 crate + 18 测试 + 5 域 Lead 全签字 + Saga 跑通 + 性能基线（per §5 #4）+ DDD Review 全终审。

### §2.1 总览

| Tier | Crate 数 | 5 域分布 | 工作量合计 (tokens) | 累计 (tokens) |
|------|----------|----------|---------------------|---------------|
| Tier 1 | 3 | Permission 3 | 2.6-3.9M | 2.6-3.9M |
| Tier 2 | 3 | Worktree 1 + Flow 2 | 3.4-4.7M | 6.0-8.6M |
| Tier 3 | 4 | Worktree 1 + Agent 1 + Integration 1 + Flow 1 | 5.2-7.1M | 11.2-15.7M |
| Tier 4 | 4 | Worktree 1 + Agent 1 + Flow 1 + Integration 1 | 4.7-6.6M | 15.9-22.3M |
| Tier 5 | 4 | Permission 1 + Integration 2 + Agent 1 | 4.5-6.5M | 20.4-28.8M |
| Tier 6 | 6 | Admin 1 + Integration 3 + Flow 1 + Agent 1 | 6.0-8.5M | 26.4-37.3M |

**Phase H 总计**：22+1=23 crate（含 lease 是 22 核心 + 1 跨域），合计 26.4-37.3M tokens ≈ 3-4 人·周（per 8/21 JST 1 人·周 ≈ 1M tokens）。vs [ADR-0036 §5 L237](../../adr/0036-phase-g-architecture.md) Phase G 15-23M + [ADR-0035 §5 L205](../../adr/0035-phase-f-architecture.md) Phase F 22 domain 25-40M，Phase H 工作量与 Phase F 接入量级一致。

## §3 每个 domain 接入验收 5 项

per [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md) + [spec/mcp/02 §1 Resources 4 类](../mcp/02-resources-prompts-spec.md) + [spec/agents/02 §3 Read 权限矩阵](../agents/02-data-sources-spec.md) + [spec/agents/02 §4 Write 权限矩阵](../agents/02-data-sources-spec.md) + [spec/cache/01 §4 TTL 表](../cache/01-cache-contract-spec.md)：

| # | 验收项 | 引用 | 失败处理 |
|---|--------|------|----------|
| 1 | **Resources handler 实现** | [spec/mcp/02 §1](../mcp/02-resources-prompts-spec.md) 4 类（workspace / worktree / agent / decision）+ [spec/agents/02 §2.2 URI 模式](../agents/02-data-sources-spec.md) `{crate}://{id}` 22 crate 通用解析（`crates/star-mcp/src/resources.rs` `parse_resource_uri()`） | 资源类未注册 → `RESOURCE_NOT_FOUND`（per [spec/mcp/03 §2 标准错误码表](../mcp/03-error-model-spec.md)） |
| 2 | **Resource URI 命名** | [spec/agents/02 §2 22 domain URI 表](../agents/02-data-sources-spec.md) L16-38 + §2.2 URI 模式约束 L80-86（`{crate}://{id}` / `{crate}://list?limit&offset&filter` / `{crate}://{id}/subscribe`）| URI 格式不匹配 → `INVALID_URI_FORMAT`（per [spec/agents/02 §2.2 L85](../agents/02-data-sources-spec.md)） |
| 3 | **读权限矩阵实现** | [spec/agents/02 §3 Read 权限矩阵](../agents/02-data-sources-spec.md)（`ActorType × 5 域 × 3 角色 owner/member/public`）| 权限拒绝 → `PERMISSION_DENIED`（per [spec/mcp/03 §2](../mcp/03-error-model-spec.md)）|
| 4 | **写权限矩阵 + 状态机校验** | [spec/agents/02 §4 Write 权限矩阵](../agents/02-data-sources-spec.md) + §4.1 状态机校验 L132-138 + §4.3 写穿透伪代码 L152-167 | 状态机非法转换 → `INVALID_STATE_TRANSITION` |
| 5 | **cache 策略** | [spec/cache/01 §4 TTL 表](../cache/01-cache-contract-spec.md) L132-143（10 类数据 TTL：5s agent state / 30s worktree / 60s PR / 300s workspace / 3600s branch / 86400s commit/audit）| 缓存穿透 → `CACHE_KEY_NOT_FOUND`（per [spec/cache/01 §2.1 6 字段错误模型](../cache/01-cache-contract-spec.md)） |

### §3.1 测试基线（3 测试 / crate）

每个 domain 至少 3 个测试（per [spec/agents/02 §6 已知缺口 #1](../agents/02-data-sources-spec.md) "3 测试基线未量化"在本 spec 显式约束）：

| # | 测试类型 | 引用 | 覆盖点 |
|---|----------|------|--------|
| 1 | **read 1** | [spec/agents/02 §2.2 URI 模式](../agents/02-data-sources-spec.md) | 调 `parse_resource_uri` → `cache.get` → 22 crate 数据源（mock 或真实）→ 断言字段 |
| 2 | **write 1** | [spec/agents/02 §4.3 写穿透伪代码](../agents/02-data-sources-spec.md) L152-167 | 调 write path → 状态机校验 → 写 22 crate → cache.del → SSE 广播 |
| 3 | **permission 1** | [spec/agents/02 §3 Read 权限矩阵](../agents/02-data-sources-spec.md) | 5 域 × 3 角色 owner/member/public 至少 1 个拒绝 + 1 个允许用例 |

**总测试数**：22 core × 3 = 66 + 3 非核心 × 3 = 9 = **75 测试**（per Tier 验收门：Tier 1 = 9 / Tier 2 = 9 / Tier 3 = 12 / Tier 4 = 12 / Tier 5 = 12 / Tier 6 = 18 + 3 非核心 3 = 75）。

### §3.2 22 domain crate 各自 lead 验收签字 (v0.2 5 域脱钩后)

per 8/21 JST 用户偏好"5 域独立 Lead，不接受兼任"（per [AGENTS.md §4 #3 守门硬约束](../../../../../AGENTS.md)）+ 2026-08-31 22:45 JST Q1-D 拍板（5 域脱钩 22 DDD）：

**22 domain crate 各自 lead 签字表** (per [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md) + [spec/saga/01 v0.2 §3 SagaCoordinationRole 映射](../../adr/0031-context-graph.md))：

| 22 domain crate | 5 域历史归类 (footnote only) | SagaCoordinationRole 主导 | Lead 身份 |
|---|---|---|---|
| domain-tenant | Permission 域 | IdentityValidation | ⏳ tenant lead (DDD Review 阶段补) |
| domain-identity | Permission 域 | IdentityValidation | ⏳ identity lead |
| domain-permission | Permission 域 | DecisionAuthorization | ⏳ permission lead |
| domain-policy | Permission 域 | DecisionAuthorization | ⏳ policy lead |
| domain-workspace | Worktree 域 | ResourceMutation | ⏳ workspace lead |
| domain-project | Flow 域 | ResourceMutation | ⏳ project lead |
| domain-work-item | Flow 域 | ResourceMutation | ⏳ work-item lead |
| domain-workflow | Flow 域 | DecisionAuthorization | ⏳ workflow lead |
| domain-board | Flow 域 | ResourceMutation | ⏳ board lead |
| domain-planning | Flow 域 | ResourceMutation | ⏳ planning lead |
| domain-worktree | Worktree 域 | ResourceMutation | ⏳ worktree lead |
| domain-agent | Agent 域 | IdentityValidation | ⏳ agent lead |
| domain-feedback | Integration 域 | DecisionAuthorization | ⏳ feedback lead |
| domain-validation | Agent 域 | ResourceMutation | ⏳ validation lead |
| domain-scm | Worktree 域 | ACL(隔离) | ⏳ scm lead |
| domain-development | Worktree 域 | ResourceMutation | ⏳ development lead |
| domain-context | Integration 域 | StateObservation | ⏳ context lead |
| domain-automation | Flow 域 | DecisionAuthorization | ⏳ automation lead |
| domain-integration | Integration 域 | ACL(隔离) | ⏳ integration lead |
| domain-audit | Admin 域 | AuditLogging | ⏳ audit lead (COC 独立控制面) |
| domain-search | Integration 域 | Published Language | ⏳ search lead |
| domain-notification | Integration 域 | Separate Ways | ⏳ notification lead |
| domain-collaboration | Integration 域 | Customer-Supplier | ⏳ collaboration lead |
| domain-comment | Integration 域 | ResourceMutation | ⏳ comment lead |
| domain-relation | Integration 域 | Customer-Supplier | ⏳ relation lead |
| domain-local-runtime | Worktree 域 | Conformist | ⏳ local-runtime lead |

> **5 域历史归类列仅作 footnote** (per dual-use 警告 + AGENTS.md §5 v0.6)，实际代码层归属通过 [spec/saga/01 v0.2 §2](../saga/01-saga-coordination-spec.md) `SagaStep.responsible_crate: &str` 字段显式声明到 22 domain crate 之一。

**禁止兼任约束**（per 8/21 JST + AGENTS.md §4 #3 守门，**仅适用于 5 域历史治理命名**）：
- ❌ 架构师不得兼任 Permission / Worktree / Flow / Integration / Agent 5 域任意 Lead
- ❌ SRE 不得兼任 Admin 域（per COC 独立控制面）
- ❌ 同一 Lead 不得签 2 个 5 域

> Star 仓 22 domain crate 各自的 lead 兼任约束待 DDD Review 阶段补（per [AGENTS.md §4 #3 v0.6 Q1-D 拍板 +c](../../../../AGENTS.md)，5 域独立 Lead ≠ 22 domain 独立 lead，二者**不建立映射**）。

## §4 跨域 Saga 触发点

per [spec/saga/01 §1 跨域 Saga 协调契约](../saga/01-saga-coordination-spec.md) + §4 Q-003 交易 Saga 5 步流程 + §5 状态机 6 转换：

### §4.1 Tier 2: workspace creation → Tenant scope check

**触发条件**：用户创建 workspace（`POST /workspaces` per [spec/rest/01](../rest/) 草稿）

**Saga 步骤**（3 step，每 step 标注 `responsible_crate` per [spec/saga/01 v0.2 §2](../saga/01-saga-coordination-spec.md)）：
1. step `ValidateTenantScope`（responsible_crate: `domain-permission`, coordination_role: `IdentityValidation`）— 校验 tenant_id 有效 + 用户有 tenant 权限（调 domain-tenant + domain-permission）
2. step `CreateWorkspace`（responsible_crate: `domain-workspace`, coordination_role: `ResourceMutation`）— 调 domain-workspace 真实数据源（PostgreSQL 真实库 / Phase H+）
3. step `AuditLog`（responsible_crate: `domain-audit`, coordination_role: `AuditLogging`, 必填且最后）— 记录 workspace 创建审计

**状态机**：Pending → Running → Completed / Compensating → Compensated

**失败补偿**：workspace 创建失败 → `DeleteWorkspace`（reverse） + AuditLog no-op

### §4.2 Tier 3: worktree create → decision log

**触发条件**：用户创建 worktree（`POST /worktrees` + git checkout per [spec/vcs/05 §2 4 Git Provider 接入](../vcs/05-real-providers-spec.md)）

**Saga 步骤**（4 step，每 step 标注 `responsible_crate`）：
1. step `CreateWorktreeGit`（responsible_crate: `domain-scm`, coordination_role: `ResourceMutation`）— 调 git provider（per [spec/vcs/05 §2 4 provider 接入](../vcs/05-real-providers-spec.md) 真实 provider，star-sa OAuth token 缓存 per [spec/cache/01 §3.1 Provider 元数据](../cache/01-cache-contract-spec.md)）
2. step `PersistWorktree`（responsible_crate: `domain-worktree`, coordination_role: `ResourceMutation`）— 调 domain-worktree 真实数据源
3. step `LogWorktreeDecision`（responsible_crate: `domain-work-item` 或 `domain-context`, coordination_role: `DecisionAuthorization`）— 调 decision 记录 worktree 决策（`kind=worktree_create`）
4. step `AuditLog`（responsible_crate: `domain-audit`, coordination_role: `AuditLogging`, 必填且最后）

**状态机**：Pending → Running → Completed / Compensating → Compensated

**失败补偿**：git checkout 失败 → `CleanupWorktree` + decision invalidation + audit log

### §4.3 Tier 4: pr open → audit log + notification

**触发条件**：用户开 PR（`POST /pulls` per [spec/vcs/05 §3 PR 模型](../vcs/05-real-providers-spec.md) + [spec/saga/01 §4 Q-003 交易 Saga 流程](../saga/01-saga-coordination-spec.md) MR 触发场景）

**Saga 步骤**（5 step，每 step 标注 `responsible_crate`）：
1. step `CreatePullRequest`（responsible_crate: `domain-scm`, coordination_role: `ResourceMutation`）— 调 git provider API
2. step `LinkPRToWorkItem`（responsible_crate: `domain-work-item`, coordination_role: `ResourceMutation`）— 关联 work_item_id
3. step `LogPRDecision`（responsible_crate: `domain-work-item` 或 `domain-context`, coordination_role: `DecisionAuthorization`）— 决策 kind=pr_open
4. step `AuditLog`（responsible_crate: `domain-audit`, coordination_role: `AuditLogging`, 必填且最后）
5. step `NotifyPRCreated`（responsible_crate: `domain-notification`, coordination_role: `ResourceMutation`）— 调 domain-notification 推送（per [spec/services/02 §3 SSE event schema](../services/02-sse-streaming-spec.md)）

**状态机**：Pending → Running → Completed / Compensating → Compensated

**失败补偿**：PR 创建失败 → `ClosePR` + work_item unlink + decision invalidate + audit + notification no-op

### §4.4 Tier 5: policy update → audit + notification

**触发条件**：Permission Lead 更新 policy（`PATCH /policies/{policy_id}`）

**Saga 步骤**（4 step，每 step 标注 `responsible_crate`）：
1. step `UpdatePolicy`（responsible_crate: `domain-policy`, coordination_role: `ResourceMutation`）— 调 domain-policy 真实数据源 + cache 写穿透（per [spec/cache/01 §5.1 写穿透伪代码](../cache/01-cache-contract-spec.md) L168-205）
2. step `LogPolicyDecision`（responsible_crate: `domain-work-item` 或 `domain-context`, coordination_role: `DecisionAuthorization`）— 决策 kind=policy_update
3. step `AuditLog`（responsible_crate: `domain-audit`, coordination_role: `AuditLogging`, 必填且最后）
4. step `NotifyPolicyUpdated`（responsible_crate: `domain-notification`, coordination_role: `ResourceMutation`）

**状态机**：Pending → Running → Completed / Compensating → Compensated

**失败补偿**：policy 更新失败 → `RevertPolicy` + cache 恢复 + decision invalidate

### §4.5 Tier 6: integration event → audit + notification

**触发条件**：外部集成事件（webhook 到达 per [spec/services/03 webhook adapter](../services/03-webhook-adapter-spec.md) 或 Integration 域 event bus）

**Saga 步骤**（5 step，每 step 标注 `responsible_crate`）：
1. step `ReceiveIntegrationEvent`（responsible_crate: `domain-integration`, coordination_role: `IdentityValidation`）— webhook adapter 接收
2. step `PersistEvent`（responsible_crate: `domain-integration` 或 `domain-work-item`, coordination_role: `ResourceMutation`）— 调真实数据源
3. step `LogEventDecision`（responsible_crate: `domain-work-item` 或 `domain-context`, coordination_role: `DecisionAuthorization`）— 决策 kind=integration_event
4. step `AuditLog`（responsible_crate: `domain-audit`, coordination_role: `AuditLogging`, 必填且最后）
5. step `NotifySubscribers`（responsible_crate: `domain-notification` 或 `domain-collaboration`, coordination_role: `ResourceMutation`）— SSE 推送（per [spec/services/02 §3 SSE event schema](../services/02-sse-streaming-spec.md) `CacheInvalidate` 广播）

**状态机**：Pending → Running → Completed / Compensating → Compensated

**失败补偿**：event 持久化失败 → `DiscardEvent`（best-effort）+ audit + notification no-op（per [spec/saga/01 §4 补偿失败入死信 G-05](../saga/01-saga-coordination-spec.md)）

## §5 风险控制

per [ADR-0035 §8 不变量](../../adr/0035-phase-f-architecture.md) L304-306 + [ADR-0036 §8.3 Phase F → Phase G 不变量](../../adr/0036-phase-g-architecture.md) L300-306 + 8/21 JST token-OLU 框架 + 8/27 JST 21:59 用户授权 + 8/27 JST 11:06 JST hard ban 环境变量安全：

### §5.1 顺序不可逆

- **规则**：Tier N 未通过不能推 Tier N+1
- **强制点**：每 Tier 验收门 = 100% 接入 + 3 测试 / crate + DDD Review + 5 域 Lead 签字（per §3.2）
- **违反后果**：违反顺序可能导致下游 crate 阻塞（mock 数据不一致）+ 测试不通过
- **引用**：[AGENTS.md §4 #3 守门硬约束 5 域独立 Lead 不接受兼任](../../../../../AGENTS.md) + [spec/saga/01 §3 5 域 Lead 兼任约束](../saga/01-saga-coordination-spec.md)

### §5.2 回滚机制

- **规则**：每 Tier 完成后立即 commit + tag，失败可回退
- **强制点**：
  - `git commit -m "feat(domain): tier-N 接入完成 - <crate-list>"`
  - `git tag phase-h-tier-N-complete` 标签化
  - 失败 `git reset --hard phase-h-tier-N-complete` 回退
- **commit author 守门**：per 8/27 21:59 JST 用户授权第三次强化 + 8/27 11:06 JST hard ban —— author = Ulysses（一人公司 12 角色 per DEC-008），**禁止** commit message 引用 `$env:XXX` 等环境变量（per memory "Commit message 不能引用 env 变量 2026-08-27 21:51 JST 教训"）
- **引用**：[AGENTS.md §2 commit author 形式](../../../../../AGENTS.md) + §1.0 用户授权升级 v0.3-v0.5

### §5.3 数据一致性

- **规则**：用 [spec/cache/01 §2.3 InMemory backend](../cache/01-cache-contract-spec.md) 同步（`dashmap` + `tokio::time::sleep`），Redis Phase H+ 才上
- **强制点**：
  - Phase H 单 node 部署用 `InMemoryBackend`（per [spec/cache/01 §2.3 L89](../cache/01-cache-contract-spec.md)）
  - 多 node 部署 Phase H+ 才上 `RedisBackend`（per [ADR-0036 §7 #1 L268](../../adr/0036-phase-g-architecture.md) "Redis 后端仅 stub"）
  - 22 domain cache key 命名严格按 [spec/cache/01 §3 命名规范](../cache/01-cache-contract-spec.md) `cache:v1:{crate}:{id}` 形式
- **引用**：[spec/cache/01 §2.3 2 后端实现](../cache/01-cache-contract-spec.md) + [ADR-0036 §2 D11 L140-167](../../adr/0036-phase-g-architecture.md)

### §5.4 性能监控

- **规则**：每 Tier 跑 H4 性能基线（per [ADR-0035 §8.2 D15 性能预算 NFR](../../adr/0035-phase-f-architecture.md) 5 项 SRE NFR），对比 baseline
- **强制点**：
  - 4 指标 P50 / P95 / P99（per [ADR-0035 §8.2 D15 L140](../../adr/0035-phase-f-architecture.md)）
  - 5 域 × 22 domain × 3 测试 = 330 性能样本 / Tier
  - P99 ≤ 10ms 是 Phase H 目标（per [ADR-0036 §7 #8 L275](../../adr/0036-phase-g-architecture.md) 待 Phase G+ 报告后定）
- **基线参考**：Phase F 22 domain 接入性能基线 = 0%（per [ADR-0035 §7 #10 L247](../../adr/0035-phase-f-architecture.md) "未实装"）
- **引用**：[ADR-0035 §8.2 D15 L140-156](../../adr/0035-phase-f-architecture.md) + [ADR-0036 §7 #8 L275](../../adr/0036-phase-g-architecture.md)

### §5.5 DDD Review

- **规则**：每 Tier 完成后 Ulysses 终审签字才进下一 Tier
- **强制点**：
  - 报告 7 段结构（per [AGENTS.md §3 报告 7 段结构](../../../../../AGENTS.md)）：§0 目的 + §1 改动矩阵 + §2 验证摘要 + §3 已知缺口 + §4 子代理失败接手清单 + §5 守门规则 + §6 签字栏
  - 5 角色签字（per [AGENTS.md §3 #7](../../../../../AGENTS.md)）：架构 / SRE Lead / 平台 / 评审主持 / PM
  - 代签规则：per [AGENTS.md §1.0 用户授权升级](../../../../../AGENTS.md) + 8/27 19:39 / 20:56 / 21:59 JST 三次强化
  - 5 域 Lead 真实身份（per 8/21 JST 拒绝兼任）签字请 DDD Review 阶段补（⏳ 待签 per [ADR-0036 §4 L182-190](../../adr/0036-phase-g-architecture.md)）
- **引用**：[AGENTS.md §3 报告 7 段结构 + §4 12 项守门硬约束](../../../../../AGENTS.md)

## §6 已知缺口

per 8/26 04:30 "缺标比错标安全" + 8/27 21:59 JST Mavis 接手代签（不动 ⏳ 待签的 SRE/平台/评审/PM 5 域 Lead）+ [ADR-0036 §7 12 项已知缺口](../../adr/0036-phase-g-architecture.md) 模板：

| # | 缺口 | 影响 | 状态 | 触发 |
|---|------|------|------|------|
| 1 | 22 domain 接入实际工作量未量化 | 本 spec §2.1 估 26.4-37.3M tokens，但每 crate 实装消耗与估算偏差可能 ±30% | 缺标，PM 拍板 | per [ADR-0035 §5 L205](../../adr/0035-phase-f-architecture.md) "22 domain 接入 25-40M" + [ADR-0036 §7 #5 L272](../../adr/0036-phase-g-architecture.md) |
| 2 | 部分 domain 接入需要真实外部服务 | domain-integration 需真 Git provider（github / gitlab / bitbucket / gitea，per [spec/vcs/05 §2 4 provider 接入](../vcs/05-real-providers-spec.md)），Phase F stub → Phase H 真实 | 缺标，Worktree Lead + Integration Lead 协同 | per [ADR-0035 §2 D8 L84-103](../../adr/0035-phase-f-architecture.md) star-sa |
| 3 | 多租户隔离边界未明 | domain-tenant 接入时 tenant_id 边界 vs 5 域映射的交叉（per [spec/agents/02 §6 #2 L195-204](../agents/02-data-sources-spec.md)）| 缺标，Permission Lead 拍板 | per [spec/agents/02 §6 #2](../agents/02-data-sources-spec.md) |
| 4 | 数据迁移路径（mock → 真实）未设计 | Phase F mock stub → Phase H 真实数据源（PostgreSQL / Redis / S3 等）的 migration 脚本 + 数据回填 | 缺标，平台 Lead 拍板 | per [ADR-0035 §2 D7 L66-82](../../adr/0035-phase-f-architecture.md) |
| 5 | 5 域业务域 Lead 真实身份 | Player / Economy / Match / Social / Admin 5 业务域 Lead 签字 ⏳ 待签（per [ADR-0036 §4 L182-190](../../adr/0036-phase-g-architecture.md)）| 缺标，DDD Review 阶段补 | per 8/21 JST 5 域独立 Lead 拒绝兼任 |
| 6 | Tier 5 context domain 跨 22 域访问性能未基线 | domain-context 是 [ADR-0031 Context Graph](../../adr/0031-context-graph.md) 4 节点 MVP，跨 22 域访问 P99 未基线 | 缺标，SRE Lead 量化 | per [ADR-0036 §7 #8 L275](../../adr/0036-phase-g-architecture.md) |
| 7 | 接入完成后的 acceptance/01-17 重新跑 | acceptance 是 5/26 旧版（per [AGENTS.md §7 待办 #7](../../../../../AGENTS.md) "25 domain-* crate 真实数据接入"），22 crate 真实接入后需重写 acceptance 01-17 测试 | 缺标，PM 拍板 | per [AGENTS.md §7 待办 #7](../../../../../AGENTS.md) |

### §6.1 缺口处理原则

per 2026-08-26 11:06 JST Ulysses 拍板"缺标比错标安全"原则：所有缺口**显式列出**而不**默默假设**已解决。本 spec §6 7 项缺口均为 Phase H 待办，**不**在 v0.1 范围承诺实现。

## §7 引用文档

- [adr/0035-phase-f-architecture.md](../../adr/0035-phase-f-architecture.md) — Phase F 整体架构（§1.1 + §2 D6-D10 + §4 5 域 Lead + §5 token-OLU + §7 已知缺口 + §8.2 Phase G 方向）
- [adr/0036-phase-g-architecture.md](../../adr/0036-phase-g-architecture.md) — Phase G 整体架构（§2 D11-D15 + §4 5 决策 + §5 token-OLU + §7 12 已知缺口 + §8.2 Phase H 方向 = 本 spec 触发）
- [adr/0030-agent-lease-heartbeat-resume.md](../../adr/0030-agent-lease-heartbeat-resume.md) — Lease + Heartbeat + Resume 11 字段（domain-lease / domain-resume 接入依据）
- [adr/0031-context-graph.md](../../adr/0031-context-graph.md) — Context Graph MVP 4 节点（domain-context 接入依据）
- [adr/0033-agent-co-signing-policy.md](../../adr/0033-agent-co-signing-policy.md) — 代签规则（commit author + 报告"审批者"列）
- [spec/agents/01-agent-runtime-spec.md](../agents/01-agent-runtime-spec.md) — Agent Runtime Spec（§2 Lease 协议 30s heartbeat / 300s TTL，§2.1 L108-117 Lease 校验）
- [spec/agents/02-data-sources-spec.md](../agents/02-data-sources-spec.md) — 22 domain crate 数据源契约（§2 22 domain 清单 + §2.1 5 域映射 + §2.2 URI 模式 + §3 Read 权限矩阵 + §4 Write 权限矩阵 + §6 已知缺口）
- [spec/mcp/01-mcp-spec.md](../mcp/01-mcp-spec.md) — MCP core spec（§4 Resources + §5 Prompts + §1.1 6 字段扩展）
- [spec/mcp/02-resources-prompts-spec.md](../mcp/02-resources-prompts-spec.md) — Resources 4 类（workspace / worktree / agent / decision，§1 引用 §3 验收项 #1）
- [spec/mcp/03-error-model-spec.md](../mcp/03-error-model-spec.md) — 6 字段错误模型（per §3 验收项 #1-#5 错误码映射）
- [spec/cache/01-cache-contract-spec.md](../cache/01-cache-contract-spec.md) — 数据缓存契约（§2 Cache trait + §3 Key 命名 + §4 TTL 表 + §5 失效策略 + §6 已知缺口）
- [spec/saga/01-saga-coordination-spec.md](../saga/01-saga-coordination-spec.md) — 跨域 Saga 协调（§1 目的 + §2 Saga 抽象 + §3 5 域 Lead 映射 + §4 Q-003 5 步流程 + §5 状态机 6 转换 + §6 10 已知缺口）
- [spec/vcs/05-real-providers-spec.md](../vcs/05-real-providers-spec.md) — 4 Git Provider 接入规范（§2 4 provider + §3 PR 模型）
- [spec/services/02-sse-streaming-spec.md](../services/02-sse-streaming-spec.md) — SSE 推送通道（§3 event schema CacheInvalidate 广播）
- [spec/services/03-webhook-adapter-spec.md](../services/03-webhook-adapter-spec.md) — Webhook Adapter（§7 死信模式参考 spec/saga/01 §4 G-05）
- [AGENTS.md §0 一句话硬约束 + §3 报告 7 段结构 + §4 12 项守门硬约束](../../../../../AGENTS.md)

### §7.1 引用原则

- §2 6 Tier 接入顺序与 [spec/agents/02 §2 22 domain 数据源清单](../agents/02-data-sources-spec.md) + [spec/agents/02 §2.1 5 域映射](../agents/02-data-sources-spec.md) 对齐
- §3 5 项验收标准与 [spec/agents/02 §2-§4 22 domain 接入契约](../agents/02-data-sources-spec.md) + [spec/mcp/02 §1 4 资源类](../mcp/02-resources-prompts-spec.md) + [spec/cache/01 §4 TTL 表](../cache/01-cache-contract-spec.md) 协同
- §4 跨域 Saga 触发点与 [spec/saga/01 §1-§5 Saga 协调契约](../saga/01-saga-coordination-spec.md) + [ADR-0036 §2 D12 Saga 跨域协调](../../adr/0036-phase-g-architecture.md) 引用
- §5 风险控制与 [AGENTS.md §4 12 项守门硬约束](../../../../../AGENTS.md) + [ADR-0036 §8.3 Phase F → Phase G 不变量](../../adr/0036-phase-g-architecture.md) 引用

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：6 Tier 接入顺序（22+1=23 crate × 6 Tier 工作量 26.4-37.3M tokens）+ 5 项验收（Resources handler / URI 命名 / Read 矩阵 / Write 矩阵 + 状态机 / cache 策略）+ 3 测试 / crate = 75 测试 + 5 域 Lead 验收签字 + 5 跨域 Saga 触发点（workspace create / worktree create / pr open / policy update / integration event）+ 5 风险控制（顺序不可逆 / 回滚 commit / InMemory 同步 / 性能基线 / DDD Review）+ 7 已知缺口（per 缺标比错标安全）+ 8 引用文档 + 引用原则 | per [ADR-0036 §8.2 Phase H 方向 L293-298](../../adr/0036-phase-g-architecture.md) "22 domain 真实数据接入完整化" + 2026-08-27 21:59 JST 用户授权第三次强化代签（per [AGENTS.md §1.0 v0.5 三次强化](../../../../../AGENTS.md)）|
| v0.2 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | **5 域绑定冲突修复 (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板)**：① §0 文件头加 dual-use 警告 + 5 域 ≠ 22 DDD 映射说明；② §3.2 整段重写为"22 domain crate 各自 lead 验收签字" (26 行 5 域 lead 表 → 22 domain crate × SagaCoordinationRole 主导表 + 5 域归类 footnote)；③ §4 5 个跨域 Saga 触发点全部 step 标注 `responsible_crate` + `coordination_role` (per [spec/saga/01 v0.2 §2](../saga/01-saga-coordination-spec.md))；④ §3.2 / §4 仍保留"5 域历史归类"作为兼容性 footnote，不删历史 | 2026-09-01 14:38 JST 模块间协作细化任务 (A 架构层 22 Domain 协作 + L3 完整覆盖 + doc-only) |

---

> **审批者**：架构师 (Mavis 接手 agent per DEC-008) — 2026-08-28 (v0.1) / 2026-09-01 (v0.2)
> **per AGENTS.md §1 代签规则反转 + 2026-08-27 19:39 JST 代签授权升级 + 21:59 JST 第三次强化**：Mavis 接手默认代签 Ulysses 无需再问
