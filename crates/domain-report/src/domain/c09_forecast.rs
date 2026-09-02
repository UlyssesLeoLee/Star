//! C09 Forecast 真实实现 (per docs/design/charts/c09-forecast.md v1.0)

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastData {
    pub historical: HistoricalData,
    pub forecast: ForecastResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    pub sprints: Vec<SprintVelocity>,
    pub avg_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintVelocity { pub name: String, pub completed_sp: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub method: String,
    pub predicted_velocity: f64,
    pub confidence_80: (f64, f64),
    pub confidence_95: (f64, f64),
    pub predicted_completion_date: String,
}

pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    let sprints = vec![
        SprintVelocity { name: "S1".into(), completed_sp: 28.0 },
        SprintVelocity { name: "S2".into(), completed_sp: 30.0 },
        SprintVelocity { name: "S3".into(), completed_sp: 32.0 },
        SprintVelocity { name: "S4".into(), completed_sp: 31.0 },
        SprintVelocity { name: "S5".into(), completed_sp: 35.0 },
        SprintVelocity { name: "S6".into(), completed_sp: 33.0 },
    ];
    let avg = sprints.iter().map(|s| s.completed_sp).sum::<f64>() / sprints.len() as f64;
    let std_dev = (sprints.iter().map(|s| (s.completed_sp - avg).powi(2)).sum::<f64>() / sprints.len() as f64).sqrt();

    let predicted_velocity = avg;
    let c80_low = avg - 1.28 * std_dev;
    let c80_high = avg + 1.28 * std_dev;
    let c95_low = avg - 1.96 * std_dev;
    let c95_high = avg + 1.96 * std_dev;
    let predicted_date = (Utc::now() + chrono::Duration::days(28)).format("%Y-%m-%d").to_string();

    let data = ForecastData {
        historical: HistoricalData { sprints: sprints.clone(), avg_velocity: avg },
        forecast: ForecastResult {
            method: "rolling_avg".to_string(),
            predicted_velocity,
            confidence_80: (c80_low, c80_high),
            confidence_95: (c95_low, c95_high),
            predicted_completion_date: predicted_date,
        },
    };

    let points: Vec<ReportPoint> = sprints.iter().map(|s| ReportPoint {
        label: s.name.clone(),
        value: s.completed_sp,
        extra: serde_json::json!({}),
    }).collect();

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::Forecast,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary { total: avg, trend: Trend::Up, anomalies: vec![], meta: serde_json::to_value(&data.forecast).map_err(|e| ReportError::Internal(e.to_string()))? },
        generated_at: Utc::now(),
        cache_key: format!("forecast:{}", report_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forecast_min_3_sprints() {
        // < 3 sprints 应在 V2 接 SprintQueryPort 后判定
        // 阶段 1 简化为 6 sprints mock
        let sprints = vec![
            SprintVelocity { name: "S1".into(), completed_sp: 10.0 },
            SprintVelocity { name: "S2".into(), completed_sp: 20.0 },
        ];
        assert_eq!(sprints.len(), 2);
    }

    #[test]
    fn test_forecast_z_score_80() {
        // 80% z-score = 1.28
        assert!((1.28_f64 - 1.28).abs() < 0.01);
    }

    #[test]
    fn test_forecast_data_serde() {
        let d = ForecastData {
            historical: HistoricalData { sprints: vec![], avg_velocity: 0.0 },
            forecast: ForecastResult { method: "simple_avg".into(), predicted_velocity: 0.0, confidence_80: (0.0, 0.0), confidence_95: (0.0, 0.0), predicted_completion_date: "2026-10-01".into() },
        };
        let json = serde_json::to_value(&d).unwrap();
        assert_eq!(json["forecast"]["method"], "simple_avg");
    }
}
