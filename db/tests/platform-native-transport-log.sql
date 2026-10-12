-- @cypher schema=1 source_sha256=47c9814d8419defc10d4d574f43f3aa7f289e88694b3d645ee9fc5ec7db19391
-- MERGE (self:File {path:"db/tests/platform-native-transport-log.sql"})
-- MERGE (log_master:Table {id:"platform_transport.log_master"})
-- MERGE (log_state:Table {id:"platform_transport.log_state"})
-- MERGE (message_work:Table {id:"platform_transport.message_work"})
-- MERGE (receipt:Table {id:"platform_transport.delivery_receipt"})
-- MERGE (outbox:Table {id:"platform_transport.receipt_outbox"})
-- MERGE (append:Function {id:"platform_transport.append_message"})
-- MERGE (pull:Function {id:"platform_transport.pull_one"})
-- MERGE (ack:Function {id:"platform_transport.acknowledge_one"})
-- MERGE (purge:Function {id:"platform_transport.purge_terminal"})
-- MERGE (owner:DatabaseRole {id:"star_platform_transport_owner"})
-- MERGE (runtime:DatabaseRole {id:"star_platform_transport_runtime"})
-- MERGE (self)-[:TESTS]->(log_master)
-- MERGE (self)-[:TESTS]->(log_state)
-- MERGE (self)-[:TESTS]->(message_work)
-- MERGE (self)-[:TESTS]->(receipt)
-- MERGE (self)-[:TESTS]->(outbox)
-- MERGE (self)-[:TESTS]->(append)
-- MERGE (self)-[:TESTS]->(pull)
-- MERGE (self)-[:TESTS]->(ack)
-- MERGE (self)-[:TESTS]->(purge)
-- MERGE (self)-[:TESTS]->(owner)
-- MERGE (self)-[:TESTS]->(runtime)
-- @endcypher

DO $assertions$
DECLARE
    table_name TEXT;
    rls_enabled BOOLEAN;
    rls_forced BOOLEAN;
    restricted BOOLEAN;
    routine TEXT;
