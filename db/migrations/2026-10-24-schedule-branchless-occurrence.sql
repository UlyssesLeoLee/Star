-- @cypher schema=1 source_sha256=02846dd148aa4dd6f5b696542223e7036fd6b34e841d20fd3a923098e413480a
-- MERGE (self:File {path:"db/migrations/2026-10-24-schedule-branchless-occurrence.sql"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (target_snapshot:Column {id:"automation.occurrence.target_snapshot"})
-- MERGE (branch_check:Config {id:"automation.occurrence.target_snapshot_branch_uuid_check"})
-- MERGE (identity_check:Config {id:"automation.occurrence.target_snapshot_identity_check"})
-- MERGE (self)-[:ALTERS]->(occurrence)
-- MERGE (occurrence)-[:HAS_COLUMN]->(target_snapshot)
-- MERGE (branch_check)-[:READS]->(target_snapshot)
-- MERGE (identity_check)-[:READS]->(target_snapshot)
-- MERGE (self)-[:CONFIGURES]->(branch_check)
-- MERGE (self)-[:CONFIGURES]->(identity_check)
-- @endcypher

-- Historical occurrence snapshot migrations remain immutable. Replace their remote-Branch-only
-- CHECK with an optional metadata check and add checks for the canonical local execution target.
-- NOT VALID preserves existing rows while enforcing the contract on every new or updated row.
BEGIN;

ALTER TABLE automation.occurrence
    DROP CONSTRAINT IF EXISTS occurrence_target_snapshot_branch_uuid_check;

ALTER TABLE automation.occurrence
    ADD CONSTRAINT occurrence_target_snapshot_branch_uuid_check
    CHECK (
        CASE
            WHEN NOT (target_snapshot ? 'branch_id') THEN TRUE
            WHEN jsonb_typeof(target_snapshot -> 'branch_id') = 'null' THEN TRUE
            WHEN jsonb_typeof(target_snapshot -> 'branch_id') = 'string'
             AND target_snapshot ->> 'branch_id'
                 ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
             AND lower(target_snapshot ->> 'branch_id')
                 <> '00000000-0000-0000-0000-000000000000'
            THEN TRUE
            ELSE FALSE
        END
    ) NOT VALID;

ALTER TABLE automation.occurrence
    DROP CONSTRAINT IF EXISTS occurrence_target_snapshot_identity_check;

ALTER TABLE automation.occurrence
    ADD CONSTRAINT occurrence_target_snapshot_identity_check
    CHECK (
        (
        jsonb_typeof(target_snapshot) = 'object'
        AND jsonb_typeof(target_snapshot -> 'project_id') = 'string'
        AND target_snapshot ->> 'project_id' ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        AND lower(target_snapshot ->> 'project_id') <> '00000000-0000-0000-0000-000000000000'
        AND jsonb_typeof(target_snapshot -> 'engineering_run_id') = 'string'
        AND target_snapshot ->> 'engineering_run_id' ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        AND lower(target_snapshot ->> 'engineering_run_id') <> '00000000-0000-0000-0000-000000000000'
        AND jsonb_typeof(target_snapshot -> 'repository_id') = 'string'
        AND target_snapshot ->> 'repository_id' ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        AND lower(target_snapshot ->> 'repository_id') <> '00000000-0000-0000-0000-000000000000'
        AND jsonb_typeof(target_snapshot -> 'worktree_id') = 'string'
        AND target_snapshot ->> 'worktree_id' ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        AND lower(target_snapshot ->> 'worktree_id') <> '00000000-0000-0000-0000-000000000000'
        AND jsonb_typeof(target_snapshot -> 'work_item_id') = 'string'
        AND target_snapshot ->> 'work_item_id' ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        AND lower(target_snapshot ->> 'work_item_id') <> '00000000-0000-0000-0000-000000000000'
        AND jsonb_typeof(target_snapshot -> 'execution_profile_id') = 'string'
        AND target_snapshot ->> 'execution_profile_id' ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        AND lower(target_snapshot ->> 'execution_profile_id') <> '00000000-0000-0000-0000-000000000000'
        AND jsonb_typeof(target_snapshot -> 'execution_profile_version') = 'number'
        AND target_snapshot ->> 'execution_profile_version' ~ '^[1-9][0-9]*$'
        AND jsonb_typeof(target_snapshot -> 'execution_profile_digest') = 'string'
        AND target_snapshot ->> 'execution_profile_digest' ~ '^[0-9a-f]{64}$'
        AND jsonb_typeof(target_snapshot -> 'hook_set_version') = 'number'
        AND target_snapshot ->> 'hook_set_version' ~ '^[1-9][0-9]*$'
        AND jsonb_typeof(target_snapshot -> 'hook_set_digest') = 'string'
        AND target_snapshot ->> 'hook_set_digest' ~ '^[0-9a-f]{64}$'
    ) IS TRUE
    ) NOT VALID;

COMMENT ON CONSTRAINT occurrence_target_snapshot_branch_uuid_check
    ON automation.occurrence IS
    'New occurrence target snapshots may omit remote branch_id or contain JSON null; a present remote Branch must be a non-nil hyphenated UUID.';

COMMENT ON CONSTRAINT occurrence_target_snapshot_identity_check
    ON automation.occurrence IS
    'New occurrence target snapshots must preserve non-nil Project, Run, Repository, Worktree, Task, and Profile identities plus pinned positive Profile/HookSet versions and lowercase SHA-256 digests.';

COMMIT;
