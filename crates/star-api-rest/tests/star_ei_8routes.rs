// SPDX-License-Identifier: MIT OR Apache-2.0
//! §14.9 Star-EI EX-01..08 8 路由 integration tests (per brief v0.53 §3)
//!
//! 8 路由 wire + 8 stub handler 实证:
//! - GET  /api/v1/exclusion/idempotency-keys  → 501 not_implemented (EX-01)
//! - POST /api/v1/exclusion/mutex              → 501 not_implemented (EX-02)
//! - POST /api/v1/exclusion/dispatch-locks     → 501 not_implemented (EX-03)
//! - POST /api/v1/exclusion/subagent-locks     → 501 not_implemented (EX-04)
//! - GET  /api/v1/exclusion/ui-state           → 501 not_implemented (EX-05)
//! - POST /api/v1/exclusion/tool-idem          → 501 not_implemented (EX-06)
//! - GET  /api/v1/exclusion/metrics            → 501 not_implemented (EX-07)
//! - POST /api/v1/exclusion/test-runs          → 501 not_implemented (EX-08)
//!
//! 8 stub handler 跨 session 占位, 跟 OAuth2 introspect 模式一致 (per brief v0.53 §3):
//! - 全部 501 not_implemented
//! - 真实落地跨 session 续 (per WBS §14.9 5 域 Lead 拍板, Mavis 临时代签 per 守门 #3 v2 + #14 v3)
//!
//! 守门 #5 v2: token / secret / idempotency_key 不入 log / println.
//! 守门 #6 v2: 4xx 错误不 retriable.
//! 守门 #25 v25: 集成测试不发真 HTTP (走 axum::Router::oneshot).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use star_api_rest::build_router;
use tower::ServiceExt;

/// 验证 §14.9 EX-01 路由 stub
#[tokio::test]
async fn integration_ex_01_idempotency_keys_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/exclusion/idempotency-keys")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-01 /exclusion/idempotency-keys must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 5 张新表 metadata 实证
    let tables = body["tables"].as_array().expect("tables must be array");
    assert_eq!(tables.len(), 5, "EX-01 should describe 5 new tables");
    let table_names: Vec<&str> = tables.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(table_names.contains(&"idempotency_keys"));
    assert!(table_names.contains(&"lease_log"));
    assert!(table_names.contains(&"advisory_lock_audit"));
    assert!(table_names.contains(&"idempotency_keys_archive"));
    assert!(table_names.contains(&"exclusion_policy_master"));
}

