-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-3e-canvas-outbox-index.sql",type:"file",language:"sql"}),
--   (ix:Class {name:"idx_canvas_group_outbox_canvas_cursor",type:"class",language:"sql"}),
--   (f)-[:CONTAINS]->(ix);

-- Additive cursor index for the authenticated per-Canvas Outbox polling endpoint.
-- Apply after 2026-09-29-worktree-group-phase-3-canvas.sql.

BEGIN;

CREATE INDEX IF NOT EXISTS idx_canvas_group_outbox_canvas_cursor
    ON canvas.canvas_group_outbox (tenant_id, worktree_id, canvas_id, occurred_at, event_id);

COMMIT;
