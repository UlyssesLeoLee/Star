-- @cypher schema=1 source_sha256=db80cccf8b0b919077256f07d91e5f039b5989b795cef01d74fa4f2e812d779f
-- CREATE (self:File {path:"db/tests/task-run-outbox-route-pin.sql",type:"file",language:"sql"}),(m:Module {name:"task_run_outbox_route_pin_test",type:"module",language:"sql"}),(outbox:Table {name:"multica.task_run_outbox",type:"table",language:"sql"}),(delivery:Table {name:"multica.task_run_outbox_delivery",type:"table",language:"sql"}),(pin:Table {name:"multica.task_run_outbox_delivery_route_pin",type:"table",language:"sql"}),(delivery_migration:Config {name:"2026-10-16-task-run-outbox-delivery.sql",type:"migration",language:"sql"}),(route_migration:Config {name:"2026-10-16-task-run-outbox-route-pin.sql",type:"migration",language:"sql"}),(scope_test:Function {name:"task_route_pin_scope_and_shape_assertions",type:"function",language:"sql"}),(pin_test:Function {name:"task_route_pin_idempotency_and_fence_assertions",type:"function",language:"sql"}),(rls_test:Function {name:"task_route_pin_tenant_rls_assertions",type:"function",language:"sql"}),(worm_test:Function {name:"task_route_pin_immutability_assertions",type:"function",language:"sql"});
-- CREATE (self)-[:CONTAINS]->(m),(m)-[:USES]->(outbox),(m)-[:USES]->(delivery),(m)-[:USES]->(pin),(m)-[:READS]->(delivery_migration),(m)-[:READS]->(route_migration),(m)-[:TESTS]->(scope_test),(m)-[:TESTS]->(pin_test),(m)-[:TESTS]->(rls_test),(m)-[:TESTS]->(worm_test),(delivery)-[:REFERENCES]->(outbox),(pin)-[:REFERENCES]->(outbox),(pin)-[:FENCES]->(delivery);
-- @endcypher

-- Disposable database only. This fixture provides the pre-existing Task Outbox shape, then
-- applies the real delivery and route-pin migrations twice. Never run in a shared database.
\set ON_ERROR_STOP on

DO $roles$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_multica_run_owner') THEN
        CREATE ROLE star_multica_run_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS;
    END IF;
END;
$roles$;

CREATE SCHEMA multica;
GRANT USAGE ON SCHEMA multica TO star_multica_run_owner;

CREATE TABLE multica.task_run_outbox (
    event_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    aggregate_type VARCHAR(16) NOT NULL,
    aggregate_version INTEGER NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    schema_version INTEGER NOT NULL,
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp()
);
ALTER TABLE multica.task_run_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_run_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY task_route_pin_fixture_outbox_select ON multica.task_run_outbox
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
CREATE POLICY task_route_pin_fixture_outbox_insert ON multica.task_run_outbox
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
GRANT SELECT ON multica.task_run_outbox TO star_multica_run_owner;

\ir ../migrations/2026-10-16-task-run-outbox-delivery.sql
\ir ../migrations/2026-10-16-task-run-outbox-delivery.sql
\ir ../migrations/2026-10-16-task-run-outbox-route-pin.sql
\ir ../migrations/2026-10-16-task-run-outbox-route-pin.sql

DO $shape$
DECLARE
    missing_columns INTEGER;
    route_table_oid OID := 'multica.task_run_outbox_delivery_route_pin'::REGCLASS;
