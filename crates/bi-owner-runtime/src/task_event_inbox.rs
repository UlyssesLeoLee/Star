// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=222cba4393358dea6711a10bfb7e539ad32713042485d7d84e70f5df51574d8b
MERGE (self:File {path:"crates/bi-owner-runtime/src/task_event_inbox.rs"})
MERGE (store:Type {id:"bi_owner_runtime::TaskEventInboxStore",kind:"struct"})
MERGE (wire:Type {id:"bi_owner_runtime::TaskEventWireV1",kind:"struct"})
MERGE (validated:Type {id:"bi_owner_runtime::ValidatedTaskEventWire",kind:"struct"})
MERGE (receipt:Type {id:"bi_owner_runtime::TaskEventInboxReceipt",kind:"struct"})
MERGE (route:Type {id:"bi_owner_runtime::TaskEventRoutePin",kind:"struct"})
MERGE (principal:Type {id:"bi_owner_runtime::AuthenticatedTaskEventPrincipal",kind:"struct"})
MERGE (route_input:Type {id:"bi_owner_runtime::RoutePinDigestInput",kind:"struct"})
MERGE (outcome:Type {id:"bi_owner_runtime::ReceiptOutcome",kind:"enum"})
MERGE (error:Type {id:"bi_owner_runtime::TaskEventInboxError",kind:"enum"})
MERGE (new:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventInboxStore.new",kind:"method"})
MERGE (commit:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventInboxStore.commit_exact_wire",kind:"method"})
MERGE (begin:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventInboxStore.begin_scoped",kind:"method"})
MERGE (route_valid:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventRoutePin.is_valid",kind:"method"})
MERGE (validate:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::validate_wire",kind:"function"})
MERGE (validate_payload:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::validate_payload_scope",kind:"function"})
MERGE (route_digest:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::TaskEventRoutePin.digest",kind:"method"})
MERGE (transport_label:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::valid_transport_label",kind:"function"})
MERGE (transport_version:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::valid_transport_version",kind:"function"})
MERGE (provider_id:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::valid_provider_id",kind:"function"})
MERGE (test_module:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::tests",kind:"module"})
MERGE (test_fixture:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::tests.valid_lifecycle_wire",kind:"function"})
MERGE (valid_lifecycle:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::tests.accepts_valid_lifecycle_wire",kind:"test"})
MERGE (reject_tenant:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::tests.rejects_authenticated_tenant_mismatch",kind:"test"})
MERGE (reject_unknown:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::tests.rejects_unknown_envelope_fields",kind:"test"})
MERGE (store_fn:Database {id:"bi.commit_task_event_v1",kind:"stored_function"})
MERGE (chrono:ExternalService {id:"chrono",kind:"crate"})
MERGE (serde:ExternalService {id:"serde",kind:"crate"})
MERGE (serde_json:ExternalService {id:"serde_json",kind:"crate"})
MERGE (sha2:ExternalService {id:"sha2",kind:"crate"})
MERGE (sqlx:ExternalService {id:"sqlx",kind:"crate"})
MERGE (uuid:ExternalService {id:"uuid",kind:"crate"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(wire)
MERGE (self)-[:DEFINES]->(validated)
MERGE (self)-[:DEFINES]->(receipt)
MERGE (self)-[:DEFINES]->(route)
MERGE (self)-[:DEFINES]->(principal)
MERGE (self)-[:DEFINES]->(route_input)
MERGE (self)-[:DEFINES]->(outcome)
MERGE (self)-[:DEFINES]->(error)
MERGE (self)-[:IMPORTS]->(chrono)
MERGE (self)-[:IMPORTS]->(serde)
MERGE (self)-[:IMPORTS]->(serde_json)
MERGE (self)-[:IMPORTS]->(sha2)
MERGE (self)-[:IMPORTS]->(sqlx)
MERGE (self)-[:IMPORTS]->(uuid)
MERGE (store)-[:DEFINES]->(new)
MERGE (store)-[:DEFINES]->(commit)
MERGE (store)-[:DEFINES]->(begin)
MERGE (route)-[:DEFINES]->(route_valid)
MERGE (route)-[:DEFINES]->(route_digest)
MERGE (self)-[:DEFINES]->(validate)
MERGE (self)-[:DEFINES]->(validate_payload)
MERGE (self)-[:DEFINES]->(transport_label)
MERGE (self)-[:DEFINES]->(transport_version)
MERGE (self)-[:DEFINES]->(provider_id)
MERGE (commit)-[:CALLS]->(validate)
MERGE (commit)-[:CALLS]->(begin)
MERGE (commit)-[:CALLS]->(store_fn)
MERGE (commit)-[:CALLS]->(route_digest)
MERGE (commit)-[:CALLS]->(route_valid)
MERGE (commit)-[:READS]->(principal)
MERGE (route_valid)-[:CALLS]->(transport_label)
MERGE (route_valid)-[:CALLS]->(transport_version)
MERGE (route_valid)-[:CALLS]->(provider_id)
MERGE (validate)-[:CALLS]->(validate_payload)
MERGE (self)-[:DEFINES]->(test_module)
MERGE (test_module)-[:DEFINES]->(test_fixture)
MERGE (test_module)-[:DEFINES]->(valid_lifecycle)
MERGE (test_module)-[:DEFINES]->(reject_tenant)
MERGE (test_module)-[:DEFINES]->(reject_unknown)
MERGE (valid_lifecycle)-[:CALLS]->(test_fixture)
MERGE (reject_tenant)-[:CALLS]->(test_fixture)
MERGE (reject_unknown)-[:CALLS]->(test_fixture)
MERGE (valid_lifecycle)-[:TESTS]->(validate)
MERGE (reject_tenant)-[:TESTS]->(validate)
MERGE (reject_unknown)-[:TESTS]->(validate)
@endcypher
*/
//! Bounded Task event validation and owner-local Inbox commit for the BI subscriber.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPool, Row, Transaction};
use thiserror::Error;
use uuid::Uuid;

const MAX_TASK_PAYLOAD_BYTES: usize = 8 * 1024;
const MAX_TASK_WIRE_BYTES: usize = 16 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskEventWireV1 {
    envelope_version: u8,
    tenant_id: Uuid,
    project_id: Uuid,
    engineering_run_id: Uuid,
    event_id: Uuid,
    work_item_id: Uuid,
    aggregate_type: String,
    aggregate_version: i32,
    event_type: String,
    schema_version: i32,
    actor_id: Uuid,
    correlation_id: Uuid,
    occurred_at: DateTime<Utc>,
    payload: Value,
}

struct ValidatedTaskEventWire {
    event: TaskEventWireV1,
    wire_sha256: [u8; 32],
}

/// Non-secret fields copied from the immutable sender route pin and authenticated connection.
#[derive(Debug, Clone)]
pub struct TaskEventRoutePin {
    pub sender_service_id: Uuid,
    pub receiver_service_id: Uuid,
    pub transport_provider_id: String,
    pub transport_provider_version: String,
    pub transport_profile_id: Uuid,
    pub transport_profile_revision: i64,
    pub transport_profile_digest: [u8; 32],
    pub endpoint_identity: String,
    pub destination_id: String,
    pub destination_generation: Uuid,
}

/// Transport identity minted only inside this crate after a trusted receiver verifies it.
/// Public callers cannot construct this value from untrusted UUIDs.
#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedTaskEventPrincipal {
    pub(crate) tenant_id: Uuid,
    pub(crate) sender_service_id: Uuid,
    pub(crate) receiver_service_id: Uuid,
}

#[derive(Serialize)]
struct RoutePinDigestInput<'a> {
    contract_version: u8,
    sender_service_id: Uuid,
    receiver_service_id: Uuid,
    transport_provider_id: &'a str,
    transport_provider_version: &'a str,
    transport_profile_id: Uuid,
    transport_profile_revision: i64,
    transport_profile_digest: &'a [u8; 32],
    endpoint_identity: &'a str,
    destination_id: &'a str,
    destination_generation: Uuid,
    schema_version: i32,
    wire_sha256: &'a [u8; 32],
}

impl TaskEventRoutePin {
    fn digest(
        &self,
        schema_version: i32,
        wire_sha256: &[u8; 32],
    ) -> Result<[u8; 32], TaskEventInboxError> {
        let input = RoutePinDigestInput {
            contract_version: 1,
            sender_service_id: self.sender_service_id,
            receiver_service_id: self.receiver_service_id,
            transport_provider_id: &self.transport_provider_id,
            transport_provider_version: &self.transport_provider_version,
            transport_profile_id: self.transport_profile_id,
            transport_profile_revision: self.transport_profile_revision,
            transport_profile_digest: &self.transport_profile_digest,
            endpoint_identity: &self.endpoint_identity,
            destination_id: &self.destination_id,
            destination_generation: self.destination_generation,
            schema_version,
            wire_sha256,
        };
        let bytes = serde_json::to_vec(&input).map_err(|_| TaskEventInboxError::InvalidEnvelope)?;
        Ok(Sha256::digest(bytes).into())
    }

    fn is_valid(&self) -> bool {
        !self.sender_service_id.is_nil()
            && !self.receiver_service_id.is_nil()
            && valid_provider_id(&self.transport_provider_id)
            && valid_transport_version(&self.transport_provider_version)
            && !self.transport_profile_id.is_nil()
            && self.transport_profile_revision > 0
            && valid_transport_label(&self.endpoint_identity, 128)
            && valid_transport_label(&self.destination_id, 128)
            && !self.destination_generation.is_nil()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptOutcome {
    Committed,
    ExactReplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEventInboxReceipt {
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub engineering_run_id: Uuid,
    pub event_id: Uuid,
    pub consumer_id: &'static str,
    pub sender_service_id: Uuid,
    pub receiver_service_id: Uuid,
    pub transport_profile_id: Uuid,
    pub transport_profile_revision: i64,
    pub transport_profile_digest: [u8; 32],
    pub endpoint_identity: String,
    pub destination_id: String,
    pub destination_generation: Uuid,
    pub route_pin_sha256: [u8; 32],
    pub receipt_id: Uuid,
    pub wire_sha256: [u8; 32],
    pub committed_at: DateTime<Utc>,
    pub outcome: ReceiptOutcome,
}

#[derive(Debug, Error)]
pub enum TaskEventInboxError {
    #[error("invalid Task event envelope")]
    InvalidEnvelope,
    #[error("Task event tenant does not match the authenticated tenant scope")]
    TenantMismatch,
    #[error("Task event identity conflicts with previously committed BI content")]
    IdentityConflict,
    #[error("Task event route does not match the authenticated sender and BI receiver")]
    RouteIdentityMismatch,
    #[error("BI owner database operation failed")]
    Database(#[from] sqlx::Error),
}

pub struct TaskEventInboxStore {
    pool: PgPool,
    receiver_service_id: Uuid,
}

impl TaskEventInboxStore {
    pub fn new(pool: PgPool, receiver_service_id: Uuid) -> Self {
        Self { pool, receiver_service_id }
    }

    /// Commit one exact Task wire after an in-crate trusted receiver authenticated sender and scope.
    /// The returned receipt is produced only after the BI Inbox/fact transaction commits.
    pub async fn commit_exact_wire(
        &self,
        principal: &AuthenticatedTaskEventPrincipal,
        route: &TaskEventRoutePin,
        wire: &[u8],
    ) -> Result<TaskEventInboxReceipt, TaskEventInboxError> {
        let validated = validate_wire(principal.tenant_id, wire)?;
        if !route.is_valid()
            || principal.sender_service_id != route.sender_service_id
            || principal.receiver_service_id != self.receiver_service_id
            || self.receiver_service_id != route.receiver_service_id
        {
            return Err(TaskEventInboxError::RouteIdentityMismatch);
        }
        let route_pin_sha256 = route.digest(validated.event.schema_version, &validated.wire_sha256)?;
        let event = validated.event;
        let mut tx = self.begin_scoped(principal.tenant_id).await?;
        let row = sqlx::query(
            r#"SELECT outcome, receipt_id, committed_at
                 FROM bi.commit_task_event_v1(
                      $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,
                      $15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25)"#,
        )
        .bind(event.tenant_id)
        .bind(event.project_id)
        .bind(event.engineering_run_id)
        .bind(event.event_id)
        .bind(event.work_item_id)
        .bind(event.aggregate_type)
        .bind(event.aggregate_version)
        .bind(event.event_type)
        .bind(event.schema_version)
        .bind(event.actor_id)
        .bind(event.correlation_id)
        .bind(event.occurred_at)
        .bind(event.payload)
        .bind(route.sender_service_id)
        .bind(route.receiver_service_id)
        .bind(route.transport_provider_id.as_str())
        .bind(route.transport_provider_version.as_str())
        .bind(route.transport_profile_id)
        .bind(route.transport_profile_revision)
        .bind(route.transport_profile_digest.as_slice())
        .bind(route.endpoint_identity.as_str())
        .bind(route.destination_id.as_str())
        .bind(route.destination_generation)
        .bind(route_pin_sha256.as_slice())
        .bind(validated.wire_sha256.as_slice())
        .fetch_one(&mut *tx)
        .await?;

        let outcome: String = row.try_get("outcome")?;
        let receipt_id: Option<Uuid> = row.try_get("receipt_id")?;
        let committed_at: Option<DateTime<Utc>> = row.try_get("committed_at")?;
        if outcome == "conflict" {
            tx.commit().await?;
            return Err(TaskEventInboxError::IdentityConflict);
        }

        let receipt_id = receipt_id.ok_or(TaskEventInboxError::InvalidEnvelope)?;
        let committed_at = committed_at.ok_or(TaskEventInboxError::InvalidEnvelope)?;
        let receipt_outcome = match outcome.as_str() {
            "committed" => ReceiptOutcome::Committed,
            "exact_replay" => ReceiptOutcome::ExactReplay,
            _ => return Err(TaskEventInboxError::InvalidEnvelope),
        };
        tx.commit().await?;

        Ok(TaskEventInboxReceipt {
            tenant_id: principal.tenant_id,
            project_id: event.project_id,
            engineering_run_id: event.engineering_run_id,
            event_id: event.event_id,
            consumer_id: "bi.task-event-fact.v1",
            sender_service_id: route.sender_service_id,
            receiver_service_id: route.receiver_service_id,
            transport_profile_id: route.transport_profile_id,
            transport_profile_revision: route.transport_profile_revision,
            transport_profile_digest: route.transport_profile_digest,
            endpoint_identity: route.endpoint_identity.clone(),
            destination_id: route.destination_id.clone(),
            destination_generation: route.destination_generation,
            route_pin_sha256,
            receipt_id,
            wire_sha256: validated.wire_sha256,
            committed_at,
            outcome: receipt_outcome,
        })
    }

    async fn begin_scoped(
        &self,
        tenant_id: Uuid,
    ) -> Result<Transaction<'static, sqlx::Postgres>, TaskEventInboxError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(&mut *tx)
            .await?;
        sqlx::query("SET LOCAL lock_timeout = '5s'")
            .execute(&mut *tx)
            .await?;
        sqlx::query("SET LOCAL statement_timeout = '10s'")
            .execute(&mut *tx)
            .await?;
        Ok(tx)
    }
}

fn valid_transport_label(value: &str, max_len: usize) -> bool {
    (1..=max_len).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn valid_transport_version(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || b"._:+-".contains(&byte)
        })
}

fn valid_provider_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_wire(
    authenticated_tenant_id: Uuid,
    wire: &[u8],
) -> Result<ValidatedTaskEventWire, TaskEventInboxError> {
    if wire.is_empty() || wire.len() > MAX_TASK_WIRE_BYTES {
        return Err(TaskEventInboxError::InvalidEnvelope);
    }
    let event: TaskEventWireV1 =
        serde_json::from_slice(wire).map_err(|_| TaskEventInboxError::InvalidEnvelope)?;
    if event.envelope_version != 1 {
        return Err(TaskEventInboxError::InvalidEnvelope);
    }
    if event.tenant_id != authenticated_tenant_id {
        return Err(TaskEventInboxError::TenantMismatch);
    }
    if event.tenant_id.is_nil()
        || event.project_id.is_nil()
        || event.engineering_run_id.is_nil()
        || event.event_id.is_nil()
        || event.work_item_id.is_nil()
        || event.actor_id.is_nil()
        || event.correlation_id.is_nil()
        || event.aggregate_version <= 0
    {
        return Err(TaskEventInboxError::InvalidEnvelope);
    }
    validate_payload_scope(&event)?;
    let payload_size = serde_json::to_vec(&event.payload)
        .map_err(|_| TaskEventInboxError::InvalidEnvelope)?
        .len();
    if payload_size > MAX_TASK_PAYLOAD_BYTES {
        return Err(TaskEventInboxError::InvalidEnvelope);
    }
    Ok(ValidatedTaskEventWire {
        event,
        wire_sha256: Sha256::digest(wire).into(),
    })
}

fn validate_payload_scope(event: &TaskEventWireV1) -> Result<(), TaskEventInboxError> {
    let compatible = matches!(
        (event.schema_version, event.aggregate_type.as_str(), event.event_type.as_str()),
        (1 | 2, "metadata", "task.created" | "task.updated" | "task.owner_changed")
            | (3, "lifecycle", "task.lifecycle.changed")
    );
    if !compatible || !event.payload.is_object() {
        return Err(TaskEventInboxError::InvalidEnvelope);
    }

    let task_id = Value::String(event.work_item_id.to_string());
    let project_id = Value::String(event.project_id.to_string());
    let run_id = Value::String(event.engineering_run_id.to_string());
    let tenant_id = Value::String(event.tenant_id.to_string());
    let version = Value::from(event.aggregate_version);
    let scope_matches = match event.schema_version {
        1 => {
            event.payload["work_item_id"] == task_id
                && event.payload["project_id"] == project_id
                && event.payload["engineering_run_id"] == run_id
                && event.payload["metadata_version"] == version
        }
        2 => {
            event.payload["schema_version"] == Value::from(2)
                && event.payload["work_item_id"] == task_id
                && event.payload["owner"]["tenant_id"] == tenant_id
                && event.payload["owner"]["project_id"] == project_id
                && event.payload["owner"]["engineering_run_id"] == run_id
                && event.payload["metadata_version"] == version
        }
        3 => {
            event.payload["schema_version"] == Value::from(3)
                && event.payload["aggregate"]["type"] == event.aggregate_type
                && event.payload["aggregate"]["version"] == version
                && event.payload["work_item_id"] == task_id
                && event.payload["owner"]["tenant_id"] == tenant_id
                && event.payload["owner"]["project_id"] == project_id
                && event.payload["owner"]["engineering_run_id"] == run_id
        }
        _ => false,
    };
    if !scope_matches {
        return Err(TaskEventInboxError::InvalidEnvelope);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_lifecycle_wire() -> (Uuid, Vec<u8>) {
        let tenant_id = Uuid::from_u128(1);
        let project_id = Uuid::from_u128(2);
        let run_id = Uuid::from_u128(3);
        let work_item_id = Uuid::from_u128(4);
        let payload = json!({
            "schema_version": 3,
            "aggregate": {"type": "lifecycle", "version": 7},
            "work_item_id": work_item_id.to_string(),
            "owner": {
                "tenant_id": tenant_id.to_string(),
                "project_id": project_id.to_string(),
                "engineering_run_id": run_id.to_string()
            }
        });
        let wire = json!({
            "envelope_version": 1,
            "tenant_id": tenant_id,
            "project_id": project_id,
            "engineering_run_id": run_id,
            "event_id": Uuid::from_u128(5),
            "work_item_id": work_item_id,
            "aggregate_type": "lifecycle",
            "aggregate_version": 7,
            "event_type": "task.lifecycle.changed",
            "schema_version": 3,
            "actor_id": Uuid::from_u128(6),
            "correlation_id": Uuid::from_u128(7),
            "occurred_at": "2026-10-16T00:00:00Z",
            "payload": payload
        });
        (tenant_id, serde_json::to_vec(&wire).expect("valid fixture serializes"))
    }

    #[test]
    fn accepts_valid_lifecycle_wire() {
        let (tenant_id, wire) = valid_lifecycle_wire();
        let validated = validate_wire(tenant_id, &wire).expect("valid Task wire accepted");
        assert_eq!(validated.event.schema_version, 3);
        assert_eq!(validated.event.event_type, "task.lifecycle.changed");
        let expected_digest: [u8; 32] = Sha256::digest(&wire).into();
        assert_eq!(validated.wire_sha256, expected_digest);
    }

    #[test]
    fn rejects_authenticated_tenant_mismatch() {
        let (tenant_id, wire) = valid_lifecycle_wire();
        assert!(matches!(
            validate_wire(Uuid::from_u128(99), &wire),
            Err(TaskEventInboxError::TenantMismatch)
        ));
        assert_ne!(tenant_id, Uuid::from_u128(99));
    }

    #[test]
    fn rejects_unknown_envelope_fields() {
        let (tenant_id, wire) = valid_lifecycle_wire();
        let mut value: Value = serde_json::from_slice(&wire).expect("fixture parses");
        value["unexpected"] = Value::Bool(true);
        let drifted_wire = serde_json::to_vec(&value).expect("fixture serializes");
        assert!(matches!(
            validate_wire(tenant_id, &drifted_wire),
            Err(TaskEventInboxError::InvalidEnvelope)
        ));
    }
}
