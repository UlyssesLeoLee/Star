-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-09-infrastructure-provider-profile-master.sql"})
-- MERGE (profile:Table {id:"infrastructure.provider_profile_revision",class:"Master"})
-- MERGE (event:Table {id:"infrastructure.provider_profile_revision_event",class:"Transaction"})
-- MERGE (guard:Function {id:"infrastructure.guard_provider_profile_revision"})
-- MERGE (append_event:Function {id:"infrastructure.append_provider_profile_revision_event"})
-- MERGE (rls:Policy {id:"infrastructure.provider_profile_revision.tenant_isolation"})
-- MERGE (event_rls:Policy {id:"infrastructure.provider_profile_revision_event.tenant_isolation"})
-- MERGE (owner:Role {id:"star_infrastructure_owner"})
-- MERGE (self)-[:CREATES]->(profile)
-- MERGE (self)-[:CREATES]->(event)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:DEFINES]->(append_event)
-- MERGE (self)-[:CONFIGURES]->(rls)
-- MERGE (self)-[:CONFIGURES]->(event_rls)
-- MERGE (guard)-[:WRITES]->(event)
-- MERGE (append_event)-[:WRITES]->(event)
-- MERGE (owner)-[:READS]->(profile)
-- MERGE (owner)-[:READS]->(event)
-- @endcypher

BEGIN;

