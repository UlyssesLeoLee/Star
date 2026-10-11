//! Real PostgreSQL owner-local admission receipt/replay/rollback verification.
/*
@cypher schema=1 source_sha256=872105ee1e6512381a058fb3fa74e9dd7f6a21afa08886efea9013b7c5bbbb03
MERGE (self:File {path:"crates/run-owner-runtime/tests/schedule_admission_pg.rs"})
MERGE (test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::admission_receipt_replay_conflict_rollback_and_scope",kind:"test"})
MERGE (fixture:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::command_fixture",kind:"function"})
MERGE (binding:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::binding_fixture",kind:"function"})
MERGE (source:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::source",kind:"function"})
MERGE (test)-[:CALLS]->(seed_policy:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::seed_policy",kind:"function"})
MERGE (self)-[:DEFINES]->(seed_policy)
MERGE (seed_catalog:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::seed_catalogs",kind:"function"})
MERGE (self)-[:DEFINES]->(seed_catalog)
MERGE (test)-[:CALLS]->(seed_catalog)
MERGE (seed_catalog)-[:WRITES]->(agent_execution_catalog_revision:Table {id:"multica.agent_execution_catalog_revision"})
MERGE (seed_catalog)-[:WRITES]->(agent_execution_provider_catalog:Table {id:"multica.agent_execution_provider_catalog"})
MERGE (seed_catalog)-[:WRITES]->(agent_execution_provider_capability:Table {id:"multica.agent_execution_provider_capability"})
MERGE (seed_catalog)-[:WRITES]->(agent_execution_skill_catalog:Table {id:"multica.agent_execution_skill_catalog"})
MERGE (seed_catalog)-[:WRITES]->(agent_execution_skill_capability:Table {id:"multica.agent_execution_skill_capability"})
MERGE (seed_catalog)-[:WRITES]->(agent_execution_grant_set:Table {id:"multica.agent_execution_grant_set"})
MERGE (seed_catalog)-[:WRITES]->(agent_execution_grant_capability:Table {id:"multica.agent_execution_grant_capability"})
MERGE (test)-[:CALLS]->(catalog_gate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_execution_catalogs",kind:"method"})
MERGE (seed_policy)-[:WRITES]->(profile_master:Table {id:"multica.agent_execution_profile"})
MERGE (seed_policy)-[:WRITES]->(hook_master:Table {id:"multica.hook_policy_set"})
MERGE (test)-[:CALLS]->(policy_gate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_profile_and_hooks",kind:"method"})
MERGE (test)-[:CALLS]->(task_seed:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::seed_task_binding",kind:"function"})
MERGE (self)-[:DEFINES]->(task_seed)
MERGE (task_seed)-[:WRITES]->(w:Table {id:"public.worktree_canvas_worktree"})
MERGE (task_seed)-[:WRITES]->(p:Table {id:"multica.worktree_project_binding"})
MERGE (task_seed)-[:WRITES]->(b:Table {id:"multica.engineering_run_worktree_binding"})
MERGE (task_seed)-[:WRITES]->(m:Table {id:"multica.task_metadata"})
MERGE (task_seed)-[:WRITES]->(l:Table {id:"multica.work_item_worktree"})
MERGE (test)-[:CALLS]->(task_check:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_task_binding",kind:"method"})
MERGE (test)-[:CALLS]->(source)
MERGE (self)-[:DEFINES]->(source)
MERGE (seed:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::seed_provisional_run",kind:"function"})
MERGE (test)-[:CALLS]->(seed)
MERGE (self)-[:DEFINES]->(seed)
MERGE (result:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::rejected_result",kind:"function"})
MERGE (begin:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore.begin",kind:"method"})
MERGE (commit:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.commit_result",kind:"method"})
MERGE (self)-[:DEFINES]->(test)
MERGE (directory_seed:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::seed_directory",kind:"function"})
MERGE (revalidate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_directory_and_binding",kind:"method"})
MERGE (self)-[:DEFINES]->(directory_seed)
MERGE (test)-[:CALLS]->(directory_seed)
MERGE (test)-[:CALLS]->(revalidate)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (self)-[:DEFINES]->(binding)
MERGE (self)-[:DEFINES]->(result)
MERGE (test)-[:CALLS]->(fixture)
MERGE (test)-[:CALLS]->(result)
MERGE (test)-[:CALLS]->(begin)
MERGE (test)-[:CALLS]->(commit)
MERGE (fixture)-[:CALLS]->(binding)
MERGE (capacity_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_project_capacity",kind:"function"})
MERGE (self)-[:DEFINES]->(capacity_test)
MERGE (test)-[:CALLS]->(capacity_test)
MERGE (capacity_test)-[:CALLS]->(capacity_gate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.revalidate_project_capacity",kind:"method"})
MERGE (capacity_test)-[:READS]->(allocation:Table {id:"multica.project_execution_resource_admission_lock"})
MERGE (capacity_test)-[:WRITES]->(allocation)
MERGE (capacity_test)-[:WRITES]->(quota:Table {id:"multica.project_execution_resource_quota"})
MERGE (capacity_test)-[:WRITES]->(reservation:Table {id:"multica.project_execution_resource_reservation"})
MERGE (capacity_test)-[:CALLS]->(seed)
MERGE (preparation_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_schedule_preparation",kind:"function"})
MERGE (self)-[:DEFINES]->(preparation_test)
MERGE (capacity_test)-[:CALLS]->(preparation_test)
MERGE (preparation_test)-[:TESTS]->(prepare:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::ScheduleAdmissionTransaction.prepare_and_commit",kind:"method"})
MERGE (preparation_test)-[:WRITES]->(task_lifecycle:Table {id:"multica.task_lifecycle_current"})
MERGE (preparation_test)-[:WRITES]->(task_contract:Table {id:"multica.task_contract"})
MERGE (preparation_test)-[:READS]->(run:Table {id:"multica.task_execution_run"})
MERGE (preparation_test)-[:READS]->(reservation)
MERGE (preparation_test)-[:CALLS]->(compensation_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_schedule_compensation",kind:"function"})
MERGE (self)-[:DEFINES]->(compensation_test)
MERGE (compensation_test)-[:TESTS]->(compensate:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore.compensate_admission",kind:"method"})
MERGE (compensation_test)-[:READS]->(reservation)
MERGE (compensation_test)-[:READS]->(receipt:Table {id:"multica.schedule_compensation_inbox"})
MERGE (compensation_test)-[:CALLS]->(append:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::PostgresRunOwnerCapacityStore.append_capacity_demand",kind:"method"})
MERGE (compensation_test)-[:WRITES]->(reservation)
MERGE (compensation_test)-[:WRITES]->(events:Table {id:"multica.task_execution_run_event"})
MERGE (preparation_test)-[:CALLS]->(confirmation_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_schedule_confirmation",kind:"function"})
MERGE (self)-[:DEFINES]->(confirmation_test)
MERGE (confirmation_test)-[:TESTS]->(confirm:Symbol {id:"crates/run-owner-runtime/src/schedule_admission.rs::PostgresScheduleAdmissionStore.confirm_admission",kind:"method"})
MERGE (confirmation_test)-[:CALLS]->(prepare)
MERGE (confirmation_test)-[:READS]->(reservation)
MERGE (confirmation_test)-[:WRITES]->(reservation)
MERGE (confirmation_test)-[:READS]->(confirmation_receipt:Table {id:"multica.schedule_confirmation_inbox"})
MERGE (confirmation_test)-[:READS]->(demand_outbox:Table {id:"multica.infrastructure_capacity_demand_outbox"})
MERGE (confirmation_test)-[:WRITES]->(project_grant:Table {id:"permission.project_role_binding"})
MERGE (confirmation_test)-[:WRITES]->(quota)
MERGE (schedule_seed:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::seed_schedule_handoff",kind:"function"})
MERGE (schedule_retry:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_schedule_rejection_retry",kind:"function"})
MERGE (schedule_fence:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_schedule_fence_compensation",kind:"function"})
MERGE (self)-[:DEFINES]->(schedule_seed)
MERGE (self)-[:DEFINES]->(schedule_retry)
MERGE (self)-[:DEFINES]->(schedule_fence)
MERGE (confirmation_test)-[:CALLS]->(schedule_seed)
MERGE (confirmation_test)-[:CALLS]->(schedule_retry)
MERGE (confirmation_test)-[:CALLS]->(schedule_fence)
MERGE (schedule_fence)-[:CALLS]->(schedule_seed)
MERGE (schedule_retry)-[:CALLS]->(schedule_seed)
MERGE (schedule_seed)-[:WRITES]->(schedule_rule:Table {id:"automation.schedule_rule_revision"})
MERGE (schedule_seed)-[:WRITES]->(schedule_occurrence:Table {id:"automation.occurrence"})
MERGE (schedule_seed)-[:WRITES]->(schedule_dispatch:Table {id:"automation.occurrence_dispatch"})
MERGE (schedule_seed)-[:CALLS]->(schedule_claim:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::claim_due_occurrences",kind:"method"})
MERGE (confirmation_test)-[:CALLS]->(schedule_queue:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::persist_admission_request",kind:"method"})
MERGE (confirmation_test)-[:CALLS]->(schedule_resolve:Symbol {id:"crates/star-pg-adapter/src/repository/automation_schedule.rs::PgAutomationScheduleRepository::resolve_admission_result",kind:"method"})
MERGE (schedule_retry)-[:CALLS]->(schedule_queue)
MERGE (schedule_retry)-[:CALLS]->(schedule_resolve)
MERGE (schedule_retry)-[:CALLS]->(schedule_claim)
MERGE (schedule_fence)-[:CALLS]->(schedule_queue)
MERGE (schedule_fence)-[:CALLS]->(schedule_resolve)
MERGE (schedule_fence)-[:CALLS]->(schedule_claim)
MERGE (delivery_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_admission_delivery",kind:"function"})
MERGE (self)-[:DEFINES]->(delivery_test)
MERGE (schedule_retry)-[:CALLS]->(delivery_test)
MERGE (delivery_test)-[:USES_TYPE]->(delivery_store:Type {id:"platform_messaging_runtime::AdmissionDeliveryStore",kind:"struct"})
MERGE (delivery_test)-[:USES_TYPE]->(delivery_publisher:Type {id:"platform_messaging_runtime::AdmissionJetStreamPublisher",kind:"struct"})
MERGE (transport_profile:Type {id:"platform_messaging_runtime::AdmissionTransportProfile",kind:"struct"})
MERGE (transport_receipt:Type {id:"platform_messaging_runtime::AdmissionTransportReceipt",kind:"struct"})
MERGE (delivery_test)-[:USES_TYPE]->(transport_profile)
MERGE (delivery_test)-[:USES_TYPE]->(transport_receipt)
MERGE (delivery_test)-[:CALLS]->(profile_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportProfile::new",kind:"method"})
MERGE (delivery_test)-[:CALLS]->(receipt_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_delivery.rs::AdmissionTransportReceipt::new",kind:"method"})
MERGE (delivery_test)-[:WRITES]->(schedule_delivery:Table {id:"automation.admission_message_delivery"})
MERGE (delivery_test)-[:WRITES]->(run_delivery:Table {id:"multica.admission_message_delivery"})
MERGE (schedule_fence)-[:USES_TYPE]->(delivery_store)
MERGE (schedule_fence)-[:USES_TYPE]->(delivery_publisher)
MERGE (schedule_fence)-[:USES_TYPE]->(transport_profile)
MERGE (schedule_fence)-[:CALLS]->(profile_new)
MERGE (schedule_fence)-[:READS]->(schedule_delivery_receipt:Table {id:"automation.admission_message_delivery_receipt"})
MERGE (delivery_test)-[:READS]->(schedule_delivery_receipt)
MERGE (delivery_test)-[:READS]->(run_delivery_receipt:Table {id:"multica.admission_message_delivery_receipt"})
MERGE (d14_receive_fixture:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::receive_admission_broker_fixture",kind:"function"})
MERGE (d14_quarantine_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_admission_quarantine",kind:"function"})
MERGE (d14_auth_fixture:Type {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionAuthorizationFixture",kind:"struct"})
MERGE (d14_verify_fixture:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionAuthorizationFixture::verify",kind:"method"})
MERGE (d14_positive:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_schedule_confirmation",kind:"function"})
MERGE (self)-[:DEFINES]->(d14_receive_fixture)
MERGE (self)-[:DEFINES]->(d14_quarantine_test)
MERGE (self)-[:DEFINES]->(d14_auth_fixture)
MERGE (self)-[:DEFINES]->(d14_verify_fixture)
MERGE (d14_positive)-[:CALLS]->(d14_receive_fixture)
MERGE (d14_receive_fixture)-[:TESTS]->(d14_consume:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::consume_one",kind:"method"})
MERGE (d14_receive_fixture)-[:CALLS]->(d14_consume)
MERGE (test)-[:CALLS]->(d14_quarantine_test)
MERGE (d14_quarantine_test)-[:USES_TYPE]->(quarantine_store:Type {id:"platform_messaging_runtime::PgAdmissionReceiveQuarantine",kind:"struct"})
MERGE (d14_quarantine_test)-[:WRITES]->(quarantine_receipt:Table {id:"platform_messaging.admission_receive_quarantine_receipt"})
MERGE (d14_auth_fixture)-[:IMPLEMENTS]->(d14_auth:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization",kind:"trait"})
MERGE (d14_receive_fixture)-[:CALLS]->(d14_transport_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive_jetstream.rs::JetStreamAdmissionReceiveTransport::new",kind:"method"})
MERGE (d16_receive_test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::verify_actual_quarantine_receiver",kind:"function"})
MERGE (d16_state:Type {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionQuarantineReceiveTestState",kind:"struct"})
MERGE (d16_delivery:Type {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionQuarantineReceiveTestDelivery",kind:"struct"})
MERGE (d16_transport:Type {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionQuarantineReceiveTestTransport",kind:"struct"})
MERGE (d16_auth:Type {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionQuarantineReceiveTestAuthorization",kind:"struct"})
MERGE (d16_handler:Type {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::AdmissionQuarantineReceiveTestHandler",kind:"struct"})
MERGE (self)-[:DEFINES]->(d16_receive_test)
MERGE (self)-[:DEFINES]->(d16_state)
MERGE (self)-[:DEFINES]->(d16_delivery)
MERGE (self)-[:DEFINES]->(d16_transport)
MERGE (self)-[:DEFINES]->(d16_auth)
MERGE (self)-[:DEFINES]->(d16_handler)
MERGE (d16_receive_test)-[:CALLS]->(receiver_new:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::new",kind:"method"})
MERGE (d16_receive_test)-[:CALLS]->(receiver_consume:Symbol {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiver::consume_one",kind:"method"})
MERGE (d16_delivery)-[:IMPLEMENTS]->(receive_delivery:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceivedDelivery",kind:"trait"})
MERGE (d16_transport)-[:IMPLEMENTS]->(receive_transport:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveTransport",kind:"trait"})
MERGE (d16_auth)-[:IMPLEMENTS]->(receive_auth:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveAuthorization",kind:"trait"})
MERGE (d16_handler)-[:IMPLEMENTS]->(receive_handler:Type {id:"crates/platform-messaging-runtime/src/admission_receive.rs::AdmissionReceiveHandler",kind:"trait"})
MERGE (d16_delivery)-[:READS]->(quarantine_receipt)
MERGE (d16_receive_test)-[:USES_TYPE]->(quarantine_store)
MERGE (d16_receive_test)-[:READS]->(quarantine_receipt)
@endcypher
*/
use chrono::Utc;
use domain_automation::{schedule::*, RuleId};
use domain_infrastructure::{
    ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, KubernetesDnsLabel, ProviderKind,
    RunInfrastructureBindingSnapshotV2,
};
use run_owner_runtime::{
    PostgresRunInfrastructureBindingStore, PostgresScheduleAdmissionStore, ScheduleAdmissionSource,
    ScheduleAdmissionStart, ScheduleAdmissionStoreError,
};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires phase9f4d_run_owner_consumer.py --schema-only --admission-tests"]
async fn admission_receipt_replay_conflict_rollback_and_scope(
) -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("STAR_RUN_OWNER_TEST_DATABASE_URL")?;
    let admin = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await?;
    verify_admission_quarantine(&admin).await?;
    sqlx::raw_sql(
        "CREATE ROLE star_admission_test_login LOGIN NOINHERIT NOSUPERUSER NOBYPASSRLS;
        GRANT star_multica_run_owner TO star_admission_test_login;
        ALTER ROLE star_admission_test_login SET statement_timeout = '10s';",
    )
    .execute(&admin)
    .await?;
    let options: PgConnectOptions = url.parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options.username("star_admission_test_login"))
        .await?;
    let store = PostgresScheduleAdmissionStore::new(pool.clone());
    let command = command_fixture();
    let wire = command.encode_json()?;
    assert!(matches!(
        store.begin(source(Uuid::new_v4()), &wire).await,
        Err(ScheduleAdmissionStoreError::UntrustedScope)
    ));
    assert!(matches!(
        store.begin(source(command.tenant_id), b"{}").await,
        Err(ScheduleAdmissionStoreError::Contract(_))
    ));
    let result = rejected_result(&command)?;
    let ScheduleAdmissionStart::New(pending) =
        store.begin(source(command.tenant_id), &wire).await?
    else {
        panic!("new request")
    };
    drop(pending);
    let ScheduleAdmissionStart::New(pending) =
        store.begin(source(command.tenant_id), &wire).await?
    else {
        panic!("rollback must permit retry")
    };
    pending.commit_result(&result).await?;
    let ScheduleAdmissionStart::Replay(replay) =
        store.begin(source(command.tenant_id), &wire).await?
    else {
        panic!("exact duplicate must recover result")
    };
    assert_eq!(replay, result);
    assert!(matches!(
        store
            .begin(
                ScheduleAdmissionSource {
                    tenant_id: command.tenant_id,
                    schedule_service_id: Uuid::new_v4()
                },
                &wire
            )
            .await,
        Err(ScheduleAdmissionStoreError::ReplayConflict)
    ));
    let alternate = serde_json::to_vec_pretty(&command)?;
    assert!(matches!(
        store.begin(source(command.tenant_id), &alternate).await,
        Err(ScheduleAdmissionStoreError::ReplayConflict)
    ));
    let mut changed = command.clone();
    changed.correlation_id = Uuid::new_v4();
    assert!(matches!(
        store
            .begin(source(command.tenant_id), &changed.encode_json()?)
            .await,
        Err(ScheduleAdmissionStoreError::ReplayConflict)
    ));
    let mut other_tenant = command_fixture();
    other_tenant.request_id = command.request_id;
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(other_tenant.tenant_id), &other_tenant.encode_json()?)
        .await?
    else {
        panic!("tenant scope isolation")
    };
    drop(pending);

    let concurrent = command_fixture();
    let concurrent_wire = concurrent.encode_json()?;
    let concurrent_result = rejected_result(&concurrent)?;
    let deliver = || async {
        match store
            .begin(source(concurrent.tenant_id), &concurrent_wire)
            .await?
        {
            ScheduleAdmissionStart::New(pending) => {
                pending.commit_result(&concurrent_result).await?;
                Ok(false)
            }
            ScheduleAdmissionStart::Replay(replay) => {
                assert_eq!(replay, concurrent_result);
                Ok(true)
            }
        }
    };
    let (one, two): (
        Result<bool, ScheduleAdmissionStoreError>,
        Result<bool, ScheduleAdmissionStoreError>,
    ) = tokio::join!(deliver(), deliver());
    assert_ne!(one?, two?);

    // A result INSERT failure must roll back its just-inserted receipt.
    let accepted = command_fixture();
    let mut accepted_result = rejected_result(&accepted)?;
    accepted_result.outcome = ScheduleRunAdmissionOutcomeV1::Accepted;
    accepted_result.rejection_code = None;
    accepted_result.task_execution_run_id = Some(Uuid::new_v4());
    let accepted_wire = accepted.encode_json()?;
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(accepted.tenant_id), &accepted_wire)
        .await?
    else {
        panic!("new accepted request")
    };
    assert!(matches!(
        pending.commit_result(&accepted_result).await,
        Err(ScheduleAdmissionStoreError::Database(_))
    ));
    let absent: i64=sqlx::query_scalar("SELECT count(*) FROM multica.schedule_admission_inbox WHERE tenant_id=$1 AND request_id=$2")
        .bind(accepted.tenant_id).bind(accepted.request_id).fetch_one(&admin).await?;
    assert_eq!(absent, 0);
    // Disposable admin supplies a matching provisional Run solely to exercise the
    // accepted-result storage guard, not production authorization or reservation.
    seed_provisional_run(
        &admin,
        &accepted,
        accepted_result.task_execution_run_id.unwrap(),
    )
    .await?;
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(accepted.tenant_id), &accepted_wire)
        .await?
    else {
        panic!("failed insert rolled back")
    };
    pending.commit_result(&accepted_result).await?;
    assert!(
        matches!(store.begin(source(accepted.tenant_id),&accepted_wire).await?,ScheduleAdmissionStart::Replay(r) if r==accepted_result)
    );
    let started = command_fixture();
    let started_run_id = Uuid::new_v4();
    seed_provisional_run(&admin, &started, started_run_id).await?;
    sqlx::query(
        "INSERT INTO multica.task_execution_run_event
        (tenant_id,project_id,work_item_id,run_id,event_type,correlation_id)
        VALUES($1,$2,$3,$4,'run_started',$5)",
    )
    .bind(started.tenant_id)
    .bind(started.occurrence.rule_snapshot.target.project_id)
    .bind(started.occurrence.rule_snapshot.target.work_item_id)
    .bind(started_run_id)
    .bind(started.correlation_id)
    .execute(&admin)
    .await?;
    let mut started_result = rejected_result(&started)?;
    started_result.outcome = ScheduleRunAdmissionOutcomeV1::Accepted;
    started_result.rejection_code = None;
    started_result.task_execution_run_id = Some(started_run_id);
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(started.tenant_id), &started.encode_json()?)
        .await?
    else {
        panic!("new started request")
    };
    assert!(matches!(
        pending.commit_result(&started_result).await,
        Err(ScheduleAdmissionStoreError::Database(_))
    ));
    // One different request cannot claim the same occurrence/attempt.
    let mut duplicate_attempt = command.clone();
    duplicate_attempt.request_id = Uuid::new_v4();
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(command.tenant_id), &duplicate_attempt.encode_json()?)
        .await?
    else {
        panic!("new request identity")
    };
    assert!(matches!(
        pending
            .commit_result(&rejected_result(&duplicate_attempt)?)
            .await,
        Err(ScheduleAdmissionStoreError::Database(_))
    ));
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM multica.schedule_admission_inbox WHERE tenant_id=$1",
    )
    .bind(command.tenant_id)
    .fetch_one(&admin)
    .await?;
    assert_eq!(count, 1);
    // Even the disposable admin cannot UPDATE an append-only processed receipt.
    assert!(sqlx::query("UPDATE multica.schedule_admission_inbox SET processed_at=clock_timestamp() WHERE request_id=$1")
        .bind(command.request_id).execute(&admin).await.is_err());
    // Current directory/binding checks share the new admission transaction.
    let missing = command_fixture();
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(missing.tenant_id), &missing.encode_json()?)
        .await?
    else {
        panic!("new missing-directory request")
    };
    assert!(matches!(
        pending.revalidate_directory_and_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized
        ))
    ));
    drop(pending);
    let current = command_fixture();
    seed_directory(&admin, &current, "active").await?;
    let binding_store = PostgresRunInfrastructureBindingStore::new(admin.clone());
    binding_store
        .save_revision(
            current.occurrence.run_as_actor_id,
            None,
            &current.infrastructure_binding,
        )
        .await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(current.tenant_id), &current.encode_json()?)
        .await?
    else {
        panic!("new current request")
    };
    pending.revalidate_directory_and_binding().await?;
    assert!(matches!(
        pending.revalidate_task_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    seed_task_binding(&admin, &current).await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(current.tenant_id), &current.encode_json()?)
        .await?
    else {
        panic!("new task-binding request")
    };
    pending.revalidate_task_binding().await?;
    // The read locks hold even against a disposable admin mutation that bypasses
    // lifecycle triggers, proving neither canonical ACL nor binding can move.
    for (table, predicate, identity) in [
        (
            "public.worktree_canvas_worktree",
            "id",
            current.occurrence.rule_snapshot.target.worktree_id,
        ),
        (
            "permission.project_role_binding",
            "project_id",
            current.occurrence.rule_snapshot.target.project_id,
        ),
        (
            "multica.run_infrastructure_binding_revision",
            "binding_id",
            current.infrastructure_binding.binding_id,
        ),
        (
            "multica.work_item_worktree",
            "worktree_id",
            current.occurrence.rule_snapshot.target.worktree_id,
        ),
        (
            "multica.task_metadata",
            "work_item_id",
            current.occurrence.rule_snapshot.target.work_item_id,
        ),
        (
            "multica.worktree_project_binding",
            "worktree_id",
            current.occurrence.rule_snapshot.target.worktree_id,
        ),
        (
            "multica.engineering_run_worktree_binding",
            "worktree_id",
            current.occurrence.rule_snapshot.target.worktree_id,
        ),
    ] {
        let mut mutate = admin.begin().await?;
        sqlx::raw_sql(
            "SET LOCAL lock_timeout='150ms'; SET LOCAL session_replication_role=replica;",
        )
        .execute(&mut *mutate)
        .await?;
        let error=sqlx::query(&format!("UPDATE {table} SET valid_to=clock_timestamp() WHERE {predicate}=$1 AND valid_to IS NULL"))
            .bind(identity).execute(&mut *mutate).await.expect_err("currentness must remain locked");
        assert_eq!(
            error.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("55P03")
        );
        mutate.rollback().await?;
    }
    drop(pending);
    let mut wrong_task = current.clone();
    wrong_task.request_id = Uuid::new_v4();
    wrong_task.occurrence.rule_snapshot.target.work_item_id = Uuid::new_v4();
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(wrong_task.tenant_id), &wrong_task.encode_json()?)
        .await?
    else {
        panic!("wrong task identity")
    };
    assert!(matches!(
        pending.revalidate_task_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    // Missing associations and archived checkouts must never reuse the earlier pass.
    sqlx::query("UPDATE public.worktree_canvas_worktree SET archived=TRUE WHERE id=$1")
        .bind(current.occurrence.rule_snapshot.target.worktree_id)
        .execute(&admin)
        .await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(current.tenant_id), &current.encode_json()?)
        .await?
    else {
        panic!("new archived request")
    };
    assert!(matches!(
        pending.revalidate_task_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    sqlx::query("UPDATE public.worktree_canvas_worktree SET archived=FALSE WHERE id=$1")
        .bind(current.occurrence.rule_snapshot.target.worktree_id)
        .execute(&admin)
        .await?;
    let mut close = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *close)
        .await?;
    sqlx::query("UPDATE multica.work_item_worktree SET valid_to=clock_timestamp() WHERE tenant_id=$1 AND work_item_id=$2 AND valid_to IS NULL")
        .bind(current.tenant_id).bind(current.occurrence.rule_snapshot.target.work_item_id).execute(&mut *close).await?;
    close.commit().await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(current.tenant_id), &current.encode_json()?)
        .await?
    else {
        panic!("new closed association request")
    };
    assert!(matches!(
        pending.revalidate_task_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    let mut revoke = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *revoke)
        .await?;
    sqlx::query("UPDATE permission.project_role_binding SET valid_to=clock_timestamp() WHERE tenant_id=$1 AND project_id=$2 AND user_id=$3 AND valid_to IS NULL")
        .bind(current.tenant_id).bind(current.occurrence.rule_snapshot.target.project_id).bind(current.occurrence.run_as_actor_id)
        .execute(&mut *revoke).await?;
    revoke.commit().await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(current.tenant_id), &current.encode_json()?)
        .await?
    else {
        panic!("no admission committed")
    };
    assert!(matches!(
        pending.revalidate_directory_and_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized
        ))
    ));
    drop(pending);
    let drift = command_fixture();
    seed_directory(&admin, &drift, "active").await?;
    binding_store
        .save_revision(
            drift.occurrence.run_as_actor_id,
            None,
            &drift.infrastructure_binding,
        )
        .await?;
    let mut successor = drift.infrastructure_binding.clone();
    successor.binding_revision = 2;
    successor.k3s_cluster_generation = 2;
    binding_store
        .save_revision(drift.occurrence.run_as_actor_id, Some(1), &successor)
        .await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(drift.tenant_id), &drift.encode_json()?)
        .await?
    else {
        panic!("new drift request")
    };
    assert!(matches!(
        pending.revalidate_directory_and_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::BindingUnavailable
        ))
    ));
    drop(pending);
    let paused = command_fixture();
    seed_directory(&admin, &paused, "paused").await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(paused.tenant_id), &paused.encode_json()?)
        .await?
    else {
        panic!("new paused request")
    };
    assert!(matches!(
        pending.revalidate_directory_and_binding().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized
        ))
    ));
    drop(pending);
    let mut policies = command_fixture();
    seed_directory(&admin, &policies, "active").await?;
    seed_task_binding(&admin, &policies).await?;
    let (profile_document, project_policy_id, hook_document) =
        seed_policy(&admin, &mut policies).await?;
    binding_store
        .save_revision(
            policies.occurrence.run_as_actor_id,
            None,
            &policies.infrastructure_binding,
        )
        .await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(policies.tenant_id), &policies.encode_json()?)
        .await?
    else {
        panic!("new policy request")
    };
    assert!(matches!(
        pending.revalidate_execution_catalogs().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    seed_catalogs(&admin, &policies, &profile_document).await?;
    // A distinct task/checkout shares this Project/Run but has an independent request.
    let mut parallel = policies.clone();
    parallel.request_id = Uuid::new_v4();
    parallel.occurrence.occurrence_id = Uuid::new_v4();
    parallel.occurrence.rule_snapshot.target.work_item_id = Uuid::new_v4();
    parallel.occurrence.rule_snapshot.target.worktree_id = Uuid::new_v4();
    seed_task_binding(&admin, &parallel).await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(policies.tenant_id), &policies.encode_json()?)
        .await?
    else {
        panic!("new catalog request")
    };
    let snapshot = pending.revalidate_execution_catalogs().await?;
    assert_eq!(snapshot.catalogs().providers().len(), 4);
    assert_eq!(snapshot.catalogs().skills().len(), 1);
    snapshot.validate_lifetime()?;
    let ScheduleAdmissionStart::New(mut second) = store
        .begin(source(parallel.tenant_id), &parallel.encode_json()?)
        .await?
    else {
        panic!("parallel catalog request")
    };
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        second.revalidate_execution_catalogs(),
    )
    .await??;
    drop(second);
    // Normal ACL/capability publications must wait even without a conflicting target row.
    let mut mutation = admin.begin().await?;
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *mutation)
        .await?;
    let error=sqlx::query("INSERT INTO permission.project_role_binding(tenant_id,project_id,user_id,role,granted_by,version) VALUES($1,$2,$3,'developer',$4,1)")
        .bind(policies.tenant_id).bind(policies.occurrence.rule_snapshot.target.project_id).bind(Uuid::new_v4()).bind(policies.occurrence.run_as_actor_id)
        .execute(&mut *mutation).await.expect_err("ACL publication must wait for shared authorizers");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("55P03")
    );
    mutation.rollback().await?;
    let mut mutation = admin.begin().await?;
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *mutation)
        .await?;
    let error=sqlx::query("INSERT INTO multica.agent_execution_provider_catalog(tenant_id,project_id,provider_id,provider_version,implementation_digest,configuration_digest,capability_count,published_by) VALUES($1,$2,'new.provider',1,$3,$4,0,$5)")
        .bind(policies.tenant_id).bind(policies.occurrence.rule_snapshot.target.project_id).bind("a".repeat(64)).bind("b".repeat(64)).bind(policies.occurrence.run_as_actor_id)
        .execute(&mut *mutation).await.expect_err("catalog publication must wait for shared readers");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("55P03")
    );
    mutation.rollback().await?;
    tokio::time::sleep(std::time::Duration::from_millis(5_001)).await;
    assert!(matches!(
        snapshot.validate_lifetime(),
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    // Actual current facts, not sealed Profile declarations, govern later attempts.
    for (sql,restore) in [
      ("UPDATE multica.agent_execution_provider_catalog SET available=FALSE WHERE tenant_id=$1 AND provider_id='agent.local'","UPDATE multica.agent_execution_provider_catalog SET available=TRUE WHERE tenant_id=$1 AND provider_id='agent.local'"),
      ("UPDATE multica.agent_execution_skill_catalog SET content_digest=repeat('f',64) WHERE tenant_id=$1","UPDATE multica.agent_execution_skill_catalog SET content_digest=repeat('a',64) WHERE tenant_id=$1"),
      ("UPDATE multica.agent_execution_grant_set SET expires_at_epoch_ms=1 WHERE tenant_id=$1","UPDATE multica.agent_execution_grant_set SET expires_at_epoch_ms=1900000000000 WHERE tenant_id=$1"),
      ("UPDATE multica.agent_execution_provider_catalog SET capability_count=0 WHERE tenant_id=$1 AND provider_id='agent.local'","UPDATE multica.agent_execution_provider_catalog SET capability_count=1 WHERE tenant_id=$1 AND provider_id='agent.local'")
    ] {
        let mut change=admin.begin().await?;
        sqlx::query("SET LOCAL session_replication_role=replica").execute(&mut *change).await?;
        sqlx::query(sql).bind(policies.tenant_id).execute(&mut *change).await?;
        change.commit().await?;
        let ScheduleAdmissionStart::New(mut attempt)=store.begin(source(policies.tenant_id),&policies.encode_json()?).await? else {panic!("catalog drift request")};
        assert!(matches!(attempt.revalidate_execution_catalogs().await,Err(ScheduleAdmissionStoreError::PrerequisiteRejected(ScheduleAdmissionRejectionCodeV1::TargetUnavailable))));
        drop(attempt);
        let mut change=admin.begin().await?;
        sqlx::query("SET LOCAL session_replication_role=replica").execute(&mut *change).await?;
        sqlx::query(restore).bind(policies.tenant_id).execute(&mut *change).await?;
        change.commit().await?;
    }
    verify_project_capacity(&admin, &store, &policies, &parallel).await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(policies.tenant_id), &policies.encode_json()?)
        .await?
    else {
        panic!("restored policy request")
    };
    let snapshot = pending.revalidate_profile_and_hooks().await?;
    assert_eq!(snapshot.profile().document(), &profile_document);
    assert_eq!(snapshot.hook_policy().digest(), hook_document.digest);
    // The shared Project publication fence protects the absence of an overlay.
    let mut overlay = hook_document.clone();
    overlay.worktree_id = Some(
        policies
            .occurrence
            .rule_snapshot
            .target
            .worktree_id
            .into_bytes(),
    );
    overlay.worktree_version = Some(1);
    overlay.digest = overlay.computed_digest().unwrap();
    let mut publish = admin.begin().await?;
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *publish)
        .await?;
    let error = sqlx::query("INSERT INTO multica.hook_policy_set(tenant_id,project_id,scope_kind,worktree_id,policy_version,schema_version,evaluator_api_version,digest,policy_document,inherited_project_policy_set_id,published_by) VALUES($1,$2,'worktree',$3,1,1,2,$4,$5,$6,$7)")
        .bind(policies.tenant_id).bind(policies.occurrence.rule_snapshot.target.project_id)
        .bind(policies.occurrence.rule_snapshot.target.worktree_id).bind(overlay.digest.as_slice())
        .bind(serde_json::to_value(&overlay)?).bind(project_policy_id).bind(policies.occurrence.run_as_actor_id)
        .execute(&mut *publish).await.expect_err("overlay publication must wait for current admission");
    assert_eq!(
        error.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("55P03")
    );
    publish.rollback().await?;
    drop(pending);
    // Publication after the reader ends must invalidate a baseline-pinned request.
    let overlay_id = Uuid::new_v4();
    sqlx::query("INSERT INTO multica.hook_policy_set(policy_set_id,tenant_id,project_id,scope_kind,worktree_id,policy_version,schema_version,evaluator_api_version,digest,policy_document,inherited_project_policy_set_id,published_by) VALUES($1,$2,$3,'worktree',$4,1,1,2,$5,$6,$7,$8)")
        .bind(overlay_id).bind(policies.tenant_id).bind(policies.occurrence.rule_snapshot.target.project_id)
        .bind(policies.occurrence.rule_snapshot.target.worktree_id).bind(overlay.digest.as_slice())
        .bind(serde_json::to_value(&overlay)?).bind(project_policy_id).bind(policies.occurrence.run_as_actor_id)
        .execute(&admin).await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(policies.tenant_id), &policies.encode_json()?)
        .await?
    else {
        panic!("new overlay drift")
    };
    assert!(matches!(
        pending.revalidate_profile_and_hooks().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    let mut revised_draft = profile_document.profile.clone();
    revised_draft.hook_set.hook_set_id = overlay_id;
    revised_draft.hook_set.effective_digest =
        overlay.digest.iter().map(|b| format!("{b:02x}")).collect();
    let revised_profile = revised_draft.seal().unwrap();
    let mut revise = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *revise)
        .await?;
    sqlx::query("UPDATE multica.agent_execution_profile SET profile_version=2,content_digest=$1,profile_document=$2 WHERE tenant_id=$3 AND profile_id=$4")
        .bind(&revised_profile.document().content_digest).bind(serde_json::to_value(revised_profile.document())?)
        .bind(policies.tenant_id).bind(policies.occurrence.rule_snapshot.target.execution_profile_id).execute(&mut *revise).await?;
    revise.commit().await?;
    policies.request_id = Uuid::new_v4();
    policies
        .occurrence
        .rule_snapshot
        .target
        .execution_profile_version = 2;
    policies
        .occurrence
        .rule_snapshot
        .target
        .execution_profile_digest = revised_profile.document().content_digest.clone();
    policies.occurrence.rule_snapshot.target.hook_set_digest = revised_profile
        .document()
        .profile
        .hook_set
        .effective_digest
        .clone();
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(policies.tenant_id), &policies.encode_json()?)
        .await?
    else {
        panic!("new pinned overlay")
    };
    let snapshot = pending.revalidate_profile_and_hooks().await?;
    assert_eq!(snapshot.hook_policy().digest(), overlay.digest);
    drop(pending);
    // A changed pinned Profile version is a new request and cannot use the earlier snapshot.
    let mut wrong_version = policies.clone();
    wrong_version.request_id = Uuid::new_v4();
    wrong_version
        .occurrence
        .rule_snapshot
        .target
        .execution_profile_version = 3;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(
            source(wrong_version.tenant_id),
            &wrong_version.encode_json()?,
        )
        .await?
    else {
        panic!("new wrong version")
    };
    assert!(matches!(
        pending.revalidate_profile_and_hooks().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    // Exact infrastructure binding is valid, but the verified Profile asks for a different budget.
    let mut mismatched = policies.clone();
    mismatched.request_id = Uuid::new_v4();
    mismatched.infrastructure_binding.binding_revision = 2;
    mismatched
        .infrastructure_binding
        .resource_budget
        .max_rss_bytes += 1;
    binding_store
        .save_revision(
            mismatched.occurrence.run_as_actor_id,
            Some(1),
            &mismatched.infrastructure_binding,
        )
        .await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(mismatched.tenant_id), &mismatched.encode_json()?)
        .await?
    else {
        panic!("new budget mismatch")
    };
    assert!(matches!(
        pending.revalidate_profile_and_hooks().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::BindingUnavailable
        ))
    ));
    drop(pending);
    // Digest tampering remains rejected even when columns still match the pinned identities.
    let mut corrupt = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *corrupt)
        .await?;
    sqlx::query("UPDATE multica.agent_execution_profile SET profile_document=jsonb_set(profile_document,'{profile,resource_budget,max_cpu_ms}','1001') WHERE tenant_id=$1 AND profile_id=$2")
        .bind(policies.tenant_id).bind(policies.occurrence.rule_snapshot.target.execution_profile_id).execute(&mut *corrupt).await?;
    corrupt.commit().await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(mismatched.tenant_id), &mismatched.encode_json()?)
        .await?
    else {
        panic!("new corrupt profile")
    };
    assert!(matches!(
        pending.revalidate_profile_and_hooks().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    drop(pending);
    let mut orphan = admin.begin().await?;
    sqlx::query("INSERT INTO multica.schedule_admission_inbox
       (tenant_id,request_id,result_id,project_id,branch_id,engineering_run_id,work_item_id,worktree_id,
        occurrence_id,admission_attempt,correlation_id,run_as_actor_id,rule_id,rule_version,command_wire,command_sha256,source_schedule_service_id)
       SELECT tenant_id,$1,$2,project_id,branch_id,engineering_run_id,work_item_id,worktree_id,
        occurrence_id,admission_attempt+1,correlation_id,run_as_actor_id,rule_id,rule_version,
        convert_to(jsonb_set(jsonb_set(convert_from(command_wire,'UTF8')::jsonb,
         '{request_id}',to_jsonb($1::uuid)), '{admission_attempt}',to_jsonb(admission_attempt+1))::text,'UTF8'),
        command_sha256,source_schedule_service_id FROM multica.schedule_admission_inbox WHERE tenant_id=$3 AND request_id=$4")
        .bind(Uuid::new_v4()).bind(Uuid::new_v4()).bind(command.tenant_id).bind(command.request_id)
        .execute(&mut *orphan).await?;
    assert!(
        orphan.commit().await.is_err(),
        "a processed receipt cannot commit without its result"
    );
    let final_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM multica.schedule_admission_inbox WHERE tenant_id=$1",
    )
    .bind(command.tenant_id)
    .fetch_one(&admin)
    .await?;
    assert_eq!(final_count, 1);
    pool.close().await;
    admin.close().await;
    Ok(())
}

async fn verify_admission_quarantine(
    admin: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    use platform_messaging_runtime::admission_receive::{
        AdmissionReceiveError, AdmissionReceiveKind, AdmissionReceiveOrigin,
        AdmissionReceivePoisonFault, AdmissionReceiveQuarantine, AdmissionReceiveScope,
    };
    use platform_messaging_runtime::admission_quarantine_pg::PgAdmissionReceiveQuarantine;

    sqlx::raw_sql(
        "CREATE ROLE star_platform_ingress_test_login LOGIN NOINHERIT NOSUPERUSER NOBYPASSRLS;
        GRANT star_platform_ingress_owner TO star_platform_ingress_test_login;
        ALTER ROLE star_platform_ingress_test_login SET statement_timeout = '10s';",
    )
    .execute(admin)
    .await?;
    let options: PgConnectOptions = std::env::var("STAR_RUN_OWNER_TEST_DATABASE_URL")?.parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options.username("star_platform_ingress_test_login"))
        .await?;
    let deployment_id = Uuid::new_v4();
    assert!(PgAdmissionReceiveQuarantine::new(pool.clone(), Uuid::nil()).is_err());
    let quarantine = PgAdmissionReceiveQuarantine::new(pool.clone(), deployment_id)?;
    let origin = AdmissionReceiveOrigin {
        scope: AdmissionReceiveScope {
            tenant_id: Uuid::new_v4(),
            sender_service_id: Uuid::new_v4(),
            recipient_service_id: Uuid::new_v4(),
            kind: AdmissionReceiveKind::Request,
        },
        message_id: Uuid::nil(),
        provider_identity: "nats-jetstream/v1".to_owned(),
        log_id: "SCHEDULE_REQUESTS".to_owned(),
        log_generation: Uuid::new_v4(),
        sequence: 7,
    };
    let wire = b"{malformed";
    quarantine
        .record(&origin, wire, AdmissionReceivePoisonFault::MalformedWire)
        .await?;
    quarantine
        .record(&origin, wire, AdmissionReceivePoisonFault::MalformedWire)
        .await?;
    assert!(matches!(
        quarantine
            .record(&origin, b"different exact wire", AdmissionReceivePoisonFault::MalformedWire)
            .await,
        Err(AdmissionReceiveError::Quarantine)
    ));
    let mut changed_fault = origin.clone();
    changed_fault.message_id = Uuid::new_v4();
    assert!(matches!(
        quarantine
            .record(
                &changed_fault,
                wire,
                AdmissionReceivePoisonFault::IdentityMismatch,
            )
            .await,
        Err(AdmissionReceiveError::Quarantine)
    ));
    let mut other_provider = origin.clone();
    other_provider.provider_identity = "star-rust-log/v1".to_owned();
    other_provider.message_id = Uuid::new_v4();
    quarantine
        .record(
            &other_provider,
            b"other provider source",
            AdmissionReceivePoisonFault::MalformedWire,
        )
        .await?;
    let mut cross_tenant = origin.clone();
    cross_tenant.scope.tenant_id = Uuid::new_v4();
    assert!(matches!(
        quarantine
            .record(&cross_tenant, wire, AdmissionReceivePoisonFault::MalformedWire)
            .await,
        Err(AdmissionReceiveError::Quarantine)
    ));
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM platform_messaging.admission_receive_quarantine_receipt WHERE deployment_id=$1",
    )
    .bind(deployment_id)
    .fetch_one(admin)
    .await?;
    assert_eq!(receipts, 2, "exact replay adds no receipt; another provider has its own key");
    let mut receiver_origin = origin.clone();
    receiver_origin.sequence = 8;
    verify_actual_quarantine_receiver(
        admin.clone(),
        quarantine,
        deployment_id,
        receiver_origin,
        wire.to_vec(),
    )
    .await?;
    pool.close().await;
    Ok(())
}

