-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-09-30-worktree-task-execution-run.sql",type:"file",language:"sql"}),
--   (m:Module {name:"task_execution_run_schema",type:"module",language:"sql"}),
--   (run:Class {name:"multica.task_execution_run",type:"class",language:"sql"}),
--   (event:Class {name:"multica.task_execution_run_event",type:"class",language:"sql"}),
--   (evidence:Class {name:"multica.task_execution_evidence",type:"class",language:"sql"}),
--   (idempotency:Class {name:"multica.task_execution_run_idempotency",type:"class",language:"sql"}),
--   (reject:Function {name:"multica.reject_task_execution_fact_mutation",type:"function",language:"sql"}),
--   (append_only:Logic {name:"task_execution_append_only_triggers",type:"logic",language:"sql"}),
--   (rls:Logic {name:"task_execution_rls_policies",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(run),(m)-[:CONTAINS]->(event),
--   (m)-[:CONTAINS]->(evidence),(m)-[:CONTAINS]->(idempotency),
--   (m)-[:CONTAINS]->(reject),(m)-[:CONTAINS]->(append_only),(m)-[:CONTAINS]->(rls),
--   (append_only)-[:CALLS]->(reject);

-- Phase 8A: durable Task Execution Run identity and evidence metadata.
-- Worktree/repository references are immutable snapshots, deliberately not foreign keys.
-- M: Task Contract (SCD2). W: idempotency replay mapping (30 days).
-- T: contract audit, Run, Run event and evidence metadata.
-- Phase 9 extension contracts snapshot profile, Hook, Loop, Automation occurrence and
-- project engineering manifest references; Automation remains the only schedule owner.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.task_contract (
    contract_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    version BIGINT NOT NULL CHECK (version > 0),
    goal TEXT NOT NULL CHECK (length(trim(goal)) > 0),
    scope JSONB NOT NULL CHECK (jsonb_typeof(scope) = 'object'),
    dependencies JSONB NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(dependencies) = 'array'),
    acceptance_criteria JSONB NOT NULL
        CHECK (jsonb_typeof(acceptance_criteria) = 'array'),
    changed_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, work_item_id, version),
    UNIQUE (tenant_id, project_id, work_item_id, version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_task_contract_current
    ON multica.task_contract (tenant_id, project_id, work_item_id) WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.task_contract_change_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    contract_id UUID NOT NULL,
    contract_version BIGINT NOT NULL,
    actor_id UUID NOT NULL,
    action VARCHAR(16) NOT NULL CHECK (action IN ('created','superseded')),
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, project_id, work_item_id, contract_version)
        REFERENCES multica.task_contract (tenant_id, project_id, work_item_id, version)
        ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_task_contract_audit_task_time
    ON multica.task_contract_change_audit (tenant_id, project_id, work_item_id, occurred_at DESC);

CREATE OR REPLACE FUNCTION multica.guard_task_contract_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'task contract master rows cannot be deleted';
    END IF;
    IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL
       OR NEW.valid_to < OLD.valid_from
       OR (to_jsonb(OLD) - 'valid_to') <> (to_jsonb(NEW) - 'valid_to') THEN
        RAISE EXCEPTION 'task contract rows may only be closed once without rewriting history';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS task_contract_scd2_guard ON multica.task_contract;
CREATE TRIGGER task_contract_scd2_guard
    BEFORE UPDATE OR DELETE ON multica.task_contract
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_contract_scd2();

CREATE TABLE IF NOT EXISTS multica.task_execution_run (
    run_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    initiated_by UUID NOT NULL,
    execution_channel VARCHAR(24) NOT NULL
        CHECK (execution_channel IN ('cli','langgraph','agent','plugin','manual')),
    worktree_id UUID,
    repository_id UUID,
    runtime_id UUID,
    start_ref VARCHAR(512),
    start_commit_ref VARCHAR(128),
    task_contract_version BIGINT,
    task_snapshot JSONB NOT NULL,
    acceptance_snapshot JSONB,
    agent_id UUID,
    model_version VARCHAR(160),
    skill_version VARCHAR(160),
    orchestrator_version VARCHAR(160),
    strategy_version VARCHAR(160),
    run_origin VARCHAR(16) NOT NULL DEFAULT 'manual'
        CHECK (run_origin IN ('manual','cli','schedule','workflow','agent','plugin')),
    automation_rule_id UUID,
    automation_rule_version BIGINT,
    automation_occurrence_id UUID,
    execution_profile_id UUID,
    execution_profile_version BIGINT,
    execution_profile_digest CHAR(64),
    execution_profile_snapshot JSONB,
    resource_budget_snapshot JSONB,
    loop_policy_snapshot JSONB,
    hook_set_snapshot JSONB,
    project_engineering_manifest_snapshot JSONB,
    correlation_id UUID NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, run_id),
    UNIQUE (tenant_id, project_id, work_item_id, run_id),
    CHECK (task_contract_version IS NULL OR task_contract_version > 0),
    CHECK (automation_rule_version IS NULL OR automation_rule_version > 0),
    CHECK (execution_profile_version IS NULL OR execution_profile_version > 0),
    CHECK (execution_profile_digest IS NULL OR execution_profile_digest ~ '^[0-9a-f]{64}$'),
    CHECK (execution_profile_snapshot IS NULL OR jsonb_typeof(execution_profile_snapshot) = 'object'),
    CHECK (resource_budget_snapshot IS NULL OR jsonb_typeof(resource_budget_snapshot) = 'object'),
    CHECK (loop_policy_snapshot IS NULL OR jsonb_typeof(loop_policy_snapshot) = 'object'),
    CHECK (hook_set_snapshot IS NULL OR jsonb_typeof(hook_set_snapshot) = 'object'),
    CHECK (project_engineering_manifest_snapshot IS NULL OR jsonb_typeof(project_engineering_manifest_snapshot) = 'object'),
    CHECK ((run_origin <> 'schedule') OR
        (automation_rule_id IS NOT NULL AND automation_rule_version IS NOT NULL AND automation_occurrence_id IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS idx_task_execution_run_task_time
    ON multica.task_execution_run (tenant_id, project_id, work_item_id, started_at DESC, run_id DESC);
CREATE INDEX IF NOT EXISTS idx_task_execution_run_worktree_time
    ON multica.task_execution_run (tenant_id, project_id, worktree_id, started_at DESC)
    WHERE worktree_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_task_execution_run_schedule_occurrence
    ON multica.task_execution_run (tenant_id, automation_occurrence_id)
    WHERE automation_occurrence_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS multica.task_execution_run_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    run_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    event_type VARCHAR(32) NOT NULL CHECK (event_type IN (
        'run_started','execution_state_changed','agent_declaration','validation_result',
        'human_review','integration_state_changed','human_intervention','cost_recorded',
        'failure_classified','evidence_attached','resource_summary',
        'schedule_occurrence_linked','loop_iteration_started','loop_decision_recorded','loop_stopped',
        'hook_evaluated','hook_override_recorded'
    )),
    execution_state VARCHAR(24) CHECK (execution_state IS NULL OR execution_state IN
        ('starting','running','succeeded','failed','cancelled','unknown')),
    agent_declaration_state VARCHAR(24) CHECK (agent_declaration_state IS NULL OR agent_declaration_state IN
        ('declared_complete','declared_failed','unknown')),
    verification_state VARCHAR(24) CHECK (verification_state IS NULL OR verification_state IN
        ('not_started','running','passed','failed','unknown')),
    human_acceptance_state VARCHAR(24) CHECK (human_acceptance_state IS NULL OR human_acceptance_state IN
        ('not_reviewed','accepted','rework_requested','rejected','unknown')),
    integration_state VARCHAR(24) CHECK (integration_state IS NULL OR integration_state IN
        ('not_integrated','integrated','reverted','unknown')),
    failure_category VARCHAR(64),
    actual_cost_amount NUMERIC(20, 6),
    estimated_cost_amount NUMERIC(20, 6),
    cost_unit VARCHAR(24),
    automation_occurrence_id UUID,
    loop_iteration_no INTEGER,
    loop_phase VARCHAR(16) CHECK (loop_phase IS NULL OR loop_phase IN ('plan','act','observe','verify','decision')),
    loop_decision VARCHAR(24) CHECK (loop_decision IS NULL OR loop_decision IN
        ('continue','completed','stop','retry','manual_review','unknown')),
    stop_reason VARCHAR(64),
    hook_set_version BIGINT,
    hook_rule_id UUID,
    hook_rule_version BIGINT,
    hook_evaluator_version VARCHAR(80),
    hook_digest CHAR(64),
    hook_phase VARCHAR(40) CHECK (hook_phase IS NULL OR hook_phase IN
        ('run_admission','tool','validation','review','worktree_archive','worktree_cleanup','after_commit')),
    hook_decision VARCHAR(24) CHECK (hook_decision IS NULL OR hook_decision IN
        ('allow','deny','require_human','defer')),
    hook_reason_class VARCHAR(64),
    hook_duration_ms BIGINT,
    hook_timed_out BOOLEAN,
    peak_rss_bytes BIGINT,
    cpu_time_ms BIGINT,
    child_process_high_water INTEGER,
    input_bytes BIGINT,
    output_bytes BIGINT,
    resource_measurement_source VARCHAR(48),
    resource_measurement_window_ms BIGINT,
    actor_id UUID,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(details) = 'object'),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, project_id, work_item_id, run_id)
        REFERENCES multica.task_execution_run (tenant_id, project_id, work_item_id, run_id)
        ON DELETE RESTRICT,
    CHECK (actual_cost_amount IS NULL OR actual_cost_amount >= 0),
    CHECK (estimated_cost_amount IS NULL OR estimated_cost_amount >= 0),
    CHECK ((actual_cost_amount IS NULL AND estimated_cost_amount IS NULL) OR cost_unit IS NOT NULL),
    CHECK (loop_iteration_no IS NULL OR loop_iteration_no >= 0),
    CHECK (hook_set_version IS NULL OR hook_set_version > 0),
    CHECK (hook_rule_version IS NULL OR hook_rule_version > 0),
    CHECK (hook_digest IS NULL OR hook_digest ~ '^[0-9a-f]{64}$'),
    CHECK (hook_duration_ms IS NULL OR hook_duration_ms >= 0),
    CHECK (peak_rss_bytes IS NULL OR peak_rss_bytes >= 0),
    CHECK (cpu_time_ms IS NULL OR cpu_time_ms >= 0),
    CHECK (child_process_high_water IS NULL OR child_process_high_water >= 0),
    CHECK (input_bytes IS NULL OR input_bytes >= 0),
    CHECK (output_bytes IS NULL OR output_bytes >= 0),
    CHECK (resource_measurement_window_ms IS NULL OR resource_measurement_window_ms >= 0)
);
-- Correlation ID groups related records; it is not an event deduplication key. Multiple state
-- transitions within one retried request may share the same correlation ID.
DROP INDEX IF EXISTS multica.uq_task_execution_run_event_correlation;
CREATE INDEX IF NOT EXISTS idx_task_execution_run_event_timeline
    ON multica.task_execution_run_event (tenant_id, run_id, occurred_at, event_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_task_execution_run_resource_summary
    ON multica.task_execution_run_event (tenant_id, run_id)
    WHERE event_type = 'resource_summary';
CREATE INDEX IF NOT EXISTS idx_task_execution_run_event_hook_scope
    ON multica.task_execution_run_event
        (tenant_id, project_id, hook_set_version, hook_rule_id, occurred_at DESC)
    WHERE event_type IN ('hook_evaluated','hook_override_recorded');
CREATE INDEX IF NOT EXISTS idx_task_execution_run_event_schedule_occurrence
    ON multica.task_execution_run_event (tenant_id, automation_occurrence_id)
    WHERE automation_occurrence_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS multica.task_execution_evidence (
    evidence_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    run_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    evidence_kind VARCHAR(32) NOT NULL CHECK (evidence_kind IN (
        'validation','review','change_summary','artifact','benchmark','other'
    )),
    summary TEXT,
    artifact_locator TEXT,
    sha256_digest CHAR(64),
    media_type VARCHAR(128),
    byte_length BIGINT,
    sensitivity VARCHAR(16) NOT NULL DEFAULT 'redacted'
        CHECK (sensitivity IN ('public','internal','redacted')),
    actor_id UUID,
    correlation_id UUID NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, project_id, work_item_id, run_id)
        REFERENCES multica.task_execution_run (tenant_id, project_id, work_item_id, run_id)
        ON DELETE RESTRICT,
    CHECK (sha256_digest IS NULL OR sha256_digest ~ '^[0-9a-f]{64}$'),
    CHECK (byte_length IS NULL OR byte_length >= 0),
    CHECK (artifact_locator IS NULL OR length(artifact_locator) <= 2048),
    CHECK (summary IS NULL OR length(summary) <= 2000)
);
CREATE INDEX IF NOT EXISTS idx_task_execution_evidence_run_time
    ON multica.task_execution_evidence (tenant_id, run_id, captured_at, evidence_id);

