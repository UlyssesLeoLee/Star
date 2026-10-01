-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-10-01-multica-agent-execution-catalog.sql",type:"file",language:"sql"}),
--   (m:Module {name:"multica_agent_execution_catalog_schema",type:"module",language:"sql"}),
--   (revision:Class {name:"multica.agent_execution_catalog_revision",type:"class",language:"sql"}),
--   (provider:Class {name:"multica.agent_execution_provider_catalog",type:"class",language:"sql"}),
--   (provider_capability:Class {name:"multica.agent_execution_provider_capability",type:"class",language:"sql"}),
--   (skill:Class {name:"multica.agent_execution_skill_catalog",type:"class",language:"sql"}),
--   (skill_capability:Class {name:"multica.agent_execution_skill_capability",type:"class",language:"sql"}),
--   (grant_set:Class {name:"multica.agent_execution_grant_set",type:"class",language:"sql"}),
--   (grant_capability:Class {name:"multica.agent_execution_grant_capability",type:"class",language:"sql"}),
--   (audit:Class {name:"multica.agent_execution_catalog_audit_event",type:"class",language:"sql"}),
--   (provider_guard:Function {name:"multica.guard_agent_execution_provider_scd2",type:"function",language:"sql"}),
--   (skill_guard:Function {name:"multica.guard_agent_execution_skill_scd2",type:"function",language:"sql"}),
--   (grant_guard:Function {name:"multica.guard_agent_execution_grant_set_scd2",type:"function",language:"sql"}),
--   (revision_bump:Function {name:"multica.bump_agent_execution_catalog_revision",type:"function",language:"sql"}),
--   (audit_append:Function {name:"multica.audit_agent_execution_catalog_change",type:"function",language:"sql"}),
--   (child_guard:Function {name:"multica.guard_agent_execution_catalog_child_insert",type:"function",language:"sql"}),
--   (capability_count_guard:Function {name:"multica.verify_agent_execution_catalog_capability_count",type:"function",language:"sql"}),
--   (child_immutable:Function {name:"multica.reject_agent_execution_catalog_child_mutation",type:"function",language:"sql"}),
--   (audit_immutable:Function {name:"multica.reject_agent_execution_catalog_audit_mutation",type:"function",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(revision),(m)-[:CONTAINS]->(provider),
--   (m)-[:CONTAINS]->(provider_capability),(m)-[:CONTAINS]->(skill),
--   (m)-[:CONTAINS]->(skill_capability),(m)-[:CONTAINS]->(grant_set),
--   (m)-[:CONTAINS]->(grant_capability),(m)-[:CONTAINS]->(audit),
--   (m)-[:CONTAINS]->(provider_guard),(m)-[:CONTAINS]->(skill_guard),
--   (m)-[:CONTAINS]->(grant_guard),(m)-[:CONTAINS]->(revision_bump),
--   (m)-[:CONTAINS]->(audit_append),(m)-[:CONTAINS]->(child_guard),
--   (m)-[:CONTAINS]->(capability_count_guard),
--   (m)-[:CONTAINS]->(child_immutable),(m)-[:CONTAINS]->(audit_immutable),
--   (provider_guard)-[:USES]->(provider),(skill_guard)-[:USES]->(skill),
--   (grant_guard)-[:USES]->(grant_set),(revision_bump)-[:USES]->(revision),
--   (audit_append)-[:USES]->(audit),(child_guard)-[:USES]->(provider_capability),
--   (child_guard)-[:USES]->(skill_capability),(child_guard)-[:USES]->(grant_capability),
--   (capability_count_guard)-[:USES]->(provider_capability),
--   (capability_count_guard)-[:USES]->(skill_capability),(capability_count_guard)-[:USES]->(grant_capability),
--   (child_immutable)-[:USES]->(provider_capability),(child_immutable)-[:USES]->(skill_capability),
--   (child_immutable)-[:USES]->(grant_capability),(audit_immutable)-[:USES]->(audit);

-- Phase 9E-4C2 current catalogs. W/T/M: catalog entries, capability bindings, and the current
-- GrantSet are M/SCD2; catalog audit is T/append-only; the revision row is an M concurrency
-- projection. Only public, non-secret implementation/configuration digests are stored here.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.agent_execution_catalog_revision (
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    provider_catalog_revision BIGINT NOT NULL DEFAULT 0 CHECK (provider_catalog_revision >= 0),
    skill_catalog_revision BIGINT NOT NULL DEFAULT 0 CHECK (skill_catalog_revision >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, project_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);

CREATE TABLE IF NOT EXISTS multica.agent_execution_provider_catalog (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    provider_id VARCHAR(128) NOT NULL
        CHECK (provider_id ~ '^[a-z0-9][a-z0-9._:/-]{0,127}$'),
    provider_version BIGINT NOT NULL CHECK (provider_version > 0),
    implementation_digest CHAR(64) NOT NULL
        CHECK (implementation_digest::text ~ '^[0-9a-f]{64}$'),
    configuration_digest CHAR(64) NOT NULL
        CHECK (configuration_digest::text ~ '^[0-9a-f]{64}$'),
    capability_count SMALLINT NOT NULL CHECK (capability_count BETWEEN 0 AND 64),
    available BOOLEAN NOT NULL DEFAULT TRUE,
    published_by UUID NOT NULL,
    created_xid XID8 NOT NULL DEFAULT pg_current_xact_id(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    UNIQUE (tenant_id, entry_id),
    UNIQUE (tenant_id, project_id, provider_id, provider_version),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (published_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_agent_execution_provider_current
    ON multica.agent_execution_provider_catalog (tenant_id, project_id, provider_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_agent_execution_provider_lookup
    ON multica.agent_execution_provider_catalog
        (tenant_id, project_id, provider_id, provider_version DESC)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.agent_execution_provider_capability (
    tenant_id UUID NOT NULL,
    entry_id UUID NOT NULL,
    capability_code VARCHAR(128) NOT NULL
        CHECK (capability_code ~ '^[a-z0-9][a-z0-9._:/-]{0,127}$'),
    PRIMARY KEY (tenant_id, entry_id, capability_code),
    FOREIGN KEY (tenant_id, entry_id)
        REFERENCES multica.agent_execution_provider_catalog (tenant_id, entry_id)
        ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_agent_execution_provider_capability_entry
    ON multica.agent_execution_provider_capability (tenant_id, entry_id, capability_code);

CREATE TABLE IF NOT EXISTS multica.agent_execution_skill_catalog (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    skill_id VARCHAR(128) NOT NULL
        CHECK (skill_id ~ '^[a-z0-9][a-z0-9._:/-]{0,127}$'),
    skill_version BIGINT NOT NULL CHECK (skill_version > 0),
    content_digest CHAR(64) NOT NULL CHECK (content_digest::text ~ '^[0-9a-f]{64}$'),
    capability_count SMALLINT NOT NULL CHECK (capability_count BETWEEN 0 AND 64),
    available BOOLEAN NOT NULL DEFAULT TRUE,
    published_by UUID NOT NULL,
    created_xid XID8 NOT NULL DEFAULT pg_current_xact_id(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    UNIQUE (tenant_id, entry_id),
    UNIQUE (tenant_id, project_id, skill_id, skill_version),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (published_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_agent_execution_skill_current
    ON multica.agent_execution_skill_catalog (tenant_id, project_id, skill_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_agent_execution_skill_lookup
    ON multica.agent_execution_skill_catalog (tenant_id, project_id, skill_id, skill_version DESC)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.agent_execution_skill_capability (
    tenant_id UUID NOT NULL,
    entry_id UUID NOT NULL,
    capability_code VARCHAR(128) NOT NULL
        CHECK (capability_code ~ '^[a-z0-9][a-z0-9._:/-]{0,127}$'),
    PRIMARY KEY (tenant_id, entry_id, capability_code),
    FOREIGN KEY (tenant_id, entry_id)
        REFERENCES multica.agent_execution_skill_catalog (tenant_id, entry_id)
        ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_agent_execution_skill_capability_entry
    ON multica.agent_execution_skill_capability (tenant_id, entry_id, capability_code);

CREATE TABLE IF NOT EXISTS multica.agent_execution_grant_set (
    entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    grant_set_id UUID NOT NULL,
    grant_set_version BIGINT NOT NULL CHECK (grant_set_version > 0),
    capability_count SMALLINT NOT NULL CHECK (capability_count BETWEEN 0 AND 64),
    expires_at_epoch_ms BIGINT NOT NULL CHECK (expires_at_epoch_ms > 0),
    published_by UUID NOT NULL,
    created_xid XID8 NOT NULL DEFAULT pg_current_xact_id(),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    UNIQUE (tenant_id, entry_id),
    UNIQUE (tenant_id, project_id, grant_set_id, grant_set_version),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (grant_set_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (published_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_agent_execution_grant_set_current
    ON multica.agent_execution_grant_set (tenant_id, project_id)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.agent_execution_grant_capability (
    tenant_id UUID NOT NULL,
    entry_id UUID NOT NULL,
    capability_code VARCHAR(128) NOT NULL
        CHECK (capability_code ~ '^[a-z0-9][a-z0-9._:/-]{0,127}$'),
    PRIMARY KEY (tenant_id, entry_id, capability_code),
    FOREIGN KEY (tenant_id, entry_id)
        REFERENCES multica.agent_execution_grant_set (tenant_id, entry_id)
        ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_agent_execution_grant_capability_entry
    ON multica.agent_execution_grant_capability (tenant_id, entry_id, capability_code);

CREATE TABLE IF NOT EXISTS multica.agent_execution_catalog_audit_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    object_kind VARCHAR(16) NOT NULL
        CHECK (object_kind IN ('provider','skill','grant_set')),
    object_id UUID NOT NULL,
    object_version BIGINT NOT NULL CHECK (object_version > 0),
    event_type VARCHAR(24) NOT NULL
        CHECK (event_type IN ('published','superseded')),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (tenant_id, object_kind, object_id, object_version, event_type),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_agent_execution_catalog_audit_scope
    ON multica.agent_execution_catalog_audit_event
        (tenant_id, project_id, occurred_at DESC, event_id);

CREATE OR REPLACE FUNCTION multica.bump_agent_execution_catalog_revision()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO multica.agent_execution_catalog_revision
        (tenant_id, project_id, provider_catalog_revision, skill_catalog_revision)
    VALUES (
        NEW.tenant_id,
        NEW.project_id,
        CASE WHEN TG_ARGV[0] = 'provider' THEN 1 ELSE 0 END,
        CASE WHEN TG_ARGV[0] = 'skill' THEN 1 ELSE 0 END
    )
    ON CONFLICT (tenant_id, project_id) DO UPDATE
       SET provider_catalog_revision = multica.agent_execution_catalog_revision.provider_catalog_revision
            + CASE WHEN TG_ARGV[0] = 'provider' THEN 1 ELSE 0 END,
           skill_catalog_revision = multica.agent_execution_catalog_revision.skill_catalog_revision
            + CASE WHEN TG_ARGV[0] = 'skill' THEN 1 ELSE 0 END,
           updated_at = clock_timestamp();
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.guard_agent_execution_provider_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    previous_version BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Agent Execution Provider catalog rows cannot be deleted';
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL OR NEW.valid_to <= OLD.valid_from
           OR (to_jsonb(OLD) - 'valid_to') IS DISTINCT FROM (to_jsonb(NEW) - 'valid_to') THEN
            RAISE EXCEPTION 'Provider catalog rows may only be closed once without rewriting history';
        END IF;
        RETURN NEW;
    END IF;
    IF NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'Provider catalog revisions must be inserted as current rows';
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended(
        NEW.tenant_id::text || ':' || NEW.project_id::text || ':provider:' || NEW.provider_id, 0));
    SELECT COALESCE(MAX(provider_version), 0) INTO previous_version
      FROM multica.agent_execution_provider_catalog
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
       AND provider_id = NEW.provider_id;
    IF NEW.provider_version <> previous_version + 1 THEN
        RAISE EXCEPTION 'Provider catalog version must increase by one';
    END IF;
    IF EXISTS (SELECT 1 FROM multica.agent_execution_provider_catalog
        WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
          AND provider_id = NEW.provider_id AND valid_to IS NULL) THEN
        RAISE EXCEPTION 'Close the current Provider row before inserting a successor';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.guard_agent_execution_skill_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    previous_version BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Agent Execution Skill catalog rows cannot be deleted';
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL OR NEW.valid_to <= OLD.valid_from
           OR (to_jsonb(OLD) - 'valid_to') IS DISTINCT FROM (to_jsonb(NEW) - 'valid_to') THEN
            RAISE EXCEPTION 'Skill catalog rows may only be closed once without rewriting history';
        END IF;
        RETURN NEW;
    END IF;
    IF NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'Skill catalog revisions must be inserted as current rows';
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended(
        NEW.tenant_id::text || ':' || NEW.project_id::text || ':skill:' || NEW.skill_id, 0));
    SELECT COALESCE(MAX(skill_version), 0) INTO previous_version
      FROM multica.agent_execution_skill_catalog
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
       AND skill_id = NEW.skill_id;
    IF NEW.skill_version <> previous_version + 1 THEN
        RAISE EXCEPTION 'Skill catalog version must increase by one';
    END IF;
    IF EXISTS (SELECT 1 FROM multica.agent_execution_skill_catalog
        WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
          AND skill_id = NEW.skill_id AND valid_to IS NULL) THEN
        RAISE EXCEPTION 'Close the current Skill row before inserting a successor';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.guard_agent_execution_grant_set_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    previous_version BIGINT;
    previous_grant_set_id UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Agent Execution GrantSet rows cannot be deleted';
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL OR NEW.valid_to IS NULL OR NEW.valid_to <= OLD.valid_from
           OR (to_jsonb(OLD) - 'valid_to') IS DISTINCT FROM (to_jsonb(NEW) - 'valid_to') THEN
            RAISE EXCEPTION 'GrantSet rows may only be closed once without rewriting history';
        END IF;
        RETURN NEW;
    END IF;
    IF NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'GrantSet revisions must be inserted as current rows';
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended(
        NEW.tenant_id::text || ':' || NEW.project_id::text || ':grant-set', 0));
    SELECT grant_set_id, grant_set_version INTO previous_grant_set_id, previous_version
      FROM multica.agent_execution_grant_set
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
     ORDER BY grant_set_version DESC LIMIT 1;
    IF previous_version IS NULL THEN
        IF NEW.grant_set_version <> 1 THEN
            RAISE EXCEPTION 'First GrantSet version must be one';
        END IF;
    ELSIF NEW.grant_set_id IS DISTINCT FROM previous_grant_set_id
          OR NEW.grant_set_version <> previous_version + 1 THEN
        RAISE EXCEPTION 'GrantSet identity is immutable and version must increase by one';
    END IF;
    IF EXISTS (SELECT 1 FROM multica.agent_execution_grant_set
        WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id AND valid_to IS NULL) THEN
        RAISE EXCEPTION 'Close the current GrantSet row before inserting a successor';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.guard_agent_execution_catalog_child_insert()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    parent_created_xid XID8;
    parent_valid_to TIMESTAMPTZ;
BEGIN
    EXECUTE format('SELECT created_xid, valid_to FROM multica.%I WHERE tenant_id = $1 AND entry_id = $2',
        CASE TG_TABLE_NAME
            WHEN 'agent_execution_provider_capability' THEN 'agent_execution_provider_catalog'
            WHEN 'agent_execution_skill_capability' THEN 'agent_execution_skill_catalog'
            ELSE 'agent_execution_grant_set'
        END)
       INTO parent_created_xid, parent_valid_to
       USING NEW.tenant_id, NEW.entry_id;
    IF parent_created_xid IS DISTINCT FROM pg_current_xact_id() OR parent_valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'Catalog capability rows must be inserted with a new current parent in the same transaction';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.verify_agent_execution_catalog_capability_count()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    expected_count SMALLINT;
    actual_count BIGINT;
BEGIN
    IF TG_TABLE_NAME = 'agent_execution_provider_catalog' THEN
        SELECT capability_count INTO expected_count
          FROM multica.agent_execution_provider_catalog
         WHERE tenant_id = NEW.tenant_id AND entry_id = NEW.entry_id;
        SELECT COUNT(*) INTO actual_count
          FROM multica.agent_execution_provider_capability
         WHERE tenant_id = NEW.tenant_id AND entry_id = NEW.entry_id;
    ELSIF TG_TABLE_NAME = 'agent_execution_skill_catalog' THEN
        SELECT capability_count INTO expected_count
          FROM multica.agent_execution_skill_catalog
         WHERE tenant_id = NEW.tenant_id AND entry_id = NEW.entry_id;
        SELECT COUNT(*) INTO actual_count
          FROM multica.agent_execution_skill_capability
         WHERE tenant_id = NEW.tenant_id AND entry_id = NEW.entry_id;
    ELSE
        SELECT capability_count INTO expected_count
          FROM multica.agent_execution_grant_set
         WHERE tenant_id = NEW.tenant_id AND entry_id = NEW.entry_id;
        SELECT COUNT(*) INTO actual_count
          FROM multica.agent_execution_grant_capability
         WHERE tenant_id = NEW.tenant_id AND entry_id = NEW.entry_id;
    END IF;
    IF expected_count IS NULL OR actual_count <> expected_count THEN
        RAISE EXCEPTION 'Catalog capability count mismatch for % entry %: expected %, found %',
            TG_TABLE_NAME, NEW.entry_id, expected_count, actual_count;
    END IF;
    RETURN NULL;
END;
$$;

CREATE OR REPLACE FUNCTION multica.reject_agent_execution_catalog_child_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Published Agent Execution catalog capability rows are immutable';
END;
$$;

CREATE OR REPLACE FUNCTION multica.audit_agent_execution_catalog_change()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    actor_text TEXT;
    correlation_text TEXT;
    event_kind VARCHAR(24);
    row_id UUID;
    row_version BIGINT;
    details_value JSONB;
    source_row JSONB;
    tenant_scope UUID;
    project_scope UUID;
BEGIN
    actor_text := NULLIF(current_setting('app.actor_id', true), '');
    correlation_text := NULLIF(current_setting('app.correlation_id', true), '');
    IF actor_text IS NULL OR correlation_text IS NULL THEN
        RAISE EXCEPTION 'Catalog mutation requires app.actor_id and app.correlation_id';
    END IF;
    IF TG_OP = 'DELETE' THEN
        source_row := to_jsonb(OLD);
    ELSE
        source_row := to_jsonb(NEW);
    END IF;
    tenant_scope := (source_row ->> 'tenant_id')::UUID;
    project_scope := (source_row ->> 'project_id')::UUID;
    IF TG_TABLE_NAME = 'agent_execution_provider_catalog' THEN
        event_kind := 'provider';
        row_id := (source_row ->> 'entry_id')::UUID;
        row_version := (source_row ->> 'provider_version')::BIGINT;
        details_value := jsonb_build_object(
            'provider_id', source_row -> 'provider_id',
            'implementation_digest', source_row -> 'implementation_digest',
            'configuration_digest', source_row -> 'configuration_digest',
            'available', source_row -> 'available');
    ELSIF TG_TABLE_NAME = 'agent_execution_skill_catalog' THEN
        event_kind := 'skill';
        row_id := (source_row ->> 'entry_id')::UUID;
        row_version := (source_row ->> 'skill_version')::BIGINT;
        details_value := jsonb_build_object(
            'skill_id', source_row -> 'skill_id',
            'content_digest', source_row -> 'content_digest',
            'available', source_row -> 'available');
    ELSE
        event_kind := 'grant_set';
        row_id := (source_row ->> 'entry_id')::UUID;
        row_version := (source_row ->> 'grant_set_version')::BIGINT;
        details_value := jsonb_build_object(
            'grant_set_id', source_row -> 'grant_set_id',
            'expires_at_epoch_ms', source_row -> 'expires_at_epoch_ms',
            'capability_count', source_row -> 'capability_count');
    END IF;
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Agent Execution catalog rows cannot be deleted';
    END IF;
    INSERT INTO multica.agent_execution_catalog_audit_event
        (tenant_id, project_id, object_kind, object_id, object_version, event_type,
         actor_id, correlation_id, details)
    VALUES (
        tenant_scope,
        project_scope,
        event_kind,
        row_id,
        row_version,
        CASE WHEN TG_OP = 'INSERT' THEN 'published' ELSE 'superseded' END,
        actor_text::UUID,
        correlation_text::UUID,
        details_value);
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.reject_agent_execution_catalog_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Agent Execution catalog audit facts are append-only';
END;
$$;

DROP TRIGGER IF EXISTS agent_execution_provider_scd2_guard
    ON multica.agent_execution_provider_catalog;
CREATE TRIGGER agent_execution_provider_scd2_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.agent_execution_provider_catalog
    FOR EACH ROW EXECUTE FUNCTION multica.guard_agent_execution_provider_scd2();
DROP TRIGGER IF EXISTS agent_execution_provider_revision_bump
    ON multica.agent_execution_provider_catalog;
CREATE TRIGGER agent_execution_provider_revision_bump
    AFTER INSERT OR UPDATE OF valid_to ON multica.agent_execution_provider_catalog
    FOR EACH ROW EXECUTE FUNCTION multica.bump_agent_execution_catalog_revision('provider');
DROP TRIGGER IF EXISTS agent_execution_provider_audit
    ON multica.agent_execution_provider_catalog;
CREATE TRIGGER agent_execution_provider_audit
    AFTER INSERT OR UPDATE OF valid_to ON multica.agent_execution_provider_catalog
    FOR EACH ROW EXECUTE FUNCTION multica.audit_agent_execution_catalog_change();
DROP TRIGGER IF EXISTS agent_execution_provider_capability_count
    ON multica.agent_execution_provider_catalog;
CREATE CONSTRAINT TRIGGER agent_execution_provider_capability_count
    AFTER INSERT ON multica.agent_execution_provider_catalog
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION multica.verify_agent_execution_catalog_capability_count();

DROP TRIGGER IF EXISTS agent_execution_skill_scd2_guard
    ON multica.agent_execution_skill_catalog;
CREATE TRIGGER agent_execution_skill_scd2_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.agent_execution_skill_catalog
    FOR EACH ROW EXECUTE FUNCTION multica.guard_agent_execution_skill_scd2();
DROP TRIGGER IF EXISTS agent_execution_skill_revision_bump
    ON multica.agent_execution_skill_catalog;
CREATE TRIGGER agent_execution_skill_revision_bump
    AFTER INSERT OR UPDATE OF valid_to ON multica.agent_execution_skill_catalog
    FOR EACH ROW EXECUTE FUNCTION multica.bump_agent_execution_catalog_revision('skill');
DROP TRIGGER IF EXISTS agent_execution_skill_audit
    ON multica.agent_execution_skill_catalog;
CREATE TRIGGER agent_execution_skill_audit
    AFTER INSERT OR UPDATE OF valid_to ON multica.agent_execution_skill_catalog
    FOR EACH ROW EXECUTE FUNCTION multica.audit_agent_execution_catalog_change();
DROP TRIGGER IF EXISTS agent_execution_skill_capability_count
    ON multica.agent_execution_skill_catalog;
CREATE CONSTRAINT TRIGGER agent_execution_skill_capability_count
    AFTER INSERT ON multica.agent_execution_skill_catalog
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION multica.verify_agent_execution_catalog_capability_count();

DROP TRIGGER IF EXISTS agent_execution_grant_set_scd2_guard
    ON multica.agent_execution_grant_set;
CREATE TRIGGER agent_execution_grant_set_scd2_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.agent_execution_grant_set
    FOR EACH ROW EXECUTE FUNCTION multica.guard_agent_execution_grant_set_scd2();
DROP TRIGGER IF EXISTS agent_execution_grant_set_audit
    ON multica.agent_execution_grant_set;
CREATE TRIGGER agent_execution_grant_set_audit
    AFTER INSERT OR UPDATE OF valid_to ON multica.agent_execution_grant_set
    FOR EACH ROW EXECUTE FUNCTION multica.audit_agent_execution_catalog_change();
DROP TRIGGER IF EXISTS agent_execution_grant_capability_count
    ON multica.agent_execution_grant_set;
CREATE CONSTRAINT TRIGGER agent_execution_grant_capability_count
    AFTER INSERT ON multica.agent_execution_grant_set
    DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
    EXECUTE FUNCTION multica.verify_agent_execution_catalog_capability_count();

DO $$
DECLARE
    capability_table TEXT;
BEGIN
    FOREACH capability_table IN ARRAY ARRAY[
        'agent_execution_provider_capability',
        'agent_execution_skill_capability',
        'agent_execution_grant_capability'
    ] LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON multica.%I', capability_table || '_same_tx', capability_table);
        EXECUTE format('CREATE TRIGGER %I BEFORE INSERT ON multica.%I FOR EACH ROW EXECUTE FUNCTION multica.guard_agent_execution_catalog_child_insert()', capability_table || '_same_tx', capability_table);
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON multica.%I', capability_table || '_immutable', capability_table);
        EXECUTE format('CREATE TRIGGER %I BEFORE UPDATE OR DELETE ON multica.%I FOR EACH ROW EXECUTE FUNCTION multica.reject_agent_execution_catalog_child_mutation()', capability_table || '_immutable', capability_table);
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON multica.%I', capability_table || '_no_truncate', capability_table);
        EXECUTE format('CREATE TRIGGER %I BEFORE TRUNCATE ON multica.%I FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_agent_execution_catalog_child_mutation()', capability_table || '_no_truncate', capability_table);
    END LOOP;
END;
$$;

DROP TRIGGER IF EXISTS agent_execution_catalog_audit_immutable
    ON multica.agent_execution_catalog_audit_event;
CREATE TRIGGER agent_execution_catalog_audit_immutable
    BEFORE UPDATE OR DELETE ON multica.agent_execution_catalog_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_agent_execution_catalog_audit_mutation();
DROP TRIGGER IF EXISTS agent_execution_catalog_audit_no_truncate
    ON multica.agent_execution_catalog_audit_event;
CREATE TRIGGER agent_execution_catalog_audit_no_truncate
    BEFORE TRUNCATE ON multica.agent_execution_catalog_audit_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_agent_execution_catalog_audit_mutation();

DO $$
DECLARE
    table_name TEXT;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'agent_execution_catalog_revision',
        'agent_execution_provider_catalog',
        'agent_execution_provider_capability',
        'agent_execution_skill_catalog',
        'agent_execution_skill_capability',
        'agent_execution_grant_set',
        'agent_execution_grant_capability',
        'agent_execution_catalog_audit_event'
    ] LOOP
        EXECUTE format('ALTER TABLE multica.%I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE multica.%I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format('DROP POLICY IF EXISTS %I ON multica.%I', table_name || '_tenant_scope', table_name);
        EXECUTE format(
            'CREATE POLICY %I ON multica.%I USING (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID)',
            table_name || '_tenant_scope', table_name);
    END LOOP;
END;
$$;

COMMENT ON TABLE multica.agent_execution_catalog_revision IS
    'M projection: monotonic Project-scoped current Provider/Skill revisions; updates share each catalog mutation transaction.';
COMMENT ON TABLE multica.agent_execution_provider_catalog IS
    'M SCD2 catalog of Project-visible non-secret Provider implementation/configuration digests.';
COMMENT ON TABLE multica.agent_execution_provider_capability IS
    'M weak capability association for one immutable Provider catalog revision.';
COMMENT ON TABLE multica.agent_execution_skill_catalog IS
    'M SCD2 Skill manifest metadata; Skill content remains outside PostgreSQL.';
COMMENT ON TABLE multica.agent_execution_skill_capability IS
    'M weak capability association for one immutable Skill catalog revision.';
COMMENT ON TABLE multica.agent_execution_grant_set IS
    'M SCD2 current Project GrantSet identity, version, expiry, and capability count.';
COMMENT ON TABLE multica.agent_execution_grant_capability IS
    'M weak capability association for one immutable GrantSet revision.';
COMMENT ON TABLE multica.agent_execution_catalog_audit_event IS
    'T append-only catalog change audit; stores public identifiers/digests only, never secrets or manifest bodies.';

COMMIT;
