// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=7483c0180641ef22a974d760e0b801776ae1524ee211b745e603f1a2f49bf92d
MERGE (self:File {path:"crates/star-api-rest/tests/common/mod.rs"})
MERGE (mockCode:Type {id:"crates/star-api-rest/tests/common/mod.rs::MockOAuthAuthorizationCodeRepository",kind:"struct"})
MERGE (mockRefresh:Type {id:"crates/star-api-rest/tests/common/mod.rs::MockOAuthRefreshTokenRepository",kind:"struct"})
MERGE (mockRefreshConsume:Symbol {id:"crates/star-api-rest/tests/common/mod.rs::MockOAuthRefreshTokenRepository.consume_if_active",kind:"method"})
MERGE (mockDevice:Type {id:"crates/star-api-rest/tests/common/mod.rs::MockDeviceBindingRepository",kind:"struct"})
MERGE (codeRepo:Type {id:"star_pg_adapter::repository::OAuthAuthorizationCodeRepository",kind:"trait"})
MERGE (refreshRepo:Type {id:"star_pg_adapter::repository::OAuthRefreshTokenRepository",kind:"trait"})
MERGE (refreshConsume:Symbol {id:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs::consume_if_active",kind:"method"})
MERGE (refreshToken:Type {id:"crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs::OAuthRefreshToken",kind:"struct"})
MERGE (deviceRepo:Type {id:"star_pg_adapter::repository::DeviceBindingRepository",kind:"trait"})
MERGE (mockCode)-[:IMPLEMENTS]->(codeRepo)
MERGE (mockRefresh)-[:IMPLEMENTS]->(refreshRepo)
MERGE (mockRefresh)-[:DEFINES]->(mockRefreshConsume)
MERGE (mockRefreshConsume)-[:IMPLEMENTS]->(refreshConsume)
MERGE (mockRefreshConsume)-[:USES_TYPE]->(refreshToken)
MERGE (mockRefreshConsume)-[:READS]->(mockRefresh)
MERGE (mockRefreshConsume)-[:WRITES]->(mockRefresh)
MERGE (mockDevice)-[:IMPLEMENTS]->(deviceRepo)
MERGE (self)-[:DEFINES]->(mockCode)
MERGE (self)-[:DEFINES]->(mockRefresh)
MERGE (self)-[:DEFINES]->(mockDevice)
@endcypher
*/
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
    DeviceBinding, DeviceBindingRepository, DeviceChallengeCreateResult, DeviceEnrollmentChallenge,
    OAuthAccessToken, OAuthAccessTokenRepository, OAuthAuthorizationCode,
    OAuthAuthorizationCodeRepository, OAuthClient, OAuthClientRepository, OAuthRefreshToken,
    OAuthRefreshTokenRepository, MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES,
};
use star_pg_adapter::PgAdapterError;

type AccessTokenStore = Arc<Mutex<HashMap<String, OAuthAccessToken>>>;
type RefreshTokenStore = Arc<Mutex<HashMap<String, OAuthRefreshToken>>>;

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
    access_tokens: AccessTokenStore,
    refresh_tokens: RefreshTokenStore,
}

impl MockOAuthAuthorizationCodeRepository {
    pub(crate) fn new() -> Self {
        Self::with_token_stores(
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
        )
    }