BEGIN
    FOREACH routine IN ARRAY ARRAY[
        'platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)',
        'platform_transport.pull_one(text,uuid,integer)',
        'platform_transport.acknowledge_one(text,uuid,bigint,uuid,uuid,bytea)',
        'platform_transport.purge_terminal(text,uuid,integer)'
    ] LOOP
        IF NOT EXISTS (
            SELECT 1 FROM pg_proc p
             WHERE p.oid = routine::regprocedure
               AND p.prosecdef
               AND pg_get_userbyid(p.proowner) = 'star_platform_transport_owner'
               AND p.proconfig @> ARRAY['search_path=pg_catalog, platform_transport']
               AND NOT EXISTS (
                    SELECT 1 FROM aclexplode(p.proacl) a
                     WHERE a.grantee = 0 AND a.privilege_type = 'EXECUTE'
               )
        ) THEN
            RAISE EXCEPTION 'native transport routine % is not fixed-path, owner-scoped or non-public', routine;
        END IF;
    END LOOP;

    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO restricted FROM pg_roles WHERE rolname = 'star_platform_transport_owner';
    IF restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'native transport owner role is missing or unsafe';
    END IF;
    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO restricted FROM pg_roles WHERE rolname = 'star_platform_transport_runtime';
    IF restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'native transport runtime role is missing or unsafe';
    END IF;

    FOREACH table_name IN ARRAY ARRAY[
        'platform_transport.log_master', 'platform_transport.log_state',
        'platform_transport.message_work', 'platform_transport.delivery_receipt',
        'platform_transport.receipt_outbox'
    ] LOOP
        SELECT c.relrowsecurity, c.relforcerowsecurity INTO rls_enabled, rls_forced
          FROM pg_class c WHERE c.oid = table_name::regclass;
        IF rls_enabled IS DISTINCT FROM TRUE OR rls_forced IS DISTINCT FROM TRUE THEN
            RAISE EXCEPTION 'native transport table % must enable FORCE RLS', table_name;
        END IF;
        IF NOT EXISTS (
            SELECT 1 FROM pg_policy p
             WHERE p.polrelid = table_name::regclass AND p.polname IS NOT NULL
               AND EXISTS (
                    SELECT 1 FROM unnest(p.polroles) granted_role
                     WHERE granted_role = 'star_platform_transport_owner'::regrole::OID
               )
        ) THEN
            RAISE EXCEPTION 'native transport table % has no owner scope policy', table_name;
        END IF;
    END LOOP;

    IF has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'SELECT')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.message_work', 'UPDATE')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.delivery_receipt', 'INSERT')
       OR has_table_privilege('star_platform_transport_runtime', 'platform_transport.receipt_outbox', 'SELECT')
       OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.append_message(text,uuid,uuid,text,uuid,bytea,bytea)', 'EXECUTE')
       OR NOT has_function_privilege('star_platform_transport_runtime',
            'platform_transport.pull_one(text,uuid,integer)', 'EXECUTE') THEN
        RAISE EXCEPTION 'native transport runtime must call bounded routines without direct table access';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.delivery_receipt'::regclass
          AND tgname='platform_transport_receipt_no_update_delete' AND NOT tgisinternal
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.delivery_receipt'::regclass
          AND tgname='platform_transport_receipt_no_truncate' AND NOT tgisinternal
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.delivery_receipt'::regclass
          AND tgname='platform_transport_receipt_outbox' AND NOT tgisinternal
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.receipt_outbox'::regclass
          AND tgname='platform_transport_outbox_no_update_delete' AND NOT tgisinternal
    ) OR NOT EXISTS (
        SELECT 1 FROM pg_trigger WHERE tgrelid='platform_transport.receipt_outbox'::regclass
          AND tgname='platform_transport_outbox_no_truncate' AND NOT tgisinternal
    ) THEN
        RAISE EXCEPTION 'native transport receipts and BI Outbox must be append-only and atomically linked';
    END IF;
    IF EXISTS (
        SELECT 1 FROM information_schema.columns AS columns_catalog
         WHERE columns_catalog.table_schema='platform_transport'
           AND columns_catalog.table_name='receipt_outbox'
           AND columns_catalog.column_name IN ('wire', 'payload', 'credential', 'private_key', 'token')
    ) THEN
        RAISE EXCEPTION 'native transport BI Outbox must contain metadata only';
    END IF;
END
$assertions$;

BEGIN;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_tenant_id', '20000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_service_id', '30000000-0000-0000-0000-000000000001', TRUE);

INSERT INTO platform_transport.log_master (
    deployment_id, tenant_id, log_id, generation, sender_service_id, recipient_service_id,
    message_kind, payload_budget_bytes, terminal_retention_hours, max_delivery_attempts, lifecycle
) VALUES
    ('10000000-0000-0000-0000-000000000001', '20000000-0000-0000-0000-000000000001',
     'test.schedule.request', '40000000-0000-0000-0000-000000000001',
     '30000000-0000-0000-0000-000000000001', '30000000-0000-0000-0000-000000000002',
     'schedule.request', 16384, 24, 4, 'active'),
    ('10000000-0000-0000-0000-000000000001', '20000000-0000-0000-0000-000000000001',
     'test.schedule.result', '40000000-0000-0000-0000-000000000002',
     '30000000-0000-0000-0000-000000000001', '30000000-0000-0000-0000-000000000002',
     'schedule.result', 16384, 24, 4, 'active')
ON CONFLICT DO NOTHING;

CREATE TEMP TABLE native_transport_test_lease (sequence BIGINT PRIMARY KEY, old_lease UUID NOT NULL);
GRANT SELECT, INSERT ON native_transport_test_lease TO star_platform_transport_runtime;
SET LOCAL ROLE star_platform_transport_runtime;

DO $append_pull_ack$
DECLARE
    accepted RECORD;
    replayed RECORD;
    delivery RECORD;
    acked BOOLEAN;
