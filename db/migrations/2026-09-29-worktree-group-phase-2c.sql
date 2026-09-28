-- Worktree Group Phase 2C: canonical WorkItem metadata, lifecycle and audit spine.
-- Additive only. Requires Phase 2B project/worktree links and membership bindings.
-- W/T/M: task_lifecycle_current + task_command_idempotency are Work;
-- task_metadata + work_item_worktree are Master/SCD2;
-- task_lifecycle_audit is Transaction/append-only.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.task_metadata (
    metadata_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_item_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    project_id UUID NOT NULL,
    item_type VARCHAR(24) NOT NULL CHECK (item_type IN ('epic','story','task','bug','subtask','ai_task')),
    title VARCHAR(500) NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT NOT NULL DEFAULT '',
    priority VARCHAR(16) NOT NULL DEFAULT 'medium' CHECK (priority IN ('low','medium','high','urgent')),
    labels TEXT[] NOT NULL DEFAULT '{}',
    ai_task_data JSONB,
    reporter_user_id UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_task_metadata_current
    ON multica.task_metadata (tenant_id, work_item_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_task_metadata_project_current
    ON multica.task_metadata (tenant_id, project_id, updated_at DESC, work_item_id)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.work_item_worktree (
    relation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    relation_kind VARCHAR(24) NOT NULL DEFAULT 'associated' CHECK (relation_kind IN ('associated')),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_work_item_worktree_current
    ON multica.work_item_worktree (tenant_id, work_item_id, worktree_id, relation_kind)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_work_item_worktree_active
    ON multica.work_item_worktree (tenant_id, worktree_id, work_item_id)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.task_lifecycle_current (
    work_item_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    active_worktree_id UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','claimed','in_progress','completed','failed','cancelled')),
    review_state VARCHAR(20) NOT NULL DEFAULT 'none'
        CHECK (review_state IN ('none','pending_review','accepted','rejected')),
    claimed_by UUID,
    claim_expires_at TIMESTAMPTZ,
    runtime_session_id UUID,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '365 days' CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, active_worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CHECK (
        (status IN ('claimed','in_progress') AND active_worktree_id IS NOT NULL)
        OR (status NOT IN ('claimed','in_progress') AND active_worktree_id IS NULL)
    ),
    CHECK (
        (status = 'claimed' AND claimed_by IS NOT NULL AND claim_expires_at IS NOT NULL)
        OR (status = 'in_progress' AND claimed_by IS NOT NULL AND claim_expires_at IS NULL)
        OR (status NOT IN ('claimed','in_progress') AND claimed_by IS NULL AND claim_expires_at IS NULL)
    ),
    CHECK (status NOT IN ('completed','failed','cancelled') OR expires_at IS NOT NULL)
);
CREATE INDEX IF NOT EXISTS idx_task_lifecycle_worktree_status
    ON multica.task_lifecycle_current (tenant_id, active_worktree_id, status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_task_lifecycle_expiry
    ON multica.task_lifecycle_current (expires_at) WHERE expires_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS multica.task_command_idempotency (
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    request_hash BYTEA NOT NULL,
    response_body JSONB NOT NULL,
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '30 days' CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, actor_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_task_command_idempotency_expiry
    ON multica.task_command_idempotency (expires_at);

CREATE TABLE IF NOT EXISTS multica.task_lifecycle_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    task_card_id UUID NOT NULL,
    from_status VARCHAR(20),
    to_status VARCHAR(20),
    event_type VARCHAR(48) NOT NULL,
    actor_id UUID NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CHECK (task_card_id = work_item_id)
);
CREATE INDEX IF NOT EXISTS idx_task_lifecycle_audit_item_time
    ON multica.task_lifecycle_audit (tenant_id, worktree_id, work_item_id, occurred_at DESC);

CREATE OR REPLACE FUNCTION multica.reject_task_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'task lifecycle audit is append-only';
END;
$$;
DROP TRIGGER IF EXISTS task_lifecycle_audit_no_mutation ON multica.task_lifecycle_audit;
CREATE TRIGGER task_lifecycle_audit_no_mutation
    BEFORE UPDATE OR DELETE ON multica.task_lifecycle_audit
    FOR EACH ROW EXECUTE FUNCTION multica.reject_task_audit_mutation();

ALTER TABLE multica.task_metadata ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_metadata FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_metadata_tenant_select ON multica.task_metadata;
CREATE POLICY task_metadata_tenant_select ON multica.task_metadata
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_metadata_tenant_insert ON multica.task_metadata;
CREATE POLICY task_metadata_tenant_insert ON multica.task_metadata
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_metadata_tenant_update ON multica.task_metadata;
CREATE POLICY task_metadata_tenant_update ON multica.task_metadata
    FOR UPDATE
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.work_item_worktree ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.work_item_worktree FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS work_item_worktree_tenant_select ON multica.work_item_worktree;
CREATE POLICY work_item_worktree_tenant_select ON multica.work_item_worktree
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS work_item_worktree_tenant_insert ON multica.work_item_worktree;
CREATE POLICY work_item_worktree_tenant_insert ON multica.work_item_worktree
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS work_item_worktree_tenant_update ON multica.work_item_worktree;
CREATE POLICY work_item_worktree_tenant_update ON multica.work_item_worktree
    FOR UPDATE
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_lifecycle_current ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_lifecycle_current FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_lifecycle_current_tenant_scope ON multica.task_lifecycle_current;
CREATE POLICY task_lifecycle_current_tenant_scope ON multica.task_lifecycle_current
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_command_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_command_idempotency FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_command_idempotency_tenant_scope ON multica.task_command_idempotency;
CREATE POLICY task_command_idempotency_tenant_scope ON multica.task_command_idempotency
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.task_lifecycle_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_lifecycle_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_lifecycle_audit_tenant_select ON multica.task_lifecycle_audit;
CREATE POLICY task_lifecycle_audit_tenant_select ON multica.task_lifecycle_audit
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_lifecycle_audit_tenant_insert ON multica.task_lifecycle_audit;
CREATE POLICY task_lifecycle_audit_tenant_insert ON multica.task_lifecycle_audit
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.task_metadata IS
    'Canonical WorkItem master metadata; all task views share work_item_id. Master/SCD2.';
COMMENT ON TABLE multica.work_item_worktree IS
    'Explicit WorkItem-to-Worktree association; one task may be associated with multiple Worktrees.';
COMMENT ON TABLE multica.task_lifecycle_current IS
    'Multica six-state current lifecycle projection; rebuildable from append-only audit.';
COMMENT ON TABLE multica.task_command_idempotency IS
    'Temporary command replay response; Work with bounded 30-day retention.';
COMMENT ON TABLE multica.task_lifecycle_audit IS
    'Append-only task lifecycle Transaction history; physical UPDATE/DELETE prohibited.';

COMMIT;
