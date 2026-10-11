/*
@cypher schema=1 source_sha256=4d19e82b6ce897605a59b5166b2e40db02d9ff62e636f0786ada74439fdac6e5
MERGE (self:File {path:"crates/star-pg-adapter/tests/schedule_postgres.rs"})
MERGE (fixture:Type {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::Fixture"})
MERGE (connect:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::connect_pools",kind:"function"})
MERGE (seed_rule:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::seed_rule",kind:"function"})
MERGE (seed_occurrence:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::seed_pending_occurrence",kind:"function"})
MERGE (test_materialize:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::materialization_is_idempotent_and_tenant_scoped",kind:"test"})
MERGE (test_claim:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::concurrent_claims_and_fencing_are_database_authoritative",kind:"test"})
MERGE (test_retry:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::retry_terminal_ttl_and_expired_lease_reclaim_are_durable",kind:"test"})
MERGE (test_run_as:Symbol {id:"crates/star-pg-adapter/tests/schedule_postgres.rs::schedule_rule_run_as_actor_cannot_change_between_revisions",kind:"test"})
MERGE (adapter:ExternalService {id:"star-pg-adapter.PgAutomationScheduleRepository",kind:"type"})
MERGE (sqlx:ExternalService {id:"sqlx.PgPool",kind:"type"})
MERGE (occurrence:Table {id:"automation.occurrence"})
MERGE (occurrence_run_as_actor:Column {id:"automation.occurrence.run_as_actor_id"})
MERGE (dispatch:Table {id:"automation.occurrence_dispatch"})
MERGE (event:Table {id:"automation.occurrence_event"})
MERGE (schedule_rule:Table {id:"automation.schedule_rule_revision"})
MERGE (self)-[:DEFINES]->(fixture)
MERGE (self)-[:DEFINES]->(connect)
MERGE (self)-[:DEFINES]->(seed_rule)
MERGE (self)-[:DEFINES]->(seed_occurrence)
MERGE (self)-[:DEFINES]->(test_materialize)
MERGE (self)-[:DEFINES]->(test_claim)
MERGE (self)-[:DEFINES]->(test_retry)
MERGE (self)-[:DEFINES]->(test_run_as)
MERGE (self)-[:DEFINES]->(occurrence_run_as_actor)
MERGE (connect)-[:CALLS]->(sqlx)
MERGE (seed_rule)-[:WRITES]->(occurrence)
MERGE (seed_occurrence)-[:WRITES]->(dispatch)
MERGE (test_materialize)-[:CALLS]->(adapter)
MERGE (test_materialize)-[:TESTS]->(occurrence)
MERGE (test_materialize)-[:TESTS]->(event)
MERGE (test_claim)-[:CALLS]->(adapter)
MERGE (test_claim)-[:TESTS]->(dispatch)
MERGE (test_retry)-[:CALLS]->(adapter)
MERGE (test_retry)-[:TESTS]->(dispatch)
MERGE (test_retry)-[:TESTS]->(event)
MERGE (test_run_as)-[:TESTS]->(schedule_rule)
MERGE (test_run_as)-[:TESTS]->(occurrence)
MERGE (test_run_as)-[:TESTS]->(occurrence_run_as_actor)
@endcypher
*/

//! Disposable-PostgreSQL integration tests for Schedule materialization and dispatch.

use chrono::{Duration, Utc};
use domain_automation::schedule::{
    SCHEDULE_PARSER_VERSION, SCHEDULE_TZDB_VERSION, materialize_schedule_window,
};
use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use star_pg_adapter::repository::{
    OccurrenceTerminalOutcome, PgAutomationScheduleRepository, ScheduleDispatchResult,
    ScheduleRepositoryError,
};
use uuid::Uuid;

