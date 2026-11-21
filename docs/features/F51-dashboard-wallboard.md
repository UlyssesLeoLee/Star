# F51 仪表盘与 Wallboard

> 状态摘要: `domain-dashboard` 仅有内存中的 Dashboard/Gadget 模型、`DashboardService` 的创建与添加 Gadget 操作和 7 个单元测试；持久化、REST 路由、前端页面、导航入口与 Wallboard 公开访问均未实现，页面与 UI 状态为 ⬜。本文以 `docs/specs/domain-dashboard-spec.md` v1.0 为目标设计，给出 `/dashboards` 页面方案、数据契约与实施顺序。
> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 用户在 `/dashboards` 下创建仪表盘，把 Gadget（报告图表、个人待办、活动流、过滤结果、文本、时钟等）放入 12 列网格，支持拖拽移动与缩放，配置保存后可复用（spec §1、§2、§5）。
- 仪表盘可共享给用户、组或公开链接。公开链接用于 Wallboard 全屏只读展示，30 秒自动刷新，链接 30 天过期（spec §3 INV-DASH-03、INV-DASH-06、INV-DASH-10，§8.3）。
- 仪表盘只持有配置与共享关系，不持有业务事实；工作项与计划事实留在 SoR 域，图表数据经报告引擎获取（`docs/requirements/charts-and-reports.md` §9.2）。

### 1.2 范围内 / 范围外

范围内：Dashboard 与 Gadget 的 CRUD、网格布局与校验、共享、Wallboard、分析组导航入口、`/dashboards` 与 `/dashboards/[id]` 页面、`/wallboard/[token]` 公开页、REST 与前端数据层、与 `domain-report` 的 chart-* 接线。

范围外：22 个图表本体的交互细节（属 `docs/design/charts/`，本文只记录 Gadget 的引用方式）；商业 BI 嵌入与 ECharts/D3（charts §0.1 已排除）；订阅与定时邮件归入 P2（FR-51-31），本文只留接口位。

优先级口径：源文档未给出需求优先级，本文统一标注：P0 = 首个可用页面必需；P1 = spec v1.0 核心需求；P2 = 后续增强。

领域定位：`domain-dashboard` 是 supporting crate，不属于 22 个 bounded context，与 5 域历史治理命名不建立映射（spec 文件头 dual-use 说明）。

## 2. 用户与场景

| 角色 | 权限要点 | 主要场景 |
|---|---|---|
| 仪表盘所有者 | 编辑、共享、开启与关闭 Wallboard | US-1：创建"Sprint 概览"，放入燃尽图、速度图与 SLA 数字，拖拽调整布局 |
| 团队成员 | 按 ShareGrant 获得 view 或 edit | US-2：打开团队仪表盘查看；edit 授权者可调整布局 |
| 管理员 | 可创建 Global 范围仪表盘 | US-3：创建全公司仪表盘（域层规则见 `crates/domain-dashboard/src/lib.rs:250-258`） |
| 大屏观众 | 无登录，凭公开链接只读 | US-4：办公室大屏展示 Wallboard；链接过期后由所有者重新生成 |
| 所有成员 | — | US-5：某个 Gadget 数据源故障时，其余 Gadget 正常渲染，失败卡片可重试（INV-DASH-11） |

## 3. 需求

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-51-01 | 创建仪表盘（名称、所有者、租户、范围） | P0 | 🟡 | `DashboardService::create` 可创建内存 Dashboard（`crates/domain-dashboard/src/lib.rs:242-261`）；无 REST、无持久化 |
| FR-51-02 | 列表与查询：我的、共享给我、项目内 | P0 | ⬜ | spec §4.2 定义 `DashboardQueryPort`；代码无查询接口（`crates/domain-dashboard/src/lib.rs` 全文无 trait） |
| FR-51-03 | 更新与软删除（INV-DASH-15） | P1 | ⬜ | 代码仅有 `add_gadget`、`remove_gadget`、`enable_wallboard`（lib.rs:173、183、194），无 update、delete |
| FR-51-04 | 范围四级 Personal/Team/Project/Global；Global 仅管理员创建 | P0 | 🟡 | `DashboardScope` 四个变体（lib.rs:86-95）；Global 非管理员拒绝（lib.rs:250-258，测试 lib.rs:333-344）；Team、Project 无任何校验 |
| FR-51-05 | Gadget 类型集合取 spec §5 的 10 类 | P0 | 🟡 | 代码 `GadgetType` 10 个变体但集合不同（lib.rs:22-43）；详见 §4.2.2 映射表 |
| FR-51-06 | Gadget 默认尺寸取 spec §5 | P1 | 🟡 | `default_size` 数值不同，例如 Burndown 为 3×2（lib.rs:65），spec §5.1 的 chart 为 6×4 |
| FR-51-07 | 12 列网格：`x` 取值 0–11，`x + w ≤ 12`（INV-DASH-01） | P0 | ⬜ | `InvalidPosition` 已定义（lib.rs:213-215）但从未返回；`add_gadget_at` 固定 `x: 0`（lib.rs:281） |
| FR-51-08 | 不重叠校验与拖拽实时校验（INV-DASH-02、INV-DASH-07；spec §8.1） | P0 | ⬜ | 无 `LayoutValidator`；`GadgetOverlap` 错误变体不存在（lib.rs:205-226） |
| FR-51-09 | 单仪表盘 Gadget 上限 12（INV-DASH-04） | P0 | 🟡 | 代码上限为 20（lib.rs:174-176） |
| FR-51-10 | 移动、缩放与重排（move、resize、reorder） | P0 | ⬜ | 仅有 add 与 remove（lib.rs:173、183） |
| FR-51-11 | 紧凑化自动布局（spec §8.2） | P2 | ⬜ | spec §8.2 为 TypeScript 草案；代码无对应实现 |
| FR-51-12 | chart-* Gadget 引用报告定义（`chart_report_id`，INV-DASH-08） | P1 | ⬜ | `Gadget` 无 `chart_report_id` 字段（lib.rs:102-116）；`domain-report` 无 `ReportDefinition` 类型（全仓 grep 无结果），报告类型为 `ReportType`（`crates/domain-report/src/lib.rs:34-81`，`chart_id` 见 :168） |
| FR-51-13 | Gadget 独立加载与失败隔离（INV-DASH-11） | P0 | ⬜ | 无 Gadget 数据接口与加载逻辑（`load_gadget_data` 仅在 spec §4.2） |
| FR-51-14 | Gadget 数据并行加载，最多 12 个，单 Gadget 缓存 5 分钟（basic-design §4.5） | P1 | ⬜ | 无实现 |
| FR-51-15 | Wallboard 全屏模式开关 | P1 | 🟡 | `wallboard_mode: bool`（lib.rs:143）；`enable_wallboard` 置位（lib.rs:194-197）；无关闭操作 |
| FR-51-16 | Wallboard 只读，隐藏编辑 UI（INV-DASH-03；spec §8.3） | P1 | ⬜ | `add_gadget` 不检查 `wallboard_mode`（lib.rs:173-180） |
| FR-51-17 | Wallboard 30 秒自动刷新（INV-DASH-10） | P1 | ⬜ | `Dashboard` 无 `auto_refresh_seconds`（lib.rs:128-148） |
| FR-51-18 | Wallboard 公开链接 `link_token`，30 天过期（INV-DASH-06） | P1 | ⬜ | 无 token 字段；无 `WallboardLinkExpired` 错误（lib.rs:205-226） |
| FR-51-19 | Wallboard 响应不含 `owner_id`，无编辑接口（INV-DASH-14） | P1 | ⬜ | 无 wallboard 查询接口 |
| FR-51-20 | 共享授予 ShareGrant（user、group、link；view、edit；单仪表盘不超过 50，INV-DASH-09） | P1 | ⬜ | 全仓无 `ShareGrant`（grep 无结果）；仅 `DashboardScope` 枚举（lib.rs:86-95） |
| FR-51-21 | 跨租户访问拒绝（INV-DASH-13） | P0 | ⬜ | `tenant_id` 字段存在（lib.rs:139）但无任何比对逻辑 |
| FR-51-22 | 按 actor 的读写授权（PermissionPort） | P0 | 🟡 | `create` 接收 `actor_id` 与 `is_admin`（lib.rs:242-249）；`add_gadget`、`remove_gadget`、`enable_wallboard` 不接收 actor（lib.rs:173、183、194） |
| FR-51-23 | 标题 1–200 字，描述 0–2000 字（INV-DASH-12） | P1 | ⬜ | `name` 无长度校验（lib.rs:157-161） |
| FR-51-24 | 查看与编辑审计（`dashboard_view_audit`、`dashboard_edit_audit`；basic-design §9） | P2 | ⬜ | 全仓无 dashboard 相关 SQL（grep 无结果） |
| FR-51-25 | 导航入口：分析组中的"仪表盘" | P0 | ⬜ | 分析组 tabs 仅含 analytics、ops、audit、sandbox（`frontend/src/lib/nav/registry.ts:777-783`）；全仓无 `/dashboards` 路由引用 |
| FR-51-26 | 页面 `/dashboards`（列表）与 `/dashboards/[id]`（网格） | P0 | ⬜ | pin 上无 `frontend/src/app/(app)/dashboards` 目录 |
| FR-51-27 | REST 接口 `/api/v1/dashboards`（CRUD、Gadget、共享、Wallboard） | P0 | ⬜ | `crates/star-api-rest` 中 dashboard 相关仅 `routes/ex_07.rs:33` 的注释；`build_group_router` 无合并项（`crates/star-api-rest/src/group_api.rs:806-829`） |
| FR-51-28 | 前端数据层：client 与 MSW 处理器 | P0 | ⬜ | `frontend/src/lib/group-api/` 仅有 `group-context.ts`；`frontend/src/mocks/handlers/index.ts` 无 dashboard 处理器 |
| FR-51-29 | 空、加载、错误、未接入状态（ui-3pane §3 规则 5–7、10） | P1 | ⬜ | 页面未建立 |
| FR-51-30 | 键盘可达，并提供移动与缩放的键盘替代（charts §8.1；ui-3pane §4 规则 7） | P1 | ⬜ | 页面未建立 |
| FR-51-31 | 订阅与定时通知（REQ-NOTIF-002；spec §4.3） | P2 | ⬜ | `crates/domain-dashboard/src` 无 subscription 相关符号 |
| FR-51-32 | 单 Gadget 数据点不超过 100K（INV-DASH-05） | P2 | ⬜ | 无数据接口 |
| FR-51-33 | 与工作项、审计、身份、权限、通知域的只读接口对接（spec §6） | P1 | ⬜ | `crates/domain-dashboard/Cargo.toml:18-23` 仅依赖 serde、serde_json、thiserror、uuid、chrono |

