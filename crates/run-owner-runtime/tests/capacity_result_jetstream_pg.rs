//! Disposable PostgreSQL + NATS JetStream verification for the Run-owner result Inbox.
/*
@cypher schema=1 source_sha256=eb6d31ebc499220b2ca1d3d1279143122b0aef39401711e77c92f2ceeae8b590
MERGE (self:File {path:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs"})
MERGE (test:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::capacity_result_persist_replay_conflict_ack_and_rollback",kind:"test"})
MERGE (seed:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::seed_demand",kind:"function"})
MERGE (fixture:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::demand_fixture",kind:"function"})
MERGE (envelope:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::rejected_envelope",kind:"function"})
MERGE (create_durable:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::create_durable",kind:"function"})
MERGE (create_durable_ack:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::create_durable_with_ack_wait",kind:"function"})
MERGE (publish:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::publish",kind:"function"})
MERGE (result_count:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::result_count",kind:"function"})
MERGE (result_event_count:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::result_count_for_event",kind:"function"})
MERGE (install_fault:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::install_result_insert_fault",kind:"function"})
MERGE (remove_fault:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::remove_result_insert_fault",kind:"function"})
MERGE (consumer:Type {id:"run_owner_runtime::RunOwnerCapacityResultConsumer",kind:"external_type"})
MERGE (store:Type {id:"run_owner_runtime::PostgresRunOwnerCapacityStore",kind:"external_type"})
MERGE (postgres:ExternalService {id:"PostgreSQL disposable test instance"})
MERGE (nats:ExternalService {id:"NATS JetStream disposable test instance"})
MERGE (inbox:Table {id:"multica.infrastructure_capacity_result_inbox"})
MERGE (outbox:Table {id:"multica.infrastructure_capacity_demand_outbox"})
MERGE (delete_stream:ExternalService {id:"async_nats::jetstream::Context.delete_stream",kind:"method"})
MERGE (poison_test:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::poison_quarantine_failure_redelivery_commit_ack_loss_and_term",kind:"test"})
MERGE (install_poison_fault:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::install_quarantine_receipt_fault",kind:"function"})
MERGE (remove_poison_fault:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::remove_quarantine_receipt_fault",kind:"function"})
MERGE (poison_receipt_count:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::quarantine_receipt_count",kind:"function"})
MERGE (poison_payload_count:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::quarantine_payload_count",kind:"function"})
MERGE (no_redelivery:Symbol {id:"crates/run-owner-runtime/tests/capacity_result_jetstream_pg.rs::assert_no_redelivery",kind:"function"})
MERGE (quarantine_connect:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.connect",kind:"method"})
MERGE (record_message:Symbol {id:"crates/platform-messaging-runtime/src/lib.rs::PlatformPoisonQuarantine.record_message",kind:"method"})
MERGE (with_quarantine:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.with_quarantine",kind:"method"})
MERGE (consume_one:Symbol {id:"crates/run-owner-runtime/src/capacity_result.rs::RunOwnerCapacityResultConsumer.consume_one",kind:"method"})
MERGE (platform_quarantine:Type {id:"platform_messaging_runtime::PlatformPoisonQuarantine",kind:"external_type"})
MERGE (trusted_source:Type {id:"platform_messaging_runtime::TrustedJetStreamSource",kind:"external_type"})
MERGE (retention_policy:Type {id:"platform_messaging_runtime::PayloadRetentionPolicy",kind:"external_type"})
MERGE (poison_fault:Type {id:"platform_messaging_runtime::PermanentPoisonFault",kind:"external_type"})
MERGE (poison_outcome:Type {id:"platform_messaging_runtime::QuarantineRecordOutcome",kind:"external_type"})
MERGE (quarantine_receipt:Table {id:"platform_messaging.poison_quarantine_receipt"})
MERGE (quarantine_payload:Table {id:"platform_messaging.poison_quarantine_payload_work"})
MERGE (poison_consume_outcome:Type {id:"run_owner_runtime::CapacityResultConsumeOutcome",kind:"enum"})
MERGE (self)-[:DEFINES]->(test)
MERGE (self)-[:DEFINES]->(seed)
MERGE (self)-[:DEFINES]->(fixture)
MERGE (self)-[:DEFINES]->(envelope)
MERGE (self)-[:DEFINES]->(create_durable)
MERGE (self)-[:DEFINES]->(create_durable_ack)
MERGE (self)-[:DEFINES]->(publish)
MERGE (self)-[:DEFINES]->(result_count)
MERGE (self)-[:DEFINES]->(result_event_count)
MERGE (self)-[:DEFINES]->(install_fault)
MERGE (self)-[:DEFINES]->(remove_fault)
MERGE (test)-[:CALLS]->(seed)
MERGE (test)-[:CALLS]->(envelope)
MERGE (test)-[:CALLS]->(create_durable)
MERGE (test)-[:CALLS]->(create_durable_ack)
MERGE (test)-[:CALLS]->(publish)
MERGE (test)-[:CALLS]->(result_count)
MERGE (test)-[:CALLS]->(result_event_count)
MERGE (test)-[:CALLS]->(install_fault)
MERGE (test)-[:CALLS]->(remove_fault)
MERGE (seed)-[:CALLS]->(fixture)
MERGE (seed)-[:CALLS]->(append:Symbol {id:"crates/run-owner-runtime/src/outbox.rs::PostgresRunOwnerCapacityStore.append_capacity_demand",kind:"method"})
MERGE (create_durable)-[:CALLS]->(create_durable_ack)
MERGE (test)-[:USES_TYPE]->(consumer)
MERGE (test)-[:USES_TYPE]->(store)
MERGE (test)-[:TESTS]->(inbox)
MERGE (test)-[:TESTS]->(outbox)
MERGE (test)-[:USES_TYPE]->(postgres)
MERGE (test)-[:USES_TYPE]->(nats)
MERGE (test)-[:CALLS]->(delete_stream)
MERGE (self)-[:DEFINES]->(poison_test)
MERGE (self)-[:DEFINES]->(install_poison_fault)
MERGE (self)-[:DEFINES]->(remove_poison_fault)
MERGE (self)-[:DEFINES]->(poison_receipt_count)
MERGE (self)-[:DEFINES]->(poison_payload_count)
MERGE (self)-[:DEFINES]->(no_redelivery)
MERGE (poison_test)-[:CALLS]->(create_durable_ack)
MERGE (poison_test)-[:CALLS]->(publish)
MERGE (poison_test)-[:CALLS]->(quarantine_connect)
MERGE (poison_test)-[:CALLS]->(record_message)
MERGE (poison_test)-[:CALLS]->(with_quarantine)
MERGE (poison_test)-[:CALLS]->(consume_one)
MERGE (poison_test)-[:CALLS]->(install_poison_fault)
MERGE (poison_test)-[:CALLS]->(remove_poison_fault)
MERGE (poison_test)-[:CALLS]->(poison_receipt_count)
MERGE (poison_test)-[:CALLS]->(poison_payload_count)
MERGE (poison_test)-[:CALLS]->(no_redelivery)
MERGE (poison_test)-[:USES_TYPE]->(platform_quarantine)
MERGE (poison_test)-[:USES_TYPE]->(trusted_source)
MERGE (poison_test)-[:USES_TYPE]->(retention_policy)
MERGE (poison_test)-[:USES_TYPE]->(poison_fault)
MERGE (poison_test)-[:USES_TYPE]->(poison_outcome)
MERGE (poison_test)-[:USES_TYPE]->(poison_consume_outcome)
MERGE (poison_test)-[:TESTS]->(quarantine_receipt)
MERGE (poison_test)-[:TESTS]->(quarantine_payload)
MERGE (poison_test)-[:CALLS]->(delete_stream)
@endcypher
*/

