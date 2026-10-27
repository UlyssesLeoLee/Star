// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=8d4b398cbf3c91e92e5aea18d030b57c05276cbb2f74918492a00f12e2a2f234
MERGE (self:File {path:"crates/star-api-rest/src/group_api/canvas_event_stream.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::module",kind:"module"})
MERGE (hub:Type {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub",kind:"struct"})
MERGE (signal:Type {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::WakeSignal",kind:"enum"})
MERGE (router:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::router",kind:"function"})
MERGE (start:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::CanvasEventWakeHub.start",kind:"method"})
MERGE (listener:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::run_listener",kind:"function"})
MERGE (stream:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::stream_worktree_canvas_events",kind:"function"})
MERGE (authorize:Symbol {id:"crates/star-api-rest/src/group_api/canvas.rs::reauthorize_worktree_canvas_read",kind:"function"})
MERGE (notify:Topic {id:"postgresql.star_canvas_outbox_wake",kind:"topic"})
MERGE (endpoint:Endpoint {id:"GET /api/v1/worktrees/{worktree_id}/canvas-events/stream",kind:"http_route"})
MERGE (connect:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::connect_listener",kind:"function"})
MERGE (reconnect:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::reconnect_listener",kind:"function"})
MERGE (filter:Symbol {id:"crates/star-api-rest/src/group_api/canvas_event_stream.rs::signal_for_subscriber",kind:"function"})
MERGE (apiState:Type {id:"crates/star-api-rest/src/group_api.rs::GroupApiState",kind:"struct"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(hub)
MERGE (module)-[:DEFINES]->(signal)
MERGE (module)-[:DEFINES]->(router)
MERGE (module)-[:DEFINES]->(listener)
MERGE (module)-[:DEFINES]->(stream)
MERGE (module)-[:DEFINES]->(connect)
MERGE (module)-[:DEFINES]->(reconnect)
MERGE (module)-[:DEFINES]->(filter)
MERGE (router)-[:EXPOSES]->(endpoint)
MERGE (endpoint)-[:HANDLES]->(stream)
MERGE (hub)-[:HAS_METHOD]->(start)
MERGE (start)-[:CALLS]->(connect)
MERGE (connect)-[:SUBSCRIBES]->(notify)
MERGE (listener)-[:CALLS]->(reconnect)
MERGE (listener)-[:SUBSCRIBES]->(notify)
MERGE (listener)-[:EMITS]->(signal)
MERGE (stream)-[:CALLS]->(authorize)
MERGE (stream)-[:CALLS]->(filter)
MERGE (router)-[:CALLS]->(stream)
MERGE (stream)-[:EMITS]->(signal)
MERGE (apiState)-[:USES_TYPE]->(hub)
@endcypher
*/
//! Worktree-wide bounded SSE wake stream. The durable Outbox API carries all event detail.

use std::{convert::Infallible, mem::size_of, sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderName, header::HeaderValue},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::get,
};
use futures_util::stream;
use sqlx::{PgPool, postgres::PgListener};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, broadcast};
use tracing::warn;
use uuid::Uuid;

use super::{
    AuthUser, AuthenticatedUser, GroupApiError, GroupApiState,
    canvas::reauthorize_worktree_canvas_read,
};

const NOTIFY_CHANNEL: &str = "star_canvas_outbox_wake";
const WAKE_QUEUE_CAPACITY: usize = 64;
const MAX_ACTIVE_STREAMS_PER_PROCESS: usize = 128;
const MAX_NOTIFY_PAYLOAD_BYTES: usize = 36;
const MAX_SSE_EVENT_FRAME_BYTES: usize = 30;

// These are resource ceilings, not measured latency/throughput guarantees.
const MAX_WAKE_QUEUE_BYTES: usize = WAKE_QUEUE_CAPACITY * size_of::<WakeSignal>();
const _: () = assert!(MAX_WAKE_QUEUE_BYTES <= 2_048 && MAX_SSE_EVENT_FRAME_BYTES <= 32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WakeSignal {
    Worktree(Uuid),
    ResyncRequired,
}

/// Process-scoped fanout: one PostgreSQL listener and a fixed-size queue serve all streams.
pub struct CanvasEventWakeHub {
    sender: broadcast::Sender<WakeSignal>,
    stream_slots: Arc<Semaphore>,
}

impl CanvasEventWakeHub {
    /// Start one long-lived PgListener for this Rust API process after validating the trigger migration.
    pub async fn start(pool: PgPool) -> anyhow::Result<Arc<Self>> {
        let trigger_ready: bool = sqlx::query_scalar(
            "SELECT EXISTS (\
                SELECT 1 FROM pg_trigger \
                WHERE tgrelid = to_regclass('canvas.canvas_group_outbox') \
                  AND tgname = 'canvas_group_outbox_wake' \
                  AND NOT tgisinternal AND tgenabled IN ('O', 'A')\
            )",
        )
        .fetch_one(&pool)
        .await?;
        if !trigger_ready {
            anyhow::bail!("Canvas Outbox wake trigger migration is not installed");
        }

        let listener = connect_listener(&pool).await?;
        let (sender, _) = broadcast::channel(WAKE_QUEUE_CAPACITY);
        let hub = Arc::new(Self {
            sender,
            stream_slots: Arc::new(Semaphore::new(MAX_ACTIVE_STREAMS_PER_PROCESS)),
        });

        tokio::spawn(run_listener(pool, listener, Arc::clone(&hub)));
        Ok(hub)
    }

    fn subscribe(&self) -> broadcast::Receiver<WakeSignal> {
        self.sender.subscribe()
    }

    fn try_acquire_stream(&self) -> Result<OwnedSemaphorePermit, GroupApiError> {
        Arc::clone(&self.stream_slots)
            .try_acquire_owned()
            .map_err(|_| GroupApiError::feature_unavailable("canvas_event_stream_capacity_reached"))
    }

    fn publish(&self, signal: WakeSignal) {
        let _ = self.sender.send(signal);
    }
}

pub(super) fn router() -> Router<GroupApiState> {
    Router::new().route(
        "/api/v1/worktrees/{worktree_id}/canvas-events/stream",
        get(stream_worktree_canvas_events),
    )
}

async fn stream_worktree_canvas_events(
    State(state): State<GroupApiState>,
    AuthenticatedUser(actor): AuthenticatedUser,
    Path(worktree_id): Path<String>,
) -> Result<Response, GroupApiError> {
    super::validate_actor(&actor)?;
    super::require_scope(&actor, "worktree:read")?;
    super::require_scope(&actor, "canvas:read")?;
    let worktree_id = Uuid::parse_str(&worktree_id).map_err(|_| GroupApiError::bad_request())?;
    let hub = state
        .canvas_event_wake_hub
        .as_ref()
        .ok_or_else(|| GroupApiError::feature_unavailable("canvas_event_stream_unavailable"))?;
    let permit = hub.try_acquire_stream()?;

    let receiver = hub.subscribe();
    reauthorize_worktree_canvas_read(&state.resolver.pool, &actor, worktree_id).await?;
    let stream_state = StreamState {
        receiver,
        actor,
        worktree_id,
        pool: state.resolver.pool.clone(),
        close_after_resync: false,
        _permit: permit,
    };
    let events = stream::unfold(stream_state, |mut state| async move {
        if state.close_after_resync {
            return None;
        }
        loop {
            match state.receiver.recv().await {
                Ok(signal) => match signal_for_subscriber(signal, state.worktree_id) {
                    Some(WakeSignal::Worktree(_)) => {
                        if reauthorize_worktree_canvas_read(
                            &state.pool,
                            &state.actor,
                            state.worktree_id,
                        )
                        .await
                        .is_err()
                        {
                            return None;
                        }
                        return Some((Ok::<_, Infallible>(event("canvas-wake")), state));
                    }
                    Some(WakeSignal::ResyncRequired) => {
                        state.close_after_resync = true;
                        return Some((Ok::<_, Infallible>(event("resync_required")), state));
                    }
                    None => continue,
                },
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    state.close_after_resync = true;
                    return Some((Ok::<_, Infallible>(event("resync_required")), state));
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    let mut response = Sse::new(events).into_response();
    response.headers_mut().insert(
        HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    Ok(response)
}

struct StreamState {
    receiver: broadcast::Receiver<WakeSignal>,
    actor: AuthUser,
    worktree_id: Uuid,
    pool: PgPool,
    close_after_resync: bool,
    // Keeps the global connection permit for exactly the stream lifetime.
    _permit: OwnedSemaphorePermit,
}

fn event(name: &'static str) -> Event {
    Event::default().event(name).data("")
}

fn signal_for_subscriber(signal: WakeSignal, worktree_id: Uuid) -> Option<WakeSignal> {
    match signal {
        WakeSignal::Worktree(wake_worktree_id) if wake_worktree_id == worktree_id => Some(signal),
        WakeSignal::Worktree(_) => None,
        WakeSignal::ResyncRequired => Some(signal),
    }
}

async fn connect_listener(pool: &PgPool) -> Result<PgListener, sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(NOTIFY_CHANNEL).await?;
    Ok(listener)
}

async fn run_listener(pool: PgPool, mut listener: PgListener, hub: Arc<CanvasEventWakeHub>) {
    loop {
        match listener.try_recv().await {
            Ok(Some(notification)) if notification.channel() == NOTIFY_CHANNEL => {
                let payload = notification.payload();
                let worktree_id = (payload.len() <= MAX_NOTIFY_PAYLOAD_BYTES)
                    .then(|| Uuid::parse_str(payload).ok())
                    .flatten();
                match worktree_id {
                    Some(worktree_id) => hub.publish(WakeSignal::Worktree(worktree_id)),
                    None => hub.publish(WakeSignal::ResyncRequired),
                }
            }
            Ok(Some(_)) => hub.publish(WakeSignal::ResyncRequired),
            Ok(None) => {
                warn!("Canvas PgListener disconnected; closing streams for durable catch-up");
                hub.publish(WakeSignal::ResyncRequired);
                listener = reconnect_listener(&pool, &hub).await;
            }
            Err(_) => {
                warn!("Canvas PgListener receive failed; closing streams for durable catch-up");
                hub.publish(WakeSignal::ResyncRequired);
                listener = reconnect_listener(&pool, &hub).await;
            }
        }
    }
}

async fn reconnect_listener(pool: &PgPool, hub: &CanvasEventWakeHub) -> PgListener {
    let mut backoff = Duration::from_secs(1);
    loop {
        match connect_listener(pool).await {
            Ok(listener) => {
                // Also closes streams opened while the listener was offline.
                hub.publish(WakeSignal::ResyncRequired);
                return listener;
            }
            Err(_) => {
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wake_signal_payload_and_event_frames_are_bounded() {
        assert!(MAX_WAKE_QUEUE_BYTES <= 2_048);
        assert!(MAX_SSE_EVENT_FRAME_BYTES <= 32);
        assert_eq!(
            std::mem::size_of::<WakeSignal>() * WAKE_QUEUE_CAPACITY,
            MAX_WAKE_QUEUE_BYTES
        );
        assert_eq!(Uuid::nil().to_string().len(), MAX_NOTIFY_PAYLOAD_BYTES);
    }

    #[tokio::test]
    async fn slow_subscriber_is_detected_by_bounded_broadcast_lag() {
        let (sender, mut receiver) = broadcast::channel(WAKE_QUEUE_CAPACITY);
        for index in 0..=WAKE_QUEUE_CAPACITY {
            sender
                .send(WakeSignal::Worktree(Uuid::from_u128(index as u128)))
                .expect("receiver remains subscribed");
        }
        assert!(matches!(
            receiver.recv().await,
            Err(broadcast::error::RecvError::Lagged(_))
        ));
    }

    #[tokio::test]
    async fn subscribers_filter_by_worktree_id_only() {
        let target = Uuid::new_v4();
        let other = Uuid::new_v4();
        assert_eq!(
            signal_for_subscriber(WakeSignal::Worktree(other), target),
            None
        );
        assert_eq!(
            signal_for_subscriber(WakeSignal::Worktree(target), target),
            Some(WakeSignal::Worktree(target))
        );
        assert_eq!(
            signal_for_subscriber(WakeSignal::ResyncRequired, target),
            Some(WakeSignal::ResyncRequired)
        );
        assert_ne!(target, other);
    }
}
