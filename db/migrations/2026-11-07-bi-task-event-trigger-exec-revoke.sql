-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-11-07-bi-task-event-trigger-exec-revoke.sql"})
-- MERGE (module:Symbol {id:"db/migrations/2026-11-07-bi-task-event-trigger-exec-revoke.sql::bi_task_event_trigger_exec_revoke_migration",kind:"module"})
-- MERGE (reject:Database {id:"bi.reject_task_event_mutation",kind:"trigger_function"})
-- MERGE (rejectQuarantine:Database {id:"bi.reject_task_event_quarantine_mutation",kind:"trigger_function"})
-- MERGE (self)-[:DEFINES]->(module)
-- MERGE (module)-[:CONFIGURES]->(reject)
-- MERGE (module)-[:CONFIGURES]->(rejectQuarantine)
-- @endcypher

-- Trigger execution is checked when a trigger is created, not when it fires.
-- Keep these append-only guard functions private while preserving their owners and triggers.
REVOKE EXECUTE ON FUNCTION bi.reject_task_event_mutation() FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION bi.reject_task_event_quarantine_mutation() FROM PUBLIC;
