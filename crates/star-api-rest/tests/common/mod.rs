// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mock OAuth2 repository implementations for integration tests
//! (per brief v0.51 §3 "mock OAuth2State" + 守门 #25 v25 no_network_mode)
//!
//! Provides `InMemory` (or `Mock*`) impls of the 4 OAuth*Repository traits
//! from `star-pg-adapter`. These store state in `Arc<Mutex<...>>` and let us
//! drive the 5 OAuth2 endpoints end-to-end without a real Postgres pool.
//!
//! 守门 #25 v25: 集成测试不发真 HTTP (走 axum::Router::oneshot); 这里不发真 PG.
//! 守门 #5 v2: token 不入 log; 错误路径只暴露错误 code, 不打 hash/secret.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use star_api_rest::auth::oauth::keypair::OAuthKeyManager;
use star_api_rest::auth::oauth::OAuth2State;
use star_api_rest::auth::JwtConfig;
use star_pg_adapter::repository::{
    OAuthAccessToken, OAuthAccessTokenRepository, OAuthAuthorizationCode,
    OAuthAuthorizationCodeRepository, OAuthClient, OAuthClientRepository, OAuthRefreshToken,
    OAuthRefreshTokenRepository,
};
use star_pg_adapter::PgAdapterError;

// ============================================
// Mock OAuthClientRepository
// ============================================

/// In-memory mock for `OAuthClientRepository`.
///
/// Pre-populate via `insert(client)`; reads return clones.
#[derive(Default)]
pub(crate) struct MockOAuthClientRepository {
    inner: Arc<Mutex<HashMap<String, OAuthClient>>>,
}

impl MockOAuthClientRepository {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Pre-populate a client (by `client_id`).
    pub(crate) fn insert(&self, client: OAuthClient) {
        let mut map = self.inner.lock().expect("poisoned");
        map.insert(client.client_id.clone(), client);
    }
}

#[async_trait]
impl OAuthClientRepository for MockOAuthClientRepository {
    async fn find_by_client_id(
        &self,
        _tenant_id: Uuid,
        client_id: &str,
    ) -> Result<Option<OAuthClient>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        Ok(map.get(client_id).cloned())
    }

    async fn list_current(&self, _tenant_id: Uuid) -> Result<Vec<OAuthClient>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        Ok(map.values().cloned().collect())
    }

    async fn insert_new_version(&self, client: &OAuthClient) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        map.insert(client.client_id.clone(), client.clone());
        Ok(())
    }
}

// ============================================
// Mock OAuthAuthorizationCodeRepository
// ============================================

/// In-memory mock for `OAuthAuthorizationCodeRepository`.
#[derive(Default)]
pub(crate) struct MockOAuthAuthorizationCodeRepository {
    inner: Arc<Mutex<HashMap<String, OAuthAuthorizationCode>>>,
}

impl MockOAuthAuthorizationCodeRepository {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Pre-populate a code (by `code_hash`).
    pub(crate) fn insert(&self, code: OAuthAuthorizationCode) {
        let mut map = self.inner.lock().expect("poisoned");
        map.insert(code.code_hash.clone(), code);
    }

    /// Read-only snapshot of stored codes (by code_hash).
    pub(crate) fn snapshot(&self) -> Vec<OAuthAuthorizationCode> {
        let map = self.inner.lock().expect("poisoned");
        map.values().cloned().collect()
    }
}

#[async_trait]
impl OAuthAuthorizationCodeRepository for MockOAuthAuthorizationCodeRepository {
    async fn find_by_code_hash(
        &self,
        _tenant_id: Uuid,
        code_hash: &str,
    ) -> Result<Option<OAuthAuthorizationCode>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        // Mirror the real impl's behavior: skip consumed or expired codes.
        let now = Utc::now();
        Ok(map.get(code_hash).and_then(|c| {
            if c.consumed_at.is_some() || c.expires_at <= now {
                None
            } else {
                Some(c.clone())
            }
        }))
    }

    async fn mark_consumed(
        &self,
        _tenant_id: Uuid,
        code_hash: &str,
        consumed_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        if let Some(c) = map.get_mut(code_hash) {
            c.consumed_at = Some(consumed_at);
            Ok(())
        } else {
            Err(PgAdapterError::Query(format!(
                "code_hash {} not found",
                code_hash
            )))
        }
    }

    async fn insert(&self, code: &OAuthAuthorizationCode) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        map.insert(code.code_hash.clone(), code.clone());
        Ok(())
    }
}

// ============================================
// Mock OAuthAccessTokenRepository
// ============================================

/// In-memory mock for `OAuthAccessTokenRepository`.
#[derive(Default)]
pub(crate) struct MockOAuthAccessTokenRepository {
    inner: Arc<Mutex<HashMap<String, OAuthAccessToken>>>,
}

impl MockOAuthAccessTokenRepository {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Count of revoked access tokens (test assertion helper).
    pub(crate) fn revoked_count(&self) -> usize {
        let map = self.inner.lock().expect("poisoned");
        map.values().filter(|t| t.revoked_at.is_some()).count()
    }
}

#[async_trait]
impl OAuthAccessTokenRepository for MockOAuthAccessTokenRepository {
    async fn find_by_token_hash(
        &self,
        _tenant_id: Uuid,
        token_hash: &str,
    ) -> Result<Option<OAuthAccessToken>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        Ok(map.get(token_hash).cloned())
    }

    async fn revoke(
        &self,
        _tenant_id: Uuid,
        token_hash: &str,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        if let Some(t) = map.get_mut(token_hash) {
            t.revoked_at = Some(revoked_at);
        }
        // Per the real revoke handler, missing tokens are silently ignored.
        Ok(())
    }

    async fn insert(&self, token: &OAuthAccessToken) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        map.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }
}

