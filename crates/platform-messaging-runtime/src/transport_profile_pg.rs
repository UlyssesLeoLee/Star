// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=f2a260336e9eb576c4fb6418b8442aec81a8fb6b36d6713aafa38e7ec5917c8e
MERGE (self:File {path:"crates/platform-messaging-runtime/src/transport_profile_pg.rs"})
MERGE (owner:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster",kind:"struct"})
MERGE (resolver:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileResolver",kind:"struct"})
MERGE (command:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::ProfileChangeRequest",kind:"struct"})
MERGE (error:Type {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::TransportProfileStoreError",kind:"enum"})
MERGE (new_owner:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster.new",kind:"method"})
MERGE (publish:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileMaster.publish_revision",kind:"method"})
MERGE (new_resolver:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileResolver.new",kind:"method"})
MERGE (resolve:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::PgTransportProfileResolver.resolve",kind:"method"})
MERGE (read_row:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::decode_profile_row",kind:"function"})
MERGE (set_owner:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::set_owner_scope",kind:"function"})
MERGE (set_resolver:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::set_resolver_scope",kind:"function"})
MERGE (same_request:Symbol {id:"crates/platform-messaging-runtime/src/transport_profile_pg.rs::same_change_request",kind:"function"})
MERGE (master:Table {id:"platform_messaging.transport_profile_revision",class:"Master"})
MERGE (audit:Table {id:"platform_messaging.transport_profile_revision_audit",class:"Transaction"})
MERGE (outbox:Table {id:"platform_messaging.transport_profile_revision_outbox",class:"Transaction"})
MERGE (self)-[:DEFINES]->(owner)
MERGE (self)-[:DEFINES]->(resolver)
MERGE (self)-[:DEFINES]->(command)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(new_owner)
MERGE (self)-[:DEFINES]->(publish)
MERGE (self)-[:DEFINES]->(new_resolver)
MERGE (self)-[:DEFINES]->(resolve)
MERGE (self)-[:DEFINES]->(read_row)
MERGE (self)-[:DEFINES]->(set_owner)
MERGE (self)-[:DEFINES]->(set_resolver)
MERGE (self)-[:DEFINES]->(same_request)
MERGE (publish)-[:CALLS]->(set_owner)
MERGE (publish)-[:CALLS]->(read_row)
MERGE (publish)-[:CALLS]->(same_request)
MERGE (publish)-[:READS]->(master)
MERGE (publish)-[:WRITES]->(master)
MERGE (publish)-[:WRITES]->(audit)
MERGE (publish)-[:WRITES]->(outbox)
MERGE (resolve)-[:CALLS]->(set_resolver)
MERGE (resolve)-[:CALLS]->(read_row)
MERGE (resolve)-[:READS]->(master)
MERGE (resolve)-[:USES_TYPE]->(context:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::AuthenticatedTransportContext",kind:"struct"})
MERGE (resolve)-[:USES_TYPE]->(session:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::AuthenticatedTransportSession",kind:"trait"})
MERGE (read_row)-[:USES_TYPE]->(profile:Type {id:"crates/platform-messaging-runtime/src/transport_profile.rs::TransportProfileRevision",kind:"struct"})
@endcypher
*/
//! Owner-scoped PostgreSQL publication and bounded transport profile resolution.

use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

use crate::transport_profile::{
    AuthenticatedTransportContext, AuthenticatedTransportSession, ProfileAuthorizationError,
    ProfileDefinitionError, TransportMessageKind, TransportProfileDefinition,
    TransportProfileLifecycle, TransportProfileRevision,
};

/// One immutable profile revision publication request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileChangeRequest {
    /// Stable identity of the profile being revised.
    pub profile_id: Uuid,
    /// Current revision expected by compare-and-swap; absent only for the first revision.
    pub expected_revision: Option<i64>,
    /// New lifecycle state.
    pub lifecycle: TransportProfileLifecycle,
    /// New bounded provider configuration.
    pub definition: TransportProfileDefinition,
    /// Stable operation identity used for exact replay.
    pub request_id: Uuid,
    /// Distributed trace identity.
    pub trace_id: Uuid,
    /// Optional event that caused this publication.
    pub causation_id: Option<Uuid>,
    /// Bounded, non-secret idempotency identity.
    pub idempotency_key: String,
}

impl ProfileChangeRequest {
    /// Construct and validate an idempotent profile change command.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile_id: Uuid,
        expected_revision: Option<i64>,
        lifecycle: TransportProfileLifecycle,
        definition: TransportProfileDefinition,
        request_id: Uuid,
        trace_id: Uuid,
        causation_id: Option<Uuid>,
        idempotency_key: String,
    ) -> Result<Self, TransportProfileStoreError> {
        let request = Self {
            profile_id,
            expected_revision,
            lifecycle,
            definition,
            request_id,
            trace_id,
            causation_id,
            idempotency_key,
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> Result<(), TransportProfileStoreError> {
        if self.profile_id.is_nil()
            || self.expected_revision.is_some_and(|revision| revision <= 0)
            || (self.expected_revision.is_none()
                && self.lifecycle != TransportProfileLifecycle::Active)
            || self.request_id.is_nil()
            || self.trace_id.is_nil()
            || self.causation_id.is_some_and(|id| id.is_nil())
            || !valid_idempotency_key(&self.idempotency_key)
        {
            return Err(TransportProfileStoreError::InvalidIdentity);
        }
        Ok(())
    }
}

/// Profile publication, compare-and-swap, or persistence failure.
#[derive(Debug, Error)]
pub enum TransportProfileStoreError {
    /// Deployment, tenant, actor or request identity is invalid.
    #[error("invalid transport profile identity")]
    InvalidIdentity,
    /// The profile revision no longer matches the expected revision.
    #[error("transport profile revision compare-and-swap conflict")]
    CompareAndSwapConflict,
    /// An idempotency key was replayed with different immutable command content.
    #[error("transport profile idempotency key conflicts with a prior command")]
    IdempotencyConflict,
    /// A stored profile row violates its schema or digest.
    #[error("transport profile integrity validation failed")]
    Integrity,
    /// No current profile exists in the trusted deployment and tenant scope.
    #[error("transport profile was not found")]
    NotFound,
    /// The resolved profile is disabled or revoked.
    #[error("transport profile is inactive")]
    Inactive,
    /// The authenticated provider identity or route is outside the profile.
    #[error(transparent)]
    Unauthorized(#[from] ProfileAuthorizationError),
    /// A profile value failed typed validation.
    #[error(transparent)]
    InvalidProfile(#[from] ProfileDefinitionError),
    /// PostgreSQL operation failed or its commit outcome is unknown.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// Platform Messaging owner writer bound to one deployment and service actor.
#[derive(Clone)]
pub struct PgTransportProfileMaster {
    pool: PgPool,
    deployment_id: Uuid,
    actor_service_id: Uuid,
}

impl PgTransportProfileMaster {
    /// Construct the owner adapter from trusted deployment composition.
    pub fn new(
        pool: PgPool,
        deployment_id: Uuid,
        actor_service_id: Uuid,
    ) -> Result<Self, TransportProfileStoreError> {
        if deployment_id.is_nil() || actor_service_id.is_nil() {
            return Err(TransportProfileStoreError::InvalidIdentity);
        }
        Ok(Self {
            pool,
            deployment_id,
            actor_service_id,
        })
    }

    /// Atomically publish a successor revision, audit fact and BI/Benchmark Outbox fact.
    ///
    /// Exact replay with the same idempotency key returns the original immutable revision.
    /// Unknown commit outcomes remain errors; callers recover by replaying the same request.
    pub async fn publish_revision(
        &self,
        tenant_id: Uuid,
        command: &ProfileChangeRequest,
    ) -> Result<TransportProfileRevision, TransportProfileStoreError> {
        if tenant_id.is_nil() {
            return Err(TransportProfileStoreError::InvalidIdentity);
        }
        command.validate()?;
        let mut tx = self.pool.begin().await?;
        set_owner_scope(
            &mut tx,
            self.deployment_id,
            tenant_id,
            self.actor_service_id,
        )
        .await?;

        if let Some(row) = sqlx::query(
            "SELECT * FROM platform_messaging.transport_profile_revision \
             WHERE deployment_id=$1 AND tenant_id=$2 AND idempotency_key=$3",
        )
        .bind(self.deployment_id)
        .bind(tenant_id)
        .bind(&command.idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            let (revision, metadata) = decode_profile_row(&row)?;
            if !same_change_request(
                self.actor_service_id,
                tenant_id,
                command,
                &revision,
                &metadata,
            ) {
                return Err(TransportProfileStoreError::IdempotencyConflict);
            }
            tx.commit().await?;
            return Ok(revision);
        }

        let current_revision: Option<i64> = sqlx::query_scalar(
            "SELECT revision FROM platform_messaging.transport_profile_revision \
             WHERE deployment_id=$1 AND tenant_id=$2 AND profile_id=$3 AND valid_to IS NULL \
             FOR UPDATE",
        )
        .bind(self.deployment_id)
        .bind(tenant_id)
        .bind(command.profile_id)
        .fetch_optional(&mut *tx)
        .await?;
        if current_revision != command.expected_revision {
            return Err(TransportProfileStoreError::CompareAndSwapConflict);
        }
        if current_revision.is_none() && command.lifecycle != TransportProfileLifecycle::Active {
            return Err(TransportProfileStoreError::InvalidIdentity);
        }
        let revision_number = current_revision
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(TransportProfileStoreError::InvalidIdentity)?;
        let revision = TransportProfileRevision::new(
            self.deployment_id,
            tenant_id,
            command.profile_id,
            revision_number,
            command.lifecycle,
            command.definition.clone(),
        )?;

        if let Some(expected_revision) = current_revision {
            let updated = sqlx::query(
                "UPDATE platform_messaging.transport_profile_revision \
                 SET valid_to=transaction_timestamp() \
                 WHERE deployment_id=$1 AND tenant_id=$2 AND profile_id=$3 \
                   AND revision=$4 AND valid_to IS NULL",
            )
            .bind(self.deployment_id)
            .bind(tenant_id)
            .bind(command.profile_id)
            .bind(expected_revision)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() != 1 {
                return Err(TransportProfileStoreError::CompareAndSwapConflict);
            }
        }

        let sender_ids = command.definition.allowed_sender_service_ids().to_vec();
        let recipient_ids = command.definition.allowed_recipient_service_ids().to_vec();
        let message_kinds: Vec<String> = command
            .definition
            .allowed_message_kinds()
            .iter()
            .map(|kind| kind.as_str().to_owned())
            .collect();
        let secret_reference = command
            .definition
            .secret_reference()
            .map(|reference| reference.as_str().to_owned());
        let inserted = sqlx::query(
            "INSERT INTO platform_messaging.transport_profile_revision (\
                deployment_id,tenant_id,profile_id,revision,expected_previous_revision,\
                lifecycle_state,schema_version,provider_id,provider_version,endpoint_identity,\
                log_generation_policy,allowed_sender_service_ids,allowed_recipient_service_ids,\
                allowed_message_kinds,secret_reference,profile_digest,actor_service_id,\
                request_id,trace_id,causation_id,idempotency_key\
             ) VALUES (\
                $1,$2,$3,$4,$5,$6,1,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20\
             ) RETURNING *",
        )
        .bind(self.deployment_id)
        .bind(tenant_id)
        .bind(command.profile_id)
        .bind(revision_number)
        .bind(command.expected_revision)
        .bind(command.lifecycle.as_str())
        .bind(command.definition.provider_id())
        .bind(command.definition.provider_version())
        .bind(command.definition.endpoint_identity())
        .bind(command.definition.log_generation_policy())
        .bind(sender_ids)
        .bind(recipient_ids)
        .bind(message_kinds)
        .bind(secret_reference)
        .bind(revision.digest().as_slice())
        .bind(self.actor_service_id)
        .bind(command.request_id)
        .bind(command.trace_id)
        .bind(command.causation_id)
        .bind(&command.idempotency_key)
        .fetch_one(&mut *tx)
        .await?;
        let (persisted, _) = decode_profile_row(&inserted)?;
        if persisted != revision {
            return Err(TransportProfileStoreError::Integrity);
        }
        tx.commit().await?;
        Ok(revision)
    }
}

/// Bounded, uncached reader for the one active profile selected by trusted composition.
#[derive(Clone)]
pub struct PgTransportProfileResolver {
    pool: PgPool,
}

impl PgTransportProfileResolver {
    /// Construct the profile resolver from the deployment-managed pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Resolve one active profile against authenticated session identity and route scope.
    ///
    /// The profile ID and message kind must come from trusted application routing, never a
    /// message body. The session implementation must represent an already authenticated channel.
    pub async fn resolve<S: AuthenticatedTransportSession>(
        &self,
        session: &S,
        profile_id: Uuid,
        kind: TransportMessageKind,
    ) -> Result<TransportProfileRevision, TransportProfileStoreError> {
        if profile_id.is_nil() {
            return Err(TransportProfileStoreError::InvalidIdentity);
        }
        let context = AuthenticatedTransportContext::from_session(session)?;
        let mut tx = self.pool.begin().await?;
        set_resolver_scope(&mut tx, context.deployment_id(), context.tenant_id()).await?;
        let row = sqlx::query(
            "SELECT * FROM platform_messaging.transport_profile_revision \
             WHERE deployment_id=$1 AND tenant_id=$2 AND profile_id=$3 AND valid_to IS NULL",
        )
        .bind(context.deployment_id())
        .bind(context.tenant_id())
        .bind(profile_id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        let Some(row) = row else {
            return Err(TransportProfileStoreError::NotFound);
        };
        let (revision, _) = decode_profile_row(&row)?;
        if revision.lifecycle() != TransportProfileLifecycle::Active {
            return Err(TransportProfileStoreError::Inactive);
        }
        revision.authorize(&context, kind)?;
        Ok(revision)
    }
}

struct RevisionMetadata {
    expected_previous_revision: Option<i64>,
    actor_service_id: Uuid,
    request_id: Uuid,
    trace_id: Uuid,
    causation_id: Option<Uuid>,
    idempotency_key: String,
}

fn decode_profile_row(
    row: &sqlx::postgres::PgRow,
) -> Result<(TransportProfileRevision, RevisionMetadata), TransportProfileStoreError> {
    let provider_id: String = row.try_get("provider_id")?;
    let provider_version: String = row.try_get("provider_version")?;
    let endpoint_identity: String = row.try_get("endpoint_identity")?;
    let log_generation_policy: String = row.try_get("log_generation_policy")?;
    let raw_sender_ids: Vec<Uuid> = row.try_get("allowed_sender_service_ids")?;
    let raw_recipient_ids: Vec<Uuid> = row.try_get("allowed_recipient_service_ids")?;
    let raw_message_kinds: Vec<String> = row.try_get("allowed_message_kinds")?;
    let message_kinds = raw_message_kinds
        .iter()
        .map(|kind| {
            crate::transport_profile::TransportMessageKind::from_database(kind)
                .ok_or(TransportProfileStoreError::Integrity)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let secret_reference: Option<String> = row.try_get("secret_reference")?;
    let definition = TransportProfileDefinition::new(
        provider_id,
        provider_version,
        endpoint_identity,
        log_generation_policy,
        raw_sender_ids.clone(),
        raw_recipient_ids.clone(),
        message_kinds,
        secret_reference
            .map(crate::transport_profile::OpaqueSecretReference::new)
            .transpose()?,
    )?;
    if definition.allowed_sender_service_ids() != raw_sender_ids.as_slice()
        || definition.allowed_recipient_service_ids() != raw_recipient_ids.as_slice()
        || definition
            .allowed_message_kinds()
            .iter()
            .map(|kind| kind.as_str())
            .ne(raw_message_kinds.iter().map(String::as_str))
        || row.try_get::<i16, _>("schema_version")? != 1
    {
        return Err(TransportProfileStoreError::Integrity);
    }
    let lifecycle_value: String = row.try_get("lifecycle_state")?;
    let lifecycle = TransportProfileLifecycle::from_database(&lifecycle_value)
        .ok_or(TransportProfileStoreError::Integrity)?;
    let revision = TransportProfileRevision::new(
        row.try_get("deployment_id")?,
        row.try_get("tenant_id")?,
        row.try_get("profile_id")?,
        row.try_get("revision")?,
        lifecycle,
        definition,
    )?;
    let stored_digest: Vec<u8> = row.try_get("profile_digest")?;
    if stored_digest.as_slice() != revision.digest() {
        return Err(TransportProfileStoreError::Integrity);
    }
    let metadata = RevisionMetadata {
        expected_previous_revision: row.try_get("expected_previous_revision")?,
        actor_service_id: row.try_get("actor_service_id")?,
        request_id: row.try_get("request_id")?,
        trace_id: row.try_get("trace_id")?,
        causation_id: row.try_get("causation_id")?,
        idempotency_key: row.try_get("idempotency_key")?,
    };
    Ok((revision, metadata))
}

async fn set_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
    tenant_id: Uuid,
    actor_service_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL ROLE star_platform_profile_owner")
        .execute(&mut **tx)
        .await?;
    set_common_scope(tx, deployment_id, tenant_id).await?;
    sqlx::query("SELECT set_config('app.platform_actor_service_id',$1,true)")
        .bind(actor_service_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn set_resolver_scope(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL ROLE star_platform_profile_resolver")
        .execute(&mut **tx)
        .await?;
    set_common_scope(tx, deployment_id, tenant_id).await
}

async fn set_common_scope(
    tx: &mut Transaction<'_, Postgres>,
    deployment_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL lock_timeout='5s'")
        .execute(&mut **tx)
        .await?;
    sqlx::query("SET LOCAL statement_timeout='10s'")
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.platform_deployment_id',$1,true)")
        .bind(deployment_id.to_string())
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.platform_profile_tenant_id',$1,true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn same_change_request(
    actor_service_id: Uuid,
    tenant_id: Uuid,
    command: &ProfileChangeRequest,
    revision: &TransportProfileRevision,
    metadata: &RevisionMetadata,
) -> bool {
    revision.tenant_id() == tenant_id
        && revision.profile_id() == command.profile_id
        && revision.lifecycle() == command.lifecycle
        && revision.definition() == &command.definition
        && metadata.expected_previous_revision == command.expected_revision
        && metadata.actor_service_id == actor_service_id
        && metadata.request_id == command.request_id
        && metadata.trace_id == command.trace_id
        && metadata.causation_id == command.causation_id
        && metadata.idempotency_key == command.idempotency_key
}

fn valid_idempotency_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_key_is_bounded_and_secret_free() {
        assert!(valid_idempotency_key("profile-revision:1"));
        assert!(!valid_idempotency_key(""));
        assert!(!valid_idempotency_key(":profile-revision"));
        assert!(!valid_idempotency_key("secret/value"));
        assert!(!valid_idempotency_key(&"a".repeat(129)));
    }

    #[test]
    fn profile_change_requires_a_positive_cas_revision() {
        let definition = TransportProfileDefinition::new(
            "star-rust-log".to_owned(),
            "1.0.0".to_owned(),
            "run-admission".to_owned(),
            "provider-log-generation-v1".to_owned(),
            vec![Uuid::from_u128(1)],
            vec![Uuid::from_u128(2)],
            vec![TransportMessageKind::Request],
            None,
        )
        .unwrap();
        assert!(
            ProfileChangeRequest::new(
                Uuid::from_u128(3),
                Some(0),
                TransportProfileLifecycle::Active,
                definition.clone(),
                Uuid::from_u128(4),
                Uuid::from_u128(5),
                None,
                "profile-revision:1".to_owned(),
            )
            .is_err()
        );
        assert!(
            ProfileChangeRequest::new(
                Uuid::from_u128(3),
                None,
                TransportProfileLifecycle::Active,
                definition,
                Uuid::from_u128(4),
                Uuid::from_u128(5),
                None,
                "profile-revision:1".to_owned(),
            )
            .is_ok()
        );
    }
}
