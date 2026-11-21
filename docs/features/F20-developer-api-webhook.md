# F20 开发者 API 与出站 Webhook

> 状态摘要: 开发者控制台 (`/settings/developer`) 的 API 参考与 Webhook 本地管理已实现; 后端 Webhook 端点 CRUD 与投递记录接口为进程内存态且生产挂载时无鉴权。出站 HTTP 投递、HMAC-SHA256 出站签名、重试驱动、API Key 鉴权/限流/审计均未实现, 控制台数据与后端端点互不相通。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 为集成开发者与外部自动化客户端提供 Developer API: REST 资源路径镜像 star-mcp 的 16 个工具, 统一 6 字段错误模型与 `data`/`meta` 响应封装。
- 提供出站 Webhook 管理: 登记 endpoint、选择订阅事件、测试投递、查询与重放投递记录; 投递以 HMAC-SHA256 签名, 失败按重试策略进入死信。
- 在开发者控制台提供 API 参考与 Webhook 管理, 并从导航注册表与设置页可达。

### 1.2 范围

- 范围内: `crates/star-api-rest` 的镜像路由与 Webhook 管理路由; `crates/star-webhook` 的投递存储、状态与重试类型; 前端开发者控制台页及其入口。
- 范围外: 入站 SCM Webhook 接收与验签 (`crates/star-webhook/src/signature_verify.rs` 与 `docs/architecture/2026-08-26-upgrade/spec/services/03-webhook-adapter-spec.md` §0 所述的 vendor 到 STAR 方向); LLM 提供商凭据 (`/settings/api-keys`); Google 登录与 JWT 签发 (`crates/star-api-rest/src/auth/google.rs`); OAuth2 服务端与设备注册 (`crates/star-api-rest/src/auth/oauth`, 未挂入生产入口)。

## 2. 用户与场景

| 角色 | 主要场景 | 入口 | 现状 |
|---|---|---|---|
| 集成开发者 | 查看 Developer API 参考; 登记接收事件的出站 Webhook; 测试 endpoint | `/settings/developer` | 🟡 页面可用, 数据仅存于浏览器 (`frontend/src/app/(app)/settings/developer/page.tsx:118-210`) |
| 外部自动化或 AI 客户端 | 以 API Key 调用镜像 REST 路径, 查询工作项、代码符号、流水线等 | `/api/v1/*` | ⬜ 无 API Key 鉴权 (`crates/star-api-rest/src/middleware/auth.rs:23-32`) |
| 集成开发者 (投递排障) | 查看投递记录, 重放失败投递 | `/api/v1/webhooks/deliveries*` | 🟡 接口存在, 重放不写回状态 (`crates/star-api-rest/src/routes/webhooks.rs:309-334`) |

角色划分依据: `docs/features/F17-settings-capabilities.md:40` 将 "集成开发者" 定义为查看 Developer API 参考与登记出站 Webhook 的角色。

## 3. 需求

优先级: P1 为 AGENTS.md §8 v0.34 记录的 MVP 决策范围 (REST 与 MCP 双协议、出站 Webhook、API Keys 与 Webhooks 双 tab); P2 为代码注释标注的后续阶段 (`crates/star-api-rest/src/lib.rs:66`); P3 为 Phase 2+ 评估项 (`lib.rs:65, 67`)。

