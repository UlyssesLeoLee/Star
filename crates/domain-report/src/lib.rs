//! Star Report Engine v0.2 (per docs/requirements/charts-and-reports.md + docs/basic-design/charts-and-reports.md)
//!
//! 22 图表 (per Jira Cloud 报告中心对标, 阶段 1 落地 8 P0 + 6 P1 stub):
//! - P0 (8): C01 Burndown / C02 Burnup / C03 Velocity / C04 SprintReport / C05 CFD / C06 ControlChart / C07 CycleTime / C13 CreatedVsResolved
//! - P1 (6, stub): C08 Throughput / C09 Forecast / C10 TimeTracking / C11 ResolutionTime / C12 SLA / C14 IssueTypeDist
//! - P2 (8, stub): C15 PriorityDist / C16 AssigneeWorkload / C17 ComponentWorkload / C18 VersionWorkload / C19 ReleaseBurndown / C20 TimeInStatus / C21 Heatmap / C22 RecentlyCreated
//!
//! 阶段 1 重点: C01 Burndown 完整实装 (SQL + Port + 缓存 + 错误 + 测试), 其它 13 路由 stub

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// 模块
pub mod application;
pub mod infrastructure;
pub mod domain;

// 重新导出
pub use application::{cache::*, ports::*};
pub use infrastructure::{in_memory_cache::*, port_stubs::*};
pub use domain::c01_burndown::*;

// =====================================================================
// 1. value_object - ReportType 22 图表枚举 (阶段 1: 8 真实 + 14 stub)
// =====================================================================

/// 22 图表类型 (per docs/specs/domain-report-spec.md v1.0 §2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    // P0 (8 真实)
    Burndown,           // C01
    Burnup,             // C02
    Velocity,           // C03
    SprintReport,       // C04
    Cfd,                // C05
    ControlChart,       // C06
    CycleTime,          // C07
    CreatedVsResolved,  // C13
    // P1 (6 stub - 阶段 1 返回 placeholder)
    Throughput,         // C08
    Forecast,           // C09
    TimeTracking,       // C10
    ResolutionTime,     // C11
    Sla,                // C12
    IssueTypeDist,      // C14
    // P2 (8 stub)
    PriorityDist,       // C15
    AssigneeWorkload,   // C16
    ComponentWorkload,  // C17
    VersionWorkload,    // C18
    ReleaseBurndown,    // C19
    TimeInStatus,       // C20
    Heatmap,            // C21
    RecentlyCreated,    // C22
}

impl ReportType {
    /// 全部 22 图表
    pub fn all() -> &'static [ReportType] {
        &[
            Self::Burndown, Self::Burnup, Self::Velocity, Self::SprintReport,
            Self::Cfd, Self::ControlChart, Self::CycleTime, Self::CreatedVsResolved,
            Self::Throughput, Self::Forecast, Self::TimeTracking, Self::ResolutionTime,
            Self::Sla, Self::IssueTypeDist, Self::PriorityDist, Self::AssigneeWorkload,
            Self::ComponentWorkload, Self::VersionWorkload, Self::ReleaseBurndown,
            Self::TimeInStatus, Self::Heatmap, Self::RecentlyCreated,
        ]
    }

    /// P0 批 8 图表 (阶段 1 真实实现)
    pub fn p0_batch() -> &'static [ReportType] {
        &[
            Self::Burndown, Self::Burnup, Self::Velocity, Self::SprintReport,
            Self::Cfd, Self::ControlChart, Self::CycleTime, Self::CreatedVsResolved,
        ]
    }

    /// 是否 P0 真实实现
    pub fn is_p0(&self) -> bool {
        Self::p0_batch().contains(self)
    }

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
    pub tenant_id: Uuid,                          // RLS 必携
    pub project_id: Option<Uuid>,
    pub sprint_id: Option<Uuid>,
    pub version_id: Option<Uuid>,
    pub filter_id: Option<Uuid>,                  // S5 (Issue Filter)
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
}

// =====================================================================
// 2. entity
// =====================================================================

/// 报告定义 (聚合根, per docs/basic-design §3.1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub filter: ReportFilter,
    pub config: serde_json::Value,    // ChartConfig, 22 图表共用 schema
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

/// 报告数据点 (各图表 schema 不同, 阶段 1 走 serde_json::Value 灵活)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportPoint {
    pub label: String,
    pub value: f64,
    pub extra: serde_json::Value,
}

/// 报告结果 (ReportSnapshot, per docs/basic-design §3.3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    pub report_id: Uuid,
    pub report_type: ReportType,
    pub points: Vec<ReportPoint>,
    pub data: serde_json::Value,        // 22 图表 data schema (TS 同构, per docs/design/charts/c01-burndown.md §3)
    pub summary: ReportSummary,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub cache_key: String,              // 5min TTL
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total: f64,
    pub trend: Trend,
    pub anomalies: Vec<String>,
    pub meta: serde_json::Value,        // 图表-specific 摘要 (e.g. C01 remaining_sp / on_track)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    Flat,
}

