// SPDX-License-Identifier: MIT OR Apache-2.0
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
        tenant_id: Uuid,
        token_hash: &str,
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
}

#[async_trait::async_trait]
impl OAuthRefreshTokenRepository for PgOAuthRefreshTokenRepository {
    async fn find_by_token_hash(
        &self,
        tenant_id: Uuid,
        token_hash: &str,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OAuthRefreshToken>(
            r#"
                        SELECT id, tenant_id, token_hash, client_id, user_id, access_token_id, scope, expires_at, revoked_at, created_at, source_module, source_kind, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM oauth_refresh_tokens
            WHERE tenant_id = $1 AND token_hash = $2 AND revoked_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(tenant_id)
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_token_hash: {}", e)))?;
        Ok(rows)
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
        sqlx::query(
            r#"
                        INSERT INTO oauth_refresh_tokens
                (id, tenant_id, token_hash, client_id, user_id, access_token_id, scope, expires_at, revoked_at, created_at, source_module, source_kind, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            "#,
        )
        .bind(&token.id)
        .bind(&token.tenant_id)
        .bind(&token.token_hash)
        .bind(&token.client_id)
        .bind(&token.user_id)
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
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert: {}", e)))?;
        Ok(())
    }
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

    /// 派生 #N: OAuthRefreshTokenRepository trait 必含 3 方法
    #[test]
    fn oauth_refresh_tokens_repository_trait_has_3_methods() {
        fn _check_methods<T: OAuthRefreshTokenRepository>() {}
        _check_methods::<PgOAuthRefreshTokenRepository>();
    }
}
