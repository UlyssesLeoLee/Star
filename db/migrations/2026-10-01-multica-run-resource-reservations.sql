-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-10-01-multica-run-resource-reservations.sql",type:"file",language:"sql"}),
--   (m:Module {name:"multica_run_resource_reservations",type:"module",language:"sql"}),
--   (quota:Class {name:"multica.project_execution_resource_quota",type:"class",language:"sql"}),
--   (admission_lock:Class {name:"multica.project_execution_resource_admission_lock",type:"class",language:"sql"}),
--   (quota_audit:Class {name:"multica.project_execution_resource_quota_audit_event",type:"class",language:"sql"}),
--   (reservation:Class {name:"multica.project_execution_resource_reservation",type:"class",language:"sql"}),
--   (reservation_event:Class {name:"multica.project_execution_resource_reservation_event",type:"class",language:"sql"}),
--   (run:Class {name:"multica.task_execution_run",type:"class",language:"sql"}),
--   (run_event:Class {name:"multica.task_execution_run_event",type:"class",language:"sql"}),
--   (quota_guard:Function {name:"multica.guard_project_execution_resource_quota_scd2",type:"function",language:"sql"}),
--   (quota_successor:Function {name:"multica.require_project_execution_resource_quota_successor",type:"function",language:"sql"}),
--   (quota_audit_scope:Function {name:"multica.validate_project_execution_resource_quota_audit_scope",type:"function",language:"sql"}),
--   (quota_audit_required:Function {name:"multica.require_project_execution_resource_quota_audit",type:"function",language:"sql"}),
--   (quota_audit_mutation:Function {name:"multica.reject_project_execution_resource_quota_audit_mutation",type:"function",language:"sql"}),
--   (reservation_guard:Function {name:"multica.guard_project_execution_resource_reservation",type:"function",language:"sql"}),
--   (reservation_event_scope:Function {name:"multica.validate_project_execution_resource_reservation_event",type:"function",language:"sql"}),
--   (reservation_event_required:Function {name:"multica.require_project_execution_resource_reservation_event",type:"function",language:"sql"}),
--   (reservation_run_event_required:Function {name:"multica.require_project_execution_resource_reservation_run_event",type:"function",language:"sql"}),
--   (reservation_event_mutation:Function {name:"multica.reject_project_execution_resource_reservation_event_mutation",type:"function",language:"sql"}),
--   (quota_rls:Logic {name:"project_execution_resource_quota_tenant_rls",type:"logic",language:"sql"}),
--   (admission_lock_rls:Logic {name:"project_execution_resource_admission_lock_tenant_rls",type:"logic",language:"sql"}),
--   (quota_audit_rls:Logic {name:"project_execution_resource_quota_audit_tenant_rls",type:"logic",language:"sql"}),
--   (reservation_rls:Logic {name:"project_execution_resource_reservation_tenant_rls",type:"logic",language:"sql"}),
--   (reservation_event_rls:Logic {name:"project_execution_resource_reservation_event_tenant_rls",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(quota),(m)-[:CONTAINS]->(admission_lock),
--   (m)-[:CONTAINS]->(quota_audit),
--   (m)-[:CONTAINS]->(reservation),(m)-[:CONTAINS]->(reservation_event),
--   (m)-[:USES]->(run),(m)-[:USES]->(run_event),
--   (m)-[:CONTAINS]->(quota_guard),(m)-[:CONTAINS]->(quota_successor),
--   (m)-[:CONTAINS]->(quota_audit_scope),(m)-[:CONTAINS]->(quota_audit_required),
--   (m)-[:CONTAINS]->(quota_audit_mutation),(m)-[:CONTAINS]->(reservation_guard),
--   (m)-[:CONTAINS]->(reservation_event_scope),(m)-[:CONTAINS]->(reservation_event_required),
--   (m)-[:CONTAINS]->(reservation_run_event_required),
--   (m)-[:CONTAINS]->(reservation_event_mutation),(m)-[:CONTAINS]->(quota_rls),
--   (m)-[:CONTAINS]->(admission_lock_rls),
--   (m)-[:CONTAINS]->(quota_audit_rls),(m)-[:CONTAINS]->(reservation_rls),
--   (m)-[:CONTAINS]->(reservation_event_rls),(quota_guard)-[:GUARDS]->(quota),
--   (quota_successor)-[:USES]->(quota),(quota_audit_scope)-[:USES]->(quota),
--   (quota_audit_required)-[:USES]->(quota),(quota_audit_required)-[:USES]->(quota_audit),
--   (quota_audit_mutation)-[:USES]->(quota_audit),(reservation_guard)-[:GUARDS]->(reservation),
--   (reservation_event_scope)-[:USES]->(reservation),(reservation_event_required)-[:USES]->(reservation),
--   (reservation_event_required)-[:USES]->(reservation_event),(reservation_rls)-[:USES]->(reservation),
--   (reservation_run_event_required)-[:USES]->(reservation_event),
--   (reservation_run_event_required)-[:USES]->(run_event),
--   (reservation_event_rls)-[:USES]->(reservation_event);

-- Phase 9E-4C3. Project-wide resource capacity is serialized across Worktrees.
-- W/T/M: immutable quota revisions are Master; quota and reservation lifecycle events are
-- append-only Transaction; live reservations are Work with a short readiness-fence lease.
-- No quota is seeded: Run admission fails closed until a trusted publisher installs one.
BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

DO $$
BEGIN
    ALTER TABLE multica.task_execution_run_event
        DROP CONSTRAINT IF EXISTS task_execution_run_event_event_type_check;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.task_execution_run_event'::regclass
           AND conname = 'ck_task_execution_run_event_type_v2'
    ) THEN
        ALTER TABLE multica.task_execution_run_event
            ADD CONSTRAINT ck_task_execution_run_event_type_v2 CHECK (event_type IN (
                'run_started','execution_state_changed','agent_declaration','validation_result',
                'human_review','integration_state_changed','human_intervention','cost_recorded',
                'failure_classified','evidence_attached','resource_summary',
                'resource_budget_reserved','resource_reservation_activated','resource_reservation_released',
                'schedule_occurrence_linked','loop_iteration_started','loop_decision_recorded','loop_stopped',
                'hook_evaluated','hook_override_recorded'
            ));
    END IF;
END;
$$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'multica.task_execution_run'::regclass
           AND conname = 'ck_task_execution_run_profile_budgets_complete'
    ) THEN
        ALTER TABLE multica.task_execution_run
            ADD CONSTRAINT ck_task_execution_run_profile_budgets_complete CHECK (
                execution_profile_snapshot IS NULL
                OR (
                    resource_budget_snapshot IS NOT NULL
                    AND loop_policy_snapshot IS NOT NULL
                    AND resource_budget_snapshot = execution_profile_snapshot #> '{profile,resource_budget}'
                    AND loop_policy_snapshot = execution_profile_snapshot #> '{profile,loop_budget}'
                )
            );
    END IF;
