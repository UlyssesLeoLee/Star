/*
@cypher schema=1 source_sha256=d8ba1f571c783e473f9e2981174b8ee0a04b5c907e537d882bacb38aca7826f0
MERGE (self:File {path:"crates/platform-messaging-runtime/src/admission_delivery.rs"})
MERGE (jetstream_feature:Config {id:"cargo.feature.platform-messaging-runtime.jetstream-adapter"})
MERGE (store:Type {id:"platform_messaging_runtime::AdmissionDeliveryStore",kind:"struct"})
MERGE (claim:Type {id:"platform_messaging_runtime::AdmissionDeliveryClaim",kind:"struct"})
MERGE (owner:Type {id:"platform_messaging_runtime::AdmissionDeliveryOwner",kind:"enum"})
MERGE (publisher:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionPublisher",kind:"struct"})
MERGE (claim_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryStore::claim",kind:"method"})
MERGE (settle_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryStore::settle",kind:"method"})
MERGE (reconcile_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryStore::reconcile",kind:"method"})
MERGE (publish_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionPublisher::publish_due",kind:"method"})
MERGE (validate_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::validate_wire",kind:"function"})
MERGE (scope_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryStore::scope",kind:"method"})
MERGE (transport:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransport",kind:"trait"})
MERGE (jetstream:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::JetStreamAdmissionTransport",kind:"struct"})
MERGE (profile:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile",kind:"struct"})
MERGE (transport_publish:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::JetStreamAdmissionTransport::publish",kind:"method"})
MERGE (transport_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::JetStreamAdmissionTransport::new",kind:"method"})
MERGE (transport_port:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransport::publish",kind:"method"})
MERGE (transport_profile:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransport::profile",kind:"method"})
MERGE (profile_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile::new",kind:"method"})
MERGE (profile_valid:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile::is_valid",kind:"method"})
MERGE (receipt_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportReceipt::new",kind:"method"})
MERGE (validate_receipt:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::transport_receipt_matches",kind:"function"})
MERGE (provider_id:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::valid_provider_id",kind:"function"})
MERGE (provider_version:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::valid_transport_version",kind:"function"})
MERGE (transport_label:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::valid_transport_label",kind:"function"})
MERGE (with_transport:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionPublisher::with_transport",kind:"method"})
MERGE (receipt:Type {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportReceipt",kind:"struct"})
MERGE (self)-[:DEFINES]->(transport)
MERGE (self)-[:DEPENDS_ON]->(jetstream_feature)
MERGE (self)-[:DEFINES]->(jetstream)
MERGE (self)-[:DEFINES]->(transport_publish)
MERGE (self)-[:DEFINES]->(transport_new)
MERGE (self)-[:DEFINES]->(transport_port)
MERGE (self)-[:DEFINES]->(transport_profile)
MERGE (self)-[:DEFINES]->(profile_new)
MERGE (self)-[:DEFINES]->(profile_valid)
MERGE (self)-[:DEFINES]->(receipt_new)
MERGE (self)-[:DEFINES]->(validate_receipt)
MERGE (self)-[:DEFINES]->(provider_id)
MERGE (self)-[:DEFINES]->(provider_version)
MERGE (self)-[:DEFINES]->(transport_label)
MERGE (self)-[:DEFINES]->(with_transport)
MERGE (self)-[:DEFINES]->(receipt)
MERGE (self)-[:DEFINES]->(profile)
MERGE (jetstream)-[:IMPLEMENTS]->(transport)
MERGE (publish_fn)-[:CALLS]->(transport_port)
MERGE (publisher)-[:USES_TYPE]->(transport)
MERGE (transport_publish)-[:USES_TYPE]->(receipt)
MERGE (transport)-[:USES_TYPE]->(profile)
MERGE (transport_profile)-[:USES_TYPE]->(profile)
MERGE (profile_new)-[:CALLS]->(profile_valid)
MERGE (profile_valid)-[:CALLS]->(provider_id)
MERGE (profile_valid)-[:CALLS]->(provider_version)
MERGE (profile_valid)-[:CALLS]->(transport_label)
MERGE (receipt_new)-[:USES_TYPE]->(profile)
MERGE (receipt_new)-[:CALLS]->(transport_label)
MERGE (receipt_new)-[:CALLS]->(sha256:ExternalService {id:"sha2::Sha256::digest",kind:"method"})
MERGE (validate_receipt)-[:CALLS]->(sha256)
MERGE (transport_publish)-[:CALLS]->(receipt_new)
MERGE (publish_fn)-[:CALLS]->(validate_receipt)
MERGE (settle_fn)-[:CALLS]->(validate_receipt)
MERGE (publish_fn)-[:CALLS]->(transport_profile)
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(claim)
MERGE (self)-[:DEFINES]->(owner)
MERGE (self)-[:DEFINES]->(publisher)
MERGE (self)-[:DEFINES]->(claim_fn)
MERGE (self)-[:DEFINES]->(settle_fn)
MERGE (self)-[:DEFINES]->(reconcile_fn)
MERGE (self)-[:DEFINES]->(publish_fn)
MERGE (self)-[:DEFINES]->(validate_fn)
MERGE (self)-[:DEFINES]->(scope_fn)
MERGE (store_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryStore::new",kind:"method"})
MERGE (publisher_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionPublisher::new",kind:"method"})
MERGE (owner_schema:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryOwner::schema",kind:"method"})
MERGE (owner_role:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryOwner::role",kind:"method"})
MERGE (owner_permits:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionDeliveryOwner::permits",kind:"method"})
MERGE (self)-[:DEFINES]->(store_new)
MERGE (self)-[:DEFINES]->(publisher_new)
MERGE (publisher_new)-[:CALLS]->(transport_new)
MERGE (publisher_new)-[:CALLS]->(with_transport)
MERGE (self)-[:DEFINES]->(owner_schema)
MERGE (self)-[:DEFINES]->(owner_role)
MERGE (self)-[:DEFINES]->(owner_permits)
MERGE (scope_fn)-[:CALLS]->(owner_role)
MERGE (claim_fn)-[:CALLS]->(owner_schema)
MERGE (claim_fn)-[:CALLS]->(profile_valid)
MERGE (reconcile_fn)-[:CALLS]->(owner_schema)
MERGE (settle_fn)-[:CALLS]->(owner_schema)
MERGE (settle_fn)-[:CALLS]->(owner_permits)
MERGE (transport_publish)-[:CALLS]->(owner_schema)
MERGE (validate_fn)-[:CALLS]->(owner_permits)
MERGE (broker_name_fn:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::valid_broker_name",kind:"function"})
MERGE (self)-[:DEFINES]->(broker_name_fn)
MERGE (transport_new)-[:CALLS]->(broker_name_fn)
MERGE (transport_new)-[:CALLS]->(profile_valid)
MERGE (publish_fn)-[:CALLS]->(claim_fn)
MERGE (publish_fn)-[:CALLS]->(settle_fn)
MERGE (publish_fn)-[:CALLS]->(validate_fn)
MERGE (claim_fn)-[:CALLS]->(scope_fn)
MERGE (settle_fn)-[:CALLS]->(scope_fn)
MERGE (reconcile_fn)-[:CALLS]->(scope_fn)
MERGE (reconcile_fn)-[:CALLS]->(schedule_reconcile:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::automation.reconcile_admission_delivery",kind:"function"})
MERGE (reconcile_fn)-[:CALLS]->(run_reconcile:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::multica.reconcile_admission_delivery",kind:"function"})
MERGE (claim_fn)-[:CALLS]->(schedule_claim:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::automation.claim_admission_delivery_v2",kind:"function"})
MERGE (claim_fn)-[:CALLS]->(run_claim:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::multica.claim_admission_delivery_v2",kind:"function"})
MERGE (settle_fn)-[:CALLS]->(schedule_settle:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::automation.settle_admission_delivery_v2",kind:"function"})
MERGE (settle_fn)-[:CALLS]->(run_settle:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::multica.settle_admission_delivery_v2",kind:"function"})
MERGE (transport_publish)-[:CALLS]->(send:ExternalService {id:"async_nats::jetstream::Context::send_publish",kind:"method"})
MERGE (validate_fn)-[:USES_TYPE]->(command:Type {id:"domain_automation::schedule::ScheduleRunAdmissionCommandV1"})
MERGE (validate_fn)-[:USES_TYPE]->(result:Type {id:"domain_automation::schedule::ScheduleRunAdmissionResultV1"})
MERGE (validate_fn)-[:USES_TYPE]->(resolution:Type {id:"domain_automation::schedule::ScheduleRunAdmissionResolutionV1"})
MERGE (wire_test:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::tests::result_wire_is_owner_and_identity_bound",kind:"test"})
MERGE (limits_test:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::tests::wire_limits_and_transport_names_are_explicit",kind:"test"})
MERGE (receipt_test:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::tests::receipt_binds_exact_wire_to_pinned_profile_and_log_generation",kind:"test"})
MERGE (profile_test:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::tests::transport_profile_rejects_ambiguous_or_secret_like_identity",kind:"test"})
MERGE (self)-[:DEFINES]->(wire_test)
MERGE (self)-[:DEFINES]->(limits_test)
MERGE (self)-[:DEFINES]->(receipt_test)
MERGE (self)-[:DEFINES]->(profile_test)
MERGE (wire_test)-[:TESTS]->(validate_fn)
MERGE (limits_test)-[:TESTS]->(validate_fn)
MERGE (limits_test)-[:TESTS]->(broker_name_fn)
MERGE (receipt_test)-[:TESTS]->(validate_receipt)
MERGE (profile_test)-[:TESTS]->(profile_new)
MERGE (test_fixture:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::tests::result_claim",kind:"function"})
MERGE (self)-[:DEFINES]->(test_fixture)
MERGE (wire_test)-[:CALLS]->(test_fixture)
MERGE (limits_test)-[:CALLS]->(test_fixture)
@endcypher
*/
//! Shared adapter code executed in one pinned business owner's process and database role.
//! It is not a central Platform service with permission to mutate both owners.

