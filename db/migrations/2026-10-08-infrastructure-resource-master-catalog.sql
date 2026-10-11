-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-08-infrastructure-resource-master-catalog.sql"})
-- MERGE (schema:Schema {id:"infrastructure"})
-- MERGE (host:Table {id:"infrastructure.host_environment_catalog"})
-- MERGE (instance:Table {id:"infrastructure.multipass_instance_catalog"})
-- MERGE (cluster:Table {id:"infrastructure.k3s_cluster_catalog"})
-- MERGE (membership:Table {id:"infrastructure.k3s_cluster_instance_membership"})
-- MERGE (node:Table {id:"infrastructure.kubernetes_node_catalog"})
-- MERGE (event:Table {id:"infrastructure.resource_catalog_event"})
-- MERGE (master_guard:Function {id:"infrastructure.guard_resource_master_revision"})
-- MERGE (event_guard:Function {id:"infrastructure.validate_resource_catalog_event"})
-- MERGE (registration_guard:Function {id:"infrastructure.require_resource_master_registration"})
-- MERGE (worm_guard:Function {id:"infrastructure.reject_resource_catalog_fact_mutation"})
-- MERGE (self)-[:CREATES]->(schema)
-- MERGE (self)-[:DEFINES]->(host)
-- MERGE (self)-[:DEFINES]->(instance)
-- MERGE (self)-[:DEFINES]->(cluster)
-- MERGE (self)-[:DEFINES]->(membership)
-- MERGE (self)-[:DEFINES]->(node)
-- MERGE (self)-[:DEFINES]->(event)
-- MERGE (self)-[:DEFINES]->(master_guard)
-- MERGE (self)-[:DEFINES]->(event_guard)
-- MERGE (self)-[:DEFINES]->(registration_guard)
-- MERGE (self)-[:DEFINES]->(worm_guard)
-- MERGE (self)-[:CONSTRAINS]->(membership)
-- MERGE (self)-[:CONSTRAINS]->(node)
-- MERGE (instance)-[:REFERENCES]->(host)
-- MERGE (cluster)-[:REFERENCES]->(host)
-- MERGE (membership)-[:REFERENCES]->(cluster)
-- MERGE (membership)-[:REFERENCES]->(instance)
-- MERGE (node)-[:REFERENCES]->(membership)
-- MERGE (master_guard)-[:GUARDS]->(host)
-- MERGE (master_guard)-[:GUARDS]->(instance)
-- MERGE (master_guard)-[:GUARDS]->(cluster)
-- MERGE (master_guard)-[:GUARDS]->(membership)
-- MERGE (master_guard)-[:GUARDS]->(node)
-- MERGE (event_guard)-[:VALIDATES]->(event)
-- MERGE (registration_guard)-[:VALIDATES]->(host)
-- MERGE (registration_guard)-[:READS]->(event)
-- MERGE (registration_guard)-[:VALIDATES]->(instance)
-- MERGE (registration_guard)-[:VALIDATES]->(cluster)
-- MERGE (registration_guard)-[:VALIDATES]->(membership)
-- MERGE (registration_guard)-[:VALIDATES]->(node)
-- MERGE (worm_guard)-[:GUARDS]->(event)
-- @endcypher

-- INFRA-2C1 Master/Transaction persistence substrate.
-- Only trusted Infrastructure-owner registrations belong here.
-- External Multipass inventory remains read-only and is not claimed by this catalog.
BEGIN;

CREATE SCHEMA IF NOT EXISTS infrastructure;

