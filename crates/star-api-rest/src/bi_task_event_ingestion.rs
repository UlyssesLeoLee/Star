// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=26ef5e521632641721019f3882fa2be31c73c416661f80bc2bd35e91baadb342
MERGE (self:File {path:"crates/star-api-rest/src/bi_task_event_ingestion.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::bi_task_event_ingestion",kind:"module"})
MERGE (runtime:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion",kind:"struct"})
MERGE (config:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::RuntimeConfig",kind:"struct"})
MERGE (route_pin:Type {id:"bi_owner_runtime::TaskEventRoutePin",kind:"struct"})
MERGE (receiver:Type {id:"bi_owner_runtime::TaskEventReceiver",kind:"struct"})
MERGE (transport:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcEventReceiveTransport",kind:"struct"})
MERGE (configure:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.configure",kind:"method"})
MERGE (shutdown_signal:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.shutdown_signal",kind:"method"})
MERGE (shutdown:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.shutdown",kind:"method"})
MERGE (shutdown_with_timeout:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::BiTaskEventIngestion.shutdown_with_timeout",kind:"method"})
MERGE (serve_until_shutdown:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::serve_until_shutdown",kind:"function"})
MERGE (wait_shutdown:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::wait_for_shutdown_signal",kind:"function"})
MERGE (build:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::build_receiver",kind:"function"})
MERGE (validate_db:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::validate_write_database",kind:"function"})
MERGE (run_loop:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::run_consumer",kind:"function"})
MERGE (ready:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::readiness_endpoint",kind:"function"})
MERGE (adapter:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::NativeTaskEventTransport",kind:"struct"})
MERGE (authorize:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::NativeTaskEventAuthorization",kind:"struct"})
MERGE (delivery:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::NativeTaskEventDelivery",kind:"struct"})
MERGE (capabilities:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::WriteDatabaseCapabilities",kind:"struct"})
MERGE (role_separation:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::writer_role_is_separate",kind:"function"})
MERGE (profile_monitor:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::ProfilePinMonitor",kind:"struct"})
MERGE (profile_validate:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::ProfilePinMonitor.validate",kind:"method"})
MERGE (profile_match:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::profile_pin_matches",kind:"function"})
MERGE (worker_guard:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::WorkerExitGuard",kind:"struct"})
MERGE (readiness_state:Type {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::ReadinessRouteState",kind:"struct"})
MERGE (effective_ready:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::effective_readiness",kind:"function"})
MERGE (test_module:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests",kind:"module"})
MERGE (test_capabilities:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.restricted_capabilities",kind:"function"})
MERGE (test_restricted_role:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.restricted_write_role_requires_runtime_membership_and_only_bi_functions",kind:"test"})
MERGE (test_elevated_role:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.write_role_rejects_privileges_that_bypass_owner_boundaries",kind:"test"})
MERGE (test_query_contract:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.write_role_query_checks_truncate_and_authenticated_session_privileges",kind:"test"})
MERGE (test_opt_in:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.ingestion_requires_explicit_valid_opt_in",kind:"test"})
MERGE (test_config:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.incomplete_or_empty_runtime_identity_cannot_build_a_consumer_config",kind:"test"})
MERGE (test_role_separation:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.write_role_must_be_distinct_from_application_and_query_roles",kind:"test"})
MERGE (test_readiness:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.readiness_does_not_remain_ready_after_consumer_failure",kind:"test"})
MERGE (test_profile_pin:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.profile_pin_monitor_detects_revision_digest_and_endpoint_changes",kind:"test"})
MERGE (test_capability_query:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.write_role_rejects_other_star_memberships_and_transport_execute",kind:"test"})
MERGE (test_join_failure:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.shutdown_propagates_consumer_join_failure",kind:"test"})
MERGE (test_drain_timeout:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.shutdown_propagates_drain_timeout",kind:"test"})
MERGE (test_serve_drop:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.serve_future_is_dropped_when_shutdown_signal_fires",kind:"test"})
MERGE (test_serve_error:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.serve_io_error_is_propagated",kind:"test"})
MERGE (test_bind_order:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::tests.main_binds_listener_before_starting_service_resources",kind:"test"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(runtime)
MERGE (module)-[:DEFINES]->(config)
MERGE (module)-[:DEFINES]->(adapter)
MERGE (module)-[:DEFINES]->(authorize)
MERGE (module)-[:DEFINES]->(delivery)
MERGE (module)-[:DEFINES]->(configure)
MERGE (module)-[:DEFINES]->(shutdown_signal)
MERGE (module)-[:DEFINES]->(shutdown)
MERGE (module)-[:DEFINES]->(shutdown_with_timeout)
MERGE (module)-[:DEFINES]->(serve_until_shutdown)
MERGE (module)-[:DEFINES]->(wait_shutdown)
MERGE (module)-[:DEFINES]->(build)
MERGE (module)-[:DEFINES]->(validate_db)
MERGE (module)-[:DEFINES]->(run_loop)
MERGE (module)-[:DEFINES]->(ready)
MERGE (module)-[:DEFINES]->(capabilities)
MERGE (module)-[:DEFINES]->(role_separation)
MERGE (module)-[:DEFINES]->(profile_monitor)
MERGE (module)-[:DEFINES]->(profile_validate)
MERGE (module)-[:DEFINES]->(profile_match)
MERGE (module)-[:DEFINES]->(worker_guard)
MERGE (module)-[:DEFINES]->(readiness_state)
MERGE (module)-[:DEFINES]->(effective_ready)
MERGE (module)-[:DEFINES]->(test_module)
MERGE (test_module)-[:DEFINES]->(test_capabilities)
MERGE (test_module)-[:DEFINES]->(test_restricted_role)
MERGE (test_module)-[:DEFINES]->(test_elevated_role)
MERGE (test_module)-[:DEFINES]->(test_query_contract)
MERGE (test_module)-[:DEFINES]->(test_opt_in)
MERGE (test_module)-[:DEFINES]->(test_config)
MERGE (test_module)-[:DEFINES]->(test_role_separation)
MERGE (test_module)-[:DEFINES]->(test_readiness)
MERGE (test_module)-[:DEFINES]->(test_profile_pin)
MERGE (test_module)-[:DEFINES]->(test_capability_query)
MERGE (test_module)-[:DEFINES]->(test_join_failure)
MERGE (test_module)-[:DEFINES]->(test_drain_timeout)
MERGE (test_module)-[:DEFINES]->(test_serve_drop)
MERGE (test_module)-[:DEFINES]->(test_serve_error)
MERGE (test_module)-[:DEFINES]->(test_bind_order)
MERGE (configure)-[:CALLS]->(build)
MERGE (configure)-[:CALLS]->(validate_db)
MERGE (shutdown_signal)-[:CALLS]->(wait_shutdown)
MERGE (shutdown)-[:CALLS]->(shutdown_with_timeout)
MERGE (test_serve_drop)-[:EXERCISES]->(serve_until_shutdown)
MERGE (test_serve_error)-[:EXERCISES]->(serve_until_shutdown)
MERGE (test_join_failure)-[:EXERCISES]->(shutdown)
MERGE (test_drain_timeout)-[:EXERCISES]->(shutdown_with_timeout)
MERGE (test_bind_order)-[:EXERCISES]->(:Symbol {id:"crates/star-api-rest/src/main.rs::main",kind:"function"})
MERGE (build)-[:USES_TYPE]->(route_pin)
MERGE (build)-[:USES_TYPE]->(transport)
MERGE (build)-[:CREATES]->(receiver)
MERGE (build)-[:CREATES]->(profile_monitor)
MERGE (validate_db)-[:CALLS]->(role_separation)
MERGE (profile_validate)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs::TrustedProfileEndpointResolver.resolve",kind:"method"})
MERGE (profile_validate)-[:CALLS]->(profile_match)
MERGE (run_loop)-[:CALLS]->(profile_validate)
MERGE (run_loop)-[:USES_TYPE]->(worker_guard)
MERGE (ready)-[:CALLS]->(effective_ready)
MERGE (ready)-[:USES_TYPE]->(readiness_state)
MERGE (test_readiness)-[:EXERCISES]->(effective_ready)
MERGE (test_profile_pin)-[:EXERCISES]->(profile_match)
MERGE (run_loop)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.receive_one",kind:"method"})
MERGE (run_loop)-[:SUBSCRIBES]->(:Endpoint {id:"platform-messaging-runtime.native-grpc.event-log",kind:"native_event_log"})
MERGE (ready)-[:EXPOSES]->(:Endpoint {id:"/readyz/bi-task-event-ingestion",kind:"http_readiness"})
MERGE (validate_db)-[:READS]->(:DatabaseRole {id:"star_bi_runtime",kind:"database_role"})
MERGE (validate_db)-[:READS]->(:DatabaseRole {id:"session_user",kind:"authenticated_database_principal"})
MERGE (validate_db)-[:READS]->(:DatabaseRole {id:"application_pool.current_user",kind:"application_database_principal"})
MERGE (validate_db)-[:READS]->(:DatabaseRole {id:"application_pool.session_user",kind:"application_authenticated_principal"})
MERGE (validate_db)-[:READS]->(:DatabaseRole {id:"bi_query_pool.current_user",kind:"bi_query_database_principal"})
MERGE (validate_db)-[:READS]->(:DatabaseRole {id:"bi_query_pool.session_user",kind:"bi_query_authenticated_principal"})
MERGE (wait_shutdown)-[:READS]->(:ExternalService {id:"tokio::signal::unix::SignalKind::terminate",kind:"operating_system_signal"})
MERGE (capabilities)-[:READS]->(:DatabaseTable {id:"bi.task_event_inbox",kind:"protected_table"})
MERGE (capabilities)-[:READS]->(:DatabaseTable {id:"bi.task_event_fact",kind:"protected_table"})
MERGE (capabilities)-[:READS]->(:DatabaseTable {id:"bi.task_event_quarantine",kind:"protected_table"})
MERGE (test_restricted_role)-[:EXERCISES]->(capabilities)
MERGE (test_elevated_role)-[:EXERCISES]->(capabilities)
MERGE (test_capability_query)-[:EXERCISES]->(capabilities)
MERGE (test_query_contract)-[:EXERCISES]->(:Symbol {id:"crates/star-api-rest/src/bi_task_event_ingestion.rs::WRITE_DATABASE_CAPABILITY_QUERY",kind:"constant"})
MERGE (test_role_separation)-[:EXERCISES]->(role_separation)
MERGE (delivery)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveDelivery",kind:"trait"})
MERGE (adapter)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveTransport",kind:"trait"})
MERGE (authorize)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveAuthorization",kind:"trait"})
@endcypher
*/
//! Application-owned, opt-in composition for BI Task-event receive.

use std::{
    collections::HashMap,
    env,
    future::{Future, IntoFuture},
    io,
    path::PathBuf,
    str::FromStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use bi_owner_runtime::{
    TaskEventInboxStore, TaskEventQuarantineStore, TaskEventReceiveAuthorization,
    TaskEventReceiveDelivery, TaskEventReceiveError, TaskEventReceiveOrigin, TaskEventReceiveRoute,
    TaskEventReceiveTransport, TaskEventReceiver, TaskEventRoutePin,
};
use platform_messaging_runtime::{
    admission_delivery::AdmissionTransportProfile,
    native_grpc::{
        NativeGrpcEventDelivery, NativeGrpcEventOrigin, NativeGrpcEventReceiveAuthorization,
        NativeGrpcEventReceiveClientConfig, NativeGrpcEventReceiveScope,
        NativeGrpcEventReceiveTransport, native_provider_identity,
    },
    transport_endpoint::{TrustedEndpointConfig, TrustedEndpointResolver},
    transport_profile::{AuthenticatedTransportSession, TransportDirection, TransportMessageKind},
    transport_profile_pg::PgTransportProfileResolver,
    trusted_profile_endpoint::TrustedProfileEndpointResolver,
};
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tokio::{
    sync::{RwLock, watch},
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};
use tracing::warn;
use uuid::Uuid;

pub(crate) const READINESS_PATH: &str = "/readyz/bi-task-event-ingestion";

const ENABLED_ENV: &str = "STAR_BI_TASK_EVENT_INGESTION_ENABLED";
const WRITE_DATABASE_ENV: &str = "BI_TASK_EVENT_INGESTION_DATABASE_URL";
const PROVIDER_ID: &str = "star-rust-log";
const RECEIVE_DEADLINE: Duration = Duration::from_secs(14);
const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(16);
const RETRY_DELAY: Duration = Duration::from_secs(1);
const PROFILE_REVALIDATION_INTERVAL: Duration = Duration::from_secs(30);
const IDLE_DELAY: Duration = Duration::from_millis(250);
const MAX_CERT_FILE_BYTES: u64 = 1024 * 1024;
const WORKER_NOT_STARTED: u8 = 0;
const WORKER_RUNNING: u8 = 1;
const WORKER_STOPPED: u8 = 2;
const WORKER_PANICKED: u8 = 3;
const NATIVE_EVENT_LEASE_SECONDS: u32 = 60;

const WRITE_DATABASE_CAPABILITY_QUERY: &str = r#"
WITH protected_tables(table_name) AS (
    VALUES
        ('bi.task_event_inbox'::text),
        ('bi.task_event_fact'::text),
        ('bi.task_event_quarantine'::text)
), protected_privileges(privilege_name) AS (
    VALUES
        ('SELECT'::text),
        ('INSERT'::text),
        ('UPDATE'::text),
        ('DELETE'::text),
        ('TRUNCATE'::text)
), transport_function_signatures(signature) AS (
    VALUES
        ('platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)'::text),
        ('platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)'::text),
        ('platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)'::text),
        ('platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)'::text),
        ('platform_transport.pull_one(text,uuid,integer)'::text),
        ('platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)'::text),
        ('platform_transport.purge_terminal(text,uuid,integer)'::text)
), resolved_transport_functions AS (
    SELECT signature, to_regprocedure(signature)::oid AS function_oid
      FROM transport_function_signatures
)
SELECT current_user AS role_name,
       session_user AS session_role_name,
       effective_role.rolsuper AS effective_is_superuser,
       effective_role.rolbypassrls AS effective_bypasses_rls,
       effective_role.rolcreaterole AS effective_can_manage_roles,
       effective_role.rolcreatedb AS effective_can_create_databases,
       effective_role.rolreplication AS effective_can_replicate,
       pg_has_role(current_user, 'star_bi_runtime', 'MEMBER') AS effective_runtime_member,
       COALESCE((
           SELECT bool_or(
               pg_has_role(current_user, star_role.rolname, 'MEMBER')
               OR pg_has_role(current_user, star_role.rolname, 'SET')
           )
             FROM pg_roles AS star_role
            WHERE star_role.rolname ~ '^star_'
              AND star_role.rolname <> 'star_bi_runtime'
              AND star_role.rolname <> current_user
       ), false) AS effective_non_bi_star_role_member,
       COALESCE((
           SELECT bool_or(
               has_function_privilege(
                   current_user,
                   transport_function.function_oid,
                   'EXECUTE'
               )
           )
             FROM resolved_transport_functions AS transport_function
            WHERE transport_function.function_oid IS NOT NULL
       ), false) AS effective_transport_execute,
        COALESCE((
            SELECT bool_or(pg_has_role(current_user, owner_role.rolname, 'MEMBER'))
              FROM pg_roles AS owner_role
             WHERE right(owner_role.rolname, 6) = '_owner'
        ), false) AS effective_owner_member,
       has_schema_privilege(current_user, 'bi', 'CREATE') AS effective_schema_create,
       COALESCE(has_function_privilege(
           current_user,
           to_regprocedure('bi.commit_task_event_v2(uuid,uuid,uuid,uuid,uuid,varchar,integer,varchar,integer,uuid,uuid,timestamptz,jsonb,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,bytea,bytea)'),
           'EXECUTE'
       ), false) AS effective_can_commit,
       COALESCE(has_function_privilege(
           current_user,
           to_regprocedure('bi.record_task_event_quarantine_v1(uuid,uuid,uuid,varchar,varchar,uuid,bigint,bytea,varchar,varchar,uuid,varchar,varchar,uuid,bigint,uuid,bytea,integer,varchar)'),
           'EXECUTE'
       ), false) AS effective_can_quarantine,
       EXISTS (
           SELECT 1
             FROM protected_tables
             CROSS JOIN protected_privileges
            WHERE has_table_privilege(current_user, protected_tables.table_name, protected_privileges.privilege_name)
       ) AS effective_direct_table_access,
       authenticated_role.rolsuper AS authenticated_is_superuser,
       authenticated_role.rolbypassrls AS authenticated_bypasses_rls,
       authenticated_role.rolcreaterole AS authenticated_can_manage_roles,
       authenticated_role.rolcreatedb AS authenticated_can_create_databases,
       authenticated_role.rolreplication AS authenticated_can_replicate,
       pg_has_role(session_user, 'star_bi_runtime', 'MEMBER') AS authenticated_runtime_member,
       COALESCE((
           SELECT bool_or(
               pg_has_role(session_user, star_role.rolname, 'MEMBER')
               OR pg_has_role(session_user, star_role.rolname, 'SET')
           )
             FROM pg_roles AS star_role
            WHERE star_role.rolname ~ '^star_'
              AND star_role.rolname <> 'star_bi_runtime'
              AND star_role.rolname <> session_user
       ), false) AS authenticated_non_bi_star_role_member,
       COALESCE((
           SELECT bool_or(
               has_function_privilege(
                   session_user,
                   transport_function.function_oid,
                   'EXECUTE'
               )
           )
             FROM resolved_transport_functions AS transport_function
            WHERE transport_function.function_oid IS NOT NULL
       ), false) AS authenticated_transport_execute,
        COALESCE((
            SELECT bool_or(pg_has_role(session_user, owner_role.rolname, 'SET'))
              FROM pg_roles AS owner_role
             WHERE right(owner_role.rolname, 6) = '_owner'
        ), false) AS authenticated_owner_role_settable,
       has_schema_privilege(session_user, 'bi', 'CREATE') AS authenticated_schema_create,
       EXISTS (
           SELECT 1
             FROM protected_tables
             CROSS JOIN protected_privileges
            WHERE has_table_privilege(session_user, protected_tables.table_name, protected_privileges.privilege_name)
       ) AS authenticated_direct_table_access
  FROM pg_roles AS effective_role
 CROSS JOIN pg_roles AS authenticated_role
 WHERE effective_role.rolname = current_user
   AND authenticated_role.rolname = session_user
"#;

#[derive(Clone, Copy, Debug, Default)]
struct WriteDatabaseCapabilities {
    effective_is_superuser: bool,
    effective_bypasses_rls: bool,
    effective_can_manage_roles: bool,
    effective_can_create_databases: bool,
    effective_can_replicate: bool,
    effective_runtime_member: bool,
    effective_non_bi_star_role_member: bool,
    effective_transport_execute: bool,
    effective_owner_member: bool,
    effective_schema_create: bool,
    effective_can_commit: bool,
    effective_can_quarantine: bool,
    effective_direct_table_access: bool,
    authenticated_is_superuser: bool,
    authenticated_bypasses_rls: bool,
    authenticated_can_manage_roles: bool,
    authenticated_can_create_databases: bool,
    authenticated_can_replicate: bool,
    authenticated_runtime_member: bool,
    authenticated_non_bi_star_role_member: bool,
    authenticated_transport_execute: bool,
    authenticated_owner_role_settable: bool,
    authenticated_schema_create: bool,
    authenticated_direct_table_access: bool,
}

impl WriteDatabaseCapabilities {
    fn is_restricted(self) -> bool {
        !self.effective_is_superuser
            && !self.effective_bypasses_rls
            && !self.effective_can_manage_roles
            && !self.effective_can_create_databases
            && !self.effective_can_replicate
            && self.effective_runtime_member
            && !self.effective_non_bi_star_role_member
            && !self.effective_transport_execute
            && !self.effective_owner_member
            && !self.effective_schema_create
            && self.effective_can_commit
            && self.effective_can_quarantine
            && !self.effective_direct_table_access
            && !self.authenticated_is_superuser
            && !self.authenticated_bypasses_rls
            && !self.authenticated_can_manage_roles
            && !self.authenticated_can_create_databases
            && !self.authenticated_can_replicate
            && self.authenticated_runtime_member
            && !self.authenticated_non_bi_star_role_member
            && !self.authenticated_transport_execute
            && !self.authenticated_owner_role_settable
            && !self.authenticated_schema_create
            && !self.authenticated_direct_table_access
    }
}

type SharedReadiness = Arc<RwLock<ReadinessSnapshot>>;
type SharedWorkerState = Arc<AtomicU8>;

#[derive(Clone)]
struct ReadinessRouteState {
    readiness: SharedReadiness,
    worker_state: SharedWorkerState,
}

type RuntimeReceiver = TaskEventReceiver<
    NativeTaskEventTransport,
    NativeTaskEventAuthorization,
    TaskEventInboxStore,
    TaskEventQuarantineStore,
>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadinessPhase {
    Disabled,
    Starting,
    Ready,
    Blocked,
    Draining,
    Stopped,
}

impl ReadinessPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug)]
struct ReadinessSnapshot {
    phase: ReadinessPhase,
    reason: &'static str,
}

impl ReadinessSnapshot {
    fn new(phase: ReadinessPhase, reason: &'static str) -> Self {
        Self { phase, reason }
    }
}

/// Opt-in ingestion worker and its dedicated readiness surface.
pub(crate) struct BiTaskEventIngestion {
    readiness: SharedReadiness,
    worker_state: SharedWorkerState,
    shutdown: watch::Sender<bool>,
    worker: Option<JoinHandle<()>>,
}

impl BiTaskEventIngestion {
    /// Resolve all trusted runtime configuration before starting the bounded consumer task.
    pub(crate) async fn configure(app_pool: PgPool, query_pool: Option<PgPool>) -> Self {
        let enabled = match parse_opt_in(env::var(ENABLED_ENV).ok().as_deref()) {
            Ok(enabled) => enabled,
            Err(()) => return Self::disabled("invalid_opt_in"),
        };
        if !enabled {
            return Self::disabled("not_opted_in");
        }

        let config = match RuntimeConfig::from_environment() {
            Ok(config) => config,
            Err(ConfigError::Missing | ConfigError::Invalid) => {
                return Self::disabled("incomplete_configuration");
            }
        };
        let Some(query_pool) = query_pool else {
            return Self::disabled("missing_bi_query_pool");
        };

        let (receiver, profile_pin_monitor) =
            match build_receiver(app_pool, query_pool, config).await {
                Ok(runtime) => runtime,
                Err(reason) => {
                    warn!(reason, "BI Task-event ingestion remains disabled");
                    return Self::blocked(reason);
                }
            };

        let readiness = Arc::new(RwLock::new(ReadinessSnapshot::new(
            ReadinessPhase::Starting,
            "awaiting_native_receive_check",
        )));
        let worker_state = Arc::new(AtomicU8::new(WORKER_NOT_STARTED));
        let (shutdown, shutdown_rx) = watch::channel(false);
        let worker = tokio::spawn(run_consumer(
            receiver,
            profile_pin_monitor,
            readiness.clone(),
            shutdown_rx,
            worker_state.clone(),
        ));
        Self {
            readiness,
            worker_state,
            shutdown,
            worker: Some(worker),
        }
    }

    /// Construct a disabled state for deployments without explicit ingestion opt-in.
    pub(crate) fn disabled(reason: &'static str) -> Self {
        Self::without_worker(ReadinessPhase::Disabled, reason)
    }

    fn blocked(reason: &'static str) -> Self {
        Self::without_worker(ReadinessPhase::Blocked, reason)
    }

    fn without_worker(phase: ReadinessPhase, reason: &'static str) -> Self {
        let (shutdown, _) = watch::channel(false);
        Self {
            readiness: Arc::new(RwLock::new(ReadinessSnapshot::new(phase, reason))),
            worker_state: Arc::new(AtomicU8::new(WORKER_NOT_STARTED)),
            shutdown,
            worker: None,
        }
    }

    /// Add an explicit readiness route; disabled and blocked states return HTTP 503.
    pub(crate) fn readiness_router(&self) -> Router {
        Router::new()
            .route(READINESS_PATH, get(readiness_endpoint))
            .with_state(ReadinessRouteState {
                readiness: self.readiness.clone(),
                worker_state: self.worker_state.clone(),
            })
    }

    /// Build the signal future without borrowing the runtime across Axum's server future.
    pub(crate) fn shutdown_signal(&self) -> impl Future<Output = ()> + Send + 'static {
        let shutdown = self.shutdown.clone();
        let readiness = self.readiness.clone();
        let has_worker = self.worker.is_some();
        async move {
            wait_for_shutdown_signal().await;
            let _ = shutdown.send(true);
            if has_worker {
                set_readiness(&readiness, ReadinessPhase::Draining, "shutdown_requested").await;
            }
        }
    }

    /// Stop new pulls, allow one bounded receive to settle, then abort only after the drain limit.
    pub(crate) async fn shutdown(&mut self) -> Result<(), &'static str> {
        self.shutdown_with_timeout(SHUTDOWN_DRAIN_TIMEOUT).await
    }

    async fn shutdown_with_timeout(&mut self, drain_timeout: Duration) -> Result<(), &'static str> {
        let _ = self.shutdown.send(true);
        if self.worker.is_none() {
            return Ok(());
        }
        set_readiness(
            &self.readiness,
            ReadinessPhase::Draining,
            "shutdown_requested",
        )
        .await;
        let shutdown_result = if let Some(mut worker) = self.worker.take() {
            match timeout(drain_timeout, &mut worker).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(_)) => {
                    set_readiness(
                        &self.readiness,
                        ReadinessPhase::Blocked,
                        "consumer_task_failed",
                    )
                    .await;
                    Err("consumer_task_failed")
                }
                Err(_) => {
                    warn!(
                        "BI Task-event consumer exceeded its bounded shutdown drain; aborting it"
                    );
                    worker.abort();
                    let _ = worker.await;
                    set_readiness(
                        &self.readiness,
                        ReadinessPhase::Blocked,
                        "shutdown_drain_timeout",
                    )
                    .await;
                    Err("shutdown_drain_timeout")
                }
            }
        } else {
            Ok(())
        };
        if shutdown_result.is_ok() && self.readiness.read().await.phase != ReadinessPhase::Blocked {
            set_readiness(
                &self.readiness,
                ReadinessPhase::Stopped,
                "shutdown_complete",
            )
            .await;
        }
        shutdown_result
    }
}

/// Stop waiting for Axum's accept loop as soon as shutdown starts.
///
/// Axum runs connections in spawned Tokio tasks; dropping this accept future does not
/// drain those tasks. `main` performs the bounded BI drain and then returns, allowing
/// Tokio runtime teardown to cancel any remaining HTTP/SSE request tasks.
pub(crate) async fn serve_until_shutdown<S, Shutdown>(
    serve: S,
    shutdown_signal: Shutdown,
) -> io::Result<()>
where
    S: IntoFuture<Output = io::Result<()>>,
    Shutdown: Future<Output = ()>,
{
    let serve = serve.into_future();
    tokio::pin!(serve);
    tokio::select! {
        biased;
        result = &mut serve => result,
        _ = shutdown_signal => Ok(()),
    }
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate = match tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        ) {
            Ok(terminate) => terminate,
            Err(error) => {
                warn!(%error, "cannot install SIGTERM handler; stopping BI Task-event ingestion");
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        warn!(%error, "cannot install console shutdown handler; stopping BI Task-event ingestion");
    }
}

/// Parse explicit opt-in. Unknown values fail closed instead of silently enabling ingestion.
fn parse_opt_in(value: Option<&str>) -> Result<bool, ()> {
    match value.map(str::trim) {
        None | Some("false") | Some("0") => Ok(false),
        Some("true") | Some("1") => Ok(true),
        Some(_) => Err(()),
    }
}

#[derive(Clone)]
struct RuntimeConfig {
    database_url: String,
    deployment_id: Uuid,
    tenant_id: Uuid,
    sender_service_id: Uuid,
    receiver_service_id: Uuid,
    profile_id: Uuid,
    provider_version: String,
    endpoint_identity: String,
    endpoint_url: String,
    tls_server_name: String,
    destination_id: String,
    destination_generation: Uuid,
    log_id: String,
    server_ca_path: PathBuf,
    client_cert_path: PathBuf,
    client_key_path: PathBuf,
}

#[derive(Clone, Copy)]
enum ConfigError {
    Missing,
    Invalid,
}

impl RuntimeConfig {
    fn from_environment() -> Result<Self, ConfigError> {
        let mut values = HashMap::new();
        for key in [
            WRITE_DATABASE_ENV,
            "STAR_BI_TASK_EVENT_DEPLOYMENT_ID",
            "STAR_BI_TASK_EVENT_TENANT_ID",
            "STAR_BI_TASK_EVENT_SENDER_SERVICE_ID",
            "STAR_BI_TASK_EVENT_RECEIVER_SERVICE_ID",
            "STAR_BI_TASK_EVENT_PROFILE_ID",
            "STAR_BI_TASK_EVENT_PROVIDER_VERSION",
            "STAR_BI_TASK_EVENT_ENDPOINT_IDENTITY",
            "STAR_BI_TASK_EVENT_ENDPOINT_URL",
            "STAR_BI_TASK_EVENT_TLS_SERVER_NAME",
            "STAR_BI_TASK_EVENT_DESTINATION_ID",
            "STAR_BI_TASK_EVENT_DESTINATION_GENERATION",
            "STAR_BI_TASK_EVENT_LOG_ID",
            "STAR_BI_TASK_EVENT_SERVER_CA_PATH",
            "STAR_BI_TASK_EVENT_CLIENT_CERT_PATH",
            "STAR_BI_TASK_EVENT_CLIENT_KEY_PATH",
        ] {
            if let Ok(value) = env::var(key) {
                values.insert(key.to_owned(), value);
            }
        }
        Self::from_values(&values)
    }

    fn from_values(values: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let required = |key: &str| -> Result<String, ConfigError> {
            let value = values.get(key).ok_or(ConfigError::Missing)?.trim();
            if value.is_empty() {
                return Err(ConfigError::Missing);
            }
            Ok(value.to_owned())
        };
        let parse_id = |key: &str| -> Result<Uuid, ConfigError> {
            Uuid::parse_str(&required(key)?).map_err(|_| ConfigError::Invalid)
        };
        Ok(Self {
            database_url: required(WRITE_DATABASE_ENV)?,
            deployment_id: parse_id("STAR_BI_TASK_EVENT_DEPLOYMENT_ID")?,
            tenant_id: parse_id("STAR_BI_TASK_EVENT_TENANT_ID")?,
            sender_service_id: parse_id("STAR_BI_TASK_EVENT_SENDER_SERVICE_ID")?,
            receiver_service_id: parse_id("STAR_BI_TASK_EVENT_RECEIVER_SERVICE_ID")?,
            profile_id: parse_id("STAR_BI_TASK_EVENT_PROFILE_ID")?,
            provider_version: required("STAR_BI_TASK_EVENT_PROVIDER_VERSION")?,
            endpoint_identity: required("STAR_BI_TASK_EVENT_ENDPOINT_IDENTITY")?,
            endpoint_url: required("STAR_BI_TASK_EVENT_ENDPOINT_URL")?,
            tls_server_name: required("STAR_BI_TASK_EVENT_TLS_SERVER_NAME")?,
            destination_id: required("STAR_BI_TASK_EVENT_DESTINATION_ID")?,
            destination_generation: parse_id("STAR_BI_TASK_EVENT_DESTINATION_GENERATION")?,
            log_id: required("STAR_BI_TASK_EVENT_LOG_ID")?,
            server_ca_path: PathBuf::from(required("STAR_BI_TASK_EVENT_SERVER_CA_PATH")?),
            client_cert_path: PathBuf::from(required("STAR_BI_TASK_EVENT_CLIENT_CERT_PATH")?),
            client_key_path: PathBuf::from(required("STAR_BI_TASK_EVENT_CLIENT_KEY_PATH")?),
        })
    }
}

#[derive(Clone)]
struct TrustedBiTaskEventSession {
    deployment_id: Uuid,
    tenant_id: Uuid,
    provider_version: String,
    endpoint_identity: String,
    sender_service_id: Uuid,
    receiver_service_id: Uuid,
}

struct ProfilePinMonitor {
    resolver: TrustedProfileEndpointResolver,
    session: TrustedBiTaskEventSession,
    profile_id: Uuid,
    expected_deployment_id: Uuid,
    expected_tenant_id: Uuid,
    expected_route_pin: TaskEventRoutePin,
}

enum ProfilePinCheckError {
    Unavailable,
    Changed,
}

impl ProfilePinMonitor {
    async fn validate(&self) -> Result<(), ProfilePinCheckError> {
        let endpoint = self
            .resolver
            .resolve(&self.session, self.profile_id, TransportMessageKind::Event)
            .await
            .map_err(|_| ProfilePinCheckError::Unavailable)?;
        if !profile_pin_matches(
            &self.expected_route_pin,
            self.expected_deployment_id,
            self.expected_tenant_id,
            endpoint.deployment_id(),
            endpoint.tenant_id(),
            endpoint.profile_id(),
            endpoint.profile_revision(),
            endpoint.profile_digest(),
            endpoint.endpoint_identity(),
        ) {
            return Err(ProfilePinCheckError::Changed);
        }
        Ok(())
    }
}

fn profile_pin_matches(
    expected: &TaskEventRoutePin,
    expected_deployment_id: Uuid,
    expected_tenant_id: Uuid,
    deployment_id: Uuid,
    tenant_id: Uuid,
    profile_id: Uuid,
    profile_revision: i64,
    profile_digest: [u8; 32],
    endpoint_identity: &str,
) -> bool {
    deployment_id == expected_deployment_id
        && tenant_id == expected_tenant_id
        && profile_id == expected.transport_profile_id
        && profile_revision == expected.transport_profile_revision
        && profile_digest == expected.transport_profile_digest
        && endpoint_identity == expected.endpoint_identity
}

impl AuthenticatedTransportSession for TrustedBiTaskEventSession {
    fn deployment_id(&self) -> Uuid {
        self.deployment_id
    }
    fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn provider_version(&self) -> &str {
        &self.provider_version
    }
    fn endpoint_identity(&self) -> &str {
        &self.endpoint_identity
    }
    fn local_service_id(&self) -> Uuid {
        self.receiver_service_id
    }
    fn peer_service_id(&self) -> Uuid {
        self.sender_service_id
    }
    fn direction(&self) -> TransportDirection {
        TransportDirection::Receive
    }
}

#[derive(Clone)]
struct NativeOriginCache(Arc<Mutex<Option<NativeGrpcEventOrigin>>>);

impl NativeOriginCache {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    fn put(&self, origin: NativeGrpcEventOrigin) -> Result<(), TaskEventReceiveError> {
        let mut current = self
            .0
            .lock()
            .map_err(|_| TaskEventReceiveError::Unauthorized)?;
        *current = Some(origin);
        Ok(())
    }

    fn matching(
        &self,
        route: &TaskEventReceiveRoute,
        expected_pin: &TaskEventRoutePin,
        expected_scope: NativeGrpcEventReceiveScope,
        origin: &TaskEventReceiveOrigin,
        wire: &[u8],
    ) -> Result<NativeGrpcEventOrigin, TaskEventReceiveError> {
        if !same_route_pin(route.pin(), expected_pin) || !route_matches_scope(route, expected_scope)
        {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        let current = self
            .0
            .lock()
            .map_err(|_| TaskEventReceiveError::Unauthorized)?;
        let native = current
            .as_ref()
            .filter(|native| {
                native.scope() == expected_scope
                    && native.provider_identity() == origin.provider_identity()
                    && native.log_id() == origin.log_id()
                    && native.log_generation() == origin.log_generation()
                    && native.sequence() == origin.sequence()
                    && native.message_id() == origin.message_id()
                    && native.wire_sha256() == <[u8; 32]>::from(Sha256::digest(wire))
            })
            .cloned()
            .ok_or(TaskEventReceiveError::Unauthorized)?;
        Ok(native)
    }
}

struct NativeTaskEventTransport {
    inner: NativeGrpcEventReceiveTransport,
    scope: NativeGrpcEventReceiveScope,
    route_pin: TaskEventRoutePin,
    origins: NativeOriginCache,
}

struct NativeTaskEventDelivery {
    inner: NativeGrpcEventDelivery,
    origin: TaskEventReceiveOrigin,
}

impl TaskEventReceiveDelivery for NativeTaskEventDelivery {
    fn exact_wire(&self) -> &[u8] {
        self.inner.exact_wire()
    }

    fn origin(&self) -> &TaskEventReceiveOrigin {
        &self.origin
    }

    async fn acknowledge(&self) -> Result<(), TaskEventReceiveError> {
        self.inner
            .acknowledge()
            .await
            .map_err(|_| TaskEventReceiveError::Transport)
    }

    async fn terminate_permanently(&self) -> Result<(), TaskEventReceiveError> {
        // The BI receiver calls this only after the owner quarantine receipt commits and the
        // native origin is re-verified. Native event delivery has one terminal disposition: ACK.
        self.acknowledge().await
    }
}

impl TaskEventReceiveTransport for NativeTaskEventTransport {
    type Delivery = NativeTaskEventDelivery;

    async fn pull_one(
        &self,
        route: &TaskEventReceiveRoute,
    ) -> Result<Option<Self::Delivery>, TaskEventReceiveError> {
        if !same_route_pin(route.pin(), &self.route_pin) || !route_matches_scope(route, self.scope)
        {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        let Some(inner) = self
            .inner
            .pull_one(self.scope)
            .await
            .map_err(|_| TaskEventReceiveError::Transport)?
        else {
            return Ok(None);
        };
        let native_origin = inner.origin().clone();
        let origin = TaskEventReceiveOrigin::new(
            native_origin.provider_identity().to_owned(),
            native_origin.log_id().to_owned(),
            native_origin.log_generation(),
            native_origin.sequence(),
            native_origin.message_id(),
        )?;
        self.origins.put(native_origin)?;
        Ok(Some(NativeTaskEventDelivery { inner, origin }))
    }
}

struct NativeTaskEventAuthorization {
    inner: NativeGrpcEventReceiveAuthorization,
    scope: NativeGrpcEventReceiveScope,
    route_pin: TaskEventRoutePin,
    origins: NativeOriginCache,
}

impl TaskEventReceiveAuthorization for NativeTaskEventAuthorization {
    async fn verify_origin(
        &self,
        route: &TaskEventReceiveRoute,
        origin: &TaskEventReceiveOrigin,
        exact_wire: &[u8],
    ) -> Result<(), TaskEventReceiveError> {
        let native_origin =
            self.origins
                .matching(route, &self.route_pin, self.scope, origin, exact_wire)?;
        self.inner
            .verify_origin(self.scope, &native_origin, exact_wire)
            .await
            .map_err(|_| TaskEventReceiveError::Unauthorized)
    }
}

async fn build_receiver(
    app_pool: PgPool,
    query_pool: PgPool,
    config: RuntimeConfig,
) -> Result<(Arc<RuntimeReceiver>, ProfilePinMonitor), &'static str> {
    validate_tls_files(&config)?;
    let write_options = PgConnectOptions::from_str(&config.database_url)
        .map_err(|_| "bi_write_database_configuration")?;
    let write_pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(2))
        .connect_with(write_options)
        .await
        .map_err(|_| "bi_write_database_unavailable")?;
    validate_write_database(&write_pool, &app_pool, &query_pool).await?;

    let endpoint_config = TrustedEndpointConfig::new(
        config.endpoint_identity.clone(),
        config.endpoint_url.clone(),
        config.tls_server_name.clone(),
    )
    .map_err(|_| "trusted_endpoint_invalid")?;
    let endpoints = TrustedEndpointResolver::new(config.deployment_id, vec![endpoint_config])
        .map_err(|_| "trusted_endpoint_invalid")?;
    let profiles = PgTransportProfileResolver::new(app_pool);
    let resolver = TrustedProfileEndpointResolver::new(profiles, endpoints);
    let session = TrustedBiTaskEventSession {
        deployment_id: config.deployment_id,
        tenant_id: config.tenant_id,
        provider_version: config.provider_version.clone(),
        endpoint_identity: config.endpoint_identity.clone(),
        sender_service_id: config.sender_service_id,
        receiver_service_id: config.receiver_service_id,
    };
    let endpoint = resolver
        .resolve(&session, config.profile_id, TransportMessageKind::Event)
        .await
        .map_err(|_| "current_profile_or_endpoint_unavailable")?;
    if endpoint.deployment_id() != config.deployment_id
        || endpoint.tenant_id() != config.tenant_id
        || endpoint.profile_id() != config.profile_id
        || endpoint.endpoint_identity() != config.endpoint_identity
    {
        return Err("current_profile_or_endpoint_mismatch");
    }
    let profile = AdmissionTransportProfile::new(
        PROVIDER_ID.to_owned(),
        config.provider_version.clone(),
        endpoint.profile_digest(),
        endpoint.endpoint_identity().to_owned(),
    )
    .map_err(|_| "current_profile_invalid")?;
    let scope = NativeGrpcEventReceiveScope::new(
        config.tenant_id,
        config.sender_service_id,
        config.receiver_service_id,
    )
    .map_err(|_| "receive_identity_invalid")?;
    let native_transport =
        NativeGrpcEventReceiveTransport::connect(NativeGrpcEventReceiveClientConfig {
            endpoint: &endpoint,
            server_ca_path: &config.server_ca_path,
            client_cert_path: &config.client_cert_path,
            client_key_path: &config.client_key_path,
            connect_timeout: Duration::from_secs(3),
            request_timeout: Duration::from_secs(8),
            scope,
            expected_log_id: &config.log_id,
            expected_log_generation: config.destination_generation,
            lease_seconds: NATIVE_EVENT_LEASE_SECONDS,
            profile: profile.clone(),
        })
        .await
        .map_err(|_| "native_transport_setup_failed")?;

    let route_pin = TaskEventRoutePin {
        sender_service_id: config.sender_service_id,
        receiver_service_id: config.receiver_service_id,
        transport_provider_id: native_provider_identity(&profile),
        transport_provider_version: config.provider_version,
        transport_profile_id: endpoint.profile_id(),
        transport_profile_revision: endpoint.profile_revision(),
        transport_profile_digest: endpoint.profile_digest(),
        endpoint_identity: endpoint.endpoint_identity().to_owned(),
        destination_id: config.destination_id,
        destination_generation: config.destination_generation,
    };
    let route = TaskEventReceiveRoute::new(config.tenant_id, route_pin.clone())
        .map_err(|_| "receive_route_invalid")?;
    let origins = NativeOriginCache::new();
    let authorization = native_transport.authorization();
    let receiver = TaskEventReceiver::new(
        route,
        NativeTaskEventTransport {
            inner: native_transport,
            scope,
            route_pin: route_pin.clone(),
            origins: origins.clone(),
        },
        NativeTaskEventAuthorization {
            inner: authorization,
            scope,
            route_pin: route_pin.clone(),
            origins,
        },
        TaskEventInboxStore::new(write_pool.clone(), config.receiver_service_id),
        TaskEventQuarantineStore::new(write_pool),
    );
    let profile_pin_monitor = ProfilePinMonitor {
        resolver,
        session,
        profile_id: config.profile_id,
        expected_deployment_id: config.deployment_id,
        expected_tenant_id: config.tenant_id,
        expected_route_pin: route_pin,
    };
    Ok((Arc::new(receiver), profile_pin_monitor))
}

fn validate_tls_files(config: &RuntimeConfig) -> Result<(), &'static str> {
    for path in [
        &config.server_ca_path,
        &config.client_cert_path,
        &config.client_key_path,
    ] {
        let metadata =
            std::fs::metadata(path).map_err(|_| "client_identity_material_unavailable")?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_CERT_FILE_BYTES {
            return Err("client_identity_material_invalid");
        }
    }
    Ok(())
}

async fn validate_write_database(
    write_pool: &PgPool,
    app_pool: &PgPool,
    query_pool: &PgPool,
) -> Result<(), &'static str> {
    let app_identity =
        sqlx::query("SELECT current_user AS role_name, session_user AS session_role_name")
            .fetch_one(app_pool)
            .await
            .map_err(|_| "application_database_unavailable")?;
    let app_role: String = app_identity
        .try_get("role_name")
        .map_err(|_| "application_database_unavailable")?;
    let app_session_role: String = app_identity
        .try_get("session_role_name")
        .map_err(|_| "application_database_unavailable")?;
    let query_identity =
        sqlx::query("SELECT current_user AS role_name, session_user AS session_role_name")
            .fetch_one(query_pool)
            .await
            .map_err(|_| "bi_query_database_unavailable")?;
    let query_role: String = query_identity
        .try_get("role_name")
        .map_err(|_| "bi_query_database_unavailable")?;
    let query_session_role: String = query_identity
        .try_get("session_role_name")
        .map_err(|_| "bi_query_database_unavailable")?;
    let row = sqlx::query(WRITE_DATABASE_CAPABILITY_QUERY)
        .fetch_one(write_pool)
        .await
        .map_err(|_| "bi_write_capability_check_failed")?;
    let write_role: String = row
        .try_get("role_name")
        .map_err(|_| "bi_write_capability_check_failed")?;
    let write_session_role: String = row
        .try_get("session_role_name")
        .map_err(|_| "bi_write_capability_check_failed")?;
    let capabilities = WriteDatabaseCapabilities {
        effective_is_superuser: row
            .try_get("effective_is_superuser")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_bypasses_rls: row
            .try_get("effective_bypasses_rls")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_can_manage_roles: row
            .try_get("effective_can_manage_roles")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_can_create_databases: row
            .try_get("effective_can_create_databases")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_can_replicate: row
            .try_get("effective_can_replicate")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_runtime_member: row
            .try_get("effective_runtime_member")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_non_bi_star_role_member: row
            .try_get("effective_non_bi_star_role_member")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_transport_execute: row
            .try_get("effective_transport_execute")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_owner_member: row
            .try_get("effective_owner_member")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_schema_create: row
            .try_get("effective_schema_create")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_can_commit: row
            .try_get("effective_can_commit")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_can_quarantine: row
            .try_get("effective_can_quarantine")
            .map_err(|_| "bi_write_capability_check_failed")?,
        effective_direct_table_access: row
            .try_get("effective_direct_table_access")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_is_superuser: row
            .try_get("authenticated_is_superuser")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_bypasses_rls: row
            .try_get("authenticated_bypasses_rls")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_can_manage_roles: row
            .try_get("authenticated_can_manage_roles")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_can_create_databases: row
            .try_get("authenticated_can_create_databases")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_can_replicate: row
            .try_get("authenticated_can_replicate")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_runtime_member: row
            .try_get("authenticated_runtime_member")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_non_bi_star_role_member: row
            .try_get("authenticated_non_bi_star_role_member")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_transport_execute: row
            .try_get("authenticated_transport_execute")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_owner_role_settable: row
            .try_get("authenticated_owner_role_settable")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_schema_create: row
            .try_get("authenticated_schema_create")
            .map_err(|_| "bi_write_capability_check_failed")?,
        authenticated_direct_table_access: row
            .try_get("authenticated_direct_table_access")
            .map_err(|_| "bi_write_capability_check_failed")?,
    };
    if !writer_role_is_separate(
        &write_role,
        &write_session_role,
        &app_role,
        &app_session_role,
        &query_role,
        &query_session_role,
    ) || !capabilities.is_restricted()
    {
        return Err("bi_write_role_not_restricted");
    }
    Ok(())
}

fn writer_role_is_separate(
    write_role: &str,
    write_session_role: &str,
    app_role: &str,
    app_session_role: &str,
    query_role: &str,
    query_session_role: &str,
) -> bool {
    let writer_roles = [write_role, write_session_role];
    let application_and_query_roles = [app_role, app_session_role, query_role, query_session_role];
    writer_roles.iter().all(|role| !role.trim().is_empty())
        && application_and_query_roles
            .iter()
            .all(|role| !role.trim().is_empty())
        && writer_roles
            .iter()
            .all(|role| !application_and_query_roles.contains(role))
}

struct WorkerExitGuard(SharedWorkerState);

impl Drop for WorkerExitGuard {
    fn drop(&mut self) {
        let state = if std::thread::panicking() {
            WORKER_PANICKED
        } else {
            WORKER_STOPPED
        };
        self.0.store(state, Ordering::Release);
    }
}

async fn run_consumer(
    receiver: Arc<RuntimeReceiver>,
    profile_pin_monitor: ProfilePinMonitor,
    readiness: SharedReadiness,
    mut shutdown: watch::Receiver<bool>,
    worker_state: SharedWorkerState,
) {
    let _exit_guard = WorkerExitGuard(worker_state.clone());
    worker_state.store(WORKER_RUNNING, Ordering::Release);
    let mut next_profile_validation = Instant::now();
    loop {
        if *shutdown.borrow() {
            break;
        }
        if Instant::now() >= next_profile_validation {
            match profile_pin_monitor.validate().await {
                Ok(()) => {
                    next_profile_validation = Instant::now() + PROFILE_REVALIDATION_INTERVAL;
                }
                Err(ProfilePinCheckError::Changed) => {
                    set_readiness(
                        &readiness,
                        ReadinessPhase::Blocked,
                        "profile_pin_changed_drain_and_restart_required",
                    )
                    .await;
                    return;
                }
                Err(ProfilePinCheckError::Unavailable) => {
                    set_readiness(
                        &readiness,
                        ReadinessPhase::Blocked,
                        "profile_pin_check_unavailable",
                    )
                    .await;
                    next_profile_validation = Instant::now() + RETRY_DELAY;
                    if !wait_or_shutdown(&mut shutdown, RETRY_DELAY).await {
                        break;
                    }
                    continue;
                }
            }
        }
        let cancellation = Default::default();
        let result = receiver
            .receive_one(&cancellation, Instant::now() + RECEIVE_DEADLINE)
            .await;
        if *shutdown.borrow() {
            break;
        }
        let delay = match result {
            Ok(bi_owner_runtime::TaskEventReceiveOutcome::Idle) => {
                set_readiness(&readiness, ReadinessPhase::Ready, "native_receive_verified").await;
                IDLE_DELAY
            }
            Ok(_) => {
                set_readiness(&readiness, ReadinessPhase::Ready, "native_receive_verified").await;
                Duration::ZERO
            }
            Err(TaskEventReceiveError::Unauthorized) => {
                set_readiness(
                    &readiness,
                    ReadinessPhase::Blocked,
                    "route_or_profile_authorization_rejected_restart_required",
                )
                .await;
                return;
            }
            Err(_) => {
                set_readiness(&readiness, ReadinessPhase::Blocked, "receive_unavailable").await;
                RETRY_DELAY
            }
        };
        if !delay.is_zero() && !wait_or_shutdown(&mut shutdown, delay).await {
            break;
        }
    }
    set_readiness(&readiness, ReadinessPhase::Stopped, "consumer_stopped").await;
}

async fn wait_or_shutdown(shutdown: &mut watch::Receiver<bool>, delay: Duration) -> bool {
    tokio::select! {
        _ = sleep(delay) => !*shutdown.borrow(),
        changed = shutdown.changed() => changed.is_ok() && !*shutdown.borrow(),
    }
}

async fn set_readiness(readiness: &SharedReadiness, phase: ReadinessPhase, reason: &'static str) {
    *readiness.write().await = ReadinessSnapshot::new(phase, reason);
}

async fn readiness_endpoint(
    State(state): State<ReadinessRouteState>,
) -> (StatusCode, Json<serde_json::Value>) {
    let snapshot = state.readiness.read().await.clone();
    let worker_state = state.worker_state.load(Ordering::Acquire);
    let (phase, reason) = effective_readiness(&snapshot, worker_state);
    let status_code = if phase == ReadinessPhase::Ready && worker_state == WORKER_RUNNING {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(serde_json::json!({
            "component": "bi-task-event-ingestion",
            "status": phase.as_str(),
            "reason": reason,
        })),
    )
}

fn effective_readiness(
    snapshot: &ReadinessSnapshot,
    worker_state: u8,
) -> (ReadinessPhase, &'static str) {
    match worker_state {
        WORKER_PANICKED => Some((ReadinessPhase::Blocked, "consumer_panicked")),
        WORKER_STOPPED if snapshot.phase == ReadinessPhase::Ready => {
            Some((ReadinessPhase::Blocked, "consumer_stopped_unexpectedly"))
        }
        _ => None,
    }
    .unwrap_or((snapshot.phase, snapshot.reason))
}

fn same_route_pin(left: &TaskEventRoutePin, right: &TaskEventRoutePin) -> bool {
    left.sender_service_id == right.sender_service_id
        && left.receiver_service_id == right.receiver_service_id
        && left.transport_provider_id == right.transport_provider_id
        && left.transport_provider_version == right.transport_provider_version
        && left.transport_profile_id == right.transport_profile_id
        && left.transport_profile_revision == right.transport_profile_revision
        && left.transport_profile_digest == right.transport_profile_digest
        && left.endpoint_identity == right.endpoint_identity
        && left.destination_id == right.destination_id
        && left.destination_generation == right.destination_generation
}

fn route_matches_scope(route: &TaskEventReceiveRoute, scope: NativeGrpcEventReceiveScope) -> bool {
    route.tenant_id() == scope.tenant_id()
        && route.pin().sender_service_id == scope.sender_service_id()
        && route.pin().receiver_service_id == scope.receiver_service_id()
}

#[cfg(test)]
mod tests {
    use super::{
        BiTaskEventIngestion, ENABLED_ENV, ReadinessPhase, ReadinessSnapshot, RuntimeConfig,
        TaskEventRoutePin, WRITE_DATABASE_CAPABILITY_QUERY, WriteDatabaseCapabilities,
        parse_opt_in, profile_pin_matches, writer_role_is_separate,
    };
    use std::collections::HashMap;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    };
    use tokio::sync::{RwLock, watch};
    use uuid::Uuid;

    fn restricted_capabilities() -> WriteDatabaseCapabilities {
        WriteDatabaseCapabilities {
            effective_runtime_member: true,
            authenticated_runtime_member: true,
            effective_can_commit: true,
            effective_can_quarantine: true,
            ..WriteDatabaseCapabilities::default()
        }
    }

    #[test]
    fn ingestion_requires_explicit_valid_opt_in() {
        assert_eq!(parse_opt_in(None), Ok(false));
        assert_eq!(parse_opt_in(Some("false")), Ok(false));
        assert_eq!(parse_opt_in(Some("0")), Ok(false));
        assert_eq!(parse_opt_in(Some("true")), Ok(true));
        assert_eq!(parse_opt_in(Some("1")), Ok(true));
        assert!(parse_opt_in(Some("yes")).is_err());
    }

    #[test]
    fn incomplete_or_empty_runtime_identity_cannot_build_a_consumer_config() {
        let mut values = HashMap::new();
        values.insert(ENABLED_ENV.to_owned(), "true".to_owned());
        assert!(RuntimeConfig::from_values(&values).is_err());
    }

    #[test]
    fn restricted_write_role_requires_runtime_membership_and_only_bi_functions() {
        assert!(restricted_capabilities().is_restricted());

        let mut capabilities = restricted_capabilities();
        capabilities.effective_runtime_member = false;
        assert!(!capabilities.is_restricted());

        let mut capabilities = restricted_capabilities();
        capabilities.effective_owner_member = true;
        assert!(!capabilities.is_restricted());

        let mut capabilities = restricted_capabilities();
        capabilities.effective_can_commit = false;
        assert!(!capabilities.is_restricted());

        let mut capabilities = restricted_capabilities();
        capabilities.effective_can_quarantine = false;
        assert!(!capabilities.is_restricted());
    }

    #[test]
    fn write_role_must_be_distinct_from_application_and_query_roles() {
        assert!(writer_role_is_separate(
            "star_bi_runtime",
            "star_bi_runtime_login",
            "star_app",
            "star_app_login",
            "star_bi_read",
            "star_bi_read_login",
        ));
        assert!(!writer_role_is_separate(
            "star_app",
            "star_bi_runtime_login",
            "star_app",
            "star_app_login",
            "star_bi_read",
            "star_bi_read_login",
        ));
        assert!(!writer_role_is_separate(
            "star_bi_runtime",
            "star_app_login",
            "star_app",
            "star_app_login",
            "star_bi_read",
            "star_bi_read_login",
        ));
        assert!(!writer_role_is_separate(
            "star_bi_runtime",
            "star_bi_runtime_login",
            "star_app",
            "star_app_login",
            "star_bi_read",
            "star_bi_runtime_login",
        ));
        assert!(!writer_role_is_separate(
            " ",
            "star_bi_runtime_login",
            "star_app",
            "star_app_login",
            "star_bi_read",
            "star_bi_read_login",
        ));
    }

    #[test]
    fn readiness_does_not_remain_ready_after_consumer_failure() {
        let ready = super::ReadinessSnapshot::new(super::ReadinessPhase::Ready, "receive_ok");
        assert_eq!(
            super::effective_readiness(&ready, super::WORKER_PANICKED),
            (super::ReadinessPhase::Blocked, "consumer_panicked")
        );
        assert_eq!(
            super::effective_readiness(&ready, super::WORKER_STOPPED),
            (
                super::ReadinessPhase::Blocked,
                "consumer_stopped_unexpectedly"
            )
        );
        assert_eq!(
            super::effective_readiness(&ready, super::WORKER_RUNNING),
            (super::ReadinessPhase::Ready, "receive_ok")
        );
    }

    #[test]
    fn profile_pin_monitor_detects_revision_digest_and_endpoint_changes() {
        let expected = TaskEventRoutePin {
            sender_service_id: Uuid::from_u128(1),
            receiver_service_id: Uuid::from_u128(2),
            transport_provider_id: "star-rust-log".to_owned(),
            transport_provider_version: "1".to_owned(),
            transport_profile_id: Uuid::from_u128(3),
            transport_profile_revision: 7,
            transport_profile_digest: [5; 32],
            endpoint_identity: "bi-ingress".to_owned(),
            destination_id: "bi-task-events".to_owned(),
            destination_generation: Uuid::from_u128(4),
        };
        let deployment_id = Uuid::from_u128(5);
        let tenant_id = Uuid::from_u128(6);
        assert!(profile_pin_matches(
            &expected,
            deployment_id,
            tenant_id,
            deployment_id,
            tenant_id,
            expected.transport_profile_id,
            expected.transport_profile_revision,
            expected.transport_profile_digest,
            "bi-ingress",
        ));
        assert!(!profile_pin_matches(
            &expected,
            deployment_id,
            tenant_id,
            deployment_id,
            tenant_id,
            expected.transport_profile_id,
            expected.transport_profile_revision + 1,
            expected.transport_profile_digest,
            "bi-ingress",
        ));
        assert!(!profile_pin_matches(
            &expected,
            deployment_id,
            tenant_id,
            deployment_id,
            tenant_id,
            expected.transport_profile_id,
            expected.transport_profile_revision,
            [9; 32],
            "bi-ingress",
        ));
        assert!(!profile_pin_matches(
            &expected,
            deployment_id,
            tenant_id,
            deployment_id,
            tenant_id,
            expected.transport_profile_id,
            expected.transport_profile_revision,
            expected.transport_profile_digest,
            "new-bi-ingress",
        ));
    }

    #[test]
    fn write_role_rejects_privileges_that_bypass_owner_boundaries() {
        let mutations: [fn(&mut WriteDatabaseCapabilities); 15] = [
            |c: &mut WriteDatabaseCapabilities| c.effective_is_superuser = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_bypasses_rls = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_can_manage_roles = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_can_create_databases = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_can_replicate = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_schema_create = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_direct_table_access = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_is_superuser = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_bypasses_rls = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_can_manage_roles = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_can_create_databases = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_can_replicate = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_owner_role_settable = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_schema_create = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_direct_table_access = true,
        ];
        for mutate in mutations {
            let mut capabilities = restricted_capabilities();
            mutate(&mut capabilities);
            assert!(!capabilities.is_restricted());
        }
    }

    #[test]
    fn write_role_query_checks_truncate_and_authenticated_session_privileges() {
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("'TRUNCATE'::text"));
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(current_user, owner_role.rolname, 'MEMBER')")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(session_user, owner_role.rolname, 'SET')")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(current_user, star_role.rolname, 'MEMBER')")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(current_user, star_role.rolname, 'SET')")
        );
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("star_role.rolname <> 'star_bi_runtime'"));
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(session_user, 'star_bi_runtime', 'MEMBER')")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(session_user, star_role.rolname, 'MEMBER')")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("pg_has_role(session_user, star_role.rolname, 'SET')")
        );
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("AS effective_non_bi_star_role_member"));
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("AS effective_transport_execute"));
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY.contains("AS authenticated_non_bi_star_role_member")
        );
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("AS authenticated_transport_execute"));
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("WHERE star_role.rolname ~ '^star_'"));
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("AND star_role.rolname <> current_user"));
        assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains("AND star_role.rolname <> session_user"));
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("AND authenticated_role.rolname = session_user")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("has_function_privilege(\n                   current_user,")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("has_function_privilege(\n                   session_user,")
        );
        for signature in [
            "platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)",
            "platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)",
            "platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)",
            "platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)",
            "platform_transport.pull_one(text,uuid,integer)",
            "platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)",
            "platform_transport.purge_terminal(text,uuid,integer)",
        ] {
            assert!(WRITE_DATABASE_CAPABILITY_QUERY.contains(signature));
        }
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY.contains("right(owner_role.rolname, 6) = '_owner'")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("authenticated_role.rolcreaterole AS authenticated_can_manage_roles")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("effective_role.rolcreatedb AS effective_can_create_databases")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("effective_role.rolreplication AS effective_can_replicate")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("authenticated_role.rolcreatedb AS authenticated_can_create_databases")
        );
        assert!(
            WRITE_DATABASE_CAPABILITY_QUERY
                .contains("authenticated_role.rolreplication AS authenticated_can_replicate")
        );
    }

    #[test]
    fn write_role_rejects_other_star_memberships_and_transport_execute() {
        let mutations: [fn(&mut WriteDatabaseCapabilities); 5] = [
            |c: &mut WriteDatabaseCapabilities| c.authenticated_runtime_member = false,
            |c: &mut WriteDatabaseCapabilities| c.effective_non_bi_star_role_member = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_non_bi_star_role_member = true,
            |c: &mut WriteDatabaseCapabilities| c.effective_transport_execute = true,
            |c: &mut WriteDatabaseCapabilities| c.authenticated_transport_execute = true,
        ];
        for mutate in mutations {
            let mut capabilities = restricted_capabilities();
            mutate(&mut capabilities);
            assert!(!capabilities.is_restricted());
        }
    }

    fn ingestion_with_worker(worker: tokio::task::JoinHandle<()>) -> BiTaskEventIngestion {
        let (shutdown, _) = watch::channel(false);
        BiTaskEventIngestion {
            readiness: Arc::new(RwLock::new(ReadinessSnapshot::new(
                ReadinessPhase::Ready,
                "receive_ok",
            ))),
            worker_state: Arc::new(AtomicU8::new(super::WORKER_RUNNING)),
            shutdown,
            worker: Some(worker),
        }
    }

    #[tokio::test]
    async fn shutdown_propagates_consumer_join_failure() {
        let worker = tokio::spawn(async { panic!("simulated consumer panic") });
        let mut ingestion = ingestion_with_worker(worker);

        let error = ingestion
            .shutdown()
            .await
            .expect_err("join failure must propagate");

        assert_eq!(error, "consumer_task_failed");
        assert_eq!(
            ingestion.readiness.read().await.phase,
            ReadinessPhase::Blocked
        );
    }

    #[tokio::test]
    async fn shutdown_propagates_drain_timeout() {
        let worker = tokio::spawn(std::future::pending::<()>());
        let mut ingestion = ingestion_with_worker(worker);

        let error = ingestion
            .shutdown_with_timeout(std::time::Duration::from_millis(10))
            .await
            .expect_err("drain timeout must propagate");

        assert_eq!(error, "shutdown_drain_timeout");
        assert_eq!(
            ingestion.readiness.read().await.phase,
            ReadinessPhase::Blocked
        );
    }

    struct DropObserved(Arc<AtomicBool>);

    impl Drop for DropObserved {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn serve_future_is_dropped_when_shutdown_signal_fires() {
        let dropped = Arc::new(AtomicBool::new(false));
        let drop_observed = DropObserved(dropped.clone());
        let serve = std::future::poll_fn(move |_| {
            let _ = &drop_observed;
            std::task::Poll::<std::io::Result<()>>::Pending
        });

        let result = super::serve_until_shutdown(serve, std::future::ready(())).await;

        assert!(result.is_ok());
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn serve_io_error_is_propagated() {
        let serve = std::future::ready(Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "simulated server failure",
        )));

        let error = super::serve_until_shutdown(serve, std::future::ready(()))
            .await
            .expect_err("server I/O failures must propagate");

        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }

    #[test]
    fn main_binds_listener_before_starting_service_resources() {
        let main_source = include_str!("main.rs");
        let bind_position = main_source
            .find("let listener = tokio::net::TcpListener::bind(bind_addr).await?;")
            .expect("main binds its listener");

        for startup_call in [
            "JwtConfig::verifier_from_env()",
            "connect_pool(&pg_config)",
            "CanvasEventWakeHub::start(pool.clone())",
            "BiTaskEventIngestion::configure(pool.clone(), bi_query_pool)",
        ] {
            let startup_position = main_source
                .find(startup_call)
                .unwrap_or_else(|| panic!("main contains {startup_call}"));
            assert!(
                bind_position < startup_position,
                "listener bind must precede {startup_call}"
            );
        }
    }
}
