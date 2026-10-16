// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=44e9444f35bb89edb1478d2be9643d9b1c1f06e05b0af4525939d352b8bbfb1a
MERGE (self:File {path:"crates/bi-owner-runtime/src/task_event_receiver.rs"})
MERGE (route:Type {id:"bi_owner_runtime::TaskEventReceiveRoute",kind:"struct"})
MERGE (origin:Type {id:"bi_owner_runtime::TaskEventReceiveOrigin",kind:"struct"})
MERGE (delivery:Type {id:"bi_owner_runtime::TaskEventReceiveDelivery",kind:"trait"})
MERGE (transport:Type {id:"bi_owner_runtime::TaskEventReceiveTransport",kind:"trait"})
MERGE (authorization:Type {id:"bi_owner_runtime::TaskEventReceiveAuthorization",kind:"trait"})
MERGE (quarantine:Type {id:"bi_owner_runtime::TaskEventQuarantinePort",kind:"trait"})
MERGE (commit_port:Type {id:"bi_owner_runtime::TaskEventInboxCommitPort",kind:"trait"})
MERGE (receiver:Type {id:"bi_owner_runtime::TaskEventReceiver",kind:"struct"})
MERGE (fault:Type {id:"bi_owner_runtime::TaskEventQuarantineFault",kind:"enum"})
MERGE (quarantine_receipt:Type {id:"bi_owner_runtime::TaskEventQuarantineReceipt",kind:"struct"})
MERGE (outcome:Type {id:"bi_owner_runtime::TaskEventReceiveOutcome",kind:"enum"})
MERGE (error:Type {id:"bi_owner_runtime::TaskEventReceiveError",kind:"enum"})
MERGE (semaphore:Type {id:"tokio::sync::Semaphore",kind:"struct"})
MERGE (route_new:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveRoute.new",kind:"method"})
MERGE (origin_new:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveOrigin.new",kind:"method"})
MERGE (construct:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.new",kind:"method"})
MERGE (receive:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.receive_one",kind:"method"})
MERGE (process:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.process_one",kind:"method"})
MERGE (poison:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.quarantine",kind:"method"})
MERGE (impl_commit:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventInboxStore.commit_exact_wire",kind:"method"})
MERGE (valid_label:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::valid_label",kind:"function"})
MERGE (tests:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests",kind:"module"})
MERGE (route_tenant:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveRoute.tenant_id",kind:"method"})
MERGE (route_pin:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveRoute.pin",kind:"method"})
MERGE (origin_provider:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveOrigin.provider_identity",kind:"method"})
MERGE (origin_log:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveOrigin.log_id",kind:"method"})
MERGE (origin_generation:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveOrigin.log_generation",kind:"method"})
MERGE (origin_sequence:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveOrigin.sequence",kind:"method"})
MERGE (origin_message:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveOrigin.message_id",kind:"method"})
MERGE (delivery_ack:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveDelivery.acknowledge",kind:"method"})
MERGE (delivery_terminal:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveDelivery.terminate_permanently",kind:"method"})
MERGE (transport_pull:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveTransport.pull_one",kind:"method"})
MERGE (authorization_verify:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveAuthorization.verify_origin",kind:"method"})
MERGE (quarantine_record:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventQuarantinePort.record",kind:"method"})
MERGE (commit_port_method:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventInboxCommitPort.commit_exact_wire",kind:"method"})
MERGE (receiver_new:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.new",kind:"method"})
MERGE (test_commit:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests.owner_commit_and_post_commit_origin_check_precede_ack",kind:"function"})
MERGE (test_revoke:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests.source_revocation_after_commit_leaves_delivery_unacknowledged",kind:"function"})
MERGE (test_conflict:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests.identity_conflict_requires_quarantine_commit_before_terminal_disposition",kind:"function"})
MERGE (test_malformed:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests.malformed_wire_is_quarantined_without_calling_inbox",kind:"function"})
MERGE (test_quarantine:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests.quarantine_failure_never_terminally_acknowledges",kind:"function"})
MERGE (test_cancel:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::tests.cancellation_before_pull_has_no_transport_side_effects",kind:"function"})
MERGE (self)-[:DEFINES]->(route)
MERGE (self)-[:DEFINES]->(origin)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(authorization)
MERGE (self)-[:DEFINES]->(quarantine)
MERGE (self)-[:DEFINES]->(commit_port)
MERGE (self)-[:DEFINES]->(receiver)
MERGE (self)-[:DEFINES]->(fault)
MERGE (self)-[:DEFINES]->(quarantine_receipt)
MERGE (self)-[:DEFINES]->(outcome)
MERGE (self)-[:DEFINES]->(error)
MERGE (route)-[:DEFINES]->(route_new)
MERGE (route)-[:DEFINES]->(route_tenant)
MERGE (route)-[:DEFINES]->(route_pin)
MERGE (origin)-[:DEFINES]->(origin_new)
MERGE (origin)-[:DEFINES]->(origin_provider)
MERGE (origin)-[:DEFINES]->(origin_log)
MERGE (origin)-[:DEFINES]->(origin_generation)
MERGE (origin)-[:DEFINES]->(origin_sequence)
MERGE (origin)-[:DEFINES]->(origin_message)
MERGE (delivery)-[:DEFINES]->(delivery_ack)
MERGE (delivery)-[:DEFINES]->(delivery_terminal)
MERGE (transport)-[:DEFINES]->(transport_pull)
MERGE (authorization)-[:DEFINES]->(authorization_verify)
MERGE (quarantine)-[:DEFINES]->(quarantine_record)
MERGE (commit_port)-[:DEFINES]->(commit_port_method)
MERGE (receiver)-[:DEFINES]->(construct)
MERGE (receiver)-[:DEFINES]->(receiver_new)
MERGE (receiver)-[:DEFINES]->(receive)
MERGE (receiver)-[:DEFINES]->(process)
MERGE (receiver)-[:DEFINES]->(poison)
MERGE (impl_commit)-[:IMPLEMENTS]->(commit_port)
MERGE (receive)-[:CALLS]->(process)
MERGE (construct)-[:CREATES]->(semaphore)
MERGE (receive)-[:USES_TYPE]->(semaphore)
MERGE (receive)-[:CALLS]->(transport_pull)
MERGE (origin_new)-[:CALLS]->(valid_label)
MERGE (route_new)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventRoutePin.is_valid",kind:"method"})
MERGE (tests)-[:CONTAINS]->(test_commit)
MERGE (tests)-[:CONTAINS]->(test_revoke)
MERGE (tests)-[:CONTAINS]->(test_conflict)
MERGE (tests)-[:CONTAINS]->(test_malformed)
MERGE (tests)-[:CONTAINS]->(test_quarantine)
MERGE (tests)-[:CONTAINS]->(test_cancel)
MERGE (process)-[:CALLS]->(poison)
MERGE (process)-[:CALLS]->(impl_commit)
MERGE (process)-[:CALLS]->(authorization_verify)
MERGE (process)-[:CALLS]->(delivery_ack)
MERGE (process)-[:USES_TYPE]->(transport)
MERGE (process)-[:USES_TYPE]->(authorization)
MERGE (process)-[:USES_TYPE]->(delivery)
MERGE (poison)-[:USES_TYPE]->(quarantine)
MERGE (poison)-[:USES_TYPE]->(authorization)
MERGE (poison)-[:USES_TYPE]->(delivery)
MERGE (poison)-[:CALLS]->(quarantine_record)
MERGE (poison)-[:CALLS]->(delivery_terminal)
MERGE (test_commit)-[:CALLS]->(receive)
MERGE (test_revoke)-[:CALLS]->(receive)
MERGE (test_conflict)-[:CALLS]->(receive)
MERGE (test_malformed)-[:CALLS]->(receive)
MERGE (test_quarantine)-[:CALLS]->(receive)
MERGE (test_cancel)-[:CALLS]->(receive)
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"tokio",kind:"crate"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"tokio-util",kind:"crate"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"sha2",kind:"crate"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"uuid",kind:"crate"})
@endcypher
*/
//! One-delivery BI receiver core. Concrete transports and quarantine persistence are injected.

use std::future::Future;

use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{
    sync::Semaphore,
    time::{Instant, timeout_at},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::task_event_inbox::{
    AuthenticatedTaskEventPrincipal, MAX_TASK_WIRE_BYTES, ReceiptOutcome, TaskEventInboxError,
    TaskEventInboxReceipt, TaskEventInboxStore, TaskEventRoutePin, validated_event_id,
};

/// Tenant and immutable route pin for one BI Task-event subscription.
#[derive(Clone, Debug)]
pub struct TaskEventReceiveRoute {
    tenant_id: Uuid,
    pin: TaskEventRoutePin,
}

impl TaskEventReceiveRoute {
    /// Reject incomplete or invalid routes before a transport adapter is opened.
    pub fn new(tenant_id: Uuid, pin: TaskEventRoutePin) -> Result<Self, TaskEventReceiveError> {
        if tenant_id.is_nil() || !pin.is_valid() {
            return Err(TaskEventReceiveError::InvalidRoute);
        }
        Ok(Self { tenant_id, pin })
    }

    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    pub fn pin(&self) -> &TaskEventRoutePin {
        &self.pin
    }
}

/// Durable transport origin independently checked against the pinned source service.
#[derive(Clone, Debug)]
pub struct TaskEventReceiveOrigin {
    provider_identity: String,
    log_id: String,
    log_generation: Uuid,
    sequence: i64,
    message_id: Uuid,
}

impl TaskEventReceiveOrigin {
    pub fn new(
        provider_identity: String,
        log_id: String,
        log_generation: Uuid,
        sequence: i64,
        message_id: Uuid,
    ) -> Result<Self, TaskEventReceiveError> {
        if !valid_label(&provider_identity, 64)
            || !valid_label(&log_id, 128)
            || log_generation.is_nil()
            || sequence <= 0
            || message_id.is_nil()
        {
            return Err(TaskEventReceiveError::InvalidOrigin);
        }
        Ok(Self {
            provider_identity,
            log_id,
            log_generation,
            sequence,
            message_id,
        })
    }

    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub fn log_id(&self) -> &str {
        &self.log_id
    }

    pub fn log_generation(&self) -> Uuid {
        self.log_generation
    }

    pub fn sequence(&self) -> i64 {
        self.sequence
    }

    pub fn message_id(&self) -> Uuid {
        self.message_id
    }
}

/// One bounded delivery. Adapters must reject over-limit frames before buffering them.
pub trait TaskEventReceiveDelivery: Send + Sync {
    fn exact_wire(&self) -> &[u8];
    fn origin(&self) -> &TaskEventReceiveOrigin;

    /// Confirm only after the owner Inbox commit; adapters must tolerate uncertain, repeated ACK.
    fn acknowledge(&self) -> impl Future<Output = Result<(), TaskEventReceiveError>> + Send;

    /// Unsupported terminal disposition fails closed by default.
    fn terminate_permanently(
        &self,
    ) -> impl Future<Output = Result<(), TaskEventReceiveError>> + Send {
        async { Err(TaskEventReceiveError::Transport) }
    }
}

/// Pull one route-pinned delivery without unbounded prefetch.
///
/// The returned future can be dropped by cancellation or deadline. Adapters must therefore leave an
/// unreturned delivery unacknowledged and redeliverable. They must enforce the wire bound before
/// buffering a frame and make receive/ack operations safe to retry after an uncertain cancellation.
pub trait TaskEventReceiveTransport: Send + Sync {
    type Delivery: TaskEventReceiveDelivery;

    async fn pull_one(
        &self,
        route: &TaskEventReceiveRoute,
    ) -> Result<Option<Self::Delivery>, TaskEventReceiveError>;
}

/// Trusted deployment adapter verifies provider identity, pinned route generation, current
/// authorization and the exact bytes before returning an authenticated principal to the core.
pub trait TaskEventReceiveAuthorization: Send + Sync {
    async fn verify_origin(
        &self,
        route: &TaskEventReceiveRoute,
        origin: &TaskEventReceiveOrigin,
        exact_wire: &[u8],
    ) -> Result<(), TaskEventReceiveError>;
}

/// Metadata-only quarantine must durably and idempotently record route, origin, wire digest/length,
/// and fault before a poison delivery can be terminally acknowledged. It must never persist the wire.
/// Use `(route, provider_identity, log_generation, log_id, sequence)` as its key; a replay with a
/// different wire digest or fault must fail closed instead of replacing the committed receipt.
pub trait TaskEventQuarantinePort: Send + Sync {
    async fn record(
        &self,
        route: &TaskEventReceiveRoute,
        origin: &TaskEventReceiveOrigin,
        wire_sha256: [u8; 32],
        wire_len: u32,
        fault: TaskEventQuarantineFault,
    ) -> Result<TaskEventQuarantineReceipt, TaskEventReceiveError>;
}

/// Owner-local Inbox commit port. Implementations return only after the BI transaction commits and
/// must recover an original receipt through exact replay if cancellation obscures the commit result.
pub trait TaskEventInboxCommitPort: Send + Sync {
    async fn commit_exact_wire(
        &self,
        principal: &AuthenticatedTaskEventPrincipal,
        route: &TaskEventRoutePin,
        exact_wire: &[u8],
    ) -> Result<TaskEventInboxReceipt, TaskEventInboxError>;
}

impl TaskEventInboxCommitPort for TaskEventInboxStore {
    async fn commit_exact_wire(
        &self,
        principal: &AuthenticatedTaskEventPrincipal,
        route: &TaskEventRoutePin,
        exact_wire: &[u8],
    ) -> Result<TaskEventInboxReceipt, TaskEventInboxError> {
        TaskEventInboxStore::commit_exact_wire(self, principal, route, exact_wire).await
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskEventQuarantineFault {
    MalformedEnvelope,
    MessageIdentityMismatch,
    ContentConflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskEventQuarantineReceipt {
    pub quarantine_id: Uuid,
}

#[derive(Debug)]
pub enum TaskEventReceiveOutcome {
    Idle,
    Committed(TaskEventInboxReceipt),
    ExactReplay(TaskEventInboxReceipt),
    Quarantined {
        receipt: TaskEventQuarantineReceipt,
        fault: TaskEventQuarantineFault,
    },
}

#[derive(Debug, Error)]
pub enum TaskEventReceiveError {
    #[error("invalid BI Task-event route")]
    InvalidRoute,
    #[error("invalid durable Task-event origin")]
    InvalidOrigin,
    #[error("Task-event wire exceeds the receiver limit")]
    WireTooLarge,
    #[error("Task-event source identity or route authorization failed")]
    Unauthorized,
    #[error("Task-event transport operation failed")]
    Transport,
    #[error("durable Task-event quarantine did not commit")]
    Quarantine,
    #[error("Task-event receive was cancelled")]
    Cancelled,
    #[error("another delivery is already being processed for this receiver")]
    Busy,
    #[error("Task-event receive deadline elapsed")]
    Deadline,
    #[error(transparent)]
    Inbox(#[from] TaskEventInboxError),
}

/// Explicitly driven, static-dispatch receiver. Constructing it does not start background work.
pub struct TaskEventReceiver<T, A, I, Q> {
    route: TaskEventReceiveRoute,
    transport: T,
    authorization: A,
    inbox: I,
    quarantine: Q,
    in_flight: Semaphore,
}

impl<T, A, I, Q> TaskEventReceiver<T, A, I, Q>
where
    T: TaskEventReceiveTransport,
    A: TaskEventReceiveAuthorization,
    I: TaskEventInboxCommitPort,
    Q: TaskEventQuarantinePort,
{
    pub fn new(
        route: TaskEventReceiveRoute,
        transport: T,
        authorization: A,
        inbox: I,
        quarantine: Q,
    ) -> Self {
        Self {
            route,
            transport,
            authorization,
            inbox,
            quarantine,
            in_flight: Semaphore::new(1),
        }
    }

    /// Process at most one delivery with caller-owned cancellation and a fixed deadline.
    /// Cancellation can race a durable commit or ACK; source and BI idempotency resolve replay.
    pub async fn receive_one(
        &self,
        cancellation: &CancellationToken,
        deadline: Instant,
    ) -> Result<TaskEventReceiveOutcome, TaskEventReceiveError> {
        if cancellation.is_cancelled() {
            return Err(TaskEventReceiveError::Cancelled);
        }
        let _permit = self
            .in_flight
            .try_acquire()
            .map_err(|_| TaskEventReceiveError::Busy)?;
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err(TaskEventReceiveError::Cancelled),
            result = timeout_at(deadline, self.process_one()) => {
                result.map_err(|_| TaskEventReceiveError::Deadline)?
            }
        }
    }

    async fn process_one(&self) -> Result<TaskEventReceiveOutcome, TaskEventReceiveError> {
        let Some(delivery) = self.transport.pull_one(&self.route).await? else {
            return Ok(TaskEventReceiveOutcome::Idle);
        };
        let exact_wire = delivery.exact_wire();
        if exact_wire.len() > MAX_TASK_WIRE_BYTES {
            return Err(TaskEventReceiveError::WireTooLarge);
        }
        let origin = delivery.origin();
        if origin.provider_identity != self.route.pin.transport_provider_id {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        self.authorization
            .verify_origin(&self.route, origin, exact_wire)
            .await?;

        let event_id = match validated_event_id(self.route.tenant_id, exact_wire) {
            Ok(event_id) => event_id,
            Err(TaskEventInboxError::InvalidEnvelope | TaskEventInboxError::TenantMismatch) => {
                return self
                    .quarantine(
                        &delivery,
                        TaskEventQuarantineFault::MalformedEnvelope,
                    )
                    .await;
            }
            Err(error) => return Err(TaskEventReceiveError::Inbox(error)),
        };
        if event_id != origin.message_id {
            return self
                .quarantine(
                    &delivery,
                    TaskEventQuarantineFault::MessageIdentityMismatch,
                )
                .await;
        }

        let principal = AuthenticatedTaskEventPrincipal {
            tenant_id: self.route.tenant_id,
            sender_service_id: self.route.pin.sender_service_id,
            receiver_service_id: self.route.pin.receiver_service_id,
        };
        match self
            .inbox
            .commit_exact_wire(&principal, &self.route.pin, exact_wire)
            .await
        {
            Ok(receipt) => {
                self.authorization
                    .verify_origin(&self.route, origin, exact_wire)
                    .await?;
                delivery.acknowledge().await?;
                match receipt.outcome {
                    ReceiptOutcome::Committed => Ok(TaskEventReceiveOutcome::Committed(receipt)),
                    ReceiptOutcome::ExactReplay => {
                        Ok(TaskEventReceiveOutcome::ExactReplay(receipt))
                    }
                }
            }
            Err(TaskEventInboxError::IdentityConflict) => {
                self.quarantine(&delivery, TaskEventQuarantineFault::ContentConflict)
                    .await
            }
            Err(error) => Err(TaskEventReceiveError::Inbox(error)),
        }
    }

    async fn quarantine(
        &self,
        delivery: &T::Delivery,
        fault: TaskEventQuarantineFault,
    ) -> Result<TaskEventReceiveOutcome, TaskEventReceiveError> {
        let exact_wire = delivery.exact_wire();
        let evidence = self
            .quarantine
            .record(
                &self.route,
                delivery.origin(),
                Sha256::digest(exact_wire).into(),
                u32::try_from(exact_wire.len()).map_err(|_| TaskEventReceiveError::WireTooLarge)?,
                fault,
            )
            .await?;
        if evidence.quarantine_id.is_nil() {
            return Err(TaskEventReceiveError::Quarantine);
        }
        self.authorization
            .verify_origin(&self.route, delivery.origin(), exact_wire)
            .await?;
        delivery.terminate_permanently().await?;
        Ok(TaskEventReceiveOutcome::Quarantined {
            receipt: evidence,
            fault,
        })
    }
}

fn valid_label(value: &str, max_len: usize) -> bool {
    (1..=max_len).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;

    use chrono::Utc;
    use serde_json::json;

    use super::*;

    #[derive(Clone, Default)]
    struct EventLog(Arc<Mutex<Vec<&'static str>>>);

    impl EventLog {
        fn push(&self, event: &'static str) {
            self.0.lock().expect("event log mutex").push(event);
        }

        fn values(&self) -> Vec<&'static str> {
            self.0.lock().expect("event log mutex").clone()
        }
    }

    struct FixtureDelivery {
        wire: Vec<u8>,
        origin: TaskEventReceiveOrigin,
        events: EventLog,
    }

    impl TaskEventReceiveDelivery for FixtureDelivery {
        fn exact_wire(&self) -> &[u8] {
            &self.wire
        }

        fn origin(&self) -> &TaskEventReceiveOrigin {
            &self.origin
        }

        async fn acknowledge(&self) -> Result<(), TaskEventReceiveError> {
            self.events.push("ack");
            Ok(())
        }

        async fn terminate_permanently(&self) -> Result<(), TaskEventReceiveError> {
            self.events.push("terminate");
            Ok(())
        }
    }

    struct FixtureTransport {
        delivery: Mutex<Option<FixtureDelivery>>,
        events: EventLog,
    }

    impl TaskEventReceiveTransport for FixtureTransport {
        type Delivery = FixtureDelivery;

        async fn pull_one(
            &self,
            _route: &TaskEventReceiveRoute,
        ) -> Result<Option<Self::Delivery>, TaskEventReceiveError> {
            self.events.push("pull");
            Ok(self.delivery.lock().expect("delivery mutex").take())
        }
    }

    struct FixtureAuthorization {
        events: EventLog,
        deny_verification: Option<usize>,
        verify_count: AtomicUsize,
    }

    impl TaskEventReceiveAuthorization for FixtureAuthorization {
        async fn verify_origin(
            &self,
            _route: &TaskEventReceiveRoute,
            _origin: &TaskEventReceiveOrigin,
            _wire: &[u8],
        ) -> Result<(), TaskEventReceiveError> {
            self.events.push("verify");
            let count = self.verify_count.fetch_add(1, Ordering::SeqCst) + 1;
            if self.deny_verification == Some(count) {
                Err(TaskEventReceiveError::Unauthorized)
            } else {
                Ok(())
            }
        }
    }

    struct FixtureInbox {
        result: Mutex<Option<Result<TaskEventInboxReceipt, TaskEventInboxError>>>,
        events: EventLog,
    }

    impl TaskEventInboxCommitPort for FixtureInbox {
        async fn commit_exact_wire(
            &self,
            _principal: &AuthenticatedTaskEventPrincipal,
            _route: &TaskEventRoutePin,
            _exact_wire: &[u8],
        ) -> Result<TaskEventInboxReceipt, TaskEventInboxError> {
            self.events.push("commit");
            self.result
                .lock()
                .expect("inbox result mutex")
                .take()
                .expect("one inbox result per fixture")
        }
    }

    struct FixtureQuarantine {
        events: EventLog,
        fail: bool,
    }

    impl TaskEventQuarantinePort for FixtureQuarantine {
        async fn record(
            &self,
            _route: &TaskEventReceiveRoute,
            _origin: &TaskEventReceiveOrigin,
            _wire_sha256: [u8; 32],
            _wire_len: u32,
            _fault: TaskEventQuarantineFault,
        ) -> Result<TaskEventQuarantineReceipt, TaskEventReceiveError> {
            self.events.push("quarantine");
            if self.fail {
                Err(TaskEventReceiveError::Quarantine)
            } else {
                Ok(TaskEventQuarantineReceipt {
                    quarantine_id: Uuid::from_u128(60),
                })
            }
        }
    }

    type FixtureReceiver =
        TaskEventReceiver<FixtureTransport, FixtureAuthorization, FixtureInbox, FixtureQuarantine>;

    fn route() -> TaskEventReceiveRoute {
        TaskEventReceiveRoute::new(Uuid::from_u128(1), route_pin())
            .expect("valid fixture route")
    }

    fn route_pin() -> TaskEventRoutePin {
        TaskEventRoutePin {
            sender_service_id: Uuid::from_u128(8),
            receiver_service_id: Uuid::from_u128(9),
            transport_provider_id: "native-grpc".into(),
            transport_provider_version: "1.0".into(),
            transport_profile_id: Uuid::from_u128(10),
            transport_profile_revision: 1,
            transport_profile_digest: [1; 32],
            endpoint_identity: "bi-receiver".into(),
            destination_id: "task-event-facts".into(),
            destination_generation: Uuid::from_u128(11),
        }
    }

    fn origin() -> TaskEventReceiveOrigin {
        TaskEventReceiveOrigin::new(
            "native-grpc".into(),
            "task-event-facts".into(),
            Uuid::from_u128(13),
            1,
            Uuid::from_u128(5),
        )
        .expect("valid fixture origin")
    }

    fn valid_wire() -> Vec<u8> {
        serde_json::to_vec(&json!({
            "envelope_version": 1,
            "tenant_id": Uuid::from_u128(1),
            "project_id": Uuid::from_u128(2),
            "engineering_run_id": Uuid::from_u128(3),
            "event_id": Uuid::from_u128(5),
            "work_item_id": Uuid::from_u128(4),
            "aggregate_type": "lifecycle",
            "aggregate_version": 7,
            "event_type": "task.lifecycle.changed",
            "schema_version": 3,
            "actor_id": Uuid::from_u128(6),
            "correlation_id": Uuid::from_u128(7),
            "occurred_at": "2026-10-16T00:00:00Z",
            "payload": {
                "schema_version": 3,
                "aggregate": {"type": "lifecycle", "version": 7},
                "work_item_id": Uuid::from_u128(4),
                "owner": {
                    "tenant_id": Uuid::from_u128(1),
                    "project_id": Uuid::from_u128(2),
                    "engineering_run_id": Uuid::from_u128(3)
                },
                "lifecycle": {"status": "in_progress", "review_state": "none"},
                "change": {
                    "event_type": "task.lifecycle.changed",
                    "from_status": "claimed",
                    "to_status": "in_progress"
                }
            }
        }))
        .expect("valid fixture serializes")
    }

    fn receipt(outcome: ReceiptOutcome) -> TaskEventInboxReceipt {
        TaskEventInboxReceipt {
            tenant_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            engineering_run_id: Uuid::from_u128(3),
            event_id: Uuid::from_u128(5),
            consumer_id: "bi.task-event-fact.v1",
            sender_service_id: Uuid::from_u128(8),
            receiver_service_id: Uuid::from_u128(9),
            transport_profile_id: Uuid::from_u128(10),
            transport_profile_revision: 1,
            transport_profile_digest: [1; 32],
            endpoint_identity: "bi-receiver".into(),
            destination_id: "task-event-facts".into(),
            destination_generation: Uuid::from_u128(11),
            route_pin_sha256: [2; 32],
            receipt_id: Uuid::from_u128(12),
            wire_sha256: [3; 32],
            committed_at: Utc::now(),
            outcome,
        }
    }

    fn build_receiver(
        wire: Vec<u8>,
        commit_result: Result<TaskEventInboxReceipt, TaskEventInboxError>,
        deny_verification: Option<usize>,
        quarantine_fails: bool,
    ) -> (FixtureReceiver, EventLog) {
        let events = EventLog::default();
        let delivery = FixtureDelivery {
            wire,
            origin: origin(),
            events: events.clone(),
        };
        let transport = FixtureTransport {
            delivery: Mutex::new(Some(delivery)),
            events: events.clone(),
        };
        let authorization = FixtureAuthorization {
            events: events.clone(),
            deny_verification,
            verify_count: AtomicUsize::new(0),
        };
        let inbox = FixtureInbox {
            result: Mutex::new(Some(commit_result)),
            events: events.clone(),
        };
        let quarantine = FixtureQuarantine {
            events: events.clone(),
            fail: quarantine_fails,
        };
        (
            TaskEventReceiver::new(route(), transport, authorization, inbox, quarantine),
            events,
        )
    }

    async fn receive(
        receiver: &FixtureReceiver,
    ) -> Result<TaskEventReceiveOutcome, TaskEventReceiveError> {
        receiver
            .receive_one(
                &CancellationToken::new(),
                Instant::now() + Duration::from_secs(2),
            )
            .await
    }

    #[tokio::test]
    async fn owner_commit_and_post_commit_origin_check_precede_ack() {
        let (receiver, events) = build_receiver(
            valid_wire(),
            Ok(receipt(ReceiptOutcome::Committed)),
            None,
            false,
        );
        let result = receive(&receiver).await.expect("delivery committed");
        assert!(matches!(result, TaskEventReceiveOutcome::Committed(_)));
        assert_eq!(events.values(), ["pull", "verify", "commit", "verify", "ack"]);
    }

    #[tokio::test]
    async fn source_revocation_after_commit_leaves_delivery_unacknowledged() {
        let (receiver, events) = build_receiver(
            valid_wire(),
            Ok(receipt(ReceiptOutcome::Committed)),
            Some(2),
            false,
        );
        assert!(matches!(
            receive(&receiver).await,
            Err(TaskEventReceiveError::Unauthorized)
        ));
        assert_eq!(events.values(), ["pull", "verify", "commit", "verify"]);
    }

    #[tokio::test]
    async fn identity_conflict_requires_quarantine_commit_before_terminal_disposition() {
        let (receiver, events) = build_receiver(
            valid_wire(),
            Err(TaskEventInboxError::IdentityConflict),
            None,
            false,
        );
        let result = receive(&receiver).await.expect("poison durably quarantined");
        assert!(matches!(result, TaskEventReceiveOutcome::Quarantined { .. }));
        assert_eq!(
            events.values(),
            ["pull", "verify", "commit", "quarantine", "verify", "terminate"]
        );
    }

    #[tokio::test]
    async fn malformed_wire_is_quarantined_without_calling_inbox() {
        let (receiver, events) = build_receiver(
            b"{".to_vec(),
            Ok(receipt(ReceiptOutcome::Committed)),
            None,
            false,
        );
        let result = receive(&receiver).await.expect("poison durably quarantined");
        assert!(matches!(result, TaskEventReceiveOutcome::Quarantined { .. }));
        assert_eq!(
            events.values(),
            ["pull", "verify", "quarantine", "verify", "terminate"]
        );
    }

    #[tokio::test]
    async fn quarantine_failure_never_terminally_acknowledges() {
        let (receiver, events) = build_receiver(
            b"{".to_vec(),
            Ok(receipt(ReceiptOutcome::Committed)),
            None,
            true,
        );
        assert!(matches!(
            receive(&receiver).await,
            Err(TaskEventReceiveError::Quarantine)
        ));
        assert_eq!(events.values(), ["pull", "verify", "quarantine"]);
    }

    #[tokio::test]
    async fn cancellation_before_pull_has_no_transport_side_effects() {
        let (receiver, events) = build_receiver(
            valid_wire(),
            Ok(receipt(ReceiptOutcome::Committed)),
            None,
            false,
        );
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            receiver
                .receive_one(&cancellation, Instant::now() + Duration::from_secs(2))
                .await,
            Err(TaskEventReceiveError::Cancelled)
        ));
        assert!(events.values().is_empty());
    }
}
