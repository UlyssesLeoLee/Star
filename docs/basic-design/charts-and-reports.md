# 图表 & 报告系统 总基本设计 v1.0

> **状态**: Draft v1.0 (2026-09-02)
> **对标基线**: [charts-and-reports 总需求 v1.0](../requirements/charts-and-reports.md) + Jira Cloud Reports & Dashboards
> **触发**: 2026-09-02 10:04 JST Ulysses 拍板 (per ask_user 4 维拍板)
> **下游交付**:
> - 22 份详细设计 → `docs/design/charts/{chart-id}.md`
> - spec 升版 → `docs/specs/domain-report-spec.md` v1.0 + `docs/specs/domain-dashboard-spec.md` v1.0
> - Rust 实现 → `crates/domain-report/` + `crates/domain-dashboard/`
> - 前端实现 → `frontend/src/components/charts/` (22 组件) + `frontend/src/app/(app)/reports/` + `frontend/src/app/(app)/dashboards/`
> **技术栈** (per ask_user tech-stack=A):
> - 前端图表: **Recharts** (~250KB gzipped, React 友好, 主题/暗色/可访问性好, 覆盖 22 图表 90% 场景)
> - 后端: Rust (axum + tokio) per Star Modular Monolith 既有栈
> - 数据: PostgreSQL 事实表 + Redis 5min TTL 缓存

---

## 0. 文档说明

### 0.1 目标与定位

本文档是图表 & 报告系统的**总基本设计**,在需求 v1.0 之上给出:

1. **架构决策**: Recharts 选型理由 + 模块拆分 + 数据流
2. **数据模型**: ReportDefinition / ReportSnapshot / Dashboard / Gadget 完整 schema
3. **22 图表 data schema 模板**: 复用 `ChartSeriesConfig` + `ChartAxisConfig`
4. **API 设计**: REST/JSON 端点契约
5. **跨域接触面**: 与 work-item / planning / identity / permission 的协作
6. **非功能性**: 缓存 / 安全 / 错误 / 可观测

### 0.2 文档关系

```
requirements/charts-and-reports.md (总需求 v1.0)
    ↓
basic-design/charts-and-reports.md (本文档)
    ↓
design/charts/{chart-id}.md (22 份详细设计, 每图 1 份)
    ↓
specs/domain-report-spec.md v1.0 + specs/domain-dashboard-spec.md v1.0
    ↓
crates/domain-report/ + crates/domain-dashboard/ + frontend/src/components/charts/
```

### 0.3 不在范围

- ECharts / D3 / 自研 SVG 替代方案(per ask_user tech-stack=A, Recharts 锁定)
- 实时流式图表(WebSocket 推送留 V2, 本期 30s polling)
- BI 集成(Tableau / Power BI 嵌入留 V2)
- 跨 tenant 数据聚合(per NFR-OP-010 强租户隔离, 永远单 tenant 内)

### 0.4 dual-use 警告

per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板:

> domain-report / domain-dashboard 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 架构概览

### 1.1 系统边界

