-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-10-01-multica-run-profile-snapshot-guards.sql",type:"file",language:"sql"}),
--   (m:Module {name:"multica_run_profile_snapshot_guards",type:"module",language:"sql"}),
--   (run:Class {name:"multica.task_execution_run",type:"class",language:"sql"}),
--   (tuple_guard:Logic {name:"ck_task_execution_run_profile_snapshot_complete",type:"logic",language:"sql"}),
--   (scope_guard:Logic {name:"ck_task_execution_run_profile_snapshot_scope",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:GUARDS]->(run),
--   (m)-[:DECLARES]->(tuple_guard),(m)-[:DECLARES]->(scope_guard),
--   (tuple_guard)-[:GUARDS]->(run),(scope_guard)-[:GUARDS]->(run);

-- Phase 9E-4A: enforce a complete, scope-bound Run Profile snapshot without a Profile FK.
-- Legacy Runs with no Profile remain valid; historical snapshots do not depend on Master rows.
BEGIN;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
          FROM pg_constraint
         WHERE conrelid = 'multica.task_execution_run'::regclass
           AND conname = 'ck_task_execution_run_profile_snapshot_complete'
    ) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT ck_task_execution_run_profile_snapshot_complete
            CHECK (
                (execution_profile_id IS NULL
                    AND execution_profile_version IS NULL
                    AND execution_profile_digest IS NULL
                    AND execution_profile_snapshot IS NULL)
                OR
                (execution_profile_id IS NOT NULL
                    AND execution_profile_version IS NOT NULL
                    AND execution_profile_digest IS NOT NULL
                    AND execution_profile_snapshot IS NOT NULL)
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1
          FROM pg_constraint
         WHERE conrelid = 'multica.task_execution_run'::regclass
           AND conname = 'ck_task_execution_run_profile_snapshot_scope'
    ) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT ck_task_execution_run_profile_snapshot_scope
            CHECK (
                execution_profile_snapshot IS NULL
                OR (
                    execution_profile_snapshot #>> '{profile,scope,tenant_id}'
                        IS NOT DISTINCT FROM tenant_id::text
                    AND execution_profile_snapshot #>> '{profile,scope,project_id}'
                        IS NOT DISTINCT FROM project_id::text
                    AND (
                        execution_profile_snapshot #>> '{profile,scope,worktree_id}' IS NULL
                        OR execution_profile_snapshot #>> '{profile,scope,worktree_id}'
                            IS NOT DISTINCT FROM worktree_id::text
                    )
                    AND execution_profile_snapshot ->> 'content_digest'
                        IS NOT DISTINCT FROM execution_profile_digest::text
                )
            );
    END IF;
END;
$$;

COMMENT ON CONSTRAINT ck_task_execution_run_profile_snapshot_complete
    ON multica.task_execution_run IS
    'A Run stores either no AgentExecutionProfile reference or a complete ID/version/digest/document snapshot tuple.';
COMMENT ON CONSTRAINT ck_task_execution_run_profile_snapshot_scope
    ON multica.task_execution_run IS
    'The self-contained Profile document tenant/project/optional Worktree scope and digest must match the immutable Run envelope; no Profile foreign key is used.';

COMMIT;