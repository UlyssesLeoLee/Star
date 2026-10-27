-- CYPHER STRUCTURE MANIFEST
-- CREATE
--   (f:File {name:"2026-10-28-canvas-outbox-wakeup.sql",type:"file",language:"sql"}),
--   (outbox:Table {name:"canvas.canvas_group_outbox",type:"table"}),
--   (notify:Function {name:"canvas.notify_canvas_outbox_wake",type:"function"}),
--   (trigger:Trigger {name:"canvas_group_outbox_wake",type:"trigger"}),
--   (channel:Topic {name:"star_canvas_outbox_wake",type:"topic"}),
--   (worktree:Column {name:"worktree_id",type:"column"}),
--   (f)-[:READS]->(outbox),(f)-[:DEFINES]->(notify),(outbox)-[:HAS_TRIGGER]->(trigger),
--   (trigger)-[:CALLS]->(notify),(notify)-[:EMITS]->(channel),(notify)-[:READS]->(worktree);
-- Notify connected Worktree clients only after an Outbox transaction commits.
-- The payload carries the Worktree UUID only; the append-only Outbox remains the durable truth.

BEGIN;

CREATE OR REPLACE FUNCTION canvas.notify_canvas_outbox_wake()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    PERFORM pg_notify('star_canvas_outbox_wake', NEW.worktree_id::TEXT);
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS canvas_group_outbox_wake ON canvas.canvas_group_outbox;
CREATE TRIGGER canvas_group_outbox_wake
    AFTER INSERT ON canvas.canvas_group_outbox
    FOR EACH ROW
    EXECUTE FUNCTION canvas.notify_canvas_outbox_wake();

COMMENT ON FUNCTION canvas.notify_canvas_outbox_wake() IS
    'Transactional wake-only notification carrying a Worktree UUID; Outbox pagination remains the source of truth.';

COMMIT;
