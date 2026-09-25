//! `demo_server.rs` — 一键启动 terminal-stack demo server (per ULYS-232 P1-E).
//!
//! **目的**: 给前端 PR #98.5 + Playwright E2E 提供一个真实 wsServer 启动入口.
//! 同时给手动 smoke test 提供一个 `cargo run -p terminal-stack --bin demo_server` 命令行.
//!
//! **MVP v0 简化**:
//! - PTY 子 crate 留 P2 (per ULYS-200 description §5), demo server 用 Mock PTY:
//!   - 每 2s broadcast 一个 `output` 消息 (mock PTY stdout)
//!   - SnapshotLoader: 内存种子 5 行 demo (per AC-1)
//! - SplitTree per AC-3 broadcast 留 P2 集成阶段 (per ULYS-200 §5); demo 只测 Pane-level broadcast.
//!
//! **端到端集成测试** (per AC-1/2/3/4):
//! 1. client connect → 收到 `hello` (with seed scrollback) per AC-1
//! 2. client send `stdin` → server `on_stdin` sink 收到事件 (mock PTY sink) per AC-2
//! 3. server 每 2s broadcast `output` 字符 (per §3.3.3 AC-4)
//! 4. 10 client 并发 connect + write 100 lines → 0 lost/dup (per AC-4)
//!
//! 守门:
//! - #1 v15 cargo test 实证 (`cargo run --bin demo_server` 可起)
//! - #7 `unsafe_code = "forbid"` (workspace lint)
//! - #11 缺标比错标 (dep 全部来自 [workspace.dependencies] 或 crate-local)
//! - #13 L0 协调派生 (in-process hub + handler)

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::time::sleep as tokio_sleep;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use terminal_stack::persistence::TerminalStackPersistence;
use terminal_stack::scrollback_buffer::{ScrollbackLine, ScrollbackSource};
use terminal_stack::ws::handler::{WsHandlerState, WsTerminalEvent, WsTerminalSession};
use terminal_stack::ws::hub::WsHub;
use terminal_stack::ws::integration::{WsSnapshotLoader, WsSnapshotLoaderError};
use terminal_stack::ws::protocol::WsSnapshot;

