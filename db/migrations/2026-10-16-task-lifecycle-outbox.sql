-- CYPHER STRUCTURAL MANIFEST
-- CREATE (f:File {name:"2026-10-16-task-lifecycle-outbox.sql",type:"file",language:"sql"}),(m:Module {name:"task_lifecycle_outbox_migration",type:"module",language:"sql"}),(outbox:Table {name:"multica.task_run_outbox",type:"table",language:"sql"}),(audit:Table {name:"multica.task_lifecycle_audit",type:"table",language:"sql"}),(lifecycle:Table {name:"multica.task_lifecycle_current",type:"table",language:"sql"}),(metadata:Table {name:"multica.task_metadata",type:"table",language:"sql"}),(writer:Function {name:"multica.emit_task_lifecycle_run_outbox",type:"function",language:"sql"}),(trigger:Trigger {name:"task_lifecycle_run_outbox",type:"trigger",language:"sql"});
-- CREATE (f)-[:CONTAINS]->(m),(m)-[:ALTERS]->(outbox),(m)-[:USES]->(audit),(m)-[:USES]->(lifecycle),(m)-[:USES]->(metadata),(m)-[:DEFINES]->(writer),(m)-[:DEFINES]->(trigger),(trigger)-[:FIRES_AFTER]->(audit),(trigger)-[:CALLS]->(writer),(writer)-[:WRITES]->(outbox),(writer)-[:READS]->(metadata),(writer)-[:READS]->(lifecycle);

-- Add lifecycle events to the existing owner-local append-only Task Outbox.
-- Metadata and lifecycle versions are independent streams and must not collide.
BEGIN;

ALTER TABLE multica.task_run_outbox
    ADD COLUMN IF NOT EXISTS aggregate_type VARCHAR(16) NOT NULL DEFAULT 'metadata';

ALTER TABLE multica.task_run_outbox
    DROP CONSTRAINT IF EXISTS task_run_outbox_tenant_id_work_item_id_aggregate_version_key,
    DROP CONSTRAINT IF EXISTS task_run_outbox_event_type_check,
    DROP CONSTRAINT IF EXISTS task_run_outbox_schema_version_check,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_schema_version,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_aggregate_type,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_v2_aggregate_type,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_v3_aggregate_type,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_v1_event_type,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_v2_event_type,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_v3_event_type,
    DROP CONSTRAINT IF EXISTS ck_task_run_outbox_event_type;

ALTER TABLE multica.task_run_outbox
    ADD CONSTRAINT ck_task_run_outbox_schema_version
        CHECK (schema_version IN (1, 2, 3)),
    ADD CONSTRAINT ck_task_run_outbox_aggregate_type
        CHECK (aggregate_type IN ('metadata', 'lifecycle')),
    ADD CONSTRAINT ck_task_run_outbox_v2_aggregate_type
        CHECK (schema_version <> 2 OR aggregate_type = 'metadata'),
    ADD CONSTRAINT ck_task_run_outbox_v3_aggregate_type
        CHECK (schema_version <> 3 OR aggregate_type = 'lifecycle'),
    ADD CONSTRAINT ck_task_run_outbox_v1_event_type
        CHECK (schema_version <> 1 OR event_type IN ('task.created', 'task.updated', 'task.owner_changed')),
    ADD CONSTRAINT ck_task_run_outbox_v2_event_type
        CHECK (schema_version <> 2 OR event_type IN ('task.created', 'task.updated', 'task.owner_changed')),
    ADD CONSTRAINT ck_task_run_outbox_v3_event_type
        CHECK (schema_version <> 3 OR event_type = 'task.lifecycle.changed'),
    ADD CONSTRAINT ck_task_run_outbox_event_type
        CHECK (event_type IN ('task.created', 'task.updated', 'task.owner_changed', 'task.lifecycle.changed'));

CREATE UNIQUE INDEX IF NOT EXISTS uq_task_run_outbox_aggregate_version
    ON multica.task_run_outbox (tenant_id, work_item_id, aggregate_type, aggregate_version);

