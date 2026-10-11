-- @cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
-- MERGE (self:File {path:"db/tests/platform-poison-quarantine-schema.sql"})
-- MERGE (receipt:Table {id:"platform_messaging.poison_quarantine_receipt"})
-- MERGE (payload:Table {id:"platform_messaging.poison_quarantine_payload_work"})
-- MERGE (audit:Table {id:"platform_messaging.poison_quarantine_payload_purge_audit"})
-- MERGE (admission_receipt:Table {id:"platform_messaging.admission_receive_quarantine_receipt"})
-- MERGE (ingress:DatabaseRole {id:"star_platform_ingress_owner"})
-- MERGE (purger:DatabaseRole {id:"star_platform_quarantine_purger"})
-- MERGE (self)-[:TESTS]->(receipt)
-- MERGE (self)-[:TESTS]->(payload)
-- MERGE (self)-[:TESTS]->(audit)
-- MERGE (self)-[:TESTS]->(admission_receipt)
-- MERGE (self)-[:TESTS]->(ingress)
-- MERGE (self)-[:TESTS]->(purger)
-- @endcypher

-- Run only against a disposable PostgreSQL database after applying
-- 2026-10-08-platform-poison-quarantine.sql. Synthetic ciphertext bytes below exercise schema
-- constraints only; they are not evidence of KMS encryption or production payload retention.

DO $assertions$
DECLARE
    role_record RECORD;
    protected_table TEXT;
BEGIN
    SELECT * INTO role_record FROM pg_roles WHERE rolname = 'star_platform_ingress_owner';
    IF NOT FOUND OR role_record.rolcanlogin OR role_record.rolsuper OR role_record.rolcreatedb
       OR role_record.rolcreaterole OR role_record.rolreplication OR role_record.rolbypassrls
       OR role_record.rolinherit THEN
        RAISE EXCEPTION 'ingress owner role is not a NOLOGIN, NOINHERIT, least-privilege role';
    END IF;
    SELECT * INTO role_record FROM pg_roles WHERE rolname = 'star_platform_quarantine_purger';
    IF NOT FOUND OR role_record.rolcanlogin OR role_record.rolsuper OR role_record.rolcreatedb
       OR role_record.rolcreaterole OR role_record.rolreplication OR role_record.rolbypassrls
       OR role_record.rolinherit THEN
        RAISE EXCEPTION 'quarantine purger role is not a NOLOGIN, NOINHERIT, least-privilege role';
    END IF;

    FOREACH protected_table IN ARRAY ARRAY[
        'platform_messaging.poison_quarantine_receipt',
        'platform_messaging.poison_quarantine_payload_work',
        'platform_messaging.poison_quarantine_payload_purge_audit',
        'platform_messaging.admission_receive_quarantine_receipt'
    ] LOOP
        IF NOT (SELECT c.relrowsecurity AND c.relforcerowsecurity
                  FROM pg_class AS c WHERE c.oid = protected_table::regclass) THEN
            RAISE EXCEPTION 'FORCE RLS is missing on %', protected_table;
        END IF;
    END LOOP;

    IF NOT has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.poison_quarantine_receipt', 'SELECT')
       OR NOT has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.poison_quarantine_receipt', 'INSERT')
       OR has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.poison_quarantine_receipt', 'UPDATE')
       OR has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.poison_quarantine_receipt', 'DELETE')
       OR has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.poison_quarantine_payload_work', 'DELETE') THEN
        RAISE EXCEPTION 'ingress owner privileges exceed append/write scope';
    END IF;
    IF NOT has_table_privilege('star_platform_quarantine_purger',
            'platform_messaging.poison_quarantine_payload_work', 'DELETE')
       OR has_table_privilege('star_platform_quarantine_purger',
            'platform_messaging.poison_quarantine_receipt', 'INSERT')
       OR has_table_privilege('star_platform_quarantine_purger',
            'platform_messaging.poison_quarantine_payload_purge_audit', 'UPDATE')
       OR has_table_privilege('star_platform_quarantine_purger',
            'platform_messaging.poison_quarantine_payload_purge_audit', 'DELETE') THEN
        RAISE EXCEPTION 'purger owner privileges exceed audited TTL deletion scope';
    END IF;

    IF (SELECT count(*) FROM pg_trigger AS t
          JOIN pg_class AS c ON c.oid = t.tgrelid
          JOIN pg_namespace AS n ON n.oid = c.relnamespace
         WHERE n.nspname = 'platform_messaging' AND NOT t.tgisinternal
           AND t.tgname IN (
               'poison_quarantine_receipt_worm',
               'poison_quarantine_receipt_truncate_guard',
               'poison_quarantine_payload_requires_encrypted_receipt',
               'poison_quarantine_payload_quota_guard',
               'poison_quarantine_payload_audit_worm',
               'poison_quarantine_payload_audit_truncate_guard',
               'poison_quarantine_payload_purge_matches_expired_work'
           )) <> 9 THEN
        RAISE EXCEPTION 'quarantine integrity triggers are incomplete';
    END IF;
