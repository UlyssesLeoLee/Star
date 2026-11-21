# F15 收件箱、通知与即时通信

> 状态摘要: `/inbox` 已接入真实通知 REST（GET/PATCH `/api/v1/notifications`），并对未接入、未授权、失败与空态诚实呈现；但 REST 通知与 IM 存储为进程内存，没有业务事件产生通知，接口无鉴权与用户隔离，领域 crate `domain-notification` 无生产调用方、无渠道发送与重试。用户间私聊与群聊目前只在浏览器本地运行，后端 IM REST 与 IM1.0 网关客户端均未与之接通，仓内没有网关服务端。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 把站内通知（评审请求、评论、提及、系统）汇入 `/inbox`，支持类别查看、详情、已读与未读切换（FR-15-01 至 FR-15-04）。
- 提供通知与评论的 REST 契约，并在领域层给出投递、已读、降噪与模板维护规则（`docs/specs/domain-notification-spec.md` §1、§3）。
- 提供用户间私聊与群聊。网关不可达时，界面必须明确标注“本地模式”，不得冒充实时送达（FR-15-26）。

### 1.2 范围

| 类别 | 内容 |
|---|---|
| 范围内 | `/inbox` 页面及其 real-mode 数据通道；`notificationsApi` 客户端与通知类型；`routes/notifications.rs` 与 `routes/im.rs`；`crates/domain-notification`；右侧栏的即时通信面板 `DmCenterPanel`、`dmStore`、`imGateway` 客户端；6 个旧入口到 `/inbox` 的重定向；收件箱 MSW mock |
| 范围外 | 右侧 Agent 对话栏 `chatStore` 与 `SideChatPanel`，归属 F02（`docs/features/F02-branch-workspace.md` FR-02 相关行及第 197、210 行）；群组页“AI 更新”页签，归属 F02（FR-02-08）与 F01（FR-01-26）；Email、Slack、钉钉渠道的实际发送（spec §1 与 §30.3 的分期，本仓无实现）；原生操作系统通知（spec 标注为 P1 后续，见 FR-15-11）；审计、反馈、搜索、上下文各自的页面内容，这些入口仅在本功能中做重定向，页面功能归其所属功能，本文不逐一核对 |

## 2. 用户与场景

| 角色 | 主要场景 | 代码依据 |
|---|---|---|
| 收件人（登录用户） | 查看收件箱列表与详情，标为已读或未读 | `frontend/src/app/(app)/inbox/page.tsx:89-132`；`crates/star-api-rest/src/routes/notifications.rs:635-662` |
| 评论作者 | 对目标发表评论，删除本人评论 | `crates/star-api-rest/src/routes/notifications.rs:701-752` |
| 即时通信用户 | 新建私聊或群聊，发送消息，查看未读数与已读回执 | `frontend/src/components/chat/DmCenterPanel.tsx:191-202`；`frontend/src/lib/chat/dmStore.ts:178-325` |
| 租户或项目管理员 | 维护通知模板（INV-N-04） | `crates/domain-notification/src/lib.rs:679-682`（仅领域层） |
| 事件源 | 评审请求、评论、校验失败等事件触发通知 | 未接通，见 FR-15-09 |

主要用户故事：S1 收件人打开收件箱并按类别查看；S2 收件人标记已读后，列表与计数同步变化；S3 收件人打开一条带外部链接的通知；S4 评论作者发表与删除评论；S5 用户展开右侧栏即时通信面板，与他人私聊或建群；S6 系统在评审请求等节点为收件人生成通知。S6 目前不能端到端成立。

## 3. 需求

