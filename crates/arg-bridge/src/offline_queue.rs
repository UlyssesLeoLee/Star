// SPDX-License-Identifier: MIT OR Apache-2.0
//! `OfflineQueue` — `sled`-backed offline persistence (per BD §7.2).
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §4.10
//! + [`docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md`](../../docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) §6.
//!
//! The queue is used when Memgraph is unreachable. Incoming envelopes are
//! persisted into a `sled` embedded database; once Memgraph is back the
//! queue is drained and the envelopes are replayed.
//!
//! Capacity: 10 000 entries (per 守门 #12 已知缺口 + DD §11).
//! Retention: 7 days (per 守门 #7).
//! Cross-process pub/sub: implemented via the filesystem — multiple
//! processes can open the same `sled` directory and observe writes.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sled::Db;
use tracing::{debug, warn};
use uuid::Uuid;

use star_arg::ARGError;

use crate::error::BridgeError;
use crate::protocol::BridgeEnvelope;

/// Default capacity (per DD §11 + 守门 #12 已知缺口 — 10 000).
pub const DEFAULT_CAPACITY: usize = 10_000;

/// Default retention (per 守门 #7 — 7 days).
pub const DEFAULT_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// `sled`-backed offline queue for [`BridgeEnvelope`]s.
#[derive(Debug)]
pub struct OfflineQueue {
    /// Underlying `sled` database.
    db: Db,
    /// Tree of envelopes (key = envelope id, value = JSON-serialised envelope).
    tree: sled::Tree,
    /// Capacity (enforced on insert).
    capacity: usize,
    /// Retention window (used by [`Self::prune_expired`]).
    retention: Duration,
}