    fn with_token_stores(
        access_tokens: AccessTokenStore,
        refresh_tokens: RefreshTokenStore,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            access_tokens,
            refresh_tokens,
        }
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
        code_hash: &str,
        client_id: &str,
    ) -> Result<Option<OAuthAuthorizationCode>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        // Mirror the real impl's behavior: skip consumed or expired codes.
        let now = Utc::now();
        Ok(map.get(code_hash).and_then(|c| {
            if c.consumed_at.is_some() || c.expires_at <= now || c.client_id != client_id {
                None
            } else {
                Some(c.clone())
            }
        }))
    }

    async fn consume_if_active(
        &self,
        _tenant_id: Uuid,
        code_hash: &str,
        consumed_at: DateTime<Utc>,
    ) -> Result<Option<OAuthAuthorizationCode>, PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        let Some(code) = map.get_mut(code_hash) else {
            return Ok(None);
        };
        if code.consumed_at.is_some() || code.expires_at <= consumed_at {
            return Ok(None);
        }
        code.consumed_at = Some(consumed_at);
        Ok(Some(code.clone()))
    }

    async fn consume_and_issue_if_active(
        &self,
        code_hash: &str,
        client_id: &str,
        consumed_at: DateTime<Utc>,
        access_token: &OAuthAccessToken,
        refresh_token: &OAuthRefreshToken,
    ) -> Result<Option<OAuthAuthorizationCode>, PgAdapterError> {
        let mut access_tokens = self.access_tokens.lock().expect("poisoned");
        let mut refresh_tokens = self.refresh_tokens.lock().expect("poisoned");
        let mut codes = self.inner.lock().expect("poisoned");
        let Some(code) = codes.get_mut(code_hash) else {
            return Ok(None);
        };
        if code.consumed_at.is_some()
            || code.expires_at <= consumed_at
            || code.client_id != client_id
            || access_token.tenant_id != code.tenant_id
            || access_token.user_id != Some(code.user_id)
            || access_token.client_id != code.client_id
            || access_token.scope != code.scope
            || access_token.grant_type != "authorization_code"
            || refresh_token.tenant_id != code.tenant_id
            || refresh_token.user_id != code.user_id
            || refresh_token.client_id != code.client_id
            || refresh_token.scope != code.scope
            || refresh_token.access_token_id != access_token.id
        {
            return Ok(None);
        }
        let consumed = code.clone();
        code.consumed_at = Some(consumed_at);
        access_tokens.insert(access_token.token_hash.clone(), access_token.clone());
        refresh_tokens.insert(refresh_token.token_hash.clone(), refresh_token.clone());
        Ok(Some(consumed))
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
    inner: AccessTokenStore,
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
    inner: RefreshTokenStore,
    access_tokens: AccessTokenStore,
}

impl MockOAuthRefreshTokenRepository {
    pub(crate) fn new() -> Self {
        Self::with_access_store(Arc::new(Mutex::new(HashMap::new())))
    }

    fn with_access_store(access_tokens: AccessTokenStore) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            access_tokens,
        }
    }
}

