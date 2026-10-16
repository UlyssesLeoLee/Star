-- @cypher schema=1 source_sha256=e83350da742eda0ba434d6e135a5e137ffe76bdd0cb8f5a36fc75105af985905
-- CREATE (self:File {path:"2026-10-16-task-run-outbox-delivery.sql",type:"file",language:"sql"}),(m:Module {name:"task_run_outbox_delivery_migration",type:"module",language:"sql"}),(outbox:Table {name:"multica.task_run_outbox",type:"table",language:"sql"}),(work:Table {name:"multica.task_run_outbox_delivery",type:"table",language:"sql"}),(receipt:Table {name:"multica.task_run_outbox_delivery_receipt",type:"table",language:"sql"}),(enqueue:Function {name:"multica.enqueue_task_run_outbox_delivery",type:"function",language:"sql"}),(reconcile:Function {name:"multica.reconcile_task_run_outbox_delivery",type:"function",language:"sql"}),(claim:Function {name:"multica.claim_task_run_outbox_delivery",type:"function",language:"sql"}),(settle:Function {name:"multica.settle_task_run_outbox_delivery",type:"function",language:"sql"}),(immutable:Function {name:"multica.reject_task_run_outbox_delivery_receipt_mutation",type:"function",language:"sql"}),(enqueue_trigger:Symbol {name:"task_run_outbox_delivery_enqueue",type:"trigger"}),(receipt_trigger:Symbol {name:"task_run_outbox_delivery_receipt_worm",type:"trigger"}),(owner:Config {name:"star_multica_run_owner",type:"database_role"}),(delivery_owner:Config {name:"star_task_run_delivery_owner",type:"database_role"});
-- CREATE (self)-[:CONTAINS]->(m),(m)-[:USES]->(outbox),(m)-[:DEFINES]->(work),(m)-[:DEFINES]->(receipt),(m)-[:DEFINES]->(enqueue),(m)-[:DEFINES]->(reconcile),(m)-[:DEFINES]->(claim),(m)-[:DEFINES]->(settle),(m)-[:DEFINES]->(immutable),(m)-[:DEFINES]->(enqueue_trigger),(m)-[:DEFINES]->(receipt_trigger),(m)-[:DEFINES]->(delivery_owner),(enqueue)-[:READS]->(outbox),(enqueue)-[:READS]->(work),(enqueue)-[:READS]->(receipt),(enqueue)-[:WRITES]->(work),(reconcile)-[:READS]->(outbox),(reconcile)-[:READS]->(work),(reconcile)-[:READS]->(receipt),(reconcile)-[:WRITES]->(work),(claim)-[:CALLS]->(reconcile),(claim)-[:READS]->(outbox),(claim)-[:READS]->(work),(claim)-[:WRITES]->(work),(claim)-[:WRITES]->(receipt),(settle)-[:READS]->(work),(settle)-[:WRITES]->(work),(settle)-[:WRITES]->(receipt),(outbox)-[:EMITS]->(enqueue_trigger),(enqueue_trigger)-[:CALLS]->(enqueue),(receipt_trigger)-[:CALLS]->(immutable),(enqueue)-[:CONFIGURES]->(delivery_owner),(reconcile)-[:CONFIGURES]->(delivery_owner),(claim)-[:CONFIGURES]->(delivery_owner),(settle)-[:CONFIGURES]->(delivery_owner),(enqueue)-[:EXPOSES]->(owner),(reconcile)-[:EXPOSES]->(owner),(claim)-[:EXPOSES]->(owner),(settle)-[:EXPOSES]->(owner);
-- @endcypher

-- Durable, bounded Task Run event delivery work and consumer Inbox receipts.
-- This contract is transport-neutral. An accepted receipt stores the Inbox commit receipt returned
-- by an authenticated ingress owner. The caller must authenticate that response; this database
-- cannot independently prove a transaction committed in another owner's database. A broker ACK
-- or socket write alone is not an accepted receipt.
BEGIN;

DO $role$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_task_run_delivery_owner') THEN
        CREATE ROLE star_task_run_delivery_owner
            NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS;
    END IF;
    IF EXISTS (
        SELECT 1
          FROM pg_auth_members membership
          JOIN pg_roles granted_role ON granted_role.oid = membership.roleid
         WHERE granted_role.rolname = 'star_task_run_delivery_owner'
    ) THEN
        RAISE EXCEPTION 'star_task_run_delivery_owner must have no role members';
    END IF;
