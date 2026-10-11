-- Native Run-owner Project capacity fence; no Run or reservation insertion.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-project-capacity.sql"})
-- MERGE (read:Symbol {id:"db/migrations/2026-10-10-run-owner-project-capacity.sql::read_schedule_project_capacity_current",kind:"function"})
-- MERGE (self)-[:DEFINES]->(read)
-- MERGE (read)-[:CALLS]->(task:Symbol {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql::verify_schedule_task_current",kind:"function"})
-- MERGE (read)-[:WRITES]->(epoch:Table {id:"multica.project_execution_resource_admission_lock"})
-- MERGE (read)-[:READS]->(quota:Table {id:"multica.project_execution_resource_quota"})
-- MERGE (read)-[:READS]->(usage:Table {id:"multica.project_execution_resource_reservation"})
-- @endcypher
BEGIN;
CREATE OR REPLACE FUNCTION multica.read_schedule_project_capacity_current(
 p_tenant UUID,p_actor UUID,p_project UUID,p_branch UUID,p_run UUID,p_repository UUID,
 p_worktree UUID,p_task UUID
) RETURNS TEXT LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE q RECORD; usage JSONB; result JSONB;
BEGIN
 IF NOT multica.verify_schedule_task_current(p_tenant,p_actor,p_project,p_branch,p_run,p_repository,p_worktree,p_task)
 THEN RETURN NULL; END IF;
 -- Same allocation epoch as the manual Run writer. An actual write detects stale
 -- REPEATABLE READ snapshots rather than admitting against outdated reservations.
 INSERT INTO multica.project_execution_resource_admission_lock
 (tenant_id,project_id,allocation_epoch,updated_at,expires_at)
 VALUES(p_tenant,p_project,1,clock_timestamp(),clock_timestamp()+interval '30 days')
 ON CONFLICT(tenant_id,project_id) DO UPDATE SET
 allocation_epoch=multica.project_execution_resource_admission_lock.allocation_epoch+1,
 updated_at=clock_timestamp(),expires_at=clock_timestamp()+interval '30 days';
 -- Same key as the existing quota publisher; protect absence and current revision.
 PERFORM pg_advisory_xact_lock_shared(hashtextextended(p_tenant::TEXT || ':' || p_project::TEXT,0));
 SELECT r.* INTO q FROM multica.project_execution_resource_quota r
 WHERE r.tenant_id=p_tenant AND r.project_id=p_project AND r.valid_to IS NULL
 AND r.valid_from<=clock_timestamp() AND r.lifecycle_state='active' FOR SHARE;
 IF NOT FOUND THEN RETURN NULL; END IF;
 SELECT jsonb_build_object('active_run_count',count(*),
 'reserved_rss_bytes',COALESCE(sum(r.max_rss_bytes),0)::BIGINT,
 'reserved_child_processes',COALESCE(sum(r.max_child_processes),0)::BIGINT,
 'reserved_parallel_tools',COALESCE(sum(r.max_parallel_tools),0)::BIGINT,
 'reserved_provider_calls',COALESCE(sum(r.max_provider_calls),0)::BIGINT,
 'reserved_output_bytes',COALESCE(sum(r.max_output_bytes),0)::BIGINT,
 'reserved_event_buffer_bytes',COALESCE(sum(r.max_event_buffer_bytes),0)::BIGINT)
 INTO usage FROM multica.project_execution_resource_reservation r
 WHERE r.tenant_id=p_tenant AND r.project_id=p_project
 AND (r.reservation_state='active' OR
      (r.reservation_state='pending' AND r.lease_expires_at>clock_timestamp()));
 result:=jsonb_build_object('quota',jsonb_build_object(
 'quota_revision_id',q.quota_revision_id,'quota_version',q.quota_version,
 'max_active_runs',q.max_active_runs,'max_rss_bytes',q.max_rss_bytes,
 'max_cpu_ms_per_run',q.max_cpu_ms_per_run,'max_runtime_ms_per_run',q.max_runtime_ms_per_run,
 'max_child_processes',q.max_child_processes,'max_parallel_tools',q.max_parallel_tools,
 'max_provider_calls',q.max_provider_calls,'max_output_bytes',q.max_output_bytes,
 'max_event_buffer_bytes',q.max_event_buffer_bytes),'usage',usage);
 IF octet_length(result::TEXT)>4096 THEN RETURN NULL; END IF;
 RETURN result::TEXT;
END $$;
REVOKE ALL ON FUNCTION multica.read_schedule_project_capacity_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.read_schedule_project_capacity_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) TO star_multica_run_owner;
COMMENT ON FUNCTION multica.read_schedule_project_capacity_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) IS
 'Same-transaction Project capacity observation serialized with manual Run allocations; not a reservation or execution permit. New W/T/M tables 0/0/0; existing Work allocation epoch retains 30-day expiry.';
COMMIT;
