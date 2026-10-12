-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-12-platform-transport-z-native-server-auth.sql"})
-- MERGE (identity:Table {id:"platform_transport.service_certificate_identity",class:"Master"})
-- MERGE (identity_audit:Table {id:"platform_transport.service_certificate_identity_audit",class:"Transaction"})
-- MERGE (binding:Table {id:"platform_transport.profile_log_binding",class:"Master"})
-- MERGE (binding_audit:Table {id:"platform_transport.profile_log_binding_audit",class:"Transaction"})
-- MERGE (authorize:Function {id:"platform_transport.authorize_profile_route"})
-- MERGE (append:Function {id:"platform_transport.append_authenticated"})
-- MERGE (pull:Function {id:"platform_transport.pull_authenticated"})
-- MERGE (ack:Function {id:"platform_transport.acknowledge_authenticated"})
-- MERGE (append_base:Function {id:"platform_transport.append_message"})
-- MERGE (pull_base:Function {id:"platform_transport.pull_one"})
-- MERGE (ack_base:Function {id:"platform_transport.acknowledge_one"})
-- MERGE (profile:Table {id:"platform_messaging.transport_profile_revision",class:"Master"})
-- MERGE (log:Table {id:"platform_transport.log_master",class:"Master"})
-- MERGE (self)-[:CONFIGURES]->(identity)
-- MERGE (self)-[:CONFIGURES]->(identity_audit)
-- MERGE (self)-[:CONFIGURES]->(binding)
-- MERGE (self)-[:CONFIGURES]->(binding_audit)
-- MERGE (self)-[:EXPOSES]->(append)
-- MERGE (self)-[:EXPOSES]->(pull)
-- MERGE (self)-[:EXPOSES]->(ack)
-- MERGE (authorize)-[:READS]->(identity)
-- MERGE (authorize)-[:READS]->(profile)
-- MERGE (authorize)-[:READS]->(binding)
-- MERGE (authorize)-[:READS]->(log)
-- MERGE (append)-[:CALLS]->(authorize)
-- MERGE (append)-[:CALLS]->(append_base)
-- MERGE (pull)-[:CALLS]->(authorize)
-- MERGE (pull)-[:CALLS]->(pull_base)
-- MERGE (ack)-[:CALLS]->(authorize)
-- MERGE (ack)-[:CALLS]->(ack_base)
-- @endcypher
-- Certificate fingerprints identify principals globally; only SECURITY DEFINER route functions expose
-- them to runtime callers. Profile, route, and exact log generation are revalidated per operation.
BEGIN;

CREATE TABLE IF NOT EXISTS platform_transport.service_certificate_identity (
    certificate_sha256 BYTEA PRIMARY KEY CHECK (octet_length(certificate_sha256) = 32),
    deployment_id UUID NOT NULL CHECK (deployment_id <> '00000000-0000-0000-0000-000000000000'),
    tenant_id UUID NOT NULL CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'),
    service_id UUID NOT NULL CHECK (service_id <> '00000000-0000-0000-0000-000000000000'),
    provider_id TEXT NOT NULL CHECK (provider_id = 'star-rust-log'),
    provider_version TEXT NOT NULL CHECK (provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'),
    endpoint_identity TEXT NOT NULL CHECK (endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'),
    lifecycle_state TEXT NOT NULL CHECK (lifecycle_state IN ('active', 'revoked')),
    registered_by_service_id UUID NOT NULL CHECK (registered_by_service_id <> '00000000-0000-0000-0000-000000000000'),
    registration_request_id UUID NOT NULL CHECK (registration_request_id <> '00000000-0000-0000-0000-000000000000'),
    registered_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    revoked_by_service_id UUID,
    revocation_request_id UUID,
    revoked_at TIMESTAMPTZ,
    CHECK (
        (lifecycle_state = 'active' AND revoked_by_service_id IS NULL AND revocation_request_id IS NULL AND revoked_at IS NULL)
        OR
        (lifecycle_state = 'revoked' AND revoked_by_service_id IS NOT NULL
            AND revocation_request_id IS NOT NULL AND revoked_at IS NOT NULL)
    ),
    CHECK (revoked_by_service_id IS NULL OR revoked_by_service_id <> '00000000-0000-0000-0000-000000000000'),
    CHECK (revocation_request_id IS NULL OR revocation_request_id <> '00000000-0000-0000-0000-000000000000')
);

CREATE TABLE IF NOT EXISTS platform_transport.service_certificate_identity_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certificate_sha256 BYTEA NOT NULL CHECK (octet_length(certificate_sha256) = 32),
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    service_id UUID NOT NULL,
    provider_id TEXT NOT NULL,
    provider_version TEXT NOT NULL,
    endpoint_identity TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('registered', 'revoked')),
    actor_service_id UUID NOT NULL,
    request_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    UNIQUE (deployment_id, tenant_id, certificate_sha256, event_type)
);

