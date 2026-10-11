-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-08-infrastructure-resource-manager-receipts.sql"})
-- MERGE (receipt:Table {id:"infrastructure.resource_catalog_manager_receipt",class:"Transaction"})
-- MERGE (event:Table {id:"infrastructure.resource_catalog_event",class:"Transaction"})
-- MERGE (instance:Table {id:"infrastructure.multipass_instance_catalog",class:"Master"})
-- MERGE (host:Table {id:"infrastructure.host_environment_catalog",class:"Master"})
-- MERGE (cluster:Table {id:"infrastructure.k3s_cluster_catalog",class:"Master"})
-- MERGE (membership:Table {id:"infrastructure.k3s_cluster_instance_membership",class:"Master"})
-- MERGE (node:Table {id:"infrastructure.kubernetes_node_catalog",class:"Master"})
-- MERGE (validate:Function {id:"infrastructure.validate_resource_catalog_manager_receipt"})
-- MERGE (require_event:Function {id:"infrastructure.require_resource_catalog_receipt_event"})
-- MERGE (worm:Function {id:"infrastructure.reject_resource_catalog_receipt_mutation"})
-- MERGE (self)-[:DEFINES]->(receipt)
-- MERGE (self)-[:ALTERS]->(event)
-- MERGE (self)-[:CONSTRAINS]->(instance)
-- MERGE (validate)-[:CONSTRAINS]->(event)
-- MERGE (validate)-[:READS]->(receipt)
-- MERGE (require_event)-[:CONSTRAINS]->(receipt)
-- MERGE (require_event)-[:READS]->(event)
-- MERGE (worm)-[:CONSTRAINS]->(receipt)
-- MERGE (event)-[:CONSTRAINS]->(receipt)
-- @endcypher

-- INFRA-2C2: signed Rust Manager evidence is required for every new catalog registration.
-- Existing unsigned catalog facts are not silently adopted as product-owned resources.
BEGIN;

