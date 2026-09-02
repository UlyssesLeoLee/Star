// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-api-rest` binary — STAR Developer REST API server
//!
//! CLI:
//! ```text
//! star-api-rest [--bind-addr <ADDR>]
//!   --bind-addr ADDR     # 监听地址 (默认 127.0.0.1:8081)
//!                        # 可通过 STAR_API_REST_BIND_ADDR 环境变量覆盖
//! ```
//!
//! 22 路由 stub 当前返 501 Not Implemented,
//! P2 阶段 (Phase M+) worker 子代理实装业务逻辑 (派前必先 `automation/dispatcher.py brief(...)`).

use std::net::SocketAddr;

use star_api_rest::build_router;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind_addr: SocketAddr = std::env::var("STAR_API_REST_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .parse()
        .expect("invalid bind addr");

    let app = build_router();

    info!(%bind_addr, "star-api-rest starting (skeleton v0.1, 22 routes stub)");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
