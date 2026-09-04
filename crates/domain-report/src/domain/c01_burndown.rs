//! C01 Burndown Chart 真实实现 (per docs/design/charts/c01-burndown.md v1.0)
//!
//! 阶段 1 重点: SQL 查询 (走 Port stub) + 真实算法 (理想线 + 实际线 + scope change) + 错误处理

use crate::application::ports::{SprintQueryPort, WorkItemQueryPort};
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sprint 元数据 (从 SprintQueryPort 拉)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintMeta {
    /// Sprint ID
    pub sprint_id: Uuid,
    /// Sprint 名称
    pub name: String,
    /// Sprint 开始时间
    pub start_date: DateTime<Utc>,
    /// Sprint 结束时间
    pub end_date: DateTime<Utc>,
    /// Sprint 总 Story Point
    pub total_sp: f64,
    /// Sprint 期间的范围变更记录
    pub scope_change_log: Vec<ScopeChange>,
}

/// Sprint 范围变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeChange {
    /// 变更发生时间
    pub at: DateTime<Utc>,
    /// SP 变化量 (正 = 增加范围, 负 = 减少范围)
    pub delta_sp: f64,
    /// 变更原因
    pub reason: String,
    /// 变更后的总 SP
    pub new_total_sp: f64,
}

/// Burndown 完整数据 schema (与 frontend src/lib/chart-data-schema.ts 同构)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownData {
    /// Sprint 元数据
    pub sprint: SprintMeta,
    /// 理想线 + 实际线
    pub series: BurndownSeries,
    /// 范围变更记录
    pub scope_changes: Vec<ScopeChange>,
    /// 汇总统计
    pub summary: BurndownSummary,
}

/// Burndown 曲线数据 (理想线 + 实际线)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownSeries {
    /// 理想燃尽线
    pub ideal: Vec<TimeSeriesPoint>,
    /// 实际燃尽线
    pub actual: Vec<TimeSeriesPoint>,
}

/// 时间序列上的单个数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// ISO 日期, e.g. "2026-09-02"
    pub x: String,
    /// 数值 (剩余 SP)
    pub y: f64,
}

/// Burndown 汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurndownSummary {
    /// 剩余 SP
    pub remaining_sp: f64,
    /// 已完成 SP
    pub completed_sp: f64,
    /// 已完成 issue 数
    pub completed_issues: u32,
    /// 总 issue 数
    pub total_issues: u32,
    /// 预测最终完成 SP (线性外推)
    pub predicted_completion_sp: f64,
    /// 是否在计划轨道内
    pub on_track: bool,
}

/// 公开入口: 异步生成 Burndown Report
pub async fn generate(
    work_item_port: &dyn WorkItemQueryPort,
    sprint_port: &dyn SprintQueryPort,
    filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 1. 拉 Sprint 元数据
    let sprint_id = filter.sprint_id.ok_or_else(|| {
        ReportError::ValidationFailed("Burndown requires sprint_id in filter".into())
    })?;
    let sprint = sprint_port
        .get_sprint(filter.tenant_id, sprint_id)
        .await
        .map_err(|e| ReportError::DataSource(e.to_string()))?
        .ok_or_else(|| ReportError::NotFound(sprint_id))?;

    // 2. 拉已完成 issue
    let completed_issues = work_item_port
        .list_completed_in_sprint(
            filter.tenant_id,
            sprint_id,
            sprint.start_date,
            sprint.end_date,
        )
        .await
        .map_err(|e| ReportError::DataSource(e.to_string()))?;

    // 3. 拉所有 issue (用于算总数)
    let all_issues = work_item_port
        .list_in_sprint(filter.tenant_id, sprint_id)
        .await
        .map_err(|e| ReportError::DataSource(e.to_string()))?;

    // 4. 计算 daily completed SP
    let total_issues = all_issues.len() as u32;
    let completed_count = completed_issues.len() as u32;
    let total_completed_sp: f64 = completed_issues.iter().filter_map(|i| i.story_points).sum();

    // 5. 构造 ideal + actual + summary
    let burndown = compute_burndown(
        &sprint,
        &completed_issues,
        total_completed_sp,
        total_issues,
        completed_count,
    );

    // 6. 转 ReportPoint (向后兼容旧接口)
    let points: Vec<ReportPoint> = burndown.series.actual.iter().enumerate().map(|(i, p)| ReportPoint {
        label: p.x.clone(),
        value: p.y,
        extra: serde_json::json!({"ideal": burndown.series.ideal.get(i).map(|x| x.y).unwrap_or(0.0)}),
    }).collect();

    // 7. 返回 ReportResult
    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::Burndown,
        points,
        data: serde_json::to_value(&burndown).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: sprint.total_sp,
            trend: if burndown.summary.on_track {
                Trend::Down
            } else {
                Trend::Up
            },
            anomalies: vec![],
            meta: serde_json::to_value(&burndown.summary)
                .map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("burndown:{}:{}", filter.tenant_id, sprint_id),
    })
}

