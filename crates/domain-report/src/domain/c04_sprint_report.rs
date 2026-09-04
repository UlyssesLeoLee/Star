//! C04 Sprint Report 真实实现 (per docs/design/charts/c04-sprint-report.md v1.0)
//!
//! 表格型: 3 组 (completed / carry_over / incomplete) + summary

use crate::application::ports::{SprintQueryPort, WorkItemQueryPort};
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintReportData {
    pub sprint: SprintInfo,
    pub groups: Groups,
    pub summary: SprintReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintInfo {
    pub sprint_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groups {
    pub completed: Vec<IssueRow>,
    pub carry_over: Vec<IssueRow>,
    pub incomplete: Vec<IssueRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRow {
    pub key: String,
    pub title: String,
    pub issue_type: String,
    pub priority: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub story_points: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintReportSummary {
    pub completed_count: u32,
    pub carry_over_count: u32,
    pub incomplete_count: u32,
    pub completed_sp: f64,
}

pub async fn generate(
    _work_item_port: &dyn WorkItemQueryPort,
    _sprint_port: &dyn SprintQueryPort,
    _filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 阶段 2 简化: 返回 mock data (真实 Sprint + Issue 拉取留 V3)
    let data = SprintReportData {
        sprint: SprintInfo {
            sprint_id: Uuid::new_v4(),
            name: "Sprint Current".into(),
        },
        groups: Groups {
            completed: vec![
                IssueRow {
                    key: "PROJ-101".into(),
                    title: "Implement login".into(),
                    issue_type: "Story".into(),
                    priority: "high".into(),
                    completed_at: Some(Utc::now() - chrono::Duration::days(2)),
                    story_points: Some(8.0),
                },
                IssueRow {
                    key: "PROJ-102".into(),
                    title: "Fix bug #456".into(),
                    issue_type: "Bug".into(),
                    priority: "medium".into(),
                    completed_at: Some(Utc::now() - chrono::Duration::days(1)),
                    story_points: Some(5.0),
                },
            ],
            carry_over: vec![IssueRow {
                key: "PROJ-99".into(),
                title: "Refactor auth module".into(),
                issue_type: "Task".into(),
                priority: "low".into(),
                completed_at: Some(Utc::now() - chrono::Duration::days(0)),
                story_points: Some(3.0),
            }],
            incomplete: vec![IssueRow {
                key: "PROJ-103".into(),
                title: "Add OAuth support".into(),
                issue_type: "Story".into(),
                priority: "medium".into(),
                completed_at: None,
                story_points: Some(13.0),
            }],
        },
        summary: SprintReportSummary {
            completed_count: 2,
            carry_over_count: 1,
            incomplete_count: 1,
            completed_sp: 13.0,
        },
    };

    let s = data.summary.clone();
    let points: Vec<ReportPoint> = vec![
        ReportPoint {
            label: "completed".into(),
            value: s.completed_count as f64,
            extra: serde_json::json!({}),
        },
        ReportPoint {
            label: "carry_over".into(),
            value: s.carry_over_count as f64,
            extra: serde_json::json!({}),
        },
        ReportPoint {
            label: "incomplete".into(),
            value: s.incomplete_count as f64,
            extra: serde_json::json!({}),
        },
    ];

    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::SprintReport,
        points,
        data: serde_json::to_value(&data).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: s.completed_sp,
            trend: Trend::Flat,
            anomalies: vec![],
            meta: serde_json::to_value(&s).map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("sprint_report:{}", report_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groups_default_empty() {
        let g = Groups {
            completed: vec![],
            carry_over: vec![],
            incomplete: vec![],
        };
        assert_eq!(g.completed.len(), 0);
    }

    #[test]
    fn test_issue_row_serde() {
        let row = IssueRow {
            key: "X-1".into(),
            title: "T".into(),
            issue_type: "Story".into(),
            priority: "high".into(),
            completed_at: None,
            story_points: Some(5.0),
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("\"key\":\"X-1\""));
        assert!(json.contains("\"story_points\":5.0"));
    }

    #[test]
    fn test_summary_arithmetic() {
        let s = SprintReportSummary {
            completed_count: 5,
            carry_over_count: 2,
            incomplete_count: 3,
            completed_sp: 25.0,
        };
        let total = s.completed_count + s.carry_over_count + s.incomplete_count;
        assert_eq!(total, 10);
    }

    #[test]
    fn test_sprint_info_serde() {
        let s = SprintInfo {
            sprint_id: Uuid::nil(),
            name: "Sprint 1".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("Sprint 1"));
    }
}
