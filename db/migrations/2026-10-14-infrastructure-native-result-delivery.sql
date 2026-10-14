-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-14-infrastructure-native-result-delivery.sql"})
-- MERGE (route:Table {id:"infrastructure.capacity_outbox_transport_pin",class:"Transaction"})
-- MERGE (receipt:Table {id:"infrastructure.capacity_native_append_receipt",class:"Transaction"})
-- MERGE (owner:DatabaseRole {id:"star_infrastructure_owner"})
-- MERGE (routeGuard:Function {id:"infrastructure.guard_capacity_outbox_transport_pin",kind:"function"})
-- MERGE (receiptGuard:Function {id:"infrastructure.guard_capacity_native_append_receipt",kind:"function"})
-- MERGE (worm:Function {id:"infrastructure.reject_capacity_owner_fact_mutation",kind:"function"})
-- MERGE (self)-[:DEFINES]->(route)
-- MERGE (self)-[:DEFINES]->(receipt)
-- MERGE (self)-[:CONFIGURES]->(owner)
-- MERGE (self)-[:DEFINES]->(routeGuard)
-- MERGE (self)-[:DEFINES]->(receiptGuard)
-- MERGE (self)-[:USES_TYPE]->(worm)
-- MERGE (routeGuard)-[:GUARDS]->(route)
-- MERGE (receiptGuard)-[:GUARDS]->(receipt)
-- MERGE (worm)-[:GUARDS]->(route)
-- MERGE (worm)-[:GUARDS]->(receipt)
-- MERGE (self)-[:GRANTS]->(owner)
-- @endcypher

-- Immutable Infrastructure-owned route selection and native append receipt. A route is pinned
-- before the first external publish; NATS and native cannot both deliver the same Outbox event.
BEGIN;

CREATE TABLE IF NOT EXISTS infrastructure.capacity_outbox_transport_pin (
    tenant_id UUID NOT NULL,
    event_id UUID NOT NULL,
    transport_provider VARCHAR(24) NOT NULL CHECK (transport_provider IN ('nats-jetstream','native-grpc')),
    profile_id UUID,
    profile_revision BIGINT,
    provider_id VARCHAR(64),
    provider_version VARCHAR(64),
    profile_digest BYTEA,
    endpoint_identity VARCHAR(128),
    sender_service_id UUID,
    recipient_service_id UUID,
    message_kind VARCHAR(16),
    wire_payload BYTEA,
    wire_sha256 BYTEA,
    pinned_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    FOREIGN KEY (tenant_id, event_id)
        REFERENCES infrastructure.capacity_owner_outbox (tenant_id, event_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (transport_provider = 'nats-jetstream'
            AND profile_id IS NULL
            AND profile_revision IS NULL
            AND provider_id IS NULL
            AND provider_version IS NULL
            AND profile_digest IS NULL
            AND endpoint_identity IS NULL
            AND sender_service_id IS NULL
            AND recipient_service_id IS NULL
            AND message_kind IS NULL
            AND wire_payload IS NULL
            AND wire_sha256 IS NULL)
        OR (transport_provider = 'native-grpc'
            AND profile_id IS NOT NULL
            AND profile_revision > 0
            AND provider_id IS NOT NULL
            AND provider_id ~ '^[a-z][a-z0-9-]{0,63}$'
            AND provider_version IS NOT NULL
            AND provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'
            AND octet_length(profile_digest) = 32
            AND endpoint_identity IS NOT NULL
            AND endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
            AND sender_service_id IS NOT NULL
            AND sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID
            AND recipient_service_id IS NOT NULL
            AND recipient_service_id <> '00000000-0000-0000-0000-000000000000'::UUID
            AND sender_service_id <> recipient_service_id
            AND message_kind = 'result'
            AND octet_length(wire_payload) BETWEEN 1 AND 16384
            AND octet_length(wire_sha256) = 32)
    )
);

