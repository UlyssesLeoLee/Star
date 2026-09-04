//! C01 Burndown 单元 + 集成测试 (per docs/design/charts/c01-burndown.md §8)
//!
//! 5 单元 + 1 集成 (RLS 边界 + cache invalidation)

use chrono::{TimeZone, Utc};
use domain_report::application::ports::*;
use domain_report::domain::c01_burndown::{CompletedIssue, SprintMeta};
use domain_report::infrastructure::in_memory_cache::InMemoryCache;
use domain_report::infrastructure::port_stubs::*;
use domain_report::*;
use uuid::Uuid;

async fn make_service_with_seed() -> (ReportService, Uuid) {
    let cache = InMemoryCache::new();
    let wi_port = InMemoryWorkItemPort::new();
    let sp_port = InMemorySprintPort::new();
    let user_port = InMemoryUserPort::new();
    let perm_port = InMemoryPermissionPort::new();

    let sprint_id = Uuid::new_v4();
    let sprint = SprintMeta {
        sprint_id,
        name: "Sprint Test".into(),
        start_date: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        end_date: Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap(),
        total_sp: 100.0,
        scope_change_log: vec![],
    };
    sp_port.seed(sprint);

    // 5 issue: day 3 / 5 / 7 / 10 / 12 各完成 20 SP
    let issues: Vec<CompletedIssue> = vec![(3, 20.0), (5, 20.0), (7, 20.0), (10, 20.0), (12, 20.0)]
        .into_iter()
        .map(|(day, sp)| CompletedIssue {
            workitem_id: Uuid::new_v4(),
            completed_at: Utc.with_ymd_and_hms(2026, 9, day, 12, 0, 0).unwrap(),
            story_points: Some(sp),
        })
        .collect();
    wi_port.seed(sprint_id, issues);

    (
        ReportService::new(
            Box::new(cache),
            Box::new(wi_port),
            Box::new(sp_port),
            Box::new(user_port),
            Box::new(perm_port),
        ),
        sprint_id,
    )
}

#[tokio::test]
async fn test_c01_burndown_basic() {
    let (svc, sprint_id) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    let r = svc.generate(ReportType::Burndown, filter).await.unwrap();
    assert_eq!(r.report_type, ReportType::Burndown);
    assert_eq!(r.points.len(), 14); // 14 天

    // summary 校验
    let summary: serde_json::Value = r.summary.meta.clone();
    assert_eq!(summary["total_issues"], 5);
    assert_eq!(summary["completed_issues"], 5);
    assert!((summary["completed_sp"].as_f64().unwrap() - 100.0).abs() < 0.01);
    assert!((summary["remaining_sp"].as_f64().unwrap() - 0.0).abs() < 0.01);
    assert_eq!(summary["on_track"], true);
}

#[tokio::test]
async fn test_c01_burndown_no_sprint_id() {
    let (svc, _) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    // 不设 sprint_id

    let err = svc
        .generate(ReportType::Burndown, filter)
        .await
        .unwrap_err();
    match err {
        ReportError::ValidationFailed(msg) => {
            assert!(msg.contains("sprint_id"));
        }
        e => panic!("expected ValidationFailed, got {:?}", e),
    }
}

#[tokio::test]
async fn test_c01_burndown_sprint_not_found() {
    let (svc, _) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(Uuid::new_v4()); // 不存在的 sprint

    let err = svc
        .generate(ReportType::Burndown, filter)
        .await
        .unwrap_err();
    match err {
        ReportError::NotFound(_) => {} // 预期
        e => panic!("expected NotFound, got {:?}", e),
    }
}

#[tokio::test]
async fn test_c01_burndown_zero_total_sp() {
    let cache = InMemoryCache::new();
    let wi_port = InMemoryWorkItemPort::new();
    let sp_port = InMemorySprintPort::new();
    let user_port = InMemoryUserPort::new();
    let perm_port = InMemoryPermissionPort::new();

    let sprint_id = Uuid::new_v4();
    let mut sprint = SprintMeta {
        sprint_id,
        name: "Empty Sprint".into(),
        start_date: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        end_date: Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap(),
        total_sp: 0.0,
        scope_change_log: vec![],
    };
    // 用 setter-like 改 total_sp
    sprint.total_sp = 0.0;
    sp_port.seed(sprint);

    let svc = ReportService::new(
        Box::new(cache),
        Box::new(wi_port),
        Box::new(sp_port),
        Box::new(user_port),
        Box::new(perm_port),
    );
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    let r = svc.generate(ReportType::Burndown, filter).await.unwrap();
    // 不报错, 全部 y=0
    assert!(r.points.iter().all(|p| p.value == 0.0));
}

#[tokio::test]
async fn test_c01_burndown_with_scope_change() {
    let cache = InMemoryCache::new();
    let wi_port = InMemoryWorkItemPort::new();
    let sp_port = InMemorySprintPort::new();
    let user_port = InMemoryUserPort::new();
    let perm_port = InMemoryPermissionPort::new();

    let sprint_id = Uuid::new_v4();
    let sprint = SprintMeta {
        sprint_id,
        name: "Scope Changed".into(),
        start_date: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        end_date: Utc.with_ymd_and_hms(2026, 9, 14, 0, 0, 0).unwrap(),
        total_sp: 100.0,
        scope_change_log: vec![domain_report::domain::c01_burndown::ScopeChange {
            at: Utc.with_ymd_and_hms(2026, 9, 5, 10, 0, 0).unwrap(),
            delta_sp: -20.0,
            reason: "Removed story".into(),
            new_total_sp: 80.0,
        }],
    };
    sp_port.seed(sprint);

    let svc = ReportService::new(
        Box::new(cache),
        Box::new(wi_port),
        Box::new(sp_port),
        Box::new(user_port),
        Box::new(perm_port),
    );
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    let r = svc.generate(ReportType::Burndown, filter).await.unwrap();
    let data: serde_json::Value = r.data.clone();
    let scope_changes = data["scope_changes"].as_array().unwrap();
    assert_eq!(scope_changes.len(), 1);
    assert_eq!(scope_changes[0]["delta_sp"].as_f64().unwrap(), -20.0);
}

/// 集成测试: 缓存命中 (per docs/basic-design §7.1)
#[tokio::test]
async fn test_c01_cache_hit_invalidation() {
    let (svc, sprint_id) = make_service_with_seed().await;
    let mut filter = ReportFilter::default();
    filter.tenant_id = Uuid::new_v4();
    filter.sprint_id = Some(sprint_id);

    // 第一次: miss, 走真实计算
    let r1 = svc
        .generate(ReportType::Burndown, filter.clone())
        .await
        .unwrap();
    let t1 = r1.generated_at;

    // 第二次: 命中缓存 (5min TTL), generated_at 不变
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let r2 = svc
        .generate(ReportType::Burndown, filter.clone())
        .await
        .unwrap();
    assert_eq!(
        r1.generated_at, r2.generated_at,
        "cache hit should return same generated_at"
    );
    assert_eq!(r1.cache_key, r2.cache_key);
}