CREATE TABLE IF NOT EXISTS platform_transport.profile_log_binding (
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    log_generation_policy TEXT NOT NULL CHECK (log_generation_policy ~ '^[a-z][a-z0-9._-]{0,63}$'),
    sender_service_id UUID NOT NULL CHECK (sender_service_id <> '00000000-0000-0000-0000-000000000000'),
    recipient_service_id UUID NOT NULL CHECK (recipient_service_id <> '00000000-0000-0000-0000-000000000000'),
    message_kind TEXT NOT NULL CHECK (message_kind IN ('request', 'result', 'resolution')),
    log_id TEXT NOT NULL,
    generation UUID NOT NULL CHECK (generation <> '00000000-0000-0000-0000-000000000000'),
    provisioned_by_service_id UUID NOT NULL CHECK (provisioned_by_service_id <> '00000000-0000-0000-0000-000000000000'),
    request_id UUID NOT NULL CHECK (request_id <> '00000000-0000-0000-0000-000000000000'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    PRIMARY KEY (deployment_id, tenant_id, profile_id, revision, sender_service_id, recipient_service_id, message_kind),
    UNIQUE (deployment_id, tenant_id, profile_digest, sender_service_id, recipient_service_id, message_kind),
    FOREIGN KEY (deployment_id, tenant_id, profile_id, revision)
        REFERENCES platform_messaging.transport_profile_revision (deployment_id, tenant_id, profile_id, revision),
    FOREIGN KEY (deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id, message_kind)
        REFERENCES platform_transport.log_master
            (deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id, message_kind),
    CHECK (sender_service_id <> recipient_service_id)
);
CREATE INDEX IF NOT EXISTS ix_platform_transport_profile_log_binding_lookup
    ON platform_transport.profile_log_binding
       (deployment_id, tenant_id, profile_digest, sender_service_id, recipient_service_id, message_kind);

CREATE TABLE IF NOT EXISTS platform_transport.profile_log_binding_audit (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    profile_id UUID NOT NULL,
    revision BIGINT NOT NULL,
    profile_digest BYTEA NOT NULL CHECK (octet_length(profile_digest) = 32),
    sender_service_id UUID NOT NULL,
    recipient_service_id UUID NOT NULL,
    message_kind TEXT NOT NULL,
    log_id TEXT NOT NULL,
    generation UUID NOT NULL,
    actor_service_id UUID NOT NULL,
    request_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(),
    UNIQUE (deployment_id, tenant_id, profile_digest, sender_service_id, recipient_service_id, message_kind)
);

CREATE OR REPLACE FUNCTION platform_transport.record_certificate_identity_event()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport
AS $function$
DECLARE
    v_expected_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_expected_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_actor UUID := NULLIF(current_setting('app.platform_actor_service_id', true), '')::UUID;
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.lifecycle_state <> 'active' OR NEW.registered_at IS DISTINCT FROM transaction_timestamp()
           OR NEW.deployment_id IS DISTINCT FROM v_expected_deployment
           OR NEW.tenant_id IS DISTINCT FROM v_expected_tenant
           OR NEW.registered_by_service_id IS DISTINCT FROM v_actor THEN
            RAISE EXCEPTION 'certificate identity registration must use its scoped owner context';
        END IF;
        INSERT INTO platform_transport.service_certificate_identity_audit (
            certificate_sha256, deployment_id, tenant_id, service_id, provider_id,
            provider_version, endpoint_identity, event_type, actor_service_id, request_id
        ) VALUES (
            NEW.certificate_sha256, NEW.deployment_id, NEW.tenant_id, NEW.service_id,
            NEW.provider_id, NEW.provider_version, NEW.endpoint_identity, 'registered',
            NEW.registered_by_service_id, NEW.registration_request_id
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.lifecycle_state <> 'active' OR NEW.lifecycle_state <> 'revoked'
           OR (to_jsonb(NEW) - ARRAY['lifecycle_state','revoked_by_service_id','revocation_request_id','revoked_at'])
                IS DISTINCT FROM (to_jsonb(OLD) - ARRAY['lifecycle_state','revoked_by_service_id','revocation_request_id','revoked_at'])
           OR NEW.revoked_at IS DISTINCT FROM transaction_timestamp()
           OR NEW.deployment_id IS DISTINCT FROM v_expected_deployment
           OR NEW.tenant_id IS DISTINCT FROM v_expected_tenant
           OR NEW.revoked_by_service_id IS DISTINCT FROM v_actor THEN
            RAISE EXCEPTION 'certificate identities may only be revoked once by their scoped owner';
        END IF;
        INSERT INTO platform_transport.service_certificate_identity_audit (
            certificate_sha256, deployment_id, tenant_id, service_id, provider_id,
            provider_version, endpoint_identity, event_type, actor_service_id, request_id
        ) VALUES (
            NEW.certificate_sha256, NEW.deployment_id, NEW.tenant_id, NEW.service_id,
            NEW.provider_id, NEW.provider_version, NEW.endpoint_identity, 'revoked',
            NEW.revoked_by_service_id, NEW.revocation_request_id
        );
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'certificate identity rows cannot be deleted';
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.record_profile_log_binding()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, platform_transport, platform_messaging
AS $function$
DECLARE
    v_profile platform_messaging.transport_profile_revision%ROWTYPE;
    v_expected_deployment UUID := NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID;
    v_expected_tenant UUID := NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID;
    v_profile_tenant UUID := NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID;
    v_actor UUID := NULLIF(current_setting('app.platform_actor_service_id', true), '')::UUID;
BEGIN
    IF TG_OP <> 'INSERT' THEN
        RAISE EXCEPTION 'profile log bindings are immutable';
    END IF;
    IF NEW.deployment_id IS DISTINCT FROM v_expected_deployment
       OR NEW.tenant_id IS DISTINCT FROM v_expected_tenant
       OR NEW.tenant_id IS DISTINCT FROM v_profile_tenant
       OR NEW.provisioned_by_service_id IS DISTINCT FROM v_actor
       OR NEW.created_at IS DISTINCT FROM transaction_timestamp() THEN
        RAISE EXCEPTION 'profile log binding must use its scoped owner context';
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended(
        NEW.deployment_id::TEXT || ':' || NEW.tenant_id::TEXT || ':' || NEW.profile_id::TEXT, 0
    ));
    SELECT p.* INTO v_profile
      FROM platform_messaging.transport_profile_revision p
     WHERE p.deployment_id = NEW.deployment_id
       AND p.tenant_id = NEW.tenant_id
       AND p.profile_id = NEW.profile_id
       AND p.revision = NEW.revision
       AND p.valid_to IS NULL
       AND p.lifecycle_state = 'active'
       AND p.profile_digest = NEW.profile_digest
       AND p.provider_id = 'star-rust-log'
       AND p.log_generation_policy = NEW.log_generation_policy
       AND NEW.sender_service_id = ANY(p.allowed_sender_service_ids)
       AND NEW.recipient_service_id = ANY(p.allowed_recipient_service_ids)
       AND NEW.message_kind = ANY(p.allowed_message_kinds);
    IF NOT FOUND THEN
        RAISE EXCEPTION 'profile log binding does not match a current active profile route';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM platform_transport.log_master m
         WHERE m.deployment_id = NEW.deployment_id AND m.tenant_id = NEW.tenant_id
           AND m.log_id = NEW.log_id AND m.generation = NEW.generation
           AND m.sender_service_id = NEW.sender_service_id
           AND m.recipient_service_id = NEW.recipient_service_id
           AND m.message_kind = NEW.message_kind AND m.lifecycle = 'active'
    ) THEN
        RAISE EXCEPTION 'profile log binding must target the exact active route generation';
    END IF;
    INSERT INTO platform_transport.profile_log_binding_audit (
        deployment_id, tenant_id, profile_id, revision, profile_digest,
        sender_service_id, recipient_service_id, message_kind, log_id, generation,
        actor_service_id, request_id
    ) VALUES (
        NEW.deployment_id, NEW.tenant_id, NEW.profile_id, NEW.revision, NEW.profile_digest,
        NEW.sender_service_id, NEW.recipient_service_id, NEW.message_kind, NEW.log_id,
        NEW.generation, NEW.provisioned_by_service_id, NEW.request_id
    );
    RETURN NEW;
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.reject_server_master_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
AS $function$
BEGIN
    RAISE EXCEPTION 'append-only server master facts cannot be mutated or truncated';
