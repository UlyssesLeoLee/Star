// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=72724a6b56bfc2049f904e46b76f40f289c4a8f736f56b6c657cf3aa0f560252
MERGE (self:File {path:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs"})
MERGE (test:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::pg_source_position_is_unique_across_inbox_and_quarantine",kind:"test"})
MERGE (connect:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::connect_disposable_pool",kind:"function"})
MERGE (identity:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::require_database_identity",kind:"function"})
MERGE (same_db:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::require_same_database",kind:"function"})
MERGE (schema:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::require_shared_claim_contract",kind:"function"})
MERGE (assert_counts:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::assert_terminal_counts",kind:"function"})
MERGE (terminal_counts:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::terminal_counts",kind:"function"})
MERGE (seed_history:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::seed_cursorless_quarantine_history",kind:"function"})
MERGE (seed_known_claim:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::seed_legacy_nonnull_quarantine_claim",kind:"function"})
MERGE (seed_known_receipt:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::seed_legacy_nonnull_quarantine_receipt",kind:"function"})
MERGE (mode_conflict:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::assert_pg_mode_conflict",kind:"function"})
MERGE (receiver_helper:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::receive_one",kind:"function"})
MERGE (route_helper:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::route_pin",kind:"function"})
MERGE (origin_helper:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::source_origin",kind:"function"})
MERGE (origin_cursor_helper:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::source_origin_with_cursor",kind:"function"})
MERGE (wire_helper:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::lifecycle_wire",kind:"function"})
MERGE (delivery:Type {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::PgDelivery",kind:"struct"})
MERGE (transport:Type {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::OneShotTransport",kind:"struct"})
MERGE (authorization:Type {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::AllowAuthorization",kind:"struct"})
MERGE (counters:Type {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::DispositionCounts",kind:"struct"})
MERGE (delivery_impl:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::PgDelivery.acknowledge",kind:"method"})
MERGE (terminate_impl:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::PgDelivery.terminate_permanently",kind:"method"})
MERGE (pull_impl:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::OneShotTransport.pull_one",kind:"method"})
MERGE (verify_impl:Symbol {id:"crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs::AllowAuthorization.verify_origin",kind:"method"})
MERGE (receiver:Type {id:"bi_owner_runtime::TaskEventReceiver",kind:"struct"})
MERGE (inbox_store:Type {id:"bi_owner_runtime::TaskEventInboxStore",kind:"struct"})
MERGE (quarantine_store:Type {id:"bi_owner_runtime::TaskEventQuarantineStore",kind:"struct"})
MERGE (claim_table:Table {id:"bi.task_event_source_position_claim",class:"Transaction"})
MERGE (origin_table:Table {id:"bi.task_event_source_origin",class:"Transaction"})
MERGE (quarantine_table:Table {id:"bi.task_event_quarantine",class:"Transaction"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(connect)
MERGE (self)-[:DEFINES]->(identity)
MERGE (self)-[:DEFINES]->(same_db)
MERGE (self)-[:DEFINES]->(schema)
MERGE (self)-[:DEFINES]->(assert_counts)
MERGE (self)-[:DEFINES]->(terminal_counts)
MERGE (self)-[:DEFINES]->(seed_history)
MERGE (self)-[:DEFINES]->(seed_known_claim)
MERGE (self)-[:DEFINES]->(seed_known_receipt)
MERGE (self)-[:DEFINES]->(mode_conflict)
MERGE (self)-[:DEFINES]->(receiver_helper)
MERGE (self)-[:DEFINES]->(route_helper)
MERGE (self)-[:DEFINES]->(origin_helper)
MERGE (self)-[:DEFINES]->(origin_cursor_helper)
MERGE (self)-[:DEFINES]->(wire_helper)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(authorization)
MERGE (self)-[:DEFINES]->(counters)
MERGE (self)-[:IMPORTS]->(receiver)
MERGE (self)-[:IMPORTS]->(inbox_store)
MERGE (self)-[:IMPORTS]->(quarantine_store)
MERGE (self)-[:TESTS]->(claim_table)
MERGE (self)-[:TESTS]->(origin_table)
MERGE (self)-[:TESTS]->(quarantine_table)
MERGE (test)-[:CALLS]->(connect)
MERGE (test)-[:CALLS]->(identity)
MERGE (test)-[:CALLS]->(same_db)
MERGE (test)-[:CALLS]->(schema)
MERGE (test)-[:CALLS]->(assert_counts)
MERGE (test)-[:CALLS]->(receiver_helper)
MERGE (test)-[:CALLS]->(route_helper)
MERGE (test)-[:CALLS]->(origin_helper)
MERGE (test)-[:CALLS]->(origin_cursor_helper)
MERGE (test)-[:CALLS]->(seed_history)
MERGE (test)-[:CALLS]->(seed_known_claim)
MERGE (test)-[:CALLS]->(seed_known_receipt)
MERGE (test)-[:CALLS]->(mode_conflict)
MERGE (test)-[:CALLS]->(wire_helper)
MERGE (assert_counts)-[:CALLS]->(terminal_counts)
MERGE (terminal_counts)-[:READS]->(claim_table)
MERGE (terminal_counts)-[:READS]->(origin_table)
MERGE (terminal_counts)-[:READS]->(quarantine_table)
MERGE (seed_history)-[:WRITES]->(quarantine_table)
MERGE (seed_known_claim)-[:WRITES]->(claim_table)
MERGE (seed_known_receipt)-[:WRITES]->(quarantine_table)
MERGE (seed_known_receipt)-[:WRITES]->(claim_table)
MERGE (origin_helper)-[:CALLS]->(origin_cursor_helper)
MERGE (test)-[:TESTS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.receive_one",kind:"method"})
MERGE (receiver_helper)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.receive_one",kind:"method"})
MERGE (transport)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveTransport",kind:"trait"})
MERGE (delivery)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveDelivery",kind:"trait"})
MERGE (authorization)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveAuthorization",kind:"trait"})
MERGE (self)-[:DEFINES]->(delivery_impl)
MERGE (self)-[:DEFINES]->(terminate_impl)
MERGE (self)-[:DEFINES]->(pull_impl)
MERGE (self)-[:DEFINES]->(verify_impl)
MERGE (test)-[:CALLS]->(delivery_impl)
MERGE (test)-[:CALLS]->(terminate_impl)
MERGE (test)-[:CALLS]->(pull_impl)
MERGE (test)-[:CALLS]->(verify_impl)
@endcypher
*/
//! Disposable PostgreSQL regressions for cross-disposition source-position arbitration.

use std::{
    env,
    future::Future,
    net::IpAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use bi_owner_runtime::{
    TaskEventInboxStore, TaskEventQuarantineStore, TaskEventReceiveAuthorization,
    TaskEventReceiveDelivery, TaskEventReceiveError, TaskEventReceiveOrigin,
    TaskEventReceiveOutcome, TaskEventReceiveRoute, TaskEventReceiveTransport, TaskEventReceiver,
    TaskEventRoutePin,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Row,
};
use tokio::{sync::Barrier, time::Instant};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_BI_TASK_EVENT_TEST_DATABASE_URL";
const DATABASE_AUDIT_URL_ENV: &str = "STAR_BI_TASK_EVENT_TEST_DATABASE_AUDIT_URL";
const DATABASE_NAME: &str = "star_bi_task_event_test";

#[derive(Default)]
struct DispositionCounts {
    inbox_ack: AtomicUsize,
    quarantine_ack: AtomicUsize,
}

struct PgDelivery {
    wire: Vec<u8>,
    origin: TaskEventReceiveOrigin,
    counts: Arc<DispositionCounts>,
}

impl TaskEventReceiveDelivery for PgDelivery {
    fn exact_wire(&self) -> &[u8] {
        &self.wire
    }

    fn origin(&self) -> &TaskEventReceiveOrigin {
        &self.origin
    }

    fn acknowledge(&self) -> impl Future<Output = Result<(), TaskEventReceiveError>> + Send {
        let counts = Arc::clone(&self.counts);
        async move {
            counts.inbox_ack.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn terminate_permanently(
        &self,
    ) -> impl Future<Output = Result<(), TaskEventReceiveError>> + Send {
        let counts = Arc::clone(&self.counts);
        async move {
            counts.quarantine_ack.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
}

struct OneShotTransport {
    delivery: Mutex<Option<PgDelivery>>,
    barrier: Option<Arc<Barrier>>,
}

impl TaskEventReceiveTransport for OneShotTransport {
    type Delivery = PgDelivery;

    async fn pull_one(
        &self,
        _route: &TaskEventReceiveRoute,
    ) -> Result<Option<Self::Delivery>, TaskEventReceiveError> {
        if let Some(barrier) = &self.barrier {
            barrier.wait().await;
        }
        Ok(self.delivery.lock().expect("delivery mutex").take())
    }
}

struct AllowAuthorization;

impl TaskEventReceiveAuthorization for AllowAuthorization {
    async fn verify_origin(
        &self,
        _route: &TaskEventReceiveRoute,
        _origin: &TaskEventReceiveOrigin,
        _exact_wire: &[u8],
    ) -> Result<(), TaskEventReceiveError> {
        Ok(())
    }
}

#[tokio::test]
#[ignore = "requires the dedicated star_bi_task_event_test PostgreSQL database with all BI migrations and runtime grants applied"]
async fn pg_source_position_is_unique_across_inbox_and_quarantine() {
    let pool = connect_disposable_pool(DATABASE_URL_ENV, "star_bi_runtime").await;
    let audit_pool = connect_disposable_pool(DATABASE_AUDIT_URL_ENV, "star_bi_owner").await;
    require_database_identity(&pool).await;
    require_database_identity(&audit_pool).await;
    require_same_database(&pool, &audit_pool).await;
    require_shared_claim_contract(&pool).await;

    let tenant = Uuid::new_v4();
    let pin = route_pin();
    let receive_route = TaskEventReceiveRoute::new(tenant, pin.clone())
        .expect("valid route for the shared BI consumer");
    // Inbox A claims the source first. Exact Inbox replay remains idempotent, while a poison
    // delivery B at the same source position cannot create a quarantine receipt or terminal ACK.
    let inbox_event = Uuid::new_v4();
    let inbox_origin = source_origin(&pin, inbox_event, 17);
    let inbox_wire = lifecycle_wire(
        tenant,
        inbox_event,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "original",
    );
    let (inbox_result, inbox_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        inbox_wire.clone(),
        inbox_origin.clone(),
        None,
    )
    .await;
    assert!(matches!(
        inbox_result,
        Ok(TaskEventReceiveOutcome::Committed(_))
    ));
    assert_eq!(inbox_counts.inbox_ack.load(Ordering::SeqCst), 1);
    assert_eq!(inbox_counts.quarantine_ack.load(Ordering::SeqCst), 0);

    let (replay_result, replay_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        inbox_wire.clone(),
        inbox_origin.clone(),
        None,
    )
    .await;
    assert!(matches!(
        replay_result,
        Ok(TaskEventReceiveOutcome::ExactReplay(_))
    ));
    assert_eq!(replay_counts.inbox_ack.load(Ordering::SeqCst), 1);
    assert_eq!(replay_counts.quarantine_ack.load(Ordering::SeqCst), 0);

    let quarantine_event = Uuid::new_v4();
    let quarantine_origin = source_origin(&pin, quarantine_event, 17);
    let (cross_quarantine_result, cross_quarantine_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        b"{".to_vec(),
        quarantine_origin.clone(),
        None,
    )
    .await;
    assert!(cross_quarantine_result.is_err());
    assert_eq!(cross_quarantine_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(
        cross_quarantine_counts
            .quarantine_ack
            .load(Ordering::SeqCst),
        0
    );
    let (quarantine_retry, quarantine_retry_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        b"{".to_vec(),
        quarantine_origin,
        None,
    )
    .await;
    assert!(quarantine_retry.is_err());
    assert_eq!(
        quarantine_retry_counts
            .quarantine_ack
            .load(Ordering::SeqCst),
        0
    );
    assert_terminal_counts(&audit_pool, tenant, &pin, 17, 1, 1, 0).await;

    // Same source identity with changed content must not route Inbox's content conflict into a
    // second quarantine terminal outcome. The original Inbox receipt remains replayable.
    let changed_content = lifecycle_wire(
        tenant,
        inbox_event,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "changed-content",
    );
    let (content_conflict_result, content_conflict_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        changed_content.clone(),
        inbox_origin.clone(),
        None,
    )
    .await;
    assert!(content_conflict_result.is_err());
    assert_eq!(content_conflict_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(
        content_conflict_counts
            .quarantine_ack
            .load(Ordering::SeqCst),
        0
    );
    let (content_conflict_retry, content_conflict_retry_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        changed_content,
        inbox_origin.clone(),
        None,
    )
    .await;
    assert!(content_conflict_retry.is_err());
    assert_eq!(
        content_conflict_retry_counts
            .quarantine_ack
            .load(Ordering::SeqCst),
        0
    );
    assert_terminal_counts(&audit_pool, tenant, &pin, 17, 1, 1, 0).await;
    let (inbox_after_conflict, _) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        inbox_wire,
        inbox_origin,
        None,
    )
    .await;
    assert!(matches!(
        inbox_after_conflict,
        Ok(TaskEventReceiveOutcome::ExactReplay(_))
    ));

    // Quarantine A claims a second source position first. Exact poison replay returns the same
    // receipt; altered poison content and a valid Inbox B cannot claim or acknowledge that slot.
    let quarantine_first_event = Uuid::new_v4();
    let quarantine_first_origin = source_origin(&pin, quarantine_first_event, 18);
    let poison_wire = b"{".to_vec();
    let (quarantine_result, quarantine_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        poison_wire.clone(),
        quarantine_first_origin.clone(),
        None,
    )
    .await;
    let quarantine_id = match quarantine_result {
        Ok(TaskEventReceiveOutcome::Quarantined { receipt, .. }) => receipt.quarantine_id,
        other => panic!("first poison delivery must commit quarantine, got {other:?}"),
    };
    assert_eq!(quarantine_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(quarantine_counts.quarantine_ack.load(Ordering::SeqCst), 1);
    let (quarantine_replay, _) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        poison_wire.clone(),
        quarantine_first_origin.clone(),
        None,
    )
    .await;
    assert!(matches!(
        quarantine_replay,
        Ok(TaskEventReceiveOutcome::Quarantined { receipt, .. }) if receipt.quarantine_id == quarantine_id
    ));

    let (changed_poison, changed_poison_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        b"{ ".to_vec(),
        quarantine_first_origin.clone(),
        None,
    )
    .await;
    assert!(changed_poison.is_err());
    assert_eq!(changed_poison_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(
        changed_poison_counts.quarantine_ack.load(Ordering::SeqCst),
        0
    );
    let (changed_poison_retry, changed_poison_retry_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        b"{ ".to_vec(),
        quarantine_first_origin.clone(),
        None,
    )
    .await;
    assert!(changed_poison_retry.is_err());
    assert_eq!(
        changed_poison_retry_counts
            .quarantine_ack
            .load(Ordering::SeqCst),
        0
    );
    assert_terminal_counts(&audit_pool, tenant, &pin, 18, 1, 0, 1).await;

    let inbox_second_event = Uuid::new_v4();
    let inbox_second_origin = source_origin(&pin, inbox_second_event, 18);
    let inbox_second_wire = lifecycle_wire(
        tenant,
        inbox_second_event,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "inbox-after-quarantine",
    );
    let (cross_inbox_result, cross_inbox_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        inbox_second_wire.clone(),
        inbox_second_origin.clone(),
        None,
    )
    .await;
    assert!(cross_inbox_result.is_err());
    assert_eq!(cross_inbox_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(cross_inbox_counts.quarantine_ack.load(Ordering::SeqCst), 0);
    let (cross_inbox_retry, cross_inbox_retry_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        inbox_second_wire,
        inbox_second_origin,
        None,
    )
    .await;
    assert!(cross_inbox_retry.is_err());
    assert_eq!(cross_inbox_retry_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_terminal_counts(&audit_pool, tenant, &pin, 18, 1, 0, 1).await;

    // Concurrent Inbox and quarantine deliveries at one third position must produce exactly
    // one durable disposition; the losing delivery cannot authorize either kind of ACK.
    let race_event = Uuid::new_v4();
    let race_origin = source_origin(&pin, race_event, 19);
    let race_valid_wire =
        lifecycle_wire(tenant, race_event, Uuid::new_v4(), Uuid::new_v4(), "race");
    let race_poison_origin = source_origin(&pin, Uuid::new_v4(), 19);
    let race_poison_wire = b"{".to_vec();
    let barrier = Arc::new(Barrier::new(2));
    let inbox_race = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        race_valid_wire.clone(),
        race_origin.clone(),
        Some(Arc::clone(&barrier)),
    );
    let quarantine_race = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        race_poison_wire.clone(),
        race_poison_origin.clone(),
        Some(barrier),
    );
    let ((inbox_race_result, inbox_race_counts), (quarantine_race_result, quarantine_race_counts)) =
        tokio::join!(inbox_race, quarantine_race);
    assert_ne!(inbox_race_result.is_ok(), quarantine_race_result.is_ok());
    let total_dispositions = inbox_race_counts.inbox_ack.load(Ordering::SeqCst)
        + inbox_race_counts.quarantine_ack.load(Ordering::SeqCst)
        + quarantine_race_counts.inbox_ack.load(Ordering::SeqCst)
        + quarantine_race_counts.quarantine_ack.load(Ordering::SeqCst);
    assert_eq!(
        total_dispositions, 1,
        "exactly one terminal ACK may follow the owner claim"
    );
    if inbox_race_result.is_ok() {
        let (loser_retry, loser_retry_counts) = receive_one(
            TaskEventReceiveRoute::new(tenant, pin.clone()).expect("valid race retry route"),
            &pool,
            &pin,
            race_poison_wire,
            race_poison_origin,
            None,
        )
        .await;
        assert!(loser_retry.is_err());
        assert_eq!(loser_retry_counts.quarantine_ack.load(Ordering::SeqCst), 0);
    } else {
        let (loser_retry, loser_retry_counts) = receive_one(
            TaskEventReceiveRoute::new(tenant, pin.clone()).expect("valid race retry route"),
            &pool,
            &pin,
            race_valid_wire,
            race_origin,
            None,
        )
        .await;
        assert!(loser_retry.is_err());
        assert_eq!(loser_retry_counts.inbox_ack.load(Ordering::SeqCst), 0);
    }
    let (claims, inbox_receipts, quarantine_receipts) =
        terminal_counts(&audit_pool, tenant, &pin, 19).await;
    assert_eq!(claims, 1);
    assert_eq!(inbox_receipts + quarantine_receipts, 1);

    // A shared cursor is unique even when two terminal paths report different sequences.
    let repeated_cursor = b"shared-cursor-different-sequence".to_vec();
    let cursor_owner_event = Uuid::new_v4();
    let cursor_owner_origin =
        source_origin_with_cursor(&pin, cursor_owner_event, 20, repeated_cursor.clone());
    let (cursor_owner_result, cursor_owner_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        b"{".to_vec(),
        cursor_owner_origin.clone(),
        None,
    )
    .await;
    assert!(matches!(
        cursor_owner_result,
        Ok(TaskEventReceiveOutcome::Quarantined { .. })
    ));
    assert_eq!(cursor_owner_counts.quarantine_ack.load(Ordering::SeqCst), 1);

    let repeated_cursor_event = Uuid::new_v4();
    let repeated_cursor_origin =
        source_origin_with_cursor(&pin, repeated_cursor_event, 21, repeated_cursor);
    let repeated_cursor_wire = lifecycle_wire(
        tenant,
        repeated_cursor_event,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "same cursor at a different sequence",
    );
    let (repeated_cursor_result, repeated_cursor_counts) = receive_one(
        receive_route.clone(),
        &pool,
        &pin,
        repeated_cursor_wire,
        repeated_cursor_origin,
        None,
    )
    .await;
    assert!(repeated_cursor_result.is_err());
    assert_eq!(repeated_cursor_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(
        repeated_cursor_counts.quarantine_ack.load(Ordering::SeqCst),
        0
    );
    assert_terminal_counts(&audit_pool, tenant, &pin, 20, 1, 0, 1).await;
    assert_terminal_counts(&audit_pool, tenant, &pin, 21, 0, 0, 0).await;

    // A known-first generation cannot transition to a new cursorless claim later.
    let known_first_pin = route_pin();
    let known_first_route =
        TaskEventReceiveRoute::new(tenant, known_first_pin.clone()).expect("known-first route");
    let known_first_event = Uuid::new_v4();
    let known_first_origin = source_origin_with_cursor(
        &known_first_pin,
        known_first_event,
        40,
        b"known-first-cursor".to_vec(),
    );
    let known_first_wire = lifecycle_wire(
        tenant,
        known_first_event,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "known-first mode",
    );
    let (known_first_result, known_first_counts) = receive_one(
        known_first_route,
        &pool,
        &known_first_pin,
        known_first_wire,
        known_first_origin,
        None,
    )
    .await;
    assert!(matches!(
        known_first_result,
        Ok(TaskEventReceiveOutcome::Committed(_))
    ));
    assert_eq!(known_first_counts.inbox_ack.load(Ordering::SeqCst), 1);
    let late_cursorless = seed_cursorless_quarantine_history(
        &audit_pool,
        tenant,
        &known_first_pin,
        41,
        Uuid::new_v4(),
        None,
    )
    .await;
    assert_pg_mode_conflict(
        &late_cursorless.expect_err("known-first generation rejects a new cursorless claim"),
        "known cursor history",
    );
    assert_terminal_counts(&audit_pool, tenant, &known_first_pin, 40, 1, 1, 0).await;
    assert_terminal_counts(&audit_pool, tenant, &known_first_pin, 41, 0, 0, 0).await;

    // Cursorless-first history uses its own fresh generation. New cursor-bearing positions fail;
    // retries cannot create an Inbox receipt or authorize an ACK.
    let cursorless_pin = route_pin();
    let cursorless_route =
        TaskEventReceiveRoute::new(tenant, cursorless_pin.clone()).expect("cursorless-first route");
    seed_cursorless_quarantine_history(
        &audit_pool,
        tenant,
        &cursorless_pin,
        50,
        Uuid::new_v4(),
        None,
    )
    .await
    .expect("cursorless-first history commits in its fresh generation");
    assert_terminal_counts(&audit_pool, tenant, &cursorless_pin, 50, 1, 0, 1).await;

    let after_gap_event = Uuid::new_v4();
    let after_gap_origin = source_origin_with_cursor(
        &cursorless_pin,
        after_gap_event,
        51,
        b"cursor-after-unknown-history".to_vec(),
    );
    let after_gap_wire = lifecycle_wire(
        tenant,
        after_gap_event,
        Uuid::new_v4(),
        Uuid::new_v4(),
        "must fail closed after cursorless history",
    );
    let (after_gap_result, after_gap_counts) = receive_one(
        cursorless_route.clone(),
        &pool,
        &cursorless_pin,
        after_gap_wire.clone(),
        after_gap_origin.clone(),
        None,
    )
    .await;
    assert!(after_gap_result.is_err());
    assert!(format!("{after_gap_result:?}").contains("unknown cursor history"));
    assert_eq!(after_gap_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_eq!(after_gap_counts.quarantine_ack.load(Ordering::SeqCst), 0);
    assert_terminal_counts(&audit_pool, tenant, &cursorless_pin, 51, 0, 0, 0).await;

    let (after_gap_retry, after_gap_retry_counts) = receive_one(
        cursorless_route,
        &pool,
        &cursorless_pin,
        after_gap_wire,
        after_gap_origin,
        None,
    )
    .await;
    assert!(after_gap_retry.is_err());
    assert_eq!(after_gap_retry_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_terminal_counts(&audit_pool, tenant, &cursorless_pin, 51, 0, 0, 0).await;

    // Migration backfill may preserve an already mixed legacy generation. Only fully identical
    // replays of its historical rows are allowed; new unmatched claims in either mode fail.
    let mixed_pin = route_pin();
    let mixed_route = TaskEventReceiveRoute::new(tenant, mixed_pin.clone()).expect("mixed route");
    seed_cursorless_quarantine_history(&audit_pool, tenant, &mixed_pin, 60, Uuid::new_v4(), None)
        .await
        .expect("seed a historical cursorless row in a fresh generation");

    let legacy_sequence = 59;
    let legacy_message_id = Uuid::new_v4();
    let legacy_cursor = b"backfilled-known-cursor".to_vec();
    let legacy_wire = b"{".to_vec();
    let legacy_digest: [u8; 32] = Sha256::digest(&legacy_wire).into();
    seed_legacy_nonnull_quarantine_claim(
        &audit_pool,
        tenant,
        &mixed_pin,
        legacy_sequence,
        &legacy_cursor,
        legacy_message_id,
        legacy_digest,
    )
    .await;
    let legacy_origin = source_origin_with_cursor(
        &mixed_pin,
        legacy_message_id,
        legacy_sequence,
        legacy_cursor.clone(),
    );
    let legacy_receipt_id = seed_legacy_nonnull_quarantine_receipt(
        &audit_pool,
        tenant,
        &mixed_pin,
        legacy_sequence,
        &legacy_cursor,
        legacy_message_id,
        &legacy_wire,
        legacy_digest,
    )
    .await;
    assert_terminal_counts(&audit_pool, tenant, &mixed_pin, 59, 1, 0, 1).await;
    assert_terminal_counts(&audit_pool, tenant, &mixed_pin, 60, 1, 0, 1).await;

    let (legacy_replay, legacy_replay_counts) = receive_one(
        mixed_route.clone(),
        &pool,
        &mixed_pin,
        legacy_wire,
        legacy_origin,
        None,
    )
    .await;
    assert!(matches!(
        legacy_replay,
        Ok(TaskEventReceiveOutcome::Quarantined { receipt, .. })
            if receipt.quarantine_id == legacy_receipt_id
    ));
    assert_eq!(
        legacy_replay_counts.quarantine_ack.load(Ordering::SeqCst),
        1
    );

    let mixed_known_event = Uuid::new_v4();
    let mixed_known_origin = source_origin_with_cursor(
        &mixed_pin,
        mixed_known_event,
        61,
        b"new-known-after-mixed-history".to_vec(),
    );
    let (mixed_known_result, mixed_known_counts) = receive_one(
        mixed_route.clone(),
        &pool,
        &mixed_pin,
        lifecycle_wire(
            tenant,
            mixed_known_event,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "new known claim must fail",
        ),
        mixed_known_origin,
        None,
    )
    .await;
    assert!(mixed_known_result.is_err());
    assert!(format!("{mixed_known_result:?}").contains("unknown cursor history"));
    assert_eq!(mixed_known_counts.inbox_ack.load(Ordering::SeqCst), 0);
    assert_terminal_counts(&audit_pool, tenant, &mixed_pin, 61, 0, 0, 0).await;

    let mixed_late_cursorless = seed_cursorless_quarantine_history(
        &audit_pool,
        tenant,
        &mixed_pin,
        62,
        Uuid::new_v4(),
        None,
    )
    .await;
    assert_pg_mode_conflict(
        &mixed_late_cursorless.expect_err("mixed legacy generation rejects a new cursorless claim"),
        "known cursor history",
    );
    assert_terminal_counts(&audit_pool, tenant, &mixed_pin, 62, 0, 0, 0).await;

    // Both modes start together in another fresh generation. The shared/exclusive lock and
    // symmetric checks must let exactly one transaction commit, independent of lock winner.
    let race_pin = route_pin();
    let race_route = TaskEventReceiveRoute::new(tenant, race_pin.clone()).expect("mode-race route");
    let race_barrier = Arc::new(Barrier::new(2));
    let race_event = Uuid::new_v4();
    let race_origin =
        source_origin_with_cursor(&race_pin, race_event, 70, b"parallel-known-mode".to_vec());
    let known_mode_race = receive_one(
        race_route,
        &pool,
        &race_pin,
        lifecycle_wire(
            tenant,
            race_event,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "concurrent known mode",
        ),
        race_origin,
        Some(Arc::clone(&race_barrier)),
    );
    let cursorless_mode_race = seed_cursorless_quarantine_history(
        &audit_pool,
        tenant,
        &race_pin,
        71,
        Uuid::new_v4(),
        Some(race_barrier),
    );
    let ((known_mode_result, known_mode_counts), cursorless_mode_result) =
        tokio::join!(known_mode_race, cursorless_mode_race);
    assert_ne!(known_mode_result.is_ok(), cursorless_mode_result.is_ok());
    if let Err(error) = &known_mode_result {
        assert!(format!("{error:?}").contains("unknown cursor history"));
        assert_eq!(known_mode_counts.inbox_ack.load(Ordering::SeqCst), 0);
    } else {
        assert_eq!(known_mode_counts.inbox_ack.load(Ordering::SeqCst), 1);
    }
    if let Err(error) = &cursorless_mode_result {
        assert_pg_mode_conflict(error, "known cursor history");
    }
    let (known_race_claims, known_race_inbox, known_race_quarantine) =
        terminal_counts(&audit_pool, tenant, &race_pin, 70).await;
    let (null_race_claims, null_race_inbox, null_race_quarantine) =
        terminal_counts(&audit_pool, tenant, &race_pin, 71).await;
    assert_eq!(known_race_claims + null_race_claims, 1);
    assert_eq!(
        known_race_inbox + known_race_quarantine + null_race_inbox + null_race_quarantine,
        1
    );

    pool.close().await;
    audit_pool.close().await;
}

async fn connect_disposable_pool(env_name: &str, expected_role: &str) -> PgPool {
    let database_url = env::var(env_name).unwrap_or_else(|_| {
        panic!("BI source-position proof requires {env_name} for the dedicated disposable PostgreSQL database")
    });
    let options = database_url
        .parse::<PgConnectOptions>()
        .expect("parse disposable PostgreSQL connection options");
    let host = options.get_host();
    let loopback = host == "localhost"
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    assert!(
        loopback && options.get_port() != 5432,
        "this test may write only to loopback PostgreSQL on a non-default port"
    );
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .expect("connect to the disposable BI persistence database");
    let current_user: String = sqlx::query_scalar("SELECT current_user")
        .fetch_one(&pool)
        .await
        .expect("read the database role before running source-position assertions");
    assert_eq!(
        current_user, expected_role,
        "unexpected database role for {env_name}"
    );
    pool
}

async fn require_database_identity(pool: &PgPool) {
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect("read the connected database identity before writing fixtures");
    assert_eq!(
        database, DATABASE_NAME,
        "only the dedicated disposable BI database is allowed"
    );
}

async fn require_same_database(runtime: &PgPool, audit: &PgPool) {
    let runtime_identity = sqlx::query_as::<_, (String, Option<String>, i32)>(
        "SELECT current_database(), inet_server_addr()::text, inet_server_port()",
    )
    .fetch_one(runtime)
    .await
    .expect("read runtime database endpoint");
    let audit_identity = sqlx::query_as::<_, (String, Option<String>, i32)>(
        "SELECT current_database(), inet_server_addr()::text, inet_server_port()",
    )
    .fetch_one(audit)
    .await
    .expect("read audit database endpoint");
    assert_eq!(
        runtime_identity, audit_identity,
        "runtime and audit URLs must target one database"
    );
}

async fn require_shared_claim_contract(pool: &PgPool) {
    let row = sqlx::query(
        r#"
        SELECT
            EXISTS (
                SELECT 1 FROM pg_class
                 WHERE oid = to_regclass('bi.task_event_source_position_claim')
                   AND relrowsecurity AND relforcerowsecurity
            ) AS claim_force_rls,
            EXISTS (
                SELECT 1 FROM pg_trigger
                 WHERE tgrelid = to_regclass('bi.task_event_source_origin')
                   AND tgname = 'task_event_source_origin_claim_position'
                   AND NOT tgisinternal AND tgenabled <> 'D'
            ) AS inbox_claim_trigger,
            EXISTS (
                SELECT 1 FROM pg_trigger
                 WHERE tgrelid = to_regclass('bi.task_event_quarantine')
                   AND tgname = 'task_event_quarantine_claim_position'
                   AND NOT tgisinternal AND tgenabled <> 'D'
            ) AS quarantine_claim_trigger,
            EXISTS (
                SELECT 1 FROM pg_index
                 WHERE indexrelid = to_regclass('bi.task_event_source_position_claim_pkey')
                   AND indisunique
            ) AS sequence_unique,
            EXISTS (
                SELECT 1 FROM pg_index
                 WHERE indexrelid = to_regclass('bi.task_event_source_position_claim_cursor_uq')
                   AND indisunique
            ) AS cursor_unique,
            EXISTS (
                SELECT 1 FROM pg_index
                 WHERE indexrelid = to_regclass('bi.task_event_source_position_claim_cursorless_scope_idx')
                   AND NOT indisunique
            ) AS cursorless_scope_index,
            NOT has_table_privilege(current_user, 'bi.task_event_source_position_claim', 'SELECT')
              AND NOT has_table_privilege(current_user, 'bi.task_event_source_position_claim', 'INSERT')
              AND NOT has_table_privilege(current_user, 'bi.task_event_source_position_claim', 'UPDATE')
              AND NOT has_table_privilege(current_user, 'bi.task_event_source_position_claim', 'DELETE')
                AS runtime_has_no_direct_access,
            NOT has_function_privilege(current_user, 'bi.claim_task_event_source_position()', 'EXECUTE')
                AS runtime_cannot_call_trigger_function,
            EXISTS (
                SELECT 1 FROM pg_proc p JOIN pg_roles r ON r.oid = p.proowner
                 WHERE p.oid = to_regprocedure('bi.claim_task_event_source_position()')
                   AND p.prosecdef AND r.rolname = 'star_bi_owner'
            ) AS trigger_is_owner_definer,
            NOT EXISTS (
                SELECT 1 FROM pg_attribute
                 WHERE attrelid = to_regclass('bi.task_event_source_position_claim')
                   AND attnum > 0 AND NOT attisdropped
                   AND attname IN ('wire', 'raw_wire', 'wire_bytes', 'payload')
            ) AS claim_has_no_wire_bytes
        FROM pg_roles
        WHERE rolname = current_user
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("inspect source-position claim constraints, RLS, grants, and wire privacy");

    for key in [
        "claim_force_rls",
        "inbox_claim_trigger",
        "quarantine_claim_trigger",
        "sequence_unique",
        "cursor_unique",
        "cursorless_scope_index",
        "runtime_has_no_direct_access",
        "runtime_cannot_call_trigger_function",
        "trigger_is_owner_definer",
        "claim_has_no_wire_bytes",
    ] {
        assert!(
            row.try_get::<bool, _>(key).expect("claim contract column"),
            "{key}"
        );
    }
}

async fn assert_terminal_counts(
    audit_pool: &PgPool,
    tenant: Uuid,
    pin: &TaskEventRoutePin,
    sequence: i64,
    expected_claims: i64,
    expected_inbox_receipts: i64,
    expected_quarantine_receipts: i64,
) {
    let (claims, inbox_receipts, quarantine_receipts) =
        terminal_counts(audit_pool, tenant, pin, sequence).await;
    assert_eq!(claims, expected_claims, "source-position claim count");
    assert_eq!(
        inbox_receipts, expected_inbox_receipts,
        "Inbox source-origin receipt count"
    );
    assert_eq!(
        quarantine_receipts, expected_quarantine_receipts,
        "quarantine receipt count"
    );
}

async fn terminal_counts(
    audit_pool: &PgPool,
    tenant: Uuid,
    pin: &TaskEventRoutePin,
    sequence: i64,
) -> (i64, i64, i64) {
    let mut tx = audit_pool
        .begin()
        .await
        .expect("begin tenant-scoped audit read transaction");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("set the tenant scope for forced-RLS audit reads");
    let row = sqlx::query(
        r#"
        WITH position AS (
            SELECT $1::UUID AS tenant_id, $2::VARCHAR AS provider_identity,
                   $3::VARCHAR AS source_log_id, $4::UUID AS source_log_generation,
                   $5::BIGINT AS source_sequence
        )
        SELECT
            (SELECT count(*)::BIGINT
               FROM bi.task_event_source_position_claim AS claim, position
              WHERE claim.tenant_id = position.tenant_id
                AND claim.consumer_id = 'bi.task-event-fact.v1'
                AND claim.provider_identity = position.provider_identity
                AND claim.source_log_id = position.source_log_id
                AND claim.source_log_generation = position.source_log_generation
                AND claim.source_sequence = position.source_sequence) AS claims,
            (SELECT count(*)::BIGINT
               FROM bi.task_event_source_origin AS source, position
              WHERE source.tenant_id = position.tenant_id
                AND source.consumer_id = 'bi.task-event-fact.v1'
                AND source.provider_identity = position.provider_identity
                AND source.source_log_id = position.source_log_id
                AND source.source_log_generation = position.source_log_generation
                AND source.source_sequence = position.source_sequence) AS inbox_receipts,
            (SELECT count(*)::BIGINT
               FROM bi.task_event_quarantine AS saved, position
              WHERE saved.tenant_id = position.tenant_id
                AND saved.consumer_id = 'bi.task-event-fact.v1'
                AND saved.origin_provider_identity = position.provider_identity
                AND saved.source_log_id = position.source_log_id
                AND saved.source_log_generation = position.source_log_generation
                AND saved.source_sequence = position.source_sequence) AS quarantine_receipts
        "#,
    )
    .bind(tenant)
    .bind(&pin.transport_provider_id)
    .bind(&pin.destination_id)
    .bind(pin.destination_generation)
    .bind(sequence)
    .fetch_one(&mut *tx)
    .await
    .expect("read source claim and both terminal paths through the owner audit role");
    let counts = (
        row.try_get("claims").expect("claim count"),
        row.try_get("inbox_receipts")
            .expect("Inbox source-origin count"),
        row.try_get("quarantine_receipts")
            .expect("quarantine receipt count"),
    );
    tx.commit()
        .await
        .expect("commit read-only audit transaction");
    counts
}

async fn seed_cursorless_quarantine_history(
    audit_pool: &PgPool,
    tenant: Uuid,
    pin: &TaskEventRoutePin,
    sequence: i64,
    message_id: Uuid,
    start_barrier: Option<Arc<Barrier>>,
) -> Result<Uuid, sqlx::Error> {
    if let Some(barrier) = start_barrier {
        barrier.wait().await;
    }
    let mut tx = audit_pool
        .begin()
        .await
        .expect("begin tenant-scoped cursorless history fixture transaction");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("set tenant scope before seeding legacy cursorless quarantine history");
    let inserted = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO bi.task_event_quarantine (
            tenant_id, sender_service_id, receiver_service_id,
            transport_provider_id, transport_provider_version,
            transport_profile_id, transport_profile_revision, transport_profile_digest,
            endpoint_identity, destination_id, destination_generation,
            origin_provider_identity, source_log_id, source_log_generation,
            source_sequence, source_cursor, message_id, wire_sha256,
            wire_length, fault_code
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, NULL, $16, $17, 1, 'malformed_envelope'
        )
        RETURNING quarantine_id
        "#,
    )
    .bind(tenant)
    .bind(pin.sender_service_id)
    .bind(pin.receiver_service_id)
    .bind(&pin.transport_provider_id)
    .bind(&pin.transport_provider_version)
    .bind(pin.transport_profile_id)
    .bind(pin.transport_profile_revision)
    .bind(pin.transport_profile_digest.to_vec())
    .bind(&pin.endpoint_identity)
    .bind(&pin.destination_id)
    .bind(pin.destination_generation)
    .bind(&pin.transport_provider_id)
    .bind(&pin.destination_id)
    .bind(pin.destination_generation)
    .bind(sequence)
    .bind(message_id)
    .bind(vec![0x71_u8; 32])
    .fetch_one(&mut *tx)
    .await;
    let quarantine_id = match inserted {
        Ok(quarantine_id) => quarantine_id,
        Err(error) => {
            let _ = tx.rollback().await;
            return Err(error);
        }
    };
    tx.commit().await?;
    Ok(quarantine_id)
}

async fn seed_legacy_nonnull_quarantine_claim(
    audit_pool: &PgPool,
    tenant: Uuid,
    pin: &TaskEventRoutePin,
    sequence: i64,
    cursor: &[u8],
    message_id: Uuid,
    wire_digest: [u8; 32],
) {
    let mut tx = audit_pool
        .begin()
        .await
        .expect("begin tenant-scoped legacy claim fixture transaction");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("set tenant scope before seeding historical claim");
    sqlx::query(
        r#"
        INSERT INTO bi.task_event_source_position_claim (
            tenant_id, consumer_id, provider_identity, source_log_id,
            source_log_generation, source_sequence, source_cursor, source_message_id,
            disposition, wire_sha256
        ) VALUES (
            $1, 'bi.task-event-fact.v1', $2, $3, $4, $5, $6, $7, 'quarantine', $8
        )
        "#,
    )
    .bind(tenant)
    .bind(&pin.transport_provider_id)
    .bind(&pin.destination_id)
    .bind(pin.destination_generation)
    .bind(sequence)
    .bind(cursor)
    .bind(message_id)
    .bind(wire_digest.to_vec())
    .execute(&mut *tx)
    .await
    .expect("backfill a historical known-cursor claim into the mixed generation fixture");
    tx.commit().await.expect("commit historical claim fixture");
}

async fn seed_legacy_nonnull_quarantine_receipt(
    audit_pool: &PgPool,
    tenant: Uuid,
    pin: &TaskEventRoutePin,
    sequence: i64,
    cursor: &[u8],
    message_id: Uuid,
    wire: &[u8],
    wire_digest: [u8; 32],
) -> Uuid {
    let mut tx = audit_pool
        .begin()
        .await
        .expect("begin tenant-scoped legacy receipt fixture transaction");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("set tenant scope before seeding historical quarantine receipt");
    let quarantine_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO bi.task_event_quarantine (
            tenant_id, sender_service_id, receiver_service_id,
            transport_provider_id, transport_provider_version,
            transport_profile_id, transport_profile_revision, transport_profile_digest,
            endpoint_identity, destination_id, destination_generation,
            origin_provider_identity, source_log_id, source_log_generation,
            source_sequence, source_cursor, message_id, wire_sha256,
            wire_length, fault_code
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16, $17, $18, $19, 'malformed_envelope'
        )
        RETURNING quarantine_id
        "#,
    )
    .bind(tenant)
    .bind(pin.sender_service_id)
    .bind(pin.receiver_service_id)
    .bind(&pin.transport_provider_id)
    .bind(&pin.transport_provider_version)
    .bind(pin.transport_profile_id)
    .bind(pin.transport_profile_revision)
    .bind(pin.transport_profile_digest.to_vec())
    .bind(&pin.endpoint_identity)
    .bind(&pin.destination_id)
    .bind(pin.destination_generation)
    .bind(&pin.transport_provider_id)
    .bind(&pin.destination_id)
    .bind(pin.destination_generation)
    .bind(sequence)
    .bind(cursor)
    .bind(message_id)
    .bind(wire_digest.to_vec())
    .bind(i32::try_from(wire.len()).expect("wire length fits the quarantine limit"))
    .fetch_one(&mut *tx)
    .await
    .expect("insert historical quarantine metadata through exact shared-claim replay");
    tx.commit()
        .await
        .expect("commit historical receipt fixture");
    quarantine_id
}

fn assert_pg_mode_conflict(error: &sqlx::Error, expected_fragment: &str) {
    let database_error = error
        .as_database_error()
        .expect("mode barrier should reject with a PostgreSQL database error");
    assert_eq!(database_error.code().as_deref(), Some("23505"));
    assert!(
        database_error.message().contains(expected_fragment),
        "unexpected PostgreSQL mode-conflict error: {database_error}"
    );
}

async fn receive_one(
    route: TaskEventReceiveRoute,
    pool: &PgPool,
    pin: &TaskEventRoutePin,
    wire: Vec<u8>,
    origin: TaskEventReceiveOrigin,
    barrier: Option<Arc<Barrier>>,
) -> (
    Result<TaskEventReceiveOutcome, TaskEventReceiveError>,
    Arc<DispositionCounts>,
) {
    let counts = Arc::new(DispositionCounts::default());
    let delivery = PgDelivery {
        wire,
        origin,
        counts: Arc::clone(&counts),
    };
    let receiver = TaskEventReceiver::new(
        route,
        OneShotTransport {
            delivery: Mutex::new(Some(delivery)),
            barrier,
        },
        AllowAuthorization,
        TaskEventInboxStore::new(pool.clone(), pin.receiver_service_id),
        TaskEventQuarantineStore::new(pool.clone()),
    );
    let result = receiver
        .receive_one(
            &CancellationToken::new(),
            Instant::now() + Duration::from_secs(10),
        )
        .await;
    (result, counts)
}

fn route_pin() -> TaskEventRoutePin {
    TaskEventRoutePin {
        sender_service_id: Uuid::new_v4(),
        receiver_service_id: Uuid::new_v4(),
        transport_provider_id: "star-rust-log".to_owned(),
        transport_provider_version: "1.0.0".to_owned(),
        transport_profile_id: Uuid::new_v4(),
        transport_profile_revision: 1,
        transport_profile_digest: [0x42; 32],
        endpoint_identity: "bi-source-position-pg-test".to_owned(),
        destination_id: "bi-task-events".to_owned(),
        destination_generation: Uuid::new_v4(),
    }
}

fn source_origin(
    pin: &TaskEventRoutePin,
    message_id: Uuid,
    sequence: i64,
) -> TaskEventReceiveOrigin {
    source_origin_with_cursor(pin, message_id, sequence, sequence.to_be_bytes().to_vec())
}

fn source_origin_with_cursor(
    pin: &TaskEventRoutePin,
    message_id: Uuid,
    sequence: i64,
    cursor: Vec<u8>,
) -> TaskEventReceiveOrigin {
    TaskEventReceiveOrigin::new(
        pin.transport_provider_id.clone(),
        pin.destination_id.clone(),
        pin.destination_generation,
        sequence,
        cursor,
        message_id,
    )
    .expect("valid source-position fixture")
}

fn lifecycle_wire(
    tenant_id: Uuid,
    event_id: Uuid,
    project_id: Uuid,
    run_id: Uuid,
    private_note: &str,
) -> Vec<u8> {
    let work_item_id = Uuid::new_v4();
    let payload = json!({
        "schema_version": 3,
        "aggregate": {"type": "lifecycle", "version": 7},
        "work_item_id": work_item_id,
        "owner": {
            "tenant_id": tenant_id,
            "project_id": project_id,
            "engineering_run_id": run_id
        },
        "lifecycle": {"status": "in_progress", "review_state": "none"},
        "change": {
            "event_type": "task.lifecycle.changed",
            "from_status": "claimed",
            "to_status": "in_progress",
            "private_note": private_note
        },
        "host_context": {"branch_name": "fixture/ref"}
    });
    let wire: Value = json!({
        "envelope_version": 1,
        "tenant_id": tenant_id,
        "project_id": project_id,
        "engineering_run_id": run_id,
        "event_id": event_id,
        "work_item_id": work_item_id,
        "aggregate_type": "lifecycle",
        "aggregate_version": 7,
        "event_type": "task.lifecycle.changed",
        "schema_version": 3,
        "actor_id": Uuid::new_v4(),
        "correlation_id": Uuid::new_v4(),
        "occurred_at": "2026-10-25T00:00:00Z",
        "payload": payload
    });
    serde_json::to_vec(&wire).expect("valid lifecycle fixture serializes")
}
