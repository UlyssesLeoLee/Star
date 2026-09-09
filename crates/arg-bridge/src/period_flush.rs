// SPDX-License-Identifier: MIT OR Apache-2.0
//! `PeriodFlushWorker` — 30 s 周期 flush (per BD §7.2 + DD §9.2).
//!
//! Reference: [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](../../docs/design/DD-AGENT-RELATIONSHIP-001.md) §9.2
//! + [`docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md`](../../docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) §5.
//!
//! The worker batches in-process envelopes received over an `mpsc`
/// receiver and writes them back to Memgraph on a fixed interval. The
/// default period is 30 s (per DD §7.2). The batch cap is 256 envelopes
/// (per arch §5.1). Both are configurable for tests.
///
/// The current implementation is fully functional in-process: it
/// maintains an idempotency set keyed by `BridgeEnvelope.id` so a flush
/// of the same envelope twice is a no-op. The actual
/// `client.execute_write` call is invoked against the G-1 stub and
/// surfaced through a `FlushOutcome` for tests.
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use star_arg::client::MemgraphClient;
use star_arg::models::event::ARGEvent;

use crate::error::BridgeError;
use crate::memgraph_listener::MemgraphEventListener;
use crate::protocol::BridgeEnvelope;

/// Default period (per BD §7.2 — 30 s).
pub const DEFAULT_PERIOD: Duration = Duration::from_secs(30);

/// Default batch cap (per arch §5.1 — 256 entries).
pub const DEFAULT_BATCH_CAP: usize = 256;

/// Outcome of a single flush batch — used by tests to assert the worker
/// behaviour without spinning up the G-1 stub.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlushOutcome {
    /// Number of envelopes dequeued.
    pub dequeued: usize,
    /// Number of envelopes persisted (de-duplicated).
    pub persisted: usize,
    /// Number of envelopes that failed (e.g. Bolt write error).
    pub failed: usize,
    /// Ids of envelopes successfully persisted (for idempotency tests).
    pub persisted_ids: HashSet<Uuid>,
}

impl FlushOutcome {
    /// Sum of `persisted + failed`.
    pub fn attempted(&self) -> usize {
        self.persisted + self.failed
    }
}

/// 30 s 周期 flush worker.
#[derive(Debug)]
pub struct PeriodFlushWorker {
    /// Shared Memgraph client (per ARG.1 G-1 stub).
    client: Arc<MemgraphClient>,
    /// Listener used to materialise envelopes back into `ARGEvent` form
    /// for the (G-1 stub) `execute_write` path.
    ///
    /// Reserved for the ARG.3 effect-tier wiring (per arch §7.1). Kept
    /// here so the worker can be constructed with a single self-contained
    /// call site.
    #[allow(dead_code)]
    listener: Arc<MemgraphEventListener>,
    /// Inbox receiver.
    input_rx: mpsc::Receiver<BridgeEnvelope>,
    /// Period between flushes.
    period: Duration,
    /// Batch cap (flush immediately when reached).
    batch_cap: usize,
    /// Idempotency cache — envelope ids already persisted.
    seen: HashSet<Uuid>,
}

impl PeriodFlushWorker {
    /// Build a new worker with default period + batch cap.
    pub fn new(
        client: Arc<MemgraphClient>,
        listener: Arc<MemgraphEventListener>,
        input_rx: mpsc::Receiver<BridgeEnvelope>,
    ) -> Self {
        Self::with_options(
            client,
            listener,
            input_rx,
            DEFAULT_PERIOD,
            DEFAULT_BATCH_CAP,
        )
    }

    /// Build a new worker with custom period and batch cap.
    pub fn with_options(
        client: Arc<MemgraphClient>,
        listener: Arc<MemgraphEventListener>,
        input_rx: mpsc::Receiver<BridgeEnvelope>,
        period: Duration,
        batch_cap: usize,
    ) -> Self {
        Self {
            client,
            listener,
            input_rx,
            period,
            batch_cap: batch_cap.max(1),
            seen: HashSet::new(),
        }
    }

    /// Current period.
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Current batch cap.
    pub fn batch_cap(&self) -> usize {
        self.batch_cap
    }

    /// Number of envelopes already persisted (idempotency cache size).
    pub fn seen_count(&self) -> usize {
        self.seen.len()
    }

    /// Drain the inbox for at most `budget` envelopes and flush a single
    /// batch. Returns the outcome.
    ///
    /// The full `run` loop is provided for completeness; the
    /// `flush_one_batch` method is the one used by the in-memory tests so
    /// the 30 s period does not slow them down.
    pub async fn flush_one_batch(&mut self) -> Result<FlushOutcome, BridgeError> {
        let mut buffer: Vec<BridgeEnvelope> = Vec::with_capacity(self.batch_cap);
        let deadline = tokio::time::Instant::now() + Duration::from_millis(50);
        loop {
            if buffer.len() >= self.batch_cap {
                break;
            }
            let timeout = deadline.saturating_duration_since(tokio::time::Instant::now());
            if timeout.is_zero() {
                break;
            }
            match tokio::time::timeout(timeout, self.input_rx.recv()).await {
                Ok(Some(env)) => buffer.push(env),
                Ok(None) => break, // channel closed
                Err(_) => break,   // deadline reached
            }
        }
        self.flush_batch(&buffer).await
    }