#[derive(Clone)]
struct Fixture {
    tenant_id: Uuid,
    project_id: Uuid,
    rule_id: Uuid,
    branch_id: Uuid,
    engineering_run_id: Uuid,
    repository_id: Uuid,
    worktree_id: Uuid,
    work_item_id: Uuid,
    execution_profile_id: Uuid,
    hook_set_digest: String,
    execution_profile_digest: String,
    actor_id: Uuid,
}

async fn connect_pools() -> (PgPool, PgPool) {
    let admin_url = std::env::var("STAR_SCHEDULE_ADMIN_DATABASE_URL")
        .expect("the isolated-PG runner supplies an admin URL");
    let runtime_url = std::env::var("STAR_SCHEDULE_TEST_DATABASE_URL")
        .expect("the isolated-PG runner supplies a non-superuser RLS URL");
    let admin = PgPoolOptions::new()
        .max_connections(4)
        .connect(&admin_url)
        .await
        .expect("admin pool connects to the disposable cluster");
    let runtime = PgPoolOptions::new()
        .max_connections(8)
        .connect(&runtime_url)
        .await
        .expect("runtime pool connects to the disposable cluster");
    (admin, runtime)
}

async fn seed_rule(admin: &PgPool, max_attempts: i16) -> Fixture {
    let fixture = Fixture {
        tenant_id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        rule_id: Uuid::new_v4(),
        branch_id: Uuid::new_v4(),
        engineering_run_id: Uuid::new_v4(),
        repository_id: Uuid::new_v4(),
        worktree_id: Uuid::new_v4(),
        work_item_id: Uuid::new_v4(),
        execution_profile_id: Uuid::new_v4(),
        hook_set_digest: "b".repeat(64),
        execution_profile_digest: "a".repeat(64),
        actor_id: Uuid::new_v4(),
    };
    sqlx::query(
        "INSERT INTO automation.schedule_rule_revision \
            (tenant_id, project_id, rule_id, rule_version, enabled, cron_expression, time_zone, \
             parser_version, tzdb_version, dst_gap_policy, dst_fold_policy, overlap_policy, \
             overlap_max_active, misfire_policy, misfire_max_occurrences, pause_policy, \
             retry_max_attempts, retry_initial_backoff_seconds, retry_max_backoff_seconds, deadline_seconds, \
             branch_id, engineering_run_id, repository_id, worktree_id, work_item_id, \
             execution_profile_id, execution_profile_version, execution_profile_digest, \
             hook_set_version, hook_set_digest, run_as_actor_id, changed_by) \
         VALUES ($1,$2,$3,1,true,'0 * * * * *','UTC',$4,$5,'skip','earlier_instant', \
                 'queue_one',1,'skip',1,'skip_elapsed',$6,1,1,600,$7,$8,$9,$10,$11,$12,1,$13,1,$14,$15,$15)",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.rule_id)
    .bind(SCHEDULE_PARSER_VERSION)
    .bind(SCHEDULE_TZDB_VERSION)
    .bind(max_attempts)
    .bind(fixture.branch_id)
    .bind(fixture.engineering_run_id)
    .bind(fixture.repository_id)
    .bind(fixture.worktree_id)
    .bind(fixture.work_item_id)
    .bind(fixture.execution_profile_id)
    .bind(&fixture.execution_profile_digest)
    .bind(&fixture.hook_set_digest)
    .bind(fixture.actor_id)
    .execute(admin)
    .await
    .expect("seed a valid current Rule revision");
    fixture
}

async fn seed_pending_occurrence(
    admin: &PgPool,
    fixture: &Fixture,
    retention_seconds: i64,
) -> Uuid {
    seed_pending_occurrence_with_deadline(admin, fixture, retention_seconds, 600).await
}

