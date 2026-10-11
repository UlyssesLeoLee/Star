-- Run-owner native current catalog reader; new W/T/M tables 0/0/0.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-catalog-currentness.sql"})
-- MERGE (read:Symbol {id:"db/migrations/2026-10-10-run-owner-catalog-currentness.sql::read_schedule_catalog_current",kind:"function"})
-- MERGE (lock:Symbol {id:"db/migrations/2026-10-10-run-owner-catalog-currentness.sql::lock_execution_catalog_publication",kind:"function"})
-- MERGE (auth:Symbol {id:"db/migrations/2026-10-10-run-owner-catalog-currentness.sql::verify_schedule_run_as_current",kind:"function"})
-- MERGE (self)-[:DEFINES]->(read)
-- MERGE (self)-[:DEFINES]->(lock)
-- MERGE (self)-[:DEFINES]->(auth)
-- MERGE (auth)-[:READS]->(project_grant:Table {id:"permission.project_role_binding"})
-- MERGE (auth)-[:READS]->(branch_grant:Table {id:"permission.cloud_branch_role_binding"})
-- MERGE (auth)-[:READS]->(run_grant:Table {id:"permission.engineering_run_role_binding"})
-- MERGE (auth)-[:READS]->(run:Table {id:"multica.engineering_run"})
-- MERGE (auth)-[:READS]->(run_revision:Table {id:"multica.engineering_run_revision"})
-- MERGE (auth)-[:READS]->(branch:Table {id:"scm.cloud_branch"})
-- MERGE (auth)-[:READS]->(branch_revision:Table {id:"scm.cloud_branch_revision"})
-- MERGE (read)-[:CALLS]->(task:Symbol {id:"db/migrations/2026-10-10-run-owner-task-currentness.sql::verify_schedule_task_current",kind:"function"})
-- MERGE (read)-[:READS]->(revision:Table {id:"multica.agent_execution_catalog_revision"})
-- MERGE (read)-[:READS]->(provider:Table {id:"multica.agent_execution_provider_catalog"})
-- MERGE (read)-[:READS]->(skill:Table {id:"multica.agent_execution_skill_catalog"})
-- MERGE (read)-[:READS]->(grant:Table {id:"multica.agent_execution_grant_set"})
-- MERGE (read)-[:READS]->(pc:Table {id:"multica.agent_execution_provider_capability"})
-- MERGE (read)-[:READS]->(sc:Table {id:"multica.agent_execution_skill_capability"})
-- MERGE (read)-[:READS]->(gc:Table {id:"multica.agent_execution_grant_capability"})
-- @endcypher
BEGIN;
CREATE OR REPLACE FUNCTION multica.verify_schedule_run_as_current(
    p_tenant UUID, p_actor UUID, p_project UUID, p_branch UUID, p_run UUID, p_repository UUID
) RETURNS BOOLEAN LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog AS $$
BEGIN
    IF p_tenant IS NULL OR p_actor IS NULL OR p_project IS NULL OR p_branch IS NULL
       OR p_run IS NULL OR p_repository IS NULL
       OR p_actor = '00000000-0000-0000-0000-000000000000'::UUID
       OR NULLIF(current_setting('app.tenant_id', true), '')::UUID IS DISTINCT FROM p_tenant
       OR NULLIF(current_setting('app.actor_id', true), '')::UUID IS DISTINCT FROM p_actor THEN
        RETURN FALSE;
    END IF;
    -- Same key as the canonical ACL writer's exclusive lock; readers may coexist.
    PERFORM pg_advisory_xact_lock_shared(hashtextextended(
      'star.schedule.run-as.authorization:project:' || p_tenant::TEXT || ':' || p_project::TEXT,741391));
    PERFORM r.engineering_run_id
      FROM multica.engineering_run r
      JOIN multica.engineering_run_revision rv
        ON rv.tenant_id=r.tenant_id AND rv.engineering_run_id=r.engineering_run_id
       AND rv.valid_from<=now() AND rv.valid_to IS NULL AND rv.state='active'
      JOIN scm.cloud_branch b
        ON b.tenant_id=r.tenant_id AND b.branch_id=r.branch_id
       AND b.project_id=r.project_id AND b.repository_id=r.repository_id
      JOIN scm.cloud_branch_revision bv
        ON bv.tenant_id=b.tenant_id AND bv.branch_id=b.branch_id
       AND bv.valid_from<=now() AND bv.valid_to IS NULL AND bv.state='active'
      JOIN permission.project_role_binding p
        ON p.tenant_id=r.tenant_id AND p.project_id=r.project_id AND p.user_id=p_actor
       AND p.valid_from<=now() AND p.valid_to IS NULL
       AND p.role IN ('tenant_admin','project_admin','developer')
      JOIN permission.cloud_branch_role_binding bg
        ON bg.tenant_id=b.tenant_id AND bg.branch_id=b.branch_id AND bg.user_id=p_actor
       AND bg.valid_from<=now() AND bg.valid_to IS NULL
      JOIN permission.engineering_run_role_binding rg
        ON rg.tenant_id=r.tenant_id AND rg.engineering_run_id=r.engineering_run_id AND rg.user_id=p_actor
       AND rg.valid_from<=now() AND rg.valid_to IS NULL
       AND rg.role IN ('tenant_admin','project_admin','developer')
      WHERE r.tenant_id=p_tenant AND r.project_id=p_project AND r.branch_id=p_branch
        AND r.engineering_run_id=p_run AND r.repository_id=p_repository
      FOR SHARE OF r,rv,b,bv,p,bg,rg;
    RETURN FOUND;
