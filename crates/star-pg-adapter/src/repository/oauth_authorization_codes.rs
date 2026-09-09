// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuthAuthorizationCode Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/oauth_authorization_codes.sql` 表对应
//! - 守门 #13 T 类: 物理删除禁止 + 監査必須 + RLS 13 類必携
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::PgAdapterError;

/// OAuthAuthorizationCode (W/T/M = T, 跟 DDL oauth_authorization_codes 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OAuthAuthorizationCode {
    /// 主键
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// sha256(code) 哈希 (守门 #5 v2 不存明文)
    pub code_hash: String,
    /// 客户端 ID
    pub client_id: String,
    /// 授权人 (RLS 13 類)
    pub user_id: Uuid,
    /// redirect URI
    pub redirect_uri: String,
    /// scope 字符串 (空格分隔)
    pub scope: String,
    /// PKCE code_challenge
    pub code_challenge: String,
    /// plain/S256
    pub code_challenge_method: String,
    /// 10 分钟过期
    pub expires_at: DateTime<Utc>,
    /// 兑换时间 (WORM 派生)
    pub consumed_at: Option<DateTime<Utc>>,
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

/// OAuthAuthorizationCodeRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OAuthAuthorizationCodeRepository: Send + Sync {
    /// find_by_code_hash (per DDL oauth_authorization_codes T 派生规)
    async fn find_by_code_hash(
        &self,
        tenant_id: Uuid,
        code_hash: &str,
    ) -> Result<Option<OAuthAuthorizationCode>, PgAdapterError>;
    /// mark_consumed (per DDL oauth_authorization_codes T 派生规)
    async fn mark_consumed(
        &self,
        tenant_id: Uuid,
        code_hash: &str,
        consumed_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError>;
    /// insert (per DDL oauth_authorization_codes T 派生规)
    async fn insert(&self, code: &OAuthAuthorizationCode) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/oauth_authorization_codes 对应)
pub struct PgOAuthAuthorizationCodeRepository {
    pool: PgPool,
}

impl PgOAuthAuthorizationCodeRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OAuthAuthorizationCodeRepository for PgOAuthAuthorizationCodeRepository {
    async fn find_by_code_hash(
        &self,
        tenant_id: Uuid,
        code_hash: &str,
    ) -> Result<Option<OAuthAuthorizationCode>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OAuthAuthorizationCode>(
            r#"
                        SELECT id, tenant_id, code_hash, client_id, user_id, redirect_uri, scope, code_challenge, code_challenge_method, expires_at, consumed_at, created_at, source_module, source_kind, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM oauth_authorization_codes
            WHERE tenant_id = $1 AND code_hash = $2 AND consumed_at IS NULL
              AND expires_at > NOW()
            "#,
        )
        .bind(tenant_id)
        .bind(code_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_code_hash: {}", e)))?;
        Ok(rows)
    }

    async fn mark_consumed(
        &self,
        tenant_id: Uuid,
        code_hash: &str,
        consumed_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        UPDATE oauth_authorization_codes
            SET consumed_at = $3
            WHERE tenant_id = $1 AND code_hash = $2 AND consumed_at IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(code_hash)
        .bind(&consumed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("mark_consumed: {}", e)))?;
        Ok(())
    }

    async fn insert(&self, code: &OAuthAuthorizationCode) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        INSERT INTO oauth_authorization_codes
                (id, tenant_id, code_hash, client_id, user_id, redirect_uri, scope, code_challenge, code_challenge_method, expires_at, consumed_at, created_at, source_module, source_kind, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            "#,
        )
        .bind(&code.id)
        .bind(&code.tenant_id)
        .bind(&code.code_hash)
        .bind(&code.client_id)
        .bind(&code.user_id)
        .bind(&code.redirect_uri)
        .bind(&code.scope)
        .bind(&code.code_challenge)
        .bind(&code.code_challenge_method)
        .bind(&code.expires_at)
        .bind(&code.consumed_at)
        .bind(&code.created_at)
        .bind(&code.source_module)
        .bind(&code.source_kind)
        .bind(&code.role_id)
        .bind(&code.permission_id)
        .bind(&code.policy_id)
        .bind(&code.workspace_id)
        .bind(&code.project_id)
        .bind(&code.work_item_id)
        .bind(&code.agent_id)
        .bind(&code.session_id)
        .bind(&code.trace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OAuthAuthorizationCode 必含 23 字段 (跟 DDL 100% 一致)
    #[test]
    fn oauth_authorization_codes_struct_has_23_fields() {
        let x = OAuthAuthorizationCode {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            code_hash: "test".to_string(),
            client_id: "test".to_string(),
            user_id: Uuid::nil(),
            redirect_uri: "test".to_string(),
            scope: "test".to_string(),
            code_challenge: "test".to_string(),
            code_challenge_method: "test".to_string(),
            expires_at: Utc::now(),
            consumed_at: None,
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

    /// 派生 #N: OAuthAuthorizationCodeRepository trait 必含 3 方法
    #[test]
    fn oauth_authorization_codes_repository_trait_has_3_methods() {
        fn _check_methods<T: OAuthAuthorizationCodeRepository>() {}
        _check_methods::<PgOAuthAuthorizationCodeRepository>();
    }
}
