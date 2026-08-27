# Spec-02: 22 domain crate 数据源契约

> **状态**：Draft v0.1
> **日期**：2026-08-27
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0035 D7 L66 / 2026-08-27 21:59 JST 用户授权

## §1 目的

定义 22 domain crate 的数据源访问契约，统一为 Resource URI 模式 `agent://{crate}/{id}`，供 `crates/star-mcp/src/resources.rs` Resources handler（per [spec/mcp/02 §1](../mcp/02-resources-prompts-spec.md)）+ Phase G 缓存层（per ADR-0035 §2 D8 L80-90）消费。本 spec 是 Phase E mock data（per ADR-0034 §2 D5 L146 `// TODO(phase-f): 接入 22 domain crate 真实数据`）的替换目标，Agent 启动时（per spec/agents/01）必须能查到 22 crate 的访问契约（trait + 参数表 + 错误码）以决定缓存键 / 失效钩子 / SSE 推送通道。

## §2 22 domain crate 数据源清单

per 实际 `crates/` 目录（`git ls-tree main crates/`，仅列 22 核心 domain crate，3 非核心列 §6 #1 已知缺口）：

| Crate | 主键 | 关键字段 | Resource URI |
|-------|------|----------|--------------|
| domain-agent | agent_id | state / session_id / created_at | agent://{agent_id} |
| domain-worktree | wt_id | branch / commit / path / status | worktree://{wt_id} |
| domain-feedback | fb_id | target_type / target_id / severity | feedback://{fb_id} |
| domain-work-item | wi_id | title / status / assignee | workitem://{wi_id} |
| domain-tenant | tenant_id | name / plan / created_at | tenant://{tenant_id} |
| domain-identity | user_id | email / role / tenant_id | identity://{user_id} |
| domain-permission | rule_id | role / resource / action | permission://{rule_id} |
| domain-policy | policy_id | name / rules | policy://{policy_id} |
| domain-context | ctx_id | packet / priority | context://{ctx_id} |
| domain-decision | dec_id | status / superseded_by | decision://{dec_id} |
| domain-event | event_id | type / source / payload | event://{event_id} |
| domain-audit | audit_id | actor / action / target | audit://{audit_id} |
| domain-validation | val_id | result / kind | validation://{val_id} |
| domain-lease | lease_id | agent_id / expires_at | lease://{lease_id} |
| domain-resume | resume_id | state / checkpoint | resume://{resume_id} |
| domain-integration | int_id | source / target / status | integration://{int_id} |
| domain-automation | rule_id | trigger / action | automation://{rule_id} |
| domain-search | query_id | terms / filters | search://{query_id} |
| domain-notification | nt_id | channel / template | notification://{nt_id} |
| domain-scm | scm_id | provider / repo | scm://{scm_id} |
| domain-workspace | ws_id | name / members | workspace://{ws_id} |
| domain-flow | flow_id | name / state | flow://{flow_id} |

列表 URI：`{crate}://list?limit=N&offset=M&filter=...`

### §2.1 22 crate 分类（5 域映射 per ADR-0035 §4）

| 分类 | 包含 crate | 域归属（per ADR-0035 §4 5 域 Lead 映射）|
|------|-----------|----------------------------------|
| Agent 运行时 | domain-agent / domain-lease / domain-resume / domain-validation | Agent 域 Lead |
| Worktree / Workspace | domain-worktree / domain-workspace / domain-scm | Worktree 域 Lead |
| 工作流 / 任务 | domain-work-item / domain-flow / domain-decision / domain-event / domain-automation | Flow 域 Lead |
| 权限 / 多租户 | domain-tenant / domain-identity / domain-permission / domain-policy / domain-audit | Permission 域 Lead |
| 集成 / 通知 | domain-integration / domain-search / domain-notification / domain-feedback / domain-context | Integration 域 Lead |

### §2.2 URI 模式约束

- 单资源：`{crate}://{id}` — `id` 必须匹配 crate 主键类型（u64 / uuid / 字符串）
- 列表：`{crate}://list?limit=N&offset=M&filter=key:value,...`
  - `limit` 默认 50，上限 1000
  - `offset` 默认 0
  - `filter` 语法 = `key:value` 逗号分隔，可重复；非法 key 返回 `INVALID_FILTER_KEY`（per spec/mcp/03 §3 错误模型）
