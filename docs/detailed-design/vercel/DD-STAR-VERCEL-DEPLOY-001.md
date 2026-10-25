# DD-STAR-VERCEL-DEPLOY-001

> **STAR Vercel 部署详细设计 v0.3**
>
> - 状态：Draft；VCD-0 设计输入，未批准、未实施、未部署
> - 需求基线：`docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md` v0.4
> - 基本设计：`docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md`
> - ADR：`docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md`
> - 工作计划：`docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` 的 VCD-0..VCD-4
> - 日期：2026-10-24
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 审批：架构师（Mavis 接手 agent per DEC-008）

本文定义受控的 Vercel Web 部署与条件式 Rust HTTP 可行性路径。它是设计基线，不表示任一 Vercel Project、账户能力、API、Owner、数据库、Cron 或运行时已配置或验证。任何带有“待验证”“条件”“未知”的条款在其证据门关闭前均不得视为上线能力。

## 1. 目标、范围和架构不变量

### 1.1 本设计目标

- 让 Web 呈现面能够独立部署到 Vercel，并经同源 BFF 访问 Star 的受保护 API。
- 保留 Star 现有身份会话、Project → Engineering Run → Worktree → Branch 授权及业务 Owner 的事实权威。
- 将环境隔离、构建契约、能力降级、回滚、观测和后续可行性验证写成可审核门禁。
- 仅在 Rust Runtime 通过独立可行性证明、Owner/API 映射以及 WBS 前置门后，评估小型无状态 HTTP adapter。

### 1.2 不变量

1. Vercel 是可选的 Web 托管与受限 HTTP 传输层，不建立第二套业务真相。
2. Star Owner 负责当前授权、业务写入、持久回执、审计、事务 Outbox/Inbox、Schedule 和 Run 执行事实。
3. BFF 只执行固定上游、允许方法和路径内的请求转发与安全过滤。它不得签发身份、推导 Project/Run 权限、独立落业务事实或把 HTTP 成功等同于执行成功。
4. Preview 与 Production 是隔离的部署环境。未信任 Preview 不得读取生产数据、持有生产密钥、写入生产 Owner、发布生产 Outbox 事件或触发生产 Schedule。
5. Agent 执行、任务卡 CLI、PTY/noVNC、Canvas 实时协作、原生 Rust Hook、Worktree 文件系统、Multipass、K3s 和常驻消费者均留在 Star 的自有执行面。Vercel 页面只呈现其可达状态；缺少真实能力时明确禁用并 fail closed。
6. Cron 只可作为 best-effort 唤醒信号。它不拥有 Schedule 真相、执行预算、基础设施预留、Run 创建或 Agent 启动。
7. 不新增 Owner、业务数据库、迁移器、独立 worker/scheduler、对象存储事实平面或 VM 执行平面。若未来提出这些边界变更，须单独修订需求、ADR 和 WBS。

## 2. 部署和调用拓扑

### 2.1 Phase 1：呈现面与同源 BFF

    浏览器
      -> Vercel Next.js 页面与同源 BFF
      -> 固定的 Star API 上游
      -> Star 身份与 Project/Run 授权
      -> 对应 Star Owner / 既有持久化及 Outbox/Inbox
      -> Star 自有执行面（仅由 Star 控制面授权后使用）

- Phase 1 使用一个 Next.js Vercel Project；Preview/Production 由该 Project 的受保护环境配置隔离。它们是部署环境，不是不同的业务 Owner。
- BFF 作为 Next.js Route Handler 运行在请求生命周期内，不把内存当作共享状态，不保持后台消费循环，也不代理 PTY、CRDT 会话或本地文件访问。
- 浏览器调用已有同源路由骨架；服务端上游地址由受保护的环境配置固定，不接受请求参数、Host 或转发头改变目标地址。
- 既有实现入口 `frontend/src/app/api/group/[...path]/route.ts` 是安全约束参考。通用化或扩展路由仍须先有具体 Star API 契约；本设计不命名新 API endpoint。
- 同源 BFF 仅能转发当前已实现并获授权的功能。新页面、静态配置、preview adapter、seed 或 mock 不能证明业务闭环。

### 2.2 Phase 2 候选：Rust HTTP adapter

Rust 函数不是当前部署目标。只有 VCD-2 目标账户和套餐可行性门通过、Owner/API/身份/数据库边界完成评审，且 WLD-0/WLD-6 前置条件满足后，才进入 VCD-3 的 adapter 设计和实施。即使获准：

