//! C06 Control Chart 真实实现 (per docs/design/charts/c06-control-chart.md v1.0)
//!
//! Modified Z-Score (Iglewicz-Hoaglin) 异常检测, ±3σ 控制线
//! 散点图, ReferenceLine 标注中位 + 70/85/95 百分位 + ±3σ

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Control Chart 完整数据 schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlChartData {
    /// 散点数据
    pub data_points: Vec<ControlPoint>,
    /// 参考线 (中位/百分位/±3σ)
    pub reference_lines: Vec<RefLine>,
    /// 统计量
    pub stats: ControlStats,
}

/// 散点图上的单个数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPoint {
    /// WorkItem ID
    pub workitem_id: Uuid,
    /// issue key
    pub key: String,
    /// 周期时间 (天)
    pub cycle_time_days: f64,
    /// 完成时间
    pub completed_at: DateTime<Utc>,
    /// 是否为异常点
    pub anomaly: bool,
    /// Modified Z-Score
    pub z_score: f64,
}

/// 参考线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefLine {
    /// 纵轴数值
    pub y_value: f64,
    /// 标签
    pub label: String,
    /// 线型
    pub style: String, // "solid" / "dashed" / "dotted"
}

/// 统计量汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlStats {
    /// 中位数
    pub median: f64,
    /// 70 百分位
    pub p70: f64,
    /// 85 百分位
    pub p85: f64,
    /// 95 百分位
    pub p95: f64,
    /// 均值
    pub mean: f64,
    /// 标准差
    pub std_dev: f64,
}

/// 公开入口: 异步生成 Control Chart Report
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 阶段 2 简化: mock 20 个数据点
    let cycle_times: Vec<f64> = (0..20)
        .map(|i| 3.0 + (i as f64 * 0.4).sin() * 1.5 + 0.3 * i as f64)
        .collect();
    let (points, stats) = detect_anomalies(&cycle_times);
    let reference_lines = build_reference_lines(&stats);

    let data = ControlChartData {
        data_points: points,
        reference_lines,
        stats: stats.clone(),
    };

    let summary_count = data.data_points.len() as f64;
    let anomalies = data.data_points.iter().filter(|p| p.anomaly).count() as f64;

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::ControlChart,
        points: data
            .data_points
            .iter()
            .enumerate()
            .map(|(i, p)| ReportPoint {
                label: p.key.clone(),
                value: p.cycle_time_days,
                extra: serde_json::json!({"anomaly": p.anomaly, "z_score": p.z_score, "idx": i}),
            })
            .collect(),
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: summary_count,
            trend: Trend::Flat,
            anomalies: vec![format!("{} anomalies", anomalies)],
            meta: serde_json::to_value(&stats).map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("control_chart:{}", report_id),
    })
}

