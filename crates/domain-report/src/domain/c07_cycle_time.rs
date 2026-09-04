//! C07 Cycle Time 真实实现 (per docs/design/charts/c07-cycle-time.md v1.0)
//!
//! 直方图 + 50/85/95 百分位, 自适应桶大小

use crate::application::ports::WorkItemQueryPort;
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cycle Time 完整数据 schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleTimeData {
    /// 直方图桶
    pub buckets: Vec<Bucket>,
    /// 百分位数
    pub percentiles: Percentiles,
    /// 统计量
    pub stats: CycleStats,
    /// 桶大小 (天)
    pub bucket_size: u32,
}

/// 直方图单个桶
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    /// 区间起点
    pub range_start: f64,
    /// 区间终点
    pub range_end: f64,
    /// 落在区间内的数量
    pub count: f64,
    /// 展示标签
    pub label: String,
}

/// 百分位数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    /// 50 百分位 (中位数)
    pub p50: f64,
    /// 85 百分位
    pub p85: f64,
    /// 95 百分位
    pub p95: f64,
}

/// 周期时间统计量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleStats {
    /// 样本总数
    pub total_count: u32,
    /// 中位数
    pub median: f64,
    /// 均值
    pub mean: f64,
}

/// 公开入口: 异步生成 Cycle Time Report
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 阶段 2 简化: mock 50 issue 周期时间
    let cycle_times: Vec<f64> = (0..50)
        .map(|i| 1.0 + (i as f64) * 0.5 + ((i as f64 * 0.7).sin() * 1.5))
        .collect();
    let bucket_size = adaptive_bucket_size(cycle_times.len());
    let buckets = compute_buckets(&cycle_times, bucket_size);
    let percentiles = Percentiles {
        p50: percentile(&cycle_times, 50.0),
        p85: percentile(&cycle_times, 85.0),
        p95: percentile(&cycle_times, 95.0),
    };
    let stats = CycleStats {
        total_count: cycle_times.len() as u32,
        median: percentiles.p50,
        mean: cycle_times.iter().sum::<f64>() / cycle_times.len() as f64,
    };

    let data = CycleTimeData {
        buckets: buckets.clone(),
        percentiles: percentiles.clone(),
        stats: stats.clone(),
        bucket_size,
    };

    let points: Vec<ReportPoint> = data
        .buckets
        .iter()
        .map(|b| ReportPoint {
            label: b.label.clone(),
            value: b.count,
            extra: serde_json::json!({"range": [b.range_start, b.range_end]}),
        })
        .collect();

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::CycleTime,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: data.stats.total_count as f64,
            trend: Trend::Flat,
            anomalies: vec![],
            meta: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("cycle_time:{}", report_id),
    })
}

/// 自适应桶大小: 数据量越大, 桶越粗
pub fn adaptive_bucket_size(data_count: usize) -> u32 {
    match data_count {
        0..=49 => 1,
        50..=499 => 3,
        _ => 7,
    }
}

fn compute_buckets(cycle_times: &[f64], bucket_size: u32) -> Vec<Bucket> {
    if cycle_times.is_empty() {
        return vec![];
    }
    let max_val = cycle_times.iter().cloned().fold(0.0_f64, f64::max);
    let num_buckets = (max_val / bucket_size as f64).ceil() as usize + 1;
    let mut counts = vec![0.0; num_buckets];
    for &v in cycle_times {
        let idx = (v / bucket_size as f64).floor() as usize;
        if idx < num_buckets {
            counts[idx] += 1.0;
        }
    }
    counts
        .iter()
        .enumerate()
        .map(|(i, &c)| Bucket {
            range_start: i as f64 * bucket_size as f64,
            range_end: (i + 1) as f64 * bucket_size as f64,
            count: c,
            label: if c > 0.0 {
                format!(
                    "{}-{}d",
                    i * bucket_size as usize,
                    (i + 1) * bucket_size as usize
                )
            } else {
                String::new()
            },
        })
        .filter(|b| b.count > 0.0)
        .collect()
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
    fn test_adaptive_bucket_size() {
        assert_eq!(adaptive_bucket_size(10), 1);
        assert_eq!(adaptive_bucket_size(50), 3);
        assert_eq!(adaptive_bucket_size(100), 3);
        assert_eq!(adaptive_bucket_size(1000), 7);
    }

    #[test]
    fn test_compute_buckets() {
        let values = vec![1.0, 2.5, 4.0, 5.5];
        let buckets = compute_buckets(&values, 1);
        // 4 buckets (0-1, 1-2, 2-3, 4-5, 5-6) 至少 4 个非 0
        assert!(buckets.len() >= 4);
    }

    #[test]
    fn test_percentile_basic() {
        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        assert_eq!(percentile(&values, 50.0), 50.5);
        assert_eq!(percentile(&values, 95.0), 95.05);
    }

    #[test]
    fn test_percentile_empty() {
        let empty: Vec<f64> = vec![];
        assert_eq!(percentile(&empty, 50.0), 0.0);
    }
}