- 函数仍为无状态入口，必须调用现存 Star Owner；不得绕过 Owner 直接形成新事实。
- 必须沿用真实宿主会话与 Star 当前授权，不得引入独立 Identity Owner。
- 仅增加薄 adapter；不得复制 Domain 业务逻辑、引入新业务存储、迁移器、进程内 EventBus、缓存事实或 worker 循环。
- Rust Runtime 当前为 Beta。其专属 WebSocket、streaming、maxDuration、内存、并发、日志和指标支持保持 UNKNOWN，直至目标账户/计划实测。
- 通用 Vercel Functions WebSocket GA 不能证明 Rust Runtime 的 WebSocket 支持。现有 PTY、Canvas WSS 和长生命周期协作通道仍由 Star 自有运行时承载。

### 2.3 Star Gateway 长连接与 chat SSE（契约待批准）

长生命周期连接保持在 Star 自有 Gateway/执行面，不进入 Vercel BFF 的代理或持久状态路径。TD T-049 覆盖五类浏览器连接：terminal PTY/session WebSocket、noVNC 桌面会话、ARG WebSocket、Canvas WebSocket 与 chat SSE。前四类的 Gateway 数据路径、凭证握手、Preview Origin allow/deny、当前 Project/Run/capability 授权、撤权传播、协议事件/帧、heartbeat、关闭/重连生命周期及可审计证据，均须由真实 Star Gateway 契约确认。当前文档集没有足以确定这些协议细节的已批准契约，本设计不推定 issuer、token/cookie 形式、握手消息、Origin 值、URL 或 wire format。

SRS FR-VC-013 允许 chat SSE 经 Next Route Handler 透传；TD T-049 则要求按最终批准的路由验证 chat SSE，其中允许采用 SRS 已列出的 Next Route Handler 路径。**路由选择仍未批准**，且必须在 SRS/ADR/BD/DD/TD 中统一，并证明该路由的真实 session、当前授权/撤权、Origin、streaming、关闭/重连及审计契约。不得把 T-049 解读为 chat SSE 必须经 Gateway，也不得自行发明协议。选定路由且完成对应契约验证前，Vercel chat SSE 保持 unavailable；不启用未批准的 BFF/Function/Gateway 路径或隐式回退。T-049 保持 NOT_RUN，VCD-0 保持 OPEN。

## 3. 请求、授权和 BFF 安全边界

### 3.1 身份与范围授权

1. 浏览器会话必须来自当前嵌入宿主实际提供的会话契约。凭证只通过受保护的 HttpOnly cookie 由服务端读取，再向固定 Star 上游转发。
2. BFF 不接受客户端提供的 user、tenant、Project、Run 角色或授权结论作为身份依据；不得从自定义请求头重建身份，也不得以出口 IP 白名单代替 token/OIDC 授权。
3. Star API/Owner 对每个读取和写入操作复验当下的身份、Project/Run 范围与能力。UI 隐藏按钮不是授权。
4. 授权目录不可用、会话缺失、token 无效或 Run 绑定不完整时，按真实认证/授权错误 fail closed。401 触发会话失效；403 表示当前范围无权，不得伪装为登录失效或重放写请求。
5. 跨 Run、跨 Project 和越权 Worktree 请求必须由 Star 端拒绝；不能因 BFF 路径前缀或浏览器选中项而推定有权限。

### 3.2 BFF 请求约束

每一条现有或未来的同源转发路径都应满足以下条件：

- 使用明确的 HTTP method allowlist；未知方法拒绝。
- 对非安全方法执行同源 Origin/CSRF 检查；缺失或不匹配时拒绝。
- 只读取允许的 cookie 名，限制凭证长度；拒绝不合规值。
- Production 上游必须为 HTTPS；拒绝带用户信息、非预期 host 或凭证的上游 URL。
- 使用路径 allowlist，并逐段校验编码、`..`、斜杠和规范化结果，阻止路径逃逸和代理到任意地址。
- 请求体双重限制为既有实现的 2,000,000 bytes；同时不得超过平台 4.5MB 载荷边界。大文件与大日志使用 Star 自有传输面。
- 以 no-store 处理 actor-scoped 数据；关闭自动 redirect，拒绝意外 3xx。
- 从上游响应中移除 hop-by-hop 头和 Set-Cookie；除明确允许的响应头外不透传。
- 日志只记录必要的 request/correlation 元数据，不记录 token、cookie、完整敏感 body 或可复用凭证。

