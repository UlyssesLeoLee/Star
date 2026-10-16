-- @cypher schema=1 source_sha256=PLACEHOLDER
-- MERGE (self:File {path:"db/migrations/2026-10-16-bi-task-event-inbox.sql"})
-- MERGE (module:Symbol {id:"db/migrations/2026-10-16-bi-task-event-inbox.sql::bi_task_event_inbox_migration",kind:"module"})
-- MERGE (inbox:Table {id:"bi.task_event_inbox",kind:"table"})
-- MERGE (fact:Table {id:"bi.task_event_fact",kind:"table"})
-- MERGE (incident:Table {id:"bi.task_event_integrity_incident",kind:"table"})
-- MERGE (commit:Database {id:"bi.commit_task_event_v1",kind:"stored_function"})
-- MERGE (reject:Database {id:"bi.reject_task_event_mutation",kind:"trigger_function"})
-- MERGE (self)-[:DEFINES]->(module)
-- MERGE (module)-[:DEFINES]->(inbox)
-- MERGE (module)-[:DEFINES]->(fact)
-- MERGE (module)-[:DEFINES]->(incident)
-- MERGE (module)-[:DEFINES]->(commit)
-- MERGE (module)-[:DEFINES]->(reject)
-- MERGE (commit)-[:READS]->(inbox)
-- MERGE (commit)-[:READS]->(fact)
-- MERGE (commit)-[:WRITES]->(inbox)
-- MERGE (commit)-[:WRITES]->(fact)
-- MERGE (commit)-[:WRITES]->(incident)
-- MERGE (inbox)-[:CONFIGURES]->(reject)
-- MERGE (fact)-[:CONFIGURES]->(reject)
-- MERGE (incident)-[:CONFIGURES]->(reject)
-- @endcypher

-- BI-owned durable receipt and immutable fact for the first Task lifecycle subscriber.
-- This is a business Inbox, not a transport ACK and does not settle the source delivery.
BEGIN;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_bi_owner') THEN
        CREATE ROLE star_bi_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOREPLICATION NOBYPASSRLS NOINHERIT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_bi_runtime') THEN
        CREATE ROLE star_bi_runtime NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOREPLICATION NOBYPASSRLS NOINHERIT;
    END IF;
END;
$$;

ALTER ROLE star_bi_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
    NOREPLICATION NOBYPASSRLS NOINHERIT;
ALTER ROLE star_bi_runtime NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
    NOREPLICATION NOBYPASSRLS NOINHERIT;

CREATE SCHEMA IF NOT EXISTS bi AUTHORIZATION star_bi_owner;
ALTER SCHEMA bi OWNER TO star_bi_owner;
GRANT USAGE ON SCHEMA bi TO star_bi_owner, star_bi_runtime;
REVOKE CREATE ON SCHEMA bi FROM PUBLIC, star_bi_runtime;

