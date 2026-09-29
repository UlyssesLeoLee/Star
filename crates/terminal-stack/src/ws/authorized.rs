//! CYPHER STRUCTURE MANIFEST.
//! CREATE
//!   (f:File {name:"authorized.rs",type:"file",language:"rust"}),(m:Module {name:"authorized",type:"module",language:"rust"}),
//!   (s:Class {name:"AuthorizedWsHandlerState",type:"class",language:"rust"}),(a:Class {name:"WsAttachmentAuthorizer",type:"class",language:"rust"}),(e:Class {name:"WsAttachmentError",type:"class",language:"rust"}),(fr:Class {name:"AuthorizationFrame",type:"class",language:"rust"}),(sink:Class {name:"TerminalEventSink",type:"class",language:"rust"}),(session:Class {name:"WsTerminalSession",type:"class",language:"rust"}),
//!   (max_auth:Variable {name:"MAX_AUTH_FRAME_BYTES",type:"variable",language:"rust"}),(max_ws:Variable {name:"MAX_WS_MESSAGE_BYTES",type:"variable",language:"rust"}),(auth_timeout:Variable {name:"AUTH_FRAME_TIMEOUT",type:"variable",language:"rust"}),
//!   (new:Function {name:"AuthorizedWsHandlerState::new",type:"function",language:"rust"}),(auth:Function {name:"authorize_attachment",type:"function",language:"rust"}),(rt:Function {name:"router",type:"function",language:"rust"}),(wh:Function {name:"ws_handler",type:"function",language:"rust"}),(as:Function {name:"authorize_and_serve",type:"function",language:"rust"}),(rf:Function {name:"read_authorization_frame",type:"function",language:"rust"}),(pf:Function {name:"parse_authorization_frame",type:"function",language:"rust"}),(reg:Function {name:"WsHub::register_pane",type:"function",language:"rust"}),(session_new:Function {name:"WsTerminalSession::new",type:"function",language:"rust"}),(session_pane:Function {name:"WsTerminalSession::with_pane_id",type:"function",language:"rust"}),(session_sink:Function {name:"WsTerminalSession::with_sink",type:"function",language:"rust"}),(session_run:Function {name:"WsTerminalSession::run",type:"function",language:"rust"}),(tv:Function {name:"valid_authorization_frame_is_accepted",type:"function",language:"rust"}),(tw:Function {name:"wrong_or_oversized_authorization_frames_are_rejected",type:"function",language:"rust"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(s),(m)-[:CONTAINS]->(a),(m)-[:CONTAINS]->(e),(m)-[:CONTAINS]->(fr),(m)-[:CONTAINS]->(max_auth),(m)-[:CONTAINS]->(max_ws),(m)-[:CONTAINS]->(auth_timeout),(m)-[:CONTAINS]->(new),(m)-[:CONTAINS]->(auth),(m)-[:CONTAINS]->(rt),(m)-[:CONTAINS]->(wh),(m)-[:CONTAINS]->(as),(m)-[:CONTAINS]->(rf),(m)-[:CONTAINS]->(pf),(m)-[:CONTAINS]->(tv),(m)-[:CONTAINS]->(tw),
//!   (s)-[:HAS_METHOD]->(new),(a)-[:HAS_METHOD]->(auth),(wh)-[:CALLS]->(as),(wh)-[:USES]->(max_ws),(as)-[:CALLS]->(rf),(as)-[:CALLS]->(auth),(as)-[:CALLS]->(reg),(as)-[:CALLS]->(session_new),(as)-[:CALLS]->(session_pane),(as)-[:CALLS]->(session_sink),(as)-[:CALLS]->(session_run),(rf)-[:CALLS]->(pf),(rf)-[:USES]->(auth_timeout),(pf)-[:USES]->(max_auth),(tv)-[:CALLS]->(pf),(tw)-[:CALLS]->(pf),(new)-[:USES]->(sink);

//! Protected terminal WebSocket route with first-frame attachment authorization.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::{
    Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use thiserror::Error;
use tokio::time::timeout;
use uuid::Uuid;

use super::handler::{self, WsHandlerState};

const MAX_AUTH_FRAME_BYTES: usize = 2 * 1024;
const MAX_WS_MESSAGE_BYTES: usize = 1024 * 1024;
const AUTH_FRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// State for the protected terminal attachment route.
#[derive(Clone)]
pub struct AuthorizedWsHandlerState {
    terminal: WsHandlerState,
    authorizer: Arc<dyn WsAttachmentAuthorizer>,
    terminal_sink: Arc<dyn handler::TerminalEventSink>,
}