各环境中的 public build config 只保存 SRS 所需的非敏感模式/同源配置值；后端地址、密钥、token 和凭证绝不能进入 NEXT_PUBLIC_* 或客户端 bundle。FR-VC-005 所列 build-time 变量必须按 Build/Production 环境显式声明；缺项使构建失败，不能退回 mock。

### 3.3 写入结果与幂等恢复

- BFF/函数返回的 HTTP 状态只描述该次传输或 Owner API 收据，不表示 Agent 已执行、Hook 已通过或业务工作已成功。
- 当请求可能已到达 Owner、但响应丢失/超时，客户端进入 `state_unknown`；不能自动生成另一幂等键重发，也不能把超时显示成失败并立即重试。
- 只有原 Star Owner 使用同一请求幂等键查询到的持久结果才能消除未知状态。Owner 尚未提供该查证能力时，受影响写入操作保持禁用。
- 业务重试、取消、恢复、审计和副作用由 Star Owner 处理；BFF 不实现租约、重试队列、fencing 或补偿写事务。

## 4. 环境、构建与密钥

| 维度 | Preview | Production |
|---|---|---|
| 访问保护 | 使用部署平台及宿主认可的访问保护；未信任分支不得获得生产身份 | 仅受保护的生产发布流程 |
| 数据和凭证 | 独立 Preview 凭证和数据；没有 Production Owner 权限、密钥、DB 角色或 Schedule 凭证 | 只配置其实际功能所需的最小凭证 |
| 写入能力 | 对生产 Owner/Outbox/Schedule 的写路径一律拒绝；对独立测试环境的行为须明确标注 | 由 Star 当前授权和 Owner 状态决定 |
| 发布权限 | 无权提升为 Production | 必须记录受保护 promotion 的操作者和审计证据 |
| Cron | 不配置生产触发权限；如 Preview 需演练，只能使用隔离 fixture | best-effort 唤醒，并由 Star Owner 查证、授权和处理 |

构建/发布输入应固定到可审计的 commit、锁文件、构建配置和环境标识。CI 在部署前执行前端 lint/type/test/build 门；TypeScript 错误必须成为阻塞门，不得以 ignoreBuildErrors 或 advisory 状态放行。Rust Phase 2 的二进制、每函数入口、MSRV、包大小和构建根只有通过对应 VCD-2 证据后才纳入 release manifest。

Secrets 通过受控平台环境配置注入，按 Preview/Production 分离并限制可读角色；禁止将 secret 写入源代码、生成物、构建日志、OTel span、URL 或客户端变量。CI 不输出变量值。供应链交付生成 Node 与 Rust 的 SPDX/SBOM，逐项审查完整部署闭包的许可证、版权、NOTICE 和专利义务。

## 5. 条件式 Rust/数据库边界

此节只定义可行性门和架构保护，不授权创建数据库、部署 endpoint 或执行迁移。

### 5.1 VCD-2 最小可行性证明

在目标 Vercel 账户/套餐以隔离 Project 和无敏感测试数据，验证：

- Rust MSRV 1.99.0 是否可用；构建根、依赖上传和两种既有候选构建形态能否访问 `crates/star-api-rest`。
- 最小 handler 与代表性 adapter 依赖闭包的部署结果及实际包体积；以 250MB 标准预算作保守门。
- 函数 cold start、首字节延迟、maxDuration、并发、峰值 RSS、请求体、日志/指标、回滚，以及官方表未确认的 streaming/WebSocket 行为。
- 任何失败不得转为 production Owner 调用、业务写入或执行能力。通用 WebSocket GA 与 Rust 专属能力分别记录。

VCD-2 报告需保存目标计划/region、测试 commit、函数依赖、构建产物摘要、原始运行证据和 UNKNOWN 项。只在本地 Cargo build 成功，不算 Vercel runtime 通过。

### 5.2 Owner、数据库和迁移

FR-VC-009..012 的责任映射限定到已经存在的 Star 授权/业务 Owner、Star 管理的数据面，以及仓库既有 DDD/W/T/M 与迁移治理；此处不创建 Vercel-owned database、第二业务 Owner、独立 migrator 或新的权限权威。下表定义必须复用的责任边界，并记录当前证据状态：

