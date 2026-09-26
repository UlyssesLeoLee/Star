//! domain-canvas main: axum HTTP server 监听 :8081.

use domain_canvas::router;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(domain_canvas::ServiceMetadata::DOMAIN_CANVAS.http_port);

    let app = router();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!(
        service = domain_canvas::ServiceMetadata::DOMAIN_CANVAS.name,
        version = domain_canvas::ServiceMetadata::DOMAIN_CANVAS.version,
        %addr,
        "domain-canvas starting (阶段 1 skeleton)"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    warn!("domain-canvas exited");
    Ok(())
}