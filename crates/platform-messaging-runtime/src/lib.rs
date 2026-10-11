//! Platform-owned bounded poison persistence; the JetStream integration is an optional adapter.
/*
@cypher schema=1 source_sha256=f6905373d6d7d6c645eeb4a13a41ddcf98fcb622f2e0a8f8979b96a05e0643cf
MERGE (self:File {path:"crates/platform-messaging-runtime/src/lib.rs"})
MERGE (jetstream_feature:Config {id:"cargo.feature.platform-messaging-runtime.jetstream-adapter"})
MERGE (module:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::platform_messaging_runtime",kind:"module"})
MERGE (source:Type {id:"crates/platform-messaging-runtime/src/lib.rs::TrustedJetStreamSource",kind:"struct"})
MERGE (identity:Type {id:"crates/platform-messaging-runtime/src/lib.rs::JetStreamSourceIdentity",kind:"struct"})
MERGE (fault:Type {id:"crates/platform-messaging-runtime/src/lib.rs::PermanentPoisonFault",kind:"enum"})
MERGE (policy:Type {id:"crates/platform-messaging-runtime/src/lib.rs::PayloadRetentionPolicy",kind:"enum"})
MERGE (outcome:Type {id:"crates/platform-messaging-runtime/src/lib.rs::QuarantineRecordOutcome",kind:"enum"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/lib.rs::QuarantineError",kind:"enum"})
MERGE (service:Type {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine",kind:"struct"})
MERGE (purger:Type {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformQuarantinePurger",kind:"struct"})
MERGE (new_source:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::TrustedJetStreamSource.new",kind:"method"})
MERGE (new_service:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.connect",kind:"method"})
MERGE (new_purger:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformQuarantinePurger.new",kind:"method"})
MERGE (record:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.record_message",kind:"method"})
MERGE (purge:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformQuarantinePurger.purge_expired",kind:"method"})
MERGE (identity_fn:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::source_identity",kind:"function"})
MERGE (digest_fn:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::payload_digest",kind:"function"})
MERGE (replay_fn:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::encode_replay_envelope",kind:"function"})
MERGE (ingress_scope:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::set_platform_ingress_scope",kind:"function"})
MERGE (purger_scope:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::set_platform_purger_scope",kind:"function"})
MERGE (deployment_scope:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::set_platform_deployment_id",kind:"function"})
MERGE (verify_fn:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::verify_existing_receipt",kind:"function"})
MERGE (verify_row:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::verify_existing_row",kind:"function"})
MERGE (source_validate:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::validate_source",kind:"function"})
MERGE (purge_batch:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::validate_purge_batch",kind:"function"})
MERGE (replay_header_limits:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::validate_replay_header_limits",kind:"function"})
MERGE (encrypt:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.encrypt_if_configured",kind:"method"})
MERGE (existing:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.existing_receipt",kind:"method"})
MERGE (receipt:Table {id:"platform_messaging.poison_quarantine_receipt"})
MERGE (payload:Table {id:"platform_messaging.poison_quarantine_payload_work"})
MERGE (audit:Table {id:"platform_messaging.poison_quarantine_payload_purge_audit"})
MERGE (jetstream:Type {id:"external::async_nats::jetstream::Message",kind:"external_type"})
MERGE (kms:Type {id:"crates/domain-kms/src/lib.rs::KmsClient",kind:"trait"})
MERGE (kms_health:Symbol {id:"crates/domain-kms/src/lib.rs::KmsClient.health",kind:"method"})
MERGE (kms_encrypt:Symbol {id:"crates/domain-kms/src/lib.rs::KmsClient.encrypt",kind:"method"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEPENDS_ON]->(jetstream_feature)
MERGE (module)-[:DEFINES]->(source)
MERGE (module)-[:DEFINES]->(identity)
MERGE (module)-[:DEFINES]->(fault)
MERGE (module)-[:DEFINES]->(policy)
MERGE (module)-[:DEFINES]->(outcome)
MERGE (module)-[:DEFINES]->(error)
MERGE (module)-[:DEFINES]->(service)
MERGE (module)-[:DEFINES]->(purger)
MERGE (module)-[:DEFINES]->(new_source)
MERGE (module)-[:DEFINES]->(new_service)
MERGE (module)-[:DEFINES]->(record)
MERGE (module)-[:DEFINES]->(purge)
MERGE (module)-[:DEFINES]->(identity_fn)
MERGE (module)-[:DEFINES]->(digest_fn)
MERGE (module)-[:DEFINES]->(replay_fn)
MERGE (module)-[:DEFINES]->(ingress_scope)
MERGE (module)-[:DEFINES]->(purger_scope)
MERGE (module)-[:DEFINES]->(deployment_scope)
MERGE (module)-[:DEFINES]->(verify_fn)
MERGE (module)-[:DEFINES]->(verify_row)
MERGE (module)-[:DEFINES]->(source_validate)
MERGE (module)-[:DEFINES]->(purge_batch)
MERGE (module)-[:DEFINES]->(encrypt)
MERGE (module)-[:DEFINES]->(existing)
MERGE (module)-[:DEFINES]->(new_purger)
MERGE (self)-[:IMPORTS]->(jetstream)
MERGE (self)-[:IMPORTS]->(kms)
MERGE (new_source)-[:CALLS]->(source_validate)
MERGE (new_service)-[:CALLS]->(source_validate)
MERGE (new_service)-[:USES_TYPE]->(kms)
MERGE (new_service)-[:CALLS]->(kms_health)
MERGE (new_purger)-[:CALLS]->(source_validate)
MERGE (record)-[:USES_TYPE]->(jetstream)
MERGE (record)-[:USES_TYPE]->(kms)
MERGE (record)-[:CALLS]->(identity_fn)
MERGE (record)-[:CALLS]->(digest_fn)
MERGE (record)-[:CALLS]->(replay_fn)
MERGE (record)-[:CALLS]->(ingress_scope)
MERGE (record)-[:CALLS]->(verify_fn)
MERGE (record)-[:CALLS]->(existing)
MERGE (record)-[:CALLS]->(encrypt)
MERGE (record)-[:READS]->(receipt)
MERGE (record)-[:WRITES]->(receipt)
MERGE (record)-[:WRITES]->(payload)
MERGE (identity_fn)-[:USES_TYPE]->(jetstream)
MERGE (purge)-[:CALLS]->(purge_batch)
MERGE (replay_fn)-[:CALLS]->(replay_header_limits)
MERGE (purge)-[:WRITES]->(payload)
MERGE (purge)-[:WRITES]->(audit)
MERGE (ingress_scope)-[:CALLS]->(deployment_scope)
MERGE (purger_scope)-[:CALLS]->(deployment_scope)
MERGE (ingress_scope)-[:CONFIGURES]->(ingress_role:DatabaseRole {id:"star_platform_ingress_owner"})
MERGE (purger_scope)-[:CONFIGURES]->(purger_role:DatabaseRole {id:"star_platform_quarantine_purger"})
MERGE (deployment_scope)-[:CONFIGURES]->(deployment_setting:Config {id:"app.platform_deployment_id"})
MERGE (verify_fn)-[:CALLS]->(verify_row)
MERGE (purge)-[:CALLS]->(purger_scope)
MERGE (existing)-[:CALLS]->(ingress_scope)
MERGE (existing)-[:READS]->(receipt)
MERGE (encrypt)-[:USES_TYPE]->(kms)
MERGE (encrypt)-[:CALLS]->(kms_encrypt)
MERGE (payload)-[:REFERENCES]->(receipt)
MERGE (audit)-[:REFERENCES]->(receipt)
MERGE (test_source:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::tests.trusted_source_rejects_missing_or_invalid_identity",kind:"test"})
MERGE (test_faults:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::tests.fault_codes_are_stable_and_do_not_include_run_owner_data_corruption",kind:"test"})
MERGE (test_digest:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::tests.source_payload_digest_is_sha256_over_exact_bytes",kind:"test"})
MERGE (test_policy:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::tests.policies_are_explicit_and_have_distinct_database_codes",kind:"test"})
MERGE (test_purge:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::tests.ttl_purge_batches_are_bounded",kind:"test"})
MERGE (test_headers:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::tests.replay_headers_are_bounded_by_count_and_bytes",kind:"test"})
MERGE (module)-[:DEFINES]->(test_source)
MERGE (module)-[:DEFINES]->(test_faults)
MERGE (module)-[:DEFINES]->(test_digest)
MERGE (module)-[:DEFINES]->(test_policy)
MERGE (module)-[:DEFINES]->(test_purge)
MERGE (module)-[:DEFINES]->(test_headers)
MERGE (test_source)-[:TESTS]->(source_validate)
MERGE (test_faults)-[:TESTS]->(fault)
MERGE (test_digest)-[:TESTS]->(digest_fn)
MERGE (test_policy)-[:TESTS]->(policy)
MERGE (test_purge)-[:TESTS]->(purge_batch)
MERGE (test_headers)-[:TESTS]->(replay_header_limits)
MERGE (module)-[:IMPORTS]->(admission_delivery_file:File {path:"crates/platform-messaging-runtime/src/admission_delivery.rs"})
MERGE (receive_module:File {path:"crates/platform-messaging-runtime/src/admission_receive.rs"})
MERGE (self)-[:IMPORTS]->(receive_module)
MERGE (receive_jetstream_module:File {path:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs"})
MERGE (self)-[:IMPORTS]->(receive_jetstream_module)
@endcypher
*/