#[derive(Default)]
struct AdmissionQuarantineReceiveTestState {
    source_verifications: usize,
    owner_handler_calls: usize,
    terminal_dispositions: usize,
    terminal_saw_committed_receipt: Vec<bool>,
    acknowledgements: usize,
}

#[derive(Clone)]
struct AdmissionQuarantineReceiveTestDelivery {
    admin: sqlx::PgPool,
    deployment_id: Uuid,
    origin: platform_messaging_runtime::admission_receive::AdmissionReceiveOrigin,
    wire: Vec<u8>,
    state: std::sync::Arc<std::sync::Mutex<AdmissionQuarantineReceiveTestState>>,
}

impl platform_messaging_runtime::admission_receive::AdmissionReceivedDelivery
    for AdmissionQuarantineReceiveTestDelivery
{
    fn wire(&self) -> &[u8] {
        &self.wire
    }

    fn origin(&self) -> &platform_messaging_runtime::admission_receive::AdmissionReceiveOrigin {
        &self.origin
    }

    async fn acknowledge(
        &self,
    ) -> Result<(), platform_messaging_runtime::admission_receive::AdmissionReceiveError> {
        self.state.lock().unwrap().acknowledgements += 1;
        Ok(())
    }

    async fn terminate_permanently(
        &self,
    ) -> Result<(), platform_messaging_runtime::admission_receive::AdmissionReceiveError> {
        use platform_messaging_runtime::admission_receive::AdmissionReceiveError;

        let committed: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM platform_messaging.admission_receive_quarantine_receipt
             WHERE deployment_id=$1 AND provider_identity=$2 AND log_generation=$3
               AND log_id=$4 AND sequence=$5",
        )
        .bind(self.deployment_id)
        .bind(&self.origin.provider_identity)
        .bind(self.origin.log_generation)
        .bind(&self.origin.log_id)
        .bind(self.origin.sequence)
        .fetch_one(&self.admin)
        .await
        .map_err(|_| AdmissionReceiveError::Transport)?;
        let mut state = self.state.lock().unwrap();
        state.terminal_dispositions += 1;
        state.terminal_saw_committed_receipt.push(committed == 1);
        if committed != 1 || state.terminal_dispositions == 1 {
            return Err(AdmissionReceiveError::Transport);
        }
        Ok(())
    }
}