use std::time::Duration;

#[cfg(feature = "jetstream-adapter")]
use async_nats::jetstream::{Context, message::PublishMessage};
use chrono::{DateTime, Utc};
use domain_automation::schedule::{
    ScheduleRunAdmissionCommandV1, ScheduleRunAdmissionResolutionV1, ScheduleRunAdmissionResultV1,
};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use tokio::sync::watch;
use uuid::Uuid;

/// Database owner pinned at construction; schema and role are compile-time allowlists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionDeliveryOwner {
    /// Schedule owns request and resolution delivery.
    Schedule,
    /// Run owns result delivery.
    Run,
}

impl AdmissionDeliveryOwner {
    fn schema(self) -> &'static str {
        match self {
            Self::Schedule => "automation",
            Self::Run => "multica",
        }
    }
    fn role(self) -> &'static str {
        match self {
            Self::Schedule => "star_automation_schedule_owner",
            Self::Run => "star_multica_run_owner",
        }
    }
    fn permits(self, kind: &str) -> bool {
        match self {
            Self::Schedule => matches!(kind, "request" | "resolution"),
            Self::Run => kind == "result",
        }
    }
}

/// A bounded lease over an immutable original Outbox wire.
#[derive(Debug, Clone)]
pub struct AdmissionDeliveryClaim {
    /// Tenant supplied by trusted deployment routing, not discovered from message headers.
    pub tenant_id: Uuid,
    /// Owner worker holding this delivery lease.
    pub worker_id: Uuid,
    /// Stable request/result/resolution identity, namespaced by owner and kind.
    pub message_id: Uuid,
    /// Original request identity.
    pub request_id: Uuid,
    /// One of request/result/resolution, validated against the pinned owner.
    pub message_kind: String,
    /// Receiver principal pinned by the original owner fact.
    pub target_service_id: Uuid,
    /// Exact original typed wire; no reserialization before publishing.
    pub wire: Vec<u8>,
    /// Monotonically increasing delivery attempt; independent of admission attempt.
    pub attempt_count: i32,
    /// Database-clock lease expiry.
    pub lease_expires_at: DateTime<Utc>,
    /// Immutable transport profile selected on the first claim and reused by every retry.
    pub transport_profile: AdmissionTransportProfile,
}

