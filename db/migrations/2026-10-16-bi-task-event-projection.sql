-- @cypher schema=1 source_sha256=4d721dc2cb9b7d7cf8aa612324d1eb0c7d069e0d9b22e6a09cda21befa4d0da7
-- MERGE (self:File {path:"db/migrations/2026-10-16-bi-task-event-projection.sql"})
-- MERGE (project:Database {id:"bi.project_task_event_payload_v1",kind:"stored_function"})
-- MERGE (commit:Database {id:"bi.commit_task_event_v2",kind:"stored_function"})
-- MERGE (legacy:Database {id:"bi.commit_task_event_v1",kind:"stored_function"})
-- MERGE (oldFact:Table {id:"bi.task_event_fact_raw",kind:"table"})
-- MERGE (fact:Table {id:"bi.task_event_fact",kind:"table"})
-- MERGE (inbox:Table {id:"bi.task_event_inbox",kind:"table"})
-- MERGE (self)-[:DEFINES]->(project)
-- MERGE (self)-[:DEFINES]->(commit)
-- MERGE (self)-[:READS]->(oldFact)
-- MERGE (self)-[:WRITES]->(fact)
-- MERGE (self)-[:READS]->(inbox)
-- MERGE (project)-[:READS]->(oldFact)
-- MERGE (commit)-[:CALLS]->(project)
-- MERGE (commit)-[:CALLS]->(legacy)
-- MERGE (commit)-[:WRITES]->(fact)
-- MERGE (fact)-[:DEPENDS_ON]->(inbox)
-- @endcypher

-- Replace BI's retained raw Task payload with a versioned allowlist projection.
-- The original source Outbox remains authoritative; BI never stores free-form task content.
BEGIN;

CREATE OR REPLACE FUNCTION bi.project_task_event_payload_v1(
    p_tenant_id UUID,
    p_project_id UUID,
    p_engineering_run_id UUID,
    p_work_item_id UUID,
    p_aggregate_type VARCHAR,
    p_aggregate_version INTEGER,
    p_event_type VARCHAR,
    p_schema_version INTEGER,
    p_payload JSONB
)
RETURNS JSONB
LANGUAGE plpgsql IMMUTABLE
SET search_path = pg_catalog
AS $$
DECLARE
    lifecycle_status TEXT;
    review_state TEXT;
    from_status TEXT;
    to_status TEXT;
