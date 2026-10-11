-- CYPHER STRUCTURAL MANIFEST
-- CREATE (f:File {name:"2026-10-02-task-run-engineering-run-binding.sql",type:"file",language:"sql"}),(m:Module {name:"task_run_engineering_run_binding_migration",type:"module",language:"sql"}),(taskRun:Class {name:"multica.task_execution_run",type:"class",language:"sql"}),(directory:Class {name:"multica.engineering_run_worktree_binding",type:"class",language:"sql"}),(branch:Class {name:"multica.engineering_run",type:"class",language:"sql"}),(projectBinding:Class {name:"multica.worktree_project_binding",type:"class",language:"sql"}),(snapshot:Variable {name:"engineering_run_snapshot",type:"variable",language:"sql"}),(guard:Function {name:"multica.guard_cli_task_run_engineering_run_binding",type:"function",language:"sql"});
-- CREATE (f)-[:CONTAINS]->(m),(m)-[:USES]->(taskRun),(m)-[:USES]->(directory),(m)-[:USES]->(branch),(m)-[:USES]->(projectBinding),(taskRun)-[:USES]->(snapshot),(directory)-[:USES]->(branch),(directory)-[:USES]->(projectBinding);
-- CREATE (m)-[:CONTAINS]->(guard),(guard)-[:USES]->(taskRun),(guard)-[:USES]->(snapshot);
-- CREATE (event:Class {name:"multica.task_execution_run_event",type:"class",language:"sql"}),(sessionIndex:Variable {name:"idx_task_run_event_cli_session_binding",type:"variable",language:"sql"}),(m)-[:USES]->(event),(m)-[:CONTAINS]->(sessionIndex),(sessionIndex)-[:USES]->(event);

-- Additive/run-safe. Historic executions keep an all-NULL directory identity and are never
-- assigned to a guessed Engineering Run. Newly admitted CLI runs must write the full tuple.
-- W/T/M: TaskExecutionRun is append-only Transaction; zero Work tables are introduced.
BEGIN;

CREATE UNIQUE INDEX IF NOT EXISTS uq_engineering_run_worktree_binding_run_project_tuple
    ON multica.engineering_run_worktree_binding
       (tenant_id, project_id, repository_id, branch_id, engineering_run_id, worktree_id,
        binding_id, project_binding_id);

ALTER TABLE multica.task_execution_run
    ADD COLUMN IF NOT EXISTS branch_id UUID,
    ADD COLUMN IF NOT EXISTS engineering_run_id UUID,
    ADD COLUMN IF NOT EXISTS worktree_project_binding_id UUID,
    ADD COLUMN IF NOT EXISTS engineering_run_worktree_binding_id UUID,
    ADD COLUMN IF NOT EXISTS engineering_run_snapshot JSONB;

-- Bounded attachment lookup uses only the server-persisted CLI session receipt association.
CREATE INDEX IF NOT EXISTS idx_task_run_event_cli_session_binding
    ON multica.task_execution_run_event
       (tenant_id, project_id, work_item_id, (details ->> 'cli_session_id'), run_id)
    WHERE event_type = 'execution_state_changed' AND details ? 'cli_session_id';

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'ck_task_run_engineering_run_identity_complete'
                   AND conrelid = 'multica.task_execution_run'::regclass) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT ck_task_run_engineering_run_identity_complete CHECK (
                (branch_id IS NULL AND engineering_run_id IS NULL
                 AND worktree_project_binding_id IS NULL
                 AND engineering_run_worktree_binding_id IS NULL
                 AND engineering_run_snapshot IS NULL)
                OR
                (branch_id IS NOT NULL AND engineering_run_id IS NOT NULL
                 AND worktree_project_binding_id IS NOT NULL
                 AND engineering_run_worktree_binding_id IS NOT NULL
                 AND engineering_run_snapshot IS NOT NULL
                 AND repository_id IS NOT NULL AND worktree_id IS NOT NULL
                 AND (execution_channel <> 'cli' OR runtime_id IS NOT NULL)
                 AND engineering_run_id <> run_id
                 AND jsonb_typeof(engineering_run_snapshot) = 'object'
                 AND octet_length(engineering_run_snapshot::TEXT) <= 8192
                 AND (engineering_run_snapshot ->> 'tenant_id' = tenant_id::TEXT
                  AND engineering_run_snapshot ->> 'project_id' = project_id::TEXT
                  AND engineering_run_snapshot ->> 'repository_id' = repository_id::TEXT
                  AND engineering_run_snapshot ->> 'branch_id' = branch_id::TEXT
                  AND engineering_run_snapshot ->> 'engineering_run_id' = engineering_run_id::TEXT
                  AND engineering_run_snapshot ->> 'worktree_id' = worktree_id::TEXT
                  AND engineering_run_snapshot ->> 'work_item_id' = work_item_id::TEXT
                  AND engineering_run_snapshot ->> 'worktree_project_binding_id' = worktree_project_binding_id::TEXT
                  AND engineering_run_snapshot ->> 'engineering_run_worktree_binding_id' = engineering_run_worktree_binding_id::TEXT) IS TRUE)
            );
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_task_run_engineering_run_scope'
                   AND conrelid = 'multica.task_execution_run'::regclass) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT fk_task_run_engineering_run_scope
            FOREIGN KEY (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
            REFERENCES multica.engineering_run
                (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
            ON DELETE RESTRICT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_task_run_engineering_run_worktree'
                   AND conrelid = 'multica.task_execution_run'::regclass) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT fk_task_run_engineering_run_worktree
            FOREIGN KEY (tenant_id, project_id, repository_id, branch_id, engineering_run_id,
                         worktree_id, engineering_run_worktree_binding_id, worktree_project_binding_id)
            REFERENCES multica.engineering_run_worktree_binding
                (tenant_id, project_id, repository_id, branch_id, engineering_run_id,
                 worktree_id, binding_id, project_binding_id)
            ON DELETE RESTRICT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_task_run_worktree_project_binding'
                   AND conrelid = 'multica.task_execution_run'::regclass) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT fk_task_run_worktree_project_binding
            FOREIGN KEY (tenant_id, project_id, worktree_id, worktree_project_binding_id)
            REFERENCES multica.worktree_project_binding
                (tenant_id, project_id, worktree_id, binding_id)
            ON DELETE RESTRICT;
    END IF;
