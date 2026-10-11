-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/tests/task-run-admission-attempt-lifecycle.sql"})
-- MERGE (run:Table {id:"multica.task_execution_run"})
-- MERGE (event:Table {id:"multica.task_execution_run_event"})
-- MERGE (self)-[:TESTS]->(run)
-- MERGE (self)-[:TESTS]->(event)
-- @endcypher

BEGIN;

DO $attempt_lifecycle$
DECLARE
    v_tenant UUID := gen_random_uuid();
    v_project UUID := gen_random_uuid();
    v_work_item UUID := gen_random_uuid();
    v_actor UUID := gen_random_uuid();
    v_rule UUID := gen_random_uuid();
    v_occurrence UUID := gen_random_uuid();
    v_correlation UUID := gen_random_uuid();
    v_run_one UUID := gen_random_uuid();
    v_run_two UUID := gen_random_uuid();
BEGIN
    INSERT INTO multica.task_execution_run (
        run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
        task_snapshot, run_origin, automation_rule_id, automation_rule_version,
        automation_occurrence_id, automation_admission_attempt, correlation_id, started_at
    ) VALUES (
        v_run_one, v_tenant, v_project, v_work_item, v_actor, 'agent', '{}'::jsonb,
        'schedule', v_rule, 1, v_occurrence, 1, v_correlation, NULL
    );
    INSERT INTO multica.task_execution_run (
        run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
        task_snapshot, run_origin, automation_rule_id, automation_rule_version,
        automation_occurrence_id, automation_admission_attempt, correlation_id, started_at
    ) VALUES (
        v_run_two, v_tenant, v_project, v_work_item, v_actor, 'agent', '{}'::jsonb,
        'schedule', v_rule, 1, v_occurrence, 2, gen_random_uuid(), NULL
    );

    IF (SELECT count(*) FROM multica.task_execution_run
        WHERE tenant_id = v_tenant AND automation_occurrence_id = v_occurrence) <> 2
       OR EXISTS (
           SELECT 1 FROM multica.task_execution_run
            WHERE tenant_id = v_tenant AND automation_occurrence_id = v_occurrence
              AND (started_at IS NOT NULL OR created_at IS NULL)
       ) THEN
        RAISE EXCEPTION 'distinct provisional logical attempts must be recorded without claiming execution started';
    END IF;

    BEGIN
        INSERT INTO multica.task_execution_run (
            run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
            task_snapshot, run_origin, automation_rule_id, automation_rule_version,
            automation_occurrence_id, correlation_id, started_at
        ) VALUES (
            gen_random_uuid(), v_tenant, v_project, v_work_item, v_actor, 'agent', '{}'::jsonb,
            'schedule', v_rule, 1, gen_random_uuid(), gen_random_uuid(), NULL
        );
        RAISE EXCEPTION 'provisional Schedule Run without an attempt ordinal was accepted';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;

    BEGIN
        INSERT INTO multica.task_execution_run (
            run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
            task_snapshot, run_origin, correlation_id, started_at
        ) VALUES (
            gen_random_uuid(), v_tenant, v_project, v_work_item, v_actor, 'manual',
            '{}'::jsonb, 'manual', gen_random_uuid(), NULL
        );
        RAISE EXCEPTION 'non-Schedule Run without an actual start time was accepted';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;

    BEGIN
        INSERT INTO multica.task_execution_run (
            run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
            task_snapshot, run_origin, automation_rule_id, automation_rule_version,
            automation_occurrence_id, automation_admission_attempt, correlation_id, started_at
        ) VALUES (
            gen_random_uuid(), v_tenant, v_project, v_work_item, v_actor, 'agent', '{}'::jsonb,
            'schedule', v_rule, 1, v_occurrence, 1, gen_random_uuid(), NULL
        );
        RAISE EXCEPTION 'duplicate logical admission attempt was accepted';
    EXCEPTION WHEN unique_violation THEN
        NULL;
    END;

    BEGIN
        INSERT INTO multica.task_execution_run (
            run_id, tenant_id, project_id, work_item_id, initiated_by, execution_channel,
            task_snapshot, run_origin, automation_rule_id, automation_rule_version,
            automation_occurrence_id, correlation_id, started_at
        ) VALUES (
            gen_random_uuid(), v_tenant, v_project, v_work_item, v_actor, 'agent', '{}'::jsonb,
            'schedule', v_rule, 1, v_occurrence, gen_random_uuid(), clock_timestamp()
        );
        RAISE EXCEPTION 'legacy ordinal-less attempt did not collide with logical attempt one';
    EXCEPTION WHEN unique_violation THEN
        NULL;
    END;

    INSERT INTO multica.task_execution_run_event (
        event_id, tenant_id, project_id, run_id, work_item_id, event_type, correlation_id
    ) VALUES
        (gen_random_uuid(), v_tenant, v_project, v_run_one, v_work_item, 'schedule_admission_confirmed', v_correlation),
        (gen_random_uuid(), v_tenant, v_project, v_run_two, v_work_item, 'schedule_admission_compensated', v_correlation);
END
$attempt_lifecycle$;

ROLLBACK;
