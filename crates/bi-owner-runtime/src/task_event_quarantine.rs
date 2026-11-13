// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=8a21c76ab5b2542635439960f7ead2777f6abcf5eca99f783dea562083e64595
MERGE (self:File {path:"crates/bi-owner-runtime/src/task_event_quarantine.rs"})
MERGE (store:Type {id:"bi_owner_runtime::TaskEventQuarantineStore",kind:"struct"})
MERGE (constructor:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::TaskEventQuarantineStore.new",kind:"method"})
MERGE (record:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::TaskEventQuarantineStore.record",kind:"method"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQuarantinePort",kind:"trait"})
MERGE (route:Type {id:"bi_owner_runtime::TaskEventReceiveRoute",kind:"struct"})
MERGE (origin:Type {id:"bi_owner_runtime::TaskEventReceiveOrigin",kind:"type_alias"})
MERGE (fault:Type {id:"bi_owner_runtime::TaskEventQuarantineFault",kind:"enum"})
MERGE (receipt:Type {id:"bi_owner_runtime::TaskEventQuarantineReceipt",kind:"struct"})
MERGE (error:Type {id:"bi_owner_runtime::TaskEventReceiveError",kind:"enum"})
MERGE (pool:Type {id:"sqlx::PgPool",kind:"struct"})
MERGE (row:Type {id:"sqlx::Row",kind:"trait"})
MERGE (function:Database {id:"bi.record_task_event_quarantine_v2",kind:"stored_function"})
MERGE (table:Table {id:"bi.task_event_quarantine",kind:"table"})
MERGE (cursor:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventSourceOrigin.cursor",kind:"method"})
MERGE (source_sequence:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventSourceOrigin.source_sequence",kind:"method"})
MERGE (matches_pin:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::source_origin_matches_pin",kind:"function"})
MERGE (route_origin_test:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::tests.quarantine_origin_must_match_provider_log_and_generation",kind:"test"})
MERGE (quarantine_identity_test:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::tests.quarantine_migration_preserves_sequence_conflicts_and_exact_receipts",kind:"test"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(matches_pin)
MERGE (self)-[:IMPORTS]->(pool)
MERGE (self)-[:IMPORTS]->(row)
MERGE (store)-[:DEFINES]->(constructor)
MERGE (store)-[:IMPLEMENTS]->(port)
MERGE (record)-[:USES_TYPE]->(route)
MERGE (record)-[:USES_TYPE]->(origin)
MERGE (record)-[:USES_TYPE]->(fault)
MERGE (record)-[:USES_TYPE]->(receipt)
MERGE (record)-[:USES_TYPE]->(error)
MERGE (record)-[:CALLS]->(matches_pin)
MERGE (record)-[:CALLS]->(function)
MERGE (record)-[:CALLS]->(cursor)
MERGE (record)-[:CALLS]->(source_sequence)
MERGE (self)-[:DEFINES]->(route_origin_test)
MERGE (route_origin_test)-[:TESTS]->(matches_pin)
MERGE (self)-[:DEFINES]->(quarantine_identity_test)
MERGE (quarantine_identity_test)-[:TESTS]->(function)
MERGE (quarantine_identity_test)-[:TESTS]->(table)
@endcypher
*/
//! Durable BI owner persistence for metadata-only Task event quarantine.

use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::task_event_inbox::{TaskEventRoutePin, MAX_TASK_WIRE_BYTES};
use crate::task_event_receiver::{
    TaskEventQuarantineFault, TaskEventQuarantinePort, TaskEventQuarantineReceipt,
    TaskEventReceiveError, TaskEventReceiveOrigin, TaskEventReceiveRoute,
};

/// Owner-local adapter that persists quarantine metadata through one fixed BI database function.
pub struct TaskEventQuarantineStore {
    pool: PgPool,
}

impl TaskEventQuarantineStore {
    /// Bind the adapter to the BI owner's pool; it does not open a connection or start a worker.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TaskEventQuarantinePort for TaskEventQuarantineStore {
    async fn record(
        &self,
        route: &TaskEventReceiveRoute,
        origin: &TaskEventReceiveOrigin,
        wire_sha256: [u8; 32],
        wire_len: u32,
        fault: TaskEventQuarantineFault,
    ) -> Result<TaskEventQuarantineReceipt, TaskEventReceiveError> {
        let pin = route.pin();
        if !source_origin_matches_pin(origin, pin) {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        let wire_len =
            usize::try_from(wire_len).map_err(|_| TaskEventReceiveError::WireTooLarge)?;
        if wire_len > MAX_TASK_WIRE_BYTES {
            return Err(TaskEventReceiveError::WireTooLarge);
        }

        let mut transaction = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
            .bind(route.tenant_id().to_string())
            .execute(&mut *transaction)
            .await?;

        let row = sqlx::query(
            "SELECT outcome, quarantine_id \
             FROM bi.record_task_event_quarantine_v2( \
                 $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20 \
             )",
        )
        .bind(route.tenant_id())
        .bind(pin.sender_service_id)
        .bind(pin.receiver_service_id)
        .bind(pin.transport_provider_id.as_str())
        .bind(pin.transport_provider_version.as_str())
        .bind(pin.transport_profile_id)
        .bind(pin.transport_profile_revision)
        .bind(pin.transport_profile_digest.as_slice())
        .bind(pin.endpoint_identity.as_str())
        .bind(pin.destination_id.as_str())
        .bind(pin.destination_generation)
        .bind(origin.provider_identity())
        .bind(origin.log_id())
        .bind(origin.log_generation())
        .bind(origin.source_sequence())
        .bind(origin.cursor())
        .bind(origin.message_id())
        .bind(wire_sha256.as_slice())
        .bind(i32::try_from(wire_len).map_err(|_| TaskEventReceiveError::WireTooLarge)?)
        .bind(fault.database_code())
        .fetch_one(&mut *transaction)
        .await?;

        let outcome: String = row.try_get("outcome")?;
        let quarantine_id: Option<Uuid> = row.try_get("quarantine_id")?;
        if outcome == "conflict" {
            transaction.commit().await?;
            return Err(TaskEventReceiveError::QuarantineConflict);
        }
        if !matches!(outcome.as_str(), "recorded" | "exact_replay") {
            return Err(TaskEventReceiveError::Quarantine);
        }
        let quarantine_id = quarantine_id
            .filter(|value| !value.is_nil())
            .ok_or(TaskEventReceiveError::Quarantine)?;
        transaction.commit().await?;

        Ok(TaskEventQuarantineReceipt { quarantine_id })
    }
}

impl TaskEventQuarantineFault {
    fn database_code(self) -> &'static str {
        match self {
            Self::MalformedEnvelope => "malformed_envelope",
            Self::MessageIdentityMismatch => "message_identity_mismatch",
            Self::ContentConflict => "content_conflict",
        }
    }
}

fn source_origin_matches_pin(origin: &TaskEventReceiveOrigin, pin: &TaskEventRoutePin) -> bool {
    pin.is_valid()
        && origin.provider_identity() == pin.transport_provider_id
        && origin.log_id() == pin.destination_id
        && origin.log_generation() == pin.destination_generation
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route_pin() -> TaskEventRoutePin {
        TaskEventRoutePin {
            sender_service_id: Uuid::new_v4(),
            receiver_service_id: Uuid::new_v4(),
            transport_provider_id: "star-rust-log".to_owned(),
            transport_provider_version: "1.0.0".to_owned(),
            transport_profile_id: Uuid::new_v4(),
            transport_profile_revision: 1,
            transport_profile_digest: [0x24; 32],
            endpoint_identity: "bi-quarantine-test".to_owned(),
            destination_id: "bi-task-events".to_owned(),
            destination_generation: Uuid::new_v4(),
        }
    }

    fn source_origin(provider: &str, log_id: &str, generation: Uuid) -> TaskEventReceiveOrigin {
        TaskEventReceiveOrigin::new(
            provider.to_owned(),
            log_id.to_owned(),
            generation,
            23,
            23_i64.to_be_bytes().to_vec(),
            Uuid::new_v4(),
        )
        .expect("test source origin is valid")
    }

    #[test]
    fn quarantine_origin_must_match_provider_log_and_generation() {
        let pin = route_pin();
        let exact = source_origin(
            &pin.transport_provider_id,
            &pin.destination_id,
            pin.destination_generation,
        );
        assert!(source_origin_matches_pin(&exact, &pin));

        let wrong_provider = source_origin(
            "other-provider",
            &pin.destination_id,
            pin.destination_generation,
        );
        assert!(!source_origin_matches_pin(&wrong_provider, &pin));

        let wrong_log = source_origin(
            &pin.transport_provider_id,
            "different-log",
            pin.destination_generation,
        );
        assert!(!source_origin_matches_pin(&wrong_log, &pin));

        let wrong_generation = source_origin(
            &pin.transport_provider_id,
            &pin.destination_id,
            Uuid::new_v4(),
        );
        assert!(!source_origin_matches_pin(&wrong_generation, &pin));
    }

    #[test]
    fn quarantine_migration_preserves_sequence_conflicts_and_exact_receipts() {
        let migration =
            include_str!("../../../db/migrations/2026-10-24-bi-task-event-source-origin.sql");
        let compact_migration = migration.split_whitespace().collect::<Vec<_>>().join(" ");
        let quarantine_function = migration
            .split("CREATE OR REPLACE FUNCTION bi.record_task_event_quarantine_v2")
            .nth(1)
            .expect("migration defines the cursor-aware quarantine writer")
            .split("ALTER FUNCTION bi.record_task_event_quarantine_v2")
            .next()
            .expect("quarantine function body is bounded before grant changes");
        assert!(!migration.contains("ALTER COLUMN source_sequence DROP NOT NULL"));
        assert!(quarantine_function.contains("legacy.source_sequence = p_source_sequence"));
        assert!(migration.contains("DROP CONSTRAINT IF EXISTS task_event_quarantine_origin_uq"));
        assert!(
            migration.contains("CREATE UNIQUE INDEX task_event_quarantine_sequence_origin_uidx")
        );
        assert!(compact_migration.contains(
            "tenant_id, consumer_id, origin_provider_identity, source_log_id, source_log_generation, source_sequence"
        ));
        assert!(quarantine_function.contains("pg_advisory_xact_lock(hashtextextended("));
        let identity_lookup = quarantine_function
            .split("SELECT count(*) INTO matched_rows")
            .nth(1)
            .expect("quarantine looks up an existing source identity after insert conflict")
            .split("IF matched_rows = 1 THEN")
            .next()
            .expect("source lookup is bounded before exact replay comparisons");
        assert!(quarantine_function.contains(
            "saved.source_sequence = p_source_sequence OR saved.source_cursor = p_source_cursor"
        ));
        assert!(
            identity_lookup.contains("saved.origin_provider_identity = p_origin_provider_identity")
        );
        assert!(identity_lookup.contains("saved.source_log_id = p_source_log_id"));
        assert!(!identity_lookup.contains("saved.sender_service_id = p_sender_service_id"));
        assert!(!identity_lookup.contains("saved.transport_profile_id = p_transport_profile_id"));
        assert!(quarantine_function.contains("stored_row.source_sequence = p_source_sequence"));
        assert!(quarantine_function.contains("stored_row.message_id = p_message_id"));
        assert!(quarantine_function
            .contains("stored_row.transport_profile_id = p_transport_profile_id"));
        assert!(quarantine_function
            .contains("'exact_replay'::TEXT, stored_row.quarantine_id, stored_row.recorded_at"));
        assert!(quarantine_function.contains("ON CONFLICT DO NOTHING"));
        assert!(migration.contains("CREATE TABLE IF NOT EXISTS bi.task_event_source_origin"));
        assert!(migration.contains("ADD COLUMN IF NOT EXISTS source_cursor BYTEA"));
        assert!(migration.contains("CREATE OR REPLACE FUNCTION bi.commit_task_event_v3"));
        assert!(migration.contains("CREATE OR REPLACE FUNCTION bi.record_task_event_quarantine_v2"));
        assert!(migration.contains("DROP POLICY IF EXISTS task_event_source_origin_tenant_policy"));
        assert!(migration.contains("DROP TRIGGER IF EXISTS task_event_source_origin_worm"));
        assert!(migration.contains("GRANT EXECUTE ON FUNCTION bi.record_task_event_quarantine_v2"));
    }
}