use std::time::Duration;

use async_nats::jetstream::{
    consumer::{self, AckPolicy, DeliverPolicy, PullConsumer},
    message::PublishMessage,
    stream, Context,
};
use chrono::{Duration as ChronoDuration, Utc};
use domain_infrastructure::{
    ClusterPlacementPolicyV1, ExecutionResourceBudgetV1, HostInfrastructureDemandV2,
    KubernetesDnsLabel, ProviderKind, RunInfrastructureBindingSnapshotV2,
};
use futures_util::StreamExt;
use platform_messaging_runtime::{
    PayloadRetentionPolicy, PermanentPoisonFault, PlatformPoisonQuarantine,
    QuarantineRecordOutcome, TrustedJetStreamSource,
};
use run_owner_runtime::{
    CapacityDemandAppendOutcome, CapacityResultConsumeOutcome, CapacityResultError,
    PostgresRunOwnerCapacityStore, RunOwnerCapacityResultConsumer, RunOwnerCapacityStoreError,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::watch;
use uuid::Uuid;

const DATABASE_URL_ENV: &str = "STAR_RUN_OWNER_TEST_DATABASE_URL";
const NATS_URL_ENV: &str = "STAR_RUN_OWNER_TEST_NATS_URL";
const SUBJECT: &str = "star.infrastructure.v2.capacity_rejected";

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL + NATS runner: scripts/automation/phase9f4d_run_owner_consumer.py"]
async fn capacity_result_persist_replay_conflict_ack_and_rollback(
) -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var(DATABASE_URL_ENV)?;
    let nats_url = std::env::var(NATS_URL_ENV)?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await?;
    let store = PostgresRunOwnerCapacityStore::new(pool.clone());
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    let mut created_streams = Vec::new();

    let demand_one = seed_demand(&pool, &store, Uuid::new_v4()).await?;
    let demand_two = seed_demand(&pool, &store, Uuid::new_v4()).await?;
    let event_one = Uuid::new_v4();
    let event_two = Uuid::new_v4();
    let wire_one = rejected_envelope(&demand_one, event_one, "host_capacity_insufficient")?;
    let wire_two = rejected_envelope(&demand_two, event_two, "cluster_capacity_insufficient")?;

    // A pre-cancelled consumer must not issue a pull or persist/ack a message.
    let (stream_name, durable) = create_durable(&jetstream).await?;
    created_streams.push(stream_name.clone());
    publish(&jetstream, SUBJECT, event_one, &wire_one).await?;
    publish(&jetstream, SUBJECT, event_two, &wire_two).await?;
    let mut consumer = RunOwnerCapacityResultConsumer::new(store.clone(), durable.clone())?;
    let (_cancel_tx, mut cancelled) = watch::channel(true);
    assert_eq!(
        consumer.consume_one(&mut cancelled).await?,
        CapacityResultConsumeOutcome::Cancelled
    );
    assert_eq!(result_count(&pool, demand_one.tenant_id).await?, 0);
    assert_eq!(durable.get_info().await?.num_ack_pending, 0);

    // One call stores only one event even though both are available in the stream.
    let (_resume_tx, mut resume) = watch::channel(false);
    assert_eq!(
        consumer.consume_one(&mut resume).await?,
        CapacityResultConsumeOutcome::Stored {
            event_id: event_one
        }
    );
    assert_eq!(result_count(&pool, demand_one.tenant_id).await?, 1);
    assert_eq!(durable.get_info().await?.num_pending, 1);
    assert_eq!(
        consumer.consume_one(&mut resume).await?,
        CapacityResultConsumeOutcome::Stored {
            event_id: event_two
        }
    );
    assert_eq!(result_count(&pool, demand_two.tenant_id).await?, 1);
    jetstream.delete_stream(&stream_name).await?;
    created_streams.clear();

    // A new stream bypasses JetStream's duplicate window so the durable DB Inbox proves replay.
    let (replay_stream, replay_durable) = create_durable(&jetstream).await?;
    created_streams.push(replay_stream.clone());
    publish(&jetstream, SUBJECT, event_one, &wire_one).await?;
    let mut replay_consumer = RunOwnerCapacityResultConsumer::new(store.clone(), replay_durable)?;
    assert_eq!(
        replay_consumer.consume_one(&mut resume).await?,
        CapacityResultConsumeOutcome::Duplicate {
            event_id: event_one
        }
    );
    assert_eq!(result_count(&pool, demand_one.tenant_id).await?, 1);
    jetstream.delete_stream(&replay_stream).await?;
    created_streams.clear();

    // Same event_id with altered payload must hard-conflict and stay unacknowledged.
    let (conflict_stream, conflict_durable) = create_durable(&jetstream).await?;
    created_streams.push(conflict_stream.clone());
    let conflicting_wire = rejected_envelope(&demand_one, event_one, "provider_profile_not_found")?;
    publish(&jetstream, SUBJECT, event_one, &conflicting_wire).await?;
    let mut conflict_consumer =
        RunOwnerCapacityResultConsumer::new(store.clone(), conflict_durable.clone())?;
    assert!(matches!(
        conflict_consumer.consume_one(&mut resume).await,
        Err(CapacityResultError::IdempotencyConflict)
    ));
    assert_eq!(conflict_durable.get_info().await?.num_ack_pending, 1);
    jetstream.delete_stream(&conflict_stream).await?;
    created_streams.clear();

    // Force a database statement error for one event. The transaction rolls back and JetStream
    // remains pending; after removing the temporary fault, redelivery stores then acknowledges.
    let demand_three = seed_demand(&pool, &store, Uuid::new_v4()).await?;
    let event_three = Uuid::new_v4();
    let wire_three = rejected_envelope(&demand_three, event_three, "provider_profile_not_found")?;
    let (rollback_stream, rollback_durable) =
        create_durable_with_ack_wait(&jetstream, Duration::from_millis(300)).await?;
    created_streams.push(rollback_stream);
    let (trigger_name, function_name) = install_result_insert_fault(&pool, event_three).await?;
    publish(&jetstream, SUBJECT, event_three, &wire_three).await?;
    let mut rollback_consumer =
        RunOwnerCapacityResultConsumer::new(store.clone(), rollback_durable.clone())?;
    assert!(matches!(
        rollback_consumer.consume_one(&mut resume).await,
        Err(CapacityResultError::Database(_))
    ));
    assert_eq!(result_count_for_event(&pool, event_three).await?, 0);
    assert_eq!(rollback_durable.get_info().await?.num_ack_pending, 1);
    remove_result_insert_fault(&pool, &trigger_name, &function_name).await?;
    tokio::time::sleep(Duration::from_millis(350)).await;
    assert_eq!(
        rollback_consumer.consume_one(&mut resume).await?,
        CapacityResultConsumeOutcome::Stored {
            event_id: event_three
        }
    );
    assert_eq!(result_count_for_event(&pool, event_three).await?, 1);

    for stream_name in created_streams {
        jetstream.delete_stream(&stream_name).await?;
    }
    pool.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL + NATS runner: scripts/automation/phase9f4d_run_owner_consumer.py"]
async fn poison_quarantine_failure_redelivery_commit_ack_loss_and_term(
) -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var(DATABASE_URL_ENV)?;
    let nats_url = std::env::var(NATS_URL_ENV)?;
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await?;
    let store = PostgresRunOwnerCapacityStore::new(pool.clone());
    let client = async_nats::connect(nats_url).await?;
    let jetstream = async_nats::jetstream::new(client);
    let (_resume_tx, mut resume) = watch::channel(false);
    let deployment_id = Uuid::new_v4();
    let generation = Uuid::new_v4();
    let malformed = b"{invalid-json";
    let fault = PermanentPoisonFault::MalformedCapacityResultJson;

    // A durable receipt insert failure must leave the source delivery pending. After the
    // configured AckWait, the real JetStream redelivery must commit before the confirmed Term ACK.
    let (stream_name, durable) =
        create_durable_with_ack_wait(&jetstream, Duration::from_millis(300)).await?;
    let event_id = Uuid::new_v4();
    publish(&jetstream, SUBJECT, event_id, malformed).await?;
    let source = TrustedJetStreamSource::new(deployment_id, generation, stream_name.clone())?;
    let quarantine = PlatformPoisonQuarantine::connect(
        pool.clone(),
        source,
        PayloadRetentionPolicy::MetadataOnly,
        None,
    )
    .await?;
    let mut consumer = RunOwnerCapacityResultConsumer::new(store.clone(), durable.clone())?
        .with_quarantine(quarantine);
    let (trigger, function) = install_quarantine_receipt_fault(&pool).await?;
    assert!(matches!(
        consumer.consume_one(&mut resume).await,
        Err(CapacityResultError::Quarantine(_))
    ));
    assert_eq!(durable.get_info().await?.num_ack_pending, 1);
    assert_eq!(
        quarantine_receipt_count(&pool, deployment_id, generation, &stream_name).await?,
        0
    );
    remove_quarantine_receipt_fault(&pool, &trigger, &function).await?;
    tokio::time::sleep(Duration::from_millis(350)).await;

    match consumer.consume_one(&mut resume).await? {
        CapacityResultConsumeOutcome::Quarantined {
            duplicate: false, ..
        } => {}
        other => return Err(format!("expected first quarantine commit, got {other:?}").into()),
    }
    assert_eq!(
        quarantine_receipt_count(&pool, deployment_id, generation, &stream_name).await?,
        1
    );
    assert_eq!(
        quarantine_payload_count(&pool, deployment_id, generation, &stream_name).await?,
        0
    );
    assert_eq!(durable.get_info().await?.num_ack_pending, 0);
    assert_no_redelivery(&durable).await?;
    jetstream.delete_stream(&stream_name).await?;

    // Model consumer loss after the immutable receipt commit but before ACK by dropping both
    // delivery handles. The source message is redelivered; exact replay must return Duplicate and
    // still terminate redelivery. No OS process is killed or restarted here.
    let (replay_stream, replay_durable) =
        create_durable_with_ack_wait(&jetstream, Duration::from_millis(300)).await?;
    let replay_event_id = Uuid::new_v4();
    publish(&jetstream, SUBJECT, replay_event_id, malformed).await?;
    let replay_source =
        TrustedJetStreamSource::new(deployment_id, generation, replay_stream.clone())?;
    let replay_quarantine = PlatformPoisonQuarantine::connect(
        pool.clone(),
        replay_source,
        PayloadRetentionPolicy::MetadataOnly,
        None,
    )
    .await?;
    let mut first_delivery = replay_durable
        .fetch()
        .max_messages(1)
        .expires(Duration::from_secs(2))
        .messages()
        .await?;
    let message = tokio::time::timeout(Duration::from_secs(3), first_delivery.next())
        .await?
        .ok_or("expected published poison message")?
        .map_err(|error| error.to_string())?;
    let first_sequence = i64::try_from(
        message
            .info()
            .map_err(|error| error.to_string())?
            .stream_sequence,
    )?;
    assert_eq!(
        replay_quarantine.record_message(&message, fault, 1).await?,
        QuarantineRecordOutcome::Stored {
            stream_sequence: first_sequence,
            body_retained: false,
        }
    );
    drop(message);
    drop(first_delivery);
    tokio::time::sleep(Duration::from_millis(350)).await;

    let mut replay_consumer = RunOwnerCapacityResultConsumer::new(store, replay_durable.clone())?
        .with_quarantine(replay_quarantine);
    assert_eq!(
        replay_consumer.consume_one(&mut resume).await?,
        CapacityResultConsumeOutcome::Quarantined {
            stream_sequence: first_sequence,
            duplicate: true,
        }
    );
    assert_eq!(
        quarantine_receipt_count(&pool, deployment_id, generation, &replay_stream).await?,
        1
    );
    assert_eq!(replay_durable.get_info().await?.num_ack_pending, 0);
    assert_no_redelivery(&replay_durable).await?;
    jetstream.delete_stream(&replay_stream).await?;

    pool.close().await;
    Ok(())
}

