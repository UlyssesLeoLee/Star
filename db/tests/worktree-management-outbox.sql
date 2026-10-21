-- @cypher schema=1 source_sha256=a36cd61a98a92bfdf6ae50649dff0bef30fc3a3b9ed330a58bc8a8161bcfa567
-- MERGE (self:File {path:"db/tests/worktree-management-outbox.sql"})
-- MERGE (migration:Config {id:"db/migrations/2026-10-20-worktree-management-outbox.sql"})
-- MERGE (outbox:Table {id:"multica.worktree_management_outbox"})
-- MERGE (rls:Function {id:"db/tests/worktree-management-outbox.sql::tenant_rls_assertions"})
-- MERGE (immutable:Function {id:"db/tests/worktree-management-outbox.sql::append_only_assertions"})
-- MERGE (replay:Function {id:"db/tests/worktree-management-outbox.sql::migration_replay_assertions"})
-- MERGE (self)-[:READS]->(migration)
-- MERGE (self)-[:TESTS]->(outbox)
-- MERGE (self)-[:DEFINES]->(rls)
-- MERGE (self)-[:DEFINES]->(immutable)
-- MERGE (self)-[:DEFINES]->(replay)
-- MERGE (rls)-[:USES]->(outbox)
-- MERGE (immutable)-[:USES]->(outbox)
-- MERGE (replay)-[:USES]->(migration)
-- @endcypher

-- Run only in a fresh, disposable PostgreSQL database. The fixture deliberately supplies
-- the existing Worktree and management-plan owner tables before applying the real migration.
\set ON_ERROR_STOP on

DO $precondition$
BEGIN
    IF to_regclass('public.worktree_canvas_worktree') IS NOT NULL
       OR to_regclass('multica.worktree_management_plan') IS NOT NULL
       OR to_regclass('multica.worktree_management_outbox') IS NOT NULL THEN
        RAISE EXCEPTION 'Use a fresh disposable database for this Worktree management Outbox test';
    END IF;
END;
$precondition$;

CREATE SCHEMA IF NOT EXISTS multica;
CREATE TABLE public.worktree_canvas_worktree (
    tenant_id UUID NOT NULL,
    id UUID NOT NULL,
    PRIMARY KEY (tenant_id, id)
);
CREATE TABLE multica.worktree_management_plan (
    tenant_id UUID NOT NULL,
    plan_id UUID NOT NULL,
    PRIMARY KEY (plan_id),
    UNIQUE (tenant_id, plan_id)
);

\ir ../migrations/2026-10-20-worktree-management-outbox.sql
\ir ../migrations/2026-10-20-worktree-management-outbox.sql

DO $role$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_wm_outbox_test') THEN
        CREATE ROLE star_wm_outbox_test NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOINHERIT NOBYPASSRLS;
    END IF;
    IF EXISTS (
        SELECT 1 FROM pg_roles
        WHERE rolname = 'star_wm_outbox_test'
          AND (rolcanlogin OR rolsuper OR rolcreatedb OR rolcreaterole OR rolbypassrls)
    ) THEN
        RAISE EXCEPTION 'Outbox test role must remain non-login and non-bypass';
    END IF;
END;
$role$;

GRANT USAGE ON SCHEMA multica TO star_wm_outbox_test;
GRANT SELECT, INSERT, UPDATE, DELETE, TRUNCATE
    ON multica.worktree_management_outbox TO star_wm_outbox_test;

INSERT INTO public.worktree_canvas_worktree (tenant_id, id)
VALUES ('10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001');
INSERT INTO multica.worktree_management_plan (tenant_id, plan_id)
VALUES
    ('10000000-0000-4000-8000-000000000001', '30000000-0000-4000-8000-000000000001'),
    ('10000000-0000-4000-8000-000000000001', '30000000-0000-4000-8000-000000000002'),
    ('10000000-0000-4000-8000-000000000001', '30000000-0000-4000-8000-000000000003'),
    ('10000000-0000-4000-8000-000000000001', '30000000-0000-4000-8000-000000000004'),
    ('10000000-0000-4000-8000-000000000001', '30000000-0000-4000-8000-000000000005');

