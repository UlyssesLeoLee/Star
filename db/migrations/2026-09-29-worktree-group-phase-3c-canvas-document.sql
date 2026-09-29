-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-3c-canvas-document.sql",type:"file",language:"sql"}),
--   (viewport:Class {name:"canvas.group_canvas_registry.viewport",type:"class"}),
--   (frames:Class {name:"canvas.group_canvas_registry.frames",type:"class"}),
--   (connectors:Class {name:"canvas.group_canvas_registry.connectors",type:"class"}),
--   (check:Function {name:"canvas.canvas_document_shape_check",type:"function",signature:"CHECK(jsonb)",visibility:"private"}),
--   (f)-[:CONTAINS]->(viewport),(f)-[:CONTAINS]->(frames),(f)-[:CONTAINS]->(connectors),(f)-[:CONTAINS]->(check);
-- Worktree Group Phase 3C Canvas document persistence.
-- Additive only; apply after 2026-09-29-worktree-group-phase-3-canvas.sql.
-- W/T/M: document layout is Master/SCD2 on the existing Worktree Canvas registry.
-- Frames and connectors are presentation metadata; connector links are not WorkItem relations.

BEGIN;

ALTER TABLE canvas.group_canvas_registry
    ADD COLUMN IF NOT EXISTS viewport JSONB NOT NULL DEFAULT '{"x":0,"y":0,"zoom":1}'::jsonb,
    ADD COLUMN IF NOT EXISTS frames JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS connectors JSONB NOT NULL DEFAULT '[]'::jsonb;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_group_canvas_document_shape'
          AND conrelid = 'canvas.group_canvas_registry'::regclass
    ) THEN
        ALTER TABLE canvas.group_canvas_registry
            ADD CONSTRAINT ck_group_canvas_document_shape CHECK (
                jsonb_typeof(viewport) = 'object'
                AND jsonb_typeof(viewport->'x') = 'number'
                AND jsonb_typeof(viewport->'y') = 'number'
                AND jsonb_typeof(viewport->'zoom') = 'number'
                AND (viewport->>'zoom')::numeric BETWEEN 0.1 AND 4
                AND jsonb_typeof(frames) = 'array'
                AND jsonb_typeof(connectors) = 'array'
            );
    END IF;
END
$$;

COMMENT ON COLUMN canvas.group_canvas_registry.viewport IS
    'Versioned Canvas viewport. Stored in the Worktree-scoped Master/SCD2 registry.';
COMMENT ON COLUMN canvas.group_canvas_registry.frames IS
    'Versioned Canvas frame layout. Element IDs are validated against current elements by the API.';
COMMENT ON COLUMN canvas.group_canvas_registry.connectors IS
    'Versioned visual Canvas connectors; not canonical Jira/WorkItem relation facts.';

COMMIT;
