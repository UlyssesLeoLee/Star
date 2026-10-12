-- @cypher schema=1
-- MERGE (self:File {path:"db/tests/platform-native-transport-server-auth.sql"})
-- MERGE (identity:Table {id:"platform_transport.service_certificate_identity"})
-- MERGE (identity_audit:Table {id:"platform_transport.service_certificate_identity_audit"})
-- MERGE (binding:Table {id:"platform_transport.profile_log_binding"})
-- MERGE (binding_audit:Table {id:"platform_transport.profile_log_binding_audit"})
-- MERGE (profile:Table {id:"platform_messaging.transport_profile_revision"})
-- MERGE (append:Function {id:"platform_transport.append_authenticated"})
-- MERGE (pull:Function {id:"platform_transport.pull_authenticated"})
-- MERGE (ack:Function {id:"platform_transport.acknowledge_authenticated"})
-- MERGE (self)-[:TESTS]->(identity)
-- MERGE (self)-[:TESTS]->(identity_audit)
-- MERGE (self)-[:TESTS]->(binding)
-- MERGE (self)-[:TESTS]->(binding_audit)
-- MERGE (self)-[:TESTS]->(profile)
-- MERGE (self)-[:TESTS]->(append)
-- MERGE (self)-[:TESTS]->(pull)
-- MERGE (self)-[:TESTS]->(ack)
-- @endcypher

DO $assertions$
DECLARE
    table_name TEXT;
    routine TEXT;
BEGIN
    FOREACH table_name IN ARRAY ARRAY[
        'platform_transport.service_certificate_identity',
        'platform_transport.service_certificate_identity_audit',
        'platform_transport.profile_log_binding',
        'platform_transport.profile_log_binding_audit'
    ] LOOP
        IF NOT (SELECT c.relrowsecurity AND c.relforcerowsecurity
                  FROM pg_class c WHERE c.oid = table_name::regclass) THEN
            RAISE EXCEPTION 'native server table % must have FORCE RLS', table_name;
        END IF;
    END LOOP;
    FOREACH routine IN ARRAY ARRAY[
        'platform_transport.authorize_profile_route(bytea,bytea,text,uuid,text)',
        'platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)',
        'platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)',
        'platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)'
    ] LOOP
        IF NOT EXISTS (
            SELECT 1 FROM pg_proc p
             WHERE p.oid = routine::regprocedure
               AND p.prosecdef
               AND pg_get_userbyid(p.proowner) = 'star_platform_transport_owner'
               AND p.proconfig @> ARRAY['search_path=pg_catalog, platform_transport, platform_messaging']
               AND NOT EXISTS (
                    SELECT 1 FROM aclexplode(p.proacl) a
                     WHERE a.grantee = 0 AND a.privilege_type = 'EXECUTE'
               )
        ) THEN
            RAISE EXCEPTION 'native server routine % is not fixed-path, owner-scoped and non-public', routine;
        END IF;
    END LOOP;
    IF has_table_privilege('star_platform_transport_runtime', 'platform_transport.service_certificate_identity', 'SELECT')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.profile_log_binding', 'SELECT')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'SELECT')
       OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)', 'EXECUTE')
       OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.pull_one(text,uuid,integer)', 'EXECUTE')
       OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)', 'EXECUTE')
       OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.purge_terminal(text,uuid,integer)', 'EXECUTE')
       OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)', 'EXECUTE')
       OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)', 'EXECUTE')
       OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)', 'EXECUTE') THEN
        RAISE EXCEPTION 'native runtime must call only the mTLS/profile-authenticated procedures';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.service_certificate_identity'::regclass
          AND tgname='certificate_identity_event' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.service_certificate_identity_audit'::regclass
          AND tgname='certificate_identity_audit_worm' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.profile_log_binding'::regclass
          AND tgname='profile_log_binding_event' AND NOT tgisinternal)
       OR NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.profile_log_binding_audit'::regclass
          AND tgname='profile_log_binding_audit_worm' AND NOT tgisinternal) THEN
        RAISE EXCEPTION 'certificate and profile/log master audit guards are incomplete';
    END IF;
END
$assertions$;