async fn seed_pending_occurrence_with_deadline(
    admin: &PgPool,
    fixture: &Fixture,
    retention_seconds: i64,
    deadline_seconds: i64,
) -> Uuid {
    let occurrence_id = Uuid::new_v4();
    let scheduled_for = Utc::now() - Duration::seconds(5);
    sqlx::query(
        "INSERT INTO automation.occurrence \
            (occurrence_id, tenant_id, project_id, rule_id, rule_version, run_as_actor_id, scheduled_for_utc, \
             scheduled_local_label, utc_offset_seconds, parser_version, tzdb_version, target_snapshot, \
             target_snapshot_digest, materialized_at) \
         VALUES ($1,$2,$3,$4,1,$5,$6,'2026-10-02T12:00:00',0,$7,$8,$9,$10,clock_timestamp())",
    )
    .bind(occurrence_id)
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.rule_id)
    .bind(fixture.actor_id)
    .bind(scheduled_for)
    .bind(SCHEDULE_PARSER_VERSION)
    .bind(SCHEDULE_TZDB_VERSION)
    .bind(json!({ "fixture": true }))
    .bind("c".repeat(64))
    .execute(admin)
    .await
    .expect("seed immutable occurrence");
    sqlx::query(
        "INSERT INTO automation.occurrence_dispatch \
            (tenant_id, occurrence_id, dispatch_state, next_attempt_at, occurrence_deadline_at, retention_period) \
         VALUES ($1,$2,'pending',clock_timestamp() - INTERVAL '1 second', \
                 clock_timestamp() + ($4 * INTERVAL '1 second'), $3 * INTERVAL '1 second')",
    )
    .bind(fixture.tenant_id)
    .bind(occurrence_id)
    .bind(retention_seconds)
    .bind(deadline_seconds)
    .execute(admin)
    .await
    .expect("seed due dispatch Work row");
    occurrence_id
}

#[tokio::test]
#[ignore = "requires the phase9f3_schedule.py disposable PostgreSQL runner"]
async fn schedule_rule_run_as_actor_cannot_change_between_revisions() {
    let (admin, runtime) = connect_pools().await;
    let fixture = seed_rule(&admin, 3).await;
    let original_creator = sqlx::query_scalar::<_, Uuid>(
        "SELECT changed_by FROM automation.schedule_rule_revision \
         WHERE tenant_id=$1 AND rule_id=$2 AND valid_to IS NULL",
    )
    .bind(fixture.tenant_id)
    .bind(fixture.rule_id)
    .fetch_one(&admin)
    .await
    .expect("the seed rule retains its original creator");
    let replacement_actor = Uuid::new_v4();
    assert_ne!(replacement_actor, original_creator);
    let mismatched_occurrence_error = sqlx::query(
        "INSERT INTO automation.occurrence \
            (occurrence_id, tenant_id, project_id, rule_id, rule_version, run_as_actor_id, \
             scheduled_for_utc, scheduled_local_label, utc_offset_seconds, parser_version, \
             tzdb_version, target_snapshot, target_snapshot_digest, materialized_at) \
         VALUES ($1,$2,$3,$4,1,$5,now(),'2026-10-04T12:00:00',0,$6,$7,$8,$9,now())",
    )
    .bind(Uuid::new_v4())
    .bind(fixture.tenant_id)
    .bind(fixture.project_id)
    .bind(fixture.rule_id)
    .bind(replacement_actor)
    .bind(SCHEDULE_PARSER_VERSION)
    .bind(SCHEDULE_TZDB_VERSION)
    .bind(json!({ "fixture": true }))
    .bind("d".repeat(64))
    .execute(&admin)
    .await
    .expect_err("an occurrence must retain the pinned Rule revision's run-as principal");
    assert!(
        mismatched_occurrence_error
            .to_string()
            .contains("occurrence run-as identity differs from pinned Rule revision")
    );

    let initial_rule_id = Uuid::new_v4();
    let initial_error = sqlx::query(
        "INSERT INTO automation.schedule_rule_revision \
         SELECT (jsonb_populate_record(NULL::automation.schedule_rule_revision, \
           to_jsonb(rule) || jsonb_build_object(\
             'rule_id', $2, \
             'run_as_actor_id', $3, \
             'rule_version', 1, \
             'valid_from', now(), \
             'valid_to', NULL\
           ))).* \
         FROM automation.schedule_rule_revision rule \
         WHERE rule.tenant_id = $1 AND rule.rule_id = $4 AND rule.valid_to IS NULL",
    )
    .bind(fixture.tenant_id)
    .bind(initial_rule_id)
    .bind(replacement_actor)
    .bind(fixture.rule_id)
    .execute(&admin)
    .await
    .expect_err("an initial run-as principal must be the authenticated creator");
    assert!(
        initial_error
            .to_string()
            .contains("initial automation schedule run-as identity must match its creator")
    );

    let error = sqlx::query(
        "INSERT INTO automation.schedule_rule_revision \
         SELECT (jsonb_populate_record(NULL::automation.schedule_rule_revision, \
           to_jsonb(rule) || jsonb_build_object(\
             'rule_version', 2, \
             'run_as_actor_id', $2, \
             'valid_from', now() - interval '2 seconds', \
             'valid_to', now() - interval '1 second'\
           ))).* \
         FROM automation.schedule_rule_revision rule \
         WHERE rule.tenant_id = $1 AND rule.rule_id = $3 AND rule.valid_to IS NULL",
    )
    .bind(fixture.tenant_id)
    .bind(replacement_actor)
    .bind(fixture.rule_id)
    .execute(&admin)
    .await
    .expect_err("a successor revision cannot replace the original run-as principal");
    assert!(error.to_string().contains("run-as identity is immutable"));
    admin.close().await;
    runtime.close().await;
}