BEGIN
    SELECT * INTO accepted FROM platform_transport.append_message(
        'test.schedule.request', '40000000-0000-0000-0000-000000000001',
        '30000000-0000-0000-0000-000000000002', 'schedule.request',
        '50000000-0000-0000-0000-000000000001', convert_to('bounded-first-wire', 'UTF8'), decode('92949fa2139a2adf6e5509cfc124d39e29d51775e1a2252ff7856ace350f20eb', 'hex'));
    IF accepted.accepted_sequence <> 1 OR accepted.exact_replay THEN
        RAISE EXCEPTION 'first append must allocate the first durable cursor';
    END IF;
    SELECT * INTO replayed FROM platform_transport.append_message(
        'test.schedule.request', '40000000-0000-0000-0000-000000000001',
        '30000000-0000-0000-0000-000000000002', 'schedule.request',
        '50000000-0000-0000-0000-000000000001', convert_to('bounded-first-wire', 'UTF8'), decode('92949fa2139a2adf6e5509cfc124d39e29d51775e1a2252ff7856ace350f20eb', 'hex'));
    IF replayed.accepted_sequence <> accepted.accepted_sequence OR NOT replayed.exact_replay THEN
        RAISE EXCEPTION 'exact append replay must return its original cursor';
    END IF;
    BEGIN
        PERFORM * FROM platform_transport.append_message(
            'test.schedule.request', '40000000-0000-0000-0000-000000000001',
            '30000000-0000-0000-0000-000000000002', 'schedule.request',
            '50000000-0000-0000-0000-000000000001', convert_to('altered-wire', 'UTF8'), decode('e4673379c5823e92adced490ed4c0fe960edcbbbd112fc8b6106d772d22ad142', 'hex'));
        RAISE EXCEPTION 'altered append replay unexpectedly succeeded';
    EXCEPTION WHEN SQLSTATE '22000' THEN NULL;
    END;
    PERFORM platform_transport.append_message(
        'test.schedule.request', '40000000-0000-0000-0000-000000000001',
        '30000000-0000-0000-0000-000000000002', 'schedule.request',
        '50000000-0000-0000-0000-000000000002', convert_to('fenced-wire', 'UTF8'), decode('03b8a1bdd96e625f09313746cca86a7e4c4f2b60d51812a3194c7b47fc5067ca', 'hex'));
    PERFORM set_config('app.platform_service_id', '30000000-0000-0000-0000-000000000002', TRUE);
    SELECT * INTO delivery FROM platform_transport.pull_one(
        'test.schedule.request', '40000000-0000-0000-0000-000000000001', 30);
    IF delivery.message_id <> '50000000-0000-0000-0000-000000000001'
       OR delivery.wire <> convert_to('bounded-first-wire', 'UTF8') OR delivery.attempt_count <> 1 THEN
        RAISE EXCEPTION 'pull must return the exact stored bounded wire and first lease';
    END IF;
    INSERT INTO pg_temp.native_transport_test_lease VALUES (delivery.delivery_sequence, delivery.lease_id);
    SELECT platform_transport.acknowledge_one(
        delivery.delivery_log_id, delivery.delivery_generation, delivery.delivery_sequence,
        delivery.lease_id, delivery.message_id, delivery.payload_sha256) INTO acked;
    IF NOT acked THEN RAISE EXCEPTION 'owner-committed delivery ACK must persist'; END IF;
    SELECT platform_transport.acknowledge_one(
        delivery.delivery_log_id, delivery.delivery_generation, delivery.delivery_sequence,
        delivery.lease_id, delivery.message_id, delivery.payload_sha256) INTO acked;
    IF NOT acked THEN RAISE EXCEPTION 'exact ACK replay must be idempotent'; END IF;
    SELECT * INTO delivery FROM platform_transport.pull_one(
        'test.schedule.request', '40000000-0000-0000-0000-000000000001', 30);
    IF delivery.message_id <> '50000000-0000-0000-0000-000000000002' THEN
        RAISE EXCEPTION 'second message must be independently claimable after first ACK';
    END IF;
    INSERT INTO pg_temp.native_transport_test_lease VALUES (delivery.delivery_sequence, delivery.lease_id);

    BEGIN
        PERFORM 1 FROM platform_transport.message_work;
        RAISE EXCEPTION 'runtime role unexpectedly read raw payload table';
    EXCEPTION WHEN insufficient_privilege THEN NULL;
    END;
