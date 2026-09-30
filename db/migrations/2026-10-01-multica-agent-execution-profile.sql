-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-10-01-multica-agent-execution-profile.sql",type:"file",language:"sql"}),
--   (m:Module {name:"multica_agent_execution_profile_schema",type:"module",language:"sql"}),
--   (profile:Class {name:"multica.agent_execution_profile",type:"class",language:"sql"}),
--   (audit:Class {name:"multica.agent_execution_profile_audit_event",type:"class",language:"sql"}),
--   (scd_guard:Function {name:"multica.guard_agent_execution_profile_scd2",type:"function",language:"sql"}),
--   (successor_guard:Function {name:"multica.require_agent_execution_profile_successor",type:"function",language:"sql"}),
--   (audit_scope:Function {name:"multica.validate_agent_execution_profile_audit_scope",type:"function",language:"sql"}),
--   (audit_required:Function {name:"multica.require_agent_execution_profile_audit",type:"function",language:"sql"}),
--   (audit_mutation:Function {name:"multica.reject_agent_execution_profile_audit_mutation",type:"function",language:"sql"}),
--   (profile_rls:Logic {name:"agent_execution_profile_tenant_rls",type:"logic",language:"sql"}),
--   (audit_rls:Logic {name:"agent_execution_profile_audit_tenant_rls",type:"logic",language:"sql"}),
--   (m)-[:CONTAINS]->(profile),(m)-[:CONTAINS]->(audit),
--   (m)-[:CONTAINS]->(scd_guard),(m)-[:CONTAINS]->(successor_guard),
--   (m)-[:CONTAINS]->(audit_scope),(m)-[:CONTAINS]->(audit_required),
--   (m)-[:CONTAINS]->(audit_mutation),(m)-[:CONTAINS]->(profile_rls),
--   (m)-[:CONTAINS]->(audit_rls),(scd_guard)-[:USES]->(profile),
--   (successor_guard)-[:USES]->(profile),(audit_scope)-[:USES]->(profile),
--   (audit_scope)-[:USES]->(audit),(audit_required)-[:USES]->(profile),
--   (audit_required)-[:USES]->(audit),(audit_mutation)-[:USES]->(audit),
--   (profile_rls)-[:USES]->(profile),(audit_rls)-[:USES]->(audit);

-- Phase 9E-3. W/T/M: Profile is immutable SCD2 Master; its lifecycle Audit is Transaction.
-- Run rows keep a self-contained snapshot and intentionally do not reference the mutable Master.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.agent_execution_profile (
    profile_revision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    scope_kind VARCHAR(12) NOT NULL CHECK (scope_kind IN ('project','worktree')),
    worktree_id UUID,
    profile_version BIGINT NOT NULL CHECK (profile_version > 0),
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    lifecycle_state VARCHAR(12) NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active','disabled')),
    content_digest CHAR(64) NOT NULL
        CHECK (content_digest::text ~ '^[0-9a-f]{64}$'),
    profile_document JSONB NOT NULL
        CHECK (jsonb_typeof(profile_document) = 'object')
        CHECK (octet_length(profile_document::text) <= 131072),
    changed_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, profile_id, profile_version),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (changed_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (worktree_id IS NULL OR worktree_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (scope_kind = 'project' AND worktree_id IS NULL)
        OR (scope_kind = 'worktree' AND worktree_id IS NOT NULL)
    ),
    CHECK (valid_to IS NULL OR valid_to > valid_from),
    CHECK (
        profile_document #>> '{profile,scope,tenant_id}' IS NOT DISTINCT FROM tenant_id::text
    ),
    CHECK (
        profile_document #>> '{profile,scope,project_id}' IS NOT DISTINCT FROM project_id::text
    ),
    CHECK (
        profile_document #>> '{profile,scope,worktree_id}' IS NOT DISTINCT FROM worktree_id::text
    ),
    CHECK (
        profile_document #>> '{profile,schema_version}' IS NOT DISTINCT FROM schema_version::text
    ),
    CHECK (
        profile_document ->> 'content_digest' IS NOT DISTINCT FROM content_digest::text
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_agent_execution_profile_current
    ON multica.agent_execution_profile (tenant_id, profile_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_agent_execution_profile_scope
    ON multica.agent_execution_profile
        (tenant_id, project_id, scope_kind, worktree_id, profile_id, profile_version DESC);

CREATE TABLE IF NOT EXISTS multica.agent_execution_profile_audit_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    profile_version BIGINT NOT NULL CHECK (profile_version > 0),
    scope_kind VARCHAR(12) NOT NULL CHECK (scope_kind IN ('project','worktree')),
    worktree_id UUID,
    event_type VARCHAR(24) NOT NULL CHECK (event_type IN (
        'profile_published','profile_rolled_back','profile_disabled','profile_reenabled'
    )),
    source_profile_version BIGINT,
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, profile_id, profile_version),
    FOREIGN KEY (tenant_id, profile_id, profile_version)
        REFERENCES multica.agent_execution_profile (tenant_id, profile_id, profile_version)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, profile_id, source_profile_version)
        REFERENCES multica.agent_execution_profile (tenant_id, profile_id, profile_version)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (scope_kind = 'project' AND worktree_id IS NULL)
        OR (scope_kind = 'worktree' AND worktree_id IS NOT NULL)
    ),
    CHECK (
        (event_type = 'profile_rolled_back'
            AND source_profile_version IS NOT NULL
            AND source_profile_version < profile_version)
        OR (event_type <> 'profile_rolled_back' AND source_profile_version IS NULL)
    )
);
CREATE INDEX IF NOT EXISTS idx_agent_execution_profile_audit_scope
    ON multica.agent_execution_profile_audit_event
        (tenant_id, project_id, profile_id, occurred_at DESC, event_id);