// ============================================
// Mock OAuthRefreshTokenRepository
// ============================================

/// In-memory mock for `OAuthRefreshTokenRepository`.
#[derive(Default)]
pub(crate) struct MockOAuthRefreshTokenRepository {
    inner: Arc<Mutex<HashMap<String, OAuthRefreshToken>>>,
}

impl MockOAuthRefreshTokenRepository {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl OAuthRefreshTokenRepository for MockOAuthRefreshTokenRepository {
    async fn find_by_token_hash(
        &self,
        _tenant_id: Uuid,
        token_hash: &str,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        Ok(map.get(token_hash).cloned())
    }

    async fn revoke(
        &self,
        _tenant_id: Uuid,
        token_hash: &str,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        if let Some(t) = map.get_mut(token_hash) {
            t.revoked_at = Some(revoked_at);
        }
        Ok(())
    }

    async fn insert(&self, token: &OAuthRefreshToken) -> Result<(), PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        map.insert(token.token_hash.clone(), token.clone());
        Ok(())
    }
}

// ============================================
// Mock fixtures
// ============================================

/// Holds mock repos so a test can pre-populate them.
pub(crate) struct MockOAuth2Bundles {
    pub client_repo: Arc<MockOAuthClientRepository>,
    pub auth_code_repo: Arc<MockOAuthAuthorizationCodeRepository>,
    pub access_token_repo: Arc<MockOAuthAccessTokenRepository>,
    pub refresh_token_repo: Arc<MockOAuthRefreshTokenRepository>,
}

impl MockOAuth2Bundles {
    pub(crate) fn new() -> Self {
        Self {
            client_repo: Arc::new(MockOAuthClientRepository::new()),
            auth_code_repo: Arc::new(MockOAuthAuthorizationCodeRepository::new()),
            access_token_repo: Arc::new(MockOAuthAccessTokenRepository::new()),
            refresh_token_repo: Arc::new(MockOAuthRefreshTokenRepository::new()),
        }
    }

    /// Build a `OAuth2State` whose 4 repos are the mocks, and whose
    /// `key_manager` + `jwt_config` use a fresh RSA 2048 keypair.
    ///
    /// 守门 #5 v2: never logs the private key.
    pub(crate) async fn build_state(&self) -> OAuth2State {
        let key_manager = Arc::new(OAuthKeyManager::new().expect("key_manager"));
        let private_pem = key_manager.get().await.private_pem().to_string();
        let public_pem = key_manager.get().await.public_pem().to_string();
        let jwt_config = Arc::new(JwtConfig {
            private_key_pem: private_pem,
            public_key_pem: public_pem,
            issuer: "https://test.star.local".to_string(),
            audience: "star-api-rest".to_string(),
            ttl_seconds: 3600,
        });

        OAuth2State {
            key_manager,
            jwt_config,
            client_repo: self.client_repo.clone(),
            auth_code_repo: self.auth_code_repo.clone(),
            access_token_repo: self.access_token_repo.clone(),
            refresh_token_repo: self.refresh_token_repo.clone(),
        }
    }

    /// Pre-populate a `public` client with PKCE required and a single
    /// redirect_uri.  Returns the registered `client_id`.
    pub(crate) fn seed_public_client(&self, client_id: &str, redirect_uri: &str) -> OAuthClient {
        let now = Utc::now();
        let client = OAuthClient {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            client_id: client_id.to_string(),
            client_secret_hash: None,
            client_name: format!("Test Client {}", client_id),
            client_type: "public".to_string(),
            redirect_uris: vec![redirect_uri.to_string()],
            allowed_scopes: vec!["read".to_string(), "write".to_string()],
            allowed_grant_types: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            require_pkce: true,
            require_authentication: false,
            owner_user_id: Uuid::nil(),
            valid_from: now,
            valid_to: None,
            created_at: now,
            updated_at: now,
            source_module: "star_api_rest::tests::common".to_string(),
            source_kind: "master".to_string(),
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
        self.client_repo.insert(client.clone());
        client
    }

    /// Pre-populate an authorization code row that can later be exchanged
    /// for tokens.  `code_hash` should be `sha256_hex(the_plaintext_code)`.
    pub(crate) fn seed_authorization_code(
        &self,
        client_id: &str,
        code_hash: &str,
        code_challenge: &str,
        code_challenge_method: &str,
        redirect_uri: &str,
        scope: &str,
    ) -> OAuthAuthorizationCode {
        let now = Utc::now();
        let code = OAuthAuthorizationCode {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            code_hash: code_hash.to_string(),
            client_id: client_id.to_string(),
            user_id: Uuid::nil(),
            redirect_uri: redirect_uri.to_string(),
            scope: scope.to_string(),
            code_challenge: code_challenge.to_string(),
            code_challenge_method: code_challenge_method.to_string(),
            expires_at: now + chrono::Duration::minutes(10),
            consumed_at: None,
            created_at: now,
            source_module: "star_api_rest::tests::common".to_string(),
            source_kind: "audit".to_string(),
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
        self.auth_code_repo.insert(code.clone());
        code
    }
}

impl Default for MockOAuth2Bundles {
    fn default() -> Self {
        Self::new()
    }
}