优先级沿用 SRS-INBOX-001 的 FR-IN-001 至 FR-IN-004 与 domain-notification 的 P0/P1 划分，IM 需求按本功能的用户价值划分。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-15-01 | 收件箱列表与详情：按创建时间倒序，选中条目显示标题、正文、发送者与时间 | P0 | ✅ | `frontend/src/app/(app)/inbox/page.tsx:235-267`、`270-317`；`frontend/src/lib/notifications/notificationsApi.ts:159-166`；`crates/star-api-rest/src/routes/notifications.rs:533`、`512-549` |
| FR-15-02 | 类别过滤：全部、评审请求、评论、通知 | P0 | 🟡 | 分类由前端正则判定（`page.tsx:51-56`、`143-145`），未使用后端 `kind` 过滤；后端支持 `kind` 参数（`notifications.rs:516-519`）。标签与设计稿一致（`docs/prototypes/3 · 收件箱-html/Inbox.dc.html:77`） |
| FR-15-03 | 标为已读与未读，乐观更新，失败回滚 | P0 | ✅ | `page.tsx:119-132`、`297-310`；`notificationsApi.ts:168-182`；`notifications.rs:635-662` |
| FR-15-04 | 未接入、未授权、失败、空态的诚实呈现，不回落到 mock 数据 | P0 | ✅ | `page.tsx:89-168`、`209-230`；`frontend/src/components/status/StatusNotice.tsx:140-179`、`216-225` |
| FR-15-05 | 在 Branch 中打开：跳转到通知关联的对象 | P1 | 🟡 | 仅当 `subject_url` 存在时渲染外链按钮，并以新标签页打开（`page.tsx:286-296`）；没有到 Branch 工作区的应用内路由映射 |
| FR-15-06 | 未读计数：页头与侧栏 | P1 | 🟡 | 页头计数由已加载列表本地求得（`page.tsx:146`、`178-185`）；未读计数端点存在（`notifications.rs:555-567`），前端 `fetchUnreadCount` 无调用方（`notificationsApi.ts:184-189`）；侧栏计数为硬编码 `count: 5`（`frontend/src/lib/nav/registry.ts:170-181`，第 180 行） |
| FR-15-07 | 六个旧入口统一进入收件箱：notification、comment、audit、feedback、search、context | P1 | 🟡 | 重定向规则存在（`frontend/src/lib/redirects.ts:105-120`；`redirects.shim.cjs:54-59`；`next.config.js:39-44`）；收件箱页面不读取 `type` 参数，分类过滤未生效（`page.tsx` 无 `searchParams` 读取） |
| FR-15-08 | 通知分类与后端枚举一致：review_request、comment、mention、system | P1 | 🟡 | 后端枚举四值（`notifications.rs:100-112`）；前端 `kind` 类型宽容为 `string`（`frontend/src/types/notifications.ts:39-56`）；页面以文本规则分类（`page.tsx:51-56`） |
| FR-15-09 | 业务事件触发通知：评审请求、评论、提及、系统事件 | P0 | ⬜ | REST 只提供手工 `POST`（`notifications.rs:574-626`）；领域 `dispatch` 无调用方（见 §4.6）；spec §5 的订阅者未接通 |
| FR-15-10 | 默认降噪（INV-N-07）：突破事件必送，其余事件默认抑制 | P0 | ✅ | 分类 `crates/domain-notification/src/lib.rs:55-96`、`124-142`；抑制在 `dispatch` 中执行（`lib.rs:727-731`）；测试 `tests/integration_test.rs:262`、`280`。生产路径未调用，见 FR-15-09 |
| FR-15-11 | FR-ORCA-040：Agent 完成与需要用户介入为突破事件 | P1 | ✅ | `crates/domain-notification/src/lib.rs:73-75`、`109-110`、`134-135`；测试 `tests/integration_test.rs:301`。原生操作系统通知适配为 ⬜（spec §6.3 标注为后续） |
| FR-15-12 | FR-ORCA-041：未读判定与侧栏加粗判据 | P1 | 🟡 | 判据 `lib.rs:334-347`，测试 `integration_test.rs:329`；侧栏渲染加粗未见（`frontend/src/components/Sidebar.tsx:250-257` 仅渲染数字徽标） |
| FR-15-13 | 渠道注册与启停（Email、InApp、Slack、DingTalk，本人操作） | P1 | 🟡 | 渠道类型四值（`lib.rs:212-221`）；端口仅有 `register_channel`（`lib.rs:481`、`634-665`），无更新与删除端口（`lib.rs:479-508`）；无渠道 REST 端点 |
| FR-15-14 | 模板维护仅限 Project 或 Tenant 管理员（INV-N-04） | P2 | ✅ | `lib.rs:679-682`；测试 `integration_test.rs:394`。无 REST 或界面入口 |
| FR-15-15 | 渠道实际发送：Email、Slack、钉钉、站内推送 | P1 | ⬜ | `dispatch` 直接写入 `Sent` 状态（`lib.rs:767`），无任何渠道发送实现；前端无站内推送通道 |
| FR-15-16 | 异步投递，不阻塞业务事务（INV-N-02） | P1 | ⬜ | `dispatch` 为同步写入（`lib.rs:715-779`）；无 worker 与消息总线接入 |
| FR-15-17 | 失败指数退避重试，最多 5 次，超限进入死信（INV-N-06） | P1 | 🟡 | `MaxRetryExceeded` 变体与 `MAX_RETRY` 常量存在（`lib.rs:373-374`、`630`）；无重试循环，`retry_count` 恒为 0（`lib.rs:771`）；`DeadLettered` 状态存在（`lib.rs:306-307`）但无写入路径 |
| FR-15-18 | 通知标题与正文不可修改（INV-N-05） | P1 | ✅ | `mark_read` 只改 `status` 与 `read_at`（`lib.rs:820-821`）；REST `PATCH` 只改 `read`（`notifications.rs:642`） |
| FR-15-19 | 租户隔离（INV-N-01） | P0 | 🟡 | 领域层校验租户（`lib.rs:720-725`、`786-791`）；REST 列表的 `tenant_id` 为可选参数，缺省时返回所有租户的通知（`notifications.rs:527-529`） |
| FR-15-20 | 通知与渠道仅本人可读与改（INV-N-03） | P0 | 🟡 | 领域 `mark_read` 校验用户（`lib.rs:811-813`）；REST 列表、计数与标记均无用户校验（`notifications.rs:512-515`、`555-558`、`635-662`） |
| FR-15-21 | 评论列表、新增、删除，仅作者可删除本人评论 | P1 | ✅ | 路由 `notifications.rs:246-248`；处理函数 `669-693`、`701-748`，所有权检查 `751-752`；测试 `tests/notifications_endpoints.rs:504`、`593`。前端客户端无调用方（见 §6） |
| FR-15-22 | 评论发表时生成评论类通知 | P2 | ⬜ | 文件头明确未联动（`notifications.rs:59-60`） |
| FR-15-23 | 私聊与群聊：同一成员集合幂等复用；一名对方为私聊，两名及以上为群聊 | P0 | 🟡 | 前端本地实现（`frontend/src/lib/chat/dmStore.ts:178-209`）；后端 `create_conversation` 具有同样规则与幂等（`crates/star-api-rest/src/routes/im.rs:450-483`），前端未调用 |
| FR-15-24 | 消息发送、会话内序号、本端未读与已读回执 | P0 | 🟡 | 前端本地实现（`dmStore.ts:211-269`、`304-325`；`DmCenterPanel.tsx:297-392`）；后端 `send_message`、`list_messages`、`mark_read` 存在（`im.rs:604`、`694`、`750`），前端未调用 |
| FR-15-25 | 会话与消息跨用户、跨设备共享 | P0 | ⬜ | 前端仅写入浏览器 `localStorage`（`dmStore.ts:439-466`）；后端为进程内存（`im.rs:59-61`） |
| FR-15-26 | 诚实的实时状态：未接入时标注“本地模式”，消息不冒充实时送达 | P0 | ✅ | `DmCenterPanel.tsx:170-186`、`392`；`dmStore.ts:157-166`、`335-364` |
| FR-15-27 | IM1.0 网关实时收发：认证、确认、重连、心跳、断线队列 | P1 | 🟡 | 客户端已实现（`frontend/src/lib/chat/imGateway.ts:239-511`，状态 `146`，重连 `300-318`，队列 `249`）；默认地址 `ws://localhost:9001`（`imGateway.ts:187`）；仓内没有 WebSocket 服务端（见 §6） |
| FR-15-28 | 群聊的群主角色与成员上限 | P2 | 🟡 | 后端实现群主角色与 500 人上限（`im.rs:94-95`、`443-448`、`497-500`）；前端无成员角色展示 |
| FR-15-29 | 在线成员（presence） | P2 | 🟡 | 接口为占位，返回 `connected: false` 与空成员（`im.rs:817-822`） |
| FR-15-30 | 消息类型：image、file、sticker、custom | P2 | ⬜ | 后端只接受 `text`（`im.rs:612-617`）；前端类型定义存在（`imGateway.ts:30-56`），但发送路径只发文本（`dmStore.ts:246-255`） |
| FR-15-31 | 通知与 IM 接口的身份认证（JWT 或等效机制） | P0 | ⬜ | 通知与 IM 路由挂在 supplemental 路由上，无鉴权层（`crates/star-api-rest/src/lib.rs:165-183`；`main.rs:155-166`）；身份取自 `X-User-Id` 请求头（`notifications.rs:313-325`；`im.rs:66-68`） |
| FR-15-32 | 即时通信的当前用户来自登录态 | P0 | 🟡 | 当前用户为常量 `usr-001`（`dmStore.ts:102`），可由 `setCurrentUserId` 修改（`dmStore.ts:333`）；无登录态注入 |
| FR-15-33 | 收件箱经 real-mode 访问真实 API，并注入 Bearer 凭证 | P0 | ✅ | `frontend/src/mocks/real-mode.ts:22-50`、`58-69`；`notificationsApi.ts:76-83`、`96-130` |
| FR-15-34 | 通知列表分页：按 `limit` 与 `offset` 取页 | P1 | 🟡 | 后端支持 `limit` 与 `offset`，缺省 20，上限 200（`notifications.rs:92-94`、`521-522`）；页面固定 `limit: 200`，无分页界面（`page.tsx:100-101`） |
| FR-15-35 | 通知模板的内容渲染与 XSS 防护 | P2 | ⬜ | spec §11 列为风险；仓内无模板引擎实现 |
| FR-15-36 | spec 中的端口方法 `update_channel`、`delete_channel`、`mark_all_read` 与错误码 N-001 至 N-005 | P2 | ⬜ | 端口中不存在这些方法（`lib.rs:479-508`）；对 `N-00` 与 `notification:` 的全仓检索无匹配 |

