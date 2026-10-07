-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/migrations/2026-10-10-infrastructure-physical-capacity-ledger.sql"})
-- MERGE (reservation:Table {id:"infrastructure.capacity_physical_reservation",class:"Work"})
-- MERGE (event:Table {id:"infrastructure.capacity_physical_reservation_event",class:"Transaction"})
-- MERGE (guard:Function {id:"infrastructure.guard_capacity_physical_reservation"})
-- MERGE (transition_event:Function {id:"infrastructure.record_capacity_physical_reservation_transition"})
-- MERGE (worm:Function {id:"infrastructure.reject_capacity_physical_event_mutation"})
-- MERGE (reservation_rls:Policy {id:"infrastructure.capacity_physical_reservation.infrastructure_owner"})
-- MERGE (event_rls:Policy {id:"infrastructure.capacity_physical_reservation_event.infrastructure_owner"})
-- MERGE (owner:Role {id:"star_infrastructure_owner"})
-- MERGE (self)-[:CREATES]->(reservation)
-- MERGE (self)-[:CREATES]->(event)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:DEFINES]->(transition_event)
-- MERGE (self)-[:DEFINES]->(worm)
-- MERGE (self)-[:CONFIGURES]->(reservation_rls)
-- MERGE (self)-[:CONFIGURES]->(event_rls)
-- MERGE (self)-[:CONFIGURES]->(owner)
-- MERGE (guard)-[:WRITES]->(reservation)
-- MERGE (transition_event)-[:WRITES]->(event)
-- MERGE (worm)-[:WRITES]->(event)
-- @endcypher

-- Global physical capacity is owned by the Infrastructure service, not by a tenant.
-- These rows intentionally contain no tenant_id; tenant demand and audit facts remain in
-- the existing FORCE-RLS tenant tables. One Infrastructure transaction writes both ledgers.
BEGIN;

CREATE TABLE IF NOT EXISTS infrastructure.capacity_physical_reservation (
    reservation_bundle_id UUID PRIMARY KEY,
    host_capacity_reservation_id UUID NOT NULL UNIQUE,
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    host_cpu_millicores BIGINT NOT NULL CHECK (host_cpu_millicores > 0),
    host_memory_bytes BIGINT NOT NULL CHECK (host_memory_bytes > 0),
    host_ephemeral_storage_bytes BIGINT NOT NULL CHECK (host_ephemeral_storage_bytes >= 0),
    host_gpu_units INTEGER NOT NULL CHECK (host_gpu_units >= 0),
    cluster_capacity_reservation_id UUID NOT NULL UNIQUE,
    k3s_cluster_id UUID NOT NULL,
    k3s_cluster_generation BIGINT NOT NULL CHECK (k3s_cluster_generation > 0),
    cluster_cpu_millicores BIGINT NOT NULL CHECK (cluster_cpu_millicores > 0),
    cluster_memory_bytes BIGINT NOT NULL CHECK (cluster_memory_bytes > 0),
    cluster_ephemeral_storage_bytes BIGINT NOT NULL CHECK (cluster_ephemeral_storage_bytes >= 0),
    cluster_gpu_units INTEGER NOT NULL CHECK (cluster_gpu_units >= 0),
    capacity_snapshot_digest BYTEA NOT NULL CHECK (octet_length(capacity_snapshot_digest) = 32),
    capacity_observed_at TIMESTAMPTZ NOT NULL,
    lease_expires_at TIMESTAMPTZ NOT NULL,
    reservation_state VARCHAR(10) NOT NULL DEFAULT 'held'
        CHECK (reservation_state IN ('held','released','expired')),
    terminal_at TIMESTAMPTZ,
    retention_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CHECK (host_capacity_reservation_id <> cluster_capacity_reservation_id),
    CHECK (lease_expires_at > created_at),
    CHECK (
        (reservation_state = 'held' AND terminal_at IS NULL AND retention_expires_at IS NULL)
        OR (reservation_state IN ('released','expired') AND terminal_at IS NOT NULL
            AND retention_expires_at > terminal_at)
    )
);

CREATE INDEX IF NOT EXISTS idx_capacity_physical_host_active
    ON infrastructure.capacity_physical_reservation
       (host_environment_id, host_generation, lease_expires_at)
    WHERE reservation_state = 'held';
CREATE INDEX IF NOT EXISTS idx_capacity_physical_cluster_active
    ON infrastructure.capacity_physical_reservation
       (k3s_cluster_id, k3s_cluster_generation, lease_expires_at)
    WHERE reservation_state = 'held';