END
$function$;

DROP TRIGGER IF EXISTS certificate_identity_event ON platform_transport.service_certificate_identity;
CREATE TRIGGER certificate_identity_event
    BEFORE INSERT OR UPDATE OR DELETE ON platform_transport.service_certificate_identity
    FOR EACH ROW EXECUTE FUNCTION platform_transport.record_certificate_identity_event();
DROP TRIGGER IF EXISTS certificate_identity_no_truncate ON platform_transport.service_certificate_identity;
CREATE TRIGGER certificate_identity_no_truncate
    BEFORE TRUNCATE ON platform_transport.service_certificate_identity
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_server_master_mutation();
DROP TRIGGER IF EXISTS profile_log_binding_event ON platform_transport.profile_log_binding;
CREATE TRIGGER profile_log_binding_event
    BEFORE INSERT OR UPDATE OR DELETE ON platform_transport.profile_log_binding
    FOR EACH ROW EXECUTE FUNCTION platform_transport.record_profile_log_binding();
DROP TRIGGER IF EXISTS profile_log_binding_no_truncate ON platform_transport.profile_log_binding;
CREATE TRIGGER profile_log_binding_no_truncate
    BEFORE TRUNCATE ON platform_transport.profile_log_binding
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_server_master_mutation();
DROP TRIGGER IF EXISTS certificate_identity_audit_worm ON platform_transport.service_certificate_identity_audit;
CREATE TRIGGER certificate_identity_audit_worm
    BEFORE UPDATE OR DELETE ON platform_transport.service_certificate_identity_audit
    FOR EACH ROW EXECUTE FUNCTION platform_transport.reject_server_master_mutation();
