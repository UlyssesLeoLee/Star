// SPDX-License-Identifier: MIT OR Apache-2.0
//! F-03 运维数据子域 (per SRS-001 §4 F-03)
//!
//! MVP-骨架: 1 数据结构 `OpsMetric` + 5 KPI stub
//! 实装阶段: 接入 star-telemetry + Prometheus client

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 趋势方向 (3 态)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    Rising,
    Stable,
    Falling,
}

/// 运维指标 (per OPS-BASIC-DESIGN §3.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub trend: TrendDirection,
    pub last_updated: DateTime<Utc>,
}

impl OpsMetric {
    /// MVP stub: 5 KPI hardcoded (per OPS-BASIC-DESIGN §3.3)
    pub fn summary_stub() -> Vec<Self> {
        let now: DateTime<Utc> = "2026-09-08T07:53:00Z".parse().expect("hardcoded UTC");
        vec![
            Self {
                name: "cpu_avg".to_string(),
                value: 0.35,
                unit: "ratio".to_string(),
                trend: TrendDirection::Stable,
                last_updated: now,
            },
            Self {
                name: "mem_avg".to_string(),
                value: 0.62,
                unit: "ratio".to_string(),
                trend: TrendDirection::Rising,
                last_updated: now,
            },
            Self {
                name: "active_tasks".to_string(),
                value: 17.0,
                unit: "count".to_string(),
                trend: TrendDirection::Stable,
                last_updated: now,
            },
            Self {
                name: "mcp_qps".to_string(),
                value: 4.2,
                unit: "qps".to_string(),
                trend: TrendDirection::Falling,
                last_updated: now,
            },
            Self {
                name: "llm_token_daily".to_string(),
                value: 1_240_000.0,
                unit: "tokens".to_string(),
                trend: TrendDirection::Rising,
                last_updated: now,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_stub_returns_five_kpis() {
        let metrics = OpsMetric::summary_stub();
        assert_eq!(metrics.len(), 5);
        let names: Vec<&str> = metrics.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"cpu_avg"));
        assert!(names.contains(&"llm_token_daily"));
    }
}
