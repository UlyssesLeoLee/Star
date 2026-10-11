//! PostgreSQL-backed provider-neutral poison quarantine receipts.
/*
@cypher schema=1 source_sha256=07287853961a76513121152cd77a85be21566bc697c82c4afe51fd0c070ead62
MERGE (self:File {path:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs"})
MERGE (store:Type {id:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs::PgAdmissionReceiveQuarantine",kind:"struct"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs::AdmissionQuarantineStoreError",kind:"enum"})
MERGE (new:Symbol {id:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs::PgAdmissionReceiveQuarantine.new",kind:"method"})
MERGE (persist:Symbol {id:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs::PgAdmissionReceiveQuarantine.persist",kind:"method"})
MERGE (record:Symbol {id:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs::AdmissionReceiveQuarantine::record",kind:"method"})
MERGE (validate:Symbol {id:"crates/platform-messaging-runtime/src/admission_quarantine_pg.rs::validate_origin",kind:"function"})
MERGE (receipt:Table {id:"platform_messaging.admission_receive_quarantine_receipt"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(new)
MERGE (self)-[:DEFINES]->(persist)
MERGE (self)-[:DEFINES]->(record)
MERGE (self)-[:DEFINES]->(validate)
MERGE (new)-[:IMPLEMENTS]->(port:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveQuarantine",kind:"trait"})
MERGE (record)-[:CALLS]->(persist)
MERGE (persist)-[:CALLS]->(validate)
MERGE (persist)-[:WRITES]->(receipt)
MERGE (persist)-[:READS]->(receipt)
@endcypher
*/

use std::{future::Future, pin::Pin};

use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

use crate::admission_receive::{
    AdmissionReceiveError, AdmissionReceiveOrigin, AdmissionReceivePoisonFault,
    AdmissionReceiveQuarantine,
};

/// Maximum wire size accepted by the receiver transport and copied into a quarantine digest.
pub const MAX_ADMISSION_QUARANTINE_WIRE_BYTES: usize = 20 * 1024;
const CLASSIFIER_VERSION: i16 = 1;
const PARSER_VERSION: i16 = 1;

/// Durable metadata write failures always leave the source delivery unconfirmed.
#[derive(Debug, Error)]
pub enum AdmissionQuarantineStoreError {
    /// The platform deployment identity or transport source identity is invalid.
    #[error("invalid admission quarantine identity")]
    InvalidIdentity,
    /// The exact source wire exceeds the quarantine write bound.
    #[error("admission quarantine wire exceeds its size limit")]
    PayloadTooLarge,
    /// A stable source key was replayed with different immutable evidence.
    #[error("admission quarantine source key conflicts with its committed receipt")]
    IdempotencyConflict,
    /// The owner-local database transaction failed or its commit outcome is unknown.
    #[error("admission quarantine database operation failed")]
    Database(#[from] sqlx::Error),
}

/// Append-only PostgreSQL receipt writer, independent of any broker SDK.
pub struct PgAdmissionReceiveQuarantine {
    pool: PgPool,
    deployment_id: Uuid,
}

impl PgAdmissionReceiveQuarantine {
    /// Build the writer with an explicit platform deployment identity and owner-role pool.
    pub fn new(pool: PgPool, deployment_id: Uuid) -> Result<Self, AdmissionQuarantineStoreError> {
        if deployment_id.is_nil() {
            return Err(AdmissionQuarantineStoreError::InvalidIdentity);
        }
        Ok(Self {
            pool,
            deployment_id,
        })
    }

