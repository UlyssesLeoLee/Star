/*
@cypher schema=1 source_sha256=2428da2a2e33b0fe252347b4499f8f7d70219c158dad10d70ebc6db0b420a037
MERGE (self:File {path:"crates/platform-messaging-runtime/src/admission_receive.rs"})
MERGE (receiver:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver",kind:"struct"})
MERGE (transport:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport",kind:"trait"})
MERGE (delivery:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery",kind:"trait"})
MERGE (auth:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization",kind:"trait"})
MERGE (handler:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveHandler",kind:"trait"})
MERGE (quarantine:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveQuarantine",kind:"trait"})
MERGE (poison:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivePoisonFault",kind:"enum"})
MERGE (consume:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::consume_one",kind:"method"})
MERGE (process:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::process",kind:"method"})
MERGE (validate:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::validate_delivery",kind:"function"})
MERGE (validate_origin:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::validate_origin",kind:"function"})
MERGE (decode_delivery:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::decode_delivery",kind:"function"})
MERGE (pull:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport::pull_one",kind:"method"})
MERGE (verify:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization::verify",kind:"method"})
MERGE (verify_source:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization::verify_source",kind:"method"})
MERGE (commit:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveHandler::commit",kind:"method"})
MERGE (ack:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery::acknowledge",kind:"method"})
MERGE (terminate:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery::terminate_permanently",kind:"method"})
MERGE (record_poison:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveQuarantine::record",kind:"method"})
MERGE (self)-[:DEFINES]->(receiver)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(auth)
MERGE (self)-[:DEFINES]->(handler)
MERGE (self)-[:DEFINES]->(quarantine)
MERGE (self)-[:DEFINES]->(poison)
MERGE (self)-[:DEFINES]->(consume)
MERGE (self)-[:DEFINES]->(process)
MERGE (self)-[:DEFINES]->(validate)
MERGE (self)-[:DEFINES]->(validate_origin)
MERGE (self)-[:DEFINES]->(decode_delivery)
MERGE (self)-[:DEFINES]->(pull)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(verify_source)
MERGE (self)-[:DEFINES]->(commit)
MERGE (self)-[:DEFINES]->(ack)
MERGE (self)-[:DEFINES]->(terminate)
MERGE (self)-[:DEFINES]->(record_poison)
MERGE (consume)-[:CALLS]->(process)
MERGE (process)-[:CALLS]->(pull)
MERGE (process)-[:CALLS]->(validate_origin)
MERGE (process)-[:CALLS]->(decode_delivery)
MERGE (process)-[:CALLS]->(verify)
MERGE (process)-[:CALLS]->(commit)
MERGE (process)-[:CALLS]->(ack)
MERGE (process)-[:CALLS]->(verify_source)
MERGE (process)-[:CALLS]->(record_poison)
MERGE (process)-[:CALLS]->(terminate)
MERGE (validate)-[:CALLS]->(validate_origin)
MERGE (validate)-[:CALLS]->(decode_delivery)
MERGE (d14_type_0:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveKind"})
MERGE (self)-[:DEFINES]->(d14_type_0)
MERGE (d14_type_1:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveScope"})
MERGE (self)-[:DEFINES]->(d14_type_1)
MERGE (d14_type_2:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveOrigin"})
MERGE (self)-[:DEFINES]->(d14_type_2)
MERGE (d14_type_3:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveError"})
MERGE (self)-[:DEFINES]->(d14_type_3)
MERGE (d14_type_4:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::Fixture"})
MERGE (self)-[:DEFINES]->(d14_type_4)
MERGE (d14_type_5:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::TestQuarantine"})
MERGE (self)-[:DEFINES]->(d14_type_5)
MERGE (d14_test_0:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::confirmation_follows_commit_and_second_authorization",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_0)
MERGE (d14_test_0)-[:TESTS]->(consume)
MERGE (d14_test_1:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::owner_failure_is_not_confirmed",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_1)
MERGE (d14_test_1)-[:TESTS]->(consume)
MERGE (d14_test_2:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::revocation_after_commit_is_not_confirmed",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_2)
MERGE (d14_test_2)-[:TESTS]->(consume)
MERGE (d14_test_3:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::cancelled_and_closed_control_do_not_pull",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_3)
MERGE (d14_test_3)-[:TESTS]->(consume)
MERGE (d14_test_4:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::malformed_wire_never_reaches_authorization_or_owner",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_4)
MERGE (d14_test_4)-[:TESTS]->(consume)
MERGE (d14_test_5:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::acknowledgement_failure_is_reported_after_commit",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_5)
MERGE (d14_test_5)-[:TESTS]->(consume)
MERGE (d14_test_6:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::wrong_route_identity_and_unbounded_wire_are_rejected",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_6)
MERGE (d14_test_6)-[:TESTS]->(validate)
MERGE (d14_test_7:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::poison_is_quarantined_before_terminal_disposition",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_7)
MERGE (d14_test_7)-[:TESTS]->(process)
MERGE (d14_test_8:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::quarantine_failure_never_terminally_dispositions",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_8)
MERGE (d14_test_8)-[:TESTS]->(process)
MERGE (d14_test_9:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::source_authentication_failure_keeps_poison_unconfirmed",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_9)
MERGE (d14_test_9)-[:TESTS]->(process)
MERGE (d14_test_10:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::terminal_disposition_failure_preserves_quarantine_for_replay",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_10)
MERGE (d14_test_10)-[:TESTS]->(process)
MERGE (d14_test_11:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::tests::source_revocation_after_quarantine_keeps_poison_unconfirmed",kind:"test"})
MERGE (self)-[:DEFINES]->(d14_test_11)
MERGE (d14_test_11)-[:TESTS]->(process)
MERGE (new:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::new",kind:"method"})
MERGE (self)-[:DEFINES]->(new)
@endcypher
*/
//! Broker-neutral, one-in-flight admission receive policy. No default authorization provider.
use std::{future::Future, pin::Pin, time::Duration};

