-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-09-30-multica-hook-policy.sql",type:"file",language:"sql"}),
--   (m:Module {name:"multica_hook_policy_schema",type:"module",language:"sql"}),
--   (policy:Class {name:"multica.hook_policy_set",type:"class",language:"sql"}),
--   (draft:Class {name:"multica.hook_policy_draft",type:"class",language:"sql"}),
--   (audit:Class {name:"multica.hook_policy_audit_event",type:"class",language:"sql"}),
--   (scd:Function {name:"multica.guard_hook_policy_set_scd2",type:"function",language:"sql"}),
--   (append:Function {name:"multica.reject_hook_policy_audit_mutation",type:"function",language:"sql"}),
--   (audit_scope:Function {name:"multica.validate_hook_policy_audit_scope",type:"function",language:"sql"}),
--   (rls:Logic {name:"hook_policy_tenant_rls",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(policy),(m)-[:CONTAINS]->(draft),
--   (m)-[:CONTAINS]->(audit),(m)-[:CONTAINS]->(scd),
--   (m)-[:CONTAINS]->(append),(m)-[:CONTAINS]->(rls),(scd)-[:GUARDS]->(policy),
--   (m)-[:CONTAINS]->(audit_scope),
--   (append)-[:GUARDS]->(audit),(rls)-[:SCOPES]->(policy),
--   (audit_scope)-[:VALIDATES]->(audit),
--   (rls)-[:SCOPES]->(draft),(rls)-[:SCOPES]->(audit);

-- Phase 9B2A: authoritative Hook policy persistence substrate.
-- M: immutable Project baseline and Worktree restrictive overlay revisions (SCD2).
-- W: expiring drafts; they are never read by the runtime evaluator.
-- T: append-only policy publication/rollback audit events.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.hook_policy_set (
    policy_set_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    scope_kind VARCHAR(12) NOT NULL CHECK (scope_kind IN ('project','worktree')),
    worktree_id UUID,
    policy_version BIGINT NOT NULL CHECK (policy_version > 0),
    schema_version SMALLINT NOT NULL CHECK (schema_version > 0),
    evaluator_api_version SMALLINT NOT NULL CHECK (evaluator_api_version > 0),
    digest BYTEA NOT NULL CHECK (octet_length(digest) = 32),
    policy_document JSONB NOT NULL
        CHECK (jsonb_typeof(policy_document) = 'object')
        CHECK (octet_length(policy_document::text) <= 65536),
    inherited_project_policy_set_id UUID,
    published_by UUID NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, policy_set_id),
    FOREIGN KEY (tenant_id, inherited_project_policy_set_id)
        REFERENCES multica.hook_policy_set (tenant_id, policy_set_id) ON DELETE RESTRICT,
    CHECK (
        (scope_kind = 'project' AND worktree_id IS NULL AND inherited_project_policy_set_id IS NULL)
        OR
        (scope_kind = 'worktree' AND worktree_id IS NOT NULL
            AND inherited_project_policy_set_id IS NOT NULL)
    ),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_hook_policy_project_current
    ON multica.hook_policy_set (tenant_id, project_id)
    WHERE scope_kind = 'project' AND valid_to IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_hook_policy_worktree_current
    ON multica.hook_policy_set (tenant_id, project_id, worktree_id)
    WHERE scope_kind = 'worktree' AND valid_to IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_hook_policy_project_version
    ON multica.hook_policy_set (tenant_id, project_id, policy_version)
    WHERE scope_kind = 'project';
CREATE UNIQUE INDEX IF NOT EXISTS uq_hook_policy_worktree_version
    ON multica.hook_policy_set (tenant_id, project_id, worktree_id, policy_version)
    WHERE scope_kind = 'worktree';
CREATE INDEX IF NOT EXISTS idx_hook_policy_inherited_project
    ON multica.hook_policy_set (tenant_id, inherited_project_policy_set_id)
    WHERE inherited_project_policy_set_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS multica.hook_policy_draft (
    draft_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    scope_kind VARCHAR(12) NOT NULL CHECK (scope_kind IN ('project','worktree')),
    worktree_id UUID,
    base_policy_set_id UUID,
    inherited_project_policy_set_id UUID,
    draft_state VARCHAR(16) NOT NULL DEFAULT 'editing'
        CHECK (draft_state IN ('editing','pending_approval')),
    draft_version BIGINT NOT NULL DEFAULT 1 CHECK (draft_version > 0),
    policy_document JSONB NOT NULL
        CHECK (jsonb_typeof(policy_document) = 'object')
        CHECK (octet_length(policy_document::text) <= 65536),
    created_by UUID NOT NULL,
    updated_by UUID NOT NULL,
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '7 days'
        CHECK (retention_period > INTERVAL '0 seconds'
            AND retention_period <= INTERVAL '30 days'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '7 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, draft_id),
    FOREIGN KEY (tenant_id, base_policy_set_id)
        REFERENCES multica.hook_policy_set (tenant_id, policy_set_id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, inherited_project_policy_set_id)
        REFERENCES multica.hook_policy_set (tenant_id, policy_set_id) ON DELETE RESTRICT,
    CHECK (
        (scope_kind = 'project' AND worktree_id IS NULL AND inherited_project_policy_set_id IS NULL)
        OR
        (scope_kind = 'worktree' AND worktree_id IS NOT NULL
            AND inherited_project_policy_set_id IS NOT NULL)
    ),
    CHECK (expires_at > created_at AND expires_at <= created_at + retention_period)
);
CREATE INDEX IF NOT EXISTS idx_hook_policy_draft_expiry
    ON multica.hook_policy_draft (expires_at);
CREATE INDEX IF NOT EXISTS idx_hook_policy_draft_scope
    ON multica.hook_policy_draft (tenant_id, project_id, scope_kind, worktree_id, updated_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS uq_hook_policy_project_draft
    ON multica.hook_policy_draft (tenant_id, project_id)
    WHERE scope_kind = 'project' AND draft_state IN ('editing','pending_approval');
CREATE UNIQUE INDEX IF NOT EXISTS uq_hook_policy_worktree_draft
    ON multica.hook_policy_draft (tenant_id, project_id, worktree_id)
    WHERE scope_kind = 'worktree' AND draft_state IN ('editing','pending_approval');

CREATE TABLE IF NOT EXISTS multica.hook_policy_audit_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    scope_kind VARCHAR(12) NOT NULL CHECK (scope_kind IN ('project','worktree')),
    worktree_id UUID,
    policy_set_id UUID,
    draft_id UUID,
    actor_id UUID NOT NULL,
    event_type VARCHAR(24) NOT NULL CHECK (event_type IN (
        'draft_saved','policy_published','policy_rolled_back','policy_rejected'
    )),
    policy_version BIGINT CHECK (policy_version IS NULL OR policy_version > 0),
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, policy_set_id)
        REFERENCES multica.hook_policy_set (tenant_id, policy_set_id) ON DELETE RESTRICT,
    CHECK (
        (scope_kind = 'project' AND worktree_id IS NULL)
        OR (scope_kind = 'worktree' AND worktree_id IS NOT NULL)
    ),
    CHECK (policy_set_id IS NOT NULL OR draft_id IS NOT NULL),
    CHECK (
        (event_type = 'draft_saved' AND draft_id IS NOT NULL)
        OR (event_type IN ('policy_published','policy_rolled_back') AND policy_set_id IS NOT NULL)
        OR (event_type = 'policy_rejected' AND (policy_set_id IS NOT NULL OR draft_id IS NOT NULL))
    )
);
CREATE INDEX IF NOT EXISTS idx_hook_policy_audit_scope_time
    ON multica.hook_policy_audit_event
        (tenant_id, project_id, scope_kind, worktree_id, occurred_at DESC, event_id);
CREATE INDEX IF NOT EXISTS idx_hook_policy_audit_version
    ON multica.hook_policy_audit_event (tenant_id, policy_set_id, occurred_at DESC)
    WHERE policy_set_id IS NOT NULL;

CREATE OR REPLACE FUNCTION multica.validate_hook_policy_audit_scope()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    policy_project_id UUID;
    policy_scope_kind VARCHAR(12);
    policy_worktree_id UUID;
BEGIN
    IF NEW.policy_set_id IS NULL THEN
        RETURN NEW;
    END IF;

    SELECT project_id, scope_kind, worktree_id
      INTO policy_project_id, policy_scope_kind, policy_worktree_id
      FROM multica.hook_policy_set
     WHERE tenant_id = NEW.tenant_id
       AND policy_set_id = NEW.policy_set_id;

    IF NOT FOUND
       OR policy_project_id IS DISTINCT FROM NEW.project_id
       OR policy_scope_kind IS DISTINCT FROM NEW.scope_kind
       OR policy_worktree_id IS DISTINCT FROM NEW.worktree_id THEN
        RAISE EXCEPTION 'Hook policy audit scope must match its referenced policy version';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS hook_policy_audit_scope_guard ON multica.hook_policy_audit_event;
CREATE TRIGGER hook_policy_audit_scope_guard
    BEFORE INSERT ON multica.hook_policy_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.validate_hook_policy_audit_scope();

CREATE OR REPLACE FUNCTION multica.guard_hook_policy_set_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    parent_scope_kind VARCHAR(12);
    parent_project_id UUID;
    parent_valid_to TIMESTAMPTZ;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'published Hook policy versions cannot be physically deleted';
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL
           OR NEW.valid_to <= OLD.valid_from
           OR (to_jsonb(NEW) - 'valid_to') IS DISTINCT FROM (to_jsonb(OLD) - 'valid_to') THEN
            RAISE EXCEPTION 'Hook policy versions may only be closed once without rewriting history';
        END IF;
        IF OLD.scope_kind = 'project' AND EXISTS (
            SELECT 1
              FROM multica.hook_policy_set AS worktree_policy
             WHERE worktree_policy.tenant_id = OLD.tenant_id
               AND worktree_policy.project_id = OLD.project_id
               AND worktree_policy.scope_kind = 'worktree'
               AND worktree_policy.inherited_project_policy_set_id = OLD.policy_set_id
               AND worktree_policy.valid_to IS NULL
        ) THEN
            RAISE EXCEPTION 'Current Worktree Hook overlays must be closed and rebased before their Project baseline changes';
        END IF;
        RETURN NEW;
    END IF;

    IF NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'new Hook policy versions must begin as the current open revision';
    END IF;

    IF NEW.scope_kind = 'worktree' THEN
        SELECT scope_kind, project_id, valid_to
          INTO parent_scope_kind, parent_project_id, parent_valid_to
          FROM multica.hook_policy_set
         WHERE tenant_id = NEW.tenant_id
           AND policy_set_id = NEW.inherited_project_policy_set_id
         FOR SHARE;
        IF NOT FOUND OR parent_scope_kind <> 'project'
           OR parent_project_id <> NEW.project_id OR parent_valid_to IS NOT NULL THEN
            RAISE EXCEPTION 'Worktree Hook policy must inherit the current Project policy for the same Project';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS hook_policy_set_scd2_guard ON multica.hook_policy_set;
CREATE TRIGGER hook_policy_set_scd2_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.hook_policy_set
    FOR EACH ROW EXECUTE FUNCTION multica.guard_hook_policy_set_scd2();

CREATE OR REPLACE FUNCTION multica.reject_hook_policy_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Hook policy audit facts are append-only';
END;
$$;
DROP TRIGGER IF EXISTS hook_policy_audit_no_mutation ON multica.hook_policy_audit_event;
CREATE TRIGGER hook_policy_audit_no_mutation
    BEFORE UPDATE OR DELETE ON multica.hook_policy_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_hook_policy_audit_mutation();
DROP TRIGGER IF EXISTS hook_policy_audit_no_truncate ON multica.hook_policy_audit_event;
CREATE TRIGGER hook_policy_audit_no_truncate
    BEFORE TRUNCATE ON multica.hook_policy_audit_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_hook_policy_audit_mutation();

ALTER TABLE multica.hook_policy_set ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.hook_policy_set FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS hook_policy_set_tenant_scope ON multica.hook_policy_set;
CREATE POLICY hook_policy_set_tenant_scope ON multica.hook_policy_set
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.hook_policy_draft ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.hook_policy_draft FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS hook_policy_draft_tenant_scope ON multica.hook_policy_draft;
CREATE POLICY hook_policy_draft_tenant_scope ON multica.hook_policy_draft
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.hook_policy_audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.hook_policy_audit_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS hook_policy_audit_tenant_scope ON multica.hook_policy_audit_event;
CREATE POLICY hook_policy_audit_tenant_scope ON multica.hook_policy_audit_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.hook_policy_set IS
    'Published versioned Project baseline or Worktree restrictive overlay; immutable SCD2 Master data.';
COMMENT ON TABLE multica.hook_policy_draft IS
    'Short-lived Worktree Hook policy editing state; runtime evaluators must never read drafts.';
COMMENT ON TABLE multica.hook_policy_audit_event IS
    'Append-only audit for draft, publish, rollback and rejection; details contain sanitized metadata only.';
COMMENT ON COLUMN multica.hook_policy_audit_event.draft_id IS
    'Draft provenance identifier; intentionally not a foreign key because expired W drafts are physically purged.';

COMMIT;
