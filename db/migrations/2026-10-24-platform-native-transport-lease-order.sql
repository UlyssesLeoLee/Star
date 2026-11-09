-- @cypher schema=1 source_sha256=205eba4ef6786abdcbf0cd534020e906cc4debeef93bbf93fdc1fae8342d9c12
-- MERGE (self:File {path:"db/migrations/2026-10-24-platform-native-transport-lease-order.sql"})
-- MERGE (pull:Function {id:"platform_transport.pull_one"})
-- MERGE (log_master:Table {id:"platform_transport.log_master",class:"Master"})
-- MERGE (message_work:Table {id:"platform_transport.message_work",class:"Work"})
-- MERGE (active_lease_index:Config {id:"ix_platform_transport_active_lease_lookup",kind:"postgres_index"})
-- MERGE (runtime_role:Config {id:"star_platform_transport_runtime",kind:"postgres_role"})
-- MERGE (self)-[:REPLACES]->(pull)
-- MERGE (self)-[:CONFIGURES]->(active_lease_index)
-- MERGE (self)-[:CONFIGURES]->(runtime_role)
-- MERGE (pull)-[:LOCKS]->(log_master)
-- MERGE (pull)-[:READS]->(message_work)
-- MERGE (pull)-[:WRITES]->(message_work)
-- @endcypher
-- Serialize per-log claims and prevent later sequences from overtaking an active lease.
BEGIN;

-- The active-lease query shares the destination equality scope and lease-expiry range.
-- Restrict the index to leased rows so pending and terminal rows do not inflate this guard.
CREATE INDEX IF NOT EXISTS ix_platform_transport_active_lease_lookup
    ON platform_transport.message_work (
        deployment_id, tenant_id, log_id, generation, recipient_service_id, lease_expires_at
    ) WHERE state = 'leased';

CREATE OR REPLACE FUNCTION platform_transport.pull_one(
    p_log_id TEXT,
    p_generation UUID,
    p_lease_seconds INTEGER
)
RETURNS TABLE (
    delivery_log_id TEXT, delivery_generation UUID, delivery_sequence BIGINT,
    sender_service_id UUID, message_kind TEXT, message_id UUID, wire BYTEA,
    payload_sha256 BYTEA, lease_id UUID, lease_expires_at TIMESTAMPTZ, attempt_count SMALLINT
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_service UUID := NULLIF(current_setting('app.platform_service_id', true), '')::UUID;
    v_row platform_transport.message_work%ROWTYPE;
    v_max_attempts SMALLINT;
    v_lease UUID;
    v_until TIMESTAMPTZ;
BEGIN
    IF v_deployment IS NULL OR v_tenant IS NULL OR v_service IS NULL
       OR p_log_id IS NULL OR p_generation IS NULL OR p_lease_seconds IS NULL
       OR p_lease_seconds NOT BETWEEN 30 AND 120 THEN
        RAISE EXCEPTION 'invalid platform transport pull boundary' USING ERRCODE = '22023';
    END IF;

    -- Every claimant for this destination serializes on the same master row. The lock is held
    -- until the caller's transaction ends, so the following lease check sees prior claims.
    SELECT m.max_delivery_attempts INTO v_max_attempts
      FROM platform_transport.log_master m
     WHERE m.deployment_id = v_deployment AND m.tenant_id = v_tenant
       AND m.log_id = p_log_id AND m.generation = p_generation
       AND m.recipient_service_id = v_service AND m.lifecycle IN ('active', 'draining')
     FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'platform transport log is not active for this authenticated recipient' USING ERRCODE = '42501';
    END IF;

    -- Local permits disappear on cancellation or process restart. Durable source leases remain
    -- authoritative and block every later sequence until ACK or lease expiry.
    IF EXISTS (
        SELECT 1
          FROM platform_transport.message_work w
         WHERE w.deployment_id = v_deployment AND w.tenant_id = v_tenant
           AND w.log_id = p_log_id AND w.generation = p_generation
           AND w.recipient_service_id = v_service
           AND w.state = 'leased' AND w.lease_expires_at > clock_timestamp()
    ) THEN
        RETURN;
    END IF;

    SELECT w.* INTO v_row
      FROM platform_transport.message_work w
     WHERE w.deployment_id = v_deployment AND w.tenant_id = v_tenant
       AND w.log_id = p_log_id AND w.generation = p_generation
       AND w.recipient_service_id = v_service
       AND w.attempt_count < v_max_attempts
       AND (w.state = 'pending' OR (w.state = 'leased' AND w.lease_expires_at <= clock_timestamp()))
     ORDER BY w.sequence
     LIMIT 1
     FOR UPDATE SKIP LOCKED;
    IF NOT FOUND THEN
        RETURN;
    END IF;
    v_lease := gen_random_uuid();
    v_until := clock_timestamp() + make_interval(secs => p_lease_seconds);
    UPDATE platform_transport.message_work w
       SET state = 'leased', attempt_count = w.attempt_count + 1,
           lease_id = v_lease, lease_expires_at = v_until
     WHERE w.deployment_id = v_row.deployment_id AND w.tenant_id = v_row.tenant_id
       AND w.log_id = v_row.log_id AND w.generation = v_row.generation AND w.sequence = v_row.sequence
     RETURNING w.* INTO v_row;
    RETURN QUERY SELECT v_row.log_id, v_row.generation, v_row.sequence,
        v_row.sender_service_id, v_row.message_kind, v_row.message_id, v_row.wire,
        v_row.payload_sha256, v_row.lease_id, v_row.lease_expires_at, v_row.attempt_count;
END
$function$;

ALTER FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    OWNER TO star_platform_transport_owner;
REVOKE ALL ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    FROM PUBLIC, star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    TO star_platform_transport_runtime;

COMMIT;