END $$;

-- This INSERT-only gate preserves historical NULL rows while preventing new unbound CLI writes,
-- including direct database writers. Existing append-only guards continue to protect history.
CREATE OR REPLACE FUNCTION multica.guard_cli_task_run_engineering_run_binding()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    uuid_fields TEXT[] := ARRAY[
        'tenant_id','project_id','repository_id','branch_id','engineering_run_id','worktree_id',
        'work_item_id','worktree_project_binding_id','project_role_binding_id','branch_revision_id',
        'branch_role_binding_id','engineering_run_revision_id','engineering_run_role_binding_id',
        'engineering_run_worktree_binding_id'];
    version_fields TEXT[] := ARRAY[
        'worktree_project_binding_version','project_role_binding_version','branch_revision_version',
        'branch_role_binding_version','engineering_run_revision_version','engineering_run_role_binding_version',
        'engineering_run_worktree_binding_version'];
    role_fields TEXT[] := ARRAY['project_role','branch_role','engineering_run_role'];
    required_fields TEXT[] := uuid_fields || version_fields || role_fields || ARRAY['branch_full_ref'];
    snapshot JSONB := NEW.engineering_run_snapshot;
BEGIN
    IF NEW.execution_channel = 'cli' AND NEW.engineering_run_id IS NULL THEN
        RAISE EXCEPTION 'new CLI TaskExecutionRun requires canonical Engineering Run identity'
            USING ERRCODE = '23514', CONSTRAINT = 'cli_task_run_engineering_run_binding_guard';
    END IF;
    IF snapshot IS NOT NULL THEN
        IF jsonb_typeof(snapshot) IS DISTINCT FROM 'object'
           OR octet_length(snapshot::TEXT) > 8192 THEN
            RAISE EXCEPTION 'Engineering Run snapshot must be a bounded object'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_engineering_run_snapshot_shape';
        END IF;
        IF NOT (snapshot ?& required_fields) OR snapshot - required_fields <> '{}'::JSONB THEN
            RAISE EXCEPTION 'Engineering Run snapshot requires the complete typed identity without unknown fields'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_engineering_run_snapshot_shape';
        END IF;
        IF EXISTS (
            SELECT 1 FROM unnest(uuid_fields) AS field(name)
            WHERE jsonb_typeof(snapshot -> field.name) IS DISTINCT FROM 'string'
               OR snapshot ->> field.name !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
               OR snapshot ->> field.name = '00000000-0000-0000-0000-000000000000'
        ) OR EXISTS (
            SELECT 1 FROM unnest(version_fields) AS field(name)
            WHERE CASE
                WHEN jsonb_typeof(snapshot -> field.name) = 'number'
                 AND snapshot ->> field.name ~ '^[0-9]+$'
                THEN NOT ((snapshot ->> field.name)::NUMERIC BETWEEN 1 AND 2147483647)
                ELSE TRUE END
        ) OR EXISTS (
            SELECT 1 FROM unnest(role_fields) AS field(name)
            WHERE jsonb_typeof(snapshot -> field.name) IS DISTINCT FROM 'string'
               OR snapshot ->> field.name NOT IN ('tenant_admin','project_admin','developer','agent')
        ) OR jsonb_typeof(snapshot -> 'branch_full_ref') IS DISTINCT FROM 'string'
          OR octet_length(snapshot ->> 'branch_full_ref') NOT BETWEEN 12 AND 512
          OR snapshot ->> 'branch_full_ref' NOT LIKE 'refs/heads/%' THEN
            RAISE EXCEPTION 'Engineering Run snapshot identities, versions, grants and branch ref must be valid'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_engineering_run_snapshot_shape';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS cli_task_run_engineering_run_binding_guard ON multica.task_execution_run;
CREATE TRIGGER cli_task_run_engineering_run_binding_guard
    BEFORE INSERT ON multica.task_execution_run
    FOR EACH ROW EXECUTE FUNCTION multica.guard_cli_task_run_engineering_run_binding();

COMMENT ON COLUMN multica.task_execution_run.branch_id IS
    'Server-derived canonical Branch identity; NULL retained for pre-ERUN history and unbound non-CLI channels. New CLI rows require the complete identity.';
COMMENT ON COLUMN multica.task_execution_run.engineering_run_id IS
    'Canonical collaboration Engineering Run identity, distinct from task_execution_run.run_id.';
COMMENT ON COLUMN multica.task_execution_run.engineering_run_snapshot IS
    'Immutable exact Project/Branch/EngineeringRun/Worktree/Task tuple and grant/revision evidence used at admission.';

COMMIT;
