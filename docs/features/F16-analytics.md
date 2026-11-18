# F16 分析 (Analytics)

> 状态摘要: `/analytics` 页面的 19 个标签、4 个指标卡、15 个图表接线与甘特图入口已实现，但数据来自 store 派生与页面字面量，尚未接入授权投影。`domain-report` 与 `domain-dashboard` 为内存实现，没有 HTTP 路由与调用方；报告 REST、仪表盘持久化与共享、订阅与调度均未实现。

> 代码基线: HEAD 1643559f; 核对方法: 对所列路径做 path/symbol grep，并读取工作区文件；工作区与 HEAD 1643559f 对所引用文件无差异（`git diff --quiet HEAD` 通过），cdd00a78 至 1643559f 之间分析相关路径仅 `frontend/src/lib/nav/registry.ts` 的 projects 模块描述有变化，不影响 analytics 条目; 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

为项目提供效能分析入口：在一个页面内查看 Sprint、任务与 Agent 指标，敏捷图表（燃尽、燃起、速度、Sprint 报告、CFD、控制图、周期时间、吞吐量、预测、时间跟踪、解决时间、SLA、创建 vs 解决、问题分布、优先级分布），甘特图入口，运维指标与操作日志。图表的权威数据应来自报告引擎（`domain-report`）的授权投影；仪表盘引擎（`domain-dashboard`）承载用户自定义看板。

### 1.2 范围

| 范围内 | 范围外 |
|---|---|
| `/analytics` 页面与 19 个标签（`frontend/src/app/(app)/analytics/page.tsx:38-42`） | Run 级 BI 任务事件 API（`crates/star-api-rest/src/group_api/bi_analytics.rs`）与 `RunBiPanel`：独立功能，仅在 §5 列出与本功能的连接状态 |
| 图表组件 Chart01–Chart15 与 `ChartFrame`；图表数据派生 `chartData.ts` | 甘特图排期细节（属排期功能） |
| 甘特图入口 `GanttView` | 通知投递（domain-notification）与审计域实现（domain-audit 仅作设计依赖列在 §4.6） |
| MSW analytics mock | `/ops` 运维控制台的 MetricsTab、LogAITab（分析页未导入，`page.tsx:14-36`） |
| `crates/domain-report`、`crates/domain-dashboard` | 邮件与 Slack 等订阅渠道 |
| 进入分析页的入口链接（BranchWorkspace、看板、worktree 分组页） | |

优先级取自源文档的分批（`domain-report-spec.md` §8）与 SRS 基线；SRS 未标注优先级的条目按功能依赖定为 P0 或 P1。

## 2. 用户与场景

| 角色 | 主要场景 | 当前可用性 |
|---|---|---|
| 敏捷负责人 / 项目经理 | 查看燃尽、燃起、速度、CFD，判断进度与风险 | 图表可见，数据为 store 派生与常量（FR-16-08） |
| 开发者 | 查看周期时间、吞吐量、创建 vs 解决 | 同上 |
| 平台 / 运维 | 查看运维指标与发布状态 | 仅 4 个字面量数值（FR-16-10） |
| 管理员 / 审计人员 | 查看关键操作日志 | 字面量表格（FR-16-04） |
| 报告订阅者 | 定时收到图表报告 | 未实现（FR-16-26） |
| 仪表盘用户 | 自定义 Gadget 看板 | 仅内存模型（FR-16-27 至 FR-16-32） |

## 3. 需求

