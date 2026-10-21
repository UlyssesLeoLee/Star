-- @cypher schema=1 source_sha256=fdab604ac60a7b303d539323bbb1ebefa7eff3f2cf34ed58373a718d22ffa948
-- MERGE (self:File {path:"db/migrations/2026-10-20-worktree-management-outbox.sql"})
-- MERGE (plan:Table {id:"multica.worktree_management_plan"})
-- MERGE (outbox:Table {id:"multica.worktree_management_outbox"})
-- MERGE (worktree:Table {id:"public.worktree_canvas_worktree"})
-- MERGE (guard:Function {id:"multica.reject_worktree_management_outbox_mutation"})
-- MERGE (no_mutation:Config {id:"worktree_management_outbox_no_mutation"})
-- MERGE (no_truncate:Config {id:"worktree_management_outbox_no_truncate"})
-- MERGE (payload_guard:Config {id:"worktree_management_outbox_payload_guard"})
-- MERGE (self)-[:DEPENDS_ON]->(plan)
-- MERGE (self)-[:DEPENDS_ON]->(worktree)
-- MERGE (self)-[:DEFINES]->(outbox)
-- MERGE (self)-[:DEFINES]->(guard)
-- MERGE (self)-[:CONFIGURES]->(no_mutation)
-- MERGE (self)-[:CONFIGURES]->(no_truncate)
-- MERGE (self)-[:CONFIGURES]->(payload_guard)
-- MERGE (no_mutation)-[:USES]->(guard)
-- MERGE (no_mutation)-[:USES]->(outbox)
-- MERGE (no_truncate)-[:USES]->(guard)
-- MERGE (no_truncate)-[:USES]->(outbox)
-- @endcypher

-- Worktree owner-local Transaction events for logical archive/restore only.
-- The existing management plan result_body remains the replayable API receipt;
-- this Outbox records the versioned fact in the same owner transaction.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

CREATE TABLE IF NOT EXISTS multica.worktree_management_outbox (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    worktree_id UUID NOT NULL,
    worktree_version INTEGER NOT NULL CHECK (worktree_version > 0),
    plan_id UUID NOT NULL,
    event_type VARCHAR(32) NOT NULL
        CHECK (event_type IN ('worktree.archived', 'worktree.restored')),
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version = 1),
    actor_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL
        CHECK (jsonb_typeof(payload) = 'object' AND octet_length(payload::TEXT) <= 8192),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_worktree_management_outbox_plan UNIQUE (tenant_id, plan_id),
    CONSTRAINT uq_worktree_management_outbox_version
        UNIQUE (tenant_id, worktree_id, worktree_version),
    CONSTRAINT ck_worktree_management_outbox_payload_shape CHECK (
        payload ?& ARRAY['worktree_id', 'plan_id', 'transition', 'before', 'after']
        AND payload - ARRAY['worktree_id', 'plan_id', 'transition', 'before', 'after'] = '{}'::JSONB
        AND payload->>'worktree_id' = worktree_id::TEXT
        AND payload->>'plan_id' = plan_id::TEXT
        AND jsonb_typeof(payload->'before') = 'object'
        AND jsonb_typeof(payload->'after') = 'object'
        AND (payload->'before') ?& ARRAY['archived', 'version']
        AND (payload->'before') - ARRAY['archived', 'version'] = '{}'::JSONB
        AND (payload->'after') ?& ARRAY['archived', 'version']
        AND (payload->'after') - ARRAY['archived', 'version'] = '{}'::JSONB
        AND jsonb_typeof(payload#>'{before,archived}') = 'boolean'
        AND jsonb_typeof(payload#>'{after,archived}') = 'boolean'
        AND CASE
            WHEN jsonb_typeof(payload#>'{before,version}') = 'number'
             AND jsonb_typeof(payload#>'{after,version}') = 'number'
            THEN (payload#>>'{before,version}')::NUMERIC >= 0
             AND trunc((payload#>>'{before,version}')::NUMERIC)
                 = (payload#>>'{before,version}')::NUMERIC
             AND (payload#>>'{after,version}')::NUMERIC = worktree_version::NUMERIC
             AND (payload#>>'{after,version}')::NUMERIC
                 = (payload#>>'{before,version}')::NUMERIC + 1
            ELSE FALSE
        END
        AND (
            (event_type = 'worktree.archived'
                AND payload->>'transition' = 'archive'
                AND payload#>>'{before,archived}' = 'false'
                AND payload#>>'{after,archived}' = 'true')
            OR
            (event_type = 'worktree.restored'
                AND payload->>'transition' = 'restore'
                AND payload#>>'{before,archived}' = 'true'
                AND payload#>>'{after,archived}' = 'false')
        )
    ),
    CONSTRAINT fk_worktree_management_outbox_worktree
        FOREIGN KEY (tenant_id, worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT,
    CONSTRAINT fk_worktree_management_outbox_plan
        FOREIGN KEY (tenant_id, plan_id)
        REFERENCES multica.worktree_management_plan (tenant_id, plan_id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_worktree_management_outbox_delivery
    ON multica.worktree_management_outbox (occurred_at, event_id);

CREATE OR REPLACE FUNCTION multica.reject_worktree_management_outbox_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Worktree management Outbox Transaction rows are append-only'
        USING ERRCODE = '55000';
END;
$$;

DROP TRIGGER IF EXISTS worktree_management_outbox_no_mutation
    ON multica.worktree_management_outbox;
CREATE TRIGGER worktree_management_outbox_no_mutation
    BEFORE UPDATE OR DELETE ON multica.worktree_management_outbox
    FOR EACH ROW EXECUTE FUNCTION multica.reject_worktree_management_outbox_mutation();
DROP TRIGGER IF EXISTS worktree_management_outbox_no_truncate
    ON multica.worktree_management_outbox;
CREATE TRIGGER worktree_management_outbox_no_truncate
    BEFORE TRUNCATE ON multica.worktree_management_outbox
    FOR EACH STATEMENT EXECUTE FUNCTION multica.reject_worktree_management_outbox_mutation();

ALTER TABLE multica.worktree_management_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.worktree_management_outbox FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS worktree_management_outbox_tenant_scope
    ON multica.worktree_management_outbox;
CREATE POLICY worktree_management_outbox_tenant_scope
    ON multica.worktree_management_outbox FOR SELECT
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS worktree_management_outbox_tenant_insert
    ON multica.worktree_management_outbox;
CREATE POLICY worktree_management_outbox_tenant_insert
    ON multica.worktree_management_outbox FOR INSERT
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

COMMENT ON TABLE multica.worktree_management_outbox IS
    'Append-only Worktree owner facts for logical archive/restore. Delivery workers and transport are separate concerns.';
COMMENT ON COLUMN multica.worktree_management_outbox.payload IS
    'Versioned state transition with identifiers and Worktree versions only; excludes host paths, credentials, prompts, and raw logs.';

COMMIT;
