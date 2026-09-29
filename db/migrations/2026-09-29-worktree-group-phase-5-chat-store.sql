-- Cypher structural manifest.
-- CREATE
--   (f:File {name:"2026-09-29-worktree-group-phase-5-chat-store.sql",type:"file",language:"sql"}),
--   (m:Module {name:"group_chat_schema",type:"module",language:"sql"}),
--   (session:Class {name:"multica.group_chat_session",type:"class",language:"sql"}),
--   (message:Class {name:"multica.group_chat_message",type:"class",language:"sql"}),
--   (payload:Class {name:"multica.group_chat_message_payload",type:"class",language:"sql"}),
--   (run:Class {name:"multica.group_chat_run",type:"class",language:"sql"}),
--   (idempotency:Class {name:"multica.group_chat_idempotency",type:"class",language:"sql"}),
--   (dispatch:Class {name:"multica.group_chat_dispatch_event",type:"class",language:"sql"}),
--   (audit:Class {name:"multica.group_chat_audit_event",type:"class",language:"sql"}),
--   (reject:Function {name:"multica.reject_group_chat_fact_mutation",type:"function",language:"sql"}),
--   (append_only:Logic {name:"group_chat_append_only_triggers",type:"logic",language:"sql"}),
--   (rls:Logic {name:"group_chat_rls_policies",type:"logic",language:"sql"}),
--   (f)-[:CONTAINS]->(m),(m)-[:CONTAINS]->(session),(m)-[:CONTAINS]->(message),
--   (m)-[:CONTAINS]->(payload),
--   (m)-[:CONTAINS]->(run),(m)-[:CONTAINS]->(idempotency),(m)-[:CONTAINS]->(dispatch),
--   (m)-[:CONTAINS]->(audit),(m)-[:CONTAINS]->(reject),(m)-[:CONTAINS]->(append_only),
--   (m)-[:CONTAINS]->(rls),(append_only)-[:CALLS]->(reject);

-- Worktree Group Phase 5: immutable Transcript metadata, expiring encrypted payloads,
-- append-only dispatch facts and bounded run/idempotency work.
-- W/T classification is explicit per table; there are no Master tables in this migration.

BEGIN;

CREATE SCHEMA IF NOT EXISTS multica;

-- T: session creation is a business fact. The LangGraph thread ID is server generated,
-- immutable, and distinct from Chat Session, Task Card, WorkItem and Agent Session IDs.
CREATE TABLE IF NOT EXISTS multica.group_chat_session (
    session_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    origin_worktree_id UUID NOT NULL,
    langgraph_thread_id UUID NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, session_id),
    FOREIGN KEY (tenant_id, origin_worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);

-- T: transcript metadata is append-only; expiring body ciphertext lives in a separate W table.
CREATE TABLE IF NOT EXISTS multica.group_chat_message (
    message_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    session_id UUID NOT NULL,
    run_id UUID NOT NULL,
    payload_id UUID NOT NULL,
    role VARCHAR(16) NOT NULL CHECK (role IN ('user','assistant','tool')),
    scope VARCHAR(16) NOT NULL CHECK (scope IN ('WORKTREE','GLOBAL')),
    target_worktree_ids UUID[] NOT NULL CHECK (cardinality(target_worktree_ids) BETWEEN 1 AND 20),
    entity_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    correlation_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, message_id),
    FOREIGN KEY (tenant_id, session_id)
        REFERENCES multica.group_chat_session (tenant_id, session_id) ON DELETE RESTRICT
);

-- W: encrypted transcript payload; expiry is policy-resolved by the injected protector.
-- No FK to the immutable T metadata: cleanup can erase the body while preserving its audit fact.
CREATE TABLE IF NOT EXISTS multica.group_chat_message_payload (
    payload_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    ciphertext BYTEA NOT NULL CHECK (octet_length(ciphertext) > 0),
    wrapped_data_key BYTEA NOT NULL CHECK (octet_length(wrapped_data_key) > 0),
    key_id VARCHAR(200) NOT NULL,
    retention_period INTERVAL NOT NULL CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, payload_id)
);
CREATE INDEX IF NOT EXISTS idx_group_chat_message_payload_expiry
    ON multica.group_chat_message_payload (expires_at);