use domain_automation::schedule::{
    ScheduleRunAdmissionCommandV1, ScheduleRunAdmissionResolutionV1, ScheduleRunAdmissionResultV1,
};
use thiserror::Error;
use tokio::sync::watch;
use uuid::Uuid;

/// Logical wire kind; broker routing is confined to the transport implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionReceiveKind {
    /// Schedule to Run: prepare a non-executing admission.
    Request,
    /// Run to Schedule: durable preparation outcome.
    Result,
    /// Schedule to Run: confirm or compensate.
    Resolution,
}

/// Deployment-pinned routing, not evidence of authenticated identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionReceiveScope {
    /// Authorized tenant.
    pub tenant_id: Uuid,
    /// Expected authenticated sender service.
    pub sender_service_id: Uuid,
    /// Local receiving service principal.
    pub recipient_service_id: Uuid,
    /// Permitted message kind.
    pub kind: AdmissionReceiveKind,
}

/// Evidence supplied by an adapter; authorization must independently verify its provenance.
#[derive(Clone, Debug)]
pub struct AdmissionReceiveOrigin {
    /// Logical route asserted by the adapter.
    pub scope: AdmissionReceiveScope,
    /// Stable logical message identity.
    pub message_id: Uuid,
    /// Versioned, deployment-pinned provider/server identity.
    pub provider_identity: String,
    /// Durable transport log identity, distinct from the owner Inbox commit.
    pub log_id: String,
    /// Trusted generation of the durable log; changes when a same-named log is recreated.
    pub log_generation: Uuid,
    /// Positive durable cursor for the current version of the transport contract.
    pub sequence: i64,
}

/// Receive failures leave the delivery unconfirmed; messages require safe idempotent replay.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionReceiveError {
    /// Invalid routing, metadata or typed wire.
    #[error("admission receive contract rejected")]
    Contract,
    /// Current identity/policy authorization failed or is unavailable.
    #[error("admission receive authorization rejected")]
    Unauthorized,
    /// Transport pull/confirmation failed.
    #[error("admission receive transport failed")]
    Transport,
    /// Owner transaction failed; this does not imply rollback if commit outcome is unknown.
    #[error("admission owner commit failed or unknown")]
    OwnerCommit,
    /// Poison quarantine could not be durably recorded or an idempotency conflict was found.
    #[error("admission poison quarantine failed or conflicted")]
    Quarantine,
    /// Bounded overall deadline exceeded.
    #[error("admission receive deadline exceeded")]
    Deadline,
    /// Cancellation or closed cancellation channel.
    #[error("admission receive cancelled")]
    Cancelled,
}

