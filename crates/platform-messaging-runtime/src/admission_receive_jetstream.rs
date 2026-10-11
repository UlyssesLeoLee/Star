/*
@cypher schema=1 source_sha256=73b3290dccb6ddc39453157e98e249fa61a1c17f4582e59cf477e0dc889b53d3
MERGE (self:File {path:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs"})
MERGE (transport:Type {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::JetStreamAdmissionReceiveTransport",kind:"struct"})
MERGE (delivery:Type {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::JetStreamAdmissionDelivery",kind:"struct"})
MERGE (pull:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::JetStreamAdmissionReceiveTransport::pull_one",kind:"method"})
MERGE (ack:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::JetStreamAdmissionDelivery::acknowledge",kind:"method"})
MERGE (config:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::validate_consumer",kind:"function"})
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEFINES]->(delivery)
MERGE (self)-[:DEFINES]->(pull)
MERGE (self)-[:DEFINES]->(ack)
MERGE (self)-[:DEFINES]->(config)
MERGE (transport)-[:IMPLEMENTS]->(port:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport",kind:"trait"})
MERGE (delivery)-[:IMPLEMENTS]->(delivery_port:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery",kind:"trait"})
MERGE (pull)-[:CALLS]->(config)
MERGE (ack)-[:CALLS]->(double_ack:ExternalService {id:"async_nats::jetstream::Message::double_ack",kind:"method"})
MERGE (new:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::JetStreamAdmissionReceiveTransport::new",kind:"method"})
MERGE (route:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::route",kind:"function"})
MERGE (self)-[:DEFINES]->(new)
MERGE (self)-[:DEFINES]->(route)
MERGE (new)-[:CALLS]->(config)
MERGE (new)-[:CALLS]->(route)
MERGE (pull)-[:CALLS]->(route)
@endcypher
*/
//! Transitional broker details; injected service UUIDs are not production authentication proof.
use super::admission_receive::{
    AdmissionReceiveError, AdmissionReceiveKind, AdmissionReceiveOrigin, AdmissionReceiveScope,
    AdmissionReceiveTransport, AdmissionReceivedDelivery,
};
use async_nats::jetstream::{
    consumer::{AckPolicy, PullConsumer},
    Message,
};
use domain_automation::schedule::{
    ScheduleRunAdmissionCommandV1, ScheduleRunAdmissionResolutionV1, ScheduleRunAdmissionResultV1,
};
use futures_util::StreamExt;
use std::time::Duration;
use tokio::sync::Mutex;

/// Bounded durable consumer pinned to a single logical owner route.
pub struct JetStreamAdmissionReceiveTransport {
    consumer: Mutex<PullConsumer>,
    scope: AdmissionReceiveScope,
    subject: String,
    stream: String,
    domain: String,
    provider_identity: String,
}

impl JetStreamAdmissionReceiveTransport {
    /// Wrap a provisioned consumer. Authorization/provisioning are independent requirements.
    pub fn new(
        consumer: PullConsumer,
        scope: AdmissionReceiveScope,
        stream: String,
        domain: String,
        provider_identity: String,
    ) -> Result<Self, AdmissionReceiveError> {
        let (schema, kind) = route(scope.kind);
        let subject = format!(
            "star.{schema}.admission.v1.{}.{}.{kind}",
            scope.tenant_id, scope.recipient_service_id
        );
        if stream.is_empty()
            || stream.len() > 128
            || domain.len() > 128
            || provider_identity.is_empty()
            || provider_identity.len() > 128
        {
            return Err(AdmissionReceiveError::Contract);
        }
        validate_consumer(consumer.cached_info(), &subject, &stream)?;
        Ok(Self {
            consumer: Mutex::new(consumer),
            scope,
            subject,
            stream,
            domain,
            provider_identity,
        })
    }
}

fn route(kind: AdmissionReceiveKind) -> (&'static str, &'static str) {
    match kind {
        AdmissionReceiveKind::Request => ("automation", "request"),
        AdmissionReceiveKind::Resolution => ("automation", "resolution"),
        AdmissionReceiveKind::Result => ("multica", "result"),
    }
}

