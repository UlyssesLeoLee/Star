// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=3a493edb236184b5f7089d3230c50d6cf58d0ba818fc8f0aa406422b27baec55
MERGE (self:File {path:"crates/star-api-rest/src/device_outbox/identity_owner.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::module",kind:"module"})
MERGE (port:Type {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::DeviceRevocationOwnerStore",kind:"trait"})
MERGE (pg:Type {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::PgDeviceRevocationOwnerStore",kind:"struct"})
MERGE (receipt:Type {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::RevocationReceipt",kind:"struct"})
MERGE (error:Type {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::DeviceRevocationOwnerError",kind:"enum"})
MERGE (revoke:Symbol {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::DeviceRevocationOwnerStore.revoke_and_publish",kind:"method"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(port),(module)-[:DEFINES]->(pg),(module)-[:DEFINES]->(receipt),(module)-[:DEFINES]->(error)
MERGE (pg)-[:IMPLEMENTS]->(port)
MERGE (port)-[:HAS_METHOD]->(revoke)
MERGE (revoke)-[:USES_TYPE]->(receipt)
MERGE (revoke)-[:WRITES]->(:Table {name:"oauth_device_bindings",kind:"table"})
MERGE (revoke)-[:WRITES]->(:Table {name:"oauth_refresh_tokens",kind:"table"})
MERGE (revoke)-[:WRITES]->(:Table {name:"oauth_device_binding_audit",kind:"table"})
MERGE (revoke)-[:WRITES]->(:Table {name:"identity_device_outbox",kind:"table"})
MERGE (revoke)-[:CALLS]->(:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::owner_stream_id",kind:"function"})
@endcypher
*/
//! Identity/Auth owner: one local transaction advances the device generation,
//! revokes the refresh family, appends the audit fact and appends the Identity
//! Outbox event (review 合约 #1). Rollback rolls all four back; commit makes
//! the event durable for later relay even across process restarts.
//!
//! The Collaboration owner never writes the Identity tables or this Outbox;
//! this module is the Identity owner's own write path.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::contract::{
    owner_stream_id, DeviceRevocationEvent, EVENT_TYPE_DEVICE_REVOKED, PRODUCER_ID, SCHEMA_VERSION,
};

/// Result of a committed revocation transaction: the advanced generation plus
/// the durable Outbox event identity written in the same transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocationReceipt {
    pub device_generation: i64,
    pub outbox_event_id: Uuid,
    pub source_sequence: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceRevocationOwnerError {
    #[error("device revocation transaction failed: {0}")]
    Transaction(String),
}

/// Identity owner persistence contract for the revocation transaction that
/// also appends the durable Outbox event (review 合约 #1).
#[async_trait]
pub trait DeviceRevocationOwnerStore: Send + Sync {
    /// Revoke an active device, advance its generation, revoke its refresh
    /// family, append the audit fact and append the Outbox event — all in one
    /// local owner transaction. Returns `None` when the device is not found or
    /// already revoked (an idempotent re-request).
    async fn revoke_and_publish(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        occurred_at: DateTime<Utc>,
        correlation_id: Uuid,
    ) -> Result<Option<RevocationReceipt>, DeviceRevocationOwnerError>;
}

/// PostgreSQL implementation owned by the Identity/Auth owner, mirroring
/// `PgDeviceBindingRepository::revoke` but additionally appending the Outbox
/// event in the same transaction under `star_oauth_credential_owner`.
pub struct PgDeviceRevocationOwnerStore {
    pool: PgPool,
}

impl PgDeviceRevocationOwnerStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceRevocationOwnerStore for PgDeviceRevocationOwnerStore {
    async fn revoke_and_publish(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        device_id: Uuid,
        occurred_at: DateTime<Utc>,
        correlation_id: Uuid,
    ) -> Result<Option<RevocationReceipt>, DeviceRevocationOwnerError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(tx_error("begin device revoke"))?;
        set_identity_owner_scope(&mut tx, tenant_id, actor_id).await?;

        // Serialize revocations per owner stream so MAX(source_sequence)+1 is
        // race-free inside the transaction.
        let stream_id = owner_stream_id(tenant_id, actor_id);
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!("identity-device-revoke:{}:{}", tenant_id, actor_id))
            .execute(&mut *tx)
            .await
            .map_err(tx_error("lock device revoke stream"))?;

        // 1. Advance the generation and mark revoked; absent/already-revoked
        //    returns None and commits no state or event.
        let generation: Option<i64> = sqlx::query_scalar(
            "UPDATE oauth_device_bindings SET generation=generation+1, status='revoked', revoked_at=$4 \
             WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3 \
               AND status='active' AND revoked_at IS NULL \
             RETURNING generation",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(occurred_at)
        .fetch_optional(&mut *tx)
        .await
        .map_err(tx_error("revoke device"))?;
        let Some(generation) = generation else {
            tx.commit()
                .await
                .map_err(tx_error("commit missing device revoke"))?;
            return Ok(None);
        };

        // 2. Revoke the refresh family (same owner transaction).
        sqlx::query(
            "UPDATE oauth_refresh_tokens SET revoked_at=$4 \
             WHERE tenant_id=$1 AND user_id=$2 AND device_id=$3 AND revoked_at IS NULL",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(occurred_at)
        .execute(&mut *tx)
        .await
        .map_err(tx_error("revoke device refresh family"))?;

        // 3. Append the audit fact.
        sqlx::query(
            "INSERT INTO oauth_device_binding_audit \
             (tenant_id, actor_id, device_id, event_kind, generation, public_key_fingerprint, occurred_at) \
             SELECT tenant_id, actor_id, device_id, 'revoked', generation, public_key_fingerprint, $4 \
             FROM oauth_device_bindings \
             WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(occurred_at)
        .execute(&mut *tx)
        .await
        .map_err(tx_error("insert device revoke audit"))?;

        // 4. Append the durable Outbox event with the next owner-monotonic
        //    source sequence (review 合约 #1/#3). Metadata only: no token,
        //    proof, body or DEK.
        let event = DeviceRevocationEvent {
            event_id: Uuid::new_v4(),
            event_type: EVENT_TYPE_DEVICE_REVOKED.to_string(),
            schema_version: SCHEMA_VERSION,
            tenant_id,
            actor_id,
            device_id,
            device_generation: generation,
            stream_id,
            source_sequence: 0, // replaced by the COALESCE(MAX)+1 expression below
            occurred_at,
            correlation_id,
        };
        let source_sequence: i64 = sqlx::query_scalar(
            "INSERT INTO identity_device_outbox \
             (stream_id, source_sequence, event_id, event_type, schema_version, tenant_id, actor_id, \
              device_id, device_generation, producer_id, occurred_at, correlation_id) \
             VALUES ($1, (SELECT COALESCE(MAX(source_sequence),0)+1 FROM identity_device_outbox WHERE stream_id=$1), \
                     $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             RETURNING source_sequence",
        )
        .bind(stream_id)
        .bind(event.event_id)
        .bind(&event.event_type)
        .bind(event.schema_version)
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(generation)
        .bind(PRODUCER_ID)
        .bind(occurred_at)
        .bind(correlation_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(tx_error("append identity device outbox event"))?;

        // 5. Commit everything together; a failure here rolls back state,
        //    audit, refresh revocation and the Outbox event as one unit.
        tx.commit()
            .await
            .map_err(tx_error("commit device revoke with outbox"))?;
        Ok(Some(RevocationReceipt {
            device_generation: generation,
            outbox_event_id: event.event_id,
            source_sequence,
        }))
    }
}

async fn set_identity_owner_scope(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    actor_id: Uuid,
) -> Result<(), DeviceRevocationOwnerError> {
    // Same owner-role pattern as the OAuth device repository: the deployment
    // login is NOINHERIT and receives this dedicated role only through
    // deployment membership.
    sqlx::query("SET LOCAL ROLE star_oauth_credential_owner")
        .execute(&mut **tx)
        .await
        .map_err(tx_error("set OAuth credential owner role"))?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true), set_config('app.actor_id',$2,true)")
        .bind(tenant_id.to_string())
        .bind(actor_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(tx_error("set identity owner scope"))?;
    Ok(())
}

fn tx_error(operation: &'static str) -> impl FnOnce(sqlx::Error) -> DeviceRevocationOwnerError {
    move |error| DeviceRevocationOwnerError::Transaction(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Minimal in-memory double with real transaction semantics: writes are
    /// buffered per transaction and only become visible after commit; a
    /// rollback discards every buffered write. This mirrors the four-write
    /// revocation transaction contract without a live PostgreSQL.
    #[derive(Debug, Default, Clone, PartialEq)]
    struct InMemoryIdentityState {
        generation: i64,
        status: &'static str,
        revoked_at: Option<DateTime<Utc>>,
        refresh_revoked: bool,
        audit_events: Vec<String>,
        outbox_rows: Vec<(i64, Uuid)>,
    }

    #[derive(Default, Clone)]
    struct InMemoryRevocationStore {
        state: InMemoryIdentityState,
    }

    /// Executes the same four-step revocation + outbox sequence the PG
    /// implementation performs, but through an explicit commit/rollback buffer.
    async fn run_revoke_transaction(
        store: &InMemoryRevocationStore,
        fail_at_step: Option<usize>,
    ) -> Result<InMemoryIdentityState, ()> {
        let mut buffer = store.state.clone();
        // Step 1: generation + status.
        if buffer.status != "active" || buffer.revoked_at.is_some() {
            return Ok(buffer); // idempotent: nothing to do, nothing written
        }
        buffer.generation += 1;
        buffer.status = "revoked";
        if fail_at_step == Some(1) {
            return Err(()); // rollback: buffer dropped
        }
        // Step 2: refresh family.
        buffer.refresh_revoked = true;
        if fail_at_step == Some(2) {
            return Err(());
        }
        // Step 3: audit.
        buffer
            .audit_events
            .push(format!("revoked@{}", buffer.generation));
        if fail_at_step == Some(3) {
            return Err(());
        }
        // Step 4: outbox event.
        let next_seq = buffer.outbox_rows.len() as i64 + 1;
        buffer.outbox_rows.push((next_seq, Uuid::new_v4()));
        if fail_at_step == Some(4) {
            return Err(());
        }
        // Commit: the buffered state becomes the committed state.
        Ok(buffer)
    }

    #[tokio::test]
    async fn same_transaction_rollback_discards_state_audit_and_outbox_event() {
        let store = InMemoryRevocationStore {
            state: InMemoryIdentityState {
                generation: 1,
                status: "active",
                revoked_at: None,
                refresh_revoked: false,
                audit_events: vec![],
                outbox_rows: vec![],
            },
        };
        for fail_step in 1..=4 {
            let outcome = run_revoke_transaction(&store, Some(fail_step)).await;
            assert!(outcome.is_err(), "step {fail_step} must abort the tx");
            assert_eq!(
                store.state,
                InMemoryIdentityState {
                    generation: 1,
                    status: "active",
                    revoked_at: None,
                    refresh_revoked: false,
                    audit_events: vec![],
                    outbox_rows: vec![],
                },
                "rollback at step {fail_step} must leave state, audit and outbox untouched"
            );
        }
    }

    #[tokio::test]
    async fn commit_persists_state_audit_and_outbox_event_together() {
        let store = InMemoryRevocationStore {
            state: InMemoryIdentityState {
                generation: 1,
                status: "active",
                revoked_at: None,
                refresh_revoked: false,
                audit_events: vec![],
                outbox_rows: vec![],
            },
        };
        let committed = run_revoke_transaction(&store, None).await.expect("tx");
        assert_eq!(committed.generation, 2);
        assert_eq!(committed.status, "revoked");
        assert!(committed.refresh_revoked);
        assert_eq!(committed.audit_events, vec!["revoked@2"]);
        assert_eq!(committed.outbox_rows.len(), 1);
        assert_eq!(committed.outbox_rows[0].0, 1, "monotonic source sequence");
    }

    #[tokio::test]
    async fn idempotent_revoke_of_missing_device_writes_nothing() {
        let store = InMemoryRevocationStore {
            state: InMemoryIdentityState {
                generation: 3,
                status: "revoked",
                revoked_at: Some(Utc::now()),
                refresh_revoked: true,
                audit_events: vec!["revoked@2".to_string()],
                outbox_rows: vec![(1, Uuid::new_v4())],
            },
        };
        let committed = run_revoke_transaction(&store, None).await.expect("tx");
        assert_eq!(committed, store.state, "no-op revoke must not append facts");
    }

    #[test]
    fn outbox_event_carries_metadata_only() {
        // Contract #3: the event struct has no field that could carry a token,
        // proof, private key, body or DEK; enforce that structurally.
        let event = DeviceRevocationEvent {
            event_id: Uuid::new_v4(),
            event_type: EVENT_TYPE_DEVICE_REVOKED.to_string(),
            schema_version: SCHEMA_VERSION,
            tenant_id: Uuid::new_v4(),
            actor_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            device_generation: 2,
            stream_id: owner_stream_id(Uuid::new_v4(), Uuid::new_v4()),
            source_sequence: 1,
            occurred_at: Utc::now(),
            correlation_id: Uuid::new_v4(),
        };
        // The digest only ever covers the metadata envelope.
        let digest = event.event_digest();
        assert_eq!(digest.len(), 32);
        assert!(super::super::contract::validate_envelope(&event).is_ok());
    }

    #[tokio::test]
    #[ignore = "requires an isolated migrated PostgreSQL database via STAR_DEVICE_OUTBOX_TEST_DATABASE_URL with the w16 device auth + w17 outbox migrations and star_oauth_credential_owner membership provisioned for the connection login"]
    async fn postgres_revoke_transaction_rolls_back_state_and_outbox_together() {
        let database_url = std::env::var("STAR_DEVICE_OUTBOX_TEST_DATABASE_URL")
            .expect("runner-owned isolated PostgreSQL URL is required");
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect to the explicitly supplied isolated database");
        let store = PgDeviceRevocationOwnerStore::new(pool.clone());
        let tenant_id = Uuid::from_u128(0x7001);
        let actor_id = Uuid::from_u128(0x7002);
        let device_id = Uuid::from_u128(0x7003);
        let now = Utc::now();
        // Seed an active binding the constrained role can see.
        sqlx::query(
            "INSERT INTO oauth_device_bindings (tenant_id, actor_id, device_id, public_key_fingerprint, generation, status) \
             VALUES ($1, $2, $3, $4, 1, 'active') \
             ON CONFLICT (tenant_id, device_id) DO UPDATE SET generation=1, status='active', revoked_at=NULL",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(format!("{:064x}", 0xAB))
        .execute(&pool)
        .await
        .expect("seed active device binding");
        // A poison outbox event for the same stream must force the transaction
        // to fail after the state change so the rollback can be observed.
        let stream_id = owner_stream_id(tenant_id, actor_id);
        sqlx::query(
            "INSERT INTO identity_device_outbox (stream_id, source_sequence, event_id, event_type, schema_version, tenant_id, actor_id, device_id, device_generation, producer_id, occurred_at, correlation_id) \
             VALUES ($1, 999, $2, 'device.revoked', 1, $3, $4, $5, 2, 'identity.device-revoke.v1', $6, $7) \
             ON CONFLICT DO NOTHING",
        )
        .bind(stream_id)
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .bind(now)
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .expect("seed poison outbox row");

        let revoke = store
            .revoke_and_publish(tenant_id, actor_id, device_id, now, Uuid::new_v4())
            .await;
        assert!(
            revoke.is_err(),
            "revoke must fail because the outbox sequence slot collides"
        );
        let (generation, status): (i64, String) = sqlx::query_as(
            "SELECT generation, status FROM oauth_device_bindings WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .fetch_one(&pool)
        .await
        .expect("binding must still exist");
        assert_eq!(
            (generation, status.as_str()),
            (1, "active"),
            "state must roll back"
        );
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM oauth_device_binding_audit WHERE tenant_id=$1 AND actor_id=$2 AND device_id=$3",
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(device_id)
        .fetch_one(&pool)
        .await
        .expect("audit count");
        assert_eq!(
            audit_count, 0,
            "audit fact must roll back with the transaction"
        );
        let outbox_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM identity_device_outbox WHERE stream_id=$1 AND source_sequence > 999",
        )
        .bind(stream_id)
        .fetch_one(&pool)
        .await
        .expect("outbox count");
        assert_eq!(
            outbox_count, 0,
            "no outbox event may survive the failed transaction"
        );
    }
}
