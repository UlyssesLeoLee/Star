-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-3-canvas.sql",type:"file",language:"sql"}),
--   (registry:Class {name:"canvas.group_canvas_registry",type:"class"}),
--   (elements:Class {name:"canvas.canvas_elements_backend",type:"class"}),
--   (refs:Class {name:"canvas.canvas_entity_ref",type:"class"}),
--   (audit:Class {name:"canvas.canvas_group_audit",type:"class"}),
--   (outbox:Class {name:"canvas.canvas_group_outbox",type:"class"}),
--   (guard:Function {name:"canvas.reject_group_append_only_mutation",type:"function",signature:"trigger()",visibility:"private"}),
--   (f)-[:CONTAINS]->(registry),(f)-[:CONTAINS]->(elements),(f)-[:CONTAINS]->(refs),
--   (f)-[:CONTAINS]->(audit),(f)-[:CONTAINS]->(outbox),(f)-[:CONTAINS]->(guard);
-- Worktree Group Phase 3 Canvas persistence boundary.
-- Additive only; apply after Phase 2B/2C/2D migrations and project/worktree bindings.
-- W/T/M: registry, elements, EntityRef are Master/SCD2; audit and outbox are Transaction/append-only.

BEGIN;

CREATE SCHEMA IF NOT EXISTS canvas;