DROP TRIGGER IF EXISTS certificate_identity_audit_no_truncate ON platform_transport.service_certificate_identity_audit;
CREATE TRIGGER certificate_identity_audit_no_truncate
    BEFORE TRUNCATE ON platform_transport.service_certificate_identity_audit
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_server_master_mutation();
DROP TRIGGER IF EXISTS profile_log_binding_audit_worm ON platform_transport.profile_log_binding_audit;
CREATE TRIGGER profile_log_binding_audit_worm
    BEFORE UPDATE OR DELETE ON platform_transport.profile_log_binding_audit
    FOR EACH ROW EXECUTE FUNCTION platform_transport.reject_server_master_mutation();
DROP TRIGGER IF EXISTS profile_log_binding_audit_no_truncate ON platform_transport.profile_log_binding_audit;
CREATE TRIGGER profile_log_binding_audit_no_truncate
    BEFORE TRUNCATE ON platform_transport.profile_log_binding_audit
    FOR EACH STATEMENT EXECUTE FUNCTION platform_transport.reject_server_master_mutation();

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
       OR p_message_kind IS NULL OR p_message_kind NOT IN ('request', 'result', 'resolution') THEN
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
    RETURN QUERY SELECT a.accepted_log_id, a.accepted_generation, a.accepted_sequence, a.accepted_sha256, a.exact_replay
      FROM platform_transport.append_message(
          v_route.route_log_id, v_route.route_generation, p_recipient_service_id,
          p_message_kind, p_message_id, p_wire, p_payload_sha256
      ) a;
