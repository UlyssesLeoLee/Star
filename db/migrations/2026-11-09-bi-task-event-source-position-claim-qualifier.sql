-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-11-09-bi-task-event-source-position-claim-qualifier.sql"})
-- MERGE (claim:Database {id:"bi.claim_task_event_source_position",kind:"trigger_function"})
-- MERGE (claimLedger:Table {id:"bi.task_event_source_position_claim",class:"Transaction"})
-- MERGE (inboxTrigger:Database {id:"bi.task_event_source_origin",kind:"table"})
-- MERGE (quarantineTrigger:Database {id:"bi.task_event_quarantine",kind:"table"})
-- MERGE (self)-[:DEFINES]->(claim)
-- MERGE (self)-[:REPLACES]->(claim)
-- MERGE (claim)-[:READS]->(claimLedger)
-- MERGE (claim)-[:WRITES]->(claimLedger)
-- MERGE (claim)-[:TRIGGERS_ON]->(inboxTrigger)
-- MERGE (claim)-[:TRIGGERS_ON]->(quarantineTrigger)
-- @endcypher

-- Forward-only remediation for bi.claim_task_event_source_position().
--
-- The 2026-10-25 shared-position-claim migration declares two PL/pgSQL locals named
-- "source_sequence" and "source_cursor" that are also column names of
-- bi.task_event_source_position_claim. On the exact-replay branch the trigger compares
-- a stored claim column against the bare local name:
--
--     AND (claim.source_sequence = source_sequence
--          OR (source_cursor IS NOT NULL AND claim.source_cursor = source_cursor))
--
-- PostgreSQL resolves a bare name that matches both a PL/pgSQL variable and a table
-- column as ambiguous and raises 42702 ("column reference ... is not unique"), so the
-- whole quarantine exact-replay path aborts. The first insert of a new source position
-- never reaches that comparison, which is why the defect only surfaced on replay.
--
-- This migration recreates the trigger function with the two locals renamed to a
-- distinct v_ prefix and every comparison qualified against the local variable.
-- The arbitration contract is unchanged: same source identity, same disposition
-- conflict detection, same high-isolation narrow replay path, same fail-closed
-- behaviour on any partial or ambiguous match.
BEGIN;

CREATE OR REPLACE FUNCTION bi.claim_task_event_source_position()
RETURNS TRIGGER
LANGUAGE plpgsql SECURITY DEFINER
-- VOLATILE does not refresh a REPEATABLE READ/SERIALIZABLE transaction snapshot. The advisory-lock
-- visibility proof below therefore accepts new claim writes only under READ COMMITTED. A narrow
-- quarantine-only exception verifies a unique, fully matching receipt and claim already visible
-- in the fixed snapshot before suppressing the duplicate receipt INSERT.
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
    v_source_sequence BIGINT;
    v_source_cursor BYTEA;
    source_message UUID;
    source_digest BYTEA;
    source_disposition VARCHAR(16);
    source_history_lock_key BIGINT;
    inserted_count INTEGER;
    matching_count INTEGER;
    matching_receipts BIGINT;
    exact_receipts BIGINT;
    matching_claims BIGINT;
    exact_claims BIGINT;
    stored_row bi.task_event_source_position_claim%ROWTYPE;
