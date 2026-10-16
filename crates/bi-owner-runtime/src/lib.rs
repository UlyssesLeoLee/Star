// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=1a68cd3edee95516e59adea4da34c977159bb667189bf7eadd1e4d9d0516951f
MERGE (self:File {path:"crates/bi-owner-runtime/src/lib.rs"})
MERGE (module:Symbol {id:"crates/bi-owner-runtime/src/lib.rs::bi_owner_runtime",kind:"module"})
MERGE (inbox:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::task_event_inbox",kind:"module"})
MERGE (receiver:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::task_event_receiver",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:IMPORTS]->(inbox)
MERGE (self)-[:IMPORTS]->(receiver)
@endcypher
*/
//! Durable owner-local BI event ingestion.

pub mod task_event_inbox;
pub mod task_event_receiver;

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
