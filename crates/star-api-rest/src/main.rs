// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=916d83dc746dcb557dcc14b093dc75fc73d562bb3595070b5044ce7646ef76bf
MERGE (self:File {path:"crates/star-api-rest/src/main.rs"})
MERGE (main:Symbol {id:"crates/star-api-rest/src/main.rs::main",kind:"function"})
MERGE (store:Type {id:"bi_owner_runtime::PgTaskEventQueryStore",kind:"struct"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQueryPort",kind:"trait"})
MERGE (state:Type {id:"crates/star-api-rest/src/group_api.rs::GroupApiState",kind:"struct"})
MERGE (install:Symbol {id:"crates/star-api-rest/src/group_api.rs::GroupApiState.with_bi_task_event_query_port",kind:"method"})
MERGE (self)-[:DEFINES]->(main)
MERGE (main)-[:USES_TYPE]->(store)
MERGE (main)-[:USES_TYPE]->(port)
MERGE (main)-[:CALLS]->(install)
MERGE (install)-[:USES_TYPE]->(port)
MERGE (state)-[:HAS_METHOD]->(install)
@endcypher
*/
//! Cypher structural manifest.
//! CREATE
//!   (f:File {name:"main.rs",type:"file",language:"rust"}),(m:Module {name:"main",type:"module",language:"rust"}),
//!   (mn:Function {name:"main",type:"function"}),(jv:Function {name:"JwtConfig::verifier_from_env",type:"function"}),(pg:Function {name:"PgConfig::from_env",type:"function"}),(cp:Function {name:"connect_pool",type:"function"}),(sn:Function {name:"GroupApiState::new",type:"function"}),(rp:Class {name:"PgGroupAppRegistryProvider",type:"class"}),(pn:Function {name:"PgGroupAppRegistryProvider::new",type:"function"}),(br:Function {name:"build_group_router",type:"function"}),(lb:Function {name:"TcpListener::bind",type:"function"}),(sv:Function {name:"axum::serve",type:"function"}),
//!   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(mn),(mn)-[:CALLS]->(jv),(mn)-[:CALLS]->(pg),(mn)-[:CALLS]->(cp),(mn)-[:CALLS]->(sn),(mn)-[:CALLS]->(pn),(mn)-[:CALLS]->(br),(mn)-[:CALLS]->(lb),(mn)-[:CALLS]->(sv),(rp)-[:HAS_METHOD]->(pn);
//! CYPHER STRUCTURAL MANIFEST ADDENDUM
//! MATCH (mn:Function {name:"main",type:"function"});
//! CREATE (queryStore:Class {name:"PgTaskEventQueryStore",type:"class",language:"rust"}),(queryPort:Interface {name:"TaskEventQueryPort",type:"interface",language:"rust"});
//! CREATE (mn)-[:CALLS]->(queryStore),(queryStore)-[:IMPLEMENTS]->(queryPort);
//! Production Worktree Group REST API server.
//!
//! The legacy build_router remains available to existing tests and local route
//! previews. The production binary exposes only authenticated Group API routes.

use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use bi_owner_runtime::{PgTaskEventQueryStore, TaskEventQueryPort};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use star_api_rest::{
    auth::JwtConfig,
    group_api::{build_group_router, GroupApiState, PgGroupAppRegistryProvider},
};
use star_pg_adapter::{connect_pool, PgConfig};
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

    let bi_task_event_query_port: Option<Arc<dyn TaskEventQueryPort>> =
        match std::env::var("BI_DATABASE_URL") {
            Ok(database_url) => {
                if database_url.trim().is_empty() {
                    return Err(anyhow::anyhow!("BI_DATABASE_URL is empty"));
                }
                let connect_options = PgConnectOptions::from_str(&database_url)
                    .map_err(|_| anyhow::anyhow!("BI_DATABASE_URL is invalid"))?;
                let bi_pool = PgPoolOptions::new()
                    .max_connections(2)
                    .acquire_timeout(Duration::from_secs(2))
                    .connect_lazy_with(connect_options);
                Some(Arc::new(PgTaskEventQueryStore::new(bi_pool)))
            }
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(anyhow::anyhow!("BI_DATABASE_URL must be valid UTF-8"));
            }
        };

    let group_app_registry = Arc::new(PgGroupAppRegistryProvider::new(
        pool.clone(),
        PgGroupAppRegistryProvider::HOST_API_VERSION,
    ));
    let mut group_api_state =
        GroupApiState::new(jwt, pool).with_group_app_registry(group_app_registry);
    if let Some(query_port) = bi_task_event_query_port {
        group_api_state = group_api_state.with_bi_task_event_query_port(query_port);
    }
    let app = build_group_router(group_api_state);
    info!(%bind_addr, "star-api-rest Worktree Group API starting");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