编号 FR-16-NN。状态以代码为准。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-16-01 | 概览页展示 4 个指标卡：完成任务、验证通过率、Agent 运行时长、IPA 技能覆盖（SRS FR-AN-001） | P0 | 🟡 | 指标由 store 计算：完成任务 `page.tsx:67`；“验证通过率”实为完成率 `page.tsx:69, 76`；“Agent 运行时长”实为 token 均值 `page.tsx:70-71, 77`；趋势为常量 `page.tsx:75-78`；渲染 `page.tsx:155-168` |
| FR-16-02 | 概览页展示 IPA 阶段进度 P2–P6.2（SRS FR-AN-002） | P0 | 🟡 | 渲染 `page.tsx:170-192`；阶段计数为常量 `page.tsx:45-52`；分支名为常量 `page.tsx:173, 207` |
| FR-16-03 | 运维类指标在未接入时显示“未接入”标识（SRS FR-AN-003） | P1 | ⬜ | `page.tsx` 中无“未接入”文字（grep 计数 0）；运维指标为字面量 `page.tsx:237-242` |
| FR-16-04 | 操作日志表：时间、操作者、事件、范围（SRS FR-AN-004） | P1 | 🟡 | 表格渲染 `page.tsx:253-291`；列为时间/用户/操作/资源/结果 `page.tsx:261-266`；行为字面量 `page.tsx:270-275`；未读取 domain-audit |
| FR-16-05 | 标签页导航：概览、15 个图表、甘特图、运维、审计日志（共 19 项） | P0 | ✅ | 标签定义 `page.tsx:38-42`；导航与切换 `page.tsx:131-150`；测试标识 `analytics-tab-*` `page.tsx:140` |
| FR-16-06 | 图表为预览数据时明确标注，不冒充授权事实 | P0 | ✅ | 预览横幅 `page.tsx:82-87`；每个图表标签注入横幅 `page.tsx:89-108`，甘特图同样 `page.tsx:217` |
| FR-16-07 | 15 个图表组件（C01–C15）由标签页渲染 | P0 | ✅ | 组件导入 `page.tsx:19-34`；渲染映射 `page.tsx:91-105`；组件文件 `frontend/src/components/charts/Chart01.tsx` 至 `Chart15.tsx` 共 15 个 |
| FR-16-08 | 图表数据由 store 中 Sprint 与 WorkItem 聚合派生 | P0 | 🟡 | 派生入口 `chartData.ts:48-223`；store 选择 `page.tsx:58-62`；常量或合成序列：速度 `chartData.ts:89-95`，CFD 中 in_progress/review `chartData.ts:112-123`，周期时间分桶 `chartData.ts:125-135`，控制图 `chartData.ts:137-147`，吞吐量 `chartData.ts:158-161`，预测 `chartData.ts:163-173`，时间跟踪 `chartData.ts:175-181`，解决时间 `chartData.ts:183-190`，SLA `chartData.ts:192-202` |
| FR-16-09 | 甘特图标签嵌入 GanttView，数据取自 store 的 board、workItems、identities | P1 | ✅ | `page.tsx:214-223`；`frontend/src/components/gantt/GanttView.tsx` 存在 |
| FR-16-10 | 运维标签展示集群、指标、发布三类卡片，未接入时诚实标注（原型要求） | P1 | 🟡 | 仅 4 个字面量数值 `page.tsx:237-242`，无集群与发布卡；标题注释称对接 MetricsTab/LogAITab `page.tsx:234`，但未导入 `page.tsx:14-36`；原型运维区含三卡 |
| FR-16-11 | 范围（项目）与时间范围（近 14 天）选择器，影响图表数据 | P1 | 🟡 | 两个 `<select>` 渲染于 `page.tsx:118-127`，选项为常量，无 value 与 onChange；`buildAnalyticsData` 只接收 sprints 与 workItems `chartData.ts:48` |
| FR-16-12 | 图表外壳提供导出与订阅按钮 | P2 | 🟡 | `ChartFrame` 接受 `onExport`、`onSubscribe` 并在传入时渲染 `ChartFrame.tsx:21-22, 27, 43-52`；页面未传入 `page.tsx:91-105` |
| FR-16-13 | 报告范围 S1–S5（项目、层级、Sprint、版本、过滤器） | P1 | 🟡 | `ReportFilter` 以可选字段表达 `project_id`、`sprint_id`、`version_id`、`filter_id` `domain-report/src/lib.rs:226-240`；无 `ReportScope` 枚举；无 S2 层级聚合（INV-REPORT-04）；无 JQL 解析 |
| FR-16-14 | 22 类图表类型枚举 | P0 | ✅ | `ReportType` `lib.rs:34-83`；`all()` 列出 22 项 `lib.rs:86-112` |
| FR-16-15 | P0 八类图表（C01、C02、C03、C04、C05、C06、C07、C13）计算逻辑 | P0 | 🟡 | 分派 `lib.rs:536` 起的 `generate_p0`；模块如 `domain/c01_burndown.rs:90`；数据来自内存端口桩（FR-16-22），无调用方 |
| FR-16-16 | P1 六类图表（C08、C09、C10、C11、C12、C14）计算逻辑 | P1 | 🟡 | 模块存在，如 `domain/c08_throughput.rs:52`；`generate` 对非 P0/P2 类型调用 `generate_stub` `lib.rs:485-531, 648`，返回固定 10 个点且 `stub: true`；真实模块不可达 |
| FR-16-17 | C15 优先级分布 | P1 | 🟡 | C15 不在任何批次列表 `lib.rs:114-165`，落入 `generate_stub`；模块存在 `domain/c15_priority_dist.rs:32` |
| FR-16-18 | C16–C22：工作量、版本、发布燃尽、状态耗时、热力图、最近创建 | P2 | ⬜ | 分派后返回 `ReportError::not_implemented`，如 `domain/c16_assignee_workload.rs:13-21`，C17–C22 同构 |
| FR-16-19 | 报告结果缓存 5 分钟 | P1 | 🟡 | 写入 TTL 300 秒 `lib.rs:530`；内存实现 `infrastructure/in_memory_cache.rs`；`domain-report` 无 Redis 依赖 |
| FR-16-20 | 数据量超限时拒绝或采样（INV-REPORT-09、10） | P1 | ⬜ | `DataTooLarge` 定义 `lib.rs:379`，映射 413 `lib.rs:429`，错误码 `lib.rs:444`；crate 内无构造点 |
| FR-16-21 | 导出 CSV、JSON，以及 XLSX、PNG、PDF | P2 | 🟡 | 进程内 `export_json` `lib.rs:679`、`export_csv` `lib.rs:684`；XLSX、PNG、PDF 无实现；无 HTTP 或 UI 入口 |
| FR-16-22 | 数据端口：工作项、Sprint、用户、权限 | P0 | 🟡 | 端口 trait `application/ports.rs:16, 36, 47, 65`；实现为内存桩 `infrastructure/port_stubs.rs`，文件头注明 V2 替换 |
| FR-16-23 | 报告与仪表盘按租户隔离（RLS 必携） | P0 | 🟡 | `ReportFilter.tenant_id` `lib.rs:227`，参与缓存键；`Dashboard.tenant_id` `domain-dashboard/src/lib.rs:139`；无数据库，RLS 未执行 |
| FR-16-24 | 报告调用前执行权限校验（INV-REPORT-08、19） | P0 | 🟡 | `PermissionPort` 作为构造参数注入 `lib.rs:463, 473, 480`；`generate` `lib.rs:485-531` 未调用 |
| FR-16-25 | 报告 REST 端点：列表、详情、创建、更新、删除、数据、刷新、导出、订阅 | P0 | ⬜ | 设计见 `docs/basic-design/charts-and-reports.md` §5.1；`crates` 与 `frontend/src` 中无 `/api/reports` 路由 |
| FR-16-26 | 报告订阅、定时调度（cron） | P2 | ⬜ | `domain-report` 与 `domain-dashboard` 源码中无 cron 或 subscription 实现（grep 无结果） |
| FR-16-27 | 仪表盘 Gadget 类型集合 | P1 | 🟡 | `GadgetType` 十个变体 `domain-dashboard/src/lib.rs:22-40`，`all()` `lib.rs:47`，默认尺寸 `lib.rs:63`；与设计 §5 的十类集合不同（§8） |
| FR-16-28 | 仪表盘创建；全局仪表盘要求管理员 | P1 | ✅ | `DashboardService::create` `lib.rs:242-261`，Global 且非管理员返回 `PermissionDenied` `lib.rs:251-257`；测试 `lib.rs:333-344` |
| FR-16-29 | 12 列网格、Gadget 不重叠与边界校验 | P1 | ⬜ | crate 中无重叠或边界校验；`InvalidPosition` 无构造点 `lib.rs:215`；`add_gadget` 仅限制数量 `lib.rs:173-180`（上限 20） |
| FR-16-30 | Wallboard 只读模式、30 天公开链接、30 秒刷新 | P2 | 🟡 | 布尔字段 `wallboard_mode` `lib.rs:143`；`enable_wallboard` `lib.rs:194-197`；无只读强制、link_token、过期与刷新 |
| FR-16-31 | 仪表盘持久化、共享与订阅 | P2 | ⬜ | `Dashboard` 为纯内存结构 `lib.rs:129-148`；`DashboardScope` 枚举存在 `lib.rs:86-95` 但无共享授予；无存储与数据库表 |
| FR-16-32 | 仪表盘前端页面（列表、编辑、Wallboard） | P2 | ⬜ | `frontend/src/app/(app)/` 下无 dashboards 目录 |
| FR-16-33 | 分析页 KPI 与图表数据来自分析读 API，而非 store | P1 | ⬜ | 页面不请求 `/api/analytics/*`；设计 `docs/frontend/design/ui-detailed-design.md:173` 标注“后端 KPI/Cost API 待接入真实 Prometheus/ClickHouse 流” |
| FR-16-34 | MSW 提供 `/api/analytics/kpi` 与 `/api/analytics/cost` 的 mock | P2 | 🟡 | 处理器 `frontend/src/mocks/handlers/analytics.ts:15-22`；合并于 `handlers/index.ts:35, 58`；页面不引用 `MOCK_KPI` 或 `COST_SERIES`，无运行时调用方 |
| FR-16-35 | 从 Run 分支 BI 页签、看板与 worktree 分组页进入分析页 | P1 | ✅ | `frontend/src/components/run/BranchWorkspace.tsx:180-190`（链接 `lane-bi-open-analytics` 位于 186 行）；`frontend/src/app/(app)/kanban/page.tsx:500`；`frontend/src/app/worktree/[id]/group/page.tsx:2793` |