## 4. 设计

### 4.1 概念模型

| 概念 | 含义 | 当前代码的形态 |
|---|---|---|
| 收件箱条目（Notification，线格式） | 面向收件人的站内消息 | 前端类型 `types/notifications.ts:39-56`，REST 结构 `notifications.rs:128-154` |
| 评论（Comment） | 挂在目标 `target_type` 与 `target_id` 上的文本 | `types/notifications.ts:58-71`；`notifications.rs:158-176` |
| 领域通知（Notification，领域层） | 含 `event_type`、`channel_id`、六态状态与重试计数的实体 | `crates/domain-notification/src/lib.rs:262-307`；与线格式不是同一实体，线格式没有 `status` 与 `event_type` |
| 通知渠道与模板 | 渠道（`kind`、`address`、`enabled`）与模板（`event_type`、`channel_kinds`、`body_template`） | `lib.rs:191-255` |
| 私聊与群聊会话 | 前端 `DmConversation` 与后端 `Conversation`、`Member`、`Message` | `dmStore.ts:53-66`；`im.rs:103-150` |
| IM 网关帧 | 客户端 8 类帧、服务端 11 类帧，按 `type` 判别 | `imGateway.ts:72-80`、`117-145` |

### 4.2 数据模型

| 实体 | 存储位置 | 位置 | 说明 |
|---|---|---|---|
| 通知（线格式） | 进程内存 `HashMap` | `notifications.rs:188-193` | 字段 `id`、`tenant_id`、`kind`、`subject`、`body`、`subject_url`、`actor_id`、`actor_name`、`target_type`、`target_id`、`read`、`created_at` |
| 评论 | 进程内存 | `notifications.rs:158-176`、`192` | 字段 `id`、`tenant_id`、`target_type`、`target_id`、`author_id`、`author_name`、`body`、`created_at` |
| 领域通知、渠道、模板 | 进程内存 `RwLock` 与 `InMemoryNotificationRepository` | `lib.rs:595-599`、`893-896` | 无持久化实现；`NotificationRepository` 只有内存实现（`lib.rs:917`） |
| IM 会话、成员、消息 | 进程内存 | `im.rs:59-61`、`141-150` | 消息序号按会话自增（`im.rs:602-603`） |
| 前端私聊会话与消息 | 浏览器 `localStorage`，键 `star-dm-conversations` | `dmStore.ts:439-466` | 重载后 `realtimeConnected` 一律回落为 false（`dmStore.ts:452-465`） |
| 收件箱 mock 条目 | 测试与开发用固定数据 | `frontend/src/mocks/data/inbox.ts:15-26`；`schemas/inbox.ts:4-26` | 七种 kind，与线格式的四种 kind 不一致 |
| 数据库表 | 无 | `db/migrations/2026-09-10-p3d6-13-tables.sql` | 迁移文件中只有 `canvas_comments`（第 1188 行）；没有 notification 或 IM 相关表，spec §2 与 data-design §4.15 的 `notification` schema 无对应 DDL |

### 4.3 接口

#### REST（挂载前缀 `/api/v1`）

| 方法与路径 | 处理函数 | 状态 | 证据 |
|---|---|---|---|
| `GET /notifications` | `list_notifications` | ✅ 已挂载，内存存储 | `notifications.rs:230`、`512-549`；`lib.rs:174` |
| `GET /notifications/unread-count` | `unread_count` | ✅ 已挂载 | `notifications.rs:231`、`555-567` |
| `POST /notifications` | `create_notification` | ✅ 已挂载，仅手工创建 | `notifications.rs:232`、`574-626` |
| `PATCH /notifications/{id}` | `patch_notification_read` | ✅ 已挂载 | `notifications.rs:233`、`635-646` |
| `POST /notifications/{id}/read` | `mark_notification_read` | ✅ 已挂载，幂等 | `notifications.rs:234`、`652-662` |
| `GET /comments`、`POST /comments` | `list_comments`、`create_comment` | ✅ 已挂载 | `notifications.rs:246-247`、`669-742`；`lib.rs:175` |
| `DELETE /comments/{id}` | `delete_comment` | ✅ 已挂载，仅作者可删 | `notifications.rs:248`、`743-752` |
| `GET`、`POST /im/conversations` | `create_conversation`、`list_conversations` | ✅ 已挂载，内存存储 | `im.rs:250-251`、`420-593`；`lib.rs:173` |
| `GET`、`POST /im/conversations/{id}/messages` | `list_messages`、`send_message` | ✅ 已挂载，内存存储 | `im.rs:252-258`、`604`、`694` |
| `POST /im/conversations/{id}/read` | `mark_read` | ✅ 已挂载 | `im.rs:260`、`750` |
| `GET /im/presence` | `presence` | 🟡 占位，返回 `connected: false` | `im.rs:261`、`817-822` |

