-- Fixed-path task/checkout read capability. Adds no tables or execution permission.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-task-currentness.sql"})
-- MERGE (verify:Symbol {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql::verify_schedule_task_current",kind:"function"})
-- MERGE (self)-[:DEFINES]->(verify)
-- MERGE (verify)-[:CALLS]->(auth:Symbol {id:"db/migrations/2026-10-10-run-owner-admission-currentness.sql::verify_schedule_run_as_current",kind:"function"})
-- MERGE (verify)-[:READS]->(w:Table {id:"public.worktree_canvas_worktree"})
-- MERGE (verify)-[:READS]->(p:Table {id:"multica.worktree_project_binding"})
-- MERGE (verify)-[:READS]->(b:Table {id:"multica.engineering_run_worktree_binding"})
-- MERGE (verify)-[:READS]->(m:Table {id:"multica.task_metadata"})
-- MERGE (verify)-[:READS]->(l:Table {id:"multica.work_item_worktree"})
-- @endcypher
BEGIN;
CREATE OR REPLACE FUNCTION multica.verify_schedule_task_current(
 p_tenant UUID,p_actor UUID,p_project UUID,p_branch UUID,p_run UUID,
 p_repository UUID,p_worktree UUID,p_task UUID
) RETURNS BOOLEAN LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
BEGIN
 IF p_worktree IS NULL OR p_task IS NULL
    OR p_worktree='00000000-0000-0000-0000-000000000000'::UUID
    OR p_task='00000000-0000-0000-0000-000000000000'::UUID
    OR NOT multica.verify_schedule_run_as_current(p_tenant,p_actor,p_project,p_branch,p_run,p_repository) THEN
   RETURN FALSE;
 END IF;
 PERFORM m.metadata_id
 FROM public.worktree_canvas_worktree w
 JOIN multica.worktree_project_binding p
   ON p.tenant_id=w.tenant_id AND p.worktree_id=w.id AND p.project_id=w.project_id
  AND p.valid_from<=now() AND p.valid_to IS NULL
 JOIN multica.engineering_run_worktree_binding b
   ON b.tenant_id=p.tenant_id AND b.project_id=p.project_id AND b.repository_id=w.repo_id
  AND b.worktree_id=w.id AND b.project_binding_id=p.binding_id
  AND b.valid_from<=now() AND b.valid_to IS NULL
 JOIN multica.task_metadata m
   ON m.tenant_id=b.tenant_id AND m.project_id=b.project_id AND m.repository_id=b.repository_id
  AND m.branch_id=b.branch_id AND m.engineering_run_id=b.engineering_run_id
  AND m.work_item_id=p_task AND m.valid_from<=now() AND m.valid_to IS NULL
 JOIN multica.work_item_worktree l
   ON l.tenant_id=m.tenant_id AND l.project_id=m.project_id AND l.work_item_id=m.work_item_id
  AND l.worktree_id=w.id AND l.valid_from<=now() AND l.valid_to IS NULL
 WHERE w.tenant_id=p_tenant AND w.project_id=p_project AND w.repo_id=p_repository
   AND w.id=p_worktree AND w.archived=FALSE AND w.valid_from<=now() AND w.valid_to IS NULL
   AND b.branch_id=p_branch AND b.engineering_run_id=p_run
 FOR SHARE OF w,p,b,m,l;
 RETURN FOUND;
END $$;
REVOKE ALL ON FUNCTION multica.verify_schedule_task_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.verify_schedule_task_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID)
 TO star_multica_run_owner;
COMMENT ON FUNCTION multica.verify_schedule_task_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID) IS
 'Current canonical task/checkout membership held through owner transaction. Does not validate Profile/Hook/quota or permit execution. Target deployment function owner privileges require review.';
COMMIT;
