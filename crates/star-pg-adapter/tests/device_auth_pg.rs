// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=f88731b16974b03d19826b35eeca8973ed643bf85a51ded194f5141ab6633bc9
MERGE (self:File {path:"crates/star-pg-adapter/tests/device_auth_pg.rs"})
MERGE (test:Symbol {id:"crates/star-pg-adapter/tests/device_auth_pg.rs::postgres_device_proof_generation_and_revocation",kind:"test"})
MERGE (capTest:Symbol {id:"crates/star-pg-adapter/tests/device_auth_pg.rs::postgres_device_challenge_creation_enforces_actor_cap_under_concurrency",kind:"test"})
MERGE (roleTest:Symbol {id:"crates/star-pg-adapter/tests/device_auth_pg.rs::assert_oauth_owner_role_contract",kind:"function"})
MERGE (repo:Type {id:"star_pg_adapter::repository::DeviceBindingRepository",kind:"trait"})
MERGE (test)-[:TESTS]->(repo)
MERGE (capTest)-[:TESTS]->(repo)
MERGE (test)-[:CALLS]->(roleTest),(capTest)-[:CALLS]->(roleTest)
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(capTest)
MERGE (self)-[:DEFINES]->(roleTest)
@endcypher
*/
//! PostgreSQL proof of durable device enrollment, owner scoping, generation checks, and revocation.

use chrono::{Duration, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use star_pg_adapter::repository::{
    DeviceBindingRepository, DeviceChallengeCreateResult, DeviceEnrollmentChallenge,
    PgDeviceBindingRepository, MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn postgres_device_proof_generation_and_revocation() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .expect("dedicated W16 test PostgreSQL endpoint");
    assert_oauth_owner_role_contract(&pool).await;
    let repository = Arc::new(PgDeviceBindingRepository::new(pool.clone()));
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let challenge_id = Uuid::new_v4();
    let nonce_hash = [0x42_u8; 32];
    let public_key = [0x17_u8; 32];
    let fingerprint = hex::encode(sha256(&public_key));
    let now = Utc::now();
    let challenge = DeviceEnrollmentChallenge {
        challenge_id,
        tenant_id,
        actor_id,
        challenge_hash: nonce_hash.to_vec(),
        public_key: public_key.to_vec(),
        public_key_fingerprint: fingerprint,
        expires_at: now + Duration::minutes(5),
    };
    repository
        .create_challenge(&challenge)
        .await
        .expect("create owner-scoped challenge");

    assert!(repository
        .find_challenge(Uuid::new_v4(), actor_id, challenge_id)
        .await
        .expect("cross-tenant challenge read")
        .is_none());

    let loaded = repository
        .find_challenge(tenant_id, actor_id, challenge_id)
        .await
        .expect("find challenge")
        .expect("challenge exists");
    assert_eq!(loaded.public_key, public_key);
    assert_eq!(loaded.challenge_hash, nonce_hash);

    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..2 {
        let repository = Arc::clone(&repository);
        tasks.spawn(async move {
            repository
                .complete_challenge(
                    tenant_id,
                    actor_id,
                    challenge_id,
                    &nonce_hash,
                    Uuid::new_v4(),
                    Utc::now(),
                )
                .await
                .expect("complete challenge transaction")
        });
    }
    let mut winner = None;
    let mut winners = 0;
    while let Some(result) = tasks.join_next().await {
        if let Some(binding) = result.expect("task result") {
            winners += 1;
            winner = Some(binding);
        }
    }
    assert_eq!(winners, 1, "one challenge must activate at most one device");
    let binding = winner.expect("activated binding");
    assert_eq!(binding.generation, 1);
    assert!(repository
        .is_active(tenant_id, actor_id, binding.device_id, 1)
        .await
        .expect("active generation check"));

    insert_refresh_fixture(&pool, tenant_id, actor_id, binding.device_id).await;
    let revoked = repository
        .revoke(tenant_id, actor_id, binding.device_id, Utc::now())
        .await
        .expect("revoke device transaction")
        .expect("device was active");
    assert_eq!(revoked.generation, 2);
    assert_eq!(revoked.status, "revoked");
    assert!(!repository
        .is_active(tenant_id, actor_id, binding.device_id, 1)
        .await
        .expect("revoked generation check"));
    assert!(refresh_fixture_is_revoked(&pool, tenant_id, actor_id, binding.device_id).await);
    pool.close().await;
}

#[tokio::test]
#[ignore = "requires an isolated migrated PostgreSQL database via STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL"]
async fn postgres_device_challenge_creation_enforces_actor_cap_under_concurrency() {
    let database_url = std::env::var("STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL")
        .expect("ignored PostgreSQL test requires its isolated database URL");
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("dedicated W16 test PostgreSQL endpoint");
    assert_oauth_owner_role_contract(&pool).await;
    let repository = Arc::new(PgDeviceBindingRepository::new(pool.clone()));
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let mut tasks = tokio::task::JoinSet::new();

    for _ in 0..12 {
        let repository = Arc::clone(&repository);
        let public_key = [0x17_u8; 32];
        let challenge = DeviceEnrollmentChallenge {
            challenge_id: Uuid::new_v4(),
            tenant_id,
            actor_id,
            challenge_hash: vec![0x42; 32],
            public_key: public_key.to_vec(),
            public_key_fingerprint: hex::encode(sha256(&public_key)),
            expires_at: Utc::now() + Duration::minutes(5),
        };
        tasks.spawn(async move {
            repository
                .create_challenge(&challenge)
                .await
                .expect("concurrent owner-scoped challenge create")
        });
    }

    let mut created = 0_i64;
    let mut rate_limited = 0_i64;
    while let Some(result) = tasks.join_next().await {
        match result.expect("task result") {
            DeviceChallengeCreateResult::Created => created += 1,
            DeviceChallengeCreateResult::RateLimited => rate_limited += 1,
        }
    }
    assert_eq!(created, MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES);
    assert_eq!(rate_limited, 12 - MAX_ACTIVE_DEVICE_ENROLLMENT_CHALLENGES);

    let other_actor_challenge = DeviceEnrollmentChallenge {
        challenge_id: Uuid::new_v4(),
        tenant_id,
        actor_id: Uuid::new_v4(),
        challenge_hash: vec![0x24; 32],
        public_key: vec![0x18; 32],
        public_key_fingerprint: hex::encode(sha256(&[0x18_u8; 32])),
        expires_at: Utc::now() + Duration::minutes(5),
    };
    assert_eq!(
        repository
            .create_challenge(&other_actor_challenge)
            .await
            .expect("another actor has an independent cap"),
        DeviceChallengeCreateResult::Created
    );
    pool.close().await;
}

async fn insert_refresh_fixture(pool: &PgPool, tenant_id: Uuid, actor_id: Uuid, device_id: Uuid) {
    let mut tx = pool.begin().await.expect("begin refresh fixture");
    set_owner_scope(&mut tx, tenant_id, actor_id).await;
    sqlx::query(
        "INSERT INTO oauth_refresh_tokens \
         (tenant_id,token_hash,client_id,user_id,access_token_id,scope,expires_at,device_id,device_generation) \
         VALUES ($1,$2,'w16-pg-fixture',$3,$4,'canvas:read',now()+interval '1 day',$5,1)",
    )
    .bind(tenant_id)
    .bind(format!("w16-refresh-fixture-{}", Uuid::new_v4()))
    .bind(actor_id)
    .bind(Uuid::new_v4())
    .bind(device_id)
    .execute(&mut *tx)
    .await
    .expect("insert device-bound refresh fixture");
    tx.commit().await.expect("commit refresh fixture");
}

async fn refresh_fixture_is_revoked(
    pool: &PgPool,
    tenant_id: Uuid,
    actor_id: Uuid,
    device_id: Uuid,
) -> bool {
    let mut tx = pool.begin().await.expect("begin refresh assertion");
    set_owner_scope(&mut tx, tenant_id, actor_id).await;
    let revoked = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM oauth_refresh_tokens \
         WHERE tenant_id=$1 AND user_id=$2 AND device_id=$3 AND revoked_at IS NOT NULL)",
    )
    .bind(tenant_id)
    .bind(actor_id)
    .bind(device_id)
    .fetch_one(&mut *tx)
    .await
    .expect("query refresh revocation");
    tx.commit().await.expect("commit refresh assertion");
    revoked
}