impl AuthorizedWsHandlerState {
    /// Construct a protected route state; the authorizer must consume the ticket atomically.
    pub fn new(
        terminal: WsHandlerState,
        authorizer: Arc<dyn WsAttachmentAuthorizer>,
        terminal_sink: Arc<dyn handler::TerminalEventSink>,
    ) -> Self {
        Self {
            terminal,
            authorizer,
            terminal_sink,
        }
    }
}

/// Application-owned validation and single-use consumption of a browser attachment ticket.
#[async_trait]
pub trait WsAttachmentAuthorizer: Send + Sync + 'static {
    /// Check the existing session, current ACL, and ticket binding before accepting the socket.
    async fn authorize_attachment(
        &self,
        session_id: Uuid,
        ticket: &str,
    ) -> Result<(), WsAttachmentError>;
}

/// Deliberately opaque handshake errors; ticket contents and authorization reasons are never sent.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WsAttachmentError {
    /// The first frame was absent, malformed, oversized, or not an authorization frame.
    #[error("invalid authorization frame")]
    InvalidFrame,
    /// The ticket, session, or current authorization was rejected.
    #[error("attachment rejected")]
    Rejected,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizationFrame {
    #[serde(rename = "type")]
    kind: String,
    ticket: String,
}

/// Router for task-card terminal sessions. Inject a current-ACL ticket authorizer and real PTY sink.
pub fn router() -> Router<AuthorizedWsHandlerState> {
    Router::new().route("/v1/terminal/{session_id}/connect", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AuthorizedWsHandlerState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    ws.max_message_size(MAX_WS_MESSAGE_BYTES)
        .max_frame_size(MAX_WS_MESSAGE_BYTES)
        .on_upgrade(move |socket| authorize_and_serve(socket, session_id, state))
}

async fn authorize_and_serve(
    mut socket: WebSocket,
    requested_session_id: String,
    state: AuthorizedWsHandlerState,
) {
    let Ok(session_id) = Uuid::parse_str(&requested_session_id) else {
        let _ = socket.send(Message::Close(None)).await;
        return;
    };
    let ticket = match read_authorization_frame(&mut socket).await {
        Ok(ticket) => ticket,
        Err(_) => {
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };
    if state
        .authorizer
        .authorize_attachment(session_id, &ticket)
        .await
        .is_err()
    {
        let _ = socket.send(Message::Close(None)).await;
        return;
    }

    // Registration and snapshot replay happen only after the ticket authorizer succeeds.
    state.terminal.hub.register_pane(session_id);
    handler::WsTerminalSession::new(socket, session_id.to_string(), state.terminal)
        .with_pane_id(session_id)
        .with_sink(state.terminal_sink)
        .run()
        .await;
}

async fn read_authorization_frame(socket: &mut WebSocket) -> Result<String, WsAttachmentError> {
    let received = timeout(AUTH_FRAME_TIMEOUT, socket.recv())
        .await
        .map_err(|_| WsAttachmentError::InvalidFrame)?
        .ok_or(WsAttachmentError::InvalidFrame)?
        .map_err(|_| WsAttachmentError::InvalidFrame)?;
    match received {
        Message::Text(text) => parse_authorization_frame(text.as_str()),
        _ => Err(WsAttachmentError::InvalidFrame),
    }
}

fn parse_authorization_frame(text: &str) -> Result<String, WsAttachmentError> {
    if text.len() > MAX_AUTH_FRAME_BYTES {
        return Err(WsAttachmentError::InvalidFrame);
    }
    let frame: AuthorizationFrame =
        serde_json::from_str(text).map_err(|_| WsAttachmentError::InvalidFrame)?;
    if frame.kind != "authorize" || frame.ticket.is_empty() || frame.ticket.len() > 1024 {
        return Err(WsAttachmentError::InvalidFrame);
    }
    Ok(frame.ticket)
}

#[cfg(test)]
mod tests {
    use super::{WsAttachmentError, parse_authorization_frame};

    #[test]
    fn valid_authorization_frame_is_accepted() {
        assert_eq!(
            parse_authorization_frame(r#"{"type":"authorize","ticket":"opaque-once"}"#),
            Ok("opaque-once".to_string())
        );
    }

    #[test]
    fn wrong_or_oversized_authorization_frames_are_rejected() {
        assert_eq!(
            parse_authorization_frame(r#"{"type":"stdin","data":"x"}"#),
            Err(WsAttachmentError::InvalidFrame)
        );
        assert_eq!(
            parse_authorization_frame(&format!(
                r#"{{"type":"authorize","ticket":"{}"}}"#,
                "x".repeat(1100)
            )),
            Err(WsAttachmentError::InvalidFrame)
        );
    }
}