CREATE TABLE IF NOT EXISTS infrastructure.capacity_native_append_receipt (
    tenant_id UUID NOT NULL,
    event_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    profile_revision BIGINT NOT NULL CHECK (profile_revision > 0),
    provider_id VARCHAR(64) NOT NULL CHECK (provider_id ~ '^[a-z][a-z0-9-]{0,63}$'),
    provider_version VARCHAR(64) NOT NULL
        CHECK (provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'),
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    endpoint_identity VARCHAR(128) NOT NULL
        CHECK (endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    sender_service_id UUID NOT NULL,
    recipient_service_id UUID NOT NULL,
    message_kind VARCHAR(16) NOT NULL CHECK (message_kind = 'result'),
    message_id UUID NOT NULL,
    log_id VARCHAR(128) NOT NULL CHECK (log_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    log_generation UUID NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    exact_replay BOOLEAN NOT NULL,
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    lease_owner_id UUID NOT NULL,
    attempt_count INTEGER NOT NULL CHECK (attempt_count > 0),
    appended_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    FOREIGN KEY (tenant_id, event_id)
        REFERENCES infrastructure.capacity_outbox_transport_pin (tenant_id, event_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (recipient_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> recipient_service_id),
    CHECK (message_id = event_id),
    CHECK (lease_owner_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (log_generation <> '00000000-0000-0000-0000-000000000000'::UUID)
);

ALTER TABLE infrastructure.capacity_outbox_transport_pin ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_outbox_transport_pin FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_outbox_transport_pin_tenant_scope
    ON infrastructure.capacity_outbox_transport_pin;
CREATE POLICY capacity_outbox_transport_pin_tenant_scope
    ON infrastructure.capacity_outbox_transport_pin
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.capacity_native_append_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_native_append_receipt FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_native_append_receipt_tenant_scope
    ON infrastructure.capacity_native_append_receipt;
CREATE POLICY capacity_native_append_receipt_tenant_scope
    ON infrastructure.capacity_native_append_receipt
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

CREATE OR REPLACE FUNCTION infrastructure.guard_capacity_outbox_transport_pin()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    current_state TEXT;
    current_lease_expires_at TIMESTAMPTZ;
BEGIN
    SELECT d.delivery_state, d.lease_expires_at
      INTO current_state, current_lease_expires_at
      FROM infrastructure.capacity_outbox_delivery AS d
     WHERE d.tenant_id = NEW.tenant_id
       AND d.event_id = NEW.event_id
     FOR UPDATE;
    IF current_state IS DISTINCT FROM 'leased'
       OR current_lease_expires_at IS NULL
       OR current_lease_expires_at <= clock_timestamp() THEN
        RAISE EXCEPTION 'Infrastructure transport route requires a current delivery lease';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.guard_capacity_native_append_receipt()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    delivery_state_value TEXT;
    lease_owner_value UUID;
    attempt_value INTEGER;
    lease_expires_value TIMESTAMPTZ;
    route_provider_value VARCHAR(24);
    route_profile_id_value UUID;
    route_profile_revision_value BIGINT;
    route_provider_id_value VARCHAR(64);
    route_provider_version_value VARCHAR(64);
    route_profile_digest_value BYTEA;
    route_endpoint_identity_value VARCHAR(128);
    route_sender_service_id_value UUID;
    route_recipient_service_id_value UUID;
    route_message_kind_value VARCHAR(16);
    route_wire_sha256_value BYTEA;
BEGIN
    SELECT d.delivery_state, d.lease_owner_id, d.attempt_count, d.lease_expires_at,
           p.transport_provider, p.profile_id, p.profile_revision, p.provider_id,
           p.provider_version, p.profile_digest, p.endpoint_identity,
           p.sender_service_id, p.recipient_service_id, p.message_kind, p.wire_sha256
      INTO delivery_state_value, lease_owner_value, attempt_value, lease_expires_value,
           route_provider_value, route_profile_id_value, route_profile_revision_value,
           route_provider_id_value, route_provider_version_value, route_profile_digest_value,
           route_endpoint_identity_value, route_sender_service_id_value,
           route_recipient_service_id_value, route_message_kind_value, route_wire_sha256_value
      FROM infrastructure.capacity_outbox_delivery AS d
      JOIN infrastructure.capacity_outbox_transport_pin AS p
        ON p.tenant_id = d.tenant_id AND p.event_id = d.event_id
     WHERE d.tenant_id = NEW.tenant_id AND d.event_id = NEW.event_id
     FOR UPDATE OF d;
    IF delivery_state_value IS DISTINCT FROM 'leased'
       OR lease_owner_value IS NULL
       OR attempt_value IS NULL
       OR lease_expires_value IS NULL
       OR lease_expires_value <= clock_timestamp()
       OR lease_owner_value IS DISTINCT FROM NEW.lease_owner_id
       OR attempt_value IS DISTINCT FROM NEW.attempt_count
       OR route_provider_value IS DISTINCT FROM 'native-grpc'
       OR route_profile_id_value IS DISTINCT FROM NEW.profile_id
       OR route_profile_revision_value IS DISTINCT FROM NEW.profile_revision
       OR route_provider_id_value IS DISTINCT FROM NEW.provider_id
       OR route_provider_version_value IS DISTINCT FROM NEW.provider_version
       OR route_profile_digest_value IS DISTINCT FROM NEW.profile_digest
       OR route_endpoint_identity_value IS DISTINCT FROM NEW.endpoint_identity
       OR route_sender_service_id_value IS DISTINCT FROM NEW.sender_service_id
       OR route_recipient_service_id_value IS DISTINCT FROM NEW.recipient_service_id
       OR route_message_kind_value IS DISTINCT FROM NEW.message_kind
       OR route_wire_sha256_value IS DISTINCT FROM NEW.wire_sha256
       OR NEW.message_id IS DISTINCT FROM NEW.event_id THEN
        RAISE EXCEPTION 'Infrastructure native append receipt does not match the current route and lease fence';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS capacity_outbox_transport_pin_insert_guard
    ON infrastructure.capacity_outbox_transport_pin;
CREATE TRIGGER capacity_outbox_transport_pin_insert_guard
    BEFORE INSERT ON infrastructure.capacity_outbox_transport_pin
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_outbox_transport_pin();

DROP TRIGGER IF EXISTS capacity_native_append_receipt_insert_guard
    ON infrastructure.capacity_native_append_receipt;
CREATE TRIGGER capacity_native_append_receipt_insert_guard
    BEFORE INSERT ON infrastructure.capacity_native_append_receipt
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_native_append_receipt();

DROP TRIGGER IF EXISTS capacity_outbox_transport_pin_worm
    ON infrastructure.capacity_outbox_transport_pin;
CREATE TRIGGER capacity_outbox_transport_pin_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON infrastructure.capacity_outbox_transport_pin
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();

DROP TRIGGER IF EXISTS capacity_native_append_receipt_worm
    ON infrastructure.capacity_native_append_receipt;
CREATE TRIGGER capacity_native_append_receipt_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON infrastructure.capacity_native_append_receipt
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();

REVOKE ALL ON infrastructure.capacity_outbox_transport_pin FROM PUBLIC;
REVOKE ALL ON infrastructure.capacity_native_append_receipt FROM PUBLIC;
GRANT SELECT, INSERT ON infrastructure.capacity_outbox_transport_pin TO star_infrastructure_owner;
GRANT SELECT, INSERT ON infrastructure.capacity_native_append_receipt TO star_infrastructure_owner;

COMMENT ON TABLE infrastructure.capacity_outbox_transport_pin IS
    'Transaction: immutable first-selected transport route and exact native result wire; prevents provider double delivery.';
COMMENT ON TABLE infrastructure.capacity_native_append_receipt IS
    'Transaction: immutable Infrastructure-owned proof of exact native append acceptance; not Run Inbox commit or execution readiness.';
COMMENT ON COLUMN infrastructure.capacity_outbox_transport_pin.wire_payload IS
    'Exact bounded native result wire persisted at route pin so retries reuse identical bytes.';

COMMIT;
