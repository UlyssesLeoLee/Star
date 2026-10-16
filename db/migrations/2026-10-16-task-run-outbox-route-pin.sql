-- @cypher schema=1 source_sha256=39371fbd7b59186af78f57366cda60507f64643aa37cc6c57f82359f0a211f77
-- CREATE (self:File {path:"db/migrations/2026-10-16-task-run-outbox-route-pin.sql",type:"file",language:"sql"}),(m:Module {name:"task_run_outbox_route_pin_migration",type:"module",language:"sql"}),(route:Table {name:"multica.task_run_outbox_delivery_route_pin",type:"table",language:"sql"}),(outbox:Table {name:"multica.task_run_outbox",type:"table",language:"sql"}),(delivery:Table {name:"multica.task_run_outbox_delivery",type:"table",language:"sql"}),(reject:Function {name:"multica.reject_task_run_outbox_delivery_route_pin_mutation",type:"function",language:"sql"}),(guard:Function {name:"multica.guard_task_run_outbox_delivery_route_pin_insert",type:"function",language:"sql"}),(pin:Function {name:"multica.get_or_pin_task_run_outbox_delivery_route",type:"function",language:"sql"}),(insert_guard:Symbol {name:"task_run_outbox_delivery_route_pin_insert_guard",type:"trigger"}),(worm:Symbol {name:"task_run_outbox_delivery_route_pin_worm",type:"trigger"}),(delivery_owner:Config {name:"star_task_run_delivery_owner",type:"database_role"}),(run_owner:Config {name:"star_multica_run_owner",type:"database_role"});
-- CREATE (self)-[:CONTAINS]->(m),(m)-[:DEFINES]->(route),(m)-[:USES]->(outbox),(m)-[:USES]->(delivery),(m)-[:DEFINES]->(reject),(m)-[:DEFINES]->(guard),(m)-[:DEFINES]->(pin),(m)-[:DEFINES]->(insert_guard),(m)-[:DEFINES]->(worm),(m)-[:CONFIGURES]->(delivery_owner),(m)-[:EXPOSES]->(run_owner),(guard)-[:READS]->(delivery),(guard)-[:READS]->(outbox),(pin)-[:READS]->(delivery),(pin)-[:READS]->(outbox),(pin)-[:READS]->(route),(pin)-[:WRITES]->(route),(insert_guard)-[:CALLS]->(guard),(worm)-[:CALLS]->(reject);
-- @endcypher

-- Pin the trusted route identity and exact wire digest before the first external append.
-- The permanent source Outbox is the parent; the expiring delivery Work is only a lease fence.
-- The row deliberately contains no event payload, endpoint URI, credential, or token.
BEGIN;

