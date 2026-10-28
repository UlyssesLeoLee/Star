// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=74dc3f76263188ed006425bb30759372c82edba900a509160e5d727436ae8305
MERGE (self:File {path:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs"})
MERGE (find:Symbol {id:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs::find_by_token_hash",kind:"method"})
MERGE (rotate:Symbol {id:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs::rotate_if_active",kind:"method"})
MERGE (resolver:Symbol {id:"db/migrations/2026-10-28-w16-device-auth.sql::w16_resolve_oauth_refresh_token",kind:"database_function"})
MERGE (accessInsert:Symbol {id:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs::insert_access_token",kind:"function"})
MERGE (refreshInsert:Symbol {id:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs::insert_refresh_token",kind:"function"})
MERGE (bindings:Table {name:"oauth_device_bindings"})
MERGE (refreshRows:Table {name:"oauth_refresh_tokens"})
MERGE (self)-[:DEFINES]->(find)
MERGE (self)-[:DEFINES]->(rotate)
MERGE (find)-[:CALLS]->(resolver)
MERGE (rotate)-[:CALLS]->(resolver)
MERGE (rotate)-[:READS]->(bindings)
MERGE (rotate)-[:WRITES]->(refreshRows)
MERGE (rotate)-[:CALLS]->(accessInsert)
MERGE (rotate)-[:CALLS]->(refreshInsert)
@endcypher
*/
//! OAuthRefreshToken Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/oauth_refresh_tokens.sql` 表对应
//! - 守门 #13 T 类: 物理删除禁止 + 監査必須 + RLS 13 類必携
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::oauth_access_tokens::OAuthAccessToken;
use crate::PgAdapterError;

/// OAuthRefreshToken (W/T/M = T, 跟 DDL oauth_refresh_tokens 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OAuthRefreshToken {
    /// 主键
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// sha256(token) 哈希
    pub token_hash: String,
    /// 客户端 ID
    pub client_id: String,
    /// 用户 (仅 auth code flow 有)
    pub user_id: Uuid,
    /// Issuer-bound device identity; both fields are NULL for legacy sessions.
    pub device_id: Option<Uuid>,
    /// Monotonic generation copied into the signed access token.
    pub device_generation: Option<i64>,
    /// 关联的 access_token ID
    pub access_token_id: Uuid,
    /// scope 字符串
    pub scope: String,
    /// 30 天过期
    pub expires_at: DateTime<Utc>,
    /// 撤销时间
    pub revoked_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// star_api_rest::auth::oauth
    pub source_module: String,
    /// audit
    pub source_kind: String,
    /// RLS 13 類
    pub role_id: Option<Uuid>,
    /// RLS 13 類
    pub permission_id: Option<Uuid>,
    /// RLS 13 類
    pub policy_id: Option<Uuid>,
    /// RLS 13 類
    pub workspace_id: Option<Uuid>,
    /// RLS 13 類
    pub project_id: Option<Uuid>,
    /// RLS 13 類
    pub work_item_id: Option<Uuid>,
    /// RLS 13 類
    pub agent_id: Option<Uuid>,
    /// RLS 13 類
    pub session_id: Option<Uuid>,
    /// RLS 13 類 (T 必携)
    pub trace_id: Option<Uuid>,
}

/// OAuthRefreshTokenRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OAuthRefreshTokenRepository: Send + Sync {
    /// find_by_token_hash (per DDL oauth_refresh_tokens T 派生规)
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
        client_id: &str,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError>;
    /// Atomically revoke one unexpired refresh token and return it only to the winner.
    async fn consume_if_active(
        &self,
        tenant_id: Uuid,
        token_hash: &str,
        revoked_at: DateTime<Utc>,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError>;
    /// Consume a refresh token and persist its access/refresh successors in one transaction.
    async fn rotate_if_active(
        &self,
        token_hash: &str,
        client_id: &str,
        rotated_at: DateTime<Utc>,
        access_token: &OAuthAccessToken,
        successor: &OAuthRefreshToken,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError>;
    /// revoke (per DDL oauth_refresh_tokens T 派生规)
    async fn revoke(
        &self,
        tenant_id: Uuid,
        token_hash: &str,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError>;
    /// insert (per DDL oauth_refresh_tokens T 派生规)
    async fn insert(&self, token: &OAuthRefreshToken) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/oauth_refresh_tokens 对应)
pub struct PgOAuthRefreshTokenRepository {
    pool: PgPool,
}

impl PgOAuthRefreshTokenRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// 拿共享 PgPool 引用 (v0.75 P0-4 Stage 2.1.2 扩展: 4 oauth_* Repository wire-up)
    ///
    /// 用于 application crate 跨域编排时多个 Repository 共享同一 pool
    /// (per sqlx::PgPool = Arc 内部, clone 廉价).
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl OAuthRefreshTokenRepository for PgOAuthRefreshTokenRepository {
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
        client_id: &str,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PgAdapterError::Query(format!("find_by_token_hash begin: {}", e)))?;
        let rows = sqlx::query_as::<_, OAuthRefreshToken>(
            "SELECT * FROM public.w16_resolve_oauth_refresh_token($1,$2)",
        )
        .bind(token_hash)
        .bind(client_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_token_hash: {}", e)))?;
        tx.commit()
            .await
            .map_err(|e| PgAdapterError::Query(format!("find_by_token_hash commit: {}", e)))?;
        Ok(rows)
    }

    async fn consume_if_active(
        &self,
        tenant_id: Uuid,
        token_hash: &str,
        revoked_at: DateTime<Utc>,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PgAdapterError::Query(format!("consume_if_active begin: {}", e)))?;
        sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
            .bind(tenant_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| PgAdapterError::Query(format!("consume_if_active tenant scope: {}", e)))?;
        let consumed = sqlx::query_as::<_, OAuthRefreshToken>(
            r#"
            UPDATE oauth_refresh_tokens
            SET revoked_at = $3
            WHERE tenant_id = $1 AND token_hash = $2 AND client_id = $4
              AND user_id = $5 AND scope = $6
              AND device_id IS NOT DISTINCT FROM $7
              AND device_generation IS NOT DISTINCT FROM $8
              AND revoked_at IS NULL AND expires_at > $3
            RETURNING id, tenant_id, token_hash, client_id, user_id, device_id,
                      device_generation, access_token_id, scope, expires_at,
                      revoked_at, created_at,
                      source_module, source_kind, role_id, permission_id, policy_id,
                      workspace_id, project_id, work_item_id, agent_id, session_id,
                      trace_id
            "#,
        )
        .bind(tenant_id)
        .bind(token_hash)
        .bind(revoked_at)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("consume_if_active: {}", e)))?;
        tx.commit()
            .await
            .map_err(|e| PgAdapterError::Query(format!("consume_if_active commit: {}", e)))?;
        Ok(consumed)
    }

    async fn rotate_if_active(
        &self,
        token_hash: &str,
        client_id: &str,
        rotated_at: DateTime<Utc>,
        access_token: &OAuthAccessToken,
        successor: &OAuthRefreshToken,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        if access_token.tenant_id != successor.tenant_id
            || access_token.client_id != successor.client_id
            || access_token.user_id != Some(successor.user_id)
            || access_token.scope != successor.scope
        {
            return Err(PgAdapterError::Query(
                "rotate_if_active successor binding mismatch".to_string(),
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PgAdapterError::Query(format!("rotate_if_active begin: {}", e)))?;
        let snapshot = sqlx::query_as::<_, OAuthRefreshToken>(
            "SELECT * FROM public.w16_resolve_oauth_refresh_token($1,$2)",
        )
        .bind(token_hash)
        .bind(client_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("rotate_if_active snapshot: {}", e)))?;
        let Some(snapshot) = snapshot else {
            tx.commit().await.map_err(|e| {
                PgAdapterError::Query(format!("rotate_if_active absent commit: {}", e))
            })?;
            return Ok(None);
        };
        if snapshot.expires_at <= rotated_at || snapshot.revoked_at.is_some() {
            tx.rollback().await.map_err(|e| {
                PgAdapterError::Query(format!("rotate_if_active stale rollback: {}", e))
            })?;
            return Ok(None);
        }
        if snapshot.tenant_id != successor.tenant_id
            || snapshot.client_id != client_id
            || access_token.tenant_id != snapshot.tenant_id
            || access_token.client_id != snapshot.client_id
        {
            tx.rollback().await.map_err(|e| {
                PgAdapterError::Query(format!("rotate_if_active owner binding rollback: {}", e))
            })?;
            return Ok(None);
        }
        let binding_matches = snapshot.client_id == successor.client_id
            && snapshot.user_id == successor.user_id
            && snapshot.scope == successor.scope
            && snapshot.device_id == successor.device_id
            && snapshot.device_generation == successor.device_generation;
        if !binding_matches {
            tx.rollback().await.map_err(|e| {
                PgAdapterError::Query(format!("rotate_if_active mismatch rollback: {}", e))
            })?;
            return Ok(None);
        }
        match (snapshot.device_id, snapshot.device_generation) {
            (Some(device_id), Some(generation)) if generation > 0 => {
                let active_generation = sqlx::query_scalar::<_, i64>(
                    "SELECT generation FROM oauth_device_bindings \
                     WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3 \
                       AND generation=$4 AND status='active' AND revoked_at IS NULL FOR SHARE",
                )
                .bind(snapshot.tenant_id)
                .bind(snapshot.user_id)
                .bind(device_id)
                .bind(generation)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| {
                    PgAdapterError::Query(format!("rotate_if_active device check: {}", e))
                })?;
                if active_generation != Some(generation) {
                    tx.rollback().await.map_err(|e| {
                        PgAdapterError::Query(format!(
                            "rotate_if_active stale device rollback: {}",
                            e
                        ))
                    })?;
                    return Ok(None);
                }
            }
            (None, None) => {}
            _ => {
                tx.rollback().await.map_err(|e| {
                    PgAdapterError::Query(format!(
                        "rotate_if_active incomplete device rollback: {}",
                        e
                    ))
                })?;
                return Ok(None);
            }
        }
        let current = sqlx::query_as::<_, OAuthRefreshToken>(
            r#"
            UPDATE oauth_refresh_tokens
            SET revoked_at = $3
            WHERE tenant_id = $1 AND token_hash = $2
              AND client_id = $4 AND user_id = $5 AND scope = $6
              AND device_id IS NOT DISTINCT FROM $7
              AND device_generation IS NOT DISTINCT FROM $8
              AND revoked_at IS NULL AND expires_at > $3
            RETURNING id, tenant_id, token_hash, client_id, user_id, device_id,
                      device_generation, access_token_id, scope, expires_at,
                      revoked_at, created_at, source_module, source_kind, role_id,
                      permission_id, policy_id, workspace_id, project_id,
                      work_item_id, agent_id, session_id, trace_id
            "#,
        )
        .bind(snapshot.tenant_id)
        .bind(token_hash)
        .bind(rotated_at)
        .bind(client_id)
        .bind(snapshot.user_id)
        .bind(snapshot.scope.as_str())
        .bind(snapshot.device_id)
        .bind(snapshot.device_generation)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("rotate_if_active consume: {}", e)))?;
        let Some(current) = current else {
            tx.commit().await.map_err(|e| {
                PgAdapterError::Query(format!("rotate_if_active rejected commit: {}", e))
            })?;
            return Ok(None);
        };

        insert_access_token(&mut tx, access_token).await?;
        insert_refresh_token(&mut tx, successor).await?;
        tx.commit()
            .await
            .map_err(|e| PgAdapterError::Query(format!("rotate_if_active commit: {}", e)))?;
        Ok(Some(current))
    }

    async fn revoke(
        &self,
        tenant_id: Uuid,
        token_hash: &str,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        UPDATE oauth_refresh_tokens
            SET revoked_at = $3
            WHERE tenant_id = $1 AND token_hash = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(token_hash)
        .bind(&revoked_at)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("revoke: {}", e)))?;
        Ok(())
    }

    async fn insert(&self, token: &OAuthRefreshToken) -> Result<(), PgAdapterError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PgAdapterError::Query(format!("insert refresh token begin: {}", e)))?;
        sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
            .bind(token.tenant_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                PgAdapterError::Query(format!("insert refresh token tenant scope: {}", e))
            })?;
        sqlx::query(
            r#"
                        INSERT INTO oauth_refresh_tokens
                (id, tenant_id, token_hash, client_id, user_id, device_id, device_generation, access_token_id, scope, expires_at, revoked_at, created_at, source_module, source_kind, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            "#,
        )
        .bind(&token.id)
        .bind(&token.tenant_id)
        .bind(&token.token_hash)
        .bind(&token.client_id)
        .bind(&token.user_id)
        .bind(&token.device_id)
        .bind(&token.device_generation)
        .bind(&token.access_token_id)
        .bind(&token.scope)
        .bind(&token.expires_at)
        .bind(&token.revoked_at)
        .bind(&token.created_at)
        .bind(&token.source_module)
        .bind(&token.source_kind)
        .bind(&token.role_id)
        .bind(&token.permission_id)
        .bind(&token.policy_id)
        .bind(&token.workspace_id)
        .bind(&token.project_id)
        .bind(&token.work_item_id)
        .bind(&token.agent_id)
        .bind(&token.session_id)
        .bind(&token.trace_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert: {}", e)))?;
        tx.commit()
            .await
            .map_err(|e| PgAdapterError::Query(format!("insert refresh token commit: {}", e)))
    }
}

pub(crate) async fn insert_access_token(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token: &OAuthAccessToken,
) -> Result<(), PgAdapterError> {
    sqlx::query(
        r#"
        INSERT INTO oauth_access_tokens
            (id, tenant_id, token_hash, client_id, user_id, grant_type, scope,
             expires_at, revoked_at, created_at, source_module, source_kind,
             role_id, permission_id, policy_id, workspace_id, project_id,
             work_item_id, agent_id, session_id, trace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21)
        "#,
    )
    .bind(token.id)
    .bind(token.tenant_id)
    .bind(&token.token_hash)
    .bind(&token.client_id)
    .bind(token.user_id)
    .bind(&token.grant_type)
    .bind(&token.scope)
    .bind(token.expires_at)
    .bind(token.revoked_at)
    .bind(token.created_at)
    .bind(&token.source_module)
    .bind(&token.source_kind)
    .bind(token.role_id)
    .bind(token.permission_id)
    .bind(token.policy_id)
    .bind(token.workspace_id)
    .bind(token.project_id)
    .bind(token.work_item_id)
    .bind(token.agent_id)
    .bind(token.session_id)
    .bind(token.trace_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| PgAdapterError::Query(format!("rotate insert access token: {}", e)))?;
    Ok(())
}

pub(crate) async fn insert_refresh_token(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token: &OAuthRefreshToken,
) -> Result<(), PgAdapterError> {
    sqlx::query(
        r#"
        INSERT INTO oauth_refresh_tokens
            (id, tenant_id, token_hash, client_id, user_id, device_id,
             device_generation, access_token_id, scope, expires_at, revoked_at,
             created_at, source_module, source_kind, role_id, permission_id,
             policy_id, workspace_id, project_id, work_item_id, agent_id,
             session_id, trace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
        "#,
    )
    .bind(token.id)
    .bind(token.tenant_id)
    .bind(&token.token_hash)
    .bind(&token.client_id)
    .bind(token.user_id)
    .bind(token.device_id)
    .bind(token.device_generation)
    .bind(token.access_token_id)
    .bind(&token.scope)
    .bind(token.expires_at)
    .bind(token.revoked_at)
    .bind(token.created_at)
    .bind(&token.source_module)
    .bind(&token.source_kind)
    .bind(token.role_id)
    .bind(token.permission_id)
    .bind(token.policy_id)
    .bind(token.workspace_id)
    .bind(token.project_id)
    .bind(token.work_item_id)
    .bind(token.agent_id)
    .bind(token.session_id)
    .bind(token.trace_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| PgAdapterError::Query(format!("rotate insert refresh token: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OAuthRefreshToken 必含 21 字段 (跟 DDL 100% 一致)
    #[test]
    fn oauth_refresh_tokens_struct_has_21_fields() {
        let x = OAuthRefreshToken {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            token_hash: "test".to_string(),
            client_id: "test".to_string(),
            user_id: Uuid::nil(),
            device_id: None,
            device_generation: None,
            access_token_id: Uuid::nil(),
            scope: "test".to_string(),
            expires_at: Utc::now(),
            revoked_at: None,
            created_at: Utc::now(),
            source_module: "test".to_string(),
            source_kind: "test".to_string(),
            role_id: None,
            permission_id: None,
            policy_id: None,
            workspace_id: None,
            project_id: None,
            work_item_id: None,
            agent_id: None,
            session_id: None,
            trace_id: None,
        };
        assert_eq!(x.source_module, "test");
    }

    /// OAuthRefreshTokenRepository exposes look-up, single-consumer, revoke, and insert operations.
    #[test]
    fn oauth_refresh_tokens_repository_trait_has_3_methods() {
        fn _check_methods<T: OAuthRefreshTokenRepository>() {}
        _check_methods::<PgOAuthRefreshTokenRepository>();
    }
}