BEGIN;
SELECT set_config('app.platform_deployment_id', '91000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_tenant_id', '92000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_profile_tenant_id', '92000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_actor_service_id', '93000000-0000-0000-0000-000000000003', TRUE);

SET LOCAL ROLE star_platform_profile_owner;
INSERT INTO platform_messaging.transport_profile_revision (
    deployment_id, tenant_id, profile_id, revision, expected_previous_revision,
    lifecycle_state, schema_version, provider_id, provider_version, endpoint_identity,
    log_generation_policy, allowed_sender_service_ids, allowed_recipient_service_ids,
    allowed_message_kinds, secret_reference, profile_digest, actor_service_id,
    request_id, trace_id, idempotency_key
) VALUES (
    '91000000-0000-0000-0000-000000000001', '92000000-0000-0000-0000-000000000001',
    '96000000-0000-0000-0000-000000000001', 1, NULL,
    'active', 1, 'star-rust-log', '1.0', 'native-test', 'platform-log-v1',
    ARRAY['93000000-0000-0000-0000-000000000001'::UUID],
    ARRAY['93000000-0000-0000-0000-000000000002'::UUID], ARRAY['request']::TEXT[],
    NULL, decode(repeat('ab', 32), 'hex'), '93000000-0000-0000-0000-000000000003',
    '97000000-0000-0000-0000-000000000001', '98000000-0000-0000-0000-000000000001',
    'native-server-auth-profile'
);
RESET ROLE;

SET LOCAL ROLE star_platform_transport_owner;
INSERT INTO platform_transport.log_master (
    deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id,
    message_kind, payload_budget_bytes, terminal_retention_hours, max_delivery_attempts, lifecycle
) VALUES (
    '91000000-0000-0000-0000-000000000001', '92000000-0000-0000-0000-000000000001',
    'test.native.auth', '95000000-0000-0000-0000-000000000001',
    '93000000-0000-0000-0000-000000000001', '93000000-0000-0000-0000-000000000002',
    'request', 16384, 24, 4, 'active'
);
INSERT INTO platform_transport.service_certificate_identity (
    certificate_sha256, deployment_id, tenant_id, service_id, provider_id,
    provider_version, endpoint_identity, lifecycle_state, registered_by_service_id, registration_request_id
) VALUES
    (decode(repeat('11', 32), 'hex'), '91000000-0000-0000-0000-000000000001', '92000000-0000-0000-0000-000000000001',
     '93000000-0000-0000-0000-000000000001', 'star-rust-log', '1.0', 'native-test', 'active',
     '93000000-0000-0000-0000-000000000003', '97000000-0000-0000-0000-000000000002'),
    (decode(repeat('22', 32), 'hex'), '91000000-0000-0000-0000-000000000001', '92000000-0000-0000-0000-000000000001',
     '93000000-0000-0000-0000-000000000002', 'star-rust-log', '1.0', 'native-test', 'active',
     '93000000-0000-0000-0000-000000000003', '97000000-0000-0000-0000-000000000003');
INSERT INTO platform_transport.profile_log_binding (
    deployment_id, tenant_id, profile_id, revision, profile_digest, log_generation_policy,
    sender_service_id, recipient_service_id, message_kind, log_id, generation,
    provisioned_by_service_id, request_id
) VALUES (
    '91000000-0000-0000-0000-000000000001', '92000000-0000-0000-0000-000000000001',
    '96000000-0000-0000-0000-000000000001', 1, decode(repeat('ab', 32), 'hex'), 'platform-log-v1',
    '93000000-0000-0000-0000-000000000001', '93000000-0000-0000-0000-000000000002',
    'request', 'test.native.auth', '95000000-0000-0000-0000-000000000001',
    '93000000-0000-0000-0000-000000000003', '97000000-0000-0000-0000-000000000004'
);
RESET ROLE;

SET LOCAL ROLE star_platform_transport_runtime;
DO $authorized_delivery$
DECLARE
    accepted RECORD;
    replayed RECORD;
    delivery RECORD;
    was_acked BOOLEAN;
