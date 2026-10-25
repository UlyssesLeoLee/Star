-- @cypher schema=1 source_sha256=d2ee89e43fabbc107a5336bdd791e67349b50c879c07b948eee4f762450465e8
-- MERGE (self:File {path:"db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql"})
-- MERGE (claim:Table {id:"bi.task_event_source_position_claim",class:"Transaction"})
-- MERGE (source_origin:Table {id:"bi.task_event_source_origin",class:"Transaction"})
-- MERGE (quarantine:Table {id:"bi.task_event_quarantine",class:"Transaction"})
-- MERGE (inbox:Table {id:"bi.task_event_inbox",class:"Transaction"})
-- MERGE (claim_fn:Database {id:"bi.claim_task_event_source_position",kind:"trigger_function"})
-- MERGE (reject_fn:Database {id:"bi.reject_task_event_source_position_claim_mutation",kind:"trigger_function"})
-- MERGE (claim_sequence_idx:Config {id:"task_event_source_position_claim_sequence_uq",kind:"postgres_index"})
-- MERGE (claim_cursor_idx:Config {id:"task_event_source_position_claim_cursor_uq",kind:"postgres_index"})
-- MERGE (claim_cursorless_idx:Config {id:"task_event_source_position_claim_cursorless_scope_idx",kind:"postgres_index"})
-- MERGE (cursor_history_lock:Config {id:"bi-task-source-cursor-history",kind:"postgres_advisory_lock",modes:["shared","exclusive"]})
-- MERGE (claim_isolation:Config {id:"READ COMMITTED",kind:"postgres_transaction_isolation"})
-- MERGE (source_trigger:Config {id:"task_event_source_origin_claim_position",kind:"postgres_trigger"})
-- MERGE (quarantine_trigger:Config {id:"task_event_quarantine_claim_position",kind:"postgres_trigger"})
-- MERGE (claim_worm:Config {id:"task_event_source_position_claim_worm",kind:"postgres_trigger"})
-- MERGE (self)-[:DEFINES]->(claim)
-- MERGE (self)-[:DEFINES]->(claim_fn)
-- MERGE (self)-[:DEFINES]->(reject_fn)
-- MERGE (self)-[:CONFIGURES]->(claim_sequence_idx)
-- MERGE (self)-[:CONFIGURES]->(claim_cursor_idx)
-- MERGE (self)-[:CONFIGURES]->(claim_cursorless_idx)
-- MERGE (self)-[:CONFIGURES]->(cursor_history_lock)
-- MERGE (self)-[:CONFIGURES]->(claim_isolation)
-- MERGE (self)-[:CONFIGURES]->(source_trigger)
-- MERGE (self)-[:CONFIGURES]->(quarantine_trigger)
-- MERGE (self)-[:CONFIGURES]->(claim_worm)
-- MERGE (claim_fn)-[:READS]->(source_origin)
-- MERGE (claim_fn)-[:READS]->(quarantine)
-- MERGE (claim_fn)-[:READS]->(inbox)
-- MERGE (claim_fn)-[:USES]->(cursor_history_lock)
-- MERGE (claim_fn)-[:USES]->(claim_isolation)
-- MERGE (claim_fn)-[:READS]->(claim_cursorless_idx)
-- MERGE (claim_fn)-[:READS]->(claim_cursor_idx)
-- MERGE (claim_fn)-[:WRITES]->(claim)
-- MERGE (claim)-[:CONFIGURES]->(claim_worm)
-- MERGE (claim_worm)-[:CALLS]->(reject_fn)
-- MERGE (source_origin)-[:CONFIGURES]->(source_trigger)
-- MERGE (quarantine)-[:CONFIGURES]->(quarantine_trigger)
-- @endcypher

-- T: one immutable BI transaction claim arbitrates a provider-neutral source position across
-- Inbox receipt and metadata quarantine. No message bytes are retained in this claim ledger.
-- Lock both writers across collision detection, backfill, and trigger installation so no delivery
-- can enter between the snapshot and the shared unique constraints becoming authoritative.
BEGIN;

-- Historical collision detection and backfill must see every tenant. A migration role without
-- RLS-bypass authority must fail instead of silently validating/backfilling only visible rows.
SET LOCAL row_security = off;