| SRS 需求 | 既有 Star 责任边界 | Vercel 限制 | 当前映射状态 |
|---|---|---|---|
| FR-VC-009 托管 PostgreSQL/数据存储 | 持有相应业务事实的既有 Star Owner 是数据权威；只可使用该 Owner 当前批准并管理的数据存储/API。 | 不得由 Vercel 建立独立业务数据库或绕过 Owner 直连任意 PG。 | 当前 VCD 设计材料没有批准具体 Provider、Owner-to-storage/API 映射或网络/凭证路径；保持 UNKNOWN。 |
| FR-VC-010 事务级租户上下文与 RLS | 当前 Star 授权责任提供经验证的 actor/Project/Run/tenant 范围；相应既有数据 Owner 在同一事务内执行授权后的数据操作与 `SET LOCAL`/RLS。 | Vercel 不自行推导 tenant/Run 权限、不设置会话级上下文、不持有可绕过 Owner 的角色。 | 具体 Owner API、上下文传递及事务/连接池契约尚未在本设计集中获批；保持 UNKNOWN。 |
| FR-VC-011 独立迁移流程 | Schema 变更继续归属 Star 既有 DDD、W/T/M、RLS 和迁移治理；只能使用已存在且经批准的 Star CI/迁移执行流程。 | Function、请求路径和 Vercel Project 均无 DDL 权限、无迁移执行职责；不得新增 Vercel migrator。 | 仓库迁移文件及治理规则存在，但本设计集尚未确定其获批的目标环境执行身份、作业和授权证据；保持 UNKNOWN。 |
| FR-VC-012 连接预算 | 既有 Star 数据 Owner 与其批准的 Provider/数据面共同管理有限连接池及连接总预算。 | Vercel 不自行打开业务数据库连接或建立无界缓存/连接池。 | Provider、池化模式、每实例预算及与 fd/Provider 上限的实测映射尚未确认；保持 UNKNOWN。 |

**Fail-closed 门禁：** 当前没有可据以批准上述具体 Owner/Provider/迁移映射的证据。因此 FR-VC-009..012 不能被解释为允许新建 Vercel 数据面。映射经既有 Owner、授权/数据面负责人及迁移治理显式批准，并补齐 SRS/AC 与运行契约之前，不启用 Vercel 数据库连接、依赖这些连接的 Vercel Function/部署能力或任何执行能力；仅可进行不触达 Star 业务事实的隔离验证。若既有 Star Owner/迁移路径无法满足 SRS FR-VC-009..012，必须先修订 SRS 与验收条件，不得用新 Owner、Vercel 数据库或新迁移权威填补缺口。

在门通过后，所有租户事务仍须以 `SET LOCAL` 建立上下文，并在同一事务中执行、提交/回滚；禁止会话级 `SET` 泄漏到复用连接。连接池总预算与 fd/Provider 上限须按目标环境实测制定，禁止每请求新建连接。Schema 改动不得脱离现有 DDD、W/T/M、RLS 与迁移治理；本设计不新增表名或业务数据模型。

## 6. Cron、Schedule 与执行事实

若将 Vercel Cron 用作可选触发器，处理路径只能是：

    平台 Cron 请求（best-effort）
      -> 受保护、幂等的 Star Schedule/Owner intake（仅在真实契约存在后启用）
      -> Run/Execution owner 持久扫描、复验权限和 run_as_actor_id、登记 owner 事实
      -> owner transactional Outbox
      -> Infrastructure owner 独立核验 RunInfrastructureBinding 并预留 Host/cluster 容量
      -> Star 自有 Runtime/Sandbox 执行

- Cron 丢失、重复、延迟、失败、并发重叠均属预期；Cron 自身无失败重试保证。Owner 以持久未完成工作扫描、幂等键、锁/租约 fencing 和可恢复收据去重及补偿。
- 创建 Rule 时固定的 `run_as_actor_id` 不可变。每次 trigger、retry、resume 都须实时复验该身份当前 Project/Run 授权和 execution capability；撤权、授权目录不可用或授权不完整时拒绝并审计。
- Owner 必须解析当次有效 RunInfrastructureBinding 并固定 cluster identity/generation。Run/Execution owner 事务只负责本 Owner 的 Project/Run 预算、非执行态 TaskExecutionRun 与 owner-local Outbox；它不声称 Host/cluster 资源已预留。
- Infrastructure owner 只在自己的 Inbox/owner command 事务中分别预留 Host 与 cluster 容量；未知/不足时不接受 demand，不启动 executor。两 Owner 不共享写事务、不跨 Owner 写表。
- TaskExecutionRun 在授权、绑定、两级 reservation、目标 K3s API/node/CRI 与独立 Agent Runtime sandbox、原生 Hook、取消/恢复和独立验证门全部满足前维持非执行态。
- Vercel Cron 不能预留资源、创建 executor、访问 Multipass daemon、运行宿主 shell、读写 Worktree 或替 Star 消费 Outbox。缺少明确的 Star intake/API 契约时，不配置该 Cron。
- 任何 Schedule/Rust adapter 的重试语义不允许突破 Star 的预算、幂等、cancel/drain 和审计边界。

