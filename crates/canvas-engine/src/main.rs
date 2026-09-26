//! canvas-engine main: axum HTTP server 监听 :8080 (per deploy/canvas-game-k3s.yaml).
//!
//! 阶段 1: 健康检查 + 服务版本 + 占位 endpoint. 0 业务.
//! 阶段 2+: 落地 跨 5 domain 共享 API (per SRS-STAR-CANVAS-GAME-001 v0.1 §4.1).

use canvas_engine::router;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing init (per workspace lint + 守门 #13 a L0 协调)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(canvas_engine::ServiceMetadata::CANVAS_ENGINE.http_port);

    let app = router();

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!(
        service = canvas_engine::ServiceMetadata::CANVAS_ENGINE.name,
        version = canvas_engine::ServiceMetadata::CANVAS_ENGINE.version,
        %addr,
        "canvas-engine starting (阶段 1 skeleton)"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    warn!("canvas-engine exited");
    Ok(())
}