async fn seed_demand(
    pool: &PgPool,
    store: &PostgresRunOwnerCapacityStore,
    work_item_id: Uuid,
) -> Result<HostInfrastructureDemandV2, Box<dyn std::error::Error>> {
    let demand = demand_fixture(work_item_id);
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"INSERT INTO multica.task_execution_run
               (run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
                task_snapshot, run_origin, automation_rule_id, automation_rule_version,
                automation_occurrence_id, correlation_id)
           VALUES ($1,$2,$3,$4,$5,'langgraph','{}'::jsonb,'schedule',$6,1,$7,$8)"#,
    )
    .bind(demand.task_execution_run_id)
    .bind(demand.tenant_id)
    .bind(demand.project_id)
    .bind(work_item_id)
    .bind(Uuid::new_v4())
    .bind(demand.schedule_rule_id)
    .bind(demand.occurrence_id)
    .bind(demand.correlation_id)
    .execute(&mut *tx)
    .await?;
    let mut mismatched_occurrence = demand.clone();
    mismatched_occurrence.occurrence_id = Uuid::new_v4();
    assert!(matches!(
        store
            .append_capacity_demand(&mut tx, work_item_id, &mismatched_occurrence)
            .await,
        Err(RunOwnerCapacityStoreError::InvalidDemand)
    ));
    assert_eq!(
        store
            .append_capacity_demand(&mut tx, work_item_id, &demand)
            .await?,
        CapacityDemandAppendOutcome::Inserted {
            event_id: demand.event_id
        }
    );
    assert_eq!(
        store
            .append_capacity_demand(&mut tx, work_item_id, &demand)
            .await?,
        CapacityDemandAppendOutcome::Duplicate {
            event_id: demand.event_id
        }
    );
    tx.commit().await?;
    Ok(demand)
}

