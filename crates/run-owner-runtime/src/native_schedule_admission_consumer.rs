//! Explicit, bounded native transport composition for Run-owned Schedule admission.
/*
@cypher schema=1 source_sha256=851b5bb88edf1cac823f0c44a6c77342e9cfc001252caa3a6567a9dccd93d0e5
MERGE (self:File {path:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::module",kind:"module"})
MERGE (consumer:Type {id:"run_owner_runtime::NativeGrpcScheduleAdmissionConsumer",kind:"struct"})
MERGE (consumer_error:Type {id:"run_owner_runtime::NativeScheduleAdmissionConsumerError",kind:"enum"})
MERGE (connect:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::NativeGrpcScheduleAdmissionConsumer::connect",kind:"method"})
MERGE (with_quarantine:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::NativeGrpcScheduleAdmissionConsumer::with_quarantine",kind:"method"})
MERGE (consume_one:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::NativeGrpcScheduleAdmissionConsumer::consume_one",kind:"method"})
MERGE (validate_kind:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::supports_schedule_admission_kind",kind:"function"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(consumer)
MERGE (module)-[:DEFINES]->(consumer_error)
MERGE (consumer_error)-[:USES_TYPE]->(native_setup_error:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcSetupError",kind:"enum"})
MERGE (consumer_error)-[:USES_TYPE]->(receive_error:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveError",kind:"enum"})
MERGE (consumer)-[:USES_TYPE]->(receiver:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiver",kind:"struct"})
MERGE (consumer)-[:USES_TYPE]->(store:Type {id:"run_owner_runtime::PostgresScheduleAdmissionStore",kind:"struct"})
MERGE (consumer)-[:USES_TYPE]->(transport:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAdmissionReceiveTransport",kind:"struct"})
MERGE (consumer)-[:USES_TYPE]->(authorization:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcAdmissionReceiveAuthorization",kind:"struct"})
MERGE (connect)-[:USES_TYPE]->(receive_config:Type {id:"platform_messaging_runtime::native_grpc::NativeGrpcReceiveClientConfig",kind:"struct"})
MERGE (connect)-[:USES_TYPE]->(scope:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveScope",kind:"struct"})
MERGE (with_quarantine)-[:USES_TYPE]->(quarantine:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveQuarantine",kind:"trait"})
MERGE (consume_one)-[:USES_TYPE]->(cancellation:Type {id:"tokio::sync::watch::Receiver<bool>",kind:"struct"})
MERGE (consume_one)-[:USES_TYPE]->(uuid:Type {id:"uuid::Uuid",kind:"struct"})
MERGE (module)-[:IMPORTS]->(receive_file:File {path:"crates/platform-messaging-runtime/src/admission_receive.rs"})
MERGE (module)-[:IMPORTS]->(native_file:File {path:"crates/platform-messaging-runtime/src/native_grpc.rs"})
MERGE (module)-[:IMPORTS]->(store_file:File {path:"crates/run-owner-runtime/src/schedule_admission.rs"})
MERGE (connect)-[:CALLS]->(validate_kind)
MERGE (connect)-[:CALLS]->(transport_connect:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport::connect",kind:"method"})
MERGE (connect)-[:CALLS]->(authorization_factory:Symbol {id:"crates/platform-messaging-runtime/src/native_grpc.rs::NativeGrpcAdmissionReceiveTransport::authorization",kind:"method"})
MERGE (connect)-[:CALLS]->(receiver_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::new",kind:"method"})
MERGE (with_quarantine)-[:CALLS]->(receiver_quarantine:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::with_quarantine",kind:"method"})
MERGE (consume_one)-[:CALLS]->(receiver_consume:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::consume_one",kind:"method"})
MERGE (test:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::tests::schedule_route_kind_is_bounded",kind:"test"})
MERGE (test)-[:TESTS]->(validate_kind)
MERGE (preconnect_test:Symbol {id:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs::tests::result_route_is_rejected_before_transport_connect",kind:"test"})
MERGE (preconnect_test)-[:TESTS]->(connect)
@endcypher
*/

use std::sync::Arc;

use platform_messaging_runtime::{
    admission_receive::{
        AdmissionReceiveError, AdmissionReceiveKind, AdmissionReceiveQuarantine, AdmissionReceiver,
    },
    native_grpc::{
        NativeGrpcAdmissionReceiveAuthorization, NativeGrpcAdmissionReceiveTransport,
        NativeGrpcReceiveClientConfig, NativeGrpcSetupError,
    },
};
use thiserror::Error;
use tokio::sync::watch;
use uuid::Uuid;

use crate::PostgresScheduleAdmissionStore;

