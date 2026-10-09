-- Native fixed-path canonical authorization read; no directory write capability.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-admission-currentness.sql"})
-- MERGE (verify:Symbol {id:"db/migrations/2026-10-10-run-owner-admission-currentness.sql::verify_schedule_run_as_current",kind:"function"})
-- MERGE (self)-[:DEFINES]->(verify)
-- MERGE (verify)-[:CALLS]->(lock:Symbol {id:"db/migrations/2026-10-05-schedule-run-as-acl-lock.sql::lock_schedule_run_as_authorization",kind:"function"})
-- MERGE (verify)-[:READS]->(project_grant:Table {id:"permission.project_role_binding"})
-- MERGE (verify)-[:READS]->(branch_grant:Table {id:"permission.cloud_branch_role_binding"})
-- MERGE (verify)-[:READS]->(run_grant:Table {id:"permission.engineering_run_role_binding"})
-- MERGE (verify)-[:READS]->(run:Table {id:"multica.engineering_run"})
-- MERGE (verify)-[:READS]->(run_revision:Table {id:"multica.engineering_run_revision"})
-- MERGE (verify)-[:READS]->(branch:Table {id:"scm.cloud_branch"})
-- MERGE (verify)-[:READS]->(branch_revision:Table {id:"scm.cloud_branch_revision"})
-- @endcypher
BEGIN;
CREATE OR REPLACE FUNCTION multica.verify_schedule_run_as_current(
    p_tenant UUID, p_actor UUID, p_project UUID, p_branch UUID, p_run UUID, p_repository UUID
) RETURNS BOOLEAN LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog AS $$
BEGIN
    IF p_tenant IS NULL OR p_actor IS NULL OR p_project IS NULL OR p_branch IS NULL
       OR p_run IS NULL OR p_repository IS NULL
       OR p_actor = '00000000-0000-0000-0000-000000000000'::UUID
       OR NULLIF(current_setting('app.tenant_id', true), '')::UUID IS DISTINCT FROM p_tenant
       OR NULLIF(current_setting('app.actor_id', true), '')::UUID IS DISTINCT FROM p_actor THEN
        RETURN FALSE;
    END IF;
    IF NOT multica.lock_schedule_run_as_authorization(p_tenant,p_actor,p_project,p_run) THEN
        RETURN FALSE;
    END IF;
    PERFORM r.engineering_run_id
      FROM multica.engineering_run r
      JOIN multica.engineering_run_revision rv
        ON rv.tenant_id=r.tenant_id AND rv.engineering_run_id=r.engineering_run_id
       AND rv.valid_from<=now() AND rv.valid_to IS NULL AND rv.state='active'
      JOIN scm.cloud_branch b
        ON b.tenant_id=r.tenant_id AND b.branch_id=r.branch_id
       AND b.project_id=r.project_id AND b.repository_id=r.repository_id
      JOIN scm.cloud_branch_revision bv
        ON bv.tenant_id=b.tenant_id AND bv.branch_id=b.branch_id
       AND bv.valid_from<=now() AND bv.valid_to IS NULL AND bv.state='active'
      JOIN permission.project_role_binding p
        ON p.tenant_id=r.tenant_id AND p.project_id=r.project_id AND p.user_id=p_actor
       AND p.valid_from<=now() AND p.valid_to IS NULL
       AND p.role IN ('tenant_admin','project_admin','developer')
      JOIN permission.cloud_branch_role_binding bg
        ON bg.tenant_id=b.tenant_id AND bg.branch_id=b.branch_id AND bg.user_id=p_actor
       AND bg.valid_from<=now() AND bg.valid_to IS NULL
      JOIN permission.engineering_run_role_binding rg
        ON rg.tenant_id=r.tenant_id AND rg.engineering_run_id=r.engineering_run_id AND rg.user_id=p_actor
       AND rg.valid_from<=now() AND rg.valid_to IS NULL
       AND rg.role IN ('tenant_admin','project_admin','developer')
      WHERE r.tenant_id=p_tenant AND r.project_id=p_project AND r.branch_id=p_branch
        AND r.engineering_run_id=p_run AND r.repository_id=p_repository
      FOR SHARE OF r,rv,b,bv,p,bg,rg;
    RETURN FOUND;
END $$;
REVOKE ALL ON FUNCTION multica.verify_schedule_run_as_current(UUID,UUID,UUID,UUID,UUID,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.verify_schedule_run_as_current(UUID,UUID,UUID,UUID,UUID,UUID)
 TO star_multica_run_owner;
COMMENT ON FUNCTION multica.verify_schedule_run_as_current(UUID,UUID,UUID,UUID,UUID,UUID) IS
 'Fixed scoped native read and transaction row locks; source service authentication, target/profile/hook/quota and execution checks are separate. Function owner/grants must be reviewed in target deployment.';
COMMIT;
