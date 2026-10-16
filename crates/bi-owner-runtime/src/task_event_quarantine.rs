// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=129609d41b70b8f4acfba6eeff9e93a5185f80a26c97425a25ed6578190c9d51
MERGE (self:File {path:"crates/bi-owner-runtime/src/task_event_quarantine.rs"})
MERGE (store:Type {id:"bi_owner_runtime::TaskEventQuarantineStore",kind:"struct"})
MERGE (constructor:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::TaskEventQuarantineStore.new",kind:"method"})
MERGE (record:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::TaskEventQuarantineStore.record",kind:"method"})
MERGE (port:Type {id:"bi_owner_runtime::TaskEventQuarantinePort",kind:"trait"})
MERGE (route:Type {id:"bi_owner_runtime::TaskEventReceiveRoute",kind:"struct"})
MERGE (origin:Type {id:"bi_owner_runtime::TaskEventReceiveOrigin",kind:"struct"})
MERGE (fault:Type {id:"bi_owner_runtime::TaskEventQuarantineFault",kind:"enum"})
MERGE (receipt:Type {id:"bi_owner_runtime::TaskEventQuarantineReceipt",kind:"struct"})
MERGE (error:Type {id:"bi_owner_runtime::TaskEventReceiveError",kind:"enum"})
MERGE (pool:Type {id:"sqlx::PgPool",kind:"struct"})
MERGE (row:Type {id:"sqlx::Row",kind:"trait"})
MERGE (function:Database {id:"bi.record_task_event_quarantine_v1",kind:"stored_function"})
MERGE (table:Table {id:"bi.task_event_quarantine",kind:"table"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:IMPORTS]->(pool)
MERGE (self)-[:IMPORTS]->(row)
MERGE (store)-[:DEFINES]->(constructor)
MERGE (store)-[:IMPLEMENTS]->(port)
MERGE (record)-[:USES_TYPE]->(route)
MERGE (record)-[:USES_TYPE]->(origin)
MERGE (record)-[:USES_TYPE]->(fault)
MERGE (record)-[:USES_TYPE]->(receipt)
MERGE (record)-[:USES_TYPE]->(error)
MERGE (record)-[:CALLS]->(function)
MERGE (function)-[:WRITES]->(table)
@endcypher
*/
//! Durable BI owner persistence for metadata-only Task event quarantine.

use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::task_event_inbox::MAX_TASK_WIRE_BYTES;
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
        if !pin.is_valid() || origin.provider_identity() != pin.transport_provider_id.as_str() {
            return Err(TaskEventReceiveError::Unauthorized);
        }
        let wire_len = usize::try_from(wire_len).map_err(|_| TaskEventReceiveError::WireTooLarge)?;
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
             FROM bi.record_task_event_quarantine_v1( \
                 $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19 \
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
        .bind(origin.sequence())
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
