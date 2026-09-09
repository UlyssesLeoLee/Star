// SPDX-License-Identifier: MIT OR Apache-2.0
//! `MemgraphEventListener` — Bolt subscription → in-process broadcast.
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §4.10
//! + [`docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md`](../../docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) §3.
//!
//! The listener owns:
//! - a shared `MemgraphClient` (per ARG.1 G-1 stub)
//! - a `tokio::sync::broadcast::Sender<BridgeEnvelope>` for fanout to the
//!   in-process consumers (LangGraph updater, period flush worker).
//!
//! The current implementation accepts the ARG.1 `MemgraphClient` stub
//! (G-1) and synthesises envelopes from the in-memory edge rows. The real
//! Bolt `SubscriptionRun` integration lands in the P3-C W1 follow-up
//! (G-1 closure).

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, warn};
use uuid::Uuid;

use star_arg::client::MemgraphClient;
use star_arg::models::event::ARGEvent;

use crate::error::BridgeError;
use crate::protocol::{ArgEdgeChanged, BridgeEnvelope, BridgeEnvelopeKind};

/// Default broadcast channel capacity (per DD §4.10 — 256 laggers).
pub const DEFAULT_CHANNEL_CAPACITY: usize = 256;

/// Bolt subscription listener that converts Memgraph `edge.changed`
/// events into [`BridgeEnvelope`]s and broadcasts them in-process.
///
/// Construction is infallible: the broadcast channel is created with the
/// configured capacity, and the [`MemgraphClient`] is the ARG.1
/// placeholder. Use [`Self::start`] to open the Bolt subscription
/// (currently a no-op against the G-1 stub).
#[derive(Debug)]
pub struct MemgraphEventListener {
    /// Shared Memgraph client (per ARG.1 G-1 stub).
    client: Arc<MemgraphClient>,
    /// Broadcast channel sender.
    event_tx: broadcast::Sender<BridgeEnvelope>,
    /// Per-listener epoch counter.
    epoch: std::sync::atomic::AtomicU64,
    /// Configured channel capacity (recorded at construction; the
    /// underlying `tokio::sync::broadcast::Sender` does not expose it).
    capacity: usize,
}

impl MemgraphEventListener {
    /// Build a new listener with the default broadcast capacity.
    pub fn new(client: Arc<MemgraphClient>) -> Self {
        Self::with_capacity(client, DEFAULT_CHANNEL_CAPACITY)
    }

    /// Build a new listener with a custom broadcast capacity.
    pub fn with_capacity(client: Arc<MemgraphClient>, capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let (event_tx, _) = broadcast::channel(capacity);
        Self {
            client,
            event_tx,
            epoch: std::sync::atomic::AtomicU64::new(0),
            capacity,
        }
    }

    /// Open the Bolt subscription and start fanning out envelopes.
    ///
    /// **Stub**: the G-1 `MemgraphClient` does not implement real
    /// `SubscriptionRun` yet, so this method returns `Ok(())` after
    /// logging the intent. The real path lands with `G-1` in P3-C W1.
    pub async fn start(&self) -> Result<(), BridgeError> {
        // Per DD §4.10 F-9 fix: clone the sender first, never move `self`
        // into the closure.
        let tx = self.event_tx.clone();
        let epoch = self
            .epoch
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!(epoch, "MemgraphEventListener::start (G-1 stub path)");
        // Real Bolt subscription would look like:
        //   self.client
        //       .subscribe("MATCH (a)-[r]->(b) WHERE r.updated_at > $last_seen RETURN r", move |row| {
        //           let event = ARGEvent::from_edge_row(&row);
        //           let _ = tx.send(event);
        //       })
        //       .await?;
        // For now we just confirm the channel can deliver.
        let health = self.client.health_check().await.unwrap_or(false);
        if !health {
            warn!("Memgraph health_check returned false (G-1 stub path, expected)");
        }
        let _ = (tx, epoch);
        Ok(())
    }

