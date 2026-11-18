# F15 收件箱、通知与即时通信

> 状态摘要: 收件箱页面、通知与评论的前端客户端和后端 REST 契约、领域层投递与已读规则 (domain-notification) 均已有代码。**2026-11-18 更新: G-01 已修** — 服务进程 `main.rs` 现挂载 `routes::notifications::router()`、`comments_router()`、`routes::im::router()` (33 条路由)，oneshot 集成测试 12 passed (见 `tests/notifications_endpoints.rs`)。遗留: REST 存储仍为进程内存 (G-02/G-03/G-04)，领域通知无生产调用方、无持久化、无重试与渠道发送。右侧聊天栏的私聊与群聊为浏览器本地存储，IM1.0 网关客户端已实现，网关服务端不在本仓库且默认地址与其记录端口不一致。
>
> 代码基线: HEAD 1643559f; 核对方法: path/symbol grep plus file reads of the working tree (工作树与 HEAD 仅 crates/star-api-rest/src/routes/im.rs 中一行文档注释不同，行号不受影响); 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

- 将站内通知 (评审请求、评论、系统通知) 聚合到 `/inbox` 收件箱，支持类别过滤、详情查看与已读切换。
- 提供通知与评论的后端 REST 契约，以及领域层的投递、已读与降噪规则。
- 在右侧聊天栏提供用户间私聊与群聊，并可接入 IM1.0 实时网关。

### 1.2 范围内 / 范围外

| 范围内 | 范围外 |
|---|---|
| `/inbox` 页面、通知与评论客户端、通知与评论 REST 契约、`domain-notification` 模型与端口 | 邮件、Slack、钉钉渠道发送 (见 FR-15-18) |
| 6 域旧路由 (notification、comment、audit、feedback、search、context) 到 `/inbox` 的重定向 | 6 个被重定向域各自的目标页面，归属其他功能 |
| 右侧聊天栏私聊与群聊面板 (DmCenterPanel) 及其本地存储 | IM1.0 服务端实现，位于外部仓库 `D:/IM1.0`，不属于本 workspace |
| IM 网关客户端 `imGateway.ts` 与 IM 后端 REST 契约 `routes/im.rs` | 原生系统通知适配、通知模板渲染、多语言 |

## 2. 用户与场景

| 角色 | 主要场景 | 现状 |
|---|---|---|
| 个人用户 (developer / viewer) | 查看评审请求与评论，按类别过滤，标记已读，打开关联资源 | 页面与契约存在，生产进程不可达 (FR-15-10) |
| 个人用户 | 在右侧聊天栏与成员私聊或建群，查看未读与已读状态 | 本地可用，无服务端持久化 (FR-15-20) |
| 项目或租户管理员 | 维护通知模板 | 仅领域层端口存在，无 UI 与 REST (FR-15-14) |
| 事件生产者 (系统、业务事件) | 派发通知 | 无生产调用方 (FR-15-16) |
| 运维与诊断 | 区分未接入、未授权、失败、空状态 | 五态已实现 (FR-15-05) |

主要用户故事:

1. 用户打开 `/inbox`，按"评审请求 / 评论 / 通知"过滤，选中一条消息查看详情。
2. 用户点击"标为已读"，列表项未读标记消失；后端失败时状态回滚并提示。
3. 用户在右侧聊天栏展开私聊，选择成员发送消息，看到已发送、已送达、已读标记。
4. 后端不可达或未授权时，收件箱显示对应的诚实状态，不显示 mock 数据。

## 3. 需求