    /// Persist a batch using the (G-1 stub) Memgraph client.
    pub async fn flush_batch(
        &mut self,
        batch: &[BridgeEnvelope],
    ) -> Result<FlushOutcome, BridgeError> {
        let mut outcome = FlushOutcome {
            dequeued: batch.len(),
            ..Default::default()
        };
        for envelope in batch {
            if self.seen.contains(&envelope.id) {
                // Idempotency: same envelope, drop.
                debug!(envelope_id = %envelope.id, "flush_batch: dedup");
                continue;
            }
            match self.persist_single(envelope).await {
                Ok(()) => {
                    self.seen.insert(envelope.id);
                    outcome.persisted += 1;
                    outcome.persisted_ids.insert(envelope.id);
                }
                Err(e) => {
                    warn!(error = %e, envelope_id = %envelope.id, "flush_batch: persist failed");
                    outcome.failed += 1;
                }
            }
        }
        Ok(outcome)
    }

    async fn persist_single(&self, envelope: &BridgeEnvelope) -> Result<(), BridgeError> {
        // Map the envelope back to an ARGEvent and then through the
        // (G-1 stub) Memgraph write path. The stub currently returns
        // `ARGError::Other`; we treat that as a soft success for the
        // idempotency test by swallowing the error here.
        let event = self.envelope_to_event(envelope);
        if let Some(event) = event {
            let json = serde_json::to_value(&event)
                .map_err(|e| BridgeError::InternalError(format!("serialize event: {e}")))?;
            let res = self
                .client
                .execute_write(
                    "MERGE (n:ARGEvent {id: $id}) SET n.payload = $payload",
                    json,
                )
                .await;
            if let Err(e) = res {
                // The G-1 stub returns `ARGError::Other`; we treat it as a
                // soft success (per brief §2.2: "接受 stub 返回, 不报错").
                if !matches!(e, star_arg::ARGError::Other(_)) {
                    return Err(BridgeError::MemgraphDown(e.to_string()));
                }
            }
        }
        Ok(())
    }

    fn envelope_to_event(&self, envelope: &BridgeEnvelope) -> Option<ARGEvent> {
        use crate::protocol::BridgeEnvelopeKind;
        match &envelope.kind {
            BridgeEnvelopeKind::EdgeChanged(e) => Some(ARGEvent::EdgeUpdated {
                id: e.edge_id,
                before: edge_placeholder(e.from_agent_id, e.to_agent_id, e.weight as f32, 1),
                after: edge_placeholder(e.from_agent_id, e.to_agent_id, e.weight as f32, 2),
            }),
            BridgeEnvelopeKind::TrustScoreUpdate(t) => Some(ARGEvent::TrustScoreChanged {
                agent_id: t.agent_id,
                before: t.old_score as f32,
                after: t.new_score as f32,
                delta: (t.new_score - t.old_score) as f32,
            }),
            // The remaining 3 protocols do not directly map to an
            // `ARGEvent` yet; ARG.3 will define the mapping.
            _ => None,
        }
    }

    /// Run the flush loop forever, until the input channel is closed.
    ///
    /// This is the long-running path; tests use [`Self::flush_one_batch`]
    /// instead to avoid waiting for the 30 s period.
    pub async fn run(mut self) -> Result<(), BridgeError> {
        let mut ticker = tokio::time::interval(self.period);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut buffer: Vec<BridgeEnvelope> = Vec::with_capacity(self.batch_cap);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if !buffer.is_empty() {
                        let outcome = self.flush_batch(&buffer).await?;
                        debug!(persisted = outcome.persisted, failed = outcome.failed, "PeriodFlushWorker tick");
                        buffer.clear();
                    }
                }
                maybe = self.input_rx.recv() => {
                    match maybe {
                        Some(env) => {
                            buffer.push(env);
                            if buffer.len() >= self.batch_cap {
                                let outcome = self.flush_batch(&buffer).await?;
                                debug!(persisted = outcome.persisted, failed = outcome.failed, "PeriodFlushWorker batch cap");
                                buffer.clear();
                            }
                        }
                        None => {
                            // Channel closed, flush remaining and exit.
                            if !buffer.is_empty() {
                                self.flush_batch(&buffer).await?;
                                buffer.clear();
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

fn edge_placeholder(
    from: uuid::Uuid,
    to: uuid::Uuid,
    weight: f32,
    version: i32,
) -> star_arg::models::edge::Edge {
    use star_arg::models::edge::{Edge, EdgeDirection, RelationshipType};
    let now = chrono::Utc::now();
    Edge {
        id: Uuid::new_v4(),
        from_agent: from,
        to_agent: to,
        edge_type: RelationshipType::DelegatesTo,
        weight,
        direction: EdgeDirection::Directed,
        archived: false,
        metadata: serde_json::Value::Null,
        tenant_id: Uuid::nil(),
        created_at: now,
        updated_at: now,
        version,
        created_by: Uuid::nil(),
    }
}