BEGIN
    SELECT COUNT(*) INTO missing_columns
      FROM (VALUES
          ('tenant_id'), ('engineering_run_id'), ('event_id'), ('sender_service_id'),
          ('receiver_service_id'), ('transport_provider_id'), ('transport_provider_version'),
          ('transport_profile_id'), ('transport_profile_revision'), ('transport_profile_digest'),
          ('endpoint_identity'), ('destination_id'), ('destination_generation'), ('schema_version'),
          ('wire_sha256'), ('pinned_by_worker_id'), ('pinned_attempt_count'), ('pinned_at')
      ) AS expected(column_name)
     WHERE NOT EXISTS (
         SELECT 1 FROM information_schema.columns c
          WHERE c.table_schema = 'multica'
            AND c.table_name = 'task_run_outbox_delivery_route_pin'
            AND c.column_name = expected.column_name
     );
    IF missing_columns <> 0 THEN
        RAISE EXCEPTION 'Task route-pin schema is missing % required columns', missing_columns;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_class WHERE oid = route_table_oid AND relrowsecurity AND relforcerowsecurity
    ) THEN
        RAISE EXCEPTION 'Task route-pin table must enable and force RLS';
    END IF;
    IF (SELECT COUNT(*) FROM pg_policies
         WHERE schemaname = 'multica'
           AND tablename = 'task_run_outbox_delivery_route_pin'
           AND policyname IN ('task_run_outbox_delivery_route_pin_owner_select',
                              'task_run_outbox_delivery_route_pin_owner_insert')) <> 2 THEN
        RAISE EXCEPTION 'Task route-pin tenant policies are missing';
    END IF;
    IF has_table_privilege('star_multica_run_owner', route_table_oid, 'SELECT')
       OR has_table_privilege('star_multica_run_owner', route_table_oid, 'INSERT')
       OR has_table_privilege('star_task_run_delivery_owner', route_table_oid, 'UPDATE')
       OR has_table_privilege('star_task_run_delivery_owner', route_table_oid, 'DELETE') THEN
        RAISE EXCEPTION 'Task route-pin table privileges exceed the owner boundary';
    END IF;
    IF NOT has_function_privilege(
        'star_multica_run_owner',
        'multica.get_or_pin_task_run_outbox_delivery_route(uuid,uuid,integer,uuid,uuid,character varying,character varying,uuid,bigint,bytea,character varying,character varying,uuid,integer,bytea)',
        'EXECUTE'
    ) THEN
        RAISE EXCEPTION 'Run owner must call the fixed route-pin function';
    END IF;
    IF EXISTS (
        SELECT 1 FROM pg_auth_members
         WHERE roleid = 'star_task_run_delivery_owner'::REGROLE
    ) OR (SELECT rolcanlogin OR rolbypassrls FROM pg_roles WHERE rolname = 'star_task_run_delivery_owner') THEN
        RAISE EXCEPTION 'Task delivery owner must remain NOLOGIN, NOBYPASSRLS, and without members';
    END IF;
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
         WHERE table_schema = 'multica' AND table_name = 'task_run_outbox_delivery_route_pin'
           AND column_name IN ('payload', 'wire_bytes', 'endpoint_uri', 'credential', 'token')
    ) THEN
        RAISE EXCEPTION 'Task route pin must not retain payload, bytes, endpoint URI, or credentials';
    END IF;
END;
$shape$;

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
INSERT INTO multica.task_run_outbox (
    event_id, tenant_id, project_id, work_item_id, engineering_run_id,
    aggregate_type, aggregate_version, event_type, schema_version, actor_id, correlation_id, payload
) VALUES
    ('00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000111',
     '00000000-0000-0000-0000-000000000121', 'lifecycle', 1, 'task.lifecycle.changed', 3,
     '00000000-0000-0000-0000-000000000131', '00000000-0000-0000-0000-000000000141', '{}'),
    ('00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000112',
     '00000000-0000-0000-0000-000000000121', 'lifecycle', 1, 'task.lifecycle.changed', 3,
     '00000000-0000-0000-0000-000000000131', '00000000-0000-0000-0000-000000000142', '{}');
COMMIT;

UPDATE multica.task_run_outbox_delivery
   SET delivery_state = 'leased', attempt_count = 1,
       lease_owner_id = CASE event_id
           WHEN '00000000-0000-0000-0000-000000000201' THEN '00000000-0000-0000-0000-000000000301'::UUID
           ELSE '00000000-0000-0000-0000-000000000302'::UUID
       END,
       lease_expires_at = clock_timestamp() + INTERVAL '2 minutes'
 WHERE tenant_id = '00000000-0000-0000-0000-000000000001'
   AND event_id IN ('00000000-0000-0000-0000-000000000201',
                    '00000000-0000-0000-0000-000000000202');

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
SET LOCAL ROLE star_multica_run_owner;
DO $pin$
DECLARE
    pinned RECORD;
BEGIN
    SELECT * INTO STRICT pinned
      FROM multica.get_or_pin_task_run_outbox_delivery_route(
        '00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000301', 1,
        '00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000402',
        'nats', '2.11.0', '00000000-0000-0000-0000-000000000501', 1, decode(repeat('b', 64), 'hex'),
        'endpoint-a', 'log-a', '00000000-0000-0000-0000-000000000601', 3,
        decode(repeat('a', 64), 'hex')
      );
    IF pinned.pinned_by_worker_id <> '00000000-0000-0000-0000-000000000301'
       OR pinned.pinned_attempt_count <> 1
       OR pinned.schema_version <> 3
       OR pinned.wire_sha256 <> decode(repeat('a', 64), 'hex') THEN
        RAISE EXCEPTION 'Initial Task route pin differs from its exact route snapshot';
    END IF;
END;
$pin$;
COMMIT;

UPDATE multica.task_run_outbox_delivery
   SET attempt_count = 2, lease_owner_id = '00000000-0000-0000-0000-000000000302',
       lease_expires_at = clock_timestamp() + INTERVAL '2 minutes'
 WHERE tenant_id = '00000000-0000-0000-0000-000000000001'
   AND event_id = '00000000-0000-0000-0000-000000000201';

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
SET LOCAL ROLE star_multica_run_owner;
DO $replay$
DECLARE
    pinned RECORD;
    rejected BOOLEAN := FALSE;
