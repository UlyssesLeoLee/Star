//! Star Report Engine v0.2 (per docs/requirements/charts-and-reports.md + docs/basic-design/charts-and-reports.md)
//!
//! 22 图表 (per Jira Cloud 报告中心对标, 阶段 1 落地 8 P0 + 6 P1 stub):
//! - P0 (8): C01 Burndown / C02 Burnup / C03 Velocity / C04 SprintReport / C05 CFD / C06 ControlChart / C07 CycleTime / C13 CreatedVsResolved
//! - P1 (6, stub): C08 Throughput / C09 Forecast / C10 TimeTracking / C11 ResolutionTime / C12 SLA / C14 IssueTypeDist
//! - P2 (8, stub): C15 PriorityDist / C16 AssigneeWorkload / C17 ComponentWorkload / C18 VersionWorkload / C19 ReleaseBurndown / C20 TimeInStatus / C21 Heatmap / C22 RecentlyCreated
//!
//! 阶段 1 重点: C01 Burndown 完整实装 (SQL + Port + 缓存 + 错误 + 测试), 其它 13 路由 stub

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// 模块
/// application 层 (ports + cache trait)
pub mod application;
/// domain 层 (22 图表实现)
pub mod domain;
/// infrastructure 层 (in-memory cache + port stubs)
pub mod infrastructure;

// 重新导出
pub use application::{cache::*, ports::*};
pub use domain::c01_burndown::*;
pub use infrastructure::{in_memory_cache::*, port_stubs::*};

// =====================================================================
// 1. value_object - ReportType 22 图表枚举 (阶段 1: 8 真实 + 14 stub)
// =====================================================================

/// 22 图表类型 (per docs/specs/domain-report-spec.md v1.0 §2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    // P0 (8 真实)
    /// Sprint Burndown 图表
    Burndown, // C01
    /// Sprint Burnup 图表
    Burnup, // C02
    /// 团队速度图表
    Velocity, // C03
    /// Sprint 报告
    SprintReport, // C04
    /// 累积流图 (CFD)
    Cfd, // C05
    /// 周期时间控制图
    ControlChart, // C06
    /// 周期时间报告
    CycleTime, // C07
    /// 新建 vs 已解决
    CreatedVsResolved, // C13
    // P1 (6 stub - 阶段 1 返回 placeholder)
    /// 吞吐量报告
    Throughput, // C08
    /// 预测图表
    Forecast, // C09
    /// 工时跟踪报告
    TimeTracking, // C10
    /// 解决时间报告
    ResolutionTime, // C11
    /// SLA 合规
    Sla, // C12
    /// Issue 类型分布
    IssueTypeDist, // C14
    // P2 (8 stub)
    /// 优先级分布
    PriorityDist, // C15
    /// 经办人工作量
    AssigneeWorkload, // C16
    /// 组件工作量
    ComponentWorkload, // C17
    /// 版本工作量
    VersionWorkload, // C18
    /// 发布燃尽图
    ReleaseBurndown, // C19
    /// 状态停留时间
    TimeInStatus, // C20
    /// 活动热力图
    Heatmap, // C21
    /// 最近创建
    RecentlyCreated, // C22
}