## 4. 设计

### 4.1 概念模型

- 页面层：`AnalyticsPage` 持有 `activeTab`（`page.tsx:56`），从 store 选择 `workItems`、`agentSessions`、`sprints`、`board`、`identities`（`page.tsx:58-62`），派生 `charts` 与 `kpiData`（`page.tsx:64-80`）。
- 派生层：`buildAnalyticsData(sprints, workItems)` 返回 15 组图表数据（`chartData.ts:48`），类型来自 `frontend/src/lib/chart-data-schema.ts`。
- 展示层：`ChartFrame` 外壳加 15 个图表组件；13 个组件 import recharts，`Chart04SprintReport.tsx` 与 `Chart10TimeTracking.tsx` 未 import。
- 报告引擎（后端，未接入）：`ReportService::generate` 按 `ReportType` 分派（`domain-report/src/lib.rs:485`），数据端口由调用方注入。
- 仪表盘引擎（后端，未接入）：`Dashboard` 聚合根与 `Gadget`（`domain-dashboard/src/lib.rs:103-148`）。
- 两个引擎互不依赖，且都没有调用方（§4.6）。

### 4.2 数据模型

| 实体 | 来源 | 代码现状 | 状态 |
|---|---|---|---|
| ReportDefinition（聚合根） | domain-report-spec §2 | 无；代码只有 `Report` `lib.rs:269` | ⬜ |
| ReportFilter | 代码 `lib.rs:226-240` | `tenant_id`、`project_id`、`sprint_id`、`version_id`、`filter_id`、`time_range` | 🟡 |
| ReportResult、ReportPoint、ReportSummary、Trend | 代码 `lib.rs:292-349` | 存在 | ✅ |
| ReportSnapshot、ReportSchedule、ReportSubscription | spec §2 | 无 | ⬜ |
| ReportError | 代码 `lib.rs:351` 起 | 含 `DataTooLarge`、`NotImplemented`；变体名与 spec §4.6 部分不同 | 🟡 |
| Dashboard | 代码 `domain-dashboard/src/lib.rs:129-148` | `id`、`name`、`scope`、`owner_id`、`tenant_id`、`gadgets`、`wallboard_mode`、时间戳；spec 的 `title`、`layout`、`shared_with`、`auto_refresh_seconds`、`version` 缺失 | 🟡 |
| Gadget、GadgetPosition、GadgetSize | 代码 `lib.rs:76-125` | 存在；x 0–11、w 1–12 仅以注释描述 | 🟡 |
| DashboardScope | 代码 `lib.rs:86-95` | Personal、Team、Project、Global；spec 无此枚举 | 🟡 |
| ShareGrant | spec §2 | 无 | ⬜ |
| 数据库表（dashboard、dashboard_gadget、dashboard_share、dashboard_subscription、dashboard_view_audit、dashboard_edit_audit、report_definition、report_snapshot） | spec §9、basic-design §3.5 | 代码库中无 CREATE TABLE 或 SQL 文件（grep 无结果） | ⬜ |
| 页面 store 实体 | `frontend/src/lib/store.ts` | `identities` `:147`、`agentSessions` `:156`；WorkItem、Sprint、board 由页面选择 | ✅ |

