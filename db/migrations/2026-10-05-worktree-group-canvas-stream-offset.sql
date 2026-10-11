-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-10-05-worktree-group-canvas-stream-offset.sql",type:"file",language:"sql"}),
--   (outbox:Table {name:"canvas.canvas_group_outbox",type:"table"}),
--   (streamOffset:Column {name:"stream_offset",type:"column"}),
--   (scopeOffset:Index {name:"idx_canvas_group_outbox_scope_offset",type:"index"}),
--   (f)-[:ALTERS]->(outbox),(outbox)-[:HAS_COLUMN]->(streamOffset),(outbox)-[:HAS_INDEX]->(scopeOffset);
-- Worktree Group Phase 3C: ordered, bounded replay cursor for Worktree-scoped Canvas events.
-- Writers serialize stream-offset allocation per tenant/worktree in the same transaction.

BEGIN;

ALTER TABLE canvas.canvas_group_outbox
    ADD COLUMN IF NOT EXISTS stream_offset BIGINT GENERATED ALWAYS AS IDENTITY;

CREATE UNIQUE INDEX IF NOT EXISTS idx_canvas_group_outbox_scope_offset
    ON canvas.canvas_group_outbox (tenant_id, worktree_id, stream_offset);

COMMENT ON COLUMN canvas.canvas_group_outbox.stream_offset IS
    'Per-Worktree Canvas event replay cursor; append_event serializes offset allocation until transaction commit.';

COMMIT;
