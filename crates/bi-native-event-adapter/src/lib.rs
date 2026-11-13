// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=59eef8f21880e93cd78fb2cfd15ce3b74fd5cd81b7c484dbdcf547e61f7690fc
MERGE (self:File {path:"crates/bi-native-event-adapter/src/lib.rs"})
MERGE (config:Type {id:"bi_native_event_adapter::NativeTaskEventReceiverConfig",kind:"struct"})
MERGE (receiver:Type {id:"bi_native_event_adapter::NativeTaskEventReceiver",kind:"struct"})
MERGE (transport:Type {id:"bi_native_event_adapter::NativeTaskEventTransport",kind:"struct"})
MERGE (authorization:Type {id:"bi_native_event_adapter::NativeTaskEventAuthorization",kind:"struct"})
MERGE (delivery:Type {id:"bi_native_event_adapter::NativeTaskEventDelivery",kind:"struct"})
MERGE (origin_slot:Type {id:"bi_native_event_adapter::NativeOriginSlot",kind:"struct"})
MERGE (pending_origin:Type {id:"bi_native_event_adapter::PendingNativeOrigin",kind:"struct"})
MERGE (origin_key:Type {id:"bi_native_event_adapter::OriginKey",kind:"struct"})
MERGE (adapter_error:Type {id:"bi_native_event_adapter::NativeTaskEventAdapterError",kind:"enum"})
MERGE (connect:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventReceiver.connect",kind:"method"})
MERGE (receive_one:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventReceiver.receive_one",kind:"method"})
MERGE (pull_one:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventTransport.pull_one",kind:"method"})
MERGE (verify_origin:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventAuthorization.verify_origin",kind:"method"})
MERGE (exact_wire:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventDelivery.exact_wire",kind:"method"})
MERGE (delivery_origin:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventDelivery.origin",kind:"method"})
MERGE (origin_sequence:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventSourceOrigin.source_sequence",kind:"method"})
MERGE (acknowledge:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventDelivery.acknowledge",kind:"method"})
MERGE (terminate:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventDelivery.terminate_permanently",kind:"method"})
MERGE (drop_delivery:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeTaskEventDelivery.drop",kind:"method"})
MERGE (slot_insert:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeOriginSlot.insert",kind:"method"})
MERGE (slot_resolve:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeOriginSlot.resolve",kind:"method"})
MERGE (slot_clear:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::NativeOriginSlot.clear_if_matches",kind:"method"})
MERGE (key_from_origin:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::OriginKey.from_origin",kind:"method"})
MERGE (key_matches_native:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::OriginKey.matches_native",kind:"method"})
MERGE (route_matches:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::route_matches",kind:"function"})
MERGE (validate_binding:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::validate_route_binding",kind:"function"})
MERGE (map_error:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::map_native_error",kind:"function"})
MERGE (test_binding:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.route_binding_requires_profile_and_service_identity_pins",kind:"test"})
MERGE (test_route:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.route_comparison_includes_tenant_and_every_route_pin",kind:"test"})
MERGE (test_origin_key:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.origin_key_changes_with_each_durable_identity_component",kind:"test"})
MERGE (route_fixture:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.route_fixture",kind:"function"})
MERGE (profile_fixture:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.profile_fixture",kind:"function"})
MERGE (forward_owner_receive:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::forward_owner_receive",kind:"function"})
MERGE (test_legacy_origin:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.legacy_origin_error_propagates_without_terminal_ack",kind:"test"})
MERGE (test_map_error:Symbol {id:"crates/bi-native-event-adapter/src/lib.rs::tests.map_native_error_preserves_auth_and_retryable_transport",kind:"test"})
MERGE (disposition_log:Type {id:"bi_native_event_adapter::tests::DispositionLog",kind:"struct"})
MERGE (legacy_delivery:Type {id:"bi_native_event_adapter::tests::LegacyOriginDelivery",kind:"struct"})
MERGE (legacy_transport:Type {id:"bi_native_event_adapter::tests::LegacyOriginTransport",kind:"struct"})
MERGE (allow_origin:Type {id:"bi_native_event_adapter::tests::AllowOrigin",kind:"struct"})
MERGE (legacy_inbox:Type {id:"bi_native_event_adapter::tests::LegacyOriginInbox",kind:"struct"})
MERGE (reject_quarantine:Type {id:"bi_native_event_adapter::tests::RejectUnexpectedQuarantine",kind:"struct"})
MERGE (self)-[:DEFINES]->(config)
MERGE (self)-[:DEFINES]->(receiver)
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(authorization)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(origin_slot)
MERGE (self)-[:DEFINES]->(pending_origin)
MERGE (self)-[:DEFINES]->(origin_key)
MERGE (self)-[:DEFINES]->(adapter_error)
MERGE (receiver)-[:DEFINES]->(connect)
MERGE (receiver)-[:DEFINES]->(receive_one)
MERGE (transport)-[:DEFINES]->(pull_one)
MERGE (authorization)-[:DEFINES]->(verify_origin)
MERGE (delivery)-[:DEFINES]->(exact_wire)
MERGE (delivery)-[:DEFINES]->(delivery_origin)
MERGE (delivery)-[:DEFINES]->(acknowledge)
MERGE (delivery)-[:DEFINES]->(terminate)
MERGE (delivery)-[:DEFINES]->(drop_delivery)
MERGE (origin_slot)-[:DEFINES]->(slot_insert)
MERGE (origin_slot)-[:DEFINES]->(slot_resolve)
MERGE (origin_slot)-[:DEFINES]->(slot_clear)
MERGE (origin_key)-[:DEFINES]->(key_from_origin)
MERGE (origin_key)-[:DEFINES]->(key_matches_native)
MERGE (self)-[:DEFINES]->(route_matches)
MERGE (self)-[:DEFINES]->(validate_binding)
MERGE (self)-[:DEFINES]->(map_error)
MERGE (self)-[:DEFINES]->(forward_owner_receive)
MERGE (self)-[:IMPORTS]->(:File {path:"crates/bi-owner-runtime/src/task_event_receiver.rs"})
MERGE (self)-[:IMPORTS]->(:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
MERGE (connect)-[:CALLS]->(validate_binding)
MERGE (connect)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventReceiveTransport.connect",kind:"method"})
MERGE (connect)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.new",kind:"method"})
MERGE (receive_one)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.receive_one",kind:"method"})
MERGE (receive_one)-[:CALLS]->(forward_owner_receive)
MERGE (pull_one)-[:CALLS]->(route_matches)
MERGE (pull_one)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventReceiveTransport.pull_one",kind:"method"})
MERGE (pull_one)-[:CALLS]->(slot_insert)
MERGE (verify_origin)-[:CALLS]->(route_matches)
MERGE (verify_origin)-[:CALLS]->(key_from_origin)
MERGE (verify_origin)-[:CALLS]->(slot_resolve)
MERGE (verify_origin)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventReceiveAuthorization.verify_origin",kind:"method"})
MERGE (exact_wire)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventDelivery.exact_wire",kind:"method"})
MERGE (delivery_origin)-[:USES_TYPE]->(:Type {id:"bi_owner_runtime::TaskEventSourceOrigin",kind:"struct"})
MERGE (acknowledge)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventDelivery.acknowledge",kind:"method"})
MERGE (terminate)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventDelivery.acknowledge",kind:"method"})
MERGE (drop_delivery)-[:CALLS]->(slot_clear)
MERGE (slot_insert)-[:WRITES]->(:Config {id:"single_pending_native_origin"})
MERGE (slot_resolve)-[:READS]->(:Config {id:"single_pending_native_origin"})
MERGE (slot_clear)-[:WRITES]->(:Config {id:"single_pending_native_origin"})
MERGE (key_from_origin)-[:USES_TYPE]->(:Type {id:"bi_owner_runtime::TaskEventSourceOrigin",kind:"struct"})
MERGE (key_from_origin)-[:CALLS]->(origin_sequence)
MERGE (key_matches_native)-[:USES_TYPE]->(:Type {id:"platform_messaging_runtime::NativeGrpcEventOrigin",kind:"struct"})
MERGE (validate_binding)-[:USES_TYPE]->(:Type {id:"bi_owner_runtime::TaskEventRoutePin",kind:"struct"})
MERGE (validate_binding)-[:USES_TYPE]->(:Type {id:"platform_messaging_runtime::ResolvedTransportEndpoint",kind:"struct"})
MERGE (validate_binding)-[:USES_TYPE]->(:Type {id:"platform_messaging_runtime::TransportProfileRevision",kind:"struct"})
MERGE (connect)-[:USES_TYPE]->(:Type {id:"platform_messaging_runtime::AdmissionTransportProfile",kind:"struct"})
MERGE (self)-[:IMPORTS]->(:File {path:"crates/platform-messaging-runtime/src/native_append.rs"})
MERGE (self)-[:IMPORTS]->(:File {path:"crates/platform-messaging-runtime/src/transport_endpoint.rs"})
MERGE (self)-[:IMPORTS]->(:File {path:"crates/platform-messaging-runtime/src/transport_profile.rs"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"std::sync::Arc",kind:"type"})
MERGE (map_error)-[:USES_TYPE]->(:Type {id:"platform_messaging_runtime::NativeGrpcEventReceiveError",kind:"enum"})
MERGE (map_error)-[:USES_TYPE]->(:Type {id:"bi_owner_runtime::TaskEventReceiveError",kind:"enum"})
MERGE (connect)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventReceiveTransport.authorization",kind:"method"})
MERGE (connect)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile.new",kind:"method"})
MERGE (pull_one)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventSourceOrigin.new",kind:"method"})
MERGE (verify_origin)-[:CALLS]->(key_matches_native)
MERGE (verify_origin)-[:CALLS]->(map_error)
MERGE (acknowledge)-[:CALLS]->(map_error)
MERGE (terminate)-[:CALLS]->(map_error)
MERGE (validate_binding)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcEventReceiveScope.new",kind:"method"})
MERGE (self)-[:DEFINES]->(test_binding)
MERGE (self)-[:DEFINES]->(test_route)
MERGE (self)-[:DEFINES]->(test_origin_key)
MERGE (self)-[:DEFINES]->(test_legacy_origin)
MERGE (self)-[:DEFINES]->(disposition_log)
MERGE (self)-[:DEFINES]->(legacy_delivery)
MERGE (self)-[:DEFINES]->(legacy_transport)
MERGE (self)-[:DEFINES]->(allow_origin)
MERGE (self)-[:DEFINES]->(legacy_inbox)
MERGE (self)-[:DEFINES]->(reject_quarantine)
MERGE (self)-[:DEFINES]->(route_fixture)
MERGE (self)-[:DEFINES]->(profile_fixture)
MERGE (test_binding)-[:TESTS]->(validate_binding)
MERGE (test_route)-[:TESTS]->(route_matches)
MERGE (test_origin_key)-[:TESTS]->(key_from_origin)
MERGE (test_legacy_origin)-[:CALLS]->(forward_owner_receive)
MERGE (test_map_error)-[:TESTS]->(map_error)
MERGE (test_legacy_origin)-[:TESTS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiver.process_one",kind:"method"})
MERGE (test_legacy_origin)-[:USES_TYPE]->(:Type {id:"bi_owner_runtime::TaskEventInboxError",kind:"enum"})
MERGE (test_legacy_origin)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveRoute.new",kind:"method"})
MERGE (test_legacy_origin)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventSourceOrigin.new",kind:"method"})
MERGE (legacy_delivery)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveDelivery",kind:"trait"})
MERGE (legacy_transport)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveTransport",kind:"trait"})
MERGE (allow_origin)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventReceiveAuthorization",kind:"trait"})
MERGE (legacy_inbox)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventInboxCommitPort",kind:"trait"})
MERGE (reject_quarantine)-[:IMPLEMENTS]->(:Type {id:"bi_owner_runtime::TaskEventQuarantinePort",kind:"trait"})
MERGE (test_binding)-[:CALLS]->(route_fixture)
MERGE (test_binding)-[:CALLS]->(profile_fixture)
MERGE (test_route)-[:CALLS]->(route_fixture)
MERGE (test_route)-[:CALLS]->(profile_fixture)
MERGE (route_fixture)-[:CALLS]->(:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::TaskEventReceiveRoute.new",kind:"method"})
MERGE (profile_fixture)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileDefinition.new",kind:"method"})
MERGE (profile_fixture)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision.new",kind:"method"})
MERGE (profile_fixture)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointConfig.new",kind:"method"})
MERGE (profile_fixture)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.new",kind:"method"})
MERGE (profile_fixture)-[:CALLS]->(:Symbol {id:"crates/platform-messaging-runtime/src/transport_endpoint.rs::TrustedEndpointResolver.resolve",kind:"method"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"thiserror::Error",kind:"macro"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"uuid::Uuid",kind:"type"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"tokio::time::Instant",kind:"type"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"tokio_util::sync::CancellationToken",kind:"type"})
MERGE (self)-[:IMPORTS]->(:ExternalService {id:"std::sync::Mutex",kind:"type"})
@endcypher
*/
//! Profile-pinned bridge from the native durable event receiver into BI's owner transaction.

use std::sync::{Arc, Mutex};

use bi_owner_runtime::{
    TaskEventInboxCommitPort, TaskEventQuarantinePort, TaskEventReceiveAuthorization,
    TaskEventReceiveDelivery, TaskEventReceiveError, TaskEventReceiveOrigin,
    TaskEventReceiveOutcome, TaskEventReceiveRoute, TaskEventReceiveTransport, TaskEventReceiver,
};
use platform_messaging_runtime::{
    admission_delivery::AdmissionTransportProfile,
    native_append::MAX_NATIVE_APPEND_WIRE_BYTES,
    native_grpc::{
        NativeGrpcEventDelivery, NativeGrpcEventOrigin, NativeGrpcEventReceiveAuthorization,
        NativeGrpcEventReceiveClientConfig, NativeGrpcEventReceiveError,
        NativeGrpcEventReceiveScope, NativeGrpcEventReceiveTransport, NativeGrpcSetupError,
    },
    transport_endpoint::ResolvedTransportEndpoint,
    transport_profile::{
        TransportMessageKind, TransportProfileLifecycle, TransportProfileRevision,
    },
};
use thiserror::Error;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Trusted construction inputs for a single BI receiver bound to one immutable route.
pub struct NativeTaskEventReceiverConfig<'a> {
    /// BI tenant and immutable provider/profile/destination pin.
    pub route: TaskEventReceiveRoute,
    /// Deployment-trusted endpoint resolved from the same current profile revision.
    pub endpoint: &'a ResolvedTransportEndpoint,
    /// Trusted CA bundle used to validate the native server certificate.
    pub server_ca_path: &'a std::path::Path,
    /// Client certificate for the receiving service's mTLS identity.
    pub client_cert_path: &'a std::path::Path,
    /// Private key corresponding to `client_cert_path`.
    pub client_key_path: &'a std::path::Path,
    /// Connection timeout, further capped by the native client.
    pub connect_timeout: std::time::Duration,
    /// Per-RPC timeout, further capped by the native client.
    pub request_timeout: std::time::Duration,
    /// Bounded native source lease.
    pub lease_seconds: u32,
    /// Full trusted revision used to prove tenant, service, message-kind, and digest pins.
    pub profile: &'a TransportProfileRevision,
}

/// Adapter setup failed before a BI receiver was made available.
#[derive(Debug, Error)]
pub enum NativeTaskEventAdapterError {
    /// BI, endpoint, profile, and native destination identities did not all match.
    #[error("BI route does not match the native transport profile and destination")]
    RouteBinding,
    /// The native client rejected its endpoint, TLS material, or bounded timeout configuration.
    #[error(transparent)]
    NativeSetup(#[from] NativeGrpcSetupError),
}

/// Explicitly driven receiver. Construction connects one pinned native client and starts no worker.
pub struct NativeTaskEventReceiver<I, Q> {
    inner: TaskEventReceiver<NativeTaskEventTransport, NativeTaskEventAuthorization, I, Q>,
}

impl<I, Q> NativeTaskEventReceiver<I, Q>
where
    I: TaskEventInboxCommitPort,
    Q: TaskEventQuarantinePort,
{
    /// Validate every route pin before connecting, then assemble the existing BI owner receiver.
    pub async fn connect(
        config: NativeTaskEventReceiverConfig<'_>,
        inbox: I,
        quarantine: Q,
    ) -> Result<Self, NativeTaskEventAdapterError> {
        let NativeTaskEventReceiverConfig {
            route,
            endpoint,
            server_ca_path,
            client_cert_path,
            client_key_path,
            connect_timeout,
            request_timeout,
            lease_seconds,
            profile,
        } = config;
        let scope = validate_route_binding(&route, endpoint, profile)?;
        let native_profile = AdmissionTransportProfile::new(
            profile.definition().provider_id().to_owned(),
            profile.definition().provider_version().to_owned(),
            profile.digest(),
            profile.definition().endpoint_identity().to_owned(),
        )
        .map_err(|_| NativeTaskEventAdapterError::RouteBinding)?;
        let pin = route.pin();
        let native = NativeGrpcEventReceiveTransport::connect(NativeGrpcEventReceiveClientConfig {
            endpoint,
            server_ca_path,
            client_cert_path,
            client_key_path,
            connect_timeout,
            request_timeout,
            scope,
            expected_log_id: &pin.destination_id,
            expected_log_generation: pin.destination_generation,
            lease_seconds,
            profile: native_profile,
        })
        .await?;
        let expected_native_provider_identity = native.provider_identity().to_owned();
        let native_authorization = native.authorization();
        let pending_origin = Arc::new(NativeOriginSlot::default());
        let transport = NativeTaskEventTransport {
            route: route.clone(),
            scope,
            expected_native_provider_identity: expected_native_provider_identity.clone(),
            native,
            pending_origin: pending_origin.clone(),
        };
        let authorization = NativeTaskEventAuthorization {
            route: route.clone(),
            scope,
            expected_native_provider_identity,
            native: native_authorization,
            pending_origin,
        };
        Ok(Self {
            inner: TaskEventReceiver::new(route, transport, authorization, inbox, quarantine),
        })
    }

    /// Process at most one delivery with caller-owned cancellation and deadline.
    pub async fn receive_one(
        &self,
        cancellation: &CancellationToken,
        deadline: Instant,
    ) -> Result<TaskEventReceiveOutcome, TaskEventReceiveError> {
        forward_owner_receive(&self.inner, cancellation, deadline).await
    }
}

async fn forward_owner_receive<T, A, I, Q>(
    inner: &TaskEventReceiver<T, A, I, Q>,
    cancellation: &CancellationToken,
    deadline: Instant,
) -> Result<TaskEventReceiveOutcome, TaskEventReceiveError>
where
    T: TaskEventReceiveTransport,
    A: TaskEventReceiveAuthorization,
    I: TaskEventInboxCommitPort,
    Q: TaskEventQuarantinePort,
{
    inner.receive_one(cancellation, deadline).await
}

struct NativeTaskEventTransport {
    route: TaskEventReceiveRoute,
    scope: NativeGrpcEventReceiveScope,
    expected_native_provider_identity: String,
    native: NativeGrpcEventReceiveTransport,
    pending_origin: Arc<NativeOriginSlot>,
}

impl TaskEventReceiveTransport for NativeTaskEventTransport {
    type Delivery = NativeTaskEventDelivery;

    async fn pull_one(
        &self,
        route: &TaskEventReceiveRoute,
    ) -> Result<Option<Self::Delivery>, TaskEventReceiveError> {
        if !route_matches(route, &self.route) {
            return Err(TaskEventReceiveError::InvalidRoute);
        }
        let Some(native) = self
            .native
            .pull_one(self.scope)
            .await
            .map_err(map_native_error)?
        else {
            return Ok(None);
        };
        let source = native.origin();
        if source.scope() != self.scope
            || source.provider_identity() != self.expected_native_provider_identity
            || source.log_id() != self.route.pin().destination_id
            || source.log_generation() != self.route.pin().destination_generation
            || native.exact_wire().is_empty()
            || native.exact_wire().len() > MAX_NATIVE_APPEND_WIRE_BYTES
        {
            return Err(TaskEventReceiveError::InvalidOrigin);
        }
        let origin = TaskEventReceiveOrigin::new(
            self.route.pin().transport_provider_id.clone(),
            source.log_id().to_owned(),
            source.log_generation(),
            source.sequence(),
            source.sequence().to_be_bytes().to_vec(),
            source.message_id(),
        )?;
        let key = OriginKey::from_origin(&origin);
        self.pending_origin.insert(PendingNativeOrigin {
            key: key.clone(),
            origin: source.clone(),
        })?;
        Ok(Some(NativeTaskEventDelivery {
            native,
            origin,
            key,
            pending_origin: self.pending_origin.clone(),
        }))
    }
}

struct NativeTaskEventAuthorization {
    route: TaskEventReceiveRoute,
    scope: NativeGrpcEventReceiveScope,
    expected_native_provider_identity: String,
    native: NativeGrpcEventReceiveAuthorization,
    pending_origin: Arc<NativeOriginSlot>,
}

impl TaskEventReceiveAuthorization for NativeTaskEventAuthorization {
    async fn verify_origin(
        &self,
        route: &TaskEventReceiveRoute,
        origin: &TaskEventReceiveOrigin,
        exact_wire: &[u8],
    ) -> Result<(), TaskEventReceiveError> {
        if !route_matches(route, &self.route)
            || origin.provider_identity() != self.route.pin().transport_provider_id
            || exact_wire.is_empty()
            || exact_wire.len() > MAX_NATIVE_APPEND_WIRE_BYTES
        {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        let key = OriginKey::from_origin(origin);
        let native_origin = self.pending_origin.resolve(&key)?;
        if !key.matches_native(&native_origin)
            || native_origin.scope() != self.scope
            || native_origin.provider_identity() != self.expected_native_provider_identity
        {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        self.native
            .verify_origin(self.scope, &native_origin, exact_wire)
            .await
            .map_err(map_native_error)
    }
}

struct NativeTaskEventDelivery {
    native: NativeGrpcEventDelivery,
    origin: TaskEventReceiveOrigin,
    key: OriginKey,
    pending_origin: Arc<NativeOriginSlot>,
}

impl TaskEventReceiveDelivery for NativeTaskEventDelivery {
    fn exact_wire(&self) -> &[u8] {
        self.native.exact_wire()
    }

    fn origin(&self) -> &TaskEventReceiveOrigin {
        &self.origin
    }

    async fn acknowledge(&self) -> Result<(), TaskEventReceiveError> {
        self.native.acknowledge().await.map_err(map_native_error)
    }

    async fn terminate_permanently(&self) -> Result<(), TaskEventReceiveError> {
        // The native protocol has one terminal ACK operation. BI calls this only after metadata-only
        // quarantine commits and source authorization is rechecked; a failed quarantine never ACKs.
        self.native.acknowledge().await.map_err(map_native_error)
    }
}

impl Drop for NativeTaskEventDelivery {
    fn drop(&mut self) {
        self.pending_origin.clear_if_matches(&self.key);
    }
}

#[derive(Default)]
struct NativeOriginSlot(Mutex<Option<PendingNativeOrigin>>);

struct PendingNativeOrigin {
    key: OriginKey,
    origin: NativeGrpcEventOrigin,
}

impl NativeOriginSlot {
    fn insert(&self, pending: PendingNativeOrigin) -> Result<(), TaskEventReceiveError> {
        let mut slot = self
            .0
            .lock()
            .map_err(|_| TaskEventReceiveError::Transport)?;
        if slot.is_some() {
            return Err(TaskEventReceiveError::Busy);
        }
        *slot = Some(pending);
        Ok(())
    }

    fn resolve(&self, key: &OriginKey) -> Result<NativeGrpcEventOrigin, TaskEventReceiveError> {
        let slot = self
            .0
            .lock()
            .map_err(|_| TaskEventReceiveError::Transport)?;
        slot.as_ref()
            .filter(|pending| pending.key == *key)
            .map(|pending| pending.origin.clone())
            .ok_or(TaskEventReceiveError::Unauthorized)
    }

    fn clear_if_matches(&self, key: &OriginKey) {
        let Ok(mut slot) = self.0.lock() else {
            return;
        };
        if slot.as_ref().is_some_and(|pending| pending.key == *key) {
            *slot = None;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OriginKey {
    log_id: String,
    log_generation: Uuid,
    source_sequence: i64,
    cursor: Vec<u8>,
    message_id: Uuid,
}

impl OriginKey {
    fn from_origin(origin: &TaskEventReceiveOrigin) -> Self {
        Self {
            log_id: origin.log_id().to_owned(),
            log_generation: origin.log_generation(),
            source_sequence: origin.source_sequence(),
            cursor: origin.cursor().to_vec(),
            message_id: origin.message_id(),
        }
    }

    fn matches_native(&self, origin: &NativeGrpcEventOrigin) -> bool {
        self.log_id == origin.log_id()
            && self.log_generation == origin.log_generation()
            && self.source_sequence == origin.sequence()
            && self.cursor.as_slice() == origin.sequence().to_be_bytes().as_slice()
            && self.message_id == origin.message_id()
    }
}

fn route_matches(left: &TaskEventReceiveRoute, right: &TaskEventReceiveRoute) -> bool {
    let left_pin = left.pin();
    let right_pin = right.pin();
    left.tenant_id() == right.tenant_id()
        && left_pin.sender_service_id == right_pin.sender_service_id
        && left_pin.receiver_service_id == right_pin.receiver_service_id
        && left_pin.transport_provider_id == right_pin.transport_provider_id
        && left_pin.transport_provider_version == right_pin.transport_provider_version
        && left_pin.transport_profile_id == right_pin.transport_profile_id
        && left_pin.transport_profile_revision == right_pin.transport_profile_revision
        && left_pin.transport_profile_digest == right_pin.transport_profile_digest
        && left_pin.endpoint_identity == right_pin.endpoint_identity
        && left_pin.destination_id == right_pin.destination_id
        && left_pin.destination_generation == right_pin.destination_generation
}

fn validate_route_binding(
    route: &TaskEventReceiveRoute,
    endpoint: &ResolvedTransportEndpoint,
    profile: &TransportProfileRevision,
) -> Result<NativeGrpcEventReceiveScope, NativeTaskEventAdapterError> {
    let pin = route.pin();
    let definition = profile.definition();
    if profile.lifecycle() != TransportProfileLifecycle::Active
        || profile.tenant_id() != route.tenant_id()
        || endpoint.tenant_id() != route.tenant_id()
        || endpoint.deployment_id() != profile.deployment_id()
        || endpoint.profile_id() != profile.profile_id()
        || endpoint.profile_revision() != profile.revision()
        || endpoint.profile_digest() != profile.digest()
        || endpoint.endpoint_identity() != definition.endpoint_identity()
        || endpoint.profile_id() != pin.transport_profile_id
        || endpoint.profile_revision() != pin.transport_profile_revision
        || endpoint.profile_digest() != pin.transport_profile_digest
        || endpoint.endpoint_identity() != pin.endpoint_identity
        || definition.provider_id() != pin.transport_provider_id
        || definition.provider_version() != pin.transport_provider_version
        || definition.endpoint_identity() != pin.endpoint_identity
        || !definition
            .allowed_sender_service_ids()
            .contains(&pin.sender_service_id)
        || !definition
            .allowed_recipient_service_ids()
            .contains(&pin.receiver_service_id)
        || !definition
            .allowed_message_kinds()
            .contains(&TransportMessageKind::Event)
    {
        return Err(NativeTaskEventAdapterError::RouteBinding);
    }
    NativeGrpcEventReceiveScope::new(
        route.tenant_id(),
        pin.sender_service_id,
        pin.receiver_service_id,
    )
    .map_err(|_| NativeTaskEventAdapterError::RouteBinding)
}

fn map_native_error(error: NativeGrpcEventReceiveError) -> TaskEventReceiveError {
    match error {
        NativeGrpcEventReceiveError::Route => TaskEventReceiveError::InvalidRoute,
        NativeGrpcEventReceiveError::Contract => TaskEventReceiveError::InvalidOrigin,
        NativeGrpcEventReceiveError::Unauthorized => TaskEventReceiveError::Unauthorized,
        NativeGrpcEventReceiveError::Transport => TaskEventReceiveError::Transport,
        NativeGrpcEventReceiveError::Busy => TaskEventReceiveError::Busy,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use super::*;
    use bi_owner_runtime::{
        AuthenticatedTaskEventPrincipal, TaskEventInboxError, TaskEventInboxReceipt,
        TaskEventQuarantineFault, TaskEventQuarantineReceipt,
    };
    use platform_messaging_runtime::{
        transport_endpoint::{TrustedEndpointConfig, TrustedEndpointResolver},
        transport_profile::{
            TransportMessageKind, TransportProfileDefinition, TransportProfileLifecycle,
            TransportProfileRevision,
        },
    };

    fn route_fixture(
        endpoint: &ResolvedTransportEndpoint,
        profile: &TransportProfileRevision,
        destination_id: &str,
        destination_generation: Uuid,
    ) -> TaskEventReceiveRoute {
        let pin = bi_owner_runtime::TaskEventRoutePin {
            sender_service_id: Uuid::from_u128(10),
            receiver_service_id: Uuid::from_u128(11),
            transport_provider_id: profile.definition().provider_id().to_owned(),
            transport_provider_version: profile.definition().provider_version().to_owned(),
            transport_profile_id: endpoint.profile_id(),
            transport_profile_revision: endpoint.profile_revision(),
            transport_profile_digest: endpoint.profile_digest(),
            endpoint_identity: endpoint.endpoint_identity().to_owned(),
            destination_id: destination_id.to_owned(),
            destination_generation,
        };
        TaskEventReceiveRoute::new(Uuid::from_u128(1), pin).expect("valid route fixture")
    }

    fn profile_fixture() -> (ResolvedTransportEndpoint, TransportProfileRevision) {
        let deployment_id = Uuid::from_u128(2);
        let tenant_id = Uuid::from_u128(1);
        let sender = Uuid::from_u128(10);
        let receiver = Uuid::from_u128(11);
        let definition = TransportProfileDefinition::new(
            "star-rust-log".to_owned(),
            "1.0".to_owned(),
            "bi-receiver".to_owned(),
            "run-log-v1".to_owned(),
            vec![sender],
            vec![receiver],
            vec![TransportMessageKind::Event],
            None,
        )
        .expect("valid profile definition");
        let revision = TransportProfileRevision::new(
            deployment_id,
            tenant_id,
            Uuid::from_u128(3),
            4,
            TransportProfileLifecycle::Active,
            definition,
        )
        .expect("valid profile revision");
        let resolver = TrustedEndpointResolver::new(
            deployment_id,
            vec![TrustedEndpointConfig::new(
                "bi-receiver".to_owned(),
                "https://events.example.test".to_owned(),
                "events.example.test".to_owned(),
            )
            .expect("valid endpoint")],
        )
        .expect("valid endpoint registry");
        let endpoint = resolver.resolve(&revision).expect("resolved endpoint");
        (endpoint, revision)
    }

    #[derive(Clone, Default)]
    struct DispositionLog(Arc<Mutex<Vec<&'static str>>>);

    impl DispositionLog {
        fn push(&self, event: &'static str) {
            self.0.lock().expect("disposition log mutex").push(event);
        }

        fn values(&self) -> Vec<&'static str> {
            self.0.lock().expect("disposition log mutex").clone()
        }
    }

    struct LegacyOriginDelivery {
        wire: Vec<u8>,
        origin: TaskEventReceiveOrigin,
        log: DispositionLog,
    }

    impl TaskEventReceiveDelivery for LegacyOriginDelivery {
        fn exact_wire(&self) -> &[u8] {
            &self.wire
        }

        fn origin(&self) -> &TaskEventReceiveOrigin {
            &self.origin
        }

        async fn acknowledge(&self) -> Result<(), TaskEventReceiveError> {
            self.log.push("ack");
            Ok(())
        }

        async fn terminate_permanently(&self) -> Result<(), TaskEventReceiveError> {
            self.log.push("terminate");
            Ok(())
        }
    }

    struct LegacyOriginTransport {
        delivery: Mutex<Option<LegacyOriginDelivery>>,
        log: DispositionLog,
    }

    impl TaskEventReceiveTransport for LegacyOriginTransport {
        type Delivery = LegacyOriginDelivery;

        async fn pull_one(
            &self,
            _route: &TaskEventReceiveRoute,
        ) -> Result<Option<Self::Delivery>, TaskEventReceiveError> {
            self.log.push("pull");
            Ok(self.delivery.lock().expect("delivery mutex").take())
        }
    }

    struct AllowOrigin(DispositionLog);

    impl TaskEventReceiveAuthorization for AllowOrigin {
        async fn verify_origin(
            &self,
            _route: &TaskEventReceiveRoute,
            _origin: &TaskEventReceiveOrigin,
            _wire: &[u8],
        ) -> Result<(), TaskEventReceiveError> {
            self.0.push("verify");
            Ok(())
        }
    }

    struct LegacyOriginInbox(DispositionLog);

    impl TaskEventInboxCommitPort for LegacyOriginInbox {
        async fn commit_exact_wire(
            &self,
            _principal: &AuthenticatedTaskEventPrincipal,
            _route: &bi_owner_runtime::TaskEventRoutePin,
            _source_origin: &TaskEventReceiveOrigin,
            _exact_wire: &[u8],
        ) -> Result<TaskEventInboxReceipt, TaskEventInboxError> {
            self.0.push("commit");
            Err(TaskEventInboxError::SourceOriginUnverifiable)
        }
    }

    struct RejectUnexpectedQuarantine(DispositionLog);

    impl TaskEventQuarantinePort for RejectUnexpectedQuarantine {
        async fn record(
            &self,
            _route: &TaskEventReceiveRoute,
            _origin: &TaskEventReceiveOrigin,
            _wire_sha256: [u8; 32],
            _wire_len: u32,
            _fault: TaskEventQuarantineFault,
        ) -> Result<TaskEventQuarantineReceipt, TaskEventReceiveError> {
            self.0.push("quarantine");
            Err(TaskEventReceiveError::Quarantine)
        }
    }

    #[tokio::test]
    async fn legacy_origin_error_propagates_without_terminal_ack() {
        let log = DispositionLog::default();
        let pin = bi_owner_runtime::TaskEventRoutePin {
            sender_service_id: Uuid::from_u128(8),
            receiver_service_id: Uuid::from_u128(9),
            transport_provider_id: "native-grpc".to_owned(),
            transport_provider_version: "1.0".to_owned(),
            transport_profile_id: Uuid::from_u128(10),
            transport_profile_revision: 1,
            transport_profile_digest: [1; 32],
            endpoint_identity: "bi-receiver".to_owned(),
            destination_id: "task-event-facts".to_owned(),
            destination_generation: Uuid::from_u128(11),
        };
        let route =
            TaskEventReceiveRoute::new(Uuid::from_u128(1), pin).expect("valid legacy replay route");
        let origin = TaskEventReceiveOrigin::new(
            "native-grpc".to_owned(),
            "task-event-facts".to_owned(),
            Uuid::from_u128(11),
            1,
            1_i64.to_be_bytes().to_vec(),
            Uuid::from_u128(5),
        )
        .expect("valid native origin");
        let wire = br#"{"envelope_version":1,"tenant_id":"00000000-0000-0000-0000-000000000001","project_id":"00000000-0000-0000-0000-000000000002","engineering_run_id":"00000000-0000-0000-0000-000000000003","event_id":"00000000-0000-0000-0000-000000000005","work_item_id":"00000000-0000-0000-0000-000000000004","aggregate_type":"lifecycle","aggregate_version":7,"event_type":"task.lifecycle.changed","schema_version":3,"actor_id":"00000000-0000-0000-0000-000000000006","correlation_id":"00000000-0000-0000-0000-000000000007","occurred_at":"2026-10-16T00:00:00Z","payload":{"schema_version":3,"aggregate":{"type":"lifecycle","version":7},"work_item_id":"00000000-0000-0000-0000-000000000004","owner":{"tenant_id":"00000000-0000-0000-0000-000000000001","project_id":"00000000-0000-0000-0000-000000000002","engineering_run_id":"00000000-0000-0000-0000-000000000003"},"lifecycle":{"status":"in_progress","review_state":"none"},"change":{"event_type":"task.lifecycle.changed","from_status":"claimed","to_status":"in_progress"}}}"#.to_vec();
        let transport = LegacyOriginTransport {
            delivery: Mutex::new(Some(LegacyOriginDelivery {
                wire,
                origin,
                log: log.clone(),
            })),
            log: log.clone(),
        };
        let receiver = TaskEventReceiver::new(
            route,
            transport,
            AllowOrigin(log.clone()),
            LegacyOriginInbox(log.clone()),
            RejectUnexpectedQuarantine(log.clone()),
        );

        assert!(matches!(
            forward_owner_receive(
                &receiver,
                &CancellationToken::new(),
                Instant::now() + Duration::from_secs(2),
            )
            .await,
            Err(TaskEventReceiveError::Inbox(
                TaskEventInboxError::SourceOriginUnverifiable
            ))
        ));
        assert_eq!(log.values(), ["pull", "verify", "commit"]);
    }

    #[test]
    fn map_native_error_preserves_auth_and_retryable_transport() {
        // 2026-11-09 (star-api-rest option-A 接线): 该映射随内联副本移入本 crate 后,
        // 语义测试一并迁入 (原 star-api-rest 测试已删除)。
        assert!(matches!(
            map_native_error(NativeGrpcEventReceiveError::Route),
            TaskEventReceiveError::InvalidRoute
        ));
        assert!(matches!(
            map_native_error(NativeGrpcEventReceiveError::Unauthorized),
            TaskEventReceiveError::Unauthorized
        ));
        assert!(matches!(
            map_native_error(NativeGrpcEventReceiveError::Contract),
            TaskEventReceiveError::InvalidOrigin
        ));
        assert!(matches!(
            map_native_error(NativeGrpcEventReceiveError::Transport),
            TaskEventReceiveError::Transport
        ));
        assert!(matches!(
            map_native_error(NativeGrpcEventReceiveError::Busy),
            TaskEventReceiveError::Busy
        ));
    }

    #[test]
    fn route_binding_requires_profile_and_service_identity_pins() {
        let (endpoint, profile) = profile_fixture();
        let generation = Uuid::from_u128(12);
        let route = route_fixture(&endpoint, &profile, "task-event-facts", generation);
        let scope =
            validate_route_binding(&route, &endpoint, &profile).expect("all route pins agree");
        assert_eq!(scope.tenant_id(), Uuid::from_u128(1));
        assert_eq!(scope.sender_service_id(), Uuid::from_u128(10));
        assert_eq!(scope.receiver_service_id(), Uuid::from_u128(11));

        let wrong_revision_pin = bi_owner_runtime::TaskEventRoutePin {
            transport_profile_revision: endpoint.profile_revision() + 1,
            ..route.pin().clone()
        };
        let wrong_revision = TaskEventReceiveRoute::new(route.tenant_id(), wrong_revision_pin)
            .expect("structurally valid but mismatched revision");
        assert!(matches!(
            validate_route_binding(&wrong_revision, &endpoint, &profile),
            Err(NativeTaskEventAdapterError::RouteBinding)
        ));

        let wrong_sender_pin = bi_owner_runtime::TaskEventRoutePin {
            sender_service_id: Uuid::from_u128(99),
            ..route.pin().clone()
        };
        let wrong_sender = TaskEventReceiveRoute::new(route.tenant_id(), wrong_sender_pin)
            .expect("structurally valid but unauthorized sender");
        assert!(matches!(
            validate_route_binding(&wrong_sender, &endpoint, &profile),
            Err(NativeTaskEventAdapterError::RouteBinding)
        ));

        let wrong_tenant = TaskEventReceiveRoute::new(Uuid::from_u128(99), route.pin().clone())
            .expect("structurally valid but mismatched tenant");
        assert!(matches!(
            validate_route_binding(&wrong_tenant, &endpoint, &profile),
            Err(NativeTaskEventAdapterError::RouteBinding)
        ));

        let wrong_digest_pin = bi_owner_runtime::TaskEventRoutePin {
            transport_profile_digest: [0xA5; 32],
            ..route.pin().clone()
        };
        let wrong_digest = TaskEventReceiveRoute::new(route.tenant_id(), wrong_digest_pin)
            .expect("structurally valid but mismatched profile digest");
        assert!(matches!(
            validate_route_binding(&wrong_digest, &endpoint, &profile),
            Err(NativeTaskEventAdapterError::RouteBinding)
        ));

        let revoked = TransportProfileRevision::new(
            Uuid::from_u128(2),
            route.tenant_id(),
            Uuid::from_u128(3),
            4,
            TransportProfileLifecycle::Revoked,
            profile.definition().clone(),
        )
        .expect("valid revoked profile revision");
        assert!(matches!(
            validate_route_binding(&route, &endpoint, &revoked),
            Err(NativeTaskEventAdapterError::RouteBinding)
        ));

        let changed_destination = route_fixture(&endpoint, &profile, "other-log", generation);
        assert_ne!(
            route.pin().destination_id,
            changed_destination.pin().destination_id
        );
    }

    #[test]
    fn route_comparison_includes_tenant_and_every_route_pin() {
        let (endpoint, profile) = profile_fixture();
        let generation = Uuid::from_u128(12);
        let route = route_fixture(&endpoint, &profile, "task-event-facts", generation);
        let same = route_fixture(&endpoint, &profile, "task-event-facts", generation);
        assert!(route_matches(&route, &same));

        let changed_tenant = TaskEventReceiveRoute::new(Uuid::from_u128(99), route.pin().clone())
            .expect("valid alternate tenant");
        let mut changed_digest = route.pin().transport_profile_digest;
        changed_digest[0] ^= 0xff;
        let candidates = [
            changed_tenant,
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    sender_service_id: Uuid::from_u128(99),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate sender"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    receiver_service_id: Uuid::from_u128(99),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate receiver"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    transport_provider_id: "other-provider".to_owned(),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate provider"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    transport_provider_version: "2.0".to_owned(),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate provider version"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    transport_profile_id: Uuid::from_u128(99),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate profile"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    transport_profile_revision: route.pin().transport_profile_revision + 1,
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate revision"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    transport_profile_digest: changed_digest,
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate digest"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    endpoint_identity: "another-endpoint".to_owned(),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate endpoint"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    destination_id: "task-event-facts-v2".to_owned(),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate destination"),
            TaskEventReceiveRoute::new(
                route.tenant_id(),
                bi_owner_runtime::TaskEventRoutePin {
                    destination_generation: Uuid::from_u128(99),
                    ..route.pin().clone()
                },
            )
            .expect("valid alternate destination generation"),
        ];
        for different in candidates {
            assert!(!route_matches(&route, &different));
        }
    }

    #[test]
    fn origin_key_changes_with_each_durable_identity_component() {
        let make_origin = |log_id: &str, generation: Uuid, sequence: i64, message_id: Uuid| {
            TaskEventReceiveOrigin::new(
                "star-rust-log".to_owned(),
                log_id.to_owned(),
                generation,
                sequence,
                sequence.to_be_bytes().to_vec(),
                message_id,
            )
            .expect("valid origin")
        };
        let generation = Uuid::from_u128(12);
        let message_id = Uuid::from_u128(13);
        let base_origin = make_origin("task-event-facts", generation, 7, message_id);
        assert_eq!(base_origin.cursor(), 7_i64.to_be_bytes().as_slice());
        let base = OriginKey::from_origin(&base_origin);
        for changed in [
            make_origin("other-log", generation, 7, message_id),
            make_origin("task-event-facts", Uuid::from_u128(14), 7, message_id),
            make_origin("task-event-facts", generation, 8, message_id),
            make_origin("task-event-facts", generation, 7, Uuid::from_u128(15)),
        ] {
            assert_ne!(base, OriginKey::from_origin(&changed));
        }
    }
}