BEGIN
    IF active_tenant IS NULL OR active_tenant IS DISTINCT FROM NEW.tenant_id THEN
        RAISE EXCEPTION 'BI tenant scope mismatch for source-position claim' USING ERRCODE = '42501';
    END IF;

    IF TG_TABLE_SCHEMA <> 'bi' THEN
        RAISE EXCEPTION 'unexpected BI source-position trigger relation' USING ERRCODE = '23514';
    ELSIF TG_TABLE_NAME = 'task_event_source_origin' THEN
        source_consumer := NEW.consumer_id;
        source_provider := NEW.provider_identity;
        source_log := NEW.source_log_id;
        source_generation := NEW.source_log_generation;
        v_source_sequence := NEW.source_sequence;
        v_source_cursor := NEW.source_cursor;
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
        v_source_sequence := NEW.source_sequence;
        v_source_cursor := NEW.source_cursor;
        source_message := NULLIF(NEW.message_id, '00000000-0000-0000-0000-000000000000'::UUID);
        source_digest := NEW.wire_sha256;
        source_disposition := 'quarantine';
    ELSE
        RAISE EXCEPTION 'unexpected BI source-position trigger relation' USING ERRCODE = '23514';
    END IF;

    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        IF TG_TABLE_NAME <> 'task_event_quarantine' THEN
            RAISE EXCEPTION 'BI source-position claims require READ COMMITTED transaction isolation'
                USING ERRCODE = '0A000';
        END IF;

        -- Under a fixed transaction snapshot, only a complete quarantine replay already visible
        -- in that snapshot may pass. Suppress the duplicate row with RETURN NULL so the outer
        -- writer can use its existing exact-replay comparison to return the original ID/timestamp;
        -- this trigger must not create a second claim or make an unmatched claim visible. Count
        -- every same-position candidate so
        -- sequence/cursor disagreement, duplicate history, or a partial mismatch fails closed.
        SELECT count(*), count(*) FILTER (WHERE
                   saved.sender_service_id IS NOT DISTINCT FROM NEW.sender_service_id
               AND saved.receiver_service_id IS NOT DISTINCT FROM NEW.receiver_service_id
               AND saved.transport_provider_id IS NOT DISTINCT FROM NEW.transport_provider_id
               AND saved.transport_provider_version IS NOT DISTINCT FROM NEW.transport_provider_version
               AND saved.transport_profile_id IS NOT DISTINCT FROM NEW.transport_profile_id
               AND saved.transport_profile_revision IS NOT DISTINCT FROM NEW.transport_profile_revision
               AND saved.transport_profile_digest IS NOT DISTINCT FROM NEW.transport_profile_digest
               AND saved.endpoint_identity IS NOT DISTINCT FROM NEW.endpoint_identity
               AND saved.destination_id IS NOT DISTINCT FROM NEW.destination_id
               AND saved.destination_generation IS NOT DISTINCT FROM NEW.destination_generation
               AND saved.source_sequence IS NOT DISTINCT FROM NEW.source_sequence
               AND saved.source_cursor IS NOT DISTINCT FROM NEW.source_cursor
               AND saved.message_id IS NOT DISTINCT FROM NEW.message_id
               AND saved.wire_sha256 IS NOT DISTINCT FROM NEW.wire_sha256
               AND saved.wire_length IS NOT DISTINCT FROM NEW.wire_length
               AND saved.fault_code IS NOT DISTINCT FROM NEW.fault_code)
          INTO matching_receipts, exact_receipts
          FROM bi.task_event_quarantine AS saved
         WHERE saved.tenant_id = NEW.tenant_id
           AND saved.consumer_id = NEW.consumer_id
           AND saved.origin_provider_identity = NEW.origin_provider_identity
           AND saved.source_log_id = NEW.source_log_id
           AND saved.source_log_generation = NEW.source_log_generation
           AND (saved.source_sequence = NEW.source_sequence
                OR (NEW.source_cursor IS NOT NULL AND saved.source_cursor = NEW.source_cursor));

        IF matching_receipts = 0 THEN
            RAISE EXCEPTION 'BI source-position claims require READ COMMITTED transaction isolation'
                USING ERRCODE = '0A000';
        END IF;
        IF matching_receipts <> 1 OR exact_receipts <> 1 THEN
            RAISE EXCEPTION 'BI high-isolation quarantine replay does not match one durable receipt and claim'
                USING ERRCODE = '23505';
        END IF;

        SELECT count(*), count(*) FILTER (WHERE
                   claim.disposition IS NOT DISTINCT FROM 'quarantine'
               AND claim.source_sequence IS NOT DISTINCT FROM NEW.source_sequence
               AND claim.source_cursor IS NOT DISTINCT FROM NEW.source_cursor
               AND claim.source_message_id IS NOT DISTINCT FROM
                   NULLIF(NEW.message_id, '00000000-0000-0000-0000-000000000000'::UUID)
               AND claim.wire_sha256 IS NOT DISTINCT FROM NEW.wire_sha256)
          INTO matching_claims, exact_claims
          FROM bi.task_event_source_position_claim AS claim
         WHERE claim.tenant_id = NEW.tenant_id
           AND claim.consumer_id = NEW.consumer_id
           AND claim.provider_identity = NEW.origin_provider_identity
           AND claim.source_log_id = NEW.source_log_id
           AND claim.source_log_generation = NEW.source_log_generation
           AND (claim.source_sequence = NEW.source_sequence
                OR (NEW.source_cursor IS NOT NULL AND claim.source_cursor = NEW.source_cursor));

        IF matching_claims <> 1 OR exact_claims <> 1 THEN
            RAISE EXCEPTION 'BI high-isolation quarantine replay does not match one durable receipt and claim'
                USING ERRCODE = '23505';
        END IF;

        RETURN NULL;
    END IF;

    -- Claims in one source generation must not mix known and unknown cursor modes. Known-cursor
    -- writes share the generation lock and remain parallel; cursorless writes take it exclusively
    -- so either mode's indexed barrier lookup cannot race past a concurrent opposite-mode claim.
    source_history_lock_key := hashtextextended(
        format('bi-task-source-cursor-history/%s/%s/%s/%s/%s', NEW.tenant_id,
            source_consumer, source_provider, source_log, source_generation), 0);
    IF v_source_cursor IS NULL THEN
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
        v_source_sequence, v_source_cursor, source_message, source_disposition, source_digest
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
           AND (claim.source_sequence = v_source_sequence
                OR (v_source_cursor IS NOT NULL AND claim.source_cursor = v_source_cursor));

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
           AND (claim.source_sequence = v_source_sequence
                OR (v_source_cursor IS NOT NULL AND claim.source_cursor = v_source_cursor));

        IF NOT FOUND
           OR stored_row.disposition IS DISTINCT FROM source_disposition
           OR stored_row.source_sequence IS DISTINCT FROM v_source_sequence
           OR stored_row.source_cursor IS DISTINCT FROM v_source_cursor
           OR stored_row.source_message_id IS DISTINCT FROM source_message
           OR stored_row.wire_sha256 IS DISTINCT FROM source_digest THEN
            RAISE EXCEPTION 'BI source position already has a conflicting terminal disposition'
                USING ERRCODE = '23505';
        END IF;
    ELSIF v_source_cursor IS NULL AND EXISTS (
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
    ELSIF v_source_cursor IS NOT NULL AND EXISTS (
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

COMMENT ON FUNCTION bi.claim_task_event_source_position() IS
    'Atomically claims a provider-neutral source sequence and optional opaque cursor from either BI source-origin or quarantine before the terminal receipt is inserted. Recreated 2026-11-09 with disambiguated PL/pgSQL locals so the exact-replay comparison no longer raises 42702.';

COMMIT;