CREATE TABLE IF NOT EXISTS infrastructure.provider_profile_revision (
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL CHECK (profile_id <> '00000000-0000-0000-0000-000000000000'),
    revision BIGINT NOT NULL CHECK (revision > 0),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    provider_kind VARCHAR(32) NOT NULL
        CHECK (provider_kind IN ('multipass_k3s_cluster', 'remote_k3s')),
    placement_policy VARCHAR(24) NOT NULL
        CHECK (placement_policy IN ('shared_pool', 'run_dedicated', 'pinned_cluster')),
    host_cpu_millicores BIGINT NOT NULL CHECK (host_cpu_millicores > 0),
    host_memory_bytes BIGINT NOT NULL CHECK (host_memory_bytes > 0),
    host_ephemeral_storage_bytes BIGINT NOT NULL CHECK (host_ephemeral_storage_bytes >= 0),
    host_gpu_units INTEGER NOT NULL CHECK (host_gpu_units >= 0),
    cluster_cpu_millicores BIGINT NOT NULL CHECK (cluster_cpu_millicores > 0),
    cluster_memory_bytes BIGINT NOT NULL CHECK (cluster_memory_bytes > 0),
    cluster_ephemeral_storage_bytes BIGINT NOT NULL CHECK (cluster_ephemeral_storage_bytes >= 0),
    cluster_gpu_units INTEGER NOT NULL CHECK (cluster_gpu_units >= 0),
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    valid_to TIMESTAMPTZ,
    changed_by UUID NOT NULL DEFAULT NULLIF(current_setting('app.actor_id', true), '')::UUID
        CHECK (changed_by <> '00000000-0000-0000-0000-000000000000'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    PRIMARY KEY (tenant_id, profile_id, revision),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_provider_profile_revision_current
    ON infrastructure.provider_profile_revision (tenant_id, profile_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_provider_profile_revision_lookup
    ON infrastructure.provider_profile_revision (tenant_id, profile_id, revision, valid_from);

CREATE TABLE IF NOT EXISTS infrastructure.provider_profile_revision_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    event_type VARCHAR(16) NOT NULL CHECK (event_type IN ('created', 'superseded')),
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'),
    happened_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    FOREIGN KEY (tenant_id, profile_id, revision)
        REFERENCES infrastructure.provider_profile_revision (tenant_id, profile_id, revision)
        ON DELETE RESTRICT,
    UNIQUE (tenant_id, profile_id, revision, event_type)
);
CREATE INDEX IF NOT EXISTS idx_provider_profile_revision_event_lookup
    ON infrastructure.provider_profile_revision_event
       (tenant_id, profile_id, revision, happened_at, event_id);

CREATE OR REPLACE FUNCTION infrastructure.guard_provider_profile_revision()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.valid_to IS NOT NULL
           OR NEW.valid_from > pg_catalog.transaction_timestamp() + interval '1 second'
           OR NEW.created_at > pg_catalog.transaction_timestamp() + interval '1 second'
           OR NEW.changed_by IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::UUID
           OR NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::UUID IS DISTINCT FROM NEW.tenant_id THEN
            RAISE EXCEPTION 'provider profile revision must be created as a current actor-scoped Master row';
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL
           OR NEW.valid_to IS NULL
           OR (pg_catalog.to_jsonb(NEW) - 'valid_to') IS DISTINCT FROM (pg_catalog.to_jsonb(OLD) - 'valid_to')
           OR NEW.valid_to < pg_catalog.transaction_timestamp() - interval '1 second'
           OR NEW.valid_to > pg_catalog.transaction_timestamp() + interval '1 second'
           OR NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::UUID IS NULL
           OR NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::UUID IS DISTINCT FROM NEW.tenant_id THEN
            RAISE EXCEPTION 'provider profile Master rows may only be closed once by the current tenant actor';
        END IF;
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'provider profile Master rows cannot be physically deleted';
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.append_provider_profile_revision_event()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    INSERT INTO infrastructure.provider_profile_revision_event (
        tenant_id, profile_id, revision, event_type, profile_digest, actor_id
    ) VALUES (
        NEW.tenant_id, NEW.profile_id, NEW.revision,
        CASE WHEN TG_OP = 'INSERT' THEN 'created' ELSE 'superseded' END,
        NEW.profile_digest,
        NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::UUID
    );
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS provider_profile_revision_guard ON infrastructure.provider_profile_revision;
CREATE TRIGGER provider_profile_revision_guard
    BEFORE INSERT OR UPDATE OR DELETE ON infrastructure.provider_profile_revision
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_provider_profile_revision();
DROP TRIGGER IF EXISTS provider_profile_revision_event_insert ON infrastructure.provider_profile_revision;
CREATE TRIGGER provider_profile_revision_event_insert
    AFTER INSERT ON infrastructure.provider_profile_revision
    FOR EACH ROW EXECUTE FUNCTION infrastructure.append_provider_profile_revision_event();
DROP TRIGGER IF EXISTS provider_profile_revision_event_supersede ON infrastructure.provider_profile_revision;
CREATE TRIGGER provider_profile_revision_event_supersede
    AFTER UPDATE OF valid_to ON infrastructure.provider_profile_revision
    FOR EACH ROW EXECUTE FUNCTION infrastructure.append_provider_profile_revision_event();

CREATE OR REPLACE FUNCTION infrastructure.reject_provider_profile_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    RAISE EXCEPTION 'provider profile revision events are append-only';
END;
$$;

DROP TRIGGER IF EXISTS provider_profile_revision_event_no_update_delete ON infrastructure.provider_profile_revision_event;
CREATE TRIGGER provider_profile_revision_event_no_update_delete
    BEFORE UPDATE OR DELETE ON infrastructure.provider_profile_revision_event
    FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_provider_profile_event_mutation();
DROP TRIGGER IF EXISTS provider_profile_revision_event_no_truncate ON infrastructure.provider_profile_revision_event;
CREATE TRIGGER provider_profile_revision_event_no_truncate
    BEFORE TRUNCATE ON infrastructure.provider_profile_revision_event
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_provider_profile_event_mutation();

ALTER TABLE infrastructure.provider_profile_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.provider_profile_revision FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS provider_profile_revision_tenant_isolation ON infrastructure.provider_profile_revision;
CREATE POLICY provider_profile_revision_tenant_isolation
    ON infrastructure.provider_profile_revision
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.provider_profile_revision_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.provider_profile_revision_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS provider_profile_revision_event_tenant_isolation ON infrastructure.provider_profile_revision_event;
CREATE POLICY provider_profile_revision_event_tenant_isolation
    ON infrastructure.provider_profile_revision_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

REVOKE ALL ON TABLE infrastructure.provider_profile_revision,
    infrastructure.provider_profile_revision_event FROM PUBLIC;
GRANT SELECT ON TABLE infrastructure.provider_profile_revision,
    infrastructure.provider_profile_revision_event TO star_infrastructure_owner;

COMMENT ON TABLE infrastructure.provider_profile_revision IS
    'M: versioned per-tenant provider capacity profile; revisions are SCD2 and never physically deleted.';
COMMENT ON TABLE infrastructure.provider_profile_revision_event IS
    'T: append-only actor-scoped profile creation and supersession audit facts.';

COMMIT;