/// Setup or one-message consumption failures for the opt-in Run-owner receiver.
#[derive(Debug, Error)]
pub enum NativeScheduleAdmissionConsumerError {
    /// Native gRPC setup failed without exposing TLS material.
    #[error(transparent)]
    NativeSetup(#[from] NativeGrpcSetupError),
    /// The shared receiver rejected its pinned scope or one-message operation.
    #[error(transparent)]
    Receive(#[from] AdmissionReceiveError),
    /// Run-owner admission does not own Multica result messages.
    #[error("Run-owner Schedule admission cannot consume Result routes")]
    UnsupportedKind,
}

/// One explicitly driven, bounded native receiver for Run-owned Schedule admission.
///
/// Constructing this value does not start a background task or enable Run execution. Callers
/// decide when to invoke `consume_one`; each call processes at most one delivery.
pub struct NativeGrpcScheduleAdmissionConsumer {
    receiver: AdmissionReceiver<
        NativeGrpcAdmissionReceiveTransport,
        PostgresScheduleAdmissionStore,
        NativeGrpcAdmissionReceiveAuthorization,
    >,
}

impl NativeGrpcScheduleAdmissionConsumer {
    /// Connect one mTLS-pinned native route and bind it to the Run-owner admission store.
    pub async fn connect(
        config: NativeGrpcReceiveClientConfig<'_>,
        store: PostgresScheduleAdmissionStore,
    ) -> Result<Self, NativeScheduleAdmissionConsumerError> {
        let scope = config.scope;
        if !supports_schedule_admission_kind(scope.kind) {
            return Err(NativeScheduleAdmissionConsumerError::UnsupportedKind);
        }

        let transport = NativeGrpcAdmissionReceiveTransport::connect(config).await?;
        let authorization = transport.authorization();
        let receiver = AdmissionReceiver::new(transport, store, authorization, scope)?;
        Ok(Self { receiver })
    }

    /// Attach a durable quarantine implementation; poison deliveries otherwise remain unacked.
    pub fn with_quarantine(mut self, quarantine: Arc<dyn AdmissionReceiveQuarantine>) -> Self {
        self.receiver = self.receiver.with_quarantine(quarantine);
        self
    }

    /// Consume no more than one delivery, subject to cancellation and the shared 10-second limit.
    pub async fn consume_one(
        &mut self,
        cancellation: &mut watch::Receiver<bool>,
    ) -> Result<Option<Uuid>, NativeScheduleAdmissionConsumerError> {
        Ok(self.receiver.consume_one(cancellation).await?)
    }
}

fn supports_schedule_admission_kind(kind: AdmissionReceiveKind) -> bool {
    matches!(
        kind,
        AdmissionReceiveKind::Request | AdmissionReceiveKind::Resolution
    )
}

#[cfg(test)]
mod tests {
    use std::{path::Path, time::Duration};

    use platform_messaging_runtime::{
        admission_delivery::AdmissionTransportProfile,
        admission_receive::{AdmissionReceiveKind, AdmissionReceiveScope},
        native_grpc::NativeGrpcReceiveClientConfig,
    };
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use super::{
        NativeGrpcScheduleAdmissionConsumer, NativeScheduleAdmissionConsumerError,
        supports_schedule_admission_kind,
    };

    #[test]
    fn schedule_route_kind_is_bounded() {
        assert!(supports_schedule_admission_kind(
            AdmissionReceiveKind::Request
        ));
        assert!(supports_schedule_admission_kind(
            AdmissionReceiveKind::Resolution
        ));
        assert!(!supports_schedule_admission_kind(
            AdmissionReceiveKind::Result
        ));
    }

    #[tokio::test]
    async fn result_route_is_rejected_before_transport_connect() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://localhost/test")
            .expect("lazy test pool config is valid");
        let store = crate::PostgresScheduleAdmissionStore::new(pool);
        let scope = AdmissionReceiveScope {
            tenant_id: Uuid::from_u128(1),
            sender_service_id: Uuid::from_u128(2),
            recipient_service_id: Uuid::from_u128(3),
            kind: AdmissionReceiveKind::Result,
        };
        let profile = AdmissionTransportProfile::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            [7; 32],
            "unused-endpoint".to_owned(),
        )
        .expect("test profile is valid");
        let config = NativeGrpcReceiveClientConfig {
            uri: "https://example.invalid:7443",
            server_domain: "example.invalid",
            endpoint_identity: "unused-endpoint",
            server_ca_path: Path::new("unused-ca.pem"),
            client_cert_path: Path::new("unused-cert.pem"),
            client_key_path: Path::new("unused-key.pem"),
            connect_timeout: Duration::from_secs(1),
            request_timeout: Duration::from_secs(1),
            scope,
            lease_seconds: 30,
            profile,
        };

        let result = NativeGrpcScheduleAdmissionConsumer::connect(config, store).await;

        assert!(matches!(
            result,
            Err(NativeScheduleAdmissionConsumerError::UnsupportedKind)
        ));
    }
}