- 订阅：`{crate}://{id}/subscribe` — 返回 SSE channel id（per [spec/services/02 §3](../services/02-sse-streaming-spec.md)）

### §2.3 与 spec/mcp/02 §1 Resources 4 类的关系

[spec/mcp/02 §1](../mcp/02-resources-prompts-spec.md) L14-78 已定义 4 个**首批 Resources 类**（workspace / worktree / agent / decision），**与本 spec 不冲突**：
- spec/mcp/02 §1.3 L32-42 worktree 资源 = 本 spec domain-worktree 子集
- spec/mcp/02 §1.4 L43-54 agent 资源 = 本 spec domain-agent 子集
- spec/mcp/02 §1.5 L55-68 decision 资源 = 本 spec domain-decision 子集
- spec/mcp/02 §1.2 L23-31 workspace 资源 = 本 spec domain-workspace 子集
- 其余 18 crate 待 Phase F+ 通过本 spec §2 URI 模式追加（per spec/mcp/02 §1.6 L69-79 通用协议）
- 资源类解析顺序：`crates/star-mcp/src/resources.rs` `parse_resource_uri()` 先匹配 spec/mcp/02 §1.1-1.5 4 资源类，未命中回退到本 spec §2.1 URI 模式 22 crate 通用解析器

## §3 Read 权限矩阵

每 crate × 5 ActorType × 3 角色（owner / member / public）：

| ActorType | 自身数据 | tenant 共享 | public |
|-----------|----------|------------|--------|
| Human | ✅ | tenant 内 ✅ | 公开字段 ✅ |
| Agent | ⚠️ 需 lease | ⚠️ 仅 task 范围 | ❌ |
| System | ✅ | ✅ | ❌ |
| Service | ⚠️ 需 service token | ⚠️ 仅声明范围 | ❌ |
| Automation | ⚠️ 需 trigger context | ⚠️ 仅 rule 触发范围 | ❌ |

### §3.1 校验路径

1. ActorType 识别：来自 JWT claim `actor_type` 或 MCP request `meta.actor_type`（per spec/mcp/01 §1.4 `actorType` enum）
2. 角色识别：`owner` 来自资源 creator_id 匹配 actor.id；`member` 来自 tenant membership；`public` 来自资源 `visibility=public` 字段
3. 范围校验：
   - Agent + ⚠️ 需 lease → 校验 `lease_id` 存在且未过期（per spec/agents/01 §2.1 L108-117）
   - Service + ⚠️ 需 service token → 校验 `service_token.scope` 覆盖目标资源
   - Automation + ⚠️ 需 trigger context → 校验 `automation_rule.trigger_event` 匹配当前事件

### §3.2 Audit 要求

- 所有 ⚠️ 条件 Read 必须写 `domain-audit`（`actor=current_actor` / `action=READ` / `target=resource_uri` / `result=ALLOW|DENY`）
- 公开字段 Read 可省略 audit（性能优化，默认批量读时跳过单条 audit）
- Human + owner 写 audit 保留 90 天；其他 ActorType 保留 365 天（per NFR-SEC-003 草案，未落 spec）

### §3.3 反例（denied Read 场景）

| 场景 | ActorType | 错误码 | 引用 |
|------|-----------|--------|------|
| Agent 读 `domain-audit://{id}` 公开字段 | Agent | `RESOURCE_FORBIDDEN` | §3 矩阵 Agent public ❌ |
| Service 读 `domain-tenant://{id}` 无 token | Service | `SERVICE_TOKEN_REQUIRED` | spec/mcp/03 §3 |
| Automation 读 `domain-lease://{id}` 缺 trigger | Automation | `TRIGGER_CONTEXT_MISSING` | spec/mcp/03 §3 |
| Human 读 `domain-policy://{id}` 跨 tenant | Human | `CROSS_TENANT_DENIED` | §6 #2 已知缺口 |

## §4 Write 权限矩阵

| ActorType | 创建 | 转换 | 删除 |
|-----------|------|------|------|
| Human | ✅ own | ✅ own | ⚠️ soft delete only |
| Agent | ⚠️ 需 task context | ⚠️ 需 lease | ❌ |
| System | ✅ | ✅ | ⚠️ audit log 必填 |
| Service | ⚠️ 需 token | ⚠️ 需 token | ❌ |
| Automation | ⚠️ 需 trigger | ⚠️ 需 trigger | ❌ |

### §4.1 转换校验