fn demand_fixture(_work_item_id: Uuid) -> HostInfrastructureDemandV2 {
    let id = Uuid::new_v4;
    let budget = ExecutionResourceBudgetV1 {
        max_rss_bytes: 64 * 1024 * 1024,
        max_cpu_ms: 1_000,
        max_runtime_ms: 30_000,
        max_child_processes: 1,
        max_parallel_tools: 1,
        max_provider_calls: 1,
        max_output_bytes: 4_096,
        max_event_buffer_bytes: 4_096,
    };
    let binding = RunInfrastructureBindingSnapshotV2 {
        schema_version: 2,
        tenant_id: id(),
        project_id: id(),
        branch_id: id(),
        engineering_run_id: id(),
        binding_id: id(),
        binding_revision: 1,
        host_environment_id: id(),
        k3s_cluster_id: id(),
        k3s_cluster_generation: 1,
        cluster_placement_policy: ClusterPlacementPolicyV1::PinnedCluster,
        provider: ProviderKind::MultipassK3sCluster,
        provider_profile_id: id(),
        provider_profile_revision: 1,
        provider_profile_digest: [7; 32],
        namespace: KubernetesDnsLabel::new("run-test").unwrap(),
        service_account: KubernetesDnsLabel::new("agent-test").unwrap(),
        runtime_profile_id: id(),
        runtime_profile_revision: 1,
        runtime_profile_digest: [8; 32],
        resource_budget: budget,
    };
    let requested_at = Utc::now();
    HostInfrastructureDemandV2 {
        schema_version: 2,
        event_id: id(),
        correlation_id: id(),
        tenant_id: binding.tenant_id,
        project_id: binding.project_id,
        branch_id: binding.branch_id,
        engineering_run_id: binding.engineering_run_id,
        task_execution_run_id: id(),
        schedule_rule_id: id(),
        schedule_rule_version: 1,
        occurrence_id: id(),
        binding,
        resource_budget: budget,
        requested_at,
        deadline_at: requested_at + ChronoDuration::minutes(5),
    }
}