## 4. 设计

### 4.1 概念模型

```
Dashboard（聚合根，按 tenant_id 隔离）
 ├─ layout: DashboardLayout { columns = 12, row_height = 80px, gap = 16px }   ⬜
 ├─ gadgets[0..12]: Gadget { id, type, title, position{x,y}, size{w,h}, config, chart_report_id? }
 │     └─ 数据：由 type 决定的只读查询（domain-report / work-item / audit / 客户端时钟）
 ├─ shares[0..50]: ShareGrant { grantee_type user|group|link, grantee_id | link_token, permission view|edit, expires_at }   ⬜
 ├─ wallboard: 开关 + link_token（30 天过期）                                     🟡 仅开关
 ├─ scope: Personal | Team | Project | Global                                     🟡
 └─ subscriptions[*]（P2）                                                        ⬜
```

目标设计的裁决（依据见 §8）：

- 以 spec v1.0 的聚合为目标；`DashboardScope` 保留为 Global 管理员规则的载体，与 `ShareGrant` 的关系列入 §6 差距 G-03。
- Gadget 类型集合取 spec §5；序列化值采用 snake_case，与 `crates/domain-dashboard/src/lib.rs:21` 的 serde 约定一致。
- 网格固定 12 列，上限 12 个 Gadget。

### 4.2 数据模型

#### 4.2.1 Dashboard 与 Gadget 字段

| 实体.字段 | 目标（spec §2） | 代码（lib.rs） | 状态 |
|---|---|---|---|
| Dashboard.dashboard_id | UUID | `id: Uuid`（lib.rs:131） | 🟡 字段名不同 |
| Dashboard.title | `String`，1–200 字 | `name: String`（lib.rs:133），无长度校验 | 🟡 |
| Dashboard.description | `Option<String>`，0–2000 字 | 无 | ⬜ |
| Dashboard.owner_id | UUID | `owner_id: Uuid`（lib.rs:137） | ✅ |
| Dashboard.tenant_id | UUID，RLS 必携 | `tenant_id: Uuid`（lib.rs:139），无 RLS | 🟡 |
| Dashboard.layout | `DashboardLayout` | 无 | ⬜ |
| Dashboard.gadgets | 最多 12 个 | `Vec<Gadget>`，上限 20（lib.rs:141、174） | 🟡 |
| Dashboard.shared_with | `Vec<ShareGrant>` | 无 | ⬜ |
| Dashboard.is_wallboard | bool | `wallboard_mode: bool`（lib.rs:143） | 🟡 字段名不同 |
| Dashboard.auto_refresh_seconds | 可选，默认 30 | 无 | ⬜ |
| Dashboard.version | 乐观并发版本号 | 无 | ⬜ |
| Dashboard.scope | spec 无此字段 | `DashboardScope`（lib.rs:86-95、135） | 🟡 见 G-03 |
| Dashboard.created_at、updated_at | 时间戳 | lib.rs:145-147 | ✅ |
| Gadget.id | `gadget_id` | `id: Uuid`（lib.rs:105） | ✅ |
| Gadget.gadget_type | §5 集合 | `GadgetType`（lib.rs:107） | 🟡 见 §4.2.2 |
| Gadget.title | 可选覆盖标题 | `title: String`（lib.rs:109） | ✅ |
| Gadget.position | `{x: 0–11, y}` | `GadgetPosition { x: u8, y: u8 }`（lib.rs:119-125），无上界校验 | 🟡 |
| Gadget.size | `{w: 1–12, h}` | `GadgetSize { w: u8, h: u8 }`（lib.rs:75-81），仅注释，无校验 | 🟡 |
| Gadget.config | JSON | `serde_json::Value`（lib.rs:115） | ✅ |
| Gadget.chart_report_id | chart-* 必填 | 无 | ⬜ |
| ShareGrant（全部字段） | §2 | 无 | ⬜ |

