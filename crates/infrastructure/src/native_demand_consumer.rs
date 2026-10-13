//! @cypher schema=1 source_sha256=347c4a72e35ff9fe6cfe2894b8acabe9123fb69445339125eb4344d243cc6f5d
//! MERGE (self:File {path:"crates/infrastructure/src/native_demand_consumer.rs"})
//! MERGE (module:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::module",kind:"module"})
//! MERGE (consumer:Type {id:"infrastructure::NativeGrpcInfrastructureDemandConsumer",kind:"struct"})
//! MERGE (receiver:Type {id:"infrastructure::InfrastructureDemandReceiver",kind:"struct"})
//! MERGE (resolver:Type {id:"infrastructure::CurrentInfrastructureBindingResolver",kind:"trait"})
//! MERGE (inbox_port:Type {id:"infrastructure::InfrastructureDemandInboxPort",kind:"trait"})
//! MERGE (consumer_error:Type {id:"infrastructure::NativeInfrastructureDemandConsumerError",kind:"enum"})
//! MERGE (connect:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::NativeGrpcInfrastructureDemandConsumer::connect",kind:"method"})
//! MERGE (with_quarantine:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::NativeGrpcInfrastructureDemandConsumer::with_quarantine",kind:"method"})
//! MERGE (consume_one:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::NativeGrpcInfrastructureDemandConsumer::consume_one",kind:"method"})
//! MERGE (process:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::InfrastructureDemandReceiver::process",kind:"method"})
//! MERGE (decode:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::decode_demand",kind:"function"})
//! MERGE (validate_scope:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::validate_request_scope",kind:"function"})
//! MERGE (validate_origin:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::validate_origin",kind:"function"})
//! MERGE (validate_binding:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::validate_current_binding",kind:"function"})
//! MERGE (quarantine:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::InfrastructureDemandReceiver::quarantine_poison",kind:"method"})
//! MERGE (receive_demand:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::InfrastructureDemandInboxPort::receive_demand",kind:"method"})
//! MERGE (postgres_receive:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::PostgresInfrastructureOwnerInbox::receive_demand",kind:"method"})
//! MERGE (max_wire:Variable {id:"crates/infrastructure/src/native_demand_consumer.rs::MAX_DEMAND_WIRE_BYTES",kind:"constant"})
//! MERGE (scope_type:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveScope",kind:"external_type"})
//! MERGE (transport:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAdmissionReceiveTransport",kind:"external_type"})
//! MERGE (auth:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAdmissionReceiveAuthorization",kind:"external_type"})
//! MERGE (demand:Type {id:"domain_infrastructure::HostInfrastructureDemandV2",kind:"external_type"})
//! MERGE (binding:Type {id:"infrastructure::CurrentInfrastructureBindingV2",kind:"struct"})
//! MERGE (outcome:Type {id:"infrastructure::InboxReceiveOutcome",kind:"enum"})
//! MERGE (store:Type {id:"infrastructure::PostgresInfrastructureOwnerInbox",kind:"struct"})
//! MERGE (quarantine_port:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveQuarantine",kind:"external_type"})
//! MERGE (test_scope:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::only_request_scope_is_accepted",kind:"test"})
//! MERGE (test_wire:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::demand_wire_requires_schema_scope_and_origin_identity",kind:"test"})
//! MERGE (test_binding:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::current_binding_drift_fails_closed",kind:"test"})
//! MERGE (test_resolver:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::resolver_and_store_errors_leave_delivery_unacked",kind:"test"})
//! MERGE (test_missing_resolver:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::missing_binding_resolver_fails_closed_before_pull",kind:"test"})
//! MERGE (test_order:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::owner_commit_and_source_revalidation_precede_ack",kind:"test"})
//! MERGE (test_outcomes:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::pending_terminal_and_replay_outcomes_are_acknowledged_after_commit",kind:"test"})
//! MERGE (test_quarantine:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::poison_requires_durable_quarantine_before_termination",kind:"test"})
//! MERGE (test_quarantine_failure:Symbol {id:"crates/infrastructure/src/native_demand_consumer.rs::tests::quarantine_failure_and_absence_leave_poison_unacked",kind:"test"})
//! MERGE (self)-[:DEFINES]->(module)
//! MERGE (module)-[:DEFINES]->(consumer)
//! MERGE (module)-[:DEFINES]->(receiver)
//! MERGE (module)-[:DEFINES]->(resolver)
//! MERGE (module)-[:DEFINES]->(inbox_port)
//! MERGE (module)-[:DEFINES]->(consumer_error)
//! MERGE (module)-[:DEFINES]->(connect)
//! MERGE (module)-[:DEFINES]->(with_quarantine)
//! MERGE (module)-[:DEFINES]->(consume_one)
//! MERGE (module)-[:DEFINES]->(process)
//! MERGE (module)-[:DEFINES]->(decode)
//! MERGE (module)-[:DEFINES]->(validate_scope)
//! MERGE (module)-[:DEFINES]->(validate_origin)
//! MERGE (module)-[:DEFINES]->(validate_binding)
//! MERGE (module)-[:DEFINES]->(quarantine)
//! MERGE (module)-[:DEFINES]->(receive_demand)
//! MERGE (module)-[:DEFINES]->(postgres_receive)
//! MERGE (module)-[:DEFINES]->(max_wire)
//! MERGE (consumer)-[:USES_TYPE]->(receiver)
//! MERGE (consumer)-[:USES_TYPE]->(store)
//! MERGE (receiver)-[:USES_TYPE]->(transport)
//! MERGE (receiver)-[:USES_TYPE]->(auth)
//! MERGE (receiver)-[:USES_TYPE]->(resolver)
//! MERGE (receiver)-[:USES_TYPE]->(inbox_port)
//! MERGE (receiver)-[:USES_TYPE]->(demand)
//! MERGE (connect)-[:USES_TYPE]->(scope_type)
//! MERGE (connect)-[:CALLS]->(validate_scope)
//! MERGE (connect)-[:CALLS]->(native_connect:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport::connect",kind:"method"})
//! MERGE (connect)-[:CALLS]->(native_auth:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport::authorization",kind:"method"})
//! MERGE (consume_one)-[:CALLS]->(process)
//! MERGE (process)-[:CALLS]->(decode)
//! MERGE (process)-[:CALLS]->(validate_origin)
//! MERGE (process)-[:CALLS]->(validate_binding)
//! MERGE (process)-[:CALLS]->(receive_demand)
//! MERGE (process)-[:CALLS]->(quarantine)
//! MERGE (quarantine)-[:CALLS]->(terminate:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery::terminate_permanently",kind:"method"})
//! MERGE (receive_demand)-[:CALLS]->(postgres_receive)
//! MERGE (postgres_receive)-[:CALLS]->(owner_store:Symbol {id:"crates/infrastructure/src/infrastructure_owner_inbox.rs::PostgresInfrastructureOwnerInbox.receive_demand",kind:"method"})
//! MERGE (process)-[:USES_TYPE]->(outcome)
//! MERGE (process)-[:USES_TYPE]->(quarantine_port)
//! MERGE (test_scope)-[:TESTS]->(validate_scope)
//! MERGE (test_wire)-[:TESTS]->(decode)
//! MERGE (test_binding)-[:TESTS]->(validate_binding)
//! MERGE (test_resolver)-[:TESTS]->(process)
//! MERGE (test_missing_resolver)-[:TESTS]->(process)
//! MERGE (test_order)-[:TESTS]->(process)
//! MERGE (test_outcomes)-[:TESTS]->(process)
//! MERGE (test_quarantine)-[:TESTS]->(quarantine)
//! MERGE (test_quarantine_failure)-[:TESTS]->(quarantine)
//! @endcypher

