-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/tests/run-infrastructure-binding-master.sql"})
-- MERGE (assertions:Function {id:"db/tests/run-infrastructure-binding-master.sql::role_and_schema_assertions"})
-- MERGE (seed:Function {id:"db/tests/run-infrastructure-binding-master.sql::seed_engineering_run"})
-- MERGE (revision1:Function {id:"db/tests/run-infrastructure-binding-master.sql::write_first_binding_revision"})
-- MERGE (revision2:Function {id:"db/tests/run-infrastructure-binding-master.sql::supersede_binding_revision"})
-- MERGE (worm:Function {id:"db/tests/run-infrastructure-binding-master.sql::assert_immutable_master_and_event"})
-- MERGE (binding:Table {id:"multica.run_infrastructure_binding_revision",class:"Master"})
-- MERGE (event:Table {id:"multica.run_infrastructure_binding_event",class:"Transaction"})
-- MERGE (owner:Role {id:"star_multica_run_owner"})
-- MERGE (self)-[:DEFINES]->(assertions)
-- MERGE (self)-[:DEFINES]->(seed)
-- MERGE (self)-[:DEFINES]->(revision1)
-- MERGE (self)-[:DEFINES]->(revision2)
-- MERGE (self)-[:DEFINES]->(worm)
-- MERGE (assertions)-[:READS]->(binding)
-- MERGE (assertions)-[:READS]->(event)
-- MERGE (assertions)-[:USES]->(owner)
-- MERGE (seed)-[:WRITES]->(binding)
-- MERGE (revision1)-[:WRITES]->(binding)
-- MERGE (revision1)-[:WRITES]->(event)
-- MERGE (revision2)-[:WRITES]->(binding)
-- MERGE (revision2)-[:WRITES]->(event)
-- MERGE (worm)-[:USES]->(binding)
-- MERGE (worm)-[:USES]->(event)
-- @endcypher

DO $assertions$
DECLARE
    role_is_restricted BOOLEAN;
    role_name TEXT;
BEGIN
    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO role_is_restricted
      FROM pg_roles
     WHERE rolname = 'star_multica_run_owner';
    IF role_is_restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'Run owner role is missing or has unsafe role attributes';
    END IF;
    IF NOT has_schema_privilege('star_multica_run_owner', 'multica', 'USAGE')
       OR has_schema_privilege('star_multica_run_owner', 'multica', 'CREATE') THEN
        RAISE EXCEPTION 'Run owner schema privileges are not least-privilege';
    END IF;
    IF (SELECT count(*) FROM pg_class AS c
          JOIN pg_namespace AS n ON n.oid = c.relnamespace
         WHERE n.nspname = 'multica'
           AND c.relname IN ('run_infrastructure_binding_revision', 'run_infrastructure_binding_event')
           AND c.relkind = 'r' AND c.relrowsecurity AND c.relforcerowsecurity) <> 2 THEN
        RAISE EXCEPTION 'Binding Master and Transaction tables require ENABLE and FORCE RLS';
    END IF;
    IF (SELECT count(*) FROM pg_policies
         WHERE schemaname = 'multica'
           AND tablename IN ('run_infrastructure_binding_revision', 'run_infrastructure_binding_event')
           AND qual ILIKE '%app.tenant_id%'
           AND with_check ILIKE '%app.tenant_id%') <> 2 THEN
        RAISE EXCEPTION 'Binding tables require tenant-scoped RLS policies';
    END IF;
    IF NOT has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'SELECT')
       OR NOT has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'INSERT')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'UPDATE')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'DELETE')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'TRUNCATE')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'REFERENCES')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'TRIGGER')
       OR NOT has_column_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'valid_to', 'UPDATE')
       OR has_column_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_revision', 'binding_snapshot', 'UPDATE') THEN
        RAISE EXCEPTION 'Binding Master grants are not limited to insert, read and one-time closure';
    END IF;
    IF NOT has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'SELECT')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'INSERT')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'UPDATE')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'DELETE')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'TRUNCATE')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'REFERENCES')
       OR has_table_privilege('star_multica_run_owner', 'multica.run_infrastructure_binding_event', 'TRIGGER') THEN
        RAISE EXCEPTION 'Binding event grants are not append-only';
    END IF;
    IF (SELECT count(*) FROM pg_trigger AS t
          JOIN pg_class AS c ON c.oid = t.tgrelid
          JOIN pg_namespace AS n ON n.oid = c.relnamespace
         WHERE n.nspname = 'multica' AND NOT t.tgisinternal
           AND t.tgname IN (
               'run_infrastructure_binding_revision_guard',
               'run_infrastructure_binding_event_insert',
               'run_infrastructure_binding_event_supersede',
               'run_infrastructure_binding_event_no_update_delete',
               'run_infrastructure_binding_event_no_truncate'
           )) <> 5 THEN
        RAISE EXCEPTION 'Binding SCD2 and append-only guards are incomplete';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.run_infrastructure_binding_revision'::regclass
           AND contype = 'f' AND confrelid = 'multica.engineering_run'::regclass
    ) THEN
        RAISE EXCEPTION 'Binding must reference the canonical Engineering Run scope';
    END IF;
    FOREACH role_name IN ARRAY ARRAY['star_infrastructure_owner', 'star_automation_owner'] LOOP
        IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = role_name)
           AND (has_table_privilege(role_name, 'multica.run_infrastructure_binding_revision', 'SELECT')
             OR has_table_privilege(role_name, 'multica.run_infrastructure_binding_revision', 'INSERT')
             OR has_table_privilege(role_name, 'multica.run_infrastructure_binding_event', 'SELECT')
             OR has_table_privilege(role_name, 'multica.run_infrastructure_binding_event', 'INSERT')) THEN
            RAISE EXCEPTION 'another owner role received Run binding table privileges';
        END IF;
    END LOOP;