CREATE INDEX IF NOT EXISTS idx_capacity_physical_retention
    ON infrastructure.capacity_physical_reservation (retention_expires_at)
    WHERE terminal_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS infrastructure.capacity_physical_reservation_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reservation_bundle_id UUID NOT NULL,
    event_type VARCHAR(12) NOT NULL CHECK (event_type IN ('held','released','expired')),
    host_capacity_reservation_id UUID NOT NULL,
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    host_cpu_millicores BIGINT NOT NULL CHECK (host_cpu_millicores > 0),
    host_memory_bytes BIGINT NOT NULL CHECK (host_memory_bytes > 0),
    host_ephemeral_storage_bytes BIGINT NOT NULL CHECK (host_ephemeral_storage_bytes >= 0),
    host_gpu_units INTEGER NOT NULL CHECK (host_gpu_units >= 0),
    cluster_capacity_reservation_id UUID NOT NULL,
    k3s_cluster_id UUID NOT NULL,
    k3s_cluster_generation BIGINT NOT NULL CHECK (k3s_cluster_generation > 0),
    cluster_cpu_millicores BIGINT NOT NULL CHECK (cluster_cpu_millicores > 0),
    cluster_memory_bytes BIGINT NOT NULL CHECK (cluster_memory_bytes > 0),
    cluster_ephemeral_storage_bytes BIGINT NOT NULL CHECK (cluster_ephemeral_storage_bytes >= 0),
    cluster_gpu_units INTEGER NOT NULL CHECK (cluster_gpu_units >= 0),
    capacity_snapshot_digest BYTEA NOT NULL CHECK (octet_length(capacity_snapshot_digest) = 32),
    capacity_observed_at TIMESTAMPTZ NOT NULL,
    lease_expires_at TIMESTAMPTZ NOT NULL,
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    happened_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CHECK (host_capacity_reservation_id <> cluster_capacity_reservation_id)
);
CREATE INDEX IF NOT EXISTS idx_capacity_physical_event_bundle
    ON infrastructure.capacity_physical_reservation_event
       (reservation_bundle_id, happened_at, event_id);

CREATE OR REPLACE FUNCTION infrastructure.guard_capacity_physical_reservation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.reservation_state <> 'held' OR NEW.terminal_at IS NOT NULL
           OR NEW.retention_expires_at IS NOT NULL
           OR NEW.created_at > clock_timestamp() + interval '1 minute'
           OR NEW.updated_at > clock_timestamp() + interval '1 minute' THEN
            RAISE EXCEPTION 'physical capacity Work row must start as a current held reservation';
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.terminal_at IS NULL OR OLD.retention_expires_at > clock_timestamp() THEN
            RAISE EXCEPTION 'physical capacity reservation may only expire after terminal retention';
        END IF;
        RETURN OLD;
    END IF;

    IF (to_jsonb(OLD) - ARRAY[
            'reservation_state','lease_expires_at','terminal_at','retention_expires_at','updated_at'
        ]) IS DISTINCT FROM (to_jsonb(NEW) - ARRAY[
            'reservation_state','lease_expires_at','terminal_at','retention_expires_at','updated_at'
        ]) THEN
        RAISE EXCEPTION 'physical capacity reservation identity and resource vectors are immutable';
    END IF;
    IF OLD.reservation_state <> 'held'
       OR NEW.reservation_state NOT IN ('held','released','expired')
       OR (NEW.reservation_state IN ('released','expired')
           AND (NEW.terminal_at IS NULL OR NEW.retention_expires_at <= NEW.terminal_at))
       OR (NEW.reservation_state = 'held'
           AND (NEW.terminal_at IS NOT NULL OR NEW.retention_expires_at IS NOT NULL)) THEN
        RAISE EXCEPTION 'invalid physical capacity reservation lifecycle transition';
    END IF;
    IF NEW.reservation_state IN ('released','expired')
       AND (NEW.terminal_at < clock_timestamp() - interval '1 minute'
            OR NEW.terminal_at > clock_timestamp() + interval '1 minute'
            OR NEW.retention_expires_at < NEW.terminal_at + interval '24 hours'
            OR NEW.retention_expires_at > NEW.terminal_at + interval '25 hours') THEN
        RAISE EXCEPTION 'physical capacity reservation retention must be 24-25 hours from a current transition';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.reject_capacity_physical_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    RAISE EXCEPTION 'physical capacity reservation events are append-only';
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.record_capacity_physical_reservation_transition()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    actor_text TEXT;
BEGIN
    IF OLD.reservation_state IS DISTINCT FROM NEW.reservation_state THEN
        actor_text := NULLIF(current_setting('app.actor_id', true), '');
        IF actor_text IS NULL THEN
            RAISE EXCEPTION 'physical reservation transition requires an Infrastructure actor';
        END IF;
        INSERT INTO infrastructure.capacity_physical_reservation_event (
            reservation_bundle_id, event_type, host_capacity_reservation_id,
            host_environment_id, host_generation, host_cpu_millicores, host_memory_bytes,
            host_ephemeral_storage_bytes, host_gpu_units, cluster_capacity_reservation_id,
            k3s_cluster_id, k3s_cluster_generation, cluster_cpu_millicores,
            cluster_memory_bytes, cluster_ephemeral_storage_bytes, cluster_gpu_units,
            capacity_snapshot_digest, capacity_observed_at, lease_expires_at, actor_id
        ) VALUES (
            NEW.reservation_bundle_id, NEW.reservation_state, NEW.host_capacity_reservation_id,
            NEW.host_environment_id, NEW.host_generation, NEW.host_cpu_millicores,
            NEW.host_memory_bytes, NEW.host_ephemeral_storage_bytes, NEW.host_gpu_units,
            NEW.cluster_capacity_reservation_id, NEW.k3s_cluster_id,
            NEW.k3s_cluster_generation, NEW.cluster_cpu_millicores, NEW.cluster_memory_bytes,
            NEW.cluster_ephemeral_storage_bytes, NEW.cluster_gpu_units,
            NEW.capacity_snapshot_digest, NEW.capacity_observed_at, NEW.lease_expires_at,
            actor_text::UUID
        );
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS capacity_physical_reservation_guard
    ON infrastructure.capacity_physical_reservation;
