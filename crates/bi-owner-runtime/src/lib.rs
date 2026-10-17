// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=81ccdc638b683169e05b15ff2eb177b2592dfb35c1720e8e2cb6cc2dc3319100
MERGE (self:File {path:"crates/bi-owner-runtime/src/lib.rs"})
MERGE (module:Symbol {id:"crates/bi-owner-runtime/src/lib.rs::bi_owner_runtime",kind:"module"})
MERGE (inbox:Symbol {id:"crates/bi-owner-runtime/src/task_event_inbox.rs::task_event_inbox",kind:"module"})
MERGE (receiver:Symbol {id:"crates/bi-owner-runtime/src/task_event_receiver.rs::task_event_receiver",kind:"module"})
MERGE (quarantine:Symbol {id:"crates/bi-owner-runtime/src/task_event_quarantine.rs::task_event_quarantine",kind:"module"})
MERGE (query:Symbol {id:"crates/bi-owner-runtime/src/task_event_query.rs::task_event_query",kind:"module"})
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:IMPORTS]->(inbox)
MERGE (self)-[:IMPORTS]->(receiver)
MERGE (self)-[:IMPORTS]->(quarantine)
MERGE (self)-[:IMPORTS]->(query)
MERGE (query)-[:DEFINES]->(query_api:Type {id:"bi_owner_runtime::TaskEventQueryPort",kind:"trait"})
MERGE (query)-[:DEFINES]->(query_store:Type {id:"bi_owner_runtime::PgTaskEventQueryStore",kind:"struct"})
@endcypher
*/
//! Durable owner-local BI event ingestion.

pub mod task_event_inbox;
pub mod task_event_quarantine;
/// Bounded, allowlisted queries over observed BI Task event facts.
pub mod task_event_query;
pub mod task_event_receiver;

pub use task_event_inbox::{
    AuthenticatedTaskEventPrincipal, ReceiptOutcome, TaskEventInboxError, TaskEventInboxReceipt,
    TaskEventInboxStore, TaskEventRoutePin,
};

pub use task_event_receiver::{
    TaskEventInboxCommitPort, TaskEventQuarantineFault, TaskEventQuarantinePort,
    TaskEventQuarantineReceipt, TaskEventReceiveAuthorization, TaskEventReceiveDelivery,
    TaskEventReceiveError, TaskEventReceiveOrigin, TaskEventReceiveOutcome, TaskEventReceiveRoute,
    TaskEventReceiveTransport, TaskEventReceiver,
};

pub use task_event_quarantine::TaskEventQuarantineStore;
pub use task_event_query::{
    PgTaskEventQueryStore, TaskEventQueryCoverage, TaskEventQueryCoverageState,
    TaskEventQueryCursor, TaskEventQueryError, TaskEventQueryItem, TaskEventQueryPage,
    TaskEventQueryPageRequest, TaskEventQueryPort, MAX_TASK_EVENT_QUERY_PAGE_SIZE,
};