#[derive(Clone)]
struct AdmissionQuarantineReceiveTestTransport {
    delivery: AdmissionQuarantineReceiveTestDelivery,
}

impl platform_messaging_runtime::admission_receive::AdmissionReceiveTransport
    for AdmissionQuarantineReceiveTestTransport
{
    type Delivery = AdmissionQuarantineReceiveTestDelivery;

    async fn pull_one(
        &self,
        _scope: platform_messaging_runtime::admission_receive::AdmissionReceiveScope,
    ) -> Result<
        Option<Self::Delivery>,
        platform_messaging_runtime::admission_receive::AdmissionReceiveError,
    > {
        Ok(Some(self.delivery.clone()))
    }
}

#[derive(Clone)]
struct AdmissionQuarantineReceiveTestAuthorization {
    state: std::sync::Arc<std::sync::Mutex<AdmissionQuarantineReceiveTestState>>,
}

impl platform_messaging_runtime::admission_receive::AdmissionReceiveAuthorization
    for AdmissionQuarantineReceiveTestAuthorization
{
    async fn verify(
        &self,
        _origin: &platform_messaging_runtime::admission_receive::AdmissionReceiveOrigin,
        _wire: &[u8],
    ) -> Result<(), platform_messaging_runtime::admission_receive::AdmissionReceiveError> {
        Ok(())
    }

    async fn verify_source(
        &self,
        _origin: &platform_messaging_runtime::admission_receive::AdmissionReceiveOrigin,
    ) -> Result<(), platform_messaging_runtime::admission_receive::AdmissionReceiveError> {
        self.state.lock().unwrap().source_verifications += 1;
        Ok(())
    }
}

struct AdmissionQuarantineReceiveTestHandler {
    state: std::sync::Arc<std::sync::Mutex<AdmissionQuarantineReceiveTestState>>,
}

impl platform_messaging_runtime::admission_receive::AdmissionReceiveHandler
    for AdmissionQuarantineReceiveTestHandler
{
    async fn commit(
        &self,
        _scope: platform_messaging_runtime::admission_receive::AdmissionReceiveScope,
        _wire: &[u8],
    ) -> Result<Uuid, platform_messaging_runtime::admission_receive::AdmissionReceiveError> {
        self.state.lock().unwrap().owner_handler_calls += 1;
        Err(platform_messaging_runtime::admission_receive::AdmissionReceiveError::OwnerCommit)
    }
}

async fn verify_actual_quarantine_receiver(
    admin: sqlx::PgPool,
    quarantine: platform_messaging_runtime::admission_quarantine_pg::PgAdmissionReceiveQuarantine,
    deployment_id: Uuid,
    origin: platform_messaging_runtime::admission_receive::AdmissionReceiveOrigin,
    wire: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    use platform_messaging_runtime::admission_receive::AdmissionReceiver;

    let state = std::sync::Arc::new(std::sync::Mutex::new(
        AdmissionQuarantineReceiveTestState::default(),
    ));
    let receipt_reader = admin.clone();
    let delivery = AdmissionQuarantineReceiveTestDelivery {
        admin,
        deployment_id,
        origin: origin.clone(),
        wire,
        state: state.clone(),
    };
    let transport = AdmissionQuarantineReceiveTestTransport { delivery };
    let authorization = AdmissionQuarantineReceiveTestAuthorization {
        state: state.clone(),
    };
    let handler = AdmissionQuarantineReceiveTestHandler {
        state: state.clone(),
    };
    let mut receiver = AdmissionReceiver::new(transport, handler, authorization, origin.scope)?
        .with_quarantine(std::sync::Arc::new(quarantine));
    let (_cancel, mut cancellation) = tokio::sync::watch::channel(false);

    assert!(matches!(
        receiver.consume_one(&mut cancellation).await,
        Err(platform_messaging_runtime::admission_receive::AdmissionReceiveError::Transport)
    ));
    {
        let state = state.lock().unwrap();
        assert_eq!(
            state.source_verifications, 2,
            "source is checked before and after receipt"
        );
        assert_eq!(
            state.owner_handler_calls, 0,
            "malformed poison never reaches owner handler"
        );
        assert_eq!(state.terminal_dispositions, 1);
        assert_eq!(state.terminal_saw_committed_receipt, [true]);
        assert_eq!(
            state.acknowledgements, 0,
            "poison uses terminal disposition, never ACK"
        );
    }

    assert_eq!(receiver.consume_one(&mut cancellation).await?, None);
    {
        let state = state.lock().unwrap();
        assert_eq!(
            state.source_verifications, 4,
            "replay rechecks source around the receipt"
        );
        assert_eq!(state.owner_handler_calls, 0);
        assert_eq!(state.terminal_dispositions, 2);
        assert_eq!(state.terminal_saw_committed_receipt, [true, true]);
        assert_eq!(state.acknowledgements, 0);
    }
    let replayed_receipt: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM platform_messaging.admission_receive_quarantine_receipt
         WHERE deployment_id=$1 AND provider_identity=$2 AND log_generation=$3
           AND log_id=$4 AND sequence=$5",
    )
    .bind(deployment_id)
    .bind(&origin.provider_identity)
    .bind(origin.log_generation)
    .bind(&origin.log_id)
    .bind(origin.sequence)
    .fetch_one(&receipt_reader)
    .await?;
    assert_eq!(
        replayed_receipt, 1,
        "same source delivery replays one durable receipt"
    );
    Ok(())
}

