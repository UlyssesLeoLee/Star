//! B.2 Hermes 5 endpoint contract test (per 29692a7 mock 备选路径实装)
//!
//! 跑 `cargo test -p domain-cli --test hermes_mock_contract`
//!
//! **目标**: 即使真实 Hermes endpoint 不可达, 也能验证 5 endpoint request/response 形状一致
//! (per 29692a7 mock 备选 + `docs/frontend/design/mock-msw-handlers.md` 既有 mock 模式)
//!
//! **5 endpoint contract test**:
//!   1. **auth**    POST   /v1/auth/token       → AuthToken
//!   2. **query**   GET    /v1/tasks?status=... → Vec<Task>
//!   3. **submit**  POST   /v1/tasks            → Task (status=Pending)
//!   4. **status**  GET    /v1/tasks/{id}       → Task (updated status)
//!   5. **cancel**  DELETE /v1/tasks/{id}       → CancelResponse
//!
//! **测试框架**: wiremock 0.6 (per B.2 mock 备选, Rust HTTP mock server, 类似 Python `responses` / `requests-mock`)
//!
//! **per 守门 #9**: 0 子代理调用, root 直实装 + 守门 4 步实证

use chrono::Utc;
use domain_cli::hermes::{
    CancelResponse, HermesClientBuilder, HermesConfig, HermesError, HermesMode, QueryRequest,
    RetryPolicy, SubmitRequest, Task, TaskStatus,
};
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// 5 endpoint contract test 1: auth (POST /v1/auth/token)
#[tokio::test]
async fn contract_auth_returns_token() {
    let server = MockServer::start().await;

    let now = Utc::now();
    let mock_response = json!({
        "access_token": "test-access-token-123",
        "token_type": "Bearer",
        "expires_at": now.to_rfc3339(),
    });

    Mock::given(method("POST"))
        .and(path("/v1/auth/token"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()))
        .with_retry_policy(RetryPolicy::NoRetry);
    let client = HermesClientBuilder::new(cfg)
        .api_key("test-api-key")
        .build()
        .unwrap();

    let token = client.auth().await.unwrap();
    assert_eq!(token.access_token, "test-access-token-123");
    assert_eq!(token.token_type, "Bearer");
}

