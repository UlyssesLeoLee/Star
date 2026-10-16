-- @cypher schema=1 source_sha256=d4b36d7637d437003d7ad678a5681325bcf160162d757c4b27d755373eef6155
-- MERGE (self:File {path:"db/tests/bi-task-event-inbox.sql"})
-- MERGE (test:Symbol {id:"db/tests/bi-task-event-inbox.sql::bi_task_event_inbox_contract",kind:"test"})
-- MERGE (function:Database {id:"bi.commit_task_event_v2",kind:"stored_function"})
-- MERGE (inbox:Table {id:"bi.task_event_inbox",kind:"table"})
-- MERGE (fact:Table {id:"bi.task_event_fact",kind:"table"})
-- MERGE (incident:Table {id:"bi.task_event_integrity_incident",kind:"table"})
-- MERGE (self)-[:DEFINES]->(test)
-- MERGE (test)-[:TESTS]->(function)
-- MERGE (test)-[:READS]->(inbox)
-- MERGE (test)-[:READS]->(fact)
-- MERGE (test)-[:READS]->(incident)
-- @endcypher

-- Run only against the isolated disposable database created by the 9F4E runner.
SET ROLE star_bi_runtime;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';

DO $$
DECLARE
    tenant UUID := '11111111-1111-4111-8111-111111111111';
    project UUID := '22222222-2222-4222-8222-222222222222';
    run_id UUID := '33333333-3333-4333-8333-333333333333';
    event_uuid UUID := '44444444-4444-4444-8444-444444444444';
    work_item UUID := '55555555-5555-4555-8555-555555555555';
    actor UUID := '66666666-6666-4666-8666-666666666666';
    correlation UUID := '77777777-7777-4777-8777-777777777777';
    sender_service UUID := '88888888-8888-4888-8888-888888888888';
    receiver_service UUID := '99999999-9999-4999-8999-999999999999';
    profile UUID := 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
    destination_generation UUID := 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb';
    happened_at TIMESTAMPTZ := '2026-10-16T00:00:00Z';
    payload JSONB;
    first_receipt UUID;
    replay_receipt UUID;
    result_row RECORD;
BEGIN
    payload := jsonb_build_object(
        'schema_version', 3,
        'aggregate', jsonb_build_object('type', 'lifecycle', 'version', 1),
        'work_item_id', work_item::TEXT,
        'owner', jsonb_build_object(
            'tenant_id', tenant::TEXT,
            'project_id', project::TEXT,
            'engineering_run_id', run_id::TEXT
        ),
        'lifecycle', jsonb_build_object('status', 'in_progress', 'review_state', 'none'),
        'change', jsonb_build_object(
            'event_type', 'task.lifecycle.changed',
            'from_status', 'claimed',
            'to_status', 'in_progress',
            'private_note', 'BI must not retain this free-form value'
        ),
        'host_context', jsonb_build_object('branch_name', 'sensitive/freeform')
    );

    SELECT * INTO result_row FROM bi.commit_task_event_v2(
        tenant, project, run_id, event_uuid, work_item, 'lifecycle', 1,
        'task.lifecycle.changed', 3, actor, correlation, happened_at,
        payload, sender_service, receiver_service, 'star-rust-log', 'v1', profile, 1,
        decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
        destination_generation, decode(repeat('e5', 32), 'hex'), decode(repeat('a1', 32), 'hex')
    );
    IF result_row.outcome <> 'committed' OR result_row.receipt_id IS NULL THEN
        RAISE EXCEPTION 'first Task event commit did not return a durable receipt';
    END IF;
    first_receipt := result_row.receipt_id;

    BEGIN
        PERFORM * FROM bi.commit_task_event_v2(
            tenant, project, run_id, 'cccccccc-cccc-4ccc-8ccc-cccccccccccc', work_item,
            'lifecycle', 1, 'task.lifecycle.changed', 3, actor, correlation, happened_at,
            jsonb_set(payload, '{lifecycle,status}', 'null'::JSONB),
            sender_service, receiver_service, 'star-rust-log', 'v1', profile, 1,
            decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
            destination_generation, decode(repeat('e5', 32), 'hex'), decode(repeat('c4', 32), 'hex')
        );
        RAISE EXCEPTION 'BI projection accepted a missing lifecycle status';
    EXCEPTION WHEN SQLSTATE '23514' THEN
        NULL;
    END;

    SELECT * INTO result_row FROM bi.commit_task_event_v2(
        tenant, project, run_id, event_uuid, work_item, 'lifecycle', 1,
        'task.lifecycle.changed', 3, actor, correlation, happened_at,
        payload, sender_service, receiver_service, 'star-rust-log', 'v1', profile, 1,
        decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
        destination_generation, decode(repeat('e5', 32), 'hex'), decode(repeat('a1', 32), 'hex')
    );
    IF result_row.outcome <> 'exact_replay' OR result_row.receipt_id <> first_receipt THEN
        RAISE EXCEPTION 'exact Task event replay did not return its stable receipt';
    END IF;
    replay_receipt := result_row.receipt_id;

    SELECT * INTO result_row FROM bi.commit_task_event_v2(
        tenant, project, run_id, event_uuid, work_item, 'lifecycle', 1,
        'task.lifecycle.changed', 3, actor,
        '88888888-8888-4888-8888-888888888888',
        happened_at,
        payload, sender_service, receiver_service, 'star-rust-log', 'v1', profile, 1,
        decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
        destination_generation, decode(repeat('f6', 32), 'hex'), decode(repeat('b2', 32), 'hex')
    );
    IF result_row.outcome <> 'conflict' OR result_row.receipt_id IS NOT NULL THEN
        RAISE EXCEPTION 'conflicting Task event identity was not rejected';
    END IF;

    PERFORM set_config('app.tenant_id', '99999999-9999-4999-8999-999999999999', false);
    SELECT * INTO result_row FROM bi.commit_task_event_v2(
        '99999999-9999-4999-8999-999999999999',
        'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', run_id, event_uuid, work_item,
        'lifecycle', 1, 'task.lifecycle.changed', 3, actor, correlation,
        happened_at,
        jsonb_set(
            jsonb_set(payload, '{owner,tenant_id}', to_jsonb('99999999-9999-4999-8999-999999999999'::TEXT)),
            '{owner,project_id}', to_jsonb('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'::TEXT)
        ),
        sender_service, receiver_service, 'star-rust-log', 'v1', profile, 1,
        decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
        destination_generation, decode(repeat('77', 32), 'hex'), decode(repeat('c3', 32), 'hex')
    );
    IF result_row.outcome <> 'conflict' OR result_row.receipt_id IS NOT NULL THEN
        RAISE EXCEPTION 'cross-tenant event identity reuse was not rejected';
    END IF;