#[tokio::test]
#[ignore = "requires the phase9f3_schedule.py disposable PostgreSQL runner"]
async fn materialization_is_idempotent_and_tenant_scoped() {
    let (admin, runtime) = connect_pools().await;
    let repository = PgAutomationScheduleRepository::new(runtime.clone());
    let fixture = seed_rule(&admin, 3).await;
    let other_tenant = seed_rule(&admin, 3).await;

    let rule = repository
        .load_current_rule(
            fixture.tenant_id,
            domain_automation::RuleId::from_uuid(fixture.rule_id),
        )
        .await
        .expect("runtime can read its current Rule under RLS");
    let now = Utc::now();
    let batch = materialize_schedule_window(
        &rule,
        now - Duration::minutes(2),
        now + Duration::minutes(2),
        now - Duration::seconds(5),
        now,
        32,
    )
    .expect("bounded UTC recurrence window");
    assert!(!batch.occurrences.is_empty());
    assert!(!batch.has_more);

    let inserted = repository
        .persist_materialization_batch(fixture.tenant_id, &batch)
        .await
        .expect("persist snapshots, dispatch rows, and events in one transaction");
    assert_eq!(inserted.len(), batch.occurrences.len());
    let replay = repository
        .persist_materialization_batch(fixture.tenant_id, &batch)
        .await
        .expect("unique tenant/rule/revision/UTC-slot key makes replay safe");
    assert!(replay.is_empty());

    let _other_occurrence = seed_pending_occurrence(&admin, &other_tenant, 3600).await;
    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM automation.occurrence")
        .fetch_one(&mut *runtime.acquire().await.expect("runtime connection"))
        .await
        .expect("unset tenant scope is filtered by FORCE RLS");
    assert_eq!(visible, 0);
    let mut scoped = runtime.begin().await.expect("runtime transaction");
    sqlx::query("SELECT set_config('app.tenant_id',$1,true)")
        .bind(fixture.tenant_id.to_string())
        .execute(&mut *scoped)
        .await
        .expect("set current tenant");
    let persisted_run_as: Uuid = sqlx::query_scalar(
        "SELECT run_as_actor_id FROM automation.occurrence \
         WHERE tenant_id=$1 AND occurrence_id=$2",
    )
    .bind(fixture.tenant_id)
    .bind(inserted[0])
    .fetch_one(&mut *scoped)
    .await
    .expect("the occurrence persists its immutable run-as snapshot");
    assert_eq!(persisted_run_as, rule.run_as_actor_id);
    let own_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM automation.occurrence")
        .fetch_one(&mut *scoped)
        .await
        .expect("tenant scope query");
    assert_eq!(own_rows, inserted.len() as i64);
    scoped.rollback().await.expect("release scoped transaction");

    let event_id: Uuid = sqlx::query_scalar(
        "SELECT event_id FROM automation.occurrence_event WHERE tenant_id = $1 LIMIT 1",
    )
    .bind(fixture.tenant_id)
    .fetch_one(&admin)
    .await
    .expect("materializer emitted an append-only event");
    let rewrite = sqlx::query(
        "UPDATE automation.occurrence_event SET details = '{}'::jsonb WHERE event_id = $1",
    )
    .bind(event_id)
    .execute(&admin)
    .await;
    assert!(
        rewrite.is_err(),
        "append-only event UPDATE must be rejected"
    );
    admin.close().await;
    runtime.close().await;
}

