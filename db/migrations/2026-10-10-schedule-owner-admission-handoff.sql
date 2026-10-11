-- Schedule owns its request/result/resolution facts; no Run/Infrastructure writes.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-schedule-owner-admission-handoff.sql"})
-- MERGE (queue:Symbol {id:"db/migrations/2026-10-10-schedule-owner-admission-handoff.sql::queue_run_admission",kind:"function"})
-- MERGE (resolve:Symbol {id:"db/migrations/2026-10-10-schedule-owner-admission-handoff.sql::resolve_run_admission",kind:"function"})
-- MERGE (self)-[:DEFINES]->(queue)
-- MERGE (self)-[:DEFINES]->(resolve)
-- MERGE (queue)-[:READS]->(occurrence:Table {id:"automation.occurrence"})
-- MERGE (queue)-[:READS]->(dispatch:Table {id:"automation.occurrence_dispatch"})
-- MERGE (queue)-[:READS]->(rule:Table {id:"automation.schedule_rule_revision"})
-- MERGE (queue)-[:WRITES]->(request:Table {id:"automation.run_admission_request_outbox"})
-- MERGE (resolve)-[:READS]->(request)
-- MERGE (resolve)-[:READS]->(occurrence)
-- MERGE (resolve)-[:READS]->(rule)
-- MERGE (resolve)-[:WRITES]->(dispatch)
-- MERGE (resolve)-[:WRITES]->(event:Table {id:"automation.occurrence_event"})
-- MERGE (resolve)-[:WRITES]->(link:Table {id:"automation.schedule_run_outbox"})
-- MERGE (resolve)-[:WRITES]->(receipt:Table {id:"automation.run_admission_result_inbox"})
-- MERGE (resolve)-[:WRITES]->(resolution:Table {id:"automation.run_admission_resolution_outbox"})
-- @endcypher
BEGIN;
DO $$ BEGIN IF NOT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='star_automation_schedule_owner') THEN
 CREATE ROLE star_automation_schedule_owner NOLOGIN NOBYPASSRLS; END IF; END $$;
GRANT USAGE ON SCHEMA automation TO star_automation_schedule_owner;
GRANT SELECT ON automation.schedule_rule_revision TO star_automation_schedule_owner;
CREATE TABLE IF NOT EXISTS automation.run_admission_request_outbox (
 tenant_id UUID NOT NULL,request_id UUID NOT NULL,occurrence_id UUID NOT NULL,
 admission_attempt INTEGER NOT NULL CHECK(admission_attempt BETWEEN 1 AND 25),
 lease_owner_id UUID NOT NULL,fencing_generation BIGINT NOT NULL CHECK(fencing_generation>0),
 target_run_service_id UUID NOT NULL CHECK(target_run_service_id<>'00000000-0000-0000-0000-000000000000'::UUID),
 command_wire BYTEA NOT NULL CHECK(octet_length(command_wire) BETWEEN 1 AND 16384),
 recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
 PRIMARY KEY(tenant_id,request_id),UNIQUE(tenant_id,occurrence_id,admission_attempt),
 FOREIGN KEY(tenant_id,occurrence_id) REFERENCES automation.occurrence(tenant_id,occurrence_id)
);
CREATE TABLE IF NOT EXISTS automation.run_admission_result_inbox (
 tenant_id UUID NOT NULL,request_id UUID NOT NULL,result_id UUID NOT NULL UNIQUE,
 source_run_service_id UUID NOT NULL,result_wire BYTEA NOT NULL CHECK(octet_length(result_wire) BETWEEN 1 AND 16384),
 outcome TEXT NOT NULL CHECK(outcome IN ('accepted','rejected')),resolution_id UUID,
 recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
 PRIMARY KEY(tenant_id,request_id),UNIQUE(tenant_id,request_id,result_id),
 CHECK((outcome='accepted')=(resolution_id IS NOT NULL)),
 FOREIGN KEY(tenant_id,request_id) REFERENCES automation.run_admission_request_outbox(tenant_id,request_id)
);
CREATE TABLE IF NOT EXISTS automation.run_admission_resolution_outbox (
 tenant_id UUID NOT NULL,request_id UUID NOT NULL,result_id UUID NOT NULL,resolution_id UUID NOT NULL UNIQUE,
 resolution_wire BYTEA NOT NULL CHECK(octet_length(resolution_wire) BETWEEN 1 AND 16384),
 recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),PRIMARY KEY(tenant_id,request_id),
 UNIQUE(tenant_id,request_id,resolution_id),
 FOREIGN KEY(tenant_id,request_id,result_id) REFERENCES automation.run_admission_result_inbox(tenant_id,request_id,result_id)
);
DO $$ BEGIN IF NOT EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='automation.run_admission_result_inbox'::regclass AND conname='schedule_result_resolution_pair') THEN
 ALTER TABLE automation.run_admission_result_inbox ADD CONSTRAINT schedule_result_resolution_pair
 FOREIGN KEY(tenant_id,request_id,resolution_id) REFERENCES automation.run_admission_resolution_outbox(tenant_id,request_id,resolution_id)
 DEFERRABLE INITIALLY DEFERRED; END IF; END $$;
