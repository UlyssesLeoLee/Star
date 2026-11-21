# F16 分析

> 状态摘要: `/analytics` 页面以标签页呈现 C01–C15 图表、4 项 KPI、IPA 阶段进度、运维指标与操作日志。图表与 KPI 数据来自前端 store 聚合并带字面量回退；运维指标、操作日志与 IPA 进度为静态内容。C16–C22 前端未接入；报表引擎与仪表盘引擎均未接入任何 REST 路由或页面。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 提供单一分析入口 `/analytics`，以标签页切换效能指标、图表、运维指标与操作日志（SRS-ANALYTICS-001 §1；docs/prototypes/5 · 分析-html/Analytics.dc.html）。
- 图表覆盖对标 Jira 报表中心的 22 类图表与 5 类数据范围（docs/requirements/charts-and-reports.md §1–§2）。
- 数据必须来自授权事实。宿主 BI 或数据库投影接入前，未接入的数据以「未接入」诚实标注，不得以静态或回退数字冒充事实（SRS-ANALYTICS-001 §0、§2 FR-AN-003；frontend/src/lib/store.ts:394 的空数据原则）。
- 报表引擎 `domain-report` 与仪表盘引擎 `domain-dashboard` 提供图表计算、缓存、导出、订阅与仪表盘聚合（docs/specs/domain-report-spec.md §1；docs/specs/domain-dashboard-spec.md §1）。

### 1.2 范围内 / 范围外

范围内:

- `/analytics` 页面的标签页、KPI、IPA 进度、范围控件、运维区、审计日志区与图表标签页。
- 图表前端组件 `frontend/src/components/charts/`，以及页面使用的数据聚合 `chartData.ts`。
- crates/domain-report 与 crates/domain-dashboard 的实体、端口、计算与缓存。
- `/analytics` 的导航入口。

范围外:

- 甘特图组件细节（`frontend/src/components/gantt/GanttView.tsx`，本页仅作为标签页嵌入）。
- `/ops` 运维控制台（frontend/src/app/ops/page.tsx）与 `/audit` 审计页（frontend/src/app/audit/page.tsx），二者为独立路由。
- Lane BI 面板 `RunBiPanel` 与 `bi/task-events` 接口的归属，见 §4.6 与 §6。
- Canvas、插件、Sandbox 等同组模块。

## 2. 用户与场景

| 角色 | 场景 | 现状 |
|---|---|---|
| 项目负责人 | 在燃尽、燃起、速度、Sprint 报告之间切换，查看进度 | 图表可切换渲染；数据为 store 聚合（FR-16-09、FR-16-10） |
| 项目负责人 | 按项目与时间范围过滤 | 控件可见但不生效（FR-16-05） |
| 项目负责人 | 导出图表 CSV/PNG，订阅定期报告 | 页面不提供（FR-16-16、FR-16-17） |
| 开发者 | 查看 Assignee Workload、Component Workload、Recently Created | C16–C22 前端未接入（FR-16-12） |
| 工程负责人 | 查看 IPA 阶段进度与 IPA 技能覆盖 | 进度为常量，覆盖率由 store 计算（FR-16-01、FR-16-02） |
| 运维人员 | 查看运维指标与操作日志 | 指标与日志为静态内容，无「未接入」标注（FR-16-03、FR-16-04、FR-16-24） |
| 仪表盘使用者 | 组合 Gadget 并共享、以 Wallboard 全屏展示 | 仅有实体与少量方法，无页面与接口（FR-16-26 至 FR-16-31） |

## 3. 需求