fn rejected_result(
    command: &ScheduleRunAdmissionCommandV1,
) -> Result<ScheduleRunAdmissionResultV1, ScheduleAdmissionContractError> {
    Ok(ScheduleRunAdmissionResultV1 {
        schema_version: 1,
        tenant_id: command.tenant_id,
        request_id: command.request_id,
        result_id: Uuid::new_v4(),
        admission_attempt: command.admission_attempt,
        correlation_id: command.correlation_id,
        occurrence_id: command.occurrence.occurrence_id,
        outcome: ScheduleRunAdmissionOutcomeV1::Rejected,
        task_execution_run_id: None,
        rejection_code: Some(ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized),
        binding_digest: command.binding_digest()?,
    })
}
fn command_fixture() -> ScheduleRunAdmissionCommandV1 {
    let binding = binding_fixture();
    let rule = AutomationScheduleRuleRevisionV1 {
        tenant_id: binding.tenant_id,
        project_id: binding.project_id,
        rule_id: RuleId::new(),
        rule_version: 1,
        run_as_actor_id: Uuid::new_v4(),
        enabled: true,
        cron_expression: "0 9 * * 1-5".to_owned(),
        time_zone: "Asia/Tokyo".to_owned(),
        parser_version: SCHEDULE_PARSER_VERSION.to_owned(),
        tzdb_version: SCHEDULE_TZDB_VERSION.to_owned(),
        dst_gap_policy: ScheduleDstGapPolicyV1::Skip,
        dst_fold_policy: ScheduleDstFoldPolicyV1::EarlierInstant,
        overlap_policy: ScheduleOverlapPolicyV1::QueueOne,
        misfire_policy: ScheduleMisfirePolicyV1::CatchUp { max_occurrences: 3 },
        pause_policy: SchedulePausePolicyV1::CoalesceLatest,
        retry_policy: ScheduleRetryPolicyV1 {
            max_attempts: 3,
            initial_backoff_seconds: 5,
            max_backoff_seconds: 300,
        },
        deadline_seconds: 3600,
        target: ScheduleTargetV1 {
            project_id: binding.project_id,
            branch_id: binding.branch_id,
            engineering_run_id: binding.engineering_run_id,
            repository_id: Uuid::new_v4(),
            worktree_id: Uuid::new_v4(),
            work_item_id: Uuid::new_v4(),
            execution_profile_id: Uuid::new_v4(),
            execution_profile_version: 1,
            execution_profile_digest: "a".repeat(64),
            hook_set_version: 1,
            hook_set_digest: "b".repeat(64),
        },
    };
    ScheduleRunAdmissionCommandV1 {
        schema_version: 1,
        tenant_id: rule.tenant_id,
        request_id: Uuid::new_v4(),
        correlation_id: Uuid::new_v4(),
        admission_attempt: 1,
        infrastructure_binding: binding,
        occurrence: AutomationOccurrenceSnapshotV1 {
            schema_version: 1,
            occurrence_id: Uuid::new_v4(),
            key: AutomationOccurrenceKey {
                tenant_id: rule.tenant_id,
                rule_id: rule.rule_id,
                rule_version: 1,
                scheduled_for_utc: Utc::now(),
            },
            run_as_actor_id: rule.run_as_actor_id,
            rule_snapshot: rule,
            scheduled_local_label: "2026-10-10T09:00:00".to_owned(),
            utc_offset_seconds: 32400,
            materialized_at: Utc::now(),
        },
    }
}

fn binding_fixture() -> RunInfrastructureBindingSnapshotV2 {
    RunInfrastructureBindingSnapshotV2 {
        schema_version: 2,
        tenant_id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        branch_id: Uuid::new_v4(),
        engineering_run_id: Uuid::new_v4(),
        binding_id: Uuid::new_v4(),
        binding_revision: 1,
        host_environment_id: Uuid::new_v4(),
        k3s_cluster_id: Uuid::new_v4(),
        k3s_cluster_generation: 1,
        cluster_placement_policy: ClusterPlacementPolicyV1::PinnedCluster,
        provider: ProviderKind::MultipassK3sCluster,
        provider_profile_id: Uuid::new_v4(),
        provider_profile_revision: 1,
        provider_profile_digest: [9; 32],
        namespace: KubernetesDnsLabel::new("run-sqlx").unwrap(),
        service_account: KubernetesDnsLabel::new("agent-sqlx").unwrap(),
        runtime_profile_id: Uuid::new_v4(),
        runtime_profile_revision: 1,
        runtime_profile_digest: [11; 32],
        resource_budget: ExecutionResourceBudgetV1 {
            max_rss_bytes: 64 * 1024 * 1024,
            max_cpu_ms: 1000,
            max_runtime_ms: 30000,
            max_child_processes: 1,
            max_parallel_tools: 1,
            max_provider_calls: 1,
            max_output_bytes: 4096,
            max_event_buffer_bytes: 4096,
        },
    }
}

fn source(tenant_id: Uuid) -> ScheduleAdmissionSource {
    ScheduleAdmissionSource {
        tenant_id,
        schedule_service_id: Uuid::from_u128(9001),
    }
}

async fn seed_provisional_run(
    admin: &sqlx::PgPool,
    command: &ScheduleRunAdmissionCommandV1,
    run_id: Uuid,
) -> Result<(), sqlx::Error> {
    let target = &command.occurrence.rule_snapshot.target;
    sqlx::query(
        "INSERT INTO multica.task_execution_run
        (run_id,tenant_id,project_id,work_item_id,initiated_by,execution_channel,worktree_id,
         task_snapshot,run_origin,automation_rule_id,automation_rule_version,
         automation_occurrence_id,automation_admission_attempt,correlation_id,started_at)
        VALUES($1,$2,$3,$4,$5,'agent',$6,'{}','schedule',$7,1,$8,1,$9,NULL)",
    )
    .bind(Some(run_id))
    .bind(command.tenant_id)
    .bind(target.project_id)
    .bind(target.work_item_id)
    .bind(command.occurrence.run_as_actor_id)
    .bind(target.worktree_id)
    .bind(command.occurrence.key.rule_id.as_uuid())
    .bind(command.occurrence.occurrence_id)
    .bind(command.correlation_id)
    .execute(admin)
    .await?;

    Ok(())
}

async fn seed_directory(
    admin: &sqlx::PgPool,
    command: &ScheduleRunAdmissionCommandV1,
    state: &str,
) -> Result<(), sqlx::Error> {
    assert!(matches!(state, "active" | "paused"));
    let t = &command.occurrence.rule_snapshot.target;
    let tenant = command.tenant_id;
    let actor = command.occurrence.run_as_actor_id;
    let project = t.project_id;
    let branch = t.branch_id;
    let run = t.engineering_run_id;
    let repo = t.repository_id;
    // Generated UUIDs and allowlisted state only; no external/user strings enter SQL.
    sqlx::raw_sql(&format!("BEGIN; SET LOCAL session_replication_role=replica;
        INSERT INTO scm.cloud_branch(branch_id,tenant_id,project_id,repository_id,remote_identity)
        VALUES('{branch}','{tenant}','{project}','{repo}','test://admission-currentness');
        INSERT INTO scm.cloud_branch_revision(tenant_id,branch_id,name,full_ref,state,changed_by,version)
        VALUES('{tenant}','{branch}','dev','refs/heads/dev','active','{actor}',1);
        INSERT INTO multica.engineering_run(engineering_run_id,tenant_id,project_id,repository_id,branch_id)
        VALUES('{run}','{tenant}','{project}','{repo}','{branch}');
        INSERT INTO multica.engineering_run_revision(tenant_id,engineering_run_id,title,state,changed_by,version)
        VALUES('{tenant}','{run}','Admission fixture','{state}','{actor}',1);
        INSERT INTO permission.project_role_binding(tenant_id,project_id,user_id,role,granted_by,version)
        VALUES('{tenant}','{project}','{actor}','developer','{actor}',1);
        INSERT INTO permission.cloud_branch_role_binding(tenant_id,branch_id,user_id,role,granted_by,version)
        VALUES('{tenant}','{branch}','{actor}','developer','{actor}',1);
        INSERT INTO permission.engineering_run_role_binding(tenant_id,engineering_run_id,user_id,role,granted_by,version)
        VALUES('{tenant}','{run}','{actor}','developer','{actor}',1); COMMIT;"))
        .execute(admin).await?;
    Ok(())
}

async fn seed_task_binding(
    admin: &sqlx::PgPool,
    command: &ScheduleRunAdmissionCommandV1,
) -> Result<(), sqlx::Error> {
    let t = &command.occurrence.rule_snapshot.target;
    let tenant = command.tenant_id;
    let actor = command.occurrence.run_as_actor_id;
    let project = t.project_id;
    let branch = t.branch_id;
    let run = t.engineering_run_id;
    let repo = t.repository_id;
    let worktree = t.worktree_id;
    let task = t.work_item_id;
    let project_binding = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    // Disposable fixture only; generated UUIDs are the only interpolated values.
    sqlx::raw_sql(&format!("BEGIN; SET LOCAL session_replication_role=replica;
      INSERT INTO public.worktree_canvas_worktree(id,tenant_id,workspace_id,repo_id,project_id,name,branch,path)
      VALUES('{worktree}','{tenant}','{workspace}','{repo}','{project}','fixture-{worktree}','dev','fixture://{worktree}');
      INSERT INTO multica.worktree_project_binding(binding_id,tenant_id,project_id,worktree_id,assigned_by)
      VALUES('{project_binding}','{tenant}','{project}','{worktree}','{actor}');
      INSERT INTO multica.engineering_run_worktree_binding(tenant_id,project_id,repository_id,branch_id,engineering_run_id,worktree_id,project_binding_id,assigned_by,version)
      VALUES('{tenant}','{project}','{repo}','{branch}','{run}','{worktree}','{project_binding}','{actor}',1);
      INSERT INTO multica.task_metadata(tenant_id,workspace_id,project_id,repository_id,branch_id,engineering_run_id,work_item_id,item_type,title,reporter_user_id)
      VALUES('{tenant}','{workspace}','{project}','{repo}','{branch}','{run}','{task}','task','Task fixture','{actor}');
      INSERT INTO multica.work_item_worktree(tenant_id,project_id,work_item_id,worktree_id)
      VALUES('{tenant}','{project}','{task}','{worktree}'); COMMIT;"))
        .execute(admin).await?;
    Ok(())
}

async fn seed_policy(
    admin: &sqlx::PgPool,
    command: &mut ScheduleRunAdmissionCommandV1,
) -> Result<
    (
        domain_agent::execution_profile::AgentExecutionProfileDocument,
        Uuid,
        domain_hook::HookPolicyDocument,
    ),
    Box<dyn std::error::Error>,
> {
    use domain_agent::execution_profile::AgentExecutionProfileDraft;
    let tenant = command.tenant_id;
    let actor = command.occurrence.run_as_actor_id;
    let target = &command.occurrence.rule_snapshot.target;
    let project = target.project_id;
    let policy_id = Uuid::new_v4();
    let mut hook = domain_hook::HookPolicyDocument {
        schema_version: domain_hook::POLICY_SCHEMA_VERSION,
        evaluator_api_version: domain_hook::EVALUATOR_API_VERSION,
        tenant_id: tenant.into_bytes(),
        project_id: project.into_bytes(),
        worktree_id: None,
        project_version: 1,
        worktree_version: None,
        digest: [0; 32],
        project_rules: vec![],
        worktree_rules: vec![],
    };
    hook.digest = hook.computed_digest().unwrap();
    hook.clone().verify().unwrap();
    let hook_digest: String = hook.digest.iter().map(|b| format!("{b:02x}")).collect();
    let provider = |name: &str| serde_json::json!({"provider_id":name,"version":1,"implementation_digest":"a".repeat(64),"configuration_digest":"b".repeat(64),"capabilities":if name=="agent.local" {vec!["tool.read"]} else {vec![]}});
    let budget = &command.infrastructure_binding.resource_budget;
    let draft: AgentExecutionProfileDraft = serde_json::from_value(serde_json::json!({
      "schema_version":1,"scope":{"tenant_id":tenant,"project_id":project,"worktree_id":null},
      "agent_provider":provider("agent.local"),"memory":{"state":"disabled"},"skills":[{"skill_id":"review.rust","version":1,"content_digest":"a".repeat(64),"capabilities":["tool.read"]}],
      "context":{"assembler":provider("context.local"),"max_input_bytes":4096,"max_input_tokens":1024,"max_sources":8,"compaction_policy_digest":"c".repeat(64),"preserve_task_contract":true,"preserve_acceptance_criteria":true,"preserve_authorization_scope":true},
      "validation":{"provider":provider("validator.local"),"suite_digest":"d".repeat(64),"toolchain_digest":"e".repeat(64),"acceptance_criteria":["ac-test"]},
      "loop_budget":{"policy":provider("loop.local"),"max_iterations":1,"max_no_progress_iterations":1,"max_wall_clock_ms":30000,"max_provider_calls":1},
      "resource_budget":{"max_rss_bytes":budget.max_rss_bytes,"max_cpu_ms":budget.max_cpu_ms,"max_runtime_ms":budget.max_runtime_ms,"max_child_processes":budget.max_child_processes,"max_parallel_tools":budget.max_parallel_tools,"max_provider_calls":budget.max_provider_calls,"max_output_bytes":budget.max_output_bytes,"max_event_buffer_bytes":budget.max_event_buffer_bytes},
      "hook_set":{"hook_set_id":policy_id,"version":1,"effective_digest":hook_digest},
      "grants":{"grant_set_id":Uuid::new_v4(),"version":1,"capabilities":["tool.read"],"expires_at_epoch_ms":1900000000000u64},"engineering_manifest":null
    }))?;
    let verified = draft.seal().unwrap();
    let document = verified.document().clone();
    let mut tx = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO multica.hook_policy_set(policy_set_id,tenant_id,project_id,scope_kind,policy_version,schema_version,evaluator_api_version,digest,policy_document,published_by) VALUES($1,$2,$3,'project',1,1,2,$4,$5,$6)")
        .bind(policy_id).bind(tenant).bind(project).bind(hook.digest.as_slice()).bind(serde_json::to_value(&hook)?).bind(actor).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO multica.agent_execution_profile(tenant_id,project_id,profile_id,scope_kind,profile_version,schema_version,content_digest,profile_document,changed_by) VALUES($1,$2,$3,'project',1,1,$4,$5,$6)")
        .bind(tenant).bind(project).bind(target.execution_profile_id).bind(&document.content_digest).bind(serde_json::to_value(&document)?).bind(actor).execute(&mut *tx).await?;
    tx.commit().await?;
    let target = &mut command.occurrence.rule_snapshot.target;
    target.execution_profile_digest = document.content_digest.clone();
    target.hook_set_version = 1;
    target.hook_set_digest = hook_digest;
    Ok((document, policy_id, hook))
}

async fn seed_catalogs(
    admin: &sqlx::PgPool,
    command: &ScheduleRunAdmissionCommandV1,
    document: &domain_agent::execution_profile::AgentExecutionProfileDocument,
) -> Result<(), sqlx::Error> {
    let tenant = command.tenant_id;
    let project = command.occurrence.rule_snapshot.target.project_id;
    let actor = command.occurrence.run_as_actor_id;
    let draft = &document.profile;
    let mut tx = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO multica.agent_execution_catalog_revision(tenant_id,project_id,provider_catalog_revision,skill_catalog_revision) VALUES($1,$2,1,1)")
  .bind(tenant).bind(project).execute(&mut *tx).await?;
    for provider in [
        &draft.agent_provider,
        &draft.context.assembler,
        &draft.validation.provider,
        &draft.loop_budget.policy,
    ] {
        let entry = Uuid::new_v4();
        sqlx::query("INSERT INTO multica.agent_execution_provider_catalog(entry_id,tenant_id,project_id,provider_id,provider_version,implementation_digest,configuration_digest,capability_count,published_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
   .bind(entry).bind(tenant).bind(project).bind(&provider.provider_id).bind(i64::from(provider.version)).bind(&provider.implementation_digest).bind(&provider.configuration_digest).bind(i16::try_from(provider.capabilities.len()).unwrap()).bind(actor).execute(&mut *tx).await?;
        for label in &provider.capabilities {
            sqlx::query("INSERT INTO multica.agent_execution_provider_capability(tenant_id,entry_id,capability_code) VALUES($1,$2,$3)").bind(tenant).bind(entry).bind(label).execute(&mut *tx).await?;
        }
    }
    for skill in &draft.skills {
        let entry = Uuid::new_v4();
        sqlx::query("INSERT INTO multica.agent_execution_skill_catalog(entry_id,tenant_id,project_id,skill_id,skill_version,content_digest,capability_count,published_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
   .bind(entry).bind(tenant).bind(project).bind(&skill.skill_id).bind(i64::from(skill.version)).bind(&skill.content_digest).bind(i16::try_from(skill.capabilities.len()).unwrap()).bind(actor).execute(&mut *tx).await?;
        for label in &skill.capabilities {
            sqlx::query("INSERT INTO multica.agent_execution_skill_capability(tenant_id,entry_id,capability_code) VALUES($1,$2,$3)").bind(tenant).bind(entry).bind(label).execute(&mut *tx).await?;
        }
    }
    let grant = &draft.grants;
    let entry = Uuid::new_v4();
    sqlx::query("INSERT INTO multica.agent_execution_grant_set(entry_id,tenant_id,project_id,grant_set_id,grant_set_version,capability_count,expires_at_epoch_ms,published_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
  .bind(entry).bind(tenant).bind(project).bind(grant.grant_set_id).bind(i64::try_from(grant.version).unwrap()).bind(i16::try_from(grant.capabilities.len()).unwrap()).bind(i64::try_from(grant.expires_at_epoch_ms).unwrap()).bind(actor).execute(&mut *tx).await?;
    for label in &grant.capabilities {
        sqlx::query("INSERT INTO multica.agent_execution_grant_capability(tenant_id,entry_id,capability_code) VALUES($1,$2,$3)").bind(tenant).bind(entry).bind(label).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

// Current capacity fixtures do not prove the production quota publisher or Run writer.
async fn verify_project_capacity(
    admin: &sqlx::PgPool,
    store: &PostgresScheduleAdmissionStore,
    command: &ScheduleRunAdmissionCommandV1,
    parallel: &ScheduleRunAdmissionCommandV1,
) -> Result<(), Box<dyn std::error::Error>> {
    let t = &command.occurrence.rule_snapshot.target;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(command.tenant_id), &command.encode_json()?)
        .await?
    else {
        panic!("missing quota request")
    };
    assert!(matches!(
        pending.revalidate_project_capacity().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::QuotaExceeded
        ))
    ));
    drop(pending);
    let quota_id = Uuid::new_v4();
    let b = &command.infrastructure_binding.resource_budget;
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("INSERT INTO multica.project_execution_resource_quota(quota_revision_id,tenant_id,project_id,quota_version,max_active_runs,max_rss_bytes,max_cpu_ms_per_run,max_runtime_ms_per_run,max_child_processes,max_parallel_tools,max_provider_calls,max_output_bytes,max_event_buffer_bytes,changed_by) VALUES($1,$2,$3,1,1,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
 .bind(quota_id).bind(command.tenant_id).bind(t.project_id).bind(b.max_rss_bytes).bind(b.max_cpu_ms).bind(b.max_runtime_ms)
 .bind(b.max_child_processes).bind(b.max_parallel_tools).bind(i32::try_from(b.max_provider_calls)?)
 .bind(b.max_output_bytes).bind(b.max_event_buffer_bytes).bind(command.occurrence.run_as_actor_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(command.tenant_id), &command.encode_json()?)
        .await?
    else {
        panic!("exact capacity request")
    };
    let snapshot = pending.revalidate_project_capacity().await?;
    assert_eq!(snapshot.quota_revision_id(), quota_id);
    assert_eq!(snapshot.quota_version(), 1);
    snapshot.validate_lifetime()?;
    // A second scoped allocation must serialize, although read-only D7 preflights coexist.
    let ScheduleAdmissionStart::New(mut second) = store
        .begin(source(parallel.tenant_id), &parallel.encode_json()?)
        .await?
    else {
        panic!("second capacity request")
    };
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut **second.transaction())
        .await?;
    let err = second
        .revalidate_project_capacity()
        .await
        .err()
        .expect("allocation must serialize");
    match err {
        ScheduleAdmissionStoreError::Database(e) => assert_eq!(
            e.as_database_error().and_then(|e| e.code()).as_deref(),
            Some("55P03")
        ),
        _ => panic!("expected allocation lock timeout"),
    };
    drop(second);
    // The existing manual allocation epoch uses the same lock, not a separate schedule mutex.
    let mut mutation = admin.begin().await?;
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *mutation)
        .await?;
    let err=sqlx::query("INSERT INTO multica.project_execution_resource_admission_lock(tenant_id,project_id,allocation_epoch) VALUES($1,$2,1) ON CONFLICT(tenant_id,project_id) DO UPDATE SET allocation_epoch=multica.project_execution_resource_admission_lock.allocation_epoch+1")
 .bind(command.tenant_id).bind(t.project_id).execute(&mut *mutation).await.expect_err("manual allocation must serialize");
    assert_eq!(
        err.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("55P03")
    );
    mutation.rollback().await?;
    let mut mutation = admin.begin().await?;
    sqlx::query("SET LOCAL lock_timeout='150ms'")
        .execute(&mut *mutation)
        .await?;
    let err=sqlx::query("UPDATE multica.project_execution_resource_quota SET valid_to=clock_timestamp() WHERE quota_revision_id=$1")
 .bind(quota_id).execute(&mut *mutation).await.expect_err("quota publication must wait");
    assert_eq!(
        err.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("55P03")
    );
    mutation.rollback().await?;
    drop(pending);
    let epoch_count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.project_execution_resource_admission_lock WHERE tenant_id=$1 AND project_id=$2")
 .bind(command.tenant_id).bind(t.project_id).fetch_one(admin).await?;
    assert_eq!(
        epoch_count, 0,
        "dropped capacity observation rolls back Work epoch"
    );
    // A stale REPEATABLE READ snapshot must fail after a committed manual allocation epoch.
    let mut stale = admin.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *stale)
        .await?;
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut *stale)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true),set_config('app.actor_id',$2,true)")
        .bind(command.tenant_id.to_string())
        .bind(command.occurrence.run_as_actor_id.to_string())
        .execute(&mut *stale)
        .await?;
    sqlx::query("SELECT count(*) FROM multica.schedule_admission_inbox")
        .execute(&mut *stale)
        .await?;
    sqlx::query("INSERT INTO multica.project_execution_resource_admission_lock(tenant_id,project_id,allocation_epoch) VALUES($1,$2,1)")
        .bind(command.tenant_id).bind(t.project_id).execute(admin).await?;
    let err = sqlx::query(
        "SELECT multica.read_schedule_project_capacity_current($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(command.tenant_id)
    .bind(command.occurrence.run_as_actor_id)
    .bind(t.project_id)
    .bind(t.branch_id)
    .bind(t.engineering_run_id)
    .bind(t.repository_id)
    .bind(t.worktree_id)
    .bind(t.work_item_id)
    .execute(&mut *stale)
    .await
    .expect_err("stale allocation snapshot must not pass");
    assert_eq!(
        err.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("40001")
    );
    stale.rollback().await?;
    // Unstarted Run and reservation are disposable admin fixtures, never production writes.
    let run_id = Uuid::new_v4();
    seed_provisional_run(admin, command, run_id).await?;
    let reservation_id = Uuid::new_v4();
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("INSERT INTO multica.project_execution_resource_reservation(reservation_id,tenant_id,project_id,work_item_id,worktree_id,run_id,quota_version,quota_revision_id,admission_fence_id,reserved_by,correlation_id,lease_expires_at,max_rss_bytes,max_cpu_ms,max_runtime_ms,max_child_processes,max_parallel_tools,max_provider_calls,max_output_bytes,max_event_buffer_bytes) VALUES($1,$2,$3,$4,$5,$6,1,$7,$8,$9,$10,clock_timestamp()+interval '1 hour',$11,$12,$13,$14,$15,$16,$17,$18)")
 .bind(reservation_id).bind(command.tenant_id).bind(t.project_id).bind(t.work_item_id).bind(t.worktree_id).bind(run_id).bind(quota_id)
 .bind(Uuid::new_v4()).bind(command.occurrence.run_as_actor_id).bind(command.correlation_id).bind(b.max_rss_bytes).bind(b.max_cpu_ms)
 .bind(b.max_runtime_ms).bind(b.max_child_processes).bind(b.max_parallel_tools).bind(b.max_provider_calls)
 .bind(b.max_output_bytes).bind(b.max_event_buffer_bytes).execute(&mut *fixture).await?;
    fixture.commit().await?;
    // Expired pending is ignored, expired active still occupies capacity, released is ignored.
    for (sql,expected) in [
 ("UPDATE multica.project_execution_resource_reservation SET lease_expires_at=clock_timestamp()+interval '1 hour' WHERE reservation_id=$1",false),
 ("UPDATE multica.project_execution_resource_reservation SET created_at=clock_timestamp()-interval '2 hours',lease_expires_at=clock_timestamp()-interval '1 hour' WHERE reservation_id=$1",true),
 ("UPDATE multica.project_execution_resource_reservation SET reservation_state='active',activated_at=clock_timestamp() WHERE reservation_id=$1",false),
 ("UPDATE multica.project_execution_resource_reservation SET reservation_state='released',released_at=clock_timestamp() WHERE reservation_id=$1",true)] {
  let mut change=admin.begin().await?; sqlx::query("SET LOCAL session_replication_role=replica").execute(&mut *change).await?;
  sqlx::query(sql).bind(reservation_id).execute(&mut *change).await?; change.commit().await?;
  let ScheduleAdmissionStart::New(mut pending)=store.begin(source(parallel.tenant_id),&parallel.encode_json()?).await? else {panic!("usage request")};
  let result=pending.revalidate_project_capacity().await;
  if expected {assert!(result.is_ok());} else {assert!(matches!(result,Err(ScheduleAdmissionStoreError::PrerequisiteRejected(ScheduleAdmissionRejectionCodeV1::QuotaExceeded))));}
  drop(pending);
 }
    verify_schedule_preparation(admin, store, command, parallel).await?;
    let mut change = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *change)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_quota SET lifecycle_state='disabled' WHERE quota_revision_id=$1").bind(quota_id).execute(&mut *change).await?;
    change.commit().await?;
    let ScheduleAdmissionStart::New(mut pending) = store
        .begin(source(command.tenant_id), &command.encode_json()?)
        .await?
    else {
        panic!("disabled quota request")
    };
    assert!(matches!(
        pending.revalidate_project_capacity().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::QuotaExceeded
        ))
    ));
    drop(pending);
    Ok(())
}

