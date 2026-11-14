// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=24710f693bc89119c080946a624014cdb316096f94044b3b49d2945bc8c6389e
MERGE (self:File {path:"crates/star-api-rest/src/device_outbox/relay.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/device_outbox/relay.rs::module",kind:"module"})
MERGE (backoff:Type {id:"crates/star-api-rest/src/device_outbox/relay.rs::RelayBackoffPolicy",kind:"struct"})
MERGE (relay:Type {id:"crates/star-api-rest/src/device_outbox/relay.rs::BoundedRelay",kind:"struct"})
MERGE (profileSwitch:Type {id:"crates/star-api-rest/src/device_outbox/relay.rs::ProfileSwitchOutcome",kind:"enum"})
MERGE (delay:Symbol {id:"crates/star-api-rest/src/device_outbox/relay.rs::RelayBackoffPolicy.delay_for",kind:"method"})
MERGE (drain:Symbol {id:"crates/star-api-rest/src/device_outbox/relay.rs::BoundedRelay.drain_before_switch",kind:"method"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(backoff),(module)-[:DEFINES]->(relay),(module)-[:DEFINES]->(profileSwitch)
MERGE (relay)-[:HAS_METHOD]->(drain),(backoff)-[:HAS_METHOD]->(delay)
@endcypher
*/
//! Bounded at-least-once relay discipline (review 合约 #7):
//! - transient failures retry with bounded exponential backoff;
//! - pending facts are never dropped by cancellation: workers cancel only
//!   between delivery transactions and resume from the durable cursor;
//! - switching transport profile requires draining (or an explicit migration)
//!   of the old profile's pending records — a new profile's cursor is never
//!   used to interpret the old profile's state.

use std::time::Duration;

use super::collaboration_inbox::ApplyOutcome;

/// Bounded exponential backoff: the delay doubles per consecutive failure but
/// is capped, so a permanently failing sink cannot stall the relay forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayBackoffPolicy {
    pub base: Duration,
    pub max: Duration,
}

impl Default for RelayBackoffPolicy {
    fn default() -> Self {
        Self {
            base: Duration::from_secs(1),
            max: Duration::from_secs(30),
        }
    }
}

impl RelayBackoffPolicy {
    /// Delay after `consecutive_failures` failures (1-based), bounded by
    /// [`Self::max`]. Review 合约 #7: bounded backoff, pending facts retained.
    pub fn delay_for(&self, consecutive_failures: u32) -> Duration {
        let failures = consecutive_failures.max(1);
        let factor = 1_u32 << (failures - 1).min(10); // cap the shift, not just the result
        self.base.saturating_mul(factor).min(self.max)
    }
}

/// Pending delivery bookkeeping for one stream on one transport profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingDelivery {
    pub stream_id: uuid::Uuid,
    pub transport_profile_id: uuid::Uuid,
    /// Oldest undelivered source sequence (inclusive).
    pub from_sequence: i64,
    /// Newest pending source sequence (inclusive).
    pub to_sequence: i64,
}

impl PendingDelivery {
    pub fn pending_count(&self) -> i64 {
        (self.to_sequence - self.from_sequence + 1).max(0)
    }
}

/// Result of asking the relay to switch the transport profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSwitchOutcome {
    /// No pending records for the old profile; switching is safe.
    Drained,
    /// Pending records remain; the switch is refused until they are drained
    /// or an explicit migration is performed (review 合约 #7).
    Refused { pending: i64 },
}

/// Relay-side state machine kept deliberately synchronous so cancellation can
/// only ever happen between delivery transactions (review 合约 #7).
#[derive(Debug, Default)]
pub struct BoundedRelay {
    consecutive_failures: u32,
    pending: Vec<PendingDelivery>,
}