BEGIN;
SET LOCAL ROLE star_wm_outbox_test;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
INSERT INTO multica.worktree_management_outbox (
    tenant_id, project_id, worktree_id, worktree_version, plan_id,
    event_type, schema_version, actor_id, correlation_id, payload
) VALUES (
    '10000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001', 2,
    '30000000-0000-4000-8000-000000000001', 'worktree.archived', 1,
    '50000000-0000-4000-8000-000000000001',
    '60000000-0000-4000-8000-000000000001',
    '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000001","transition":"archive","before":{"archived":false,"version":1},"after":{"archived":true,"version":2}}'
);
COMMIT;

DO $shape$
BEGIN
    IF (SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'multica' AND c.relname = 'worktree_management_outbox'
           AND c.relkind = 'r' AND c.relrowsecurity AND c.relforcerowsecurity) <> 1 THEN
        RAISE EXCEPTION 'Outbox requires ENABLE and FORCE RLS';
    END IF;
    IF (SELECT count(*) FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'multica' AND c.relname = 'worktree_management_outbox'
           AND NOT t.tgisinternal
           AND t.tgname IN ('worktree_management_outbox_no_mutation',
                            'worktree_management_outbox_no_truncate')) <> 2 THEN
        RAISE EXCEPTION 'Outbox append-only UPDATE/DELETE/TRUNCATE guards are incomplete';
    END IF;
END;
$shape$;

BEGIN;
SET LOCAL ROLE star_wm_outbox_test;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
INSERT INTO multica.worktree_management_outbox (
    tenant_id, project_id, worktree_id, worktree_version, plan_id,
    event_type, schema_version, actor_id, correlation_id, payload
) VALUES (
    '10000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001', 3,
    '30000000-0000-4000-8000-000000000002', 'worktree.restored', 1,
    '50000000-0000-4000-8000-000000000001',
    '60000000-0000-4000-8000-000000000002',
    '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000002","transition":"restore","before":{"archived":true,"version":2},"after":{"archived":false,"version":3}}'
);
COMMIT;

BEGIN;
SET LOCAL ROLE star_wm_outbox_test;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000002';
DO $rls$
DECLARE visible_count BIGINT;
BEGIN
    SELECT count(*) INTO visible_count FROM multica.worktree_management_outbox;
    IF visible_count <> 0 THEN
        RAISE EXCEPTION 'Outbox tenant read escaped its RLS scope';
    END IF;
    BEGIN
        INSERT INTO multica.worktree_management_outbox (
            tenant_id, project_id, worktree_id, worktree_version, plan_id,
            event_type, schema_version, actor_id, correlation_id, payload
        ) VALUES (
            '10000000-0000-4000-8000-000000000001',
            '40000000-0000-4000-8000-000000000001',
            '20000000-0000-4000-8000-000000000001', 3,
            '30000000-0000-4000-8000-000000000002', 'worktree.restored', 1,
            '50000000-0000-4000-8000-000000000001',
            '60000000-0000-4000-8000-000000000002',
            '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000002","transition":"restore","before":{"archived":true,"version":2},"after":{"archived":false,"version":3}}'
        );
        RAISE EXCEPTION 'Expected tenant RLS to reject a cross-tenant insert';
    EXCEPTION WHEN SQLSTATE '42501' THEN
        NULL;
    END;
END;
$rls$;
ROLLBACK;