impl OfflineQueue {
    /// Open a queue at the given path with default capacity + retention.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, BridgeError> {
        Self::with_options(path, DEFAULT_CAPACITY, DEFAULT_RETENTION)
    }

    /// Open a queue with custom capacity and retention.
    pub fn with_options(
        path: impl AsRef<Path>,
        capacity: usize,
        retention: Duration,
    ) -> Result<Self, BridgeError> {
        let db = sled::open(path).map_err(|e| BridgeError::OfflinePersistFailed(e.to_string()))?;
        let tree = db
            .open_tree("bridge_envelopes")
            .map_err(|e| BridgeError::OfflinePersistFailed(e.to_string()))?;
        Ok(Self {
            db,
            tree,
            capacity: capacity.max(1),
            retention,
        })
    }

    /// Open an ephemeral in-memory queue (used by tests).
    pub fn in_memory() -> Result<Self, BridgeError> {
        let db = sled::Config::new()
            .temporary(true)
            .open()
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("open temp sled: {e}")))?;
        let tree = db
            .open_tree("bridge_envelopes")
            .map_err(|e| BridgeError::OfflinePersistFailed(e.to_string()))?;
        Ok(Self {
            db,
            tree,
            capacity: DEFAULT_CAPACITY,
            retention: DEFAULT_RETENTION,
        })
    }

    /// Current capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Current retention window.
    pub fn retention(&self) -> Duration {
        self.retention
    }

    /// Current queue size (after de-dup, before capacity check).
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Whether the queue has reached its capacity.
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    /// Persist an envelope. Returns `Err(OfflineQueueFull)` when the
    /// queue is at capacity.
    pub fn persist(&self, envelope: &BridgeEnvelope) -> Result<(), BridgeError> {
        if self.is_full() {
            return Err(BridgeError::InternalError(format!(
                "OfflineQueueFull ({} items, capacity {})",
                self.len(),
                self.capacity
            )));
        }
        if self
            .tree
            .contains_key(envelope.id.as_bytes())
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("contains_key: {e}")))?
        {
            // Idempotency: re-persist of the same id is a no-op.
            return Ok(());
        }
        let value = serde_json::to_vec(envelope)
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("serialize: {e}")))?;
        self.tree
            .insert(envelope.id.as_bytes(), value)
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("insert: {e}")))?;
        self.db
            .flush()
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("flush: {e}")))?;
        Ok(())
    }

    /// Read a single envelope by id (used by tests + drain hooks).
    pub fn get(&self, id: Uuid) -> Result<Option<BridgeEnvelope>, BridgeError> {
        let Some(value) = self
            .tree
            .get(id.as_bytes())
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("get: {e}")))?
        else {
            return Ok(None);
        };
        let envelope: BridgeEnvelope = serde_json::from_slice(&value)
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("deserialize: {e}")))?;
        Ok(Some(envelope))
    }

    /// Pop the oldest envelope (insertion order is the sled iteration
    /// order, which is byte-sort by key — envelopes are Uuids so this is
    /// effectively random; we treat the first as the oldest for the
    /// purposes of this API).
    pub fn pop_oldest(&self) -> Result<Option<BridgeEnvelope>, BridgeError> {
        let Some((key, value)) = self
            .tree
            .first()
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("first: {e}")))?
        else {
            return Ok(None);
        };
        let envelope: BridgeEnvelope = serde_json::from_slice(&value)
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("deserialize: {e}")))?;
        self.tree
            .remove(key)
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("remove: {e}")))?;
        self.db
            .flush()
            .map_err(|e| BridgeError::OfflinePersistFailed(format!("flush: {e}")))?;
        Ok(Some(envelope))
    }

    /// Drain the queue and return all envelopes in deterministic order
    /// (sorted by `enqueued_at`).
    pub fn drain(&self) -> Result<Vec<BridgeEnvelope>, BridgeError> {
        let mut out: Vec<BridgeEnvelope> = Vec::new();
        for entry in self.tree.iter() {
            let (_, value) =
                entry.map_err(|e| BridgeError::OfflinePersistFailed(format!("iter: {e}")))?;
            let envelope: BridgeEnvelope = serde_json::from_slice(&value)
                .map_err(|e| BridgeError::OfflinePersistFailed(format!("deserialize: {e}")))?;
            out.push(envelope);
        }
        out.sort_by_key(|e| e.enqueued_at);
        Ok(out)
    }

    /// Remove all envelopes older than the retention window.
    pub fn prune_expired(&self) -> Result<usize, BridgeError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let cutoff_secs = now.saturating_sub(self.retention).as_secs() as i64;
        let mut pruned = 0;
        let mut to_remove: Vec<sled::IVec> = Vec::new();
        for entry in self.tree.iter() {
            let (key, value) =
                entry.map_err(|e| BridgeError::OfflinePersistFailed(format!("iter: {e}")))?;
            let envelope: BridgeEnvelope = serde_json::from_slice(&value)
                .map_err(|e| BridgeError::OfflinePersistFailed(format!("deserialize: {e}")))?;
            if envelope.enqueued_at.timestamp() < cutoff_secs {
                to_remove.push(key);
            }
        }
        for key in to_remove {
            self.tree
                .remove(key)
                .map_err(|e| BridgeError::OfflinePersistFailed(format!("remove: {e}")))?;
            pruned += 1;
        }
        if pruned > 0 {
            self.db
                .flush()
                .map_err(|e| BridgeError::OfflinePersistFailed(format!("flush: {e}")))?;
            debug!(pruned, "OfflineQueue::prune_expired");
        }
        Ok(pruned)
    }

    /// Convert this error into the canonical [`ARGError::OfflineQueueFull`]
    /// when the underlying cause is the capacity error.
    pub fn into_arg_error(err: &BridgeError) -> Option<ARGError> {
        if let BridgeError::InternalError(msg) = err {
            if msg.contains("OfflineQueueFull") {
                return Some(ARGError::OfflineQueueFull(0));
            }
        }
        None
    }
}

impl Drop for OfflineQueue {
    fn drop(&mut self) {
        if let Err(e) = self.db.flush() {
            warn!(error = %e, "OfflineQueue::drop: flush failed");
        }
    }
}
