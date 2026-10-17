-- SPDX-License-Identifier: MIT OR Apache-2.0
-- @cypher schema=1 source_sha256=PLACEHOLDER
-- MERGE (self:File {path:"db/migrations/2026-10-17-bi-task-event-query.sql"})
-- MERGE (function:Database {id:"bi.list_task_event_page_v1",kind:"stored_function"})
-- MERGE (fact:Table {id:"bi.task_event_fact",kind:"table"})
-- MERGE (self)-[:DEFINES]->(function)
-- MERGE (function)-[:READS]->(fact)
-- @endcypher

BEGIN;

CREATE OR REPLACE FUNCTION bi.list_task_event_page_v1(
    p_tenant_id UUID,
    p_project_id UUID,
    p_engineering_run_id UUID,
    p_before_occurred_at TIMESTAMPTZ,
    p_before_event_id UUID,
    p_limit INTEGER
)
RETURNS TABLE (
    event_id UUID,
    work_item_id UUID,
    aggregate_type VARCHAR,
    aggregate_version INTEGER,
    event_type VARCHAR,
    schema_version INTEGER,
    occurred_at TIMESTAMPTZ,
    lifecycle_status TEXT,
    review_state TEXT,
    from_status TEXT,
    to_status TEXT,
    owner_changed BOOLEAN
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    scoped_tenant_id UUID;
BEGIN
    IF p_tenant_id IS NULL
       OR p_project_id IS NULL
       OR p_engineering_run_id IS NULL
       OR (p_before_occurred_at IS NULL) <> (p_before_event_id IS NULL)
       OR p_limit IS NULL
       OR p_limit < 1
       OR p_limit > 101 THEN
        RAISE EXCEPTION 'invalid BI Task event page request' USING ERRCODE = '22023';
    END IF;

    scoped_tenant_id := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    IF scoped_tenant_id IS NULL OR scoped_tenant_id <> p_tenant_id THEN
        RAISE EXCEPTION 'BI Task event tenant scope mismatch' USING ERRCODE = '42501';
    END IF;

    RETURN QUERY
    SELECT f.event_id,
           f.work_item_id,
           f.aggregate_type,
           f.aggregate_version,
           f.event_type,
           f.schema_version,
           f.occurred_at,
           f.payload #>> '{lifecycle,status}',
           f.payload #>> '{lifecycle,review_state}',
           f.payload #>> '{change,from_status}',
           f.payload #>> '{change,to_status}',
           COALESCE((f.payload ->> 'owner_changed')::BOOLEAN, FALSE)
      FROM bi.task_event_fact AS f
     WHERE f.tenant_id = p_tenant_id
       AND f.project_id = p_project_id
       AND f.engineering_run_id = p_engineering_run_id
       AND f.consumer_id = 'bi.task-event-fact.v1'
       AND (
            p_before_occurred_at IS NULL
            OR f.occurred_at < p_before_occurred_at
            OR (f.occurred_at = p_before_occurred_at AND f.event_id > p_before_event_id)
       )
     ORDER BY f.occurred_at DESC, f.event_id ASC
     LIMIT p_limit;
END;
$$;

ALTER FUNCTION bi.list_task_event_page_v1(
    UUID, UUID, UUID, TIMESTAMPTZ, UUID, INTEGER
) OWNER TO star_bi_owner;
REVOKE ALL ON FUNCTION bi.list_task_event_page_v1(
    UUID, UUID, UUID, TIMESTAMPTZ, UUID, INTEGER
) FROM PUBLIC, star_bi_runtime;
GRANT EXECUTE ON FUNCTION bi.list_task_event_page_v1(
    UUID, UUID, UUID, TIMESTAMPTZ, UUID, INTEGER
) TO star_bi_runtime;

COMMENT ON FUNCTION bi.list_task_event_page_v1(
    UUID, UUID, UUID, TIMESTAMPTZ, UUID, INTEGER
) IS 'Returns a bounded allowlist projection of observed Task facts for one authorized tenant/Project/Run; completeness and freshness are not inferred.';

COMMIT;
