//! Disposable PostgreSQL verification for Infrastructure native delivery route pins, receipts, and quarantine.
/*
@cypher schema=1 source_sha256=233f45822dc1c65a9020f0de0a50111bb45b359229b0ae496608f407c302bd93
MERGE (self:File {path:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs"})
MERGE (test:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::postgres_native_store_route_receipt_rls_worm_replay_and_attempt_fence",kind:"test"})
MERGE (seed:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::seed_owner_outbox_event",kind:"function"})
MERGE (profile:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::route",kind:"function"})
MERGE (route_endpoint:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::route_for_endpoint",kind:"function"})
MERGE (receipt_builder:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::native_receipt",kind:"function"})
MERGE (set_scope:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::set_owner_scope",kind:"function"})
MERGE (set_due:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::make_retry_due",kind:"function"})
MERGE (expire:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::expire_lease",kind:"function"})
MERGE (bad_receipt:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::reject_mismatched_receipt",kind:"function"})
MERGE (assert_worm:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::assert_worm_and_truncate_guards",kind:"function"})
MERGE (assert_quarantine:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::assert_stale_native_route_quarantine",kind:"function"})
MERGE (assert_quarantine_rls:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::assert_quarantine_tenant_scope",kind:"function"})
MERGE (rollback:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::assert_atomic_receipt_rollback",kind:"function"})
MERGE (schema_assertions:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::assert_native_result_schema",kind:"function"})
MERGE (rls_assertions:Symbol {id:"crates/infrastructure/tests/infrastructure_native_result_delivery_pg.rs::assert_tenant_scope_hides_other_tenant",kind:"function"})
MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerOutboxDelivery",kind:"external_type"})
MERGE (route:Type {id:"infrastructure::NativeInfrastructureDeliveryRoute",kind:"external_type"})
MERGE (profile_revision:Type {id:"platform_messaging_runtime::transport_profile::TransportProfileRevision",kind:"external_type"})
MERGE (endpoint_resolver:Type {id:"platform_messaging_runtime::transport_endpoint::TrustedEndpointResolver",kind:"external_type"})
MERGE (claim:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.claim_native_due",kind:"method"})
MERGE (quarantine_page:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.native_route_pin_quarantine_page",kind:"method"})
MERGE (pin_wire:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs::PostgresInfrastructureOwnerOutboxDelivery.get_or_pin_native_wire",kind:"method"})
MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox",class:"Transaction"})
MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery",class:"Work"})
MERGE (route_pin:Table {id:"infrastructure.capacity_outbox_transport_pin",class:"Transaction"})
MERGE (receipt:Table {id:"infrastructure.capacity_native_append_receipt",class:"Transaction"})
MERGE (quarantine:Table {id:"infrastructure.capacity_native_route_pin_quarantine",class:"Transaction"})
MERGE (postgres:ExternalService {id:"disposable PostgreSQL SQLx test database"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(seed)
MERGE (self)-[:DEFINES]->(profile)
MERGE (self)-[:DEFINES]->(receipt_builder)
MERGE (self)-[:DEFINES]->(set_scope)
MERGE (self)-[:DEFINES]->(set_due)
MERGE (self)-[:DEFINES]->(expire)
MERGE (self)-[:DEFINES]->(bad_receipt)
MERGE (self)-[:DEFINES]->(assert_worm)
MERGE (self)-[:DEFINES]->(assert_quarantine)
MERGE (self)-[:DEFINES]->(assert_quarantine_rls)
MERGE (self)-[:DEFINES]->(rollback)
MERGE (self)-[:DEFINES]->(schema_assertions)
MERGE (self)-[:DEFINES]->(rls_assertions)
MERGE (test)-[:CALLS]->(seed)
MERGE (test)-[:CALLS]->(profile)
MERGE (test)-[:CALLS]->(route_endpoint)
MERGE (test)-[:CALLS]->(receipt_builder)
MERGE (test)-[:CALLS]->(schema_assertions)
MERGE (test)-[:CALLS]->(rls_assertions)
MERGE (test)-[:CALLS]->(set_due)
MERGE (test)-[:CALLS]->(expire)
MERGE (test)-[:CALLS]->(bad_receipt)
MERGE (test)-[:CALLS]->(assert_worm)
MERGE (test)-[:CALLS]->(assert_quarantine)
MERGE (test)-[:CALLS]->(rollback)
MERGE (test)-[:USES_TYPE]->(store)
MERGE (test)-[:USES_TYPE]->(route)
MERGE (test)-[:CALLS]->(claim)
MERGE (test)-[:CALLS]->(quarantine_page)
MERGE (test)-[:CALLS]->(pin_wire)
MERGE (test)-[:USES_TYPE]->(postgres)
MERGE (route_endpoint)-[:USES_TYPE]->(profile_revision)
MERGE (route_endpoint)-[:USES_TYPE]->(endpoint_resolver)
MERGE (seed)-[:WRITES]->(outbox)
MERGE (seed)-[:WRITES]->(delivery)
MERGE (set_due)-[:WRITES]->(delivery)
MERGE (expire)-[:WRITES]->(delivery)
MERGE (test)-[:TESTS]->(route_pin)
MERGE (test)-[:TESTS]->(receipt)
MERGE (test)-[:TESTS]->(delivery)
MERGE (test)-[:TESTS]->(quarantine)
MERGE (assert_quarantine)-[:TESTS]->(quarantine)
MERGE (assert_quarantine)-[:TESTS]->(route_pin)
MERGE (assert_quarantine)-[:TESTS]->(delivery)
MERGE (assert_quarantine)-[:CALLS]->(assert_quarantine_rls)
MERGE (assert_worm)-[:TESTS]->(route_pin)
MERGE (assert_worm)-[:TESTS]->(receipt)
MERGE (assert_worm)-[:TESTS]->(quarantine)
@endcypher
*/