END;
$$;

CREATE TABLE IF NOT EXISTS multica.project_execution_resource_quota (
    quota_revision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    quota_version BIGINT NOT NULL CHECK (quota_version > 0),
    lifecycle_state VARCHAR(12) NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active','disabled')),
    max_active_runs INTEGER NOT NULL CHECK (max_active_runs > 0),
    max_rss_bytes BIGINT NOT NULL CHECK (max_rss_bytes > 0),
    max_cpu_ms_per_run BIGINT NOT NULL CHECK (max_cpu_ms_per_run > 0),
    max_runtime_ms_per_run BIGINT NOT NULL CHECK (max_runtime_ms_per_run > 0),
    max_child_processes INTEGER NOT NULL CHECK (max_child_processes > 0),
    max_parallel_tools INTEGER NOT NULL CHECK (max_parallel_tools > 0),
    max_provider_calls INTEGER NOT NULL CHECK (max_provider_calls > 0),
    max_output_bytes BIGINT NOT NULL CHECK (max_output_bytes > 0),
    max_event_buffer_bytes BIGINT NOT NULL CHECK (max_event_buffer_bytes > 0),
    changed_by UUID NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, project_id, quota_version),
    UNIQUE (tenant_id, project_id, quota_version, quota_revision_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (changed_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (valid_to IS NULL OR valid_to > valid_from)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_project_execution_resource_quota_current
    ON multica.project_execution_resource_quota (tenant_id, project_id)
    WHERE valid_to IS NULL;

CREATE TABLE IF NOT EXISTS multica.project_execution_resource_quota_audit_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    quota_version BIGINT NOT NULL CHECK (quota_version > 0),
    event_type VARCHAR(24) NOT NULL CHECK (event_type IN ('quota_configured','quota_disabled')),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, project_id, quota_version),
    FOREIGN KEY (tenant_id, project_id, quota_version)
        REFERENCES multica.project_execution_resource_quota (tenant_id, project_id, quota_version)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_project_execution_resource_quota_audit_scope
    ON multica.project_execution_resource_quota_audit_event
        (tenant_id, project_id, occurred_at DESC, event_id);

-- This small Work row serializes Project-wide admissions under REPEATABLE READ. The 30-day
-- expiry allows a tenant-scoped maintenance worker to prune inactive Project lock rows.
CREATE TABLE IF NOT EXISTS multica.project_execution_resource_admission_lock (
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    allocation_epoch BIGINT NOT NULL DEFAULT 0 CHECK (allocation_epoch >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '30 days'),
    PRIMARY KEY (tenant_id, project_id),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (expires_at > updated_at)
);
CREATE INDEX IF NOT EXISTS idx_project_execution_resource_admission_lock_expiry
    ON multica.project_execution_resource_admission_lock (expires_at);

CREATE OR REPLACE FUNCTION multica.guard_project_execution_resource_quota_scd2()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    latest_version BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Project Execution Resource Quota Master rows cannot be deleted';
    END IF;

    PERFORM pg_advisory_xact_lock(
        hashtextextended(NEW.tenant_id::text || ':' || NEW.project_id::text, 0)
    );

    IF TG_OP = 'UPDATE' THEN
        IF OLD.valid_to IS NOT NULL
           OR NEW.valid_to IS NULL
           OR NEW.valid_to <= OLD.valid_from
           OR (to_jsonb(OLD) - 'valid_to') IS DISTINCT FROM (to_jsonb(NEW) - 'valid_to') THEN
            RAISE EXCEPTION 'Project Execution Resource Quota revisions may only close once without rewriting history';
        END IF;
        RETURN NEW;
    END IF;

    IF TG_OP <> 'INSERT' OR NEW.valid_to IS NOT NULL THEN
        RAISE EXCEPTION 'Project Execution Resource Quota revisions must be inserted as current rows';
    END IF;

    SELECT COALESCE(MAX(quota_version), 0)
      INTO latest_version
      FROM multica.project_execution_resource_quota
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id;
    IF NEW.quota_version <> latest_version + 1 THEN
        RAISE EXCEPTION 'Project Execution Resource Quota revision must increment by one';
    END IF;
    IF EXISTS (
        SELECT 1 FROM multica.project_execution_resource_quota
         WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id AND valid_to IS NULL
    ) THEN
        RAISE EXCEPTION 'Close the current Project Execution Resource Quota before inserting a successor';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_quota_scd2_guard
    ON multica.project_execution_resource_quota;
CREATE TRIGGER project_execution_resource_quota_scd2_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.project_execution_resource_quota
    FOR EACH ROW EXECUTE FUNCTION multica.guard_project_execution_resource_quota_scd2();

CREATE OR REPLACE FUNCTION multica.require_project_execution_resource_quota_successor()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.valid_to IS NULL AND NEW.valid_to IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM multica.project_execution_resource_quota
         WHERE tenant_id = OLD.tenant_id AND project_id = OLD.project_id
           AND quota_version = OLD.quota_version + 1
    ) THEN
        RAISE EXCEPTION 'Closing a Project Execution Resource Quota revision requires a successor in the same transaction';
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_quota_successor_guard
    ON multica.project_execution_resource_quota;