CREATE TABLE IF NOT EXISTS multica.task_execution_run_idempotency (
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    operation VARCHAR(32) NOT NULL CHECK (operation IN ('cli_session_start')),
    idempotency_key VARCHAR(128) NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    run_id UUID NOT NULL,
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '30 days'
        CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, actor_id, operation, idempotency_key),
    FOREIGN KEY (tenant_id, run_id)
        REFERENCES multica.task_execution_run (tenant_id, run_id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_task_execution_run_idempotency_expiry
    ON multica.task_execution_run_idempotency (expires_at);

CREATE OR REPLACE FUNCTION multica.reject_task_execution_fact_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'task execution facts are append-only';
END;
$$;

DROP TRIGGER IF EXISTS task_execution_run_no_mutation ON multica.task_execution_run;
CREATE TRIGGER task_execution_run_no_mutation
    BEFORE UPDATE OR DELETE ON multica.task_execution_run
    FOR EACH ROW EXECUTE FUNCTION multica.reject_task_execution_fact_mutation();
DROP TRIGGER IF EXISTS task_execution_run_event_no_mutation ON multica.task_execution_run_event;
CREATE TRIGGER task_execution_run_event_no_mutation
    BEFORE UPDATE OR DELETE ON multica.task_execution_run_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_task_execution_fact_mutation();
DROP TRIGGER IF EXISTS task_execution_evidence_no_mutation ON multica.task_execution_evidence;
CREATE TRIGGER task_execution_evidence_no_mutation
    BEFORE UPDATE OR DELETE ON multica.task_execution_evidence
    FOR EACH ROW EXECUTE FUNCTION multica.reject_task_execution_fact_mutation();

ALTER TABLE multica.task_execution_run ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_execution_run FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_execution_run_tenant_scope ON multica.task_execution_run;
CREATE POLICY task_execution_run_tenant_scope ON multica.task_execution_run
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_contract ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_contract FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_contract_tenant_scope ON multica.task_contract;
CREATE POLICY task_contract_tenant_scope ON multica.task_contract
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_contract_change_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_contract_change_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_contract_audit_tenant_scope ON multica.task_contract_change_audit;
CREATE POLICY task_contract_audit_tenant_scope ON multica.task_contract_change_audit
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP TRIGGER IF EXISTS task_contract_audit_no_mutation ON multica.task_contract_change_audit;
CREATE TRIGGER task_contract_audit_no_mutation
    BEFORE UPDATE OR DELETE ON multica.task_contract_change_audit
    FOR EACH ROW EXECUTE FUNCTION multica.reject_task_execution_fact_mutation();

ALTER TABLE multica.task_execution_run_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_execution_run_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_execution_run_event_tenant_scope ON multica.task_execution_run_event;
CREATE POLICY task_execution_run_event_tenant_scope ON multica.task_execution_run_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_execution_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_execution_evidence FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_execution_evidence_tenant_scope ON multica.task_execution_evidence;
CREATE POLICY task_execution_evidence_tenant_scope ON multica.task_execution_evidence
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_execution_run_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_execution_run_idempotency FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_execution_run_idempotency_actor_scope ON multica.task_execution_run_idempotency;
CREATE POLICY task_execution_run_idempotency_actor_scope ON multica.task_execution_run_idempotency
    USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    )
    WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

COMMENT ON TABLE multica.task_execution_run IS
    'Immutable Task execution attempt snapshot; worktree_id/repository_id are historical refs, not lifecycle foreign keys.';
COMMENT ON TABLE multica.task_execution_run_event IS
    'Append-only, dimensioned execution/validation/review/integration/cost events; details must be sanitized metadata.';
COMMENT ON TABLE multica.task_execution_evidence IS
    'Append-only evidence metadata and digest; large/raw artifacts are stored externally under their own access policy.';
COMMENT ON TABLE multica.task_execution_run_idempotency IS
    'Short-lived CLI start replay mapping; expires after 30 days and never defines Run identity.';
COMMENT ON TABLE multica.task_contract IS
    'Versioned Task goal/scope/dependencies/acceptance criteria; close current and append a successor.';
COMMENT ON TABLE multica.task_contract_change_audit IS
    'Append-only audit for Task Contract version creation and supersession.';

COMMIT;
