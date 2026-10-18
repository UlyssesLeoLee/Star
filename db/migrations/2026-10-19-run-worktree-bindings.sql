-- CYPHER STRUCTURAL MANIFEST
-- CREATE (f:File {name:"2026-10-19-run-worktree-bindings.sql",type:"file",language:"sql"}),
--        (m:Module {name:"run_worktree_binding_outbox_events",type:"module",language:"sql"}),
--        (outbox:Class {name:"multica.engineering_run_outbox",type:"class",language:"sql"}),
--        (eventType:Variable {name:"run_worktree_binding_lifecycle_event_types",type:"variable",language:"sql"}),
--        (constraint:Logic {name:"ck_engineering_run_outbox_event_type",type:"logic",language:"sql"});
-- CREATE (f)-[:CONTAINS]->(m),(m)-[:USES]->(outbox),(m)-[:DEFINES]->(eventType),
--        (m)-[:DEFINES]->(constraint),(constraint)-[:CONSTRAINS]->(eventType),
--        (constraint)-[:APPLIES_TO]->(outbox);

-- Add lifecycle event names for the existing Run-owned binding Outbox.
-- Binding revisions, trigger audit, idempotency, and Outbox insertion stay in
-- the Run directory owner transaction; this does not enable task execution.
BEGIN;

ALTER TABLE multica.engineering_run_outbox
    DROP CONSTRAINT IF EXISTS engineering_run_outbox_event_type_check,
    DROP CONSTRAINT IF EXISTS ck_engineering_run_outbox_event_type;

ALTER TABLE multica.engineering_run_outbox
    ADD CONSTRAINT ck_engineering_run_outbox_event_type
    CHECK (event_type IN (
        'engineering_run.created',
        'engineering_run.worktree_bound',
        'engineering_run.worktree_reassigned',
        'engineering_run.worktree_detached'
    ));

COMMIT;