END;
$role$;
ALTER ROLE star_task_run_delivery_owner
    NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOBYPASSRLS;

-- event_id is globally unique, but the delivery/receipt rows also carry tenant and Run scope.
-- Enforce that denormalized scope against the immutable source event at the database boundary.
CREATE UNIQUE INDEX IF NOT EXISTS uq_task_run_outbox_delivery_source_scope
    ON multica.task_run_outbox (tenant_id, engineering_run_id, event_id);

CREATE TABLE IF NOT EXISTS multica.task_run_outbox_delivery (
    tenant_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    event_id UUID NOT NULL,
    delivery_state VARCHAR(8) NOT NULL DEFAULT 'pending'
        CHECK (delivery_state IN ('pending', 'leased', 'retry')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 12),
    lease_owner_id UUID,
    lease_expires_at TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp() + INTERVAL '7 days',
    PRIMARY KEY (tenant_id, event_id),
    CHECK ((delivery_state = 'leased' AND lease_owner_id IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (delivery_state <> 'leased' AND lease_owner_id IS NULL AND lease_expires_at IS NULL)),
    CHECK (lease_expires_at IS NULL OR lease_expires_at <= expires_at),
    CHECK (expires_at = created_at + INTERVAL '7 days'),
    FOREIGN KEY (tenant_id, engineering_run_id, event_id)
        REFERENCES multica.task_run_outbox (tenant_id, engineering_run_id, event_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_task_run_outbox_delivery_due
    ON multica.task_run_outbox_delivery
        (tenant_id, engineering_run_id, next_attempt_at, event_id)
    WHERE delivery_state IN ('pending', 'retry');
CREATE INDEX IF NOT EXISTS idx_task_run_outbox_delivery_lease
    ON multica.task_run_outbox_delivery
        (tenant_id, engineering_run_id, lease_expires_at, event_id)
    WHERE delivery_state = 'leased';

CREATE TABLE IF NOT EXISTS multica.task_run_outbox_delivery_receipt (
    tenant_id UUID NOT NULL,
    engineering_run_id UUID NOT NULL,
    event_id UUID NOT NULL,
    outcome VARCHAR(8) NOT NULL CHECK (outcome IN ('accepted', 'dead')),
    attempts INTEGER NOT NULL CHECK (attempts BETWEEN 0 AND 12),
    terminal_reason VARCHAR(32) NOT NULL CHECK (
        terminal_reason IN ('consumer_inbox_committed', 'invalid_envelope', 'consumer_rejected',
                            'attempts_exhausted', 'retention_elapsed')
    ),
    consumer_owner_id UUID,
    consumer_inbox_receipt_id UUID,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (tenant_id, event_id),
    CHECK ((outcome = 'accepted') = (consumer_owner_id IS NOT NULL AND consumer_inbox_receipt_id IS NOT NULL)),
    CHECK ((outcome = 'accepted') = (terminal_reason = 'consumer_inbox_committed')),
    CHECK (consumer_owner_id IS NULL OR consumer_owner_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    CHECK (consumer_inbox_receipt_id IS NULL OR consumer_inbox_receipt_id <> '00000000-0000-0000-0000-000000000000'::UUID),
    FOREIGN KEY (tenant_id, engineering_run_id, event_id)
        REFERENCES multica.task_run_outbox (tenant_id, engineering_run_id, event_id)
        ON DELETE RESTRICT
);

CREATE OR REPLACE FUNCTION multica.reject_task_run_outbox_delivery_receipt_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Task Run delivery receipts are append-only' USING ERRCODE = '55000';
END;
$$;
DROP TRIGGER IF EXISTS task_run_outbox_delivery_receipt_worm
    ON multica.task_run_outbox_delivery_receipt;
CREATE TRIGGER task_run_outbox_delivery_receipt_worm
    BEFORE UPDATE OR DELETE OR TRUNCATE ON multica.task_run_outbox_delivery_receipt
    FOR EACH STATEMENT
    EXECUTE FUNCTION multica.reject_task_run_outbox_delivery_receipt_mutation();

ALTER TABLE multica.task_run_outbox_delivery ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_run_outbox_delivery FORCE ROW LEVEL SECURITY;
ALTER TABLE multica.task_run_outbox_delivery_receipt ENABLE ROW LEVEL SECURITY;
ALTER TABLE multica.task_run_outbox_delivery_receipt FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_run_outbox_delivery_owner_tenant ON multica.task_run_outbox_delivery;
CREATE POLICY task_run_outbox_delivery_owner_tenant ON multica.task_run_outbox_delivery
    FOR ALL TO star_task_run_delivery_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_run_outbox_delivery_receipt_owner_select
    ON multica.task_run_outbox_delivery_receipt;
CREATE POLICY task_run_outbox_delivery_receipt_owner_select
    ON multica.task_run_outbox_delivery_receipt FOR SELECT TO star_task_run_delivery_owner
    USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
DROP POLICY IF EXISTS task_run_outbox_delivery_receipt_owner_insert
    ON multica.task_run_outbox_delivery_receipt;
CREATE POLICY task_run_outbox_delivery_receipt_owner_insert
    ON multica.task_run_outbox_delivery_receipt FOR INSERT TO star_task_run_delivery_owner
    WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);

REVOKE ALL ON multica.task_run_outbox_delivery,
              multica.task_run_outbox_delivery_receipt FROM PUBLIC, star_multica_run_owner;
GRANT USAGE ON SCHEMA multica TO star_task_run_delivery_owner, star_multica_run_owner;
GRANT SELECT ON multica.task_run_outbox TO star_task_run_delivery_owner;
GRANT SELECT, INSERT, UPDATE, DELETE ON multica.task_run_outbox_delivery
    TO star_task_run_delivery_owner;
GRANT SELECT, INSERT ON multica.task_run_outbox_delivery_receipt
    TO star_task_run_delivery_owner;

CREATE OR REPLACE FUNCTION multica.enqueue_task_run_outbox_delivery()
RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    pending_count BIGINT;
BEGIN
    IF active_tenant IS NULL OR active_tenant <> NEW.tenant_id
       OR NEW.engineering_run_id IS NULL THEN
        RAISE EXCEPTION 'Task Run delivery enqueue requires the current tenant and Run scope'
            USING ERRCODE = '23514', CONSTRAINT = 'ck_task_run_delivery_scope_required';
    END IF;

    -- Serialize only this Run's short enqueue operation so concurrent agents cannot exceed
    -- the durable backlog budget. Different Runs remain independent.
    PERFORM pg_advisory_xact_lock(hashtextextended(
        'task-run-outbox-delivery:' || NEW.tenant_id::TEXT || ':' || NEW.engineering_run_id::TEXT, 0));

    IF EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery d
               WHERE d.tenant_id = NEW.tenant_id AND d.event_id = NEW.event_id)
       OR EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery_receipt r
                  WHERE r.tenant_id = NEW.tenant_id AND r.event_id = NEW.event_id) THEN
        RETURN NEW;
    END IF;

    SELECT count(*) INTO pending_count
      FROM multica.task_run_outbox_delivery d
     WHERE d.tenant_id = NEW.tenant_id AND d.engineering_run_id = NEW.engineering_run_id;
    IF pending_count >= 1024 THEN
        RAISE EXCEPTION 'Task Run delivery backlog budget exceeded'
            USING ERRCODE = '54000', CONSTRAINT = 'ck_task_run_delivery_backlog_budget';
    END IF;

    INSERT INTO multica.task_run_outbox_delivery (tenant_id, engineering_run_id, event_id)
    VALUES (NEW.tenant_id, NEW.engineering_run_id, NEW.event_id);
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION multica.reconcile_task_run_outbox_delivery(p_limit INTEGER)
RETURNS INTEGER LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    source_event RECORD;
    pending_count BIGINT;
    added INTEGER := 0;
BEGIN
    IF active_tenant IS NULL OR active_tenant = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_limit IS NULL OR p_limit NOT BETWEEN 1 AND 8 THEN
        RAISE EXCEPTION 'Invalid Task Run delivery reconciliation boundary' USING ERRCODE = '22023';
    END IF;

    -- No timestamp/high-water cursor: each bounded pass rechecks missing event IDs so a late
    -- committing transaction cannot be skipped because its event time sorts behind the cursor.
    FOR source_event IN
        SELECT o.tenant_id, o.engineering_run_id, o.event_id
          FROM multica.task_run_outbox o
         WHERE o.tenant_id = active_tenant
           AND (SELECT count(*)
                  FROM multica.task_run_outbox_delivery capacity
                 WHERE capacity.tenant_id = o.tenant_id
                   AND capacity.engineering_run_id = o.engineering_run_id) < 1024
           AND NOT EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery d
                            WHERE d.tenant_id = o.tenant_id AND d.event_id = o.event_id)
           AND NOT EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery_receipt r
                            WHERE r.tenant_id = o.tenant_id AND r.event_id = o.event_id)
         ORDER BY o.event_id
         LIMIT p_limit
    LOOP
        PERFORM pg_advisory_xact_lock(hashtextextended(
            'task-run-outbox-delivery:' || source_event.tenant_id::TEXT || ':'
            || source_event.engineering_run_id::TEXT, 0));

        IF EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery d
                   WHERE d.tenant_id = source_event.tenant_id AND d.event_id = source_event.event_id)
           OR EXISTS (SELECT 1 FROM multica.task_run_outbox_delivery_receipt r
                      WHERE r.tenant_id = source_event.tenant_id AND r.event_id = source_event.event_id) THEN
            CONTINUE;
        END IF;

        SELECT count(*) INTO pending_count
          FROM multica.task_run_outbox_delivery d
         WHERE d.tenant_id = source_event.tenant_id
           AND d.engineering_run_id = source_event.engineering_run_id;
        IF pending_count < 1024 THEN
            INSERT INTO multica.task_run_outbox_delivery (tenant_id, engineering_run_id, event_id)
            VALUES (source_event.tenant_id, source_event.engineering_run_id, source_event.event_id);
            added := added + 1;
        END IF;
    END LOOP;
    RETURN added;
