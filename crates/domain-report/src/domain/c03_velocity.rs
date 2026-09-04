//! C03 Velocity Chart 真实实现 (per docs/design/charts/c03-velocity.md v1.0)
//!
//! 跨多个 Sprint 团队承诺 SP vs 完成 SP 对比 + 平均完成线
//! 与 C01/C02 差异: 多 Sprint 聚合 (S1 Project scope), Bar + ReferenceLine

use crate::application::ports::{SprintQueryPort, WorkItemQueryPort};
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Velocity 完整数据 schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityData {
    /// 各 Sprint 的承诺/完成 SP
    pub sprints: Vec<SprintVelocity>,
    /// 平均完成 SP
    pub average_completed_sp: f64,
    /// 速度趋势
    pub trend: VelocityTrend,
}

/// 单个 Sprint 的承诺/完成 SP 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintVelocity {
    /// Sprint ID
    pub sprint_id: Uuid,
    /// Sprint 名称
    pub name: String,
    /// Sprint 开始时间
    pub start_date: DateTime<Utc>,
    /// Sprint 结束时间
    pub end_date: DateTime<Utc>,
    /// Sprint 状态
    pub status: SprintStatus,
    /// 承诺 SP
    pub committed_sp: f64,
    /// 完成 SP (None = active sprint)
    pub completed_sp: Option<f64>, // None = active sprint
}

/// Sprint 状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SprintStatus {
    /// 已完成
    Completed,
    /// 进行中
    Active,
    /// 已计划但未开始
    Planned,
}

/// 速度趋势
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VelocityTrend {
    /// 上升
    Increasing,
    /// 下降
    Decreasing,
    /// 稳定
    Stable,
}

/// 公开入口: 异步生成 Velocity Report
pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _sprint_port: &dyn SprintQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 阶段 2 简化: 返回 mock data, 真实 SprintQueryPort 接入留 V3
    let data = VelocityData {
        sprints: vec![
            SprintVelocity {
                sprint_id: Uuid::new_v4(),
                name: "Sprint 1".into(),
                start_date: Utc::now() - chrono::Duration::days(84),
                end_date: Utc::now() - chrono::Duration::days(70),
                status: SprintStatus::Completed,
                committed_sp: 30.0,
                completed_sp: Some(28.0),
            },
            SprintVelocity {
                sprint_id: Uuid::new_v4(),
                name: "Sprint 2".into(),
                start_date: Utc::now() - chrono::Duration::days(70),
                end_date: Utc::now() - chrono::Duration::days(56),
                status: SprintStatus::Completed,
                committed_sp: 32.0,
                completed_sp: Some(30.0),
            },
            SprintVelocity {
                sprint_id: Uuid::new_v4(),
                name: "Sprint 3".into(),
                start_date: Utc::now() - chrono::Duration::days(56),
                end_date: Utc::now() - chrono::Duration::days(42),
                status: SprintStatus::Completed,
                committed_sp: 35.0,
                completed_sp: Some(33.0),
            },
            SprintVelocity {
                sprint_id: Uuid::new_v4(),
                name: "Sprint 4".into(),
                start_date: Utc::now() - chrono::Duration::days(14),
                end_date: Utc::now() + chrono::Duration::days(0),
                status: SprintStatus::Active,
                committed_sp: 35.0,
                completed_sp: None,
            },
        ],
        average_completed_sp: 30.3,
        trend: VelocityTrend::Stable,
    };

    let avg = data.average_completed_sp;
    let trend = data.trend;
    let points: Vec<ReportPoint> = data.sprints.iter().map(|s| ReportPoint {
        label: s.name.clone(),
        value: s.completed_sp.unwrap_or(0.0),
        extra: serde_json::json!({"committed_sp": s.committed_sp, "status": format!("{:?}", s.status)}),
    }).collect();

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::Velocity,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: avg,
            trend: Trend::Flat,
            anomalies: vec![],
            meta: serde_json::json!({"average_completed_sp": avg, "trend": format!("{:?}", trend)}),
        },
        generated_at: Utc::now(),
        cache_key: format!("velocity:{}", report_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_status_serde() {
        let s = SprintStatus::Completed;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"Completed\"");
    }

    #[test]
    fn test_velocity_trend_eq() {
        assert_eq!(VelocityTrend::Stable, VelocityTrend::Stable);
        assert_ne!(VelocityTrend::Increasing, VelocityTrend::Decreasing);
    }

    #[test]
    fn test_velocity_default() {
        let v = SprintVelocity {
            sprint_id: Uuid::nil(),
            name: "Test".into(),
            start_date: Utc::now(),
            end_date: Utc::now(),
            status: SprintStatus::Planned,
            committed_sp: 0.0,
            completed_sp: None,
        };
        assert_eq!(v.completed_sp, None);
    }

    #[test]
    fn test_velocity_data_clone() {
        let data = VelocityData {
            sprints: vec![],
            average_completed_sp: 0.0,
            trend: VelocityTrend::Stable,
        };
        let cloned = data.clone();
        assert_eq!(cloned.sprints.len(), 0);
    }

    #[test]
    fn test_velocity_serialize_data() {
        let data = VelocityData {
            sprints: vec![],
            average_completed_sp: 25.0,
            trend: VelocityTrend::Increasing,
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["average_completed_sp"], 25.0);
        assert_eq!(json["trend"], "Increasing");
    }
}