CREATE TABLE IF NOT EXISTS multica.task_run_outbox_delivery_route_pin (
    tenant_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    event_id UUID NOT NULL,
    sender_service_id UUID NOT NULL,
    receiver_service_id UUID NOT NULL,
    transport_provider_id VARCHAR(64) NOT NULL,
    transport_provider_version VARCHAR(64) NOT NULL,
    transport_profile_id UUID NOT NULL,
    transport_profile_revision BIGINT NOT NULL,
    transport_profile_digest BYTEA NOT NULL,
    endpoint_identity VARCHAR(128) NOT NULL,
    destination_id VARCHAR(128) NOT NULL,
    destination_generation UUID NOT NULL,
    schema_version INTEGER NOT NULL,
    wire_sha256 BYTEA NOT NULL,
    pinned_by_worker_id UUID NOT NULL,
    pinned_attempt_count INTEGER NOT NULL,
    pinned_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    FOREIGN KEY (tenant_id, engineering_run_id, event_id)
        REFERENCES multica.task_run_outbox (tenant_id, engineering_run_id, event_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (receiver_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (sender_service_id <> receiver_service_id),
    CHECK (transport_provider_id ~ '^[a-z][a-z0-9-]{0,63}$'),
    CHECK (transport_provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'),
    CHECK (transport_profile_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (transport_profile_revision > 0),
    CHECK (octet_length(transport_profile_digest) = 32),
    CHECK (endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    CHECK (destination_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    CHECK (destination_generation <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (schema_version > 0),
    CHECK (octet_length(wire_sha256) = 32),
    CHECK (pinned_by_worker_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (pinned_attempt_count BETWEEN 1 AND 12)
);

ALTER TABLE multica.task_run_outbox_delivery_route_pin ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_run_outbox_delivery_route_pin FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_run_outbox_delivery_route_pin_owner_select
    ON multica.task_run_outbox_delivery_route_pin;
CREATE POLICY task_run_outbox_delivery_route_pin_owner_select
    ON multica.task_run_outbox_delivery_route_pin FOR SELECT TO star_task_run_delivery_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_run_outbox_delivery_route_pin_owner_insert
    ON multica.task_run_outbox_delivery_route_pin;
CREATE POLICY task_run_outbox_delivery_route_pin_owner_insert
    ON multica.task_run_outbox_delivery_route_pin FOR INSERT TO star_task_run_delivery_owner
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

REVOKE ALL ON multica.task_run_outbox_delivery_route_pin FROM PUBLIC, star_multica_run_owner;
GRANT SELECT, INSERT ON multica.task_run_outbox_delivery_route_pin TO star_task_run_delivery_owner;

CREATE OR REPLACE FUNCTION multica.reject_task_run_outbox_delivery_route_pin_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Task Run delivery route pins are immutable' USING ERRCODE = '55000';
END;
$$;

CREATE OR REPLACE FUNCTION multica.guard_task_run_outbox_delivery_route_pin_insert()
RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    delivery RECORD;
BEGIN
    SELECT d.*, o.schema_version
      INTO delivery
      FROM multica.task_run_outbox_delivery AS d
      JOIN multica.task_run_outbox AS o
        ON o.tenant_id = d.tenant_id
       AND o.engineering_run_id = d.engineering_run_id
       AND o.event_id = d.event_id
     WHERE d.tenant_id = NEW.tenant_id
       AND d.engineering_run_id = NEW.engineering_run_id
       AND d.event_id = NEW.event_id
     FOR UPDATE OF d;

    IF active_tenant IS NULL OR active_tenant <> NEW.tenant_id
       OR NOT FOUND
       OR delivery.delivery_state <> 'leased'
       OR delivery.lease_owner_id IS DISTINCT FROM NEW.pinned_by_worker_id
       OR delivery.attempt_count IS DISTINCT FROM NEW.pinned_attempt_count
       OR delivery.lease_expires_at IS NULL
       OR delivery.lease_expires_at <= clock_timestamp()
       OR delivery.expires_at <= clock_timestamp()
       OR delivery.schema_version IS DISTINCT FROM NEW.schema_version THEN
        RAISE EXCEPTION 'Task Run delivery route pin requires the current matching lease and source schema'
            USING ERRCODE = '40001';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS task_run_outbox_delivery_route_pin_insert_guard
    ON multica.task_run_outbox_delivery_route_pin;
CREATE TRIGGER task_run_outbox_delivery_route_pin_insert_guard
    BEFORE INSERT ON multica.task_run_outbox_delivery_route_pin
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_run_outbox_delivery_route_pin_insert();

DROP TRIGGER IF EXISTS task_run_outbox_delivery_route_pin_worm
    ON multica.task_run_outbox_delivery_route_pin;
CREATE TRIGGER task_run_outbox_delivery_route_pin_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON multica.task_run_outbox_delivery_route_pin
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_task_run_outbox_delivery_route_pin_mutation();

CREATE OR REPLACE FUNCTION multica.get_or_pin_task_run_outbox_delivery_route(
    p_event_id UUID,
    p_worker UUID,
    p_attempt_count INTEGER,
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
    p_schema_version INTEGER,
    p_wire_sha256 BYTEA
)
RETURNS TABLE (
    tenant_id UUID,
    engineering_run_id UUID,
    event_id UUID,
    sender_service_id UUID,
    receiver_service_id UUID,
    transport_provider_id VARCHAR,
    transport_provider_version VARCHAR,
    transport_profile_id UUID,
    transport_profile_revision BIGINT,
    transport_profile_digest BYTEA,
    endpoint_identity VARCHAR,
    destination_id VARCHAR,
    destination_generation UUID,
    schema_version INTEGER,
    wire_sha256 BYTEA,
    pinned_by_worker_id UUID,
    pinned_attempt_count INTEGER,
    pinned_at TIMESTAMPTZ
)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    delivery RECORD;
    pinned multica.task_run_outbox_delivery_route_pin%ROWTYPE;
BEGIN
    IF active_tenant IS NULL OR active_tenant = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_event_id IS NULL OR p_worker IS NULL
       OR p_worker = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_attempt_count IS NULL OR p_attempt_count NOT BETWEEN 1 AND 12
       OR p_sender_service_id IS NULL OR p_sender_service_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_receiver_service_id IS NULL OR p_receiver_service_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_sender_service_id = p_receiver_service_id
       OR p_transport_provider_id IS NULL
       OR p_transport_provider_id !~ '^[a-z][a-z0-9-]{0,63}$'
       OR p_transport_provider_version IS NULL
       OR p_transport_provider_version !~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'
       OR p_transport_profile_id IS NULL
       OR p_transport_profile_id = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_transport_profile_revision IS NULL OR p_transport_profile_revision <= 0
       OR p_transport_profile_digest IS NULL OR octet_length(p_transport_profile_digest) <> 32
       OR p_endpoint_identity IS NULL
       OR p_endpoint_identity !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_destination_id IS NULL
       OR p_destination_id !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
       OR p_destination_generation IS NULL
       OR p_destination_generation = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_schema_version IS NULL OR p_schema_version <= 0
       OR p_wire_sha256 IS NULL OR octet_length(p_wire_sha256) <> 32 THEN
        RAISE EXCEPTION 'Invalid Task Run delivery route pin boundary' USING ERRCODE = '22023';
    END IF;

    SELECT d.*, o.schema_version
      INTO delivery
      FROM multica.task_run_outbox_delivery AS d
      JOIN multica.task_run_outbox AS o
        ON o.tenant_id = d.tenant_id
       AND o.engineering_run_id = d.engineering_run_id
       AND o.event_id = d.event_id
     WHERE d.tenant_id = active_tenant AND d.event_id = p_event_id
     FOR UPDATE OF d;

    IF NOT FOUND OR delivery.delivery_state <> 'leased'
       OR delivery.lease_owner_id IS DISTINCT FROM p_worker
       OR delivery.attempt_count IS DISTINCT FROM p_attempt_count
       OR delivery.lease_expires_at IS NULL
       OR delivery.lease_expires_at <= clock_timestamp()
       OR delivery.expires_at <= clock_timestamp()
       OR delivery.schema_version IS DISTINCT FROM p_schema_version THEN
        RAISE EXCEPTION 'Task Run delivery route pin lost the current lease or source schema fence'
            USING ERRCODE = '40001';
    END IF;

    INSERT INTO multica.task_run_outbox_delivery_route_pin (
        tenant_id, engineering_run_id, event_id,
        sender_service_id, receiver_service_id,
        transport_provider_id, transport_provider_version,
        transport_profile_id, transport_profile_revision, transport_profile_digest,
        endpoint_identity, destination_id, destination_generation,
        schema_version, wire_sha256, pinned_by_worker_id, pinned_attempt_count
    ) VALUES (
        active_tenant, delivery.engineering_run_id, p_event_id,
        p_sender_service_id, p_receiver_service_id,
        p_transport_provider_id, p_transport_provider_version,
        p_transport_profile_id, p_transport_profile_revision, p_transport_profile_digest,
        p_endpoint_identity, p_destination_id, p_destination_generation,
        p_schema_version, p_wire_sha256, p_worker, p_attempt_count
    ) ON CONFLICT ON CONSTRAINT task_run_outbox_delivery_route_pin_pkey DO NOTHING;

    SELECT * INTO pinned
      FROM multica.task_run_outbox_delivery_route_pin AS route
     WHERE route.tenant_id = active_tenant AND route.event_id = p_event_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Task Run delivery route pin was not durably readable after insert'
            USING ERRCODE = '40001';
    END IF;

    IF pinned.engineering_run_id IS DISTINCT FROM delivery.engineering_run_id
       OR pinned.sender_service_id IS DISTINCT FROM p_sender_service_id
       OR pinned.receiver_service_id IS DISTINCT FROM p_receiver_service_id
       OR pinned.transport_provider_id IS DISTINCT FROM p_transport_provider_id
       OR pinned.transport_provider_version IS DISTINCT FROM p_transport_provider_version
       OR pinned.transport_profile_id IS DISTINCT FROM p_transport_profile_id
       OR pinned.transport_profile_revision IS DISTINCT FROM p_transport_profile_revision
       OR pinned.transport_profile_digest IS DISTINCT FROM p_transport_profile_digest
       OR pinned.endpoint_identity IS DISTINCT FROM p_endpoint_identity
       OR pinned.destination_id IS DISTINCT FROM p_destination_id
       OR pinned.destination_generation IS DISTINCT FROM p_destination_generation
       OR pinned.schema_version IS DISTINCT FROM p_schema_version
       OR pinned.wire_sha256 IS DISTINCT FROM p_wire_sha256 THEN
        RAISE EXCEPTION 'Task Run delivery route differs from its immutable first-send pin'
            USING ERRCODE = '55000';
    END IF;

    RETURN QUERY
    SELECT route.tenant_id, route.engineering_run_id, route.event_id,
           route.sender_service_id, route.receiver_service_id,
           route.transport_provider_id, route.transport_provider_version,
           route.transport_profile_id, route.transport_profile_revision,
           route.transport_profile_digest, route.endpoint_identity, route.destination_id,
           route.destination_generation, route.schema_version, route.wire_sha256,
           route.pinned_by_worker_id, route.pinned_attempt_count, route.pinned_at
      FROM multica.task_run_outbox_delivery_route_pin AS route
     WHERE route.tenant_id = active_tenant AND route.event_id = p_event_id;
END;
$$;

ALTER FUNCTION multica.reject_task_run_outbox_delivery_route_pin_mutation()
    OWNER TO star_task_run_delivery_owner;
ALTER FUNCTION multica.guard_task_run_outbox_delivery_route_pin_insert()
    OWNER TO star_task_run_delivery_owner;
ALTER FUNCTION multica.get_or_pin_task_run_outbox_delivery_route(
    UUID, UUID, INTEGER, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA,
    VARCHAR, VARCHAR, UUID, INTEGER, BYTEA
) OWNER TO star_task_run_delivery_owner;
REVOKE ALL ON FUNCTION multica.reject_task_run_outbox_delivery_route_pin_mutation() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.guard_task_run_outbox_delivery_route_pin_insert() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.get_or_pin_task_run_outbox_delivery_route(
    UUID, UUID, INTEGER, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA,
    VARCHAR, VARCHAR, UUID, INTEGER, BYTEA
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.get_or_pin_task_run_outbox_delivery_route(
    UUID, UUID, INTEGER, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA,
    VARCHAR, VARCHAR, UUID, INTEGER, BYTEA
) TO star_multica_run_owner;

COMMENT ON TABLE multica.task_run_outbox_delivery_route_pin IS
    'T: immutable Task Run delivery identity pinned under the current lease before first send; stores route/profile and exact-wire digest only, never payload, endpoint URI, credential, or token.';
COMMENT ON FUNCTION multica.get_or_pin_task_run_outbox_delivery_route(
    UUID, UUID, INTEGER, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT, BYTEA,
    VARCHAR, VARCHAR, UUID, INTEGER, BYTEA
) IS
    'Returns the immutable route only when the caller presents the current delivery lease and an identical resolved route and exact-wire digest; route resolution and receiver authorization remain trusted Rust responsibilities.';

COMMIT;
