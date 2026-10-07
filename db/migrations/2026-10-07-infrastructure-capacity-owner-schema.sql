-- @cypher schema=1 source_sha256=49c2f5b478ccd6d648a2d460b6bf9de8d549750f03bcf33c0b7a41c46c5b54ed
-- MERGE (self:File {path:"db/migrations/2026-10-07-infrastructure-capacity-owner-schema.sql"})
-- MERGE (schema:Schema {id:"infrastructure"})
-- MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
-- MERGE (bundle:Table {id:"infrastructure.capacity_reservation_bundle"})
-- MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
-- MERGE (reservationEvent:Table {id:"infrastructure.capacity_reservation_event"})
-- MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox"})
-- MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery"})
-- MERGE (workGuard:Function {id:"infrastructure.guard_capacity_owner_work_row"})
-- MERGE (wormGuard:Function {id:"infrastructure.reject_capacity_owner_fact_mutation"})
-- MERGE (self)-[:CREATES]->(schema)
-- MERGE (self)-[:DEFINES]->(inbox)
-- MERGE (self)-[:DEFINES]->(bundle)
-- MERGE (self)-[:DEFINES]->(receipt)
-- MERGE (self)-[:DEFINES]->(reservationEvent)
-- MERGE (self)-[:DEFINES]->(outbox)
-- MERGE (self)-[:DEFINES]->(delivery)
-- MERGE (self)-[:DEFINES]->(workGuard)
-- MERGE (self)-[:DEFINES]->(wormGuard)
-- MERGE (workGuard)-[:GUARDS]->(inbox)
-- MERGE (workGuard)-[:GUARDS]->(bundle)
-- MERGE (workGuard)-[:GUARDS]->(delivery)
-- MERGE (wormGuard)-[:GUARDS]->(receipt)
-- MERGE (wormGuard)-[:GUARDS]->(reservationEvent)
-- MERGE (wormGuard)-[:GUARDS]->(outbox)
-- @endcypher

-- Phase 9F4D-3A persistence substrate. This adds only Infrastructure-owner storage:
-- Run/Execution remains a separate owner and publishes the demand through its own Outbox.
-- No admission writer, provider call, or executor is enabled by this schema.
BEGIN;

CREATE SCHEMA IF NOT EXISTS infrastructure;
REVOKE ALL ON SCHEMA infrastructure FROM PUBLIC;