优先级依据: 图表批次采用 docs/requirements/charts-and-reports.md 与 crates/domain-report/src/lib.rs:3–6 的注释（P0 为 C01–C07、C13；P1 为 C08–C12、C14；P2 为 C15–C22）。SRS-ANALYTICS-001 未标注优先级，FR-AN 项记为 P1（推定）。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-16-01 | 4 指标卡呈现完成任务、验证通过率、Agent 运行时长、IPA 技能覆盖，且指标口径与标签一致（SRS FR-AN-001） | P1（推定） | 🟡 | 卡片渲染 frontend/src/app/(app)/analytics/page.tsx:155–168；「验证通过率」实为 done/total（page.tsx:69、76）；「Agent 运行时长」实为平均 token 数（page.tsx:71、77）；趋势值 `+12%`/`+5%`/`-3%`/`+8%` 为字面量（page.tsx:75–78） |
| FR-16-02 | IPA 阶段进度条呈现 P2 至 P6.2 的完成比例（SRS FR-AN-002） | P1（推定） | 🟡 | 渲染 page.tsx:170；数据为常量 `IPA_PHASES`（page.tsx:45–52）；分支名字面量（page.tsx:173） |
| FR-16-03 | 运维区对未接入的指标显示「未接入」标识（SRS FR-AN-003） | P1（推定） | ⬜ | 运维 tab 为字面量指标卡（page.tsx:237–242），无「未接入」文案；页面内无「未接入」字样 |
| FR-16-04 | 操作日志表呈现时间、操作者、事件、范围（SRS FR-AN-004；mockup 列结构见 Analytics.dc.html:87） | P1（推定） | 🟡 | 表格渲染 page.tsx:259–290；列为 时间/用户/操作/资源/结果（page.tsx:259–268 区段）；数据为 4 行字面量（page.tsx:271–274）；不读取 store.auditEvents |
| FR-16-05 | 项目与时间范围选择影响页面数据（SRS §1「范围选择 (项目) + 近 14 天」） | P1（推定） | ⬜ | 两个 `<select>` 渲染于 page.tsx:118–127，无 `value`、无 `onChange`；选项为字面量（page.tsx:119–121、124–126）；范围卡字面量 page.tsx:195–200 |
| FR-16-06 | 集群区块呈现 K3s 集群状态，未接入时显示「未接入」（mockup） | P2 | ⬜ | Analytics.dc.html:79；页面无集群区块 |
| FR-16-07 | 指标区块呈现 P95 延迟、错误率、队列深度（mockup） | P2 | 🟡 | 运维 tab 字面量 4 项：API 可用性 99.97%、P95 延迟 142ms、错误率 0.03%、事件积压 12（page.tsx:237–242）；无队列深度；Analytics.dc.html:80 |
| FR-16-08 | 发布区块呈现金丝雀进度与回滚记录（mockup） | P2 | ⬜ | Analytics.dc.html:81；页面无发布区块 |
| FR-16-09 | 图表标签页渲染 C01–C15 十五类图表，按标签切换 | P0（C01–C07、C13）/ P1（C08–C12、C14）/ P2（C15） | ✅ | 标签与切换 page.tsx:38–42、140–141；渲染 switch page.tsx:89–107；组件 frontend/src/components/charts/Chart01Burndown.tsx 至 Chart15PriorityDist.tsx（15 个文件） |
| FR-16-10 | 图表数据来自授权事实（经报表引擎或授权投影），非前端字面量 | P0 | 🟡 | 数据由 frontend/src/app/(app)/analytics/chartData.ts 从 store 聚合（chartData.ts:48–50 起）；字面量回退见 chartData.ts:52、91、132–133、160、175–178、211–217；报表引擎未被页面调用（见 FR-16-13） |
| FR-16-11 | 数据为空时显示空态，不以合成数据冒充事实（charts-and-reports §3.1 验收「0 SP sprint 显示无数据提示」；store.ts:394） | P0 | 🟡 | 「预览数据」横幅 page.tsx:82–87 仅为通用提示；store 初始为空数组（store.ts:397–436）；空 store 时 `sprintOf` 生成 `spr-preview`（chartData.ts:31–34）；回退数字使图表显示合成值（chartData.ts:52、91、211、216）；KPI 在空数据时显示 `0 / 1`（page.tsx:68、75） |
| FR-16-12 | C16–C22 图表在分析页呈现 | P2 | ⬜ | 标签 switch 止于 C15（page.tsx:105–106）；components/charts 目录仅含 Chart01–Chart15 与 shared/ChartFrame.tsx（git ls-tree 于 frontend/src/components/charts） |
| FR-16-13 | C16–C22 报表计算可执行 | P2 | 🟡 | `generate` 入口存在但返回 `not_implemented`：c16_assignee_workload.rs:16–21；c17_component_workload.rs:12–17；c18–c22 同构（c22_recently_created.rs:12–17） |
| FR-16-14 | C01 Burndown 与 C02 Burnup 按 Sprint 与 WorkItem 端口计算理想线与实际线 | P0 | 🟡 | C01：crates/domain-report/src/domain/c01_burndown.rs:90 起，调用 `work_item_port`；C02：c02_burnup.rs:56–74 调用 `work_item_port` 与 `sprint_port`；端口实现为内存桩 port_stubs.rs:26、76；无路由调用方 |
| FR-16-15 | C03–C15 报表计算读取真实数据 | P1 / P2 | 🟡 | 以下模块返回 mock 数据：c03_velocity.rs:71、c04_sprint_report.rs:79、c05_cfd.rs:48、c06_control_chart.rs:74、c07_cycle_time.rs:65、c08_throughput.rs:52（mock 14 周数据）、c09_forecast.rs:147、c13_created_vs_resolved.rs:52；以下模块参数 `_work_item_port` 未使用：c10_time_tracking.rs:50、c11_resolution_time.rs:33、c12_sla_compliance.rs:53、c14_issue_type_dist.rs:33、c15_priority_dist.rs:33 |
| FR-16-16 | 图表导出 CSV、XLSX、PNG、PDF（charts-and-reports §5；domain-report-spec §4.4） | P1 | 🟡 | 引擎层 `export_json`、`export_csv` 存在（crates/domain-report/src/lib.rs:679–684）；无 XLSX、PNG、PDF；无路由；页面 ChartFrame 仅在传入 `onExport` 时显示导出按钮（ChartFrame.tsx:21，page.tsx:91–105 未传入） |
| FR-16-17 | 订阅定期报告与降噪（charts-and-reports §5；domain-report-spec §4.3） | P1 | ⬜ | domain-report/src 中无订阅实体与端口；页面不传 `onSubscribe`（ChartFrame.tsx:22） |
| FR-16-18 | ReportDefinition 聚合：创建、更新、删除、启停、调度（domain-report-spec §2、§4.1） | P1 | ⬜ | domain-report/src 中无 `ReportDefinition`；ReportService 仅持有缓存与端口（lib.rs:456–465） |
| FR-16-19 | 报表与仪表盘 REST 端点 `/api/reports/*`、`/api/dashboards/*`（basic-design 接口表） | P1 | ⬜ | 设计定义于 docs/basic-design/charts-and-reports.md:661–677；star-api-rest 路由仅挂载 bi_analytics（crates/star-api-rest/src/group_api.rs:814） |
| FR-16-20 | Scope S1–S5 与 JQL 风格过滤（charts-and-reports §1、§4） | P1 | ⬜ | `ReportFilter` 存在（crates/domain-report/src/lib.rs:226）；domain-report/src 与 analytics 目录均无 JQL 解析；页面 scope 控件不生效（FR-16-05） |
| FR-16-21 | 数据点上限：超过 10K 采样提示，超过 100K 拒绝（INV-REPORT-09、INV-REPORT-10） | P1 | ⬜ | `DataTooLarge` 错误变体存在（lib.rs:379、429、444），但 domain-report/src 中无阈值判断 |
| FR-16-22 | 报表结果缓存 5 分钟 TTL（INV-REPORT-02） | P1 | 🟡 | `generate` 读取缓存（lib.rs:504）并写入 300 秒（lib.rs:530）；缓存为内存实现（infrastructure/in_memory_cache.rs:53–56），cache.rs:9 注明 Redis 留待 V2 |
| FR-16-23 | 分析页数据来自授权投影（宿主 BI 或数据库）（SRS §0「数据源: store (seed 预览模式) — 宿主接入后切换授权投影」） | P0 | ⬜ | 页面仅读取 store（page.tsx:58–62）；无 BI 投影端口或接口 |
| FR-16-24 | 操作日志数据来自授权审计或事件源（SRS FR-AN-004） | P1 | ⬜ | 页面日志为字面量（page.tsx:271–274）；`/audit` 页读取 `store.auditEvents`（frontend/src/app/audit/page.tsx:12）；真实事件接口 `bi/task-events` 存在（bi_analytics.rs:61–63）但页面不调用 |
| FR-16-25 | 甘特图标签页呈现冲刺与里程碑排期 | P1 | 🟡 | 渲染 page.tsx:214–222，嵌入 `GanttView`（page.tsx:219）；ChartFrame 的 `chartId` 为 `C16_GANTT`（page.tsx:216），与报表 C16 Assignee Workload 编号冲突（domain-report lib.rs:69、185） |
| FR-16-26 | 仪表盘实体：Dashboard 聚合、Gadget 位置与尺寸、12 列布局（dashboard-spec §2、§3 INV-DASH-01） | P2 | 🟡 | 实体存在：`Gadget`（domain-dashboard/src/lib.rs:103）、`Dashboard`（lib.rs:129）、`Dashboard::new`（lib.rs:152）；位置与尺寸类型存在（lib.rs:76、120）；无 12 列校验 |
| FR-16-27 | Gadget 类型 10 种：chart-*、text、activity、assigned-to-me、filter-results、wallboard-clock、wallboard-sla、heatmap、two-dimensional-filter、rich-text（dashboard-spec §5） | P2 | 🟡 | `GadgetType` 含 10 个变体（lib.rs:22 起），名称为 IssueStats、Burndown、Velocity、MyWork、RecentActivity、DueSoon、JqlTable、Markdown、Iframe、Clock，与 spec 不对应 |
| FR-16-28 | Gadget 上限 12 且不重叠（INV-DASH-02、INV-DASH-04） | P2 | 🟡 | 上限为 20（lib.rs:174–175，`TooManyGadgets(20)`）；无重叠校验（domain-dashboard/src 中无相关逻辑） |
| FR-16-29 | 仪表盘创建按 scope 校验权限，Global 范围需管理员（dashboard-spec INV 与 §4） | P2 | 🟡 | `DashboardService::create` 在 lib.rs:242–261 对 `Global` 检查 `is_admin`；`DashboardService` 为无状态单元结构（lib.rs:233），无持久化 |
| FR-16-30 | Wallboard 只读全屏模式，公开链接 30 天过期（INV-DASH-03、INV-DASH-06） | P2 | 🟡 | `enable_wallboard` 仅置位 `wallboard_mode`（lib.rs:194–197）；无链接令牌、无过期时间 |
| FR-16-31 | 仪表盘共享 ShareGrant，仪表盘页面 `/dashboards` | P2 | ⬜ | 无 ShareGrant 类型（domain-dashboard/src/lib.rs 全文）；frontend/src/app 下无 dashboards 目录 |
| FR-16-32 | `/analytics` 路由存在，并可从导航进入 | P0 | ✅ | 页面存在；全高度前缀 frontend/src/app/(app)/layout.tsx:35，AppShell 渲染 :49；核心模块定义 frontend/src/lib/nav/registry.ts:219–230（isCore 位于 :229）；侧栏渲染 Sidebar.tsx:227；测试断言 Sidebar.test.tsx:88；工作区链接 frontend/src/app/(app)/kanban/page.tsx:500、frontend/src/app/worktree/[id]/group/page.tsx:2793、frontend/src/components/run/BranchWorkspace.tsx:186 |
| FR-16-33 | 页面加载、错误、未授权状态（SRS 状态约定；ChartFrame 支持 `isLoading`、`error`） | P0 | ⬜ | ChartFrame 属性存在（ChartFrame.tsx:19–20），页面不传入（page.tsx:91–107）；页面无数据请求，无未授权分支 |