CREATE TABLE IF NOT EXISTS infrastructure.host_environment_catalog (
    host_environment_id UUID NOT NULL,
    generation BIGINT NOT NULL CHECK (generation > 0),
    provider_kind VARCHAR(24) NOT NULL CHECK (provider_kind = 'multipass_local'),
    host_os VARCHAR(16) NOT NULL CHECK (host_os IN ('windows','linux','macos')),
    host_arch VARCHAR(32) NOT NULL CHECK (host_arch ~ '^[A-Za-z0-9_-]{1,32}$'),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (host_environment_id, generation),
    CHECK (host_environment_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_host_environment_catalog_current
    ON infrastructure.host_environment_catalog (host_environment_id)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS infrastructure.multipass_instance_catalog (
    multipass_instance_id UUID NOT NULL,
    generation BIGINT NOT NULL CHECK (generation > 0),
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    provider_instance_name VARCHAR(63) NOT NULL
        CHECK (provider_instance_name ~ '^[a-z0-9][a-z0-9-]{0,62}$'),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (multipass_instance_id, generation),
    UNIQUE (multipass_instance_id, generation, host_environment_id, host_generation),
    FOREIGN KEY (host_environment_id, host_generation)
        REFERENCES infrastructure.host_environment_catalog (host_environment_id, generation)
        ON DELETE RESTRICT,
    CHECK (multipass_instance_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_multipass_instance_catalog_current
    ON infrastructure.multipass_instance_catalog (multipass_instance_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_multipass_instance_catalog_host_name
    ON infrastructure.multipass_instance_catalog
       (host_environment_id, host_generation, provider_instance_name);

CREATE TABLE IF NOT EXISTS infrastructure.k3s_cluster_catalog (
    k3s_cluster_id UUID NOT NULL,
    generation BIGINT NOT NULL CHECK (generation > 0),
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    provider_kind VARCHAR(24) NOT NULL CHECK (provider_kind = 'multipass_local'),
    cluster_identity_digest BYTEA NOT NULL CHECK (octet_length(cluster_identity_digest) = 32),
    k3s_version VARCHAR(64) NOT NULL
        CHECK (k3s_version ~ '^[A-Za-z0-9.+_-]{1,64}$'),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (k3s_cluster_id, generation),
    UNIQUE (k3s_cluster_id, generation, host_environment_id, host_generation),
    FOREIGN KEY (host_environment_id, host_generation)
        REFERENCES infrastructure.host_environment_catalog (host_environment_id, generation)
        ON DELETE RESTRICT,
    CHECK (k3s_cluster_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_k3s_cluster_catalog_current
    ON infrastructure.k3s_cluster_catalog (k3s_cluster_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_k3s_cluster_catalog_host
    ON infrastructure.k3s_cluster_catalog (host_environment_id, host_generation);

CREATE TABLE IF NOT EXISTS infrastructure.k3s_cluster_instance_membership (
    membership_id UUID NOT NULL,
    generation BIGINT NOT NULL CHECK (generation > 0),
    k3s_cluster_id UUID NOT NULL,
    k3s_cluster_generation BIGINT NOT NULL CHECK (k3s_cluster_generation > 0),
    multipass_instance_id UUID NOT NULL,
    multipass_instance_generation BIGINT NOT NULL CHECK (multipass_instance_generation > 0),
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    node_role VARCHAR(8) NOT NULL CHECK (node_role IN ('server','agent')),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (membership_id, generation),
    UNIQUE (
        membership_id, generation, k3s_cluster_id, k3s_cluster_generation,
        multipass_instance_id, multipass_instance_generation, host_environment_id, host_generation
    ),
    FOREIGN KEY (k3s_cluster_id, k3s_cluster_generation, host_environment_id, host_generation)
        REFERENCES infrastructure.k3s_cluster_catalog
            (k3s_cluster_id, generation, host_environment_id, host_generation)
        ON DELETE RESTRICT,
    FOREIGN KEY (multipass_instance_id, multipass_instance_generation, host_environment_id, host_generation)
        REFERENCES infrastructure.multipass_instance_catalog
            (multipass_instance_id, generation, host_environment_id, host_generation)
        ON DELETE RESTRICT,
    CHECK (membership_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_k3s_membership_catalog_current
    ON infrastructure.k3s_cluster_instance_membership (membership_id)
    WHERE valid_to IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_k3s_membership_instance_current
    ON infrastructure.k3s_cluster_instance_membership (multipass_instance_id)
    WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_k3s_membership_cluster
    ON infrastructure.k3s_cluster_instance_membership
       (k3s_cluster_id, k3s_cluster_generation, valid_to);
CREATE INDEX IF NOT EXISTS idx_k3s_membership_instance
    ON infrastructure.k3s_cluster_instance_membership
       (multipass_instance_id, multipass_instance_generation, valid_to);

CREATE TABLE IF NOT EXISTS infrastructure.kubernetes_node_catalog (
    kubernetes_node_id UUID NOT NULL,
    generation BIGINT NOT NULL CHECK (generation > 0),
    membership_id UUID NOT NULL,
    membership_generation BIGINT NOT NULL CHECK (membership_generation > 0),
    k3s_cluster_id UUID NOT NULL,
    k3s_cluster_generation BIGINT NOT NULL CHECK (k3s_cluster_generation > 0),
    multipass_instance_id UUID NOT NULL,
    multipass_instance_generation BIGINT NOT NULL CHECK (multipass_instance_generation > 0),
    host_environment_id UUID NOT NULL,
    host_generation BIGINT NOT NULL CHECK (host_generation > 0),
    kubernetes_node_uid VARCHAR(128) NOT NULL
        CHECK (kubernetes_node_uid ~ '^[A-Za-z0-9._:-]{1,128}$'),
    kubernetes_node_name VARCHAR(253) NOT NULL
        CHECK (kubernetes_node_name ~ '^[a-z0-9]([-a-z0-9.]{0,251}[a-z0-9])?$'),
    node_role VARCHAR(8) NOT NULL CHECK (node_role IN ('server','agent')),
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    valid_from TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    valid_to TIMESTAMPTZ,
    PRIMARY KEY (kubernetes_node_id, generation),
    FOREIGN KEY (
        membership_id, membership_generation, k3s_cluster_id, k3s_cluster_generation,
        multipass_instance_id, multipass_instance_generation, host_environment_id, host_generation
    ) REFERENCES infrastructure.k3s_cluster_instance_membership (
        membership_id, generation, k3s_cluster_id, k3s_cluster_generation,
        multipass_instance_id, multipass_instance_generation, host_environment_id, host_generation
    ) ON DELETE RESTRICT,
    CHECK (kubernetes_node_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from),
    UNIQUE (k3s_cluster_id, k3s_cluster_generation, kubernetes_node_uid)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_kubernetes_node_catalog_current
    ON infrastructure.kubernetes_node_catalog (kubernetes_node_id)
    WHERE valid_to IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_kubernetes_node_name_current
    ON infrastructure.kubernetes_node_catalog
       (k3s_cluster_id, k3s_cluster_generation, kubernetes_node_name)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS infrastructure.resource_catalog_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_kind VARCHAR(32) NOT NULL CHECK (resource_kind IN (
        'host_environment','multipass_instance','k3s_cluster',
        'cluster_instance_membership','kubernetes_node'
    )),
    resource_id UUID NOT NULL,
    resource_generation BIGINT NOT NULL CHECK (resource_generation > 0),
    event_type VARCHAR(16) NOT NULL CHECK (event_type IN ('registered','state_changed','retired')),
    owner_state VARCHAR(12) NOT NULL CHECK (owner_state IN ('active','draining','retired')),
    source_command_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    details_digest BYTEA CHECK (details_digest IS NULL OR octet_length(details_digest) = 32),
    UNIQUE (resource_kind, resource_id, resource_generation, source_command_id),
    CHECK (event_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (resource_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (source_command_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (
        (event_type = 'retired' AND owner_state = 'retired')
        OR (event_type <> 'retired' AND owner_state IN ('active','draining'))
    )
);
CREATE INDEX IF NOT EXISTS idx_resource_catalog_event_resource
    ON infrastructure.resource_catalog_event
       (resource_kind, resource_id, resource_generation, recorded_at);

CREATE OR REPLACE FUNCTION infrastructure.guard_resource_master_revision()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_resource_id UUID;
    requested_generation BIGINT;
    latest_generation BIGINT;
    open_count BIGINT;
    latest_valid_to TIMESTAMPTZ;
    last_owner_state VARCHAR(12);
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'resource catalog Master rows cannot be physically deleted';
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL
           OR NEW.valid_to IS NULL
           OR (to_jsonb(NEW) - 'valid_to') IS DISTINCT FROM (to_jsonb(OLD) - 'valid_to') THEN
            RAISE EXCEPTION 'resource catalog Master rows permit only one-time valid_to closure';
        END IF;
        v_resource_id := (to_jsonb(OLD) ->> TG_ARGV[0])::UUID;
        SELECT owner_state INTO last_owner_state
          FROM infrastructure.resource_catalog_event
         WHERE resource_kind = TG_ARGV[1]
           AND resource_id = v_resource_id
           AND resource_generation = OLD.generation
         ORDER BY recorded_at DESC, event_id DESC
         LIMIT 1;
        IF last_owner_state IS DISTINCT FROM 'retired' THEN
            RAISE EXCEPTION 'resource catalog generation can close only after a retired Transaction fact';
        END IF;
        NEW.valid_to := clock_timestamp();
        RETURN NEW;
    END IF;

    v_resource_id := (to_jsonb(NEW) ->> TG_ARGV[0])::UUID;
    requested_generation := NEW.generation;
    NEW.valid_from := clock_timestamp();
    PERFORM pg_advisory_xact_lock(hashtextextended(TG_TABLE_SCHEMA || '.' || TG_TABLE_NAME || ':' || v_resource_id::TEXT, 0));
    EXECUTE format(
        'SELECT COALESCE(MAX(generation), 0), COUNT(*) FILTER (WHERE valid_to IS NULL), MAX(valid_to) '
        'FROM infrastructure.%I WHERE %I = $1',
        TG_TABLE_NAME, TG_ARGV[0]
    )
    INTO latest_generation, open_count, latest_valid_to
    USING v_resource_id;

    IF requested_generation <> latest_generation + 1
       OR (latest_generation = 0 AND requested_generation <> 1)
       OR (latest_generation > 0 AND (open_count <> 0 OR latest_valid_to IS NULL OR NEW.valid_from <= latest_valid_to)) THEN
        RAISE EXCEPTION 'resource catalog generation must advance once after the prior generation retires and closes';
    END IF;
    IF NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'new resource catalog generation must be current';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.validate_resource_catalog_event()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    resource_exists BOOLEAN;
    previous_state VARCHAR(12);
BEGIN
    NEW.recorded_at := clock_timestamp();
    PERFORM pg_advisory_xact_lock(hashtextextended(
        'infrastructure.resource_catalog_event:' || NEW.resource_kind || ':' || NEW.resource_id::TEXT || ':' || NEW.resource_generation::TEXT,
        0
    ));
    CASE NEW.resource_kind
        WHEN 'host_environment' THEN
            SELECT EXISTS (
                SELECT 1 FROM infrastructure.host_environment_catalog
                 WHERE host_environment_id = NEW.resource_id AND generation = NEW.resource_generation AND valid_to IS NULL
            ) INTO resource_exists;
        WHEN 'multipass_instance' THEN
            SELECT EXISTS (
                SELECT 1 FROM infrastructure.multipass_instance_catalog
                 WHERE multipass_instance_id = NEW.resource_id AND generation = NEW.resource_generation AND valid_to IS NULL
            ) INTO resource_exists;
        WHEN 'k3s_cluster' THEN
            SELECT EXISTS (
                SELECT 1 FROM infrastructure.k3s_cluster_catalog
                 WHERE k3s_cluster_id = NEW.resource_id AND generation = NEW.resource_generation AND valid_to IS NULL
            ) INTO resource_exists;
        WHEN 'cluster_instance_membership' THEN
            SELECT EXISTS (
                SELECT 1 FROM infrastructure.k3s_cluster_instance_membership
                 WHERE membership_id = NEW.resource_id AND generation = NEW.resource_generation AND valid_to IS NULL
            ) INTO resource_exists;
        WHEN 'kubernetes_node' THEN
            SELECT EXISTS (
                SELECT 1 FROM infrastructure.kubernetes_node_catalog
                 WHERE kubernetes_node_id = NEW.resource_id AND generation = NEW.resource_generation AND valid_to IS NULL
            ) INTO resource_exists;
        ELSE
            resource_exists := FALSE;
    END CASE;

    IF resource_exists IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'resource catalog event must reference a current owner-registered generation';
    END IF;

    SELECT owner_state INTO previous_state
      FROM infrastructure.resource_catalog_event
     WHERE resource_kind = NEW.resource_kind
       AND resource_id = NEW.resource_id
       AND resource_generation = NEW.resource_generation
     ORDER BY recorded_at DESC, event_id DESC
     LIMIT 1;

    IF previous_state IS NULL THEN
        IF NEW.event_type <> 'registered' OR NEW.owner_state <> 'active' THEN
            RAISE EXCEPTION 'the first resource catalog event must register the generation as active';
        END IF;
    ELSIF previous_state = 'retired' OR NEW.event_type = 'registered' THEN
        RAISE EXCEPTION 'retired resources are terminal and registration cannot be replayed with a new command';
    ELSIF NEW.event_type = 'state_changed' AND (
        NEW.owner_state NOT IN ('active','draining') OR NEW.owner_state = previous_state
    ) THEN
        RAISE EXCEPTION 'resource state change must change active/draining state';
    ELSIF NEW.event_type = 'retired' AND NEW.owner_state <> 'retired' THEN
        RAISE EXCEPTION 'retirement event must carry retired state';
    ELSIF NEW.event_type NOT IN ('state_changed','retired') THEN
        RAISE EXCEPTION 'unsupported resource catalog event transition';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.require_resource_master_registration()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_resource_id UUID;
BEGIN
    v_resource_id := (to_jsonb(NEW) ->> TG_ARGV[0])::UUID;
    IF NOT EXISTS (
        SELECT 1
          FROM infrastructure.resource_catalog_event
         WHERE resource_kind = TG_ARGV[1]
           AND resource_id = v_resource_id
           AND resource_generation = NEW.generation
           AND event_type = 'registered'
           AND owner_state = 'active'
    ) THEN
        RAISE EXCEPTION 'resource catalog Master generation requires a registered Transaction fact in the same transaction';
    END IF;
    RETURN NULL;
END;
$$;

CREATE OR REPLACE FUNCTION infrastructure.reject_resource_catalog_fact_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'resource catalog Transaction facts are append-only';
END;
$$;

DO $$
DECLARE
    table_name TEXT;
    resource_id_column TEXT;
    resource_kind_value TEXT;
    table_names TEXT[] := ARRAY[
        'host_environment_catalog','multipass_instance_catalog','k3s_cluster_catalog',
        'k3s_cluster_instance_membership','kubernetes_node_catalog','resource_catalog_event'
    ];
BEGIN
    FOREACH table_name IN ARRAY table_names LOOP
        EXECUTE format('ALTER TABLE infrastructure.%I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE infrastructure.%I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format('DROP POLICY IF EXISTS %I ON infrastructure.%I', table_name || '_infrastructure_owner', table_name);
        EXECUTE format(
            'CREATE POLICY %I ON infrastructure.%I TO star_infrastructure_owner USING (true) WITH CHECK (true)',
            table_name || '_infrastructure_owner', table_name
        );
        EXECUTE format('REVOKE ALL ON TABLE infrastructure.%I FROM PUBLIC', table_name);
        EXECUTE format('REVOKE ALL ON TABLE infrastructure.%I FROM star_infrastructure_owner', table_name);

        IF table_name = 'resource_catalog_event' THEN
            EXECUTE 'DROP TRIGGER IF EXISTS validate_resource_catalog_event ON infrastructure.resource_catalog_event';
            EXECUTE 'DROP TRIGGER IF EXISTS reject_resource_catalog_event_mutation ON infrastructure.resource_catalog_event';
            EXECUTE 'DROP TRIGGER IF EXISTS reject_resource_catalog_event_truncate ON infrastructure.resource_catalog_event';
            EXECUTE 'CREATE TRIGGER validate_resource_catalog_event BEFORE INSERT ON infrastructure.resource_catalog_event '
                 || 'FOR EACH ROW EXECUTE FUNCTION infrastructure.validate_resource_catalog_event()';
            EXECUTE 'CREATE TRIGGER reject_resource_catalog_event_mutation BEFORE UPDATE OR DELETE ON infrastructure.resource_catalog_event '
                 || 'FOR EACH ROW EXECUTE FUNCTION infrastructure.reject_resource_catalog_fact_mutation()';
            EXECUTE 'CREATE TRIGGER reject_resource_catalog_event_truncate BEFORE TRUNCATE ON infrastructure.resource_catalog_event '
                 || 'FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_resource_catalog_fact_mutation()';
            EXECUTE 'GRANT SELECT, INSERT ON TABLE infrastructure.resource_catalog_event TO star_infrastructure_owner';
        ELSE
            resource_id_column := CASE table_name
                WHEN 'host_environment_catalog' THEN 'host_environment_id'
                WHEN 'multipass_instance_catalog' THEN 'multipass_instance_id'
                WHEN 'k3s_cluster_catalog' THEN 'k3s_cluster_id'
                WHEN 'k3s_cluster_instance_membership' THEN 'membership_id'
                WHEN 'kubernetes_node_catalog' THEN 'kubernetes_node_id'
            END;
            resource_kind_value := CASE table_name
                WHEN 'host_environment_catalog' THEN 'host_environment'
                WHEN 'multipass_instance_catalog' THEN 'multipass_instance'
                WHEN 'k3s_cluster_catalog' THEN 'k3s_cluster'
                WHEN 'k3s_cluster_instance_membership' THEN 'cluster_instance_membership'
                WHEN 'kubernetes_node_catalog' THEN 'kubernetes_node'
            END;
            EXECUTE format('DROP TRIGGER IF EXISTS %I ON infrastructure.%I', 'guard_' || table_name || '_revision', table_name);
            EXECUTE format('DROP TRIGGER IF EXISTS %I ON infrastructure.%I', 'require_' || table_name || '_registration', table_name);
            EXECUTE format('DROP TRIGGER IF EXISTS %I ON infrastructure.%I', 'reject_' || table_name || '_truncate', table_name);
            EXECUTE format(
                'CREATE TRIGGER %I BEFORE INSERT OR UPDATE OR DELETE ON infrastructure.%I '
                'FOR EACH ROW EXECUTE FUNCTION infrastructure.guard_resource_master_revision(%L, %L)', 'guard_' || table_name || '_revision', table_name, resource_id_column, resource_kind_value
            );
            EXECUTE format(
                'CREATE CONSTRAINT TRIGGER %I AFTER INSERT ON infrastructure.%I DEFERRABLE INITIALLY DEFERRED '
                'FOR EACH ROW EXECUTE FUNCTION infrastructure.require_resource_master_registration(%L, %L)',
                'require_' || table_name || '_registration', table_name, resource_id_column, resource_kind_value
            );
            EXECUTE format(
                'CREATE TRIGGER %I BEFORE TRUNCATE ON infrastructure.%I '
                'FOR EACH STATEMENT EXECUTE FUNCTION infrastructure.reject_resource_catalog_fact_mutation()',
                'reject_' || table_name || '_truncate', table_name
            );
            EXECUTE format(
                'GRANT SELECT, INSERT, UPDATE ON TABLE infrastructure.%I TO star_infrastructure_owner',
                table_name
            );
        END IF;
    END LOOP;
END;
$$;

COMMENT ON TABLE infrastructure.host_environment_catalog IS
    'Master: one Multipass-backed host-environment generation; global to the trusted Infrastructure owner, no tenant payload.';
COMMENT ON TABLE infrastructure.multipass_instance_catalog IS
    'Master: one owner-managed Multipass instance generation; provider names are lookup attributes, not ownership proof.';
COMMENT ON TABLE infrastructure.k3s_cluster_catalog IS
    'Master: one owner-managed Multipass-backed K3s cluster generation; identity digest is not readiness or authorization.';
COMMENT ON TABLE infrastructure.k3s_cluster_instance_membership IS
    'Master: SCD2 membership generation joining one K3s cluster and one exact Multipass instance on the same host generation.';
COMMENT ON TABLE infrastructure.kubernetes_node_catalog IS
    'Master: one Kubernetes node identity generation tied to its exact cluster, instance, and membership generations.';
COMMENT ON TABLE infrastructure.resource_catalog_event IS
    'Transaction: append-only owner lifecycle facts; never stores raw CLI output, credentials, host paths, or tenant claims.';

COMMIT;
