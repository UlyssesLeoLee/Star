/*
@cypher schema=1 source_sha256=847d653ace7380d78f797844c90ededc7876f2a7e050c68344832f2f40f28ed3
MERGE (self:File {path:"crates/platform-messaging-runtime/src/admission_receive.rs"})
MERGE (receiver:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver",kind:"struct"})
MERGE (transport:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport",kind:"trait"})
MERGE (delivery:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery",kind:"trait"})
MERGE (auth:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization",kind:"trait"})
MERGE (handler:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveHandler",kind:"trait"})
MERGE (consume:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::consume_one",kind:"method"})
MERGE (process:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::process",kind:"method"})
MERGE (validate:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::validate_delivery",kind:"function"})
MERGE (pull:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport::pull_one",kind:"method"})
MERGE (verify:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization::verify",kind:"method"})
MERGE (commit:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveHandler::commit",kind:"method"})
MERGE (ack:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery::acknowledge",kind:"method"})
MERGE (self)-[:DEFINES]->(receiver)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(auth)
MERGE (self)-[:DEFINES]->(handler)
MERGE (self)-[:DEFINES]->(consume)
MERGE (self)-[:DEFINES]->(process)
MERGE (self)-[:DEFINES]->(validate)
MERGE (self)-[:DEFINES]->(pull)
MERGE (self)-[:DEFINES]->(verify)
MERGE (self)-[:DEFINES]->(commit)
MERGE (self)-[:DEFINES]->(ack)
MERGE (consume)-[:CALLS]->(process)
MERGE (process)-[:CALLS]->(pull)
MERGE (process)-[:CALLS]->(validate)
MERGE (process)-[:CALLS]->(verify)
MERGE (process)-[:CALLS]->(commit)
MERGE (process)-[:CALLS]->(ack)
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
MERGE (new:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::new",kind:"method"})
MERGE (self)-[:DEFINES]->(new)
@endcypher
*/
//! Broker-neutral, one-in-flight admission receive policy. No default authorization provider.
use std::{future::Future, time::Duration};

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
    /// Bounded overall deadline exceeded.
    #[error("admission receive deadline exceeded")]
    Deadline,
    /// Cancellation or closed cancellation channel.
    #[error("admission receive cancelled")]
    Cancelled,
}

/// One bounded delivery. Adapters must reject oversized wire before allocating a payload buffer.
pub trait AdmissionReceivedDelivery: Send + Sync {
    /// Original exact wire, at most 16 KiB.
    fn wire(&self) -> &[u8];
    /// Authenticated provider evidence to be independently verified.
    fn origin(&self) -> &AdmissionReceiveOrigin;
    /// Confirm only after the owner commit and current authorization checks.
    fn acknowledge(&self) -> impl Future<Output = Result<(), AdmissionReceiveError>> + Send;
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
}

impl<
        T: AdmissionReceiveTransport,
        H: AdmissionReceiveHandler,
        A: AdmissionReceiveAuthorization,
    > AdmissionReceiver<T, H, A>
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
        })
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
        let id = validate_delivery(self.scope, delivery.origin(), delivery.wire())?;
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

fn validate_delivery(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
    wire: &[u8],
) -> Result<Uuid, AdmissionReceiveError> {
    if wire.is_empty()
        || wire.len() > 16384
        || origin.scope != scope
        || origin.message_id.is_nil()
        || origin.sequence <= 0
        || origin.provider_identity.is_empty()
        || origin.provider_identity.len() > 128
        || origin.log_id.is_empty()
        || origin.log_id.len() > 128
    {
        return Err(AdmissionReceiveError::Contract);
    }
    let (tenant, id) = match scope.kind {
        AdmissionReceiveKind::Request => {
            let m = ScheduleRunAdmissionCommandV1::decode_json(wire)
                .map_err(|_| AdmissionReceiveError::Contract)?;
            (m.tenant_id, m.request_id)
        }
        AdmissionReceiveKind::Result => {
            let m = ScheduleRunAdmissionResultV1::decode_json(wire)
                .map_err(|_| AdmissionReceiveError::Contract)?;
            (m.tenant_id, m.result_id)
        }
        AdmissionReceiveKind::Resolution => {
            let m = ScheduleRunAdmissionResolutionV1::decode_json(wire)
                .map_err(|_| AdmissionReceiveError::Contract)?;
            (m.tenant_id, m.resolution_id)
        }
    };
    if tenant != scope.tenant_id || id != origin.message_id {
        return Err(AdmissionReceiveError::Contract);
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
                    sequence: 1,
                },
                wire,
                events: Arc::new(Mutex::new(Vec::new())),
                fail_commit: false,
                revoke_after_commit: false,
                fail_ack: false,
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
        let f = Fixture::new();
        assert!(validate_delivery(f.origin.scope, &f.origin, &vec![0; 16385]).is_err());
        let mut f = Fixture::new();
        f.origin.sequence = 0;
        assert!(validate_delivery(f.origin.scope, &f.origin, &f.wire).is_err());
    }
}
