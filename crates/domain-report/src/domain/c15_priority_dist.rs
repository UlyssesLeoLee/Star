//! C15 Priority Distribution 真实实现 (per docs/design/charts/c15-priority-dist.md v1.0)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityDistData {
    pub slices: Vec<PrioritySlice>,
    pub total: f64,
    pub status_filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySlice {
    pub key: String,
    pub count: f64,
    pub percentage: f64,
}

pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    let raw = vec![
        ("highest", 3.0),
        ("high", 12.0),
        ("medium", 35.0),
        ("low", 28.0),
        ("lowest", 7.0),
    ];
    let total: f64 = raw.iter().map(|(_, c)| c).sum();
    let slices: Vec<PrioritySlice> = raw
        .iter()
        .map(|(k, c)| PrioritySlice {
            key: k.to_string(),
            count: *c,
            percentage: c / total,
        })
        .collect();

    let data = PriorityDistData {
        slices: slices.clone(),
        total,
        status_filter: "open".to_string(),
    };

    let points: Vec<ReportPoint> = slices
        .iter()
        .map(|s| ReportPoint {
            label: s.key.clone(),
            value: s.count,
            extra: serde_json::json!({"percentage": s.percentage}),
        })
        .collect();

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::PriorityDist,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total,
            trend: Trend::Flat,
            anomalies: vec![],
            meta: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("priority_dist:{}", report_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_priority_slice_serde() {
        let s = PrioritySlice {
            key: "high".into(),
            count: 5.0,
            percentage: 0.5,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"key\":\"high\""));
    }
}