-- W: queued/running workflow intent is bounded work; expiry is extended by the consumer.
CREATE TABLE IF NOT EXISTS multica.group_chat_run (
    run_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    session_id UUID NOT NULL,
    origin_worktree_id UUID NOT NULL,
    scope VARCHAR(16) NOT NULL CHECK (scope IN ('WORKTREE','GLOBAL')),
    target_worktree_ids UUID[] NOT NULL CHECK (cardinality(target_worktree_ids) BETWEEN 1 AND 20),
    entity_refs JSONB NOT NULL DEFAULT '[]'::jsonb,
    state VARCHAR(16) NOT NULL DEFAULT 'queued'
        CHECK (state IN ('queued','running','waiting','completed','failed','cancelled')),
    correlation_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '30 days'
        CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, run_id),
    FOREIGN KEY (tenant_id, session_id)
        REFERENCES multica.group_chat_session (tenant_id, session_id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, origin_worktree_id)
        REFERENCES worktree_canvas_worktree (tenant_id, id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_group_chat_run_dispatch
    ON multica.group_chat_run (tenant_id, state, created_at, run_id)
    WHERE state IN ('queued','running','waiting');
CREATE INDEX IF NOT EXISTS idx_group_chat_run_expiry
    ON multica.group_chat_run (expires_at)
    WHERE state IN ('completed','failed','cancelled');

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'group_chat_message_run_fk'
          AND conrelid = 'multica.group_chat_message'::regclass
    ) THEN
        ALTER TABLE multica.group_chat_message
            ADD CONSTRAINT group_chat_message_run_fk
            FOREIGN KEY (tenant_id, run_id)
            REFERENCES multica.group_chat_run (tenant_id, run_id) ON DELETE RESTRICT;
    END IF;
END;
$$;

-- W: replay receipts are short lived; a different request hash under the same key conflicts.
CREATE TABLE IF NOT EXISTS multica.group_chat_idempotency (
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    response_body JSONB NOT NULL,
    retention_period INTERVAL NOT NULL DEFAULT INTERVAL '30 days'
        CHECK (retention_period > INTERVAL '0 seconds'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, actor_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_group_chat_idempotency_expiry
    ON multica.group_chat_idempotency (expires_at);

-- T: immutable dispatch intent; delivery/lease state belongs to the consumer's Work projection.
CREATE TABLE IF NOT EXISTS multica.group_chat_dispatch_event (
    event_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    run_id UUID NOT NULL,
    event_type VARCHAR(64) NOT NULL CHECK (event_type = 'group_chat.run.requested'),
    schema_version SMALLINT NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, event_id),
    UNIQUE (tenant_id, run_id, event_type),
    FOREIGN KEY (tenant_id, run_id)
        REFERENCES multica.group_chat_run (tenant_id, run_id) ON DELETE RESTRICT
);

-- T: audit facts are append-only and intentionally exclude message bodies/secrets.
CREATE TABLE IF NOT EXISTS multica.group_chat_audit_event (
    event_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    session_id UUID,
    run_id UUID,
    event_type VARCHAR(64) NOT NULL
        CHECK (event_type IN ('session.created','message.accepted','run.queued')),
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, session_id)
        REFERENCES multica.group_chat_session (tenant_id, session_id) ON DELETE RESTRICT,
    FOREIGN KEY (tenant_id, run_id)
        REFERENCES multica.group_chat_run (tenant_id, run_id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_group_chat_audit_session_time
    ON multica.group_chat_audit_event (tenant_id, session_id, occurred_at DESC);

CREATE OR REPLACE FUNCTION multica.reject_group_chat_fact_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'group chat transaction facts are append-only';
END;
$$;

DROP TRIGGER IF EXISTS group_chat_session_no_mutation ON multica.group_chat_session;
CREATE TRIGGER group_chat_session_no_mutation
    BEFORE UPDATE OR DELETE ON multica.group_chat_session
    FOR EACH ROW EXECUTE FUNCTION multica.reject_group_chat_fact_mutation();
DROP TRIGGER IF EXISTS group_chat_message_no_mutation ON multica.group_chat_message;
CREATE TRIGGER group_chat_message_no_mutation
    BEFORE UPDATE OR DELETE ON multica.group_chat_message
    FOR EACH ROW EXECUTE FUNCTION multica.reject_group_chat_fact_mutation();
DROP TRIGGER IF EXISTS group_chat_dispatch_event_no_mutation ON multica.group_chat_dispatch_event;
CREATE TRIGGER group_chat_dispatch_event_no_mutation
    BEFORE UPDATE OR DELETE ON multica.group_chat_dispatch_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_group_chat_fact_mutation();
DROP TRIGGER IF EXISTS group_chat_audit_event_no_mutation ON multica.group_chat_audit_event;
CREATE TRIGGER group_chat_audit_event_no_mutation
    BEFORE UPDATE OR DELETE ON multica.group_chat_audit_event
    FOR EACH ROW EXECUTE FUNCTION multica.reject_group_chat_fact_mutation();

ALTER TABLE multica.group_chat_session ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_session FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_session_select ON multica.group_chat_session;
CREATE POLICY group_chat_session_select ON multica.group_chat_session
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );
DROP POLICY IF EXISTS group_chat_session_insert ON multica.group_chat_session;
CREATE POLICY group_chat_session_insert ON multica.group_chat_session
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

ALTER TABLE multica.group_chat_message ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_message FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_message_select ON multica.group_chat_message;
CREATE POLICY group_chat_message_select ON multica.group_chat_message
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );
DROP POLICY IF EXISTS group_chat_message_insert ON multica.group_chat_message;
CREATE POLICY group_chat_message_insert ON multica.group_chat_message
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

