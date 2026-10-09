-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-09-task-run-admission-attempt-lifecycle.sql"})
-- MERGE (run:Table {id:"multica.task_execution_run"})
-- MERGE (event:Table {id:"multica.task_execution_run_event"})
-- MERGE (self)-[:CONFIGURES]->(run)
-- MERGE (self)-[:CONFIGURES]->(event)
-- @endcypher

-- Phase 9F4C-D1: model retryable logical admission attempts and provisional Runs.
-- Existing immutable Run facts are not rewritten. Legacy Schedule rows with no attempt
-- ordinal are treated as attempt 1 by the unique expression index.
BEGIN;

DO $precondition$
BEGIN
    IF to_regclass('multica.task_execution_run') IS NULL
       OR to_regclass('multica.task_execution_run_event') IS NULL THEN
        RAISE EXCEPTION 'TaskExecutionRun prerequisite migration must be applied first';
    END IF;
END
$precondition$;

ALTER TABLE multica.task_execution_run
    ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS automation_admission_attempt INTEGER;

ALTER TABLE multica.task_execution_run
    ALTER COLUMN created_at SET DEFAULT clock_timestamp(),
    ALTER COLUMN started_at DROP NOT NULL,
    ALTER COLUMN started_at DROP DEFAULT;

ALTER TABLE multica.task_execution_run
    DROP CONSTRAINT IF EXISTS ck_task_execution_run_admission_attempt,
    ADD CONSTRAINT ck_task_execution_run_admission_attempt
        CHECK (automation_admission_attempt IS NULL OR
               (run_origin = 'schedule' AND automation_admission_attempt > 0)),
    DROP CONSTRAINT IF EXISTS ck_task_execution_run_recorded_or_started,
    ADD CONSTRAINT ck_task_execution_run_recorded_or_started
        CHECK (created_at IS NOT NULL OR started_at IS NOT NULL),
    DROP CONSTRAINT IF EXISTS ck_task_execution_run_non_schedule_started,
    ADD CONSTRAINT ck_task_execution_run_non_schedule_started
        CHECK (run_origin = 'schedule' OR started_at IS NOT NULL),
    DROP CONSTRAINT IF EXISTS ck_task_execution_run_schedule_provisional_attempt,
    ADD CONSTRAINT ck_task_execution_run_schedule_provisional_attempt
        CHECK (run_origin <> 'schedule' OR started_at IS NOT NULL OR
               automation_admission_attempt IS NOT NULL);

DROP INDEX IF EXISTS multica.idx_task_execution_run_task_time;
CREATE INDEX idx_task_execution_run_task_time
    ON multica.task_execution_run
       (tenant_id, project_id, work_item_id,
        (COALESCE(started_at, created_at)) DESC, run_id DESC);

DROP INDEX IF EXISTS multica.idx_task_execution_run_worktree_time;
CREATE INDEX idx_task_execution_run_worktree_time
    ON multica.task_execution_run
       (tenant_id, project_id, worktree_id,
        (COALESCE(started_at, created_at)) DESC)
    WHERE worktree_id IS NOT NULL;

DROP INDEX IF EXISTS multica.uq_task_execution_run_schedule_occurrence;
CREATE UNIQUE INDEX IF NOT EXISTS uq_task_execution_run_schedule_occurrence_attempt
    ON multica.task_execution_run
       (tenant_id, automation_occurrence_id,
        (COALESCE(automation_admission_attempt, 1)))
    WHERE automation_occurrence_id IS NOT NULL;

ALTER TABLE multica.task_execution_run_event
    DROP CONSTRAINT IF EXISTS task_execution_run_event_event_type_check,
    DROP CONSTRAINT IF EXISTS ck_task_execution_run_event_type_v2,
    DROP CONSTRAINT IF EXISTS ck_task_execution_run_event_type_v3,
    ADD CONSTRAINT ck_task_execution_run_event_type_v3
        CHECK (event_type IN (
            'run_started','execution_state_changed','agent_declaration','validation_result',
            'human_review','integration_state_changed','human_intervention','cost_recorded',
            'failure_classified','evidence_attached','resource_summary',
            'resource_budget_reserved','resource_reservation_activated','resource_reservation_released',
            'schedule_occurrence_linked','schedule_admission_confirmed',
            'schedule_admission_compensated',
            'loop_iteration_started','loop_decision_recorded','loop_stopped',
            'hook_evaluated','hook_override_recorded'
        ));

COMMENT ON COLUMN multica.task_execution_run.created_at IS
    'Run-owner record creation time. Schedule provisional admission sets this while started_at remains NULL.';
COMMENT ON COLUMN multica.task_execution_run.started_at IS
    'Actual execution start time; provisional Schedule admission must leave this NULL until run_started is recorded.';
COMMENT ON COLUMN multica.task_execution_run.automation_admission_attempt IS
    'Positive logical admission attempt ordinal for Schedule-origin Runs; transport redelivery keeps the same ordinal.';
COMMENT ON INDEX multica.uq_task_execution_run_schedule_occurrence_attempt IS
    'One immutable Run per Schedule occurrence and logical admission attempt; legacy null ordinal is interpreted as attempt 1.';
COMMENT ON CONSTRAINT ck_task_execution_run_event_type_v3
    ON multica.task_execution_run_event IS
    'Append-only Run event vocabulary; Schedule confirmation and compensation are distinct from actual run_started.';

COMMIT;
