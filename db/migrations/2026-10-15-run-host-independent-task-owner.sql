-- CYPHER STRUCTURAL MANIFEST
-- CREATE (f:File {name:"2026-10-15-run-host-independent-task-owner.sql",type:"file",language:"sql"}),(m:Module {name:"run_host_independent_task_owner_migration",type:"module",language:"sql"}),(metadata:Class {name:"multica.task_metadata",type:"class",language:"sql"}),(relation:Class {name:"multica.work_item_worktree",type:"class",language:"sql"}),(runBinding:Class {name:"multica.engineering_run_worktree_binding",type:"class",language:"sql"}),(audit:Class {name:"multica.task_lifecycle_audit",type:"class",language:"sql"}),(outbox:Class {name:"multica.task_run_outbox",type:"class",language:"sql"}),(run:Class {name:"multica.engineering_run",type:"class",language:"sql"}),(metadataGuard:Function {name:"multica.guard_task_metadata_project_run_owner",type:"function",language:"sql"}),(legacyGuard:Function {name:"multica.guard_task_metadata_legacy_owner_mutation",type:"function",language:"sql"}),(relationGuard:Function {name:"multica.guard_task_worktree_project_run_owner",type:"function",language:"sql"}),(bindingGuard:Function {name:"multica.guard_engineering_run_worktree_project_run_owner",type:"function",language:"sql"}),(auditGuard:Function {name:"multica.guard_task_lifecycle_audit_project_run_owner",type:"function",language:"sql"}),(outboxWriter:Function {name:"multica.emit_task_run_outbox",type:"function",language:"sql"}),(canonicalOwner:Variable {name:"tenant_project_engineering_run_owner",type:"variable",language:"sql"}),(hostContext:Variable {name:"repository_context_with_optional_remote_branch",type:"variable",language:"sql"}),(bindingRun:Variable {name:"binding_run_id",type:"variable",language:"sql"}),(taskRun:Variable {name:"task_run_id",type:"variable",language:"sql"});
-- CREATE (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(metadata),(m)-[:CONTAINS]->(relation),(m)-[:CONTAINS]->(runBinding),(m)-[:CONTAINS]->(audit),(m)-[:CONTAINS]->(outbox),(m)-[:CONTAINS]->(run),(m)-[:CONTAINS]->(metadataGuard),(m)-[:CONTAINS]->(legacyGuard),(m)-[:CONTAINS]->(relationGuard),(m)-[:CONTAINS]->(bindingGuard),(m)-[:CONTAINS]->(auditGuard),(m)-[:CONTAINS]->(outboxWriter),(metadata)-[:USES]->(canonicalOwner),(metadata)-[:USES]->(hostContext),(metadata)-[:USES]->(run),(outbox)-[:USES]->(canonicalOwner),(outbox)-[:USES]->(hostContext),(outbox)-[:USES]->(run),(runBinding)-[:USES]->(run),(audit)-[:USES]->(run),(metadataGuard)-[:USES]->(metadata),(metadataGuard)-[:USES]->(relation),(metadataGuard)-[:USES]->(runBinding),(legacyGuard)-[:USES]->(metadata),(relationGuard)-[:USES]->(metadata),(relationGuard)-[:USES]->(runBinding),(relationGuard)-[:USES]->(bindingRun),(relationGuard)-[:USES]->(taskRun),(bindingGuard)-[:USES]->(relation),(bindingGuard)-[:USES]->(metadata),(auditGuard)-[:USES]->(metadata),(auditGuard)-[:USES]->(runBinding),(outboxWriter)-[:USES]->(metadata),(outboxWriter)-[:USES]->(outbox),(metadata)-[:CALLS]->(metadataGuard),(metadata)-[:CALLS]->(legacyGuard),(relation)-[:CALLS]->(relationGuard),(runBinding)-[:CALLS]->(bindingGuard),(audit)-[:CALLS]->(auditGuard),(metadata)-[:CALLS]->(outboxWriter);

-- Additive Run-owned Task Card schema. Existing unassigned rows remain unassigned;
-- no Worktree, remote Branch, or workspace inference/backfill is performed.
-- W/T/M: task_metadata and work_item_worktree are Master/SCD2; workspace_id is
-- optional context only. task_lifecycle_audit and task_run_outbox are append-only
-- Transaction records. Existing lifecycle current/idempotency Work tables retain
-- their established 365-day/30-day retention contracts.
-- Application writers must acquire Project/Run → target Run/Worktree binding →
-- Task metadata → Task/Worktree association → lifecycle/audit. The relation and
-- binding guards below order their explicit locks consistently where applicable;
-- other trigger paths still require call-path review. This is not a deadlock proof.
-- Existing RLS policies and privileges are preserved; this migration adds no GRANTs
-- because it does not establish the deployed application-role privilege contract.
-- Schedule and CLI execution-admission schemas are intentionally unchanged and
-- remain fail-closed until their Branch-dependent contracts are separately migrated.
BEGIN;

-- Project + Run is the canonical Task owner. Repository/remote Branch and workspace
-- are optional context and are never inferred from a Worktree or a host connection.
ALTER TABLE multica.task_metadata
    ALTER COLUMN workspace_id DROP NOT NULL;

ALTER TABLE multica.task_metadata
    DROP CONSTRAINT IF EXISTS ck_task_metadata_run_owner_complete,
    DROP CONSTRAINT IF EXISTS fk_task_metadata_engineering_run_scope;

ALTER TABLE multica.task_metadata
    ADD CONSTRAINT ck_task_metadata_branch_requires_repository
        CHECK (branch_id IS NULL OR repository_id IS NOT NULL),
    ADD CONSTRAINT fk_task_metadata_project_run_owner
        FOREIGN KEY (tenant_id, project_id, engineering_run_id)
        REFERENCES multica.engineering_run
            (tenant_id, project_id, engineering_run_id)
        ON DELETE RESTRICT;

COMMENT ON COLUMN multica.task_metadata.workspace_id IS
    'Optional legacy/workspace context. Run-owned Task Cards may omit it; this value is not inferred from Project, Run, Worktree, or remote Branch.';
COMMENT ON COLUMN multica.task_metadata.engineering_run_id IS
    'Canonical Project-scoped Task owner. Null current rows are legacy and remain unassigned until an explicit authorized SCD2 operation.';
COMMENT ON COLUMN multica.task_metadata.repository_id IS
    'Optional host repository context; not part of the canonical Task owner or authorization prerequisite.';
COMMENT ON COLUMN multica.task_metadata.branch_id IS
    'Optional remote host Branch context; not part of the canonical Task owner or authorization prerequisite.';
COMMENT ON TABLE multica.task_metadata IS
    'M/SCD2: canonical Task Card master metadata. Owner is tenant + Project + Engineering Run; workspace and host Branch context are optional.';
COMMENT ON TABLE multica.work_item_worktree IS
    'M/SCD2: optional explicit Task Card-to-Worktree association, constrained to the same Project + Engineering Run.';
COMMENT ON TABLE multica.task_lifecycle_current IS
    'W: rebuildable current Task lifecycle projection with the established 365-day retention contract.';
COMMENT ON TABLE multica.task_command_idempotency IS
    'W: actor-scoped command replay records with the established 30-day expiry contract.';

CREATE OR REPLACE FUNCTION multica.guard_task_metadata_project_run_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    has_current_worktree_outside_run BOOLEAN;
BEGIN
    IF TG_OP = 'UPDATE' AND OLD.valid_to IS NULL AND NEW.valid_to IS NULL AND
       (OLD.project_id, OLD.workspace_id, OLD.repository_id, OLD.branch_id, OLD.engineering_run_id)
           IS DISTINCT FROM
       (NEW.project_id, NEW.workspace_id, NEW.repository_id, NEW.branch_id, NEW.engineering_run_id) THEN
        RAISE EXCEPTION 'Task owner or owner context changes require a new SCD2 metadata version'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_project_run_owner_scd2';
    END IF;

    IF TG_OP = 'UPDATE' AND OLD.valid_to IS NULL AND NEW.valid_to IS NULL
       AND NEW.engineering_run_id IS NULL THEN
        RAISE EXCEPTION 'legacy unassigned Task Cards must be explicitly versioned into a Project + Run before mutation'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_legacy_owner_migration_required';
    END IF;

    -- Legacy current rows may remain untouched. Every newly inserted or successor
    -- current row must have a real Project + Run owner; a host Branch is optional.
    IF NEW.valid_to IS NULL THEN
        IF NEW.engineering_run_id IS NULL THEN
            RAISE EXCEPTION 'new current Task Cards require explicit Project + Engineering Run ownership'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_project_run_owner_required';
        END IF;

        -- The association is optional. If present, each active Worktree must already
        -- be bound to this Project + Run; a remote Branch match is deliberately unused.
        PERFORM 1
          FROM multica.work_item_worktree relation
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.work_item_id = NEW.work_item_id
           AND relation.valid_to IS NULL
         ORDER BY relation.relation_id
         FOR SHARE OF relation;

        PERFORM 1
          FROM multica.work_item_worktree relation
          JOIN multica.engineering_run_worktree_binding binding
            ON binding.tenant_id = relation.tenant_id
           AND binding.project_id = relation.project_id
           AND binding.worktree_id = relation.worktree_id
           AND binding.valid_to IS NULL
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.work_item_id = NEW.work_item_id
           AND relation.valid_to IS NULL
         ORDER BY relation.relation_id
         FOR SHARE OF binding;

        SELECT EXISTS (
            SELECT 1
              FROM multica.work_item_worktree relation
              LEFT JOIN multica.engineering_run_worktree_binding binding
                ON binding.tenant_id = relation.tenant_id
               AND binding.project_id = relation.project_id
               AND binding.worktree_id = relation.worktree_id
               AND binding.valid_to IS NULL
             WHERE relation.tenant_id = NEW.tenant_id
               AND relation.work_item_id = NEW.work_item_id
               AND relation.valid_to IS NULL
               AND (relation.project_id IS DISTINCT FROM NEW.project_id
                    OR binding.engineering_run_id IS NULL
                    OR binding.project_id IS DISTINCT FROM NEW.project_id
                    OR binding.engineering_run_id IS DISTINCT FROM NEW.engineering_run_id)
        ) INTO has_current_worktree_outside_run;

        IF has_current_worktree_outside_run THEN
            RAISE EXCEPTION 'Task Card cannot be owned by a Run different from an associated Worktree Run'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_worktree_project_run_match';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS task_metadata_run_owner_guard ON multica.task_metadata;
CREATE TRIGGER task_metadata_run_owner_guard
    BEFORE INSERT OR UPDATE OF project_id, workspace_id, repository_id, branch_id, engineering_run_id, valid_to
    ON multica.task_metadata
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_metadata_project_run_owner();

CREATE OR REPLACE FUNCTION multica.guard_task_metadata_legacy_owner_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.valid_to IS NULL AND NEW.valid_to IS NULL AND OLD.engineering_run_id IS NULL THEN
        RAISE EXCEPTION 'legacy unassigned Task Cards must be explicitly versioned into a Project + Run before mutation'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_metadata_legacy_owner_migration_required';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS task_metadata_legacy_owner_update_guard ON multica.task_metadata;
CREATE TRIGGER task_metadata_legacy_owner_update_guard
    BEFORE UPDATE ON multica.task_metadata
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_metadata_legacy_owner_mutation();

CREATE OR REPLACE FUNCTION multica.guard_task_worktree_project_run_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    binding_run_id UUID;
    task_run_id UUID;
BEGIN
    IF NEW.valid_to IS NULL THEN
        -- Lock the current Run → Worktree binding before Task metadata, matching
        -- the API writer's binding-first order for this association operation.
        SELECT binding.engineering_run_id
          INTO binding_run_id
          FROM multica.engineering_run_worktree_binding binding
         WHERE binding.tenant_id = NEW.tenant_id
           AND binding.project_id = NEW.project_id
           AND binding.worktree_id = NEW.worktree_id
           AND binding.valid_to IS NULL
         FOR SHARE OF binding;

        IF NOT FOUND OR binding_run_id IS NULL THEN
            RAISE EXCEPTION 'Worktree association requires a current Project + Run binding'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_work_item_worktree_project_run_owner';
        END IF;

        SELECT metadata.engineering_run_id
          INTO task_run_id
          FROM multica.task_metadata metadata
         WHERE metadata.tenant_id = NEW.tenant_id
           AND metadata.project_id = NEW.project_id
           AND metadata.work_item_id = NEW.work_item_id
           AND metadata.valid_to IS NULL
         FOR SHARE OF metadata;

        IF NOT FOUND OR task_run_id IS NULL THEN
            RAISE EXCEPTION 'Worktree association requires a current Project + Run-owned Task Card'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_work_item_worktree_project_run_owner';
        END IF;

        IF task_run_id IS DISTINCT FROM binding_run_id THEN
            RAISE EXCEPTION 'Task Card and Worktree must share the same current Project + Run'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_work_item_worktree_project_run_match';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS work_item_worktree_run_owner_guard ON multica.work_item_worktree;
CREATE TRIGGER work_item_worktree_run_owner_guard
    BEFORE INSERT OR UPDATE OF tenant_id, project_id, work_item_id, worktree_id, valid_to
    ON multica.work_item_worktree
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_worktree_project_run_owner();

CREATE OR REPLACE FUNCTION multica.guard_engineering_run_worktree_project_run_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    has_task_outside_run BOOLEAN;
BEGIN
    IF NEW.valid_to IS NULL THEN
        -- Callers acquire Run/binding identity first; lock dependent Task metadata
        -- before associations. INSERT does not yet have a persisted target row, so
        -- this trigger alone cannot establish the caller's full lock order.
        PERFORM 1
          FROM multica.work_item_worktree relation
          JOIN multica.task_metadata metadata
            ON metadata.tenant_id = relation.tenant_id
           AND metadata.project_id = relation.project_id
           AND metadata.work_item_id = relation.work_item_id
           AND metadata.valid_to IS NULL
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.project_id = NEW.project_id
           AND relation.worktree_id = NEW.worktree_id
           AND relation.valid_to IS NULL
         ORDER BY metadata.metadata_id
         FOR SHARE OF metadata;

        PERFORM 1
          FROM multica.work_item_worktree relation
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.project_id = NEW.project_id
           AND relation.worktree_id = NEW.worktree_id
           AND relation.valid_to IS NULL
         ORDER BY relation.relation_id
         FOR SHARE OF relation;

        SELECT EXISTS (
            SELECT 1
              FROM multica.work_item_worktree relation
              LEFT JOIN multica.task_metadata metadata
                ON metadata.tenant_id = relation.tenant_id
               AND metadata.project_id = relation.project_id
               AND metadata.work_item_id = relation.work_item_id
               AND metadata.valid_to IS NULL
             WHERE relation.tenant_id = NEW.tenant_id
               AND relation.project_id = NEW.project_id
               AND relation.worktree_id = NEW.worktree_id
               AND relation.valid_to IS NULL
               AND (metadata.metadata_id IS NULL
                    OR metadata.engineering_run_id IS NULL
                    OR metadata.project_id IS DISTINCT FROM NEW.project_id
                    OR metadata.engineering_run_id IS DISTINCT FROM NEW.engineering_run_id)
        ) INTO has_task_outside_run;

        IF has_task_outside_run THEN
            RAISE EXCEPTION 'Worktree cannot be bound to a Run different from an associated current Task Card'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_engineering_run_worktree_task_project_run_match';
        END IF;
    ELSIF TG_OP = 'UPDATE' AND OLD.valid_to IS NULL AND EXISTS (
        SELECT 1
          FROM multica.work_item_worktree relation
         WHERE relation.tenant_id = OLD.tenant_id
           AND relation.project_id = OLD.project_id
           AND relation.worktree_id = OLD.worktree_id
           AND relation.valid_to IS NULL
    ) THEN
        RAISE EXCEPTION 'Close active Task-to-Worktree associations before closing the current Run binding'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_engineering_run_worktree_active_task_association';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS engineering_run_worktree_task_owner_guard ON multica.engineering_run_worktree_binding;
CREATE TRIGGER engineering_run_worktree_task_owner_guard
    BEFORE INSERT OR UPDATE OF tenant_id, project_id, engineering_run_id, worktree_id, valid_to
    ON multica.engineering_run_worktree_binding
    FOR EACH ROW EXECUTE FUNCTION multica.guard_engineering_run_worktree_project_run_owner();

-- Existing audit rows keep their original Worktree identity and remain immutable.
-- New lifecycle events carry canonical Run identity and may omit the Worktree.
ALTER TABLE multica.task_lifecycle_audit
    ALTER COLUMN worktree_id DROP NOT NULL,
    ADD COLUMN engineering_run_id UUID,
    ADD CONSTRAINT ck_task_lifecycle_audit_owner_present
        CHECK (worktree_id IS NOT NULL OR engineering_run_id IS NOT NULL),
    ADD CONSTRAINT fk_task_lifecycle_audit_project_run
        FOREIGN KEY (tenant_id, project_id, engineering_run_id)
        REFERENCES multica.engineering_run
            (tenant_id, project_id, engineering_run_id)
        ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_task_lifecycle_audit_run_item_time
    ON multica.task_lifecycle_audit
        (tenant_id, project_id, engineering_run_id, work_item_id, occurred_at DESC)
    WHERE engineering_run_id IS NOT NULL;

CREATE OR REPLACE FUNCTION multica.guard_task_lifecycle_audit_project_run_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    task_run_id UUID;
BEGIN
    IF NEW.task_card_id IS DISTINCT FROM NEW.work_item_id THEN
        RAISE EXCEPTION 'Task lifecycle audit card identity must match its Work Item'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_audit_card_identity';
    END IF;

    SELECT metadata.engineering_run_id
      INTO task_run_id
      FROM multica.task_metadata metadata
     WHERE metadata.tenant_id = NEW.tenant_id
       AND metadata.project_id = NEW.project_id
       AND metadata.work_item_id = NEW.work_item_id
       AND metadata.valid_to IS NULL
     FOR SHARE OF metadata;

    IF NOT FOUND OR task_run_id IS NULL THEN
        RAISE EXCEPTION 'New lifecycle evidence requires a current Project + Run-owned Task Card'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_audit_current_task_run';
    END IF;

    IF NEW.engineering_run_id IS NOT NULL AND NEW.engineering_run_id <> task_run_id THEN
        RAISE EXCEPTION 'Lifecycle evidence Run must match the current Task owner'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_audit_run_matches_task';
    END IF;
    NEW.engineering_run_id := task_run_id;

    IF NEW.worktree_id IS NOT NULL THEN
        PERFORM 1
          FROM multica.work_item_worktree relation
         WHERE relation.tenant_id = NEW.tenant_id
           AND relation.project_id = NEW.project_id
           AND relation.work_item_id = NEW.work_item_id
           AND relation.worktree_id = NEW.worktree_id
           AND relation.valid_to IS NULL
         FOR SHARE OF relation;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'Lifecycle Worktree must be explicitly associated with the current Task Card'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_audit_work_item_worktree_relation';
        END IF;

        PERFORM 1
          FROM multica.engineering_run_worktree_binding binding
         WHERE binding.tenant_id = NEW.tenant_id
           AND binding.project_id = NEW.project_id
           AND binding.worktree_id = NEW.worktree_id
           AND binding.engineering_run_id = task_run_id
           AND binding.valid_to IS NULL
         FOR SHARE OF binding;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'Lifecycle Worktree must belong to the current Task Project + Run'
                USING ERRCODE = '23514', CONSTRAINT = 'ck_task_lifecycle_audit_worktree_project_run';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS task_lifecycle_audit_run_owner_guard ON multica.task_lifecycle_audit;
CREATE TRIGGER task_lifecycle_audit_run_owner_guard
    BEFORE INSERT ON multica.task_lifecycle_audit
    FOR EACH ROW EXECUTE FUNCTION multica.guard_task_lifecycle_audit_project_run_owner();

COMMENT ON COLUMN multica.task_lifecycle_audit.worktree_id IS
    'Optional execution context for new Run-scoped lifecycle evidence; historical rows retain their original non-null Worktree.';
COMMENT ON COLUMN multica.task_lifecycle_audit.engineering_run_id IS
    'Canonical owner scope for newly appended lifecycle evidence. Existing historical rows remain NULL and unchanged.';
COMMENT ON TABLE multica.task_lifecycle_audit IS
    'T: append-only lifecycle evidence. New records are Run-scoped; Worktree context is optional and historical rows are immutable.';

-- Evolve the append-only Outbox without rewriting v1 events. New producer events
-- use v2, where Project + Run are canonical and host Branch context may be absent.
ALTER TABLE multica.task_run_outbox
    ALTER COLUMN repository_id DROP NOT NULL,
    ALTER COLUMN branch_id DROP NOT NULL,
    ALTER COLUMN schema_version SET DEFAULT 2;

DO $$
DECLARE
    constraint_name TEXT;
BEGIN
    FOR constraint_name IN
        SELECT conname
          FROM pg_constraint
         WHERE conrelid = 'multica.task_run_outbox'::regclass
           AND confrelid = 'multica.engineering_run'::regclass
           AND contype = 'f'
    LOOP
        EXECUTE format('ALTER TABLE multica.task_run_outbox DROP CONSTRAINT %I', constraint_name);
    END LOOP;
END;
$$;

ALTER TABLE multica.task_run_outbox
    DROP CONSTRAINT IF EXISTS task_run_outbox_schema_version_check,
    ADD CONSTRAINT ck_task_run_outbox_schema_version
        CHECK (schema_version IN (1, 2)),
    ADD CONSTRAINT ck_task_run_outbox_branch_requires_repository
        CHECK (branch_id IS NULL OR repository_id IS NOT NULL),
    ADD CONSTRAINT ck_task_run_outbox_v1_host_context
        CHECK (schema_version <> 1 OR (repository_id IS NOT NULL AND branch_id IS NOT NULL)),
    ADD CONSTRAINT fk_task_run_outbox_project_run
        FOREIGN KEY (tenant_id, project_id, engineering_run_id)
        REFERENCES multica.engineering_run
            (tenant_id, project_id, engineering_run_id)
        ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_task_run_outbox_project_run_cursor
    ON multica.task_run_outbox
        (tenant_id, project_id, engineering_run_id, occurred_at, event_id);

CREATE OR REPLACE FUNCTION multica.emit_task_run_outbox()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    previous_owner JSONB;
    actor UUID := NULLIF(current_setting('app.actor_id', true), '')::UUID;
    correlation UUID := NULLIF(current_setting('app.correlation_id', true), '')::UUID;
    task_event_type VARCHAR(32);
BEGIN
    IF NEW.valid_to IS NOT NULL THEN
        RETURN NEW;
    END IF;
    IF NEW.engineering_run_id IS NULL THEN
        RAISE EXCEPTION 'Task Run Outbox requires canonical Project + Run ownership'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_outbox_project_run_required';
    END IF;
    IF actor IS NULL OR correlation IS NULL THEN
        RAISE EXCEPTION 'Task Run Outbox requires app.actor_id and app.correlation_id in the transaction context'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_outbox_context_required';
    END IF;

    SELECT jsonb_build_object(
               'tenant_id', prior.tenant_id,
               'project_id', prior.project_id,
               'engineering_run_id', prior.engineering_run_id,
               'repository_id', prior.repository_id,
               'branch_id', prior.branch_id
           )
      INTO previous_owner
      FROM multica.task_metadata prior
     WHERE prior.tenant_id = NEW.tenant_id
       AND prior.work_item_id = NEW.work_item_id
       AND prior.metadata_id <> NEW.metadata_id
       AND prior.version < NEW.version
     ORDER BY prior.version DESC
     LIMIT 1;

    task_event_type := CASE
        WHEN previous_owner IS NULL THEN 'task.created'
        WHEN (previous_owner ->> 'tenant_id') IS DISTINCT FROM NEW.tenant_id::TEXT
          OR (previous_owner ->> 'project_id') IS DISTINCT FROM NEW.project_id::TEXT
          OR (previous_owner ->> 'engineering_run_id') IS DISTINCT FROM NEW.engineering_run_id::TEXT
            THEN 'task.owner_changed'
        ELSE 'task.updated'
    END;

    INSERT INTO multica.task_run_outbox (
        tenant_id, project_id, work_item_id, task_metadata_id, repository_id, branch_id,
        engineering_run_id, aggregate_version, event_type, schema_version,
        actor_id, correlation_id, payload
    ) VALUES (
        NEW.tenant_id, NEW.project_id, NEW.work_item_id, NEW.metadata_id,
        NEW.repository_id, NEW.branch_id, NEW.engineering_run_id, NEW.version,
        task_event_type, 2, actor, correlation,
        jsonb_build_object(
            'schema_version', 2,
            'work_item_id', NEW.work_item_id,
            'workspace_id', NEW.workspace_id,
            'owner', jsonb_build_object(
                'tenant_id', NEW.tenant_id,
                'project_id', NEW.project_id,
                'engineering_run_id', NEW.engineering_run_id
            ),
            'host_context', jsonb_build_object(
                'repository_id', NEW.repository_id,
                'branch_id', NEW.branch_id
            ),
            'metadata_version', NEW.version,
            'previous_owner', previous_owner
        )
    );
    RETURN NEW;
END;
$$;

COMMENT ON COLUMN multica.task_run_outbox.schema_version IS
    'V1 events retain hosted tuple semantics. New rows use v2 Project + Run ownership with optional repository/Branch context.';
COMMENT ON TABLE multica.task_run_outbox IS
    'T: append-only transactional Task events. V2 owner is (tenant, Project, Engineering Run); host context may be absent.';

COMMIT;
