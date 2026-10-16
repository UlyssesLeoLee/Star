// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=1bb3763e9a56715523e3bb73368ef91fe20b38039fab7018ee53b259fa67c886
MERGE (self:File {path:"crates/bi-owner-runtime/src/lib.rs"})
MERGE (module:Symbol {id:"crates/bi-owner-runtime/src/lib.rs::bi_owner_runtime",kind:"module"})
MERGE (inbox:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::task_event_inbox",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:IMPORTS]->(inbox)
@endcypher
*/
//! Durable owner-local BI event ingestion.

pub mod task_event_inbox;

pub use task_event_inbox::{
    AuthenticatedTaskEventPrincipal, ReceiptOutcome, TaskEventInboxError,
    TaskEventInboxReceipt, TaskEventInboxStore, TaskEventRoutePin,
};
