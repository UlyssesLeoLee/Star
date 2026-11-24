// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"crates/star-api-rest/src/defect_outbox/mod.rs"})
MERGE (module:Symbol {id:"crates/star-api-rest/src/defect_outbox/mod.rs::defect_outbox",kind:"module"})
MERGE (contract:Symbol {id:"crates/star-api-rest/src/defect_outbox/contract.rs::module",kind:"module"})
MERGE (consumer:Symbol {id:"crates/star-api-rest/src/defect_outbox/consumer.rs::module",kind:"module"})
MERGE (relay:Symbol {id:"crates/star-api-rest/src/defect_outbox/relay.rs::module",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:CONTAINS]->(contract),(module)-[:CONTAINS]->(consumer),(module)-[:CONTAINS]->(relay)
MERGE (self)-[:CREATES]->(:Table {name:"defect.outbox",kind:"table"})
MERGE (self)-[:CREATES]->(:Table {name:"defect.outbox_checkpoint",kind:"table"})
MERGE (self)-[:CREATES]->(:Table {name:"defect.notification_projection",kind:"table"})
@cypher
*/
//! Defect Outbox → Notification owner contract
//!
//! Owner boundaries enforced by construction:
//! - The Defect owner writes the defect fact + audit + Outbox event in ONE local transaction
//!   (`domain_defect` via `PgDefectService`).
//! - The Notification owner consumes events through its own Inbox + idempotent receipt
//!   + local projection + durable checkpoint, committing receipt/projection/cursor
//!   atomically in ONE transaction.
//! - The Notification projection only accelerates notification delivery;
//!   no event is a valid notification — positive decisions come only from a
//!   fresh authorization recheck.
//! - Relay: at-least-once, bounded backoff, cancel-only-between-transactions,
//!   drain-before-profile-switch discipline.
//!
//! Delivery semantics are at-least-once; nothing claims exactly-once.
//! DDL lives at `db/migrations/2026-11-24-defect-outbox-notification.sql` and is
//! applied by the repository migration runner (one statement per line).

pub mod contract;
pub mod consumer;
pub mod relay;

pub use contract::{
    decide_apply, event_digest, owner_stream_id, validate_envelope, ApplyDecision,
    DefectOutboxEvent, EnvelopeError, InboxCursor, ReceivedEvent, StoredInboxRow,
    SuspensionReason, CONSUMER_ID, EVENT_DIGEST_LEN,
    EVENT_TYPE_DEFECT_CREATED, EVENT_TYPE_DEFECT_TRANSITIONED,
    EVENT_TYPE_VERIFICATION_SUBMITTED, EVENT_TYPE_VERIFICATION_PASSED,
    EVENT_TYPE_VERIFICATION_FAILED,
    PRODUCER_ID, SCHEMA_VERSION,
};
pub use consumer::{
    ApplyOutcome, DefectInboxError, DefectInboxStore,
    InMemoryDefectInbox, PgDefectInboxStore,
};
pub use relay::{BoundedRelay, PendingDelivery, ProfileSwitchOutcome, RelayBackoffPolicy};
