-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-6-plugin-registry.sql",type:"file",language:"sql"}),
--   (m:Module {name:"plugin_group_app_registry",type:"module",language:"sql"}),
--   (manifest:Class {name:"plugin.group_app_manifest",type:"class",language:"sql"}),
--   (registry:Class {name:"plugin.group_app_registry_state",type:"class",language:"sql"}),
--   (binding:Class {name:"plugin.group_app_binding",type:"class",language:"sql"}),
--   (grant:Class {name:"plugin.group_app_access_grant",type:"class",language:"sql"}),
--   (audit:Class {name:"plugin.group_app_audit_event",type:"class",language:"sql"}),
--   (rls:Logic {name:"plugin_group_app_rls",type:"logic",language:"sql"}),
--   (append:Logic {name:"plugin_group_app_audit_append_only",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(manifest),(m)-[:CONTAINS]->(registry),
--   (m)-[:CONTAINS]->(binding),(m)-[:CONTAINS]->(grant),(m)-[:CONTAINS]->(audit),
--   (m)-[:CONTAINS]->(rls),(m)-[:CONTAINS]->(append);

-- Phase 6 Group App Registry: versioned tenant Master data plus append-only audit.
-- Registry ingestion / lifecycle commands are intentionally separate from this read model.

BEGIN;

CREATE SCHEMA IF NOT EXISTS plugin;

-- M: signed manifest versions are immutable identities. verification_status is set only by
-- the trusted ingestion process; this migration does not establish the publisher trust root.
CREATE TABLE IF NOT EXISTS plugin.group_app_manifest (
    manifest_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    plugin_id VARCHAR(64) NOT NULL
        CHECK (plugin_id ~ '^[a-z0-9][a-z0-9._-]{0,63}$'),
    manifest_version VARCHAR(128) NOT NULL
        CHECK (length(btrim(manifest_version)) > 0),
    label VARCHAR(160) NOT NULL
        CHECK (length(btrim(label)) > 0),
    publisher VARCHAR(200) NOT NULL
        CHECK (length(btrim(publisher)) > 0),
    manifest_digest BYTEA NOT NULL
        CHECK (octet_length(manifest_digest) = 32),
    signature_ref TEXT NOT NULL
        CHECK (length(btrim(signature_ref)) > 0),
    verification_status VARCHAR(16) NOT NULL DEFAULT 'pending'
        CHECK (verification_status IN ('pending','verified','rejected','revoked')),
    host_api_min INTEGER NOT NULL CHECK (host_api_min >= 1),
    host_api_max INTEGER NOT NULL CHECK (host_api_max >= host_api_min),
    manifest JSONB NOT NULL,
    verified_at TIMESTAMPTZ,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, manifest_id),
    CHECK ((verification_status = 'verified') = (verified_at IS NOT NULL)),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_group_app_manifest_current_version
    ON plugin.group_app_manifest (tenant_id, plugin_id, manifest_version)
    WHERE valid_to IS NULL;

-- M/SCD2: one current registry revision per Worktree; a lifecycle command closes the current
-- row and inserts a successor in the same transaction.
CREATE TABLE IF NOT EXISTS plugin.group_app_registry_state (
    registry_state_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    registry_version BIGINT NOT NULL DEFAULT 1 CHECK (registry_version > 0),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_to >= valid_from),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_group_app_registry_state_current
    ON plugin.group_app_registry_state (tenant_id, worktree_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_group_app_registry_project
    ON plugin.group_app_registry_state (tenant_id, project_id, worktree_id, valid_from DESC);

-- M: SCD2 Group/Worktree installation. An active row is a navigation candidate only;
-- it never grants a plugin capability by itself.
CREATE TABLE IF NOT EXISTS plugin.group_app_binding (
    binding_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    plugin_id VARCHAR(64) NOT NULL,
    manifest_id UUID NOT NULL,
    lifecycle_state VARCHAR(16) NOT NULL
        CHECK (lifecycle_state IN ('registered','validating','configuring','active','degraded','draining','disabled')),
    sort_order INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    assigned_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_to >= valid_from),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, manifest_id)
        REFERENCES plugin.group_app_manifest (tenant_id, manifest_id) ON DELETE RESTRICT,
    UNIQUE (tenant_id, binding_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_group_app_binding_active_plugin
    ON plugin.group_app_binding (tenant_id, worktree_id, plugin_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_group_app_binding_projection
    ON plugin.group_app_binding (tenant_id, project_id, worktree_id, lifecycle_state, sort_order, plugin_id)
    WHERE valid_to IS NULL;

-- M: explicit actor grant for opening a Group App. Tool/data capabilities remain separate
-- grants checked by the plugin gateway for each call.
CREATE TABLE IF NOT EXISTS plugin.group_app_access_grant (
    grant_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    plugin_id VARCHAR(64) NOT NULL,
    binding_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    capability VARCHAR(64) NOT NULL CHECK (capability = 'group_app:open'),
    granted_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_to >= valid_from),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, binding_id)
        REFERENCES plugin.group_app_binding (tenant_id, binding_id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_group_app_access_grant_active
    ON plugin.group_app_access_grant (tenant_id, worktree_id, plugin_id, actor_id, capability)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_group_app_access_grant_actor
    ON plugin.group_app_access_grant (tenant_id, actor_id, worktree_id, plugin_id)
    WHERE valid_to IS NULL;

-- T: audit lifecycle facts without manifest body, capability secret, or user content.
CREATE TABLE IF NOT EXISTS plugin.group_app_audit_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    plugin_id VARCHAR(64) NOT NULL,
    actor_id UUID NOT NULL,
    event_type VARCHAR(48) NOT NULL
        CHECK (event_type IN ('manifest.registered','manifest.verified','binding.changed','access.granted','access.revoked','runtime.changed')),
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_group_app_audit_worktree_time
    ON plugin.group_app_audit_event (tenant_id, worktree_id, occurred_at DESC, event_id);

CREATE OR REPLACE FUNCTION plugin.reject_group_app_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'group app audit facts are append-only';
END;
$$;
DROP TRIGGER IF EXISTS group_app_audit_no_mutation ON plugin.group_app_audit_event;
CREATE TRIGGER group_app_audit_no_mutation
    BEFORE UPDATE OR DELETE ON plugin.group_app_audit_event
    FOR EACH ROW EXECUTE FUNCTION plugin.reject_group_app_audit_mutation();

-- Master records are retained. A row may only be closed once through valid_to; all other
-- changes create a successor row so historical registry decisions remain explainable.
CREATE OR REPLACE FUNCTION plugin.guard_group_app_master_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'group app Master rows cannot be physically deleted';
    END IF;
    IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL
       OR (to_jsonb(NEW) - 'valid_to') IS DISTINCT FROM (to_jsonb(OLD) - 'valid_to') THEN
        RAISE EXCEPTION 'group app Master rows may only be closed once';
    END IF;
    RETURN NEW;
END;
$$;
DO $$
DECLARE table_name TEXT;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'group_app_manifest', 'group_app_registry_state', 'group_app_binding', 'group_app_access_grant'
    ] LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON plugin.%I', table_name || '_scd2_guard', table_name);
        EXECUTE format(
            'CREATE TRIGGER %I BEFORE UPDATE OR DELETE ON plugin.%I FOR EACH ROW EXECUTE FUNCTION plugin.guard_group_app_master_scd2()',
            table_name || '_scd2_guard', table_name
        );
    END LOOP;
END;
$$;

-- FORCE RLS on every table. This migration grants no write policy to the application role;
-- a later, separately authorized lifecycle command will add its narrow write path.
ALTER TABLE plugin.group_app_manifest ENABLE ROW LEVEL SECURITY;
ALTER TABLE plugin.group_app_manifest FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_app_manifest_tenant_select ON plugin.group_app_manifest;
CREATE POLICY group_app_manifest_tenant_select ON plugin.group_app_manifest
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE plugin.group_app_registry_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE plugin.group_app_registry_state FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_app_registry_state_tenant_select ON plugin.group_app_registry_state;
CREATE POLICY group_app_registry_state_tenant_select ON plugin.group_app_registry_state
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE plugin.group_app_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE plugin.group_app_binding FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_app_binding_tenant_select ON plugin.group_app_binding;
CREATE POLICY group_app_binding_tenant_select ON plugin.group_app_binding
    FOR SELECT USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE plugin.group_app_access_grant ENABLE ROW LEVEL SECURITY;
ALTER TABLE plugin.group_app_access_grant FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_app_access_grant_actor_select ON plugin.group_app_access_grant;
CREATE POLICY group_app_access_grant_actor_select ON plugin.group_app_access_grant
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

ALTER TABLE plugin.group_app_audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE plugin.group_app_audit_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_app_audit_event_actor_select ON plugin.group_app_audit_event;
CREATE POLICY group_app_audit_event_actor_select ON plugin.group_app_audit_event
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

COMMENT ON TABLE plugin.group_app_manifest IS
    'M: immutable tenant plugin manifest versions; verified status requires trusted ingestion not provided by this migration.';
COMMENT ON TABLE plugin.group_app_registry_state IS
    'M/SCD2: per-Worktree Group App Registry revision; lifecycle mutations close and append atomically.';
COMMENT ON TABLE plugin.group_app_binding IS
    'M: SCD2 Worktree Group App installation; active state does not itself grant capabilities.';
COMMENT ON TABLE plugin.group_app_access_grant IS
    'M: actor-scoped group_app:open authorization; plugin tool/data capabilities are checked separately.';
COMMENT ON TABLE plugin.group_app_audit_event IS
    'T: append-only Group App Registry lifecycle audit; contains no capability secrets.';

COMMIT;
