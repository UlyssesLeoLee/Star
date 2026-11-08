// SPDX-License-Identifier: MIT
/*
@cypher schema=1 source_sha256=a8b3c8cf080857f88a0de4b7a72e445f3ee18bd4e3001dc00d73bba8df0c6aa7
MERGE (self:File {path:"crates/star-api-rest/tests/bi_task_event_commit_before_ack_pg.rs"})
MERGE (test:Symbol {id:"crates/star-api-rest/tests/bi_task_event_commit_before_ack_pg.rs::native_event_commits_before_ack_and_exact_replay_is_idempotent",kind:"test"})
MERGE (receiver:Type {id:"bi_owner_runtime::TaskEventReceiver",kind:"struct"})
MERGE (native_transport:Type {id:"platform_messaging_runtime::NativeGrpcEventReceiveTransport",kind:"struct"})
MERGE (inbox:Type {id:"bi_owner_runtime::TaskEventInboxStore",kind:"struct"})
MERGE (ack_latch:Type {id:"crates/star-api-rest/tests/bi_task_event_commit_before_ack_pg.rs::AckLatch",kind:"struct"})
MERGE (database:Config {id:"STAR_BI_TASK_EVENT_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (transport_database:Config {id:"STAR_PLATFORM_PROFILE_TEST_DATABASE_URL",kind:"environment_variable"})
MERGE (test)-[:USES_TYPE]->(receiver)
MERGE (test)-[:USES_TYPE]->(native_transport)
MERGE (test)-[:USES_TYPE]->(inbox)
MERGE (test)-[:USES_TYPE]->(ack_latch)
MERGE (test)-[:READS]->(database)
MERGE (test)-[:READS]->(transport_database)
@endcypher
*/

//! PostgreSQL-backed native Event receive acceptance.
//!
//! The test is ignored by default. Its runner must provision separate disposable BI and native
//! transport databases, an authorized remote native gRPC endpoint, and dedicated mTLS identities.
//! It seeds one Event through the authenticated database function; append-over-gRPC is outside the
//! assertion. Pull, origin verification, and acknowledgement all use the real native mTLS client.

