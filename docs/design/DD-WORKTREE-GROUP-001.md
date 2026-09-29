# DD-WORKTREE-GROUP-001

> **渡口 Project Worktree 管理与 Group Apps 详细设计 v0.1**
>
> - 状态：🟡 Draft（交互、上下文与安全契约已定义；后端实现与集成验收未完成）
> - 日期：2026-09-29
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 上位需求：[`docs/requirements.md`](../requirements.md) v2.4 §50
> - 上位基本设计：[`docs/basic-design.md`](../basic-design.md) v0.5 §16
> - 配套详细设计：[`DD-MULTICA-TASK-001.md`](DD-MULTICA-TASK-001.md) v0.3、[`DD-WORKTREE-CANVAS-001.md`](DD-WORKTREE-CANVAS-001.md) v1.3、[`DD-SHARED-TASK-001.md`](DD-SHARED-TASK-001.md) §11
> - 文档边界：本 DD 定义 Project → Worktree → Group Apps 的应用契约，不新增 WorktreeGroup / ProjectGroup 业务聚合，不宣称原型已具备生产授权、持久化或多 Agent 调度能力。

---

## §0 目的与完成判定

本设计把 Worktree 管理作为项目工作入口的核心：用户选定 Project 后比较和管理该项目的 Worktree；展开某个 Worktree 后进入以该 Worktree 为上下文的应用组。多 Agent 的归属、分支、运行状态、冲突、锁、PR 和最近活动必须集中可见；操作必须有明确授权、状态守卫、审计和幂等键。

生产完成须同时满足：

1. Index 的 Project 范围来自服务端授权，不接受客户端声明的 `tenant_id`、actor 或权限。
2. 所有 Group App API 先解析同一 `GroupContext`，对被引用实体再次授权。
3. WorkItem、Canvas、Agent Session、CLI、Chat、LangGraph 与 Plugin 通过 canonical ID、命令和事件互操作，不维护互相矛盾的事实副本。
4. 创建、归属调整、归档和清理 Worktree 可审计、并发安全，并能处理运行中 Agent、CLI 与 Git 锁。
5. 需求 §50 的 AC-WTG、AC-TCI、AC-CAN、AC-CHAT、AC-PLG 与 AC-TRACE 均通过服务端集成验收。

当前 UI 是基于浏览器 seed 的预览；WorkItem REST 路由的 Actor 注入、Project/Worktree ACL 与持久化仍是 release blocker。

## §1 范围与导航契约

### 1.1 产品树

```text
Project Selector
└─ Project Worktree Index
   └─ Worktree（展开后形成当前上下文）
      └─ Worktree Group Shell
         ├─ Multica Task Lifecycle
         ├─ Jira-class Board / Backlog / Sprint / Relation
         ├─ Task Card Index
         ├─ Infinite Canvas
         ├─ Workflow / LangGraph
         └─ Group Plugin Apps

Group Shell 固定底栏：Chat(scope = WORKTREE | GLOBAL)
Task Card 内：Agent Session / 受控 CLI
```

Task Card 与 Canvas 是 Group Apps 的直接同级入口。CLI 是 Task Card 内的执行面板；Agent Session 是执行事实，均不进入 Worktree 导航树。Group Chat 底栏仅在展开 Worktree 后显示，Project Index 不显示此底栏。

### 1.2 路由

| 路由 | 用途 | 解析规则 |
|---|---|---|
| `/worktree?project_id={project_id}` | 当前 Project 的 Worktree Index | `project_id` 只作为目标引用；服务端校验 Project membership。缺少参数时展示 Project Selector；本地预览可恢复用户已选项目，但生产 API 不以默认 Project 代替授权 |
| `/worktree/{worktree_id}/group?app={app_id}` | 指定 Worktree 的 Group Shell 与某一同级 App | Project、Repository、Workspace、Tenant 均从已登记 Worktree 服务端解析；URL 中的 `app_id` 不授予能力 |
| `/worktree/{worktree_id}/group?app=task-card&work_item_id={id}&cli=1` | 打开 Task Card，可选打开 CLI 面板 | 任务与 Worktree 关联、权限和 CLI policy 必须由服务端检查；没有关联时只允许查看或发起受权关联流程，不启动 CLI |