    /// Subscribe to envelopes from the broadcast channel.
    pub fn subscribe(&self) -> broadcast::Receiver<BridgeEnvelope> {
        self.event_tx.subscribe()
    }

    /// In-process producer hook used by the in-memory tests and by the
    /// `BridgeTier` when the real Bolt subscription is unavailable.
    ///
    /// Each call consumes one epoch and pushes a fresh envelope onto the
    /// broadcast channel. The `tokio::sync::broadcast::Sender::send` API
    /// returns an error when no receivers are registered; for the bridge
    /// tier this is the expected "fire-and-forget" mode, so we downgrade
    /// it to a debug log and return the envelope so callers can still
    /// observe what was produced.
    pub fn publish(&self, event: &ARGEvent) -> Result<BridgeEnvelope, BridgeError> {
        let envelope = self.build_envelope(event)?;
        if let Err(e) = self.event_tx.send(envelope.clone()) {
            debug!(error = %e, "broadcast send: no active receivers (fire-and-forget)");
        }
        Ok(envelope)
    }

    /// Convert an [`ARGEvent`] (from the ARG.1 Data Tier) into a
    /// [`BridgeEnvelope`].
    pub fn build_envelope(&self, event: &ARGEvent) -> Result<BridgeEnvelope, BridgeError> {
        let epoch = self
            .epoch
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let kind = match event {
            ARGEvent::EdgeCreated(e) => BridgeEnvelopeKind::EdgeChanged(ArgEdgeChanged {
                edge_id: e.id,
                from_agent_id: e.from_agent,
                to_agent_id: e.to_agent,
                relationship_type: e.edge_type.cypher_label().to_string(),
                weight: f64::from(e.weight),
                created_at: e.created_at,
                trigger_sa: "SA_01".to_string(),
            }),
            ARGEvent::EdgeUpdated { id, after, .. } => {
                BridgeEnvelopeKind::EdgeChanged(ArgEdgeChanged {
                    edge_id: *id,
                    from_agent_id: after.from_agent,
                    to_agent_id: after.to_agent,
                    relationship_type: after.edge_type.cypher_label().to_string(),
                    weight: f64::from(after.weight),
                    created_at: after.updated_at,
                    trigger_sa: "SA_01".to_string(),
                })
            }
            ARGEvent::EdgeArchived { id } => BridgeEnvelopeKind::EdgeChanged(ArgEdgeChanged {
                edge_id: *id,
                from_agent_id: Uuid::nil(),
                to_agent_id: Uuid::nil(),
                relationship_type: "ARCHIVED".to_string(),
                weight: 0.0,
                created_at: chrono::Utc::now(),
                trigger_sa: "SA_01".to_string(),
            }),
            ARGEvent::TrustScoreChanged {
                agent_id,
                before,
                after,
                delta,
            } => BridgeEnvelopeKind::TrustScoreUpdate(crate::protocol::ArgTrustScoreUpdate {
                agent_id: *agent_id,
                old_score: f64::from(*before),
                new_score: f64::from(*after),
                delta_reason: if *delta >= 0.0 {
                    "success".to_string()
                } else {
                    "failure".to_string()
                },
                trigger_sa: "SA_03".to_string(),
                created_at: chrono::Utc::now(),
            }),
            // The remaining 5 variants are not yet part of the 5 protocols;
            // route them through the offline queue (per arch §2.1) by
            // surfacing them as `InternalError` so the caller can persist
            // them in the sled-backed queue. ARG.3 will tighten the mapping.
            other => {
                return Err(BridgeError::InternalError(format!(
                    "ARGEvent::{} has no BridgeEnvelopeKind mapping in ARG.2",
                    other.kind()
                )))
            }
        };
        Ok(BridgeEnvelope::new(Uuid::new_v4(), kind, epoch))
    }

    /// Returns the current broadcast channel capacity.
    pub fn channel_capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current number of active receivers.
    pub fn receiver_count(&self) -> usize {
        self.event_tx.receiver_count()
    }
}
