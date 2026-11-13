//! v0.61 WBS 9.6: RestError::into_response HTTP 状态映射 + 响应封装集成测试
//!
//! 覆盖:
//! 1. `RestError` code → HTTP 状态的完整映射表 (error.rs 巨型 match 所有分支)
//! 2. 响应体 JSON 形态 `{"error": {...6 字段...}}`
//! 3. response.rs: ResponseMeta / RestResponse / Pagination / PagedResponse

use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use star_api_rest::error::RestError;
use star_api_rest::response::{PagedResponse, Pagination, ResponseMeta, RestResponse};

fn err(code: &str) -> RestError {
    RestError {
        code: code.to_string(),
        message: "msg".to_string(),
        source_module: "test".to_string(),
        source_kind: "Test".to_string(),
        retriable: false,
        hint: "hint".to_string(),
    }
}

fn status_of(code: &str) -> StatusCode {
    err(code).into_response().status()
}

#[tokio::test]
async fn error_response_body_shape() {
    use axum::body::{to_bytes, Body};
    use tower::ServiceExt;
    // 用 RestError 作为 handler 返回值走真实 router 拿 bytes
    let app = axum::Router::new().route(
        "/err",
        axum::routing::get(|| async { err("POLICY_DENIED") }),
    );
    let resp = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/err")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "POLICY_DENIED");
    assert_eq!(v["error"]["message"], "msg");
    assert_eq!(v["error"]["source_module"], "test");
    assert_eq!(v["error"]["source_kind"], "Test");
    assert_eq!(v["error"]["retriable"], false);
    assert_eq!(v["error"]["hint"], "hint");
}

#[test]
fn not_implemented_maps_to_501() {
    assert_eq!(status_of("NOT_IMPLEMENTED"), StatusCode::NOT_IMPLEMENTED);
    assert_eq!(
        status_of("BATCH_NOT_IMPLEMENTED"),
        StatusCode::NOT_IMPLEMENTED
    );
}

#[test]
fn validation_codes_map_to_400() {
    assert_eq!(status_of("VALIDATION_FAILED"), StatusCode::BAD_REQUEST);
    assert_eq!(status_of("VALIDATION_RUN_FAILED"), StatusCode::BAD_REQUEST);
}

#[test]
fn not_found_codes_map_to_404() {
    for code in [
        "RESOURCE_NOT_FOUND",
        "WORKTREE_NOT_FOUND",
        "WORKTREE_REPO_NOT_REGISTERED",
        "FB_NOT_FOUND",
        "COMMENT_NOT_FOUND",
        "AGENT_NOT_FOUND",
        "THEME_NOT_FOUND",
        "BATCH_TASK_NOT_FOUND",
        "BATCH_RUN_NOT_FOUND",
        "BATCH_NODE_NOT_FOUND",
        "BATCH_NODE_TYPE_NOT_FOUND",
        "TENANT_NOT_FOUND",
        "IDENTITY_NOT_FOUND",
        "PERMISSION_NOT_FOUND",
        "PROJECT_NOT_FOUND",
    ] {
        assert_eq!(status_of(code), StatusCode::NOT_FOUND, "{code}");
    }
}

#[test]
fn unauthorized_code_maps_to_401() {
    assert_eq!(status_of("BATCH_UNAUTHENTICATED"), StatusCode::UNAUTHORIZED);
}

#[test]
fn forbidden_codes_map_to_403() {
    for code in [
        "POLICY_DENIED",
        "FB_PERMISSION_DENIED",
        "AGENT_WORKTREE_MISMATCH",
    ] {
        assert_eq!(status_of(code), StatusCode::FORBIDDEN, "{code}");
    }
}

#[test]
fn conflict_codes_map_to_409() {
    for code in [
        "WORKTREE_CONFLICT",
        "INTEGRATION_CONFLICT",
        "COMMENT_CONFLICT",
        "VL_CONFLICT",
        "FB_CONFLICT",
        "SCM_CONFLICT",
        "BATCH_TASK_NAME_CONFLICT",
        "BATCH_RUN_ALREADY_RUNNING",
        "AGENT_CONFLICT",
        "AGENT_ALREADY_EXISTS",
        "TENANT_CONFLICT",
        "TENANT_SLUG_EXISTS",
        "IDENTITY_CONFLICT",
        "IDENTITY_EMAIL_EXISTS",
        "IDENTITY_DEVICE_REVOKED",
        "PERMISSION_CONFLICT",
        "PROJECT_CONFLICT",
        "PROJECT_SLUG_EXISTS",
    ] {
        assert_eq!(status_of(code), StatusCode::CONFLICT, "{code}");
    }
}