Project 页的 Worktrees 入口必须带上 `project_id`。Index 和 Group 的深链刷新后仍须恢复同一范围；`/worktree` 不得重定向至 Sprint 或通用任务树。前端导航状态可缓存，但不作为授权依据。

### 1.3 当前 UI 与目标状态

| 区域 | 预览实现 | 生产目标 |
|---|---|---|
| Project Index | Zustand seed 过滤 | 授权 Project 的分页 API + Worktree 投影 |
| Worktree 归属 | 当前 AgentSession / Runtime 引用 | 独立 `owner_user_id`、AgentSession 当前绑定及历史会话 |
| Worktree 操作 | 不提供创建或清理按钮 | 按 §6 的授权动作、状态守卫和 Audit |
| Group Apps | 本地路由切换 | App Registry + 当前 GroupContext + 共享实体引用 |
| Chat / CLI / Plugin | 控件预览或禁用 | 授权 API、真实 Runtime 或 Plugin Gateway |

## §2 模块与责任边界

| 模块 | 输入 | 责任 | 不负责 |
|---|---|---|---|
| `ProjectSelector` | 已认证 actor 的 Project 列表 | 选择 Project、清除旧 Worktree selection、写入可分享路由 | 自行判断 membership |
| `ProjectWorktreeIndex` | `project_id` + 服务端投影 | 比较 Worktree 运行信号、显示风险并提供允许的管理动作 | 创建第二份 Worktree 状态事实 |
| `WorktreeGroupShell` | `worktree_id` + `GroupContext` | 解析同级 App、当前 Worktree Header、唯一 Chat 底栏与路由状态 | 充当新的领域聚合根 |
| `GroupAppRegistry` | 原生 App registry、Plugin manifest、actor grants | 生成同级 App 节点及 capability 集合 | 以 UI 隐藏代替服务端授权 |
| `GroupContextResolver` | Authenticated Actor、目标 Worktree、membership/ACL | 从持久事实建立上下文并返回授权快照 | 接受客户端上传的 actor 或 tenant 身份 |
| `EntityRefResolver` | typed EntityRef + GroupContext | 解析实体并校验实体归属、版本和权限 | 仅凭 entity ID 授权 |
| `GroupCommandGateway` | 命令、GroupContext、幂等键 | 调用事实所有者 Domain，写事务与 Outbox | 在前端直接改另一个 App 的 store |
| `GroupProjection` | Outbox / 领域事件 | 更新 Index、Group App 和订阅投影 | 覆写事实所有者的状态 |

Worktree Domain 持有 Worktree lifecycle 与 checkout 引用；WorkItem Domain 持有任务事实；Canvas Domain 持有画布；Agent / Runtime Domain 持有执行与终端会话；Workflow Domain 持有流程状态；Plugin Registry 持有 App manifest / enablement；Audit 持有不可变审计事实。

## §3 核心 DTO 与不变量

### 3.1 Project Worktree Index item

```text
WorktreeIndexItem {
  worktree_id, project_id, repository_id,
  display_name, branch, base_branch,
  lifecycle_status, health_status,
  owner_user_id?, owner_display_name?,
  active_agent_session_id?, active_agent_name?, active_agent_status?,
  active_runtime_id?, runtime_status?,
  pull_request_ref?, conflict_summary?, lock_summary?,
  recent_activity_at?, recent_activity_kind?,
  lock_version, allowed_actions[]
}
```

`owner_user_id` 表示人类责任归属；`active_agent_session_id` 表示当前自动化执行，二者不得互相替代。历史 Agent Sessions 通过分页详情 API 获取。缺失字段显示“未绑定/未知”，不得根据分支名或运行进程推断 owner。

