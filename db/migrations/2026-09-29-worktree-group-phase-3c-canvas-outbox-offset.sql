-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-3c-canvas-outbox-offset.sql",type:"file",language:"sql"}),
--   (outbox:Class {name:"canvas.canvas_group_outbox",type:"class"}),
--   (offset:Class {name:"stream_offset",type:"column"}),
--   (scopeIndex:Class {name:"idx_canvas_group_outbox_scope_offset",type:"index"}),
--   (f)-[:ALTERS]->(outbox),(outbox)-[:HAS_COLUMN]->(offset),(outbox)-[:HAS_INDEX]->(scopeIndex);
-- Worktree Group Phase 3C: additive commit-ordered Canvas Outbox cursor.
-- W/T: the Outbox remains an append-only Transaction table.
-- Every event writer must take the same tenant/worktree transaction advisory lock before insert.

BEGIN;

ALTER TABLE canvas.canvas_group_outbox
    ADD COLUMN IF NOT EXISTS stream_offset BIGINT GENERATED ALWAYS AS IDENTITY;

CREATE UNIQUE INDEX IF NOT EXISTS idx_canvas_group_outbox_scope_offset
    ON canvas.canvas_group_outbox (tenant_id, worktree_id, stream_offset);

COMMENT ON COLUMN canvas.canvas_group_outbox.stream_offset IS
    'Per-Worktree consumer cursor. Writers serialize inserts with a tenant/worktree transaction advisory lock so allocated offsets are commit ordered.';

COMMIT;
