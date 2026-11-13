// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=2a3ad45850a2a99bdeb3795b35e132fbf95826fbae577d5164422b391ab0bfd5
MERGE (self:File {path:"crates/star-pg-adapter/src/repository/scm_repository_master.rs"})
MERGE (repo:Type {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository",kind:"struct"})
MERGE (port:Type {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::ScmRepositoryMasterRepository",kind:"trait"})
MERGE (error:Type {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::ScmRepositoryMasterError",kind:"enum"})
MERGE (new:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::new",kind:"method"})
MERGE (find:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::find_current",kind:"method"})
MERGE (create:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::create",kind:"method"})
MERGE (revise:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::PgScmRepositoryMasterRepository::revise",kind:"method"})
MERGE (scope:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::set_repository_owner_scope",kind:"function"})
MERGE (writeScope:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::set_repository_write_scope",kind:"function"})
MERGE (setLocal:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::set_local_value",kind:"function"})
MERGE (lock:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::lock_repository_identity",kind:"function"})
MERGE (insert:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::insert_revision",kind:"function"})
MERGE (validate:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::validate_write",kind:"function"})
MERGE (decode:Symbol {id:"crates/star-pg-adapter/src/repository/scm_repository_master.rs::decode_revision",kind:"function"})
MERGE (master:Table {id:"scm.repository_master"})
MERGE (pool:ExternalService {id:"sqlx.PgPool",kind:"type"})
MERGE (transaction:ExternalService {id:"sqlx.Transaction",kind:"type"})
MERGE (row:ExternalService {id:"sqlx.PgRow",kind:"type"})
MERGE (advisoryLock:ExternalService {id:"postgres.pg_advisory_xact_lock",kind:"database_function"})
MERGE (settings:Config {id:"PostgreSQL transaction-local app.tenant_id/app.actor_id/app.correlation_id",kind:"database_settings"})
MERGE (domain:File {path:"crates/domain-scm/src/lib.rs"})
MERGE (metadata:Type {id:"crates/domain-scm/src/lib.rs::RepositoryMasterMetadata",kind:"struct"})
MERGE (revision:Type {id:"crates/domain-scm/src/lib.rs::RepositoryMasterRevision",kind:"struct"})
MERGE (repositoryId:Type {id:"crates/domain-scm/src/lib.rs::RepositoryId",kind:"uuid_id"})
MERGE (tenantId:Type {id:"crates/domain-scm/src/lib.rs::TenantId",kind:"uuid_id"})
MERGE (userId:Type {id:"crates/domain-scm/src/lib.rs::UserId",kind:"uuid_id"})
MERGE (uuid:ExternalService {id:"uuid::Uuid",kind:"type"})
MERGE (metadataValidate:Symbol {id:"crates/domain-scm/src/lib.rs::RepositoryMasterMetadata.validate",kind:"method"})
MERGE (self)-[:DEFINES]->(repo)
MERGE (self)-[:DEFINES]->(port)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:DEFINES]->(new)
MERGE (self)-[:DEFINES]->(find)
MERGE (self)-[:DEFINES]->(create)
MERGE (self)-[:DEFINES]->(revise)
MERGE (self)-[:DEFINES]->(scope)
MERGE (self)-[:DEFINES]->(writeScope)
MERGE (self)-[:DEFINES]->(setLocal)
MERGE (self)-[:DEFINES]->(lock)
MERGE (self)-[:DEFINES]->(insert)
MERGE (self)-[:DEFINES]->(validate)
MERGE (self)-[:DEFINES]->(decode)
MERGE (repo)-[:IMPLEMENTS]->(port)
MERGE (repo)-[:USES_TYPE]->(pool)
MERGE (repo)-[:DEFINES]->(new)
MERGE (repo)-[:DEFINES]->(find)
MERGE (repo)-[:DEFINES]->(create)
MERGE (repo)-[:DEFINES]->(revise)
MERGE (self)-[:IMPORTS]->(domain)
MERGE (new)-[:USES_TYPE]->(pool)
MERGE (find)-[:USES_TYPE]->(error)
MERGE (create)-[:USES_TYPE]->(error)
MERGE (revise)-[:USES_TYPE]->(error)
MERGE (find)-[:USES_TYPE]->(tenantId)
MERGE (find)-[:USES_TYPE]->(repositoryId)
MERGE (find)-[:USES_TYPE]->(revision)
MERGE (create)-[:USES_TYPE]->(tenantId)
MERGE (create)-[:USES_TYPE]->(repositoryId)
MERGE (create)-[:USES_TYPE]->(metadata)
MERGE (create)-[:USES_TYPE]->(userId)
MERGE (create)-[:USES_TYPE]->(uuid)
MERGE (create)-[:USES_TYPE]->(revision)
MERGE (revise)-[:USES_TYPE]->(tenantId)
MERGE (revise)-[:USES_TYPE]->(repositoryId)
MERGE (revise)-[:USES_TYPE]->(metadata)
MERGE (revise)-[:USES_TYPE]->(userId)
MERGE (revise)-[:USES_TYPE]->(uuid)
MERGE (revise)-[:USES_TYPE]->(revision)
MERGE (find)-[:USES_TYPE]->(pool)
MERGE (create)-[:USES_TYPE]->(pool)
MERGE (revise)-[:USES_TYPE]->(pool)
MERGE (find)-[:USES_TYPE]->(transaction)
MERGE (create)-[:USES_TYPE]->(transaction)
MERGE (revise)-[:USES_TYPE]->(transaction)
MERGE (find)-[:CALLS]->(scope)
MERGE (find)-[:CALLS]->(decode)
MERGE (find)-[:READS]->(master)
MERGE (create)-[:CALLS]->(writeScope)
MERGE (create)-[:CALLS]->(lock)
MERGE (create)-[:CALLS]->(validate)
MERGE (create)-[:CALLS]->(insert)
MERGE (create)-[:CALLS]->(decode)
MERGE (create)-[:READS]->(master)
MERGE (create)-[:WRITES]->(master)
MERGE (revise)-[:CALLS]->(writeScope)
MERGE (revise)-[:CALLS]->(lock)
MERGE (revise)-[:CALLS]->(validate)
MERGE (revise)-[:CALLS]->(insert)
MERGE (revise)-[:CALLS]->(decode)
MERGE (revise)-[:READS]->(master)
MERGE (revise)-[:WRITES]->(master)
MERGE (writeScope)-[:CALLS]->(scope)
MERGE (writeScope)-[:CALLS]->(setLocal)
MERGE (scope)-[:CALLS]->(setLocal)
MERGE (scope)-[:USES_TYPE]->(transaction)
MERGE (writeScope)-[:USES_TYPE]->(transaction)
MERGE (setLocal)-[:USES_TYPE]->(transaction)
MERGE (setLocal)-[:CONFIGURES]->(settings)
MERGE (lock)-[:USES_TYPE]->(transaction)
MERGE (lock)-[:CALLS]->(advisoryLock)
MERGE (insert)-[:USES_TYPE]->(transaction)
MERGE (insert)-[:USES_TYPE]->(metadata)
MERGE (insert)-[:USES_TYPE]->(tenantId)
MERGE (insert)-[:USES_TYPE]->(repositoryId)
MERGE (insert)-[:USES_TYPE]->(userId)
MERGE (insert)-[:USES_TYPE]->(uuid)
MERGE (insert)-[:WRITES]->(master)
MERGE (validate)-[:USES_TYPE]->(metadata)
MERGE (validate)-[:USES_TYPE]->(tenantId)
MERGE (validate)-[:USES_TYPE]->(repositoryId)
MERGE (validate)-[:USES_TYPE]->(userId)
MERGE (validate)-[:USES_TYPE]->(uuid)
MERGE (validate)-[:CALLS]->(metadataValidate)
MERGE (decode)-[:USES_TYPE]->(row)
MERGE (decode)-[:USES_TYPE]->(metadata)
MERGE (decode)-[:USES_TYPE]->(revision)
MERGE (decode)-[:CALLS]->(metadataValidate)
MERGE (master)-[:USES_TYPE]->(metadata)
@endcypher
*/
//! SCM-owned Repository master persistence. Project/Run authorization is owned elsewhere.

use chrono::{DateTime, Utc};
use domain_scm::{
    RepositoryId, RepositoryMasterMetadata, RepositoryMasterRevision, TenantId, UserId,
};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

/// Stable outcomes that do not disclose database row contents or driver messages.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ScmRepositoryMasterError {
    /// Required identifiers or metadata are invalid.
    #[error("invalid Repository master input")]
    InvalidInput,
    /// The stable identity already has persisted history.
    #[error("Repository master identity already exists")]
    AlreadyExists,
    /// No current identity revision exists in this tenant.
    #[error("Repository master identity was not found")]
    NotFound,
    /// The supplied expected version is no longer current.
    #[error("Repository master version changed")]
    VersionConflict,
    /// The revision counter cannot be advanced safely.
    #[error("Repository master revision limit reached")]
    VersionOverflow,
    /// PostgreSQL operation failed or returned an invalid persisted row.
    #[error("Repository master persistence failed")]
    Persistence,
}

/// SCM owner persistence operations for the canonical tenant-owned Repository identity.
#[async_trait::async_trait]
pub trait ScmRepositoryMasterRepository: Send + Sync {
    /// Read the tenant's current metadata revision under owner-role RLS.
    async fn find_current(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
    ) -> Result<Option<RepositoryMasterRevision>, ScmRepositoryMasterError>;

    /// Create revision one and its append-only audit event in one transaction.
    async fn create(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        metadata: RepositoryMasterMetadata,
        actor_id: UserId,
        correlation_id: Uuid,
    ) -> Result<RepositoryMasterRevision, ScmRepositoryMasterError>;

    /// Compare-and-swap the current revision and append audit in the same transaction.
    async fn revise(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        expected_version: u32,
        metadata: RepositoryMasterMetadata,
        actor_id: UserId,
        correlation_id: Uuid,
    ) -> Result<RepositoryMasterRevision, ScmRepositoryMasterError>;
}

/// PostgreSQL adapter. This persists identity facts only; it does not authorize visibility.
#[derive(Clone)]
pub struct PgScmRepositoryMasterRepository {
    pool: PgPool,
}

impl PgScmRepositoryMasterRepository {
    /// Construct the adapter over the application's bounded shared pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ScmRepositoryMasterRepository for PgScmRepositoryMasterRepository {
    async fn find_current(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
    ) -> Result<Option<RepositoryMasterRevision>, ScmRepositoryMasterError> {
        if tenant_id.as_uuid().is_nil() || repository_id.as_uuid().is_nil() {
            return Err(ScmRepositoryMasterError::InvalidInput);
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        set_repository_owner_scope(&mut tx, tenant_id).await?;
        let row = sqlx::query(
            "SELECT tenant_id, repository_id, version, name, provider_key, remote_identity, remote_url, default_branch, created_by_actor_id, changed_by_actor_id, valid_from, valid_to FROM scm.repository_master WHERE tenant_id = $1 AND repository_id = $2 AND valid_to IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(repository_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        let revision = row.map(decode_revision).transpose()?;
        tx.commit()
            .await
            .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        Ok(revision)
    }

    async fn create(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        metadata: RepositoryMasterMetadata,
        actor_id: UserId,
        correlation_id: Uuid,
    ) -> Result<RepositoryMasterRevision, ScmRepositoryMasterError> {
        validate_write(
            tenant_id,
            repository_id,
            &metadata,
            actor_id,
            correlation_id,
        )?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        set_repository_write_scope(&mut tx, tenant_id, actor_id, correlation_id).await?;
        lock_repository_identity(&mut tx, tenant_id, repository_id).await?;
        let history_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM scm.repository_master WHERE tenant_id = $1 AND repository_id = $2)",
        )
        .bind(tenant_id.as_uuid())
        .bind(repository_id.as_uuid())
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        if history_exists {
            return Err(ScmRepositoryMasterError::AlreadyExists);
        }
        let row = insert_revision(
            &mut tx,
            tenant_id,
            repository_id,
            1,
            &metadata,
            actor_id,
            actor_id,
        )
        .await?;
        let revision = decode_revision(row)?;
        tx.commit()
            .await
            .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        Ok(revision)
    }

    async fn revise(
        &self,
        tenant_id: TenantId,
        repository_id: RepositoryId,
        expected_version: u32,
        metadata: RepositoryMasterMetadata,
        actor_id: UserId,
        correlation_id: Uuid,
    ) -> Result<RepositoryMasterRevision, ScmRepositoryMasterError> {
        validate_write(
            tenant_id,
            repository_id,
            &metadata,
            actor_id,
            correlation_id,
        )?;
        if expected_version == 0 {
            return Err(ScmRepositoryMasterError::InvalidInput);
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        set_repository_write_scope(&mut tx, tenant_id, actor_id, correlation_id).await?;
        lock_repository_identity(&mut tx, tenant_id, repository_id).await?;
        let current_row = sqlx::query(
            "SELECT tenant_id, repository_id, version, name, provider_key, remote_identity, remote_url, default_branch, created_by_actor_id, changed_by_actor_id, valid_from, valid_to FROM scm.repository_master WHERE tenant_id = $1 AND repository_id = $2 AND valid_to IS NULL FOR UPDATE",
        )
        .bind(tenant_id.as_uuid())
        .bind(repository_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?
        .ok_or(ScmRepositoryMasterError::NotFound)?;
        let current = decode_revision(current_row)?;
        if current.version != expected_version {
            return Err(ScmRepositoryMasterError::VersionConflict);
        }
        let next_version = current
            .version
            .checked_add(1)
            .filter(|version| i32::try_from(*version).is_ok())
            .ok_or(ScmRepositoryMasterError::VersionOverflow)?;
        let closed = sqlx::query(
            "UPDATE scm.repository_master SET valid_to = transaction_timestamp() WHERE tenant_id = $1 AND repository_id = $2 AND version = $3 AND valid_to IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .bind(repository_id.as_uuid())
        .bind(i32::try_from(current.version).map_err(|_| ScmRepositoryMasterError::Persistence)?)
        .execute(&mut *tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        if closed.rows_affected() != 1 {
            return Err(ScmRepositoryMasterError::VersionConflict);
        }
        let row = insert_revision(
            &mut tx,
            tenant_id,
            repository_id,
            next_version,
            &metadata,
            current.created_by,
            actor_id,
        )
        .await?;
        let revision = decode_revision(row)?;
        tx.commit()
            .await
            .map_err(|_| ScmRepositoryMasterError::Persistence)?;
        Ok(revision)
    }
}

async fn set_repository_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: TenantId,
) -> Result<(), ScmRepositoryMasterError> {
    sqlx::query("SET LOCAL ROLE star_scm_repository_owner")
        .execute(&mut **tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    sqlx::query("SET LOCAL lock_timeout = '2s'")
        .execute(&mut **tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    sqlx::query("SET LOCAL statement_timeout = '5s'")
        .execute(&mut **tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    set_local_value(tx, "app.tenant_id", tenant_id.as_uuid()).await
}

async fn set_repository_write_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: TenantId,
    actor_id: UserId,
    correlation_id: Uuid,
) -> Result<(), ScmRepositoryMasterError> {
    set_repository_owner_scope(tx, tenant_id).await?;
    set_local_value(tx, "app.actor_id", actor_id.as_uuid()).await?;
    set_local_value(tx, "app.correlation_id", correlation_id).await
}

async fn set_local_value(
    tx: &mut Transaction<'_, Postgres>,
    setting: &'static str,
    value: Uuid,
) -> Result<(), ScmRepositoryMasterError> {
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind(setting)
        .bind(value.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    Ok(())
}

async fn lock_repository_identity(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: TenantId,
    repository_id: RepositoryId,
) -> Result<(), ScmRepositoryMasterError> {
    let lock_key = format!("{}:{}", tenant_id, repository_id);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(lock_key)
        .execute(&mut **tx)
        .await
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    Ok(())
}

async fn insert_revision(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: TenantId,
    repository_id: RepositoryId,
    version: u32,
    metadata: &RepositoryMasterMetadata,
    created_by: UserId,
    changed_by: UserId,
) -> Result<PgRow, ScmRepositoryMasterError> {
    let version = i32::try_from(version).map_err(|_| ScmRepositoryMasterError::VersionOverflow)?;
    sqlx::query(
        "INSERT INTO scm.repository_master (tenant_id, repository_id, version, name, provider_key, remote_identity, remote_url, default_branch, created_by_actor_id, changed_by_actor_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING tenant_id, repository_id, version, name, provider_key, remote_identity, remote_url, default_branch, created_by_actor_id, changed_by_actor_id, valid_from, valid_to",
    )
    .bind(tenant_id.as_uuid())
    .bind(repository_id.as_uuid())
    .bind(version)
    .bind(&metadata.name)
    .bind(&metadata.provider_key)
    .bind(&metadata.remote_identity)
    .bind(&metadata.remote_url)
    .bind(&metadata.default_branch)
    .bind(created_by.as_uuid())
    .bind(changed_by.as_uuid())
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| ScmRepositoryMasterError::Persistence)
}

fn validate_write(
    tenant_id: TenantId,
    repository_id: RepositoryId,
    metadata: &RepositoryMasterMetadata,
    actor_id: UserId,
    correlation_id: Uuid,
) -> Result<(), ScmRepositoryMasterError> {
    if tenant_id.as_uuid().is_nil()
        || repository_id.as_uuid().is_nil()
        || actor_id.as_uuid().is_nil()
        || correlation_id.is_nil()
    {
        return Err(ScmRepositoryMasterError::InvalidInput);
    }
    metadata
        .validate()
        .map_err(|_| ScmRepositoryMasterError::InvalidInput)
}

fn decode_revision(row: PgRow) -> Result<RepositoryMasterRevision, ScmRepositoryMasterError> {
    let version: i32 = row
        .try_get("version")
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    let version = u32::try_from(version).map_err(|_| ScmRepositoryMasterError::Persistence)?;
    let revision = RepositoryMasterRevision {
        tenant_id: TenantId(
            row.try_get("tenant_id")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
        ),
        id: RepositoryId(
            row.try_get("repository_id")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
        ),
        version,
        metadata: RepositoryMasterMetadata {
            name: row
                .try_get("name")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
            provider_key: row
                .try_get("provider_key")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
            remote_identity: row
                .try_get("remote_identity")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
            remote_url: row
                .try_get("remote_url")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
            default_branch: row
                .try_get("default_branch")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
        },
        created_by: UserId(
            row.try_get("created_by_actor_id")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
        ),
        changed_by: UserId(
            row.try_get("changed_by_actor_id")
                .map_err(|_| ScmRepositoryMasterError::Persistence)?,
        ),
        valid_from: row
            .try_get::<DateTime<Utc>, _>("valid_from")
            .map_err(|_| ScmRepositoryMasterError::Persistence)?,
        valid_to: row
            .try_get::<Option<DateTime<Utc>>, _>("valid_to")
            .map_err(|_| ScmRepositoryMasterError::Persistence)?,
    };
    revision
        .metadata
        .validate()
        .map_err(|_| ScmRepositoryMasterError::Persistence)?;
    Ok(revision)
}