Index 行上的状态由独立字段组成：Worktree lifecycle、健康探测、Agent Session、Runtime、Git 冲突、Git lock 与 PR 状态。不得将 WorkItem status 当成 Worktree status，也不得把没有冲突证据显示成“无冲突”。

### 3.2 GroupContext

```text
GroupContext {
  tenant_id, workspace_id, project_id, repository_id, worktree_id,
  actor_id, actor_kind, actor_roles[], granted_scopes[],
  context_version, resolved_at, correlation_id
}
```

GroupContext 是服务端解析结果，只能在一次已授权请求中使用。缓存键必须包括 actor、Project/Worktree 身份与授权版本；membership、角色、Worktree 归属或 Plugin grant 被撤销后，相关缓存与实时订阅须失效。它不能从浏览器 Zustand、URL 参数或 LangGraph checkpoint 恢复为可信凭据。

### 3.3 EntityRef

跨 App 使用 `EntityRef { entity_type, entity_id, tenant_id, project_id, worktree_id?, version? }`。`entity_id` 是寻址字段，不是授权 token。resolver 必须核对实际实体与传入的 tenant/project/worktree 一致；引用过期版本返回冲突，跨项目或越权引用返回统一不可见响应，避免泄漏实体存在性。

## §4 API 契约

所有 `/api/v1` 业务 API 使用已验证 Bearer JWT。响应使用统一 REST envelope 和 cursor 分页；每个写命令带 `Idempotency-Key`、`If-Match` 或显式 `expected_version`、`correlation_id`。请求体不得接受 `actor_id`、`tenant_id`、`roles` 或 `granted_scopes` 作为授权来源。

| Method / path | 用途 | 必要授权 | 关键结果 |
|---|---|---|---|
| `GET /api/v1/projects/{project_id}/worktrees` | Index 分页与筛选 | `project:read` + Project membership | `WorktreeIndexItem[]`, `next_cursor`, `snapshot_version` |
| `GET /api/v1/worktrees/{worktree_id}/group-context` | 打开 Group 前取得服务端上下文与 App grants | `worktree:read` + Project membership | `GroupContextProjection`, `apps[]`, `allowed_actions[]` |
| `POST /api/v1/projects/{project_id}/worktrees` | 受权注册/启动 Worktree | `worktree:create` + Repository access | `worktree_id`, lifecycle operation ref |
| `POST /api/v1/worktrees/{worktree_id}/owner` | 变更人类 owner | `worktree:manage` + 目标用户 Project membership | 新 owner、版本、Audit ref |
| `POST /api/v1/worktrees/{worktree_id}/archive` | 可恢复归档 | `worktree:manage` | `operation_id`, `new_version`, 资源保留策略 |
| `POST /api/v1/worktrees/{worktree_id}/cleanup/plan` | 生成清理预检 | `worktree:cleanup` | 只读检查、阻断原因、短期 `plan_id` |
| `POST /api/v1/worktrees/{worktree_id}/cleanup/confirm` | 执行过期清理计划 | `worktree:cleanup` + 二次确认 | 幂等 operation result 与 Audit ref |
| `GET /api/v1/worktrees/{worktree_id}/agent-sessions` | 查询 Agent Session 历史 | `agent-session:read` + 同一 GroupContext | cursor 分页与当前/历史标记 |

WorkItem 的读写继续走 WorkItem Domain API。Index / Group API 不绕过 WorkItem Command Port，不新增第二个 WorkItem endpoint owner。当前 REST work-item route 仍用 `auth_layer_stub`、默认 `ActorContext` 和 `InMemoryWorkItemService`，因此在这些实现替换前不作为生产数据源。

### 4.1 错误映射