CREATE OR REPLACE FUNCTION multica.guard_agent_execution_profile_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    latest_version BIGINT;
    previous_project_id UUID;
    previous_scope_kind VARCHAR(12);
    previous_worktree_id UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Agent Execution Profile Master rows cannot be deleted';
    END IF;

    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_id::text || ':' || NEW.profile_id::text, 0)
    );

    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL
           OR NEW.valid_to IS NULL
           OR NEW.valid_to <= OLD.valid_from
           OR (to_jsonb(OLD) - 'valid_to') IS DISTINCT FROM (to_jsonb(NEW) - 'valid_to') THEN
            RAISE EXCEPTION 'Agent Execution Profile rows may only be closed once without rewriting history';
        END IF;
        RETURN NEW;
    END IF;

    IF TG_OP <> 'INSERT' OR NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'Agent Execution Profile revisions must be inserted as current rows';
    END IF;

    SELECT COALESCE(MAX(profile_version), 0)
      INTO latest_version
      FROM multica.agent_execution_profile
     WHERE tenant_id = NEW.tenant_id AND profile_id = NEW.profile_id;

    IF NEW.profile_version <> latest_version + 1 THEN
        RAISE EXCEPTION 'Agent Execution Profile revision must increment by one';
    END IF;
    IF EXISTS (
        SELECT 1 FROM multica.agent_execution_profile
         WHERE tenant_id = NEW.tenant_id AND profile_id = NEW.profile_id AND valid_to IS NULL
    ) THEN
        RAISE EXCEPTION 'Close the current Agent Execution Profile revision before inserting a successor';
    END IF;

    IF latest_version > 0 THEN
        SELECT project_id, scope_kind, worktree_id
          INTO previous_project_id, previous_scope_kind, previous_worktree_id
          FROM multica.agent_execution_profile
         WHERE tenant_id = NEW.tenant_id AND profile_id = NEW.profile_id
         ORDER BY profile_version DESC
         LIMIT 1;
        IF previous_project_id IS DISTINCT FROM NEW.project_id
           OR previous_scope_kind IS DISTINCT FROM NEW.scope_kind
           OR previous_worktree_id IS DISTINCT FROM NEW.worktree_id THEN
            RAISE EXCEPTION 'Agent Execution Profile scope is immutable; create a new profile ID to move scope';
        END IF;
    END IF;

    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS agent_execution_profile_scd2_guard
    ON multica.agent_execution_profile;
CREATE TRIGGER agent_execution_profile_scd2_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.agent_execution_profile
    FOR EACH ROW EXECUTE FUNCTION multica.guard_agent_execution_profile_scd2();

CREATE OR REPLACE FUNCTION multica.require_agent_execution_profile_successor()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.valid_to IS NULL AND NEW.valid_to IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM multica.agent_execution_profile
         WHERE tenant_id = OLD.tenant_id
           AND profile_id = OLD.profile_id
           AND profile_version = OLD.profile_version + 1
           AND project_id = OLD.project_id
           AND scope_kind = OLD.scope_kind
           AND worktree_id IS NOT DISTINCT FROM OLD.worktree_id
    ) THEN
        RAISE EXCEPTION 'Closing an Agent Execution Profile revision requires a successor in the same transaction';
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS agent_execution_profile_successor_guard
    ON multica.agent_execution_profile;