## 7. 能力边界与 UI 降级

Vercel 呈现面读取当前可用的 Project、Run 与 Worktree 状态时，必须经过现有 Star 授权契约。它不改变主导航的 Project → Engineering Run → Worktree → Branch 语义，也不把 Vercel Deployment/Preview 当成 Run 或 Branch 真相。

Gateway 连接是明确的安全前置门，不是 Vercel 端已具备的能力。只有真实契约批准并完成验证后，才可定义浏览器握手及各协议的数据流；每次初始连接、重连和受保护数据/操作都必须使用当前 Star 授权，并按契约执行撤权关闭/拒绝。Preview Origin 必须依据契约实行 allow/deny；未定义的 Origin、身份、协议事件或生命周期一律 UNKNOWN 并拒绝连接。不得通过 actor/Run 请求头、自定义身份字段、陈旧连接授权或 Vercel 凭证替代 Star 决策。

chat SSE 的最终部署路由仍未批准：SRS FR-VC-013 允许 Next Route Handler 透传，TD T-049 按经批准的路由进行验证。必须先让 SRS/ADR/BD/DD/TD 与对应的真实 session、授权/撤权、Origin、streaming 和生命周期契约一致；在此之前 Vercel UI 必须显示 unavailable，且不得经 BFF、Function、Gateway 或未定义的替代路径连接。T-049 保持 NOT_RUN。

| 能力 | Vercel Web 处理方式 |
|---|---|
| Star 授权读取 | 仅显示真实 Owner/API 返回且当前用户有权读取的数据；上游不可达则显示不可达状态 |
| Agent、Task CLI、Loop/Workflow | 不在 Vercel 执行；无 Star 执行 Runtime/授权时禁用，清晰说明所需本地运行时 |
| 原生 Hook | 始终由 Rust Star Runtime 决策；浏览器配置或第三方插件不能替代/减弱强制门 |
| Terminal PTY、noVNC、Canvas WSS、ARG WS、chat SSE | 前四类由 Star 自有 Gateway/执行面承载；chat SSE 的最终路由尚未选定。全部在各自真实授权、路由、Origin/跨站防护和生命周期契约批准并验证前标记 unavailable。Vercel 不代理 PTY/CRDT；不得猜测或隐式回退 |
| Canvas 非实时页面 | 可呈现受授权的已持久内容；写入仍由 Canvas/Star Owner 控制 |
| Automation Debug | 依赖本机 Python console 的路由在 Vercel 形态明确不可用，不暴露一个会误报 502 的功能入口 |
| BI / Benchmark | 来源与口径由 Star BI owner 固定；Vercel trace/平台监控只作部署诊断，不直接创建或覆盖业务指标 |
| Plugin | 仅显示授权的 metadata/capability；不得在 Vercel 主进程加载任意 native code 或授予宿主权限 |

所有“尚未由 Owner/provider/runtime 提供”的能力必须明确标记 unavailable。任何页面或 API 不得以空列表、成功 toast、mock runner、预览 adapter 或本地 seed 冒充生产结果。

## 8. 失败、恢复、部署与回滚

| 故障 | 处理与恢复 |
|---|---|
| 必需 build-time 配置缺失或类型门未通过 | 阻止构建；不得发布 mock 配置 |
| BFF 路由、方法、来源、cookie、上游或 body 校验失败 | 拒绝请求，不尝试 alternate upstream |
| Star API/Owner 不可达 | 返回明确 unavailable/timeout；只允许只读的可重试提示，写入未知状态按同一幂等键查证 |
| 会话过期或权限撤销 | 失效该会话/当前能力并重新授权；不得以客户端缓存延续权限 |
| Preview 向生产 Owner 发起写入 | 在隔离边界拒绝；产生隔离测试审计，不向生产发送副作用 |
| Cron 漏发/重复/失败/重叠 | 后续唤醒扫描 Star 持久 due work；通过 Owner idempotency 与 fencing 恢复，不依赖 Cron 自动重试 |
| 回滚切换部署及 Cron 定义 | 回滚后核对目标部署的精确 Cron 定义和项目 Disable Cron Jobs 设置；随后对账 Star Schedule 持久状态，不能仅确认“存在一个 Cron” |
| Vercel Rust Runtime 限制未知/变化 | 对应 capability 保持关闭；重新执行隔离 VCD-2 矩阵，未获证据时 Rust 控制面留在自有运行时 |
| Owner 回执缺失或持久化未知 | 不宣称成功；保持状态未知，使用原 Owner 和原幂等身份恢复 |
| 流量/成本达到实际配置告警 | 按已批准的目标账户预算告警并限制发布；在预算、实例规格未知时不编造数值目标 |

