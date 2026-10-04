-- @cypher schema=1 source_sha256=86de29f35663351de8fa68f69c633ece27122913644e6c0d2f37157847dc4689
-- MERGE (self:File {path:"db/migrations/2026-10-03-automation-schedule-rule-api.sql"})
-- MERGE (rules:Table {id:"automation.schedule_rule_revision"})
-- MERGE (outbox:Table {id:"automation.schedule_rule_outbox"})
-- MERGE (idempotency:Table {id:"automation.schedule_rule_command_idempotency"})
-- MERGE (self)-[:ALTERS]->(rules)
-- MERGE (self)-[:DEFINES]->(outbox)
-- MERGE (self)-[:DEFINES]->(idempotency)
-- @endcypher

-- Phase 9F4A: authenticated Schedule Rule API persistence, transactional outbox, and replay fence.
-- The migration does not enable a clock/dispatcher or admit TaskExecutionRuns.

BEGIN;

ALTER TABLE automation.schedule_rule_revision
    ADD COLUMN IF NOT EXISTS display_name VARCHAR(120) NOT NULL DEFAULT 'Schedule rule';
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname='schedule_rule_display_name_valid'
          AND conrelid='automation.schedule_rule_revision'::regclass
    ) THEN
        ALTER TABLE automation.schedule_rule_revision
            ADD CONSTRAINT schedule_rule_display_name_valid
            CHECK (length(trim(display_name)) BETWEEN 1 AND 120
                   AND display_name !~ '[[:cntrl:]]');
    END IF;
END $$;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname='schedule_rule_revision_run_identity_unique'
          AND conrelid='automation.schedule_rule_revision'::regclass
    ) THEN
        ALTER TABLE automation.schedule_rule_revision
            ADD CONSTRAINT schedule_rule_revision_run_identity_unique
            UNIQUE (tenant_id, project_id, rule_id, rule_version, engineering_run_id);
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS automation.schedule_rule_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    rule_id UUID NOT NULL,
    rule_version BIGINT NOT NULL CHECK (rule_version > 0),
    event_type VARCHAR(32) NOT NULL
        CHECK (event_type IN ('schedule_rule.created','schedule_rule.revised')),
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload)='object' AND octet_length(payload::text)<=32768),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, rule_id, rule_version, event_type),
    CONSTRAINT schedule_rule_outbox_revision_run_fk
    FOREIGN KEY (tenant_id, project_id, rule_id, rule_version, engineering_run_id)
        REFERENCES automation.schedule_rule_revision
            (tenant_id, project_id, rule_id, rule_version, engineering_run_id)
        ON DELETE RESTRICT,
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (engineering_run_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (rule_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (correlation_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname='schedule_rule_outbox_revision_run_fk'
          AND conrelid='automation.schedule_rule_outbox'::regclass
    ) THEN
        ALTER TABLE automation.schedule_rule_outbox
            ADD CONSTRAINT schedule_rule_outbox_revision_run_fk
            FOREIGN KEY (tenant_id, project_id, rule_id, rule_version, engineering_run_id)
            REFERENCES automation.schedule_rule_revision
                (tenant_id, project_id, rule_id, rule_version, engineering_run_id)
            ON DELETE RESTRICT;
    END IF;
END $$;
CREATE INDEX IF NOT EXISTS idx_automation_schedule_rule_outbox_project
    ON automation.schedule_rule_outbox
        (tenant_id, project_id, occurred_at, event_id);
DROP TRIGGER IF EXISTS automation_schedule_rule_outbox_no_mutation
    ON automation.schedule_rule_outbox;
CREATE TRIGGER automation_schedule_rule_outbox_no_mutation
    BEFORE UPDATE OR DELETE ON automation.schedule_rule_outbox
    FOR EACH ROW EXECUTE FUNCTION automation.reject_schedule_history_mutation();
DROP TRIGGER IF EXISTS automation_schedule_rule_outbox_no_truncate
    ON automation.schedule_rule_outbox;
CREATE TRIGGER automation_schedule_rule_outbox_no_truncate
    BEFORE TRUNCATE ON automation.schedule_rule_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();
ALTER TABLE automation.schedule_rule_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.schedule_rule_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_schedule_rule_outbox_tenant_scope
    ON automation.schedule_rule_outbox;
CREATE POLICY automation_schedule_rule_outbox_tenant_scope
    ON automation.schedule_rule_outbox
    USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID)
    WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID);

CREATE TABLE IF NOT EXISTS automation.schedule_rule_command_idempotency (
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    operation VARCHAR(8) NOT NULL CHECK (operation IN ('create','revise')),
    idempotency_key_hash BYTEA NOT NULL CHECK (octet_length(idempotency_key_hash)=32),
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash)=32),
    response_status SMALLINT NOT NULL CHECK (response_status BETWEEN 200 AND 299),
    response_body JSONB NOT NULL
        CHECK (jsonb_typeof(response_body)='object' AND octet_length(response_body::text)<=65536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '24 hours'
        CHECK (retention_period > INTERVAL '0 seconds' AND retention_period <= INTERVAL '7 days'),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, project_id, actor_id, operation, idempotency_key_hash),
    CHECK (expires_at=created_at+retention_period),
    CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (project_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'::UUID)
);
CREATE INDEX IF NOT EXISTS idx_automation_schedule_rule_idempotency_expiry
    ON automation.schedule_rule_command_idempotency (tenant_id, expires_at);
ALTER TABLE automation.schedule_rule_command_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE automation.schedule_rule_command_idempotency FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS automation_schedule_rule_idempotency_tenant_scope
    ON automation.schedule_rule_command_idempotency;
CREATE POLICY automation_schedule_rule_idempotency_tenant_scope
    ON automation.schedule_rule_command_idempotency
    USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID)
    WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID);

COMMENT ON COLUMN automation.schedule_rule_revision.display_name IS
    'User-visible bounded label for the immutable Schedule Rule revision.';
COMMENT ON TABLE automation.schedule_rule_outbox IS
    'T: append-only transactional Rule-created/revised events; inserted with the Master revision and Audit facts.';
COMMENT ON TABLE automation.schedule_rule_command_idempotency IS
    'W: actor/project-scoped Schedule API response replay fence with 24-hour bounded retention.';

COMMIT;
