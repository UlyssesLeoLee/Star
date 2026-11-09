-- @cypher schema=1 source_sha256=de07fb002c344afc916902eade95e983443c45b6b74173a1f6acc4a1b120890a
-- MERGE (self:File {path:"db/migrations/2026-10-24-bi-task-event-source-origin.sql"})
-- MERGE (origin:Table {id:"bi.task_event_source_origin",class:"Transaction"})
-- MERGE (commit:Database {id:"bi.commit_task_event_v3",kind:"stored_function"})
-- MERGE (legacy:Database {id:"bi.commit_task_event_v2",kind:"stored_function"})
-- MERGE (inbox:Table {id:"bi.task_event_inbox",class:"Transaction"})
-- MERGE (quarantine:Table {id:"bi.task_event_quarantine",class:"Transaction"})
-- MERGE (quarantine_fn:Database {id:"bi.record_task_event_quarantine_v2",kind:"stored_function"})
-- MERGE (sequence_index:Config {id:"task_event_source_origin_sequence_uq",kind:"postgres_index"})
-- MERGE (cursor_index:Config {id:"task_event_source_origin_cursor_uq",kind:"postgres_index"})
-- MERGE (quarantine_sequence_index:Config {id:"task_event_quarantine_sequence_origin_uidx",kind:"postgres_index"})
-- MERGE (quarantine_cursor_index:Config {id:"task_event_quarantine_cursor_origin_uidx",kind:"postgres_index"})
-- MERGE (self)-[:DEFINES]->(origin)
-- MERGE (self)-[:DEFINES]->(commit)
-- MERGE (self)-[:DEFINES]->(quarantine_fn)
-- MERGE (commit)-[:CALLS]->(legacy)
-- MERGE (commit)-[:READS]->(origin)
-- MERGE (commit)-[:WRITES]->(origin)
-- MERGE (commit)-[:READS]->(inbox)
-- MERGE (origin)-[:DEPENDS_ON]->(inbox)
-- MERGE (quarantine_fn)-[:READS]->(quarantine)
-- MERGE (quarantine_fn)-[:WRITES]->(quarantine)
-- MERGE (self)-[:CONFIGURES]->(sequence_index)
-- MERGE (self)-[:CONFIGURES]->(cursor_index)
-- MERGE (self)-[:CONFIGURES]->(quarantine_sequence_index)
-- MERGE (self)-[:CONFIGURES]->(quarantine_cursor_index)
-- @endcypher

-- Persist the provider-neutral source sequence and exact opaque cursor with each BI Inbox receipt.
-- Legacy receipts have no recoverable source cursor and therefore fail closed on replay.
BEGIN;

