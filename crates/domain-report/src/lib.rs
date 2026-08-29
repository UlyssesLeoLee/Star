//! Star Report Engine (精简实装 v0.1)
//!
//! 10 种报告:
//! 1. Burndown (Sprint 燃尽图)
//! 2. Burnup (Sprint 燃起图)
//! 3. Velocity (跨 Sprint 速度)
//! 4. CFD (Cumulative Flow Diagram)
//! 5. Control Chart (周期时间 + 异常检测)
//! 6. Average Age (工作项平均年龄)
//! 7. Created vs Resolved (时间序列对比)
//! 8. Workload (per assignee 负载)
//! 9. Epic Burndown (单 Epic 进度)
//! 10. Sprint Report (完成 / 移出 / 移入)
//!
//! 数据源: domain-work-item + domain-planning + domain-audit
//! 导出: JSON / CSV (本任务) + PDF (Phase 2)
//! 缓存: star-cache (Phase 2)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. value_object
// =====================================================================

/// 报告类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Burndown,
    Burnup,
    Velocity,
    Cfd,
    ControlChart,
    AverageAge,
    CreatedVsResolved,
    Workload,
    EpicBurndown,
    SprintReport,
}

impl ReportType {
    pub fn all() -> &'static [ReportType] {
        &[
            Self::Burndown,
            Self::Burnup,
            Self::Velocity,
            Self::Cfd,
            Self::ControlChart,
            Self::AverageAge,
            Self::CreatedVsResolved,
            Self::Workload,
            Self::EpicBurndown,
            Self::SprintReport,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Burndown => "Sprint Burndown",
            Self::Burnup => "Sprint Burnup",
            Self::Velocity => "Team Velocity",
            Self::Cfd => "Cumulative Flow",
            Self::ControlChart => "Cycle Time Control",
            Self::AverageAge => "Average Age",
            Self::CreatedVsResolved => "Created vs Resolved",
            Self::Workload => "Workload",
            Self::EpicBurndown => "Epic Burndown",
            Self::SprintReport => "Sprint Report",
        }
    }
}

/// 报告作用域过滤
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportFilter {
    pub project_id: Option<Uuid>,
    pub sprint_id: Option<Uuid>,
    pub epic_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub time_range: Option<TimeRange>,
}

impl Default for ReportFilter {
    fn default() -> Self {
        Self {
            project_id: None,
            sprint_id: None,
            epic_id: None,
            assignee_id: None,
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

/// 报告定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub filter: ReportFilter,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 报告数据点 (不同报告类型用不同字段, 用 serde_json::Value 灵活)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportPoint {
    pub label: String,
    pub value: f64,
    pub extra: serde_json::Value,
}

/// 报告结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    pub report_id: Uuid,
    pub report_type: ReportType,
    pub points: Vec<ReportPoint>,
    pub summary: ReportSummary,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total: f64,
    pub trend: Trend,           // Up / Down / Flat
    pub anomalies: Vec<String>, // 异常点
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    Flat,
}

// =====================================================================
// 3. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReportError {
    #[error("data source error: {0}")]
    DataSource(String),
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
    #[error("computation error: {0}")]
    Computation(String),
    #[error("export error: {0}")]
    Export(String),
}

// =====================================================================
// 4. service — 10 报告生成器 (stub 实现 + 真实占位数据)
// =====================================================================

pub struct ReportService;

impl ReportService {
    pub fn new() -> Self {
        Self
    }

    /// 生成报告 (本任务用 stub, 真实数据走 Phase 2 接 domain-work-item)
    pub fn generate(
        &self,
        report_type: ReportType,
        filter: ReportFilter,
    ) -> Result<ReportResult, ReportError> {
        let report_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let (points, summary) = match report_type {
            ReportType::Burndown => burndown_stub(),
            ReportType::Burnup => burnup_stub(),
            ReportType::Velocity => velocity_stub(),
            ReportType::Cfd => cfd_stub(),
            ReportType::ControlChart => control_chart_stub(),
            ReportType::AverageAge => average_age_stub(),
            ReportType::CreatedVsResolved => cvr_stub(),
            ReportType::Workload => workload_stub(),
            ReportType::EpicBurndown => epic_burndown_stub(),
            ReportType::SprintReport => sprint_report_stub(),
        };
        Ok(ReportResult {
            report_id,
            report_type,
            points,
            summary,
            generated_at: now,
        })
    }

    /// 导出 JSON
    pub fn export_json(&self, result: &ReportResult) -> Result<String, ReportError> {
        serde_json::to_string_pretty(result).map_err(|e| ReportError::Export(e.to_string()))
    }

    /// 导出 CSV
    pub fn export_csv(&self, result: &ReportResult) -> Result<String, ReportError> {
        let mut out = String::from("label,value\n");
        for p in &result.points {
            out.push_str(&format!("{},{}\n", p.label, p.value));
        }
        Ok(out)
    }
}

