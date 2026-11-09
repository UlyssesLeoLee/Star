# BD-STAR-VERCEL-DEPLOY-001 — Vercel Web Hosting Basic Design

> **STAR Vercel 部署基本设计 v0.3**
> **状态：草案，未批准；VCD-0 OPEN；实现/部署状态：NOT_RUN**
> **上游：** `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md` v0.4、`docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md` v0.2（草案）
> **计划基线：** `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` v4.19
> **决策所有者：** 架构师（Mavis 接手审核）
> **日期：** 2026-10-24

## §0 文档目的与状态

定义 Star Web 呈现面使用 Vercel 的候选边界、组件职责、请求/发布流程、环境信任边界、能力门和需求追踪。此基本设计不授权生产部署，不证明平台能力，不新增 SRS 需求或验收阈值。Vercel 只是一条可选呈现/BFF 托管路径；现有 Star Owners、数据事实、后台消费者和执行设施维持权威。

### 0.1 证据分类

| 分类 | 本设计如何使用 |
|---|---|
| SRS 需求 | 仅使用 `SRS-STAR-VERCEL-DEPLOY-001.md` v0.4 的 FR-VC-001..021、NFR-VC-01..08、AC-VC-001..013、GAP-01..11。 |
| WBS 门 | 使用 VCD-0..VCD-4 与依赖关系；不从本设计文字宣称任何阶段完成。 |
| 平台事实 | 按 SRS §2.2、附录 B 与兼容性审查记录；Rust Runtime 的专属资格与限制仍为未知，须在目标账号/套餐验证。 |
| 设计决定 | 以本 ADR 草案与 `AGENTS.md` 的 Owner、Hook、Worktree、资源、隔离和 Multipass 约束为边界；仍待 VCD-0 批准。 |
| 未验证项 | 所有部署、账号、身份、预览隔离、Rust Runtime、数据库/RLS、性能与生产行为标记 **NOT_RUN**。 |

## §1 设计原则与拓扑

1. 一个业务事实只有一个既有 Star Owner；web 服务器/BFF/Rust function 是入口和薄 transport，不是授权或持久化权威。
2. 浏览器只访问 Vercel 同源 `/api/*`；服务端按 allowlist 转发到已配置的 Star API。不能确认会话、目标 Project/Run/Worktree 授权或 Owner 契约时，拒绝请求。
3. 不将 Agent 执行、任务卡 CLI、原生 Rust Hook、BI 事实计算、Canvas WSS、PTY/noVNC、Worktree 本地文件系统、Schedule 状态、Outbox/Inbox worker、Multipass/K3s 控制面迁入 Vercel。
4. Preview 与 Production 使用分离的访问保护、凭据、数据和后端环境。来自未信任分支/Agent 的 Preview 不可触达 Production Owner、生产密钥、业务写接口或 Schedule trigger。
5. 异步语义和成功状态以 Star Owner 的持久事实、幂等回执和 Outbox/Inbox 为准；HTTP 202、函数退出或部署成功都不证明执行成功。
6. 前端采用按需加载、分页和有界缓存；函数 adapter 无进程级业务缓存/队列，限制连接数与载荷，避免把函数实例内存用于跨请求状态。
7. Engineering Run 是 Star 在 Project 下的一级业务身份，可以在没有远端宿主连接或远端 Branch 身份时创建、授权、列出和管理。远端 repository、branch、provider 关联均为可选元数据，不能成为 Run 身份、授权或上述管理操作的前置条件；调用 provider 专属操作时，才额外要求有效关联和 provider 授权。

### 1.1 组件拓扑

```text
Browser
  │ HTTPS; Star session cookie; same-origin calls only
  ▼
Vercel Web Project (Next.js 16)
  ├─ Static/SSR presentation
  ├─ Protected Preview / Production environments
  └─ Same-origin BFF (/api/*; method/path allowlist; no-store)
       │ server-side credential forwarding; no client-visible backend URL
       ▼
Star API / existing authoritative Star Owners
  ├─ current identity + Project/Run/Worktree authorization
  ├─ owner-local transaction + idempotency + audit + durable receipt
  └─ owner-local transactional Outbox → existing Inbox/consumer
       │
       ├─ Star Run/Execution and BI owners
       ├─ Star Schedule owner + reconciliation
       └─ Star Infrastructure owner → trusted Rust manager
             → Multipass guest(s) → bound K3s cluster / sandbox

Star-owned execution plane (unchanged): Agent, native Hooks, task CLI,
PTY/noVNC, Canvas real-time gateway, local Worktree filesystem,
workers and Multipass/K3s operations.
```

