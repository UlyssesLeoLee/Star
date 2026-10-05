-- @cypher schema=1 source_sha256=6c027d2b383d915b1fda965b30fddbc9dfb2df77250c0728019bd57bbacbffe7
-- MERGE (self:File {path:"db/migrations/2026-10-04-schedule-run-admission.sql"})
-- MERGE (dispatch:Table {id:"automation.occurrence_dispatch"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (taskRun:Table {id:"multica.task_execution_run"})
-- MERGE (outbox:Table {id:"automation.schedule_run_outbox"})
-- MERGE (dispatchGuard:Function {id:"automation.guard_occurrence_dispatch"})
-- MERGE (outboxGuard:Function {id:"automation.guard_schedule_run_outbox"})
-- MERGE (historyGuard:Function {id:"automation.reject_schedule_history_mutation"})
-- MERGE (self)-[:ALTERS]->(dispatch)
-- MERGE (self)-[:READS]->(occurrence)
-- MERGE (dispatchGuard)-[:READS]->(dispatch)
-- MERGE (dispatchGuard)-[:READS]->(taskRun)
-- MERGE (self)-[:DEFINES]->(outbox)
-- MERGE (self)-[:DEFINES]->(dispatchGuard)
-- MERGE (self)-[:DEFINES]->(outboxGuard)
-- MERGE (self)-[:USES]->(historyGuard)
-- MERGE (outboxGuard)-[:READS]->(outbox)
-- MERGE (outboxGuard)-[:READS]->(occurrence)
-- MERGE (outboxGuard)-[:READS]->(dispatch)
-- MERGE (outboxGuard)-[:READS]->(taskRun)
-- @endcypher

-- Phase 9F4C persistence contract for atomic Schedule occurrence -> TaskExecutionRun admission.
-- The canonical TaskExecutionRun schema must be installed before this migration.
-- This adds persistence links only; it does not enable an occurrence worker or execution.

BEGIN;

ALTER TABLE automation.occurrence_dispatch
    ADD COLUMN IF NOT EXISTS admitted_run_id UUID;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'occurrence_dispatch_admitted_run_fk'
          AND conrelid = 'automation.occurrence_dispatch'::regclass
    ) THEN
        ALTER TABLE automation.occurrence_dispatch
            ADD CONSTRAINT occurrence_dispatch_admitted_run_fk
            FOREIGN KEY (tenant_id, admitted_run_id)
            REFERENCES multica.task_execution_run (tenant_id, run_id)
            ON DELETE RESTRICT;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_automation_occurrence_dispatch_admitted_run
    ON automation.occurrence_dispatch (tenant_id, admitted_run_id)
    WHERE admitted_run_id IS NOT NULL;

CREATE OR REPLACE FUNCTION automation.guard_occurrence_dispatch()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    v_now TIMESTAMPTZ := clock_timestamp();
    v_run_occurrence_id UUID;
    v_run_origin VARCHAR(16);
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.terminal_at IS NULL OR OLD.expires_at > v_now THEN
            RAISE EXCEPTION 'automation occurrence dispatch may only expire after terminal retention';
        END IF;
        RETURN OLD;
    END IF;

    IF (to_jsonb(OLD) - ARRAY[
            'dispatch_state','attempt_count','next_attempt_at','fencing_generation',
            'lease_owner_id','lease_expires_at','last_failure_code','terminal_at',
            'expires_at','updated_at','admitted_run_id'
        ]) <> (to_jsonb(NEW) - ARRAY[
            'dispatch_state','attempt_count','next_attempt_at','fencing_generation',
            'lease_owner_id','lease_expires_at','last_failure_code','terminal_at',
            'expires_at','updated_at','admitted_run_id'
        ]) THEN
        RAISE EXCEPTION 'automation occurrence dispatch identity and retention are immutable';
    END IF;
    IF OLD.terminal_at IS NOT NULL THEN
        RAISE EXCEPTION 'terminal automation occurrence dispatch cannot be rewritten';
    END IF;
    IF OLD.admitted_run_id IS NOT NULL
       AND NEW.admitted_run_id IS DISTINCT FROM OLD.admitted_run_id THEN
        RAISE EXCEPTION 'admitted TaskExecutionRun identity is immutable';
    END IF;
    IF OLD.admitted_run_id IS NULL AND NEW.admitted_run_id IS NOT NULL
       AND NOT (OLD.dispatch_state = 'leased' AND NEW.dispatch_state = 'admitted') THEN
        RAISE EXCEPTION 'a TaskExecutionRun link may only be written during fenced admission';
    END IF;
    IF NEW.dispatch_state <> OLD.dispatch_state AND NOT (
        (OLD.dispatch_state = 'pending' AND NEW.dispatch_state IN ('leased','failed','skipped','cancelled'))
        OR (OLD.dispatch_state = 'leased' AND NEW.dispatch_state IN
            ('retry_wait','admitted','succeeded','failed','skipped','cancelled'))
        OR (OLD.dispatch_state = 'retry_wait' AND NEW.dispatch_state IN ('leased','failed','skipped','cancelled'))
        OR (OLD.dispatch_state = 'admitted' AND NEW.dispatch_state IN ('succeeded','failed','cancelled'))
    ) THEN
        RAISE EXCEPTION 'invalid automation occurrence dispatch state transition';
    END IF;
    IF NEW.dispatch_state = 'admitted' AND NEW.admitted_run_id IS NULL THEN
        RAISE EXCEPTION 'admitted automation occurrence requires a TaskExecutionRun identity';
    END IF;
    IF NEW.admitted_run_id IS NOT NULL THEN
        SELECT automation_occurrence_id, run_origin
          INTO v_run_occurrence_id, v_run_origin
          FROM multica.task_execution_run
         WHERE tenant_id = NEW.tenant_id AND run_id = NEW.admitted_run_id;
        IF NOT FOUND OR v_run_origin <> 'schedule'
           OR v_run_occurrence_id IS DISTINCT FROM NEW.occurrence_id THEN
            RAISE EXCEPTION 'TaskExecutionRun does not belong to this Schedule occurrence';
        END IF;
        IF NEW.dispatch_state NOT IN ('admitted','succeeded','failed','cancelled') THEN
            RAISE EXCEPTION 'TaskExecutionRun link requires an admitted or terminal dispatch';
        END IF;
    END IF;
    IF NEW.fencing_generation < OLD.fencing_generation
       OR NEW.fencing_generation > OLD.fencing_generation + 1 THEN
        RAISE EXCEPTION 'automation occurrence fencing generation must be monotonic and contiguous';
    END IF;
    IF NEW.attempt_count < OLD.attempt_count OR NEW.attempt_count > OLD.attempt_count + 1 THEN
        RAISE EXCEPTION 'automation occurrence attempt count must be monotonic and contiguous';
    END IF;
    IF NEW.attempt_count > OLD.attempt_count
       AND NEW.fencing_generation <> OLD.fencing_generation + 1 THEN
        RAISE EXCEPTION 'automation occurrence attempt count can advance only with a new fence';
    END IF;
    IF NEW.dispatch_state = 'leased' AND OLD.dispatch_state <> 'leased'
       AND NEW.fencing_generation <> OLD.fencing_generation + 1 THEN
        RAISE EXCEPTION 'a new automation occurrence lease must increment its fencing generation';
    END IF;
    IF NEW.fencing_generation = OLD.fencing_generation + 1 THEN
        IF NEW.dispatch_state <> 'leased'
           OR NEW.lease_owner_id IS NULL
           OR NEW.lease_expires_at <= v_now
           OR NEW.attempt_count <> OLD.attempt_count + 1 THEN
            RAISE EXCEPTION 'new automation occurrence lease requires owner, expiry, attempt, and generation';
        END IF;
        IF OLD.dispatch_state = 'leased' AND OLD.lease_expires_at > v_now THEN
            RAISE EXCEPTION 'active automation occurrence lease cannot be stolen';
        END IF;
    END IF;
    IF NEW.dispatch_state = 'leased' AND OLD.dispatch_state = 'leased'
       AND NEW.fencing_generation = OLD.fencing_generation
       AND (NEW.lease_owner_id IS DISTINCT FROM OLD.lease_owner_id
            OR NEW.attempt_count <> OLD.attempt_count
            OR NEW.lease_expires_at < OLD.lease_expires_at) THEN
        RAISE EXCEPTION 'same-generation lease heartbeat must preserve owner, attempt, and expiry monotonicity';
    END IF;
    IF NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'automation occurrence dispatch updated_at cannot move backwards';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TABLE IF NOT EXISTS automation.schedule_run_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    rule_id UUID NOT NULL,
    rule_version BIGINT NOT NULL CHECK (rule_version > 0),
    occurrence_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    run_id UUID NOT NULL,
    run_as_actor_id UUID NOT NULL,
    event_type VARCHAR(32) NOT NULL CHECK (event_type = 'schedule_run.admitted'),
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload) = 'object' AND octet_length(payload::text) <= 32768),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, occurrence_id, event_type),
    FOREIGN KEY (tenant_id, project_id, rule_id, rule_version, engineering_run_id)
        REFERENCES automation.schedule_rule_revision
            (tenant_id, project_id, rule_id, rule_version, engineering_run_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, occurrence_id)
        REFERENCES automation.occurrence (tenant_id, project_id, occurrence_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, work_item_id, run_id)
        REFERENCES multica.task_execution_run (tenant_id, project_id, work_item_id, run_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (rule_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (occurrence_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (work_item_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (run_as_actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);

CREATE OR REPLACE FUNCTION automation.guard_schedule_run_outbox()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    v_run RECORD;
    v_occurrence RECORD;
    v_dispatch_run_id UUID;
BEGIN
    IF TG_OP <> 'INSERT' THEN
        RAISE EXCEPTION 'Schedule Run Outbox is append-only';
    END IF;
    SELECT run_origin, execution_channel, engineering_run_id,
           automation_rule_id, automation_rule_version, automation_occurrence_id, initiated_by
      INTO v_run
      FROM multica.task_execution_run
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
       AND work_item_id = NEW.work_item_id AND run_id = NEW.run_id;
    SELECT rule_id, rule_version, run_as_actor_id
      INTO v_occurrence
      FROM automation.occurrence
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
       AND occurrence_id = NEW.occurrence_id;
    SELECT admitted_run_id INTO v_dispatch_run_id
      FROM automation.occurrence_dispatch
     WHERE tenant_id = NEW.tenant_id AND occurrence_id = NEW.occurrence_id;
    IF NOT FOUND OR v_run.run_origin <> 'schedule' OR v_run.execution_channel <> 'agent'
       OR v_run.engineering_run_id IS DISTINCT FROM NEW.engineering_run_id
       OR v_run.automation_rule_id IS DISTINCT FROM NEW.rule_id
       OR v_run.automation_rule_version IS DISTINCT FROM NEW.rule_version
       OR v_run.automation_occurrence_id IS DISTINCT FROM NEW.occurrence_id
       OR v_run.initiated_by IS DISTINCT FROM NEW.run_as_actor_id
       OR v_occurrence.rule_id IS DISTINCT FROM NEW.rule_id
       OR v_occurrence.rule_version IS DISTINCT FROM NEW.rule_version
       OR v_occurrence.run_as_actor_id IS DISTINCT FROM NEW.run_as_actor_id
       OR v_dispatch_run_id IS DISTINCT FROM NEW.run_id THEN
        RAISE EXCEPTION 'Schedule Run Outbox does not match the admitted occurrence and Run';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS schedule_run_outbox_guard ON automation.schedule_run_outbox;
CREATE TRIGGER schedule_run_outbox_guard
    BEFORE INSERT OR UPDATE OR DELETE ON automation.schedule_run_outbox
    FOR EACH ROW EXECUTE FUNCTION automation.guard_schedule_run_outbox();
DROP TRIGGER IF EXISTS schedule_run_outbox_no_truncate ON automation.schedule_run_outbox;
CREATE TRIGGER schedule_run_outbox_no_truncate
    BEFORE TRUNCATE ON automation.schedule_run_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();
CREATE INDEX IF NOT EXISTS idx_automation_schedule_run_outbox_consumer
    ON automation.schedule_run_outbox (tenant_id, engineering_run_id, occurred_at, event_id);
ALTER TABLE automation.schedule_run_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.schedule_run_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_schedule_run_outbox_tenant_scope
    ON automation.schedule_run_outbox;
CREATE POLICY automation_schedule_run_outbox_tenant_scope
    ON automation.schedule_run_outbox
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
COMMENT ON COLUMN automation.occurrence_dispatch.admitted_run_id IS
    'Canonical TaskExecutionRun link for a future admitted dispatch writer; nullable legacy rows remain unresolved.';
COMMENT ON TABLE automation.schedule_run_outbox IS
    'T: append-only transactional admission event for Schedule occurrence to TaskExecutionRun; distinct from Rule and Task metadata Outboxes.';

COMMIT;