END
$assertions$;

DO $admission_permissions$
BEGIN
    IF NOT has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.admission_receive_quarantine_receipt', 'SELECT')
       OR NOT has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.admission_receive_quarantine_receipt', 'INSERT')
       OR has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.admission_receive_quarantine_receipt', 'UPDATE')
       OR has_table_privilege('star_platform_ingress_owner',
            'platform_messaging.admission_receive_quarantine_receipt', 'DELETE') THEN
        RAISE EXCEPTION 'admission quarantine owner privileges exceed append-only scope';
    END IF;
END
$admission_permissions$;

BEGIN;
SET LOCAL ROLE star_platform_ingress_owner;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', true);
INSERT INTO platform_messaging.poison_quarantine_receipt (
    deployment_id, stream_generation, stream_name, stream_sequence,
    fault_code, classifier_version, parser_version, payload_bytes, payload_sha256, retention_policy
) VALUES (
    '10000000-0000-0000-0000-000000000001',
    '20000000-0000-0000-0000-000000000001', 'STAR_RESULTS', 1,
    'invalid_capacity_result_event', 1, 2, 3, decode(repeat('00', 32), 'hex'), 'metadata_only'
);
COMMIT;

BEGIN;
SET LOCAL ROLE star_platform_ingress_owner;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000002', true);
DO $rls$
BEGIN
    IF EXISTS (
        SELECT 1 FROM platform_messaging.poison_quarantine_receipt
        WHERE deployment_id = '10000000-0000-0000-0000-000000000001'
    ) THEN
        RAISE EXCEPTION 'deployment RLS exposed another deployment receipt';
    END IF;
END
$rls$;
COMMIT;

BEGIN;
GRANT UPDATE ON platform_messaging.poison_quarantine_receipt TO star_platform_ingress_owner;
SET LOCAL ROLE star_platform_ingress_owner;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', true);
DO $worm$
BEGIN
    BEGIN
        UPDATE platform_messaging.poison_quarantine_receipt
           SET fault_code = 'malformed_capacity_result_json'
         WHERE stream_sequence = 1;
        RAISE EXCEPTION 'append-only receipt trigger did not reject UPDATE';
    EXCEPTION WHEN SQLSTATE '55000' THEN
        NULL;
    END;
END
$worm$;
ROLLBACK;

BEGIN;
SET LOCAL ROLE star_platform_ingress_owner;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', true);
DO $metadata_body_guard$
BEGIN
    BEGIN
        INSERT INTO platform_messaging.poison_quarantine_payload_work (
            deployment_id, stream_generation, stream_name, stream_sequence,
            encrypted_blob, encrypted_blob_sha256, encoded_bytes,
            retained_at, expires_at
        ) VALUES (
            '10000000-0000-0000-0000-000000000001',
            '20000000-0000-0000-0000-000000000001', 'STAR_RESULTS', 1,
            decode('cafe', 'hex'), decode(repeat('11', 32), 'hex'), 2,
            transaction_timestamp(), transaction_timestamp() + interval '24 hours'
        );
        RAISE EXCEPTION 'metadata-only receipt accepted a retained payload';
    EXCEPTION WHEN SQLSTATE '23514' THEN
        NULL;
    END;
END
$metadata_body_guard$;
ROLLBACK;

BEGIN;
SET LOCAL ROLE star_platform_ingress_owner;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', true);
INSERT INTO platform_messaging.poison_quarantine_receipt (
    deployment_id, stream_generation, stream_name, stream_sequence,
    fault_code, classifier_version, parser_version, payload_bytes, payload_sha256, retention_policy
) VALUES (
    '10000000-0000-0000-0000-000000000001',
    '20000000-0000-0000-0000-000000000001', 'STAR_RESULTS', 2,
    'invalid_capacity_result_event', 1, 2, 3, decode(repeat('22', 32), 'hex'), 'encrypted_24h'
);
INSERT INTO platform_messaging.poison_quarantine_payload_work (
    deployment_id, stream_generation, stream_name, stream_sequence,
    encrypted_blob, encrypted_blob_sha256, encoded_bytes,
    retained_at, expires_at
) VALUES (
    '10000000-0000-0000-0000-000000000001',
    '20000000-0000-0000-0000-000000000001', 'STAR_RESULTS', 2,
    decode('cafe', 'hex'), decode(repeat('33', 32), 'hex'), 2,
    transaction_timestamp(), transaction_timestamp() + interval '24 hours'
);
COMMIT;