END
$append_pull_ack$;

RESET ROLE;
UPDATE platform_transport.message_work
   SET lease_expires_at = clock_timestamp() - INTERVAL '1 second'
 WHERE deployment_id='10000000-0000-0000-0000-000000000001'
   AND tenant_id='20000000-0000-0000-0000-000000000001'
   AND log_id='test.schedule.request' AND sequence=2;
SET LOCAL ROLE star_platform_transport_runtime;

DO $lease_fence$
DECLARE
    old_lease UUID;
    delivery RECORD;
    acked BOOLEAN;
BEGIN
    SELECT n.old_lease INTO old_lease FROM pg_temp.native_transport_test_lease n WHERE n.sequence = 2;
    SELECT * INTO delivery FROM platform_transport.pull_one(
        'test.schedule.request', '40000000-0000-0000-0000-000000000001', 30);
    IF delivery.message_id <> '50000000-0000-0000-0000-000000000002' OR delivery.lease_id = old_lease THEN
        RAISE EXCEPTION 'expired work must be re-leased with a fresh fence';
    END IF;
    BEGIN
        PERFORM platform_transport.acknowledge_one(
            delivery.delivery_log_id, delivery.delivery_generation, delivery.delivery_sequence,
            old_lease, delivery.message_id, delivery.payload_sha256);
        RAISE EXCEPTION 'stale lease ACK unexpectedly succeeded';
    EXCEPTION WHEN SQLSTATE '40001' THEN NULL;
    END;
    SELECT platform_transport.acknowledge_one(
        delivery.delivery_log_id, delivery.delivery_generation, delivery.delivery_sequence,
        delivery.lease_id, delivery.message_id, delivery.payload_sha256) INTO acked;
    IF NOT acked THEN RAISE EXCEPTION 'current lease ACK must succeed'; END IF;

    PERFORM set_config('app.platform_tenant_id', '20000000-0000-0000-0000-000000000099', TRUE);
    BEGIN
        PERFORM * FROM platform_transport.pull_one(
            'test.schedule.request', '40000000-0000-0000-0000-000000000001', 30);
        RAISE EXCEPTION 'cross-tenant transport pull unexpectedly succeeded';
    EXCEPTION WHEN SQLSTATE '42501' THEN NULL;
    END;
END
$lease_fence$;

RESET ROLE;
SELECT set_config('app.platform_tenant_id', '20000000-0000-0000-0000-000000000001', TRUE);
SELECT set_config('app.platform_service_id', '30000000-0000-0000-0000-000000000001', TRUE);
SET LOCAL ROLE star_platform_transport_runtime;
DO $capacity_and_retention$
DECLARE
    accepted RECORD;
    delivery RECORD;
    purged INTEGER;
BEGIN
    BEGIN
        PERFORM platform_transport.purge_terminal(
            'test.schedule.result', '40000000-0000-0000-0000-000000000002', NULL);
        RAISE EXCEPTION 'NULL terminal purge batch limit unexpectedly succeeded';
    EXCEPTION WHEN SQLSTATE '22023' THEN NULL;
    END;
    SELECT * INTO accepted FROM platform_transport.append_message(
        'test.schedule.result', '40000000-0000-0000-0000-000000000002',
        '30000000-0000-0000-0000-000000000002', 'schedule.result',
        '50000000-0000-0000-0000-000000000003', decode(repeat('ab', 16384), 'hex'), decode('c44e98eae8e50a903d658422c0482d50e7c764403600877ba974d8f5e9da2f3f', 'hex'));
    IF accepted.accepted_sequence <> 1 OR accepted.exact_replay THEN
        RAISE EXCEPTION '16 KiB bounded wire must fit its configured payload budget';
    END IF;
    BEGIN
        PERFORM * FROM platform_transport.append_message(
            'test.schedule.result', '40000000-0000-0000-0000-000000000002',
            '30000000-0000-0000-0000-000000000002', 'schedule.result',
            '50000000-0000-0000-0000-000000000004', decode('aa', 'hex'), decode('bceef655b5a034911f1c3718ce056531b45ef03b4c7b1f15629e867294011a7d', 'hex'));
        RAISE EXCEPTION 'payload byte budget unexpectedly allowed an over-cap append';
    EXCEPTION WHEN SQLSTATE '54000' THEN NULL;
    END;
    PERFORM set_config('app.platform_service_id', '30000000-0000-0000-0000-000000000002', TRUE);
    SELECT * INTO delivery FROM platform_transport.pull_one(
        'test.schedule.result', '40000000-0000-0000-0000-000000000002', 30);
    PERFORM platform_transport.acknowledge_one(
        delivery.delivery_log_id, delivery.delivery_generation, delivery.delivery_sequence,
        delivery.lease_id, delivery.message_id, delivery.payload_sha256);
