//! C02 Burnup Chart 真实实现 (per docs/design/charts/c02-burnup.md v1.0)
//!
//! Sprint 期内累积完成 SP 上升趋势 + Sprint 范围调整线 (stepAfter)
//! 复用 C01 70%: 数据源 + Port 共享, 算法差异 = 累积而非剩余

use crate::application::ports::{SprintQueryPort, WorkItemQueryPort};
use crate::domain::c01_burndown::{CompletedIssue, ScopeChange, SprintMeta};
use crate::{ReportError, ReportFilter, ReportPoint, ReportResult, ReportSummary, Trend};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Burnup data schema (与 frontend lib/chart-data-schema.ts BurnupData 同构)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnupData {
    /// Sprint 元数据
    pub sprint: SprintMeta,
    /// 累积完成线 + 范围阶梯线
    pub series: BurnupSeries,
    /// 范围变更记录
    pub scope_changes: Vec<ScopeChange>,
    /// 汇总统计
    pub summary: BurnupSummary,
}

/// Burnup 曲线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnupSeries {
    /// 累积完成 SP 曲线
    pub actual: Vec<TimeSeriesPoint>, // 累积完成
    /// Sprint 范围阶梯曲线 (stepAfter)
    pub scope: Vec<TimeSeriesPoint>, // 范围阶梯
}

/// 时间序列上的单个数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// ISO 日期
    pub x: String,
    /// 数值
    pub y: f64,
}

/// Burnup 汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnupSummary {
    /// 已完成 SP
    pub completed_sp: f64,
    /// 总 SP
    pub total_sp: f64,
    /// 完成比例 (0-1)
    pub completion_ratio: f64, // 0-1
}