部署流程：锁定 commit 与依赖 -> 运行阻塞 CI 门 -> 构建 immutable artifact -> 验证 env 契约及 SBOM -> 先进入隔离 Preview -> 执行 AC-VC-005/006/013 对应的身份、能力和隔离检查 -> 受保护 promotion -> 观察发布遥测 -> 必要时回滚到已知 artifact 并做 Owner/Schedule 对账。配置/Secret 发布和业务迁移分离；Production Promotion 身份必须可追溯。

回滚是部署恢复，不是业务状态逆转。不得自动倒回 Star Owner 事务、Worktree、任务结果、Outbox 或 Schedule 事实。

## 9. 性能、资源、成本与观测

| 预算/指标 | 设计处理 | 状态 |
|---|---|---|
| 请求延迟 | 仅在目标 Runtime/计划证明后使用 SRS p95 < 2s；函数请求不做长轮询，长作业通过 Owner 持久收据异步完成 | 未实测 |
| 请求体 | 平台载荷不超过 4.5MB；现有 BFF 上限为 2,000,000 bytes；大文件绕过函数 | 约束来源已在 SRS，实施待测 |
| Rust 函数包 | 以每 endpoint 实际依赖闭包测量，先用 250MB 保守标准预算；不假设 5GB Beta 适用于 Rust | 未实测 |
| 内存与并发 | 每 endpoint 记录目标计划、实例规格、峰值 RSS、冷启动与并发；没有 Rust 专属资格证据前不从通用规格推导可用预算 | UNKNOWN |
| 冷启动 | 记录实测分布并回填 NFR-VC-06；在实测和正式定档前不设 SLA | 未实测 |
| 数据库连接 | 每个函数进程显式有限连接预算；平台共享 fd 与 Provider 限制共同核验；不建立每请求连接 | 选型/容量待验证 |
| Telemetry | 用既有 trace/log/BI contract 关联 request 与 Star Owner receipt；Vercel 平台遥测可用于部署/运行诊断；OTLP 外送密钥服务端保管 | 接收端、字段和成本未知 |
| 成本 | 按真实 Active CPU / Provisioned Memory 账单项测量，告警阈值经业务/运行预算批准后配置；禁止在本 DD 造数值门槛 | 未配置 |

观测事件不得包含 access token、cookie、完整敏感请求体、源代码片段或不必要的个人数据。部署健康、请求成功和业务完成是不同指标：业务成功以 Star Owner 持久回执为准，BI 只消费 Owner 生成的既定事实。

## 10. 安全与数据保护

- Production 与 Preview 凭证、数据库角色、访问策略和审计边界分开；Preview 环境变量中不得残留生产 secret。
- 依赖可信的嵌入宿主认证契约和当前 Star Project/Run 授权；不接受未验证代理头、客户端角色字段或 IP allowlist 作为身份。
- 对上游实施固定目标、TLS、路径/方法/来源/长度约束；请求与响应均过滤危险头及重定向。
- cookie/token 仅服务端使用；前端静态资源、浏览器存储、URL、错误文本、console、trace/log 不出现凭证。
- 未知写入结果的查证必须访问同一 Star Owner 和相同幂等键。不得跨 Owner 或环境推测结果。
- Agent、Hook、插件和宿主 shell 权限始终留在 Star 自有隔离 Runtime。Web UI 不直接取得 Multipass、K3s、Worktree 或本地文件权限。
- 发布 SBOM 覆盖全依赖闭包，并审查不限行业/用量商业许可与相应再分发义务；保留许可证、版权、NOTICE、专利声明。

## 11. 需求、验收与验证追踪