DO $$ DECLARE n TEXT; BEGIN
 FOREACH n IN ARRAY ARRAY['run_admission_request_outbox','run_admission_result_inbox','run_admission_resolution_outbox'] LOOP
  EXECUTE format('ALTER TABLE automation.%I ENABLE ROW LEVEL SECURITY',n);
  EXECUTE format('ALTER TABLE automation.%I FORCE ROW LEVEL SECURITY',n);
  EXECUTE format('DROP POLICY IF EXISTS schedule_handoff_tenant ON automation.%I',n);
  EXECUTE format('CREATE POLICY schedule_handoff_tenant ON automation.%I USING(tenant_id=NULLIF(current_setting(''app.tenant_id'',true),'''')::UUID) WITH CHECK(tenant_id=NULLIF(current_setting(''app.tenant_id'',true),'''')::UUID)',n);
  EXECUTE format('DROP TRIGGER IF EXISTS schedule_handoff_worm ON automation.%I',n);
  EXECUTE format('CREATE TRIGGER schedule_handoff_worm BEFORE UPDATE OR DELETE OR TRUNCATE ON automation.%I FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation()',n);
  EXECUTE format('REVOKE ALL ON automation.%I FROM PUBLIC',n);
  EXECUTE format('GRANT SELECT ON automation.%I TO star_automation_schedule_owner',n);
 END LOOP;
END $$;
CREATE OR REPLACE FUNCTION automation.queue_run_admission(p_wire BYTEA,p_run_service UUID,p_owner UUID,p_generation BIGINT)
RETURNS BYTEA LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE w JSONB; tenant UUID; request UUID; ordinal INTEGER; o automation.occurrence%ROWTYPE;
 d automation.occurrence_dispatch%ROWTYPE; r automation.schedule_rule_revision%ROWTYPE;
 prior automation.run_admission_request_outbox%ROWTYPE; previous INTEGER;