//! Opt-in, one-message native receiver for Run-owner Infrastructure capacity demands.
//!
//! The receiver commits only the Infrastructure-owned Inbox fact. It does not admit capacity,
//! create reservations, invoke Multipass/K3s, or start a TaskExecutionRun executor.

use std::{sync::Arc, time::Duration};

#[cfg(test)]
use std::{future::Future, pin::Pin};

use async_trait::async_trait;
use domain_infrastructure::HostInfrastructureDemandV2;
use platform_messaging_runtime::{
    admission_receive::{
        AdmissionReceiveAuthorization, AdmissionReceiveError, AdmissionReceiveKind,
        AdmissionReceiveOrigin, AdmissionReceivePoisonFault, AdmissionReceiveQuarantine,
        AdmissionReceiveScope, AdmissionReceiveTransport, AdmissionReceivedDelivery,
    },
    native_grpc::{
        NativeGrpcAdmissionReceiveAuthorization, NativeGrpcAdmissionReceiveTransport,
        NativeGrpcReceiveClientConfig, NativeGrpcSetupError,
    },
};
use thiserror::Error;
use tokio::sync::watch;
#[cfg(test)]
use uuid::Uuid;

use crate::infrastructure_owner_inbox::{
    CurrentInfrastructureBindingV2, InboxReceiveOutcome, PostgresInfrastructureOwnerInbox,
};

const MAX_DEMAND_WIRE_BYTES: usize = 16 * 1024;
const RECEIVE_DEADLINE: Duration = Duration::from_secs(10);