CREATE TABLE IF NOT EXISTS canvas.group_canvas_registry (
    registry_row_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    canvas_id UUID NOT NULL,
    version BIGINT NOT NULL CHECK (version > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT fk_group_canvas_worktree FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CONSTRAINT ck_group_canvas_validity CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT ck_group_canvas_current CHECK
        ((is_current AND valid_to IS NULL) OR (NOT is_current AND valid_to IS NOT NULL)),
    CONSTRAINT uq_group_canvas_version UNIQUE (tenant_id, worktree_id, canvas_id, version)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_group_canvas_current
    ON canvas.group_canvas_registry (tenant_id, worktree_id, canvas_id) WHERE is_current;
CREATE UNIQUE INDEX IF NOT EXISTS uq_group_canvas_single_owner_current
    ON canvas.group_canvas_registry (tenant_id, canvas_id) WHERE is_current;
CREATE INDEX IF NOT EXISTS idx_group_canvas_worktree_current
    ON canvas.group_canvas_registry (tenant_id, worktree_id, created_at DESC, canvas_id)
    WHERE is_current;

CREATE TABLE IF NOT EXISTS canvas.canvas_elements_backend (
    element_row_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    canvas_id UUID NOT NULL,
    element_id UUID NOT NULL,
    kind VARCHAR(32) NOT NULL CHECK (kind IN
        ('sticky_note','text','shape','image','embed','work_item_card','worktree_node',
         'agent_cursor','automation_node','comment_pin')),
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    width DOUBLE PRECISION NOT NULL CHECK (width > 0 AND width <= 10000),
    height DOUBLE PRECISION NOT NULL CHECK (height > 0 AND height <= 10000),
    rotation DOUBLE PRECISION NOT NULL DEFAULT 0,
    z_index INTEGER NOT NULL DEFAULT 0,
    content JSONB NOT NULL DEFAULT '{}'::JSONB,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    hidden BOOLEAN NOT NULL DEFAULT FALSE,
    version BIGINT NOT NULL CHECK (version > 0),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT fk_canvas_element_worktree FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CONSTRAINT ck_canvas_element_validity CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT ck_canvas_element_current CHECK
        ((is_current AND valid_to IS NULL) OR (NOT is_current AND valid_to IS NOT NULL)),
    CONSTRAINT uq_canvas_element_version UNIQUE
        (tenant_id, worktree_id, canvas_id, element_id, version)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_canvas_element_current
    ON canvas.canvas_elements_backend (tenant_id, worktree_id, canvas_id, element_id)
    WHERE is_current;
CREATE INDEX IF NOT EXISTS idx_canvas_elements_current
    ON canvas.canvas_elements_backend (tenant_id, worktree_id, canvas_id, z_index, element_id)
    WHERE is_current AND NOT hidden;

CREATE TABLE IF NOT EXISTS canvas.canvas_entity_ref (
    ref_row_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    canvas_id UUID NOT NULL,
    element_id UUID NOT NULL,
    ref_type VARCHAR(32) NOT NULL CHECK (ref_type IN ('work_item','worktree')),
    ref_id UUID NOT NULL,
    version BIGINT NOT NULL CHECK (version > 0),
    created_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT fk_canvas_entity_ref_worktree FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CONSTRAINT ck_canvas_entity_ref_validity CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT ck_canvas_entity_ref_current CHECK
        ((is_current AND valid_to IS NULL) OR (NOT is_current AND valid_to IS NOT NULL)),
    CONSTRAINT uq_canvas_entity_ref_version UNIQUE
        (tenant_id, worktree_id, canvas_id, element_id, version)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_canvas_entity_ref_current
    ON canvas.canvas_entity_ref (tenant_id, worktree_id, canvas_id, element_id)
    WHERE is_current;
CREATE INDEX IF NOT EXISTS idx_canvas_entity_ref_target_current
    ON canvas.canvas_entity_ref (tenant_id, worktree_id, ref_type, ref_id, canvas_id)
    WHERE is_current;

CREATE TABLE IF NOT EXISTS canvas.canvas_group_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    canvas_id UUID NOT NULL,
    element_id UUID,
    event_type VARCHAR(48) NOT NULL,
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::JSONB,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_canvas_group_audit_worktree FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_canvas_group_audit_scope_time
    ON canvas.canvas_group_audit (tenant_id, worktree_id, canvas_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS canvas.canvas_group_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    canvas_id UUID NOT NULL,
    element_id UUID,
    event_type VARCHAR(64) NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    aggregate_version BIGINT NOT NULL CHECK (aggregate_version > 0),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_canvas_group_outbox_worktree FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_canvas_group_outbox_scope_time
    ON canvas.canvas_group_outbox (tenant_id, worktree_id, occurred_at, event_id);

CREATE OR REPLACE FUNCTION canvas.reject_group_append_only_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'canvas group Transaction rows are append-only' USING ERRCODE = '55000';
END;
$$;
DROP TRIGGER IF EXISTS canvas_group_audit_immutable ON canvas.canvas_group_audit;
CREATE TRIGGER canvas_group_audit_immutable
    BEFORE UPDATE OR DELETE ON canvas.canvas_group_audit
    FOR EACH ROW EXECUTE FUNCTION canvas.reject_group_append_only_mutation();
DROP TRIGGER IF EXISTS canvas_group_outbox_immutable ON canvas.canvas_group_outbox;
CREATE TRIGGER canvas_group_outbox_immutable
    BEFORE UPDATE OR DELETE ON canvas.canvas_group_outbox
    FOR EACH ROW EXECUTE FUNCTION canvas.reject_group_append_only_mutation();

ALTER TABLE canvas.group_canvas_registry ENABLE ROW LEVEL SECURITY;
ALTER TABLE canvas.group_canvas_registry FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_canvas_registry_select_scope ON canvas.group_canvas_registry;
CREATE POLICY group_canvas_registry_select_scope ON canvas.group_canvas_registry
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS group_canvas_registry_insert_scope ON canvas.group_canvas_registry;
CREATE POLICY group_canvas_registry_insert_scope ON canvas.group_canvas_registry
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS group_canvas_registry_update_scope ON canvas.group_canvas_registry;
CREATE POLICY group_canvas_registry_update_scope ON canvas.group_canvas_registry
    FOR UPDATE USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    ) WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS group_canvas_registry_no_delete ON canvas.group_canvas_registry;
CREATE POLICY group_canvas_registry_no_delete ON canvas.group_canvas_registry
    FOR DELETE USING (FALSE);

ALTER TABLE canvas.canvas_elements_backend ENABLE ROW LEVEL SECURITY;
ALTER TABLE canvas.canvas_elements_backend FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS canvas_elements_backend_select_scope ON canvas.canvas_elements_backend;
CREATE POLICY canvas_elements_backend_select_scope ON canvas.canvas_elements_backend
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_elements_backend_insert_scope ON canvas.canvas_elements_backend;
CREATE POLICY canvas_elements_backend_insert_scope ON canvas.canvas_elements_backend
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_elements_backend_update_scope ON canvas.canvas_elements_backend;
CREATE POLICY canvas_elements_backend_update_scope ON canvas.canvas_elements_backend
    FOR UPDATE USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    ) WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_elements_backend_no_delete ON canvas.canvas_elements_backend;
CREATE POLICY canvas_elements_backend_no_delete ON canvas.canvas_elements_backend
    FOR DELETE USING (FALSE);

ALTER TABLE canvas.canvas_entity_ref ENABLE ROW LEVEL SECURITY;
ALTER TABLE canvas.canvas_entity_ref FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS canvas_entity_ref_select_scope ON canvas.canvas_entity_ref;
CREATE POLICY canvas_entity_ref_select_scope ON canvas.canvas_entity_ref
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_entity_ref_insert_scope ON canvas.canvas_entity_ref;
CREATE POLICY canvas_entity_ref_insert_scope ON canvas.canvas_entity_ref
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_entity_ref_update_scope ON canvas.canvas_entity_ref;
CREATE POLICY canvas_entity_ref_update_scope ON canvas.canvas_entity_ref
    FOR UPDATE USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    ) WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_entity_ref_no_delete ON canvas.canvas_entity_ref;
CREATE POLICY canvas_entity_ref_no_delete ON canvas.canvas_entity_ref
    FOR DELETE USING (FALSE);

ALTER TABLE canvas.canvas_group_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE canvas.canvas_group_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS canvas_group_audit_select_scope ON canvas.canvas_group_audit;
CREATE POLICY canvas_group_audit_select_scope ON canvas.canvas_group_audit
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_group_audit_insert_scope ON canvas.canvas_group_audit;
CREATE POLICY canvas_group_audit_insert_scope ON canvas.canvas_group_audit
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_group_audit_no_update ON canvas.canvas_group_audit;
CREATE POLICY canvas_group_audit_no_update ON canvas.canvas_group_audit
    FOR UPDATE USING (FALSE);
DROP POLICY IF EXISTS canvas_group_audit_no_delete ON canvas.canvas_group_audit;
CREATE POLICY canvas_group_audit_no_delete ON canvas.canvas_group_audit
    FOR DELETE USING (FALSE);

ALTER TABLE canvas.canvas_group_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE canvas.canvas_group_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS canvas_group_outbox_select_scope ON canvas.canvas_group_outbox;
CREATE POLICY canvas_group_outbox_select_scope ON canvas.canvas_group_outbox
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_group_outbox_insert_scope ON canvas.canvas_group_outbox;
CREATE POLICY canvas_group_outbox_insert_scope ON canvas.canvas_group_outbox
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_group_outbox_no_update ON canvas.canvas_group_outbox;
CREATE POLICY canvas_group_outbox_no_update ON canvas.canvas_group_outbox
    FOR UPDATE USING (FALSE);
DROP POLICY IF EXISTS canvas_group_outbox_no_delete ON canvas.canvas_group_outbox;
CREATE POLICY canvas_group_outbox_no_delete ON canvas.canvas_group_outbox
    FOR DELETE USING (FALSE);

COMMENT ON TABLE canvas.group_canvas_registry IS
    'Master/SCD2 Worktree-owned Infinite Canvas registry; project/free legacy Canvas rows are not auto-bound.';
COMMENT ON TABLE canvas.canvas_elements_backend IS
    'Master/SCD2 Canvas element layout; entity identity is stored separately in canvas_entity_ref.';
COMMENT ON TABLE canvas.canvas_entity_ref IS
    'Master/SCD2 typed EntityRef; current Group API supports canonical WorkItem and current Worktree refs.';
COMMENT ON TABLE canvas.canvas_group_audit IS
    'Transaction/append-only audit for Group Canvas commands.';
COMMENT ON TABLE canvas.canvas_group_outbox IS
    'Transaction/append-only event outbox for Group Canvas projections and realtime subscribers.';

COMMIT;