use std::{
    collections::HashMap,
    env,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bi_owner_runtime::{
    TaskEventInboxStore, TaskEventQuarantineStore, TaskEventReceiveAuthorization,
    TaskEventReceiveDelivery, TaskEventReceiveError, TaskEventReceiveOrigin,
    TaskEventReceiveOutcome, TaskEventReceiveRoute, TaskEventReceiveTransport, TaskEventReceiver,
    TaskEventRoutePin,
};
use chrono::Utc;
use platform_messaging_runtime::{
    admission_delivery::AdmissionTransportProfile,
    native_grpc::{
        NativeGrpcEventDelivery, NativeGrpcEventOrigin, NativeGrpcEventReceiveAuthorization,
        NativeGrpcEventReceiveClientConfig, NativeGrpcEventReceiveScope,
        NativeGrpcEventReceiveTransport, native_provider_identity,
    },
    transport_endpoint::{TrustedEndpointConfig, TrustedEndpointResolver},
    transport_profile::{
        OpaqueSecretReference, TransportMessageKind, TransportProfileDefinition,
        TransportProfileLifecycle, TransportProfileRevision,
    },
};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tokio::sync::Notify;
use uuid::Uuid;

const BI_DATABASE_URL_ENV: &str = "STAR_BI_TASK_EVENT_TEST_DATABASE_URL";
const BI_OBSERVER_DATABASE_URL_ENV: &str = "STAR_BI_TASK_EVENT_TEST_OBSERVER_DATABASE_URL";
const TRANSPORT_DATABASE_URL_ENV: &str = "STAR_PLATFORM_PROFILE_TEST_DATABASE_URL";
const TRANSPORT_OBSERVER_DATABASE_URL_ENV: &str =
    "STAR_PLATFORM_PROFILE_TEST_OBSERVER_DATABASE_URL";
const BI_DATABASE_NAME: &str = "star_bi_task_event_test";
const TRANSPORT_DATABASE_NAME: &str = "star_d19_test";
const PROVIDER_ID: &str = "star-rust-log";
const TASK_EVENT_RECEIVE_LEASE_SECONDS: u32 = 30;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct OriginKey {
    provider_identity: String,
    log_id: String,
    log_generation: Uuid,
    sequence: i64,
    message_id: Uuid,
    wire_sha256: [u8; 32],
}

impl OriginKey {
    fn from_native(origin: &NativeGrpcEventOrigin, wire: &[u8]) -> Self {
        Self {
            provider_identity: origin.provider_identity().to_owned(),
            log_id: origin.log_id().to_owned(),
            log_generation: origin.log_generation(),
            sequence: origin.sequence(),
            message_id: origin.message_id(),
            wire_sha256: Sha256::digest(wire).into(),
        }
    }

    fn from_task(origin: &TaskEventReceiveOrigin, wire: &[u8]) -> Self {
        Self {
            provider_identity: origin.provider_identity().to_owned(),
            log_id: origin.log_id().to_owned(),
            log_generation: origin.log_generation(),
            sequence: origin.sequence(),
            message_id: origin.message_id(),
            wire_sha256: Sha256::digest(wire).into(),
        }
    }
}

#[derive(Clone, Default)]
struct NativeOriginCache(Arc<Mutex<HashMap<OriginKey, NativeGrpcEventOrigin>>>);

impl NativeOriginCache {
    fn replace(
        &self,
        origin: NativeGrpcEventOrigin,
        wire: &[u8],
    ) -> Result<(), TaskEventReceiveError> {
        let key = OriginKey::from_native(&origin, wire);
        let mut entries = self
            .0
            .lock()
            .map_err(|_| TaskEventReceiveError::Unauthorized)?;
        // This test has one route and one in-flight delivery; keep the authorization cache bounded.
        entries.clear();
        entries.insert(key, origin);
        Ok(())
    }

    fn get(
        &self,
        origin: &TaskEventReceiveOrigin,
        wire: &[u8],
    ) -> Result<NativeGrpcEventOrigin, TaskEventReceiveError> {
        self.0
            .lock()
            .map_err(|_| TaskEventReceiveError::Unauthorized)?
            .get(&OriginKey::from_task(origin, wire))
            .cloned()
            .ok_or(TaskEventReceiveError::Unauthorized)
    }
}

#[derive(Default)]
struct AckLatch {
    first_ack: AtomicBool,
    reached: Notify,
}

impl AckLatch {
    fn armed() -> Self {
        Self {
            first_ack: AtomicBool::new(true),
            reached: Notify::new(),
        }
    }

    async fn pause_first_ack(&self) {
        if self.first_ack.swap(false, Ordering::AcqRel) {
            self.reached.notify_one();
            std::future::pending::<()>().await;
        }
    }

    async fn wait_until_reached(&self) {
        self.reached.notified().await;
    }
}

struct LatchedNativeDelivery {
    inner: NativeGrpcEventDelivery,
    origin: TaskEventReceiveOrigin,
    latch: Arc<AckLatch>,
}

impl TaskEventReceiveDelivery for LatchedNativeDelivery {
    fn exact_wire(&self) -> &[u8] {
        self.inner.exact_wire()
    }

    fn origin(&self) -> &TaskEventReceiveOrigin {
        &self.origin
    }

    async fn acknowledge(&self) -> Result<(), TaskEventReceiveError> {
        self.latch.pause_first_ack().await;
        self.inner
            .acknowledge()
            .await
            .map_err(|_| TaskEventReceiveError::Transport)
    }
}

struct LatchedNativeTransport {
    inner: NativeGrpcEventReceiveTransport,
    scope: NativeGrpcEventReceiveScope,
    route_pin: TaskEventRoutePin,
    origins: NativeOriginCache,
    latch: Arc<AckLatch>,
}

impl TaskEventReceiveTransport for LatchedNativeTransport {
    type Delivery = LatchedNativeDelivery;

    async fn pull_one(
        &self,
        route: &TaskEventReceiveRoute,
    ) -> Result<Option<Self::Delivery>, TaskEventReceiveError> {
        if !route_matches_pin(route.pin(), &self.route_pin) {
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
        self.origins.replace(native_origin, inner.exact_wire())?;
        Ok(Some(LatchedNativeDelivery {
            inner,
            origin,
            latch: Arc::clone(&self.latch),
        }))
    }
}

struct NativeAuthorizationAdapter {
    inner: NativeGrpcEventReceiveAuthorization,
    scope: NativeGrpcEventReceiveScope,
    route_pin: TaskEventRoutePin,
    origins: NativeOriginCache,
}

impl TaskEventReceiveAuthorization for NativeAuthorizationAdapter {
    async fn verify_origin(
        &self,
        route: &TaskEventReceiveRoute,
        origin: &TaskEventReceiveOrigin,
        exact_wire: &[u8],
    ) -> Result<(), TaskEventReceiveError> {
        if !route_matches_pin(route.pin(), &self.route_pin) {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        let native_origin = self.origins.get(origin, exact_wire)?;
        self.inner
            .verify_origin(self.scope, &native_origin, exact_wire)
            .await
            .map_err(|_| TaskEventReceiveError::Unauthorized)
    }
}

fn route_matches_pin(actual: &TaskEventRoutePin, expected: &TaskEventRoutePin) -> bool {
    actual.sender_service_id == expected.sender_service_id
        && actual.receiver_service_id == expected.receiver_service_id
        && actual.transport_provider_id == expected.transport_provider_id
        && actual.transport_provider_version == expected.transport_provider_version
        && actual.transport_profile_id == expected.transport_profile_id
        && actual.transport_profile_revision == expected.transport_profile_revision
        && actual.transport_profile_digest == expected.transport_profile_digest
        && actual.endpoint_identity == expected.endpoint_identity
        && actual.destination_id == expected.destination_id
        && actual.destination_generation == expected.destination_generation
}

#[tokio::test]
#[ignore = "requires disposable BI/transport PostgreSQL databases and a provisioned native gRPC+mTLS Event route"]
async fn native_event_commits_before_ack_and_exact_replay_is_idempotent() {
    let bi_database_url = required_env(BI_DATABASE_URL_ENV);
    let bi_observer_url = required_env(BI_OBSERVER_DATABASE_URL_ENV);
    let transport_database_url = required_env(TRANSPORT_DATABASE_URL_ENV);
    let transport_observer_url = required_env(TRANSPORT_OBSERVER_DATABASE_URL_ENV);

    // Reject public or unresolved database hosts before opening any connection.
    require_isolated_database_host(&bi_database_url);
    require_isolated_database_host(&bi_observer_url);
    require_isolated_database_host(&transport_database_url);
    require_isolated_database_host(&transport_observer_url);

    let bi_pool = connect_pool(&bi_database_url, 4).await;
    let bi_observer = connect_pool(&bi_observer_url, 4).await;
    let transport_append_pool = connect_pool(&transport_database_url, 2).await;
    let transport_observer = connect_pool(&transport_observer_url, 4).await;
    require_database_name(&bi_pool, BI_DATABASE_NAME).await;
    require_database_name(&bi_observer, BI_DATABASE_NAME).await;
    require_database_name(&transport_append_pool, TRANSPORT_DATABASE_NAME).await;
    require_database_name(&transport_observer, TRANSPORT_DATABASE_NAME).await;

    // The route is intentionally fixed to one disposable test profile/log. Serialize test
    // processes that share the source database before appending or pulling from that route.
    let mut route_lock = transport_append_pool
        .begin()
        .await
        .expect("begin source-route test lock transaction");
    let route_lock_acquired: bool = sqlx::query_scalar(
        "SELECT pg_try_advisory_xact_lock(hashtextextended(current_database() || ':star_9f4j4_event_receive_acceptance', 0))",
    )
    .fetch_one(&mut *route_lock)
    .await
    .expect("acquire source-route test lock");
    assert!(
        route_lock_acquired,
        "another acceptance test is using this native Event route"
    );

    let deployment_id = required_uuid("STAR_BI_TASK_EVENT_TEST_DEPLOYMENT_ID");
    let tenant_id = required_uuid("STAR_BI_TASK_EVENT_TEST_TENANT_ID");
    let profile_id = required_uuid("STAR_BI_TASK_EVENT_TEST_PROFILE_ID");
    let profile_revision_number = required_u64("STAR_BI_TASK_EVENT_TEST_PROFILE_REVISION") as i64;
    let sender_service_id = required_uuid("STAR_BI_TASK_EVENT_TEST_SENDER_SERVICE_ID");
    let receiver_service_id = required_uuid("STAR_BI_TASK_EVENT_TEST_RECEIVER_SERVICE_ID");
    let destination_id = required_env("STAR_BI_TASK_EVENT_TEST_DESTINATION_ID");
    let destination_generation = required_uuid("STAR_BI_TASK_EVENT_TEST_DESTINATION_GENERATION");
    let provider_version = required_env("STAR_BI_TASK_EVENT_TEST_PROVIDER_VERSION");
    let endpoint_identity = required_env("STAR_BI_TASK_EVENT_TEST_ENDPOINT_IDENTITY");
    let endpoint_uri = required_env("STAR_BI_TASK_EVENT_TEST_ENDPOINT_URI");
    let tls_server_name = required_env("STAR_BI_TASK_EVENT_TEST_TLS_SERVER_NAME");
    let generation_policy = required_env("STAR_BI_TASK_EVENT_TEST_LOG_GENERATION_POLICY");
    let sender_certificate_sha256 = required_sha256("STAR_BI_TASK_EVENT_TEST_SENDER_CERT_SHA256");
    let server_ca_path = required_path("STAR_BI_TASK_EVENT_TEST_SERVER_CA_PEM");
    let receiver_certificate_path = required_path("STAR_BI_TASK_EVENT_TEST_RECEIVER_CERT_PEM");
    let receiver_private_key_path = required_path("STAR_BI_TASK_EVENT_TEST_RECEIVER_KEY_PEM");

    let secret_reference = env::var("STAR_BI_TASK_EVENT_TEST_PROFILE_SECRET_REFERENCE")
        .ok()
        .map(|value| {
            OpaqueSecretReference::new(value)
                .expect("test profile secret reference must be an opaque locator")
        });
    let profile_definition = TransportProfileDefinition::new(
        PROVIDER_ID.to_owned(),
        provider_version.clone(),
        endpoint_identity.clone(),
        generation_policy,
        vec![sender_service_id],
        vec![receiver_service_id],
        vec![TransportMessageKind::Event],
        secret_reference,
    )
    .expect("test profile definition must satisfy the production profile contract");
    let profile_revision = TransportProfileRevision::new(
        deployment_id,
        tenant_id,
        profile_id,
        profile_revision_number,
        TransportProfileLifecycle::Active,
        profile_definition,
    )
    .expect("test profile revision must be valid");
    let endpoint_resolver = TrustedEndpointResolver::new(
        deployment_id,
        vec![
            TrustedEndpointConfig::new(endpoint_identity.clone(), endpoint_uri, tls_server_name)
                .expect("test endpoint must satisfy the trusted HTTPS endpoint contract"),
        ],
    )
    .expect("test resolver must have one trusted endpoint");
    let endpoint = endpoint_resolver
        .resolve(&profile_revision)
        .expect("profile endpoint must resolve through the trusted allow-list");
    let profile = AdmissionTransportProfile::new(
        PROVIDER_ID.to_owned(),
        provider_version.clone(),
        profile_revision.digest(),
        endpoint_identity.clone(),
    )
    .expect("native transport profile snapshot must be valid");
    let scope = NativeGrpcEventReceiveScope::new(tenant_id, sender_service_id, receiver_service_id)
        .expect("Task event route identities must be valid");

    let event_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let engineering_run_id = Uuid::new_v4();
    let work_item_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let wire = task_event_wire(
        tenant_id,
        project_id,
        engineering_run_id,
        event_id,
        work_item_id,
        actor_id,
        correlation_id,
    );
    let wire_sha256: [u8; 32] = Sha256::digest(&wire).into();

    // Seed only the source Event through the authenticated stored function. This does not test the
    // append RPC; the real mTLS pull, origin verification, and ACK calls follow below.
    let seeded = sqlx::query(
        "SELECT accepted_log_id, accepted_generation, accepted_sequence, accepted_sha256, exact_replay \
         FROM platform_transport.append_authenticated($1,$2,$3,'event',$4,$5,$6)",
    )
    .bind(sender_certificate_sha256.to_vec())
    .bind(profile_revision.digest().to_vec())
    .bind(receiver_service_id)
    .bind(event_id)
    .bind(&wire)
    .bind(wire_sha256.to_vec())
    .fetch_one(&transport_append_pool)
    .await
    .expect("append one test Event through the authenticated native transport function");
    let log_id: String = seeded
        .try_get("accepted_log_id")
        .expect("read accepted log ID");
    let log_generation: Uuid = seeded
        .try_get("accepted_generation")
        .expect("read accepted log generation");
    let sequence: i64 = seeded
        .try_get("accepted_sequence")
        .expect("read accepted sequence");
    let accepted_sha256: Vec<u8> = seeded
        .try_get("accepted_sha256")
        .expect("read accepted digest");
    let exact_replay: bool = seeded
        .try_get("exact_replay")
        .expect("read append replay flag");
    assert!(
        !exact_replay,
        "fresh test Event must be appended exactly once"
    );
    assert_eq!(accepted_sha256.as_slice(), wire_sha256);
    assert!(sequence > 0);
    assert_source_route_has_one_open_delivery(
        &transport_observer,
        deployment_id,
        tenant_id,
        &log_id,
        log_generation,
        event_id,
    )
    .await;

    let native_transport =
        NativeGrpcEventReceiveTransport::connect(NativeGrpcEventReceiveClientConfig {
            endpoint: &endpoint,
            server_ca_path: &server_ca_path,
            client_cert_path: &receiver_certificate_path,
            client_key_path: &receiver_private_key_path,
            connect_timeout: Duration::from_secs(3),
            request_timeout: Duration::from_secs(8),
            scope,
            expected_log_id: &log_id,
            expected_log_generation: log_generation,
            lease_seconds: TASK_EVENT_RECEIVE_LEASE_SECONDS,
            profile: profile.clone(),
        })
        .await
        .expect("connect to the trusted native gRPC+mTLS Event endpoint");
    let route_pin = TaskEventRoutePin {
        sender_service_id,
        receiver_service_id,
        transport_provider_id: native_provider_identity(&profile),
        transport_provider_version: provider_version,
        transport_profile_id: profile_id,
        transport_profile_revision: profile_revision_number,
        transport_profile_digest: profile_revision.digest(),
        endpoint_identity,
        destination_id,
        destination_generation,
    };
    let route = TaskEventReceiveRoute::new(tenant_id, route_pin.clone())
        .expect("test Task Event receive route must be valid");
    let origins = NativeOriginCache::default();
    let latch = Arc::new(AckLatch::armed());
    let native_authorization = native_transport.authorization();
    let receiver = Arc::new(TaskEventReceiver::new(
        route,
        LatchedNativeTransport {
            inner: native_transport,
            scope,
            route_pin: route_pin.clone(),
            origins: origins.clone(),
            latch: Arc::clone(&latch),
        },
        NativeAuthorizationAdapter {
            inner: native_authorization,
            scope,
            route_pin,
            origins,
        },
        TaskEventInboxStore::new(bi_pool.clone(), receiver_service_id),
        TaskEventQuarantineStore::new(bi_pool.clone()),
    ));

    let first_receiver = Arc::clone(&receiver);
    let first_attempt = tokio::spawn(async move {
        let cancellation = Default::default();
        first_receiver
            .receive_one(
                &cancellation,
                tokio::time::Instant::now() + Duration::from_secs(15),
            )
            .await
    });
    tokio::time::timeout(Duration::from_secs(15), latch.wait_until_reached())
        .await
        .expect("receiver must reach the ACK latch after committing BI Inbox and fact");

    let committed_before_ack = observe_bi(&bi_observer, tenant_id, event_id).await;
    assert_eq!(committed_before_ack.fact_count, 1);
    assert_eq!(committed_before_ack.wire_sha256.as_slice(), wire_sha256);
    let source_before_ack = observe_source(
        &transport_observer,
        deployment_id,
        tenant_id,
        &log_id,
        log_generation,
        sequence,
        event_id,
    )
    .await;
    assert_eq!(source_before_ack.state, "leased");
    assert_eq!(source_before_ack.ack_receipt_count, 0);

    // Simulate a process interruption after the owner commit but before the first ACK RPC.
    first_attempt.abort();
    assert!(
        first_attempt
            .await
            .expect_err("pre-ACK receiver task must be aborted")
            .is_cancelled()
    );

    let lease_remaining = Duration::from_millis(source_before_ack.lease_remaining_ms.max(0) as u64);
    assert!(
        lease_remaining <= Duration::from_secs(31),
        "test lease exceeds its configured 30 second bound"
    );
    tokio::time::sleep(lease_remaining + Duration::from_millis(150)).await;

    let cancellation = Default::default();
    let replay_outcome = receiver
        .receive_one(
            &cancellation,
            tokio::time::Instant::now() + Duration::from_secs(15),
        )
        .await
        .expect("exact source replay must commit idempotently and ACK over native gRPC");
    let replay_receipt = match replay_outcome {
        TaskEventReceiveOutcome::ExactReplay(receipt) => receipt,
        other => panic!("expected BI exact replay after lease expiry, got {other:?}"),
    };
    assert_eq!(replay_receipt.receipt_id, committed_before_ack.receipt_id);

    let committed_after_replay = observe_bi(&bi_observer, tenant_id, event_id).await;
    assert_eq!(
        committed_after_replay.receipt_id,
        committed_before_ack.receipt_id
    );
    assert_eq!(committed_after_replay.fact_count, 1);
    let source_after_ack = observe_source(
        &transport_observer,
        deployment_id,
        tenant_id,
        &log_id,
        log_generation,
        sequence,
        event_id,
    )
    .await;
    assert_eq!(source_after_ack.state, "acked");
    assert_eq!(source_after_ack.ack_receipt_count, 1);
    assert_eq!(source_after_ack.accepted_receipt_count, 1);

    route_lock
        .commit()
        .await
        .expect("release source-route test lock");
    bi_pool.close().await;
    bi_observer.close().await;
    transport_append_pool.close().await;
    transport_observer.close().await;
}

struct BiObservation {
    receipt_id: Uuid,
    wire_sha256: Vec<u8>,
    fact_count: i64,
}

async fn observe_bi(pool: &PgPool, tenant_id: Uuid, event_id: Uuid) -> BiObservation {
    let row = sqlx::query(
        "SELECT receipt_id, wire_sha256 FROM bi.task_event_inbox \
         WHERE tenant_id = $1 AND consumer_id = 'bi.task-event-fact.v1' AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .fetch_one(pool)
    .await
    .expect("independent BI observer must see the committed Inbox receipt");
    let receipt_id: Uuid = row
        .try_get("receipt_id")
        .expect("read Inbox receipt identity");
    let wire_sha256: Vec<u8> = row
        .try_get("wire_sha256")
        .expect("read Inbox exact-wire digest");
    let fact_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM bi.task_event_fact WHERE tenant_id = $1 AND consumer_id = 'bi.task-event-fact.v1' AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .fetch_one(pool)
    .await
    .expect("independent BI observer must read committed fact count");
    BiObservation {
        receipt_id,
        wire_sha256,
        fact_count,
    }
}

struct SourceObservation {
    state: String,
    lease_remaining_ms: i64,
    accepted_receipt_count: i64,
    ack_receipt_count: i64,
}

async fn observe_source(
    pool: &PgPool,
    deployment_id: Uuid,
    tenant_id: Uuid,
    log_id: &str,
    generation: Uuid,
    sequence: i64,
    message_id: Uuid,
) -> SourceObservation {
    let row = sqlx::query(
        "SELECT state, COALESCE(GREATEST(0, CEIL(EXTRACT(EPOCH FROM lease_expires_at - clock_timestamp()) * 1000)::BIGINT), 0) AS lease_remaining_ms \
         FROM platform_transport.message_work \
         WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3 AND generation = $4 \
           AND sequence = $5 AND message_id = $6",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(log_id)
    .bind(generation)
    .bind(sequence)
    .bind(message_id)
    .fetch_one(pool)
    .await
    .expect("independent transport observer must see the source delivery");
    let state: String = row.try_get("state").expect("read source delivery state");
    let lease_remaining_ms: i64 = row
        .try_get("lease_remaining_ms")
        .expect("read source lease remaining time from the PostgreSQL clock");
    let accepted_receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM platform_transport.delivery_receipt \
         WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3 AND generation = $4 \
           AND sequence = $5 AND action = 'accepted'",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(log_id)
    .bind(generation)
    .bind(sequence)
    .fetch_one(pool)
    .await
    .expect("independent transport observer must read accepted receipt count");
    let ack_receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM platform_transport.delivery_receipt \
         WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3 AND generation = $4 \
           AND sequence = $5 AND action = 'acked'",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(log_id)
    .bind(generation)
    .bind(sequence)
    .fetch_one(pool)
    .await
    .expect("independent transport observer must read ACK receipt count");
    SourceObservation {
        state,
        lease_remaining_ms,
        accepted_receipt_count,
        ack_receipt_count,
    }
}

async fn assert_source_route_has_one_open_delivery(
    pool: &PgPool,
    deployment_id: Uuid,
    tenant_id: Uuid,
    log_id: &str,
    generation: Uuid,
    expected_message_id: Uuid,
) {
    let rows = sqlx::query(
        "SELECT message_id FROM platform_transport.message_work \
         WHERE deployment_id = $1 AND tenant_id = $2 AND log_id = $3 AND generation = $4 \
           AND state IN ('pending', 'leased')",
    )
    .bind(deployment_id)
    .bind(tenant_id)
    .bind(log_id)
    .bind(generation)
    .fetch_all(pool)
    .await
    .expect("read pending deliveries for the isolated test route");
    assert_eq!(rows.len(), 1, "test route must contain one open delivery");
    let message_id: Uuid = rows[0]
        .try_get("message_id")
        .expect("read isolated route message identity");
    assert_eq!(message_id, expected_message_id);
}

fn task_event_wire(
    tenant_id: Uuid,
    project_id: Uuid,
    run_id: Uuid,
    event_id: Uuid,
    work_item_id: Uuid,
    actor_id: Uuid,
    correlation_id: Uuid,
) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "envelope_version": 1,
        "tenant_id": tenant_id,
        "project_id": project_id,
        "engineering_run_id": run_id,
        "event_id": event_id,
        "work_item_id": work_item_id,
        "aggregate_type": "lifecycle",
        "aggregate_version": 1,
        "event_type": "task.lifecycle.changed",
        "schema_version": 3,
        "actor_id": actor_id,
        "correlation_id": correlation_id,
        "occurred_at": Utc::now().to_rfc3339(),
        "payload": {
            "schema_version": 3,
            "aggregate": {"type": "lifecycle", "version": 1},
            "work_item_id": work_item_id.to_string(),
            "owner": {
                "tenant_id": tenant_id.to_string(),
                "project_id": project_id.to_string(),
                "engineering_run_id": run_id.to_string()
            },
            "lifecycle": {"status": "in_progress", "review_state": "none"},
            "change": {
                "event_type": "task.lifecycle.changed",
                "from_status": "claimed",
                "to_status": "in_progress"
            }
        }
    }))
    .expect("serialize bounded Task event wire")
}

async fn connect_pool(database_url: &str, max_connections: u32) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .expect("connect to a dedicated disposable PostgreSQL test database")
}