#[tokio::test]
#[ignore = "requires the phase9f3_schedule.py disposable PostgreSQL runner"]
async fn concurrent_claims_and_fencing_are_database_authoritative() {
    let (admin, runtime) = connect_pools().await;
    let fixture = seed_rule(&admin, 3).await;
    let first_id = seed_pending_occurrence(&admin, &fixture, 3600).await;
    let second_id = seed_pending_occurrence(&admin, &fixture, 3600).await;
    let repository = PgAutomationScheduleRepository::new(runtime.clone());
    let worker_a = Uuid::new_v4();
    let worker_b = Uuid::new_v4();
    let (claimed_a, claimed_b) = tokio::join!(
        repository.claim_due_occurrences(fixture.tenant_id, worker_a, 30, 1),
        repository.claim_due_occurrences(fixture.tenant_id, worker_b, 30, 1)
    );
    let claimed_a = claimed_a.expect("first claim");
    let claimed_b = claimed_b.expect("second claim");
    assert_eq!(claimed_a.len(), 1);
    assert_eq!(claimed_b.len(), 1);
    assert_ne!(claimed_a[0].occurrence_id, claimed_b[0].occurrence_id);
    assert!([first_id, second_id].contains(&claimed_a[0].occurrence_id));
    assert!([first_id, second_id].contains(&claimed_b[0].occurrence_id));
    assert_eq!(claimed_a[0].fencing_generation, 1);
    assert_eq!(claimed_b[0].fencing_generation, 1);

    let renewed = repository
        .heartbeat(fixture.tenant_id, &claimed_a[0], 60)
        .await
        .expect("current lease can be extended");
    assert_eq!(renewed.fencing_generation, claimed_a[0].fencing_generation);
    assert!(renewed.lease_expires_at >= claimed_a[0].lease_expires_at);
    let mut wrong_generation = claimed_a[0].clone();
    wrong_generation.fencing_generation += 1;
    assert!(matches!(
        repository
            .heartbeat(fixture.tenant_id, &wrong_generation, 30)
            .await,
        Err(ScheduleRepositoryError::StaleFence)
    ));

    repository
        .finish(
            fixture.tenant_id,
            &claimed_b[0],
            OccurrenceTerminalOutcome::Succeeded,
        )
        .await
        .expect("terminal completion is fenced and durable");
    let state: String = sqlx::query_scalar(
        "SELECT dispatch_state FROM automation.occurrence_dispatch WHERE tenant_id = $1 AND occurrence_id = $2",
    )
    .bind(fixture.tenant_id)
    .bind(claimed_b[0].occurrence_id)
    .fetch_one(&admin)
    .await
    .expect("admin reads terminal state");
    assert_eq!(state, "succeeded");
    admin.close().await;
    runtime.close().await;
}