| 情况 | HTTP | 处理 |
|---|---:|---|
| 缺少 / 无效 / 过期 token | 401 | 返回认证错误与可重试标志；不进入业务查询 |
| 缺少 scope 或 membership | 403 或统一 404 | 按安全策略不泄漏对象存在性 |
| 版本过期 / lock_version 不匹配 | 409 | 返回最新版本引用，要求客户端重新加载 |
| 幂等键重复 | 原操作状态码 | 返回同一 operation 结果，不重复创建副作用 |
| 已有 Agent / CLI / Git 操作阻止归档清理 | 409 | 返回明确、无敏感内容的阻断类型 |
| 资源依赖不可用 | 503 | 可重试字段与 correlation ID；不伪造成功 |

## §5 认证与 GroupContext 授权流程

```text
HTTP request
  → Bearer JWT verification (signature / issuer / audience / expiry)
  → Authenticated Actor from verified claims
  → resolve target Worktree → Repository → Project → Workspace → Tenant
  → check Project membership + required scope + resource state
  → construct request-scoped GroupContext
  → resolve and authorize each EntityRef / action / target
  → Domain command/query
```

强制规则：

1. `actor_id` 来自已验证 token；若 `sub` 与显式 `user_id` claim 不一致，拒绝 token。JWT roles/scopes 只表达签发时身份能力；项目级资源访问仍须检查当前 membership / ACL。
2. 从登录 Project Worktrees 入口选中的 Project 与 Worktree 的真实归属必须一致。请求指定的 `project_id` 只能缩小范围，不可覆盖 Worktree registration 中的 Project。
3. 每个 GET、命令、WebSocket/SSE 订阅、LangGraph start/resume 与 Plugin capability 调用都重新做必要授权检查。GroupContext cache 和 checkpoint 不能延长已撤销 grant。
4. 不存在默认 `developer` actor、nil tenant/user 或匿名 fallback。认证服务不可用时 fail closed。
5. 批量操作按目标独立授权，结果记录成功、拒绝、冲突或可重试状态；任一目标失败不得自动将其他目标改成成功。
6. 安全日志记录 actor、目标、scope、decision、correlation ID；不记录 Bearer token、私密 prompt、CLI secret 或未脱敏终端输出。

## §6 Worktree 管理、并发与 Agent 协调

### 6.1 Index 管理动作

每个 Worktree 返回 `allowed_actions[]`；前端按该字段展示动作，但服务端在执行时再次授权。Phase 1 只读展示，Phase 2 起实现动作。

| 动作 | 预检 | 写入规则 |
|---|---|---|
| 创建 / 导入 | Project 与 Repository 可访问；路径 / 分支不冲突；Runtime 可达 | Idempotency-Key，Domain 创建 operation，写 Audit / Outbox |
| 转派 owner | 目标用户仍是该 Project member；无策略冲突 | 乐观锁 `lock_version`；记录 before/after 与 actor |
| 归档 | 无未处置的会话；PR/任务/Canvas 引用按保留策略处理 | 先标记 archived 并停止新执行；不物理删除业务事实 |
| 清理 | 先跑 plan：Agent / CLI inactive、Runtime detached、Git lock absent、保留策略满足 | 仅使用未过期 plan；二次确认、幂等执行、审计实际 Git 操作 |
| 取消 / 恢复 | operation 与当前 lifecycle 匹配 | 版本条件更新，记录恢复来源和失败原因 |

归档与 Git worktree 物理清理是不同操作。存在 active run、未释放 Runtime、未知 Git lock 或无法查询状态时，清理一律拒绝；不通过“强制”开关绕过。清理预检需短 TTL；计划过期后必须重新计算。

### 6.2 并行 Agent 可见性

Index 同时呈现一个 Worktree 的 owner、当前 Agent、活跃 Runtime、PR 和风险信号；执行历史由 Session 列表展开。一个 WorkItem 可关联 `0..N` Worktrees；每次 Agent / CLI run 必须选择一个授权 Worktree，并在 Run 上记录 `active_worktree_id`。Agent 不能因 Task Card 关联而获得其它 Worktree 的读取权。