CREATE CONSTRAINT TRIGGER project_execution_resource_quota_successor_guard
    AFTER UPDATE OF valid_to ON multica.project_execution_resource_quota
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION multica.require_project_execution_resource_quota_successor();

CREATE OR REPLACE FUNCTION multica.validate_project_execution_resource_quota_audit_scope()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    revision_state VARCHAR(12);
BEGIN
    SELECT lifecycle_state INTO revision_state
      FROM multica.project_execution_resource_quota
     WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
       AND quota_version = NEW.quota_version;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Project Resource Quota Audit must reference an existing revision';
    END IF;
    IF (NEW.event_type = 'quota_configured' AND revision_state <> 'active')
       OR (NEW.event_type = 'quota_disabled' AND revision_state <> 'disabled') THEN
        RAISE EXCEPTION 'Project Resource Quota Audit event must match revision lifecycle state';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_quota_audit_scope_guard
    ON multica.project_execution_resource_quota_audit_event;
CREATE TRIGGER project_execution_resource_quota_audit_scope_guard
    BEFORE INSERT ON multica.project_execution_resource_quota_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.validate_project_execution_resource_quota_audit_scope();

CREATE OR REPLACE FUNCTION multica.require_project_execution_resource_quota_audit()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM multica.project_execution_resource_quota_audit_event
         WHERE tenant_id = NEW.tenant_id AND project_id = NEW.project_id
           AND quota_version = NEW.quota_version
    ) THEN
        RAISE EXCEPTION 'Every Project Execution Resource Quota revision requires an Audit event in the same transaction';
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_quota_audit_required
    ON multica.project_execution_resource_quota;
