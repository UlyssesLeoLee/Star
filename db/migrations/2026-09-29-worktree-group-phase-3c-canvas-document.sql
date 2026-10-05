-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-3c-canvas-document.sql",type:"file",language:"sql"}),
--   (document:Class {name:"canvas.canvas_document_state",type:"class"}),
--   (version:Class {name:"document_version",type:"column"}),
--   (viewport:Class {name:"viewport",type:"column"}),
--   (frames:Class {name:"frames",type:"column"}),
--   (connectors:Class {name:"connectors",type:"column"}),
--   (backfill:Logic {name:"backfill_current_canvas_documents",type:"logic"}),
--   (f)-[:CREATES]->(document),
--   (document)-[:HAS_COLUMN]->(version),
--   (document)-[:HAS_COLUMN]->(viewport),
--   (document)-[:HAS_COLUMN]->(frames),
--   (document)-[:HAS_COLUMN]->(connectors),
--   (f)-[:CONTAINS]->(backfill);
-- Worktree Group Phase 3C: versioned Canvas Document persistence.
-- W/T/M: Canvas Document is Master/SCD2; audit and outbox remain Transaction append-only.

BEGIN;

CREATE TABLE IF NOT EXISTS canvas.canvas_document_state (
    document_row_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    canvas_id UUID NOT NULL,
    document_version BIGINT NOT NULL CHECK (document_version > 0),
    viewport JSONB NOT NULL DEFAULT '{"x":0,"y":0,"zoom":1}'::JSONB
        CHECK (jsonb_typeof(viewport) = 'object'),
    frames JSONB NOT NULL DEFAULT '[]'::JSONB
        CHECK (jsonb_typeof(frames) = 'array'),
    connectors JSONB NOT NULL DEFAULT '[]'::JSONB
        CHECK (jsonb_typeof(connectors) = 'array'),
    updated_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE,
    CONSTRAINT fk_canvas_document_worktree FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CONSTRAINT ck_canvas_document_validity CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT ck_canvas_document_current CHECK
        ((is_current AND valid_to IS NULL) OR (NOT is_current AND valid_to IS NOT NULL)),
    CONSTRAINT uq_canvas_document_version UNIQUE
        (tenant_id, worktree_id, canvas_id, document_version)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_canvas_document_current
    ON canvas.canvas_document_state (tenant_id, worktree_id, canvas_id)
    WHERE is_current;
CREATE INDEX IF NOT EXISTS idx_canvas_document_history
    ON canvas.canvas_document_state (tenant_id, worktree_id, canvas_id, document_version DESC);

INSERT INTO canvas.canvas_document_state
    (tenant_id, worktree_id, canvas_id, document_version, updated_by)
SELECT tenant_id, worktree_id, canvas_id, 1, created_by
FROM canvas.group_canvas_registry
WHERE is_current AND valid_to IS NULL
ON CONFLICT DO NOTHING;

ALTER TABLE canvas.canvas_document_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE canvas.canvas_document_state FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS canvas_document_select_scope ON canvas.canvas_document_state;
CREATE POLICY canvas_document_select_scope ON canvas.canvas_document_state
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_document_insert_scope ON canvas.canvas_document_state;
CREATE POLICY canvas_document_insert_scope ON canvas.canvas_document_state
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_document_update_scope ON canvas.canvas_document_state;
CREATE POLICY canvas_document_update_scope ON canvas.canvas_document_state
    FOR UPDATE USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    ) WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND worktree_id = NULLIF(current_setting('app.worktree_id', true), '')::UUID
    );
DROP POLICY IF EXISTS canvas_document_no_delete ON canvas.canvas_document_state;
CREATE POLICY canvas_document_no_delete ON canvas.canvas_document_state
    FOR DELETE USING (FALSE);

COMMENT ON TABLE canvas.canvas_document_state IS
    'Master/SCD2 Worktree Canvas document state. Viewport, frames and visual connectors are versioned together.';
COMMENT ON COLUMN canvas.canvas_document_state.connectors IS
    'Canvas-local visual edges between current element IDs; this is not a canonical Multica WorkItem relation.';

COMMIT;