use infrastructure::infrastructure_owner_outbox_delivery::{
    NativeInfrastructureDeliveryRoute, NativeRoutePinQuarantinePage, OutboxDeliveryFailureKind,
    PostgresInfrastructureOwnerOutboxDelivery,
};
use platform_messaging_runtime::{
    admission_delivery::AdmissionTransportProfile,
    native_append::NativeAppendReceipt,
    transport_endpoint::{TrustedEndpointConfig, TrustedEndpointResolver},
    transport_profile::{
        TransportMessageKind, TransportProfileDefinition, TransportProfileLifecycle,
        TransportProfileRevision,
    },
};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_INFRA_OWNER_TEST_DATABASE_URL";

#[tokio::test]
#[ignore = "requires scripts/automation/phase9f4d_run_owner_consumer.py --schema-only --native-result-tests"]
async fn postgres_native_store_route_receipt_rls_worm_replay_and_attempt_fence(
) -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var(DATABASE_URL_ENV)?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await?;
    let store = PostgresInfrastructureOwnerOutboxDelivery::new(pool.clone());

    assert_native_result_schema(&pool).await?;

    let tenant_a = Uuid::new_v4();
    let event_a = seed_owner_outbox_event(&pool, tenant_a).await?;
    let route_a = route(tenant_a);
    let first_claim = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &route_a)
        .await?
        .claims
        .pop()
        .expect("one native result event is claimed for the resolved tenant");
    assert_eq!(first_claim.event_id, event_a);
    let first_wire = store.get_or_pin_native_wire(&first_claim, &route_a).await?;
    store
        .record_failure(&first_claim, OutboxDeliveryFailureKind::Transient)
        .await?;
    let pinned_wire: Vec<u8> = sqlx::query_scalar(
        "SELECT wire_payload FROM infrastructure.capacity_outbox_transport_pin WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_a)
    .bind(event_a)
    .fetch_one(&pool)
    .await?;
    let pinned_digest: Vec<u8> = sqlx::query_scalar(
        "SELECT wire_sha256 FROM infrastructure.capacity_outbox_transport_pin WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_a)
    .bind(event_a)
    .fetch_one(&pool)
    .await?;
    let expected_digest: [u8; 32] = Sha256::digest(&pinned_wire).into();
    assert_eq!(pinned_digest.as_slice(), expected_digest.as_slice());
    assert_eq!(pinned_wire, first_wire.payload);

    let nats_claims = store.claim_due(tenant_a, Uuid::new_v4(), 1, 30).await?;
    assert!(
        nats_claims.is_empty(),
        "a native-pinned row cannot fall back to NATS"
    );
    make_retry_due(&pool, tenant_a, event_a).await?;

    let second_claim = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &route_a)
        .await?
        .claims
        .pop()
        .expect("retry is claimable after its due time");
    assert_eq!(second_claim.attempt_count, first_claim.attempt_count + 1);
    let second_wire = store
        .get_or_pin_native_wire(&second_claim, &route_a)
        .await?;
    assert_eq!(
        first_wire.payload, second_wire.payload,
        "retry must reuse byte-identical pinned wire"
    );
    let replay_receipt = native_receipt(&route_a, &second_wire.sha256, true, 2);
    store
        .mark_native_delivered(&second_claim, &route_a, &replay_receipt)
        .await?;
    let receipt_attempt: i32 = sqlx::query_scalar(
        "SELECT attempt_count FROM infrastructure.capacity_native_append_receipt WHERE tenant_id = $1 AND event_id = $2 AND exact_replay",
    )
    .bind(tenant_a)
    .bind(event_a)
    .fetch_one(&pool)
    .await?;
    assert_eq!(receipt_attempt, 2);

    let tenant_b = Uuid::new_v4();
    let event_b = seed_owner_outbox_event(&pool, tenant_b).await?;
    let route_b = route(tenant_b);
    let claim_b = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &route_b)
        .await?;
    let claim_b = claim_b
        .claims
        .into_iter()
        .next()
        .expect("tenant B result uses its separately resolved route");
    assert_eq!(claim_b.event_id, event_b);
    let wire_b = store.get_or_pin_native_wire(&claim_b, &route_b).await?;
    store
        .mark_native_delivered(
            &claim_b,
            &route_b,
            &native_receipt(&route_b, &wire_b.sha256, false, 1),
        )
        .await?;
    assert_tenant_scope_hides_other_tenant(&pool, tenant_a, tenant_b, event_b).await?;

    let tenant_fenced = Uuid::new_v4();
    let event_fenced = seed_owner_outbox_event(&pool, tenant_fenced).await?;
    let route_fenced = route(tenant_fenced);
    let old_owner = Uuid::new_v4();
    let stale_claim = store
        .claim_native_due(old_owner, 1, 120, &route_fenced)
        .await?
        .claims
        .pop()
        .expect("one native event is claimed");
    let old_wire = store
        .get_or_pin_native_wire(&stale_claim, &route_fenced)
        .await?;
    expire_lease(&pool, tenant_fenced, event_fenced).await?;
    let new_claim = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &route_fenced)
        .await?
        .claims
        .pop()
        .expect("expired native lease is reclaimed");
    assert_eq!(new_claim.attempt_count, stale_claim.attempt_count + 1);
    let new_wire = store
        .get_or_pin_native_wire(&new_claim, &route_fenced)
        .await?;
    assert_eq!(old_wire.payload, new_wire.payload);
    let conflicting_route =
        route_for_endpoint(tenant_fenced, Uuid::new_v4(), "native-endpoint-conflict");
    assert!(matches!(
        store
            .get_or_pin_native_wire(&new_claim, &conflicting_route)
            .await,
        Err(infrastructure::infrastructure_owner_outbox_delivery::InfrastructureOwnerOutboxDeliveryError::RouteConflict)
    ));
    let valid_receipt = native_receipt(&route_fenced, &new_wire.sha256, false, 3);
    assert!(store
        .mark_native_delivered(&stale_claim, &route_fenced, &valid_receipt)
        .await
        .is_err());
    let stale_receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_append_receipt WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_fenced)
    .bind(event_fenced)
    .fetch_one(&pool)
    .await?;
    assert_eq!(stale_receipt_count, 0, "a stale receipt must roll back");
    reject_mismatched_receipt(&pool, &new_claim, &route_fenced, &new_wire.sha256, true).await?;
    reject_mismatched_receipt(&pool, &new_claim, &route_fenced, &new_wire.sha256, false).await?;
    assert_atomic_receipt_rollback(&pool, &store, &new_claim, &route_fenced, &valid_receipt)
        .await?;
    store
        .mark_native_delivered(&new_claim, &route_fenced, &valid_receipt)
        .await?;

    let quarantined_event = assert_stale_native_route_quarantine(&pool, &store, tenant_a).await?;
    assert_worm_and_truncate_guards(&pool, tenant_a, event_a, quarantined_event).await?;
    pool.close().await;
    Ok(())
}

