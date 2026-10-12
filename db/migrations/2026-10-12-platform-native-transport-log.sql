-- @cypher schema=1 source_sha256=c838de348a509f182dec63ff8ebd3a0ed2defb5840d6335620ab65d9abefd0da
-- MERGE (self:File {path:"db/migrations/2026-10-12-platform-native-transport-log.sql"})
-- MERGE (log_master:Table {id:"platform_transport.log_master"})
-- MERGE (log_state:Table {id:"platform_transport.log_state"})
-- MERGE (message_work:Table {id:"platform_transport.message_work"})
-- MERGE (receipt:Table {id:"platform_transport.delivery_receipt"})
-- MERGE (outbox:Table {id:"platform_transport.receipt_outbox"})
-- MERGE (append:Function {id:"platform_transport.append_message"})
-- MERGE (pull:Function {id:"platform_transport.pull_one"})
-- MERGE (ack:Function {id:"platform_transport.acknowledge_one"})
-- MERGE (purge:Function {id:"platform_transport.purge_terminal"})
-- MERGE (emit_outbox:Function {id:"platform_transport.append_receipt_outbox"})
-- MERGE (advisory_lock:ExternalService {id:"postgres.pg_advisory_xact_lock",kind:"database function"})
-- MERGE (self)-[:CONFIGURES]->(log_master)
-- MERGE (self)-[:CONFIGURES]->(log_state)
-- MERGE (self)-[:CONFIGURES]->(message_work)
-- MERGE (self)-[:CONFIGURES]->(receipt)
-- MERGE (self)-[:CONFIGURES]->(outbox)
-- MERGE (append)-[:READS]->(log_master)
-- MERGE (append)-[:CALLS]->(advisory_lock)
-- MERGE (append)-[:WRITES]->(log_state)
-- MERGE (append)-[:WRITES]->(message_work)
-- MERGE (append)-[:WRITES]->(receipt)
-- MERGE (pull)-[:READS]->(log_master)
-- MERGE (pull)-[:READS]->(message_work)
-- MERGE (pull)-[:WRITES]->(message_work)
-- MERGE (ack)-[:READS]->(message_work)
-- MERGE (ack)-[:WRITES]->(message_work)
-- MERGE (ack)-[:WRITES]->(receipt)
-- MERGE (receipt)-[:EMITS]->(emit_outbox)
-- MERGE (emit_outbox)-[:WRITES]->(outbox)
-- MERGE (purge)-[:READS]->(log_master)
-- MERGE (purge)-[:WRITES]->(message_work)
-- MERGE (purge)-[:WRITES]->(log_state)
-- @endcypher
-- D18 schema foundation for the future Star-owned Rust transport service.
-- This stores transport facts only; it does not replace JetStream or authorize consumers.
BEGIN;

DO $roles$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_transport_owner') THEN
        CREATE ROLE star_platform_transport_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_transport_runtime') THEN
        CREATE ROLE star_platform_transport_runtime NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
    END IF;
END
$roles$;
DO $role_guard$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_roles
         WHERE rolname IN ('star_platform_transport_owner', 'star_platform_transport_runtime')
           AND (rolcanlogin OR rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls OR rolinherit)
    ) THEN
        RAISE EXCEPTION 'platform transport roles must remain restricted NOLOGIN roles';
    END IF;
END
$role_guard$;

CREATE SCHEMA IF NOT EXISTS platform_transport;
REVOKE ALL ON SCHEMA platform_transport FROM PUBLIC;
GRANT USAGE ON SCHEMA platform_transport TO star_platform_transport_owner, star_platform_transport_runtime;