仓内没有单条通知的 `GET`、渠道端点、模板端点、死信端点、SSE 与 WebSocket 推送端点。

#### 前端客户端（`frontend/src/lib/notifications/notificationsApi.ts`）

| 函数 | 调用的 REST | 调用方 | 状态 |
|---|---|---|---|
| `listNotifications`（`:159-166`） | `GET /notifications` | `inbox/page.tsx:101` | ✅ |
| `markRead`、`markUnread`（`:168-182`） | `PATCH /notifications/{id}` | `inbox/page.tsx:124` | ✅ |
| `fetchUnreadCount`（`:184-189`） | `GET /notifications/unread-count` | 无 | ⬜ 无调用方 |
| `listComments`、`addComment`、`deleteComment`（`:198-226`） | `/comments` 系列 | 无 | ⬜ 无调用方 |

#### 其他接口

- Next 旧接口：`frontend/src/app/api/notifications/route.ts:21-31` 与 `[id]/route.ts:26-32` 返回 410 Gone，文案指向真实后端。状态 🗑（路径仍在，行为为退役）。
- MSW mock：`frontend/src/mocks/handlers/inbox.ts:16-23` 注册 `GET /api/notifications` 与 `PATCH /api/notifications/:id`，经 `frontend/src/mocks/handlers/index.ts:36`、`59` 挂入。路径与页面调用的 `/api/v1/notifications` 不一致。
- 领域端口：`NotificationCommandPort` 含 `register_channel`、`upsert_template`、`dispatch`、`mark_read`（`lib.rs:479-508`）；`NotificationQueryPort` 为查询端口（`lib.rs:511-527`）。两者均无生产调用方。
- MCP 资源：`crates/star-mcp/src/handlers/notification.rs:28-37` 对任意 `notification://{id}` 返回固定值，文件头注明 TODO（第 7 行）。状态 🟡。
- 网关帧：客户端 8 类发送帧（`imGateway.ts:72-80`），服务端 11 类接收帧（`imGateway.ts:117-145`）。

### 4.4 状态机与关键流程

**领域通知状态**（`lib.rs:295-307`）：`Pending`、`Sent`、`Delivered`、`Read`、`Failed`、`DeadLettered`。实际写入只有两条路径：`dispatch` 直接写入 `Sent`（`lib.rs:767`），`mark_read` 写入 `Read` 并校验终态（`lib.rs:814-821`）。`Pending`、`Delivered`、`Failed`、`DeadLettered` 没有写入路径。REST 线格式只有 `read` 布尔值（`notifications.rs:642`、`658`）。

**收件箱加载状态**（`page.tsx:89-168`）：`loading` → 若 real-mode 未开启，直接进入 `disconnected`（`94-98`、`153-155`）；若请求成功，进入 `ready` 或 `empty`（`166-167`）；若请求失败，按 `inferNoticeState` 映射为 `unauthorized`、`disconnected` 或 `failed`（`157-164`）。

**已读切换**（`page.tsx:119-132`）：乐观更新；发送 `PATCH`；成功后以响应中的 `read` 对齐；失败时回滚并设置 `toggleErrorId`，显示失败提示（`306-310`）。

**领域投递**（`lib.rs:715-779`）：校验租户（`720-725`）→ 降噪判断（`726-731`）→ 选取用户已启用渠道，若无则临时创建站内渠道（`733-755`）→ 写入 `Sent` 状态（`757-778`）。没有异步化、没有重试、没有死信。

**私聊发送**（`dmStore.ts:211-269`）：在本地插入消息 → 若网关对象存在且已连通，则按接口上行 → 上行失败时调用 `markRealtimeDown(false)`，把全局与会话的实时标志一并置为 false（`157-166`）。

**网关连接**（`imGateway.ts`）：状态 `connecting`、`connected`、`reconnecting`、`offline`（`146`）；断线后指数退避，1s 至 30s 上限（测试 `frontend/src/lib/chat/imGateway.test.ts:248`）；心跳 25 秒（测试 `:339` 起）。`connectImGateway` 创建即连接（`imGateway.ts:507-511`）。

**即时通信面板挂载**（`DmCenterPanel.tsx:69-80`）：挂载时调用 `tryLoadImGateway`，卸载时调用 `bindImGateway(null)`。面板只在右侧栏打开并展开“即时通信”后才渲染（见 §4.7 补充）。

### 4.5 权限与安全

| 项 | 现状 | 证据 | 状态 |
|---|---|---|---|
| 通知列表与未读计数的身份 | 不读取任何身份，`tenant_id` 可选，缺省返回全部租户 | `notifications.rs:512-515`、`527-529`、`555-563` | ⬜ |
| 通知标记已读的身份 | 不校验调用者，任何人可按 id 修改任意通知的 `read` | `notifications.rs:635-662` | ⬜ |
| 评论作者与删除权限 | 以 `X-User-Id` 请求头派生身份，可被伪造 | `notifications.rs:62-64`、`313-325`、`706`、`751-752` | 🟡 |
| IM 身份与成员校验 | 以 `X-User-Id` 派生；非成员的发送与标记被拒绝 | `im.rs:66-68`、`425`；测试 `tests/im_endpoints.rs:319` | 🟡 |
| 路由层鉴权 | 旧版 `build_router` 挂有 `auth_layer_stub`，supplemental 路由与生产合并路由均无鉴权层 | `lib.rs:138`、`165-183`；`main.rs:155-166` | ⬜ |
| 前端凭证 | Bearer 取自 `localStorage.real_api_key`，或构建期 `NEXT_PUBLIC_API_KEY` | `real-mode.ts:62-64`、`44-45` | 🟡 构建期变量会进入客户端包 |
| 领域层租户与用户隔离 | 投递与标记均校验租户，标记校验用户 | `lib.rs:720-725`、`786-813` | ✅ |
| 模板维护 | 仅 `project_admin` 或 `tenant_admin` 角色 | `lib.rs:679-682` | ✅ |
| 错误映射 | 401 与 403 映射为 `unauthorized`，界面显示无权限态 | `notificationsApi.ts:107-113`；`StatusNotice.tsx:95-102` | ✅ |

### 4.6 依赖关系

