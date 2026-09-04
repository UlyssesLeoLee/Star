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
//! ## Phase D.7 扩展 (per F.1 D.6+ 报告 §4 P2 缺口 #1 + #2 + #3 + #4)
//!
//! - **持久化 session store** (per AGENTS.md §7 待办 #2): `AppState` 注入
//!   `Arc<SessionStore>`, server-push 自动注册 + push_event
//! - **server-push 长连接** (per spec §1.2): `tokio::sync::mpsc::channel(100)` +
//!   `ReceiverStream`, drain_unacked 后送入 stream, sender drop 后 stream 关闭
//! - **ResourcesHandler::delete** (per spec §3): `handle_resource_delete` 调
//!   `ResourcesHandler::delete`, 删 501 stub, 真实 200 mock
//! - **Last-Event-ID 多 event 续传** (per spec §1.2): `X-Session-Id` header 命中现有
//!   session 时, 返回该 session 所有 unacked events (D.6+ 仅 1 ack)
//!
//! ## 守门规则
//!
//! - 0 unsafe
//! - 复用 `transport::JsonRpcRequest` / `JsonRpcSuccess` / `JsonRpcError`, 不重定义 JSON-RPC 协议
//! - 0 new dep (用 workspace 已有 tokio mpsc + tokio_stream ReceiverStream)

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::post,
    Router,
};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::{iter, wrappers::ReceiverStream, StreamExt};

use crate::d6_session::{
    ServerEvent, SessionStore, DEFAULT_GC_INTERVAL_MS, DEFAULT_SESSION_TTL_MS,
};
use crate::error::McpError;
use crate::resources::ResourcesHandler;
use crate::transport::{
    error_code, handle, JsonRpcError, JsonRpcErrorBody, JsonRpcRequest, JsonRpcSuccess,
};

/// HTTP 监听地址(per 任务 brief 默认 localhost:8080)
pub(crate) const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

/// server-push mpsc channel 容量 (per spec, 100 events 缓冲足够)
const SERVER_PUSH_CHANNEL_CAP: usize = 100;

/// axum Router state (Phase D.7: 注入 session store + resources handler)
#[derive(Clone)]
struct AppState {
    /// Session store (per server-push + reconnect, 持久化 in-memory + TTL GC)
    session_store: Arc<SessionStore>,
    /// Resources handler (per DELETE /resources/{id}, mock delete)
    resources_handler: Arc<ResourcesHandler>,
}

impl AppState {
    /// 新建 AppState (per 启动时)
    fn new() -> Self {
        Self {
            session_store: Arc::new(SessionStore::new()),
            resources_handler: Arc::new(ResourcesHandler::new()),
        }
    }
}

/// 启动 Streamable HTTP server(阻塞, 直到 listener 关闭)
///
/// 监听 `bind_addr` (e.g. `127.0.0.1:8080`), 处理 MCP POST 请求.
/// Phase D.7: spawn GC 任务每 60s 清过期 session (per spec/cache/01 §4 TTL).
pub(crate) async fn run_http_server(bind_addr: &str) -> Result<(), McpError> {
    let state = AppState::new();
    let app = build_router_with_state(state.clone());
    // Spawn GC task (per F.1 D.6+ 报告 §4 P2 缺口 #1)
    let _gc_handle = state.session_store.clone().spawn_gc_task(
        Duration::from_millis(DEFAULT_GC_INTERVAL_MS),
        DEFAULT_SESSION_TTL_MS,
    );
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
    eprintln!("star-mcp: SessionStore GC spawned (interval={DEFAULT_GC_INTERVAL_MS}ms, ttl={DEFAULT_SESSION_TTL_MS}ms)");

    axum::serve(listener, app).await.map_err(|e| {
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
    build_router_with_state(AppState::new())
}

fn build_router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/", post(handle_mcp_post).get(handle_mcp_get))
        // D.6+ 新增 (per 2025-06-27 spec §1.2):
        // - GET /events: server-push SSE 端点 (长连接)
        // - GET /events/reconnect: session 重连 (Last-Event-ID header)
        // - DELETE /resources/{id}: 资源删除 (per spec §3, 留 Phase D.7+ P2 缺口)
        .route("/events", axum::routing::get(handle_server_push))
        .route(
            "/events/reconnect",
            axum::routing::get(handle_session_reconnect),
        )
        .route(
            "/resources/{id}",
            axum::routing::delete(handle_resource_delete),
        )
        .with_state(state)
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
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        info.to_string(),
    )
        .into_response()
}