### 4.3 接口

| 接口 | 状态 | 证据 |
|---|---|---|
| 前端路由 `/analytics` | ✅ | `frontend/src/app/(app)/analytics/page.tsx:54`；导航模块 `frontend/src/lib/nav/registry.ts:219-230` |
| 导航分组 `analytics`（名称“分析”） | 🟡 | 声明 tabs `analytics`、`ops`、`audit`、`sandbox` `registry.ts:778-782`；页面未消费这些 id；`audit`、`ops`、`sandbox` 为独立路由 `registry.ts:491-494, 584-587, 594-597` |
| MSW `GET /api/analytics/kpi`、`GET /api/analytics/cost` | 🟡 | `frontend/src/mocks/handlers/analytics.ts:16, 19` |
| 报告端口 WorkItemQueryPort、SprintQueryPort、UserQueryPort、PermissionPort | 🟡 | `domain-report/src/application/ports.rs:16, 36, 47, 65` |
| `ReportService::generate(report_type, filter)`、`export_json`、`export_csv` | 🟡 | `domain-report/src/lib.rs:485, 679, 684`；无 HTTP 调用 |
| 设计 REST `/api/reports/*`、`/api/dashboards/*` | ⬜ | `docs/basic-design/charts-and-reports.md` §5.1；代码中无对应路由 |
| `DashboardService::create`、`add_gadget_at`；`Dashboard::add_gadget`、`remove_gadget`、`enable_wallboard` | 🟡 | `domain-dashboard/src/lib.rs:242, 264, 173, 183, 194`；无调用方 |
| 设计 DashboardCommandPort、DashboardQueryPort、DashboardSubscriptionPort | ⬜ | spec §4；crate 中无 `pub trait` |
| Run 级 BI 任务事件 `GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-events` | 🟡 | 路由 `bi_analytics.rs:61-66`；合并 `group_api.rs:804`；仅配置 `BI_DATABASE_URL` 时注入查询端口 `main.rs:115, 130-132`；前端唯一调用点 `frontend/src/lib/run/runDirectoryApi.ts:693` |

### 4.4 状态机与关键流程

- 标签切换：`activeTab` 为本地状态，默认“概览”（`page.tsx:56`），切换于 `page.tsx:141`，不写入 URL。
- 页面数据流：store 变化后，`useMemo` 重算 `charts`（`page.tsx:64`）与 `kpiData`（`page.tsx:66-80`）。
- 报告生成（`ReportService::generate`，`lib.rs:485-531`）：
  1. 缓存键为 `report:{tenant}:{scope}:{chart_id}`，scope 依次取 sprint、version、project、filter，最后回退到 tenant。
  2. 命中缓存且可反序列化则直接返回。
  3. 分派：P0 进入 `generate_p0`（`lib.rs:536`）；P2 在 `generate_p0` 内返回 NotImplemented，并在缓存写入前以 `?` 返回；其余（P1 与 C15）进入 `generate_stub`（`lib.rs:648`）。
  4. 写入缓存，TTL 300 秒（`lib.rs:530`）。stub 结果同样被写入缓存。
- 仪表盘（实现）：创建；`add_gadget_at` 以 x=0、y=最大 y+1 放置（`lib.rs:264-287`）；删除；`enable_wallboard` 置标志（`lib.rs:194-197`）。无持久化，无重叠校验。
- Run BI 请求（`bi_analytics.rs:112-153`）：`validate_actor` 与 `project:read` 检查（`:118-119`）→ 开启事务并设置 `statement_timeout` 3000 毫秒、`lock_timeout` 1000 毫秒（`:122-134`）→ `authorize_run_owned_task_scope`（`:135`）→ 端口调用限时 6 秒（`:143`），端口缺失或超时返回 `feature_unavailable`（`:139-146`）→ 提交事务。

### 4.5 权限与安全

- 分析页：`page.tsx` 不含权限判断；`(app)` 分组的鉴权逻辑未在本文件核对（见 §9）。
- 导出与订阅入口：页面未向 `ChartFrame` 传入回调，因此不可见（`page.tsx:91-105`；`ChartFrame.tsx:43, 52`）。
- 报告：`ReportFilter` 携带 `tenant_id` 并参与缓存键；`PermissionPort` 注入但 `generate` 未调用（`lib.rs:485-531`）；无数据库，RLS 未执行。
- 仪表盘：仅 Global 范围创建需 `is_admin`（`lib.rs:251-257`）；其它操作无权限检查；无共享授予。
- Wallboard 公开链接、30 天过期、访问审计：未实现（FR-16-30）。
- Run BI 端点：`validate_actor`、`require_scope("project:read")`、按 Run 归属授权、`set_tenant` 设置租户上下文、语句超时（`bi_analytics.rs:118-135`）；默认页大小 50（`:58`），上限 100（测试 `:203-209`）。