fn rejected_envelope(
    demand: &HostInfrastructureDemandV2,
    result_event_id: Uuid,
    reason_code: &str,
) -> Result<Vec<u8>, serde_json::Error> {
    let binding_digest = demand.binding.digest().expect("valid binding");
    let binding_digest = binding_digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let payload = json!({
        "schema_version": 2,
        "source_event_id": demand.event_id,
        "occurrence_id": demand.occurrence_id,
        "task_execution_run_id": demand.task_execution_run_id,
        "binding_digest": binding_digest,
        "reason_code": reason_code,
    });
    serde_json::to_vec(&json!({
        "envelope_version": 1,
        "tenant_id": demand.tenant_id,
        "event_id": result_event_id,
        "event_type": "capacity_rejected",
        "schema_version": 2,
        "idempotency_key": format!("capacity_rejected:{}", demand.event_id),
        "correlation_id": demand.correlation_id,
        "payload": payload,
    }))
}

async fn create_durable(
    jetstream: &Context,
) -> Result<(String, PullConsumer), Box<dyn std::error::Error>> {
    create_durable_with_ack_wait(jetstream, Duration::from_secs(5)).await
}

async fn create_durable_with_ack_wait(
    jetstream: &Context,
    ack_wait: Duration,
) -> Result<(String, PullConsumer), Box<dyn std::error::Error>> {
    let suffix = Uuid::new_v4().simple().to_string();
    let stream_name = format!("RUN_CAP_{}", &suffix[..16]);
    let stream = jetstream
        .get_or_create_stream(stream::Config {
            name: stream_name.clone(),
            subjects: vec!["star.infrastructure.v2.*".to_owned()],
            max_age: Duration::from_secs(60),
            ..Default::default()
        })
        .await?;
    let durable_name = format!("run-owner-{}", &suffix[16..]);
    let consumer = stream
        .get_or_create_consumer(
            &durable_name,
            consumer::pull::Config {
                durable_name: Some(durable_name.clone()),
                filter_subject: "star.infrastructure.v2.*".to_owned(),
                deliver_policy: DeliverPolicy::All,
                ack_policy: AckPolicy::Explicit,
                ack_wait,
                max_deliver: 5,
                ..Default::default()
            },
        )
        .await?;
    Ok((stream_name, consumer))
}

