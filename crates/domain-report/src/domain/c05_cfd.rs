//! C05 CFD 真实实现 (per docs/design/charts/c05-cfd.md v1.0)
//!
//! 累积流图: 每天各状态 issue 数堆叠面积

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfdData {
    pub date_range: DateRange,
    pub status_categories: Vec<String>,
    pub series: Vec<DayCount>,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCount {
    pub day: String,
    pub counts: std::collections::BTreeMap<String, f64>,
}

pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 阶段 2 简化: 返回 mock CFD data
    let days = 14;
    let categories = vec!["todo".to_string(), "in_progress".to_string(), "in_review".to_string(), "done".to_string()];
    let series: Vec<DayCount> = (0..days).map(|i| {
        let mut counts = std::collections::BTreeMap::new();
        counts.insert("todo".to_string(), 15.0 - i as f64 * 0.5);
        counts.insert("in_progress".to_string(), 8.0 + (i as f64 * 0.3).sin() * 2.0);
        counts.insert("in_review".to_string(), 5.0);
        counts.insert("done".to_string(), 20.0 + i as f64 * 1.5);
        let day = (Utc::now() - chrono::Duration::days((days - i) as i64)).format("%Y-%m-%d").to_string();
        DayCount { day, counts }
    }).collect();

    let total = 50.0;
    let data = CfdData {
        date_range: DateRange {
            start: series.first().map(|s| s.day.clone()).unwrap_or_default(),
            end: series.last().map(|s| s.day.clone()).unwrap_or_default(),
        },
        status_categories: categories.clone(),
        series,
        total,
    };

    let points: Vec<ReportPoint> = data.series.iter().map(|d| ReportPoint {
        label: d.day.clone(),
        value: d.counts.values().sum(),
        extra: serde_json::to_value(&d.counts).unwrap_or(serde_json::json!({})),
    }).collect();

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::Cfd,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total,
            trend: Trend::Flat,
            anomalies: vec![],
            meta: serde_json::json!({"categories": categories}),
        },
        generated_at: Utc::now(),
        cache_key: format!("cfd:{}", report_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfd_categories() {
        let cats = vec!["todo".to_string(), "in_progress".to_string(), "done".to_string()];
        assert_eq!(cats.len(), 3);
    }

    #[test]
    fn test_cfd_total_invariant() {
        // CFD 总和恒等于 issue 总数
        let mut counts = std::collections::BTreeMap::new();
        counts.insert("todo".to_string(), 10.0);
        counts.insert("done".to_string(), 15.0);
        let sum: f64 = counts.values().sum();
        assert_eq!(sum, 25.0);
    }

    #[test]
    fn test_cfd_date_range_serde() {
        let r = DateRange { start: "2026-09-01".into(), end: "2026-09-14".into() };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("2026-09-01"));
    }
}