此追踪矩阵只引用 SRS v0.4 现有标识；不新增/扩展 SRS 的 FR、NFR、AC 或 GAP。相应测试用例及执行状态见 `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md`。

### 11.1 FR

| SRS ID | 本设计位置 | 验收追踪 |
|---|---|---|
| FR-VC-001 | §2.1–2.2 | AC-VC-001；T-001/T-046 |
| FR-VC-002 | §2.2、§5.1 | AC-VC-007；T-028/T-029；GAP-10 |
| FR-VC-003 | §2.2、§5.2 | AC-VC-008；T-031/T-036 |
| FR-VC-004 | §4、§5.1 | AC-VC-011；T-028/T-030；GAP-01 |
| FR-VC-005 | §3.2、§4 | AC-VC-004；T-003/T-010/T-013 |
| FR-VC-006 | §2.1、§3.2 | AC-VC-002；T-002/T-013 |
| FR-VC-007 | §3.2 | AC-VC-003；T-004..T-008 |
| FR-VC-008 | §3.1–3.2、§10、§2.3/§7 | AC-VC-003/005；T-006/T-009/T-014..T-017/T-049 |
| FR-VC-009 | §5.2 | AC-VC-009/010；T-038/T-039/T-042；GAP-04/06；owner/provider gate OPEN |
| FR-VC-010 | §5.2 | AC-VC-009；T-038/T-042；GAP-04；authorization/transaction mapping OPEN |
| FR-VC-011 | §5.2 | AC-VC-010；T-036/T-039；approved migration-job identity OPEN |
| FR-VC-012 | §5.2、§9 | AC-VC-009；T-038；provider/pool budget OPEN |
| FR-VC-013 | §2.2–2.3、§7、§12 | AC-VC-005/006；T-018/T-034/T-049；GAP-02/09/11；Gateway contract OPEN |
| FR-VC-014 | §6、§8 | T-020..T-027；NFR-VC-07 |
| FR-VC-015 | §1.2、§6 | AC-VC-012；T-027/T-037/T-040/T-041 |
| FR-VC-016 | §1.2、§7 | AC-VC-005/006；T-018/T-019/T-045 |
| FR-VC-017 | §3.1、§10、§2.3/§7 | AC-VC-003/005；T-006/T-017/T-049 |
| FR-VC-018 | §4、§10 | T-002/T-009/T-012/T-035/T-046 |
| FR-VC-019 | §4、§10 | T-012/T-048 |
| FR-VC-020 | §9 | T-035/T-047 |
| FR-VC-021 | §1.2、§4、§10 | AC-VC-013；T-043/T-044 |

### 11.2 NFR, AC and GAP

| SRS ID | 本设计位置 | 测试设计 |
|---|---|---|
| NFR-VC-01 | §9 | TD §6.1; T-032/T-033 |
| NFR-VC-02 | §3.2、§9 | TD §6.1; T-007 |
| NFR-VC-03 | §5.1、§9 | TD §6.1; T-030 |
| NFR-VC-04 | §5.1、§9 | TD §6.1; T-032/T-047 |
| NFR-VC-05 | §5.1、§9 | TD §6.1; T-032 |
| NFR-VC-06 | §5.1、§9 | TD §6.1; T-032 |
| NFR-VC-07 | §6 | TD §6.1; T-020..T-027 |
| NFR-VC-08 | §5.1 | TD §6.1; T-028 |

| SRS acceptance | 本设计位置 | 测试追踪 |
|---|---|---|
| AC-VC-001 | §2、§4、§8 | T-001/T-046 |
| AC-VC-002 | §3.2 | T-002 |
| AC-VC-003 | §3.2、§10 | T-004..T-009/T-013 |
| AC-VC-004 | §3.2、§4 | T-003/T-010 |
| AC-VC-005 | §3.1、§2.3、§7 | T-014..T-017/T-019/T-049 |
| AC-VC-006 | §2.3、§7 | T-011/T-018/T-019/T-045/T-049 |
| AC-VC-007 | §2.2、§5.1 | T-028..T-029 |
| AC-VC-008 | §2.2、§5.2 | T-031/T-036 |
| AC-VC-009 | §5.2 | T-038 |
| AC-VC-010 | §5.2 | T-039 |
| AC-VC-011 | §5.1、§9 | T-030/T-032 |
| AC-VC-012 | §3.3、§6 | T-022/T-027/T-040..T-041 |
| AC-VC-013 | §1.2、§4、§10 | T-043..T-044 |