END
$assertions$;

BEGIN;
SET LOCAL session_replication_role = replica;
INSERT INTO scm.cloud_branch (branch_id, tenant_id, project_id, repository_id, remote_identity)
VALUES (
    '30000000-0000-4000-8000-000000000001',
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    'test://run-infrastructure-binding-master'
);
INSERT INTO multica.engineering_run (
    engineering_run_id, tenant_id, project_id, repository_id, branch_id
) VALUES (
    '50000000-0000-4000-8000-000000000001',
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001'
);
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
SET LOCAL ROLE star_multica_run_owner;
DO $event_insert_denied$
DECLARE
    blocked BOOLEAN := FALSE;
BEGIN
    BEGIN
        INSERT INTO multica.run_infrastructure_binding_event (
            tenant_id, project_id, branch_id, engineering_run_id,
            binding_id, binding_revision, event_type, binding_digest, actor_id
        )
        SELECT tenant_id, project_id, branch_id, engineering_run_id,
               binding_id, binding_revision, 'created', binding_digest,
               '60000000-0000-4000-8000-000000000001'
          FROM multica.run_infrastructure_binding_revision
         WHERE valid_to IS NULL;
    EXCEPTION WHEN insufficient_privilege THEN
        blocked := TRUE;
    END;
    IF NOT blocked THEN
        RAISE EXCEPTION 'Run owner forged a binding audit event directly';
    END IF;
END
$event_insert_denied$;
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
SET LOCAL app.actor_id = '60000000-0000-4000-8000-000000000001';
SET LOCAL ROLE star_multica_run_owner;
INSERT INTO multica.run_infrastructure_binding_revision (
    tenant_id, project_id, branch_id, engineering_run_id, binding_id,
    binding_revision, schema_version, binding_snapshot, binding_digest, changed_by
)
SELECT
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    '70000000-0000-4000-8000-000000000001',
    1, 2,
    jsonb_build_object(
        'schema_version', 2,
        'tenant_id', '10000000-0000-4000-8000-000000000001',
        'project_id', '20000000-0000-4000-8000-000000000001',
        'branch_id', '30000000-0000-4000-8000-000000000001',
        'engineering_run_id', '50000000-0000-4000-8000-000000000001',
        'binding_id', '70000000-0000-4000-8000-000000000001',
        'binding_revision', 1,
        'host_environment_id', '80000000-0000-4000-8000-000000000001',
        'k3s_cluster_id', '90000000-0000-4000-8000-000000000001',
        'k3s_cluster_generation', 1,
        'cluster_placement_policy', 'PinnedCluster',
        'provider', 'MultipassK3sCluster',
        'provider_profile_id', 'a0000000-0000-4000-8000-000000000001',
        'provider_profile_revision', 1,
        'provider_profile_digest', to_jsonb(array_fill(9, ARRAY[32])),
        'namespace', 'run-1',
        'service_account', 'agent-1',
        'runtime_profile_id', 'b0000000-0000-4000-8000-000000000001',
        'runtime_profile_revision', 1,
        'runtime_profile_digest', to_jsonb(array_fill(11, ARRAY[32])),
        'resource_budget', jsonb_build_object(
            'max_rss_bytes', 67108864, 'max_cpu_ms', 1000, 'max_runtime_ms', 30000,
            'max_child_processes', 1, 'max_parallel_tools', 1,
            'max_provider_calls', 1, 'max_output_bytes', 4096, 'max_event_buffer_bytes', 4096
        )
    ),
    decode(repeat('a', 64), 'hex'),
    '60000000-0000-4000-8000-000000000001';
DO $visible$
BEGIN
    IF (SELECT count(*) FROM multica.run_infrastructure_binding_revision) <> 1 THEN
        RAISE EXCEPTION 'Run owner cannot read its current binding';
    END IF;