impl ReportType {
    /// 全部 22 图表
    pub fn all() -> &'static [ReportType] {
        &[
            Self::Burndown,
            Self::Burnup,
            Self::Velocity,
            Self::SprintReport,
            Self::Cfd,
            Self::ControlChart,
            Self::CycleTime,
            Self::CreatedVsResolved,
            Self::Throughput,
            Self::Forecast,
            Self::TimeTracking,
            Self::ResolutionTime,
            Self::Sla,
            Self::IssueTypeDist,
            Self::PriorityDist,
            Self::AssigneeWorkload,
            Self::ComponentWorkload,
            Self::VersionWorkload,
            Self::ReleaseBurndown,
            Self::TimeInStatus,
            Self::Heatmap,
            Self::RecentlyCreated,
        ]
    }

    /// P0 批 8 图表 (阶段 1 真实实现)
    pub fn p0_batch() -> &'static [ReportType] {
        &[
            Self::Burndown,
            Self::Burnup,
            Self::Velocity,
            Self::SprintReport,
            Self::Cfd,
            Self::ControlChart,
            Self::CycleTime,
            Self::CreatedVsResolved,
        ]
    }

    /// 是否 P0 真实实现
    pub fn is_p0(&self) -> bool {
        Self::p0_batch().contains(self)
    }

    /// P2 批 7 图表 (C16-C22, 阶段 1 stub 走 NotImplemented, per OPT-NEXT-06-code-stub §3.3 8 P2 图表)
    pub fn p2_batch() -> &'static [ReportType] {
        &[
            Self::AssigneeWorkload,
            Self::ComponentWorkload,
            Self::VersionWorkload,
            Self::ReleaseBurndown,
            Self::TimeInStatus,
            Self::Heatmap,
            Self::RecentlyCreated,
        ]
    }

    /// 是否 P2 阶段 (走 NotImplemented 错误结构化)
    pub fn is_p2(&self) -> bool {
        Self::p2_batch().contains(self)
    }

    /// P1 批 6 图表 (C08-C12, C14, 阶段 1 stub 走 generate_stub mock)
    pub fn p1_batch() -> &'static [ReportType] {
        &[
            Self::Throughput,
            Self::Forecast,
            Self::TimeTracking,
            Self::ResolutionTime,
            Self::Sla,
            Self::IssueTypeDist,
        ]
    }

    /// 是否 P1 阶段 (走 generate_stub fallback)
    pub fn is_p1(&self) -> bool {
        Self::p1_batch().contains(self)
    }

    /// 图表 ID (e.g. "C01")
    pub fn chart_id(&self) -> &'static str {
        match self {
            Self::Burndown => "C01",
            Self::Burnup => "C02",
            Self::Velocity => "C03",
            Self::SprintReport => "C04",
            Self::Cfd => "C05",
            Self::ControlChart => "C06",
            Self::CycleTime => "C07",
            Self::CreatedVsResolved => "C13",
            Self::Throughput => "C08",
            Self::Forecast => "C09",
            Self::TimeTracking => "C10",
            Self::ResolutionTime => "C11",
            Self::Sla => "C12",
            Self::IssueTypeDist => "C14",
            Self::PriorityDist => "C15",
            Self::AssigneeWorkload => "C16",
            Self::ComponentWorkload => "C17",
            Self::VersionWorkload => "C18",
            Self::ReleaseBurndown => "C19",
            Self::TimeInStatus => "C20",
            Self::Heatmap => "C21",
            Self::RecentlyCreated => "C22",
        }
    }

    /// 图表展示名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Burndown => "Sprint Burndown",
            Self::Burnup => "Sprint Burnup",
            Self::Velocity => "Team Velocity",
            Self::SprintReport => "Sprint Report",
            Self::Cfd => "Cumulative Flow Diagram",
            Self::ControlChart => "Cycle Time Control Chart",
            Self::CycleTime => "Cycle Time Report",
            Self::CreatedVsResolved => "Created vs Resolved",
            Self::Throughput => "Throughput Report",
            Self::Forecast => "Forecast Chart",
            Self::TimeTracking => "Time Tracking Report",
            Self::ResolutionTime => "Resolution Time Report",
            Self::Sla => "SLA Compliance",
            Self::IssueTypeDist => "Issue Type Distribution",
            Self::PriorityDist => "Priority Distribution",
            Self::AssigneeWorkload => "Assignee Workload",
            Self::ComponentWorkload => "Component Workload",
            Self::VersionWorkload => "Version Workload",
            Self::ReleaseBurndown => "Release Burndown",
            Self::TimeInStatus => "Time in Status",
            Self::Heatmap => "Activity Heatmap",
            Self::RecentlyCreated => "Recently Created",
        }
    }
}

/// 报告作用域 (5 scope, per docs/requirements §1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportFilter {
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid, // RLS 必携
    /// 项目 ID (可选)
    pub project_id: Option<Uuid>,
    /// Sprint ID (可选)
    pub sprint_id: Option<Uuid>,
    /// 版本 ID (可选)
    pub version_id: Option<Uuid>,
    /// Issue Filter ID (可选, S5)
    pub filter_id: Option<Uuid>, // S5 (Issue Filter)
    /// 时间范围 (可选)
    pub time_range: Option<TimeRange>,
}

