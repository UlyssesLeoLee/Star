-- @cypher schema=1 source_sha256=ed962bde5ee2617a3e0885b074e8b04b5b2032373175a025b385eb131fc424e0
-- MERGE (self:File {path:"db/tests/bi-task-event-raw-payload-seed.sql"})
-- MERGE (test:Symbol {id:"db/tests/bi-task-event-raw-payload-seed.sql::seed_legacy_bi_fact",kind:"test"})
-- MERGE (function:Database {id:"bi.commit_task_event_v1",kind:"stored_function"})
-- MERGE (self)-[:DEFINES]->(test)
-- MERGE (test)-[:CALLS]->(function)
-- @endcypher

-- Simulate a row written by the 9F4E migration before the projection migration.
SET ROLE star_bi_runtime;
SET app.tenant_id = '11111111-1111-4111-8111-111111111111';

DO $$
DECLARE
    payload JSONB := jsonb_build_object(
        'schema_version', 3,
        'aggregate', jsonb_build_object('type', 'lifecycle', 'version', 4),
        'work_item_id', '55555555-5555-4555-8555-555555555555',
        'owner', jsonb_build_object(
            'tenant_id', '11111111-1111-4111-8111-111111111111',
            'project_id', '22222222-2222-4222-8222-222222222222',
            'engineering_run_id', '33333333-3333-4333-8333-333333333333'
        ),
        'lifecycle', jsonb_build_object('status', 'claimed', 'review_state', 'none'),
        'change', jsonb_build_object(
            'event_type', 'task.lifecycle.changed',
            'from_status', 'pending',
            'to_status', 'claimed',
            'private_note', 'LEGACY-RAW-PAYLOAD-SENTINEL'
        ),
        'host_context', jsonb_build_object('branch_name', 'must-not-survive-migration')
    );
BEGIN
    PERFORM * FROM bi.commit_task_event_v1(
        '11111111-1111-4111-8111-111111111111',
        '22222222-2222-4222-8222-222222222222',
        '33333333-3333-4333-8333-333333333333',
        '00000000-0000-4000-8000-000000000001',
        '55555555-5555-4555-8555-555555555555',
        'lifecycle', 4, 'task.lifecycle.changed', 3,
        '66666666-6666-4666-8666-666666666666',
        '77777777-7777-4777-8777-777777777777',
        '2026-10-16T00:00:00Z', payload,
        '88888888-8888-4888-8888-888888888888',
        '99999999-9999-4999-8999-999999999999',
        'star-rust-log', 'v1', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 1,
        decode(repeat('d4', 32), 'hex'), 'loopback-test', 'bi-task-events',
        'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
        decode(repeat('e5', 32), 'hex'), decode(repeat('a1', 32), 'hex')
    );
END;
$$;

RESET ROLE;