优先级: P0 = 当前版本必须; P1 = 设计要求，尚未进入当前版本; P2 = 远期设计意图。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-15-01 | 收件箱展示通知列表与详情，列表含主题、类别、相对时间与未读标记 | P0 | 🟡 | 列表 `frontend/src/app/(app)/inbox/page.tsx:235-267`；详情 `page.tsx:270-317`；数据路径在生产不可达，见 FR-15-10 |
| FR-15-02 | 按类别过滤 (全部、评审、评论、通知) | P0 | ✅ | 类别由前端按 kind 与 subject/body 关键词推断 `page.tsx:51-56`；tabs `page.tsx:190-207`；过滤 `page.tsx:143-145` |
| FR-15-03 | 标记已读与未读，乐观更新，失败回滚 | P0 | 🟡 | `page.tsx:119-132`；PATCH 客户端 `frontend/src/lib/notifications/notificationsApi.ts:168-182`；后端 `crates/star-api-rest/src/routes/notifications.rs:635-646`；路由未被服务进程挂载 (FR-15-08) |
| FR-15-04 | 在关联资源处打开 ("在 Branch 中打开") | P0 | 🟡 | 仅当 `subject_url` 非空时渲染外链按钮 `page.tsx:286-296`；无应用内 Branch 导航 |
| FR-15-05 | 未接入、未授权、加载、失败、空五态，禁止回落 mock | P0 | ✅ | 状态映射 `page.tsx:149-168`；real-mode 门禁 `notificationsApi.ts:76-83`；401/403 映射 `notificationsApi.ts:107-113`；页面不引用 MOCK 数据 (导入 `page.tsx:29-45`) |
| FR-15-06 | 6 域旧路由重定向到 `/inbox` | P0 | 🟡 | 规则 `frontend/src/lib/redirects.ts:104-122`，`frontend/src/lib/redirects.shim.cjs:53-59`，由 `frontend/next.config.js:39-44` 载入；页面不读取 `type` 查询参数，类别参数无效 |
| FR-15-07 | 根路径 `/` 默认落到 `/inbox` | P0 | ✅ | `frontend/src/app/page.tsx:32` |
| FR-15-08 | 通知 REST 契约：列表、未读计数、创建、PATCH 已读、POST 已读 | P0 | 🟡 | 路由 `notifications.rs:222-236`；处理器 `notifications.rs:506-662`；注册仅在 `crates/star-api-rest/src/lib.rs:158` 的 `build_router()` (`lib.rs:93-227`)；服务二进制 `crates/star-api-rest/src/main.rs:133` 使用 `build_group_router` (`crates/star-api-rest/src/group_api.rs:796`)，不含该路由 |
| FR-15-09 | 评论 REST 契约：按目标列表、新增、删除 (仅作者) | P0 | 🟡 | 路由 `notifications.rs:238-250`；处理器 `notifications.rs:669-760`；同 FR-15-08 未挂载；前端客户端 `notificationsApi.ts:198-226` 无 UI 调用方 |
| FR-15-10 | 真实模式下收件箱读取真实后端列表 | P0 | 🟡 | 请求 `page.tsx:99-103` → `notificationsApi.ts:159-166` → `realFetch` `frontend/src/mocks/real-mode.ts:58-66`；开发态经 `frontend/next.config.js:46-58` 代理到 `127.0.0.1:8081`；8081 进程不含该路由，推断返回 404，并被 `StatusNotice.tsx:183-198` 归为"加载失败"。❓ 运行时未实测 |
| FR-15-11 | 通知与评论持久化 (PostgreSQL，RLS) | P1 | ⬜ | 状态为进程内 HashMap `notifications.rs:186-193`，自述"服务重启即失" `notifications.rs:52-53`；`db/migrations` 无 notification 表；设计见 `docs/data-design.md:2361` 起 |
| FR-15-12 | REST 层租户隔离 (INV-N-01)：列表与计数只返回调用方租户 | P0 | ⬜ | `tenant_id` 仅为可选查询过滤，未从身份派生 `notifications.rs:506-516`、`:555-563`；创建接受任意 `tenant_id` `notifications.rs:574-595` |
| FR-15-13 | 本人隔离 (INV-N-03)：仅本人可读与标记已读 | P0 | 🟡 | 领域层 ✅：`crates/domain-notification/src/lib.rs:810-813` (mark_read)、`:856-859` (get)、`:874-878` (list)；REST 层 ⬜，无按用户过滤 `notifications.rs:55-56` |
| FR-15-14 | 领域层租户与角色校验：渠道注册仅本人，模板维护限项目或租户管理员 (INV-N-03、INV-N-04) | P0 | ✅ | `crates/domain-notification/src/lib.rs:639-648` (register_channel)；`:679-682` (upsert_template 角色检查)；`:720-725` (dispatch 租户)；`:786-790` (mark_read 租户) |
| FR-15-15 | 默认降噪 (INV-N-07)：非突破事件的派发被拒绝 | P0 | ✅ | 分类 `lib.rs:124-142`；派发门禁 `lib.rs:727-731`；领域单元测试 `lib.rs:1049-1050` 起。REST 创建不经过该门禁 |
| FR-15-16 | 生产事件源经领域层派发通知 (feedback、validation、agent_session 等) | P0 | ⬜ | 领域派发的唯一引用为 `crates/domain-work-item/Cargo.toml:27` 的 dev-dependency 与其测试 `crates/domain-work-item/tests/cross_saga_test.rs:27`；MCP 资源为硬编码 mock `crates/star-mcp/src/handlers/notification.rs:26-35`；`crates/star-dispatcher/src/sa_real_impls.rs:258` 仅登记 crate 名 |
| FR-15-17 | 失败重试 (指数退避，最多 5 次) 与死信队列 (INV-N-06) | P1 | ⬜ | 常量 `MAX_RETRY` 标注 `#[allow(dead_code)]` `lib.rs:629-630`；派发直接写 Sent `lib.rs:767`；`DeadLettered` 无写入路径 |
| FR-15-18 | 渠道发送：Email、Slack、钉钉 | P1 | ⬜ | 仅有渠道枚举 `lib.rs:212-218`；无发送适配器 |
| FR-15-19 | 通知状态生命周期 Pending → Sent → Delivered → Read | P1 | 🟡 | Sent → Read 已实现 `lib.rs:767`、`:820`；Pending、Delivered、Failed 无写入路径，`lib.rs:323-325` 与 `:334-340` 仅作判断 |
| FR-15-20 | 站内私聊与群聊：创建、发送、未读计数、已读回执 | P0 | 🟡 | 创建 `frontend/src/lib/chat/dmStore.ts:178-209`；发送 `dmStore.ts:211-269`；已读 `dmStore.ts:304-325`；面板 `frontend/src/components/chat/DmCenterPanel.tsx`；挂载 `frontend/src/components/chat/RightChatRail.tsx:213-215`；仅 localStorage `dmStore.ts:436-440`，无服务端调用 |
| FR-15-21 | IM1.0 网关客户端：帧收发、断线重连、心跳、待发队列 | P1 | 🟡 | 客户端 `frontend/src/lib/chat/imGateway.ts:239-511`；ClientFrame 8 类 `imGateway.ts:62-80`，ServerFrame 11 类 `imGateway.ts:117-140`，枚举变体数与 `D:/IM1.0/crates/im-protocol/src/ws_frames.rs` 一致；心跳 `imGateway.ts:377`；退避 `imGateway.ts:340-346`；网关服务端不在本仓库 |
| FR-15-22 | IM 后端 REST：会话、消息、已读、在线状态，成员校验 | P1 | 🟡 | 路由 `crates/star-api-rest/src/routes/im.rs:250-261`；处理器 `im.rs:420-817`；成员校验 `im.rs:645`、`:713`、`:773`；由 `build_router()` 挂载 `lib.rs:157`，服务进程未挂载；前端无调用方 |
| FR-15-23 | 当前用户与用户目录来自真实身份 | P1 | ⬜ | 当前用户写死 `dmStore.ts:101-102` (`usr-001`)；目录取 seed `dmStore.ts:104-107` |
| FR-15-24 | 头部铃铛与侧栏显示真实未读数 | P1 | ⬜ | 铃铛固定为 3 `frontend/src/components/AppHeader.tsx:50`；侧栏 Inbox 计数固定为 5 `frontend/src/lib/nav/registry.ts:180`，由 `frontend/src/components/Sidebar.tsx:253-255` 渲染；`fetchUnreadCount` 无调用方 `notificationsApi.ts:184-189` |
| FR-15-25 | 突破事件 AgentFinished、NeedsYou 不被默认降噪抑制 | P1 | ✅ | 变体 `lib.rs:73-75`；突破分类 `lib.rs:124-138`；派生自设计 `docs/specs/domain-notification-spec.md` §6.1 |
| FR-15-26 | 未读判据与侧栏加粗 | P1 | 🟡 | 判据 ✅ `lib.rs:334-340` (is_unread)、`lib.rs:345-347` (should_bold_for_sidebar)；侧栏加粗 ⬜，无前端实现 |
| FR-15-27 | 原生系统通知 (macOS、Windows、Linux) | P2 | ⬜ | `docs/specs/domain-notification-spec.md` §6.3 列为 MVP 外 |
| FR-15-28 | 静默时段、通知去重、模板多语言 | P2 | ⬜ | `docs/specs/domain-notification-spec.md` §12 J-N-02、J-N-03、J-N-04 |
| FR-15-29 | 原型三栏布局中的项目分组左栏 | P2 | ⬜ | 设计稿左栏为项目分组 `docs/prototypes/3 · 收件箱-html/Inbox.dc.html`；实现采用全局侧栏 |

