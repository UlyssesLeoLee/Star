-- @cypher schema=1 source_sha256=67afb3e9b042e107b73d2f4334d20e50784dd166b09537bd363534ff4deb43b5
-- MERGE (self:File {path:"db/migrations/2026-10-14-infrastructure-native-route-pin-quarantine.sql"})
-- MERGE (quarantine:Table {id:"infrastructure.capacity_native_route_pin_quarantine",class:"Transaction"})
-- MERGE (owner:DatabaseRole {id:"star_infrastructure_owner"})
-- MERGE (guard:Function {id:"infrastructure.guard_capacity_native_route_pin_quarantine",kind:"function"})
-- MERGE (worm:Function {id:"infrastructure.reject_capacity_owner_fact_mutation",kind:"function"})
-- MERGE (route:Table {id:"infrastructure.capacity_outbox_transport_pin",class:"Transaction"})
-- MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery",class:"Work"})
-- MERGE (receipt:Table {id:"infrastructure.capacity_native_append_receipt",class:"Transaction"})
-- MERGE (self)-[:DEFINES]->(quarantine)
-- MERGE (self)-[:CONFIGURES]->(owner)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:USES_TYPE]->(worm)
-- MERGE (self)-[:READS]->(route)
-- MERGE (self)-[:READS]->(delivery)
-- MERGE (self)-[:READS]->(receipt)
-- MERGE (guard)-[:GUARDS]->(quarantine)
-- MERGE (guard)-[:READS]->(route)
-- MERGE (guard)-[:READS]->(delivery)
-- MERGE (guard)-[:READS]->(receipt)
-- MERGE (worm)-[:GUARDS]->(quarantine)
-- MERGE (self)-[:GRANTS]->(owner)
-- @endcypher

-- Transaction: immutable, tenant-scoped evidence that the configured native route differs from
-- the route already pinned to a delivery. It stores no payload, URL, credential, or transport error.
BEGIN;

