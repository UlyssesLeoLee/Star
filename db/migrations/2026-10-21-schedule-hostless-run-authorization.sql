-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-21-schedule-hostless-run-authorization.sql"})
-- MERGE (rules:Table {id:"automation.schedule_rule_revision"})
-- MERGE (rule_branch:Column {id:"automation.schedule_rule_revision.branch_id"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (occurrence_snapshot:Column {id:"automation.occurrence.target_snapshot"})
-- MERGE (lock:Function {id:"multica.lock_schedule_run_as_authorization"})
-- MERGE (run:Table {id:"multica.engineering_run"})
-- MERGE (project_grants:Table {id:"permission.project_role_binding"})
-- MERGE (branch:Table {id:"scm.cloud_branch"})
-- MERGE (branch_grants:Table {id:"permission.cloud_branch_role_binding"})
-- MERGE (run_grants:Table {id:"permission.engineering_run_role_binding"})
-- MERGE (self)-[:ALTERS]->(rules)
-- MERGE (self)-[:ALTERS_NULLABILITY]->(rule_branch)
-- MERGE (self)-[:PRESERVES]->(occurrence)
-- MERGE (occurrence)-[:CONTAINS]->(occurrence_snapshot)
-- MERGE (self)-[:REPLACES]->(lock)
-- MERGE (lock)-[:READS]->(run)
-- MERGE (lock)-[:READS]->(project_grants)
-- MERGE (lock)-[:READS]->(branch)
-- MERGE (lock)-[:READS]->(branch_grants)
-- MERGE (lock)-[:READS]->(run_grants)
-- @endcypher

-- Schedule Rule configuration may omit remote Branch context for a hostless Engineering Run.
-- Occurrence, admission, and execution contracts remain Branch-required and fail closed.
BEGIN;

ALTER TABLE automation.schedule_rule_revision
    ALTER COLUMN branch_id DROP NOT NULL;

COMMENT ON COLUMN automation.schedule_rule_revision.branch_id IS
    'Optional remote Branch context for a configured Schedule Rule; occurrence materialization and execution remain Branch-required.';

CREATE OR REPLACE FUNCTION multica.lock_schedule_run_as_authorization(
    p_tenant_id UUID,
    p_actor_id UUID,
    p_project_id UUID,
    p_engineering_run_id UUID
)
RETURNS BOOLEAN
LANGUAGE plpgsql
VOLATILE
SET search_path = pg_catalog
AS $$
DECLARE
    project_lock_id BIGINT;
BEGIN
    IF p_tenant_id IS NULL OR p_actor_id IS NULL OR p_project_id IS NULL
       OR p_engineering_run_id IS NULL
       OR NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::UUID IS DISTINCT FROM p_tenant_id
       OR NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::UUID IS DISTINCT FROM p_actor_id THEN
        RETURN FALSE;
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM multica.engineering_run r
        JOIN permission.project_role_binding p
          ON p.tenant_id = r.tenant_id AND p.project_id = r.project_id
         AND p.user_id = p_actor_id AND p.valid_from <= pg_catalog.now() AND p.valid_to IS NULL
        LEFT JOIN scm.cloud_branch b
          ON r.branch_id IS NOT NULL
         AND b.tenant_id = r.tenant_id AND b.branch_id = r.branch_id
         AND b.project_id = r.project_id AND b.repository_id = r.repository_id
        LEFT JOIN permission.cloud_branch_role_binding bg
          ON bg.tenant_id = b.tenant_id AND bg.branch_id = b.branch_id
         AND bg.user_id = p_actor_id AND bg.valid_from <= pg_catalog.now() AND bg.valid_to IS NULL
        JOIN permission.engineering_run_role_binding rg
          ON rg.tenant_id = r.tenant_id AND rg.engineering_run_id = r.engineering_run_id
         AND rg.user_id = p_actor_id AND rg.valid_from <= pg_catalog.now() AND rg.valid_to IS NULL
        WHERE r.tenant_id = p_tenant_id AND r.project_id = p_project_id
          AND r.engineering_run_id = p_engineering_run_id
          AND (r.branch_id IS NULL OR (b.branch_id IS NOT NULL AND bg.binding_id IS NOT NULL))
    ) THEN
        RETURN FALSE;
    END IF;

    project_lock_id := pg_catalog.hashtextextended(
        'star.schedule.run-as.authorization:project:' || p_tenant_id::TEXT || ':' || p_project_id::TEXT,
        741391
    );
    PERFORM pg_catalog.pg_advisory_xact_lock(project_lock_id);
    RETURN TRUE;
END;
$$;

REVOKE ALL ON FUNCTION multica.lock_schedule_run_as_authorization(UUID, UUID, UUID, UUID) FROM PUBLIC;

COMMIT;
