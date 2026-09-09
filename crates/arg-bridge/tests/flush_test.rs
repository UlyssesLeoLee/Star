// SPDX-License-Identifier: MIT OR Apache-2.0
//! `flush_test` — 3 UT (per DD §10.1.2 UT-28..UT-30).
//!
//! Covers [`PeriodFlushWorker`]:
//! - UT-28 test_period_flush_30s — 周期触发 (via the configurable period)
//! - UT-29 test_period_flush_batch — 批量写
//!
//! Note: UT-30 in the DD enumeration is `test_offline_queue_write`; that
//! is covered in `offline_test.rs` (per the file split below).

use std::sync::Arc;
use std::time::Duration;

use star_arg::client::MemgraphClient;
use star_arg_bridge::{
    BridgeEnvelope, BridgeEnvelopeKind, MemgraphEventListener, PeriodFlushWorker,
};
use tokio::sync::mpsc;
use uuid::Uuid;

fn make_client() -> Arc<MemgraphClient> {
    Arc::new(MemgraphClient::new(
        "bolt://localhost:7687".into(),
        "test".into(),
        "test".into(),
        1,
    ))
}

fn make_listener() -> Arc<MemgraphEventListener> {
    Arc::new(MemgraphEventListener::new(make_client()))
}

fn make_envelope(score: f64) -> BridgeEnvelope {
    BridgeEnvelope::new(
        Uuid::new_v4(),
        BridgeEnvelopeKind::TrustScoreUpdate(star_arg_bridge::protocol::ArgTrustScoreUpdate {
            agent_id: Uuid::new_v4(),
            old_score: score,
            new_score: score + 0.01,
            delta_reason: "success".to_string(),
            trigger_sa: "SA_03".to_string(),
            created_at: chrono::Utc::now(),
        }),
        0,
    )
}

#[tokio::test]
async fn ut28_period_flush_30s_default_period() {
    // The default period must be 30 s per BD §7.2.
    let (tx, rx) = mpsc::channel::<BridgeEnvelope>(8);
    drop(tx);
    let worker = PeriodFlushWorker::new(make_client(), make_listener(), rx);
    assert_eq!(worker.period(), Duration::from_secs(30));
    assert_eq!(worker.batch_cap(), 256);
    assert_eq!(worker.seen_count(), 0);
}

#[tokio::test]
async fn ut29_period_flush_batch_persists_and_dedups() {
    // Build 5 envelopes and call `flush_batch` directly so the result
    // does not depend on the 50 ms deadline used by `flush_one_batch`.
    let (_tx, rx) = mpsc::channel::<BridgeEnvelope>(8);
    let mut worker = PeriodFlushWorker::with_options(
        make_client(),
        make_listener(),
        rx,
        Duration::from_secs(30),
        4,
    );

    let envelopes: Vec<BridgeEnvelope> = (0..5)
        .map(|i| make_envelope(0.5 + 0.01 * f64::from(i)))
        .collect();

    let outcome = worker.flush_batch(&envelopes).await.expect("flush ok");
    assert_eq!(outcome.dequeued, 5);
    assert_eq!(outcome.persisted, 5);
    assert_eq!(outcome.failed, 0);
    assert_eq!(worker.seen_count(), 5);
    assert_eq!(outcome.persisted_ids.len(), 5);

    // Re-flushing the same batch yields zero persisted (dedup).
    let outcome2 = worker.flush_batch(&envelopes).await.expect("flush ok");
    assert_eq!(outcome2.dequeued, 5);
    assert_eq!(outcome2.persisted, 0);
    assert_eq!(outcome2.failed, 0);
}

#[tokio::test]
async fn ut29b_period_flush_idempotency() {
    // Manually re-flush the same envelope via `flush_batch` to confirm
    // idempotency holds across batches.
    let (_tx, rx) = mpsc::channel::<BridgeEnvelope>(8);
    let mut worker = PeriodFlushWorker::with_options(
        make_client(),
        make_listener(),
        rx,
        Duration::from_secs(30),
        4,
    );
    let env = make_envelope(0.42);

    let outcome1 = worker
        .flush_batch(std::slice::from_ref(&env))
        .await
        .expect("flush ok");
    assert_eq!(outcome1.persisted, 1);

    let outcome2 = worker
        .flush_batch(std::slice::from_ref(&env))
        .await
        .expect("flush ok");
    assert_eq!(outcome2.persisted, 0, "second flush must be dedup");
    assert_eq!(outcome2.failed, 0);
}