async fn verify_schedule_preparation(
    admin: &sqlx::PgPool,
    store: &PostgresScheduleAdmissionStore,
    base: &ScheduleRunAdmissionCommandV1,
    parallel: &ScheduleRunAdmissionCommandV1,
) -> Result<(), Box<dyn std::error::Error>> {
    let t = &base.occurrence.rule_snapshot.target;
    let fresh = || {
        let mut cmd = base.clone();
        cmd.request_id = Uuid::new_v4();
        cmd.occurrence.occurrence_id = Uuid::new_v4();
        cmd
    };
    // Missing lifecycle and contract are recorded as immutable rejected outcomes, never fake snapshots.
    let missing = fresh();
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(missing.tenant_id), &missing.encode_json()?)
        .await?
    else {
        panic!("missing lifecycle prep")
    };
    let result = pending.prepare_and_commit().await?;
    assert_eq!(result.outcome, ScheduleRunAdmissionOutcomeV1::Rejected);
    assert_eq!(
        result.rejection_code,
        Some(ScheduleAdmissionRejectionCodeV1::TargetUnavailable)
    );
    sqlx::query("INSERT INTO multica.task_lifecycle_current(work_item_id,tenant_id,active_worktree_id,status,claimed_by) VALUES($1,$2,$3,'in_progress',$4)")
 .bind(t.work_item_id).bind(base.tenant_id).bind(t.worktree_id).bind(base.occurrence.run_as_actor_id).execute(admin).await?;
    let missing = fresh();
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(missing.tenant_id), &missing.encode_json()?)
        .await?
    else {
        panic!("missing contract prep")
    };
    assert_eq!(
        pending.prepare_and_commit().await?.outcome,
        ScheduleRunAdmissionOutcomeV1::Rejected
    );
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query(r#"INSERT INTO multica.task_contract(tenant_id,project_id,work_item_id,version,goal,scope,acceptance_criteria,changed_by) VALUES($1,$2,$3,1,'verify actual task','{}','["ac-test"]',$4)"#)
 .bind(base.tenant_id).bind(t.project_id).bind(t.work_item_id).bind(base.occurrence.run_as_actor_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    // Wrong lifecycle owner and pending human review remain ineligible.
    for (sql,restore) in [
 ("UPDATE multica.task_lifecycle_current SET claimed_by=gen_random_uuid() WHERE work_item_id=$1","UPDATE multica.task_lifecycle_current SET claimed_by=$2 WHERE work_item_id=$1"),
 ("UPDATE multica.task_lifecycle_current SET review_state='pending_review' WHERE work_item_id=$1","UPDATE multica.task_lifecycle_current SET review_state='none' WHERE work_item_id=$1")
 ] {
  sqlx::query(sql).bind(t.work_item_id).execute(admin).await?;
  let cmd=fresh();let ScheduleAdmissionStart::New(pending)=store.begin(source(cmd.tenant_id),&cmd.encode_json()?).await? else {panic!("invalid lifecycle prep")};
  assert_eq!(pending.prepare_and_commit().await?.outcome,ScheduleRunAdmissionOutcomeV1::Rejected);
  if restore.contains("$2") {sqlx::query(restore).bind(t.work_item_id).bind(base.occurrence.run_as_actor_id).execute(admin).await?;}
  else {sqlx::query(restore).bind(t.work_item_id).execute(admin).await?;}
 }
    // Scope-valid but elapsed occurrence never creates a Run.
    let mut expired = fresh();
    expired.occurrence.key.scheduled_for_utc = Utc::now() - chrono::Duration::hours(2);
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(expired.tenant_id), &expired.encode_json()?)
        .await?
    else {
        panic!("expired prep")
    };
    assert_eq!(
        pending.prepare_and_commit().await?.rejection_code,
        Some(ScheduleAdmissionRejectionCodeV1::OccurrenceExpired)
    );
    // Native preparation cannot commit without its paired admission receipt/result.
    let orphan = fresh();
    let orphan_id = Uuid::new_v4();
    let profile: serde_json::Value = sqlx::query_scalar("SELECT profile_document FROM multica.agent_execution_profile WHERE tenant_id=$1 AND profile_id=$2 AND valid_to IS NULL")
        .bind(base.tenant_id).bind(t.execution_profile_id).fetch_one(admin).await?;
    let mut unpaired = admin.begin().await?;
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut *unpaired)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true),set_config('app.actor_id',$2,true)")
        .bind(base.tenant_id.to_string())
        .bind(base.occurrence.run_as_actor_id.to_string())
        .execute(&mut *unpaired)
        .await?;
    let outcome: String =
        sqlx::query_scalar("SELECT multica.prepare_schedule_execution_run($1,$2,$3,$4,$5)")
            .bind(orphan.encode_json()?)
            .bind(serde_json::to_string(&profile)?)
            .bind(orphan_id)
            .bind(Uuid::new_v4())
            .bind(Uuid::new_v4())
            .fetch_one(&mut *unpaired)
            .await?;
    assert_eq!(outcome, "prepared");
    assert!(
        unpaired.commit().await.is_err(),
        "orphan Run/reservation must not commit"
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run WHERE run_id=$1")
            .bind(orphan_id)
            .fetch_one(admin)
            .await?;
    assert_eq!(count, 0);
    // Real time elapses after staging Run facts; the Rust catalog fence must abort all writes.
    let late = fresh();
    sqlx::raw_sql("CREATE FUNCTION multica.delay_d9_test_event() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(5.1); RETURN NEW; END $$; CREATE TRIGGER d9_test_delay AFTER INSERT ON multica.task_execution_run_event FOR EACH ROW EXECUTE FUNCTION multica.delay_d9_test_event()")
        .execute(admin).await?;
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(late.tenant_id), &late.encode_json()?)
        .await?
    else {
        panic!("late fence prep")
    };
    assert!(matches!(
        pending.prepare_and_commit().await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    sqlx::raw_sql("DROP TRIGGER d9_test_delay ON multica.task_execution_run_event; DROP FUNCTION multica.delay_d9_test_event()").execute(admin).await?;
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run WHERE tenant_id=$1 AND schedule_admission_request_id=$2")
        .bind(late.tenant_id).bind(late.request_id).fetch_one(admin).await?;
    assert_eq!(count, 0);
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.schedule_admission_inbox WHERE tenant_id=$1 AND request_id=$2")
        .bind(late.tenant_id).bind(late.request_id).fetch_one(admin).await?;
    assert_eq!(count, 0);
    let cmd = fresh();
    // Inject a result persistence failure after real owner Run/reservation writes, then retry exactly.
    let function=format!("CREATE OR REPLACE FUNCTION multica.reject_d9_test_result() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.request_id='{}'::uuid THEN RAISE EXCEPTION 'd9 result failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER d9_test_result BEFORE INSERT ON multica.schedule_admission_result_outbox FOR EACH ROW EXECUTE FUNCTION multica.reject_d9_test_result()",cmd.request_id);
    sqlx::raw_sql(&function).execute(admin).await?;
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(cmd.tenant_id), &cmd.encode_json()?)
        .await?
    else {
        panic!("rollback prep")
    };
    assert!(pending.prepare_and_commit().await.is_err());
    sqlx::raw_sql("DROP TRIGGER d9_test_result ON multica.schedule_admission_result_outbox; DROP FUNCTION multica.reject_d9_test_result()").execute(admin).await?;
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run WHERE tenant_id=$1 AND schedule_admission_request_id=$2")
 .bind(cmd.tenant_id).bind(cmd.request_id).fetch_one(admin).await?;
    assert_eq!(count, 0);
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(cmd.tenant_id), &cmd.encode_json()?)
        .await?
    else {
        panic!("retry rolled-back prep")
    };
    let result = pending.prepare_and_commit().await?;
    assert_eq!(result.outcome, ScheduleRunAdmissionOutcomeV1::Accepted);
    let run_id = result.task_execution_run_id.unwrap();
    let row=sqlx::query("SELECT r.started_at,r.execution_channel,r.task_contract_version,r.task_snapshot,r.acceptance_snapshot,r.execution_profile_snapshot,r.infrastructure_binding_snapshot,r.infrastructure_binding_digest,r.engineering_run_id,s.reservation_state,s.lease_expires_at, (SELECT count(*) FROM multica.task_execution_run_event e WHERE e.tenant_id=r.tenant_id AND e.run_id=r.run_id AND (e.event_type='run_started' OR e.execution_state IS NOT NULL)) AS execution_events FROM multica.task_execution_run r JOIN multica.project_execution_resource_reservation s ON s.tenant_id=r.tenant_id AND s.run_id=r.run_id WHERE r.run_id=$1")
 .bind(run_id).fetch_one(admin).await?;
    assert!(row
        .try_get::<Option<chrono::DateTime<Utc>>, _>("started_at")?
        .is_none());
    assert_eq!(row.try_get::<String, _>("execution_channel")?, "agent");
    assert_eq!(row.try_get::<i64, _>("task_contract_version")?, 1);
    assert_eq!(
        row.try_get::<serde_json::Value, _>("task_snapshot")?["lifecycle_status"],
        "in_progress"
    );
    assert_eq!(
        row.try_get::<serde_json::Value, _>("acceptance_snapshot")?["acceptance_criteria"],
        serde_json::json!(["ac-test"])
    );
    assert!(
        row.try_get::<serde_json::Value, _>("execution_profile_snapshot")?["content_digest"]
            .is_string()
    );
    assert_eq!(
        row.try_get::<serde_json::Value, _>("infrastructure_binding_snapshot")?,
        serde_json::to_value(&cmd.infrastructure_binding)?
    );
    assert_eq!(
        row.try_get::<Vec<u8>, _>("infrastructure_binding_digest")?,
        cmd.binding_digest()?.to_vec()
    );
    assert_eq!(
        row.try_get::<Uuid, _>("engineering_run_id")?,
        t.engineering_run_id
    );
    assert_eq!(row.try_get::<String, _>("reservation_state")?, "pending");
    assert!(row.try_get::<chrono::DateTime<Utc>, _>("lease_expires_at")? > Utc::now());
    assert_eq!(row.try_get::<i64, _>("execution_events")?, 0);
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.infrastructure_capacity_demand_outbox WHERE tenant_id=$1 AND task_execution_run_id=$2")
 .bind(cmd.tenant_id).bind(run_id).fetch_one(admin).await?;
    assert_eq!(count, 0, "demand must await Schedule confirmation");
    let ScheduleAdmissionStart::Replay(replay) = store
        .begin(source(cmd.tenant_id), &cmd.encode_json()?)
        .await?
    else {
        panic!("prepared result replay")
    };
    assert_eq!(replay, result);
    // Other task receives an immutable quota rejection while the accepted pending reservation occupies the slot.
    let mut other = parallel.clone();
    other.request_id = Uuid::new_v4();
    other.occurrence.occurrence_id = Uuid::new_v4();
    let ScheduleAdmissionStart::New(pending) = store
        .begin(source(other.tenant_id), &other.encode_json()?)
        .await?
    else {
        panic!("occupied project prep")
    };
    assert_eq!(
        pending.prepare_and_commit().await?.rejection_code,
        Some(ScheduleAdmissionRejectionCodeV1::QuotaExceeded)
    );
    verify_schedule_compensation(admin, store, &cmd, &result).await?;
    verify_schedule_confirmation(admin, store, &cmd).await?;
    Ok(())
}

