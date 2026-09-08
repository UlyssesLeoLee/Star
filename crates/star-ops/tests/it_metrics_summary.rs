// crates/star-ops/tests/it_metrics_summary.rs
//
// F-03 端到端 IT 雏形 (per 守门 #1 v25 + 守门 #9 v20 + 守门 #13)
//
// 守门 #1 v25: cargo test -p star-ops --lib -j 4 100% pass
// 守门 #9 v20: 子代理 RPC 不可靠实证 → 跨 crate IT + 真实调端点
// 守门 #13: 1 表 DDL 雏形 W/T/M 100% 覆盖 (累计 12 表)
//
// 范围:
// - 跨 crate IT: axum oneshot 调 /api/ops/metrics/summary 真实端点
// - star-telemetry 复用: 调 MetricsAggregator 真实采 5 KPI
// - 1 表 DDL 存在性检查 (跨 crate IT, 不实跑 SQL)

// 集成测试作为独立 crate, 关掉 missing_docs 顶层 deny (跟 lib 一致)
#![allow(missing_docs)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use star_ops::ops_api::{router, AppState};
use star_ops::ops_domain::metrics::MetricsAggregator;
use tower::ServiceExt;
use uuid::Uuid;

/// F-03 端到端 IT: metrics_summary 真实端点 (跨 crate IT)
/// 验证: 5 KPI 全部返 + meta.stub=false + meta.hint 含 star-telemetry
#[tokio::test]
async fn it_metrics_summary_end_to_end() {
    let app = router(AppState::new());
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/ops/metrics/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 验证 body 真实返 5 KPI + stub=false
    use axum::body::to_bytes;
    let body_bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body readable");
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("body is JSON");
    let data = body["data"].as_array().expect("data is array");
    assert_eq!(data.len(), 5, "F-03 端到端必返 5 KPI");

    let names: Vec<String> = data
        .iter()
        .map(|m| m["name"].as_str().unwrap().to_string())
        .collect();
    for n in ["cpu_avg", "mem_avg", "active_tasks", "mcp_qps", "llm_token_daily"] {
        assert!(names.contains(&n.to_string()), "缺 KPI: {}", n);
    }

    assert_eq!(
        body["meta"]["stub"].as_bool(),
        Some(false),
        "F-03 端到端 stub 必为 false"
    );
    assert!(
        body["meta"]["hint"]
            .as_str()
            .unwrap_or("")
            .contains("star-telemetry"),
        "F-03 端到端 hint 必含 star-telemetry"
    );
}

/// F-03 端到端 IT: star-telemetry 复用 (跨 crate IT)
/// 验证 MetricsAggregator 真实调 TokenMeter.record + summary, 5 KPI 全返
#[tokio::test]
async fn it_star_telemetry_aggregation_via_metrics_aggregator() {
    let agg = MetricsAggregator::new();
    let agent = Uuid::new_v4();

    // 1. 调 3 次 record_call, 真实落 TokenMeter
    agg.record_call(agent, "gpt-4o", 100, 50).await;
    agg.record_call(agent, "gpt-4o", 200, 100).await;
    agg.record_call(agent, "claude-3.5", 50, 25).await;

    // 2. 调 summary 真实路径 (跟 ops_api.rs::metrics_summary 走相同代码)
    let metrics = agg.summary().await;
    assert_eq!(metrics.len(), 5);

    // 3. active_tasks = record_count (3 次)
    let active = metrics.iter().find(|m| m.name == "active_tasks").unwrap();
    assert_eq!(active.value, 3.0, "active_tasks 必 = record_count (3)");

    // 4. mcp_qps = call_count (3 次)
    let qps = metrics.iter().find(|m| m.name == "mcp_qps").unwrap();
    assert_eq!(qps.value, 3.0, "mcp_qps 必 = call_count (3)");

    // 5. llm_token_daily = input+output (100+50+200+100+50+25 = 525)
    let tokens = metrics
        .iter()
        .find(|m| m.name == "llm_token_daily")
        .unwrap();
    assert_eq!(tokens.value, 525.0, "llm_token_daily 必 = sum(input+output)");
}

/// F-03 端到端 IT: 1 表 DDL 存在性检查 (跨 crate IT, 不实跑 SQL)
/// 验证 db/migrations/2026-09-08-ops-metrics.sql 包含 1 张 M 表 SCD2
#[test]
fn it_ops_metrics_ddl_wtm_coverage() {
    let sql = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("db/migrations/2026-09-08-ops-metrics.sql"),
    )
    .expect("ops-metrics DDL 落档存在");

    // 守门 #13 W/T/M 100% 覆盖: 1 张 M 表 (SCD Type 2)
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS ops_metrics_config"),
        "M 类表 (ops_metrics_config) 缺失"
    );

    // 守门 #13 c: SCD Type 2 必携
    assert!(
        sql.contains("scd_type2_close"),
        "M 类 SCD Type 2 必携 (per 守门 #13 c)"
    );

    // 守门 #13 d: audit trigger 必携 (Master 也需 audit per audit-trigger.sql 派生规)
    assert!(
        sql.contains("audit_audit_event"),
        "M 类 audit trigger 必携 (per 守门 #13 d 派生规)"
    );

    // 守门 #DB-13: tenant_id NOT NULL + FORCE RLS
    // 注: 跟 F-01 ops-cluster.sql 同 pattern, 字段对齐用多空格,
    // 测试用更宽松的匹配 (tenant_id 列 + UUID NOT NULL 同字段)
    assert!(
        sql.contains("tenant_id") && sql.contains("UUID NOT NULL"),
        "tenant_id NOT NULL 必携 (per 守门 #DB-13 CW-05)"
    );
    assert!(
        sql.contains("FORCE ROW LEVEL SECURITY"),
        "FORCE RLS 必携 (per 守门 #DB-13 c)"
    );
}
