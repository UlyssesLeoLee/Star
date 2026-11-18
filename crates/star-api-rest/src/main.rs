// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=1b838b950f2cc8f2c92456927b15d8abd680628530c9f5e4e020c7f34f17fa69
MERGE (self:File {path:"crates/star-api-rest/src/main.rs"})
MERGE (main:Symbol {id:"crates/star-api-rest/src/main.rs::main",kind:"function"})
MERGE (store:Type {id:"bi_owner_runtime::PgTaskEventQueryStore",kind:"struct"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQueryPort",kind:"trait"})
MERGE (state:Type {id:"crates/star-api-rest/src/group_api.rs::GroupApiState",kind:"struct"})
MERGE (install:Symbol {id:"crates/star-api-rest/src/group_api.rs::GroupApiState.with_bi_task_event_query_port",kind:"method"})
MERGE (wakeHub:Type {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub",kind:"struct"})
MERGE (wakeStart:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub.start",kind:"method"})
MERGE (wakeInstall:Symbol {id:"crates/star-api-rest/src/group_api.rs::GroupApiState.with_canvas_event_wake_hub",kind:"method"})
MERGE (ingestion:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion",kind:"struct"})
MERGE (configure_ingestion:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.configure",kind:"method"})
MERGE (readiness_router:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.readiness_router",kind:"method"})
MERGE (shutdown_signal:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.shutdown_signal",kind:"method"})
MERGE (shutdown:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.shutdown",kind:"method"})
MERGE (serve_until_shutdown:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::serve_until_shutdown",kind:"function"})
MERGE (self)-[:DEFINES]->(main)
MERGE (main)-[:USES_TYPE]->(store)
MERGE (main)-[:USES_TYPE]->(port)
MERGE (main)-[:CALLS]->(install)
MERGE (main)-[:CALLS]->(wakeStart)
MERGE (main)-[:CALLS]->(wakeInstall)
MERGE (main)-[:CALLS]->(configure_ingestion)
MERGE (main)-[:CALLS]->(readiness_router)
MERGE (main)-[:CALLS]->(shutdown_signal)
MERGE (main)-[:CALLS]->(shutdown)
MERGE (main)-[:CALLS]->(serve_until_shutdown)
MERGE (main)-[:USES_TYPE]->(wakeHub)
MERGE (main)-[:USES_TYPE]->(ingestion)
MERGE (ingestion)-[:HAS_METHOD]->(configure_ingestion)
MERGE (ingestion)-[:HAS_METHOD]->(readiness_router)
MERGE (ingestion)-[:HAS_METHOD]->(shutdown_signal)
MERGE (ingestion)-[:HAS_METHOD]->(shutdown)
MERGE (wakeStart)-[:USES_TYPE]->(:Type {id:"sqlx::postgres::PgListener",kind:"struct"})
MERGE (install)-[:USES_TYPE]->(port)
MERGE (state)-[:HAS_METHOD]->(install)
MERGE (scmState:Type {id:"crates/star-api-rest/src/routes/scm_state.rs::ScmApiState",kind:"struct"})
MERGE (scmStateNew:Symbol {id:"crates/star-api-rest/src/routes/scm_state.rs::ScmApiState.new",kind:"method"})
MERGE (scmService:Type {id:"crates/domain-scm/src/lib.rs::InMemoryScmService",kind:"struct"})
MERGE (newForTest:Symbol {id:"crates/domain-scm/src/lib.rs::InMemoryScmService.new_for_test",kind:"method"})
MERGE (jwtCfg:Symbol {id:"crates/star-api-rest/src/group_api.rs::GroupApiState.jwt_config",kind:"method"})
MERGE (devRepo:Symbol {id:"crates/star-api-rest/src/group_api.rs::GroupApiState.device_binding_repository",kind:"method"})
MERGE (scmRouter:Symbol {id:"crates/star-api-rest/src/lib.rs::build_scm_router",kind:"function"})
MERGE (main)-[:CALLS]->(scmStateNew)
MERGE (main)-[:CALLS]->(newForTest)
MERGE (main)-[:CALLS]->(jwtCfg)
MERGE (main)-[:CALLS]->(devRepo)
MERGE (main)-[:CALLS]->(scmRouter)
MERGE (main)-[:USES_TYPE]->(scmState)
MERGE (main)-[:USES_TYPE]->(scmService)
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
    group_api::{
        build_group_router, CanvasEventWakeHub, GroupApiState, PgGroupAppRegistryProvider,
    },
};
use star_pg_adapter::{connect_pool, PgConfig};
use tracing::info;

mod bi_task_event_ingestion;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind_addr: SocketAddr = std::env::var("STAR_API_REST_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8081".to_string())
        .parse()
        .expect("invalid bind addr");
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    // 2026-11-11 Google 登录: 需要签发 Star JWT ⇒ 优先带私钥的完整 issuer 配置;
    // 无 JWT_PRIVATE_KEY_PEM 时退回 verifier-only (现有部署兼容, Google 登录返回 503)。
    let jwt = Arc::new(match JwtConfig::from_env() {
        Ok(full) => full,
        Err(_) => JwtConfig::verifier_from_env().map_err(|error| {
            anyhow::anyhow!("JWT verifier configuration invalid ({})", error.code())
        })?,
    });
    let google_auth = star_api_rest::auth::google::GoogleAuthState::from_env(jwt.clone());
    let pg_config = PgConfig::from_env()
        .map_err(|error| anyhow::anyhow!("PostgreSQL configuration invalid ({})", error.code()))?;
    let pool = connect_pool(&pg_config)
        .await
        .map_err(|error| anyhow::anyhow!("PostgreSQL connection failed ({})", error.code()))?;
    let canvas_event_wake_hub = CanvasEventWakeHub::start(pool.clone())
        .await
        .map_err(|_| anyhow::anyhow!("Canvas event wake listener setup failed"))?;

    let (bi_task_event_query_port, bi_query_pool): (
        Option<Arc<dyn TaskEventQueryPort>>,
        Option<sqlx::PgPool>,
    ) = match std::env::var("BI_DATABASE_URL") {
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
            (
                Some(Arc::new(PgTaskEventQueryStore::new(bi_pool.clone()))),
                Some(bi_pool),
            )
        }
        Err(std::env::VarError::NotPresent) => (None, None),
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(anyhow::anyhow!("BI_DATABASE_URL must be valid UTF-8"));
        }
    };
    let mut bi_task_event_ingestion =
        bi_task_event_ingestion::BiTaskEventIngestion::configure(pool.clone(), bi_query_pool).await;

    let group_app_registry = Arc::new(PgGroupAppRegistryProvider::new(
        pool.clone(),
        PgGroupAppRegistryProvider::HOST_API_VERSION,
    ));
    let mut group_api_state = GroupApiState::new(jwt, pool)
        .with_group_app_registry(group_app_registry)
        .with_canvas_event_wake_hub(canvas_event_wake_hub);
    if let Some(query_port) = bi_task_event_query_port {
        group_api_state = group_api_state.with_bi_task_event_query_port(query_port);
    }
    // 2026-11-19 批次 B: SCM 生产路由与 group 域同源 (同一 JWT 配置 +
    // 设备绑定仓储 + 单一共享 InMemoryScmService), 先取 accessor 再 move state。
    let scm_state = star_api_rest::routes::scm_state::ScmApiState::new(
        domain_scm::InMemoryScmService::new_for_test(),
        group_api_state.jwt_config(),
        group_api_state.device_binding_repository(),
    );
    // new_for_test() 已返回 Arc<Self>, 直接传入保持单一共享实例。
    let app = build_group_router(group_api_state)
        .merge(bi_task_event_ingestion.readiness_router())
        .merge(star_api_rest::auth::google::router(google_auth))
        // 2026-11-11 用户发令: 网页版关联本地文件夹 (受控只读, 白名单 STAR_FILE_ROOTS)
        .merge(star_api_rest::workspace_files::router(
            star_api_rest::workspace_files::WorkspaceFilesState::from_env(),
        ))
        // F15 G-01 (2026-11-17): 补充 8 模块路由进生产二进制
        // 复用 lib.rs build_supplemental_routers(), 避免两处各挂一份再次漂移
        .merge(star_api_rest::build_supplemental_routers())
        // 2026-11-19 批次 B: SCM 真实化路由 (MR/review/pipeline 共享 service + 真实 JWT)
        .merge(star_api_rest::build_scm_router(scm_state));
    info!(%bind_addr, "star-api-rest Worktree Group API starting");

    let server_result = bi_task_event_ingestion::serve_until_shutdown(
        axum::serve(listener, app),
        bi_task_event_ingestion.shutdown_signal(),
    )
    .await;
    let shutdown_result = bi_task_event_ingestion.shutdown().await;
    server_result?;
    shutdown_result
        .map_err(|reason| anyhow::anyhow!("BI Task-event ingestion shutdown failed ({reason})"))?;
    Ok(())
}