async fn set_owner_scope(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
) {
    sqlx::query("SET LOCAL ROLE star_oauth_credential_owner")
        .execute(&mut **tx)
        .await
        .expect("test login can explicitly assume the OAuth owner role");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(actor_id.to_string())
        .execute(&mut **tx)
        .await
        .expect("set database owner scope");
}

async fn assert_oauth_owner_role_contract(pool: &PgPool) {
    let mut tx = pool.begin().await.expect("begin OAuth owner role probe");
    sqlx::query("SET LOCAL ROLE star_oauth_credential_owner")
        .execute(&mut *tx)
        .await
        .expect("NOINHERIT test login can explicitly assume OAuth owner role");
    let (current_role, owner_super, owner_bypass_rls, owner_login, owner_inherit, login_inherit) =
        sqlx::query_as::<_, (String, bool, bool, bool, bool, bool)>(
            "SELECT current_user::text, owner.rolsuper, owner.rolbypassrls, owner.rolcanlogin, \
             owner.rolinherit, login.rolinherit FROM pg_roles AS owner, pg_roles AS login \
             WHERE owner.rolname=current_user AND login.rolname=session_user",
        )
        .fetch_one(&mut *tx)
        .await
        .expect("read OAuth owner and test login role flags");
    assert_eq!(current_role, "star_oauth_credential_owner");
    assert!(!owner_super && !owner_bypass_rls && !owner_login && !owner_inherit);
    assert!(!login_inherit, "acceptance login must remain NOINHERIT");
    tx.commit().await.expect("commit OAuth owner role probe");
}

fn sha256(value: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(value);
    let mut output = [0; 32];
    output.copy_from_slice(&digest);
    output
}