## 4. 设计

### 4.1 概念模型

| 概念 | 设计来源 | 代码对应 | 状态 |
|---|---|---|---|
| KPI 卡 | SRS §2 FR-AN-001 | page.tsx:66–80 | 🟡 |
| IPA 阶段 | SRS §1 | page.tsx:45–52 | 🟡 |
| 图表（C01–C22，含 scope） | charts-and-reports §2–§3 | 前端 C01–C15；引擎 ReportType（lib.rs，22 变体，impl 起于 :84） | 🟡 |
| 报表结果 | domain-report-spec §2 ReportSnapshot | ReportResult（lib.rs:303）、ReportPoint（lib.rs:292）、ReportSummary（lib.rs:322） | ✅ |
| 报表定义 ReportDefinition | domain-report-spec §2 | 无 | ⬜ |
| 报表快照（持久） | domain-report-spec §2 ReportSnapshot | 仅内存缓存条目（infrastructure/in_memory_cache.rs） | 🟡 |
| 调度 ReportSchedule、订阅 ReportSubscription | domain-report-spec §2 | 无 | ⬜ |
| 仪表盘 Dashboard | domain-dashboard-spec §2 | domain-dashboard/src/lib.rs:129 | 🟡 |
| Gadget | domain-dashboard-spec §2、§5 | domain-dashboard/src/lib.rs:103 | 🟡 |
| 运维指标 | SRS §1、mockup | page.tsx:237–242（字面量） | 🟡 |
| 操作日志行 | SRS FR-AN-004 | page.tsx:271–274（字面量） | 🟡 |