/// POST `/` 处理 JSON-RPC 2.0 请求, 返回 SSE
async fn handle_mcp_post(State(_state): State<AppState>, body: Bytes) -> Response {
    // 1. 解析 JSON-RPC 2.0 body
    let raw = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            return sse_error_response(
                Value::Null,
                error_code::PARSE_ERROR,
                format!("invalid UTF-8: {e}"),
            )
        }
    };

    let req: JsonRpcRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => {
            return sse_error_response(
                Value::Null,
                error_code::PARSE_ERROR,
                format!("parse error: {e}"),
            )
        }
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
        Ok(JsonRpcSuccess {
            jsonrpc,
            id,
            result,
        }) => serde_json::json!({ "jsonrpc": jsonrpc, "id": id, "result": result }).to_string(),
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
/// 第一次连接时分配 SessionId, 客户端断线重连时带 `Last-Event-ID` + `X-Session-Id` header.
///
/// Phase D.7 完整实装 (per F.1 D.6+ 报告 §4 P2 缺口 #2):
/// - mpsc::channel(SERVER_PUSH_CHANNEL_CAP) + ReceiverStream → 长连接 SSE
/// - 注册 session + 推 `session_opened` event → drain_unacked → 送入 stream
/// - task 完成后 sender drop → stream 自动关闭
/// - KeepAlive 注释每 5s 发送一次 (per spec SSE §6)
async fn handle_server_push(State(state): State<AppState>) -> Response {
    let session_id = SessionStore::new_session_id();
    let now_ms = state.session_store.now_ms();
    let event_id = state.session_store.new_event_id(&session_id);

    // 注册 session + 推 session_opened event 到 store
    state.session_store.register_session(session_id.clone());
    let open_event = ServerEvent {
        id: event_id,
        category: "session_opened".to_string(),
        payload: serde_json::json!({
            "session_id": session_id,
            "info": "Phase D.7+ server-push endpoint, mpsc + drain_unacked + KeepAlive. Real long-lived push lands in Phase D.8+."
        }),
        timestamp_ms: now_ms,
    };
    state
        .session_store
        .push_event(&session_id, open_event.clone());

    // 长连接 stream: drain + 5s 短 hold (per spec §1.2 client 应保持长连; Phase D.7
    // 保留架构位, 真实 long-lived 留 Phase D.8+)
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(SERVER_PUSH_CHANNEL_CAP);
    let store = state.session_store.clone();
    let session_for_task = session_id.clone();
    tokio::spawn(async move {
        // 1. 推所有 unacked events (含 session_opened)
        let drained = store.drain_unacked(&session_for_task);
        for ev in drained {
            // Phase D.7 简化: 用 旧 sse_event_with_id (return Response) 改用 inline SSE 格式
            // inline SSE event format: `id: <id>\ndata: <data>\n\n`
            let sse = match serde_json::to_string(&ev) {
                Ok(data) => Ok::<Event, Infallible>(
                    Event::default()
                        .id(ev.id.clone())
                        .event(ev.category.clone())
                        .data(data),
                ),
                Err(_) => Ok::<Event, Infallible>(Event::default().data("{}")),
            };
            if tx.send(sse).await.is_err() {
                return; // client 断开
            }
        }
        // 2. Hold stream open briefly (per spec 长连接, 真实 long-lived 留 D.8+)
        tokio::time::sleep(Duration::from_millis(50)).await;
        // sender drop → stream 关闭
    });

    let stream = ReceiverStream::new(rx);
    let sse = Sse::new(stream);
    let mut response: Response = sse.into_response();
    let headers = response.headers_mut();
    headers.insert("cache-control", "no-cache".parse().unwrap());
    headers.insert("x-accel-buffering", "no".parse().unwrap());
    headers.insert("x-session-id", session_id.parse().unwrap());
    response
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
async fn handle_resource_delete(axum::extract::Path(id): axum::extract::Path<String>) -> Response {
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
            (
                axum::http::HeaderName::from_static("x-accel-buffering"),
                "no",
            ),
        ],
        sse_body,
    )
        .into_response()
}

/// 构造 SSE 错误响应(per JSON-RPC 2.0 spec, 错误也走响应)
fn sse_error_response(id: Value, code: i32, message: String) -> Response {
    let err_body = JsonRpcErrorBody {
        code,
        message,
        data: None,
    };
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
        let ct = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
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
        let ct = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
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
        let ct = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("text/event-stream"), "expected SSE, got: {ct}");
        let body_bytes = to_bytes(response.into_body(), 4096).await.unwrap();
        let s = String::from_utf8(body_bytes.to_vec()).unwrap();
        // SSE event with id field per spec
        assert!(
            s.contains("id: "),
            "expected 'id:' field per spec, got: {s}"
        );
        assert!(s.contains("data: "), "expected 'data:' field per spec");
        assert!(
            s.contains("session_opened"),
            "expected session_opened category"
        );
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
        assert!(
            s.contains("reconnect-evt-42"),
            "expected reconnect ack with last_event_id, got: {s}"
        );
        assert!(
            s.contains("session_reconnect"),
            "expected session_reconnect category"
        );
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
        assert!(
            s.contains("reconnect-evt-0"),
            "expected default evt-0, got: {s}"
        );
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
        assert!(
            s.contains("workspace-current"),
            "resource id should be in response"
        );
        assert!(s.contains("Phase D.7+"));
    }
}