```
┌─────────────────────────────────────────────────────────────┐
│  Star Frontend (Next.js 14 App Router)                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  React Components (Recharts)                          │  │
│  │  <Chart01Burndown/> <Chart02Burnup/> ... <Chart22/>   │  │
│  │  + <ReportBuilder/> <DashboardEditor/>                │  │
│  └───────────────────┬───────────────────────────────────┘  │
│                      │ fetch (REST/JSON)                    │
│                      │ + Server-Sent Events (实时刷新)       │
└──────────────────────┼──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  API Gateway (axum, port 8080)                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  /api/reports/*  /api/dashboards/*  /api/charts/*     │  │
│  │  + Auth + Tenant middleware + Permission check        │  │
│  └───────────────────┬───────────────────────────────────┘  │
└──────────────────────┼──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  Application Service Layer (Rust)                           │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │  domain-report   │  │ domain-dashboard │                │
│  │  (10 report type)│  │ (10 gadget type) │                │
│  │  - Aggregate     │  │ - Layout         │                │
│  │  - Snapshot      │  │ - Permission     │                │
│  │  - Schedule      │  │ - Wallboard      │                │
│  └────────┬─────────┘  └────────┬─────────┘                │
│           │                     │                           │
│           │  Port (interface)  │                           │
│           ▼                     ▼                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  domain-work-item Projection (事实源)                 │  │
│  │  domain-planning Projection                           │  │
│  │  domain-identity Projection                           │  │
│  │  domain-permission                                    │  │
│  └───────────────────────────────────────────────────────┘  │
│                          │                                  │
│                          ▼                                  │
│              ┌──────────────────────┐                       │
│              │  PostgreSQL + Redis  │                       │
│              │  (5min TTL 缓存)     │                       │
│              └──────────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 关键架构决策

#### ADR-CHART-001: Recharts 选型(per ask_user tech-stack=A)

| 维度 | Recharts | ECharts | D3 + 自研 |
|---|---|---|---|
| 体积 | ~250KB gzipped | ~400KB gzipped | 0 但自研 22 组件 |
| React 集成 | 声明式, 零包装 | 需 echarts-for-react | 命令式 + ref |
| 主题/暗色 | 内置 (Recharts 2.x) | 需手动配置 | 全自研 |
| 可访问性 | 内置 aria-label | 需手动 | 全自研 |
| 图表覆盖 | 22 图表 90% (90% 覆盖) | 100% (50+ 图表) | 100% (但成本高 2-3x) |
| 维护成本 | 低 (社区活跃) | 中 (大版本有 breaking) | 高 (全自研) |
| 性能 (10K 点) | 中 (Canvas + SVG 混合) | 高 (Canvas) | 取决于实现 |

**决策**: 选 **Recharts**。理由:
1. 22 图表 90% 覆盖,10% (Heatmap / Gantt-like) 走 SVG 自研包装
2. React 友好,与 Star 仓 Next.js 14 栈契合
3. 主题/暗色/a11y 内置,符合 §8 NFR
4. 维护成本最低,符合 token-OLU 节约原则

**例外** (per Recharts 限制):
- **Heatmap (C21)**: 自研 SVG (Recharts 无原生 Heatmap, per react-heatmap 也不够灵活)
- **Sankey / Treemap** (本期不在 22 内, 留 V2)

#### ADR-CHART-002: 5min TTL 缓存(per INV-REPORT-02)

所有 Report 数据走 Redis 5min TTL 缓存,理由:
- 报表对实时性要求低(分析视图, 秒级 vs 5min 级肉眼无感)
- 5min 缓存能消解 90%+ 重复查询
- 大数据查询(< 100K issue) P95 < 3s 难达, 缓存后 P95 < 200ms

**失效策略**:
- TTL 自然过期 (5min)
- 显式 invalidate: domain-work-item 写操作触发 `ReportCache.invalidate(scope)`
- 强制刷新: Report 订阅触发走 worker 异步生成新 snapshot

#### ADR-CHART-003: Report 订阅走 worker projection role(per INV-REPORT-03)

per basic-design v0.16 §4.12.2, Report 订阅触发必须走 worker projection role:
- API 接收订阅 → 入队 (`report_subscribe_queue`)
- worker 异步生成 snapshot + 通知
- 避免阻塞 API 线程

#### ADR-CHART-004: Report 不持 SoR 业务事实(per INV-REPORT-01)

Report / Dashboard 仅持有:
- 配置 (ReportDefinition / Dashboard)
- 缓存 (ReportSnapshot, TTL 5min)
- 订阅关系 (User ↔ Report)

事实永远在 SoR 域(work-item / planning / identity)。

---

## 2. 模块拆分

### 2.1 顶层模块图

```
crates/
├── domain-report/           # 10 报表类型 + 订阅 + 调度
│   ├── src/
│   │   ├── lib.rs
│   │   ├── domain/
│   │   │   ├── report_definition.rs  # 聚合根
│   │   │   ├── report_snapshot.rs    # 投影
│   │   │   ├── report_schedule.rs    # cron
│   │   │   ├── report_subscription.rs
│   │   │   └── report_filter.rs      # JQL/SQL 解析
│   │   ├── application/
│   │   │   ├── command_service.rs    # create/update/delete/enable
│   │   │   ├── query_service.rs      # get/list-by-project
│   │   │   └── generate_service.rs   # 异步生成 snapshot
│   │   ├── infrastructure/
│   │   │   ├── postgres_repo.rs
│   │   │   ├── redis_cache.rs
│   │   │   └── port_impl/
│   │   │       ├── workitem_port.rs
│   │   │       ├── planning_port.rs
│   │   │       ├── identity_port.rs
│   │   │       └── permission_port.rs
│   │   └── api/
│   │       ├── rest_handlers.rs
│   │       └── graphql_handlers.rs   # (可选 V2)
│   └── tests/
│       └── e2e_22_charts.rs          # 22 图表端到端测试
│
├── domain-dashboard/        # 10 Gadget + 12-grid + Wallboard
│   ├── src/
│   │   ├── lib.rs
│   │   ├── domain/
│   │   │   ├── dashboard.rs
│   │   │   ├── gadget.rs
│   │   │   ├── subscription.rs
│   │   │   └── wallboard.rs
│   │   ├── application/
│   │   ├── infrastructure/
│   │   └── api/
│   └── tests/
│
frontend/
├── src/
│   ├── components/
│   │   ├── charts/                    # 22 Recharts 包装
│   │   │   ├── Chart01Burndown.tsx
│   │   │   ├── Chart02Burnup.tsx
│   │   │   ├── ...
│   │   │   ├── Chart22RecentlyCreated.tsx
│   │   │   └── shared/
│   │   │       ├── ChartFrame.tsx     # 通用外壳 (导出/订阅/分享按钮)
│   │   │       ├── ChartFilterBar.tsx # filter 选择
│   │   │       ├── ChartLegend.tsx
│   │   │       └── ChartEmpty.tsx
│   │   ├── reports/
│   │   │   ├── ReportBuilder.tsx     # 创建 Report
│   │   │   ├── ReportView.tsx
│   │   │   ├── ReportList.tsx
│   │   │   └── ReportSubscription.tsx
│   │   └── dashboards/
│   │       ├── DashboardEditor.tsx
│   │       ├── DashboardView.tsx
│   │       ├── DashboardGrid.tsx     # 12-grid 拖拽
│   │       ├── GadgetPicker.tsx
│   │       └── WallboardView.tsx
│   ├── lib/
│   │   ├── chart-data-schema.ts      # 22 图表 data schema TS 类型
│   │   ├── chart-render-helpers.ts   # Recharts 通用配置
│   │   └── chart-export.ts           # CSV/XLSX/PNG/PDF 导出
│   ├── app/
│   │   ├── (app)/
│   │   │   ├── reports/              # /reports 路由组
│   │   │   │   ├── page.tsx          # Report 列表
│   │   │   │   ├── new/page.tsx
│   │   │   │   └── [id]/page.tsx     # Report 详情
│   │   │   └── dashboards/
│   │   │       ├── page.tsx
│   │   │       ├── new/page.tsx
│   │   │       └── [id]/page.tsx
│   │   └── api/
│   │       ├── reports/
│   │       └── dashboards/
│   └── i18n/
│       ├── zh-CN.json
│       ├── en-US.json
│       └── ja-JP.json
```

### 2.2 后端模块边界

| 模块 | 职责 | 不负责 |
|---|---|---|
| `domain-report` | 10 报表 type + 调度 + 订阅 | 事实存储, 跨域权限 |
| `domain-dashboard` | 10 Gadget + 12-grid + Wallboard | 图表数据, 订阅调度 |
| `domain-work-item` (既有) | WorkItem 事实 | 报表, 仪表盘 |
| `domain-planning` (既有) | Sprint / Version 事实 | 报表, 仪表盘 |
| `domain-identity` (既有) | User / Group | 报表, 仪表盘 |
| `domain-permission` (既有) | RBAC | (无) |

---

## 3. 数据模型

### 3.1 ReportDefinition (聚合根)

```rust
// crates/domain-report/src/domain/report_definition.rs

