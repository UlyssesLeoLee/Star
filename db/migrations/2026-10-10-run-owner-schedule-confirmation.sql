-- Confirmation commits only Run-owner facts and its infrastructure demand, never execution.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-schedule-confirmation.sql"})
-- MERGE (f:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-confirmation.sql::confirm_schedule_preparation",kind:"function"})
-- MERGE (self)-[:DEFINES]->(f)
-- MERGE (f)-[:READS]->(inbox:Table {id:"multica.schedule_admission_inbox"})
-- MERGE (f)-[:READS]->(result:Table {id:"multica.schedule_admission_result_outbox"})
-- MERGE (f)-[:READS]->(compensation:Table {id:"multica.schedule_compensation_inbox"})
-- MERGE (f)-[:READS]->(run:Table {id:"multica.task_execution_run"})
-- MERGE (f)-[:READS]->(reservation:Table {id:"multica.project_execution_resource_reservation"})
-- MERGE (f)-[:READS]->(task:Table {id:"multica.task_lifecycle_current"})
-- MERGE (f)-[:READS]->(metadata:Table {id:"multica.task_metadata"})
-- MERGE (f)-[:READS]->(contract:Table {id:"multica.task_contract"})
-- MERGE (f)-[:WRITES]->(receipt:Table {id:"multica.schedule_confirmation_inbox"})
-- MERGE (f)-[:WRITES]->(event:Table {id:"multica.task_execution_run_event"})
-- MERGE (f)-[:WRITES]->(demand:Table {id:"multica.infrastructure_capacity_demand_outbox"})
-- MERGE (f)-[:CALLS]->(capacity:Symbol {id:"db/migrations/2026-10-10-run-owner-project-capacity.sql::read_schedule_project_capacity_current",kind:"function"})
-- MERGE (f)-[:READS]->(binding:Table {id:"multica.run_infrastructure_binding_revision"})
-- MERGE (f)-[:CALLS]->(policy:Symbol {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql::read_schedule_policies_current",kind:"function"})
-- @endcypher
BEGIN;
CREATE TABLE IF NOT EXISTS multica.schedule_confirmation_inbox (
 tenant_id UUID NOT NULL, request_id UUID NOT NULL, resolution_id UUID NOT NULL UNIQUE,
 source_schedule_service_id UUID NOT NULL, demand_event_id UUID NOT NULL,
 resolution_wire BYTEA NOT NULL CHECK(octet_length(resolution_wire) BETWEEN 1 AND 16384),
 recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
 PRIMARY KEY(tenant_id,request_id),
 FOREIGN KEY(tenant_id,request_id) REFERENCES multica.schedule_admission_inbox(tenant_id,request_id),
 FOREIGN KEY(tenant_id,demand_event_id) REFERENCES multica.infrastructure_capacity_demand_outbox(tenant_id,event_id),
 CHECK(resolution_id<>'00000000-0000-0000-0000-000000000000'::UUID),
 CHECK(source_schedule_service_id<>'00000000-0000-0000-0000-000000000000'::UUID)
);
ALTER TABLE multica.schedule_confirmation_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.schedule_confirmation_inbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS schedule_confirmation_tenant ON multica.schedule_confirmation_inbox;
CREATE POLICY schedule_confirmation_tenant ON multica.schedule_confirmation_inbox
 USING(tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID)
 WITH CHECK(tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID);
DROP TRIGGER IF EXISTS schedule_confirmation_worm ON multica.schedule_confirmation_inbox;
CREATE TRIGGER schedule_confirmation_worm BEFORE UPDATE OR DELETE OR TRUNCATE
 ON multica.schedule_confirmation_inbox FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_schedule_admission_mutation();
REVOKE ALL ON multica.schedule_confirmation_inbox FROM PUBLIC;
GRANT SELECT ON multica.schedule_confirmation_inbox TO star_multica_run_owner;
CREATE OR REPLACE FUNCTION multica.confirm_schedule_preparation(p_wire BYTEA,p_service UUID,p_profile TEXT,p_demand TEXT)
RETURNS TEXT LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE w JSONB; result JSONB; cmd JSONB; d JSONB; budget JSONB; observed JSONB; q JSONB; u JSONB;
 i multica.schedule_admission_inbox%ROWTYPE; r multica.task_execution_run%ROWTYPE;
 s multica.project_execution_resource_reservation%ROWTYPE; l multica.task_lifecycle_current%ROWTYPE;
 key TEXT; usage_key TEXT; deadline TIMESTAMPTZ; event UUID:=gen_random_uuid(); details JSONB;
 policy RECORD; profile JSONB; hook JSONB; profile_version BIGINT;
BEGIN
 IF p_wire IS NULL OR octet_length(p_wire) NOT BETWEEN 1 AND 16384 OR p_service IS NULL
 OR p_profile IS NULL OR octet_length(p_profile)>73728 OR p_demand IS NULL OR octet_length(p_demand)>16384
 THEN RAISE EXCEPTION 'invalid confirmation boundary'; END IF;
 w:=convert_from(p_wire,'UTF8')::JSONB; d:=p_demand::JSONB;
 IF (w->>'schema_version')::INTEGER IS DISTINCT FROM 1 OR w->>'decision' IS DISTINCT FROM 'confirm'
 OR w->'compensation_reason' IS DISTINCT FROM 'null'::JSONB
 OR (w->>'tenant_id')::UUID IS DISTINCT FROM NULLIF(current_setting('app.tenant_id',true),'')::UUID
 THEN RAISE EXCEPTION 'invalid confirmation scope'; END IF;
 SELECT * INTO STRICT i FROM multica.schedule_admission_inbox
 WHERE tenant_id=(w->>'tenant_id')::UUID AND request_id=(w->>'request_id')::UUID;
 IF i.source_schedule_service_id IS DISTINCT FROM p_service THEN RAISE EXCEPTION 'confirmation source mismatch'; END IF;
 SELECT convert_from(o.result_wire,'UTF8')::JSONB INTO STRICT result
 FROM multica.schedule_admission_result_outbox o WHERE o.tenant_id=i.tenant_id AND o.request_id=i.request_id;
 IF result->>'outcome' IS DISTINCT FROM 'accepted' OR
 (w - 'schema_version' - 'resolution_id' - 'decision' - 'compensation_reason') IS DISTINCT FROM
 (result - 'schema_version' - 'outcome' - 'rejection_code')
 THEN RAISE EXCEPTION 'confirmation must match accepted result'; END IF;
 cmd:=convert_from(i.command_wire,'UTF8')::JSONB;
 PERFORM pg_advisory_xact_lock(hashtextextended('schedule-admission:'||i.tenant_id::TEXT||':'||i.request_id::TEXT,0));
 IF EXISTS(SELECT 1 FROM multica.schedule_confirmation_inbox WHERE tenant_id=i.tenant_id AND request_id=i.request_id)
 OR EXISTS(SELECT 1 FROM multica.schedule_compensation_inbox WHERE tenant_id=i.tenant_id AND request_id=i.request_id)
 THEN RAISE EXCEPTION 'preparation already resolved'; END IF;
 observed:=multica.read_schedule_project_capacity_current(i.tenant_id,i.run_as_actor_id,i.project_id,i.branch_id,
 i.engineering_run_id,(cmd#>>'{occurrence,rule_snapshot,target,repository_id}')::UUID,i.worktree_id,i.work_item_id)::JSONB;
 IF observed IS NULL THEN RAISE EXCEPTION 'current confirmation quota unavailable'; END IF;
 q:=observed->'quota'; u:=observed->'usage'; budget:=cmd#>'{infrastructure_binding,resource_budget}';
 -- The pending preparation is already included: never add its budget a second time.
 IF (u->>'active_run_count')::NUMERIC>(q->>'max_active_runs')::NUMERIC
 OR (budget->>'max_cpu_ms')::NUMERIC>(q->>'max_cpu_ms_per_run')::NUMERIC
 OR (budget->>'max_runtime_ms')::NUMERIC>(q->>'max_runtime_ms_per_run')::NUMERIC
 THEN RAISE EXCEPTION 'current confirmation quota exceeded'; END IF;
 FOREACH key IN ARRAY ARRAY['max_rss_bytes','max_child_processes','max_parallel_tools','max_provider_calls','max_output_bytes','max_event_buffer_bytes'] LOOP
  usage_key:='reserved_'||substring(key FROM 5);
  IF (u->>usage_key)::NUMERIC>(q->>key)::NUMERIC THEN RAISE EXCEPTION 'current confirmation capacity exceeded'; END IF;
 END LOOP;
 SELECT * INTO STRICT r FROM multica.task_execution_run
 WHERE tenant_id=i.tenant_id AND run_id=(w->>'task_execution_run_id')::UUID FOR UPDATE;
 SELECT * INTO STRICT s FROM multica.project_execution_resource_reservation
 WHERE tenant_id=r.tenant_id AND run_id=r.run_id FOR UPDATE;
 IF r.schedule_admission_request_id IS DISTINCT FROM i.request_id OR r.started_at IS NOT NULL
 OR r.execution_profile_snapshot IS DISTINCT FROM p_profile::JSONB
 OR s.reservation_state<>'pending' OR s.lease_expires_at<=clock_timestamp()
 OR EXISTS(SELECT 1 FROM multica.task_execution_run_event e WHERE e.tenant_id=r.tenant_id AND e.run_id=r.run_id
  AND (e.event_type IN ('run_started','schedule_admission_confirmed','schedule_admission_compensated') OR e.execution_state IS NOT NULL))
 OR EXISTS(SELECT 1 FROM multica.infrastructure_capacity_demand_outbox o WHERE o.tenant_id=r.tenant_id AND o.task_execution_run_id=r.run_id)
 THEN RAISE EXCEPTION 'confirmation requires live unchanged non-executing preparation'; END IF;
 PERFORM 1 FROM multica.run_infrastructure_binding_revision b WHERE b.tenant_id=r.tenant_id
 AND b.project_id=i.project_id AND b.branch_id=i.branch_id AND b.engineering_run_id=i.engineering_run_id
 AND b.valid_from<=clock_timestamp() AND b.valid_to IS NULL
 AND b.binding_snapshot=r.infrastructure_binding_snapshot AND b.binding_digest=r.infrastructure_binding_digest FOR SHARE;
 IF NOT FOUND THEN RAISE EXCEPTION 'confirmation binding changed'; END IF;
 FOR policy IN SELECT * FROM multica.read_schedule_policies_current(i.tenant_id,i.run_as_actor_id,i.project_id,i.branch_id,
 i.engineering_run_id,r.repository_id,i.worktree_id,i.work_item_id,r.execution_profile_id) LOOP
  IF policy.kind='profile' THEN profile:=policy.payload::JSONB->'profile_document'; profile_version:=(policy.payload::JSONB->>'profile_version')::BIGINT;
  ELSE hook:=policy.payload::JSONB->'policy_document'; END IF;
 END LOOP;
 IF profile IS DISTINCT FROM r.execution_profile_snapshot OR profile_version IS DISTINCT FROM r.execution_profile_version
 OR hook IS DISTINCT FROM r.hook_set_snapshot->'effective_policy_document'
 THEN RAISE EXCEPTION 'confirmation policy changed'; END IF;
 SELECT * INTO STRICT l FROM multica.task_lifecycle_current WHERE tenant_id=r.tenant_id AND work_item_id=r.work_item_id FOR SHARE;
 IF l.status<>'in_progress' OR l.review_state='pending_review' OR l.claimed_by IS DISTINCT FROM i.run_as_actor_id
 OR l.active_worktree_id IS DISTINCT FROM i.worktree_id THEN RAISE EXCEPTION 'confirmation task lifecycle unavailable'; END IF;
 PERFORM 1 FROM multica.task_contract c WHERE c.tenant_id=r.tenant_id AND c.work_item_id=r.work_item_id
 AND c.project_id=r.project_id AND c.version=r.task_contract_version AND c.valid_from<=clock_timestamp() AND c.valid_to IS NULL FOR SHARE;
 IF NOT FOUND THEN RAISE EXCEPTION 'confirmation task contract changed'; END IF;
 PERFORM 1 FROM multica.task_metadata m WHERE m.tenant_id=r.tenant_id AND m.work_item_id=r.work_item_id
 AND m.project_id=r.project_id AND m.version=(r.task_snapshot->>'metadata_version')::BIGINT
 AND m.valid_from<=clock_timestamp() AND m.valid_to IS NULL FOR SHARE;
 IF NOT FOUND THEN RAISE EXCEPTION 'confirmation task metadata changed'; END IF;
 deadline:=(cmd#>>'{occurrence,key,scheduled_for_utc}')::TIMESTAMPTZ+
 (cmd#>>'{occurrence,rule_snapshot,deadline_seconds}')::BIGINT*interval '1 second';
 IF deadline<=clock_timestamp() OR (d->>'schema_version')::INTEGER IS DISTINCT FROM 2
 OR (d->>'tenant_id')::UUID IS DISTINCT FROM i.tenant_id OR (d->>'project_id')::UUID IS DISTINCT FROM i.project_id
 OR (d->>'branch_id')::UUID IS DISTINCT FROM i.branch_id OR (d->>'engineering_run_id')::UUID IS DISTINCT FROM i.engineering_run_id
 OR (d->>'task_execution_run_id')::UUID IS DISTINCT FROM r.run_id OR (d->>'occurrence_id')::UUID IS DISTINCT FROM i.occurrence_id
 OR (d->>'schedule_rule_id')::UUID IS DISTINCT FROM i.rule_id OR (d->>'schedule_rule_version')::BIGINT IS DISTINCT FROM i.rule_version
 OR (d->>'correlation_id')::UUID IS DISTINCT FROM i.correlation_id OR d->'binding' IS DISTINCT FROM r.infrastructure_binding_snapshot
 OR d->'resource_budget' IS DISTINCT FROM r.resource_budget_snapshot
 OR (d->>'deadline_at')::TIMESTAMPTZ IS DISTINCT FROM deadline
 OR (d->>'requested_at')::TIMESTAMPTZ>clock_timestamp()
 THEN RAISE EXCEPTION 'confirmation demand must match prepared scope and deadline'; END IF;
 d:=jsonb_set(d,'{deadline_at}',to_jsonb(LEAST(deadline,s.lease_expires_at)));
 IF (d->>'deadline_at')::TIMESTAMPTZ<=clock_timestamp() THEN RAISE EXCEPTION 'confirmation demand deadline elapsed'; END IF;
 details:=jsonb_build_object('request_id',i.request_id,'resolution_id',w->'resolution_id','result_id',i.result_id,
 'demand_event_id',d->'event_id','quota_revision_id',q->'quota_revision_id','quota_version',q->'quota_version',
 'stage','awaiting_infrastructure_capacity');
 INSERT INTO multica.task_execution_run_event(event_id,tenant_id,project_id,run_id,work_item_id,event_type,actor_id,correlation_id,details)
 VALUES(event,r.tenant_id,r.project_id,r.run_id,r.work_item_id,'schedule_admission_confirmed',i.run_as_actor_id,i.correlation_id,details);
 INSERT INTO multica.infrastructure_capacity_demand_outbox(event_id,tenant_id,project_id,branch_id,engineering_run_id,
 work_item_id,task_execution_run_id,occurrence_id,schema_version,idempotency_key,correlation_id,payload,occurred_at)
 VALUES((d->>'event_id')::UUID,r.tenant_id,r.project_id,i.branch_id,i.engineering_run_id,r.work_item_id,r.run_id,
 i.occurrence_id,2,'capacity-demand:v2:'||i.occurrence_id::TEXT,i.correlation_id,d,(d->>'requested_at')::TIMESTAMPTZ);
 INSERT INTO multica.schedule_confirmation_inbox(tenant_id,request_id,resolution_id,source_schedule_service_id,demand_event_id,resolution_wire)
 VALUES(i.tenant_id,i.request_id,(w->>'resolution_id')::UUID,p_service,(d->>'event_id')::UUID,p_wire);
 RETURN d::TEXT;
END $$;
REVOKE ALL ON FUNCTION multica.confirm_schedule_preparation(BYTEA,UUID,TEXT,TEXT) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.confirm_schedule_preparation(BYTEA,UUID,TEXT,TEXT) TO star_multica_run_owner;
COMMENT ON TABLE multica.schedule_confirmation_inbox IS
 'Transaction: immutable exact Schedule confirmation receipt paired with Run confirmation event and referenced Run-owner demand. No TTL/physical deletion; FORCE tenant RLS. Not executor authority or proof of Schedule producer authentication.';
COMMIT;