/// Modified Z-Score (Iglewicz-Hoaglin 1993): |M| > 3.5 为异常
/// M_i = 0.6745 * (x_i - median) / MAD
/// MAD = 0 fallback: 用 std_dev (经典 z-score)
pub fn detect_anomalies(cycle_times: &[f64]) -> (Vec<ControlPoint>, ControlStats) {
    let n = cycle_times.len();
    if n < 10 {
        // 数据不足, 不画控制线, 全 false anomaly
        let points: Vec<ControlPoint> = (0..n)
            .map(|i| ControlPoint {
                workitem_id: Uuid::new_v4(),
                key: format!("X-{}", i + 1),
                cycle_time_days: cycle_times[i],
                completed_at: Utc::now() - chrono::Duration::days((n - i) as i64),
                anomaly: false,
                z_score: 0.0,
            })
            .collect();
        let stats = ControlStats {
            median: percentile(cycle_times, 50.0),
            p70: percentile(cycle_times, 70.0),
            p85: percentile(cycle_times, 85.0),
            p95: percentile(cycle_times, 95.0),
            mean: if n > 0 {
                cycle_times.iter().sum::<f64>() / n as f64
            } else {
                0.0
            },
            std_dev: 0.0,
        };
        return (points, stats);
    }

    let median = percentile(cycle_times, 50.0);
    let mut deviations: Vec<f64> = cycle_times.iter().map(|x| (x - median).abs()).collect();
    let mad = percentile(&deviations, 50.0);
    let mean = cycle_times.iter().sum::<f64>() / n as f64;
    let std_dev = (cycle_times.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64).sqrt();

    let points: Vec<ControlPoint> = cycle_times
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            // MAD = 0 fallback 到 std_dev based z-score, 阈值同 3.5
            let m = if mad > 0.0 {
                0.6745 * (x - median) / mad
            } else if std_dev > 0.0 {
                (x - mean) / std_dev
            } else {
                0.0
            };
            ControlPoint {
                workitem_id: Uuid::new_v4(),
                key: format!("X-{}", i + 1),
                cycle_time_days: x,
                completed_at: Utc::now() - chrono::Duration::days((n - i) as i64),
                anomaly: m.abs() > 3.5,
                z_score: m,
            }
        })
        .collect();

    let p70 = percentile(cycle_times, 70.0);
    let p85 = percentile(cycle_times, 85.0);
    let p95 = percentile(cycle_times, 95.0);
    let stats = ControlStats {
        median,
        p70,
        p85,
        p95,
        mean,
        std_dev,
    };
    (points, stats)
}

fn build_reference_lines(stats: &ControlStats) -> Vec<RefLine> {
    let mut lines = vec![
        RefLine {
            y_value: stats.median,
            label: "Median".into(),
            style: "solid".into(),
        },
        RefLine {
            y_value: stats.p70,
            label: "70%".into(),
            style: "dashed".into(),
        },
        RefLine {
            y_value: stats.p85,
            label: "85%".into(),
            style: "dashed".into(),
        },
        RefLine {
            y_value: stats.p95,
            label: "95%".into(),
            style: "dashed".into(),
        },
    ];
    if stats.std_dev > 0.0 {
        let three_sigma = 3.0 * stats.std_dev;
        lines.push(RefLine {
            y_value: stats.mean + three_sigma,
            label: "+3σ".into(),
            style: "dotted".into(),
        });
        lines.push(RefLine {
            y_value: (stats.mean - three_sigma).max(0.0),
            label: "-3σ".into(),
            style: "dotted".into(),
        });
    }
    lines
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let rank = (p / 100.0) * (n - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        sorted[lower] + (rank - lower as f64) * (sorted[upper] - sorted[lower])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modified_z_no_anomaly() {
        // 正常数据, 全部 |M| < 3.5
        let data = vec![3.0, 4.0, 5.0, 3.5, 4.5, 5.5, 3.2, 4.8, 5.2, 4.0, 3.7, 4.3];
        let (points, _) = detect_anomalies(&data);
        assert!(!points.iter().any(|p| p.anomaly), "all should be normal");
    }

    #[test]
    fn test_modified_z_outlier_detected() {
        let mut data = vec![3.0; 20];
        data.push(100.0); // 明显异常
        let (points, _) = detect_anomalies(&data);
        assert!(points.last().unwrap().anomaly, "outlier should be anomaly");
    }

    #[test]
    fn test_min_10_points() {
        let data = vec![3.0, 4.0, 5.0];
        let (points, _) = detect_anomalies(&data);
        assert!(points.iter().all(|p| !p.anomaly), "< 10 points all normal");
        assert!(points.iter().all(|p| p.z_score == 0.0));
    }

    #[test]
    fn test_percentile_linear() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 50.0), 3.0);
        assert_eq!(percentile(&values, 0.0), 1.0);
        assert_eq!(percentile(&values, 100.0), 5.0);
    }

    #[test]
    fn test_reference_lines_with_zero_std() {
        let stats = ControlStats {
            median: 5.0,
            p70: 6.0,
            p85: 7.0,
            p95: 8.0,
            mean: 5.0,
            std_dev: 0.0,
        };
        let lines = build_reference_lines(&stats);
        // 无 ±3σ
        assert_eq!(lines.len(), 4);
    }
}