证据路径为仓库相对路径; `crates/star-api-rest/src/` 简写为 `rest/src/`, `crates/star-webhook/src/` 简写为 `webhook/src/`。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-20-01 | 控制台提供 Developer API 参考 (REST 端点清单) | P1 | 🟡 | `frontend/src/app/(app)/settings/developer/page.tsx:118-160` 为静态表; 其中 `GET /api/v1/projects` 在 `rest/src` 中无路由注册 (见 4.3) |
| FR-20-02 | REST 镜像 star-mcp 16 个工具的资源路径 | P1 | 🟡 | 16 工具清单 `crates/star-mcp/src/main.rs:53-56`; 路径 `rest/src/lib.rs:207-267`; `code::get_symbol` 与 `find_references` 为占位 (`rest/src/routes/mod.rs:23`); 镜像在生产中无鉴权挂载 (`rest/src/main.rs:164`) |
| FR-20-03 | 统一 6 字段错误模型 | P1 | 🟡 | `rest/src/error.rs:22-37`; 24 个错误码目录的来源 `star_mcp::error_code` 未在本基线核实 (`error.rs:25`) |
| FR-20-04 | 统一 `data` 与 `meta` 响应封装 | P1 | ✅ | `rest/src/response.rs:48-57` |
| FR-20-05 | REST 与 MCP 共用鉴权与网关 | P1 | ⬜ | star-mcp 为独立传输进程, 默认监听 127.0.0.1:8080 (`crates/star-mcp/src/main.rs:45-48`); 基线无共享鉴权实现; 控制台文案称 MCP "经同一网关暴露" (`page.tsx:132-133`) 无代码支撑 |
| FR-20-06 | Bearer API Key 鉴权 | P1 | ⬜ | `rest/src/middleware/auth.rs:23-32` 直通; 设计注释 `auth.rs:6-8`; 基线无 `api_keys` 建表语句 |
| FR-20-07 | API Key scope 校验 (`developer.read`, `developer.write`, `webhook.manage`) | P1 | ⬜ | 仅见注释 `rest/src/middleware/auth.rs:9` |
| FR-20-08 | 每 Key 限流 60 次/分钟, 含 burst | P2 | ⬜ | 设计注释 `rest/src/middleware/rate_limit.rs:6-9`; 直通实现 `rate_limit.rs:21-30` |
| FR-20-09 | API 调用写入 `api_key_audit_log` (T 类, 追加写) | P1 | ⬜ | 设计注释 `rest/src/middleware/audit.rs:6-9`; 直通实现 `audit.rs:20-29` |
| FR-20-10 | 鉴权、限流、审计中间件挂载于生产入口 | P1 | ⬜ | 中间件仅挂在 `build_router` (`rest/src/lib.rs:138-144`); 生产 `main.rs:155-166` 无 `layer` 调用 |
| FR-20-11 | endpoint secret 不在任何响应中回显 | P1 | ⬜ | `rest/src/routes/webhooks.rs:124-127, 167-169, 177` 返回完整 endpoint, 含 `secret` 字段 (`webhooks.rs:34`) |
| FR-20-12 | 服务端生成 endpoint secret | P2 | ⬜ | `create_endpoint` 要求调用方提供 secret (`webhooks.rs:150-155`); 控制台表单无 secret 字段 (`page.tsx:27-34`); 设置页 "生成密钥" 按钮禁用 (`frontend/src/components/settings/IntegrationPanels.tsx:322-329`) |
| FR-20-13 | Endpoint CRUD (list, create, get, update, delete) | P1 | 🟡 | `rest/src/routes/webhooks.rs:116-225`; 路由 `rest/src/lib.rs:268-288`; 存储为进程内 `OnceLock` (`webhooks.rs:98-101`) |
| FR-20-14 | create 校验 url 与 secret 非空 | P1 | 🟡 | `webhooks.rs:144-155` 仅检查非空; 无 URL 格式或协议校验 |
| FR-20-15 | 订阅事件 `event_types` 受事件目录约束 | P1 | 🟡 | 后端按任意字符串存储 (`webhooks.rs:35, 161`); 控制台为固定 7 个事件名 (`page.tsx:165-173`); 投递路径不读取该字段 |
| FR-20-16 | endpoint 启停 | P1 | 🟡 | `active` 字段与更新 (`webhooks.rs:36, 162, 187-192`, `webhooks.rs:83-84`); 控制台仅改本地记录 (`page.tsx:216-218`) |
| FR-20-17 | 测试投递 (test ping) | P1 | 🟡 | `webhooks.rs:232-267` 仅登记一条 `PENDING` 记录, 无 HTTP 请求; 控制台测试只回显文本 (`page.tsx:220-228`) |
| FR-20-18 | endpoint 与投递记录持久化 | P1 | ⬜ | 注释 "Phase II 持久化决策时迁 DB" (`webhooks.rs:41`); 基线无 `webhook_endpoints` 或 `webhook_deliveries` 建表语句 |
| FR-20-19 | 业务事件进入出站投递 | P1 | ⬜ | `star_webhook` 仅被 `webhooks.rs:21` 引用; 通知事件名 (`crates/domain-notification/src/lib.rs:101-118`) 无订阅方 |
| FR-20-20 | 出站 HTTP POST 投递 | P1 | ⬜ | 基线无出站发送代码; `webhook/Cargo.toml:17-21` 无 HTTP 客户端依赖 |
| FR-20-21 | HMAC-SHA256 出站签名 | P1 | ⬜ | `webhook/src/signature_verify.rs:34-55` 仅入站验签; `docs/basic-design.md:427` 描述的出站 HMAC 无对应代码 |
| FR-20-22 | 重试 (默认 3 次, 基准 60 秒指数退避) 与死信 | P1 | 🟡 | 类型存在: `webhook/src/delivery.rs:12-40`; 状态 `DeadLetter` (`webhook/src/lib.rs:12-23`); `mark_*` 转换仅测试调用 (`delivery.rs:76-103`) |
| FR-20-23 | 投递记录查询 (list, get) | P1 | 🟡 | get 返回记录 (`webhooks.rs:285-304`); list 返回空数组与 total 计数 (`webhooks.rs:272-279`) |
| FR-20-24 | 投递重放 | P2 | 🟡 | replay 仅计算并返回 `new_state`, 不写回存储 (`webhooks.rs:320-333`); 注释称真实重投留待 Phase II (`webhooks.rs:312-313`) |
| FR-20-25 | 5 域预置 vendor 模板 (Slack, Teams 等) | P2 | ⬜ | `rest/src/lib.rs:66` 明示未实装, 待 P2 由 5 域 Lead 拍板; 注册表与 i18n 描述含 "5 域预置" (`frontend/src/lib/nav/registry.ts:455`; `frontend/src/lib/i18n/zh-CN.ts:441`), 基线无模板数据 |
| FR-20-26 | 控制台 Webhook 与后端 endpoint 同源 | P1 | ⬜ | 控制台存于 localStorage 键 `star.outbound-webhooks.v1` (`page.tsx:36, 50-57`); 页面无 `fetch` 调用; 后端另有进程内存储 (`webhooks.rs:41-45`) |
| FR-20-27 | 开发者控制台双 tab (API 参考, Outbound Webhooks) | P1 | ✅ | `page.tsx:87-110` |
| FR-20-28 | `?tab=webhooks` 深链直达 Webhook tab | P1 | ✅ | `page.tsx:72-75`; `page.test.tsx:42-46` |
| FR-20-29 | 右上角用户菜单入口 | P1 | ⬜ | 用户菜单由 `frontend/src/components/AppHeader.tsx:182` 渲染; `frontend/src/components/UserMenu.tsx` 的链接组 (`:206-253`) 与设置链接 (`:343-352`) 中无 `/settings/developer` |
| FR-20-30 | 导航注册表与设置页入口 | P2 | ✅ | `frontend/src/lib/nav/registry.ts:438-457, 697, 709`; `frontend/src/app/(app)/settings/page.tsx:101-102, 244` |
| FR-20-31 | 控制台 API Keys 页签 (开发者 API Key 发放与撤销) | P1 | ⬜ | 页签仅 Developer API 与 Outbound Webhooks (`page.tsx:98-107`); 决策记录见 `AGENTS.md:464`; `/settings/api-keys` 与 `frontend/src/app/api/api-keys/route.ts:12-16` 管理的是模型提供商凭据 (`frontend/src/app/(app)/settings/api-keys/page.tsx:40-41`) |
| FR-20-32 | 设置页 "API 与 Webhook" 面板 | P2 | 🟡 | 静态预览数据 (`IntegrationPanels.tsx:115-119`, `:316-361`); 文案称 "当前端点表为本地预览数据" (`:319`); 与 `/settings/developer` 为重复入口 |
| FR-20-33 | OpenAPI 3.1 规格自动生成 | P3 | ⬜ | `rest/src/lib.rs:67` (等 v0.2 utoipa); 控制台已知缺口文案 `page.tsx:156` |
| FR-20-34 | OAuth 2.0 授权, mTLS 与 IP 白名单 | P3 | ⬜ | `rest/src/lib.rs:65`; `rest/src/middleware/auth.rs:10` (IP 白名单为 P2) |
| FR-20-35 | 规格原文可追溯 | P1 | ❓ | 规格路径 `docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md` 在基线不存在, 版本库历史中亦无入库记录 (见 §8); 确认方法: 在版本库其他引用或归档中定位该文件并入库 |