    async fn persist(
        &self,
        origin: &AdmissionReceiveOrigin,
        wire: &[u8],
        fault: AdmissionReceivePoisonFault,
    ) -> Result<(), AdmissionQuarantineStoreError> {
        validate_origin(self.deployment_id, origin, fault)?;
        if wire.len() > MAX_ADMISSION_QUARANTINE_WIRE_BYTES {
            return Err(AdmissionQuarantineStoreError::PayloadTooLarge);
        }
        let payload_bytes = i32::try_from(wire.len())
            .map_err(|_| AdmissionQuarantineStoreError::PayloadTooLarge)?;
        let payload_sha256: [u8; 32] = Sha256::digest(wire).into();
        let message_kind = match origin.scope.kind {
            super::admission_receive::AdmissionReceiveKind::Request => "request",
            super::admission_receive::AdmissionReceiveKind::Result => "result",
            super::admission_receive::AdmissionReceiveKind::Resolution => "resolution",
        };
        let fault_code = match fault {
            AdmissionReceivePoisonFault::MalformedWire => "malformed_admission_wire",
            AdmissionReceivePoisonFault::IdentityMismatch => "admission_identity_mismatch",
        };

        let mut tx = self.pool.begin().await?;
        set_quarantine_scope(&mut tx, self.deployment_id, origin.scope.tenant_id).await?;
        let inserted = sqlx::query(
            r#"INSERT INTO platform_messaging.admission_receive_quarantine_receipt
                  (deployment_id, provider_identity, log_generation, log_id, sequence,
                   tenant_id, sender_service_id, recipient_service_id, message_kind, message_id,
                   fault_code, classifier_version, parser_version, wire_bytes, wire_sha256)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
               ON CONFLICT (deployment_id, provider_identity, log_generation, log_id, sequence)
               DO NOTHING
               RETURNING sequence"#,
        )
        .bind(self.deployment_id)
        .bind(&origin.provider_identity)
        .bind(origin.log_generation)
        .bind(&origin.log_id)
        .bind(origin.sequence)
        .bind(origin.scope.tenant_id)
        .bind(origin.scope.sender_service_id)
        .bind(origin.scope.recipient_service_id)
        .bind(message_kind)
        .bind(origin.message_id)
        .bind(fault_code)
        .bind(CLASSIFIER_VERSION)
        .bind(PARSER_VERSION)
        .bind(payload_bytes)
        .bind(payload_sha256.as_slice())
        .fetch_optional(&mut *tx)
        .await?;

        if inserted.is_none() {
            let existing = sqlx::query(
                r#"SELECT tenant_id, sender_service_id, recipient_service_id, message_kind,
                          message_id, fault_code, classifier_version, parser_version,
                          wire_bytes, wire_sha256
                   FROM platform_messaging.admission_receive_quarantine_receipt
                   WHERE deployment_id = $1 AND provider_identity = $2
                     AND log_generation = $3 AND log_id = $4 AND sequence = $5"#,
            )
            .bind(self.deployment_id)
            .bind(&origin.provider_identity)
            .bind(origin.log_generation)
            .bind(&origin.log_id)
            .bind(origin.sequence)
            .fetch_optional(&mut *tx)
            .await?;
            let Some(existing) = existing else {
                return Err(AdmissionQuarantineStoreError::IdempotencyConflict);
            };
            let exact_replay = existing.try_get::<Uuid, _>("tenant_id")? == origin.scope.tenant_id
                && existing.try_get::<Uuid, _>("sender_service_id")?
                    == origin.scope.sender_service_id
                && existing.try_get::<Uuid, _>("recipient_service_id")?
                    == origin.scope.recipient_service_id
                && existing.try_get::<String, _>("message_kind")? == message_kind
                && existing.try_get::<Uuid, _>("message_id")? == origin.message_id
                && existing.try_get::<String, _>("fault_code")? == fault_code
                && existing.try_get::<i16, _>("classifier_version")? == CLASSIFIER_VERSION
                && existing.try_get::<i16, _>("parser_version")? == PARSER_VERSION
                && existing.try_get::<i32, _>("wire_bytes")? == payload_bytes
                && existing.try_get::<Vec<u8>, _>("wire_sha256")?.as_slice()
                    == payload_sha256.as_slice();
            if !exact_replay {
                return Err(AdmissionQuarantineStoreError::IdempotencyConflict);
            }
        }

        tx.commit().await?;
        Ok(())
    }
}

impl AdmissionReceiveQuarantine for PgAdmissionReceiveQuarantine {
    fn record<'a>(
        &'a self,
        origin: &'a AdmissionReceiveOrigin,
        wire: &'a [u8],
        fault: AdmissionReceivePoisonFault,
    ) -> Pin<Box<dyn Future<Output = Result<(), AdmissionReceiveError>> + Send + 'a>> {
        Box::pin(async move {
            self.persist(origin, wire, fault)
                .await
                .map_err(|_| AdmissionReceiveError::Quarantine)
        })
    }
}

async fn set_quarantine_scope(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL ROLE star_platform_ingress_owner")
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.platform_deployment_id', $1, true)")
        .bind(deployment_id.to_string())
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.platform_quarantine_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn validate_origin(
    deployment_id: Uuid,
    origin: &AdmissionReceiveOrigin,
    fault: AdmissionReceivePoisonFault,
) -> Result<(), AdmissionQuarantineStoreError> {
    let valid_name = |value: &str| {
        !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/' | b':')
            })
    };
    if deployment_id.is_nil()
        || !valid_name(&origin.provider_identity)
        || !valid_name(&origin.log_id)
        || origin.log_generation.is_nil()
        || origin.sequence <= 0
        || origin.scope.tenant_id.is_nil()
        || origin.scope.sender_service_id.is_nil()
        || origin.scope.recipient_service_id.is_nil()
        || origin.scope.sender_service_id == origin.scope.recipient_service_id
        || (origin.message_id.is_nil() && fault != AdmissionReceivePoisonFault::MalformedWire)
    {
        return Err(AdmissionQuarantineStoreError::InvalidIdentity);
    }
    Ok(())
}