async fn require_database_name(pool: &PgPool, expected: &str) {
    let actual: String = sqlx::query_scalar("SELECT current_database()::TEXT")
        .fetch_one(pool)
        .await
        .expect("read test database identity");
    assert_eq!(
        actual, expected,
        "refusing an unexpected PostgreSQL database"
    );
}

fn require_isolated_database_host(database_url: &str) {
    let options = database_url
        .parse::<PgConnectOptions>()
        .expect("parse disposable PostgreSQL connection options");
    let host = options.get_host();
    let isolated = is_allowed_test_host(host);
    assert!(
        isolated,
        "acceptance databases must use loopback, RFC1918, or IPv6 ULA host addresses"
    );
}

fn is_allowed_test_host(host: &str) -> bool {
    host == "localhost"
        || host.parse::<IpAddr>().is_ok_and(|address| match address {
            IpAddr::V4(address) => address.is_loopback() || address.is_private(),
            IpAddr::V6(address) => address.is_loopback() || address.is_unique_local(),
        })
}

fn required_env(name: &str) -> String {
    env::var(name)
        .unwrap_or_else(|_| panic!("required test environment variable {name} is missing"))
}

fn required_uuid(name: &str) -> Uuid {
    Uuid::parse_str(&required_env(name))
        .unwrap_or_else(|_| panic!("test environment variable {name} must contain a UUID"))
}

