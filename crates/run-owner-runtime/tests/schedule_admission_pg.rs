//! Real PostgreSQL owner-local admission receipt/replay/rollback verification.
/*
@cypher schema=1 source_sha256=b1012909c9dbc8f8b289e1abb956cfb6a55020f36c68ccbebbb552e1bfc71b5f
MERGE (self:File {path:"crates/run-owner-runtime/tests/schedule_admission_pg.rs"})
MERGE (test:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::admission_receipt_replay_conflict_rollback_and_scope",kind:"test"})
MERGE (fixture:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::command_fixture",kind:"function"})
MERGE (binding:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::binding_fixture",kind:"function"})
MERGE (source:Symbol {id:"crates/run-owner-runtime/tests/schedule_admission_pg.rs::source",kind:"function"})
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
      VALUES('{worktree}','{tenant}','{workspace}','{repo}','{project}','fixture','dev','fixture://{worktree}');
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