/// Bounded adapter failures; no payload or credentials are included in messages.
#[derive(Debug, Error)]
pub enum AdmissionDeliveryError {
    /// Tenant/worker/batch/lease/transport profile configuration is invalid.
    #[error("invalid admission delivery configuration")]
    InvalidConfiguration,
    /// The immutable source wire does not match the claimed identity and owner.
    #[error("invalid stored admission message")]
    InvalidMessage,
    /// The pinned owner role or fenced native operation failed.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// One owner's pool and native procedure adapter. Each operation uses its own owner transaction.
#[derive(Debug, Clone)]
pub struct AdmissionDeliveryStore {
    pool: PgPool,
    owner: AdmissionDeliveryOwner,
}

impl AdmissionDeliveryStore {
    /// Pin the owner and its deployment-managed pool. Role membership must be provisioned separately.
    pub fn new(pool: PgPool, owner: AdmissionDeliveryOwner) -> Self {
        Self { pool, owner }
    }

    async fn scope(
        &self,
        tenant: Uuid,
    ) -> Result<Transaction<'static, Postgres>, AdmissionDeliveryError> {
        if tenant.is_nil() {
            return Err(AdmissionDeliveryError::InvalidConfiguration);
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query(&format!("SET LOCAL ROLE {}", self.owner.role()))
            .execute(&mut *tx)
            .await?;
        sqlx::query("SET LOCAL lock_timeout='5s'")
            .execute(&mut *tx)
            .await?;
        sqlx::query("SET LOCAL statement_timeout='10s'")
            .execute(&mut *tx)
            .await?;
        sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
            .bind(tenant.to_string())
            .execute(&mut *tx)
            .await?;
        Ok(tx)
    }