#[test]
fn timeout_and_overloaded_codes() {
    assert_eq!(status_of("BATCH_NODE_TIMEOUT"), StatusCode::REQUEST_TIMEOUT);
    assert_eq!(
        status_of("BATCH_ENGINE_OVERLOADED"),
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[test]
fn unprocessable_entity_codes_map_to_422() {
    for code in [
        "FB_INVALID_STATE_TRANSITION",
        "FB_TARGET_UNRESOLVABLE",
        "FB_READ_ONLY",
        "FB_NOT_DELETABLE",
        "FB_MISSING_SUCCESSOR",
        "FB_CROSS_WORKTREE",
        "I_LOOP_GUARD_MISSING",
        "I_CREDENTIAL_MISSING",
        "VL_INVARIANT_VIOLATED",
        "SCM_NOT_FOUND",
        "SCM_PROVIDER_ERROR",
        "BATCH_INVALID_DAG_SCHEMA",
        "BATCH_DAG_CYCLE",
        "BATCH_INVALID_NODE_TYPE_CONFIG",
        "BATCH_INVALID_CRON",
        "BATCH_VALIDATION_FAILED",
        "AGENT_INVALID_TRANSITION",
        "TENANT_INVALID_STATE",
        "IDENTITY_INCOMPLETE_BINDING",
        "PERMISSION_INVALID_RULE",
        "PROJECT_INVALID_STATE",
    ] {
        assert_eq!(status_of(code), StatusCode::UNPROCESSABLE_ENTITY, "{code}");
    }
}

#[test]
fn internal_error_codes_map_to_500() {
    for code in [
        "INTERNAL",
        "BATCH_NODE_EXECUTION_FAILED",
        "BATCH_WORKER_LEASE_LOST",
        "BATCH_DATABASE_ERROR",
    ] {
        assert_eq!(status_of(code), StatusCode::INTERNAL_SERVER_ERROR, "{code}");
    }
}

#[test]
fn unknown_code_falls_back_to_500() {
    assert_eq!(
        status_of("SOMETHING_UNMAPPED"),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

// =====================================================================
// response.rs 封装
// =====================================================================

#[test]
fn response_meta_stub_shape() {
    let m = ResponseMeta::stub();
    assert_eq!(m.request_id, "req_stub");
    assert_eq!(m.version, "v1");
    // RFC 3339 时间戳可解析回 DateTime
    let parsed = chrono::DateTime::parse_from_rfc3339(&m.timestamp);
    assert!(parsed.is_ok(), "timestamp must be RFC3339: {}", m.timestamp);
}

#[test]
fn rest_response_ok_serde_roundtrip() {
    let r = RestResponse::ok(json!({"id": 1}));
    let json = serde_json::to_string(&r).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["data"]["id"], 1);
    assert_eq!(v["meta"]["request_id"], "req_stub");
    assert_eq!(v["meta"]["version"], "v1");
    assert!(v["meta"]["timestamp"].is_string());

    // typed 反序列化
    let parsed: RestResponse<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.data["id"], 1);
    assert_eq!(parsed.meta.version, "v1");
}

#[test]
fn pagination_serde_roundtrip() {
    let p = Pagination {
        page: 2,
        page_size: 20,
        total: 45,
        total_pages: 3,
        has_next: true,
    };
    let json = serde_json::to_string(&p).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["page"], 2);
    assert_eq!(v["page_size"], 20);
    assert_eq!(v["total"], 45);
    assert_eq!(v["total_pages"], 3);
    assert_eq!(v["has_next"], true);
    let parsed: Pagination = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.total, 45);
}

#[test]
fn paged_response_serde_roundtrip() {
    let r = PagedResponse {
        data: vec![json!({"a": 1}), json!({"a": 2})],
        meta: ResponseMeta::stub(),
        pagination: Pagination {
            page: 1,
            page_size: 10,
            total: 2,
            total_pages: 1,
            has_next: false,
        },
    };
    let json = serde_json::to_string(&r).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["data"].as_array().unwrap().len(), 2);
    assert_eq!(v["pagination"]["total"], 2);
    assert_eq!(v["pagination"]["has_next"], false);
    assert!(v["meta"]["timestamp"].is_string());

    let parsed: PagedResponse<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.data.len(), 2);
    assert_eq!(parsed.pagination.page_size, 10);
}