冲突检测分为 Git merge/rebase 冲突、同一 Worktree 的并发写入占用和资源健康错误；三者分别表示，不用单个“locked”状态混写。UI 展示最后一次检测时间与 source version；过期探测显示“未知/待刷新”。

## §7 Task Card、CLI 与 Multica/Jira 联动

1. WorkItem ID 是 Multica、Jira 视图、Task Card 与 Canvas 共同的 canonical ID；Task Card 不复制状态机。详情、负责人、状态、priority 与 lock version 从 WorkItem projection 读取。
2. Task Card 打开时展示当前 Group Worktree。若 WorkItem 未关联当前 Worktree，允许查看并提示关联/选择 Worktree；在关联确认之前不允许启动 CLI、Agent 或写入 Worktree 文件。
3. 服务端构造 `TaskExecutionContext { actor, project_id, repository_id, worktree_id, work_item_id, runtime_id, approved_profile_id, policy_version, expiry, nonce }`。路径、命令白名单、环境变量和 Secret 句柄均由服务端 policy 与 Runtime 配置派生，客户端不能提交可覆盖值。
4. Local Runtime 只接受短期、签名且一次性的 session grant；启动、输入、退出和 session 重连均重新验证范围。审计记录 session ID、命令类别、结果摘要和退出码，不把任意终端内容复制进 WorkItem 描述。
5. Multica 维护 execution / review / poison / stale-dispatch 专属生命周期；Jira Board/Backlog/Sprint 展示可规划的 WorkItem 字段。两者通过 canonical WorkItem ID 与领域事件更新 Task Card，不能彼此覆盖状态事实。

## §8 Canvas、Chat、LangGraph 与 Plugin 的交互契约

### 8.1 Canvas

Group Canvas 与 Task Card、Multica、Jira 同级；Canvas Element 仅持有带类型 `EntityRef`，不持有 WorkItem 状态副本。双击任务 Element 导航到 `app=task-card&work_item_id={id}`；“创建任务”“变更状态”“建立关系”须走实体 owner 的 Application Command，再由 Outbox 更新 Canvas 与其它投影。Project 级 Worktree Overview Graph 与 Worktree Group Infinite Canvas 分路由、分查询范围、分用户目的。

### 8.2 Group Chat 与 LangGraph

Chat scope 被显式写入请求与 checkpoint：

```text
GroupChatCommand {
  scope: WORKTREE | GLOBAL,
  worktree_id?: UUID,
  target_worktree_ids?: UUID[],
  message, entity_refs[], expected_context_version,
  idempotency_key, correlation_id
}
```

`WORKTREE` 必须等于当前已授权 `worktree_id`。`GLOBAL` 先查询 actor 可访问目标，再由用户明确选择目标集合；不得默认扩展为整个 Tenant。每个目标独立校验并产生子任务结果，允许部分失败但必须呈现逐目标结果。

LangGraph checkpoint 保存 thread/run 关联和非敏感状态，不保存可复用授权快照、JWT 或长期 Secret。每次 start、resume、interrupt approval 和 tool call 都重新解析 GroupContext 与 capability。LangGraph thread ID 与 WorkItem、Task Card、Agent Session ID 分离。

### 8.3 Plugin 热插拔

原生 App 与 Plugin App 使用同级 registry 节点和统一 shell。Plugin manifest 声明 publisher、版本、host API 兼容区间、capabilities、事件订阅与 UI entry；Registry 的 enablement 不是 capability grant。每次 capability call 由 Gateway 按当前 actor、Project、Worktree 与 plugin grant 授权。

disable/uninstall 后新调用立即拒绝，活动调用按独立 ADR 定义 cancel 或 drain；业务 Transaction、Audit 和 Canvas EntityRef 保留。发布者签名信任根、沙箱承载和 upgrade rollback 是 release blocker，不能由 UI 原型开关代替。

## §9 跨 App 事件与一致性

业务写入使用事实 owner 的 Domain Command，在同一数据库事务提交业务事实与 Outbox；投影与通知异步、幂等消费。事件至少包含：