### 4.6 依赖关系

- 分析页依赖：`@/lib/store`（`page.tsx:15`）、`@/lib/i18n`（`:17`）、`./chartData`（`:18`）、`ChartFrame` 与 15 个图表组件（`:19-34`）、`GanttView`（`:35`）。
- `chartData.ts` 依赖 `frontend/src/lib/chart-data-schema.ts`（`chartData.ts:13-19`）与 `@/types/ids`。
- `domain-report` 与 `domain-dashboard` 为 workspace 成员（根 `Cargo.toml:40-41`）；无其它 crate 依赖它们，`star-dispatcher/src/sa_real_impls.rs:250, 263` 仅为字符串列表。
- 设计依赖（未接入）：domain-work-item、domain-planning、domain-identity、domain-permission（spec §5）；当前由内存桩替代（`port_stubs.rs`）。
- domain-audit：设计上 activity gadget 与审计写入依赖它（spec §6），代码未接入。
- Run BI 端点依赖 `bi-owner-runtime` 的 `TaskEventQueryPort`（`bi_analytics.rs:44`）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 分析页面（标签、KPI、IPA、运维、审计日志） | `frontend/src/app/(app)/analytics/page.tsx` | 🟡 | 19 标签 `:38-42`；数据为 store 派生与字面量（FR-16-01 至 04、08、10、11） |
| 图表派生 | `frontend/src/app/(app)/analytics/chartData.ts` | 🟡 | `buildAnalyticsData` `:48-223`；多处常量 |
| 图表组件 C01–C15 | `frontend/src/components/charts/Chart01.tsx` 至 `Chart15.tsx` | ✅ | 15 个文件由页面渲染 `page.tsx:91-105`；组件内部未逐一核对 |
| 图表外壳 | `frontend/src/components/charts/shared/ChartFrame.tsx` | 🟡 | 导出与订阅按钮仅在传入回调时渲染 `:21-22, 43, 52`；页面未传入 |
| 甘特图嵌入 | `frontend/src/components/gantt/GanttView.tsx` | ✅ | `page.tsx:214-223` |
| 图表类型定义 | `frontend/src/lib/chart-data-schema.ts` | 🟡 | 被 `chartData.ts` 引用；与后端 `ReportResult` 无对接 |
| MSW mock | `frontend/src/mocks/handlers/analytics.ts`、`mocks/data/analytics.ts`、`mocks/schemas/analytics.ts` | 🟡 | 处理器 `:15-22`；数据文件第 3 行注明原位于页面内；页面不引用 |
| 导航模块 | `frontend/src/lib/nav/registry.ts:219-230` | ✅ | ALL_MODULES 中 analytics 条目（无 tabs 字段） |
| 导航分组 | `frontend/src/lib/nav/registry.ts:778-782` | 🟡 | 声明 tabs，页面未消费 |
| 布局全高前缀 | `frontend/src/app/(app)/layout.tsx:28-40`（`/analytics` 在 `:35`） | ✅ | 列表项 |
| 页面副标题文案 | `frontend/src/lib/i18n/zh-CN.ts:460`、`en.ts:447`、`ja.ts:448` | 🟡 | 副标题列出“成本趋势”“排行榜”，页面未渲染 |
| 入口链接 | `BranchWorkspace.tsx:180-190`；`kanban/page.tsx:500`；`worktree/[id]/group/page.tsx:2793` | ✅ | 三处链接指向 `/analytics` |
| 报告类型与分派 | `crates/domain-report/src/lib.rs:34-165, 485-531` | 🟡 | 22 类枚举，分派结构见 §4.4 |
| 报告 P0 模块 | `crates/domain-report/src/domain/c01…c07_*.rs`、`c13_created_vs_resolved.rs` | 🟡 | 计算存在；经内存端口；无调用方 |
| 报告 P1 模块 | `domain/c08…c12_*.rs`、`c14_*.rs` | 🟡 | 存在；经 `generate` 不可达 |
| 报告 P2 模块 | `domain/c15_priority_dist.rs`（不可达）、`c16…c22_*.rs` | ⬜ | c16–c22 返回 not_implemented |
| 报告端口与缓存 | `application/ports.rs`、`application/cache.rs`、`infrastructure/port_stubs.rs`、`infrastructure/in_memory_cache.rs` | 🟡 | 端口 trait 完整；实现为内存桩 |
| 报告导出 | `crates/domain-report/src/lib.rs:679-684` | 🟡 | 仅 JSON 与 CSV，进程内 |
| 仪表盘引擎 | `crates/domain-dashboard/src/lib.rs` | 🟡 | 内存模型；7 个单元测试 `:301-383` |
| Run 级 BI 端点 | `crates/star-api-rest/src/group_api/bi_analytics.rs`（路由 `:61-66`，处理 `:112-153`） | 🟡 | 需 `BI_DATABASE_URL` 才注入端口 `main.rs:115, 130-132` |
| Run BI 客户端与面板 | `frontend/src/lib/run/runDirectoryApi.ts:693`；`frontend/src/components/run/RunBiPanel.tsx` | 🟡 | `RunBiPanel` 无非测试文件导入；BranchWorkspace 的 BI 页签改为指向 `/analytics` |
| 报告与仪表盘 REST、仪表盘页面 | 无 | ⬜ | 无 `/api/reports`、`/api/dashboards` 路由；`(app)/` 下无 dashboards 目录 |
| 已退役：页面内下拉式图表选择器 | `page.tsx`（旧版） | 🗑 | 由标签页替代，提交 `a6ce3bda`（说明“替代下拉菜单”） |
| 已退役：页面内联 `MOCK_KPI`、`COST_SERIES` | `page.tsx`（旧版） | 🗑 | 页面不再引用，数据迁至 `mocks/data/analytics.ts`；`git log -S` 定位提交 `23d60e75` |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| 页面数据非授权投影（store 派生加字面量，`page.tsx:45-52, 237-242, 270-275`；`chartData.ts`） | 指标可能被误读为真实数据 | 定义分析读模型端口，由授权 API 提供数据；未接入时显示“未接入”（FR-16-03） |
| KPI 语义不符：“验证通过率”为完成率（`page.tsx:69`），“Agent 运行时长”为 token 均值（`page.tsx:70-71`）；趋势为常量（`page.tsx:75-78`） | 指标名称与含义不一致 | 按定义重算或改名；趋势改为真实比较，或移除 |
| 运维与审计为字面量且无未接入标注（`page.tsx:237-242, 270-275`） | 展示无来源数值 | 替换为数据源；无来源时显示未接入态 |
| 运维标签缺集群与发布卡；注释称对接 MetricsTab、LogAITab 但未导入（`page.tsx:234`） | 注释与实现不一致；原型未实现 | 按原型补三卡，或删除误导注释 |
| 范围与时间选择器未绑定（`page.tsx:118-127`） | 用户以为可筛选 | 绑定到 state 并传入 `buildAnalyticsData`；或暂时隐藏 |
| 导出与订阅无回调（`page.tsx:91-105`） | 按钮不可达 | 后端完成后接入，之前保持隐藏 |
| 标签状态不进 URL（`page.tsx:56`） | 无法深链；看板与 BranchWorkspace 只能进入默认页 | 增加 tab 查询参数同步 |
| 分组声明的 tab id（`registry.ts:781`）与页面标签不对应；`sandbox` 无渲染 | 导航与页面对不上 | 对齐分组声明与页面标签，或由分组引用独立路由 |
| 副标题提及“成本趋势”“排行榜”（`zh-CN.ts:460` 等），页面未实现这两块 | 对外文案超出实现 | 修改文案，或实现对应模块 |
| 甘特图 chartId 为 `C16_GANTT`（`page.tsx:216`），与 domain-report 的 C16 AssigneeWorkload 编号冲突 | 对接时易混淆 | 甘特图改用非图表编号 |
| P1 图表经 `generate_stub` 旁路，返回 `stub: true` 数据（`lib.rs:523-524, 648`）；C15 不属于任何批次 | 调用方可能得到伪数据 | 修正分派，使 P1 与 C15 走真实模块；补批次定义（§9 待决策） |
| 无 HTTP 路由，无调用方（domain-report、domain-dashboard） | 引擎不可用 | 按 basic-design §5.1 实现 `/api/reports`、`/api/dashboards`，并接入前端 |
| `PermissionPort` 注入但未调用（`lib.rs:463-480, 485-531`） | 报告无权限校验 | `generate` 前调用权限端口，拒绝越权请求 |
| 端口仅内存桩（`port_stubs.rs`） | 无真实数据源 | 接入 domain-work-item、domain-planning、domain-identity、domain-permission 的查询实现 |
| 缓存为进程内存，无 Redis | 多实例不共享缓存 | 接入共享缓存，或在文档中限定为单进程 |
| `DataTooLarge` 无构造点（`lib.rs:379`） | 大数据量无保护 | `generate` 中计数，超限拒绝或采样（INV-REPORT-09、10） |
| 仪表盘无布局校验；`InvalidPosition` 无构造点（`lib.rs:215`）；上限 20 与设计的 12 不同（`lib.rs:175`） | Gadget 可能重叠或越界 | 实现校验；上限按 §9 裁决 |
| 仪表盘 Gadget 类型集合与设计不同（`lib.rs:22-40` 对 spec §5） | 后续对接需返工 | 按 §9 裁决统一 |
| 仪表盘无持久化、共享、Wallboard 只读与过期 | 不能用于生产 | 落库（W/T/M 分类，spec §9）；实现 ShareGrant 与 link_token |
| `RunBiPanel` 未挂载；BI 端点需 `BI_DATABASE_URL` | 功能对用户不可见，或返回 `feature_unavailable` | 决定是否挂载到 BranchWorkspace 的 BI 页签；在部署说明中注明环境变量 |
| MSW analytics mock 无调用方 | 维护负担 | 移除，或让页面读取同一数据源 |
| 测试只覆盖页面渲染 KPI 与 IPA 区块（`panels.test.tsx:110-121`）；无标签切换、选择器、审计表测试；无 e2e 覆盖 `/analytics` | 行为回归无法被发现 | 补标签切换与空数据态测试；补 e2e |