CREATE TABLE IF NOT EXISTS platform_transport.log_master (
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    log_id TEXT NOT NULL CHECK (log_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    generation UUID NOT NULL CHECK (generation <> '00000000-0000-0000-0000-000000000000'),
    sender_service_id UUID NOT NULL CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'),
    recipient_service_id UUID NOT NULL CHECK (recipient_service_id <> '00000000-0000-0000-0000-000000000000'),
    message_kind TEXT NOT NULL CHECK (message_kind ~ '^[a-z][a-z0-9._-]{0,63}$'),
    payload_budget_bytes BIGINT NOT NULL CHECK (payload_budget_bytes BETWEEN 16384 AND 1073741824),
    terminal_retention_hours INTEGER NOT NULL CHECK (terminal_retention_hours BETWEEN 24 AND 8760),
    max_delivery_attempts SMALLINT NOT NULL CHECK (max_delivery_attempts BETWEEN 1 AND 12),
    lifecycle TEXT NOT NULL CHECK (lifecycle IN ('active', 'draining', 'disabled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (deployment_id, tenant_id, log_id, generation),
    UNIQUE (deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id, message_kind)
);

CREATE TABLE IF NOT EXISTS platform_transport.log_state (
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    log_id TEXT NOT NULL,
    generation UUID NOT NULL,
    next_sequence BIGINT NOT NULL DEFAULT 1 CHECK (next_sequence > 0),
    payload_bytes_in_use BIGINT NOT NULL DEFAULT 0 CHECK (payload_bytes_in_use >= 0),
    PRIMARY KEY (deployment_id, tenant_id, log_id, generation),
    FOREIGN KEY (deployment_id, tenant_id, log_id, generation)
        REFERENCES platform_transport.log_master (deployment_id, tenant_id, log_id, generation)
);

CREATE TABLE IF NOT EXISTS platform_transport.message_work (
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    log_id TEXT NOT NULL,
    generation UUID NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    sender_service_id UUID NOT NULL,
    recipient_service_id UUID NOT NULL,
    message_kind TEXT NOT NULL,
    message_id UUID NOT NULL CHECK (message_id <> '00000000-0000-0000-0000-000000000000'),
    wire BYTEA NOT NULL CHECK (octet_length(wire) BETWEEN 1 AND 16384),
    payload_sha256 BYTEA NOT NULL CHECK (octet_length(payload_sha256) = 32),
    state TEXT NOT NULL CHECK (state IN ('pending', 'leased', 'acked')),
    attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 12),
    lease_id UUID,
    lease_expires_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    acked_at TIMESTAMPTZ,
    PRIMARY KEY (deployment_id, tenant_id, log_id, generation, sequence),
    UNIQUE (deployment_id, tenant_id, sender_service_id, recipient_service_id, message_kind, message_id),
    FOREIGN KEY (deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id, message_kind)
        REFERENCES platform_transport.log_master
            (deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id, message_kind),
    CHECK (
        (state = 'leased' AND lease_id IS NOT NULL AND lease_expires_at IS NOT NULL AND acked_at IS NULL)
        OR (state = 'pending' AND lease_id IS NULL AND lease_expires_at IS NULL AND acked_at IS NULL)
        OR (state = 'acked' AND lease_id IS NULL AND lease_expires_at IS NULL AND acked_at IS NOT NULL)
    )
);
CREATE INDEX IF NOT EXISTS ix_platform_transport_message_pull
    ON platform_transport.message_work (deployment_id, tenant_id, log_id, generation, sequence)
    WHERE state IN ('pending', 'leased');
CREATE INDEX IF NOT EXISTS ix_platform_transport_message_terminal_retention
    ON platform_transport.message_work (deployment_id, tenant_id, log_id, generation, acked_at)
    WHERE state = 'acked';

CREATE TABLE IF NOT EXISTS platform_transport.delivery_receipt (
    receipt_id UUID PRIMARY KEY,
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    log_id TEXT NOT NULL,
    generation UUID NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    sender_service_id UUID NOT NULL,
    recipient_service_id UUID NOT NULL,
    message_kind TEXT NOT NULL,
    message_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('accepted', 'acked')),
    payload_sha256 BYTEA NOT NULL CHECK (octet_length(payload_sha256) = 32),
    lease_id UUID,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (deployment_id, tenant_id, sender_service_id, recipient_service_id, message_kind, message_id, action),
    UNIQUE (deployment_id, tenant_id, log_id, generation, sequence, action),
    CHECK ((action = 'accepted' AND lease_id IS NULL) OR (action = 'acked' AND lease_id IS NOT NULL))
);
CREATE TABLE IF NOT EXISTS platform_transport.receipt_outbox (
    outbox_id UUID PRIMARY KEY,
    receipt_id UUID NOT NULL UNIQUE REFERENCES platform_transport.delivery_receipt (receipt_id),
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    event_schema_version SMALLINT NOT NULL CHECK (event_schema_version = 1),
    event_kind TEXT NOT NULL CHECK (event_kind IN ('message.accepted', 'message.acked')),
    log_id TEXT NOT NULL,
    generation UUID NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    message_kind TEXT NOT NULL,
    message_id UUID NOT NULL,
    payload_sha256 BYTEA NOT NULL CHECK (octet_length(payload_sha256) = 32),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (deployment_id, tenant_id, log_id, generation, sequence, event_kind)
);
CREATE INDEX IF NOT EXISTS ix_platform_transport_receipt_bi
    ON platform_transport.delivery_receipt (deployment_id, tenant_id, occurred_at, action);
CREATE INDEX IF NOT EXISTS ix_platform_transport_outbox_page
    ON platform_transport.receipt_outbox (deployment_id, tenant_id, occurred_at, outbox_id);

CREATE OR REPLACE FUNCTION platform_transport.seed_log_state()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
BEGIN
    INSERT INTO platform_transport.log_state (deployment_id, tenant_id, log_id, generation)
    VALUES (NEW.deployment_id, NEW.tenant_id, NEW.log_id, NEW.generation)
    ON CONFLICT DO NOTHING;
    RETURN NEW;
END
$function$;
CREATE OR REPLACE FUNCTION platform_transport.guard_log_master()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
AS $function$
BEGIN
    IF TG_OP = 'UPDATE'
       AND NEW.deployment_id = OLD.deployment_id
       AND NEW.tenant_id = OLD.tenant_id
       AND NEW.log_id = OLD.log_id
       AND NEW.generation = OLD.generation
       AND NEW.sender_service_id = OLD.sender_service_id
       AND NEW.recipient_service_id = OLD.recipient_service_id
       AND NEW.message_kind = OLD.message_kind
       AND NEW.payload_budget_bytes = OLD.payload_budget_bytes
       AND NEW.terminal_retention_hours = OLD.terminal_retention_hours
       AND NEW.max_delivery_attempts = OLD.max_delivery_attempts
       AND NEW.created_at = OLD.created_at
       AND ((OLD.lifecycle = 'active' AND NEW.lifecycle = 'draining')
         OR (OLD.lifecycle = 'draining' AND NEW.lifecycle = 'disabled')) THEN
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'platform transport log identity/configuration is immutable' USING ERRCODE = '55000';
END
$function$;
CREATE OR REPLACE FUNCTION platform_transport.reject_log_master_truncate()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
AS $function$
BEGIN
    RAISE EXCEPTION 'platform transport log Master is append-only' USING ERRCODE = '55000';
END
$function$;
DROP TRIGGER IF EXISTS platform_transport_seed_log_state ON platform_transport.log_master;
CREATE TRIGGER platform_transport_seed_log_state
    AFTER INSERT ON platform_transport.log_master
    FOR EACH ROW EXECUTE FUNCTION platform_transport.seed_log_state();
DROP TRIGGER IF EXISTS platform_transport_log_master_update_guard ON platform_transport.log_master;
CREATE TRIGGER platform_transport_log_master_update_guard
    BEFORE UPDATE OR DELETE ON platform_transport.log_master
    FOR EACH ROW EXECUTE FUNCTION platform_transport.guard_log_master();
DROP TRIGGER IF EXISTS platform_transport_log_master_truncate_guard ON platform_transport.log_master;
CREATE TRIGGER platform_transport_log_master_truncate_guard
    BEFORE TRUNCATE ON platform_transport.log_master
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_log_master_truncate();

CREATE OR REPLACE FUNCTION platform_transport.reject_receipt_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
AS $function$
BEGIN
    RAISE EXCEPTION 'platform transport delivery receipts are append-only' USING ERRCODE = '55000';
END
$function$;
CREATE OR REPLACE FUNCTION platform_transport.append_receipt_outbox()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
BEGIN
    INSERT INTO platform_transport.receipt_outbox (
        outbox_id, receipt_id, deployment_id, tenant_id, event_schema_version,
        event_kind, log_id, generation, sequence, message_kind, message_id, payload_sha256, occurred_at
    ) VALUES (
        gen_random_uuid(), NEW.receipt_id, NEW.deployment_id, NEW.tenant_id, 1,
        CASE NEW.action WHEN 'accepted' THEN 'message.accepted' ELSE 'message.acked' END,
        NEW.log_id, NEW.generation, NEW.sequence, NEW.message_kind, NEW.message_id,
        NEW.payload_sha256, NEW.occurred_at
    );
    RETURN NEW;
END
$function$;
DROP TRIGGER IF EXISTS platform_transport_receipt_no_update_delete ON platform_transport.delivery_receipt;
CREATE TRIGGER platform_transport_receipt_no_update_delete
    BEFORE UPDATE OR DELETE ON platform_transport.delivery_receipt
    FOR EACH ROW EXECUTE FUNCTION platform_transport.reject_receipt_mutation();
DROP TRIGGER IF EXISTS platform_transport_receipt_no_truncate ON platform_transport.delivery_receipt;
CREATE TRIGGER platform_transport_receipt_no_truncate
    BEFORE TRUNCATE ON platform_transport.delivery_receipt
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_receipt_mutation();
DROP TRIGGER IF EXISTS platform_transport_receipt_outbox ON platform_transport.delivery_receipt;
CREATE TRIGGER platform_transport_receipt_outbox
    AFTER INSERT ON platform_transport.delivery_receipt
    FOR EACH ROW EXECUTE FUNCTION platform_transport.append_receipt_outbox();
DROP TRIGGER IF EXISTS platform_transport_outbox_no_update_delete ON platform_transport.receipt_outbox;
CREATE TRIGGER platform_transport_outbox_no_update_delete
    BEFORE UPDATE OR DELETE ON platform_transport.receipt_outbox
    FOR EACH ROW EXECUTE FUNCTION platform_transport.reject_receipt_mutation();
DROP TRIGGER IF EXISTS platform_transport_outbox_no_truncate ON platform_transport.receipt_outbox;
CREATE TRIGGER platform_transport_outbox_no_truncate
    BEFORE TRUNCATE ON platform_transport.receipt_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_receipt_mutation();

CREATE OR REPLACE FUNCTION platform_transport.append_message(
    p_log_id TEXT,
    p_generation UUID,
    p_recipient_service_id UUID,
    p_message_kind TEXT,
    p_message_id UUID,
    p_wire BYTEA,
    p_payload_sha256 BYTEA
)
RETURNS TABLE (accepted_log_id TEXT, accepted_generation UUID, accepted_sequence BIGINT, accepted_sha256 BYTEA, exact_replay BOOLEAN)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_sender UUID := NULLIF(current_setting('app.platform_service_id', true), '')::UUID;
    v_receipt platform_transport.delivery_receipt%ROWTYPE;
    v_master platform_transport.log_master%ROWTYPE;
    v_sequence BIGINT;
    v_bytes BIGINT;
BEGIN
    IF v_deployment IS NULL OR v_tenant IS NULL OR v_sender IS NULL
       OR p_log_id IS NULL OR p_generation IS NULL OR p_recipient_service_id IS NULL
       OR p_message_kind IS NULL OR p_message_id IS NULL OR p_message_id = '00000000-0000-0000-0000-000000000000'
       OR p_wire IS NULL OR octet_length(p_wire) NOT BETWEEN 1 AND 16384
       OR p_payload_sha256 IS NULL OR octet_length(p_payload_sha256) <> 32 THEN
        RAISE EXCEPTION 'invalid platform transport append boundary' USING ERRCODE = '22023';
    END IF;

    -- Serialize exact producer identities, including attempts pinned to different log generations.
    PERFORM pg_advisory_xact_lock(hashtextextended(
        v_deployment::TEXT || ':' || v_tenant::TEXT || ':' || v_sender::TEXT || ':' ||
        p_recipient_service_id::TEXT || ':' || p_message_kind || ':' || p_message_id::TEXT,
        0
    ));

    SELECT r.* INTO v_receipt
      FROM platform_transport.delivery_receipt r
     WHERE r.deployment_id = v_deployment AND r.tenant_id = v_tenant
       AND r.sender_service_id = v_sender AND r.recipient_service_id = p_recipient_service_id
       AND r.message_kind = p_message_kind AND r.message_id = p_message_id AND r.action = 'accepted';
    IF FOUND THEN
        IF v_receipt.payload_sha256 <> p_payload_sha256
           OR v_receipt.log_id <> p_log_id OR v_receipt.generation <> p_generation THEN
            RAISE EXCEPTION 'platform transport exact replay conflict' USING ERRCODE = '22000';
        END IF;
        RETURN QUERY SELECT v_receipt.log_id, v_receipt.generation, v_receipt.sequence, v_receipt.payload_sha256, TRUE;
        RETURN;
    END IF;

    SELECT m.* INTO v_master
      FROM platform_transport.log_master m
     WHERE m.deployment_id = v_deployment AND m.tenant_id = v_tenant
       AND m.log_id = p_log_id AND m.generation = p_generation
       AND m.sender_service_id = v_sender AND m.recipient_service_id = p_recipient_service_id
       AND m.message_kind = p_message_kind AND m.lifecycle = 'active';
    IF NOT FOUND THEN
        RAISE EXCEPTION 'platform transport log is not active for this authenticated route' USING ERRCODE = '42501';
    END IF;

    SELECT s.next_sequence, s.payload_bytes_in_use INTO v_sequence, v_bytes
      FROM platform_transport.log_state s
     WHERE s.deployment_id = v_deployment AND s.tenant_id = v_tenant
       AND s.log_id = p_log_id AND s.generation = p_generation
     FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'platform transport log state is missing' USING ERRCODE = '55000';
    END IF;
    IF v_bytes + octet_length(p_wire) > v_master.payload_budget_bytes THEN
        RAISE EXCEPTION 'platform transport log byte budget exhausted' USING ERRCODE = '54000';
    END IF;

    UPDATE platform_transport.log_state s
       SET next_sequence = s.next_sequence + 1,
           payload_bytes_in_use = s.payload_bytes_in_use + octet_length(p_wire)
     WHERE s.deployment_id = v_deployment AND s.tenant_id = v_tenant
       AND s.log_id = p_log_id AND s.generation = p_generation;

    INSERT INTO platform_transport.message_work (
        deployment_id, tenant_id, log_id, generation, sequence, sender_service_id,
        recipient_service_id, message_kind, message_id, wire, payload_sha256, state
    ) VALUES (
        v_deployment, v_tenant, p_log_id, p_generation, v_sequence, v_sender,
        p_recipient_service_id, p_message_kind, p_message_id, p_wire, p_payload_sha256, 'pending'
    );
    INSERT INTO platform_transport.delivery_receipt (
        receipt_id, deployment_id, tenant_id, log_id, generation, sequence,
        sender_service_id, recipient_service_id, message_kind, message_id, action, payload_sha256
    ) VALUES (
        gen_random_uuid(), v_deployment, v_tenant, p_log_id, p_generation, v_sequence,
        v_sender, p_recipient_service_id, p_message_kind, p_message_id, 'accepted', p_payload_sha256
    );
    RETURN QUERY SELECT p_log_id, p_generation, v_sequence, p_payload_sha256, FALSE;
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.pull_one(
    p_log_id TEXT,
    p_generation UUID,
    p_lease_seconds INTEGER
)
RETURNS TABLE (
    delivery_log_id TEXT, delivery_generation UUID, delivery_sequence BIGINT,
    sender_service_id UUID, message_kind TEXT, message_id UUID, wire BYTEA,
    payload_sha256 BYTEA, lease_id UUID, lease_expires_at TIMESTAMPTZ, attempt_count SMALLINT
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_service UUID := NULLIF(current_setting('app.platform_service_id', true), '')::UUID;
    v_row platform_transport.message_work%ROWTYPE;
    v_max_attempts SMALLINT;
    v_lease UUID;
    v_until TIMESTAMPTZ;
BEGIN
    IF v_deployment IS NULL OR v_tenant IS NULL OR v_service IS NULL
       OR p_log_id IS NULL OR p_generation IS NULL OR p_lease_seconds IS NULL OR p_lease_seconds NOT BETWEEN 30 AND 120 THEN
        RAISE EXCEPTION 'invalid platform transport pull boundary' USING ERRCODE = '22023';
    END IF;
    SELECT m.max_delivery_attempts INTO v_max_attempts
      FROM platform_transport.log_master m
     WHERE m.deployment_id = v_deployment AND m.tenant_id = v_tenant
       AND m.log_id = p_log_id AND m.generation = p_generation
       AND m.recipient_service_id = v_service AND m.lifecycle IN ('active', 'draining');
    IF NOT FOUND THEN
        RAISE EXCEPTION 'platform transport log is not active for this authenticated recipient' USING ERRCODE = '42501';
    END IF;

    SELECT w.* INTO v_row
      FROM platform_transport.message_work w
     WHERE w.deployment_id = v_deployment AND w.tenant_id = v_tenant
       AND w.log_id = p_log_id AND w.generation = p_generation
       AND w.recipient_service_id = v_service
       AND w.attempt_count < v_max_attempts
       AND (w.state = 'pending' OR (w.state = 'leased' AND w.lease_expires_at <= clock_timestamp()))
     ORDER BY w.sequence
     LIMIT 1
     FOR UPDATE SKIP LOCKED;
    IF NOT FOUND THEN
        RETURN;
    END IF;
    v_lease := gen_random_uuid();
    v_until := clock_timestamp() + make_interval(secs => p_lease_seconds);
    UPDATE platform_transport.message_work w
       SET state = 'leased', attempt_count = w.attempt_count + 1,
           lease_id = v_lease, lease_expires_at = v_until
     WHERE w.deployment_id = v_row.deployment_id AND w.tenant_id = v_row.tenant_id
       AND w.log_id = v_row.log_id AND w.generation = v_row.generation AND w.sequence = v_row.sequence
     RETURNING w.* INTO v_row;
    RETURN QUERY SELECT v_row.log_id, v_row.generation, v_row.sequence,
        v_row.sender_service_id, v_row.message_kind, v_row.message_id, v_row.wire,
        v_row.payload_sha256, v_row.lease_id, v_row.lease_expires_at, v_row.attempt_count;
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.acknowledge_one(
    p_log_id TEXT,
    p_generation UUID,
    p_sequence BIGINT,
    p_lease_id UUID,
    p_message_id UUID,
    p_payload_sha256 BYTEA
)
RETURNS BOOLEAN
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_service UUID := NULLIF(current_setting('app.platform_service_id', true), '')::UUID;
    v_row platform_transport.message_work%ROWTYPE;
BEGIN
    IF v_deployment IS NULL OR v_tenant IS NULL OR v_service IS NULL
       OR p_log_id IS NULL OR p_generation IS NULL OR p_sequence IS NULL OR p_sequence <= 0
       OR p_lease_id IS NULL OR p_message_id IS NULL OR p_payload_sha256 IS NULL
       OR octet_length(p_payload_sha256) <> 32 THEN
        RAISE EXCEPTION 'invalid platform transport acknowledgement boundary' USING ERRCODE = '22023';
    END IF;
    IF EXISTS (
        SELECT 1 FROM platform_transport.delivery_receipt r
         WHERE r.deployment_id = v_deployment AND r.tenant_id = v_tenant
           AND r.log_id = p_log_id AND r.generation = p_generation AND r.sequence = p_sequence
           AND r.recipient_service_id = v_service AND r.message_id = p_message_id
           AND r.action = 'acked' AND r.payload_sha256 = p_payload_sha256
    ) THEN
        RETURN TRUE;
    END IF;
    SELECT w.* INTO v_row
      FROM platform_transport.message_work w
     WHERE w.deployment_id = v_deployment AND w.tenant_id = v_tenant
       AND w.log_id = p_log_id AND w.generation = p_generation AND w.sequence = p_sequence
       AND w.recipient_service_id = v_service AND w.message_id = p_message_id
     FOR UPDATE;
    IF NOT FOUND OR v_row.state <> 'leased' OR v_row.lease_id <> p_lease_id
       OR v_row.lease_expires_at <= clock_timestamp() OR v_row.payload_sha256 <> p_payload_sha256 THEN
        RAISE EXCEPTION 'stale or mismatched platform transport acknowledgement' USING ERRCODE = '40001';
    END IF;
    UPDATE platform_transport.message_work w
       SET state = 'acked', lease_id = NULL, lease_expires_at = NULL, acked_at = clock_timestamp()
     WHERE w.deployment_id = v_deployment AND w.tenant_id = v_tenant
       AND w.log_id = p_log_id AND w.generation = p_generation AND w.sequence = p_sequence;
    INSERT INTO platform_transport.delivery_receipt (
        receipt_id, deployment_id, tenant_id, log_id, generation, sequence,
        sender_service_id, recipient_service_id, message_kind, message_id, action, payload_sha256, lease_id
    ) VALUES (
        gen_random_uuid(), v_deployment, v_tenant, p_log_id, p_generation, p_sequence,
        v_row.sender_service_id, v_row.recipient_service_id, v_row.message_kind,
        v_row.message_id, 'acked', v_row.payload_sha256, p_lease_id
    );
    RETURN TRUE;
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.purge_terminal(
    p_log_id TEXT,
    p_generation UUID,
    p_limit INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_service UUID := NULLIF(current_setting('app.platform_service_id', true), '')::UUID;
    v_retention INTEGER;
    v_total BIGINT;
    v_deleted INTEGER;
BEGIN
    IF v_deployment IS NULL OR v_tenant IS NULL OR v_service IS NULL
       OR p_log_id IS NULL OR p_generation IS NULL OR p_limit IS NULL OR p_limit NOT BETWEEN 1 AND 100 THEN
        RAISE EXCEPTION 'invalid platform transport purge boundary' USING ERRCODE = '22023';
    END IF;
    SELECT m.terminal_retention_hours INTO v_retention
      FROM platform_transport.log_master m
     WHERE m.deployment_id = v_deployment AND m.tenant_id = v_tenant
       AND m.log_id = p_log_id AND m.generation = p_generation
       AND m.recipient_service_id = v_service AND m.lifecycle IN ('active', 'draining', 'disabled');
    IF NOT FOUND THEN
        RAISE EXCEPTION 'platform transport log purge scope denied' USING ERRCODE = '42501';
    END IF;
    PERFORM 1 FROM platform_transport.log_state s
     WHERE s.deployment_id = v_deployment AND s.tenant_id = v_tenant
       AND s.log_id = p_log_id AND s.generation = p_generation
     FOR UPDATE;
    WITH terminal AS MATERIALIZED (
        SELECT w.deployment_id, w.tenant_id, w.log_id, w.generation, w.sequence, octet_length(w.wire)::BIGINT AS bytes
          FROM platform_transport.message_work w
         WHERE w.deployment_id = v_deployment AND w.tenant_id = v_tenant
           AND w.log_id = p_log_id AND w.generation = p_generation AND w.state = 'acked'
           AND w.acked_at <= clock_timestamp() - make_interval(hours => v_retention)
         ORDER BY w.sequence LIMIT p_limit FOR UPDATE SKIP LOCKED
    ), deleted AS (
        DELETE FROM platform_transport.message_work w USING terminal t
         WHERE w.deployment_id = t.deployment_id AND w.tenant_id = t.tenant_id
           AND w.log_id = t.log_id AND w.generation = t.generation AND w.sequence = t.sequence
        RETURNING octet_length(w.wire)::BIGINT AS bytes
    ) SELECT count(*)::INTEGER, coalesce(sum(bytes), 0)::BIGINT INTO v_deleted, v_total FROM deleted;
    UPDATE platform_transport.log_state s SET payload_bytes_in_use = s.payload_bytes_in_use - v_total
     WHERE s.deployment_id = v_deployment AND s.tenant_id = v_tenant
       AND s.log_id = p_log_id AND s.generation = p_generation;
    RETURN v_deleted;
END
$function$;

ALTER TABLE platform_transport.log_master ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.log_master FORCE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.log_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.log_state FORCE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.message_work ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.message_work FORCE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.delivery_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.delivery_receipt FORCE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.receipt_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.receipt_outbox FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS transport_log_master_scope ON platform_transport.log_master;
CREATE POLICY transport_log_master_scope ON platform_transport.log_master
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS transport_log_state_scope ON platform_transport.log_state;
CREATE POLICY transport_log_state_scope ON platform_transport.log_state
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS transport_message_work_scope ON platform_transport.message_work;
CREATE POLICY transport_message_work_scope ON platform_transport.message_work
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS transport_receipt_scope ON platform_transport.delivery_receipt;
CREATE POLICY transport_receipt_scope ON platform_transport.delivery_receipt
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS transport_receipt_outbox_scope ON platform_transport.receipt_outbox;
CREATE POLICY transport_receipt_outbox_scope ON platform_transport.receipt_outbox
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
       AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);

ALTER TABLE platform_transport.log_master OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.log_state OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.message_work OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.delivery_receipt OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.receipt_outbox OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.seed_log_state() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.guard_log_master() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.reject_log_master_truncate() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.reject_receipt_mutation() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.append_receipt_outbox() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.append_message(TEXT, UUID, UUID, TEXT, UUID, BYTEA, BYTEA) OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER) OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.acknowledge_one(TEXT, UUID, BIGINT, UUID, UUID, BYTEA) OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.purge_terminal(TEXT, UUID, INTEGER) OWNER TO star_platform_transport_owner;

GRANT SELECT, INSERT, UPDATE ON platform_transport.log_master TO star_platform_transport_owner;
GRANT SELECT, INSERT, UPDATE ON platform_transport.log_state TO star_platform_transport_owner;
GRANT SELECT, INSERT, UPDATE, DELETE ON platform_transport.message_work TO star_platform_transport_owner;
GRANT SELECT, INSERT ON platform_transport.delivery_receipt TO star_platform_transport_owner;
GRANT SELECT, INSERT ON platform_transport.receipt_outbox TO star_platform_transport_owner;
REVOKE ALL ON ALL TABLES IN SCHEMA platform_transport FROM PUBLIC, star_platform_transport_runtime;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA platform_transport FROM PUBLIC;
GRANT EXECUTE ON FUNCTION platform_transport.append_message(TEXT, UUID, UUID, TEXT, UUID, BYTEA, BYTEA) TO star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER) TO star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.acknowledge_one(TEXT, UUID, BIGINT, UUID, UUID, BYTEA) TO star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.purge_terminal(TEXT, UUID, INTEGER) TO star_platform_transport_runtime;

COMMIT;
