-- Worktree Group Phase 2D: Worktree plan/confirm controls and immutable audit.
-- Additive only. Requires Phase 2B Worktree version and Project membership columns.
-- W/T: management plans are bounded Work; management audit is append-only Transaction.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.worktree_owner_assignment (
    assignment_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    assigned_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_worktree_owner_assignment_active
    ON multica.worktree_owner_assignment (tenant_id, worktree_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_worktree_owner_assignment_history
    ON multica.worktree_owner_assignment (tenant_id, project_id, worktree_id, valid_from DESC);

CREATE TABLE IF NOT EXISTS multica.worktree_management_plan (
    plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    requester_id UUID NOT NULL,
    operation VARCHAR(24) NOT NULL CHECK (operation IN ('assign_owner','set_archived')),
    target_owner_user_id UUID,
    target_archived BOOLEAN,
    expected_version INTEGER NOT NULL CHECK (expected_version > 0),
    idempotency_key VARCHAR(128) NOT NULL,
    request_hash BYTEA NOT NULL,
    correlation_id UUID NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','confirmed','expired','cancelled')),
    plan_response JSONB NOT NULL,
    result_body JSONB,
    confirm_by TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '5 minutes'),
    confirmed_at TIMESTAMPTZ,
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '30 days' CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (operation = 'assign_owner' AND target_owner_user_id IS NOT NULL AND target_archived IS NULL)
        OR (operation = 'set_archived' AND target_owner_user_id IS NULL AND target_archived IS NOT NULL)
    ),
    UNIQUE (tenant_id, requester_id, idempotency_key),
    UNIQUE (tenant_id, plan_id),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_worktree_management_plan_expiry
    ON multica.worktree_management_plan (expires_at, confirm_by)
    WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_worktree_management_plan_target
    ON multica.worktree_management_plan (tenant_id, worktree_id, created_at DESC);

CREATE TABLE IF NOT EXISTS multica.worktree_management_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    plan_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    correlation_id UUID NOT NULL,
    operation VARCHAR(24) NOT NULL,
    before_state JSONB NOT NULL,
    after_state JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, plan_id)
        REFERENCES multica.worktree_management_plan (tenant_id, plan_id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_worktree_management_audit_target_time
    ON multica.worktree_management_audit (tenant_id, project_id, worktree_id, occurred_at DESC);

CREATE OR REPLACE FUNCTION multica.reject_worktree_management_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'worktree management audit is append-only';
END;
$$;
DROP TRIGGER IF EXISTS worktree_management_audit_no_mutation ON multica.worktree_management_audit;
CREATE TRIGGER worktree_management_audit_no_mutation
    BEFORE UPDATE OR DELETE ON multica.worktree_management_audit
    FOR EACH ROW EXECUTE FUNCTION multica.reject_worktree_management_audit_mutation();

ALTER TABLE multica.worktree_owner_assignment ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.worktree_owner_assignment FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS worktree_owner_assignment_tenant_scope ON multica.worktree_owner_assignment;
CREATE POLICY worktree_owner_assignment_tenant_scope ON multica.worktree_owner_assignment
    FOR SELECT
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS worktree_owner_assignment_tenant_insert ON multica.worktree_owner_assignment;
CREATE POLICY worktree_owner_assignment_tenant_insert ON multica.worktree_owner_assignment
    FOR INSERT
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS worktree_owner_assignment_tenant_update ON multica.worktree_owner_assignment;
CREATE POLICY worktree_owner_assignment_tenant_update ON multica.worktree_owner_assignment
    FOR UPDATE
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.worktree_management_plan ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.worktree_management_plan FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS worktree_management_plan_tenant_scope ON multica.worktree_management_plan;
CREATE POLICY worktree_management_plan_tenant_scope ON multica.worktree_management_plan
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.worktree_management_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.worktree_management_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS worktree_management_audit_tenant_scope ON multica.worktree_management_audit;
CREATE POLICY worktree_management_audit_tenant_scope ON multica.worktree_management_audit
    FOR SELECT
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS worktree_management_audit_tenant_insert ON multica.worktree_management_audit;
CREATE POLICY worktree_management_audit_tenant_insert ON multica.worktree_management_audit
    FOR INSERT
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.worktree_management_plan IS
    'Short-lived owner/archive preview requiring explicit confirmation and expected Worktree version.';
COMMENT ON TABLE multica.worktree_owner_assignment IS
    'Current and historical human Worktree ownership; Master/SCD Type 2.';
COMMENT ON TABLE multica.worktree_management_audit IS
    'Append-only audit of confirmed Worktree ownership/archive management actions.';

COMMIT;