/// Canonical Rule/occurrence fixtures; claiming and the admission handoff use real adapters.
async fn seed_schedule_handoff(
    admin: &sqlx::PgPool,
    schedule: &star_pg_adapter::repository::PgAutomationScheduleRepository,
    cmd: &ScheduleRunAdmissionCommandV1,
) -> Result<domain_automation::schedule::AutomationOccurrenceLeaseFenceV1, Box<dyn std::error::Error>>
{
    use sha2::{Digest, Sha256};
    let r = &cmd.occurrence.rule_snapshot;
    let t = &r.target;
    sqlx::query("INSERT INTO automation.schedule_rule_revision
      (tenant_id,project_id,rule_id,rule_version,enabled,cron_expression,time_zone,parser_version,tzdb_version,
       dst_gap_policy,dst_fold_policy,overlap_policy,overlap_max_active,misfire_policy,misfire_max_occurrences,
       pause_policy,retry_max_attempts,retry_initial_backoff_seconds,retry_max_backoff_seconds,deadline_seconds,
       branch_id,engineering_run_id,repository_id,worktree_id,work_item_id,execution_profile_id,
       execution_profile_version,execution_profile_digest,hook_set_version,hook_set_digest,run_as_actor_id,changed_by)
      VALUES($1,$2,$3,1,true,$4,$5,$6,$7,'skip','earlier_instant','queue_one',1,'catch_up',3,'coalesce_latest',3,5,300,3600,
       $8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$18) ON CONFLICT DO NOTHING")
        .bind(cmd.tenant_id).bind(r.project_id).bind(r.rule_id.as_uuid())
        .bind(&r.cron_expression).bind(&r.time_zone).bind(&r.parser_version).bind(&r.tzdb_version)
        .bind(t.branch_id).bind(t.engineering_run_id).bind(t.repository_id).bind(t.worktree_id).bind(t.work_item_id)
        .bind(t.execution_profile_id).bind(i64::try_from(t.execution_profile_version)?)
        .bind(&t.execution_profile_digest).bind(i64::try_from(t.hook_set_version)?)
        .bind(&t.hook_set_digest).bind(r.run_as_actor_id).execute(admin).await?;
    let target = serde_json::to_value(t)?;
    let digest: String = Sha256::digest(serde_json::to_vec(&target)?)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    sqlx::query("INSERT INTO automation.occurrence
      (tenant_id,project_id,occurrence_id,rule_id,rule_version,run_as_actor_id,scheduled_for_utc,
       scheduled_local_label,utc_offset_seconds,parser_version,tzdb_version,target_snapshot,target_snapshot_digest,materialized_at)
      VALUES($1,$2,$3,$4,1,$5,$6,$7,$8,$9,$10,$11,$12,clock_timestamp())")
        .bind(cmd.tenant_id).bind(r.project_id).bind(cmd.occurrence.occurrence_id).bind(r.rule_id.as_uuid())
        .bind(r.run_as_actor_id).bind(cmd.occurrence.key.scheduled_for_utc)
        .bind(&cmd.occurrence.scheduled_local_label).bind(cmd.occurrence.utc_offset_seconds)
        .bind(&r.parser_version).bind(&r.tzdb_version).bind(target).bind(digest).execute(admin).await?;
    sqlx::query("INSERT INTO automation.occurrence_dispatch
      (tenant_id,occurrence_id,dispatch_state,next_attempt_at,occurrence_deadline_at,retention_period)
      VALUES($1,$2,'pending',clock_timestamp(),$3+interval '1 hour',interval '1 day')")
        .bind(cmd.tenant_id).bind(cmd.occurrence.occurrence_id).bind(cmd.occurrence.key.scheduled_for_utc)
        .execute(admin).await?;
    let mut fences = schedule
        .claim_due_occurrences(cmd.tenant_id, Uuid::new_v4(), 60, 1)
        .await?;
    let fence = fences
        .pop()
        .expect("canonical pending occurrence is claimed");
    assert_eq!(fence.occurrence_id, cmd.occurrence.occurrence_id);
    Ok(fence)
}

struct AdmissionAuthorizationFixture {
    scope: platform_messaging_runtime::admission_receive::AdmissionReceiveScope,
    log: String,
    log_generation: Uuid,
}

impl platform_messaging_runtime::admission_receive::AdmissionReceiveAuthorization
    for AdmissionAuthorizationFixture
{
    async fn verify(
        &self,
        origin: &platform_messaging_runtime::admission_receive::AdmissionReceiveOrigin,
        _: &[u8],
    ) -> Result<(), platform_messaging_runtime::admission_receive::AdmissionReceiveError> {
        if origin.scope != self.scope
            || origin.provider_identity != "ISOLATED_TEST_ONLY"
            || origin.log_id != self.log
            || origin.log_generation != self.log_generation
        {
            return Err(
                platform_messaging_runtime::admission_receive::AdmissionReceiveError::Unauthorized,
            );
        }
        Ok(()) // Explicit test fixture; neither credentials nor production authentication evidence.
    }
}

#[allow(clippy::too_many_arguments)] // One bounded integration fixture for the three owner wire kinds.
async fn receive_admission_broker_fixture<
    H: platform_messaging_runtime::admission_receive::AdmissionReceiveHandler,
>(
    tenant: Uuid,
    sender: Uuid,
    recipient: Uuid,
    kind: platform_messaging_runtime::admission_receive::AdmissionReceiveKind,
    message_id: Uuid,
    wire: &[u8],
    handler: H,
    fail_commit: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use async_nats::jetstream::{
        consumer::{pull, AckPolicy},
        message::PublishMessage,
        stream,
    };
    use platform_messaging_runtime::{admission_receive::*, admission_receive_jetstream::*};
    let Ok(url) = std::env::var("STAR_ADMISSION_DELIVERY_TEST_NATS_URL") else {
        return Ok(());
    };
    let context = async_nats::jetstream::new(async_nats::connect(url).await?);
    let scope = AdmissionReceiveScope {
        tenant_id: tenant,
        sender_service_id: sender,
        recipient_service_id: recipient,
        kind,
    };
    let (owner, wire_kind) = match kind {
        AdmissionReceiveKind::Request => ("automation", "request"),
        AdmissionReceiveKind::Result => ("multica", "result"),
        AdmissionReceiveKind::Resolution => ("automation", "resolution"),
    };
    let subject = format!("star.{owner}.admission.v1.{tenant}.{recipient}.{wire_kind}");
    let name = format!("RECEIVE_{}", Uuid::new_v4().simple());
    let stream = context
        .create_stream(stream::Config {
            name: name.clone(),
            subjects: vec![subject.clone()],
            max_bytes: 65536,
            max_messages: 8,
            max_message_size: 16384,
            ..Default::default()
        })
        .await?;
    let consumer = stream
        .create_consumer(pull::Config {
            durable_name: Some("admission_fixture".into()),
            filter_subject: subject.clone(),
            ack_policy: AckPolicy::Explicit,
            max_ack_pending: 1,
            max_waiting: 1,
            max_batch: 1,
            max_bytes: 20480,
            max_deliver: 12,
            ack_wait: std::time::Duration::from_secs(20),
            ..Default::default()
        })
        .await?;
    let ack = context
        .send_publish(
            subject,
            PublishMessage::build()
                .payload(wire.to_vec().into())
                .message_id(format!(
                    "admission.v1.{owner}.{wire_kind}.{tenant}.{message_id}"
                ))
                .expected_stream(&name),
        )
        .await?
        .await?;
    assert_eq!(ack.stream, name);
    let stream_generation = Uuid::new_v4();
    let transport = JetStreamAdmissionReceiveTransport::new(
        consumer,
        scope,
        name.clone(),
        stream_generation,
        String::new(),
        "ISOLATED_TEST_ONLY".into(),
    )?;
    let auth = AdmissionAuthorizationFixture {
        scope,
        log: name.clone(),
        log_generation: stream_generation,
    };
    let mut receiver = AdmissionReceiver::new(transport, handler, auth, scope)?;
    let (_control, mut cancellation) = tokio::sync::watch::channel(false);
    let outcome = receiver.consume_one(&mut cancellation).await;
    let mut observer: async_nats::jetstream::consumer::PullConsumer = stream
        .get_consumer("admission_fixture")
        .await
        .map_err(|error| -> Box<dyn std::error::Error> { error })?;
    if fail_commit {
        assert_eq!(outcome, Err(AdmissionReceiveError::OwnerCommit));
        assert_eq!(observer.info().await?.num_ack_pending, 1); // Real DB fault left delivery unconfirmed.
    } else {
        assert_eq!(outcome, Ok(Some(message_id)));
        assert_eq!(observer.info().await?.num_ack_pending, 0);
        assert_eq!(
            observer.cached_info().ack_floor.stream_sequence,
            ack.sequence
        );
    }
    context.delete_stream(name).await?;
    Ok(())
}

async fn verify_schedule_confirmation(
    admin: &sqlx::PgPool,
    store: &PostgresScheduleAdmissionStore,
    previous: &ScheduleRunAdmissionCommandV1,
) -> Result<(), Box<dyn std::error::Error>> {
    // Finish compensation before the positive case retains its capacity demand/reservation.
    verify_schedule_fence_compensation(admin, store, previous).await?;
    let mut cmd = previous.clone();
    cmd.request_id = Uuid::new_v4();
    cmd.occurrence.occurrence_id = Uuid::new_v4();
    // PostgreSQL timestamps have microsecond precision; the immutable wire uses that precision.
    cmd.occurrence.key.scheduled_for_utc =
        chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    let schedule = star_pg_adapter::repository::PgAutomationScheduleRepository::new(admin.clone());
    let run_service = Uuid::new_v4();
    let fence = seed_schedule_handoff(admin, &schedule, &cmd).await?;
    let request_wire = schedule
        .persist_admission_request(&fence, run_service, &cmd)
        .await?;
    assert_eq!(
        schedule
            .persist_admission_request(&fence, run_service, &cmd)
            .await?,
        request_wire
    );
    receive_admission_broker_fixture(
        cmd.tenant_id,
        source(cmd.tenant_id).schedule_service_id,
        run_service,
        platform_messaging_runtime::admission_receive::AdmissionReceiveKind::Request,
        cmd.request_id,
        &request_wire,
        store.clone(),
        false,
    )
    .await?;
    let result = match store.begin(source(cmd.tenant_id), &request_wire).await? {
        ScheduleAdmissionStart::New(pending) => pending.prepare_and_commit().await?,
        ScheduleAdmissionStart::Replay(result) => result,
    };
    assert_eq!(result.outcome, ScheduleRunAdmissionOutcomeV1::Accepted);
    let run = result.task_execution_run_id.unwrap();
    let result_wire = result.encode_json()?;
    assert!(schedule
        .resolve_admission_result(cmd.tenant_id, Uuid::new_v4(), &result_wire)
        .await
        .is_err());
    let mut wrong_result = result.clone();
    wrong_result.binding_digest[0] ^= 1;
    assert!(schedule
        .resolve_admission_result(cmd.tenant_id, run_service, &wrong_result.encode_json()?)
        .await
        .is_err());
    // The final Outbox insert must roll back the link, event and Inbox together.
    sqlx::raw_sql("CREATE FUNCTION automation.fail_d12_resolution() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'D12 fault'; END $$; CREATE TRIGGER d12_fail BEFORE INSERT ON automation.run_admission_resolution_outbox FOR EACH ROW EXECUTE FUNCTION automation.fail_d12_resolution()").execute(admin).await?;
    receive_admission_broker_fixture(
        cmd.tenant_id,
        run_service,
        source(cmd.tenant_id).schedule_service_id,
        platform_messaging_runtime::admission_receive::AdmissionReceiveKind::Result,
        result.result_id,
        &result_wire,
        schedule.clone(),
        true,
    )
    .await?;
    assert!(schedule
        .resolve_admission_result(cmd.tenant_id, run_service, &result_wire)
        .await
        .is_err());
    sqlx::raw_sql("DROP TRIGGER d12_fail ON automation.run_admission_resolution_outbox; DROP FUNCTION automation.fail_d12_resolution()").execute(admin).await?;
    let linked: Option<Uuid> = sqlx::query_scalar("SELECT admitted_run_id FROM automation.occurrence_dispatch WHERE tenant_id=$1 AND occurrence_id=$2").bind(cmd.tenant_id).bind(cmd.occurrence.occurrence_id).fetch_one(admin).await?;
    assert_eq!(linked, None);
    let receipts: i64 = sqlx::query_scalar("SELECT count(*) FROM automation.run_admission_result_inbox WHERE tenant_id=$1 AND request_id=$2").bind(cmd.tenant_id).bind(cmd.request_id).fetch_one(admin).await?;
    assert_eq!(receipts, 0);
    receive_admission_broker_fixture(
        cmd.tenant_id,
        run_service,
        source(cmd.tenant_id).schedule_service_id,
        platform_messaging_runtime::admission_receive::AdmissionReceiveKind::Result,
        result.result_id,
        &result_wire,
        schedule.clone(),
        false,
    )
    .await?;
    let resolution = schedule
        .resolve_admission_result(cmd.tenant_id, run_service, &result_wire)
        .await?
        .expect("live original Schedule fence confirms");
    assert_eq!(
        resolution.decision,
        ScheduleAdmissionResolutionKindV1::Confirm
    );
    assert_eq!(
        schedule
            .resolve_admission_result(cmd.tenant_id, run_service, &result_wire)
            .await?,
        Some(resolution.clone())
    );
    let wire = resolution.encode_json()?;
    let mut wrong_source = source(cmd.tenant_id);
    wrong_source.schedule_service_id = Uuid::new_v4();
    assert!(matches!(
        store.confirm_admission(wrong_source, &wire).await,
        Err(ScheduleAdmissionStoreError::UntrustedScope)
    ));
    assert!(store
        .confirm_admission(source(Uuid::new_v4()), &wire)
        .await
        .is_err());
    let mut wrong = resolution.clone();
    wrong.binding_digest[0] ^= 1;
    assert!(store
        .confirm_admission(source(cmd.tenant_id), &wrong.encode_json()?)
        .await
        .is_err());
    // Revoke current Project role after preparation; a fresh confirmation must reauthorize.
    let grant:Uuid=sqlx::query_scalar("SELECT id FROM permission.project_role_binding WHERE tenant_id=$1 AND project_id=$2 AND user_id=$3 AND valid_to IS NULL")
        .bind(cmd.tenant_id).bind(cmd.occurrence.rule_snapshot.target.project_id).bind(cmd.occurrence.run_as_actor_id).fetch_one(admin).await?;
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query(
        "UPDATE permission.project_role_binding SET valid_to=clock_timestamp() WHERE id=$1",
    )
    .bind(grant)
    .execute(&mut *fixture)
    .await?;
    fixture.commit().await?;
    assert!(matches!(
        store.confirm_admission(source(cmd.tenant_id), &wire).await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::RunAsUnauthorized
        ))
    ));
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE permission.project_role_binding SET valid_to=NULL WHERE id=$1")
        .bind(grant)
        .execute(&mut *fixture)
        .await?;
    fixture.commit().await?;
    let lease:chrono::DateTime<Utc>=sqlx::query_scalar("SELECT lease_expires_at FROM multica.project_execution_resource_reservation WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(run).fetch_one(admin).await?;
    let created:chrono::DateTime<Utc>=sqlx::query_scalar("SELECT created_at FROM multica.project_execution_resource_reservation WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(run).fetch_one(admin).await?;
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_reservation SET created_at=clock_timestamp()-interval '5 minutes',lease_expires_at=clock_timestamp()-interval '1 second' WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(run).execute(&mut *fixture).await?;
    fixture.commit().await?;
    assert!(store
        .confirm_admission(source(cmd.tenant_id), &wire)
        .await
        .is_err());
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_reservation SET lease_expires_at=$3,created_at=$4 WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(run).bind(lease).bind(created).execute(&mut *fixture).await?;
    fixture.commit().await?;
    // A revoked quota must reject fresh confirmation despite an existing preparation reservation.
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_quota SET lifecycle_state='disabled' WHERE tenant_id=$1 AND project_id=$2 AND valid_to IS NULL")
        .bind(cmd.tenant_id).bind(cmd.occurrence.rule_snapshot.target.project_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    assert!(store
        .confirm_admission(source(cmd.tenant_id), &wire)
        .await
        .is_err());
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_quota SET lifecycle_state='active' WHERE tenant_id=$1 AND project_id=$2 AND valid_to IS NULL")
        .bind(cmd.tenant_id).bind(cmd.occurrence.rule_snapshot.target.project_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    // Last receipt persistence fault must roll back the staged confirmation event and demand.
    sqlx::raw_sql("CREATE FUNCTION multica.reject_d11_receipt() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'd11 receipt failure'; END $$; CREATE TRIGGER d11_fail BEFORE INSERT ON multica.schedule_confirmation_inbox FOR EACH ROW EXECUTE FUNCTION multica.reject_d11_receipt()").execute(admin).await?;
    assert!(store
        .confirm_admission(source(cmd.tenant_id), &wire)
        .await
        .is_err());
    sqlx::raw_sql("DROP TRIGGER d11_fail ON multica.schedule_confirmation_inbox; DROP FUNCTION multica.reject_d11_receipt()").execute(admin).await?;
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.infrastructure_capacity_demand_outbox WHERE tenant_id=$1 AND task_execution_run_id=$2")
        .bind(cmd.tenant_id).bind(run).fetch_one(admin).await?;
    assert_eq!(count, 0);
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run_event WHERE tenant_id=$1 AND run_id=$2 AND event_type='schedule_admission_confirmed'")
        .bind(cmd.tenant_id).bind(run).fetch_one(admin).await?;
    assert_eq!(count, 0);
    sqlx::raw_sql("CREATE FUNCTION multica.delay_d11_demand() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(5.1); RETURN NEW; END $$; CREATE TRIGGER d11_delay AFTER INSERT ON multica.infrastructure_capacity_demand_outbox FOR EACH ROW EXECUTE FUNCTION multica.delay_d11_demand()").execute(admin).await?;
    assert!(matches!(
        store.confirm_admission(source(cmd.tenant_id), &wire).await,
        Err(ScheduleAdmissionStoreError::PrerequisiteRejected(
            ScheduleAdmissionRejectionCodeV1::TargetUnavailable
        ))
    ));
    sqlx::raw_sql("DROP TRIGGER d11_delay ON multica.infrastructure_capacity_demand_outbox; DROP FUNCTION multica.delay_d11_demand()").execute(admin).await?;
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.schedule_confirmation_inbox WHERE tenant_id=$1 AND request_id=$2")
        .bind(cmd.tenant_id).bind(cmd.request_id).fetch_one(admin).await?;
    assert_eq!(count, 0);
    receive_admission_broker_fixture(
        cmd.tenant_id,
        source(cmd.tenant_id).schedule_service_id,
        run_service,
        platform_messaging_runtime::admission_receive::AdmissionReceiveKind::Resolution,
        resolution.resolution_id,
        &wire,
        store.clone(),
        false,
    )
    .await?;
    let demand = store
        .confirm_admission(source(cmd.tenant_id), &wire)
        .await?;
    assert_eq!(demand.binding, cmd.infrastructure_binding);
    assert_eq!(demand.task_execution_run_id, run);
    assert_eq!(demand.deadline_at, lease);
    let mut tx = admin.begin().await?;
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(Uuid::new_v4().to_string())
        .execute(&mut *tx)
        .await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM multica.schedule_confirmation_inbox")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(count, 0);
    tx.rollback().await?;
    assert_eq!(
        store
            .confirm_admission(source(cmd.tenant_id), &wire)
            .await?,
        demand
    );
    let mut conflict = resolution.clone();
    conflict.resolution_id = Uuid::new_v4();
    assert!(matches!(
        store
            .confirm_admission(source(cmd.tenant_id), &conflict.encode_json()?)
            .await,
        Err(ScheduleAdmissionStoreError::ReplayConflict)
    ));
    let mut compensate = resolution.clone();
    compensate.decision = ScheduleAdmissionResolutionKindV1::Compensate;
    compensate.compensation_reason = Some(ScheduleAdmissionCompensationReasonV1::ScheduleFenceLost);
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &compensate.encode_json()?)
        .await
        .is_err());
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.infrastructure_capacity_demand_outbox WHERE tenant_id=$1 AND task_execution_run_id=$2")
        .bind(cmd.tenant_id).bind(run).fetch_one(admin).await?;
    assert_eq!(count, 1);
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run_event WHERE tenant_id=$1 AND run_id=$2 AND (event_type='run_started' OR execution_state IS NOT NULL)")
        .bind(cmd.tenant_id).bind(run).fetch_one(admin).await?;
    assert_eq!(count, 0);
    verify_schedule_rejection_retry(admin, store).await?;
    Ok(())
}

/// Rejected-result supplier fixtures exercise Schedule's real retry and exhaustion writer.
async fn verify_schedule_rejection_retry(
    admin: &sqlx::PgPool,
    run_store: &PostgresScheduleAdmissionStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = command_fixture();
    cmd.occurrence.key.scheduled_for_utc =
        chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    let schedule = star_pg_adapter::repository::PgAutomationScheduleRepository::new(admin.clone());
    let mut fence = seed_schedule_handoff(admin, &schedule, &cmd).await?;
    let service = Uuid::new_v4();
    for attempt in 1..=3 {
        cmd.admission_attempt = attempt;
        cmd.request_id = Uuid::new_v4();
        schedule
            .persist_admission_request(&fence, service, &cmd)
            .await?;
        let typed_result = rejected_result(&cmd)?;
        if attempt == 1 {
            let ScheduleAdmissionStart::New(pending) = run_store
                .begin(source(cmd.tenant_id), &cmd.encode_json()?)
                .await?
            else {
                panic!("new delivery result")
            };
            pending.commit_result(&typed_result).await?;
            verify_admission_delivery(admin, &cmd, &typed_result).await?;
        }
        let result = typed_result.encode_json()?;
        assert_eq!(
            schedule
                .resolve_admission_result(cmd.tenant_id, service, &result)
                .await?,
            None
        );
        assert_eq!(
            schedule
                .resolve_admission_result(cmd.tenant_id, service, &result)
                .await?,
            None
        );
        let state: String = sqlx::query_scalar("SELECT dispatch_state FROM automation.occurrence_dispatch WHERE tenant_id=$1 AND occurrence_id=$2")
            .bind(cmd.tenant_id).bind(cmd.occurrence.occurrence_id).fetch_one(admin).await?;
        assert_eq!(state, if attempt < 3 { "retry_wait" } else { "failed" });
        if attempt < 3 {
            // Advance only the retry due time, preserving all production state guards.
            sqlx::query("UPDATE automation.occurrence_dispatch SET next_attempt_at=clock_timestamp() WHERE tenant_id=$1 AND occurrence_id=$2")
                .bind(cmd.tenant_id).bind(cmd.occurrence.occurrence_id).execute(admin).await?;
            fence = schedule
                .claim_due_occurrences(cmd.tenant_id, Uuid::new_v4(), 60, 1)
                .await?
                .pop()
                .unwrap();
        }
    }
    let receipts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM automation.run_admission_result_inbox WHERE tenant_id=$1",
    )
    .bind(cmd.tenant_id)
    .fetch_one(admin)
    .await?;
    assert_eq!(receipts, 3);
    let mut tx = admin.begin().await?;
    sqlx::query("SET LOCAL ROLE star_automation_schedule_owner")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(Uuid::new_v4().to_string())
        .execute(&mut *tx)
        .await?;
    let invisible: i64 =
        sqlx::query_scalar("SELECT count(*) FROM automation.run_admission_result_inbox")
            .fetch_one(&mut *tx)
            .await?;
    assert_eq!(invisible, 0);
    tx.rollback().await?;
    Ok(())
}

async fn verify_admission_delivery(
    admin: &sqlx::PgPool,
    cmd: &ScheduleRunAdmissionCommandV1,
    result: &ScheduleRunAdmissionResultV1,
) -> Result<(), Box<dyn std::error::Error>> {
    use platform_messaging_runtime::admission_delivery::*;
    let profile = AdmissionTransportProfile::new(
        "nats-jetstream".to_owned(),
        "2.12.14".to_owned(),
        [9; 32],
        "test-nats".to_owned(),
    )?;
    sqlx::raw_sql("CREATE ROLE star_delivery_schedule_test_login LOGIN NOINHERIT NOSUPERUSER NOBYPASSRLS; GRANT star_automation_schedule_owner TO star_delivery_schedule_test_login").execute(admin).await?;
    let options: PgConnectOptions = std::env::var("STAR_RUN_OWNER_TEST_DATABASE_URL")?.parse()?;
    let schedule_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(
            options
                .clone()
                .username("star_delivery_schedule_test_login"),
        )
        .await?;
    let run_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options.username("star_admission_test_login"))
        .await?;
    let schedule_delivery =
        AdmissionDeliveryStore::new(schedule_pool.clone(), AdmissionDeliveryOwner::Schedule);
    let run_delivery = AdmissionDeliveryStore::new(run_pool.clone(), AdmissionDeliveryOwner::Run);
    let wrong_owner =
        AdmissionDeliveryStore::new(run_pool.clone(), AdmissionDeliveryOwner::Schedule);
    assert!(wrong_owner
        .claim(cmd.tenant_id, Uuid::new_v4(), 1, 120, &profile)
        .await
        .is_err());
    assert!(schedule_delivery
        .claim(cmd.tenant_id, Uuid::new_v4(), 9, 120, &profile)
        .await
        .is_err());
    assert!(schedule_delivery
        .claim(Uuid::nil(), Uuid::new_v4(), 1, 120, &profile)
        .await
        .is_err());
    let first = schedule_delivery
        .claim(cmd.tenant_id, Uuid::new_v4(), 1, 120, &profile)
        .await?
        .pop()
        .unwrap();
    assert_eq!(first.wire, cmd.encode_json()?);
    assert_eq!(first.message_id, cmd.request_id);
    assert_eq!(first.message_kind, "request");
    schedule_delivery
        .settle(&first, AdmissionDeliveryAction::Retry)
        .await?;
    sqlx::query("UPDATE automation.admission_message_delivery SET next_attempt_at=clock_timestamp() WHERE tenant_id=$1 AND message_id=$2")
        .bind(cmd.tenant_id).bind(first.message_id).execute(admin).await?;
    let replacement_profile = AdmissionTransportProfile::new(
        "star-rust-log".to_owned(),
        "1.0.0".to_owned(),
        [10; 32],
        "test-star-log".to_owned(),
    )?;
    assert!(schedule_delivery
        .claim(
            cmd.tenant_id,
            Uuid::new_v4(),
            1,
            120,
            &replacement_profile,
        )
        .await?
        .is_empty());
    let second = schedule_delivery
        .claim(cmd.tenant_id, Uuid::new_v4(), 1, 120, &profile)
        .await?
        .pop()
        .unwrap();
    assert_eq!(second.attempt_count, 2);
    assert_eq!(second.wire, first.wire);
    assert!(schedule_delivery
        .settle(&first, AdmissionDeliveryAction::Retry)
        .await
        .is_err());
    let mut tx = schedule_pool.begin().await?;
    sqlx::query("SET LOCAL ROLE star_automation_schedule_owner")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(Uuid::new_v4().to_string())
        .execute(&mut *tx)
        .await?;
    let hidden: i64 =
        sqlx::query_scalar("SELECT count(*) FROM automation.admission_message_delivery")
            .fetch_one(&mut *tx)
            .await?;
    assert_eq!(hidden, 0);
    tx.rollback().await?;
    if let Ok(url) = std::env::var("STAR_ADMISSION_DELIVERY_TEST_NATS_URL") {
        use async_nats::jetstream::{message::PublishMessage, stream};
        let context = async_nats::jetstream::new(async_nats::connect(url).await?);
        let name = format!("ADMISSION_{}", Uuid::new_v4().simple());
        let stream = context
            .create_stream(stream::Config {
                name: name.clone(),
                subjects: vec![
                    format!("star.automation.admission.v1.{}.>", cmd.tenant_id),
                    format!("star.multica.admission.v1.{}.>", cmd.tenant_id),
                ],
                max_bytes: 2 * 1024 * 1024,
                max_messages: 100,
                max_message_size: 16384,
                duplicate_window: std::time::Duration::from_secs(600),
                ..Default::default()
            })
            .await?;
        // Simulate process loss after a real broker PubAck, before committing the owner receipt.
        let subject = format!(
            "star.automation.admission.v1.{}.{}.request",
            cmd.tenant_id, second.target_service_id
        );
        let id = format!(
            "admission.v1.automation.request.{}.{}",
            cmd.tenant_id, second.message_id
        );
        let ack = context
            .send_publish(
                subject,
                PublishMessage::build()
                    .payload(second.wire.clone().into())
                    .message_id(id)
                    .expected_stream(&name),
            )
            .await?
            .await?;
        sqlx::query("UPDATE automation.admission_message_delivery SET lease_expires_at=clock_timestamp()-interval '1 second' WHERE tenant_id=$1 AND message_id=$2")
            .bind(cmd.tenant_id).bind(second.message_id).execute(admin).await?;
        let publisher = AdmissionJetStreamPublisher::new(
            schedule_delivery.clone(),
            context.clone(),
            name.clone(),
            String::new(),
            Uuid::new_v4(),
            profile.clone(),
        )?;
        let (_cancel, active) = tokio::sync::watch::channel(false);
        let delivered = publisher
            .publish_due(cmd.tenant_id, Uuid::new_v4(), 1, &active)
            .await?;
        assert_eq!(delivered.delivered, 1);
        let cursor: Vec<u8> = sqlx::query_scalar("SELECT transport_cursor FROM automation.admission_message_delivery_receipt WHERE tenant_id=$1 AND message_kind='request' AND message_id=$2")
            .bind(cmd.tenant_id).bind(cmd.request_id).fetch_one(admin).await?;
        let sequence = i64::from_be_bytes(cursor.as_slice().try_into()?);
        assert_eq!(sequence, i64::try_from(ack.sequence)?);
        let info = stream.get_info().await?;
        assert_eq!(
            info.state.messages, 1,
            "stable ID redelivery is duplicate within broker window"
        );
        let wrong_stream = AdmissionJetStreamPublisher::new(
            run_delivery.clone(),
            context.clone(),
            "ABSENT_ADMISSION_STREAM".to_owned(),
            String::new(),
            Uuid::new_v4(),
            profile.clone(),
        )?;
        assert_eq!(
            wrong_stream
                .publish_due(cmd.tenant_id, Uuid::new_v4(), 1, &active)
                .await?
                .retry_settlements,
            1
        );
        sqlx::query("UPDATE multica.admission_message_delivery SET next_attempt_at=clock_timestamp() WHERE tenant_id=$1 AND message_id=$2")
            .bind(cmd.tenant_id).bind(result.result_id).execute(admin).await?;
        let run_publisher = AdmissionJetStreamPublisher::new(
            run_delivery.clone(),
            context.clone(),
            name.clone(),
            String::new(),
            Uuid::new_v4(),
            profile.clone(),
        )?;
        let (_cancel, cancelled) = tokio::sync::watch::channel(true);
        assert!(
            run_publisher
                .publish_due(cmd.tenant_id, Uuid::new_v4(), 1, &cancelled)
                .await?
                .cancelled
        );
        assert_eq!(
            run_publisher
                .publish_due(cmd.tenant_id, Uuid::new_v4(), 1, &active)
                .await?
                .delivered,
            1
        );
        assert_eq!(stream.get_info().await?.state.messages, 2);
        context.delete_stream(name).await?;
    } else {
        // Schema-only mode supplies an explicit receipt fixture, never claimed as broker proof.
        schedule_delivery
            .settle(
                &second,
                AdmissionDeliveryAction::Delivered {
                    receipt: AdmissionTransportReceipt::new(
                        profile.clone(),
                        "ACK_FIXTURE".to_owned(),
                        Uuid::new_v4(),
                        42_i64.to_be_bytes().to_vec(),
                        &second.wire,
                    )?,
                },
            )
            .await?;
        let run_claim = run_delivery
            .claim(cmd.tenant_id, Uuid::new_v4(), 1, 120, &profile)
            .await?
            .pop()
            .unwrap();
        assert_eq!(run_claim.wire, result.encode_json()?);
        run_delivery
            .settle(
                &run_claim,
                AdmissionDeliveryAction::Delivered {
                    receipt: AdmissionTransportReceipt::new(
                        profile.clone(),
                        "ACK_FIXTURE".to_owned(),
                        Uuid::new_v4(),
                        43_i64.to_be_bytes().to_vec(),
                        &run_claim.wire,
                    )?,
                },
            )
            .await?;
    }
    assert_eq!(schedule_delivery.reconcile(cmd.tenant_id, 8).await?, 0);
    assert_eq!(run_delivery.reconcile(cmd.tenant_id, 8).await?, 0);
    assert!(schedule_delivery
        .claim(cmd.tenant_id, Uuid::new_v4(), 1, 120, &profile)
        .await?
        .is_empty());
    assert!(run_delivery
        .claim(cmd.tenant_id, Uuid::new_v4(), 1, 120, &profile)
        .await?
        .is_empty());
    // Disposable metadata fixtures exercise native cleanup without inventing source messages.
    for (schema, delivery, kind) in [
        ("automation", &schedule_delivery, "request"),
        ("multica", &run_delivery, "result"),
    ] {
        let tenant = Uuid::new_v4();
        let expired = Uuid::new_v4();
        let exhausted = Uuid::new_v4();
        sqlx::query(&format!("INSERT INTO {schema}.admission_message_delivery(tenant_id,message_id,request_id,message_kind,created_at,expires_at) VALUES($1,$2,$2,$3,statement_timestamp()-interval '8 days',statement_timestamp()-interval '1 day')"))
            .bind(tenant).bind(expired).bind(kind).execute(admin).await?;
        sqlx::query(&format!("INSERT INTO {schema}.admission_message_delivery(tenant_id,message_id,request_id,message_kind,state,attempt_count,lease_owner_id,lease_expires_at) VALUES($1,$2,$2,$3,'leased',12,$4,clock_timestamp()-interval '1 second')"))
            .bind(tenant).bind(exhausted).bind(kind).bind(Uuid::new_v4()).execute(admin).await?;
        assert!(delivery
            .claim(tenant, Uuid::new_v4(), 8, 120, &profile)
            .await?
            .is_empty());
        let reasons: Vec<String> = sqlx::query_scalar(&format!("SELECT terminal_reason FROM {schema}.admission_message_delivery_receipt WHERE tenant_id=$1 ORDER BY terminal_reason"))
            .bind(tenant).fetch_all(admin).await?;
        assert_eq!(reasons, vec!["attempts_exhausted", "retention_elapsed"]);
        let remaining: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM {schema}.admission_message_delivery WHERE tenant_id=$1"
        ))
        .bind(tenant)
        .fetch_one(admin)
        .await?;
        assert_eq!(remaining, 0);
        // Even the disposable admin cannot mutate a terminal receipt through normal SQL.
        assert!(sqlx::query(&format!(
            "DELETE FROM {schema}.admission_message_delivery_receipt WHERE tenant_id=$1"
        ))
        .bind(tenant)
        .execute(admin)
        .await
        .is_err());
        let capped = Uuid::new_v4();
        sqlx::query(&format!("INSERT INTO {schema}.admission_message_delivery(tenant_id,message_id,request_id,message_kind) SELECT $1,gen_random_uuid(),gen_random_uuid(),$2 FROM generate_series(1,1000)"))
            .bind(capped).bind(kind).execute(admin).await?;
        assert!(delivery.reconcile(capped, 8).await.is_err());
    }
    run_pool.close().await;
    schedule_pool.close().await;
    Ok(())
}