// =====================================================================
// 3. error (per docs/specs/domain-report-spec.md v1.0 §4.6)
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReportError {
    #[error("report not found: {0}")]
    NotFound(Uuid),
    #[error("permission denied for actor {actor} action {action}")]
    PermissionDenied { actor: Uuid, action: String },
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("filter invalid: {0}")]
    FilterInvalid(String),
    #[error("scope mismatch: expected {expected}, got {got}")]
    ScopeMismatch { expected: String, got: String },
    #[error("data too large: {points} points, limit {limit}")]
    DataTooLarge { points: u32, limit: u32 },
    #[error("data source error: {0}")]
    DataSource(String),
    #[error("computation error: {0}")]
    Computation(String),
    #[error("export error: {0}")]
    Export(String),
    #[error("cache unavailable: {0}")]
    CacheUnavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
}

// =====================================================================
// 4. service - ReportService (P0 真实, P1/P2 stub)
// =====================================================================

pub struct ReportService {
    cache: Box<dyn Cache>,
    work_item_port: Box<dyn WorkItemQueryPort>,
    sprint_port: Box<dyn SprintQueryPort>,
    user_port: Box<dyn UserQueryPort>,
    permission_port: Box<dyn PermissionPort>,
}

impl ReportService {
    pub fn new(
        cache: Box<dyn Cache>,
        work_item_port: Box<dyn WorkItemQueryPort>,
        sprint_port: Box<dyn SprintQueryPort>,
        user_port: Box<dyn UserQueryPort>,
        permission_port: Box<dyn PermissionPort>,
    ) -> Self {
        Self { cache, work_item_port, sprint_port, user_port, permission_port }
    }

    /// 生成报告 (per docs/specs/domain-report-spec.md §4.2 get_data)
    pub async fn generate(
        &self,
        report_type: ReportType,
        filter: ReportFilter,
    ) -> Result<ReportResult, ReportError> {
        // 稳定 cache key: 基于 filter 而非 report_id (report_id 每次新生成, 缓存会 miss)
        let scope_id = filter.sprint_id
            .or(filter.version_id)
            .or(filter.project_id)
            .or(filter.filter_id)
            .unwrap_or(filter.tenant_id);
        let cache_key = format!("report:{}:{}:{}", filter.tenant_id, scope_id, report_type.chart_id());

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
            self.generate_p0(report_type, &filter, report_id, cache_key.clone()).await?
        } else {
            self.generate_stub(report_type, &filter, report_id, cache_key.clone()).await?
        };

        // 写缓存
        if let Ok(val) = serde_json::to_value(&result) {
            let _ = self.cache.set_json(&cache_key, &val, 300).await;  // 5min TTL
        }
        Ok(result)
    }

    /// P0 真实生成
    async fn generate_p0(
        &self,
        report_type: ReportType,
        filter: &ReportFilter,
        report_id: Uuid,
        cache_key: String,
    ) -> Result<ReportResult, ReportError> {
        match report_type {
            ReportType::Burndown => {
                domain::c01_burndown::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::Burnup => {
                domain::c02_burnup::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::Velocity => {
                domain::c03_velocity::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::SprintReport => {
                domain::c04_sprint_report::generate(
                    &*self.work_item_port,
                    &*self.sprint_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::Cfd => {
                domain::c05_cfd::generate(
                    &*self.work_item_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::ControlChart => {
                domain::c06_control_chart::generate(
                    &*self.work_item_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::CycleTime => {
                domain::c07_cycle_time::generate(
                    &*self.work_item_port,
                    filter,
                    report_id,
                ).await
            }
            ReportType::CreatedVsResolved => {
                domain::c13_created_vs_resolved::generate(
                    &*self.work_item_port,
                    filter,
                    report_id,
                ).await
            }
            // P1/P2 + 暂未实装的 P0 子图走 stub
            _ => self.generate_stub(report_type, filter, report_id, cache_key).await,
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
        let points = (0..10).map(|i| ReportPoint {
            label: format!("Point {}", i + 1),
            value: 100.0 - (i as f64) * 10.0,
            extra: serde_json::json!({"stub": true, "chart_id": report_type.chart_id()}),
        }).collect();
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
        let mut out = String::from("label,value
");
        for p in &result.points {
            out.push_str(&format!("{},{}
", p.label, p.value));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::*;
    use crate::infrastructure::in_memory_cache::InMemoryCache;
    use crate::infrastructure::port_stubs::*;

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
        let r = svc.generate(ReportType::Heatmap, ReportFilter::default()).await.unwrap();
        assert_eq!(r.report_type, ReportType::Heatmap);
        assert_eq!(r.points.len(), 10);
    }
}
