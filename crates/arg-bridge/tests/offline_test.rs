// SPDX-License-Identifier: MIT OR Apache-2.0
//! `offline_test` — 4 UT (per DD §10.1.2 UT-30..UT-33).
//!
//! Covers [`OfflineQueue`]:
//! - UT-30 test_offline_queue_write — 不可达时缓存
//! - UT-31 test_offline_queue_read  — 重连后 flush
//! - UT-32 test_offline_queue_full  — 满 10000 拒绝
//! - UT-33 (proxy) — `langgraph_state_update` and `langgraph_state_5_reducer`
//!   are covered in `langgraph_test.rs` (split for readability, see below).

use std::time::Duration;

use star_arg_bridge::{
    protocol::ArgAchievementUnlocked, BridgeEnvelope, BridgeEnvelopeKind, OfflineQueue,
};
use tempfile::TempDir;
use uuid::Uuid;

fn make_envelope(achievement: &str) -> BridgeEnvelope {
    BridgeEnvelope::new(
        Uuid::new_v4(),
        BridgeEnvelopeKind::AchievementUnlocked(ArgAchievementUnlocked {
            unlock_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            achievement_id: achievement.to_string(),
            topology_snapshot: serde_json::json!({}),
            unlock_text: format!("unlocked {achievement}"),
            created_at: chrono::Utc::now(),
        }),
        0,
    )
}

#[test]
fn ut30_offline_queue_write() {
    // Persist 3 envelopes; the queue is in-memory so no filesystem.
    let q = OfflineQueue::in_memory().expect("in-memory queue");
    assert!(q.is_empty());

    for code in ["TOP-001", "TOP-002", "TOP-003"] {
        let env = make_envelope(code);
        q.persist(&env).expect("persist ok");
    }
    assert_eq!(q.len(), 3);
    assert!(!q.is_empty());

    // Idempotency: persisting the same id again is a no-op.
    let env = make_envelope("TOP-DUP");
    q.persist(&env).expect("persist ok");
    q.persist(&env).expect("persist ok (dedup)");
    assert_eq!(q.len(), 4);
}

#[test]
fn ut31_offline_queue_read_after_reopen() {
    // Persist → close the directory → reopen → drain.
    let dir = TempDir::new().expect("tempdir");
    {
        let q = OfflineQueue::new(dir.path()).expect("open");
        for code in ["TOP-001", "TOP-002"] {
            q.persist(&make_envelope(code)).expect("persist");
        }
    }
    // reopen
    let q = OfflineQueue::new(dir.path()).expect("reopen");
    assert_eq!(q.len(), 2);
    let drained = q.drain().expect("drain");
    assert_eq!(drained.len(), 2);
    assert!(drained.iter().any(|e| matches!(
        &e.kind,
        BridgeEnvelopeKind::AchievementUnlocked(a) if a.achievement_id == "TOP-001"
    )));
    // After drain the queue should still report len=2 because drain
    // does not clear; we call `pop_oldest` to remove entries explicitly.
    let popped = q.pop_oldest().expect("pop").expect("some");
    assert!(matches!(
        popped.kind,
        BridgeEnvelopeKind::AchievementUnlocked(_)
    ));
    assert_eq!(q.len(), 1);
    let _ = Duration::from_millis(10);
}

#[test]
fn ut32_offline_queue_full_rejects() {
    // Build a queue with capacity 2; persist 2; the 3rd must fail.
    let dir = TempDir::new().expect("tempdir");
    let q = OfflineQueue::with_options(dir.path(), 2, Duration::from_secs(60)).expect("open");
    q.persist(&make_envelope("a")).expect("a");
    q.persist(&make_envelope("b")).expect("b");
    assert!(q.is_full());

    let res = q.persist(&make_envelope("c"));
    assert!(res.is_err(), "capacity 2 should reject the 3rd persist");
    let arg_err = OfflineQueue::into_arg_error(&res.unwrap_err());
    assert!(
        arg_err.is_some(),
        "must be mappable to ARGError::OfflineQueueFull"
    );
}

#[test]
fn ut32b_offline_queue_prune_expired() {
    // Persist 2 envelopes; their `enqueued_at` is `Utc::now()` so they
    // are not expired; prune should return 0.
    let q = OfflineQueue::in_memory().expect("in-memory queue");
    q.persist(&make_envelope("x")).expect("x");
    q.persist(&make_envelope("y")).expect("y");
    let pruned = q.prune_expired().expect("prune");
    assert_eq!(pruned, 0);
    assert_eq!(q.len(), 2);
}