## 4. 设计

### 4.1 概念模型

- **Endpoint (订阅端点)**: 外部接收方的 URL 与签名密钥, 带事件订阅列表与启停状态 (`rest/src/routes/webhooks.rs:31-38`)。
- **Event (事件类型)**: 投递的触发源。当前无统一目录: 控制台硬编码 7 个事件名 (`page.tsx:165-173`); `domain-notification` 另有一组通知事件名 (`crates/domain-notification/src/lib.rs:101-118`), 与控制台名称不完全一致 (例如控制台为 `agent.session_failed`, 通知为 `agent_session.failed`)。
- **Delivery (投递记录)**: 一次投递尝试的记录, 字段 `delivery_id`, `provider`, `event_type`, `signature`, `payload`, `state` (`webhook/src/lib.rs:26-40`)。`signature` 字段在测试投递中为占位串 `test-sig-…` (`webhooks.rs:244`)。
- **DeliveryState (投递状态)**: `Pending`, `Delivered`, `Failed`, `DeadLetter` (`webhook/src/lib.rs:12-23`)。
- **Developer API Key (设计项, 未实现)**: 注释定义的 Bearer 凭据与 scope 模型 (`rest/src/middleware/auth.rs:6-11`)。
- **API 调用审计 (设计项, 未实现)**: 注释定义的 `api_key_audit_log` 追加写记录 (`rest/src/middleware/audit.rs:6-9`)。

### 4.2 数据模型

当前数据均为进程内存态, 无持久化:

| 结构 | 字段 | 存储 | 证据 |
|---|---|---|---|
| `WebhookEndpoint` | `id` (UUID 字符串), `url`, `secret`, `event_types`, `active`, `created_at`, `updated_at` (RFC 3339) | `EndpointStore` 内 `HashMap` 包 `Arc<RwLock>`; 进程级 `OnceLock` | `rest/src/routes/webhooks.rs:31-45, 98-101` |
| `WebhookEvent` (投递记录) | `delivery_id`, `provider`, `event_type`, `signature`, `payload`, `state` | `DeliveryStore` 内 `HashMap` 按 `delivery_id` 索引; 进程级 `OnceLock` | `webhook/src/lib.rs:26-40`; `webhook/src/delivery.rs:43-46`; `webhooks.rs:103-106` |
| `RetryPolicy` | `max_retries` (默认 3), `backoff_sec` (默认 60) | 随 `DeliveryStore` 持有 | `webhook/src/delivery.rs:12-28` |
| `OutboundWebhook` (控制台) | `id`, `name`, `url`, `events`, `active`, `created_at` (无 `secret`) | 浏览器 localStorage 键 `star.outbound-webhooks.v1` | `page.tsx:27-36` |

设计项 (未实现): 注释引用 4 张表 `api_keys` (M 类, SCD Type 2)、`api_key_audit_log` (T 类, 追加写)、`webhook_endpoints` (M 类)、`webhook_deliveries` (W 类, 短 TTL), 字段依据 `rest/src/middleware/auth.rs:7-8`, `audit.rs:8-9`, `rest/src/lib.rs:64`。基线中无这些表的建表语句 (⬜)。

### 4.3 接口

Webhook 管理路由在 `build_legacy_mirror_routes` 中注册 (`rest/src/lib.rs:268-301`), 生产入口经 `build_supplemental_routers` 挂载 (`rest/src/lib.rs:182`; `rest/src/main.rs:164`), 挂载前缀为 `/api/v1`。

| 方法与路径 | 处理函数 | 生产鉴权 | 现状 |
|---|---|---|---|
| GET `/api/v1/webhooks/endpoints` (可选 `?active=`) | `list_endpoints` (`webhooks.rs:116-128`) | 无 | 🟡 返回完整 endpoint 含 secret |
| POST `/api/v1/webhooks/endpoints` | `create_endpoint` (`webhooks.rs:141-170`) | 无 | 🟡 `url`, `secret`, `event_types` 必填或校验非空 |
| GET `/api/v1/webhooks/endpoints/{id}` | `get_endpoint` (`webhooks.rs:175-183`) | 无 | 🟡 不存在时返回 400 `VALIDATION_FAILED` (`webhooks.rs:178-181`; `error.rs:830`) |
| PATCH `/api/v1/webhooks/endpoints/{id}` | `update_endpoint` (`webhooks.rs:195-206`) | 无 | 🟡 可改 `url`, `event_types`, `active` |
| DELETE `/api/v1/webhooks/endpoints/{id}` | `delete_endpoint` (`webhooks.rs:211-225`) | 无 | 🟡 物理删除 (`webhooks.rs:89-91`) |
| POST `/api/v1/webhooks/endpoints/{id}/test` | `test_endpoint` (`webhooks.rs:232-267`) | 无 | 🟡 登记 `test-{endpoint}-{uuid}` 的 `PENDING` 记录, 无 HTTP 发送 |
| GET `/api/v1/webhooks/deliveries` | `list_deliveries` (`webhooks.rs:272-280`) | 无 | 🟡 `deliveries` 恒为空数组, 仅返回 `total` |
| GET `/api/v1/webhooks/deliveries/{delivery_id}` | `get_delivery` (`webhooks.rs:285-304`) | 无 | 🟡 返回单条记录 |
| POST `/api/v1/webhooks/deliveries/{delivery_id}/replay` | `replay_delivery` (`webhooks.rs:309-334`) | 无 | 🟡 只返回 `previous_state` 与 `new_state`, 不写存储 |

