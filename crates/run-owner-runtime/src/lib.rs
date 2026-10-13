//! Run-owner persistence and bounded Infrastructure result consumption.
/*
@cypher schema=1 source_sha256=205e7fb970ff646a6316ee28a8137d28759c165baddb7e4d4671a028878269eb
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
MERGE (module)-[:IMPORTS]->(catalog_file:File {path:"crates/run-owner-runtime/src/schedule_catalog.rs"})
MERGE (module)-[:EXPOSES]->(catalog_snapshot:Type {id:"run_owner_runtime::ScheduleExecutionSnapshot",kind:"struct"})
MERGE (module)-[:IMPORTS]->(policy_file:File {path:"crates/run-owner-runtime/src/schedule_policy.rs"})
MERGE (module)-[:EXPOSES]->(policy_snapshot:Type {id:"run_owner_runtime::SchedulePolicySnapshot",kind:"struct"})
MERGE (module)-[:IMPORTS]->(admission_file:File {path:"crates/run-owner-runtime/src/schedule_admission.rs"})
MERGE (module)-[:EXPOSES]->(admission_store:Type {id:"run_owner_runtime::PostgresScheduleAdmissionStore",kind:"struct"})
MERGE (module)-[:EXPOSES]->(admission_tx:Type {id:"run_owner_runtime::ScheduleAdmissionTransaction",kind:"struct"})
MERGE (module)-[:EXPOSES]->(admission_start:Type {id:"run_owner_runtime::ScheduleAdmissionStart",kind:"enum"})
MERGE (module)-[:EXPOSES]->(admission_error:Type {id:"run_owner_runtime::ScheduleAdmissionStoreError",kind:"enum"})
MERGE (module)-[:EXPOSES]->(admission_source:Type {id:"run_owner_runtime::ScheduleAdmissionSource",kind:"struct"})
MERGE (module)-[:IMPORTS]->(capacity_file:File {path:"crates/run-owner-runtime/src/schedule_capacity.rs"})
MERGE (module)-[:EXPOSES]->(capacity_snapshot:Type {id:"run_owner_runtime::ScheduleProjectCapacitySnapshot",kind:"struct"})
MERGE (receive_module:File {path:"crates/run-owner-runtime/src/admission_receive.rs"})
MERGE (self)-[:IMPORTS]->(receive_module)
MERGE (native_consumer_file:File {path:"crates/run-owner-runtime/src/native_schedule_admission_consumer.rs"})
MERGE (self)-[:IMPORTS]->(native_consumer_file)
MERGE (native_consumer:Type {id:"run_owner_runtime::NativeGrpcScheduleAdmissionConsumer",kind:"struct"})
MERGE (native_consumer_error:Type {id:"run_owner_runtime::NativeScheduleAdmissionConsumerError",kind:"enum"})
MERGE (native_capacity_consumer:Type {id:"run_owner_runtime::NativeGrpcCapacityResultConsumer",kind:"struct"})
MERGE (native_capacity_error:Type {id:"run_owner_runtime::NativeCapacityResultConsumerError",kind:"enum"})
MERGE (module)-[:EXPOSES]->(native_consumer)
MERGE (module)-[:EXPOSES]->(native_consumer_error)
MERGE (native_capacity_file:File {path:"crates/run-owner-runtime/src/native_capacity_result_consumer.rs"})
MERGE (self)-[:IMPORTS]->(native_capacity_file)
MERGE (module)-[:EXPOSES]->(native_capacity_consumer)
MERGE (module)-[:EXPOSES]->(native_capacity_error)
MERGE (native_consumer_feature:Config {id:"cargo.feature.run-owner-runtime.native-grpc-adapter"})
MERGE (module)-[:CONFIGURES]->(native_consumer_feature)
@endcypher
*/

mod admission_receive;
mod capacity_result;
#[cfg(feature = "native-grpc-adapter")]
mod native_capacity_result_consumer;
#[cfg(feature = "native-grpc-adapter")]
mod native_schedule_admission_consumer;
mod outbox;
mod run_infrastructure_binding;
mod schedule_admission;
mod schedule_capacity;
mod schedule_catalog;
mod schedule_policy;

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

pub use schedule_policy::SchedulePolicySnapshot;

pub use schedule_catalog::ScheduleExecutionSnapshot;

pub use schedule_capacity::ScheduleProjectCapacitySnapshot;

#[cfg(feature = "native-grpc-adapter")]
pub use native_schedule_admission_consumer::{
    NativeGrpcScheduleAdmissionConsumer, NativeScheduleAdmissionConsumerError,
};

#[cfg(feature = "native-grpc-adapter")]
pub use native_capacity_result_consumer::{
    NativeCapacityResultConsumerError, NativeGrpcCapacityResultConsumer,
};
