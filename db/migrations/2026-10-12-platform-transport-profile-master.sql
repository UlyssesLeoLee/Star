-- @cypher schema=1 source_sha256=0fd92ab9b5201c0a2243f123185c33446d0d4e57c7ffcd1d6d216608240e5cb8
-- MERGE (self:File {path:"db/migrations/2026-10-12-platform-transport-profile-master.sql"})
-- MERGE (schema:Database {id:"platform_messaging"})
-- MERGE (owner:DatabaseRole {id:"star_platform_profile_owner"})
-- MERGE (resolver:DatabaseRole {id:"star_platform_profile_resolver"})
-- MERGE (profile:Table {id:"platform_messaging.transport_profile_revision",class:"Master"})
-- MERGE (audit:Table {id:"platform_messaging.transport_profile_revision_audit",class:"Transaction"})
-- MERGE (outbox:Table {id:"platform_messaging.transport_profile_revision_outbox",class:"Transaction"})
-- MERGE (guard:Function {id:"platform_messaging.guard_transport_profile_revision"})
-- MERGE (successor:Function {id:"platform_messaging.require_transport_profile_successor"})
-- MERGE (append:Function {id:"platform_messaging.append_transport_profile_revision_facts"})
-- MERGE (worm:Function {id:"platform_messaging.reject_transport_profile_transaction_mutation"})
-- MERGE (self)-[:CONFIGURES]->(schema)
-- MERGE (self)-[:CONFIGURES]->(owner)
-- MERGE (self)-[:CONFIGURES]->(resolver)
-- MERGE (self)-[:CREATES]->(profile)
-- MERGE (self)-[:CREATES]->(audit)
-- MERGE (self)-[:CREATES]->(outbox)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:DEFINES]->(successor)
-- MERGE (self)-[:DEFINES]->(append)
-- MERGE (self)-[:DEFINES]->(worm)
-- MERGE (profile)-[:CALLS]->(guard)
-- MERGE (profile)-[:CALLS]->(successor)
-- MERGE (profile)-[:CALLS]->(append)
-- MERGE (audit)-[:CALLS]->(worm)
-- MERGE (outbox)-[:CALLS]->(worm)
-- MERGE (append)-[:WRITES]->(audit)
-- MERGE (append)-[:WRITES]->(outbox)
-- MERGE (owner)-[:READS]->(profile)
-- MERGE (owner)-[:WRITES]->(profile)
-- MERGE (owner)-[:WRITES]->(audit)
-- MERGE (owner)-[:WRITES]->(outbox)
-- MERGE (resolver)-[:READS]->(profile)
-- @endcypher

-- Platform Messaging owns transport selection. Profile revisions are Master facts; audit and
-- versioned Outbox facts are append-only Transactions. No credential bytes or secret references
-- are copied into audit or Outbox payloads.
BEGIN;

CREATE SCHEMA IF NOT EXISTS platform_messaging;
REVOKE ALL ON SCHEMA platform_messaging FROM PUBLIC;

DO $roles$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_profile_owner') THEN
        EXECUTE 'ALTER ROLE star_platform_profile_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_platform_profile_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_profile_resolver') THEN
        EXECUTE 'ALTER ROLE star_platform_profile_resolver NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_platform_profile_resolver NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
END;
$roles$;

