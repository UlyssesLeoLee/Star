//! Run-owner persistence and bounded Infrastructure result consumption.
/*
@cypher schema=1 source_sha256=e0d49527cd1a7198033de8bb79aeadb0eceb12d6ef9ed78a30dffd8f45782814
MERGE (self:File {path:"crates/run-owner-runtime/src/lib.rs"})
MERGE (module:Symbol {id:"crates/run-owner-runtime/src/lib.rs::run_owner_runtime",kind:"module"})
MERGE (binding_module:Symbol {id:"crates/run-owner-runtime/src/run_infrastructure_binding.rs::module",kind:"module"})
MERGE (outbox:Type {id:"run_owner_runtime::PostgresRunOwnerCapacityStore",kind:"struct"})
MERGE (consumer:Type {id:"run_owner_runtime::RunOwnerCapacityResultConsumer",kind:"struct"})
MERGE (binding_store:Type {id:"run_owner_runtime::PostgresRunInfrastructureBindingStore",kind:"struct"})
MERGE (binding_scope:Type {id:"run_owner_runtime::RunInfrastructureBindingScope",kind:"struct"})
MERGE (resolved_binding:Type {id:"run_owner_runtime::ResolvedRunInfrastructureBinding",kind:"struct"})
MERGE (binding_error:Type {id:"run_owner_runtime::RunInfrastructureBindingStoreError",kind:"enum"})
MERGE (append_outcome:Type {id:"run_owner_runtime::CapacityDemandAppendOutcome",kind:"enum"})
MERGE (store_error:Type {id:"run_owner_runtime::RunOwnerCapacityStoreError",kind:"enum"})
MERGE (consume_outcome:Type {id:"run_owner_runtime::CapacityResultConsumeOutcome",kind:"enum"})
MERGE (consume_error:Type {id:"run_owner_runtime::CapacityResultError",kind:"enum"})
MERGE (self)-[:DEFINES]->(module)
MERGE (module)-[:DEFINES]->(outbox)
MERGE (module)-[:DEFINES]->(consumer)
MERGE (module)-[:DEFINES]->(binding_store)
MERGE (module)-[:DEFINES]->(binding_scope)
MERGE (module)-[:DEFINES]->(resolved_binding)
MERGE (module)-[:DEFINES]->(binding_error)
MERGE (module)-[:DEFINES]->(append_outcome)
MERGE (module)-[:DEFINES]->(store_error)
MERGE (module)-[:DEFINES]->(consume_outcome)
MERGE (module)-[:DEFINES]->(consume_error)
MERGE (module)-[:IMPORTS]->(outbox_file:File {path:"crates/run-owner-runtime/src/outbox.rs"})
MERGE (module)-[:IMPORTS]->(consumer_file:File {path:"crates/run-owner-runtime/src/capacity_result.rs"})
MERGE (module)-[:IMPORTS]->(binding_file:File {path:"crates/run-owner-runtime/src/run_infrastructure_binding.rs"})
MERGE (module)-[:IMPORTS]->(admission_file:File {path:"crates/run-owner-runtime/src/schedule_admission.rs"})
MERGE (module)-[:EXPOSES]->(admission_store:Type {id:"run_owner_runtime::PostgresScheduleAdmissionStore",kind:"struct"})
MERGE (module)-[:EXPOSES]->(admission_tx:Type {id:"run_owner_runtime::ScheduleAdmissionTransaction",kind:"struct"})
MERGE (module)-[:EXPOSES]->(admission_start:Type {id:"run_owner_runtime::ScheduleAdmissionStart",kind:"enum"})
MERGE (module)-[:EXPOSES]->(admission_error:Type {id:"run_owner_runtime::ScheduleAdmissionStoreError",kind:"enum"})
MERGE (module)-[:EXPOSES]->(admission_source:Type {id:"run_owner_runtime::ScheduleAdmissionSource",kind:"struct"})
@endcypher
*/

mod capacity_result;
mod outbox;
mod run_infrastructure_binding;
mod schedule_admission;

pub use capacity_result::{
    CapacityResultConsumeOutcome, CapacityResultError, RunOwnerCapacityResultConsumer,
};
pub use outbox::{
    CapacityDemandAppendOutcome, PostgresRunOwnerCapacityStore, RunOwnerCapacityStoreError,
};
pub use run_infrastructure_binding::{
    PostgresRunInfrastructureBindingStore, ResolvedRunInfrastructureBinding,
    RunInfrastructureBindingScope, RunInfrastructureBindingStoreError,
};
pub use schedule_admission::{
    PostgresScheduleAdmissionStore, ScheduleAdmissionSource, ScheduleAdmissionStart,
    ScheduleAdmissionStoreError, ScheduleAdmissionTransaction,
};
