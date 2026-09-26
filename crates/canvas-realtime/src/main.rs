//! canvas-realtime main: axum HTTP :8082 + 单独 WSS :8083 listener.
//!
//! 阶段 1: 健康检查 + echo WSS.
//! 阶段 2+: Yjs/yrs CRDT 集成 (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.3).
//!
//! 注: per deploy/canvas-game-k3s.yaml 两个端口分开 (8082 HTTP + 8083 WS),
//! 所以这里跑两个独立 listener, 不用 axum 单 router 跑双端口.

use canvas_realtime::{router as http_router, ServiceMetadata};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let http_port: u16 = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(ServiceMetadata::CANVAS_REALTIME.http_port);
    let ws_port: u16 = std::env::var("WS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(ServiceMetadata::CANVAS_REALTIME.ws_port);

    info!(
        service = ServiceMetadata::CANVAS_REALTIME.name,
        version = ServiceMetadata::CANVAS_REALTIME.version,
        http_port, ws_port,
        "canvas-realtime starting (阶段 1 skeleton)"
    );

    let http_app = http_router();
    let http_listener = TcpListener::bind(std::net::SocketAddr::from(([0, 0, 0, 0], http_port))).await?;
    let http_serve = tokio::spawn(async move {
        if let Err(e) = axum::serve(http_listener, http_app).await {
            tracing::error!("HTTP serve error: {e}");
        }
    });

    // 阶段 1: WS 端口仅 echo, 用 tokio-tungstenite 裸 acceptor (绕过 axum ws upgrade)
    let ws_listener = TcpListener::bind(std::net::SocketAddr::from(([0, 0, 0, 0], ws_port))).await?;
    let ws_serve = tokio::spawn(async move {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::accept_async;
        while let Ok((stream, _peer)) = ws_listener.accept().await {
            tokio::spawn(async move {
                let mut ws = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        tracing::warn!("ws accept failed: {e}");
                        return;
                    }
                };
                while let Some(msg) = ws.next().await {
                    match msg {
                        Ok(m) => {
                            if ws.send(m).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    });

    tokio::select! {
        _ = http_serve => warn!("HTTP serve task exited"),
        _ = ws_serve => warn!("WS serve task exited"),
    }

    warn!("canvas-realtime exited");
    Ok(())
}