BEGIN
    IF p_payload IS NULL OR jsonb_typeof(p_payload) <> 'object'
       OR octet_length(p_payload::TEXT) > 8192
       OR p_work_item_id IS NULL
       OR p_payload ->> 'work_item_id' IS DISTINCT FROM p_work_item_id::TEXT THEN
        RAISE EXCEPTION 'invalid BI Task projection source' USING ERRCODE = '23514';
    END IF;

    IF p_schema_version = 1 AND p_aggregate_type = 'metadata'
       AND p_event_type IN ('task.created', 'task.updated', 'task.owner_changed') THEN
        IF p_payload ->> 'project_id' IS DISTINCT FROM p_project_id::TEXT
           OR p_payload ->> 'engineering_run_id' IS DISTINCT FROM p_engineering_run_id::TEXT
           OR p_payload ->> 'metadata_version' IS DISTINCT FROM p_aggregate_version::TEXT THEN
            RAISE EXCEPTION 'BI Task v1 projection scope mismatch' USING ERRCODE = '23514';
        END IF;
        RETURN jsonb_build_object(
            'projection_version', 1,
            'work_item_id', p_work_item_id,
            'project_id', p_project_id,
            'engineering_run_id', p_engineering_run_id,
            'metadata_version', p_aggregate_version,
            'event_type', p_event_type,
            'owner_changed', p_event_type = 'task.owner_changed'
        );
    ELSIF p_schema_version = 2 AND p_aggregate_type = 'metadata'
       AND p_event_type IN ('task.created', 'task.updated', 'task.owner_changed') THEN
        IF p_payload ->> 'schema_version' IS DISTINCT FROM '2'
           OR p_payload #>> '{owner,tenant_id}' IS DISTINCT FROM p_tenant_id::TEXT
           OR p_payload #>> '{owner,project_id}' IS DISTINCT FROM p_project_id::TEXT
           OR p_payload #>> '{owner,engineering_run_id}' IS DISTINCT FROM p_engineering_run_id::TEXT
           OR p_payload ->> 'metadata_version' IS DISTINCT FROM p_aggregate_version::TEXT THEN
            RAISE EXCEPTION 'BI Task v2 projection scope mismatch' USING ERRCODE = '23514';
        END IF;
        RETURN jsonb_build_object(
            'projection_version', 1,
            'schema_version', 2,
            'work_item_id', p_work_item_id,
            'owner', jsonb_build_object(
                'tenant_id', p_tenant_id,
                'project_id', p_project_id,
                'engineering_run_id', p_engineering_run_id
            ),
            'metadata_version', p_aggregate_version,
            'event_type', p_event_type,
            'owner_changed', p_event_type = 'task.owner_changed'
        );
    ELSIF p_schema_version = 3 AND p_aggregate_type = 'lifecycle'
       AND p_event_type = 'task.lifecycle.changed' THEN
        IF p_payload ->> 'schema_version' IS DISTINCT FROM '3'
           OR p_payload #>> '{aggregate,type}' IS DISTINCT FROM p_aggregate_type
           OR p_payload #>> '{aggregate,version}' IS DISTINCT FROM p_aggregate_version::TEXT
           OR p_payload #>> '{owner,tenant_id}' IS DISTINCT FROM p_tenant_id::TEXT
           OR p_payload #>> '{owner,project_id}' IS DISTINCT FROM p_project_id::TEXT
           OR p_payload #>> '{owner,engineering_run_id}' IS DISTINCT FROM p_engineering_run_id::TEXT
           OR p_payload #>> '{change,event_type}' IS DISTINCT FROM p_event_type THEN
            RAISE EXCEPTION 'BI Task v3 projection scope mismatch' USING ERRCODE = '23514';
        END IF;

        lifecycle_status := p_payload #>> '{lifecycle,status}';
        review_state := p_payload #>> '{lifecycle,review_state}';
        from_status := p_payload #>> '{change,from_status}';
        to_status := p_payload #>> '{change,to_status}';
        IF lifecycle_status IS NULL
           OR lifecycle_status NOT IN ('pending','claimed','in_progress','completed','failed','cancelled')
           OR review_state IS NULL
           OR review_state NOT IN ('none','pending_review','accepted','rejected')
           OR (from_status IS NOT NULL AND from_status NOT IN ('pending','claimed','in_progress','completed','failed','cancelled'))
           OR (to_status IS NOT NULL AND to_status NOT IN ('pending','claimed','in_progress','completed','failed','cancelled')) THEN
            RAISE EXCEPTION 'BI Task v3 projection state is unsupported' USING ERRCODE = '23514';
        END IF;
        RETURN jsonb_build_object(
            'projection_version', 1,
            'schema_version', 3,
            'aggregate', jsonb_build_object('type', p_aggregate_type, 'version', p_aggregate_version),
            'work_item_id', p_work_item_id,
            'owner', jsonb_build_object(
                'tenant_id', p_tenant_id,
                'project_id', p_project_id,
                'engineering_run_id', p_engineering_run_id
            ),
            'event_type', p_event_type,
            'lifecycle', jsonb_build_object(
                'status', lifecycle_status,
                'review_state', review_state
            ),
            'change', jsonb_build_object(
                'event_type', p_event_type,
                'from_status', from_status,
                'to_status', to_status
            ),
            'owner_changed', FALSE
        );
    END IF;

    RAISE EXCEPTION 'unsupported BI Task projection contract' USING ERRCODE = '23514';
END;
$$;
ALTER FUNCTION bi.project_task_event_payload_v1(
    UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER, JSONB
) OWNER TO star_bi_owner;

CREATE TABLE bi.task_event_fact_projection_v1 (
    tenant_id UUID NOT NULL,
    consumer_id VARCHAR(64) NOT NULL,
    event_id UUID NOT NULL,
    project_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    aggregate_type VARCHAR(16) NOT NULL,
    aggregate_version INTEGER NOT NULL CHECK (aggregate_version > 0),
    event_type VARCHAR(64) NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version BETWEEN 1 AND 3),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL CHECK (
        jsonb_typeof(payload) = 'object'
        AND octet_length(payload::TEXT) <= 8192
        AND payload ->> 'projection_version' = '1'
    ),
    wire_sha256 BYTEA NOT NULL CHECK (octet_length(wire_sha256) = 32),
    PRIMARY KEY (tenant_id, consumer_id, event_id),
    FOREIGN KEY (tenant_id, consumer_id, event_id)
        REFERENCES bi.task_event_inbox (tenant_id, consumer_id, event_id) ON DELETE RESTRICT,
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK ((schema_version IN (1, 2) AND aggregate_type = 'metadata'
            AND event_type IN ('task.created', 'task.updated', 'task.owner_changed'))
        OR (schema_version = 3 AND aggregate_type = 'lifecycle'
            AND event_type = 'task.lifecycle.changed'))
);

INSERT INTO bi.task_event_fact_projection_v1 (
    tenant_id, consumer_id, event_id, project_id, engineering_run_id, work_item_id,
    aggregate_type, aggregate_version, event_type, schema_version,
    actor_id, correlation_id, occurred_at, payload, wire_sha256
)
SELECT f.tenant_id, f.consumer_id, f.event_id, f.project_id, f.engineering_run_id,
       f.work_item_id, f.aggregate_type, f.aggregate_version, f.event_type,
       f.schema_version, f.actor_id, f.correlation_id, f.occurred_at,
       bi.project_task_event_payload_v1(
           f.tenant_id, f.project_id, f.engineering_run_id, f.work_item_id,
           f.aggregate_type, f.aggregate_version, f.event_type, f.schema_version, f.payload
       ),
       f.wire_sha256
  FROM bi.task_event_fact AS f;