Vercel UI may read only the capabilities exposed by the authenticated Star API. Project → Engineering Run → Worktree → Branch ownership and the selected Run/Worktree context remain defined by Star. BI/Benchmark values are Star-owned and must not be recomputed from incomplete Vercel telemetry as business truth.

### 1.2 Engineering Run 身份与可选远端关联

- Star Owner 在 Project 下创建并持有 Engineering Run 的身份、授权与生命周期。Run 的创建、授权、列表和管理请求只依据 Star 的 Project/Run 权限及 Owner 契约；不得要求用户先连接远端宿主、绑定远端 repository 或选择远端 Branch。
- 远端 repository、branch 和 provider 关联以可空、可独立变更的元数据保存，不形成 Run 的父级或身份来源。未关联时，UI/API 仍可呈现和管理 Run；涉及远端 provider 的专属操作明确显示未连接/不可用并拒绝执行，不得伪造关联或降级为 mock。
- 已关联时，Star 权限仍是 Run 操作的授权依据。provider 专属操作还必须单独验证当前 provider 凭据与权限；provider 关联失效或被移除只影响这些专属操作，不得隐式删除 Run、改变其 Star 身份或把 provider 授权当成 Star 授权。
- Worktree 仍是 Run 管理的本地 checkout；本节不要求建立远端 Branch，也不改变 Worktree/Branch 的本地 checkout 语义。Vercel 部署只提供受保护的呈现面与 BFF，不拥有上述 Run 身份或权限。

## §2 组件与部署职责

| 组件 | 阶段/位置 | 职责 | 禁止职责 |
|---|---|---|---|
| Next.js Web Project | VCD-1 / Vercel | 静态资源、SSR、应用界面；无客户端后端主机地址；按功能显示不可用状态。 | 不持有业务状态权威，不把未连通 API 降级为 mock。 |
| Same-origin BFF routes | VCD-1 / Next.js server routes | 对浏览器 API 请求执行方法、路径、Origin、cookie/凭据和载荷限制；无状态地代理 allowlist API。 | 不自行授权 Project/Run，不直接写 Star 表，不建立 worker、队列、业务缓存或迁移逻辑。 |
| Optional Rust HTTP Functions | VCD-2 proof; VCD-3 conditional | 仅在全门通过后，作为独立构建单元提供指定的无状态 HTTP adapter，调用已存在的 Star Owner 契约。 | 未经批准不得部署；不得新增 Owner、数据库权威、DDL 权限、worker、scheduler、WS 服务、shell、Worktree 或 VM 权限。 |
| Star API / Owner | 既有权威运行面 | 持有 Project 下的 Engineering Run 身份；独立于远端宿主关联执行授权、创建、列表与管理；负责业务读取/命令、事务、幂等、审计、持久回执和 Outbox。 | 不得把 repository/branch/provider 关联作为 Run 身份或授权前提；不得信任 BFF 传入的用户/Run 身份声明而跳过当前授权。 |
| Star consumer / Runtime / Infrastructure | 既有权威执行面 | 消费持久事件；管理任务执行、Hooks、工作区、Schedule、BI 与受控 Multipass/K3s 生命周期。 | 不接受 Vercel 函数直接管理宿主、任意 shell 或集群凭据。 |

部署形态不预先指定目标 Vercel 账号、套餐、区域、构建形态或 PostgreSQL 服务商。Rust 构建使用仓外独立 `Cargo.toml` 或预构建方案只能作为 VCD-2 评估候选；未通过 GAP-10 构建根验证前不创建生产 Rust Project。

## §3 主要流程

### 3.1 页面读取与 BFF 代理

