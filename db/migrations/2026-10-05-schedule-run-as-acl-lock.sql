-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-05-schedule-run-as-acl-lock.sql"})
-- MERGE (lockAuthorization:Function {id:"multica.lock_schedule_run_as_authorization"})
-- MERGE (lockMutation:Function {id:"multica.lock_schedule_acl_mutation"})
-- MERGE (projectGrant:Table {id:"permission.project_role_binding"})
-- MERGE (branchGrant:Table {id:"permission.cloud_branch_role_binding"})
-- MERGE (runGrant:Table {id:"permission.engineering_run_role_binding"})
-- MERGE (run:Table {id:"multica.engineering_run"})
-- MERGE (branch:Table {id:"scm.cloud_branch"})
-- MERGE (advisory:ExternalService {id:"postgres.pg_advisory_xact_lock"})
-- MERGE (projectTrigger:Trigger {id:"permission.project_role_binding.schedule_acl_authorization_lock"})
-- MERGE (branchTrigger:Trigger {id:"permission.cloud_branch_role_binding.schedule_acl_authorization_lock"})
-- MERGE (runTrigger:Trigger {id:"permission.engineering_run_role_binding.schedule_acl_authorization_lock"})
-- MERGE (self)-[:DEFINES]->(lockAuthorization)
-- MERGE (self)-[:DEFINES]->(lockMutation)
-- MERGE (self)-[:CONFIGURES]->(projectTrigger)
-- MERGE (self)-[:CONFIGURES]->(branchTrigger)
-- MERGE (self)-[:CONFIGURES]->(runTrigger)
-- MERGE (lockAuthorization)-[:READS]->(projectGrant)
-- MERGE (lockAuthorization)-[:READS]->(branchGrant)
-- MERGE (lockAuthorization)-[:READS]->(runGrant)
-- MERGE (lockAuthorization)-[:READS]->(run)
-- MERGE (lockAuthorization)-[:READS]->(branch)
-- MERGE (lockAuthorization)-[:CALLS]->(advisory)
-- MERGE (lockMutation)-[:CALLS]->(advisory)
-- MERGE (lockMutation)-[:READS]->(projectGrant)
-- MERGE (lockMutation)-[:READS]->(branchGrant)
-- MERGE (lockMutation)-[:READS]->(runGrant)
-- @endcypher

-- Serialize Schedule run-as admission with canonical Project/Branch/Run ACL mutations.
-- Project-scoped transaction advisory locks preserve parallelism across Projects while
-- keeping runtime authorization SELECT-only on the canonical Directory grant tables.

BEGIN;

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
        JOIN scm.cloud_branch b
          ON b.tenant_id = r.tenant_id AND b.branch_id = r.branch_id
         AND b.project_id = r.project_id AND b.repository_id = r.repository_id
        JOIN permission.project_role_binding p
          ON p.tenant_id = r.tenant_id AND p.project_id = r.project_id
         AND p.user_id = p_actor_id AND p.valid_from <= pg_catalog.now() AND p.valid_to IS NULL
        JOIN permission.cloud_branch_role_binding bg
          ON bg.tenant_id = b.tenant_id AND bg.branch_id = b.branch_id
         AND bg.user_id = p_actor_id AND bg.valid_from <= pg_catalog.now() AND bg.valid_to IS NULL
        JOIN permission.engineering_run_role_binding rg
          ON rg.tenant_id = r.tenant_id AND rg.engineering_run_id = r.engineering_run_id
         AND rg.user_id = p_actor_id AND rg.valid_from <= pg_catalog.now() AND rg.valid_to IS NULL
        WHERE r.tenant_id = p_tenant_id AND r.project_id = p_project_id
          AND r.engineering_run_id = p_engineering_run_id
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

CREATE OR REPLACE FUNCTION multica.lock_schedule_acl_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    old_row JSONB;
    new_row JSONB;
    v_tenant_id UUID;
    project_ids UUID[];
    project_id UUID;
    project_lock_id BIGINT;