END;
$$;

RESET ROLE;
DO $$
BEGIN
    IF (SELECT count(*) FROM bi.task_event_inbox) <> 2 THEN
        RAISE EXCEPTION 'expected the migrated and new BI Inbox receipts';
    END IF;
    IF (SELECT count(*) FROM bi.task_event_fact) <> 2 THEN
        RAISE EXCEPTION 'expected the migrated and new BI immutable event facts';
    END IF;
    IF (SELECT count(*) FROM bi.task_event_integrity_incident) <> 2 THEN
        RAISE EXCEPTION 'expected two durable identity conflict incidents';
    END IF;
    IF has_table_privilege('star_bi_runtime', 'bi.task_event_inbox', 'SELECT')
       OR has_table_privilege('star_bi_runtime', 'bi.task_event_fact', 'INSERT')
       OR has_schema_privilege('star_bi_runtime', 'bi', 'CREATE') THEN
        RAISE EXCEPTION 'BI runtime role has direct table access';
    END IF;
    IF has_function_privilege('star_bi_runtime', 'bi.commit_task_event_v1(uuid,uuid,uuid,uuid,uuid,character varying,integer,character varying,integer,uuid,uuid,timestamp with time zone,jsonb,uuid,uuid,character varying,character varying,uuid,bigint,bytea,character varying,character varying,uuid,bytea,bytea)', 'EXECUTE')
       OR NOT has_function_privilege('star_bi_runtime', 'bi.commit_task_event_v2(uuid,uuid,uuid,uuid,uuid,character varying,integer,character varying,integer,uuid,uuid,timestamp with time zone,jsonb,uuid,uuid,character varying,character varying,uuid,bigint,bytea,character varying,character varying,uuid,bytea,bytea)', 'EXECUTE') THEN
        RAISE EXCEPTION 'BI runtime function boundary is incorrect';
    END IF;
END;
$$;

DO $$
BEGIN
    BEGIN
        UPDATE bi.task_event_fact SET event_type = 'task.updated';
        RAISE EXCEPTION 'WORM trigger allowed BI fact mutation';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        DELETE FROM bi.task_event_inbox;
        RAISE EXCEPTION 'WORM trigger allowed BI Inbox deletion';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        TRUNCATE bi.task_event_integrity_incident;
        RAISE EXCEPTION 'WORM trigger allowed BI incident truncation';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
END;
$$;