fn validate_consumer(
    info: &async_nats::jetstream::consumer::Info,
    subject: &str,
    stream: &str,
) -> Result<(), AdmissionReceiveError> {
    let c = &info.config;
    if info.stream_name != stream
        || c.durable_name.as_deref().is_none_or(str::is_empty)
        || c.ack_policy != AckPolicy::Explicit
        || c.deliver_subject.is_some()
        || c.filter_subject != subject
        || !c.filter_subjects.is_empty()
        || c.headers_only
        || c.max_ack_pending != 1
        || c.max_waiting != 1
        || c.max_batch != 1
        || c.max_bytes <= 0
        || c.max_bytes > 20480
        || c.max_deliver <= 0
        || c.max_deliver > 12
        || c.ack_wait < Duration::from_secs(20)
        || c.ack_wait > Duration::from_secs(120)
    {
        return Err(AdmissionReceiveError::Contract);
    }
    Ok(())
}

/// Original broker delivery; no payload clone or ACK before owner commit.
pub struct JetStreamAdmissionDelivery {
    message: Message,
    origin: AdmissionReceiveOrigin,
}

impl AdmissionReceivedDelivery for JetStreamAdmissionDelivery {
    fn wire(&self) -> &[u8] {
        &self.message.payload
    }
    fn origin(&self) -> &AdmissionReceiveOrigin {
        &self.origin
    }
    async fn acknowledge(&self) -> Result<(), AdmissionReceiveError> {
        self.message
            .double_ack()
            .await
            .map_err(|_| AdmissionReceiveError::Transport)
    }
}

impl AdmissionReceiveTransport for JetStreamAdmissionReceiveTransport {
    type Delivery = JetStreamAdmissionDelivery;
    async fn pull_one(
        &self,
        scope: AdmissionReceiveScope,
    ) -> Result<Option<Self::Delivery>, AdmissionReceiveError> {
        if scope != self.scope {
            return Err(AdmissionReceiveError::Contract);
        }
        let mut consumer = self.consumer.lock().await;
        let info = consumer
            .info()
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?;
        validate_consumer(info, &self.subject, &self.stream)?;
        let mut messages = consumer
            .fetch()
            .max_messages(1)
            .max_bytes(20480)
            .expires(Duration::from_secs(2))
            .messages()
            .await
            .map_err(|_| AdmissionReceiveError::Transport)?;
        let Some(message) = messages.next().await else {
            return Ok(None);
        };
        let message = message.map_err(|_| AdmissionReceiveError::Transport)?;
        if message.subject.as_str() != self.subject
            || message.payload.is_empty()
            || message.payload.len() > 16384
        {
            return Err(AdmissionReceiveError::Contract);
        }
        let info = message
            .info()
            .map_err(|_| AdmissionReceiveError::Contract)?;
        if info.stream != self.stream
            || info.domain.unwrap_or_default() != self.domain
            || info.consumer != consumer.cached_info().name
        {
            return Err(AdmissionReceiveError::Contract);
        }
        let sequence =
            i64::try_from(info.stream_sequence).map_err(|_| AdmissionReceiveError::Contract)?;
        let (tenant, id) = match scope.kind {
            AdmissionReceiveKind::Request => {
                let m = ScheduleRunAdmissionCommandV1::decode_json(&message.payload)
                    .map_err(|_| AdmissionReceiveError::Contract)?;
                (m.tenant_id, m.request_id)
            }
            AdmissionReceiveKind::Result => {
                let m = ScheduleRunAdmissionResultV1::decode_json(&message.payload)
                    .map_err(|_| AdmissionReceiveError::Contract)?;
                (m.tenant_id, m.result_id)
            }
            AdmissionReceiveKind::Resolution => {
                let m = ScheduleRunAdmissionResolutionV1::decode_json(&message.payload)
                    .map_err(|_| AdmissionReceiveError::Contract)?;
                (m.tenant_id, m.resolution_id)
            }
        };
        let (schema, kind) = route(scope.kind);
        let expected_id = format!("admission.v1.{schema}.{kind}.{tenant}.{id}");
        if tenant != scope.tenant_id
            || sequence <= 0
            || message
                .headers
                .as_ref()
                .and_then(|h| h.get("Nats-Msg-Id"))
                .map(|v| v.as_str())
                != Some(expected_id.as_str())
        {
            return Err(AdmissionReceiveError::Contract);
        }
        Ok(Some(JetStreamAdmissionDelivery {
            message,
            origin: AdmissionReceiveOrigin {
                scope,
                message_id: id,
                provider_identity: self.provider_identity.clone(),
                log_id: self.stream.clone(),
                sequence,
            },
        }))
    }
}
