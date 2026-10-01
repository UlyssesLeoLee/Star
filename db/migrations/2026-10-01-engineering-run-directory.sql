-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-10-01-engineering-run-directory.sql",type:"file",language:"sql"}),
--   (m:Module {name:"engineering_run_directory",type:"module",language:"sql"}),
--   (branch:Class {name:"scm.cloud_branch",type:"class",language:"sql"}),
--   (branchRevision:Class {name:"scm.cloud_branch_revision",type:"class",language:"sql"}),
--   (branchGrant:Class {name:"permission.cloud_branch_role_binding",type:"class",language:"sql"}),
--   (run:Class {name:"multica.engineering_run",type:"class",language:"sql"}),
--   (runRevision:Class {name:"multica.engineering_run_revision",type:"class",language:"sql"}),
--   (runGrant:Class {name:"permission.engineering_run_role_binding",type:"class",language:"sql"}),
--   (binding:Class {name:"multica.engineering_run_worktree_binding",type:"class",language:"sql"}),
--   (event:Class {name:"multica.engineering_directory_event",type:"class",language:"sql"}),
--   (scope:Function {name:"multica.engineering_directory_scope",type:"function",language:"sql"}),
--   (guard:Function {name:"multica.guard_engineering_directory_write",type:"function",language:"sql"}),
--   (audit:Function {name:"multica.audit_engineering_directory_write",type:"function",language:"sql"}),
--   (install:Logic {name:"install_directory_policies_and_triggers",type:"logic",language:"sql"}),
--   (tables:Variable {name:"directory_tables",type:"variable",language:"sql"}),
--   (currentSetting:Function {name:"current_setting",type:"function",language:"sql"}),
--   (toJson:Function {name:"to_jsonb",type:"function",language:"sql"}),
--   (clock:Function {name:"statement_timestamp",type:"function",language:"sql"}),
--   (now:Function {name:"now",type:"function",language:"sql"}),
--   (uuid:Function {name:"gen_random_uuid",type:"function",language:"sql"}),
--   (format:Function {name:"format",type:"function",language:"sql"}),
--   (octets:Function {name:"octet_length",type:"function",language:"sql"}),
--   (jsonType:Function {name:"jsonb_typeof",type:"function",language:"sql"}),
--   (triggerDepth:Function {name:"pg_trigger_depth",type:"function",language:"sql"}),
--   (length:Function {name:"length",type:"function",language:"sql"}),
--   (trim:Function {name:"trim",type:"function",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(branch),(m)-[:CONTAINS]->(branchRevision),
--   (m)-[:CONTAINS]->(branchGrant),(m)-[:CONTAINS]->(run),(m)-[:CONTAINS]->(runRevision),
--   (m)-[:CONTAINS]->(runGrant),(m)-[:CONTAINS]->(binding),(m)-[:CONTAINS]->(event),
--   (m)-[:CONTAINS]->(scope),(m)-[:CONTAINS]->(guard),(m)-[:CONTAINS]->(audit),
--   (m)-[:CONTAINS]->(install),(install)-[:USES]->(tables),(install)-[:CALLS]->(format),
--   (install)-[:USES]->(guard),(install)-[:USES]->(audit),(install)-[:CALLS]->(currentSetting),
--   (scope)-[:USES]->(branch),(scope)-[:USES]->(run),
--   (guard)-[:CALLS]->(scope),(guard)-[:CALLS]->(currentSetting),(guard)-[:CALLS]->(toJson),
--   (guard)-[:CALLS]->(clock),(guard)-[:CALLS]->(now),
--   (guard)-[:CALLS]->(triggerDepth),(guard)-[:CALLS]->(format),
--   (audit)-[:CALLS]->(scope),(audit)-[:CALLS]->(currentSetting),(audit)-[:CALLS]->(toJson),
--   (audit)-[:USES]->(event),
--   (branch)-[:CALLS]->(now),(run)-[:CALLS]->(now),
--   (branch)-[:CALLS]->(length),(branch)-[:CALLS]->(trim),
--   (branchRevision)-[:CALLS]->(length),(branchRevision)-[:CALLS]->(trim),
--   (runRevision)-[:CALLS]->(length),(runRevision)-[:CALLS]->(trim),
--   (branch)-[:CALLS]->(guard),(branchRevision)-[:CALLS]->(guard),(branchGrant)-[:CALLS]->(guard),
--   (run)-[:CALLS]->(guard),(runRevision)-[:CALLS]->(guard),(runGrant)-[:CALLS]->(guard),
--   (binding)-[:CALLS]->(guard),(event)-[:CALLS]->(guard),
--   (branch)-[:CALLS]->(audit),(branchRevision)-[:CALLS]->(audit),(branchGrant)-[:CALLS]->(audit),
--   (run)-[:CALLS]->(audit),(runRevision)-[:CALLS]->(audit),(runGrant)-[:CALLS]->(audit),(binding)-[:CALLS]->(audit),
--   (branchRevision)-[:CALLS]->(uuid),(branchRevision)-[:CALLS]->(now),(branchRevision)-[:CALLS]->(currentSetting),
--   (branchGrant)-[:CALLS]->(uuid),(branchGrant)-[:CALLS]->(now),(branchGrant)-[:CALLS]->(currentSetting),
--   (runRevision)-[:CALLS]->(uuid),(runRevision)-[:CALLS]->(now),(runRevision)-[:CALLS]->(currentSetting),
--   (runGrant)-[:CALLS]->(uuid),(runGrant)-[:CALLS]->(now),(runGrant)-[:CALLS]->(currentSetting),
--   (binding)-[:CALLS]->(uuid),(binding)-[:CALLS]->(now),
--   (event)-[:CALLS]->(uuid),(event)-[:CALLS]->(now),(event)-[:CALLS]->(octets),(event)-[:CALLS]->(jsonType);

-- ERUN-P1 persisted directory foundation; additive after Worktree Group Phase 2B.
-- W/T/M: zero W; five M revision/grant/binding tables; three T registration/audit tables.
-- No seed/backfill, remote SCM operation, execution admission, or outbox delivery.
-- Every DML write requires transaction-local app.tenant_id, app.actor_id and
-- app.correlation_id plus a current Project manager. Explicit Branch/Run grants
-- remain mandatory for reads; management authority does not create a read grant.

BEGIN;

CREATE SCHEMA IF NOT EXISTS scm;
CREATE SCHEMA IF NOT EXISTS permission;
CREATE SCHEMA IF NOT EXISTS multica;

-- Stable registration facts. The trusted SCM producer must supply the canonical
-- domain-scm BranchId and Repository/Project mapping; local branch names are not identity.
CREATE TABLE IF NOT EXISTS scm.cloud_branch (
    branch_id UUID PRIMARY KEY CHECK (branch_id <> '00000000-0000-0000-0000-000000000000'),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    repository_id UUID NOT NULL,
    remote_identity TEXT NOT NULL CHECK (length(trim(remote_identity)) BETWEEN 1 AND 512),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, branch_id),
    UNIQUE (tenant_id, project_id, repository_id, branch_id),
    UNIQUE (tenant_id, repository_id, remote_identity)
);

CREATE TABLE IF NOT EXISTS scm.cloud_branch_revision (
    revision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 255),
    full_ref TEXT NOT NULL CHECK (length(full_ref) BETWEEN 12 AND 512 AND full_ref LIKE 'refs/heads/%'),
    head_commit_id TEXT CHECK (head_commit_id IS NULL OR length(trim(head_commit_id)) BETWEEN 1 AND 128),
    state VARCHAR(16) NOT NULL CHECK (state IN ('active','archived')),
    changed_by UUID NOT NULL DEFAULT NULLIF(current_setting('app.actor_id', true), '')::UUID,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, branch_id) REFERENCES scm.cloud_branch (tenant_id, branch_id) ON DELETE RESTRICT,
    UNIQUE (tenant_id, branch_id, version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_cloud_branch_revision_current
    ON scm.cloud_branch_revision (tenant_id, branch_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_cloud_branch_project_directory
    ON scm.cloud_branch (tenant_id, project_id, branch_id);

CREATE TABLE IF NOT EXISTS permission.cloud_branch_role_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    user_id UUID NOT NULL CHECK (user_id <> '00000000-0000-0000-0000-000000000000'),
    role VARCHAR(32) NOT NULL CHECK (role IN ('tenant_admin','project_admin','developer','viewer','agent')),
    granted_by UUID NOT NULL DEFAULT NULLIF(current_setting('app.actor_id', true), '')::UUID,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, branch_id) REFERENCES scm.cloud_branch (tenant_id, branch_id) ON DELETE RESTRICT,
    UNIQUE (tenant_id, branch_id, user_id, version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_cloud_branch_role_binding_current
    ON permission.cloud_branch_role_binding (tenant_id, branch_id, user_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_cloud_branch_role_binding_subject
    ON permission.cloud_branch_role_binding (tenant_id, user_id, branch_id) WHERE valid_to IS NULL;

-- A collaboration workspace identity, distinct from task_execution_run.run_id.
-- This directory phase has no admission snapshots; execution remains unavailable.
CREATE TABLE IF NOT EXISTS multica.engineering_run (
    engineering_run_id UUID PRIMARY KEY CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    repository_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, engineering_run_id),
    UNIQUE (tenant_id, project_id, repository_id, branch_id, engineering_run_id),
    FOREIGN KEY (tenant_id, project_id, repository_id, branch_id)
        REFERENCES scm.cloud_branch (tenant_id, project_id, repository_id, branch_id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_engineering_run_branch_directory
    ON multica.engineering_run (tenant_id, branch_id, engineering_run_id);

CREATE TABLE IF NOT EXISTS multica.engineering_run_revision (
    revision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 500),
    state VARCHAR(16) NOT NULL CHECK (state IN ('draft','active','paused','completed','archived')),
    owner_user_id UUID CHECK (owner_user_id IS NULL OR owner_user_id <> '00000000-0000-0000-0000-000000000000'),
    changed_by UUID NOT NULL DEFAULT NULLIF(current_setting('app.actor_id', true), '')::UUID,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, engineering_run_id)
        REFERENCES multica.engineering_run (tenant_id, engineering_run_id) ON DELETE RESTRICT,
    UNIQUE (tenant_id, engineering_run_id, version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_engineering_run_revision_current
    ON multica.engineering_run_revision (tenant_id, engineering_run_id) WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS permission.engineering_run_role_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    user_id UUID NOT NULL CHECK (user_id <> '00000000-0000-0000-0000-000000000000'),
    role VARCHAR(32) NOT NULL CHECK (role IN ('tenant_admin','project_admin','developer','viewer','agent')),
    granted_by UUID NOT NULL DEFAULT NULLIF(current_setting('app.actor_id', true), '')::UUID,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, engineering_run_id)
        REFERENCES multica.engineering_run (tenant_id, engineering_run_id) ON DELETE RESTRICT,
    UNIQUE (tenant_id, engineering_run_id, user_id, version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_engineering_run_role_binding_current
    ON permission.engineering_run_role_binding (tenant_id, engineering_run_id, user_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_engineering_run_role_binding_subject
    ON permission.engineering_run_role_binding (tenant_id, user_id, engineering_run_id) WHERE valid_to IS NULL;

-- Existing PKs already make these added reference keys unique; the additional
-- keys let new FKs prove the complete tuple without making projections authority.
CREATE UNIQUE INDEX IF NOT EXISTS uq_worktree_project_binding_directory_tuple
    ON multica.worktree_project_binding (tenant_id, project_id, worktree_id, binding_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_worktree_repository_directory_tuple
    ON worktree_canvas_worktree (tenant_id, repo_id, id);

CREATE TABLE IF NOT EXISTS multica.engineering_run_worktree_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    repository_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    project_binding_id UUID NOT NULL,
    assigned_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    version INTEGER NOT NULL CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
        REFERENCES multica.engineering_run (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, worktree_id, project_binding_id)
        REFERENCES multica.worktree_project_binding (tenant_id, project_id, worktree_id, binding_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, repository_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, repo_id, id) ON DELETE RESTRICT,
    UNIQUE (tenant_id, worktree_id, version),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_engineering_run_worktree_binding_current
    ON multica.engineering_run_worktree_binding (tenant_id, worktree_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_engineering_run_worktree_directory
    ON multica.engineering_run_worktree_binding
        (tenant_id, engineering_run_id, worktree_id) WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.engineering_directory_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    repository_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID,
    source_table VARCHAR(64) NOT NULL CHECK (source_table IN (
        'scm.cloud_branch','scm.cloud_branch_revision','permission.cloud_branch_role_binding',
        'multica.engineering_run','multica.engineering_run_revision',
        'permission.engineering_run_role_binding','multica.engineering_run_worktree_binding')),
    entity_id UUID NOT NULL,
    action VARCHAR(16) NOT NULL CHECK (action IN ('created','closed')),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    before_state JSONB,
    after_state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, project_id, repository_id, branch_id)
        REFERENCES scm.cloud_branch (tenant_id, project_id, repository_id, branch_id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
        REFERENCES multica.engineering_run (tenant_id, project_id, repository_id, branch_id, engineering_run_id)
        ON DELETE RESTRICT,
    CHECK ((source_table IN ('scm.cloud_branch','scm.cloud_branch_revision','permission.cloud_branch_role_binding')
            AND engineering_run_id IS NULL)
        OR (source_table IN ('multica.engineering_run','multica.engineering_run_revision',
                'permission.engineering_run_role_binding','multica.engineering_run_worktree_binding')
            AND engineering_run_id IS NOT NULL)),
    CHECK ((action = 'created' AND before_state IS NULL) OR (action = 'closed' AND before_state IS NOT NULL)),
    CHECK (before_state IS NULL OR (jsonb_typeof(before_state) = 'object' AND octet_length(before_state::TEXT) <= 16384)),
    CHECK (jsonb_typeof(after_state) = 'object' AND octet_length(after_state::TEXT) <= 16384)
);
CREATE INDEX IF NOT EXISTS idx_engineering_directory_event_branch
    ON multica.engineering_directory_event (tenant_id, project_id, branch_id, created_at DESC, event_id DESC);
CREATE INDEX IF NOT EXISTS idx_engineering_directory_event_run
    ON multica.engineering_directory_event (tenant_id, engineering_run_id, created_at DESC, event_id DESC)
    WHERE engineering_run_id IS NOT NULL;

CREATE OR REPLACE FUNCTION multica.engineering_directory_scope(
    source_schema TEXT, source_name TEXT, row_data JSONB
) RETURNS TABLE (scope_project_id UUID, scope_repository_id UUID, scope_branch_id UUID, scope_run_id UUID)
LANGUAGE plpgsql AS $$
DECLARE
    row_tenant_id UUID := (row_data ->> 'tenant_id')::UUID;
BEGIN
    IF (source_schema = 'scm' AND source_name = 'cloud_branch')
       OR (source_schema = 'multica' AND source_name IN
           ('engineering_run','engineering_run_worktree_binding','engineering_directory_event')) THEN
        RETURN QUERY SELECT (row_data ->> 'project_id')::UUID, (row_data ->> 'repository_id')::UUID,
            (row_data ->> 'branch_id')::UUID, (row_data ->> 'engineering_run_id')::UUID;
    ELSIF (source_schema = 'scm' AND source_name = 'cloud_branch_revision')
       OR (source_schema = 'permission' AND source_name = 'cloud_branch_role_binding') THEN
        RETURN QUERY SELECT b.project_id, b.repository_id, b.branch_id, NULL::UUID
            FROM scm.cloud_branch b WHERE b.tenant_id = row_tenant_id
            AND b.branch_id = (row_data ->> 'branch_id')::UUID;
        IF NOT FOUND THEN RAISE EXCEPTION 'canonical Cloud Branch scope is unavailable'; END IF;
    ELSIF (source_schema = 'multica' AND source_name = 'engineering_run_revision')
       OR (source_schema = 'permission' AND source_name = 'engineering_run_role_binding') THEN
        RETURN QUERY SELECT r.project_id, r.repository_id, r.branch_id, r.engineering_run_id
            FROM multica.engineering_run r WHERE r.tenant_id = row_tenant_id
            AND r.engineering_run_id = (row_data ->> 'engineering_run_id')::UUID;
        IF NOT FOUND THEN RAISE EXCEPTION 'canonical Engineering Run scope is unavailable'; END IF;
    ELSE
        RAISE EXCEPTION 'unsupported engineering directory source';
    END IF;
END;
$$;

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
    IF scope_project_id IS NULL OR scope_repository_id IS NULL OR scope_branch_id IS NULL
       OR scope_project_id = '00000000-0000-0000-0000-000000000000'
       OR scope_repository_id = '00000000-0000-0000-0000-000000000000' THEN
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
            WHERE p.binding_id = NEW.project_binding_id AND p.tenant_id = NEW.tenant_id
              AND p.project_id = NEW.project_id AND p.worktree_id = NEW.worktree_id
              AND p.valid_from <= now() AND p.valid_to IS NULL
              AND w.project_id = p.project_id AND w.repo_id = NEW.repository_id
            FOR SHARE OF p, w;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'Engineering Run binding requires a current matching Project and repository Worktree relation';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.audit_engineering_directory_write()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    row_data JSONB := to_jsonb(NEW);
    scope_project_id UUID;
    scope_repository_id UUID;
    scope_branch_id UUID;
    scope_run_id UUID;
    row_entity_id UUID;
BEGIN
    SELECT s.scope_project_id, s.scope_repository_id, s.scope_branch_id, s.scope_run_id
        INTO scope_project_id, scope_repository_id, scope_branch_id, scope_run_id
        FROM multica.engineering_directory_scope(TG_TABLE_SCHEMA, TG_TABLE_NAME, row_data) s;
    row_entity_id := COALESCE((row_data ->> 'revision_id')::UUID, (row_data ->> 'binding_id')::UUID,
        (row_data ->> 'engineering_run_id')::UUID, (row_data ->> 'branch_id')::UUID);
    INSERT INTO multica.engineering_directory_event
        (tenant_id, project_id, repository_id, branch_id, engineering_run_id, source_table, entity_id,
         action, actor_id, correlation_id, before_state, after_state)
        VALUES (NEW.tenant_id, scope_project_id, scope_repository_id, scope_branch_id, scope_run_id,
            TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME, row_entity_id,
            CASE WHEN TG_OP = 'INSERT' THEN 'created' ELSE 'closed' END,
            NULLIF(current_setting('app.actor_id', true), '')::UUID,
            NULLIF(current_setting('app.correlation_id', true), '')::UUID,
            CASE WHEN TG_OP = 'INSERT' THEN NULL ELSE to_jsonb(OLD) END, row_data);
    RETURN NEW;
END;
$$;

DO $$
DECLARE
    directory_tables TEXT[] := ARRAY[
        'scm.cloud_branch','scm.cloud_branch_revision','permission.cloud_branch_role_binding',
        'multica.engineering_run','multica.engineering_run_revision',
        'permission.engineering_run_role_binding','multica.engineering_run_worktree_binding',
        'multica.engineering_directory_event'];
    target_table TEXT;
BEGIN
    FOREACH target_table IN ARRAY directory_tables LOOP
        EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', target_table);
        EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', target_table);
        EXECUTE format('DROP POLICY IF EXISTS engineering_directory_tenant_select ON %s', target_table);
        EXECUTE format('CREATE POLICY engineering_directory_tenant_select ON %s FOR SELECT
            USING (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID)', target_table);
        EXECUTE format('DROP POLICY IF EXISTS engineering_directory_tenant_insert ON %s', target_table);
        EXECUTE format('CREATE POLICY engineering_directory_tenant_insert ON %s FOR INSERT
            WITH CHECK (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID)', target_table);
        EXECUTE format('DROP POLICY IF EXISTS engineering_directory_tenant_update ON %s', target_table);
        IF target_table NOT IN ('scm.cloud_branch','multica.engineering_run','multica.engineering_directory_event') THEN
            EXECUTE format('CREATE POLICY engineering_directory_tenant_update ON %s FOR UPDATE
                USING (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID)
                WITH CHECK (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID)', target_table);
        END IF;
        EXECUTE format('DROP TRIGGER IF EXISTS engineering_directory_write_guard ON %s', target_table);
        EXECUTE format('CREATE TRIGGER engineering_directory_write_guard BEFORE INSERT OR UPDATE OR DELETE
            ON %s FOR EACH ROW EXECUTE FUNCTION multica.guard_engineering_directory_write()', target_table);
        IF target_table <> 'multica.engineering_directory_event' THEN
            EXECUTE format('DROP TRIGGER IF EXISTS engineering_directory_write_audit ON %s', target_table);
            EXECUTE format('CREATE TRIGGER engineering_directory_write_audit AFTER INSERT OR UPDATE
                ON %s FOR EACH ROW EXECUTE FUNCTION multica.audit_engineering_directory_write()', target_table);
        END IF;
    END LOOP;
END;
$$;

COMMENT ON TABLE scm.cloud_branch IS
    'T: immutable canonical SCM Branch registration facts; no local Worktree branch-name inference, physical delete or implicit grants.';
COMMENT ON COLUMN scm.cloud_branch.remote_identity IS
    'Opaque provider branch reference supplied by a trusted SCM ingest producer; never a URL, credential or locally inferred branch name.';
COMMENT ON TABLE scm.cloud_branch_revision IS
    'M/SCD2: Cloud Branch metadata; close once, append a new version; future current rows and historical rewrites are rejected.';
COMMENT ON TABLE permission.cloud_branch_role_binding IS
    'M/SCD2: explicit Cloud Branch read authority; Project management permission and owner metadata are not implicit grants.';
COMMENT ON TABLE multica.engineering_run IS
    'T: immutable Engineering Run collaboration-workspace registration, distinct from a task execution attempt; this phase provides no execution admission.';
COMMENT ON TABLE multica.engineering_run_revision IS
    'M/SCD2: Engineering Run metadata; owner_user_id is human management metadata, never an implicit capability or a Worktree/runtime owner.';
COMMENT ON TABLE permission.engineering_run_role_binding IS
    'M/SCD2: explicit Engineering Run authority; consumers must also revalidate current Project membership and Cloud Branch grant.';
COMMENT ON TABLE multica.engineering_run_worktree_binding IS
    'M/SCD2: authoritative Run-to-checkout relation, one current Run per Worktree; preserve the exact Project binding and repository tuple.';
COMMENT ON COLUMN multica.engineering_run_worktree_binding.project_binding_id IS
    'Exact historical Project binding identity. Reads must revalidate p.valid_from <= now(), p.valid_to IS NULL and the matching Worktree Project/repository projection.';
COMMENT ON TABLE multica.engineering_directory_event IS
    'T: bounded append-only audit of Branch/Run registration, SCD2 changes and explicit grants; this is not an outbox delivery queue.';

COMMIT;