未实现的接口: 出站发送与签名 (⬜, FR-20-20/21); MCP 工具形式的 webhook 管理 (⬜, star-mcp 16 工具清单不含 webhook 相关工具, `crates/star-mcp/src/main.rs:53-56`); API Key 管理接口 (⬜, FR-20-06, 31)。

REST 镜像的 16 个工具资源路径 (`rest/src/lib.rs:207-267`) 与 star-mcp 工具名的对应关系: `get_issue` 到 `/work-items/{id}`, `search_issues` 到 `GET /work-items`, `get_current_task` 到 `/work-items/current`, `get_workspace` 到 `/workspaces/{id}`, `get_worktree` 到 `/worktrees/{id}`, `create_worktree` 到 `POST /worktrees`, `search_code` 到 `/code/search`, `get_symbol` 到 `/code/symbols/{id}`, `find_references` 到 `/code/symbols/{id}/references`, `get_code_context` 到 `/code/context`, `get_context` 到 `/context`, `create_merge_request` 到 `POST /merge-requests`, `request_review` 到 `POST /reviews`, `run_validation` 到 `POST /validations`, `get_pipeline_status` 到 `/pipelines/{id}`, `submit` 到 `POST /submissions`。该对应按路径语义逐一核对, 规格原文 (§2.2) 不可读, 不能确认是否为规格约定的映射。

错误与响应: 失败统一返回 `RestError` 6 字段 (`rest/src/error.rs:22-37`); 校验失败与资源不存在均映射为 `VALIDATION_FAILED`, HTTP 400 (`error.rs:53-62, 830`)。成功响应为 `RestResponse { data, meta }` (`rest/src/response.rs:48-57`)。

### 4.4 状态机与关键流程

**投递状态 (设计)**: 投递记录创建时为 `Pending`; 成功转 `Delivered`; 失败且 `attempt < max_retries` 转 `Failed` 并按退避重投; 超过次数转 `DeadLetter`。转换方法为 `mark_delivered`, `mark_failed`, `mark_dead_letter` (`webhook/src/delivery.rs:76-103`), 重试判定为 `should_retry`, `backoff_for` (`delivery.rs:32-39`)。驱动这些转换的生产代码 ⬜ 未实现; 当前仅 `DeliveryStore::record` 与 `get` 在生产路径调用 (`webhooks.rs:253, 288`)。

**Endpoint 生命周期**: 创建时 `active` 缺省为 `true` (`webhooks.rs:162`); 通过 PATCH 启停; DELETE 为物理删除 (`webhooks.rs:89-91`), 不保留历史。

**测试投递**: POST `/test` 生成 `delivery_id = test-{endpoint}-{uuid}`, 写入 `PENDING` 记录并返回 `inserted` (`webhooks.rs:239-266`); 该记录不会被发送。

**重放**: 读取记录后计算 `new_state = PENDING` 并返回 (`webhooks.rs:320-333`), 存储中的 `state` 不变。

**控制台添加流程**: `name` 与 `url` 去空白后必填, 缺一则提示 "name / url 必填" (`page.tsx:192-196`); 成功后插入列表首位并写入 localStorage (`page.tsx:197-209`); 事件默认选中第一个 (`page.tsx:179`)。

### 4.5 权限与安全

- **现状 (生产)**: Webhook 与镜像路由在生产入口无鉴权 (`rest/src/main.rs:155-166`; `rest/src/lib.rs:180-181` 注释 "41 条无鉴权镜像"); 任何可达的请求方可创建、读取、修改、删除 endpoint 并读取其 secret。
- **设计 (未实现)**: Bearer API Key 认证、scope 校验、每 Key 限流、调用审计与 IP 白名单 (`rest/src/middleware/auth.rs:6-11`; `rate_limit.rs:6-9`; `audit.rs:6-9`)。
- **Secret 处理**: 明文存于进程内存 (`webhooks.rs:34`) 且在 list、create、get 响应中返回 (`webhooks.rs:124-127, 167-169, 177`)。入站验签模块的注释规定错误信息不打印 secret (`webhook/src/signature_verify.rs:9-10`), 该规则不适用于出站路径 (未实现)。
- **控制台**: 页面无角色判断, 对所有进入 `(app)` 布局的用户渲染 (`page.tsx` 全文); 表单不收集 secret (`page.tsx:27-34`)。
- **测试投递的 signature**: 占位串 (`webhooks.rs:244`), 不是 HMAC 结果。

### 4.6 依赖关系

- `rest` 依赖 `star-webhook` 获取 `DeliveryStore`, `RetryPolicy`, `WebhookEvent`, `WebhookDeliveryState` (`rest/Cargo.toml:103-104`; `rest/src/routes/webhooks.rs:21`)。`rest/Cargo.toml:45-47` 声明的 `sha2`, `hex` 在 `webhooks.rs` 中未使用。
- `star-webhook` 依赖 `hmac`, `sha2`, `hex`, `thiserror`, `tokio` (`webhook/Cargo.toml:17-21`); 入站验签使用 `hmac` 与 `sha2` (`webhook/src/signature_verify.rs:13-15`)。
- 前端页面依赖 `PageHeader` 组件 (`page.tsx:25`), `lucide-react` 图标 (`page.tsx:24`), `next/navigation` 的 `useSearchParams` (`page.tsx:23`); 不依赖任何 API client。
- 浏览器到后端的 `/api/v1/*` 经 Next rewrite 代理到 `STAR_API_BACKEND_ORIGIN` 或 `http://127.0.0.1:8081` (`frontend/next.config.js:46-57`), 控制台未使用该代理访问 Webhook 接口。
- 导航与设置入口依赖 `frontend/src/lib/nav/registry.ts` 与 `frontend/src/app/(app)/settings/page.tsx`。

