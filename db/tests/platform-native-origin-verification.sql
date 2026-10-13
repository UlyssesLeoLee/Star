-- @cypher schema=1 source_sha256=14a66b4c0ec9e97f780b68dcbec9e145e48e4432055fb74798ad0f2af03b6834
-- MERGE (self:File {path:"db/tests/platform-native-origin-verification.sql"})
-- MERGE (work:Table {id:"platform_transport.message_work",class:"Work"})
-- MERGE (certificate:Table {id:"platform_transport.service_certificate_identity",class:"Master"})
-- MERGE (verify:Function {id:"platform_transport.verify_origin_authenticated"})
-- MERGE (append:Function {id:"platform_transport.append_authenticated"})
-- MERGE (ack:Function {id:"platform_transport.acknowledge_authenticated"})
-- MERGE (owner_role:Config {id:"star_platform_transport_owner"})
-- MERGE (runtime_role:Config {id:"star_platform_transport_runtime"})
-- MERGE (self)-[:TESTS]->(owner_role)
-- MERGE (self)-[:TESTS]->(runtime_role)
-- MERGE (self)-[:TESTS]->(work)
-- MERGE (self)-[:TESTS]->(certificate)
-- MERGE (self)-[:TESTS]->(verify)
-- MERGE (self)-[:TESTS]->(append)
-- MERGE (self)-[:TESTS]->(ack)
-- @endcypher

DO $assertions$
BEGIN
    IF (SELECT count(*) FROM pg_roles
         WHERE rolname IN ('star_platform_transport_owner', 'star_platform_transport_runtime')) <> 2
       OR EXISTS (
            SELECT 1 FROM pg_roles
             WHERE rolname IN ('star_platform_transport_owner', 'star_platform_transport_runtime')
               AND (rolcanlogin OR rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication
                    OR rolbypassrls OR rolinherit)
       ) THEN
        RAISE EXCEPTION 'native transport owner/runtime roles must be restricted NOLOGIN NOBYPASSRLS NOINHERIT roles';
    END IF;
    IF NOT (SELECT c.relrowsecurity AND c.relforcerowsecurity
              FROM pg_class c WHERE c.oid = 'platform_transport.message_work'::regclass)
       OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.verify_origin_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,bytea)', 'EXECUTE')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'SELECT')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'INSERT')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'UPDATE')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'DELETE')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'TRUNCATE')
       OR EXISTS (
            SELECT 1
              FROM pg_class c
              JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = 'platform_transport'
               AND c.relkind IN ('r', 'p')
               AND (
                    has_table_privilege('star_platform_transport_runtime', c.oid, 'SELECT')
                    OR has_table_privilege('star_platform_transport_runtime', c.oid, 'INSERT')
                    OR has_table_privilege('star_platform_transport_runtime', c.oid, 'UPDATE')
                    OR has_table_privilege('star_platform_transport_runtime', c.oid, 'DELETE')
                    OR has_table_privilege('star_platform_transport_runtime', c.oid, 'TRUNCATE')
               )
       )
       OR has_schema_privilege('star_platform_transport_runtime', 'platform_transport', 'CREATE')
       OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.capture_message_source_certificate()', 'EXECUTE')
       OR NOT EXISTS (
            SELECT 1 FROM pg_trigger
             WHERE tgrelid = 'platform_transport.message_work'::regclass
               AND tgname = 'message_source_certificate_guard'
               AND NOT tgisinternal
       ) THEN
        RAISE EXCEPTION 'native source verification must remain fixed-path and least-privilege';
    END IF;
    IF EXISTS (
        SELECT 1
          FROM pg_proc p
          JOIN pg_namespace n ON n.oid = p.pronamespace
         WHERE n.nspname = 'platform_transport'
           AND has_function_privilege('star_platform_transport_runtime', p.oid, 'EXECUTE')
           AND p.oid NOT IN (
                'platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)'::regprocedure,
                'platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)'::regprocedure,
                'platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)'::regprocedure,
                'platform_transport.verify_origin_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,bytea)'::regprocedure
           )
    ) OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)', 'EXECUTE')
      OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)', 'EXECUTE')
      OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)', 'EXECUTE')
      OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)', 'EXECUTE')
      OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.pull_one(text,uuid,integer)', 'EXECUTE')
      OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)', 'EXECUTE')
      OR has_function_privilege('star_platform_transport_runtime',
            'platform_transport.purge_terminal(text,uuid,integer)', 'EXECUTE') THEN
        RAISE EXCEPTION 'native transport runtime must have exactly the authenticated operation function allowlist';
    END IF;
    IF EXISTS (
        SELECT 1
          FROM pg_proc p
          JOIN pg_namespace n ON n.oid = p.pronamespace
          CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl, acldefault('f', p.proowner))) acl
         WHERE n.nspname = 'platform_transport'
           AND p.proname IN (
                'append_authenticated', 'pull_authenticated',
                'acknowledge_authenticated', 'verify_origin_authenticated',
                'capture_message_source_certificate'
           )
           AND acl.grantee = 0
           AND acl.privilege_type = 'EXECUTE'
    ) THEN
        RAISE EXCEPTION 'PUBLIC must not execute native authenticated transport routines';
    END IF;
    IF EXISTS (
        SELECT 1 FROM pg_proc p
         WHERE p.oid IN (
            'platform_transport.append_authenticated(bytea,bytea,uuid,text,uuid,bytea,bytea)'::regprocedure,
            'platform_transport.pull_authenticated(bytea,bytea,uuid,text,integer)'::regprocedure,
            'platform_transport.acknowledge_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,uuid,bytea)'::regprocedure,
            'platform_transport.verify_origin_authenticated(bytea,bytea,uuid,text,text,uuid,bigint,uuid,bytea)'::regprocedure
         )
           AND (NOT prosecdef
                OR pg_get_userbyid(proowner) <> 'star_platform_transport_owner'
                OR NOT (proconfig @> ARRAY['search_path=pg_catalog, platform_transport, platform_messaging']))
    ) THEN
        RAISE EXCEPTION 'native authenticated routines must use the restricted owner and fixed search path';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM platform_transport.message_work
         WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
           AND tenant_id = '92000000-0000-0000-0000-000000000001'
           AND message_id = '99000000-0000-0000-0000-000000000001'
           AND source_certificate_sha256 IS NULL
    ) THEN
        RAISE EXCEPTION 'pre-attestation message must remain explicitly unverified';
    END IF;