pub mod admission_delivery;
#[cfg(feature = "durable-quarantine")]
pub mod admission_quarantine_pg;
pub mod admission_receive;
#[cfg(feature = "jetstream-adapter")]
pub mod admission_receive_jetstream;

#[cfg(feature = "jetstream-adapter")]
use std::{sync::Arc, time::Duration};

#[cfg(feature = "jetstream-adapter")]
use async_nats::jetstream::Message;
#[cfg(feature = "jetstream-adapter")]
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
#[cfg(feature = "jetstream-adapter")]
use domain_kms::{KeyId, KmsClient, KmsMode, TenantId};
#[cfg(feature = "jetstream-adapter")]
use serde::Serialize;
#[cfg(feature = "jetstream-adapter")]
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "jetstream-adapter")]
/// Hard upper bound for an original JetStream body accepted by quarantine.
pub const MAX_SOURCE_PAYLOAD_BYTES: usize = 132 * 1024;
#[cfg(feature = "jetstream-adapter")]
/// Maximum `Nats-Msg-Id` bytes copied into an encrypted replay envelope.
pub const MAX_REPLAY_HEADER_BYTES: usize = 8 * 1024;
#[cfg(feature = "jetstream-adapter")]
/// Maximum number of `Nats-Msg-Id` values copied from an untrusted message.
pub const MAX_REPLAY_HEADER_COUNT: usize = 16;
#[cfg(feature = "jetstream-adapter")]
/// Maximum serialized KMS envelope stored for one quarantined message.
pub const MAX_ENCRYPTED_BLOB_BYTES: usize = 200 * 1024;
#[cfg(feature = "jetstream-adapter")]
/// Hard total encrypted-payload quota per deployment; expired rows count until audited purge.
pub const MAX_ENCRYPTED_QUARANTINE_BYTES: i64 = 4 * 1024 * 1024;
#[cfg(feature = "jetstream-adapter")]
const CLASSIFIER_VERSION: i16 = 1;
#[cfg(feature = "jetstream-adapter")]
const REPLAY_ENVELOPE_VERSION: u16 = 1;
#[cfg(feature = "jetstream-adapter")]
const KMS_ENCRYPT_TIMEOUT: Duration = Duration::from_secs(2);

