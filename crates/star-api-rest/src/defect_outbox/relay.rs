// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/src/defect_outbox/relay.rs"})
MERGE (backoff:Type {id:"crates/star-api-rest/src/defect_outbox/relay.rs::RelayBackoffPolicy",kind:"struct"})
MERGE (relay:Type {id:"crates/star-api-rest/src/defect_outbox/relay.rs::BoundedRelay",kind:"struct"})
MERGE (profileSwitch:Type {id:"crates/star-api-rest/src/defect_outbox/relay.rs::ProfileSwitchOutcome",kind:"enum"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(backoff),(module)-[:DEFINES]->(relay),(module)-[:DEFINES]->(profileSwitch)
@cypher
*/
//! Bounded at-least-once relay for Defect → Notification outbox delivery.
//!
//! Based on `device_outbox::relay` pattern, adapted for Defect → Notification.
//!
//! - Transient failures retry with bounded exponential backoff
//! - Pending facts never dropped by cancellation: workers cancel only
//!   between delivery transactions and resume from the durable cursor
//! - Switching transport profile requires draining the old profile's
//!   pending records (review 合约 #7)

use std::time::Duration;
use uuid::Uuid;

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
    /// [`Self::max`].
    pub fn delay_for(&self, consecutive_failures: u32) -> Duration {
        let failures = consecutive_failures.max(1);
        let factor = 1_u32 << (failures - 1).min(10);
        self.base.saturating_mul(factor).min(self.max)
    }
}

/// Pending delivery bookkeeping for one stream on one transport profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingDelivery {
    pub stream_id: Uuid,
    pub transport_profile_id: Uuid,
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
    /// or an explicit migration is performed.
    Refused { pending: i64 },
}

/// Relay-side state machine kept deliberately synchronous so cancellation can
/// only ever happen between delivery transactions.
#[derive(Debug, Default)]
pub struct BoundedRelay {
    consecutive_failures: u32,
    pending: Vec<PendingDelivery>,
}

impl BoundedRelay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the durable pending window for a stream/profile pair.
    pub fn register_pending(&mut self, delivery: PendingDelivery) {
        if delivery.pending_count() == 0 {
            return;
        }
        self.pending.push(delivery);
    }

    /// Record a delivery attempt result.
    pub fn record_result(&mut self, success: bool) {
        if success {
            self.consecutive_failures = 0;
        } else {
            self.consecutive_failures += 1;
        }
    }

    /// Get the current backoff delay.
    pub fn current_delay(&self) -> Duration {
        let policy = RelayBackoffPolicy::default();
        policy.delay_for(self.consecutive_failures)
    }

    /// Check if a profile switch is safe (no pending deliveries for old profile).
    pub fn can_switch_profile(&self, old_profile_id: uuid::Uuid) -> ProfileSwitchOutcome {
        let pending = self.pending.iter()
            .filter(|d| d.transport_profile_id == old_profile_id)
            .map(|d| d.pending_count())
            .sum::<i64>();
        if pending == 0 {
            ProfileSwitchOutcome::Drained
        } else {
            ProfileSwitchOutcome::Refused { pending }
        }
    }
}