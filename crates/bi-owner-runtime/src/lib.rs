// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=5fc658d5a2fd0c017c4ae50ff7a0d93bdf7a3b567d4b06e2222bd2b1e0799ac3
MERGE (self:File {path:"crates/bi-owner-runtime/src/lib.rs"})
MERGE (module:Symbol {id:"crates/bi-owner-runtime/src/lib.rs::bi_owner_runtime",kind:"module"})
MERGE (inbox:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::task_event_inbox",kind:"module"})
MERGE (receiver:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::task_event_receiver",kind:"module"})
MERGE (quarantine:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::task_event_quarantine",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:IMPORTS]->(inbox)
MERGE (self)-[:IMPORTS]->(receiver)
MERGE (self)-[:IMPORTS]->(quarantine)
@endcypher
*/
//! Durable owner-local BI event ingestion.

pub mod task_event_inbox;
pub mod task_event_receiver;
pub mod task_event_quarantine;

pub use task_event_inbox::{
    AuthenticatedTaskEventPrincipal, ReceiptOutcome, TaskEventInboxError,
    TaskEventInboxReceipt, TaskEventInboxStore, TaskEventRoutePin,
};

pub use task_event_receiver::{
    TaskEventInboxCommitPort, TaskEventQuarantineFault, TaskEventQuarantinePort,
    TaskEventQuarantineReceipt, TaskEventReceiveAuthorization, TaskEventReceiveDelivery,
    TaskEventReceiveError, TaskEventReceiveOrigin, TaskEventReceiveOutcome,
    TaskEventReceiveRoute, TaskEventReceiveTransport, TaskEventReceiver,
};

pub use task_event_quarantine::TaskEventQuarantineStore;
