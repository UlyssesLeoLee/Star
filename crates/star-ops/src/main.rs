// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-ops` binary 入口 (per OPS-BASIC-DESIGN §6.1)
//!
//! axum 0.8 server, 端口 8090 (per star-mcp 8080/8081 顺延)
//! MVP: 单实例, 2 health check 端点 + 8 REST stub

use star_ops::ops_api::{router, AppState};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tracing (复用 workspace tracing 依赖)
    tracing_subscriber_init();

    let port: u16 = std::env::var("STAR_OPS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);

    let app = router(AppState::new());

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!(port = port, "star-ops MVP-骨架 启动");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 简化版 tracing init (MVP-骨架, 实装阶段接 star-telemetry)
fn tracing_subscriber_init() {
    use tracing_subscriber::{fmt, EnvFilter};
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
}