CREATE CONSTRAINT TRIGGER project_execution_resource_quota_audit_required
    AFTER INSERT ON multica.project_execution_resource_quota
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION multica.require_project_execution_resource_quota_audit();

CREATE OR REPLACE FUNCTION multica.reject_project_execution_resource_quota_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Project Execution Resource Quota Audit facts are append-only';
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_quota_audit_no_mutation
    ON multica.project_execution_resource_quota_audit_event;
CREATE TRIGGER project_execution_resource_quota_audit_no_mutation
    BEFORE UPDATE OR DELETE ON multica.project_execution_resource_quota_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_project_execution_resource_quota_audit_mutation();
DROP TRIGGER IF EXISTS project_execution_resource_quota_audit_no_truncate
    ON multica.project_execution_resource_quota_audit_event;
CREATE TRIGGER project_execution_resource_quota_audit_no_truncate
    BEFORE TRUNCATE ON multica.project_execution_resource_quota_audit_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_project_execution_resource_quota_audit_mutation();

CREATE TABLE IF NOT EXISTS multica.project_execution_resource_reservation (
    reservation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    run_id UUID NOT NULL,
    quota_version BIGINT NOT NULL,
    quota_revision_id UUID NOT NULL,
    admission_fence_id UUID NOT NULL,
    reserved_by UUID NOT NULL,
    correlation_id UUID NOT NULL,
    reservation_state VARCHAR(12) NOT NULL DEFAULT 'pending'
        CHECK (reservation_state IN ('pending','active','released')),
    lease_expires_at TIMESTAMPTZ NOT NULL,
    activated_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ,
    max_rss_bytes BIGINT NOT NULL CHECK (max_rss_bytes >= 0),
    max_cpu_ms BIGINT NOT NULL CHECK (max_cpu_ms >= 0),
    max_runtime_ms BIGINT NOT NULL CHECK (max_runtime_ms >= 0),
    max_child_processes INTEGER NOT NULL CHECK (max_child_processes >= 0),
    max_parallel_tools INTEGER NOT NULL CHECK (max_parallel_tools >= 0),
    max_provider_calls BIGINT NOT NULL CHECK (max_provider_calls >= 0),
    max_output_bytes BIGINT NOT NULL CHECK (max_output_bytes >= 0),
    max_event_buffer_bytes BIGINT NOT NULL CHECK (max_event_buffer_bytes >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, run_id),
    UNIQUE (tenant_id, reservation_id, run_id),
    FOREIGN KEY (tenant_id, project_id, work_item_id, run_id)
        REFERENCES multica.task_execution_run (tenant_id, project_id, work_item_id, run_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, project_id, quota_version, quota_revision_id)
        REFERENCES multica.project_execution_resource_quota
            (tenant_id, project_id, quota_version, quota_revision_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (worktree_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (admission_fence_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (reserved_by <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (lease_expires_at > created_at),
    CHECK (
        (reservation_state = 'pending' AND activated_at IS NULL AND released_at IS NULL)
        OR (reservation_state = 'active' AND activated_at IS NOT NULL AND released_at IS NULL)
        OR (reservation_state = 'released' AND released_at IS NOT NULL)
    )
);
CREATE INDEX IF NOT EXISTS idx_project_execution_resource_reservation_held
    ON multica.project_execution_resource_reservation (tenant_id, project_id, lease_expires_at)
    WHERE reservation_state IN ('pending','active');

CREATE TABLE IF NOT EXISTS multica.project_execution_resource_reservation_event (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    reservation_id UUID NOT NULL,
    run_id UUID NOT NULL,
    event_type VARCHAR(16) NOT NULL CHECK (event_type IN ('reserved','activated','released')),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object')
        CHECK (octet_length(details::text) <= 4096),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, reservation_id, event_type),
    FOREIGN KEY (tenant_id, reservation_id, run_id)
        REFERENCES multica.project_execution_resource_reservation (tenant_id, reservation_id, run_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_project_execution_resource_reservation_event_scope
    ON multica.project_execution_resource_reservation_event
        (tenant_id, project_id, reservation_id, occurred_at, event_id);

CREATE OR REPLACE FUNCTION multica.guard_project_execution_resource_reservation()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    run_resource_budget JSONB;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Project Execution Resource Reservation rows cannot be deleted';
    END IF;
    IF TG_OP = 'INSERT' THEN
        SELECT r.resource_budget_snapshot
          INTO run_resource_budget
          FROM multica.task_execution_run r
         WHERE r.tenant_id = NEW.tenant_id AND r.project_id = NEW.project_id
           AND r.work_item_id = NEW.work_item_id AND r.run_id = NEW.run_id;
        IF NOT FOUND OR run_resource_budget IS DISTINCT FROM jsonb_build_object(
            'max_rss_bytes', NEW.max_rss_bytes,
            'max_cpu_ms', NEW.max_cpu_ms,
            'max_runtime_ms', NEW.max_runtime_ms,
            'max_child_processes', NEW.max_child_processes,
            'max_parallel_tools', NEW.max_parallel_tools,
            'max_provider_calls', NEW.max_provider_calls,
            'max_output_bytes', NEW.max_output_bytes,
            'max_event_buffer_bytes', NEW.max_event_buffer_bytes
        ) THEN
            RAISE EXCEPTION 'Project Execution Resource Reservation must exactly match the immutable Run ResourceBudget snapshot';
        END IF;
        IF NEW.reservation_state <> 'pending'
           OR NOT EXISTS (
               SELECT 1 FROM multica.project_execution_resource_quota q
                WHERE q.tenant_id = NEW.tenant_id AND q.project_id = NEW.project_id
                  AND q.quota_version = NEW.quota_version
                  AND q.quota_revision_id = NEW.quota_revision_id
                  AND q.valid_to IS NULL AND q.lifecycle_state = 'active'
           ) THEN
            RAISE EXCEPTION 'New Project Execution Resource Reservations require a current active quota and pending state';
        END IF;
        RETURN NEW;
    END IF;

    IF NOT (
        (OLD.reservation_state = 'pending' AND NEW.reservation_state IN ('active','released'))
        OR (OLD.reservation_state = 'active' AND NEW.reservation_state = 'released')
    ) OR (to_jsonb(OLD) - 'reservation_state' - 'activated_at' - 'released_at')
            IS DISTINCT FROM
       (to_jsonb(NEW) - 'reservation_state' - 'activated_at' - 'released_at') THEN
        RAISE EXCEPTION 'Project Execution Resource Reservation may only transition pending to active/released or active to released';
    END IF;
    IF OLD.reservation_state = 'pending' AND NEW.reservation_state = 'active'
       AND (OLD.lease_expires_at <= clock_timestamp()
            OR NEW.activated_at IS NULL OR NEW.activated_at < NEW.created_at
            OR NEW.released_at IS NOT NULL) THEN
        RAISE EXCEPTION 'Expired reservations cannot activate';
    END IF;
    IF NEW.reservation_state = 'released'
       AND (NEW.released_at IS NULL OR NEW.released_at < NEW.created_at) THEN
        RAISE EXCEPTION 'Released reservations require a valid release timestamp';
    END IF;
    IF NEW.reservation_state = 'active'
       AND (NEW.activated_at IS NULL OR NEW.released_at IS NOT NULL) THEN
        RAISE EXCEPTION 'Active reservations require an activation timestamp';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_reservation_guard
    ON multica.project_execution_resource_reservation;
CREATE TRIGGER project_execution_resource_reservation_guard
    BEFORE INSERT OR UPDATE OR DELETE ON multica.project_execution_resource_reservation
    FOR EACH ROW EXECUTE FUNCTION multica.guard_project_execution_resource_reservation();

CREATE OR REPLACE FUNCTION multica.validate_project_execution_resource_reservation_event()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    reservation_project_id UUID;
    current_reservation_state VARCHAR(12);
    reservation_maxima JSONB;
BEGIN
    SELECT r.project_id, r.reservation_state,
           jsonb_build_object(
               'max_rss_bytes', r.max_rss_bytes,
               'max_cpu_ms', r.max_cpu_ms,
               'max_runtime_ms', r.max_runtime_ms,
               'max_child_processes', r.max_child_processes,
               'max_parallel_tools', r.max_parallel_tools,
               'max_provider_calls', r.max_provider_calls,
               'max_output_bytes', r.max_output_bytes,
               'max_event_buffer_bytes', r.max_event_buffer_bytes
           )
      INTO reservation_project_id, current_reservation_state, reservation_maxima
      FROM multica.project_execution_resource_reservation r
     WHERE r.tenant_id = NEW.tenant_id AND r.reservation_id = NEW.reservation_id
       AND r.run_id = NEW.run_id;
    IF NOT FOUND OR reservation_project_id IS DISTINCT FROM NEW.project_id THEN
        RAISE EXCEPTION 'Project Execution Resource Reservation event scope must match its reservation';
    END IF;
    IF (NEW.event_type = 'reserved' AND current_reservation_state <> 'pending')
       OR (NEW.event_type = 'activated' AND current_reservation_state <> 'active')
       OR (NEW.event_type = 'released' AND current_reservation_state <> 'released') THEN
        RAISE EXCEPTION 'Project Execution Resource Reservation event must match current state';
    END IF;
    IF NEW.event_type = 'reserved'
       AND NEW.details -> 'reserved_maxima' IS DISTINCT FROM reservation_maxima THEN
        RAISE EXCEPTION 'Reservation event maxima must match the reserved Run budget';
    END IF;
    RETURN NEW;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_reservation_event_scope_guard
    ON multica.project_execution_resource_reservation_event;
CREATE TRIGGER project_execution_resource_reservation_event_scope_guard
    BEFORE INSERT ON multica.project_execution_resource_reservation_event
    FOR EACH ROW EXECUTE FUNCTION multica.validate_project_execution_resource_reservation_event();

CREATE OR REPLACE FUNCTION multica.require_project_execution_resource_reservation_event()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    expected_event_type VARCHAR(16);
BEGIN
    IF TG_OP = 'INSERT' THEN
        expected_event_type := 'reserved';
    ELSIF NEW.reservation_state = 'active' THEN
        expected_event_type := 'activated';
    ELSE
        expected_event_type := 'released';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM multica.project_execution_resource_reservation_event
         WHERE tenant_id = NEW.tenant_id AND reservation_id = NEW.reservation_id
           AND run_id = NEW.run_id AND event_type = expected_event_type
    ) THEN
        RAISE EXCEPTION 'Every Project Execution Resource Reservation transition requires a matching append-only event';
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_reservation_event_required
    ON multica.project_execution_resource_reservation;
CREATE CONSTRAINT TRIGGER project_execution_resource_reservation_event_required
    AFTER INSERT OR UPDATE OF reservation_state ON multica.project_execution_resource_reservation
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION multica.require_project_execution_resource_reservation_event();

CREATE OR REPLACE FUNCTION multica.require_project_execution_resource_reservation_run_event()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    expected_run_event_type VARCHAR(40);
BEGIN
    expected_run_event_type := CASE NEW.event_type
        WHEN 'reserved' THEN 'resource_budget_reserved'
        WHEN 'activated' THEN 'resource_reservation_activated'
        ELSE 'resource_reservation_released'
    END;
    IF NOT EXISTS (
        SELECT 1
          FROM multica.task_execution_run_event e
          JOIN multica.project_execution_resource_reservation r
            ON r.tenant_id = e.tenant_id AND r.project_id = e.project_id
           AND r.run_id = e.run_id AND r.work_item_id = e.work_item_id
         WHERE e.event_id = NEW.event_id
           AND e.tenant_id = NEW.tenant_id AND e.project_id = NEW.project_id
           AND e.run_id = NEW.run_id AND e.event_type = expected_run_event_type
           AND e.actor_id = NEW.actor_id AND e.correlation_id = NEW.correlation_id
           AND e.details = NEW.details
           AND r.reservation_id = NEW.reservation_id
    ) THEN
        RAISE EXCEPTION 'Every resource reservation event requires its paired RunEvent with the same event ID and facts';
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_reservation_run_event_required
    ON multica.project_execution_resource_reservation_event;
CREATE CONSTRAINT TRIGGER project_execution_resource_reservation_run_event_required
    AFTER INSERT ON multica.project_execution_resource_reservation_event
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION multica.require_project_execution_resource_reservation_run_event();

CREATE OR REPLACE FUNCTION multica.reject_project_execution_resource_reservation_event_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Project Execution Resource Reservation events are append-only';
END;
$$;
DROP TRIGGER IF EXISTS project_execution_resource_reservation_event_no_mutation
    ON multica.project_execution_resource_reservation_event;
CREATE TRIGGER project_execution_resource_reservation_event_no_mutation
    BEFORE UPDATE OR DELETE ON multica.project_execution_resource_reservation_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_project_execution_resource_reservation_event_mutation();
DROP TRIGGER IF EXISTS project_execution_resource_reservation_event_no_truncate
    ON multica.project_execution_resource_reservation_event;
CREATE TRIGGER project_execution_resource_reservation_event_no_truncate
    BEFORE TRUNCATE ON multica.project_execution_resource_reservation_event
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_project_execution_resource_reservation_event_mutation();

CREATE UNIQUE INDEX IF NOT EXISTS uq_task_execution_run_resource_budget_reserved
    ON multica.task_execution_run_event (tenant_id, run_id)
    WHERE event_type = 'resource_budget_reserved';

ALTER TABLE multica.project_execution_resource_admission_lock ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.project_execution_resource_admission_lock FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS project_execution_resource_admission_lock_tenant_scope
    ON multica.project_execution_resource_admission_lock;
CREATE POLICY project_execution_resource_admission_lock_tenant_scope
    ON multica.project_execution_resource_admission_lock
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.project_execution_resource_quota ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.project_execution_resource_quota FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS project_execution_resource_quota_tenant_scope
    ON multica.project_execution_resource_quota;
CREATE POLICY project_execution_resource_quota_tenant_scope
    ON multica.project_execution_resource_quota
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.project_execution_resource_quota_audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.project_execution_resource_quota_audit_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS project_execution_resource_quota_audit_tenant_scope
    ON multica.project_execution_resource_quota_audit_event;
CREATE POLICY project_execution_resource_quota_audit_tenant_scope
    ON multica.project_execution_resource_quota_audit_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.project_execution_resource_reservation ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.project_execution_resource_reservation FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS project_execution_resource_reservation_tenant_scope
    ON multica.project_execution_resource_reservation;
CREATE POLICY project_execution_resource_reservation_tenant_scope
    ON multica.project_execution_resource_reservation
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.project_execution_resource_reservation_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.project_execution_resource_reservation_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS project_execution_resource_reservation_event_tenant_scope
    ON multica.project_execution_resource_reservation_event;
CREATE POLICY project_execution_resource_reservation_event_tenant_scope
    ON multica.project_execution_resource_reservation_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.project_execution_resource_quota IS
    'M: Project-wide resource ceilings shared across all Worktrees; quota changes create audited SCD2 revisions. No default quota is seeded.';
COMMENT ON TABLE multica.project_execution_resource_admission_lock IS
    'W: one small Project-scoped allocation epoch row serializes REPEATABLE READ admissions; 30-day expiry is pruned by tenant-scoped maintenance.';
COMMENT ON TABLE multica.project_execution_resource_quota_audit_event IS
    'T: append-only author and correlation audit for Project quota revisions.';
COMMENT ON TABLE multica.project_execution_resource_reservation IS
    'W: bounded Project-wide pending/active Run capacity; pending leases expire with the Runtime admission fence and retention follows the parent Run.';
COMMENT ON TABLE multica.project_execution_resource_reservation_event IS
    'T: append-only reservation, activation, and release ledger; its reserved maxima are not observed resource measurements.';
COMMENT ON CONSTRAINT ck_task_execution_run_profile_budgets_complete
    ON multica.task_execution_run IS
    'Profile-bound Runs must preserve exact ResourceBudget and Engineering Loop snapshots from the immutable self-contained Profile document.';

COMMIT;
