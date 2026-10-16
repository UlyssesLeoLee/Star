-- @cypher schema=1 source_sha256=bca893474999619b55fde30895e53b43280a3d8f78281f5babd7e925944ce547
-- MERGE (self:File {path:"db/migrations/2026-10-17-bi-task-event-quarantine.sql"})
-- MERGE (module:Symbol {id:"db/migrations/2026-10-17-bi-task-event-quarantine.sql::bi_task_event_quarantine_migration",kind:"module"})
-- MERGE (table:Table {id:"bi.task_event_quarantine",kind:"table"})
-- MERGE (writer:Database {id:"bi.record_task_event_quarantine_v1",kind:"stored_function"})
-- MERGE (reject:Database {id:"bi.reject_task_event_quarantine_mutation",kind:"trigger_function"})
-- MERGE (self)-[:DEFINES]->(module)
-- MERGE (module)-[:DEFINES]->(table)
-- MERGE (module)-[:DEFINES]->(writer)
-- MERGE (module)-[:DEFINES]->(reject)
-- MERGE (writer)-[:WRITES]->(table)
-- MERGE (table)-[:CONFIGURES]->(reject)
-- @endcypher

-- T: durable BI quarantine audit. Store route/origin metadata and digests only; never store event bytes.
BEGIN;

