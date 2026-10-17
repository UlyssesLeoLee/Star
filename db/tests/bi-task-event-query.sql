-- SPDX-License-Identifier: MIT OR Apache-2.0
-- @cypher schema=1 source_sha256=PLACEHOLDER
-- MERGE (self:File {path:"db/tests/bi-task-event-query.sql"})
-- MERGE (test:Symbol {id:"db/tests/bi-task-event-query.sql::bi_task_event_query_contract",kind:"test"})
-- MERGE (function:Database {id:"bi.list_task_event_page_v1",kind:"stored_function"})
-- MERGE (fact:Table {id:"bi.task_event_fact",kind:"table"})
-- MERGE (self)-[:DEFINES]->(test)
-- MERGE (test)-[:TESTS]->(function)
-- MERGE (test)-[:READS]->(fact)
-- @endcypher

-- Run only against the isolated disposable database created by the 9F4E runner.
SET ROLE star_bi_runtime;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';

DO $$
DECLARE
    tenant UUID := '11111111-1111-4111-8111-111111111111';
    project UUID := '22222222-2222-4222-8222-222222222222';
    run_id UUID := '33333333-3333-4333-8333-333333333333';
    first_page RECORD;
    second_page RECORD;
    returned_count INTEGER;
    public_execute BOOLEAN;
BEGIN
    IF NOT has_function_privilege(
        'star_bi_runtime',
        'bi.list_task_event_page_v1(uuid,uuid,uuid,timestamp with time zone,uuid,integer)',
        'EXECUTE'
    ) OR has_table_privilege('star_bi_runtime', 'bi.task_event_fact', 'SELECT') THEN
        RAISE EXCEPTION 'BI query runtime grants are not function-only';
    END IF;

    SELECT EXISTS (
        SELECT 1
          FROM pg_proc AS p
          CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl, acldefault('f', p.proowner))) AS acl
         WHERE p.oid = 'bi.list_task_event_page_v1(uuid,uuid,uuid,timestamp with time zone,uuid,integer)'::REGPROCEDURE
           AND acl.grantee = 0
           AND acl.privilege_type = 'EXECUTE'
    ) INTO public_execute;
    IF public_execute THEN
        RAISE EXCEPTION 'PUBLIC can execute the BI query function';
    END IF;

    SELECT count(*) INTO returned_count
      FROM bi.list_task_event_page_v1(tenant, project, run_id, NULL, NULL, 101);
    IF returned_count <> 2 THEN
        RAISE EXCEPTION 'BI query did not return the two scoped observed facts';
    END IF;

    SELECT * INTO first_page
      FROM bi.list_task_event_page_v1(tenant, project, run_id, NULL, NULL, 1);
    SELECT * INTO second_page
      FROM bi.list_task_event_page_v1(
          tenant, project, run_id, first_page.occurred_at, first_page.event_id, 1
      );
    IF first_page.event_id <> '00000000-0000-4000-8000-000000000001'
       OR second_page.event_id <> '44444444-4444-4444-8444-444444444444'
       OR second_page.lifecycle_status <> 'in_progress'
       OR second_page.to_status <> 'in_progress'
       OR second_page.owner_changed IS NULL THEN
        RAISE EXCEPTION 'BI query did not apply the stable mixed-direction keyset cursor';
    END IF;

    SELECT count(*) INTO returned_count
      FROM bi.list_task_event_page_v1(
          tenant, 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', run_id, NULL, NULL, 10
      );
    IF returned_count <> 0 THEN
        RAISE EXCEPTION 'BI query crossed the requested Project scope';
    END IF;

    BEGIN
        PERFORM * FROM bi.list_task_event_page_v1(
            '99999999-9999-4999-8999-999999999999', project, run_id, NULL, NULL, 1
        );
        RAISE EXCEPTION 'BI query accepted a tenant different from the call scope';
    EXCEPTION WHEN insufficient_privilege THEN
        NULL;
    END;

    BEGIN
        PERFORM * FROM bi.list_task_event_page_v1(tenant, project, run_id, NULL, NULL, 102);
        RAISE EXCEPTION 'BI query accepted an oversized page';
    EXCEPTION WHEN invalid_parameter_value THEN
        NULL;
    END;

    BEGIN
        PERFORM * FROM bi.list_task_event_page_v1(
            tenant, project, run_id, now(), NULL, 1
        );
        RAISE EXCEPTION 'BI query accepted a partial cursor';
    EXCEPTION WHEN invalid_parameter_value THEN
        NULL;
    END;
END;
$$;

RESET ROLE;

DO $$
BEGIN
    IF (SELECT pg_get_function_result(
            'bi.list_task_event_page_v1(uuid,uuid,uuid,timestamp with time zone,uuid,integer)'::REGPROCEDURE
        ) ILIKE '%payload%') THEN
        RAISE EXCEPTION 'BI query return contract exposes the stored payload';
    END IF;
END;
$$;
