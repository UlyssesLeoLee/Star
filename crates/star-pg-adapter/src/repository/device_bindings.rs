// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=d5b70681fa6eab10551855109da248b6b30aa566691fe48c8485a807af414ced
MERGE (self:File {path:"crates/star-pg-adapter/src/repository/device_bindings.rs"})
MERGE (repo:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::PgDeviceBindingRepository",kind:"struct"})
MERGE (trait:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceBindingRepository",kind:"trait"})
MERGE (challenge:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceEnrollmentChallenge",kind:"struct"})
MERGE (binding:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceBinding",kind:"struct"})
MERGE (createResult:Type {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::DeviceChallengeCreateResult",kind:"enum"})
MERGE (challengeCap:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES",kind:"constant"})
MERGE (create:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::PgDeviceBindingRepository.create_challenge",kind:"method"})
MERGE (findChallenge:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::PgDeviceBindingRepository.find_challenge",kind:"method"})
MERGE (complete:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::PgDeviceBindingRepository.complete_challenge",kind:"method"})
MERGE (active:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::PgDeviceBindingRepository.is_active",kind:"method"})
MERGE (revoke:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::PgDeviceBindingRepository.revoke",kind:"method"})
MERGE (scope:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::set_actor_scope",kind:"function"})
MERGE (oauthOwnerRole:Type {id:"star_oauth_credential_owner",kind:"database_role"})
MERGE (ownerScope:Symbol {id:"crates/star-pg-adapter/src/repository/device_bindings.rs::set_oauth_owner_scope",kind:"function"})
MERGE (self)-[:DEFINES]->(repo),(self)-[:DEFINES]->(trait),(self)-[:DEFINES]->(challenge),(self)-[:DEFINES]->(binding),(self)-[:DEFINES]->(createResult),(self)-[:DEFINES]->(challengeCap)
MERGE (repo)-[:IMPLEMENTS]->(trait)
MERGE (repo)-[:HAS_METHOD]->(create),(repo)-[:HAS_METHOD]->(findChallenge),(repo)-[:HAS_METHOD]->(complete),(repo)-[:HAS_METHOD]->(active),(repo)-[:HAS_METHOD]->(revoke)
MERGE (create)-[:CALLS]->(ownerScope),(findChallenge)-[:CALLS]->(ownerScope),(complete)-[:CALLS]->(ownerScope),(active)-[:CALLS]->(ownerScope),(revoke)-[:CALLS]->(ownerScope)
MERGE (create)-[:USES_TYPE]->(createResult),(create)-[:READS]->(challenges:Table {name:"oauth_device_enrollment_challenges",type:"table"}),(create)-[:WRITES]->(challenges),(create)-[:USES_TYPE]->(challengeCap)
MERGE (findChallenge)-[:READS]->(challenges)
MERGE (complete)-[:WRITES]->(challenges),(complete)-[:WRITES]->(bindings:Table {name:"oauth_device_bindings",type:"table"}),(complete)-[:WRITES]->(audit:Table {name:"oauth_device_binding_audit",type:"table"})
MERGE (active)-[:READS]->(bindings)
MERGE (ownerScope)-[:CONFIGURES]->(oauthOwnerRole),(ownerScope)-[:CALLS]->(scope)
MERGE (revoke)-[:WRITES]->(bindings),(revoke)-[:WRITES]->(audit),(revoke)-[:WRITES]->(refresh:Table {name:"oauth_refresh_tokens",type:"table"})
MERGE (self)-[:DEFINES]->(scope)
@endcypher
*/
//! PostgreSQL-owned device enrollment challenges and generation-bound device identities.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::PgAdapterError;

/// Hard per-actor cap for short-lived active device-enrollment challenges.
pub const MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES: i64 = 3;

#[derive(Debug, Clone, sqlx::FromRow)]
/// A single-use, actor-scoped device enrollment challenge.
pub struct DeviceEnrollmentChallenge {
    /// Server-issued challenge identifier.
    pub challenge_id: Uuid,
    /// Owning tenant identifier.
    pub tenant_id: Uuid,
    /// Authenticated actor that requested the challenge.
    pub actor_id: Uuid,
    /// SHA-256 digest of the nonce; raw nonce material is not persisted.
    pub challenge_hash: Vec<u8>,
    /// Ed25519 public credential to be bound after proof verification.
    pub public_key: Vec<u8>,
    /// Lowercase SHA-256 fingerprint of the public credential.
    pub public_key_fingerprint: String,
    /// Challenge expiry timestamp.
    pub expires_at: DateTime<Utc>,
}

/// Result of creating a challenge under the owner-scoped active-challenge cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChallengeCreateResult {
    /// A new challenge was persisted for the actor.
    Created,
    /// The actor already has the maximum number of active challenges.
    RateLimited,
}

#[derive(Debug, Clone, sqlx::FromRow)]
/// Persisted device identity and its current revocation generation.
pub struct DeviceBinding {
    /// Owning tenant identifier.
    pub tenant_id: Uuid,
    /// Authenticated owner of the device.
    pub actor_id: Uuid,
    /// Opaque server-issued device identifier.
    pub device_id: Uuid,
    /// Fingerprint of the verified public credential.
    pub public_key_fingerprint: String,
    /// Monotonic generation; a revocation advances this value.
    pub generation: i64,
    /// Current binding state (`active` or `revoked`).
    pub status: String,
    /// Binding creation time.
    pub created_at: DateTime<Utc>,
    /// Revocation time, if the binding is revoked.
    pub revoked_at: Option<DateTime<Utc>>,
}

#[async_trait]
/// Owner-scoped persistence contract for device enrollment and revocation.
pub trait DeviceBindingRepository: Send + Sync {
    /// Persist a short-lived challenge under the actor and tenant RLS scope.
    async fn create_challenge(
        &self,
        challenge: &DeviceEnrollmentChallenge,
    ) -> Result<DeviceChallengeCreateResult, PgAdapterError>;

    /// Read an unconsumed challenge belonging to exactly this actor and tenant.
    async fn find_challenge(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        challenge_id: Uuid,
    ) -> Result<Option<DeviceEnrollmentChallenge>, PgAdapterError>;

    /// Consume a challenge and create its device binding and audit fact atomically.
    async fn complete_challenge(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        challenge_id: Uuid,
        challenge_hash: &[u8],
        device_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError>;

    /// Confirm that the requested generation is the current active generation.
    async fn is_active(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        generation: i64,
    ) -> Result<bool, PgAdapterError>;

    /// Revoke an active device, advance its generation, and revoke its refresh family.
    async fn revoke(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError>;
}

/// PostgreSQL implementation of the device binding persistence contract.
pub struct PgDeviceBindingRepository {
    pool: PgPool,
}

impl PgDeviceBindingRepository {
    /// Construct a repository that uses the supplied shared pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceBindingRepository for PgDeviceBindingRepository {
    async fn create_challenge(
        &self,
        challenge: &DeviceEnrollmentChallenge,
    ) -> Result<DeviceChallengeCreateResult, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(query_error("begin challenge"))?;
        set_oauth_owner_scope(&mut tx, challenge.tenant_id, challenge.actor_id).await?;

        let owner_lock = format!("{}:{}", challenge.tenant_id, challenge.actor_id);
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(owner_lock)
            .execute(&mut *tx)
            .await
            .map_err(query_error("lock device challenge owner"))?;

        let now = Utc::now();
        sqlx::query(
            "DELETE FROM oauth_device_enrollment_challenges \
             WHERE tenant_id=$1 AND actor_id=$2 \
               AND (expires_at <= $3 OR consumed_at IS NOT NULL)",
        )
        .bind(challenge.tenant_id)
        .bind(challenge.actor_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(query_error("prune owner device challenges"))?;

        let active_challenges = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM oauth_device_enrollment_challenges \
             WHERE tenant_id=$1 AND actor_id=$2 AND consumed_at IS NULL AND expires_at > $3",
        )
        .bind(challenge.tenant_id)
        .bind(challenge.actor_id)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(query_error("count owner device challenges"))?;
        if active_challenges >= MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES {
            tx.commit()
                .await
                .map_err(query_error("commit limited challenge cleanup"))?;
            return Ok(DeviceChallengeCreateResult::RateLimited);
        }

        sqlx::query(
            "INSERT INTO oauth_device_enrollment_challenges \
             (challenge_id, tenant_id, actor_id, challenge_hash, public_key, public_key_fingerprint, expires_at) \
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(challenge.challenge_id)
        .bind(challenge.tenant_id)
        .bind(challenge.actor_id)
        .bind(&challenge.challenge_hash)
        .bind(&challenge.public_key)
        .bind(&challenge.public_key_fingerprint)
        .bind(challenge.expires_at)
        .execute(&mut *tx)
        .await
        .map_err(query_error("insert enrollment challenge"))?;
        tx.commit().await.map_err(query_error("commit challenge"))?;
        Ok(DeviceChallengeCreateResult::Created)
    }

    async fn find_challenge(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        challenge_id: Uuid,
    ) -> Result<Option<DeviceEnrollmentChallenge>, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(query_error("begin challenge lookup"))?;
        set_oauth_owner_scope(&mut tx, tenant_id, actor_id).await?;
        let challenge = sqlx::query_as::<_, DeviceEnrollmentChallenge>(
            "SELECT challenge_id, tenant_id, actor_id, challenge_hash, public_key, \
             public_key_fingerprint, expires_at FROM oauth_device_enrollment_challenges \
             WHERE tenant_id=$1 AND actor_id=$2 AND challenge_id=$3 \
             AND consumed_at IS NULL AND expires_at > now()",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(challenge_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(query_error("find enrollment challenge"))?;
        tx.commit()
            .await
            .map_err(query_error("commit challenge lookup"))?;
        Ok(challenge)
    }

    async fn complete_challenge(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        challenge_id: Uuid,
        challenge_hash: &[u8],
        device_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(query_error("begin enrollment"))?;
        set_oauth_owner_scope(&mut tx, tenant_id, actor_id).await?;
        let challenge = sqlx::query_as::<_, DeviceEnrollmentChallenge>(
            "UPDATE oauth_device_enrollment_challenges SET consumed_at=$5 \
             WHERE tenant_id=$1 AND actor_id=$2 AND challenge_id=$3 \
             AND challenge_hash=$4 AND consumed_at IS NULL AND expires_at>$5 \
             RETURNING challenge_id, tenant_id, actor_id, challenge_hash, public_key, \
             public_key_fingerprint, expires_at",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(challenge_id)
        .bind(challenge_hash)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(query_error("consume enrollment challenge"))?;
        let Some(challenge) = challenge else {
            tx.commit()
                .await
                .map_err(query_error("commit rejected enrollment"))?;
            return Ok(None);
        };

        let binding = sqlx::query_as::<_, DeviceBinding>(
            "INSERT INTO oauth_device_bindings \
             (tenant_id, actor_id, device_id, public_key_fingerprint, generation, status, created_at) \
             VALUES ($1,$2,$3,$4,1,'active',$5) \
             RETURNING tenant_id, actor_id, device_id, public_key_fingerprint, generation, \
             status, created_at, revoked_at",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(&challenge.public_key_fingerprint)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(query_error("insert device binding"))?;
        insert_audit(&mut tx, &binding, "enrolled", now).await?;
        tx.commit()
            .await
            .map_err(query_error("commit enrollment"))?;
        Ok(Some(binding))
    }

    async fn is_active(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        generation: i64,
    ) -> Result<bool, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(query_error("begin device check"))?;
        set_oauth_owner_scope(&mut tx, tenant_id, actor_id).await?;
        let active = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM oauth_device_bindings \
             WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3 \
               AND generation=$4 AND status='active' AND revoked_at IS NULL)",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(generation)
        .fetch_one(&mut *tx)
        .await
        .map_err(query_error("check active device"))?;
        tx.commit()
            .await
            .map_err(query_error("commit device check"))?;
        Ok(active)
    }

    async fn revoke(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(query_error("begin device revoke"))?;
        set_oauth_owner_scope(&mut tx, tenant_id, actor_id).await?;
        let binding = sqlx::query_as::<_, DeviceBinding>(
            "UPDATE oauth_device_bindings SET generation=generation+1, status='revoked', revoked_at=$4 \
             WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3 \
               AND status='active' AND revoked_at IS NULL \
             RETURNING tenant_id, actor_id, device_id, public_key_fingerprint, generation, \
             status, created_at, revoked_at",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(query_error("revoke device"))?;
        let Some(binding) = binding else {
            tx.commit()
                .await
                .map_err(query_error("commit missing device revoke"))?;
            return Ok(None);
        };

        sqlx::query(
            "UPDATE oauth_refresh_tokens SET revoked_at=$4 \
             WHERE tenant_id=$1 AND user_id=$2 AND device_id=$3 AND revoked_at IS NULL",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(query_error("revoke device refresh family"))?;
        insert_audit(&mut tx, &binding, "revoked", now).await?;
        tx.commit()
            .await
            .map_err(query_error("commit device revoke"))?;
        Ok(Some(binding))
    }
}

async fn set_oauth_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
) -> Result<(), PgAdapterError> {
    // The deployment login is NOINHERIT and receives this dedicated role only
    // through deployment membership; never grant OAuth/device table ACLs to it.
    sqlx::query("SET LOCAL ROLE star_oauth_credential_owner")
        .execute(&mut **tx)
        .await
        .map_err(query_error("set OAuth credential owner role"))?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(actor_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(query_error("set OAuth device owner scope"))?;
    Ok(())
}

async fn insert_audit(
    tx: &mut Transaction<'_, Postgres>,
    binding: &DeviceBinding,
    event_kind: &str,
    occurred_at: DateTime<Utc>,
) -> Result<(), PgAdapterError> {
    sqlx::query(
        "INSERT INTO oauth_device_binding_audit \
         (tenant_id, actor_id, device_id, event_kind, generation, public_key_fingerprint, occurred_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(binding.tenant_id)
    .bind(binding.actor_id)
    .bind(binding.device_id)
    .bind(event_kind)
    .bind(binding.generation)
    .bind(&binding.public_key_fingerprint)
    .bind(occurred_at)
    .execute(&mut **tx)
    .await
    .map_err(query_error("insert device audit event"))?;
    Ok(())
}

fn query_error(operation: &'static str) -> impl FnOnce(sqlx::Error) -> PgAdapterError {
    move |error| PgAdapterError::Query(format!("{operation}: {error}"))
}