BEGIN;
SET LOCAL ROLE star_wm_outbox_test;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
DO $payload_guard$
BEGIN
    BEGIN
        INSERT INTO multica.worktree_management_outbox (
            tenant_id, project_id, worktree_id, worktree_version, plan_id,
            event_type, schema_version, actor_id, correlation_id, payload
        ) VALUES (
            '10000000-0000-4000-8000-000000000001',
            '40000000-0000-4000-8000-000000000001',
            '20000000-0000-4000-8000-000000000001', 5,
            '30000000-0000-4000-8000-000000000003', 'worktree.archived', 1,
            '50000000-0000-4000-8000-000000000001',
            '60000000-0000-4000-8000-000000000003',
            '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000003","transition":"archive","before":{"archived":false,"version":2},"after":{"archived":true,"version":5}}'
        );
        RAISE EXCEPTION 'Expected discontinuous Worktree versions to be rejected';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;

    BEGIN
        INSERT INTO multica.worktree_management_outbox (
            tenant_id, project_id, worktree_id, worktree_version, plan_id,
            event_type, schema_version, actor_id, correlation_id, payload
        ) VALUES (
            '10000000-0000-4000-8000-000000000001',
            '40000000-0000-4000-8000-000000000001',
            '20000000-0000-4000-8000-000000000001', 4,
            '30000000-0000-4000-8000-000000000004', 'worktree.archived', 1,
            '50000000-0000-4000-8000-000000000001',
            '60000000-0000-4000-8000-000000000004',
            '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000004","transition":"archive","before":{"archived":false,"version":3},"after":{"archived":true,"version":4,"extra":"forbidden"}}'
        );
        RAISE EXCEPTION 'Expected extra payload fields to be rejected';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;

    BEGIN
        INSERT INTO multica.worktree_management_outbox (
            tenant_id, project_id, worktree_id, worktree_version, plan_id,
            event_type, schema_version, actor_id, correlation_id, payload
        ) VALUES (
            '10000000-0000-4000-8000-000000000001',
            '40000000-0000-4000-8000-000000000001',
            '20000000-0000-4000-8000-000000000001', 4,
            '30000000-0000-4000-8000-000000000005', 'worktree.archived', 1,
            '50000000-0000-4000-8000-000000000001',
            '60000000-0000-4000-8000-000000000005',
            '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000005","transition":"archive","before":{"archived":false,"version":3.5},"after":{"archived":true,"version":4}}'
        );
        RAISE EXCEPTION 'Expected fractional Worktree versions to be rejected';
    EXCEPTION WHEN check_violation THEN
        NULL;
    END;
END;
$payload_guard$;
ROLLBACK;

BEGIN;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
RESET ROLE;
DO $append_only$
BEGIN
    BEGIN
        UPDATE multica.worktree_management_outbox SET payload = '{}'::JSONB;
        RAISE EXCEPTION 'Expected UPDATE to be rejected';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        DELETE FROM multica.worktree_management_outbox;
        RAISE EXCEPTION 'Expected DELETE to be rejected';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        TRUNCATE multica.worktree_management_outbox;
        RAISE EXCEPTION 'Expected TRUNCATE to be rejected';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
    BEGIN
        INSERT INTO multica.worktree_management_outbox (
            tenant_id, project_id, worktree_id, worktree_version, plan_id,
            event_type, schema_version, actor_id, correlation_id, payload
        ) VALUES (
            '10000000-0000-4000-8000-000000000001',
            '40000000-0000-4000-8000-000000000001',
            '20000000-0000-4000-8000-000000000001', 2,
            '30000000-0000-4000-8000-000000000003', 'worktree.restored', 1,
            '50000000-0000-4000-8000-000000000001',
            '60000000-0000-4000-8000-000000000002',
            '{"worktree_id":"20000000-0000-4000-8000-000000000001","plan_id":"30000000-0000-4000-8000-000000000003","transition":"restore","before":{"archived":true,"version":1},"after":{"archived":false,"version":2}}'
        );
        RAISE EXCEPTION 'Expected one Worktree version to have only one event';
    EXCEPTION WHEN unique_violation THEN
        NULL;
    END;
END;
$append_only$;
ROLLBACK;
