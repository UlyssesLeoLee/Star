# DD-WORKTREE-GROUP-001

> **渡口 Project / Branch / Engineering Run / Worktree 与 Run Apps 详细设计 v4.25**
>
> - 状态：🟡 Draft（Phase 2B/2C/2D 与 Phase 3B-3F 已有多项条件式 API/UI 切片；Phase 4A signed grant helper、4B1 Session start seam、4B2 PTY adapter、4B3 卡内 xterm ticket-first UI、status/cancel/reattach、bounded Session listing/recovery API seam 与手动 UI 已实现；Phase 5 有 scope-aware Chat 授权提交、GLOBAL 目标目录、多选 UI、加密 Transcript/Run/outbox persistence adapter，但 production main 未装 protector/L0；Phase 6 有五表 Registry migration、生产 main 装配的 PostgreSQL 只读 projection provider/API 与 Group UI live consumer；Phase 5/6 migrations 已在隔离库重复执行并验证 12 张 FORCE RLS、策略及 trigger（事务内临时授权已回滚），目标 DB/runtime role grants 未配置；manifest trust root/ingest、lifecycle writer、capability runtime/revocation、真实 PostgreSQL RLS 验收未完成。仍缺宿主认证 provider、目标 DB migration 部署与 ACL/RLS 运行验收、真实 CLI provisioner/OS sandbox/terminal sink/audit、LangGraph 部署版本/服务身份/权限 broker；Canvas 仍缺服务端 durable event offset/realtime；历史归属 reconciliation 与跨 App 生产验收未完成）
> - Phase 8A/8B 条件式实现：Run migration 在隔离 PostgreSQL 临时集群重复执行，6 张 Run 表均验证 `FORCE ROW LEVEL SECURITY`；目标数据库/runtime grants 未部署。CLI start writer 与 Worktree/Task-scoped Run list/detail API、Task Card Run History 面板已有代码切片；其余 Event/Evidence producer、Task Contract 写 API 和真实 Runtime provider 未实现。
> - 日期：2026-10-01
> - Phase 2D 状态：Git Worktree retention-lock observer contract 与 Index UI 已有条件式切片；生产 main 未配置 Host Runtime observer，因此运行态仍显示 unknown。
> - Phase 9B2C 状态：REST archive-confirm gate 已接入已验证的 Project/Worktree Hook policy 与 Rust evaluator；数据库事务锁外先观测 Git lock，仅新鲜 Unlocked 时才请求 Host Runtime drain/readiness，并取得 operation-scoped admission fence expiry；最终 archive mutation 前要求至少 5 秒余量并复核。provider 需保证 fence 覆盖命令完成窗口，目标 DB 事务时限仍需定义和验收。production main 未安装 readiness provider，缺失时 fail-closed 返回 503；目标 DB/RLS、RunEvent/outbox 与物理 checkout cleanup 未验收。Hooks 导航仍是 Advanced Settings 内与 Skills/MCP/Plugins 并列标签，不属于 Worktree 树。
> - Phase 9D 状态：Project-scoped hook-events/summary API 已提供 1–90 天 source-only metric v1，按 phase/decision 汇总当前 ledger 并保留 partial/null coverage；尚未 join RunEvent/outcomes、实现完整 BI read model 或接入 Quality & Improvement。目标 DB/RLS/grants 与 app auth Provider 未验收。Hooks 导航仍是 Advanced Settings 内与 Skills/MCP/Plugins 并列标签，不属于 Worktree 树。
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 上位需求：[`docs/requirements.md`](../requirements.md) v5.42 §50
> - 上位基本设计：[`docs/basic-design.md`](../basic-design.md) v5.39 §16
> - 配套详细设计：[`DD-MULTICA-TASK-001.md`](DD-MULTICA-TASK-001.md) v1.16、[`DD-MULTICA-HOOK-001.md`](../detailed-design/DD-MULTICA-HOOK-001.md) v0.5.14、[`DD-WORKTREE-CANVAS-001.md`](DD-WORKTREE-CANVAS-001.md) v1.4、[`DD-SHARED-TASK-001.md`](DD-SHARED-TASK-001.md) §11
> - 文档边界：本 DD 定义 Project → Cloud Branch → Engineering Run → Run Worktree 的导航与应用契约；Worktree Index 仅为 Project aggregate 管理视图，Worktree 不拥有 Run Apps；不新增 WorktreeGroup / ProjectGroup 业务聚合，不宣称原型已具备生产授权、持久化或多 Agent 调度能力。

---

## §0 目的与完成判定

本设计把工程协作导航定义为 Project → Cloud Branch → Engineering Run → Run Worktree。Project 是左侧第一级；Branch 是远端合并目标；Engineering Run 是跨 App 的 canonical 协作工作区；Worktree 是 Run/Branch 绑定的本地 checkout。点击某个 Run Worktree 后才打开其 Run tabs，并设为 focus/CLI/Git target。多 Agent Worktree 的归属、Branch/checkout 绑定、运行状态、冲突、锁、PR 和最近活动仍须在 Project Worktree Index 聚合可见；该 Index 不取代 Branch/Run 主导航。所有动作须有明确授权、状态守卫、审计和幂等键。

生产完成须同时满足：

1. Index 的 Project 范围来自服务端授权，不接受客户端声明的 `tenant_id`、actor 或权限。
2. 所有 Run App API 先解析同一服务端 `RunContext`；Workbench/CLI/Git 请求再校验独立 `focus_worktree_id` binding。
3. WorkItem、Canvas、Agent Session、CLI、Chat、LangGraph 与 Plugin 通过 canonical ID、命令和事件互操作，不维护互相矛盾的事实副本。
4. 创建、归属调整、归档和清理 Worktree 可审计、并发安全，并能处理运行中 Agent、CLI 与 Git 锁。
5. 需求 §50 的 AC-ERUN、AC-EVENT、AC-WTG、AC-WTI、AC-TCI、AC-CAN、AC-CHAT、AC-PLG 与 AC-TRACE 均通过服务端集成验收。

Group Shell 当前仍是浏览器 seed 预览。Project Worktree Index 已有条件式授权 API 投影与 archive/restore plan-confirm UI 接线，但应用路由树尚未安装宿主 provider，因此运行时仍是 preview。生产 Group API 已有认证、Project ACL、WorkItem 持久化/生命周期和 Worktree Index/管理命令代码；target migration、membership provisioning/reconciliation 与 ACL/RLS 负向集成验收仍未完成，因此这些接口尚不可作为已启用生产能力。

## §1 范围与导航契约

### 1.1 产品树

```text
Project Selector（左侧第一级）
└─ Project
   ├─ Cloud Branch（远端合并目标）
   │  └─ Engineering Run（跨 App 协作工作区）
   │     ├─ Run Worktree Set（绑定 Branch 与 Run 的本地 checkout）
   │     │  └─ Worktree focus（点击后设置当前 focus）
   │     └─ Run App tabs（Engineering Run owner；focus 后展示）
   │        ├─ Inbox / Work Items
   │        ├─ Multica Lifecycle 与 Jira-class Board / Backlog / Sprint / Relation
   │        ├─ Task Card Index（与 Canvas 同级）
   │        ├─ Infinite Canvas（可关联 Run 中多个 Worktree）
   │        ├─ Workflow / LangGraph
   │        ├─ Run BI / Benchmark
   │        └─ Plugin Apps
   └─ Project Worktree Index（跨 Branch/Run 的 aggregate 管理视图）

Engineering Run Shell 固定底栏：Chat(scope = WORKTREE | GLOBAL)
Task Card 内：Agent Session / 受控 CLI
```

Task Card 与 Canvas 是 Engineering Run tabs 的直接同级入口。CLI 是 Task Card 内的执行面板；`TaskExecutionRun` 是单次执行尝试，绝不能与 `EngineeringRun` 混用。右侧 tabs 在选定 Run 内 Worktree 后才显示，但 Worktree 只作 focus/checkout target。底栏 Chat 显示在 Run Shell；Project Worktree Index 是独立 aggregate 管理入口。Run BI/Benchmark 显示当前 Run；Project Quality & Improvement 汇总跨 Run 指标并保留 Task → TaskExecutionRun → Evidence 下钻。

### 1.2 路由

| 路由 | 用途 | 解析规则 |
|---|---|---|
| `/projects/{project_id}/branches/{branch_id}/runs/{engineering_run_id}/worktrees/{worktree_id}?tab={app_id}` | 主导航 canonical Run shell deep link | 每次解析并授权 Project/Branch/Run，确认 Worktree 当前 binding 后设置 focus；`tab` 仅选择同级 Run App，不授予 capability |
| `/worktree?project_id={project_id}` | 兼容 Project Worktree Index 路由 | `project_id` 只作为目标引用；服务端校验 Project membership；迁移期映射到 Project Index，不替代 Project→Branch→Run 主导航 |
| `/worktree/{worktree_id}/group?app={app_id}` | 兼容 Worktree Group 路由 | 从 Worktree 服务端解析 Branch/Run 并转入 Run Shell；URL 中的 `app_id` 不授予能力，未知或失效 binding fail closed |
| `/worktree/{worktree_id}/group?app=task-card&work_item_id={id}&cli=1` | 打开 Task Card，可选打开 CLI 面板 | 任务与 Worktree 关联、权限和 CLI policy 必须由服务端检查；没有关联时只允许查看或发起受权关联流程，不启动 CLI |

当前 `WorktreeGroupApiClient`、GroupContext、Worktree-scoped WorkItem/Canvas/Plugin routes 和数据库 migration 均为迁移兼容实现。目标 API 以 Engineering Run 为 App resource owner；每个旧 Worktree API 须按有效 Branch/Run binding 解析 Run、检查 actor 对 Run App 的授权并返回可校验 `engineering_run_id`，再调用事实 owner。Worktree 只保留 lifecycle、Repository/checkout、Git/CLI/文件操作目标身份。兼容路径不得创建 Worktree-owned Task/Canvas/Chat/Plugin/BI 事实，也不得因 route 带有 `worktree_id` 而扩大授权。

生产 `ProjectSelector` 通过 `GET /api/v1/projects` 读取当前 actor 的有效 Project membership；目录只返回 Project UUID/role，以 UUID keyset cursor 分页并设 no-store。当前仓库未找到持久 Project 名称 SoR，因此生产标签用 `Project {UUID}`。宿主 API 错误时不得回退到本地 seed；仅在没有 provider 且显式标记为本地原型时可以展示 seed。Project 页的 Worktrees 入口必须带上 `project_id`。Index 和 Group 的深链刷新后仍须恢复同一范围；`/worktree` 不得重定向至 Sprint 或通用任务树。前端导航状态可缓存，但不作为授权依据。

### 1.4 Group UI JWT Provider 与 API Adapter

`WorktreeGroupApiClient` (`frontend/src/lib/group/worktreeGroupApi.ts`) 只接收宿主提供的 `GroupAccessTokenProvider`，每个 HTTP 请求都重新调用 provider 取得当前用户 JWT。登录、token 刷新、登出与 token 生命周期由宿主会话层负责；adapter 不访问 localStorage/cookie/token endpoint，不持有 refresh token，也不缓存 access token。客户端封装 Project Worktree Index/filter、Project member directory、owner/archive plan-confirm、GroupContext、WorkItem、Canvas/Element/Outbox events 和 Canvas→WorkItem 命令，路由中的 Project / Worktree ID 保持显式。

请求必须把当前路由中的 `worktree_id` 纳入 Group API 路径，并通过 `Authorization: Bearer <access-token>` 发送；禁止把 token 放入 query、SSE/terminal WebSocket URL 或协议帧。请求使用 `credentials: omit`、`cache: no-store`；远程 API origin 必须为 HTTPS，仅 loopback 本地开发允许 HTTP，并拒绝包含凭据、query、fragment 或 scheme-relative host 的 base URL。provider 返回空值时抛出 `session_required` 且不调用 fetch；HTTP 401/403 保留状态码和服务端错误码并传回宿主，不得回退 seed。provider 还必须传入非敏感 `sessionKey` generation；用户 principal、会话登录或登出切换时该 key 必须变化，使 UI 在切换后的首次 render 立即丢弃旧 Group projection，并使用新 JWT 重新加载。`sessionKey` 不作为授权依据、不发送给服务端、不持久化。服务端仍从 JWT 构造 Actor，并对每个 endpoint 复验 scope、Project membership、当前 Worktree 和实体关联；客户端收到 token 不代表获得访问权。

Group route 已提供 `WorktreeGroupApiProvider` 注入点和条件式 projection：宿主提供 provider 时读取当前 Worktree 的 GroupContext / WorkItems，进入 Canvas App 后才请求该 Worktree 的 Canvas 列表与所选 Canvas Elements，并启动绑定所选 Canvas 的本地持久游标 poller；API 错误不回退 seed。live Canvas 提供认证态 Canvas→Task Card 创建入口、已有 Task Card 关联入口和 Canvas 新建入口；既有任务关联使用单个认证幂等 Element 命令原子写入布局与 EntityRef，成功后刷新投影并提供 Task Card 链接。新建响应中的 `canvas.canvas_id` 用于更新路由并切换画布，随后强制刷新授权投影；Document viewport 可经 CAS 保存，Element 位置可经独立 versioned update CAS 保存，viewport/Frame/connector Document CAS 与 Element position/geometry/content/delete CAS 已有条件式接线；其它 Element 内容仍未接入。当前宿主尚未装配 provider，因此默认仍显示有标记的 mock/preview。余下 blocker 包括 Canvas 其它编辑/删除、服务端 durable offset/realtime、目标库部署及跨 App 浏览器验收。

### 7.1 Task Contract 与 TaskExecutionRun

Task Card 持有 Task 的 goal/scope/dependencies/acceptance criteria 合同版本；每次 CLI/Agent/LangGraph 等真实尝试产生独立 `TaskExecutionRun`，Run 引用合同版本并固定 input/acceptance snapshot 和可空 resource budget snapshot。Worktree 是可选执行上下文，Run 保存 Worktree/repository/ref/commit 快照但不依赖其生命周期；不同尝试生成不同 Run，不能复用 `group_chat_run`。CLI start API 仅为已通过授权与状态检查的启动建立 Run，绑定 idempotency replay 到同一 Run，并把 `task_run_id` 传递给 Runtime provisioner。Run resource summary 只存一次 high-water 事件，unknown 保持 NULL；实时资源采样只用有界 TTL telemetry。

执行器状态、Agent 声明、自动验证、人工接受/返工、集成与成本分别作为追加事件写入。Task Card Detail 提供 Goal / Execution / Evidence / Feedback / Compare 页签；默认摘要包含最近 Run、验收进度、人工介入和阻塞，原始证据按需获取。Evidence 保存脱敏 metadata/digest/受控 locator，不内嵌大日志、Secret、完整 transcript 或模型推理。现有 `2026-09-30-worktree-task-execution-run.sql` 与 REST CLI start handler 已形成 Phase 8A 条件式 schema/writer 切片；migration 在隔离临时 PostgreSQL 集群重复应用并验证 6 张表 FORCE RLS，但目标数据库/runtime grants 未部署。Phase 8B 的 Worktree/Task-scoped list/detail API 与 Task Card Run History 面板已形成条件式切片；Task Contract 写命令、CLI exit/status、Validation/Review/Integration/Cost 与其余 Evidence/resource-summary producer、真实 Runtime provisioner 尚未接入。

Run 查询路由固定为 `GET /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/runs` 与 `.../{run_id}`。服务端每次重新验证 Bearer actor、`work-item:read`、tenant、当前 Project membership、Worktree binding、WorkItem canonical 关联及路径 Run 所属关系。列表默认 20/最多 50，以 `(started_at, run_id)` 倒序复合游标翻页；详情最多投影 100 个 Event 与 100 个 Evidence。响应 `no-store`，不返回任意 Event details 或 artifact locator/raw output；execution、verification、human acceptance 分别读取自己的最新非空值。UI 仅在当前 Task Card 展示，认证 provider 缺失/请求失败/身份切换时不以 seed 补数据。

### 1.3 当前 UI 与目标状态

| 区域 | 预览实现 | 生产目标 |
|---|---|---|
| Project Index | 显式标记的本地原型预览 | `GET /api/v1/projects` membership 目录 + 授权 Worktree 投影；API session generation 更换时清除旧目录、Index 与成员角色 |
| Worktree 归属 | 当前 AgentSession / Runtime 引用 | 独立 `owner_user_id`、AgentSession 当前绑定及历史会话 |
| Worktree 操作 | 不提供创建或清理按钮 | 按 §6 的授权动作、状态守卫和 Audit |
| Group Apps | 原生 App 本地路由；有 API provider 时读取服务端授权 projection 并清空 stale/error 导航；无 provider 时仅显示标记的本地预览 | 服务端 App Registry + 当前 GroupContext + 共享实体引用；Plugin Gateway/runtime 与热撤权仍需部署 |
| Chat / CLI / Plugin | Chat 发送禁用；CLI 等待 Session API；Plugin 入口/能力只做本地预览 | 授权 API、真实 PTY Runtime、Plugin Registry/Gateway 与即时撤权 |

## §2 模块与责任边界

| 模块 | 输入 | 责任 | 不负责 |
|---|---|---|---|
| `ProjectSelector` | `GET /api/v1/projects` 返回的 actor membership directory | 仅呈现服务端当前 tenant/user 授权的 Project ID/role；选择、清除旧 Worktree selection、写入可分享路由 | 自行判断 membership；用本地 seed 补生产名称 |
| `ProjectWorktreeIndex` | `project_id` + 服务端投影 | 比较 Worktree 运行信号、显示风险并提供允许的管理动作 | 创建第二份 Worktree 状态事实 |
| `ProjectBranchNavigator` | actor 的 Project/Branch/Run membership projection | 渲染 Project → Cloud Branch → Engineering Run → Worktree tree | 把 Project Worktree Index 当成主树父级 |
| `ProjectWorktreeIndex` | `project_id` + 跨 Branch/Run Worktree projection | 对多 Agent Worktree 做聚合比较和受权管理 | 拥有 Run Apps 或取代 Branch/Run 导航 |
| `EngineeringRunShell` | `engineering_run_id` + `RunContext` + optional focus Worktree | 解析 Run 同级 App、header、Chat 与 tabs | 让 focus Worktree 变成 Run App 的数据 owner |
| `RunAppRegistry` | 原生 App registry、Plugin manifest、actor grants | 生成 Run tabs 及 capability projection | 以 UI 隐藏代替服务端授权 |
| `RunContextResolver` | Authenticated Actor、Project/Branch/Run membership/ACL | 从持久事实解析 Run owner context，并独立验证 optional Worktree binding | 接受客户端上传的 actor、tenant 或权限 |
| `EntityRefResolver` | typed EntityRef + RunContext | 解析实体并校验真实 owner、Run、版本和权限 | 仅凭 entity ID 或当前 focus 授权 |
| `DomainOwnerCommandGateway` | typed command、owner API、幂等键 | 调用事实所有者 API；owner txn 写 Audit/Outbox | 跨 schema 直接写表或在前端更新另一个 App store |
| `RunProjection` | Outbox / Inbox-consumed domain events | 更新 Run tabs、Project aggregate 和订阅投影 | 覆写事实所有者状态 |