## 4. 设计

### 4.1 概念模型

```
收件箱 (/inbox)
 ├─ 站内通知 (REST 契约)   kind 四类；read 布尔值
 ├─ 领域通知 (domain-notification)   user_id、event_type、status、read_at；与 REST 无映射代码
 ├─ 评论 (REST 契约)        target_type + target_id
 └─ 分类 (前端推断)         评审 / 评论 / 通知

即时通信
 ├─ 前端会话 (dmStore)      浏览器 localStorage，DM 与群聊
 ├─ 服务端会话 (routes/im.rs)  进程内存，未挂载到服务进程
 └─ imGateway (WebSocket 客户端) ── 外部 IM1.0 网关
```

### 4.2 数据模型

| 实体 | 字段 (代码定义) | 位置 | 设计来源与差异 |
|---|---|---|---|
| REST Notification | id, tenant_id, kind (review_request / comment / mention / system), subject, body, subject_url?, actor_id?, actor_name?, target_type?, target_id?, read, created_at | `notifications.rs:128-154`；wire `notifications.rs:369-384` | `docs/api-design.md:451-453` 设计字段为 recipient_user_id、event_type、payload、read_at、sent_at，REST 未提供 |
| REST Comment | id, tenant_id, target_type (自由字符串), target_id, author_id, author_name, body, created_at | `notifications.rs:158-176` | 设计未列评论实体 |
| 领域 Notification | id, tenant_id, user_id, event_type, resource_type, resource_id, channel_id, subject, body, status, created_at, sent_at?, read_at?, retry_count | `crates/domain-notification/src/lib.rs:262-292` | `docs/specs/domain-notification-spec.md` §2 |
| 领域 NotificationChannel | id, tenant_id, user_id, kind (Email / InApp / Slack / DingTalk), address, enabled, created_at, updated_at | `lib.rs:191-206`；`ChannelKind` `lib.rs:212-218` | spec §2；PG 设计含 config JSONB，`docs/data-design.md:2370` 起 |
| 领域 NotificationTemplate | id, tenant_id, project_id, event_type, channel_kinds, subject, body_template, enabled, created_at, updated_at | `lib.rs:237-251` | `docs/data-design.md:2400` 起含 locale 与版本列，代码无 |
| 领域 NotificationStatus | Pending, Sent, Delivered, Read, Failed, DeadLettered | `lib.rs:295-307` | spec 生命周期 Pending → Sent → Delivered → Read |
| 领域 NotificationEventType | 18 个变体：突破 9、抑制 8、Custom 1 | `lib.rs:55-98` | spec §3 INV-N-07 |
| 前端 Notification / Comment | 与 REST wire 逐字段对齐；kind 宽容为 string | `frontend/src/types/notifications.ts:39-71` | 文件注释声明与后端对齐 |
| 前端 DmConversation / DmMessage | DmConversation: id, kind (dm / group), memberIds, title?, messages, unread, lastActivityAt, realtimeConnected；DmMessage: id, conversationId, authorId, authorName, text, at, sequence, state (sent / delivered / read), readBy | `frontend/src/lib/chat/dmStore.ts:32-66` | 文件注释称对齐 IM1.0 (`dmStore.ts:16`) |
| 服务端 IM Conversation | id, kind (Dm / Group), title?, members, next_sequence, created_at | `crates/star-api-rest/src/routes/im.rs:155` | 无 tenant_id |
| 服务端 IM Member | user_id, role (Owner / Admin / Member), last_read_sequence | `im.rs:143-152` | — |
| 服务端 IM Message | id, conversation_id, sequence, sender_id, kind, content (JSON), reply_to?, state (Sent / Delivered / Read / Recalled / Deleted), created_at | `im.rs:173-186`；状态 `im.rs:128` | 无 tenant_id |
| 网关帧 | ClientFrame 8 类；ServerFrame 11 类 | `imGateway.ts:62-80`、`imGateway.ts:117-140` | 与 IM1.0 `ws_frames.rs` 枚举变体数一致 |