CREATE TABLE IF NOT EXISTS infrastructure.resource_catalog_manager_receipt (
    receipt_id UUID PRIMARY KEY,
    operation_id UUID NOT NULL UNIQUE,
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    manager_id UUID NOT NULL CHECK (manager_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    manager_key_id VARCHAR(128) NOT NULL
        CHECK (manager_key_id ~ '^[A-Za-z0-9._:-]{1,128}$'),
    resource_kind VARCHAR(32) NOT NULL CHECK (resource_kind IN (
        'host_environment','multipass_instance','k3s_cluster',
        'cluster_instance_membership','kubernetes_node'
    )),
    resource_id UUID NOT NULL,
    resource_generation BIGINT NOT NULL CHECK (resource_generation > 0),
    event_type VARCHAR(16) NOT NULL CHECK (event_type IN ('registered','state_changed','retired')),
    owner_state VARCHAR(12) NOT NULL CHECK (owner_state IN ('active','draining','retired')),
    correlation_id UUID NOT NULL CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32),
    signature BYTEA NOT NULL CHECK (octet_length(signature) = 64),
    observed_at TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CHECK (receipt_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (operation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (resource_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (event_type = 'registered' AND owner_state = 'active')
        OR (event_type = 'state_changed' AND owner_state IN ('active','draining'))
        OR (event_type = 'retired' AND owner_state = 'retired')
    )
);

ALTER TABLE infrastructure.resource_catalog_event
    ADD COLUMN IF NOT EXISTS manager_receipt_id UUID;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
          FROM infrastructure.resource_catalog_event event
          LEFT JOIN infrastructure.resource_catalog_manager_receipt receipt
            ON receipt.receipt_id = event.manager_receipt_id
           AND receipt.operation_id = event.source_command_id
           AND receipt.resource_kind = event.resource_kind
           AND receipt.resource_id = event.resource_id
           AND receipt.resource_generation = event.resource_generation
           AND receipt.event_type = event.event_type
           AND receipt.owner_state = event.owner_state
           AND receipt.correlation_id = event.correlation_id
           AND receipt.actor_id = event.actor_id
           AND receipt.payload_digest = event.details_digest
         WHERE receipt.receipt_id IS NULL
    ) OR EXISTS (
        SELECT 1 FROM infrastructure.host_environment_catalog master
         WHERE NOT EXISTS (
            SELECT 1 FROM infrastructure.resource_catalog_event event
            JOIN infrastructure.resource_catalog_manager_receipt receipt
              ON receipt.receipt_id = event.manager_receipt_id
             WHERE event.resource_kind = 'host_environment'
               AND event.resource_id = master.host_environment_id
               AND event.resource_generation = master.generation
               AND event.event_type = 'registered'
         )
    ) OR EXISTS (
        SELECT 1 FROM infrastructure.multipass_instance_catalog master
         WHERE NOT EXISTS (
            SELECT 1 FROM infrastructure.resource_catalog_event event
            JOIN infrastructure.resource_catalog_manager_receipt receipt
              ON receipt.receipt_id = event.manager_receipt_id
             WHERE event.resource_kind = 'multipass_instance'
               AND event.resource_id = master.multipass_instance_id
               AND event.resource_generation = master.generation
               AND event.event_type = 'registered'
         )
    ) OR EXISTS (
        SELECT 1 FROM infrastructure.k3s_cluster_catalog master
         WHERE NOT EXISTS (
            SELECT 1 FROM infrastructure.resource_catalog_event event
            JOIN infrastructure.resource_catalog_manager_receipt receipt
              ON receipt.receipt_id = event.manager_receipt_id
             WHERE event.resource_kind = 'k3s_cluster'
               AND event.resource_id = master.k3s_cluster_id
               AND event.resource_generation = master.generation
               AND event.event_type = 'registered'
         )
    ) OR EXISTS (
        SELECT 1 FROM infrastructure.k3s_cluster_instance_membership master
         WHERE NOT EXISTS (
            SELECT 1 FROM infrastructure.resource_catalog_event event
            JOIN infrastructure.resource_catalog_manager_receipt receipt
              ON receipt.receipt_id = event.manager_receipt_id
             WHERE event.resource_kind = 'cluster_instance_membership'
               AND event.resource_id = master.membership_id
               AND event.resource_generation = master.generation
               AND event.event_type = 'registered'
         )
    ) OR EXISTS (
        SELECT 1 FROM infrastructure.kubernetes_node_catalog master
         WHERE NOT EXISTS (
            SELECT 1 FROM infrastructure.resource_catalog_event event
            JOIN infrastructure.resource_catalog_manager_receipt receipt
              ON receipt.receipt_id = event.manager_receipt_id
             WHERE event.resource_kind = 'kubernetes_node'
               AND event.resource_id = master.kubernetes_node_id
               AND event.resource_generation = master.generation
               AND event.event_type = 'registered'
         )
    ) THEN
        RAISE EXCEPTION 'INFRA-2C2 refuses to adopt unsigned resource catalog facts; perform a separately authorized signed import first';
    END IF;
END;
$$;

ALTER TABLE infrastructure.resource_catalog_event
    ALTER COLUMN manager_receipt_id SET NOT NULL;
ALTER TABLE infrastructure.resource_catalog_event
    DROP CONSTRAINT IF EXISTS resource_catalog_event_manager_receipt_fk;
ALTER TABLE infrastructure.resource_catalog_event
    ADD CONSTRAINT resource_catalog_event_manager_receipt_fk
    FOREIGN KEY (manager_receipt_id)
    REFERENCES infrastructure.resource_catalog_manager_receipt (receipt_id)
    ON DELETE RESTRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_multipass_instance_catalog_current_host_name
    ON infrastructure.multipass_instance_catalog
       (host_environment_id, host_generation, provider_instance_name)
    WHERE valid_to IS NULL;

CREATE OR REPLACE FUNCTION infrastructure.validate_resource_catalog_manager_receipt()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    receipt infrastructure.resource_catalog_manager_receipt%ROWTYPE;
BEGIN
    SELECT * INTO receipt
      FROM infrastructure.resource_catalog_manager_receipt
     WHERE receipt_id = NEW.manager_receipt_id;
    IF NOT FOUND
       OR receipt.operation_id <> NEW.source_command_id
       OR receipt.resource_kind <> NEW.resource_kind
       OR receipt.resource_id <> NEW.resource_id
       OR receipt.resource_generation <> NEW.resource_generation
       OR receipt.event_type <> NEW.event_type
       OR receipt.owner_state <> NEW.owner_state
       OR receipt.correlation_id <> NEW.correlation_id
       OR receipt.actor_id <> NEW.actor_id
       OR receipt.payload_digest IS DISTINCT FROM NEW.details_digest THEN
        RAISE EXCEPTION 'resource catalog event must match its immutable signed Manager receipt';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.require_resource_catalog_receipt_event()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
          FROM infrastructure.resource_catalog_event event
         WHERE event.manager_receipt_id = NEW.receipt_id
           AND event.source_command_id = NEW.operation_id
           AND event.resource_kind = NEW.resource_kind
           AND event.resource_id = NEW.resource_id
           AND event.resource_generation = NEW.resource_generation
           AND event.event_type = NEW.event_type
           AND event.owner_state = NEW.owner_state
           AND event.correlation_id = NEW.correlation_id
           AND event.actor_id = NEW.actor_id
           AND event.details_digest = NEW.payload_digest
    ) THEN
        RAISE EXCEPTION 'Manager receipt must commit with its matching resource catalog event';
    END IF;
    RETURN NULL;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.reject_resource_catalog_receipt_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'resource catalog Manager receipts are append-only';