impl Default for ReportFilter {
    fn default() -> Self {
        Self {
            tenant_id: Uuid::nil(),
            project_id: None,
            sprint_id: None,
            version_id: None,
            filter_id: None,
            time_range: None,
        }
    }
}

/// 时间范围
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// 起始时间
    pub from: chrono::DateTime<chrono::Utc>,
    /// 结束时间
    pub to: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 2. entity
// =====================================================================

/// 报告定义 (聚合根, per docs/basic-design §3.1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// 报告 ID
    pub id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 图表类型
    pub report_type: ReportType,
    /// 标题
    pub title: String,
    /// 作用域过滤条件
    pub filter: ReportFilter,
    /// 图表配置 (ChartConfig, 22 图表共用 schema)
    pub config: serde_json::Value, // ChartConfig, 22 图表共用 schema
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 版本号
    pub version: i32,
}

/// 报告数据点 (各图表 schema 不同, 阶段 1 走 serde_json::Value 灵活)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportPoint {
    /// 展示标签
    pub label: String,
    /// 数值
    pub value: f64,
    /// 附加信息 (各图表自定义)
    pub extra: serde_json::Value,
}

/// 报告结果 (ReportSnapshot, per docs/basic-design §3.3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    /// 报告 ID
    pub report_id: Uuid,
    /// 图表类型
    pub report_type: ReportType,
    /// 数据点
    pub points: Vec<ReportPoint>,
    /// 图表 data schema (TS 同构, per docs/design/charts/c01-burndown.md §3)
    pub data: serde_json::Value, // 22 图表 data schema (TS 同构, per docs/design/charts/c01-burndown.md §3)
    /// 摘要
    pub summary: ReportSummary,
    /// 生成时间
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// 缓存 key (5min TTL)
    pub cache_key: String, // 5min TTL
}

/// 报告摘要
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSummary {
    /// 总计
    pub total: f64,
    /// 趋势
    pub trend: Trend,
    /// 异常列表
    pub anomalies: Vec<String>,
    /// 图表-specific 摘要 (e.g. C01 remaining_sp / on_track)
    pub meta: serde_json::Value, // 图表-specific 摘要 (e.g. C01 remaining_sp / on_track)
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    /// 上升
    Up,
    /// 下降
    Down,
    /// 持平
    Flat,
}

// =====================================================================
// 3. error (per docs/specs/domain-report-spec.md v1.0 §4.6)
// =====================================================================

/// domain-report 错误类型
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReportError {
    /// 报告未找到
    #[error("report not found: {0}")]
    NotFound(Uuid),
    /// 权限不足
    #[error("permission denied for actor {actor} action {action}")]
    PermissionDenied {
        /// 操作者 ID
        actor: Uuid,
        /// 被拒绝的操作
        action: String,
    },
    /// 校验失败
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// 过滤条件无效
    #[error("filter invalid: {0}")]
    FilterInvalid(String),
    /// 作用域不匹配
    #[error("scope mismatch: expected {expected}, got {got}")]
    ScopeMismatch {
        /// 期望的作用域
        expected: String,
        /// 实际的作用域
        got: String,
    },
    /// 数据量过大
    #[error("data too large: {points} points, limit {limit}")]
    DataTooLarge {
        /// 实际点数
        points: u32,
        /// 限制点数
        limit: u32,
    },
    /// 数据源错误
    #[error("data source error: {0}")]
    DataSource(String),
    /// 计算错误
    #[error("computation error: {0}")]
    Computation(String),
    /// 导出错误
    #[error("export error: {0}")]
    Export(String),
    /// 缓存不可用
    #[error("cache unavailable: {0}")]
    CacheUnavailable(String),
    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),
    /// **P2 阶段未实装标记** (per OPT-NEXT-06 47 stub 实装, per `docs/briefs/next-session/OPT-NEXT-06-code-stub.md` §3.3)
    ///
    /// per OPT-WORKER-01 模式: `Error::NotImplemented { feature, suggestion }`
    /// 14 P1/P2 图表 (C08-C12, C14-C22) + 4 port stub 走此错误结构化.
    /// P2 阶段 worker 子代理实装时按 feature 名替换为真实数据接入 (V2 接 domain-work-item 等).
    #[error("not implemented: feature={feature}; suggestion={suggestion}")]
    NotImplemented {
        /// 未实装的图表/功能名 (e.g. `"ReportType::C16_AssigneeWorkload"`)
        feature: String,
        /// 可执行的修复建议 (e.g. `"Wait for V2 port to domain-work-item per守门 #4"`)
        suggestion: String,
    },
}