| SRS gap | 本设计位置 | 后续证据追踪 |
|---|---|---|
| GAP-01 | §5.1、§9 | TD T-030 |
| GAP-02 | §2.2、§5.1 | TD T-034 |
| GAP-03 | §5.1 | TD T-028 |
| GAP-04 | §5.2 | TD T-038/T-042 |
| GAP-05 | §4 | TD T-001 |
| GAP-06 | §5.2、§12 | TD T-038/T-042 and official source review |
| GAP-07 | §4 | TD T-010 |
| GAP-08 | §7 | TD T-011 |
| GAP-09 | §2.2、§5.1 | TD T-034/T-035 |
| GAP-10 | §2.2、§5.1 | TD T-029 |
| GAP-11 | §2.2、§5.1、§9 | TD T-030..T-035 |

## 12. Open decisions and release gates

以下仍是 UNKNOWN/OPEN，不由本设计替代业务或平台决策：

1. 目标 Vercel 账户、套餐、正式域名、地域和访问保护是否具备所需能力。
2. 嵌入宿主的真实 session issuer/audience/cookie contract 以及当前 Run 授权 API 映射。
3. 生产上游 Star API 地址、路径 allowlist 和每个页面实际依赖的 owner-backed read/write 契约。
4. Preview 供应商侧保护和 Promotion 审计的实证方式；禁止访问生产 Owner 的负向证明。
5. Rust Runtime MSRV 1.99.0、Cargo build root、函数包、时长、RSS、并发、streaming/WebSocket、log/metrics 支持矩阵。
6. PostgreSQL/provider 与 RLS 13 类、事务级池化、连接预算和迁移运行流程；不得据第三方文章认定供应商事实。
7. FR-VC-009..012 所需的既有 Star Owner/API/Provider 与授权上下文映射、获批迁移作业身份和连接预算。未有书面批准映射及对应验证证据前，§5.2 的 Vercel DB/函数部署/执行能力维持关闭；若既有路径不能满足 FR，先修订 SRS/AC。
8. TD T-049 所需的真实 Gateway 握手、Preview Origin allow/deny、当前授权/撤权、terminal/noVNC/ARG/Canvas 协议数据流和生命周期证据；chat SSE 则需先批准最终路由（可为 SRS 已允许的 Next Route Handler），并同步 SRS/ADR/BD/DD/TD 与相应真实契约。未完成前相关连接和验收继续关闭/NOT_RUN。
9. Schedule 所需的真实 Star Owner intake、幂等收据查询及 owner-side durable catch-up API。接口未核实前不启用 Cron。
10. SBOM/许可证流水线、OTLP 接收端、成本告警预算和数据保留策略。

当前证据只足以定义设计边界，不足以批准上线。VCD-0 需完成 ADR/基本/详细/测试设计的横向一致性评审；VCD-1 以受保护 Web Preview 和真实会话/授权为目标；VCD-2 是隔离 Rust runtime 可行性证明；VCD-3 仅可在 VCD-2 PASS 且 WLD-0/WLD-6 门通过后实施；VCD-4 验证回滚、Schedule 对账、成本与运行手册。任何未达到其门槛的能力继续关闭。

## 修订历史

| 版本 | 日期 | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses — Mavis 接手审核 | 基于 SRS v0.4 和 VCD-0/DD/TD 兼容性报告建立单一 Vercel Web/BFF 详细设计，明确 Star Owner/执行权威、隔离环境、条件式 Rust 证明、Cron 边界、失败恢复和追踪；无实现或部署声明。 |
| v0.2 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按独立 DD R2 补齐 FR-VC-009..012 到既有 Star Owner/数据面/迁移治理的条件式责任映射及未获批准时的 fail-closed 门禁；将 T-049 Gateway 握手、Preview Origin、当前授权/撤权、协议数据流/lifecycle 与 chat SSE 路由差异纳入架构、能力边界、追踪和开放门禁。保持 Draft、VCD-0 OPEN，所有执行/部署/运行验证 NOT_RUN；未发明协议或映射。 |
| v0.3 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对齐 T-049 的五类浏览器连接（含 noVNC），并澄清 SRS 允许 chat SSE 使用 Next Route Handler、测试按批准路由执行；最终路由及其真实 session/授权/Origin/streaming/lifecycle 契约仍未批准，故 VCD-0 和 T-049 保持 OPEN/NOT_RUN。未推定 Gateway wire contract，未实施或部署。 |
