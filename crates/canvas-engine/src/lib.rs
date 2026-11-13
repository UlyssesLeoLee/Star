/*
@cypher schema=1 source_sha256=ad068c3d10c0804e211fd6c0b5f5ec39758f6033dc58d0f797b435d7f3ac60c1
MERGE (self:File {path:"crates/canvas-engine/src/lib.rs"})
MERGE (metadata:Type {id:"crates/canvas-engine/src/lib.rs::ServiceMetadata"})
MERGE (phase:Type {id:"crates/canvas-engine/src/lib.rs::Phase"})
MERGE (error:Type {id:"crates/canvas-engine/src/lib.rs::CanvasEngineError"})
MERGE (router:Symbol {id:"crates/canvas-engine/src/lib.rs::router",kind:"function"})
MERGE (healthz:Symbol {id:"crates/canvas-engine/src/lib.rs::healthz",kind:"function"})
MERGE (ready:Symbol {id:"crates/canvas-engine/src/lib.rs::ready",kind:"function"})
MERGE (version:Symbol {id:"crates/canvas-engine/src/lib.rs::version",kind:"function"})
MERGE (index_html:Symbol {id:"crates/canvas-engine/src/lib.rs::index_html",kind:"function"})
MERGE (services_status:Symbol {id:"crates/canvas-engine/src/lib.rs::services_status",kind:"function"})
MERGE (self)-[:DEFINES]->(metadata)
MERGE (self)-[:DEFINES]->(phase)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(router)
MERGE (self)-[:DEFINES]->(healthz)
MERGE (self)-[:DEFINES]->(ready)
MERGE (self)-[:DEFINES]->(version)
MERGE (self)-[:DEFINES]->(index_html)
MERGE (self)-[:DEFINES]->(services_status)
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/healthz",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/ready",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/version",method:"GET"})
MERGE (router)-[:EXPOSES]->(:Endpoint {path:"/api/v1/services",method:"GET"})
MERGE (router)-[:CALLS]->(healthz)
MERGE (router)-[:CALLS]->(ready)
MERGE (router)-[:CALLS]->(version)
MERGE (router)-[:CALLS]->(index_html)
MERGE (router)-[:CALLS]->(services_status)
MERGE (ready)-[:USES_TYPE]->(metadata)
MERGE (version)-[:USES_TYPE]->(metadata)
MERGE (services_status)-[:USES_TYPE]->(metadata)
@endcypher
*/
//! Star Canvas engine service metadata and HTTP router.
//!
//! The service currently exposes health, readiness, version, and service metadata endpoints.

#![warn(missing_docs)]

use axum::{http::header, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

/// Metadata for this Canvas service.
#[derive(Debug, Clone, Copy)]
pub struct ServiceMetadata {
    /// Service name.
    pub name: &'static str,
    /// Semantic service version.
    pub version: &'static str,
    /// HTTP listener port.
    pub http_port: u16,
    /// Current implementation phase.
    pub phase: Phase,
}

/// Service implementation phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Phase {
    /// Service scaffold and metadata endpoints are available.
    Skeleton,
    /// Product behavior has been implemented.
    Implemented,
}

impl ServiceMetadata {
    /// Metadata for the Canvas engine service.
    pub const CANVAS_ENGINE: Self = Self {
        name: "canvas-engine",
        version: env!("CARGO_PKG_VERSION"),
        http_port: 8080,
        phase: Phase::Skeleton,
    };
}

/// Errors reported by the Canvas engine service.
#[derive(Debug, thiserror::Error)]
pub enum CanvasEngineError {
    /// Product behavior is not implemented in this service scaffold.
    #[error("canvas-engine service behavior is not implemented")]
    NotImplemented,
}

/// Build the HTTP router for health and service metadata.
pub fn router() -> Router {
    Router::new()
        .route("/", get(index_html))
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/version", get(version))
        .route("/api/v1/services", get(services_status))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ready() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "ready": true,
        "phase": "skeleton",
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "version": ServiceMetadata::CANVAS_ENGINE.version,
        "phase": ServiceMetadata::CANVAS_ENGINE.phase,
    }))
}

async fn index_html() -> impl IntoResponse {
    let body = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Star Canvas Engine</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 720px; margin: 3rem auto; padding: 0 1rem; background: #0d1117; color: #e6edf3; }
    h1 { color: #58a6ff; }
    code, pre { font-family: ui-monospace, monospace; }
    a { color: #58a6ff; margin-right: 1rem; }
    button { margin: .75rem .5rem .75rem 0; padding: .5rem .8rem; cursor: pointer; }
    pre { min-height: 5rem; padding: 1rem; background: #161b22; border: 1px solid #30363d; white-space: pre-wrap; }
  </style>
</head>
<body>
  <h1>Star Canvas Engine</h1>
  <p>Service metadata and health endpoints for the Star Canvas platform.</p>
  <p>Phase: <strong id="phase">loading</strong> · Version: <strong id="version">loading</strong></p>
  <nav>
    <a href="/healthz">Health</a>
    <a href="/ready">Readiness</a>
    <a href="/version">Version</a>
    <a href="/api/v1/services">Service metadata</a>
  </nav>
  <section>
    <h2>Endpoint check</h2>
    <button onclick="check('/healthz')">GET /healthz</button>
    <button onclick="check('/ready')">GET /ready</button>
    <button onclick="check('/version')">GET /version</button>
    <button onclick="check('/api/v1/services')">GET /api/v1/services</button>
    <pre id="result" aria-live="polite">Select an endpoint to inspect its response.</pre>
  </section>
  <script>
    async function check(path) {
      const output = document.getElementById('result');
      output.textContent = `GET ${path} ...`;
      try {
        const response = await fetch(path);
        const body = await response.text();
        output.textContent = `${response.status} ${response.statusText}\n${body}`;
      } catch (error) {
        output.textContent = String(error);
      }
    }
    fetch('/version').then(response => response.json()).then(value => {
      document.getElementById('phase').textContent = value.phase;
      document.getElementById('version').textContent = value.version;
    }).catch(() => {});
  </script>
</body>
</html>"#;

    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], body)
}

async fn services_status() -> Json<serde_json::Value> {
    Json(json!({
        "service": ServiceMetadata::CANVAS_ENGINE.name,
        "role": "Star Canvas platform service",
        "phase": "skeleton",
        "endpoints": ["/healthz", "/ready", "/version", "/api/v1/services"],
    }))
}