CREATE INDEX IF NOT EXISTS idx_task_run_outbox_run_stream_cursor
    ON multica.task_run_outbox
        (tenant_id, project_id, engineering_run_id, aggregate_type, occurred_at, event_id);

CREATE OR REPLACE FUNCTION multica.emit_task_lifecycle_run_outbox()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    current_task RECORD;
BEGIN
    SELECT metadata.metadata_id,
           metadata.workspace_id,
           metadata.project_id,
           metadata.repository_id,
           metadata.branch_id,
           metadata.engineering_run_id,
           lifecycle.version,
           lifecycle.status,
           lifecycle.review_state,
           lifecycle.active_worktree_id,
           lifecycle.claimed_by,
           lifecycle.claim_expires_at
      INTO current_task
      FROM multica.task_metadata metadata
      JOIN multica.task_lifecycle_current lifecycle
        ON lifecycle.tenant_id = metadata.tenant_id
       AND lifecycle.work_item_id = metadata.work_item_id
     WHERE metadata.tenant_id = NEW.tenant_id
       AND metadata.project_id = NEW.project_id
       AND metadata.work_item_id = NEW.work_item_id
       AND metadata.valid_to IS NULL
     FOR SHARE OF metadata, lifecycle;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Task lifecycle Outbox requires a current Project + Run-owned Task Card'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_outbox_project_run_required';
    END IF;
    IF current_task.engineering_run_id IS NULL THEN
        RAISE EXCEPTION 'Task lifecycle Outbox requires a current Project + Run-owned Task Card'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_outbox_project_run_required';
    END IF;

    INSERT INTO multica.task_run_outbox (
        tenant_id, project_id, work_item_id, task_metadata_id, repository_id, branch_id,
        engineering_run_id, aggregate_type, aggregate_version, event_type, schema_version,
        actor_id, correlation_id, payload, occurred_at
    ) VALUES (
        NEW.tenant_id, current_task.project_id, NEW.work_item_id, current_task.metadata_id,
        current_task.repository_id, current_task.branch_id, current_task.engineering_run_id,
        'lifecycle', current_task.version, 'task.lifecycle.changed', 3,
        NEW.actor_id, NEW.correlation_id,
        jsonb_build_object(
            'schema_version', 3,
            'aggregate', jsonb_build_object(
                'type', 'lifecycle',
                'version', current_task.version
            ),
            'work_item_id', NEW.work_item_id,
            'workspace_id', current_task.workspace_id,
            'owner', jsonb_build_object(
                'tenant_id', NEW.tenant_id,
                'project_id', current_task.project_id,
                'engineering_run_id', current_task.engineering_run_id
            ),
            'host_context', jsonb_build_object(
                'repository_id', current_task.repository_id,
                'branch_id', current_task.branch_id
            ),
            'lifecycle', jsonb_build_object(
                'status', current_task.status,
                'review_state', current_task.review_state,
                'active_worktree_id', current_task.active_worktree_id,
                'claimed_by', current_task.claimed_by,
                'claim_expires_at', current_task.claim_expires_at
            ),
            'change', jsonb_build_object(
                'audit_event_id', NEW.event_id,
                'event_type', NEW.event_type,
                'from_status', NEW.from_status,
                'to_status', NEW.to_status,
                'worktree_id', NEW.worktree_id
            )
        ),
        NEW.occurred_at
    );

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS task_lifecycle_run_outbox ON multica.task_lifecycle_audit;
CREATE TRIGGER task_lifecycle_run_outbox
    AFTER INSERT ON multica.task_lifecycle_audit
    FOR EACH ROW EXECUTE FUNCTION multica.emit_task_lifecycle_run_outbox();

COMMENT ON COLUMN multica.task_run_outbox.aggregate_type IS
    'Independent Task metadata and lifecycle version streams; lifecycle uses schema v3.';
COMMENT ON TABLE multica.task_run_outbox IS
    'T: append-only transactional Task metadata and lifecycle events. Lifecycle v3 rows are emitted from lifecycle audit inserts in the same owner transaction; consumers and BI projections remain separate owners.';

COMMIT;