    /// Reconcile at most eight old Outbox facts; permanent receipts prevent resurrection.
    pub async fn reconcile(&self, tenant: Uuid, limit: i32) -> Result<i32, AdmissionDeliveryError> {
        if !(1..=8).contains(&limit) {
            return Err(AdmissionDeliveryError::InvalidConfiguration);
        }
        let mut tx = self.scope(tenant).await?;
        let added = sqlx::query_scalar(&format!(
            "SELECT {}.reconcile_admission_delivery($1)",
            self.owner.schema()
        ))
        .bind(limit)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(added)
    }

    /// Lease one bounded page using database time, SKIP LOCKED and a monotonic attempt fence.
    pub async fn claim(
        &self,
        tenant: Uuid,
        worker: Uuid,
        limit: i32,
        lease_seconds: i32,
        profile: &AdmissionTransportProfile,
    ) -> Result<Vec<AdmissionDeliveryClaim>, AdmissionDeliveryError> {
        if worker.is_nil()
            || !(1..=8).contains(&limit)
            || !(30..=300).contains(&lease_seconds)
            || !profile.is_valid()
        {
            return Err(AdmissionDeliveryError::InvalidConfiguration);
        }
        let mut tx = self.scope(tenant).await?;
        let rows = sqlx::query(&format!(
            "SELECT * FROM {}.claim_admission_delivery_v2($1,$2,$3,$4,$5,$6,$7)",
            self.owner.schema()
        ))
        .bind(worker)
        .bind(limit)
        .bind(lease_seconds)
        .bind(&profile.provider_id)
        .bind(&profile.provider_version)
        .bind(profile.profile_digest.as_slice())
        .bind(&profile.endpoint_identity)
        .fetch_all(&mut *tx)
        .await?;
        let mut claims = Vec::with_capacity(rows.len());
        for row in &rows {
            let digest: Vec<u8> = row.try_get("transport_profile_digest")?;
            let digest: [u8; 32] = digest
                .as_slice()
                .try_into()
                .map_err(|_| AdmissionDeliveryError::InvalidConfiguration)?;
            let pinned_profile = AdmissionTransportProfile {
                provider_id: row.try_get("transport_provider_id")?,
                provider_version: row.try_get("transport_provider_version")?,
                profile_digest: digest,
                endpoint_identity: row.try_get("transport_endpoint_identity")?,
            };
            if &pinned_profile != profile {
                return Err(AdmissionDeliveryError::InvalidConfiguration);
            }
            claims.push(AdmissionDeliveryClaim {
                tenant_id: tenant,
                worker_id: worker,
                message_id: row.try_get("message_id")?,
                request_id: row.try_get("request_id")?,
                message_kind: row.try_get("message_kind")?,
                target_service_id: row.try_get("target_service_id")?,
                wire: row.try_get("wire")?,
                attempt_count: row.try_get("attempt_count")?,
                lease_expires_at: row.try_get("lease_expires_at")?,
                transport_profile: pinned_profile,
            });
        }
        tx.commit().await?;
        Ok(claims)
    }