CREATE TABLE IF NOT EXISTS bi.task_event_source_origin (
    tenant_id UUID NOT NULL,
    consumer_id VARCHAR(64) NOT NULL DEFAULT 'bi.task-event-fact.v1'
        CHECK (consumer_id = 'bi.task-event-fact.v1'),
    event_id UUID NOT NULL,
    provider_identity VARCHAR(64) NOT NULL
        CHECK (provider_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,63}$'),
    source_log_id VARCHAR(128) NOT NULL
        CHECK (source_log_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    source_log_generation UUID NOT NULL
        CHECK (source_log_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    source_sequence BIGINT NOT NULL CHECK (source_sequence > 0),
    source_cursor BYTEA NOT NULL CHECK (octet_length(source_cursor) BETWEEN 1 AND 512),
    source_message_id UUID NOT NULL
        CHECK (source_message_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, consumer_id, event_id),
    FOREIGN KEY (tenant_id, consumer_id, event_id)
        REFERENCES bi.task_event_inbox (tenant_id, consumer_id, event_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (event_id = source_message_id)
);
ALTER TABLE bi.task_event_source_origin OWNER TO star_bi_owner;
ALTER TABLE bi.task_event_source_origin ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_source_origin FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_event_source_origin_tenant_policy ON bi.task_event_source_origin;
CREATE POLICY task_event_source_origin_tenant_policy ON bi.task_event_source_origin
    FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
CREATE UNIQUE INDEX IF NOT EXISTS task_event_source_origin_sequence_uq
    ON bi.task_event_source_origin (
        tenant_id, consumer_id, provider_identity, source_log_id,
        source_log_generation, source_sequence
    );
CREATE UNIQUE INDEX IF NOT EXISTS task_event_source_origin_cursor_uq
    ON bi.task_event_source_origin (
        tenant_id, consumer_id, provider_identity, source_log_id,
        source_log_generation, source_cursor
    );
DROP TRIGGER IF EXISTS task_event_source_origin_worm ON bi.task_event_source_origin;
CREATE TRIGGER task_event_source_origin_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_source_origin
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_mutation();
REVOKE ALL ON bi.task_event_source_origin FROM PUBLIC, star_bi_runtime;
GRANT SELECT, INSERT ON bi.task_event_source_origin TO star_bi_owner;

CREATE OR REPLACE FUNCTION bi.commit_task_event_v3(
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
    p_wire_sha256 BYTEA,
    p_source_provider_identity VARCHAR,
    p_source_log_id VARCHAR,
    p_source_log_generation UUID,
    p_source_sequence BIGINT,
    p_source_cursor BYTEA,
    p_source_message_id UUID
)
RETURNS TABLE(
    outcome TEXT,
    receipt_id UUID,
    committed_at TIMESTAMPTZ,
    source_provider_identity VARCHAR,
    source_log_id VARCHAR,
    source_log_generation UUID,
    source_sequence BIGINT,
    source_cursor BYTEA,
    source_message_id UUID
)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    commit_outcome TEXT;
    stored_receipt UUID;
    stored_at TIMESTAMPTZ;
    stored_origin bi.task_event_source_origin%ROWTYPE;
    matching_origins INTEGER;
BEGIN
    IF p_tenant_id IS NULL OR p_tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_source_provider_identity IS NULL
       OR p_source_provider_identity !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,63}$'
       OR p_source_provider_identity IS DISTINCT FROM p_transport_provider_id
       OR p_source_log_id IS NULL
       OR p_source_log_id !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_source_log_id IS DISTINCT FROM p_destination_id
       OR p_source_log_generation IS NULL
       OR p_source_log_generation = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_source_log_generation IS DISTINCT FROM p_destination_generation
       OR p_source_sequence IS NULL OR p_source_sequence <= 0
       OR p_source_cursor IS NULL
       OR octet_length(p_source_cursor) NOT BETWEEN 1 AND 512
       OR p_source_message_id IS NULL
       OR p_source_message_id <> p_event_id THEN
        RAISE EXCEPTION 'invalid BI Task source origin' USING ERRCODE = '23514';
    END IF;

    -- Serialize both sequence and opaque-cursor identity. Hash collisions only serialize extra
    -- work; unique indexes below remain the authority for the exact scoped key.
    PERFORM pg_advisory_xact_lock(hashtextextended(
        format('bi-task-origin-sequence/%s/%s/%s/%s/%s/%s', p_tenant_id,
            'bi.task-event-fact.v1', p_source_provider_identity, p_source_log_id,
            p_source_log_generation, p_source_sequence), 0));
    PERFORM pg_advisory_xact_lock(hashtextextended(
        format('bi-task-origin-cursor/%s/%s/%s/%s/%s/%s', p_tenant_id,
            'bi.task-event-fact.v1', p_source_provider_identity, p_source_log_id,
            p_source_log_generation, encode(p_source_cursor, 'hex')), 0));

    SELECT count(*) INTO matching_origins
      FROM bi.task_event_source_origin AS source
     WHERE source.tenant_id = p_tenant_id
       AND source.consumer_id = 'bi.task-event-fact.v1'
       AND source.provider_identity = p_source_provider_identity
       AND source.source_log_id = p_source_log_id
       AND source.source_log_generation = p_source_log_generation
       AND (source.source_sequence = p_source_sequence OR source.source_cursor = p_source_cursor);

    IF matching_origins > 0 THEN
        IF matching_origins = 1 THEN
            SELECT source.*
              INTO stored_origin
              FROM bi.task_event_source_origin AS source
             WHERE source.tenant_id = p_tenant_id
               AND source.consumer_id = 'bi.task-event-fact.v1'
               AND source.provider_identity = p_source_provider_identity
               AND source.source_log_id = p_source_log_id
               AND source.source_log_generation = p_source_log_generation
               AND (source.source_sequence = p_source_sequence OR source.source_cursor = p_source_cursor);
            IF FOUND
               AND stored_origin.event_id = p_event_id
               AND stored_origin.provider_identity = p_source_provider_identity
               AND stored_origin.source_log_id = p_source_log_id
               AND stored_origin.source_log_generation = p_source_log_generation
               AND stored_origin.source_sequence = p_source_sequence
               AND stored_origin.source_cursor = p_source_cursor
               AND stored_origin.source_message_id = p_source_message_id THEN
                -- Continue below so the Inbox function also verifies event content and returns
                -- the original receipt identity and commit timestamp.
                NULL;
            ELSE
                RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ,
                    NULL::VARCHAR, NULL::VARCHAR, NULL::UUID, NULL::BIGINT, NULL::BYTEA, NULL::UUID;
                RETURN;
            END IF;
        ELSE
            RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ,
                NULL::VARCHAR, NULL::VARCHAR, NULL::UUID, NULL::BIGINT, NULL::BYTEA, NULL::UUID;
            RETURN;
        END IF;
    END IF;

    SELECT committed.outcome, committed.receipt_id, committed.committed_at
      INTO commit_outcome, stored_receipt, stored_at
      FROM bi.commit_task_event_v2(
          p_tenant_id, p_project_id, p_engineering_run_id, p_event_id,
          p_work_item_id, p_aggregate_type, p_aggregate_version, p_event_type,
          p_schema_version, p_actor_id, p_correlation_id, p_occurred_at, p_payload,
          p_sender_service_id, p_receiver_service_id, p_transport_provider_id,
          p_transport_provider_version, p_transport_profile_id,
          p_transport_profile_revision, p_transport_profile_digest,
          p_endpoint_identity, p_destination_id, p_destination_generation,
          p_route_pin_sha256, p_wire_sha256
      ) AS committed;

    IF commit_outcome = 'conflict' THEN
        RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ,
            NULL::VARCHAR, NULL::VARCHAR, NULL::UUID, NULL::BIGINT, NULL::BYTEA, NULL::UUID;
        RETURN;
    END IF;

    IF commit_outcome = 'committed' THEN
        INSERT INTO bi.task_event_source_origin (
            tenant_id, event_id, provider_identity, source_log_id,
            source_log_generation, source_sequence, source_cursor, source_message_id
        ) VALUES (
            p_tenant_id, p_event_id, p_source_provider_identity, p_source_log_id,
            p_source_log_generation, p_source_sequence, p_source_cursor, p_source_message_id
        );
        RETURN QUERY SELECT 'committed'::TEXT, stored_receipt, stored_at,
            p_source_provider_identity, p_source_log_id, p_source_log_generation,
            p_source_sequence, p_source_cursor, p_source_message_id;
        RETURN;
    END IF;

    IF commit_outcome <> 'exact_replay' THEN
        RAISE EXCEPTION 'unexpected BI Task Inbox outcome' USING ERRCODE = '23514';
    END IF;

    SELECT source.*
      INTO stored_origin
      FROM bi.task_event_source_origin AS source
     WHERE source.tenant_id = p_tenant_id
       AND source.consumer_id = 'bi.task-event-fact.v1'
       AND source.event_id = p_event_id;

    IF NOT FOUND THEN
        RETURN QUERY SELECT 'source_origin_unverifiable'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ,
            NULL::VARCHAR, NULL::VARCHAR, NULL::UUID, NULL::BIGINT, NULL::BYTEA, NULL::UUID;
        RETURN;
    END IF;

    IF stored_origin.provider_identity = p_source_provider_identity
       AND stored_origin.source_log_id = p_source_log_id
       AND stored_origin.source_log_generation = p_source_log_generation
       AND stored_origin.source_sequence = p_source_sequence
       AND stored_origin.source_cursor = p_source_cursor
       AND stored_origin.source_message_id = p_source_message_id THEN
        RETURN QUERY SELECT 'exact_replay'::TEXT, stored_receipt, stored_at,
            stored_origin.provider_identity, stored_origin.source_log_id,
            stored_origin.source_log_generation, stored_origin.source_sequence,
            stored_origin.source_cursor,
            stored_origin.source_message_id;
        RETURN;
    END IF;

    RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ,
        NULL::VARCHAR, NULL::VARCHAR, NULL::UUID, NULL::BIGINT, NULL::BYTEA, NULL::UUID;
END;
$$;

ALTER FUNCTION bi.commit_task_event_v3(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID
) OWNER TO star_bi_owner;
REVOKE ALL ON FUNCTION bi.commit_task_event_v2(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) FROM PUBLIC, star_bi_runtime;
REVOKE ALL ON FUNCTION bi.commit_task_event_v3(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION bi.commit_task_event_v3(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID
) TO star_bi_runtime;

COMMENT ON TABLE bi.task_event_source_origin IS
    'T: immutable provider-neutral source identity, scoped sequence, and exact opaque cursor committed atomically with the BI Task Inbox/fact; cursors are never interpreted by BI.';
COMMENT ON FUNCTION bi.commit_task_event_v3(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID
) IS 'Atomically commits the BI Task Inbox/fact and durable source origin; exact replay requires matching provider, log, generation, sequence, opaque cursor, and message identity.';

-- Keep quarantine source identity provider-neutral as well. Existing rows retain their legacy
-- sequence field; new rows store both that scoped sequence and the exact opaque cursor. A replay of
-- a legacy row cannot be acknowledged because the cursor needed for exact comparison was not saved.
ALTER TABLE bi.task_event_quarantine
    ADD COLUMN IF NOT EXISTS source_cursor BYTEA,
    DROP CONSTRAINT IF EXISTS task_event_quarantine_cursor_shape_ck,
    ADD CONSTRAINT task_event_quarantine_cursor_shape_ck CHECK (
        source_cursor IS NULL OR octet_length(source_cursor) BETWEEN 1 AND 512
    );

-- Quarantine origin identity is independent of the receiver's route/profile revision. Refuse to
-- silently choose between any historical duplicate source positions before tightening the key.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
          FROM bi.task_event_quarantine
         GROUP BY tenant_id, consumer_id, origin_provider_identity,
                  source_log_id, source_log_generation, source_sequence
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'duplicate BI quarantine source sequence requires manual reconciliation'
            USING ERRCODE = '23505';
    END IF;
    IF EXISTS (
        SELECT 1
          FROM bi.task_event_quarantine
         WHERE source_cursor IS NOT NULL
         GROUP BY tenant_id, consumer_id, origin_provider_identity,
                  source_log_id, source_log_generation, source_cursor
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'duplicate BI quarantine source cursor requires manual reconciliation'
            USING ERRCODE = '23505';
    END IF;
END;
$$;

ALTER TABLE bi.task_event_quarantine
    DROP CONSTRAINT IF EXISTS task_event_quarantine_origin_uq;
DROP INDEX IF EXISTS bi.task_event_quarantine_sequence_origin_uidx;
DROP INDEX IF EXISTS bi.task_event_quarantine_cursor_origin_uidx;
CREATE UNIQUE INDEX task_event_quarantine_sequence_origin_uidx
    ON bi.task_event_quarantine (
        tenant_id, consumer_id, origin_provider_identity,
        source_log_id, source_log_generation, source_sequence
    );
CREATE UNIQUE INDEX task_event_quarantine_cursor_origin_uidx
    ON bi.task_event_quarantine (
        tenant_id, consumer_id, origin_provider_identity,
        source_log_id, source_log_generation, source_cursor
    ) WHERE source_cursor IS NOT NULL;

CREATE OR REPLACE FUNCTION bi.record_task_event_quarantine_v2(
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
    p_source_cursor BYTEA,
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
    matched_rows INTEGER;
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
       OR p_source_log_id IS DISTINCT FROM p_destination_id
       OR p_source_log_generation IS NULL
       OR p_source_sequence IS NULL OR p_source_sequence <= 0
       OR p_source_cursor IS NULL OR octet_length(p_source_cursor) NOT BETWEEN 1 AND 512
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
       OR p_source_log_generation IS DISTINCT FROM p_destination_generation
       OR p_message_id = '00000000-0000-0000-0000-000000000000'::UUID THEN
        RAISE EXCEPTION 'invalid BI Task event quarantine metadata' USING ERRCODE = '23514';
    END IF;

    -- Serialize exact replay and conflicting source positions before writing a quarantine receipt.
    PERFORM pg_advisory_xact_lock(hashtextextended(
        format('bi-task-quarantine-sequence/%s/%s/%s/%s/%s/%s', p_tenant_id,
            'bi.task-event-fact.v1', p_origin_provider_identity, p_source_log_id,
            p_source_log_generation, p_source_sequence), 0));
    PERFORM pg_advisory_xact_lock(hashtextextended(
        format('bi-task-quarantine-cursor/%s/%s/%s/%s/%s/%s', p_tenant_id,
            'bi.task-event-fact.v1', p_origin_provider_identity, p_source_log_id,
            p_source_log_generation, encode(p_source_cursor, 'hex')), 0));

    IF EXISTS (
        SELECT 1 FROM bi.task_event_quarantine AS legacy
         WHERE legacy.tenant_id = p_tenant_id
           AND legacy.consumer_id = 'bi.task-event-fact.v1'
           AND legacy.origin_provider_identity = p_origin_provider_identity
           AND legacy.source_log_generation = p_source_log_generation
           AND legacy.source_log_id = p_source_log_id
           AND legacy.source_sequence = p_source_sequence
           AND legacy.source_cursor IS NULL
    ) THEN
        RETURN QUERY SELECT 'conflict'::TEXT, NULL::UUID, NULL::TIMESTAMPTZ;
        RETURN;
    END IF;

    INSERT INTO bi.task_event_quarantine (
        tenant_id, sender_service_id, receiver_service_id,
        transport_provider_id, transport_provider_version,
        transport_profile_id, transport_profile_revision, transport_profile_digest,
        endpoint_identity, destination_id, destination_generation,
        origin_provider_identity, source_log_id, source_log_generation, source_sequence,
        source_cursor, message_id, wire_sha256, wire_length, fault_code
    ) VALUES (
        p_tenant_id, p_sender_service_id, p_receiver_service_id,
        p_transport_provider_id, p_transport_provider_version,
        p_transport_profile_id, p_transport_profile_revision, p_transport_profile_digest,
        p_endpoint_identity, p_destination_id, p_destination_generation,
        p_origin_provider_identity, p_source_log_id, p_source_log_generation, p_source_sequence,
        p_source_cursor, p_message_id, p_wire_sha256, p_wire_length, p_fault_code
    )
    ON CONFLICT DO NOTHING
    RETURNING bi.task_event_quarantine.quarantine_id, bi.task_event_quarantine.recorded_at
      INTO stored_id, stored_at;

    IF FOUND THEN
        RETURN QUERY SELECT 'recorded'::TEXT, stored_id, stored_at;
        RETURN;
    END IF;

    SELECT count(*) INTO matched_rows
      FROM bi.task_event_quarantine AS saved
     WHERE saved.tenant_id = p_tenant_id
       AND saved.consumer_id = 'bi.task-event-fact.v1'
       AND saved.origin_provider_identity = p_origin_provider_identity
       AND saved.source_log_generation = p_source_log_generation
       AND saved.source_log_id = p_source_log_id
       AND (saved.source_sequence = p_source_sequence OR saved.source_cursor = p_source_cursor);

    IF matched_rows = 1 THEN
        SELECT * INTO stored_row
          FROM bi.task_event_quarantine AS saved
         WHERE saved.tenant_id = p_tenant_id
           AND saved.consumer_id = 'bi.task-event-fact.v1'
           AND saved.origin_provider_identity = p_origin_provider_identity
           AND saved.source_log_generation = p_source_log_generation
           AND saved.source_log_id = p_source_log_id
           AND (saved.source_sequence = p_source_sequence OR saved.source_cursor = p_source_cursor);
        IF FOUND
           AND stored_row.source_sequence = p_source_sequence
           AND stored_row.source_cursor = p_source_cursor
           AND stored_row.message_id = p_message_id
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

ALTER FUNCTION bi.record_task_event_quarantine_v2(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID, BYTEA, INTEGER, VARCHAR
) OWNER TO star_bi_owner;
REVOKE ALL ON FUNCTION bi.record_task_event_quarantine_v1(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, UUID, BYTEA, INTEGER, VARCHAR
) FROM PUBLIC, star_bi_runtime;
REVOKE ALL ON FUNCTION bi.record_task_event_quarantine_v2(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID, BYTEA, INTEGER, VARCHAR
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION bi.record_task_event_quarantine_v2(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID, BYTEA, INTEGER, VARCHAR
) TO star_bi_runtime;
COMMENT ON FUNCTION bi.record_task_event_quarantine_v2(
    UUID, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, VARCHAR, VARCHAR,
    UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA, UUID, BYTEA, INTEGER, VARCHAR
) IS 'Owner-local tenant-scoped metadata-only quarantine; source sequence and opaque cursor uniqueness are preserved, exact replay returns the original receipt, and legacy sequence-only rows fail closed.';

COMMIT;
