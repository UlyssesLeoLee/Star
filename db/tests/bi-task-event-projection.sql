-- @cypher schema=1 source_sha256=234f8a5f3c48877a924c62377e5c32af7aeffab3ccbd000dc25c8898a37e48f4
-- MERGE (self:File {path:"db/tests/bi-task-event-projection.sql"})
-- MERGE (test:Symbol {id:"db/tests/bi-task-event-projection.sql::bi_task_event_projection_contract",kind:"test"})
-- MERGE (fact:Table {id:"bi.task_event_fact",kind:"table"})
-- MERGE (self)-[:DEFINES]->(test)
-- MERGE (test)-[:READS]->(fact)
-- @endcypher

DO $$
DECLARE
    migrated JSONB;
    inserted JSONB;
BEGIN
    SELECT payload INTO migrated
      FROM bi.task_event_fact
     WHERE event_id = '00000000-0000-4000-8000-000000000001';
    SELECT payload INTO inserted
      FROM bi.task_event_fact
     WHERE event_id = '44444444-4444-4444-8444-444444444444';

    IF migrated IS NULL OR inserted IS NULL
       OR migrated ->> 'projection_version' <> '1'
       OR inserted ->> 'projection_version' <> '1'
       OR migrated #>> '{lifecycle,status}' <> 'claimed'
       OR migrated #>> '{change,from_status}' <> 'pending'
       OR inserted #>> '{lifecycle,status}' <> 'in_progress'
       OR inserted #>> '{change,to_status}' <> 'in_progress' THEN
        RAISE EXCEPTION 'BI Task projection lost supported event dimensions';
    END IF;

    IF migrated::TEXT LIKE '%LEGACY-RAW-PAYLOAD-SENTINEL%'
       OR inserted::TEXT LIKE '%BI must not retain this free-form value%'
       OR migrated ? 'host_context'
       OR inserted ? 'host_context'
       OR migrated #> '{change,private_note}' IS NOT NULL
       OR inserted #> '{change,private_note}' IS NOT NULL THEN
        RAISE EXCEPTION 'free-form Task content survived into the BI projection';
    END IF;

    IF (SELECT count(*) FROM bi.task_event_fact) <> 2 THEN
        RAISE EXCEPTION 'projection migration changed the immutable BI event count';
    END IF;
END;
$$;