/// 公开入口: 异步生成 Burnup Report
pub async fn generate(
    work_item_port: &dyn WorkItemQueryPort,
    sprint_port: &dyn SprintQueryPort,
    filter: &ReportFilter,
    report_id: Uuid,
) -> Result<ReportResult, ReportError> {
    // 1. 拉 Sprint 元数据
    let sprint_id = filter.sprint_id.ok_or_else(|| {
        ReportError::ValidationFailed("Burnup requires sprint_id in filter".into())
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

    // 3. 计算累积完成 SP
    let total_completed_sp: f64 = completed_issues.iter().filter_map(|i| i.story_points).sum();

    // 4. 构造 actual + scope + summary
    let burnup = compute_burnup(&sprint, &completed_issues, total_completed_sp);

    // 5. 转 ReportPoint (向后兼容)
    let points: Vec<ReportPoint> = burnup.series.actual.iter().enumerate().map(|(i, p)| ReportPoint {
        label: p.x.clone(),
        value: p.y,
        extra: serde_json::json!({"scope": burnup.series.scope.get(i).map(|x| x.y).unwrap_or(0.0)}),
    }).collect();

    // 6. 返回 ReportResult
    Ok(ReportResult {
        report_id,
        report_type: crate::ReportType::Burnup,
        points,
        data: serde_json::to_value(&burnup).map_err(|e| ReportError::Internal(e.to_string()))?,
        summary: ReportSummary {
            total: sprint.total_sp,
            trend: Trend::Up,
            anomalies: vec![],
            meta: serde_json::to_value(&burnup.summary)
                .map_err(|e| ReportError::Internal(e.to_string()))?,
        },
        generated_at: Utc::now(),
        cache_key: format!("burnup:{}:{}", filter.tenant_id, sprint_id),
    })
}

/// 纯函数: 计算 actual (累积完成) + scope (阶梯)
fn compute_burnup(
    sprint: &SprintMeta,
    completed_issues: &[CompletedIssue],
    total_completed_sp: f64,
) -> BurnupData {
    let days = (sprint.end_date - sprint.start_date).num_days() + 1;
    let total_sp = sprint.total_sp;

    // 1. Actual 线: 累积完成 (单调递增)
    let mut daily_completed: std::collections::BTreeMap<String, f64> =
        std::collections::BTreeMap::new();
    for issue in completed_issues {
        let day_key = issue.completed_at.format("%Y-%m-%d").to_string();
        *daily_completed.entry(day_key).or_insert(0.0) += issue.story_points.unwrap_or(0.0);
    }
    let mut cumulative = 0.0;
    let mut actual: Vec<TimeSeriesPoint> = Vec::new();
    for i in 0..days {
        let day = sprint.start_date + Duration::days(i);
        let day_key = day.format("%Y-%m-%d").to_string();
        if let Some(sp) = daily_completed.get(&day_key) {
            cumulative += sp;
        }
        actual.push(TimeSeriesPoint {
            x: day_key,
            y: cumulative,
        });
    }

    // 2. Scope 线: 阶梯式 (Sprint 范围, 变更时跳跃)
    let mut scope: Vec<TimeSeriesPoint> = Vec::new();
    let mut current_total = total_sp;
    // 简化: scope 一直 = total_sp (per day 0); 范围变更日跳跃
    let scope_changes_sorted: Vec<&ScopeChange> = {
        let mut sc = sprint.scope_change_log.iter().collect::<Vec<_>>();
        sc.sort_by_key(|c| c.at);
        sc
    };
    for i in 0..days {
        let day = sprint.start_date + Duration::days(i);
        let day_key = day.format("%Y-%m-%d").to_string();
        for sc in &scope_changes_sorted {
            if sc.at.format("%Y-%m-%d").to_string() == day_key {
                current_total = sc.new_total_sp;
            }
        }
        scope.push(TimeSeriesPoint {
            x: day_key,
            y: current_total,
        });
    }

    let completion_ratio = if total_sp > 0.0 {
        total_completed_sp / total_sp
    } else {
        0.0
    };

    BurnupData {
        sprint: sprint.clone(),
        series: BurnupSeries { actual, scope },
        scope_changes: sprint.scope_change_log.clone(),
        summary: BurnupSummary {
            completed_sp: total_completed_sp,
            total_sp,
            completion_ratio,
        },
    }
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
    fn test_burnup_cumulative_sum() {
        let sprint = make_sprint();
        // 完成 day 3, day 5, day 7 (即 9/3, 9/5, 9/7)
        // days = 14, 索引 i=0..14 对应 9/1..9/14
        // 9/3 完成 20 SP → i=2
        // 9/5 完成 30 SP → i=4
        // 9/7 完成 10 SP → i=6
        let issues = vec![
            make_issue(3, 20.0),
            make_issue(5, 30.0),
            make_issue(7, 10.0),
        ];
        let bu = compute_burnup(&sprint, &issues, 60.0);
        assert_eq!(bu.series.actual[0].y, 0.0); // 9/1
        assert_eq!(bu.series.actual[1].y, 0.0); // 9/2
        assert!((bu.series.actual[2].y - 20.0).abs() < 0.01); // 9/3
        assert!((bu.series.actual[3].y - 20.0).abs() < 0.01); // 9/4
        assert!((bu.series.actual[4].y - 50.0).abs() < 0.01); // 9/5
        assert!((bu.series.actual[5].y - 50.0).abs() < 0.01); // 9/6
        assert!((bu.series.actual[6].y - 60.0).abs() < 0.01); // 9/7
        assert_eq!(bu.summary.completed_sp, 60.0);
        assert!((bu.summary.completion_ratio - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_burnup_scope_step() {
        let mut sprint = make_sprint();
        // 9/5 10:00 scope change 100 → 80
        // 索引: 9/1=i=0, 9/2=i=1, ..., 9/4=i=3, 9/5=i=4
        // 变更发生在 9/5 同一天, day_key 比较只看日期, 所以 9/5 当天就是 80
        sprint.scope_change_log.push(ScopeChange {
            at: Utc.with_ymd_and_hms(2026, 9, 5, 10, 0, 0).unwrap(),
            delta_sp: -20.0,
            reason: "Removed story".into(),
            new_total_sp: 80.0,
        });
        let issues = vec![];
        let bu = compute_burnup(&sprint, &issues, 0.0);
        assert_eq!(bu.series.scope[0].y, 100.0); // 9/1
        assert_eq!(bu.series.scope[3].y, 100.0); // 9/4 (变更前)
        assert_eq!(bu.series.scope[4].y, 80.0); // 9/5 (变更当日)
        assert_eq!(bu.series.scope[13].y, 80.0); // 9/14
    }

    #[test]
    fn test_burnup_zero_total_sp() {
        let mut sprint = make_sprint();
        sprint.total_sp = 0.0;
        let bu = compute_burnup(&sprint, &[], 0.0);
        assert_eq!(bu.summary.completion_ratio, 0.0);
    }

    #[test]
    fn test_burnup_overshoot() {
        let sprint = make_sprint();
        // 完成 110 SP > total_sp 100
        let issues = vec![make_issue(5, 110.0)];
        let bu = compute_burnup(&sprint, &issues, 110.0);
        assert_eq!(bu.summary.completed_sp, 110.0);
        assert!(bu.summary.completion_ratio > 1.0);
    }

    #[test]
    fn test_burnup_no_changes() {
        let sprint = make_sprint();
        let bu = compute_burnup(&sprint, &[], 0.0);
        // scope 全程 100
        assert!(bu.series.scope.iter().all(|p| p.y == 100.0));
    }
}
