/*
@cypher schema=1 source_sha256=d289bbb793ad6ecf0a3aa4256d459f5f3885e6ca0f2d397d281258ad226929b9
MERGE (self:File {path:"crates/star-pg-adapter/src/repository/admission_receive.rs"})
MERGE (handler:Type {id:"platform_messaging_runtime::admission_receive::AdmissionReceiveHandler",kind:"trait"})
MERGE (store:Type {id:"star_pg_adapter::repository::PgAutomationScheduleRepository",kind:"struct"})
MERGE (commit:Symbol {id:"crates/star-pg-adapter/src/repository/admission_receive.rs::PgAutomationScheduleRepository::commit",kind:"method"})
MERGE (self)-[:DEFINES]->(commit)
MERGE (store)-[:IMPLEMENTS]->(handler)
MERGE (commit)-[:CALLS]->(resolve:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::resolve_admission_result",kind:"method"})
@endcypher
*/
//! Schedule owner result handler; a successful return follows the owner transaction commit.
use domain_automation::schedule::ScheduleRunAdmissionResultV1;
use platform_messaging_runtime::admission_receive::{
    AdmissionReceiveError, AdmissionReceiveHandler, AdmissionReceiveKind, AdmissionReceiveScope,
};
use uuid::Uuid;

use super::PgAutomationScheduleRepository;

impl AdmissionReceiveHandler for PgAutomationScheduleRepository {
    async fn commit(
        &self,
        scope: AdmissionReceiveScope,
        wire: &[u8],
    ) -> Result<Uuid, AdmissionReceiveError> {
        if scope.kind != AdmissionReceiveKind::Result {
            return Err(AdmissionReceiveError::Contract);
        }
        let result = ScheduleRunAdmissionResultV1::decode_json(wire)
            .map_err(|_| AdmissionReceiveError::Contract)?;
        self.resolve_admission_result(scope.tenant_id, scope.sender_service_id, wire)
            .await
            .map_err(|_| AdmissionReceiveError::OwnerCommit)?;
        Ok(result.result_id)
    }
}