- 状态机校验：Write 触发的状态转换必须经过 [spec/agents/01 §1](../agents/01-agent-runtime-spec.md) L14-100 14 状态 PascalCase 状态机 + [spec/flows/01 §1](../flows/01-agent-task-lifecycle.md) Task.status 14 状态机双校验
- 错误码：非法转换返回 `INVALID_TRANSITION`（per spec/mcp/03 §3 错误模型 6 字段）
- 不可逆转换：终态 → 任何态返回 `TERMINAL_STATE_NO_REVIVE`（per spec/agents/01 §6 已知缺口 #5）

### §4.2 Soft delete 语义

- Human + 自身资源 + soft delete：`deleted_at` 字段置当前时间戳，资源不可见但保留 30 天（per NFR-DATA-007 草案）
- 30 天后由后台 job 物理删除 + 写 `domain-audit` 终态
- System + 物理删除：必须写 `domain-audit` 含 `reason` 字段 + 双签（per spec/agents/01 §5 F-06 `hint` 字段建议填双签 challenge）
- Agent / Service / Automation 一律 hard delete 禁止，软删也禁止

### §4.3 Write 状态机校验伪代码

```rust
// crates/star-mcp/src/write_guard.rs（草案,非实测行号）
fn pre_write_check(actor: ActorType, target: &ResourceUri, op: WriteOp) -> Result<()> {
    // 1. 资源存在性
    let current = target.fetch()?;  // per §2 URI 模式
    // 2. 权限矩阵 (per §4)
    check_write_matrix(actor, &current, op)?;
    // 3. 状态机校验 (per §4.1)
    if let WriteOp::Transition(new_state) = op {
        check_transition(&current.state, &new_state, &actor)?;
    }
    // 4. Lease 校验 (per §5)
    if actor == ActorType::Agent && op != WriteOp::Create {
        check_lease(actor.lease_id(), target)?;
    }
    Ok(())
}
```

## §5 与 spec/agents/01 §2 Lease 协议的关系

per [spec/agents/01-agent-runtime-spec.md §2 L104-141](../agents/01-agent-runtime-spec.md)（Lease 协议）：

- **Read 不持锁**：ActorType 任意可 read，**无需** lease（§3 矩阵所有 ✅ 行直接读）
- **Write 需 lease**：Agent 写操作必须持有对应资源的 lease（per spec/agents/01 §2.1 L108-117 6 字段）
  - 例：Agent 写 `domain-decision://{dec_id}` 必须持有 `lease.lease_id` 且 `lease.expires_at > now`
  - 例外：Agent 创建新资源（per §4 矩阵 `创建` 列 ⚠️ 需 task context）只需 task context 不需 lease，因为新资源尚无并发竞争
- **Transition 需 state machine 校验**：per spec/agents/01 §1 + spec/mcp/03-error-model-spec.md §3

### §5.1 错误码映射

| 场景 | 错误码 | 引用 |
|------|--------|------|
| Read 公开资源被拒 | `RESOURCE_FORBIDDEN` | spec/mcp/03 §3 |
| Read Agent 无 lease | `LEASE_REQUIRED` | spec/agents/01 §2.1 |
| Write Agent lease 过期 | `LEASE_EXPIRED` | spec/agents/01 §5 错误模型 L211 |
| 状态机非法转换 | `INVALID_TRANSITION` | spec/agents/01 §1.1 |
| Service token 越界 | `SERVICE_TOKEN_SCOPE_EXCEEDED` | spec/mcp/03 §3 |
| 软删资源再读 | `RESOURCE_SOFT_DELETED` | spec/mcp/03 §3 |

### §5.2 Phase F 实现状态

- Phase F（D6-D10）实现 22 crate trait 暴露 + 5 类资源（per ADR-0035 §2 D7 L66-78）
- Phase F+ 接入剩余 18 crate 的 Resources handler 路由（per spec/mcp/02 §1.6 L69-79 通用协议）
- Phase G 缓存层（D8 per ADR-0035 §2 L80-90）使用本 spec §2 URI 模式作为 cache key 模板

### §5.3 Read/Write 与 Lease 关系示例

