-- @cypher schema=1 source_sha256=6803f8804ee597b24deed51a64eaa3e82bc60b0931320118f21a18fa1f170075
-- MERGE (self:File {path:"db/migrations/2026-10-23-schedule-occurrence-branch-snapshot-correction.sql"})
-- MERGE (legacy:File {path:"db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (target_snapshot:Column {id:"automation.occurrence.target_snapshot"})
-- MERGE (branch_check:Config {id:"automation.occurrence.target_snapshot_branch_uuid_check"})
-- MERGE (self)-[:FOLLOWS]->(legacy)
-- MERGE (self)-[:ALTERS]->(occurrence)
-- MERGE (occurrence)-[:HAS_COLUMN]->(target_snapshot)
-- MERGE (branch_check)-[:READS]->(target_snapshot)
-- MERGE (self)-[:CONFIGURES]->(branch_check)
-- @endcypher

-- The 2026-10-22 migration is immutable history and may already be recorded by
-- schema_migrations. Replace only its named CHECK so new occurrence snapshots use
-- the flattened ScheduleTargetV1 JSON object stored at target_snapshot.
BEGIN;

ALTER TABLE automation.occurrence
    DROP CONSTRAINT IF EXISTS occurrence_target_snapshot_branch_uuid_check;

ALTER TABLE automation.occurrence
    ADD CONSTRAINT occurrence_target_snapshot_branch_uuid_check
    CHECK (
        CASE
            WHEN jsonb_typeof(target_snapshot #> '{branch_id}') = 'string'
             AND target_snapshot #>> '{branch_id}'
                 ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
            THEN (target_snapshot #>> '{branch_id}')::uuid
                 <> '00000000-0000-0000-0000-000000000000'::uuid
            ELSE FALSE
        END
    ) NOT VALID;

COMMENT ON CONSTRAINT occurrence_target_snapshot_branch_uuid_check
    ON automation.occurrence IS
    'New occurrence target snapshots must contain a non-nil, case-insensitive hyphenated UUID at branch_id; legacy rows are not backfilled.';

COMMIT;