### 4.2 数据模型

- 前端页面读取的 store 字段：`identities`（store.ts:147）、`workItems`（store.ts:149）、`agentSessions`（store.ts:156）、`sprints`（store.ts:173）、`board`（store.ts:176）；`auditEvents`（store.ts:436）未被本页读取。
- 页面使用的 WorkItem 字段：`status`、`ipa_skill_ids`、`story_points`、`kind`、`priority`、`key`、`title`、`updated_at`、`id`（chartData.ts:48–50 与 page.tsx:67–73）。
- 页面使用的 AgentSession 字段：`token_usage.total`（page.tsx:70）。
- 报表引擎：`ReportFilter`（domain-report/src/lib.rs:226）、`ReportResult`（lib.rs:303）、`ReportPoint`（lib.rs:292）、`ReportSummary`（lib.rs:322）、`ReportError`（lib.rs:379 起）。
- 仪表盘：`DashboardScope`、`Gadget`（lib.rs:103）、`GadgetPosition`（lib.rs:120）、`GadgetSize`（lib.rs:76）、`Dashboard`（lib.rs:129）。
- 本功能未在代码中发现专属数据表。migrations 目录未核对，❓（需检索 migrations 或 schema 文件确认报表与仪表盘表的存在性）。

### 4.3 接口 routes、APIs、ports

| 接口 | 类型 | 位置 | 状态 |
|---|---|---|---|
| `GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-events` | REST | crates/star-api-rest/src/group_api/bi_analytics.rs:61–63；处理函数 :112；授权 :135；6 秒超时 :143；挂载 group_api.rs:814 | ✅（调用方为 RunBiPanel，见 §4.6） |
| `/api/reports/*` | REST | 设计：docs/basic-design/charts-and-reports.md:661–677 | ⬜ |
| `/api/dashboards/*` | REST | 设计：docs/basic-design/charts-and-reports.md:672–677 | ⬜ |
| `/api/analytics/kpi`、`/api/analytics/cost` | MSW mock | frontend/src/mocks/handlers/analytics.ts:16、19 | ⬜（无真实路由，页面不调用） |
| `WorkItemQueryPort`、`SprintQueryPort`、`UserQueryPort`、`PermissionPort` | Rust port | crates/domain-report/src/application/ports.rs；实现为内存桩 port_stubs.rs:26、76、114、131 | 🟡 |
| `ReportService::generate` | Rust 方法 | crates/domain-report/src/lib.rs:485；Burndown 分支 :544；AssigneeWorkload 分支 :618 | 🟡（无外部调用方） |
| `ReportService::export_json`、`export_csv` | Rust 方法 | crates/domain-report/src/lib.rs:679、684 | 🟡 |
| 报表 Command、Query、Schedule、Export 端口 | domain-report-spec §4 | 无对应 trait | ⬜ |
| `DashboardService::create` | Rust 方法 | crates/domain-dashboard/src/lib.rs:242 | 🟡 |
| `Dashboard::add_gadget`、`remove_gadget`、`enable_wallboard` | Rust 方法 | lib.rs:173、183、194 | 🟡 |
| 仪表盘 Command、Query、Subscription 端口 | domain-dashboard-spec §4 | 无对应 trait | ⬜ |

### 4.4 状态机与关键流程

- 标签状态：`activeTab` 为页面本地状态，初始值「概览」（page.tsx:56）；点击标签调用 `setActiveTab`（page.tsx:140–141）后按 `activeTab` 渲染对应区块（page.tsx:153、214、225、231、253）。
- 图表数据流：store 变化 → `useMemo` 调用 `buildAnalyticsData(sprints, workItems)`（page.tsx:64）→ 各图表组件。该流程无轮询与实时推送，charts-and-reports §0.1 所述 30 秒轮询未实现。
- 报表计算流（设计）：查询 → 缓存检查（lib.rs:504）→ 端口取数 → 计算 → 缓存写入（lib.rs:530）。当前仅 C01 与 C02 读取端口；其余模块返回 mock 或 `not_implemented`（见 FR-16-13、FR-16-15）。
- 仪表盘生命周期（设计）：创建 → 添加或移除 Gadget → 开启 Wallboard。当前为内存实体，无持久化与状态迁移校验（lib.rs:150–199）。

### 4.5 权限与安全

- 页面层：无鉴权分支，无未授权态（FR-16-33）。store 中的数据未按租户过滤（page.tsx:58–62）；store 的 `tenantId` 字段存在（store.ts:400，初始状态自 store.ts:397 起）但页面不使用，❓（需确认宿主会话是否在入口过滤租户）。
- 报表引擎：`ReportService` 持有 `user_port` 与 `permission_port`，两者标注 `#[allow(dead_code)]`（lib.rs:456–465），权限校验未生效。
- 仪表盘：`DashboardService::create` 对 `Global` 范围检查 `is_admin`（lib.rs:242–261）；Wallboard 公开链接与过期不存在（FR-16-30）。
- BI 任务事件接口：处理函数先调用 `authorize_run_owned_task_scope`（bi_analytics.rs:135）并设置超时（:143）。
- 导出：无水印、无审计写入（charts-and-reports INV-REPORT-11 未实现）。

### 4.6 依赖关系