CREATE TABLE IF NOT EXISTS bi.task_event_quarantine (
    tenant_id UUID NOT NULL,
    quarantine_id UUID NOT NULL DEFAULT gen_random_uuid(),
    consumer_id VARCHAR(64) NOT NULL DEFAULT 'bi.task-event-fact.v1'
        CHECK (consumer_id = 'bi.task-event-fact.v1'),
    sender_service_id UUID NOT NULL,
    receiver_service_id UUID NOT NULL,
    transport_provider_id VARCHAR(64) NOT NULL
        CHECK (transport_provider_id ~ '^[a-z0-9][a-z0-9-]{0,63}$'),
    transport_provider_version VARCHAR(64) NOT NULL
        CHECK (transport_provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._:+-]{0,63}$'),
    transport_profile_id UUID NOT NULL,
    transport_profile_revision BIGINT NOT NULL CHECK (transport_profile_revision > 0),
    transport_profile_digest BYTEA NOT NULL CHECK (octet_length(transport_profile_digest) = 32),
    endpoint_identity VARCHAR(128) NOT NULL
        CHECK (endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    destination_id VARCHAR(128) NOT NULL
        CHECK (destination_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    destination_generation UUID NOT NULL,
    origin_provider_identity VARCHAR(64) NOT NULL
        CHECK (origin_provider_identity ~ '^[a-z0-9][a-z0-9-]{0,63}$'),
    source_log_id VARCHAR(128) NOT NULL
        CHECK (source_log_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    source_log_generation UUID NOT NULL,
    source_sequence BIGINT NOT NULL CHECK (source_sequence > 0),
    message_id UUID NOT NULL,
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    wire_length INTEGER NOT NULL CHECK (wire_length BETWEEN 0 AND 16384),
    fault_code VARCHAR(32) NOT NULL
        CHECK (fault_code IN ('malformed_envelope', 'message_identity_mismatch', 'content_conflict')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, quarantine_id),
    CONSTRAINT task_event_quarantine_origin_uq UNIQUE (
        tenant_id, consumer_id,
        sender_service_id, receiver_service_id,
        transport_provider_id, transport_provider_version,
        transport_profile_id, transport_profile_revision, transport_profile_digest,
        endpoint_identity, destination_id, destination_generation,
        origin_provider_identity, source_log_generation, source_log_id, source_sequence
    ),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (receiver_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (transport_profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (destination_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (source_log_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (message_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (origin_provider_identity = transport_provider_id)
);

ALTER TABLE bi.task_event_quarantine OWNER TO star_bi_owner;
ALTER TABLE bi.task_event_quarantine ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_quarantine FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_event_quarantine_owner_tenant ON bi.task_event_quarantine;
CREATE POLICY task_event_quarantine_owner_tenant ON bi.task_event_quarantine
    FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

CREATE OR REPLACE FUNCTION bi.reject_task_event_quarantine_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'BI Task event quarantine is append-only' USING ERRCODE = '55000';
END;
$$;
ALTER FUNCTION bi.reject_task_event_quarantine_mutation() OWNER TO star_bi_owner;
DROP TRIGGER IF EXISTS task_event_quarantine_worm ON bi.task_event_quarantine;
CREATE TRIGGER task_event_quarantine_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_quarantine
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_quarantine_mutation();

CREATE OR REPLACE FUNCTION bi.record_task_event_quarantine_v1(
    p_tenant_id UUID,
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
    p_origin_provider_identity VARCHAR,
    p_source_log_id VARCHAR,
    p_source_log_generation UUID,
    p_source_sequence BIGINT,
    p_message_id UUID,
    p_wire_sha256 BYTEA,
    p_wire_length INTEGER,
    p_fault_code VARCHAR
)
RETURNS TABLE(outcome TEXT, quarantine_id UUID, recorded_at TIMESTAMPTZ)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    stored_id UUID;
    stored_at TIMESTAMPTZ;
    stored_row RECORD;
BEGIN
    IF active_tenant IS NULL OR p_tenant_id IS DISTINCT FROM active_tenant THEN
        RAISE EXCEPTION 'BI tenant scope mismatch' USING ERRCODE = '42501';
    END IF;
    IF p_tenant_id IS NULL
       OR p_sender_service_id IS NULL OR p_receiver_service_id IS NULL
       OR p_transport_provider_id IS NULL
       OR p_transport_provider_id !~ '^[a-z0-9][a-z0-9-]{0,63}$'
       OR p_transport_provider_version IS NULL
       OR p_transport_provider_version !~ '^[A-Za-z0-9][A-Za-z0-9._:+-]{0,63}$'
       OR p_transport_profile_id IS NULL
       OR p_transport_profile_revision IS NULL OR p_transport_profile_revision <= 0
       OR p_transport_profile_digest IS NULL OR octet_length(p_transport_profile_digest) <> 32
       OR p_endpoint_identity IS NULL
       OR p_endpoint_identity !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_destination_id IS NULL
       OR p_destination_id !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_destination_generation IS NULL
       OR p_origin_provider_identity IS NULL
       OR p_origin_provider_identity !~ '^[a-z0-9][a-z0-9-]{0,63}$'
       OR p_source_log_id IS NULL
       OR p_source_log_id !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_source_log_generation IS NULL
       OR p_source_sequence IS NULL OR p_source_sequence <= 0
       OR p_message_id IS NULL
       OR p_wire_sha256 IS NULL OR octet_length(p_wire_sha256) <> 32
       OR p_wire_length IS NULL OR p_wire_length NOT BETWEEN 0 AND 16384
       OR p_fault_code IS NULL
       OR p_fault_code NOT IN ('malformed_envelope', 'message_identity_mismatch', 'content_conflict')
       OR p_origin_provider_identity IS DISTINCT FROM p_transport_provider_id
       OR p_tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_sender_service_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_receiver_service_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_transport_profile_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_destination_generation = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_source_log_generation = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_message_id = '00000000-0000-0000-0000-000000000000'::UUID THEN
        RAISE EXCEPTION 'invalid BI Task event quarantine metadata' USING ERRCODE = '23514';
    END IF;

    INSERT INTO bi.task_event_quarantine (
        tenant_id, sender_service_id, receiver_service_id,
        transport_provider_id, transport_provider_version,
        transport_profile_id, transport_profile_revision, transport_profile_digest,
        endpoint_identity, destination_id, destination_generation,
        origin_provider_identity, source_log_id, source_log_generation, source_sequence,
        message_id, wire_sha256, wire_length, fault_code
    ) VALUES (
        p_tenant_id, p_sender_service_id, p_receiver_service_id,
        p_transport_provider_id, p_transport_provider_version,
        p_transport_profile_id, p_transport_profile_revision, p_transport_profile_digest,
        p_endpoint_identity, p_destination_id, p_destination_generation,
        p_origin_provider_identity, p_source_log_id, p_source_log_generation, p_source_sequence,
        p_message_id, p_wire_sha256, p_wire_length, p_fault_code
    )
    ON CONFLICT ON CONSTRAINT task_event_quarantine_origin_uq DO NOTHING
    RETURNING bi.task_event_quarantine.quarantine_id, bi.task_event_quarantine.recorded_at
      INTO stored_id, stored_at;

    IF FOUND THEN
        RETURN QUERY SELECT 'recorded'::TEXT, stored_id, stored_at;
        RETURN;
    END IF;

    SELECT * INTO stored_row
      FROM bi.task_event_quarantine
     WHERE tenant_id = p_tenant_id
       AND consumer_id = 'bi.task-event-fact.v1'
       AND sender_service_id = p_sender_service_id
       AND receiver_service_id = p_receiver_service_id
       AND transport_provider_id = p_transport_provider_id
       AND transport_provider_version = p_transport_provider_version
       AND transport_profile_id = p_transport_profile_id
       AND transport_profile_revision = p_transport_profile_revision
       AND transport_profile_digest = p_transport_profile_digest
       AND endpoint_identity = p_endpoint_identity
       AND destination_id = p_destination_id
       AND destination_generation = p_destination_generation
       AND origin_provider_identity = p_origin_provider_identity
       AND source_log_generation = p_source_log_generation
       AND source_log_id = p_source_log_id
       AND source_sequence = p_source_sequence;

    IF FOUND THEN
        IF stored_row.message_id = p_message_id
           AND stored_row.wire_sha256 = p_wire_sha256
           AND stored_row.wire_length = p_wire_length
           AND stored_row.fault_code = p_fault_code THEN
            RETURN QUERY SELECT 'exact_replay'::TEXT, stored_row.quarantine_id, stored_row.recorded_at;
            RETURN;
        END IF;
    END IF;

    RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ;
END;
$$;
ALTER FUNCTION bi.record_task_event_quarantine_v1(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, UUID, BYTEA, INTEGER, VARCHAR
) OWNER TO star_bi_owner;

REVOKE ALL ON bi.task_event_quarantine FROM PUBLIC, star_bi_runtime;
REVOKE ALL ON FUNCTION bi.record_task_event_quarantine_v1(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, UUID, BYTEA, INTEGER, VARCHAR
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION bi.record_task_event_quarantine_v1(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, UUID, BYTEA, INTEGER, VARCHAR
) TO star_bi_runtime;

COMMENT ON TABLE bi.task_event_quarantine IS
    'T: append-only BI quarantine audit containing route/source cursor, digest, length and fixed fault code only; event wire/payload is never stored.';
COMMENT ON FUNCTION bi.record_task_event_quarantine_v1(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, UUID, BYTEA, INTEGER, VARCHAR
) IS
    'Owner-local, tenant-scoped metadata-only quarantine; exact cursor replay returns its original receipt and changed content fails closed.';

COMMIT;
