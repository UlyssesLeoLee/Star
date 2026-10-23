BEGIN;

-- Preserve each existing restrictive UPDATE deny policy. PostgreSQL requires
-- UPDATE privilege for SELECT ... FOR SHARE, so the USING predicate must admit
-- row locking while WITH CHECK continues to reject every real source-row write.
ALTER POLICY engineering_directory_owner_no_project_grant_update
    ON permission.project_role_binding
    USING (true) WITH CHECK (false);

ALTER POLICY engineering_directory_owner_no_worktree_binding_update
    ON multica.worktree_project_binding
    USING (true) WITH CHECK (false);

ALTER POLICY engineering_directory_owner_no_worktree_projection_update
    ON public.worktree_canvas_worktree
    USING (true) WITH CHECK (false);

ALTER POLICY engineering_directory_owner_no_run_revision_update
    ON multica.engineering_run_revision
    USING (true) WITH CHECK (false);

ALTER POLICY engineering_directory_owner_no_run_grant_update
    ON permission.engineering_run_role_binding
    USING (true) WITH CHECK (false);

COMMIT;
