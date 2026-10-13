-- Native transport source provenance is fixed at authenticated append and checked before owner ACK.
-- @cypher schema=1 source_sha256=71cc6434b88b425337b37e1c8015f78afdb76fdd356562a7296b6c39b69b67e9
-- MERGE (self:File {path:"db/migrations/2026-10-12-platform-native-origin-verification.sql"})
-- MERGE (work:Table {id:"platform_transport.message_work",class:"Work"})
-- MERGE (certificate:Table {id:"platform_transport.service_certificate_identity",class:"Master"})
-- MERGE (capture:Function {id:"platform_transport.capture_message_source_certificate"})
-- MERGE (verify:Function {id:"platform_transport.verify_origin_authenticated"})
-- MERGE (append:Function {id:"platform_transport.append_authenticated"})
-- MERGE (ack:Function {id:"platform_transport.acknowledge_authenticated"})
-- MERGE (self)-[:ALTERS]->(work)
-- MERGE (self)-[:READS]->(certificate)
-- MERGE (self)-[:DEFINES]->(capture)
-- MERGE (self)-[:DEFINES]->(verify)
-- MERGE (self)-[:REVISES]->(append)
-- MERGE (self)-[:REVISES]->(ack)
-- MERGE (owner_role:Config {id:"star_platform_transport_owner"})
-- MERGE (runtime_role:Config {id:"star_platform_transport_runtime"})
-- MERGE (self)-[:CONFIGURES]->(owner_role)
-- MERGE (self)-[:CONFIGURES]->(runtime_role)
-- MERGE (capture)-[:READS]->(certificate)
-- MERGE (capture)-[:WRITES]->(work)
-- MERGE (append)-[:CALLS]->(capture)
-- MERGE (verify)-[:READS]->(work)
-- MERGE (verify)-[:READS]->(certificate)
-- MERGE (ack)-[:CALLS]->(verify)
-- @endcypher

BEGIN;

DO $role_guard$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_transport_owner')
       OR NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'star_platform_transport_runtime')
       OR EXISTS (
            SELECT 1 FROM pg_roles
             WHERE rolname IN ('star_platform_transport_owner', 'star_platform_transport_runtime')
               AND (rolcanlogin OR rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication
                    OR rolbypassrls OR rolinherit)
       ) THEN
        RAISE EXCEPTION 'native transport owner/runtime roles must exist as restricted NOLOGIN NOBYPASSRLS NOINHERIT roles'
            USING ERRCODE = '42501';
    END IF;
END
$role_guard$;

ALTER TABLE platform_transport.message_work
    ADD COLUMN IF NOT EXISTS source_certificate_sha256 BYTEA;
DO $constraint$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conrelid = 'platform_transport.message_work'::regclass
           AND conname = 'message_work_source_certificate_sha256_width'
    ) THEN
        ALTER TABLE platform_transport.message_work
            ADD CONSTRAINT message_work_source_certificate_sha256_width
            CHECK (source_certificate_sha256 IS NULL OR octet_length(source_certificate_sha256) = 32);
    END IF;
END
$constraint$;
COMMENT ON COLUMN platform_transport.message_work.source_certificate_sha256 IS
    'Authenticated sender certificate fingerprint captured at append; NULL on pre-attestation rows, which fail closed for native receive.';

CREATE OR REPLACE FUNCTION platform_transport.capture_message_source_certificate()
RETURNS TRIGGER
LANGUAGE plpgsql
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_certificate BYTEA;
    v_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_sender UUID := NULLIF(current_setting('app.platform_service_id', true), '')::UUID;