/// Permanent wire faults that may be terminally dispositioned after durable quarantine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionReceivePoisonFault {
    /// Payload was empty, oversized, or could not be decoded as the typed JSON contract.
    MalformedWire,
    /// Decoded payload tenant or message identity did not match the pinned receive route.
    IdentityMismatch,
}

/// One bounded delivery. Adapters must reject oversized wire before allocating a payload buffer.
pub trait AdmissionReceivedDelivery: Send + Sync {
    /// Original exact wire, at most 16 KiB.
    fn wire(&self) -> &[u8];
    /// Authenticated provider evidence to be independently verified.
    fn origin(&self) -> &AdmissionReceiveOrigin;
    /// Confirm only after the owner commit and current authorization checks.
    fn acknowledge(&self) -> impl Future<Output = Result<(), AdmissionReceiveError>> + Send;
    /// Permanently disposition a quarantined message; providers without terminal ack fail closed.
    fn terminate_permanently(
        &self,
    ) -> impl Future<Output = Result<(), AdmissionReceiveError>> + Send {
        async { Err(AdmissionReceiveError::Transport) }
    }
}

/// Pull at most one delivery; do not prefetch into an unbounded adapter queue.
pub trait AdmissionReceiveTransport: Send + Sync {
    /// Provider-specific handle; no per-message future boxing required by this port.
    type Delivery: AdmissionReceivedDelivery;
    /// Pull one message within the pinned route.
    fn pull_one(
        &self,
        scope: AdmissionReceiveScope,
    ) -> impl Future<Output = Result<Option<Self::Delivery>, AdmissionReceiveError>> + Send;
}

/// Trusted deployment component; injected scope/UUID fields alone must never authorize a message.
pub trait AdmissionReceiveAuthorization: Send + Sync {
    /// Verify provider/service identity and current tenant/recipient/kind policy.
    fn verify(
        &self,
        origin: &AdmissionReceiveOrigin,
        wire: &[u8],
    ) -> impl Future<Output = Result<(), AdmissionReceiveError>> + Send;

    /// Authenticate the configured provider/source independently of an untrusted message body.
    /// The default denies poison disposition when the deployment has no source verifier.
    fn verify_source(
        &self,
        _origin: &AdmissionReceiveOrigin,
        _wire: &[u8],
    ) -> impl Future<Output = Result<(), AdmissionReceiveError>> + Send {
        async { Err(AdmissionReceiveError::Unauthorized) }
    }
}

/// Provider-neutral durable quarantine port. Implementations return only after committing a receipt.
pub trait AdmissionReceiveQuarantine: Send + Sync {
    /// Persist bounded source identity, exact wire digest, parser fault, and configured retention.
    /// Use `(provider_identity, log_generation, log_id, sequence)` as an idempotency key; a replay
    /// with a different digest or fault must fail closed instead of replacing the committed receipt.
    fn record<'a>(
        &'a self,
        origin: &'a AdmissionReceiveOrigin,
        wire: &'a [u8],
        fault: AdmissionReceivePoisonFault,
    ) -> Pin<Box<dyn Future<Output = Result<(), AdmissionReceiveError>> + Send + 'a>>;
}

/// Owner-specific commit boundary. An implementation must return only after its DB commit.
pub trait AdmissionReceiveHandler: Send + Sync {
    /// Apply exact wire under the authorized route, returning the incoming logical message ID.
    /// Replay must recover the original committed fact without repeating side effects.
    fn commit(
        &self,
        scope: AdmissionReceiveScope,
        wire: &[u8],
    ) -> impl Future<Output = Result<Uuid, AdmissionReceiveError>> + Send;
}

/// Static-dispatch receiver policy. Constructing this type does not enable product execution.
pub struct AdmissionReceiver<T, H, A> {
    transport: T,
    handler: H,
    authorization: A,
    scope: AdmissionReceiveScope,
    quarantine: Option<std::sync::Arc<dyn AdmissionReceiveQuarantine>>,
}

