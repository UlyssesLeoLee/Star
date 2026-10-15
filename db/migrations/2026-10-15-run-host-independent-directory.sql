-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-15-run-host-independent-directory.sql"})
-- MERGE (run:Table {id:"multica.engineering_run"})
-- MERGE (run_worktree:Table {id:"multica.engineering_run_worktree_binding"})
-- MERGE (directory_event:Table {id:"multica.engineering_directory_event"})
-- MERGE (idempotency:Table {id:"multica.engineering_run_command_idempotency"})
-- MERGE (outbox:Table {id:"multica.engineering_run_outbox"})
-- MERGE (guard:Function {id:"multica.guard_engineering_directory_write"})
-- MERGE (scope:Function {id:"multica.engineering_directory_scope"})
-- MERGE (project_grant:Table {id:"permission.project_role_binding"})
-- MERGE (run_repository:Table {id:"multica.engineering_run"})
-- MERGE (self)-[:CONFIGURES]->(run)
-- MERGE (self)-[:CONFIGURES]->(run_worktree)
-- MERGE (self)-[:CONFIGURES]->(directory_event)
-- MERGE (self)-[:CONFIGURES]->(idempotency)
-- MERGE (self)-[:CONFIGURES]->(outbox)
-- MERGE (self)-[:CONFIGURES]->(guard)
-- MERGE (guard)-[:CALLS]->(scope)
-- MERGE (guard)-[:READS]->(project_grant)
-- MERGE (guard)-[:READS]->(run_worktree)
-- MERGE (guard)-[:READS]->(run_repository)
-- @endcypher
-- Make Engineering Run a Project-owned workspace. A remote host Branch is optional metadata.
BEGIN;

ALTER TABLE multica.engineering_run
    ALTER COLUMN repository_id DROP NOT NULL,
    ALTER COLUMN branch_id DROP NOT NULL;
ALTER TABLE multica.engineering_run_worktree_binding
    ALTER COLUMN branch_id DROP NOT NULL;
ALTER TABLE multica.engineering_directory_event
    ALTER COLUMN repository_id DROP NOT NULL,
    ALTER COLUMN branch_id DROP NOT NULL;

ALTER TABLE multica.engineering_run
    ADD CONSTRAINT ck_engineering_run_remote_branch_requires_repository
    CHECK (branch_id IS NULL OR repository_id IS NOT NULL);
CREATE UNIQUE INDEX IF NOT EXISTS uq_engineering_run_project_identity
    ON multica.engineering_run (tenant_id, project_id, engineering_run_id);

-- The old branch-bearing FKs remain useful for hosted records. These additional
-- FKs prove Project/Run ownership when optional host fields are NULL.
DO $$
DECLARE
    constraint_name TEXT;
BEGIN
    FOR constraint_name IN
        SELECT conname FROM pg_constraint
        WHERE conrelid = 'multica.engineering_run_worktree_binding'::regclass
          AND confrelid = 'multica.engineering_run'::regclass AND contype = 'f'
    LOOP
        EXECUTE format('ALTER TABLE multica.engineering_run_worktree_binding DROP CONSTRAINT %I', constraint_name);
    END LOOP;
END;
$$;
ALTER TABLE multica.engineering_run_worktree_binding
    ADD CONSTRAINT fk_engineering_run_worktree_project_run
    FOREIGN KEY (tenant_id, project_id, engineering_run_id)
    REFERENCES multica.engineering_run (tenant_id, project_id, engineering_run_id)
    ON DELETE RESTRICT;

ALTER TABLE multica.engineering_directory_event
    ADD CONSTRAINT fk_engineering_directory_event_run_identity
    FOREIGN KEY (tenant_id, engineering_run_id)
    REFERENCES multica.engineering_run (tenant_id, engineering_run_id)
    ON DELETE RESTRICT;
ALTER TABLE multica.engineering_directory_event
    ADD CONSTRAINT ck_engineering_directory_event_scope
    CHECK (
      (source_table IN ('scm.cloud_branch','scm.cloud_branch_revision','permission.cloud_branch_role_binding')
       AND engineering_run_id IS NULL AND repository_id IS NOT NULL AND branch_id IS NOT NULL)
      OR
      (source_table IN ('multica.engineering_run','multica.engineering_run_revision',
                        'permission.engineering_run_role_binding','multica.engineering_run_worktree_binding')
       AND engineering_run_id IS NOT NULL)
    );