END
$assertions$;

DO $tenant_fixture_present$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM platform_transport.message_work
         WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
           AND tenant_id = '92000000-0000-0000-0000-000000000001'
    ) THEN
        RAISE EXCEPTION 'tenant A fixture is required for a non-vacuous RLS visibility check';
    END IF;
END
$tenant_fixture_present$;

BEGIN;
SELECT set_config('app.platform_deployment_id', '91000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_tenant_id', '92000000-0000-0000-0000-000000000002', TRUE);
SET LOCAL ROLE star_platform_transport_owner;
DO $tenant_rls$
BEGIN
    IF EXISTS (
        SELECT 1 FROM platform_transport.message_work
         WHERE deployment_id = '91000000-0000-0000-0000-000000000001'
           AND tenant_id = '92000000-0000-0000-0000-000000000001'
    ) THEN
        RAISE EXCEPTION 'transport owner RLS exposed another tenant source row';
    END IF;
END
$tenant_rls$;
RESET ROLE;
ROLLBACK;

BEGIN;
SELECT set_config('app.platform_deployment_id', '91000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_tenant_id', '92000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_profile_tenant_id', '92000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_actor_service_id', '93000000-0000-0000-0000-000000000003', TRUE);

SET LOCAL ROLE star_platform_transport_owner;
INSERT INTO platform_transport.service_certificate_identity (
    certificate_sha256, deployment_id, tenant_id, service_id, provider_id,
    provider_version, endpoint_identity, lifecycle_state, registered_by_service_id, registration_request_id
) VALUES (
    decode(repeat('44', 32), 'hex'), '91000000-0000-0000-0000-000000000001', '92000000-0000-0000-0000-000000000001',
    '93000000-0000-0000-0000-000000000001', 'star-rust-log', '1.0', 'native-test', 'active',
    '93000000-0000-0000-0000-000000000003', '97000000-0000-0000-0000-000000000006'
);
RESET ROLE;

SET LOCAL ROLE star_platform_transport_runtime;
DO $live_origin$
DECLARE
    accepted RECORD;
    first_delivery RECORD;
    second_delivery RECORD;
    authorized BOOLEAN;
    -- Fixed constants avoid pgcrypto under the verifier's restricted search_path.
    exact_digest BYTEA := decode('a66b39241bc30964f86cb340f82632d87226b6a6897b49aa1bccb77325949b34', 'hex');