BEGIN
    SELECT * INTO STRICT pinned
      FROM multica.get_or_pin_task_run_outbox_delivery_route(
        '00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000302', 2,
        '00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000402',
        'nats', '2.11.0', '00000000-0000-0000-0000-000000000501', 1, decode(repeat('b', 64), 'hex'),
        'endpoint-a', 'log-a', '00000000-0000-0000-0000-000000000601', 3,
        decode(repeat('a', 64), 'hex')
      );
    IF pinned.pinned_by_worker_id <> '00000000-0000-0000-0000-000000000301'
       OR pinned.pinned_attempt_count <> 1 THEN
        RAISE EXCEPTION 'Exact retry must preserve the first immutable route pin';
    END IF;
    BEGIN
        PERFORM * FROM multica.get_or_pin_task_run_outbox_delivery_route(
          '00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000302', 2,
          '00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000402',
          'nats', '2.11.0', '00000000-0000-0000-0000-000000000501', 1, decode(repeat('b', 64), 'hex'),
          'endpoint-a', 'log-a', '00000000-0000-0000-0000-000000000601', 3,
          decode(repeat('c', 64), 'hex')
        );
    EXCEPTION WHEN SQLSTATE '55000' THEN
        rejected := TRUE;
    END;
    IF NOT rejected THEN
        RAISE EXCEPTION 'Changed exact-wire digest must be rejected';
    END IF;
    rejected := FALSE;
    BEGIN
        PERFORM * FROM multica.get_or_pin_task_run_outbox_delivery_route(
          '00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000301', 2,
          '00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000402',
          'nats', '2.11.0', '00000000-0000-0000-0000-000000000501', 1, decode(repeat('b', 64), 'hex'),
          'endpoint-a', 'log-a', '00000000-0000-0000-0000-000000000601', 3,
          decode(repeat('a', 64), 'hex')
        );
    EXCEPTION WHEN SQLSTATE '40001' THEN
        rejected := TRUE;
    END;
    IF NOT rejected THEN
        RAISE EXCEPTION 'Stale delivery lease owner must be rejected';
    END IF;
END;
$replay$;
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000002';
SET LOCAL ROLE star_task_run_delivery_owner;
DO $rls$
BEGIN
    IF EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery_route_pin
                WHERE event_id = '00000000-0000-0000-0000-000000000201') THEN
        RAISE EXCEPTION 'Task route-pin RLS exposed another tenant row';
    END IF;
END;
$rls$;
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
SET LOCAL ROLE star_task_run_delivery_owner;
DO $visible$
BEGIN
    IF (SELECT COUNT(*) FROM multica.task_run_outbox_delivery_route_pin
         WHERE event_id = '00000000-0000-0000-0000-000000000201') <> 1 THEN
        RAISE EXCEPTION 'Task delivery owner cannot read its tenant route pin';
    END IF;
END;
$visible$;
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000001';
SET LOCAL ROLE star_multica_run_owner;
SELECT * FROM multica.get_or_pin_task_run_outbox_delivery_route(
    '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000302', 1,
    '00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000402',
    'nats', '2.11.0', '00000000-0000-0000-0000-000000000501', 1, decode(repeat('b', 64), 'hex'),
    'endpoint-a', 'log-a', '00000000-0000-0000-0000-000000000601', 3,
    decode(repeat('a', 64), 'hex')
);
ROLLBACK;
DO $rollback$
BEGIN
    IF EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery_route_pin
                WHERE event_id = '00000000-0000-0000-0000-000000000202') THEN
        RAISE EXCEPTION 'Rolled-back route pin unexpectedly persisted';
    END IF;
END;
$rollback$;

DO $worm$
DECLARE
    rejected BOOLEAN;
BEGIN
    rejected := FALSE;
    BEGIN
        UPDATE multica.task_run_outbox_delivery_route_pin
           SET destination_id = 'mutated'
         WHERE event_id = '00000000-0000-0000-0000-000000000201';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        rejected := TRUE;
    END;
    IF NOT rejected THEN RAISE EXCEPTION 'Route-pin UPDATE was not blocked'; END IF;

    rejected := FALSE;
    BEGIN
        DELETE FROM multica.task_run_outbox_delivery_route_pin
         WHERE event_id = '00000000-0000-0000-0000-000000000201';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        rejected := TRUE;
    END;
    IF NOT rejected THEN RAISE EXCEPTION 'Route-pin DELETE was not blocked'; END IF;

    rejected := FALSE;
    BEGIN
        EXECUTE 'TRUNCATE TABLE multica.task_run_outbox_delivery_route_pin';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        rejected := TRUE;
    END;
    IF NOT rejected THEN RAISE EXCEPTION 'Route-pin TRUNCATE was not blocked'; END IF;
    IF (SELECT COUNT(*) FROM multica.task_run_outbox_delivery_route_pin) <> 1 THEN
        RAISE EXCEPTION 'Route-pin WORM assertions changed or removed the pin';
    END IF;
END;
$worm$;