fn route(tenant_id: Uuid) -> NativeInfrastructureDeliveryRoute {
    route_for_endpoint(tenant_id, Uuid::new_v4(), "native-endpoint")
}

async fn assert_stale_native_route_quarantine(
    pool: &PgPool,
    store: &PostgresInfrastructureOwnerOutboxDelivery,
    tenant_id: Uuid,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    let empty_route = route(Uuid::new_v4());
    let empty_pass = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &empty_route)
        .await?;
    assert!(empty_pass.claims.is_empty());
    assert_eq!(empty_pass.route_pin_quarantine.total_blocked, 0);
    assert_eq!(empty_pass.route_pin_quarantine.newly_quarantined, 0);
    assert!(empty_pass.route_pin_quarantine.entries.is_empty());

    let event_id = seed_owner_outbox_event(pool, tenant_id).await?;
    let pinned_route = route(tenant_id);
    let first_claim = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &pinned_route)
        .await?
        .claims
        .pop()
        .expect("new event is claimable before a route mismatch exists");
    let original_wire = store
        .get_or_pin_native_wire(&first_claim, &pinned_route)
        .await?;
    store
        .record_failure(&first_claim, OutboxDeliveryFailureKind::Transient)
        .await?;
    make_retry_due(pool, tenant_id, event_id).await?;

    let changed_route = route_for_endpoint(tenant_id, Uuid::new_v4(), "native-endpoint-conflict");
    let blocked_pass = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &changed_route)
        .await?;
    assert!(
        blocked_pass.claims.is_empty(),
        "mismatched pin must not be claimed"
    );
    assert_eq!(blocked_pass.route_pin_quarantine.total_blocked, 1);
    assert_eq!(blocked_pass.route_pin_quarantine.newly_quarantined, 1);
    assert_eq!(blocked_pass.route_pin_quarantine.entries.len(), 1);
    let diagnostic = &blocked_pass.route_pin_quarantine.entries[0];
    assert_eq!(diagnostic.event_id, event_id);
    assert_eq!(diagnostic.observed_profile_id, changed_route.profile_id());
    assert!(diagnostic
        .mismatch_fields
        .iter()
        .any(|field| field == "profile_id"));
    assert!(diagnostic
        .mismatch_fields
        .iter()
        .any(|field| field == "endpoint_identity"));
    assert!(diagnostic
        .mismatch_fields
        .iter()
        .any(|field| field == "profile_digest"));

    let original_pin: (
        Uuid,
        i64,
        String,
        String,
        Vec<u8>,
        String,
        Uuid,
        Uuid,
        String,
        Vec<u8>,
        Vec<u8>,
    ) = sqlx::query_as(
        "SELECT profile_id, profile_revision, provider_id, provider_version, profile_digest, \
                endpoint_identity, sender_service_id, recipient_service_id, message_kind, \
                wire_sha256, wire_payload \
         FROM infrastructure.capacity_outbox_transport_pin WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .fetch_one(pool)
    .await?;
    assert_eq!(original_pin.0, pinned_route.profile_id());
    assert_eq!(original_pin.1, pinned_route.profile_revision());
    assert_eq!(original_pin.2, pinned_route.profile().provider_id);
    assert_eq!(original_pin.3, pinned_route.profile().provider_version);
    assert_eq!(
        original_pin.4,
        pinned_route.profile().profile_digest.to_vec()
    );
    assert_eq!(original_pin.5, pinned_route.profile().endpoint_identity);
    assert_eq!(original_pin.6, pinned_route.sender_service_id());
    assert_eq!(original_pin.7, pinned_route.recipient_service_id());
    assert_eq!(original_pin.8, "result");
    assert_eq!(original_pin.9.as_slice(), original_wire.sha256.as_slice());
    assert_eq!(original_pin.10, original_wire.payload);
    let delivery_state: String = sqlx::query_scalar(
        "SELECT delivery_state FROM infrastructure.capacity_outbox_delivery \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .fetch_one(pool)
    .await?;
    assert_eq!(
        delivery_state, "dead",
        "quarantine must terminalize blocked pending/retry work"
    );
    let receipt_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_append_receipt \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .fetch_one(pool)
    .await?;
    assert_eq!(receipt_count, 0);
    assert!(store
        .claim_due(tenant_id, Uuid::new_v4(), 1, 30)
        .await?
        .is_empty());

    let repeated_pass = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &changed_route)
        .await?;
    assert!(repeated_pass.claims.is_empty());
    assert_eq!(repeated_pass.route_pin_quarantine.total_blocked, 1);
    assert_eq!(repeated_pass.route_pin_quarantine.newly_quarantined, 0);
    let read_page = store
        .native_route_pin_quarantine_page(&changed_route, None, 1)
        .await?;
    assert_eq!(read_page.total_blocked, 1);
    assert_eq!(read_page.entries[0].event_id, event_id);
    assert!(read_page.next_cursor.is_none());

    let active_event = seed_owner_outbox_event(pool, tenant_id).await?;
    let active_claim = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &pinned_route)
        .await?
        .claims
        .into_iter()
        .find(|claim| claim.event_id == active_event)
        .expect("second native event is leased on the pinned route");
    let active_wire = store
        .get_or_pin_native_wire(&active_claim, &pinned_route)
        .await?;
    let during_active_lease = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &changed_route)
        .await?;
    assert!(during_active_lease.claims.is_empty());
    assert_eq!(during_active_lease.route_pin_quarantine.total_blocked, 1);
    let active_quarantine_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_route_pin_quarantine \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(active_event)
    .fetch_one(pool)
    .await?;
    assert_eq!(
        active_quarantine_count, 0,
        "active leases must remain untouched"
    );
    store
        .mark_native_delivered(
            &active_claim,
            &pinned_route,
            &native_receipt(&pinned_route, &active_wire.sha256, false, 4),
        )
        .await?;
    let after_receipt = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &changed_route)
        .await?;
    assert_eq!(after_receipt.route_pin_quarantine.total_blocked, 1);
    let delivered_quarantine_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_route_pin_quarantine \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(active_event)
    .fetch_one(pool)
    .await?;
    assert_eq!(
        delivered_quarantine_count, 0,
        "a delivered receipt must not be quarantined"
    );

    let other_tenant = Uuid::new_v4();
    let other_event = seed_owner_outbox_event(pool, other_tenant).await?;
    let other_route = route(other_tenant);
    let other_claim = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &other_route)
        .await?
        .claims
        .pop()
        .expect("other tenant's event is claimable on its own route");
    store
        .get_or_pin_native_wire(&other_claim, &other_route)
        .await?;
    store
        .record_failure(&other_claim, OutboxDeliveryFailureKind::Transient)
        .await?;
    make_retry_due(pool, other_tenant, other_event).await?;
    let other_tenant_state: String = sqlx::query_scalar(
        "SELECT delivery_state FROM infrastructure.capacity_outbox_delivery \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(other_tenant)
    .bind(other_event)
    .fetch_one(pool)
    .await?;
    let other_tenant_quarantine_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_route_pin_quarantine \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(other_tenant)
    .bind(other_event)
    .fetch_one(pool)
    .await?;
    assert_eq!(other_tenant_state, "retry");
    assert_eq!(other_tenant_quarantine_count, 0);
    let other_changed_route = route_for_endpoint(
        other_tenant,
        Uuid::new_v4(),
        "other-native-endpoint-conflict",
    );
    let other_pass = store
        .claim_native_due(Uuid::new_v4(), 1, 120, &other_changed_route)
        .await?;
    assert_eq!(other_pass.route_pin_quarantine.total_blocked, 1);
    assert_eq!(
        other_pass.route_pin_quarantine.entries[0].event_id,
        other_event
    );

    let tenant_a_page = assert_quarantine_tenant_scope(store, &changed_route, other_event).await?;
    assert_eq!(tenant_a_page.total_blocked, 1);
    assert!(tenant_a_page
        .entries
        .iter()
        .any(|entry| entry.event_id == event_id));
    assert!(!tenant_a_page
        .entries
        .iter()
        .any(|entry| entry.event_id == other_event));

    Ok(event_id)
}

