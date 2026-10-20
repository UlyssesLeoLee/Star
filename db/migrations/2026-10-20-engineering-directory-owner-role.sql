-- CYPHER STRUCTURAL MANIFEST
-- CREATE (f:File {name:"2026-10-20-engineering-directory-owner-role.sql",type:"file",language:"sql"}),
--        (owner:DatabaseRole {name:"star_engineering_directory_owner",type:"database_role"}),
--        (binding:Table {name:"multica.engineering_run_worktree_binding",type:"table"}),
--        (idempotency:Table {name:"multica.engineering_run_command_idempotency",type:"table"}),
--        (outbox:Table {name:"multica.engineering_run_outbox",type:"table"}),
--        (runRevision:Table {name:"multica.engineering_run_revision",type:"table"}),
--        (taskMetadata:Table {name:"multica.task_metadata",type:"table"}),
--        (taskWorktree:Table {name:"multica.work_item_worktree",type:"table"});
-- CREATE (f)-[:CREATES]->(owner),(f)-[:GRANTS]->(binding),(f)-[:GRANTS]->(idempotency),(f)-[:GRANTS]->(outbox),
--        (f)-[:GRANTS]->(runRevision),(f)-[:READS]->(taskMetadata),(f)-[:READS]->(taskWorktree);
-- MATCH (f:File {name:"2026-10-20-engineering-directory-owner-role.sql"}),(owner:DatabaseRole {name:"star_engineering_directory_owner"})
-- CREATE (f)-[:CONFIGURES]->(owner);

-- Create the non-login owner role used only inside Run-to-Worktree binding transactions.
-- The API login must receive membership through deployment-specific role provisioning.
BEGIN;

DO $role$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_engineering_directory_owner') THEN
        EXECUTE 'ALTER ROLE star_engineering_directory_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_engineering_directory_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
END
$role$;

-- Normalize a pre-existing dedicated role before granting its narrow contract.
-- API logins that are members of this role are intentionally preserved.
REVOKE ALL PRIVILEGES ON SCHEMA public, scm, permission, multica
    FROM star_engineering_directory_owner;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public, scm, permission, multica
    FROM star_engineering_directory_owner;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public, scm, permission, multica
    FROM star_engineering_directory_owner;
REVOKE ALL PRIVILEGES ON ALL ROUTINES IN SCHEMA public, scm, permission, multica
    FROM star_engineering_directory_owner;
DO $database_privileges$
DECLARE
    database_name NAME;
BEGIN
    FOR database_name IN SELECT datname FROM pg_database WHERE datallowconn LOOP
        EXECUTE format(
            'REVOKE CONNECT, CREATE, TEMPORARY ON DATABASE %I FROM star_engineering_directory_owner',
            database_name
        );
    END LOOP;
END
$database_privileges$;

DO $membership$
DECLARE
    granted_role NAME;
BEGIN
    FOR granted_role IN
        SELECT granted.rolname
        FROM pg_auth_members membership
        JOIN pg_roles granted ON granted.oid = membership.roleid
        WHERE membership.member = 'star_engineering_directory_owner'::regrole
    LOOP
        EXECUTE format('REVOKE %I FROM star_engineering_directory_owner', granted_role);
    END LOOP;
END
$membership$;

DO $ownership$
DECLARE
    owner_oid OID := 'star_engineering_directory_owner'::regrole;
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_shdepend
        WHERE refclassid = 'pg_authid'::regclass
          AND refobjid = owner_oid
          AND deptype = 'o'
    ) THEN
        RAISE EXCEPTION 'star_engineering_directory_owner must not own database objects';
    END IF;
END
$ownership$;

GRANT USAGE ON SCHEMA public, scm, permission, multica TO star_engineering_directory_owner;

-- Current authorization and source facts are read in the same mutation transaction.
GRANT SELECT ON TABLE
    permission.project_role_binding,
    scm.cloud_branch,
    scm.cloud_branch_revision,
    permission.cloud_branch_role_binding,
    multica.engineering_run,
    multica.engineering_run_revision,
    permission.engineering_run_role_binding,
    multica.worktree_project_binding,
    public.worktree_canvas_worktree,
    multica.task_metadata,
    multica.work_item_worktree
TO star_engineering_directory_owner;