### 4.7 页面设计

#### 4.7.1 /settings/developer

- **文件**: `frontend/src/app/(app)/settings/developer/page.tsx`
- **布局**: 顶栏 `AppHeader` (`frontend/src/components/AppShell.tsx:49`, 由 `frontend/src/app/(app)/layout.tsx:43-49` 提供); 侧栏: 无 (`AppShell.tsx` 不渲染侧栏节点); 主区 `main` (`AppShell.tsx:61`) 内容器 `max-w-5xl` (`page.tsx:78`); 右侧栏 `RightChatRail` (`AppShell.tsx:77`); 底栏: 无。
- **组件树**: `DeveloperSettingsPage` (`page.tsx:63`) 包裹 `Suspense` (`:66`) → `DeveloperSettingsInner` (`:72`) → `PageHeader` (`:79`), tab 按钮组 (`:87-108`), `ApiTab` (`:118`) 或 `WebhooksTab` (`:175`)。

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 进入页面, 切换 tab | API 表 (`page.tsx:135-154`) 或 webhook 列表 (`:279-315`) | ✅ 本地渲染 (`page.tsx:110`) |
| 加载 | 挂载后读取 localStorage | 无加载指示; 首帧为空列表后替换 (`page.tsx:176, 183-185`) | 🟡 无加载态 |
| 空 | 无 webhook | "暂无 Webhook …" 文案 (`page.tsx:316-320`) | ✅ |
| 错误 | localStorage 解析或写入失败 | 静默回退为空数组或忽略 (`page.tsx:44-47, 52-55`), 无错误提示 | 🟡 静默 |
| 未授权 | 无身份或非管理角色访问 | 无判断 | ⬜ |
| 未接入 | 投递能力与后端端点未接入 | 测试结果文案 "真实投递待后端出站 Webhook 接入" (`page.tsx:226`); API 页 "已知缺口" 文案 (`page.tsx:156`); 添加成功提示为 "本地存储" (`page.tsx:209`) | 🟡 仅文案, 无统一未接入标识 |
| 禁用 | 停用 webhook | 切换 `active` 翻转本地记录 (`page.tsx:216-218`); 无禁用态控件 | 🟡 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换 Developer API / Outbound Webhooks | 无 | 切换 tab (`page.tsx:98-110`) | 组件 state; URL 不随 tab 更新 (`page.tsx:74-75` 仅读取初值) | 🟡 |
| 添加 Webhook | name 与 url 非空 (`page.tsx:193-196`) | 列表首位插入, 显示提示 (`page.tsx:197-209`) | localStorage `star.outbound-webhooks.v1` (`page.tsx:36, 50-53`) | 🟡 不调用 POST `/api/v1/webhooks/endpoints` |
| 选择事件 | 无 | 切换事件数组 (`page.tsx:254-271`) | 组件 state | ✅ 固定 7 项 |
| 启用或停用 | 记录已存在 | 翻转 `active` (`page.tsx:216-218`) | localStorage | 🟡 不同步后端 |
| 测试 | 记录已存在 | 回显 demo 文本 (`page.tsx:220-228`) | 无 | 🟡 不发送 |
| 删除 | 记录已存在 | 从列表移除 (`page.tsx:212-214`) | localStorage | 🟡 不同步后端 |
| 查看 API 参考 | 无 | 静态表 (`page.tsx:135-154`) | 无 | 🟡 含无路由条目 |

数据: 数据源为 localStorage 与组件内静态数组 (`page.tsx:119-128` 端点表, `:165-173` 事件名)。不使用 REST 或 MSW mock。localStorage 记录字段为 `id`, `name`, `url`, `events`, `active`, `created_at` (`page.tsx:27-34`)。

与文档设计的差距:
- 控制台数据与后端 endpoint 不同源: 违反 FR-20-13 与 FR-20-26。
- 表单无 secret 字段, 后端 create 必填 secret (`webhooks.rs:150-155`): 控制台建立的记录在后端 create 会被拒绝, 违反 FR-20-12。
- 无 API Keys 页签: 违反 FR-20-31。
- API 参考表含无路由的 `GET /api/v1/projects` (`page.tsx:123`): 违反 FR-20-01。
- 测试仅回显, 无真实投递: 违反 FR-20-17, FR-20-20。
- 固定 7 个事件名, 无事件目录校验: 违反 FR-20-15。
- 页面无权限判断: 违反 FR-20-10 所述的生产鉴权要求, 对应 FR-20-06。
- 右上角用户菜单无入口: 违反 FR-20-29。

#### 4.7.2 /settings?tab=webhooks

- **文件**: `frontend/src/app/(app)/settings/page.tsx` (`activeId === "webhooks"` 分支, `:234-236`), 面板 `frontend/src/components/settings/IntegrationPanels.tsx:316-361`。
- **布局**: 顶栏 `AppHeader` (`AppShell.tsx:49`); 侧栏: 设置页左导航 `aside` (`settings/page.tsx:175`); 主区 `main` (`settings/page.tsx:201`); 右侧栏 `RightChatRail` (`AppShell.tsx:77`); 底栏: 无。
- **组件树**: `SettingsPage` (`settings/page.tsx:321`) → `SettingsInner` (`:162`) → `IntegrationPanels` (`IntegrationPanels.tsx:540`) → `WebhooksPanel` (`:316`) → `PanelIntro` (`:204`), `NotConnectedBadge` (`:150`)。

