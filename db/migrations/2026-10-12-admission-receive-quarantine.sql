-- @cypher schema=1 source_sha256=a102c3536067090e6bbaa35e2916bb13a9f09a82f614199065a0cc2c8125773f
-- MERGE (self:File {path:"db/migrations/2026-10-12-admission-receive-quarantine.sql"})
-- MERGE (schema:Database {id:"platform_messaging"})
-- MERGE (ingress:DatabaseRole {id:"star_platform_ingress_owner"})
-- MERGE (receipt:Table {id:"platform_messaging.admission_receive_quarantine_receipt"})
-- MERGE (self)-[:CONFIGURES]->(schema)
-- MERGE (self)-[:CONFIGURES]->(ingress)
-- MERGE (self)-[:CONFIGURES]->(receipt)
-- MERGE (receipt_worm:Symbol {id:"db/migrations/2026-10-12-admission-receive-quarantine.sql::admission_receive_quarantine_worm",kind:"trigger"})
-- MERGE (receipt_truncate_guard:Symbol {id:"db/migrations/2026-10-12-admission-receive-quarantine.sql::admission_receive_quarantine_truncate_guard",kind:"trigger"})
-- MERGE (worm:Symbol {id:"db/migrations/2026-10-08-platform-poison-quarantine.sql::reject_quarantine_transaction_mutation",kind:"function"})
-- MERGE (receipt)-[:CALLS]->(worm)
-- MERGE (self)-[:CREATES]->(receipt_worm)
-- MERGE (self)-[:CREATES]->(receipt_truncate_guard)
-- MERGE (receipt_worm)-[:CALLS]->(worm)
-- MERGE (receipt_truncate_guard)-[:CALLS]->(worm)
-- @endcypher

-- Provider-neutral, metadata-only poison receipts. The source wire remains in the transport until
-- terminal disposition; only a bounded digest and immutable authenticated route evidence are stored.
-- W/T/M: one append-only Transaction fact table; no payload Work or policy Master table is added.
BEGIN;

CREATE TABLE IF NOT EXISTS platform_messaging.admission_receive_quarantine_receipt (
    deployment_id UUID NOT NULL,
    provider_identity VARCHAR(128) NOT NULL,
    log_generation UUID NOT NULL,
    log_id VARCHAR(128) NOT NULL,
    sequence BIGINT NOT NULL,
    tenant_id UUID NOT NULL,
    sender_service_id UUID NOT NULL,
    recipient_service_id UUID NOT NULL,
    message_kind VARCHAR(16) NOT NULL CHECK (message_kind IN ('request','result','resolution')),
    message_id UUID NOT NULL,
    fault_code VARCHAR(64) NOT NULL CHECK (fault_code IN (
        'malformed_admission_wire',
        'admission_identity_mismatch'
    )),
    classifier_version SMALLINT NOT NULL CHECK (classifier_version = 1),
    parser_version SMALLINT NOT NULL CHECK (parser_version = 1),
    wire_bytes INTEGER NOT NULL CHECK (wire_bytes BETWEEN 0 AND 20480),
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (deployment_id, provider_identity, log_generation, log_id, sequence),
    CHECK (deployment_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (provider_identity ~ '^[A-Za-z0-9._:/-]{1,128}$'),
    CHECK (log_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (log_id ~ '^[A-Za-z0-9._:/-]{1,128}$'),
    CHECK (sequence > 0),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (recipient_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> recipient_service_id),
    CHECK (message_id <> '00000000-0000-0000-0000-000000000000'::UUID
        OR fault_code = 'malformed_admission_wire')
);

DROP TRIGGER IF EXISTS admission_receive_quarantine_worm
    ON platform_messaging.admission_receive_quarantine_receipt;
CREATE TRIGGER admission_receive_quarantine_worm
    BEFORE UPDATE OR DELETE ON platform_messaging.admission_receive_quarantine_receipt
    FOR EACH ROW EXECUTE FUNCTION platform_messaging.reject_quarantine_transaction_mutation();
DROP TRIGGER IF EXISTS admission_receive_quarantine_truncate_guard
    ON platform_messaging.admission_receive_quarantine_receipt;
CREATE TRIGGER admission_receive_quarantine_truncate_guard
    BEFORE TRUNCATE ON platform_messaging.admission_receive_quarantine_receipt
    FOR EACH STATEMENT EXECUTE FUNCTION platform_messaging.reject_quarantine_transaction_mutation();

ALTER TABLE platform_messaging.admission_receive_quarantine_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_messaging.admission_receive_quarantine_receipt FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS admission_receive_quarantine_deployment_tenant_scope
    ON platform_messaging.admission_receive_quarantine_receipt;
CREATE POLICY admission_receive_quarantine_deployment_tenant_scope
    ON platform_messaging.admission_receive_quarantine_receipt
    USING (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_quarantine_tenant_id', true), '')::UUID
    )
    WITH CHECK (
        deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_quarantine_tenant_id', true), '')::UUID
    );

REVOKE ALL ON platform_messaging.admission_receive_quarantine_receipt FROM PUBLIC;
GRANT USAGE ON SCHEMA platform_messaging TO star_platform_ingress_owner;
GRANT SELECT, INSERT ON platform_messaging.admission_receive_quarantine_receipt
    TO star_platform_ingress_owner;
COMMENT ON TABLE platform_messaging.admission_receive_quarantine_receipt IS
    'Append-only provider-neutral Transaction receipt; source key is deployment/provider/log-generation/log/cursor and RLS additionally scopes tenant.';
COMMENT ON COLUMN platform_messaging.admission_receive_quarantine_receipt.wire_sha256 IS
    'SHA-256 of the exact source wire; the body is not retained in this metadata-only admission quarantine.';

COMMIT;