fn required_u64(name: &str) -> u64 {
    required_env(name)
        .parse()
        .unwrap_or_else(|_| panic!("test environment variable {name} must contain an integer"))
}

fn required_sha256(name: &str) -> [u8; 32] {
    let bytes = hex::decode(required_env(name))
        .unwrap_or_else(|_| panic!("test environment variable {name} must be a hex SHA-256"));
    bytes
        .try_into()
        .unwrap_or_else(|_| panic!("test environment variable {name} must be 32 bytes"))
}

fn required_path(name: &str) -> PathBuf {
    let path = PathBuf::from(required_env(name));
    assert!(
        Path::new(&path).is_file(),
        "required mTLS PEM file from {name} must exist"
    );
    path
}

#[test]
fn disposable_database_host_allows_loopback_and_multipass_private_addresses() {
    assert!(is_allowed_test_host("localhost"));
    assert!(is_allowed_test_host("127.0.0.1"));
    assert!(is_allowed_test_host("10.12.0.8"));
    assert!(is_allowed_test_host("172.16.4.2"));
    assert!(is_allowed_test_host("192.168.64.3"));
    assert!(is_allowed_test_host("fd42:abcd::10"));
    assert!(is_allowed_test_host("::1"));
    assert!(!is_allowed_test_host("8.8.8.8"));
    assert!(!is_allowed_test_host("example.test"));
}