BEGIN
    SELECT * INTO accepted FROM platform_transport.append_authenticated(
        decode(repeat('44', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000002', 'request',
        '99000000-0000-0000-0000-000000000002', convert_to('native-origin-verified-wire', 'UTF8'), exact_digest);
    IF accepted.exact_replay OR accepted.accepted_sequence <> 2 THEN
        RAISE EXCEPTION 'new authenticated append did not allocate its exact durable row';
    END IF;
    SELECT * INTO first_delivery FROM platform_transport.pull_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', 30);
    IF first_delivery.message_id <> '99000000-0000-0000-0000-000000000002'
       OR first_delivery.payload_sha256 <> exact_digest THEN
        RAISE EXCEPTION 'native source verifier did not pull the exact appended wire';
    END IF;
    SELECT platform_transport.verify_origin_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', first_delivery.delivery_log_id,
        first_delivery.delivery_generation, first_delivery.delivery_sequence,
        first_delivery.message_id, first_delivery.payload_sha256) INTO authorized;
    IF NOT authorized THEN RAISE EXCEPTION 'exact current lease and authenticated sender must verify'; END IF;
    SELECT platform_transport.verify_origin_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', first_delivery.delivery_log_id,
        first_delivery.delivery_generation, first_delivery.delivery_sequence,
        first_delivery.message_id, decode(repeat('ef', 32), 'hex')) INTO authorized;
    IF authorized THEN RAISE EXCEPTION 'tampered exact-wire digest unexpectedly verified'; END IF;

    PERFORM platform_transport.acknowledge_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', first_delivery.delivery_log_id,
        first_delivery.delivery_generation, first_delivery.delivery_sequence, first_delivery.lease_id,
        first_delivery.message_id, first_delivery.payload_sha256);
    SELECT platform_transport.verify_origin_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', first_delivery.delivery_log_id,
        first_delivery.delivery_generation, first_delivery.delivery_sequence,
        first_delivery.message_id, first_delivery.payload_sha256) INTO authorized;
    IF authorized THEN RAISE EXCEPTION 'acked delivery must not remain a live source proof'; END IF;

    SELECT * INTO accepted FROM platform_transport.append_authenticated(
        decode(repeat('44', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000002', 'request',
        '99000000-0000-0000-0000-000000000003', convert_to('native-origin-revocation-wire', 'UTF8'),
        decode('406619cc5476c57da4a3d7fd49d02a0225c3a6d4533e13e77a4bdb6b17f87167', 'hex'));
    SELECT * INTO second_delivery FROM platform_transport.pull_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', 30);
    IF second_delivery.message_id <> '99000000-0000-0000-0000-000000000003' THEN
        RAISE EXCEPTION 'second source-proof fixture did not lease its exact message';
    END IF;
END
$live_origin$;
RESET ROLE;
COMMIT;

BEGIN;
SELECT set_config('app.platform_deployment_id', '91000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_tenant_id', '92000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_profile_tenant_id', '92000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_actor_service_id', '93000000-0000-0000-0000-000000000003', TRUE);
SET LOCAL ROLE star_platform_transport_owner;
UPDATE platform_transport.service_certificate_identity
   SET lifecycle_state = 'revoked', revoked_by_service_id = '93000000-0000-0000-0000-000000000003',
       revocation_request_id = '97000000-0000-0000-0000-000000000007', revoked_at = transaction_timestamp()
 WHERE certificate_sha256 = decode(repeat('44', 32), 'hex');
RESET ROLE;
SET LOCAL ROLE star_platform_transport_runtime;
DO $historical_origin$
DECLARE authorized BOOLEAN;
BEGIN
    SELECT platform_transport.verify_origin_authenticated(
        decode(repeat('22', 32), 'hex'), decode(repeat('ab', 32), 'hex'),
        '93000000-0000-0000-0000-000000000001', 'request', 'test.native.auth',
        '95000000-0000-0000-0000-000000000001', 3,
        '99000000-0000-0000-0000-000000000003',
        decode('406619cc5476c57da4a3d7fd49d02a0225c3a6d4533e13e77a4bdb6b17f87167', 'hex')) INTO authorized;
    IF NOT authorized THEN
        RAISE EXCEPTION 'credential revoked after acceptance must preserve historical source proof';
    END IF;
END
$historical_origin$;
RESET ROLE;
COMMIT;