数据库层: `docs/data-design.md` §4.15 定义了 `notification.notification_channel`、`notification.notification_template` 及 `notification` 表，并要求 RLS 与 tenant 隔离。代码中未发现对应迁移 (`db/migrations` 仅在注释中出现 notification 一词)。

### 4.3 接口

| 方法与路径 | 处理器与位置 | 注册位置 | 服务进程挂载 | 前端调用 |
|---|---|---|---|---|
| GET /api/v1/notifications | list_notifications `notifications.rs:506-549` | `lib.rs:158`，路由 `notifications.rs:230` | 否 | `listNotifications` `notificationsApi.ts:159-166` |
| GET /api/v1/notifications/unread-count | unread_count `notifications.rs:555-567` | `notifications.rs:231` | 否 | 无 (`fetchUnreadCount` 未被调用) |
| POST /api/v1/notifications | create_notification `notifications.rs:574-629` | `notifications.rs:232` | 否 | 无 |
| PATCH /api/v1/notifications/{id} | patch_notification_read `notifications.rs:635-646` | `notifications.rs:233` | 否 | `markRead` / `markUnread` `notificationsApi.ts:168-182` |
| POST /api/v1/notifications/{id}/read | mark_notification_read `notifications.rs:652-662` | `notifications.rs:234` | 否 | 无 |
| GET /api/v1/comments | list_comments `notifications.rs:669-693` | `lib.rs:159`，路由 `notifications.rs:246` | 否 | `listComments` 无 UI 调用 |
| POST /api/v1/comments | create_comment `notifications.rs:701-736` | `notifications.rs:247` | 否 | `addComment` 无 UI 调用 |
| DELETE /api/v1/comments/{id} | delete_comment `notifications.rs:743-760` | `notifications.rs:248` | 否 | `deleteComment` 无 UI 调用 |
| GET / POST /api/v1/im/conversations | list_conversations `im.rs:543`；create_conversation `im.rs:420` | `lib.rs:157`，路由 `im.rs:250-251` | 否 | 无 |
| GET / POST /api/v1/im/conversations/{id}/messages | list_messages `im.rs:694`；send_message `im.rs:604` | `im.rs:252-257` | 否 | 无 |
| POST /api/v1/im/conversations/{id}/read | mark_read `im.rs:750` | `im.rs:260` | 否 | 无 |
| GET /api/v1/im/presence | presence `im.rs:817` (占位) | `im.rs:261` | 否 | 无 |
| GET /api/notifications, /api/notifications/{id} (GET, PATCH) | 返回 410 | `frontend/src/app/api/notifications/route.ts:21-30`；`[id]/route.ts:26-31` | 不适用 | 无 |
| notification://{id} (MCP) | 硬编码 mock | `crates/star-mcp/src/handlers/notification.rs:26-35` | 不适用 | 无 |

领域端口 (`crates/domain-notification/src/lib.rs`):

- `NotificationCommandPort` `lib.rs:479`：register_channel、upsert_template、dispatch、mark_read。
- `NotificationQueryPort` `lib.rs:511`：get、list_by_user。
- `NotificationRepository` `lib.rs:529`：唯一实现为内存版 `InMemoryNotificationRepository` `lib.rs:893-917`。

网关帧与连接 (`frontend/src/lib/chat/imGateway.ts`): `ImGateway` 类 `:239`，工厂 `connectImGateway` `:507`，默认地址 `DEFAULT_WS_URL = "ws://localhost:9001"` `:187`，可由 `NEXT_PUBLIC_IM_WS_URL` 覆盖 `:218-224`。

### 4.4 状态机与关键流程

领域通知 (仅两条生产写入路径):

```
dispatch(非抑制事件)            → Sent   (sent_at = now)           lib.rs:715-779，写入 :767
Sent --mark_read(本人)-->        Read     (read_at = now)            lib.rs:781-825，写入 :820
Pending / Delivered / Failed / DeadLettered: 无生产写入路径
```

REST 通知: 状态仅为 `read` 布尔值 `notifications.rs:128-154`，与领域 status 无映射。

收件箱页面加载 (`page.tsx:80-168`):

```
loading → error(real_mode_disabled) → "未接入"
        → error(401/403)           → "未授权"
        → error(其他 HTTP 错误或网络)  → "失败" 或 "未接入" (按 StatusNotice 正则)
        → ready 且空               → "空"
        → ready 且有数据           → 列表 + 详情
```

已读切换: 乐观更新 `page.tsx:121-122`，请求成功以响应为准 `:126`，失败回滚并设置错误标记 `:127-131`。

私聊发送 (`dmStore.ts:211-269`): 先本地写入 `state: "sent"` `:219-229`；若网关已绑定且连通，则上行 `:247-268`，失败则将 `realtimeConnected` 置为 false。接收到他人消息时状态为 `delivered` `:283`；`markConversationRead` 将他人消息置为 `read` `:313-318`。

网关连接状态: 类型为 `connecting | connected | reconnecting | offline` `imGateway.ts:146`；初始 offline `:242`；断线后按 `min(baseBackoffMs × 2^attempt, maxBackoffMs)` 重连 `imGateway.ts:340-346`，默认 base 1 s、上限 30 s `:171-172`；`disconnect()` 置为 offline `imGateway.ts:287-296`；心跳默认 25 s `imGateway.ts:167`、`:377`。

规范中的事件驱动流程 (`docs/specs/domain-notification-spec.md` 附录 A: 事件 → 模板 → 渠道 → 派发 → 邮件 / 站内 → 已读) 当前未实现: 无事件订阅者，无模板渲染，无渠道发送。