CREATE CONSTRAINT TRIGGER agent_execution_profile_successor_guard
    AFTER UPDATE OF valid_to ON multica.agent_execution_profile
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION multica.require_agent_execution_profile_successor();

CREATE OR REPLACE FUNCTION multica.validate_agent_execution_profile_audit_scope()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    profile_project_id UUID;
    profile_scope_kind VARCHAR(12);
    profile_worktree_id UUID;
    profile_lifecycle_state VARCHAR(12);
BEGIN
    SELECT project_id, scope_kind, worktree_id, lifecycle_state
      INTO profile_project_id, profile_scope_kind, profile_worktree_id, profile_lifecycle_state
      FROM multica.agent_execution_profile
     WHERE tenant_id = NEW.tenant_id
       AND profile_id = NEW.profile_id
       AND profile_version = NEW.profile_version;

    IF NOT FOUND
       OR profile_project_id IS DISTINCT FROM NEW.project_id
       OR profile_scope_kind IS DISTINCT FROM NEW.scope_kind
       OR profile_worktree_id IS DISTINCT FROM NEW.worktree_id THEN
        RAISE EXCEPTION 'Agent Execution Profile Audit scope must match its revision';
    END IF;
    IF NEW.event_type = 'profile_disabled' AND profile_lifecycle_state <> 'disabled' THEN
        RAISE EXCEPTION 'profile_disabled Audit requires a disabled revision';
    END IF;
    IF NEW.event_type IN ('profile_published','profile_rolled_back','profile_reenabled')
       AND profile_lifecycle_state <> 'active' THEN
        RAISE EXCEPTION 'publish/rollback/reenable Audit requires an active revision';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS agent_execution_profile_audit_scope_guard
    ON multica.agent_execution_profile_audit_event;
CREATE TRIGGER agent_execution_profile_audit_scope_guard
    BEFORE INSERT ON multica.agent_execution_profile_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.validate_agent_execution_profile_audit_scope();

CREATE OR REPLACE FUNCTION multica.require_agent_execution_profile_audit()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM multica.agent_execution_profile_audit_event
         WHERE tenant_id = NEW.tenant_id
           AND profile_id = NEW.profile_id
           AND profile_version = NEW.profile_version
    ) THEN
        RAISE EXCEPTION 'Every Agent Execution Profile revision requires an Audit event in the same transaction';
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS agent_execution_profile_audit_required
    ON multica.agent_execution_profile;
CREATE CONSTRAINT TRIGGER agent_execution_profile_audit_required
    AFTER INSERT ON multica.agent_execution_profile
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION multica.require_agent_execution_profile_audit();

CREATE OR REPLACE FUNCTION multica.reject_agent_execution_profile_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Agent Execution Profile Audit facts are append-only';
END;
$$;
DROP TRIGGER IF EXISTS agent_execution_profile_audit_no_mutation
    ON multica.agent_execution_profile_audit_event;
CREATE TRIGGER agent_execution_profile_audit_no_mutation
    BEFORE UPDATE OR DELETE ON multica.agent_execution_profile_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_agent_execution_profile_audit_mutation();
DROP TRIGGER IF EXISTS agent_execution_profile_audit_no_truncate
    ON multica.agent_execution_profile_audit_event;
CREATE TRIGGER agent_execution_profile_audit_no_truncate
    BEFORE TRUNCATE ON multica.agent_execution_profile_audit_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_agent_execution_profile_audit_mutation();

ALTER TABLE multica.agent_execution_profile ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.agent_execution_profile FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS agent_execution_profile_tenant_scope
    ON multica.agent_execution_profile;
CREATE POLICY agent_execution_profile_tenant_scope
    ON multica.agent_execution_profile
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.agent_execution_profile_audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.agent_execution_profile_audit_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS agent_execution_profile_audit_tenant_scope
    ON multica.agent_execution_profile_audit_event;
CREATE POLICY agent_execution_profile_audit_tenant_scope
    ON multica.agent_execution_profile_audit_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.agent_execution_profile IS
    'M: immutable Project/Worktree scoped AgentExecutionProfile revisions; active/disabled changes are SCD2 successors.';
COMMENT ON TABLE multica.agent_execution_profile_audit_event IS
    'T: append-only publish/rollback/enable/disable audit; details contain sanitized metadata only.';
COMMENT ON COLUMN multica.agent_execution_profile.profile_document IS
    'Verified schema-versioned AgentExecutionProfileDocument; Rust recomputes SHA-256 on every read.';
COMMENT ON COLUMN multica.agent_execution_profile.lifecycle_state IS
    'Revision-scoped admission state; changing active/disabled requires a new Profile version.';

COMMIT;