#[tokio::test]
#[ignore = "requires the phase9f3_schedule.py disposable PostgreSQL runner"]
async fn retry_terminal_ttl_and_expired_lease_reclaim_are_durable() {
    let (admin, runtime) = connect_pools().await;
    let fixture = seed_rule(&admin, 2).await;
    let occurrence_id = seed_pending_occurrence(&admin, &fixture, 1).await;
    let repository = PgAutomationScheduleRepository::new(runtime.clone());

    let first = repository
        .claim_due_occurrences(fixture.tenant_id, Uuid::new_v4(), 30, 1)
        .await
        .expect("first attempt")
        .into_iter()
        .next()
        .expect("one occurrence claimed");
    let retry = repository
        .record_failure(fixture.tenant_id, &first, "provider_unavailable")
        .await
        .expect("failure schedules bounded retry");
    let ScheduleDispatchResult::RetryScheduled { next_attempt_at } = retry else {
        panic!("first failed attempt must enter retry_wait");
    };
    assert!(
        repository
            .claim_due_occurrences(fixture.tenant_id, Uuid::new_v4(), 30, 1)
            .await
            .expect("early retry poll")
            .is_empty()
    );
    let remaining = (next_attempt_at - Utc::now()).to_std().unwrap_or_default();
    tokio::time::sleep(remaining + std::time::Duration::from_millis(150)).await;

    let second = repository
        .claim_due_occurrences(fixture.tenant_id, Uuid::new_v4(), 30, 1)
        .await
        .expect("retry attempt")
        .into_iter()
        .next()
        .expect("retry becomes claimable");
    assert_eq!(second.occurrence_id, occurrence_id);
    assert_eq!(second.fencing_generation, first.fencing_generation + 1);
    assert_eq!(
        repository
            .record_failure(fixture.tenant_id, &second, "provider_unavailable")
            .await
            .expect("attempt budget exhaustion is terminal"),
        ScheduleDispatchResult::Failed
    );
    assert_eq!(
        repository
            .purge_expired_dispatch_rows(fixture.tenant_id, 10)
            .await
            .expect("terminal TTL check"),
        0
    );
    tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
    assert_eq!(
        repository
            .purge_expired_dispatch_rows(fixture.tenant_id, 10)
            .await
            .expect("expired terminal Work row can be purged"),
        1
    );
    let immutable_fact: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM automation.occurrence WHERE tenant_id = $1 AND occurrence_id = $2",
    )
    .bind(fixture.tenant_id)
    .bind(occurrence_id)
    .fetch_one(&admin)
    .await
    .expect("transaction occurrence remains after Work TTL");
    assert_eq!(immutable_fact, 1);

    let reclaim_fixture = seed_rule(&admin, 3).await;
    let reclaim_id = seed_pending_occurrence(&admin, &reclaim_fixture, 3600).await;
    let abandoned = repository
        .claim_due_occurrences(reclaim_fixture.tenant_id, Uuid::new_v4(), 1, 1)
        .await
        .expect("short lease claim")
        .into_iter()
        .next()
        .expect("lease acquired");
    tokio::time::sleep(std::time::Duration::from_millis(1_150)).await;
    let reclaimed = repository
        .claim_due_occurrences(reclaim_fixture.tenant_id, Uuid::new_v4(), 30, 1)
        .await
        .expect("expired lease can be reclaimed")
        .into_iter()
        .next()
        .expect("reclaimed lease");
    assert_eq!(reclaimed.occurrence_id, reclaim_id);
    assert_eq!(
        reclaimed.fencing_generation,
        abandoned.fencing_generation + 1
    );
    let (expired_attempt, expired_generation): (i16, i64) = sqlx::query_as(
        "SELECT attempt_no, fencing_generation FROM automation.occurrence_event \
         WHERE tenant_id = $1 AND occurrence_id = $2 AND event_type = 'lease_expired'",
    )
    .bind(reclaim_fixture.tenant_id)
    .bind(reclaim_id)
    .fetch_one(&admin)
    .await
    .expect("expired lease event records the abandoned attempt and fence");
    assert_eq!((expired_attempt, expired_generation), (1, 1));
    assert!(matches!(
        repository
            .heartbeat(reclaim_fixture.tenant_id, &abandoned, 30)
            .await,
        Err(ScheduleRepositoryError::StaleFence)
    ));
    admin.close().await;
    runtime.close().await;
}