/// Demo server bind address (overridable by `DEMO_SERVER_ADDR`).
fn bind_addr() -> String {
    std::env::var("DEMO_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing init (per AC-6 observability)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let bind = bind_addr();
    info!(addr = %bind, "demo_server starting (per ULYS-232 P1-E)");

    // -----------------------------------------------------------------
    // 1. Persistence (in-memory for demo; SQLite WAL optional via DEMO_PERSIST_DB)
    // -----------------------------------------------------------------
    let snapshot_loader: Arc<dyn WsSnapshotLoader> =
        if let Ok(db_path) = std::env::var("DEMO_PERSIST_DB") {
            let persistence = Arc::new(TerminalStackPersistence::open(&db_path)?);
            Arc::new(terminal_stack::ws::integration::WsPersistenceSnapshotLoader::new(persistence))
        } else {
            Arc::new(DemoSnapshotLoader::seed_demo())
        };

    // -----------------------------------------------------------------
    // 2. Mock PTY event sink (per AC-2 sink)
    // -----------------------------------------------------------------
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<WsTerminalEvent>(128);
    let sink = Arc::new(terminal_stack::ws::handler::NoopTerminalEventSink::new(
        event_tx,
    ));

    // -----------------------------------------------------------------
    // 3. WS handler state
    // -----------------------------------------------------------------
    let hub = WsHub::new();
    let state = Arc::new(WsHandlerState::new(
        hub,
        snapshot_loader.clone(),
        5000, // default_capacity_lines
    ));

    // -----------------------------------------------------------------
    // 4. Mock event log: every stdin/resize logged to trace
    //    (per AC-2 MVP v2 mock PTY logging)
    // -----------------------------------------------------------------
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                WsTerminalEvent::Stdin { pane_id, data } => {
                    info!(%pane_id, len = data.len(), "mock PTY stdin received");
                }
                WsTerminalEvent::Resize {
                    pane_id,
                    cols,
                    rows,
                } => {
                    info!(%pane_id, cols, rows, "mock PTY resize received");
                }
                WsTerminalEvent::Disconnect {
                    pane_id,
                    session_id,
                } => {
                    info!(%pane_id, %session_id, "client disconnected");
                }
            }
        }
    });

    // -----------------------------------------------------------------
    // 5. Demo echo loop: every 2s, broadcast a mock "output" message on a
    //    demo pane (per AC-4 broadcast test)
    // -----------------------------------------------------------------
    spawn_demo_echo_loop(state.hub.clone());

    // -----------------------------------------------------------------
    // 6. Routes
    // -----------------------------------------------------------------
    let app = Router::new()
        .route(
            "/v1/terminal/{runtime_id}/connect",
            get({
                let state = state.clone();
                let sink = sink.clone();
                move |ws: WebSocketUpgrade,
                      axum::extract::Path(runtime_id): axum::extract::Path<String>| {
                    let state = (*state).clone();
                    let sink = sink.clone();
                    async move {
                        ws.on_upgrade(move |socket: WebSocket| async move {
                            let session =
                                WsTerminalSession::new(socket, runtime_id, state).with_sink(sink);
                            session.run().await;
                        })
                        .into_response()
                    }
                }
            }),
        )
        .route("/healthz", get(healthz));

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    info!(addr = %bind, "demo_server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

// =====================================================================
// Demo helpers (per AC-1/2 mock PTY + broadcast tick)
// =====================================================================

/// Demo echo loop: every 2s, broadcast a mock "output" message on a
/// demo pane. Lets E2E tests verify server → client push without
/// needing a real PTY subprocess.
fn spawn_demo_echo_loop(hub: WsHub) {
    let demo_pane_id = Uuid::new_v4();
    hub.register_pane(demo_pane_id);

    tokio::spawn(async move {
        let mut seq: i64 = 0;
        loop {
            tokio_sleep(Duration::from_secs(2)).await;
            seq = seq.wrapping_add(1);
            let data = format!("[demo server tick {seq} — pane_id={demo_pane_id}]\r\n");
            if let Err(e) = hub.broadcast_output(demo_pane_id, data, seq) {
                warn!(error = ?e, "demo echo broadcast failed");
            }
        }
    });
}

// =====================================================================
// DemoSnapshotLoader (in-memory seed; for E2E without SQLite)
// =====================================================================

/// In-memory snapshot loader seeded with 5 demo lines per AC-1 demo.
struct DemoSnapshotLoader {
    seed_lines: Vec<ScrollbackLine>,
}

impl DemoSnapshotLoader {
    fn seed_demo() -> Self {
        let now = chrono::Utc::now();
        let lines: Vec<ScrollbackLine> = (0..5)
            .map(|i| ScrollbackLine {
                id: Uuid::new_v4(),
                timestamp: now,
                text: format!("[seed] demo line {i}"),
                source: ScrollbackSource::Stdout,
                byte_len: 26,
            })
            .collect();
        Self { seed_lines: lines }
    }
}

#[async_trait::async_trait]
impl WsSnapshotLoader for DemoSnapshotLoader {
    async fn load(
        &self,
        _session_id: Uuid,
        cli_session_id: &str,
        _last_seq: Option<i64>,
        _capacity_lines: usize,
    ) -> Result<Option<WsSnapshot>, WsSnapshotLoaderError> {
        // For demo we map cli_session_id -> a synthetic session_id and return seed lines.
        let session_id = Uuid::parse_str(cli_session_id).unwrap_or_else(|_| Uuid::new_v4());
        Ok(Some(WsSnapshot {
            session_id,
            panes: vec![session_id],
            total_bytes: self.seed_lines.iter().map(|l| l.byte_len as u64).sum(),
            server_time: chrono::Utc::now(),
            lines: self.seed_lines.clone(),
            from_seq: None,
        }))
    }
}