pub struct ReportDefinition {
    pub report_id: ReportId,                // UUID
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,      // None = 全 tenant
    pub scope: ReportScope,                 // S1-S5
    pub chart_type: ChartType,              // C01-C22
    pub title: String,
    pub description: Option<String>,
    pub filter_id: Option<FilterId>,        // S5 必须
    pub config: ChartConfig,                // 图表 specific
    pub schedule: Option<ReportSchedule>,   // cron
    pub subscriptions: Vec<ReportSubscription>,
    pub owner_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,                       // 乐观锁
}

pub enum ReportScope {
    Project(ProjectId),
    ProjectHierarchy(WorkspaceId),
    Sprint(SprintId),
    Version(VersionId),
    IssueFilter(FilterId),                  // S5
}

pub enum ChartType {
    C01Burndown, C02Burnup, C03Velocity, C04SprintReport, C05Cfd,
    C06ControlChart, C07CycleTime, C08Throughput, C09Forecast,
    C10TimeTracking, C11ResolutionTime, C12Sla,
    C13CreatedVsResolved, C14IssueType, C15Priority,
    C16AssigneeWorkload, C17ComponentWorkload,
    C18VersionWorkload, C19ReleaseBurndown, C20TimeInStatus,
    C21Heatmap, C22RecentlyCreated,
}
```

### 3.2 ChartConfig (22 图表统一配置 schema)

```rust
pub struct ChartConfig {
    pub time_range: TimeRange,                  // 默认 / 自定义
    pub granularity: Option<TimeGranularity>,   // day / week / month
    pub top_n: Option<u32>,                      // 限制 top N (Assignee/Component)
    pub y_axis_unit: Option<YAxisUnit>,         // sp / issue / hours / days
    pub stack_mode: Option<StackMode>,           // none / stack / percent
    pub log_scale: bool,                        // C06 Control Chart log 刻度
    pub forecast_method: Option<ForecastMethod>, // C09
    pub sla_definition_id: Option<SlaId>,        // C12
    pub show_ideal_line: bool,                   // C01/C02/C19
    pub show_average_line: bool,                // C03/C07
    pub color_scheme: ColorScheme,               // default / protanopia / deuteranopia
    pub locale: Locale,                         // zh-CN / en-US / ja-JP
}