CREATE TABLE IF NOT EXISTS infrastructure.capacity_native_route_pin_quarantine (
    tenant_id UUID NOT NULL,
    event_id UUID NOT NULL,
    observed_profile_id UUID NOT NULL,
    observed_profile_revision BIGINT NOT NULL CHECK (observed_profile_revision > 0),
    observed_provider_id VARCHAR(64) NOT NULL CHECK (observed_provider_id ~ '^[a-z][a-z0-9-]{0,63}$'),
    observed_provider_version VARCHAR(64) NOT NULL
        CHECK (observed_provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'),
    observed_profile_digest BYTEA NOT NULL CHECK (octet_length(observed_profile_digest) = 32),
    observed_endpoint_identity VARCHAR(128) NOT NULL
        CHECK (observed_endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    observed_sender_service_id UUID NOT NULL,
    observed_recipient_service_id UUID NOT NULL,
    mismatch_fields TEXT[] NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    FOREIGN KEY (tenant_id, event_id)
        REFERENCES infrastructure.capacity_outbox_transport_pin (tenant_id, event_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (observed_profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (observed_sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (observed_recipient_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (observed_sender_service_id <> observed_recipient_service_id),
    CHECK (cardinality(mismatch_fields) BETWEEN 1 AND 8),
    CHECK (mismatch_fields <@ ARRAY[
        'profile_id','profile_revision','provider_id','provider_version',
        'profile_digest','endpoint_identity','sender_service_id','recipient_service_id'
    ]::TEXT[])
);

CREATE INDEX IF NOT EXISTS idx_capacity_native_route_pin_quarantine_page
    ON infrastructure.capacity_native_route_pin_quarantine (tenant_id, detected_at, event_id);

ALTER TABLE infrastructure.capacity_native_route_pin_quarantine ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_native_route_pin_quarantine FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_native_route_pin_quarantine_tenant_scope
    ON infrastructure.capacity_native_route_pin_quarantine;
CREATE POLICY capacity_native_route_pin_quarantine_tenant_scope
    ON infrastructure.capacity_native_route_pin_quarantine
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

CREATE OR REPLACE FUNCTION infrastructure.guard_capacity_native_route_pin_quarantine()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    delivery_state_value TEXT;
    lease_expires_value TIMESTAMPTZ;
    route_provider_value VARCHAR(24);
    expected_mismatches TEXT[];
BEGIN
    SELECT d.delivery_state, d.lease_expires_at
      INTO delivery_state_value, lease_expires_value
      FROM infrastructure.capacity_outbox_delivery AS d
     WHERE d.tenant_id = NEW.tenant_id AND d.event_id = NEW.event_id
     FOR UPDATE;
    IF delivery_state_value IS NULL
       OR delivery_state_value = 'delivered'
       OR (delivery_state_value = 'leased' AND lease_expires_value > clock_timestamp())
       OR EXISTS (
            SELECT 1
              FROM infrastructure.capacity_native_append_receipt AS receipt
             WHERE receipt.tenant_id = NEW.tenant_id AND receipt.event_id = NEW.event_id
       ) THEN
        RAISE EXCEPTION 'Infrastructure native route quarantine cannot override an active lease or receipt';
    END IF;

    SELECT route.transport_provider,
           ARRAY_REMOVE(ARRAY[
               CASE WHEN route.profile_id IS DISTINCT FROM NEW.observed_profile_id THEN 'profile_id' END,
               CASE WHEN route.profile_revision IS DISTINCT FROM NEW.observed_profile_revision THEN 'profile_revision' END,
               CASE WHEN route.provider_id IS DISTINCT FROM NEW.observed_provider_id THEN 'provider_id' END,
               CASE WHEN route.provider_version IS DISTINCT FROM NEW.observed_provider_version THEN 'provider_version' END,
               CASE WHEN route.profile_digest IS DISTINCT FROM NEW.observed_profile_digest THEN 'profile_digest' END,
               CASE WHEN route.endpoint_identity IS DISTINCT FROM NEW.observed_endpoint_identity THEN 'endpoint_identity' END,
               CASE WHEN route.sender_service_id IS DISTINCT FROM NEW.observed_sender_service_id THEN 'sender_service_id' END,
               CASE WHEN route.recipient_service_id IS DISTINCT FROM NEW.observed_recipient_service_id THEN 'recipient_service_id' END
           ]::TEXT[], NULL::TEXT)
      INTO route_provider_value, expected_mismatches
      FROM infrastructure.capacity_outbox_transport_pin AS route
     WHERE route.tenant_id = NEW.tenant_id AND route.event_id = NEW.event_id;

    IF route_provider_value IS DISTINCT FROM 'native-grpc'
       OR cardinality(expected_mismatches) = 0
       OR NEW.mismatch_fields IS DISTINCT FROM expected_mismatches THEN
        RAISE EXCEPTION 'Infrastructure native route quarantine must describe the exact immutable pin mismatch';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS capacity_native_route_pin_quarantine_insert_guard
    ON infrastructure.capacity_native_route_pin_quarantine;
CREATE TRIGGER capacity_native_route_pin_quarantine_insert_guard
    BEFORE INSERT ON infrastructure.capacity_native_route_pin_quarantine
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_native_route_pin_quarantine();

DROP TRIGGER IF EXISTS capacity_native_route_pin_quarantine_worm
    ON infrastructure.capacity_native_route_pin_quarantine;
CREATE TRIGGER capacity_native_route_pin_quarantine_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON infrastructure.capacity_native_route_pin_quarantine
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();

REVOKE ALL ON infrastructure.capacity_native_route_pin_quarantine FROM PUBLIC;
GRANT SELECT, INSERT ON infrastructure.capacity_native_route_pin_quarantine TO star_infrastructure_owner;

COMMENT ON TABLE infrastructure.capacity_native_route_pin_quarantine IS
    'Transaction: append-only tenant-scoped evidence for a native route mismatch; never rewrites the immutable route pin or authorizes recovery.';
COMMENT ON COLUMN infrastructure.capacity_native_route_pin_quarantine.mismatch_fields IS
    'Bounded allow-listed route identity fields that differ from the immutable pin; never stores payload or credential text.';

COMMIT;