CREATE TRIGGER capacity_physical_reservation_guard
    BEFORE INSERT OR UPDATE OR DELETE ON infrastructure.capacity_physical_reservation
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_physical_reservation();
DROP TRIGGER IF EXISTS capacity_physical_reservation_transition_event
    ON infrastructure.capacity_physical_reservation;
CREATE TRIGGER capacity_physical_reservation_transition_event
    AFTER UPDATE OF reservation_state ON infrastructure.capacity_physical_reservation
    FOR EACH ROW EXECUTE FUNCTION infrastructure.record_capacity_physical_reservation_transition();
DROP TRIGGER IF EXISTS capacity_physical_reservation_event_worm
    ON infrastructure.capacity_physical_reservation_event;
CREATE TRIGGER capacity_physical_reservation_event_worm
    BEFORE UPDATE OR DELETE ON infrastructure.capacity_physical_reservation_event
    FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_capacity_physical_event_mutation();
DROP TRIGGER IF EXISTS capacity_physical_reservation_event_truncate_guard
    ON infrastructure.capacity_physical_reservation_event;
CREATE TRIGGER capacity_physical_reservation_event_truncate_guard
    BEFORE TRUNCATE ON infrastructure.capacity_physical_reservation_event
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_physical_event_mutation();

ALTER TABLE infrastructure.capacity_physical_reservation ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_physical_reservation FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_physical_reservation_infrastructure_owner
    ON infrastructure.capacity_physical_reservation;
CREATE POLICY capacity_physical_reservation_infrastructure_owner
    ON infrastructure.capacity_physical_reservation
    TO star_infrastructure_owner USING (true) WITH CHECK (true);

ALTER TABLE infrastructure.capacity_physical_reservation_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_physical_reservation_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_physical_reservation_event_infrastructure_owner
    ON infrastructure.capacity_physical_reservation_event;
CREATE POLICY capacity_physical_reservation_event_infrastructure_owner
    ON infrastructure.capacity_physical_reservation_event
    TO star_infrastructure_owner USING (true) WITH CHECK (true);

REVOKE ALL ON TABLE infrastructure.capacity_physical_reservation,
    infrastructure.capacity_physical_reservation_event FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE
    infrastructure.capacity_physical_reservation TO star_infrastructure_owner;
GRANT SELECT, INSERT ON TABLE
    infrastructure.capacity_physical_reservation_event TO star_infrastructure_owner;

COMMENT ON TABLE infrastructure.capacity_physical_reservation IS
    'W: global Infrastructure-owner Host/cluster physical reservation ledger; contains no tenant identity.';
COMMENT ON TABLE infrastructure.capacity_physical_reservation_event IS
    'T: global append-only physical capacity lifecycle facts; no tenant identity or Run payload.';

COMMIT;