状态矩阵 (加载、空、错误不适用: 数据为静态常量):

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 打开 `?tab=webhooks` | 4 列预览表, 3 行静态 endpoint (`IntegrationPanels.tsx:336-360`) | 🟡 静态预览 |
| 未授权 | 无身份访问 | 无判断 | ⬜ |
| 未接入 | 密钥服务未接入 | "密钥管理服务未接入 — 接入后启用生成/轮换" 与未接入徽标 (`IntegrationPanels.tsx:330-333`) | ✅ |
| 禁用 | 生成密钥 | 按钮 `disabled` (`IntegrationPanels.tsx:322-329`) | ✅ |

交互: 仅 "生成密钥" 按钮, 处于禁用态; 无其他交互。

数据: 静态 `WEBHOOK_ENDPOINTS` (`IntegrationPanels.tsx:115-119`), 三条记录的路径为 `/api/v1/webhooks/workitems/status`, `/api/v1/webhooks/sprints/updated`, `/api/v1/webhooks/deliveries`。前两条在 `rest/src/lib.rs:268-301` 中无路由; 第三条的 GET 路由存在。

与文档设计的差距:
- 静态预览与后端路由不一致: 违反 FR-20-01, FR-20-32。
- 与 `/settings/developer` 为重复入口, 功能分裂: 违反 FR-20-26。
- 密钥生成被禁用: 违反 FR-20-12。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| Webhook endpoint 管理 (进程内) | `crates/star-api-rest/src/routes/webhooks.rs` | 🟡 | `:27-106` 存储, `:115-225` CRUD |
| Webhook 路由挂载 (生产) | `crates/star-api-rest/src/lib.rs`; `src/main.rs` | 🟡 | `lib.rs:165-183, 268-301`; `main.rs:164` |
| 镜像 REST 路由 (16 工具路径) | `crates/star-api-rest/src/lib.rs` | 🟡 | `lib.rs:205-267` |
| 统一错误与响应 | `crates/star-api-rest/src/error.rs`; `src/response.rs` | ✅ | `error.rs:22-62, 830`; `response.rs:48-57` |
| 投递存储与重试类型 | `crates/star-webhook/src/delivery.rs`; `src/lib.rs` | 🟡 | `delivery.rs:12-108`; `lib.rs:12-40` |
| 出站 HTTP 投递与签名 | 无 | ⬜ | 基线内无对应代码 (见 FR-20-20, 21) |
| 鉴权、限流、审计中间件 | `crates/star-api-rest/src/middleware/` | 🟡 | 直通实现 `auth.rs:23-32`, `rate_limit.rs:21-30`, `audit.rs:20-29`; 仅挂在 `build_router` (`lib.rs:138-144`) |
| API Key 模型与管理 | 无 | ⬜ | 基线无 `api_keys` 建表语句与对应接口 |
| 开发者控制台页 | `frontend/src/app/(app)/settings/developer/page.tsx` | 🟡 | `:63-340`; 本地 CRUD 完整, 与后端未对接 |
| 设置页预览面板 | `frontend/src/components/settings/IntegrationPanels.tsx` | 🟡 | `:316-361` |
| 导航与设置入口 | `frontend/src/lib/nav/registry.ts`; `frontend/src/app/(app)/settings/page.tsx` | ✅ | `registry.ts:438-457`; `settings/page.tsx:101-102, 244` |
| 右上角用户菜单入口 | `frontend/src/components/UserMenu.tsx` | ⬜ | `:206-253, 343-352` 无开发者入口 |
| `/api/v1` 同源代理 | `frontend/next.config.js` | ✅ | `:46-57` (依赖, 非本功能专属) |
| 入站 SCM 验签 (范围外参照) | `crates/star-webhook/src/signature_verify.rs` | ✅ | `:34-69, 81-98` |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 生产挂载的 Webhook 与镜像路由无鉴权 (FR-20-06, 10) | 任何可达请求方可管理 endpoint 并读取 secret | 在生产入口为这些路由挂载鉴权层, 或在鉴权实装前不挂载 `build_supplemental_routers` 中的 webhook 与镜像路由 |
| 鉴权、限流、审计为直通 stub 且未挂载生产 (FR-20-07, 08, 09) | 无 per-key 限流与调用审计 | 实装三层中间件并挂入生产入口 |
| secret 明文存储并回显 (FR-20-11) | 管理端读取即泄露签名密钥 | 响应脱敏; 服务端生成 secret; 存储加密 |
| 无出站 HTTP 投递与 HMAC 签名 (FR-20-20, 21) | Webhook 无法实际送达; `basic-design.md:427` 的描述与代码不符 | 实装 sender (可用 star-api-rest 已声明的 `reqwest` 依赖或 star-webhook 新增依赖); 按 `HMAC-SHA256(secret, body)` 签名 |
| 投递状态转换无驱动 (FR-20-22) | 重试与死信类型存在但不运行 | 投递循环调用 `mark_*` 并按 `RetryPolicy` 退避 |
| 重放不写回, 列表恒空 (FR-20-23, 24) | 投递排障不可用 | `DeliveryStore` 增加列表与写回接口, `replay` 写回并调度 |
| 业务事件无订阅方 (FR-20-19) | 订阅配置不产生任何投递 | 定义事件目录, 从事件总线订阅并生成投递记录 |
| 存储为进程内 (FR-20-18) | 重启丢失 endpoint 与记录 | 建 `webhook_endpoints`, `webhook_deliveries` 表, 按 W/T/M 分类 (规格原文不可读, 分类依据待核实) |
| 控制台与后端两套数据 (FR-20-26) | 控制台登记的 webhook 不生效 | 控制台改调 `/api/v1/webhooks/endpoints`, 移除 localStorage 副本 |
| 控制台表单无 secret, 后端必填 (FR-20-12) | 控制台创建的记录在后端被拒绝 | 表单增加 secret 或改为服务端生成 |
| 资源不存在返回 400 (`webhooks.rs:178-181`; `lib.rs:709` 测试断言) | 客户端无法区分 "不存在" 与 "校验失败" | 增加 `NOT_FOUND` 映射; 与规格 §2.4 对照 (规格不可读, ❓) |
| 无 URL 格式与协议校验 (FR-20-14) | 可登记无效或非 HTTPS 目标 | 增加格式与协议校验 |
| 事件类型无目录 (FR-20-15) | 控制台与通知事件名不一致 | 统一事件目录, 后端校验 `event_types` |
| API 参考含无路由条目 `/api/v1/projects` (FR-20-01) | 文档误导 | 删除该行, 或补充路由; 长期由路由清单生成 (FR-20-33) |
| "MCP 经同一网关暴露" 无代码支撑 (FR-20-05) | 文案与实现不符 | 改写控制台文案, 或实现 REST 与 MCP 的统一网关 |
| 无 API Key 发放与撤销 (FR-20-06, 31) | 外部客户端无法获得 Developer API 凭据 | 建 `api_keys` 表与发放撤销接口, 控制台增加 API Keys 页签 |
| 右上角用户菜单无入口 (FR-20-29) | 与 AGENTS.md §8 v0.34 决策不符 | 在 `UserMenu.tsx` 增加 `/settings/developer` 链接 |
| 设置页重复入口与静态面板 (FR-20-32) | 两处数据不一致, 用户困惑 | 统一指向 `/settings/developer`, 移除或重定向预览面板 |
| 5 域预置 vendor 模板缺失 (FR-20-25) | 出站模板不可用 | P2: 待 5 域 Lead 拍板模板内容后实装 |
| 生产组合中 legacy 镜像与 `build_scm_router` 对同一路径重复注册 (❓) | `POST /api/v1/merge-requests`, `POST /api/v1/reviews`, `GET /api/v1/pipelines/{id}` 同时出现在 `lib.rs:262-266` 与 `routes/merge_requests.rs:273`, `reviews.rs:224`, `pipelines.rs:108`; `main.rs:164-166` 同时合并两者; 运行时是否冲突未核实 (❓) | 以 `build_group_router`, `build_supplemental_routers`, `build_scm_router` 的生产组合构造 Router 并观察是否 panic; 确认后去除重复注册 |
| 规格原文缺失 (FR-20-35) | 规格 §1.1, §1.3, §2, §5, §6, §7 的引用无法核对 | 定位或撰写规格原文并入库 |

