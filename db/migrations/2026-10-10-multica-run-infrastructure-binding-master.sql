-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-10-multica-run-infrastructure-binding-master.sql"})
-- MERGE (binding:Table {id:"multica.run_infrastructure_binding_revision",class:"Master"})
-- MERGE (event:Table {id:"multica.run_infrastructure_binding_event",class:"Transaction"})
-- MERGE (run:Table {id:"multica.engineering_run",class:"Master"})
-- MERGE (guard:Function {id:"multica.guard_run_infrastructure_binding_revision"})
-- MERGE (append_event:Function {id:"multica.append_run_infrastructure_binding_event"})
-- MERGE (reject_event:Function {id:"multica.reject_run_infrastructure_binding_event_mutation"})
-- MERGE (rls:Policy {id:"multica.run_infrastructure_binding_revision.tenant_scope"})
-- MERGE (event_rls:Policy {id:"multica.run_infrastructure_binding_event.tenant_scope"})
-- MERGE (owner:Role {id:"star_multica_run_owner"})
-- MERGE (self)-[:CREATES]->(binding)
-- MERGE (self)-[:CREATES]->(event)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:DEFINES]->(append_event)
-- MERGE (self)-[:DEFINES]->(reject_event)
-- MERGE (self)-[:CONFIGURES]->(rls)
-- MERGE (self)-[:CONFIGURES]->(event_rls)
-- MERGE (binding)-[:REFERENCES]->(run)
-- MERGE (guard)-[:USES]->(binding)
-- MERGE (append_event)-[:WRITES]->(event)
-- MERGE (reject_event)-[:USES]->(event)
-- MERGE (owner)-[:READS]->(binding)
-- MERGE (owner)-[:WRITES]->(binding)
-- MERGE (owner)-[:READS]->(event)
-- @endcypher

-- The Run/Execution owner owns this SCD2 binding snapshot. Infrastructure-owned
-- Provider Profile, HostEnvironment, and K3s catalog rows remain in their own store.
-- W/T/M: binding revision is Master; binding lifecycle event is append-only Transaction.
BEGIN;

DO $precondition$
BEGIN
    IF to_regclass('multica.engineering_run') IS NULL THEN
        RAISE EXCEPTION 'Engineering Run directory migration must be applied first';
    END IF;
END
$precondition$;

CREATE UNIQUE INDEX IF NOT EXISTS uq_engineering_run_infrastructure_binding_scope
    ON multica.engineering_run (tenant_id, project_id, branch_id, engineering_run_id);

