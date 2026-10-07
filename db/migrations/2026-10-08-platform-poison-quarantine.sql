-- @cypher schema=1 source_sha256=03767896281401a361d87b07b56810d04769207024b0dd354c746adcf587c61a
-- MERGE (self:File {path:"db/migrations/2026-10-08-platform-poison-quarantine.sql"})
-- MERGE (schema:Database {id:"platform_messaging"})
-- MERGE (ingress:DatabaseRole {id:"star_platform_ingress_owner"})
-- MERGE (purger:DatabaseRole {id:"star_platform_quarantine_purger"})
-- MERGE (receipt:Table {id:"platform_messaging.poison_quarantine_receipt"})
-- MERGE (payload:Table {id:"platform_messaging.poison_quarantine_payload_work"})
-- MERGE (audit:Table {id:"platform_messaging.poison_quarantine_payload_purge_audit"})
-- MERGE (worm:Symbol {id:"db/migrations/2026-10-08-platform-poison-quarantine.sql::reject_quarantine_transaction_mutation",kind:"function"})
-- MERGE (encrypted_guard:Symbol {id:"db/migrations/2026-10-08-platform-poison-quarantine.sql::require_encrypted_quarantine_receipt",kind:"function"})
-- MERGE (purge_guard:Symbol {id:"db/migrations/2026-10-08-platform-poison-quarantine.sql::validate_quarantine_purge_record",kind:"function"})
-- MERGE (quota_guard:Symbol {id:"db/migrations/2026-10-08-platform-poison-quarantine.sql::enforce_quarantine_payload_quota",kind:"function"})
-- MERGE (self)-[:CONFIGURES]->(schema)
-- MERGE (self)-[:CONFIGURES]->(ingress)
-- MERGE (self)-[:CONFIGURES]->(purger)
-- MERGE (self)-[:CONFIGURES]->(receipt)
-- MERGE (self)-[:CONFIGURES]->(payload)
-- MERGE (self)-[:CONFIGURES]->(audit)
-- MERGE (payload)-[:REFERENCES]->(receipt)
-- MERGE (audit)-[:REFERENCES]->(receipt)
-- MERGE (receipt)-[:CALLS]->(worm)
-- MERGE (audit)-[:CALLS]->(worm)
-- MERGE (payload)-[:CALLS]->(encrypted_guard)
-- MERGE (payload)-[:CALLS]->(quota_guard)
-- MERGE (audit)-[:CALLS]->(purge_guard)
-- @endcypher

-- Platform Messaging owner facts only. Source identity is derived from trusted deployment config
-- plus the JetStream ACK reply metadata; event tenant and message headers never set the RLS scope.
-- W/T/M: receipt and purge audit are append-only Transaction facts; encrypted body is bounded Work
-- with a fixed 24-hour TTL; no Master table is introduced because policy is explicit deployment
-- configuration and there is intentionally no default retention policy.
BEGIN;

CREATE SCHEMA IF NOT EXISTS platform_messaging;
REVOKE ALL ON SCHEMA platform_messaging FROM PUBLIC;
REVOKE ALL ON ALL TABLES IN SCHEMA platform_messaging FROM PUBLIC;