| 依赖方 → 被依赖方 | 形式 | 证据 | 状态 |
|---|---|---|---|
| `InboxPage` → `notificationsApi` → `realFetch` → 后端 | 直接 import 与 fetch | `page.tsx:32-38`；`real-mode.ts:58-69` | ✅ |
| `InboxPage` → `StatusNotice`、i18n | 组件与 hook | `page.tsx:42-45` | ✅ |
| `RightChatRail` → `DmCenterPanel` | 组件渲染 | `frontend/src/components/chat/RightChatRail.tsx:26`、`215` | ✅ |
| `DmCenterPanel` → `dmStore` | Zustand hook | `DmCenterPanel.tsx:25-29`、`51-61` | ✅ |
| `dmStore` → 静态用户目录 | seed 数据 | `dmStore.ts:29`、`105-107` | 🟡 用户列表为本地静态数据 |
| `dmStore` → `imGateway` | 动态 import，失败回退本地 | `dmStore.ts:371`、`379` | 🟡 客户端在，服务端不在仓内 |
| `notifications.rs` / `im.rs` → `domain-notification` | 无；两者自建内存态，文件头明确不复用领域 crate | `notifications.rs:47-50`；`im.rs:45-47` | ⬜ |
| 业务代码 → `domain-notification` | 无源码引用：`domain-work-item/Cargo.toml:27` 仅有依赖声明；`domain-form/src/lib.rs:220-221` 为注释；`star-dispatcher/src/sa_real_impls.rs:258` 为字符串 | 见左 | ❓ 依赖声明是否在源码中调用，未确认 |
| `event-bus` 的 `NotificationConsumer` → 领域通知 | 无连接证据；消费者注释列出的事件为 `RiskDetected`、`HealthChanged` | `crates/event-bus/src/consumer.rs:1-14`、`44` | ❓ |
| `star-mcp` 通知资源 → 领域通知 | 未接通，返回固定值 | `crates/star-mcp/src/handlers/notification.rs:7`、`28-37` | 🟡 |
| 重定向 → `/inbox` | 配置级 307 重定向 | `redirects.ts:105-120`；`next.config.js:39-44` | ✅ |
| `app/page.tsx` → `/inbox` | 根路径重定向 | `frontend/src/app/page.tsx:32` | ✅ |
| 外部 IM1.0 网关服务端 | 不在本仓库 | 仓内 `crates/` 检索 `WebSocketUpgrade` 与 `axum::extract::ws` 只命中 `api`、`canvas-realtime`、`terminal-stack`，`star-api-rest` 与 `star-mcp` 中无命中 | ❓ 需接入外部仓库后核对端口与路径 |

### 4.7 页面设计

#### 4.7.1 /inbox

- 文件：`frontend/src/app/(app)/inbox/page.tsx`
- 布局：
  - 顶栏：`AppHeader`（`frontend/src/components/AppShell.tsx:49`）。
  - 侧栏：全局 `Sidebar`（`frontend/src/app/layout.tsx:78`），收件箱条目来自 `registry.ts:170-181`，计数徽标由 `Sidebar.tsx:250-257` 渲染。
  - 主区：`AppShell` 的 `<main>`（`AppShell.tsx:57-73`）。`/inbox` 属于全高页面（`frontend/src/app/(app)/layout.tsx:34`，`45-47`），主区采用 `overflow-y-hidden` 的 fullBleed 样式（`AppShell.tsx:64-66`）。
  - 右侧栏：`RightChatRail`（`AppShell.tsx:77`），默认收起（`RightChatRail.tsx:53`）。
  - 底栏：本页无独立底栏。
  - 页内两栏：列表 `w-96`（`page.tsx:235`），详情 `aside`（`page.tsx:270`）。
- 组件树：
  - `InboxPage`（`page.tsx:80`）
  - 页头：标题与未读计数（`page.tsx:173-187`）
  - 分类标签（`page.tsx:190-207`）
  - 状态区：`StatusNotice`（`page.tsx:213`，组件定义于 `StatusNotice.tsx:140`），错误详情（`page.tsx:214-218`），重试按钮（`page.tsx:219-228`）
  - 列表 `ul`（`page.tsx:235-267`）
  - 详情区（`page.tsx:270-317`）：类别与时间（`272-282`）、标题与正文（`283-284`）、打开外链（`285-296`）、已读切换（`297-304`）、切换失败提示（`306-310`）、空详情（`312-316`）
  - 辅助函数：`categoryOf`（`51-56`）、`relativeTime`（`64-76`）、`errText`（`58-61`）
- 状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 列表加载成功且非空 | 列表与详情，页头显示未读数（`page.tsx:209-317`） | ✅ |
| 加载 | 进入页面或点击重试（`page.tsx:151-152`） | `StatusNotice` 加载态，文案来自 i18n（`page.tsx:152`） | ✅ |
| 空 | 成功但列表为空（`page.tsx:166-167`） | `StatusNotice` 空态 | ✅ |
| 错误 | 请求失败，且不属于下列两类（`page.tsx:162-164`） | 失败提示、错误详情、重试按钮（`page.tsx:213-228`） | ✅ |
| 未授权 | 401 或 403（`notificationsApi.ts:107-113`；`page.tsx:158-159`） | 无权限提示，不显示列表 | ✅ |
| 未接入 | real-mode 未开启（`page.tsx:94-98`、`153-155`） | 未接入提示，文案说明需开启 real-mode，不显示 mock 数据 | ✅ |
| 禁用 | 无对应分支 | 标为已读按钮与分类标签无禁用态（`page.tsx:297-304`、`190-207`） | ⬜ |

- 交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 打开页面 | real-mode 已开启 | 拉取最多 200 条 | `GET /api/v1/notifications?limit=200` | ✅ |
| 切换分类 | 列表已加载 | 本地过滤 | 无请求 | 🟡 分类依据为文本规则（FR-15-02） |
| 选中条目 | 列表非空 | 显示详情 | 无请求 | ✅ |
| 标为已读或未读 | 已选中条目 | 乐观更新，以响应对齐，失败回滚并提示 | `PATCH /api/v1/notifications/{id}` | ✅ |
| 在 Branch 中打开 | `subject_url` 非空 | 新标签页打开外链 | 外链 | 🟡 无应用内 Branch 路由（FR-15-05） |
| 重试 | 错误态 | 重新加载 | `GET /api/v1/notifications` | ✅ |
| 全部标为已读 | 无 | 无入口 | 无端点 | ⬜ |
| 翻页或加载更多 | 无 | 无入口，固定取 200 条 | 无 | ⬜ |
| 访问旧入口（notification、comment 等） | 访问旧路径 | 重定向到 `/inbox`，`type` 参数被忽略 | 配置级重定向 | 🟡（FR-15-07） |