async fn assert_quarantine_tenant_scope(
    store: &PostgresInfrastructureOwnerOutboxDelivery,
    tenant_a_route: &NativeInfrastructureDeliveryRoute,
    tenant_b_event: Uuid,
) -> Result<NativeRoutePinQuarantinePage, Box<dyn std::error::Error>> {
    let page = store
        .native_route_pin_quarantine_page(tenant_a_route, None, 32)
        .await?;
    assert!(!page
        .entries
        .iter()
        .any(|entry| entry.event_id == tenant_b_event));
    Ok(page)
}

fn route_for_endpoint(
    tenant_id: Uuid,
    profile_id: Uuid,
    endpoint_identity: &str,
) -> NativeInfrastructureDeliveryRoute {
    let deployment_id = Uuid::from_u128(0xD20);
    let sender_service_id = Uuid::from_u128(2);
    let recipient_service_id = Uuid::from_u128(3);
    let definition = TransportProfileDefinition::new(
        "star-rust-log".to_owned(),
        "1.0.0".to_owned(),
        endpoint_identity.to_owned(),
        "provider-log-generation-v1".to_owned(),
        vec![sender_service_id],
        vec![recipient_service_id],
        vec![TransportMessageKind::Result],
        None,
    )
    .expect("valid native profile definition");
    let revision = TransportProfileRevision::new(
        deployment_id,
        tenant_id,
        profile_id,
        3,
        TransportProfileLifecycle::Active,
        definition,
    )
    .expect("valid immutable profile revision");
    let trusted_endpoint = TrustedEndpointConfig::new(
        endpoint_identity.to_owned(),
        "https://transport.internal.example:7443/grpc/v1".to_owned(),
        "transport.internal.example".to_owned(),
    )
    .expect("valid trusted endpoint configuration");
    let endpoint = TrustedEndpointResolver::new(deployment_id, vec![trusted_endpoint])
        .expect("valid trusted endpoint resolver")
        .resolve(&revision)
        .expect("profile resolves to endpoint");
    let profile = AdmissionTransportProfile::new(
        "star-rust-log".to_owned(),
        "1.0.0".to_owned(),
        endpoint.profile_digest(),
        endpoint.endpoint_identity().to_owned(),
    )
    .expect("valid transport profile snapshot");
    NativeInfrastructureDeliveryRoute::from_resolved_profile_for_test(
        &endpoint,
        profile,
        sender_service_id,
        recipient_service_id,
    )
    .expect("route binds the trusted tenant and profile")
}

