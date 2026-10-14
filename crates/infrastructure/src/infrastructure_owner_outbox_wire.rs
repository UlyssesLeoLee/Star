// @cypher schema=1 source_sha256=03d339b3f0db0754d9b3297fbb20651a8c4ecc3d0596e32348656bffaf2a9292
// MERGE (self:File {path:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs"})
// MERGE (module:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs::infrastructure_owner_outbox_wire",kind:"module"})
// MERGE (envelope:Type {id:"infrastructure::InfrastructureEventEnvelope",kind:"struct"})
// MERGE (wire_error:Type {id:"infrastructure::CapacityResultWireError",kind:"struct"})
// MERGE (build_wire:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs::build_capacity_result_wire",kind:"function"})
// MERGE (test_claim:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs::tests::claim",kind:"function"})
// MERGE (test_allowlist:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs::tests::result_events_are_allowlisted_and_versioned",kind:"test"})
// MERGE (test_bounds:Symbol {id:"crates/infrastructure/src/infrastructure_owner_outbox_wire.rs::tests::invalid_and_oversized_payloads_fail_closed",kind:"test"})
// MERGE (self)-[:DEFINES]->(module)
// MERGE (module)-[:DEFINES]->(envelope)
// MERGE (module)-[:DEFINES]->(wire_error)
// MERGE (module)-[:DEFINES]->(build_wire)
// MERGE (module)-[:DEFINES]->(test_claim)
// MERGE (module)-[:DEFINES]->(test_allowlist)
// MERGE (module)-[:DEFINES]->(test_bounds)
// MERGE (build_wire)-[:USES_TYPE]->(envelope)
// MERGE (test_allowlist)-[:TESTS]->(build_wire)
// MERGE (test_allowlist)-[:CALLS]->(test_claim)
// MERGE (test_bounds)-[:TESTS]->(build_wire)
// MERGE (test_bounds)-[:CALLS]->(test_claim)
// @endcypher
//! Provider-neutral, bounded serialization for Infrastructure capacity-result Outbox events.

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::infrastructure_owner_outbox_delivery::InfrastructureOwnerOutboxDeliveryClaim;

const MAX_EVENT_PAYLOAD_BYTES: usize = 16 * 1024;
const MAX_EVENT_ENVELOPE_BYTES: usize = 128 * 1024;

#[derive(Serialize)]
struct InfrastructureEventEnvelope {
    envelope_version: u8,
    tenant_id: Uuid,
    event_id: Uuid,
    event_type: String,
    schema_version: i16,
    idempotency_key: String,
    correlation_id: Uuid,
    payload: Value,
}

/// Validation or serialization failed for an Infrastructure result event.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("Infrastructure capacity-result event is invalid")]
pub(crate) struct CapacityResultWireError;

/// Serialize the immutable owner event once so every transport sees the same wire contract.
pub(crate) fn build_capacity_result_wire(
    claim: &InfrastructureOwnerOutboxDeliveryClaim,
) -> Result<Vec<u8>, CapacityResultWireError> {
    if !matches!(
        claim.event_type.as_str(),
        "capacity_accepted" | "capacity_rejected" | "capacity_expired"
    ) || claim.tenant_id.is_nil()
        || claim.event_id.is_nil()
        || claim.lease_owner_id.is_nil()
        || claim.correlation_id.is_nil()
        || claim.attempt_count <= 0
        || claim.schema_version != 2
        || claim.idempotency_key.is_empty()
        || claim.idempotency_key.len() > 200
        || claim.payload_json.len() > MAX_EVENT_PAYLOAD_BYTES
    {
        return Err(CapacityResultWireError);
    }

    let payload: Value =
        serde_json::from_str(&claim.payload_json).map_err(|_| CapacityResultWireError)?;
    if !payload.is_object() {
        return Err(CapacityResultWireError);
    }
    let envelope = InfrastructureEventEnvelope {
        envelope_version: 1,
        tenant_id: claim.tenant_id,
        event_id: claim.event_id,
        event_type: claim.event_type.clone(),
        schema_version: claim.schema_version,
        idempotency_key: claim.idempotency_key.clone(),
        correlation_id: claim.correlation_id,
        payload,
    };
    let wire = serde_json::to_vec(&envelope).map_err(|_| CapacityResultWireError)?;
    if wire.len() > MAX_EVENT_ENVELOPE_BYTES {
        return Err(CapacityResultWireError);
    }
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn claim(event_type: &str, payload_json: &str) -> InfrastructureOwnerOutboxDeliveryClaim {
        InfrastructureOwnerOutboxDeliveryClaim {
            tenant_id: Uuid::from_u128(1),
            event_id: Uuid::from_u128(2),
            lease_owner_id: Uuid::from_u128(3),
            attempt_count: 1,
            event_type: event_type.to_owned(),
            schema_version: 2,
            idempotency_key: "capacity-demand:run-1:occurrence-1".to_owned(),
            correlation_id: Uuid::from_u128(4),
            payload_json: payload_json.to_owned(),
            lease_expires_at: Utc::now(),
        }
    }

    #[test]
    fn result_events_are_allowlisted_and_versioned() {
        let wire = build_capacity_result_wire(&claim("capacity_accepted", r#"{"host":"h1"}"#))
            .expect("valid event should serialize");
        let envelope: Value = serde_json::from_slice(&wire).expect("serialized JSON envelope");
        assert_eq!(envelope["envelope_version"], 1);
        assert_eq!(envelope["event_id"], Uuid::from_u128(2).to_string());
        assert_eq!(envelope["event_type"], "capacity_accepted");
        assert_eq!(envelope["payload"]["host"], "h1");

        assert!(build_capacity_result_wire(&claim("capacity_released", "{}")).is_err());
    }

    #[test]
    fn invalid_and_oversized_payloads_fail_closed() {
        assert!(build_capacity_result_wire(&claim("capacity_expired", "not-json")).is_err());
        assert!(build_capacity_result_wire(&claim("capacity_expired", "[]")).is_err());
        let oversized = format!(r#"{{"data":"{}"}}"#, "x".repeat(MAX_EVENT_PAYLOAD_BYTES));
        assert!(build_capacity_result_wire(&claim("capacity_expired", &oversized)).is_err());

        let mut unsupported = claim("capacity_rejected", "{}");
        unsupported.schema_version = 3;
        assert!(build_capacity_result_wire(&unsupported).is_err());
    }
}