版本号与并发控制：仓库 Next.js 代理已转发 `idempotency-key` 与 `if-match` 请求头（`frontend/src/app/api/group/[...path]/route.ts:46-52`），目标设计以 `if-match` 携带 `version` 做乐观并发。

#### 4.2.2 Gadget 类型映射

| spec 类型（§5） | spec 默认尺寸 w×h | spec 数据源 | 代码 `GadgetType`（lib.rs:22-43） | 状态 |
|---|---|---|---|---|
| chart-{C01–C22} | 6×4 | domain-report | `Burndown` 对应 C01，`Velocity` 对应 C03（lib.rs:25-28）；其余 20 个图表无变体 | 🟡 |
| text | 6×2 | 无 | `Markdown`（lib.rs:37-38），默认 3×1（lib.rs:68） | 🟡 |
| activity | 4×6 | domain-audit | `RecentActivity`（lib.rs:31-32），默认 2×2（lib.rs:66） | 🟡 |
| assigned-to-me | 4×6 | domain-work-item | `MyWork`（lib.rs:29-30） | 🟡 |
| filter-results | 6×6 | domain-work-item（经 Filter） | `JqlTable`（lib.rs:35-36），默认 4×2（lib.rs:67） | 🟡 |
| wallboard-clock | 3×2 | 客户端时间 | `Clock`（lib.rs:41-42），默认 1×1（lib.rs:69） | 🟡 |
| wallboard-sla | 3×3 | domain-report C12 | 无 | ⬜ |
| heatmap | 8×4 | domain-report C21 | 无 | ⬜ |
| two-dimensional-filter | 8×6 | domain-work-item 分组 | 无 | ⬜ |
| rich-text（V1.1） | 6×4 | 无 | 无 | ⬜ |
| 代码独有：`IssueStats`、`DueSoon`、`Iframe` | — | — | lib.rs:23-24、33-34、39-40 | 🟡 不在目标集合，见 §8 裁决 |

裁决：`IssueStats` 由 chart-C14、C15 覆盖；`DueSoon` 归入 filter-results 的排序配置；`Iframe` 不进入目标集合，嵌入外部页面需单独的安全评审。

### 4.3 接口

#### 4.3.1 领域 API（代码现状）

| 函数 | 位置 | 行为 | 状态 |
|---|---|---|---|
| `Dashboard::new` | lib.rs:152-170 | 构造，`wallboard_mode = false` | ✅ |
| `Dashboard::add_gadget` | lib.rs:173-180 | 上限 20，超限返回 `TooManyGadgets(20)` | 🟡 |
| `Dashboard::remove_gadget` | lib.rs:183-191 | 按 id 移除，未找到返回 `GadgetNotFound` | ✅ |
| `Dashboard::enable_wallboard` | lib.rs:194-197 | 置位 `wallboard_mode` | 🟡 |
| `DashboardService::create` | lib.rs:242-261 | Global 需管理员 | 🟡 |
| `DashboardService::add_gadget_at` | lib.rs:264-287 | 按 `y = max(y) + 1` 放置，`x = 0` | 🟡 |

#### 4.3.2 目标端口（spec §4）

| 端口 | 方法 | 状态 |
|---|---|---|
| `DashboardCommandPort` | create、update、delete、add_gadget、remove_gadget、move_gadget、resize_gadget、reorder_gadgets、share、revoke_share、enable_wallboard、disable_wallboard | ⬜ 代码中无 trait |
| `DashboardQueryPort` | get、list_by_owner、list_shared_with_me、list_by_project、get_wallboard、load_gadget_data | ⬜ |
| `DashboardSubscriptionPort` | subscribe、unsubscribe、notify_update | ⬜ |

错误模型：代码 `DashboardError` 含 `TooManyGadgets`、`GadgetNotFound`、`InvalidPosition`、`PermissionDenied`（lib.rs:205-226）。spec §4.4 所列 `NotFound`、`ValidationFailed`、`GadgetOverlap`、`GadgetLimitExceeded`、`ChartReportNotFound`、`WallboardLinkExpired`、`GadgetLoadFailed` 在代码中均无对应变体。

#### 4.3.3 REST 契约（目标设计，⬜）

基路径取 `/api/v1`，与 `crates/star-api-rest/src/lib.rs:135-137` 的挂载方式一致。basic-design §API 表中的 `/api/dashboards/*` 不采用（§8 裁决）。

| 方法 | 路径 | 作用 | 授权 scope（拟） | 对应端口 |
|---|---|---|---|---|
| GET | `/api/v1/dashboards?view=owned\|shared\|project` | 列表 | `dashboard:read` | list_* |
| POST | `/api/v1/dashboards` | 创建 | `dashboard:write` | create |
| GET | `/api/v1/dashboards/{id}` | 详情（含 gadgets、layout、wallboard 状态） | `dashboard:read` | get |
| PATCH | `/api/v1/dashboards/{id}` | 更新标题、描述、刷新周期，`if-match` 带版本 | `dashboard:write` | update |
| DELETE | `/api/v1/dashboards/{id}` | 软删除 | `dashboard:write` | delete |
| POST | `/api/v1/dashboards/{id}/gadgets` | 添加 Gadget | `dashboard:write` | add_gadget |
| PATCH | `/api/v1/dashboards/{id}/gadgets/{gadget_id}` | 移动或缩放，body 含 position 与 size | `dashboard:write` | move_gadget、resize_gadget |
| DELETE | `/api/v1/dashboards/{id}/gadgets/{gadget_id}` | 移除 Gadget | `dashboard:write` | remove_gadget |
| GET | `/api/v1/dashboards/{id}/gadgets/{gadget_id}/data` | 单 Gadget 数据 | `dashboard:read` | load_gadget_data |
| POST、DELETE | `/api/v1/dashboards/{id}/shares[/{share_id}]` | 授予与撤销 ShareGrant | `dashboard:share` | share、revoke_share |
| POST、DELETE | `/api/v1/dashboards/{id}/wallboard` | 生成或撤销公开链接（返回 token 与过期时间） | `dashboard:share` | enable、disable |
| GET | `/api/v1/wallboards/{token}` | 公开只读快照，无认证 | 无（凭 token） | get_wallboard |

Wallboard 以 token 寻址而非 `{id}`，理由见 §8 与 INV-DASH-14。

#### 4.3.4 前端数据接口（目标设计，⬜）

客户端沿用 `frontend/src/lib/group-api/group-context.ts` 的模式：`constructor(private readonly baseUrl = "/api/group")`（group-context.ts:158）。代理路由 `frontend/src/app/api/group/[...path]/route.ts` 引入 `next/headers` 的 `cookies`（:40），白名单函数 `isAllowedGroupApiPath` 目前只放行 worktrees 与项目 worktree 索引（route.ts:68-75）。目标设计需在该白名单中加入 `dashboards` 路径与公开 `wallboards` 路径。

### 4.4 状态机与关键流程

#### 4.4.1 Wallboard 开关与链接