LOCK TABLE bi.task_event_source_origin IN SHARE ROW EXCLUSIVE MODE;
LOCK TABLE bi.task_event_quarantine IN SHARE ROW EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (
        WITH positions AS (
            SELECT tenant_id, consumer_id, provider_identity,
                   source_log_id, source_log_generation, source_sequence
              FROM bi.task_event_source_origin
            UNION ALL
            SELECT tenant_id, consumer_id, origin_provider_identity,
                   source_log_id, source_log_generation, source_sequence
              FROM bi.task_event_quarantine
        )
        SELECT 1 FROM positions
         GROUP BY tenant_id, consumer_id, provider_identity,
                  source_log_id, source_log_generation, source_sequence
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'cross-disposition BI source sequence collision requires manual reconciliation'
            USING ERRCODE = '23505';
    END IF;

    IF EXISTS (
        WITH positions AS (
            SELECT tenant_id, consumer_id, provider_identity,
                   source_log_id, source_log_generation, source_cursor
              FROM bi.task_event_source_origin
            UNION ALL
            SELECT tenant_id, consumer_id, origin_provider_identity,
                   source_log_id, source_log_generation, source_cursor
              FROM bi.task_event_quarantine
             WHERE source_cursor IS NOT NULL
        )
        SELECT 1 FROM positions
         WHERE source_cursor IS NOT NULL
         GROUP BY tenant_id, consumer_id, provider_identity,
                  source_log_id, source_log_generation, source_cursor
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'cross-disposition BI source cursor collision requires manual reconciliation'
            USING ERRCODE = '23505';
    END IF;
END;
$$;

CREATE TABLE bi.task_event_source_position_claim (
    tenant_id UUID NOT NULL,
    consumer_id VARCHAR(64) NOT NULL CHECK (consumer_id = 'bi.task-event-fact.v1'),
    provider_identity VARCHAR(64) NOT NULL,
    source_log_id VARCHAR(128) NOT NULL,
    source_log_generation UUID NOT NULL,
    source_sequence BIGINT NOT NULL CHECK (source_sequence > 0),
    source_cursor BYTEA,
    source_message_id UUID,
    disposition VARCHAR(16) NOT NULL CHECK (disposition IN ('inbox', 'quarantine')),
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    claimed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (
        tenant_id, consumer_id, provider_identity, source_log_id,
        source_log_generation, source_sequence
    ),
    CHECK (provider_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,63}$'),
    CHECK (source_log_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    CHECK (source_log_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (source_cursor IS NULL OR octet_length(source_cursor) BETWEEN 1 AND 512),
    CHECK (disposition <> 'inbox' OR (source_cursor IS NOT NULL AND source_message_id IS NOT NULL)),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE UNIQUE INDEX task_event_source_position_claim_cursor_uq
    ON bi.task_event_source_position_claim (
        tenant_id, consumer_id, provider_identity, source_log_id,
        source_log_generation, source_cursor
    ) WHERE source_cursor IS NOT NULL;
CREATE INDEX task_event_source_position_claim_cursorless_scope_idx
    ON bi.task_event_source_position_claim (
        tenant_id, consumer_id, provider_identity, source_log_id,
        source_log_generation
    ) WHERE source_cursor IS NULL;
INSERT INTO bi.task_event_source_position_claim (
    tenant_id, consumer_id, provider_identity, source_log_id,
    source_log_generation, source_sequence, source_cursor, source_message_id,
    disposition, wire_sha256, claimed_at
)
SELECT source.tenant_id, source.consumer_id, source.provider_identity,
       source.source_log_id, source.source_log_generation, source.source_sequence,
       source.source_cursor, source.source_message_id, 'inbox', inbox.wire_sha256,
       source.recorded_at
  FROM bi.task_event_source_origin AS source
  JOIN bi.task_event_inbox AS inbox
    ON inbox.tenant_id = source.tenant_id
   AND inbox.consumer_id = source.consumer_id
   AND inbox.event_id = source.event_id
UNION ALL
SELECT saved.tenant_id, saved.consumer_id, saved.origin_provider_identity,
       saved.source_log_id, saved.source_log_generation, saved.source_sequence,
       saved.source_cursor, NULLIF(saved.message_id, '00000000-0000-0000-0000-000000000000'::UUID),
       'quarantine', saved.wire_sha256, saved.recorded_at
  FROM bi.task_event_quarantine AS saved;

ALTER TABLE bi.task_event_source_position_claim OWNER TO star_bi_owner;
ALTER TABLE bi.task_event_source_position_claim ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_source_position_claim FORCE ROW LEVEL SECURITY;
CREATE POLICY task_event_source_position_claim_owner_tenant
    ON bi.task_event_source_position_claim
    FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
REVOKE ALL ON bi.task_event_source_position_claim FROM PUBLIC, star_bi_runtime;
GRANT SELECT, INSERT ON bi.task_event_source_position_claim TO star_bi_owner;

CREATE FUNCTION bi.claim_task_event_source_position()
RETURNS TRIGGER
LANGUAGE plpgsql SECURITY DEFINER
-- VOLATILE does not refresh a REPEATABLE READ/SERIALIZABLE transaction snapshot. The advisory-lock
-- visibility proof below therefore accepts claim writes only under READ COMMITTED.
VOLATILE
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    source_consumer VARCHAR(64);
    source_provider VARCHAR(64);
    source_log VARCHAR(128);
    source_generation UUID;
    source_sequence BIGINT;
    source_cursor BYTEA;
    source_message UUID;
    source_digest BYTEA;
    source_disposition VARCHAR(16);
    source_history_lock_key BIGINT;
    inserted_count INTEGER;
    matching_count INTEGER;
    stored_row bi.task_event_source_position_claim%ROWTYPE;
BEGIN
    IF active_tenant IS NULL OR active_tenant IS DISTINCT FROM NEW.tenant_id THEN
        RAISE EXCEPTION 'BI tenant scope mismatch for source-position claim' USING ERRCODE = '42501';
    END IF;

    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'BI source-position claims require READ COMMITTED transaction isolation'
            USING ERRCODE = '0A000';
    END IF;

    IF TG_TABLE_SCHEMA <> 'bi' THEN
        RAISE EXCEPTION 'unexpected BI source-position trigger relation' USING ERRCODE = '23514';
    ELSIF TG_TABLE_NAME = 'task_event_source_origin' THEN
        source_consumer := NEW.consumer_id;
        source_provider := NEW.provider_identity;
        source_log := NEW.source_log_id;
        source_generation := NEW.source_log_generation;
        source_sequence := NEW.source_sequence;
        source_cursor := NEW.source_cursor;
        source_message := NEW.source_message_id;
        source_disposition := 'inbox';

        SELECT inbox.wire_sha256
          INTO source_digest
          FROM bi.task_event_inbox AS inbox
         WHERE inbox.tenant_id = NEW.tenant_id
           AND inbox.consumer_id = NEW.consumer_id
           AND inbox.event_id = NEW.event_id;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'BI Inbox receipt missing for source-origin claim' USING ERRCODE = '23503';
        END IF;
    ELSIF TG_TABLE_NAME = 'task_event_quarantine' THEN
        source_consumer := NEW.consumer_id;
        source_provider := NEW.origin_provider_identity;
        source_log := NEW.source_log_id;
        source_generation := NEW.source_log_generation;
        source_sequence := NEW.source_sequence;
        source_cursor := NEW.source_cursor;
        source_message := NULLIF(NEW.message_id, '00000000-0000-0000-0000-000000000000'::UUID);
        source_digest := NEW.wire_sha256;
        source_disposition := 'quarantine';
    ELSE
        RAISE EXCEPTION 'unexpected BI source-position trigger relation' USING ERRCODE = '23514';
    END IF;

    -- Claims in one source generation must not mix known and unknown cursor modes. Known-cursor
    -- writes share the generation lock and remain parallel; cursorless writes take it exclusively
    -- so either mode's indexed barrier lookup cannot race past a concurrent opposite-mode claim.
    source_history_lock_key := hashtextextended(
        format('bi-task-source-cursor-history/%s/%s/%s/%s/%s', NEW.tenant_id,
            source_consumer, source_provider, source_log, source_generation), 0);
    IF source_cursor IS NULL THEN
        PERFORM pg_advisory_xact_lock(source_history_lock_key);
    ELSE
        PERFORM pg_advisory_xact_lock_shared(source_history_lock_key);
    END IF;

    INSERT INTO bi.task_event_source_position_claim (
        tenant_id, consumer_id, provider_identity, source_log_id,
        source_log_generation, source_sequence, source_cursor, source_message_id,
        disposition, wire_sha256
    ) VALUES (
        NEW.tenant_id, source_consumer, source_provider, source_log, source_generation,
        source_sequence, source_cursor, source_message, source_disposition, source_digest
    )
    ON CONFLICT DO NOTHING;
    GET DIAGNOSTICS inserted_count = ROW_COUNT;

    IF inserted_count = 0 THEN
        SELECT count(*)
          INTO matching_count
          FROM bi.task_event_source_position_claim AS claim
         WHERE claim.tenant_id = NEW.tenant_id
           AND claim.consumer_id = source_consumer
           AND claim.provider_identity = source_provider
           AND claim.source_log_id = source_log
           AND claim.source_log_generation = source_generation
           AND (claim.source_sequence = source_sequence
                OR (source_cursor IS NOT NULL AND claim.source_cursor = source_cursor));

        IF matching_count <> 1 THEN
            RAISE EXCEPTION 'ambiguous BI source-position claim' USING ERRCODE = '23505';
        END IF;

        SELECT claim.*
          INTO stored_row
          FROM bi.task_event_source_position_claim AS claim
         WHERE claim.tenant_id = NEW.tenant_id
           AND claim.consumer_id = source_consumer
           AND claim.provider_identity = source_provider
           AND claim.source_log_id = source_log
           AND claim.source_log_generation = source_generation
           AND (claim.source_sequence = source_sequence
                OR (source_cursor IS NOT NULL AND claim.source_cursor = source_cursor));

        IF NOT FOUND
           OR stored_row.disposition IS DISTINCT FROM source_disposition
           OR stored_row.source_sequence IS DISTINCT FROM source_sequence
           OR stored_row.source_cursor IS DISTINCT FROM source_cursor
           OR stored_row.source_message_id IS DISTINCT FROM source_message
           OR stored_row.wire_sha256 IS DISTINCT FROM source_digest THEN
            RAISE EXCEPTION 'BI source position already has a conflicting terminal disposition'
                USING ERRCODE = '23505';
        END IF;
    ELSIF source_cursor IS NULL AND EXISTS (
        SELECT 1
          FROM bi.task_event_source_position_claim AS known_history
         WHERE known_history.tenant_id = NEW.tenant_id
           AND known_history.consumer_id = source_consumer
           AND known_history.provider_identity = source_provider
           AND known_history.source_log_id = source_log
           AND known_history.source_log_generation = source_generation
           AND known_history.source_cursor IS NOT NULL
    ) THEN
        RAISE EXCEPTION 'BI source generation has known cursor history; use a fresh generation or reconcile history'
            USING ERRCODE = '23505';
    ELSIF source_cursor IS NOT NULL AND EXISTS (
        SELECT 1
          FROM bi.task_event_source_position_claim AS unknown_history
         WHERE unknown_history.tenant_id = NEW.tenant_id
           AND unknown_history.consumer_id = source_consumer
           AND unknown_history.provider_identity = source_provider
           AND unknown_history.source_log_id = source_log
           AND unknown_history.source_log_generation = source_generation
           AND unknown_history.source_cursor IS NULL
    ) THEN
        -- The insert is part of this same owner transaction; raising here rolls it back before the
        -- source-origin/quarantine row exists and before the receiver can authorize a terminal ACK.
        RAISE EXCEPTION 'BI source generation has unknown cursor history; use a fresh generation or reconcile history'
            USING ERRCODE = '23505';
    END IF;

    RETURN NEW;
END;
$$;
ALTER FUNCTION bi.claim_task_event_source_position() OWNER TO star_bi_owner;
REVOKE ALL ON FUNCTION bi.claim_task_event_source_position() FROM PUBLIC, star_bi_runtime;

CREATE FUNCTION bi.reject_task_event_source_position_claim_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog
AS $$
BEGIN
    RAISE EXCEPTION 'BI source-position claims are append-only' USING ERRCODE = '55000';
END;
$$;
ALTER FUNCTION bi.reject_task_event_source_position_claim_mutation() OWNER TO star_bi_owner;
REVOKE ALL ON FUNCTION bi.reject_task_event_source_position_claim_mutation() FROM PUBLIC, star_bi_runtime;

CREATE TRIGGER task_event_source_position_claim_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_source_position_claim
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_source_position_claim_mutation();
CREATE TRIGGER task_event_source_origin_claim_position
    BEFORE INSERT ON bi.task_event_source_origin
    FOR EACH ROW EXECUTE FUNCTION bi.claim_task_event_source_position();
CREATE TRIGGER task_event_quarantine_claim_position
    BEFORE INSERT ON bi.task_event_quarantine
    FOR EACH ROW EXECUTE FUNCTION bi.claim_task_event_source_position();

COMMENT ON TABLE bi.task_event_source_position_claim IS
    'T: immutable owner-local arbitration ledger shared by BI Inbox source-origin and metadata quarantine; stores source identity and wire digest only, never message bytes.';
COMMENT ON FUNCTION bi.claim_task_event_source_position() IS
    'Atomically claims a provider-neutral source sequence and optional opaque cursor from either BI source-origin or quarantine before the terminal receipt is inserted.';

COMMIT;