CREATE TABLE IF NOT EXISTS platform_messaging.transport_profile_revision (
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    expected_previous_revision BIGINT,
    lifecycle_state VARCHAR(16) NOT NULL
        CHECK (lifecycle_state IN ('active', 'disabled', 'revoked')),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    provider_id VARCHAR(64) NOT NULL CHECK (provider_id ~ '^[a-z][a-z0-9-]{0,63}$'),
    provider_version VARCHAR(64) NOT NULL
        CHECK (provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'),
    endpoint_identity VARCHAR(128) NOT NULL
        CHECK (endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    log_generation_policy VARCHAR(64) NOT NULL
        CHECK (log_generation_policy ~ '^[a-z][a-z0-9._-]{0,63}$'),
    allowed_sender_service_ids UUID[] NOT NULL
        CHECK (cardinality(allowed_sender_service_ids) BETWEEN 1 AND 32),
    allowed_recipient_service_ids UUID[] NOT NULL
        CHECK (cardinality(allowed_recipient_service_ids) BETWEEN 1 AND 32),
    allowed_message_kinds TEXT[] NOT NULL
        CHECK (cardinality(allowed_message_kinds) BETWEEN 1 AND 3
            AND allowed_message_kinds <@ ARRAY['request','result','resolution']::TEXT[]),
    secret_reference VARCHAR(202)
        CHECK (secret_reference IS NULL OR secret_reference ~ '^secretref:[A-Za-z0-9][A-Za-z0-9._/-]{0,191}$'),
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    actor_service_id UUID NOT NULL,
    request_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    causation_id UUID,
    idempotency_key VARCHAR(128) NOT NULL
        CHECK (idempotency_key ~ '^[A-Za-z0-9][A-Za-z0-9:_-]{0,127}$'),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (deployment_id, tenant_id, profile_id, revision),
    UNIQUE (deployment_id, tenant_id, idempotency_key),
    UNIQUE (deployment_id, tenant_id, request_id),
    CHECK (deployment_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (request_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (trace_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (expected_previous_revision IS NULL AND revision = 1)
        OR
        (expected_previous_revision IS NOT NULL AND expected_previous_revision > 0
            AND revision = expected_previous_revision + 1)
    ),
    CHECK (array_position(allowed_sender_service_ids, '00000000-0000-0000-0000-000000000000'::UUID) IS NULL),
    CHECK (array_position(allowed_recipient_service_ids, '00000000-0000-0000-0000-000000000000'::UUID) IS NULL),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_transport_profile_revision_current
    ON platform_messaging.transport_profile_revision (deployment_id, tenant_id, profile_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS ix_transport_profile_revision_lookup
    ON platform_messaging.transport_profile_revision
       (deployment_id, tenant_id, profile_id, revision DESC);

CREATE TABLE IF NOT EXISTS platform_messaging.transport_profile_revision_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    revision BIGINT NOT NULL,
    previous_revision BIGINT,
    event_type VARCHAR(16) NOT NULL
        CHECK (event_type IN ('created','revised','disabled','revoked','reactivated')),
    actor_service_id UUID NOT NULL,
    request_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    causation_id UUID,
    idempotency_key VARCHAR(128) NOT NULL,
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    happened_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    FOREIGN KEY (deployment_id, tenant_id, profile_id, revision)
        REFERENCES platform_messaging.transport_profile_revision
            (deployment_id, tenant_id, profile_id, revision) ON DELETE RESTRICT,
    UNIQUE (deployment_id, tenant_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS ix_transport_profile_revision_audit_lookup
    ON platform_messaging.transport_profile_revision_audit
       (deployment_id, tenant_id, profile_id, revision, happened_at, event_id);

CREATE TABLE IF NOT EXISTS platform_messaging.transport_profile_revision_outbox (
    event_id UUID PRIMARY KEY,
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    revision BIGINT NOT NULL,
    event_schema_version SMALLINT NOT NULL CHECK (event_schema_version = 1),
    event_type VARCHAR(48) NOT NULL CHECK (event_type = 'transport_profile_revision.v1'),
    profile_event_type VARCHAR(16) NOT NULL
        CHECK (profile_event_type IN ('created','revised','disabled','revoked','reactivated')),
    previous_revision BIGINT,
    actor_service_id UUID NOT NULL,
    request_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    causation_id UUID,
    idempotency_key VARCHAR(128) NOT NULL,
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    provider_id VARCHAR(64) NOT NULL,
    provider_version VARCHAR(64) NOT NULL,
    endpoint_identity VARCHAR(128) NOT NULL,
    log_generation_policy VARCHAR(64) NOT NULL,
    allowed_sender_service_ids UUID[] NOT NULL,
    allowed_recipient_service_ids UUID[] NOT NULL,
    allowed_message_kinds TEXT[] NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    FOREIGN KEY (event_id)
        REFERENCES platform_messaging.transport_profile_revision_audit (event_id) ON DELETE RESTRICT,
    UNIQUE (deployment_id, tenant_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS ix_transport_profile_revision_outbox_pending
    ON platform_messaging.transport_profile_revision_outbox (occurred_at, event_id);

CREATE OR REPLACE FUNCTION platform_messaging.guard_transport_profile_revision()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    expected_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    expected_tenant UUID := NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID;
    expected_actor UUID := NULLIF(current_setting('app.platform_actor_service_id', true), '')::UUID;
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF expected_deployment IS DISTINCT FROM NEW.deployment_id
           OR expected_tenant IS DISTINCT FROM NEW.tenant_id
           OR expected_actor IS DISTINCT FROM NEW.actor_service_id
           OR NEW.valid_to IS NOT NULL
           OR NEW.valid_from IS DISTINCT FROM transaction_timestamp()
           OR NEW.revision <> coalesce(NEW.expected_previous_revision, 0) + 1 THEN
            RAISE EXCEPTION 'transport profile revision must be created by the scoped owner with its expected successor';
        END IF;
        PERFORM pg_advisory_xact_lock(hashtextextended(
            NEW.deployment_id::TEXT || ':' || NEW.tenant_id::TEXT || ':' || NEW.profile_id::TEXT, 0
        ));
        IF coalesce((
            SELECT max(p.revision)
              FROM platform_messaging.transport_profile_revision p
             WHERE p.deployment_id = NEW.deployment_id
               AND p.tenant_id = NEW.tenant_id
               AND p.profile_id = NEW.profile_id
        ), 0) <> coalesce(NEW.expected_previous_revision, 0) THEN
            RAISE EXCEPTION 'transport profile compare-and-swap revision conflict';
        END IF;
        IF NEW.expected_previous_revision IS NULL AND NEW.lifecycle_state <> 'active' THEN
            RAISE EXCEPTION 'transport profile must be activated before it can be disabled or revoked';
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL
           OR NEW.valid_to IS NULL
           OR (to_jsonb(NEW) - 'valid_to') IS DISTINCT FROM (to_jsonb(OLD) - 'valid_to')
           OR NEW.valid_to IS DISTINCT FROM transaction_timestamp()
           OR expected_deployment IS DISTINCT FROM NEW.deployment_id
           OR expected_tenant IS DISTINCT FROM NEW.tenant_id
           OR expected_actor IS DISTINCT FROM NEW.actor_service_id THEN
            RAISE EXCEPTION 'transport profile Master rows may only be closed once by their scoped owner';
        END IF;
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'transport profile Master rows cannot be physically deleted';
END;
$$;

CREATE OR REPLACE FUNCTION platform_messaging.require_transport_profile_successor()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
          FROM platform_messaging.transport_profile_revision successor
         WHERE successor.deployment_id = OLD.deployment_id
           AND successor.tenant_id = OLD.tenant_id
           AND successor.profile_id = OLD.profile_id
           AND successor.expected_previous_revision = OLD.revision
           AND successor.revision = OLD.revision + 1
           AND successor.valid_to IS NULL
    ) THEN
        RAISE EXCEPTION 'a closed transport profile revision requires its immutable successor';
    END IF;
    RETURN NULL;
END;
$$;

CREATE OR REPLACE FUNCTION platform_messaging.append_transport_profile_revision_facts()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
DECLARE
    previous_revision BIGINT;
    previous_lifecycle TEXT;
    revision_event_type TEXT;
    revision_event_id UUID := gen_random_uuid();
BEGIN
    SELECT p.revision, p.lifecycle_state
      INTO previous_revision, previous_lifecycle
      FROM platform_messaging.transport_profile_revision p
     WHERE p.deployment_id = NEW.deployment_id
       AND p.tenant_id = NEW.tenant_id
       AND p.profile_id = NEW.profile_id
       AND p.revision = NEW.expected_previous_revision;

    revision_event_type := CASE
        WHEN NEW.lifecycle_state = 'revoked' THEN 'revoked'
        WHEN NEW.lifecycle_state = 'disabled' THEN 'disabled'
        WHEN previous_lifecycle IN ('disabled','revoked') THEN 'reactivated'
        WHEN NEW.expected_previous_revision IS NULL THEN 'created'
        ELSE 'revised'
    END;

    INSERT INTO platform_messaging.transport_profile_revision_audit (
        event_id, deployment_id, tenant_id, profile_id, revision, previous_revision,
        event_type, actor_service_id, request_id, trace_id, causation_id,
        idempotency_key, profile_digest
    ) VALUES (
        revision_event_id, NEW.deployment_id, NEW.tenant_id, NEW.profile_id, NEW.revision,
        previous_revision, revision_event_type, NEW.actor_service_id, NEW.request_id,
        NEW.trace_id, NEW.causation_id, NEW.idempotency_key, NEW.profile_digest
    );

    INSERT INTO platform_messaging.transport_profile_revision_outbox (
        event_id, deployment_id, tenant_id, profile_id, revision, event_schema_version,
        event_type, profile_event_type, previous_revision, actor_service_id, request_id,
        trace_id, causation_id, idempotency_key, profile_digest, provider_id,
        provider_version, endpoint_identity, log_generation_policy,
        allowed_sender_service_ids, allowed_recipient_service_ids, allowed_message_kinds
    ) VALUES (
        revision_event_id, NEW.deployment_id, NEW.tenant_id, NEW.profile_id, NEW.revision, 1,
        'transport_profile_revision.v1', revision_event_type, previous_revision,
        NEW.actor_service_id, NEW.request_id, NEW.trace_id, NEW.causation_id,
        NEW.idempotency_key, NEW.profile_digest, NEW.provider_id, NEW.provider_version,
        NEW.endpoint_identity, NEW.log_generation_policy, NEW.allowed_sender_service_ids,
        NEW.allowed_recipient_service_ids, NEW.allowed_message_kinds
    );
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION platform_messaging.reject_transport_profile_transaction_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    RAISE EXCEPTION 'transport profile audit and Outbox facts are append-only';
END;
$$;

DROP TRIGGER IF EXISTS transport_profile_revision_guard
    ON platform_messaging.transport_profile_revision;
CREATE TRIGGER transport_profile_revision_guard
    BEFORE INSERT OR UPDATE OR DELETE ON platform_messaging.transport_profile_revision
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.guard_transport_profile_revision();
DROP TRIGGER IF EXISTS transport_profile_revision_successor
    ON platform_messaging.transport_profile_revision;
CREATE CONSTRAINT TRIGGER transport_profile_revision_successor
    AFTER UPDATE OF valid_to ON platform_messaging.transport_profile_revision
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW WHEN (OLD.valid_to IS NULL AND NEW.valid_to IS NOT NULL)
    EXECUTE FUNCTION platform_messaging.require_transport_profile_successor();
DROP TRIGGER IF EXISTS transport_profile_revision_append_facts
    ON platform_messaging.transport_profile_revision;
CREATE TRIGGER transport_profile_revision_append_facts
    AFTER INSERT ON platform_messaging.transport_profile_revision
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.append_transport_profile_revision_facts();

DROP TRIGGER IF EXISTS transport_profile_audit_worm
    ON platform_messaging.transport_profile_revision_audit;
CREATE TRIGGER transport_profile_audit_worm
    BEFORE UPDATE OR DELETE ON platform_messaging.transport_profile_revision_audit
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.reject_transport_profile_transaction_mutation();
DROP TRIGGER IF EXISTS transport_profile_audit_truncate_guard
    ON platform_messaging.transport_profile_revision_audit;
CREATE TRIGGER transport_profile_audit_truncate_guard
    BEFORE TRUNCATE ON platform_messaging.transport_profile_revision_audit
    FOR EACH STATEMENT EXECUTE FUNCTION platform_messaging.reject_transport_profile_transaction_mutation();
DROP TRIGGER IF EXISTS transport_profile_outbox_worm
    ON platform_messaging.transport_profile_revision_outbox;
CREATE TRIGGER transport_profile_outbox_worm
    BEFORE UPDATE OR DELETE ON platform_messaging.transport_profile_revision_outbox
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.reject_transport_profile_transaction_mutation();
DROP TRIGGER IF EXISTS transport_profile_outbox_truncate_guard
    ON platform_messaging.transport_profile_revision_outbox;
CREATE TRIGGER transport_profile_outbox_truncate_guard
    BEFORE TRUNCATE ON platform_messaging.transport_profile_revision_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION platform_messaging.reject_transport_profile_transaction_mutation();

ALTER TABLE platform_messaging.transport_profile_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.transport_profile_revision FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS transport_profile_revision_scope
    ON platform_messaging.transport_profile_revision;
CREATE POLICY transport_profile_revision_scope
    ON platform_messaging.transport_profile_revision
    USING (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID
    )
    WITH CHECK (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID
    );

ALTER TABLE platform_messaging.transport_profile_revision_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.transport_profile_revision_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS transport_profile_audit_scope
    ON platform_messaging.transport_profile_revision_audit;
CREATE POLICY transport_profile_audit_scope
    ON platform_messaging.transport_profile_revision_audit
    USING (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID
    )
    WITH CHECK (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID
    );

ALTER TABLE platform_messaging.transport_profile_revision_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.transport_profile_revision_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS transport_profile_outbox_scope
    ON platform_messaging.transport_profile_revision_outbox;
CREATE POLICY transport_profile_outbox_scope
    ON platform_messaging.transport_profile_revision_outbox
    USING (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID
    )
    WITH CHECK (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID
    );

REVOKE ALL ON TABLE
    platform_messaging.transport_profile_revision,
    platform_messaging.transport_profile_revision_audit,
    platform_messaging.transport_profile_revision_outbox
    FROM PUBLIC, star_platform_profile_owner, star_platform_profile_resolver;
GRANT USAGE ON SCHEMA platform_messaging TO star_platform_profile_owner, star_platform_profile_resolver;
GRANT SELECT, INSERT ON platform_messaging.transport_profile_revision
    TO star_platform_profile_owner;
GRANT UPDATE (valid_to) ON platform_messaging.transport_profile_revision
    TO star_platform_profile_owner;
GRANT SELECT ON
    platform_messaging.transport_profile_revision_audit,
    platform_messaging.transport_profile_revision_outbox
    TO star_platform_profile_owner;
GRANT SELECT ON platform_messaging.transport_profile_revision
    TO star_platform_profile_resolver;

REVOKE ALL ON FUNCTION platform_messaging.guard_transport_profile_revision() FROM PUBLIC;
REVOKE ALL ON FUNCTION platform_messaging.require_transport_profile_successor() FROM PUBLIC;
REVOKE ALL ON FUNCTION platform_messaging.append_transport_profile_revision_facts() FROM PUBLIC;
REVOKE ALL ON FUNCTION platform_messaging.reject_transport_profile_transaction_mutation() FROM PUBLIC;

COMMENT ON TABLE platform_messaging.transport_profile_revision IS
    'M: immutable deployment+tenant-scoped transport profile revisions with SCD2 currentness; credentials are never stored.';
COMMENT ON TABLE platform_messaging.transport_profile_revision_audit IS
    'T: append-only actor, revision, digest, trace and idempotency facts; excludes secret references and credentials.';
COMMENT ON TABLE platform_messaging.transport_profile_revision_outbox IS
    'T: append-only, same-transaction versioned profile revision events for BI/Benchmark projections; excludes secret references.';

COMMIT;
