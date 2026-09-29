// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"main.rs",type:"file",language:"rust"}),(m:Module {name:"main",type:"module",language:"rust"}),
//!   (mn:Function {name:"main",type:"function"}),(jv:Function {name:"JwtConfig::verifier_from_env",type:"function"}),(pg:Function {name:"PgConfig::from_env",type:"function"}),(cp:Function {name:"connect_pool",type:"function"}),(sn:Function {name:"GroupApiState::new",type:"function"}),(rp:Class {name:"PgGroupAppRegistryProvider",type:"class"}),(pn:Function {name:"PgGroupAppRegistryProvider::new",type:"function"}),(br:Function {name:"build_group_router",type:"function"}),(lb:Function {name:"TcpListener::bind",type:"function"}),(sv:Function {name:"axum::serve",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(mn),(mn)-[:CALLS]->(jv),(mn)-[:CALLS]->(pg),(mn)-[:CALLS]->(cp),(mn)-[:CALLS]->(sn),(mn)-[:CALLS]->(pn),(mn)-[:CALLS]->(br),(mn)-[:CALLS]->(lb),(mn)-[:CALLS]->(sv),(rp)-[:HAS_METHOD]->(pn);
//! Production Worktree Group REST API server.
//!
//! The legacy build_router remains available to existing tests and local route
//! previews. The production binary exposes only authenticated Group API routes.

use std::{net::SocketAddr, sync::Arc};

use star_api_rest::{
    auth::JwtConfig,
    group_api::{GroupApiState, PgGroupAppRegistryProvider, build_group_router},
};
use star_pg_adapter::{PgConfig, connect_pool};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind_addr: SocketAddr = std::env::var("STAR_API_REST_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .parse()
        .expect("invalid bind addr");

    let jwt = Arc::new(JwtConfig::verifier_from_env().map_err(|error| {
        anyhow::anyhow!("JWT verifier configuration invalid ({})", error.code())
    })?);
    let pg_config = PgConfig::from_env()
        .map_err(|error| anyhow::anyhow!("PostgreSQL configuration invalid ({})", error.code()))?;
    let pool = connect_pool(&pg_config)
        .await
        .map_err(|error| anyhow::anyhow!("PostgreSQL connection failed ({})", error.code()))?;

    let group_app_registry = Arc::new(PgGroupAppRegistryProvider::new(
        pool.clone(),
        PgGroupAppRegistryProvider::HOST_API_VERSION,
    ));
    let app = build_group_router(
        GroupApiState::new(jwt, pool).with_group_app_registry(group_app_registry),
    );
    info!(%bind_addr, "star-api-rest Worktree Group API starting");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
