// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=4d7ffdfcd3fb3effef849284165c2240e2ad02cd4e724f6ca1410890aaf51257
MERGE (self:File {path:"crates/star-api-rest/src/device_outbox/mod.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/device_outbox/mod.rs::device_outbox",kind:"module"})
MERGE (contract:Symbol {id:"crates/star-api-rest/src/device_outbox/contract.rs::module",kind:"module"})
MERGE (identity:Symbol {id:"crates/star-api-rest/src/device_outbox/identity_owner.rs::module",kind:"module"})
MERGE (inbox:Symbol {id:"crates/star-api-rest/src/device_outbox/collaboration_inbox.rs::module",kind:"module"})
MERGE (policy:Symbol {id:"crates/star-api-rest/src/device_outbox/auth_policy.rs::module",kind:"module"})
MERGE (relay:Symbol {id:"crates/star-api-rest/src/device_outbox/relay.rs::module",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:CONTAINS]->(contract),(module)-[:CONTAINS]->(identity),(module)-[:CONTAINS]->(inbox),(module)-[:CONTAINS]->(policy),(module)-[:CONTAINS]->(relay)
MERGE (self)-[:CREATES]->(:Table {name:"identity_device_outbox",kind:"table"})
MERGE (self)-[:CREATES]->(:Table {name:"collaboration_device_inbox",kind:"table"})
@endcypher
*/
//! Durable Identity Outbox → Collaboration Inbox owner contract core
//! (WBS 7.17.3, per PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106 推荐 owner
//! 合约 #1–#7).
//!
//! Owner boundaries enforced by construction:
//! - [`identity_owner`]: the Identity/Auth owner writes the revocation fact
//!   (generation advance, refresh-family revocation, audit) and the Outbox
//!   event in **one local transaction** (`star_oauth_credential_owner`).
//! - [`collaboration_inbox`]: the Collaboration owner applies events through
//!   its **own** Inbox + idempotent receipt + local projection + durable
//!   checkpoint (`star_collaboration_owner` / `star_collaboration_runtime`),
//!   committing the receipt, projection and cursor advance atomically.
//! - [`auth_policy`]: the projection only accelerates revocation; **no event
//!   is never a valid authorization** — positive decisions come only from a
//!   fresh Identity recheck and everything else fails closed.
//! - [`relay`]: at-least-once, bounded backoff, cancel-only-between-
//!   transactions, drain-before-profile-switch discipline.
//!
//! Delivery semantics are at-least-once; nothing in this module claims
//! exactly-once (review 合约 #4). The DDL lives at
//! `db/migrations/2026-11-14-w17-device-outbox-inbox.sql` and is applied by
//! the repository migration runner (one statement per line).

pub mod auth_policy;
pub mod collaboration_inbox;
pub mod contract;
pub mod identity_owner;
pub mod relay;

pub use auth_policy::{
    decide_device_authorization, DeviceAuthorizationDecision, FailClosedReason,
    IdentityRecheckOutcome, ProjectionRecord,
};
pub use collaboration_inbox::{
    ApplyOutcome, BoundedReplay, CollaborationInboxError, CollaborationInboxStore,
    InMemoryCollaborationInbox, PgCollaborationInboxStore,
};
pub use contract::{
    decide_apply, event_digest, owner_stream_id, validate_envelope, ApplyDecision,
    DeviceRevocationEvent, EnvelopeError, InboxCursor, ReceivedEvent, StoredInboxRow,
    SuspensionReason, CONSUMER_ID, EVENT_DIGEST_LEN, EVENT_TYPE_DEVICE_REVOKED,
    MAX_CURSOR_HANDLE_LEN, PRODUCER_ID, SCHEMA_VERSION,
};
pub use identity_owner::{
    DeviceRevocationOwnerError, DeviceRevocationOwnerStore, PgDeviceRevocationOwnerStore,
    RevocationReceipt,
};
pub use relay::{BoundedRelay, PendingDelivery, ProfileSwitchOutcome, RelayBackoffPolicy};

#[cfg(test)]
mod tests {
    //! Structural checks for the companion migration. The repository runner
    //! executes exactly one SQL statement per physical line (no BEGIN/COMMIT),
    //! so this test pins the line discipline and the contract-defining
    //! constraints without needing a live PostgreSQL.

    #[test]
    fn migration_is_one_statement_per_line_without_transaction_wrappers() {
        let migration =
            include_str!("../../../../db/migrations/2026-11-14-w17-device-outbox-inbox.sql");
        let statements: Vec<&str> = migration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("--"))
            .collect();
        assert!(
            statements.len() >= 60,
            "migration must carry the full DDL surface, got {} statements",
            statements.len()
        );
        for statement in &statements {
            assert!(
                !statement.starts_with("BEGIN") && !statement.starts_with("COMMIT"),
                "migration runner wraps statements in one transaction; explicit BEGIN/COMMIT is forbidden: {}",
                statement
            );
            assert_eq!(
                statement.matches('(').count(),
                statement.matches(')').count(),
                "unbalanced parentheses in statement: {}",
                &statement[..statement.len().min(80)]
            );
        }
    }

    #[test]
    fn migration_pins_idempotency_cursor_and_rls_contracts() {
        let migration =
            include_str!("../../../../db/migrations/2026-11-14-w17-device-outbox-inbox.sql");
        let compact: String = migration.split_whitespace().collect::<Vec<_>>().join(" ");
        // Idempotency unique constraint: producer + stream + sequence + event_id (review 合约 #4).
        assert!(
            compact.contains("PRIMARY KEY (tenant_id, consumer_id, producer_id, stream_id, source_sequence, event_id)"),
            "inbox idempotency primary key missing"
        );
        // Owner-monotonic replay order on the producer side (review 合约 #1/#5).
        assert!(
            compact.contains("PRIMARY KEY (stream_id, source_sequence)"),
            "outbox replay-order primary key missing"
        );
        // Durable checkpoint with an opaque, profile-bound cursor (review 合约 #5).
        assert!(
            compact.contains("cursor_handle BYTEA NOT NULL")
                && compact.contains("next_expected_sequence BIGINT"),
            "checkpoint cursor fields missing"
        );
        // Fail-closed integrity incidents (review 合约 #4/#5/#6).
        assert!(
            compact.contains("reason_code TEXT NOT NULL CHECK (reason_code IN ('digest_conflict','sequence_gap','stale_cursor','unknown_schema_version'))"),
            "incident reason codes missing"
        );
        // Owner-scoped RLS on every durable table.
        for table in [
            "identity_device_outbox",
            "collaboration_device_inbox",
            "collaboration_device_projection",
            "collaboration_inbox_checkpoint",
            "collaboration_device_event_incident",
        ] {
            assert!(
                compact.contains(&format!("ALTER TABLE {table} ENABLE ROW LEVEL SECURITY")),
                "{table} must have RLS enabled"
            );
            assert!(
                compact.contains(&format!("ALTER TABLE {table} FORCE ROW LEVEL SECURITY")),
                "{table} must force RLS"
            );
        }
        // Events must never carry payload material: the digest is BYTEA(32)
        // metadata and no JSONB/TEXT body column exists on the outbox.
        assert!(
            compact.contains("event_digest BYTEA NOT NULL CHECK (octet_length(event_digest) = 32)"),
            "digest pinning missing"
        );
    }
}