impl ReportError {
    /// 构造 NotImplemented 错误的便捷方法 (per OPT-WORKER-01 模式, 跟 `BatchError::not_implemented` 对齐)
    pub fn not_implemented(feature: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
            suggestion: suggestion.into(),
        }
    }

    /// HTTP 状态码 (per 守门 #12 P2 阶段 R-007 API 边界)
    pub fn http_status(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::PermissionDenied { .. } => 403,
            Self::ValidationFailed(_) | Self::FilterInvalid(_) | Self::ScopeMismatch { .. } => 422,
            Self::DataTooLarge { .. } => 413,
            Self::DataSource(_) | Self::Computation(_) | Self::Export(_) | Self::Internal(_) => 500,
            Self::CacheUnavailable(_) => 503,
            Self::NotImplemented { .. } => 501,
        }
    }

    /// 错误码字符串 (per star-context 错误模型)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "REPORT_NOT_FOUND",
            Self::PermissionDenied { .. } => "REPORT_PERMISSION_DENIED",
            Self::ValidationFailed(_) => "REPORT_VALIDATION_FAILED",
            Self::FilterInvalid(_) => "REPORT_FILTER_INVALID",
            Self::ScopeMismatch { .. } => "REPORT_SCOPE_MISMATCH",
            Self::DataTooLarge { .. } => "REPORT_DATA_TOO_LARGE",
            Self::DataSource(_) => "REPORT_DATA_SOURCE_ERROR",
            Self::Computation(_) => "REPORT_COMPUTATION_ERROR",
            Self::Export(_) => "REPORT_EXPORT_ERROR",
            Self::CacheUnavailable(_) => "REPORT_CACHE_UNAVAILABLE",
            Self::Internal(_) => "REPORT_INTERNAL",
            Self::NotImplemented { .. } => "REPORT_NOT_IMPLEMENTED",
        }
    }
}

/// 报告服务 (聚合 Cache + 4 个 Port, 生成/导出报告)
pub struct ReportService {
    cache: Box<dyn Cache>,
    work_item_port: Box<dyn WorkItemQueryPort>,
    sprint_port: Box<dyn SprintQueryPort>,
    #[allow(dead_code)] // reserved for V2 P2+ chart wiring (per P1-5 cleanup, awaiting caller)
    user_port: Box<dyn UserQueryPort>,
    #[allow(dead_code)] // reserved for V2 P2+ chart wiring (per P1-5 cleanup, awaiting caller)
    permission_port: Box<dyn PermissionPort>,
}

impl ReportService {
    /// 构造 ReportService
    pub fn new(
        cache: Box<dyn Cache>,
        work_item_port: Box<dyn WorkItemQueryPort>,
        sprint_port: Box<dyn SprintQueryPort>,
        user_port: Box<dyn UserQueryPort>,
        permission_port: Box<dyn PermissionPort>,
    ) -> Self {
        Self {
            cache,
            work_item_port,
            sprint_port,
            user_port,
            permission_port,
        }
    }

