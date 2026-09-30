-- Cypher structural manifest.
-- CREATE
--   (file:File {name:"2026-10-01-multica-task-run-hook-summary-index.sql",type:"file",language:"sql"}),
--   (module:Module {name:"multica_task_run_hook_summary_index",type:"module",language:"sql"}),
--   (index:Logic {name:"idx_task_execution_run_event_project_hook_time",type:"logic",language:"sql"}),
--   (file)-[:CONTAINS]->(module),(module)-[:CONTAINS]->(index);

-- Phase 9D-4: bound Project/window scans over Run-linked Hook evaluation facts.
BEGIN;

CREATE INDEX IF NOT EXISTS idx_task_execution_run_event_project_hook_time
    ON multica.task_execution_run_event (tenant_id, project_id, occurred_at DESC, event_id)
    WHERE event_type = 'hook_evaluated';

COMMIT;