-- Work: idempotent delivery state with bounded retry and terminal retention.
CREATE TABLE IF NOT EXISTS infrastructure.capacity_demand_inbox (
    inbox_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    source_event_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    project_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    task_execution_run_id UUID NOT NULL,
    schedule_rule_id UUID NOT NULL,
    schedule_rule_revision BIGINT NOT NULL CHECK (schedule_rule_revision > 0),
    binding_id UUID NOT NULL,
    binding_revision BIGINT NOT NULL CHECK (binding_revision > 0),
    binding_digest BYTEA NOT NULL CHECK (octet_length(binding_digest) = 32),
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    k3s_cluster_id UUID NOT NULL,
    k3s_cluster_generation BIGINT NOT NULL CHECK (k3s_cluster_generation > 0),
    provider_kind VARCHAR(32) NOT NULL
        CHECK (provider_kind IN ('multipass_k3s_cluster','remote_k3s')),
    placement_policy VARCHAR(24) NOT NULL
        CHECK (placement_policy IN ('shared_pool','run_dedicated','pinned_cluster')),
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload) = 'object')
        CHECK (octet_length(payload::text) <= 65536),
    payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32),
    deadline_at TIMESTAMPTZ NOT NULL,
    processing_state VARCHAR(12) NOT NULL DEFAULT 'received'
        CHECK (processing_state IN ('received','leased','retry','accepted','rejected')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_owner_id UUID,
    lease_expires_at TIMESTAMPTZ,
    terminal_at TIMESTAMPTZ,
    retention_expires_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (tenant_id, source_event_id),
    UNIQUE (tenant_id, occurrence_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (source_event_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (occurrence_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (deadline_at > received_at),
    CHECK (
        (processing_state = 'leased' AND lease_owner_id IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (processing_state <> 'leased' AND lease_owner_id IS NULL AND lease_expires_at IS NULL)
    ),
    CHECK (
        (processing_state IN ('accepted','rejected') AND terminal_at IS NOT NULL
            AND retention_expires_at > terminal_at)
        OR (processing_state NOT IN ('accepted','rejected') AND terminal_at IS NULL
            AND retention_expires_at IS NULL)
    )
);
CREATE INDEX IF NOT EXISTS idx_capacity_demand_inbox_due
    ON infrastructure.capacity_demand_inbox (next_attempt_at, received_at)
    WHERE processing_state IN ('received','retry');
CREATE INDEX IF NOT EXISTS idx_capacity_demand_inbox_lease_expiry
    ON infrastructure.capacity_demand_inbox (lease_expires_at)
    WHERE processing_state = 'leased';
CREATE INDEX IF NOT EXISTS idx_capacity_demand_inbox_retention
    ON infrastructure.capacity_demand_inbox (retention_expires_at)
    WHERE terminal_at IS NOT NULL;

-- Work: Host and exact-cluster leases share one row so the pair cannot be half-created.
-- The two reservation IDs remain distinct and are echoed by the readiness contract.
CREATE TABLE IF NOT EXISTS infrastructure.capacity_reservation_bundle (
    reservation_bundle_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    source_event_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    task_execution_run_id UUID NOT NULL,
    binding_id UUID NOT NULL,
    binding_revision BIGINT NOT NULL CHECK (binding_revision > 0),
    binding_digest BYTEA NOT NULL CHECK (octet_length(binding_digest) = 32),
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
    reservation_state VARCHAR(10) NOT NULL DEFAULT 'held'
        CHECK (reservation_state IN ('held','released','expired')),
    terminal_at TIMESTAMPTZ,
    retention_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (tenant_id, source_event_id),
    UNIQUE (tenant_id, occurrence_id),
    UNIQUE (tenant_id, host_capacity_reservation_id),
    UNIQUE (tenant_id, cluster_capacity_reservation_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (source_event_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (occurrence_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (host_capacity_reservation_id <> cluster_capacity_reservation_id),
    CHECK (lease_expires_at > created_at),
    CHECK (
        (reservation_state = 'held' AND terminal_at IS NULL AND retention_expires_at IS NULL)
        OR (reservation_state IN ('released','expired') AND terminal_at IS NOT NULL
            AND retention_expires_at > terminal_at)
    )
);
CREATE INDEX IF NOT EXISTS idx_capacity_reservation_host_active
    ON infrastructure.capacity_reservation_bundle (tenant_id, host_environment_id, host_generation)
    WHERE reservation_state = 'held';
CREATE INDEX IF NOT EXISTS idx_capacity_reservation_cluster_active
    ON infrastructure.capacity_reservation_bundle (tenant_id, k3s_cluster_id, k3s_cluster_generation)
    WHERE reservation_state = 'held';
CREATE INDEX IF NOT EXISTS idx_capacity_reservation_lease_expiry
    ON infrastructure.capacity_reservation_bundle (lease_expires_at)
    WHERE reservation_state = 'held';
CREATE INDEX IF NOT EXISTS idx_capacity_reservation_retention
    ON infrastructure.capacity_reservation_bundle (retention_expires_at)
    WHERE terminal_at IS NOT NULL;

-- Transaction: immutable source-event receipt prevents old Work rows from becoming
-- an idempotency authority after their TTL cleanup.
CREATE TABLE IF NOT EXISTS infrastructure.capacity_demand_receipt (
    receipt_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    source_event_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32),
    outcome VARCHAR(10) NOT NULL CHECK (outcome IN ('accepted','rejected')),
    reservation_bundle_id UUID,
    reason_code VARCHAR(64),
    correlation_id UUID NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (tenant_id, source_event_id),
    UNIQUE (tenant_id, occurrence_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (outcome = 'accepted' AND reservation_bundle_id IS NOT NULL AND reason_code IS NULL)
        OR (outcome = 'rejected' AND reservation_bundle_id IS NULL AND reason_code IS NOT NULL)
    )
);

-- Transaction: append-only owner lifecycle facts. No payload may contain credentials,
-- source files, or arbitrary provider output.
CREATE TABLE IF NOT EXISTS infrastructure.capacity_reservation_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    reservation_bundle_id UUID NOT NULL,
    event_type VARCHAR(12) NOT NULL CHECK (event_type IN ('held','released','expired')),
    host_capacity_reservation_id UUID NOT NULL,
    cluster_capacity_reservation_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::JSONB
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (host_capacity_reservation_id <> cluster_capacity_reservation_id)
);
CREATE INDEX IF NOT EXISTS idx_capacity_reservation_event_scope
    ON infrastructure.capacity_reservation_event
        (tenant_id, reservation_bundle_id, occurred_at, event_id);

-- Transaction: owner-to-owner events are immutable and carry only bounded typed facts.
CREATE TABLE IF NOT EXISTS infrastructure.capacity_owner_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    source_event_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    reservation_bundle_id UUID,
    event_type VARCHAR(24) NOT NULL
        CHECK (event_type IN ('capacity_accepted','capacity_rejected','capacity_released','capacity_expired')),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 2),
    idempotency_key VARCHAR(200) NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload) = 'object')
        CHECK (octet_length(payload::text) <= 16384),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (tenant_id, idempotency_key),
    UNIQUE (tenant_id, event_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (length(idempotency_key) BETWEEN 1 AND 200),
    CHECK (
        (event_type IN ('capacity_accepted','capacity_released','capacity_expired')
            AND reservation_bundle_id IS NOT NULL)
        OR (event_type = 'capacity_rejected' AND reservation_bundle_id IS NULL)
    )
);
CREATE INDEX IF NOT EXISTS idx_capacity_owner_outbox_scope
    ON infrastructure.capacity_owner_outbox (tenant_id, occurrence_id, occurred_at, event_id);

-- Work: independently expirable delivery state for immutable Infrastructure Outbox facts.
CREATE TABLE IF NOT EXISTS infrastructure.capacity_outbox_delivery (
    tenant_id UUID NOT NULL,
    event_id UUID NOT NULL,
    delivery_state VARCHAR(10) NOT NULL DEFAULT 'pending'
        CHECK (delivery_state IN ('pending','leased','retry','delivered','dead')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_owner_id UUID,
    lease_expires_at TIMESTAMPTZ,
    terminal_at TIMESTAMPTZ,
    retention_expires_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    FOREIGN KEY (tenant_id, event_id)
        REFERENCES infrastructure.capacity_owner_outbox (tenant_id, event_id)
        ON DELETE RESTRICT,
    CHECK (
        (delivery_state = 'leased' AND lease_owner_id IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (delivery_state <> 'leased' AND lease_owner_id IS NULL AND lease_expires_at IS NULL)
    ),
    CHECK (
        (delivery_state IN ('delivered','dead') AND terminal_at IS NOT NULL
            AND retention_expires_at > terminal_at)
        OR (delivery_state NOT IN ('delivered','dead') AND terminal_at IS NULL
            AND retention_expires_at IS NULL)
    )
);
CREATE INDEX IF NOT EXISTS idx_capacity_outbox_delivery_due
    ON infrastructure.capacity_outbox_delivery (next_attempt_at, event_id)
    WHERE delivery_state IN ('pending','retry');
CREATE INDEX IF NOT EXISTS idx_capacity_outbox_delivery_lease_expiry
    ON infrastructure.capacity_outbox_delivery (lease_expires_at)
    WHERE delivery_state = 'leased';
CREATE INDEX IF NOT EXISTS idx_capacity_outbox_delivery_retention
    ON infrastructure.capacity_outbox_delivery (retention_expires_at)
    WHERE terminal_at IS NOT NULL;

CREATE OR REPLACE FUNCTION infrastructure.reject_capacity_owner_fact_mutation()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
    RAISE EXCEPTION 'Infrastructure capacity fact % is append-only', TG_TABLE_NAME;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.guard_capacity_owner_work_row()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
    current_state TEXT;
    next_state TEXT;
    terminal_at_value TIMESTAMPTZ;
    retention_until_value TIMESTAMPTZ;
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF TG_TABLE_NAME = 'capacity_demand_inbox' THEN
            IF NEW.processing_state <> 'received' THEN
                RAISE EXCEPTION 'Infrastructure Work row must start in its initial state';
            END IF;
        ELSIF TG_TABLE_NAME = 'capacity_reservation_bundle' THEN
            IF NEW.reservation_state <> 'held' THEN
                RAISE EXCEPTION 'Infrastructure Work row must start in its initial state';
            END IF;
        ELSIF TG_TABLE_NAME = 'capacity_outbox_delivery' THEN
            IF NEW.delivery_state <> 'pending' THEN
                RAISE EXCEPTION 'Infrastructure Work row must start in its initial state';
            END IF;
        ELSE
            RAISE EXCEPTION 'Infrastructure Work row must start in its initial state';
        END IF;
        RETURN NEW;
    END IF;

    IF TG_TABLE_NAME = 'capacity_demand_inbox' THEN
        current_state := OLD.processing_state;
        IF TG_OP = 'DELETE' THEN
            IF OLD.terminal_at IS NULL OR OLD.retention_expires_at > clock_timestamp() THEN
                RAISE EXCEPTION 'Infrastructure demand Inbox may only expire after terminal retention';
            END IF;
            RETURN OLD;
        END IF;
        next_state := NEW.processing_state;
        terminal_at_value := NEW.terminal_at;
        retention_until_value := NEW.retention_expires_at;
        IF (to_jsonb(OLD) - ARRAY[
                'processing_state','attempt_count','next_attempt_at','lease_owner_id',
                'lease_expires_at','terminal_at','retention_expires_at','updated_at'
            ]) IS DISTINCT FROM (to_jsonb(NEW) - ARRAY[
                'processing_state','attempt_count','next_attempt_at','lease_owner_id',
                'lease_expires_at','terminal_at','retention_expires_at','updated_at'
            ]) THEN
            RAISE EXCEPTION 'Infrastructure demand Inbox identity and payload are immutable';
        END IF;
        IF current_state IN ('accepted','rejected') THEN
            RAISE EXCEPTION 'Terminal Infrastructure demand Inbox row cannot be rewritten';
        END IF;
        IF (current_state = 'received' AND next_state NOT IN ('received','leased','rejected'))
           OR (current_state = 'retry' AND next_state NOT IN ('retry','leased','rejected'))
           OR (current_state = 'leased' AND next_state NOT IN ('leased','retry','accepted','rejected')) THEN
            RAISE EXCEPTION 'Invalid Infrastructure demand Inbox state transition';
        END IF;
        IF (next_state IN ('accepted','rejected')) <> (terminal_at_value IS NOT NULL)
           OR (terminal_at_value IS NOT NULL AND retention_until_value <= terminal_at_value) THEN
            RAISE EXCEPTION 'Infrastructure demand Inbox terminal retention is invalid';
        END IF;
        IF next_state IN ('accepted','rejected')
           AND (terminal_at_value < clock_timestamp() - interval '1 minute'
                OR terminal_at_value > clock_timestamp() + interval '1 minute'
                OR retention_until_value < terminal_at_value + interval '24 hours'
                OR retention_until_value > terminal_at_value + interval '25 hours') THEN
            RAISE EXCEPTION 'Infrastructure demand Inbox retention must be bounded to 24-25 hours from a current terminal transition';
        END IF;
        RETURN NEW;
    ELSIF TG_TABLE_NAME = 'capacity_reservation_bundle' THEN
        current_state := OLD.reservation_state;
        IF TG_OP = 'DELETE' THEN
            IF OLD.terminal_at IS NULL OR OLD.retention_expires_at > clock_timestamp() THEN
                RAISE EXCEPTION 'Infrastructure reservation may only expire after terminal retention';
            END IF;
            RETURN OLD;
        END IF;
        next_state := NEW.reservation_state;
        terminal_at_value := NEW.terminal_at;
        retention_until_value := NEW.retention_expires_at;
        IF (to_jsonb(OLD) - ARRAY[
                'reservation_state','lease_expires_at','terminal_at','retention_expires_at','updated_at'
            ]) IS DISTINCT FROM (to_jsonb(NEW) - ARRAY[
                'reservation_state','lease_expires_at','terminal_at','retention_expires_at','updated_at'
            ]) THEN
            RAISE EXCEPTION 'Infrastructure reservation identity and capacity are immutable';
        END IF;
        IF current_state <> 'held'
           OR next_state NOT IN ('held','released','expired')
           OR (current_state = 'held' AND next_state IN ('released','expired')
               AND (terminal_at_value IS NULL OR retention_until_value <= terminal_at_value)) THEN
            RAISE EXCEPTION 'Invalid Infrastructure reservation lifecycle transition';
        END IF;
        IF next_state IN ('released','expired')
           AND (terminal_at_value < clock_timestamp() - interval '1 minute'
                OR terminal_at_value > clock_timestamp() + interval '1 minute'
                OR retention_until_value < terminal_at_value + interval '24 hours'
                OR retention_until_value > terminal_at_value + interval '25 hours') THEN
            RAISE EXCEPTION 'Infrastructure reservation retention must be bounded to 24-25 hours from a current terminal transition';
        END IF;
        RETURN NEW;
    ELSIF TG_TABLE_NAME = 'capacity_outbox_delivery' THEN
        current_state := OLD.delivery_state;
        IF TG_OP = 'DELETE' THEN
            IF OLD.terminal_at IS NULL OR OLD.retention_expires_at > clock_timestamp() THEN
                RAISE EXCEPTION 'Infrastructure Outbox delivery may only expire after terminal retention';
            END IF;
            RETURN OLD;
        END IF;
        next_state := NEW.delivery_state;
        terminal_at_value := NEW.terminal_at;
        retention_until_value := NEW.retention_expires_at;
        IF (to_jsonb(OLD) - ARRAY[
                'delivery_state','attempt_count','next_attempt_at','lease_owner_id',
                'lease_expires_at','terminal_at','retention_expires_at','updated_at'
            ]) IS DISTINCT FROM (to_jsonb(NEW) - ARRAY[
                'delivery_state','attempt_count','next_attempt_at','lease_owner_id',
                'lease_expires_at','terminal_at','retention_expires_at','updated_at'
            ]) THEN
            RAISE EXCEPTION 'Infrastructure Outbox delivery identity is immutable';
        END IF;
        IF current_state IN ('delivered','dead')
           OR (current_state = 'pending' AND next_state NOT IN ('pending','leased'))
           OR (current_state = 'retry' AND next_state NOT IN ('retry','leased'))
           OR (current_state = 'leased' AND next_state NOT IN ('leased','retry','delivered','dead')) THEN
            RAISE EXCEPTION 'Invalid Infrastructure Outbox delivery state transition';
        END IF;
        IF (next_state IN ('delivered','dead')) <> (terminal_at_value IS NOT NULL)
           OR (terminal_at_value IS NOT NULL AND retention_until_value <= terminal_at_value) THEN
            RAISE EXCEPTION 'Infrastructure Outbox delivery terminal retention is invalid';
        END IF;
        IF next_state IN ('delivered','dead')
           AND (terminal_at_value < clock_timestamp() - interval '1 minute'
                OR terminal_at_value > clock_timestamp() + interval '1 minute'
                OR retention_until_value < terminal_at_value + interval '24 hours'
                OR retention_until_value > terminal_at_value + interval '25 hours') THEN
            RAISE EXCEPTION 'Infrastructure Outbox delivery retention must be bounded to 24-25 hours from a current terminal transition';
        END IF;
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'Unexpected Infrastructure Work table %', TG_TABLE_NAME;
END;
$$;

DROP TRIGGER IF EXISTS capacity_demand_inbox_guard ON infrastructure.capacity_demand_inbox;
CREATE TRIGGER capacity_demand_inbox_guard
    BEFORE INSERT OR UPDATE OR DELETE ON infrastructure.capacity_demand_inbox
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_owner_work_row();
DROP TRIGGER IF EXISTS capacity_reservation_bundle_guard ON infrastructure.capacity_reservation_bundle;
CREATE TRIGGER capacity_reservation_bundle_guard
    BEFORE INSERT OR UPDATE OR DELETE ON infrastructure.capacity_reservation_bundle
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_owner_work_row();
DROP TRIGGER IF EXISTS capacity_outbox_delivery_guard ON infrastructure.capacity_outbox_delivery;
CREATE TRIGGER capacity_outbox_delivery_guard
    BEFORE INSERT OR UPDATE OR DELETE ON infrastructure.capacity_outbox_delivery
    FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_capacity_owner_work_row();

DROP TRIGGER IF EXISTS capacity_demand_receipt_worm ON infrastructure.capacity_demand_receipt;
CREATE TRIGGER capacity_demand_receipt_worm
    BEFORE UPDATE OR DELETE ON infrastructure.capacity_demand_receipt
    FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();
DROP TRIGGER IF EXISTS capacity_reservation_event_worm ON infrastructure.capacity_reservation_event;
CREATE TRIGGER capacity_reservation_event_worm
    BEFORE UPDATE OR DELETE ON infrastructure.capacity_reservation_event
    FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();
DROP TRIGGER IF EXISTS capacity_owner_outbox_worm ON infrastructure.capacity_owner_outbox;
CREATE TRIGGER capacity_owner_outbox_worm
    BEFORE UPDATE OR DELETE ON infrastructure.capacity_owner_outbox
    FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();
DROP TRIGGER IF EXISTS capacity_demand_receipt_truncate_guard ON infrastructure.capacity_demand_receipt;
CREATE TRIGGER capacity_demand_receipt_truncate_guard
    BEFORE TRUNCATE ON infrastructure.capacity_demand_receipt
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();
DROP TRIGGER IF EXISTS capacity_reservation_event_truncate_guard ON infrastructure.capacity_reservation_event;
CREATE TRIGGER capacity_reservation_event_truncate_guard
    BEFORE TRUNCATE ON infrastructure.capacity_reservation_event
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();
DROP TRIGGER IF EXISTS capacity_owner_outbox_truncate_guard ON infrastructure.capacity_owner_outbox;
CREATE TRIGGER capacity_owner_outbox_truncate_guard
    BEFORE TRUNCATE ON infrastructure.capacity_owner_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_capacity_owner_fact_mutation();

ALTER TABLE infrastructure.capacity_demand_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_demand_inbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_demand_inbox_tenant_scope ON infrastructure.capacity_demand_inbox;
CREATE POLICY capacity_demand_inbox_tenant_scope ON infrastructure.capacity_demand_inbox
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.capacity_reservation_bundle ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_reservation_bundle FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_reservation_bundle_tenant_scope ON infrastructure.capacity_reservation_bundle;
CREATE POLICY capacity_reservation_bundle_tenant_scope ON infrastructure.capacity_reservation_bundle
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.capacity_demand_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_demand_receipt FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_demand_receipt_tenant_scope ON infrastructure.capacity_demand_receipt;
CREATE POLICY capacity_demand_receipt_tenant_scope ON infrastructure.capacity_demand_receipt
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.capacity_reservation_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_reservation_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_reservation_event_tenant_scope ON infrastructure.capacity_reservation_event;
CREATE POLICY capacity_reservation_event_tenant_scope ON infrastructure.capacity_reservation_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.capacity_owner_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_owner_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_owner_outbox_tenant_scope ON infrastructure.capacity_owner_outbox;
CREATE POLICY capacity_owner_outbox_tenant_scope ON infrastructure.capacity_owner_outbox
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE infrastructure.capacity_outbox_delivery ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.capacity_outbox_delivery FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS capacity_outbox_delivery_tenant_scope ON infrastructure.capacity_outbox_delivery;
CREATE POLICY capacity_outbox_delivery_tenant_scope ON infrastructure.capacity_outbox_delivery
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

REVOKE ALL ON ALL TABLES IN SCHEMA infrastructure FROM PUBLIC;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA infrastructure FROM PUBLIC;

COMMIT;