### 4.5 权限与安全

| 控制点 | 设计意图 | 现状 | 证据 |
|---|---|---|---|
| 领域层租户隔离 | 所有操作校验 actor.tenant_id 与命令 tenant_id | ✅ | `lib.rs:639-644`、`:673-678`、`:720-725`、`:786-790` |
| 领域层本人隔离 | 仅本人可读、标记已读、列表 | ✅ | `lib.rs:810-813`、`:856-859`、`:874-878` |
| 领域层模板角色 | project_admin 或 tenant_admin | ✅ | `lib.rs:679-682` |
| REST 租户隔离 | 同领域层 | ⬜ 列表、计数接受任意 tenant_id，创建接受任意 tenant_id | `notifications.rs:506-516`、`:555-563`、`:574-595` |
| REST 本人隔离 | 仅本人可见通知 | ⬜ 无按用户过滤，代码注释承认 | `notifications.rs:55-56` |
| REST 身份来源 | ActorContext 或 JWT | ⬜ 认证层为 pass-through stub，且仅挂在 `build_router` 上 | `crates/star-api-rest/src/middleware/auth.rs:23-29`；`lib.rs:221-224` |
| 评论身份 | 由认证上下文派生 | 🟡 取 `X-User-Id` 请求头，未验证，可伪造 | `notifications.rs:308-325` |
| 评论删除 | 仅作者可删 | 🟡 以请求头与 author_id 比对，依赖未验证头部 | `notifications.rs:743-754` |
| IM 成员校验 | 会话成员才能读写 | 🟡 后端实现，服务进程未挂载 | `im.rs:645`、`:713`、`:773` |
| 前端凭证 | Bearer 注入，凭证存于浏览器 | 🟡 `real_api_key` 存 localStorage，`Authorization` 由 realFetch 注入 | `real-mode.ts:24-33`、`:58-66` |
| IM 令牌 | localStorage `star-im-token` | 🟡 读取存在，无签发流程 | `imGateway.ts:188`、`:207-216` |
| 通知正文渲染 | 文本渲染，防注入 | ✅ subject 与 body 以 React 文本输出 | `page.tsx:257`、`:261`、`:283-284` |
| 通知链接协议 | 仅允许安全协议 | ⬜ `subject_url` 直接作为 `<a href>`，未校验协议 | `page.tsx:286-288` |
| 私聊身份 | 来自真实会话 | ⬜ 当前用户写死 | `dmStore.ts:101-102` |

### 4.6 依赖关系

| 依赖方 | 被依赖方 | 方向与状态 | 证据 |
|---|---|---|---|
| 收件箱页面 | 通知客户端、real-mode、StatusNotice、i18n | 运行时依赖，✅ | `page.tsx:29-45` |
| 通知客户端 | realFetch | 运行时依赖，✅ | `notificationsApi.ts:51` |
| 开发态代理 | star-api-rest (8081) | 配置依赖，🟡 目标进程不含通知路由 | `frontend/next.config.js:46-58` |
| 服务进程 | build_group_router | 事实依赖，✅ 不含通知与 IM | `main.rs:133`、`group_api.rs:796` |
| REST 通知路由 | domain-notification | 设计依赖，⬜ 无引用，REST 使用本地内存 store | `notifications.rs:186-193` |
| domain-notification | star-context | 编译依赖，✅ | `crates/domain-notification/Cargo.toml:23` |
| domain-work-item | domain-notification | 仅 dev-dependency，✅ 测试用 | `crates/domain-work-item/Cargo.toml:27` |
| 重定向规则 | next.config.js | 配置依赖，✅ | `frontend/next.config.js:39-44` |
| 右侧聊天栏 | AppShell | 挂载依赖，✅ | `frontend/src/components/AppShell.tsx:21`、`:77` |
| DmCenterPanel | dmStore、imGateway (动态导入) | 运行时依赖，🟡 | `DmCenterPanel.tsx:69-80`；`dmStore.ts:366-430` |
| imGateway | IM1.0 网关 (外部) | 运行时可选依赖，❓ 端口与路径未核实 | 见 §6 G-13 |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 收件箱页面 | `frontend/src/app/(app)/inbox/page.tsx` | 🟡 | 322 行；直连真实路径，服务进程不含该路由 |
| 通知与评论客户端 | `frontend/src/lib/notifications/notificationsApi.ts` | ✅ | 226 行；评论函数无 UI 调用 |
| 通知类型 | `frontend/src/types/notifications.ts` | ✅ | 99 行 |
| 旧 MSW 通知 mock | `frontend/src/mocks/handlers/inbox.ts`、`data/inbox.ts`、`schemas/inbox.ts` | 🗑 | 页面不再引用，文件与测试 `frontend/src/mocks/__tests__/inbox.test.ts` 保留；路径为 `/api/notifications` (非 `/api/v1`) `handlers/inbox.ts:1-12` |
| 旧 Next 代理路由 | `frontend/src/app/api/notifications/route.ts`、`[id]/route.ts` | 🗑 | 返回 410 `route.ts:21-30`；`[id]/route.ts:26-31` |
| 通知与评论 REST | `crates/star-api-rest/src/routes/notifications.rs` | 🟡 | 760 行；仅 `build_router()` 挂载 |
| IM REST | `crates/star-api-rest/src/routes/im.rs` | 🟡 | 822 行；仅 `build_router()` 挂载 |
| 服务进程路由组合 | `crates/star-api-rest/src/main.rs:133-137`；`group_api.rs:796` | ✅ (事实) | 不含通知与 IM 路由 |
| 领域通知 | `crates/domain-notification/src/lib.rs` | ✅ 规则；⬜ 调用方 | 1500 行；无生产调用，仅内存 repository |
| MCP 通知资源 | `crates/star-mcp/src/handlers/notification.rs` | 🟡 | 硬编码 mock `:26-35` |
| 重定向规则 | `frontend/src/lib/redirects.ts`、`redirects.shim.cjs`、`next.config.js:39-44` | ✅ | 19 条规则 (`redirects.ts:49-145`) |
| 旧通知页 | `frontend/src/app/notification/page.tsx` | ❓ | 数据来自 zustand seed store (`frontend/src/lib/store.ts:420`、`:819-821`)；`/notification` 被重定向规则遮蔽，遮蔽判断依据 Next.js 配置重定向先于页面路由的框架行为，未实测 |
| 头部铃铛 | `frontend/src/components/AppHeader.tsx:50`、`:157-167` | ⬜ | 固定计数 3 |
| 侧栏计数 | `frontend/src/lib/nav/registry.ts:171-182`、`:180`；`Sidebar.tsx:253-255` | ⬜ | 固定计数 5 |
| 私聊本地模型 | `frontend/src/lib/chat/dmStore.ts` | 🟡 | 468 行；本地持久化，用户目录为 seed |
| 私聊面板 | `frontend/src/components/chat/DmCenterPanel.tsx` | 🟡 | 478 行；仅 `dmOpen` 时挂载 `RightChatRail.tsx:213-215` |
| 右侧聊天栏 | `frontend/src/components/chat/RightChatRail.tsx` | ✅ | 273 行；`AppShell.tsx:77` 挂载 |
| IM 网关客户端 | `frontend/src/lib/chat/imGateway.ts` | 🟡 | 511 行；服务端不在本仓库 |
| IM 模块契约占位 | `frontend/src/types/imGateway.d.ts` | 🟡 | 声明 `getImGateway`、`createImGateway`，真实模块仅导出 `connectImGateway` `imGateway.ts:507` |
| IM 服务端 (IM1.0) | 外部仓库 `D:/IM1.0` | ❓ | 不在本仓库；Cargo workspace 无 im-gateway 依赖 |
| 收件箱全屏布局 | `frontend/src/app/(app)/layout.tsx:34` | ✅ | 列入全高前缀 |