fn native_receipt(
    route: &NativeInfrastructureDeliveryRoute,
    wire_sha256: &[u8; 32],
    exact_replay: bool,
    sequence: i64,
) -> NativeAppendReceipt {
    NativeAppendReceipt {
        profile: route.profile().clone(),
        log_id: "run-results-v1".to_owned(),
        log_generation: Uuid::from_u128(0xD20),
        sequence,
        exact_replay,
        wire_sha256: *wire_sha256,
    }
}

async fn seed_owner_outbox_event(pool: &PgPool, tenant_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO infrastructure.capacity_owner_outbox \
         (event_id, tenant_id, source_event_id, occurrence_id, event_type, schema_version, \
          idempotency_key, correlation_id, payload) \
         VALUES ($1, $2, $3, $4, 'capacity_rejected', 2, $5, $6, '{\"reason\":\"native-pg-test\"}'::jsonb)",
    )
    .bind(event_id)
    .bind(tenant_id)
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .bind(format!("native-pg-test:{event_id}"))
    .bind(Uuid::new_v4())
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO infrastructure.capacity_outbox_delivery (tenant_id, event_id) VALUES ($1, $2)",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(event_id)
}

async fn set_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL ROLE star_infrastructure_owner")
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn make_retry_due(pool: &PgPool, tenant_id: Uuid, event_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, tenant_id).await?;
    sqlx::query(
        "UPDATE infrastructure.capacity_outbox_delivery \
         SET next_attempt_at = clock_timestamp() - interval '1 second' \
         WHERE tenant_id = $1 AND event_id = $2 AND delivery_state = 'retry'",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn expire_lease(pool: &PgPool, tenant_id: Uuid, event_id: Uuid) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, tenant_id).await?;
    sqlx::query(
        "UPDATE infrastructure.capacity_outbox_delivery \
         SET lease_expires_at = clock_timestamp() - interval '1 second' \
         WHERE tenant_id = $1 AND event_id = $2 AND delivery_state = 'leased'",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn reject_mismatched_receipt(
    pool: &PgPool,
    claim: &infrastructure::infrastructure_owner_outbox_delivery::InfrastructureOwnerOutboxDeliveryClaim,
    route: &NativeInfrastructureDeliveryRoute,
    digest: &[u8; 32],
    wrong_provider: bool,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, claim.tenant_id).await?;
    let provider_id = if wrong_provider {
        "other-native-provider"
    } else {
        route.profile().provider_id.as_str()
    };
    let wire_digest = if wrong_provider {
        digest.to_vec()
    } else {
        vec![0; 32]
    };
    let insert = sqlx::query(
        "INSERT INTO infrastructure.capacity_native_append_receipt \
         (tenant_id, event_id, profile_id, profile_revision, provider_id, provider_version, \
          profile_digest, endpoint_identity, sender_service_id, recipient_service_id, message_kind, \
          message_id, log_id, log_generation, sequence, exact_replay, wire_sha256, lease_owner_id, attempt_count) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'result',$2,'run-results-v1',$11,4,false,$12,$13,$14)",
    )
    .bind(claim.tenant_id)
    .bind(claim.event_id)
    .bind(route.profile_id())
    .bind(route.profile_revision())
    .bind(provider_id)
    .bind(&route.profile().provider_version)
    .bind(route.profile().profile_digest.to_vec())
    .bind(&route.profile().endpoint_identity)
    .bind(route.sender_service_id())
    .bind(route.recipient_service_id())
    .bind(Uuid::from_u128(0xD20))
    .bind(wire_digest)
    .bind(claim.lease_owner_id)
    .bind(claim.attempt_count)
    .execute(&mut *tx)
    .await;
    tx.rollback().await?;
    assert!(
        insert.is_err(),
        "database must reject a provider or wire-digest mismatch"
    );
    Ok(())
}