    /// Settle an unexpired exact lease. Delivered requires a verified durable transport receipt.
    /// Retry is bounded; dead and exhausted claims leave permanent owner-local receipts.
    pub async fn settle(
        &self,
        claim: &AdmissionDeliveryClaim,
        action: AdmissionDeliveryAction,
    ) -> Result<(), AdmissionDeliveryError> {
        if claim.worker_id.is_nil()
            || claim.message_id.is_nil()
            || !(1..=12).contains(&claim.attempt_count)
            || !self.owner.permits(&claim.message_kind)
        {
            return Err(AdmissionDeliveryError::InvalidConfiguration);
        }
        let (mode, receipt) = match action {
            AdmissionDeliveryAction::Delivered { receipt } => {
                if !transport_receipt_matches(claim, &claim.transport_profile, &receipt) {
                    return Err(AdmissionDeliveryError::InvalidConfiguration);
                }
                ("delivered", Some(receipt))
            }
            AdmissionDeliveryAction::Retry => ("retry", None),
            AdmissionDeliveryAction::Dead => ("dead", None),
        };
        let mut tx = self.scope(claim.tenant_id).await?;
        sqlx::query(&format!(
            "SELECT {}.settle_admission_delivery_v2($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
            self.owner.schema()
        ))
        .bind(claim.message_id)
        .bind(&claim.message_kind)
        .bind(claim.worker_id)
        .bind(claim.attempt_count)
        .bind(mode)
        .bind(receipt.as_ref().map(|r| r.profile.provider_id.as_str()))
        .bind(
            receipt
                .as_ref()
                .map(|r| r.profile.provider_version.as_str()),
        )
        .bind(
            receipt
                .as_ref()
                .map(|r| r.profile.profile_digest.as_slice()),
        )
        .bind(
            receipt
                .as_ref()
                .map(|r| r.profile.endpoint_identity.as_str()),
        )
        .bind(receipt.as_ref().map(|r| r.log_id.as_str()))
        .bind(receipt.as_ref().map(|r| r.log_generation))
        .bind(receipt.as_ref().map(|r| r.cursor.as_slice()))
        .bind(receipt.as_ref().map(|r| r.payload_sha256.as_slice()))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

/// Settlement chosen only after transport or typed-wire validation.
#[derive(Debug)]
pub enum AdmissionDeliveryAction {
    /// Verified provider/profile-scoped durable acceptance proof.
    Delivered {
        /// Receipt must match the claim's pinned profile and exact wire digest.
        receipt: AdmissionTransportReceipt,
    },
    /// Transport timeout/failure; retry the exact same bytes and stable ID.
    Retry,
    /// Invalid immutable source wire; audit terminal failure without publishing it.
    Dead,
}

/// One bounded publishing pass; cancellation leaves unstarted claims for lease recovery.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionPublishOutcome {
    /// Leased message count (at most eight).
    pub claimed: usize,
    /// Verified durable acceptance and committed owner delivery receipt count.
    pub delivered: usize,
    /// Durably scheduled transport retries (including bounded exhaustion).
    pub retry_settlements: usize,
    /// Invalid source wires audited dead.
    pub invalid_messages: usize,
    /// Cancellation was observed before starting another publish.
    pub cancelled: bool,
}

/// Immutable, non-secret identity of the provider configuration used for a delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionTransportProfile {
    /// Stable provider key, such as `nats-jetstream` or `star-rust-log`.
    pub provider_id: String,
    /// Exact provider protocol/build contract version.
    pub provider_version: String,
    /// SHA-256 digest of the immutable profile content.
    pub profile_digest: [u8; 32],
    /// Stable endpoint label; this must not contain a URL, credential, or secret.
    pub endpoint_identity: String,
}

impl AdmissionTransportProfile {
    /// Build a validated snapshot. The digest is supplied by the trusted profile resolver.
    pub fn new(
        provider_id: String,
        provider_version: String,
        profile_digest: [u8; 32],
        endpoint_identity: String,
    ) -> Result<Self, AdmissionDeliveryError> {
        let profile = Self {
            provider_id,
            provider_version,
            profile_digest,
            endpoint_identity,
        };
        if profile.is_valid() {
            Ok(profile)
        } else {
            Err(AdmissionDeliveryError::InvalidConfiguration)
        }
    }

    fn is_valid(&self) -> bool {
        valid_provider_id(&self.provider_id)
            && valid_transport_version(&self.provider_version)
            && valid_transport_label(&self.endpoint_identity, 128)
    }
}

/// Provider-neutral proof that the exact message was durably accepted.
/// This is not a receiver Inbox commit or task execution proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionTransportReceipt {
    /// Profile identity pinned on the owner delivery row before its first attempt.
    pub profile: AdmissionTransportProfile,
    /// Durable log identity within the selected profile.
    pub log_id: String,
    /// Generation changes when a provider recreates a log with the same name.
    pub log_generation: Uuid,
    /// Opaque bounded durable cursor; providers are not required to use integers.
    pub cursor: Vec<u8>,
    /// SHA-256 of the exact immutable wire accepted by the provider.
    pub payload_sha256: [u8; 32],
}

