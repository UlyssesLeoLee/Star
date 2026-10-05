# DD-WORKTREE-GROUP-001

> **渡口 Project Worktree 管理与 Group Apps 详细设计 v0.13**
>
> - 状态：🟡 Draft（Phase 2B/2C/2D 与 Phase 3B Canvas persistence/API 代码切片已实现；Phase 3C 增加 Outbox offset/read API、Canvas→WorkItem 原子命令、同源 BFF、Canvas Document viewport/frames/connectors 的 SCD2 更新 API 与 Group 页面读取/viewport 保存/Canvas 创建；Phase 4 增加 Local Runtime TaskExecutionContext 与 checkout path gate 且 crate 编译通过；统一登录 cookie 签发、数据库部署、Outbox consumer/checkpoint/realtime、Canvas element/frame/connector 编辑、PTY/UI 会话、ACL/RLS 负向验收与历史归属回填待完成）
> - 日期：2026-09-29
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 上位需求：[`docs/requirements.md`](../requirements.md) v2.8 §50
> - 上位基本设计：[`docs/basic-design.md`](../basic-design.md) v1.0 §16
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

UUID Project Index 与 UUID Worktree Group 页面已接入同源 BFF 的只读 API projection；Index manager 可提交归档/恢复 plan-confirm，非 UUID ID 继续走明确的 seed preview。统一登录 cookie issuer 尚不存在，数据库 migration 未应用、Project membership 未初始化、负向集成验收未运行，因此这些接口仍是未部署代码切片，不可视为已启用生产能力。

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
  worktree_id, project_id, workspace_id, repository_id,
  name, path, parent_id?, branch, human_state, machine_state,
  owner_user_id?, work_item_id?, agent_id?, agent_session_id?, runtime_id?,
  ahead, behind, dirty, health_score, test_state, risk_count,
  locked, archived, pull_request_url?, version, created_at, updated_at
}
```

当前 API 投影只返回上述持久字段；Agent / Runtime 展示名、运行状态、冲突摘要、锁来源和历史 Session 列表仍待专用权威数据源。`owner_user_id` 表示人类责任归属；`agent_session_id` 表示会话关联，二者不得互相替代。缺失字段显示“未绑定/未知”，不得根据分支名或运行进程推断 owner。

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

所有生产 Group API 使用已验证 Bearer JWT。浏览器经同源 `/api/group/api/v1/...` BFF 请求 Group API；BFF 只代理 `GET /projects/{project_id}/worktrees` 与 `/worktrees/{worktree_id}/...`，并限制 GET/POST/PUT/PATCH/DELETE 方法和 2 MB 请求体；从 HttpOnly Cookie (`GROUP_ACCESS_TOKEN_COOKIE`，默认 `star_access_token`) 读取由认证流程签发的 token，并仅由服务器注入上游 Authorization 头。写请求要求 Origin 与当前站点一致；上游地址由服务端 `GROUP_API_BASE_URL` 配置，token 与 upstream URL 不接受浏览器输入，重定向不跟随。BFF 未实现登录、cookie 签发或刷新；无 cookie 返回 401、未配置 upstream 返回 503、上游不可达返回 502。BFF 不替代 Group API 的 JWT、membership、Worktree 与 capability 校验。Index 使用有界 cursor 分页；Task Card 列表当前是有界 limit。成功响应目前为 JSON projection，统一 envelope 尚未接入。每个写命令须带 `Idempotency-Key`，状态转移 / Worktree 管理另带 `expected_version`；未提供 `correlation_id` 时由服务端生成。请求体不得接受 `actor_id`、`tenant_id`、`roles` 或 `granted_scopes` 作为授权来源。

| Method / path | 用途 | 必要授权 | 关键结果 |
|---|---|---|---|
| `GET /api/v1/projects/{project_id}/worktrees` | Index 游标分页与 owner/state/archive 筛选 | `project:read` + Project membership | Worktree 投影、`next_cursor` |
| `GET /api/v1/worktrees/{worktree_id}/group-context` | 打开 Worktree Group 前解析当前上下文 | `worktree:read` + Project membership | `group_context` 与 Worktree 投影；尚无 `apps[]` / `allowed_actions[]` |
| `POST /api/v1/worktrees/{worktree_id}/work-items` | 创建 Task Card / canonical WorkItem | `work-item:write` + Project writer role | 任务、Task Card 同 ID，初始 `pending`, version 1 |
| `GET /api/v1/worktrees/{worktree_id}/work-items` | Task Card / Multica 当前 Worktree 任务列表 | `work-item:read` + Project membership | 有界列表、canonical `work_item_id` 与 lifecycle version |
| `GET /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}` | 读取 Task Card | `work-item:read` + 当前 Worktree association | metadata + Multica lifecycle projection |
| `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/lifecycle` | 更新 Multica 六态 | `work-item:write` + Project writer role | version 检查、幂等响应、Transaction audit |
| `GET /api/v1/worktrees/{worktree_id}/canvases` | 查询当前 Worktree 的 Canvas | `worktree:read` + `canvas:read` + Project membership | 仅返回当前 Worktree registry 与元素数量 |
| `POST /api/v1/worktrees/{worktree_id}/canvases` | 创建 Worktree-owned Canvas | `worktree:read` + `canvas:write` + Project writer role | `Idempotency-Key`；Canvas Master version 1；同事务 Audit / Outbox |
| `GET /api/v1/worktrees/{worktree_id}/canvas-events?after_offset={offset}&limit={n}` | 读取当前 Worktree 的 Canvas Outbox 增量页 | `worktree:read` + `canvas:read` + Project membership | 默认 50、上限 100；offset 与 next_offset 以十进制字符串传输；`has_more` 提示续页 |
| `POST /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/work-items` | 从 Canvas 创建 canonical WorkItem 和任务卡元素 | `canvas:write` + `work-item:write` + `work-item:read` + Project writer role | 单事务写入 Task metadata、Multica lifecycle、Worktree relation、Canvas Element/EntityRef、两类 Audit、Outbox 与幂等响应 |
| `GET /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements` | 读取 Canvas 元素和当前可解析 EntityRef | `worktree:read` + `canvas:read` + Project membership | stale/越界 WorkItem 关联不解析；无 `work-item:read` 时隐藏 WorkItem ref；详情由 canonical WorkItem API 授权读取 |
| `POST /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements` | 创建 Canvas 元素，可同时绑定 typed EntityRef | `worktree:read` + `canvas:write` + Project writer role；绑定 WorkItem 另需 `work-item:read` | `Idempotency-Key`；元素/EntityRef Master 与 Audit / Outbox 同事务 |
| `PUT` / `DELETE /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements/{element_id}` | 更新布局或逻辑移除元素 | 同上，另带 `expected_version` | SCD2 版本条件更新；DELETE 关闭当前版本，不物理删除 |
| `PUT` / `DELETE /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements/{element_id}/entity-ref` | 设置、替换或解除元素的 typed EntityRef | 同上，另带 EntityRef `expected_version`；绑定 WorkItem 需 `work-item:read` | WorkItem 必须是同 tenant/project 且当前关联当前 Worktree 的 canonical WorkItem |
| `POST /api/v1/worktrees/{worktree_id}/management-plans` | 预览 owner reassignment 或归档 / 恢复 | `worktree:manage` + `tenant_admin` / `project_admin` | 5 分钟 plan；必须带 `Idempotency-Key` 与当前 version |
| `POST /api/v1/worktrees/{worktree_id}/management-plans/{plan_id}/confirm` | 确认 Worktree owner / archived 变更 | 同一 requester、当前 membership、原 version | 版本条件更新、Owner SCD2、不可变 Audit |
| `POST /api/v1/worktrees/{worktree_id}/cleanup/plan` | 生成清理预检 | `worktree:cleanup` | 只读检查、阻断原因、短期 `plan_id` |
| `POST /api/v1/worktrees/{worktree_id}/cleanup/confirm` | 执行过期清理计划 | `worktree:cleanup` + 二次确认 | 幂等 operation result 与 Audit ref |
| `GET /api/v1/worktrees/{worktree_id}/agent-sessions` | 查询 Agent Session 历史 | `agent-session:read` + 同一 GroupContext | cursor 分页与当前/历史标记 |

新增 Group Task API 在一个 PostgreSQL transaction 内协调 Task metadata、Multica lifecycle、Worktree association、idempotency 和 audit；任务读写同时校验授权 Worktree、关系 Project 与 canonical metadata Project 一致，阻断 tenant 内跨 Project 的错误关联。Worktree 与 Project binding 在请求事务中加共享锁，避免上下文解析期间改变归属。legacy `/api/v1/work-items` 仍是 no-op auth / 默认 Actor / `InMemoryWorkItemService`，生产二进制不挂载该旧路由。当前协调逻辑仍在 REST crate，尚未抽成 WorkItem Command Port / PostgreSQL adapter；因此 2C 是可编译的持久化实现切片，不等于 Domain 架构门已关闭。Jira 外部同步与 alias adapter 仍待后续阶段。

### 4.1 错误映射

| 情况 | HTTP | 处理 |
|---|---:|---|
| 缺少 / 无效 / 过期 token | 401 | 返回认证错误与可重试标志；不进入业务查询 |
| 缺少 scope 或 membership | 403 或统一 404 | 按安全策略不泄漏对象存在性 |
| 版本过期 / lock_version 不匹配 | 409 | 返回最新版本引用，要求客户端重新加载 |
| 幂等键重复 | 原操作状态码 | 返回同一 operation 结果，不重复创建副作用 |
| 已有 Agent / CLI / Git 操作阻止归档清理 | 409 | 返回明确、无敏感内容的阻断类型 |
| 资源依赖不可用 | 503 | 可重试字段与 correlation ID；不伪造成功 |

### 4.2 Phase 2B/2C/2D 已实现的生产路由（部署门未关闭）

生产二进制只挂载下列 Group API；保留的 build_router() 是旧路由骨架，不能用于生产 Group Shell 数据源。

| Method / path | 验证 | 实际行为与边界 |
|---|---|---|
| GET /api/v1/projects/{project_id}/worktrees | RS256 Bearer；project:read；active Project Role Binding | 服务端 tenant + project 过滤；owner/state/archive 筛选；按 `(updated_at,id)` 倒序 cursor 分页，cursor 绑定当前查询参数 |
| GET /api/v1/worktrees/{worktree_id}/group-context | RS256 Bearer；worktree:read；当前 Project membership | tenant-scoped Worktree 投影、Actor、成员角色、granted scopes、membership version 和 permission snapshot ref；当前不返回 App Registry / allowed actions |
| POST /api/v1/worktrees/{worktree_id}/work-items | RS256 Bearer；work-item:write；developer / project_admin / tenant_admin / agent membership role | 同一事务创建 canonical WorkItem metadata、pending lifecycle、Worktree association、created audit 和幂等结果；`task_card_id = work_item_id` |
| GET /api/v1/worktrees/{worktree_id}/work-items[/{work_item_id}] | RS256 Bearer；work-item:read；当前 Project membership | 当前 Worktree association 限定 Task Card 读列表或详情；列表最多 100 条 |
| POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/lifecycle | RS256 Bearer；work-item:write；writer membership role | Multica 六态合法转移、active Worktree 绑定、`expected_version`、`Idempotency-Key`、correlation ID 和 append-only audit 在一个事务中处理；claim lease 与终态期限使用 PostgreSQL 时钟，completed/failed/cancelled 设置保留期限，failed → pending 清除终态期限 |
| POST /api/v1/worktrees/{worktree_id}/management-plans | RS256 Bearer；worktree:manage；tenant_admin / project_admin | 只允许转派 owner 或设置 archived；检查目标 owner membership、Worktree version；确认截止时间由 PostgreSQL 时钟生成，5 分钟确认窗 |
| POST /api/v1/worktrees/{worktree_id}/management-plans/{plan_id}/confirm | RS256 Bearer；同一 requester + 当前 membership + expected version | 同一事务更新 Worktree projection、owner SCD2、管理 Transaction audit 与 plan 状态；已绑定 AgentSession/Runtime 时拒绝归档 |
| 错误处理 | invalid actor / scope / UUID / resource / version | 未授权成员与不存在目标统一 404；缺 scope 403；冲突 409；数据库错误为不含 SQL/行内容的通用 500 |

### 4.3 Phase 3B Canvas 路由代码切片（尚未部署）

上述 Canvas endpoints 已接入 Group router 并通过 crate 编译，但对应 Phase 3 migration 尚未应用到目标数据库，不能视为已部署能力。它们复用当前 Worktree Project binding、active membership、writer role 和事务级 RLS scope；所有写入需要 `Idempotency-Key`，元素与 EntityRef 更新需要 `expected_version`。EntityRef 只暴露 canonical type/ID/scope；当前调用者缺少 `work-item:read` 时，Canvas 元素响应不返回 WorkItem ref，任务详情须再通过 canonical WorkItem API 校验。

AuthenticatedUser 拒绝 sub 与 user_id 不一致或 nil tenant/user。JWT roles 不作为 Project ACL；角色从 permission.project_role_binding 实时读取。每次查询在事务内设置 app.tenant_id，并显式加 tenant/project 条件以配合 Worktree RLS。成员表尚无 Project 外键（仓库尚无 PostgreSQL Project SoR）；现存 Worktree 的 project_id 为 NULL 时不会进入 Index，不能从 task_id 或 branch 猜测。

Phase 2B/2C/2D migrations 均为 additive；不运行旧的破坏性 Worktree migration。migration 未应用；membership provisioning、历史 Worktree Project/Owner/WorkItem reconciliation、跨 tenant/project 负向集成、并发重放与真实 RLS 测试尚未完成。旧 Worktree 行的空 project/owner/task link 不做推断回填。

### 4.4 Phase 3C 前端 API 与 Outbox 消费边界

`frontend/src/lib/group-api/canvas.ts` 提供与 §4.3 路由对齐的 typed client：Worktree Canvas 查询/创建、Canvas Document 更新、Canvas Element CRUD 与 EntityRef 管理、Outbox 增量页读取，以及 Canvas→WorkItem 原子创建命令。client 只调用同源 BFF，使用浏览器同源 Cookie，不接收、不缓存、不发送 Bearer token。每个写方法都要求调用方提供 `Idempotency-Key`，版本修改继续使用 `expected_version`。API 错误以 HTTP status + 稳定错误 code 暴露给 UI。

Group API BFF 路由和 typed client 已存在：浏览器请求同源 `/api/group/api/v1/...`，服务端读取认证 cookie 后代理到受信 Group API。UUID Worktree Group 页面调用 GroupContext、WorkItem 与 Canvas element 读取；UUID Project Index 调用 Worktree list 与归档/恢复 plan-confirm。仓库尚无登录流程签发、轮换或撤销 `star_access_token` cookie，因此没有有效会话的部署仍会明确失败，不能代表生产 UI 已启用。正式启用前须由统一身份流程提供 Secure、HttpOnly、SameSite 会话 cookie，并配置 `GROUP_API_BASE_URL`；缺少 cookie、身份过期及 scope 不足必须明确失败，不得降级成匿名/默认 actor。

Phase 3C 增加 additive migration，为 Outbox 加 `stream_offset BIGINT GENERATED ALWAYS AS IDENTITY` 和 `(tenant_id,worktree_id,stream_offset)` 唯一索引。Group API 在同一 tenant + Worktree 维度取得 PostgreSQL transaction advisory lock 后才写 Audit / Outbox；锁持续到提交，offset 在锁内分配，因此 migration 完成后通过该 API 写入的同 Worktree 事件按提交顺序排列且不会因事务提交竞态漏读。此协议会串行化单个 Worktree 的 Canvas Outbox 写入，不影响不同 Worktree 间并行；任意绕过该写路径直接插入 Outbox 的 writer 不享有此保证，所有未来 writer 必须遵守同一锁协议。migration 为已有行回填 offset，但这些历史行的 offset 不代表其真实历史提交顺序；启动消费时应把它们作为 bootstrap invalidation 并重载 canonical 状态，而不要按它们重放领域转移。offset 在 HTTP/TypeScript 合约中用字符串传输；分页上限 100。当前只交付带权限检查的读 API，consumer checkpoint、事件投影、Realtime 推送与断线恢复仍未实现。消费端应在投影成功后持久化 next offset、按 `event_id` 幂等，并在 scope 撤销后关闭订阅及重新鉴权。

Canvas → Task Card 创建已有 `POST .../canvases/{canvas_id}/work-items` 原子命令切片：通过共享 canonical WorkItem 写入函数，在一个 PostgreSQL transaction 中写入 Master、当前 Worktree relation、Multica lifecycle/current + audit、Canvas Element + typed EntityRef、Canvas Audit / Outbox 和幂等响应。写命令需 `canvas:write`、`work-item:write`、`work-item:read` 与 Project writer role；元素引用由服务端生成，浏览器不能提交任意任务 ID。REST SQL coordinator 尚未抽到独立 Domain Command Port / PostgreSQL adapter，因此这是 API 实现切片；`cargo check -p star-api-rest --all-targets -j 4` 已通过，但不能标成 Domain adapter 已完成。Jira Issue relation 仍等待 Jira alias / SoR contract，不能伪装成 WorkItem EntityRef。BFF 已完成但 cookie 签发、migration 与消费端未落地，Phase 3C 仍未完成。

### 4.5 Canvas Document 持久化与并发写

Additive migration `2026-09-29-worktree-group-phase-3c-canvas-document.sql` 建立 `canvas.canvas_document_state` Master/SCD2：同一版本包含 `viewport`、`frames`、`connectors`，已有 current Canvas 通过默认原点 viewport 和空数组建立 v1 初始投影。每个状态更新由 `PUT /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/document` 执行，要求 `canvas:write`、Project writer role、`Idempotency-Key` 与 `expected_version`；服务端在事务内锁当前文档版本、校验引用后关闭旧 SCD2 行、插入新版本并追加 Audit / Outbox。版本不匹配返回 `409 canvas_document_version_conflict`，不得静默覆盖。

服务端限制 viewport/frame/connector 的数量、尺寸、颜色及序列化大小。Frame `element_ids` 与 Connector 两端 ID 必须是同 tenant、Worktree、Canvas 下 current Element；未知或跨画布引用拒绝写入。Group 页面读取返回的 document projection；viewport 的平移/缩放和 fit 操作以短防抖、幂等版本更新保存，空 Canvas 可按授权创建。已有 Canvas 还可通过 `POST .../canvases/{canvas_id}/work-items` 从页面创建 Task Card；WorkItem、Worktree relation、Multica lifecycle、Canvas Element/EntityRef、Audit/Outbox 在一个服务端事务创建，创建后的任务摘要重新从 canonical WorkItem API 读取。Element 内容仍为只读，frame/connector 目前只有读取与 API 持久化，不提供编辑 UI。

Group Canvas Connector 表达画布内 Element 的可视关系；它不自动创建、更新或证明 Multica `dependency` / `blocked` WorkItem relation。真实任务关系须由 Multica/Relation Domain Command 写入并经事件更新 Canvas 投影。Document 表与 API 当前属于未部署实现切片；migration、RLS/ACL 与并发冲突行为尚未在目标 PostgreSQL 运行验收。

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

GroupContext 当前尚未返回 `allowed_actions[]`。Phase 2D 已实现的 plan/confirm API 由服务端重新检查 scope、当前成员角色、Worktree version 和目标 owner membership；UUID Project Index 只为 manager role 显示归档/恢复入口，且必须等到 API 确认后刷新状态。Owner assignment 虽有 API 切片但尚无 UI；Worktree create/import、Git lock 与 Runtime active state 仍未实现。

| 动作 | 预检 | 写入规则 |
|---|---|---|
| 创建 / 导入 | Project 与 Repository 可访问；路径 / 分支不冲突；Runtime 可达 | 当前未实现；待 Worktree lifecycle / Runtime provisioning API |
| 转派 owner | `tenant_admin` / `project_admin`；目标用户仍是该 Project member | 已实现 5 分钟 plan/confirm、expected version、SCD2 owner assignment 与 before/after Audit |
| 归档 / 恢复 | `tenant_admin` / `project_admin`；已有 AgentSession / Runtime 关联时拒绝归档 | 已实现 plan/confirm、expected version、保留截止时间与 Audit；检查当前只按关联 ID 非空保守拒绝，未接 Runtime 活跃状态查询 |
| 清理 | 先跑 plan：Agent / CLI inactive、Runtime detached、Git lock absent、保留策略满足 | 仅使用未过期 plan；二次确认、幂等执行、审计实际 Git 操作 |
| 取消 / 恢复 | operation 与当前 lifecycle 匹配 | 版本条件更新，记录恢复来源和失败原因 |

归档与 Git Worktree 物理清理是不同操作。Phase 2D 只做数据库归档标志，不运行 `git worktree remove`，也不探测 Git lock；在 Agent / CLI / Runtime 状态数据源和 cleanup plan 接通前，清理动作不可用。已绑定 Session / Runtime 的 Worktree 目前一律不允许归档，避免把未知状态误认为空闲。

### 6.2 并行 Agent 可见性

Index 同时呈现一个 Worktree 的 owner、当前 Agent、活跃 Runtime、PR 和风险信号；执行历史由 Session 列表展开。一个 WorkItem 可关联 `0..N` Worktrees；每次 Agent / CLI run 必须选择一个授权 Worktree，并在 Run 上记录 `active_worktree_id`。Agent 不能因 Task Card 关联而获得其它 Worktree 的读取权。

冲突检测分为 Git merge/rebase 冲突、同一 Worktree 的并发写入占用和资源健康错误；三者分别表示，不用单个“locked”状态混写。UI 展示最后一次检测时间与 source version；过期探测显示“未知/待刷新”。

## §7 Task Card、CLI 与 Multica/Jira 联动

1. Phase 2C 新 API 使用一个 canonical `work_item_id`，`task_card_id` 等于该 ID；WorkItem metadata 与 Multica 六态 lifecycle 分表持久化。响应中的 `lifecycle.version` 是 Multica 当前版本。旧三态 `domain-work-item::InMemoryWorkItemService` / legacy REST 不作为生产 Group 数据源。
2. Task Card 打开时展示当前 Group Worktree。若 WorkItem 未关联当前 Worktree，允许查看并提示关联/选择 Worktree；在关联确认之前不允许启动 CLI、Agent 或写入 Worktree 文件。
3. 服务端构造 `TaskExecutionContext { actor, project_id, repository_id, worktree_id, work_item_id, runtime_id, approved_profile_id, policy_version, expiry, nonce }`。路径、命令白名单、环境变量和 Secret 句柄均由服务端 policy 与 Runtime 配置派生，客户端不能提交可覆盖值。
4. `domain-local-runtime::task_execution` 已提供短期上下文模型：私有字段绑定 tenant/actor/project/worktree/work-item/runtime、授权版本、profile 与 expiry；构造时解析可信 checkout，并在启动前再次检查过期时间、Runtime/profile 匹配与 canonical path containment。`spawn_authorized_cli` 只接受服务端 `TaskCliLaunchProfile`，不传入浏览器环境变量，并把 worktree ID 关联到 ProcessHandle。它仍是 domain gate slice：当前未连接 Group API、签名/一次性 nonce grant、PTY interactive session、attach/reconnect、实时撤权或会话审计。
5. 完整 Local Runtime 只接受短期、签名且一次性的 session grant；启动、输入、退出和 session 重连均重新验证范围。审计记录 session ID、命令类别、结果摘要和退出码，不把任意终端内容复制进 WorkItem 描述。
6. 当前实现支持 pending → claimed → in_progress → completed / failed / cancelled 与 failed → pending 的状态转移，`review_state` 独立保存在表中；Review Gate、poison/stale-dispatch、Jira 外部 alias/sync、Outbox 与事件 adapter 尚未落地。不能宣称 Multica/Jira 已接通，只能确认 canonical persistence contract 与 API 第一切片已实现。

## §8 Canvas、Chat、LangGraph 与 Plugin 的交互契约

### 8.1 Canvas

Group Canvas 与 Task Card、Multica、Jira 同级；Canvas Element 仅持有带类型 `EntityRef { ref_type, ref_id, worktree_id }`，不持有 WorkItem 状态副本。Canvas registry 以当前 Worktree 为唯一查询边界；`project` / `free` Canvas 和旧元素中的裸 `work_item_id` 不构成 Worktree 绑定证据。双击任务 Element 导航到保留当前 Worktree 的 `app=task-card&work_item_id={id}`；后端须同时验证 EntityRef 的 `worktree_id`、当前 Worktree 的 canonical WorkItem association 与 Project ACL。“创建任务”“关联任务”“变更状态”“建立关系”须走对应事实 owner 的 Application Command，再由 Outbox 更新 Canvas 与其它投影。Canvas→WorkItem 创建已有同事务 API 命令切片；现有 REST SQL coordinator 与 domain command port 的拆分仍是架构缺口。Project 级 Worktree Overview Graph 与 Worktree Group Infinite Canvas 分路由、分查询范围、分用户目的。

Phase 3B migration 将 `canvas.group_canvas_registry`、`canvas.canvas_elements_backend`、`canvas.canvas_entity_ref` 定义为 Master/SCD2；`canvas.canvas_group_audit` 与 `canvas.canvas_group_outbox` 是 Transaction/append-only。新 Group API 支持 Worktree Canvas 查询/创建、元素查询/创建/版本更新/逻辑移除、EntityRef 设置/替换/解除。读写均先验证 JWT actor、Worktree 所属 Project 与当前 membership，再设置事务级 `app.tenant_id` / `app.worktree_id` 供 RLS 使用；写命令还需 `canvas:write` 和 Project writer role。Phase 3B 当前只接受 canonical `work_item` 与当前 `worktree` 两类 EntityRef；Jira issue 引用要等 Jira alias/SoR contract 落定后扩展。任务 EntityRef 只解析到同 tenant/project、当前 Worktree association 下同时存在 canonical metadata 与 Multica lifecycle 的 WorkItem；写入或读取 WorkItem 引用还需 `work-item:read`，否则元素响应隐藏此类引用。Canvas `content` 不得另存身份 ID；任务卡元素和 Worktree 节点必须分别绑定同类 typed EntityRef。

Canvas Element 与 EntityRef 的 SCD2 变更、append-only Audit、Outbox 和幂等响应在同一 PostgreSQL transaction 提交。事件携带 actor、correlation ID、Worktree / Project scope 与 aggregate version；元素与其 EntityRef 共用单调递增的 Element version，EntityRef 自身版本另放入事件 payload，Canvas registry 事件使用 Canvas version。Canvas Document 另以独立 `document_version` 共同保存 viewport、frames、connectors；expected-version 冲突返回 409，不覆盖较新版本。Canvas API 不更新 Multica lifecycle；任务状态仍走 WorkItem Lifecycle Command。Outbox offset/API、Canvas→WorkItem 原子创建与 Canvas Document 更新已有代码切片，但 consumer/checkpoint/realtime 投影、Canvas/Jira relation command 均未实现。同源 BFF Canvas client 已实现且不持有 Bearer token，UUID Group 页面读取并保存 viewport、读取 frames/connectors；Element/frame/connector 编辑 UI、cookie 签发和数据库部署仍未完成，因此 Phase 3C 仍是未部署实现切片。

Phase 3A 前端切片只加载 `Canvas.ref_kind=worktree && Canvas.ref_id=current_worktree_id`，任务深链要求显式 typed EntityRef 与当前 Worktree task association；旧 Project / Free seed 不自动迁移。UUID Group Canvas 从 BFF 只读加载 Canvas/element 与授权 WorkItem/Worktree 摘要；任务跳转必须在当前授权列表中命中，非 WorkItem legacy EntityRef 不跳全局路由。Canvas Document viewport/frames/connectors 从 `canvas_document_state` 加载；viewport 的平移/缩放使用 expected-version + idempotent command 保存，空 Canvas 可按 capability 创建。Frame/Connector 只能引用当前 Canvas 的 Element，连接线不表示 canonical WorkItem dependency。Canvas Element/frame/connector 编辑、Outbox 消费、实时同步和登录 cookie issuer 未接入。未绑定/越权任务不显示 CLI 入口。

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

当前仓库的 `crates/api/src/chat.rs` 仍是独立 W1/W2 API：`user_id` 来自请求体、session/messages 放在进程内 map，缺少 Worktree/GLOBAL scope 与 Group ACL；`scripts/automation/task_ops/` 提供已有的 asyncio + TypedDict TMO 节点，但不等同于已接入 Group Chat 的 LangGraph StateGraph runtime。两者均不得由 Group Chat 直接复用为生产授权入口。Phase 5 的产品接线需要统一认证、scope/targets DTO、GroupContext re-resolution、事实 owner command bridge、消息与 checkpoint 分离持久化，并在接入时锁定 SDK/checkpointer compatibility。

### 8.3 Plugin 热插拔

原生 App 与 Plugin App 使用同级 registry 节点和统一 shell。Plugin manifest 声明 publisher、版本、host API 兼容区间、capabilities、事件订阅与 UI entry；Registry 的 enablement 不是 capability grant。每次 capability call 由 Gateway 按当前 actor、Project、Worktree 与 plugin grant 授权。

disable/uninstall 后新调用立即拒绝，活动调用按独立 ADR 定义 cancel 或 drain；业务 Transaction、Audit 和 Canvas EntityRef 保留。发布者签名信任根、沙箱承载和 upgrade rollback 是 release blocker，不能由 UI 原型开关代替。

当前 Group Plugins 页面中的 `enabledPlugins` 仅是组件内本地预览状态；仓库尚无 Group App Registry API、manifest trust/signature verifier、Group-scoped grant store、调用 Gateway 或 in-flight run registry。`domain-llm` ProviderRegistry、SubAgent registry 和该预览状态均不是 App Registry。Phase 6 在签名、sandbox 和撤权语义确定前不得启用外部插件执行。

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

Phase 2B-2D 与 Phase 3B migrations 按事实表分类；旧 `worktree_canvas_worktree` 仍是 Work 当前投影，新增的 project/owner 字段仅作 denormalized read projection，权威 Master 事实分别存储在 `worktree_project_binding` 与 `worktree_owner_assignment`。空投影仍需显式 reconciliation：

| 数据事实 | 主分类 | 约束 |
|---|---|---|
| `permission.project_role_binding`、`multica.worktree_project_binding`、`multica.worktree_owner_assignment`、`multica.task_metadata`、`multica.work_item_worktree` | Master | SCD2 `valid_from/valid_to/version`；禁止物理删除；RLS。当前只有 Project/owner 管理写 API，Task metadata SCD 更新待 Domain adapter |
| `worktree_canvas_worktree` 与 `multica.task_lifecycle_current` | Work | 当前运行 / lifecycle projection；`retention_period` + nullable `expires_at`；终态 completed/failed/cancelled 设期限，重试回 pending 清除期限；不承载 owner 历史或唯一审计事实 |
| `multica.task_command_idempotency`、`multica.worktree_management_plan` | Work | 30 天 retention；管理 plan 确认窗 5 分钟；允许按 TTL 清理 |
| `multica.task_lifecycle_audit`、`multica.worktree_management_audit` | Transaction | Append-only；RLS；拒绝 UPDATE/DELETE；生命周期审计同事务写入 |
| `canvas.group_canvas_registry`、`canvas.canvas_elements_backend`、`canvas.canvas_entity_ref` | Master | Worktree scoped SCD2；RLS 同时限制 tenant/worktree；每个 Canvas element 至多一个 current EntityRef；物理删除禁止 |
| `canvas.canvas_group_audit`、`canvas.canvas_group_outbox` | Transaction | 同一业务事务内 append-only；RLS 同时限制 tenant/worktree；migration 后由 API 写入的新 Outbox 事件以 identity `stream_offset` 提供提交有序读取游标，历史 offset 只用于 bootstrap；consumer checkpoint / realtime 投影另行实现 |
| `worktree_canvas_worktree.project_id / owner_user_id / work_item_id` | Work 投影（混合字段已知缺口） | Project / owner 的事实分别在独立 Master 表；这些 nullable 列仅作旧 Index 查询投影，不可推断回填；须由受控 reconciliation 维持一致 |
| GroupContext | Derived projection（不持久化授权快照） | 需要缓存时为 Work/短 TTL；撤权后失效，不可恢复授权 |
| Plugin manifest / Project enablement / capability grant | Master | 版本化、审计变更；撤权立即作用于新调用 |
| Chat message / LangGraph run checkpoint | Transaction + Work 分表设计 | 用户可见消息保留策略独立于可过期 checkpoint；两者不可混成一个无期限 blob |

Phase 2 migrations 已逐表加入 tenant RLS；Master 表无 DELETE policy，Transaction 表仅有 SELECT/INSERT policy 且拒绝 UPDATE/DELETE，Work 表记录 retention。仍未覆盖仓库守门要求的完整 RLS policy 分类与 PostgreSQL 真实实例验证；不得据此宣称 W/T/M / Security release gate 已全部通过。

## §11 关键验收映射

| Requirement / AC | 详细设计落点 | 服务端验收重点 |
|---|---|---|
| WTG-001 / AC-WTG-001 | §1-§5 | Project membership 限定 Index；同级 App 树与唯一 GroupContext |
| WTG-004 / AC-WTG-003 | §3、§6 | owner、Agent、Runtime、PR、冲突/锁和最近活动来自有版本的服务端投影；操作有授权与 Audit |
| WTG-005 / AC-WTG-002 | §1-§5、§9 | Project/Worktree 切换清空旧订阅；跨项目实体引用拒绝 |
| WTG-006 / AC-WTG-004 | §1.2 | Index 与 Group 深链独立、刷新稳定，不落入通用任务列表 |
| TCI-001 / AC-TCI-001 | §7 | Multica/Jira/Task Card 同一 WorkItem ID 和版本 |
| TCI-002 / AC-TCI-001 | §7 | CLI 只在正确 Worktree、授权 profile 与 Runtime 运行 |
| CAN-001 / AC-CAN-001; CAN-006/007 / AC-CAN-002 | §4.4、§8.1、§9 | Canvas 双向导航只跟随当前授权 canonical WorkItem；布局/connector projection 持久化；Group 引用不得回退旧全局路由；Outbox 使用 Worktree commit-order offset 续读和去重 |
| CHAT-001 / AC-CHAT-001 | §8.2 | WORKTREE 与 GLOBAL scope、目标 ACL、部分失败逐目标可见 |
| CHAT-003 / AC-CHAT-003 | §8.2 | 每次 start/resume/approval/tool call 重新解析 GroupContext；legacy Chat/TMO 不可绕过授权 |
| PLG-001 / AC-PLG-001 | §8.3 | 热撤权对新调用即时生效，活动调用有确定终止语义 |
| PLG-004 / AC-PLG-002 | §8.3 | App、SubAgent、Provider registry 职责隔离；preview 开关不产生生产授权 |
| ARCH-OBL-GRP-001 / AC-TRACE-001 | §4-§10 | 一个 correlation ID 串起授权、命令、Outbox、投影与 Audit |

## §12 实施顺序与阻断门

1. **2B 代码切片完成**：RS256 Actor、scope、Project membership 查询、GroupContext、Project/owner Master binding DDL。部署、membership provisioning、历史数据 reconciliation 和跨 Tenant/Project 负向集成仍是启用门。
2. **2C 代码切片完成**：canonical task metadata、Worktree relation、Multica 六态 current projection、幂等 command 和 append-only audit；REST SQL coordinator 尚未抽到 Domain/PostgreSQL adapter，Review Gate、Jira alias/sync 与 UI 写入仍未启用。
3. **2D 代码切片完成**：cursor Index、owner/archived plan-confirm、owner SCD2 和 management audit。创建/导入、Git lock/Runtime 活跃探测、Session drain、物理清理与恢复仍未实现。
4. 完成 migration apply + 项目成员 provisioning 后，运行跨 Tenant/Project ACL 负向、数据库 RLS、并发 version conflict、同键幂等重放、审计不可变性和 API 集成验收，才能关闭生产写能力 gate。
5. UUID Project 的 Group UI 已读取 GroupContext、WorkItem 和 Canvas element 的服务端 projection；未认证时必须显式失败，seed 仅保留给非 UUID preview。生产启用仍需统一登录 session、数据库部署、完整 Canvas projection 与跨 App 验收。

## §13 已知缺口与实现前置

| # | 缺口 | 影响 | 关闭条件 |
|---|---|---|---|
| 1 | 新 Group WorkItem coordinator 仍在 REST crate 直接 orchestrate SQL；旧 domain-work-item 是三态内存 service | Domain Command Port / PG adapter 的边界尚未完成；旧 `/work-items` 不可用作生产 | 抽出同事务 WorkItem Command / Multica Lifecycle application port，并用 PG repository 实作；完成 API 兼容迁移 |
| 2 | 当前六态 API 只有状态转移；Review Gate、review command、Task metadata 更新和 Jira external alias/sync 未实现；Group 页面仅投影 WorkItem，Multica 编排和 Jira Board adapter 尚未接入 | 任务流程与 Jira adapter 尚不完整 | 接入 DD-MULTICA §14 review/metadata port、alias registry + sync/outbox，跨 App 实测同一 ID/version |
| 3 | 旧 Worktree projection 保留 `project_id` / `owner_user_id` denormalized columns；权威 Project / Owner SCD2 表已创建 | 历史投影与 Master binding 需保持一致；旧 owner/Project 行未回填 | provisioning/reconciliation 同事务维护 Master 与投影；历史映射有审计证据 |
| 4 | GroupContext 只返回 context 与 Worktree，没有 App Registry / `allowed_actions[]`；Canvas 当前只有 WorkItem/Canvas element 只读投影，Chat、LangGraph、Plugin API 仍未落地 | Worktree Group 仍不能完成跨应用写入、编排与插件加载 | 接入 Registry / EntityRef resolver 与各 App API，权限测试覆盖每个入口 |
| 5 | 当前 archive guard 仅根据 AgentSession/Runtime reference 是否为空；没有 Git lock、活跃会话、Runtime drain 状态源 | 对有过绑定但已结束的 Runtime 也会保守拒绝；不能安全执行物理清理 | 接 Agent/Runtime/Git 权威状态、cleanup plan/confirm、恢复策略与故障演练 |
| 6 | Phase 3B/3C 后端、typed client 和同源 BFF 已有代码切片；UUID Group 页面已通过 BFF 读取 GroupContext、WorkItem 与 Canvas Document/element；登录流程尚未签发/轮换 `star_access_token` cookie；migration 未部署；Outbox consumer/checkpoint/realtime projection、Element/frame/connector 编辑、Canvas→Jira command 未实现 | 只有完成登录的部署环境才能显示授权 read projection；Canvas 与任务/Jira 仍不能端到端协作 | 接入统一 per-user login/session 并以 Secure + HttpOnly + SameSite cookie 提供 JWT；评审并应用 migrations；实现可靠 Outbox offset/重连；Canvas Element/Frame/Connector 写入只允许当前 Worktree 中的 typed refs；Canvas 创建/关联任务只走事实 owner command；确定 Jira alias/SoR；审计分类旧 Project Canvas，歧义项隔离；完成跨 Worktree/Project ACL 与 RLS 负向验收 |
| 7 | Group Chat 仍与 legacy Chat/TMO 分离；TMO 虽有 asyncio + TypedDict 实现，但没有 GroupContext adapter 或已锁定的 LangGraph runtime/checkpointer dependency | 底栏无法安全创建/恢复 scope-aware L0 run；旧 session/user_id/checkpoint 不能作授权依据 | 引入认证 Group Chat API；WORKTREE/GLOBAL target preflight；每次 resume/interrupt/tool-call 重验；独立持久化消息与 checkpoint；完成锁定 SDK API review 和最小运行演练 |
| 8 | 2B/2C/2D migrations 未应用；Project SoR / Role Binding provisioning API 尚不存在，Worktree 历史 scope/owner/work-item association 未回填 | 代码可编译，但无可用 membership 和可信历史映射；数据库 RLS / FK 未真实验证 | 在目标数据库评审并应用 migrations；通过受控 provisioning 建立 M bindings，审计 reconciliation，并完成跨 tenant/project/RLS 负向集成 |
| 9 | Phase 4 只实现 Local Runtime 的短期 TaskExecutionContext、profile/runtime 绑定与 checkout containment gate；还没有 Group API start/cancel/status 路由、一次性签名 session grant、PTY interactive transport、attach/reconnect、session persistence/audit 或 Task Card UI 接线 | Task Card 仍无法启动、查看、取消或恢复真实 CLI 会话；当前 preview 不能转为执行能力 | 建立服务端授权与短期签名 grant；把 Task Card command 接到受授权 Local Runtime；复用 terminal-stack 之前先补 PTY sink 和 session ownership；落 session/audit/outbox；完成越权路径、过期 token、撤权、断线重连与取消验收 |
| 10 | Group Plugins 页面只有本地 preview 开关；没有 Group App Registry / manifest signature trust / capability Gateway / active invocation cancel-or-drain 实现 | 热插拔和即时撤权尚未真实发生；不能安全加载第三方代码 | 完成插件信任/签名、隔离 sandbox、版本/权限兼容、grant/audit schema 与在途调用语义；实现每次调用强制授权和 disable 后拒绝新调用；执行撤权与保留引用验收 |

## §14 审阅栏与修订履历

| 角色 | 状态 |
|---|---|
| 架构 | Draft；Mavis 接手审核，Phase 2B-2D、Phase 3B/3C API/client/BFF 和 Phase 4 Local Runtime path gate 代码切片完成；待 DB / ACL / Domain Port / cookie 签发 / Outbox consumer / CLI session transport 验收 |
| SRE Lead | 待详细验证：故障切换、事件积压与 Runtime 清理恢复 |
| 平台 | JWT / scope / ACL、Task lifecycle 与 Index 管理 API 可编译；需验证密钥部署、迁移、RLS 与数据回填 |
| 评审主持 | Draft；检查事务边界、幂等重放、跨 App 事件和错误映射 |
| PM | Draft；确认 production API 接入、Jira sync 与清理策略的发布范围 |

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Project → Worktree → Group Apps 详细交互、安全上下文、管理操作、CLI、Canvas、Chat/LangGraph、Plugin、事件与 W/T/M 实施门；记录 REST no-op auth 与 in-memory blocker | 推进 Worktree Group Phase 2，并依据 Worktree Index 当前实现核对 API 安全边界 |
| v0.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 落地 RS256 actor 校验、Project scope + active membership、租户 RLS GroupContext resolver 与只读 Index/Group 路由；定义 2B migration 和未回填数据边界 | 推进 Worktree Group Phase 2B；落实 Project → Worktree Index → Worktree 同级应用架构 |
| v0.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补齐 canonical WorkItem / Multica 六态持久化、幂等与追加审计，稳定 cursor Index，Worktree owner SCD2 与 archive plan/confirm；逐项标出未部署数据库、旧投影 reconciliation、Jira/Review/Runtime/Canvas/Chat/Plugin 等生产缺口 | 用户要求继续完成 Phase 2B/2C/2D，并重申 Project Worktree Index 是管理多 Agent Worktree 的主入口 |
| v0.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 收紧 Task 关系、metadata 与授权 Project 一致性查询；GroupContext/WorkItem 请求锁定 Worktree Project 绑定；claim lease 与管理确认期限改由 DB 时钟判断；为 failed 终态补齐 Work 保留期限与重试清除语义 | 自审发现 tenant 内异常 WorkItem-Project 关系可能越过当前 Project 查询边界，并统一短期 lease / confirm deadline 时钟来源 |
| v0.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义带 `worktree_id` 的 Canvas EntityRef 以及旧 Project/Free Canvas 隔离规则；前端 Group Canvas 仅接受显式 Worktree 绑定，并阻止未绑定任务的 Canvas 深链和 CLI 入口；记录 Canvas persistence/API/Outbox 尚未实现 | Phase 3 源码核对发现 Group 页面复用全局 Project Canvas seed，可能把跨 Worktree 任务当成当前上下文 |
| v0.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 落地 Canvas Worktree scope migration/API 代码契约：Canvas/Element/EntityRef SCD2、Audit/Outbox append-only、幂等与版本更新、当前 WorkItem 关联解析；明确未部署数据库、未接前端 API / Outbox consumer / Canvas WorkItem Command | 用户要求继续推进 Phase 3，补齐 Canvas 与 Worktree 内 Task Card 的服务端持久化边界 |
| v0.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Phase 3C typed Canvas API client 的 Bearer / 幂等调用契约；明确当前 UI 无 token provider，Outbox 缺少可靠提交顺序消费位置，Canvas→WorkItem 需要事实 owner 原子命令，阶段仍未关闭 | 推进 Canvas API 接线并复核前端认证态和 Outbox schema 后确认端到端前置条件缺失 |
| v0.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Outbox stream_offset 与读取页 API、按 tenant+Worktree 事务锁的写序协议及 Canvas→WorkItem 单事务命令；同步 typed client 和未部署/未接入/未消费边界 | 推进 Phase 3C 后端跨 App 闭环，落实可续读游标和任务创建原子性 |
| v0.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对齐 requirements v2.5 / basic design v0.6；定义同源 Group BFF 的 cookie、Origin、path/method allowlist 与上游边界；把缺口更新为 cookie 签发/刷新流程和页面 client 接线 | 为 Canvas 与后续同级 App 建立浏览器不持有 Bearer token 的服务端接入层 |
| v0.10 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补入 Local Runtime TaskExecutionContext、profile/runtime 绑定与 canonical checkout path gate；明确 Group API、签名一次性 grant、PTY attach/reconnect、session audit 与 UI 仍未实现 | Phase 4 Task Card CLI 首个授权执行边界落地并通过 domain-local-runtime 编译检查 |
| v0.11 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 legacy Chat/TMO 与 GroupContext/LangGraph runtime 的边界；明确当前 Plugin 开关仅为 preview，并将 Group Chat 授权接线与 Group App Registry/sandbox/revocation 列为独立未完成门 | Phase 5/6 实际代码盘点发现相邻 legacy service 与 UI preview 尚未集成到 Worktree Group |
| v0.12 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对齐 requirements v2.7 / basic design v0.9；记录 UUID Project Index 与 Group 页面通过 BFF 读取授权 Worktree projection，Index manager 接入归档/恢复 plan-confirm；规定 Canvas typed reference 不得回退到全局路由，并列明 viewport、frames、connectors 和统一登录仍未接入 | Phase 2D/3C UI 对接后，需记录已接通的只读/管理切片与仍未完成的认证和 Canvas 投影 |
| v0.13 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对齐 requirements v2.8 / basic design v1.0；定义 Canvas Document SCD2 migration、expected-version/幂等更新、同 Canvas Element 引用校验、授权创建和 viewport 页面保存；明确 connectors 不是 WorkItem dependency SoR 与后端仍未部署 | Phase 3C 实装 viewport/frames/connectors 持久化读写和 UI 接线后同步详细设计及剩余验收门 |