- 数据：来源为 REST，经 `notificationsApi.listNotifications`。页面使用的字段为 `id`、`read`、`subject`、`body`、`created_at`、`actor_name`、`actor_id`、`subject_url`，以及用于分类的 `kind`。`tenant_id`、`target_type`、`target_id` 未被使用。MSW mock 不服务此路径（见 §5）。

- 与文档设计的差距：
  - 分类为前端文本规则，与后端 `kind` 不一致（`page.tsx:51-56`；FR-15-02、FR-15-08）。
  - 六个旧入口的 `type` 参数未被消费（`page.tsx` 无 `searchParams`；`redirects.ts:106-116`；FR-15-07）。
  - 在 Branch 中打开为外链，没有应用内映射（`page.tsx:286-296`；FR-15-05）。
  - 页头未读数只统计已加载列表，侧栏计数为硬编码 5（`page.tsx:146`；`registry.ts:180`；FR-15-06）。
  - SRS-INBOX-001 §1 设计要点列出左侧项目折叠列表（Physis、StarGate、Mobile）。页面本身不渲染项目列表（`page.tsx` 全文）；全局侧栏是否承担该列表未确认（❓）。
  - 列表只能由手工 `POST` 产生，生产环境没有通知来源（FR-15-09）。
  - 接口无身份过滤，列表包含所有租户的数据（FR-15-19、FR-15-31）。

**补充：右侧栏即时通信面板（非路由）**

- 挂载位置：`RightChatRail.tsx:200-219`。需同时满足右侧栏打开（`:53`、`:58` 从 `localStorage` 恢复）与“即时通信”展开（`:36`、`:205-206`），面板才渲染（`:213-215`）。
- 结构：状态条（`DmCenterPanel.tsx:170-186`，文案 `:174-176`）、会话列表（`:202`）、消息区（`:297`）、输入区（`:373-392`）、新建会话弹窗（`:65-66`、`:137`）。
- 状态：实时已连接与本地模式两态（`:186`）；本地模式下输入区附注说明（`:392`）。
- 与文档设计的差距：会话与消息只存在于浏览器（FR-15-25）；后端 IM 接口未被调用（FR-15-23、FR-15-24）；网关服务端缺失，展开面板即触发网关连接尝试并按退避重连（FR-15-27，`imGateway.ts:300-318`）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 收件箱页面 | `frontend/src/app/(app)/inbox/page.tsx` | 🟡 | 真实接入、三态、已读切换为 ✅（`:89-168`、`:119-132`）；分类与外链为 🟡（`:51-56`、`:286-296`） |
| 通知客户端 | `frontend/src/lib/notifications/notificationsApi.ts` | 🟡 | 列表与已读切换 ✅；计数与评论客户端无调用方（`:184-226`） |
| 通知类型 | `frontend/src/types/notifications.ts` | ✅ | `:27-86` |
| real-mode 通道 | `frontend/src/mocks/real-mode.ts` | ✅ | `:22-69` |
| 状态提示组件 | `frontend/src/components/status/StatusNotice.tsx` | ✅ | `:140-225` |
| 通知与评论 REST | `crates/star-api-rest/src/routes/notifications.rs` | 🟡 | 端点已实现；进程内存，无鉴权（`:52-64`、`:512-515`） |
| IM REST | `crates/star-api-rest/src/routes/im.rs` | 🟡 | 端点已实现；进程内存，presence 为占位（`:59-61`、`:817-822`） |
| 生产挂载 | `crates/star-api-rest/src/lib.rs:165-183`；`crates/star-api-rest/src/main.rs:155-166` | ✅ | 生产合并 `build_supplemental_routers()`（`main.rs:164`） |
| 领域通知 | `crates/domain-notification/src/lib.rs` | 🟡 | 投递规则、降噪、已读、模板权限已实现（`:55-142`、`:679-682`、`:715-825`）；无持久化、无渠道发送、无重试循环、无生产调用方 |
| 领域测试 | `crates/domain-notification/tests/integration_test.rs` | ✅ | 21 个用例，仅使用内存服务（见 §7） |
| MCP 通知资源 | `crates/star-mcp/src/handlers/notification.rs` | 🟡 | 返回固定值（`:28-37`） |
| 旧收件箱接口 | `frontend/src/app/api/notifications/route.ts`；`frontend/src/app/api/notifications/[id]/route.ts` | 🗑 | 返回 410（`route.ts:21-31`；`[id]/route.ts:26-32`） |
| 收件箱 MSW mock | `frontend/src/mocks/handlers/inbox.ts`；`mocks/data/inbox.ts`；`mocks/schemas/inbox.ts` | 🟡 | 仅服务于 mock 测试，路径为 `/api/notifications`，与页面不一致（`inbox.ts:16-23`） |
| 旧通知页面文件 | `frontend/src/app/notification/page.tsx` | ❓ | 使用 seed 的 `useStore`（`:3`、`:11`），路径被重定向规则覆盖（`redirects.ts:105`），是否可达未确认 |
| 侧栏收件箱条目 | `frontend/src/lib/nav/registry.ts:170-181`；`frontend/src/components/Sidebar.tsx:250-257` | 🟡 | 入口存在，计数硬编码为 5 |
| 旧入口重定向 | `frontend/src/lib/redirects.ts:104-120`；`redirects.shim.cjs:54-59`；`frontend/next.config.js:39-44` | ✅ | 六个入口均指向 `/inbox`（`type` 不生效） |
| 即时通信面板 | `frontend/src/components/chat/DmCenterPanel.tsx`；`RightChatRail.tsx:200-219` | 🟡 | 本地模式可用，状态诚实；无后端接通 |
| 私聊状态 | `frontend/src/lib/chat/dmStore.ts` | 🟡 | 本地会话与消息、`localStorage` 持久化 ✅；无后端接通 |
| IM 网关客户端 | `frontend/src/lib/chat/imGateway.ts`；`frontend/src/types/imGateway.d.ts` | 🟡 | 客户端 ✅；服务端不在仓内 |
| 数据库表 | `db/migrations/` | ⬜ | 无 notification 与 IM 表 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 没有业务事件产生通知；`domain-notification` 无生产调用方（FR-15-09） | 收件箱在生产中只会显示手工创建的数据，实际无通知来源 | 在评审请求、评论、提及等事件路径接入 `dispatch`，并给出事件订阅点 |
| REST 通知与 IM 为进程内存（`notifications.rs:52-54`、`im.rs:59-61`） | 服务重启即丢失；多实例不一致 | 接入 PostgreSQL 表与仓储实现，补迁移（FR-15-25） |
| 通知与 IM 接口无鉴权，列表不按身份过滤（FR-15-19、FR-15-20、FR-15-31） | 任何调用方可读取所有租户的通知，可标记任意通知，可伪造 `X-User-Id` | 为生产合并路由加鉴权层，改用 `ActorContext`，列表按 `tenant_id` 与 `user_id` 过滤 |
| 领域层没有渠道发送、异步投递、重试与死信（FR-15-15、16、17） | 邮件与站外渠道不可用；`DeadLettered` 状态无法产生 | 引入 worker 与渠道适配器，实现指数退避与死信写入 |
| 降噪规则文案与实现不一致：`notification/page.tsx:21` 写有“同 actor 60 分钟内同 kind 第二次自动抑制”，代码中只有按事件类型的抑制（`lib.rs:140-142`、`727-731`） | 页面文案与行为不符，页面本身不可达 | 决定是否实现时间窗口去重，并修正文案或删除该页面 |
| `type` 参数未被消费（FR-15-07） | 六个旧入口统一落到未过滤列表 | 在页面读取 `searchParams.type` 并映射为分类 |
| 分类依据为文本正则，与后端 `kind` 不一致（FR-15-02、FR-15-08） | 同一条通知的分类可能与服务端不同；`kind` 与 mock 的七种取值不一致 | 以后端 `kind` 为准，统一取值表 |
| 侧栏计数硬编码为 5（`registry.ts:180`） | 与真实未读数不符 | 调用 `fetchUnreadCount`，并由侧栏读取 |
| “在 Branch 中打开”仅为外链（FR-15-05） | 无法回到应用内对象 | 为 `target_type` 与 `target_id` 建立应用内路由映射 |
| 评论客户端无调用方，界面无评论入口（FR-15-21） | 评论 REST 已可用，但用户无法使用 | 在目标对象详情中接入评论列表与发表 |
| MSW mock 路径为 `/api/notifications`，页面调用 `/api/v1/notifications`（`inbox.ts:17-22`、`notificationsApi.ts:54`） | mock 模式下收件箱无法展示 mock 数据 | 对齐 mock 路径，或删除旧 mock |
| 私聊与群聊只在浏览器本地（FR-15-23、FR-15-24、FR-15-25） | 其他用户看不到消息，刷新与换设备后会话不同步 | 将 `dmStore` 接入 `/api/v1/im` 接口，并以服务端会话为准 |
| IM1.0 网关服务端不在仓内，默认地址 `ws://localhost:9001`（`imGateway.ts:187`），展开面板即尝试连接并按退避重连（`imGateway.ts:300-318`） | 实时送达不可用；后台重连请求持续产生；端口与路径无法在本仓核对（❓） | 启动网关后核对端口与路径，并以环境变量 `NEXT_PUBLIC_IM_WS_URL` 配置；在网关就绪前可考虑延迟连接 |
| IM 使用 `X-User-Id` 作为身份，前端当前用户为常量（FR-15-31、FR-15-32） | 身份可被伪造，登录用户无法映射到 IM 用户 | 接入登录态，身份以 JWT 为准 |
| presence 为占位（FR-15-29） | 在线状态不可用 | 在网关接入后由网关提供 presence 帧 |
| 消息仅支持文本（FR-15-30） | 图片、文件、贴纸不可用 | 后端扩展消息类型，前端补发送路径 |
| 渠道端点与模板端点缺失（FR-15-13、FR-15-14） | 通知偏好无法设置 | 设计渠道与模板的 REST 契约，并补齐端口方法 |
| spec 的端口方法与错误码未在代码中出现（FR-15-36） | 设计与实现的契约不一致 | 按 spec 补充端口方法与 `N-001` 至 `N-005`，或修订 spec |
| 旧页面文件被重定向遮蔽：`notification`、`audit`、`comment`、`context`、`search`、`feedback` 的 `page.tsx` 仍在 `frontend/src/app/` 下（`redirects.ts:105-120`） | 文件存在但路由不可达，易误导 | 运行 `next build` 核对可达性后删除或归档 |
| 下游 IM1.0 网关与本仓的协议版本未在本仓验证 | 协议变体数的一致性只能靠注释保证 | 在网关仓的 CI 中加入协议帧快照测试（❓） |