async fn assert_native_result_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT c.relrowsecurity, c.relforcerowsecurity, \
                has_table_privilege('star_infrastructure_owner', c.oid, 'SELECT') AS can_select, \
                has_table_privilege('star_infrastructure_owner', c.oid, 'INSERT') AS can_insert, \
                has_table_privilege('star_infrastructure_owner', c.oid, 'UPDATE') AS can_update, \
                has_table_privilege('star_infrastructure_owner', c.oid, 'DELETE') AS can_delete, \
                has_table_privilege('star_infrastructure_owner', c.oid, 'TRUNCATE') AS can_truncate \
         FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'infrastructure' AND c.relname IN \
           ('capacity_outbox_transport_pin','capacity_native_append_receipt', \
            'capacity_native_route_pin_quarantine') \
         ORDER BY c.relname",
    )
    .fetch_all(pool)
    .await?;
    assert_eq!(rows.len(), 3);
    for row in rows {
        assert!(row.try_get::<bool, _>("relrowsecurity")?);
        assert!(row.try_get::<bool, _>("relforcerowsecurity")?);
        assert!(row.try_get::<bool, _>("can_select")?);
        assert!(row.try_get::<bool, _>("can_insert")?);
        assert!(!row.try_get::<bool, _>("can_update")?);
        assert!(!row.try_get::<bool, _>("can_delete")?);
        assert!(!row.try_get::<bool, _>("can_truncate")?);
    }
    let role_is_restricted: bool = sqlx::query_scalar(
        "SELECT NOT rolsuper AND NOT rolbypassrls AND NOT rolcanlogin \
         FROM pg_roles WHERE rolname = 'star_infrastructure_owner'",
    )
    .fetch_one(pool)
    .await?;
    assert!(role_is_restricted);
    let tenant_policies: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_policies WHERE schemaname = 'infrastructure' \
         AND tablename IN ('capacity_outbox_transport_pin','capacity_native_append_receipt', \
                           'capacity_native_route_pin_quarantine') \
         AND qual LIKE '%app.tenant_id%' AND with_check LIKE '%app.tenant_id%'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(
        tenant_policies, 3,
        "all route-pin facts require tenant-scoped USING and WITH CHECK policies"
    );
    let guard_triggers: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'infrastructure' AND NOT t.tgisinternal \
           AND ((c.relname = 'capacity_outbox_transport_pin' AND t.tgname IN \
                 ('capacity_outbox_transport_pin_insert_guard','capacity_outbox_transport_pin_worm')) \
              OR (c.relname = 'capacity_native_append_receipt' AND t.tgname IN \
                  ('capacity_native_append_receipt_insert_guard','capacity_native_append_receipt_worm')) \
              OR (c.relname = 'capacity_native_route_pin_quarantine' AND t.tgname IN \
                  ('capacity_native_route_pin_quarantine_insert_guard','capacity_native_route_pin_quarantine_worm'))) ",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(
        guard_triggers, 6,
        "route pins, receipts, and quarantine require insert fencing and WORM guards"
    );
    let truncate_guards: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'infrastructure' AND NOT t.tgisinternal \
            AND t.tgname IN ('capacity_outbox_transport_pin_worm','capacity_native_append_receipt_worm', \
                             'capacity_native_route_pin_quarantine_worm') \
           AND (t.tgtype & 32) = 32",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(
        truncate_guards, 3,
        "each immutable result table requires a statement-level TRUNCATE trigger"
    );
    Ok(())
}