## 6. 差距与后续

| 编号 | 差距 | 影响 | 下一步 |
|---|---|---|---|
| G-01 | ~~服务进程不挂载 `/api/v1/notifications`、`/api/v1/comments`、`/api/v1/im`~~ **已修 (2026-11-18)** | 真实模式收件箱曾显示"加载失败"；现已挂载 33 条路由 | ✅ `main.rs` 合入 `routes::notifications::router()`、`comments_router()`、`routes::im::router()`；oneshot 测试 `tests/notifications_endpoints.rs` 12 passed (含 401 无 Bearer、跨用户隔离、mark-read)。遗留: 挂载的仍是内存版 REST (G-02/G-04 未修) |
| G-02 | REST 与领域两套模型互不映射；REST 不调用 `domain-notification` | 领域 INV-N-03、INV-N-07 对 HTTP 调用方不生效；REST 无 user_id 与 event_type | REST 处理器改为调用 `NotificationCommandPort`、`NotificationQueryPort`，身份由 ActorContext 注入 |
| G-03 | REST 层无租户与本人隔离；创建接口无角色限制 | 跨租户读取通知；任意调用方可向任意租户写入通知 | 认证中间件注入 ActorContext；列表按 actor 过滤；创建限定为服务身份或系统角色 |
| G-04 | 通知与评论、IM 会话均为进程内存；`docs/data-design.md` §4.15 未实现 | 重启丢失数据；无 RLS | 按 §4.15 与 IM 设计建表，替换 `NotificationsApiState` 与 `ImApiState` 的存储 |
| G-05 | 领域 `dispatch` 无生产调用方 | 通知不会由业务事件产生 | 从 feedback、validation、agent_session 订阅事件并调用 dispatch，依据 spec §5 订阅列表 |
| G-06 | 无重试与死信队列；无渠道发送适配器 | 邮件与 Slack 类渠道不可用 | 实现 worker 与指数退避；在 infrastructure 层实现 Email 与 Slack 适配器 |
| G-07 | Pending、Delivered、Failed、DeadLettered 无写入路径 | 状态机与 spec 不一致 | 随 G-05、G-06 补齐写入路径，或在 spec 中收敛状态集 |
| G-08 | 收件箱不读取 `type` 查询参数，6 域 sink 的类别失效；导航中 "Agent Feedback" 入口 `registry.ts:344` 同样失效 | 旧链接与导航无法预选类别 | 页面读取 `searchParams` 并映射到 Category |
| G-09 | 头部铃铛与侧栏计数硬编码 | 显示错误的未读数 | 接入 `fetchUnreadCount` 与列表未读计数，并在列表变化后刷新 |
| G-10 | `subject_url` 未校验协议 | 通知内链接可注入 `javascript:` 等协议 | 仅允许 http、https 或应用内路由 |
| G-11 | 私聊仅本地存储；当前用户写死 `usr-001` | 多设备与多用户不可用 | 接入真实身份与服务端会话 REST (G-01 之后) |
| G-12 | `DmCenterPanel` 每次挂载调用 `connectImGateway()` 创建新实例；卸载仅解除订阅，未调用 `disconnect()`；`autoReconnect` 默认 true | 每次展开面板可能留下一个持续重连的网关实例 (❓ 运行时未确认) | 卸载时调用 `gateway.disconnect()`，或改为应用级单例 |
| G-13 | `DEFAULT_WS_URL` 为 `ws://localhost:9001`，与 IM1.0 记录不一致: `crates/im-gateway/src/main.rs:19` 文档注释写 `IM_HTTP_PORT` 默认 8080；`D:/IM1.0/.worktrees/reg-it/tests/README.md:195` 记为 18080 与 `/v1/ws`；`crates/im-gateway/src/http/mod.rs:145-150` 注明路径可能嵌套为 `/v1/ws/ws` | 网关可能始终不可达，且路径未定 | 启动 IM1.0 网关后核对端口与路径，更新默认值，或仅通过 `NEXT_PUBLIC_IM_WS_URL` 配置 |
| G-14 | 旧 `/notification` 页面被重定向遮蔽，数据来自 seed store | 页面不可达或与真实数据脱节 | 二选一：删除页面，或移除 `/notification` 重定向；经 e2e 验证 |
| G-15 | `types/imGateway.d.ts` 占位声明与真实导出不一致 | 类型契约误导 | 删除占位，或与 `connectImGateway` 签名对齐 |
| G-16 | `frontend/e2e/redirects.spec.ts:69` 注释的分类数字 (8、2、5、4) 与规则组成 (7、2、6、4) 不一致，总数 19 一致 | 文档漂移 | 修正注释 |
| G-17 | 领域派发在无已注册渠道时回退到未持久化的虚拟 InApp 渠道 `lib.rs:744-753` | 通知的 channel_id 指向未存储的渠道 | 派发前要求已注册渠道，或在派发时创建并持久化 InApp 渠道 |