async fn publish(
    jetstream: &Context,
    subject: &str,
    event_id: Uuid,
    wire: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let ack = jetstream
        .send_publish(
            subject.to_owned(),
            PublishMessage::build()
                .payload(wire.to_vec().into())
                .message_id(event_id.to_string()),
        )
        .await?;
    ack.await?;
    Ok(())
}

async fn result_count(pool: &PgPool, tenant_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM multica.infrastructure_capacity_result_inbox WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await
}

async fn result_count_for_event(pool: &PgPool, event_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM multica.infrastructure_capacity_result_inbox WHERE event_id = $1",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await
}

async fn install_quarantine_receipt_fault(pool: &PgPool) -> Result<(String, String), sqlx::Error> {
    let suffix = Uuid::new_v4().simple().to_string();
    let trigger_name = format!("zz_test_fail_quarantine_{suffix}");
    let function_name = format!("test_fail_quarantine_{suffix}");
    let function_sql = format!(
        "CREATE FUNCTION platform_messaging.{function_name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected quarantine receipt failure' USING ERRCODE = '40001'; END $$"
    );
    sqlx::query(&function_sql).execute(pool).await?;
    let trigger_sql = format!(
        "CREATE TRIGGER {trigger_name} BEFORE INSERT ON platform_messaging.poison_quarantine_receipt FOR EACH ROW EXECUTE FUNCTION platform_messaging.{function_name}()"
    );
    sqlx::query(&trigger_sql).execute(pool).await?;
    Ok((trigger_name, function_name))
}

