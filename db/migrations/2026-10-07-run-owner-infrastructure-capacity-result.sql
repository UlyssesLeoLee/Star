-- @cypher schema=1 source_sha256=fbd75a8b0764acbff83f7147d29abe65c3b8fa8ad98aa09039abbd36ea37bd10
-- MERGE (self:File {path:"db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql"})
-- MERGE (role:DatabaseRole {id:"star_multica_run_owner"})
-- MERGE (demand:Table {id:"multica.infrastructure_capacity_demand_outbox"})
-- MERGE (result:Table {id:"multica.infrastructure_capacity_result_inbox"})
-- MERGE (run:Table {id:"multica.task_execution_run"})
-- MERGE (worm:Symbol {id:"multica.reject_run_capacity_fact_mutation",kind:"function"})
-- MERGE (self)-[:CONFIGURES]->(role)
-- MERGE (self)-[:CONFIGURES]->(demand)
-- MERGE (self)-[:CONFIGURES]->(result)
-- MERGE (demand)-[:REFERENCES]->(run)
-- MERGE (demand)-[:CALLS]->(worm)
-- MERGE (result)-[:REFERENCES]->(demand)
-- MERGE (result)-[:CALLS]->(worm)
-- @endcypher

-- Run/Execution owner Transaction facts only. Delivery acknowledgements are owned by the
-- JetStream durable consumer; no mutable projection or cross-owner write is introduced.
-- The TaskExecutionRun FK proves execution-attempt existence. Rust owner validation checks
-- canonical Engineering Run, Branch, and occurrence consistency; no schedule writer is added.
-- W/T/M: no Work rows; both new tables are append-only Transaction facts.
BEGIN;

DO $precondition$
BEGIN
    IF to_regclass('multica.task_execution_run') IS NULL THEN
        RAISE EXCEPTION 'task_execution_run prerequisite migration must be applied first';
    END IF;
END
$precondition$;

