-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-02-automation-schedule-occurrence.sql"})
-- MERGE (rule:Table {id:"automation.schedule_rule_revision"})
-- MERGE (rule_audit:Table {id:"automation.schedule_rule_audit"})
-- MERGE (occurrence:Table {id:"automation.occurrence"})
-- MERGE (dispatch:Table {id:"automation.occurrence_dispatch"})
-- MERGE (event:Table {id:"automation.occurrence_event"})
-- MERGE (rule_guard:Symbol {id:"automation.guard_schedule_rule_scd2",kind:"function"})
-- MERGE (fact_guard:Symbol {id:"automation.reject_schedule_history_mutation",kind:"function"})
-- MERGE (dispatch_guard:Symbol {id:"automation.guard_occurrence_dispatch",kind:"function"})
-- MERGE (self)-[:DEFINES]->(rule)
-- MERGE (self)-[:DEFINES]->(rule_audit)
-- MERGE (self)-[:DEFINES]->(occurrence)
-- MERGE (self)-[:DEFINES]->(dispatch)
-- MERGE (self)-[:DEFINES]->(event)
-- MERGE (self)-[:DEFINES]->(rule_guard)
-- MERGE (self)-[:DEFINES]->(fact_guard)
-- MERGE (self)-[:DEFINES]->(dispatch_guard)
-- MERGE (self)-[:CONFIGURES]->(rule_guard)
-- MERGE (self)-[:CONFIGURES]->(fact_guard)
-- MERGE (self)-[:CONFIGURES]->(dispatch_guard)
-- @endcypher

-- Phase 9F2: durable Schedule rule revision and occurrence/lease substrate.
-- M: immutable Schedule rule revisions (SCD2 close-only).
-- T: rule audit, materialized occurrence snapshots, and append-only occurrence events.
-- W: mutable bounded dispatch state with explicit retention and lease expiry.
-- This migration does not enable a dispatcher or create a TaskExecutionRun.

BEGIN;

CREATE SCHEMA IF NOT EXISTS automation;

