-- CYPHER STRUCTURAL MANIFEST
-- CREATE (f:File {name:"2026-10-02-task-run-owner.sql",type:"file",language:"sql"}),(m:Module {name:"task_run_owner_migration",type:"module",language:"sql"}),(metadata:Class {name:"multica.task_metadata",type:"class",language:"sql"}),(worktreeRelation:Class {name:"multica.work_item_worktree",type:"class",language:"sql"}),(runBinding:Class {name:"multica.engineering_run_worktree_binding",type:"class",language:"sql"}),(outbox:Class {name:"multica.task_run_outbox",type:"class",language:"sql"}),(metadataGuard:Function {name:"multica.guard_task_metadata_run_owner",type:"function",language:"sql"}),(worktreeGuard:Function {name:"multica.guard_task_worktree_run_owner",type:"function",language:"sql"}),(outboxWriter:Function {name:"multica.emit_task_run_outbox",type:"function",language:"sql"});
-- CREATE (f)-[:CONTAINS]->(m),(m)-[:USES]->(metadata),(m)-[:USES]->(worktreeRelation),(m)-[:USES]->(runBinding),(m)-[:CONTAINS]->(outbox),(m)-[:CONTAINS]->(metadataGuard),(m)-[:CONTAINS]->(worktreeGuard),(m)-[:CONTAINS]->(outboxWriter),(metadata)-[:USES]->(runBinding),(worktreeRelation)-[:USES]->(runBinding),(outbox)-[:USES]->(metadata);

-- Additive and non-destructive. Existing task rows remain unassigned until an explicit,
-- authorized Run ownership operation versions their metadata; no Worktree heuristic is used.
-- W/T/M: task_metadata remains Master/SCD2; task_run_outbox is Transaction/append-only.
BEGIN;

ALTER TABLE multica.task_metadata
    ADD COLUMN IF NOT EXISTS repository_id UUID,
    ADD COLUMN IF NOT EXISTS branch_id UUID,
    ADD COLUMN IF NOT EXISTS engineering_run_id UUID;

DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ck_task_metadata_run_owner_complete'
          AND conrelid = 'multica.task_metadata'::regclass
    ) THEN
        ALTER TABLE multica.task_metadata
            ADD CONSTRAINT ck_task_metadata_run_owner_complete CHECK (
                (repository_id IS NULL AND branch_id IS NULL AND engineering_run_id IS NULL)
                OR
                (repository_id IS NOT NULL AND branch_id IS NOT NULL AND engineering_run_id IS NOT NULL)
            );
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_task_metadata_engineering_run_scope'
          AND conrelid = 'multica.task_metadata'::regclass
    ) THEN
        ALTER TABLE multica.task_metadata
            ADD CONSTRAINT fk_task_metadata_engineering_run_scope
            FOREIGN KEY (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
            REFERENCES multica.engineering_run
                (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
            ON DELETE RESTRICT;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_task_metadata_run_current
    ON multica.task_metadata (tenant_id, project_id, engineering_run_id, updated_at DESC, work_item_id)
    WHERE valid_to IS NULL AND engineering_run_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS multica.task_run_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    task_metadata_id UUID NOT NULL,
    repository_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    aggregate_version INTEGER NOT NULL CHECK (aggregate_version > 0),
    event_type VARCHAR(32) NOT NULL CHECK (event_type IN ('task.created','task.updated','task.owner_changed')),
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object' AND octet_length(payload::TEXT) <= 8192),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (task_metadata_id)
        REFERENCES multica.task_metadata (metadata_id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
        REFERENCES multica.engineering_run
            (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
        ON DELETE RESTRICT,
    UNIQUE (tenant_id, work_item_id, aggregate_version)
);
CREATE INDEX IF NOT EXISTS idx_task_run_outbox_consumer_cursor
    ON multica.task_run_outbox (tenant_id, engineering_run_id, occurred_at, event_id);
CREATE INDEX IF NOT EXISTS idx_task_run_outbox_task_timeline
    ON multica.task_run_outbox (tenant_id, work_item_id, aggregate_version);

CREATE OR REPLACE FUNCTION multica.reject_task_run_outbox_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Task Run outbox Transaction rows are append-only' USING ERRCODE = '55000';
END;
$$;
DROP TRIGGER IF EXISTS task_run_outbox_immutable ON multica.task_run_outbox;
CREATE TRIGGER task_run_outbox_immutable
    BEFORE UPDATE OR DELETE ON multica.task_run_outbox
    FOR EACH ROW EXECUTE FUNCTION multica.reject_task_run_outbox_mutation();

CREATE OR REPLACE FUNCTION multica.guard_task_metadata_run_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    has_current_worktree_outside_run BOOLEAN;
BEGIN
    IF TG_OP = 'UPDATE' AND OLD.valid_to IS NULL AND NEW.valid_to IS NULL AND
       (OLD.repository_id, OLD.branch_id, OLD.engineering_run_id)
           IS DISTINCT FROM (NEW.repository_id, NEW.branch_id, NEW.engineering_run_id) THEN
        RAISE EXCEPTION 'Task Run ownership changes require a new SCD2 metadata version'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_run_owner_scd2';
    END IF;

    IF NEW.valid_to IS NULL THEN
        IF NEW.repository_id IS NULL OR NEW.branch_id IS NULL OR NEW.engineering_run_id IS NULL THEN
            RAISE EXCEPTION 'new current Task Cards require explicit Engineering Run ownership'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_run_owner_required';
        END IF;

        -- Lock active relations and their current Run bindings before checking the tuple.
        -- Association and binding writers acquire the same facts, so concurrent rebinds fail closed.
        PERFORM 1
          FROM multica.work_item_worktree relation
          JOIN multica.engineering_run_worktree_binding worktree_binding
            ON worktree_binding.tenant_id = relation.tenant_id
           AND worktree_binding.project_id = relation.project_id
           AND worktree_binding.worktree_id = relation.worktree_id
           AND worktree_binding.valid_to IS NULL
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.project_id = NEW.project_id
           AND relation.work_item_id = NEW.work_item_id
           AND relation.valid_to IS NULL
         ORDER BY relation.relation_id
         FOR SHARE OF relation, worktree_binding;

        SELECT EXISTS (
            SELECT 1
            FROM multica.work_item_worktree relation
            JOIN multica.engineering_run_worktree_binding worktree_binding
              ON worktree_binding.tenant_id = relation.tenant_id
             AND worktree_binding.project_id = relation.project_id
             AND worktree_binding.worktree_id = relation.worktree_id
             AND worktree_binding.valid_to IS NULL
            WHERE relation.tenant_id = NEW.tenant_id
              AND relation.project_id = NEW.project_id
              AND relation.work_item_id = NEW.work_item_id
              AND relation.valid_to IS NULL
              AND (worktree_binding.repository_id, worktree_binding.branch_id,
                   worktree_binding.engineering_run_id)
                  IS DISTINCT FROM (NEW.repository_id, NEW.branch_id, NEW.engineering_run_id)
        ) INTO has_current_worktree_outside_run;

        IF has_current_worktree_outside_run THEN
            RAISE EXCEPTION 'Task Card cannot change Run while it has Worktrees owned by another Run'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_worktree_run_match';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS task_metadata_run_owner_guard ON multica.task_metadata;
CREATE TRIGGER task_metadata_run_owner_guard
    BEFORE INSERT OR UPDATE OF repository_id, branch_id, engineering_run_id, valid_to
    ON multica.task_metadata
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_metadata_run_owner();

CREATE OR REPLACE FUNCTION multica.guard_task_worktree_run_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    task_run_id UUID;
    task_repository_id UUID;
    task_branch_id UUID;
BEGIN
    IF NEW.valid_to IS NULL THEN
        SELECT metadata.repository_id, metadata.branch_id, metadata.engineering_run_id
          INTO task_repository_id, task_branch_id, task_run_id
          FROM multica.task_metadata metadata
         WHERE metadata.tenant_id = NEW.tenant_id
           AND metadata.project_id = NEW.project_id
           AND metadata.work_item_id = NEW.work_item_id
           AND metadata.valid_to IS NULL;

        IF task_repository_id IS NULL OR task_branch_id IS NULL OR task_run_id IS NULL THEN
            RAISE EXCEPTION 'Task Card must have a current Engineering Run owner before Worktree association'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_work_item_worktree_task_run_owner';
        END IF;

        PERFORM 1
          FROM multica.engineering_run_worktree_binding binding
         WHERE binding.tenant_id = NEW.tenant_id
           AND binding.project_id = NEW.project_id
           AND binding.worktree_id = NEW.worktree_id
           AND binding.repository_id = task_repository_id
           AND binding.branch_id = task_branch_id
           AND binding.engineering_run_id = task_run_id
           AND binding.valid_to IS NULL
         FOR SHARE OF binding;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'Task Card and Worktree must share the same current Engineering Run'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_work_item_worktree_run_match';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS work_item_worktree_run_owner_guard ON multica.work_item_worktree;
CREATE TRIGGER work_item_worktree_run_owner_guard
    BEFORE INSERT OR UPDATE OF tenant_id, project_id, work_item_id, worktree_id, valid_to
    ON multica.work_item_worktree
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_worktree_run_owner();

CREATE OR REPLACE FUNCTION multica.guard_engineering_run_worktree_task_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.valid_to IS NULL THEN
        PERFORM 1
          FROM multica.work_item_worktree relation
          JOIN multica.task_metadata metadata
            ON metadata.tenant_id = relation.tenant_id
           AND metadata.project_id = relation.project_id
           AND metadata.work_item_id = relation.work_item_id
           AND metadata.valid_to IS NULL
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.project_id = NEW.project_id
           AND relation.worktree_id = NEW.worktree_id
           AND relation.valid_to IS NULL
         ORDER BY metadata.metadata_id, relation.relation_id
         FOR SHARE OF relation, metadata;

        IF EXISTS (
        SELECT 1
          FROM multica.work_item_worktree relation
          JOIN multica.task_metadata metadata
            ON metadata.tenant_id = relation.tenant_id
           AND metadata.project_id = relation.project_id
           AND metadata.work_item_id = relation.work_item_id
           AND metadata.valid_to IS NULL
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.project_id = NEW.project_id
           AND relation.worktree_id = NEW.worktree_id
           AND relation.valid_to IS NULL
           AND (metadata.repository_id, metadata.branch_id, metadata.engineering_run_id)
               IS DISTINCT FROM (NEW.repository_id, NEW.branch_id, NEW.engineering_run_id)
        ) THEN
            RAISE EXCEPTION 'Worktree cannot be reassigned while it has Task Cards owned by another Run'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_engineering_run_worktree_task_owner_match';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS engineering_run_worktree_task_owner_guard ON multica.engineering_run_worktree_binding;
CREATE TRIGGER engineering_run_worktree_task_owner_guard
    BEFORE INSERT OR UPDATE OF tenant_id, project_id, engineering_run_id, worktree_id, valid_to
    ON multica.engineering_run_worktree_binding
    FOR EACH ROW EXECUTE FUNCTION multica.guard_engineering_run_worktree_task_owner();

CREATE OR REPLACE FUNCTION multica.emit_task_run_outbox()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    previous_owner JSONB;
    actor UUID := NULLIF(current_setting('app.actor_id', true), '')::UUID;
    correlation UUID := NULLIF(current_setting('app.correlation_id', true), '')::UUID;
    task_event_type VARCHAR(32);
BEGIN
    IF NEW.valid_to IS NOT NULL THEN
        RETURN NEW;
    END IF;
    IF actor IS NULL OR correlation IS NULL THEN
        RAISE EXCEPTION 'Task Run outbox requires app.actor_id and app.correlation_id in the transaction context'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_outbox_context_required';
    END IF;

    SELECT jsonb_build_object(
               'repository_id', prior.repository_id,
               'branch_id', prior.branch_id,
               'engineering_run_id', prior.engineering_run_id
           )
      INTO previous_owner
      FROM multica.task_metadata prior
     WHERE prior.tenant_id = NEW.tenant_id
       AND prior.project_id = NEW.project_id
       AND prior.work_item_id = NEW.work_item_id
       AND prior.metadata_id <> NEW.metadata_id
       AND prior.version < NEW.version
     ORDER BY prior.version DESC
     LIMIT 1;

    task_event_type := CASE
        WHEN previous_owner IS NULL THEN 'task.created'
        WHEN previous_owner IS DISTINCT FROM jsonb_build_object(
            'repository_id', NEW.repository_id,
            'branch_id', NEW.branch_id,
            'engineering_run_id', NEW.engineering_run_id
        ) THEN 'task.owner_changed'
        ELSE 'task.updated'
    END;

    INSERT INTO multica.task_run_outbox (
        tenant_id, project_id, work_item_id, task_metadata_id, repository_id, branch_id,
        engineering_run_id, aggregate_version,
        event_type, actor_id, correlation_id, payload
    ) VALUES (
        NEW.tenant_id, NEW.project_id, NEW.work_item_id, NEW.metadata_id,
        NEW.repository_id, NEW.branch_id, NEW.engineering_run_id, NEW.version,
        task_event_type, actor, correlation,
        jsonb_build_object(
            'work_item_id', NEW.work_item_id,
            'project_id', NEW.project_id,
            'repository_id', NEW.repository_id,
            'branch_id', NEW.branch_id,
            'engineering_run_id', NEW.engineering_run_id,
            'metadata_version', NEW.version,
            'previous_owner', previous_owner
        )
    );
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS task_metadata_run_outbox ON multica.task_metadata;
CREATE TRIGGER task_metadata_run_outbox
    AFTER INSERT ON multica.task_metadata
    FOR EACH ROW EXECUTE FUNCTION multica.emit_task_run_outbox();

ALTER TABLE multica.task_run_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_run_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_run_outbox_tenant_select ON multica.task_run_outbox;
CREATE POLICY task_run_outbox_tenant_select ON multica.task_run_outbox
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_run_outbox_tenant_insert ON multica.task_run_outbox;
CREATE POLICY task_run_outbox_tenant_insert ON multica.task_run_outbox
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );
DROP POLICY IF EXISTS task_run_outbox_no_update ON multica.task_run_outbox;
CREATE POLICY task_run_outbox_no_update ON multica.task_run_outbox
    FOR UPDATE USING (false) WITH CHECK (false);
DROP POLICY IF EXISTS task_run_outbox_no_delete ON multica.task_run_outbox;
CREATE POLICY task_run_outbox_no_delete ON multica.task_run_outbox
    FOR DELETE USING (false);

COMMENT ON COLUMN multica.task_metadata.engineering_run_id IS
    'Canonical owner of this SCD2 Task Card version. Null rows are legacy and are never implicitly assigned.';
COMMENT ON TABLE multica.task_run_outbox IS
    'T: append-only transactional events for Run-scoped Task Cards, BI projections and cross-App consumers.';

COMMIT;
