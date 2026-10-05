-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-04-schedule-run-as-actor.sql"})
-- MERGE (rule:Table {id:"automation.schedule_rule_revision"})
-- MERGE (run_as:Column {id:"automation.schedule_rule_revision.run_as_actor_id"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (occurrence_run_as:Column {id:"automation.occurrence.run_as_actor_id"})
-- MERGE (guard:Symbol {id:"automation.guard_schedule_rule_run_as_identity",kind:"function"})
-- MERGE (occurrence_guard:Symbol {id:"automation.guard_schedule_occurrence_run_as_identity",kind:"function"})
-- MERGE (self)-[:ALTERS]->(rule)
-- MERGE (self)-[:DEFINES]->(run_as)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (rule)-[:HAS_COLUMN]->(run_as)
-- MERGE (self)-[:ALTERS]->(occurrence)
-- MERGE (occurrence)-[:HAS_COLUMN]->(occurrence_run_as)
-- MERGE (self)-[:CONFIGURES]->(guard)
-- MERGE (self)-[:CONFIGURES]->(occurrence_guard)
-- @endcypher

-- Phase 9F4B: persist the immutable creator/run-as principal on every Rule revision.
-- This records identity; each unattended trigger must still reauthorize it against current ACLs.
-- No scheduler worker or Run admission is enabled by this migration.

BEGIN;

ALTER TABLE automation.schedule_rule_revision
    ADD COLUMN IF NOT EXISTS run_as_actor_id UUID;

-- Backfill under the migration transaction's table lock as the schema owner. The owner may
-- bypass RLS while FORCE is temporarily removed; app roles retain tenant RLS. Disable only the
-- close-only guard that correctly rejects ordinary row updates.
ALTER TABLE automation.schedule_rule_revision NO FORCE ROW LEVEL SECURITY;
ALTER TABLE automation.schedule_rule_revision
    DISABLE TRIGGER automation_schedule_rule_scd2_guard;

-- Existing revisions may have different editors. The first revision's author is the stable creator.
WITH first_creator AS (
    SELECT DISTINCT ON (tenant_id, project_id, rule_id)
           tenant_id, project_id, rule_id, changed_by AS actor_id
    FROM automation.schedule_rule_revision
    ORDER BY tenant_id, project_id, rule_id, rule_version, valid_from, created_at
)
UPDATE automation.schedule_rule_revision revision
SET run_as_actor_id = first_creator.actor_id
FROM first_creator
WHERE revision.tenant_id = first_creator.tenant_id
  AND revision.project_id = first_creator.project_id
  AND revision.rule_id = first_creator.rule_id
  AND revision.run_as_actor_id IS NULL;

ALTER TABLE automation.schedule_rule_revision
    ALTER COLUMN run_as_actor_id SET NOT NULL;

ALTER TABLE automation.schedule_rule_revision
    ENABLE TRIGGER automation_schedule_rule_scd2_guard;
ALTER TABLE automation.schedule_rule_revision FORCE ROW LEVEL SECURITY;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'schedule_rule_run_as_actor_non_nil'
          AND conrelid = 'automation.schedule_rule_revision'::regclass
    ) THEN
        ALTER TABLE automation.schedule_rule_revision
            ADD CONSTRAINT schedule_rule_run_as_actor_non_nil
            CHECK (run_as_actor_id <> '00000000-0000-0000-0000-000000000000'::UUID);
    END IF;
END $$;

CREATE OR REPLACE FUNCTION automation.guard_schedule_rule_run_as_identity()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    first_actor UUID;
BEGIN
    -- Serialize inserts for one Rule so two concurrent revisions cannot race the identity check.
    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_id::text || ':' || NEW.project_id::text || ':' || NEW.rule_id::text, 9004)
    );
    SELECT revision.run_as_actor_id INTO first_actor
    FROM automation.schedule_rule_revision revision
    WHERE revision.tenant_id = NEW.tenant_id
      AND revision.project_id = NEW.project_id
      AND revision.rule_id = NEW.rule_id
    ORDER BY revision.rule_version
    LIMIT 1;
    IF NOT FOUND AND NEW.run_as_actor_id <> NEW.changed_by THEN
        RAISE EXCEPTION 'initial automation schedule run-as identity must match its creator'
            USING ERRCODE = '23514';
    END IF;
    IF FOUND AND first_actor <> NEW.run_as_actor_id THEN
        RAISE EXCEPTION 'automation schedule run-as identity is immutable'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS automation_schedule_rule_run_as_identity_guard
    ON automation.schedule_rule_revision;