impl AdmissionTransportReceipt {
    /// Create a proof over the exact wire bytes after the provider confirms durable acceptance.
    pub fn new(
        profile: AdmissionTransportProfile,
        log_id: String,
        log_generation: Uuid,
        cursor: Vec<u8>,
        wire: &[u8],
    ) -> Result<Self, AdmissionDeliveryError> {
        if !profile.is_valid()
            || !valid_transport_label(&log_id, 128)
            || log_generation.is_nil()
            || cursor.is_empty()
            || cursor.len() > 128
            || wire.is_empty()
            || wire.len() > 16 * 1024
        {
            return Err(AdmissionDeliveryError::InvalidConfiguration);
        }
        Ok(Self {
            profile,
            log_id,
            log_generation,
            cursor,
            payload_sha256: Sha256::digest(wire).into(),
        })
    }
}

/// Bounded transport failure; credentials and payload are not exposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("admission transport did not prove durable acceptance")]
pub struct AdmissionTransportError;

/// Rust-owned replaceable transport port. Implementations authenticate the peer and pin its log.
/// Accept the original bytes and owner/kind/tenant/message dedupe identity; no business DB writes.
/// Native providers must prove durability/recovery before deployment enables them.
pub trait AdmissionTransport: Send + Sync {
    /// Profile identity supplied by trusted deployment composition and pinned by the owner store.
    fn profile(&self) -> &AdmissionTransportProfile;

    /// Publish one original message. The owner publisher bounds this future to ten seconds.
    fn publish<'a>(
        &'a self,
        owner: AdmissionDeliveryOwner,
        claim: &'a AdmissionDeliveryClaim,
    ) -> impl std::future::Future<
        Output = Result<AdmissionTransportReceipt, AdmissionTransportError>,
    > + Send
    + 'a;
}

/// Transitional JetStream adapter; all NATS subject/ID/header/PubAck mapping lives here.
#[cfg(feature = "jetstream-adapter")]
pub struct JetStreamAdmissionTransport {
    context: Context,
    stream: String,
    domain: String,
    stream_generation: Uuid,
    profile: AdmissionTransportProfile,
}

#[cfg(feature = "jetstream-adapter")]
impl JetStreamAdmissionTransport {
    /// Pin the already-authenticated Context's expected stream/domain; no provisioning or drain.
    pub fn new(
        context: Context,
        stream: String,
        domain: String,
        stream_generation: Uuid,
        profile: AdmissionTransportProfile,
    ) -> Result<Self, AdmissionDeliveryError> {
        if !valid_broker_name(&stream)
            || (!domain.is_empty() && !valid_broker_name(&domain))
            || stream_generation.is_nil()
            || !profile.is_valid()
            || profile.provider_id != "nats-jetstream"
        {
            return Err(AdmissionDeliveryError::InvalidConfiguration);
        }
        Ok(Self {
            context,
            stream,
            domain,
            stream_generation,
            profile,
        })
    }
}

#[cfg(feature = "jetstream-adapter")]
impl AdmissionTransport for JetStreamAdmissionTransport {
    fn profile(&self) -> &AdmissionTransportProfile {
        &self.profile
    }

    async fn publish(
        &self,
        owner: AdmissionDeliveryOwner,
        claim: &AdmissionDeliveryClaim,
    ) -> Result<AdmissionTransportReceipt, AdmissionTransportError> {
        if claim.transport_profile != self.profile {
            return Err(AdmissionTransportError);
        }
        let subject = format!(
            "star.{}.admission.v1.{}.{}.{}",
            owner.schema(),
            claim.tenant_id,
            claim.target_service_id,
            claim.message_kind
        );
        let id = format!(
            "admission.v1.{}.{}.{}.{}",
            owner.schema(),
            claim.message_kind,
            claim.tenant_id,
            claim.message_id
        );
        let message = PublishMessage::build()
            .payload(claim.wire.clone().into())
            .message_id(id)
            .expected_stream(&self.stream);
        let ack = self
            .context
            .send_publish(subject, message)
            .await
            .map_err(|_| AdmissionTransportError)?
            .await
            .map_err(|_| AdmissionTransportError)?;
        if ack.stream != self.stream || ack.domain != self.domain || ack.sequence == 0 {
            return Err(AdmissionTransportError);
        }
        AdmissionTransportReceipt::new(
            self.profile.clone(),
            self.stream.clone(),
            self.stream_generation,
            ack.sequence.to_be_bytes().to_vec(),
            &claim.wire,
        )
        .map_err(|_| AdmissionTransportError)
    }
}