CREATE TABLE IF NOT EXISTS platform_messaging.poison_quarantine_receipt (
    deployment_id UUID NOT NULL,
    stream_generation UUID NOT NULL,
    stream_name VARCHAR(255) NOT NULL,
    stream_sequence BIGINT NOT NULL,
    fault_code VARCHAR(64) NOT NULL CHECK (fault_code IN (
        'invalid_capacity_result_event',
        'malformed_capacity_result_json',
        'capacity_result_event_id_conflict'
    )),
    classifier_version SMALLINT NOT NULL CHECK (classifier_version = 1),
    parser_version SMALLINT NOT NULL CHECK (parser_version > 0),
    payload_bytes INTEGER NOT NULL CHECK (payload_bytes BETWEEN 0 AND 135168),
    payload_sha256 BYTEA NOT NULL CHECK (octet_length(payload_sha256) = 32),
    retention_policy VARCHAR(32) NOT NULL CHECK (retention_policy IN ('metadata_only','encrypted_24h')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (deployment_id, stream_generation, stream_name, stream_sequence),
    CHECK (deployment_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (stream_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (length(stream_name) BETWEEN 1 AND 255),
    CHECK (stream_sequence > 0)
);

CREATE TABLE IF NOT EXISTS platform_messaging.poison_quarantine_payload_work (
    deployment_id UUID NOT NULL,
    stream_generation UUID NOT NULL,
    stream_name VARCHAR(255) NOT NULL,
    stream_sequence BIGINT NOT NULL,
    encrypted_blob BYTEA NOT NULL CHECK (octet_length(encrypted_blob) BETWEEN 1 AND 204800),
    encrypted_blob_sha256 BYTEA NOT NULL CHECK (octet_length(encrypted_blob_sha256) = 32),
    encoded_bytes INTEGER NOT NULL CHECK (encoded_bytes BETWEEN 1 AND 204800)
        CHECK (encoded_bytes = octet_length(encrypted_blob)),
    retained_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (deployment_id, stream_generation, stream_name, stream_sequence),
    FOREIGN KEY (deployment_id, stream_generation, stream_name, stream_sequence)
        REFERENCES platform_messaging.poison_quarantine_receipt
        (deployment_id, stream_generation, stream_name, stream_sequence)
        ON DELETE RESTRICT,
    CHECK (deployment_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (stream_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (length(stream_name) BETWEEN 1 AND 255),
    CHECK (stream_sequence > 0),
    CHECK (expires_at = retained_at + interval '24 hours')
);
CREATE INDEX IF NOT EXISTS idx_poison_quarantine_payload_expiry
    ON platform_messaging.poison_quarantine_payload_work (deployment_id, expires_at, stream_sequence);

CREATE TABLE IF NOT EXISTS platform_messaging.poison_quarantine_payload_purge_audit (
    deployment_id UUID NOT NULL,
    stream_generation UUID NOT NULL,
    stream_name VARCHAR(255) NOT NULL,
    stream_sequence BIGINT NOT NULL,
    encrypted_blob_sha256 BYTEA NOT NULL CHECK (octet_length(encrypted_blob_sha256) = 32),
    encoded_bytes INTEGER NOT NULL CHECK (encoded_bytes BETWEEN 1 AND 204800),
    purge_reason VARCHAR(32) NOT NULL CHECK (purge_reason = 'ttl_expired'),
    purged_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (deployment_id, stream_generation, stream_name, stream_sequence),
    FOREIGN KEY (deployment_id, stream_generation, stream_name, stream_sequence)
        REFERENCES platform_messaging.poison_quarantine_receipt
        (deployment_id, stream_generation, stream_name, stream_sequence)
        ON DELETE RESTRICT,
    CHECK (deployment_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (stream_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (length(stream_name) BETWEEN 1 AND 255),
    CHECK (stream_sequence > 0)
);

CREATE OR REPLACE FUNCTION platform_messaging.reject_quarantine_transaction_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Platform poison quarantine Transaction facts are append-only'
        USING ERRCODE = '55000';
END;
$$;

CREATE OR REPLACE FUNCTION platform_messaging.require_encrypted_quarantine_receipt()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM platform_messaging.poison_quarantine_receipt AS receipt
        WHERE receipt.deployment_id = NEW.deployment_id
          AND receipt.stream_generation = NEW.stream_generation
          AND receipt.stream_name = NEW.stream_name
          AND receipt.stream_sequence = NEW.stream_sequence
          AND receipt.retention_policy = 'encrypted_24h'
    ) THEN
        RAISE EXCEPTION 'encrypted quarantine Work requires an encrypted-retention receipt'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION platform_messaging.validate_quarantine_purge_record()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM platform_messaging.poison_quarantine_payload_work AS payload
        WHERE payload.deployment_id = NEW.deployment_id
          AND payload.stream_generation = NEW.stream_generation
          AND payload.stream_name = NEW.stream_name
          AND payload.stream_sequence = NEW.stream_sequence
          AND payload.encrypted_blob_sha256 = NEW.encrypted_blob_sha256
          AND payload.encoded_bytes = NEW.encoded_bytes
          AND payload.expires_at <= clock_timestamp()
    ) THEN
        RAISE EXCEPTION 'quarantine purge audit must match an expired encrypted Work row'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

-- Keep this 4 MiB per-deployment ceiling synchronized with the Rust constant. The trigger is
-- the final guard even if an application caller bypasses its preflight quota calculation.
CREATE OR REPLACE FUNCTION platform_messaging.enforce_quarantine_payload_quota()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    current_bytes BIGINT;
BEGIN
    PERFORM pg_advisory_xact_lock(
        hashtext('star.platform_messaging.quarantine'),
        hashtext(NEW.deployment_id::TEXT)
    );
    SELECT COALESCE(SUM(encoded_bytes), 0)::BIGINT
      INTO current_bytes
      FROM platform_messaging.poison_quarantine_payload_work
     WHERE deployment_id = NEW.deployment_id;
    IF current_bytes + NEW.encoded_bytes > 4194304 THEN
        RAISE EXCEPTION 'encrypted quarantine payload quota exceeded for deployment'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS poison_quarantine_payload_requires_encrypted_receipt
    ON platform_messaging.poison_quarantine_payload_work;
CREATE TRIGGER poison_quarantine_payload_requires_encrypted_receipt
    BEFORE INSERT ON platform_messaging.poison_quarantine_payload_work
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.require_encrypted_quarantine_receipt();
DROP TRIGGER IF EXISTS poison_quarantine_payload_quota_guard
    ON platform_messaging.poison_quarantine_payload_work;
CREATE TRIGGER poison_quarantine_payload_quota_guard
    BEFORE INSERT ON platform_messaging.poison_quarantine_payload_work
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.enforce_quarantine_payload_quota();

DROP TRIGGER IF EXISTS poison_quarantine_payload_purge_matches_expired_work
    ON platform_messaging.poison_quarantine_payload_purge_audit;
CREATE TRIGGER poison_quarantine_payload_purge_matches_expired_work
    BEFORE INSERT ON platform_messaging.poison_quarantine_payload_purge_audit
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.validate_quarantine_purge_record();

DROP TRIGGER IF EXISTS poison_quarantine_receipt_worm
    ON platform_messaging.poison_quarantine_receipt;
CREATE TRIGGER poison_quarantine_receipt_worm
    BEFORE UPDATE OR DELETE ON platform_messaging.poison_quarantine_receipt
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.reject_quarantine_transaction_mutation();
DROP TRIGGER IF EXISTS poison_quarantine_receipt_truncate_guard
    ON platform_messaging.poison_quarantine_receipt;
CREATE TRIGGER poison_quarantine_receipt_truncate_guard
    BEFORE TRUNCATE ON platform_messaging.poison_quarantine_receipt
    FOR EACH STATEMENT EXECUTE FUNCTION platform_messaging.reject_quarantine_transaction_mutation();
DROP TRIGGER IF EXISTS poison_quarantine_payload_audit_worm
    ON platform_messaging.poison_quarantine_payload_purge_audit;
CREATE TRIGGER poison_quarantine_payload_audit_worm
    BEFORE UPDATE OR DELETE ON platform_messaging.poison_quarantine_payload_purge_audit
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.reject_quarantine_transaction_mutation();
DROP TRIGGER IF EXISTS poison_quarantine_payload_audit_truncate_guard
    ON platform_messaging.poison_quarantine_payload_purge_audit;
CREATE TRIGGER poison_quarantine_payload_audit_truncate_guard
    BEFORE TRUNCATE ON platform_messaging.poison_quarantine_payload_purge_audit
    FOR EACH STATEMENT EXECUTE FUNCTION platform_messaging.reject_quarantine_transaction_mutation();

ALTER TABLE platform_messaging.poison_quarantine_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.poison_quarantine_receipt FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS poison_quarantine_receipt_deployment_scope
    ON platform_messaging.poison_quarantine_receipt;
CREATE POLICY poison_quarantine_receipt_deployment_scope
    ON platform_messaging.poison_quarantine_receipt
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID);

ALTER TABLE platform_messaging.poison_quarantine_payload_work ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.poison_quarantine_payload_work FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS poison_quarantine_payload_deployment_scope
    ON platform_messaging.poison_quarantine_payload_work;
CREATE POLICY poison_quarantine_payload_deployment_scope
    ON platform_messaging.poison_quarantine_payload_work
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID);

ALTER TABLE platform_messaging.poison_quarantine_payload_purge_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.poison_quarantine_payload_purge_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS poison_quarantine_payload_audit_deployment_scope
    ON platform_messaging.poison_quarantine_payload_purge_audit;
CREATE POLICY poison_quarantine_payload_audit_deployment_scope
    ON platform_messaging.poison_quarantine_payload_purge_audit
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID);

DO $roles$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_ingress_owner') THEN
        EXECUTE 'ALTER ROLE star_platform_ingress_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_platform_ingress_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_quarantine_purger') THEN
        EXECUTE 'ALTER ROLE star_platform_quarantine_purger NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_platform_quarantine_purger NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
END
$roles$;

REVOKE ALL ON SCHEMA platform_messaging FROM star_platform_ingress_owner, star_platform_quarantine_purger;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA platform_messaging
    FROM star_platform_ingress_owner, star_platform_quarantine_purger;
GRANT USAGE ON SCHEMA platform_messaging TO star_platform_ingress_owner, star_platform_quarantine_purger;
GRANT SELECT, INSERT ON platform_messaging.poison_quarantine_receipt,
    platform_messaging.poison_quarantine_payload_work
TO star_platform_ingress_owner;
GRANT SELECT ON platform_messaging.poison_quarantine_receipt,
    platform_messaging.poison_quarantine_payload_work
TO star_platform_quarantine_purger;
GRANT DELETE ON platform_messaging.poison_quarantine_payload_work
TO star_platform_quarantine_purger;
GRANT INSERT ON platform_messaging.poison_quarantine_payload_purge_audit
TO star_platform_quarantine_purger;

REVOKE ALL ON FUNCTION platform_messaging.reject_quarantine_transaction_mutation() FROM PUBLIC;
REVOKE ALL ON FUNCTION platform_messaging.require_encrypted_quarantine_receipt() FROM PUBLIC;
REVOKE ALL ON FUNCTION platform_messaging.validate_quarantine_purge_record() FROM PUBLIC;
REVOKE ALL ON FUNCTION platform_messaging.enforce_quarantine_payload_quota() FROM PUBLIC;
COMMENT ON TABLE platform_messaging.poison_quarantine_receipt IS
    'Platform Messaging append-only Transaction receipt; source identity uses deployment, stream generation/name, and server stream sequence.';
COMMENT ON TABLE platform_messaging.poison_quarantine_payload_work IS
    'Optional encrypted Work payload with 24-hour TTL, per-item bound, deployment quota, and separate audited purger role.';
COMMENT ON TABLE platform_messaging.poison_quarantine_payload_purge_audit IS
    'Append-only Transaction evidence for TTL deletion of encrypted quarantine Work payloads.';
COMMENT ON COLUMN platform_messaging.poison_quarantine_receipt.retention_policy IS
    'Explicit deployment policy; startup must fail closed if no policy is configured.';

COMMIT;