1. 浏览器访问受保护的 Preview/Production Web Project，并仅调用相对路径 `/api/*`。
2. BFF 校验 HTTP method 与路径白名单，并拒绝路径段中的 `..`/分隔符绕过；非 GET 请求校验 Origin，Cookie 名及 token 长度受限，Production 上游必须是 HTTPS 且不得含 user-info 凭据。请求体不得超过 2,000,000 bytes。响应 `no-store`、不跟随重定向并拒绝 3xx，剥离 hop-by-hop 与 `set-cookie` 响应头；任一项失败即拒绝请求。
3. BFF 从服务端会话凭据取出必要 token，仅向固定 Star API 上游附带授权凭据；不得接受客户端指定 upstream、任意 URL、用户身份 header 或环境变量覆盖。
4. Star API/Owner 对每个请求校验 session issuer/audience、当前 Project/Run/Worktree scope 与能力授权。Run 的核心创建、授权、列表和管理仅要求 Star Project/Run 权限；远端 provider 操作另行校验可选关联及 provider 权限。host-session 实际 issuer、audience 与 Run contract 由 VCD-1/WLD-0 验证；未证实则不开放该路径。
5. 返回最少必要数据。UI 对 401/403、Owner 不可达及能力未启用显示明确状态，不回退 mock。

### 3.2 写请求与持久化事实

1. BFF 只代理明确 allowlist 的写命令，并保留调用方幂等键/请求关联信息；它不产生业务成功事实。
2. Star Owner 实时复验身份、Run/Worktree scope、策略版本和执行能力，在本 Owner 事务中执行授权检查、幂等登记、业务事实、审计和 Outbox 写入。
3. 多 Owner 流程通过各自 Inbox/Owner command 和 transactional Outbox 推进；不跨 Owner 共享写事务或由 BFF 写他人表。
4. transport timeout 或断连后显示 `state_unknown` 并查询同一 Owner/幂等键的权威回执；不得自动生成新幂等键重试，不以 HTTP 状态推断 Task/Agent 已运行。
5. 如果某写路径缺少可验证的 Owner/API、当前权限、幂等回执、Outbox 或审计契约，该路径保持关闭。VCD-3 需对每个 endpoint 逐项批准，不由一般 BFF 接入推导。

### 3.3 Schedule 唤醒与对账

1. 可选的 Vercel Cron 仅发出无业务负载的唤醒/对账请求；不包含任务执行、预约或 Schedule 状态转换。
2. 既有 Star Schedule owner 扫描持久未完成工作，以 cursor、幂等 occurrence/receipt 与既有 lease/fencing 恢复；漏发、重复、并发重叠或失败均不产生重复业务事实。
3. 每次 trigger/retry/resume 都按不可变 `run_as_actor_id` 实时复验当前 Project/Run 权限与 execution capability，并固定当时有效 `RunInfrastructureBinding`；撤权、授权目录不可用或事实不完整时 fail closed。
4. Cron 不自动重试失败调用。部署回滚后，运维核对目标 deployment 恢复的 Cron definitions，并与 Star Schedule owner 持久状态进行对账。
5. 没有可靠唤醒时由 owner 侧既有恢复/对账机制保证持久事实可再次发现；Cron 不作为 SLA 调度或交付承诺。

### 3.4 Preview 与 Production 发布

1. Pull request Preview 使用访问保护和隔离凭据/数据；不暴露生产后端 token，不连生产数据库/Owner，不具有 Schedule trigger 或 Outbox 写权限。
2. 未信任提交的 Preview 运行相同的最小权限策略；不得由构建变量或前端代码切换到生产 API。
3. Promotion 仅由受保护审批身份触发；部署目标、构建 commit、环境、审批身份与回滚目标可审计。
4. 回滚只恢复 Web deployment 配置和其关联 Cron definitions；不回滚、重写或复制 Star Owner 业务事实。回滚后执行 §3.3 对账。
5. 任一隔离证明缺失时，Preview 不可访问任何生产能力；将功能标为 disabled，而不是以本地 seed/mock 继续。

## §4 信任与环境边界