    /// 生成报告 (per docs/specs/domain-report-spec.md §4.2 get_data)
    pub async fn generate(
        &self,
        report_type: ReportType,
        filter: ReportFilter,
    ) -> Result<ReportResult, ReportError> {
        // 稳定 cache key: 基于 filter 而非 report_id (report_id 每次新生成, 缓存会 miss)
        let scope_id = filter
            .sprint_id
            .or(filter.version_id)
            .or(filter.project_id)
            .or(filter.filter_id)
            .unwrap_or(filter.tenant_id);
        let cache_key = format!(
            "report:{}:{}:{}",
            filter.tenant_id,
            scope_id,
            report_type.chart_id()
        );

        // 5min TTL 缓存检查
        if let Ok(Some(cached_val)) = self.cache.get_json(&cache_key).await {
            if let Ok(cached) = serde_json::from_value::<ReportResult>(cached_val) {
                tracing::debug!("cache hit: {}", cache_key);
                return Ok(cached);
            }
        }

        // 5min TTL miss, 走真实计算
        let report_id = Uuid::new_v4();
        let result = if report_type.is_p0() {
            self.generate_p0(report_type, &filter, report_id, cache_key.clone())
                .await?
        } else if report_type.is_p2() {
            // P2 阶段 C16-C22 (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)
            // 走真实 module → 返 NotImplemented 错误
            self.generate_p0(report_type, &filter, report_id, cache_key.clone())
                .await?
        } else {
            // P1 阶段 C08-C12, C14-C15 走 generate_stub (mock 数据, 阶段 1 fallback)
            self.generate_stub(report_type, &filter, report_id, cache_key.clone())
                .await?
        };

        // 写缓存
        if let Ok(val) = serde_json::to_value(&result) {
            let _ = self.cache.set_json(&cache_key, &val, 300).await; // 5min TTL
        }
        Ok(result)
    }