BEGIN
    old_row := CASE WHEN TG_OP IN ('UPDATE', 'DELETE') THEN pg_catalog.to_jsonb(OLD) ELSE '{}'::JSONB END;
    new_row := CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN pg_catalog.to_jsonb(NEW) ELSE '{}'::JSONB END;
    v_tenant_id := COALESCE(NULLIF(new_row ->> 'tenant_id', '')::UUID, NULLIF(old_row ->> 'tenant_id', '')::UUID);

    IF v_tenant_id IS NULL THEN
        RAISE EXCEPTION 'Schedule ACL mutation requires a tenant scope';
    END IF;

    IF TG_TABLE_SCHEMA = 'permission' AND TG_TABLE_NAME = 'project_role_binding' THEN
        SELECT pg_catalog.array_agg(scope.project_id ORDER BY scope.project_id)
          INTO project_ids
          FROM (
              SELECT DISTINCT value::UUID AS project_id
              FROM pg_catalog.unnest(ARRAY[
                  NULLIF(old_row ->> 'project_id', '')::UUID,
                  NULLIF(new_row ->> 'project_id', '')::UUID
              ]) AS ids(value)
              WHERE value IS NOT NULL
          ) scope;
    ELSIF TG_TABLE_SCHEMA = 'permission' AND TG_TABLE_NAME = 'cloud_branch_role_binding' THEN
        SELECT pg_catalog.array_agg(scope.project_id ORDER BY scope.project_id)
          INTO project_ids
          FROM (
              SELECT DISTINCT b.project_id
              FROM scm.cloud_branch b
              WHERE b.tenant_id = v_tenant_id
                AND b.branch_id IN (
                    NULLIF(old_row ->> 'branch_id', '')::UUID,
                    NULLIF(new_row ->> 'branch_id', '')::UUID
                )
          ) scope;
    ELSIF TG_TABLE_SCHEMA = 'permission' AND TG_TABLE_NAME = 'engineering_run_role_binding' THEN
        SELECT pg_catalog.array_agg(scope.project_id ORDER BY scope.project_id)
          INTO project_ids
          FROM (
              SELECT DISTINCT r.project_id
              FROM multica.engineering_run r
              WHERE r.tenant_id = v_tenant_id
                AND r.engineering_run_id IN (
                    NULLIF(old_row ->> 'engineering_run_id', '')::UUID,
                    NULLIF(new_row ->> 'engineering_run_id', '')::UUID
                )
          ) scope;
    ELSE
        RAISE EXCEPTION 'Schedule ACL lock trigger was attached to an unsupported table';
    END IF;

    IF COALESCE(pg_catalog.cardinality(project_ids), 0) = 0 THEN
        RAISE EXCEPTION 'Schedule ACL mutation could not resolve its canonical Project';
    END IF;

    FOREACH project_id IN ARRAY project_ids LOOP
        project_lock_id := pg_catalog.hashtextextended(
            'star.schedule.run-as.authorization:project:' || v_tenant_id::TEXT || ':' || project_id::TEXT,
            741391
        );
        PERFORM pg_catalog.pg_advisory_xact_lock(project_lock_id);
    END LOOP;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;

REVOKE ALL ON FUNCTION multica.lock_schedule_run_as_authorization(UUID, UUID, UUID, UUID) FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.lock_schedule_acl_mutation() FROM PUBLIC;

DROP TRIGGER IF EXISTS schedule_acl_authorization_lock ON permission.project_role_binding;
CREATE TRIGGER schedule_acl_authorization_lock
    BEFORE INSERT OR UPDATE OR DELETE ON permission.project_role_binding
    FOR EACH ROW EXECUTE FUNCTION multica.lock_schedule_acl_mutation();

DROP TRIGGER IF EXISTS schedule_acl_authorization_lock ON permission.cloud_branch_role_binding;
CREATE TRIGGER schedule_acl_authorization_lock
    BEFORE INSERT OR UPDATE OR DELETE ON permission.cloud_branch_role_binding
    FOR EACH ROW EXECUTE FUNCTION multica.lock_schedule_acl_mutation();

DROP TRIGGER IF EXISTS schedule_acl_authorization_lock ON permission.engineering_run_role_binding;
CREATE TRIGGER schedule_acl_authorization_lock
    BEFORE INSERT OR UPDATE OR DELETE ON permission.engineering_run_role_binding
    FOR EACH ROW EXECUTE FUNCTION multica.lock_schedule_acl_mutation();

COMMIT;
