//! `star-mcp` Streamable HTTP transport (per 2025-06-27 MCP spec §1.2)
//!
//! Phase D.5+ 实装:
//! - HTTP POST `/` 接收 JSON-RPC 2.0 请求
//! - 返回 `text/event-stream` (SSE), 1 个 event 包含完整 JSON-RPC 响应
//! - 复用 `transport::handle` 的 3 标准方法 + Resources/Prompts 路由
//! - 基于 axum 0.8 (per 任务 brief 选 axum, token-OLU 简单性对齐)
//!
//! ## Streamable HTTP 协议概要
//!
//! - Client → Server: `POST /` with `Content-Type: application/json` (JSON-RPC 2.0 body)
//! - Server → Client: `Content-Type: text/event-stream` (SSE) — 每个 event 是一行 `data: <json>`
//! - 单一请求/响应(无 session): server 收到请求后立即 handle, 1 个 event 后关闭流
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 复用 `transport::JsonRpcRequest` / `JsonRpcSuccess` / `JsonRpcError`, 不重定义 JSON-RPC 协议
//! - 不实现 SSE 长连接推送 / session 重连 / `Last-Event-ID` (per 任务 brief "MVP 走通", 完整 spec 留 Phase D.6+)

#![warn(missing_docs)]

use std::convert::Infallible;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::post,
};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_stream::iter;

use crate::error::McpError;
use crate::transport::{
    JsonRpcError, JsonRpcErrorBody, JsonRpcRequest, JsonRpcSuccess, error_code, handle,
};

/// HTTP 监听地址(per 任务 brief 默认 localhost:8080)
pub(crate) const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

/// axum Router state (本阶段为空, 保留扩展点 for session 持久化等)
#[derive(Clone, Default)]
struct AppState {
    /// 占位字段: 未来 session / cache / config 等可通过 State 注入
    #[allow(dead_code)]
    _placeholder: (),
}

/// 启动 Streamable HTTP server(阻塞, 直到 listener 关闭)
///
/// 监听 `bind_addr` (e.g. `127.0.0.1:8080`), 处理 MCP POST 请求
pub(crate) async fn run_http_server(bind_addr: &str) -> Result<(), McpError> {
    let app = build_router();
    let listener = TcpListener::bind(bind_addr).await.map_err(|e| {
        McpError::new(
            crate::error::error_code::IO,
            format!("failed to bind to {bind_addr}: {e}"),
            "mcp",
            crate::error::ErrorSourceKind::External,
            true,
            Some(format!("check if {bind_addr} is free (port in use?)")),
        )
    })?;
    eprintln!("star-mcp: Streamable HTTP server listening on http://{bind_addr}/");
    eprintln!("star-mcp: POST JSON-RPC 2.0 requests to / (returns text/event-stream SSE)");
    eprintln!("star-mcp: GET / returns server info (no MCP requests on GET per 2025-06-27 spec)");

    axum::serve(listener, app)
        .await
        .map_err(|e| {
            McpError::new(
                crate::error::error_code::IO,
                format!("axum::serve error: {e}"),
                "mcp",
                crate::error::ErrorSourceKind::External,
                true,
                None,
            )
        })?;
    Ok(())
}

fn build_router() -> Router {
    Router::new()
        .route("/", post(handle_mcp_post).get(handle_mcp_get))
        // D.6+ 新增 (per 2025-06-27 spec §1.2):
        // - GET /events: server-push SSE 端点 (长连接)
        // - GET /events/reconnect: session 重连 (Last-Event-ID header)
        // - DELETE /resources/{id}: 资源删除 (per spec §3, 留 Phase D.7+ P2 缺口)
        .route("/events", axum::routing::get(handle_server_push))
        .route("/events/reconnect", axum::routing::get(handle_session_reconnect))
        .route("/resources/{id}", axum::routing::delete(handle_resource_delete))
        .with_state(AppState::default())
}

/// GET `/` 返回服务器信息(per 2025-06-27 spec, GET 仅用于 server 能力探测)
async fn handle_mcp_get() -> Response {
    let info = serde_json::json!({
        "name": "star-mcp",
        "version": "0.1.0",
        "transport": "streamable-http",
        "protocolVersion": "2025-06-27",
        "instructions": "POST JSON-RPC 2.0 requests to this endpoint. Responses are returned as Server-Sent Events (text/event-stream)."
    });
    (StatusCode::OK, [("content-type", "application/json")], info.to_string()).into_response()
}