CREATE TABLE IF NOT EXISTS multica.run_infrastructure_binding_revision (
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    binding_id UUID NOT NULL CHECK (binding_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    binding_revision BIGINT NOT NULL CHECK (binding_revision > 0),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 2),
    binding_snapshot JSONB NOT NULL
        CHECK (jsonb_typeof(binding_snapshot) = 'object')
        CHECK (octet_length(binding_snapshot::text) <= 16384),
    binding_digest BYTEA NOT NULL CHECK (octet_length(binding_digest) = 32),
    changed_by UUID NOT NULL CHECK (changed_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    PRIMARY KEY (tenant_id, binding_id, binding_revision),
    FOREIGN KEY (tenant_id, project_id, branch_id, engineering_run_id)
        REFERENCES multica.engineering_run (tenant_id, project_id, branch_id, engineering_run_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (branch_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to >= valid_from),
    CHECK (binding_snapshot ->> 'schema_version' IS NOT DISTINCT FROM schema_version::TEXT),
    CHECK (binding_snapshot ->> 'tenant_id' IS NOT DISTINCT FROM tenant_id::TEXT),
    CHECK (binding_snapshot ->> 'project_id' IS NOT DISTINCT FROM project_id::TEXT),
    CHECK (binding_snapshot ->> 'branch_id' IS NOT DISTINCT FROM branch_id::TEXT),
    CHECK (binding_snapshot ->> 'engineering_run_id' IS NOT DISTINCT FROM engineering_run_id::TEXT),
    CHECK (binding_snapshot ->> 'binding_id' IS NOT DISTINCT FROM binding_id::TEXT),
    CHECK (binding_snapshot ->> 'binding_revision' IS NOT DISTINCT FROM binding_revision::TEXT)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_run_infrastructure_binding_current
    ON multica.run_infrastructure_binding_revision
       (tenant_id, project_id, branch_id, engineering_run_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_run_infrastructure_binding_lookup
    ON multica.run_infrastructure_binding_revision
       (tenant_id, project_id, branch_id, engineering_run_id, binding_revision DESC);

CREATE TABLE IF NOT EXISTS multica.run_infrastructure_binding_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    binding_id UUID NOT NULL,
    binding_revision BIGINT NOT NULL CHECK (binding_revision > 0),
    event_type VARCHAR(16) NOT NULL CHECK (event_type IN ('created', 'superseded')),
    binding_digest BYTEA NOT NULL CHECK (octet_length(binding_digest) = 32),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    happened_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    FOREIGN KEY (tenant_id, binding_id, binding_revision)
        REFERENCES multica.run_infrastructure_binding_revision (tenant_id, binding_id, binding_revision)
        ON DELETE RESTRICT,
    UNIQUE (tenant_id, binding_id, binding_revision, event_type),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (branch_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_run_infrastructure_binding_event_lookup
    ON multica.run_infrastructure_binding_event
       (tenant_id, project_id, branch_id, engineering_run_id, happened_at, event_id);

CREATE OR REPLACE FUNCTION multica.guard_run_infrastructure_binding_revision()
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
            RAISE EXCEPTION 'Run infrastructure binding must be created as a current actor-scoped Master row';
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
            RAISE EXCEPTION 'Run infrastructure binding Master rows may only be superseded once by the current tenant actor';
        END IF;
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'Run infrastructure binding Master rows cannot be physically deleted';
END;
$$;

CREATE OR REPLACE FUNCTION multica.append_run_infrastructure_binding_event()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
BEGIN
    INSERT INTO multica.run_infrastructure_binding_event (
        tenant_id, project_id, branch_id, engineering_run_id,
        binding_id, binding_revision, event_type, binding_digest, actor_id
    ) VALUES (
        NEW.tenant_id, NEW.project_id, NEW.branch_id, NEW.engineering_run_id,
        NEW.binding_id, NEW.binding_revision,
        CASE WHEN TG_OP = 'INSERT' THEN 'created' ELSE 'superseded' END,
        NEW.binding_digest,
        NULLIF(pg_catalog.current_setting('app.actor_id', true), '')::UUID
    );
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.reject_run_infrastructure_binding_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    RAISE EXCEPTION 'Run infrastructure binding events are append-only';
END;
$$;

DROP TRIGGER IF EXISTS run_infrastructure_binding_revision_guard
    ON multica.run_infrastructure_binding_revision;
CREATE TRIGGER run_infrastructure_binding_revision_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.run_infrastructure_binding_revision
    FOR EACH ROW EXECUTE FUNCTION multica.guard_run_infrastructure_binding_revision();
DROP TRIGGER IF EXISTS run_infrastructure_binding_event_insert
    ON multica.run_infrastructure_binding_revision;
CREATE TRIGGER run_infrastructure_binding_event_insert
    AFTER INSERT ON multica.run_infrastructure_binding_revision
    FOR EACH ROW EXECUTE FUNCTION multica.append_run_infrastructure_binding_event();
DROP TRIGGER IF EXISTS run_infrastructure_binding_event_supersede
    ON multica.run_infrastructure_binding_revision;
CREATE TRIGGER run_infrastructure_binding_event_supersede
    AFTER UPDATE OF valid_to ON multica.run_infrastructure_binding_revision
    FOR EACH ROW EXECUTE FUNCTION multica.append_run_infrastructure_binding_event();
DROP TRIGGER IF EXISTS run_infrastructure_binding_event_no_update_delete
    ON multica.run_infrastructure_binding_event;
CREATE TRIGGER run_infrastructure_binding_event_no_update_delete
    BEFORE UPDATE OR DELETE ON multica.run_infrastructure_binding_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_run_infrastructure_binding_event_mutation();
DROP TRIGGER IF EXISTS run_infrastructure_binding_event_no_truncate
    ON multica.run_infrastructure_binding_event;
CREATE TRIGGER run_infrastructure_binding_event_no_truncate
    BEFORE TRUNCATE ON multica.run_infrastructure_binding_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_run_infrastructure_binding_event_mutation();

ALTER TABLE multica.run_infrastructure_binding_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.run_infrastructure_binding_revision FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS run_infrastructure_binding_revision_tenant_scope
    ON multica.run_infrastructure_binding_revision;
CREATE POLICY run_infrastructure_binding_revision_tenant_scope
    ON multica.run_infrastructure_binding_revision
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.run_infrastructure_binding_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.run_infrastructure_binding_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS run_infrastructure_binding_event_tenant_scope
    ON multica.run_infrastructure_binding_event;
CREATE POLICY run_infrastructure_binding_event_tenant_scope
    ON multica.run_infrastructure_binding_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DO $role$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_multica_run_owner') THEN
        EXECUTE 'ALTER ROLE star_multica_run_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_multica_run_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
END
$role$;

REVOKE ALL ON SCHEMA multica FROM star_multica_run_owner;
REVOKE ALL ON TABLE multica.run_infrastructure_binding_revision,
    multica.run_infrastructure_binding_event,
    multica.engineering_run
FROM star_multica_run_owner;
REVOKE ALL ON TABLE multica.run_infrastructure_binding_revision,
    multica.run_infrastructure_binding_event FROM PUBLIC;

DO $other_owners$
DECLARE
    role_name TEXT;
BEGIN
    FOREACH role_name IN ARRAY ARRAY['star_infrastructure_owner', 'star_automation_owner'] LOOP
        IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = role_name) THEN
            EXECUTE format(
                'REVOKE ALL PRIVILEGES ON TABLE multica.run_infrastructure_binding_revision, '
                'multica.run_infrastructure_binding_event FROM %I',
                role_name
            );
        END IF;
    END LOOP;
END
$other_owners$;

GRANT USAGE ON SCHEMA multica TO star_multica_run_owner;
GRANT SELECT, INSERT ON TABLE
    multica.run_infrastructure_binding_revision
TO star_multica_run_owner;
GRANT SELECT ON TABLE multica.run_infrastructure_binding_event
TO star_multica_run_owner;
GRANT UPDATE (valid_to) ON TABLE multica.run_infrastructure_binding_revision
TO star_multica_run_owner;
GRANT SELECT (tenant_id, project_id, branch_id, engineering_run_id)
    ON TABLE multica.engineering_run TO star_multica_run_owner;

REVOKE ALL ON FUNCTION multica.guard_run_infrastructure_binding_revision() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.append_run_infrastructure_binding_event() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.reject_run_infrastructure_binding_event_mutation() FROM PUBLIC;

COMMENT ON TABLE multica.run_infrastructure_binding_revision IS
    'M: Run-owner SCD2 snapshot of the selected HostEnvironment, exact K3s cluster generation, provider/runtime profile revisions, namespace and resource budget; external Infrastructure facts are revalidated by their owner.';
COMMENT ON TABLE multica.run_infrastructure_binding_event IS
    'T: append-only audit facts for Run infrastructure binding creation and supersession.';
COMMENT ON COLUMN multica.run_infrastructure_binding_revision.binding_digest IS
    'Canonical Rust RunInfrastructureBindingSnapshotV2 digest; readers recompute and compare it before returning a stored binding.';

COMMIT;