| 边界 | 允许 | 不允许 / fail-closed 条件 |
|---|---|---|
| Browser → BFF | 同源 HTTPS、受限 Cookie/session、allowlist 请求。 | 浏览器取得 backend host、bearer token、平台 secret；客户端选择 upstream；违反 Origin/path/method/size 策略。 |
| BFF → Star API | 服务端固定上游、服务端凭据透传、有限请求/响应。 | BFF 代替 Owner 判定 Run 权限；信任未验证身份 header；写 Star DB 或发布事件。 |
| Vercel Preview → Star | 仅隔离环境与非生产测试 Owner/数据（且验证访问保护）。 | 生产 secrets/data、生产 Owner 副作用、Schedule trigger、生产 Outbox；无法证明隔离即拒绝。 |
| Vercel Production → Star | 对实际用户会话授权的受限读取/获批写命令。 | 任意函数代码获得宿主/Kubernetes/Mount 权限；把管理凭据放入浏览器。 |
| Rust Function → Owner（如未来获批） | 无状态、allowlist HTTP command/query；Star Owner 对每次调用复验。 | 新权威、跨 Owner 直接表写、进程内状态、DDL、常驻 worker、Agent runtime、Hook 放行。 |
| Cron → Schedule Owner | 可认证、可重复的 wake-up/reconciliation。 | 创建执行器、跳过 `run_as_actor_id` 权限检查、占用 Host/cluster 预算或当作唯一调度事实。 |

Production secret 与 Preview secret 分别管理；任何 secret 不进入 `NEXT_PUBLIC_*`、浏览器 bundle、URL、终端或日志。不得导出环境变量值以排障。IP allowlist 不能单独作为平台身份验证。数据库连接凭据只在已批准的服务端 Owner/adapter 环境内使用。

## §5 能力归属与 UI 行为

| 能力 | 权威归属 | Vercel 形态可见行为 |
|---|---|---|
| 登录、Project/Run/Worktree 权限 | Star identity/permission 与当前 host-session contract | 按 Star 权限呈现授权数据；无远端宿主/Branch 关联的 Run 仍可创建、授权、列出和管理。provider 专属操作须单独通过 provider 授权；权限事实不完整时拒绝。 |
| Inbox、Task Card、Canvas 数据、Run BI/Benchmark | Star 对应 Owner/API | 仅显示已授权读取结果；无接口/无数据时明确标示不可用/未知。 |
| Agent 执行、任务卡 CLI、sandbox、资源预约 | Star Run/Execution + Infrastructure owners 与 Rust Runtime | 禁用执行并说明需要 Star 本地/托管执行面；不提供任意 shell fallback。 |
| 原生 Hook | Star Rust Hook 核心 | 展示有效策略/结果或禁用状态；任何 Vercel 插件/route 不可替代安全关键 Hook。 |
| Canvas 实时 WSS、PTY/noVNC、ARG WS | Star 自有长连接网关/执行面 | 不由 Vercel function 承载。页面可直连已验证 Star gateway；无 gateway 授权时禁用。 |
| chat SSE | Star API/gateway；受限 Next.js route 可选透传，最终路由尚未选定 | 先在 SRS/ADR/BD/DD/TD 与真实 session、授权/撤权、Origin/跨站防护、streaming 和生命周期契约中统一并验证，再开启；不从 Rust Runtime WebSocket 状态推导。 |
| Schedule 与 Cron | Star Schedule Owner；Vercel Cron 可选 wake-up | 展示持久 Schedule 状态；Cron 仅请求 owner reconcile，不显示为调度成功本身。 |
| Multipass/K3s、RunInfrastructureBinding | Star Infrastructure Owner 与可信 Rust Host Manager | Vercel 不调用 Multipass daemon/socket，不访问 host shell 或 cluster 凭据。测试 VM 受两条记录总上限约束。 |

## §6 性能、构建和资源限制

### 6.1 前端与 BFF

- 静态/SSR 页面和同源 BFF 与 Star execution plane 独立部署。所有运行时环境必需配置由构建/部署校验；关键环境缺失时构建或启动失败，不能静默选择 mock。
- 浏览器端不缓存 token、整段日志或全量 Worktree/Run 历史。使用现有分页/按需请求和有界缓存原则；大日志/制品走专门对象存储或 Star API 的受控下载，而非函数 payload。
- BFF 请求体执行 SRS 既有 2,000,000-byte 上限；函数请求/响应遵守平台 ≤4.5 MB 载荷上限（NFR-VC-02）。超限拒绝或让客户端改用受控上传路径。没有经验证的上传路径时，不伪装为成功。
- 路由处理必须有有限 upstream timeout、有限连接和请求并发；确切数值由 VCD-1/VCD-2 目标套餐实测并写入详细设计，不在本 BD 增加新门槛。