| 状态 | 进入条件 | 退出条件 | 现状 |
|---|---|---|---|
| 关闭 | 默认 | 开启 | 🟡 仅 `wallboard_mode = false`（lib.rs:166） |
| 开启（有效 token） | 所有者开启，生成 token，过期时间为 now + 30 天 | 所有者撤销，或到期 | ⬜ 无 token |
| 过期 | 到达 `expires_at` | 所有者重新生成 | ⬜ 错误变体不存在 |

#### 4.4.2 Gadget 放置（目标流程）

选择类型 → 取默认尺寸 → 查找第一个能容纳该尺寸的空位（先从最上方行开始）→ 校验 INV-DASH-01、02、04 → 持久化 → 返回 Gadget。现状：`add_gadget_at` 只完成"找最底行、x 固定为 0、插入内存向量"，缺少校验与空位搜索（lib.rs:264-287）。

#### 4.4.3 拖拽移动（目标流程）

进入编辑态 → 拖拽开始记录原位 → 每次位置变化执行 INV-DASH-07 校验，非法位置禁止放下 → 放下后 PATCH，携带 `if-match` → 失败时回滚到原位并显示错误。现状：无编辑态，无拖拽，无校验。

#### 4.4.4 仪表盘加载（目标流程）

依据 basic-design §4.5（charts-and-reports.md:642-651）：

1. 打开 `/dashboards/{id}`。
2. 拉取 Dashboard 与 gadgets。
3. 并行拉取各 Gadget 数据，最多 12 个。
4. 各 Gadget 独立缓存 5 分钟，单个失败不阻塞其他。
5. 以 CSS Grid 渲染 12 列。
6. Wallboard 模式每 30 秒刷新。

现状：以上步骤均未实现。

### 4.5 权限与安全

| 规则 | 来源 | 现状 |
|---|---|---|
| 租户隔离 | INV-DASH-13；basic-design §5 | ⬜ 无表；但已有 `set_config('app.tenant_id', …)` 模式（`crates/star-api-rest/src/group_api.rs:734-744`）可复用 |
| Global 仅管理员创建 | spec §1、lib.rs:250-258 | 🟡 仅域层 `create` |
| 读写 scope | 拟定 `dashboard:read`、`dashboard:write`、`dashboard:share` | ⬜ 无定义；已有 `require_scope` 模式（group_api.rs:779-789）与 `worktree:read` 用例（group_api.rs:797） |
| actor 校验 | 目标：每个写操作携带 actor | 🟡 `add_gadget`、`remove_gadget`、`enable_wallboard` 不接收 actor（lib.rs:173、183、194） |
| 身份来源 | group 路由使用 `AuthenticatedUser` 提取器（group_api.rs:793；`crates/star-api-rest/src/auth/oauth/middleware.rs:152、209`） | ✅ 模式存在 |
| 全局鉴权中间件 | `auth_layer_stub` 为直通（`crates/star-api-rest/src/middleware/auth.rs:23-31`） | 🟡 真实校验依赖各路由的提取器 |
| 共享 view、edit，上限 50 | INV-DASH-09 | ⬜ |
| Wallboard 公开只读 | INV-DASH-03、INV-DASH-14 | ⬜ |
| Wallboard token 30 天过期 | INV-DASH-06；RISK-DASH-05 | ⬜ |
| 删除审计 | INV-DASH-15；T 类表 | ⬜ |

### 4.6 依赖关系

| 依赖 | 目标用途（spec §6） | 代码现状 | 状态 |
|---|---|---|---|
| domain-report | chart-*、wallboard-sla、heatmap 数据 | `ReportType` C01–C22、`ReportFilter`、`ReportResult`（`crates/domain-report/src/lib.rs:34-81、226、303`）；无 crate 依赖它（Cargo.toml:40 仅为 workspace 成员） | 🟡 |
| domain-work-item | assigned-to-me、filter-results、two-dimensional-filter | `crates/domain-dashboard/Cargo.toml:18-23` 无依赖 | ⬜ |
| domain-audit | activity | 同上 | ⬜ |
| domain-identity | 用户与组共享对象 | 同上 | ⬜ |
| domain-permission | 读写授权（PermissionPort） | 同上 | ⬜ |
| domain-notification | 订阅通知（P2） | 同上 | ⬜ |
| recharts | 图表渲染 | `frontend/package.json:44` | ✅ |
| @dnd-kit/core、@dnd-kit/sortable | 网格拖拽 | `frontend/package.json:18-19` | ✅ |
| react-grid-layout | 备选网格库（lib.rs:9 注释提及） | 不在 `frontend/package.json` 中 | ⬜ 本设计选用 @dnd-kit 与 CSS Grid |
| @tanstack/react-query | 数据获取缓存 | `frontend/package.json:26` | ✅ |
| zustand | 本地状态 | `frontend/package.json:48`；无 dashboard slice | ✅ 依赖存在，业务状态未接入 |
| msw | 模拟处理器 | `frontend/package.json:63`；`frontend/src/mocks/handlers/index.ts:35、58` 注册模式 | ✅ 依赖与注册模式存在 |
| 图表组件 | chart-* 渲染 | `frontend/src/components/charts/Chart01Burndown.tsx` 至 `Chart15PriorityDist.tsx`（15 个）及 `shared/ChartFrame.tsx` | 🟡 C16–C22 缺失 |

### 4.7 页面设计

#### 4.7.0 所有页面共享的布局

- 左侧栏：根布局挂载 `Sidebar`（`frontend/src/app/layout.tsx:77-79`）。分析入口位于 `home-personal` 全局组（`frontend/src/components/Sidebar.tsx:213-220`；`frontend/src/lib/nav/registry.ts:689`）。
- 顶栏：`(app)` 布局经 `AppShell` 挂载 `AppHeader`（`frontend/src/components/AppShell.tsx:48-50`；`frontend/src/app/(app)/layout.tsx:49`）。
- 主区：`AppShell` 的 `main`（`AppShell.tsx:61-73`）。`(app)/layout.tsx:41` 的 `WIDE_PREFIXES` 目前只含 `/projects`；目标设计加入 `/dashboards`，取消 1440px 居中（`AppShell.tsx:69`）。
- 右侧栏：`RightChatRail` 挂载于 `AppShell.tsx:77`，展开 320px、折叠 56px（`frontend/src/components/chat/RightChatRail.tsx:73、97`）。
- 底栏：桌面端无；<768px 的底部导航见 `frontend/src/app/layout.tsx:96` 注释起始处。
- 顶栏导航：`AppHeader` 的顶部标签由用户管理（`DEFAULT_HEADER_TABS` 不含 dashboards，`frontend/src/lib/nav/navStore.ts:94`）；App Matrix 从 `MODULE_GROUPS` 渲染（`frontend/src/components/nav/AppMatrixDrawer.tsx:229`）。

#### 4.7.1 /dashboards

- 文件：`frontend/src/app/(app)/dashboards/page.tsx`（目标设计，⬜；pin 上无此路径）。
- 布局：沿用 4.7.0；主区为宽屏模式。
- 组件树（目标设计，⬜）：

```
DashboardsPage
├─ PageHeader（标题"仪表盘" + "新建仪表盘"）
├─ ScopeTabs（我的 / 共享给我 / 项目 / 全局；"全局"仅管理员可用）
├─ DashboardCardGrid
│  └─ DashboardCard（名称、范围标签、Gadget 数、更新时间、Wallboard 标记）
├─ EmptyState
├─ SkeletonList（不使用 spinner，ui-3pane §3 规则 7）
└─ NotConnectedBanner（REST 未接入时显示）
```