/// Resolve the present Infrastructure-owned binding for a parsed demand.
///
/// Implementations must read trusted owner state. Values in `demand` are lookup claims, never
/// authentication evidence. Resolution errors are fail-closed and leave the delivery unacked.
#[async_trait]
pub trait CurrentInfrastructureBindingResolver: Send + Sync {
    /// Return the current binding snapshot and provider-owned Host generation.
    async fn resolve_current_binding(
        &self,
        demand: &HostInfrastructureDemandV2,
    ) -> Result<CurrentInfrastructureBindingV2, AdmissionReceiveError>;
}

/// Concrete native receiver. It is driven explicitly and has no background task.
pub struct NativeGrpcInfrastructureDemandConsumer {
    receiver: InfrastructureDemandReceiver<
        NativeGrpcAdmissionReceiveTransport,
        PostgresDemandInbox,
        NativeGrpcAdmissionReceiveAuthorization,
    >,
}

/// Setup failures for the opt-in Infrastructure native receiver.
#[derive(Debug, Error)]
pub enum NativeInfrastructureDemandConsumerError {
    /// mTLS native transport setup failed.
    #[error(transparent)]
    NativeSetup(#[from] NativeGrpcSetupError),
    /// The configured owner route was not a valid pinned Request route.
    #[error(transparent)]
    Receive(#[from] AdmissionReceiveError),
}

impl NativeGrpcInfrastructureDemandConsumer {
    /// Connect one mTLS-pinned Request route to the Infrastructure owner Inbox.
    ///
    /// The resolver is mandatory at consume time. Passing `None` keeps the receiver fail-closed:
    /// it returns before pulling a delivery, and cannot acknowledge or persist incoming claims.
    pub async fn connect(
        config: NativeGrpcReceiveClientConfig<'_>,
        inbox: PostgresInfrastructureOwnerInbox,
        resolver: Option<Arc<dyn CurrentInfrastructureBindingResolver>>,
    ) -> Result<Self, NativeInfrastructureDemandConsumerError> {
        validate_request_scope(config.scope)?;
        let scope = config.scope;
        let transport = NativeGrpcAdmissionReceiveTransport::connect(config).await?;
        let authorization = transport.authorization();
        let receiver = InfrastructureDemandReceiver::new(
            transport,
            PostgresDemandInbox(inbox),
            authorization,
            scope,
            resolver,
        )?;
        Ok(Self { receiver })
    }

    /// Attach durable poison quarantine; malformed deliveries remain unacked without it.
    pub fn with_quarantine(mut self, quarantine: Arc<dyn AdmissionReceiveQuarantine>) -> Self {
        self.receiver.quarantine = Some(quarantine);
        self
    }

    /// Consume at most one capacity demand within the cancellation and ten-second deadline.
    pub async fn consume_one(
        &mut self,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<Option<InboxReceiveOutcome>, AdmissionReceiveError> {
        self.receiver.consume_one(cancellation).await
    }
}

#[async_trait]
trait InfrastructureDemandInboxPort: Send + Sync {
    async fn receive_demand(
        &self,
        demand: &HostInfrastructureDemandV2,
        current: &CurrentInfrastructureBindingV2,
    ) -> Result<InboxReceiveOutcome, AdmissionReceiveError>;
}

struct PostgresDemandInbox(PostgresInfrastructureOwnerInbox);

#[async_trait]
impl InfrastructureDemandInboxPort for PostgresDemandInbox {
    async fn receive_demand(
        &self,
        demand: &HostInfrastructureDemandV2,
        current: &CurrentInfrastructureBindingV2,
    ) -> Result<InboxReceiveOutcome, AdmissionReceiveError> {
        self.0
            .receive_demand(demand, current)
            .await
            .map_err(|_| AdmissionReceiveError::OwnerCommit)
    }
}

struct InfrastructureDemandReceiver<T, I, A>
where
    T: AdmissionReceiveTransport,
    I: InfrastructureDemandInboxPort,
    A: AdmissionReceiveAuthorization,
{
    transport: T,
    inbox: I,
    authorization: A,
    scope: AdmissionReceiveScope,
    resolver: Option<Arc<dyn CurrentInfrastructureBindingResolver>>,
    quarantine: Option<Arc<dyn AdmissionReceiveQuarantine>>,
}

impl<T, I, A> InfrastructureDemandReceiver<T, I, A>
where
    T: AdmissionReceiveTransport,
    I: InfrastructureDemandInboxPort,
    A: AdmissionReceiveAuthorization,
{
    fn new(
        transport: T,
        inbox: I,
        authorization: A,
        scope: AdmissionReceiveScope,
        resolver: Option<Arc<dyn CurrentInfrastructureBindingResolver>>,
    ) -> Result<Self, AdmissionReceiveError> {
        validate_request_scope(scope)?;
        Ok(Self {
            transport,
            inbox,
            authorization,
            scope,
            resolver,
            quarantine: None,
        })
    }

    async fn consume_one(
        &mut self,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<Option<InboxReceiveOutcome>, AdmissionReceiveError> {
        if *cancellation.borrow() || cancellation.has_changed().is_err() {
            return Err(AdmissionReceiveError::Cancelled);
        }
        tokio::select! {
            biased;
            _ = async {
                loop {
                    if cancellation.changed().await.is_err() || *cancellation.borrow() {
                        break;
                    }
                }
            } => Err(AdmissionReceiveError::Cancelled),
            result = tokio::time::timeout(RECEIVE_DEADLINE, self.process()) => {
                result.map_err(|_| AdmissionReceiveError::Deadline)?
            }
        }
    }

    async fn process(&self) -> Result<Option<InboxReceiveOutcome>, AdmissionReceiveError> {
        let resolver = self
            .resolver
            .as_ref()
            .ok_or(AdmissionReceiveError::Unauthorized)?;
        let Some(delivery) = self.transport.pull_one(self.scope).await? else {
            return Ok(None);
        };
        let origin = delivery.origin();
        if let Err(fault) = validate_origin(self.scope, origin) {
            self.quarantine_poison(&delivery, fault).await?;
            return Ok(None);
        }
        let demand = match decode_demand(self.scope, origin, delivery.wire()) {
            Ok(demand) => demand,
            Err(fault) => {
                self.quarantine_poison(&delivery, fault).await?;
                return Ok(None);
            }
        };

        // Authentication comes from the native provider's verified origin, never demand fields.
        self.authorization.verify(origin, delivery.wire()).await?;
        let current = resolver.resolve_current_binding(&demand).await?;
        validate_current_binding(&demand, &current)?;
        let outcome = self.inbox.receive_demand(&demand, &current).await?;

        // Revalidate the exact native source after the durable owner commit and before ACK.
        self.authorization.verify(origin, delivery.wire()).await?;
        delivery.acknowledge().await?;
        Ok(Some(outcome))
    }

    async fn quarantine_poison(
        &self,
        delivery: &T::Delivery,
        fault: AdmissionReceivePoisonFault,
    ) -> Result<(), AdmissionReceiveError> {
        let quarantine = self
            .quarantine
            .as_ref()
            .ok_or(AdmissionReceiveError::Quarantine)?;
        let origin = delivery.origin();
        let wire = delivery.wire();
        self.authorization.verify_source(origin, wire).await?;
        quarantine.record(origin, wire, fault).await?;
        self.authorization.verify_source(origin, wire).await?;
        // Native transport maps permanent disposition to the same lease-fenced durable ACK.
        // This occurs only after the poison receipt and source revalidation both committed.
        delivery.terminate_permanently().await?;
        Ok(())
    }
}

fn validate_request_scope(scope: AdmissionReceiveScope) -> Result<(), AdmissionReceiveError> {
    if scope.kind != AdmissionReceiveKind::Request
        || scope.tenant_id.is_nil()
        || scope.sender_service_id.is_nil()
        || scope.recipient_service_id.is_nil()
        || scope.sender_service_id == scope.recipient_service_id
    {
        return Err(AdmissionReceiveError::Contract);
    }
    Ok(())
}

fn validate_origin(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
) -> Result<(), AdmissionReceivePoisonFault> {
    let valid_log_id = !origin.log_id.is_empty()
        && origin.log_id.len() <= 128
        && origin
            .log_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if origin.scope != scope
        || origin.sequence <= 0
        || origin.provider_identity.is_empty()
        || origin.provider_identity.len() > 128
        || !valid_log_id
        || origin.log_generation.is_nil()
        || origin.message_id.is_nil()
    {
        return Err(AdmissionReceivePoisonFault::IdentityMismatch);
    }
    Ok(())
}

fn decode_demand(
    scope: AdmissionReceiveScope,
    origin: &AdmissionReceiveOrigin,
    wire: &[u8],
) -> Result<HostInfrastructureDemandV2, AdmissionReceivePoisonFault> {
    if wire.is_empty() || wire.len() > MAX_DEMAND_WIRE_BYTES {
        return Err(AdmissionReceivePoisonFault::MalformedWire);
    }
    let demand = serde_json::from_slice::<HostInfrastructureDemandV2>(wire)
        .map_err(|_| AdmissionReceivePoisonFault::MalformedWire)?;
    demand
        .validate()
        .map_err(|_| AdmissionReceivePoisonFault::MalformedWire)?;
    if demand.tenant_id != scope.tenant_id || demand.event_id != origin.message_id {
        return Err(AdmissionReceivePoisonFault::IdentityMismatch);
    }
    Ok(demand)
}

fn validate_current_binding(
    demand: &HostInfrastructureDemandV2,
    current: &CurrentInfrastructureBindingV2,
) -> Result<(), AdmissionReceiveError> {
    current
        .binding
        .validate()
        .map_err(|_| AdmissionReceiveError::Unauthorized)?;
    if current.host_generation <= 0 || current.binding != demand.binding {
        return Err(AdmissionReceiveError::Unauthorized);
    }
    // The demand pins K3s generation in its binding. Host generation is separately resolved from
    // Infrastructure owner state and persisted by receive_demand; the wire does not claim it.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration as ChronoDuration, Utc};
    use domain_infrastructure::{
        ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, KubernetesDnsLabel, ProviderKind,
        RunInfrastructureBindingSnapshotV2,
    };
    use platform_messaging_runtime::admission_receive::{
        AdmissionReceiveKind, AdmissionReceivedDelivery,
    };
    use std::sync::Mutex;

    #[derive(Clone)]
    struct Events(Arc<Mutex<Vec<&'static str>>>);

    impl Events {
        fn push(&self, value: &'static str) {
            self.0.lock().expect("events lock").push(value);
        }

        fn snapshot(&self) -> Vec<&'static str> {
            self.0.lock().expect("events lock").clone()
        }
    }

    #[derive(Clone)]
    struct FakeDelivery {
        wire: Vec<u8>,
        origin: AdmissionReceiveOrigin,
        events: Events,
    }

    impl AdmissionReceivedDelivery for FakeDelivery {
        fn wire(&self) -> &[u8] {
            &self.wire
        }

        fn origin(&self) -> &AdmissionReceiveOrigin {
            &self.origin
        }

        async fn acknowledge(&self) -> Result<(), AdmissionReceiveError> {
            self.events.push("ack");
            Ok(())
        }

        async fn terminate_permanently(&self) -> Result<(), AdmissionReceiveError> {
            self.events.push("terminate");
            Ok(())
        }
    }

    struct FakeTransport(Mutex<Option<FakeDelivery>>, Events);

    impl AdmissionReceiveTransport for FakeTransport {
        type Delivery = FakeDelivery;

        async fn pull_one(
            &self,
            scope: AdmissionReceiveScope,
        ) -> Result<Option<Self::Delivery>, AdmissionReceiveError> {
            self.1.push("pull");
            let delivery = self.0.lock().expect("delivery lock").take();
            if delivery
                .as_ref()
                .is_some_and(|item| item.origin.scope != scope)
            {
                return Err(AdmissionReceiveError::Contract);
            }
            Ok(delivery)
        }
    }

    struct FakeAuthorization {
        events: Events,
        deny_source: bool,
    }

    impl AdmissionReceiveAuthorization for FakeAuthorization {
        async fn verify(
            &self,
            _origin: &AdmissionReceiveOrigin,
            _wire: &[u8],
        ) -> Result<(), AdmissionReceiveError> {
            self.events.push("verify");
            Ok(())
        }

        async fn verify_source(
            &self,
            _origin: &AdmissionReceiveOrigin,
            _wire: &[u8],
        ) -> Result<(), AdmissionReceiveError> {
            self.events.push("verify_source");
            if self.deny_source {
                Err(AdmissionReceiveError::Unauthorized)
            } else {
                Ok(())
            }
        }
    }

    struct FakeResolver {
        events: Events,
        current: CurrentInfrastructureBindingV2,
        fail: bool,
    }

    #[async_trait]
    impl CurrentInfrastructureBindingResolver for FakeResolver {
        async fn resolve_current_binding(
            &self,
            _demand: &HostInfrastructureDemandV2,
        ) -> Result<CurrentInfrastructureBindingV2, AdmissionReceiveError> {
            self.events.push("resolve");
            if self.fail {
                Err(AdmissionReceiveError::Unauthorized)
            } else {
                Ok(self.current.clone())
            }
        }
    }

    struct FakeInbox {
        events: Events,
        outcome: InboxReceiveOutcome,
        fail: bool,
    }

    #[async_trait]
    impl InfrastructureDemandInboxPort for FakeInbox {
        async fn receive_demand(
            &self,
            _demand: &HostInfrastructureDemandV2,
            _current: &CurrentInfrastructureBindingV2,
        ) -> Result<InboxReceiveOutcome, AdmissionReceiveError> {
            self.events.push("commit");
            if self.fail {
                Err(AdmissionReceiveError::OwnerCommit)
            } else {
                Ok(self.outcome)
            }
        }
    }

    struct FakeQuarantine {
        events: Events,
        fail: bool,
    }

    impl AdmissionReceiveQuarantine for FakeQuarantine {
        fn record<'a>(
            &'a self,
            _origin: &'a AdmissionReceiveOrigin,
            _wire: &'a [u8],
            _fault: AdmissionReceivePoisonFault,
        ) -> Pin<Box<dyn Future<Output = Result<(), AdmissionReceiveError>> + Send + 'a>> {
            Box::pin(async move {
                self.events.push("quarantine");
                if self.fail {
                    Err(AdmissionReceiveError::Quarantine)
                } else {
                    Ok(())
                }
            })
        }
    }

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn budget() -> ExecutionResourceBudgetV1 {
        ExecutionResourceBudgetV1 {
            max_rss_bytes: 256 * 1024 * 1024,
            max_cpu_ms: 30_000,
            max_runtime_ms: 60_000,
            max_child_processes: 4,
            max_parallel_tools: 2,
            max_provider_calls: 12,
            max_output_bytes: 64 * 1024,
            max_event_buffer_bytes: 16 * 1024,
        }
    }

    fn demand() -> HostInfrastructureDemandV2 {
        let resource_budget = budget();
        let binding = RunInfrastructureBindingSnapshotV2 {
            schema_version: 2,
            tenant_id: id(1),
            project_id: id(2),
            branch_id: id(3),
            engineering_run_id: id(4),
            binding_id: id(5),
            binding_revision: 1,
            host_environment_id: id(6),
            k3s_cluster_id: id(7),
            k3s_cluster_generation: 2,
            cluster_placement_policy: ClusterPlacementPolicyV1::PinnedCluster,
            provider: ProviderKind::MultipassK3sCluster,
            provider_profile_id: id(8),
            provider_profile_revision: 1,
            provider_profile_digest: [8; 32],
            namespace: KubernetesDnsLabel::new("run-1").expect("valid namespace"),
            service_account: KubernetesDnsLabel::new("agent").expect("valid service account"),
            runtime_profile_id: id(9),
            runtime_profile_revision: 1,
            runtime_profile_digest: [9; 32],
            resource_budget,
        };
        let now = Utc::now();
        HostInfrastructureDemandV2 {
            schema_version: 2,
            event_id: id(10),
            correlation_id: id(11),
            tenant_id: binding.tenant_id,
            project_id: binding.project_id,
            branch_id: binding.branch_id,
            engineering_run_id: binding.engineering_run_id,
            task_execution_run_id: id(12),
            schedule_rule_id: id(13),
            schedule_rule_version: 1,
            occurrence_id: id(14),
            binding,
            resource_budget,
            requested_at: now,
            deadline_at: now + ChronoDuration::minutes(5),
        }
    }

    fn scope() -> AdmissionReceiveScope {
        AdmissionReceiveScope {
            tenant_id: id(1),
            sender_service_id: id(20),
            recipient_service_id: id(21),
            kind: AdmissionReceiveKind::Request,
        }
    }

    fn origin(demand: &HostInfrastructureDemandV2) -> AdmissionReceiveOrigin {
        AdmissionReceiveOrigin {
            scope: scope(),
            message_id: demand.event_id,
            provider_identity: "native-test-provider-v1".to_owned(),
            log_id: "log-1".to_owned(),
            log_generation: id(22),
            sequence: 1,
        }
    }

    fn binding(demand: &HostInfrastructureDemandV2) -> CurrentInfrastructureBindingV2 {
        CurrentInfrastructureBindingV2 {
            binding: demand.binding.clone(),
            host_generation: 3,
        }
    }

    fn build_receiver(
        demand: &HostInfrastructureDemandV2,
        events: Events,
        current: CurrentInfrastructureBindingV2,
        resolver_fail: bool,
        inbox_fail: bool,
        outcome: InboxReceiveOutcome,
    ) -> InfrastructureDemandReceiver<FakeTransport, FakeInbox, FakeAuthorization> {
        let delivery = FakeDelivery {
            wire: serde_json::to_vec(demand).expect("demand JSON"),
            origin: origin(demand),
            events: events.clone(),
        };
        let resolver: Arc<dyn CurrentInfrastructureBindingResolver> = Arc::new(FakeResolver {
            events: events.clone(),
            current,
            fail: resolver_fail,
        });
        InfrastructureDemandReceiver::new(
            FakeTransport(Mutex::new(Some(delivery)), events.clone()),
            FakeInbox {
                events: events.clone(),
                outcome,
                fail: inbox_fail,
            },
            FakeAuthorization {
                events,
                deny_source: false,
            },
            scope(),
            Some(resolver),
        )
        .expect("valid request scope")
    }

    async fn consume(
        receiver: &mut InfrastructureDemandReceiver<FakeTransport, FakeInbox, FakeAuthorization>,
    ) -> Result<Option<InboxReceiveOutcome>, AdmissionReceiveError> {
        let (_cancel, mut cancellation) = watch::channel(false);
        receiver.consume_one(&mut cancellation).await
    }

    #[test]
    fn only_request_scope_is_accepted() {
        assert!(validate_request_scope(scope()).is_ok());
        let mut invalid = scope();
        invalid.kind = AdmissionReceiveKind::Result;
        assert_eq!(
            validate_request_scope(invalid),
            Err(AdmissionReceiveError::Contract)
        );
        let mut invalid = scope();
        invalid.sender_service_id = invalid.recipient_service_id;
        assert_eq!(
            validate_request_scope(invalid),
            Err(AdmissionReceiveError::Contract)
        );
    }

    #[test]
    fn demand_wire_requires_schema_scope_and_origin_identity() {
        let mut demand = demand();
        let valid_origin = origin(&demand);
        let wire = serde_json::to_vec(&demand).expect("wire");
        assert_eq!(
            decode_demand(scope(), &valid_origin, &wire),
            Ok(demand.clone())
        );

        demand.tenant_id = id(999);
        demand.binding.tenant_id = id(999);
        let wrong_tenant = serde_json::to_vec(&demand).expect("wire");
        assert_eq!(
            decode_demand(scope(), &valid_origin, &wrong_tenant),
            Err(AdmissionReceivePoisonFault::IdentityMismatch)
        );

        let mut wrong_origin = valid_origin.clone();
        wrong_origin.message_id = id(1000);
        assert_eq!(
            decode_demand(scope(), &wrong_origin, &wire),
            Err(AdmissionReceivePoisonFault::IdentityMismatch)
        );
        let mut wrong_route = valid_origin.clone();
        wrong_route.scope.kind = AdmissionReceiveKind::Result;
        assert_eq!(
            validate_origin(scope(), &wrong_route),
            Err(AdmissionReceivePoisonFault::IdentityMismatch)
        );
        assert_eq!(
            decode_demand(scope(), &valid_origin, &[b' '; MAX_DEMAND_WIRE_BYTES + 1]),
            Err(AdmissionReceivePoisonFault::MalformedWire)
        );
        assert_eq!(
            decode_demand(scope(), &valid_origin, b"{\"unknown\":true}"),
            Err(AdmissionReceivePoisonFault::MalformedWire)
        );
    }

    #[tokio::test]
    async fn current_binding_drift_fails_closed() {
        let demand = demand();
        let mut current = binding(&demand);
        assert!(validate_current_binding(&demand, &current).is_ok());
        current.binding.k3s_cluster_generation += 1;
        assert_eq!(
            validate_current_binding(&demand, &current),
            Err(AdmissionReceiveError::Unauthorized)
        );

        let mut current = binding(&demand);
        current.binding.k3s_cluster_generation += 1;
        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let mut receiver = build_receiver(
            &demand,
            events.clone(),
            current,
            false,
            false,
            InboxReceiveOutcome::Enqueued { inbox_id: id(36) },
        );
        assert_eq!(
            consume(&mut receiver).await,
            Err(AdmissionReceiveError::Unauthorized)
        );
        assert!(!events.snapshot().contains(&"commit"));
        assert!(!events.snapshot().contains(&"ack"));
        current = binding(&demand);
        current.host_generation = 0;
        assert_eq!(
            validate_current_binding(&demand, &current),
            Err(AdmissionReceiveError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn resolver_and_store_errors_leave_delivery_unacked() {
        let demand = demand();
        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let mut receiver = build_receiver(
            &demand,
            events.clone(),
            binding(&demand),
            true,
            false,
            InboxReceiveOutcome::DuplicatePending { inbox_id: id(30) },
        );
        assert_eq!(
            consume(&mut receiver).await,
            Err(AdmissionReceiveError::Unauthorized)
        );
        assert!(!events.snapshot().contains(&"ack"));

        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let mut receiver = build_receiver(
            &demand,
            events.clone(),
            binding(&demand),
            false,
            true,
            InboxReceiveOutcome::DuplicatePending { inbox_id: id(30) },
        );
        assert_eq!(
            consume(&mut receiver).await,
            Err(AdmissionReceiveError::OwnerCommit)
        );
        assert!(!events.snapshot().contains(&"ack"));
    }

    #[tokio::test]
    async fn missing_binding_resolver_fails_closed_before_pull() {
        let demand = demand();
        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let delivery = FakeDelivery {
            wire: serde_json::to_vec(&demand).expect("demand JSON"),
            origin: origin(&demand),
            events: events.clone(),
        };
        let mut receiver = InfrastructureDemandReceiver::new(
            FakeTransport(Mutex::new(Some(delivery)), events.clone()),
            FakeInbox {
                events: events.clone(),
                outcome: InboxReceiveOutcome::Enqueued { inbox_id: id(35) },
                fail: false,
            },
            FakeAuthorization {
                events: events.clone(),
                deny_source: false,
            },
            scope(),
            None,
        )
        .expect("valid scope");
        assert_eq!(
            consume(&mut receiver).await,
            Err(AdmissionReceiveError::Unauthorized)
        );
        assert!(events.snapshot().is_empty());
    }

    #[tokio::test]
    async fn owner_commit_and_source_revalidation_precede_ack() {
        let demand = demand();
        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let mut receiver = build_receiver(
            &demand,
            events.clone(),
            binding(&demand),
            false,
            false,
            InboxReceiveOutcome::Enqueued { inbox_id: id(31) },
        );
        assert_eq!(
            consume(&mut receiver).await,
            Ok(Some(InboxReceiveOutcome::Enqueued { inbox_id: id(31) }))
        );
        assert_eq!(
            events.snapshot(),
            ["pull", "verify", "resolve", "commit", "verify", "ack"]
        );
    }

    #[tokio::test]
    async fn pending_terminal_and_replay_outcomes_are_acknowledged_after_commit() {
        let outcomes = [
            InboxReceiveOutcome::Enqueued { inbox_id: id(32) },
            InboxReceiveOutcome::DuplicatePending { inbox_id: id(33) },
            InboxReceiveOutcome::AlreadyTerminal {
                outcome: crate::infrastructure_owner_inbox::TerminalDemandOutcome::Accepted,
            },
            InboxReceiveOutcome::AlreadyTerminal {
                outcome: crate::infrastructure_owner_inbox::TerminalDemandOutcome::Rejected,
            },
        ];
        for outcome in outcomes {
            let demand = demand();
            let events = Events(Arc::new(Mutex::new(Vec::new())));
            let mut receiver = build_receiver(
                &demand,
                events.clone(),
                binding(&demand),
                false,
                false,
                outcome,
            );
            assert_eq!(consume(&mut receiver).await, Ok(Some(outcome)));
            assert_eq!(events.snapshot().last(), Some(&"ack"));
        }
    }

    fn poison_receiver(
        demand: &HostInfrastructureDemandV2,
        events: Events,
        quarantine: Option<Arc<dyn AdmissionReceiveQuarantine>>,
    ) -> InfrastructureDemandReceiver<FakeTransport, FakeInbox, FakeAuthorization> {
        let mut delivery = FakeDelivery {
            wire: b"not-json".to_vec(),
            origin: origin(demand),
            events: events.clone(),
        };
        delivery.origin.message_id = id(999);
        let resolver: Arc<dyn CurrentInfrastructureBindingResolver> = Arc::new(FakeResolver {
            events: events.clone(),
            current: binding(demand),
            fail: false,
        });
        InfrastructureDemandReceiver {
            transport: FakeTransport(Mutex::new(Some(delivery)), events.clone()),
            inbox: FakeInbox {
                events: events.clone(),
                outcome: InboxReceiveOutcome::Enqueued { inbox_id: id(34) },
                fail: false,
            },
            authorization: FakeAuthorization {
                events,
                deny_source: false,
            },
            scope: scope(),
            resolver: Some(resolver),
            quarantine,
        }
    }

    #[tokio::test]
    async fn poison_requires_durable_quarantine_before_termination() {
        let demand = demand();
        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let quarantine: Arc<dyn AdmissionReceiveQuarantine> = Arc::new(FakeQuarantine {
            events: events.clone(),
            fail: false,
        });
        let mut receiver = poison_receiver(&demand, events.clone(), Some(quarantine));
        assert_eq!(consume(&mut receiver).await, Ok(None));
        assert_eq!(
            events.snapshot(),
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
    async fn quarantine_failure_and_absence_leave_poison_unacked() {
        let demand = demand();
        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let quarantine: Arc<dyn AdmissionReceiveQuarantine> = Arc::new(FakeQuarantine {
            events: events.clone(),
            fail: true,
        });
        let mut receiver = poison_receiver(&demand, events.clone(), Some(quarantine));
        assert_eq!(
            consume(&mut receiver).await,
            Err(AdmissionReceiveError::Quarantine)
        );
        assert!(!events.snapshot().contains(&"terminate"));
        assert!(!events.snapshot().contains(&"ack"));

        let events = Events(Arc::new(Mutex::new(Vec::new())));
        let mut receiver = poison_receiver(&demand, events.clone(), None);
        assert_eq!(
            consume(&mut receiver).await,
            Err(AdmissionReceiveError::Quarantine)
        );
        assert!(!events.snapshot().contains(&"verify_source"));
        assert!(!events.snapshot().contains(&"terminate"));
        assert!(!events.snapshot().contains(&"ack"));
    }
}
