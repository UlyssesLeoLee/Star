/*
@cypher schema=1 source_sha256=8962094c15875cdbbb4fd81c1ff4aaf3b4a529b811cbeb04d5b8958d125049e7
MERGE (self:File {path:"crates/canvas-engine/src/main.rs"})
MERGE (main:Symbol {id:"crates/canvas-engine/src/main.rs::main",kind:"function"})
MERGE (service:Type {id:"crates/canvas-engine/src/lib.rs::ServiceMetadata"})
MERGE (router:Symbol {id:"crates/canvas-engine/src/lib.rs::router",kind:"function"})
MERGE (self)-[:DEFINES]->(main)
MERGE (self)-[:IMPORTS]->(:File {path:"crates/canvas-engine/src/lib.rs"})
MERGE (main)-[:CALLS]->(router)
MERGE (main)-[:USES_TYPE]->(service)
@endcypher
*/
//! Star Canvas engine HTTP server; the deployment manifest configures port 8080.
//!
//! Serves health, readiness, version, and service metadata.

use canvas_engine::router;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing init (per workspace lint + 守门 #13 a L0 协调)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
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