## 7. 验收与测试

| 测试 | 位置 | 验证内容 | 未验证内容 |
|---|---|---|---|
| 页面渲染 | `frontend/src/app/(app)/__tests__/panels.test.tsx:110-121` | 4 个 KPI 测试标识、IPA 区块、阶段名 | 图表数值、标签切换、选择器 |
| MSW mock 数据 | `frontend/src/mocks/__tests__/analytics.test.ts:8-47` | `MOCK_KPI` 4 项、`COST_SERIES` 7 天、类型守卫、非负 | 页面是否消费（无消费者） |
| MSW handler | `frontend/src/mocks/__tests__/handlers.test.ts:29-45` | 两个 mock 的形状与数量 | 同上 |
| MSW 快照与夹具 | `mocks/__tests__/snapshot.test.ts:15-19`；`fixtures-sync.test.ts:24-28` | 数据与快照、夹具一致 | 页面行为 |
| Run BI 面板 | `frontend/src/components/run/RunBiPanel.test.tsx:28, 36, 44, 57` | 4 个用例：不可用态、空结果、中止旧请求、游标翻页与卸载中止 | 面板在页面中的挂载（未挂载） |
| 报告 C01 集成 | `crates/domain-report/tests/c01_burndown_test.rs` | 6 个 `#[tokio::test]` | 其余 21 类图表 |
| 报告单元 | `domain/c01…c22_*.rs` 与 `lib.rs` 测试模块 | 各模块计算与 NotImplemented 结构；c01 6 个、c02 5、c03 5、c04 4、c05 3、c06 5、c07 4、c08 2、c09 3、c10 2、c11、c12、c14–c22 各 1、c13 3；缓存 3 个 | P1 模块经 `generate` 的可达性（旁路）；导出 |
| 仪表盘单元 | `crates/domain-dashboard/src/lib.rs:301-383` | 7 个：Gadget 类型数量与默认尺寸、个人与全局创建、添加与删除、Wallboard 标志 | 布局、持久化、共享 |
| Run BI 端点单元 | `crates/star-api-rest/src/group_api/bi_analytics.rs:184-220` | 游标校验与分页上限 | 鉴权路径与 DB 端口（需集成环境） |