## 7. 验收与测试

已存在的测试 (由基线源码阅读确认; 本文未运行测试):

| 测试 | 位置 | 覆盖 |
|---|---|---|
| Webhook endpoint CRUD 往返 | `crates/star-api-rest/src/lib.rs:622-710` (`webhook_endpoints_crud_roundtrip_after_phase_i_batch_3`) | POST 200, GET 200, PATCH 200, DELETE 200, 删除后 GET 400 |
| 路由构建烟雾测试 | `crates/star-api-rest/src/lib.rs:459-462` | 仅构建 `build_router`, 不断言路径 |
| 生产挂载可达性 | `crates/star-api-rest/tests/production_router.rs:111-126` | `/api/v1/webhooks/endpoints` 与 `/deliveries` 非 404; 请求携带 JWT, 但测试不断言鉴权 |
| 投递存储与重试类型 | `crates/star-webhook/src/delivery.rs:147-200` 起 (8 个用例) | 幂等 record, `mark_*` 状态转换, 重试默认值与退避 |
| 入站验签 | `crates/star-webhook/src/signature_verify.rs` (7 个用例) | 仅 GitHub, GitLab 入站验签 (范围外) |
| 中间件直通 | `crates/star-api-rest/src/middleware/auth.rs:34-64`; `audit.rs:31-55`; `rate_limit.rs:32-56` | 仅断言直通, 不断言安全行为 |
| 控制台页 | `frontend/src/app/(app)/settings/developer/page.test.tsx:35-88` (4 个用例) | 默认 API tab, `?tab=webhooks` 深链, CRUD 写入 localStorage, 测试回显 demo |
| 设置页 Webhook 面板 | `frontend/src/components/settings/IntegrationPanels.test.tsx:101-107` | 面板渲染与 3 个端点行 |
| 导航链接有效性 | `frontend/src/lib/nav/registry.integrity.test.ts:10-13` | 导航 href 指向真实页面, 含 `/settings/developer` |

已验证 (源码阅读): 路由注册与挂载路径; 错误模型字段; 控制台双 tab 与本地 CRUD; 导航与设置入口。