    /// P0 真实生成
    async fn generate_p0(
        &self,
        report_type: ReportType,
        filter: &ReportFilter,
        report_id: Uuid,
        _cache_key: String,
    ) -> Result<ReportResult, ReportError> {
        match report_type {
            ReportType::Burndown => {
                domain::c01_burndown::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                )
                .await
            }
            ReportType::Burnup => {
                domain::c02_burnup::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                )
                .await
            }
            ReportType::Velocity => {
                domain::c03_velocity::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                )
                .await
            }
            ReportType::SprintReport => {
                domain::c04_sprint_report::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                )
                .await
            }
            ReportType::Cfd => {
                domain::c05_cfd::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::ControlChart => {
                domain::c06_control_chart::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::CycleTime => {
                domain::c07_cycle_time::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::CreatedVsResolved => {
                domain::c13_created_vs_resolved::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::Throughput => {
                domain::c08_throughput::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::Forecast => {
                domain::c09_forecast::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::TimeTracking => {
                domain::c10_time_tracking::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::ResolutionTime => {
                domain::c11_resolution_time::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::Sla => {
                domain::c12_sla_compliance::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::IssueTypeDist => {
                domain::c14_issue_type_dist::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::PriorityDist => {
                domain::c15_priority_dist::generate(&*self.work_item_port, filter, report_id).await
            }
            // P2 阶段 C16-C22 (per OPT-NEXT-06-code-stub §3.3 8 P2 图表)
            // V2 接 domain-work-item 等真实数据, 当前返回 NotImplemented
            ReportType::AssigneeWorkload => {
                domain::c16_assignee_workload::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::ComponentWorkload => {
                domain::c17_component_workload::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::VersionWorkload => {
                domain::c18_version_workload::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::ReleaseBurndown => {
                domain::c19_release_burndown::generate(&*self.work_item_port, filter, report_id)
                    .await
            }
            ReportType::TimeInStatus => {
                domain::c20_time_in_status::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::Heatmap => {
                domain::c21_heatmap::generate(&*self.work_item_port, filter, report_id).await
            }
            ReportType::RecentlyCreated => {
                domain::c22_recently_created::generate(&*self.work_item_port, filter, report_id)
                    .await
            } // 暂未实装的子图走 stub (例如 P3 阶段的扩展图表) - all 22 variants covered, no _ needed
        }
    }

    /// Stub 生成 (P1/P2 + 暂未实装的 P0 子图)
    async fn generate_stub(
        &self,
        report_type: ReportType,
        _filter: &ReportFilter,
        report_id: Uuid,
        cache_key: String,
    ) -> Result<ReportResult, ReportError> {
        let points = (0..10)
            .map(|i| ReportPoint {
                label: format!("Point {}", i + 1),
                value: 100.0 - (i as f64) * 10.0,
                extra: serde_json::json!({"stub": true, "chart_id": report_type.chart_id()}),
            })
            .collect();
        Ok(ReportResult {
            report_id,
            report_type,
            points,
            data: serde_json::json!({"stub": true}),
            summary: ReportSummary {
                total: 100.0,
                trend: Trend::Flat,
                anomalies: vec![],
                meta: serde_json::json!({"stub": true}),
            },
            generated_at: chrono::Utc::now(),
            cache_key,
        })
    }

    /// 导出 JSON
    pub fn export_json(&self, result: &ReportResult) -> Result<String, ReportError> {
        serde_json::to_string_pretty(result).map_err(|e| ReportError::Export(e.to_string()))
    }

    /// 导出 CSV
    pub fn export_csv(&self, result: &ReportResult) -> Result<String, ReportError> {
        let mut out = String::from(
            "label,value
",
        );
        for p in &result.points {
            out.push_str(&format!(
                "{},{}
",
                p.label, p.value
            ));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_svc() -> ReportService {
        ReportService::new(
            Box::new(InMemoryCache::new()),
            Box::new(InMemoryWorkItemPort::new()),
            Box::new(InMemorySprintPort::new()),
            Box::new(InMemoryUserPort::new()),
            Box::new(InMemoryPermissionPort::new()),
        )
    }

    #[tokio::test]
    async fn test_report_type_all_22() {
        assert_eq!(ReportType::all().len(), 22);
    }

    #[tokio::test]
    async fn test_p0_batch_8() {
        assert_eq!(ReportType::p0_batch().len(), 8);
    }

    #[tokio::test]
    async fn test_chart_id_mapping() {
        assert_eq!(ReportType::Burndown.chart_id(), "C01");
        assert_eq!(ReportType::Cfd.chart_id(), "C05");
        assert_eq!(ReportType::RecentlyCreated.chart_id(), "C22");
    }

    #[tokio::test]
    async fn test_generate_stub_for_p2() {
        let svc = make_svc();
        // C21 Heatmap 现在走真模块 (返回 NotImplemented), 不再走 generate_stub
        // 测试 c19 ReleaseBurndown 走 generate_stub fallback (per lib.rs:586)
        // 注意: 当前所有 22 图表都已注册到 generate_p0, 没有 fallback 路径
        // 验证 NotImplemented 错误
        let r = svc
            .generate(ReportType::Heatmap, ReportFilter::default())
            .await;
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert_eq!(err.http_status(), 501);
        assert_eq!(err.code(), "REPORT_NOT_IMPLEMENTED");
    }

    /// per OPT-NEXT-06-code-stub §3.3 4 port stub: 验证 V2 迁移标记
    /// 4 port 仍能 in-memory 正常响应 (单测) + V2 迁移入口
    #[tokio::test]
    async fn test_4_port_stubs_smoke() {
        use crate::infrastructure::port_stubs::{
            InMemoryPermissionPort, InMemorySprintPort, InMemoryUserPort, InMemoryWorkItemPort,
        };
        // 4 port 都能构造
        let _ = InMemoryWorkItemPort::new();
        let _ = InMemorySprintPort::new();
        let _ = InMemoryUserPort::new();
        let _ = InMemoryPermissionPort::new();
    }

    /// 14 chart P1/P2 + 4 port stub → 18 item 走 NotImplemented / V2 迁移路径
    /// (c08-c15 走真 generate mock, c16-c22 走 NotImplemented, 4 port 走 V2 迁移)
    #[tokio::test]
    async fn test_p2_charts_return_not_implemented() {
        let svc = make_svc();
        // 7 P2 阶段图表 (C16-C22) 应返回 NotImplemented
        let p2_charts = [
            ReportType::AssigneeWorkload,
            ReportType::ComponentWorkload,
            ReportType::VersionWorkload,
            ReportType::ReleaseBurndown,
            ReportType::TimeInStatus,
            ReportType::Heatmap,
            ReportType::RecentlyCreated,
        ];
        for chart in p2_charts {
            let r = svc.generate(chart, ReportFilter::default()).await;
            assert!(r.is_err(), "P2 chart {chart:?} should return error");
            let err = r.unwrap_err();
            assert_eq!(err.http_status(), 501, "P2 chart {chart:?} should be 501");
            assert_eq!(err.code(), "REPORT_NOT_IMPLEMENTED");
        }
    }
}
