-- @cypher schema=1 source_sha256=265d846857f6ff78ed063ce2719240e99072f7c5d893a22a2a75d34274a7001f
-- MERGE (self:File {path:"db/migrations/2026-10-08-infrastructure-owner-runtime-grants.sql"})
-- MERGE (role:DatabaseRole {id:"star_infrastructure_owner"})
-- MERGE (inbox:Table {id:"infrastructure.capacity_demand_inbox"})
-- MERGE (bundle:Table {id:"infrastructure.capacity_reservation_bundle"})
-- MERGE (delivery:Table {id:"infrastructure.capacity_outbox_delivery"})
-- MERGE (receipt:Table {id:"infrastructure.capacity_demand_receipt"})
-- MERGE (event:Table {id:"infrastructure.capacity_reservation_event"})
-- MERGE (outbox:Table {id:"infrastructure.capacity_owner_outbox"})
-- MERGE (self)-[:CONFIGURES]->(role)
-- MERGE (self)-[:CONFIGURES]->(inbox)
-- MERGE (self)-[:CONFIGURES]->(bundle)
-- MERGE (self)-[:CONFIGURES]->(delivery)
-- MERGE (self)-[:CONFIGURES]->(receipt)
-- MERGE (self)-[:CONFIGURES]->(event)
-- MERGE (self)-[:CONFIGURES]->(outbox)
-- @endcypher

-- Provision only a NOLOGIN privilege group. Deployment creates a separate login principal
-- and grants membership through its secret-management/bootstrap path; no credentials belong here.
BEGIN;

DO $role$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_infrastructure_owner') THEN
        EXECUTE 'ALTER ROLE star_infrastructure_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_infrastructure_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
END
$role$;

REVOKE ALL ON SCHEMA infrastructure FROM PUBLIC;
REVOKE ALL ON ALL TABLES IN SCHEMA infrastructure FROM PUBLIC;
REVOKE ALL ON SCHEMA infrastructure FROM star_infrastructure_owner;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA infrastructure FROM star_infrastructure_owner;

GRANT USAGE ON SCHEMA infrastructure TO star_infrastructure_owner;

-- Work rows have bounded lifecycle state and are writable by the owner worker under FORCE RLS.
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE
    infrastructure.capacity_demand_inbox,
    infrastructure.capacity_reservation_bundle,
    infrastructure.capacity_outbox_delivery
TO star_infrastructure_owner;

-- Transaction facts are append-only; triggers remain an independent integrity guard.
GRANT SELECT, INSERT ON TABLE
    infrastructure.capacity_demand_receipt,
    infrastructure.capacity_reservation_event,
    infrastructure.capacity_owner_outbox
TO star_infrastructure_owner;

COMMIT;