BEGIN
    SELECT * INTO accepted FROM platform_transport.append_authenticated(
        decode(repeat('11', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000002', 'request',
        '99000000-0000-0000-0000-000000000001', convert_to('native-auth-wire', 'UTF8'),
        decode('fb9e201abd68c6afc321f4136d33c1d1afc38d3f0214ceb0a8e65d07ba68f5e5', 'hex'));
    IF accepted.accepted_sequence <> 1 OR accepted.exact_replay THEN
        RAISE EXCEPTION 'authenticated append must allocate one durable route receipt';
    END IF;
    SELECT * INTO replayed FROM platform_transport.append_authenticated(
        decode(repeat('11', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000002', 'request',
        '99000000-0000-0000-0000-000000000001', convert_to('native-auth-wire', 'UTF8'),
        decode('fb9e201abd68c6afc321f4136d33c1d1afc38d3f0214ceb0a8e65d07ba68f5e5', 'hex'));
    IF replayed.accepted_sequence <> accepted.accepted_sequence OR NOT replayed.exact_replay THEN
        RAISE EXCEPTION 'authenticated append exact replay must preserve its receipt';
    END IF;
    SELECT * INTO delivery FROM platform_transport.pull_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', 30);
    IF delivery.delivery_log_id <> 'test.native.auth'
       OR delivery.delivery_generation <> '95000000-0000-0000-0000-000000000001'
       OR delivery.wire <> convert_to('native-auth-wire', 'UTF8')
       OR delivery.attempt_count <> 1 THEN
        RAISE EXCEPTION 'authenticated pull must lease the exact profile-bound durable route';
    END IF;
    SELECT platform_transport.acknowledge_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', delivery.delivery_log_id,
        delivery.delivery_generation, delivery.delivery_sequence, delivery.lease_id,
        delivery.message_id, delivery.payload_sha256) INTO was_acked;
    IF NOT was_acked THEN RAISE EXCEPTION 'authenticated ACK must commit the lease receipt'; END IF;
    BEGIN
        PERFORM * FROM platform_transport.append_authenticated(
            decode(repeat('33', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
            '93000000-0000-0000-0000-000000000002', 'request',
            '99000000-0000-0000-0000-000000000002', convert_to('unknown-peer', 'UTF8'),
            decode(repeat('ef', 32), 'hex'));
        RAISE EXCEPTION 'unregistered certificate unexpectedly authorized append';
    EXCEPTION WHEN SQLSTATE '42501' THEN NULL;
    END;
END
$authorized_delivery$;
RESET ROLE;

SET LOCAL ROLE star_platform_transport_owner;
UPDATE platform_transport.service_certificate_identity
   SET lifecycle_state = 'revoked', revoked_by_service_id = '93000000-0000-0000-0000-000000000003',
       revocation_request_id = '97000000-0000-0000-0000-000000000005', revoked_at = transaction_timestamp()
 WHERE certificate_sha256 = decode(repeat('11', 32), 'hex');
RESET ROLE;

SET LOCAL ROLE star_platform_transport_runtime;
DO $revoked_peer$
BEGIN
    BEGIN
        PERFORM * FROM platform_transport.append_authenticated(
            decode(repeat('11', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
            '93000000-0000-0000-0000-000000000002', 'request',
            '99000000-0000-0000-0000-000000000002', convert_to('revoked-peer', 'UTF8'),
            decode(repeat('ef', 32), 'hex'));
        RAISE EXCEPTION 'revoked certificate unexpectedly authorized append';
    EXCEPTION WHEN SQLSTATE '42501' THEN NULL;
    END;
END
$revoked_peer$;
RESET ROLE;

DO $facts$
BEGIN
    IF (SELECT count(*) FROM platform_transport.delivery_receipt
         WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
           AND tenant_id = '92000000-0000-0000-0000-000000000001') <> 2
       OR (SELECT count(*) FROM platform_transport.receipt_outbox
            WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
              AND tenant_id = '92000000-0000-0000-0000-000000000001') <> 2
       OR (SELECT count(*) FROM platform_transport.service_certificate_identity_audit
            WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
              AND tenant_id = '92000000-0000-0000-0000-000000000001') <> 3
       OR (SELECT count(*) FROM platform_transport.profile_log_binding_audit
            WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
              AND tenant_id = '92000000-0000-0000-0000-000000000001') <> 1 THEN
        RAISE EXCEPTION 'native transport authorization, ACK and audit facts were not committed exactly once';
    END IF;
END
$facts$;
COMMIT;