impl BoundedRelay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the durable pending window for a stream/profile pair. After a
    /// crash the pending set is rebuilt from the durable outbox + checkpoint,
    /// never from memory.
    pub fn register_pending(&mut self, delivery: PendingDelivery) {
        if delivery.pending_count() == 0 {
            return;
        }
        if let Some(existing) = self.pending.iter_mut().find(|pending| {
            pending.stream_id == delivery.stream_id
                && pending.transport_profile_id == delivery.transport_profile_id
        }) {
            existing.from_sequence = existing.from_sequence.min(delivery.from_sequence);
            existing.to_sequence = existing.to_sequence.max(delivery.to_sequence);
        } else {
            self.pending.push(delivery);
        }
    }

    /// Mark one delivery applied: the durable commit already advanced the
    /// receiver checkpoint, so only the in-memory window shrinks here. On
    /// success the failure counter resets (bounded backoff).
    pub fn record_outcome(&mut self, outcome: &ApplyOutcome) {
        match outcome {
            ApplyOutcome::Committed { .. } | ApplyOutcome::ExactReplay { .. } => {
                self.consecutive_failures = 0;
            }
            ApplyOutcome::DigestConflict
            | ApplyOutcome::SequenceGap { .. }
            | ApplyOutcome::StaleCursor { .. }
            | ApplyOutcome::Suspended { .. }
            | ApplyOutcome::Rejected => {
                // Integrity outcomes are not retried by blind redelivery; they
                // require reconciliation or quarantine first (review 合约 #7).
                self.consecutive_failures = 0;
            }
        }
    }

    /// Record a transient failure (connection loss, timeout): the fact stays
    /// pending and the next attempt is delayed with bounded backoff.
    pub fn record_transient_failure(&mut self) -> Duration {
        self.consecutive_failures += 1;
        RelayBackoffPolicy::default().delay_for(self.consecutive_failures)
    }

    /// Advance the pending window past a delivered sequence.
    pub fn ack_sequence(&mut self, stream_id: uuid::Uuid, profile_id: uuid::Uuid, sequence: i64) {
        for pending in &mut self.pending {
            if pending.stream_id == stream_id && pending.transport_profile_id == profile_id {
                pending.from_sequence = pending.from_sequence.max(sequence + 1);
            }
        }
        self.pending.retain(|pending| pending.pending_count() > 0);
    }

    /// Check whether a profile switch is allowed: the old profile must have no
    /// pending records, or the caller must perform an explicit migration —
    /// the relay never silently discards pending facts (review 合约 #7).
    pub fn pending_for(&self, stream_id: uuid::Uuid, transport_profile_id: uuid::Uuid) -> i64 {
        self.pending
            .iter()
            .filter(|pending| {
                pending.stream_id == stream_id
                    && pending.transport_profile_id == transport_profile_id
            })
            .map(PendingDelivery::pending_count)
            .sum()
    }

    pub fn drain_before_switch(
        &self,
        stream_id: uuid::Uuid,
        old_profile_id: uuid::Uuid,
    ) -> ProfileSwitchOutcome {
        let pending = self.pending_for(stream_id, old_profile_id);
        if pending == 0 {
            ProfileSwitchOutcome::Drained
        } else {
            ProfileSwitchOutcome::Refused { pending }
        }
    }

    /// Cursors are opaque and profile-bound (review 合约 #5/#7): the new
    /// profile must never reuse or interpret the old profile's cursor.
    pub fn new_profile_needs_fresh_cursor(
        &self,
        old_profile_id: uuid::Uuid,
        new_profile_id: uuid::Uuid,
    ) -> bool {
        old_profile_id != new_profile_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn stream() -> Uuid {
        Uuid::from_u128(0xF1)
    }

    fn profile(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn backoff_is_bounded_even_after_many_failures() {
        let policy = RelayBackoffPolicy::default();
        assert_eq!(policy.delay_for(0), policy.base);
        assert_eq!(policy.delay_for(1), Duration::from_secs(1));
        assert_eq!(policy.delay_for(2), Duration::from_secs(2));
        assert_eq!(policy.delay_for(5), Duration::from_secs(16));
        assert_eq!(policy.delay_for(100), policy.max);
        assert_eq!(policy.delay_for(u32::MAX), policy.max);
    }

    #[test]
    fn transient_failure_keeps_fact_pending_and_backs_off() {
        let mut relay = BoundedRelay::new();
        relay.register_pending(PendingDelivery {
            stream_id: stream(),
            transport_profile_id: profile(0xF2),
            from_sequence: 7,
            to_sequence: 9,
        });
        let delay = relay.record_transient_failure();
        assert!(delay >= Duration::from_secs(1));
        // Cancellation/restart between attempts must not drop the fact:
        assert_eq!(relay.pending_for(stream(), profile(0xF2)), 3);
    }

    #[test]
    fn profile_switch_is_refused_while_pending_records_remain() {
        let mut relay = BoundedRelay::new();
        relay.register_pending(PendingDelivery {
            stream_id: stream(),
            transport_profile_id: profile(0xF3),
            from_sequence: 4,
            to_sequence: 6,
        });
        assert_eq!(
            relay.drain_before_switch(stream(), profile(0xF3)),
            ProfileSwitchOutcome::Refused { pending: 3 }
        );
        relay.ack_sequence(stream(), profile(0xF3), 4);
        relay.ack_sequence(stream(), profile(0xF3), 5);
        relay.ack_sequence(stream(), profile(0xF3), 6);
        assert_eq!(
            relay.drain_before_switch(stream(), profile(0xF3)),
            ProfileSwitchOutcome::Drained
        );
    }

    #[test]
    fn new_profile_never_reuses_old_profile_cursor() {
        let relay = BoundedRelay::new();
        assert!(relay.new_profile_needs_fresh_cursor(profile(0xF4), profile(0xF5)));
        assert!(!relay.new_profile_needs_fresh_cursor(profile(0xF4), profile(0xF4)));
    }

    #[test]
    fn integrity_outcomes_do_not_trigger_blind_redelivery_backoff() {
        let mut relay = BoundedRelay::new();
        relay.record_outcome(&ApplyOutcome::DigestConflict);
        relay.record_outcome(&ApplyOutcome::SequenceGap {
            next_expected_sequence: 3,
        });
        relay.record_outcome(&ApplyOutcome::Rejected);
        assert_eq!(
            relay.consecutive_failures, 0,
            "integrity outcomes require reconciliation, not retries"
        );
        // A committed apply resets transient failure counters too.
        relay.record_transient_failure();
        relay.record_transient_failure();
        assert_eq!(relay.consecutive_failures, 2);
        relay.record_outcome(&ApplyOutcome::Committed {
            receipt_id: Uuid::new_v4(),
            committed_at: chrono::Utc::now(),
            next_expected_sequence: 5,
        });
        assert_eq!(relay.consecutive_failures, 0);
    }

    #[test]
    fn pending_windows_merge_without_duplicate_delivery() {
        let mut relay = BoundedRelay::new();
        relay.register_pending(PendingDelivery {
            stream_id: stream(),
            transport_profile_id: profile(0xF6),
            from_sequence: 1,
            to_sequence: 3,
        });
        relay.register_pending(PendingDelivery {
            stream_id: stream(),
            transport_profile_id: profile(0xF6),
            from_sequence: 3,
            to_sequence: 5,
        });
        assert_eq!(relay.pending_for(stream(), profile(0xF6)), 5);
        relay.ack_sequence(stream(), profile(0xF6), 4);
        assert_eq!(relay.pending_for(stream(), profile(0xF6)), 1);
    }
}