## 7. 验收与测试

| 范围 | 位置 | 验证内容 | 未验证 |
|---|---|---|---|
| 收件箱页面 | `frontend/src/app/(app)/inbox/__tests__/inbox.test.tsx:111-248` (7 个用例，MSW 拦截 `/api/v1/notifications`，`:127` 起) | real-mode 未开启的未接入态、加载态、列表与分类过滤、空态、后端不可达、已读失败回滚、已读成功 | 生产 router；`type` 参数无用例 |
| 页面冒烟 | `frontend/src/app/(app)/__tests__/panels.test.tsx:123` | `/inbox` 渲染无异常 | 功能断言 |
| 旧 mock 数据 | `frontend/src/mocks/__tests__/inbox.test.ts` (5 个) | 旧 MSW 数据形状 | 与真实契约无关 |
| 重定向规则 | `frontend/e2e/redirects.spec.ts:70` (`LEGACY_REDIRECTS.length === 19`) | 规则数量与关键条目 | 目标页面行为；运行时 307 响应 |
| 收件箱 e2e | `frontend/e2e` 中无收件箱页面用例 | — | 端到端收件箱行为 |
| 私聊面板 | `frontend/src/components/chat/DmCenterPanel.test.tsx` (12 个) | 状态条、新建会话、发送、未读、已读回执、排序 | 真实网关 |
| 私聊本地模型 | `frontend/src/lib/chat/dmStore.test.ts` (20 个) | 会话、发送、未读、网关绑定与回退、持久化 | 服务端 |
| IM 网关客户端 | `frontend/src/lib/chat/imGateway.test.ts` (30 个) | 帧形状、状态机、退避、心跳、待发队列、分发 | 与真实 IM1.0 服务端联调 |
| 通知与评论 REST | `crates/star-api-rest/tests/notifications_endpoints.rs` (12 个 `#[tokio::test]`，经 `router_with_state` 隔离) | 创建、列表、计数、标记已读、404、校验、评论 CRUD、删除权限 | 服务进程组合；认证；租户隔离 |
| IM REST | `crates/star-api-rest/tests/im_endpoints.rs` (11 个) | 会话、消息、已读、成员校验 | 服务进程组合 |
| 领域通知 | `crates/domain-notification/src/lib.rs:1049-1050` 起 (8 个 `#[test]`)；`crates/domain-notification/tests/integration_test.rs` (21 个 `#[tokio::test]`) | INV-N-01、03、04、07；突破分类；未读判据；终态；角色权限 | 持久化；重试；渠道发送；worker |
| 领域调用方 | `crates/domain-work-item/tests/cross_saga_test.rs:27` | 仅引用错误类型 | 派发链路 |

已验证: 收件箱页面与客户端契约 (MSW 拦截)；领域层规则 (单元与集成测试)；REST 契约形状 (隔离 router)；私聊本地模型。

未验证: 服务进程中的通知端到端路径 (G-01)；业务事件产生通知的链路 (G-05)；与真实 IM1.0 服务端的联调 (G-13)；REST 层的租户与本人隔离 (G-03)。

## 8. 来源与裁决

