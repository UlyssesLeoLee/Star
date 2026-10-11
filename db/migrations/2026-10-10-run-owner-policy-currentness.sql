-- Native scoped bounded policy reader; W/T/M new tables 0/0/0.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-policy-currentness.sql"})
-- MERGE (read:Symbol {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql::read_schedule_policies_current",kind:"function"})
-- MERGE (lock:Symbol {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql::lock_hook_policy_publication",kind:"function"})
-- MERGE (self)-[:DEFINES]->(read)
-- MERGE (self)-[:DEFINES]->(lock)
-- MERGE (trigger:Symbol {id:"db/migrations/2026-10-10-run-owner-policy-currentness.sql::hook_policy_00_publication_lock",kind:"trigger"})
-- MERGE (self)-[:DEFINES]->(trigger)
-- MERGE (trigger)-[:CALLS]->(lock)
-- MERGE (read)-[:CALLS]->(auth:Symbol {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql::verify_schedule_task_current",kind:"function"})
-- MERGE (read)-[:READS]->(profile:Table {id:"multica.agent_execution_profile"})
-- MERGE (read)-[:READS]->(hook:Table {id:"multica.hook_policy_set"})
-- @endcypher
BEGIN;
CREATE OR REPLACE FUNCTION multica.lock_hook_policy_publication()
RETURNS TRIGGER LANGUAGE plpgsql SET search_path=pg_catalog AS $$
BEGIN
 PERFORM pg_advisory_xact_lock(hashtextextended(
   'schedule-hook-currentness:' || COALESCE(NEW.tenant_id,OLD.tenant_id)::TEXT || ':' ||
   COALESCE(NEW.project_id,OLD.project_id)::TEXT,0));
 RETURN COALESCE(NEW,OLD);
END $$;
REVOKE ALL ON FUNCTION multica.lock_hook_policy_publication() FROM PUBLIC;
-- Alphabetically precedes the existing inheritance guard, avoiding lock inversion.
DROP TRIGGER IF EXISTS hook_policy_00_publication_lock ON multica.hook_policy_set;
CREATE TRIGGER hook_policy_00_publication_lock BEFORE INSERT OR UPDATE OR DELETE
 ON multica.hook_policy_set FOR EACH ROW EXECUTE FUNCTION multica.lock_hook_policy_publication();

CREATE OR REPLACE FUNCTION multica.read_schedule_policies_current(
 p_tenant UUID,p_actor UUID,p_project UUID,p_branch UUID,p_run UUID,
 p_repository UUID,p_worktree UUID,p_task UUID,p_profile UUID
) RETURNS TABLE(kind TEXT,payload TEXT)
LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE row_data RECORD;
BEGIN
 IF p_profile IS NULL OR NOT multica.verify_schedule_task_current(
  p_tenant,p_actor,p_project,p_branch,p_run,p_repository,p_worktree,p_task) THEN RETURN; END IF;
 PERFORM pg_advisory_xact_lock_shared(hashtextextended(
   'schedule-hook-currentness:' || p_tenant::TEXT || ':' || p_project::TEXT,0));
 SELECT p.* INTO row_data FROM multica.agent_execution_profile p
 WHERE p.tenant_id=p_tenant AND p.project_id=p_project AND p.profile_id=p_profile
  AND p.valid_from<=now() AND p.valid_to IS NULL AND p.lifecycle_state='active'
  AND (p.scope_kind='project' OR (p.scope_kind='worktree' AND p.worktree_id=p_worktree))
  AND octet_length(p.profile_document::TEXT)<=65536 FOR SHARE;
 IF NOT FOUND THEN RETURN; END IF;
 kind := 'profile'; payload := to_jsonb(row_data)::TEXT;
 IF octet_length(payload)>73728 THEN RETURN; END IF;
 RETURN NEXT;
 SELECT h.* INTO row_data FROM multica.hook_policy_set h
 WHERE h.tenant_id=p_tenant AND h.project_id=p_project AND h.scope_kind='project'
  AND h.worktree_id IS NULL AND h.valid_from<=now() AND h.valid_to IS NULL FOR SHARE;
 IF NOT FOUND THEN RETURN; END IF;
 kind := 'project_hook'; payload := to_jsonb(row_data)::TEXT;
 IF octet_length(payload)>73728 THEN RETURN; END IF;
 RETURN NEXT;
 SELECT h.* INTO row_data FROM multica.hook_policy_set h
 WHERE h.tenant_id=p_tenant AND h.project_id=p_project AND h.scope_kind='worktree'
  AND h.worktree_id=p_worktree AND h.valid_from<=now() AND h.valid_to IS NULL FOR SHARE;
 IF FOUND THEN
  kind := 'worktree_hook'; payload := to_jsonb(row_data)::TEXT;
  IF octet_length(payload)>73728 THEN RAISE EXCEPTION 'Policy envelope exceeds bound'; END IF;
  RETURN NEXT;
 END IF;
END $$;
REVOKE ALL ON FUNCTION multica.read_schedule_policies_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.read_schedule_policies_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID)
 TO star_multica_run_owner;
COMMENT ON FUNCTION multica.read_schedule_policies_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) IS
 'At most three 72KiB envelopes. Rust verifies documents and pinned identities. Does not authorize execution or replace current provider/skill/grant, quota or native Hook evaluation. Target function owner/RLS review required.';
COMMIT;