impl Default for ReportService {
    fn default() -> Self {
        Self::new()
    }
}

// 10 个 stub 生成器 — 真实数据源接入 Phase 2
fn burndown_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (0..10)
        .map(|i| ReportPoint {
            label: format!("Day {}", i + 1),
            value: 100.0 - (i as f64) * 10.0,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 100.0,
            trend: Trend::Down,
            anomalies: vec![],
        },
    )
}

fn burnup_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (0..10)
        .map(|i| ReportPoint {
            label: format!("Day {}", i + 1),
            value: (i as f64) * 10.0,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 100.0,
            trend: Trend::Up,
            anomalies: vec![],
        },
    )
}

fn velocity_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (1..=6)
        .map(|i| ReportPoint {
            label: format!("Sprint {}", i),
            value: 30.0 + (i as f64) * 2.5,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 187.5,
            trend: Trend::Up,
            anomalies: vec![],
        },
    )
}

fn cfd_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let labels = ["Todo", "In Progress", "Review", "Done"];
    let pts = labels
        .iter()
        .enumerate()
        .map(|(i, l)| ReportPoint {
            label: l.to_string(),
            value: 25.0 - (i as f64) * 6.0,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 25.0,
            trend: Trend::Flat,
            anomalies: vec![],
        },
    )
}

fn control_chart_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (0..20)
        .map(|i| ReportPoint {
            label: format!("Sample {}", i + 1),
            value: 5.0 + ((i as f64) * 0.3).sin(),
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 5.0,
            trend: Trend::Flat,
            anomalies: vec!["Sample 15".into()],
        },
    )
}

fn average_age_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (0..7)
        .map(|i| ReportPoint {
            label: format!("Day -{}", i),
            value: 3.5 + (i as f64) * 0.2,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 3.7,
            trend: Trend::Up,
            anomalies: vec![],
        },
    )
}

fn cvr_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (1..=14)
        .map(|i| ReportPoint {
            label: format!("Day {}", i),
            value: 8.0 + ((i as f64) * 0.5).sin() * 2.0,
            extra: serde_json::json!({"created": 9.0, "resolved": 7.5}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 8.0,
            trend: Trend::Up,
            anomalies: vec![],
        },
    )
}

fn workload_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let names = ["Alice", "Bob", "Charlie", "Dave", "Eve"];
    let pts = names
        .iter()
        .enumerate()
        .map(|(i, n)| ReportPoint {
            label: n.to_string(),
            value: 5.0 + (i as f64) * 1.5,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 20.0,
            trend: Trend::Flat,
            anomalies: vec![],
        },
    )
}

fn epic_burndown_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let pts = (0..14)
        .map(|i| ReportPoint {
            label: format!("Day {}", i + 1),
            value: 50.0 - (i as f64) * 3.5,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 50.0,
            trend: Trend::Down,
            anomalies: vec![],
        },
    )
}

fn sprint_report_stub() -> (Vec<ReportPoint>, ReportSummary) {
    let labels = ["Completed", "Moved Out", "Moved In", "Not Done"];
    let pts = labels
        .iter()
        .enumerate()
        .map(|(i, l)| ReportPoint {
            label: l.to_string(),
            value: 18.0 - (i as f64) * 4.0,
            extra: serde_json::json!({}),
        })
        .collect();
    (
        pts,
        ReportSummary {
            total: 18.0,
            trend: Trend::Flat,
            anomalies: vec![],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_all_count() {
        assert_eq!(ReportType::all().len(), 10);
    }

    #[test]
    fn test_report_type_name() {
        assert_eq!(ReportType::Burndown.name(), "Sprint Burndown");
        assert_eq!(ReportType::Velocity.name(), "Team Velocity");
    }

    #[test]
    fn test_generate_all_10_types() {
        let svc = ReportService::new();
        for rt in ReportType::all() {
            let r = svc.generate(*rt, ReportFilter::default()).unwrap();
            assert!(!r.points.is_empty(), "{:?} should have points", rt);
        }
    }

    #[test]
    fn test_export_json() {
        let svc = ReportService::new();
        let r = svc
            .generate(ReportType::Burndown, ReportFilter::default())
            .unwrap();
        let json = svc.export_json(&r).unwrap();
        assert!(json.contains("burndown"));
        assert!(json.contains("Day 1"));
    }

    #[test]
    fn test_export_csv() {
        let svc = ReportService::new();
        let r = svc
            .generate(ReportType::Velocity, ReportFilter::default())
            .unwrap();
        let csv = svc.export_csv(&r).unwrap();
        assert!(csv.starts_with("label,value\n"));
        assert!(csv.contains("Sprint 1"));
    }
}