```text
场景 1: Agent 读 domain-decision://D-100
  - lease 检查: skip (per §5 Read 不持锁)
  - 权限矩阵: Agent + tenant 共享 = ⚠️ 仅 task 范围
  - 校验: actor.task_id 必须 = D-100.task_id
  - 结果: ALLOW / DENY (LEASE_REQUIRED 不会触发,RESOURCE_FORBIDDEN 可能触发)

场景 2: Agent 写 domain-decision://D-100 (Transition: Pending → Decided)
  - lease 检查: required (per §5 Write 需 lease)
  - 校验: lease.lease_id 存在 + lease.expires_at > now
  - 状态机校验: Pending → Decided 在 14 状态 PascalCase 合法转换集合内
  - 结果: ALLOW / LEASE_EXPIRED / INVALID_TRANSITION
```

## §6 已知缺口

| # | 项 | 影响 | Phase F+ 计划 |
|---|----|------|---------------|
| 1 | domain-collaboration / domain-comment / domain-board 3 个非核心 22 domain crate 接入 | spec 范围仅 22 核心，3 协作类 crate 走 Phase F+ 追加 PR | per ADR-0035 §7 L232-250（v0.1 列出 6 项已知缺口，协作类未列）|
| 2 | 多租户隔离边界未明（cross-tenant resource access 模式）| Agent 跨 tenant 任务可能误读资源；当前 §3 矩阵未覆盖 | per ADR-0035 §7 L232-250 多租户隔离项 |
| 3 | Resource URI 跨 crate 引用（如 agent → workspace）权限校验 | 当前 §3 矩阵按"目标资源 crate"单维度校验，未考虑"引用链"上各节点权限 | Phase F+ 引入 `access_chain` 校验模式 |
| 4 | 大数据量 Resource 的分页 + Last-Event-ID 模式 | 当前 §2.2 URI 仅 `limit/offset`，无 SSE 续传锚点 | per spec/services/02 §4 Last-Event-ID 草案 |
| 5 | 实时变更推送（与 spec/services/02 SSE 集成点）| §2.2 subscribe 给出 channel id 但未列事件 schema | per spec/services/02 §3 SSE event schema |
| 6 | 离线缓存策略（与 Phase G 缓存层）| §2.2 URI 模式 + spec/mcp/01 §1.1 `ttlMs` 暂未给出每个 crate 的 `cacheScope` 默认值 | per ADR-0035 §2 D8 L80-90 缓存键 L3 维度 |

## §7 引用文档

- [adr/0023-version-control-provider.md](../../adr/0023-version-control-provider.md)
- [adr/0035-phase-f-architecture.md](../../adr/0035-phase-f-architecture.md)
- [spec/agents/01-agent-runtime-spec.md](../agents/01-agent-runtime-spec.md)
- [spec/mcp/02-resources-prompts-spec.md](../mcp/02-resources-prompts-spec.md)
- [spec/services/02-sse-streaming-spec.md](../services/02-sse-streaming-spec.md)

### §7.1 引用原则

- 本 spec §2 URI 模式与 spec/mcp/02 §1 Resources 协议对齐，URI 字符串解析统一在 `crates/star-mcp/src/resources.rs`（per ADR-0034 §3 关系图 L158-160）
- §3 / §4 权限矩阵与 spec/mcp/01 §1.4 `actorType` enum + spec/agent-api/01 §3 ActorType 对齐
- §5 错误码与 spec/mcp/03 §3 6 字段错误模型 + spec/agents/01 §5 错误模型对齐

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转 + 19:39 JST 代签授权升级 + 21:59 JST 第三次强化）| 初版：22 domain 数据源契约（§2 表 + §2.1 5 域映射 + §2.2 URI 模式 + §2.3 与 spec/mcp/02 §1 4 资源类关系）+ Read/Write 权限矩阵（§3 矩阵 + §3.1 校验路径 + §3.2 Audit 要求 / §4 矩阵 + §4.1 转换校验 + §4.2 软删语义）+ Lease 关系（§5 + §5.1 错误码映射 + §5.2 Phase F 实现状态）+ 6 已知缺口 + 引用文档 + 引用原则 | ADR-0035 D7 L66（22 domain crate 数据源契约任务）+ 2026-08-27 21:59 JST 用户授权第三次强化代签 |

---

> **审批者**：架构师 (Mavis 接手 agent per DEC-008) — 2026-08-27
> **per AGENTS.md §1 代签规则反转 + 2026-08-27 19:39 JST 代签授权升级 + 21:59 JST 第三次强化**：Mavis 接手默认代签 Ulysses 无需再问
