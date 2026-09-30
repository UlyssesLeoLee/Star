-- Cypher structural manifest.
-- CREATE
--   (file:File {name:"2026-10-01-multica-hook-execution-event.sql",type:"file",language:"sql"}),
--   (module:Module {name:"multica_hook_execution_event",type:"module",language:"sql"}),
--   (event:Class {name:"multica.hook_execution_event",type:"class",language:"sql"}),
--   (reject:Function {name:"multica.reject_hook_execution_event_mutation",type:"function",language:"sql"}),
--   (rls:Logic {name:"hook_execution_event_tenant_rls",type:"logic",language:"sql"}),
--   (file)-[:CONTAINS]->(module),(module)-[:CONTAINS]->(event),
--   (module)-[:CONTAINS]->(reject),(module)-[:CONTAINS]->(rls),
--   (reject)-[:GUARDS]->(event),(rls)-[:SCOPES]->(event);

-- Phase 9D: append-only Hook execution event and transactional consumer source.
-- Project/Worktree/Run identifiers are historical references; this ledger must
-- survive Worktree cleanup and must not hold lifecycle rows open with foreign keys.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.hook_execution_event (
    event_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID,
    work_item_id UUID,
    run_id UUID,
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    source_kind VARCHAR(24) NOT NULL CHECK (source_kind IN (
        'worktree_lifecycle','task_run','agent_runtime','automation'
    )),
    hook_phase VARCHAR(40) NOT NULL CHECK (hook_phase IN (
        'run_admission','tool','validation','review','worktree_archive','worktree_cleanup','after_commit'
    )),
    hook_decision VARCHAR(24) NOT NULL CHECK (hook_decision IN (
        'allow','deny','require_human','defer'
    )),
    hook_reason_code VARCHAR(64) NOT NULL CHECK (hook_reason_code IN (
        'allowed_by_builtin_baseline','incomplete_scope','event_schema_unsupported',
        'actor_not_authorized','lifecycle_version_stale','runtime_unhealthy_or_unknown',
        'retention_lock_unusable','execution_not_drained','policy_unavailable','policy_invalid',
        'rule_denied','human_approval_required','external_condition_pending'
    )),
    matched_rule_id UUID,
    project_policy_version BIGINT CHECK (project_policy_version IS NULL OR project_policy_version > 0),
    worktree_policy_version BIGINT CHECK (worktree_policy_version IS NULL OR worktree_policy_version > 0),
    evaluator_api_version SMALLINT NOT NULL CHECK (evaluator_api_version > 0),
    policy_digest CHAR(64) CHECK (policy_digest IS NULL OR policy_digest ~ '^[0-9a-f]{64}$'),
    evaluated_condition_count INTEGER NOT NULL CHECK (evaluated_condition_count >= 0),
    duration_ms BIGINT NOT NULL CHECK (duration_ms >= 0),
    timed_out BOOLEAN NOT NULL DEFAULT false,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((run_id IS NULL) = (work_item_id IS NULL)),
    CHECK (source_kind <> 'worktree_lifecycle' OR worktree_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_hook_execution_event_project_cursor
    ON multica.hook_execution_event (tenant_id, project_id, occurred_at DESC, event_id DESC);
CREATE INDEX IF NOT EXISTS idx_hook_execution_event_project_metric
    ON multica.hook_execution_event
        (tenant_id, project_id, hook_phase, hook_decision, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_hook_execution_event_run
    ON multica.hook_execution_event (tenant_id, run_id, occurred_at DESC, event_id DESC)
    WHERE run_id IS NOT NULL;

CREATE OR REPLACE FUNCTION multica.reject_hook_execution_event_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Hook execution events are append-only';
END;
$$;
DROP TRIGGER IF EXISTS hook_execution_event_no_mutation ON multica.hook_execution_event;
CREATE TRIGGER hook_execution_event_no_mutation
    BEFORE UPDATE OR DELETE ON multica.hook_execution_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_hook_execution_event_mutation();
DROP TRIGGER IF EXISTS hook_execution_event_no_truncate ON multica.hook_execution_event;
CREATE TRIGGER hook_execution_event_no_truncate
    BEFORE TRUNCATE ON multica.hook_execution_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_hook_execution_event_mutation();

ALTER TABLE multica.hook_execution_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.hook_execution_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS hook_execution_event_tenant_scope ON multica.hook_execution_event;
CREATE POLICY hook_execution_event_tenant_scope ON multica.hook_execution_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.hook_execution_event IS
    'Append-only, sanitized Hook evaluation event and transactional source for bounded BI consumers; historical scope IDs do not retain Worktree/Run rows.';
COMMENT ON COLUMN multica.hook_execution_event.details IS
    'Bounded sanitized decision facts only; never store prompts, secrets, command bodies, stdout/stderr, or chain-of-thought.';

COMMIT;
