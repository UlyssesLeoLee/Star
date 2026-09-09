// SPDX-License-Identifier: MIT OR Apache-2.0
//! OAuthClient Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
//!
//! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
//! - 跟 `db/migrations/oauth_clients.sql` 表对应
//! - 守门 #13 M 类: 物理删除 / タイマー失効 / 短 TTL 明示 retention
//! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
//! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::PgAdapterError;

/// OAuthClient (W/T/M = M, 跟 DDL oauth_clients 100% 对齐)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct OAuthClient {
    /// 主键 (复合 PK id+valid_from)
    pub id: Uuid,
    /// 租户 ID (RLS 必携)
    pub tenant_id: Uuid,
    /// 公开 client_id (e.g. star-frontend-spa)
    pub client_id: String,
    /// bcrypt 哈希 (confidential 才有)
    pub client_secret_hash: Option<String>,
    /// 显示名
    pub client_name: String,
    /// public/confidential
    pub client_type: String,
    /// redirect URI 列表 (public auth code 需要)
    pub redirect_uris: Vec<String>,
    /// 允许的 scope (e.g. read/write/admin)
    pub allowed_scopes: Vec<String>,
    /// authorization_code/client_credentials/refresh_token
    pub allowed_grant_types: Vec<String>,
    /// 强制 PKCE (public 默认 TRUE)
    pub require_pkce: bool,
    /// 需要 client_secret (confidential 默认 TRUE)
    pub require_authentication: bool,
    /// 注册人
    pub owner_user_id: Uuid,
    /// SCD Type 2 生效起始
    pub valid_from: DateTime<Utc>,
    /// SCD Type 2 生效结束
    pub valid_to: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// star_api_rest::auth::oauth
    pub source_module: String,
    /// master
    pub source_kind: String,
    /// RLS 13 類
    pub user_id: Option<Uuid>,
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

/// OAuthClientRepository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
#[async_trait::async_trait]
pub trait OAuthClientRepository: Send + Sync {
    /// find_by_client_id (per DDL oauth_clients M 派生规)
    async fn find_by_client_id(
        &self,
        tenant_id: Uuid,
        client_id: &str,
    ) -> Result<Option<OAuthClient>, PgAdapterError>;
    /// list_current (per DDL oauth_clients M 派生规)
    async fn list_current(&self, tenant_id: Uuid) -> Result<Vec<OAuthClient>, PgAdapterError>;
    /// insert_new_version (per DDL oauth_clients M 派生规)
    async fn insert_new_version(&self, client: &OAuthClient) -> Result<(), PgAdapterError>;
}

/// Postgres impl (跟 db/migrations/oauth_clients 对应)
pub struct PgOAuthClientRepository {
    pool: PgPool,
}

impl PgOAuthClientRepository {
    /// 构造 PG Repository (持有 PgPool 引用)
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OAuthClientRepository for PgOAuthClientRepository {
    async fn find_by_client_id(
        &self,
        tenant_id: Uuid,
        client_id: &str,
    ) -> Result<Option<OAuthClient>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OAuthClient>(
            r#"
                        SELECT id, tenant_id, client_id, client_secret_hash, client_name, client_type, redirect_uris, allowed_scopes, allowed_grant_types, require_pkce, require_authentication, owner_user_id, valid_from, valid_to, created_at, updated_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM oauth_clients
            WHERE tenant_id = $1 AND client_id = $2 AND valid_to IS NULL
            "#,
        )
        .bind(tenant_id)
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("find_by_client_id: {}", e)))?;
        Ok(rows)
    }

    async fn list_current(&self, tenant_id: Uuid) -> Result<Vec<OAuthClient>, PgAdapterError> {
        let rows = sqlx::query_as::<_, OAuthClient>(
            r#"
                        SELECT id, tenant_id, client_id, client_secret_hash, client_name, client_type, redirect_uris, allowed_scopes, allowed_grant_types, require_pkce, require_authentication, owner_user_id, valid_from, valid_to, created_at, updated_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id
            FROM oauth_clients
            WHERE tenant_id = $1 AND valid_to IS NULL
            ORDER BY client_name
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("list_current: {}", e)))?;
        Ok(rows)
    }

    async fn insert_new_version(&self, client: &OAuthClient) -> Result<(), PgAdapterError> {
        sqlx::query(
            r#"
                        INSERT INTO oauth_clients
                (id, tenant_id, client_id, client_secret_hash, client_name, client_type, redirect_uris, allowed_scopes, allowed_grant_types, require_pkce, require_authentication, owner_user_id, valid_from, valid_to, created_at, updated_at, source_module, source_kind, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
        )
        .bind(&client.id)
        .bind(&client.tenant_id)
        .bind(&client.client_id)
        .bind(&client.client_secret_hash)
        .bind(&client.client_name)
        .bind(&client.client_type)
        .bind(&client.redirect_uris)
        .bind(&client.allowed_scopes)
        .bind(&client.allowed_grant_types)
        .bind(client.require_pkce)
        .bind(client.require_authentication)
        .bind(&client.owner_user_id)
        .bind(&client.valid_from)
        .bind(&client.valid_to)
        .bind(&client.created_at)
        .bind(&client.updated_at)
        .bind(&client.source_module)
        .bind(&client.source_kind)
        .bind(&client.user_id)
        .bind(&client.role_id)
        .bind(&client.permission_id)
        .bind(&client.policy_id)
        .bind(&client.workspace_id)
        .bind(&client.project_id)
        .bind(&client.work_item_id)
        .bind(&client.agent_id)
        .bind(&client.session_id)
        .bind(&client.trace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PgAdapterError::Query(format!("insert_new_version: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 派生 #N: OAuthClient 必含 28 字段 (跟 DDL 100% 一致)
    #[test]
    fn oauth_clients_struct_has_28_fields() {
        let x = OAuthClient {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            client_id: "test".to_string(),
            client_secret_hash: None,
            client_name: "test".to_string(),
            client_type: "test".to_string(),
            redirect_uris: Default::default(),
            allowed_scopes: Default::default(),
            allowed_grant_types: Default::default(),
            require_pkce: false,
            require_authentication: false,
            owner_user_id: Uuid::nil(),
            valid_from: Utc::now(),
            valid_to: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source_module: "test".to_string(),
            source_kind: "test".to_string(),
            user_id: None,
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

    /// 派生 #N: OAuthClientRepository trait 必含 3 方法
    #[test]
    fn oauth_clients_repository_trait_has_3_methods() {
        fn _check_methods<T: OAuthClientRepository>() {}
        _check_methods::<PgOAuthClientRepository>();
    }
}