END $$;

CREATE OR REPLACE FUNCTION multica.lock_execution_catalog_publication()
RETURNS TRIGGER LANGUAGE plpgsql SET search_path=pg_catalog AS $$
BEGIN
 PERFORM pg_advisory_xact_lock(hashtextextended('schedule-catalog-currentness:' ||
   COALESCE(NEW.tenant_id,OLD.tenant_id)::TEXT || ':' || COALESCE(NEW.project_id,OLD.project_id)::TEXT,0));
 RETURN COALESCE(NEW,OLD);
END $$;
REVOKE ALL ON FUNCTION multica.lock_execution_catalog_publication() FROM PUBLIC;
DO $$ DECLARE t TEXT; BEGIN
 FOREACH t IN ARRAY ARRAY['agent_execution_provider_catalog','agent_execution_skill_catalog','agent_execution_grant_set'] LOOP
  EXECUTE format('DROP TRIGGER IF EXISTS agent_execution_00_publication_lock ON multica.%I',t);
  EXECUTE format('CREATE TRIGGER agent_execution_00_publication_lock BEFORE INSERT OR UPDATE OR DELETE ON multica.%I FOR EACH ROW EXECUTE FUNCTION multica.lock_execution_catalog_publication()',t);
 END LOOP;
END $$;
CREATE OR REPLACE FUNCTION multica.read_schedule_catalog_current(
 p_tenant UUID,p_actor UUID,p_project UUID,p_branch UUID,p_run UUID,p_repository UUID,
 p_worktree UUID,p_task UUID,p_providers TEXT[],p_skills TEXT[]
) RETURNS TEXT LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $$
DECLARE rev RECORD; g RECORD; result JSONB; ps JSONB; ss JSONB; caps JSONB; heap BIGINT;
BEGIN
 IF p_providers IS NULL OR p_skills IS NULL OR cardinality(p_providers) NOT BETWEEN 1 AND 5
   OR cardinality(p_skills)>128 OR array_position(p_providers,NULL) IS NOT NULL
   OR array_position(p_skills,NULL) IS NOT NULL
   OR (SELECT count(DISTINCT v) FROM unnest(p_providers) v)<>cardinality(p_providers)
   OR (SELECT count(DISTINCT v) FROM unnest(p_skills) v)<>cardinality(p_skills)
   OR NOT multica.verify_schedule_task_current(p_tenant,p_actor,p_project,p_branch,p_run,p_repository,p_worktree,p_task)
 THEN RETURN NULL; END IF;
 PERFORM pg_advisory_xact_lock_shared(hashtextextended('schedule-catalog-currentness:' || p_tenant::TEXT || ':' || p_project::TEXT,0));
 SELECT r.* INTO rev FROM multica.agent_execution_catalog_revision r
 WHERE r.tenant_id=p_tenant AND r.project_id=p_project FOR SHARE;
 IF NOT FOUND THEN RETURN NULL; END IF;
 SELECT s.* INTO g FROM multica.agent_execution_grant_set s
 WHERE s.tenant_id=p_tenant AND s.project_id=p_project AND s.valid_from<=now() AND s.valid_to IS NULL FOR SHARE;
 IF NOT FOUND THEN RETURN NULL; END IF;
 PERFORM p.entry_id FROM multica.agent_execution_provider_catalog p WHERE p.tenant_id=p_tenant AND p.project_id=p_project
  AND p.provider_id=ANY(p_providers) AND p.valid_from<=now() AND p.valid_to IS NULL FOR SHARE;
 PERFORM s.entry_id FROM multica.agent_execution_skill_catalog s WHERE s.tenant_id=p_tenant AND s.project_id=p_project
  AND s.skill_id=ANY(p_skills) AND s.valid_from<=now() AND s.valid_to IS NULL FOR SHARE;
 SELECT COALESCE(sum(octet_length(c.capability_code)+64),0) INTO heap
 FROM multica.agent_execution_skill_capability c JOIN multica.agent_execution_skill_catalog s
 ON s.tenant_id=c.tenant_id AND s.entry_id=c.entry_id
 WHERE s.tenant_id=p_tenant AND s.project_id=p_project AND s.skill_id=ANY(p_skills)
 AND s.valid_from<=now() AND s.valid_to IS NULL;
 IF heap>393216 THEN RETURN NULL; END IF;
 IF EXISTS (SELECT 1 FROM multica.agent_execution_provider_catalog p WHERE p.tenant_id=p_tenant AND p.project_id=p_project
  AND p.provider_id=ANY(p_providers) AND p.valid_from<=now() AND p.valid_to IS NULL
  AND p.capability_count<>(SELECT count(*) FROM multica.agent_execution_provider_capability c WHERE c.tenant_id=p.tenant_id AND c.entry_id=p.entry_id))
 OR EXISTS (SELECT 1 FROM multica.agent_execution_skill_catalog s WHERE s.tenant_id=p_tenant AND s.project_id=p_project
  AND s.skill_id=ANY(p_skills) AND s.valid_from<=now() AND s.valid_to IS NULL
  AND s.capability_count<>(SELECT count(*) FROM multica.agent_execution_skill_capability c WHERE c.tenant_id=s.tenant_id AND c.entry_id=s.entry_id))
 OR g.capability_count<>(SELECT count(*) FROM multica.agent_execution_grant_capability c WHERE c.tenant_id=p_tenant AND c.entry_id=g.entry_id)
 THEN RETURN NULL; END IF;
 SELECT COALESCE(jsonb_agg(jsonb_build_object('provider',jsonb_build_object(
   'provider_id',p.provider_id,'version',p.provider_version,'implementation_digest',rtrim(p.implementation_digest::TEXT),
   'configuration_digest',rtrim(p.configuration_digest::TEXT),'capabilities',
   (SELECT COALESCE(jsonb_agg(c.capability_code ORDER BY c.capability_code COLLATE "C"),'[]'::JSONB) FROM multica.agent_execution_provider_capability c WHERE c.tenant_id=p_tenant AND c.entry_id=p.entry_id)),
   'available',p.available) ORDER BY p.provider_id COLLATE "C"),'[]'::JSONB) INTO ps
 FROM multica.agent_execution_provider_catalog p WHERE p.tenant_id=p_tenant AND p.project_id=p_project
  AND p.provider_id=ANY(p_providers) AND p.valid_from<=now() AND p.valid_to IS NULL;
 SELECT COALESCE(jsonb_agg(jsonb_build_object('skill',jsonb_build_object('skill_id',s.skill_id,'version',s.skill_version,
  'content_digest',rtrim(s.content_digest::TEXT),'capabilities',
  (SELECT COALESCE(jsonb_agg(c.capability_code ORDER BY c.capability_code COLLATE "C"),'[]'::JSONB) FROM multica.agent_execution_skill_capability c WHERE c.tenant_id=p_tenant AND c.entry_id=s.entry_id)),
  'available',s.available) ORDER BY s.skill_id COLLATE "C"),'[]'::JSONB) INTO ss
 FROM multica.agent_execution_skill_catalog s WHERE s.tenant_id=p_tenant AND s.project_id=p_project
  AND s.skill_id=ANY(p_skills) AND s.valid_from<=now() AND s.valid_to IS NULL;
 IF jsonb_array_length(ps)<>cardinality(p_providers) OR jsonb_array_length(ss)<>cardinality(p_skills) THEN RETURN NULL; END IF;
 SELECT COALESCE(jsonb_agg(c.capability_code ORDER BY c.capability_code COLLATE "C"),'[]'::JSONB) INTO caps
 FROM multica.agent_execution_grant_capability c WHERE c.tenant_id=p_tenant AND c.entry_id=g.entry_id;
 result:=jsonb_build_object('providers',ps,'skills',ss,'provider_revision',rev.provider_catalog_revision,
 'skill_revision',rev.skill_catalog_revision,'grants',jsonb_build_object('grant_set_id',g.grant_set_id,'version',g.grant_set_version,
 'capabilities',caps,'expires_at_epoch_ms',g.expires_at_epoch_ms));
 IF octet_length(result::TEXT)>524288 THEN RETURN NULL; END IF;
 RETURN result::TEXT;
END $$;
REVOKE ALL ON FUNCTION multica.read_schedule_catalog_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID,TEXT[],TEXT[]) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.read_schedule_catalog_current(UUID,UUID,UUID,UUID,UUID,UUID,UUID,UUID,TEXT[],TEXT[]) TO star_multica_run_owner;
COMMIT;