```text
event_id, event_type, schema_version,
tenant_id, project_id, worktree_id?,
entity_ref, aggregate_version, actor_id,
correlation_id, occurred_at, payload
```

事件不携带 Bearer token、长期 Secret 或不必要的终端输出。消费者按 `event_id` 去重，并按 `aggregate_version` 丢弃迟到更新；同一事件可重试，但重复投影不得重复产生用户操作。前端订阅按 GroupContext 过滤，Project membership 撤销时关闭对应连接。

| 事实变化 | 事实 owner | 需要更新的投影 |
|---|---|---|
| Worktree 创建、状态、owner 或健康探测变化 | Worktree Domain | Project Index、Group Header、Graph Overview、Audit |
| WorkItem 状态、负责人或 Relation 变化 | WorkItem / Workflow Domain | Multica、Jira、Task Card、Canvas、Chat / Plugin subscribers |
| Agent / CLI run 状态变化 | Agent / Runtime Domain | Index、Task Card、Canvas cursors、Multica lifecycle |
| Canvas Element / EntityRef 变化 | Canvas Domain | Group Canvas、被授权的实体链接视图 |
| Plugin enablement / grant 变化 | Plugin Registry / Authorization | Group App Registry、active subscriptions、Audit |

## §10 持久化分类约束（W/T/M）

本 DD 不新增物理表；下列为落库时的主分类，DDL 必须在 Data Design 阶段对每张表逐表确认，不能照搬 UI projection：

| 数据事实 | 主分类 | 约束 |
|---|---|---|
| Worktree 注册元数据与人类 owner 历史 | Master | SCD Type 2；物理删除禁止；RLS 13 类必携 |
| Agent / CLI run、审计、Outbox、状态变更历史 | Transaction | Append-only；物理删除禁止；Audit + RLS 13 类必携 |
| 当前 Runtime lease、临时健康探测、清理预检 plan | Work | 显式 `retention_period` / expiry；允许按 TTL 清除，不承载唯一业务事实 |
| GroupContext | Derived projection（不持久化授权快照） | 需要缓存时为 Work/短 TTL；撤权后失效，不可恢复授权 |
| Plugin manifest / Project enablement / capability grant | Master | 版本化、审计变更；撤权立即作用于新调用 |
| Chat message / LangGraph run checkpoint | Transaction + Work 分表设计 | 用户可见消息保留策略独立于可过期 checkpoint；两者不可混成一个无期限 blob |

具体数据库表名、外键、索引、RLS、保留期和迁移脚本由 Data Design + Security Design 评审冻结；每张新增表都须单独标注 W/T/M 与派生规则。

## §11 关键验收映射

| Requirement / AC | 详细设计落点 | 服务端验收重点 |
|---|---|---|
| WTG-001 / AC-WTG-001 | §1-§5 | Project membership 限定 Index；同级 App 树与唯一 GroupContext |
| WTG-004 / AC-WTG-003 | §3、§6 | owner、Agent、Runtime、PR、冲突/锁和最近活动来自有版本的服务端投影；操作有授权与 Audit |
| WTG-005 / AC-WTG-002 | §1-§5、§9 | Project/Worktree 切换清空旧订阅；跨项目实体引用拒绝 |
| WTG-006 / AC-WTG-004 | §1.2 | Index 与 Group 深链独立、刷新稳定，不落入通用任务列表 |
| TCI-001 / AC-TCI-001 | §7 | Multica/Jira/Task Card 同一 WorkItem ID 和版本 |
| TCI-002 / AC-TCI-001 | §7 | CLI 只在正确 Worktree、授权 profile 与 Runtime 运行 |
| CAN-001 / AC-CAN-001 | §8.1 | Canvas 双向导航引用 canonical WorkItem，写入经过 Domain Command |
| CHAT-001 / AC-CHAT-001 | §8.2 | WORKTREE 与 GLOBAL scope、目标 ACL、部分失败逐目标可见 |
| PLG-001 / AC-PLG-001 | §8.3 | 热撤权对新调用即时生效，活动调用有确定终止语义 |
| ARCH-OBL-GRP-001 / AC-TRACE-001 | §4-§10 | 一个 correlation ID 串起授权、命令、Outbox、投影与 Audit |