async fn remove_quarantine_receipt_fault(
    pool: &PgPool,
    trigger_name: &str,
    function_name: &str,
) -> Result<(), sqlx::Error> {
    let trigger_sql = format!(
        "DROP TRIGGER IF EXISTS {trigger_name} ON platform_messaging.poison_quarantine_receipt"
    );
    sqlx::query(&trigger_sql).execute(pool).await?;
    let function_sql = format!("DROP FUNCTION IF EXISTS platform_messaging.{function_name}()");
    sqlx::query(&function_sql).execute(pool).await?;
    Ok(())
}

async fn quarantine_receipt_count(
    pool: &PgPool,
    deployment_id: Uuid,
    generation: Uuid,
    stream_name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"SELECT count(*) FROM platform_messaging.poison_quarantine_receipt
           WHERE deployment_id = $1 AND stream_generation = $2 AND stream_name = $3"#,
    )
    .bind(deployment_id)
    .bind(generation)
    .bind(stream_name)
    .fetch_one(pool)
    .await
}

async fn quarantine_payload_count(
    pool: &PgPool,
    deployment_id: Uuid,
    generation: Uuid,
    stream_name: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"SELECT count(*) FROM platform_messaging.poison_quarantine_payload_work
           WHERE deployment_id = $1 AND stream_generation = $2 AND stream_name = $3"#,
    )
    .bind(deployment_id)
    .bind(generation)
    .bind(stream_name)
    .fetch_one(pool)
    .await
}