END;
$$;

DROP TRIGGER IF EXISTS a_validate_resource_catalog_manager_receipt ON infrastructure.resource_catalog_event;
CREATE TRIGGER a_validate_resource_catalog_manager_receipt
    BEFORE INSERT ON infrastructure.resource_catalog_event
    FOR EACH ROW EXECUTE FUNCTION infrastructure.validate_resource_catalog_manager_receipt();

DROP TRIGGER IF EXISTS require_resource_catalog_receipt_event ON infrastructure.resource_catalog_manager_receipt;
CREATE CONSTRAINT TRIGGER require_resource_catalog_receipt_event
    AFTER INSERT ON infrastructure.resource_catalog_manager_receipt
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION infrastructure.require_resource_catalog_receipt_event();

ALTER TABLE infrastructure.resource_catalog_manager_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE infrastructure.resource_catalog_manager_receipt FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS resource_catalog_manager_receipt_infrastructure_owner
    ON infrastructure.resource_catalog_manager_receipt;
CREATE POLICY resource_catalog_manager_receipt_infrastructure_owner
    ON infrastructure.resource_catalog_manager_receipt
    TO star_infrastructure_owner USING (true) WITH CHECK (true);
REVOKE ALL ON TABLE infrastructure.resource_catalog_manager_receipt FROM PUBLIC;
REVOKE ALL ON TABLE infrastructure.resource_catalog_manager_receipt FROM star_infrastructure_owner;
GRANT SELECT, INSERT ON TABLE infrastructure.resource_catalog_manager_receipt TO star_infrastructure_owner;

DROP TRIGGER IF EXISTS reject_resource_catalog_manager_receipt_mutation
    ON infrastructure.resource_catalog_manager_receipt;
CREATE TRIGGER reject_resource_catalog_manager_receipt_mutation
    BEFORE UPDATE OR DELETE ON infrastructure.resource_catalog_manager_receipt
    FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_resource_catalog_receipt_mutation();
DROP TRIGGER IF EXISTS reject_resource_catalog_manager_receipt_truncate
    ON infrastructure.resource_catalog_manager_receipt;
CREATE TRIGGER reject_resource_catalog_manager_receipt_truncate
    BEFORE TRUNCATE ON infrastructure.resource_catalog_manager_receipt
    FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_resource_catalog_receipt_mutation();

COMMENT ON TABLE infrastructure.resource_catalog_manager_receipt IS
    'Transaction: append-only signed evidence verified by the trusted Rust Infrastructure owner before registration; stores no provider output or secret.';
COMMENT ON COLUMN infrastructure.resource_catalog_event.manager_receipt_id IS
    'Required immutable Manager receipt identity; Rust verifies Ed25519 before the owner-local transaction.';

COMMIT;
