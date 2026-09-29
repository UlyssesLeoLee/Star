-- Worktree Group Phase 2B: persisted project membership and authoritative Worktree links.
-- Additive only. Requires the existing worktree_canvas_worktree table.
-- W/T/M: project_role_binding is Master with SCD2 validity; no Work table is introduced here.

BEGIN;

CREATE SCHEMA IF NOT EXISTS permission;
CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS permission.project_role_binding (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role VARCHAR(32) NOT NULL CHECK (role IN ('tenant_admin', 'project_admin', 'developer', 'viewer', 'agent')),
    granted_by UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ NULL,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_project_role_binding_validity CHECK (valid_to IS NULL OR valid_to >= valid_from)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_project_role_binding_active_subject
    ON permission.project_role_binding (tenant_id, project_id, user_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_project_role_binding_active_user
    ON permission.project_role_binding (tenant_id, user_id, project_id)
    WHERE valid_to IS NULL;

ALTER TABLE permission.project_role_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE permission.project_role_binding FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS project_role_binding_tenant_isolation ON permission.project_role_binding;
DROP POLICY IF EXISTS project_role_binding_tenant_select ON permission.project_role_binding;
CREATE POLICY project_role_binding_tenant_select ON permission.project_role_binding
    FOR SELECT
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS project_role_binding_tenant_insert ON permission.project_role_binding;
CREATE POLICY project_role_binding_tenant_insert ON permission.project_role_binding
    FOR INSERT
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS project_role_binding_tenant_update ON permission.project_role_binding;
CREATE POLICY project_role_binding_tenant_update ON permission.project_role_binding
    FOR UPDATE
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE worktree_canvas_worktree
    ADD COLUMN IF NOT EXISTS project_id UUID NULL,
    ADD COLUMN IF NOT EXISTS owner_user_id UUID NULL,
    ADD COLUMN IF NOT EXISTS work_item_id UUID NULL,
    ADD COLUMN IF NOT EXISTS agent_session_id UUID NULL,
    ADD COLUMN IF NOT EXISTS runtime_id UUID NULL,
    ADD COLUMN IF NOT EXISTS pull_request_url TEXT NULL,
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS retention_period INTERVAL NOT NULL DEFAULT INTERVAL '365 days',
    ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ NULL;

CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_project_index
    ON worktree_canvas_worktree (tenant_id, project_id, updated_at DESC, id)
    WHERE project_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS worktree_canvas_worktree_tenant_id_unique
    ON worktree_canvas_worktree (tenant_id, id);

CREATE TABLE IF NOT EXISTS multica.worktree_project_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    assigned_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_worktree_project_binding_active
    ON multica.worktree_project_binding (tenant_id, worktree_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_worktree_project_binding_project
    ON multica.worktree_project_binding (tenant_id, project_id, worktree_id, valid_from DESC);
ALTER TABLE multica.worktree_project_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.worktree_project_binding FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS worktree_project_binding_tenant_select ON multica.worktree_project_binding;
CREATE POLICY worktree_project_binding_tenant_select ON multica.worktree_project_binding
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS worktree_project_binding_tenant_insert ON multica.worktree_project_binding;
CREATE POLICY worktree_project_binding_tenant_insert ON multica.worktree_project_binding
    FOR INSERT WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS worktree_project_binding_tenant_update ON multica.worktree_project_binding;
CREATE POLICY worktree_project_binding_tenant_update ON multica.worktree_project_binding
    FOR UPDATE
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
CREATE INDEX IF NOT EXISTS worktree_canvas_worktree_project_owner
    ON worktree_canvas_worktree (tenant_id, project_id, owner_user_id)
    WHERE project_id IS NOT NULL;

COMMENT ON TABLE permission.project_role_binding IS
    'Project membership Master, SCD Type 2; revoke by setting valid_to, never physically delete.';
COMMENT ON TABLE multica.worktree_project_binding IS
    'Authoritative Project-to-Worktree Master/SCD2 relation; worktree_canvas_worktree.project_id is a read projection.';
COMMENT ON COLUMN permission.project_role_binding.valid_to IS
    'NULL means active binding; non-NULL preserves revocation history.';
COMMENT ON COLUMN worktree_canvas_worktree.project_id IS
    'Authoritative project scope for Worktree Index and GroupContext; null rows are excluded until reconciled.';
COMMENT ON COLUMN worktree_canvas_worktree.owner_user_id IS
    'Human owner projection; null means unknown and must not be inferred from agent_id.';
COMMENT ON COLUMN worktree_canvas_worktree.work_item_id IS
    'Canonical WorkItem ID; not backfilled from legacy task_id without reconciliation evidence.';
COMMENT ON COLUMN worktree_canvas_worktree.version IS
    'Optimistic concurrency version for Worktree management commands.';
COMMENT ON COLUMN worktree_canvas_worktree.retention_period IS
    'W retention policy; active Worktrees have no expiry and expiry is set only after an approved lifecycle transition.';
COMMENT ON COLUMN worktree_canvas_worktree.expires_at IS
    'Nullable Work retention deadline; no expiry is inferred for existing Worktrees.';

COMMIT;