/// 纯函数: 计算 ideal + actual + summary (易测试)
fn compute_burndown(
    sprint: &SprintMeta,
    completed_issues: &[CompletedIssue],
    total_completed_sp: f64,
    total_issues: u32,
    completed_count: u32,
) -> BurndownData {
    // 算 Sprint 天数
    let days = (sprint.end_date - sprint.start_date).num_days() + 1;
    let total_sp = sprint.total_sp;
    let daily_ideal_decrement = if days > 1 {
        total_sp / (days - 1) as f64
    } else {
        0.0
    };

    // 1. Ideal 线 (线性下降)
    let ideal: Vec<TimeSeriesPoint> = (0..days)
        .map(|i| {
            let day = sprint.start_date + Duration::days(i);
            TimeSeriesPoint {
                x: day.format("%Y-%m-%d").to_string(),
                y: (total_sp - daily_ideal_decrement * i as f64).max(0.0),
            }
        })
        .collect();

    // 2. Actual 线 (累积完成的反向)
    // 按 completed_at date 分桶
    let mut daily_completed: std::collections::BTreeMap<String, f64> =
        std::collections::BTreeMap::new();
    for issue in completed_issues {
        let day_key = issue.completed_at.format("%Y-%m-%d").to_string();
        *daily_completed.entry(day_key).or_insert(0.0) += issue.story_points.unwrap_or(0.0);
    }
    let mut cumulative = 0.0;
    let mut actual: Vec<TimeSeriesPoint> = Vec::new();
    let mut last_progress_idx: usize = 0; // 最后一个有 SP 完成的天
    for i in 0..days {
        let i_us = i as usize;
        let day = sprint.start_date + Duration::days(i);
        let day_key = day.format("%Y-%m-%d").to_string();
        if let Some(sp) = daily_completed.get(&day_key) {
            cumulative += sp;
            last_progress_idx = i_us;
        }
        actual.push(TimeSeriesPoint {
            x: day_key,
            y: (total_sp - cumulative).max(0.0),
        });
    }

    // 3. Scope change 事件
    let scope_changes: Vec<ScopeChange> = sprint.scope_change_log.clone();

    // 4. 预测完成 SP (线性外推)
    let predicted_completion = if actual.len() >= 2 && last_progress_idx >= 1 {
        let last_y = actual[last_progress_idx].y;
        let prev_y = actual[last_progress_idx - 1].y;
        let daily_decrease = (prev_y - last_y).max(0.0);
        if daily_decrease > 0.0 {
            let days_f = days as f64;
            let lpi_f = last_progress_idx as f64;
            last_y + daily_decrease * (days_f - lpi_f)
        } else {
            last_y
        }
    } else {
        total_sp
    };

    // 5. on_track 判定: 完成进度 ≥ 时间进度 (考虑 0.8 缓冲)
    //   用 last_progress_idx 而非 actual.len()-1, 反映"今天"位置
    let day_progress = if days > 0 {
        last_progress_idx as f64 / days as f64
    } else {
        0.0
    };
    let completion_progress = if total_sp > 0.0 {
        cumulative / total_sp
    } else {
        1.0
    };
    let on_track = completion_progress >= day_progress * 0.8;

    let remaining_sp = actual.last().map(|p| p.y).unwrap_or(total_sp);
    let completed_sp = total_sp - remaining_sp;

    BurndownData {
        sprint: sprint.clone(),
        series: BurndownSeries { ideal, actual },
        scope_changes,
        summary: BurndownSummary {
            remaining_sp,
            completed_sp,
            completed_issues: completed_count,
            total_issues,
            predicted_completion_sp: predicted_completion,
            on_track,
        },
    }
}

