-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-18-platform-native-event-kind.sql"})
-- MERGE (binding:Table {id:"platform_transport.profile_log_binding",class:"Master"})
-- MERGE (authorize:Symbol {id:"platform_transport.authorize_profile_route",kind:"function"})
-- MERGE (self)-[:CONFIGURES]->(binding)
-- MERGE (self)-[:DEFINES]->(authorize)
-- MERGE (authorize)-[:READS]->(binding)
-- MERGE (authorize)-[:READS]->(identity:Table {id:"platform_transport.service_certificate_identity",class:"Master"})
-- MERGE (authorize)-[:READS]->(profile:Table {id:"platform_messaging.transport_profile_revision",class:"Master"})
-- MERGE (authorize)-[:READS]->(log:Table {id:"platform_transport.log_master",class:"Master"})
-- @endcypher
-- Requires the separate Profile event-kind migration. No existing Profile is widened.
BEGIN;
ALTER TABLE platform_transport.profile_log_binding
    DROP CONSTRAINT IF EXISTS profile_log_binding_message_kind_check,
    DROP CONSTRAINT IF EXISTS ck_native_profile_binding_message_kind_v2;
ALTER TABLE platform_transport.profile_log_binding
    ADD CONSTRAINT ck_native_profile_binding_message_kind_v2
    CHECK (message_kind IN ('request', 'result', 'resolution', 'event'));

CREATE OR REPLACE FUNCTION platform_transport.authorize_profile_route(
    p_certificate_sha256 BYTEA,
    p_profile_digest BYTEA,
    p_operation TEXT,
    p_remote_service_id UUID,
    p_message_kind TEXT
)
RETURNS TABLE (
    route_deployment_id UUID,
    route_tenant_id UUID,
    route_service_id UUID,
    route_sender_service_id UUID,
    route_recipient_service_id UUID,
    route_log_id TEXT,
    route_generation UUID
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport, platform_messaging
AS $function$
DECLARE
    v_identity platform_transport.service_certificate_identity%ROWTYPE;
    v_profile platform_messaging.transport_profile_revision%ROWTYPE;
    v_profile_id UUID;
    v_binding platform_transport.profile_log_binding%ROWTYPE;
    v_log platform_transport.log_master%ROWTYPE;
    v_sender UUID;
    v_recipient UUID;
BEGIN
    IF p_certificate_sha256 IS NULL OR octet_length(p_certificate_sha256) <> 32
       OR p_profile_digest IS NULL OR octet_length(p_profile_digest) <> 32
       OR p_operation IS NULL OR p_operation NOT IN ('append', 'pull', 'ack')
       OR p_remote_service_id IS NULL OR p_remote_service_id = '00000000-0000-0000-0000-000000000000'
       OR p_message_kind IS NULL OR p_message_kind NOT IN ('request', 'result', 'resolution', 'event') THEN
        RAISE EXCEPTION 'invalid native transport request boundary' USING ERRCODE = '22023';
    END IF;
    SELECT i.* INTO v_identity
      FROM platform_transport.service_certificate_identity i
     WHERE i.certificate_sha256 = p_certificate_sha256 AND i.lifecycle_state = 'active'
     FOR SHARE;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'native transport peer identity is not active' USING ERRCODE = '42501';
    END IF;
    PERFORM set_config('app.platform_deployment_id', v_identity.deployment_id::TEXT, true);
    PERFORM set_config('app.platform_tenant_id', v_identity.tenant_id::TEXT, true);
    PERFORM set_config('app.platform_service_id', v_identity.service_id::TEXT, true);
    PERFORM set_config('app.platform_profile_tenant_id', v_identity.tenant_id::TEXT, true);
    IF p_operation = 'append' THEN
        v_sender := v_identity.service_id;
        v_recipient := p_remote_service_id;
    ELSE
        v_sender := p_remote_service_id;
        v_recipient := v_identity.service_id;
    END IF;
    SELECT p.profile_id INTO STRICT v_profile_id
      FROM platform_messaging.transport_profile_revision p
     WHERE p.deployment_id = v_identity.deployment_id
       AND p.tenant_id = v_identity.tenant_id
       AND p.profile_digest = p_profile_digest;
    PERFORM pg_advisory_xact_lock(hashtextextended(
        v_identity.deployment_id::TEXT || ':' || v_identity.tenant_id::TEXT || ':' || v_profile_id::TEXT, 0
    ));
    SELECT p.* INTO v_profile
      FROM platform_messaging.transport_profile_revision p
     WHERE p.deployment_id = v_identity.deployment_id
       AND p.tenant_id = v_identity.tenant_id
       AND p.profile_id = v_profile_id
       AND p.profile_digest = p_profile_digest
       AND p.valid_to IS NULL
       AND p.lifecycle_state = 'active'
       AND p.provider_id = v_identity.provider_id
       AND p.provider_version = v_identity.provider_version
       AND p.endpoint_identity = v_identity.endpoint_identity
       AND v_sender = ANY(p.allowed_sender_service_ids)
       AND v_recipient = ANY(p.allowed_recipient_service_ids)
       AND p_message_kind = ANY(p.allowed_message_kinds);
    IF NOT FOUND THEN
        RAISE EXCEPTION 'native transport profile is not active for the authenticated route' USING ERRCODE = '42501';
    END IF;
    SELECT b.* INTO v_binding
      FROM platform_transport.profile_log_binding b
      JOIN platform_transport.log_master m
        ON m.deployment_id = b.deployment_id AND m.tenant_id = b.tenant_id
       AND m.log_id = b.log_id AND m.generation = b.generation
       AND m.sender_service_id = b.sender_service_id
       AND m.recipient_service_id = b.recipient_service_id
       AND m.message_kind = b.message_kind
     WHERE b.deployment_id = v_identity.deployment_id
       AND b.tenant_id = v_identity.tenant_id
       AND b.profile_id = v_profile.profile_id
       AND b.revision = v_profile.revision
       AND b.profile_digest = p_profile_digest
       AND b.log_generation_policy = v_profile.log_generation_policy
       AND b.sender_service_id = v_sender
       AND b.recipient_service_id = v_recipient
       AND b.message_kind = p_message_kind
       AND ((p_operation = 'append' AND m.lifecycle = 'active')
          OR (p_operation IN ('pull', 'ack') AND m.lifecycle IN ('active', 'draining')))
     FOR SHARE OF m;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'native transport profile has no active bound log generation for this route' USING ERRCODE = '42501';
    END IF;
    route_deployment_id := v_identity.deployment_id;
    route_tenant_id := v_identity.tenant_id;
    route_service_id := v_identity.service_id;
    route_sender_service_id := v_sender;
    route_recipient_service_id := v_recipient;
    route_log_id := v_binding.log_id;
    route_generation := v_binding.generation;
    RETURN NEXT;
END
$function$;

ALTER FUNCTION platform_transport.authorize_profile_route(BYTEA, BYTEA, TEXT, UUID, TEXT)
    OWNER TO star_platform_transport_owner;
REVOKE ALL ON FUNCTION platform_transport.authorize_profile_route(BYTEA, BYTEA, TEXT, UUID, TEXT)
    FROM PUBLIC, star_platform_transport_runtime;
COMMIT;