CREATE OR REPLACE FUNCTION multica.guard_engineering_directory_write()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    row_data JSONB;
    writer_tenant_id UUID;
    writer_actor_id UUID;
    writer_correlation_id UUID;
    scope_project_id UUID;
    scope_repository_id UUID;
    scope_branch_id UUID;
    scope_run_id UUID;
    subject_column TEXT;
    subject_id UUID;
    previous_version INTEGER;
    previous_valid_to TIMESTAMPTZ;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'engineering directory history cannot be physically deleted';
    END IF;
    row_data := to_jsonb(NEW);
    IF TG_TABLE_NAME IN ('cloud_branch','engineering_run','engineering_directory_event') THEN
        IF TG_OP <> 'INSERT' THEN
            RAISE EXCEPTION 'engineering directory registration and audit facts are immutable';
        END IF;
    ELSIF TG_OP = 'INSERT' THEN
        IF NEW.valid_to IS NOT NULL OR NEW.valid_from > statement_timestamp() THEN
            RAISE EXCEPTION 'engineering directory SCD2 rows must start current and cannot start in the future';
        END IF;
    ELSE
        IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL
           OR NEW.valid_to < OLD.valid_from OR NEW.valid_to > statement_timestamp()
           OR (to_jsonb(OLD) - 'valid_to') <> (row_data - 'valid_to') THEN
            RAISE EXCEPTION 'engineering directory SCD2 rows may only be closed once without rewriting history';
        END IF;
    END IF;
    IF NEW.created_at > statement_timestamp() THEN
        RAISE EXCEPTION 'engineering directory creation time cannot be in the future';
    END IF;

    writer_tenant_id := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    writer_actor_id := NULLIF(current_setting('app.actor_id', true), '')::UUID;
    writer_correlation_id := NULLIF(current_setting('app.correlation_id', true), '')::UUID;
    IF writer_tenant_id IS NULL OR writer_actor_id IS NULL OR writer_correlation_id IS NULL
       OR writer_tenant_id = '00000000-0000-0000-0000-000000000000'
       OR writer_actor_id = '00000000-0000-0000-0000-000000000000'
       OR writer_correlation_id = '00000000-0000-0000-0000-000000000000'
       OR NEW.tenant_id <> writer_tenant_id THEN
        RAISE EXCEPTION 'engineering directory write requires matching tenant, actor and correlation context';
    END IF;
    SELECT s.scope_project_id, s.scope_repository_id, s.scope_branch_id, s.scope_run_id
        INTO scope_project_id, scope_repository_id, scope_branch_id, scope_run_id
        FROM multica.engineering_directory_scope(TG_TABLE_SCHEMA, TG_TABLE_NAME, row_data) s;
    IF scope_project_id IS NULL
       OR (scope_run_id IS NULL AND (scope_repository_id IS NULL OR scope_branch_id IS NULL))
       OR scope_project_id = '00000000-0000-0000-0000-000000000000'
       OR (scope_repository_id IS NOT NULL AND scope_repository_id = '00000000-0000-0000-0000-000000000000')
       OR (scope_branch_id IS NOT NULL AND scope_branch_id = '00000000-0000-0000-0000-000000000000') THEN
        RAISE EXCEPTION 'engineering directory canonical scope is unavailable';
    END IF;
    PERFORM 1 FROM permission.project_role_binding p
        WHERE p.tenant_id = writer_tenant_id AND p.project_id = scope_project_id
          AND p.user_id = writer_actor_id AND p.valid_from <= now() AND p.valid_to IS NULL
          AND p.role IN ('tenant_admin','project_admin') FOR SHARE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'engineering directory write requires current Project management authority';
    END IF;

    IF TG_OP = 'INSERT' AND TG_TABLE_NAME NOT IN ('cloud_branch','engineering_run','engineering_directory_event') THEN
        subject_column := CASE TG_TABLE_NAME
            WHEN 'cloud_branch_revision' THEN 'branch_id'
            WHEN 'cloud_branch_role_binding' THEN 'branch_id'
            WHEN 'engineering_run_revision' THEN 'engineering_run_id'
            WHEN 'engineering_run_role_binding' THEN 'engineering_run_id'
            WHEN 'engineering_run_worktree_binding' THEN 'worktree_id'
        END;
        subject_id := (row_data ->> subject_column)::UUID;
        IF TG_TABLE_SCHEMA = 'permission' THEN
            EXECUTE format('SELECT version, valid_to FROM %I.%I
                WHERE tenant_id = $1 AND %I = $2 AND user_id = $3 ORDER BY version DESC LIMIT 1',
                TG_TABLE_SCHEMA, TG_TABLE_NAME, subject_column)
                INTO previous_version, previous_valid_to
                USING NEW.tenant_id, subject_id, (row_data ->> 'user_id')::UUID;
        ELSE
            EXECUTE format('SELECT version, valid_to FROM %I.%I
                WHERE tenant_id = $1 AND %I = $2 ORDER BY version DESC LIMIT 1',
                TG_TABLE_SCHEMA, TG_TABLE_NAME, subject_column)
                INTO previous_version, previous_valid_to USING NEW.tenant_id, subject_id;
        END IF;
        IF NEW.version <> COALESCE(previous_version, 0) + 1
           OR (previous_version IS NOT NULL AND
               (previous_valid_to IS NULL OR NEW.valid_from < previous_valid_to)) THEN
            RAISE EXCEPTION 'engineering directory SCD2 successor must follow the closed previous version without overlapping history';
        END IF;
    END IF;

    IF TG_OP = 'INSERT' AND (
        (row_data ? 'changed_by' AND (row_data ->> 'changed_by')::UUID <> writer_actor_id)
        OR (row_data ? 'granted_by' AND (row_data ->> 'granted_by')::UUID <> writer_actor_id)
        OR (row_data ? 'assigned_by' AND (row_data ->> 'assigned_by')::UUID <> writer_actor_id)
    ) THEN
        RAISE EXCEPTION 'engineering directory author must match the authenticated writer';
    END IF;
    IF TG_TABLE_NAME = 'engineering_directory_event' THEN
        IF pg_trigger_depth() < 2 THEN
            RAISE EXCEPTION 'engineering directory audit is produced only by source-table write triggers';
        END IF;
        IF (row_data ->> 'actor_id')::UUID <> writer_actor_id
           OR (row_data ->> 'correlation_id')::UUID <> writer_correlation_id THEN
            RAISE EXCEPTION 'engineering directory audit must match the authenticated writer context';
        END IF;
    END IF;
    IF TG_OP = 'INSERT' AND TG_TABLE_NAME = 'engineering_run_worktree_binding' THEN
        PERFORM 1 FROM multica.worktree_project_binding p
            JOIN worktree_canvas_worktree w ON w.tenant_id = p.tenant_id AND w.id = p.worktree_id
            JOIN multica.engineering_run r ON r.tenant_id = NEW.tenant_id
              AND r.project_id = NEW.project_id AND r.engineering_run_id = NEW.engineering_run_id
            LEFT JOIN scm.cloud_branch b ON b.tenant_id = NEW.tenant_id AND b.project_id = NEW.project_id
              AND b.repository_id = NEW.repository_id AND b.branch_id = NEW.branch_id
            WHERE p.binding_id = NEW.project_binding_id AND p.tenant_id = NEW.tenant_id
              AND p.project_id = NEW.project_id AND p.worktree_id = NEW.worktree_id
              AND p.valid_from <= now() AND p.valid_to IS NULL
              AND w.project_id = p.project_id AND w.repo_id = NEW.repository_id
              AND (r.repository_id IS NULL OR r.repository_id = NEW.repository_id)
              AND (r.branch_id IS NULL OR NEW.branch_id IS NULL OR r.branch_id = NEW.branch_id)
              AND (NEW.branch_id IS NULL OR b.branch_id IS NOT NULL)
            FOR SHARE OF p, w;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'Engineering Run binding requires a current matching Project and repository Worktree relation';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TABLE IF NOT EXISTS multica.engineering_run_command_idempotency (
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    response_body JSONB NOT NULL CHECK (jsonb_typeof(response_body) = 'object' AND octet_length(response_body::TEXT) <= 16384),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, actor_id, idempotency_key),
    CHECK (expires_at > created_at AND expires_at <= created_at + INTERVAL '24 hours')
);
CREATE INDEX IF NOT EXISTS idx_engineering_run_command_idempotency_expiry
    ON multica.engineering_run_command_idempotency (tenant_id, actor_id, expires_at, idempotency_key);