pub struct TimeRange {
    pub mode: TimeRangeMode,                    // LastNDays / ThisSprint / Custom
    pub n_days: Option<u32>,
    pub custom_start: Option<DateTime<Utc>>,
    pub custom_end: Option<DateTime<Utc>>,
}
```

### 3.3 ReportSnapshot (投影)

```rust
pub struct ReportSnapshot {
    pub snapshot_id: SnapshotId,
    pub report_id: ReportId,
    pub generated_at: DateTime<Utc>,
    pub data: serde_json::Value,         // 22 图表 data schema (TS 同构)
    pub data_source_refs: Vec<DataSourceRef>,  // 引用了哪些 work-item / sprint id
    pub cache_key: String,               // Redis key
    pub ttl: Duration,                   // 5min
    pub render_hints: RenderHints,       // 客户端渲染优化提示
}

pub struct DataSourceRef {
    pub source_type: String,             // "work_item" / "sprint" / "version"
    pub source_ids: Vec<String>,
}

pub struct RenderHints {
    pub total_data_points: u32,          // > 10K 提示采样
    pub chart_height: u32,               // 默认 400
    pub chart_width: u32,                // 自适应
    pub show_legend: bool,
    pub show_tooltip: bool,
}
```

### 3.4 Dashboard / Gadget (聚合根)

```rust
pub struct Dashboard {
    pub dashboard_id: DashboardId,
    pub tenant_id: TenantId,
    pub owner_id: UserId,
    pub title: String,
    pub description: Option<String>,
    pub layout: DashboardLayout,                // 12-grid
    pub gadgets: Vec<DashboardGadget>,
    pub shared_with: Vec<ShareGrant>,
    pub is_wallboard: bool,                     // 全屏模式
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct DashboardLayout {
    pub columns: u32,                            // 12 (Tailwind 标准)
    pub row_height: u32,                         // 默认 80px
    pub gap: u32,                                // 默认 16px
}

pub struct DashboardGadget {
    pub gadget_id: GadgetId,
    pub gadget_type: GadgetType,                 // 10 种
    pub position: GadgetPosition,
    pub size: GadgetSize,
    pub config: serde_json::Value,               // 各 gadget 不同
    pub chart_report_id: Option<ReportId>,       // chart-* gadget 引用 report
}

pub struct GadgetPosition {
    pub x: u32,                                  // 0-11
    pub y: u32,
}
pub struct GadgetSize {
    pub w: u32,                                  // 1-12
    pub h: u32,
}

pub enum GadgetType {
    Chart(ChartType),                            // 22 图表
    Text,                                        // markdown
    Activity,                                    // 活动流
    AssignedToMe,                                // 个人待办
    FilterResults,                               // 过滤列表
    WallboardClock,                              // 数字时钟
    WallboardSla,                                // 大字号 SLA
    // + 3 扩展 (新增) — 留给 Phase 2
}
```

### 3.5 PostgreSQL DDL (W/T/M 分类 per AGENTS.md §4 #13)

> per AGENTS.md §4 守门 #13 (per 2026-09-01 18:30 JST 拍板), DB 表必须 W/T/M 三類横展開, 禁止混在。

#### Work (作業中, 短 TTL)

```sql
-- 报表生成任务 (per basic-design v0.16 §4.12.2 worker projection role)
CREATE TABLE report_generation_task (
    task_id UUID PRIMARY KEY,
    report_id UUID NOT NULL,
    status TEXT NOT NULL,  -- 'pending' / 'running' / 'done' / 'failed'
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ NOT NULL,  -- 完成后 24h 删除 (per W 派生)
    error_message TEXT
);
-- W 类: 物理删除 / retention 24h / 短 TTL
```

#### Transaction (业务事実, append-only, 監査必須)

```sql
-- 报表订阅事件 (per REQ-NOTIF-002 降噪, 走审计)
CREATE TABLE report_subscription_event (
    event_id UUID PRIMARY KEY,
    subscription_id UUID NOT NULL,
    user_id UUID NOT NULL,
    event_type TEXT NOT NULL,  -- 'subscribed' / 'unsubscribed' / 'notified' / 'opened'
    event_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    notification_id UUID,
    metadata JSONB,
    tenant_id UUID NOT NULL  -- RLS 必携
);
-- T 类: 物理删除禁止 / 監査必須 / RLS 必携

-- 报表查看审计
CREATE TABLE report_view_audit (
    audit_id UUID PRIMARY KEY,
    report_id UUID NOT NULL,
    user_id UUID NOT NULL,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ip_address INET,
    user_agent TEXT,
    tenant_id UUID NOT NULL
);
-- T 类

-- Dashboard 查看审计
CREATE TABLE dashboard_view_audit (
    audit_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    user_id UUID NOT NULL,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ip_address INET,
    user_agent TEXT,
    tenant_id UUID NOT NULL
);
-- T 类
```

#### Master (参考, SCD Type 2, RLS 必携)

```sql
-- Report 定义 (Master, per 配置)
CREATE TABLE report_definition (
    report_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID,
    scope_type TEXT NOT NULL,    -- 'project' / 'sprint' / 'version' / 'filter'
    scope_id UUID,
    chart_type TEXT NOT NULL,    -- C01-C22
    title TEXT NOT NULL,
    description TEXT,
    filter_id UUID,
    config JSONB NOT NULL,
    schedule JSONB,
    owner_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),  -- SCD Type 2
    valid_to TIMESTAMPTZ,                            -- SCD Type 2
    is_current BOOLEAN NOT NULL DEFAULT TRUE         -- SCD Type 2
);
-- M 类: 物理删除禁止 / SCD Type 2 / RLS 必携