BEGIN
    IF TG_OP = 'INSERT' THEN
        v_certificate := decode(NULLIF(current_setting('app.platform_source_certificate_sha256', true), ''), 'hex');
        IF v_certificate IS NULL OR octet_length(v_certificate) <> 32
           OR NEW.deployment_id IS DISTINCT FROM v_deployment
           OR NEW.tenant_id IS DISTINCT FROM v_tenant
           OR NEW.sender_service_id IS DISTINCT FROM v_sender
           OR NOT EXISTS (
                SELECT 1 FROM platform_transport.service_certificate_identity i
                 WHERE i.certificate_sha256 = v_certificate
                   AND i.deployment_id = v_deployment
                   AND i.tenant_id = v_tenant
                   AND i.service_id = v_sender
                   AND i.lifecycle_state = 'active'
           ) THEN
            RAISE EXCEPTION 'native source credential must come from the active authenticated append route'
                USING ERRCODE = '42501';
        END IF;
        NEW.source_certificate_sha256 := v_certificate;
        RETURN NEW;
    END IF;

    IF NEW.source_certificate_sha256 IS DISTINCT FROM OLD.source_certificate_sha256 THEN
        RAISE EXCEPTION 'native source credential evidence is immutable'
            USING ERRCODE = '55000';
    END IF;
    RETURN NEW;
END
$function$;

DROP TRIGGER IF EXISTS message_source_certificate_guard ON platform_transport.message_work;
CREATE TRIGGER message_source_certificate_guard
    BEFORE INSERT OR UPDATE OF source_certificate_sha256 ON platform_transport.message_work
    FOR EACH ROW EXECUTE FUNCTION platform_transport.capture_message_source_certificate();

CREATE OR REPLACE FUNCTION platform_transport.append_authenticated(
    p_certificate_sha256 BYTEA, p_profile_digest BYTEA, p_recipient_service_id UUID,
    p_message_kind TEXT, p_message_id UUID, p_wire BYTEA, p_payload_sha256 BYTEA
)
RETURNS TABLE (accepted_log_id TEXT, accepted_generation UUID, accepted_sequence BIGINT, accepted_sha256 BYTEA, exact_replay BOOLEAN)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog, platform_transport, platform_messaging
AS $function$
DECLARE v_route RECORD;
BEGIN
    SELECT r.* INTO STRICT v_route
      FROM platform_transport.authorize_profile_route(
          p_certificate_sha256, p_profile_digest, 'append', p_recipient_service_id, p_message_kind
      ) r;
    PERFORM set_config('app.platform_source_certificate_sha256', encode(p_certificate_sha256, 'hex'), true);
    RETURN QUERY SELECT a.accepted_log_id, a.accepted_generation, a.accepted_sequence, a.accepted_sha256, a.exact_replay
      FROM platform_transport.append_message(
          v_route.route_log_id, v_route.route_generation, p_recipient_service_id,
          p_message_kind, p_message_id, p_wire, p_payload_sha256
      ) a;
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.verify_origin_authenticated(
    p_certificate_sha256 BYTEA, p_profile_digest BYTEA, p_sender_service_id UUID,
    p_message_kind TEXT, p_log_id TEXT, p_generation UUID, p_sequence BIGINT,
    p_message_id UUID, p_payload_sha256 BYTEA
)
RETURNS BOOLEAN
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog, platform_transport, platform_messaging
AS $function$
DECLARE
    v_route RECORD;
    v_authorized BOOLEAN := FALSE;
