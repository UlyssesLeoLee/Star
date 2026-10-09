-- Run-owner processed admission Inbox + immutable result Outbox (Transaction).
-- No pending queue, Schedule table write, execution authority or delivery worker.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-run-owner-schedule-admission.sql"})
-- MERGE (inbox:Table {id:"multica.schedule_admission_inbox"})
-- MERGE (outbox:Table {id:"multica.schedule_admission_result_outbox"})
-- MERGE (run:Table {id:"multica.task_execution_run"})
-- MERGE (inbox_guard:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-admission.sql::guard_schedule_admission_inbox",kind:"function"})
-- MERGE (result_guard:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-admission.sql::guard_schedule_admission_result",kind:"function"})
-- MERGE (worm:Symbol {id:"db/migrations/2026-10-10-run-owner-schedule-admission.sql::reject_schedule_admission_mutation",kind:"function"})
-- MERGE (self)-[:DEFINES]->(inbox_guard)
-- MERGE (self)-[:DEFINES]->(result_guard)
-- MERGE (self)-[:DEFINES]->(worm)
-- MERGE (result_guard)-[:READS]->(inbox)
-- MERGE (result_guard)-[:READS]->(run)
-- MERGE (result_guard)-[:READS]->(run_event:Table {id:"multica.task_execution_run_event"})
-- MERGE (self)-[:DEFINES]->(inbox)
-- MERGE (self)-[:DEFINES]->(outbox)
-- MERGE (outbox)-[:REFERENCES]->(inbox)
-- MERGE (outbox)-[:REFERENCES]->(run)
-- @endcypher
BEGIN;
CREATE TABLE IF NOT EXISTS multica.schedule_admission_inbox (
    tenant_id UUID NOT NULL,
    request_id UUID NOT NULL,
    result_id UUID NOT NULL,
    project_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    admission_attempt BIGINT NOT NULL CHECK (admission_attempt BETWEEN 1 AND 4294967295),
    correlation_id UUID NOT NULL,
    run_as_actor_id UUID NOT NULL,
    source_schedule_service_id UUID NOT NULL
      CHECK (source_schedule_service_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    rule_id UUID NOT NULL,
    rule_version BIGINT NOT NULL CHECK (rule_version > 0),
    command_wire BYTEA NOT NULL CHECK (octet_length(command_wire) BETWEEN 1 AND 16384),
    command_sha256 BYTEA NOT NULL CHECK (octet_length(command_sha256) = 32),
    processed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, request_id),
    UNIQUE (tenant_id, request_id, result_id),
    UNIQUE (tenant_id, occurrence_id, admission_attempt),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (request_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (result_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (run_as_actor_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE TABLE IF NOT EXISTS multica.schedule_admission_result_outbox (
    tenant_id UUID NOT NULL,
    request_id UUID NOT NULL,
    result_id UUID NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('accepted','rejected')),
    task_execution_run_id UUID,
    result_wire BYTEA NOT NULL CHECK (octet_length(result_wire) BETWEEN 1 AND 16384),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, request_id),
    UNIQUE (result_id),
    UNIQUE (tenant_id, request_id, result_id),
    CHECK ((outcome = 'accepted') = (task_execution_run_id IS NOT NULL)),
    FOREIGN KEY (tenant_id, request_id, result_id)
      REFERENCES multica.schedule_admission_inbox (tenant_id, request_id, result_id)
      ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, task_execution_run_id)
      REFERENCES multica.task_execution_run (tenant_id, run_id) ON DELETE RESTRICT
);
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint
       WHERE conrelid = 'multica.schedule_admission_inbox'::regclass
         AND conname = 'schedule_admission_paired_result') THEN
        ALTER TABLE multica.schedule_admission_inbox
          ADD CONSTRAINT schedule_admission_paired_result
          FOREIGN KEY (tenant_id, request_id, result_id)
          REFERENCES multica.schedule_admission_result_outbox (tenant_id, request_id, result_id)
          DEFERRABLE INITIALLY DEFERRED;
    END IF;
END $$;

CREATE OR REPLACE FUNCTION multica.guard_schedule_admission_inbox()
RETURNS TRIGGER LANGUAGE plpgsql SET search_path = pg_catalog AS $$
DECLARE wire JSONB := convert_from(NEW.command_wire, 'UTF8')::JSONB;
        target JSONB := wire #> '{occurrence,rule_snapshot,target}';
BEGIN
    IF (wire->>'schema_version')::INTEGER IS DISTINCT FROM 1
       OR (wire->>'tenant_id')::UUID IS DISTINCT FROM NEW.tenant_id
       OR (wire->>'request_id')::UUID IS DISTINCT FROM NEW.request_id
       OR (wire->>'correlation_id')::UUID IS DISTINCT FROM NEW.correlation_id
       OR (wire->>'admission_attempt')::BIGINT IS DISTINCT FROM NEW.admission_attempt
       OR (wire#>>'{occurrence,occurrence_id}')::UUID IS DISTINCT FROM NEW.occurrence_id
       OR (wire#>>'{occurrence,run_as_actor_id}')::UUID IS DISTINCT FROM NEW.run_as_actor_id
       OR (wire#>>'{occurrence,key,rule_id}')::UUID IS DISTINCT FROM NEW.rule_id
       OR (wire#>>'{occurrence,key,rule_version}')::BIGINT IS DISTINCT FROM NEW.rule_version
       OR (target->>'project_id')::UUID IS DISTINCT FROM NEW.project_id
       OR (target->>'branch_id')::UUID IS DISTINCT FROM NEW.branch_id
       OR (target->>'engineering_run_id')::UUID IS DISTINCT FROM NEW.engineering_run_id
       OR (target->>'work_item_id')::UUID IS DISTINCT FROM NEW.work_item_id
       OR (target->>'worktree_id')::UUID IS DISTINCT FROM NEW.worktree_id THEN
        RAISE EXCEPTION 'admission Inbox projection must match immutable command wire';
    END IF;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS schedule_admission_inbox_scope ON multica.schedule_admission_inbox;
CREATE TRIGGER schedule_admission_inbox_scope BEFORE INSERT ON multica.schedule_admission_inbox
 FOR EACH ROW EXECUTE FUNCTION multica.guard_schedule_admission_inbox();

CREATE OR REPLACE FUNCTION multica.guard_schedule_admission_result()
RETURNS TRIGGER LANGUAGE plpgsql SET search_path = pg_catalog AS $$
DECLARE receipt multica.schedule_admission_inbox%ROWTYPE;
        wire JSONB := convert_from(NEW.result_wire, 'UTF8')::JSONB;
BEGIN
    SELECT * INTO STRICT receipt FROM multica.schedule_admission_inbox
      WHERE tenant_id = NEW.tenant_id AND request_id = NEW.request_id;
    IF (wire->>'schema_version')::INTEGER IS DISTINCT FROM 1
       OR (wire->>'tenant_id')::UUID IS DISTINCT FROM NEW.tenant_id
       OR (wire->>'request_id')::UUID IS DISTINCT FROM NEW.request_id
       OR (wire->>'result_id')::UUID IS DISTINCT FROM NEW.result_id
       OR (wire->>'correlation_id')::UUID IS DISTINCT FROM receipt.correlation_id
       OR (wire->>'occurrence_id')::UUID IS DISTINCT FROM receipt.occurrence_id
       OR (wire->>'admission_attempt')::BIGINT IS DISTINCT FROM receipt.admission_attempt
       OR wire->>'outcome' IS DISTINCT FROM NEW.outcome
       OR (wire->>'task_execution_run_id')::UUID IS DISTINCT FROM NEW.task_execution_run_id THEN
        RAISE EXCEPTION 'admission Outbox projection must match immutable result wire';
    END IF;
    IF NEW.outcome = 'accepted' AND NOT EXISTS (
        SELECT 1 FROM multica.task_execution_run r
          WHERE r.tenant_id = receipt.tenant_id AND r.project_id = receipt.project_id
            AND r.work_item_id = receipt.work_item_id AND r.run_id = NEW.task_execution_run_id
            AND r.worktree_id = receipt.worktree_id AND r.initiated_by = receipt.run_as_actor_id
            AND r.run_origin = 'schedule' AND r.started_at IS NULL
            AND r.automation_rule_id = receipt.rule_id
            AND r.automation_rule_version = receipt.rule_version
            AND r.automation_occurrence_id = receipt.occurrence_id
            AND r.automation_admission_attempt = receipt.admission_attempt
            AND r.correlation_id = receipt.correlation_id
            AND NOT EXISTS (SELECT 1 FROM multica.task_execution_run_event e
              WHERE e.tenant_id = r.tenant_id AND e.run_id = r.run_id
                AND (e.event_type = 'run_started' OR e.execution_state IS NOT NULL))
    ) THEN RAISE EXCEPTION 'accepted admission requires a matching non-executing Run'; END IF;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS schedule_admission_result_scope ON multica.schedule_admission_result_outbox;
CREATE TRIGGER schedule_admission_result_scope BEFORE INSERT ON multica.schedule_admission_result_outbox
  FOR EACH ROW EXECUTE FUNCTION multica.guard_schedule_admission_result();

CREATE OR REPLACE FUNCTION multica.reject_schedule_admission_mutation()
RETURNS TRIGGER LANGUAGE plpgsql SET search_path = pg_catalog AS $$
BEGIN RAISE EXCEPTION 'Schedule admission facts are append-only'; END $$;
DO $$ DECLARE table_name TEXT;
BEGIN
    FOREACH table_name IN ARRAY ARRAY['schedule_admission_inbox','schedule_admission_result_outbox'] LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS schedule_admission_worm ON multica.%I', table_name);
        EXECUTE format('CREATE TRIGGER schedule_admission_worm BEFORE UPDATE OR DELETE OR TRUNCATE ON multica.%I FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_schedule_admission_mutation()', table_name);
        EXECUTE format('ALTER TABLE multica.%I ENABLE ROW LEVEL SECURITY', table_name);
        EXECUTE format('ALTER TABLE multica.%I FORCE ROW LEVEL SECURITY', table_name);
        EXECUTE format('DROP POLICY IF EXISTS schedule_admission_tenant ON multica.%I', table_name);
        EXECUTE format('CREATE POLICY schedule_admission_tenant ON multica.%I USING (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting(''app.tenant_id'', true), '''')::UUID)', table_name);
        EXECUTE format('REVOKE ALL ON multica.%I FROM PUBLIC', table_name);
        EXECUTE format('GRANT SELECT, INSERT ON multica.%I TO star_multica_run_owner', table_name);
    END LOOP;
END $$;
GRANT SELECT (initiated_by, worktree_id, started_at, automation_admission_attempt)
 ON multica.task_execution_run TO star_multica_run_owner;
GRANT SELECT (tenant_id, run_id, event_type, execution_state)
 ON multica.task_execution_run_event TO star_multica_run_owner;
REVOKE ALL ON FUNCTION multica.guard_schedule_admission_result() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.guard_schedule_admission_inbox() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.reject_schedule_admission_mutation() FROM PUBLIC;
COMMENT ON TABLE multica.schedule_admission_inbox IS
 'Transaction: processed owner-local command receipt; paired result required at commit; no TTL or physical deletion.';
COMMENT ON TABLE multica.schedule_admission_result_outbox IS
 'Transaction: immutable admission result; acceptance is non-executing and requires separate authorization/quota/confirmation/readiness gates.';
COMMIT;
