-- @cypher schema=1 source_sha256=41549fa5fc7000886a8042eb16c8cad8f7236af207f9b857081816af1e014eb0
-- MERGE (self:File {path:"db/migrations/2026-10-22-schedule-occurrence-branch-snapshot.sql"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (target_snapshot:Column {id:"automation.occurrence.target_snapshot"})
-- MERGE (branch_check:Config {id:"automation.occurrence.target_snapshot_branch_uuid_check"})
-- MERGE (self)-[:ALTERS]->(occurrence)
-- MERGE (occurrence)-[:HAS_COLUMN]->(target_snapshot)
-- MERGE (branch_check)-[:READS]->(target_snapshot)
-- MERGE (self)-[:CONFIGURES]->(branch_check)
-- @endcypher

-- Schedule Rules may be configured without a remote Branch. A materialized occurrence
-- must pin a non-nil Branch UUID in standard hyphenated form inside its immutable snapshot.
-- NOT VALID preserves legacy rows while enforcing this check for every new or changed row.
BEGIN;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'automation.occurrence'::regclass
          AND conname = 'occurrence_target_snapshot_branch_uuid_check'
    ) THEN
        ALTER TABLE automation.occurrence
            ADD CONSTRAINT occurrence_target_snapshot_branch_uuid_check
            CHECK (
                CASE
                    WHEN jsonb_typeof(target_snapshot #> '{rule_snapshot,target,branch_id}') = 'string'
                     AND target_snapshot #>> '{rule_snapshot,target,branch_id}'
                         ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
                    THEN (target_snapshot #>> '{rule_snapshot,target,branch_id}')::uuid
                         <> '00000000-0000-0000-0000-000000000000'::uuid
                    ELSE FALSE
                END
            ) NOT VALID;
    END IF;
END;
$$;

COMMENT ON CONSTRAINT occurrence_target_snapshot_branch_uuid_check
    ON automation.occurrence IS
    'New occurrence snapshots must contain a non-nil, case-insensitive hyphenated UUID at rule_snapshot.target.branch_id; legacy rows are not backfilled.';

COMMIT;