/// 验证 §14.9 EX-02 路由 stub (star-mutex)
#[tokio::test]
async fn integration_ex_02_mutex_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let body_json = serde_json::json!({
        "lock_name": "work-item:STAR-1024",
        "owner_id": "L0-TopAgent-001",
        "ttl_seconds": 30
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/exclusion/mutex")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-02 /exclusion/mutex must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 7 module metadata 实证
    let modules = body["modules"].as_array().expect("modules must be array");
    assert_eq!(modules.len(), 7, "EX-02 should describe 7 modules");
    let module_ids: Vec<&str> = modules.iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert!(module_ids.contains(&"M-08"));
    assert!(module_ids.contains(&"M-14"));
}

/// 验证 §14.9 EX-03 路由 stub (L0 DispatchLockManager)
#[tokio::test]
async fn integration_ex_03_dispatch_locks_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let body_json = serde_json::json!({
        "dispatch_id": "dispatch-001",
        "client_uuid": "550e8400-e29b-41d4-a716-446655440000",
        "business_hash": "sha256:abcdef",
        "domain": "player"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/exclusion/dispatch-locks")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-03 /exclusion/dispatch-locks must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 5 域列表
    let domains = body["domains"].as_array().expect("domains must be array");
    assert_eq!(domains.len(), 5, "5 域 per 守门 #3 v2");
    let domain_strs: Vec<&str> = domains.iter().map(|d| d.as_str().unwrap()).collect();
    assert!(domain_strs.contains(&"player"));
    assert!(domain_strs.contains(&"economy"));
    assert!(domain_strs.contains(&"match"));
    assert!(domain_strs.contains(&"social"));
    assert!(domain_strs.contains(&"admin"));
    // 验证双键 dedup strategy
    assert_eq!(body["dedup_strategy"]["primary_key"], "client_uuid");
    assert_eq!(body["dedup_strategy"]["secondary_key"], "business_hash");
}

/// 验证 §14.9 EX-04 路由 stub (L1 SubAgentLock)
#[tokio::test]
async fn integration_ex_04_subagent_locks_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let body_json = serde_json::json!({
        "subagent_id": "SA-01",
        "lock_key": "domain-mutex:work-item:STAR-1024",
        "domain": "economy",
        "lease_seconds": 60
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/exclusion/subagent-locks")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-04 /exclusion/subagent-locks must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 10 SA 编号
    let sa_ids = body["subagent_ids"]
        .as_array()
        .expect("subagent_ids must be array");
    assert_eq!(sa_ids.len(), 9, "9 SA 编号 SA-01..SA-09 per §14.9 EX-04");
    // 验证 L0 协调 L1↔L1 (per 守门 #13 a)
    assert!(
        body["l0_coordination"].as_str().unwrap().contains("L0"),
        "l0_coordination 字段必须含 L0 (per 守门 #13 a)"
    );
}

/// 验证 §14.9 EX-05 路由 stub (UI IdempotencyManager)
#[tokio::test]
async fn integration_ex_05_ui_state_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/exclusion/ui-state")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-05 /exclusion/ui-state must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 4 lib + 3 component metadata
    let libs = body["libs"].as_array().expect("libs must be array");
    assert_eq!(libs.len(), 4, "EX-05 should describe 4 ui libs");
    let components = body["components"]
        .as_array()
        .expect("components must be array");
    assert_eq!(components.len(), 3, "EX-05 should describe 3 components");
    // 验证 gm-console 集成目标
    assert_eq!(body["integration_target"], "gm-console AppShell");
}

/// 验证 §14.9 EX-06 路由 stub (16 tool 幂等改造)
#[tokio::test]
async fn integration_ex_06_tool_idem_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let body_json = serde_json::json!({
        "tool_name": "search_code",
        "idempotency_key": "STAR-1024-attempt-1",
        "domain": "match",
        "business_hash": "sha256:123456"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/exclusion/tool-idem")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-06 /exclusion/tool-idem must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 16 tool 列表
    let tools = body["tools"].as_array().expect("tools must be array");
    assert_eq!(tools.len(), 16, "16 MCP tool per §14.9 EX-06");
    // 验证 idempotency target table
    assert!(
        body["idempotency_target_table"]
            .as_str()
            .unwrap()
            .contains("idempotency_keys"),
        "target table must be idempotency_keys"
    );
    // 验证 middleware 路径
    assert!(
        body["middleware"]
            .as_str()
            .unwrap()
            .contains("idempotency.rs"),
        "middleware must be idempotency.rs"
    );
}

/// 验证 §14.9 EX-07 路由 stub (4 层统一可观测性)
#[tokio::test]
async fn integration_ex_07_metrics_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/exclusion/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-07 /exclusion/metrics must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 4 层 (per §14.9 D-03 拍板)
    let layers = body["layers"].as_array().expect("layers must be array");
    assert_eq!(layers.len(), 4, "4 层 per D-03 拍板 (UI/L0/L1/L2)");
    // 验证 5 Prometheus 指标
    let metrics = body["prometheus_metrics"]
        .as_array()
        .expect("prometheus_metrics must be array");
    assert_eq!(metrics.len(), 5, "5 Prometheus 指标 per EX-07 §3.7");
    // 验证 守门 #22 控制台不污染 main
    let components = body["components"]
        .as_array()
        .expect("components must be array");
    let console = components
        .iter()
        .find(|c| c["name"] == "console_server.py")
        .expect("console_server.py must be present");
    assert_eq!(
        console["per守门_#22_不污染_main"], true,
        "console_server.py 必须 per 守门 #22 不污染 main"
    );
}

/// 验证 §14.9 EX-08 路由 stub (集成测试 + 性能压测)
#[tokio::test]
async fn integration_ex_08_test_runs_returns_501_not_implemented_after_v0_53() {
    let app = build_router();
    let body_json = serde_json::json!({
        "scenario_id": "S-08",
        "test_kind": "pt",
        "concurrency": 1000,
        "domain": "admin"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/exclusion/test-runs")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "EX-08 /exclusion/test-runs must 501 not_implemented (跨 session 续)"
    );
    let body_bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["error"], "not_implemented");
    // 验证 8 想定シナリオ
    let scenarios = body["scenarios"]
        .as_array()
        .expect("scenarios must be array");
    assert_eq!(scenarios.len(), 8, "8 想定シナリオ S-01..S-08");
    // 验证 38 测试矩阵 (18 UT + 8 IT + 12 E2E + 1 1000 并发压测)
    assert_eq!(body["test_matrix"]["ut"], 18);
    assert_eq!(body["test_matrix"]["it"], 8);
    assert_eq!(body["test_matrix"]["e2e"], 12);
    assert_eq!(body["test_matrix"]["pt_concurrency"], 1000);
    // 验证 守门 #1 v25 单 crate 模式
    assert!(
        body["守门_#1_v25"].as_str().unwrap().contains("cargo test"),
        "守门 #1 v25 must be present"
    );
}

/// 验证 8 路由都注册在 router (不 panic)
#[test]
fn v0_53_8_routes_registered_in_router() {
    // 跟 v0.47 router_contains_expected_paths 同模式
    let _ = build_router();
}

/// 验证 8 路由 都返回 501 (跨 session 占位, 跟 OAuth2 introspect 同模式)
#[tokio::test]
async fn integration_v0_53_all_8_routes_return_501_not_implemented() {
    let test_cases = [
        ("GET", "/api/v1/exclusion/idempotency-keys", None),
        (
            "POST",
            "/api/v1/exclusion/mutex",
            Some(serde_json::json!({"lock_name": "test", "owner_id": "test"})),
        ),
        (
            "POST",
            "/api/v1/exclusion/dispatch-locks",
            Some(serde_json::json!({
                "dispatch_id": "test",
                "client_uuid": "00000000-0000-0000-0000-000000000000",
                "domain": "player"
            })),
        ),
        (
            "POST",
            "/api/v1/exclusion/subagent-locks",
            Some(serde_json::json!({
                "subagent_id": "SA-01",
                "lock_key": "test",
                "domain": "player"
            })),
        ),
        ("GET", "/api/v1/exclusion/ui-state", None),
        (
            "POST",
            "/api/v1/exclusion/tool-idem",
            Some(serde_json::json!({
                "tool_name": "search_code",
                "idempotency_key": "test",
                "domain": "player"
            })),
        ),
        ("GET", "/api/v1/exclusion/metrics", None),
        (
            "POST",
            "/api/v1/exclusion/test-runs",
            Some(serde_json::json!({
                "scenario_id": "S-01",
                "test_kind": "ut"
            })),
        ),
    ];

    assert_eq!(test_cases.len(), 8, "8 路由 stub case");

    for (method, path, body) in test_cases.iter() {
        let app = build_router();
        let method_str: &str = method;
        let path_str: &str = path;
        let mut req_builder = Request::builder().method(method_str).uri(path_str);
        let body_bytes = if let Some(b) = body {
            req_builder = req_builder.header("content-type", "application/json");
            serde_json::to_vec(b).unwrap()
        } else {
            Vec::new()
        };
        let resp = app
            .oneshot(req_builder.body(Body::from(body_bytes)).unwrap())
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_IMPLEMENTED,
            "{} {} must 501 not_implemented, got {}",
            method_str,
            path_str,
            resp.status()
        );
    }
}