Engineering Run Domain 持有 Run membership 与 App context；Worktree Domain 持有 checkout lifecycle/binding 与 Git 操作；Work Item Domain 持有 Run-scoped task/plan/relation facts；Canvas Domain 持有 Run-scoped Canvas；Agent / Runtime Domain 持有 `TaskExecutionRun`、执行和终端会话；Workflow Domain 持有流程状态；Plugin Registry 持有 Run App manifest/binding；BI/Benchmark 只拥有自身指标定义、cohort 与 replay receipts；Audit 持有不可变审计事实。UI tab 可以独立发布/挂载，后端事实仍按 domain owner 管理。

## §3 核心 DTO 与不变量

### 3.1 Project Worktree Index item

```text
WorktreeIndexItem {
  worktree_id, project_id, workspace_id, repository_id,
  name, path, parent_id?, branch, human_state, machine_state,
  owner_user_id?, work_item_id?, agent_id?, agent_session_id?, runtime_id?,
  ahead, behind, dirty, health_score, test_state, risk_count,
  locked, git_lock { state, source, observed_at },
  archived, pull_request_url?, version, created_at, updated_at
}
```

`locked` 是 Worktree current projection 中的持久化兼容字段，可能由管理命令或历史 Git 导入写入，不能作为当前 Git 锁事实。`git_lock` 是独立 Host Runtime observation：`state ∈ {locked, unlocked, unknown}`，`source ∈ {host_runtime, unavailable}`，并带 `observed_at?`。这里的 Git 锁专指 [`git worktree lock`](https://git-scm.com/docs/git-worktree) retention lock：阻止 Git prune Worktree 管理记录，并限制 Git 对该 Worktree 的移动/删除；它不报告 Agent/Session 活跃状态，也不是共享文件编辑锁或租约。API 仅在可信 observer 返回时间戳、时间戳不晚于当前时刻且年龄不超过 30 秒时保留确定的 locked/unlocked；浏览器消费端再次要求 `source=host_runtime` 且时间戳可解析、未超前并在 30 秒内，否则一律显示 unknown；缺少 Worktree Runtime binding、observer 未装配、调用失败、无时间戳、过期或未来时间戳都归一为 unknown。列表读取仅在 actor、Project membership 与 Worktree binding 已验证后调用 observer；query 同时绑定 tenant/project/repository/worktree/runtime，避免跨 Runtime 查错对象。每页最多并发 8 项、单项 provider 等待上限 2 秒、整页等待上限 3 秒，超出等待时间的项目保持 unknown。`unknown` 或 `unlocked` 均不能证明 Agent 已停止；`owner_user_id` 表示人类责任归属；`agent_session_id` 表示会话关联，二者不得互相替代。缺失字段显示“未绑定/未知”，不得根据分支名或运行进程推断 owner。

`unknown` 既不表示 unlocked，也不能用来通过归档/清理 guard；fresh `unlocked` 只说明 Git retention lock 未设置，不表示无 Agent 在工作。`git_lock` 是管理面信号，不是物理删除授权：未来 cleanup plan/confirm 必须读取独立 Agent/Session/Runtime 活跃状态，先停止并 drain Agent/Session，再对同一 Repository/Worktree 实时复验 Git retention lock、dirty state 和 Runtime 状态，最后由授权 Repository/Runtime lifecycle executor 执行；本 observer 接口本身不执行 Git 命令或删除。

Index UI 通过 `WorktreeGroupApiClient.listProjectWorktrees` 请求当前 Project 的认证投影；校验 envelope 与每行的 `project_id` 后映射 owner、执行绑定、生命周期、PR、dirty/ahead/behind、health/risk、lock 与 version。页面只持有当前 Project 的游标分页结果，并把 owner UUID、human state、archived 筛选传给服务端，不把本地 seed 拼进 API 投影。宿主 provider 尚未安装时明确进入 preview；已配置 provider 后遇到 401/403、网络或 schema 错误则显示错误且不回退 seed。API 未返回 owner / Agent / Runtime 展示名称时，UI 显示 ID 或“未绑定”，不得从本地 seed 补名称。

Index 行上的状态由独立字段组成：Worktree lifecycle、健康探测、Agent Session、Runtime、Git 冲突、Git lock 与 PR 状态。不得将 WorkItem status 当成 Worktree status，也不得把没有冲突证据显示成“无冲突”。

### 3.2 RunContext 与兼容 GroupContext

```text
RunContext {
  tenant_id, workspace_id, project_id, branch_id, engineering_run_id,
  actor_id, actor_kind, actor_roles[], granted_scopes[],
  context_version, resolved_at, correlation_id
}

WorktreeFocus {
  worktree_id, repository_id, binding_version, observed_at
}
```

`RunContext` 仅由服务端基于 actor 的 Project/Branch/EngineeringRun membership 与当前 ACL 解析；`WorktreeFocus` 只为 checkout/Git/CLI 操作提供独立的当前 binding proof，不授予读取 Run Apps 的权限。缓存键包含 actor、Project/Branch/Run 与授权版本；membership、角色、Run ownership 或 Plugin grant 撤销后，相关缓存和实时订阅失效。旧 `/group-context` 响应中的 `GroupContext` 是迁移 DTO：兼容期可回传 `engineering_run_id` 与 `focus_worktree_id`，但不得继续把 Worktree ID 视为完整 App scope。任何 context 都不能从浏览器 Zustand、URL 参数或 LangGraph checkpoint 恢复为可信凭据。

### 3.3 EntityRef

跨 App 使用 `EntityRef { entity_type, entity_id, tenant_id, project_id, branch_id?, engineering_run_id?, worktree_id?, version? }`。Run-owned entity 必须绑定 `engineering_run_id`；仅 checkout/Git/CLI resource 必须使用 `worktree_id`。`entity_id` 是寻址字段，不是授权 token。resolver 必须核对真实 domain owner 与传入的 tenant/project/branch/run/worktree 一致；引用过期版本返回冲突，跨项目或越权引用返回统一不可见响应，避免泄漏实体存在性。

## §4 API 契约

所有生产 API 使用已验证 Bearer JWT。Run App 查询以 `engineering_run_id` 和 `RunContext` 为授权边界；Worktree lifecycle/CLI/Git 命令还需单独验证 focus binding。Index 使用有界 cursor 分页；Task Card 列表当前是有界 limit。成功响应目前为 JSON projection，统一 envelope 尚未接入。每个写命令须带 `Idempotency-Key`，状态转移 / Worktree 管理另带 `expected_version`；未提供 `correlation_id` 时由服务端生成。请求体不得接受 `actor_id`、`tenant_id`、`roles` 或 `granted_scopes` 作为授权来源。

Run-owned canonical API family:

| Method / path | 用途 | 授权 / owner |
|---|---|---|
| `GET /api/v1/projects/{project_id}/branches` | 左侧 Project 下的 Cloud Branch 目录 | 当前 Project membership；Branch Registry owner |
| `GET /api/v1/branches/{branch_id}/engineering-runs` | 展示 Branch 下的 Engineering Run | Project + Branch + Run visibility；Run Domain owner |
| `GET /api/v1/engineering-runs/{engineering_run_id}/context` | 解析 RunContext、App versions 与可选 Worktree set | 每次校验 actor membership/ACL；回传 context version |
| `GET /api/v1/engineering-runs/{engineering_run_id}/worktrees` | Run 下绑定的 Worktree 列表 | RunContext + Worktree `read` capability；不返回 path |
| `GET /api/v1/engineering-runs/{engineering_run_id}/apps` | Inbox/Work Items/Task Card/Canvas/Workflow/BI/Plugin tabs 投影 | RunContext + app open/capability grants；pagination + no-store |
| `/api/v1/engineering-runs/{engineering_run_id}/{app-resource}` | Run-owned App API facade；具体命令由对应 domain owner 执行 | Run membership + resource owner ACL；稳定版本化 DTO |

当前 Worktree routes 表示已存在的代码/迁移兼容 API，并不代表目标 canonical Run API 已实施。迁移顺序：新增 Branch/EngineeringRun registry 与授权 projection → 建立 Worktree-to-Branch/Run binding/reconciliation → 暴露 RunContext 与 Run App registry → 新 UI 切到 Run routes → 兼容 adapter 观测期 → 在所有 client 更新且旧路径访问归零后再按 API version deprecate。现有 routes 不得改成未经授权的 HTTP redirect；兼容 adapter 必须做身份/授权解析后显式返回新 deep link 或代理到 owner API。

| Method / path | 用途 | 必要授权 | 关键结果 |
|---|---|---|---|
| `GET /api/v1/projects/{project_id}/worktrees` | Index 游标分页与 owner/state/archive 筛选 | `project:read` + Project membership | Worktree 投影、`next_cursor` |
| `GET /api/v1/projects/{project_id}/members` | 读取负责人可选成员 | `project:read` + 当前 Project membership | 仅当前有效成员的 `user_id` / role；不用于跨 Project 搜索，写入时再次校验 membership |
| GET /api/v1/projects/{project_id}/worktree-import-candidates | 发现可导入 checkout；limit 默认 25、范围 1–50 | worktree:create + 当前 Project membership 与 writer role | 仅读可信 provider 的不透明 candidate_id、repository_id、branch、commit 与 dirty 状态；no-store，不返回路径/URL；provider 缺失返回 503 |
| POST /api/v1/projects/{project_id}/worktrees | 在 Project 已绑定 Repository 中创建 Worktree | worktree:create + tenant_admin / project_admin / developer | body 仅含 repository_id、branch、base_ref、可选 correlation_id；Idempotency-Key；provider 复核 membership/binding 并幂等持久化 operation、Worktree projection、Audit/Outbox 后返回 202 receipt |
| POST /api/v1/projects/{project_id}/worktrees/import | 导入已发现的 Host Runtime checkout | worktree:create + tenant_admin / project_admin / developer | body 仅含 repository_id、不透明 candidate_id、可选 correlation_id；Idempotency-Key；不接受路径、URL 或命令；返回 202 receipt |
| `GET /api/v1/worktrees/{worktree_id}/group-context` | 打开 Worktree Group 前解析当前上下文 | `worktree:read` + Project membership | `group_context` 与 Worktree 投影；尚无 `apps[]` / `allowed_actions[]` |
| `POST /api/v1/worktrees/{worktree_id}/work-items` | 创建 Task Card / canonical WorkItem | `work-item:write` + Project writer role | 任务、Task Card 同 ID，初始 `pending`, version 1 |
| `GET /api/v1/worktrees/{worktree_id}/work-items` | Task Card / Multica 当前 Worktree 任务列表 | `work-item:read` + Project membership | 有界列表、canonical `work_item_id` 与 lifecycle version |
| `GET /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}` | 读取 Task Card | `work-item:read` + 当前 Worktree association | metadata + Multica lifecycle projection |
| `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/lifecycle` | 更新 Multica 六态；由 Multica、Jira、Task Card 共用 | `work-item:write` + Project writer role | 合法迁移、`expected_version`、`Idempotency-Key`、`correlation_id`、claim lease / review gate 与 Transaction audit；成功/冲突后刷新 Group projection |
| `GET /api/v1/worktrees/{worktree_id}/canvases` | 查询当前 Worktree 的 Canvas | `worktree:read` + `canvas:read` + Project membership | 仅返回当前 Worktree registry 与元素数量 |
| `POST /api/v1/worktrees/{worktree_id}/canvases` | 创建 Worktree-owned Canvas | `worktree:read` + `canvas:write` + Project writer role | `Idempotency-Key`；Canvas Master version 1；同事务 Audit / Outbox |
| `GET /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/events` | 有界读取指定 Canvas 的 Outbox 事件元数据 | `worktree:read` + `canvas:read` + 当前 Project membership / Worktree scope | 可选完整 `(cursor_at,cursor_event_id)` 复合游标；默认 100、上限 200；不返回 payload / EntityRef 目标；不是 consumer 或实时推送 |
| `PUT /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/document` | 替换当前 Canvas Document 的 viewport / frames / visual connectors | `worktree:read` + `canvas:write` + Project writer role | `expected_version` + `Idempotency-Key`；SCD2/CAS、同 Canvas element reference 校验、Audit / Outbox 同事务；connector 不写 canonical WorkItem relation |
| `GET /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements` | 读取 Canvas 元素和当前可解析 EntityRef | `worktree:read` + `canvas:read` + Project membership | stale/越界 WorkItem 关联不解析；无 `work-item:read` 时隐藏 WorkItem ref；详情由 canonical WorkItem API 授权读取 |
| `POST /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements` | 创建 Canvas 元素，可同时绑定 typed EntityRef | `worktree:read` + `canvas:write` + Project writer role；绑定 WorkItem 另需 `work-item:read` | `Idempotency-Key`；元素/EntityRef Master 与 Audit / Outbox 同事务 |
| `POST /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/work-items` | 从 Canvas 一次性创建 canonical WorkItem 与任务卡元素 | `work-item:write` + `canvas:write` + Project writer role | `Idempotency-Key`；metadata、Multica lifecycle、Worktree link、Canvas Element/EntityRef、Task Audit、Canvas Audit/Outbox 同事务；共享 `correlation_id` |
| `PUT` / `DELETE /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements/{element_id}` | PUT 更新布局、内容或未锁定 Element 几何；DELETE 逻辑移除 | 同上，另带 `expected_version` 与 `Idempotency-Key` | PUT 保留未提交修改的其他字段并校验范围；DELETE 同事务推进 Document 并清理悬空引用，不物理删除 WorkItem |
| `PUT` / `DELETE /api/v1/worktrees/{worktree_id}/canvases/{canvas_id}/elements/{element_id}/entity-ref` | 设置、替换或解除元素的 typed EntityRef | 同上，另带 EntityRef `expected_version`；绑定 WorkItem 需 `work-item:read` | WorkItem 必须是同 tenant/project 且当前关联当前 Worktree 的 canonical WorkItem |
| `POST /api/v1/worktrees/{worktree_id}/management-plans` | 预览 owner reassignment 或归档 / 恢复 | `worktree:manage` + `tenant_admin` / `project_admin` | 5 分钟 plan；必须带 `Idempotency-Key` 与当前 version |
| `POST /api/v1/worktrees/{worktree_id}/management-plans/{plan_id}/confirm` | 确认 Worktree owner / archived 变更 | 同一 requester、当前 membership、原 version | 版本条件更新、Owner SCD2、不可变 Audit |
| `POST /api/v1/worktrees/{worktree_id}/cleanup/plan` | 生成清理预检 | `worktree:cleanup` | 只读检查、阻断原因、短期 `plan_id` |
| `POST /api/v1/worktrees/{worktree_id}/cleanup/confirm` | 执行过期清理计划 | `worktree:cleanup` + 二次确认 | 幂等 operation result 与 Audit ref |
| `GET /api/v1/worktrees/{worktree_id}/agent-sessions` | 查询 Agent Session 历史 | `agent-session:read` + 同一 GroupContext | cursor 分页与当前/历史标记 |
| `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions` | 在当前 Task Card / Worktree 请求启动 CLI Session | Bearer actor + `agent_session:start` + 当前 Project writer membership + canonical Task/Worktree association | `expected_lifecycle_version`、当前 claimant、`in_progress`、Runtime binding 与 profile ID（由受信任 provisioner 对照服务端目录）；`Idempotency-Key`、`correlation_id`；request fingerprint 覆盖 actor 与完整 tenant/project/repository/worktree/task/runtime/profile/version 执行上下文；只有实际运行并签发 ≤60 秒 single-use ticket 后成功返回，ticket 响应 `Cache-Control: no-store` |
| `GET /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions?limit=N` | 列出当前 Task/Worktree 最近 CLI Sessions，用于页面刷新后发现 session | Bearer actor + `agent_session:read` + 当前 Project writer membership + canonical Task/Worktree association + `X-Correlation-ID` | limit 默认 20、范围 1–50；Runtime 再核验完整 binding；仅返回 session ID、有限状态、exit code、updated_at；禁止返回 attachment ticket 与 PTY/output 数据；响应 no-store；无 provisioner 返回 503 |
| `GET /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}` | 查询 Task CLI Session 状态 | Bearer actor + `agent_session:read` + 当前 Project writer membership + canonical Task/Worktree association + `X-Correlation-ID` | Runtime 再核验完整 session binding / 当前 ACL / Runtime health；仅返回有限状态、exit code、updated_at，不含 PTY 数据或 ticket；响应 no-store |
| `DELETE /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}` | 请求取消 Task CLI Session | Bearer actor + `agent_session:cancel` + 同一当前 membership / canonical Task scope + `X-Correlation-ID` | provisioner 按完整 binding 幂等取消并写关联 TaskRun Audit；不跨进程保留 DB transaction |
| `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/cli-sessions/{session_id}/attachment-tickets` | 为浏览器断线恢复签发新的 attachment ticket | Bearer actor + `agent_session:attach` + 同一当前 membership / canonical Task scope + `X-Correlation-ID`；Worktree 不得归档 | Runtime 确认 session 仍可附加且重验 policy 后签发新鲜 ≤60 秒单次 ticket；旧 ticket 不复用；no-store |

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
| GET /api/v1/projects?limit=&cursor= | RS256 Bearer；project:read | 按当前 actor tenant/user 查询有效 Project Role Binding；仅返回 `project_id` 与 role，UUID keyset cursor，limit 默认 100、范围 1–200，no-store；不返回 Project 名称/仓库 URL，生产名称等待 Project SoR |
| GET /api/v1/projects/{project_id}/worktrees | RS256 Bearer；project:read；active Project Role Binding | 服务端 tenant + project 过滤；owner/state/archive 筛选；按 `(updated_at,id)` 倒序 cursor 分页，cursor 绑定当前查询参数 |
| GET /api/v1/projects/{project_id}/members | RS256 Bearer；project:read；active Project Role Binding | 仅投影该 Project 当前有效成员 ID 与 role；`assign_owner` plan / confirm 再次验证目标 membership |
| GET /api/v1/projects/{project_id}/worktree-repositories | RS256 Bearer；project:read；active Project Role Binding | 调用受信任 lifecycle provider 查询当前 Project 已绑定仓库；仅返回 repository_id/name/default_branch，不返回 URL 或 checkout path；最多 100 项并校验重复 ID、路径/URL 型名称和 ref；provider 未安装返回 503 |
| GET /api/v1/projects/{project_id}/worktree-import-candidates?repository_id=&limit= | RS256 Bearer；worktree:create；active Project Role Binding + writer role | provider 返回当前 Project/Repository 的 opaque candidate_id、branch、commit、dirty 和 observed_at；limit 默认 25、范围 1–50；不返回 path/URL，provider 未安装返回 503 |
| POST /api/v1/projects/{project_id}/worktrees | RS256 Bearer；worktree:create；tenant_admin / project_admin / developer | 严格请求 DTO；仅接受已绑定 repository_id、合法 branch/base_ref 与可选 correlation_id；Idempotency-Key；provider 在 side effect 前重验 membership/binding，并在返回 202 前持久化 operation、Worktree projection、Audit/Outbox |
| POST /api/v1/projects/{project_id}/worktrees/import | RS256 Bearer；worktree:create；tenant_admin / project_admin / developer | 严格请求 DTO；仅接受 repository_id、可信 opaque candidate_id 与可选 correlation_id；Idempotency-Key；不接受路径、URL、命令；返回关联 Project/Repository/correlation 的 202 receipt |
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

GroupContext 当前尚未返回 `allowed_actions[]`。Phase 2D 已实现的 plan/confirm API 由服务端重新检查 scope、当前成员角色、Worktree version 和目标 owner membership；Index 已有条件式 owner reassignment 与 archive/restore plan-confirm UI，但宿主 provider 未安装，因此产品运行态仍是 preview，不能报告为已启用的服务端管理能力。

| 动作 | 预检 | 写入规则 |
|---|---|---|
| 创建 / 导入 | Project Repository binding 可见；branch/base_ref 符合 Git ref 约束；导入使用可信候选 ID | 三条 fail-closed API contract 已实现；Host Runtime provider、可信 binding/路径解析、operation + Worktree + Audit/Outbox writer 与 Index UI 未接通，故生产 create/import 仍不可用 |
| 转派 owner | `tenant_admin` / `project_admin`；目标用户仍是该 Project member | 已实现 5 分钟 plan/confirm、expected version、SCD2 owner assignment 与 before/after Audit |
| 归档 / 恢复 | `tenant_admin` / `project_admin`；已有 AgentSession / Runtime 关联时拒绝归档 | 已实现 plan/confirm、expected version、保留截止时间与 Audit；检查当前只按关联 ID 非空保守拒绝，未接 Runtime 活跃状态查询 |
| 清理 | 先跑 plan：Agent / CLI inactive、Runtime detached、Git lock absent、保留策略满足 | 仅使用未过期 plan；二次确认、幂等执行、审计实际 Git 操作 |
| 取消 / 恢复 | operation 与当前 lifecycle 匹配 | 版本条件更新，记录恢复来源和失败原因 |

Project lifecycle provider contract 要求：请求只携带 Project 内 repository_id 与 branch/base_ref，导入只携带 Host Runtime candidate_id；所有 checkout 路径和远端 URL 从服务端受信任 Repository registry/config 解析。API 在数据库事务中校验当前 Project membership 与 worktree:create scope，provider 在任何 Git 副作用前再次检查 membership、Project-Repository binding 和当前 Worktree 冲突。provider 必须以稳定 Idempotency-Key/fingerprint 去重，并在返回 202 前持久化 operation、Worktree/Project 归属、Audit 与 Outbox；receipt 只包含 operation/worktree/project/repository/branch/state/correlation，不暴露主机路径。未装配 provider 时一律返回 503。当前 API DTO 对未知字段拒绝，阻止 path/repo_url 注入；该接口尚无 Host Runtime 实现或 Index 控件。

归档与 Git Worktree 物理清理是不同操作。Phase 2D 只做数据库归档标志，不运行 `git worktree remove`，也不探测 Git lock；在 Agent / CLI / Runtime 状态数据源和 cleanup plan 接通前，清理动作不可用。已绑定 Session / Runtime 的 Worktree 目前一律不允许归档，避免把未知状态误认为空闲。

#### 6.1.1 Index UI 的管理计划绑定

当前 Group API provider 已存在注入契约，但宿主尚未在应用路由树内装配，因此 Worktree Index 默认仍使用明确标注的本地 preview。宿主装配后，Index 按 `project_id` 调用授权 cursor API；切换 Project 时丢弃旧列表和 cursor，加载失败时保留错误态，不得以 seed 伪造实时数据。owner UUID / human state / `include_archived` 均作为服务端列表筛选并与 cursor 绑定；恢复入口必须从包含 archived 的授权结果打开。负责人候选通过 `GET /api/v1/projects/{project_id}/members` 读取；UI 只显示当前 Project 有效成员的 UUID 与 role，并依据响应中的当前 Project role 只向 `tenant_admin` / `project_admin` 展示转派操作；目录加载失败时关闭转派入口，服务端 plan 与 confirm 始终复核权限和成员状态。

live Worktree detail 当前接通 `assign_owner`、`set_archived` 与 restore UI 命令：客户端以行上的 `version` 和新生成的 correlation/idempotency key 请求短时 plan，验证返回的 `plan_id`、`worktree_id`、operation 与 `expires_at` 后展示独立确认按钮；过期计划不可确认。负责人只能从当前 Project 成员目录选择，确认前服务端仍重验 manager role、membership、plan 所属 requester、目标成员有效性、当前 version 和执行关联。确认后重新读取当前授权列表；请求失败保留错误，不乐观改 owner 或 archived 状态。UI 在已存在 AgentSession 或 Runtime 引用时预先禁用 archive，但该判断只是提示，服务端仍是最终守门。创建/导入、Runtime stop/drain 与物理 cleanup 不属于此 slice。

### 6.2 并行 Agent 可见性

Index 同时呈现一个 Worktree 的 owner、当前 Agent、活跃 Runtime、PR 和风险信号；执行历史由 Session 列表展开。一个 WorkItem 可关联 `0..N` Worktrees；每次 Agent / CLI run 必须选择一个授权 Worktree，并在 Run 上记录 `active_worktree_id`。Agent 不能因 Task Card 关联而获得其它 Worktree 的读取权。

冲突检测分为 Git merge/rebase 冲突、同一 Worktree 的并发写入占用和资源健康错误；三者分别表示，不用单个“locked”状态混写。UI 展示最后一次检测时间与 source version；过期探测显示“未知/待刷新”。

### 6.3 多 Agent 调度、资源预算与并发控制

Project Coordinator 负责跨 Worktree admission 与公平份额，Worktree Coordinator 负责 checkout/文件 claim/Agent lease，TaskExecutionRun 固定每次尝试的输入、资源预算与执行事实；每层继承父层剩余预算，不能因拆分子 Agent 扩大总预算。ready DAG 节点才可入队；队列有界，采用带权公平调度和 Project/Worktree 并发上限，背压时产生可见等待/拒绝原因，不在内存无限排队。

Run budget 至少包括 memory/RSS、CPU 并发、wall-clock deadline、子进程数、fd、事件缓冲与 provider/tool 并发；每个子 Run/Plugin 使用父 budget 子额度。真实资源采样保留 unit/source/time window，估值与实测分开；Run 只追加完成时 high-water `resource_summary`，高频采样进入短 TTL 环形监控，不形成无限增长的事务历史。未知读数保持 unknown。

同一 checkout 的文件写入通过 coordinator lease 与 path claim 冲突检查；不同 Worktree 的 Agent 可独立并行，跨 Worktree shared resource 再使用细粒度锁/幂等 command。Git retention lock 只表达 Git 行政保护，不能授权文件写、证明 Agent 空闲或代替 drain。取消/deadline/撤权/归档沿 Run→Agent→tool/process 传播并等待释放句柄；observer stale、drain incomplete 或冲突时显示 unknown/block，不推测安全。

### 6.4 Schedule Loop 与 Engineering Loop

Schedule Loop 的唯一计划定义与 occurrence source 是 `domain-automation` 的版本化 `AutomationRule` / `AutomationOccurrence`；Group/Workflow 只接收已创建的 occurrence 并为目标 Task 派生 Run，不另存 Cron 表或自行计时。`star-scheduler` 只解析依赖 DAG 的 ready 节点，不是 wall-clock scheduler；LangGraph/Workflow 负责已启动 Run 内的编排，不是第二个 schedule owner。规则保存 timezone、并发/overlap、misfire、retry、deadline、pause 与目标 scope；每个 occurrence 有稳定 ID、fencing lease、幂等分发和可审计 stop reason。当前 Schedule/Cron 仍是候选契约，尚未形成生产定义或 worker。

Engineering Loop 是单一 `TaskExecutionRun` 内版本化、预算受限的 Plan/Act/Observe/Verify/Decision 周期。每轮追加 Loop event 和必要 Evidence；不得改写 Run 的 Task Contract/acceptance/profile/HookSet 快照。达到迭代、deadline、CPU/RSS、子进程或 provider 请求上限，检测到无进展/振荡，或发生撤权/cancel 时停止接收新动作、取消并 drain child，再记录 stop reason 和 drain 结果。循环度量来自 durable occurrence/RunEvent/Evidence/Audit；单纯增加轮数或调用量不算成功。

## §7 Task Card、CLI 与 Multica/Jira 联动

1. Phase 2C 新 API 使用一个 canonical `work_item_id`，`task_card_id` 等于该 ID；WorkItem metadata 与 Multica 六态 lifecycle 分表持久化。响应中的 `lifecycle.version` 是 Multica 当前版本。旧三态 `domain-work-item::InMemoryWorkItemService` / legacy REST 不作为生产 Group 数据源。
2. Task Card 打开时展示当前 Group Worktree。若 WorkItem 未关联当前 Worktree，允许查看并提示关联/选择 Worktree；在关联确认之前不允许启动 CLI、Agent 或写入 Worktree 文件。
3. 服务端构造 `TaskExecutionContext { actor, project_id, repository_id, worktree_id, work_item_id, runtime_id, approved_profile_id, policy_version, expiry, nonce }`。路径、命令白名单、环境变量和 Secret 句柄均由服务端 policy 与 Runtime 配置派生，客户端不能提交可覆盖值。
4. Local Runtime 只接受短期、签名且一次性的 session grant；启动、输入、退出和 session 重连均重新验证范围。审计记录 session ID、命令类别、结果摘要和退出码，不把任意终端内容复制进 WorkItem 描述。
5. 当前实现支持 pending → claimed → in_progress → completed / failed / cancelled 与 failed → pending 的状态转移；`review_state` 独立保存。Group API 已增加版本化 review command：当前 claimant 可提交 `in_progress → pending_review`（主状态保持 in_progress）；非 claimant 的当前 Project `tenant_admin` / `project_admin` / `developer` 可通过或驳回；驳回要求理由，通过转 `completed`，驳回转 `failed`。所有动作锁定当前 lifecycle 行、检查 Worktree association / version / idempotency，并写 append-only lifecycle audit。它仍是 REST SQL coordinator 代码切片，未抽为 Review Domain port、未产生 Outbox event；poison/stale-dispatch 与 Jira 外部 alias/sync 也尚未落地，不能据此宣称 Multica/Jira 生产链路完成。

Review route 为 `POST /api/v1/worktrees/{worktree_id}/work-items/{work_item_id}/review`，请求携带 `action=submit|accept|reject`、`expected_version`、`correlation_id`，并通过 `Idempotency-Key` 去重。submit 要求当前 `claimed_by=actor_id` 且 Worktree 一致；accept/reject 需 reviewer 角色且 actor 不等于 claimant。服务端返回同一 canonical WorkItem 投影。Group UI 仅在 provider-backed live projection 下显示提交/通过/驳回入口；驳回原因必填，成功或冲突刷新 projection。权限仍由 API 最终裁决，前端入口不作为 authorization。

Phase 3F 将该 API 接入 Group UI 的 Multica、Jira 和 Task Card 视图。前端使用投影中的 `lifecycle.status / version / review_state / active_worktree_id` 显示动作；动作集合限定为 pending→claimed、claimed→in_progress/cancelled、in_progress→completed/failed/cancelled、failed→pending；review 动作在当前 `in_progress` 中独立显示。`claimed→in_progress` 仍由服务端校验 claim actor 与 lease；active Worktree 不匹配时 UI 不提供执行动作；`pending_review` 时隐藏直接完成操作。每个命令携带当前 version、correlation ID 和幂等键；成功及冲突都刷新 Worktree 授权投影。在线 Jira 按完整六态及 review 状态分列，所有视图仍读取同一 WorkItem。

Phase 4A 在 `domain-local-runtime::task_execution` 提供执行请求预备边界：低层 `prepare_task_cli_execution` 只校验已认证 context 的字段与 scope；新增 `prepare_and_consume_verified_task_cli_execution` 接收签名 envelope，并在执行准备和 nonce 消费前验证 issuer 签名。上下文必须与 RuntimeWorktreeBinding 的 tenant/project/repository/worktree/runtime 完全一致，profile ID 必须匹配，grant 生命周期不超过 5 分钟；Runtime 上报 mount path 经 `canonicalize` 后必须等于 Worktree/Git 服务解析出的 canonical checkout，executable 必须为 profile 固定的绝对文件路径。profile 固定 args 与非敏感 `static_environment` 从服务端配置复制，不接收浏览器命令文本；token、secret、password、credential、private key 等敏感环境名会被拒绝，秘密值必须由独立、可审计的 secret capability 提供。路径字符串校验本身不能防止检查后替换目录的 TOCTOU；实际 runner 必须将 Runtime mount 与 Worktree 身份绑定，并在受控执行器中维持该绑定。

纯校验函数 `prepare_task_cli_execution` 不消费 nonce；`prepare_and_consume_task_cli_execution` 会在校验 scope/profile/checkout 后，通过 `CliSessionRegistry` 的专用 SQLite WAL `synchronous=FULL` 连接，以 `BEGIN IMMEDIATE` + 全局 nonce 主键原子消费。该 W 记录在 grant expiry 后保留 5 分钟时钟偏差窗，并在后续请求中懒清理。可信入口应调用 `prepare_and_consume_verified_task_cli_execution`：它按 `key_id` 从 Runtime 公钥表选择 Ed25519 验证密钥，对包含完整 context 的 `star.task-cli.execution-grant.v1` 域分离 payload 验签，再校验 scope/profile/checkout 并原子消费 nonce；未知 key、坏签名与 nonce replay 均 fail-closed。私钥只部署在 API grant issuer，Runtime 只配置可轮换公钥，并至少保留 retiring key 至在途 grant 过期（最长五分钟）。该签名不替代 spawn 前的当前 ACL / Runtime health 重验；helper 尚未接入 Session API、审计或进程启动。

仓库已有通用 SQLite `CliSessionRegistry` 与 pipe-based `RealCliRuntime`，它们原先仅按 tenant/worktree 管理通用命令，不带 TaskExecutionContext 或任务卡授权；pipe stdio 不能替代交互 PTY。Phase 4B2 已在 `domain-local-runtime::task_execution::pty` 新增 crate-internal `TaskPtyManager` adapter：只接受 prepared execution 的固定 executable/argv/cwd，以 `portable-pty` 建立交互 PTY，清空进程继承环境后注入批准的静态环境，限制输入 chunk 和 cols/rows，输出未经字符转换的字节块并附 session 内序号。输出经单消费者、有界 FIFO 队列传递，消费者慢时以背压暂停 PTY reader，避免 attachment 建立前静默丢失；manager 退出时终止所拥有的子进程。该 adapter 不提供 ACL、grant 验签、nonce 消费、Runtime health、审计、scrollback 持久化或 OS sandbox；调用方仍须在调用前完成全部授权与审计，且隔离执行器缺失时不得调用。`terminal-stack` 现有 demo handler 仍用 `NoopTerminalEventSink` 并 lazy-register 任意 pane，因此不得作为 Task CLI 入口。Phase 4B1 新增 `POST .../cli-sessions` Group REST route：验证 JWT actor、`agent_session:start`、当前 Project writer membership、canonical WorkItem-Worktree link、`in_progress` claimant、lifecycle version 与 Runtime binding；`GroupApiState` 可注入 `TaskCliSessionProvisioner`，默认未装配时返回 503。接口要求 provisioner 按 tenant/actor/idempotency key 去重并校验 request fingerprint，且必须在签 grant/spawn 前重新检查 ACL、Worktree/task version、Runtime health、Approved Profile 与 sandbox；REST 事务已结束，前置查询不能代替这次实时复核。当前没有真实 provisioner、生产 authorizer、签名调用、spawn 或 PTY sink 接线，因此路由只建立 fail-closed 接线边界，不代表 Task CLI 已能运行。Phase 4B 另有受保护 WebSocket route seam：首帧要求 authorization frame，授权成功后才注册 pane 和发送 snapshot；state 必须注入 `WsAttachmentAuthorizer` 与 `TerminalEventSink`。Local Runtime ticket ledger 仅存 SHA-256 摘要并原子单次消费。

Phase 4B 按以下顺序接通：

1. **Session API 与 grant**：REST start route 已有 authenticated route seam 和 DB 前置校验；下一步由真实 provisioner 在 grant 签发 / spawn 前重查 GroupContext、membership、Task/Worktree version、Runtime health、Approved Profile 与当前 policy；API signer 生成短时 Ed25519 grant，Local Runtime 验签、复验 Runtime/checkout binding，并在 spawn 前通过 durable nonce helper 原子消费。start/cancel/reattach/status 都写同一 `correlation_id` 下的 TaskRun Audit。当前 seam 不会签发 grant 或创建 session。
2. **PTY 与 terminal attachment**：`TaskPtyManager` 已提供 PTY process adapter：仅使用 prepared execution 的固定 executable/argv/cwd，清空父进程环境，接受获批静态变量，限制输入 chunk 与终端尺寸，输出经单消费者有界 FIFO 队列保留原始字节块和序号，消费者迟缓时 reader 背压；manager 退出时终止所拥有的子进程；它目前不连接 session API 或 `terminal-stack`，也不提供持久 scrollback。Local Runtime 仍须管理 PTY、stdin/stdout/stderr、resize、退出和 Runtime 心跳；spawn 前必须确认 OS-enforced sandbox / path jail 已就绪，否则拒绝启动。PTY 不是隔离边界。`terminal-stack` adapter 必须先证明 session 已启动且仍绑定原 Task/Worktree，不能从任意 pane ID 创建进程。WebSocket 不带 Bearer JWT；REST start 成功后签发短时、单次 ticket，绑定 actor、tenant/project/repository/worktree/work_item/session，浏览器通过首个 `authorize` frame 提交。服务端原子消费 ticket 并重新检查当前 membership/session 状态，成功前不注册 pane、不发送 snapshot、不接受 stdin/resize；日志只写 ticket 摘要或 session ID。
3. **前端 Task Card**：Group 页面从受认证的 Session API 获取 session ref 和一次性 ticket，传给 `TerminalStackContainer`；CLI 面板在未获 session ref 时明确显示不可连接原因，不把 mock terminal / `sessionId` 当成授权或成功状态。终端输出在进入 Task Context / LangGraph 前执行 secret redaction 并标记为 untrusted content。

已落地的 transport / ledger 契约：attachment ticket 绑定 `tenant_id / project_id / repository_id / worktree_id / work_item_id / actor_id / runtime_id / session_id / policy_version`；ticket TTL ≤ 60 秒；Local Runtime SQLite ticket ledger 只保存 SHA-256 摘要、在 `BEGIN IMMEDIATE` 中原子消费，并于 expiry + 5 分钟后懒清理。WebSocket 第一帧限制 2 KiB，后续消息上限 1 MiB；每次 reconnect 必须通过 REST 获取新 ticket。`TerminalWsClient` 收到服务端 `Hello` 前保持未授权/未连接。受保护路由先消费 ticket，再注册 pane / 加载 snapshot，并要求显式注入真实终端 sink。具体 runtime authorizer 必须先重新检查当前 ACL 与 session/policy 版本，再消费 ticket；本次代码尚未提供该 authorizer。

Group 前端已有逐请求调用的 `GroupAccessTokenProvider`、API adapter 和 API-backed read projection，但宿主登录 provider 尚未安装到 Group 路由，因此当前产品运行仍是 preview；不得用 shared `NEXT_PUBLIC_API_KEY` 冒充用户 actor。Task Session start REST seam 已接入 Group router，但当前 `main` 未配置 provisioner，也没有真实用户会话 provider 来调用它；ticket ledger 已在 Local Runtime SQLite FULL-sync ticket connection 内实现，但尚无真实 session authority 调用该 ledger。Phase 4B3 已将卡内 start/status/cancel/reattach adapter 和 xterm 条件式接入 Group 页面：只有 live Group projection 下的进行中 WorkItem 与批准 Profile 可提交；Session receipt 返回后才挂载终端，ticket 保存在组件内存并单次用于 WebSocket 首帧，Hello 授权前不启用 stdin；用户可刷新脱敏状态、幂等取消，并在连接失败/断开后先检查 session 状态，仅 `running` / `disconnected` 才由服务端签发新 ticket。自动重连关闭。Session 关联不跨页面持久化 ticket；Phase 4B4 已增加 Worktree/Task-scoped bounded listing route 与刷新后手动发现 UI，但生产 provisioner 未装配时仍返回 503。该 UI 不代表当前产品已有宿主登录态、真实 provisioner、PTY sink 或 sandbox。附加连接不能延长 CLI grant 或绕过当前 ACL。

Phase 4B 新增 status、cancel、reattach REST route seam：所有请求要求 `X-Correlation-ID`，先重验 Project writer membership 与当前 canonical Task/Worktree 关联，再把含 tenant/project/repository/worktree/task/actor/runtime/session/correlation 的授权上下文交给 provisioner；Runtime 必须二次核验当前 session 的完整绑定、ACL 与 health。未知或错绑 session 对外统一 404；status 投影排除终端内容；cancel 在 provider 内幂等并以 correlation 记录 Audit；reattach 仅产生新鲜 ticket。Task Card UI 已通过 Bearer API client 调用这些 route：status/cancel 使用新的 correlation ID；断线恢复先查状态，只接受 `running` / `disconnected`，再校验 reattach receipt 的 session、Worktree、Task、Runtime 绑定及不超过 60 秒的 ticket expiry 后重建终端。ticket 不写 URL 或浏览器持久化；刷新页面后可经受授权 listing 发现 Session；因真实 provisioner 尚未装配，该路径仍仅是条件式 API/UI 切片。当前 GroupApiState 未装配生产 provisioner，真实 provider 缺失时仍返回 503；前端不得重用已消费 ticket。

## §8 Canvas、Chat、LangGraph 与 Plugin 的交互契约

### 8.1 Canvas

**Run-owned target contract:** Infinite Canvas 与 Task Card 是 Engineering Run 的同级 Apps；Canvas owner 为 `engineering_run_id`，一个 Run Canvas 可引用该 Run 下多个 Worktree，并通过 `EntityRef { entity_type, entity_id, engineering_run_id, worktree_id?, version? }` 链接。当前 Canvas 数据/API/UI 仍以 Worktree ID 为路由和数据库 scope，是迁移兼容 slice，不能标记为 Run ownership 已完成。迁移时新增 Run binding 与 RLS/schema contract，将旧 Canvas association 映射到唯一 Run；认证 adapter 先解析/验证 Run，再调用 Canvas owner API；非法或多重归属进入 reconciliation queue 并 fail closed。

以下 `worktree_id` 路由、表与单 Worktree Canvas 查询/写入规则描述现有兼容切片；它们必须经 Run binding adapter 校验归属。迁移目标是 Run-owned Canvas registry 与 Run RLS，不能将旧实现细节解释为目标层级。

Group Canvas 与 Task Card、Multica、Jira 同级；Canvas Element 仅持有带类型 `EntityRef { ref_type, ref_id, worktree_id }`，不持有 WorkItem 状态副本。Canvas registry 以当前 Worktree 为唯一查询边界；`project` / `free` Canvas 和旧元素中的裸 `work_item_id` 不构成 Worktree 绑定证据。双击任务 Element 导航到保留当前 Worktree 的 `app=task-card&work_item_id={id}`；后端须同时验证 EntityRef 的 `worktree_id`、当前 Worktree 的 canonical WorkItem association 与 Project ACL。“创建任务”“关联任务”“变更状态”“建立关系”须走对应事实 owner 的 Application Command，再由 Outbox 更新 Canvas 与其它投影。Project 级 Worktree Overview Graph 与 Worktree Group Infinite Canvas 分路由、分查询范围、分用户目的。

一个 Worktree 可以登记多个 Canvas。Group route 的 `canvas_id` query 参数选择其中一个可访问 Canvas，也使选择可以分享和恢复；缺省时投影选首项，非法或越界 ID 不发送跨 Worktree 查询，而回退到服务端返回的有效 Canvas。Canvas picker、列表、元素投影及 Outbox poller 都限定在同一 `worktree_id`，poller 随选择切换。新建命令使用 `Idempotency-Key`，API 返回 `canvas.canvas_id` 后页面更新路由并刷新授权投影；请求成功但投影暂未确认时保留同一待确认命令，避免重试重复创建。

将既有 Task Card 加到当前 Canvas 使用 `POST .../canvases/{canvas_id}/elements`，请求一次携带 `kind=work_item_card`、布局、`entity_ref={ref_type:work_item,ref_id,worktree_id}`、`correlation_id` 与 `Idempotency-Key`。`create_element` 在同一 transaction 下校验当前 Canvas 与 WorkItem 的 Worktree association，再写 Element、EntityRef、Audit、Outbox 和幂等响应；客户端不拆成两步，也不接受来自其它 Worktree 的 Task Card。Group UI 只列出尚未出现在当前 Canvas 的 WorkItem；成功后刷新授权投影并提供 Task Card 深链，失败则保留命令身份供同一操作重试。

将既有 Task Card 加到当前 Canvas 使用 `POST .../canvases/{canvas_id}/elements`，请求一次携带 `kind=work_item_card`、布局、`entity_ref={ref_type:work_item,ref_id,worktree_id}`、`correlation_id` 与 `Idempotency-Key`。`create_element` 在同一 transaction 下校验当前 Canvas 与 WorkItem 的 Worktree association，再写 Element、EntityRef、Audit、Outbox 和幂等响应；客户端不拆成两步，也不接受来自其它 Worktree 的 Task Card。Group UI 只列出尚未出现在当前 Canvas 的 WorkItem；成功后刷新授权投影并提供 Task Card 深链，失败则保留命令身份供同一操作重试。

Phase 3B migration 将 `canvas.group_canvas_registry`、`canvas.canvas_elements_backend`、`canvas.canvas_entity_ref` 定义为 Master/SCD2；`canvas.canvas_group_audit` 与 `canvas.canvas_group_outbox` 是 Transaction/append-only。Phase 3C additive migration 在 Canvas registry 上以同一版本序列保存 `viewport`、`frames`、`connectors`，不另建并行 Canvas document aggregate。新 Group API 支持 Worktree Canvas 查询/创建、元素查询/创建/版本更新/逻辑移除、EntityRef 设置/替换/解除及 Document CAS 全量替换。Phase 3D 增加 `POST .../canvases/{canvas_id}/work-items` 原子命令：一个 PostgreSQL transaction 写 canonical WorkItem metadata、Multica lifecycle、当前 Worktree link、`work_item_card` Element、typed EntityRef、Task Audit、Canvas Audit/Outbox 与幂等响应；同时校验 `work-item:write`、`canvas:write`、当前 writer role 和 AI task 的 repository scope，任一写入失败整体回滚。该命令不改变 Canvas registry 的 Document version；Element 有自己的初始 version=1。

Phase 3E 增加 `GET .../canvases/{canvas_id}/events` 有界轮询查询：每次请求重新检查当前 actor scope、Project membership、Worktree binding 与 Canvas 存在性，在当前事务/RLS scope 下按 `(occurred_at,event_id)` 递增读取；游标时间和事件 ID 必须同时提供或同时省略，`limit` 默认 100 且范围为 1–200。响应不包含 Outbox payload（其中可能有旧 WorkItem ID）或 EntityRef 目标，只返回 event ID、类型、schema / aggregate version、actor、correlation 和时间等刷新提示。查询索引通过 additive migration `2026-09-29-worktree-group-phase-3e-canvas-outbox-index.sql` 按 `(tenant_id,worktree_id,canvas_id,occurred_at,event_id)` 支持范围过滤与 keyset 游标。客户端应基于提示重新读取当前授权的 Canvas Document / Elements；浏览器已实现按 Worktree/Canvas scope 恢复的本地持久游标 at-least-once poller；每次请求仍走授权 API，且仅在当前授权 projection refresh 成功后持久化并前移游标，失败时重放当前页。本地游标是可丢弃的刷新提示，不是授权凭据、服务端 durable consumer offset、NATS consumer 或实时推送。Group Canvas 页面在 provider-backed live projection 下调用 `POST .../canvases/{canvas_id}/work-items` 创建 canonical Task Card；服务端使用同事务写入 WorkItem、Multica lifecycle、Worktree association、Canvas Element / EntityRef、Audit、Outbox 和幂等响应；客户端仅在请求成功后重载 WorkItem/Canvas/Elements projection，并提供任务卡链接。没有宿主 provider 时不显示真实写入口。Frame 创建/删除/标题/几何/演示标记、元素加入/移出 Frame、纯视觉连线创建/删除及展示样式已有条件式 Document CAS UI；Frame geometry/connector styling、无实体引用 Text/Sticky Note 内容 CAS 与事务性 Element 删除已有条件式 UI/API 切片；Element 尺寸/旋转已有条件式 Element CAS UI/API 切片；durable consumer/realtime 和 Canvas/Jira relation command 仍未完成。
Phase 3E 增加 `GET .../canvases/{canvas_id}/events` 有界轮询查询：每次请求重新检查当前 actor scope、Project membership、Worktree binding 与 Canvas 存在性，在当前事务/RLS scope 下按 `(occurred_at,event_id)` 递增读取；游标时间和事件 ID 必须同时提供或同时省略，`limit` 默认 100 且范围为 1–200。响应不包含 Outbox payload（其中可能有旧 WorkItem ID）或 EntityRef 目标，只返回 event ID、类型、schema / aggregate version、actor、correlation 和时间等刷新提示。查询索引通过 additive migration `2026-09-29-worktree-group-phase-3e-canvas-outbox-index.sql` 按 `(tenant_id,worktree_id,canvas_id,occurred_at,event_id)` 支持范围过滤与 keyset 游标。客户端应基于提示重新读取当前授权的 Canvas Document / Elements；浏览器已实现按 Worktree/Canvas scope 恢复的本地持久游标 at-least-once poller；每次请求仍走授权 API，且仅在当前授权 projection refresh 成功后持久化并前移游标，失败时重放当前页。本地游标是可丢弃的刷新提示，不是授权凭据、服务端 durable consumer offset、NATS consumer 或实时推送。Group Canvas 页面在 provider-backed live projection 下，当 Worktree 尚无 Canvas 时先调用认证、幂等的 `POST .../canvases` 初始化 Canvas；只有命令成功并刷新授权投影后才开放 `POST .../canvases/{canvas_id}/work-items` 创建 canonical Task Card。服务端使用同事务写入 WorkItem、Multica lifecycle、Worktree association、Canvas Element / EntityRef、Audit、Outbox 和幂等响应；客户端仅在请求成功后重载 WorkItem/Canvas/Elements projection，并提供任务卡链接。浏览器在同一页面生命周期内保留待确认创建命令的完整请求体、correlation ID 与 idempotency key，成功响应后才清除；改变任务内容会开启独立命令。任一命令失败均不回退 seed。没有宿主 provider 时不显示真实写入口。Frame 创建/删除/标题/几何/演示标记、元素加入/移出 Frame、纯视觉连线创建/删除及展示样式已有条件式 Document CAS UI；Frame geometry/connector styling、无实体引用 Text/Sticky Note 内容 CAS 与事务性 Element 删除已有条件式 UI/API 切片；Element 尺寸/旋转已有条件式 Element CAS UI/API 切片；durable consumer/realtime 和 Canvas/Jira relation command 仍未完成。
Phase 3E 增加 `GET .../canvases/{canvas_id}/events` 有界轮询查询：每次请求重新检查当前 actor scope、Project membership、Worktree binding 与 Canvas 存在性，在当前事务/RLS scope 下按 `(occurred_at,event_id)` 递增读取；游标时间和事件 ID 必须同时提供或同时省略，`limit` 默认 100 且范围为 1–200。响应不包含 Outbox payload（其中可能有旧 WorkItem ID）或 EntityRef 目标，只返回 event ID、类型、schema / aggregate version、actor、correlation 和时间等刷新提示。查询索引通过 additive migration `2026-09-29-worktree-group-phase-3e-canvas-outbox-index.sql` 按 `(tenant_id,worktree_id,canvas_id,occurred_at,event_id)` 支持范围过滤与 keyset 游标。客户端应基于提示重新读取当前授权的 Canvas Document / Elements；浏览器已实现按 Worktree/Canvas scope 恢复的本地持久游标 at-least-once poller；每次请求仍走授权 API，且仅在当前授权 projection refresh 成功后持久化并前移游标，失败时重放当前页。本地游标是可丢弃的刷新提示，不是授权凭据、服务端 durable consumer offset、NATS consumer 或实时推送。Group Canvas 页面在 provider-backed live projection 下，当 Worktree 尚无 Canvas 时先调用认证、幂等的 `POST .../canvases` 初始化 Canvas；只有命令成功并刷新授权投影后才开放 `POST .../canvases/{canvas_id}/work-items` 创建 canonical Task Card。服务端使用同事务写入 WorkItem、Multica lifecycle、Worktree association、Canvas Element / EntityRef、Audit、Outbox 和幂等响应；客户端仅在请求成功后重载 WorkItem/Canvas/Elements projection，并提供任务卡链接。浏览器在同一页面生命周期内保留待确认创建命令的完整请求体、correlation ID 与 idempotency key；Canvas 初始化键在授权投影确认 Canvas 后清除，任务键在成功响应后清除；改变任务内容会开启独立命令。任一命令失败均不回退 seed。没有宿主 provider 时不显示真实写入口。Canvas viewport 的平移、缩放、滚轮与 fit-to-content 由 `CanvasView` 暂存，显式保存调用 `PUT .../canvases/{canvas_id}/document`，携带当前 Canvas version、完整 frames/connectors 快照、`expected_version`、`correlation_id` 和 `Idempotency-Key`；保存期间屏蔽交互，成功后重载授权投影，版本冲突显示服务端错误并保留待提交视口。当前 UI 通过完整 Document draft/CAS 创建或删除 Frame、将选中元素归入 Frame，并创建或删除纯视觉连线；Frame geometry/connector styling 已通过 Document CAS 接线；无实体引用 Text/Sticky Note 内容通过 Element version CAS 更新；Element 删除同事务推进 Document version 并清理 Frame/connector 引用；未锁定 Element 尺寸/旋转已接入 versioned Element CAS UI/API；未锁定 Element 的坐标可由认证态拖动命令保存，不会经 Zustand seed 写入 live Group。浏览器 adapter 对该 CAS endpoint 的 contract test 已覆盖 method、route、Bearer、幂等键和 request body。该切片不代表宿主 provider、目标数据库或实时 consumer 已部署。

其余读写均先验证 JWT actor、Worktree 所属 Project 与当前 membership，再设置事务级 `app.tenant_id` / `app.worktree_id` 供 RLS 使用；一般 Canvas 写命令还需 `canvas:write` 和 Project writer role。Document 写入带 `expected_version`、`Idempotency-Key`，只接受当前 Canvas 元素 ID，frames/connectors 有数量与尺寸上限。当前 Group UI 将 viewport、frames、connectors 作为完整 Canvas Document draft 暂存并以固定 expected_version 显式 CAS 保存；Element 位置更新单独调用 Worktree-scoped `PUT .../elements/{element_id}`，携带 Element expected_version、Idempotency-Key 与 correlation_id，完整保留其它字段，只改 x/y；成功或冲突均刷新授权投影。Frame 和视觉连线的结构性修改经完整 Document draft/CAS；Frame 与 connector 展示属性通过完整 Document CAS 编辑；无实体引用 Text/Sticky Note 内容通过 Element version CAS 更新；Element 删除通过受认证 API 同事务清理 Document 引用。Visual connector 是纯布局边，不代表 Jira 或 WorkItem relation。Phase 3B 当前只接受 canonical `work_item` 与当前 `worktree` 两类 EntityRef；Jira issue 引用要等 Jira alias/SoR contract 落定后扩展。任务 EntityRef 只解析到同 tenant/project、当前 Worktree association 下同时存在 canonical metadata 与 Multica lifecycle 的 WorkItem；读取 WorkItem 引用还需 `work-item:read`，否则元素响应隐藏此类引用。Canvas `content` 不得另存身份 ID；任务卡元素和 Worktree 节点必须分别绑定同类 typed EntityRef。

元素、EntityRef 与 Canvas Document 的 SCD2 变更、append-only Audit、Outbox 和幂等响应在同一 PostgreSQL transaction 提交。事件携带 actor、correlation ID、Worktree / Project scope 与 aggregate version；元素与其 EntityRef 共用单调递增的 Element version，EntityRef 自身版本另放入事件 payload，Canvas registry / Document 事件使用 Canvas version。Document version conflict 返回 409，不覆盖新版本。Canvas API 不更新 Multica lifecycle；任务状态仍走 WorkItem Lifecycle Command。Canvas→WorkItem 原子创建 API、Outbox metadata polling API、认证态 Canvas 初始化入口、任务创建入口和 viewport CAS 保存 UI 已落代码；创建成功刷新授权投影并提供 canonical Task Card 深链。浏览器 poller 不是 durable consumer/realtime；宿主 provider 未安装，Frame 创建/删除、元素归入 Frame 与纯视觉连线增删已有条件式 UI；Frame 几何/样式、Text/Sticky Note 内容与安全 Element 删除已有条件式 UI/API 切片；Element 尺寸/旋转已有条件式 Element CAS UI/API 切片；Canvas/Jira relation command 仍未实现；位置拖动只是条件式前端代码切片；Element 位置更新由 live UI 调用现有 CAS API；迁移尚未在目标数据库应用、宿主 JWT provider 未装配，因此这些仍是未部署代码切片。

Phase 3A 前端切片只加载 `Canvas.ref_kind=worktree && Canvas.ref_id=current_worktree_id`，任务深链要求显式 typed EntityRef 与当前 Worktree task association；旧 Project / Free seed 不自动迁移。Phase 3C 后端 Document API 与 Phase 3E JWT adapter / 条件式 projection、空 Canvas 初始化、Canvas→Task Card 创建和 viewport CAS 保存 UI 已落代码；宿主登录 provider 尚未装配到 Group 路由，因此当前运行仍读 mock preview，provider 存在时 API 失败则 fail-closed。未绑定当前 Worktree 的任务不显示 CLI 入口。Frame 和视觉连线的结构性 Document 编辑已有条件式 UI；Frame/connector 展示属性、Text/Sticky Note 内容和引用安全删除已有条件式代码切片；Element 尺寸/旋转已有条件式 Element CAS UI/API；其他类型内容和实时协作尚未实现；Element 位置 CAS 仅有条件式 UI 代码切片，数据库未在目标环境部署；当前结果不能验收 Phase 3。

Group Canvas 的结构编辑使用本地 Document draft，不在拖动或点选时写服务端：Shift+click 可选择多个当前 Canvas 元素；用户可新建/删除 Frame、把所选元素加入 Frame、创建/删除 `kind=free` 的视觉连线。保存时一次提交完整 viewport/frames/connectors，并沿用 draft 创建时的 Canvas `expected_version`、`correlation_id` 与幂等键；同一 draft 重试复用命令身份。版本冲突不会自动将旧草稿套到新版本，用户需显式放弃草稿并重载。Frame 最多 1,000 个、视觉连线最多 5,000 条，元素引用由服务端限制在当前 Canvas。该预览/条件式 UI 仍依赖宿主 JWT provider 和已部署数据库才可运行。

当前 live Group 页面已把未锁定 Element 的宽、高、旋转编辑接到上述 Element PUT：使用 `update_mode=geometry`，只提交几何 draft，不重写 x/y、content、locked/hidden 或 EntityRef；服务端从 `FOR UPDATE` 当前行合并未提交字段。位置与便笺分别使用 `update_mode=position` / `content`，每种模式拒绝额外字段。请求固定当前 Element version、复用 correlation / idempotency identity，并在成功或冲突后刷新授权投影。数值需满足 `0 < width,height <= 10,000`、`abs(rotation) <= 36,000`。该代码切片仍依赖宿主 JWT provider 与已部署迁移，未锁定 Element 几何切片不表示真实会话或数据库环境已验收。

### 8.2 Group Chat 与 LangGraph

Chat Shell 由 Engineering Run 提供，底栏固定 `WORKTREE | GLOBAL` scope。`WORKTREE` 精确表示当前 Run 下一个显式选定且已授权的 focus Worktree；没有有效 focus 时禁止发送并要求先选 Worktree。`GLOBAL` 从服务端解析 actor 可访问的 Run/Worktree 集合并要求显式选择 target。Transcript、chat session、dispatch intent 与 checkpoint 归 Chat/Workflow owner；Worktree 是 WORKTREE scope 的具体目标而不是 transcript owner。现有 Worktree chat routes 是兼容 adapter，须返回并校验 `engineering_run_id`。

以下 Worktree chat endpoints 与 Worktree target 列表为现存兼容 API；canonical API 以 Engineering Run 为入口，并在 WORKTREE scope 要求一个属于该 Run 的 focus Worktree。

Chat scope 被显式写入请求与 checkpoint：

```text
POST /api/v1/worktrees/{worktree_id}/chat/messages
GroupChatMessageRequest {
  scope: WORKTREE | GLOBAL,
  target_worktree_ids?: UUID[],
  session_id?: UUID,
  message: string,
  entity_refs: EntityRef[],
  correlation_id?: UUID
}
Idempotency-Key: UUID
```

`WORKTREE` 必须等于当前已授权 `worktree_id`。`GLOBAL` 由用户明确选择 1–20 个目标，不得默认扩展为整个 Tenant；执行前对当前 Group Worktree 与所有目标作全有或全无的授权预检，任一目标失败都不派发 L0。通过预检并派发后，L0 才对每个目标分别编排并呈现子任务结果，允许执行阶段部分失败。

Group Chat 提交 endpoint 为 `POST /api/v1/worktrees/{worktree_id}/chat/messages`。Bearer actor 必须具有 `chat:submit`，请求携带 scope、message、可选 `session_id`、typed EntityRef、correlation ID 和 UUID `Idempotency-Key`，不接受调用方提供的 actor/tenant/role。WORKTREE 的目标只能是路径 `worktree_id`；GLOBAL 必须包含 1–20 个不同目标。API 总是重新授权路径 Worktree，并对每个 GLOBAL 目标执行当前 GroupContext / Project membership 解析；EntityRef 的 Worktree 必须属于目标集合，实体类型与实体本身还须由 workflow 的领域 resolver 复验。任一目标校验失败时，不调用 workflow。请求 fingerprint 覆盖 actor、tenant、path Worktree、scope、目标、session、消息、EntityRef 与 correlation；workflow 必须以 actor+幂等键持久校验，防止同 key 异 payload 重放。

若请求省略 `correlation_id`，handler 以 UUID `Idempotency-Key` 作为稳定 correlation，而非为每次尝试生成随机值；因此同一 actor / key / payload 的重放拥有相同 fingerprint。调用方显式提供 correlation 时，该值进入 fingerprint，不能在同一幂等键下改写。

Global 目标发现使用 `GET /api/v1/worktrees/{worktree_id}/chat/targets?limit=50&cursor={worktree_id}`，与消息提交共用 Bearer actor 和 `chat:submit`；在同一 tenant-scoped transaction 中锁定路径 Worktree 与当前 Project membership，再查询 actor `permission.project_role_binding.valid_to IS NULL` 且 Worktree 当前 `worktree_project_binding.valid_to IS NULL` 的候选，同时排除 `archived=true`。结果按 UUID `worktree_id` 升序 keyset 分页，`limit` clamp 到 1–100，仅返回 `worktree_id / project_id / name / next_cursor`，不返回 path、Runtime、Agent 或 owner 元数据。分页每次重新执行 Tenant RLS 与 membership 查询；游标只是位置，不携带 actor grant。目标目录与提交之间可能发生撤权，因此 POST 仍需逐目标重新解析 GroupContext；发生撤权则整批拒绝。Group 底栏只显示目录返回的授权目标复选框，且将 GLOBAL 选择上限限制为 20；没有 live API provider 时不显示 seed 候选。

`ScopedChatWorkflow` 是从 Group REST 控制面到 L0/TMO 的受信任注入边界。接收请求后必须先原子持久化 Transcript message 与 run intent，再返回 202 receipt 并异步派发；不能把 API 预检时得到的 `AuthorizedChatTarget` 或 `permission_snapshot_ref` 当长期 grant。Resume、interrupt approval 与每次 tool/capability 调用都重新调用当前权限 resolver；每个目标工具动作独立授权与审计。EntityRef 必须经对应领域 resolver 解析，不得只凭客户端声明的 `ref_type/ref_id/worktree_id` 拼接模型上下文。未配置 workflow 返回 503；失败、未授权或幂等冲突不得生成 L0 run。

当前 `scoped_chat.rs` 实现 Bearer scope、GLOBAL Worktree 目标目录、WORKTREE/GLOBAL target guard、逐目标 GroupContext preflight、EntityRef Worktree containment、request fingerprint 和 202/503 边界。`scoped_chat_store.rs` 的 `PgScopedChatWorkflow` 已实现持久化接收端：同 actor/tenant/key 的重复请求锁定幂等行并比较 SHA-256 fingerprint；相同 fingerprint 返回已保存 receipt，不同 fingerprint 冲突；新请求在同一事务中写 Session/server-generated LangGraph thread、user Transcript、queued Run、幂等 response、dispatch outbox 与 Audit，commit 后返回 `queued`。LangGraph thread ID 不与请求 actor 提供的业务 ID 共用，授权快照不写入 checkpoint/outbox。

数据库迁移 `2026-09-29-worktree-group-phase-5-chat-store.sql` 覆盖七张表：Session、Message metadata、Dispatch Event、Audit Event 为 T/append-only；加密 Transcript payload、Run、Idempotency 为 W；无 Master 表。七张表均 FORCE RLS；Session/Message metadata/Payload/Run/Idempotency/Audit 使用 tenant+actor scope，Dispatch Event 使用 tenant scope。T metadata 表拒绝 UPDATE/DELETE；W payload/Run/Idempotency 均有 `retention_period` 与 `expires_at`，但当前尚无 payload cleanup/KMS key destruction scheduler。`PgScopedChatWorkflow` 强制依赖 `TranscriptBodyProtector`，加密封套应绑定 tenant/actor/session/message/targets 作为 AEAD associated data，并从所有目标策略中应用最短有效保留期；repository 未提供具体 protector。当前 `main.rs` 没有安装该 adapter，outbox consumer/L0 dispatcher 也不存在，所以默认 API 仍返回 503、composer 仍禁用；未来只有受控 worker 消费 outbox 并逐次复验权限后才可执行。

此 migration 已在隔离 PostgreSQL 验证库应用并重复执行；7 张表均启用并 FORCE RLS。事务内使用临时 schema/table grants 和 `SET ROLE star_app` 验证 tenant/actor 可见范围、错误 actor 隔离、append-only UPDATE/DELETE 拒绝及无写 policy 时写入拒绝，事务回滚清理 fixture/grants。目标数据库、正式 runtime role 与持久 SQL grants 尚未部署；因此这只是 schema/policy/trigger 的隔离验证，不代表生产 Transcript 已接通。LangGraph SDK/checkpointer compatibility、resume/interrupt、目标级 stream 与跨角色 ACL/工具审计仍未完成，Phase 5 不得关闭。

Transcript 正文另受 Agent Policy 的敏感 AI Prompt/Response 治理约束：默认保留上限 90 天，Project Policy 可配置；到期物理删除密文或销毁专属数据密钥，append-only 元数据仍不得复制正文。当前 migration 已不再含明文字段，且 adapter 在构造时必须注入 protector；但当前仓库没有具体 KMS/envelope provider，W payload cleanup/key destruction worker、密钥轮换与授权导出/删除机制仍不存在。故此 migration 仍不能进入生产；Phase 5 production gate 还须实现并验收这些机制、日志脱敏和真实 PostgreSQL RLS 行为。

LangGraph Python checkpointer 使用 `thread_id` 关联 checkpoint；此 ID 必须由服务端生成并映射到 Chat Session，不得接受浏览器自报值，也不与 WorkItem、Task Card 或 Agent Session ID 共用。checkpoint 只保存非敏感 workflow state 和 thread/run 关联，不保存 JWT、`GroupContext` / permission snapshot、长期 Secret 或 Plugin capability grant。官方 resume 用 `Command(resume=...)` 提交 interrupt 决定；start、resume、interrupt approval、checkpoint replay 与每次 capability/tool call 都重新解析当前 actor、GroupContext、目标与 grant。LangGraph 从 checkpoint 边界恢复时会重新执行后续节点，所以所有外部副作用必须经带稳定幂等键、授权版本与 Audit 的 Domain Command/outbox，不可直接在可重放节点里做非幂等写入。官方 Python PostgresSaver 的 `.setup()` 会创建 checkpoint 表，应由受控 schema migration/setup 作业执行；API runtime 不能在收到首个用户请求时自动创建表或扩大数据库权限。该 API 语义已从官方文档核对，但当前仓库尚未固定部署包版本、服务身份、独立 checkpoint schema/retention、LangGraph runtime 或 Transcript 接线。

### 8.3 Plugin 热插拔

Plugin App tabs 与内置 Run Apps 同级，binding 与 registry state 的目标 scope 是 Engineering Run；Project/tenant 级 manifest 与 grants 维持各自 owner。现有 Phase 6 的 Worktree-scoped registry migration/API/UI 是已知兼容实现，不是 Run registry 完成证据；迁移要增加 Run binding/current projection，并在每次 capability call 按 actor + Project/Branch/Run + grant 复验。Run Worktree focus 不能隐式授予插件文件/CLI 能力。

原生 App 与 Plugin App 使用同级 registry 节点和统一 shell。Plugin manifest 声明 publisher、版本、host API 兼容区间、capabilities、事件订阅与 UI entry；Registry 的 enablement 不是 capability grant。每次 capability call 由 Gateway 按当前 actor、Project、Worktree 与 plugin grant 授权。

导航投影 endpoint 为 `GET /api/v1/worktrees/{worktree_id}/group-apps`，使用当前 Bearer actor 与 `worktree:read`。Handler 先解析当前 Worktree/Project membership，再调用生产 `main.rs` 已安装的 `PgGroupAppRegistryProvider`（Host API version 1）。provider 在 tenant/actor scoped transaction 内重验 membership version、当前 Worktree/Project binding 和未归档状态，并只联查 current binding、current verified manifest、Host API 兼容区间与当前 actor `group_app:open` grant；其余 Tool/Data capabilities 仍由未来 Gateway 对每次调用单独复验。响应只含 `plugin_id / manifest_version / label / sort_order`，最多 100 条、plugin ID 唯一并稳定排序，设置 `Cache-Control: no-store`；不得回传 capability、Secret、原始 manifest 或任意外部 URL。provider 查询失败或 schema 尚未部署时返回非成功响应。

Phase 6 migration 建立 `plugin.group_app_manifest`、`group_app_registry_state`、`group_app_binding`、`group_app_access_grant`、`group_app_audit_event`。前四张为 Master/SCD2，使用有效区间和 current-row 唯一约束；DB trigger 仅允许关闭 current row，禁止更新历史字段与物理删除。Audit 为 Transaction，禁止 UPDATE/DELETE。所有表开启并 FORCE RLS；迁移不包含 app write policy。Manifest 的 `verified` 状态要求后续受信任 ingest 写入，单独的 status 值不能建立 publisher trust root。迁移已在隔离 PostgreSQL 验证库应用并重复执行，验证五张表 FORCE RLS、actor/tenant 隔离和 metadata/Audit 的 UPDATE/DELETE 拒绝；临时 grants 与测试数据均在事务结束时回滚。目标数据库和正式 runtime role/schema/table grants 尚未配置。签名验证/ingest、lifecycle commands、registry revision 原子推进、capability gateway、插件隔离执行、drain/cancel 和热撤权事件尚未实现。

生产 Group 导航以服务端当前 Worktree App Registry/授权投影为准，只投影 `active` 且授权有效的 Plugin App；收到 disable、uninstall 或 grant revoke 后，必须使导航 projection 和新调用入口失效。客户端不允许根据本地 checkbox 生成生产 App 入口。当前 Group 页面已接入只读 endpoint consumer：验证 Worktree ID、UUID correlation ID、非负安全整数 Registry version、最多 100 条、受限且唯一 plugin ID、无控制字符且限长的 manifest version/label、int32 sort order 并稳定排序；支持手动刷新、30 秒轮询和 tab 恢复可见刷新。加载、错误、Worktree 切换及 provider/session generation 变化时清除旧导航，错误时显式重试且不回退预览；仅未安装 API provider 的演示环境显示内存预览。live 入口只表示服务端导航授权 projection 已返回，页面仍不运行插件代码、不调用 capability；migration 未部署和受信任 manifest ingest 未实现时生产路由保持 fail closed。

disable/uninstall 后新调用立即拒绝，活动调用按独立 ADR 定义 cancel 或 drain；业务 Transaction、Audit 和 Canvas EntityRef 保留。发布者签名信任根、沙箱承载和 upgrade rollback 是 release blocker，不能由 UI 原型开关代替。

### 8.4 Pi-inspired Agent core 与 Rust 桌面 UI

Pi 的设计参考限于精简 orchestration core、组合工具/skills/extensions、显式生命周期事件、可分支 session history 和 context compaction。渡口提供这些概念的 Rust-native contract；不嵌入 Pi/Node、不复制其执行器，也不把“Pi 本身不内建 sub-agent”解释为产品不能并行。每个 TaskExecutionRun 是独立执行身份；同一交互 session 的上下文分支使用 parent/branch reference，RunEvent/Audit 仍 append-only。模型上下文只读取活动 branch 与按需 artifact，摘要不能改写验收条件或源历史。

Rust desktop 主路径覆盖 Worktree Index、Run timeline、Canvas 与 CLI；这和现有浏览器 Group UI 是两个发布目标。UI 通过 cursor page + versioned delta 获取授权 projection，使用共享不可变快照/有界队列，不复制全量事件或大日志；列表虚拟化、Canvas viewport/spatial culling、终端有界 ring buffer 与 disk-backed artifact 限制常驻 RAM。不可见 pane 停止订阅/轮询；阻塞 I/O 与图计算走可取消、有上限 worker pool。Desktop UI framework/rendering choice 必须在相同大列表/Canvas/CLI workload 下比较进程树 peak RSS、CPU、响应 p95 与帧时间后冻结，未测数字保持 `TBD-MEASURE`。

Plugin 使用版本化 capability manifest 与每实例资源预算；运行时选隔离进程或受限 WASM，具体策略通过威胁模型和性能 benchmark 冻结。热更顺序为停止新调用、撤销 grant、drain/cancel 在途 Run、释放资源、切换 Registry projection；禁止在主进程载入任意 native code。

### 8.5 Agent Execution Profile 与可替换 Provider

版本化 `AgentExecutionProfile` 组合 Agent、Memory、Skill、Context、Validation、Loop、HookSet、ProjectEngineeringManifest 与资源预算。每个 Provider contract 固定稳定 ID、API/implementation version、capability/scope、兼容性、资源要求和脱敏错误/evidence 映射；授权、撤销与可用性在每次执行前复验。Run 固定 profile/provider/version/hash/grant/config digest snapshot；历史 Run 不跟随 provider 升级。Context assembler 显式记录来源与 token/byte 预算，压缩只作用于活动上下文并保留可恢复引用；Validation 由独立于 Agent 声明的验证器产生规则、工具链和输入 digest、criterion coverage 与 Evidence。Project 可按 repo commit 绑定任务模板、批准的验证入口 ID、toolchain、fixture 和 artifact/redaction mapping；manifest 文本本身不能授予命令 capability。

第一阶段可由 Rust-owned `AgentCliAdapter` 调用已有 CLI：使用直接 executable/argv、allowlisted env、canonical Worktree cwd、显式 capability、bounded stdin/stdout/stderr/event channel、deadline/cancel 和子进程树回收。CLI 是 Agent provider，不是权限、scope、Context、Task Contract、Hook、验证或验收的事实源。CLI transcript/terminal 输出只按受控 Evidence 与 TTL 保存，不无限驻留在桌面内存。

### 8.6 高级设置 Hooks tab 与 Worktree enforcement

Hook 编辑入口固定为 **高级设置 → Hooks**，沿用 ULYS-235 已拍板的设置导航，与 Skills/MCP/Plugins 等 tab 并列；不新增 Worktree 树节点或 Group App。可视化 Builder 管理 Project baseline 与 Worktree 仅可收紧的 overlay，提供 typed condition/action、范围继承视图、核心规则不可覆盖说明、冲突诊断、diff、dry-run、影响预览、审批发布与回滚；不接受任意 Python/JavaScript/shell/动态库或无界 DSL。Worktree Index 展示 effective HookSet/version/health/block summary；Run Detail/Project BI 可筛选相关规则并深链回该 tab。

Rust builtin Hook Engine 在 Run admission、tool、validation/review 和 Worktree archive/cleanup 等同步安全点执行，返回范围限定的 allow/deny/require_human/defer；不能授予 capability、改写 argv、改变 Task Contract 或伪造验证。builtin safety rules 不可关闭，critical policy/evaluator/audit 失败或超时 fail closed；Plugin hooks 只可在隔离且获 grant 的执行器中提供 advisory/post-commit signal。Worktree 清理必须在 destructive Domain Command 前检查当前 ACL、Worktree lifecycle version、活动 Run/Agent lease/path claim、进程/文件句柄 drain 与 fresh Git retention-lock observation；状态缺失/过期/冲突即阻断，最终仍由 Domain Command 原子重验，Hook 不单独授权物理清理。

每个 Hook evaluation/override 事件固定 HookSet/rule/evaluator version+digest、scope/actor、phase、decision/reason class、耗时、timeout/fail-closed、override 与 Worktree/Run/Task/correlation IDs；不记 Secret、prompt 或完整输出。append-only event/audit/outbox 与 BI coverage、deny、require-human、timeout、override、Worktree cleanup/claim/drain 故障、Validation 与接受/返工结果按版本化 cohort 联动；缺失覆盖保留 unknown。编辑入口与策略执行仍是设计阶段，本次不宣称生产 Hook engine 或 tab 已实现。

### 8.7 Automation 与 Workflow 的 Schedule adapter

`DD-CANVAS-WORKFLOW-001` 中的 Schedule Trigger 只能消费 `domain-automation` 创建的 occurrence ID/version/fencing token，并经授权创建/恢复目标 Run；不得运行第二套 CronScheduler 或维护并行规则表。`ExecutionScheduler` 仅负责 Workflow 内节点依赖 DAG ready-state。Occurrence 到 Run 的 dispatch 必须幂等，resume 时复验 Project/Worktree/Task/Hook/profile 授权，撤销旧 fencing token 后拒绝迟到 worker。当前相关实现仍未完成。

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

### 9.1 Owner API 与服务提取边界

Run tabs 是可单独路由、延迟加载、授权、版本化与卸载的 frontend Apps；后端按事实 owner 划分 bounded context，而不按 tab 创建服务。第一阶段以 Rust modular monolith 部署，各 owner module/crate 通过版本化 command/query API 与 DTO 暴露能力，跨域 projection 只读授权 API/read model。建议 owner 边界：Work Item/Planning，Engineering Run/Agent Runtime/Scheduler，Canvas，Plugin Registry/Capability，Hook Policy/Execution，BI/Benchmark，Project/Branch/SCM。每个 owner 声明自己的 PostgreSQL schema/table 写 grants、RLS、API compatibility policy、Audit/Outbox contract 与 resource budget。

PostgreSQL stored procedure/function 只封装同一 owner transaction 中需要 CAS、行锁、RLS 或原子多行更新的不变量，并仅由 owner API 调用。跨域同步命令必须调用目标 owner API；异步事实由同 owner transaction 写 Transactional Outbox，消费者先写 local Inbox/processed-event key 去重再重建 projection。Saga/Run coordinator 可跨域编排，但每一步带稳定 command ID、idempotency key、correlation/causation 与 append-only Audit；补偿不宣称 distributed ACID。浏览器/App 和其它 owner 禁止直接写非 owner 表、调用 stored procedure 或连接数据库。

只有独立伸缩、故障隔离、发布节奏或合规边界的实测收益足以抵消网络、常驻内存和运维成本时，才把 bounded context 原子提取成服务。提取门包括 service identity/authorization propagation、API/event schema version、数据写 ownership、Outbox/Inbox replay、timeouts/backpressure/circuit behavior、observability、migration/rollback 和 memory/resource budget。App 拆分与 service promotion 是两项独立决策。

### 9.2 Event broker 决策

当前仓库架构基线为 PostgreSQL SoR + Transactional Outbox + NATS JetStream；Kafka/Fluvio 均不是当前 runtime dependency/deployment。领域事件默认留在 NATS，BI/Benchmark 使用 owner 事件生成可重建 read model。除非留存、历史 replay、connector 或 analytics 隔离 SLO 的实测结果要求更换，否则不增加第二个 broker。

若未来必须在 Kafka 与 Fluvio 中为 production analytics stream 选型，先 PoC Kafka：官方 Kafka 4.3 提供 Connect/Streams 等 API，适合 Run BI 外部 connector 与历史分析；需支付额外 broker 运维与内存成本，并使用修复 Kafka Streams 4.3.0 native-memory leak 的 4.3.1+。Fluvio 保留为 Rust/Kubernetes/低资源候选，但低内存属于供应方主张且 Star 没有同负载数据；release/support cadence、connector/托管、故障恢复均需验证。PoC 固定事件体、保留期、吞吐、消费组数、故障注入与部署资源，比较峰值 RSS/CPU/磁盘、consumer lag、恢复时间和 connector 覆盖率。更换 broker 仅限 Outbox 之后的 consumer 链，绝不改变 SoR 或跨域事实 owner。

## §10 持久化分类约束（W/T/M）

本节以下列 W/T/M 表与 Phase 2–6 migration 描述当前兼容实现；它们保留 Worktree-scoped legacy schema/API 边界，不能被读成目标 Run-owned model 已上线。目标 Master row、RLS policy 和 API writer 需由明确的 additive migration 建立 `engineering_run_id` owner 与 `worktree_id` optional binding；历史数据只通过可审计 reconciliation 导入，冲突/缺少 Run binding 保持 unknown 并关闭写入。迁移时新旧 owner 不能双写成两个真相；兼容 API 最终只调用 canonical owner。

Phase 2B-2D 与 Phase 3B migrations 按事实表分类；旧 `worktree_canvas_worktree` 仍是 Work 当前投影，新增的 project/owner 字段仅作 denormalized read projection，权威 Master 事实分别存储在 `worktree_project_binding` 与 `worktree_owner_assignment`。空投影仍需显式 reconciliation：

| 数据事实 | 主分类 | 约束 |
|---|---|---|
| `permission.project_role_binding`、`multica.worktree_project_binding`、`multica.worktree_owner_assignment`、`multica.task_metadata`、`multica.work_item_worktree` | Master | SCD2 `valid_from/valid_to/version`；禁止物理删除；RLS。当前只有 Project/owner 管理写 API，Task metadata SCD 更新待 Domain adapter |
| `worktree_canvas_worktree` 与 `multica.task_lifecycle_current` | Work | 当前运行 / lifecycle projection；`retention_period` + nullable `expires_at`；终态 completed/failed/cancelled 设期限，重试回 pending 清除期限；不承载 owner 历史或唯一审计事实 |
| `multica.task_command_idempotency`、`multica.worktree_management_plan` | Work | 30 天 retention；管理 plan 确认窗 5 分钟；允许按 TTL 清理 |
| `multica.task_lifecycle_audit`、`multica.worktree_management_audit` | Transaction | Append-only；RLS；拒绝 UPDATE/DELETE；生命周期审计同事务写入 |
| `canvas.group_canvas_registry`、`canvas.canvas_elements_backend`、`canvas.canvas_entity_ref` | Master | Worktree scoped SCD2；RLS 同时限制 tenant/worktree；每个 Canvas element 至多一个 current EntityRef；物理删除禁止 |
| `canvas.canvas_group_audit`、`canvas.canvas_group_outbox` | Transaction | 同一业务事务内 append-only；RLS 同时限制 tenant/worktree；consumer offset / realtime 投影另行实现 |
| `worktree_canvas_worktree.project_id / owner_user_id / work_item_id` | Work 投影（混合字段已知缺口） | Project / owner 的事实分别在独立 Master 表；这些 nullable 列仅作旧 Index 查询投影，不可推断回填；须由受控 reconciliation 维持一致 |
| GroupContext | Derived projection（不持久化授权快照） | 需要缓存时为 Work/短 TTL；撤权后失效，不可恢复授权 |
| Task CLI WebSocket attachment ticket | Work | Local Runtime SQLite WAL，grant/ticket 专用 `synchronous=FULL` connection | 仅存 SHA-256 摘要；绑定 tenant/project/repository/worktree/work_item/actor/runtime/session/policy_version；TTL ≤ 60 秒，单次原子消费；expiry + 5 分钟后懒清理；不持久化 Bearer JWT |
| `plugin.group_app_manifest` | Master | SCD2 verified manifest revision；RLS、current-row unique、禁止物理删除；可信 ingest 尚缺 |
| `plugin.group_app_registry_state` | Master | per-Worktree registry revision SCD2；变更时关闭旧行并追加新版本 |
| `plugin.group_app_binding` | Master | Worktree plugin binding/lifecycle SCD2；capability 不由 binding 隐式授予 |
| `plugin.group_app_access_grant` | Master | actor `group_app:open` SCD2 与过期时间；每次工具/数据调用仍需单独授权 |
| `plugin.group_app_audit_event` | Transaction | append-only Registry lifecycle Audit；不含 Secret 与用户内容 |
| `multica.task_contract` | Master | Task Contract SCD2，RLS/no-delete；变更需 append-only audit |
| `multica.task_contract_change_audit` | Transaction | Contract version create/supersede；append-only + RLS |
| `multica.task_execution_run` | Transaction | immutable attempt + task/acceptance/profile/provider/HookSet/Loop/resource/ProjectEngineeringManifest snapshots；Schedule Run 只引用 Automation Rule/Occurrence ID/version；Worktree/repo refs 不设 FK |
| `multica.task_execution_run_event` / `multica.task_execution_evidence` | Transaction | 独立状态维度、Schedule occurrence/Loop decision/Hook evaluation/override、每 Run 至多一条资源高水位汇总与脱敏证据索引；append-only + RLS |
| `multica.task_execution_run_idempotency` | Work | actor-scoped CLI start replay mapping；30 天 TTL，不拥有 Run 身份 |
| Chat message / LangGraph run checkpoint | Transaction + Work 分表设计 | 用户可见消息保留策略独立于可过期 checkpoint；两者不可混成一个无期限 blob |

Phase 2 migrations 已逐表加入 tenant RLS；Master 表无 DELETE policy，Transaction 表仅有 SELECT/INSERT policy 且拒绝 UPDATE/DELETE，Work 表记录 retention。Phase 2 尚未完成 PostgreSQL 实例验证；Phase 5/6 的隔离库结果仅覆盖其各自新增 schema，不覆盖 Phase 2/3 表或生产角色。完整 RLS policy 分类与 Security release gate 仍未通过。

### 10.1 PostgreSQL 角色授权与 RLS 验收

RLS policy 不会自动授予 `CONNECT`、schema `USAGE` 或表级 `SELECT/INSERT/UPDATE/DELETE`。部署/bootstrap 清单必须给出 migration owner、API runtime principal、Chat/L0 worker 与 Plugin ingest/lifecycle principal 的身份和逐表权限矩阵；未实现的 worker 不提前获得写权限。运行时身份不得是 superuser、`BYPASSRLS`、migration owner 或表 owner。迁移不硬编码或自行推断环境角色名称；部署必须显式建立/映射角色、授予 schema 和表权限，并验证 default privileges 不扩大既有权限。

运行验收使用应用实际连接身份（包含连接池 `SET ROLE` 后的有效角色），验证 schema access、各 endpoint 所需 SQL 操作、tenant/actor/worktree RLS 正反向隔离、无 policy 写入被拒、append-only trigger、并发/幂等行为。Phase 5/6 当前只在隔离库做过临时 grant + `SET ROLE star_app` 的策略测试；临时权限和 fixture 已回滚。该库中的 `star_app` 不是经确认的生产服务身份，亦未证明 `star_app_role` 存在；生产角色及 grants 仍是 release blocker。

## §11 关键验收映射

| Requirement / AC | 详细设计落点 | 服务端验收重点 |
|---|---|---|
| WTG-001 / AC-WTG-001 | §1-§5 | Project membership 限定 Index；同级 App 树与唯一 GroupContext |
| WTG-004 / AC-WTG-003 | §3、§6 | owner、Agent、Runtime、PR、冲突/锁和最近活动来自有版本的服务端投影；操作有授权与 Audit |
| WTG-010 / AC-WTI-001 | §3.1、§4、§5、§12 | Git retention lock 与持久化 `locked` 分离，且不代表 Agent 活跃/互斥；unknown/unlocked 不表示空闲；cleanup 需独立活跃状态、drain 和最终重观测 |
| WTG-011 / AC-WTG-008 | §4、§6.1、§10、§11 A | Project Repository discovery + Index create/import UI；严格字段 allowlist、当前授权与 binding 复核，拒绝客户端路径/URL，持久化 operation/Audit/Outbox，provider 缺失返回 503 |
| WTG-012 / AC-WTG-009 | §1.2、§2、§4.2、§11 | Project selector 来自当前用户 membership 目录；ID/role 最小投影、cursor/缓存边界、seed 不回退和无名称 SoR 时使用 UUID 标签 |
| WTG-013/014/015 + TCI-013/014/015/016 / AC-RUN-001/002/003/004 | §7.1、§10、§11 | versioned Task Contract；Task→Run→optional Worktree；immutable Run/Event/Evidence；幂等 CLI start；Worktree-scoped authorized list/detail、稳定游标、响应上限与 Task Card 历史 |
| WTG-016/017 / AC-WTG-010/011 | §1.1、§7.1、§12 | Project Index 的 Quality & Improvement 入口；metric provenance/drilldown、固定 benchmark/holdout 与可回滚 proposal |
| PAR-001..004 / AC-PAR-001..003 | §6.3、§9、§10 | DAG readiness、hierarchical quota/fairness、bounded queue/backpressure、独立 lease/claim/Git lock、可级联 cancel/drain |
| LOOP-001..005 / AC-LOOP-001..006 | §6.4、§8.7、§12 | 唯一 Automation occurrence source、fencing/idempotent dispatch、单 Run Engineering Loop、budget/stop/drain |
| AEC-001..008 / AC-AEC-001..007 | §8.5、§7 | 版本化 Profile/Provider、Rust CLI adapter、独立 Validation、Project Engineering Manifest、BI/Benchmark 可复现 |
| HOOK-001..007 / AC-HOOK-001..006 | §8.6、§9、§10 | Rust-native fail-closed guard、Advanced Settings Hooks tab、Worktree lifecycle gate、append-only Run/BI evidence |
| PERF-001..004 / AC-PERF-001..003 | §8.4、§11、§12 | Rust desktop 虚拟列表/viewport culling/有界缓存与测量门；隔离 Plugin 热插拔；Pi 只作设计参考 |
| WTG-009 / AC-WTG-007 | §4.2、§6.1.1 | 当前 Project 成员目录、候选人验证、版本化 plan-confirm 和成功后刷新 |
| WTG-005 / AC-WTG-002 | §1-§5、§9 | Project/Worktree 切换清空旧订阅；跨项目实体引用拒绝 |
| WTG-006 / AC-WTG-004 | §1.2 | Index 与 Group 深链独立、刷新稳定，不落入通用任务列表 |
| GRP-AUTH-001 / AC-GRP-AUTH-001 | §1.4 | 每次请求取宿主 JWT；缺 session 不触网；无 token persistence / cookie / URL 泄露；401/403 不退化为 mock 写入 |
| TCI-001 / AC-TCI-001 | §7 | Multica/Jira/Task Card 同一 WorkItem ID 和版本 |
| TCI-005 / AC-TCI-002 | §7 | 版本化生命周期命令、legal transition、review gate、冲突刷新 |
| TCI-006 / AC-TCI-003 | §7 | claimant submit、非 claimant reviewer 决策、版本冲突与驳回理由审计 |
| TCI-007 / AC-TCI-004 | §7 | Ed25519 grant issuer/private-key 与 Runtime/public-key 分离、key id 轮换、验签先于 nonce 消费 |
| TCI-008 / AC-TCI-005 | §7 | authenticated Task Session start 的 Worktree/Task/claimant/version 校验、幂等委托、no-store ticket 和缺少 provisioner 时 fail closed |
| TCI-011 / AC-TCI-008 | §7 | Task Card Session status/cancel/人工 reattach；先检查状态、限制可重新连接状态、校验新票据绑定和有效期；页面刷新后从受授权列表发现会话 |
| TCI-012 / AC-TCI-009 | §7 | 有界、脱敏、no-store 的 Worktree/Task Session listing；恢复时不复用旧 ticket |
| TCI-013 / AC-TCI-008 | §7.1 | CLI start 事务快照 Task/Contract 并创建 TaskExecutionRun；idempotency mapping 不定义 Run identity |
| TCI-002 / AC-TCI-001 | §7 | CLI 只在正确 Worktree、授权 profile 与 Runtime 运行 |
| CAN-001 / AC-CAN-001 | §8.1 | Canvas 双向导航引用 canonical WorkItem，写入经过 Domain Command |
| CAN-009 / AC-CAN-005 | §8.1 | Element 位置更新限于当前 Worktree Canvas，使用版本 CAS、幂等键与 correlation ID；冲突刷新授权投影 |
| CAN-010 / AC-CAN-007 | §8.1 | Frame/connector 展示属性以 Document CAS 保存；便笺内容以 Element CAS 保存；删除 Element 同事务清理 Document 引用 |
| CAN-011 / AC-CAN-008 | §8.1 | 未锁定 Element 尺寸/旋转以 expected-version CAS 独立保存，并保留其它字段 |
| CAN-012 / AC-CAN-009 | §8.1 | 浏览器恢复 Worktree/Canvas 游标；projection 成功后持久化，服务端逐请求重新授权 |
| CHAT-001 / AC-CHAT-001 | §8.2 | WORKTREE 与 GLOBAL scope、目标 ACL、部分失败逐目标可见 |
| CHAT-003 / AC-CHAT-003 | §8.2 | 当前路径和每个目标逐一重新授权、越权整批不派发、稳定幂等指纹与 workflow 缺失时 503 |
| CHAT-004 / AC-CHAT-004 | §8.2 | 当前成员可见的非归档最小目标目录、UUID 游标分页、目录不授予权限且提交再授权 |
| CHAT-005 / AC-CHAT-005 | §8.2 | 原子 Session/Transcript metadata + 加密 payload/Run/idempotency/outbox/Audit persistence、stable replay receipt 与 fingerprint 冲突 |
| CHAT-006 / AC-CHAT-006 | §8.2 | 正文加密、Project Policy TTL、crypto-erasure/清理、authorized export/delete 与不含正文的 append-only 元数据 |
| PLG-004 / AC-PLG-002 | §8.3 | Worktree Group App 的授权导航 projection、provider 实时重验、最小输出与 fail-closed 未配置边界 |
| PLG-005 / AC-PLG-003 | §8.3、§9 | Registry 四类 Master 的 SCD2/no-delete、Audit append-only 与 tenant/worktree FORCE RLS |
| AC-GRP-DB-001 | §10.1 | migration/runtime principal 分离、最小 schema/table grants、非特权实际连接身份下的 RLS/触发器运行验收 |
| PLG-001 / AC-PLG-001 | §8.3 | 热撤权对新调用即时生效，活动调用有确定终止语义 |
| ARCH-OBL-GRP-001 / AC-TRACE-001 | §4-§10 | 一个 correlation ID 串起授权、命令、Outbox、投影与 Audit |

## §12 实施顺序与阻断门

1. **2B 代码切片完成**：RS256 Actor、scope、Project membership 查询、GroupContext、Project/owner Master binding DDL。部署、membership provisioning、历史数据 reconciliation 和跨 Tenant/Project 负向集成仍是启用门。
2. **2C 代码切片完成**：canonical task metadata、Worktree relation、Multica 六态 current projection、幂等 command 和 append-only audit；REST SQL coordinator 尚未抽到 Domain/PostgreSQL adapter，Review Gate、Jira alias/sync 与 UI 读写仍未启用。
3. **2D 条件式代码切片**：cursor Index、owner/archived plan-confirm、owner SCD2、management audit 和 Git lock observer contract/UI。生产 main 尚未装配 Host Runtime observer；创建/导入、Runtime 活跃探测、Session drain、物理清理与恢复仍未实现，Index lock unknown 不得视为 unlocked。
4. 完成 migration apply + 项目成员 provisioning 后，运行跨 Tenant/Project ACL 负向、数据库 RLS、并发 version conflict、同键幂等重放、审计不可变性和 API 集成验收，才能关闭生产写能力 gate。
5. Group route 已支持宿主 provider 注入后的只读 GroupContext / WorkItem / Canvas 和 Plugin Registry projection；插件入口消费者已实现校验、排序、显式刷新、30 秒/可见性刷新与错误 fail-closed。真实宿主会话 provider 尚未挂载，因此当前运行仍显示 `preview / seed / not connected`；只有在目标 DB/RLS 和真实会话部署后完成跨 App 浏览器验收，才可把生产接线计为通过，不把 browser store 变化作为产品事实。
6. Phase 8A 已有 CLI start writer 与隔离 PostgreSQL migration 重放/RLS 检查；Phase 8B 已有 Worktree-scoped Run list/detail API 和 Task Card Run History 面板条件式切片。后续仍需补齐 Task Contract commands、CLI exit/status、Validation/Review/Integration/Cost/Evidence/ResourceSummary producer 与目标 DB/RLS 运行验收；Run 历史继续留在 Task Card，不新增 Worktree 导航层级。
7. Phase 9 必须先实现版本化 Profile/provider resolver、scope recheck 与 hierarchical admission，再实现受控 Rust CLI adapter；之后才可落地 Schedule occurrence dispatch、单 Run Engineering Loop 与 Hook engine。Schedule 复用唯一 Automation Rule/Occurrence，不允许 Workflow/`star-scheduler` 各自建 Cron owner。
8. Hook 配置只在 Advanced Settings → Hooks tab；Worktree Index/Run/BI 是 effective policy 与结果 consumer。Rust builtin guard、Worktree Domain Command 原子二次校验、fail-closed 错误处理和无代码 typed-rule builder 均应实现并验收后才可关闭 HOOK 阶段；现有 Python guard 不是产品 Hook engine。
9. Phase 10-13 分别完成有 coverage 的 Project BI、隔离可复现 Benchmark/Proposal、Rust 桌面内存/渲染实测与端到端 release gate；identity、目标 PostgreSQL/RLS grants、host Runtime 等外部条件未就绪时保留 blocker，不以 mock/preview 代替验收。

## §13 已知缺口与实现前置

| # | 缺口 | 影响 | 关闭条件 |
|---|---|---|---|
| 1 | 新 Group WorkItem coordinator 仍在 REST crate 直接 orchestrate SQL；旧 domain-work-item 是三态内存 service | Domain Command Port / PG adapter 的边界尚未完成；旧 `/work-items` 不可用作生产 | 抽出同事务 WorkItem Command / Multica Lifecycle application port，并用 PG repository 实作；完成 API 兼容迁移 |
| 2 | 当前六态 API 与 review submit/accept/reject REST command 已有条件式代码切片；Review Domain port、Task metadata 更新和 Jira external alias/sync 尚未实现 | 任务流程与 Jira adapter 尚不完整 | 接入 DD-MULTICA §14 review/metadata port、alias registry + sync/outbox，跨 App 实测同一 ID/version |
| 3 | 旧 Worktree projection 保留 `project_id` / `owner_user_id` denormalized columns；权威 Project / Owner SCD2 表已创建 | 历史投影与 Master binding 需保持一致；旧 owner/Project 行未回填 | provisioning/reconciliation 同事务维护 Master 与投影；历史映射有审计证据 |
| 4 | App Registry 只读授权导航 API/provider 与 Group UI consumer 已有条件式切片；GroupContext 尚未包含 `allowed_actions[]`；Canvas API 有多项条件式切片，Chat 仅有逐目标 GroupContext 预检与 fail-closed workflow seam；Registry lifecycle writer、LangGraph 与 Plugin runtime/Gateway 仍未落地 | Worktree group shell 仍无法真实完成跨应用交互；显示入口不代表插件可执行 | 接入可信 manifest ingest、Registry lifecycle command、EntityRef resolver / Chat Workflow 与各 App API，补齐 LangGraph runtime 和 Plugin Gateway，并对每个入口通过权限验收 |
| 5 | 当前 archive guard 仅根据 AgentSession/Runtime reference 是否为空；Git lock 有注入式 observer contract/UI 但 production main 未装配 Host Runtime provider；没有可靠活跃会话状态和 Runtime/Agent drain | 对有过绑定但已结束的 Runtime 也会保守拒绝；Git lock 显示 unknown，不能安全执行物理清理 | 装配带 freshness/deadline 的 Host Runtime observer；接 Agent/Runtime 活跃状态和 drain；实现 cleanup plan/confirm、恢复策略与故障演练；确认动作时对同一 Repository/Worktree 再观测 |
| 6 | Canvas migration/API、Document CAS、Frame/connector/便笺编辑与引用安全删除已有代码切片；真实宿主 provider 未挂载、目标数据库未部署、Canvas/Worktree 历史冲突未分类；Outbox 浏览器已用本地游标恢复，但没有服务端 durable consumer offset | 无法在已部署系统中提供生产 Canvas 联动、可靠事件消费或跨应用实时更新 | 装配宿主 JWT provider；评审并应用 migration；建立 membership / 历史归属；接服务端 durable consumer offset 与 realtime、Canvas/Jira Relation adapter；完成跨 Worktree ACL/RLS 与并发验收 |
| 7 | LangGraph 官方 Python checkpoint/resume/replay API 语义已核对；生产包版本、服务部署/身份、独立 Postgres checkpointer schema/retention 与 Plugin sandbox/revocation ADR 尚未冻结 | 无法完成生产恢复、租户隔离和插件热撤权；checkpoint replay 可能重复执行节点副作用 | 固定并审核 runtime/checkpointer 版本与数据库权限/retention；完成 checkpoint restore、interrupt replay、side-effect 幂等、逐次 reauthorization 与 Plugin revoke 演练 |
| 8 | 2B/2C/2D migrations 未应用；Project SoR / Role Binding provisioning API 尚不存在，Worktree 历史 scope/owner/work-item association 未回填 | 代码可编译，但无可用 membership 和可信历史映射；数据库 RLS / FK 未真实验证 | 在目标数据库评审并应用 migrations；通过受控 provisioning 建立 M bindings，审计 reconciliation，并完成跨 tenant/project/RLS 负向集成 |
| 9 | Worktree/Task-scoped Session listing route 与刷新后发现/恢复 UI 已有 fail-closed 切片，但仓内没有真实 `TaskCliSessionProvisioner` 实现 | 当前应用无宿主 provider，listing 在生产中不可用并返回 503；页面不得通过持久化 ticket 绕过授权 | 接入生产 provisioner；按当前 tenant/actor/project/repository/worktree/task/runtime/session binding 查询最近记录，完成 no-store、授权拒绝、恢复新票据与刷新页面的运行验收 |
| 10 | Project Worktree create/import 已有认证 API contract 和 Index 控件，但没有生产 Host Runtime provider、Project-Repository SoR binding writer 或 durable writer | API 返回 503；无法从 Project 安全解析 Repository checkout，也无法将 Git operation 原子投影至 Worktree/Audit/Outbox | 实现受信 Repository registry/provisioning、Git lifecycle adapter 与持久化 operation writer；验收 race-safe membership/binding 复核、幂等重放、失败恢复和审计 |
| 11 | Project 授权目录可列 ID/role，但没有持久 Project 主数据/name SoR | 生产 Index 可按授权 ID 定位；显示名缺失，且不能用本地 seed 冒充权威 | 接入持久 `ProjectRepository`/Project master 与 membership projection，并用同一 actor/tenant 边界验证目录 ID、名称和 binding |
| 12 | Phase 8A CLI Run writer 与 migration 已有条件式实现；migration 仅在隔离临时 PostgreSQL 验证。Phase 8B Worktree/Task list/detail API 与 Task Card Run History UI 已落代码，但目标 DB/RLS 未部署，Task Contract write API、CLI exit/status、Validation/Review/Integration/Cost/Evidence/ResourceSummary producer 未接入 | 可授权浏览当前已记录 Run；无法从 Task Card 写入/完整呈现生命周期与全部证据，BI coverage 不完整 | 在目标数据库配置并验收 runtime role/grants 与 RLS；实现 Contract version commands、各来源状态/证据 producer 和全状态 reconciliation；扩充 API/E2E 与 Evidence ACL/retention 验收 |
| 13 | Phase 9/10 Project BI、Benchmark 与 Improvement Proposal 只在需求/设计阶段定义 | 无 metric projection、coverage dashboard、隔离 replay 或策略采纳/回滚接口 | 按 DD-MULTICA-TASK-001 §14.8 增加有版本公式的 Project read model、固定 benchmark sets、隔离 runner、授权 proposal lifecycle 和 BI follow-up |
| 14 | Phase 9 Schedule/Engineering Loop 尚无 occurrence worker、fencing、budget/stall stop 或 drain implementation | 时间计划可能重复派发；工程循环可能超预算或把 Agent 声明当验收 | 复用 `domain-automation` 唯一 Schedule Rule/Occurrence source，实现 durable idempotent dispatch、Run 内 Loop event/stop reason 与多层资源 admission/cancel/drain |
| 15 | Rust-native evaluator、9B2B scoped policy REST API、9B2C archive-confirm Hook gate 与 9D Project summary API 已有条件式代码切片；9B2C 在 Host Runtime drain/readiness 与 Git lock 外部观测期间释放数据库行锁，随后短事务重新授权/锁定、加载 verified policy 并复核 lifecycle version、facts freshness 与 admission fence（至少 5 秒提交余量） | production main 未安装 Runtime readiness provider，归档确认当前 fail-closed 返回 503；5 秒只是提交前最小余量，provider 必须维持 fence 到命令完成，事务时限与该保证尚未在目标环境验收；目标 DB/grants/RLS integration、物理 checkout cleanup 与通用 Hook RunEvent/outbox 仍未验收，不能宣称生产级归档门已启用 | 安装 Host Runtime provider 并定义/验收与 admission fence 匹配的有界事务期限，完成目标 DB/RLS/grants 验收；补齐 archive lifecycle/physical cleanup 端到端测试，补齐目标环境生命周期验收，并接入 Run-linked producers、Outbox delivery、完整 BI read model 与 Quality & Improvement drilldown |
| 16 | Agent/Memory/Skill/Context/Validation provider profiles 与 Rust CLI adapter 只有架构契约 | 无法一致冻结 provider 版本、上下文来源、独立验证与资源开销 | Phase 9 以稳定 schema/version/capability/digest resolver 实现；CLI adapter direct argv/allowlist/canonical cwd/bounded IO/deadline/process drain 通过端到端验证 |

## §14 审阅栏与修订履历

| 角色 | 状态 |
|---|---|
| 架构 | Draft；Mavis 接手审核，Phase 2B-2D、Phase 3B/3C/3D Canvas、Phase 3E JWT adapter + 条件式 projection + Canvas 写入切片、Phase 4A signed grant guard、Phase 4B1 fail-closed Session start REST seam 与 Phase 4B ticket / protected transport seam 代码切片完成；待 DB / ACL / Domain Port / 宿主 token provider / 真实 provisioner / PTY sandbox / TaskRun audit 验收 |
| SRE Lead | 待详细验证：故障切换、事件积压与 Runtime 清理恢复 |
| 平台 | JWT / scope / ACL、Task lifecycle、Index、Canvas Document CAS、Ed25519 Task CLI grant helper、Task Session start/lifecycle route 与浏览器手动 status/cancel/reattach adapter 可编译；需验证真实 provisioner、issuer/private key 与 Runtime/public key 部署、grant/spawn API、迁移、RLS、数据回填与 Runtime session adapter |
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
| v0.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Canvas Document 在 Canvas registry 上的 SCD2 版本序列和 CAS 更新契约；新增 document API、Frame/connector 同 Canvas 元素引用校验、统一幂等/Audit/Outbox；明确 connector 只作布局投影及前端认证接线仍阻塞 | Phase 3C Document persistence API 代码切片实现后同步详细设计 |
| v0.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Phase 4A Local Runtime 执行上下文、五分钟 grant window、approved launch profile 固定参数和 runtime mount 与 canonical checkout 一致性校验；标明 helper 尚未消费 nonce 或接入 Session API，Phase 4B 保留执行、审计与 UI 集成 | 继续推进 Worktree Group 所有阶段，先收紧 Task Card CLI 的执行信任边界 |
| v0.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Phase 4A helper 只校验可信服务端上下文的一致性、不验证签发者/签名；profile 限制为非敏感 static_environment；补充独立 secret capability、nonce/ACL/start 必备门和 canonical path TOCTOU 限制 | 继续推进所有 Phase，复核 Task Card CLI helper 的信任边界 |
| v1.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Canvas→WorkItem 原子创建 API 契约与事务边界；同步 3D 状态，并确认 Outbox 消费、Group UI 接线与 DB 部署仍未完成；上游同步 requirements v2.6 / basic design v0.7 | 推进 Phase 3D，让无限画布可直接创建和互动任务卡 |
| v1.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增受保护 Canvas Outbox metadata polling API 契约：每次复核当前授权、复合游标与限页、响应不暴露 payload/实体目标；同步 requirements v2.7 与 basic design v1.1，并明确 Group UI、consumer、realtime 和数据库部署仍是 blocker | 推进 Phase 3E，建立 Canvas Outbox 的授权读取切片 |
| v1.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 Group UI JWT provider 与 API adapter 契约：逐请求获取 Bearer、禁用 cookie/cache、缺 token fail-closed、拒绝非 HTTPS 远程 API；记录 Project Worktree Index / plan-confirm、GroupContext、WorkItem、Canvas API 方法与 9 项前端单测，并明确宿主 provider、页面数据投影、consumer/realtime 尚未接通；上游同步 requirements v2.8 / basic design v1.2 | Phase 3E 实现可注入用户 JWT 的 Group REST API adapter 后同步认证契约 |
| v1.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 `CanvasOutboxPoller` 的实现边界：刷新当前授权 projection 成功后才推进进程内复合游标，失败重放当前页；记录 10 项 adapter 单测通过，但未接页面、没有持久 consumer offset 或 realtime，不改变 Phase 3E 未完成状态 | Phase 3E 新增受保护 Outbox metadata polling helper 并验证失败重放 |
| v1.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新 Group 路由的 provider 注入后只读 API projection 契约；明确 GroupContext/WorkItem 与 Canvas App 的按需读取、失败时禁止回退 seed，Canvas route 可调用 in-memory Outbox poller；注明宿主 JWT provider、服务端 durable offset/realtime、写操作和数据库验收仍未完成；focused Vitest 13/13 与 typecheck 通过 | Phase 3E Group UI 从 mock-only 进入可注入 provider 的 API projection 切片 |
| v1.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 provider 的非敏感 session generation key；身份/会话变化时同步重建 client、清除旧数据 projection，避免认证 callback 引用不变造成跨会话残留；同步 requirements v2.9 / basic design v1.3；projection test 覆盖 session key 轮换 | Phase 3E 自审识别登录会话切换后的客户端 projection 生命周期需显式约束 |
| v1.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.0 / basic design v1.4；记录 authenticated Group Canvas 已接入 Canvas→WorkItem 原子创建命令，创建后重载授权投影并提供 canonical Task Card 链接；Document/Element 布局写入、宿主 provider、数据库部署和 realtime 仍未完成 | 推进 Phase 3E 从只读投影进入一条可审计的跨 App 写路径 |
| v2.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.1 / basic design v1.5；增加 Worktree 空 Canvas 的认证幂等初始化入口，明确 Canvas→Task Card 入口必须在初始化成功、授权投影刷新后才可用，失败不回退 seed | 补齐首次进入 Worktree 时启动 Canvas 协作的 UI/API 顺序 |
| v2.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.2 / basic design v1.6；Group Canvas 以显式 CAS 命令保存 viewport，冲突保留新版本并显示错误；Frame、connector、Element 写接线、宿主 provider、DB 部署与 realtime 仍未完成 | Phase 3E 接通 Canvas Document viewport 的认证持久化交互 |
| v2.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.3 / basic design v1.7；Group projection 返回 Worktree 内多 Canvas 列表，支持 `canvas_id` URL 选择、授权回退、新建后切换与所选 Canvas 事件订阅；focused Group 验证 18/18 | 推进 Phase 3E 多 Canvas 管理入口 |
| v2.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.4 / basic design v1.8；新增 Worktree 内未关联 Task Card 列表及原子 Element + EntityRef 关联命令，成功后刷新授权投影；focused Group/API 验证 19/19 | Phase 3E 接通已有任务卡放入 Canvas 的交互 |
| v2.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.5 / basic design v1.9；接通 live Canvas Element 位置拖动与 Element version CAS 保存，冲突刷新授权投影；其它布局编辑、宿主 provider、目标库和实时消费仍未完成 | Phase 3E 使用既有 versioned Element update API 开放受限位置编辑 |
| v2.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.6 / basic design v2.0；将 WorkItem 生命周期 API 接入 Multica、Jira 和 Task Card UI，细化合法迁移、claim lease、review gate、CAS/幂等命令及冲突刷新边界 | Phase 3F 接通同一 canonical lifecycle service 的在线跨 App 写入 |
| v2.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.7 / basic design v2.1；明确生产插件同级导航必须来自当前 Worktree 服务端授权投影；记录客户端启用插件的动态同级导航仅为本地预览，不执行代码、不授予 capability | Phase 6 UI 切片演示插件 App 同级入口及本地启停状态 |
| v2.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.8 / basic design v2.2；定义 viewport、Frame 创建/删除、Frame 元素归属和纯视觉连线创建/删除的完整 Document draft、固定 expected_version、幂等重试与冲突放弃/刷新流程；明确视觉连线不是 WorkItem/Jira 关系 | Phase 3E 接入既有 Canvas Document CAS API 的结构性编辑切片 |
| v2.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v3.9 / basic design v2.3；补充 Frame 几何/演示属性与 connector 样式 CAS、无实体引用便笺 Element version CAS，以及 Element 删除与 Canvas Document 引用清理的同事务边界；记录其余生产部署与实时消费 blocker | Phase 3E 补齐 Canvas 展示编辑和引用安全删除 |
| v2.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.0 / basic design v2.4；定义未锁定 Element 的宽高/旋转独立 Element CAS，明确字段保留、数值范围与冲突后刷新规则 | Phase 3E 将 Canvas Element 几何属性接入版本化写 API |
| v3.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.1 / basic design v2.5；定义 Worktree Index 授权投影映射、游标续页、seed fail-closed 边界，以及归档/恢复的短时 plan、二次确认与刷新契约；记录宿主 provider 未装配和 owner directory / Git checkout lifecycle 缺口 | Phase 2D 将 Worktree Index API 与 archive plan-confirm 连接到管理 UI |
| v3.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.2 / basic design v2.6；定义受 Project ACL 保护的成员目录 API 与负责人转派 UI，约束候选 UUID/role、manager 权限、二次确认、成员重验和失败刷新行为 | Phase 2D 补齐成员目录及负责人转派 UI/API 代码切片 |
| v3.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.3 / basic design v2.7；定义按 Worktree/Canvas 恢复并在授权 projection 成功后持久化浏览器 Outbox 游标的消费边界，明确本地游标不替代服务端 durable offset / NATS / realtime | Phase 3E 增加页面重载后可恢复的投影刷新游标 |
| v3.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.4 / basic design v2.8；定义 claimant 提交评审、非 claimant reviewer 通过/驳回、理由必填、Worktree scope、version CAS、幂等审计与 Task Card UI 条件式接线；保留 Review Domain adapter、Outbox、宿主 provider 与生产验收缺口 | Phase 2C / 3F 实现首个 WorkItem review command 与用户入口 |
| v3.4 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.5 / basic design v2.9；定义 Ed25519 Task CLI grant key id、API 私钥与 Runtime 公钥分离、验签 payload/domain、fail-closed nonce 消费顺序；明确签名 helper 未接 Session API、current ACL/health、审计或 PTY spawn | Phase 4A 增加签名 grant verifier 和受保护的执行准备入口 |
| v3.5 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.6 / basic design v3.0；定义 authenticated Task CLI Session start REST route、GroupApiState provisioner 注入、Worktree/WorkItem/claimant/version/Runtime 前置校验、idempotency fingerprint、grant/spawn 前实时复验和 no-store 单次 attachment ticket；明确当前无真实 provisioner，route 未产生 CLI session | Phase 4B1 建立可集成且默认 fail-closed 的 Task Session API 边界 |
| v3.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v4.7 / basic design v3.1；补充 `TaskPtyManager` 的 PTY 生命周期、原始 byte output、bounded input/resize、cleared process environment；明确该 helper 不含 sandbox / grant / ACL / TaskRun Audit，不接入 Group API 或 `terminal-stack` | Phase 4B2 建立可复用 PTY 执行适配，同时显式保留生产隔离与授权门 |
| v3.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.0 / basic design v3.3；记录 Phase 4B3 卡内 Session API 与 ticket-first xterm 接线；定义 Phase 5 scoped Chat endpoint 的显式 scope/targets、逐目标 GroupContext 检查、EntityRef 解析、幂等 fingerprint 与 202/503 边界；LangGraph、Transcript repository 和流式 UI 仍待完成 | 继续推进所有 Phase，完成 CLI Task Card 界面切片并启动 Phase 5 服务端授权入口 |
| v3.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.1 / basic design v3.4；定义授权 GLOBAL Worktree 目录 GET、membership/非归档过滤、最小字段与稳定 keyset 游标；增加底栏多选及 submit 再授权约束，workflow / Transcript / LangGraph 运行时仍未完成 | 继续推进所有 Phase，落实 GLOBAL 聊天目标发现与 UI 选择入口 |
| v4.0 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.2 / basic design v3.5；新增 Task CLI Session status/cancel/reattach REST contracts、专用 scope、逐请求 Group membership 与 Task/Worktree 关联校验、Runtime 完整 session binding 复验、脱敏状态、幂等 cancel 和全新短时单次 ticket；明确生产 provisioner 与 UI 重连仍缺 | 继续推进 Phase 4B session 生命周期控制面，保留无 Runtime 环境 fail closed |
| v4.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.3 / basic design v3.6；Task Card 接通状态刷新、幂等取消与人工 reattach UI；重连先查状态并校验新 ticket 的 session/Worktree/Task/Runtime binding 和有效期；明确 Session listing/recovery API、真实 provisioner、PTY sink、sandbox、Audit 与宿主 provider 仍缺 | Phase 4B3 完成 Session 生命周期 REST client 与卡内控制 UI 切片 |
| v4.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.4 / basic design v3.7；新增 Task/Worktree-scoped Session list GET 与列表上限/脱敏/no-store 契约；Task Card 在刷新后读取最近会话并提供显式状态刷新、取消和新 ticket 恢复；真实 provisioner/provider 未装配仍返回 503 | Phase 4B4 关闭页面刷新后无法发现 Session 的 API/UI 缺口并保持执行 fail closed |
| v4.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.5 / basic design v3.8；根据官方 LangGraph Python 文档细化服务端 thread ID、`Command(resume=...)`、checkpoint replay 重新执行节点、实时 scope/Plugin grant reauthorization、幂等 Domain Command/outbox 和 PostgresSaver 受控 setup 语义；记录目前只完成 API 语义核对，部署版本、独立 checkpoint store、Transcript/runtime/stream UI 仍未实现 | 继续 Phase 5 前先锁定 checkpoint 与 replay 的授权、隔离及副作用语义 |
| v4.6 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.8 / basic design v4.1；将 Transcript body 从明文列改为 AEAD ciphertext + wrapped key 的 W payload、保留 T 元数据；adapter 强制 protector seam；密钥服务、policy 清理、导出/删除与目标 DB 验收仍未完成 | 按 CHAT-006 收紧 Phase 5 本地 persistence schema 与 adapter |
| v4.7 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.9 / basic design v4.2；新增 `GET /group-apps` fail-closed Registry provider contract、实时 membership/binding/grant 复验、最小导航字段、数量与唯一性验证；实际 provider、enable/disable、hot revoke 和 sandbox runtime 仍缺 | Phase 6 开始建立 Worktree 同级插件导航的服务端授权读 seam |
| v4.8 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.10 / basic design v4.3；设计并装配只读 `PgGroupAppRegistryProvider`，在事务内复验 membership version、Worktree binding/archive、verified manifest/Host API compatibility 与 actor `group_app:open`；新增五表 Registry migration、Master SCD2/no-delete trigger 和 Audit append-only trigger/FORCE RLS；迁移未部署，manifest trust root/ingest、lifecycle writer、capability gateway、UI live consumer 与 PostgreSQL 真实验收仍缺 | Phase 6 从授权 read API seam 推进到数据库驱动的导航投影，并校正 Registry 数据治理边界 |
| v4.9 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.11 / basic design v4.4；Group UI 消费只读 `/group-apps` 投影并校验 Worktree ID、Registry version、entry 数量/唯一性/字段，支持稳定排序和定时、可见性、手动刷新；loading/error/provider generation 变化时清除旧入口且不回退预览；真实 provider、migration 部署、manifest trust/lifecycle/runtime、撤权和 PostgreSQL RLS 验收仍未完成 | 继续 Phase 6，完成只读 Registry 导航投影的 UI 消费切片 |
| v4.10 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.12 / basic design v4.5；抽取纯函数校验授权 Registry projection，并对 Worktree/correlation/version/entries、ID 唯一性、控制字符、长度、sort order 与稳定排序增加 3 项聚焦测试；`pnpm typecheck` 通过；真实认证 provider、数据库/RLS、manifest trust/lifecycle/runtime 和撤权仍未完成 | 继续 Phase 6，完成可在本地验证的授权导航数据边界 |
| v4.11 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正首页上位基本设计引用为当前 v4.5；补记 Group/terminal 与 REST 本地回归结果，保持 Phase 7 生产验收边界 | 文档版本交叉核对与本地回归发现引用及 fixture 需同步 |
| v4.12 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.13 / basic design v4.6；记录 Phase 5/6 migrations 在隔离 PostgreSQL 的幂等重放、12 张 FORCE RLS、scope/append-only 实测；新增 runtime role/bootstrap grant 与实际角色验收契约，明确目标库与生产身份未接通 | Phase 5/6 隔离数据库实测发现 RLS 与 SQL schema/table privilege 是独立权限门 |
| v1.1 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Local Runtime SQLite durable nonce consume helper 与 W 分类、TTL/时钟偏差清理策略；区分签名/ACL/start API 仍未接通的部分；上游同步 basic design v0.8 | 继续推进 Task Card CLI 授权与 session 闭环 |
| v1.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 盘点 terminal-stack 的协议/hub/snapshot 与 Noop sink、无授权 lazy pane 边界；定义 Phase 4B authenticated REST grant、Local Runtime PTY、首帧单次 ticket、实时 ACL 重验与 Task Card UI 接线；上游同步 basic design v0.9 | 继续推进所有 Phase，冻结受控终端 attachment 边界 |
| v1.2 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 盘点 terminal-stack 现状，定义 Phase 4B grant→PTY session→首帧单次 WebSocket attachment 流程；明确 Noop sink / lazy pane / pipe stdio 不满足受控 Task CLI，列出 Group UI Token Provider 与 Runtime PTY 的依赖 | 继续推进所有 Phase，基于现有 terminal-stack 推进 Task Card CLI Session |
| v1.3 | 2026-09-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 Local Runtime 单次 attachment ticket ledger 与 binding/TTL/清理契约；新增 protected WebSocket 首帧授权和真实 sink 注入 seam，并明确实际 ACL authorizer、Task Session API、Group router、PTY/sandbox 与 Bearer UI wiring 仍未接通；上游同步 basic design v1.0 | 继续推进所有 Phase，开始落地 Task Card CLI attachment 的可实现安全边界 |
| v4.13 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.14 / basic design v4.7；定义 Project Index 的 Host Runtime Git Worktree retention lock observation DTO、授权后查询、30 秒 freshness、单项/整页 timeout 与 unknown fallback；明确该信号不表示 Agent 活跃或文件互斥，区分历史持久 `locked` 标记，cleanup 需独立活跃状态、drain 与最终重观测；observer 未装配，生产验收仍缺 | Phase 2D 增加 Worktree lock 可见性切片并固定 cleanup 安全门 |
| v4.14 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.15 / basic design v4.8；新增 Project-scoped create/import candidate API contract、scope/role/membership checks、严格拒绝路径和仓库 URL、幂等 receipt 与 provider 503 边界；明确 Host Runtime adapter、归属/operation/audit/outbox writer 和 Index UI 尚未实现 | Phase 2D 补齐 Worktree 生命周期写接口边界 |
| v4.15 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.16 / basic design v4.9；新增脱敏 Project Repository discovery route 与 Index create/import UI 的校验、角色门、候选查询和受理后刷新契约；reject path/URL 型 Repository/candidate name 与响应多余字段；记录 Repository SoR、Host Runtime provider、durable writer、认证装配和生产验收仍未完成 | Phase 2D 把 Worktree 生命周期 seam 接到 Project Index 可交互入口 |
| v4.16 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.17 / basic design v5.0；新增 `/api/v1/projects` 有效 membership 目录与 UUID cursor、role/field allowlist、200 条上限及 no-store 契约；API session generation 更换时清除旧 Project/Index/member-role 投影；生产 Project 名称 SoR 与宿主认证/目标数据库验收仍未完成 | Project Selector 必须以当前用户授权目录为生产数据源，并清除跨 session 旧授权投影 |

| v4.19 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.19/basic design v5.3/Task DD v0.5；定义 Schedule/Engineering Loop、唯一 Automation occurrence owner、版本化 Agent Provider/CLI contract、Advanced Settings Hooks tab、Rust-native fail-closed Worktree gate 与 Run/BI provenance；Phase 9-13 实现门和当前未实现边界进入正文 | 用户要求多 Agent Worktree 作为管理核心，并将可扩展 Agent、Loop 与原生可视化 Hook 纳入同一体系 |
| v4.20 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 requirements v5.20/basic design v5.4/Task DD v0.6；补齐 Phase 8B Worktree/Task Run list/detail API 与 Task Card Run History UI 的 scope 授权、游标、字段和数量上限；记录 Run migration 仅在隔离数据库验证，目标 DB/RLS 与剩余事件 producer 未完成 | Phase 8B 新增 Run 历史代码切片，需要让 Group 详细设计反映真实边界 |
| v4.21 | 2026-09-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.9 与 Hook DD v0.5.4；记录 9B2B scoped policy API 已有代码/99 个 library tests 通过，目标 DB/grants/API RLS integration 和 9B2C lifecycle gate 未完成；重申 Hook 管理入口属于 Advanced Settings 并列 tab，Worktree 只消费有效策略与结果 | 完成 Hook policy REST 切片并同步 Group 级导航与生产边界 |
| v4.22 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步基本设计 v5.10 与 Hook DD v0.5.5；记录 9B2C REST archive-confirm gate 将 verified policy/Rust evaluator 接入 Worktree 归档确认：锁外先检查 Git lock，仅 fresh Unlocked 才请求 Runtime drain/readiness；释放行锁等待后重新授权与校验 lifecycle version/事实新鲜度；说明 production readiness provider、目标 DB/RLS、物理 cleanup、RunEvent/outbox 尚未接通；Hooks 仍留在 Advanced Settings 并列标签 | 推进 Phase 9B2C 并复核归档 provider 等待期间的数据库锁占用 |
| v4.23 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 basic design v5.11/Hook DD v0.5.6；补入 operation-scoped archive admission fence expiry、提交前最少 5 秒余量与事务期限/provider fence 联合验收缺口；保持 Hooks 位于 Advanced Settings 并列 tab | 收紧归档命令期间新 Run/lease 的接入竞态并同步阶段边界 |
| v4.24 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 basic design v5.15 与 Hook DD v0.5.10；记录 Project-scoped Hook summary API 的 1–90 天 bounded window、metric v1、phase/decision 聚合和 partial/null coverage；明确它只读 archive ledger，RunEvent/outcome join、Outbox/BI UI、目标 DB/RLS/grants 与 Hooks app auth Provider 仍开放；Hooks 继续属于 Advanced Settings 并列 tab | 推进 Phase 9D summary API 并保持 Worktree 导航边界 |
| v4.25 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 重构目标导航为 Project → Cloud Branch → Engineering Run → Run Worktree；Project Worktree Index 定位为跨 Run aggregate 管理面；定义 RunContext / Worktree focus、Run-owned同级 Apps 和旧 Worktree API/schema compatibility migration 边界；补充 owner API + same-owner stored procedure + Outbox/Inbox 与 NATS/Kafka/Fluvio 决策 | 用户确认 Project 主导航层级、Run 内 tabs 所属关系和原子化服务边界 |