#[cfg(feature = "jetstream-adapter")]
/// Trusted source identity pinned by deployment configuration, not message headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedJetStreamSource {
    deployment_id: Uuid,
    stream_generation: Uuid,
    stream_name: String,
}

#[cfg(feature = "jetstream-adapter")]
impl TrustedJetStreamSource {
    /// Validate a configured deployment and the expected server-side stream identity.
    pub fn new(
        deployment_id: Uuid,
        stream_generation: Uuid,
        stream_name: impl Into<String>,
    ) -> Result<Self, QuarantineError> {
        let source = Self {
            deployment_id,
            stream_generation,
            stream_name: stream_name.into(),
        };
        validate_source(&source)?;
        Ok(source)
    }
}

#[cfg(feature = "jetstream-adapter")]
/// Server-derived message identity combined with the trusted configured generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JetStreamSourceIdentity {
    /// Stable deployment identity from trusted process configuration.
    pub deployment_id: Uuid,
    /// Configured stream generation; stream recreation must use a new generation UUID.
    pub stream_generation: Uuid,
    /// Server stream name parsed from the JetStream ACK reply subject.
    pub stream_name: String,
    /// Server stream sequence parsed from the JetStream ACK reply subject.
    pub stream_sequence: i64,
}

#[cfg(feature = "jetstream-adapter")]
/// Versioned permanent-fault reasons that may be quarantined and then acknowledged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermanentPoisonFault {
    /// Envelope, subject, message ID, or typed event contract is invalid.
    InvalidCapacityResultEvent,
    /// The byte payload cannot be decoded as the versioned JSON event contract.
    MalformedCapacityResultJson,
    /// A source event identity was reused with different immutable event content.
    CapacityResultEventIdConflict,
}

#[cfg(feature = "jetstream-adapter")]
impl PermanentPoisonFault {
    /// Stable database code for this classifier version.
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidCapacityResultEvent => "invalid_capacity_result_event",
            Self::MalformedCapacityResultJson => "malformed_capacity_result_json",
            Self::CapacityResultEventIdConflict => "capacity_result_event_id_conflict",
        }
    }
}

#[cfg(feature = "jetstream-adapter")]
/// Explicit poison-body policy. There is intentionally no implicit `Default` policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadRetentionPolicy {
    /// Store only source metadata, exact body length, and SHA-256 digest.
    MetadataOnly,
    /// Store an encrypted replay envelope for 24 hours under a trusted platform KMS scope.
    Encrypted24Hours {
        /// Platform-controlled KMS tenant/scope; never derived from untrusted event content.
        kms_scope: TenantId,
        /// Pre-provisioned KMS data-encryption key identity.
        dek_id: KeyId,
    },
}

#[cfg(feature = "jetstream-adapter")]
impl PayloadRetentionPolicy {
    const fn database_code(&self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::Encrypted24Hours { .. } => "encrypted_24h",
        }
    }
}

#[cfg(feature = "jetstream-adapter")]
/// Result of an append-only quarantine receipt write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineRecordOutcome {
    /// New receipt committed; encrypted body was stored only when the explicit policy requires it.
    Stored {
        /// Server-assigned source stream sequence.
        stream_sequence: i64,
        /// Whether an encrypted payload Work row was committed.
        body_retained: bool,
    },
    /// Exact source-key replay matched an already committed receipt.
    Duplicate {
        /// Server-assigned source stream sequence.
        stream_sequence: i64,
    },
}