async fn assert_tenant_scope_hides_other_tenant(
    pool: &PgPool,
    tenant_a: Uuid,
    tenant_b: Uuid,
    event_b: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_owner_scope(&mut tx, tenant_a).await?;
    let visible_b_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_outbox_transport_pin \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_b)
    .bind(event_b)
    .fetch_one(&mut *tx)
    .await?;
    let own_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_outbox_transport_pin WHERE tenant_id = $1",
    )
    .bind(tenant_a)
    .fetch_one(&mut *tx)
    .await?;
    let visible_b_receipts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_append_receipt \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_b)
    .bind(event_b)
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(visible_b_rows, 0);
    assert_eq!(visible_b_receipts, 0);
    assert!(own_rows > 0);
    tx.rollback().await
}

async fn assert_atomic_receipt_rollback(
    pool: &PgPool,
    store: &PostgresInfrastructureOwnerOutboxDelivery,
    claim: &infrastructure::infrastructure_owner_outbox_delivery::InfrastructureOwnerOutboxDeliveryClaim,
    route: &NativeInfrastructureDeliveryRoute,
    receipt: &NativeAppendReceipt,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE OR REPLACE FUNCTION infrastructure.test_native_delivery_finalize_fault() \
         RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN RAISE EXCEPTION 'disposable native receipt rollback fault'; END $$",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TRIGGER test_native_delivery_finalize_fault \
         BEFORE UPDATE OF delivery_state ON infrastructure.capacity_outbox_delivery \
         FOR EACH ROW WHEN (NEW.delivery_state = 'delivered') \
         EXECUTE FUNCTION infrastructure.test_native_delivery_finalize_fault()",
    )
    .execute(pool)
    .await?;
    let failed = store.mark_native_delivered(claim, route, receipt).await;
    sqlx::query("DROP TRIGGER test_native_delivery_finalize_fault ON infrastructure.capacity_outbox_delivery")
        .execute(pool)
        .await?;
    sqlx::query("DROP FUNCTION infrastructure.test_native_delivery_finalize_fault()")
        .execute(pool)
        .await?;
    assert!(
        failed.is_err(),
        "injected finalize fault must abort the owner transaction"
    );
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM infrastructure.capacity_native_append_receipt WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(claim.tenant_id)
    .bind(claim.event_id)
    .fetch_one(pool)
    .await?;
    let delivery_state: String = sqlx::query_scalar(
        "SELECT delivery_state FROM infrastructure.capacity_outbox_delivery WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(claim.tenant_id)
    .bind(claim.event_id)
    .fetch_one(pool)
    .await?;
    assert_eq!(
        receipts, 0,
        "the receipt insert must roll back with delivery finalization"
    );
    assert_eq!(
        delivery_state, "leased",
        "failed finalization must keep the delivery retryable"
    );
    Ok(())
}