未执行 vitest、cargo test 与 Playwright。以上为测试存在性与断言内容的核对，通过状态为 ❓。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-ANALYTICS-001.md` | §1 设计要点；FR-AN-001 至 004；§3 文件映射 | FR-AN-001 与 FR-AN-004 标为已实现，与代码语义不符；FR-AN-003 标为不实现，与代码一致；§4 修订履历所列 commit 号未核对；§1 的运维区含“集群、发布”，页面未实现 | 需求文本采纳；状态以代码为准（FR-16-01、04 为 🟡，FR-16-03 为 ⬜） |
| `docs/specs/domain-dashboard-spec.md`（v1.0，2026-09-02） | 实体、INV-DASH-01 至 15、Port 设计、10 类 Gadget、DDL（W/T/M）、12 列算法 | 代码 GadgetType 为 IssueStats 等十类，与设计集合不同；代码上限 20（`lib.rs:175`）对设计 12（INV-DASH-04）；代码 Dashboard 字段为 name、scope、wallboard_mode，设计为 title、layout、shared_with、auto_refresh_seconds；代码头部标注“精简实装 v0.1” | 目标设计采用 v1.0（较新，显式扩写）；现状以代码为准，差异列为 🟡；Gadget 集合的最终取舍列为待决策 |
| `docs/specs/domain-report-spec.md`（v1.0，2026-09-02） | 22 类图表、5 类范围、INV-REPORT-01 至 20、Port、错误模型、分批 P0 8 / P1 7 / P2 7（§8）、范围矩阵（§7）、Redis 缓存、导出格式 | spec 的 P1 含 C15，代码 `p1_batch` 不含（`lib.rs:151-160`），且 C15 不在任何批次；spec 的 P2 为 C16–C22，代码 `p2_batch` 同（`lib.rs:133-144`）；`lib.rs` 文件头注释称 P2 为 8 类含 C15；代码无 ReportScope 枚举；无 Redis；导出仅 JSON 与 CSV | 22 类与分批采用 spec 为目标设计；C15 的归属以代码当前行为为状态依据；头注释与实际批次不一致列为差距；C15 归属列为待决策 |
| `docs/prototypes/5 · 分析-html/README.md` | 原型目录说明，指向 `Analytics.dc.html` | 无行为规格 | 仅作参考 |
| `docs/prototypes/5 · 分析-html/Analytics.dc.html` | 3 个标签（概览、运维、审计日志）；4 个指标卡；6 个 IPA 阶段条；运维三卡（集群、指标、发布）及“未接入时显示「未接入」”；审计 4 列（时间、操作者、事件、范围）；范围与时间为按钮 | 页面有 19 个标签（`page.tsx:38-42`）；审计 5 列且行内容不同（`page.tsx:261-275`）；运维为 4 个字面量（`page.tsx:237-242`）；范围为 select（`page.tsx:118-127`） | 运维三卡与“未接入”语义采纳为设计意图（FR-16-03、FR-16-10）；标签结构以页面为准（`page.tsx:9-10` 的 2026-11-15 标签页决策） |
| `docs/prototypes/5 · 分析-html/support.js`、`vendor/react.js`、`vendor/react-dom.js` | 未读取；README 声明为渲染运行时 | — | 不作为设计依据 |
| `docs/features/F16-analytics.md`（同名文件的前一版，基线 cdd00a78） | 章节结构；部分证据（运维注释与 MetricsTab 未导入、ChartFrame 回调未传入、域 crate 无调用方） | 基线不同；部分行号与状态与当前代码不一致 | 结构沿用；证据与状态以当前代码为准 |
| `docs/frontend/design/ui-detailed-design.md` 第 171–174 行 | `/analytics` 真实数据接入：现状 Mock 驱动，阶段 Phase D.6+，缺口“后端 KPI/Cost API 待接入真实 Prometheus/ClickHouse 流” | 其余章节未读取 | 第 173 行的缺口描述作为 FR-16-33 依据 |
| `docs/basic-design.md`、`docs/basic-design/charts-and-reports.md`、`docs/requirements/charts-and-reports.md` | 存在性核对；`charts-and-reports.md` §5.1 作为 REST 设计参照，经 spec 交叉引用 | 内容未逐条读取 | 作为设计参照；状态依据不取自此处 |
| `docs/design/charts/`（22 份） | 存在性核对（22 个文件） | 内容未读取 | 未采用；列为待核对 |
| `docs/features/F15-inbox-notification-im.md` | 文档结构参照（前 60 行） | — | 仅格式参照，不作为事实来源 |
| `frontend/src/app/(app)/analytics/page.tsx` | 全文读取 | — | 现状依据 |
| `frontend/src/app/(app)/analytics/chartData.ts` | 全文读取 | — | 现状依据 |
| `frontend/src/mocks/handlers/analytics.ts`、`data/analytics.ts`、`schemas/analytics.ts`、`__tests__/analytics.test.ts` | 全文读取 | — | 现状依据 |
| `frontend/src/mocks/__tests__/handlers.test.ts`、`fixtures-sync.test.ts`、`snapshot.test.ts` | 仅 grep 命中行 | — | 测试存在性依据 |
| `crates/domain-dashboard/src/lib.rs` | 第 1–300 行全文；测试函数名（grep） | 测试体（301–391）未全文读取 | 现状依据 |
| `crates/domain-report/src/lib.rs`、`domain/*.rs`、`application/*.rs`、`infrastructure/port_stubs.rs` | 枚举、批次、分派、stub、端口、缓存键、导出（分段读取与 grep） | 各 `c*.rs` 计算细节未逐一读取 | 现状依据 |
| `crates/star-api-rest/src/group_api/bi_analytics.rs` | 全文读取 | — | 现状依据 |
| `crates/star-api-rest/src/group_api.rs`、`src/main.rs` | 路由合并（`group_api.rs:804`）；端口注入（`main.rs:112-133`）；`with_bi_task_event_query_port`（`group_api.rs:458-465`） | — | 现状依据 |
| `frontend/src/lib/nav/registry.ts` | ALL_MODULES 与 MODULE_GROUPS 相关条目（`:219-230`、`:778-782`）；grep 行 `:491-597`；与基线的差异仅 projects 模块描述 | — | 现状依据 |
| `frontend/src/app/(app)/layout.tsx`；`frontend/src/lib/redirects.ts` | 全高前缀（`layout.tsx:28-40`）；`NEW_PANEL_ROUTES` 含 `/analytics`（`redirects.ts:169`，注释称信息性） | — | 布局依据；redirects 不作为状态依据 |
| `frontend/src/components/run/BranchWorkspace.tsx`；`RunBiPanel.tsx`；`RunBiPanel.test.tsx`；`lib/run/runDirectoryApi.ts` | 入口与面板现状（`BranchWorkspace.tsx:178-192`；`RunBiPanel.tsx:26`；测试名 grep；`runDirectoryApi.ts:693` grep） | — | 现状依据 |
| `frontend/src/app/(app)/__tests__/panels.test.tsx` | 页面渲染用例（`:100-125`） | — | 测试依据 |
| `frontend/src/components/charts/shared/ChartFrame.tsx` | 回调 props 与按钮渲染（`:21-52`） | — | 现状依据 |
| `frontend/src/lib/i18n/zh-CN.ts`、`en.ts`、`ja.ts` | 页面副标题（grep） | — | 文案现状依据 |
| `frontend/src/components/charts/Chart01–15.tsx`；`components/gantt/GanttView.tsx`；`lib/chart-data-schema.ts` | 存在性与 recharts import（grep）；组件内部未读取 | — | 存在性依据 |
| `frontend/src/lib/store.ts` | `identities`（`:147`）、`agentSessions`（`:156`）（grep） | — | 现状依据 |
| 仅 grep 命中的其它文件 | `kanban/page.tsx:500`、`worktree/[id]/group/page.tsx:2793`（入口）；`subNavRegistry.ts`（无 analytics 条目）；`frontend/e2e`（无 analytics 用例）；`mocks/handlers/index.ts`、`mocks/data/index.ts`（注册）；Sidebar 相关测试（`sidebar-item-analytics`） | — | 存在性与入口依据 |
| `crates/star-dispatcher/src/sa_real_impls.rs` | crate 名字符串列表（`:250, 263`） | 仅字符串，不是依赖 | 未采用为依赖证据 |
| `crates/bi-owner-runtime`、`crates/bi-native-event-adapter` | 仅目录与文件规模，未读取 | — | 未采用（独立 BI 功能，范围外） |
| git 历史：`4755212b`（19 标签，甘特图并入）、`a6ce3bda`（标签页替代下拉，15 图表合并）、`23d60e75`（页面按原型改造） | 🗑 证据；基线核对 | — | 历史证据，不作为现状依据 |

## 9. 待决策项

1. 仪表盘 Gadget 类型集合以 spec §5 还是 `domain-dashboard` 代码为准；Gadget 上限取 12 还是 20。
2. C15 属于 P1 还是 P2 批次；P2 为 7 类还是 8 类（代码头注释与 spec 不一致）。
3. 分析页保留 19 标签结构，还是回归原型的 3 标签结构；标签是否需要 URL 深链。
4. 分组 `analytics` 声明的 `sandbox` tab 是否保留。
5. `RunBiPanel` 是否挂载到 BranchWorkspace 的 BI 页签（当前页签仅指向 `/analytics`）。
6. `(app)` 分组的鉴权逻辑未读取，§4.5 只描述页面自身。
7. 15 个图表组件内部渲染与数据绑定未逐一核对；`docs/design/charts` 22 份与 basic-design、requirements 的 charts 文档未逐条比对。
8. SRS 修订履历中的 commit 号未逐一核对。