### 6.1.1 控制面请求延迟（NFR-VC-01）

- **目标**：控制面请求端到端响应延迟 p95 < 2 s。计时从同源 BFF 开始处理请求起，至 BFF 完成向调用方返回响应止；包括身份/Project/Run/Worktree 授权、同步 Star API/Owner 处理，以及命令被接受时提交持久化业务事实与 Outbox 所需的同步事务时间。
- **异步边界**：Outbox relay、Inbox/consumer、Agent/CLI 执行、Schedule 后续处理及业务结果完成均发生在响应返回后，不计入该 HTTP 请求延迟；异步命令只能在持久化接受事实后返回相应回执（如 HTTP 202），回执不表示业务执行完成。函数内不得通过长轮询等待异步结果。
- **测量与证据**：VCD-1/VCD-2 应按控制面操作族记录端到端直方图和 p95，关联 request/correlation ID、部署版本、运行时及响应类别；不得让聚合均值掩盖慢操作族，也不得把未完成、超时或错误请求静默当作成功样本。目标运行时、套餐、配置、代表性工作负载和观测窗口须在详细设计/验证方案中记录并实测；不得据通用平台限制推定 Rust 专属 maxDuration。
- **执行规则**：每个同步请求均须在所选且已验证运行时的 maxDuration 内结束。无法在该界限内完成的业务工作转为 Owner 持久化异步命令；若目标运行时限制、负载或 p95 证据未验证，相关 VCD 阶段保持 NOT_RUN，不宣称该 NFR 已通过。

### 6.2 已有平台基线（非 Rust 专属承诺）

| 维度 | SRS 基线 | 设计处理 |
|---|---|---|
| 请求/响应 payload | ≤ 4.5 MB（NFR-VC-02） | BFF 限制载荷；大文件不经过函数体。 |
| Rust 包体 | 250 MB 保守预算（NFR-VC-03、GAP-01/11） | 按每个真实 handler 的依赖闭包测量；未知或超预算时不部署 Rust endpoint。 |
| 内存/并发 | Rust 目标 Runtime 实测与确认后设预算（NFR-VC-04） | 不把平台通用规格外推为 Rust 保证；记录 RSS、并发、fd、连接数与成本。 |
| 区域/驻留 | 固定到与数据可达、驻留策略相容且已验证的区域（NFR-VC-05） | 不预置区域；在部署设计中记录选择与故障域。 |
| 冷启动 | 实测后定档（NFR-VC-06） | 无测量数据时标记 unknown，不虚构 SLO。 |
| 调度 | 不承载 SLA 级调度（NFR-VC-07） | Schedule 精度与恢复由 Star 执行面保证。 |
| 工具链 | Rust MSRV 1.99.1 / edition 2021（NFR-VC-08） | VCD-2 验证目标构建镜像；验证前无 Rust 生产 handler。 |

### 6.3 Rust Runtime 与数据库门

- Rust Runtime 当前按 SRS 标记为 Beta。公开通用 Functions WebSocket GA 不证明 Rust Runtime 支持 WebSocket/Streaming；Rust 专属 duration、内存、包体扩展、并发、FD、日志/指标和目标计划资格须单独实测（GAP-02、GAP-09、GAP-11）。
- VCD-2 使用无生产数据、无业务写入的最小样例及代表性 dependency closure 验证 Rust 1.99.1、两种已列构建形态、`api/*.rs`/`[[bin]]`、部署、限额、RSS/冷启动、连接复用、日志/指标和回滚。任何未知或失败保持 Rust 端点关闭。
- 不在本设计新增数据库、migration owner 或数据权威。任何 Rust/托管 PostgreSQL 接入须先明确它服务于现有 Star Owner 的权威持久化、通过 W/T/M 与 RLS 评审，证明事务级 `SET LOCAL`、连接池复用和租户隔离；否则 Vercel 不直接连数据库，继续经现有 Star API。
- 迁移不在函数内运行。数据库 schema/迁移 authority 保持在既有 Star 迁移流程；任何托管 PostgreSQL 服务商选择仍受 SRS GAP-06 与 WLD owner review 约束。