## §12 实施顺序与阻断门

1. **2B 认证 Actor 与 GroupContext**：把已验证 Authenticated Actor 注入业务请求；移除默认 Actor；建立 Project membership 和 Worktree ownership resolution；未经授权的 Index / Group API fail closed。
2. **2C 持久化与 WorkItem closure**：实现 WorkItem PostgreSQL repository / migration，对齐 Multica lifecycle、WorkItem status 与 W/T/M；先实现 list/get/transition，再启用 UI 读写。
3. **2D Worktree Index 投影与安全管理**：接入 owner、Session、Runtime、PR、冲突/锁和 Audit；先实现 read-only，再按 §6 打开受权操作。
4. 每个子阶段通过权限负向用例、跨 Tenant/Project 隔离、并发版本冲突、幂等重试、审计和真实 API 集成验收后，才能启用下一项写能力。
5. Group UI 在 2B 与 2C 完成前保持 `preview / seed / not connected` 提示；不把浏览器 store 的变化作为产品事实。

## §13 已知缺口与实现前置

| # | 缺口 | 影响 | 关闭条件 |
|---|---|---|---|
| 1 | REST work-item routes 使用 no-op `auth_layer_stub`、默认 actor 和 `InMemoryWorkItemService` | API 不能承载生产 Group App 数据 | Actor extractor、Project ACL、持久 repository 落地并通过负向验收 |
| 2 | WorkItem Domain 只有三态 `InMemoryWorkItemService`；Multica 详细设计使用六态 lifecycle 和独立 review state | 命令与迁移映射不一致 | 确认 canonical 写模型、迁移来源和状态映射后实现 PG repository |
| 3 | Worktree projection 缺 `owner_user_id`，当前列表只读出 session/runtime 引用 | 多 Agent 人类责任归属不可控 | 指定 owner 事实源、owner 转派审计与历史查询 API |
| 4 | `GroupContextResolver`、EntityRef 授权与统一 `allowed_actions` API 未落地 | 跨 App 可能出现上下文混用 | middleware / application port 与跨 Project 拒绝验收 |
| 5 | 清理操作的 Git lock 探测、agent/terminal drain 和恢复策略未定 | 清理可能破坏活动工作 | Worktree cleanup ADR + plan/confirm API + 故障恢复演练 |
| 6 | Canvas Group 数据仍由 Project seed 取得 | Canvas 与当前 Worktree 范围不一致 | Worktree-scoped query/migration 完成，跨 Worktree overview 保持独立 |
| 7 | LangGraph SDK/checkpointer 和 Plugin sandbox/revocation ADR 未冻结 | 无法安全恢复流程或热拔除插件 | SDK/API compatibility review 与运行时撤权演练通过 |

## §14 审阅栏与修订履历

| 角色 | 状态 |
|---|---|
| 架构 | Draft；Mavis 接手审核，待 Phase 2B/2C 实证后复审 |
| SRE Lead | 待详细验证：故障切换、事件积压与 Runtime 清理恢复 |
| 平台 | 待详细验证：JWT state 注入、ACL、RLS 与部署配置 |
| 评审主持 | Draft；实现前检查跨 App 事件和错误映射 |
| PM | Draft；确认 Index 管理动作与发布范围 |

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Project → Worktree → Group Apps 详细交互、安全上下文、管理操作、CLI、Canvas、Chat/LangGraph、Plugin、事件与 W/T/M 实施门；记录 REST no-op auth 与 in-memory blocker | 推进 Worktree Group Phase 2，并依据 Worktree Index 当前实现核对 API 安全边界 |
