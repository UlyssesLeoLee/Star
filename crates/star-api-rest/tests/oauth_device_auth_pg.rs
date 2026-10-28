// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=df982c1143ec4df4c5c2e69c8025c72a3bb29bb9f874d0c072340a070249566e
MERGE (self:File {path:"crates/star-api-rest/tests/oauth_device_auth_pg.rs"})
MERGE (challengeTest:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::mock_device_challenge_creation_enforces_actor_scoped_cap",kind:"test"})
MERGE (challengeFactory:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::device_enrollment_challenge",kind:"function"})
MERGE (codeTest:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::mock_auth_code_has_exactly_one_concurrent_winner_and_rejects_replay",kind:"test"})
MERGE (atomicCodeTest:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::mock_auth_code_redemption_commits_one_access_refresh_pair",kind:"test"})
MERGE (refreshTest:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::mock_refresh_token_has_exactly_one_concurrent_winner_and_rejects_replay",kind:"test"})
MERGE (rotationTest:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::mock_refresh_rotation_persists_one_successor_and_rejects_replay",kind:"test"})
MERGE (pgTest:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::postgres_oauth_credentials_have_one_winner_when_test_database_is_configured",kind:"test"})
MERGE (pgRedemption:Symbol {id:"crates/star-api-rest/tests/oauth_device_auth_pg.rs::postgres_auth_code_redemption_persists_tokens_with_consumption",kind:"test"})
MERGE (deviceRepo:Type {id:"star_pg_adapter::repository::DeviceBindingRepository",kind:"trait"})
MERGE (codeRepo:Type {id:"star_pg_adapter::repository::OAuthAuthorizationCodeRepository",kind:"trait"})
MERGE (refreshRepo:Type {id:"star_pg_adapter::repository::OAuthRefreshTokenRepository",kind:"trait"})
MERGE (self)-[:DEFINES]->(challengeTest),(self)-[:DEFINES]->(challengeFactory),(self)-[:DEFINES]->(codeTest),(self)-[:DEFINES]->(atomicCodeTest),(self)-[:DEFINES]->(refreshTest),(self)-[:DEFINES]->(rotationTest),(self)-[:DEFINES]->(pgTest),(self)-[:DEFINES]->(pgRedemption)
MERGE (challengeTest)-[:CALLS]->(challengeFactory),(challengeTest)-[:TESTS]->(deviceRepo),(codeTest)-[:TESTS]->(codeRepo),(atomicCodeTest)-[:TESTS]->(codeRepo),(refreshTest)-[:TESTS]->(refreshRepo),(rotationTest)-[:TESTS]->(refreshRepo),(pgTest)-[:TESTS]->(codeRepo),(pgTest)-[:TESTS]->(refreshRepo),(pgRedemption)-[:TESTS]->(codeRepo)
@endcypher
*/
//! Replay and single-consumer regression checks for OAuth credentials.

mod common;

use chrono::{Duration, Utc};
use common::{MockDeviceBindingRepository, MockOAuth2Bundles};
use star_pg_adapter::repository::{
    DeviceBindingRepository, DeviceChallengeCreateResult, DeviceEnrollmentChallenge,
    MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES, OAuthAccessToken, OAuthAuthorizationCode,
    OAuthAuthorizationCodeRepository, OAuthRefreshToken, OAuthRefreshTokenRepository,
    PgOAuthAuthorizationCodeRepository, PgOAuthRefreshTokenRepository,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn mock_device_challenge_creation_enforces_actor_scoped_cap() {
    let repository = MockDeviceBindingRepository::default();
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    for _ in 0..MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES {
        let challenge = device_enrollment_challenge(tenant_id, actor_id);
        assert_eq!(
            repository
                .create_challenge(&challenge)
                .await
                .expect("create challenge"),
            DeviceChallengeCreateResult::Created
        );
    }

    assert_eq!(
        repository
            .create_challenge(&device_enrollment_challenge(tenant_id, actor_id))
            .await
            .expect("limited challenge result"),
        DeviceChallengeCreateResult::RateLimited
    );

    assert_eq!(
        repository
            .create_challenge(&device_enrollment_challenge(tenant_id, Uuid::new_v4()))
            .await
            .expect("another actor has an independent cap"),
        DeviceChallengeCreateResult::Created
    );
}

fn device_enrollment_challenge(tenant_id: Uuid, actor_id: Uuid) -> DeviceEnrollmentChallenge {
    DeviceEnrollmentChallenge {
        challenge_id: Uuid::new_v4(),
        tenant_id,
        actor_id,
        challenge_hash: vec![0x42; 32],
        public_key: vec![0x17; 32],
        public_key_fingerprint: "a".repeat(64),
        expires_at: Utc::now() + Duration::minutes(5),
    }
}

#[tokio::test]
async fn mock_auth_code_has_exactly_one_concurrent_winner_and_rejects_replay() {
    let bundles = MockOAuth2Bundles::new();
    let code = bundles.seed_authorization_code(
        "single-use-code-hash",
        "device-client",
        "https://example.invalid/callback",
        "challenge",
        "S256",
        "read write",
    );
    let hash = code.code_hash;
    let now = Utc::now();
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..24 {
        let repository = Arc::clone(&bundles.auth_code_repo);
        let hash = hash.clone();
        tasks.spawn(async move {
            repository
                .consume_if_active(Uuid::nil(), &hash, now)
                .await
                .expect("mock consume")
                .is_some()
        });
    }
    let mut winners = 0;
    while let Some(result) = tasks.join_next().await {
        winners += usize::from(result.expect("task result"));
    }
    assert_eq!(winners, 1);
    assert!(
        bundles
            .auth_code_repo
            .consume_if_active(Uuid::nil(), &hash, now + Duration::seconds(1))
            .await
            .expect("replay query")
            .is_none()
    );
}

#[tokio::test]
async fn mock_auth_code_redemption_commits_one_access_refresh_pair() {
    let bundles = MockOAuth2Bundles::new();
    let code = bundles.seed_authorization_code(
        "device-client",
        "atomic-code-hash",
        "challenge",
        "S256",
        "https://example.invalid/callback",
        "canvas:read canvas:write",
    );
    let now = Utc::now();
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..24 {
        let repository = Arc::clone(&bundles.auth_code_repo);
        let code = code.clone();
        tasks.spawn(async move {
            let access = access_token_for_code(&code, Uuid::new_v4());
            let mut refresh = refresh_token_for_tenant(
                code.tenant_id,
                &format!("successor-{}", Uuid::new_v4()),
                now + Duration::days(30),
            );
            refresh.client_id = code.client_id.clone();
            refresh.user_id = code.user_id;
            refresh.access_token_id = access.id;
            repository
                .consume_and_issue_if_active(
                    &code.code_hash,
                    &code.client_id,
                    now,
                    &access,
                    &refresh,
                )
                .await
                .expect("atomic mock authorization-code redemption")
                .is_some()
        });
    }
    assert_eq!(collect_winners(&mut tasks).await, 1);
}

#[tokio::test]
async fn mock_refresh_token_has_exactly_one_concurrent_winner_and_rejects_replay() {
    let bundles = MockOAuth2Bundles::new();
    let now = Utc::now();
    let token = refresh_token("single-use-refresh-hash", now + Duration::days(1));
    bundles
        .refresh_token_repo
        .insert(&token)
        .await
        .expect("seed refresh");
    let hash = token.token_hash;
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..24 {
        let repository = Arc::clone(&bundles.refresh_token_repo);
        let hash = hash.clone();
        tasks.spawn(async move {
            repository
                .consume_if_active(Uuid::nil(), &hash, now)
                .await
                .expect("mock consume")
                .is_some()
        });
    }
    let mut winners = 0;
    while let Some(result) = tasks.join_next().await {
        winners += usize::from(result.expect("task result"));
    }
    assert_eq!(winners, 1);
    assert!(
        bundles
            .refresh_token_repo
            .consume_if_active(Uuid::nil(), &hash, now + Duration::seconds(1))
            .await
            .expect("replay query")
            .is_none()
    );
}

#[tokio::test]
async fn mock_refresh_rotation_persists_one_successor_and_rejects_replay() {
    let bundles = MockOAuth2Bundles::new();
    let now = Utc::now();
    let original = refresh_token("rotate-refresh-hash", now + Duration::days(1));
    bundles
        .refresh_token_repo
        .insert(&original)
        .await
        .expect("seed refresh");

    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..24 {
        let repository = Arc::clone(&bundles.refresh_token_repo);
        let original = original.clone();
        tasks.spawn(async move {
            let access = access_token_for_refresh(&original, Uuid::new_v4());
            let mut successor = original.clone();
            successor.id = Uuid::new_v4();
            successor.token_hash = format!("successor-{}", Uuid::new_v4());
            successor.access_token_id = access.id;
            successor.created_at = now;
            successor.expires_at = now + Duration::days(30);
            successor.revoked_at = None;
            let hash = successor.token_hash.clone();
            let rotated = repository
                .rotate_if_active(
                    &original.token_hash,
                    &original.client_id,
                    now,
                    &access,
                    &successor,
                )
                .await
                .expect("rotate mock refresh");
            rotated.map(|_| hash)
        });
    }
    let mut winning_hashes = Vec::new();
    while let Some(result) = tasks.join_next().await {
        if let Some(hash) = result.expect("task result") {
            winning_hashes.push(hash);
        }
    }
    assert_eq!(winning_hashes.len(), 1);
    assert!(
        bundles
            .refresh_token_repo
            .find_by_token_hash(&winning_hashes[0], &original.client_id)
            .await
            .expect("find persisted successor")
            .is_some()
    );
    assert!(
        bundles
            .refresh_token_repo
            .rotate_if_active(
                &original.token_hash,
                &original.client_id,
                now + Duration::seconds(1),
                &access_token_for_refresh(&original, Uuid::new_v4()),
                &refresh_token("replay-successor", now + Duration::days(1)),
            )
            .await
            .expect("reject replay")
            .is_none()
    );
}

#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn postgres_oauth_credentials_have_one_winner_when_test_database_is_configured() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("dedicated W16 test PostgreSQL endpoint");
    let tenant_id = Uuid::new_v4();
    let code_hash = format!("w16-code-{}", Uuid::new_v4());
    let refresh_hash = format!("w16-refresh-{}", Uuid::new_v4());
    let now = Utc::now();
    let code_repo = Arc::new(PgOAuthAuthorizationCodeRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PgOAuthRefreshTokenRepository::new(pool.clone()));
    let code = authorization_code(tenant_id, &code_hash, now);
    code_repo
        .insert(&code)
        .await
        .expect("insert auth-code fixture");
    let resolved = code_repo
        .find_by_code_hash(&code_hash, &code.client_id)
        .await
        .expect("resolve auth code through owner-scoped locator");
    assert_eq!(resolved.as_ref().map(|row| row.tenant_id), Some(tenant_id));
    assert!(
        code_repo
            .find_by_code_hash(&code_hash, "wrong-client")
            .await
            .expect("reject mismatched client")
            .is_none()
    );
    let refresh = refresh_token_for_tenant(tenant_id, &refresh_hash, now + Duration::days(1));
    refresh_repo
        .insert(&refresh)
        .await
        .expect("insert refresh fixture");

    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..24 {
        let code_repo = Arc::clone(&code_repo);
        let code_hash = code_hash.clone();
        tasks.spawn(async move {
            code_repo
                .consume_if_active(tenant_id, &code_hash, now)
                .await
                .expect("atomic PostgreSQL code consume")
                .is_some()
        });
    }
    let code_winners = collect_winners(&mut tasks).await;
    assert_eq!(code_winners, 1);

    for _ in 0..24 {
        let refresh_repo = Arc::clone(&refresh_repo);
        let refresh_hash = refresh_hash.clone();
        tasks.spawn(async move {
            refresh_repo
                .consume_if_active(tenant_id, &refresh_hash, now)
                .await
                .expect("atomic PostgreSQL refresh consume")
                .is_some()
        });
    }
    let refresh_winners = collect_winners(&mut tasks).await;
    assert_eq!(refresh_winners, 1);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn postgres_auth_code_redemption_persists_tokens_with_consumption() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("dedicated W16 test PostgreSQL endpoint");
    let tenant_id = Uuid::new_v4();
    let now = Utc::now();
    let hash = format!("w16-atomic-code-{}", Uuid::new_v4());
    let repository = PgOAuthAuthorizationCodeRepository::new(pool.clone());
    let code = authorization_code(tenant_id, &hash, now);
    repository.insert(&code).await.expect("insert code fixture");
    let access = access_token_for_code(&code, Uuid::new_v4());
    let mut refresh = refresh_token_for_tenant(
        tenant_id,
        &format!("w16-atomic-refresh-{}", Uuid::new_v4()),
        now + Duration::days(30),
    );
    refresh.user_id = code.user_id;
    refresh.access_token_id = access.id;

    assert!(
        repository
            .consume_and_issue_if_active(&hash, &code.client_id, now, &access, &refresh)
            .await
            .expect("consume and issue token pair in one transaction")
            .is_some()
    );
    assert!(
        repository
            .consume_and_issue_if_active(&hash, &code.client_id, now, &access, &refresh)
            .await
            .expect("reject consumed code replay")
            .is_none()
    );

    let mut tx = pool.begin().await.expect("begin token-row assertion");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(code.user_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set owner RLS scope");
    let access_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM oauth_access_tokens WHERE tenant_id=$1 AND id=$2",
    )
    .bind(tenant_id)
    .bind(access.id)
    .fetch_one(&mut *tx)
    .await
    .expect("count persisted access token");
    let refresh_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM oauth_refresh_tokens WHERE tenant_id=$1 AND token_hash=$2",
    )
    .bind(tenant_id)
    .bind(&refresh.token_hash)
    .fetch_one(&mut *tx)
    .await
    .expect("count persisted refresh token");
    tx.commit().await.expect("commit token-row assertion");
    assert_eq!(access_count, 1);
    assert_eq!(refresh_count, 1);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn postgres_refresh_rotation_commits_one_successor_with_the_consume() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("dedicated W16 test PostgreSQL endpoint");
    let tenant_id = Uuid::new_v4();
    let now = Utc::now();
    let original = refresh_token_for_tenant(
        tenant_id,
        &format!("w16-rotate-{}", Uuid::new_v4()),
        now + Duration::days(1),
    );
    let repository = Arc::new(PgOAuthRefreshTokenRepository::new(pool.clone()));
    repository
        .insert(&original)
        .await
        .expect("insert original refresh");

    let mut candidates = Vec::new();
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..24 {
        let repository = Arc::clone(&repository);
        let original = original.clone();
        let access_id = Uuid::new_v4();
        let access = access_token_for_refresh(&original, access_id);
        let mut successor = original.clone();
        successor.id = Uuid::new_v4();
        successor.token_hash = format!("w16-successor-{}", Uuid::new_v4());
        successor.access_token_id = access_id;
        successor.created_at = now;
        successor.expires_at = now + Duration::days(30);
        successor.revoked_at = None;
        candidates.push((successor.token_hash.clone(), access_id));
        tasks.spawn(async move {
            repository
                .rotate_if_active(
                    &original.token_hash,
                    &original.client_id,
                    now,
                    &access,
                    &successor,
                )
                .await
                .expect("atomic PostgreSQL refresh rotation")
                .is_some()
        });
    }
    assert_eq!(collect_winners(&mut tasks).await, 1);

    let mut tx = pool.begin().await.expect("begin successor assertion");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(original.user_id.to_string())
        .execute(&mut *tx)
        .await
        .expect("set owner RLS scope");
    let hashes = candidates
        .iter()
        .map(|(hash, _)| hash.clone())
        .collect::<Vec<_>>();
    let access_ids = candidates.iter().map(|(_, id)| *id).collect::<Vec<_>>();
    let successor_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM oauth_refresh_tokens WHERE tenant_id=$1 AND token_hash=ANY($2)",
    )
    .bind(tenant_id)
    .bind(hashes)
    .fetch_one(&mut *tx)
    .await
    .expect("count persisted successors");
    let access_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM oauth_access_tokens WHERE tenant_id=$1 AND id=ANY($2)",
    )
    .bind(tenant_id)
    .bind(access_ids)
    .fetch_one(&mut *tx)
    .await
    .expect("count persisted access successors");
    tx.commit().await.expect("commit successor assertion");
    assert_eq!(successor_count, 1);
    assert_eq!(access_count, 1);
    pool.close().await;
}

async fn collect_winners(tasks: &mut tokio::task::JoinSet<bool>) -> usize {
    let mut winners = 0;
    while let Some(result) = tasks.join_next().await {
        winners += usize::from(result.expect("task result"));
    }
    winners
}

fn authorization_code(
    tenant_id: Uuid,
    code_hash: &str,
    now: chrono::DateTime<Utc>,
) -> OAuthAuthorizationCode {
    OAuthAuthorizationCode {
        id: Uuid::new_v4(),
        tenant_id,
        code_hash: code_hash.to_string(),
        client_id: "w16-device-client".to_string(),
        user_id: Uuid::new_v4(),
        redirect_uri: "https://example.invalid/callback".to_string(),
        scope: "canvas:read canvas:write".to_string(),
        code_challenge: "test-challenge".to_string(),
        code_challenge_method: "S256".to_string(),
        expires_at: now + Duration::minutes(10),
        consumed_at: None,
        created_at: now,
        source_module: "star_api_rest::tests::w16".to_string(),
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
    }
}

fn refresh_token(token_hash: &str, expires_at: chrono::DateTime<Utc>) -> OAuthRefreshToken {
    refresh_token_for_tenant(Uuid::nil(), token_hash, expires_at)
}

fn refresh_token_for_tenant(
    tenant_id: Uuid,
    token_hash: &str,
    expires_at: chrono::DateTime<Utc>,
) -> OAuthRefreshToken {
    OAuthRefreshToken {
        id: Uuid::new_v4(),
        tenant_id,
        token_hash: token_hash.to_string(),
        client_id: "w16-device-client".to_string(),
        user_id: Uuid::new_v4(),
        device_id: None,
        device_generation: None,
        access_token_id: Uuid::new_v4(),
        scope: "canvas:read canvas:write".to_string(),
        expires_at,
        revoked_at: None,
        created_at: Utc::now(),
        source_module: "star_api_rest::tests::w16".to_string(),
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
    }
}

fn access_token_for_refresh(refresh: &OAuthRefreshToken, id: Uuid) -> OAuthAccessToken {
    OAuthAccessToken {
        id,
        tenant_id: refresh.tenant_id,
        token_hash: format!("access-{}", Uuid::new_v4()),
        client_id: refresh.client_id.clone(),
        user_id: Some(refresh.user_id),
        grant_type: "refresh_token".to_string(),
        scope: refresh.scope.clone(),
        expires_at: Utc::now() + Duration::hours(1),
        revoked_at: None,
        created_at: Utc::now(),
        source_module: "star_api_rest::tests::w16".to_string(),
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
    }
}

fn access_token_for_code(code: &OAuthAuthorizationCode, id: Uuid) -> OAuthAccessToken {
    OAuthAccessToken {
        id,
        tenant_id: code.tenant_id,
        token_hash: format!("access-{}", Uuid::new_v4()),
        client_id: code.client_id.clone(),
        user_id: Some(code.user_id),
        grant_type: "authorization_code".to_string(),
        scope: code.scope.clone(),
        expires_at: Utc::now() + Duration::hours(1),
        revoked_at: None,
        created_at: Utc::now(),
        source_module: "star_api_rest::tests::w16".to_string(),
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
    }
}
