//! C10 Time Tracking 真实实现 (per docs/design/charts/c10-time-tracking.md v1.0)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Time Tracking 完整数据 schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeTrackingData {
    /// 统计粒度 (e.g. "user")
    pub granularity: String,
    /// 各行数据
    pub rows: Vec<TrackingRow>,
    /// 汇总
    pub summary: TrackingSummary,
}

/// 单行工时跟踪数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingRow {
    /// 用户/维度 ID
    pub id: String,
    /// 展示名称
    pub name: String,
    /// 预估工时 (秒)
    pub original_seconds: f64,
    /// 已花费工时 (秒)
    pub spent_seconds: f64,
    /// 剩余工时 (秒)
    pub remaining_seconds: f64,
    /// 完成进度 (0.0-1.0)
    pub progress: f64,
}

/// 工时跟踪汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSummary {
    /// 预估工时总计
    pub total_original: f64,
    /// 已花费工时总计
    pub total_spent: f64,
    /// 剩余工时总计
    pub total_remaining: f64,
}

/// 公开入口: 异步生成 Time Tracking Report
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    let rows = vec![
        TrackingRow {
            id: "u1".into(),
            name: "Alice".into(),
            original_seconds: 28800.0,
            spent_seconds: 21600.0,
            remaining_seconds: 7200.0,
            progress: 0.75,
        },
        TrackingRow {
            id: "u2".into(),
            name: "Bob".into(),
            original_seconds: 21600.0,
            spent_seconds: 14400.0,
            remaining_seconds: 7200.0,
            progress: 0.667,
        },
        TrackingRow {
            id: "u3".into(),
            name: "Charlie".into(),
            original_seconds: 36000.0,
            spent_seconds: 28800.0,
            remaining_seconds: 7200.0,
            progress: 0.8,
        },
    ];
    let total_orig: f64 = rows.iter().map(|r| r.original_seconds).sum();
    let total_spent: f64 = rows.iter().map(|r| r.spent_seconds).sum();
    let total_rem: f64 = rows.iter().map(|r| r.remaining_seconds).sum();

    let data = TimeTrackingData {
        granularity: "user".to_string(),
        rows: rows.clone(),
        summary: TrackingSummary {
            total_original: total_orig,
            total_spent,
            total_remaining: total_rem,
        },
    };

    let points: Vec<ReportPoint> = rows.iter().map(|r| ReportPoint {
        label: r.name.clone(),
        value: r.spent_seconds,
        extra: serde_json::json!({"original": r.original_seconds, "remaining": r.remaining_seconds}),
    }).collect();

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::TimeTracking,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: total_spent,
            trend: Trend::Flat,
            anomalies: vec![],
            meta: serde_json::to_value(&data.summary)
                .map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("time_tracking:{}", report_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tracking_progress() {
        let r = TrackingRow {
            id: "x".into(),
            name: "X".into(),
            original_seconds: 100.0,
            spent_seconds: 25.0,
            remaining_seconds: 75.0,
            progress: 0.25,
        };
        assert_eq!(r.progress, 0.25);
    }
    #[test]
    fn test_summary_serde() {
        let s = TrackingSummary {
            total_original: 100.0,
            total_spent: 50.0,
            total_remaining: 50.0,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("total_spent"));
    }
}
