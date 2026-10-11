-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-08-infrastructure-resource-catalog-lifecycle.sql"})
-- MERGE (event:Table {id:"infrastructure.resource_catalog_event",class:"Transaction"})
-- MERGE (host:Table {id:"infrastructure.host_environment_catalog",class:"Master"})
-- MERGE (instance:Table {id:"infrastructure.multipass_instance_catalog",class:"Master"})
-- MERGE (cluster:Table {id:"infrastructure.k3s_cluster_catalog",class:"Master"})
-- MERGE (membership:Table {id:"infrastructure.k3s_cluster_instance_membership",class:"Master"})
-- MERGE (node:Table {id:"infrastructure.kubernetes_node_catalog",class:"Master"})
-- MERGE (guard:Function {id:"infrastructure.require_resource_catalog_retirement_close"})
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:CONSTRAINS]->(event)
-- MERGE (guard)-[:READS]->(host)
-- MERGE (guard)-[:READS]->(instance)
-- MERGE (guard)-[:READS]->(cluster)
-- MERGE (guard)-[:READS]->(membership)
-- MERGE (guard)-[:READS]->(node)
-- MERGE (guard)-[:CONSTRAINS]->(host)
-- MERGE (guard)-[:CONSTRAINS]->(instance)
-- MERGE (guard)-[:CONSTRAINS]->(cluster)
-- MERGE (guard)-[:CONSTRAINS]->(membership)
-- MERGE (guard)-[:CONSTRAINS]->(node)
-- @endcypher

-- INFRA-2C3: a signed retired event and the matching SCD2 Master close must commit together.
BEGIN;

CREATE OR REPLACE FUNCTION infrastructure.require_resource_catalog_retirement_close()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    master_table TEXT;
    identity_column TEXT;
    generation_closed BOOLEAN;
BEGIN
    IF NEW.event_type <> 'retired' THEN
        RETURN NULL;
    END IF;

    CASE NEW.resource_kind
        WHEN 'host_environment' THEN
            master_table := 'host_environment_catalog';
            identity_column := 'host_environment_id';
        WHEN 'multipass_instance' THEN
            master_table := 'multipass_instance_catalog';
            identity_column := 'multipass_instance_id';
        WHEN 'k3s_cluster' THEN
            master_table := 'k3s_cluster_catalog';
            identity_column := 'k3s_cluster_id';
        WHEN 'cluster_instance_membership' THEN
            master_table := 'k3s_cluster_instance_membership';
            identity_column := 'membership_id';
        WHEN 'kubernetes_node' THEN
            master_table := 'kubernetes_node_catalog';
            identity_column := 'kubernetes_node_id';
        ELSE
            RAISE EXCEPTION 'unsupported resource catalog kind for retirement';
    END CASE;

    EXECUTE format(
        'SELECT valid_to IS NOT NULL FROM infrastructure.%I WHERE %I = $1 AND generation = $2',
        master_table,
        identity_column
    )
    INTO generation_closed
    USING NEW.resource_id, NEW.resource_generation;

    IF generation_closed IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'retired resource catalog event must close its exact Master generation in the same transaction';
    END IF;
    RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS require_resource_catalog_retirement_close
    ON infrastructure.resource_catalog_event;
CREATE CONSTRAINT TRIGGER require_resource_catalog_retirement_close
    AFTER INSERT ON infrastructure.resource_catalog_event
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION infrastructure.require_resource_catalog_retirement_close();

COMMENT ON FUNCTION infrastructure.require_resource_catalog_retirement_close() IS
    'Transaction: every retired resource event must close the exact SCD2 Master generation before commit.';

COMMIT;