- 分析页依赖：`useStore`（frontend/src/lib/store.ts）、`buildAnalyticsData`（chartData.ts）、`ChartFrame`（components/charts/shared/ChartFrame.tsx）、15 个图表组件、`GanttView`（components/gantt/GanttView.tsx:219 嵌入）、i18n `useTranslation`（page.tsx 顶部导入）、lucide 图标。
- 图表组件依赖 `recharts`（frontend/package.json:44；Chart01Burndown.tsx:21）。
- 报表引擎依赖：应用层端口（application/ports.rs）与内存桩；缓存（application/cache.rs:9 注明 InMemory 为阶段实现）。
- 仪表盘引擎：无运行时依赖；未被其他 crate 调用。
- star-dispatcher 在 DOMAIN_CRATES 列表中引用 `domain-report` 与 `domain-dashboard`（crates/star-dispatcher/src/sa_real_impls.rs:14、250、263），仅用于 crate 名称校验，不调用引擎。
- Lane BI 两种表述并存：BranchWorkspace.tsx:180–190 称「BI / 分析已合并进分析界面」并链接 `/analytics`（BranchWorkspace 由 frontend/src/app/worktree/page.tsx:583、618 渲染）；RunWorkspaceShell.tsx:223 在 Lane 运行页渲染 `RunBiPanel`，后者调用 `listRunTaskEvents`（frontend/src/lib/run/runDirectoryApi.ts:685–693）。`/analytics` 不调用 `bi/task-events`。归属见 open_items。
- `/ops`（frontend/src/app/ops/page.tsx）与 `/audit`（frontend/src/app/audit/page.tsx）为独立路由；分析页内的运维与审计日志标签与其重复，且不复用其数据（FR-16-03、FR-16-24）。
- 导航：`registry.ts:778–783` 定义「分析」分组，tabs 为 analytics、ops、audit、sandbox（与分析页内标签同名，数据源未共享）。

### 4.7 页面设计

### 4.7.1 /analytics

文件: `frontend/src/app/(app)/analytics/page.tsx`（客户端组件）；数据聚合 `frontend/src/app/(app)/analytics/chartData.ts`。

布局:

| 区域 | 提供方 | 位置 |
|---|---|---|
| 顶栏 | AppHeader，由 AppShell 渲染 | components/AppShell.tsx:49；默认顶栏标签不含 analytics（frontend/src/lib/nav/navStore.ts:94） |
| 侧栏 | Sidebar，由根布局渲染 | frontend/src/app/layout.tsx:78 |
| 主区 | AppShell 的 main；(app) 布局对 `/analytics` 使用全高度前缀 | components/AppShell.tsx:61–72；frontend/src/app/(app)/layout.tsx:35、:49 |
| 右侧栏 | RightChatRail（全局） | components/AppShell.tsx:77 |
| 底栏 | ❓ AppShell 未渲染底栏组件（components/AppShell.tsx:31–79）；需检索其他底栏组件确认是否存在 | — |

组件树:

- `AnalyticsPage`（page.tsx:54）
  - 页头：标题「分析」（page.tsx:114）、「全局」徽标（:115）、项目选择（:118）、时间范围选择（:123）
  - 标签导航 `analytics-tabs`（page.tsx:131–150），标签列表 `ANALYTICS_TABS`（:38–42）
  - 主区 main（page.tsx:152）
    - 概览（:153）：KPI 网格（:155–168）、IPA 进度（:170）、范围卡（:194）
    - 甘特图（:214–222）：ChartFrame（:216）、GanttView（:219）
    - 图表标签（:225–229）：预览横幅（:82–87）、`renderChart` switch（:89–107）、ChartFrame（components/charts/shared/ChartFrame.tsx:26）与 Chart01–Chart15
    - 运维（:231–252）
    - 审计日志（:253–291）

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | store 含 workItems、sprints、agentSessions | KPI、图表、IPA 进度（常量）、运维与日志（字面量） | 🟡（数据部分为字面量，见 FR-16-01、FR-16-10） |
| 加载 | 页面无异步请求 | 无页面级加载态；甘特 Suspense 回退文案（page.tsx:217–218）仅覆盖甘特图 | ⬜ |
| 空 | store 为空数组（store.ts:397–436） | KPI 显示 `0 / 1`（page.tsx:68、75）；图表使用 `spr-preview` 与回退数字（chartData.ts:31–34、52、91、211、216） | 🟡（以合成数据冒充，见 FR-16-11） |
| 错误 | 无数据请求失败路径 | 无；ChartFrame `error` 属性未传入（page.tsx:91–107） | ⬜ |
| 未授权 | 无鉴权分支 | 无 | ⬜ |
| 未接入 | 宿主 BI 投影未接入 | 仅通用预览横幅（page.tsx:82–87）；运维与日志无「未接入」标注（page.tsx:231–291） | ⬜ |
| 禁用 | 控件无禁用条件 | 项目与时间范围选择始终可见且可操作但无效果（page.tsx:118–127） | ⬜ |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换标签 | 无 | 设置 `activeTab` 并渲染对应区块（page.tsx:140–141） | 本地状态 | ✅ |
| 选择项目 | 无 | 无效果（page.tsx:118–121） | 无 | ⬜ |
| 选择时间范围 | 无 | 无效果（page.tsx:123–126） | 无 | ⬜ |
| 导出 CSV、PNG | 图表标签 | 按钮不出现（ChartFrame.tsx:21 未传 `onExport`） | 无页面接口；引擎 `export_csv` 无路由（lib.rs:684） | ⬜ |
| 订阅 | 图表标签 | 按钮不出现（ChartFrame.tsx:22 未传 `onSubscribe`） | 无 | ⬜ |
| 甘特图内操作 | 甘特图标签 | 由 GanttView 处理（组件细节不在本功能范围） | store（board、workItems、identities） | 🟡 |
| store 变化后刷新 | store 更新 | `useMemo` 重算图表与 KPI（page.tsx:64、66） | store | 🟡（无轮询，charts-and-reports §0.1 未实现） |

数据:

- store：`workItems`、`agentSessions`、`sprints`、`board`、`identities`（store.ts:147–176），初始为空（store.ts:397–436）。
- 字面量：`IPA_PHASES`（page.tsx:45–52）、分支名（:173）、趋势值（:75–78）、项目与时间范围选项（:119–126）、范围卡（:195–200）、运维指标（:237–242）、操作日志（:271–274）、图表回退数据（chartData.ts:52、91、132–133、160、175–178、211–217）。
- REST：无。MSW `/api/analytics/*` 存在但不被页面消费（mocks/handlers/analytics.ts:16、19）。

与文档设计的差距:

- 未满足 FR-AN-003 与 FR-16-03：运维区无「未接入」标识。
- 未满足 mockup §1 的集群与发布区块（FR-16-06、FR-16-08）；指标区块项目不一致（FR-16-07）。
- 操作日志列与设计不一致，数据非事件源（FR-16-04、FR-16-24）。
- 范围与时间范围控件未生效（FR-16-05、FR-16-20）。
- C16–C22 标签缺失（FR-16-12）；甘特图 `chartId` 与 C16 冲突（FR-16-25）。
- 导出与订阅未提供（FR-16-16、FR-16-17）。
- 数据未来自授权投影（FR-16-23）；空数据以合成值冒充（FR-16-11）。
- 加载、错误、未授权、未接入的页面态未实现（FR-16-33）。
- 页面未对接 charts-and-reports §5 的交互（钻取与分享）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 分析页面 | frontend/src/app/(app)/analytics/page.tsx | 🟡 | 标签 :38–42；KPI :66–80；渲染 switch :89–107；运维 :231–252；审计 :253–291 |
| 图表数据聚合 | frontend/src/app/(app)/analytics/chartData.ts | 🟡 | sprintOf :31–34；回退字面量 :52、91、132–133、175–178、211–217 |
| 图表外壳 | frontend/src/components/charts/shared/ChartFrame.tsx | 🟡 | 属性 :14–23；组件 :26；导出与订阅按钮受 props 控制 :21–22 |
| C01–C15 图表组件 | frontend/src/components/charts/Chart01Burndown.tsx 至 Chart15PriorityDist.tsx | 🟡 | 15 个文件；无测试文件；Chart01 引入 recharts :21 |
| C16–C22 图表组件 | 无 | ⬜ | frontend/src/components/charts 目录无对应文件 |
| 甘特标签 | frontend/src/components/gantt/GanttView.tsx | 🟡 | 嵌入 page.tsx:219；组件细节归甘特功能 |
| 报表引擎 | crates/domain-report/src/lib.rs | 🟡 | ReportService :456–485；缓存 :504–530；导出 :679–684；ReportType impl :84 |
| C01 Burndown 计算 | crates/domain-report/src/domain/c01_burndown.rs | 🟡 | generate :90；测试 crates/domain-report/tests/c01_burndown_test.rs:54、74、93、110、147、190 |
| C02 Burnup 计算 | crates/domain-report/src/domain/c02_burnup.rs | 🟡 | generate :56；端口调用 :66、73–74 |
| C03–C15 计算 | crates/domain-report/src/domain/c03_velocity.rs 至 c15_priority_dist.rs | 🟡 | mock 与未用端口见 FR-16-15 |
| C16–C22 计算 | crates/domain-report/src/domain/c16_assignee_workload.rs 至 c22_recently_created.rs | 🟡 | `not_implemented`：c16 :16–21，c17 :12–17，c22 :12–17 |
| 端口内存桩 | crates/domain-report/src/infrastructure/port_stubs.rs | 🟡 | InMemoryWorkItemPort :26、InMemorySprintPort :76、InMemoryUserPort :114、InMemoryPermissionPort :131 |
| 报表缓存 | crates/domain-report/src/infrastructure/in_memory_cache.rs | 🟡 | TTL 写入 :53–56；过期测试 :97 |
| 仪表盘引擎 | crates/domain-dashboard/src/lib.rs | 🟡 | GadgetType :22；Gadget :103；Dashboard :129；add_gadget 上限 :173–175；enable_wallboard :194–197；DashboardService :233–261；测试 :301–383 |
| 任务事件接口 | crates/star-api-rest/src/group_api/bi_analytics.rs | ✅ | 路由 :61–63；处理 :112；授权 :135；超时 :143；单元测试 :185、:202；挂载 group_api.rs:814 |
| Lane BI 面板 | frontend/src/components/run/RunBiPanel.tsx；RunWorkspaceShell.tsx:223 | ✅ | 调用 runDirectoryApi.ts:685–693；归属 Lane 功能 |
| 分析页测试 | frontend/src/app/(app)/__tests__/panels.test.tsx:110–120 | 🟡 | 断言 KPI 卡 0、3 存在及 IPA 区块，见 §7 |
| 导航入口 | frontend/src/lib/nav/registry.ts:219–230、:778–783 | ✅ | 核心模块与分组定义 |
| 全高度路由 | frontend/src/app/(app)/layout.tsx:35、:49 | ✅ | 前缀列表与 AppShell 渲染 |
| 分析 MSW mock | frontend/src/mocks/handlers/analytics.ts:15–19 | 🟡 | 存在但页面不消费（测试仅断言 handler 数量） |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 运维区无「未接入」标识（FR-16-03）；运维指标为字面量（page.tsx:237–242） | 静态数字冒充真实运维指标 | 对每项指标标注数据源；无源时显示「未接入」并禁用相关展示 |
| 操作日志为字面量，列与设计不符（FR-16-04、FR-16-24；page.tsx:271–274） | 日志不反映真实操作 | 改读 `store.auditEvents` 或授权事件接口，列改为时间、操作者、事件、范围 |
| 项目与时间范围控件无效（FR-16-05；page.tsx:118–127） | 用户误以为可过滤 | 将控件状态传入数据聚合；无后端能力前禁用并说明原因 |
| C16–C22 前端未接入（FR-16-12；page.tsx:105–106） | 22 类图表只有 15 类可见 | 为 C16–C22 增加 switch 分支与组件；后端先替换 `not_implemented` |
| C16–C22 与 C03–C15 计算为 mock 或未用端口（FR-16-13、FR-16-15） | 图表数值不可信 | 逐模块接入 WorkItemQueryPort、SprintQueryPort 的真实实现，删除 mock 分支 |
| 空数据以合成值冒充（FR-16-11；chartData.ts:31–34、52、91、211、216） | 空项目展示虚假进度与分布 | 数据为空时渲染空态；移除回退字面量 |
| 数据来源为 store 聚合，无授权投影（FR-16-10、FR-16-23） | 与 SRS「授权事实」要求不符 | 定义分析读取端口，由宿主 BI 或数据库投影实现；页面经接口取数 |
| 报表引擎无 REST 路由与持久化（FR-16-19、FR-16-18） | 引擎无法被页面调用 | 增加 `/api/reports/*`；实现 ReportDefinition 仓储 |
| 端口为内存桩（port_stubs.rs:26、76、114、131） | 数据不随业务变化 | 接入 domain-work-item、domain-planning、domain-identity、domain-permission 的真实端口 |
| 导出仅有 JSON 与 CSV 引擎函数，无页面入口（FR-16-16） | 图表无法导出 | 增加导出路由与页面按钮；补 XLSX、PNG、PDF |
| 订阅、调度、降噪未实现（FR-16-17、FR-16-18） | 定期报告不可用 | 实现 ReportSubscription 与调度端口，接入通知 |
| 采样与拒绝阈值未实现（FR-16-21） | 大数据量查询无保护 | 在 ReportService 中按 10K、100K 判定 |
| Redis 缓存未实现，仅内存缓存（FR-16-22；cache.rs:9） | 多实例缓存不一致 | 实现 Redis 适配器 |
| 甘特标签 `chartId` 为 `C16_GANTT`（FR-16-25；page.tsx:216） | 与 C16 编号冲突，文档检索歧义 | 改为非报表编号，如 `GANTT` |
| 仪表盘上限 20 与 spec 的 12 不一致；无重叠校验（FR-16-28） | 布局与 spec 不符 | 以 spec 为准修改上限，实现重叠校验 |
| Gadget 类型名称与 spec 不一致（FR-16-27） | 与 chart-*、wallboard-* 设计不对应 | 按 dashboard-spec §5 重命名或映射 |
| 仪表盘无持久化、REST、页面、共享（FR-16-26、FR-16-29、FR-16-31） | 仪表盘不可用 | 增加 DashboardRepository、REST 与 `/dashboards` 页面 |
| Wallboard 无公开链接与过期（FR-16-30） | 无法对外展示 | 实现 link_token 与 30 天过期 |
| 页面无加载、错误、未授权状态（FR-16-33） | 失败时界面无反馈 | 接入异步取数后补齐四态 |
| 「成本趋势」「排行榜」出现在 `/analytics` 副标题（frontend/src/lib/i18n/zh-CN.ts:460）但页面无此内容 | 文案与功能不符 | 删除副标题中的未实现项，或实现对应区块 |
| 分析页与 `/ops`、`/audit` 的运维与审计标签重复且不共享数据（§4.6） | 同一数据两处呈现且可能不一致 | 分析页改为引用同一数据接口，或保留单一入口 |
| 页面 KPI 口径与标签不符（FR-16-01） | 指标误导 | 验证通过率与运行时长按各自数据源重算并改标签 |
| SRS 状态与代码不符（SRS 标 FR-AN-001/002/004 为 ✅，FR-AN-003 为未实现，且标「已闭环」） | 文档不反映实际状态 | 按本文件 §3 状态回写 SRS 的 §2 与 §4 |