- 状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 列表返回一条或多条 | 卡片网格，按 `updated_at` 倒序 | ⬜ 无接口 |
| 加载 | 请求进行中 | SkeletonList，占位形状与卡片一致 | ⬜ |
| 空 | 列表为空 | EmptyState 与"新建仪表盘"动作（ui-3pane §3 规则 6） | ⬜ |
| 错误 | 5xx 或网络失败 | 错误条、重试按钮与下一步说明（规则 5） | ⬜ |
| 未授权 | 401 或 403，或缺少 `dashboard:read` | 说明无权限，不显示他人仪表盘名称 | ⬜ 无 scope 定义 |
| 未接入 | REST 未接通 | 显示"未接入"状态，不展示示例数据（AGENTS.md:22） | ⬜ 页面未建立 |
| 禁用 | 非管理员 | "全局" 标签禁用并提示需管理员 | 🟡 仅域层规则（lib.rs:250-258） |

- 交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 新建 | `dashboard:write` | 行内输入名称（不使用嵌套模态，ui-3pane §3 规则 4），创建后进入 `/dashboards/{id}` | POST `/api/v1/dashboards` | ⬜（域层 `DashboardService::create` 可用，lib.rs:242-261） |
| 切换标签 | 无 | 重新请求列表 | GET `/api/v1/dashboards?view=` | ⬜ |
| 打开卡片 | `dashboard:read` | 进入 `/dashboards/{id}` | 路由 | ⬜ |
| 删除 | 所有者 | 行内确认（ui-3pane §3 规则 11），软删除后从列表移除 | DELETE `/api/v1/dashboards/{id}` | ⬜（INV-DASH-15） |
| 键盘导航 | 无 | `j`、`k` 上下移动，`Enter` 打开（ui-3pane §1.2 交互原则） | 前端 | ⬜ |

- 数据：来源为 REST，经 `/api/group` 代理（需扩展 route.ts:68-75 白名单）。目标字段：`id`、`name`、`scope`、`owner_id`、`gadget_count`、`wallboard_mode`、`updated_at`。现状：store 无 dashboard 切片，mock 无处理器，三者均无数据。

- 与文档设计的差距：
  - FR-51-02 列表与查询缺失。
  - FR-51-04 Global 标签的可用性需由服务端权限决定，目前无接口。
  - FR-51-26 页面不存在。
  - FR-51-29 空、加载、错误、未接入状态未建立。
  - basic-design §4.5 的入口 `/dashboards/{id}`（charts-and-reports.md:645）与目录 `(app)/dashboards/`（:232）在 pin 上均不存在。

#### 4.7.2 /dashboards/[id]

- 文件：`frontend/src/app/(app)/dashboards/[id]/page.tsx`（目标设计，⬜）。
- 布局：沿用 4.7.0；主区为宽屏模式。工具条位于主区顶部，网格位于其下。网格为 12 列 CSS Grid，行高 80px，间距 16px（domain-dashboard-spec.md §2 中 DashboardLayout 的默认值）。
- 组件树（目标设计，⬜）：

```
DashboardPage
├─ DashboardToolbar（标题、编辑/完成、添加 Gadget、共享、Wallboard、刷新周期）
├─ DashboardGrid（12 列；@dnd-kit/core 拖拽；缩放手柄自研）
│  └─ GadgetCard[]（标题、菜单、移除；内容按类型分发）
│     ├─ ChartGadget → ChartFrame（frontend/src/components/charts/shared/ChartFrame.tsx）+ Chart01–Chart15
│     ├─ TextGadget（Markdown）
│     ├─ ListGadget（assigned-to-me、filter-results、activity 的表格）
│     └─ ClockGadget、SlaGadget（大字号数字）
├─ AddGadgetPanel（行内展开，规则 4）
├─ SharePanel（行内展开，列出 ShareGrant）
└─ WallboardToggle
```

- 状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 详情与 Gadget 数据齐全 | 网格渲染全部 Gadget | ⬜ |
| 加载 | 详情请求进行中 | 按 Gadget 尺寸渲染 Skeleton（规则 7） | ⬜ |
| 空 | `gadgets` 为空 | "添加第一个 Gadget"动作 | ⬜ |
| 错误（仪表盘） | 404 或 5xx | 错误页，提供返回 `/dashboards` 与重试 | ⬜ |
| 错误（单个 Gadget） | 数据源失败 | 该卡片显示错误与重试，其余卡片正常（INV-DASH-11） | ⬜ |
| 未授权 | 无 `dashboard:read`，或跨租户 | 403 提示无权限（INV-DASH-13） | ⬜ |
| 未接入 | REST 未接通 | 显示"未接入"状态 | ⬜ |
| 禁用 | 查看者，或 Wallboard 模式 | 隐藏编辑入口（INV-DASH-03） | ⬜ |

- 交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 进入编辑 | `dashboard:write` | 显示拖拽与缩放手柄 | 前端状态 | ⬜ |
| 拖拽移动 | 编辑态 | 实时校验重叠（INV-DASH-07），非法位置禁止放下 | PATCH `…/gadgets/{gadget_id}` | ⬜（无 LayoutValidator） |
| 缩放 | 编辑态 | 同上，并校验 `x + w ≤ 12` | 同上 | ⬜ |
| 键盘移动或缩放 | 编辑态 | 方向键移动、Shift 加方向键缩放（FR-51-30） | 同上 | ⬜ |
| 添加 Gadget | `dashboard:write`，数量小于 12 | 选择类型，按默认尺寸放入第一个空位 | POST `…/gadgets` | 🟡 域层 `add_gadget_at`（lib.rs:264-287）上限为 20 |
| 移除 Gadget | `dashboard:write` | 行内确认后移除 | DELETE `…/gadgets/{gadget_id}` | 🟡 域层 `remove_gadget`（lib.rs:183-191），无接口 |
| 共享 | 所有者 | 增加或撤销 ShareGrant | POST、DELETE `…/shares` | ⬜ |
| 开启 Wallboard | 所有者 | 生成 token，显示链接与过期时间 | POST `…/wallboard` | 🟡 仅布尔值（lib.rs:194-197） |
| 设置刷新周期 | 所有者 | 设置 `auto_refresh_seconds` | PATCH `…/{id}` | ⬜ |

- 数据：来源为 REST（目标）：`GET /api/v1/dashboards/{id}` 返回 `layout`、`gadgets`（`id`、`type`、`title`、`position`、`size`、`config`、`chart_report_id`）、`shares`（仅所有者可见）与 wallboard 状态；`GET …/gadgets/{gadget_id}/data` 逐个返回数据。现状：无 store 切片（`frontend/src/lib/store` 中无 dashboard 字段），无 REST，无 mock。

- 与文档设计的差距：
  - FR-51-07、FR-51-08：网格边界与重叠校验缺失。
  - FR-51-09：上限为 20，目标为 12。
  - FR-51-10：无移动与缩放。
  - FR-51-13：失败隔离缺失。
  - FR-51-12：`chart_report_id` 缺失，chart-* 无法绑定报告定义。
  - FR-51-16：Wallboard 只读未强制。
  - FR-51-20：共享缺失。
  - FR-51-30：无键盘替代。

