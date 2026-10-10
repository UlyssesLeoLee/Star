-- Atomic non-executing Schedule preparation. Infrastructure demand follows confirmation.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-schedule-preparation.sql"})
-- MERGE (prepare:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-preparation.sql::prepare_schedule_execution_run",kind:"function"})
-- MERGE (self)-[:DEFINES]->(prepare)
-- MERGE (prepare)-[:CALLS]->(task:Symbol {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql::verify_schedule_task_current",kind:"function"})
-- MERGE (prepare)-[:CALLS]->(capacity:Symbol {id:"db/migrations/2026-10-10-run-owner-project-capacity.sql::read_schedule_project_capacity_current",kind:"function"})
-- MERGE (prepare)-[:CALLS]->(policies:Symbol {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql::read_schedule_policies_current",kind:"function"})
-- MERGE (prepare)-[:READS]->(metadata:Table {id:"multica.task_metadata"})
-- MERGE (prepare)-[:READS]->(lifecycle:Table {id:"multica.task_lifecycle_current"})
-- MERGE (prepare)-[:READS]->(contract:Table {id:"multica.task_contract"})
-- MERGE (prepare)-[:READS]->(binding:Table {id:"multica.run_infrastructure_binding_revision"})
-- MERGE (prepare)-[:WRITES]->(run:Table {id:"multica.task_execution_run"})
-- MERGE (prepare)-[:WRITES]->(reservation:Table {id:"multica.project_execution_resource_reservation"})
-- MERGE (prepare)-[:WRITES]->(reservation_event:Table {id:"multica.project_execution_resource_reservation_event"})
-- MERGE (prepare)-[:WRITES]->(run_event:Table {id:"multica.task_execution_run_event"})
-- MERGE (self)-[:CONFIGURES]->(run)
-- MERGE (prepare)-[:READS]->(project_binding:Table {id:"multica.worktree_project_binding"})
-- MERGE (prepare)-[:READS]->(run_binding:Table {id:"multica.engineering_run_worktree_binding"})
-- MERGE (prepare)-[:READS]->(project_grant:Table {id:"permission.project_role_binding"})
-- MERGE (prepare)-[:READS]->(branch_revision:Table {id:"scm.cloud_branch_revision"})
-- MERGE (prepare)-[:READS]->(branch_grant:Table {id:"permission.cloud_branch_role_binding"})
-- MERGE (prepare)-[:READS]->(run_revision:Table {id:"multica.engineering_run_revision"})
-- MERGE (prepare)-[:READS]->(run_grant:Table {id:"permission.engineering_run_role_binding"})
-- MERGE (pair:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-preparation.sql::require_schedule_preparation_pair",kind:"function"})
-- MERGE (self)-[:DEFINES]->(pair)
-- MERGE (pair)-[:READS]->(reservation)
-- MERGE (pair)-[:READS]->(results:Table {id:"multica.schedule_admission_result_outbox"})
-- @endcypher
BEGIN;
ALTER TABLE multica.task_execution_run
 ADD COLUMN IF NOT EXISTS schedule_admission_request_id UUID,
 ADD COLUMN IF NOT EXISTS infrastructure_binding_snapshot JSONB,
 ADD COLUMN IF NOT EXISTS infrastructure_binding_digest BYTEA;
DO $$ BEGIN
 IF NOT EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='multica.task_execution_run'::regclass AND conname='ck_run_preparation_binding') THEN
 ALTER TABLE multica.task_execution_run ADD CONSTRAINT ck_run_preparation_binding CHECK (
  (schedule_admission_request_id IS NULL AND infrastructure_binding_snapshot IS NULL AND infrastructure_binding_digest IS NULL)
  OR (schedule_admission_request_id IS NOT NULL AND run_origin='schedule'
   AND infrastructure_binding_snapshot IS NOT NULL AND jsonb_typeof(infrastructure_binding_snapshot)='object'
   AND octet_length(infrastructure_binding_snapshot::TEXT)<=16384
   AND infrastructure_binding_digest IS NOT NULL AND octet_length(infrastructure_binding_digest)=32));
 ALTER TABLE multica.task_execution_run ADD CONSTRAINT fk_run_preparation_inbox
 FOREIGN KEY(tenant_id,schedule_admission_request_id) REFERENCES multica.schedule_admission_inbox(tenant_id,request_id)
 DEFERRABLE INITIALLY DEFERRED;
 END IF;
END $$;
CREATE OR REPLACE FUNCTION multica.require_schedule_preparation_pair()
RETURNS TRIGGER LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
BEGIN
 IF NEW.schedule_admission_request_id IS NOT NULL AND NOT EXISTS (
  SELECT 1 FROM multica.schedule_admission_result_outbox o
  JOIN multica.project_execution_resource_reservation r ON r.tenant_id=o.tenant_id AND r.run_id=o.task_execution_run_id
  WHERE o.tenant_id=NEW.tenant_id AND o.request_id=NEW.schedule_admission_request_id
  AND o.task_execution_run_id=NEW.run_id AND o.outcome='accepted' AND r.reservation_state='pending'
 ) THEN RAISE EXCEPTION 'Schedule preparation requires accepted result and pending Project reservation'; END IF;
 RETURN NULL;
END $$;
DROP TRIGGER IF EXISTS schedule_preparation_pair ON multica.task_execution_run;
CREATE CONSTRAINT TRIGGER schedule_preparation_pair AFTER INSERT ON multica.task_execution_run
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION multica.require_schedule_preparation_pair();
REVOKE ALL ON FUNCTION multica.require_schedule_preparation_pair() FROM PUBLIC;
CREATE OR REPLACE FUNCTION multica.prepare_schedule_execution_run(
 p_wire BYTEA,p_profile TEXT,p_run_id UUID,p_reservation_id UUID,p_fence_id UUID
) RETURNS TEXT LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE cmd JSONB; t JSONB; profile JSONB; policies RECORD; hook JSONB; current_profile JSONB; current_profile_version BIGINT;
 tenant UUID; actor UUID; project UUID; branch UUID; erun UUID; repo UUID; wt UUID; task UUID;
 m RECORD; l RECORD; c RECORD; b RECORD; links RECORD; capacity JSONB; q JSONB; u JSONB; budget JSONB;
 key TEXT; reserved_key TEXT; expiry TIMESTAMPTZ; deadline TIMESTAMPTZ; task_doc JSONB; acceptance JSONB;
 directory JSONB; details JSONB; event_id UUID:=gen_random_uuid();
BEGIN
 IF p_wire IS NULL OR octet_length(p_wire)>16384 OR p_profile IS NULL OR octet_length(p_profile)>73728
 OR p_run_id IS NULL OR p_reservation_id IS NULL OR p_fence_id IS NULL
 OR p_run_id='00000000-0000-0000-0000-000000000000'::UUID
 OR p_reservation_id='00000000-0000-0000-0000-000000000000'::UUID
 OR p_fence_id='00000000-0000-0000-0000-000000000000'::UUID THEN RETURN 'target_unavailable'; END IF;
 cmd:=convert_from(p_wire,'UTF8')::JSONB; profile:=p_profile::JSONB;
 t:=cmd#>'{occurrence,rule_snapshot,target}';
 tenant:=(cmd->>'tenant_id')::UUID; actor:=(cmd#>>'{occurrence,run_as_actor_id}')::UUID;
 project:=(t->>'project_id')::UUID; branch:=(t->>'branch_id')::UUID; erun:=(t->>'engineering_run_id')::UUID;
 repo:=(t->>'repository_id')::UUID; wt:=(t->>'worktree_id')::UUID; task:=(t->>'work_item_id')::UUID;
 IF NOT multica.verify_schedule_task_current(tenant,actor,project,branch,erun,repo,wt,task) THEN RETURN 'target_unavailable'; END IF;
 deadline:=(cmd#>>'{occurrence,key,scheduled_for_utc}')::TIMESTAMPTZ+
 (cmd#>>'{occurrence,rule_snapshot,deadline_seconds}')::BIGINT*interval '1 second';
 IF deadline IS NULL OR deadline<=clock_timestamp() THEN RETURN 'occurrence_expired'; END IF;
 IF (cmd#>>'{occurrence,key,scheduled_for_utc}')::TIMESTAMPTZ>clock_timestamp() THEN RETURN 'target_unavailable'; END IF;
 SELECT s.* INTO l FROM multica.task_lifecycle_current s WHERE s.tenant_id=tenant AND s.work_item_id=task FOR SHARE;
 IF NOT FOUND OR l.status<>'in_progress' OR l.review_state='pending_review'
 OR l.claimed_by IS DISTINCT FROM actor OR l.active_worktree_id IS DISTINCT FROM wt
 THEN RETURN 'target_unavailable'; END IF;
 SELECT s.* INTO m FROM multica.task_metadata s WHERE s.tenant_id=tenant AND s.project_id=project
 AND s.work_item_id=task AND s.valid_from<=clock_timestamp() AND s.valid_to IS NULL FOR SHARE;
 SELECT s.* INTO c FROM multica.task_contract s WHERE s.tenant_id=tenant AND s.project_id=project
 AND s.work_item_id=task AND s.valid_from<=clock_timestamp() AND s.valid_to IS NULL FOR SHARE;
 IF NOT FOUND OR jsonb_array_length(c.acceptance_criteria) NOT BETWEEN 1 AND 128
 OR jsonb_array_length(c.dependencies)>128 OR octet_length(c.goal)>4096
 OR octet_length(c.scope::TEXT)>8192 OR octet_length(c.dependencies::TEXT)>8192
 OR octet_length(c.acceptance_criteria::TEXT)>16384 OR octet_length(m.description)>16384
 OR cardinality(m.labels)>64 OR octet_length(m.labels::TEXT)>8192 THEN RETURN 'target_unavailable'; END IF;
 task_doc:=jsonb_build_object('metadata_version',m.version,'item_type',m.item_type,'title',m.title,
 'description',m.description,'priority',m.priority,'labels',to_jsonb(m.labels),
 'lifecycle_version',l.version,'lifecycle_status',l.status);
 acceptance:=jsonb_build_object('goal',c.goal,'scope',c.scope,'dependencies',c.dependencies,'acceptance_criteria',c.acceptance_criteria);
 IF octet_length(task_doc::TEXT)>32768 OR octet_length(acceptance::TEXT)>32768 THEN RETURN 'target_unavailable'; END IF;
 FOR policies IN SELECT * FROM multica.read_schedule_policies_current(tenant,actor,project,branch,erun,repo,wt,task,(t->>'execution_profile_id')::UUID) LOOP
  IF policies.kind='profile' THEN current_profile:=policies.payload::JSONB->'profile_document'; current_profile_version:=(policies.payload::JSONB->>'profile_version')::BIGINT;
  ELSE hook:=policies.payload::JSONB->'policy_document'; END IF;
 END LOOP;
 IF current_profile IS NULL OR hook IS NULL OR profile IS DISTINCT FROM current_profile
 OR profile->>'content_digest' IS DISTINCT FROM t->>'execution_profile_digest'
 OR current_profile_version IS DISTINCT FROM (t->>'execution_profile_version')::BIGINT
 THEN RETURN 'target_unavailable'; END IF;
 SELECT r.* INTO b FROM multica.run_infrastructure_binding_revision r WHERE r.tenant_id=tenant
 AND r.project_id=project AND r.branch_id=branch AND r.engineering_run_id=erun
 AND r.valid_from<=clock_timestamp() AND r.valid_to IS NULL FOR SHARE;
 IF NOT FOUND OR b.binding_snapshot IS DISTINCT FROM cmd->'infrastructure_binding' THEN RETURN 'binding_unavailable'; END IF;
 budget:=profile#>'{profile,resource_budget}';
 IF budget IS DISTINCT FROM b.binding_snapshot->'resource_budget' THEN RETURN 'binding_unavailable'; END IF;
 capacity:=multica.read_schedule_project_capacity_current(tenant,actor,project,branch,erun,repo,wt,task)::JSONB;
 IF capacity IS NULL THEN RETURN 'quota_exceeded'; END IF;
 q:=capacity->'quota'; u:=capacity->'usage';
 IF (u->>'active_run_count')::NUMERIC+1>(q->>'max_active_runs')::NUMERIC
 OR (budget->>'max_cpu_ms')::NUMERIC>(q->>'max_cpu_ms_per_run')::NUMERIC
 OR (budget->>'max_runtime_ms')::NUMERIC>(q->>'max_runtime_ms_per_run')::NUMERIC THEN RETURN 'quota_exceeded'; END IF;
 FOREACH key IN ARRAY ARRAY['max_rss_bytes','max_child_processes','max_parallel_tools','max_provider_calls','max_output_bytes','max_event_buffer_bytes'] LOOP
  reserved_key:='reserved_' || substring(key FROM 5);
  IF (budget->>key)::NUMERIC+(u->>reserved_key)::NUMERIC>(q->>key)::NUMERIC THEN RETURN 'quota_exceeded'; END IF;
 END LOOP;
 IF EXISTS(SELECT 1 FROM multica.project_execution_resource_reservation r WHERE r.tenant_id=tenant AND r.project_id=project
  AND r.work_item_id=task AND (r.reservation_state='active' OR (r.reservation_state='pending' AND r.lease_expires_at>clock_timestamp())))
 THEN RETURN 'quota_exceeded'; END IF;
 SELECT r.binding_id,r.project_binding_id,r.version,p.version AS project_version INTO links
 FROM multica.engineering_run_worktree_binding r JOIN multica.worktree_project_binding p ON p.binding_id=r.project_binding_id
 WHERE r.tenant_id=tenant AND r.project_id=project AND r.engineering_run_id=erun AND r.worktree_id=wt
 AND r.valid_to IS NULL AND p.valid_to IS NULL FOR SHARE OF r,p;
 IF NOT FOUND THEN RETURN 'target_unavailable'; END IF;
 SELECT jsonb_build_object('tenant_id',tenant,'project_id',project,'repository_id',repo,'branch_id',branch,
 'engineering_run_id',erun,'worktree_id',wt,'work_item_id',task,
 'worktree_project_binding_id',links.project_binding_id,'worktree_project_binding_version',links.project_version,
 'engineering_run_worktree_binding_id',links.binding_id,'engineering_run_worktree_binding_version',links.version,
 'project_role_binding_id',pg.id,'project_role_binding_version',pg.version,'project_role',pg.role,
 'branch_revision_id',br.revision_id,'branch_revision_version',br.version,'branch_full_ref',br.full_ref,
 'branch_role_binding_id',bg.binding_id,'branch_role_binding_version',bg.version,'branch_role',bg.role,
 'engineering_run_revision_id',rr.revision_id,'engineering_run_revision_version',rr.version,
 'engineering_run_role_binding_id',rg.binding_id,'engineering_run_role_binding_version',rg.version,'engineering_run_role',rg.role)
 INTO directory FROM permission.project_role_binding pg
 JOIN scm.cloud_branch_revision br ON br.tenant_id=pg.tenant_id AND br.branch_id=branch
  AND br.valid_from<=clock_timestamp() AND br.valid_to IS NULL AND br.state='active'
 JOIN permission.cloud_branch_role_binding bg ON bg.tenant_id=pg.tenant_id AND bg.branch_id=br.branch_id
  AND bg.user_id=actor AND bg.valid_from<=clock_timestamp() AND bg.valid_to IS NULL
  AND bg.role IN ('tenant_admin','project_admin','developer','agent')
 JOIN multica.engineering_run_revision rr ON rr.tenant_id=pg.tenant_id AND rr.engineering_run_id=erun
  AND rr.valid_from<=clock_timestamp() AND rr.valid_to IS NULL AND rr.state='active'
 JOIN permission.engineering_run_role_binding rg ON rg.tenant_id=rr.tenant_id AND rg.engineering_run_id=rr.engineering_run_id
  AND rg.user_id=actor AND rg.valid_from<=clock_timestamp() AND rg.valid_to IS NULL
  AND rg.role IN ('tenant_admin','project_admin','developer')
 WHERE pg.tenant_id=tenant AND pg.project_id=project AND pg.user_id=actor
 AND pg.valid_from<=clock_timestamp() AND pg.valid_to IS NULL AND pg.role IN ('tenant_admin','project_admin','developer')
 FOR SHARE OF pg,br,bg,rr,rg;
 IF NOT FOUND THEN RETURN 'target_unavailable'; END IF;
 expiry:=LEAST(deadline,clock_timestamp()+interval '2 minutes');
 IF deadline<=clock_timestamp() THEN RETURN 'occurrence_expired'; END IF;
 INSERT INTO multica.task_execution_run(run_id,tenant_id,project_id,work_item_id,initiated_by,execution_channel,
 worktree_id,repository_id,task_contract_version,task_snapshot,acceptance_snapshot,run_origin,
 automation_rule_id,automation_rule_version,automation_occurrence_id,automation_admission_attempt,
 execution_profile_id,execution_profile_version,execution_profile_digest,execution_profile_snapshot,
 resource_budget_snapshot,loop_policy_snapshot,hook_set_snapshot,project_engineering_manifest_snapshot,
 correlation_id,branch_id,engineering_run_id,worktree_project_binding_id,engineering_run_worktree_binding_id,
 engineering_run_snapshot,started_at,schedule_admission_request_id,infrastructure_binding_snapshot,infrastructure_binding_digest)
 VALUES(p_run_id,tenant,project,task,actor,'agent',wt,repo,c.version,task_doc,acceptance,'schedule',
 (cmd#>>'{occurrence,rule_snapshot,rule_id}')::UUID,(cmd#>>'{occurrence,rule_snapshot,rule_version}')::BIGINT,
 (cmd#>>'{occurrence,occurrence_id}')::UUID,(cmd->>'admission_attempt')::INTEGER,
 (t->>'execution_profile_id')::UUID,(t->>'execution_profile_version')::BIGINT,t->>'execution_profile_digest',profile,
 budget,profile#>'{profile,loop_budget}',jsonb_build_object('binding',profile#>'{profile,hook_set}','effective_policy_document',hook),
 NULLIF(profile#>'{profile,engineering_manifest}','null'::JSONB),(cmd->>'correlation_id')::UUID,branch,erun,
 links.project_binding_id,links.binding_id,directory,NULL,(cmd->>'request_id')::UUID,b.binding_snapshot,b.binding_digest);
 INSERT INTO multica.project_execution_resource_reservation(reservation_id,tenant_id,project_id,work_item_id,worktree_id,run_id,
 quota_version,quota_revision_id,admission_fence_id,reserved_by,correlation_id,lease_expires_at,
 max_rss_bytes,max_cpu_ms,max_runtime_ms,max_child_processes,max_parallel_tools,max_provider_calls,max_output_bytes,max_event_buffer_bytes)
 VALUES(p_reservation_id,tenant,project,task,wt,p_run_id,(q->>'quota_version')::BIGINT,(q->>'quota_revision_id')::UUID,
 p_fence_id,actor,(cmd->>'correlation_id')::UUID,expiry,
 (budget->>'max_rss_bytes')::BIGINT,(budget->>'max_cpu_ms')::BIGINT,(budget->>'max_runtime_ms')::BIGINT,
 (budget->>'max_child_processes')::INTEGER,(budget->>'max_parallel_tools')::INTEGER,(budget->>'max_provider_calls')::BIGINT,
 (budget->>'max_output_bytes')::BIGINT,(budget->>'max_event_buffer_bytes')::BIGINT);
 details:=jsonb_build_object('reservation_id',p_reservation_id,'quota_revision_id',q->'quota_revision_id','quota_version',q->'quota_version',
 'admission_fence_id',p_fence_id,'reserved_maxima',budget,'resource_measurements','not_observed_at_admission',
 'stage','awaiting_schedule_confirmation','lease_expires_at',expiry);
 INSERT INTO multica.project_execution_resource_reservation_event(event_id,tenant_id,project_id,reservation_id,run_id,event_type,actor_id,correlation_id,details)
 VALUES(event_id,tenant,project,p_reservation_id,p_run_id,'reserved',actor,(cmd->>'correlation_id')::UUID,details);
 INSERT INTO multica.task_execution_run_event(event_id,tenant_id,project_id,run_id,work_item_id,event_type,actor_id,correlation_id,details)
 VALUES(event_id,tenant,project,p_run_id,task,'resource_budget_reserved',actor,(cmd->>'correlation_id')::UUID,details);
 RETURN 'prepared';
END $$;
REVOKE ALL ON FUNCTION multica.prepare_schedule_execution_run(BYTEA,TEXT,UUID,UUID,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.prepare_schedule_execution_run(BYTEA,TEXT,UUID,UUID,UUID) TO star_multica_run_owner;
COMMENT ON FUNCTION multica.prepare_schedule_execution_run(BYTEA,TEXT,UUID,UUID,UUID) IS
 'Run-owner preparation writer; no started_at, execution state, Infrastructure demand, external owner writes or Task lifecycle transition. Rust preflight and result-pair commit mandatory. Existing W/T/M: reservation W, immutable Run/events/results T; no new tables.';
COMMIT;