/// WorkItem 简化版 (从 WorkItemQueryPort 拉)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedIssue {
    /// WorkItem ID
    pub workitem_id: Uuid,
    /// 完成时间
    pub completed_at: DateTime<Utc>,
    /// Story Point (可能未设置)
    pub story_points: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_sprint() -> SprintMeta {
        let start = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap();
        SprintMeta {
            sprint_id: Uuid::nil(),
            name: "Sprint 1".into(),
            start_date: start,
            end_date: end,
            total_sp: 100.0,
            scope_change_log: vec![],
        }
    }

    fn make_issue(day: u32, sp: f64) -> CompletedIssue {
        let completed_at = Utc.with_ymd_and_hms(2026, 9, day, 12, 0, 0).unwrap();
        CompletedIssue {
            workitem_id: Uuid::new_v4(),
            completed_at,
            story_points: Some(sp),
        }
    }

    #[test]
    fn test_ideal_line_linear_decrease() {
        let sprint = make_sprint();
        let bd = compute_burndown(&sprint, &[], 0.0, 0, 0);
        assert_eq!(bd.series.ideal.len(), 14);
        assert_eq!(bd.series.ideal[0].y, 100.0);
        assert!(bd.series.ideal[13].y < 1.0); // 接近 0
    }

    #[test]
    fn test_actual_line_cumulative_decrease() {
        let sprint = make_sprint();
        let issues = vec![make_issue(3, 20.0), make_issue(5, 30.0)];
        let bd = compute_burndown(&sprint, &issues, 50.0, 10, 2);
        // day 1: 100, day 3: 80, day 5: 50
        assert_eq!(bd.series.actual[0].y, 100.0);
        assert!((bd.series.actual[2].y - 80.0).abs() < 0.01);
        assert!((bd.series.actual[4].y - 50.0).abs() < 0.01);
        assert_eq!(bd.summary.completed_sp, 50.0);
        assert_eq!(bd.summary.completed_issues, 2);
    }

    #[test]
    fn test_on_track_detection() {
        let sprint = make_sprint();
        // 3 issues 完成: day 3 / 5 / 7 各 20 SP, 共 60 SP, 实际 avg 速度 60/(7-1)=10 SP/day
        // 理想 100/14=7.14 SP/day, 实际 ≥ 理想 → on_track
        let issues = vec![
            make_issue(3, 20.0),
            make_issue(5, 20.0),
            make_issue(7, 20.0),
        ];
        let bd = compute_burndown(&sprint, &issues, 60.0, 10, 3);
        assert!(
            bd.summary.on_track,
            "expected on_track, got off_track, summary={:?}",
            bd.summary
        );
    }

    #[test]
    fn test_off_track_detection() {
        let sprint = make_sprint();
        // 1 issue 完成: day 3 仅 5 SP, 实际 avg 速度 5/2=2.5 SP/day
        // 理想 100/14=7.14 SP/day, 实际远低 → off_track
        let issues = vec![make_issue(3, 5.0)];
        let bd = compute_burndown(&sprint, &issues, 5.0, 10, 1);
        assert!(
            !bd.summary.on_track,
            "expected off_track, got on_track, summary={:?}",
            bd.summary
        );
    }

    #[test]
    fn test_scope_change_propagates() {
        let mut sprint = make_sprint();
        sprint.scope_change_log.push(ScopeChange {
            at: Utc.with_ymd_and_hms(2026, 9, 5, 10, 0, 0).unwrap(),
            delta_sp: -20.0,
            reason: "Removed story".into(),
            new_total_sp: 80.0,
        });
        let bd = compute_burndown(&sprint, &[], 0.0, 0, 0);
        assert_eq!(bd.scope_changes.len(), 1);
        assert_eq!(bd.scope_changes[0].delta_sp, -20.0);
    }

    #[test]
    fn test_zero_total_sp() {
        let mut sprint = make_sprint();
        sprint.total_sp = 0.0;
        let bd = compute_burndown(&sprint, &[], 0.0, 0, 0);
        // 不报错, 全部 y=0
        assert!(bd.series.ideal.iter().all(|p| p.y == 0.0));
    }
}