END;
$$;

CREATE OR REPLACE FUNCTION multica.claim_task_run_outbox_delivery(
    p_worker UUID, p_limit INTEGER, p_lease_seconds INTEGER
)
RETURNS TABLE (
    tenant_id UUID, project_id UUID, engineering_run_id UUID, event_id UUID,
    work_item_id UUID, aggregate_type VARCHAR(16), aggregate_version INTEGER,
    event_type VARCHAR(32), schema_version INTEGER, actor_id UUID, correlation_id UUID,
    payload JSONB, occurred_at TIMESTAMPTZ, attempt_count INTEGER, lease_expires_at TIMESTAMPTZ
)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    expired RECORD;
BEGIN
    IF active_tenant IS NULL OR active_tenant = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_worker IS NULL OR p_worker = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_limit IS NULL OR p_limit NOT BETWEEN 1 AND 8
       OR p_lease_seconds IS NULL OR p_lease_seconds NOT BETWEEN 30 AND 300 THEN
        RAISE EXCEPTION 'Invalid Task Run delivery claim boundary' USING ERRCODE = '22023';
    END IF;

    PERFORM multica.reconcile_task_run_outbox_delivery(p_limit);

    FOR expired IN
        SELECT d.tenant_id, d.engineering_run_id, d.event_id, d.attempt_count, d.expires_at
          FROM multica.task_run_outbox_delivery d
         WHERE d.tenant_id = active_tenant
           AND (d.expires_at <= clock_timestamp()
             OR (d.attempt_count >= 12 AND
                 (d.delivery_state <> 'leased' OR d.lease_expires_at <= clock_timestamp())))
         ORDER BY d.expires_at, d.event_id
         LIMIT p_limit
         FOR UPDATE OF d SKIP LOCKED
    LOOP
        INSERT INTO multica.task_run_outbox_delivery_receipt (
            tenant_id, engineering_run_id, event_id, outcome, attempts, terminal_reason
        ) VALUES (
            expired.tenant_id, expired.engineering_run_id, expired.event_id, 'dead',
            expired.attempt_count,
            CASE WHEN expired.expires_at <= clock_timestamp()
                 THEN 'retention_elapsed' ELSE 'attempts_exhausted' END
        ) ON CONFLICT (tenant_id, event_id) DO NOTHING;
        DELETE FROM multica.task_run_outbox_delivery d
         WHERE d.tenant_id = expired.tenant_id AND d.event_id = expired.event_id;
    END LOOP;

    RETURN QUERY
    WITH due AS (
        SELECT d.tenant_id, d.event_id
          FROM multica.task_run_outbox_delivery d
         WHERE d.tenant_id = active_tenant AND d.expires_at > clock_timestamp()
           AND d.attempt_count < 12
           AND ((d.delivery_state IN ('pending', 'retry') AND d.next_attempt_at <= clock_timestamp())
             OR (d.delivery_state = 'leased' AND d.lease_expires_at <= clock_timestamp()))
         ORDER BY d.next_attempt_at, d.event_id
         LIMIT p_limit
         FOR UPDATE OF d SKIP LOCKED
    ), claimed AS (
        UPDATE multica.task_run_outbox_delivery d
           SET delivery_state = 'leased', attempt_count = d.attempt_count + 1,
               lease_owner_id = p_worker,
               lease_expires_at = LEAST(clock_timestamp()
                   + make_interval(secs => p_lease_seconds::DOUBLE PRECISION), d.expires_at)
          FROM due
         WHERE d.tenant_id = due.tenant_id AND d.event_id = due.event_id
        RETURNING d.tenant_id, d.event_id, d.attempt_count, d.lease_expires_at
    )
    SELECT o.tenant_id, o.project_id, o.engineering_run_id, o.event_id, o.work_item_id,
           o.aggregate_type, o.aggregate_version, o.event_type, o.schema_version,
           o.actor_id, o.correlation_id, o.payload, o.occurred_at,
           c.attempt_count, c.lease_expires_at
      FROM claimed c
      JOIN multica.task_run_outbox o
        ON o.tenant_id = c.tenant_id AND o.event_id = c.event_id
     ORDER BY c.event_id;