async fn assert_worm_and_truncate_guards(
    pool: &PgPool,
    tenant_id: Uuid,
    event_id: Uuid,
    quarantine_event_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut owner_update = pool.begin().await?;
    set_owner_scope(&mut owner_update, tenant_id).await?;
    let denied_update = sqlx::query(
        "UPDATE infrastructure.capacity_outbox_transport_pin SET endpoint_identity = endpoint_identity \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(&mut *owner_update)
    .await;
    owner_update.rollback().await?;
    assert!(
        denied_update.is_err(),
        "owner role must not be granted UPDATE"
    );

    let mut owner_delete = pool.begin().await?;
    set_owner_scope(&mut owner_delete, tenant_id).await?;
    let denied_delete = sqlx::query(
        "DELETE FROM infrastructure.capacity_native_append_receipt WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(event_id)
    .execute(&mut *owner_delete)
    .await;
    owner_delete.rollback().await?;
    assert!(
        denied_delete.is_err(),
        "owner role must not be granted DELETE"
    );

    let mut owner_quarantine_update = pool.begin().await?;
    set_owner_scope(&mut owner_quarantine_update, tenant_id).await?;
    let denied_quarantine_update = sqlx::query(
        "UPDATE infrastructure.capacity_native_route_pin_quarantine \
         SET detected_at = detected_at WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(quarantine_event_id)
    .execute(&mut *owner_quarantine_update)
    .await;
    owner_quarantine_update.rollback().await?;
    assert!(denied_quarantine_update.is_err());

    let mut owner_quarantine_delete = pool.begin().await?;
    set_owner_scope(&mut owner_quarantine_delete, tenant_id).await?;
    let denied_quarantine_delete = sqlx::query(
        "DELETE FROM infrastructure.capacity_native_route_pin_quarantine \
         WHERE tenant_id = $1 AND event_id = $2",
    )
    .bind(tenant_id)
    .bind(quarantine_event_id)
    .execute(&mut *owner_quarantine_delete)
    .await;
    owner_quarantine_delete.rollback().await?;
    assert!(denied_quarantine_delete.is_err());

    for (statement, expected_table) in [
        (
            "UPDATE infrastructure.capacity_outbox_transport_pin SET pinned_at = pinned_at WHERE event_id = $1",
            "capacity_outbox_transport_pin",
        ),
        (
            "DELETE FROM infrastructure.capacity_outbox_transport_pin WHERE event_id = $1",
            "capacity_outbox_transport_pin",
        ),
        (
            "TRUNCATE infrastructure.capacity_outbox_transport_pin CASCADE",
            "capacity_outbox_transport_pin",
        ),
        (
            "UPDATE infrastructure.capacity_native_append_receipt SET appended_at = appended_at WHERE event_id = $1",
            "capacity_native_append_receipt",
        ),
        (
            "DELETE FROM infrastructure.capacity_native_append_receipt WHERE event_id = $1",
            "capacity_native_append_receipt",
        ),
        (
            "TRUNCATE infrastructure.capacity_native_append_receipt CASCADE",
            "capacity_native_append_receipt",
        ),
        (
            "UPDATE infrastructure.capacity_native_route_pin_quarantine SET detected_at = detected_at WHERE event_id = $1",
            "capacity_native_route_pin_quarantine",
        ),
        (
            "DELETE FROM infrastructure.capacity_native_route_pin_quarantine WHERE event_id = $1",
            "capacity_native_route_pin_quarantine",
        ),
        (
            "TRUNCATE infrastructure.capacity_native_route_pin_quarantine CASCADE",
            "capacity_native_route_pin_quarantine",
        ),
    ] {
        let mut tx = pool.begin().await?;
        let result = if statement.contains("$1") {
            sqlx::query(statement)
                .bind(event_id)
                .execute(&mut *tx)
                .await
        } else {
            sqlx::query(statement).execute(&mut *tx).await
        };
        tx.rollback().await?;
        let error = result.expect_err("WORM trigger must reject the mutation");
        let message = error
            .as_database_error()
            .map(|database_error| database_error.message())
            .unwrap_or_default();
        if statement.starts_with("TRUNCATE") {
            assert!(
                message.starts_with("Infrastructure capacity fact ")
                    && message.ends_with(" is append-only"),
                "an append-only trigger must reject `{statement}`; got `{message}`"
            );
        } else {
            assert_eq!(
                message,
                format!("Infrastructure capacity fact {expected_table} is append-only"),
                "the expected table-specific WORM trigger must reject `{statement}`"
            );
        }
    }
    Ok(())
}