#### 4.7.3 /wallboard/[token]

- 文件：`frontend/src/app/wallboard/[token]/page.tsx`（目标设计，⬜）。该文件位于 `(app)` 组之外，以免挂载 `AppHeader` 与 `RightChatRail`（`AppShell.tsx:48-50、77`）。实施时需确认 Next.js 16 路由组解析无冲突（❓，见 §6.2 第 9 步）。
- 布局：全屏网格，无顶栏、无右侧栏。根布局的 `Sidebar` 与 `MobileHeader` 在该路径下需隐藏：`Sidebar` 已读取 `pathname`（`frontend/src/components/Sidebar.tsx:47-48`），`MobileHeader` 挂载于 `frontend/src/app/layout.tsx:92`，两者目前都不按路径隐藏。
- 组件树（目标设计，⬜）：

```
WallboardPage（只读）
├─ WallboardGrid（12 列，只读）
│  └─ GadgetCard（只读，内容分发同 4.7.2）
└─ WallboardFooter（上次刷新时间、下次刷新倒计时）
```

- 状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | token 有效 | 只读网格，每 30 秒刷新 | ⬜ |
| 加载 | 首次进入 | Skeleton 网格 | ⬜ |
| 空 | 仪表盘无 Gadget | "此仪表盘暂无 Gadget"，不显示编辑入口 | ⬜ |
| 错误（链接过期） | token 过期（目标错误 `WallboardLinkExpired`） | "链接已过期，请联系仪表盘所有者重新生成"（规则 5） | ⬜ 错误变体不存在（lib.rs:205-226） |
| 错误（单个 Gadget） | 数据源失败 | 卡片显示错误，其余卡片继续刷新 | ⬜ |
| 未授权 | token 不存在 | 与过期使用相同文案，不泄露仪表盘是否存在 | ⬜ |
| 未接入 | REST 未接通 | 显示"未接入"状态 | ⬜ |
| 禁用 | 任何情况 | 不显示编辑、共享、设置入口（INV-DASH-03） | ⬜ |

- 交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 自动刷新 | token 有效 | 每 30 秒重新拉取快照，仅重绘变化的 Gadget（RISK-DASH-09） | GET `/api/v1/wallboards/{token}` | ⬜ |
| 浏览器全屏 | 无 | 调用浏览器全屏接口 | 前端 | ⬜ |

- 数据：来源为公开 REST（目标）：`GET /api/v1/wallboards/{token}` 返回 `name`、`layout`、`gadgets`（含 `id`、`type`、`title`、`position`、`size` 与合并后的 `data`）与 `refreshed_at`；不返回 `owner_id`、`shares` 与任何编辑字段（INV-DASH-14）。现状：无。

- 与文档设计的差距：
  - FR-51-14：合并查询与缓存缺失。
  - FR-51-15、FR-51-16、FR-51-17：只读、刷新与布尔开关缺失。
  - FR-51-18：token 与过期缺失。
  - FR-51-19：公开响应未定义。
  - §4.3.4：代理白名单不含公开路径。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 领域 crate（实体、服务、错误） | `crates/domain-dashboard/src/lib.rs` | 🟡 | 模型与服务见 lib.rs:86-287；7 个测试见 lib.rs:296-391 |
| crate 清单与依赖 | `crates/domain-dashboard/Cargo.toml` | 🟡 | 仅依赖 serde 等五个基础库（:18-23），无业务依赖 |
| workspace 成员 | `Cargo.toml:41` | ✅ | `"crates/domain-dashboard"` 已列入 |
| 被其他 crate 依赖 | 全仓 `Cargo.toml` | ⬜ | 无依赖方（grep 仅命中 workspace 成员与自身清单） |
| 持久化（表与迁移） | 全仓 | ⬜ | 无 dashboard 相关 SQL（grep 无结果） |
| REST 路由 | `crates/star-api-rest/src` | ⬜ | `lib.rs:135-137` 挂载 `/api/v1`；`group_api.rs:806-829` 无 dashboard 合并；`routes/ex_07.rs:33` 仅注释提及 Grafana |
| 前端页面 `/dashboards` | `frontend/src/app/(app)/dashboards` | ⬜ | 目录不存在 |
| 前端页面 `/wallboard` | `frontend/src/app/wallboard` | ⬜ | 目录不存在 |
| 导航入口 | `frontend/src/lib/nav/registry.ts:168-230、689、755-798` | 🟡 | 分析入口存在（:219-230），仪表盘入口不存在 |
| 分析页（相邻，非仪表盘） | `frontend/src/app/(app)/analytics/page.tsx` | ✅ | 标签页与 15 个图表接线（:38-42、131-150）；数据由 store 派生（analytics/page.tsx:58-60） |
| 图表组件 | `frontend/src/components/charts/` | 🟡 | 15 个组件（Chart01–Chart15）（路径列表核对），缺 C16–C22 |
| 报告引擎（类型与过滤） | `crates/domain-report/src/lib.rs` | 🟡 | `ReportType` 22 个变体（:34-81）、`ReportFilter`（:226）、`ReportResult`（:303）；无 `ReportDefinition`，无 HTTP 路由 |
| 前端数据层与模拟 | `frontend/src/lib/group-api/`、`frontend/src/mocks/handlers/` | ⬜ | group-api 仅 `group-context.ts`；mock 无 dashboard 处理器 |
| 壳层布局 | `frontend/src/app/layout.tsx`、`frontend/src/app/(app)/layout.tsx`、`frontend/src/components/AppShell.tsx` | ✅ | 布局与挂载见 §4.7.0，可直接复用 |
| 右侧聊天栏 | `frontend/src/components/chat/RightChatRail.tsx` | ✅ | 展开与折叠宽度见 :73、97 |

## 6. 差距与后续

