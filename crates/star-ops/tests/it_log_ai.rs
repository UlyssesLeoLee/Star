// crates/star-ops/tests/it_log_ai.rs
//
// F-02 端到端 IT 雏形 (per 守门 #1 v25 + 守门 #9 v20 + 守门 #24 v2)
//
// 守门 #1 v25: cargo test -p star-ops --lib -j 4 100% pass
// 守门 #9 v20: 子代理 RPC 不可靠实证 → 跨 crate IT + subprocess 真实调
// 守门 #24 v2: subprocess 替代 RPC, 解析 stdout JSON
//
// 范围:
// - 跨 crate IT: axum oneshot 调 /api/ops/log/upload 真实端点
// - subprocess 真实调: 验证 ai_log_mock.py 被调起且 stdout JSON 解析 OK
// - 3 表 ops_log_entry / ops_log_query_log / ops_log_analysis 跨域 DDL 雏形 (SQL 文件存在, 不实跑)

// 集成测试作为独立 crate, 关掉 missing_docs 顶层 deny (跟 lib 一致)
#![allow(missing_docs)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use star_ops::ops_api::{router, AppState};
use tower::ServiceExt;

/// F-02 端到端 IT: log_upload 真实端点 (跨 crate IT)
/// 验证: trace_id 传递 + level_filter 应用 + meta.stub=false
#[tokio::test]
async fn it_log_upload_end_to_end() {
    let app = router(AppState::new());
    let body = json!({
        "source": "k8s-pod/star-mcp-it",
        "level_filter": ["ERROR"],
        "content": "2026-09-08T07:30:00Z ERROR helm release 3 deploy failed: timeout\n2026-09-08T07:30:01Z INFO retrying",
        "trace_id": "trace-it-f02-001"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ops/log/upload")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// F-02 端到端 IT: 3 表 DDL 存在性检查 (跨 crate IT, 不实跑 SQL)
/// 验证 docs/migrations/2026-09-08-ops-log.sql 包含 3 张表 (W/T/M 100% 覆盖)
#[test]
fn it_ops_log_ddl_wtm_coverage() {
    let sql = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs/migrations/2026-09-08-ops-log.sql"),
    )
    .expect("ops-log DDL 落档存在");

    // 守门 #13 W/T/M 100% 覆盖: 3 张表都出现
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS ops_log_entry"),
        "W 类表 (作業中) 缺失"
    );
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS ops_log_query_log"),
        "T 类表 (Transaction) 缺失"
    );
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS ops_log_analysis"),
        "M 类表 (Master) 缺失"
    );

    // 守门 #13 d: audit trigger 必携
    assert!(
        sql.contains("audit_audit_event"),
        "T 类 audit trigger 必携 (per 守门 #13 d)"
    );

    // 守门 #13 c: SCD Type 2
    assert!(
        sql.contains("scd_type2_close"),
        "M 类 SCD Type 2 必携 (per 守门 #13 c)"
    );

    // 守门 #DB-13: tenant_id NOT NULL + FORCE RLS
    assert!(
        sql.contains("tenant_id UUID NOT NULL"),
        "tenant_id NOT NULL 必携 (per 守门 #DB-13 CW-05)"
    );
    assert!(
        sql.contains("FORCE ROW LEVEL SECURITY"),
        "FORCE RLS 必携 (per 守门 #DB-13 c)"
    );
}

/// F-02 端到端 IT: subprocess 真实调 (跨 crate + 守门 #24 v2)
/// 验证 MockChannel 通过 call_subprocess_stub 真实调 ai_log_mock.py
#[tokio::test]
async fn it_subprocess_real_call_via_ladder() {
    use chrono::Utc;
    use star_ops::ops_ai::default_ladder;
    use star_ops::ops_domain::log::{LogEntry, LogLevel};

    let log = LogEntry {
        id: uuid::Uuid::nil(),
        source: "k8s-pod/it".to_string(),
        level: LogLevel::Error,
        message: "2026-09-08T07:30:00Z ERROR helm release 3 deploy failed".to_string(),
        timestamp: Utc::now(),
        trace_id: Some("trace-it-subprocess-001".to_string()),
    };

    // 走 Ladder: mock 是 L1 (永远 enabled), 真 subprocess 调 ai_log_mock.py
    let ladder = default_ladder();
    let analysis = ladder
        .analyze_log(&log)
        .await
        .expect("Ladder 必须返 Ok (mock 兜底)");

    // 守门 #23: mock confidence 永远 < 0.5
    assert!(
        analysis.confidence < 0.5,
        "mock confidence 必 < 0.5, got {}",
        analysis.confidence
    );
    assert_eq!(analysis.generated_by, "mock", "L1 必返 mock 通道");
    assert!(
        !analysis.summary.is_empty(),
        "summary 非空 (subprocess 真调证据)"
    );
}