/// Owner-local durable publishing loop, independent of a broker API.
/// Static transport dispatch avoids a boxed future or payload clone in the core.
pub struct AdmissionPublisher<T: AdmissionTransport> {
    store: AdmissionDeliveryStore,
    transport: T,
}

/// Compatibility entry point for the transitional NATS provider.
#[cfg(feature = "jetstream-adapter")]
pub type AdmissionJetStreamPublisher = AdmissionPublisher<JetStreamAdmissionTransport>;

#[cfg(feature = "jetstream-adapter")]
impl AdmissionPublisher<JetStreamAdmissionTransport> {
    /// Construct the transitional adapter; owner-local delivery policy remains transport-neutral.
    pub fn new(
        store: AdmissionDeliveryStore,
        context: Context,
        stream: String,
        domain: String,
        stream_generation: Uuid,
        profile: AdmissionTransportProfile,
    ) -> Result<Self, AdmissionDeliveryError> {
        Ok(Self::with_transport(
            store,
            JetStreamAdmissionTransport::new(context, stream, domain, stream_generation, profile)?,
        ))
    }
}

impl<T: AdmissionTransport> AdmissionPublisher<T> {
    /// Inject a trusted deployment-managed provider. This is not provider readiness or authentication.
    pub fn with_transport(store: AdmissionDeliveryStore, transport: T) -> Self {
        Self { store, transport }
    }

    /// Publish at most eight messages serially, with ten-second transport timeout and 120-second lease.
    /// A crash after broker acceptance but before DB receipt replays the same identity/wire.
    pub async fn publish_due(
        &self,
        tenant: Uuid,
        worker: Uuid,
        limit: i32,
        cancellation: &watch::Receiver<bool>,
    ) -> Result<AdmissionPublishOutcome, AdmissionDeliveryError> {
        let mut outcome = AdmissionPublishOutcome::default();
        if *cancellation.borrow() {
            outcome.cancelled = true;
            return Ok(outcome);
        }
        let claims = self
            .store
            .claim(tenant, worker, limit, 120, self.transport.profile())
            .await?;
        outcome.claimed = claims.len();
        for claim in claims {
            if *cancellation.borrow() {
                outcome.cancelled = true;
                break;
            }
            if claim.transport_profile != *self.transport.profile() {
                self.store
                    .settle(&claim, AdmissionDeliveryAction::Retry)
                    .await?;
                outcome.retry_settlements += 1;
                continue;
            }
            if validate_wire(self.store.owner, &claim).is_err() {
                self.store
                    .settle(&claim, AdmissionDeliveryAction::Dead)
                    .await?;
                outcome.invalid_messages += 1;
                continue;
            }
            let ack = tokio::time::timeout(
                Duration::from_secs(10),
                self.transport.publish(self.store.owner, &claim),
            )
            .await;
            match ack {
                Ok(Ok(receipt))
                    if transport_receipt_matches(&claim, self.transport.profile(), &receipt) =>
                {
                    self.store
                        .settle(&claim, AdmissionDeliveryAction::Delivered { receipt })
                        .await?;
                    outcome.delivered += 1;
                }
                _ => {
                    self.store
                        .settle(&claim, AdmissionDeliveryAction::Retry)
                        .await?;
                    outcome.retry_settlements += 1;
                }
            }
        }
        Ok(outcome)
    }
}

#[cfg(feature = "jetstream-adapter")]
fn valid_broker_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_transport_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
}

fn valid_transport_label(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
}

fn transport_receipt_matches(
    claim: &AdmissionDeliveryClaim,
    expected_profile: &AdmissionTransportProfile,
    receipt: &AdmissionTransportReceipt,
) -> bool {
    let expected_payload_sha256: [u8; 32] = Sha256::digest(&claim.wire).into();
    receipt.profile == *expected_profile
        && claim.transport_profile == *expected_profile
        && expected_profile.is_valid()
        && valid_transport_label(&receipt.log_id, 128)
        && !receipt.log_generation.is_nil()
        && !receipt.cursor.is_empty()
        && receipt.cursor.len() <= 128
        && receipt.payload_sha256 == expected_payload_sha256
}