ALTER TABLE bi.task_event_fact RENAME TO task_event_fact_raw;
ALTER TABLE bi.task_event_fact_projection_v1 RENAME TO task_event_fact;
DROP TABLE bi.task_event_fact_raw;

ALTER TABLE bi.task_event_fact OWNER TO star_bi_owner;
REVOKE ALL ON bi.task_event_fact FROM PUBLIC, star_bi_runtime;
ALTER TABLE bi.task_event_fact ENABLE ROW LEVEL SECURITY;
ALTER TABLE bi.task_event_fact FORCE ROW LEVEL SECURITY;
CREATE POLICY task_event_fact_owner_tenant ON bi.task_event_fact
    FOR ALL TO star_bi_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
CREATE TRIGGER task_event_fact_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON bi.task_event_fact
    FOR EACH STATEMENT EXECUTE FUNCTION bi.reject_task_event_mutation();
CREATE INDEX task_event_fact_run_time_idx
    ON bi.task_event_fact (tenant_id, project_id, engineering_run_id, occurred_at DESC, event_id);

CREATE OR REPLACE FUNCTION bi.commit_task_event_v2(
    p_tenant_id UUID,
    p_project_id UUID,
    p_engineering_run_id UUID,
    p_event_id UUID,
    p_work_item_id UUID,
    p_aggregate_type VARCHAR,
    p_aggregate_version INTEGER,
    p_event_type VARCHAR,
    p_schema_version INTEGER,
    p_actor_id UUID,
    p_correlation_id UUID,
    p_occurred_at TIMESTAMPTZ,
    p_payload JSONB,
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
    p_route_pin_sha256 BYTEA,
    p_wire_sha256 BYTEA
)
RETURNS TABLE(outcome TEXT, receipt_id UUID, committed_at TIMESTAMPTZ)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    projected_payload JSONB;
BEGIN
    projected_payload := bi.project_task_event_payload_v1(
        p_tenant_id, p_project_id, p_engineering_run_id, p_work_item_id,
        p_aggregate_type, p_aggregate_version, p_event_type, p_schema_version, p_payload
    );
    RETURN QUERY
        SELECT result.outcome, result.receipt_id, result.committed_at
          FROM bi.commit_task_event_v1(
              p_tenant_id, p_project_id, p_engineering_run_id, p_event_id,
              p_work_item_id, p_aggregate_type, p_aggregate_version, p_event_type,
              p_schema_version, p_actor_id, p_correlation_id, p_occurred_at,
              projected_payload, p_sender_service_id, p_receiver_service_id,
              p_transport_provider_id, p_transport_provider_version,
              p_transport_profile_id, p_transport_profile_revision,
              p_transport_profile_digest, p_endpoint_identity, p_destination_id,
              p_destination_generation, p_route_pin_sha256, p_wire_sha256
          ) AS result;
END;
$$;
ALTER FUNCTION bi.commit_task_event_v2(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) OWNER TO star_bi_owner;

REVOKE ALL ON FUNCTION bi.project_task_event_payload_v1(
    UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER, JSONB
) FROM PUBLIC, star_bi_runtime;
GRANT EXECUTE ON FUNCTION bi.project_task_event_payload_v1(
    UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER, JSONB
) TO star_bi_owner;
REVOKE ALL ON FUNCTION bi.commit_task_event_v1(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) FROM PUBLIC, star_bi_runtime;
REVOKE ALL ON FUNCTION bi.commit_task_event_v2(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION bi.commit_task_event_v2(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) TO star_bi_runtime;

COMMENT ON TABLE bi.task_event_fact IS
    'T: immutable BI Task fact with versioned allowlist projection only; raw event payload, task text, branch/worktree labels and unbounded attributes are not retained.';
COMMENT ON COLUMN bi.task_event_fact.payload IS
    'Versioned allowlist projection generated inside the BI owner function; never the raw source event payload.';
COMMENT ON INDEX bi.task_event_fact_run_time_idx IS
    'Supports bounded tenant/Project/Run time-window reads without loading the full event history.';
COMMENT ON FUNCTION bi.commit_task_event_v1(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) IS 'Internal BI owner persistence function; runtime callers must use v2 so only an allowlisted projection is stored.';
COMMENT ON FUNCTION bi.commit_task_event_v2(
    UUID, UUID, UUID, UUID, UUID, VARCHAR, INTEGER, VARCHAR, INTEGER,
    UUID, UUID, TIMESTAMPTZ, JSONB, UUID, UUID, VARCHAR, VARCHAR, UUID, BIGINT,
    BYTEA, VARCHAR, VARCHAR, UUID, BYTEA, BYTEA
) IS 'Projects the bounded source payload to a strict non-freeform allowlist, then atomically commits the BI Inbox receipt and immutable fact.';

COMMIT;