async fn assert_no_redelivery(consumer: &PullConsumer) -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .expires(Duration::from_millis(250))
        .messages()
        .await?;
    let result = tokio::time::timeout(Duration::from_secs(1), messages.next()).await?;
    match result {
        None => Ok(()),
        Some(Ok(message)) => {
            let info = message.info().map_err(|error| error.to_string())?;
            Err(format!(
                "JetStream redelivered stream sequence {} after Term ACK",
                info.stream_sequence
            )
            .into())
        }
        Some(Err(error)) => Err(error.to_string().into()),
    }
}

async fn install_result_insert_fault(
    pool: &PgPool,
    event_id: Uuid,
) -> Result<(String, String), sqlx::Error> {
    let suffix = Uuid::new_v4().simple().to_string();
    let trigger_name = format!("zz_test_fail_result_{suffix}");
    let function_name = format!("test_fail_result_{suffix}");
    let function_sql = format!(
        "CREATE FUNCTION public.{function_name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'disposable integration fault'; END; $$"
    );
    sqlx::query(&function_sql).execute(pool).await?;
    let trigger_sql = format!(
        "CREATE TRIGGER {trigger_name} BEFORE INSERT ON multica.infrastructure_capacity_result_inbox FOR EACH ROW WHEN (NEW.event_id = '{event_id}'::uuid) EXECUTE FUNCTION public.{function_name}()"
    );
    sqlx::query(&trigger_sql).execute(pool).await?;
    Ok((trigger_name, function_name))
}

async fn remove_result_insert_fault(
    pool: &PgPool,
    trigger_name: &str,
    function_name: &str,
) -> Result<(), sqlx::Error> {
    let drop_trigger = format!(
        "DROP TRIGGER IF EXISTS {trigger_name} ON multica.infrastructure_capacity_result_inbox"
    );
    sqlx::query(&drop_trigger).execute(pool).await?;
    let drop_function = format!("DROP FUNCTION IF EXISTS public.{function_name}()");
    sqlx::query(&drop_function).execute(pool).await?;
    Ok(())
}