impl<T: AdmissionReceiveTransport, H: AdmissionReceiveHandler, A: AdmissionReceiveAuthorization>
    AdmissionReceiver<T, H, A>
{
    /// Pin one logical route with explicit transport, owner and authorization implementations.
    pub fn new(
        transport: T,
        handler: H,
        authorization: A,
        scope: AdmissionReceiveScope,
    ) -> Result<Self, AdmissionReceiveError> {
        if scope.tenant_id.is_nil()
            || scope.sender_service_id.is_nil()
            || scope.recipient_service_id.is_nil()
            || scope.sender_service_id == scope.recipient_service_id
        {
            return Err(AdmissionReceiveError::Contract);
        }
        Ok(Self {
            transport,
            handler,
            authorization,
            scope,
            quarantine: None,
        })
    }

    /// Attach an explicit durable quarantine provider; absence keeps poison deliveries unconfirmed.
    pub fn with_quarantine(
        mut self,
        quarantine: std::sync::Arc<dyn AdmissionReceiveQuarantine>,
    ) -> Self {
        self.quarantine = Some(quarantine);
        self
    }

    /// Consume at most one delivery, with a ten-second end-to-end pull/commit/ACK deadline.
    /// Cancellation may race a successful commit/ACK; only idempotent replay resolves ambiguity.
    pub async fn consume_one(
        &mut self,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<Option<Uuid>, AdmissionReceiveError> {
        if *cancellation.borrow() || cancellation.has_changed().is_err() {
            return Err(AdmissionReceiveError::Cancelled);
        }
        tokio::select! {
            biased;
            _ = async {
                loop {
                    if cancellation.changed().await.is_err() || *cancellation.borrow() { break; }
                }
            } => Err(AdmissionReceiveError::Cancelled),
            result = tokio::time::timeout(Duration::from_secs(10), self.process()) => {
                result.map_err(|_| AdmissionReceiveError::Deadline)?
            }
        }
    }

    async fn process(&self) -> Result<Option<Uuid>, AdmissionReceiveError> {
        let Some(delivery) = self.transport.pull_one(self.scope).await? else {
            return Ok(None);
        };
        validate_origin(self.scope, delivery.origin())?;
        let id = match decode_delivery(self.scope, delivery.origin(), delivery.wire()) {
            Ok(id) => id,
            Err(fault) => {
                let Some(quarantine) = &self.quarantine else {
                    return Err(AdmissionReceiveError::Contract);
                };
                self.authorization
                    .verify_source(delivery.origin(), delivery.wire())
                    .await?;
                quarantine
                    .record(delivery.origin(), delivery.wire(), fault)
                    .await?;
                self.authorization
                    .verify_source(delivery.origin(), delivery.wire())
                    .await?;
                delivery.terminate_permanently().await?;
                return Ok(None);
            }
        };
        self.authorization
            .verify(delivery.origin(), delivery.wire())
            .await?;
        let committed = self.handler.commit(self.scope, delivery.wire()).await?;
        if committed != id {
            return Err(AdmissionReceiveError::OwnerCommit);
        }
        self.authorization
            .verify(delivery.origin(), delivery.wire())
            .await?;
        delivery.acknowledge().await?;
        Ok(Some(id))
    }
}

#[cfg(test)]
fn validate_delivery(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
    wire: &[u8],
) -> Result<Uuid, AdmissionReceiveError> {
    validate_origin(scope, origin)?;
    decode_delivery(scope, origin, wire).map_err(|_| AdmissionReceiveError::Contract)
}

fn validate_origin(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
) -> Result<(), AdmissionReceiveError> {
    if origin.scope != scope
        || origin.sequence <= 0
        || origin.provider_identity.is_empty()
        || origin.provider_identity.len() > 128
        || origin.log_id.is_empty()
        || origin.log_id.len() > 128
        || origin.log_generation.is_nil()
    {
        return Err(AdmissionReceiveError::Contract);
    }
    Ok(())
}

fn decode_delivery(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
    wire: &[u8],
) -> Result<Uuid, AdmissionReceivePoisonFault> {
    if wire.is_empty() || wire.len() > 16384 {
        return Err(AdmissionReceivePoisonFault::MalformedWire);
    }
    let (tenant, id) = match scope.kind {
        AdmissionReceiveKind::Request => {
            let m = ScheduleRunAdmissionCommandV1::decode_json(wire)
                .map_err(|_| AdmissionReceivePoisonFault::MalformedWire)?;
            (m.tenant_id, m.request_id)
        }
        AdmissionReceiveKind::Result => {
            let m = ScheduleRunAdmissionResultV1::decode_json(wire)
                .map_err(|_| AdmissionReceivePoisonFault::MalformedWire)?;
            (m.tenant_id, m.result_id)
        }
        AdmissionReceiveKind::Resolution => {
            let m = ScheduleRunAdmissionResolutionV1::decode_json(wire)
                .map_err(|_| AdmissionReceivePoisonFault::MalformedWire)?;
            (m.tenant_id, m.resolution_id)
        }
    };
    if tenant != scope.tenant_id || id.is_nil() || id != origin.message_id {
        return Err(AdmissionReceivePoisonFault::IdentityMismatch);
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_automation::schedule::{
        ScheduleAdmissionRejectionCodeV1, ScheduleRunAdmissionOutcomeV1,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct Fixture {
        origin: AdmissionReceiveOrigin,
        wire: Vec<u8>,
        events: Arc<Mutex<Vec<&'static str>>>,
        fail_commit: bool,
        revoke_after_commit: bool,
        fail_ack: bool,
        fail_terminate: bool,
        reject_source: bool,
        reject_source_after_quarantine: bool,
    }

    impl Fixture {
        fn new() -> Self {
            let scope = AdmissionReceiveScope {
                tenant_id: Uuid::new_v4(),
                sender_service_id: Uuid::new_v4(),
                recipient_service_id: Uuid::new_v4(),
                kind: AdmissionReceiveKind::Result,
            };
            let id = Uuid::new_v4();
            let wire = ScheduleRunAdmissionResultV1 {
                schema_version: 1,
                tenant_id: scope.tenant_id,
                result_id: id,
                request_id: Uuid::new_v4(),
                admission_attempt: 1,
                correlation_id: Uuid::new_v4(),
                occurrence_id: Uuid::new_v4(),
                outcome: ScheduleRunAdmissionOutcomeV1::Rejected,
                task_execution_run_id: None,
                rejection_code: Some(ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized),
                binding_digest: [1; 32],
            }
            .encode_json()
            .unwrap();
            Self {
                origin: AdmissionReceiveOrigin {
                    scope,
                    message_id: id,
                    provider_identity: "TEST_ONLY".into(),
                    log_id: "FIXTURE".into(),
                    log_generation: Uuid::new_v4(),
                    sequence: 1,
                },
                wire,
                events: Arc::new(Mutex::new(Vec::new())),
                fail_commit: false,
                revoke_after_commit: false,
                fail_ack: false,
                fail_terminate: false,
                reject_source: false,
                reject_source_after_quarantine: false,
            }
        }
        fn receiver(&self) -> AdmissionReceiver<Self, Self, Self> {
            AdmissionReceiver::new(self.clone(), self.clone(), self.clone(), self.origin.scope)
                .unwrap()
        }
    }

    impl AdmissionReceivedDelivery for Fixture {
        fn wire(&self) -> &[u8] {
            &self.wire
        }
        fn origin(&self) -> &AdmissionReceiveOrigin {
            &self.origin
        }
        async fn acknowledge(&self) -> Result<(), AdmissionReceiveError> {
            self.events.lock().unwrap().push("ack");
            if self.fail_ack {
                Err(AdmissionReceiveError::Transport)
            } else {
                Ok(())
            }
        }
        async fn terminate_permanently(&self) -> Result<(), AdmissionReceiveError> {
            self.events.lock().unwrap().push("terminate");
            if self.fail_terminate {
                Err(AdmissionReceiveError::Transport)
            } else {
                Ok(())
            }
        }
    }
    impl AdmissionReceiveTransport for Fixture {
        type Delivery = Self;
        async fn pull_one(
            &self,
            _: AdmissionReceiveScope,
        ) -> Result<Option<Self>, AdmissionReceiveError> {
            self.events.lock().unwrap().push("pull");
            Ok(Some(self.clone()))
        }
    }
    impl AdmissionReceiveAuthorization for Fixture {
        async fn verify(
            &self,
            _: &AdmissionReceiveOrigin,
            _: &[u8],
        ) -> Result<(), AdmissionReceiveError> {
            let mut events = self.events.lock().unwrap();
            let revoked = self.revoke_after_commit && events.contains(&"commit");
            events.push("verify");
            if revoked {
                Err(AdmissionReceiveError::Unauthorized)
            } else {
                Ok(())
            }
        }
        async fn verify_source(
            &self,
            _: &AdmissionReceiveOrigin,
            _: &[u8],
        ) -> Result<(), AdmissionReceiveError> {
            let mut events = self.events.lock().unwrap();
            let after_quarantine = events.contains(&"quarantine");
            events.push("verify_source");
            if self.reject_source || (self.reject_source_after_quarantine && after_quarantine) {
                Err(AdmissionReceiveError::Unauthorized)
            } else {
                Ok(())
            }
        }
    }
    impl AdmissionReceiveHandler for Fixture {
        async fn commit(
            &self,
            _: AdmissionReceiveScope,
            _: &[u8],
        ) -> Result<Uuid, AdmissionReceiveError> {
            self.events.lock().unwrap().push("commit");
            if self.fail_commit {
                Err(AdmissionReceiveError::OwnerCommit)
            } else {
                Ok(self.origin.message_id)
            }
        }
    }

    struct TestQuarantine {
        events: Arc<Mutex<Vec<&'static str>>>,
        fail: bool,
    }

    impl AdmissionReceiveQuarantine for TestQuarantine {
        fn record<'a>(
            &'a self,
            _: &'a AdmissionReceiveOrigin,
            _: &'a [u8],
            _: AdmissionReceivePoisonFault,
        ) -> Pin<Box<dyn Future<Output = Result<(), AdmissionReceiveError>> + Send + 'a>> {
            Box::pin(async move {
                self.events.lock().unwrap().push("quarantine");
                if self.fail {
                    Err(AdmissionReceiveError::Quarantine)
                } else {
                    Ok(())
                }
            })
        }
    }

    #[tokio::test]
    async fn confirmation_follows_commit_and_second_authorization() {
        let f = Fixture::new();
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Ok(Some(f.origin.message_id))
        );
        assert_eq!(
            *f.events.lock().unwrap(),
            ["pull", "verify", "commit", "verify", "ack"]
        );
    }
    #[tokio::test]
    async fn owner_failure_is_not_confirmed() {
        let mut f = Fixture::new();
        f.fail_commit = true;
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::OwnerCommit)
        );
        assert_eq!(*f.events.lock().unwrap(), ["pull", "verify", "commit"]);
    }
    #[tokio::test]
    async fn revocation_after_commit_is_not_confirmed() {
        let mut f = Fixture::new();
        f.revoke_after_commit = true;
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Unauthorized)
        );
        assert_eq!(
            *f.events.lock().unwrap(),
            ["pull", "verify", "commit", "verify"]
        );
    }
    #[tokio::test]
    async fn cancelled_and_closed_control_do_not_pull() {
        let f = Fixture::new();
        let (sender, mut cancel) = watch::channel(true);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Cancelled)
        );
        sender.send(false).unwrap();
        drop(sender);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Cancelled)
        );
        assert!(f.events.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn malformed_wire_never_reaches_authorization_or_owner() {
        let mut f = Fixture::new();
        f.wire = b"{}".to_vec();
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Contract)
        );
        assert_eq!(*f.events.lock().unwrap(), ["pull"]);
    }
    #[tokio::test]
    async fn poison_is_quarantined_before_terminal_disposition() {
        let mut f = Fixture::new();
        f.wire = b"{}".to_vec();
        let quarantine = Arc::new(TestQuarantine {
            events: f.events.clone(),
            fail: false,
        });
        let mut receiver = f.receiver().with_quarantine(quarantine);
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(receiver.consume_one(&mut cancel).await, Ok(None));
        assert_eq!(
            *f.events.lock().unwrap(),
            [
                "pull",
                "verify_source",
                "quarantine",
                "verify_source",
                "terminate"
            ]
        );
    }
    #[tokio::test]
    async fn quarantine_failure_never_terminally_dispositions() {
        let mut f = Fixture::new();
        f.wire = b"{}".to_vec();
        let quarantine = Arc::new(TestQuarantine {
            events: f.events.clone(),
            fail: true,
        });
        let mut receiver = f.receiver().with_quarantine(quarantine);
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            receiver.consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Quarantine)
        );
        assert_eq!(
            *f.events.lock().unwrap(),
            ["pull", "verify_source", "quarantine"]
        );
    }
    #[tokio::test]
    async fn source_authentication_failure_keeps_poison_unconfirmed() {
        let mut f = Fixture::new();
        f.wire = b"{}".to_vec();
        f.reject_source = true;
        let quarantine = Arc::new(TestQuarantine {
            events: f.events.clone(),
            fail: false,
        });
        let mut receiver = f.receiver().with_quarantine(quarantine);
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            receiver.consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Unauthorized)
        );
        assert_eq!(*f.events.lock().unwrap(), ["pull", "verify_source"]);
    }
    #[tokio::test]
    async fn source_revocation_after_quarantine_keeps_poison_unconfirmed() {
        let mut f = Fixture::new();
        f.wire = b"{}".to_vec();
        f.reject_source_after_quarantine = true;
        let quarantine = Arc::new(TestQuarantine {
            events: f.events.clone(),
            fail: false,
        });
        let mut receiver = f.receiver().with_quarantine(quarantine);
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            receiver.consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Unauthorized)
        );
        assert_eq!(
            *f.events.lock().unwrap(),
            ["pull", "verify_source", "quarantine", "verify_source"]
        );
    }
    #[tokio::test]
    async fn terminal_disposition_failure_preserves_quarantine_for_replay() {
        let mut f = Fixture::new();
        f.wire = b"{}".to_vec();
        f.fail_terminate = true;
        let quarantine = Arc::new(TestQuarantine {
            events: f.events.clone(),
            fail: false,
        });
        let mut receiver = f.receiver().with_quarantine(quarantine);
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            receiver.consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Transport)
        );
        assert_eq!(
            *f.events.lock().unwrap(),
            [
                "pull",
                "verify_source",
                "quarantine",
                "verify_source",
                "terminate"
            ]
        );
    }
    #[tokio::test]
    async fn acknowledgement_failure_is_reported_after_commit() {
        let mut f = Fixture::new();
        f.fail_ack = true;
        let (_sender, mut cancel) = watch::channel(false);
        assert_eq!(
            f.receiver().consume_one(&mut cancel).await,
            Err(AdmissionReceiveError::Transport)
        );
        assert_eq!(
            *f.events.lock().unwrap(),
            ["pull", "verify", "commit", "verify", "ack"]
        );
    }
    #[test]
    fn wrong_route_identity_and_unbounded_wire_are_rejected() {
        let mut f = Fixture::new();
        let scope = f.origin.scope;
        f.origin.scope.tenant_id = Uuid::new_v4();
        assert!(validate_delivery(scope, &f.origin, &f.wire).is_err());
        f.origin.scope = scope;
        f.origin.message_id = Uuid::new_v4();
        assert!(validate_delivery(scope, &f.origin, &f.wire).is_err());
        f.origin.message_id = Uuid::nil();
        assert!(validate_delivery(scope, &f.origin, &f.wire).is_err());
        let mut f = Fixture::new();
        f.origin.log_generation = Uuid::nil();
        assert!(validate_delivery(f.origin.scope, &f.origin, &f.wire).is_err());
        let f = Fixture::new();
        assert!(validate_delivery(f.origin.scope, &f.origin, &vec![0; 16385]).is_err());
        let mut f = Fixture::new();
        f.origin.sequence = 0;
        assert!(validate_delivery(f.origin.scope, &f.origin, &f.wire).is_err());
    }
}