END
$capacity_and_retention$;

RESET ROLE;
UPDATE platform_transport.message_work
   SET acked_at = clock_timestamp() - INTERVAL '25 hours'
 WHERE deployment_id='10000000-0000-0000-0000-000000000001'
   AND tenant_id='20000000-0000-0000-0000-000000000001'
   AND log_id='test.schedule.result' AND sequence=1;
SET LOCAL ROLE star_platform_transport_runtime;
DO $purge_replay$
DECLARE
    purged INTEGER;
    replayed RECORD;
BEGIN
    SELECT platform_transport.purge_terminal(
        'test.schedule.result', '40000000-0000-0000-0000-000000000002', 1) INTO purged;
    IF purged <> 1 THEN RAISE EXCEPTION 'terminal payload past the configured TTL must be purged'; END IF;
    PERFORM set_config('app.platform_service_id', '30000000-0000-0000-0000-000000000001', TRUE);
    SELECT * INTO replayed FROM platform_transport.append_message(
        'test.schedule.result', '40000000-0000-0000-0000-000000000002',
        '30000000-0000-0000-0000-000000000002', 'schedule.result',
        '50000000-0000-0000-0000-000000000003', decode(repeat('ab', 16384), 'hex'), decode('c44e98eae8e50a903d658422c0482d50e7c764403600877ba974d8f5e9da2f3f', 'hex'));
    IF NOT replayed.exact_replay OR replayed.accepted_sequence <> 1 THEN
        RAISE EXCEPTION 'append after payload retention must return its permanent original receipt';
    END IF;
END
$purge_replay$;
RESET ROLE;

DO $final_assertions$
DECLARE
    bytes_in_use BIGINT;
    work_count INTEGER;
    accepted_count INTEGER;
    acked_count INTEGER;
    outbox_count INTEGER;
BEGIN
    SELECT payload_bytes_in_use INTO bytes_in_use FROM platform_transport.log_state
     WHERE deployment_id='10000000-0000-0000-0000-000000000001'
       AND tenant_id='20000000-0000-0000-0000-000000000001'
       AND log_id='test.schedule.result' AND generation='40000000-0000-0000-0000-000000000002';
    SELECT count(*)::INTEGER INTO work_count FROM platform_transport.message_work
     WHERE log_id='test.schedule.result' AND generation='40000000-0000-0000-0000-000000000002';
    SELECT count(*)::INTEGER INTO accepted_count FROM platform_transport.delivery_receipt WHERE action='accepted';
    SELECT count(*)::INTEGER INTO acked_count FROM platform_transport.delivery_receipt WHERE action='acked';
    SELECT count(*)::INTEGER INTO outbox_count FROM platform_transport.receipt_outbox;
    IF bytes_in_use <> 0 OR work_count <> 0 OR accepted_count <> 3 OR acked_count <> 3 OR outbox_count <> 6 THEN
        RAISE EXCEPTION 'native transport payload retention or append-only receipt conservation failed';
    END IF;
END
$final_assertions$;
ROLLBACK;