END
$function$;

CREATE OR REPLACE FUNCTION platform_transport.pull_authenticated(
    p_certificate_sha256 BYTEA, p_profile_digest BYTEA, p_sender_service_id UUID,
    p_message_kind TEXT, p_lease_seconds INTEGER
)
RETURNS TABLE (
    delivery_log_id TEXT, delivery_generation UUID, delivery_sequence BIGINT,
    sender_service_id UUID, message_kind TEXT, message_id UUID, wire BYTEA,
    payload_sha256 BYTEA, lease_id UUID, lease_expires_at TIMESTAMPTZ, attempt_count SMALLINT
)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog, platform_transport, platform_messaging
AS $function$
DECLARE v_route RECORD;
BEGIN
    SELECT r.* INTO STRICT v_route
      FROM platform_transport.authorize_profile_route(
          p_certificate_sha256, p_profile_digest, 'pull', p_sender_service_id, p_message_kind
      ) r;
    RETURN QUERY SELECT p.delivery_log_id, p.delivery_generation, p.delivery_sequence,
        p.sender_service_id, p.message_kind, p.message_id, p.wire, p.payload_sha256,
        p.lease_id, p.lease_expires_at, p.attempt_count
      FROM platform_transport.pull_one(v_route.route_log_id, v_route.route_generation, p_lease_seconds) p;
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
    RETURN platform_transport.acknowledge_one(
        v_route.route_log_id, v_route.route_generation, p_sequence, p_lease_id,
        p_message_id, p_payload_sha256
    );
END
$function$;

ALTER TABLE platform_transport.service_certificate_identity ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.service_certificate_identity FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS certificate_identity_owner_scope ON platform_transport.service_certificate_identity;
CREATE POLICY certificate_identity_owner_scope ON platform_transport.service_certificate_identity
    TO star_platform_transport_owner USING (true)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