/// Errors that must leave the source JetStream delivery unacknowledged.
#[derive(Debug, Error)]
pub enum QuarantineError {
    /// A general platform quarantine or purge configuration is invalid.
    #[error("invalid quarantine configuration")]
    InvalidConfiguration,
    #[cfg(feature = "jetstream-adapter")]
    /// Trusted deployment, stream generation, or stream name is invalid.
    #[error("invalid trusted JetStream source configuration")]
    InvalidSource,
    #[cfg(feature = "jetstream-adapter")]
    /// Server metadata could not be extracted from the JetStream ACK reply subject.
    #[error("JetStream source identity metadata is unavailable")]
    SourceMetadata(#[source] async_nats::Error),
    #[cfg(feature = "jetstream-adapter")]
    /// The message came from a different stream than the configured trusted stream.
    #[error("JetStream message stream does not match trusted stream configuration")]
    UnexpectedStream,
    #[cfg(feature = "jetstream-adapter")]
    /// The message body exceeds the bounded quarantine source payload size.
    #[error("JetStream payload exceeds quarantine size limit")]
    PayloadTooLarge,
    #[cfg(feature = "jetstream-adapter")]
    /// The parser version is not a positive supported version.
    #[error("quarantine parser version is invalid")]
    InvalidParserVersion,
    #[cfg(feature = "jetstream-adapter")]
    /// NATS headers exceed the bounded replay-envelope header limit.
    #[error("JetStream headers exceed quarantine replay envelope limit")]
    HeadersTooLarge,
    #[cfg(feature = "jetstream-adapter")]
    /// Encrypted retention requires a live configured KMS client and key.
    #[error("encrypted quarantine retention is unavailable")]
    EncryptionUnavailable,
    #[cfg(feature = "jetstream-adapter")]
    /// The KMS request exceeded its fixed deadline.
    #[error("quarantine encryption exceeded its deadline")]
    EncryptionTimeout,
    #[cfg(feature = "jetstream-adapter")]
    /// The KMS operation failed; the delivery remains unacknowledged.
    #[error("quarantine KMS operation failed")]
    Kms(#[source] domain_kms::KmsError),
    #[cfg(feature = "jetstream-adapter")]
    /// Serialization of the versioned encrypted replay envelope failed.
    #[error("quarantine replay envelope serialization failed")]
    Json(#[from] serde_json::Error),
    #[cfg(feature = "jetstream-adapter")]
    /// The configured per-deployment encrypted retention quota is exhausted.
    #[error("quarantine encrypted payload quota is exhausted")]
    QuotaExceeded,
    /// The TTL purger batch must be between 1 and 100 rows.
    #[error("quarantine purge batch limit is invalid")]
    InvalidPurgeBatch,
    /// A locked expired Work row could not be deleted after its audit insert.
    #[error("quarantine TTL purge could not atomically delete its audited Work row")]
    PurgeInvariantViolation,
    #[cfg(feature = "jetstream-adapter")]
    /// The source key already exists with different immutable receipt evidence.
    #[error("quarantine source identity conflicts with an existing receipt")]
    IdempotencyConflict,
    #[cfg(feature = "jetstream-adapter")]
    /// The serialized encrypted payload exceeds its strict database storage bound.
    #[error("encrypted quarantine payload exceeds storage limit")]
    EncryptedBlobTooLarge,
    /// PostgreSQL failed; no JetStream acknowledgement is sent.
    #[error("quarantine database operation failed")]
    Database(#[from] sqlx::Error),
}

#[cfg(feature = "jetstream-adapter")]
/// Platform Messaging owner service; uses a dedicated owner-role pool and trusted source config.
pub struct PlatformPoisonQuarantine {
    pool: PgPool,
    source: TrustedJetStreamSource,
    retention: PayloadRetentionPolicy,
    kms: Option<Arc<dyn KmsClient>>,
}

/// Separate bounded deletion authority for expired encrypted quarantine Work rows.
pub struct PlatformQuarantinePurger {
    pool: PgPool,
    deployment_id: Uuid,
}

impl PlatformQuarantinePurger {
    /// Construct a purger using the dedicated `star_platform_quarantine_purger` pool.
    pub fn new(pool: PgPool, deployment_id: Uuid) -> Result<Self, QuarantineError> {
        if deployment_id.is_nil() {
            return Err(QuarantineError::InvalidConfiguration);
        }
        Ok(Self {
            pool,
            deployment_id,
        })
    }

    /// Delete at most 100 expired rows, appending purge evidence in the same transaction.
    ///
    /// This method is explicit and bounded. `record_message` never triggers a purge implicitly.
    pub async fn purge_expired(&self, batch_limit: i64) -> Result<u64, QuarantineError> {
        validate_purge_batch(batch_limit)?;
        let mut tx = self.pool.begin().await?;
        set_platform_purger_scope(&mut tx, self.deployment_id).await?;
        let rows = sqlx::query(
            r#"SELECT stream_generation, stream_name, stream_sequence,
                      encrypted_blob_sha256, encoded_bytes
               FROM platform_messaging.poison_quarantine_payload_work
               WHERE deployment_id = $1 AND expires_at <= clock_timestamp()
               ORDER BY expires_at, stream_generation, stream_sequence
               LIMIT $2 FOR UPDATE SKIP LOCKED"#,
        )
        .bind(self.deployment_id)
        .bind(batch_limit)
        .fetch_all(&mut *tx)
        .await?;

        for row in &rows {
            let generation: Uuid = row.try_get("stream_generation")?;
            let stream: String = row.try_get("stream_name")?;
            let sequence: i64 = row.try_get("stream_sequence")?;
            let digest: Vec<u8> = row.try_get("encrypted_blob_sha256")?;
            let bytes: i32 = row.try_get("encoded_bytes")?;
            sqlx::query(
                r#"INSERT INTO platform_messaging.poison_quarantine_payload_purge_audit
                      (deployment_id, stream_generation, stream_name, stream_sequence,
                       encrypted_blob_sha256, encoded_bytes, purge_reason)
                   VALUES ($1,$2,$3,$4,$5,$6,'ttl_expired')"#,
            )
            .bind(self.deployment_id)
            .bind(generation)
            .bind(&stream)
            .bind(sequence)
            .bind(&digest)
            .bind(bytes)
            .execute(&mut *tx)
            .await?;
            let deleted = sqlx::query(
                r#"DELETE FROM platform_messaging.poison_quarantine_payload_work
                   WHERE deployment_id = $1 AND stream_generation = $2
                     AND stream_name = $3 AND stream_sequence = $4
                   RETURNING stream_sequence"#,
            )
            .bind(self.deployment_id)
            .bind(generation)
            .bind(&stream)
            .bind(sequence)
            .fetch_optional(&mut *tx)
            .await?;
            if deleted.is_none() {
                return Err(QuarantineError::PurgeInvariantViolation);
            }
        }
        tx.commit().await?;
        Ok(rows.len() as u64)
    }
}

#[cfg(feature = "jetstream-adapter")]
impl PlatformPoisonQuarantine {
    /// Connect the quarantine writer with an explicit policy; encrypted mode rejects LocalMock KMS.
    pub async fn connect(
        pool: PgPool,
        source: TrustedJetStreamSource,
        retention: PayloadRetentionPolicy,
        kms: Option<Arc<dyn KmsClient>>,
    ) -> Result<Self, QuarantineError> {
        validate_source(&source)?;
        match &retention {
            PayloadRetentionPolicy::MetadataOnly => {}
            PayloadRetentionPolicy::Encrypted24Hours { kms_scope, dek_id } => {
                if kms_scope.trim().is_empty() || dek_id.as_str().trim().is_empty() {
                    return Err(QuarantineError::EncryptionUnavailable);
                }
                let kms = kms.as_ref().ok_or(QuarantineError::EncryptionUnavailable)?;
                if kms.health().await.map_err(QuarantineError::Kms)?.mode == KmsMode::LocalMock {
                    return Err(QuarantineError::EncryptionUnavailable);
                }
            }
        }
        Ok(Self {
            pool,
            source,
            retention,
            kms,
        })
    }

    /// Persist a permanent-fault receipt before the caller acknowledges the source delivery.
    pub async fn record_message(
        &self,
        message: &Message,
        fault: PermanentPoisonFault,
        parser_version: i16,
    ) -> Result<QuarantineRecordOutcome, QuarantineError> {
        let identity = source_identity(&self.source, message)?;
        let body = message.payload.as_ref();
        if body.len() > MAX_SOURCE_PAYLOAD_BYTES {
            return Err(QuarantineError::PayloadTooLarge);
        }
        if parser_version <= 0 {
            return Err(QuarantineError::InvalidParserVersion);
        }
        let digest = payload_digest(body);
        let body_len = i32::try_from(body.len()).map_err(|_| QuarantineError::PayloadTooLarge)?;

        if let Some(outcome) = self
            .existing_receipt(&identity, fault, parser_version, body_len, &digest)
            .await?
        {
            return Ok(outcome);
        }

        let encrypted_blob = self
            .encrypt_if_configured(message, &identity, body, &digest)
            .await?;
        let encrypted_bytes = encrypted_blob.as_ref().map(Vec::len).unwrap_or(0);
        if encrypted_bytes > MAX_ENCRYPTED_BLOB_BYTES {
            return Err(QuarantineError::EncryptedBlobTooLarge);
        }

        let mut tx = self.pool.begin().await?;
        set_platform_ingress_scope(&mut tx, identity.deployment_id).await?;
        if encrypted_blob.is_some() {
            // Serialize encrypted writes so concurrent consumers cannot exceed the per-deployment
            // disk quota. Expired-but-not-purged rows still count until a separate audited purge.
            sqlx::query(
                "SELECT pg_advisory_xact_lock(hashtext('star.platform_messaging.quarantine'), hashtext($1::text))",
            )
                .bind(identity.deployment_id)
                .execute(&mut *tx)
                .await?;
        }
        let inserted = sqlx::query(
            r#"INSERT INTO platform_messaging.poison_quarantine_receipt
                  (deployment_id, stream_generation, stream_name, stream_sequence,
                   fault_code, classifier_version, parser_version, payload_bytes,
                   payload_sha256, retention_policy)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               ON CONFLICT (deployment_id, stream_generation, stream_name, stream_sequence)
               DO NOTHING
               RETURNING stream_sequence"#,
        )
        .bind(identity.deployment_id)
        .bind(identity.stream_generation)
        .bind(&identity.stream_name)
        .bind(identity.stream_sequence)
        .bind(fault.code())
        .bind(CLASSIFIER_VERSION)
        .bind(parser_version)
        .bind(body_len)
        .bind(digest.as_slice())
        .bind(self.retention.database_code())
        .fetch_optional(&mut *tx)
        .await?;

        if inserted.is_none() {
            verify_existing_receipt(&mut tx, &identity, fault, parser_version, body_len, &digest)
                .await?;
            tx.commit().await?;
            return Ok(QuarantineRecordOutcome::Duplicate {
                stream_sequence: identity.stream_sequence,
            });
        }

        if let Some(blob) = encrypted_blob {
            let current_bytes: i64 = sqlx::query_scalar(
                r#"SELECT COALESCE(SUM(encoded_bytes), 0)::BIGINT
                   FROM platform_messaging.poison_quarantine_payload_work
                   WHERE deployment_id = $1"#,
            )
            .bind(identity.deployment_id)
            .fetch_one(&mut *tx)
            .await?;
            let blob_len =
                i64::try_from(blob.len()).map_err(|_| QuarantineError::EncryptedBlobTooLarge)?;
            if current_bytes.saturating_add(blob_len) > MAX_ENCRYPTED_QUARANTINE_BYTES {
                return Err(QuarantineError::QuotaExceeded);
            }
            let encrypted_digest = Sha256::digest(&blob);
            sqlx::query(
                r#"INSERT INTO platform_messaging.poison_quarantine_payload_work
                      (deployment_id, stream_generation, stream_name, stream_sequence,
                       encrypted_blob, encrypted_blob_sha256, encoded_bytes, expires_at)
                   VALUES ($1,$2,$3,$4,$5,$6,$7,transaction_timestamp() + interval '24 hours')"#,
            )
            .bind(identity.deployment_id)
            .bind(identity.stream_generation)
            .bind(&identity.stream_name)
            .bind(identity.stream_sequence)
            .bind(&blob)
            .bind(encrypted_digest.as_slice())
            .bind(i32::try_from(blob.len()).map_err(|_| QuarantineError::EncryptedBlobTooLarge)?)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(QuarantineRecordOutcome::Stored {
            stream_sequence: identity.stream_sequence,
            body_retained: self.retention.database_code() == "encrypted_24h",
        })
    }

    async fn existing_receipt(
        &self,
        identity: &JetStreamSourceIdentity,
        fault: PermanentPoisonFault,
        parser_version: i16,
        body_len: i32,
        digest: &[u8; 32],
    ) -> Result<Option<QuarantineRecordOutcome>, QuarantineError> {
        let mut tx = self.pool.begin().await?;
        set_platform_ingress_scope(&mut tx, identity.deployment_id).await?;
        let row = sqlx::query(
            r#"SELECT fault_code, classifier_version, parser_version, payload_bytes, payload_sha256
               FROM platform_messaging.poison_quarantine_receipt
               WHERE deployment_id = $1 AND stream_generation = $2
                 AND stream_name = $3 AND stream_sequence = $4"#,
        )
        .bind(identity.deployment_id)
        .bind(identity.stream_generation)
        .bind(&identity.stream_name)
        .bind(identity.stream_sequence)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(None);
        };
        verify_existing_row(&row, fault, parser_version, body_len, digest)?;
        tx.commit().await?;
        Ok(Some(QuarantineRecordOutcome::Duplicate {
            stream_sequence: identity.stream_sequence,
        }))
    }

    async fn encrypt_if_configured(
        &self,
        message: &Message,
        identity: &JetStreamSourceIdentity,
        body: &[u8],
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, QuarantineError> {
        let PayloadRetentionPolicy::Encrypted24Hours { kms_scope, dek_id } = &self.retention else {
            return Ok(None);
        };
        let kms = self
            .kms
            .as_ref()
            .ok_or(QuarantineError::EncryptionUnavailable)?;
        let envelope = encode_replay_envelope(message, identity, body, digest)?;
        let encrypted = tokio::time::timeout(
            KMS_ENCRYPT_TIMEOUT,
            kms.encrypt(kms_scope, dek_id, &envelope),
        )
        .await
        .map_err(|_| QuarantineError::EncryptionTimeout)?
        .map_err(QuarantineError::Kms)?;
        if encrypted.tenant_id != *kms_scope || encrypted.dek_id != *dek_id {
            return Err(QuarantineError::EncryptionUnavailable);
        }
        let encoded = serde_json::to_vec(&encrypted)?;
        if encoded.len() > MAX_ENCRYPTED_BLOB_BYTES {
            return Err(QuarantineError::EncryptedBlobTooLarge);
        }
        Ok(Some(encoded))
    }
}

#[cfg(feature = "jetstream-adapter")]
fn validate_source(source: &TrustedJetStreamSource) -> Result<(), QuarantineError> {
    let name = source.stream_name.as_bytes();
    if source.deployment_id.is_nil()
        || source.stream_generation.is_nil()
        || name.is_empty()
        || name.len() > 255
        || !name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(QuarantineError::InvalidSource);
    }
    Ok(())
}

fn validate_purge_batch(batch_limit: i64) -> Result<(), QuarantineError> {
    if !(1..=100).contains(&batch_limit) {
        return Err(QuarantineError::InvalidPurgeBatch);
    }
    Ok(())
}

#[cfg(feature = "jetstream-adapter")]
fn source_identity(
    source: &TrustedJetStreamSource,
    message: &Message,
) -> Result<JetStreamSourceIdentity, QuarantineError> {
    let info = message.info().map_err(QuarantineError::SourceMetadata)?;
    if info.stream != source.stream_name || info.stream_sequence == 0 {
        return Err(QuarantineError::UnexpectedStream);
    }
    Ok(JetStreamSourceIdentity {
        deployment_id: source.deployment_id,
        stream_generation: source.stream_generation,
        stream_name: info.stream.to_owned(),
        stream_sequence: i64::try_from(info.stream_sequence)
            .map_err(|_| QuarantineError::UnexpectedStream)?,
    })
}

#[cfg(feature = "jetstream-adapter")]
fn payload_digest(payload: &[u8]) -> [u8; 32] {
    Sha256::digest(payload).into()
}

#[derive(Serialize)]
#[cfg(feature = "jetstream-adapter")]
struct ReplayEnvelope<'a> {
    version: u16,
    deployment_id: Uuid,
    stream_generation: Uuid,
    stream_name: &'a str,
    stream_sequence: i64,
    subject: &'a str,
    nats_message_ids: Vec<String>,
    payload_bytes: usize,
    payload_sha256: String,
    payload_base64: String,
}

#[cfg(feature = "jetstream-adapter")]
fn encode_replay_envelope(
    message: &Message,
    identity: &JetStreamSourceIdentity,
    body: &[u8],
    digest: &[u8; 32],
) -> Result<Vec<u8>, QuarantineError> {
    let mut nats_message_ids = Vec::new();
    let mut header_count = 0usize;
    let mut header_bytes = 0usize;
    if let Some(map) = message.headers.as_ref() {
        for value in map.get_all("Nats-Msg-Id") {
            header_count = header_count.saturating_add(1);
            header_bytes = header_bytes.saturating_add(value.as_str().len());
            validate_replay_header_limits(header_count, header_bytes)?;
            nats_message_ids.push(value.as_str().to_owned());
        }
    }
    let envelope = ReplayEnvelope {
        version: REPLAY_ENVELOPE_VERSION,
        deployment_id: identity.deployment_id,
        stream_generation: identity.stream_generation,
        stream_name: &identity.stream_name,
        stream_sequence: identity.stream_sequence,
        subject: message.subject.as_str(),
        nats_message_ids,
        payload_bytes: body.len(),
        payload_sha256: hex_digest(digest),
        payload_base64: BASE64.encode(body),
    };
    let bytes = serde_json::to_vec(&envelope)?;
    if bytes.len() > MAX_ENCRYPTED_BLOB_BYTES {
        return Err(QuarantineError::EncryptedBlobTooLarge);
    }
    Ok(bytes)
}

#[cfg(feature = "jetstream-adapter")]
fn validate_replay_header_limits(count: usize, total_bytes: usize) -> Result<(), QuarantineError> {
    if count > MAX_REPLAY_HEADER_COUNT || total_bytes > MAX_REPLAY_HEADER_BYTES {
        return Err(QuarantineError::HeadersTooLarge);
    }
    Ok(())
}

#[cfg(feature = "jetstream-adapter")]
fn hex_digest(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    output
}

#[cfg(feature = "jetstream-adapter")]
async fn set_platform_ingress_scope(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
) -> Result<(), QuarantineError> {
    sqlx::query("SET LOCAL ROLE star_platform_ingress_owner")
        .execute(&mut **tx)
        .await?;
    set_platform_deployment_id(tx, deployment_id).await
}

async fn set_platform_purger_scope(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
) -> Result<(), QuarantineError> {
    sqlx::query("SET LOCAL ROLE star_platform_quarantine_purger")
        .execute(&mut **tx)
        .await?;
    set_platform_deployment_id(tx, deployment_id).await
}

async fn set_platform_deployment_id(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
) -> Result<(), QuarantineError> {
    sqlx::query("SELECT set_config('app.platform_deployment_id', $1, true)")
        .bind(deployment_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(feature = "jetstream-adapter")]
async fn verify_existing_receipt(
    tx: &mut Transaction<'_, Postgres>,
    identity: &JetStreamSourceIdentity,
    fault: PermanentPoisonFault,
    parser_version: i16,
    body_len: i32,
    digest: &[u8; 32],
) -> Result<(), QuarantineError> {
    let row = sqlx::query(
        r#"SELECT fault_code, classifier_version, parser_version, payload_bytes, payload_sha256
           FROM platform_messaging.poison_quarantine_receipt
           WHERE deployment_id = $1 AND stream_generation = $2
             AND stream_name = $3 AND stream_sequence = $4"#,
    )
    .bind(identity.deployment_id)
    .bind(identity.stream_generation)
    .bind(&identity.stream_name)
    .bind(identity.stream_sequence)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(QuarantineError::IdempotencyConflict)?;
    verify_existing_row(&row, fault, parser_version, body_len, digest)
}

#[cfg(feature = "jetstream-adapter")]
fn verify_existing_row(
    row: &sqlx::postgres::PgRow,
    fault: PermanentPoisonFault,
    parser_version: i16,
    body_len: i32,
    digest: &[u8; 32],
) -> Result<(), QuarantineError> {
    let same = row.try_get::<String, _>("fault_code")? == fault.code()
        && row.try_get::<i16, _>("classifier_version")? == CLASSIFIER_VERSION
        && row.try_get::<i16, _>("parser_version")? == parser_version
        && row.try_get::<i32, _>("payload_bytes")? == body_len
        && row.try_get::<Vec<u8>, _>("payload_sha256")?.as_slice() == digest;
    if same {
        Ok(())
    } else {
        Err(QuarantineError::IdempotencyConflict)
    }
}

#[cfg(all(test, feature = "jetstream-adapter"))]
mod tests {
    use super::*;

    #[test]
    fn trusted_source_rejects_missing_or_invalid_identity() {
        assert!(matches!(
            TrustedJetStreamSource::new(Uuid::nil(), Uuid::new_v4(), "STAR_RESULTS"),
            Err(QuarantineError::InvalidSource)
        ));
        assert!(matches!(
            TrustedJetStreamSource::new(Uuid::new_v4(), Uuid::new_v4(), "bad.stream.name"),
            Err(QuarantineError::InvalidSource)
        ));
        assert!(
            TrustedJetStreamSource::new(Uuid::new_v4(), Uuid::new_v4(), "STAR_RESULTS_V2").is_ok()
        );
    }

    #[test]
    fn fault_codes_are_stable_and_do_not_include_run_owner_data_corruption() {
        assert_eq!(
            PermanentPoisonFault::InvalidCapacityResultEvent.code(),
            "invalid_capacity_result_event"
        );
        assert_eq!(
            PermanentPoisonFault::MalformedCapacityResultJson.code(),
            "malformed_capacity_result_json"
        );
        assert_eq!(
            PermanentPoisonFault::CapacityResultEventIdConflict.code(),
            "capacity_result_event_id_conflict"
        );
    }

    #[test]
    fn source_payload_digest_is_sha256_over_exact_bytes() {
        let payload = b"{\"truncated\":true";
        assert_eq!(hex_digest(&payload_digest(payload)).len(), 64);
        assert_ne!(
            payload_digest(payload),
            payload_digest(b"{\"truncated\":true}")
        );
    }

    #[test]
    fn policies_are_explicit_and_have_distinct_database_codes() {
        assert_eq!(
            PayloadRetentionPolicy::MetadataOnly.database_code(),
            "metadata_only"
        );
        assert_eq!(
            PayloadRetentionPolicy::Encrypted24Hours {
                kms_scope: "deployment".to_owned(),
                dek_id: KeyId::new("platform-quarantine-v1"),
            }
            .database_code(),
            "encrypted_24h"
        );
    }

    #[test]
    fn ttl_purge_batches_are_bounded() {
        assert!(validate_purge_batch(1).is_ok());
        assert!(validate_purge_batch(100).is_ok());
        assert!(matches!(
            validate_purge_batch(0),
            Err(QuarantineError::InvalidPurgeBatch)
        ));
        assert!(matches!(
            validate_purge_batch(101),
            Err(QuarantineError::InvalidPurgeBatch)
        ));
    }

    #[test]
    fn replay_headers_are_bounded_by_count_and_bytes() {
        assert!(
            validate_replay_header_limits(MAX_REPLAY_HEADER_COUNT, MAX_REPLAY_HEADER_BYTES).is_ok()
        );
        assert!(matches!(
            validate_replay_header_limits(MAX_REPLAY_HEADER_COUNT + 1, 0),
            Err(QuarantineError::HeadersTooLarge)
        ));
        assert!(matches!(
            validate_replay_header_limits(1, MAX_REPLAY_HEADER_BYTES + 1),
            Err(QuarantineError::HeadersTooLarge)
        ));
    }
}