### 6.1 差距

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| G-01 无持久化：无表、无迁移 | 仪表盘无法保存，无法共享 | 按 spec §9 建 `dashboard`、`dashboard_gadget`、`dashboard_share`、`dashboard_view_audit`、`dashboard_edit_audit`，并按 M、T 分类加 RLS（§6.2 第 4 步） |
| G-02 无 REST | 前端无法访问任何仪表盘能力 | 在 `group_api` 下新增 dashboards 路由（§6.2 第 5 步） |
| G-03 `DashboardScope` 与 `ShareGrant` 未裁决 | 共享模型与权限判断双轨，易不一致 | 需所有者确认；本文默认保留 scope 作为 Global 规则载体、共享以 ShareGrant 为准 |
| G-04 Gadget 类型集合与 spec 不一致（§4.2.2） | 目标 10 类仅 6 类名义对应，4 类缺失，3 类多出 | 按 §4.2.2 裁决改造枚举，并同步默认尺寸 |
| G-05 网格校验缺失 | 布局可能越界或重叠，违反 INV-DASH-01、02 | 增加 `LayoutValidator` 与 `GadgetOverlap`、`GadgetLimitExceeded` 错误变体，并让 `InvalidPosition` 真正返回 |
| G-06 无移动与缩放 | 页面无法编辑布局 | 增加 move、resize、reorder 命令与 PATCH 接口 |
| G-07 无授权与租户校验 | 跨租户与越权访问风险 | 写操作携带 actor；增加 scope 检查与 `set_tenant` 模式（group_api.rs:734-744） |
| G-08 Wallboard 仅为布尔值 | 无法公开、无只读、无刷新、无过期 | 增加 token、`WallboardLinkExpired`、30 天过期与只读快照接口 |
| G-09 无导航与页面 | 用户无入口 | 按 §6.2 第 7、8、9 步建立导航与页面 |
| G-10 无前端数据层与 mock | 页面无法在无后端时显示"未接入"或测试 | 增加 `dashboardClient.ts` 与带"模拟"标识的 mock 处理器（AGENTS.md:22） |
| G-11 `/api/group` 白名单不含 dashboards | 请求被代理拒绝 | 扩展 `isAllowedGroupApiPath`（route.ts:68-75），并为公开 wallboards 路径开放无认证访问 |
| G-12 报告数据契约缺失 | chart-* 无法绑定报告定义，C16–C22 无组件 | 在 domain-report 增加 `ReportDefinition` 或等价契约与 HTTP 接口；补齐 C16–C22 组件 |
| G-13 依赖端口未接入 | assigned-to-me、activity、filter-results 无数据 | 为 work-item、audit、identity、permission 定义只读端口（spec §6），逐个接入 |
| G-14 Wallboard 布局被壳层包裹 | 大屏会显示顶栏、侧栏与聊天栏 | 将 `/wallboard/[token]` 放在 `(app)` 组之外，并在 `Sidebar` 与 `MobileHeader` 中按路径隐藏 |
| G-15 订阅与通知缺失 | P2 功能无法推进 | 在 domain-notification 边界确定后实现（FR-51-31） |
| G-16 审计表缺失 | 无法追溯查看与编辑 | 增加 T 类审计表与写入（FR-51-24） |

### 6.2 实施顺序

1. 目标冻结：以 spec v1.0 为唯一设计来源；记录 §4.2.2 的 Gadget 类型裁决、§4.3.3 的 `/api/v1` 基路径、§8 的 Wallboard token 寻址。验证：spec 的 §2、§5、§9 与本文无冲突条目。
2. 领域层补齐（纯 Rust，无 I/O）：增加 `LayoutValidator`（重叠、`x + w ≤ 12`、上限 12）；增加 move、resize、reorder；补齐 `GadgetOverlap`、`GadgetLimitExceeded`、`WallboardLinkExpired` 错误变体；标题与描述校验；软删除标记；`ShareGrant` 与 50 上限；让 `InvalidPosition` 真正返回。验证：§7.2 的 AC-01 至 AC-05。
3. 端口边界：在 domain-dashboard 中定义 `DashboardCommandPort` 与 `DashboardQueryPort` trait（spec §4）；依赖其他域时只引用其端口，不直接耦合实现。验证：编译通过，且 crate 的 `Cargo.toml` 依赖新增项只包含端口所在 crate。
4. 持久化：增加迁移，表结构取 spec §9；`dashboard`、`dashboard_gadget`、`dashboard_share` 为 M 类（SCD Type 2，禁止物理删除），`dashboard_view_audit`、`dashboard_edit_audit` 为 T 类（append-only）；RLS 以 `set_tenant` 模式实现。验证：RLS 集成测试（AC-06）。
5. REST：在 `crates/star-api-rest` 新增 dashboards 路由模块，并在 `build_group_router`（group_api.rs:806-829）中合并；写操作与读操作使用 `AuthenticatedUser`、`validate_actor`（group_api.rs:772-777）、`require_scope`（:779-789）与 `set_tenant`；`/api/v1/wallboards/{token}` 不经 scope，仅校验 token。验证：AC-06、AC-07、AC-08。
6. 前端数据层：新增 `frontend/src/lib/group-api/dashboardClient.ts`（沿用 group-context.ts:158 模式）；扩展代理白名单（route.ts:68-75）；新增 `frontend/src/mocks/handlers/dashboards.ts` 并在 `handlers/index.ts` 注册，mock 数据页面须显示"模拟"标识。验证：AC-09。
7. 导航：在 `ALL_MODULES`（registry.ts:168 起的数组）增加 `dashboards`；在 `MODULE_GROUPS` 的 analytics 组（registry.ts:781）与 `NAV_GROUPS["home-personal"]`（:689）中加入 `dashboards`；在 `frontend/src/lib/i18n/en.ts`、`zh-CN.ts`、`ja.ts` 补 `dashboards` 键（模式见 en.ts:205、408）；`navStore.test.ts:68` 的数量断言仍成立。验证：AC-11。
8. 页面：`/dashboards` 与 `/dashboards/[id]`，并将 `/dashboards` 加入 `WIDE_PREFIXES`（`(app)/layout.tsx:41`）；网格使用 @dnd-kit 与 CSS Grid，键盘替代同步实现；Gadget 内容先接 chart-C01 至 C15、text、clock、list，其余类型显示"未接入"。验证：AC-12、AC-13。
9. Wallboard：新增 `frontend/src/app/wallboard/[token]/page.tsx`，并在 `Sidebar`（Sidebar.tsx:47-48）与 `MobileHeader`（layout.tsx:92）中按路径隐藏；每 30 秒刷新；只读。验证：AC-10。路由组解析是否无冲突（❓）需在本步通过 `next build` 确认。
10. 数据源接线（P1）：依次接入 work-item、audit、identity、permission 的只读端口，接入报告契约（G-12）；订阅与通知（P2）最后处理。

## 7. 验收与测试

### 7.1 现有测试

| 测试 | 位置 | 验证内容 | 未验证 |
|---|---|---|---|
| `test_gadget_type_all_count` | `crates/domain-dashboard/src/lib.rs:301-303` | 代码枚举数量为 10 | 与 spec §5 集合是否一致 |
| `test_gadget_default_size` | lib.rs:306-312 | `Clock` 为 1×1，`JqlTable` 为 4×2 | 其余类型与 spec 尺寸 |
| `test_dashboard_create_personal` | lib.rs:315-330 | Personal 创建成功 | 租户隔离、标题长度 |
| `test_dashboard_global_requires_admin` | lib.rs:333-353 | Global 非管理员拒绝，管理员允许 | Team、Project 的规则 |
| `test_add_gadget` | lib.rs:356-366 | `add_gadget_at` 插入一个 Burndown | 位置、重叠、上限 |
| `test_remove_gadget` | lib.rs:369-380 | 移除后数量为 0 | 未找到分支 |
| `test_wallboard_mode` | lib.rs:383-390 | `enable_wallboard` 置位 | 关闭、token、过期、只读 |

前端：无 dashboard 测试（全仓 grep 无结果）。star-api-rest：无 dashboard 测试。端到端：无。

### 7.2 验收标准