END;
$$;

-- Current code rejects accepted settlement. A future acceptance routine must authenticate the
-- ingress-owner response and bind its receipt to the claimed event before persisting these IDs.
CREATE OR REPLACE FUNCTION multica.settle_task_run_outbox_delivery(
    p_event_id UUID, p_worker UUID, p_attempt_count INTEGER, p_action TEXT,
    p_consumer_owner_id UUID, p_consumer_inbox_receipt_id UUID, p_dead_reason TEXT
)
RETURNS VOID LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    active_tenant UUID := NULLIF(current_setting('app.tenant_id', true), '')::UUID;
    delivery multica.task_run_outbox_delivery%ROWTYPE;
    reason TEXT;
BEGIN
    IF active_tenant IS NULL OR active_tenant = '00000000-0000-0000-0000-000000000000'::UUID
       OR p_event_id IS NULL OR p_worker IS NULL OR p_attempt_count IS NULL
        OR p_action IS NULL OR p_action NOT IN ('accepted', 'retry', 'dead') THEN
        RAISE EXCEPTION 'Invalid Task Run delivery settlement boundary' USING ERRCODE = '22023';
    END IF;
    IF p_action = 'accepted' THEN
        RAISE EXCEPTION 'Accepted settlement is disabled until the trusted receiver receipt contract is implemented'
            USING ERRCODE = '55000';
    END IF;

    SELECT * INTO delivery
      FROM multica.task_run_outbox_delivery d
     WHERE d.tenant_id = active_tenant AND d.event_id = p_event_id
     FOR UPDATE;
    IF NOT FOUND OR delivery.delivery_state <> 'leased'
       OR delivery.lease_owner_id IS DISTINCT FROM p_worker
       OR delivery.attempt_count IS DISTINCT FROM p_attempt_count
       OR delivery.lease_expires_at <= clock_timestamp()
       OR delivery.expires_at <= clock_timestamp() THEN
        RAISE EXCEPTION 'Task Run delivery lease fence lost' USING ERRCODE = '40001';
    END IF;

    PERFORM pg_advisory_xact_lock(hashtextextended(
        'task-run-outbox-delivery:' || active_tenant::TEXT || ':'
        || delivery.engineering_run_id::TEXT, 0));

    IF p_action = 'retry' THEN
        IF p_consumer_owner_id IS NOT NULL OR p_consumer_inbox_receipt_id IS NOT NULL
           OR p_dead_reason IS NOT NULL THEN
            RAISE EXCEPTION 'Retry settlement must not include terminal evidence' USING ERRCODE = '22023';
        END IF;
        IF delivery.attempt_count < 12 THEN
            UPDATE multica.task_run_outbox_delivery d
               SET delivery_state = 'retry', lease_owner_id = NULL, lease_expires_at = NULL,
                   next_attempt_at = LEAST(clock_timestamp()
                       + make_interval(secs => LEAST(300, power(2, delivery.attempt_count)::INTEGER)),
                       d.expires_at)
             WHERE d.tenant_id = active_tenant AND d.event_id = p_event_id;
            RETURN;
        END IF;
        reason := 'attempts_exhausted';
    ELSIF p_action = 'dead' THEN
        IF p_consumer_owner_id IS NOT NULL OR p_consumer_inbox_receipt_id IS NOT NULL
           OR p_dead_reason IS NULL
           OR p_dead_reason NOT IN ('invalid_envelope', 'consumer_rejected') THEN
            RAISE EXCEPTION 'Dead settlement requires an allowlisted reason' USING ERRCODE = '22023';
        END IF;
        reason := p_dead_reason;
    ELSE
        RAISE EXCEPTION 'Invalid Task Run delivery settlement action' USING ERRCODE = '22023';
    END IF;

    INSERT INTO multica.task_run_outbox_delivery_receipt (
        tenant_id, engineering_run_id, event_id, outcome, attempts, terminal_reason
    ) VALUES (
        active_tenant, delivery.engineering_run_id, p_event_id, 'dead',
        delivery.attempt_count, reason
    );
    DELETE FROM multica.task_run_outbox_delivery d
     WHERE d.tenant_id = active_tenant AND d.event_id = p_event_id;
