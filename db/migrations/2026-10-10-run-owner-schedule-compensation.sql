-- Run-owner cleanup before infrastructure demand; never releases external reservations.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-schedule-compensation.sql"})
-- MERGE (f:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-compensation.sql::compensate_schedule_preparation",kind:"function"})
-- MERGE (self)-[:DEFINES]->(f)
-- MERGE (f)-[:READS]->(inbox:Table {id:"multica.schedule_admission_inbox"})
-- MERGE (f)-[:READS]->(result:Table {id:"multica.schedule_admission_result_outbox"})
-- MERGE (f)-[:READS]->(run:Table {id:"multica.task_execution_run"})
-- MERGE (f)-[:READS]->(demand:Table {id:"multica.infrastructure_capacity_demand_outbox"})
-- MERGE (f)-[:WRITES]->(receipt:Table {id:"multica.schedule_compensation_inbox"})
-- MERGE (f)-[:WRITES]->(reservation:Table {id:"multica.project_execution_resource_reservation"})
-- MERGE (f)-[:WRITES]->(epoch:Table {id:"multica.project_execution_resource_admission_lock"})
-- MERGE (f)-[:WRITES]->(event:Table {id:"multica.task_execution_run_event"})
-- MERGE (f)-[:WRITES]->(audit:Table {id:"multica.project_execution_resource_reservation_event"})
-- MERGE (guard:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-compensation.sql::guard_prepared_schedule_demand",kind:"function"})
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (guard)-[:READS]->(run)
-- MERGE (guard)-[:READS]->(receipt)
-- MERGE (guard)-[:READS]->(event)
-- MERGE (guard)-[:READS]->(reservation)
-- MERGE (guard)-[:WRITES]->(epoch)
-- MERGE (self)-[:CONFIGURES]->(demand)
-- @endcypher
BEGIN;
CREATE TABLE IF NOT EXISTS multica.schedule_compensation_inbox (
 tenant_id UUID NOT NULL, request_id UUID NOT NULL, resolution_id UUID NOT NULL UNIQUE,
 source_schedule_service_id UUID NOT NULL,
 resolution_wire BYTEA NOT NULL CHECK(octet_length(resolution_wire) BETWEEN 1 AND 16384),
 recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
 PRIMARY KEY(tenant_id,request_id),
 FOREIGN KEY(tenant_id,request_id) REFERENCES multica.schedule_admission_inbox(tenant_id,request_id),
 CHECK(resolution_id<>'00000000-0000-0000-0000-000000000000'::UUID),
 CHECK(source_schedule_service_id<>'00000000-0000-0000-0000-000000000000'::UUID)
);
ALTER TABLE multica.schedule_compensation_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.schedule_compensation_inbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS schedule_compensation_tenant ON multica.schedule_compensation_inbox;
CREATE POLICY schedule_compensation_tenant ON multica.schedule_compensation_inbox
 USING(tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID)
 WITH CHECK(tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID);
DROP TRIGGER IF EXISTS schedule_compensation_worm ON multica.schedule_compensation_inbox;
CREATE TRIGGER schedule_compensation_worm BEFORE UPDATE OR DELETE OR TRUNCATE
 ON multica.schedule_compensation_inbox FOR EACH STATEMENT
 EXECUTE FUNCTION multica.reject_schedule_admission_mutation();
REVOKE ALL ON multica.schedule_compensation_inbox FROM PUBLIC;
GRANT SELECT ON multica.schedule_compensation_inbox TO star_multica_run_owner;
CREATE OR REPLACE FUNCTION multica.compensate_schedule_preparation(p_wire BYTEA,p_service UUID)
RETURNS TEXT LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE w JSONB; result JSONB; i multica.schedule_admission_inbox%ROWTYPE;
 r multica.task_execution_run%ROWTYPE; s multica.project_execution_resource_reservation%ROWTYPE;
 prior multica.schedule_compensation_inbox%ROWTYPE; event UUID:=gen_random_uuid(); details JSONB;
BEGIN
 IF p_wire IS NULL OR octet_length(p_wire) NOT BETWEEN 1 AND 16384 OR p_service IS NULL
 THEN RAISE EXCEPTION 'invalid compensation boundary'; END IF;
 w:=convert_from(p_wire,'UTF8')::JSONB;
 IF (w->>'schema_version')::INTEGER IS DISTINCT FROM 1 OR w->>'decision' IS DISTINCT FROM 'compensate'
 OR w->>'compensation_reason' IS NULL OR w->>'compensation_reason' NOT IN
 ('schedule_fence_lost','occurrence_cancelled','occurrence_expired','schedule_link_conflict')
 OR (w->>'tenant_id')::UUID IS DISTINCT FROM NULLIF(current_setting('app.tenant_id',true),'')::UUID
 THEN RAISE EXCEPTION 'invalid compensation scope'; END IF;
 SELECT * INTO STRICT i FROM multica.schedule_admission_inbox
 WHERE tenant_id=(w->>'tenant_id')::UUID AND request_id=(w->>'request_id')::UUID;
 IF i.source_schedule_service_id IS DISTINCT FROM p_service THEN RAISE EXCEPTION 'compensation source mismatch'; END IF;
 SELECT convert_from(o.result_wire,'UTF8')::JSONB INTO STRICT result
 FROM multica.schedule_admission_result_outbox o WHERE o.tenant_id=i.tenant_id AND o.request_id=i.request_id;
 IF result->>'outcome' IS DISTINCT FROM 'accepted' OR
 (w - 'schema_version' - 'resolution_id' - 'decision' - 'compensation_reason') IS DISTINCT FROM
 (result - 'schema_version' - 'outcome' - 'rejection_code')
 THEN RAISE EXCEPTION 'compensation must match accepted result'; END IF;
 -- Same request lock as Rust admission/resolution; immutable replay is historical cleanup only.
 PERFORM pg_advisory_xact_lock(hashtextextended('schedule-admission:'||i.tenant_id::TEXT||':'||i.request_id::TEXT,0));
 SELECT * INTO prior FROM multica.schedule_compensation_inbox WHERE tenant_id=i.tenant_id AND request_id=i.request_id;
 IF FOUND THEN
  IF prior.resolution_wire IS DISTINCT FROM p_wire OR prior.source_schedule_service_id IS DISTINCT FROM p_service
  THEN RAISE EXCEPTION 'compensation replay conflict'; END IF;
  RETURN 'duplicate';
 END IF;
 -- Cleanup remains possible after ACL/quota revocation; it cannot authorize new work.
 INSERT INTO multica.project_execution_resource_admission_lock
 (tenant_id,project_id,allocation_epoch,updated_at,expires_at)
 VALUES(i.tenant_id,i.project_id,1,clock_timestamp(),clock_timestamp()+interval '30 days')
 ON CONFLICT(tenant_id,project_id) DO UPDATE SET
 allocation_epoch=multica.project_execution_resource_admission_lock.allocation_epoch+1,
 updated_at=clock_timestamp(),expires_at=clock_timestamp()+interval '30 days';
 SELECT * INTO STRICT r FROM multica.task_execution_run
 WHERE tenant_id=i.tenant_id AND run_id=(w->>'task_execution_run_id')::UUID FOR UPDATE;
 IF r.schedule_admission_request_id IS DISTINCT FROM i.request_id OR r.started_at IS NOT NULL
 OR EXISTS(SELECT 1 FROM multica.task_execution_run_event e WHERE e.tenant_id=r.tenant_id AND e.run_id=r.run_id
  AND (e.event_type IN ('run_started','schedule_admission_confirmed') OR e.execution_state IS NOT NULL))
 OR EXISTS(SELECT 1 FROM multica.infrastructure_capacity_demand_outbox o
  WHERE o.tenant_id=r.tenant_id AND o.task_execution_run_id=r.run_id)
 THEN RAISE EXCEPTION 'compensation requires unconfirmed non-executing preparation without infrastructure demand'; END IF;
 SELECT * INTO STRICT s FROM multica.project_execution_resource_reservation
 WHERE tenant_id=r.tenant_id AND run_id=r.run_id FOR UPDATE;
 IF s.reservation_state<>'pending' THEN RAISE EXCEPTION 'compensation requires pending preparation reservation'; END IF;
 UPDATE multica.project_execution_resource_reservation SET reservation_state='released',released_at=clock_timestamp()
 WHERE tenant_id=s.tenant_id AND reservation_id=s.reservation_id;
 details:=jsonb_build_object('reservation_id',s.reservation_id,'request_id',i.request_id,
 'resolution_id',w->'resolution_id','compensation_reason',w->'compensation_reason','stage','pre_infrastructure_compensation');
 INSERT INTO multica.project_execution_resource_reservation_event
 (event_id,tenant_id,project_id,reservation_id,run_id,event_type,actor_id,correlation_id,details)
 VALUES(event,r.tenant_id,r.project_id,s.reservation_id,r.run_id,'released',i.run_as_actor_id,i.correlation_id,details);
 INSERT INTO multica.task_execution_run_event
 (event_id,tenant_id,project_id,run_id,work_item_id,event_type,actor_id,correlation_id,details)
 VALUES(event,r.tenant_id,r.project_id,r.run_id,r.work_item_id,'resource_reservation_released',i.run_as_actor_id,i.correlation_id,details);
 INSERT INTO multica.task_execution_run_event
 (event_id,tenant_id,project_id,run_id,work_item_id,event_type,actor_id,correlation_id,details)
 VALUES(gen_random_uuid(),r.tenant_id,r.project_id,r.run_id,r.work_item_id,'schedule_admission_compensated',i.run_as_actor_id,i.correlation_id,details);
 INSERT INTO multica.schedule_compensation_inbox(tenant_id,request_id,resolution_id,source_schedule_service_id,resolution_wire)
 VALUES(i.tenant_id,i.request_id,(w->>'resolution_id')::UUID,p_service,p_wire);
 RETURN 'released';
END $$;
REVOKE ALL ON FUNCTION multica.compensate_schedule_preparation(BYTEA,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.compensate_schedule_preparation(BYTEA,UUID) TO star_multica_run_owner;
COMMENT ON TABLE multica.schedule_compensation_inbox IS
 'Transaction: immutable exact-byte Schedule compensation receipt paired with owner-local release and audit. No TTL/deletion; tenant FORCE RLS. Does not prove Schedule producer or service authentication.';
-- Fence the existing low-level demand writer against compensation and missing confirmation.
-- Legacy fixture/manual Runs without preparation lineage retain their existing contract.
CREATE OR REPLACE FUNCTION multica.guard_prepared_schedule_demand()
RETURNS TRIGGER LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE r multica.task_execution_run%ROWTYPE;
BEGIN
 SELECT * INTO STRICT r FROM multica.task_execution_run
 WHERE tenant_id=NEW.tenant_id AND run_id=NEW.task_execution_run_id;
 IF r.schedule_admission_request_id IS NULL THEN RETURN NEW; END IF;
 IF NEW.tenant_id IS DISTINCT FROM NULLIF(current_setting('app.tenant_id',true),'')::UUID
 THEN RAISE EXCEPTION 'prepared demand tenant mismatch'; END IF;
 INSERT INTO multica.project_execution_resource_admission_lock
 (tenant_id,project_id,allocation_epoch,updated_at,expires_at)
 VALUES(r.tenant_id,r.project_id,1,clock_timestamp(),clock_timestamp()+interval '30 days')
 ON CONFLICT(tenant_id,project_id) DO UPDATE SET
 allocation_epoch=multica.project_execution_resource_admission_lock.allocation_epoch+1,
 updated_at=clock_timestamp(),expires_at=clock_timestamp()+interval '30 days';
 PERFORM 1 FROM multica.task_execution_run WHERE tenant_id=r.tenant_id AND run_id=r.run_id FOR UPDATE;
 IF EXISTS(SELECT 1 FROM multica.schedule_compensation_inbox c
 WHERE c.tenant_id=r.tenant_id AND c.request_id=r.schedule_admission_request_id)
 OR NOT EXISTS(SELECT 1 FROM multica.task_execution_run_event e
 WHERE e.tenant_id=r.tenant_id AND e.run_id=r.run_id AND e.event_type='schedule_admission_confirmed')
 OR NOT EXISTS(SELECT 1 FROM multica.project_execution_resource_reservation s
 WHERE s.tenant_id=r.tenant_id AND s.run_id=r.run_id AND s.reservation_state='pending'
 AND s.lease_expires_at>clock_timestamp())
 THEN RAISE EXCEPTION 'prepared Schedule demand requires confirmation and live uncompensated reservation'; END IF;
 RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS prepared_schedule_demand_fence ON multica.infrastructure_capacity_demand_outbox;
CREATE TRIGGER prepared_schedule_demand_fence BEFORE INSERT ON multica.infrastructure_capacity_demand_outbox
 FOR EACH ROW EXECUTE FUNCTION multica.guard_prepared_schedule_demand();
REVOKE ALL ON FUNCTION multica.guard_prepared_schedule_demand() FROM PUBLIC;
COMMIT;
