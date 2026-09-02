# domain-report 实施 spec v1.0

> **状态**: v1.0 (2026-09-02)
> **触发**:
> - v0.1 (2026-09-01): GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> - v1.0 (2026-09-02): 升 v1.0 扩写, 对标 Jira Cloud 报告中心 22 图表 + 5 scope (per 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira, 各个图表设计要完善")
> **下游交付**:
> - 需求基线: [docs/requirements/charts-and-reports.md v1.0](../../requirements/charts-and-reports.md)
> - 基本设计: [docs/basic-design/charts-and-reports.md v1.0](../../basic-design/charts-and-reports.md)
> - 详细设计: [docs/design/charts/](../../design/charts/) (22 份)
> - 实现路径: `crates/domain-report/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-report 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-report` 负责 **Star Report Engine**,提供 22 类核心图表 + 5 scope 数据聚合 + 订阅 + 调度。

**属于本 crate 的** (per v1.0 升版, 数量从 10 → 22):
- **22 图表类型** (per 需求 §2):
  - Agile & Sprint: C01 Burndown / C02 Burnup / C03 Velocity / C04 Sprint Report / C05 CFD
  - Cycle & Forecast: C06 Control Chart / C07 Cycle Time / C08 Throughput / C09 Forecast
  - Time & SLA: C10 Time Tracking / C11 Resolution Time / C12 SLA Compliance
  - Distribution: C13 Created vs Resolved / C14 Issue Type / C15 Priority / C16 Assignee Workload / C17 Component Workload
  - Version: C18 Version Workload / C19 Release Burndown / C20 Time in Status
  - Custom: C21 Heatmap / C22 Recently Created
- 5 scope 聚合 (Project / Project Hierarchy / Sprint / Version / Issue)
- 订阅 + 邮件 + 降噪 (per REQ-NOTIF-002)
- 调度 (cron) + worker projection role (per basic-design v0.16 §4.12.2)
- 导出 (CSV / XLSX / PNG / PDF)
- 过滤表达式解析 (JQL 风格)
- 缓存 (5min TTL Redis)
- 审计 (per basic-design v0.16 §3.5)

**不属于本 crate 的**:
- WorkItem 事实 (从 `domain-work-item` Projection 读取, 不持有 SoR)
- Sprint / Version 事实 (从 `domain-planning`)
- User / Group 事实 (从 `domain-identity`)
- 权限定义 (从 `domain-permission` 校验)
- 通知通道 (从 `domain-notification` 调用)
- Dashboard 容器 (从 `domain-dashboard` 嵌入)

---

## 2. 关键实体 (v1.0 完整版)

```rust
// 完整定义见 [basic-design §3](../../basic-design/charts-and-reports.md#3-数据模型)

pub struct ReportDefinition {           // 聚合根
    pub report_id: ReportId,
    pub tenant_id: TenantId,
    pub project_id: Option<ProjectId>,
    pub scope: ReportScope,             // S1-S5
    pub chart_type: ChartType,          // C01-C22
    pub title: String,
    pub description: Option<String>,
    pub filter_id: Option<FilterId>,    // S5 必须
    pub config: ChartConfig,            // 统一配置 schema
    pub schedule: Option<ReportSchedule>,
    pub subscriptions: Vec<ReportSubscription>,
    pub owner_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

pub enum ReportScope {
    Project(ProjectId),                 // S1
    ProjectHierarchy(WorkspaceId),      // S2
    Sprint(SprintId),                   // S3
    Version(VersionId),                 // S4
    IssueFilter(FilterId),              // S5
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

pub struct ReportSnapshot {             // 投影
    pub snapshot_id: SnapshotId,
    pub report_id: ReportId,
    pub generated_at: DateTime<Utc>,
    pub data: serde_json::Value,        // 22 图表 data schema
    pub data_source_refs: Vec<DataSourceRef>,
    pub cache_key: String,
    pub ttl: Duration,                  // 5min
    pub render_hints: RenderHints,
}

pub struct ReportSchedule {             // cron
    pub cron_expression: String,        // "0 9 * * *" daily 9am
    pub next_run_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub timezone: String,               // IANA
}

pub struct ReportSubscription {
    pub subscription_id: SubscriptionId,
    pub user_id: UserId,
    pub cadence: Cadence,               // realtime / hourly / daily / weekly
    pub channel: Channel,               // email / in_app
    pub enabled: bool,
    pub last_notified_at: Option<DateTime<Utc>>,
}

pub struct ReportFilter {               // JQL 风格
    pub filter_id: FilterId,
    pub jql_expression: String,
    pub parsed: FilterAst,              // 解析后 AST
    pub owner_id: UserId,
    pub shared_with: Vec<ShareGrant>,
}
```

---

## 3. 关键不变量 (v1.0 完整版, 含 22 图表)

| ID | 不变量 | 适用 |
|---|---|---|
| INV-REPORT-01 | Report 是 Projection, 不得持有 SoR 业务事实 | 全部 |
| INV-REPORT-02 | Report 数据走 Redis 5min TTL 缓存, 不实时拉源 | 全部 |
| INV-REPORT-03 | Report 订阅触发走 worker projection role (per basic-design v0.16 §4.12.2) | 全部 |
| INV-REPORT-04 | 跨 scope 图表 (S2) 必须显式聚合, 不允许散点拼接 | S2 |
| INV-REPORT-05 | 22 图表 type 必须在 ChartType 枚举内, 不可动态添加 | 全部 |
| INV-REPORT-06 | ChartConfig 必须通过 schema validation (per §4.2) | 全部 |
| INV-REPORT-07 | filter_id (S5) 必须存在, 否则拒绝创建 | S5 |
| INV-REPORT-08 | 跨租户 Report 必须 403 (RLS 必携) | 全部 |
| INV-REPORT-09 | 数据点 > 10K 触发自动采样, 提示用户细化 | 全部 |
| INV-REPORT-10 | 数据点 > 100K 拒绝生成 | 全部 |
| INV-REPORT-11 | export 必须带 watermark (user_id) | export |
| INV-REPORT-12 | subscription 24h 内同类合并 (per REQ-NOTIF-002 降噪) | 订阅 |
| INV-REPORT-13 | Burndown/CFD X 轴不能晚于 Sprint.end_date + 30d | C01/C02/C05 |
| INV-REPORT-14 | Control Chart 至少 10 个完成 issue 才画控制线 | C06 |
| INV-REPORT-15 | Forecast 至少 3 个已完成 Sprint 才预测 | C09 |
| INV-REPORT-16 | SLA Compliance 命中判定基于 SLA 定义 (per sla_definition 表) | C12 |
| INV-REPORT-17 | Heatmap 强制 7×24 矩阵聚合, 不允许自定义桶 | C21 |
| INV-REPORT-18 | 导出异步任务 24h 后删除 (per W 表 retention) | export |
| INV-REPORT-19 | 所有 Report 必须有 owner, 不可匿名 | 全部 |
| INV-REPORT-20 | Report title 长度 1-200, description 长度 0-2000 | 全部 |

---

## 4. 接口契约 (v1.0 完整版)

### 4.1 Command Port

```rust
#[async_trait]
pub trait ReportDefinitionCommandPort: Send + Sync {
    async fn create(&self, cmd: CreateReportCmd, actor: &ActorContext) -> Result<ReportDefinition, ReportError>;
    async fn update(&self, cmd: UpdateReportCmd, actor: &ActorContext) -> Result<ReportDefinition, ReportError>;
    async fn delete(&self, report_id: ReportId, actor: &ActorContext) -> Result<(), ReportError>;
    async fn enable(&self, report_id: ReportId, actor: &ActorContext) -> Result<(), ReportError>;
    async fn disable(&self, report_id: ReportId, actor: &ActorContext) -> Result<(), ReportError>;
    async fn set_schedule(&self, report_id: ReportId, schedule: Option<ReportSchedule>, actor: &ActorContext) -> Result<(), ReportError>;
    async fn set_subscription(&self, report_id: ReportId, sub: ReportSubscriptionCmd, actor: &ActorContext) -> Result<ReportSubscription, ReportError>;
    async fn remove_subscription(&self, report_id: ReportId, sub_id: SubscriptionId, actor: &ActorContext) -> Result<(), ReportError>;
    async fn force_refresh(&self, report_id: ReportId, actor: &ActorContext) -> Result<SnapshotId, ReportError>;
    async fn request_export(&self, report_id: ReportId, format: ExportFormat, actor: &ActorContext) -> Result<ExportTaskId, ReportError>;
}
```

### 4.2 Query Port

```rust
#[async_trait]
pub trait ReportQueryPort: Send + Sync {
    async fn get(&self, report_id: ReportId, actor: &ActorContext) -> Result<ReportDefinition, ReportError>;
    async fn list_by_project(&self, project_id: ProjectId, actor: &ActorContext) -> Result<Vec<ReportDefinition>, ReportError>;
    async fn list_by_owner(&self, owner_id: UserId, actor: &ActorContext) -> Result<Vec<ReportDefinition>, ReportError>;
    async fn list_shared_with_me(&self, actor: &ActorContext) -> Result<Vec<ReportDefinition>, ReportError>;
    async fn get_data(&self, report_id: ReportId, query: ReportDataQuery, actor: &ActorContext) -> Result<ReportSnapshot, ReportError>;
    async fn get_latest_snapshot(&self, report_id: ReportId, actor: &ActorContext) -> Result<Option<ReportSnapshot>, ReportError>;
}

pub struct ReportDataQuery {
    pub time_range: Option<TimeRange>,
    pub granularity: Option<TimeGranularity>,
    pub config_override: Option<ChartConfig>,
}
```

### 4.3 Schedule Port

```rust
#[async_trait]
pub trait ReportSchedulePort: Send + Sync {
    async fn list_pending_runs(&self, now: DateTime<Utc>) -> Result<Vec<(ReportId, ReportSchedule)>, ReportError>;
    async fn mark_run(&self, report_id: ReportId, run_at: DateTime<Utc>, next_run_at: DateTime<Utc>) -> Result<(), ReportError>;
    async fn validate_cron(&self, cron: &str) -> Result<(), ReportError>;
}
```

### 4.4 Export Port

```rust
#[async_trait]
pub trait ReportExportPort: Send + Sync {
    async fn export_csv(&self, report_id: ReportId, snapshot: &ReportSnapshot, actor: &ActorContext) -> Result<ExportTaskId, ReportError>;
    async fn export_xlsx(&self, report_id: ReportId, snapshot: &ReportSnapshot, actor: &ActorContext) -> Result<ExportTaskId, ReportError>;
    async fn export_png(&self, report_id: ReportId, snapshot: &ReportSnapshot, actor: &ActorContext) -> Result<ExportTaskId, ReportError>;
    async fn export_pdf(&self, report_id: ReportId, snapshot: &ReportSnapshot, actor: &ActorContext) -> Result<ExportTaskId, ReportError>;
    async fn get_export_status(&self, task_id: ExportTaskId, actor: &ActorContext) -> Result<ExportStatus, ReportError>;
}
```

### 4.5 ChartType 行为契约 (22 图表 type → Renderer 注册表)

```rust
// 22 图表注册表, 编译期穷举检查
pub static CHART_RENDERERS: phf::Map<&'static str, ChartRenderer> = phf::map! {
    "C01_BURNDOWN" => ChartRenderer { ... },
    "C02_BURNUP" => ChartRenderer { ... },
    // ... 22 个
    "C22_RECENTLY_CREATED" => ChartRenderer { ... },
};

pub struct ChartRenderer {
    pub chart_type: ChartType,
    pub data_schema: ChartDataSchemaRef,    // 22 图表 schema
    pub query_handler: fn(ReportQueryCtx) -> BoxFuture<Result<serde_json::Value>>,
    pub config_schema: ChartConfigSchemaRef,  // JSON schema
    pub frontend_component: &'static str,   // 客户端组件名
}
```

### 4.6 错误模型

```rust
pub enum ReportError {
    NotFound(ReportId),
    PermissionDenied(actor_id, action),
    ValidationFailed(Vec<FieldError>),
    FilterInvalid(String),
    ScopeMismatch { expected: ReportScope, got: String },
    DataTooLarge { points: u32, limit: u32 },
    ExportFailed(String),
    CacheUnavailable,
    Internal(String),
}
```

---

## 5. 跨 domain 接触面 (v1.0 完整版)

per [basic-design v0.16 §3.1 解耦机制 8 种](../../basic-design.md) + [basic-design/charts-and-reports v1.0 §6](../../basic-design/charts-and-reports.md#6-跨域接触面-per-basic-design-v016-31) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md)。

| 接触 | 协作方式 | Port | 实现 crate |
|---|---|---|---|
| report 读 work-item | Customer-Supplier (OHS) | `WorkItemQueryPort` | domain-work-item |
| report 读 sprint | Customer-Supplier (OHS) | `SprintQueryPort` | domain-planning |
| report 读 version | Customer-Supplier (OHS) | `VersionQueryPort` | domain-planning |
| report 读 user | Customer-Supplier (OHS) | `UserQueryPort` | domain-identity |
| report 校验权限 | Customer-Supplier (OHS) | `PermissionPort` | domain-permission |
| report 写 audit | Conformist | `AuditRecorderPort` | domain-audit |
| report 触发通知 | Separate Ways (异步) | `NotificationPort` | domain-notification |
| dashboard 嵌入 report | Shared Kernel | `ReportDefinition` (同构) | domain-dashboard |
| report 跨域 worker | Customer-Supplier | `WorkerProjectionPort` | basic-design v0.16 §4.12.2 |
| report 读 SLA | Customer-Supplier | `SlaQueryPort` | domain-report (本 crate 持有) |

> SLA 定义 (sla_definition) 是 report 自己的 Master 表, 不跨域。

---

## 6. 风险与缓解 (v1.0 完整版)

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-REPORT-01: 大报表性能 | 异步生成 + 缓存 + 分页 + 采样 (per ADR-CHART-002) | INV-REPORT-09/10 | v1.0 §6 |
| RISK-REPORT-02: 数据不一致 (源数据更新中) | snapshot 时间戳 + 增量更新 | INV-REPORT-02 | v1.0 §6 |
| RISK-REPORT-03: 订阅触发噪音 | REQ-NOTIF-002 降噪策略 + digest | INV-REPORT-12 | v1.0 §6 |
| RISK-REPORT-04: 22 图表实施周期长 | 分 3 批 (P0=8 / P1=7 / P2=7) | per 需求 §11 | v1.0 §6 |
| RISK-REPORT-05: Recharts 与 Next.js 14 SSR 不兼容 | 'use client' 边界 + 动态 import | 需求 §9.1 | v1.0 §6 |
| RISK-REPORT-06: 控制线算法误判 | C06 异常点 + z-score 验证 | INV-REPORT-14 | v1.0 §6 |
| RISK-REPORT-07: 跨域权限漏洞 | RLS + 应用层二次校验 | INV-REPORT-08 | v1.0 §6 |
| RISK-REPORT-08: 导出大文件阻塞 | 异步任务 + 24h 过期 (W 表) | INV-REPORT-18 | v1.0 §6 |
| RISK-REPORT-09: Heatmap 自研 SVG 成本 | 限定 7×24 固定矩阵, 简化实现 | INV-REPORT-17 | v1.0 §6 |
| RISK-REPORT-10: 过滤表达式注入 | 严格 AST 解析, 不直接拼接 SQL | §4.5 ChartConfig schema | v1.0 §6 |

---

## 7. 22 图表 × 5 Scope 矩阵 (v1.0 核心交付)

下表定义每图表在每个 scope 下的查询路径(per [basic-design/charts-and-reports v1.0 §4](../../basic-design/charts-and-reports.md#4-数据流)):

| Chart | S1 Project | S2 Hierarchy | S3 Sprint | S4 Version | S5 IssueFilter |
|---|---|---|---|---|---|
| C01 Burndown | (✓ via sprint) | (✓ via sprint) | ✅ 默认 | ❌ | (✓ via sprint) |
| C02 Burnup | (✓) | (✓) | ✅ 默认 | ❌ | (✓) |
| C03 Velocity | ✅ 默认 | (✓) | (parent) | ❌ | ❌ |
| C04 Sprint Report | (✓) | (✓) | ✅ 默认 | ❌ | (✓) |
| C05 CFD | ✅ 默认 | (✓) | (✓) | (✓) | ✅ |
| C06 Control Chart | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C07 Cycle Time | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C08 Throughput | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C09 Forecast | ✅ | ✅ | ✅ 默认 | (✓) | ❌ |
| C10 Time Tracking | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C11 Resolution Time | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C12 SLA Compliance | ✅ 默认 | ✅ | ❌ | ❌ | ❌ |
| C13 Created vs Resolved | ✅ 默认 | ✅ | ✅ | ✅ | ✅ |
| C14 Issue Type Dist | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C15 Priority Dist | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C16 Assignee Workload | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C17 Component Workload | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C18 Version Workload | ✅ | (✓) | (✓) | ✅ 默认 | (✓) |
| C19 Release Burndown | (✓) | (✓) | (✓) | ✅ 默认 | (✓) |
| C20 Time in Status | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C21 Heatmap | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |
| C22 Recently Created | ✅ | ✅ | ✅ | ✅ | ✅ 默认 |

✅ = 支持,默认 scope; (✓) = 支持,非默认; ❌ = 不支持

---

## 8. 22 图表实施分批 (per 需求 §11 RISK-CHART-01)

### P0 — 第 1 批(8 图表,核心敏捷分析)

| 顺序 | ID | 名称 | 工期估计 | 风险 |
|---|---|---|---|---|
| 1 | C01 | Burndown | 2d | 低 |
| 2 | C02 | Burnup | 1d | 低 |
| 3 | C03 | Velocity | 1.5d | 低 |
| 4 | C04 | Sprint Report | 1d | 低 |
| 5 | C05 | CFD | 2d | 中 |
| 6 | C06 | Control Chart | 3d | 高(异常检测算法) |
| 7 | C07 | Cycle Time | 1.5d | 低 |
| 8 | C13 | Created vs Resolved | 1d | 低 |

**P0 总计**: ~13d (1 SRE ≈ 1.5 周)

### P1 — 第 2 批(7 图表,时间/分布)

| 顺序 | ID | 名称 | 工期估计 |
|---|---|---|---|
| 9 | C08 | Throughput | 1d |
| 10 | C09 | Forecast | 2d |
| 11 | C10 | Time Tracking | 2d |
| 12 | C11 | Resolution Time | 1.5d |
| 13 | C12 | SLA Compliance | 2d |
| 14 | C14 | Issue Type Dist | 0.5d |
| 15 | C15 | Priority Dist | 0.5d |

**P1 总计**: ~9.5d (~1 周)

### P2 — 第 3 批(7 图表,工作量/版本/自定义)

| 顺序 | ID | 名称 | 工期估计 |
|---|---|---|---|
| 16 | C16 | Assignee Workload | 1.5d |
| 17 | C17 | Component Workload | 1.5d |
| 18 | C18 | Version Workload | 1.5d |
| 19 | C19 | Release Burndown | 1.5d |
| 20 | C20 | Time in Status | 2d |
| 21 | C21 | Heatmap | 3d(自研 SVG) |
| 22 | C22 | Recently Created | 1d |

**P2 总计**: ~12d (~1.5 周)

**3 批总工期**: ~34.5d (≈ 5 SRE·周, per STAR-OLU-001 1 SRE·周 = 1.2M tokens 折算)

---

## 9. 22 图表详细设计索引 (per 需求 §0.2 + 拍板 doc-org=A)

| Chart | 详细设计文件 | 关键数据源 |
|---|---|---|
| C01 | [docs/design/charts/c01-burndown.md](../../design/charts/c01-burndown.md) | Sprint + WorkItem |
| C02 | [docs/design/charts/c02-burnup.md](../../design/charts/c02-burnup.md) | Sprint + WorkItem |
| C03 | [docs/design/charts/c03-velocity.md](../../design/charts/c03-velocity.md) | Sprint (多) |
| C04 | [docs/design/charts/c04-sprint-report.md](../../design/charts/c04-sprint-report.md) | Sprint + WorkItem |
| C05 | [docs/design/charts/c05-cfd.md](../../design/charts/c05-cfd.md) | WorkItem (status) |
| C06 | [docs/design/charts/c06-control-chart.md](../../design/charts/c06-control-chart.md) | WorkItem (cycle_time) |
| C07 | [docs/design/charts/c07-cycle-time.md](../../design/charts/c07-cycle-time.md) | WorkItem (cycle_time) |
| C08 | [docs/design/charts/c08-throughput.md](../../design/charts/c08-throughput.md) | WorkItem (resolved_at) |
| C09 | [docs/design/charts/c09-forecast.md](../../design/charts/c09-forecast.md) | Sprint (history) |
| C10 | [docs/design/charts/c10-time-tracking.md](../../design/charts/c10-time-tracking.md) | WorkItem + WorkLog |
| C11 | [docs/design/charts/c11-resolution-time.md](../../design/charts/c11-resolution-time.md) | WorkItem |
| C12 | [docs/design/charts/c12-sla-compliance.md](../../design/charts/c12-sla-compliance.md) | SLA + WorkItem |
| C13 | [docs/design/charts/c13-created-vs-resolved.md](../../design/charts/c13-created-vs-resolved.md) | WorkItem |
| C14 | [docs/design/charts/c14-issue-type-dist.md](../../design/charts/c14-issue-type-dist.md) | WorkItem |
| C15 | [docs/design/charts/c15-priority-dist.md](../../design/charts/c15-priority-dist.md) | WorkItem |
| C16 | [docs/design/charts/c16-assignee-workload.md](../../design/charts/c16-assignee-workload.md) | WorkItem |
| C17 | [docs/design/charts/c17-component-workload.md](../../design/charts/c17-component-workload.md) | WorkItem |
| C18 | [docs/design/charts/c18-version-workload.md](../../design/charts/c18-version-workload.md) | Version + WorkItem |
| C19 | [docs/design/charts/c19-release-burndown.md](../../design/charts/c19-release-burndown.md) | Version + WorkItem |
| C20 | [docs/design/charts/c20-time-in-status.md](../../design/charts/c20-time-in-status.md) | WorkItemStatusHistory |
| C21 | [docs/design/charts/c21-heatmap.md](../../design/charts/c21-heatmap.md) | WorkItem (created/resolved) |
| C22 | [docs/design/charts/c22-recently-created.md](../../design/charts/c22-recently-created.md) | WorkItem |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板（10 报表） | 2026-09-01 15:03 JST GAP-01 |
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 升 v1.0: 10 → 22 图表（对标 Jira Cloud 报告中心）+ 5 scope + 20 不变量 + 5 Port + 10 风险 + 22 详细设计索引 + 实施分批 P0/P1/P2 + 接触面 v1.0 完整版 + DB DDL W/T/M 三類横展開 | 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira" |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