/// POST `/` 处理 JSON-RPC 2.0 请求, 返回 SSE
async fn handle_mcp_post(State(_state): State<AppState>, body: Bytes) -> Response {
    // 1. 解析 JSON-RPC 2.0 body
    let raw = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => return sse_error_response(Value::Null, error_code::PARSE_ERROR, format!("invalid UTF-8: {e}")),
    };

    let req: JsonRpcRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => return sse_error_response(Value::Null, error_code::PARSE_ERROR, format!("parse error: {e}")),
    };

    // 2. 校验 jsonrpc 字段(per JSON-RPC 2.0 spec)
    if req.jsonrpc != "2.0" {
        return sse_error_response(
            req.id.clone(),
            error_code::INVALID_REQUEST,
            format!("unsupported jsonrpc version: {}", req.jsonrpc),
        );
    }

    // 3. 路由到 transport::handle(共享 dispatch 逻辑)
    let response_payload = match handle(req).await {
        Ok(JsonRpcSuccess { jsonrpc, id, result }) => {
            serde_json::json!({ "jsonrpc": jsonrpc, "id": id, "result": result }).to_string()
        }
        Err(JsonRpcError { jsonrpc, id, error }) => {
            serde_json::json!({ "jsonrpc": jsonrpc, "id": id, "error": error }).to_string()
        }
    };

    // 4. 包装为 SSE 响应: 1 个 event
    sse_single_event_response(response_payload)
}

/// 构造 SSE 响应: 1 个 event 后立即关闭流
fn sse_single_event_response(data: String) -> Response {
    let event = Event::default().data(data);
    let stream = iter(vec![Ok::<Event, Infallible>(event)]);
    let sse = Sse::new(stream).keep_alive(KeepAlive::default());

    // Sse::into_response() 默认 Content-Type = text/event-stream
    let mut response = sse.into_response();

    // 显式设置 headers(per 2025-06-27 spec, Cache-Control: no-cache, X-Accel-Buffering: no)
    let headers = response.headers_mut();
    headers.insert("cache-control", "no-cache".parse().unwrap());
    headers.insert("x-accel-buffering", "no".parse().unwrap());

    response
}

// =====================================================================
// D.6+ 新增 (per 2025-06-27 spec §1.2):
// - GET /events: server-push SSE 端点 (单向 push, 不需 client request)
// - GET /events/reconnect: session 重连 (Last-Event-ID header 续传未确认 events)
// - DELETE /resources/{id}: 资源删除 (per spec §3, mock 501 留 Phase D.7+ P2 缺口)
//
// 设计: 用 mpsc channel + Last-Event-ID header, 0 unsafe, 显式缺标 (4 P2/P3 缺口)
// 守门: 8/26 JST 缺标比错标安全, 不编造 UUID, 真实持久化留 Phase E+
// =====================================================================

/// GET /events server-push SSE 端点 (per 2025-06-27 spec §1.2)
///
/// 客户端发起 GET, server 单向 SSE push 主动通知, 不需 client request.
/// 第一次连接时分配 SessionId, 客户端断线重连时带 `Last-Event-ID` header.
async fn handle_server_push() -> Response {
    // 简化: 返回 1 个 demo event 后立即关闭 (per spec, 真实 server-push 是长连接 + mpsc drain)
    // Phase D.7+ 接入: 分配 SessionId + mpsc channel + KeepAlive 长连接
    let session_id = crate::d6_session::SessionStore::new_session_id();
    let event = crate::d6_session::ServerEvent {
        id: "evt-0".to_string(),
        category: "session_opened".to_string(),
        payload: serde_json::json!({
            "session_id": session_id,
            "info": "Phase D.6+ server-push endpoint, returns 1 demo event. Full long-lived push (mpsc + KeepAlive) lands in Phase D.7+."
        }),
        timestamp_ms: 0,
    };
    sse_event_with_id(&event)
}

/// GET /events/reconnect session 重连 (per 2025-06-27 spec §1.2)
///
/// 客户端带 `Last-Event-ID: evt-N` header 重连, server 续传 evt-N 之后未确认 events.
/// 当前实现: 单 event response, 真实持久化 session store 留 Phase D.7+.
async fn handle_session_reconnect(headers: axum::http::HeaderMap) -> Response {
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("evt-0")
        .to_string();
    let event = crate::d6_session::ServerEvent {
        id: format!("reconnect-{last_event_id}"),
        category: "session_reconnect".to_string(),
        payload: serde_json::json!({
            "last_event_id": last_event_id,
            "info": "Phase D.6+ session reconnect endpoint, returns 1 ack event. Full session store (HashMap<SessionId, SessionState>) lands in Phase D.7+."
        }),
        timestamp_ms: 0,
    };
    sse_event_with_id(&event)
}

/// DELETE /resources/{id} 资源删除 (per 2025-06-27 spec §3)
///
/// 当前: 501 Not Implemented, 真实 ResourcesHandler::delete 留 Phase D.7+ P2 缺口.
async fn handle_resource_delete(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        serde_json::json!({
            "error": "not_implemented",
            "resource_id": id,
            "phase": "D.6+",
            "todo": "Phase D.7+ will implement ResourcesHandler::delete in resources.rs (per AGENTS.md §7 待办 #2 缺 4)",
        })
        .to_string(),
    )
        .into_response()
}

/// 构造 SSE event response with id field (per 2025-06-27 spec §1.2)
fn sse_event_with_id(event: &crate::d6_session::ServerEvent) -> Response {
    let data = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
    // SSE event format: `id: <id>\ndata: <data>\n\n`
    let sse_body = format!("id: {}\ndata: {}\n\n", event.id, data);
    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "text/event-stream"),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
            (axum::http::HeaderName::from_static("x-accel-buffering"), "no"),
        ],
        sse_body,
    ).into_response()
}