## 7. 验收与测试

| 测试 | 位置 | 验证内容 | 未验证内容 |
|---|---|---|---|
| 收件箱页面用例 | `frontend/src/app/(app)/inbox/__tests__/inbox.test.tsx:111-246`（7 例） | real-mode 未开启时的未接入态；加载态；真实拉取成功与分类过滤；空态；后端不可达；已读切换失败回滚；已读切换成功 | 测试用 MSW 的 `server.use` 拦截 `/api/v1/notifications`（`:126-187`），不接真实后端；不验证 Bearer 与鉴权；不验证分页 |
| 页面默认态 | `frontend/src/app/(app)/__tests__/panels.test.tsx:123-131` | 默认无 real-mode 时显示未接入态且列表为空 | 不覆盖真实数据路径 |
| 收件箱 mock | `frontend/src/mocks/__tests__/inbox.test.ts:8-32`；`handlers.test.ts:35`、`47-48` | MOCK_NOTIFS 的 10 条数据、类型守卫、id 格式、已读覆盖；收件箱 handler 数量 | 与生产路径无关 |
| 即时通信面板 | `frontend/src/components/chat/DmCenterPanel.test.tsx:31-173`（12 例） | 默认本地模式状态条；`setRealtimeConnected` 切换；新建私聊与群聊；取消弹窗；发送与清空；Enter 发送；空输入禁用；未读徽标与清零；群聊已读人数；头像与摘要；按活动时间排序 | 不涉及后端 |
| 私聊状态 | `frontend/src/lib/chat/dmStore.test.ts:49-296`（20 例） | 会话幂等与成员集合复用；`sendDm` 的序号、状态与未读；网关上行失败回退；`bindImGateway` 与解绑；`tryLoadImGateway` 适配；`localStorage` 持久化与重载；SSR 守卫 | 不涉及后端 |
| IM 网关客户端 | `frontend/src/lib/chat/imGateway.test.ts:136-514`（30 例） | 帧形状（snake_case、按 kind 判别、可选字段缺省）；状态机与退避序列；心跳；断线队列与幂等键；未知帧丢弃；工厂与地址优先级 | 通过注入的 socket 工厂模拟（`imGateway.ts:305`），不与真实服务端通信 |
| 通知 REST | `crates/star-api-rest/tests/notifications_endpoints.rs`（12 例） | 创建后列表往返；未读计数；kind 过滤与分页；标记已读幂等；404；字段校验；评论增删；评论所有权；`X-User-Id` 必填；按创建时间倒序；未读计数按租户隔离 | 使用隔离路由与内存状态（`:61`），不经生产合并路由，不验证鉴权 |
| IM REST | `crates/star-api-rest/tests/im_endpoints.rs`（11 例） | 私聊往返与未读；私聊幂等；群聊与群主角色；分页；404；非成员拒绝；已读单调与夹紧；回复引用；presence 占位；缺少 `X-User-Id` 返回 400；成员 ID 别名 | 同上 |
| 生产路由挂载 | `crates/star-api-rest/tests/production_router.rs:85-87` | `/api/v1/im/conversations`、`/api/v1/notifications`、`/api/v1/comments` 返回非 404 | 只验证路由存在，不验证行为 |
| 领域通知 | `crates/domain-notification/tests/integration_test.rs`（21 个 `it_*` 用例，`:74-930`） | 完整生命周期；列表与未读过滤；模板幂等与管理员限制；INV-N-07 默认抑制与突破；FR-ORCA-040 与 041；跨租户拒绝；他人不可读、不可列、不可标记；多用户隔离；终态标记；权限检查 | 只使用内存服务，没有持久化与渠道发送测试 |
| 重定向结构 | `frontend/e2e/redirects.spec.ts` | 文件中含 inbox 相关的重定向结构校验 | 具体断言未逐条确认（❓） |