fn validate_wire(
    owner: AdmissionDeliveryOwner,
    claim: &AdmissionDeliveryClaim,
) -> Result<(), AdmissionDeliveryError> {
    if !owner.permits(&claim.message_kind) || claim.target_service_id.is_nil() {
        return Err(AdmissionDeliveryError::InvalidMessage);
    }
    let identity = match claim.message_kind.as_str() {
        "request" => ScheduleRunAdmissionCommandV1::decode_json(&claim.wire)
            .map(|v| (v.tenant_id, v.request_id, v.request_id)),
        "result" => ScheduleRunAdmissionResultV1::decode_json(&claim.wire)
            .map(|v| (v.tenant_id, v.request_id, v.result_id)),
        "resolution" => ScheduleRunAdmissionResolutionV1::decode_json(&claim.wire)
            .map(|v| (v.tenant_id, v.request_id, v.resolution_id)),
        _ => return Err(AdmissionDeliveryError::InvalidMessage),
    }
    .map_err(|_| AdmissionDeliveryError::InvalidMessage)?;
    if identity != (claim.tenant_id, claim.request_id, claim.message_id) {
        return Err(AdmissionDeliveryError::InvalidMessage);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain_automation::schedule::{
        ScheduleAdmissionRejectionCodeV1, ScheduleRunAdmissionOutcomeV1,
    };

    fn test_profile() -> AdmissionTransportProfile {
        AdmissionTransportProfile::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            [7; 32],
            "test-log".to_owned(),
        )
        .unwrap()
    }

    fn result_claim() -> AdmissionDeliveryClaim {
        let tenant = Uuid::new_v4();
        let request = Uuid::new_v4();
        let result = Uuid::new_v4();
        let wire = ScheduleRunAdmissionResultV1 {
            schema_version: 1,
            tenant_id: tenant,
            request_id: request,
            result_id: result,
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
        AdmissionDeliveryClaim {
            tenant_id: tenant,
            worker_id: Uuid::new_v4(),
            message_id: result,
            request_id: request,
            message_kind: "result".to_owned(),
            target_service_id: Uuid::new_v4(),
            wire,
            attempt_count: 1,
            lease_expires_at: Utc::now() + chrono::Duration::seconds(120),
            transport_profile: test_profile(),
        }
    }

    #[test]
    fn result_wire_is_owner_and_identity_bound() {
        let mut c = result_claim();
        assert!(validate_wire(AdmissionDeliveryOwner::Run, &c).is_ok());
        assert!(validate_wire(AdmissionDeliveryOwner::Schedule, &c).is_err());
        c.message_id = Uuid::new_v4();
        assert!(validate_wire(AdmissionDeliveryOwner::Run, &c).is_err());
        c = result_claim();
        c.tenant_id = Uuid::new_v4();
        assert!(validate_wire(AdmissionDeliveryOwner::Run, &c).is_err());
        c = result_claim();
        c.target_service_id = Uuid::nil();
        assert!(validate_wire(AdmissionDeliveryOwner::Run, &c).is_err());
    }

    #[test]
    fn wire_limits_and_transport_names_are_explicit() {
        let mut c = result_claim();
        c.wire = vec![b' '; 16385];
        assert!(validate_wire(AdmissionDeliveryOwner::Run, &c).is_err());
        c.wire = b"{}".to_vec();
        assert!(validate_wire(AdmissionDeliveryOwner::Run, &c).is_err());
        #[cfg(feature = "jetstream-adapter")]
        {
            assert!(valid_broker_name("ADMISSION_01"));
            for name in ["", "a.>", "*", "a/b", "a b"] {
                assert!(!valid_broker_name(name));
            }
            assert!(!valid_broker_name(&"a".repeat(129)));
        }
    }

    #[test]
    fn receipt_binds_exact_wire_to_pinned_profile_and_log_generation() {
        let claim = result_claim();
        let mut receipt = AdmissionTransportReceipt::new(
            claim.transport_profile.clone(),
            "results-v1".to_owned(),
            Uuid::new_v4(),
            42_i64.to_be_bytes().to_vec(),
            &claim.wire,
        )
        .unwrap();
        assert!(transport_receipt_matches(
            &claim,
            &claim.transport_profile,
            &receipt
        ));
        receipt.profile.profile_digest = [8; 32];
        assert!(!transport_receipt_matches(
            &claim,
            &claim.transport_profile,
            &receipt
        ));
    }

    #[test]
    fn transport_profile_rejects_ambiguous_or_secret_like_identity() {
        for (provider, version, endpoint) in [
            ("NATS", "2.12.14", "test"),
            ("star-rust-log", "", "test"),
            ("star-rust-log", "1.0", "nats://user:password@host"),
        ] {
            assert!(
                AdmissionTransportProfile::new(
                    provider.to_owned(),
                    version.to_owned(),
                    [1; 32],
                    endpoint.to_owned()
                )
                .is_err()
            );
        }
    }
}