ALTER TABLE platform_transport.service_certificate_identity_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.service_certificate_identity_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS certificate_identity_audit_scope ON platform_transport.service_certificate_identity_audit;
CREATE POLICY certificate_identity_audit_scope ON platform_transport.service_certificate_identity_audit
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
ALTER TABLE platform_transport.profile_log_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.profile_log_binding FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS profile_log_binding_scope ON platform_transport.profile_log_binding;
CREATE POLICY profile_log_binding_scope ON platform_transport.profile_log_binding
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);
ALTER TABLE platform_transport.profile_log_binding_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE platform_transport.profile_log_binding_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS profile_log_binding_audit_scope ON platform_transport.profile_log_binding_audit;
CREATE POLICY profile_log_binding_audit_scope ON platform_transport.profile_log_binding_audit
    TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID)
    WITH CHECK (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_tenant_id', true), '')::UUID);

ALTER TABLE platform_transport.service_certificate_identity OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.service_certificate_identity_audit OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.profile_log_binding OWNER TO star_platform_transport_owner;
ALTER TABLE platform_transport.profile_log_binding_audit OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.record_certificate_identity_event() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.record_profile_log_binding() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.reject_server_master_mutation() OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.authorize_profile_route(BYTEA, BYTEA, TEXT, UUID, TEXT) OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.append_authenticated(BYTEA, BYTEA, UUID, TEXT, UUID, BYTEA, BYTEA) OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.pull_authenticated(BYTEA, BYTEA, UUID, TEXT, INTEGER) OWNER TO star_platform_transport_owner;
ALTER FUNCTION platform_transport.acknowledge_authenticated(BYTEA, BYTEA, UUID, TEXT, TEXT, UUID, BIGINT, UUID, UUID, BYTEA) OWNER TO star_platform_transport_owner;

GRANT SELECT, INSERT, UPDATE ON platform_transport.service_certificate_identity TO star_platform_transport_owner;
GRANT SELECT, INSERT ON platform_transport.service_certificate_identity_audit TO star_platform_transport_owner;
GRANT SELECT, INSERT ON platform_transport.profile_log_binding TO star_platform_transport_owner;
GRANT SELECT, INSERT ON platform_transport.profile_log_binding_audit TO star_platform_transport_owner;
GRANT USAGE ON SCHEMA platform_messaging TO star_platform_transport_owner;
GRANT SELECT ON platform_messaging.transport_profile_revision TO star_platform_transport_owner;
DROP POLICY IF EXISTS transport_profile_read_scope ON platform_messaging.transport_profile_revision;
CREATE POLICY transport_profile_read_scope ON platform_messaging.transport_profile_revision
    AS PERMISSIVE FOR SELECT TO star_platform_transport_owner
    USING (deployment_id = NULLIF(current_setting('app.platform_deployment_id', true), '')::UUID
        AND tenant_id = NULLIF(current_setting('app.platform_profile_tenant_id', true), '')::UUID);

REVOKE ALL ON ALL TABLES IN SCHEMA platform_transport FROM PUBLIC, star_platform_transport_runtime;
REVOKE ALL ON ALL FUNCTIONS IN SCHEMA platform_transport FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION platform_transport.append_message(TEXT, UUID, UUID, TEXT, UUID, BYTEA, BYTEA)
    FROM star_platform_transport_runtime;
REVOKE EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    FROM star_platform_transport_runtime;
REVOKE EXECUTE ON FUNCTION platform_transport.acknowledge_one(TEXT, UUID, BIGINT, UUID, UUID, BYTEA)
    FROM star_platform_transport_runtime;
REVOKE EXECUTE ON FUNCTION platform_transport.purge_terminal(TEXT, UUID, INTEGER)
    FROM star_platform_transport_runtime;
REVOKE ALL ON FUNCTION platform_transport.authorize_profile_route(BYTEA, BYTEA, TEXT, UUID, TEXT)
    FROM PUBLIC, star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.append_authenticated(BYTEA, BYTEA, UUID, TEXT, UUID, BYTEA, BYTEA)
    TO star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.pull_authenticated(BYTEA, BYTEA, UUID, TEXT, INTEGER)
    TO star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.acknowledge_authenticated(BYTEA, BYTEA, UUID, TEXT, TEXT, UUID, BIGINT, UUID, UUID, BYTEA)
    TO star_platform_transport_runtime;

COMMENT ON TABLE platform_transport.service_certificate_identity IS
    'M: globally unique leaf-certificate SHA-256 registry; runtime sees only authenticated operation routines.';
COMMENT ON TABLE platform_transport.profile_log_binding IS
    'M: immutable exact profile digest and route to one durable log generation; new profile revisions require new bindings.';
COMMENT ON TABLE platform_transport.service_certificate_identity_audit IS
    'T: append-only certificate registration and revocation facts; contains fingerprints, never certificates or keys.';
COMMENT ON TABLE platform_transport.profile_log_binding_audit IS
    'T: append-only route provisioning facts linking a profile digest to an exact log generation.';

COMMIT;
