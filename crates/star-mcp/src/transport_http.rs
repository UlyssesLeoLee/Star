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
        McpError::BadRequest(format!("failed to bind to {bind_addr}: {e}"))
    })?;
    eprintln!("star-mcp: Streamable HTTP server listening on http://{bind_addr}/");
    eprintln!("star-mcp: POST JSON-RPC 2.0 requests to / (returns text/event-stream SSE)");
    eprintln!("star-mcp: GET / returns server info (no MCP requests on GET per 2025-06-27 spec)");

    axum::serve(listener, app)
        .await
        .map_err(|e| McpError::BadRequest(format!("axum::serve error: {e}")))?;
    Ok(())
}

fn build_router() -> Router {
    Router::new()
        .route("/", post(handle_mcp_post).get(handle_mcp_get))
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
        assert!(s.contains("\"uri\":\"star://tools/get_issue\""));
        assert!(s.contains("\"uri\":\"star://tools/submit\""));
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
        let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(s.contains("\"prompts\":[]"));
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
}