BEGIN
 IF p_wire IS NULL OR octet_length(p_wire) NOT BETWEEN 1 AND 16384 OR p_run_service IS NULL
 OR p_run_service='00000000-0000-0000-0000-000000000000'::UUID OR p_owner IS NULL
 OR p_owner='00000000-0000-0000-0000-000000000000'::UUID OR p_generation IS NULL OR p_generation<=0
 THEN RAISE EXCEPTION 'invalid Schedule request boundary'; END IF;
 w:=convert_from(p_wire,'UTF8')::JSONB;tenant:=(w->>'tenant_id')::UUID;request:=(w->>'request_id')::UUID;
 ordinal:=(w->>'admission_attempt')::INTEGER;
 IF tenant IS DISTINCT FROM NULLIF(current_setting('app.tenant_id',true),'')::UUID OR (w->>'schema_version')::INTEGER IS DISTINCT FROM 1
 THEN RAISE EXCEPTION 'Schedule request tenant/schema mismatch'; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended('schedule-request:'||tenant::TEXT||':'||(w#>>'{occurrence,occurrence_id}'),0));
 SELECT * INTO prior FROM automation.run_admission_request_outbox WHERE tenant_id=tenant AND request_id=request;
 IF FOUND THEN
  IF prior.command_wire IS DISTINCT FROM p_wire OR prior.target_run_service_id IS DISTINCT FROM p_run_service
  OR prior.lease_owner_id IS DISTINCT FROM p_owner OR prior.fencing_generation IS DISTINCT FROM p_generation
  THEN RAISE EXCEPTION 'Schedule request replay conflict'; END IF; RETURN prior.command_wire;
 END IF;
 SELECT * INTO STRICT o FROM automation.occurrence WHERE tenant_id=tenant AND occurrence_id=(w#>>'{occurrence,occurrence_id}')::UUID;
 SELECT * INTO STRICT d FROM automation.occurrence_dispatch WHERE tenant_id=tenant AND occurrence_id=o.occurrence_id FOR UPDATE;
 SELECT * INTO STRICT r FROM automation.schedule_rule_revision WHERE tenant_id=tenant AND rule_id=o.rule_id AND rule_version=o.rule_version;
 IF d.dispatch_state<>'leased' OR d.lease_owner_id IS DISTINCT FROM p_owner OR d.fencing_generation IS DISTINCT FROM p_generation
 OR d.lease_expires_at<=clock_timestamp() OR d.occurrence_deadline_at<=clock_timestamp()
 OR o.project_id IS DISTINCT FROM (w#>>'{occurrence,rule_snapshot,project_id}')::UUID
 OR o.rule_id IS DISTINCT FROM (w#>>'{occurrence,key,rule_id}')::UUID
 OR o.rule_version IS DISTINCT FROM (w#>>'{occurrence,key,rule_version}')::BIGINT
 OR o.scheduled_for_utc IS DISTINCT FROM (w#>>'{occurrence,key,scheduled_for_utc}')::TIMESTAMPTZ
 OR o.run_as_actor_id IS DISTINCT FROM (w#>>'{occurrence,run_as_actor_id}')::UUID
 OR o.target_snapshot IS DISTINCT FROM w#>'{occurrence,rule_snapshot,target}'
 OR r.deadline_seconds IS DISTINCT FROM (w#>>'{occurrence,rule_snapshot,deadline_seconds}')::INTEGER
 OR o.scheduled_for_utc+r.deadline_seconds*interval '1 second'<=clock_timestamp()
 THEN RAISE EXCEPTION 'Schedule request requires canonical occurrence and current lease'; END IF;
 SELECT COALESCE(max(admission_attempt),0) INTO previous FROM automation.run_admission_request_outbox WHERE tenant_id=tenant AND occurrence_id=o.occurrence_id;
 IF ordinal<>previous+1 OR ordinal>r.retry_max_attempts OR EXISTS(
 SELECT 1 FROM automation.run_admission_request_outbox q LEFT JOIN automation.run_admission_result_inbox i
 ON i.tenant_id=q.tenant_id AND i.request_id=q.request_id WHERE q.tenant_id=tenant AND q.occurrence_id=o.occurrence_id
 AND (i.request_id IS NULL OR EXISTS(SELECT 1 FROM automation.run_admission_resolution_outbox x
 WHERE x.tenant_id=i.tenant_id AND x.request_id=i.request_id AND convert_from(x.resolution_wire,'UTF8')::JSONB->>'decision'='confirm')))
 THEN RAISE EXCEPTION 'Schedule logical admission attempt is not available'; END IF;
 INSERT INTO automation.run_admission_request_outbox(tenant_id,request_id,occurrence_id,admission_attempt,lease_owner_id,fencing_generation,target_run_service_id,command_wire)
 VALUES(tenant,request,o.occurrence_id,ordinal,p_owner,p_generation,p_run_service,p_wire);
 IF LEAST(d.lease_expires_at,d.occurrence_deadline_at,o.scheduled_for_utc+r.deadline_seconds*interval '1 second')<=clock_timestamp()
 THEN RAISE EXCEPTION 'Schedule request lease expired during persistence'; END IF;
 RETURN p_wire;
END $$;
CREATE OR REPLACE FUNCTION automation.resolve_run_admission(p_wire BYTEA,p_run_service UUID)
RETURNS BYTEA LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE w JSONB; cmd JSONB; tenant UUID; request UUID; q automation.run_admission_request_outbox%ROWTYPE;
 i automation.run_admission_result_inbox%ROWTYPE;o automation.occurrence%ROWTYPE;d automation.occurrence_dispatch%ROWTYPE;
 r automation.schedule_rule_revision%ROWTYPE;reason TEXT;resolution JSONB;resolution_id UUID;run UUID;
 deadline TIMESTAMPTZ;next_attempt TIMESTAMPTZ;stamp TIMESTAMPTZ;
BEGIN
 IF p_wire IS NULL OR octet_length(p_wire) NOT BETWEEN 1 AND 16384 OR p_run_service IS NULL THEN RAISE EXCEPTION 'invalid Schedule result boundary'; END IF;
 w:=convert_from(p_wire,'UTF8')::JSONB;tenant:=(w->>'tenant_id')::UUID;request:=(w->>'request_id')::UUID;
 IF tenant IS DISTINCT FROM NULLIF(current_setting('app.tenant_id',true),'')::UUID OR (w->>'schema_version')::INTEGER IS DISTINCT FROM 1 THEN RAISE EXCEPTION 'Schedule result tenant/schema mismatch'; END IF;
 SELECT * INTO STRICT q FROM automation.run_admission_request_outbox WHERE tenant_id=tenant AND request_id=request;
 IF q.target_run_service_id IS DISTINCT FROM p_run_service THEN RAISE EXCEPTION 'Schedule result source mismatch'; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended('schedule-request:'||tenant::TEXT||':'||q.occurrence_id::TEXT,0));
 SELECT * INTO i FROM automation.run_admission_result_inbox WHERE tenant_id=tenant AND request_id=request;
 IF FOUND THEN
  IF i.result_wire IS DISTINCT FROM p_wire OR i.source_run_service_id IS DISTINCT FROM p_run_service THEN RAISE EXCEPTION 'Schedule result replay conflict'; END IF;
  RETURN (SELECT resolution_wire FROM automation.run_admission_resolution_outbox WHERE tenant_id=tenant AND request_id=request);
 END IF;
 cmd:=convert_from(q.command_wire,'UTF8')::JSONB;
 IF (w->>'admission_attempt')::INTEGER IS DISTINCT FROM q.admission_attempt OR (w->>'occurrence_id')::UUID IS DISTINCT FROM q.occurrence_id
 OR w->>'correlation_id' IS DISTINCT FROM cmd->>'correlation_id' OR w->>'outcome' NOT IN ('accepted','rejected')
 THEN RAISE EXCEPTION 'Schedule result must match its request'; END IF;
 SELECT * INTO STRICT o FROM automation.occurrence WHERE tenant_id=tenant AND occurrence_id=q.occurrence_id;
 SELECT * INTO STRICT d FROM automation.occurrence_dispatch WHERE tenant_id=tenant AND occurrence_id=q.occurrence_id FOR UPDATE;
 SELECT * INTO STRICT r FROM automation.schedule_rule_revision WHERE tenant_id=tenant AND rule_id=o.rule_id AND rule_version=o.rule_version;
 deadline:=LEAST(d.occurrence_deadline_at,o.scheduled_for_utc+r.deadline_seconds*interval '1 second');
 IF w->>'outcome'='accepted' THEN
  run:=(w->>'task_execution_run_id')::UUID;
  IF d.dispatch_state='cancelled' THEN reason:='occurrence_cancelled';
  ELSIF deadline<=clock_timestamp() THEN reason:='occurrence_expired';
  ELSIF d.admitted_run_id IS NOT NULL THEN reason:='schedule_link_conflict';
  ELSIF d.dispatch_state<>'leased' OR d.lease_owner_id IS DISTINCT FROM q.lease_owner_id
  OR d.fencing_generation IS DISTINCT FROM q.fencing_generation OR d.lease_expires_at<=clock_timestamp()
  THEN reason:='schedule_fence_lost'; END IF;
  resolution_id:=gen_random_uuid();
  resolution:=(w-'outcome'-'rejection_code')||jsonb_build_object('resolution_id',resolution_id,
   'decision',CASE WHEN reason IS NULL THEN 'confirm' ELSE 'compensate' END,'compensation_reason',reason);
  IF reason IS NULL THEN
   UPDATE automation.occurrence_dispatch SET dispatch_state='admitted',admitted_run_id=run,
    lease_owner_id=NULL,lease_expires_at=NULL,updated_at=clock_timestamp()
   WHERE tenant_id=tenant AND occurrence_id=q.occurrence_id;
   INSERT INTO automation.occurrence_event(tenant_id,project_id,occurrence_id,event_type,attempt_no,fencing_generation,run_id,actor_id,correlation_id,details)
   VALUES(tenant,o.project_id,o.occurrence_id,'run_admitted',d.attempt_count,d.fencing_generation,run,o.run_as_actor_id,(cmd->>'correlation_id')::UUID,
   jsonb_build_object('request_id',request,'result_id',w->'result_id','resolution_id',resolution_id));
   INSERT INTO automation.schedule_run_outbox(tenant_id,project_id,engineering_run_id,rule_id,rule_version,occurrence_id,work_item_id,run_id,run_as_actor_id,event_type,correlation_id,payload)
   VALUES(tenant,o.project_id,r.engineering_run_id,o.rule_id,o.rule_version,o.occurrence_id,r.work_item_id,run,o.run_as_actor_id,'schedule_run.admitted',(cmd->>'correlation_id')::UUID,resolution);
  END IF;
 ELSIF d.dispatch_state='leased' AND d.lease_owner_id=q.lease_owner_id AND d.fencing_generation=q.fencing_generation AND d.lease_expires_at>clock_timestamp() THEN
  next_attempt:=clock_timestamp()+LEAST(r.retry_max_backoff_seconds::NUMERIC,r.retry_initial_backoff_seconds::NUMERIC*power(2::NUMERIC,q.admission_attempt-1))*interval '1 second';
  IF q.admission_attempt<r.retry_max_attempts AND next_attempt<deadline THEN
   UPDATE automation.occurrence_dispatch SET dispatch_state='retry_wait',next_attempt_at=next_attempt,lease_owner_id=NULL,lease_expires_at=NULL,last_failure_code='run_admission_rejected',updated_at=clock_timestamp() WHERE tenant_id=tenant AND occurrence_id=q.occurrence_id;
   INSERT INTO automation.occurrence_event(tenant_id,project_id,occurrence_id,event_type,attempt_no,fencing_generation,actor_id,correlation_id,details)
   VALUES(tenant,o.project_id,o.occurrence_id,'retry_scheduled',d.attempt_count,d.fencing_generation,o.run_as_actor_id,(cmd->>'correlation_id')::UUID,jsonb_build_object('request_id',request,'rejection_code',w->'rejection_code'));
  ELSE
   stamp:=clock_timestamp();
   UPDATE automation.occurrence_dispatch SET dispatch_state='failed',lease_owner_id=NULL,lease_expires_at=NULL,last_failure_code='run_admission_rejected',terminal_at=stamp,expires_at=stamp+retention_period,updated_at=stamp WHERE tenant_id=tenant AND occurrence_id=q.occurrence_id;
   INSERT INTO automation.occurrence_event(tenant_id,project_id,occurrence_id,event_type,attempt_no,fencing_generation,actor_id,correlation_id,details)
   VALUES(tenant,o.project_id,o.occurrence_id,'dispatch_failed',d.attempt_count,d.fencing_generation,o.run_as_actor_id,(cmd->>'correlation_id')::UUID,jsonb_build_object('request_id',request,'rejection_code',w->'rejection_code'));
  END IF;
 END IF;
 INSERT INTO automation.run_admission_result_inbox(tenant_id,request_id,result_id,source_run_service_id,result_wire,outcome,resolution_id)
 VALUES(tenant,request,(w->>'result_id')::UUID,p_run_service,p_wire,w->>'outcome',resolution_id);
 IF resolution IS NOT NULL THEN
  INSERT INTO automation.run_admission_resolution_outbox(tenant_id,request_id,result_id,resolution_id,resolution_wire)
  VALUES(tenant,request,(w->>'result_id')::UUID,resolution_id,convert_to(resolution::TEXT,'UTF8'));
  IF reason IS NULL AND LEAST(d.lease_expires_at,deadline)<=clock_timestamp()
  THEN RAISE EXCEPTION 'Schedule confirmation lease expired during persistence'; END IF;
  RETURN convert_to(resolution::TEXT,'UTF8');
 END IF; RETURN NULL;
END $$;
REVOKE ALL ON FUNCTION automation.queue_run_admission(BYTEA,UUID,UUID,BIGINT) FROM PUBLIC;
REVOKE ALL ON FUNCTION automation.resolve_run_admission(BYTEA,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION automation.queue_run_admission(BYTEA,UUID,UUID,BIGINT),automation.resolve_run_admission(BYTEA,UUID) TO star_automation_schedule_owner;
COMMENT ON TABLE automation.run_admission_request_outbox IS 'Transaction: immutable bounded admission request and original Schedule fence; no TTL or physical deletion; tenant FORCE RLS.';
COMMENT ON TABLE automation.run_admission_result_inbox IS 'Transaction: immutable exact result receipt paired with accepted resolution; no TTL/deletion; tenant FORCE RLS.';
COMMENT ON TABLE automation.run_admission_resolution_outbox IS 'Transaction: immutable confirm/compensate command for Run owner; no cross-owner write or executor authority; no TTL/deletion; tenant FORCE RLS.';
COMMIT;
