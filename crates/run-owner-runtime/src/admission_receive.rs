/*
@cypher schema=1 source_sha256=293f7f8f8c5f086b04e7293b49eaf04524c3ff30e225a5d88513774a2f1234bf
MERGE (self:File {path:"crates/run-owner-runtime/src/admission_receive.rs"})
MERGE (handler:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveHandler",kind:"trait"})
MERGE (store:Type {id:"run_owner_runtime::PostgresScheduleAdmissionStore",kind:"struct"})
MERGE (commit:Symbol {id:"crates/run-owner-runtime/src/admission_receive.rs::PostgresScheduleAdmissionStore::commit",kind:"method"})
MERGE (self)-[:DEFINES]->(commit)
MERGE (store)-[:IMPLEMENTS]->(handler)
MERGE (commit)-[:CALLS]->(begin:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore::begin",kind:"method"})
MERGE (commit)-[:CALLS]->(prepare:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction::prepare_and_commit",kind:"method"})
MERGE (commit)-[:CALLS]->(confirm:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore::confirm_admission",kind:"method"})
MERGE (commit)-[:CALLS]->(compensate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore::compensate_admission",kind:"method"})
@endcypher
*/
//! Run owner handler; transport and identity verification are supplied separately.
use domain_automation::schedule::{
    ScheduleAdmissionResolutionKindV1, ScheduleRunAdmissionResolutionV1,
};
use platform_messaging_runtime::admission_receive::{
    AdmissionReceiveError, AdmissionReceiveHandler, AdmissionReceiveKind, AdmissionReceiveScope,
};
use uuid::Uuid;

use crate::{PostgresScheduleAdmissionStore, ScheduleAdmissionSource, ScheduleAdmissionStart};

impl AdmissionReceiveHandler for PostgresScheduleAdmissionStore {
    async fn commit(
        &self,
        scope: AdmissionReceiveScope,
        wire: &[u8],
    ) -> Result<Uuid, AdmissionReceiveError> {
        let source = ScheduleAdmissionSource {
            tenant_id: scope.tenant_id,
            schedule_service_id: scope.sender_service_id,
        };
        match scope.kind {
            AdmissionReceiveKind::Request => {
                let result = match self
                    .begin(source, wire)
                    .await
                    .map_err(|_| AdmissionReceiveError::OwnerCommit)?
                {
                    ScheduleAdmissionStart::Replay(result) => result,
                    ScheduleAdmissionStart::New(transaction) => transaction
                        .prepare_and_commit()
                        .await
                        .map_err(|_| AdmissionReceiveError::OwnerCommit)?,
                };
                Ok(result.request_id)
            }
            AdmissionReceiveKind::Resolution => {
                let resolution = ScheduleRunAdmissionResolutionV1::decode_json(wire)
                    .map_err(|_| AdmissionReceiveError::Contract)?;
                match resolution.decision {
                    ScheduleAdmissionResolutionKindV1::Confirm => {
                        self.confirm_admission(source, wire)
                            .await
                            .map_err(|_| AdmissionReceiveError::OwnerCommit)?;
                    }
                    ScheduleAdmissionResolutionKindV1::Compensate => {
                        self.compensate_admission(source, wire)
                            .await
                            .map_err(|_| AdmissionReceiveError::OwnerCommit)?;
                    }
                }
                Ok(resolution.resolution_id)
            }
            AdmissionReceiveKind::Result => Err(AdmissionReceiveError::Contract),
        }
    }
}