未验证或无测试: 出站 HTTP 投递; HMAC 签名; 重试驱动; 重放写回; list 投递记录; secret 脱敏; 生产鉴权; API Key 发放与校验; 限流与审计行为; 控制台与后端同步; API 参考路由一致性; 生产组合中重复注册的运行时结果 (❓)。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md` (种子源) | 规格原文. 代码注释引用其 §1.1, §1.3, §1.4, §2, §2.2 至 §2.4, §3.7, §4, §5.1, §6, §7.2, §7.3 | 路径在基线不存在, 版本库全部引用中亦无入库记录 (`git log --all` 对路径无结果); 其他文件与代码注释均引用它 | 无法读取, 不作为依据; 规格相关条目以代码与可读源为准, 无法核实者标 ❓ (FR-20-35) |
| `AGENTS.md` §8 v0.34 | 三项决策: REST 与 MCP 双协议; 出站 Webhook 与 5 域预置 vendor; API Keys 与 Webhooks 双 tab MVP; 右上角菜单入口 | 声称的规格文件不存在; 无 API Keys 页签; 无右上角入口 (`UserMenu.tsx`) | 采纳为目标与优先级依据 (§1.1, §3 P1); 实现状态以代码为准 |
| `AGENTS.md` §8 v0.35 | star-api-rest 骨架落地 | `lib.rs:92` 注释仍写 "22 路由 stub", `lib.rs:49` 写 "27 路由真实接线" | 以 `lib.rs` 注册的路由为准 |
| `crates/star-api-rest/src/lib.rs` 头注释与 `build_router` 注释 (`:43-69, 92-115`) | 镜像路由范围; 中间件 stub; 不实装 DB, OAuth2, 5 域模板, OpenAPI | 头注释 "不实装业务逻辑" 与 "27 路由真实接线" 并存 | 路由清单采纳; 业务实现状态以 handler 为准 |
| `crates/star-api-rest/src/routes/webhooks.rs` 头注释与实现 (`:1-10`, `:273`, `:312-313`) | 9 路由, 进程内 EndpointStore, 复用 DeliveryStore; 简化项说明 | 头注释称 "真实接线", 实际为内存态; 简化项已在注释中标注 | 采纳为现状, 以代码为准 |
| `crates/star-webhook/src/lib.rs` 头注释 (`:2-5`) | 定位为 Webhook receiver (入站 SCM) | `docs/basic-design.md:427` 称其含 outbound HMAC 与重试; 代码仅入站验签 | 以代码为准; 出站能力记 ⬜ |
| `docs/basic-design.md:427-428` | star-webhook 的出站描述; star-api-rest 的 Developer REST API 描述 | 出站 HMAC 与重试描述与代码不符 | 不采纳为现状; 记差距 (FR-20-21) |
| `docs/architecture/2026-08-26-upgrade/spec/services/03-webhook-adapter-spec.md` §0, §2 至 §5 | 入站 webhook 的端点, 签名, 去重, 失败与死信设计 | §0 明确限定为入站; `docs/architecture/2026-08-26-upgrade/adr/0034-phase-e-architecture.md:46` 称其为 "出站 Webhook 适配" | 作为入站参照, 不作为出站设计依据; 与 ADR-0034 冲突时以本文件 §0 为准 |
| `docs/architecture/2026-08-26-upgrade/adr/0034-phase-e-architecture.md:46, 93, 108` | 出站 Webhook 触发源候选 (MR 创建, Lease 过期, 冲突, Submit 完成, Context Graph 变更); 标 "未实现" | 与 03 规格的方向定义相反 | 采纳 "出站未实现" 状态; 触发源仅作候选, 未核实 |
| `docs/features/F17-settings-capabilities.md:22, 40, 90-91, 95, 110, 127, 140, 207, 248` | 控制台入口关系; FR-17-42 ✅ (API 参考); FR-17-43 🟡 (本地 CRUD); 页面状态字段与测试数 | 与本文页面行为一致 | 采纳为页面旁证; 需求编号独立 |
| `docs/WBS.md:158` (8.7.4) | 状态 "已完成 100%" | 仅前端本地 CRUD 完成; 后端投递与持久化未完成 | 不采纳 100% 作为功能完成; 记为 UI 完成 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:929-932` | 说明 `/settings/developer` 与 `?tab=webhooks` 的路由补建 | 与代码一致 (页面存在) | 仅作背景, 不作为现状证据 |
| `crates/star-api-rest/Cargo.toml:13, 45-47, 99-104` | 包描述引用规格; HMAC 依赖注释; `reqwest` 与 `star-webhook` 依赖 | 描述引用规格原文 (不可读); `sha2`, `hex` 在 webhooks 中未用 | 采纳依赖事实; 描述不作为依据 |
| `crates/star-api-rest/src/main.rs:155-166` | 生产组合: group, supplemental, scm 路由合并, 无 layer | 与 `lib.rs:138-144` 的 `build_router` 中间件不一致 | 采纳为生产挂载事实 |
| `crates/star-mcp/src/main.rs:45-56` | 16 工具清单与传输模式 | 控制台文案称 "同一网关" 与之不一致 | 采纳工具清单; 不采纳 "同一网关" 描述 |
| `frontend/src/components/settings/IntegrationPanels.tsx:115-119, 316-361` | 设置页预览面板与静态端点 | 端点路径多数无后端路由 | 视为预览 (🟡), 不作为现状 |
| `frontend/src/lib/nav/registry.ts:438-457, 697, 709` 与 `frontend/src/lib/i18n/{en,zh-CN,ja}.ts` 导航描述 | 入口与分组; "5 域预置通知模板" 描述 | 无模板实现 | 采纳入口事实; 模板记 ⬜ |
| `frontend/src/components/UserMenu.tsx`, `AppHeader.tsx:182` | 右上角用户菜单 | 无开发者入口 | 记 ⬜ (FR-20-29) |
| `frontend/next.config.js:46-57` | `/api/v1/*` 同源代理 | 控制台未使用 | 采纳为依赖事实 |
| `docs/data-design.md:2902` (`scm.webhook_event`) | 入站 SCM 事件表 | 属入站范围 | 设为范围外, 不采纳 |
| `docs/poc/poc-026-github-adapter.md:233` | 入站 GitHub 适配 PoC 表 | 属入站范围 | 设为范围外, 不采纳 |
| `docs/architecture/2026-11-14-backend-support/WBS-BACKEND-SUPPORT-001.md:16, 31` | `/api/api-keys` 的 W1.2 补全任务 | 对象为模型提供商凭据, 非 Developer API Key | 设为范围外, 不采纳 |
| `docs/architecture/msw-real-mode.md:86-88` | `/api/api-keys` 的 MSW 路由表 | 同上 | 设为范围外, 不采纳 |
| `docs/architecture/2026-08-26-upgrade/adr/0042-onboarding-first-run.md`, `spec/agent-api/onboarding.md`, `docs/basic-design.md:4122-4166` | 模型提供商 API Key 探测与存储 | 同上 | 设为范围外, 不采纳 |
| `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md:130, 568` | 移动端不产生 webhook; 引用规格 `:46-49` 的 16 工具清单 | 引用的规格行号不可核 | 仅作旁证, 不采纳 |
| `frontend/src/app/(app)/settings/developer/page.tsx` 与 `page.test.tsx` | 控制台实现与测试 | 与 §4.7 一致 | 采纳为现状证据 |
| `crates/star-api-rest/tests/production_router.rs`; `crates/star-api-rest/src/lib.rs` 测试段 | 挂载与 CRUD 测试 | 测试不覆盖 `build_scm_router` 组合 | 采纳为测试证据 |