## 7. 验收与测试

已有测试:

- `frontend/src/app/(app)/__tests__/panels.test.tsx:110–120`：渲染 `AnalyticsPage`，断言 `analytics-kpi-0`、`analytics-kpi-3`、「完成任务」「IPA 技能覆盖」、`analytics-ipa-phases`、「需求定义」「集成测试」。
- `crates/domain-report/tests/c01_burndown_test.rs`：6 个用例（行 54、74、93、110、147、190），覆盖 C01 基本计算、缺 sprint_id、sprint 不存在、总 SP 为零、范围变更、缓存命中与失效。
- `crates/domain-report/src/infrastructure/in_memory_cache.rs:97`：TTL 过期测试。
- `crates/domain-report/src/domain/c16_assignee_workload.rs:34`、`c17_component_workload.rs:28`、`c22_recently_created.rs:28`：`not_implemented` 结构测试（c18–c21 同构）。
- `crates/domain-dashboard/src/lib.rs:301–383`：7 个用例，覆盖 GadgetType 数量与默认尺寸、个人与 Global 创建权限、添加与移除 Gadget、Wallboard 置位。
- `crates/star-api-rest/src/group_api/bi_analytics.rs:185、202`：查询参数校验与分页上限测试（任务事件接口）。

已验证（基于代码阅读）:

- C01 与 C02 读取 WorkItem 与 Sprint 端口；端口为内存桩，结果不随业务数据变化。
- C16–C22 的 `generate` 返回 `NotImplemented`。
- 仪表盘实体的添加、移除与 Wallboard 置位逻辑存在。

未验证:

