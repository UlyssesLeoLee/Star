//! Run-owner persistence and bounded Infrastructure result consumption.
/*
@cypher schema=1 source_sha256=8093aaac78a020168d2ef5ae53fc7299cb1d2bed018a6c0d39291a4d107f2498
MERGE (self:File {path:"crates/run-owner-runtime/src/lib.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/lib.rs::run_owner_runtime",kind:"module"})
MERGE (outbox:Type {id:"run_owner_runtime::PostgresRunOwnerCapacityStore",kind:"struct"})
MERGE (consumer:Type {id:"run_owner_runtime::RunOwnerCapacityResultConsumer",kind:"struct"})
MERGE (append_outcome:Type {id:"run_owner_runtime::CapacityDemandAppendOutcome",kind:"enum"})
MERGE (store_error:Type {id:"run_owner_runtime::RunOwnerCapacityStoreError",kind:"enum"})
MERGE (consume_outcome:Type {id:"run_owner_runtime::CapacityResultConsumeOutcome",kind:"enum"})
MERGE (consume_error:Type {id:"run_owner_runtime::CapacityResultError",kind:"enum"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(outbox)
MERGE (module)-[:DEFINES]->(consumer)
MERGE (module)-[:DEFINES]->(append_outcome)
MERGE (module)-[:DEFINES]->(store_error)
MERGE (module)-[:DEFINES]->(consume_outcome)
MERGE (module)-[:DEFINES]->(consume_error)
MERGE (module)-[:IMPORTS]->(outbox_file:File {path:"crates/run-owner-runtime/src/outbox.rs"})
MERGE (module)-[:IMPORTS]->(consumer_file:File {path:"crates/run-owner-runtime/src/capacity_result.rs"})
@endcypher
*/

mod capacity_result;
mod outbox;

pub use capacity_result::{
    CapacityResultConsumeOutcome, CapacityResultError, RunOwnerCapacityResultConsumer,
};
pub use outbox::{
    CapacityDemandAppendOutcome, PostgresRunOwnerCapacityStore, RunOwnerCapacityStoreError,
};
