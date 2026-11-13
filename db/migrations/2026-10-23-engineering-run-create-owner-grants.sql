-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-23-engineering-run-create-owner-grants.sql"})
-- MERGE (role:Config {id:"database-role:star_engineering_directory_owner",kind:"postgres_role"})
-- MERGE (run:Table {id:"multica.engineering_run",kind:"table"})
-- MERGE (revision:Table {id:"multica.engineering_run_revision",kind:"table"})
-- MERGE (grant:Table {id:"permission.engineering_run_role_binding",kind:"table"})
-- MERGE (self)-[:CONFIGURES]->(role)
-- MERGE (self)-[:GRANTS]->(run)
-- MERGE (self)-[:GRANTS]->(revision)
-- MERGE (self)-[:GRANTS]->(grant)
-- @endcypher

-- Hostless Run creation is owner-local: grant INSERT only to its three immutable/SCD2 facts.
BEGIN;

GRANT INSERT ON TABLE
    multica.engineering_run,
    multica.engineering_run_revision,
    permission.engineering_run_role_binding
TO star_engineering_directory_owner;

COMMENT ON ROLE star_engineering_directory_owner IS
    'NOLOGIN/NOSUPERUSER/NOBYPASSRLS/NOINHERIT owner role for hostless Engineering Run registration and Run-to-Worktree binding mutations; API login membership is provisioned per deployment and selected with SET LOCAL ROLE.';

COMMIT;