#[async_trait]
impl OAuthRefreshTokenRepository for MockOAuthRefreshTokenRepository {
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
        client_id: &str,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let map = self.inner.lock().expect("poisoned");
        Ok(map.get(token_hash).and_then(|token| {
            (token.revoked_at.is_none()
                && token.expires_at > Utc::now()
                && token.client_id == client_id)
                .then(|| token.clone())
        }))
    }

    async fn consume_if_active(
        &self,
        expected: &OAuthRefreshToken,
        revoked_at: DateTime<Utc>,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let mut map = self.inner.lock().expect("poisoned");
        let Some(token) = map.get_mut(&expected.token_hash) else {
            return Ok(None);
        };
        if token.tenant_id != expected.tenant_id
            || token.client_id != expected.client_id
            || token.user_id != expected.user_id
            || token.scope != expected.scope
            || token.device_id != expected.device_id
            || token.device_generation != expected.device_generation
            || token.revoked_at.is_some()
            || token.expires_at <= revoked_at
        {
            return Ok(None);
        }
        token.revoked_at = Some(revoked_at);
        Ok(Some(token.clone()))
    }

    async fn rotate_if_active(
        &self,
        token_hash: &str,
        client_id: &str,
        rotated_at: DateTime<Utc>,
        access_token: &OAuthAccessToken,
        successor: &OAuthRefreshToken,
    ) -> Result<Option<OAuthRefreshToken>, PgAdapterError> {
        let mut access_tokens = self.access_tokens.lock().expect("poisoned");
        let mut map = self.inner.lock().expect("poisoned");
        let Some(current) = map.get_mut(token_hash) else {
            return Ok(None);
        };
        if current.revoked_at.is_some() || current.expires_at <= rotated_at {
            return Ok(None);
        }
        if current.client_id != client_id
            || current.tenant_id != successor.tenant_id
            || current.client_id != successor.client_id
            || current.user_id != successor.user_id
            || current.scope != successor.scope
            || current.device_id != successor.device_id
            || current.device_generation != successor.device_generation
            || access_token.tenant_id != current.tenant_id
            || access_token.client_id != current.client_id
            || access_token.user_id != Some(current.user_id)
            || access_token.scope != current.scope
            || successor.access_token_id != access_token.id
        {
            return Ok(None);
        }
        let consumed = current.clone();
        current.revoked_at = Some(rotated_at);
        access_tokens.insert(access_token.token_hash.clone(), access_token.clone());
        map.insert(successor.token_hash.clone(), successor.clone());
        Ok(Some(consumed))
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

#[derive(Default)]
struct MockDeviceBindingState {
    challenges: HashMap<Uuid, (DeviceEnrollmentChallenge, bool)>,
    bindings: HashMap<Uuid, DeviceBinding>,
}

#[derive(Default)]
pub(crate) struct MockDeviceBindingRepository {
    inner: Arc<Mutex<MockDeviceBindingState>>,
}

#[async_trait]
impl DeviceBindingRepository for MockDeviceBindingRepository {
    async fn create_challenge(
        &self,
        challenge: &DeviceEnrollmentChallenge,
    ) -> Result<DeviceChallengeCreateResult, PgAdapterError> {
        let mut state = self.inner.lock().expect("poisoned");
        let now = Utc::now();
        state
            .challenges
            .retain(|_, (existing, consumed)| !*consumed && existing.expires_at > now);
        let active_for_actor = state
            .challenges
            .values()
            .filter(|(existing, _)| {
                existing.tenant_id == challenge.tenant_id && existing.actor_id == challenge.actor_id
            })
            .count() as i64;
        if active_for_actor >= MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES {
            return Ok(DeviceChallengeCreateResult::RateLimited);
        }
        state
            .challenges
            .insert(challenge.challenge_id, (challenge.clone(), false));
        Ok(DeviceChallengeCreateResult::Created)
    }

    async fn find_challenge(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        challenge_id: Uuid,
    ) -> Result<Option<DeviceEnrollmentChallenge>, PgAdapterError> {
        let state = self.inner.lock().expect("poisoned");
        Ok(state
            .challenges
            .get(&challenge_id)
            .and_then(|(challenge, consumed)| {
                (!consumed
                    && challenge.tenant_id == tenant_id
                    && challenge.actor_id == actor_id
                    && challenge.expires_at > Utc::now())
                .then(|| challenge.clone())
            }))
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
        let mut state = self.inner.lock().expect("poisoned");
        let Some((challenge, consumed)) = state.challenges.get_mut(&challenge_id) else {
            return Ok(None);
        };
        if *consumed
            || challenge.tenant_id != tenant_id
            || challenge.actor_id != actor_id
            || challenge.expires_at <= now
            || challenge.challenge_hash.as_slice() != challenge_hash
        {
            return Ok(None);
        }
        *consumed = true;
        let binding = DeviceBinding {
            tenant_id,
            actor_id,
            device_id,
            public_key_fingerprint: challenge.public_key_fingerprint.clone(),
            generation: 1,
            status: "active".to_string(),
            created_at: now,
            revoked_at: None,
        };
        state.bindings.insert(device_id, binding.clone());
        Ok(Some(binding))
    }

    async fn is_active(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        generation: i64,
    ) -> Result<bool, PgAdapterError> {
        let state = self.inner.lock().expect("poisoned");
        Ok(state.bindings.get(&device_id).is_some_and(|binding| {
            binding.tenant_id == tenant_id
                && binding.actor_id == actor_id
                && binding.generation == generation
                && binding.status == "active"
        }))
    }

    async fn revoke(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<DeviceBinding>, PgAdapterError> {
        let mut state = self.inner.lock().expect("poisoned");
        let Some(binding) = state.bindings.get_mut(&device_id) else {
            return Ok(None);
        };
        if binding.tenant_id != tenant_id
            || binding.actor_id != actor_id
            || binding.status != "active"
        {
            return Ok(None);
        }
        binding.generation += 1;
        binding.status = "revoked".to_string();
        binding.revoked_at = Some(now);
        Ok(Some(binding.clone()))
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
        let access_token_repo = Arc::new(MockOAuthAccessTokenRepository::new());
        let refresh_token_repo = Arc::new(MockOAuthRefreshTokenRepository::with_access_store(
            Arc::clone(&access_token_repo.inner),
        ));
        let auth_code_repo = Arc::new(MockOAuthAuthorizationCodeRepository::with_token_stores(
            Arc::clone(&access_token_repo.inner),
            Arc::clone(&refresh_token_repo.inner),
        ));
        Self {
            client_repo: Arc::new(MockOAuthClientRepository::new()),
            auth_code_repo,
            access_token_repo,
            refresh_token_repo,
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
            device_binding_repo: Arc::new(MockDeviceBindingRepository::default()),
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