CREATE TABLE IF NOT EXISTS automation.schedule_rule_revision (
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    rule_id UUID NOT NULL,
    rule_version BIGINT NOT NULL CHECK (rule_version > 0),
    schema_version SMALLINT NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    cron_expression VARCHAR(120) NOT NULL
        CHECK (length(trim(cron_expression)) > 0 AND cron_expression !~ '[[:cntrl:]]'),
    time_zone VARCHAR(128) NOT NULL
        CHECK (length(trim(time_zone)) > 0 AND time_zone !~ '[[:cntrl:]]'),
    parser_version VARCHAR(64) NOT NULL
        CHECK (length(trim(parser_version)) > 0 AND parser_version !~ '[[:cntrl:]]'),
    tzdb_version VARCHAR(64) NOT NULL
        CHECK (length(trim(tzdb_version)) > 0 AND tzdb_version !~ '[[:cntrl:]]'),
    dst_gap_policy VARCHAR(16) NOT NULL
        CHECK (dst_gap_policy IN ('skip','shift_forward')),
    dst_fold_policy VARCHAR(24) NOT NULL
        CHECK (dst_fold_policy IN ('earlier_instant','later_instant')),
    overlap_policy VARCHAR(16) NOT NULL
        CHECK (overlap_policy IN ('skip','queue_one','allow_bounded')),
    overlap_max_active SMALLINT NOT NULL DEFAULT 1
        CHECK (overlap_max_active BETWEEN 1 AND 64),
    misfire_policy VARCHAR(24) NOT NULL
        CHECK (misfire_policy IN ('skip','coalesce_latest','catch_up')),
    misfire_max_occurrences SMALLINT NOT NULL DEFAULT 1
        CHECK (misfire_max_occurrences BETWEEN 1 AND 256),
    pause_policy VARCHAR(24) NOT NULL
        CHECK (pause_policy IN ('skip_elapsed','coalesce_latest')),
    retry_max_attempts SMALLINT NOT NULL
        CHECK (retry_max_attempts BETWEEN 1 AND 25),
    retry_initial_backoff_seconds INTEGER NOT NULL
        CHECK (retry_initial_backoff_seconds BETWEEN 1 AND 86400),
    retry_max_backoff_seconds INTEGER NOT NULL
        CHECK (retry_max_backoff_seconds BETWEEN retry_initial_backoff_seconds AND 86400),
    deadline_seconds INTEGER NOT NULL CHECK (deadline_seconds BETWEEN 1 AND 86400),
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    repository_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    execution_profile_id UUID NOT NULL,
    execution_profile_version BIGINT NOT NULL CHECK (execution_profile_version > 0),
    execution_profile_digest CHAR(64) NOT NULL
        CHECK (execution_profile_digest ~ '^[0-9a-f]{64}$'),
    hook_set_version BIGINT NOT NULL CHECK (hook_set_version > 0),
    hook_set_digest CHAR(64) NOT NULL CHECK (hook_set_digest ~ '^[0-9a-f]{64}$'),
    changed_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, rule_id, rule_version),
    UNIQUE (tenant_id, project_id, rule_id, rule_version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (rule_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (branch_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (repository_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (worktree_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (work_item_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (execution_profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK ((overlap_policy = 'allow_bounded') OR overlap_max_active = 1),
    CHECK ((misfire_policy = 'catch_up') OR misfire_max_occurrences = 1)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_automation_schedule_rule_current
    ON automation.schedule_rule_revision (tenant_id, project_id, rule_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_automation_schedule_rule_due
    ON automation.schedule_rule_revision (tenant_id, enabled, project_id, rule_id)
    WHERE valid_to IS NULL AND enabled;

CREATE TABLE IF NOT EXISTS automation.schedule_rule_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    rule_id UUID NOT NULL,
    rule_version BIGINT NOT NULL CHECK (rule_version > 0),
    actor_id UUID NOT NULL,
    action VARCHAR(16) NOT NULL CHECK (action IN ('created','superseded')),
    correlation_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object' AND octet_length(details::text) <= 8192),
    FOREIGN KEY (tenant_id, project_id, rule_id, rule_version)
        REFERENCES automation.schedule_rule_revision
            (tenant_id, project_id, rule_id, rule_version)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (rule_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_automation_schedule_rule_audit_scope_time
    ON automation.schedule_rule_audit
        (tenant_id, project_id, rule_id, occurred_at DESC, event_id DESC);

CREATE OR REPLACE FUNCTION automation.guard_schedule_rule_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'automation schedule rule revisions cannot be deleted';
    END IF;
    IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL
       OR NEW.valid_to < OLD.valid_from
       OR (to_jsonb(OLD) - 'valid_to') <> (to_jsonb(NEW) - 'valid_to') THEN
        RAISE EXCEPTION 'automation schedule rule revisions may only be closed once';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS automation_schedule_rule_scd2_guard
    ON automation.schedule_rule_revision;
CREATE TRIGGER automation_schedule_rule_scd2_guard
    BEFORE UPDATE OR DELETE ON automation.schedule_rule_revision
    FOR EACH ROW EXECUTE FUNCTION automation.guard_schedule_rule_scd2();

CREATE TABLE IF NOT EXISTS automation.occurrence (
    occurrence_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    rule_id UUID NOT NULL,
    rule_version BIGINT NOT NULL CHECK (rule_version > 0),
    scheduled_for_utc TIMESTAMPTZ NOT NULL,
    scheduled_local_label VARCHAR(64) NOT NULL
        CHECK (length(trim(scheduled_local_label)) > 0 AND scheduled_local_label !~ '[[:cntrl:]]'),
    utc_offset_seconds INTEGER NOT NULL CHECK (utc_offset_seconds BETWEEN -86400 AND 86400),
    parser_version VARCHAR(64) NOT NULL
        CHECK (length(trim(parser_version)) > 0 AND parser_version !~ '[[:cntrl:]]'),
    tzdb_version VARCHAR(64) NOT NULL
        CHECK (length(trim(tzdb_version)) > 0 AND tzdb_version !~ '[[:cntrl:]]'),
    target_snapshot JSONB NOT NULL
        CHECK (jsonb_typeof(target_snapshot) = 'object' AND octet_length(target_snapshot::text) <= 65536),
    target_snapshot_digest CHAR(64) NOT NULL
        CHECK (target_snapshot_digest ~ '^[0-9a-f]{64}$'),
    materialized_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, occurrence_id),
    UNIQUE (tenant_id, project_id, occurrence_id),
    UNIQUE (tenant_id, rule_id, rule_version, scheduled_for_utc),
    FOREIGN KEY (tenant_id, project_id, rule_id, rule_version)
        REFERENCES automation.schedule_rule_revision
            (tenant_id, project_id, rule_id, rule_version)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (rule_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_automation_occurrence_rule_slot
    ON automation.occurrence
        (tenant_id, project_id, rule_id, scheduled_for_utc DESC, occurrence_id DESC);

CREATE TABLE IF NOT EXISTS automation.occurrence_dispatch (
    tenant_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    dispatch_state VARCHAR(16) NOT NULL DEFAULT 'pending'
        CHECK (dispatch_state IN (
            'pending','leased','retry_wait','admitted','succeeded','failed','skipped','cancelled'
        )),
    attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 25),
    next_attempt_at TIMESTAMPTZ NOT NULL,
    fencing_generation BIGINT NOT NULL DEFAULT 0 CHECK (fencing_generation >= 0),
    lease_owner_id UUID,
    lease_expires_at TIMESTAMPTZ,
    occurrence_deadline_at TIMESTAMPTZ NOT NULL,
    last_failure_code VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '30 days'
        CHECK (retention_period > INTERVAL '0 seconds' AND retention_period <= INTERVAL '365 days'),
    terminal_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, occurrence_id),
    FOREIGN KEY (tenant_id, occurrence_id)
        REFERENCES automation.occurrence (tenant_id, occurrence_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (occurrence_deadline_at > created_at),
    CHECK ((dispatch_state = 'leased') =
        (lease_owner_id IS NOT NULL AND lease_expires_at IS NOT NULL)),
    CHECK (lease_expires_at IS NULL OR lease_expires_at <= occurrence_deadline_at),
    CHECK ((dispatch_state IN ('succeeded','failed','skipped','cancelled')) =
        (terminal_at IS NOT NULL)),
    CHECK ((terminal_at IS NULL) = (expires_at IS NULL)),
    CHECK (terminal_at IS NULL OR terminal_at >= created_at),
    CHECK (expires_at IS NULL OR expires_at = terminal_at + retention_period),
    CHECK (last_failure_code IS NULL OR length(last_failure_code) <= 64)
);
CREATE INDEX IF NOT EXISTS idx_automation_occurrence_dispatch_due
    ON automation.occurrence_dispatch (tenant_id, next_attempt_at, occurrence_id)
    WHERE dispatch_state IN ('pending','retry_wait');
CREATE INDEX IF NOT EXISTS idx_automation_occurrence_dispatch_expired_lease
    ON automation.occurrence_dispatch (tenant_id, lease_expires_at, occurrence_id)
    WHERE dispatch_state = 'leased';
CREATE INDEX IF NOT EXISTS idx_automation_occurrence_dispatch_expiry
    ON automation.occurrence_dispatch (expires_at, terminal_at)
    WHERE dispatch_state IN ('succeeded','failed','skipped','cancelled');

CREATE TABLE IF NOT EXISTS automation.occurrence_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    event_type VARCHAR(24) NOT NULL CHECK (event_type IN (
        'materialized','lease_claimed','lease_expired','retry_scheduled',
        'run_admitted','run_succeeded','run_failed','skipped','cancelled'
    )),
    attempt_no SMALLINT CHECK (attempt_no IS NULL OR attempt_no BETWEEN 0 AND 25),
    fencing_generation BIGINT CHECK (fencing_generation IS NULL OR fencing_generation > 0),
    run_id UUID,
    actor_id UUID,
    correlation_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object' AND octet_length(details::text) <= 8192),
    UNIQUE (tenant_id, event_id),
    FOREIGN KEY (tenant_id, occurrence_id)
        REFERENCES automation.occurrence (tenant_id, occurrence_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, occurrence_id)
        REFERENCES automation.occurrence (tenant_id, project_id, occurrence_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id IS NULL OR actor_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_automation_occurrence_event_timeline
    ON automation.occurrence_event
        (tenant_id, project_id, occurrence_id, occurred_at, event_id);
CREATE INDEX IF NOT EXISTS idx_automation_occurrence_event_run
    ON automation.occurrence_event (tenant_id, run_id, occurred_at DESC)
    WHERE run_id IS NOT NULL;

CREATE OR REPLACE FUNCTION automation.reject_schedule_history_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'automation schedule history cannot be rewritten or truncated';
END;
$$;

CREATE OR REPLACE FUNCTION automation.guard_occurrence_dispatch()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    v_now TIMESTAMPTZ := clock_timestamp();
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
            'expires_at','updated_at'
        ]) <> (to_jsonb(NEW) - ARRAY[
            'dispatch_state','attempt_count','next_attempt_at','fencing_generation',
            'lease_owner_id','lease_expires_at','last_failure_code','terminal_at',
            'expires_at','updated_at'
        ]) THEN
        RAISE EXCEPTION 'automation occurrence dispatch identity and retention are immutable';
    END IF;
    IF OLD.terminal_at IS NOT NULL THEN
        RAISE EXCEPTION 'terminal automation occurrence dispatch cannot be rewritten';
    END IF;
    IF NEW.dispatch_state <> OLD.dispatch_state AND NOT (
        (OLD.dispatch_state = 'pending' AND NEW.dispatch_state IN ('leased','skipped','cancelled'))
        OR (OLD.dispatch_state = 'leased' AND NEW.dispatch_state IN
            ('retry_wait','admitted','succeeded','failed','skipped','cancelled'))
        OR (OLD.dispatch_state = 'retry_wait' AND NEW.dispatch_state IN ('leased','skipped','cancelled'))
        OR (OLD.dispatch_state = 'admitted' AND NEW.dispatch_state IN ('succeeded','failed','cancelled'))
    ) THEN
        RAISE EXCEPTION 'invalid automation occurrence dispatch state transition';
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

DROP TRIGGER IF EXISTS automation_occurrence_dispatch_guard
    ON automation.occurrence_dispatch;
CREATE TRIGGER automation_occurrence_dispatch_guard
    BEFORE UPDATE OR DELETE ON automation.occurrence_dispatch
    FOR EACH ROW EXECUTE FUNCTION automation.guard_occurrence_dispatch();
DROP TRIGGER IF EXISTS automation_occurrence_dispatch_no_truncate
    ON automation.occurrence_dispatch;
CREATE TRIGGER automation_occurrence_dispatch_no_truncate
    BEFORE TRUNCATE ON automation.occurrence_dispatch
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();

DROP TRIGGER IF EXISTS automation_schedule_rule_audit_no_mutation
    ON automation.schedule_rule_audit;
CREATE TRIGGER automation_schedule_rule_audit_no_mutation
    BEFORE UPDATE OR DELETE ON automation.schedule_rule_audit
    FOR EACH ROW EXECUTE FUNCTION automation.reject_schedule_history_mutation();
DROP TRIGGER IF EXISTS automation_occurrence_no_mutation
    ON automation.occurrence;
CREATE TRIGGER automation_occurrence_no_mutation
    BEFORE UPDATE OR DELETE ON automation.occurrence
    FOR EACH ROW EXECUTE FUNCTION automation.reject_schedule_history_mutation();
DROP TRIGGER IF EXISTS automation_occurrence_event_no_mutation
    ON automation.occurrence_event;
CREATE TRIGGER automation_occurrence_event_no_mutation
    BEFORE UPDATE OR DELETE ON automation.occurrence_event
    FOR EACH ROW EXECUTE FUNCTION automation.reject_schedule_history_mutation();

DROP TRIGGER IF EXISTS automation_schedule_rule_no_truncate
    ON automation.schedule_rule_revision;
CREATE TRIGGER automation_schedule_rule_no_truncate
    BEFORE TRUNCATE ON automation.schedule_rule_revision
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();
DROP TRIGGER IF EXISTS automation_schedule_rule_audit_no_truncate
    ON automation.schedule_rule_audit;
CREATE TRIGGER automation_schedule_rule_audit_no_truncate
    BEFORE TRUNCATE ON automation.schedule_rule_audit
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();
DROP TRIGGER IF EXISTS automation_occurrence_no_truncate
    ON automation.occurrence;
CREATE TRIGGER automation_occurrence_no_truncate
    BEFORE TRUNCATE ON automation.occurrence
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();
DROP TRIGGER IF EXISTS automation_occurrence_event_no_truncate
    ON automation.occurrence_event;
CREATE TRIGGER automation_occurrence_event_no_truncate
    BEFORE TRUNCATE ON automation.occurrence_event
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();

ALTER TABLE automation.schedule_rule_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.schedule_rule_revision FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_schedule_rule_tenant_scope
    ON automation.schedule_rule_revision;
CREATE POLICY automation_schedule_rule_tenant_scope
    ON automation.schedule_rule_revision
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE automation.schedule_rule_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.schedule_rule_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_schedule_rule_audit_tenant_scope
    ON automation.schedule_rule_audit;
CREATE POLICY automation_schedule_rule_audit_tenant_scope
    ON automation.schedule_rule_audit
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE automation.occurrence ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.occurrence FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_occurrence_tenant_scope
    ON automation.occurrence;
CREATE POLICY automation_occurrence_tenant_scope
    ON automation.occurrence
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE automation.occurrence_dispatch ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.occurrence_dispatch FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_occurrence_dispatch_tenant_scope
    ON automation.occurrence_dispatch;
CREATE POLICY automation_occurrence_dispatch_tenant_scope
    ON automation.occurrence_dispatch
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE automation.occurrence_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.occurrence_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_occurrence_event_tenant_scope
    ON automation.occurrence_event;
CREATE POLICY automation_occurrence_event_tenant_scope
    ON automation.occurrence_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE automation.schedule_rule_revision IS
    'M: immutable versioned Automation Schedule rule; only a close-only SCD2 valid_to update is permitted.';
COMMENT ON TABLE automation.schedule_rule_audit IS
    'T: append-only actor audit for Automation Schedule rule revision creation and supersession.';
COMMENT ON TABLE automation.occurrence IS
    'T: append-only materialized UTC slot and immutable Worktree-first target/profile/HookSet snapshot.';
COMMENT ON TABLE automation.occurrence_dispatch IS
    'W: mutable bounded dispatch/lease state with monotonic fencing generation and explicit retention.';
COMMENT ON TABLE automation.occurrence_event IS
    'T: append-only occurrence materialization, lease, retry, Run admission, and terminal outcome facts.';

COMMIT;