async fn verify_schedule_fence_compensation(
    admin: &sqlx::PgPool,
    store: &PostgresScheduleAdmissionStore,
    previous: &ScheduleRunAdmissionCommandV1,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = previous.clone();
    cmd.request_id = Uuid::new_v4();
    cmd.occurrence.occurrence_id = Uuid::new_v4();
    cmd.occurrence.key.scheduled_for_utc =
        chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    let schedule = star_pg_adapter::repository::PgAutomationScheduleRepository::new(admin.clone());
    let fence = seed_schedule_handoff(admin, &schedule, &cmd).await?;
    let run_service = Uuid::new_v4();
    let wire = schedule
        .persist_admission_request(&fence, run_service, &cmd)
        .await?;
    let ScheduleAdmissionStart::New(pending) = store.begin(source(cmd.tenant_id), &wire).await?
    else {
        panic!("new compensation preparation")
    };
    let result = pending.prepare_and_commit().await?;
    assert_eq!(
        result.outcome,
        ScheduleRunAdmissionOutcomeV1::Accepted,
        "{result:?}"
    );
    // Only the clock-expiry source is an admin fixture; reclaim is the actual adapter path.
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE automation.occurrence_dispatch SET lease_expires_at=clock_timestamp()-interval '1 second' WHERE tenant_id=$1 AND occurrence_id=$2")
        .bind(cmd.tenant_id).bind(cmd.occurrence.occurrence_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    let replacement = schedule
        .claim_due_occurrences(cmd.tenant_id, Uuid::new_v4(), 60, 1)
        .await?;
    assert_eq!(
        replacement[0].fencing_generation,
        fence.fencing_generation + 1
    );
    let resolution = schedule
        .resolve_admission_result(cmd.tenant_id, run_service, &result.encode_json()?)
        .await?
        .unwrap();
    assert_eq!(
        resolution.decision,
        ScheduleAdmissionResolutionKindV1::Compensate
    );
    assert_eq!(
        resolution.compensation_reason,
        Some(ScheduleAdmissionCompensationReasonV1::ScheduleFenceLost)
    );
    if let Ok(url) = std::env::var("STAR_ADMISSION_DELIVERY_TEST_NATS_URL") {
        use platform_messaging_runtime::admission_delivery::{
            AdmissionDeliveryOwner, AdmissionDeliveryStore, AdmissionJetStreamPublisher,
            AdmissionTransportProfile,
        };
        let profile = AdmissionTransportProfile::new(
            "nats-jetstream".to_owned(),
            "2.12.14".to_owned(),
            [9; 32],
            "test-nats".to_owned(),
        )?;
        let context = async_nats::jetstream::new(async_nats::connect(url).await?);
        let name = format!("RESOLUTION_{}", Uuid::new_v4().simple());
        let stream = context
            .create_stream(async_nats::jetstream::stream::Config {
                name: name.clone(),
                subjects: vec![format!("star.automation.admission.v1.{}.>", cmd.tenant_id)],
                max_bytes: 2 * 1024 * 1024,
                max_messages: 100,
                max_message_size: 16384,
                ..Default::default()
            })
            .await?;
        // This pool is the disposable test admin; the adapter still pins the Schedule role.
        // Restricted login/role membership is independently checked by verify_admission_delivery.
        let delivery = AdmissionDeliveryStore::new(admin.clone(), AdmissionDeliveryOwner::Schedule);
        let publisher = AdmissionJetStreamPublisher::new(
            delivery,
            context.clone(),
            name.clone(),
            String::new(),
            Uuid::new_v4(),
            profile,
        )?;
        let (_cancel, active) = tokio::sync::watch::channel(false);
        assert_eq!(
            publisher
                .publish_due(cmd.tenant_id, Uuid::new_v4(), 8, &active)
                .await?
                .delivered,
            2
        );
        assert_eq!(stream.get_info().await?.state.messages, 2);
        let reason: String = sqlx::query_scalar("SELECT terminal_reason FROM automation.admission_message_delivery_receipt WHERE tenant_id=$1 AND message_kind='resolution' AND message_id=$2")
            .bind(cmd.tenant_id).bind(resolution.resolution_id).fetch_one(admin).await?;
        assert_eq!(reason, "durable_acceptance");
        context.delete_stream(name).await?;
    }
    store
        .compensate_admission(source(cmd.tenant_id), &resolution.encode_json()?)
        .await?;
    let linked: Option<Uuid> = sqlx::query_scalar("SELECT admitted_run_id FROM automation.occurrence_dispatch WHERE tenant_id=$1 AND occurrence_id=$2")
        .bind(cmd.tenant_id).bind(cmd.occurrence.occurrence_id).fetch_one(admin).await?;
    assert_eq!(linked, None);
    Ok(())
}

async fn verify_schedule_compensation(
    admin: &sqlx::PgPool,
    store: &PostgresScheduleAdmissionStore,
    cmd: &ScheduleRunAdmissionCommandV1,
    result: &ScheduleRunAdmissionResultV1,
) -> Result<(), Box<dyn std::error::Error>> {
    let resolution = ScheduleRunAdmissionResolutionV1 {
        schema_version: 1,
        tenant_id: result.tenant_id,
        resolution_id: Uuid::new_v4(),
        request_id: result.request_id,
        admission_attempt: result.admission_attempt,
        result_id: result.result_id,
        correlation_id: result.correlation_id,
        occurrence_id: result.occurrence_id,
        task_execution_run_id: result.task_execution_run_id.unwrap(),
        binding_digest: result.binding_digest,
        decision: ScheduleAdmissionResolutionKindV1::Compensate,
        compensation_reason: Some(ScheduleAdmissionCompensationReasonV1::ScheduleFenceLost),
    };
    let wire = resolution.encode_json()?;
    let demand = domain_infrastructure::HostInfrastructureDemandV2 {
        schema_version: 2,
        event_id: Uuid::new_v4(),
        correlation_id: cmd.correlation_id,
        tenant_id: cmd.tenant_id,
        project_id: cmd.occurrence.rule_snapshot.target.project_id,
        branch_id: cmd.occurrence.rule_snapshot.target.branch_id,
        engineering_run_id: cmd.occurrence.rule_snapshot.target.engineering_run_id,
        task_execution_run_id: resolution.task_execution_run_id,
        schedule_rule_id: cmd.occurrence.rule_snapshot.rule_id.as_uuid(),
        schedule_rule_version: i64::try_from(cmd.occurrence.rule_snapshot.rule_version)?,
        occurrence_id: cmd.occurrence.occurrence_id,
        binding: cmd.infrastructure_binding.clone(),
        resource_budget: cmd.infrastructure_binding.resource_budget,
        requested_at: Utc::now(),
        deadline_at: Utc::now() + chrono::Duration::seconds(60),
    };
    let capacity_store = run_owner_runtime::PostgresRunOwnerCapacityStore::new(admin.clone());
    let mut tx = admin.begin().await?;
    let error = capacity_store
        .append_capacity_demand(
            &mut tx,
            cmd.occurrence.rule_snapshot.target.work_item_id,
            &demand,
        )
        .await
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared Schedule demand requires confirmation"));
    tx.rollback().await?;
    let mut wrong_source = source(cmd.tenant_id);
    wrong_source.schedule_service_id = Uuid::new_v4();
    assert!(matches!(
        store.compensate_admission(wrong_source, &wire).await,
        Err(ScheduleAdmissionStoreError::UntrustedScope)
    ));
    assert!(store
        .compensate_admission(source(Uuid::new_v4()), &wire)
        .await
        .is_err());
    let mut mismatch = resolution.clone();
    mismatch.binding_digest[0] ^= 1;
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &mismatch.encode_json()?)
        .await
        .is_err());
    let mut confirm = resolution.clone();
    confirm.decision = ScheduleAdmissionResolutionKindV1::Confirm;
    confirm.compensation_reason = None;
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &confirm.encode_json()?)
        .await
        .is_err());
    // Canonical state fixture only: compensation must refuse an active reservation.
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_reservation SET reservation_state='active',activated_at=clock_timestamp() WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(resolution.task_execution_run_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &wire)
        .await
        .is_err());
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("UPDATE multica.project_execution_resource_reservation SET reservation_state='pending',activated_at=NULL WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(resolution.task_execution_run_id).execute(&mut *fixture).await?;
    let confirmed = Uuid::new_v4();
    sqlx::query("INSERT INTO multica.task_execution_run_event(event_id,tenant_id,project_id,run_id,work_item_id,event_type,actor_id,correlation_id,details) VALUES($1,$2,$3,$4,$5,'schedule_admission_confirmed',$6,$7,'{}')")
        .bind(confirmed).bind(cmd.tenant_id).bind(cmd.occurrence.rule_snapshot.target.project_id)
        .bind(resolution.task_execution_run_id).bind(cmd.occurrence.rule_snapshot.target.work_item_id)
        .bind(cmd.occurrence.run_as_actor_id).bind(cmd.correlation_id).execute(&mut *fixture).await?;
    fixture.commit().await?;
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &wire)
        .await
        .is_err());
    let mut fixture = admin.begin().await?;
    sqlx::query("SET LOCAL session_replication_role=replica")
        .execute(&mut *fixture)
        .await?;
    sqlx::query("DELETE FROM multica.task_execution_run_event WHERE event_id=$1")
        .bind(confirmed)
        .execute(&mut *fixture)
        .await?;
    fixture.commit().await?;
    // Fail at the final receipt after staged release/audit writes: all owner facts must roll back.
    sqlx::raw_sql("CREATE FUNCTION multica.reject_d10_receipt() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'd10 receipt failure'; END $$; CREATE TRIGGER d10_fail BEFORE INSERT ON multica.schedule_compensation_inbox FOR EACH ROW EXECUTE FUNCTION multica.reject_d10_receipt()").execute(admin).await?;
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &wire)
        .await
        .is_err());
    sqlx::raw_sql("DROP TRIGGER d10_fail ON multica.schedule_compensation_inbox; DROP FUNCTION multica.reject_d10_receipt()").execute(admin).await?;
    let state:String=sqlx::query_scalar("SELECT reservation_state FROM multica.project_execution_resource_reservation WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(resolution.task_execution_run_id).fetch_one(admin).await?;
    assert_eq!(state, "pending");
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run_event WHERE tenant_id=$1 AND run_id=$2 AND event_type IN ('resource_reservation_released','schedule_admission_compensated')")
        .bind(cmd.tenant_id).bind(resolution.task_execution_run_id).fetch_one(admin).await?;
    assert_eq!(count, 0);
    assert!(
        store
            .compensate_admission(source(cmd.tenant_id), &wire)
            .await?
    );
    assert!(
        !store
            .compensate_admission(source(cmd.tenant_id), &wire)
            .await?
    );
    let mut conflict = resolution.clone();
    conflict.compensation_reason = Some(ScheduleAdmissionCompensationReasonV1::OccurrenceCancelled);
    assert!(store
        .compensate_admission(source(cmd.tenant_id), &conflict.encode_json()?)
        .await
        .is_err());
    let state:String=sqlx::query_scalar("SELECT reservation_state FROM multica.project_execution_resource_reservation WHERE tenant_id=$1 AND run_id=$2")
        .bind(cmd.tenant_id).bind(resolution.task_execution_run_id).fetch_one(admin).await?;
    assert_eq!(state, "released");
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.task_execution_run_event WHERE tenant_id=$1 AND run_id=$2 AND event_type IN ('resource_reservation_released','schedule_admission_compensated')")
        .bind(cmd.tenant_id).bind(resolution.task_execution_run_id).fetch_one(admin).await?;
    assert_eq!(count, 2);
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM multica.schedule_compensation_inbox WHERE tenant_id=$1 AND request_id=$2")
        .bind(cmd.tenant_id).bind(cmd.request_id).fetch_one(admin).await?;
    assert_eq!(count, 1);
    let mut tx = admin.begin().await?;
    let error = capacity_store
        .append_capacity_demand(
            &mut tx,
            cmd.occurrence.rule_snapshot.target.work_item_id,
            &demand,
        )
        .await
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared Schedule demand requires confirmation"));
    tx.rollback().await?;
    let mut tx = admin.begin().await?;
    sqlx::query("SET LOCAL ROLE star_multica_run_owner")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(Uuid::new_v4().to_string())
        .execute(&mut *tx)
        .await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM multica.schedule_compensation_inbox")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(count, 0);
    tx.rollback().await?;
    let ScheduleAdmissionStart::Replay(replay) = store
        .begin(source(cmd.tenant_id), &cmd.encode_json()?)
        .await?
    else {
        panic!("historical accepted replay")
    };
    assert_eq!(replay, *result); // Historical accepted fact cannot reactivate the released reservation.
    Ok(())
}
