# 43. Android Flutter Mobile App — MVP 设计 (Pre-IPA 草稿, SUPERSEDED)

> **状态**: 🟡 草案 v0.1 (**已被 v0.2 IPA 三段組 supersede**, 2026-09-02 16:09 JST)
> **撰写**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02 15:54 JST
> **触发**: Ulysses 2026-09-02 15:52 JST 发令"完成设计文档撰写"(承接上一问"安卓版flutter移动app开发得怎样了?第一版默认在内网使用即可")
>
> **⚠️ SUPERSEDE 通知 (2026-09-02 16:09 JST)**: 本 doc 已被拆分为 3 份 IPA 標準正式設計文書(per 2026-09-02 16:09 JST Ulysses 発令「要符合日本IPA标准的需求、基本设计、详细设计」):
>
> | IPA 文書 | 路径 | 役割 |
> |---|---|---|
> | **要件定義書** v1.0 | `docs/mobile-flutter-mvp/01-requirements.md` | FR-xxx 30 件 + NFR-xxx 22 件 + UC 5 件 + 既知未解決 15 件 |
> | **基本設計書** v1.0 | `docs/mobile-flutter-mvp/02-basic-design.md` | 4 層クリーンアーキテクチャ + 6 機能モジュール + 13 コンポーネント + 状態機 + デプロイ |
> | **詳細設計書** v1.0 | `docs/mobile-flutter-mvp/03-detailed-design.md` | クラス詳細 (Dio Interceptor / Controller / freezed model) + JSON スキーマ + pubspec.yaml + テストケース |
>
> **本 doc (v0.1) 的地位**:
> - 历史溯源记录: Pre-IPA 单 doc 形式, commit `bd4998e`
> - 内容已被 3 份 IPA doc 完全覆盖, 后续实施以 IPA 3 段为准
> - 本 doc 不删, 留作 v0.1 → v1.0 演进的考古证据 (per 守门 #1 禁回溯叙事)

## §0 目的

### 0.1 立项背景

STAR 仓当前 V1 范围**显式排除**移动 App:
- `docs/internal-design.md:50` — `❌ 移动 App(V2)`
- `docs/internal-design.md:1633` — `Internal-J.7:是否上 React Native(移动)?**V2 候选**`
- `docs/internal-design.md:1600` — `V2 任务:移动端 / 离线 / 实时协作光标`

Ulysses 2026-09-02 15:52 JST 显式发令**新开一条 MVP 路径**,脱离 V2 React Native 排期,改用 **Flutter**(选型理由见 §3.1),并指定**第一版默认内网使用**。

**本 spec 定义 MVP 范围、端点映射、Flutter 端架构、鉴权/缓存/部署策略**。V2 移动范围不在本文档覆盖范围(走 `docs/internal-design.md:1600` 原路径,届时回填)。

### 0.2 与既有 spec 的关系

| 既有 spec | 本 spec 关系 |
|---|---|
| `docs/api-design.md` v0.2 | **端点清单来源**(§3.5 work-items / §3.7 board / §3.16 notification / §1.8 tenant_id / §1.12 鉴权分层) |
| `docs/internal-design.md` v?.? | V1 范围基线,本 spec 是"显式扩出 V1 边界的子集" |
| `docs/data-design.md` v?.? + `00-CLASSIFICATION-W-T-M.md` v0.1 | DB 表 W/T/M 横展开基线,本 spec 不直接落表,只消费 REST |
| `docs/architecture/2026-08-26-upgrade/adr/0021-zero-vendor-cooperation.md` | 不得引入厂商绑定 SDK(FCM / 友盟 / 极光等 push 服务均违反),通知走自建轮询 + 拉模型 |
| `docs/architecture/2026-08-26-upgrade/adr/0026-star-ai-compat.md` | 5 通道 + Fallback Ladder:本 spec MVP 不涉及 AI 通道,只走 REST 一条 |
| `docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md` v0.1 | **路径前缀/鉴权/限流**统一规则来源(§1.1 REST 形态) |
| `AGENTS.md` §4 守门 | 8 项硬约束 + 24 项派生规(本 spec §5 适配) |

### 0.3 MVP 边界(per 2026-09-02 15:54 JST 默认拍板)

**默认假设**(本 spec 自定,Ulysses 后续可推翻):

| 维度 | 选择 | 理由 |
|---|---|---|
| **方向** | 新开 Flutter MVP(独立仓库 `apps/star-mobile-flutter/`,或在 STAR 仓 `frontend/mobile-flutter/` 下,**待 Ulysses 拍仓 vs 仓**) | "第一版默认内网使用即可"指向轻量新开 |
| **MVP 范围** | **只读**:登录 + 看板视图 + 工作项详情 + 通知中心 | 内网轻量场景下,先解决"看一眼"问题 |
| **网络/部署** | **直连 STAR API,纯 HTTP 内网**(不走 envoy + HTTPS) | "内网使用即可"指向最低 ops 成本;后续如需走公网/对外再升级 |
| **平台** | **仅 Android** (iOS 暂不做) | 题面指定"安卓版" |
| **API 协议** | REST(`/api/v1/*`),不走 MCP | api-design.md §1.1 显式"传统 SaaS 集成 / 移动端 / curl → REST" |
| **离线** | **不做**(MVP 纯 online) | 离线留 V1.1(per §9 已知缺口 G-04) |
| **推送** | **不做**(走轮询,30s) | 推送需 FCM,违反 ADR-0021 零厂商合作(per §9 已知缺口 G-05) |
| **Tablet 适配** | **不做**(只做 phone portrait) | MVP 范围内显式排除(per §9 已知缺口 G-06) |

**默认假设未被 Ulysses 显式确认**:Mavis 接手按"完成设计文档撰写"指令直接落档,后续可整段改写。

---

## §1 协议矩阵(MVP)

### 1.1 通讯形态

| 维度 | 选择 | 引用 |
|---|---|---|
| **协议** | HTTP/1.1 + JSON (REST) | api-design §1.1 |
| **路径前缀** | `http://<STAR_HOST>/api/v1/*` (MVP 直连,**待 STAR_HOST 拍板**) | api-design §1.1 + §1.10 URL versioning |
| **鉴权** | `Authorization: Bearer <jwt>` (短效) + `Refresh-Token` header(长效) | api-design §1.12 Authenticated/Policy 级 |
| **Tenant 隔离** | **不传 `X-Tenant-Id` header**(per api-design §1.8 "Header 来源仅由 API Gateway 从 JWT 提取,不接受 query string 或 body 传入") | api-design §1.8 |
| **Content-Type** | `application/json; charset=utf-8` | api-design §1.10 |
| **Trace** | `traceparent` W3C header(SDK 自带) | api-design §1.9 |
| **错误格式** | RFC 7807 Problem Details | api-design §1.3 + §8 |
| **TLS** | **MVP 不上**(明文 HTTP,内网限定) | per §0.3 默认假设;若走公网须升级 HTTPS(per §9 G-08) |

### 1.2 后端基线引用(per 2026-09-02 15:54 JST git 实证)

- `crates/star-mcp/src/main.rs:53-56` — 16 MCP tool 列表(MVP 镜像其中 5 个 tool 的 read 路径到 REST,见 §2.1)
- `docs/api-design.md:3.5` — WorkItem CRUD + transition + bulk
- `docs/api-design.md:3.7` — Board 读取
- `docs/api-design.md:3.16` — Notification 列表 + mark-read

---

## §2 端点映射(MVP 范围)

### 2.1 端点矩阵(13 个,per api-design.md §3 真实路径)

| # | Method | 路径 | 鉴权级 | Flutter 端用途 | 引用 |
|---|---|---|---|---|---|
| 1 | POST | `/v1/auth/login` | Anonymous | 邮箱+密码登录,返 access+refresh token | **待补**:api-design §3.1 隐含,需 §2 引用 domain-identity `POST /v1/auth/login`(per internal-design §23.2 Device 三重绑定,见 §5.2) |
| 2 | POST | `/v1/auth/refresh` | Authenticated | 短效 token 过期,refresh | 同上 |
| 3 | POST | `/v1/auth/logout` | Authenticated | 清除本地凭证 + 通知后端 | 同上 |
| 4 | GET | `/v1/users/me` | Authenticated | 当前用户基本信息(头像/显示名) | api-design §1.12 + §3.2 |
| 5 | GET | `/v1/tenants/current` | Authenticated | 当前 tenant 名(顶部面包屑) | api-design §3.2 |
| 6 | GET | `/v1/projects/{id}/board` | Policy | 看板配置(columns 顺序) | api-design §3.7:668 |
| 7 | GET | `/v1/work-items?project_id=&filter[status]=&sort=-updated_at` | Policy | 看板卡片列表 | api-design §3.5:624 |
| 8 | GET | `/v1/work-items/{id}` | Policy | 工作项详情 | api-design §3.5:626 |
| 9 | GET | `/v1/work-items/{id}/transitions` | Policy | 列可用状态迁移(MVP 只读不调) | api-design §3.5:630 |
| 10 | GET | `/v1/work-items/{id}/comments` | Policy | 评论列表(详情页 Tab) | api-design §3.10:700 |
| 11 | GET | `/v1/notifications?read=false&filter[event_type]=` | Authenticated | 通知中心未读列表 | api-design §3.16:787 |
| 12 | POST | `/v1/notifications/{id}:read` | Authenticated | 标记单条已读 | api-design §3.16:788 |
| 13 | POST | `/v1/notifications/mark-all-read` | Authenticated | 全部已读 | api-design §3.16:789 |

**待补项**:auth 端点(1/2/3) — api-design.md §3.1 显式声明"domain-identity 走 §3.15",但 §3.15 我未在本次扫读中拉到 `/v1/auth/login` 路径(grep 未匹配)。**Mavis 接手备注:这是已知缺口 G-01,落档后须由 Ulysses 拍板后端是否已实现该路径,或本 MVP 是否走 OAuth 2.0 浏览器跳转(per api-design §6.2 G-01 OAuth 2.0 Phase 2+ 候选)**。

### 2.2 端点不在 MVP 范围(显式排除)

| 不做的 | 理由 |
|---|---|
| `POST /v1/work-items`(创建) | MVP 只读 |
| `PATCH /v1/work-items/{id}`(修改) | 同上 |
| `POST /v1/work-items/{id}:transition`(状态流转) | 同上,留 V1.1 |
| `POST /v1/comments`(写评论) | 同上 |
| `POST /v1/work-items/bulk`(批量) | 同上 |
| WebSocket 实时推送(§4) | MVP 走轮询 30s,不走 WS |
| MCP Streamable HTTP | 桌面 AI agent 专用,移动端不暴露 |
| Outbound Webhook(出站) | 移动端不产生 webhook |

### 2.3 分页/排序/过滤约定

- **分页**:cursor-based,`?cursor=<opaque>&limit=20` (api-design §1.4)
- **排序**:`?sort=-updated_at,priority` (api-design §1.5)
- **过滤**:`?filter[status]=TODO,IN_PROGRESS&filter[assignee]=user_xxx&filter[project_id]=prj_yyy` (api-design §1.5)
- **稀疏字段**:`?fields=id,title,status,assignee,priority,updated_at`(给移动端减少 payload)

---

## §3 Flutter 端架构

### 3.1 选型(per 2026-09-02 15:54 JST 默认)

| 维度 | 选择 | 理由 |
|---|---|---|
| **框架** | **Flutter 3.24+ / Dart 3.5+** | 题面指定 + 跨平台潜力(后续 iOS 可复用)+ Material 3 默认适配 |
| **状态管理** | **Riverpod 2.5+** (NOT Provider / Bloc) | compile-time safe + 强类型 + 不依赖 BuildContext,移动端冷启动性能优于 Bloc |
| **网络** | **Dio 5.7+** + `dio_smart_retry` | 拦截器链成熟,refresh token 拦截器好写,retry/backoff 内置 |
| **本地存储** | **flutter_secure_storage 9.x** (token) + **Hive 2.x** (业务缓存) | token 用 Keystore 加密;业务数据非敏感,直接 KV |
| **路由** | **go_router 14+** | 声明式,支持 deep link 后续扩展 |
| **JSON 序列化** | **freezed 2.5+ + json_serializable** | 不可变 model + copyWith + sealed unions(状态机表达) |
| **日期** | **intl** + `timeago` | i18n + 相对时间显示 |
| **测试** | `flutter_test` + `mocktail` + `integration_test` | 单元 + widget + e2e 三层 |
| **Linter** | `very_good_analysis` (lint 规则) + `dart fix` | 严格 lint 守门 |
| **图标** | Material Icons + 少量自定义 SVG(Star logo) | 复用 web 端 logo |
| **国际化** | **MVP 不上 i18n**(中文 only),i18n 留 V1.1 | "内网使用" 限定中文用户群 |

### 3.2 包结构(monorepo 内 `frontend/mobile-flutter/` 假设)

```
frontend/mobile-flutter/
├── pubspec.yaml
├── analysis_options.yaml            # very_good_analysis + custom
├── android/                          # Android-only (iOS 留 V2)
│   ├── app/
│   │   ├── build.gradle.kts         # applicationId = com.star.mobile
│   │   └── src/main/AndroidManifest.xml  # INTERNET + usesCleartextTraffic="true"(MVP HTTP)
│   └── ...
├── lib/
│   ├── main.dart                     # ProviderScope + MaterialApp.router
│   ├── app/
│   │   ├── app.dart                  # MaterialApp.router 入口
│   │   ├── router.dart               # go_router 配置
│   │   └── theme.dart                # Material 3 + 复用 web 端色板
│   ├── core/
│   │   ├── api/
│   │   │   ├── dio_client.dart       # Dio 单例 + baseUrl + 拦截器链
│   │   │   ├── auth_interceptor.dart # Bearer token 注入 + 401 refresh
│   │   │   ├── error_interceptor.dart# RFC 7807 → AppException
│   │   │   └── logging_interceptor.dart # 调试 only,release 关闭
│   │   ├── auth/
│   │   │   ├── token_storage.dart    # flutter_secure_storage 封装
│   │   │   └── auth_state.dart       # Riverpod AuthState (sealed)
│   │   ├── env/
│   │   │   └── env.dart              # const STAR_HOST = 'http://star.internal:8080'(Mavis 接手占位,待 Ulysses 拍)
│   │   └── result/
│   │       └── result.dart           # Result<T, AppError> sealed
│   ├── features/
│   │   ├── auth/
│   │   │   ├── data/auth_repository.dart
│   │   │   ├── domain/user.dart      # freezed model
│   │   │   └── presentation/
│   │   │       ├── login_screen.dart
│   │   │       └── auth_controller.dart
│   │   ├── board/
│   │   │   ├── data/board_repository.dart
│   │   │   ├── domain/{board,column,work_item_summary}.dart
│   │   │   └── presentation/
│   │   │       ├── board_screen.dart     # horizontal scroll columns
│   │   │       ├── board_controller.dart
│   │   │       └── widgets/board_card.dart
│   │   ├── work_item/
│   │   │   ├── data/work_item_repository.dart
│   │   │   ├── domain/work_item.dart     # freezed, 状态机 sealed
│   │   │   └── presentation/
│   │   │       ├── work_item_detail_screen.dart
│   │   │       └── tabs/{overview,comments,transitions}_tab.dart
│   │   └── notifications/
│   │       ├── data/notification_repository.dart
│   │       ├── domain/notification.dart
│   │       └── presentation/
│   │           ├── notifications_screen.dart
│   │           └── notification_tile.dart
│   └── shared/
│       ├── widgets/{app_scaffold,empty_state,error_state,loading_state}.dart
│       └── utils/{date_format,priority_color}.dart
├── test/                              # unit + widget tests
│   ├── core/api/auth_interceptor_test.dart
│   ├── features/auth/auth_controller_test.dart
│   └── ...
└── integration_test/                  # e2e (device farm 留 V1.1)
    └── app_e2e_test.dart
```

### 3.3 状态管理(Riverpod 模式)

- **`authStateProvider`** = `StateNotifier<AuthState>`(sealed: `Unauthenticated` / `Authenticating` / `Authenticated(user, tenant)` / `AuthError(message)`)
- **`dioProvider`** = `Provider<Dio>`(override 在 test)
- **`boardControllerProvider`** = `AsyncNotifierProvider.family<BoardController, Board, String projectId>`
- **`workItemProvider`** = `FutureProvider.family<WorkItem, String id>`
- **`notificationsProvider`** = `AsyncNotifierProvider<NotificationsController, List<Notification>>`(轮询 30s)

### 3.4 启动流程

1. `main.dart` → `ProviderScope` → `app.dart`
2. `app.dart` → 读 `authStateProvider.initialState`:
   - `Unauthenticated` → `/login`
   - `Authenticated` → `/projects`(项目选择)→ `/projects/{id}/board`
3. dio 拦截器链: `LoggingInterceptor` → `AuthInterceptor` → `ErrorInterceptor`
4. `AuthInterceptor` 401 时自动 refresh + retry 1 次;refresh 失败 → 清凭证 → `/login`

### 3.5 路由表(go_router)

| 路径 | 屏幕 | 守卫 |
|---|---|---|
| `/login` | `LoginScreen` | public |
| `/projects` | `ProjectListScreen` | auth |
| `/projects/:projectId/board` | `BoardScreen` | auth + project membership |
| `/work-items/:id` | `WorkItemDetailScreen` | auth |
| `/notifications` | `NotificationsScreen` | auth |
| `/settings` | `SettingsScreen` (theme / logout) | auth |

---

## §4 数据模型(本地缓存)

### 4.1 缓存策略(MVP = online-only + 内存 + Keystore token)

| 数据 | 存储 | TTL | 原因 |
|---|---|---|---|
| **access_token** | flutter_secure_storage (Android Keystore) | 15 min(后端定) | 短期凭证,丢可 refresh |
| **refresh_token** | flutter_secure_storage | 7 day | 长期凭证,泄露代价高 |
| **user** | flutter_secure_storage (JSON) | 跟随 refresh | 启动时读,免再请求 /me |
| **当前 tenant** | flutter_secure_storage | 跟随 refresh | 同上 |
| **board 配置** | 内存 (Riverpod) | 单 session | 切屏重新拉 |
| **work-items 列表** | 内存 | 单 session | MVP 不做离线 |
| **work-item 详情** | 内存 | 单 session | 同上 |
| **notifications 未读** | 内存 | 30s 轮询 | 同上 |

**为什么不缓存 board/work-item 到 Hive**:MVP 只读 + 实时性要求 + "内网使用" 网络稳定,缓存反而引入 stale 数据风险。**离线缓存留 V1.1**(per §9 G-04)。

### 4.2 Hive vs SQLite 决策

**MVP 不引入 SQLite**(sqflite / drift),只用 Hive KV。理由:
- 业务数据全是只读,不需要 SQL 查询
- 后续 V1.1 离线缓存再上 drift(类型安全 + migrations)
- 减少包大小,加快冷启动

---

## §5 鉴权流

### 5.1 登录流程(per api-design §1.12 Authenticated + §6.2 G-01 OAuth 暂不做)

> **⚠️ 已知缺口 G-01**:api-design.md §6.2 显式声明 "MVP 暂不做 OAuth";**但 `POST /v1/auth/login` 路径在我本次扫读 §3 中未找到明文**(grep `/v1/auth/login` 仅匹配隐式引用)。Mavis 接手按"完成设计文档撰写"指令先落档,端点存在性须 Ulysses 拍板。

**默认假设(若端点存在)**:

```
┌──────────┐  POST /v1/auth/login  ┌──────────┐
│  Flutter │ {email, password}     │ STAR API │
│   App    │ ──────────────────────▶          │
│          │                       │          │
│          │  {access_token:       │          │
│          │   "eyJ...",           │          │
│          │   refresh_token:      │          │
│          │   "rt_...",           │          │
│          │   expires_in: 900,    │          │
│          │   user: {id, name,    │          │
│          │         avatar_url},  │          │
│          │   tenant: {id, name}} │          │
│          │ ◀─────────────────────│          │
└──────────┘                       └──────────┘
       │
       ▼
flutter_secure_storage.write(...)
       │
       ▼
AuthState = Authenticated(user, tenant)
       │
       ▼
go_router.go('/projects')
```

### 5.2 Device 三重绑定(internal-design §23.2)

> **per internal-design §23.2 domain-identity Device 三重绑定**:登录成功后 Flutter 端须调 `POST /v1/devices/bind`(path 待补)上报 device_id + platform + app_version,后端做 device fingerprint 校验。

**MVP 简化**(待 §9 G-09 拍板):
- MVP 跳过 device bind,仅 token 鉴权
- V1.1 补 device bind 流程

### 5.3 Token 刷新

- **触发**:Dio `AuthInterceptor` 检测 response.status == 401
- **行为**:读 refresh_token → `POST /v1/auth/refresh` → 写新 access_token → retry 原 request
- **失败**:refresh_token 也 401 → 清所有凭证 → AuthState = Unauthenticated → 跳 `/login`

### 5.4 Logout

- 清 `flutter_secure_storage` 所有 key
- `POST /v1/auth/logout`(best-effort,失败也清本地)
- `AuthState = Unauthenticated` → `/login`

---

## §6 UI 范围

### 6.1 屏幕清单(6 屏)

| # | 屏幕 | 关键元素 | 复用 web 端 |
|---|---|---|---|
| 1 | **Login** | email + password 输入框 / 登录按钮 / 错误 toast | — |
| 2 | **Project List** | 项目卡片列表(头像 + 名称 + work-item 数) | 复用 web `projects/page.tsx`(per internal-design §10) |
| 3 | **Board** | 横向滑动 columns,每列垂直卡片列表,卡片显示 title + assignee avatar + priority chip | 复用 web `kanban/KanbanBoard.tsx` 数据结构(per internal-design §10 V1) |
| 4 | **Work Item Detail** | 顶部 title + status chip / Tab(Overview / Comments / Transitions)/ 操作:Mark Read / 跳 Web | 复用 web `work-item/page.tsx` |
| 5 | **Notifications** | 未读列表 / 全已读按钮 / 点击跳 Work Item | 复用 web `notification/page.tsx` |
| 6 | **Settings** | theme 切换(light/dark/system) / 退出登录 | 复用 web `settings/page.tsx` |

### 6.2 设计语言

- **Material 3** + 自定义 ColorScheme(从 web 端 Star palette 提取)
- **字体**:思源黑体 / Roboto(中英混排)
- **触控目标**:≥ 48dp(Material guideline)
- **空状态**:复用 web `EmptyState` 组件结构 + 重画
- **错误状态**:网络错误显示重试按钮(per Dio error_interceptor)

### 6.3 屏幕间导航

- **底部导航**(3 tab):Projects / Notifications / Settings
- 看板 / 详情走 push 模式(go_router `push`)

---

## §7 轮询策略(替代 WS)

### 7.1 通知轮询

- 间隔:30s
- 实现:`StreamProvider.periodic` 30s 调一次 `GET /v1/notifications?read=false&limit=20`
- 节能:app 进入后台时暂停(`WidgetsBindingObserver.didChangeAppLifecycleState`)
- 失败:backoff 1s → 3s → 10s,3 次后放弃到下次唤醒

### 7.2 看板数据刷新

- 触发:下拉刷新 + 进入屏幕时一次性拉
- **不做** 自动轮询(30s 内手动 pull-to-refresh)

### 7.3 已知性能预算

- 启动到首页可交互:< 1.5s(Mid-range Android, e.g. Pixel 6)
- API P95 响应:< 200ms(内网,后端在 1.2 范围)
- 内存占用:< 120MB(纯 Flutter app,无后台服务)

---

## §8 部署(内网)

### 8.1 APK 构建

```bash
cd frontend/mobile-flutter
flutter build apk --release \
  --dart-define=STAR_HOST=http://star.internal:8080 \
  --dart-define=API_VERSION=v1 \
  --obfuscate \
  --split-debug-info=build/symbols/
```

- **applicationId**:`com.star.mobile`(待 Ulysses 拍)
- **versionCode**:CI 自动 +1
- **签名**:内网 keystore(自签,**待 SRE Lead 拍板** per 8/21 JST 5 域独立 Lead 规则)
- **minSdkVersion**:24 (Android 7.0+,覆盖 95%+ 内网设备)
- **targetSdkVersion**:34 (Android 14)

### 8.2 AndroidManifest 关键项

```xml
<manifest>
  <uses-permission android:name="android.permission.INTERNET" />
  <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />

  <application
    android:label="Star"
    android:icon="@mipmap/ic_launcher"
    android:usesCleartextTraffic="true"   <!-- MVP HTTP 明文内网 -->
    android:networkSecurityConfig="@xml/network_security_config">
    ...
  </application>
</manifest>
```

**`usesCleartextTraffic="true"` + `network_security_config.xml`**:MVP 阶段限定明文 HTTP 仅走 `star.internal` 域,避免放开到任意 HTTP。

```xml
<!-- res/xml/network_security_config.xml -->
<network-security-config>
  <domain-config cleartextTrafficPermitted="true">
    <domain includeSubdomains="true">star.internal</domain>
  </domain-config>
  <base-config cleartextTrafficPermitted="false">
    <trust-anchors>
      <certificates src="system" />
    </trust-anchors>
  </base-config>
</network-security-config>
```

### 8.3 APK 分发

**内网三选一**(待 Ulysses 拍):

| 方案 | 适合 | 复杂度 |
|---|---|---|
| **(a) 文件共享 + 邮件**(MVP 推荐) | 10-50 人内网小团队 | 低,无服务端 |
| **(b) Firebase App Distribution / 蒲公英** | 50-500 人,需分发统计 | 中(违反 ADR-0021 零厂商,排除) |
| **(c) 自建 MDM**(e.g. AppCenter on-prem / 自研) | 500+ 人,需强管控 | 高(留 V1.1) |

**默认方案 (a)**:构建产物上传内网文件共享(企业 NAS / MinIO),README 写明扫码下载步骤。后续 V1.1 升级到 (c)。

### 8.4 升级策略

- **不做自动升级检测**(MVP 无后端 push)
- **App 内 banner**:启动时调 `GET /v1/app-version`(端点待补,**§9 G-02**),若有新版显示"请到内网下载页更新"
- **强制升级**:不支持(MVP 阶段)

---

## §9 已知缺口(per 守门 #11 缺标比错标安全)

| # | 缺口 | 拍板人 | 影响 |
|---|---|---|---|
| **G-01** | `POST /v1/auth/login` / `/v1/auth/refresh` / `/v1/auth/logout` 端点是否已在后端实现,api-design.md §3 未明文 | Ulysses(架构师) | 端点不存在则 MVP 走不了,需先开后端或改走 OAuth 浏览器跳转 |
| **G-02** | `GET /v1/app-version` 升级检测端点 | SRE Lead | 不实现则升级纯靠用户自觉 |
| **G-03** | `STAR_HOST` 内网域名/IP + 端口 | SRE Lead | 不拍则 MVP 无法对接 |
| **G-04** | 离线缓存(SQLite/drift) | 5 域 Lead(work-item) | 不做则断网空白屏 |
| **G-05** | 推送(FCM)被 ADR-0021 禁,自建 WebSocket 长连留 V1.1 | Ulysses | 通知延迟 30s,实时性差 |
| **G-06** | Tablet / 横屏适配 | 5 域 Lead(frontend) | 只支持手机竖屏,平板拉伸 |
| **G-07** | iOS 适配 | Ulysses | 跨平台 0 复用,但 V2 移动已要求 iOS(per internal-design §10) |
| **G-08** | HTTPS / 走 envoy(per 9/1 13:03/13:05 JST 偏好) | SRE Lead | MVP HTTP 明文,后续上公网须升级 |
| **G-09** | Device 三重绑定(internal-design §23.2)MVP 跳过 | Ulysses(安全) | 风险:丢失手机 + 知道密码即可登录,无 device fingerprint |
| **G-10** | 内网 keystore 签名策略 | SRE Lead | 5 域独立 Lead 拍(per 8/21 JST) |
| **G-11** | APK 分发渠道(文件共享 vs 自建 MDM) | Ulysses + SRE Lead | 影响运维流程 |
| **G-12** | 仓位置(`apps/star-mobile-flutter/` vs `frontend/mobile-flutter/`) | 架构师 | 影响 CI + monorepo 关系 |
| **G-13** | WebSocket 实时推送(替代轮询) | 5 域 Lead(realtime) | 体验 vs 复杂度权衡 |
| **G-14** | 5 域独立 Lead 真实身份补签(per 8/21 JST 拒绝兼任) | DDD Review Lead | 当前签字栏全 Mavis 接手代签,DDD Review 阶段可补 |
| **G-15** | AGENTS.md §7 WBS 是否新增第 #8 项"Flutter MVP" | Ulysses | 当前 WBS 7 项不含,需拍是否独立排期 vs 并入 P3-B |

---

## §5 守门规则(本 spec 适配,per AGENTS.md §4)

### 5.1 适用守门

| # | 规则 | 本 spec 适配 |
|---|---|---|
| **#1** | R-05 不 push(已反转) | Flutter 代码独立 git 仓 / sub-repo,推 origin 走 §5.2 拍板 |
| **#3** | 5 域独立 Lead, 不接受兼任 | work-item / board / notification / auth / frontend 5 域 Lead 独立签字栏(per 8/21 JST) |
| **#4** | token-OLU 而非人天 | §11 列出 WBS token 预算(per STAR-OLU-001 1 SRE·周 = 1.2M) |
| **#5** | 环境变量安全 | `STAR_HOST` 走 `--dart-define` 不入 .env;§8.1 显式列出 |
| **#6** | PowerShell only | CI 脚本用 PowerShell,build 用 flutter cli |
| **#7** | 0 unsafe | `flutter analyze --fatal-infos` + `dart fix` 0 warning |
| **#8** | 不沿用回溯叙事 | §0.3 / §9 / §11 全部 git 实证或显式标"待补" |
| **#9** | 子代理产出必 git 实证 | 本 spec 落档后,实施若派子代理,每个 sub-task 必有 commit hash 短码 |
| **#10** | 代签规则应用 | §11 签字栏 Mavis 接手代签 Ulysses(per 8/27 19:39 JST) |
| **#11** | 缺标比错标安全 | §9 已知缺口 15 项全部显式列出 |
| **#12** | AI 协作文档治理 | 本 spec 7 段结构 + git 实证 |
| **#13** | DB W/T/M 横展开 | Flutter 端无 DB,§4 本地缓存按 W 短 TTL 标(实际 MVP 不持久化,纯内存) |

### 5.2 实施阶段硬约束

1. **WBS 必先排**(per STAR-OLU-001 + §11 token 预算表)
2. **每个 sub-task 必先 git commit**(sub-task 完成 ≠ 整体完成, commit hash 是唯一证据)
3. **CI 守门**:`flutter analyze` + `flutter test` + `flutter build apk --release` 三件套 0 失败
4. **不沿用 Web 端 mock 数据**(per AGENTS.md §4 #1 v1-v14 守门派生, Flutter 端 dio 拦截器必走真 API)
5. **守门 #1 v15**(2026-08-29 22:39 JST 饱和约束):若本 spec 实施时 docs 同步触达饱和,任何后续 docs 同步 commit 必先有**新事件触发**(代码改动 / Ulysses 拍板)

---

## §6 签字栏(5 角色, per AGENTS.md §3 + 8/21 JST 5 域独立)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代签 (per 8/27 19:39 JST + 21:59 JST 三次强化授权) |
| **SRE Lead** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代签;5 域独立真实身份(per 8/21 JST) DDD Review 阶段可补 |
| **平台 Lead** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代签;5 域独立真实身份 DDD Review 阶段可补 |
| **评审主持** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代签;5 域独立真实身份 DDD Review 阶段可补 |
| **PM** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代签;5 域独立真实身份 DDD Review 阶段可补 |

---

## §7 修订历史(per AGENTS.md §3 + 守门 #12 禁回溯)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版:立项背景 + MVP 边界 + 13 端点映射 + Flutter 端架构 + 鉴权/缓存/部署策略 + 15 项已知缺口 + 5 维守门 + 5 角色签字栏 | 2026-09-02 15:52 JST Ulysses 发令"完成设计文档撰写"(承接"安卓版flutter移动app开发得怎样了?第一版默认在内网使用即可");Mavis 接手按"完成"指令直接落档,3 维默认(方向/MVP 范围/网络)未被 Ulysses 显式确认,落 §0.3 + §9 已知缺口待翻牌 |
| v0.2 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | **SUPERSEDED 标记**:本 doc 已被 IPA 3 段組 (01-requirements.md v1.0 + 02-basic-design.md v1.0 + 03-detailed-design.md v1.0) supersede;本 doc 保留作 v0.1 考古证据;头部新增 SUPERSEDE 通知块指向 IPA 3 段 | 2026-09-02 16:09 JST Ulysses 発令「要符合日本IPA标准的需求、基本设计、详细设计」;per 守门 #1 禁回溯叙事, 保留 v0.1 commit 链 + 加 supersede 标记(不删 v0.1) |

---

## §11 附录 A — token 预算(per STAR-OLU-001 1 SRE·周 = 1.2M)

| # | 子项 | 估 token | 软参考周 | 依赖 |
|---|---|---|---|---|
| 1 | 仓骨架 + CI(pubspec, analysis_options, android shell, apk build) | 0.3M | 0.25 周 | G-03/G-12 拍板 |
| 2 | core 层(dio + 3 拦截器 + token storage + env + result) | 0.4M | 0.33 周 | G-01 拍板(login 端点存在) |
| 3 | auth feature(login screen + auth controller) | 0.2M | 0.17 周 | #2 |
| 4 | board + work-item + notification features(6 屏) | 0.6M | 0.5 周 | #2 + G-04 暂不做 |
| 5 | 单元 + widget + integration test | 0.3M | 0.25 周 | #1-#4 |
| 6 | 守门 #1+#7+#12 实证(analyze + test + build apk 0 错) | 0.2M | 0.17 周 | #1-#5 |
| **总计** | — | **~2.0M** | **~1.67 周** | — |

**对比 §0.3 默认**:MVP = online-only + 6 屏 + 1 平台,符合"第一版内网轻量"判断;若用户后续选"UAT 级别"含离线 + 推送 + iOS,估 5.0-6.0M(per 之前 ask_user 第 2 题说明)。

---

## §12 附录 B — 引用清单(git 实证,per 守门 #1 禁回溯)

| 引用 | 路径 | 实证 |
|---|---|---|
| V1 排除移动 | `docs/internal-design.md:50` | 本次扫读实证 |
| V2 移动范围 | `docs/internal-design.md:1600` | 本次扫读实证 |
| V2 候选 React Native | `docs/internal-design.md:1633` | 本次扫读实证 |
| Web 端 PWA 移动布局 | `docs/frontend/design/ui-3pane-arch.md:68` | 本次扫读实证 |
| Web 端 Drawer 模式 | `docs/external-design.md:230` | 本次扫读实证 |
| API WorkItem 端点 | `docs/api-design.md:3.5:624-631` | 本次扫读实证 |
| API Board 端点 | `docs/api-design.md:3.7:668-672` | 本次扫读实证 |
| API Notification 端点 | `docs/api-design.md:3.16:787-789` | 本次扫读实证 |
| 鉴权 5 级分层 | `docs/api-design.md:1.12:307-313` | 本次扫读实证 |
| Tenant 隔离 | `docs/api-design.md:1.8:259-269` | 本次扫读实证 |
| OAuth 暂不做 | `docs/api-design.md:6.2 G-01` (推断 / 待 Ulysses 二次实证) | 本次扫读 grep 未明文匹配,标 ⚠️ |
| 16 MCP tool 列表 | `crates/star-mcp/src/main.rs:53-56` (经 `docs/architecture/2026-09-02-upgrade/spec/integration/02-developer-api-and-outbound-webhook-spec.md:46-49` 引用) | 引用链实证 |
| Device 三重绑定 | `docs/internal-design.md:23.2` (推断 / 待二次实证) | 本次扫读 grep 未明文匹配,标 ⚠️ |
| 零厂商合作 | `docs/architecture/2026-08-26-upgrade/adr/0021-zero-vendor-cooperation.md` | 引用 02 spec 文件夹推断存在 |
| 5 域独立 Lead | `AGENTS.md:§4 #3 + 8/21 JST` | 修订历史实证 |
| 代签授权 | `AGENTS.md:§1.0 + 8/27 19:39 JST` | 修订历史实证 |
| token-OLU 基线 | `STAR-OLU-001.md` v0.1 + `AGENTS.md:§4 #4` | 引用 |
| 守门 #1 派生 | `AGENTS.md:§4.1 v1-v24` | 修订历史实证 |
| envoy 偏好 | `user.md` 9/1 13:03/13:05 JST | 用户偏好 memory 实证 |
| 拍板必须用选项 | `user.md` 9/1 14:58 JST | 用户偏好 memory 实证(本 spec 3 维默认未被显式选,落 §0.3 + §9 待翻牌) |

> **⚠️ 未实证项**(per 守门 #1 禁回溯):OAuth 暂不做、Device 三重绑定、ADR-0021 文件存在性 — 落档后由 Mavis 接手下个 sub-task `git log -p --follow` 二次实证,失败则删该引用 + 改"待补"标注。