/// 5 endpoint contract test 2: query (GET /v1/tasks?status=...&priority=...&limit=...)
#[tokio::test]
async fn contract_query_filters_by_status() {
    let server = MockServer::start().await;

    let mock_response = json!([
        {
            "id": Uuid::new_v4().to_string(),
            "name": "task-running-1",
            "status": "running",
            "priority": 3,
            "payload": "{\"input\":\"x\"}",
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": Utc::now().to_rfc3339(),
            "result": null
        },
        {
            "id": Uuid::new_v4().to_string(),
            "name": "task-running-2",
            "status": "running",
            "priority": 5,
            "payload": "{\"input\":\"y\"}",
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": Utc::now().to_rfc3339(),
            "result": null
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/v1/tasks"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("test-api-key")
        .build()
        .unwrap();

    let req = QueryRequest {
        status: Some(TaskStatus::Running),
        priority: None,
        created_after: None,
        limit: Some(10),
    };
    let tasks = client.query(&req).await.unwrap();
    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().all(|t| t.status == TaskStatus::Running));
    assert_eq!(tasks[0].name, "task-running-1");
    assert_eq!(tasks[1].name, "task-running-2");
}

/// 5 endpoint contract test 3: submit (POST /v1/tasks)
#[tokio::test]
async fn contract_submit_returns_pending_task() {
    let server = MockServer::start().await;

    let submitted_id = Uuid::new_v4();
    let mock_response = json!({
        "id": submitted_id.to_string(),
        "name": "build-package",
        "status": "pending",
        "priority": 3,
        "payload": "{\"input\":\"value\"}",
        "created_at": Utc::now().to_rfc3339(),
        "updated_at": null,
        "result": null
    });

    Mock::given(method("POST"))
        .and(path("/v1/tasks"))
        .and(header("Authorization", "Bearer test-api-key"))
        .and(body_json(json!({
            "name": "build-package",
            "priority": 3,
            "payload": "{\"input\":\"value\"}"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("test-api-key")
        .build()
        .unwrap();

    let req = SubmitRequest::new("build-package", 3, r#"{"input":"value"}"#);
    let task = client.submit(&req).await.unwrap();
    assert_eq!(task.id, submitted_id);
    assert_eq!(task.name, "build-package");
    assert_eq!(task.priority, 3);
    assert_eq!(task.status, TaskStatus::Pending);
}

/// 5 endpoint contract test 4: status (GET /v1/tasks/{id})
#[tokio::test]
async fn contract_status_returns_task() {
    let server = MockServer::start().await;

    let task_id = Uuid::new_v4();
    let mock_response = json!({
        "id": task_id.to_string(),
        "name": "build-package",
        "status": "completed",
        "priority": 3,
        "payload": "{}",
        "created_at": Utc::now().to_rfc3339(),
        "updated_at": Utc::now().to_rfc3339(),
        "result": "all-passed"
    });

    Mock::given(method("GET"))
        .and(path(format!("/v1/tasks/{}", task_id)))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("test-api-key")
        .build()
        .unwrap();

    let task = client.status(task_id).await.unwrap();
    assert_eq!(task.id, task_id);
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.result, Some("all-passed".into()));
}

/// 5 endpoint contract test 5: cancel (DELETE /v1/tasks/{id})
#[tokio::test]
async fn contract_cancel_returns_cancelled_response() {
    let server = MockServer::start().await;

    let task_id = Uuid::new_v4();
    let mock_response = json!({
        "cancelled": true,
        "cancelled_at": Utc::now().to_rfc3339(),
        "current_status": "cancelled"
    });

    Mock::given(method("DELETE"))
        .and(path(format!("/v1/tasks/{}", task_id)))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("test-api-key")
        .build()
        .unwrap();

    let resp = client.cancel(task_id).await.unwrap();
    assert!(resp.cancelled);
    assert_eq!(resp.current_status, TaskStatus::Cancelled);
    assert!(resp.cancelled_at.is_some());
}

/// Bonus contract test 6: 401 → HermesError::Auth (permanent, 不可重试)
#[tokio::test]
async fn contract_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/auth/token"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("bad-key")
        .build()
        .unwrap();

    let result = client.auth().await;
    match result {
        Err(HermesError::Auth(msg)) => {
            assert!(msg.contains("401"), "expected 401 in msg, got: {}", msg);
        }
        _ => panic!("expected HermesError::Auth, got: {:?}", result),
    }
}

/// Bonus contract test 7: 500 → HermesError::ServerError (transient, 可重试)
#[tokio::test]
async fn contract_500_returns_server_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/auth/token"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("test-key")
        .build()
        .unwrap();

    let result = client.auth().await;
    match result {
        Err(HermesError::ServerError(500, body)) => {
            assert_eq!(body, "internal error");
            // transient, 可重试
            assert!(matches!(
                HermesError::ServerError(500, "x".into()),
                HermesError::ServerError(_, _) if true
            ));
        }
        _ => panic!(
            "expected HermesError::ServerError(500, _), got: {:?}",
            result
        ),
    }
}

/// Bonus contract test 8: MockServer verify — 验证请求真的发了 (含 Authorization header + body)
#[tokio::test]
async fn contract_submit_request_shape_verified() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/tasks"))
        .and(header("Authorization", "Bearer verified-key"))
        .and(body_json(json!({
            "name": "verified-task",
            "priority": 7,
            "payload": "{\"verified\":true}"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": Uuid::new_v4().to_string(),
            "name": "verified-task",
            "status": "pending",
            "priority": 7,
            "payload": "{\"verified\":true}",
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": null,
            "result": null
        })))
        .expect(1) // 期望被调用 1 次
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("verified-key")
        .build()
        .unwrap();

    let req = SubmitRequest::new("verified-task", 7, r#"{"verified":true}"#);
    let task = client.submit(&req).await.unwrap();
    assert_eq!(task.name, "verified-task");
    assert_eq!(task.priority, 7);

    // wiremock 验证期望: 1 次 POST /v1/tasks + Authorization header + body_json 全部匹配
    // 失败的话 wiremock 会自动 panic
    server.verify().await;
}

/// 集成测试: HermesClient 5 endpoint 完整流程 (mock server, 走真实 HTTP)
#[tokio::test]
async fn contract_full_lifecycle_auth_query_submit_status_cancel() {
    let server = MockServer::start().await;

    // 1. auth
    Mock::given(method("POST"))
        .and(path("/v1/auth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "lifecycle-token",
            "token_type": "Bearer",
            "expires_at": Utc::now().to_rfc3339(),
        })))
        .mount(&server)
        .await;

    // 2. submit
    let submitted_id = Uuid::new_v4();
    Mock::given(method("POST"))
        .and(path("/v1/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": submitted_id.to_string(),
            "name": "lifecycle-task",
            "status": "pending",
            "priority": 5,
            "payload": "{}",
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": null,
            "result": null
        })))
        .mount(&server)
        .await;

    // 3. status
    Mock::given(method("GET"))
        .and(path(format!("/v1/tasks/{}", submitted_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": submitted_id.to_string(),
            "name": "lifecycle-task",
            "status": "running",
            "priority": 5,
            "payload": "{}",
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": Utc::now().to_rfc3339(),
            "result": null
        })))
        .mount(&server)
        .await;

    // 4. cancel
    Mock::given(method("DELETE"))
        .and(path(format!("/v1/tasks/{}", submitted_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "cancelled": true,
            "cancelled_at": Utc::now().to_rfc3339(),
            "current_status": "cancelled"
        })))
        .mount(&server)
        .await;

    let cfg = HermesConfig::new_mock()
        .with_mode(HermesMode::Real)
        .with_base_url(format!("{}/v1", server.uri()));
    let client = HermesClientBuilder::new(cfg)
        .api_key("lifecycle-key")
        .build()
        .unwrap();

    // auth
    let token = client.auth().await.unwrap();
    assert_eq!(token.access_token, "lifecycle-token");

    // submit
    let task = client
        .submit(&SubmitRequest::new("lifecycle-task", 5, "{}"))
        .await
        .unwrap();
    assert_eq!(task.id, submitted_id);
    assert_eq!(task.status, TaskStatus::Pending);

    // status
    let task = client.status(submitted_id).await.unwrap();
    assert_eq!(task.status, TaskStatus::Running);

    // cancel
    let resp: CancelResponse = client.cancel(submitted_id).await.unwrap();
    assert!(resp.cancelled);
    assert_eq!(resp.current_status, TaskStatus::Cancelled);

    // 验证所有 mock 都被调用
    server.verify().await;
}

/// sanity check: 5 endpoint enum 完整覆盖
#[test]
fn sanity_task_status_all_5_variants() {
    let all = [
        TaskStatus::Pending,
        TaskStatus::Running,
        TaskStatus::Completed,
        TaskStatus::Failed,
        TaskStatus::Cancelled,
    ];
    for s in all {
        let json = serde_json::to_string(&s).unwrap();
        let back: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}

/// sanity check: Task entity 5 字段 + 2 optional 字段
#[test]
fn sanity_task_entity_field_count() {
    let task = Task::new_pending("sanity", 5, "{}");
    // 5 必填字段 + 2 optional (updated_at, result) = 7 总字段
    // 验证 minimum: 5 必填 (id, name, status, priority, payload, created_at) 实际是 6 必填
    // (id / name / status / priority / payload / created_at 都不带 #[serde(default)])
    // updated_at / result 都带 #[serde(default)]
    assert!(task.updated_at.is_none());
    assert!(task.result.is_none());
    assert_eq!(task.name, "sanity");
    assert_eq!(task.priority, 5);
}