CREATE TABLE IF NOT EXISTS bi.task_event_inbox (
    tenant_id UUID NOT NULL,
    consumer_id VARCHAR(64) NOT NULL DEFAULT 'bi.task-event-fact.v1'
        CHECK (consumer_id = 'bi.task-event-fact.v1'),
    event_id UUID NOT NULL,
    receipt_id UUID NOT NULL DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    aggregate_type VARCHAR(16) NOT NULL CHECK (aggregate_type IN ('metadata', 'lifecycle')),
    aggregate_version INTEGER NOT NULL CHECK (aggregate_version > 0),
    event_type VARCHAR(64) NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version BETWEEN 1 AND 3),
    sender_service_id UUID NOT NULL,
    receiver_service_id UUID NOT NULL,
    transport_provider_id VARCHAR(64) NOT NULL CHECK (transport_provider_id ~ '^[a-z0-9][a-z0-9-]{0,63}$'),
    transport_provider_version VARCHAR(64) NOT NULL CHECK (transport_provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._:+-]{0,63}$'),
    transport_profile_id UUID NOT NULL,
    transport_profile_revision BIGINT NOT NULL CHECK (transport_profile_revision > 0),
    transport_profile_digest BYTEA NOT NULL CHECK (octet_length(transport_profile_digest) = 32),
    endpoint_identity VARCHAR(128) NOT NULL CHECK (endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    destination_id VARCHAR(128) NOT NULL CHECK (destination_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    destination_generation UUID NOT NULL,
    route_pin_sha256 BYTEA NOT NULL CHECK (octet_length(route_pin_sha256) = 32),
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    committed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, consumer_id, event_id),
    UNIQUE (consumer_id, event_id),
    UNIQUE (receipt_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (event_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (work_item_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (receiver_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (transport_profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (destination_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK ((schema_version IN (1, 2) AND aggregate_type = 'metadata'
            AND event_type IN ('task.created', 'task.updated', 'task.owner_changed'))
        OR (schema_version = 3 AND aggregate_type = 'lifecycle'
            AND event_type = 'task.lifecycle.changed'))
);

CREATE TABLE IF NOT EXISTS bi.task_event_fact (
    tenant_id UUID NOT NULL,
    consumer_id VARCHAR(64) NOT NULL,
    event_id UUID NOT NULL,
    project_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    aggregate_type VARCHAR(16) NOT NULL,
    aggregate_version INTEGER NOT NULL CHECK (aggregate_version > 0),
    event_type VARCHAR(64) NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version BETWEEN 1 AND 3),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    PRIMARY KEY (tenant_id, consumer_id, event_id),
    FOREIGN KEY (tenant_id, consumer_id, event_id)
        REFERENCES bi.task_event_inbox (tenant_id, consumer_id, event_id) ON DELETE RESTRICT,
    CHECK (octet_length(payload::TEXT) <= 8192),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);

CREATE TABLE IF NOT EXISTS bi.task_event_integrity_incident (
    tenant_id UUID NOT NULL,
    incident_id UUID NOT NULL DEFAULT gen_random_uuid(),
    consumer_id VARCHAR(64) NOT NULL DEFAULT 'bi.task-event-fact.v1'
        CHECK (consumer_id = 'bi.task-event-fact.v1'),
    event_id UUID NOT NULL,
    received_project_id UUID NOT NULL,
    received_engineering_run_id UUID NOT NULL,
    received_work_item_id UUID NOT NULL,
    received_aggregate_type VARCHAR(16) NOT NULL,
    received_aggregate_version INTEGER NOT NULL CHECK (received_aggregate_version > 0),
    received_event_type VARCHAR(64) NOT NULL,
    received_schema_version INTEGER NOT NULL CHECK (received_schema_version BETWEEN 1 AND 3),
    received_sender_service_id UUID NOT NULL,
    received_receiver_service_id UUID NOT NULL,
    received_transport_profile_id UUID NOT NULL,
    received_transport_profile_revision BIGINT NOT NULL CHECK (received_transport_profile_revision > 0),
    received_transport_profile_digest BYTEA NOT NULL CHECK (octet_length(received_transport_profile_digest) = 32),
    received_endpoint_identity VARCHAR(128) NOT NULL CHECK (received_endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    received_destination_id VARCHAR(128) NOT NULL CHECK (received_destination_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    received_destination_generation UUID NOT NULL,
    received_route_pin_sha256 BYTEA NOT NULL CHECK (octet_length(received_route_pin_sha256) = 32),
    received_wire_sha256 BYTEA NOT NULL CHECK (octet_length(received_wire_sha256) = 32),
    observed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    reason_code VARCHAR(32) NOT NULL DEFAULT 'event_identity_conflict'
        CHECK (reason_code = 'event_identity_conflict'),
    PRIMARY KEY (tenant_id, incident_id),
    CHECK (received_sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (received_receiver_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (received_transport_profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (received_destination_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    UNIQUE (tenant_id, consumer_id, event_id, received_wire_sha256)
);

ALTER TABLE bi.task_event_inbox OWNER TO star_bi_owner;
ALTER TABLE bi.task_event_fact OWNER TO star_bi_owner;
ALTER TABLE bi.task_event_integrity_incident OWNER TO star_bi_owner;

ALTER TABLE bi.task_event_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_fact ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_fact FORCE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_integrity_incident ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_integrity_incident FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS task_event_inbox_owner_tenant ON bi.task_event_inbox;
CREATE POLICY task_event_inbox_owner_tenant ON bi.task_event_inbox
    FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_event_fact_owner_tenant ON bi.task_event_fact;
CREATE POLICY task_event_fact_owner_tenant ON bi.task_event_fact
    FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_event_integrity_incident_owner_tenant
    ON bi.task_event_integrity_incident;
CREATE POLICY task_event_integrity_incident_owner_tenant
    ON bi.task_event_integrity_incident FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

CREATE OR REPLACE FUNCTION bi.reject_task_event_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'BI Task event facts are append-only' USING ERRCODE = '55000';
END;
$$;
ALTER FUNCTION bi.reject_task_event_mutation() OWNER TO star_bi_owner;

DROP TRIGGER IF EXISTS task_event_inbox_worm ON bi.task_event_inbox;
CREATE TRIGGER task_event_inbox_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_inbox
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_mutation();
DROP TRIGGER IF EXISTS task_event_fact_worm ON bi.task_event_fact;
CREATE TRIGGER task_event_fact_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_fact
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_mutation();
DROP TRIGGER IF EXISTS task_event_integrity_incident_worm
    ON bi.task_event_integrity_incident;
CREATE TRIGGER task_event_integrity_incident_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_integrity_incident
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_mutation();

CREATE OR REPLACE FUNCTION bi.commit_task_event_v1(
    p_tenant_id UUID,
    p_project_id UUID,
    p_engineering_run_id UUID,
    p_event_id UUID,
    p_work_item_id UUID,
    p_aggregate_type VARCHAR,
    p_aggregate_version INTEGER,
    p_event_type VARCHAR,
    p_schema_version INTEGER,
    p_actor_id UUID,
    p_correlation_id UUID,
    p_occurred_at TIMESTAMPTZ,
    p_payload JSONB,
    p_sender_service_id UUID,
    p_receiver_service_id UUID,
    p_transport_provider_id VARCHAR,
    p_transport_provider_version VARCHAR,
    p_transport_profile_id UUID,
    p_transport_profile_revision BIGINT,
    p_transport_profile_digest BYTEA,
    p_endpoint_identity VARCHAR,
    p_destination_id VARCHAR,
    p_destination_generation UUID,
    p_route_pin_sha256 BYTEA,
    p_wire_sha256 BYTEA
)
RETURNS TABLE(outcome TEXT, receipt_id UUID, committed_at TIMESTAMPTZ)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    stored_receipt UUID;
    stored_at TIMESTAMPTZ;
    stored_row RECORD;
BEGIN
    IF active_tenant IS NULL OR p_tenant_id IS DISTINCT FROM active_tenant THEN
        RAISE EXCEPTION 'BI tenant scope mismatch' USING ERRCODE = '42501';
    END IF;
    IF p_tenant_id IS NULL OR p_project_id IS NULL OR p_engineering_run_id IS NULL
       OR p_event_id IS NULL OR p_work_item_id IS NULL OR p_actor_id IS NULL
       OR p_correlation_id IS NULL OR p_occurred_at IS NULL OR p_payload IS NULL
       OR p_wire_sha256 IS NULL OR octet_length(p_wire_sha256) <> 32
       OR octet_length(p_payload::TEXT) > 8192 OR jsonb_typeof(p_payload) <> 'object'
       OR p_aggregate_version <= 0
       OR p_tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_project_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_engineering_run_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_event_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_work_item_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_actor_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_correlation_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_sender_service_id IS NULL OR p_receiver_service_id IS NULL
       OR p_sender_service_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_receiver_service_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_transport_provider_id !~ '^[a-z0-9][a-z0-9-]{0,63}$'
       OR p_transport_provider_version !~ '^[A-Za-z0-9][A-Za-z0-9._:+-]{0,63}$'
       OR p_transport_profile_id IS NULL
       OR p_transport_profile_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_transport_profile_revision <= 0
       OR p_transport_profile_digest IS NULL OR octet_length(p_transport_profile_digest) <> 32
       OR p_endpoint_identity !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_destination_id !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_destination_generation IS NULL
       OR p_destination_generation = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_route_pin_sha256 IS NULL OR octet_length(p_route_pin_sha256) <> 32 THEN
        RAISE EXCEPTION 'invalid BI Task event envelope' USING ERRCODE = '23514';
    END IF;
    IF NOT (
        (p_schema_version IN (1, 2) AND p_aggregate_type = 'metadata'
         AND p_event_type IN ('task.created', 'task.updated', 'task.owner_changed'))
        OR (p_schema_version = 3 AND p_aggregate_type = 'lifecycle'
            AND p_event_type = 'task.lifecycle.changed')
    ) THEN
        RAISE EXCEPTION 'unsupported BI Task event contract' USING ERRCODE = '23514';
    END IF;
    IF p_payload ->> 'work_item_id' IS DISTINCT FROM p_work_item_id::TEXT THEN
        RAISE EXCEPTION 'BI Task event work item scope mismatch' USING ERRCODE = '23514';
    END IF;
    IF p_schema_version = 1 AND (
        p_payload ->> 'project_id' IS DISTINCT FROM p_project_id::TEXT
        OR p_payload ->> 'engineering_run_id' IS DISTINCT FROM p_engineering_run_id::TEXT
        OR p_payload ->> 'metadata_version' IS DISTINCT FROM p_aggregate_version::TEXT
    ) THEN
        RAISE EXCEPTION 'BI Task v1 scope mismatch' USING ERRCODE = '23514';
    ELSIF p_schema_version = 2 AND (
        p_payload ->> 'schema_version' IS DISTINCT FROM '2'
        OR p_payload #>> '{owner,tenant_id}' IS DISTINCT FROM p_tenant_id::TEXT
        OR p_payload #>> '{owner,project_id}' IS DISTINCT FROM p_project_id::TEXT
        OR p_payload #>> '{owner,engineering_run_id}' IS DISTINCT FROM p_engineering_run_id::TEXT
        OR p_payload ->> 'metadata_version' IS DISTINCT FROM p_aggregate_version::TEXT
    ) THEN
        RAISE EXCEPTION 'BI Task v2 scope mismatch' USING ERRCODE = '23514';
    ELSIF p_schema_version = 3 AND (
        p_payload ->> 'schema_version' IS DISTINCT FROM '3'
        OR p_payload #>> '{aggregate,type}' IS DISTINCT FROM p_aggregate_type
        OR p_payload #>> '{aggregate,version}' IS DISTINCT FROM p_aggregate_version::TEXT
        OR p_payload #>> '{owner,tenant_id}' IS DISTINCT FROM p_tenant_id::TEXT
        OR p_payload #>> '{owner,project_id}' IS DISTINCT FROM p_project_id::TEXT
        OR p_payload #>> '{owner,engineering_run_id}' IS DISTINCT FROM p_engineering_run_id::TEXT
    ) THEN
        RAISE EXCEPTION 'BI Task v3 scope mismatch' USING ERRCODE = '23514';
    END IF;

    INSERT INTO bi.task_event_inbox (
        tenant_id, event_id, project_id, engineering_run_id, work_item_id,
        aggregate_type, aggregate_version, event_type, schema_version,
        sender_service_id, receiver_service_id, transport_provider_id,
        transport_provider_version, transport_profile_id, transport_profile_revision,
        transport_profile_digest, endpoint_identity, destination_id, destination_generation,
        route_pin_sha256, wire_sha256
    ) VALUES (
        p_tenant_id, p_event_id, p_project_id, p_engineering_run_id, p_work_item_id,
        p_aggregate_type, p_aggregate_version, p_event_type, p_schema_version,
        p_sender_service_id, p_receiver_service_id, p_transport_provider_id,
        p_transport_provider_version, p_transport_profile_id, p_transport_profile_revision,
        p_transport_profile_digest, p_endpoint_identity, p_destination_id,
        p_destination_generation, p_route_pin_sha256, p_wire_sha256
    )
    ON CONFLICT (consumer_id, event_id) DO NOTHING
    RETURNING bi.task_event_inbox.receipt_id, bi.task_event_inbox.committed_at
      INTO stored_receipt, stored_at;

    IF FOUND THEN
        INSERT INTO bi.task_event_fact (
            tenant_id, consumer_id, event_id, project_id, engineering_run_id,
            work_item_id, aggregate_type, aggregate_version, event_type, schema_version,
            actor_id, correlation_id, occurred_at, payload, wire_sha256
        ) VALUES (
            p_tenant_id, 'bi.task-event-fact.v1', p_event_id, p_project_id,
            p_engineering_run_id, p_work_item_id, p_aggregate_type, p_aggregate_version,
            p_event_type, p_schema_version, p_actor_id, p_correlation_id, p_occurred_at,
            p_payload, p_wire_sha256
        );
        RETURN QUERY SELECT 'committed'::TEXT, stored_receipt, stored_at;
        RETURN;
    END IF;

    SELECT i.*, f.actor_id, f.correlation_id, f.occurred_at, f.payload, f.wire_sha256 AS fact_wire_sha256
      INTO stored_row
      FROM bi.task_event_inbox AS i
      JOIN bi.task_event_fact AS f
        ON f.tenant_id = i.tenant_id
       AND f.consumer_id = i.consumer_id
       AND f.event_id = i.event_id
     WHERE i.consumer_id = 'bi.task-event-fact.v1'
       AND i.event_id = p_event_id;

    IF FOUND
       AND stored_row.tenant_id = p_tenant_id
       AND stored_row.project_id = p_project_id
       AND stored_row.engineering_run_id = p_engineering_run_id
       AND stored_row.work_item_id = p_work_item_id
       AND stored_row.aggregate_type = p_aggregate_type
       AND stored_row.aggregate_version = p_aggregate_version
       AND stored_row.event_type = p_event_type
       AND stored_row.schema_version = p_schema_version
       AND stored_row.sender_service_id = p_sender_service_id
       AND stored_row.receiver_service_id = p_receiver_service_id
       AND stored_row.transport_provider_id = p_transport_provider_id
       AND stored_row.transport_provider_version = p_transport_provider_version
       AND stored_row.transport_profile_id = p_transport_profile_id
       AND stored_row.transport_profile_revision = p_transport_profile_revision
       AND stored_row.transport_profile_digest = p_transport_profile_digest
       AND stored_row.endpoint_identity = p_endpoint_identity
       AND stored_row.destination_id = p_destination_id
       AND stored_row.destination_generation = p_destination_generation
       AND stored_row.route_pin_sha256 = p_route_pin_sha256
       AND stored_row.wire_sha256 = p_wire_sha256
       AND stored_row.fact_wire_sha256 = p_wire_sha256
       AND stored_row.actor_id = p_actor_id
       AND stored_row.correlation_id = p_correlation_id
       AND stored_row.occurred_at = p_occurred_at
       AND stored_row.payload = p_payload THEN
        RETURN QUERY SELECT 'exact_replay'::TEXT, stored_row.receipt_id, stored_row.committed_at;
        RETURN;
    END IF;

    INSERT INTO bi.task_event_integrity_incident (
        tenant_id, event_id, received_project_id, received_engineering_run_id,
        received_work_item_id, received_aggregate_type, received_aggregate_version,
        received_event_type, received_schema_version, received_sender_service_id,
        received_receiver_service_id, received_transport_profile_id,
        received_transport_profile_revision, received_transport_profile_digest,
        received_endpoint_identity, received_destination_id,
        received_destination_generation, received_route_pin_sha256, received_wire_sha256
    ) VALUES (
        p_tenant_id, p_event_id, p_project_id, p_engineering_run_id,
        p_work_item_id, p_aggregate_type, p_aggregate_version, p_event_type,
        p_schema_version, p_sender_service_id, p_receiver_service_id,
        p_transport_profile_id, p_transport_profile_revision, p_transport_profile_digest,
        p_endpoint_identity, p_destination_id, p_destination_generation,
        p_route_pin_sha256, p_wire_sha256
    ) ON CONFLICT (tenant_id, consumer_id, event_id, received_wire_sha256) DO NOTHING;
    RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ;
END;
$$;
ALTER FUNCTION bi.commit_task_event_v1(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) OWNER TO star_bi_owner;

REVOKE ALL ON bi.task_event_inbox, bi.task_event_fact,
    bi.task_event_integrity_incident FROM PUBLIC, star_bi_runtime;
REVOKE ALL ON FUNCTION bi.commit_task_event_v1(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION bi.commit_task_event_v1(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) TO star_bi_runtime;

COMMENT ON TABLE bi.task_event_inbox IS
    'T: BI subscriber durable idempotency and commit receipt; only bi.commit_task_event_v1 may insert.';
COMMENT ON TABLE bi.task_event_fact IS
    'T: immutable BI-owned Task event fact committed atomically with the subscriber Inbox; it is not a materialized report projection.';
COMMENT ON TABLE bi.task_event_integrity_incident IS
    'T: append-only receipt of Task event identity/content conflicts; contains bounded metadata, not event payload.';
COMMENT ON FUNCTION bi.commit_task_event_v1(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) IS
    'Commits BI Inbox receipt and immutable event fact atomically; exact replays return the stable receipt; identity conflicts are recorded without returning a receipt or settling source delivery.';

COMMIT;