#[tokio::test]
#[ignore = "requires the phase9f3_schedule.py disposable PostgreSQL runner"]
async fn expired_deadlines_and_exhausted_leases_are_terminalized() {
    let (admin, runtime) = connect_pools().await;
    let repository = PgAutomationScheduleRepository::new(runtime.clone());

    let deadline_fixture = seed_rule(&admin, 3).await;
    let deadline_occurrence =
        seed_pending_occurrence_with_deadline(&admin, &deadline_fixture, 60, 1).await;
    tokio::time::sleep(std::time::Duration::from_millis(1_150)).await;
    assert!(
        repository
            .claim_due_occurrences(deadline_fixture.tenant_id, Uuid::new_v4(), 30, 10)
            .await
            .expect("deadline sweep succeeds")
            .is_empty()
    );
    let (deadline_state, deadline_code): (String, String) = sqlx::query_as(
        "SELECT dispatch_state, last_failure_code FROM automation.occurrence_dispatch \
         WHERE tenant_id = $1 AND occurrence_id = $2",
    )
    .bind(deadline_fixture.tenant_id)
    .bind(deadline_occurrence)
    .fetch_one(&admin)
    .await
    .expect("deadline work row remains as terminal Work");
    assert_eq!(deadline_state, "failed");
    assert_eq!(deadline_code, "deadline_expired");
    let deadline_event: String = sqlx::query_scalar(
        "SELECT event_type FROM automation.occurrence_event WHERE tenant_id = $1 \
         AND occurrence_id = $2 ORDER BY occurred_at DESC LIMIT 1",
    )
    .bind(deadline_fixture.tenant_id)
    .bind(deadline_occurrence)
    .fetch_one(&admin)
    .await
    .expect("deadline produces an immutable event");
    assert_eq!(deadline_event, "deadline_expired");

    let exhausted_fixture = seed_rule(&admin, 1).await;
    let exhausted_occurrence = seed_pending_occurrence(&admin, &exhausted_fixture, 60).await;
    let abandoned = repository
        .claim_due_occurrences(exhausted_fixture.tenant_id, Uuid::new_v4(), 1, 1)
        .await
        .expect("last permitted attempt can be claimed")
        .into_iter()
        .next()
        .expect("one attempt was claimed");
    tokio::time::sleep(std::time::Duration::from_millis(1_150)).await;
    assert!(
        repository
            .claim_due_occurrences(exhausted_fixture.tenant_id, Uuid::new_v4(), 30, 10)
            .await
            .expect("exhausted lease sweep succeeds")
            .is_empty()
    );
    let (exhausted_state, exhausted_code): (String, String) = sqlx::query_as(
        "SELECT dispatch_state, last_failure_code FROM automation.occurrence_dispatch \
         WHERE tenant_id = $1 AND occurrence_id = $2",
    )
    .bind(exhausted_fixture.tenant_id)
    .bind(exhausted_occurrence)
    .fetch_one(&admin)
    .await
    .expect("exhausted lease row is terminalized");
    assert_eq!(exhausted_state, "failed");
    assert_eq!(exhausted_code, "attempts_exhausted");
    assert_eq!(abandoned.fencing_generation, 1);
    let exhausted_event: String = sqlx::query_scalar(
        "SELECT event_type FROM automation.occurrence_event WHERE tenant_id = $1 \
         AND occurrence_id = $2 ORDER BY occurred_at DESC LIMIT 1",
    )
    .bind(exhausted_fixture.tenant_id)
    .bind(exhausted_occurrence)
    .fetch_one(&admin)
    .await
    .expect("exhausted lease produces an immutable event");
    assert_eq!(exhausted_event, "dispatch_failed");
    admin.close().await;
    runtime.close().await;
}