-- PostgreSQL requires UPDATE on at least one column to take SELECT FOR SHARE locks.
-- These narrow column grants are lock-only: restrictive RLS policies below deny actual
-- Run Directory UPDATEs to Project, Branch, Run, Worktree and Task-owned source rows.
GRANT UPDATE (valid_to) ON TABLE permission.project_role_binding,
    scm.cloud_branch_revision,
    permission.cloud_branch_role_binding,
    permission.engineering_run_role_binding,
    multica.engineering_run_revision,
    multica.worktree_project_binding,
    multica.task_metadata,
    multica.work_item_worktree
TO star_engineering_directory_owner;
GRANT UPDATE (id) ON TABLE public.worktree_canvas_worktree TO star_engineering_directory_owner;

DROP POLICY IF EXISTS engineering_directory_owner_no_project_grant_update
    ON permission.project_role_binding;
CREATE POLICY engineering_directory_owner_no_project_grant_update
    ON permission.project_role_binding AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_branch_revision_update
    ON scm.cloud_branch_revision;
CREATE POLICY engineering_directory_owner_no_branch_revision_update
    ON scm.cloud_branch_revision AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_branch_grant_update
    ON permission.cloud_branch_role_binding;
CREATE POLICY engineering_directory_owner_no_branch_grant_update
    ON permission.cloud_branch_role_binding AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_run_grant_update
    ON permission.engineering_run_role_binding;
CREATE POLICY engineering_directory_owner_no_run_grant_update
    ON permission.engineering_run_role_binding AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_run_revision_update
    ON multica.engineering_run_revision;
CREATE POLICY engineering_directory_owner_no_run_revision_update
    ON multica.engineering_run_revision AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_worktree_binding_update
    ON multica.worktree_project_binding;
CREATE POLICY engineering_directory_owner_no_worktree_binding_update
    ON multica.worktree_project_binding AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_worktree_projection_update
    ON public.worktree_canvas_worktree;
CREATE POLICY engineering_directory_owner_no_worktree_projection_update
    ON public.worktree_canvas_worktree AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_task_metadata_update
    ON multica.task_metadata;
CREATE POLICY engineering_directory_owner_no_task_metadata_update
    ON multica.task_metadata AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

DROP POLICY IF EXISTS engineering_directory_owner_no_task_worktree_update
    ON multica.work_item_worktree;
CREATE POLICY engineering_directory_owner_no_task_worktree_update
    ON multica.work_item_worktree AS RESTRICTIVE FOR UPDATE
    TO star_engineering_directory_owner USING (false) WITH CHECK (false);

-- Run Directory-owned SCD2 facts: insert new binding revisions and close only valid_to.
GRANT SELECT, INSERT ON TABLE multica.engineering_run_worktree_binding
    TO star_engineering_directory_owner;
GRANT UPDATE (valid_to) ON TABLE multica.engineering_run_worktree_binding
    TO star_engineering_directory_owner;
GRANT INSERT ON TABLE multica.engineering_directory_event
    TO star_engineering_directory_owner;

-- Actor-scoped idempotency supports bounded stale-receipt deletion and SKIP LOCKED cleanup.
GRANT SELECT, INSERT, DELETE ON TABLE multica.engineering_run_command_idempotency
    TO star_engineering_directory_owner;
GRANT UPDATE (expires_at) ON TABLE multica.engineering_run_command_idempotency
    TO star_engineering_directory_owner;

-- API writes only to its own transactional Outbox; delivery/read ownership stays separate.
GRANT INSERT ON TABLE multica.engineering_run_outbox
    TO star_engineering_directory_owner;

GRANT EXECUTE ON FUNCTION multica.engineering_directory_scope(TEXT, TEXT, JSONB)
    TO star_engineering_directory_owner;
GRANT EXECUTE ON FUNCTION multica.guard_engineering_directory_write()
    TO star_engineering_directory_owner;
GRANT EXECUTE ON FUNCTION multica.audit_engineering_directory_write()
    TO star_engineering_directory_owner;
GRANT EXECUTE ON FUNCTION multica.guard_engineering_run_worktree_project_run_owner()
    TO star_engineering_directory_owner;

COMMENT ON ROLE star_engineering_directory_owner IS
    'NOLOGIN/NOBYPASSRLS owner role for Engineering Run to Worktree binding mutations; API login membership is provisioned per deployment and selected with SET LOCAL ROLE.';

COMMIT;