CREATE TABLE IF NOT EXISTS multica.infrastructure_capacity_demand_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    task_execution_run_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    schema_version SMALLINT NOT NULL DEFAULT 2 CHECK (schema_version = 2),
    idempotency_key VARCHAR(200) NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload) = 'object')
        CHECK (octet_length(payload::text) <= 16384),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    FOREIGN KEY (tenant_id, project_id, work_item_id, task_execution_run_id)
        REFERENCES multica.task_execution_run (tenant_id, project_id, work_item_id, run_id)
        ON DELETE RESTRICT,
    UNIQUE (tenant_id, event_id),
    UNIQUE (tenant_id, idempotency_key),
    UNIQUE (
        tenant_id, event_id, project_id, work_item_id, engineering_run_id,
        branch_id, task_execution_run_id, occurrence_id
    ),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (work_item_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (branch_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (task_execution_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (occurrence_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (length(idempotency_key) BETWEEN 1 AND 200)
);
CREATE INDEX IF NOT EXISTS idx_run_capacity_demand_outbox_scope
    ON multica.infrastructure_capacity_demand_outbox
       (tenant_id, engineering_run_id, occurred_at, event_id);

CREATE TABLE IF NOT EXISTS multica.infrastructure_capacity_result_inbox (
    tenant_id UUID NOT NULL,
    event_id UUID NOT NULL,
    source_event_id UUID NOT NULL,
    project_id UUID NOT NULL,
    work_item_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    task_execution_run_id UUID NOT NULL,
    occurrence_id UUID NOT NULL,
    event_type VARCHAR(32) NOT NULL CHECK (event_type IN (
        'capacity_accepted','capacity_rejected','capacity_released','capacity_expired'
    )),
    schema_version SMALLINT NOT NULL DEFAULT 2 CHECK (schema_version = 2),
    correlation_id UUID NOT NULL,
    idempotency_key VARCHAR(200) NOT NULL,
    payload_digest BYTEA NOT NULL CHECK (octet_length(payload_digest) = 32),
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload) = 'object')
        CHECK (octet_length(payload::text) <= 16384),
    received_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    CONSTRAINT uq_run_capacity_result_inbox_event_id UNIQUE (event_id),
    FOREIGN KEY (
        tenant_id, source_event_id, project_id, work_item_id, engineering_run_id,
        branch_id, task_execution_run_id, occurrence_id
    ) REFERENCES multica.infrastructure_capacity_demand_outbox (
        tenant_id, event_id, project_id, work_item_id, engineering_run_id,
        branch_id, task_execution_run_id, occurrence_id
    ) ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (event_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (source_event_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (branch_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (length(idempotency_key) BETWEEN 1 AND 200)
);
CREATE INDEX IF NOT EXISTS idx_run_capacity_result_inbox_source
    ON multica.infrastructure_capacity_result_inbox
       (tenant_id, source_event_id, received_at, event_id);

CREATE OR REPLACE FUNCTION multica.reject_run_capacity_fact_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Run capacity Transaction facts are append-only'
        USING ERRCODE = '55000';
END;
$$;

DROP TRIGGER IF EXISTS run_capacity_demand_outbox_worm
    ON multica.infrastructure_capacity_demand_outbox;
CREATE TRIGGER run_capacity_demand_outbox_worm
    BEFORE UPDATE OR DELETE ON multica.infrastructure_capacity_demand_outbox
    FOR EACH ROW EXECUTE FUNCTION multica.reject_run_capacity_fact_mutation();
DROP TRIGGER IF EXISTS run_capacity_demand_outbox_truncate_guard
    ON multica.infrastructure_capacity_demand_outbox;
CREATE TRIGGER run_capacity_demand_outbox_truncate_guard
    BEFORE TRUNCATE ON multica.infrastructure_capacity_demand_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_run_capacity_fact_mutation();

DROP TRIGGER IF EXISTS run_capacity_result_inbox_worm
    ON multica.infrastructure_capacity_result_inbox;
CREATE TRIGGER run_capacity_result_inbox_worm
    BEFORE UPDATE OR DELETE ON multica.infrastructure_capacity_result_inbox
    FOR EACH ROW EXECUTE FUNCTION multica.reject_run_capacity_fact_mutation();
DROP TRIGGER IF EXISTS run_capacity_result_inbox_truncate_guard
    ON multica.infrastructure_capacity_result_inbox;
CREATE TRIGGER run_capacity_result_inbox_truncate_guard
    BEFORE TRUNCATE ON multica.infrastructure_capacity_result_inbox
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_run_capacity_fact_mutation();

ALTER TABLE multica.infrastructure_capacity_demand_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.infrastructure_capacity_demand_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS infrastructure_capacity_demand_outbox_tenant_scope
    ON multica.infrastructure_capacity_demand_outbox;
CREATE POLICY infrastructure_capacity_demand_outbox_tenant_scope
    ON multica.infrastructure_capacity_demand_outbox
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.infrastructure_capacity_result_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.infrastructure_capacity_result_inbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS infrastructure_capacity_result_inbox_tenant_scope
    ON multica.infrastructure_capacity_result_inbox;
CREATE POLICY infrastructure_capacity_result_inbox_tenant_scope
    ON multica.infrastructure_capacity_result_inbox
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

DO $role$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_multica_run_owner') THEN
        EXECUTE 'ALTER ROLE star_multica_run_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    ELSE
        EXECUTE 'CREATE ROLE star_multica_run_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT';
    END IF;
END
$role$;

REVOKE ALL ON SCHEMA multica FROM star_multica_run_owner;
REVOKE ALL PRIVILEGES ON TABLE
    multica.infrastructure_capacity_demand_outbox,
    multica.infrastructure_capacity_result_inbox,
    multica.task_execution_run
FROM star_multica_run_owner;
REVOKE ALL PRIVILEGES ON TABLE
    multica.infrastructure_capacity_demand_outbox,
    multica.infrastructure_capacity_result_inbox
FROM PUBLIC;

DO $other_owners$
DECLARE
    role_name TEXT;
BEGIN
    FOREACH role_name IN ARRAY ARRAY['star_infrastructure_owner', 'star_automation_owner'] LOOP
        IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = role_name) THEN
            EXECUTE format(
                'REVOKE ALL PRIVILEGES ON TABLE multica.infrastructure_capacity_demand_outbox, '
                'multica.infrastructure_capacity_result_inbox FROM %I',
                role_name
            );
        END IF;
    END LOOP;
END
$other_owners$;

GRANT USAGE ON SCHEMA multica TO star_multica_run_owner;
GRANT SELECT, INSERT ON TABLE
    multica.infrastructure_capacity_demand_outbox,
    multica.infrastructure_capacity_result_inbox
TO star_multica_run_owner;
GRANT SELECT (
    tenant_id, project_id, work_item_id, run_id, run_origin,
    automation_rule_id, automation_rule_version, automation_occurrence_id, correlation_id
) ON TABLE multica.task_execution_run TO star_multica_run_owner;

REVOKE ALL ON FUNCTION multica.reject_run_capacity_fact_mutation() FROM PUBLIC;

COMMENT ON TABLE multica.infrastructure_capacity_demand_outbox IS
    'Run/Execution owner Transaction fact for a versioned infrastructure demand; it does not confirm Host or cluster capacity reservation.';
COMMENT ON TABLE multica.infrastructure_capacity_result_inbox IS
    'Run/Execution owner append-only Transaction Inbox and durable event_id dedupe authority for Infrastructure capacity results.';
COMMENT ON COLUMN multica.infrastructure_capacity_demand_outbox.engineering_run_id IS
    'Canonical collaboration Engineering Run identity; distinct from task_execution_run_id.';
COMMENT ON COLUMN multica.infrastructure_capacity_demand_outbox.branch_id IS
    'Canonical cloud Branch identity captured with the infrastructure demand.';
COMMENT ON COLUMN multica.infrastructure_capacity_demand_outbox.task_execution_run_id IS
    'TaskExecutionRun attempt identity; references multica.task_execution_run.run_id.';
COMMENT ON COLUMN multica.infrastructure_capacity_demand_outbox.occurrence_id IS
    'Schedule occurrence identity; Rust owner validation checks it against the canonical admission scope.';
COMMENT ON CONSTRAINT uq_run_capacity_result_inbox_event_id
    ON multica.infrastructure_capacity_result_inbox IS
    'Global Infrastructure event identity prevents cross-tenant event_id reuse; tenant visibility remains controlled by FORCE RLS.';
COMMENT ON COLUMN multica.infrastructure_capacity_result_inbox.payload_digest IS
    'SHA-256 digest of the validated canonical result payload; conflicting event_id replay handling is performed by the Rust owner store.';

COMMIT;