### 6.4 虚拟机与执行限制

Vercel 无权创建或管理本地 VM、Multipass instance、K3s cluster、sandbox 或 Worktree 文件系统。若后续独立 VCD-2 fixture 使用 Multipass，必须使用受控加锁 launcher；计入全部保留记录，最多两个；inventory 不可读或已满时停止，不 purge。Docker Desktop 不是依赖项。虚拟机就绪本身不证明 Agent runtime、Hook、授权、隔离或任务 CLI 闭环可用。

## §7 可观测、失败与恢复

- 为每个 BFF 请求保留 request/correlation ID、环境、路由与结果类别；不得记录 token/cookie/secret/完整敏感载荷。Star Owner 的 audit/BI 事件与其持久回执相关联。
- 指标区分 Web/BFF transport 成功、Owner 接受、Outbox 已提交、consumer 已处理、Task execution 完成；各状态缺证据时报告 unknown，不将其合并成单一“成功”。
- 429/超时/断连使用有限重试提示；有副作用的调用先查同一幂等键下 Owner 回执，状态未知时不产生新命令。权限错误不通过刷新/重放有副作用请求来规避。
- 部署回滚保留 Star 侧业务事实；检查恢复后的 Cron definitions，并由 Schedule owner 对账。平台观测缺失不得替代 owner-side durable cursor 或 BI 数据。
- VCD-4 前，目标套餐、区域、成本、延迟、RSS、冷启动和 Cron 运维门均记录为 NOT_RUN/unknown。

## §8 验收门与状态

所有 SRS 的 `AC-VC-001..013` 当前均为 **NOT_RUN**。执行顺序为 WBS 的 VCD-1（受保护前端）、VCD-2（隔离 Rust 可行性；可与适用项并行，但不能越过 VCD-0）、VCD-3（条件式 Owner-backed adapter）、VCD-4（运维/Cron/成本/BI 闭环）。VCD-3 要求 VCD-1、VCD-2 PASS、WLD-0，产生事件还要求 WLD-6；VCD-4 按 WBS 所列 WLD-3/WLD-6 和 VCD-1/可选 VCD-3 的依赖执行。

| 阶段 | 交付与既有验收引用 | 当前状态 |
|---|---|---|
| VCD-0 | ADR + BD + DD + TD 与现行 SRS 及 WBS 互相一致；追踪本节矩阵并处理三份兼容性审查结论。 | OPEN；本文档草案，未批准。 |
| VCD-1 | 受保护、可回滚的真实前端/BFF；`AC-VC-001..006`、`AC-VC-013`。 | NOT_RUN。 |
| VCD-2 | 隔离 Rust runtime/build/account/plan proof；`AC-VC-007..011` 与 `GAP-01..04/GAP-09..11`。 | NOT_RUN；未知即不启用 Rust 路径。 |
| VCD-3 | 满足 WLD-0/WLD-6 的 allowlisted endpoint 与 owner 回执；`AC-VC-008..010`、`AC-VC-012`。 | NOT_RUN；依赖门未证明。 |
| VCD-4 | 运行、成本、Cron 和回滚 reconciliation；`AC-VC-005/011/012/013`、`FR-VC-014/015`。 | NOT_RUN。 |

### 8.1 SRS 与架构约束追踪矩阵

本矩阵复用 SRS 已批准 ID，并单列 Star 架构约束；本设计不创建新的 SRS 或 AC 编号。详细测试设计应将测试用例逐条映射回这些验收引用，执行后记录证据；当前验证状态仍为 NOT_RUN。

