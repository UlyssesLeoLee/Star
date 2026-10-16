-- @cypher schema=1 source_sha256=f5a96ded194306e63858bc6737d0caf4a1bb0c0637b44b1badd4f088bff50692
-- MERGE (self:File {path:"db/tests/bi-task-event-quarantine.sql"})
-- MERGE (test:Symbol {id:"db/tests/bi-task-event-quarantine.sql::bi_task_event_quarantine_contract",kind:"test"})
-- MERGE (writer:Database {id:"bi.record_task_event_quarantine_v1",kind:"stored_function"})
-- MERGE (table:Table {id:"bi.task_event_quarantine",kind:"table"})
-- MERGE (self)-[:DEFINES]->(test)
-- MERGE (test)-[:TESTS]->(writer)
-- MERGE (test)-[:READS]->(table)
-- @endcypher

-- Run only against the isolated disposable database created by the BI Task-event runner.
SET ROLE star_bi_runtime;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';

DO $$
DECLARE
    tenant_a UUID := '11111111-1111-4111-8111-111111111111';
    tenant_b UUID := '99999999-9999-4999-8999-999999999999';
    project_sender UUID := '88888888-8888-4888-8888-888888888888';
    bi_receiver UUID := '77777777-7777-4777-8777-777777777777';
    profile UUID := 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa';
    destination_generation UUID := 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb';
    log_generation UUID := 'cccccccc-cccc-4ccc-8ccc-cccccccccccc';
    event_uuid UUID := 'dddddddd-dddd-4ddd-8ddd-dddddddddddd';
    result_row RECORD;
    first_receipt UUID;
BEGIN
    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
    );
    IF result_row.outcome <> 'recorded' OR result_row.quarantine_id IS NULL THEN
        RAISE EXCEPTION 'first quarantine insert did not return a durable receipt';
    END IF;
    first_receipt := result_row.quarantine_id;

    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
    );
    IF result_row.outcome <> 'exact_replay' OR result_row.quarantine_id <> first_receipt THEN
        RAISE EXCEPTION 'exact quarantine replay did not return its stable receipt';
    END IF;

    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('b2', 32), 'hex'), 1024, 'malformed_envelope'
    );
    IF result_row.outcome <> 'conflict' OR result_row.quarantine_id IS NOT NULL THEN
        RAISE EXCEPTION 'changed quarantine digest did not fail closed';
    END IF;

    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('a1', 32), 'hex'), 1024, 'content_conflict'
    );
    IF result_row.outcome <> 'conflict' OR result_row.quarantine_id IS NOT NULL THEN
        RAISE EXCEPTION 'changed quarantine fault did not fail closed';
    END IF;

    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1,
        'eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee',
        decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
    );
    IF result_row.outcome <> 'conflict' OR result_row.quarantine_id IS NOT NULL THEN
        RAISE EXCEPTION 'changed quarantine message ID did not fail closed';
    END IF;

    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('a1', 32), 'hex'), 2048, 'malformed_envelope'
    );
    IF result_row.outcome <> 'conflict' OR result_row.quarantine_id IS NOT NULL THEN
        RAISE EXCEPTION 'changed quarantine wire length did not fail closed';
    END IF;

    -- Missing and mismatched tenant GUCs must not produce a durable receipt.
    PERFORM set_config('app.tenant_id', '', false);
    BEGIN
        PERFORM * FROM bi.record_task_event_quarantine_v1(
            tenant_a, project_sender, bi_receiver,
            'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
            'loopback-test', 'bi-task-events', destination_generation,
            'star-rust-log', 'task-event-log', log_generation, 2, event_uuid,
            decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
        );
        RAISE EXCEPTION 'quarantine writer accepted a missing tenant GUC';
    EXCEPTION WHEN SQLSTATE '42501' THEN
        NULL;
    END;
    PERFORM set_config('app.tenant_id', tenant_b::TEXT, false);
    BEGIN
        PERFORM * FROM bi.record_task_event_quarantine_v1(
            tenant_a, project_sender, bi_receiver,
            'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
            'loopback-test', 'bi-task-events', destination_generation,
            'star-rust-log', 'task-event-log', log_generation, 2, event_uuid,
            decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
        );
        RAISE EXCEPTION 'quarantine writer accepted a mismatched tenant GUC';
    EXCEPTION WHEN SQLSTATE '42501' THEN
        NULL;
    END;
    PERFORM set_config('app.tenant_id', tenant_a::TEXT, false);

    BEGIN
        PERFORM * FROM bi.record_task_event_quarantine_v1(
            tenant_a, project_sender, bi_receiver,
            'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
            'loopback-test', 'bi-task-events', destination_generation,
            'star-rust-log', 'task-event-log', log_generation, 2, event_uuid,
            decode(repeat('a1', 32), 'hex'), 16385, 'malformed_envelope'
        );
        RAISE EXCEPTION 'quarantine writer accepted an oversized wire length';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;

    -- A changed immutable route is a different cursor namespace and receives its own receipt.
    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_a, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test-2', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
    );
    IF result_row.outcome <> 'recorded' OR result_row.quarantine_id IS NULL
       OR result_row.quarantine_id = first_receipt THEN
        RAISE EXCEPTION 'route identity was not part of the quarantine cursor key';
    END IF;

    PERFORM set_config('app.tenant_id', tenant_b::TEXT, false);
    SELECT * INTO result_row FROM bi.record_task_event_quarantine_v1(
        tenant_b, project_sender, bi_receiver,
        'star-rust-log', 'v1', profile, 1, decode(repeat('d4', 32), 'hex'),
        'loopback-test', 'bi-task-events', destination_generation,
        'star-rust-log', 'task-event-log', log_generation, 1, event_uuid,
        decode(repeat('a1', 32), 'hex'), 1024, 'malformed_envelope'
    );
    IF result_row.outcome <> 'recorded' OR result_row.quarantine_id IS NULL THEN
        RAISE EXCEPTION 'tenant scope was not isolated in the quarantine key';
    END IF;