| 编号 | 标准 | 对应 | 测试层 |
|---|---|---|---|
| AC-01 | 第 13 个 Gadget 添加失败，返回上限错误，仪表盘状态不变 | FR-51-09 | 单元（domain-dashboard） |
| AC-02 | `x + w > 12` 的放置被拒绝，返回 `InvalidPosition` 或校验错误 | FR-51-07 | 单元 |
| AC-03 | 与既有 Gadget 重叠的移动返回 `GadgetOverlap`，状态不变 | FR-51-08 | 单元 |
| AC-04 | 标题为空或超过 200 字时被拒绝 | FR-51-23 | 单元 |
| AC-05 | 注入时钟后，Wallboard 链接在 30 天后校验失败并返回 `WallboardLinkExpired` | FR-51-18 | 单元 |
| AC-06 | 跨租户 GET `/api/v1/dashboards/{id}` 返回 404 或 403，不泄露存在性；RLS 生效 | FR-51-21 | 集成（star-api-rest + 数据库） |
| AC-07 | 缺少 `dashboard:write` 的用户 PATCH 返回 403 | FR-51-22 | 集成 |
| AC-08 | `/api/v1/wallboards/{token}` 响应不含 `owner_id`，且无写接口 | FR-51-19 | 集成 |
| AC-09 | 某个 Gadget 数据接口失败时，其余卡片正常渲染，失败卡片可重试 | FR-51-13 | 前端（vitest + MSW） |
| AC-10 | Wallboard 每 30 秒刷新一次（使用 fake timers），编辑入口不在 DOM 中 | FR-51-16、FR-51-17 | 前端（vitest） |
| AC-11 | 分析组出现"仪表盘"入口，进入 `/dashboards`；Global 标签对非管理员禁用 | FR-51-04、FR-51-25 | 前端（vitest，registry 与 Sidebar 测试） |
| AC-12 | 端到端：新建仪表盘、添加两个 Gadget、拖拽、开启 Wallboard，公开链接可打开 | FR-51-26、FR-51-27 | 端到端（Playwright） |
| AC-13 | Gadget 可用 Tab 聚焦，键盘可移动与缩放 | FR-51-30 | 前端（vitest 与可访问性检查） |

### 7.3 未验证的范围

- 上述 AC 全部未在 pin 上实现或执行。
- 订阅、审计与 C16–C22 的数据契约不在本文验收范围，需在 §6.2 第 10 步后补充。
- `DashboardScope` 与 `ShareGrant` 的最终关系未裁决，AC-06、AC-07 的共享部分依赖该裁决。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/specs/domain-dashboard-spec.md`（v1.0） | §1 职责、§2 实体、§3 INV-DASH-01 至 15、§4 端口与错误、§5 十类 Gadget、§6 接触面、§8 网格与 Wallboard、§9 DDL | 代码为"精简实装 v0.1"（lib.rs:1），与 spec 的 Gadget 集合、上限（20 对 12）、共享模型、端口均不一致；spec §10 记录 v1.0 为对 v0.1 的升版 | 作为目标设计；代码现状以 §5 为证据 |
| `docs/requirements/charts-and-reports.md`（v1.0） | §5.3 分享、§6.1 Gadget 表、§6.2 12 列网格、§6.3 Wallboard、§7.1 性能、§8.1 可访问性、§9.2 不持事实、§10.4 测试 | §6.1 仅列 7 类，缺 heatmap、two-dimensional-filter、rich-text，且采用下划线命名（如 `assigned_to_me`），与 spec §5 的连字符命名不同 | Gadget 集合取 spec §5；序列化值取 snake_case，与 lib.rs:21 一致 |
| `docs/frontend/design/ui-3pane-arch.md` §1.3（第 183 行） | 仪表板对象的视图族：主仪表板、Wallboard、嵌入视图、导出 | 代码无对应页面 | 采用独立路由 `/dashboards`；Wallboard 为 `/wallboard/[token]`；嵌入与导出列入后续 |
| `docs/frontend/design/ui-3pane-arch.md` §0.1 第 30 行（"概览 → PageHeader"） | 将 /projects 的概览标签折入页头 | 与"独立页"表面冲突。代码中该概览标签为 `projects-overview-tab`（`frontend/src/app/(app)/projects/ProjectsClient.tsx:868`） | **裁决：该条只约束 /projects 的项目概览标签，不涉及跨项目的用户自定义仪表盘。** `/dashboards` 为独立路由，不改动 /projects 页头；`/analytics` 的概览标签（analytics/page.tsx:153-160）同样保持独立。两者互不替代 |
| `docs/frontend/design/ui-3pane-arch.md` §0.2（第 13 行）与 §1.2（v0.2） | §0.2：侧栏两段，项目树与全局组（收件箱、Agent 中心、分析）；§1.2 五组结构 | §1.2 的五组结构已被 §0.2 覆盖；代码 `NAV_GROUPS` 为另一套分组（registry.ts:684-714） | 采用 §0.2 与代码现状：入口放在 `home-personal` 全局组，并加入分析组标签 |
| `docs/basic-design/charts-and-reports.md`（§3.4 第 372 行、§4.5 第 642–651 行、API 表第 672–677 行、目录第 201–237 行） | 数据模型、加载流程、REST 表、目录结构 | API 基路径为 `/api/dashboards`；目录为 `(app)/dashboards/` 与 `dashboard.rs`、`wallboard.rs` 拆分；代码为单文件 `lib.rs` | API 基路径改为 `/api/v1/dashboards`（lib.rs:135-137 与 group_api.rs:806-829 的挂载方式）；Wallboard 以 token 寻址而非 `/api/dashboards/{id}/wallboard`，以避免暴露 id 并满足 INV-DASH-14；目录拆分在 crate 扩展时进行 |
| `docs/features/F16-analytics.md`（同级功能文档，其代码基线为 1643559f） | FR-16-23、FR-16-27、FR-16-28、FR-16-30、FR-16-31 的状态结论 | 结论与本基线一致（Gadget 集合不同、create 有管理员规则、wallboard 为布尔值、无持久化）；行号以 637ed19d 为准 | 作为交叉核对；状态以本文为准 |
| `docs/wiki/pgwiki/10-workspace/_crates/crate-domain-dashboard.md`（生成页） | 声称 `depends_on: star-context` | `crates/domain-dashboard/Cargo.toml:18-23` 不含 star-context | 以 Cargo.toml 为准；生成页不作依据 |
| `docs/design/charts/c01-burndown.md` 至 `c22-recently-created.md`（22 份） | 每个图表的详细设计 | 仅核对存在性（22 份均存在），未逐份读取 | 不属于本功能页面范围；仅用于 chart-* 映射的存在性判断 |
| `docs/specs/domain-report-spec.md` | 报告定义契约 | 未读取；代码仅有 `ReportType`、`ReportFilter`、`ReportResult`（domain-report lib.rs:34、226、303），无 `ReportDefinition` | 待 §6.2 第 10 步读取后补充 |
| `docs/frontend/design/ui-redesign-multica-style.md`（第 47、116 行） | 分析页与 /analytics 仪表盘的描述 | 指分析页，与用户自定义仪表盘不同 | 仅 grep 命中，不作为依据 |
| `AGENTS.md`（pin 版，第 22 行） | "未接通授权数据的功能须显示未接入状态，不得以示例数据冒充" | 无 | 采纳为页面状态规则（§4.7） |
| 代码与配置：`crates/domain-dashboard`、`crates/star-api-rest`（group_api、middleware、routes）、`crates/domain-report`、`frontend/src/app`、`frontend/src/components`、`frontend/src/lib`、`frontend/src/mocks`、`frontend/package.json`、`Cargo.toml` | 全部现状证据 | — | 现状以这些路径在 637ed19d 的内容为准 |