| SRS IDs / architecture constraint | 设计位置 | 既有验收引用 |
|---|---|---|
| Star Worktree-first Run invariant（`AGENTS.md`） | §§1.2、2、3.1（第 4 步）、5 | Run 核心创建/授权/列表/管理不依赖远端宿主 Branch；provider 专属操作另行授权。运行态 Owner/API 证据待 VCD-1/WLD-0 核验。 |
| FR-VC-001..004 | §§1、2、6.3、8 | AC-VC-001、007、008、010、011 |
| FR-VC-005..008 | §§3.1、4、6.1 | AC-VC-002..005、013 |
| FR-VC-009..012 | §§2、3.2、4、6.3 | AC-VC-009、010、012 |
| FR-VC-013..015 | §§3.3、5、7 | AC-VC-005、012 |
| FR-VC-016..021 | §§3.4、4、5、7 | AC-VC-003、005、006、013 |
| NFR-VC-01 | §§3.2、6.1.1、7、8 | SRS p95 < 2 s；VCD-1/VCD-2 按操作族测量端到端响应。现有 AC-VC-001/007 不构成 p95 通过证据；专门的测量及结果仍待执行（NOT_RUN）。 |
| NFR-VC-02..08 | §6 | AC-VC-001、003、004、007、009、011、012 |
| GAP-01、GAP-03、GAP-09..11 | §§6.2、6.3、8 | AC-VC-007、011；VCD-2 |
| GAP-02 | §§3.3、6.3 | AC-VC-005、007；VCD-2 |
| GAP-04、GAP-06 | §6.3 | AC-VC-009、010；VCD-2 / WLD owner review |
| GAP-05、GAP-07、GAP-08 | §§3.1、3.4、6.1 | AC-VC-001、003..006、013；VCD-1 |

## §9 风险与未决项

| 项 | 处理状态 |
|---|---|
| host-session issuer/audience、浏览器 session 与 Run scope 的真实契约 | VCD-1/WLD-0 验证前未知；不得以 BFF 头部作为授权来源。 |
| Rust Runtime 的 MSRV、功能表、durations、RSS、WebSocket/streaming、并发与目标套餐资格 | GAP-02/03/09/11，VCD-2 实测前未知。 |
| Rust 构建根、path dependencies、预构建输出及每函数依赖包体 | GAP-01/10，VCD-2 取证；不提前选定。 |
| PostgreSQL 的服务商、网络可达、Owner 持久化与事务级 RLS 兼容 | GAP-04/06 + WLD owner review；不新增 database/migration authority。 |
| 真实目标账号、Preview protection、环境隔离、区域和 rollback 操作 | VCD-1/VCD-4 NOT_RUN。 |
| WebSocket 长连接 | 通用 Functions 已 GA；Rust Runtime 专属支持未证实。 Star Canvas WSS/PTY 保留原执行面。 |
| Cron miss/duplicate/overlap/failure 与 rollback 后 Schedule 对账 | 保留在 VCD-4；Cron 不是 Schedule truth，执行前复验 `run_as_actor_id` 与 `RunInfrastructureBinding`。 |
| 许可、SBOM、部署后端及完整供应链 | FR-VC-019；VCD-1/VCD-2 交付时核验完整分发闭包，不因本设计宣称完成。 |

## §10 与兼容性审查的决议

- 采纳受保护 Preview/Production、同源 BFF、fail-closed/no-mock、Star Owner 持久事实、Cron 唤醒/对账、长连接与执行面分离的兼容意见。
- 不纳入第二 Owner、第二数据库权威、独立迁移权威、独立 worker/scheduler、Vercel VM/Agent execution provider 或未批准的扩展 ID。
- 以 SRS v0.4 的 ID 范围为唯一 Vercel 需求/验收编号来源。通用 Functions WebSocket GA 与 Rust Runtime Beta/未知能力严格区分。
- Cron rollback 的官方语义按当前兼容审查：目标 deployment 的 Cron definitions 恢复，项目级 Disable Cron Jobs 状态保留。仍需运行后检查有效定义并由 Star Schedule owner 对账。

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于 SRS v0.4、WBS v4.18 和 VCD0-BASIC-ADR/DD/TD 三份兼容性审查创建；所有部署/账号/runtime/生产门保持 NOT_RUN，VCD-0 OPEN。 |
| v0.2 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Engineering Run 可脱离远端宿主 Branch 身份创建、授权、列出和管理，远端关联仅为可选元数据；将 NFR-VC-01 的 p95 < 2 s 计时范围、异步完成边界、测量证据和 NOT_RUN 状态写入基本设计。未执行平台或运行时验证，VCD-0 仍 OPEN。 |
| v0.3 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 chat SSE 可选 Next.js route 只是候选路径，尚未选定；将其与 SRS/ADR/DD/TD 及真实 session、授权、Origin/跨站防护、streaming、生命周期契约一致性列为启用前门。未执行验证，VCD-0 仍 OPEN。 |