END
$visible$;
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '10000000-0000-4000-8000-000000000001';
SET LOCAL app.actor_id = '60000000-0000-4000-8000-000000000001';
SET LOCAL ROLE star_multica_run_owner;
UPDATE multica.run_infrastructure_binding_revision
   SET valid_to = transaction_timestamp()
 WHERE tenant_id = '10000000-0000-4000-8000-000000000001'
   AND binding_id = '70000000-0000-4000-8000-000000000001'
   AND binding_revision = 1 AND valid_to IS NULL;
INSERT INTO multica.run_infrastructure_binding_revision (
    tenant_id, project_id, branch_id, engineering_run_id, binding_id,
    binding_revision, schema_version, binding_snapshot, binding_digest, changed_by
)
SELECT
    '10000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    '30000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    '70000000-0000-4000-8000-000000000001',
    2, 2,
    jsonb_build_object(
        'schema_version', 2,
        'tenant_id', '10000000-0000-4000-8000-000000000001',
        'project_id', '20000000-0000-4000-8000-000000000001',
        'branch_id', '30000000-0000-4000-8000-000000000001',
        'engineering_run_id', '50000000-0000-4000-8000-000000000001',
        'binding_id', '70000000-0000-4000-8000-000000000001',
        'binding_revision', 2,
        'host_environment_id', '80000000-0000-4000-8000-000000000001',
        'k3s_cluster_id', '90000000-0000-4000-8000-000000000001',
        'k3s_cluster_generation', 1,
        'cluster_placement_policy', 'PinnedCluster',
        'provider', 'MultipassK3sCluster',
        'provider_profile_id', 'a0000000-0000-4000-8000-000000000001',
        'provider_profile_revision', 1,
        'provider_profile_digest', to_jsonb(array_fill(9, ARRAY[32])),
        'namespace', 'run-1',
        'service_account', 'agent-1',
        'runtime_profile_id', 'b0000000-0000-4000-8000-000000000001',
        'runtime_profile_revision', 1,
        'runtime_profile_digest', to_jsonb(array_fill(11, ARRAY[32])),
        'resource_budget', jsonb_build_object(
            'max_rss_bytes', 67108864, 'max_cpu_ms', 1000, 'max_runtime_ms', 30000,
            'max_child_processes', 1, 'max_parallel_tools', 1,
            'max_provider_calls', 1, 'max_output_bytes', 4096, 'max_event_buffer_bytes', 4096
        )
    ),
    decode(repeat('b', 64), 'hex'),
    '60000000-0000-4000-8000-000000000001';
DO $revised$
BEGIN
    IF (SELECT count(*) FROM multica.run_infrastructure_binding_revision WHERE valid_to IS NULL) <> 1
       OR (SELECT binding_revision FROM multica.run_infrastructure_binding_revision WHERE valid_to IS NULL) <> 2
       OR (SELECT count(*) FROM multica.run_infrastructure_binding_event) <> 3 THEN
        RAISE EXCEPTION 'Binding CAS revision and audit facts are inconsistent';
    END IF;
END
$revised$;
COMMIT;

BEGIN;
SET LOCAL app.tenant_id = '20000000-0000-4000-8000-000000000099';
SET LOCAL ROLE star_multica_run_owner;
DO $hidden$
BEGIN
    IF EXISTS (
        SELECT 1 FROM multica.run_infrastructure_binding_revision
         WHERE binding_id = '70000000-0000-4000-8000-000000000001'
    ) THEN
        RAISE EXCEPTION 'Run owner tenant RLS exposed another tenant binding';
    END IF;
END
$hidden$;
COMMIT;

DO $worm$
DECLARE
    blocked BOOLEAN;
BEGIN
    blocked := FALSE;
    BEGIN
        UPDATE multica.run_infrastructure_binding_revision
           SET binding_snapshot = '{}'::jsonb
         WHERE binding_id = '70000000-0000-4000-8000-000000000001';
    EXCEPTION WHEN OTHERS THEN
        blocked := TRUE;
    END;
    IF NOT blocked THEN RAISE EXCEPTION 'binding snapshot mutation unexpectedly succeeded'; END IF;

    blocked := FALSE;
    BEGIN
        DELETE FROM multica.run_infrastructure_binding_revision
         WHERE binding_id = '70000000-0000-4000-8000-000000000001';
    EXCEPTION WHEN OTHERS THEN
        blocked := TRUE;
    END;
    IF NOT blocked THEN RAISE EXCEPTION 'binding Master deletion unexpectedly succeeded'; END IF;

    blocked := FALSE;
    BEGIN
        UPDATE multica.run_infrastructure_binding_event SET event_type = 'created';
    EXCEPTION WHEN OTHERS THEN
        blocked := TRUE;
    END;
    IF NOT blocked THEN RAISE EXCEPTION 'binding event mutation unexpectedly succeeded'; END IF;

    blocked := FALSE;
    BEGIN
        TRUNCATE multica.run_infrastructure_binding_event;
    EXCEPTION WHEN OTHERS THEN
        blocked := TRUE;
    END;
    IF NOT blocked THEN RAISE EXCEPTION 'binding event truncation unexpectedly succeeded'; END IF;
END
$worm$;