ALTER TABLE multica.group_chat_message_payload ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_message_payload FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_message_payload_tenant_actor_scope ON multica.group_chat_message_payload;
CREATE POLICY group_chat_message_payload_tenant_actor_scope ON multica.group_chat_message_payload
    USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    )
    WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

ALTER TABLE multica.group_chat_run ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_run FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_run_tenant_actor_scope ON multica.group_chat_run;
CREATE POLICY group_chat_run_tenant_actor_scope ON multica.group_chat_run
    USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    )
    WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

ALTER TABLE multica.group_chat_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_idempotency FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_idempotency_tenant_actor_scope ON multica.group_chat_idempotency;
CREATE POLICY group_chat_idempotency_tenant_actor_scope ON multica.group_chat_idempotency
    USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    )
    WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

ALTER TABLE multica.group_chat_dispatch_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_dispatch_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_dispatch_event_tenant_scope ON multica.group_chat_dispatch_event;
CREATE POLICY group_chat_dispatch_event_tenant_scope ON multica.group_chat_dispatch_event
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

ALTER TABLE multica.group_chat_audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.group_chat_audit_event FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS group_chat_audit_event_tenant_actor_select ON multica.group_chat_audit_event;
CREATE POLICY group_chat_audit_event_tenant_actor_select ON multica.group_chat_audit_event
    FOR SELECT USING (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );
DROP POLICY IF EXISTS group_chat_audit_event_tenant_actor_insert ON multica.group_chat_audit_event;
CREATE POLICY group_chat_audit_event_tenant_actor_insert ON multica.group_chat_audit_event
    FOR INSERT WITH CHECK (
        tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID
        AND actor_id = NULLIF(current_setting('app.actor_id', true), '')::UUID
    );

COMMENT ON TABLE multica.group_chat_session IS
    'T: immutable Group Chat session fact with a server-generated LangGraph thread ID.';
COMMENT ON TABLE multica.group_chat_message IS
    'T: append-only Worktree-scoped transcript metadata; encrypted body is held in expiring W payload.';
COMMENT ON TABLE multica.group_chat_message_payload IS
    'W: encrypted transcript ciphertext and wrapped data key; physically purge after expires_at.';
COMMENT ON TABLE multica.group_chat_run IS
    'W: bounded Group Chat execution intent; consumer owns state transitions and extends expiry.';
COMMENT ON TABLE multica.group_chat_idempotency IS
    'W: 30-day replay receipt keyed by tenant, actor and idempotency key.';
COMMENT ON TABLE multica.group_chat_dispatch_event IS
    'T: append-only transactional outbox event; consumer delivery state is stored separately.';
COMMENT ON TABLE multica.group_chat_audit_event IS
    'T: append-only, tenant/actor-scoped audit facts; UPDATE and DELETE are prohibited.';

COMMIT;