BEGIN;
SET LOCAL ROLE star_platform_quarantine_purger;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', true);
DO $early_purge_guard$
BEGIN
    BEGIN
        INSERT INTO platform_messaging.poison_quarantine_payload_purge_audit (
            deployment_id, stream_generation, stream_name, stream_sequence,
            encrypted_blob_sha256, encoded_bytes, purge_reason
        ) VALUES (
            '10000000-0000-0000-0000-000000000001',
            '20000000-0000-0000-0000-000000000001', 'STAR_RESULTS', 2,
            decode(repeat('33', 32), 'hex'), 2, 'ttl_expired'
        );
        RAISE EXCEPTION 'purge audit accepted a payload before TTL expiry';
    EXCEPTION WHEN SQLSTATE '23514' THEN
        NULL;
    END;
END
$early_purge_guard$;
COMMIT;

-- Move only this disposable fixture into an expired state, retaining the exact 24-hour interval.
UPDATE platform_messaging.poison_quarantine_payload_work
   SET retained_at = transaction_timestamp() - interval '25 hours',
       expires_at = transaction_timestamp() - interval '1 hour'
 WHERE deployment_id = '10000000-0000-0000-0000-000000000001'
   AND stream_generation = '20000000-0000-0000-0000-000000000001'
   AND stream_name = 'STAR_RESULTS' AND stream_sequence = 2;

BEGIN;
SET LOCAL ROLE star_platform_quarantine_purger;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000001', true);
INSERT INTO platform_messaging.poison_quarantine_payload_purge_audit (
    deployment_id, stream_generation, stream_name, stream_sequence,
    encrypted_blob_sha256, encoded_bytes, purge_reason
)
SELECT deployment_id, stream_generation, stream_name, stream_sequence,
       encrypted_blob_sha256, encoded_bytes, 'ttl_expired'
  FROM platform_messaging.poison_quarantine_payload_work
 WHERE deployment_id = '10000000-0000-0000-0000-000000000001'
   AND stream_generation = '20000000-0000-0000-0000-000000000001'
   AND stream_name = 'STAR_RESULTS' AND stream_sequence = 2;
DELETE FROM platform_messaging.poison_quarantine_payload_work
 WHERE deployment_id = '10000000-0000-0000-0000-000000000001'
   AND stream_generation = '20000000-0000-0000-0000-000000000001'
   AND stream_name = 'STAR_RESULTS' AND stream_sequence = 2;
COMMIT;

DO $purge_evidence$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM platform_messaging.poison_quarantine_payload_purge_audit
        WHERE deployment_id = '10000000-0000-0000-0000-000000000001'
          AND stream_generation = '20000000-0000-0000-0000-000000000001'
          AND stream_name = 'STAR_RESULTS' AND stream_sequence = 2
    ) OR EXISTS (
        SELECT 1 FROM platform_messaging.poison_quarantine_payload_work
        WHERE deployment_id = '10000000-0000-0000-0000-000000000001'
          AND stream_generation = '20000000-0000-0000-0000-000000000001'
          AND stream_name = 'STAR_RESULTS' AND stream_sequence = 2
    ) THEN
        RAISE EXCEPTION 'expired encrypted payload purge did not preserve atomic audit evidence';
    END IF;
END
$purge_evidence$;

BEGIN;
SET LOCAL ROLE star_platform_ingress_owner;
SELECT set_config('app.platform_deployment_id', '10000000-0000-0000-0000-000000000003', true);
DO $quota$
DECLARE
    seq INTEGER;
    quota_rejected BOOLEAN := FALSE;
    payload_bytes BYTEA := decode(repeat('aa', 204800), 'hex');
BEGIN
    BEGIN
        FOR seq IN 1..21 LOOP
            INSERT INTO platform_messaging.poison_quarantine_receipt (
                deployment_id, stream_generation, stream_name, stream_sequence,
                fault_code, classifier_version, parser_version, payload_bytes,
                payload_sha256, retention_policy
            ) VALUES (
                '10000000-0000-0000-0000-000000000003',
                '20000000-0000-0000-0000-000000000003', 'STAR_RESULTS', seq,
                'invalid_capacity_result_event', 1, 2, 0,
                decode(repeat('44', 32), 'hex'), 'encrypted_24h'
            );
            INSERT INTO platform_messaging.poison_quarantine_payload_work (
                deployment_id, stream_generation, stream_name, stream_sequence,
                encrypted_blob, encrypted_blob_sha256, encoded_bytes, retained_at, expires_at
            ) VALUES (
                '10000000-0000-0000-0000-000000000003',
                '20000000-0000-0000-0000-000000000003', 'STAR_RESULTS', seq,
                payload_bytes, decode(repeat('55', 32), 'hex'), octet_length(payload_bytes),
                transaction_timestamp(), transaction_timestamp() + interval '24 hours'
            );
        END LOOP;
    EXCEPTION WHEN SQLSTATE '23514' THEN
        quota_rejected := TRUE;
    END;
    IF NOT quota_rejected THEN
        RAISE EXCEPTION 'database quarantine quota trigger did not reject an over-quota insert';
    END IF;
END
$quota$;
ROLLBACK;

SELECT 'platform poison quarantine schema checks passed' AS result;