END;
$$;

DROP TRIGGER IF EXISTS task_run_outbox_delivery_enqueue ON multica.task_run_outbox;
CREATE TRIGGER task_run_outbox_delivery_enqueue
    AFTER INSERT ON multica.task_run_outbox
    FOR EACH ROW EXECUTE FUNCTION multica.enqueue_task_run_outbox_delivery();

ALTER FUNCTION multica.enqueue_task_run_outbox_delivery() OWNER TO star_task_run_delivery_owner;
ALTER FUNCTION multica.reconcile_task_run_outbox_delivery(INTEGER) OWNER TO star_task_run_delivery_owner;
ALTER FUNCTION multica.claim_task_run_outbox_delivery(UUID, INTEGER, INTEGER) OWNER TO star_task_run_delivery_owner;
ALTER FUNCTION multica.settle_task_run_outbox_delivery(UUID, UUID, INTEGER, TEXT, UUID, UUID, TEXT)
    OWNER TO star_task_run_delivery_owner;
REVOKE ALL ON FUNCTION multica.enqueue_task_run_outbox_delivery() FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.reconcile_task_run_outbox_delivery(INTEGER) FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.claim_task_run_outbox_delivery(UUID, INTEGER, INTEGER) FROM PUBLIC;
REVOKE ALL ON FUNCTION multica.settle_task_run_outbox_delivery(UUID, UUID, INTEGER, TEXT, UUID, UUID, TEXT)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION multica.enqueue_task_run_outbox_delivery() TO star_multica_run_owner;
GRANT EXECUTE ON FUNCTION multica.reconcile_task_run_outbox_delivery(INTEGER) TO star_multica_run_owner;
GRANT EXECUTE ON FUNCTION multica.claim_task_run_outbox_delivery(UUID, INTEGER, INTEGER)
    TO star_multica_run_owner;
GRANT EXECUTE ON FUNCTION multica.settle_task_run_outbox_delivery(UUID, UUID, INTEGER, TEXT, UUID, UUID, TEXT)
    TO star_multica_run_owner;

COMMENT ON TABLE multica.task_run_outbox_delivery IS
    'W: tenant/Run-scoped bounded delivery work; 1024 outstanding events per Run, 7-day lease/retry retention, no transport-specific state.';
COMMENT ON TABLE multica.task_run_outbox_delivery_receipt IS
    'T: immutable terminal delivery evidence. Current settlement accepts retry/dead only. A future accepted receipt identity must be authenticated and event-bound by a trusted dispatcher; this source database cannot prove a remote Inbox transaction by itself.';

COMMIT;