ALTER TABLE multica.engineering_run_command_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.engineering_run_command_idempotency FORCE ROW LEVEL SECURITY;
CREATE POLICY engineering_run_command_idempotency_actor_scope
    ON multica.engineering_run_command_idempotency FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
       AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
       AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID);

CREATE TABLE IF NOT EXISTS multica.engineering_run_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    event_type VARCHAR(96) NOT NULL CHECK (event_type IN ('engineering_run.created','engineering_run.worktree_bound')),
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object' AND octet_length(payload::TEXT) <= 16384),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, project_id, engineering_run_id)
        REFERENCES multica.engineering_run (tenant_id, project_id, engineering_run_id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_engineering_run_outbox_delivery
    ON multica.engineering_run_outbox (occurred_at, event_id);
ALTER TABLE multica.engineering_run_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.engineering_run_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY engineering_run_outbox_tenant_scope ON multica.engineering_run_outbox FOR SELECT
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
CREATE POLICY engineering_run_outbox_insert_scope ON multica.engineering_run_outbox FOR INSERT
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON COLUMN multica.engineering_run.branch_id IS
    'Optional remote host Branch association. It is metadata only and never the Engineering Run parent or authorization prerequisite.';
COMMENT ON TABLE permission.engineering_run_role_binding IS
    'M/SCD2: explicit Engineering Run authority; every read revalidates current Project membership; a remote Branch grant is not required.';
COMMENT ON COLUMN multica.engineering_run.repository_id IS
    'Optional repository association; Run identity and Project/Run authorization remain valid without a remote host.';
COMMENT ON COLUMN multica.engineering_run_worktree_binding.branch_id IS
    'Optional legacy remote Branch snapshot. The local checkout ref is worktree_canvas_worktree.branch and is shown below Worktree.';
COMMENT ON TABLE multica.engineering_run_outbox IS
    'Append-only owner-local events committed with Engineering Run directory mutations. A claim, retry, or delivery worker is not implemented by this migration.';
COMMENT ON TABLE multica.engineering_run_command_idempotency IS
    'W: actor-scoped 24-hour idempotency receipts. Expired receipts are removed in bounded batches during later Run create commands; dormant actor rows require retention maintenance.';

COMMIT;