/// 构造 SSE 错误响应(per JSON-RPC 2.0 spec, 错误也走响应)
fn sse_error_response(id: Value, code: i32, message: String) -> Response {
    let err_body = JsonRpcErrorBody { code, message, data: None };
    let payload = serde_json::to_string(&JsonRpcError { jsonrpc: "2.0", id, error: err_body })
        .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"internal serialization error"}}"#.to_string());
    sse_single_event_response(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode as AxStatus};
    use tower::util::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_http_post_initialize_returns_sse() {
        let app = build_router();
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(body.to_string())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        // SSE content-type
        let ct = response.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/event-stream"), "expected SSE, got: {ct}");
        let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        // SSE data 行含 initialize 响应
        assert!(s.contains("data: "));
        assert!(s.contains("\"protocolVersion\":\"2025-06-27\""));
        assert!(s.contains("\"capabilities\""));
        // 验证 capabilities 含 resources + prompts
        assert!(s.contains("\"resources\""));
        assert!(s.contains("\"prompts\""));
    }

    #[tokio::test]
    async fn test_http_post_resources_list() {
        let app = build_router();
        let body = r#"{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(body.to_string())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(s.contains("\"resources\""));
        // Phase E 4 核心 resource URI(per task brief): workspace/worktree/agent/decision
        assert!(s.contains("\"uri\":\"workspace://current\""));
        assert!(s.contains("\"uri\":\"worktree://"));
        assert!(s.contains("\"uri\":\"agent://"));
        assert!(s.contains("\"uri\":\"decision://"));
    }

    #[tokio::test]
    async fn test_http_post_prompts_list() {
        let app = build_router();
        let body = r#"{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(body.to_string())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        // Phase E: 5 prompts (submit, review, context, workflow, debug), not 0
        assert!(s.contains("\"prompts\""));
        assert!(s.contains("\"name\":\"submit\""));
        assert!(s.contains("\"name\":\"debug\""));
    }

    #[tokio::test]
    async fn test_http_post_invalid_json_returns_parse_error() {
        let app = build_router();
        let body = "this is not valid json";
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(body.to_string())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(s.contains("-32700"));
    }

    #[tokio::test]
    async fn test_http_get_returns_server_info() {
        let app = build_router();
        let req = Request::builder()
            .method("GET")
            .uri("/")
            .body(String::new())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let ct = response.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("application/json"));
    }

    // ========== D.6+ 新增 4 个 unit test (per 2025-06-27 spec §1.2) ==========

    /// GET /events server-push 返回 1 个 demo event + session_id
    #[tokio::test]
    async fn test_d6_server_push_returns_event_with_session_id() {
        let app = build_router();
        let req = Request::builder()
            .method("GET")
            .uri("/events")
            .body(String::new())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let ct = response.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/event-stream"), "expected SSE, got: {ct}");
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        // SSE event with id field per spec
        assert!(s.contains("id: "), "expected 'id:' field per spec, got: {s}");
        assert!(s.contains("data: "), "expected 'data:' field per spec");
        assert!(s.contains("session_opened"), "expected session_opened category");
        assert!(s.contains("sess-"), "expected session_id in payload");
    }

    /// GET /events/reconnect with Last-Event-ID header 续传
    #[tokio::test]
    async fn test_d6_session_reconnect_with_last_event_id_header() {
        let app = build_router();
        let req = Request::builder()
            .method("GET")
            .uri("/events/reconnect")
            .header("last-event-id", "evt-42")
            .body(String::new())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        // reconnect event id 含 Last-Event-ID
        assert!(s.contains("reconnect-evt-42"), "expected reconnect ack with last_event_id, got: {s}");
        assert!(s.contains("session_reconnect"), "expected session_reconnect category");
    }

    /// GET /events/reconnect 无 Last-Event-ID header 默认 evt-0
    #[tokio::test]
    async fn test_d6_session_reconnect_no_header_uses_default() {
        let app = build_router();
        let req = Request::builder()
            .method("GET")
            .uri("/events/reconnect")
            .body(String::new())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::OK);
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(s.contains("reconnect-evt-0"), "expected default evt-0, got: {s}");
    }

    /// DELETE /resources/{id} 返回 501 Not Implemented (Phase D.7+ todo)
    #[tokio::test]
    async fn test_d6_delete_resource_returns_501_not_implemented() {
        let app = build_router();
        // URI 含 `://` 在 axum path 解析会 404, 用 simple id (per spec resource id 可任意 opaque string)
        let req = Request::builder()
            .method("DELETE")
            .uri("/resources/workspace-current")
            .body(String::new())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), AxStatus::NOT_IMPLEMENTED);
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(s.contains("not_implemented"));
        assert!(s.contains("workspace-current"), "resource id should be in response");
        assert!(s.contains("Phase D.7+"));
    }
}
