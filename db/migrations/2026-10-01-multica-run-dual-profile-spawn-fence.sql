-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-10-01-multica-run-dual-profile-spawn-fence.sql",type:"file",language:"sql"}),
--   (m:Module {name:"multica_run_dual_profile_spawn_fence",type:"module",language:"sql"}),
--   (run:Class {name:"multica.task_execution_run",type:"class",language:"sql"}),
--   (launch_id:Variable {name:"approved_launch_profile_id",type:"variable",language:"sql"}),
--   (launch_version:Variable {name:"approved_launch_profile_version",type:"variable",language:"sql"}),
--   (launch_digest:Variable {name:"approved_launch_profile_digest",type:"variable",language:"sql"}),
--   (fence_digest:Variable {name:"spawn_fence_binding_digest",type:"variable",language:"sql"}),
--   (identity_guard:Logic {name:"ck_task_execution_run_dual_profile_spawn_fence_complete",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:GUARDS]->(run),(m)-[:DECLARES]->(launch_id),
--   (m)-[:DECLARES]->(launch_version),(m)-[:DECLARES]->(launch_digest),
--   (m)-[:DECLARES]->(fence_digest),(m)-[:DECLARES]->(identity_guard),(identity_guard)-[:GUARDS]->(run);

-- Phase 9E-4C4: persist the Approved Launch Profile identity used by the dual-Profile spawn fence.
-- Run rows remain immutable transaction facts; legacy and pre-C4 Run rows may retain all-NULL values.
BEGIN;

ALTER TABLE multica.task_execution_run
    ADD COLUMN IF NOT EXISTS approved_launch_profile_id UUID,
    ADD COLUMN IF NOT EXISTS approved_launch_profile_version BIGINT,
    ADD COLUMN IF NOT EXISTS approved_launch_profile_digest CHAR(64),
    ADD COLUMN IF NOT EXISTS spawn_fence_binding_digest CHAR(64);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.task_execution_run'::regclass
           AND conname = 'ck_task_execution_run_dual_profile_spawn_fence_complete'
    ) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT ck_task_execution_run_dual_profile_spawn_fence_complete
            CHECK (
                (approved_launch_profile_id IS NULL
                    AND approved_launch_profile_version IS NULL
                    AND approved_launch_profile_digest IS NULL
                    AND spawn_fence_binding_digest IS NULL)
                OR
                (approved_launch_profile_id IS NOT NULL
                    AND approved_launch_profile_version IS NOT NULL
                    AND approved_launch_profile_version > 0
                    AND approved_launch_profile_digest IS NOT NULL
                    AND approved_launch_profile_digest ~ '^[0-9a-f]{64}$'
                    AND spawn_fence_binding_digest IS NOT NULL
                    AND spawn_fence_binding_digest ~ '^[0-9a-f]{64}$')
            );
    END IF;
END;
$$;

COMMENT ON COLUMN multica.task_execution_run.approved_launch_profile_id IS
    'Approved executable/argv/cwd policy identity resolved by Runtime for this immutable Run.';
COMMENT ON COLUMN multica.task_execution_run.approved_launch_profile_version IS
    'Immutable Approved Launch Profile revision attested by the consumed one-time spawn fence.';
COMMENT ON COLUMN multica.task_execution_run.approved_launch_profile_digest IS
    'Lowercase SHA-256 digest paired with approved_launch_profile_id/version; NULL only for pre-C4 Runs.';
COMMENT ON COLUMN multica.task_execution_run.spawn_fence_binding_digest IS
    'Versioned SHA-256 over request fingerprint, both Profile identities, catalog revisions, HookSet, scope, and resource budget; never stores the opaque one-time fence ID.';
COMMENT ON CONSTRAINT ck_task_execution_run_dual_profile_spawn_fence_complete
    ON multica.task_execution_run IS
    'Approved Launch Profile identity and one-time spawn-fence digest are either both absent for legacy/pre-C4 Runs or complete and digest-validated.';

COMMIT;