-- Dashboard 定义
CREATE TABLE dashboard (
    dashboard_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    layout JSONB NOT NULL,
    is_wallboard BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);
-- M 类

-- Dashboard Gadget (作为 dashboard 子表)
CREATE TABLE dashboard_gadget (
    gadget_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL REFERENCES dashboard(dashboard_id),
    gadget_type TEXT NOT NULL,
    position JSONB NOT NULL,  -- {"x": 0, "y": 0}
    size JSONB NOT NULL,      -- {"w": 6, "h": 4}
    config JSONB NOT NULL,
    chart_report_id UUID REFERENCES report_definition(report_id)
);
-- M 类 (跟随 dashboard)

-- Report 订阅配置
CREATE TABLE report_subscription (
    subscription_id UUID PRIMARY KEY,
    report_id UUID NOT NULL REFERENCES report_definition(report_id),
    user_id UUID NOT NULL,
    cadence TEXT NOT NULL,  -- 'daily' / 'weekly' / 'monthly' / 'on_change'
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- M 类

-- Dashboard 共享
CREATE TABLE dashboard_share (
    share_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL REFERENCES dashboard(dashboard_id),
    grantee_type TEXT NOT NULL,  -- 'user' / 'group'
    grantee_id UUID NOT NULL,
    permission TEXT NOT NULL,    -- 'view' / 'edit'
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- M 类

-- SLA 定义 (供 C12 SLA Compliance 引用)
CREATE TABLE sla_definition (
    sla_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID,
    priority TEXT,
    target_resolution_hours INTEGER NOT NULL,
    description TEXT,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);
-- M 类
```

**派生规 (per 守门 #13)**:
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention** — `report_generation_task` 必须 24h 后删除
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携** — 3 個 T 表都有 tenant_id 必携
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携** — 5 個 M 表都有 valid_from/valid_to/is_current
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period** — 全表落实

**RLS Policy 示例** (per T/M 类必携):
```sql
ALTER TABLE report_definition ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON report_definition
    USING (tenant_id = current_setting('app.current_tenant')::UUID);
```

---

## 4. 数据流

### 4.1 渲染路径 (用户打开 Report)

```
1. User 打开 /reports/{id}
2. Next.js 服务端组件 (RSC) 拉取 ReportDefinition (PostgreSQL)
3. 返回 ReportDefinition + ReportSnapshot 缓存 (Redis, 5min TTL)
4. 客户端组件根据 chart_type 渲染对应 <ChartXX />
5. Recharts 客户端绘制
6. 用户切换 filter → 重新触发 /api/reports/{id}/data (query parameter)
7. 后端 query service 走 cache (5min) → 命中即返回, miss 走 domain-work-item 聚合
8. SSE 30s 推送 (per §4.3 实时)
```

### 4.2 创建 Report 路径

```
1. User 进入 /reports/new
2. <ReportBuilder /> 引导选择 scope + chart_type + filter + config
3. POST /api/reports → domain-report command service
4. command service: 校验权限 (domain-permission) → 写 PostgreSQL → 触发 report_generation_task (W 表)
5. worker 异步生成 ReportSnapshot → 写 Redis 缓存
6. SSE 通知 user: report ready
7. 跳转 /reports/{id}
```

### 4.3 实时刷新 (SSE)

- 30s polling 兜底 (per §7.2)
- SSE 优化路径: 后端 `EventBus` 监听 work-item 写事件 → 触发 report 失效 → SSE push "stale" 事件 → 客户端重新拉取

### 4.4 订阅通知

```
1. User 订阅 Report (cadence = daily)
2. report_subscription 入库 (M 表)
3. cron worker 每小时扫描 next_run_at <= now 的订阅
4. worker 调 generate_service 生成 snapshot
5. 调 notification service 发邮件 (per REQ-NOTIF-002 降噪)
6. 更新 last_run_at + 计算 next_run_at
```

### 4.5 Dashboard 加载

```
1. User 打开 /dashboards/{id}
2. 拉取 Dashboard + Gadgets (PostgreSQL)
3. 并行拉取所有 Gadget 数据 (最多 12 个, 12-grid 上限)
4. 各 Gadget 独立缓存 (5min)
5. 12-grid 布局 (Tailwind CSS Grid)
6. Wallboard 模式: 30s auto-refresh
```

---

## 5. API 设计

### 5.1 REST 端点

| Method | Path | 描述 | 权限 |
|---|---|---|---|
| GET | `/api/reports` | 列表 (按 project / scope 过滤) | project_view |
| GET | `/api/reports/{id}` | 详情 | report_view |
| POST | `/api/reports` | 创建 | report_create |
| PUT | `/api/reports/{id}` | 更新 | report_edit |
| DELETE | `/api/reports/{id}` | 删除 | report_delete |
| GET | `/api/reports/{id}/data` | 拉取图表数据 (含 query params) | report_view |
| POST | `/api/reports/{id}/refresh` | 强制刷新 (生成新 snapshot) | report_edit |
| GET | `/api/reports/{id}/export.{fmt}` | 导出 (csv/xlsx/png/pdf) | report_view |
| GET | `/api/reports/{id}/subscriptions` | 订阅列表 | report_view |
| POST | `/api/reports/{id}/subscriptions` | 订阅 | report_view |
| DELETE | `/api/reports/{id}/subscriptions/{sub_id}` | 退订 | self |
| GET | `/api/dashboards` | 列表 | tenant_view |
| GET | `/api/dashboards/{id}` | 详情 | dashboard_view |
| POST | `/api/dashboards` | 创建 | dashboard_create |
| PUT | `/api/dashboards/{id}` | 更新 | dashboard_edit |
| DELETE | `/api/dashboards/{id}` | 删除 | dashboard_delete |
| GET | `/api/dashboards/{id}/wallboard` | Wallboard 模式 | public (per link) |
| GET | `/api/filters` | 命名过滤器列表 | filter_view |
| POST | `/api/filters` | 创建过滤器 | filter_create |
| GET | `/api/sla-definitions` | SLA 定义列表 | project_admin |

### 5.2 拉取图表数据契约

```typescript
// GET /api/reports/{id}/data?time_range=last_30d&granularity=day

// Response
{
  report_id: string;
  chart_type: "C01_BURNDOWN" | "C02_BURNUP" | ... | "C22_RECENTLY_CREATED";
  generated_at: string;  // ISO 8601
  ttl_seconds: number;   // 300
  data: ChartData;       // 22 图表共用 schema (per §5.3)
  render_hints: {
    total_data_points: number;
    chart_height: number;
    show_legend: boolean;
  };
  data_source_refs: Array<{
    source_type: "work_item" | "sprint" | "version";
    source_ids: string[];
  }>;
}
```

### 5.3 22 图表 Data Schema (TS 模板)

```typescript
// frontend/src/lib/chart-data-schema.ts

export type ChartData =
  | Chart01BurndownData
  | Chart02BurnupData
  | Chart03VelocityData
  | ...
  | Chart22RecentlyCreatedData;

// 模板 1: Time-series (C01/C02/C05/C08/C13/C19/C21)
export interface TimeSeriesData {
  series: TimeSeries[];
  x_axis: { type: "date"; label: string; locale: string };
  y_axis: { type: "number"; label: string; unit: string };
}
export interface TimeSeries {
  name: string;
  data_points: Array<{ x: string /* ISO date */; y: number }>;
  color: string;
  dash_style?: "solid" | "dashed" | "dotted";
}

// 模板 2: Categorical (C14/C15/C16/C17/C18)
export interface CategoricalData {
  category_axis: { label: string };
  value_axis: { label: string; unit: string };
  data_points: Array<{ category: string; value: number; sub_values?: Record<string, number> }>;
}

// 模板 3: Scatter (C06)
export interface ScatterData {
  x_axis: { label: string; type: "date" | "number" };
  y_axis: { label: string; type: "number"; log_scale: boolean };
  data_points: Array<{ x: number; y: number; id: string; label?: string; anomaly?: boolean }>;
  reference_lines: Array<{ y_value: number; label: string; style: "solid" | "dashed" }>;
}

// 模板 4: Heatmap (C21)
export interface HeatmapData {
  x_categories: string[];  // ["Mon", "Tue", ..., "Sun"] or ["0", "1", ..., "23"]
  y_categories: string[];  // ["Week 1", "Week 2", ...] or hours
  values: number[][];      // [y_idx][x_idx] = count
  color_scale: { min: number; max: number; scheme: "viridis" | "blues" | "custom" };
}

// 模板 5: Table (C04/C10/C22)
export interface TableData {
  columns: Array<{ key: string; label: string; type: "string" | "number" | "date" | "duration" }>;
  rows: Array<Record<string, string | number | null>>;
  summary?: Record<string, string | number>;
}

// 模板 6: Histogram (C07)
export interface HistogramData {
  buckets: Array<{ range_start: number; range_end: number; count: number; label: string }>;
  percentiles: { p50: number; p85: number; p95: number };
}

// 模板 7: Forecast (C09)
export interface ForecastData {
  historical: TimeSeries;
  forecast: TimeSeries;
  confidence_bands: Array<{ level: number; series: TimeSeries }>;
  predicted_completion_date: string;
}

// 模板 8: Bar-with-Stacks (C16/C17/C18/C20)
export interface StackedBarData extends CategoricalData {
  stack_keys: string[];  // ["todo", "in_progress", "done"]
}
```

### 5.4 错误模型 (per basic-design v0.16 §6)

```typescript
export interface ApiError {
  code: string;          // "REPORT_NOT_FOUND" / "PERMISSION_DENIED" / "FILTER_INVALID"
  message: string;
  details?: Record<string, unknown>;
  trace_id: string;      // 用于关联日志
  timestamp: string;
}
```

错误码规范:
- `REPORT_*` - Report 相关
- `DASHBOARD_*` - Dashboard 相关
- `FILTER_*` - 过滤器相关
- `PERMISSION_*` - 权限
- `VALIDATION_*` - 入参校验
- `INTERNAL_*` - 内部错误

---

## 6. 跨域接触面 (per basic-design v0.16 §3.1)

| 接触 | 协作方式 | Port | 实现 |
|---|---|---|---|
| report 读 work-item | Customer-Supplier (OHS) | `WorkItemQueryPort` | domain-work-item |
| report 读 sprint | Customer-Supplier (OHS) | `SprintQueryPort` | domain-planning |
| report 读 version | Customer-Supplier (OHS) | `VersionQueryPort` | domain-planning |
| report 读 user | Customer-Supplier (OHS) | `UserQueryPort` | domain-identity |
| report 校验权限 | Customer-Supplier (OHS) | `PermissionPort` | domain-permission |
| report 写 audit | Conformist | `AuditRecorderPort` | domain-audit |
| report 触发通知 | Separate Ways (异步) | `NotificationPort` | domain-notification |
| dashboard 嵌入 report | Shared Kernel | `ReportDefinition` (同构) | domain-report |
| dashboard 校验权限 | Customer-Supplier | `PermissionPort` | domain-permission |

---

## 7. 缓存与性能

### 7.1 缓存策略

- **Redis 5min TTL** for ReportSnapshot (per ADR-CHART-002)
- **Key 格式**: `report:{tenant_id}:{report_id}:{config_hash}`
- **失效触发**:
  - TTL 自然过期
  - 显式 invalidate (work-item 写操作)
  - 用户强制刷新 (POST /api/reports/{id}/refresh)

### 7.2 大数据处理

- **> 10K 数据点**: 后端自动采样 (LTTB 算法), 提示用户细化 filter
- **> 100K**: 拒绝生成, 提示缩小 scope
- **Heatmap (C21)**: 服务端预聚合到 7×24 矩阵 (固定大小, 不需采样)

### 7.3 性能预算 (per 需求 §7)

| 阶段 | 预算 | 实测位置 |
|---|---|---|
| API GET /reports/{id}/data (cache hit) | < 50ms P95 | 服务端日志 |
| API GET /reports/{id}/data (cache miss) | < 3s P95 | 服务端日志 |
| Recharts 客户端首次渲染 (FCP) | < 1.5s P95 | Lighthouse |
| 切换 filter 重渲染 | < 500ms P95 | 前端埋点 |
| 12-gadget Dashboard 加载 | < 3s P95 | E2E |

---

## 8. 安全与权限

### 8.1 权限模型

per basic-design v0.16 §3.5 + AGENTS.md §0:

- **租户隔离**: 所有 Report/Dashboard 必须带 `tenant_id`, 走 RLS
- **角色**: viewer / editor / admin (per project / dashboard)
- **共享**: 通过 `dashboard_share` 表, 权限 = view / edit
- **Wallboard**: 可公开分享 (per token URL, 30 天过期)

### 8.2 数据访问

- Report 拉数据时校验:
  1. 租户匹配 (RLS)
  2. 用户对该 scope (project / sprint / version) 有 view 权限
  3. filter 不跨租户

### 8.3 导出安全

- 导出必须包含 watermark (per 报告查看审计的 user_id)
- CSV 导出大小限制 100MB
- PDF 异步生成 + 邮件通知 (避免阻塞)

---

## 9. 错误处理

### 9.1 错误分类

| 类型 | 例子 | 处理 |
|---|---|---|
| 客户端校验 | filter 语法错, config 缺字段 | 400 + 字段级错误 |
| 权限不足 | user 无 project_view | 403 |
| 资源不存在 | report_id 不存在 | 404 |
| 数据不一致 | sprint_id 不属于 project | 422 |
| 后端失败 | DB 断连, Redis 断连 | 500 + retry |
| 资源超限 | report 数据 > 100K | 413 + 提示缩小 scope |

### 9.2 错误日志

- 所有错误带 `trace_id` (UUID v4)
- 服务端日志含: trace_id / tenant_id / user_id / report_id / 错误码
- 前端 toast 显示 trace_id, 用户可贴给支持

### 9.3 重试与降级

- API 失败: 客户端 3 次重试 (exponential backoff)
- Redis 不可用: 跳过缓存, 直查 DB (降级)
- Recharts 渲染失败: 显示空状态 + 错误信息

---

## 10. 可观测性

### 10.1 Metrics (Prometheus)

- `report_generation_duration_seconds` (histogram)
- `report_cache_hit_ratio` (gauge)
- `report_query_duration_seconds` (histogram, by chart_type)
- `dashboard_load_duration_seconds` (histogram)
- `report_export_duration_seconds` (histogram, by format)

### 10.2 Logs (Structured JSON)

每条日志含:
- timestamp
- tenant_id
- user_id
- trace_id
- report_id / dashboard_id
- action (query / create / export / ...)
- duration_ms
- result (success / error)
- error_code (if any)

### 10.3 Traces (OpenTelemetry)

- API 请求 → query service → domain-work-item (span chain)
- Recharts 渲染 (前端 span, 浏览器 → CDN)

---

## 11. 风险 & 缓解

| Risk | 影响 | 缓解 |
|---|---|---|
| RISK-CHART-08: 22 图表全上跨 session | 单次超 token 上限 | 分 3 批上线 (P0/P1/P2 per 需求 §11) |
| RISK-CHART-09: Recharts 11 与 Next.js 14 SSR 不兼容 | 客户端 only | 动态 import + 'use client' 边界 |
| RISK-CHART-10: 跨域权限漏洞 | 数据泄露 | 强制 RLS + 应用层二次校验 |
| RISK-CHART-11: subscription 通知风暴 | 邮件噪音 | 24h digest + 降噪 (per REQ-NOTIF-002) |
| RISK-CHART-12: Snapshot 缓存与实时数据漂移 | 决策失误 | 5min TTL + 显式 "data as of" 提示 |
| RISK-CHART-13: Wallboard 公开 URL 泄露 | 跨租户数据 | token 30 天过期 + IP 白名单 (可选) |

---

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 架构 + 模块 + 数据模型 + API + 缓存 + 安全 + 可观测 + 风险 (per 2026-09-02 10:04 JST Ulysses 拍板) | 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira" |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