BEGIN
    IF p_log_id IS NULL OR p_generation IS NULL OR p_sequence IS NULL OR p_sequence <= 0
       OR p_message_id IS NULL OR p_message_id = '00000000-0000-0000-0000-000000000000'
       OR p_payload_sha256 IS NULL OR octet_length(p_payload_sha256) <> 32 THEN
        RAISE EXCEPTION 'invalid native origin verification boundary' USING ERRCODE = '22023';
    END IF;
    SELECT r.* INTO STRICT v_route
      FROM platform_transport.authorize_profile_route(
          p_certificate_sha256, p_profile_digest, 'pull', p_sender_service_id, p_message_kind
      ) r;
    IF p_log_id IS DISTINCT FROM v_route.route_log_id
       OR p_generation IS DISTINCT FROM v_route.route_generation THEN
        RETURN FALSE;
    END IF;

    SELECT TRUE INTO v_authorized
      FROM platform_transport.message_work w
      JOIN platform_transport.service_certificate_identity i
        ON i.certificate_sha256 = w.source_certificate_sha256
       AND i.deployment_id = w.deployment_id
       AND i.tenant_id = w.tenant_id
       AND i.service_id = w.sender_service_id
     WHERE w.deployment_id = v_route.route_deployment_id
       AND w.tenant_id = v_route.route_tenant_id
       AND w.log_id = v_route.route_log_id
       AND w.generation = v_route.route_generation
       AND w.sequence = p_sequence
       AND w.sender_service_id = v_route.route_sender_service_id
       AND w.recipient_service_id = v_route.route_recipient_service_id
       AND w.message_kind = p_message_kind
       AND w.message_id = p_message_id
       AND w.payload_sha256 = p_payload_sha256
       AND w.source_certificate_sha256 IS NOT NULL
       AND i.registered_at <= w.accepted_at
       AND (i.revoked_at IS NULL OR i.revoked_at > w.accepted_at)
       AND w.state = 'leased'
       AND w.lease_expires_at > clock_timestamp()
     LIMIT 1
     FOR SHARE OF w, i;
    RETURN COALESCE(v_authorized, FALSE);
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.acknowledge_authenticated(
    p_certificate_sha256 BYTEA, p_profile_digest BYTEA, p_sender_service_id UUID,
    p_message_kind TEXT, p_log_id TEXT, p_generation UUID, p_sequence BIGINT,
    p_lease_id UUID, p_message_id UUID, p_payload_sha256 BYTEA
)
RETURNS BOOLEAN
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog, platform_transport, platform_messaging
AS $function$
DECLARE v_route RECORD;
BEGIN
    SELECT r.* INTO STRICT v_route
      FROM platform_transport.authorize_profile_route(
          p_certificate_sha256, p_profile_digest, 'ack', p_sender_service_id, p_message_kind
      ) r;
    IF p_log_id IS DISTINCT FROM v_route.route_log_id OR p_generation IS DISTINCT FROM v_route.route_generation THEN
        RAISE EXCEPTION 'native transport acknowledgement route does not match the authenticated profile' USING ERRCODE = '42501';
    END IF;
    IF NOT platform_transport.verify_origin_authenticated(
        p_certificate_sha256, p_profile_digest, p_sender_service_id, p_message_kind,
        p_log_id, p_generation, p_sequence, p_message_id, p_payload_sha256
    ) THEN
        RAISE EXCEPTION 'native transport source origin is not authorized for this exact live delivery'
            USING ERRCODE = '42501';
    END IF;
    RETURN platform_transport.acknowledge_one(
        v_route.route_log_id, v_route.route_generation, p_sequence, p_lease_id,
        p_message_id, p_payload_sha256
    );
END
$function$;

ALTER FUNCTION platform_transport.capture_message_source_certificate()
    OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.append_authenticated(BYTEA, BYTEA, UUID, TEXT, UUID, BYTEA, BYTEA)
    OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.verify_origin_authenticated(BYTEA, BYTEA, UUID, TEXT, TEXT, UUID, BIGINT, UUID, BYTEA)
    OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.acknowledge_authenticated(BYTEA, BYTEA, UUID, TEXT, TEXT, UUID, BIGINT, UUID, UUID, BYTEA)
    OWNER TO star_platform_transport_owner;

REVOKE ALL ON FUNCTION platform_transport.capture_message_source_certificate()
    FROM PUBLIC, star_platform_transport_runtime;
REVOKE ALL ON FUNCTION platform_transport.verify_origin_authenticated(BYTEA, BYTEA, UUID, TEXT, TEXT, UUID, BIGINT, UUID, BYTEA)
    FROM PUBLIC, star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.verify_origin_authenticated(BYTEA, BYTEA, UUID, TEXT, TEXT, UUID, BIGINT, UUID, BYTEA)
    TO star_platform_transport_runtime;

COMMIT;
