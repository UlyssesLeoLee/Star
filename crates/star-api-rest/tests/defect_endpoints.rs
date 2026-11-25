// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/tests/defect_endpoints.rs"})
MERGE (suite:Symbol {id:"crates/star-api-rest/tests/defect_endpoints.rs::defect_endpoints",kind:"test_suite"})
MERGE (self)-[:DEFINES]->(suite)
@cypher
*/
//! 缺陷 REST 契约测试（per DD §6.3 IT 阶梯）
//!
//! 覆盖：
//! - IT-DI-06a 缺已验证会话 → **401**（fail-closed，不做 header 兜底）
//! - IT-DI-06b 错误码 → HTTP 状态码映射（403 / 409 / 422）
//! - IT-DI-06c 列表分页元信息与降级标注
//! - IT-DI-06d 缺陷路由已挂进生产路由
//!
//! 这些是**不触达真实 DB** 的契约层测试（缺 PG 时降级 InMemory，
//! 响应体里的 `degraded` 字段如实标注，不冒充生产持久化）。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use star_api_rest::auth::AuthUser;
use star_api_rest::routes::defects;

const TENANT: Uuid = Uuid::from_u128(0x20000000_0000_4000_8000_000000000001);
const USER: Uuid = Uuid::from_u128(0x20000000_0000_4000_8000_000000000002);
const PROJECT: Uuid = Uuid::from_u128(0x20000000_0000_4000_8000_000000000003);

/// 已验证会话的 `AuthUser`（生产由 JWT middleware 注入）。
fn verified_user() -> AuthUser {
    AuthUser {
        user_id: USER,
        tenant_id: TENANT,
        roles: vec!["developer".to_string()],
        scope: "developer.read developer.write".to_string(),
        email: None,
        projects: vec![PROJECT],
    }
}

fn app() -> Router {
    defects::router_with_verified_state()
}

fn get_with_session(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .extension(verified_user())
        .body(Body::empty())
        .unwrap()
}

fn get_without_session(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .body(Body::empty())
        .unwrap()
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let resp = app.clone().oneshot(req).await.expect("oneshot");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 256 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, body)
}

// =====================================================================
// IT-DI-06a: 缺已验证会话 → 401（fail-closed）
// =====================================================================

/// 无 `AuthUser` extension → 401，且**不**因缺头而 500。
#[tokio::test]
async fn defect_list_without_verified_session_401() {
    let app = app();
    let (status, body) = send(
        &app,
        get_without_session(&format!("/defects?project_id={PROJECT}")),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("AUTH_REQUIRED"));
}

/// 缺会话时详情与写入同样 401。
#[tokio::test]
async fn defect_detail_without_verified_session_401() {
    let app = app();
    let (status, _) = send(
        &app,
        get_without_session(&format!("/defects/{}", Uuid::new_v4())),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// =====================================================================
// IT-DI-06c: 列表分页元信息 + 降级标注
// =====================================================================

/// 列表响应必须含分页元信息与降级标注（per G-16 显式降级）。
#[tokio::test]
async fn defect_list_returns_pagination_and_degradation_metadata() {
    let app = app();
    let (status, body) = send(
        &app,
        get_with_session(&format!("/defects?project_id={PROJECT}&page=2&page_size=10")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let data = &body["data"];
    assert!(data.is_object(), "响应应含 data 对象，实际: {body}");

    // 分页元信息
    assert_eq!(data["page"], json!(2));
    assert_eq!(data["page_size"], json!(10));
    assert!(data["total"].is_number(), "缺 total");
    assert!(data["items"].is_array(), "缺 items");

    // 降级标注：测试环境无 PG → degraded=true 且给出原因
    assert_eq!(data["degraded"], json!(true));
    let reason = data["degradation_reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("PostgreSQL") || reason.contains("未连接"),
        "降级原因应说明未连接 PG，实际: {reason}"
    );
}

/// `page_size` 超上限时被钳制到 200（防止无界查询）。
#[tokio::test]
async fn defect_list_clamps_page_size() {
    let app = app();
    let (status, body) = send(
        &app,
        get_with_session(&format!("/defects?project_id={PROJECT}&page_size=100000")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["page_size"], json!(200));
}

// =====================================================================
// IT-DI-06b: 错误码 → HTTP 状态码映射（per G-18）
// =====================================================================

/// 跨租户/跨 Project 拒绝 → **403**（不是 401，也不是 500）。
#[tokio::test]
async fn defect_cross_tenant_denied_maps_to_403() {
    use star_api_rest::error::RestError;

    // 直接构造错误并检查 HTTP 映射（避免依赖真实 PG 触发拒绝路径）
    let err = RestError::from(domain_defect::DefectError::CrossTenantDenied(
        Uuid::new_v4(),
        Uuid::new_v4(),
    ));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// 非法状态流转 → **409**。
#[tokio::test]
async fn defect_invalid_transition_maps_to_409() {
    use star_api_rest::error::RestError;

    let err = RestError::from(domain_defect::DefectError::InvalidTransition {
        from: "triage".to_string(),
        to: "closed".to_string(),
        reason: "禁止的状态流转".to_string(),
    });
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

/// 幂等键复用 → **409**。
#[tokio::test]
async fn defect_idempotency_key_used_maps_to_409() {
    use star_api_rest::error::RestError;

    let err = RestError::from(domain_defect::DefectError::IdempotencyKeyUsed(
        "key-1".to_string(),
    ));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

/// 验证版本不匹配 → **422**。
#[tokio::test]
async fn defect_verification_mismatch_maps_to_422() {
    use star_api_rest::error::RestError;

    let err = RestError::from(domain_defect::DefectError::VerificationMismatch(
        "fix_version 不匹配".to_string(),
    ));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// 跨 Project 拒绝 → **403**。
#[tokio::test]
async fn defect_cross_project_denied_maps_to_403() {
    use star_api_rest::error::RestError;

    let err = RestError::from(domain_defect::DefectError::CrossProjectDenied(
        Uuid::new_v4(),
        Uuid::new_v4(),
    ));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// 不存在的缺陷 → 404（未受影响，回归保护）。
#[tokio::test]
async fn defect_not_found_maps_to_404() {
    use star_api_rest::error::RestError;

    let err = RestError::from(domain_defect::DefectError::NotFound("x".to_string()));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// =====================================================================
// IT-DI-06d: 生产路由挂载回归
// =====================================================================

/// 缺陷路由必须挂在生产路由上（否则接线丢失）。
#[tokio::test]
async fn defect_routes_mounted_in_production_router() {
    use tower::ServiceExt;

    let app = star_api_rest::build_supplemental_routers();
    // 未认证时访问 /defects → 401（说明路由存在且受鉴权保护）
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/defects?project_id={PROJECT}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("oneshot");
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "缺陷路由应存在且 fail-closed（401），实际: {}",
        resp.status()
    );
}
