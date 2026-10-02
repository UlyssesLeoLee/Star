// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=282a001de747031314773f58f0fac0393346111b098bf8a40664e397a5c6315e
MERGE (self:File {path:"crates/star-pg-adapter/src/repository/mod.rs"})
MERGE (schedule_module:File {path:"crates/star-pg-adapter/src/repository/automation_schedule.rs"})
MERGE (terminal:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::OccurrenceTerminalOutcome"})
MERGE (repository:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository"})
MERGE (dispatch:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::ScheduleDispatchResult"})
MERGE (error:Type {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::ScheduleRepositoryError"})
MERGE (self)-[:IMPORTS]->(schedule_module)
MERGE (self)-[:EXPOSES]->(terminal)
MERGE (self)-[:EXPOSES]->(repository)
MERGE (self)-[:EXPOSES]->(dispatch)
MERGE (self)-[:EXPOSES]->(error)
@endcypher
*/
//! Repository 索引 (per WBS §14.10.4 缺口 #4 6 ops Repository + 4 OAuth Repository)
//!
//! v0.67 拆出 ops_metrics_config 到独立文件 (跟其他 5 ops + 4 OAuth Repository 模式一致)
//! 守门 #13 W/T/M 100% 覆盖: T=3 (ops_helm_release_state + ops_cluster_action_log + ops_log_query_log) + W=2 (ops_log_entry TTL 7d + ops_log_analysis TTL 30d) + M=1 (ops_metrics_config SCD2)

pub mod automation_schedule;
pub mod ops_cluster_action_log;
pub mod ops_helm_release_state;
pub mod ops_log_analysis;
pub mod ops_log_entry;
pub mod ops_log_query_log;
pub mod ops_metrics_config;

pub use automation_schedule::{
    OccurrenceTerminalOutcome, PgAutomationScheduleRepository, ScheduleDispatchResult,
    ScheduleRepositoryError,
};
pub use ops_cluster_action_log::{
    OpsClusterActionLog, OpsClusterActionLogRepository, PgOpsClusterActionLogRepository,
};
pub use ops_helm_release_state::{
    OpsHelmReleaseState, OpsHelmReleaseStateRepository, PgOpsHelmReleaseStateRepository,
};
pub use ops_log_analysis::{OpsLogAnalysis, OpsLogAnalysisRepository, PgOpsLogAnalysisRepository};
pub use ops_log_entry::{OpsLogEntry, OpsLogEntryRepository, PgOpsLogEntryRepository};
pub use ops_log_query_log::{OpsLogQueryLog, OpsLogQueryLogRepository, PgOpsLogQueryLogRepository};
pub use ops_metrics_config::{
    OpsMetricsConfig, OpsMetricsConfigRepository, PgOpsMetricsConfigRepository,
};

// ==========================================
// v0.43 OAuth2 server 4 Repository (per 9/9 20:20 JST 用户拍板 both flows)
// 守门 #13 W/T/M 100% 覆盖: M=1 (oauth_clients SCD2) + T=3 (auth_codes WORM + access_tokens + refresh_tokens WORM)
// ==========================================

pub mod oauth_access_tokens;
pub mod oauth_authorization_codes;
pub mod oauth_clients;
pub mod oauth_refresh_tokens;

pub use oauth_access_tokens::{
    OAuthAccessToken, OAuthAccessTokenRepository, PgOAuthAccessTokenRepository,
};
pub use oauth_authorization_codes::{
    OAuthAuthorizationCode, OAuthAuthorizationCodeRepository, PgOAuthAuthorizationCodeRepository,
};
pub use oauth_clients::{OAuthClient, OAuthClientRepository, PgOAuthClientRepository};
pub use oauth_refresh_tokens::{
    OAuthRefreshToken, OAuthRefreshTokenRepository, PgOAuthRefreshTokenRepository,
};

// ==========================================
// v0.82 P0-4 Stage 3.1 = multi-tenant routing DDL 持久化 (per v0.81 已知缺口 (a))
// 守门 #13 c M SCD Type 2 + 守门 #13 d T AuditEvent WORM + 守门 #13 b 物理删除禁止
// ==========================================

pub mod tenant_pool;

pub use tenant_pool::{PgTenantPoolRepository, TenantPool, TenantPoolRepository};