- 分析页的 KPI 计算口径、图表回退值、空态呈现。
- 图表组件（Chart01–Chart15）的渲染与交互：`frontend/src/components/charts` 下无测试文件。
- `chartData.ts` 无测试；`git grep` 于 frontend 中仅 panels.test.tsx 导入分析页。
- 范围与时间范围控件、导出与订阅按钮：无测试覆盖。
- 页面端到端流程：❓（需检索 frontend/e2e 中对 `/analytics` 的用例确认是否存在）。
- 报表与仪表盘的数据库表及迁移：❓（需检索 migrations 或 schema 文件）。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/requirements/SRS-ANALYTICS-001.md | FR-AN-001 至 FR-AN-004（:28–31）；§1 设计要点；§0 数据源说明 | SRS 将 FR-AN-001、002、004 标为 ✅，且标「已闭环」；代码存在字面量、口径偏差与静态表（见 FR-16-01、02、04）；FR-AN-003 标为未实现，与代码现状一致 | 以代码为准；FR 状态按 §3 重定；SRS 的 ✅ 不采用 |
| docs/specs/domain-dashboard-spec.md | Dashboard、Gadget、INV-DASH-01 至 15、端口与错误模型 | 代码上限 20 与 INV-DASH-04 的 12 不符；GadgetType 名称与 §5 不符；无 Port trait；无重叠校验；无 ShareGrant | 设计意图保留并标注 ⬜ 或 🟡；现状以代码为准 |
| docs/specs/domain-report-spec.md | ReportDefinition、ReportSnapshot、ReportSchedule、ReportSubscription、INV-REPORT-01 至 20、端口、导出 | 代码无 ReportDefinition、ReportSchedule、ReportSubscription；端口为内存桩；导出仅 JSON 与 CSV | 设计意图保留；现状以代码为准 |
| docs/requirements/charts-and-reports.md | 22 类图表、5 类 scope、P0/P1/P2 批次（§2）、交互需求（§5）、性能与验收（§3） | 设计文档中的实现路径（如 `crates/domain-report/src/charts/…`，见 docs/design/charts/c16-assignee-workload.md:4）与 pin 的 `domain/` 目录不符；批次与 lib.rs:3–6 一致 | 批次采用；路径以 pin 为准 |
| docs/basic-design/charts-and-reports.md | `/api/reports/*`、`/api/dashboards/*` 端点（:661–677）；前端 `(app)/reports/`、`(app)/dashboards/` 目录（:9–10 所列） | pin 无上述路由与目录；本页为单一标签页入口 | 端点列为 ⬜；页面入口以 `/analytics` 单页为现状，独立 reports 与 dashboards 路由不作为本功能现状 |
| docs/design/charts/c16-assignee-workload.md | C16 实现路径与依赖（头部 :4） | 路径与 pin 不符 | 仅引用头部；其余 21 份图表设计文档仅核对列表，未逐份读取 |
| docs/prototypes/5 · 分析-html/README.md、Analytics.dc.html | 集群、指标、发布区块（:79–81）；日志列（:87）；视觉参考 | 页面无集群与发布；日志列不同 | 视为视觉参考；区块需求采纳为 FR-16-06 至 08（⬜） |
| frontend/src/app/(app)/analytics/page.tsx | 页面结构、标签、字面量、store 读取 | — | 现状证据 |
| frontend/src/app/(app)/analytics/chartData.ts | 图表数据聚合与回退值 | — | 现状证据 |
| frontend/src/components/charts/shared/ChartFrame.tsx | 图表外壳属性 | — | 现状证据 |
| crates/domain-report/src/lib.rs、domain/c01、c02、c03–c22、infrastructure/*、application/* | 报表引擎实现与桩 | — | 现状证据 |
| crates/domain-dashboard/src/lib.rs | 仪表盘实体与服务 | — | 现状证据 |
| crates/star-api-rest/src/group_api/bi_analytics.rs、group_api.rs:814 | 任务事件接口 | — | 现状证据 |
| frontend/src/components/run/BranchWorkspace.tsx:180–190；RunWorkspaceShell.tsx:223；RunBiPanel.tsx；runDirectoryApi.ts:685–693 | Lane BI 两种表述 | 同一 BI 功能在两处表述不同 | 两处并存记录；归属见 open_items |
| frontend/src/lib/store.ts:394–436 | 空数据原则；初始状态；字段类型 | 页面未遵守该原则 | 原则采纳为 FR-16-11 |
| frontend/src/lib/nav/registry.ts:219–230、:778–783；navStore.ts:94、:127–136；(app)/layout.tsx:35、:49；app/layout.tsx:78；Sidebar.tsx:227；Sidebar.test.tsx:88 | 导航入口、布局区域、默认顶栏 | 默认顶栏不含 analytics；入口依赖侧栏 | 现状证据 |
| frontend/src/mocks/handlers/analytics.ts | MSW `/api/analytics/kpi`、`/cost` | 页面不消费 | 仅 mock，不作为数据源 |
| frontend/src/lib/i18n/zh-CN.ts:460 | `/analytics` 副标题「成本趋势、排行榜」 | 页面无此内容 | 以代码为准；列为差距 |
| frontend/src/app/audit/page.tsx、frontend/src/app/ops/page.tsx | 独立审计与运维路由 | 与分析页标签重复 | 归其他功能；列为差距 |
| frontend/src/app/(app)/__tests__/panels.test.tsx:110–120 | 页面渲染与 KPI、IPA 断言 | 测试标题描述的标签结构与当前 `ANALYTICS_TABS`（page.tsx:38–42）不同 | 断言以代码为准 |
| crates/star-dispatcher/src/sa_real_impls.rs:14、250、263 | DOMAIN_CRATES 名称列表 | 仅作名称校验 | 不作为运行时依赖 |
| 未读取 | docs/requirements 其他文件；docs/design/charts 其余 21 份；docs/basic-design 其余文件（OPS、BD、SANDBOX）；migrations；frontend/e2e | 与本功能的关联未核对 | 列入 open_items 或 ❓ |