END;
$$;

DO $$
BEGIN
    BEGIN
        PERFORM quarantine_id FROM bi.task_event_quarantine;
        RAISE EXCEPTION 'BI runtime unexpectedly has direct quarantine table access';
    EXCEPTION WHEN insufficient_privilege THEN
        NULL;
    END;
END;
$$;
RESET ROLE;

DO $$
DECLARE
    function_oid OID;
BEGIN
    IF (SELECT count(*) FROM bi.task_event_quarantine) <> 3 THEN
        RAISE EXCEPTION 'quarantine insert/replay/conflict rows are not idempotent';
    END IF;
    IF has_table_privilege('star_bi_runtime', 'bi.task_event_quarantine', 'SELECT')
       OR has_table_privilege('star_bi_runtime', 'bi.task_event_quarantine', 'INSERT')
       OR has_table_privilege('star_bi_runtime', 'bi.task_event_quarantine', 'UPDATE')
       OR has_table_privilege('star_bi_runtime', 'bi.task_event_quarantine', 'DELETE')
       OR has_table_privilege('star_bi_runtime', 'bi.task_event_quarantine', 'TRUNCATE') THEN
        RAISE EXCEPTION 'BI runtime has direct quarantine table privileges';
    END IF;
    IF NOT has_function_privilege(
        'star_bi_runtime',
        'bi.record_task_event_quarantine_v1(uuid,uuid,uuid,character varying,character varying,uuid,bigint,bytea,character varying,character varying,uuid,character varying,character varying,uuid,bigint,uuid,bytea,integer,character varying)',
        'EXECUTE'
    ) THEN
        RAISE EXCEPTION 'BI runtime cannot call the fixed quarantine owner function';
    END IF;
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
         WHERE table_schema = 'bi' AND table_name = 'task_event_quarantine'
           AND column_name IN ('payload', 'wire', 'raw_wire', 'exact_wire', 'error_message', 'raw_error')
    ) THEN
        RAISE EXCEPTION 'quarantine schema persists event bytes or payload';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_class AS c
        JOIN pg_namespace AS n ON n.oid = c.relnamespace
         WHERE n.nspname = 'bi' AND c.relname = 'task_event_quarantine'
           AND c.relrowsecurity AND c.relforcerowsecurity
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_policies
         WHERE schemaname = 'bi' AND tablename = 'task_event_quarantine'
           AND 'star_bi_owner' = ANY(roles)
           AND qual LIKE '%tenant_id%current_setting%'
           AND with_check LIKE '%tenant_id%current_setting%'
    ) THEN
        RAISE EXCEPTION 'quarantine tenant FORCE RLS policy is missing or incomplete';
    END IF;
    IF EXISTS (
        SELECT 1 FROM pg_roles
         WHERE rolname IN ('star_bi_owner', 'star_bi_runtime')
           AND (rolcanlogin OR rolsuper OR rolbypassrls OR rolinherit)
    ) OR (SELECT count(*) FROM pg_roles WHERE rolname IN ('star_bi_owner', 'star_bi_runtime')) <> 2
      OR pg_has_role('star_bi_runtime', 'star_bi_owner', 'SET') THEN
        RAISE EXCEPTION 'BI owner/runtime must be restricted roles and runtime must not SET ROLE to owner';
    END IF;

    SELECT p.oid INTO function_oid
      FROM pg_proc AS p JOIN pg_namespace AS n ON n.oid = p.pronamespace
     WHERE n.nspname = 'bi' AND p.proname = 'record_task_event_quarantine_v1';
    IF function_oid IS NULL OR NOT (SELECT prosecdef FROM pg_proc WHERE oid = function_oid)
       OR (SELECT rolname FROM pg_roles WHERE oid = (SELECT proowner FROM pg_proc WHERE oid = function_oid)) <> 'star_bi_owner'
       OR NOT ((SELECT proconfig FROM pg_proc WHERE oid = function_oid)
               @> ARRAY['search_path=pg_catalog', 'row_security=on']) THEN
        RAISE EXCEPTION 'quarantine writer is not owned by the BI owner SECURITY DEFINER';
    END IF;
    IF EXISTS (
        SELECT 1
          FROM pg_proc AS p,
               LATERAL aclexplode(COALESCE(p.proacl, acldefault('f', p.proowner))) AS acl
         WHERE p.oid = function_oid AND acl.grantee = 0 AND acl.privilege_type = 'EXECUTE'
    ) THEN
        RAISE EXCEPTION 'quarantine writer is executable by PUBLIC';
    END IF;
END;
$$;

SET ROLE star_bi_owner;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';
DO $$
DECLARE
    visible_rows INTEGER;
BEGIN
    SELECT count(*) INTO visible_rows FROM bi.task_event_quarantine;
    IF visible_rows <> 2 THEN
        RAISE EXCEPTION 'tenant A cannot see exactly its two quarantine rows';
    END IF;
    PERFORM set_config('app.tenant_id', '99999999-9999-4999-8999-999999999999', false);
    SELECT count(*) INTO visible_rows FROM bi.task_event_quarantine;
    IF visible_rows <> 1 THEN
        RAISE EXCEPTION 'tenant B cannot see exactly its own quarantine row';
    END IF;
END;
$$;
RESET ROLE;

SET ROLE star_bi_owner;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';
DO $$
BEGIN
    BEGIN
        UPDATE bi.task_event_quarantine SET fault_code = 'content_conflict';
        RAISE EXCEPTION 'quarantine WORM allowed an update';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        DELETE FROM bi.task_event_quarantine;
        RAISE EXCEPTION 'quarantine WORM allowed a delete';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        TRUNCATE bi.task_event_quarantine;
        RAISE EXCEPTION 'quarantine WORM allowed truncation';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
END;
$$;
RESET ROLE;