验证缺口：

- 没有针对 `/inbox` 或即时通信的浏览器端到端用例（`frontend/e2e/` 中无对应 spec）。
- 没有任何测试让前端、REST、真实数据库与鉴权同时运行。
- 没有对跨用户访问的 REST 测试；`notifications_endpoints.rs` 中的隔离只针对租户过滤。
- 没有 worker、重试与死信的测试，因为这些实现不存在。
- 没有对网关服务端的任何验证。
- 没有对 mockup 视觉还原的核对。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-INBOX-001.md` | FR-IN-001 至 FR-IN-004；§1 设计要点（左右栏、分类、详情、六域 sink）；§3 文件映射 | 头部写数据源为 seed store，但页面已改为真实 API（`page.tsx:19-26`）；FR-IN-002 写为 ✅，但外链不是应用内 Branch（`page.tsx:286-296`）；FR-IN-004 写为 ✅，但 `type` 不生效（`page.tsx` 无 `searchParams`） | 需求条目采用，编号映射为 FR-15-01、03、05、07；状态以代码为准 |
| `docs/specs/domain-notification-spec.md` | 职责与边界（§1）；实体（§2）；INV-N-01 至 INV-N-07（§3）；端口（§4）；订阅者与发布事件（§5）；RLS 与索引（§6）；权限串（§7）；错误码（§8）；任务分解（§9）；验收场景（§10）；风险（§11）；开放问题（§12）；流程图（附录 A） | 端口多于代码（§4 中的 `update_channel`、`delete_channel`、`mark_all_read`，代码中无）；错误码与权限串在代码中无匹配；§5 订阅者未接通；附录流程图直接写 PostgreSQL，代码为内存；文件中有两个 §6（数据所有权与 Orca 落地），编号重复；§15 引用的脚本未核对 | 设计意图保留，标注为 ⬜；状态与证据以代码为准；不采用 spec 中的行号引用 |
| `docs/prototypes/3 · 收件箱-html/README.md` 与 `Inbox.dc.html` | 三栏布局；四个分类标签；“在 Branch 中打开”“标为已读”；“收件箱 · 5”；项目折叠列表（`Inbox.dc.html:35`） | 页面没有项目折叠列表；侧栏计数为硬编码的 5；README 明确其为设计参考，不是生产代码 | 作为设计参考；布局与分类以页面为准，项目列表记为 ❓ |
| `docs/features/F15-inbox-notification-im.md`（同路径既有草稿，637ed19d） | 与本文件同一主题的已有草稿，包括对 G-01 至 G-04 与网关端口的记录 | 其中的时间标注与路由数量不采用，逐条以代码核对；其引用的外部仓库 `D:/IM1.0` 端口记录不在本 pin 中，无法核实 | 仅作交叉核对：通知与 IM 的内存态、领域无生产调用方、本地模式等结论与代码一致；外部端口记录标为 ❓ |
| `docs/features/F02-branch-workspace.md`（FR-02-07、08；第 174、197、210、212 行） | 右侧聊天栏与群组页的并存；AI 更新页签 | 无 | 确定范围外归属：`chatStore`、`SideChatPanel`、群组页“AI 更新”归 F02 |
| `docs/features/F01-nav-project-lane.md`（第 20、65、220 行） | AI 更新页签的数据源缺口；聊天栏不在导航范围 | 无 | 确定范围外归属，与 F02 一致 |
| `docs/prototypes/INDEX.md`、其他原型目录 | 未读取 | 与本功能无关 | 舍弃，理由：与收件箱无关 |
| `docs/architecture/2026-08-26-upgrade/spec/*`（通知与 agents 相关，经 grep 命中）、`docs/basic-design.md` | 未逐篇阅读 | 设计细节由 domain-notification spec 覆盖 | 舍弃，理由：设计依据已由 spec 与代码覆盖；如需引用须另行核对 |
| `docs/ecosystem-survey/orca-design-survey.md`（spec §6 引用） | 未读取 | FR-ORCA-040、041 的需求以 spec 与代码为准 | 舍弃，理由：本仓不包含该调研文件的证据链 |
| `db/migrations/2026-09-10-p3d6-13-tables.sql` | 检索结果：仅 `canvas_comments` 与通知、IM 无关 | 无 | 采用为“无数据库表”的证据 |
| 前端与后端源码及测试（§4、§5、§7 所列路径） | 全部端点、类型、状态、测试名 | 与 spec 的差异见上 | 作为本文件的主要事实依据 |
| 外部仓库 `D:/IM1.0` | 不在本 pin 中，无法读取 | 网关端口与路径的记录只能来自既有草稿 | 标为 ❓，列入开放问题 |