| 源文档或代码 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-INBOX-001.md` (v1.1，2026-11-15) | 6 域聚合、设计要点、FR-IN-001 至 004、文件映射 | 数据源写为 "store (seed)"，代码为真实 API (`page.tsx:19-27`)；FR-IN-001 至 004 标 ✅，本文件对应项为 🟡 | 状态以代码为准，编号以本文件 FR-15 为准 |
| `docs/specs/domain-notification-spec.md` (Draft v0.1) 与 §6 补充 | 实体、INV-N-01 至 07、端口、错误码、RLS、AC、附录 A 时序 | spec 写 NATS 订阅、Pending → Sent → Delivered、重试与 DLQ、`N-001` 至 `N-005` 错误码、`/v1` 路径；代码为同步派发、仅 Sent 与 Read、错误为 `NotificationError` 枚举 | 领域规则 INV-N-01、03、04、07 采用代码；生命周期、重试、DLQ、渠道作为设计意图标 ⬜；错误码不采用 |
| `docs/prototypes/3 · 收件箱-html/README.md` 与 `Inbox.dc.html` | 三栏布局、四个类别标签、列表字段、详情、"在 Branch 中打开"、"标为已读" | 设计稿左栏为项目分组，实现为全局侧栏；设计稿只有单向"标为已读"，实现支持切换；示例数据 | 视觉与交互参考；项目分组保留为 ⬜ (FR-15-29)；示例数据不作为行为依据 |
| `docs/api-design.md` §3.16 (行 451-453)、§3.23 (行 483) | domain-notification 端点与字段；Feedback Inbox 投影 | 设计路径为 `/v1/notification-*` 且含 recipient_user_id、payload、sent_at；代码为 `/api/v1/notifications`，字段不同 | 语义采用；路径与字段以代码为现状，差异记入 G-02；§3.23 属 feedback 功能，本文件不展开 |
| `docs/data-design.md` §4.15 (行 2361 起) | notification_channel、notification_template 的 DDL，RLS 策略 | 代码无 PG 实现；§4.15.3 notification 表的字段未逐一核对 | 作为设计意图标 ⬜；§4.15.3 字段标 ❓ |
| `docs/basic-design.md` (行 227 图；行 273 domain 状态表) | 域职责，notification 为 "邮件 / 站内通知发送" | 行 273 标 ✅，代码中站内仅为进程内存，邮件无实现 (FR-15-18) | 行 273 的 ✅ 与代码不符，不采用 |
| `docs/architecture/2026-11-14-backend-support/WBS-BACKEND-SUPPORT-001.md` (行 16、30) | 早期快照：`/api/notifications` 404 仅 MSW；W1.1 🟡 | 为 2026-11-14 快照，之后已有 v1 路由代码 | 仅作历史背景 |
| `docs/architecture/msw-real-mode.md` (行 93、177) | 记录 inbox 未 real-mode 化 | 页面已直接 realFetch (`page.tsx:19-27`)，与该记录不一致 | 以代码为准 |
| `docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md` (行 26、97) | "IM1.0 改造 ✅"、"IM1.0 chat 域模型" | 前端域模型存在；服务端网关不在本仓库，未核实连通 | 不采信为服务端连通的证据 |
| `docs/frontend/design/ui-3pane-arch.md` (行 18) | 全局底部组含收件箱 | 与 `registry.ts:171-182` 一致 | 采用 |
| `docs/features/F15-inbox-notification-im.md` (同路径既有文档，基线 cdd00a78，未入库) | 同功能的既有整理：FR-15-01 至 21、G-15-01 至 13、§7 测试清单、§4 数据模型 | 行号以旧基线为准；其结论 (服务进程不挂载通知路由、领域无调用方、私聊本地存储、铃铛与计数硬编码、`MAX_RETRY` 未使用、`type` 参数未读取) 经 1643559f 工作树逐条复核一致；新增项 (`subject_url` 协议、DmCenterPanel 断连、IM 默认端口与路径、`/notification` 遮蔽、e2e 注释漂移) 为本基线新增 | 结论采用；行号以本文件为准 |
| `docs/data-design/ipa-detail/tables/feedback_feedback_inbox_item.md`；`docs/architecture/2026-09-22-*`；`docs/architecture/domain-local-runtime.md` | 通过 grep 命中 | 属于 feedback 与其他功能 | 不采用 |
| `docs/reports/`、`docs/briefs/ulys-190-*` | 通过 grep 命中 | 非主源，未作为依据 | 不采用 |
| 前端代码 (`frontend/src/app/(app)/inbox/` 与其测试；`lib/notifications/notificationsApi.ts`；`types/notifications.ts`；`mocks/real-mode.ts`；`mocks/handlers/inbox.ts` 等 mock 文件；`app/api/notifications/*`；`app/notification/page.tsx`；`app/page.tsx`；`lib/redirects.ts`；`redirects.shim.cjs`；`next.config.js`；`lib/nav/registry.ts`；`components/Sidebar.tsx`；`AppHeader.tsx`；`AppShell.tsx`；`RightChatRail.tsx`；`DmCenterPanel.tsx` 与测试；`lib/chat/dmStore.ts` 与测试；`imGateway.ts` 与测试；`types/imGateway.d.ts`；`components/status/StatusNotice.tsx`；`lib/store.ts`；`app/(app)/layout.tsx`；`e2e/redirects.spec.ts`；`app/(app)/__tests__/panels.test.tsx`) | 页面、客户端、类型、路由、组件、测试 | 与 SRS、spec、prototype 的差异见上 | 代码为现状 |
| 后端代码 (`crates/star-api-rest/src/routes/notifications.rs`、`routes/im.rs`、`lib.rs`、`main.rs`、`group_api.rs`、`middleware/auth.rs`、`tests/notifications_endpoints.rs`、`tests/im_endpoints.rs`；`crates/domain-notification/` (Cargo.toml、lib.rs、tests)；`crates/domain-work-item/Cargo.toml` 与测试；`crates/star-mcp/src/handlers/notification.rs`；`crates/star-dispatcher/src/sa_real_impls.rs`；`db/migrations` 检索结果) | 实现、挂载、测试 | 与 spec 与 §4.15 的差异见上 | 代码为现状 |
| 外部仓库 `D:/IM1.0` (只读): `crates/im-gateway/src/main.rs`、`crates/im-gateway/src/http/mod.rs`、`crates/im-protocol/src/ws_frames.rs`、`.worktrees/reg-it/tests/README.md`、`.worktrees/reg-it/docs/Observability.md` | 网关默认端口与路径；协议枚举变体数 (客户端 8、服务端 11) | 客户端默认 9001，外部记录为 8080 或 18080；路径 `/v1/ws` 与嵌套说明不一致；`Observability.md:51` 将 9001 记为 im-router gRPC 端口 | 变体数采用；端口与路径列为 G-13 ❓，不作为本仓库验收依据 |