CREATE TRIGGER automation_schedule_rule_run_as_identity_guard
    BEFORE INSERT ON automation.schedule_rule_revision
    FOR EACH ROW EXECUTE FUNCTION automation.guard_schedule_rule_run_as_identity();

ALTER TABLE automation.occurrence
    ADD COLUMN IF NOT EXISTS run_as_actor_id UUID;

-- The occurrence table is append-only for application sessions; allow only this migration's
-- snapshot backfill while its DDL lock is held, then restore the guard and FORCE RLS below.
ALTER TABLE automation.occurrence NO FORCE ROW LEVEL SECURITY;
ALTER TABLE automation.occurrence
    DISABLE TRIGGER automation_occurrence_no_mutation;

UPDATE automation.occurrence occurrence
SET run_as_actor_id = revision.run_as_actor_id
FROM automation.schedule_rule_revision revision
WHERE occurrence.tenant_id = revision.tenant_id
  AND occurrence.project_id = revision.project_id
  AND occurrence.rule_id = revision.rule_id
  AND occurrence.rule_version = revision.rule_version
  AND occurrence.run_as_actor_id IS NULL;

ALTER TABLE automation.occurrence
    ALTER COLUMN run_as_actor_id SET NOT NULL;

ALTER TABLE automation.occurrence
    ENABLE TRIGGER automation_occurrence_no_mutation;
ALTER TABLE automation.occurrence FORCE ROW LEVEL SECURITY;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'occurrence_run_as_actor_non_nil'
          AND conrelid = 'automation.occurrence'::regclass
    ) THEN
        ALTER TABLE automation.occurrence
            ADD CONSTRAINT occurrence_run_as_actor_non_nil
            CHECK (run_as_actor_id <> '00000000-0000-0000-0000-000000000000'::UUID);
    END IF;
END $$;

CREATE OR REPLACE FUNCTION automation.guard_schedule_occurrence_run_as_identity()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    pinned_actor UUID;
BEGIN
    SELECT revision.run_as_actor_id INTO pinned_actor
    FROM automation.schedule_rule_revision revision
    WHERE revision.tenant_id = NEW.tenant_id
      AND revision.project_id = NEW.project_id
      AND revision.rule_id = NEW.rule_id
      AND revision.rule_version = NEW.rule_version;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'schedule occurrence has no pinned Rule revision'
            USING ERRCODE = '23503';
    END IF;
    IF NEW.run_as_actor_id IS DISTINCT FROM pinned_actor THEN
        RAISE EXCEPTION 'occurrence run-as identity differs from pinned Rule revision'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS automation_schedule_occurrence_run_as_identity_guard
    ON automation.occurrence;
CREATE TRIGGER automation_schedule_occurrence_run_as_identity_guard
    BEFORE INSERT ON automation.occurrence
    FOR EACH ROW EXECUTE FUNCTION automation.guard_schedule_occurrence_run_as_identity();

COMMENT ON COLUMN automation.schedule_rule_revision.run_as_actor_id IS
    'Immutable creator principal for unattended execution; reauthorize current Project and Run access at each trigger.';
COMMENT ON COLUMN automation.occurrence.run_as_actor_id IS
    'Immutable run-as principal copied from the exact Rule revision pinned by this occurrence.';

COMMIT;
